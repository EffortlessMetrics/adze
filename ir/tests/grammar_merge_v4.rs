//! Grammar merge v4 tests — construction patterns, rule merging, token+rule
//! interaction, extras/externals/conflicts metadata, and grammar operations.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, Grammar, PrecedenceKind, Symbol,
    SymbolId, TokenPattern,
};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Look up a SymbolId by its human-readable name in `rule_names`.
fn sym(g: &Grammar, name: &str) -> SymbolId {
    g.rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("symbol '{name}' not found in rule_names"))
}

/// Collect token names from a grammar.
fn token_names(g: &Grammar) -> HashSet<String> {
    g.tokens.values().map(|t| t.name.clone()).collect()
}

/// Collect rule LHS names.
fn rule_lhs_names(g: &Grammar) -> HashSet<String> {
    g.rules
        .keys()
        .filter_map(|id| g.rule_names.get(id).cloned())
        .collect()
}

/// Total production count across all LHS.
fn total_productions(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

/// Simple arithmetic grammar fixture.
fn arith() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Simple boolean grammar fixture.
fn bool_grammar() -> Grammar {
    GrammarBuilder::new("bool")
        .token("TRUE", "true")
        .token("FALSE", "false")
        .token("AND", "&&")
        .token("OR", "||")
        .rule("bexpr", vec!["bexpr", "AND", "bexpr"])
        .rule("bexpr", vec!["bexpr", "OR", "bexpr"])
        .rule("bexpr", vec!["TRUE"])
        .rule("bexpr", vec!["FALSE"])
        .start("bexpr")
        .build()
}

/// Simple string grammar fixture.
fn string_grammar() -> Grammar {
    GrammarBuilder::new("strings")
        .token("STR", r#""[^"]*""#)
        .token("CONCAT", "++")
        .rule("cat", vec!["cat", "CONCAT", "STR"])
        .rule("cat", vec!["STR"])
        .start("cat")
        .build()
}

// ===========================================================================
// 1. Grammar construction patterns (12 tests)
// ===========================================================================

#[test]
fn construct_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.conflicts.is_empty());
    assert!(g.precedences.is_empty());
    assert!(g.inline_rules.is_empty());
    assert!(g.supertypes.is_empty());
}

#[test]
fn construct_name_preserved() {
    let g = GrammarBuilder::new("my_lang").build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn construct_single_token() {
    let g = GrammarBuilder::new("t").token("X", "x").build();
    assert_eq!(g.tokens.len(), 1);
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.name, "X");
    assert_eq!(tok.pattern, TokenPattern::String("x".to_string()));
}

#[test]
fn construct_regex_token_detected() {
    let g = GrammarBuilder::new("t").token("NUM", r"\d+").build();
    let tok = g.tokens.values().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::Regex(_)));
}

#[test]
fn construct_string_literal_token_detected() {
    let g = GrammarBuilder::new("t").token("IF", "if").build();
    let tok = g.tokens.values().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::String(_)));
}

#[test]
fn construct_fragile_token() {
    let g = GrammarBuilder::new("t").fragile_token("SEMI", ";").build();
    let tok = g.tokens.values().next().unwrap();
    assert!(tok.fragile);
}

#[test]
fn construct_multiple_tokens_unique_ids() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .build();
    assert_eq!(g.tokens.len(), 3);
    let ids: Vec<_> = g.tokens.keys().copied().collect();
    let unique: HashSet<_> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn construct_single_rule_epsilon() {
    let g = GrammarBuilder::new("t")
        .rule("empty", vec![])
        .start("empty")
        .build();
    let id = sym(&g, "empty");
    let rules = &g.rules[&id];
    assert_eq!(rules.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

#[test]
fn construct_single_rule_with_terminal() {
    let g = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    assert_eq!(g.rules[&id].len(), 1);
    assert!(matches!(g.rules[&id][0].rhs[0], Symbol::Terminal(_)));
}

#[test]
fn construct_rule_with_nonterminal() {
    let g = GrammarBuilder::new("t")
        .token("ID", r"[a-z]+")
        .rule("atom", vec!["ID"])
        .rule("expr", vec!["atom"])
        .start("expr")
        .build();
    let expr_id = sym(&g, "expr");
    assert!(matches!(g.rules[&expr_id][0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn construct_start_symbol_comes_first() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("beta", vec!["A"])
        .rule("alpha", vec!["beta"])
        .start("alpha")
        .build();
    let first_key = g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(first_key).unwrap();
    assert_eq!(first_name, "alpha");
}

#[test]
fn construct_chaining_compiles_and_produces_grammar() {
    let g = GrammarBuilder::new("chain")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .extra("WS")
        .token("WS", r"\s+")
        .external("INDENT")
        .inline("s")
        .supertype("s")
        .build();
    assert_eq!(g.name, "chain");
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
}

// ===========================================================================
// 2. Rule merging — multiple alternatives (10 tests)
// ===========================================================================

#[test]
fn merge_same_lhs_accumulates_alternatives() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("item", vec!["A"])
        .rule("item", vec!["B"])
        .rule("item", vec!["C"])
        .start("item")
        .build();
    let id = sym(&g, "item");
    assert_eq!(g.rules[&id].len(), 3);
    assert_eq!(g.rules.len(), 1); // single LHS
}

#[test]
fn merge_preserves_lhs_identity() {
    let g = arith();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    for r in rules {
        assert_eq!(r.lhs, expr_id);
    }
}

#[test]
fn merge_disjoint_grammars_no_lhs_overlap() {
    let a = arith();
    let b = bool_grammar();
    assert!(rule_lhs_names(&a).is_disjoint(&rule_lhs_names(&b)));
}

#[test]
fn merge_production_count_arith() {
    assert_eq!(total_productions(&arith()), 3);
}

#[test]
fn merge_production_count_bool() {
    assert_eq!(total_productions(&bool_grammar()), 4);
}

#[test]
fn merge_combined_production_count() {
    let total = total_productions(&arith()) + total_productions(&bool_grammar());
    assert_eq!(total, 7);
}

#[test]
fn merge_two_lhs_separate() {
    let g = GrammarBuilder::new("t")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("b", vec!["Y"])
        .start("a")
        .build();
    assert_eq!(g.rules.len(), 2);
    assert_eq!(total_productions(&g), 2);
}

#[test]
fn merge_rhs_length_matches_input() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    assert_eq!(g.rules[&id][0].rhs.len(), 3);
}

#[test]
fn merge_identical_grammars_are_equal() {
    let g1 = arith();
    let g2 = arith();
    assert_eq!(g1, g2);
}

#[test]
fn merge_different_names_not_equal() {
    let g1 = GrammarBuilder::new("a")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("b")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    assert_ne!(g1, g2);
}

// ===========================================================================
// 3. Token + rule interaction (10 tests)
// ===========================================================================

#[test]
fn token_used_in_rule_resolves_terminal() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("N", r"\d+")
        .rule("e", vec!["N", "+", "N"])
        .start("e")
        .build();
    let id = sym(&g, "e");
    for s in &g.rules[&id][0].rhs {
        assert!(matches!(s, Symbol::Terminal(_)));
    }
}

#[test]
fn token_not_referenced_in_rules_still_exists() {
    let g = GrammarBuilder::new("t")
        .token("UNUSED", "unused")
        .token("USED", "used")
        .rule("s", vec!["USED"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn unknown_symbol_in_rule_becomes_nonterminal() {
    let g = GrammarBuilder::new("t")
        .rule("s", vec!["unknown"])
        .start("s")
        .build();
    let id = sym(&g, "s");
    assert!(matches!(g.rules[&id][0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn mixed_terminal_nonterminal_in_rule() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .rule("factor", vec!["NUM"])
        .rule("expr", vec!["factor", "NUM"])
        .start("expr")
        .build();
    let expr_id = sym(&g, "expr");
    let rhs = &g.rules[&expr_id][0].rhs;
    assert!(matches!(rhs[0], Symbol::NonTerminal(_)));
    assert!(matches!(rhs[1], Symbol::Terminal(_)));
}

#[test]
fn token_overlap_between_grammars() {
    let a = arith();
    let s = string_grammar();
    let a_names = token_names(&a);
    let s_names = token_names(&s);
    // arith has "+", string has "CONCAT" — no overlap
    assert!(a_names.is_disjoint(&s_names));
}

#[test]
fn disjoint_token_sets_union_count() {
    let a = arith();
    let b = bool_grammar();
    let combined: HashSet<_> = token_names(&a).union(&token_names(&b)).cloned().collect();
    assert_eq!(combined.len(), a.tokens.len() + b.tokens.len());
}

#[test]
fn empty_grammar_union_with_nonempty() {
    let empty = GrammarBuilder::new("empty").build();
    let a = arith();
    let combined: HashSet<_> = token_names(&empty)
        .union(&token_names(&a))
        .cloned()
        .collect();
    assert_eq!(combined.len(), a.tokens.len());
}

#[test]
fn token_names_no_duplicates_in_single_grammar() {
    let g = arith();
    let names: Vec<_> = g.tokens.values().map(|t| &t.name).collect();
    let unique: HashSet<_> = names.iter().collect();
    assert_eq!(names.len(), unique.len());
}

#[test]
fn token_pattern_preserved_after_build() {
    let g = GrammarBuilder::new("t")
        .token("RE", r"[a-z]+")
        .token("LIT", "hello")
        .build();
    for tok in g.tokens.values() {
        match tok.name.as_str() {
            "RE" => assert_eq!(tok.pattern, TokenPattern::Regex(r"[a-z]+".into())),
            "LIT" => assert_eq!(tok.pattern, TokenPattern::String("hello".into())),
            _ => panic!("unexpected token"),
        }
    }
}

#[test]
fn all_rules_iterator_yields_correct_count() {
    let g = arith();
    let count = g.all_rules().count();
    assert_eq!(count, 3);
}

// ===========================================================================
// 4. Extras / externals / conflicts metadata (12 tests)
// ===========================================================================

#[test]
fn extras_registered_via_builder() {
    let g = GrammarBuilder::new("t")
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn multiple_extras() {
    let g = GrammarBuilder::new("t")
        .token("WS", r"\s+")
        .token("COMMENT", r"//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn externals_registered_via_builder() {
    let g = GrammarBuilder::new("t")
        .external("INDENT")
        .external("DEDENT")
        .build();
    assert_eq!(g.externals.len(), 2);
    let names: Vec<_> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"INDENT"));
    assert!(names.contains(&"DEDENT"));
}

#[test]
fn external_has_symbol_id() {
    let g = GrammarBuilder::new("t").external("NEWLINE").build();
    let ext = &g.externals[0];
    assert_eq!(ext.name, "NEWLINE");
    // symbol_id should be a valid u16
    let _ = ext.symbol_id.0;
}

#[test]
fn conflict_set_directly_on_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let sid = sym(&g, "s");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![sid],
        resolution: ConflictResolution::GLR,
    });
    assert_eq!(g.conflicts.len(), 1);
    assert!(matches!(g.conflicts[0].resolution, ConflictResolution::GLR));
}

#[test]
fn conflict_resolution_precedence() {
    let mut g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let sid = sym(&g, "s");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![sid],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(5)),
    });
    assert!(matches!(
        g.conflicts[0].resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    ));
}

#[test]
fn conflict_resolution_associativity() {
    let mut g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let sid = sym(&g, "s");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![sid],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    assert!(matches!(
        g.conflicts[0].resolution,
        ConflictResolution::Associativity(Associativity::Left)
    ));
}

#[test]
fn precedence_via_builder() {
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
fn precedence_associativity_right() {
    let g = GrammarBuilder::new("t")
        .token("^", "^")
        .precedence(3, Associativity::Right, vec!["^"])
        .build();
    assert_eq!(g.precedences[0].associativity, Associativity::Right);
}

#[test]
fn inline_rules_via_builder() {
    let g = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("helper", vec!["X"])
        .rule("main", vec!["helper"])
        .inline("helper")
        .start("main")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn supertype_via_builder() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .supertype("expr")
        .start("expr")
        .build();
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn empty_grammar_all_metadata_empty() {
    let g = GrammarBuilder::new("bare").build();
    assert!(g.extras.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.conflicts.is_empty());
    assert!(g.precedences.is_empty());
    assert!(g.inline_rules.is_empty());
    assert!(g.supertypes.is_empty());
    assert!(g.fields.is_empty());
}

// ===========================================================================
// 5. Grammar operations — normalize, validate, optimize (11 tests)
// ===========================================================================

#[test]
fn normalize_simple_grammar_returns_rules() {
    let mut g = arith();
    let new_rules = g.normalize();
    // normalize may return auxiliary rules; original rules remain
    assert!(!g.rules.is_empty());
    // New rules list can be empty if nothing to expand
    let _ = new_rules;
}

#[test]
fn normalize_epsilon_preserved() {
    let mut g = GrammarBuilder::new("t")
        .rule("empty", vec![])
        .start("empty")
        .build();
    let _ = g.normalize();
    let id = g.find_symbol_by_name("empty").unwrap();
    let rules = g.get_rules_for_symbol(id).unwrap();
    assert!(rules.iter().any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon))));
}

#[test]
fn normalize_idempotent_on_simple_grammar() {
    let mut g1 = arith();
    let mut g2 = arith();
    g1.normalize();
    g2.normalize();
    g2.normalize(); // second normalize
    assert_eq!(g1, g2);
}

#[test]
fn validate_empty_grammar_succeeds() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.validate().is_ok());
}

#[test]
fn validate_simple_grammar_succeeds() {
    let g = arith();
    assert!(g.validate().is_ok());
}

#[test]
fn optimize_does_not_panic_on_empty() {
    let mut g = GrammarBuilder::new("t").build();
    g.optimize(); // should not panic
}

#[test]
fn optimize_does_not_panic_on_arith() {
    let mut g = arith();
    g.optimize();
    // grammar should still be valid
    assert!(!g.rules.is_empty());
}

#[test]
fn find_symbol_by_name_existing() {
    let g = arith();
    let id = g.find_symbol_by_name("expr");
    assert!(id.is_some());
}

#[test]
fn find_symbol_by_name_missing() {
    let g = arith();
    let id = g.find_symbol_by_name("nonexistent");
    assert!(id.is_none());
}

#[test]
fn start_symbol_returns_first_rule() {
    let g = arith();
    let start = g.start_symbol();
    assert!(start.is_some());
    let name = g.rule_names.get(&start.unwrap()).unwrap();
    assert_eq!(name, "expr");
}

#[test]
fn check_empty_terminals_on_valid_grammar() {
    let g = arith();
    assert!(g.check_empty_terminals().is_ok());
}

// ===========================================================================
// 6. Precedence on rules (5 tests)
// ===========================================================================

#[test]
fn rule_with_precedence_left() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym(&g, "e");
    let prec_rule = g.rules[&eid].iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

#[test]
fn rule_with_precedence_right() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("e", vec!["e", "^", "e"], 2, Associativity::Right)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym(&g, "e");
    let prec_rule = g.rules[&eid].iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn rule_without_precedence_has_none() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym(&g, "e");
    assert!(g.rules[&eid][0].precedence.is_none());
    assert!(g.rules[&eid][0].associativity.is_none());
}

#[test]
fn multiple_precedence_levels_on_different_rules() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym(&g, "e");
    let precs: Vec<_> = g.rules[&eid]
        .iter()
        .filter_map(|r| r.precedence)
        .collect();
    assert_eq!(precs.len(), 2);
    assert!(precs.contains(&PrecedenceKind::Static(1)));
    assert!(precs.contains(&PrecedenceKind::Static(2)));
}

#[test]
fn precedence_associativity_none() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("==", "==")
        .rule_with_precedence("e", vec!["e", "==", "e"], 1, Associativity::None)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym(&g, "e");
    let prec_rule = g.rules[&eid].iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::None));
}

// ===========================================================================
// 7. Factory helpers and complex grammars (5 tests)
// ===========================================================================

#[test]
fn python_like_factory_has_nullable_start() {
    let g = GrammarBuilder::python_like();
    let module_id = g.find_symbol_by_name("module").unwrap();
    let rules = g.get_rules_for_symbol(module_id).unwrap();
    // One alternative is epsilon (empty module)
    assert!(rules.iter().any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon))));
}

#[test]
fn python_like_factory_has_externals() {
    let g = GrammarBuilder::python_like();
    assert!(!g.externals.is_empty());
    let ext_names: Vec<_> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(ext_names.contains(&"INDENT"));
    assert!(ext_names.contains(&"DEDENT"));
}

#[test]
fn python_like_factory_has_extras() {
    let g = GrammarBuilder::python_like();
    assert!(!g.extras.is_empty());
}

#[test]
fn javascript_like_factory_has_tokens() {
    let g = GrammarBuilder::javascript_like();
    let names = token_names(&g);
    assert!(names.contains("function"));
    assert!(names.contains("var"));
    assert!(names.contains("return"));
}

#[test]
fn javascript_like_factory_has_rules() {
    let g = GrammarBuilder::javascript_like();
    assert!(!g.rules.is_empty());
    assert!(g.find_symbol_by_name("program").is_some());
}

// ===========================================================================
// 8. Symbol ID properties (5 tests)
// ===========================================================================

#[test]
fn symbol_id_is_copy() {
    let g = arith();
    let id = sym(&g, "expr");
    let id2 = id; // Copy, not move
    assert_eq!(id, id2);
}

#[test]
fn symbol_id_display() {
    let id = SymbolId(42);
    let s = format!("{id}");
    assert!(!s.is_empty());
}

#[test]
fn symbol_id_equality() {
    assert_eq!(SymbolId(1), SymbolId(1));
    assert_ne!(SymbolId(1), SymbolId(2));
}

#[test]
fn symbol_id_hash_in_set() {
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1)); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn production_ids_assigned_uniquely() {
    let g = arith();
    let expr_id = sym(&g, "expr");
    let rules = &g.rules[&expr_id];
    let prod_ids: HashSet<_> = rules.iter().map(|r| r.production_id).collect();
    assert_eq!(prod_ids.len(), rules.len());
}
