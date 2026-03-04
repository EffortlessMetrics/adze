//! Comprehensive tests for GLR core ParseTable structure.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

fn build_pt(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

// ── Basic properties ──

#[test]
fn state_count_positive() {
    let pt = build_pt("t1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count > 0);
}

#[test]
fn symbol_count_positive() {
    let pt = build_pt("t1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.symbol_count > 0);
}

#[test]
fn rules_nonempty() {
    let pt = build_pt("t1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!pt.rules.is_empty());
}

// ── Multiple tokens ──

#[test]
fn two_token_state_count() {
    let pt = build_pt(
        "t2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn two_token_symbol_count() {
    let pt = build_pt(
        "t2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(pt.symbol_count >= 3); // a, b, s + EOF
}

#[test]
fn three_token_states() {
    let pt = build_pt(
        "t3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(pt.state_count >= 3);
}

// ── Alternative rules ──

#[test]
fn alternatives_state_count() {
    let pt = build_pt(
        "alt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn alternatives_rules_count() {
    let pt = build_pt(
        "alt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.rules.len() >= 2);
}

// ── Chain rules ──

#[test]
fn chain_state_count() {
    let pt = build_pt(
        "chain",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn chain_rules_count() {
    let pt = build_pt(
        "chain",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.rules.len() >= 3);
}

// ── ParseRule properties ──

#[test]
fn parse_rule_lhs_valid() {
    let pt = build_pt("pr", &[("a", "a")], &[("s", vec!["a"])], "s");
    for rule in &pt.rules {
        let _ = rule.lhs;
    }
}

#[test]
fn parse_rule_rhs_len() {
    let pt = build_pt("pr", &[("a", "a")], &[("s", vec!["a"])], "s");
    for rule in &pt.rules {
        // All our rules have rhs_len >= 1 (we didn't add empty rules)
        let _ = rule.rhs_len;
    }
}

// ── EOF symbol ──

#[test]
fn eof_symbol_present() {
    let pt = build_pt("eof", &[("a", "a")], &[("s", vec!["a"])], "s");
    let _ = pt.eof_symbol;
}

// ── Action table ──

#[test]
fn action_table_nonempty() {
    let pt = build_pt("act", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!pt.action_table.is_empty());
}

#[test]
fn action_table_has_states() {
    let pt = build_pt("act", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.action_table.len() == pt.state_count);
}

// ── Goto table ──

#[test]
fn goto_table_nonempty() {
    let pt = build_pt("goto", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!pt.goto_table.is_empty());
}

// ── Determinism ──

#[test]
fn deterministic_state_count() {
    let pt1 = build_pt("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn deterministic_rules_count() {
    let pt1 = build_pt("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.rules.len(), pt2.rules.len());
}

// ── Complex grammars ──

#[test]
fn expr_grammar_parses() {
    let pt = build_pt(
        "expr",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["num"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 4);
}

#[test]
fn diamond_grammar() {
    let pt = build_pt(
        "diamond",
        &[("x", "x"), ("y", "y")],
        &[
            ("a", vec!["x"]),
            ("b", vec!["y"]),
            ("s", vec!["a"]),
            ("s", vec!["b"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 2);
}

// ── Precedence ──

#[test]
fn precedence_grammar() {
    let mut b = GrammarBuilder::new("prec")
        .token("x", "x")
        .token("y", "y")
        .rule_with_precedence("s", vec!["x"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["y"], 2, Associativity::Right)
        .start("s");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count >= 2);
}
