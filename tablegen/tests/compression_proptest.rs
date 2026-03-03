// Property-based tests for table compression algorithms.
//
// Properties verified:
// 1. Compression is lossless (decompress(compress(table)) == table)
// 2. Compressed size <= uncompressed size (row dedup never inflates)
// 3. State count is preserved through compression
// 4. Symbol count is preserved through compression
// 5. Empty table compresses to empty
// 6. Single-state table roundtrips
// 7. Identity compression (trivial tables compress to themselves)
// 8. Large random tables don't panic
// 9. All actions in compressed table match original (full cell)
// 10. Compression is deterministic (same input → same output)
// Additional:
// - Row deduplication preserves action semantics
// - Sparse representation handles all-zero rows correctly
// - Offset computation is injective (no state overlap)

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_tablegen::compression::{
    compress_action_table, compress_goto_table, decompress_action, decompress_goto,
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
// 1. Compression is lossless: decompress(compress(table)) == table
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
// 2. Compressed size <= uncompressed size (row dedup never inflates)
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
// 3. Row deduplication preserves action semantics
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
// 4. Sparse representation handles all-zero rows correctly
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn all_none_goto_rows_produce_no_entries(n_states in 1usize..=8, n_symbols in 1usize..=8) {
        let table: Vec<Vec<Option<StateId>>> = vec![vec![None; n_symbols]; n_states];
        let compressed = compress_goto_table(&table);

        prop_assert_eq!(
            compressed.entries.len(),
            0,
            "All-None table should produce 0 sparse entries"
        );

        for (state, state_row) in table.iter().enumerate() {
            for (symbol, _) in state_row.iter().enumerate() {
                prop_assert_eq!(decompress_goto(&compressed, state, symbol), None);
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

        for (state, state_row) in table.iter().enumerate() {
            for (symbol, _) in state_row.iter().enumerate() {
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
// 5. Offset computation is injective (no state overlap)
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
// 3. State count is preserved through compression
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
        // Every original cell must be recoverable
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
}

// ---------------------------------------------------------------------------
// 4. Symbol count is preserved through compression
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

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
            // Non-None entries must have symbol indices within range
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
}

// ---------------------------------------------------------------------------
// 5. Empty table compresses to empty
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

// ---------------------------------------------------------------------------
// 6. Single-state table roundtrips
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
}

// ---------------------------------------------------------------------------
// 7. Identity compression (trivial tables compress to themselves)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_distinct_rows_yield_identity_mapping(
        n_symbols in 1usize..=4,
        n_states in 1usize..=6,
    ) {
        // Build a table where every row is unique (distinct shift targets)
        let table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|s| {
                (0..n_symbols)
                    .map(|sym| vec![Action::Shift(StateId((s * n_symbols + sym) as u16))])
                    .collect()
            })
            .collect();

        let compressed = compress_action_table(&table);

        // All rows are unique so unique_rows.len() == state count
        prop_assert_eq!(
            compressed.unique_rows.len(),
            n_states,
            "All-distinct rows should not be deduplicated"
        );

        // state_to_row should be identity [0, 1, 2, ...]
        for (i, &row_idx) in compressed.state_to_row.iter().enumerate() {
            prop_assert_eq!(row_idx, i, "Identity mapping broken at state {}", i);
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Large random tables don't panic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn large_action_table_no_panic(table in action_table_strategy(64, 32)) {
        let compressed = compress_action_table(&table);

        // Basic structural invariant
        prop_assert_eq!(compressed.state_to_row.len(), table.len());
        prop_assert!(compressed.unique_rows.len() <= table.len());

        // Spot-check a few cells
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

        // Spot-check corners
        if !table.is_empty() && !table[0].is_empty() {
            let _ = decompress_goto(&compressed, 0, 0);
            let _ = decompress_goto(&compressed, table.len() - 1, table[0].len() - 1);
        }
    }
}

// ---------------------------------------------------------------------------
// 9. All actions in compressed table match original (full cell comparison)
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

// ---------------------------------------------------------------------------
// 10. Compression is deterministic (same input → same output)
// ---------------------------------------------------------------------------

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
