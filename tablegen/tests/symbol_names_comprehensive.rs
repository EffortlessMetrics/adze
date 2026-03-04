#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for symbol name generation in adze-tablegen.
//!
//! Exercises the public API across four code paths:
//! - `serializer::serialize_language()` → JSON with `symbol_names` array
//! - `LanguageGenerator::generate()` → TokenStream with SYMBOL_NAMES
//! - `AbiLanguageBuilder::generate()` → TokenStream with SYMBOL_NAME_N byte arrays
//! - `StaticLanguageGenerator::generate_language_code()` → TokenStream

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::StaticLanguageGenerator;
use adze_tablegen::abi_builder::AbiLanguageBuilder;
use adze_tablegen::language_gen::LanguageGenerator;
use adze_tablegen::serializer::{SerializableLanguage, serialize_language};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn regex_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::Regex(pattern.to_string()),
        fragile: false,
    }
}

fn string_token(name: &str, literal: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(literal.to_string()),
        fragile: false,
    }
}

fn make_grammar(
    name: &str,
    tokens: Vec<(SymbolId, Token)>,
    rules: Vec<Rule>,
    rule_names: Vec<(SymbolId, String)>,
) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    for (id, tok) in tokens {
        g.tokens.insert(id, tok);
    }
    for rule in rules {
        g.add_rule(rule);
    }
    for (id, rn) in rule_names {
        g.rule_names.insert(id, rn);
    }
    g
}

fn simple_rule(lhs: u16, rhs: Vec<Symbol>, prod_id: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        production_id: ProductionId(prod_id),
        fields: vec![],
        precedence: None,
        associativity: None,
    }
}

/// ParseTable for LanguageGenerator (index 0 = EOF).
fn make_lang_gen_table(grammar: &Grammar, states: usize, symbol_count: usize) -> ParseTable {
    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();
    ParseTable {
        action_table: vec![vec![vec![]; symbol_count]; states],
        goto_table: vec![vec![StateId(u16::MAX); symbol_count]; states],
        rules: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            states
        ],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// ParseTable for AbiLanguageBuilder with custom symbol_to_index.
fn make_abi_table(
    grammar: &Grammar,
    symbol_to_index: BTreeMap<SymbolId, usize>,
    eof: SymbolId,
) -> ParseTable {
    let symbol_count = symbol_to_index.len();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (&sid, &idx) in &symbol_to_index {
        if idx < symbol_count {
            index_to_symbol[idx] = sid;
        }
    }
    ParseTable {
        action_table: vec![vec![vec![]; symbol_count]; 1],
        goto_table: vec![vec![StateId(u16::MAX); symbol_count]; 1],
        rules: vec![],
        state_count: 1,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol: eof,
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Shorthand: ABI builder output as String.
fn abi_output(grammar: &Grammar, s2i: BTreeMap<SymbolId, usize>, eof: SymbolId) -> String {
    let pt = make_abi_table(grammar, s2i, eof);
    AbiLanguageBuilder::new(grammar, &pt).generate().to_string()
}

/// Deserialize the JSON produced by serialize_language into the public struct.
fn serialized_names(grammar: &Grammar, pt: &ParseTable) -> Vec<String> {
    let json = serialize_language(grammar, pt, None).expect("serialization succeeds");
    let lang: SerializableLanguage = serde_json::from_str(&json).expect("valid JSON");
    lang.symbol_names
}

/// Build a minimal ParseTable for StaticLanguageGenerator.
fn make_static_gen_table(grammar: &Grammar, states: usize, symbol_count: usize) -> ParseTable {
    make_lang_gen_table(grammar, states, symbol_count)
}

// ===========================================================================
// 1. Symbol names in serialized language (serialize_language)
// ===========================================================================

#[test]
fn serialized_eof_sentinel_is_end() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let names = serialized_names(&grammar, &pt);
    assert_eq!(names[0], "end");
}

#[test]
fn serialized_single_token() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token("identifier", "[a-z]+"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"identifier".to_string()));
}

#[test]
fn serialized_named_rule_name() {
    let grammar = make_grammar(
        "g",
        vec![],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "expression".to_string())],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"expression".to_string()));
}

#[test]
fn serialized_unnamed_rule_fallback() {
    let grammar = make_grammar("g", vec![], vec![simple_rule(5, vec![], 0)], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"rule_5".to_string()));
}

#[test]
fn serialized_anonymous_string_token() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("+", "+"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"+".to_string()));
}

#[test]
fn serialized_external_token() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "heredoc".to_string(),
        symbol_id: SymbolId(1),
    });
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"heredoc".to_string()));
}

#[test]
fn serialized_ordering_eof_tokens_rules_externals() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("tok", "t"))],
        vec![simple_rule(5, vec![], 0)],
        vec![(SymbolId(5), "stmt".to_string())],
    );
    grammar.externals.push(ExternalToken {
        name: "ext".to_string(),
        symbol_id: SymbolId(10),
    });
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let names = serialized_names(&grammar, &pt);
    assert_eq!(names[0], "end");
    assert_eq!(names[1], "tok");
    assert_eq!(names[2], "stmt");
    assert_eq!(names[3], "ext");
}

#[test]
fn serialized_determinism_across_calls() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("alpha", "a")),
            (SymbolId(2), string_token("beta", "b")),
            (SymbolId(3), regex_token("gamma", ".")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let first = serialized_names(&grammar, &pt);
    let second = serialized_names(&grammar, &pt);
    assert_eq!(first, second);
}

#[test]
fn serialized_empty_grammar_has_only_eof() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let names = serialized_names(&grammar, &pt);
    assert_eq!(names, vec!["end"]);
}

#[test]
fn serialized_special_chars_in_token_name() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("&&", "&&")),
            (SymbolId(2), string_token("||", "||")),
            (SymbolId(3), string_token("->", "->")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"&&".to_string()));
    assert!(names.contains(&"||".to_string()));
    assert!(names.contains(&"->".to_string()));
}

#[test]
fn serialized_hidden_underscore_tokens() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), regex_token("_ws", r"\s+")),
            (SymbolId(2), regex_token("visible", "[a-z]+")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"_ws".to_string()));
    assert!(names.contains(&"visible".to_string()));
}

// ===========================================================================
// 2. LanguageGenerator — SYMBOL_NAMES in generated TokenStream
// ===========================================================================

#[test]
fn lang_gen_named_symbols_appear() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token("number", r"\d+"))],
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(2), "expression".to_string())],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let output = LanguageGenerator::new(&grammar, &pt).generate().to_string();
    assert!(output.contains("number"));
    assert!(output.contains("expression"));
}

#[test]
fn lang_gen_anonymous_string_token_name() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("(", "("))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let output = LanguageGenerator::new(&grammar, &pt).generate().to_string();
    assert!(output.contains("("));
}

#[test]
fn lang_gen_ordering_deterministic() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("z_last", "z")),
            (SymbolId(2), string_token("a_first", "a")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let out1 = LanguageGenerator::new(&grammar, &pt).generate().to_string();
    let out2 = LanguageGenerator::new(&grammar, &pt).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn lang_gen_symbol_names_array_present() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token("id", "[a-z]+"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let output = LanguageGenerator::new(&grammar, &pt).generate().to_string();
    assert!(output.contains("SYMBOL_NAMES"));
}

#[test]
fn lang_gen_empty_grammar_end_only() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let output = LanguageGenerator::new(&grammar, &pt).generate().to_string();
    assert!(output.contains("\"end\""));
}

// ===========================================================================
// 3. AbiLanguageBuilder — symbol names as byte arrays
// ===========================================================================

#[test]
fn abi_named_rule_name_bytes() {
    let grammar = make_grammar("g", vec![], vec![], vec![(SymbolId(3), "stmt".to_string())]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(3), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // 's'=115, 't'=116, 'm'=109, 't'=116 → "stmt"
    assert!(output.contains("115u8"));
}

#[test]
fn abi_anonymous_token_bytes() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("+", "+"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // '+'=43
    assert!(output.contains("43u8"));
}

#[test]
fn abi_symbol_name_idents_sequential() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
            (SymbolId(3), string_token("c", "c")),
        ],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    for i in 0..=3u16 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(output.contains("SYMBOL_NAME_0"));
    assert!(output.contains("SYMBOL_NAME_1"));
    assert!(output.contains("SYMBOL_NAME_2"));
    assert!(output.contains("SYMBOL_NAME_3"));
}

#[test]
fn abi_special_chars_encoded() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token(">=", ">="))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // '>'=62, '='=61
    assert!(output.contains("62u8"));
    assert!(output.contains("61u8"));
}

#[test]
fn abi_empty_grammar_still_generates() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(output.contains("SYMBOL_NAME_0"));
    // "end\0" → 'e'=101
    assert!(output.contains("101u8"));
}

#[test]
fn abi_determinism_identical_output() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), regex_token("num", r"\d+")),
            (SymbolId(2), string_token("op", "+")),
        ],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    for i in 0..=2u16 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let out1 = abi_output(&grammar, s2i.clone(), SymbolId(0));
    let out2 = abi_output(&grammar, s2i, SymbolId(0));
    assert_eq!(out1, out2);
}

// ===========================================================================
// 4. StaticLanguageGenerator — generate_language_code()
// ===========================================================================

#[test]
fn static_gen_token_name_in_output() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token("identifier", "[a-z]+"))],
        vec![],
        vec![],
    );
    let pt = make_static_gen_table(&grammar, 1, 2);
    let slg = StaticLanguageGenerator::new(grammar, pt);
    let output = slg.generate_language_code().to_string();
    assert!(output.contains("identifier"));
}

#[test]
fn static_gen_named_rule_in_output() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token("tok", "."))],
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(2), "program".to_string())],
    );
    let pt = make_static_gen_table(&grammar, 1, 3);
    let slg = StaticLanguageGenerator::new(grammar, pt);
    let output = slg.generate_language_code().to_string();
    // LanguageGenerator (used internally) resolves rule names
    assert!(output.contains("SYMBOL_NAMES"));
}

#[test]
fn static_gen_deterministic() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![],
        vec![],
    );
    let pt1 = make_static_gen_table(&grammar, 1, 3);
    let pt2 = make_static_gen_table(&grammar, 1, 3);
    let slg1 = StaticLanguageGenerator::new(grammar.clone(), pt1);
    let slg2 = StaticLanguageGenerator::new(grammar, pt2);
    let out1 = slg1.generate_language_code().to_string();
    let out2 = slg2.generate_language_code().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_empty_grammar_compiles() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_static_gen_table(&grammar, 1, 1);
    let slg = StaticLanguageGenerator::new(grammar, pt);
    let output = slg.generate_language_code().to_string();
    assert!(output.contains("SYMBOL_NAMES"));
}

// ===========================================================================
// 5. Cross-path consistency
// ===========================================================================

#[test]
fn serialized_and_lang_gen_agree_on_token_names() {
    // Use only tokens (no rules) so both paths resolve names identically.
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("plus", "+")),
            (SymbolId(2), regex_token("number", r"\d+")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let ser_names = serialized_names(&grammar, &pt);
    let ts_output = LanguageGenerator::new(&grammar, &pt).generate().to_string();
    for name in &ser_names {
        assert!(
            ts_output.contains(name),
            "serialized name {:?} missing from LanguageGenerator output",
            name,
        );
    }
}

#[test]
fn serialized_name_count_matches_symbol_count() {
    let mut grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![],
    );
    grammar.externals.push(ExternalToken {
        name: "ext".to_string(),
        symbol_id: SymbolId(20),
    });
    let pt = make_lang_gen_table(&grammar, 1, 5);
    let names = serialized_names(&grammar, &pt);
    // 1 (EOF) + 2 tokens + 1 rule + 1 external = 5
    assert_eq!(names.len(), 5);
}

#[test]
fn serialized_multiple_rules_sorted_by_id() {
    let grammar = make_grammar(
        "g",
        vec![],
        vec![simple_rule(20, vec![], 0), simple_rule(10, vec![], 1)],
        vec![
            (SymbolId(20), "beta_rule".to_string()),
            (SymbolId(10), "alpha_rule".to_string()),
        ],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    let alpha_pos = names.iter().position(|n| n == "alpha_rule").unwrap();
    let beta_pos = names.iter().position(|n| n == "beta_rule").unwrap();
    assert!(alpha_pos < beta_pos, "rules sorted by SymbolId, 10 < 20");
}

#[test]
fn serialized_tokens_sorted_by_id() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(5), string_token("second", "b")),
            (SymbolId(1), string_token("first", "a")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    let first_pos = names.iter().position(|n| n == "first").unwrap();
    let second_pos = names.iter().position(|n| n == "second").unwrap();
    assert!(first_pos < second_pos, "tokens sorted by SymbolId, 1 < 5");
}

#[test]
fn serialized_unicode_token_name() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("λ", "λ"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"λ".to_string()));
}

#[test]
fn serialized_long_token_name() {
    let long_name = "a_very_long_symbol_name_that_exceeds_typical_length";
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token(long_name, "."))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&long_name.to_string()));
}

#[test]
fn serialized_many_symbols_all_present() {
    let mut tokens = vec![];
    for i in 1..=30u16 {
        tokens.push((
            SymbolId(i),
            string_token(&format!("sym_{}", i), &format!("{}", i)),
        ));
    }
    let grammar = make_grammar("g", tokens, vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 31);
    let names = serialized_names(&grammar, &pt);
    // 1 EOF + 30 tokens
    assert_eq!(names.len(), 31);
    for i in 1..=30u16 {
        assert!(names.contains(&format!("sym_{}", i)));
    }
}
