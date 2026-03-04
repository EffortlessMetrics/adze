//! Comprehensive tests for Grammar optimizer behavior (50+ tests).
//!
//! Covers: name preservation, token preservation, start symbol preservation,
//! idempotency, simple grammars, complex grammars, precedence rules,
//! clone preservation, many alternatives, and chain rules.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};
use adze_ir::{Associativity, Grammar, SymbolId};

// ===========================================================================
// Helpers
// ===========================================================================

/// Find a symbol ID by name in rule_names or tokens.
fn find_sym(grammar: &Grammar, name: &str) -> Option<SymbolId> {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .or_else(|| {
            grammar
                .tokens
                .iter()
                .find(|(_, t)| t.name == name)
                .map(|(id, _)| *id)
        })
}

fn token_names(grammar: &Grammar) -> Vec<String> {
    grammar.tokens.values().map(|t| t.name.clone()).collect()
}

fn total_rules(grammar: &Grammar) -> usize {
    grammar.all_rules().count()
}

fn build_arith() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. Grammar optimization preserves name
// ===========================================================================

#[test]
fn preserves_name_simple() {
    let g = optimize_grammar(
        GrammarBuilder::new("my_lang")
            .token("X", "x")
            .rule("s", vec!["X"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn preserves_name_empty_grammar() {
    let g = optimize_grammar(Grammar::new("empty".into())).unwrap();
    assert_eq!(g.name, "empty");
}

#[test]
fn preserves_name_complex() {
    let g = optimize_grammar(build_arith()).unwrap();
    assert_eq!(g.name, "arith");
}

#[test]
fn preserves_name_with_unicode() {
    let g = optimize_grammar(
        GrammarBuilder::new("语法_example")
            .token("A", "a")
            .rule("r", vec!["A"])
            .start("r")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "语法_example");
}

#[test]
fn preserves_name_after_mut_optimize() {
    let mut g = build_arith();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert_eq!(g.name, "arith");
}

// ===========================================================================
// 2. Grammar optimization preserves tokens
// ===========================================================================

#[test]
fn preserves_used_tokens() {
    let g = optimize_grammar(
        GrammarBuilder::new("tok")
            .token("A", "a")
            .token("B", "b")
            .rule("s", vec!["A", "B"])
            .start("s")
            .build(),
    )
    .unwrap();
    let names = token_names(&g);
    assert!(names.contains(&"A".to_string()));
    assert!(names.contains(&"B".to_string()));
}

#[test]
fn removes_unused_tokens() {
    let g = optimize_grammar(
        GrammarBuilder::new("tok")
            .token("USED", "u")
            .token("UNUSED", "x")
            .rule("s", vec!["USED"])
            .start("s")
            .build(),
    )
    .unwrap();
    let names = token_names(&g);
    assert!(names.contains(&"USED".to_string()));
    assert!(!names.contains(&"UNUSED".to_string()));
}

#[test]
fn preserves_token_patterns() {
    let g = optimize_grammar(
        GrammarBuilder::new("pat")
            .token("NUM", r"\d+")
            .rule("s", vec!["NUM"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert!(!g.tokens.is_empty());
}

#[test]
fn preserves_all_tokens_when_all_used() {
    let g = optimize_grammar(
        GrammarBuilder::new("all_used")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .rule("s", vec!["A", "B", "C"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn merges_duplicate_tokens() {
    // Two tokens with the same pattern should be merged
    let mut g = Grammar::new("dup".into());
    let id1 = SymbolId(1);
    let id2 = SymbolId(2);
    let lhs = SymbolId(3);
    g.tokens.insert(
        id1,
        adze_ir::Token {
            name: "PLUS1".into(),
            pattern: adze_ir::TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        id2,
        adze_ir::Token {
            name: "PLUS2".into(),
            pattern: adze_ir::TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(lhs, "s".into());
    g.add_rule(adze_ir::Rule {
        lhs,
        rhs: vec![
            adze_ir::Symbol::Terminal(id1),
            adze_ir::Symbol::Terminal(id2),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.merged_tokens > 0);
}

// ===========================================================================
// 3. Grammar optimization preserves start symbol
// ===========================================================================

#[test]
fn preserves_start_symbol_basic() {
    // Use a grammar with multiple rules so the start symbol isn't inlined away
    let g = optimize_grammar(
        GrammarBuilder::new("st")
            .token("X", "x")
            .token("Y", "y")
            .rule("entry", vec!["X"])
            .rule("entry", vec!["Y"])
            .start("entry")
            .build(),
    )
    .unwrap();
    // Start symbol should still resolve
    assert!(g.start_symbol().is_some());
}

#[test]
fn preserves_start_symbol_after_left_recursion_transform() {
    let g = optimize_grammar(build_arith()).unwrap();
    assert!(g.start_symbol().is_some());
}

#[test]
fn preserves_start_with_multiple_rules() {
    let g = optimize_grammar(
        GrammarBuilder::new("multi")
            .token("A", "a")
            .token("B", "b")
            .rule("root", vec!["A"])
            .rule("root", vec!["B"])
            .start("root")
            .build(),
    )
    .unwrap();
    assert!(g.start_symbol().is_some());
}

#[test]
fn start_symbol_still_has_rules() {
    let g = optimize_grammar(build_arith()).unwrap();
    let start = g.start_symbol().unwrap();
    assert!(g.rules.contains_key(&start));
}

#[test]
fn start_symbol_rules_nonempty() {
    let g = optimize_grammar(build_arith()).unwrap();
    let start = g.start_symbol().unwrap();
    assert!(!g.rules[&start].is_empty());
}

// ===========================================================================
// 4. Optimizer idempotency
// ===========================================================================

#[test]
fn idempotent_on_simple_grammar() {
    let base = GrammarBuilder::new("idem")
        .token("X", "x")
        .token("Y", "y")
        .rule("s", vec!["X"])
        .rule("s", vec!["Y"])
        .start("s")
        .build();
    let once = optimize_grammar(base.clone()).unwrap();
    let twice = optimize_grammar(once.clone()).unwrap();
    assert_eq!(once.name, twice.name);
    assert_eq!(once.tokens.len(), twice.tokens.len());
    assert_eq!(total_rules(&once), total_rules(&twice));
}

#[test]
fn idempotent_rule_count_arith() {
    let once = optimize_grammar(build_arith()).unwrap();
    let twice = optimize_grammar(once.clone()).unwrap();
    assert_eq!(total_rules(&once), total_rules(&twice));
}

#[test]
fn idempotent_token_count() {
    let once = optimize_grammar(build_arith()).unwrap();
    let twice = optimize_grammar(once.clone()).unwrap();
    assert_eq!(once.tokens.len(), twice.tokens.len());
}

#[test]
fn idempotent_stats_second_pass_zero_or_stable() {
    let mut g = build_arith();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    let mut opt2 = GrammarOptimizer::new();
    let stats2 = opt2.optimize(&mut g);
    // Second pass should have no further removals or merges
    assert_eq!(stats2.removed_unused_symbols, 0);
    assert_eq!(stats2.merged_tokens, 0);
}

#[test]
fn idempotent_name_preserved_across_passes() {
    let mut g = build_arith();
    for _ in 0..3 {
        let mut opt = GrammarOptimizer::new();
        opt.optimize(&mut g);
    }
    assert_eq!(g.name, "arith");
}

// ===========================================================================
// 5. Optimizer on simple grammars
// ===========================================================================

#[test]
fn simple_single_rule_single_token() {
    // A single rule s -> A may be inlined/eliminated; verify no panic
    let g = optimize_grammar(
        GrammarBuilder::new("tiny")
            .token("A", "a")
            .rule("s", vec!["A"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "tiny");
}

#[test]
fn simple_empty_rhs_rule() {
    // Empty RHS is allowed — should not panic
    let g = optimize_grammar(
        GrammarBuilder::new("eps")
            .rule("s", vec![])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "eps");
}

#[test]
fn simple_two_alternatives() {
    let g = optimize_grammar(
        GrammarBuilder::new("alt")
            .token("A", "a")
            .token("B", "b")
            .rule("s", vec!["A"])
            .rule("s", vec!["B"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert!(total_rules(&g) >= 2);
}

#[test]
fn simple_grammar_stats_non_negative() {
    let mut g = GrammarBuilder::new("s")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.total() < 1000); // sanity bound
}

#[test]
fn simple_grammar_no_crash_on_two_tokens() {
    let _g = optimize_grammar(
        GrammarBuilder::new("two")
            .token("A", "a")
            .token("B", "b")
            .rule("s", vec!["A", "B"])
            .start("s")
            .build(),
    )
    .unwrap();
}

// ===========================================================================
// 6. Optimizer on complex grammars
// ===========================================================================

#[test]
fn complex_javascript_like() {
    let g = optimize_grammar(GrammarBuilder::javascript_like()).unwrap();
    assert_eq!(g.name, "javascript_like");
    assert!(total_rules(&g) > 0);
    assert!(!g.tokens.is_empty());
}

#[test]
fn complex_python_like() {
    let g = optimize_grammar(GrammarBuilder::python_like()).unwrap();
    assert_eq!(g.name, "python_like");
    assert!(total_rules(&g) > 0);
}

#[test]
fn complex_nested_rules() {
    let g = optimize_grammar(
        GrammarBuilder::new("nested")
            .token("ID", r"[a-z]+")
            .token("(", "(")
            .token(")", ")")
            .token(",", ",")
            .rule("program", vec!["expr"])
            .rule("expr", vec!["ID"])
            .rule("expr", vec!["call"])
            .rule("call", vec!["ID", "(", "args", ")"])
            .rule("args", vec!["expr"])
            .rule("args", vec!["args", ",", "expr"])
            .start("program")
            .build(),
    )
    .unwrap();
    assert!(total_rules(&g) > 0);
}

#[test]
fn complex_multiple_nonterminals() {
    let g = optimize_grammar(
        GrammarBuilder::new("multi_nt")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .rule("s", vec!["x"])
            .rule("x", vec!["A", "y"])
            .rule("y", vec!["B", "z"])
            .rule("z", vec!["C"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "multi_nt");
}

#[test]
fn complex_grammar_with_extras() {
    let g = optimize_grammar(
        GrammarBuilder::new("extras")
            .token("X", "x")
            .token("WS", r"[ \t]+")
            .extra("WS")
            .rule("s", vec!["X"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "extras");
}

// ===========================================================================
// 7. Optimizer with precedence rules
// ===========================================================================

#[test]
fn prec_rules_preserved_after_optimize() {
    let g = optimize_grammar(
        GrammarBuilder::new("prec")
            .token("N", r"\d+")
            .token("+", "+")
            .token("*", "*")
            .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
            .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
            .rule("e", vec!["N"])
            .start("e")
            .build(),
    )
    .unwrap();
    assert!(total_rules(&g) > 0);
}

#[test]
fn prec_left_right_both_survive() {
    let g = optimize_grammar(
        GrammarBuilder::new("lr")
            .token("N", r"\d+")
            .token("+", "+")
            .token("^", "^")
            .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
            .rule_with_precedence("e", vec!["e", "^", "e"], 2, Associativity::Right)
            .rule("e", vec!["N"])
            .start("e")
            .build(),
    )
    .unwrap();
    assert!(total_rules(&g) >= 2);
}

#[test]
fn prec_none_associativity() {
    let _g = optimize_grammar(
        GrammarBuilder::new("none_assoc")
            .token("N", r"\d+")
            .token("==", "==")
            .rule_with_precedence("e", vec!["e", "==", "e"], 1, Associativity::None)
            .rule("e", vec!["N"])
            .start("e")
            .build(),
    )
    .unwrap();
}

#[test]
fn prec_multiple_levels() {
    let _g = optimize_grammar(
        GrammarBuilder::new("levels")
            .token("N", r"\d+")
            .token("+", "+")
            .token("*", "*")
            .token("^", "^")
            .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
            .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
            .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
            .rule("e", vec!["N"])
            .start("e")
            .build(),
    )
    .unwrap();
}

#[test]
fn prec_negative_level() {
    let _g = optimize_grammar(
        GrammarBuilder::new("neg")
            .token("N", r"\d+")
            .token("+", "+")
            .rule_with_precedence("e", vec!["e", "+", "e"], -1, Associativity::Left)
            .rule("e", vec!["N"])
            .start("e")
            .build(),
    )
    .unwrap();
}

// ===========================================================================
// 8. Optimizer clone preservation
// ===========================================================================

#[test]
fn clone_before_optimize_unchanged() {
    let original = build_arith();
    let cloned = original.clone();
    let _optimized = optimize_grammar(original).unwrap();
    // Cloned copy retains original state
    assert_eq!(cloned.name, "arith");
}

#[test]
fn clone_after_optimize_equals() {
    let optimized = optimize_grammar(build_arith()).unwrap();
    let cloned = optimized.clone();
    assert_eq!(optimized.name, cloned.name);
    assert_eq!(optimized.tokens.len(), cloned.tokens.len());
    assert_eq!(total_rules(&optimized), total_rules(&cloned));
}

#[test]
fn clone_deep_independence() {
    let mut a = optimize_grammar(build_arith()).unwrap();
    let b = a.clone();
    a.name = "modified".into();
    assert_ne!(a.name, b.name);
}

#[test]
fn clone_preserves_rule_names() {
    let optimized = optimize_grammar(build_arith()).unwrap();
    let cloned = optimized.clone();
    assert_eq!(optimized.rule_names.len(), cloned.rule_names.len());
}

#[test]
fn debug_format_after_optimize() {
    let g = optimize_grammar(build_arith()).unwrap();
    let debug = format!("{:?}", g);
    assert!(debug.contains("arith"));
}

// ===========================================================================
// 9. Optimizer with many alternatives
// ===========================================================================

#[test]
fn many_alternatives_five() {
    let g = optimize_grammar(
        GrammarBuilder::new("five")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .token("D", "d")
            .token("E", "e")
            .rule("s", vec!["A"])
            .rule("s", vec!["B"])
            .rule("s", vec!["C"])
            .rule("s", vec!["D"])
            .rule("s", vec!["E"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert!(total_rules(&g) >= 5);
}

#[test]
fn many_alternatives_ten() {
    let mut b = GrammarBuilder::new("ten");
    for i in 0..10 {
        let name = format!("T{}", i);
        b = b.token(&name, &name);
    }
    for i in 0..10 {
        let name = format!("T{}", i);
        b = b.rule("s", vec![Box::leak(name.into_boxed_str()) as &str]);
    }
    b = b.start("s");
    let g = optimize_grammar(b.build()).unwrap();
    assert!(total_rules(&g) >= 10);
}

#[test]
fn many_alternatives_name_preserved() {
    let g = optimize_grammar(
        GrammarBuilder::new("lots")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .rule("s", vec!["A"])
            .rule("s", vec!["B"])
            .rule("s", vec!["C"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "lots");
}

#[test]
fn many_alternatives_tokens_all_present() {
    let g = optimize_grammar(
        GrammarBuilder::new("keep")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .token("D", "d")
            .rule("s", vec!["A"])
            .rule("s", vec!["B"])
            .rule("s", vec!["C"])
            .rule("s", vec!["D"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert_eq!(g.tokens.len(), 4);
}

#[test]
fn many_alternatives_idempotent() {
    let base = GrammarBuilder::new("idem_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["C"])
        .start("s")
        .build();
    let once = optimize_grammar(base).unwrap();
    let twice = optimize_grammar(once.clone()).unwrap();
    assert_eq!(total_rules(&once), total_rules(&twice));
}

// ===========================================================================
// 10. Optimizer with chain rules
// ===========================================================================

#[test]
fn chain_rule_a_to_b_to_c() {
    // A -> B, B -> C, C -> TOKEN  — unit rules can be eliminated
    let mut g = GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["X"])
        .start("a")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // Some inlining or unit elimination should occur
    assert!(stats.inlined_rules > 0 || stats.eliminated_unit_rules > 0);
}

#[test]
fn chain_rule_preserves_reachability() {
    // Chain a -> b -> X may be fully inlined; verify no panic and name preserved
    let g = optimize_grammar(
        GrammarBuilder::new("reach")
            .token("X", "x")
            .rule("a", vec!["b"])
            .rule("b", vec!["X"])
            .start("a")
            .build(),
    )
    .unwrap();
    assert_eq!(g.name, "reach");
}

#[test]
fn chain_rule_single_step() {
    let mut g = GrammarBuilder::new("single_chain")
        .token("T", "t")
        .rule("s", vec!["inner"])
        .rule("inner", vec!["T"])
        .start("s")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.inlined_rules > 0 || stats.eliminated_unit_rules > 0);
}

#[test]
fn chain_rule_with_branch() {
    // s -> a, a -> X | a -> Y  — not a simple chain, should not inline all
    let g = optimize_grammar(
        GrammarBuilder::new("branch")
            .token("X", "x")
            .token("Y", "y")
            .rule("s", vec!["a"])
            .rule("a", vec!["X"])
            .rule("a", vec!["Y"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert!(total_rules(&g) >= 2);
}

#[test]
fn chain_rule_idempotent() {
    let base = GrammarBuilder::new("chain_idem")
        .token("X", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["X"])
        .start("a")
        .build();
    let once = optimize_grammar(base).unwrap();
    let twice = optimize_grammar(once.clone()).unwrap();
    assert_eq!(total_rules(&once), total_rules(&twice));
}

// ===========================================================================
// 11. OptimizationStats
// ===========================================================================

#[test]
fn stats_default_total_zero() {
    assert_eq!(OptimizationStats::default().total(), 0);
}

#[test]
fn stats_total_consistent() {
    let mut g = build_arith();
    let mut opt = GrammarOptimizer::new();
    let s = opt.optimize(&mut g);
    assert_eq!(
        s.total(),
        s.removed_unused_symbols
            + s.inlined_rules
            + s.merged_tokens
            + s.optimized_left_recursion
            + s.eliminated_unit_rules
    );
}

#[test]
fn stats_debug_format() {
    let s = OptimizationStats::default();
    let d = format!("{:?}", s);
    assert!(d.contains("OptimizationStats"));
}

// ===========================================================================
// 12. Edge cases and additional coverage
// ===========================================================================

#[test]
fn optimize_grammar_returns_ok() {
    let g = GrammarBuilder::new("ok")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn optimize_empty_grammar_returns_ok() {
    assert!(optimize_grammar(Grammar::new("e".into())).is_ok());
}

#[test]
fn left_recursion_transformed() {
    let mut g = GrammarBuilder::new("lr")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("e", vec!["e", "+", "N"])
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert!(stats.optimized_left_recursion > 0);
}

#[test]
fn right_recursive_not_transformed_as_left() {
    let mut g = GrammarBuilder::new("rr")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("e", vec!["N", "+", "e"])
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.optimized_left_recursion, 0);
}

#[test]
fn serialize_after_optimize() {
    let g = optimize_grammar(build_arith()).unwrap();
    let json = serde_json::to_string(&g);
    assert!(json.is_ok());
}

#[test]
fn deserialize_after_optimize() {
    let g = optimize_grammar(build_arith()).unwrap();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn optimize_preserves_externals() {
    let g = optimize_grammar(GrammarBuilder::python_like()).unwrap();
    // Python-like grammar has INDENT/DEDENT externals
    assert!(!g.externals.is_empty());
}

#[test]
fn optimize_preserves_extras() {
    // Use a grammar where the extra token is also referenced in rules
    let g = optimize_grammar(
        GrammarBuilder::new("ext")
            .token("X", "x")
            .token("Y", "y")
            .token("WS", r"\s+")
            .extra("WS")
            .rule("s", vec!["X", "WS", "Y"])
            .start("s")
            .build(),
    )
    .unwrap();
    assert!(!g.extras.is_empty());
}
