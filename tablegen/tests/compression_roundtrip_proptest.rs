#![allow(clippy::needless_range_loop)]
//! Property-based and unit tests for compression roundtrip correctness.
//!
//! Covers:
//! 1.  Action table row deduplication roundtrip
//! 2.  Goto table sparse compression roundtrip
//! 3.  BitPacked action table roundtrip (uniform action types)
//! 4.  Encode/decode small action encoding for all action variants
//! 5.  TableCompressor compress_action_table_small roundtrip invariants
//! 6.  TableCompressor compress_goto_table_small run-length invariants
//! 7.  Full pipeline compression roundtrip via GrammarBuilder
//! 8.  Edge cases: empty, single-entry, large tables
//! 9.  Bit packing error mask correctness
//! 10. Encoding boundary values

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId, Token, TokenPattern};
use adze_tablegen::collect_token_indices;
use adze_tablegen::compress::{
    CompressedActionTable, CompressedGotoEntry, CompressedParseTable, TableCompressor,
};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use adze_tablegen::eof_accepts_or_reduces;
use proptest::prelude::*;
use std::collections::BTreeMap;

// ── Helpers ─────────────────────────────────────────────────────────────────

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable from raw components (integration-test safe).
fn make_parse_table(
    mut actions: Vec<Vec<Vec<Action>>>,
    mut gotos: Vec<Vec<StateId>>,
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
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol[i] = SymbolId(i as u16);
    }
    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index
        .entry(start_symbol)
        .or_insert(start_symbol.0 as usize);

    let eof_idx = eof_symbol.0 as usize;
    let token_count = eof_idx;

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![],
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
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
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

/// Build a ParseTable with at least one token shift in state 0 so that
/// `TableCompressor::compress` passes its state-0 validation.
fn table_with_shift_in_s0(
    num_states: usize,
    num_terms: usize,
    extra_actions: Vec<(usize, usize, Action)>,
) -> ParseTable {
    let num_states = num_states.max(2);
    let num_terms = num_terms.max(1);
    let eof_idx = num_terms + 1;
    let start_nt = eof_idx + 1;
    let symbol_count = start_nt + 1;

    let mut actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; num_states];
    // Place a shift at column 1 (first terminal) in state 0
    actions[0][1] = vec![Action::Shift(StateId(1))];

    for (s, sym, act) in extra_actions {
        if s < num_states && sym < symbol_count {
            actions[s][sym].push(act);
        }
    }

    let gotos = vec![vec![INVALID; symbol_count]; num_states];

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
        SymbolId(start_nt as u16),
        SymbolId(eof_idx as u16),
    );
    pt.grammar = grammar;
    pt
}

/// Compress a table that has shift in state 0 using TableCompressor.
fn compress_table(pt: &ParseTable) -> adze_tablegen::CompressedTables {
    let compressor = TableCompressor::new();
    let token_indices = collect_token_indices(&pt.grammar, pt);
    let start_empty = eof_accepts_or_reduces(pt);
    compressor
        .compress(pt, &token_indices, start_empty)
        .expect("compression must succeed")
}

// ── Strategies ──────────────────────────────────────────────────────────────

fn action_strategy() -> impl Strategy<Value = Action> {
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

fn flat_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

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

// ═══════════════════════════════════════════════════════════════════════════
// 1. Action table row-dedup roundtrip (property tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_action_dedup_roundtrip_preserves_all_cells(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                let got = decompress_action(&compressed, state, sym);
                prop_assert_eq!(got, expected, "state={} sym={}", state, sym);
            }
        }
    }

    #[test]
    fn prop_action_dedup_unique_rows_le_total(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);
        prop_assert!(compressed.unique_rows.len() <= table.len());
    }

    #[test]
    fn prop_action_dedup_state_to_row_len_matches(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.state_to_row.len(), table.len());
    }

    #[test]
    fn prop_action_dedup_column_count_preserved(table in action_table_strategy(6, 10)) {
        let compressed = compress_action_table(&table);
        if let Some(first_row) = table.first() {
            for unique_row in &compressed.unique_rows {
                prop_assert_eq!(unique_row.len(), first_row.len());
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Goto table sparse roundtrip (property tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_goto_sparse_roundtrip_all_cells(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, &expected) in row.iter().enumerate() {
                let got = decompress_goto(&compressed, state, sym);
                prop_assert_eq!(got, expected, "state={} sym={}", state, sym);
            }
        }
    }

    #[test]
    fn prop_goto_sparse_entry_count_equals_some_count(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        let some_count: usize = table.iter().flat_map(|r| r.iter()).filter(|v| v.is_some()).count();
        prop_assert_eq!(compressed.entries.len(), some_count);
    }

    #[test]
    fn prop_goto_sparse_none_returns_none(table in goto_table_strategy(6, 6)) {
        let compressed = compress_goto_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, val) in row.iter().enumerate() {
                if val.is_none() {
                    prop_assert_eq!(decompress_goto(&compressed, state, sym), None);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. BitPacked action table roundtrip
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_bitpacked_all_errors_roundtrip(states in 1usize..8, symbols in 1usize..8) {
        let table = vec![vec![Action::Error; symbols]; states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Error);
            }
        }
    }

    #[test]
    fn prop_bitpacked_all_shifts_roundtrip(states in 1usize..5, symbols in 1usize..5) {
        let table: Vec<Vec<Action>> = (0..states)
            .map(|s| {
                (0..symbols)
                    .map(|sym| Action::Shift(StateId(((s * symbols + sym) % 100) as u16)))
                    .collect()
            })
            .collect();
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), table[s][sym].clone());
            }
        }
    }
}

#[test]
fn bitpacked_single_accept_roundtrip() {
    let table = vec![vec![Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    // Accept is stored as special reduce (u32::MAX)
    let result = packed.decompress(0, 0);
    assert_eq!(result, Action::Accept);
}

#[test]
fn bitpacked_mixed_error_and_shift() {
    let table = vec![
        vec![Action::Error, Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2)), Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(1, 0), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(1, 1), Action::Error);
}

#[test]
fn bitpacked_error_mask_bits_set_correctly() {
    let table = vec![vec![
        Action::Error,
        Action::Shift(StateId(1)),
        Action::Error,
        Action::Shift(StateId(2)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 3), Action::Shift(StateId(2)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Encode/decode small action encoding
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn encode_shift_zero() {
    let c = TableCompressor::new();
    assert_eq!(
        c.encode_action_small(&Action::Shift(StateId(0))).unwrap(),
        0
    );
}

#[test]
fn encode_shift_one() {
    let c = TableCompressor::new();
    assert_eq!(
        c.encode_action_small(&Action::Shift(StateId(1))).unwrap(),
        1
    );
}

#[test]
fn encode_shift_max_valid() {
    let c = TableCompressor::new();
    assert_eq!(
        c.encode_action_small(&Action::Shift(StateId(0x7FFF)))
            .unwrap(),
        0x7FFF
    );
}

#[test]
fn encode_shift_overflow_fails() {
    let c = TableCompressor::new();
    assert!(
        c.encode_action_small(&Action::Shift(StateId(0x8000)))
            .is_err()
    );
}

#[test]
fn encode_reduce_zero() {
    let c = TableCompressor::new();
    // Reduce(0) → 0x8000 | (0 + 1) = 0x8001
    assert_eq!(
        c.encode_action_small(&Action::Reduce(RuleId(0))).unwrap(),
        0x8001
    );
}

#[test]
fn encode_reduce_max_valid() {
    let c = TableCompressor::new();
    // Reduce(0x3FFF) → 0x8000 | (0x3FFF + 1) = 0x8000 | 0x4000 = 0xC000
    assert_eq!(
        c.encode_action_small(&Action::Reduce(RuleId(0x3FFF)))
            .unwrap(),
        0xC000
    );
}

#[test]
fn encode_reduce_overflow_fails() {
    let c = TableCompressor::new();
    assert!(
        c.encode_action_small(&Action::Reduce(RuleId(0x4000)))
            .is_err()
    );
}

#[test]
fn encode_accept_is_0xffff() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Accept).unwrap(), 0xFFFF);
}

#[test]
fn encode_error_is_0xfffe() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Error).unwrap(), 0xFFFE);
}

#[test]
fn encode_recover_is_0xfffd() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Recover).unwrap(), 0xFFFD);
}

#[test]
fn encode_fork_maps_to_error() {
    let c = TableCompressor::new();
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    // Fork actions are mapped to error encoding
    assert_eq!(c.encode_action_small(&fork).unwrap(), 0xFFFE);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn prop_encode_shift_preserves_state(state in 0u16..0x8000) {
        let c = TableCompressor::new();
        let encoded = c.encode_action_small(&Action::Shift(StateId(state))).unwrap();
        prop_assert_eq!(encoded, state);
    }

    #[test]
    fn prop_encode_reduce_has_high_bit(rule in 0u16..0x4000) {
        let c = TableCompressor::new();
        let encoded = c.encode_action_small(&Action::Reduce(RuleId(rule))).unwrap();
        prop_assert!(encoded & 0x8000 != 0, "reduce must have high bit set");
        prop_assert_eq!(encoded & 0x7FFF, rule + 1, "1-based rule id");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. TableCompressor compress_action_table_small invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn small_action_empty_table_produces_correct_offsets() {
    let c = TableCompressor::new();
    let table: Vec<Vec<Vec<Action>>> = vec![vec![]; 4];
    let result = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    assert_eq!(result.row_offsets.len(), 5); // 4 states + 1
    assert_eq!(result.default_actions.len(), 4);
    assert!(result.data.is_empty());
}

#[test]
fn small_action_single_shift_entry() {
    let c = TableCompressor::new();
    let table = vec![vec![vec![Action::Shift(StateId(5))]]];
    let result = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].symbol, 0);
    assert_eq!(result.data[0].action, Action::Shift(StateId(5)));
}

#[test]
fn small_action_error_cells_not_stored() {
    let c = TableCompressor::new();
    // 5 columns: only col 2 has a non-error action
    let table = vec![
        (0..5)
            .map(|i| {
                if i == 2 {
                    vec![Action::Reduce(RuleId(0))]
                } else {
                    vec![]
                }
            })
            .collect(),
    ];
    let result = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    assert_eq!(result.data.len(), 1);
}

#[test]
fn small_action_explicit_error_actions_not_stored() {
    let c = TableCompressor::new();
    // Cell containing explicit Action::Error should be skipped
    let table = vec![vec![vec![Action::Error], vec![Action::Shift(StateId(1))]]];
    let result = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].action, Action::Shift(StateId(1)));
}

#[test]
fn small_action_multi_action_cell_stores_all() {
    let c = TableCompressor::new();
    let table = vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ]]];
    let result = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    // Both actions in the GLR cell should be stored
    assert_eq!(result.data.len(), 2);
}

#[test]
fn small_action_row_offsets_nondecreasing() {
    let c = TableCompressor::new();
    let table = vec![
        vec![vec![Action::Shift(StateId(0))]; 5],
        vec![vec![]; 5],
        vec![vec![Action::Reduce(RuleId(1))]; 5],
    ];
    let result = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    for pair in result.row_offsets.windows(2) {
        assert!(pair[1] >= pair[0], "offsets must be non-decreasing");
    }
}

#[test]
fn small_action_default_actions_always_error() {
    let c = TableCompressor::new();
    let table = vec![
        vec![vec![Action::Reduce(RuleId(0))]; 10],
        vec![vec![Action::Shift(StateId(1))]; 10],
        vec![vec![Action::Accept]; 10],
    ];
    let result = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    for d in &result.default_actions {
        assert_eq!(*d, Action::Error, "default optimization disabled");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_small_action_row_offsets_len(table in action_table_strategy(6, 6)) {
        let c = TableCompressor::new();
        let result = c.compress_action_table_small(&table, &BTreeMap::new()).unwrap();
        prop_assert_eq!(result.row_offsets.len(), table.len() + 1);
    }

    #[test]
    fn prop_small_action_default_actions_len(table in action_table_strategy(6, 6)) {
        let c = TableCompressor::new();
        let result = c.compress_action_table_small(&table, &BTreeMap::new()).unwrap();
        prop_assert_eq!(result.default_actions.len(), table.len());
    }

    #[test]
    fn prop_small_action_offsets_nondecreasing(table in action_table_strategy(8, 8)) {
        let c = TableCompressor::new();
        let result = c.compress_action_table_small(&table, &BTreeMap::new()).unwrap();
        for pair in result.row_offsets.windows(2) {
            prop_assert!(pair[1] >= pair[0]);
        }
    }

    #[test]
    fn prop_small_action_last_offset_equals_data_len(table in action_table_strategy(6, 6)) {
        let c = TableCompressor::new();
        let result = c.compress_action_table_small(&table, &BTreeMap::new()).unwrap();
        prop_assert_eq!(*result.row_offsets.last().unwrap(), result.data.len() as u16);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. TableCompressor compress_goto_table_small run-length invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_rle_run_of_3_uses_run_length() {
    let c = TableCompressor::new();
    let table = vec![vec![StateId(7), StateId(7), StateId(7)]];
    let result = c.compress_goto_table_small(&table).unwrap();
    let has_rl = result
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 7, count: 3 }));
    assert!(has_rl);
}

#[test]
fn goto_rle_run_of_2_uses_singles() {
    let c = TableCompressor::new();
    let table = vec![vec![StateId(5), StateId(5)]];
    let result = c.compress_goto_table_small(&table).unwrap();
    let all_single = result
        .data
        .iter()
        .all(|e| matches!(e, CompressedGotoEntry::Single(5)));
    assert!(all_single);
}

#[test]
fn goto_rle_run_of_1_uses_single() {
    let c = TableCompressor::new();
    let table = vec![vec![StateId(3)]];
    let result = c.compress_goto_table_small(&table).unwrap();
    assert_eq!(result.data.len(), 1);
    assert!(matches!(result.data[0], CompressedGotoEntry::Single(3)));
}

#[test]
fn goto_rle_alternating_no_run_length() {
    let c = TableCompressor::new();
    let table = vec![vec![
        StateId(1),
        StateId(2),
        StateId(1),
        StateId(2),
        StateId(1),
    ]];
    let result = c.compress_goto_table_small(&table).unwrap();
    let all_single = result
        .data
        .iter()
        .all(|e| matches!(e, CompressedGotoEntry::Single(_)));
    assert!(all_single);
}

#[test]
fn goto_rle_long_run_uses_run_length() {
    let c = TableCompressor::new();
    let table = vec![vec![StateId(42); 100]];
    let result = c.compress_goto_table_small(&table).unwrap();
    let has_rl = result.data.iter().any(|e| {
        matches!(
            e,
            CompressedGotoEntry::RunLength {
                state: 42,
                count: 100
            }
        )
    });
    assert!(has_rl);
}

#[test]
fn goto_rle_multiple_runs_in_one_row() {
    let c = TableCompressor::new();
    let table = vec![vec![
        StateId(1),
        StateId(1),
        StateId(1),
        StateId(2),
        StateId(2),
        StateId(2),
    ]];
    let result = c.compress_goto_table_small(&table).unwrap();
    let rl_count = result
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::RunLength { .. }))
        .count();
    assert_eq!(rl_count, 2, "should have two RunLength entries");
}

#[test]
fn goto_rle_empty_table() {
    let c = TableCompressor::new();
    let table: Vec<Vec<StateId>> = vec![];
    let result = c.compress_goto_table_small(&table).unwrap();
    assert_eq!(result.row_offsets.len(), 1);
    assert!(result.data.is_empty());
}

#[test]
fn goto_rle_empty_rows() {
    let c = TableCompressor::new();
    let table: Vec<Vec<StateId>> = vec![vec![], vec![], vec![]];
    let result = c.compress_goto_table_small(&table).unwrap();
    assert_eq!(result.row_offsets.len(), 4);
    assert!(result.data.is_empty());
}

#[test]
fn goto_rle_row_offsets_nondecreasing() {
    let c = TableCompressor::new();
    let table = vec![
        vec![StateId(1), StateId(1), StateId(1)],
        vec![StateId(2)],
        vec![StateId(3), StateId(4)],
    ];
    let result = c.compress_goto_table_small(&table).unwrap();
    for pair in result.row_offsets.windows(2) {
        assert!(pair[1] >= pair[0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Full pipeline compression roundtrip via GrammarBuilder
// ═══════════════════════════════════════════════════════════════════════════

fn pipeline(grammar_fn: impl FnOnce() -> Grammar) -> (ParseTable, adze_tablegen::CompressedTables) {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    use adze_ir::builder::GrammarBuilder;

    let mut grammar = grammar_fn();
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton construction failed");
    let token_indices = collect_token_indices(&grammar, &table);
    let start_empty = eof_accepts_or_reduces(&table);
    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &token_indices, start_empty)
        .expect("Table compression failed");
    (table, compressed)
}

#[test]
fn pipeline_single_token_grammar() {
    use adze_ir::builder::GrammarBuilder;
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn pipeline_two_alternatives() {
    use adze_ir::builder::GrammarBuilder;
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn pipeline_sequence_grammar() {
    use adze_ir::builder::GrammarBuilder;
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn pipeline_chain_grammar() {
    use adze_ir::builder::GrammarBuilder;
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("x", "x")
            .rule("c", vec!["x"])
            .rule("b", vec!["c"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn pipeline_left_recursive_grammar() {
    use adze_ir::builder::GrammarBuilder;
    let (pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list")
            .build()
    });
    assert!(pt.state_count >= 3);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn pipeline_validates_ok() {
    use adze_ir::builder::GrammarBuilder;
    let (pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build()
    });
    assert!(compressed.validate(&pt).is_ok());
}

#[test]
fn pipeline_compressed_metadata_matches() {
    use adze_ir::builder::GrammarBuilder;
    let (pt, _compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build()
    });
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
    assert_eq!(cpt.state_count(), pt.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Edge cases: empty, single-entry, large tables
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn edge_single_state_single_symbol_action() {
    let table = vec![vec![vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
}

#[test]
fn edge_single_state_single_symbol_goto() {
    let table = vec![vec![Some(StateId(0))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(0)));
}

#[test]
fn edge_all_error_action_table() {
    let table = vec![vec![vec![]; 10]; 5];
    let compressed = compress_action_table(&table);
    for state in 0..5 {
        for sym in 0..10 {
            assert_eq!(decompress_action(&compressed, state, sym), Action::Error);
        }
    }
}

#[test]
fn edge_all_none_goto_table() {
    let table = vec![vec![None; 8]; 4];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
    for state in 0..4 {
        for sym in 0..8 {
            assert_eq!(decompress_goto(&compressed, state, sym), None);
        }
    }
}

#[test]
fn edge_large_action_table_roundtrip() {
    let n_states = 50;
    let n_syms = 20;
    let table: Vec<Vec<Vec<Action>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| match (s + sym) % 5 {
                    0 => vec![Action::Shift(StateId(((s + sym) % 30) as u16))],
                    1 => vec![Action::Reduce(RuleId(((s * sym) % 15) as u16))],
                    2 => vec![Action::Accept],
                    3 => vec![],
                    _ => vec![Action::Error],
                })
                .collect()
        })
        .collect();
    let compressed = compress_action_table(&table);
    for (state, row) in table.iter().enumerate() {
        for (sym, cell) in row.iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            assert_eq!(
                decompress_action(&compressed, state, sym),
                expected,
                "state={state} sym={sym}"
            );
        }
    }
}

#[test]
fn edge_large_goto_table_roundtrip() {
    let n_states = 30;
    let n_syms = 15;
    let table: Vec<Vec<Option<StateId>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| {
                    if (s + sym) % 3 == 0 {
                        Some(StateId(((s + sym) % 20) as u16))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    let compressed = compress_goto_table(&table);
    for (state, row) in table.iter().enumerate() {
        for (sym, &expected) in row.iter().enumerate() {
            assert_eq!(
                decompress_goto(&compressed, state, sym),
                expected,
                "state={state} sym={sym}"
            );
        }
    }
}

#[test]
fn edge_multi_action_cell_first_action_returned() {
    let table = vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
}

#[test]
fn edge_empty_cell_returns_error() {
    let table = vec![vec![vec![]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Compression determinism
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_action_dedup_deterministic(table in action_table_strategy(6, 6)) {
        let c1 = compress_action_table(&table);
        let c2 = compress_action_table(&table);
        prop_assert_eq!(c1.unique_rows, c2.unique_rows);
        prop_assert_eq!(c1.state_to_row, c2.state_to_row);
    }

    #[test]
    fn prop_goto_sparse_deterministic(table in goto_table_strategy(6, 6)) {
        let c1 = compress_goto_table(&table);
        let c2 = compress_goto_table(&table);
        prop_assert_eq!(c1.entries, c2.entries);
    }
}

#[test]
fn deterministic_small_table_compressor() {
    let c = TableCompressor::new();
    let table = vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![],
            vec![Action::Accept],
        ],
        vec![vec![Action::Reduce(RuleId(0))], vec![], vec![]],
    ];
    let c1 = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    let c2 = c
        .compress_action_table_small(&table, &BTreeMap::new())
        .unwrap();
    assert_eq!(c1.row_offsets, c2.row_offsets);
    assert_eq!(c1.default_actions, c2.default_actions);
    assert_eq!(c1.data.len(), c2.data.len());
    for (e1, e2) in c1.data.iter().zip(c2.data.iter()) {
        assert_eq!(e1.symbol, e2.symbol);
        assert_eq!(e1.action, e2.action);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. TableCompressor::compress validation edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_rejects_empty_action_table() {
    let mut pt = table_with_shift_in_s0(2, 1, vec![]);
    pt.action_table.clear();
    pt.state_count = 0;
    let c = TableCompressor::new();
    let token_indices = collect_token_indices(&pt.grammar, &pt);
    let result = c.compress(&pt, &token_indices, false);
    assert!(result.is_err());
}

#[test]
fn compress_requires_token_shift_in_state0() {
    let num_terms = 1;
    let eof_idx = num_terms + 1;
    let start_nt = eof_idx + 1;
    let symbol_count = start_nt + 1;

    // Table with no shift in state 0
    let actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; 2];
    let gotos = vec![vec![INVALID; symbol_count]; 2];

    let mut grammar = Grammar::default();
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "t1".to_string(),
            pattern: TokenPattern::String("t1".to_string()),
            fragile: false,
        },
    );

    let mut pt = make_parse_table(
        actions,
        gotos,
        SymbolId(start_nt as u16),
        SymbolId(eof_idx as u16),
    );
    pt.grammar = grammar;

    let c = TableCompressor::new();
    let token_indices = collect_token_indices(&pt.grammar, &pt);
    let result = c.compress(&pt, &token_indices, false);
    assert!(result.is_err());
}

#[test]
fn compress_accepts_nullable_start_with_eof_accept() {
    let num_terms = 1;
    let eof_idx = num_terms + 1;
    let start_nt = eof_idx + 1;
    let symbol_count = start_nt + 1;

    let mut actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; 2];
    // Put Accept on the EOF column in state 0
    actions[0][eof_idx] = vec![Action::Accept];
    let gotos = vec![vec![INVALID; symbol_count]; 2];

    let mut grammar = Grammar::default();
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "t1".to_string(),
            pattern: TokenPattern::String("t1".to_string()),
            fragile: false,
        },
    );

    let mut pt = make_parse_table(
        actions,
        gotos,
        SymbolId(start_nt as u16),
        SymbolId(eof_idx as u16),
    );
    pt.grammar = grammar;

    let c = TableCompressor::new();
    let token_indices = collect_token_indices(&pt.grammar, &pt);
    // start_can_be_empty = true should allow the nullable case
    let result = c.compress(&pt, &token_indices, true);
    assert!(result.is_ok());
}

#[test]
fn compress_full_pipeline_table_compressor_row_offsets() {
    let pt = table_with_shift_in_s0(4, 3, vec![(1, 2, Action::Reduce(RuleId(0)))]);
    let compressed = compress_table(&pt);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
    assert_eq!(
        compressed.action_table.default_actions.len(),
        pt.state_count
    );
}

#[test]
fn compress_full_pipeline_goto_row_offsets() {
    let pt = table_with_shift_in_s0(4, 3, vec![]);
    let compressed = compress_table(&pt);
    for pair in compressed.goto_table.row_offsets.windows(2) {
        assert!(pair[1] >= pair[0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. CompressedParseTable unit tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_parse_table_new_for_testing() {
    let cpt = CompressedParseTable::new_for_testing(100, 200);
    assert_eq!(cpt.symbol_count(), 100);
    assert_eq!(cpt.state_count(), 200);
}

#[test]
fn compressed_parse_table_zero_sizes() {
    let cpt = CompressedParseTable::new_for_testing(0, 0);
    assert_eq!(cpt.symbol_count(), 0);
    assert_eq!(cpt.state_count(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Identical rows deduplication efficiency
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn dedup_4_identical_rows_gives_1_unique() {
    let row = vec![
        vec![Action::Shift(StateId(5))],
        vec![Action::Reduce(RuleId(1))],
    ];
    let table = vec![row.clone(), row.clone(), row.clone(), row];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 4);
}

#[test]
fn dedup_all_distinct_preserved() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Error]],
        vec![vec![Action::Error], vec![Action::Shift(StateId(2))]],
        vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 3);
}

#[test]
fn dedup_mixed_identical_and_distinct() {
    let row_a = vec![vec![Action::Shift(StateId(1))], vec![Action::Error]];
    let row_b = vec![vec![Action::Error], vec![Action::Accept]];
    let table = vec![row_a.clone(), row_b.clone(), row_a, row_b];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
    assert_eq!(compressed.state_to_row.len(), 4);
    // First and third rows should map to same unique row
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[2]);
    assert_eq!(compressed.state_to_row[1], compressed.state_to_row[3]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Sparse goto compression efficiency
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sparse_goto_fewer_entries_than_cells() {
    let n_states = 10;
    let n_syms = 10;
    let table: Vec<Vec<Option<StateId>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| {
                    if (s + sym) % 7 == 0 {
                        Some(StateId(1))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.len() < n_states * n_syms);
}

#[test]
fn sparse_goto_fully_populated() {
    let n_states = 3;
    let n_syms = 3;
    let table: Vec<Vec<Option<StateId>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| Some(StateId(((s + sym) % 5) as u16)))
                .collect()
        })
        .collect();
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), n_states * n_syms);
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Wide symbol count
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wide_symbol_count_100_columns() {
    let n_syms = 100;
    let table: Vec<Vec<Vec<Action>>> = vec![
        (0..n_syms)
            .map(|sym| {
                if sym % 10 == 0 {
                    vec![Action::Shift(StateId((sym % 30) as u16))]
                } else {
                    vec![]
                }
            })
            .collect(),
    ];
    let compressed = compress_action_table(&table);
    for sym in 0..n_syms {
        let expected = if sym % 10 == 0 {
            Action::Shift(StateId((sym % 30) as u16))
        } else {
            Action::Error
        };
        assert_eq!(decompress_action(&compressed, 0, sym), expected);
    }
}
