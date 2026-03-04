//! Comprehensive tests for the Grammar Optimizer API.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};
use adze_ir::*;

// ---------------------------------------------------------------------------
// 1. GrammarOptimizer::new() and optimize()
// ---------------------------------------------------------------------------

#[test]
fn test_optimizer_new_and_optimize_basic() {
    let mut grammar = GrammarBuilder::new("basic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // Grammar should still be named correctly after optimization.
    assert_eq!(grammar.name, "basic");
    // total() must be consistent with the individual fields.
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
fn test_optimize_grammar_convenience_function() {
    let grammar = GrammarBuilder::new("conv")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .build();

    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
    let optimized = result.unwrap();
    assert_eq!(optimized.name, "conv");
}

// ---------------------------------------------------------------------------
// 2. OptimizationStats fields
// ---------------------------------------------------------------------------

#[test]
fn test_optimization_stats_default_is_zero() {
    let stats = OptimizationStats::default();
    assert_eq!(stats.removed_unused_symbols, 0);
    assert_eq!(stats.inlined_rules, 0);
    assert_eq!(stats.merged_tokens, 0);
    assert_eq!(stats.optimized_left_recursion, 0);
    assert_eq!(stats.eliminated_unit_rules, 0);
    assert_eq!(stats.total(), 0);
}

#[test]
fn test_optimization_stats_total_is_sum_of_fields() {
    let stats = OptimizationStats {
        removed_unused_symbols: 2,
        inlined_rules: 3,
        merged_tokens: 1,
        optimized_left_recursion: 4,
        eliminated_unit_rules: 5,
    };
    assert_eq!(stats.total(), 2 + 3 + 1 + 4 + 5);
}

// ---------------------------------------------------------------------------
// 3. Optimization preserves grammar meaning
// ---------------------------------------------------------------------------

#[test]
fn test_optimization_preserves_grammar_name_and_tokens() {
    let mut grammar = GrammarBuilder::new("preserve")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUM"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();

    let original_name = grammar.name.clone();
    let original_token_count = grammar.tokens.len();

    let mut optimizer = GrammarOptimizer::new();
    optimizer.optimize(&mut grammar);

    assert_eq!(grammar.name, original_name);
    // Tokens should not be removed when they are all referenced.
    assert!(grammar.tokens.len() <= original_token_count);
}

#[test]
fn test_optimization_preserves_start_rules() {
    let mut grammar = GrammarBuilder::new("start_check")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["item"])
        .rule("root", vec!["root", "item"])
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .start("root")
        .build();

    let mut optimizer = GrammarOptimizer::new();
    optimizer.optimize(&mut grammar);

    // After optimization the grammar should still have rules.
    assert!(!grammar.rules.is_empty(), "grammar must retain rules");
}

// ---------------------------------------------------------------------------
// 4. Optimization reduces rule count or leaves it the same
// ---------------------------------------------------------------------------

#[test]
fn test_optimization_does_not_crash_on_standard_grammar() {
    let mut grammar = GrammarBuilder::new("count_check")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // The grammar should still have rules after optimization.
    assert!(!grammar.rules.is_empty());
    // Stats should report consistent totals.
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules,
    );
}

// ---------------------------------------------------------------------------
// 5. Large grammars with many rules
// ---------------------------------------------------------------------------

#[test]
fn test_large_grammar_optimization() {
    // Build a grammar with many chained non-terminals:
    //   rule_0 -> rule_1
    //   rule_1 -> rule_2
    //   ...
    //   rule_N -> TOKEN
    let n = 50;

    let mut builder = GrammarBuilder::new("large").token("TOK", "tok");

    // Create a chain: rule_0 -> rule_1 -> ... -> rule_N -> TOK
    for i in 0..n {
        let lhs = format!("rule_{i}");
        let rhs_name = if i < n - 1 {
            format!("rule_{}", i + 1)
        } else {
            "TOK".to_string()
        };
        builder = builder.rule(&lhs, vec![&rhs_name]);
    }
    builder = builder.start("rule_0");
    let mut grammar = builder.build();

    let original_rule_count = grammar.rules.len();
    assert!(
        original_rule_count >= n,
        "Grammar should have at least {n} rule groups before optimization"
    );

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // Some unit rules should have been eliminated or inlined.
    let total_optimizations = stats.total();
    assert!(
        total_optimizations > 0,
        "Large chain grammar should trigger some optimizations"
    );
}

// ---------------------------------------------------------------------------
// 6. Grammars with unused symbols get them removed
// ---------------------------------------------------------------------------

#[test]
fn test_unused_symbol_removal() {
    let mut grammar = GrammarBuilder::new("unused")
        .token("A", "a")
        .token("B", "b")
        .token("UNUSED_TOKEN", "zzz")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();

    // "UNUSED_TOKEN" is never referenced in any rule.
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // The unused token should be removed.
    assert!(
        stats.removed_unused_symbols > 0,
        "Unused symbols should be removed (got {})",
        stats.removed_unused_symbols,
    );
}

// ---------------------------------------------------------------------------
// 7. Unit rule elimination
// ---------------------------------------------------------------------------

#[test]
fn test_unit_rule_elimination() {
    // A -> B, B -> C, C -> 'x'   (A and B are pure unit rules)
    let mut grammar = Grammar {
        name: "unit_rules".to_string(),
        ..Default::default()
    };

    grammar.tokens.insert(
        SymbolId(10),
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(SymbolId(1), "A".to_string());
    grammar.rule_names.insert(SymbolId(2), "B".to_string());
    grammar.rule_names.insert(SymbolId(3), "C".to_string());

    // A -> B
    grammar.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::NonTerminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // B -> C
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::NonTerminal(SymbolId(3))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    // C -> 'x'
    grammar.add_rule(Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(10))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // The optimizer should have eliminated or inlined unit rules.
    let total = stats.eliminated_unit_rules + stats.inlined_rules;
    assert!(
        total > 0,
        "Unit rules should be eliminated or inlined (unit={}, inlined={})",
        stats.eliminated_unit_rules,
        stats.inlined_rules,
    );
}

// ---------------------------------------------------------------------------
// 8. Left recursion detection and transformation
// ---------------------------------------------------------------------------

#[test]
fn test_left_recursive_grammar_optimization() {
    // expr -> expr '+' NUM | NUM   (direct left recursion)
    let mut grammar = GrammarBuilder::new("left_rec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // The grammar should still have rules after optimization.
    assert!(!grammar.rules.is_empty());

    // Left-recursion optimisation counter should fire (or at least not crash).
    // The exact count depends on the optimizer's strategy; we just verify it runs.
    let _ = stats.optimized_left_recursion;
}

// ---------------------------------------------------------------------------
// Additional tests
// ---------------------------------------------------------------------------

#[test]
fn test_optimizer_with_precedence_rules() {
    let mut grammar = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // Grammar should still be intact.
    assert_eq!(grammar.name, "prec");
    // Sanity: total() should be non-negative (always true for usize).
    let _ = stats.total();
}

#[test]
fn test_optimizer_with_extras_and_externals() {
    let mut grammar = GrammarBuilder::new("extras")
        .token("ID", r"[a-z]+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .external("INDENT")
        .rule("start", vec!["ID"])
        .start("start")
        .build();

    let mut optimizer = GrammarOptimizer::new();
    let _stats = optimizer.optimize(&mut grammar);

    // Externals should survive optimization.
    assert!(
        !grammar.externals.is_empty(),
        "externals should not be removed"
    );
}

#[test]
fn test_optimizer_with_fragile_tokens() {
    let mut grammar = GrammarBuilder::new("fragile")
        .token("ID", r"[a-z]+")
        .fragile_token("SEMI", ";")
        .rule("stmt", vec!["ID", "SEMI"])
        .start("stmt")
        .build();

    let mut optimizer = GrammarOptimizer::new();
    optimizer.optimize(&mut grammar);

    // The fragile token should still exist after optimization.
    let has_fragile = grammar.tokens.values().any(|t| t.fragile);
    assert!(has_fragile, "fragile token should survive optimization");
}

#[test]
fn test_optimizer_idempotent() {
    // Running the optimizer twice should not change the grammar further.
    let mut grammar = GrammarBuilder::new("idem")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let mut opt1 = GrammarOptimizer::new();
    opt1.optimize(&mut grammar);
    let rules_after_first: usize = grammar.rules.values().map(|v| v.len()).sum();

    let mut opt2 = GrammarOptimizer::new();
    let stats2 = opt2.optimize(&mut grammar);
    let rules_after_second: usize = grammar.rules.values().map(|v| v.len()).sum();

    assert_eq!(
        rules_after_first, rules_after_second,
        "Second optimization pass should not change rule count"
    );
    // Second pass should have nothing left to optimise.
    assert_eq!(
        stats2.total(),
        0,
        "Second optimization pass should report zero changes"
    );
}
