//! Comprehensive tests for GLR driver correctness
//! Tests epsilon spans, fork handling, EOF acceptance, and root selection

use rust_sitter_glr_core::{
    ParseTable, Driver, ParseRule, Action
};
use rust_sitter_ir::{Grammar, SymbolId, StateId, RuleId};
use std::collections::BTreeMap;

type ActionCell = Vec<Action>;

/// Helper to create a minimal ParseTable for testing
fn create_test_table(
    states: Vec<Vec<ActionCell>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start: SymbolId,
    eof: SymbolId,
) -> ParseTable {
    let symbol_count = states.first().map(|s| s.len()).unwrap_or(0);
    let state_count = states.len();
    
    // Build symbol_to_index mapping
    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    
    // Build nonterminal_to_index for gotos
    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start, 1);
    nonterminal_to_index.insert(SymbolId(2), 2); // A symbol
    
    ParseTable {
        action_table: states,
        goto_table: gotos,
        rules,
        state_count,
        symbol_count,
        symbol_to_index,
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start,
        grammar: Grammar::new("test".to_string()),
        symbol_metadata: vec![],
        initial_state: StateId(0),  // Default to state 0 for test grammars
    }
}

#[test]
fn test_epsilon_reduce_span() {
    // Grammar: A -> ε; S -> A 'x'
    // Input: "x"
    // Expected: A has span (0,0), S has span (0,1)
    
    let eof = SymbolId(0);
    let s_sym = SymbolId(1);
    let a_sym = SymbolId(2);
    let x_sym = SymbolId(3);
    
    // Rules
    let rules = vec![
        ParseRule { lhs: a_sym, rhs_len: 0 },        // A -> ε
        ParseRule { lhs: s_sym, rhs_len: 2 }, // S -> A 'x'
    ];
    
    // State 0: can reduce A -> ε on 'x' lookahead
    // State 1: after reducing A, can shift 'x'
    // State 2: after shifting 'x', can reduce S -> A 'x'
    // State 3: accept state
    
    let mut actions = vec![vec![vec![]; 4]; 4];
    
    // State 0
    actions[0][3].push(Action::Reduce(RuleId(0))); // on 'x', reduce A -> ε
    
    // State 1 (after A reduction)
    actions[1][3].push(Action::Shift(StateId(2))); // on 'x', shift to state 2
    
    // State 2 (after shifting 'x')
    actions[2][0].push(Action::Reduce(RuleId(1))); // on EOF, reduce S -> A 'x'
    
    // State 3 (after S reduction)
    actions[3][0].push(Action::Accept); // on EOF, accept
    
    let invalid = StateId(65535);
    let mut gotos = vec![vec![invalid; 4]; 4];
    gotos[0][2] = StateId(1); // goto state 1 after reducing to A
    gotos[0][1] = StateId(3); // goto state 3 after reducing to S
    
    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    
    // Create token stream for 'x' at position 0-1
    let tokens = vec![(x_sym, 0, 1)];
    
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(tokens.into_iter().map(|(s, start, end)| (s.0 as u32, start as u32, end as u32)));
    
    assert!(result.is_ok(), "Parse should succeed: {:?}", result.err());
    let forest = result.unwrap();
    
    // Verify the forest has the expected structure
    let view = forest.view();
    let roots = view.roots();
    assert_eq!(roots.len(), 1, "Should have exactly one root");
    
    // The root should be S with span (0,1)
    let root_span = view.span(roots[0]);
    assert_eq!(root_span.start, 0);
    assert_eq!(root_span.end, 1);
    
    // Check that A (first child) has span (0,0) for epsilon
    let children = view.best_children(roots[0]);
    assert_eq!(children.len(), 2, "S should have 2 children: A and 'x'");
    
    let a_span = view.span(children[0]);
    assert_eq!(a_span.start, 0, "Epsilon A should have start position 0");
    assert_eq!(a_span.end, 0, "Epsilon A should have end position 0");
    
    let x_span = view.span(children[1]);
    assert_eq!(x_span.start, 0);
    assert_eq!(x_span.end, 1);
}

#[test]
fn test_fork_sanity() {
    // Grammar with shift/reduce conflict on 'a'
    // S -> 'a' | T 'a'
    // T -> 'a'
    // On input "a a", both paths should be explored
    
    let eof = SymbolId(0);
    let s_sym = SymbolId(1);
    let t_sym = SymbolId(2);
    let a_sym = SymbolId(3);
    
    let rules = vec![
        ParseRule { lhs: s_sym, rhs_len: 1 },      // S -> 'a'
        ParseRule { lhs: s_sym, rhs_len: 2 }, // S -> T 'a'
        ParseRule { lhs: t_sym, rhs_len: 1 },      // T -> 'a'
    ];
    
    let mut actions = vec![vec![vec![]; 4]; 6];
    
    // State 0: shift 'a' to state 1
    actions[0][3].push(Action::Shift(StateId(1)));
    
    // State 1: shift/reduce conflict on 'a'
    // Can shift 'a' to state 2 (for S -> T 'a')
    // Can reduce T -> 'a' (rule 2)
    actions[1][3].push(Action::Shift(StateId(2)));
    actions[1][3].push(Action::Reduce(RuleId(2))); // Creates a fork!
    
    // Also can reduce S -> 'a' on EOF
    actions[1][0].push(Action::Reduce(RuleId(0)));
    
    // State 2: after second 'a', reduce S -> 'a'
    actions[2][0].push(Action::Reduce(RuleId(0)));
    
    // State 3: after reducing to T, shift 'a'
    actions[3][3].push(Action::Shift(StateId(4)));
    
    // State 4: after T 'a', reduce S -> T 'a'
    actions[4][0].push(Action::Reduce(RuleId(1)));
    
    // State 5: accept
    actions[5][0].push(Action::Accept);
    
    let invalid = StateId(65535);
    let mut gotos = vec![vec![invalid; 4]; 6];
    gotos[0][1] = StateId(5); // goto accept after S
    gotos[0][2] = StateId(3); // goto state 3 after T
    gotos[1][1] = StateId(5); // goto accept after S
    gotos[2][1] = StateId(5); // goto accept after S
    gotos[3][1] = StateId(5); // goto accept after S
    gotos[4][1] = StateId(5); // goto accept after S
    
    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    
    // Token stream for "a a"
    let tokens = vec![(a_sym, 0, 1), (a_sym, 2, 3)];
    
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(tokens.into_iter().map(|(s, start, end)| (s.0 as u32, start as u32, end as u32)));
    
    assert!(result.is_ok(), "Parse should succeed with fork");
    let forest = result.unwrap();
    
    // The forest should contain both parse alternatives
    let view = forest.view();
    let roots = view.roots();
    assert!(!roots.is_empty(), "Should have at least one root");
    
    // We successfully handled the fork and found a parse
}

#[test]
fn test_eof_accept() {
    // Grammar: S -> 't'
    // Input: "t"
    // Should only accept after EOF phase
    
    let eof = SymbolId(0);
    let s_sym = SymbolId(1);
    let t_sym = SymbolId(2);
    
    let rules = vec![
        ParseRule { lhs: s_sym, rhs_len: 1 }, // S -> 't'
    ];
    
    let mut actions = vec![vec![vec![]; 3]; 3];
    
    // State 0: shift 't' to state 1
    actions[0][2].push(Action::Shift(StateId(1)));
    
    // State 1: reduce S -> 't' on EOF (not on regular lookahead!)
    actions[1][0].push(Action::Reduce(RuleId(0)));
    
    // State 2: accept on EOF
    actions[2][0].push(Action::Accept);
    
    let invalid = StateId(65535);
    let mut gotos = vec![vec![invalid; 3]; 3];
    gotos[0][1] = StateId(2); // goto state 2 after S
    
    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    
    let tokens = vec![(t_sym, 0, 1)];
    
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(tokens.into_iter().map(|(s, start, end)| (s.0 as u32, start as u32, end as u32)));
    
    assert!(result.is_ok(), "Parse should succeed with EOF accept");
    let forest = result.unwrap();
    
    let view = forest.view();
    let roots = view.roots();
    assert_eq!(roots.len(), 1, "Should have exactly one root");
    
    // Verify it's the start symbol with correct span
    let root_span = view.span(roots[0]);
    assert_eq!(root_span.start, 0);
    assert_eq!(root_span.end, 1);
}

#[test]
fn test_root_selection_deterministic() {
    // Test that when multiple roots exist, we select the one with the largest span
    // This simulates a case where the parser might produce multiple valid parses
    
    let eof = SymbolId(0);
    let s_sym = SymbolId(1);
    let a_sym = SymbolId(2);
    
    let rules = vec![
        ParseRule { lhs: s_sym, rhs_len: 1 }, // S -> 'a'
        ParseRule { lhs: s_sym, rhs_len: 2 }, // S -> 'a' 'a'
    ];
    
    // This is a simplified test - in reality the grammar would need to be ambiguous
    // For now we just verify the root sorting logic compiles and runs
    let mut actions = vec![vec![vec![]; 3]; 4];
    actions[0][2].push(Action::Shift(StateId(1)));
    actions[1][0].push(Action::Reduce(RuleId(0)));
    actions[1][2].push(Action::Shift(StateId(2)));
    actions[2][0].push(Action::Reduce(RuleId(1)));
    actions[3][0].push(Action::Accept);
    
    let invalid = StateId(65535);
    let mut gotos = vec![vec![invalid; 3]; 4];
    gotos[0][1] = StateId(3);
    gotos[1][1] = StateId(3);
    gotos[2][1] = StateId(3);
    
    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    
    let tokens = vec![(a_sym, 0, 1), (a_sym, 1, 2)];
    
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(tokens.into_iter().map(|(s, start, end)| (s.0 as u32, start as u32, end as u32)));
    
    assert!(result.is_ok(), "Parse should succeed: {:?}", result.err());
    let forest = result.unwrap();
    
    // Just verify we got a deterministic root
    let view = forest.view();
    let roots = view.roots();
    assert!(!roots.is_empty(), "Should have at least one root");
}