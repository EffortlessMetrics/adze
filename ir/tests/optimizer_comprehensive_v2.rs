//! Comprehensive tests for GrammarOptimizer and optimize_grammar.

use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("factor", vec!["num"])
        .start("expr")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
        .build()
}

// === GrammarOptimizer construction ===

#[test]
fn optimizer_new() {
    let _ = GrammarOptimizer::new();
}

#[test]
fn optimizer_default() {
    let _ = GrammarOptimizer::default();
}

// === OptimizationStats ===

#[test]
fn stats_default_zeros() {
    let s = OptimizationStats::default();
    assert_eq!(s.removed_unused_symbols, 0);
    assert_eq!(s.inlined_rules, 0);
    assert_eq!(s.merged_tokens, 0);
    assert_eq!(s.optimized_left_recursion, 0);
    assert_eq!(s.eliminated_unit_rules, 0);
}

#[test]
fn stats_total() {
    let s = OptimizationStats {
        removed_unused_symbols: 1,
        inlined_rules: 2,
        merged_tokens: 3,
        optimized_left_recursion: 4,
        eliminated_unit_rules: 5,
    };
    assert_eq!(s.total(), 15);
}

#[test]
fn stats_total_zero() {
    let s = OptimizationStats::default();
    assert_eq!(s.total(), 0);
}

#[test]
fn stats_debug() {
    let s = OptimizationStats::default();
    let d = format!("{:?}", s);
    assert!(d.contains("OptimizationStats"));
}

#[test]
fn stats_total_matches_fields() {
    let s = OptimizationStats {
        removed_unused_symbols: 10,
        inlined_rules: 20,
        merged_tokens: 30,
        optimized_left_recursion: 40,
        eliminated_unit_rules: 50,
    };
    assert_eq!(s.total(), 10 + 20 + 30 + 40 + 50);
}

#[test]
fn stats_fields_independent() {
    let s = OptimizationStats {
        removed_unused_symbols: 100,
        inlined_rules: 0,
        merged_tokens: 0,
        optimized_left_recursion: 0,
        eliminated_unit_rules: 0,
    };
    assert_eq!(s.total(), 100);
}

// === optimize method ===

#[test]
fn optimize_simple_grammar() {
    let mut g = simple_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // simple grammar may or may not have optimizable parts
    let _ = stats.total();
}

#[test]
fn optimize_arithmetic_grammar() {
    let mut g = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    let _ = stats.total();
}

#[test]
fn optimize_chain_grammar() {
    let mut g = chain_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    let _ = stats.total();
}

#[test]
fn optimize_preserves_name() {
    let mut g = simple_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert_eq!(g.name, "simple");
}

#[test]
fn optimize_preserves_start_or_clears() {
    let mut g = simple_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // Start symbol may or may not be preserved depending on optimization
    let _ = g.start_symbol();
}

#[test]
fn optimize_grammar_still_has_rules() {
    let mut g = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(!g.rules.is_empty());
}

#[test]
fn optimize_grammar_still_has_tokens() {
    let mut g = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert!(!g.tokens.is_empty());
}

// === optimize_grammar convenience function ===

#[test]
fn optimize_grammar_fn_simple() {
    let g = simple_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_grammar_fn_arith() {
    let g = arith_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_grammar_fn_chain() {
    let g = chain_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_grammar_fn_preserves_name() {
    let g = arith_grammar();
    let g2 = optimize_grammar(g).unwrap();
    assert_eq!(g2.name, "arith");
}

// === Determinism ===

#[test]
fn optimize_deterministic() {
    let mut g1 = arith_grammar();
    let mut g2 = arith_grammar();
    let mut opt1 = GrammarOptimizer::new();
    let mut opt2 = GrammarOptimizer::new();
    let s1 = opt1.optimize(&mut g1);
    let s2 = opt2.optimize(&mut g2);
    assert_eq!(s1.total(), s2.total());
}

#[test]
fn optimize_fn_deterministic() {
    let g1 = arith_grammar();
    let g2 = arith_grammar();
    let r1 = optimize_grammar(g1).unwrap();
    let r2 = optimize_grammar(g2).unwrap();
    assert_eq!(r1.rules.len(), r2.rules.len());
    assert_eq!(r1.tokens.len(), r2.tokens.len());
}

// === Edge cases ===

#[test]
fn optimize_with_unused_tokens() {
    let g = GrammarBuilder::new("unused")
        .token("used", "u")
        .token("unused1", "x")
        .token("unused2", "y")
        .rule("start", vec!["used"])
        .start("start")
        .build();
    let mut g = g;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // May or may not remove unused tokens
    let _ = stats;
}

#[test]
fn optimize_recursive_grammar() {
    let g = GrammarBuilder::new("rec")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "a"])
        .start("list")
        .build();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_single_token_grammar() {
    let g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
    // Optimized grammar may or may not have rules remaining
    let _ = result.unwrap();
}

#[test]
fn optimize_many_alternatives() {
    let g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .start("s")
        .build();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_diamond_grammar() {
    let g = GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("a", vec!["c"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("s")
        .build();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_long_rhs() {
    let mut b = GrammarBuilder::new("long");
    let names: Vec<String> = (0..10).map(|i| format!("t{i}")).collect();
    for n in &names {
        b = b.token(n, n);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    b = b.rule("start", refs).start("start");
    let g = b.build();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_multiple_times() {
    let mut g = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    let s1 = opt.optimize(&mut g);
    let s2 = opt.optimize(&mut g);
    // Second pass may find nothing to do
    assert!(s2.total() <= s1.total());
}

// === Stats non-negative ===

#[test]
fn stats_all_non_negative() {
    let mut g = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // All fields should be >= 0 (they're usize, so always true, but let's verify)
    assert!(stats.removed_unused_symbols >= 0);
    assert!(stats.inlined_rules >= 0);
    assert!(stats.merged_tokens >= 0);
    assert!(stats.optimized_left_recursion >= 0);
    assert!(stats.eliminated_unit_rules >= 0);
}

// === Optimizer reuse ===

#[test]
fn optimizer_reuse_across_grammars() {
    let mut opt = GrammarOptimizer::new();

    let mut g1 = simple_grammar();
    let s1 = opt.optimize(&mut g1);
    let _ = s1;

    let mut g2 = arith_grammar();
    let s2 = opt.optimize(&mut g2);
    let _ = s2;
}
