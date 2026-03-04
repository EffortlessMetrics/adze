//! Comprehensive tests for ParseTable structure and properties v2.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;

fn make_table(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (tn, tp) in tokens {
        b = b.token(tn, tp);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

// ── Simple grammar tables ──

#[test]
fn v2_simple_state_count() {
    let t = make_table("s", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert!(t.state_count > 0);
}

#[test]
fn v2_simple_symbol_count() {
    let t = make_table("s", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert!(t.symbol_count > 0);
}

#[test]
fn v2_simple_rules() {
    let t = make_table("s", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert!(!t.rules.is_empty());
}

#[test]
fn v2_simple_action_table() {
    let t = make_table("s", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn v2_simple_goto_table() {
    let t = make_table("s", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert_eq!(t.goto_table.len(), t.state_count);
}

// ── Multi-token grammar ──

#[test]
fn v2_multi_token_states() {
    let t = make_table(
        "m",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(t.state_count >= 4);
}

#[test]
fn v2_multi_token_symbols() {
    let t = make_table(
        "m",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(t.symbol_count >= 3);
}

// ── Alternative grammar ──

#[test]
fn v2_alt_rules() {
    let t = make_table(
        "alt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(t.rules.len() >= 2);
}

#[test]
fn v2_alt_states() {
    let t = make_table(
        "alt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(t.state_count >= 2);
}

// ── Chain grammar ──

#[test]
fn v2_chain_rules() {
    let t = make_table(
        "chain",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(t.rules.len() >= 3);
}

// ── ParseRule properties ──

#[test]
fn v2_rule_lhs_valid() {
    let t = make_table("r", &[("x", "x")], &[("s", vec!["x"])], "s");
    for rule in &t.rules {
        // lhs should be a valid SymbolId
        let _ = rule.lhs.0;
    }
}

#[test]
fn v2_rule_rhs_len() {
    let t = make_table(
        "r",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    // At least one rule should have rhs_len == 2
    assert!(t.rules.iter().any(|r| r.rhs_len == 2));
}

#[test]
fn v2_rule_single_rhs() {
    let t = make_table("r", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert!(t.rules.iter().any(|r| r.rhs_len == 1));
}

// ── Eof symbol ──

#[test]
fn v2_eof_symbol_exists() {
    let t = make_table("e", &[("x", "x")], &[("s", vec!["x"])], "s");
    let _ = t.eof_symbol;
}

// ── Determinism ──

#[test]
fn v2_deterministic_state_count() {
    let t1 = make_table("d", &[("x", "x")], &[("s", vec!["x"])], "s");
    let t2 = make_table("d", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn v2_deterministic_symbol_count() {
    let t1 = make_table("d", &[("x", "x")], &[("s", vec!["x"])], "s");
    let t2 = make_table("d", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert_eq!(t1.symbol_count, t2.symbol_count);
}

#[test]
fn v2_deterministic_rules_count() {
    let t1 = make_table("d", &[("x", "x")], &[("s", vec!["x"])], "s");
    let t2 = make_table("d", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert_eq!(t1.rules.len(), t2.rules.len());
}

// ── Scale: many tokens ──

#[test]
fn v2_many_tokens_table() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..20 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 20);
}

// ── Precedence grammar tables ──

#[test]
fn v2_prec_table() {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, adze_ir::Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, adze_ir::Associativity::Left)
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 3);
}
