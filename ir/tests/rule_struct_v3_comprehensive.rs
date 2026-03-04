// Comprehensive tests for Rule struct v3
// Tests rule construction, field access, and serialization

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Symbol};

#[test]
fn rule_lhs_is_symbol_id() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let _ = rule.lhs.0; // u16
}

#[test]
fn rule_rhs_is_vec_symbol() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(!rule.rhs.is_empty());
}

#[test]
fn rule_multiple_rhs_symbols() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 3);
}

#[test]
fn rule_production_id_field() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let _ = rule.production_id.0; // u16
}

#[test]
fn rule_associativity_none_default() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.associativity.is_none());
}

#[test]
fn rule_with_left_assoc() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .token("op", r"\+")
        .rule("e", vec!["a"])
        .rule_with_precedence("e", vec!["e", "op", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let has_left = g
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::Left));
    assert!(has_left);
}

#[test]
fn rule_with_right_assoc() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .token("pow", r"\^")
        .rule("e", vec!["a"])
        .rule_with_precedence("e", vec!["e", "pow", "e"], 2, Associativity::Right)
        .start("e")
        .build();
    let has_right = g
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::Right));
    assert!(has_right);
}

#[test]
fn rule_terminal_in_rhs() {
    let g = GrammarBuilder::new("rv3")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.rhs.iter().any(|s| matches!(s, Symbol::Terminal(_))));
}

#[test]
fn rule_nonterminal_in_rhs() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["a"])
        .start("s")
        .build();
    let has_nt = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::NonTerminal(_))));
    assert!(has_nt);
}

#[test]
fn rule_count_matches_builder() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("x", vec!["a", "b"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn rule_serde_roundtrip() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn rule_clone_preserves_structure() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap().clone();
    assert!(!rule.rhs.is_empty());
}
