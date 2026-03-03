//! Tests for glr-test-support helper functions.

use super::*;
use adze_glr_core::{Action, ParseRule, RuleId};
use adze_ir::SymbolId;

// ---------------------------------------------------------------------------
// make_minimal_table tests
// ---------------------------------------------------------------------------

/// Helper: build the simplest possible valid table (1 state, shift+accept).
fn one_state_table() -> adze_glr_core::ParseTable {
    // Layout: ERROR(0), terminal "a"(1), EOF(2), nonterminal "S"(3)
    let actions = vec![vec![
        vec![],                          // ERROR col
        vec![Action::Shift(StateId(0))], // "a" col
        vec![Action::Accept],            // EOF col
        vec![],                          // "S" col
    ]];
    let gotos = vec![vec![INVALID, INVALID, INVALID, StateId(0)]];
    let rules = vec![ParseRule {
        lhs: SymbolId(3),
        rhs_len: 1,
    }];
    make_minimal_table(
        actions,
        gotos,
        rules,
        SymbolId(3), // start
        SymbolId(2), // eof
        0,           // no external tokens
    )
}

#[test]
fn minimal_table_has_correct_dimensions() {
    let table = one_state_table();
    assert_eq!(table.state_count, 1);
    assert_eq!(table.symbol_count, 4);
    assert_eq!(table.action_table.len(), 1);
    assert_eq!(table.goto_table.len(), 1);
    assert_eq!(table.action_table[0].len(), 4);
    assert_eq!(table.goto_table[0].len(), 4);
}

#[test]
fn minimal_table_eof_column_matches_token_count() {
    let table = one_state_table();
    // EOF is at index 2, external_token_count=0, so token_count should be 2
    assert_eq!(table.token_count, 2);
    assert_eq!(table.external_token_count, 0);
}

#[test]
fn minimal_table_start_symbol_in_nonterminals() {
    let table = one_state_table();
    assert!(table.nonterminal_to_index.contains_key(&SymbolId(3)));
}

#[test]
fn minimal_table_initial_state_is_zero() {
    let table = one_state_table();
    assert_eq!(table.initial_state, StateId(0));
}

#[test]
fn minimal_table_preserves_actions() {
    let table = one_state_table();
    // Column 1 (terminal "a") should have Shift(0)
    assert_eq!(table.action_table[0][1].len(), 1);
    assert!(matches!(
        table.action_table[0][1][0],
        Action::Shift(StateId(0))
    ));
    // Column 2 (EOF) should have Accept
    assert_eq!(table.action_table[0][2].len(), 1);
    assert!(matches!(table.action_table[0][2][0], Action::Accept));
}

#[test]
fn minimal_table_preserves_gotos() {
    let table = one_state_table();
    // Column 3 (nonterminal "S") should have goto to state 0
    assert_eq!(table.goto_table[0][3], StateId(0));
    // Other columns should be INVALID
    assert_eq!(table.goto_table[0][0], INVALID);
    assert_eq!(table.goto_table[0][1], INVALID);
}

#[test]
fn minimal_table_preserves_rules() {
    let table = one_state_table();
    assert_eq!(table.rules.len(), 1);
    assert_eq!(table.rules[0].lhs, SymbolId(3));
    assert_eq!(table.rules[0].rhs_len, 1);
}

#[test]
fn assert_invariants_passes_for_valid_table() {
    let table = one_state_table();
    assert_parse_table_invariants(&table);
}

#[test]
fn multi_state_table() {
    // 2 states, 3 symbols: ERROR(0), "a"(1), EOF(2), S(3)
    let actions = vec![
        vec![vec![], vec![Action::Shift(StateId(1))], vec![], vec![]],
        vec![vec![], vec![], vec![Action::Accept], vec![]],
    ];
    let gotos = vec![
        vec![INVALID, INVALID, INVALID, StateId(1)],
        vec![INVALID, INVALID, INVALID, INVALID],
    ];
    let rules = vec![ParseRule {
        lhs: SymbolId(3),
        rhs_len: 1,
    }];
    let table = make_minimal_table(actions, gotos, rules, SymbolId(3), SymbolId(2), 0);

    assert_eq!(table.state_count, 2);
    assert_eq!(table.action_table.len(), 2);
    assert_eq!(table.goto_table.len(), 2);
    assert_parse_table_invariants(&table);
}

#[test]
fn table_with_external_tokens() {
    // Layout: ERROR(0), "a"(1), ext(2), EOF(3), S(4)
    let actions = vec![vec![
        vec![],
        vec![Action::Shift(StateId(0))],
        vec![],
        vec![Action::Accept],
        vec![],
    ]];
    let gotos = vec![vec![INVALID, INVALID, INVALID, INVALID, StateId(0)]];
    let rules = vec![ParseRule {
        lhs: SymbolId(4),
        rhs_len: 1,
    }];
    let table = make_minimal_table(actions, gotos, rules, SymbolId(4), SymbolId(3), 1);

    assert_eq!(table.external_token_count, 1);
    // token_count = eof_idx - external_token_count = 3 - 1 = 2
    assert_eq!(table.token_count, 2);
    assert_parse_table_invariants(&table);
}

#[test]
fn table_with_reduce_action() {
    // Layout: ERROR(0), "a"(1), EOF(2), S(3)
    let actions = vec![
        vec![vec![], vec![Action::Shift(StateId(1))], vec![], vec![]],
        vec![vec![], vec![], vec![Action::Reduce(RuleId(0))], vec![]],
    ];
    let gotos = vec![
        vec![INVALID, INVALID, INVALID, StateId(1)],
        vec![INVALID, INVALID, INVALID, INVALID],
    ];
    let rules = vec![ParseRule {
        lhs: SymbolId(3),
        rhs_len: 1,
    }];
    let table = make_minimal_table(actions, gotos, rules, SymbolId(3), SymbolId(2), 0);

    assert!(matches!(
        table.action_table[1][2][0],
        Action::Reduce(RuleId(0))
    ));
    assert_parse_table_invariants(&table);
}

// ---------------------------------------------------------------------------
// perf::measure tests
// ---------------------------------------------------------------------------

#[test]
fn measure_returns_function_result() {
    let (_, result) = perf::measure(|| 42);
    assert_eq!(result, 42);
}

#[test]
fn measure_returns_counters() {
    let (counters, _) = perf::measure(|| "hello");
    // Without perf-counters feature, counters should be default (all zeros)
    let _ = counters; // just verify it compiles and doesn't panic
}

// ---------------------------------------------------------------------------
// INVALID sentinel tests
// ---------------------------------------------------------------------------

#[test]
fn invalid_sentinel_is_max() {
    assert_eq!(INVALID, StateId(u16::MAX));
}

// ---------------------------------------------------------------------------
// test_utilities module re-export
// ---------------------------------------------------------------------------

#[test]
fn test_utilities_reexport_works() {
    // Verify the test_utilities module re-exports make_minimal_table
    let _ = test_utilities::make_minimal_table;
}
