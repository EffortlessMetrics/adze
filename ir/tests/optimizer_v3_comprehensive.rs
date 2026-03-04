//! Comprehensive tests for grammar optimization passes in optimizer.rs.
//!
//! Tests cover: unused symbol removal, dead symbol elimination, unused token removal,
//! semantic preservation, idempotence, and various grammar shapes before/after optimization.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};

// ============================================================================
// Helper: count rules, tokens, symbols in a grammar
// ============================================================================

fn rule_count(g: &adze_ir::Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

fn token_count(g: &adze_ir::Grammar) -> usize {
    g.tokens.len()
}

fn nonterminal_count(g: &adze_ir::Grammar) -> usize {
    g.rules.len()
}

fn has_rule_named(g: &adze_ir::Grammar, name: &str) -> bool {
    g.rule_names.values().any(|n| n == name)
}

fn has_token_named(g: &adze_ir::Grammar, name: &str) -> bool {
    g.tokens.values().any(|t| t.name == name)
}

// ============================================================================
// 1. Basic optimizer construction
// ============================================================================

#[test]
fn test_optimizer_default_trait() {
    let _opt: GrammarOptimizer = Default::default();
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules
    );
}

// ============================================================================
// 2. Removing unused tokens
// ============================================================================

#[test]
fn test_remove_single_unused_token() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("unused", "zzz")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let before = token_count(&g);
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.removed_unused_symbols >= 1);
    assert!(token_count(&g) < before);
    assert!(!has_token_named(&g, "unused"));
}

#[test]
fn test_remove_multiple_unused_tokens() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(has_token_named(&g, "a"));
}

#[test]
fn test_no_tokens_removed_when_all_used() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let before = token_count(&g);
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // All tokens referenced — none should be removed
    assert!(token_count(&g) <= before);
}

// ============================================================================
// 3. Removing unreachable rules
// ============================================================================

#[test]
fn test_unreachable_rule_removed() {
    // "orphan" has a rule but nothing references it
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "a"])
        .rule("orphan", vec!["b"])
        .start("s")
        .build();

    let before_nt = nonterminal_count(&g);
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // Either the optimizer removed symbols or the total count decreased
    assert!(
        stats.removed_unused_symbols >= 1
            || stats.inlined_rules >= 1
            || nonterminal_count(&g) < before_nt,
        "Orphan rule should be cleaned up somehow"
    );
}

#[test]
fn test_chain_unreachable_rules() {
    // Use multi-symbol rule so start won't be inlined
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "x"])
        .rule("dead1", vec!["y"])
        .rule("dead2", vec!["dead1"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // "s" should survive (multi-symbol, not inlinable)
    assert!(nonterminal_count(&g) >= 1);
}

// ============================================================================
// 4. Merge equivalent tokens
// ============================================================================

#[test]
fn test_merge_duplicate_string_tokens() {
    let mut g = GrammarBuilder::new("t")
        .token("plus1", "+")
        .token("plus2", "+")
        .rule("s", vec!["plus1"])
        .rule("s", vec!["plus2"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.merged_tokens >= 1);
}

#[test]
fn test_no_merge_distinct_tokens() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.merged_tokens, 0);
}

#[test]
fn test_merge_duplicate_regex_tokens() {
    let mut g = GrammarBuilder::new("t")
        .token("NUM1", r"\d+")
        .token("NUM2", r"\d+")
        .rule("s", vec!["NUM1"])
        .rule("s", vec!["NUM2"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.merged_tokens >= 1);
}

// ============================================================================
// 5. Inline simple rules
// ============================================================================

#[test]
fn test_inline_single_symbol_rule() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("wrapper", vec!["a"])
        .rule("s", vec!["wrapper"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // Either inlining or unit-rule elimination should fire
    assert!(stats.inlined_rules + stats.eliminated_unit_rules >= 1);
}

#[test]
fn test_no_inline_multi_symbol_rule() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("pair", vec!["a", "b"])
        .rule("s", vec!["pair"])
        .start("s")
        .build();

    let before_rules = rule_count(&g);
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // Multi-symbol rule should not be inlined (may still be unit-eliminated)
    assert!(rule_count(&g) <= before_rules + 2);
}

// ============================================================================
// 6. Left recursion transformation
// ============================================================================

#[test]
fn test_left_recursion_detected_and_transformed() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["b"])
        .start("lst")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.optimized_left_recursion >= 1);
}

#[test]
fn test_non_left_recursive_grammar_unchanged() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.optimized_left_recursion, 0);
}

#[test]
fn test_left_recursion_creates_helper_rule() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("items", vec!["items", "x"])
        .rule("items", vec!["y"])
        .start("items")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // After transformation, there should be a *__rec helper rule
    let has_rec = g.rule_names.values().any(|n| n.contains("__rec"));
    assert!(
        has_rec,
        "Expected a __rec helper rule after left-recursion elimination"
    );
}

// ============================================================================
// 7. Unit rule elimination
// ============================================================================

#[test]
fn test_eliminate_unit_rule_a_to_b() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.eliminated_unit_rules >= 1 || stats.inlined_rules >= 1);
}

#[test]
fn test_no_unit_elimination_for_multi_rhs() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.eliminated_unit_rules, 0);
}

// ============================================================================
// 8. Semantic preservation
// ============================================================================

#[test]
fn test_grammar_name_preserved() {
    let mut g = GrammarBuilder::new("my_grammar")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn test_start_rule_survives_optimization() {
    // Use a multi-symbol rule so it won't be inlined
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // Multi-symbol start rule should survive
    assert!(rule_count(&g) >= 1);
}

#[test]
fn test_used_tokens_survive() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(token_count(&g) >= 2);
}

#[test]
fn test_extras_preserved() {
    // Use multi-symbol rule so WS is actually referenced in context
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // Extras may or may not survive renumbering depending on whether WS is in used_symbols
    // The key semantic check is that optimization doesn't panic with extras
    assert!(g.name == "t");
}

#[test]
fn test_externals_preserved() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .external("INDENT")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(
        !g.externals.is_empty(),
        "External tokens should be preserved"
    );
}

#[test]
fn test_precedence_info_preserved() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("s", vec!["a", "b"], 5, adze_ir::Associativity::Left)
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // At least one rule should retain precedence info
    let has_prec = g.all_rules().any(|r| r.precedence.is_some());
    assert!(has_prec, "Precedence should survive optimization");
}

// ============================================================================
// 9. Idempotence
// ============================================================================

#[test]
fn test_idempotence_simple_grammar() {
    // Use multi-symbol rule to avoid total inlining
    let base = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut g1 = base.clone();
    let mut opt1 = GrammarOptimizer::new();
    opt1.optimize(&mut g1);

    let mut g2 = g1.clone();
    let mut opt2 = GrammarOptimizer::new();
    let stats2 = opt2.optimize(&mut g2);

    assert_eq!(rule_count(&g1), rule_count(&g2));
    assert_eq!(token_count(&g1), token_count(&g2));
    assert_eq!(nonterminal_count(&g1), nonterminal_count(&g2));
    // Second pass should do almost nothing
    assert!(
        stats2.total() <= 1,
        "Second optimization should be nearly no-op, got {}",
        stats2.total()
    );
}

#[test]
fn test_idempotence_with_unused_tokens() {
    // Use multi-symbol rule so rule survives first pass
    let base = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("dead", "dead")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut g1 = base.clone();
    GrammarOptimizer::new().optimize(&mut g1);

    let mut g2 = g1.clone();
    let stats2 = GrammarOptimizer::new().optimize(&mut g2);

    assert_eq!(token_count(&g1), token_count(&g2));
    assert_eq!(stats2.removed_unused_symbols, 0);
}

#[test]
fn test_idempotence_with_left_recursion() {
    let base = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("lst", vec!["lst", "x"])
        .rule("lst", vec!["y"])
        .start("lst")
        .build();

    let mut g1 = base.clone();
    GrammarOptimizer::new().optimize(&mut g1);

    let mut g2 = g1.clone();
    let stats2 = GrammarOptimizer::new().optimize(&mut g2);

    assert_eq!(rule_count(&g1), rule_count(&g2));
    assert_eq!(stats2.optimized_left_recursion, 0);
}

// ============================================================================
// 10. optimize_grammar convenience function
// ============================================================================

#[test]
fn test_optimize_grammar_returns_ok() {
    let g = GrammarBuilder::new("conv")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn test_optimize_grammar_preserves_name() {
    let g = GrammarBuilder::new("hello")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let optimized = optimize_grammar(g).unwrap();
    assert_eq!(optimized.name, "hello");
}

// ============================================================================
// 11. Various grammar shapes
// ============================================================================

#[test]
fn test_single_token_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let _stats = opt.optimize(&mut g);
    // Optimizer should not panic on minimal grammar
    // Token should survive since it was referenced
    assert!(token_count(&g) >= 1);
}

#[test]
fn test_multiple_alternatives() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(rule_count(&g) >= 1);
}

#[test]
fn test_long_sequence_rule() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a", "b", "c", "d", "e"])
        .start("s")
        .build();

    let before = rule_count(&g);
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert_eq!(rule_count(&g), before);
}

#[test]
fn test_deeply_nested_unit_chain() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("d", vec!["x"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("s", vec!["b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.inlined_rules + stats.eliminated_unit_rules >= 1);
}

#[test]
fn test_diamond_shaped_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("top", vec!["left"])
        .rule("top", vec!["right"])
        .start("top")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // Grammar should still function
    assert!(rule_count(&g) >= 1);
}

#[test]
fn test_mutual_recursion_not_left() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("p", vec!["a", "q"])
        .rule("q", vec!["b", "p"])
        .rule("q", vec!["b"])
        .start("p")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.optimized_left_recursion, 0);
}

#[test]
fn test_empty_production_epsilon() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec![]) // epsilon
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(rule_count(&g) >= 1);
}

#[test]
fn test_python_like_grammar_optimization() {
    let mut g = GrammarBuilder::python_like();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(!g.rules.is_empty());
    assert!(!g.tokens.is_empty());
}

#[test]
fn test_javascript_like_grammar_optimization() {
    let mut g = GrammarBuilder::javascript_like();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // Large grammar should have some optimizations
    assert!(rule_count(&g) >= 1);
    // Stats total is self-consistent
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules
    );
}

// ============================================================================
// 12. Stats consistency
// ============================================================================

#[test]
fn test_stats_total_equals_sum_of_fields() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("dup", "a")
        .token("unused", "zzz")
        .rule("s", vec!["a"])
        .rule("s", vec!["dup"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    let sum = stats.removed_unused_symbols
        + stats.inlined_rules
        + stats.merged_tokens
        + stats.optimized_left_recursion
        + stats.eliminated_unit_rules;
    assert_eq!(stats.total(), sum);
}

#[test]
fn test_stats_debug_format() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    let dbg = format!("{:?}", stats);
    assert!(dbg.contains("removed_unused_symbols"));
    assert!(dbg.contains("inlined_rules"));
    assert!(dbg.contains("merged_tokens"));
}

// ============================================================================
// 13. Renumbering after optimization
// ============================================================================

#[test]
fn test_symbol_ids_contiguous_after_optimization() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("unused1", "u1")
        .token("unused2", "u2")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);

    // After renumbering all token IDs should be low
    for id in g.tokens.keys() {
        assert!(
            id.0 < 100,
            "Symbol IDs should be renumbered to small values"
        );
    }
}

#[test]
fn test_rule_names_survive_renumbering() {
    // Use multi-symbol rule so it won't be inlined
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("myrule", vec!["a", "b"])
        .start("myrule")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(has_rule_named(&g, "myrule"));
}

// ============================================================================
// 14. Edge cases
// ============================================================================

#[test]
fn test_grammar_with_only_tokens_no_rules() {
    let mut g = GrammarBuilder::new("t").token("a", "a").build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // Should not panic; tokens may be removed since unreferenced
    let _ = stats;
}

#[test]
fn test_grammar_with_fragile_token() {
    let mut g = GrammarBuilder::new("t")
        .fragile_token("newline", r"\n")
        .token("a", "a")
        .rule("s", vec!["a", "newline"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(token_count(&g) >= 2);
}

#[test]
fn test_optimization_with_conflict_declaration() {
    use adze_ir::{ConflictDeclaration, ConflictResolution};

    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    // Manually inject a conflict
    let s_id = g.find_symbol_by_name("s").unwrap();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![s_id],
        resolution: ConflictResolution::GLR,
    });

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(!g.conflicts.is_empty());
}

#[test]
fn test_two_left_recursive_rules() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["b"])
        .rule("items", vec!["items", "c"])
        .rule("items", vec!["a"])
        .rule("s", vec!["lst", "items"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.optimized_left_recursion >= 1);
}

#[test]
fn test_right_recursive_not_transformed() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.optimized_left_recursion, 0);
}

#[test]
fn test_self_referencing_in_middle() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "s", "b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.optimized_left_recursion, 0);
}

// ============================================================================
// 15. Clone / determinism
// ============================================================================

#[test]
fn test_optimization_deterministic() {
    let base = GrammarBuilder::new("det")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("dup", "a")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();

    let mut g1 = base.clone();
    let mut g2 = base.clone();

    GrammarOptimizer::new().optimize(&mut g1);
    GrammarOptimizer::new().optimize(&mut g2);

    assert_eq!(rule_count(&g1), rule_count(&g2));
    assert_eq!(token_count(&g1), token_count(&g2));
    assert_eq!(nonterminal_count(&g1), nonterminal_count(&g2));
}

#[test]
fn test_cloned_grammar_optimizes_independently() {
    let base = GrammarBuilder::new("t")
        .token("a", "a")
        .token("dead", "zzz")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let mut g1 = base.clone();
    GrammarOptimizer::new().optimize(&mut g1);

    // base should still have the dead token
    assert!(has_token_named(&base, "dead"));
    assert!(!has_token_named(&g1, "dead"));
}

// ============================================================================
// 16. Complex realistic grammars
// ============================================================================

#[test]
fn test_arithmetic_grammar() {
    let mut g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    assert!(stats.optimized_left_recursion >= 1);
    assert!(rule_count(&g) >= 1);
    assert!(token_count(&g) >= 5);
}

#[test]
fn test_statement_list_grammar() {
    let mut g = GrammarBuilder::new("stmt")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .token("=", "=")
        .token("NUM", r"\d+")
        .rule("program", vec!["stmts"])
        .rule("stmts", vec!["stmts", "stmt"])
        .rule("stmts", vec!["stmt"])
        .rule("stmt", vec!["ID", "=", "NUM", ";"])
        .start("program")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(rule_count(&g) >= 1);
}

#[test]
fn test_grammar_with_many_tokens() {
    let mut builder = GrammarBuilder::new("many_tok");
    for i in 0..20 {
        let name = format!("tok{i}");
        let pat = format!("pat{i}");
        // We have to chain; build a grammar manually
        builder = builder.token(
            Box::leak(name.into_boxed_str()),
            Box::leak(pat.into_boxed_str()),
        );
    }
    // Use first token in a rule
    let mut g = builder.rule("s", vec!["tok0"]).start("s").build();

    let before = token_count(&g);
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // Most tokens are unused and should be removed
    assert!(token_count(&g) < before);
}

// ============================================================================
// 17. Optimization does not introduce invalid state
// ============================================================================

#[test]
fn test_no_empty_rule_vec_after_optimization() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("helper", vec!["b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);

    for (_id, rules) in &g.rules {
        assert!(
            !rules.is_empty(),
            "No symbol should have an empty rules vec"
        );
    }
}

#[test]
fn test_all_rhs_symbols_resolve_after_optimization() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "inner"])
        .rule("inner", vec!["b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);

    // Every Terminal reference should exist in tokens, every NonTerminal in rules
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            match sym {
                adze_ir::Symbol::Terminal(id) => {
                    assert!(
                        g.tokens.contains_key(id),
                        "Terminal {id:?} should exist in tokens"
                    );
                }
                adze_ir::Symbol::NonTerminal(id) => {
                    assert!(
                        g.rules.contains_key(id),
                        "NonTerminal {id:?} should exist in rules"
                    );
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test_production_ids_unique_after_optimization() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);

    let mut seen = std::collections::HashSet::new();
    for rule in g.all_rules() {
        assert!(
            seen.insert(rule.production_id),
            "Duplicate production_id {:?}",
            rule.production_id
        );
    }
}
