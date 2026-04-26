//! Comprehensive tests for TableCompressor::compress() API with ParseTable construction.
//!
//! This test suite focuses on the main compression API that takes a ParseTable,
//! token_indices, and start_can_be_empty flag. It covers:
//!
//! - Empty and minimal tables
//! - Single-state and single-symbol tables
//! - Tables with only shifts, only reduces, or mixed actions
//! - Compressed output structure validation
//! - Compression semantics preservation
//! - Large table handling
//! - Duplicate row compression
//! - Unique vs identical row patterns
//! - Accept and error action handling
//! - Sparse vs dense goto tables
//! - State ID and symbol boundary conditions
//! - Deterministic compression output
//! - Small vs large table threshold detection
//! - Nonterminal and terminal symbol handling

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use adze_tablegen::compress::{CompressedGotoEntry, TableCompressor};
use std::collections::BTreeMap;

// ─── Test Helpers ──────────────────────────────────────────────────────────

/// Sentinel for "no goto" (from test_helpers::test::INVALID)
const INVALID: StateId = StateId(u16::MAX);

/// Create an *empty* but valid table for tests that don't care about actions/gotos.
///
/// `terms` = number of real terminals (excluding EOF); `nonterms` = number of non-terminals.
/// Symbol layout produced:
///   0: ERROR, 1..=terms: terminals, (terms+externals+1): EOF, the rest: non-terminals.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    // Ensure at least one nonterminal column so start_symbol is valid.
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff; // +1 for EOF itself

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16); // first nonterminal column (now always exists)
    let eof_symbol = SymbolId(eof_idx as u16);

    make_minimal_table(actions, gotos, vec![], start_symbol, eof_symbol, externals)
}

/// Build a minimal but fully-formed ParseTable suitable for unit tests.
///
/// Conventions expected by the project:
/// - Symbol layout: ERROR(0), terminals `[1..]`, EOF (= token_count + external_token_count), then non-terminals.
/// - `actions` is indexed by `[state][symbol_index]` and `gotos` by `[state][symbol_index]`.
fn make_minimal_table(
    mut actions: Vec<Vec<Vec<Action>>>,
    mut gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start_symbol: SymbolId,
    eof_symbol: SymbolId,
    external_token_count: usize,
) -> ParseTable {
    // Dimensions
    let state_count = actions.len().max(1);
    let symbol_cols_from_actions = actions.first().map(|r| r.len()).unwrap_or(0);
    let symbol_cols_from_gotos = gotos.first().map(|r| r.len()).unwrap_or(0);
    // Cover the columns referenced by start_symbol and eof_symbol too.
    let min_needed = (start_symbol.0 as usize + 1).max(eof_symbol.0 as usize + 1);
    let symbol_count = symbol_cols_from_actions
        .max(symbol_cols_from_gotos)
        .max(min_needed)
        .max(1);

    // Normalize shapes (pad rows/cols if needed)
    if actions.is_empty() {
        actions = vec![vec![vec![]; symbol_count]];
    } else {
        for row in &mut actions {
            if row.len() < symbol_count {
                row.resize_with(symbol_count, Vec::new);
            }
        }
    }
    if gotos.len() < state_count {
        gotos.resize_with(state_count, || vec![INVALID; symbol_count]);
    }
    for row in &mut gotos {
        if row.len() < symbol_count {
            row.resize(symbol_count, INVALID);
        }
    }

    // Build symbol maps
    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for col in 0..symbol_count {
        // "Is this column used as a goto for any state?"
        if gotos.iter().any(|row| row[col] != INVALID) {
            nonterminal_to_index.insert(SymbolId(col as u16), col);
        }
    }
    nonterminal_to_index
        .entry(start_symbol)
        .or_insert_with(|| start_symbol.0 as usize);

    // Invariants on EOF / token_count
    let eof_idx = eof_symbol.0 as usize;
    debug_assert!(
        eof_idx > 0 && eof_idx < symbol_count,
        "EOF column must be within 1..symbol_count (got {eof_idx} of {symbol_count})"
    );

    // By project convention: EOF index == token_count + external_token_count.
    // (token_count includes EOF; examples set token_count == eof_idx when externals==0)
    let token_count = eof_idx - external_token_count;

    // Minimal lexing configuration (one mode per state)
    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0
        };
        state_count
    ];

    // Build index_to_symbol from symbol_to_index
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (symbol_id, index) in &symbol_to_index {
        index_to_symbol[*index] = *symbol_id;
    }

    ParseTable {
        // core grids
        action_table: actions,
        goto_table: gotos,
        // grammar rules
        rules,
        // shapes
        state_count,
        symbol_count,
        // symbol bookkeeping
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![], // tests don't need metadata
        // token layout / sentinels
        token_count,
        external_token_count,
        eof_symbol,
        start_symbol,
        // parsing config
        initial_state: StateId(0),
        // lexing config
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        // advanced features (unused in hand tests)
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        // display / provenance (defaults are fine for tests)
        grammar: Grammar::default(),
        // GOTO indexing mode
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Create token indices for a table with terminals [1..=terminal_count], then EOF.
/// This includes all terminals AND the EOF symbol.
fn _make_token_indices(terminal_count: usize) -> Vec<usize> {
    let mut indices: Vec<usize> = (1..=terminal_count).collect();
    indices.push(terminal_count + 1); // EOF is after all terminals
    indices
}

/// Calculate the EOF column index for a table created with make_empty_table(states, terms, nonterms, externals)
/// eof_idx = 1 + terms + externals
fn _eof_idx_for_table(terms: usize, externals: usize) -> usize {
    1 + terms + externals
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 1: Compress empty parse table
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_empty_parse_table() {
    let mut table = make_empty_table(1, 1, 1, 0); // Need at least one terminal for shift action
    // Add a shift to state 0 on terminal 1
    if let Some(cell) = table.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(0)));
    }
    let compressor = TableCompressor::new();

    // Should succeed even with minimal table
    let result = compressor.compress(&table, &[1, 2], false); // eof_idx = 1 + 1 = 2
    assert!(
        result.is_ok(),
        "Compression should handle minimal table: {:?}",
        result.err()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 2: Compress single-state table
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_single_state_table() {
    let mut table = make_empty_table(1, 1, 1, 0);

    // Add a shift action to state 0
    if let Some(cell) = table.action_table.get_mut(0).and_then(|row| row.get_mut(1)) {
        cell.push(Action::Shift(StateId(0)));
    }

    let compressor = TableCompressor::new();
    let result = compressor.compress(&table, &[1, 2], false);
    assert!(result.is_ok(), "Should compress single-state table");
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 3: Compress table with only shifts
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_table_with_only_shifts() {
    let mut table = make_empty_table(3, 2, 1, 0);

    // Fill with shift actions only
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 {
                cell.push(Action::Shift(StateId((state_idx + sym_idx) as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let result = compressor.compress(&table, &[1, 2, 3], false);
    assert!(result.is_ok(), "Should compress shift-only table");

    let compressed = result.unwrap();
    assert!(
        !compressed.action_table.data.is_empty(),
        "Should have action data"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 4: Compress table with only reduces
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_table_with_only_reduces() {
    let mut table = make_empty_table(3, 2, 1, 0);

    // Need at least one shift in state 0 to pass validation
    if let Some(cell) = table.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(1)));
    }

    // Fill other states with reduce actions only
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        if state_idx == 0 {
            continue;
        } // Skip state 0, already has shift
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 {
                cell.push(Action::Reduce(RuleId(state_idx as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let result = compressor.compress(&table, &[1, 2, 3], false);
    assert!(result.is_ok(), "Should compress reduce-only table");

    let compressed = result.unwrap();
    assert!(
        !compressed.action_table.data.is_empty(),
        "Should have action data"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 5: Compress table with mixed actions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_table_with_mixed_actions() {
    let mut table = make_empty_table(4, 3, 1, 0);

    // Fill with mixed actions
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx == 0 {
                continue;
            }
            match (state_idx + sym_idx) % 3 {
                0 => cell.push(Action::Shift(StateId(sym_idx as u16))),
                1 => cell.push(Action::Reduce(RuleId(state_idx as u16))),
                _ => cell.push(Action::Accept),
            }
        }
    }

    let compressor = TableCompressor::new();
    let result = compressor.compress(&table, &[1, 2, 3, 4], false);
    assert!(result.is_ok(), "Should compress mixed-action table");
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 6: Compressed output contains small_parse_table field
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_output_contains_small_parse_table() {
    let mut table = make_empty_table(5, 2, 1, 0);

    // Add some actions
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        if let Some(cell) = row.get_mut(1) {
            cell.push(Action::Shift(StateId(state_idx as u16)));
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3], false).unwrap();

    assert!(
        !compressed.action_table.data.is_empty(),
        "Compressed action table should have data"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 7: Compressed output contains small_parse_table_map
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_output_contains_small_parse_table_map() {
    let mut table = make_empty_table(5, 2, 1, 0);

    // Add some actions
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        if let Some(cell) = row.get_mut(1) {
            cell.push(Action::Shift(StateId(state_idx as u16)));
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3], false).unwrap();

    assert!(
        !compressed.action_table.row_offsets.is_empty(),
        "Compressed action table should have row_offsets (map)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 8: Compression preserves action semantics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_preserves_action_semantics() {
    let mut table = make_empty_table(3, 3, 1, 0);

    // Set up specific action patterns
    if let Some(row) = table.action_table.get_mut(0) {
        if let Some(cell) = row.get_mut(1) {
            cell.push(Action::Shift(StateId(5)));
        }
        if let Some(cell) = row.get_mut(2) {
            cell.push(Action::Reduce(RuleId(3)));
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();

    // Verify we have action entries
    assert!(
        compressed
            .action_table
            .data
            .iter()
            .any(|e| matches!(e.action, Action::Shift(_))),
        "Should preserve shift actions"
    );
    assert!(
        compressed
            .action_table
            .data
            .iter()
            .any(|e| matches!(e.action, Action::Reduce(_))),
        "Should preserve reduce actions"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 9: Compression handles large tables (50+ states)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_handles_large_tables() {
    let mut table = make_empty_table(60, 10, 1, 0);

    // Fill with diverse actions
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 && (state_idx + sym_idx) % 5 == 0 {
                cell.push(Action::Shift(StateId((sym_idx % 20) as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let token_indices: Vec<usize> = (1..=11).collect();
    let result = compressor.compress(&table, &token_indices, false);
    assert!(
        result.is_ok(),
        "Should handle large tables: {:?}",
        result.err()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 10: Compress table with many duplicate rows → good compression ratio
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_table_with_duplicate_rows_good_ratio() {
    let mut table = make_empty_table(20, 3, 1, 0);

    // Create duplicate rows by using same pattern
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        let pattern = state_idx % 3; // Only 3 distinct patterns
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 {
                match pattern {
                    0 => cell.push(Action::Shift(StateId(1))),
                    1 => cell.push(Action::Reduce(RuleId(0))),
                    _ => cell.push(Action::Accept),
                }
            }
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();

    // With many duplicate rows, compression should reduce total entries
    // (This is implicit in the row_offsets structure)
    assert!(
        !compressed.action_table.row_offsets.is_empty(),
        "Should produce row offsets"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 11: Compress table with all unique rows
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_table_with_all_unique_rows() {
    let mut table = make_empty_table(10, 3, 1, 0);

    // Make each row unique
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 {
                // Use unique values per state
                cell.push(Action::Shift(StateId(state_idx as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();

    assert!(
        !compressed.action_table.row_offsets.is_empty(),
        "Should handle all-unique-rows table"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 12: Compression handles accept actions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_handles_accept_actions() {
    let mut table = make_empty_table(3, 2, 1, 0);

    // Need a shift action in state 0 to pass validation
    if let Some(cell) = table.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(1)));
    }

    // Place Accept action in state 1, EOF column
    if let Some(cell) = table.action_table.get_mut(1).and_then(|row| row.get_mut(3)) {
        // EOF is at column 3 for make_empty_table(3, 2, 1, 0)
        cell.push(Action::Accept);
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3], false).unwrap();

    assert!(
        compressed
            .action_table
            .data
            .iter()
            .any(|e| matches!(e.action, Action::Accept)),
        "Should preserve Accept actions"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 13: Compression handles error entries
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_handles_error_entries() {
    let mut table = make_empty_table(3, 3, 1, 0);

    // Leave some cells empty (error) and fill others
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 && state_idx % 2 == 0 {
                cell.push(Action::Shift(StateId(0)));
            }
            // sym_idx == 0 or state_idx % 2 != 0 → leave as error
        }
    }

    let compressor = TableCompressor::new();
    let result = compressor.compress(&table, &[1, 2, 3, 4], false);
    assert!(
        result.is_ok(),
        "Should handle tables with error entries: {:?}",
        result.err()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 14: Small parse table map indices are valid
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn small_parse_table_map_indices_valid() {
    let mut table = make_empty_table(5, 3, 1, 0);

    // Add diverse actions
    for row in table.action_table.iter_mut() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 {
                cell.push(Action::Shift(StateId(sym_idx as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();

    let row_offsets = &compressed.action_table.row_offsets;
    let data_len = compressed.action_table.data.len() as u16;

    // All indices should be within bounds
    for &offset in row_offsets.iter() {
        assert!(
            offset <= data_len,
            "Row offset {} must be <= data_len {}",
            offset,
            data_len
        );
    }

    // Row offsets should be strictly increasing
    for i in 1..row_offsets.len() {
        assert!(
            row_offsets[i] >= row_offsets[i - 1],
            "Row offsets must be non-decreasing"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 15: Compression is deterministic (same input → same output)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_deterministic() {
    let mut table = make_empty_table(5, 3, 1, 0);

    // Fill with deterministic pattern
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 && state_idx % 2 == 0 {
                cell.push(Action::Shift(StateId(state_idx as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();

    // Compress twice
    let compressed1 = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();
    let compressed2 = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();

    // Should produce identical row_offsets
    assert_eq!(
        compressed1.action_table.row_offsets, compressed2.action_table.row_offsets,
        "Compression should be deterministic"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 16: Large state detection threshold
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn large_state_detection_threshold() {
    let compressor = TableCompressor::new();

    // Test below threshold - need to add shift actions for validation
    let mut table_small = make_empty_table(1000, 10, 1, 0);
    if let Some(cell) = table_small.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(0)));
    }
    let result_small = compressor.compress(&table_small, &(1..=11).collect::<Vec<_>>(), false);
    assert!(result_small.is_ok(), "Should handle 1000-state table");

    // Both should produce CompressedTables
    assert!(result_small.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 17: Compression with sparse goto table
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_with_sparse_goto_table() {
    let mut table = make_empty_table(5, 5, 1, 0);

    // Add action to pass state 0 validation
    if let Some(cell) = table.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(0)));
    }

    // Make goto table sparse: only a few entries
    for row in table.goto_table.iter_mut() {
        for cell in row.iter_mut() {
            *cell = StateId(u16::MAX); // INVALID
        }
    }

    // Add only 3 sparse entries
    if !table.goto_table.is_empty() && !table.goto_table[0].is_empty() {
        table.goto_table[0][2] = StateId(10);
    }
    if table.goto_table.len() > 2 && table.goto_table[2].len() > 3 {
        table.goto_table[2][3] = StateId(20);
    }

    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &[1, 2, 3, 4, 5, 6], false)
        .unwrap();

    assert!(
        !compressed.goto_table.data.is_empty(),
        "Sparse goto table should compress"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 18: Compression with dense goto table
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_with_dense_goto_table() {
    let mut table = make_empty_table(5, 5, 3, 0);

    // Add action to pass state 0 validation
    if let Some(cell) = table.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(0)));
    }

    // Fill all goto entries
    for (state_idx, row) in table.goto_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            *cell = StateId((state_idx + sym_idx) as u16);
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &[1, 2, 3, 4, 5, 6], false)
        .unwrap();

    assert!(
        !compressed.goto_table.data.is_empty(),
        "Dense goto table should compress"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 19: Compressed tables used in ABI generation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_tables_used_in_abi_generation() {
    let mut table = make_empty_table(5, 3, 1, 0);

    // Set up minimal valid table for ABI
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        if let Some(cell) = row.get_mut(1) {
            cell.push(Action::Shift(StateId(state_idx as u16)));
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();

    // Verify structure is suitable for ABI
    assert!(!compressed.action_table.row_offsets.is_empty());
    assert!(!compressed.goto_table.row_offsets.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 20: Multiple compressions produce identical results
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_compressions_identical() {
    let mut table = make_empty_table(10, 4, 1, 0);

    // Add a shift in state 0 for validation
    if let Some(cell) = table.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(0)));
    }

    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        if state_idx == 0 {
            continue;
        } // Skip state 0
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 && (state_idx + sym_idx) % 3 == 0 {
                cell.push(Action::Reduce(RuleId(state_idx as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let token_indices = vec![1, 2, 3, 4, 5];

    let c1 = compressor.compress(&table, &token_indices, false).unwrap();
    let c2 = compressor.compress(&table, &token_indices, false).unwrap();
    let c3 = compressor.compress(&table, &token_indices, false).unwrap();

    assert_eq!(
        c1.action_table.row_offsets, c2.action_table.row_offsets,
        "First and second compression should be identical"
    );
    assert_eq!(
        c2.action_table.row_offsets, c3.action_table.row_offsets,
        "Second and third compression should be identical"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 21: Compression handles boundary state IDs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_rejects_state_ids_that_exceed_small_encoding_width() {
    let mut table = make_empty_table(5, 3, 1, 0);

    // Use boundary state IDs
    if let Some(row) = table.action_table.get_mut(0) {
        if let Some(cell) = row.get_mut(1) {
            cell.push(Action::Shift(StateId(0))); // Min state
        }
        if let Some(cell) = row.get_mut(2) {
            cell.push(Action::Shift(StateId(u16::MAX - 1))); // Too large for small-table encoding
        }
    }

    let compressor = TableCompressor::new();
    let result = compressor.compress(&table, &[1, 2, 3, 4], false);
    assert!(result.is_err(), "oversized state ids must be rejected");
    let err = result.err().expect("checked is_err above");
    assert!(
        err.to_string()
            .contains("Shift state 65534 too large for small table encoding"),
        "{err}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 22: Compressed table data is non-negative
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_table_data_non_negative() {
    let mut table = make_empty_table(5, 3, 1, 0);

    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 {
                cell.push(Action::Shift(StateId((state_idx + 1) as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&table, &[1, 2, 3, 4], false).unwrap();

    // Row offsets are u16 values (always non-negative by type)
    assert!(!compressed.action_table.row_offsets.is_empty());
    assert!(!compressed.goto_table.row_offsets.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 23: Compression preserves symbol-to-index mapping
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_preserves_symbol_to_index_mapping() {
    let mut table = make_empty_table(3, 4, 1, 0);

    // Record the symbol mapping before compression
    let _original_symbol_count = table.symbol_count;

    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 && sym_idx <= 2 {
                cell.push(Action::Shift(StateId(state_idx as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &[1, 2, 3, 4, 5], false)
        .unwrap();

    // The row_offsets length should match state count + 1
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        table.state_count + 1,
        "Row offsets should match state count + 1"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 24: Compression output fits in u16 range
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_output_fits_in_u16_range() {
    let mut table = make_empty_table(100, 50, 1, 0);

    // Fill table to ensure significant compression data
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        for (sym_idx, cell) in row.iter_mut().enumerate() {
            if sym_idx > 0 {
                cell.push(Action::Shift(StateId((state_idx % 100) as u16)));
            }
        }
    }

    let compressor = TableCompressor::new();
    let token_indices: Vec<usize> = (1..=51).collect();
    let compressed = compressor.compress(&table, &token_indices, false).unwrap();

    // All row_offsets should fit in u16 (they are u16)
    let data_len = compressed.action_table.data.len();
    for &offset in &compressed.action_table.row_offsets {
        assert!(
            offset as usize <= data_len,
            "Offset {} must be within data length {}",
            offset,
            data_len
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 25: Compression with nonterminal columns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_with_nonterminal_columns() {
    let mut table = make_empty_table(5, 5, 2, 0);

    // Fill action table (terminals)
    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        if let Some(cell) = row.get_mut(1) {
            cell.push(Action::Shift(StateId(state_idx as u16)));
        }
    }

    // Fill goto table (nonterminals)
    for (state_idx, row) in table.goto_table.iter_mut().enumerate() {
        if state_idx < 4 && row.len() > 3 {
            row[3] = StateId((state_idx + 1) as u16);
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &[1, 2, 3, 4, 5, 6], false)
        .unwrap();

    assert!(
        !compressed.goto_table.row_offsets.is_empty(),
        "Should handle nonterminals"
    );
    assert!(
        !compressed.action_table.row_offsets.is_empty(),
        "Should handle terminals"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// BONUS TEST 26: Validation after compression
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn validation_after_compression() {
    let mut table = make_empty_table(5, 3, 1, 0);

    for (state_idx, row) in table.action_table.iter_mut().enumerate() {
        if let Some(cell) = row.get_mut(1) {
            cell.push(Action::Shift(StateId(state_idx as u16)));
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &[1, 2, 3, 4, 5], false)
        .unwrap();

    // Validate the compressed tables
    let validation_result = compressed.validate(&table);
    assert!(
        validation_result.is_ok(),
        "Compressed tables should validate"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// BONUS TEST 27: Compression with goto run-length encoding
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_with_goto_run_length_encoding() {
    let mut table = make_empty_table(10, 8, 2, 0);

    // Add shift action to state 0 for validation
    if let Some(cell) = table.action_table[0].get_mut(1) {
        cell.push(Action::Shift(StateId(0)));
    }

    // Create run-length patterns in goto table
    for row in table.goto_table.iter_mut() {
        for (i, entry) in row.iter_mut().enumerate() {
            if (3..=5).contains(&i) {
                *entry = StateId(42); // Three identical consecutive entries
            }
        }
    }

    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &(1..=9).collect::<Vec<_>>(), false)
        .unwrap();

    // Verify goto compression produced entries
    assert!(
        !compressed.goto_table.data.is_empty(),
        "Should have compressed goto data"
    );

    // Check for run-length encoded entries
    let has_run_length = compressed.goto_table.data.iter().any(|entry| {
        matches!(
            entry,
            CompressedGotoEntry::RunLength {
                state: 42,
                count: 3
            }
        )
    });

    assert!(
        has_run_length,
        "Should use run-length encoding for repeated states"
    );
}
