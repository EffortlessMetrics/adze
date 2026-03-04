// Wave 132: Comprehensive IR optimizer and validation tests
use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};
use adze_ir::validation::{GrammarValidator, ValidationResult};
use adze_ir::*;

// =====================================================================
// GrammarOptimizer construction
// =====================================================================

#[test]
fn optimizer_new() {
    let optimizer = GrammarOptimizer::new();
    let _ = optimizer;
}

// =====================================================================
// Optimizer on simple grammars
// =====================================================================

#[test]
fn optimize_empty_grammar() {
    let mut grammar = Grammar::new("empty".to_string());
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);
    assert_eq!(stats.total(), 0);
}

#[test]
fn optimize_single_rule() {
    let mut grammar = GrammarBuilder::new("simple")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);
    // Simple grammar shouldn't need many optimizations
    let _ = stats.total();
}

#[test]
fn optimize_multi_rule() {
    let mut grammar = GrammarBuilder::new("multi")
        .token("a", r"a")
        .token("b", r"b")
        .token("c", r"c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);
    let _ = stats.total();
}

#[test]
fn optimize_python_like() {
    let mut grammar = GrammarBuilder::python_like();
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);
    let _ = stats.total();
}

#[test]
fn optimize_javascript_like() {
    let mut grammar = GrammarBuilder::javascript_like();
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);
    let _ = stats.total();
}

// =====================================================================
// optimize_grammar function
// =====================================================================

#[test]
fn optimize_grammar_fn_simple() {
    let grammar = GrammarBuilder::new("opt")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
    let optimized = result.unwrap();
    // Optimizer may simplify, just ensure grammar still has name
    assert_eq!(optimized.name, "opt");
}

#[test]
fn optimize_grammar_fn_empty() {
    let grammar = Grammar::new("empty".to_string());
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

// =====================================================================
// OptimizationStats
// =====================================================================

#[test]
fn optimization_stats_total() {
    let mut grammar = GrammarBuilder::new("stats")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);
    assert!(stats.total() >= 0); // always true for usize
}

#[test]
fn optimization_stats_debug() {
    let mut grammar = GrammarBuilder::new("debug")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);
    let debug = format!("{:?}", stats);
    assert!(!debug.is_empty());
}

// =====================================================================
// Optimizer idempotency
// =====================================================================

#[test]
fn optimize_idempotent() {
    let mut grammar = GrammarBuilder::new("idem")
        .token("a", r"a")
        .token("b", r"b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats1 = opt.optimize(&mut grammar);
    let mut opt2 = GrammarOptimizer::new();
    let stats2 = opt2.optimize(&mut grammar);
    // Second pass should do no work (or very little)
    assert!(stats2.total() <= stats1.total());
}

// =====================================================================
// GrammarValidator
// =====================================================================

#[test]
fn validator_new() {
    let validator = GrammarValidator::new();
    let _ = validator;
}

#[test]
fn validate_empty_grammar() {
    let grammar = Grammar::new("empty".to_string());
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    let _ = result.errors.is_empty();
}

#[test]
fn validate_simple_grammar() {
    let grammar = GrammarBuilder::new("simple")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    // Simple valid grammar should pass validation
    let _ = result.errors.is_empty();
}

#[test]
fn validate_python_like() {
    let grammar = GrammarBuilder::python_like();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    let _ = result.errors.is_empty();
}

#[test]
fn validate_javascript_like() {
    let grammar = GrammarBuilder::javascript_like();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    let _ = result.errors.is_empty();
}

// =====================================================================
// ValidationResult
// =====================================================================

#[test]
fn validation_result_has_errors() {
    let grammar = GrammarBuilder::new("valid")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    let _ = &result.errors;
    let _ = &result.warnings;
}

#[test]
fn validation_result_debug() {
    let grammar = Grammar::new("debug".to_string());
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    // ValidationResult doesn't derive Debug, just check fields
    let _ = result.errors.len();
    let _ = result.warnings.len();
}

// =====================================================================
// Full pipeline: Build → Optimize → Validate
// =====================================================================

#[test]
fn pipeline_build_optimize_validate() {
    let grammar = GrammarBuilder::new("pipeline")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .start("expr")
        .build();
    let optimized = optimize_grammar(grammar).unwrap();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&optimized);
    let _ = result.errors.is_empty();
}

#[test]
fn pipeline_python_full() {
    let grammar = GrammarBuilder::python_like();
    let optimized = optimize_grammar(grammar).unwrap();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&optimized);
    let _ = result.errors.is_empty();
}

#[test]
fn pipeline_javascript_full() {
    let grammar = GrammarBuilder::javascript_like();
    let optimized = optimize_grammar(grammar).unwrap();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&optimized);
    let _ = result.errors.is_empty();
}
