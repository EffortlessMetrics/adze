//! Comprehensive tests for IR Rule struct and rule manipulation.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Rule, SymbolId};

// ── Rule from builder ──

#[test]
fn rule_has_lhs() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.lhs;
    }
}

#[test]
fn rule_has_rhs() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        assert!(!rule.rhs.is_empty());
    }
}

#[test]
fn rule_has_production_id() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.production_id;
    }
}

// ── Rule rhs length ──

#[test]
fn rule_rhs_single() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let first_rule = g.all_rules().next().unwrap();
    assert_eq!(first_rule.rhs.len(), 1);
}

#[test]
fn rule_rhs_sequence() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let first_rule = g.all_rules().next().unwrap();
    assert_eq!(first_rule.rhs.len(), 2);
}

#[test]
fn rule_rhs_triple() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let first_rule = g.all_rules().next().unwrap();
    assert_eq!(first_rule.rhs.len(), 3);
}

// ── Multiple rules ──

#[test]
fn multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn multiple_rules_different_lhs() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 3);
}

// ── Precedence rules ──

#[test]
fn rule_with_left_precedence() {
    let g = GrammarBuilder::new("r")
        .token("n", "n")
        .token("plus", "+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let rules: Vec<_> = g.all_rules().collect();
    assert!(rules.len() >= 2);
}

#[test]
fn rule_with_right_precedence() {
    let g = GrammarBuilder::new("r")
        .token("n", "n")
        .token("eq", "=")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "eq", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

// ── Rule debug ──

#[test]
fn rule_debug() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let first_rule = g.all_rules().next().unwrap();
    let d = format!("{:?}", first_rule);
    assert!(!d.is_empty());
}

// ── Rule clone ──

#[test]
fn rule_clone() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let first_rule = g.all_rules().next().unwrap();
    let cloned = first_rule.clone();
    assert_eq!(first_rule.lhs, cloned.lhs);
    assert_eq!(first_rule.rhs.len(), cloned.rhs.len());
}

// ── Rule after normalize ──

#[test]
fn rule_after_normalize() {
    let mut g = GrammarBuilder::new("r")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    for rule in g.all_rules() {
        let _ = rule.lhs;
        let _ = &rule.rhs;
    }
}

// ── Large rule set ──

#[test]
fn many_rules() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..20 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    assert!(g.all_rules().count() >= 20);
}

// ── Rules from chain grammar ──

#[test]
fn chain_rules() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 3);
}

// ── Rule fields ──

#[test]
fn rule_fields_empty() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let first_rule = g.all_rules().next().unwrap();
    // Simple rules have no fields
    assert!(first_rule.fields.is_empty());
}

// ── Rule with many rhs symbols ──

#[test]
fn rule_long_rhs() {
    let mut b = GrammarBuilder::new("long");
    let mut rhs = vec![];
    for i in 0..10 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n);
        rhs.push(n);
    }
    let g = b.rule("s", rhs).start("s").build();
    let first_rule = g.all_rules().next().unwrap();
    assert_eq!(first_rule.rhs.len(), 10);
}
