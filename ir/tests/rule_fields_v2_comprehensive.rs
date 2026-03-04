//! Comprehensive tests for Rule struct patterns and field access.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

#[test]
fn rule_lhs_field() {
    let g = GrammarBuilder::new("rl")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for r in g.all_rules() {
        let _ = r.lhs;
    }
}

#[test]
fn rule_rhs_field() {
    let g = GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for r in g.all_rules() {
        assert!(!r.rhs.is_empty());
    }
}

#[test]
fn rule_prec_field() {
    let g = GrammarBuilder::new("rp")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .start("s")
        .build();
    for r in g.all_rules() {
        let _ = &r.precedence;
    }
}

#[test]
fn rule_assoc_field() {
    let g = GrammarBuilder::new("ra")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Right)
        .start("s")
        .build();
    for r in g.all_rules() {
        let _ = &r.associativity;
    }
}

#[test]
fn rule_fields_field() {
    let g = GrammarBuilder::new("rf")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for r in g.all_rules() {
        let _ = r.fields.len();
    }
}

#[test]
fn rule_prod_id_field() {
    let g = GrammarBuilder::new("rpid")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for r in g.all_rules() {
        let _ = r.production_id;
    }
}

#[test]
fn rule_rhs_len_two() {
    let g = GrammarBuilder::new("rl2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let r = g.all_rules().next().unwrap();
    assert_eq!(r.rhs.len(), 2);
}

#[test]
fn rule_rhs_len_three() {
    let g = GrammarBuilder::new("rl3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let r = g.all_rules().next().unwrap();
    assert_eq!(r.rhs.len(), 3);
}

#[test]
fn rule_clone_works() {
    let g = GrammarBuilder::new("rc")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let r = g.all_rules().next().unwrap().clone();
    assert_eq!(r.rhs.len(), 1);
}

#[test]
fn rule_debug_nonempty() {
    let g = GrammarBuilder::new("rd")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let r = g.all_rules().next().unwrap();
    assert!(!format!("{:?}", r).is_empty());
}

#[test]
fn rule_multiple_lhs() {
    let g = GrammarBuilder::new("rm")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let lhs: Vec<_> = g.all_rules().map(|r| r.lhs).collect();
    assert!(lhs.len() >= 2);
}

#[test]
fn rule_with_empty_rhs_input() {
    let g = GrammarBuilder::new("re")
        .token("a", "a")
        .rule("s", vec![])
        .start("s")
        .build();
    // Empty RHS may get a sentinel added
    let r = g.all_rules().next().unwrap();
    let _ = r.rhs.len();
}
