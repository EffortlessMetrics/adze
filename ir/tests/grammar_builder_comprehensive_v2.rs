//! Comprehensive tests for GrammarBuilder API — 60+ tests covering
//! construction, tokens, rules, start symbols, precedence, extras,
//! externals, presets, edge cases, and property-based tests.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, ProductionId, Symbol, SymbolId, TokenPattern};

// ============================================================================
// Helper: look up a SymbolId by rule name
// ============================================================================
fn sym(g: &adze_ir::Grammar, name: &str) -> SymbolId {
    g.find_symbol_by_name(name)
        .unwrap_or_else(|| panic!("symbol '{name}' not found in rule_names"))
}

fn _tok_id(g: &adze_ir::Grammar, name: &str) -> SymbolId {
    g.tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

// ============================================================================
// 1. Basic construction
// ============================================================================

#[test]
fn empty_grammar_has_correct_name() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.name, "empty");
}

#[test]
fn empty_grammar_has_no_rules() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.rules.is_empty());
}

#[test]
fn empty_grammar_has_no_tokens() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.tokens.is_empty());
}

#[test]
fn empty_grammar_start_symbol_is_none() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.start_symbol().is_none());
}

#[test]
fn grammar_name_preserved() {
    let g = GrammarBuilder::new("my_lang").build();
    assert_eq!(g.name, "my_lang");
}

// ============================================================================
// 2. Token registration
// ============================================================================

#[test]
fn single_token_registered() {
    let g = GrammarBuilder::new("t").token("NUM", r"\d+").build();
    assert_eq!(g.tokens.len(), 1);
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.name, "NUM");
}

#[test]
fn multiple_tokens_registered() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn token_ids_start_at_one() {
    let g = GrammarBuilder::new("t").token("X", "x").build();
    let (id, _) = g.tokens.iter().next().unwrap();
    assert!(id.0 >= 1, "SymbolId(0) is reserved for EOF");
}

#[test]
fn token_ids_are_unique() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .build();
    let ids: Vec<u16> = g.tokens.keys().map(|id| id.0).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids.len(), sorted.len());
}

#[test]
fn duplicate_token_name_overwrites() {
    let g = GrammarBuilder::new("t")
        .token("A", "first")
        .token("A", "second")
        .build();
    assert_eq!(g.tokens.len(), 1);
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("second".into()));
}

#[test]
fn string_literal_token_pattern() {
    let g = GrammarBuilder::new("t").token("kw", "return").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("return".into()));
}

#[test]
fn regex_token_pattern_detected() {
    let g = GrammarBuilder::new("t").token("NUM", r"\d+").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_not_fragile_by_default() {
    let g = GrammarBuilder::new("t").token("A", "a").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert!(!tok.fragile);
}

#[test]
fn fragile_token_flag_set() {
    let g = GrammarBuilder::new("t").fragile_token("ERR", "err").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert!(tok.fragile);
}

// ============================================================================
// 3. Rule registration
// ============================================================================

#[test]
fn single_rule_registered() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn alternative_rules_grouped_under_same_lhs() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn empty_rhs_becomes_epsilon() {
    let g = GrammarBuilder::new("t")
        .rule("empty", vec![])
        .start("empty")
        .build();
    let id = sym(&g, "empty");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

#[test]
fn rule_lhs_matches_symbol_id() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].lhs, id);
}

#[test]
fn token_in_rhs_is_terminal() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert!(matches!(rules[0].rhs[0], Symbol::Terminal(_)));
}

#[test]
fn nonterminal_in_rhs_is_nonterminal() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("inner", vec!["A"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();
    let id = sym(&g, "outer");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn production_ids_are_sequential() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let ids: Vec<ProductionId> = g.all_rules().map(|r| r.production_id).collect();
    assert_eq!(ids[0], ProductionId(0));
    assert_eq!(ids[1], ProductionId(1));
}

#[test]
fn all_rules_iterator_counts_all() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("x", vec!["B"])
        .rule("y", vec!["A", "B"])
        .start("x")
        .build();
    assert_eq!(g.all_rules().count(), 3);
}

// ============================================================================
// 4. Start symbol
// ============================================================================

#[test]
fn start_symbol_returned_from_grammar() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    // start_symbol() has heuristics; verify the rules map has root first
    let first_key = *g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(&first_key).unwrap();
    assert_eq!(first_name, "root");
}

#[test]
fn start_symbol_rules_come_first_in_map() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("other", vec!["A"])
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let first_key = *g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(&first_key).unwrap();
    assert_eq!(first_name, "root");
}

#[test]
fn start_before_rule_still_works() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .start("s")
        .rule("s", vec!["A"])
        .build();
    let first_key = *g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(&first_key).unwrap();
    assert_eq!(first_name, "s");
}

// ============================================================================
// 5. Precedence & associativity
// ============================================================================

#[test]
fn rule_with_precedence_stores_static_prec() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 5, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let id = sym(&g, "e");
    let rules = g.get_rules_for_symbol(id).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(5)));
}

#[test]
fn rule_with_precedence_stores_assoc() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Right)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let id = sym(&g, "e");
    let rules = g.get_rules_for_symbol(id).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn plain_rule_has_no_precedence() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert!(rules[0].precedence.is_none());
    assert!(rules[0].associativity.is_none());
}

#[test]
fn precedence_declaration_added() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .build();
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[1].level, 2);
}

#[test]
fn multiple_prec_levels_in_same_grammar() {
    let g = GrammarBuilder::new("calc")
        .token("N", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "-", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "/", "e"], 2, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let id = sym(&g, "e");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 5);
    let prec_vals: Vec<i16> = rules
        .iter()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(prec_vals, vec![1, 1, 2, 2]);
}

#[test]
fn non_associative_rule() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("==", "==")
        .rule_with_precedence("e", vec!["e", "==", "e"], 0, Associativity::None)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let id = sym(&g, "e");
    let rules = g.get_rules_for_symbol(id).unwrap();
    let cmp = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(cmp.associativity, Some(Associativity::None));
}

// ============================================================================
// 6. Extras
// ============================================================================

#[test]
fn extra_token_registered() {
    let g = GrammarBuilder::new("t")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn multiple_extras() {
    let g = GrammarBuilder::new("t")
        .token("WS", r"[ \t]+")
        .token("COMMENT", r"//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.extras.len(), 2);
}

// ============================================================================
// 7. External tokens
// ============================================================================

#[test]
fn external_token_registered() {
    let g = GrammarBuilder::new("t")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn multiple_externals() {
    let g = GrammarBuilder::new("t")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .build();
    assert_eq!(g.externals.len(), 2);
}

// ============================================================================
// 8. rule_names population
// ============================================================================

#[test]
fn nonterminal_appears_in_rule_names() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    assert!(g.rule_names.values().any(|n| n == "expr"));
}

#[test]
fn operator_tokens_not_in_rule_names() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("-", "-")
        .build();
    // "+" and "-" are operators and should NOT be added to rule_names
    assert!(!g.rule_names.values().any(|n| n == "+"));
    assert!(!g.rule_names.values().any(|n| n == "-"));
}

#[test]
fn uppercase_token_not_in_rule_names() {
    let g = GrammarBuilder::new("t").token("NUMBER", r"\d+").build();
    // All-uppercase names are treated as tokens by the heuristic
    assert!(!g.rule_names.values().any(|n| n == "NUMBER"));
}

// ============================================================================
// 9. find_symbol_by_name
// ============================================================================

#[test]
fn find_existing_symbol() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    assert!(g.find_symbol_by_name("expr").is_some());
}

#[test]
fn find_missing_symbol_returns_none() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.find_symbol_by_name("nonexistent").is_none());
}

// ============================================================================
// 10. get_rules_for_symbol
// ============================================================================

#[test]
fn get_rules_for_known_symbol() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    assert!(g.get_rules_for_symbol(id).is_some());
}

#[test]
fn get_rules_for_unknown_symbol_is_none() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.get_rules_for_symbol(SymbolId(999)).is_none());
}

// ============================================================================
// 11. Fluent chaining order independence
// ============================================================================

#[test]
fn token_then_rule_then_start() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    assert_eq!(g.rules.len(), 1);
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn start_then_token_then_rule() {
    let g = GrammarBuilder::new("t")
        .start("s")
        .token("A", "a")
        .rule("s", vec!["A"])
        .build();
    let first_key = *g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(&first_key).unwrap();
    assert_eq!(first_name, "s");
}

#[test]
fn rule_then_token_rhs_still_nonterminal() {
    // If we add a rule referencing "X" before registering "X" as token,
    // "X" is non-terminal at rule-add time because no token exists yet.
    let g = GrammarBuilder::new("t")
        .rule("s", vec!["X"])
        .token("X", "x")
        .start("s")
        .build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    // At the time the rule was added, "X" was not yet a token → NonTerminal
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
}

// ============================================================================
// 12. Complex grammars
// ============================================================================

#[test]
fn arithmetic_grammar() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();
    assert_eq!(g.all_rules().count(), 6);
    assert_eq!(g.tokens.len(), 5);
}

#[test]
fn recursive_rule() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("list", vec!["A"])
        .rule("list", vec!["list", "A"])
        .start("list")
        .build();
    let id = sym(&g, "list");
    let rules = g.get_rules_for_symbol(id).unwrap();
    // The second rule has "list" as NonTerminal in rhs
    let recursive = &rules[1];
    assert!(matches!(recursive.rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn multiple_nonterminals_in_grammar() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .rule("z", vec!["x", "y"])
        .start("z")
        .build();
    assert_eq!(g.rules.len(), 3);
}

// ============================================================================
// 13. Preset grammars
// ============================================================================

#[test]
fn python_like_has_externals() {
    let g = GrammarBuilder::python_like();
    assert!(g.externals.len() >= 2);
}

#[test]
fn python_like_has_extras() {
    let g = GrammarBuilder::python_like();
    assert!(!g.extras.is_empty());
}

#[test]
fn python_like_has_nullable_start() {
    let g = GrammarBuilder::python_like();
    let module_id = g.find_symbol_by_name("module").unwrap();
    let module_rules = &g.rules[&module_id];
    assert!(
        module_rules
            .iter()
            .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon))
    );
}

#[test]
fn javascript_like_has_precedence() {
    let g = GrammarBuilder::javascript_like();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    let rules = &g.rules[&expr_id];
    assert!(rules.iter().any(|r| r.precedence.is_some()));
}

#[test]
fn javascript_like_non_nullable_start() {
    let g = GrammarBuilder::javascript_like();
    let prog_id = g.find_symbol_by_name("program").unwrap();
    let rules = &g.rules[&prog_id];
    assert!(
        !rules
            .iter()
            .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon))
    );
}

// ============================================================================
// 14. Edge cases
// ============================================================================

#[test]
fn grammar_with_only_tokens_no_rules() {
    let g = GrammarBuilder::new("tokens_only")
        .token("A", "a")
        .token("B", "b")
        .build();
    assert!(g.rules.is_empty());
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn grammar_with_only_rules_no_tokens() {
    let g = GrammarBuilder::new("rules_only")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.rules.len(), 1);
    assert!(g.tokens.is_empty());
}

#[test]
fn single_epsilon_rule() {
    let g = GrammarBuilder::new("t")
        .rule("s", vec![])
        .start("s")
        .build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

#[test]
fn long_rhs_rule() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("s", vec!["A", "B", "C", "D", "E"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].rhs.len(), 5);
}

#[test]
fn many_alternatives() {
    let mut builder = GrammarBuilder::new("t").token("A", "a");
    for _ in 0..20 {
        builder = builder.rule("s", vec!["A"]);
    }
    let g = builder.start("s").build();
    let id = sym(&g, "s");
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 20);
}

#[test]
fn fields_initially_empty() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.fields.is_empty());
}

#[test]
fn alias_sequences_initially_empty() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.alias_sequences.is_empty());
}

#[test]
fn production_ids_map_initially_empty() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.production_ids.is_empty());
}

#[test]
fn max_alias_sequence_length_is_zero() {
    let g = GrammarBuilder::new("t").build();
    assert_eq!(g.max_alias_sequence_length, 0);
}

#[test]
fn symbol_registry_initially_none() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.symbol_registry.is_none());
}

#[test]
fn supertypes_initially_empty() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.supertypes.is_empty());
}

#[test]
fn inline_rules_initially_empty() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.inline_rules.is_empty());
}

// ============================================================================
// 15. Token pattern classification
// ============================================================================

#[test]
fn slash_delimited_regex_pattern() {
    let g = GrammarBuilder::new("t").token("ID", "/[a-z]+/").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::Regex(_)));
}

#[test]
fn alphanumeric_string_pattern() {
    let g = GrammarBuilder::new("t").token("kw", "while").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("while".into()));
}

#[test]
fn operator_same_as_name_is_string() {
    let g = GrammarBuilder::new("t").token("+", "+").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("+".into()));
}

// ============================================================================
// 16. Symbol resolution across rules
// ============================================================================

#[test]
fn same_token_reused_across_rules() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("x", vec!["A"])
        .rule("y", vec!["A"])
        .start("x")
        .build();
    let x_id = sym(&g, "x");
    let y_id = sym(&g, "y");
    let x_rules = g.get_rules_for_symbol(x_id).unwrap();
    let y_rules = g.get_rules_for_symbol(y_id).unwrap();
    // Both rules reference the same token symbol id
    let x_tok = match x_rules[0].rhs[0] {
        Symbol::Terminal(id) => id,
        _ => panic!("expected terminal"),
    };
    let y_tok = match y_rules[0].rhs[0] {
        Symbol::Terminal(id) => id,
        _ => panic!("expected terminal"),
    };
    assert_eq!(x_tok, y_tok);
}

#[test]
fn nonterminal_id_consistent_across_references() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("inner", vec!["A"])
        .rule("outer1", vec!["inner"])
        .rule("outer2", vec!["inner"])
        .start("outer1")
        .build();
    let outer1_rules = g.get_rules_for_symbol(sym(&g, "outer1")).unwrap();
    let outer2_rules = g.get_rules_for_symbol(sym(&g, "outer2")).unwrap();
    let ref1 = match outer1_rules[0].rhs[0] {
        Symbol::NonTerminal(id) => id,
        _ => panic!("expected nonterminal"),
    };
    let ref2 = match outer2_rules[0].rhs[0] {
        Symbol::NonTerminal(id) => id,
        _ => panic!("expected nonterminal"),
    };
    assert_eq!(ref1, ref2);
    assert_eq!(ref1, sym(&g, "inner"));
}

// ============================================================================
// 17. Build registry integration
// ============================================================================

#[test]
fn build_registry_includes_tokens() {
    let mut g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("e", vec!["NUM"])
        .start("e")
        .build();
    let registry = g.get_or_build_registry();
    assert!(registry.get_id("NUM").is_some() || registry.get_id("+").is_some());
}

#[test]
fn build_registry_includes_nonterminals() {
    let mut g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    let registry = g.get_or_build_registry();
    assert!(registry.get_id("expr").is_some());
}

// ============================================================================
// 18. Grammar::Default
// ============================================================================

#[test]
fn default_grammar_is_empty() {
    let g = adze_ir::Grammar::default();
    assert!(g.name.is_empty());
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

// ============================================================================
// 19. Serde roundtrip via builder
// ============================================================================

#[test]
fn serde_roundtrip_preserves_grammar() {
    let g = GrammarBuilder::new("serde_test")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.rules.len(), g2.rules.len());
}

// ============================================================================
// 20. Property-based tests (proptest)
// ============================================================================

mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_ident() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9]{0,7}".prop_map(|s| s)
    }

    proptest! {
        #[test]
        fn grammar_name_roundtrips(name in arb_ident()) {
            let g = GrammarBuilder::new(&name).build();
            prop_assert_eq!(&g.name, &name);
        }

        #[test]
        fn token_count_matches(n in 1usize..=20) {
            let mut builder = GrammarBuilder::new("prop");
            for i in 0..n {
                let name = format!("T{i}");
                builder = builder.token(&name, &name);
            }
            let g = builder.build();
            prop_assert_eq!(g.tokens.len(), n);
        }

        #[test]
        fn rule_count_matches(n in 1usize..=20) {
            let mut builder = GrammarBuilder::new("prop").token("A", "a");
            for _ in 0..n {
                builder = builder.rule("s", vec!["A"]);
            }
            let g = builder.start("s").build();
            let id = sym(&g, "s");
            let rules = g.get_rules_for_symbol(id).unwrap();
            prop_assert_eq!(rules.len(), n);
        }

        #[test]
        fn production_ids_never_duplicate(n in 2usize..=30) {
            let mut builder = GrammarBuilder::new("prop").token("A", "a").token("B", "b");
            for _ in 0..n {
                builder = builder.rule("s", vec!["A"]);
            }
            let g = builder.start("s").build();
            let ids: Vec<u16> = g.all_rules().map(|r| r.production_id.0).collect();
            let mut deduped = ids.clone();
            deduped.sort();
            deduped.dedup();
            prop_assert_eq!(ids.len(), deduped.len());
        }

        #[test]
        fn all_rules_count_equals_sum_of_per_symbol(
            rule_counts in prop::collection::vec(1usize..=5, 1..=5)
        ) {
            let mut builder = GrammarBuilder::new("prop").token("A", "a");
            let mut total = 0usize;
            for (i, &count) in rule_counts.iter().enumerate() {
                let name = format!("r{i}");
                for _ in 0..count {
                    builder = builder.rule(&name, vec!["A"]);
                }
                total += count;
            }
            let g = builder.build();
            prop_assert_eq!(g.all_rules().count(), total);
        }

        #[test]
        fn serde_roundtrip_property(n in 1usize..=10) {
            let mut builder = GrammarBuilder::new("prop");
            for i in 0..n {
                let tname = format!("T{i}");
                builder = builder.token(&tname, &tname);
            }
            builder = builder.rule("s", vec!["T0"]).start("s");
            let g = builder.build();
            let json = serde_json::to_string(&g).unwrap();
            let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&g.name, &g2.name);
            prop_assert_eq!(g.tokens.len(), g2.tokens.len());
            prop_assert_eq!(g.all_rules().count(), g2.all_rules().count());
        }
    }
}
