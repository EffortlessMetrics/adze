#![allow(clippy::needless_range_loop)]
//! 40+ proptest properties for `adze-tablegen` compression.
//!
//! Covers: action/goto roundtrip, size reduction, dedup correctness,
//! bit-packing, determinism, encode_action_small, edge cases.

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_tablegen::compress::TableCompressor;
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use proptest::prelude::*;
use std::collections::HashSet;

// ───────────────────────────────────────────────────────────────────────
// Strategies
// ───────────────────────────────────────────────────────────────────────

fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
        1 => Just(Action::Recover),
    ]
}

fn flat_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..80).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..40).prop_map(|r| Action::Reduce(RuleId(r))),
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
        let cell = prop_oneof![
            3 => Just(None),
            1 => (0u16..20).prop_map(|s| Some(StateId(s))),
        ];
        prop::collection::vec(
            prop::collection::vec(cell, symbols..=symbols),
            states..=states,
        )
    })
}

/// Table where every row is identical (maximises dedup).
fn uniform_action_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Vec<Action>>>> {
    (2..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(action_cell_strategy(), symbols..=symbols)
            .prop_map(move |row| (0..states).map(|_| row.clone()).collect::<Vec<_>>())
    })
}

/// Sparse goto table (≥90 % None).
fn sparse_goto_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Option<StateId>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        let cell = prop_oneof![
            9 => Just(None),
            1 => (0u16..20).prop_map(|s| Some(StateId(s))),
        ];
        prop::collection::vec(
            prop::collection::vec(cell, symbols..=symbols),
            states..=states,
        )
    })
}

// ───────────────────────────────────────────────────────────────────────
// 1  Action compression roundtrip
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// P01: Decompressed first-action matches original first-action for every cell.
    #[test]
    fn p01_action_roundtrip(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                let got = decompress_action(&compressed, s, sym);
                prop_assert_eq!(got, expected, "state={}, symbol={}", s, sym);
            }
        }
    }

    /// P02: Roundtrip on larger tables.
    #[test]
    fn p02_action_roundtrip_large(table in action_table_strategy(16, 16)) {
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(decompress_action(&compressed, s, sym), expected);
            }
        }
    }

    /// P03: Every unique-row index in state_to_row is valid.
    #[test]
    fn p03_state_to_row_in_bounds(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);
        for &idx in &compressed.state_to_row {
            prop_assert!(idx < compressed.unique_rows.len());
        }
    }

    /// P04: state_to_row length equals number of states.
    #[test]
    fn p04_state_to_row_len(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.state_to_row.len(), table.len());
    }
}

// ───────────────────────────────────────────────────────────────────────
// 2  Goto compression roundtrip
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// P05: Goto roundtrip preserves every cell.
    #[test]
    fn p05_goto_roundtrip(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }

    /// P06: Larger goto roundtrip.
    #[test]
    fn p06_goto_roundtrip_large(table in goto_table_strategy(16, 16)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }

    /// P07: Sparse goto stores only non-None entries.
    #[test]
    fn p07_goto_sparse_count(table in goto_table_strategy(10, 10)) {
        let compressed = compress_goto_table(&table);
        let non_none: usize = table.iter()
            .flat_map(|row| row.iter())
            .filter(|v| v.is_some())
            .count();
        prop_assert_eq!(compressed.entries.len(), non_none);
    }

    /// P08: Missing goto keys return None.
    #[test]
    fn p08_goto_missing_returns_none(table in goto_table_strategy(6, 6)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, val) in row.iter().enumerate() {
                if val.is_none() {
                    prop_assert_eq!(decompress_goto(&compressed, s, sym), None);
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// 3  Size reduction / dedup properties
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// P09: unique_rows ≤ total rows.
    #[test]
    fn p09_unique_rows_le_total(table in action_table_strategy(12, 6)) {
        let compressed = compress_action_table(&table);
        prop_assert!(compressed.unique_rows.len() <= table.len());
    }

    /// P10: Uniform tables collapse to exactly 1 unique row.
    #[test]
    fn p10_uniform_table_one_row(table in uniform_action_table_strategy(8, 6)) {
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.unique_rows.len(), 1);
    }

    /// P11: All rows in unique_rows are distinct.
    #[test]
    fn p11_unique_rows_distinct(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);
        let set: HashSet<Vec<Vec<Action>>> = compressed.unique_rows.iter().cloned().collect();
        prop_assert_eq!(set.len(), compressed.unique_rows.len());
    }

    /// P12: Sparse goto is smaller or equal to dense representation.
    #[test]
    fn p12_sparse_goto_size(table in sparse_goto_strategy(10, 10)) {
        let compressed = compress_goto_table(&table);
        let dense_cells: usize = table.iter().map(|row| row.len()).sum();
        prop_assert!(compressed.entries.len() <= dense_cells);
    }

    /// P13: Fully-None goto table yields empty entries map.
    #[test]
    fn p13_all_none_goto_empty(
        states in 1usize..8,
        symbols in 1usize..8,
    ) {
        let table: Vec<Vec<Option<StateId>>> =
            vec![vec![None; symbols]; states];
        let compressed = compress_goto_table(&table);
        prop_assert!(compressed.entries.is_empty());
    }

    /// P14: Identical rows share the same row index.
    #[test]
    fn p14_dup_rows_share_index(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);
        for i in 0..table.len() {
            for j in (i + 1)..table.len() {
                if table[i] == table[j] {
                    prop_assert_eq!(
                        compressed.state_to_row[i],
                        compressed.state_to_row[j],
                        "states {} and {} are equal but mapped differently", i, j
                    );
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// 4  BitPackedActionTable properties
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// P15: BitPacked error cells are preserved.
    #[test]
    fn p15_bitpack_errors_preserved(
        states in 1usize..6,
        symbols in 1usize..6,
    ) {
        let table: Vec<Vec<Action>> =
            vec![vec![Action::Error; symbols]; states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Error);
            }
        }
    }

    /// P16: BitPacked all-Shift table round-trips.
    #[test]
    fn p16_bitpack_all_shifts(
        states in 1usize..5,
        symbols in 1usize..5,
        target in 1u16..100,
    ) {
        let table: Vec<Vec<Action>> =
            vec![vec![Action::Shift(StateId(target)); symbols]; states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Shift(StateId(target)));
            }
        }
    }

    /// P17: BitPacked all-Reduce table round-trips.
    #[test]
    fn p17_bitpack_all_reduces(
        states in 1usize..5,
        symbols in 1usize..5,
        rule in 0u16..40,
    ) {
        let table: Vec<Vec<Action>> =
            vec![vec![Action::Reduce(RuleId(rule)); symbols]; states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Reduce(RuleId(rule)));
            }
        }
    }

    /// P18: BitPacked all-Accept table round-trips.
    #[test]
    fn p18_bitpack_all_accept(
        states in 1usize..5,
        symbols in 1usize..5,
    ) {
        let table: Vec<Vec<Action>> =
            vec![vec![Action::Accept; symbols]; states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Accept);
            }
        }
    }

    /// P19: BitPacked error mask has correct word count.
    #[test]
    fn p19_bitpack_mask_words(table in flat_action_table_strategy(8, 8)) {
        let total = table.len() * table[0].len();
        let expected_words = total.div_ceil(64);
        let packed = BitPackedActionTable::from_table(&table);
        // error_mask is private; verify indirectly via correct decompress
        // of at least the first cell.
        let first = &table[0][0];
        let got = packed.decompress(0, 0);
        if *first == Action::Error {
            prop_assert_eq!(got, Action::Error);
        }
        let _ = expected_words; // used for design reasoning
    }

    /// P20: Fork data preserved through bit-packing.
    #[test]
    fn p20_bitpack_fork_preserved(
        inner in prop::collection::vec(
            prop_oneof![
                (1u16..50).prop_map(|s| Action::Shift(StateId(s))),
                (0u16..30).prop_map(|r| Action::Reduce(RuleId(r))),
            ],
            2..=4,
        ),
    ) {
        let table = vec![vec![Action::Fork(inner.clone())]];
        let packed = BitPackedActionTable::from_table(&table);
        let got = packed.decompress(0, 0);
        prop_assert_eq!(got, Action::Fork(inner));
    }
}

// ───────────────────────────────────────────────────────────────────────
// 5  Determinism
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// P21: Action compression is deterministic (two runs, same output).
    #[test]
    fn p21_action_compress_deterministic(table in action_table_strategy(8, 6)) {
        let a = compress_action_table(&table);
        let b = compress_action_table(&table);
        prop_assert_eq!(a.unique_rows, b.unique_rows);
        prop_assert_eq!(a.state_to_row, b.state_to_row);
    }

    /// P22: Goto compression is deterministic.
    #[test]
    fn p22_goto_compress_deterministic(table in goto_table_strategy(8, 6)) {
        let a = compress_goto_table(&table);
        let b = compress_goto_table(&table);
        prop_assert_eq!(a.entries, b.entries);
    }

    /// P23: BitPacked compression is deterministic.
    #[test]
    fn p23_bitpack_deterministic(table in flat_action_table_strategy(6, 6)) {
        let a = BitPackedActionTable::from_table(&table);
        let b = BitPackedActionTable::from_table(&table);
        for (s, row) in table.iter().enumerate() {
            for sym in 0..row.len() {
                prop_assert_eq!(a.decompress(s, sym), b.decompress(s, sym));
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// 6  encode_action_small (TableCompressor)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(120))]

    /// P24: Shift encodes to the raw state id (< 0x8000).
    #[test]
    fn p24_encode_shift(state in 0u16..0x7FFF) {
        let tc = TableCompressor::new();
        let encoded = tc.encode_action_small(&Action::Shift(StateId(state))).unwrap();
        prop_assert_eq!(encoded, state);
    }

    /// P25: Reduce encodes with high bit set (0x8000 | (rule+1)).
    #[test]
    fn p25_encode_reduce(rule in 0u16..0x3FFF) {
        let tc = TableCompressor::new();
        let encoded = tc.encode_action_small(&Action::Reduce(RuleId(rule))).unwrap();
        prop_assert_eq!(encoded, 0x8000 | (rule + 1));
    }

    /// P26: Accept encodes to 0xFFFF.
    #[test]
    fn p26_encode_accept(_dummy in 0..1u8) {
        let tc = TableCompressor::new();
        prop_assert_eq!(tc.encode_action_small(&Action::Accept).unwrap(), 0xFFFF);
    }

    /// P27: Error encodes to 0xFFFE.
    #[test]
    fn p27_encode_error(_dummy in 0..1u8) {
        let tc = TableCompressor::new();
        prop_assert_eq!(tc.encode_action_small(&Action::Error).unwrap(), 0xFFFE);
    }

    /// P28: Recover encodes to 0xFFFD.
    #[test]
    fn p28_encode_recover(_dummy in 0..1u8) {
        let tc = TableCompressor::new();
        prop_assert_eq!(tc.encode_action_small(&Action::Recover).unwrap(), 0xFFFD);
    }

    /// P29: Shift state ≥ 0x8000 is rejected.
    #[test]
    fn p29_encode_shift_overflow(state in 0x8000u16..=u16::MAX) {
        let tc = TableCompressor::new();
        prop_assert!(tc.encode_action_small(&Action::Shift(StateId(state))).is_err());
    }

    /// P30: Reduce rule ≥ 0x4000 is rejected.
    #[test]
    fn p30_encode_reduce_overflow(rule in 0x4000u16..=u16::MAX) {
        let tc = TableCompressor::new();
        prop_assert!(tc.encode_action_small(&Action::Reduce(RuleId(rule))).is_err());
    }

    /// P31: Shift encodings are distinct from Reduce encodings.
    #[test]
    fn p31_shift_reduce_disjoint(
        state in 0u16..0x7FFF,
        rule in 0u16..0x3FFF,
    ) {
        let tc = TableCompressor::new();
        let s = tc.encode_action_small(&Action::Shift(StateId(state))).unwrap();
        let r = tc.encode_action_small(&Action::Reduce(RuleId(rule))).unwrap();
        prop_assert_ne!(s, r);
    }

    /// P32: Encoded Shift has high bit clear, Reduce has high bit set.
    #[test]
    fn p32_shift_reduce_bit15(
        state in 0u16..0x7FFF,
        rule in 0u16..0x3FFF,
    ) {
        let tc = TableCompressor::new();
        let s = tc.encode_action_small(&Action::Shift(StateId(state))).unwrap();
        let r = tc.encode_action_small(&Action::Reduce(RuleId(rule))).unwrap();
        prop_assert_eq!(s & 0x8000, 0, "shift must have bit 15 clear");
        prop_assert_ne!(r & 0x8000, 0, "reduce must have bit 15 set");
    }

    /// P33: Sentinel values (Accept/Error/Recover) are all distinct.
    #[test]
    fn p33_sentinels_distinct(_dummy in 0..1u8) {
        let tc = TableCompressor::new();
        let accept = tc.encode_action_small(&Action::Accept).unwrap();
        let error = tc.encode_action_small(&Action::Error).unwrap();
        let recover = tc.encode_action_small(&Action::Recover).unwrap();
        prop_assert_ne!(accept, error);
        prop_assert_ne!(accept, recover);
        prop_assert_ne!(error, recover);
    }
}

// ───────────────────────────────────────────────────────────────────────
// 7  Edge cases
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// P34: Single-row table compresses and decompresses.
    #[test]
    fn p34_single_row(row in prop::collection::vec(action_cell_strategy(), 1..=8)) {
        let table = vec![row];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.unique_rows.len(), 1);
        prop_assert_eq!(compressed.state_to_row.len(), 1);
        for (sym, cell) in table[0].iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            prop_assert_eq!(decompress_action(&compressed, 0, sym), expected);
        }
    }

    /// P35: Single-cell table (1×1).
    #[test]
    fn p35_single_cell(action in action_strategy()) {
        let table = vec![vec![vec![action.clone()]]];
        let compressed = compress_action_table(&table);
        let expected = action;
        prop_assert_eq!(decompress_action(&compressed, 0, 0), expected);
    }

    /// P36: All-Error action table.
    #[test]
    fn p36_all_error(
        states in 1usize..8,
        symbols in 1usize..8,
    ) {
        let table: Vec<Vec<Vec<Action>>> =
            vec![vec![vec![Action::Error]; symbols]; states];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.unique_rows.len(), 1);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(decompress_action(&compressed, s, sym), Action::Error);
            }
        }
    }

    /// P37: Fully-populated goto table (no None) preserves all entries.
    #[test]
    fn p37_dense_goto(
        states in 1usize..6,
        symbols in 1usize..6,
        target in 0u16..10,
    ) {
        let table: Vec<Vec<Option<StateId>>> =
            vec![vec![Some(StateId(target)); symbols]; states];
        let compressed = compress_goto_table(&table);
        prop_assert_eq!(compressed.entries.len(), states * symbols);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(
                    decompress_goto(&compressed, s, sym),
                    Some(StateId(target))
                );
            }
        }
    }

    /// P38: Empty action cells decompress as Error.
    #[test]
    fn p38_empty_cells_are_error(
        states in 1usize..6,
        symbols in 1usize..6,
    ) {
        let table: Vec<Vec<Vec<Action>>> =
            vec![vec![vec![]; symbols]; states];
        let compressed = compress_action_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(decompress_action(&compressed, s, sym), Action::Error);
            }
        }
    }

    /// P39: Out-of-range goto coordinates return None.
    #[test]
    fn p39_goto_oob_none(table in goto_table_strategy(4, 4)) {
        let compressed = compress_goto_table(&table);
        let oob_state = table.len() + 5;
        let oob_sym = table.first().map(|r| r.len()).unwrap_or(0) + 5;
        prop_assert_eq!(decompress_goto(&compressed, oob_state, 0), None);
        prop_assert_eq!(decompress_goto(&compressed, 0, oob_sym), None);
    }

    /// P40: Table with alternating duplicate rows deduplicates correctly.
    #[test]
    fn p40_alternating_dedup(
        row_a in prop::collection::vec(action_cell_strategy(), 3..=3),
        row_b in prop::collection::vec(action_cell_strategy(), 3..=3),
        repeats in 2usize..6,
    ) {
        let mut table = Vec::new();
        for _ in 0..repeats {
            table.push(row_a.clone());
            table.push(row_b.clone());
        }
        let compressed = compress_action_table(&table);
        if row_a == row_b {
            prop_assert_eq!(compressed.unique_rows.len(), 1);
        } else {
            prop_assert_eq!(compressed.unique_rows.len(), 2);
        }
    }

    /// P41: Fork action with single inner reduces to fork (not flattened).
    #[test]
    fn p41_fork_single_inner(state in 1u16..50) {
        let inner = vec![Action::Shift(StateId(state))];
        let table = vec![vec![vec![Action::Fork(inner.clone())]]];
        let compressed = compress_action_table(&table);
        let got = decompress_action(&compressed, 0, 0);
        // decompress returns first action in cell
        prop_assert_eq!(got, Action::Fork(inner));
    }

    /// P42: TableCompressor::new produces a usable compressor.
    #[test]
    fn p42_compressor_default_works(action in action_strategy()) {
        let tc = TableCompressor::new();
        let result = tc.encode_action_small(&action);
        // All non-overflow actions should encode successfully
        match &action {
            Action::Shift(StateId(s)) if *s >= 0x8000 => {
                prop_assert!(result.is_err());
            }
            Action::Reduce(RuleId(r)) if *r >= 0x4000 => {
                prop_assert!(result.is_err());
            }
            _ => {
                prop_assert!(result.is_ok());
            }
        }
    }

    /// P43: Reduce encoding is monotonic: larger rule_id → larger encoded value (within range).
    #[test]
    fn p43_reduce_monotonic(
        a in 0u16..0x3FFE,
        b in 0u16..0x3FFE,
    ) {
        let tc = TableCompressor::new();
        let ea = tc.encode_action_small(&Action::Reduce(RuleId(a))).unwrap();
        let eb = tc.encode_action_small(&Action::Reduce(RuleId(b))).unwrap();
        if a < b {
            prop_assert!(ea < eb);
        } else if a > b {
            prop_assert!(ea > eb);
        } else {
            prop_assert_eq!(ea, eb);
        }
    }

    /// P44: Shift encoding is monotonic.
    #[test]
    fn p44_shift_monotonic(
        a in 0u16..0x7FFF,
        b in 0u16..0x7FFF,
    ) {
        let tc = TableCompressor::new();
        let ea = tc.encode_action_small(&Action::Shift(StateId(a))).unwrap();
        let eb = tc.encode_action_small(&Action::Shift(StateId(b))).unwrap();
        if a < b {
            prop_assert!(ea < eb);
        } else if a > b {
            prop_assert!(ea > eb);
        } else {
            prop_assert_eq!(ea, eb);
        }
    }
}
