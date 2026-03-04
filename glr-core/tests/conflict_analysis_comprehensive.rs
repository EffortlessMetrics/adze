#![cfg(feature = "test-api")]
//! Comprehensive tests for conflict analysis and resolution.

use adze_glr_core::advanced_conflict::{ConflictAnalyzer, ConflictStats, PrecedenceResolver};
use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

fn build_table(grammar: &mut adze_ir::Grammar) -> ParseTable {
    grammar.normalize();
    let ff = FirstFollowSets::compute(grammar).unwrap();
    build_lr1_automaton(grammar, &ff).unwrap()
}

// ── ConflictAnalyzer construction ──

#[test]
fn analyzer_new() {
    let _a = ConflictAnalyzer::new();
}

#[test]
fn analyzer_analyze_simple() {
    let mut g = GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn analyzer_get_stats() {
    let mut g = GrammarBuilder::new("gs")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let _ = a.analyze_table(&pt);
    let stats = a.get_stats();
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// ── ConflictStats ──

#[test]
fn conflict_stats_debug() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 0,
        reduce_reduce_conflicts: 0,
        precedence_resolved: 0,
        associativity_resolved: 0,
        explicit_glr: 0,
        default_resolved: 0,
    };
    let d = format!("{:?}", stats);
    assert!(d.contains("shift_reduce"));
}

#[test]
fn conflict_stats_zero_simple_grammar() {
    let mut g = GrammarBuilder::new("zero")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    assert_eq!(stats.explicit_glr, 0);
}

// ── Two-alternative (no recursion) ──

#[test]
fn analyzer_two_alternatives() {
    let mut g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    // Simple alternatives shouldn't have conflicts
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// ── Chain grammar ──

#[test]
fn analyzer_chain() {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// ── Ambiguous grammar ──

#[test]
fn analyzer_ambiguous_grammar() {
    let mut g = GrammarBuilder::new("amb")
        .token("n", "n")
        .token("plus", "+")
        .rule("e", vec!["n"])
        .rule("e", vec!["e", "plus", "e"])
        .start("e")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    // Ambiguous grammar may have conflicts → GLR
    let total = stats.shift_reduce_conflicts + stats.reduce_reduce_conflicts;
    let _ = total; // just check it doesn't panic
}

// ── Precedence grammar ──

#[test]
fn analyzer_precedence_grammar() {
    let mut g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    let _ = stats;
}

// ── Multiple analyzes ──

#[test]
fn analyzer_multiple_runs() {
    let mut g = GrammarBuilder::new("multi")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let s1 = a.analyze_table(&pt);
    let s2 = a.analyze_table(&pt);
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
}

// ── PrecedenceResolver ──

#[test]
fn prec_resolver_new() {
    let g = GrammarBuilder::new("pr")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let _resolver = PrecedenceResolver::new(&g);
}

#[test]
fn prec_resolver_with_precedence() {
    let g = GrammarBuilder::new("prp")
        .token("n", "n")
        .token("plus", "+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let _resolver = PrecedenceResolver::new(&g);
}

// ── Large grammar conflict analysis ──

#[test]
fn analyzer_large_grammar() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    // Large non-recursive grammar should be conflict-free
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// ── Sequence grammar ──

#[test]
fn analyzer_sequence() {
    let mut g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// ── Multiple nonterminals ──

#[test]
fn analyzer_multi_nonterminals() {
    let mut g = GrammarBuilder::new("multi_nt")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let mut a = ConflictAnalyzer::new();
    let stats = a.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// ── Determinism of analysis ──

#[test]
fn analyzer_deterministic() {
    let make = || {
        let mut g = GrammarBuilder::new("det")
            .token("n", "n")
            .token("plus", "+")
            .rule("e", vec!["n"])
            .rule("e", vec!["e", "plus", "e"])
            .start("e")
            .build();
        let pt = build_table(&mut g);
        let mut a = ConflictAnalyzer::new();
        a.analyze_table(&pt)
    };
    let s1 = make();
    let s2 = make();
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
    assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    assert_eq!(s1.explicit_glr, s2.explicit_glr);
}
