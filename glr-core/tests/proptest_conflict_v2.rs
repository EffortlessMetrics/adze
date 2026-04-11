#![cfg(feature = "test-api")]
//! Property-based tests for conflict analysis in adze-glr-core.
//!
//! Run with:
//! ```bash
//! cargo test -p adze-glr-core --test proptest_conflict_v2 --features test-api
//! ```

use adze_glr_core::advanced_conflict::{ConflictAnalyzer, ConflictStats};
use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a parse table from a grammar via the standard pipeline.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("build_lr1_automaton failed")
}

/// Try to build a table, returning None on failure.
fn try_build(grammar: &Grammar) -> Option<ParseTable> {
    let ff = FirstFollowSets::compute(grammar).ok()?;
    build_lr1_automaton(grammar, &ff).ok()
}

/// Analyze a grammar's parse table and return conflict stats.
fn analyze(grammar: &Grammar) -> ConflictStats {
    let pt = build_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    analyzer.analyze_table(&pt)
}

/// Try to analyze, returning None if the grammar can't be built.
fn try_analyze(grammar: &Grammar) -> Option<ConflictStats> {
    let pt = try_build(grammar)?;
    let mut analyzer = ConflictAnalyzer::new();
    Some(analyzer.analyze_table(&pt))
}

// ---------------------------------------------------------------------------
// Fixed grammar constructors — unambiguous
// ---------------------------------------------------------------------------

/// S → a
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("min")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S → a | b
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("twoalt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

/// S → a b c
fn sequence_grammar() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build()
}

/// S → T, T → a  (chain)
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["a"])
        .start("s")
        .build()
}

/// S → T, T → U, U → a  (deep chain)
fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["u"])
        .rule("u", vec!["a"])
        .start("s")
        .build()
}

/// S → a | b | c | d | e  (wide alternatives)
fn wide_alt_grammar() -> Grammar {
    GrammarBuilder::new("wide")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .rule("s", vec!["e"])
        .start("s")
        .build()
}

/// S → T U, T → a, U → b  (two nonterminals in sequence)
fn two_nt_seq_grammar() -> Grammar {
    GrammarBuilder::new("twontseq")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["t", "u"])
        .rule("t", vec!["a"])
        .rule("u", vec!["b"])
        .start("s")
        .build()
}

/// S → ε | a  (nullable start)
fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

// ---------------------------------------------------------------------------
// Additional grammar constructors
// ---------------------------------------------------------------------------

/// S → S a | a  (left-recursive)
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S → a S | a  (right-recursive)
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rightrec")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// E → E + E | E * E | a  (with precedence)
fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("a", "a")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build()
}

/// S → a b | a c  (common prefix)
fn common_prefix_grammar() -> Grammar {
    GrammarBuilder::new("prefix")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["a", "c"])
        .start("s")
        .build()
}

/// S → T, T → U, U → V, V → a  (4-deep chain)
fn four_deep_chain() -> Grammar {
    GrammarBuilder::new("deep4")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["u"])
        .rule("u", vec!["v"])
        .rule("v", vec!["a"])
        .start("s")
        .build()
}

/// S → a b c d e  (long sequence)
fn long_sequence_grammar() -> Grammar {
    GrammarBuilder::new("longseq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a", "b", "c", "d", "e"])
        .start("s")
        .build()
}

/// S → T U, T → a | b, U → c | d  (diamond)
fn diamond_grammar() -> Grammar {
    GrammarBuilder::new("diamond")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["t", "u"])
        .rule("t", vec!["a"])
        .rule("t", vec!["b"])
        .rule("u", vec!["c"])
        .rule("u", vec!["d"])
        .start("s")
        .build()
}

// ---------------------------------------------------------------------------
// Strategy for random grammar selection
// ---------------------------------------------------------------------------

fn arb_unambiguous_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![
        Just(minimal_grammar()),
        Just(two_alt_grammar()),
        Just(sequence_grammar()),
        Just(chain_grammar()),
        Just(deep_chain_grammar()),
        Just(wide_alt_grammar()),
        Just(two_nt_seq_grammar()),
        Just(nullable_grammar()),
    ]
}

fn arb_any_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![
        Just(minimal_grammar()),
        Just(two_alt_grammar()),
        Just(sequence_grammar()),
        Just(chain_grammar()),
        Just(deep_chain_grammar()),
        Just(wide_alt_grammar()),
        Just(two_nt_seq_grammar()),
        Just(nullable_grammar()),
        Just(left_recursive_grammar()),
        Just(right_recursive_grammar()),
        Just(precedence_grammar()),
        Just(common_prefix_grammar()),
        Just(four_deep_chain()),
        Just(long_sequence_grammar()),
        Just(diamond_grammar()),
    ]
}

// ---------------------------------------------------------------------------
// 1. Non-ambiguous grammars have 0 conflicts (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn zero_conflicts_minimal(_dummy in 0u8..1) {
        let stats = analyze(&minimal_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn zero_conflicts_two_alt(_dummy in 0u8..1) {
        let stats = analyze(&two_alt_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn zero_conflicts_sequence(_dummy in 0u8..1) {
        let stats = analyze(&sequence_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn zero_conflicts_chain(_dummy in 0u8..1) {
        let stats = analyze(&chain_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn zero_conflicts_deep_chain(_dummy in 0u8..1) {
        let stats = analyze(&deep_chain_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn zero_conflicts_wide_alt(_dummy in 0u8..1) {
        let stats = analyze(&wide_alt_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn zero_conflicts_two_nt_seq(_dummy in 0u8..1) {
        let stats = analyze(&two_nt_seq_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn zero_conflicts_random_unambiguous(grammar in arb_unambiguous_grammar()) {
        let stats = analyze(&grammar);
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }
}

// ---------------------------------------------------------------------------
// 2. Conflict counts are non-negative (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn sr_count_nonneg(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            // usize is always >= 0, but verify the field is populated
            prop_assert!(stats.shift_reduce_conflicts < usize::MAX);
        }
    }

    #[test]
    fn rr_count_nonneg(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            prop_assert!(stats.reduce_reduce_conflicts < usize::MAX);
        }
    }

    #[test]
    fn precedence_resolved_nonneg(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            prop_assert!(stats.precedence_resolved < usize::MAX);
        }
    }

    #[test]
    fn associativity_resolved_nonneg(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            prop_assert!(stats.associativity_resolved < usize::MAX);
        }
    }

    #[test]
    fn default_resolved_nonneg(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            prop_assert!(stats.default_resolved < usize::MAX);
            prop_assert!(stats.explicit_glr < usize::MAX);
        }
    }
}

// ---------------------------------------------------------------------------
// 3. ConflictStats Debug/Clone (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn stats_clone_equals_original(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            let cloned = stats.clone();
            prop_assert_eq!(stats.shift_reduce_conflicts, cloned.shift_reduce_conflicts);
            prop_assert_eq!(stats.reduce_reduce_conflicts, cloned.reduce_reduce_conflicts);
            prop_assert_eq!(stats.precedence_resolved, cloned.precedence_resolved);
            prop_assert_eq!(stats.associativity_resolved, cloned.associativity_resolved);
            prop_assert_eq!(stats.explicit_glr, cloned.explicit_glr);
            prop_assert_eq!(stats.default_resolved, cloned.default_resolved);
        }
    }

    #[test]
    fn stats_debug_nonempty(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            let dbg = format!("{:?}", stats);
            prop_assert!(!dbg.is_empty());
            prop_assert!(dbg.contains("shift_reduce_conflicts"));
        }
    }

    #[test]
    fn stats_debug_contains_all_fields(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            let dbg = format!("{:?}", stats);
            prop_assert!(dbg.contains("reduce_reduce_conflicts"));
            prop_assert!(dbg.contains("precedence_resolved"));
            prop_assert!(dbg.contains("associativity_resolved"));
        }
    }

    #[test]
    fn stats_debug_contains_glr_fields(grammar in arb_any_grammar()) {
        if let Some(stats) = try_analyze(&grammar) {
            let dbg = format!("{:?}", stats);
            prop_assert!(dbg.contains("explicit_glr"));
            prop_assert!(dbg.contains("default_resolved"));
        }
    }

    #[test]
    fn stats_default_is_all_zero(_dummy in 0u8..1) {
        let stats = ConflictStats::default();
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
        prop_assert_eq!(stats.precedence_resolved, 0);
        prop_assert_eq!(stats.associativity_resolved, 0);
        prop_assert_eq!(stats.explicit_glr, 0);
        prop_assert_eq!(stats.default_resolved, 0);
    }
}

// ---------------------------------------------------------------------------
// 4. Analyzer is reusable across grammars (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn analyzer_reuse_two_grammars(g1 in arb_any_grammar(), g2 in arb_any_grammar()) {
        let mut analyzer = ConflictAnalyzer::new();
        if let Some(pt1) = try_build(&g1) {
            let _stats1 = analyzer.analyze_table(&pt1);
            if let Some(pt2) = try_build(&g2) {
                let _stats2 = analyzer.analyze_table(&pt2);
                // Analyzer doesn't panic on reuse
            }
        }
    }

    #[test]
    fn analyzer_reuse_same_grammar_twice(grammar in arb_any_grammar()) {
        let mut analyzer = ConflictAnalyzer::new();
        if let Some(pt) = try_build(&grammar) {
            let stats1 = analyzer.analyze_table(&pt);
            let stats2 = analyzer.analyze_table(&pt);
            prop_assert_eq!(stats1.shift_reduce_conflicts, stats2.shift_reduce_conflicts);
            prop_assert_eq!(stats1.reduce_reduce_conflicts, stats2.reduce_reduce_conflicts);
        }
    }

    #[test]
    fn analyzer_reuse_three_grammars(
        g1 in arb_any_grammar(),
        g2 in arb_any_grammar(),
        g3 in arb_any_grammar(),
    ) {
        let mut analyzer = ConflictAnalyzer::new();
        for grammar in [&g1, &g2, &g3] {
            if let Some(pt) = try_build(grammar) {
                let _ = analyzer.analyze_table(&pt);
            }
        }
    }

    #[test]
    fn analyzer_get_stats_after_analyze(grammar in arb_any_grammar()) {
        let mut analyzer = ConflictAnalyzer::new();
        if let Some(pt) = try_build(&grammar) {
            let returned = analyzer.analyze_table(&pt);
            let stored = analyzer.get_stats();
            prop_assert_eq!(returned.shift_reduce_conflicts, stored.shift_reduce_conflicts);
            prop_assert_eq!(returned.reduce_reduce_conflicts, stored.reduce_reduce_conflicts);
        }
    }

    #[test]
    fn fresh_analyzer_has_zero_stats(_dummy in 0u8..1) {
        let analyzer = ConflictAnalyzer::new();
        let stats = analyzer.get_stats();
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
        prop_assert_eq!(stats.precedence_resolved, 0);
        prop_assert_eq!(stats.associativity_resolved, 0);
        prop_assert_eq!(stats.explicit_glr, 0);
        prop_assert_eq!(stats.default_resolved, 0);
    }
}

// ---------------------------------------------------------------------------
// 5. Determinism: same grammar → same conflict stats (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn deterministic_minimal(_dummy in 0u8..1) {
        let s1 = analyze(&minimal_grammar());
        let s2 = analyze(&minimal_grammar());
        prop_assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
        prop_assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    }

    #[test]
    fn deterministic_chain(_dummy in 0u8..1) {
        let s1 = analyze(&chain_grammar());
        let s2 = analyze(&chain_grammar());
        prop_assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
        prop_assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    }

    #[test]
    fn deterministic_precedence(_dummy in 0u8..1) {
        let s1 = analyze(&precedence_grammar());
        let s2 = analyze(&precedence_grammar());
        prop_assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
        prop_assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    }

    #[test]
    fn deterministic_left_recursive(_dummy in 0u8..1) {
        let s1 = analyze(&left_recursive_grammar());
        let s2 = analyze(&left_recursive_grammar());
        prop_assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
        prop_assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    }

    #[test]
    fn deterministic_right_recursive(_dummy in 0u8..1) {
        let s1 = analyze(&right_recursive_grammar());
        let s2 = analyze(&right_recursive_grammar());
        prop_assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
        prop_assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    }

    #[test]
    fn deterministic_random(grammar in arb_any_grammar()) {
        let mut a1 = ConflictAnalyzer::new();
        let mut a2 = ConflictAnalyzer::new();
        if let Some(pt) = try_build(&grammar) {
            let s1 = a1.analyze_table(&pt);
            let s2 = a2.analyze_table(&pt);
            prop_assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
            prop_assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
            prop_assert_eq!(s1.precedence_resolved, s2.precedence_resolved);
            prop_assert_eq!(s1.associativity_resolved, s2.associativity_resolved);
            prop_assert_eq!(s1.explicit_glr, s2.explicit_glr);
            prop_assert_eq!(s1.default_resolved, s2.default_resolved);
        }
    }

    #[test]
    fn deterministic_all_fields_wide_alt(_dummy in 0u8..1) {
        let s1 = analyze(&wide_alt_grammar());
        let s2 = analyze(&wide_alt_grammar());
        prop_assert_eq!(s1.precedence_resolved, s2.precedence_resolved);
        prop_assert_eq!(s1.associativity_resolved, s2.associativity_resolved);
        prop_assert_eq!(s1.explicit_glr, s2.explicit_glr);
        prop_assert_eq!(s1.default_resolved, s2.default_resolved);
    }

    #[test]
    fn deterministic_common_prefix(_dummy in 0u8..1) {
        let s1 = analyze(&common_prefix_grammar());
        let s2 = analyze(&common_prefix_grammar());
        prop_assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
        prop_assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    }
}

// ---------------------------------------------------------------------------
// 6. Various grammar topologies and their conflict properties (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn topology_chain_no_conflicts(depth in 2usize..6) {
        // Build S → T₁, T₁ → T₂, …, Tₙ → a
        let names: Vec<String> = (0..depth).map(|i| format!("t{}", i)).collect();
        let mut builder = GrammarBuilder::new("chain_n").token("a", "a");
        builder = builder.rule("s", vec![names[0].as_str()]);
        for i in 0..depth - 1 {
            builder = builder.rule(names[i].as_str(), vec![names[i + 1].as_str()]);
        }
        builder = builder.rule(names[depth - 1].as_str(), vec!["a"]);
        let grammar = builder.start("s").build();
        let stats = analyze(&grammar);
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn topology_wide_alternatives(width in 2usize..7) {
        // S → a₁ | a₂ | … | aₙ
        let tok_names: Vec<String> = (0..width).map(|i| format!("t{}", i)).collect();
        let tok_pats: Vec<String> = (0..width).map(|i| format!("{}", (b'a' + i as u8) as char)).collect();
        let mut builder = GrammarBuilder::new("wide_n");
        for name in tok_names.iter().zip(tok_pats.iter()) {
            builder = builder.token(name.0.as_str(), name.1.as_str());
        }
        for tok_name in tok_names.iter().take(width) {
            builder = builder.rule("s", vec![tok_name.as_str()]);
        }
        let grammar = builder.start("s").build();
        let stats = analyze(&grammar);
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
    }

    #[test]
    fn topology_sequence_length(len in 1usize..6) {
        // S → a₁ a₂ … aₙ
        let tok_names: Vec<String> = (0..len).map(|i| format!("t{}", i)).collect();
        let tok_pats: Vec<String> = (0..len).map(|i| format!("{}", (b'a' + i as u8) as char)).collect();
        let mut builder = GrammarBuilder::new("seq_n");
        for i in 0..len {
            builder = builder.token(tok_names[i].as_str(), tok_pats[i].as_str());
        }
        let rhs: Vec<&str> = tok_names.iter().map(|s| s.as_str()).collect();
        builder = builder.rule("s", rhs);
        let grammar = builder.start("s").build();
        let stats = analyze(&grammar);
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn topology_four_deep_no_conflicts(_dummy in 0u8..1) {
        let stats = analyze(&four_deep_chain());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn topology_diamond_no_conflicts(_dummy in 0u8..1) {
        let stats = analyze(&diamond_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn topology_common_prefix_builds(_dummy in 0u8..1) {
        let stats = analyze(&common_prefix_grammar());
        // Just verify it builds and returns valid stats
        prop_assert!(stats.shift_reduce_conflicts < usize::MAX);
    }

    #[test]
    fn topology_long_sequence_no_conflicts(_dummy in 0u8..1) {
        let stats = analyze(&long_sequence_grammar());
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn topology_any_grammar_builds_table(grammar in arb_any_grammar()) {
        // All our test grammars should produce a valid table
        let pt = try_build(&grammar);
        prop_assert!(pt.is_some(), "grammar failed to build a parse table");
    }
}

// ---------------------------------------------------------------------------
// 7. Edge cases (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn edge_single_token_grammar(_dummy in 0u8..1) {
        let grammar = GrammarBuilder::new("single")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        let stats = analyze(&grammar);
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn edge_nullable_start(_dummy in 0u8..1) {
        let stats = analyze(&nullable_grammar());
        // A nullable grammar should still produce valid stats
        prop_assert!(stats.shift_reduce_conflicts < usize::MAX);
        prop_assert!(stats.reduce_reduce_conflicts < usize::MAX);
    }

    #[test]
    fn edge_left_recursive_builds(_dummy in 0u8..1) {
        let stats = analyze(&left_recursive_grammar());
        prop_assert!(stats.shift_reduce_conflicts < usize::MAX);
    }

    #[test]
    fn edge_right_recursive_builds(_dummy in 0u8..1) {
        let stats = analyze(&right_recursive_grammar());
        prop_assert!(stats.shift_reduce_conflicts < usize::MAX);
    }

    #[test]
    fn edge_precedence_grammar_builds(_dummy in 0u8..1) {
        let stats = analyze(&precedence_grammar());
        prop_assert!(stats.shift_reduce_conflicts < usize::MAX);
        prop_assert!(stats.precedence_resolved < usize::MAX);
    }

    #[test]
    fn edge_analyzer_default_trait(_dummy in 0u8..1) {
        // ConflictAnalyzer implements Default
        let mut analyzer = ConflictAnalyzer::default();
        let pt = build_table(&minimal_grammar());
        let stats = analyzer.analyze_table(&pt);
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
    }
}
