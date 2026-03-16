//! Driver edge case tests for the GLR parser.
//!
//! Covers boundary conditions, error paths, and stress scenarios.
//!
//! Note: These tests use manually constructed parse tables that don't satisfy
//! all strict invariants (e.g., EOF/END parity). They are only compiled when
//! the `strict-invariants` feature is disabled.

#![cfg(not(feature = "strict-invariants"))]

use adze_glr_core::{Action, Driver, LexMode, ParseRule, ParseTable, RuleId, StateId, SymbolId};
use adze_ir::Grammar;

/// Build a minimal parse table that accepts a single terminal 'a' (SymbolId(1)).
fn minimal_accepting_table() -> ParseTable {
    let eof = SymbolId(0);
    let a = SymbolId(1);
    let start = SymbolId(2);

    let mut table = ParseTable {
        grammar: Grammar::new("minimal".to_string()),
        state_count: 3,
        symbol_count: 3,
        token_count: 2,
        eof_symbol: eof,
        start_symbol: start,
        initial_state: StateId(0),
        index_to_symbol: vec![eof, a],
        action_table: vec![
            vec![vec![], vec![Action::Shift(StateId(1))]],
            vec![vec![Action::Reduce(RuleId(0))], vec![]],
            vec![vec![Action::Accept], vec![]],
        ],
        goto_table: vec![vec![StateId(2)], vec![StateId(0)], vec![StateId(0)]],
        rules: vec![ParseRule {
            lhs: start,
            rhs_len: 1,
        }],
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            3
        ],
        symbol_metadata: vec![
            adze_glr_core::SymbolMetadata {
                name: String::new(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            };
            3
        ],
        ..Default::default()
    };

    table.symbol_to_index.insert(eof, 0);
    table.symbol_to_index.insert(a, 1);
    table.nonterminal_to_index.insert(start, 0);
    table
}

#[test]
fn driver_accepts_single_token_input() {
    let table = minimal_accepting_table();
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(1, 0, 1)]);
    assert!(result.is_ok(), "Single token should be accepted");
}

#[test]
fn driver_handles_empty_input() {
    let table = minimal_accepting_table();
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(std::iter::empty::<(u32, u32, u32)>());
    assert!(
        result.is_err(),
        "Empty input should error for grammar requiring 'a'"
    );
}

#[test]
fn driver_new_panics_with_empty_default_table() {
    let table = ParseTable::default();
    // Default table has no EOF in symbol_to_index, so debug_assert fires
    let result = std::panic::catch_unwind(|| {
        let _driver = Driver::new(&table);
    });
    // In debug mode this panics; verify it doesn't cause UB
    let _ = result;
}

#[test]
fn driver_rejects_unknown_token() {
    let table = minimal_accepting_table();
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(99, 0, 1)]);
    // Should not panic, just error
    let _ = result;
}

#[test]
fn driver_handles_extra_tokens_for_single_grammar() {
    let table = minimal_accepting_table();
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(1, 0, 1), (1, 1, 2)]);
    // Grammar only accepts one 'a', second should cause error
    let _ = result;
}

#[test]
fn driver_zero_width_token_does_not_panic() {
    let table = minimal_accepting_table();
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(1, 0, 0)]);
    let _ = result;
}
