#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
//! Typed CST wrapper code generation.
//!
//! This module is an alpha generator target for Rust-native typed CST handles.
//! It emits wrappers over `AdzeDocument` node IDs and edge field metadata, but
//! is not yet wired into generated grammar crates.

use adze_ir::{Grammar, Symbol, SymbolId, TokenPattern};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};

/// Generate typed CST wrapper code from grammar metadata.
pub struct TypedCstGenerator<'a> {
    grammar: &'a Grammar,
}

#[derive(Clone, Debug)]
struct WrapperSpec {
    ident: Ident,
    kind_name: String,
}

#[derive(Clone, Debug)]
struct FieldAccessor {
    method_ident: Ident,
    field_name: String,
    target_ident: Ident,
}

impl<'a> TypedCstGenerator<'a> {
    /// Create a typed CST generator for one grammar.
    pub fn new(grammar: &'a Grammar) -> Self {
        Self { grammar }
    }

    /// Generate a `syntax` module containing typed CST wrappers.
    #[must_use]
    pub fn generate(&self) -> TokenStream {
        let specs = self.wrapper_specs();
        let accessors = self.field_accessors(&specs);
        let root_fn = self.root_constructor(&specs);

        let wrappers = specs.values().map(|spec| {
            let ident = &spec.ident;
            let kind_name = &spec.kind_name;
            let wrapper_accessors = accessors
                .get(&spec.ident.to_string())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|accessor| {
                    let method_ident = accessor.method_ident;
                    let field_name = accessor.field_name;
                    let target_ident = accessor.target_ident;
                    quote! {
                        pub fn #method_ident(&self) -> Option<#target_ident<'doc>> {
                            let edge = self.edge_by_field_name(#field_name)?;
                            #target_ident::cast(self.document, edge.child_id())
                        }
                    }
                });

            quote! {
                #[derive(Clone, Copy, Debug)]
                pub struct #ident<'doc> {
                    document: &'doc AdzeDocument,
                    id: NodeId,
                }

                impl<'doc> #ident<'doc> {
                    pub fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
                        node_has_kind(document, id, #kind_name).then_some(Self { document, id })
                    }

                    #(#wrapper_accessors)*
                }

                impl<'doc> SyntaxNode<'doc> for #ident<'doc> {
                    fn document(&self) -> &'doc AdzeDocument {
                        self.document
                    }

                    fn node_id(&self) -> NodeId {
                        self.id
                    }
                }
            }
        });

        quote! {
            pub mod syntax {
                use ::adze::document::{AdzeDocument, NodeId, SyntaxNode};

                #root_fn

                #(#wrappers)*

                fn node_has_kind(document: &AdzeDocument, id: NodeId, expected: &str) -> bool {
                    document.tree().node(id).and_then(|node| node.kind_name()) == Some(expected)
                }
            }
        }
    }

    fn root_constructor(&self, specs: &BTreeMap<SymbolId, WrapperSpec>) -> TokenStream {
        let source_file_id = self.grammar.find_symbol_by_name("source_file");
        let Some(spec) = source_file_id.and_then(|id| specs.get(&id)) else {
            return quote! {};
        };

        let root_ident = &spec.ident;
        quote! {
            pub fn source_file(document: &AdzeDocument) -> Option<#root_ident<'_>> {
                #root_ident::cast(document, document.tree().root_id())
            }
        }
    }

    fn wrapper_specs(&self) -> BTreeMap<SymbolId, WrapperSpec> {
        let mut specs = BTreeMap::new();
        let mut used_idents = BTreeSet::new();
        let field_target_terminals = self.field_target_terminals();

        for (symbol_id, name) in &self.grammar.rule_names {
            if name.starts_with('_') {
                continue;
            }

            specs.insert(
                *symbol_id,
                WrapperSpec {
                    ident: unique_type_ident(name, "SyntaxNode", *symbol_id, &mut used_idents),
                    kind_name: name.clone(),
                },
            );
        }

        for (symbol_id, token) in &self.grammar.tokens {
            if token.name.starts_with('_') && !field_target_terminals.contains(symbol_id) {
                continue;
            }

            let kind_name = match &token.pattern {
                TokenPattern::String(text) if token.name == format!("\"{text}\"") => {
                    token.name.clone()
                }
                TokenPattern::String(text) => text.clone(),
                TokenPattern::Regex(_) => token.name.clone(),
            };

            specs.insert(
                *symbol_id,
                WrapperSpec {
                    ident: unique_type_ident(
                        &format!("{}_token", token.name),
                        "Token",
                        *symbol_id,
                        &mut used_idents,
                    ),
                    kind_name,
                },
            );
        }

        specs
    }

    fn field_target_terminals(&self) -> BTreeSet<SymbolId> {
        let mut terminals = BTreeSet::new();

        for rules in self.grammar.rules.values() {
            for rule in rules {
                for (_, position) in &rule.fields {
                    if let Some(symbol) = rule.rhs.get(*position) {
                        collect_terminal_symbols(symbol, &mut terminals);
                    }
                }
            }
        }

        terminals
    }

    fn field_accessors(
        &self,
        specs: &BTreeMap<SymbolId, WrapperSpec>,
    ) -> BTreeMap<String, Vec<FieldAccessor>> {
        let mut by_wrapper = BTreeMap::<String, Vec<FieldAccessor>>::new();
        let mut seen = BTreeSet::<(String, String)>::new();

        for (lhs, rules) in &self.grammar.rules {
            let Some(owner_spec) = specs.get(lhs) else {
                continue;
            };

            for rule in rules {
                for (field_id, position) in &rule.fields {
                    let Some(field_name) = self.grammar.fields.get(field_id) else {
                        continue;
                    };
                    let Some(symbol) = rule.rhs.get(*position) else {
                        continue;
                    };
                    let Some(target_ident) = symbol_wrapper_ident(symbol, specs) else {
                        continue;
                    };

                    let owner = owner_spec.ident.to_string();
                    if !seen.insert((owner.clone(), field_name.clone())) {
                        continue;
                    }

                    by_wrapper.entry(owner).or_default().push(FieldAccessor {
                        method_ident: method_ident(field_name, "field"),
                        field_name: field_name.clone(),
                        target_ident,
                    });
                }
            }
        }

        by_wrapper
    }
}

fn collect_terminal_symbols(symbol: &Symbol, terminals: &mut BTreeSet<SymbolId>) {
    match symbol {
        Symbol::Terminal(id) => {
            terminals.insert(*id);
        }
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            collect_terminal_symbols(inner, terminals);
        }
        Symbol::Choice(choices) | Symbol::Sequence(choices) => {
            for choice in choices {
                collect_terminal_symbols(choice, terminals);
            }
        }
        Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon => {}
    }
}

fn symbol_wrapper_ident(symbol: &Symbol, specs: &BTreeMap<SymbolId, WrapperSpec>) -> Option<Ident> {
    match symbol {
        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => {
            specs.get(id).map(|spec| spec.ident.clone())
        }
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            symbol_wrapper_ident(inner, specs)
        }
        Symbol::Choice(choices) => choices
            .iter()
            .find_map(|choice| symbol_wrapper_ident(choice, specs)),
        Symbol::Sequence(_) | Symbol::Epsilon => None,
    }
}

fn unique_type_ident(
    name: &str,
    fallback: &str,
    symbol_id: SymbolId,
    used: &mut BTreeSet<String>,
) -> Ident {
    let base = pascal_case_ident(name, fallback);
    let mut candidate = base.clone();
    if !used.insert(candidate.clone()) {
        candidate = format!("{base}Symbol{}", symbol_id.0);
        let mut collision = 1usize;
        while !used.insert(candidate.clone()) {
            candidate = format!("{base}Symbol{}_{collision}", symbol_id.0);
            collision += 1;
        }
    }

    Ident::new(&candidate, Span::call_site())
}

fn method_ident(name: &str, fallback: &str) -> Ident {
    Ident::new(&snake_case_ident(name, fallback), Span::call_site())
}

fn pascal_case_ident(name: &str, fallback: &str) -> String {
    let mut out = String::new();
    let mut at_word_start = true;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if at_word_start {
                out.push(ch.to_ascii_uppercase());
                at_word_start = false;
            } else {
                out.push(ch);
            }
        } else {
            at_word_start = true;
        }
    }

    if out.is_empty() {
        out.push_str(fallback);
    }

    if out
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(true)
    {
        out.insert(0, 'N');
    }

    if is_reserved_ident(&out) {
        out.push_str("Node");
    }

    out
}

fn snake_case_ident(name: &str, fallback: &str) -> String {
    let mut out = String::new();
    let mut previous_was_separator = true;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !out.is_empty() {
            out.push('_');
            previous_was_separator = true;
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        out.push_str(fallback);
    }

    if out
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(true)
    {
        out.insert(0, '_');
    }

    if is_reserved_ident(&out) {
        out.push('_');
    }

    out
}

fn is_reserved_ident(ident: &str) -> bool {
    matches!(
        ident,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_ir::{FieldId, ProductionId, Rule, Token};

    #[test]
    fn typed_cst_generator_emits_valid_arithmetic_wrapper_module() {
        let grammar = arithmetic_grammar();
        let generated = TypedCstGenerator::new(&grammar).generate();

        syn::parse2::<syn::File>(generated.clone())
            .expect("typed CST generator should emit valid Rust syntax");

        let code = generated.to_string();
        assert!(code.contains("pub mod syntax"));
        assert!(code.contains("pub struct SourceFile"));
        assert!(code.contains("pub struct Expression"));
        assert!(code.contains("pub struct MinusToken"));
        assert!(code.contains("pub struct NumberToken"));
        assert!(code.contains("pub fn source_file"));
    }

    #[test]
    fn typed_cst_generator_uniquifies_colliding_wrapper_names() {
        let grammar = duplicate_punctuation_token_grammar();
        let generated = TypedCstGenerator::new(&grammar).generate();

        syn::parse2::<syn::File>(generated.clone())
            .expect("duplicate token wrapper names should still emit valid Rust syntax");

        let code = generated.to_string();
        assert_eq!(code.matches("pub struct Token <").count(), 1);
        assert!(code.contains("pub struct TokenSymbol1"));
        assert!(code.contains("node_has_kind (document , id , \"+\")"));
        assert!(code.contains("node_has_kind (document , id , \"-\")"));
    }

    #[test]
    fn typed_cst_generator_projects_fields_through_native_edges() {
        let grammar = arithmetic_grammar();
        let generated = TypedCstGenerator::new(&grammar).generate().to_string();

        assert!(generated.contains("pub fn left"));
        assert!(generated.contains("pub fn operator"));
        assert!(generated.contains("pub fn right"));
        assert!(generated.contains("edge_by_field_name (\"left\")"));
        assert!(generated.contains("edge_by_field_name (\"operator\")"));
        assert!(generated.contains("edge_by_field_name (\"right\")"));
        assert!(generated.contains("Expression :: cast"));
        assert!(generated.contains("MinusToken :: cast"));
    }

    #[test]
    fn typed_cst_generator_keeps_hidden_tokens_when_they_are_field_targets() {
        let grammar = hidden_field_token_grammar();
        let generated = TypedCstGenerator::new(&grammar).generate().to_string();

        assert!(generated.contains("pub struct NumberToken"));
        assert!(generated.contains("pub fn left"));
        assert!(generated.contains("edge_by_field_name (\"left\")"));
        assert!(generated.contains("NumberToken :: cast"));
    }

    #[test]
    fn typed_cst_generator_casts_quoted_inline_string_field_targets() {
        let grammar = quoted_inline_string_field_grammar();
        let generated = TypedCstGenerator::new(&grammar).generate().to_string();

        assert!(generated.contains("pub fn operator"));
        assert!(generated.contains("edge_by_field_name (\"operator\")"));
        assert!(generated.contains("Token :: cast"));
        assert!(generated.contains("node_has_kind (document , id , \"\\\"+\\\"\")"));
    }

    fn arithmetic_grammar() -> Grammar {
        let mut grammar = Grammar::new("arithmetic".to_string());

        let number = SymbolId(0);
        let minus = SymbolId(1);
        let source_file = SymbolId(2);
        let expression = SymbolId(3);

        grammar.tokens.insert(
            number,
            Token {
                name: "number".to_string(),
                pattern: TokenPattern::Regex(r"\d+".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            minus,
            Token {
                name: "minus".to_string(),
                pattern: TokenPattern::String("-".to_string()),
                fragile: false,
            },
        );

        grammar
            .rule_names
            .insert(source_file, "source_file".to_string());
        grammar
            .rule_names
            .insert(expression, "expression".to_string());

        grammar.fields.insert(FieldId(0), "left".to_string());
        grammar.fields.insert(FieldId(1), "operator".to_string());
        grammar.fields.insert(FieldId(2), "right".to_string());

        grammar.add_rule(Rule {
            lhs: source_file,
            rhs: vec![Symbol::NonTerminal(expression)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.add_rule(Rule {
            lhs: expression,
            rhs: vec![
                Symbol::NonTerminal(expression),
                Symbol::Terminal(minus),
                Symbol::NonTerminal(expression),
            ],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)],
            production_id: ProductionId(1),
        });
        grammar.add_rule(Rule {
            lhs: expression,
            rhs: vec![Symbol::Terminal(number)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        });

        grammar
    }

    fn duplicate_punctuation_token_grammar() -> Grammar {
        let mut grammar = Grammar::new("duplicate_punctuation_tokens".to_string());

        let plus = SymbolId(0);
        let minus = SymbolId(1);
        let source_file = SymbolId(2);

        grammar.tokens.insert(
            plus,
            Token {
                name: "+".to_string(),
                pattern: TokenPattern::String("+".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            minus,
            Token {
                name: "-".to_string(),
                pattern: TokenPattern::String("-".to_string()),
                fragile: false,
            },
        );
        grammar
            .rule_names
            .insert(source_file, "source_file".to_string());

        grammar.add_rule(Rule {
            lhs: source_file,
            rhs: vec![Symbol::Terminal(plus)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

        grammar
    }

    fn hidden_field_token_grammar() -> Grammar {
        let mut grammar = Grammar::new("hidden_field_token".to_string());

        let number = SymbolId(0);
        let source_file = SymbolId(1);
        let pair = SymbolId(2);

        grammar.tokens.insert(
            number,
            Token {
                name: "_number".to_string(),
                pattern: TokenPattern::Regex(r"\d+".to_string()),
                fragile: false,
            },
        );

        grammar
            .rule_names
            .insert(source_file, "source_file".to_string());
        grammar.rule_names.insert(pair, "pair".to_string());
        grammar.fields.insert(FieldId(0), "left".to_string());

        grammar.add_rule(Rule {
            lhs: source_file,
            rhs: vec![Symbol::NonTerminal(pair)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.add_rule(Rule {
            lhs: pair,
            rhs: vec![Symbol::Terminal(number)],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(0), 0)],
            production_id: ProductionId(1),
        });

        grammar
    }

    fn quoted_inline_string_field_grammar() -> Grammar {
        let mut grammar = Grammar::new("quoted_inline_string_field".to_string());

        let plus = SymbolId(0);
        let source_file = SymbolId(1);
        let expression = SymbolId(2);

        grammar.tokens.insert(
            plus,
            Token {
                name: "\"+\"".to_string(),
                pattern: TokenPattern::String("+".to_string()),
                fragile: false,
            },
        );

        grammar
            .rule_names
            .insert(source_file, "source_file".to_string());
        grammar
            .rule_names
            .insert(expression, "expression".to_string());
        grammar.fields.insert(FieldId(0), "operator".to_string());

        grammar.add_rule(Rule {
            lhs: source_file,
            rhs: vec![Symbol::NonTerminal(expression)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.add_rule(Rule {
            lhs: expression,
            rhs: vec![Symbol::Terminal(plus)],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(0), 0)],
            production_id: ProductionId(1),
        });

        grammar
    }
}
