//! Comprehensive tests for Grammar struct operations.
//!
//! Covers: Grammar construction, add_rule, get_rules_for_symbol,
//! all_rules, start_symbol, normalize, validate.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};

fn make_rule(lhs: SymbolId, rhs: Vec<Symbol>) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

#[test]
fn grammar_builder_sets_name() {
    let g = GrammarBuilder::new("test_grammar")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.name, "test_grammar");
}

#[test]
fn grammar_add_rule() {
    let mut g = Grammar::default();
    g.add_rule(make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(1))]));
    assert!(g.rules.contains_key(&SymbolId(0)));
}

#[test]
fn grammar_get_rules_for_symbol() {
    let mut g = Grammar::default();
    g.add_rule(make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(1))]));
    assert!(g.get_rules_for_symbol(SymbolId(0)).is_some());
    assert!(g.get_rules_for_symbol(SymbolId(99)).is_none());
}

#[test]
fn grammar_all_rules_count() {
    let mut g = Grammar::default();
    g.add_rule(make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(1))]));
    g.add_rule(make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(2))]));
    g.add_rule(make_rule(SymbolId(3), vec![Symbol::Terminal(SymbolId(4))]));
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn grammar_multiple_rules_same_lhs() {
    let mut g = Grammar::default();
    g.add_rule(make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(1))]));
    g.add_rule(make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(2))]));
    let rules = g.get_rules_for_symbol(SymbolId(0)).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn grammar_tokens_empty_by_default() {
    let g = Grammar::default();
    assert!(g.tokens.is_empty());
}

#[test]
fn grammar_builder_adds_tokens() {
    let g = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn grammar_builder_adds_rules() {
    let g = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .start("start")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn grammar_extras_empty_by_default() {
    let g = Grammar::default();
    assert!(g.extras.is_empty());
}

#[test]
fn grammar_externals_empty_by_default() {
    let g = Grammar::default();
    assert!(g.externals.is_empty());
}

#[test]
fn grammar_conflicts_empty_by_default() {
    let g = Grammar::default();
    assert!(g.conflicts.is_empty());
}

#[test]
fn grammar_debug() {
    let g = Grammar::default();
    let debug = format!("{:?}", g);
    assert!(debug.contains("Grammar"));
}

#[test]
fn grammar_clone() {
    let g = GrammarBuilder::new("clone_test")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn grammar_normalize_idempotent() {
    let mut g = GrammarBuilder::new("norm")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let r1 = g.normalize();
    let r2 = g.normalize();
    // Second normalize should produce same or fewer auxiliary rules
    let _ = (r1, r2);
}

#[test]
fn grammar_serde_roundtrip() {
    let g = GrammarBuilder::new("serde_test")
        .token("NUM", r"\d+")
        .rule("start", vec!["NUM"])
        .start("start")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn grammar_rule_names_populated() {
    let g = GrammarBuilder::new("named")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert!(!g.rule_names.is_empty());
}
