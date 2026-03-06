//! Comprehensive tests for GLR core ConflictAnalyzer.

use adze_glr_core::advanced_conflict::ConflictAnalyzer;
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

fn build_pt(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> adze_glr_core::ParseTable {
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

// ── Basic construction ──

#[test]
fn conflict_analyzer_new() {
    let _ca = ConflictAnalyzer::new();
}

#[test]
fn analyze_simple_table() {
    let pt = build_pt("t1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats;
}

#[test]
fn analyze_two_token_table() {
    let pt = build_pt(
        "t2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats;
}

// ── Conflict stats ──

#[test]
fn stats_default_resolved() {
    let pt = build_pt("t1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats.default_resolved;
}

// ── Alternative grammars ──

#[test]
fn analyze_alternatives() {
    let pt = build_pt(
        "alt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats;
}

#[test]
fn analyze_three_alternatives() {
    let pt = build_pt(
        "alt3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ── Chain grammars ──

#[test]
fn analyze_chain() {
    let pt = build_pt(
        "chain",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ── Recursive grammars ──

#[test]
fn analyze_left_recursive() {
    let pt = build_pt(
        "leftrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["s", "a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_right_recursive() {
    let pt = build_pt(
        "rightrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "s"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ── With precedence ──

#[test]
fn analyze_with_precedence() {
    let b = GrammarBuilder::new("prec")
        .token("x", "x")
        .token("y", "y")
        .rule_with_precedence("s", vec!["x"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["y"], 2, Associativity::Right)
        .start("s");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ── Expression grammar ──

#[test]
fn analyze_expr_grammar() {
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
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ── Multiple analyses ──

#[test]
fn multiple_analyses() {
    let pt1 = build_pt("t1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("t2", &[("b", "b")], &[("s", vec!["b"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let _s1 = ca.analyze_table(&pt1);
    let _s2 = ca.analyze_table(&pt2);
}

// ── Determinism ──

#[test]
fn analysis_deterministic() {
    let pt = build_pt("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let s1 = ca.analyze_table(&pt);
    let s2 = ca.analyze_table(&pt);
    assert_eq!(s1.default_resolved, s2.default_resolved);
}

// ── Various grammar sizes ──

#[test]
fn analyze_five_tokens() {
    let pt = build_pt(
        "five",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_ten_alternatives() {
    let tokens: Vec<(String, String)> = (0..10)
        .map(|i| (format!("t{}", i), format!("t{}", i)))
        .collect();
    let tok_refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let rules: Vec<(&str, Vec<&str>)> = tokens
        .iter()
        .map(|(n, _)| ("s", vec![n.as_str()]))
        .collect();
    let pt = build_pt("tenalt", &tok_refs, &rules, "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ── Diamond pattern ──

#[test]
fn analyze_diamond() {
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
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}
