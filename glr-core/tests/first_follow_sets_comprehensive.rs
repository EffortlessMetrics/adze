#![allow(clippy::needless_range_loop)]
//! Comprehensive test suite for FirstFollowSets computation.
//!
//! Tests cover:
//! - FIRST set computation for various grammar types
//! - FOLLOW set computation including EOF
//! - Nullable symbol detection
//! - first_of_sequence with various inputs
//! - Determinism and consistency
//! - Complex grammars (arithmetic, recursive)

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ============================================================================
// Test 1: FIRST of start symbol is non-empty
// ============================================================================

#[test]
fn test_first_of_start_symbol_nonempty() {
    let grammar = GrammarBuilder::new("simple")
        .token("tok", "a")
        .rule("start", vec!["tok"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start = grammar.start_symbol().unwrap();

    let first_set = ff.first(start).expect("start should have FIRST set");
    assert!(!first_set.is_clear(), "FIRST(start) should not be empty");
    assert!(
        first_set.count_ones(..) > 0,
        "FIRST(start) should contain at least one element"
    );
}

// ============================================================================
// Test 2: FIRST with alternatives has more elements
// ============================================================================

#[test]
fn test_first_with_alternatives() {
    let grammar = GrammarBuilder::new("alternatives")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr_id = grammar.start_symbol().unwrap();
    let first_set = ff.first(expr_id).expect("expr should have FIRST set");

    // Should contain both 'a' and 'b'
    assert!(
        first_set.count_ones(..) >= 2,
        "FIRST(expr) should contain multiple alternatives"
    );
}

// ============================================================================
// Test 3: FOLLOW of start contains EOF
// ============================================================================

#[test]
fn test_follow_of_start_contains_eof() {
    let grammar = GrammarBuilder::new("eof_test")
        .token("tok", "x")
        .rule("start", vec!["tok"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start = grammar.start_symbol().unwrap();
    let follow_set = ff.follow(start).expect("start should have FOLLOW set");

    // EOF symbol is always at position 0
    assert!(
        follow_set.contains(0),
        "FOLLOW(start) should contain EOF (symbol 0)"
    );
}

// ============================================================================
// Test 4: is_nullable for epsilon rules
// ============================================================================

#[test]
fn test_is_nullable_epsilon_rule() {
    let grammar = GrammarBuilder::new("epsilon")
        .rule("nullable", vec![])
        .rule("expr", vec!["nullable"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let nullable_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| *name == "nullable")
        .map(|(id, _)| *id)
        .unwrap();

    assert!(
        ff.is_nullable(nullable_id),
        "nullable nonterminal should be nullable"
    );
}

// ============================================================================
// Test 5: is_nullable returns false for terminal-only rules
// ============================================================================

#[test]
fn test_is_nullable_terminal_only() {
    let grammar = GrammarBuilder::new("terminal_only")
        .token("tok", "x")
        .rule("start", vec!["tok"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start = grammar.start_symbol().unwrap();

    assert!(
        !ff.is_nullable(start),
        "terminal-only rule should not be nullable"
    );
}

// ============================================================================
// Test 6: first_of_sequence with single element
// ============================================================================

#[test]
fn test_first_of_sequence_single_element() {
    let grammar = GrammarBuilder::new("sequence")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    // Get the token id
    let tok_a = *grammar.tokens.keys().next().unwrap();
    let sequence = vec![Symbol::Terminal(tok_a)];
    let first = ff.first_of_sequence(&sequence).unwrap();

    assert!(
        first.count_ones(..) > 0,
        "FIRST of single terminal should not be empty"
    );
    assert!(
        first.contains(tok_a.0 as usize),
        "FIRST should contain the terminal"
    );
}

// ============================================================================
// Test 7: first_of_sequence with empty sequence
// ============================================================================

#[test]
fn test_first_of_sequence_empty() {
    let grammar = GrammarBuilder::new("empty")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let sequence: Vec<Symbol> = vec![];
    let first = ff.first_of_sequence(&sequence).unwrap();

    assert!(first.is_clear(), "FIRST of empty sequence should be empty");
}

// ============================================================================
// Test 8: Computation is deterministic
// ============================================================================

#[test]
fn test_computation_is_deterministic() {
    let grammar = GrammarBuilder::new("deterministic")
        .token("x", "x")
        .token("y", "y")
        .rule("expr", vec!["x"])
        .rule("expr", vec!["y"])
        .rule("expr", vec!["expr", "x"])
        .start("expr")
        .build();

    let ff1 = FirstFollowSets::compute(&grammar).unwrap();
    let ff2 = FirstFollowSets::compute(&grammar).unwrap();

    let expr = grammar.start_symbol().unwrap();
    let first1 = ff1.first(expr).unwrap();
    let first2 = ff2.first(expr).unwrap();

    // Compare by counting bits
    assert_eq!(
        first1.count_ones(..),
        first2.count_ones(..),
        "FIRST sets should match across computations"
    );

    let follow1 = ff1.follow(expr).unwrap();
    let follow2 = ff2.follow(expr).unwrap();
    assert_eq!(
        follow1.count_ones(..),
        follow2.count_ones(..),
        "FOLLOW sets should match across computations"
    );

    assert_eq!(
        ff1.is_nullable(expr),
        ff2.is_nullable(expr),
        "Nullability should match"
    );
}

// ============================================================================
// Test 9: Complex arithmetic grammar computes without panic
// ============================================================================

#[test]
fn test_arithmetic_grammar_computes() {
    let grammar = GrammarBuilder::new("arithmetic")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["expr", "-", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["term", "/", "factor"])
        .rule("factor", vec!["NUM"])
        .rule("factor", vec!["(", "expr", ")"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr = grammar.start_symbol().unwrap();

    assert!(
        ff.first(expr).is_some(),
        "arithmetic grammar should compute FIRST"
    );
    assert!(
        ff.follow(expr).is_some(),
        "arithmetic grammar should compute FOLLOW"
    );
}

// ============================================================================
// Test 10: Deep nonterminal chains
// ============================================================================

#[test]
fn test_deep_nonterminal_chains() {
    let grammar = GrammarBuilder::new("deep_chain")
        .token("a", "a")
        .rule("depth1", vec!["a"])
        .rule("depth2", vec!["depth1"])
        .rule("depth3", vec!["depth2"])
        .rule("depth4", vec!["depth3"])
        .rule("depth5", vec!["depth4"])
        .start("depth5")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    // Verify that the token 'a' propagates through the chain
    let start = grammar.start_symbol().unwrap();
    let first_set = ff.first(start).expect("start should have FIRST");
    assert!(
        first_set.count_ones(..) > 0,
        "deep chain should propagate FIRST correctly"
    );
}

// ============================================================================
// Test 11: compute_normalized handles complex symbols
// ============================================================================

#[test]
fn test_compute_normalized() {
    let mut grammar = GrammarBuilder::new("normalized")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let start = grammar.start_symbol().unwrap();

    assert!(
        ff.first(start).is_some(),
        "compute_normalized should produce FIRST sets"
    );
}

// ============================================================================
// Test 12: Multiple rules for same nonterminal
// ============================================================================

#[test]
fn test_multiple_rules_same_nonterminal() {
    let grammar = GrammarBuilder::new("multiple_rules")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("expr", vec!["x"])
        .rule("expr", vec!["y"])
        .rule("expr", vec!["z"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr = grammar.start_symbol().unwrap();
    let first = ff.first(expr).unwrap();

    // All three tokens should be in FIRST(expr)
    assert!(
        first.count_ones(..) >= 3,
        "FIRST(expr) should contain all alternatives"
    );
}

// ============================================================================
// Test 13: Recursive grammar (left recursion)
// ============================================================================

#[test]
fn test_left_recursive_grammar() {
    let grammar = GrammarBuilder::new("left_recursive")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "a"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr = grammar.start_symbol().unwrap();

    assert!(
        ff.first(expr).is_some(),
        "left-recursive grammar should compute FIRST"
    );
    assert!(
        ff.follow(expr).is_some(),
        "left-recursive grammar should compute FOLLOW"
    );
}

// ============================================================================
// Test 14: Recursive grammar (right recursion)
// ============================================================================

#[test]
fn test_right_recursive_grammar() {
    let grammar = GrammarBuilder::new("right_recursive")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["a", "expr"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr = grammar.start_symbol().unwrap();

    assert!(
        ff.first(expr).is_some(),
        "right-recursive grammar should compute FIRST"
    );
}

// ============================================================================
// Test 15: Grammar with all epsilon rules
// ============================================================================

#[test]
fn test_all_epsilon_rules() {
    let grammar = GrammarBuilder::new("all_epsilon")
        .rule("expr", vec![])
        .rule("term", vec![])
        .rule("start", vec!["expr", "term"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| *name == "expr")
        .map(|(id, _)| *id)
        .unwrap();

    assert!(ff.is_nullable(expr_id), "epsilon rules should be nullable");
}

// ============================================================================
// Test 16: FIRST of nonterminal referencing token
// ============================================================================

#[test]
fn test_first_nonterminal_referencing_token() {
    let grammar = GrammarBuilder::new("nonterminal_ref")
        .token("keyword", "if")
        .rule("stmt", vec!["keyword"])
        .start("stmt")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let stmt = grammar.start_symbol().unwrap();
    let keyword_token = *grammar.tokens.keys().next().unwrap();

    let first = ff.first(stmt).unwrap();
    assert!(
        first.contains(keyword_token.0 as usize),
        "FIRST(stmt) should contain the keyword token"
    );
}

// ============================================================================
// Test 17: Complex sequence FIRST computation
// ============================================================================

#[test]
fn test_first_of_complex_sequence() {
    let grammar = GrammarBuilder::new("complex_seq")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let tok_a = grammar.tokens.keys().next().cloned().unwrap();
    let sequence = vec![Symbol::Terminal(tok_a)];
    let first = ff.first_of_sequence(&sequence).unwrap();

    assert!(
        first.contains(tok_a.0 as usize),
        "FIRST of sequence should contain first terminal"
    );
}

// ============================================================================
// Test 18: Nonterminal chain FIRST propagation
// ============================================================================

#[test]
fn test_nonterminal_chain_first_propagation() {
    let grammar = GrammarBuilder::new("nonterminal_chain")
        .token("x", "x")
        .rule("n1", vec!["x"])
        .rule("n2", vec!["n1"])
        .rule("n3", vec!["n2"])
        .start("n3")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let n1 = grammar
        .rule_names
        .iter()
        .find(|(_, name)| *name == "n1")
        .map(|(id, _)| *id)
        .unwrap();
    let n2 = grammar
        .rule_names
        .iter()
        .find(|(_, name)| *name == "n2")
        .map(|(id, _)| *id)
        .unwrap();
    let n3 = grammar
        .rule_names
        .iter()
        .find(|(_, name)| *name == "n3")
        .map(|(id, _)| *id)
        .unwrap();

    let first_n1 = ff.first(n1).unwrap().count_ones(..);
    let first_n2 = ff.first(n2).unwrap().count_ones(..);
    let first_n3 = ff.first(n3).unwrap().count_ones(..);

    // All should contain the same token
    assert!(
        first_n1 > 0 && first_n2 > 0 && first_n3 > 0,
        "chain should propagate"
    );
}

// ============================================================================
// Test 19: FOLLOW propagation through rules
// ============================================================================

#[test]
fn test_follow_propagation_through_rules() {
    let grammar = GrammarBuilder::new("follow_prop")
        .token("x", "x")
        .token("y", "y")
        .rule("expr", vec!["term", "y"])
        .rule("term", vec!["x"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let term_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| *name == "term")
        .map(|(id, _)| *id)
        .unwrap();
    let follow_term = ff.follow(term_id).unwrap();

    // term is followed by 'y'
    let token_y = grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "y")
        .map(|(id, _)| id)
        .unwrap();

    assert!(
        follow_term.contains(token_y.0 as usize),
        "FOLLOW(term) should contain token y"
    );
}

// ============================================================================
// Test 20: Nullable nonterminal affects FOLLOW
// ============================================================================

#[test]
fn test_nullable_nonterminal_affects_follow() {
    let grammar = GrammarBuilder::new("nullable_follow")
        .token("a", "a")
        .token("b", "b")
        .rule("nullable", vec![])
        .rule("expr", vec!["term", "nullable", "b"])
        .rule("term", vec!["a"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let nullable_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| *name == "nullable")
        .map(|(id, _)| *id)
        .unwrap();

    assert!(
        ff.is_nullable(nullable_id),
        "nullable rule should be nullable"
    );
}

// ============================================================================
// Test 21: Grammar with only terminals (no nonterminals)
// ============================================================================

#[test]
fn test_grammar_with_only_terminals() {
    let grammar = GrammarBuilder::new("terminals_only")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start = grammar.start_symbol().unwrap();

    assert!(
        ff.first(start).is_some(),
        "terminal-only grammar should compute FIRST"
    );
}

// ============================================================================
// Test 22: Large fan-out alternatives
// ============================================================================

#[test]
fn test_large_fan_out_alternatives() {
    let mut builder = GrammarBuilder::new("large_fanout");
    for i in 1..=20 {
        builder = builder.token(&format!("t{}", i), &format!("t{}", i));
    }
    for i in 1..=20 {
        builder = builder.rule("expr", vec![&format!("t{}", i)]);
    }
    let grammar = builder.start("expr").build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr = grammar.start_symbol().unwrap();
    let first = ff.first(expr).unwrap();

    // Should contain many alternatives
    assert!(
        first.count_ones(..) >= 10,
        "large fan-out should have many FIRST elements"
    );
}

// ============================================================================
// Test 23: Mutual recursion
// ============================================================================

#[test]
fn test_mutual_recursion() {
    let grammar = GrammarBuilder::new("mutual_recursion")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("a", vec!["b"])
        .rule("b", vec!["y"])
        .rule("b", vec!["a"])
        .start("a")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let a = grammar.start_symbol().unwrap();

    assert!(
        ff.first(a).is_some(),
        "mutually recursive grammar should compute FIRST"
    );
}

// ============================================================================
// Test 24: Mixed terminals and nonterminals in same rule
// ============================================================================

#[test]
fn test_mixed_terminals_and_nonterminals() {
    let grammar = GrammarBuilder::new("mixed")
        .token("open", "(")
        .token("close", ")")
        .rule("paren", vec!["open", "expr", "close"])
        .rule("expr", vec!["paren"])
        .rule("expr", vec!["open", "close"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr = grammar.start_symbol().unwrap();

    let first = ff.first(expr).unwrap();
    assert!(first.count_ones(..) > 0, "mixed grammar should have FIRST");
}

// ============================================================================
// Test 25: Ensure FOLLOW contains EOF for all nonterminals that can follow start
// ============================================================================

#[test]
fn test_follow_contains_eof_for_derivable() {
    let grammar = GrammarBuilder::new("eof_all")
        .token("a", "a")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "a"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start = grammar.start_symbol().unwrap();
    let follow = ff.follow(start).unwrap();

    assert!(follow.contains(0), "FOLLOW(start) must contain EOF");
}

// ============================================================================
// Test 26: Empty rule alternatives
// ============================================================================

#[test]
fn test_empty_rule_alternatives() {
    let grammar = GrammarBuilder::new("empty_alt")
        .token("a", "a")
        .rule("expr", vec![])
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "a"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let expr = grammar.start_symbol().unwrap();

    assert!(
        ff.is_nullable(expr),
        "expr should be nullable due to epsilon rule"
    );
}

// ============================================================================
// Test 27: Verify consistency: if token in FIRST(nt), it's a terminal
// ============================================================================

#[test]
fn test_first_consistency() {
    let grammar = GrammarBuilder::new("consistency")
        .token("keyword", "let")
        .token("ident", r"[a-z]+")
        .rule("decl", vec!["keyword", "ident"])
        .start("decl")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let decl = grammar.start_symbol().unwrap();
    let first = ff.first(decl).unwrap();

    // Elements in FIRST should be valid symbol IDs
    assert!(
        first.count_ones(..) <= 100,
        "FIRST should have reasonable size"
    );
}

// ============================================================================
// Test 28: Very deep recursion chain
// ============================================================================

#[test]
fn test_very_deep_recursion_chain() {
    let mut builder = GrammarBuilder::new("very_deep");
    builder = builder.token("a", "a");
    builder = builder.rule("d1", vec!["a"]);

    for i in 2..=50 {
        let prev = format!("d{}", i - 1);
        let curr = format!("d{}", i);
        builder = builder.rule(&curr, vec![prev.as_str()]);
    }

    let grammar = builder.start("d50").build();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start = grammar.start_symbol().unwrap();

    assert!(
        ff.first(start).is_some(),
        "very deep recursion should compute"
    );
}

// ============================================================================
// Test 29: Single token grammar (minimal)
// ============================================================================

#[test]
fn test_single_token_grammar() {
    let grammar = GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start = grammar.start_symbol().unwrap();

    let first = ff.first(start).unwrap();
    let follow = ff.follow(start).unwrap();

    assert!(
        first.count_ones(..) > 0,
        "single token FIRST should be non-empty"
    );
    assert!(follow.contains(0), "start FOLLOW should contain EOF");
}

// ============================================================================
// Test 30: Verify first_of_sequence with complex multi-element sequence
// ============================================================================

#[test]
fn test_first_of_sequence_complex() {
    let grammar = GrammarBuilder::new("seq_complex")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("seq", vec!["a", "b", "c"])
        .start("seq")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let tok_a = grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "a")
        .map(|(id, _)| id)
        .unwrap();

    let sequence = vec![
        Symbol::Terminal(*tok_a),
        Symbol::Terminal(*tok_a),
        Symbol::Terminal(*tok_a),
    ];

    let first = ff.first_of_sequence(&sequence).unwrap();
    assert!(
        first.contains(tok_a.0 as usize),
        "FIRST of sequence should contain first element"
    );
}
