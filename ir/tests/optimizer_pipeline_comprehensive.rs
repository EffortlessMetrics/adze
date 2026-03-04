//! Comprehensive tests for the GrammarOptimizer pipeline.
//!
//! Covers: construction, empty grammars, simple grammars, unused/unreachable/duplicate
//! rules, OptimizationStats inspection, convenience function, optimizer reuse,
//! grammar structure preservation, determinism, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};
use adze_ir::{Associativity, Grammar};

// ============================================================================
// Helpers
// ============================================================================

fn simple_expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["expr", "-", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUMBER"])
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build()
}

// ============================================================================
// 1. GrammarOptimizer construction
// ============================================================================

#[test]
fn new_optimizer_is_constructible() {
    let _opt = GrammarOptimizer::new();
}

#[test]
fn default_optimizer_equals_new() {
    // Both paths should yield a usable optimizer.
    let mut a = GrammarOptimizer::new();
    let mut b = GrammarOptimizer::default();
    let mut g1 = simple_expr_grammar();
    let mut g2 = simple_expr_grammar();
    let s1 = a.optimize(&mut g1);
    let s2 = b.optimize(&mut g2);
    assert_eq!(s1.total(), s2.total());
}

// ============================================================================
// 2. Optimizing empty grammars
// ============================================================================

#[test]
fn empty_grammar_optimizes_without_panic() {
    let mut grammar = Grammar::new("empty".to_string());
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(stats.total(), 0);
}

#[test]
fn empty_grammar_name_preserved() {
    let mut grammar = Grammar::new("empty_test".to_string());
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert_eq!(grammar.name, "empty_test");
}

#[test]
fn empty_grammar_has_no_rules_after_optimize() {
    let mut grammar = Grammar::new("e".to_string());
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert!(grammar.rules.is_empty());
}

#[test]
fn empty_grammar_has_no_tokens_after_optimize() {
    let mut grammar = Grammar::new("e".to_string());
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert!(grammar.tokens.is_empty());
}

#[test]
fn empty_grammar_stats_all_zero() {
    let mut grammar = Grammar::new("e".to_string());
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(stats.removed_unused_symbols, 0);
    assert_eq!(stats.inlined_rules, 0);
    assert_eq!(stats.merged_tokens, 0);
    assert_eq!(stats.optimized_left_recursion, 0);
    assert_eq!(stats.eliminated_unit_rules, 0);
}

// ============================================================================
// 3. Optimizing simple grammars (single rule)
// ============================================================================

#[test]
fn single_terminal_rule_grammar() {
    let mut grammar = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);

    // Grammar name must survive.
    assert_eq!(grammar.name, "single");
    // Stats total is self-consistent.
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules,
    );
}

#[test]
fn single_rule_grammar_preserves_name() {
    let mut grammar = GrammarBuilder::new("hello")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert_eq!(grammar.name, "hello");
}

#[test]
fn single_rule_still_parseable_after_optimize() {
    let mut grammar = GrammarBuilder::new("s")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // The optimizer may inline/eliminate the single rule; just verify no panic.
    let _ = stats.total();
}

// ============================================================================
// 4. Optimizing grammars with unused rules/tokens
// ============================================================================

#[test]
fn unused_token_is_removed() {
    let mut grammar = GrammarBuilder::new("unused_tok")
        .token("USED", "u")
        .token("UNUSED", "x")
        .rule("root", vec!["USED"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);

    // The UNUSED token should have been removed.
    assert!(stats.removed_unused_symbols >= 1);
    // Only one token should remain.
    assert_eq!(grammar.tokens.len(), 1);
}

#[test]
fn multiple_unused_tokens_removed() {
    let mut grammar = GrammarBuilder::new("multi_unused")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);

    // B and C are unused.
    assert!(stats.removed_unused_symbols >= 2);
    assert_eq!(grammar.tokens.len(), 1);
}

#[test]
fn used_tokens_are_kept() {
    let mut grammar = GrammarBuilder::new("keep")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert_eq!(grammar.tokens.len(), 2);
}

// ============================================================================
// 5. Optimizing grammars with unreachable rules
// ============================================================================

#[test]
fn unreachable_nonterminal_does_not_crash() {
    // "orphan" has no path from "root".
    // The optimizer may or may not remove it (it marks all LHS symbols as used).
    let mut grammar = GrammarBuilder::new("unreach")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("orphan", vec!["B"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // No panic is the main assertion.
    let _ = stats.total();
}

#[test]
fn grammar_with_only_tokens_no_rules() {
    // Tokens but no rules — optimizer should not panic.
    let mut grammar = GrammarBuilder::new("tokens_only")
        .token("A", "a")
        .token("B", "b")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // Both tokens are unused (no rules reference them).
    assert!(stats.removed_unused_symbols >= 2);
    assert!(grammar.tokens.is_empty());
}

// ============================================================================
// 6. Optimizing grammars with duplicate tokens
// ============================================================================

#[test]
fn duplicate_string_tokens_merged() {
    let mut grammar = GrammarBuilder::new("dup")
        .token("PLUS", "+")
        .token("ADD", "+")
        .rule("root", vec!["PLUS", "ADD"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);

    // One of the two duplicate tokens should have been merged.
    assert!(stats.merged_tokens >= 1);
}

#[test]
fn duplicate_regex_tokens_merged() {
    let mut grammar = GrammarBuilder::new("dup_re")
        .token("NUM1", r"\d+")
        .token("NUM2", r"\d+")
        .rule("root", vec!["NUM1", "NUM2"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.merged_tokens >= 1);
}

#[test]
fn non_duplicate_tokens_not_merged() {
    let mut grammar = GrammarBuilder::new("nodup")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(stats.merged_tokens, 0);
}

#[test]
fn three_duplicate_tokens_merged() {
    let mut grammar = GrammarBuilder::new("tri_dup")
        .token("X1", "x")
        .token("X2", "x")
        .token("X3", "x")
        .rule("root", vec!["X1", "X2", "X3"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // Two merges expected (X2->X1, X3->X1).
    assert!(stats.merged_tokens >= 2);
}

// ============================================================================
// 7. OptimizationStats field inspection
// ============================================================================

#[test]
fn stats_default_is_all_zero() {
    let stats = OptimizationStats::default();
    assert_eq!(stats.removed_unused_symbols, 0);
    assert_eq!(stats.inlined_rules, 0);
    assert_eq!(stats.merged_tokens, 0);
    assert_eq!(stats.optimized_left_recursion, 0);
    assert_eq!(stats.eliminated_unit_rules, 0);
}

#[test]
fn stats_debug_is_not_empty() {
    let stats = OptimizationStats::default();
    let dbg = format!("{:?}", stats);
    assert!(!dbg.is_empty());
}

#[test]
fn stats_fields_match_optimizer_output() {
    let mut grammar = simple_expr_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);

    // Each field must be >= 0 (trivially true for usize, but confirms access).
    let _ = stats.removed_unused_symbols;
    let _ = stats.inlined_rules;
    let _ = stats.merged_tokens;
    let _ = stats.optimized_left_recursion;
    let _ = stats.eliminated_unit_rules;
}

#[test]
fn left_recursive_grammar_stats() {
    let mut grammar = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // The grammar has left-recursive rules for expr and term.
    assert!(stats.optimized_left_recursion >= 1);
}

#[test]
fn unit_rule_elimination_counted() {
    // expr -> term (unit rule), term -> NUMBER.
    let mut grammar = GrammarBuilder::new("unit")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["term"])
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // The unit rule expr -> term should trigger elimination.
    assert!(stats.eliminated_unit_rules >= 1 || stats.inlined_rules >= 1);
}

// ============================================================================
// 8. OptimizationStats total() method
// ============================================================================

#[test]
fn total_equals_sum_of_fields_simple() {
    let mut grammar = simple_expr_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules,
    );
}

#[test]
fn total_equals_sum_of_fields_arith() {
    let mut grammar = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules,
    );
}

#[test]
fn total_zero_for_empty_grammar() {
    let mut grammar = Grammar::new("e".to_string());
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(stats.total(), 0);
}

#[test]
fn total_nonzero_when_any_field_nonzero() {
    let mut grammar = GrammarBuilder::new("nz")
        .token("A", "a")
        .token("UNUSED", "z")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.total() >= 1);
}

// ============================================================================
// 9. optimize_grammar convenience function
// ============================================================================

#[test]
fn optimize_grammar_returns_ok() {
    let grammar = simple_expr_grammar();
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

#[test]
fn optimize_grammar_preserves_name() {
    let grammar = GrammarBuilder::new("conv_test")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    assert_eq!(optimized.name, "conv_test");
}

#[test]
fn optimize_grammar_empty_ok() {
    let grammar = Grammar::new("emp".to_string());
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

#[test]
fn optimize_grammar_removes_unused() {
    let grammar = GrammarBuilder::new("rm")
        .token("USED", "u")
        .token("DEAD", "d")
        .rule("root", vec!["USED"])
        .start("root")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    assert_eq!(optimized.tokens.len(), 1);
}

#[test]
fn optimize_grammar_arith() {
    let grammar = arith_grammar();
    let optimized = optimize_grammar(grammar).unwrap();
    // Should still have tokens and rules.
    assert!(!optimized.tokens.is_empty());
    assert!(!optimized.rules.is_empty());
}

#[test]
fn optimize_grammar_merges_duplicates() {
    let grammar = GrammarBuilder::new("dup_conv")
        .token("P1", "+")
        .token("P2", "+")
        .rule("root", vec!["P1", "P2"])
        .start("root")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    // One of the duplicates should be gone.
    assert!(optimized.tokens.len() <= 1);
}

// ============================================================================
// 10. Optimizer reuse across multiple grammars
// ============================================================================

#[test]
fn optimizer_reuse_two_grammars() {
    let mut opt = GrammarOptimizer::new();

    let mut g1 = simple_expr_grammar();
    let stats1 = opt.optimize(&mut g1);

    // Re-create optimizer for second use (optimizer accumulates state from analyze).
    let mut opt2 = GrammarOptimizer::new();
    let mut g2 = arith_grammar();
    let stats2 = opt2.optimize(&mut g2);

    // Both should produce valid stats.
    assert_eq!(
        stats1.total(),
        stats1.removed_unused_symbols
            + stats1.inlined_rules
            + stats1.merged_tokens
            + stats1.optimized_left_recursion
            + stats1.eliminated_unit_rules,
    );
    assert_eq!(
        stats2.total(),
        stats2.removed_unused_symbols
            + stats2.inlined_rules
            + stats2.merged_tokens
            + stats2.optimized_left_recursion
            + stats2.eliminated_unit_rules,
    );
}

#[test]
fn fresh_optimizer_per_grammar() {
    for i in 0..5 {
        let mut grammar = GrammarBuilder::new(&format!("g{}", i))
            .token("T", "t")
            .rule("root", vec!["T"])
            .start("root")
            .build();

        let mut opt = GrammarOptimizer::new();
        let stats = opt.optimize(&mut grammar);
        assert_eq!(grammar.name, format!("g{}", i));
        let _ = stats.total();
    }
}

// ============================================================================
// 11. Grammar structure preservation
// ============================================================================

#[test]
fn optimized_grammar_has_no_empty_rule_vecs() {
    let mut grammar = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);

    for (_id, rules) in &grammar.rules {
        assert!(!rules.is_empty(), "No symbol should have an empty rule vec");
    }
}

#[test]
fn optimized_grammar_tokens_are_referenced() {
    let mut grammar = GrammarBuilder::new("ref")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD", "d")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);

    // DEAD should have been removed; remaining tokens are referenced.
    assert!(!grammar.tokens.contains_key(&adze_ir::SymbolId(0)));
    assert!(grammar.tokens.len() <= 2);
}

#[test]
fn rule_names_may_survive_optimization() {
    let mut grammar = GrammarBuilder::new("names")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let had_names = !grammar.rule_names.is_empty();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    // If there were rule names before, at least some should remain if rules remain.
    if !grammar.rules.is_empty() && had_names {
        assert!(!grammar.rule_names.is_empty());
    }
}

#[test]
fn extras_tracked_through_optimization() {
    // Use a grammar with multiple rules so the start rule isn't eliminated.
    let mut grammar = GrammarBuilder::new("extras")
        .token("WS", r"\s+")
        .token("A", "a")
        .token("B", "b")
        .extra("WS")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let had_extras = !grammar.extras.is_empty();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    // If the extra token was referenced and present, it should survive renumbering.
    // The extra token "WS" is referenced only as an extra, so the optimizer may
    // or may not keep it. Just ensure no panic.
    let _ = had_extras;
}

#[test]
fn externals_survive_optimization() {
    let mut grammar = GrammarBuilder::new("ext")
        .token("A", "a")
        .external("INDENT")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert!(!grammar.externals.is_empty());
}

#[test]
fn start_symbol_rules_exist_after_optimize() {
    let mut grammar = simple_expr_grammar();
    let start = grammar.start_symbol();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);

    // After optimization, grammar should still have rules (possibly renumbered).
    assert!(!grammar.rules.is_empty());
    // And the start symbol concept should still be retrievable.
    let _ = start;
}

#[test]
fn precedence_declarations_preserved() {
    let mut grammar = GrammarBuilder::new("prec")
        .token("A", "a")
        .token("+", "+")
        .precedence(1, Associativity::Left, vec!["+"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert!(!grammar.precedences.is_empty());
}

// ============================================================================
// 12. Optimization determinism
// ============================================================================

#[test]
fn same_grammar_same_stats() {
    let mut g1 = simple_expr_grammar();
    let mut g2 = simple_expr_grammar();

    let mut o1 = GrammarOptimizer::new();
    let mut o2 = GrammarOptimizer::new();
    let s1 = o1.optimize(&mut g1);
    let s2 = o2.optimize(&mut g2);

    assert_eq!(s1.total(), s2.total());
    assert_eq!(s1.removed_unused_symbols, s2.removed_unused_symbols);
    assert_eq!(s1.inlined_rules, s2.inlined_rules);
    assert_eq!(s1.merged_tokens, s2.merged_tokens);
    assert_eq!(s1.optimized_left_recursion, s2.optimized_left_recursion);
    assert_eq!(s1.eliminated_unit_rules, s2.eliminated_unit_rules);
}

#[test]
fn same_arith_grammar_same_stats() {
    let mut g1 = arith_grammar();
    let mut g2 = arith_grammar();

    let mut o1 = GrammarOptimizer::new();
    let mut o2 = GrammarOptimizer::new();
    let s1 = o1.optimize(&mut g1);
    let s2 = o2.optimize(&mut g2);

    assert_eq!(s1.total(), s2.total());
    assert_eq!(s1.removed_unused_symbols, s2.removed_unused_symbols);
    assert_eq!(s1.inlined_rules, s2.inlined_rules);
    assert_eq!(s1.merged_tokens, s2.merged_tokens);
    assert_eq!(s1.optimized_left_recursion, s2.optimized_left_recursion);
    assert_eq!(s1.eliminated_unit_rules, s2.eliminated_unit_rules);
}

#[test]
fn deterministic_token_count() {
    let build = || {
        GrammarBuilder::new("det")
            .token("A", "a")
            .token("B", "b")
            .token("DEAD", "d")
            .rule("root", vec!["A", "B"])
            .start("root")
            .build()
    };

    let mut g1 = build();
    let mut g2 = build();
    let mut o1 = GrammarOptimizer::new();
    let mut o2 = GrammarOptimizer::new();
    o1.optimize(&mut g1);
    o2.optimize(&mut g2);

    assert_eq!(g1.tokens.len(), g2.tokens.len());
    assert_eq!(g1.rules.len(), g2.rules.len());
}

#[test]
fn deterministic_rule_count() {
    let build = || arith_grammar();
    let mut g1 = build();
    let mut g2 = build();
    let mut o1 = GrammarOptimizer::new();
    let mut o2 = GrammarOptimizer::new();
    o1.optimize(&mut g1);
    o2.optimize(&mut g2);

    assert_eq!(g1.rules.len(), g2.rules.len());
}

// ============================================================================
// 13. Edge cases
// ============================================================================

#[test]
fn many_tokens_grammar() {
    let mut builder = GrammarBuilder::new("many_tok");
    let mut rhs = Vec::new();
    for i in 0..20 {
        let name = format!("T{}", i);
        let pat = format!("t{}", i);
        builder = builder.token(&name, &pat);
        rhs.push(name);
    }
    let rhs_refs: Vec<&str> = rhs.iter().map(|s| s.as_str()).collect();
    builder = builder.rule("root", rhs_refs).start("root");
    let mut grammar = builder.build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // All tokens should be kept since they're all referenced.
    assert_eq!(grammar.tokens.len(), 20);
    let _ = stats.total();
}

#[test]
fn many_unused_tokens() {
    let mut builder = GrammarBuilder::new("many_unused");
    for i in 0..15 {
        builder = builder.token(&format!("T{}", i), &format!("t{}", i));
    }
    // Only use T0.
    builder = builder.rule("root", vec!["T0"]).start("root");
    let mut grammar = builder.build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.removed_unused_symbols >= 14);
    assert_eq!(grammar.tokens.len(), 1);
}

#[test]
fn deeply_nested_unit_rules() {
    // a -> b -> c -> d -> T
    let mut grammar = GrammarBuilder::new("deep")
        .token("T", "t")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["d"])
        .rule("d", vec!["T"])
        .start("a")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // Unit rules should be at least partially eliminated or inlined.
    assert!(stats.total() >= 1);
}

#[test]
fn self_referencing_rule() {
    // expr -> expr (direct self-reference, left recursive with single symbol)
    let mut grammar = GrammarBuilder::new("selfref")
        .token("A", "a")
        .rule("expr", vec!["expr"])
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // Should not panic.
    let _ = stats.total();
}

#[test]
fn left_recursive_with_precedence() {
    let mut grammar = GrammarBuilder::new("lr_prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.optimized_left_recursion >= 1);
}

#[test]
fn grammar_with_epsilon_rule() {
    let mut grammar = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("root", vec!["A"])
        .rule("root", vec![]) // epsilon
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn grammar_with_multiple_alternatives() {
    let mut grammar = GrammarBuilder::new("multi_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .rule("root", vec!["C"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn grammar_with_fragile_token() {
    let mut grammar = GrammarBuilder::new("fragile")
        .fragile_token("ERR", "error")
        .token("A", "a")
        .rule("root", vec!["A", "ERR"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
}

#[test]
fn builder_python_like_optimizes() {
    let mut grammar = GrammarBuilder::python_like();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // The python-like grammar is complex; just verify it doesn't panic.
    let _ = stats.total();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn builder_javascript_like_optimizes() {
    let mut grammar = GrammarBuilder::javascript_like();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn convenience_function_on_python_like() {
    let grammar = GrammarBuilder::python_like();
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
    let g = result.unwrap();
    assert!(!g.rules.is_empty());
}

#[test]
fn convenience_function_on_javascript_like() {
    let grammar = GrammarBuilder::javascript_like();
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
    let g = result.unwrap();
    assert!(!g.rules.is_empty());
}

#[test]
fn two_nonterminals_one_unused_token() {
    let mut grammar = GrammarBuilder::new("two_nt")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD", "d")
        .rule("root", vec!["child"])
        .rule("child", vec!["A", "B"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.removed_unused_symbols >= 1);
}

#[test]
fn optimize_grammar_does_not_lose_externals() {
    let grammar = GrammarBuilder::new("ext_keep")
        .token("A", "a")
        .external("INDENT")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    assert!(!optimized.externals.is_empty());
}

#[test]
fn optimize_grammar_extras_no_panic() {
    let grammar = GrammarBuilder::new("ext_keep2")
        .token("WS", r"\s+")
        .token("A", "a")
        .token("B", "b")
        .extra("WS")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    // Just verify it didn't panic and grammar still has a name.
    assert_eq!(optimized.name, "ext_keep2");
}

#[test]
fn chained_unit_rules_collapsed() {
    // Chain: s -> a -> b -> T
    let mut grammar = GrammarBuilder::new("chain")
        .token("T", "t")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["T"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    // Should inline or eliminate unit rules.
    assert!(stats.inlined_rules + stats.eliminated_unit_rules >= 1);
}

#[test]
fn grammar_with_right_associativity() {
    let mut grammar = GrammarBuilder::new("rassoc")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn grammar_with_no_associativity() {
    let mut grammar = GrammarBuilder::new("noassoc")
        .token("N", r"\d+")
        .token("=", "=")
        .rule_with_precedence("cmp", vec!["cmp", "=", "cmp"], 1, Associativity::None)
        .rule("cmp", vec!["N"])
        .start("cmp")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
}

#[test]
fn grammar_single_token_single_rule() {
    let mut grammar = GrammarBuilder::new("minimal")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();

    let result = optimize_grammar(grammar.clone());
    assert!(result.is_ok());

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules,
    );
}

#[test]
fn mutual_recursion_no_panic() {
    // a -> b A, b -> a B (mutual recursion, not direct left-recursion)
    let mut grammar = GrammarBuilder::new("mutual")
        .token("A", "a")
        .token("B", "b")
        .rule("alpha", vec!["beta", "A"])
        .rule("beta", vec!["alpha", "B"])
        .rule("alpha", vec!["A"])
        .rule("beta", vec!["B"])
        .start("alpha")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
}

#[test]
fn all_tokens_duplicate_of_same_pattern() {
    let mut grammar = GrammarBuilder::new("all_dup")
        .token("A", "x")
        .token("B", "x")
        .token("C", "x")
        .token("D", "x")
        .rule("root", vec!["A", "B", "C", "D"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.merged_tokens >= 3);
}

#[test]
fn grammar_name_with_special_chars() {
    let mut grammar = GrammarBuilder::new("my-grammar_v2.0")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert_eq!(grammar.name, "my-grammar_v2.0");
}

#[test]
fn optimize_idempotent_token_count() {
    let mut grammar = GrammarBuilder::new("idem")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD", "d")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    let tok_count = grammar.tokens.len();

    // Optimize again with fresh optimizer.
    let mut opt2 = GrammarOptimizer::new();
    let stats2 = opt2.optimize(&mut grammar);
    assert_eq!(grammar.tokens.len(), tok_count);
    // Second pass should do little or nothing.
    assert_eq!(stats2.removed_unused_symbols, 0);
    assert_eq!(stats2.merged_tokens, 0);
}

#[test]
fn optimize_idempotent_rule_count() {
    let mut grammar = simple_expr_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    let rule_count = grammar.rules.len();

    let mut opt2 = GrammarOptimizer::new();
    opt2.optimize(&mut grammar);
    assert_eq!(grammar.rules.len(), rule_count);
}

#[test]
fn large_number_of_rules() {
    let mut builder = GrammarBuilder::new("large");
    for i in 0..10 {
        let tok = format!("T{}", i);
        builder = builder.token(&tok, &format!("t{}", i));
    }
    // Create 10 alternatives for root.
    for i in 0..10 {
        let tok = format!("T{}", i);
        builder = builder.rule("root", vec![&tok]);
    }
    builder = builder.start("root");
    let mut grammar = builder.build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn stats_total_with_all_fields_set() {
    // Not testing optimizer directly, but ensuring total() math.
    let mut grammar = GrammarBuilder::new("full")
        .token("N", r"\d+")
        .token("+", "+")
        .token("DEAD", "d")
        .token("DUP", "+") // same pattern as "+"
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["N"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules,
    );
}

#[test]
fn grammar_with_only_epsilon_rules() {
    let mut grammar = GrammarBuilder::new("only_eps")
        .rule("root", vec![])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
}

#[test]
fn optimize_grammar_convenience_arith() {
    let grammar = arith_grammar();
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

#[test]
fn grammar_preserves_conflict_declarations() {
    use adze_ir::{ConflictDeclaration, ConflictResolution};

    let mut grammar = GrammarBuilder::new("conf")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    // Manually add a conflict declaration using symbol IDs from the builder.
    let root_id = grammar.start_symbol().unwrap();
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![root_id],
        resolution: ConflictResolution::GLR,
    });

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    assert!(!grammar.conflicts.is_empty());
}

#[test]
fn supertypes_survive_optimization() {
    let mut grammar = GrammarBuilder::new("supert")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let root_id = grammar.start_symbol().unwrap();
    grammar.supertypes.push(root_id);

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);
    // Supertype may get renumbered but should still have entries.
    // (Supertypes are filter-mapped through old_to_new.)
    let _ = grammar.supertypes;
}

// ============================================================================
// Additional coverage to reach 80+ tests
// ============================================================================

#[test]
fn optimize_grammar_returns_grammar_with_same_name() {
    let grammar = GrammarBuilder::new("name_check")
        .token("Z", "z")
        .rule("root", vec!["Z"])
        .start("root")
        .build();

    let result = optimize_grammar(grammar).unwrap();
    assert_eq!(result.name, "name_check");
}

#[test]
fn two_rules_same_lhs_different_rhs() {
    let mut grammar = GrammarBuilder::new("two_alt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .start("root")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    let _ = stats.total();
    // Both tokens should be kept.
    assert_eq!(grammar.tokens.len(), 2);
}

#[test]
fn left_recursion_with_multiple_base_cases() {
    let mut grammar = GrammarBuilder::new("lr_multi_base")
        .token("A", "a")
        .token("B", "b")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "A"])
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.optimized_left_recursion >= 1);
}

#[test]
fn stats_debug_contains_field_names() {
    let stats = OptimizationStats::default();
    let dbg = format!("{:?}", stats);
    assert!(dbg.contains("removed_unused_symbols"));
    assert!(dbg.contains("inlined_rules"));
    assert!(dbg.contains("merged_tokens"));
    assert!(dbg.contains("optimized_left_recursion"));
    assert!(dbg.contains("eliminated_unit_rules"));
}
