#![allow(clippy::needless_range_loop)]
// Property-based tests for table compression algorithms.
//
// Properties verified:
// 1. Compression preserves data (roundtrip properties)
// 2. Compressed tables are smaller or equal to uncompressed
// 3. Row deduplication correctness
// 4. Column deduplication
// 5. Default value selection (most common value optimization)
// 6. Bit packing properties
// 7. Edge cases: empty tables, single-cell tables, uniform tables
// 8. Large table compression
// 9. Sparse table efficiency
// 10. Dense table behavior

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a single Action suitable for action cells.
fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Error),
        Just(Action::Accept),
        (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
    ]
}

/// Generate a single flat Action (non-GLR, no Fork).
fn flat_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

/// Generate an action cell (Vec<Action>) with 0..3 actions.
fn action_cell_strategy() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(action_strategy(), 0..=3)
}

/// Generate a random action table with given dimensions.
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

/// Generate a flat action table (one Action per cell, for BitPackedActionTable).
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

// ---------------------------------------------------------------------------
// 1. Compression preserves data (roundtrip properties)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn action_compression_is_lossless(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);

        for (state, state_row) in table.iter().enumerate() {
            for (symbol, _) in state_row.iter().enumerate() {
                let original_first = state_row[symbol]
                    .first()
                    .cloned()
                    .unwrap_or(Action::Error);
                let decompressed = decompress_action(&compressed, state, symbol);
                prop_assert_eq!(
                    decompressed, original_first,
                    "Mismatch at state={}, symbol={}", state, symbol
                );
            }
        }
    }

    #[test]
    fn goto_compression_is_lossless(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);

        for (state, state_row) in table.iter().enumerate() {
            for (symbol, _) in state_row.iter().enumerate() {
                let decompressed = decompress_goto(&compressed, state, symbol);
                prop_assert_eq!(
                    decompressed, state_row[symbol],
                    "Goto mismatch at state={}, symbol={}", state, symbol
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Compressed tables are smaller or equal to uncompressed
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn row_dedup_never_inflates(table in action_table_strategy(12, 8)) {
        let compressed = compress_action_table(&table);
        prop_assert!(
            compressed.unique_rows.len() <= table.len(),
            "Dedup produced {} unique rows from {} original rows",
            compressed.unique_rows.len(),
            table.len()
        );
    }

    #[test]
    fn sparse_goto_never_inflates(table in goto_table_strategy(12, 8)) {
        let compressed = compress_goto_table(&table);
        let n_cols = if table.is_empty() { 0 } else { table[0].len() };
        let total_cells = table.len() * n_cols;
        prop_assert!(
            compressed.entries.len() <= total_cells,
            "Sparse representation has {} entries but table has {} cells",
            compressed.entries.len(),
            total_cells
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Row deduplication correctness
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn row_dedup_preserves_semantics(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);

        for (state, &row_idx) in compressed.state_to_row.iter().enumerate() {
            prop_assert!(
                row_idx < compressed.unique_rows.len(),
                "State {} maps to invalid row index {}", state, row_idx
            );
            prop_assert_eq!(
                &compressed.unique_rows[row_idx],
                &table[state],
                "State {}: unique row diverges from original", state
            );
        }
    }

    #[test]
    fn duplicate_rows_share_same_index(
        base_row in prop::collection::vec(action_cell_strategy(), 1..=6),
        n_copies in 2usize..=6,
    ) {
        let table: Vec<Vec<Vec<Action>>> = vec![base_row; n_copies];
        let compressed = compress_action_table(&table);

        prop_assert_eq!(
            compressed.unique_rows.len(),
            1,
            "Identical rows should dedup to 1 unique row, got {}",
            compressed.unique_rows.len()
        );

        let first_idx = compressed.state_to_row[0];
        for (state, &idx) in compressed.state_to_row.iter().enumerate() {
            prop_assert_eq!(
                idx, first_idx,
                "State {} maps to row {}, expected {}", state, idx, first_idx
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Column deduplication via TableCompressor
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn compressed_action_entries_skip_explicit_errors(table in action_table_strategy(8, 6)) {
        let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> = if !table.is_empty() {
            (0..table[0].len())
                .map(|i| (adze_ir::SymbolId(i as u16), i))
                .collect()
        } else {
            BTreeMap::new()
        };

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &symbol_to_index)
            .expect("compression must succeed");

        // No entry in compressed data should be an explicit Error action
        for entry in &compressed.data {
            prop_assert_ne!(
                entry.action.clone(),
                Action::Error,
                "Compressed data should not contain explicit Error entries"
            );
        }
    }

    #[test]
    fn compressed_action_entries_cover_all_non_error_actions(table in action_table_strategy(6, 6)) {
        let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> = if !table.is_empty() {
            (0..table[0].len())
                .map(|i| (adze_ir::SymbolId(i as u16), i))
                .collect()
        } else {
            BTreeMap::new()
        };

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &symbol_to_index)
            .expect("compression must succeed");

        // Count non-error actions in the original table
        let mut non_error_count = 0usize;
        for row in &table {
            for cell in row {
                for action in cell {
                    if action != &Action::Error {
                        non_error_count += 1;
                    }
                }
            }
        }

        prop_assert_eq!(
            compressed.data.len(),
            non_error_count,
            "Compressed data length {} != non-error action count {}",
            compressed.data.len(),
            non_error_count
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Default value selection (most common value optimization)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn default_actions_are_always_error(table in action_table_strategy(8, 6)) {
        let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> = if !table.is_empty() {
            (0..table[0].len())
                .map(|i| (adze_ir::SymbolId(i as u16), i))
                .collect()
        } else {
            BTreeMap::new()
        };

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &symbol_to_index)
            .expect("compression must succeed");

        // Default action optimization is disabled; all defaults must be Error
        prop_assert_eq!(
            compressed.default_actions.len(),
            table.len(),
            "default_actions length must match state count"
        );
        for (i, default) in compressed.default_actions.iter().enumerate() {
            prop_assert_eq!(
                default.clone(),
                Action::Error,
                "State {} default action should be Error (optimization disabled)",
                i
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Bit packing properties (via public decompress API)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn bit_packed_error_roundtrip(
        n_states in 1usize..=8,
        n_symbols in 1usize..=8,
    ) {
        // All-error table: every cell should decompress as Error
        let table: Vec<Vec<Action>> = vec![vec![Action::Error; n_symbols]; n_states];
        let packed = BitPackedActionTable::from_table(&table);

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                prop_assert_eq!(
                    packed.decompress(state, symbol),
                    Action::Error,
                    "All-error table cell ({},{}) should decompress as Error",
                    state, symbol
                );
            }
        }
    }

    #[test]
    fn bit_packed_all_shift_roundtrip(
        n_states in 1usize..=6,
        n_symbols in 1usize..=6,
    ) {
        // All-shift table: every cell has a unique shift target
        let table: Vec<Vec<Action>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| Action::Shift(StateId((s * n_symbols + sym) as u16)))
                    .collect()
            })
            .collect();
        let packed = BitPackedActionTable::from_table(&table);

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                let expected = Action::Shift(StateId((state * n_symbols + symbol) as u16));
                prop_assert_eq!(
                    packed.decompress(state, symbol),
                    expected,
                    "All-shift table cell ({},{}) mismatch",
                    state, symbol
                );
            }
        }
    }

    #[test]
    fn bit_packed_all_reduce_roundtrip(
        n_states in 1usize..=6,
        n_symbols in 1usize..=6,
    ) {
        // All-reduce table: every cell has a unique reduce rule
        let table: Vec<Vec<Action>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| Action::Reduce(RuleId((s * n_symbols + sym) as u16)))
                    .collect()
            })
            .collect();
        let packed = BitPackedActionTable::from_table(&table);

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                let expected = Action::Reduce(RuleId((state * n_symbols + symbol) as u16));
                prop_assert_eq!(
                    packed.decompress(state, symbol),
                    expected,
                    "All-reduce table cell ({},{}) mismatch",
                    state, symbol
                );
            }
        }
    }

    #[test]
    fn bit_packed_construction_does_not_panic(table in flat_action_table_strategy(16, 16)) {
        let packed = BitPackedActionTable::from_table(&table);
        // Structural: decompress any cell without panic
        if !table.is_empty() && !table[0].is_empty() {
            let _ = packed.decompress(0, 0);
            let _ = packed.decompress(table.len() - 1, table[0].len() - 1);
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Edge cases: empty tables, single-cell tables, uniform tables
// ---------------------------------------------------------------------------

#[test]
fn empty_action_table_compresses_to_empty() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let compressed = compress_action_table(&table);
    assert!(compressed.unique_rows.is_empty());
    assert!(compressed.state_to_row.is_empty());
}

#[test]
fn empty_goto_table_compresses_to_empty() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn single_cell_action_table_roundtrip(action in action_strategy()) {
        let table = vec![vec![vec![action.clone()]]];
        let compressed = compress_action_table(&table);

        prop_assert_eq!(compressed.unique_rows.len(), 1);
        prop_assert_eq!(compressed.state_to_row.len(), 1);
        prop_assert_eq!(
            decompress_action(&compressed, 0, 0),
            action,
            "Single-cell action roundtrip failed"
        );
    }

    #[test]
    fn single_cell_goto_table_roundtrip(
        target in prop_oneof![
            Just(None),
            (0u16..20).prop_map(|s| Some(StateId(s))),
        ]
    ) {
        let table = vec![vec![target]];
        let compressed = compress_goto_table(&table);

        prop_assert_eq!(
            decompress_goto(&compressed, 0, 0),
            target,
            "Single-cell goto roundtrip failed"
        );
    }

    #[test]
    fn uniform_action_table_dedup_to_one_row(
        action in action_strategy(),
        n_states in 2usize..=10,
        n_symbols in 1usize..=6,
    ) {
        let table: Vec<Vec<Vec<Action>>> =
            vec![vec![vec![action.clone()]; n_symbols]; n_states];
        let compressed = compress_action_table(&table);

        prop_assert_eq!(
            compressed.unique_rows.len(),
            1,
            "Uniform table should dedup to 1 unique row"
        );

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                prop_assert_eq!(
                    decompress_action(&compressed, state, symbol),
                    action.clone(),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Single-state and identity compression
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn single_state_action_roundtrip(
        row in prop::collection::vec(action_cell_strategy(), 1..=8)
    ) {
        let table = vec![row];
        let compressed = compress_action_table(&table);

        prop_assert_eq!(compressed.unique_rows.len(), 1);
        prop_assert_eq!(compressed.state_to_row.len(), 1);

        for (symbol, cell) in table[0].iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            prop_assert_eq!(
                decompress_action(&compressed, 0, symbol),
                expected,
                "Single-state roundtrip failed at symbol {}", symbol
            );
        }
    }

    #[test]
    fn single_state_goto_roundtrip(
        row in prop::collection::vec(
            prop_oneof![
                3 => Just(None),
                1 => (0u16..20).prop_map(|s| Some(StateId(s))),
            ],
            1..=8
        )
    ) {
        let table = vec![row];
        let compressed = compress_goto_table(&table);

        for (symbol, &expected) in table[0].iter().enumerate() {
            prop_assert_eq!(
                decompress_goto(&compressed, 0, symbol),
                expected,
                "Single-state goto roundtrip failed at symbol {}", symbol
            );
        }
    }

    #[test]
    fn all_distinct_rows_yield_identity_mapping(
        n_symbols in 1usize..=4,
        n_states in 1usize..=6,
    ) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| vec![Action::Shift(StateId((s * n_symbols + sym) as u16))])
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        prop_assert_eq!(
            compressed.unique_rows.len(),
            n_states,
            "All-distinct rows should not be deduplicated"
        );

        for (i, &row_idx) in compressed.state_to_row.iter().enumerate() {
            prop_assert_eq!(row_idx, i, "Identity mapping broken at state {}", i);
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Large table compression
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn large_action_table_no_panic(table in action_table_strategy(64, 32)) {
        let compressed = compress_action_table(&table);

        prop_assert_eq!(compressed.state_to_row.len(), table.len());
        prop_assert!(compressed.unique_rows.len() <= table.len());

        if !table.is_empty() && !table[0].is_empty() {
            let _ = decompress_action(&compressed, 0, 0);
            let last_state = table.len() - 1;
            let last_sym = table[0].len() - 1;
            let _ = decompress_action(&compressed, last_state, last_sym);
        }
    }

    #[test]
    fn large_goto_table_no_panic(table in goto_table_strategy(64, 32)) {
        let compressed = compress_goto_table(&table);

        let n_cols = if table.is_empty() { 0 } else { table[0].len() };
        let total_cells = table.len() * n_cols;
        prop_assert!(compressed.entries.len() <= total_cells);

        if !table.is_empty() && !table[0].is_empty() {
            let _ = decompress_goto(&compressed, 0, 0);
            let _ = decompress_goto(&compressed, table.len() - 1, table[0].len() - 1);
        }
    }

    #[test]
    fn large_bit_packed_table_no_panic(table in flat_action_table_strategy(64, 32)) {
        let packed = BitPackedActionTable::from_table(&table);

        // Structural: decompress corner cells without panic
        if !table.is_empty() && !table[0].is_empty() {
            let _ = packed.decompress(0, 0);
            let _ = packed.decompress(table.len() - 1, table[0].len() - 1);
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Sparse table efficiency
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn sparse_goto_entry_count_equals_non_none_cells(table in goto_table_strategy(10, 10)) {
        let compressed = compress_goto_table(&table);

        let expected_entries: usize = table.iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.is_some())
            .count();

        prop_assert_eq!(
            compressed.entries.len(),
            expected_entries,
            "Sparse entry count {} != non-None cell count {}",
            compressed.entries.len(),
            expected_entries
        );
    }

    #[test]
    fn all_none_goto_rows_produce_no_entries(n_states in 1usize..=8, n_symbols in 1usize..=8) {
        let table: Vec<Vec<Option<StateId>>> = vec![vec![None; n_symbols]; n_states];
        let compressed = compress_goto_table(&table);

        prop_assert_eq!(
            compressed.entries.len(),
            0,
            "All-None table should produce 0 sparse entries"
        );

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                prop_assert_eq!(decompress_goto(&compressed, state, symbol), None);
            }
        }
    }

    #[test]
    fn mostly_sparse_goto_has_few_entries(
        n_states in 4usize..=12,
        n_symbols in 4usize..=12,
        seed in any::<u64>(),
    ) {
        // ~10% density
        let mut rng = seed;
        let table: Vec<Vec<Option<StateId>>> = (0..n_states)
            .map(|_| {
                (0..n_symbols)
                    .map(|_| {
                        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                        if (rng >> 56) < 26 { // ~10% chance
                            Some(StateId((rng >> 48) as u16 % 20))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();

        let compressed = compress_goto_table(&table);
        let total = n_states * n_symbols;

        // Sparse: entries should be much less than total cells
        prop_assert!(
            compressed.entries.len() <= total / 2,
            "~10% density table has {} entries in {} cells (>{:.0}%)",
            compressed.entries.len(), total,
            (compressed.entries.len() as f64 / total as f64) * 100.0
        );
    }
}

// ---------------------------------------------------------------------------
// 10. Dense table behavior
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn dense_goto_preserves_all_entries(
        n_states in 1usize..=8,
        n_symbols in 1usize..=8,
    ) {
        // Fully populated goto table (no None)
        let table: Vec<Vec<Option<StateId>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| Some(StateId((s * n_symbols + sym) as u16)))
                    .collect()
            })
            .collect();

        let compressed = compress_goto_table(&table);

        // All cells are populated so entry count == total cells
        prop_assert_eq!(
            compressed.entries.len(),
            n_states * n_symbols,
            "Dense goto table should have {} entries, got {}",
            n_states * n_symbols,
            compressed.entries.len()
        );

        // Verify all values roundtrip
        for state in 0..n_states {
            for symbol in 0..n_symbols {
                let expected = Some(StateId((state * n_symbols + symbol) as u16));
                prop_assert_eq!(
                    decompress_goto(&compressed, state, symbol),
                    expected,
                );
            }
        }
    }

    #[test]
    fn dense_action_table_all_shifts_roundtrip(
        n_states in 1usize..=8,
        n_symbols in 1usize..=8,
    ) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| vec![Action::Shift(StateId((s * n_symbols + sym) as u16))])
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                let expected = Action::Shift(StateId((state * n_symbols + symbol) as u16));
                prop_assert_eq!(
                    decompress_action(&compressed, state, symbol),
                    expected,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Encode action small properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn encode_shift_preserves_state_id(state_id in 0u16..0x7FFF) {
        let compressor = adze_tablegen::TableCompressor::new();
        let action = Action::Shift(StateId(state_id));
        let encoded = compressor.encode_action_small(&action).unwrap();

        // Shift encoding: raw state id (high bit clear)
        prop_assert_eq!(
            encoded, state_id,
            "Shift({}) encoded as {} instead of {}",
            state_id, encoded, state_id
        );
        prop_assert!(encoded < 0x8000, "Shift encoding must have high bit clear");
    }

    #[test]
    fn encode_reduce_preserves_rule_id(rule_id in 0u16..0x3FFF) {
        let compressor = adze_tablegen::TableCompressor::new();
        let action = Action::Reduce(RuleId(rule_id));
        let encoded = compressor.encode_action_small(&action).unwrap();

        // Reduce encoding: 0x8000 | (rule_id + 1)
        let expected = 0x8000 | (rule_id + 1);
        prop_assert_eq!(
            encoded, expected,
            "Reduce({}) encoded as {:#06X} instead of {:#06X}",
            rule_id, encoded, expected
        );
        prop_assert!(encoded >= 0x8000, "Reduce encoding must have high bit set");
    }

    #[test]
    fn encode_special_actions_use_distinct_values(action in flat_action_strategy()) {
        let compressor = adze_tablegen::TableCompressor::new();
        let encoded = compressor.encode_action_small(&action).unwrap();

        match action {
            Action::Accept => prop_assert_eq!(encoded, 0xFFFF),
            Action::Error => prop_assert_eq!(encoded, 0xFFFE),
            Action::Recover => prop_assert_eq!(encoded, 0xFFFD),
            _ => {} // Shift/Reduce tested separately
        }
    }

    #[test]
    fn encode_shift_too_large_fails(state_id in 0x8000u16..=u16::MAX) {
        let compressor = adze_tablegen::TableCompressor::new();
        let action = Action::Shift(StateId(state_id));
        prop_assert!(
            compressor.encode_action_small(&action).is_err(),
            "Shift({}) should fail encoding",
            state_id
        );
    }

    #[test]
    fn encode_reduce_too_large_fails(rule_id in 0x4000u16..=u16::MAX) {
        let compressor = adze_tablegen::TableCompressor::new();
        let action = Action::Reduce(RuleId(rule_id));
        prop_assert!(
            compressor.encode_action_small(&action).is_err(),
            "Reduce({}) should fail encoding",
            rule_id
        );
    }
}

// ---------------------------------------------------------------------------
// Structural invariants
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn action_row_offsets_are_non_overlapping(table in action_table_strategy(10, 8)) {
        let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> = if !table.is_empty() {
            (0..table[0].len())
                .map(|i| (adze_ir::SymbolId(i as u16), i))
                .collect()
        } else {
            BTreeMap::new()
        };

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &symbol_to_index)
            .expect("compression must succeed");

        prop_assert_eq!(compressed.row_offsets.len(), table.len() + 1);

        for window in compressed.row_offsets.windows(2) {
            prop_assert!(
                window[0] <= window[1],
                "Row offsets not non-decreasing: {} > {}",
                window[0],
                window[1]
            );
        }

        prop_assert_eq!(
            *compressed.row_offsets.last().unwrap() as usize,
            compressed.data.len(),
            "Last offset must equal data array length"
        );

        for i in 0..table.len() {
            let start = compressed.row_offsets[i] as usize;
            let end = compressed.row_offsets[i + 1] as usize;
            prop_assert!(
                start <= end,
                "State {}: start ({}) > end ({})", i, start, end
            );
            prop_assert!(
                end <= compressed.data.len(),
                "State {}: end ({}) > data.len() ({})", i, end,
                compressed.data.len()
            );
        }
    }

    #[test]
    fn goto_row_offsets_are_non_overlapping(
        n_states in 1usize..=8,
        n_symbols in 1usize..=8,
        seed in any::<u64>(),
    ) {
        let mut rng = seed;
        let mut goto_table: Vec<Vec<StateId>> = Vec::new();
        for _ in 0..n_states {
            let mut row = Vec::new();
            for _ in 0..n_symbols {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                row.push(StateId((rng >> 48) as u16 % 20));
            }
            goto_table.push(row);
        }

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_goto_table_small(&goto_table)
            .expect("goto compression must succeed");

        prop_assert_eq!(compressed.row_offsets.len(), n_states + 1);

        for window in compressed.row_offsets.windows(2) {
            prop_assert!(
                window[0] <= window[1],
                "Goto row offsets not non-decreasing: {} > {}",
                window[0],
                window[1]
            );
        }

        prop_assert_eq!(
            *compressed.row_offsets.last().unwrap() as usize,
            compressed.data.len(),
            "Last goto offset must equal data array length"
        );
    }

    #[test]
    fn state_to_row_mapping_is_well_defined(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);

        prop_assert_eq!(
            compressed.state_to_row.len(),
            table.len(),
            "state_to_row length must match state count"
        );

        for (state, &row_idx) in compressed.state_to_row.iter().enumerate() {
            prop_assert!(
                row_idx < compressed.unique_rows.len(),
                "State {}: row_idx {} >= unique_rows.len() {}", state, row_idx,
                compressed.unique_rows.len()
            );
        }

        for i in 0..table.len() {
            for j in (i + 1)..table.len() {
                if compressed.state_to_row[i] == compressed.state_to_row[j] {
                    prop_assert_eq!(
                        &table[i], &table[j],
                        "States {} and {} share row index but have different rows", i, j
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// State and symbol count preservation
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn action_state_count_preserved(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);
        prop_assert_eq!(
            compressed.state_to_row.len(),
            table.len(),
            "Compressed state count {} != original {}",
            compressed.state_to_row.len(),
            table.len()
        );
    }

    #[test]
    fn goto_state_count_preserved(table in goto_table_strategy(10, 6)) {
        let compressed = compress_goto_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (symbol, &expected) in row.iter().enumerate() {
                prop_assert_eq!(
                    decompress_goto(&compressed, state, symbol),
                    expected,
                    "Goto state {} symbol {} lost", state, symbol
                );
            }
        }
    }

    #[test]
    fn action_symbol_count_preserved(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        if !table.is_empty() {
            let expected_symbols = table[0].len();
            for unique_row in &compressed.unique_rows {
                prop_assert_eq!(
                    unique_row.len(),
                    expected_symbols,
                    "Unique row symbol count {} != original {}",
                    unique_row.len(),
                    expected_symbols
                );
            }
        }
    }

    #[test]
    fn goto_symbol_count_preserved(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        if !table.is_empty() {
            let expected_symbols = table[0].len();
            for &(_, sym) in compressed.entries.keys() {
                prop_assert!(
                    sym < expected_symbols,
                    "Goto entry symbol {} >= expected symbol count {}",
                    sym,
                    expected_symbols
                );
            }
        }
    }

    #[test]
    fn all_error_action_rows_dedup_to_one(n_states in 1usize..=8, n_symbols in 1usize..=8) {
        let table: Vec<Vec<Vec<Action>>> =
            vec![vec![vec![Action::Error]; n_symbols]; n_states];
        let compressed = compress_action_table(&table);

        prop_assert_eq!(
            compressed.unique_rows.len(),
            1,
            "All-error rows should dedup to 1 unique row"
        );

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                prop_assert_eq!(
                    decompress_action(&compressed, state, symbol),
                    Action::Error
                );
            }
        }
    }

    #[test]
    fn empty_action_cells_decompress_as_error(table in action_table_strategy(6, 6)) {
        let compressed = compress_action_table(&table);

        for (state, state_row) in table.iter().enumerate() {
            for (symbol, _) in state_row.iter().enumerate() {
                if state_row[symbol].is_empty() {
                    prop_assert_eq!(
                        decompress_action(&compressed, state, symbol),
                        Action::Error,
                        "Empty cell at ({},{}) must decompress as Error", state, symbol
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Full cell comparison and determinism
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn all_compressed_cells_match_original(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);

        for (state, row) in table.iter().enumerate() {
            let row_idx = compressed.state_to_row[state];
            let compressed_row = &compressed.unique_rows[row_idx];

            prop_assert_eq!(
                compressed_row.len(),
                row.len(),
                "State {}: compressed row width {} != original {}",
                state,
                compressed_row.len(),
                row.len()
            );

            for (symbol, cell) in row.iter().enumerate() {
                prop_assert_eq!(
                    &compressed_row[symbol],
                    cell,
                    "Full cell mismatch at state={}, symbol={}", state, symbol
                );
            }
        }
    }

    #[test]
    fn all_goto_entries_match_original(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (symbol, &expected) in row.iter().enumerate() {
                let actual = decompress_goto(&compressed, state, symbol);
                prop_assert_eq!(
                    actual, expected,
                    "Goto mismatch at state={}, symbol={}", state, symbol
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn action_compression_is_deterministic(table in action_table_strategy(8, 8)) {
        let c1 = compress_action_table(&table);
        let c2 = compress_action_table(&table);

        prop_assert_eq!(
            c1.unique_rows, c2.unique_rows,
            "Unique rows differ between two compressions of same input"
        );
        prop_assert_eq!(
            c1.state_to_row, c2.state_to_row,
            "state_to_row differs between two compressions of same input"
        );
    }

    #[test]
    fn goto_compression_is_deterministic(table in goto_table_strategy(8, 8)) {
        let c1 = compress_goto_table(&table);
        let c2 = compress_goto_table(&table);

        prop_assert_eq!(
            c1.entries, c2.entries,
            "Goto entries differ between two compressions of same input"
        );
    }
}

// ===========================================================================
// Additional tests: maximum action cells, GLR multi-action, compression edge
// cases, run-length encoding, idempotency, mixed patterns
// ===========================================================================

/// Generate a GLR action cell with up to `max` actions (may include duplicates).
#[allow(dead_code)]
fn glr_cell_strategy(max: usize) -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(action_strategy(), 0..=max)
}

/// Generate an action table where every cell has exactly `n` actions.
fn fixed_width_cell_table(
    max_states: usize,
    max_symbols: usize,
    cell_size: usize,
) -> impl Strategy<Value = Vec<Vec<Vec<Action>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(move |(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(
                prop::collection::vec(action_strategy(), cell_size..=cell_size),
                symbols..=symbols,
            ),
            states..=states,
        )
    })
}

// ---------------------------------------------------------------------------
// 11. Maximum action cells (GLR multi-action)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Cells with many actions still roundtrip correctly.
    #[test]
    fn max_action_cells_roundtrip(
        n_states in 1usize..=4,
        n_symbols in 1usize..=4,
    ) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| {
                        // Build a cell with 5 actions: mix of shift/reduce
                        (0..5)
                            .map(|k| {
                                if k % 2 == 0 {
                                    Action::Shift(StateId(((s * n_symbols + sym) * 5 + k) as u16))
                                } else {
                                    Action::Reduce(RuleId(((s * n_symbols + sym) * 5 + k) as u16))
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        // decompress_action returns first action from cell
        for (state, row) in table.iter().enumerate() {
            for (symbol, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(
                    decompress_action(&compressed, state, symbol),
                    expected,
                    "Max-action-cell roundtrip failed at ({},{})", state, symbol
                );
            }
        }
    }

    /// Full cell content preserved for large GLR cells.
    #[test]
    fn large_glr_cells_preserved_in_unique_rows(table in fixed_width_cell_table(6, 6, 4)) {
        let compressed = compress_action_table(&table);

        for (state, row) in table.iter().enumerate() {
            let row_idx = compressed.state_to_row[state];
            let compressed_row = &compressed.unique_rows[row_idx];
            for (sym, cell) in row.iter().enumerate() {
                prop_assert_eq!(
                    &compressed_row[sym], cell,
                    "GLR cell content lost at ({},{})", state, sym
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 12. Compression with Accept actions
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Accept-only table roundtrips correctly.
    #[test]
    fn all_accept_action_table_roundtrip(
        n_states in 1usize..=6,
        n_symbols in 1usize..=6,
    ) {
        let table: Vec<Vec<Vec<Action>>> =
            vec![vec![vec![Action::Accept]; n_symbols]; n_states];
        let compressed = compress_action_table(&table);

        for state in 0..n_states {
            for symbol in 0..n_symbols {
                prop_assert_eq!(
                    decompress_action(&compressed, state, symbol),
                    Action::Accept,
                );
            }
        }
    }

    /// Mixed Accept/Shift/Reduce cells roundtrip.
    #[test]
    fn mixed_accept_shift_reduce_roundtrip(
        n_states in 1usize..=6,
        n_symbols in 1usize..=6,
    ) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| match (s + sym) % 3 {
                        0 => vec![Action::Accept],
                        1 => vec![Action::Shift(StateId(s as u16))],
                        _ => vec![Action::Reduce(RuleId(sym as u16))],
                    })
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(
                    decompress_action(&compressed, state, sym),
                    expected,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 13. CompressedParseTable API
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn compressed_parse_table_dimensions(
        sym_count in 1usize..=100,
        state_count in 1usize..=100,
    ) {
        let cpt = adze_tablegen::CompressedParseTable::new_for_testing(sym_count, state_count);
        prop_assert_eq!(cpt.symbol_count(), sym_count);
        prop_assert_eq!(cpt.state_count(), state_count);
    }
}

// ---------------------------------------------------------------------------
// 14. Run-length encoding in goto_table_small
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Uniform goto row uses RLE and produces fewer entries than columns.
    #[test]
    fn rle_uniform_row_compressed(
        n_symbols in 4usize..=16,
        target in 0u16..20,
    ) {
        let goto_table = vec![vec![StateId(target); n_symbols]];
        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_goto_table_small(&goto_table)
            .expect("goto compression must succeed");

        // A fully uniform row should use RLE: entry count < n_symbols
        prop_assert!(
            compressed.data.len() < n_symbols,
            "Uniform row of {} columns should RLE to fewer entries, got {}",
            n_symbols, compressed.data.len()
        );
    }

    /// Alternating goto values prevent RLE, so entries == symbols.
    #[test]
    fn alternating_goto_no_rle(
        n_symbols in 2usize..=12,
    ) {
        let goto_table = vec![
            (0..n_symbols)
                .map(|i| StateId(if i % 2 == 0 { 1 } else { 2 }))
                .collect::<Vec<_>>()
        ];
        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_goto_table_small(&goto_table)
            .expect("goto compression must succeed");

        // Alternating values can't RLE, so total entries == n_symbols
        prop_assert_eq!(
            compressed.data.len(), n_symbols,
            "Alternating values should produce {} entries, got {}",
            n_symbols, compressed.data.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 15. Goto small table row offsets
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn goto_small_row_offsets_valid(
        n_states in 1usize..=8,
        n_symbols in 1usize..=8,
        seed in any::<u64>(),
    ) {
        let mut rng = seed;
        let goto_table: Vec<Vec<StateId>> = (0..n_states)
            .map(|_| {
                (0..n_symbols)
                    .map(|_| {
                        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                        StateId((rng >> 48) as u16 % 20)
                    })
                    .collect()
            })
            .collect();

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_goto_table_small(&goto_table)
            .expect("goto compression must succeed");

        prop_assert_eq!(compressed.row_offsets.len(), n_states + 1);

        for w in compressed.row_offsets.windows(2) {
            prop_assert!(w[0] <= w[1]);
        }

        prop_assert_eq!(
            *compressed.row_offsets.last().unwrap() as usize,
            compressed.data.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 16. Idempotency: compressing same input twice yields identical output
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn bit_packed_deterministic(table in flat_action_table_strategy(8, 8)) {
        let p1 = BitPackedActionTable::from_table(&table);
        let p2 = BitPackedActionTable::from_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (sym, _) in row.iter().enumerate() {
                prop_assert_eq!(
                    p1.decompress(state, sym),
                    p2.decompress(state, sym),
                    "BitPacked determinism failed at ({},{})", state, sym
                );
            }
        }
    }

    #[test]
    fn compress_action_table_small_deterministic(table in action_table_strategy(6, 6)) {
        let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> = if !table.is_empty() {
            (0..table[0].len())
                .map(|i| (adze_ir::SymbolId(i as u16), i))
                .collect()
        } else {
            BTreeMap::new()
        };

        let compressor = adze_tablegen::TableCompressor::new();
        let c1 = compressor.compress_action_table_small(&table, &symbol_to_index).unwrap();
        let c2 = compressor.compress_action_table_small(&table, &symbol_to_index).unwrap();

        prop_assert_eq!(c1.row_offsets, c2.row_offsets);
        prop_assert_eq!(c1.data.len(), c2.data.len());
        for (a, b) in c1.data.iter().zip(c2.data.iter()) {
            prop_assert_eq!(a.symbol, b.symbol);
            prop_assert_eq!(a.action.clone(), b.action.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// 17. Compressed size ≤ original for action table small format
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn action_table_small_no_inflate(table in action_table_strategy(8, 8)) {
        let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> = if !table.is_empty() {
            (0..table[0].len())
                .map(|i| (adze_ir::SymbolId(i as u16), i))
                .collect()
        } else {
            BTreeMap::new()
        };

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &symbol_to_index)
            .unwrap();

        // Total original non-error actions == compressed.data.len()
        let original_non_error: usize = table.iter()
            .flat_map(|row| row.iter())
            .flat_map(|cell| cell.iter())
            .filter(|a| *a != &Action::Error)
            .count();

        prop_assert_eq!(
            compressed.data.len(), original_non_error,
            "Compressed entries {} != non-error count {}",
            compressed.data.len(), original_non_error
        );
    }

    /// Goto small table entry count ≤ total cells.
    #[test]
    fn goto_table_small_no_inflate(
        n_states in 1usize..=8,
        n_symbols in 1usize..=8,
        seed in any::<u64>(),
    ) {
        let mut rng = seed;
        let goto_table: Vec<Vec<StateId>> = (0..n_states)
            .map(|_| {
                (0..n_symbols)
                    .map(|_| {
                        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                        StateId((rng >> 48) as u16 % 20)
                    })
                    .collect()
            })
            .collect();

        let compressor = adze_tablegen::TableCompressor::new();
        let compressed = compressor
            .compress_goto_table_small(&goto_table)
            .unwrap();

        let total_cells = n_states * n_symbols;
        prop_assert!(
            compressed.data.len() <= total_cells,
            "Goto small table has {} entries but only {} cells",
            compressed.data.len(), total_cells
        );
    }
}

// ---------------------------------------------------------------------------
// 18. Sparse action tables (mostly Error)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Very sparse table: ~5% non-error cells still roundtrip.
    #[test]
    fn very_sparse_action_table_roundtrip(
        n_states in 4usize..=12,
        n_symbols in 4usize..=12,
        seed in any::<u64>(),
    ) {
        let mut rng = seed;
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|_| {
                (0..n_symbols)
                    .map(|_| {
                        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                        if (rng >> 56) < 13 { // ~5% chance
                            vec![Action::Shift(StateId((rng >> 48) as u16 % 100))]
                        } else {
                            vec![Action::Error]
                        }
                    })
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(
                    decompress_action(&compressed, state, sym),
                    expected,
                );
            }
        }
    }

    /// Sparse table dedup: many all-error rows share one unique row.
    #[test]
    fn sparse_table_shares_error_rows(
        n_states in 4usize..=10,
        n_symbols in 2usize..=6,
        seed in any::<u64>(),
    ) {
        let mut rng = seed;
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|_| {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                if (rng >> 56) < 51 { // ~20% chance of non-error row
                    (0..n_symbols)
                        .map(|sym| vec![Action::Shift(StateId(sym as u16))])
                        .collect()
                } else {
                    vec![vec![Action::Error]; n_symbols]
                }
            })
            .collect();

        let compressed = compress_action_table(&table);

        // Count all-error rows in original
        let error_row: Vec<Vec<Action>> = vec![vec![Action::Error]; n_symbols];
        let all_error_count = table.iter().filter(|r| **r == error_row).count();

        if all_error_count > 1 {
            // All-error rows should share one unique row index
            let mut error_indices = std::collections::HashSet::new();
            for (state, row) in table.iter().enumerate() {
                if *row == error_row {
                    error_indices.insert(compressed.state_to_row[state]);
                }
            }
            prop_assert_eq!(
                error_indices.len(), 1,
                "{} all-error rows mapped to {} unique indices, expected 1",
                all_error_count, error_indices.len()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 19. Dense action table (no errors)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Dense table with all Reduce actions roundtrips.
    #[test]
    fn dense_all_reduce_roundtrip(
        n_states in 1usize..=8,
        n_symbols in 1usize..=8,
    ) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| vec![Action::Reduce(RuleId((s * n_symbols + sym) as u16))])
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        for state in 0..n_states {
            for sym in 0..n_symbols {
                prop_assert_eq!(
                    decompress_action(&compressed, state, sym),
                    Action::Reduce(RuleId((state * n_symbols + sym) as u16)),
                );
            }
        }
    }

    /// Dense table: unique rows == state count when all rows differ.
    #[test]
    fn dense_all_distinct_no_dedup(
        n_states in 2usize..=8,
        n_symbols in 1usize..=4,
    ) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| vec![Action::Reduce(RuleId((s * 100 + sym) as u16))])
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);
        prop_assert_eq!(
            compressed.unique_rows.len(), n_states,
            "All-distinct dense table should have {} unique rows, got {}",
            n_states, compressed.unique_rows.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 20. Single-state edge cases for compress_action_table_small
// ---------------------------------------------------------------------------

#[test]
fn single_state_single_shift_small_compression() {
    let table = vec![vec![vec![Action::Shift(StateId(42))]]];
    let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> =
        [(adze_ir::SymbolId(0), 0)].into_iter().collect();

    let compressor = adze_tablegen::TableCompressor::new();
    let compressed = compressor
        .compress_action_table_small(&table, &symbol_to_index)
        .unwrap();

    assert_eq!(compressed.data.len(), 1);
    assert_eq!(compressed.data[0].action, Action::Shift(StateId(42)));
    assert_eq!(compressed.row_offsets, vec![0, 1]);
    assert_eq!(compressed.default_actions, vec![Action::Error]);
}

#[test]
fn single_state_all_errors_small_compression() {
    let table = vec![vec![vec![Action::Error]; 4]];
    let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> =
        (0..4).map(|i| (adze_ir::SymbolId(i as u16), i)).collect();

    let compressor = adze_tablegen::TableCompressor::new();
    let compressed = compressor
        .compress_action_table_small(&table, &symbol_to_index)
        .unwrap();

    assert!(
        compressed.data.is_empty(),
        "All-error row should produce no entries"
    );
    assert_eq!(compressed.row_offsets, vec![0, 0]);
}

// ---------------------------------------------------------------------------
// 21. BitPacked with Fork actions
// ---------------------------------------------------------------------------

#[test]
fn bit_packed_fork_roundtrip() {
    let fork_actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    let table = vec![vec![
        Action::Fork(fork_actions.clone()),
        Action::Shift(StateId(5)),
        Action::Error,
    ]];

    let packed = BitPackedActionTable::from_table(&table);

    match packed.decompress(0, 0) {
        Action::Fork(actions) => assert_eq!(actions, fork_actions),
        other => panic!("Expected Fork, got {:?}", other),
    }
    assert_eq!(packed.decompress(0, 2), Action::Error);
}

// ---------------------------------------------------------------------------
// 22. Encode action boundary values
// ---------------------------------------------------------------------------

#[test]
fn encode_shift_boundary_values() {
    let compressor = adze_tablegen::TableCompressor::new();

    // Minimum valid shift
    assert_eq!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0)))
            .unwrap(),
        0
    );

    // Maximum valid shift
    assert_eq!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x7FFE)))
            .unwrap(),
        0x7FFE
    );

    // Just over boundary
    assert!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x8000)))
            .is_err()
    );
}

#[test]
fn encode_reduce_boundary_values() {
    let compressor = adze_tablegen::TableCompressor::new();

    // Minimum valid reduce
    assert_eq!(
        compressor
            .encode_action_small(&Action::Reduce(RuleId(0)))
            .unwrap(),
        0x8001 // 0x8000 | (0 + 1)
    );

    // Maximum valid reduce
    assert_eq!(
        compressor
            .encode_action_small(&Action::Reduce(RuleId(0x3FFE)))
            .unwrap(),
        0x8000 | 0x3FFF
    );

    // Just over boundary
    assert!(
        compressor
            .encode_action_small(&Action::Reduce(RuleId(0x4000)))
            .is_err()
    );
}

// ---------------------------------------------------------------------------
// 23. Empty cell handling
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Table with empty cells (no actions) roundtrips as Error.
    #[test]
    fn empty_cells_in_mixed_table(
        n_states in 2usize..=6,
        n_symbols in 2usize..=6,
        seed in any::<u64>(),
    ) {
        let mut rng = seed;
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|_| {
                (0..n_symbols)
                    .map(|_| {
                        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                        match (rng >> 56) % 3 {
                            0 => vec![], // empty cell
                            1 => vec![Action::Shift(StateId((rng >> 48) as u16 % 50))],
                            _ => vec![Action::Error],
                        }
                    })
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(
                    decompress_action(&compressed, state, sym),
                    expected,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 24. Checkerboard pattern tables
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Checkerboard pattern: alternating Shift/Error cells.
    #[test]
    fn checkerboard_action_roundtrip(
        n_states in 2usize..=8,
        n_symbols in 2usize..=8,
    ) {
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| {
                        if (s + sym) % 2 == 0 {
                            vec![Action::Shift(StateId(1))]
                        } else {
                            vec![Action::Error]
                        }
                    })
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        for state in 0..n_states {
            for sym in 0..n_symbols {
                let expected = if (state + sym) % 2 == 0 {
                    Action::Shift(StateId(1))
                } else {
                    Action::Error
                };
                prop_assert_eq!(
                    decompress_action(&compressed, state, sym),
                    expected,
                );
            }
        }
    }

    /// Checkerboard goto table roundtrips.
    #[test]
    fn checkerboard_goto_roundtrip(
        n_states in 2usize..=8,
        n_symbols in 2usize..=8,
    ) {
        let table: Vec<Vec<Option<StateId>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| {
                        if (s + sym) % 2 == 0 {
                            Some(StateId((s + sym) as u16))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();

        let compressed = compress_goto_table(&table);

        for state in 0..n_states {
            for sym in 0..n_symbols {
                prop_assert_eq!(
                    decompress_goto(&compressed, state, sym),
                    table[state][sym],
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 25. Multi-action cell ordering preserved
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// The first action in a multi-action cell is always the one returned by decompress.
    #[test]
    fn multi_action_first_wins(
        first in action_strategy(),
        rest in prop::collection::vec(action_strategy(), 1..=4),
    ) {
        let mut cell = vec![first.clone()];
        cell.extend(rest);
        let table = vec![vec![cell]];
        let compressed = compress_action_table(&table);

        prop_assert_eq!(
            decompress_action(&compressed, 0, 0),
            first,
            "decompress must return first action in cell"
        );
    }
}

// ---------------------------------------------------------------------------
// 26. Diagonal goto table
// ---------------------------------------------------------------------------

#[test]
fn diagonal_goto_table() {
    let n = 8;
    let table: Vec<Vec<Option<StateId>>> = (0..n)
        .map(|s| {
            (0..n)
                .map(|sym| {
                    if s == sym {
                        Some(StateId(s as u16))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();

    let compressed = compress_goto_table(&table);

    assert_eq!(
        compressed.entries.len(),
        n,
        "Diagonal has exactly n entries"
    );

    for s in 0..n {
        for sym in 0..n {
            let expected = if s == sym {
                Some(StateId(s as u16))
            } else {
                None
            };
            assert_eq!(decompress_goto(&compressed, s, sym), expected);
        }
    }
}

// ---------------------------------------------------------------------------
// 27. Wide table (many symbols, few states)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn wide_action_table_roundtrip(table in action_table_strategy(2, 64)) {
        let compressed = compress_action_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(
                    decompress_action(&compressed, state, sym),
                    expected,
                );
            }
        }
    }

    #[test]
    fn wide_goto_table_roundtrip(table in goto_table_strategy(2, 64)) {
        let compressed = compress_goto_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (sym, &expected) in row.iter().enumerate() {
                prop_assert_eq!(
                    decompress_goto(&compressed, state, sym),
                    expected,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 28. Tall table (many states, few symbols)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn tall_action_table_roundtrip(table in action_table_strategy(64, 2)) {
        let compressed = compress_action_table(&table);
        prop_assert!(compressed.unique_rows.len() <= table.len());

        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                prop_assert_eq!(
                    decompress_action(&compressed, state, sym),
                    expected,
                );
            }
        }
    }

    #[test]
    fn tall_goto_table_roundtrip(table in goto_table_strategy(64, 2)) {
        let compressed = compress_goto_table(&table);

        for (state, row) in table.iter().enumerate() {
            for (sym, &expected) in row.iter().enumerate() {
                prop_assert_eq!(
                    decompress_goto(&compressed, state, sym),
                    expected,
                );
            }
        }
    }
}
