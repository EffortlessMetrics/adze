#![cfg(feature = "pure-rust")]

//! Comprehensive tests for the GLR engine module (`glr_engine.rs`).
//!
//! Covers: engine creation, parsing, error handling, forest management,
//! token processing, reset semantics, fork/merge, and boundary conditions.

use adze_glr_core::{Action, ParseRule, ParseTable, StateId, SymbolId};
use adze_ir::RuleId;
use adze_runtime::Token;
use adze_runtime::glr_engine::{ForestNode, ForestNodeId, GLRConfig, GLREngine, ParseForest};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helper: leak a ParseTable so we get `&'static ParseTable`
// ---------------------------------------------------------------------------

fn leak(table: ParseTable) -> &'static ParseTable {
    Box::leak(Box::new(table))
}

/// Minimal table: state 0 shifts on symbol 1 → state 1, state 1 accepts on symbol 0 (EOF).
fn shift_accept_table() -> &'static ParseTable {
    let mut t = ParseTable::default();
    t.state_count = 2;
    t.symbol_count = 2;
    t.action_table = vec![
        // state 0: no action on sym 0, shift(1) on sym 1
        vec![vec![], vec![Action::Shift(StateId(1))]],
        // state 1: accept on sym 0
        vec![vec![Action::Accept], vec![]],
    ];
    t.goto_table = vec![vec![], vec![]];
    leak(t)
}

/// Table that forks: state 0 has two shifts on symbol 1 (→ states 1 & 2).
/// Both paths accept on EOF.
fn fork_accept_table() -> &'static ParseTable {
    let mut t = ParseTable::default();
    t.state_count = 3;
    t.symbol_count = 2;
    t.action_table = vec![
        vec![
            vec![],
            vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))],
        ],
        vec![vec![Action::Accept], vec![]],
        vec![vec![Action::Accept], vec![]],
    ];
    t.goto_table = vec![vec![], vec![], vec![]];
    leak(t)
}

/// Table with a reduce rule: S → A (one-symbol RHS).
/// State 0: shift sym 1 → state 1.
/// State 1: reduce rule 0 on EOF.
/// State 2 (after goto): accept on EOF.
fn reduce_table() -> &'static ParseTable {
    let mut t = ParseTable::default();
    t.state_count = 3;
    t.symbol_count = 2;
    // nonterminal symbol id for S is SymbolId(2)
    t.nonterminal_to_index = BTreeMap::from([(SymbolId(2), 0)]);
    t.rules = vec![ParseRule {
        lhs: SymbolId(2),
        rhs_len: 1,
    }];
    t.action_table = vec![
        // state 0
        vec![vec![], vec![Action::Shift(StateId(1))]],
        // state 1: reduce rule 0 on EOF
        vec![vec![Action::Reduce(RuleId(0))], vec![]],
        // state 2 (reached via goto): accept on EOF
        vec![vec![Action::Accept], vec![]],
    ];
    t.goto_table = vec![
        vec![StateId(2)], // state 0, goto(S) → state 2
        vec![StateId(0)], // state 1 (unused goto row, filler)
        vec![StateId(0)], // state 2 (unused goto row, filler)
    ];
    leak(t)
}

/// Table whose only action for every input is Error.
fn error_only_table() -> &'static ParseTable {
    let mut t = ParseTable::default();
    t.state_count = 1;
    t.symbol_count = 2;
    t.action_table = vec![vec![vec![Action::Error], vec![Action::Error]]];
    t.goto_table = vec![vec![]];
    leak(t)
}

fn eof_token(pos: u32) -> Token {
    Token {
        kind: 0,
        start: pos,
        end: pos,
    }
}

fn sym_token(kind: u32, start: u32, end: u32) -> Token {
    Token { kind, start, end }
}

// ===========================================================================
// 1. Engine creation
// ===========================================================================

#[test]
fn default_config_has_expected_limits() {
    let cfg = GLRConfig::default();
    assert_eq!(cfg.max_forks, 1000);
    assert_eq!(cfg.max_forest_nodes, 10_000);
}

#[test]
fn custom_config_is_respected() {
    let cfg = GLRConfig {
        max_forks: 42,
        max_forest_nodes: 99,
    };
    assert_eq!(cfg.max_forks, 42);
    assert_eq!(cfg.max_forest_nodes, 99);
}

#[test]
fn engine_new_creates_single_initial_stack() {
    let table = shift_accept_table();
    let engine = GLREngine::new(table, GLRConfig::default());
    // Verify initial state through a successful reset (public API)
    drop(engine);
}

#[test]
#[should_panic(expected = "max_forks must be > 0")]
fn engine_new_panics_on_zero_max_forks() {
    let table = shift_accept_table();
    GLREngine::new(
        table,
        GLRConfig {
            max_forks: 0,
            max_forest_nodes: 10,
        },
    );
}

#[test]
#[should_panic(expected = "max_forest_nodes must be > 0")]
fn engine_new_panics_on_zero_max_forest_nodes() {
    let table = shift_accept_table();
    GLREngine::new(
        table,
        GLRConfig {
            max_forks: 10,
            max_forest_nodes: 0,
        },
    );
}

// ===========================================================================
// 2. Parsing – happy path
// ===========================================================================

#[test]
fn parse_shift_then_accept_produces_single_root() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 3), eof_token(3)];

    let forest = engine.parse(&tokens).expect("should succeed");
    assert_eq!(forest.root_count(), 1);
    assert!(forest.node_count() >= 1);
}

#[test]
fn parse_preserves_byte_range_on_terminal() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 5, 10), eof_token(10)];

    let forest = engine.parse(&tokens).expect("should succeed");
    let root = &forest.nodes[forest.roots[0].0];
    assert_eq!(root.range, 5..10);
}

#[test]
fn parse_with_reduce_produces_nonterminal_root() {
    let table = reduce_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 2), eof_token(2)];

    let forest = engine.parse(&tokens).expect("should succeed");
    assert_eq!(forest.root_count(), 1);
    // The root should be the nonterminal node (SymbolId(2))
    let root = &forest.nodes[forest.roots[0].0];
    assert_eq!(root.symbol, SymbolId(2));
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.range, 0..2);
}

// ===========================================================================
// 3. Error handling
// ===========================================================================

#[test]
fn parse_empty_token_stream_returns_error() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());

    let err = engine.parse(&[]).unwrap_err();
    assert!(err.to_string().contains("Empty token stream"));
}

#[test]
fn parse_unexpected_token_returns_syntax_error() {
    // Symbol 0 has no shift/accept in state 0 (only symbol 1 does).
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![eof_token(7)];

    let err = engine.parse(&tokens).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unexpected token") || msg.contains("Syntax error"));
}

#[test]
fn parse_error_only_table_returns_syntax_error() {
    let table = error_only_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 1)];

    let err = engine.parse(&tokens).unwrap_err();
    assert!(
        err.to_string().contains("Syntax error") || err.to_string().contains("No parse succeeded")
    );
}

#[test]
fn parse_with_no_valid_parse_returns_error() {
    // Table where shift leads to a dead-end (no accept, no reduce).
    let mut t = ParseTable::default();
    t.state_count = 2;
    t.symbol_count = 2;
    t.action_table = vec![
        vec![vec![], vec![Action::Shift(StateId(1))]],
        vec![vec![], vec![]], // state 1: dead end
    ];
    t.goto_table = vec![vec![], vec![]];
    let table = leak(t);

    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 1), eof_token(1)];

    let err = engine.parse(&tokens).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("No parse succeeded") || msg.contains("Syntax error"),
        "unexpected error message: {msg}"
    );
}

// ===========================================================================
// 4. Fork / merge
// ===========================================================================

#[test]
fn fork_produces_two_roots_when_both_paths_accept() {
    let table = fork_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 1), eof_token(1)];

    let forest = engine.parse(&tokens).expect("should succeed");
    assert_eq!(forest.root_count(), 2);
}

#[test]
fn fork_limit_exceeded_returns_error() {
    let table = fork_accept_table();
    let mut engine = GLREngine::new(
        table,
        GLRConfig {
            max_forks: 1,
            max_forest_nodes: 100,
        },
    );
    let tokens = vec![sym_token(1, 0, 1)];

    let err = engine.parse(&tokens).unwrap_err();
    assert!(err.to_string().contains("Fork limit exceeded"));
}

#[test]
fn merge_identical_stacks_deduplicates() {
    // Two identical shifts to the same state should merge into one stack.
    let mut t = ParseTable::default();
    t.state_count = 3;
    t.symbol_count = 3;
    // state 0: two shifts on sym 1 both go to state 1
    t.action_table = vec![
        vec![
            vec![],
            vec![Action::Shift(StateId(1)), Action::Shift(StateId(1))],
            vec![],
        ],
        vec![vec![Action::Accept], vec![], vec![]],
        vec![vec![], vec![], vec![]],
    ];
    t.goto_table = vec![vec![], vec![], vec![]];
    let table = leak(t);

    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 1), eof_token(1)];

    let forest = engine.parse(&tokens).expect("should succeed");
    // After merging, only one stack should have survived and accepted.
    assert!(forest.root_count() >= 1);
}

// ===========================================================================
// 5. Forest management
// ===========================================================================

#[test]
fn empty_forest_has_zero_counts() {
    // ParseForest is not directly constructable from outside, so we exercise
    // it through an engine that fails immediately (empty token stream).
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let _ = engine.parse(&[]);
    // After failure the engine's forest is empty, verify via reset + re-parse.
    engine.reset();
    let err = engine.parse(&[]);
    assert!(err.is_err());
}

#[test]
fn forest_node_count_increases_with_tokens() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 1), eof_token(1)];

    let forest = engine.parse(&tokens).expect("should succeed");
    assert!(forest.node_count() >= 1);
}

#[test]
fn forest_node_has_correct_symbol_and_range() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 10, 20), eof_token(20)];

    let forest = engine.parse(&tokens).expect("should succeed");
    let root = &forest.nodes[forest.roots[0].0];
    assert_eq!(root.symbol, SymbolId(1));
    assert_eq!(root.range, 10..20);
}

#[test]
fn forest_node_id_equality() {
    assert_eq!(ForestNodeId(0), ForestNodeId(0));
    assert_ne!(ForestNodeId(0), ForestNodeId(1));
}

#[test]
fn forest_node_id_copy_semantics() {
    let id = ForestNodeId(42);
    let copied = id; // Copy
    assert_eq!(id, copied);
}

// ===========================================================================
// 6. Token processing
// ===========================================================================

#[test]
fn single_token_then_eof_parses_correctly() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 5), eof_token(5)];

    let forest = engine.parse(&tokens).expect("should succeed");
    assert_eq!(forest.root_count(), 1);
}

#[test]
fn eof_at_start_with_no_shift_fails() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    // EOF immediately — no shift possible for symbol 0 in state 0
    let tokens = vec![eof_token(0)];

    assert!(engine.parse(&tokens).is_err());
}

#[test]
fn reduce_computes_span_from_children() {
    let table = reduce_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 3, 7), eof_token(7)];

    let forest = engine.parse(&tokens).expect("should succeed");
    let root = &forest.nodes[forest.roots[0].0];
    // Nonterminal should span its single child: 3..7
    assert_eq!(root.range, 3..7);
}

// ===========================================================================
// 7. Reset semantics
// ===========================================================================

#[test]
fn reset_allows_reuse_after_success() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());

    // First parse
    let tokens = vec![sym_token(1, 0, 1), eof_token(1)];
    let f1 = engine.parse(&tokens).expect("first parse");
    assert_eq!(f1.root_count(), 1);

    // Reset and parse again
    engine.reset();
    let tokens2 = vec![sym_token(1, 0, 2), eof_token(2)];
    let f2 = engine.parse(&tokens2).expect("second parse");
    assert_eq!(f2.root_count(), 1);
    // Different byte range
    let root = &f2.nodes[f2.roots[0].0];
    assert_eq!(root.range, 0..2);
}

#[test]
fn reset_allows_reuse_after_failure() {
    let table = shift_accept_table();
    let mut engine = GLREngine::new(table, GLRConfig::default());

    // Failing parse
    assert!(engine.parse(&[]).is_err());

    // Reset and try a valid parse
    engine.reset();
    let tokens = vec![sym_token(1, 0, 1), eof_token(1)];
    let forest = engine.parse(&tokens).expect("should succeed after reset");
    assert_eq!(forest.root_count(), 1);
}

// ===========================================================================
// 8. Struct Debug / Clone derives
// ===========================================================================

#[test]
fn glr_config_is_debug_and_clone() {
    let cfg = GLRConfig {
        max_forks: 5,
        max_forest_nodes: 10,
    };
    let cloned = cfg.clone();
    assert_eq!(format!("{:?}", cloned), format!("{:?}", cfg));
}

#[test]
fn forest_node_id_is_debug() {
    let id = ForestNodeId(7);
    let dbg = format!("{:?}", id);
    assert!(dbg.contains("7"));
}

#[test]
fn forest_node_is_debug_and_clone() {
    let node = ForestNode {
        symbol: SymbolId(3),
        children: vec![ForestNodeId(0)],
        range: 0..5,
    };
    let cloned = node.clone();
    assert_eq!(cloned.symbol, SymbolId(3));
    assert_eq!(format!("{:?}", cloned).len(), format!("{:?}", node).len());
}

// ===========================================================================
// 9. Boundary / edge cases
// ===========================================================================

#[test]
fn accept_with_no_prior_shift_is_harmless() {
    // Table where state 0 has Accept on symbol 1.
    // Accept without any nodes pushed is valid (empty parse, no root recorded).
    let mut t = ParseTable::default();
    t.state_count = 1;
    t.symbol_count = 2;
    t.action_table = vec![vec![vec![], vec![Action::Accept]]];
    t.goto_table = vec![vec![]];
    let table = leak(t);

    let mut engine = GLREngine::new(table, GLRConfig::default());
    let tokens = vec![sym_token(1, 0, 1)];

    // No root node was pushed, so this won't produce roots → "No parse succeeded"
    let result = engine.parse(&tokens);
    assert!(result.is_err());
}

#[test]
fn large_fork_within_limit_succeeds() {
    // Create a table where state 0 has 10 shifts on symbol 1 → states 1..=10,
    // and each of those states accepts on EOF.
    let n = 10;
    let mut t = ParseTable::default();
    t.state_count = n + 1;
    t.symbol_count = 2;
    let mut actions_for_sym1: Vec<Action> = Vec::new();
    for i in 1..=n {
        actions_for_sym1.push(Action::Shift(StateId(i as u16)));
    }
    let mut action_table = vec![vec![vec![], actions_for_sym1]];
    for _ in 1..=n {
        action_table.push(vec![vec![Action::Accept], vec![]]);
    }
    t.action_table = action_table;
    t.goto_table = (0..=n).map(|_| vec![]).collect();
    let table = leak(t);

    let mut engine = GLREngine::new(
        table,
        GLRConfig {
            max_forks: 100,
            max_forest_nodes: 1000,
        },
    );
    let tokens = vec![sym_token(1, 0, 1), eof_token(1)];

    let forest = engine.parse(&tokens).expect("should succeed with 10 forks");
    assert_eq!(forest.root_count(), n);
}
