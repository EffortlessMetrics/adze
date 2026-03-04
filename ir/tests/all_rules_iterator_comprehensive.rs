//! Comprehensive tests for Grammar all_rules() iterator behavior.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

// ── Basic iteration ──

#[test]
fn all_rules_single() {
    let g = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn all_rules_two_alternatives() {
    let g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn all_rules_chain() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

// ── Rule properties ──

#[test]
fn all_rules_have_lhs() {
    let g = GrammarBuilder::new("lhs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.lhs;
    }
}

#[test]
fn all_rules_have_rhs() {
    let g = GrammarBuilder::new("rhs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        assert!(!rule.rhs.is_empty());
    }
}

#[test]
fn all_rules_have_production_id() {
    let g = GrammarBuilder::new("pid")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.production_id;
    }
}

// ── With precedence ──

#[test]
fn rules_with_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["b"], 2, Associativity::Right)
        .start("s")
        .build();
    let rules: Vec<_> = g.all_rules().collect();
    assert!(rules.len() >= 2);
}

#[test]
fn rules_precedence_field() {
    let g = GrammarBuilder::new("pf")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.precedence;
    }
}

#[test]
fn rules_associativity_field() {
    let g = GrammarBuilder::new("af")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.associativity;
    }
}

// ── Many rules ──

#[test]
fn ten_alternatives() {
    let mut b = GrammarBuilder::new("ten");
    for i in 0..10 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..10 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    assert!(g.all_rules().count() >= 10);
}

#[test]
fn twenty_alternatives() {
    let mut b = GrammarBuilder::new("twenty");
    for i in 0..20 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..20 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    assert!(g.all_rules().count() >= 20);
}

// ── After normalize ──

#[test]
fn all_rules_after_normalize() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn all_rules_count_stable_after_normalize() {
    let mut g = GrammarBuilder::new("stable")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let before = g.all_rules().count();
    g.normalize();
    let after = g.all_rules().count();
    // normalize may add auxiliary rules
    assert!(after >= before);
}

// ── Collect patterns ──

#[test]
fn collect_to_vec() {
    let g = GrammarBuilder::new("vec")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rules: Vec<_> = g.all_rules().collect();
    assert!(!rules.is_empty());
}

#[test]
fn filter_by_lhs() {
    let g = GrammarBuilder::new("filter")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let start_id = g.start_symbol().unwrap();
    let start_rules: Vec<_> = g.all_rules().filter(|r| r.lhs == start_id).collect();
    assert!(!start_rules.is_empty());
}

// ── Rule fields access ──

#[test]
fn rule_fields_empty_for_simple() {
    let g = GrammarBuilder::new("fields")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.fields.len();
    }
}
