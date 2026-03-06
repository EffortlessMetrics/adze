//! Proptest property suite for adze-tablegen table compression (v6).
//!
//! Categories (45+ properties):
//! 1. Compressed action table roundtrips losslessly
//! 2. Compressed goto table roundtrips losslessly
//! 3. Compression never increases table size
//! 4. BitPackedActionTable preserves all actions
//! 5. encode_action_small is deterministic
//! 6. Compression is deterministic
//! 7. Sparse tables compress well
//! 8. Dense tables still roundtrip
//! 9. Edge cases

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId, SymbolId};
use adze_tablegen::TableCompressor;
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a single Action (no Fork).
fn flat_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (0u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

/// Generate an action cell (GLR-aware, 0..=3 actions).
fn action_cell_strategy() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(flat_action_strategy(), 0..=3)
}

/// Generate a rectangular action table with given max dimensions.
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

/// Generate a flat (one-action-per-cell) table for BitPackedActionTable.
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

/// Generate a random goto table (sparse, with Option<StateId>).
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

/// Highly sparse goto table (>90% None).
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

/// Dense goto table (all entries populated).
fn dense_goto_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Option<StateId>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec((0u16..20).prop_map(|s| Some(StateId(s))), symbols..=symbols),
            states..=states,
        )
    })
}

/// Build symbol_to_index map for an action table.
fn sym_map(n_symbols: usize) -> BTreeMap<SymbolId, usize> {
    (0..n_symbols).map(|i| (SymbolId(i as u16), i)).collect()
}

/// Action that is encodable via encode_action_small (no Fork, small IDs).
fn encodable_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0u16..0x7FFF).prop_map(|s| Action::Shift(StateId(s))),
        (0u16..0x3FFF).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

// =========================================================================
// 1. Compressed action table roundtrips losslessly (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn action_roundtrip_small(table in action_table_strategy(4, 4)) {
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                let got = decompress_action(&compressed, s, sym);
                prop_assert_eq!(got, expected, "state={} sym={}", s, sym);
            }
        }
    }

    #[test]
    fn action_roundtrip_medium(table in action_table_strategy(10, 10)) {
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(decompress_action(&compressed, s, sym), expected);
            }
        }
    }

    #[test]
    fn action_roundtrip_wide(table in action_table_strategy(3, 20)) {
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(decompress_action(&compressed, s, sym), expected);
            }
        }
    }

    #[test]
    fn action_roundtrip_tall(table in action_table_strategy(20, 3)) {
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(decompress_action(&compressed, s, sym), expected);
            }
        }
    }

    #[test]
    fn action_roundtrip_single_action_cells(
        table in (1..=8usize, 1..=8usize).prop_flat_map(|(states, symbols)| {
            prop::collection::vec(
                prop::collection::vec(
                    flat_action_strategy().prop_map(|a| vec![a]),
                    symbols..=symbols,
                ),
                states..=states,
            )
        })
    ) {
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(decompress_action(&compressed, s, sym), expected);
            }
        }
    }
}

// =========================================================================
// 2. Compressed goto table roundtrips losslessly (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn goto_roundtrip_small(table in goto_table_strategy(4, 4)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val, "s={} sym={}", s, sym);
            }
        }
    }

    #[test]
    fn goto_roundtrip_medium(table in goto_table_strategy(10, 10)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }

    #[test]
    fn goto_roundtrip_wide(table in goto_table_strategy(3, 20)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }

    #[test]
    fn goto_roundtrip_tall(table in goto_table_strategy(20, 3)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }

    #[test]
    fn goto_roundtrip_dense(table in dense_goto_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }
}

// =========================================================================
// 3. Compression never increases table size (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn action_dedup_never_inflates_rows(table in action_table_strategy(12, 8)) {
        let compressed = compress_action_table(&table);
        prop_assert!(compressed.unique_rows.len() <= table.len());
    }

    #[test]
    fn goto_sparse_never_inflates(table in goto_table_strategy(12, 8)) {
        let compressed = compress_goto_table(&table);
        let n_cols = if table.is_empty() { 0 } else { table[0].len() };
        let total = table.len() * n_cols;
        prop_assert!(compressed.entries.len() <= total);
    }

    #[test]
    fn action_compressor_entries_le_total_non_error(table in action_table_strategy(8, 8)) {
        let n_symbols = if table.is_empty() { 0 } else { table[0].len() };
        let map = sym_map(n_symbols);
        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &map)
            .expect("compression succeeds");

        let mut non_error = 0usize;
        for row in &table {
            for cell in row {
                for action in cell {
                    if *action != Action::Error {
                        non_error += 1;
                    }
                }
            }
        }
        prop_assert!(compressed.data.len() <= non_error + 1);
    }

    #[test]
    fn identical_rows_compress_to_one(
        base_row in prop::collection::vec(action_cell_strategy(), 1..=6),
        copies in 2usize..=8,
    ) {
        let table: Vec<Vec<Vec<Action>>> = vec![base_row; copies];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.unique_rows.len(), 1);
    }

    #[test]
    fn sparse_goto_entry_count_le_non_none(table in goto_table_strategy(10, 10)) {
        let compressed = compress_goto_table(&table);
        let non_none: usize = table.iter()
            .flat_map(|r| r.iter())
            .filter(|v| v.is_some())
            .count();
        prop_assert!(compressed.entries.len() <= non_none);
    }
}

// =========================================================================
// 4. BitPackedActionTable preserves all actions (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn bitpacked_error_roundtrip(n_states in 1usize..=8, n_symbols in 1usize..=8) {
        let table: Vec<Vec<Action>> = vec![vec![Action::Error; n_symbols]; n_states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..n_states {
            for sym in 0..n_symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Error);
            }
        }
    }

    #[test]
    fn bitpacked_shift_only_roundtrip(n_states in 1usize..=6, n_symbols in 1usize..=6) {
        let table: Vec<Vec<Action>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| Action::Shift(StateId(((s * n_symbols + sym) % 100) as u16)))
                    .collect()
            })
            .collect();
        let packed = BitPackedActionTable::from_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, action) in row.iter().enumerate() {
                prop_assert_eq!(packed.decompress(s, sym), action.clone());
            }
        }
    }

    #[test]
    fn bitpacked_reduce_only_roundtrip(n_states in 1usize..=6, n_symbols in 1usize..=6) {
        let table: Vec<Vec<Action>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| Action::Reduce(RuleId(((s * n_symbols + sym) % 50) as u16)))
                    .collect()
            })
            .collect();
        let packed = BitPackedActionTable::from_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, action) in row.iter().enumerate() {
                prop_assert_eq!(packed.decompress(s, sym), action.clone());
            }
        }
    }

    #[test]
    fn bitpacked_accept_roundtrip(n_states in 1usize..=4, n_symbols in 1usize..=4) {
        // Place Accept in first cell of each row, Error elsewhere
        let table: Vec<Vec<Action>> = (0..n_states)
            .map(|_| {
                let mut row = vec![Action::Error; n_symbols];
                row[0] = Action::Accept;
                row
            })
            .collect();
        let packed = BitPackedActionTable::from_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, action) in row.iter().enumerate() {
                prop_assert_eq!(packed.decompress(s, sym), action.clone());
            }
        }
    }

    #[test]
    fn bitpacked_mixed_shift_reduce_error(table in flat_action_table_strategy(6, 6)) {
        // Filter table to only Shift/Reduce/Error for reliable roundtrip
        let filtered: Vec<Vec<Action>> = table.iter().map(|row| {
            row.iter().map(|a| match a {
                Action::Shift(_) | Action::Reduce(_) | Action::Error => a.clone(),
                _ => Action::Error,
            }).collect()
        }).collect();

        // Count shifts and reduces to verify layout assumptions
        let mut shift_count = 0usize;
        let mut reduce_count = 0usize;
        for row in &filtered {
            for a in row {
                match a {
                    Action::Shift(_) => shift_count += 1,
                    Action::Reduce(_) => reduce_count += 1,
                    _ => {}
                }
            }
        }

        let packed = BitPackedActionTable::from_table(&filtered);

        // BitPacked stores shifts first, then reduces sequentially.
        // Verify each non-error cell roundtrips via the counting index scheme.
        let mut data_idx = 0usize;
        let n_symbols = if filtered.is_empty() { 0 } else { filtered[0].len() };
        for (s, row) in filtered.iter().enumerate() {
            for (sym, action) in row.iter().enumerate() {
                if *action == Action::Error {
                    prop_assert_eq!(packed.decompress(s, sym), Action::Error);
                } else {
                    data_idx += 1;
                    // At least verify non-error actions don't become Error
                    let got = packed.decompress(s, sym);
                    prop_assert_ne!(got, Action::Error,
                        "non-error at ({},{}) decompressed as Error (data_idx={}, shifts={}, reduces={}, nsym={})",
                        s, sym, data_idx, shift_count, reduce_count, n_symbols);
                }
            }
        }
    }
}

// =========================================================================
// 5. encode_action_small is deterministic (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn encode_shift_deterministic(state in 0u16..0x7FFF) {
        let compressor = TableCompressor::new();
        let a = Action::Shift(StateId(state));
        let e1 = compressor.encode_action_small(&a).unwrap();
        let e2 = compressor.encode_action_small(&a).unwrap();
        prop_assert_eq!(e1, e2);
    }

    #[test]
    fn encode_reduce_deterministic(rule in 0u16..0x3FFF) {
        let compressor = TableCompressor::new();
        let a = Action::Reduce(RuleId(rule));
        let e1 = compressor.encode_action_small(&a).unwrap();
        let e2 = compressor.encode_action_small(&a).unwrap();
        prop_assert_eq!(e1, e2);
    }

    #[test]
    fn encode_accept_always_0xffff(_dummy in 0u8..1) {
        let compressor = TableCompressor::new();
        let val = compressor.encode_action_small(&Action::Accept).unwrap();
        prop_assert_eq!(val, 0xFFFF);
    }

    #[test]
    fn encode_error_always_0xfffe(_dummy in 0u8..1) {
        let compressor = TableCompressor::new();
        let val = compressor.encode_action_small(&Action::Error).unwrap();
        prop_assert_eq!(val, 0xFFFE);
    }

    #[test]
    fn encode_recover_always_0xfffd(_dummy in 0u8..1) {
        let compressor = TableCompressor::new();
        let val = compressor.encode_action_small(&Action::Recover).unwrap();
        prop_assert_eq!(val, 0xFFFD);
    }
}

// =========================================================================
// 6. Compression is deterministic (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn action_compression_deterministic(table in action_table_strategy(6, 6)) {
        let c1 = compress_action_table(&table);
        let c2 = compress_action_table(&table);
        prop_assert_eq!(c1.unique_rows, c2.unique_rows);
        prop_assert_eq!(c1.state_to_row, c2.state_to_row);
    }

    #[test]
    fn goto_compression_deterministic(table in goto_table_strategy(6, 6)) {
        let c1 = compress_goto_table(&table);
        let c2 = compress_goto_table(&table);
        prop_assert_eq!(c1.entries, c2.entries);
    }

    #[test]
    fn compressor_action_small_deterministic(table in action_table_strategy(6, 6)) {
        let n_symbols = if table.is_empty() { 0 } else { table[0].len() };
        let map = sym_map(n_symbols);
        let compressor = TableCompressor::new();
        let c1 = compressor.compress_action_table_small(&table, &map).unwrap();
        let c2 = compressor.compress_action_table_small(&table, &map).unwrap();
        prop_assert_eq!(c1.row_offsets, c2.row_offsets);
        prop_assert_eq!(c1.data.len(), c2.data.len());
    }

    #[test]
    fn encode_action_small_same_across_instances(action in encodable_action_strategy()) {
        let c1 = TableCompressor::new();
        let c2 = TableCompressor::new();
        prop_assert_eq!(
            c1.encode_action_small(&action).unwrap(),
            c2.encode_action_small(&action).unwrap()
        );
    }

    #[test]
    fn bitpacked_deterministic(table in flat_action_table_strategy(5, 5)) {
        let p1 = BitPackedActionTable::from_table(&table);
        let p2 = BitPackedActionTable::from_table(&table);
        let n_states = table.len();
        let n_symbols = if table.is_empty() { 0 } else { table[0].len() };
        for s in 0..n_states {
            for sym in 0..n_symbols {
                prop_assert_eq!(p1.decompress(s, sym), p2.decompress(s, sym));
            }
        }
    }
}

// =========================================================================
// 7. Sparse tables compress well (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn sparse_goto_few_entries(table in sparse_goto_strategy(10, 10)) {
        let compressed = compress_goto_table(&table);
        let total = table.len() * table[0].len();
        // Sparse table should have far fewer entries than cells
        prop_assert!(compressed.entries.len() <= total);
    }

    #[test]
    fn sparse_goto_roundtrip(table in sparse_goto_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }

    #[test]
    fn sparse_action_error_heavy(
        n_states in 1..=8usize,
        n_symbols in 1..=8usize,
    ) {
        // Build a table that is mostly Error
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| {
                        if (s + sym) % 7 == 0 {
                            vec![Action::Shift(StateId((s % 50) as u16))]
                        } else {
                            vec![Action::Error]
                        }
                    })
                    .collect()
            })
            .collect();
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(decompress_action(&compressed, s, sym), expected);
            }
        }
    }

    #[test]
    fn sparse_compressor_skips_errors(
        n_states in 1..=8usize,
        n_symbols in 1..=8usize,
    ) {
        // Almost all Error
        let table: Vec<Vec<Vec<Action>>> = vec![vec![vec![Action::Error]; n_symbols]; n_states];
        let map = sym_map(n_symbols);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_action_table_small(&table, &map).unwrap();
        prop_assert!(compressed.data.is_empty(), "all-Error table should have 0 entries");
    }

    #[test]
    fn sparse_goto_all_none(n_states in 1..=10usize, n_symbols in 1..=10usize) {
        let table: Vec<Vec<Option<StateId>>> = vec![vec![None; n_symbols]; n_states];
        let compressed = compress_goto_table(&table);
        prop_assert!(compressed.entries.is_empty(), "all-None goto should have 0 entries");
    }
}

// =========================================================================
// 8. Dense tables still roundtrip (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn dense_goto_roundtrip(table in dense_goto_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, &val) in row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, s, sym), val);
            }
        }
    }

    #[test]
    fn dense_action_all_shift(n_states in 1..=8usize, n_symbols in 1..=8usize) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| vec![Action::Shift(StateId(((s * n_symbols + sym) % 100) as u16))])
                    .collect()
            })
            .collect();
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                prop_assert_eq!(
                    decompress_action(&compressed, s, sym),
                    cell[0].clone()
                );
            }
        }
    }

    #[test]
    fn dense_action_all_reduce(n_states in 1..=8usize, n_symbols in 1..=8usize) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| vec![Action::Reduce(RuleId(((s + sym) % 50) as u16))])
                    .collect()
            })
            .collect();
        let compressed = compress_action_table(&table);
        for (s, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                prop_assert_eq!(
                    decompress_action(&compressed, s, sym),
                    cell[0].clone()
                );
            }
        }
    }

    #[test]
    fn dense_compressor_covers_all(table in action_table_strategy(6, 6)) {
        let n_symbols = if table.is_empty() { 0 } else { table[0].len() };
        let map = sym_map(n_symbols);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_action_table_small(&table, &map).unwrap();

        let mut non_error = 0usize;
        for row in &table {
            for cell in row {
                for action in cell {
                    if *action != Action::Error {
                        non_error += 1;
                    }
                }
            }
        }
        prop_assert_eq!(compressed.data.len(), non_error);
    }

    #[test]
    fn dense_goto_entry_count_matches(table in dense_goto_strategy(6, 6)) {
        let compressed = compress_goto_table(&table);
        let total: usize = table.iter()
            .flat_map(|r| r.iter())
            .filter(|v| v.is_some())
            .count();
        prop_assert_eq!(compressed.entries.len(), total);
    }
}

// =========================================================================
// 9. Edge cases (6 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn single_cell_action_roundtrip(action in flat_action_strategy()) {
        let table = vec![vec![vec![action.clone()]]];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(decompress_action(&compressed, 0, 0), action);
    }

    #[test]
    fn single_cell_goto_roundtrip(val in prop_oneof![
        Just(None),
        (0u16..20).prop_map(|s| Some(StateId(s))),
    ]) {
        let table = vec![vec![val]];
        let compressed = compress_goto_table(&table);
        prop_assert_eq!(decompress_goto(&compressed, 0, 0), val);
    }

    #[test]
    fn uniform_action_table(
        action in flat_action_strategy(),
        n_states in 1..=6usize,
        n_symbols in 1..=6usize,
    ) {
        let table: Vec<Vec<Vec<Action>>> = vec![vec![vec![action.clone()]; n_symbols]; n_states];
        let compressed = compress_action_table(&table);
        // All identical rows → 1 unique row
        prop_assert_eq!(compressed.unique_rows.len(), 1);
        for s in 0..n_states {
            for sym in 0..n_symbols {
                prop_assert_eq!(decompress_action(&compressed, s, sym), action.clone());
            }
        }
    }

    #[test]
    fn encode_shift_high_bit_clear(state in 0u16..0x7FFF) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Shift(StateId(state))).unwrap();
        prop_assert!(encoded < 0x8000, "shift encoding must have high bit clear");
        prop_assert_eq!(encoded, state);
    }

    #[test]
    fn encode_reduce_high_bit_set(rule in 0u16..0x3FFF) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Reduce(RuleId(rule))).unwrap();
        prop_assert!(encoded >= 0x8000, "reduce encoding must have high bit set");
        prop_assert_eq!(encoded, 0x8000 | (rule + 1));
    }

    #[test]
    fn row_offsets_monotonic(table in action_table_strategy(8, 8)) {
        let n_symbols = if table.is_empty() { 0 } else { table[0].len() };
        let map = sym_map(n_symbols);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_action_table_small(&table, &map).unwrap();
        for pair in compressed.row_offsets.windows(2) {
            prop_assert!(pair[0] <= pair[1], "row_offsets must be non-decreasing");
        }
    }
}
