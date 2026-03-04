#![allow(clippy::needless_range_loop)]
//! Property-based and unit tests for CompressedParseTable in adze-tablegen.
//!
//! Covers: compression from ParseTable, lookup correctness, size invariants,
//! state/symbol count preservation, action/goto lookup, and determinism.

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId, Token, TokenPattern};
use adze_tablegen::compress::{CompressedParseTable, TableCompressor};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ── helpers ──────────────────────────────────────────────────────────────

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable from scratch (integration-test-safe, no cfg(test) helpers).
fn make_parse_table(
    mut actions: Vec<Vec<Vec<Action>>>,
    mut gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start_symbol: SymbolId,
    eof_symbol: SymbolId,
) -> ParseTable {
    let state_count = actions.len().max(1);
    let sym_from_act = actions.first().map(|r| r.len()).unwrap_or(0);
    let sym_from_goto = gotos.first().map(|r| r.len()).unwrap_or(0);
    let min_needed = (start_symbol.0 as usize + 1).max(eof_symbol.0 as usize + 1);
    let symbol_count = sym_from_act.max(sym_from_goto).max(min_needed).max(1);

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

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index = BTreeMap::new();
    for col in 0..symbol_count {
        if gotos.iter().any(|r| r[col] != INVALID) {
            nonterminal_to_index.insert(SymbolId(col as u16), col);
        }
    }
    nonterminal_to_index
        .entry(start_symbol)
        .or_insert(start_symbol.0 as usize);

    let eof_idx = eof_symbol.0 as usize;
    let token_count = eof_idx;

    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0
        };
        state_count
    ];
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (&sid, &idx) in &symbol_to_index {
        index_to_symbol[idx] = sid;
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules,
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count,
        external_token_count: 0,
        eof_symbol,
        start_symbol,
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: Grammar::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build an empty but valid parse table.
fn make_empty_table(states: usize, terms: usize, nonterms: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms; // 0=ERROR, 1..=terms, then EOF
    let nonterms_eff = nonterms.max(1);
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];
    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);

    make_parse_table(actions, gotos, vec![], start_symbol, eof_symbol)
}

/// Build a ParseTable with at least one token shift in state 0 so that
/// `TableCompressor::compress` passes its state-0 validation.
fn table_with_shift_in_s0(
    num_states: usize,
    num_terms: usize,
    extra_actions: Vec<(usize, usize, Action)>,
) -> ParseTable {
    let num_states = num_states.max(2); // need at least 2 states for shift target
    let num_terms = num_terms.max(1);
    let eof_idx = num_terms + 1;
    let start_nt = eof_idx + 1;
    let symbol_count = start_nt + 1;

    let mut actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; num_states];
    actions[0][1] = vec![Action::Shift(StateId(1))];

    for (s, sym, act) in extra_actions {
        if s < num_states && sym < symbol_count {
            actions[s][sym].push(act);
        }
    }

    let gotos = vec![vec![INVALID; symbol_count]; num_states];

    // Register terminal symbols 1..=num_terms as tokens in the grammar
    let mut grammar = Grammar::default();
    for t in 1..=num_terms {
        grammar.tokens.insert(
            SymbolId(t as u16),
            Token {
                name: format!("t{t}"),
                pattern: TokenPattern::String(format!("t{t}")),
                fragile: false,
            },
        );
    }

    let mut pt = make_parse_table(
        actions,
        gotos,
        vec![],
        SymbolId(start_nt as u16),
        SymbolId(eof_idx as u16),
    );
    pt.grammar = grammar;
    pt
}

fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

#[allow(dead_code)]
fn flat_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

fn action_cell_strategy() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(action_strategy(), 0..=3)
}

fn action_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Vec<Action>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(action_cell_strategy(), symbols..=symbols),
            states..=states,
        )
    })
}

#[allow(dead_code)]
fn flat_action_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Action>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(flat_action_strategy(), symbols..=symbols),
            states..=states,
        )
    })
}

fn goto_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Option<StateId>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(
                prop::option::of((0u16..50).prop_map(StateId)),
                symbols..=symbols,
            ),
            states..=states,
        )
    })
}

// ═════════════════════════════════════════════════════════════════════════
// 1. Compression from ParseTable
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_compressed_parse_table_from_empty_table() {
    let pt = make_empty_table(2, 2, 1);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

#[test]
fn test_compressed_parse_table_from_single_state() {
    let pt = make_empty_table(1, 1, 1);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.state_count() >= 1);
    assert!(cpt.symbol_count() >= 1);
}

#[test]
fn test_compressed_parse_table_from_large_table() {
    let pt = make_empty_table(20, 10, 5);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

// ═════════════════════════════════════════════════════════════════════════
// 2. Compressed table lookup correctness (action table round-trip)
// ═════════════════════════════════════════════════════════════════════════

proptest! {
    #[test]
    fn prop_action_roundtrip(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                let got = decompress_action(&compressed, state, sym);
                prop_assert_eq!(got, expected);
            }
        }
    }
}

#[test]
fn test_action_lookup_shift() {
    let table = vec![vec![vec![Action::Shift(StateId(5))], vec![Action::Error]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(5))
    );
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
}

#[test]
fn test_action_lookup_reduce() {
    let table = vec![vec![vec![Action::Reduce(RuleId(3))], vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Reduce(RuleId(3))
    );
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Accept);
}

// ═════════════════════════════════════════════════════════════════════════
// 3. Compressed table size <= original
// ═════════════════════════════════════════════════════════════════════════

proptest! {
    #[test]
    fn prop_compressed_action_rows_le_original(table in action_table_strategy(10, 10)) {
        let compressed = compress_action_table(&table);
        // Unique rows must be <= total rows
        prop_assert!(compressed.unique_rows.len() <= table.len());
    }
}

#[test]
fn test_identical_rows_deduplicated() {
    let row = vec![vec![Action::Error], vec![Action::Shift(StateId(1))]];
    let table = vec![row.clone(), row.clone(), row.clone()];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 3);
}

proptest! {
    #[test]
    fn prop_goto_sparse_size_le_dense(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        let dense_count: usize = table.iter().flat_map(|r| r.iter()).filter(|v| v.is_some()).count();
        prop_assert_eq!(compressed.entries.len(), dense_count);
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 4. State count preservation
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_state_count_preserved_in_compressed_parse_table() {
    for states in [1, 2, 5, 10, 20] {
        let pt = make_empty_table(states, 3, 1);
        let cpt = CompressedParseTable::from_parse_table(&pt);
        assert_eq!(cpt.state_count(), pt.state_count, "states={states}");
    }
}

proptest! {
    #[test]
    fn prop_state_count_preserved(table in action_table_strategy(12, 6)) {
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.state_to_row.len(), table.len());
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 5. Symbol count preservation
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_count_preserved_in_compressed_parse_table() {
    let pt = make_empty_table(3, 5, 2);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

proptest! {
    #[test]
    fn prop_symbol_count_preserved(table in action_table_strategy(6, 12)) {
        let compressed = compress_action_table(&table);
        // Each unique row must have the same column count as the original
        for row in &compressed.unique_rows {
            prop_assert_eq!(row.len(), table[0].len());
        }
    }
}

#[test]
fn test_new_for_testing_preserves_counts() {
    let cpt = CompressedParseTable::new_for_testing(42, 17);
    assert_eq!(cpt.symbol_count(), 42);
    assert_eq!(cpt.state_count(), 17);
}

// ═════════════════════════════════════════════════════════════════════════
// 6. Action lookup after compression (via TableCompressor)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_table_compressor_action_entries() {
    let pt = table_with_shift_in_s0(3, 2, vec![(1, 2, Action::Reduce(RuleId(0)))]);
    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&pt.grammar, &pt);
    let compressed = compressor.compress(&pt, &token_indices, false).unwrap();

    // Verify row_offsets has state_count + 1 entries
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
    // Verify default_actions has state_count entries
    assert_eq!(
        compressed.action_table.default_actions.len(),
        pt.state_count
    );
}

#[test]
fn test_table_compressor_action_row_offsets_increasing() {
    let pt = table_with_shift_in_s0(4, 3, vec![]);
    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&pt.grammar, &pt);
    let compressed = compressor.compress(&pt, &token_indices, false).unwrap();

    for i in 1..compressed.action_table.row_offsets.len() {
        assert!(
            compressed.action_table.row_offsets[i] >= compressed.action_table.row_offsets[i - 1],
            "row_offsets not monotonically increasing at index {i}"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 7. Goto lookup after compression
// ═════════════════════════════════════════════════════════════════════════

proptest! {
    #[test]
    fn prop_goto_roundtrip(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, &original) in row.iter().enumerate() {
                let got = decompress_goto(&compressed, state, sym);
                prop_assert_eq!(got, original);
            }
        }
    }
}

#[test]
fn test_goto_lookup_present() {
    let table = vec![vec![Some(StateId(7)), None, Some(StateId(3))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(7)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(3)));
}

#[test]
fn test_goto_all_none() {
    let table = vec![vec![None, None, None]; 3];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
    for s in 0..3 {
        for sym in 0..3 {
            assert_eq!(decompress_goto(&compressed, s, sym), None);
        }
    }
}

#[test]
fn test_table_compressor_goto_row_offsets_increasing() {
    let pt = table_with_shift_in_s0(4, 3, vec![]);
    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&pt.grammar, &pt);
    let compressed = compressor.compress(&pt, &token_indices, false).unwrap();

    for i in 1..compressed.goto_table.row_offsets.len() {
        assert!(
            compressed.goto_table.row_offsets[i] >= compressed.goto_table.row_offsets[i - 1],
            "goto row_offsets not monotonically increasing at index {i}"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 8. Compression determinism
// ═════════════════════════════════════════════════════════════════════════

proptest! {
    #[test]
    fn prop_action_compression_deterministic(table in action_table_strategy(6, 6)) {
        let c1 = compress_action_table(&table);
        let c2 = compress_action_table(&table);
        prop_assert_eq!(c1.unique_rows, c2.unique_rows);
        prop_assert_eq!(c1.state_to_row, c2.state_to_row);
    }
}

proptest! {
    #[test]
    fn prop_goto_compression_deterministic(table in goto_table_strategy(6, 6)) {
        let c1 = compress_goto_table(&table);
        let c2 = compress_goto_table(&table);
        prop_assert_eq!(c1.entries, c2.entries);
    }
}

#[test]
fn test_table_compressor_deterministic() {
    let pt = table_with_shift_in_s0(
        3,
        2,
        vec![
            (0, 2, Action::Reduce(RuleId(1))),
            (1, 1, Action::Shift(StateId(2))),
        ],
    );
    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&pt.grammar, &pt);

    let c1 = compressor.compress(&pt, &token_indices, false).unwrap();
    let c2 = compressor.compress(&pt, &token_indices, false).unwrap();

    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
    assert_eq!(c1.action_table.row_offsets, c2.action_table.row_offsets);
    assert_eq!(c1.goto_table.row_offsets, c2.goto_table.row_offsets);
    assert_eq!(c1.goto_table.data.len(), c2.goto_table.data.len());
}

// ═════════════════════════════════════════════════════════════════════════
// Additional edge-case and cross-cutting tests
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_bitpacked_roundtrip_all_errors() {
    let table = vec![
        vec![Action::Error, Action::Error, Action::Error],
        vec![Action::Error, Action::Error, Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    for s in 0..table.len() {
        for sym in 0..table[0].len() {
            assert_eq!(packed.decompress(s, sym), table[s][sym].clone());
        }
    }
}

#[test]
fn test_bitpacked_all_shifts_roundtrip() {
    // BitPacked decompress works correctly for uniform action types
    let table = vec![
        vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))],
        vec![Action::Shift(StateId(3)), Action::Shift(StateId(4))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    for s in 0..table.len() {
        for sym in 0..table[0].len() {
            assert_eq!(packed.decompress(s, sym), table[s][sym].clone());
        }
    }
}

proptest! {
    #[test]
    fn prop_bitpacked_error_roundtrip(
        states in 1usize..8,
        symbols in 1usize..8,
    ) {
        // BitPacked roundtrip works for uniform error tables
        let table = vec![vec![Action::Error; symbols]; states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Error);
            }
        }
    }
}

#[test]
fn test_uniform_action_table_compresses_to_one_row() {
    let row = vec![vec![Action::Error]; 5];
    let table = vec![row; 4];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn test_all_distinct_rows_preserved() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Error]],
        vec![vec![Action::Error], vec![Action::Shift(StateId(2))]],
        vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 3);
}

#[test]
fn test_compressed_tables_validate_ok() {
    let pt = table_with_shift_in_s0(2, 2, vec![]);
    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&pt.grammar, &pt);
    let compressed = compressor.compress(&pt, &token_indices, false).unwrap();
    assert!(compressed.validate(&pt).is_ok());
}

#[test]
fn test_encode_action_small_shift() {
    let compressor = TableCompressor::new();
    let encoded = compressor
        .encode_action_small(&Action::Shift(StateId(42)))
        .unwrap();
    assert_eq!(encoded, 42);
}

#[test]
fn test_encode_action_small_reduce() {
    let compressor = TableCompressor::new();
    let encoded = compressor
        .encode_action_small(&Action::Reduce(RuleId(0)))
        .unwrap();
    // Reduce: high bit set, 1-based rule id
    assert_eq!(encoded, 0x8000 | 1);
}

#[test]
fn test_encode_action_small_accept() {
    let compressor = TableCompressor::new();
    let encoded = compressor.encode_action_small(&Action::Accept).unwrap();
    assert_eq!(encoded, 0xFFFF);
}

#[test]
fn test_encode_action_small_error() {
    let compressor = TableCompressor::new();
    let encoded = compressor.encode_action_small(&Action::Error).unwrap();
    assert_eq!(encoded, 0xFFFE);
}

#[test]
fn test_compressed_parse_table_zero_sizes() {
    let cpt = CompressedParseTable::new_for_testing(0, 0);
    assert_eq!(cpt.symbol_count(), 0);
    assert_eq!(cpt.state_count(), 0);
}
