//! Comprehensive tests for FirstFollowSets via adze-glr-core integration.

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, Symbol};

// ============================================================================
// Helpers
// ============================================================================

fn build_ff(grammar: &mut Grammar) -> FirstFollowSets {
    FirstFollowSets::compute_normalized(grammar).expect("compute failed")
}

// ============================================================================
// Tests: FIRST sets are non-empty for nonterminals with terminals
// ============================================================================

#[test]
fn first_of_start_nonempty_single_token() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    let start_id = g.start_symbol().unwrap();
    let first = ff.first(start_id).expect("FIRST(start) should exist");
    assert!(!first.is_clear(), "FIRST(start) should be non-empty");
}

#[test]
fn first_of_start_nonempty_two_tokens() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    let start_id = g.start_symbol().unwrap();
    let first = ff.first(start_id).unwrap();
    assert!(
        first.count_ones(..) >= 2,
        "FIRST with 2 alternatives should have >= 2"
    );
}

#[test]
fn first_of_start_three_alternatives() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    let start_id = g.start_symbol().unwrap();
    assert!(ff.first(start_id).unwrap().count_ones(..) >= 3);
}

#[test]
fn first_of_start_sequence() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    let start_id = g.start_symbol().unwrap();
    assert!(!ff.first(start_id).unwrap().is_clear());
}

// ============================================================================
// Tests: Nullable
// ============================================================================

#[test]
fn start_nullable_with_epsilon() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    assert!(ff.is_nullable(g.start_symbol().unwrap()));
}

#[test]
fn start_not_nullable_with_terminal() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    assert!(!ff.is_nullable(g.start_symbol().unwrap()));
}

// ============================================================================
// Tests: FOLLOW sets
// ============================================================================

#[test]
fn follow_of_start_contains_eof() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    let start_id = g.start_symbol().unwrap();
    let follow = ff.follow(start_id).expect("FOLLOW(start) should exist");
    assert!(!follow.is_clear(), "FOLLOW(start) should contain EOF");
}

// ============================================================================
// Tests: first_of_sequence
// ============================================================================

#[test]
fn first_of_empty_sequence() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    let result = ff.first_of_sequence(&[]);
    assert!(result.is_ok());
}

#[test]
fn first_of_single_terminal_sequence() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    // Find the token SymbolId for "a"
    let a_id = g.tokens.keys().next().copied().unwrap();
    let seq = vec![Symbol::Terminal(a_id)];
    let result = ff.first_of_sequence(&seq).unwrap();
    assert!(!result.is_clear());
}

// ============================================================================
// Tests: Scaling
// ============================================================================

#[test]
fn many_alternatives_first_count() {
    let mut builder = GrammarBuilder::new("t");
    for i in 0..10 {
        let name = format!("tok{}", i);
        builder = builder.token(&name, &name);
        builder = builder.rule("start", vec![&name]);
    }
    let mut g = builder.start("start").build();
    let ff = build_ff(&mut g);
    let start_id = g.start_symbol().unwrap();
    let count = ff.first(start_id).unwrap().count_ones(..);
    assert!(count >= 10, "expected >= 10, got {}", count);
}

#[test]
fn alternative_first_larger_than_single() {
    let mut g1 = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff1 = build_ff(&mut g1);
    let c1 = ff1
        .first(g1.start_symbol().unwrap())
        .unwrap()
        .count_ones(..);

    let mut g2 = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff2 = build_ff(&mut g2);
    let c2 = ff2
        .first(g2.start_symbol().unwrap())
        .unwrap()
        .count_ones(..);

    assert!(c2 >= c1, "2 alternatives >= 1 alternative");
}

// ============================================================================
// Tests: Determinism
// ============================================================================

#[test]
fn compute_deterministic() {
    let mk = || {
        let mut g = GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build();
        let ff = build_ff(&mut g);
        ff.first(g.start_symbol().unwrap()).unwrap().count_ones(..)
    };
    assert_eq!(mk(), mk());
}

// ============================================================================
// Tests: Complex grammars compute without panic
// ============================================================================

#[test]
fn arithmetic_grammar_computes() {
    let mut g = GrammarBuilder::new("t")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    assert!(ff.first(g.start_symbol().unwrap()).is_some());
}

#[test]
fn nested_nonterminal_chain() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("d", vec!["x"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    assert!(!ff.first(g.start_symbol().unwrap()).unwrap().is_clear());
}

#[test]
fn recursive_grammar_terminates() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    assert!(ff.first(g.start_symbol().unwrap()).is_some());
}

#[test]
fn left_recursive_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("op", "+")
        .rule("expr", vec!["expr", "op", "a"])
        .rule("expr", vec!["a"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    assert!(ff.first(g.start_symbol().unwrap()).is_some());
}

#[test]
fn right_recursive_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("op", "+")
        .rule("expr", vec!["a", "op", "expr"])
        .rule("expr", vec!["a"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    assert!(ff.first(g.start_symbol().unwrap()).is_some());
}

#[test]
fn nullable_in_sequence() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .rule("start", vec!["opt", "b"])
        .start("start")
        .build();
    let ff = build_ff(&mut g);
    let start_id = g.start_symbol().unwrap();
    let first = ff.first(start_id).unwrap();
    // FIRST(start) should include both 'a' (from opt) and 'b' (opt nullable)
    assert!(first.count_ones(..) >= 2);
}
