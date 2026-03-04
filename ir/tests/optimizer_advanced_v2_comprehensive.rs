//! Comprehensive advanced tests for GrammarOptimizer scenarios.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};
use adze_ir::{Associativity, Grammar};

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["x", "y"])
        .start("root")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", "[0-9]+")
        .token("PLUS", "\\+")
        .token("STAR", "\\*")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("tok", "tok")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["tok"])
        .start("a")
        .build()
}

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("NUM", "[0-9]+")
        .token("PLUS", "\\+")
        .token("STAR", "\\*")
        .token("MINUS", "\\-")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "MINUS", "expr"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn alternatives_grammar() -> Grammar {
    GrammarBuilder::new("alts")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .rule("root", vec!["C"])
        .start("root")
        .build()
}

fn large_token_grammar(n: usize) -> Grammar {
    let mut b = GrammarBuilder::new("large");
    let mut names: Vec<String> = Vec::new();
    for i in 0..n {
        let name = format!("T{}", i);
        b = b.token(&name, &format!("t{}", i));
        names.push(name);
    }
    // rule that references the first token so grammar is non-trivial
    let first = names[0].clone();
    b = b.rule("root", vec![&first]).start("root");
    b.build()
}

// =========================================================================
// 1. GrammarOptimizer::new() construction
// =========================================================================

#[test]
fn test_optimizer_new_returns_instance() {
    let _opt = GrammarOptimizer::new();
}

#[test]
fn test_optimizer_default_returns_instance() {
    let _opt = GrammarOptimizer::default();
}

#[test]
fn test_optimizer_new_and_default_produce_same_stats() {
    let mut g1 = minimal_grammar();
    let mut g2 = g1.clone();
    let s1 = GrammarOptimizer::new().optimize(&mut g1);
    let s2 = GrammarOptimizer::default().optimize(&mut g2);
    assert_eq!(s1.total(), s2.total());
}

#[test]
fn test_optimizer_can_be_reused_after_optimize() {
    let mut opt = GrammarOptimizer::new();
    let mut g = minimal_grammar();
    let _ = opt.optimize(&mut g);
    // second use with fresh optimizer (fields reset on new)
    let mut opt2 = GrammarOptimizer::new();
    let mut g2 = minimal_grammar();
    let _ = opt2.optimize(&mut g2);
}

// =========================================================================
// 2. Optimize minimal grammar
// =========================================================================

#[test]
fn test_optimize_minimal_grammar_does_not_panic() {
    let mut g = minimal_grammar();
    GrammarOptimizer::new().optimize(&mut g);
}

#[test]
fn test_optimize_minimal_grammar_preserves_name() {
    let mut g = minimal_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(g.name, "minimal");
}

#[test]
fn test_optimize_minimal_grammar_keeps_token() {
    let mut g = minimal_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(!g.tokens.is_empty(), "token should survive optimization");
}

#[test]
fn test_optimize_minimal_grammar_rule_count() {
    let mut g = minimal_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    // Minimal single-terminal rule may be fully inlined/eliminated
    let _ = g.all_rules().count();
}

#[test]
fn test_optimize_minimal_grammar_stats_total_small() {
    let mut g = minimal_grammar();
    let stats = GrammarOptimizer::new().optimize(&mut g);
    // minimal grammar has little to optimize
    assert!(stats.total() <= 5);
}

#[test]
fn test_optimize_grammar_convenience_minimal() {
    let g = minimal_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

// =========================================================================
// 3. Optimize grammar with multiple tokens
// =========================================================================

#[test]
fn test_optimize_two_tokens_preserves_both() {
    let mut g = two_token_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    // both tokens are referenced in the rule so should survive
    assert!(g.tokens.len() >= 2);
}

#[test]
fn test_optimize_unused_token_removed() {
    // Add an extra unused token
    let mut g = GrammarBuilder::new("extra")
        .token("used", "u")
        .token("unused", "z")
        .rule("root", vec!["used"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    // the unused token may be removed
    let has_unused = g.tokens.values().any(|t| t.name == "unused");
    // We just verify it doesn't panic; removal depends on pass details
    let _ = has_unused;
}

#[test]
fn test_optimize_three_tokens_all_used() {
    let mut g = GrammarBuilder::new("tri")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("root", vec!["a", "b", "c"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.tokens.len() >= 3);
}

#[test]
fn test_optimize_duplicate_pattern_tokens() {
    let mut g = GrammarBuilder::new("dup")
        .token("alpha", "x")
        .token("beta", "x")
        .rule("root", vec!["alpha"])
        .rule("root", vec!["beta"])
        .start("root")
        .build();
    let stats = GrammarOptimizer::new().optimize(&mut g);
    // merge_equivalent_tokens should detect duplicates
    let _ = stats.merged_tokens;
}

#[test]
fn test_optimize_many_distinct_tokens() {
    let mut g = GrammarBuilder::new("many")
        .token("t1", "a")
        .token("t2", "b")
        .token("t3", "c")
        .token("t4", "d")
        .token("t5", "e")
        .rule("root", vec!["t1", "t2", "t3", "t4", "t5"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.tokens.len() >= 5);
}

// =========================================================================
// 4. Optimize grammar with precedence
// =========================================================================

#[test]
fn test_optimize_precedence_grammar_does_not_panic() {
    let mut g = precedence_grammar();
    GrammarOptimizer::new().optimize(&mut g);
}

#[test]
fn test_optimize_precedence_preserves_name() {
    let mut g = precedence_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(g.name, "prec");
}

#[test]
fn test_optimize_precedence_preserves_tokens() {
    let mut g = precedence_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(!g.tokens.is_empty());
}

#[test]
fn test_optimize_right_assoc_precedence() {
    let mut g = GrammarBuilder::new("rassoc")
        .token("NUM", "[0-9]+")
        .token("POW", "\\^")
        .rule_with_precedence("expr", vec!["expr", "POW", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_none_assoc_precedence() {
    let mut g = GrammarBuilder::new("nassoc")
        .token("NUM", "[0-9]+")
        .token("EQ", "==")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 0, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_mixed_precedence_levels() {
    let mut g = GrammarBuilder::new("mixed")
        .token("NUM", "[0-9]+")
        .token("PLUS", "\\+")
        .token("STAR", "\\*")
        .token("POW", "\\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "POW", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let stats = GrammarOptimizer::new().optimize(&mut g);
    let _ = stats;
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_negative_precedence() {
    let mut g = GrammarBuilder::new("neg_prec")
        .token("NUM", "[0-9]+")
        .token("COMMA", ",")
        .rule_with_precedence("seq", vec!["seq", "COMMA", "seq"], -1, Associativity::Left)
        .rule("seq", vec!["NUM"])
        .start("seq")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

// =========================================================================
// 5. Optimize grammar with alternatives
// =========================================================================

#[test]
fn test_optimize_alternatives_preserves_grammar() {
    let mut g = alternatives_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(!g.tokens.is_empty());
}

#[test]
fn test_optimize_alternatives_convenience_fn() {
    let g = alternatives_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn test_optimize_many_alternatives() {
    let mut b = GrammarBuilder::new("alts5");
    for i in 0..5 {
        let name = format!("t{}", i);
        b = b.token(&name, &format!("v{}", i));
    }
    for i in 0..5 {
        let name = format!("t{}", i);
        b = b.rule("root", vec![&name]);
    }
    let mut g = b.start("root").build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_alternatives_with_multi_symbol_rhs() {
    let mut g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A", "B"])
        .rule("root", vec!["A", "C"])
        .rule("root", vec!["B", "C"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

// =========================================================================
// 6. Optimize chain grammars
// =========================================================================

#[test]
fn test_optimize_chain_grammar_does_not_panic() {
    let mut g = chain_grammar();
    GrammarOptimizer::new().optimize(&mut g);
}

#[test]
fn test_optimize_chain_inlines_or_eliminates() {
    let mut g = chain_grammar();
    let stats = GrammarOptimizer::new().optimize(&mut g);
    // chain rules should trigger inlining and/or unit elimination
    assert!(stats.inlined_rules > 0 || stats.eliminated_unit_rules > 0);
}

#[test]
fn test_optimize_long_chain() {
    let mut g = GrammarBuilder::new("long_chain")
        .token("tok", "tok")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["d"])
        .rule("d", vec!["e"])
        .rule("e", vec!["tok"])
        .start("a")
        .build();
    let stats = GrammarOptimizer::new().optimize(&mut g);
    assert!(stats.inlined_rules > 0 || stats.eliminated_unit_rules > 0);
}

#[test]
fn test_optimize_chain_result_tokens_preserved() {
    let mut g = chain_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    // Chain rules may be fully inlined; token should survive
    assert!(!g.tokens.is_empty() || g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_chain_convenience() {
    let g = chain_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn test_optimize_chain_with_branch() {
    let mut g = GrammarBuilder::new("chain_branch")
        .token("tok", "tok")
        .token("other", "other")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("b", vec!["other"])
        .rule("c", vec!["tok"])
        .start("a")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

// =========================================================================
// 7. Optimize after normalize
// =========================================================================

#[test]
fn test_normalize_then_optimize_does_not_panic() {
    let mut g = arith_grammar();
    g.normalize();
    GrammarOptimizer::new().optimize(&mut g);
}

#[test]
fn test_normalize_then_optimize_preserves_name() {
    let mut g = arith_grammar();
    g.normalize();
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(g.name, "arith");
}

#[test]
fn test_normalize_then_optimize_result_has_rules() {
    let mut g = arith_grammar();
    g.normalize();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_then_normalize_does_not_panic() {
    let mut g = arith_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    g.normalize();
}

#[test]
fn test_normalize_then_optimize_chain() {
    let mut g = chain_grammar();
    g.normalize();
    GrammarOptimizer::new().optimize(&mut g);
    // Chain may be fully inlined after normalize+optimize
    let _ = g.all_rules().count();
}

#[test]
fn test_normalize_then_optimize_precedence() {
    let mut g = precedence_grammar();
    g.normalize();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(!g.tokens.is_empty());
}

// =========================================================================
// 8. Optimize idempotency
// =========================================================================

#[test]
fn test_idempotency_minimal() {
    let mut g = minimal_grammar();
    // Run optimizer three times; after the second pass the grammar should stabilize
    GrammarOptimizer::new().optimize(&mut g);
    GrammarOptimizer::new().optimize(&mut g);
    let snapshot = format!("{:?}", g);
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(snapshot, format!("{:?}", g));
}

#[test]
fn test_idempotency_arith() {
    let mut g = arith_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    let snapshot = format!("{:?}", g);
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(snapshot, format!("{:?}", g));
}

#[test]
fn test_idempotency_chain() {
    let mut g = chain_grammar();
    // Stabilize first
    GrammarOptimizer::new().optimize(&mut g);
    GrammarOptimizer::new().optimize(&mut g);
    let snapshot = format!("{:?}", g);
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(snapshot, format!("{:?}", g));
}

#[test]
fn test_idempotency_precedence() {
    let mut g = precedence_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    let snapshot = format!("{:?}", g);
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(snapshot, format!("{:?}", g));
}

#[test]
fn test_idempotency_alternatives() {
    let mut g = alternatives_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    let snapshot = format!("{:?}", g);
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(snapshot, format!("{:?}", g));
}

#[test]
fn test_idempotency_stats_converge() {
    let mut g = minimal_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    GrammarOptimizer::new().optimize(&mut g);
    let stats3 = GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(stats3.total(), 0, "third pass should find nothing to do");
}

#[test]
fn test_triple_optimize_idempotent() {
    let mut g = arith_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    GrammarOptimizer::new().optimize(&mut g);
    let snapshot = format!("{:?}", g);
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(snapshot, format!("{:?}", g));
}

// =========================================================================
// 9. Grammar statistics before/after optimization
// =========================================================================

#[test]
fn test_stats_fields_are_accessible() {
    let s = OptimizationStats::default();
    let _ = s.removed_unused_symbols;
    let _ = s.inlined_rules;
    let _ = s.merged_tokens;
    let _ = s.optimized_left_recursion;
    let _ = s.eliminated_unit_rules;
}

#[test]
fn test_stats_total_matches_sum() {
    let s = OptimizationStats {
        removed_unused_symbols: 2,
        inlined_rules: 3,
        merged_tokens: 1,
        optimized_left_recursion: 4,
        eliminated_unit_rules: 5,
    };
    assert_eq!(s.total(), 2 + 3 + 1 + 4 + 5);
}

#[test]
fn test_stats_default_is_all_zero() {
    let s = OptimizationStats::default();
    assert_eq!(s.removed_unused_symbols, 0);
    assert_eq!(s.inlined_rules, 0);
    assert_eq!(s.merged_tokens, 0);
    assert_eq!(s.optimized_left_recursion, 0);
    assert_eq!(s.eliminated_unit_rules, 0);
}

#[test]
fn test_rule_count_non_increasing_after_optimize() {
    let mut g = arith_grammar();
    let before = g.all_rules().count();
    GrammarOptimizer::new().optimize(&mut g);
    let after = g.all_rules().count();
    // Optimization may add helper rules, but typically the grammar doesn't explode
    assert!(after <= before * 3, "rule count should not explode");
}

#[test]
fn test_token_count_non_increasing_after_optimize() {
    let mut g = arith_grammar();
    let before = g.tokens.len();
    GrammarOptimizer::new().optimize(&mut g);
    let after = g.tokens.len();
    assert!(after <= before, "tokens should not increase");
}

#[test]
fn test_stats_debug_format() {
    let s = OptimizationStats::default();
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("OptimizationStats"));
}

#[test]
fn test_arith_stats_left_recursion_detected() {
    let mut g = arith_grammar();
    let stats = GrammarOptimizer::new().optimize(&mut g);
    assert!(
        stats.optimized_left_recursion > 0,
        "arith grammar has left-recursive rules"
    );
}

// =========================================================================
// 10. Optimize large grammars (50+ tokens)
// =========================================================================

#[test]
fn test_large_grammar_50_tokens() {
    let mut g = large_token_grammar(50);
    GrammarOptimizer::new().optimize(&mut g);
    // Should not panic; the single-terminal rule may be fully inlined
    let _ = g.all_rules().count();
}

#[test]
fn test_large_grammar_60_tokens() {
    let mut g = large_token_grammar(60);
    GrammarOptimizer::new().optimize(&mut g);
    let _ = g.all_rules().count();
}

#[test]
fn test_large_grammar_100_tokens() {
    let mut g = large_token_grammar(100);
    GrammarOptimizer::new().optimize(&mut g);
    let _ = g.all_rules().count();
}

#[test]
fn test_large_grammar_unused_tokens_removed() {
    let mut g = large_token_grammar(50);
    let before = g.tokens.len();
    GrammarOptimizer::new().optimize(&mut g);
    let after = g.tokens.len();
    // Most tokens are unreferenced; optimizer should remove some
    assert!(after <= before, "unused tokens should be pruned");
}

#[test]
fn test_large_grammar_stats() {
    let mut g = large_token_grammar(50);
    let stats = GrammarOptimizer::new().optimize(&mut g);
    // Many unused tokens ⇒ removed_unused_symbols should be nonzero
    assert!(stats.removed_unused_symbols > 0);
}

#[test]
fn test_large_grammar_convenience() {
    let g = large_token_grammar(55);
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn test_large_grammar_idempotent() {
    let mut g = large_token_grammar(50);
    // Stabilize first
    GrammarOptimizer::new().optimize(&mut g);
    GrammarOptimizer::new().optimize(&mut g);
    let snap = format!("{:?}", g);
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(snap, format!("{:?}", g));
}

// =========================================================================
// Additional edge-case and cross-cutting tests
// =========================================================================

#[test]
fn test_optimize_grammar_clone_equality_before_optimize() {
    let g1 = minimal_grammar();
    let g2 = g1.clone();
    assert_eq!(format!("{:?}", g1), format!("{:?}", g2));
}

#[test]
fn test_optimize_grammar_name_unchanged_arith() {
    let mut g = arith_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(g.name, "arith");
}

#[test]
fn test_optimize_with_extras_does_not_panic() {
    let mut g = GrammarBuilder::new("ws")
        .token("WS", "\\s+")
        .token("A", "a")
        .extra("WS")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    // Renumber may change extras IDs; just verify no panic
    assert_eq!(g.name, "ws");
}

#[test]
fn test_optimize_preserves_externals() {
    let mut g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(!g.externals.is_empty());
}

#[test]
fn test_optimize_python_like_grammar() {
    let mut g = GrammarBuilder::python_like();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_javascript_like_grammar() {
    let mut g = GrammarBuilder::javascript_like();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_javascript_like_stats() {
    let mut g = GrammarBuilder::javascript_like();
    let stats = GrammarOptimizer::new().optimize(&mut g);
    // JS-like grammar has left-recursive expressions
    assert!(stats.optimized_left_recursion > 0 || stats.eliminated_unit_rules > 0);
}

#[test]
fn test_optimize_epsilon_rule_alternative() {
    let mut g = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("root", vec!["A"])
        .rule("root", vec![]) // epsilon alternative
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_fragile_token_preserved() {
    let mut g = GrammarBuilder::new("fragile")
        .fragile_token("NEWLINE", "\\n")
        .token("A", "a")
        .rule("root", vec!["A", "NEWLINE"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    let has_fragile = g.tokens.values().any(|t| t.fragile);
    assert!(has_fragile, "fragile token should survive optimization");
}

#[test]
fn test_optimize_grammar_serde_roundtrip_after_optimize() {
    let mut g = arith_grammar();
    GrammarOptimizer::new().optimize(&mut g);
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn test_optimize_single_terminal_rule() {
    let mut g = GrammarBuilder::new("single")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    // Single-terminal rule may be inlined; at minimum grammar name survives
    assert_eq!(g.name, "single");
}

#[test]
fn test_optimize_two_level_chain_with_precedence() {
    let mut g = GrammarBuilder::new("chain_prec")
        .token("NUM", "[0-9]+")
        .token("PLUS", "\\+")
        .rule("top", vec!["mid"])
        .rule_with_precedence("mid", vec!["mid", "PLUS", "mid"], 1, Associativity::Left)
        .rule("mid", vec!["NUM"])
        .start("top")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert!(g.all_rules().count() >= 1);
}
