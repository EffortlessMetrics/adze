//! Test suite for Tree-sitter table normalization and round-trip verification

#![cfg(all(feature = "pure-rust", not(feature = "incremental_glr")))]
#![allow(clippy::duplicate_mod)]

#[cfg(feature = "ts-compat")]
use adze::adze_glr_core as glr_core;
#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::decoder::decode_parse_table;
use adze::ts_format::choose_action;

#[cfg(not(feature = "ts-compat"))]
use adze_glr_core as glr_core;
#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use glr_core::{Action, ParseRule, ParseTable};
use ir::{Grammar, RuleId, StateId, SymbolId};

#[path = "support/json_grammar.rs"]
mod json_grammar;
#[allow(clippy::duplicate_mod)]
#[path = "support/language_builder.rs"]
mod language_builder;

// Note: unified_json_helper removed to avoid FFI segmentation faults
// Tests now use safer mock language approaches

/// Test that identity mapping is correctly established
#[test]
#[ignore = "TS normalization not yet stable"]
fn test_identity_mapping() {
    let mut table = create_simple_table();
    language_builder::normalize_table_for_ts(&mut table);

    // Verify identity mapping: symbol i at column i
    for (i, &sym) in table.index_to_symbol.iter().enumerate() {
        assert_eq!(sym.0 as usize, i, "Non-identity symbol at column {}", i);
        assert_eq!(
            *table.symbol_to_index.get(&sym).unwrap(),
            i,
            "symbol_to_index doesn't match for symbol {}",
            sym.0
        );
    }
}

/// Test that NT gotos are added to action table as Shift actions
#[test]
#[ignore = "TS normalization not yet stable"]
fn test_nt_gotos_in_action_table() {
    let mut table = create_simple_table();
    let token_boundary = table.token_count + table.external_token_count;

    // Add a goto for testing
    if table.goto_table.is_empty() {
        table.goto_table = vec![vec![StateId(0); 10]; table.state_count];
    }
    table.goto_table[0][token_boundary] = StateId(2); // NT goto from state 0

    language_builder::normalize_table_for_ts(&mut table);

    // After normalization, the NT goto should appear as a Shift in action_table
    let nt_col = token_boundary; // After identity mapping
    assert!(
        table.action_table[0][nt_col]
            .iter()
            .any(|a| matches!(a, Action::Shift(StateId(2)))),
        "NT goto not found as Shift action after normalization"
    );
}

/// Test that Accept is injected at the correct location
#[test]
#[ignore = "TS normalization not yet stable"]
fn test_accept_injection() {
    let mut table = create_simple_table();
    language_builder::normalize_table_for_ts(&mut table);

    // Verify Accept exists on EOF column
    let eof_col = table.eof_symbol.0 as usize;
    let has_accept = table
        .action_table
        .iter()
        .any(|row| eof_col < row.len() && row[eof_col].iter().any(|a| matches!(a, Action::Accept)));
    assert!(
        has_accept,
        "No Accept action found on EOF column after normalization"
    );
}

/// Test round-trip: encode → decode → verify actions preserved
/// This test now uses a mock language to avoid FFI segmentation faults
#[test]
#[ignore = "TS normalization not yet stable"]
fn test_round_trip_preservation() {
    // Create a mock language instead of using FFI bridge
    let mut table = create_simple_table();

    // Add Accept action to make it realistic
    table.action_table[0][0].push(Action::Accept);

    language_builder::normalize_table_for_ts(&mut table);
    let grammar = Grammar::default();
    let lang = Box::leak(Box::new(language_builder::build_json_ts_language(
        &grammar, &table,
    )));

    // Decode it back
    let decoded = decode_parse_table(lang);

    // The decoder should have computed a valid start symbol
    assert_ne!(
        decoded.start_symbol.0, 0,
        "Start symbol should not be ERROR"
    );

    // Verify EOF is in valid range
    let eof_col = decoded.eof_symbol.0 as usize;
    assert!(
        eof_col < decoded.index_to_symbol.len(),
        "EOF column {} out of bounds",
        eof_col
    );

    // Verify Accept exists
    let has_accept = decoded
        .action_table
        .iter()
        .any(|row| eof_col < row.len() && row[eof_col].iter().any(|a| matches!(a, Action::Accept)));
    assert!(has_accept, "No Accept action in decoded table");
}

/// Test that rules are correctly preserved with rule IDs
#[test]
#[ignore = "TS normalization not yet stable"]
fn test_rule_preservation() {
    let mut table = create_simple_table();

    // Add some rules
    table.rules.push(ParseRule {
        lhs: SymbolId(10), // NT
        rhs_len: 2,
    });

    language_builder::normalize_table_for_ts(&mut table);

    // Verify rules are still accessible
    assert!(!table.rules.is_empty(), "Rules lost during normalization");
    assert_eq!(table.rules[0].rhs_len, 2, "Rule RHS length changed");
}

/// Test that choose_action is consistent before and after normalization
#[test]
#[ignore = "TS normalization not yet stable"]
fn test_choose_action_consistency() {
    let mut table = create_simple_table();

    // Add multiple actions to a cell
    table.action_table[0][0].push(Action::Shift(StateId(1)));
    table.action_table[0][0].push(Action::Reduce(RuleId(0)));

    let before = choose_action(&table.action_table[0][0]);

    language_builder::normalize_table_for_ts(&mut table);

    // After identity mapping, symbol 0 should still be at column 0
    let after = choose_action(&table.action_table[0][0]);

    assert_eq!(before, after, "choose_action changed after normalization");
}

// Helper function to create a simple test table
fn create_simple_table() -> ParseTable {
    use std::collections::BTreeMap;

    ParseTable {
        state_count: 5,
        token_count: 3,
        external_token_count: 0,
        symbol_count: 8,
        action_table: vec![vec![vec![]; 8]; 5],
        goto_table: vec![vec![StateId(0); 8]; 5],
        symbol_to_index: [
            (SymbolId(0), 0), // EOF
            (SymbolId(1), 1), // Token 1
            (SymbolId(2), 2), // Token 2
            (SymbolId(3), 3), // NT 3
            (SymbolId(4), 4), // NT 4
        ]
        .into_iter()
        .collect(),
        index_to_symbol: vec![
            SymbolId(0),
            SymbolId(1),
            SymbolId(2),
            SymbolId(3),
            SymbolId(4),
        ],
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(4),
        initial_state: StateId(0),
        rules: vec![],
        lex_modes: vec![],
        symbol_metadata: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: glr_core::GotoIndexing::NonterminalMap,
        alias_sequences: vec![],
        extras: vec![],
        grammar: Grammar::default(),
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}
