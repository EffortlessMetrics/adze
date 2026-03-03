#![allow(clippy::needless_range_loop)]
//! Property-based tests for goto table generation in adze-tablegen.
//!
//! Tests cover:
//! - Goto table in generated code
//! - Goto table state transitions
//! - Goto table determinism
//! - Goto table with nonterminals
//! - Goto table size
//! - Goto table empty states
//! - Goto table compression

use adze_ir::StateId;
use adze_tablegen::compress::{CompressedGotoEntry, CompressedGotoTable, TableCompressor};
use adze_tablegen::compression::{compress_goto_table, decompress_goto};
use proptest::prelude::*;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Build an Option-based goto table for the `compression` module.
fn option_table(rows: Vec<Vec<Option<u16>>>) -> Vec<Vec<Option<StateId>>> {
    rows.into_iter()
        .map(|row| row.into_iter().map(|v| v.map(StateId)).collect())
        .collect()
}

/// Build a dense StateId goto table for the `compress` module (TableCompressor).
fn dense_table(rows: Vec<Vec<u16>>) -> Vec<Vec<StateId>> {
    rows.into_iter()
        .map(|row| row.into_iter().map(StateId).collect())
        .collect()
}

/// Expand a CompressedGotoTable (RLE) back to flat rows for verification.
fn expand_compressed(compressed: &CompressedGotoTable) -> Vec<Vec<u16>> {
    let n_rows = compressed.row_offsets.len().saturating_sub(1);
    let mut result = Vec::with_capacity(n_rows);
    for row_idx in 0..n_rows {
        let start = compressed.row_offsets[row_idx] as usize;
        let end = compressed.row_offsets[row_idx + 1] as usize;
        let mut row = Vec::new();
        for entry in &compressed.data[start..end] {
            match entry {
                CompressedGotoEntry::Single(s) => row.push(*s),
                CompressedGotoEntry::RunLength { state, count } => {
                    for _ in 0..*count {
                        row.push(*state);
                    }
                }
            }
        }
        result.push(row);
    }
    result
}

// ── strategies ──────────────────────────────────────────────────────────────

/// Generate a sparse goto table cell: 75% None, 25% Some(state).
fn goto_cell_strategy() -> impl Strategy<Value = Option<u16>> {
    prop_oneof![
        3 => Just(None),
        1 => (0u16..500).prop_map(Some),
    ]
}

/// Generate a sparse Option-based goto table with random dimensions.
fn sparse_goto_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Option<u16>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(goto_cell_strategy(), symbols..=symbols),
            states..=states,
        )
    })
}

/// Generate a dense goto table with random dimensions and values.
fn dense_goto_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<u16>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(0u16..500, symbols..=symbols),
            states..=states,
        )
    })
}

/// Generate a dense goto table where each row is a constant value (for RLE).
fn uniform_row_strategy(max_states: usize, width: usize) -> impl Strategy<Value = Vec<Vec<u16>>> {
    prop::collection::vec(
        (0u16..500).prop_map(move |v| vec![v; width]),
        1..=max_states,
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Goto table in generated code — roundtrip losslessness
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Sparse goto compress/decompress roundtrip is lossless.
    #[test]
    fn sparse_roundtrip_is_lossless(raw in sparse_goto_strategy(10, 10)) {
        let table = option_table(raw.clone());
        let compressed = compress_goto_table(&table);

        for (state, state_row) in raw.iter().enumerate() {
            for (symbol, cell) in state_row.iter().enumerate() {
                let expected = cell.map(StateId);
                let got = decompress_goto(&compressed, state, symbol);
                prop_assert_eq!(got, expected,
                    "Mismatch at state={}, symbol={}", state, symbol);
            }
        }
    }

    /// Dense goto RLE compress/expand roundtrip is lossless.
    #[test]
    fn dense_rle_roundtrip_is_lossless(raw in dense_goto_strategy(8, 12)) {
        let table = dense_table(raw.clone());
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let expanded = expand_compressed(&compressed);

        prop_assert_eq!(expanded.len(), raw.len());
        for i in 0..raw.len() {
            prop_assert_eq!(&expanded[i], &raw[i],
                "Row {} mismatch after RLE roundtrip", i);
        }
    }

    /// Lossless roundtrip holds for tables with uniform rows (heavy RLE).
    #[test]
    fn uniform_rows_roundtrip(raw in uniform_row_strategy(6, 20)) {
        let table = dense_table(raw.clone());
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let expanded = expand_compressed(&compressed);

        for i in 0..raw.len() {
            prop_assert_eq!(&expanded[i], &raw[i]);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Goto table state transitions — lookup correctness
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Every populated entry can be retrieved from the sparse compressed table.
    #[test]
    fn every_populated_entry_retrievable(raw in sparse_goto_strategy(8, 8)) {
        let table = option_table(raw.clone());
        let compressed = compress_goto_table(&table);

        let populated: Vec<(usize, usize, u16)> = raw.iter().enumerate()
            .flat_map(|(s, row)| row.iter().enumerate()
                .filter_map(move |(sym, v)| v.map(|val| (s, sym, val))))
            .collect();

        for (s, sym, val) in &populated {
            prop_assert_eq!(decompress_goto(&compressed, *s, *sym), Some(StateId(*val)));
        }
    }

    /// Unpopulated entries always return None.
    #[test]
    fn unpopulated_entries_return_none(raw in sparse_goto_strategy(8, 8)) {
        let table = option_table(raw.clone());
        let compressed = compress_goto_table(&table);

        for (s, row) in raw.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                if cell.is_none() {
                    prop_assert_eq!(decompress_goto(&compressed, s, sym), None,
                        "Expected None at ({}, {})", s, sym);
                }
            }
        }
    }

    /// Out-of-bounds lookups return None for sparse tables.
    #[test]
    fn out_of_bounds_returns_none(raw in sparse_goto_strategy(5, 5)) {
        let table = option_table(raw.clone());
        let compressed = compress_goto_table(&table);
        let n_states = raw.len();
        let n_syms = if n_states > 0 { raw[0].len() } else { 0 };

        prop_assert_eq!(decompress_goto(&compressed, n_states + 10, 0), None);
        prop_assert_eq!(decompress_goto(&compressed, 0, n_syms + 10), None);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Goto table determinism
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Compressing the same sparse table twice yields identical results.
    #[test]
    fn sparse_compression_deterministic(raw in sparse_goto_strategy(6, 6)) {
        let table = option_table(raw);
        let c1 = compress_goto_table(&table);
        let c2 = compress_goto_table(&table);

        prop_assert_eq!(c1.entries.len(), c2.entries.len());
        for (key, val) in &c1.entries {
            prop_assert_eq!(c2.entries.get(key), Some(val));
        }
    }

    /// Compressing the same dense table twice yields identical RLE output.
    #[test]
    fn dense_rle_compression_deterministic(raw in dense_goto_strategy(6, 10)) {
        let table = dense_table(raw);
        let compressor = TableCompressor::new();
        let c1 = compressor.compress_goto_table_small(&table).unwrap();
        let c2 = compressor.compress_goto_table_small(&table).unwrap();

        prop_assert_eq!(&c1.row_offsets, &c2.row_offsets);
        prop_assert_eq!(c1.data.len(), c2.data.len());
        let e1 = expand_compressed(&c1);
        let e2 = expand_compressed(&c2);
        prop_assert_eq!(e1, e2);
    }

    /// TableCompressor::new() and TableCompressor::default() produce identical output.
    #[test]
    fn new_and_default_identical(raw in dense_goto_strategy(4, 6)) {
        let table = dense_table(raw);
        let c1 = TableCompressor::new().compress_goto_table_small(&table).unwrap();
        let c2 = TableCompressor::default().compress_goto_table_small(&table).unwrap();
        prop_assert_eq!(expand_compressed(&c1), expand_compressed(&c2));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Goto table with nonterminals — symbol coverage
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Wide nonterminal ranges are handled: tables up to 100 columns.
    #[test]
    fn wide_nonterminal_range(n_syms in 10usize..100, val in 0u16..500) {
        let row: Vec<u16> = (0..n_syms).map(|i| val.wrapping_add(i as u16)).collect();
        let table = dense_table(vec![row.clone()]);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let expanded = expand_compressed(&compressed);
        prop_assert_eq!(&expanded[0], &row);
    }

    /// Sparse table with one populated column across many states.
    #[test]
    fn single_column_populated(n_states in 2usize..20, col in 0usize..10) {
        let n_syms = 10;
        let actual_col = col.min(n_syms - 1);
        let mut raw = vec![vec![None; n_syms]; n_states];
        for s in 0..n_states {
            raw[s][actual_col] = Some(s as u16);
        }
        let table = option_table(raw.clone());
        let compressed = compress_goto_table(&table);

        for s in 0..n_states {
            prop_assert_eq!(
                decompress_goto(&compressed, s, actual_col),
                Some(StateId(s as u16))
            );
        }
    }

    /// Diagonal pattern: each state has exactly one populated column.
    #[test]
    fn diagonal_pattern(size in 2usize..15) {
        let mut raw = vec![vec![None; size]; size];
        for i in 0..size {
            raw[i][i] = Some((i + 1) as u16);
        }
        let table = option_table(raw);
        let compressed = compress_goto_table(&table);

        prop_assert_eq!(compressed.entries.len(), size);
        for i in 0..size {
            prop_assert_eq!(
                decompress_goto(&compressed, i, i),
                Some(StateId((i + 1) as u16))
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Goto table size — compression never inflates
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Sparse compression entry count ≤ total non-None cells.
    #[test]
    fn sparse_entries_leq_populated_cells(raw in sparse_goto_strategy(10, 10)) {
        let table = option_table(raw.clone());
        let compressed = compress_goto_table(&table);
        let populated = raw.iter()
            .flat_map(|row| row.iter())
            .filter(|c| c.is_some())
            .count();
        prop_assert!(compressed.entries.len() <= populated,
            "entries={} > populated={}", compressed.entries.len(), populated);
    }

    /// RLE compression never produces more expanded values than original width.
    #[test]
    fn rle_expanded_width_matches_original(raw in dense_goto_strategy(6, 15)) {
        let table = dense_table(raw.clone());
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let expanded = expand_compressed(&compressed);

        for i in 0..raw.len() {
            prop_assert_eq!(expanded[i].len(), raw[i].len(),
                "Row {} width mismatch: expanded {} vs original {}",
                i, expanded[i].len(), raw[i].len());
        }
    }

    /// Uniform rows compress to at most one RLE entry per row.
    #[test]
    fn uniform_rows_compress_well(raw in uniform_row_strategy(5, 20)) {
        let table = dense_table(raw.clone());
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        // Each uniform row of width > 2 should produce exactly one RLE entry
        for i in 0..raw.len() {
            let start = compressed.row_offsets[i] as usize;
            let end = compressed.row_offsets[i + 1] as usize;
            let entries_for_row = end - start;
            // width 20 with uniform values → exactly 1 RunLength entry
            prop_assert_eq!(entries_for_row, 1,
                "Uniform row {} should have 1 RLE entry, got {}", i, entries_for_row);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Goto table empty states
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// All-None rows produce zero sparse entries for that row.
    #[test]
    fn all_none_rows_produce_zero_entries(n_states in 1usize..10, n_syms in 1usize..10) {
        let raw = vec![vec![None; n_syms]; n_states];
        let table = option_table(raw);
        let compressed = compress_goto_table(&table);
        prop_assert_eq!(compressed.entries.len(), 0);
    }

    /// Empty dense rows produce zero RLE entries.
    #[test]
    fn empty_dense_rows_produce_zero_entries(n_states in 1usize..10) {
        let table: Vec<Vec<StateId>> = vec![vec![]; n_states];
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        prop_assert!(compressed.data.is_empty());
        prop_assert_eq!(compressed.row_offsets.len(), n_states + 1);
    }

    /// row_offsets sentinel length is always n_rows + 1.
    #[test]
    fn row_offsets_length_is_rows_plus_one(raw in dense_goto_strategy(8, 5)) {
        let n_rows = raw.len();
        let table = dense_table(raw);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        prop_assert_eq!(compressed.row_offsets.len(), n_rows + 1);
    }

    /// row_offsets are monotonically non-decreasing.
    #[test]
    fn row_offsets_monotonic(raw in dense_goto_strategy(8, 10)) {
        let table = dense_table(raw);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        for i in 1..compressed.row_offsets.len() {
            prop_assert!(compressed.row_offsets[i] >= compressed.row_offsets[i - 1],
                "row_offsets not monotonic at index {}", i);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Goto table compression — RLE properties
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Runs of exactly 2 always produce Single entries, never RunLength.
    #[test]
    fn run_of_two_uses_singles(val in 0u16..500) {
        let table = dense_table(vec![vec![val, val]]);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        let all_single = compressed.data.iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(_)));
        prop_assert!(all_single, "Run of 2 should use Single entries");
        prop_assert_eq!(compressed.data.len(), 2);
    }

    /// Runs of exactly 3 always produce a RunLength entry.
    #[test]
    fn run_of_three_uses_rle(val in 0u16..500) {
        let table = dense_table(vec![vec![val, val, val]]);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        prop_assert_eq!(compressed.data.len(), 1);
        match &compressed.data[0] {
            CompressedGotoEntry::RunLength { state, count } => {
                prop_assert_eq!(*state, val);
                prop_assert_eq!(*count, 3);
            }
            other => prop_assert!(false, "Expected RunLength, got {:?}", other),
        }
    }

    /// Long uniform runs produce a single RunLength entry.
    #[test]
    fn long_run_single_rle_entry(val in 0u16..500, len in 4u16..200) {
        let table = dense_table(vec![vec![val; len as usize]]);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        prop_assert_eq!(compressed.data.len(), 1);
        match &compressed.data[0] {
            CompressedGotoEntry::RunLength { state, count } => {
                prop_assert_eq!(*state, val);
                prop_assert_eq!(*count, len);
            }
            other => prop_assert!(false, "Expected RunLength, got {:?}", other),
        }
    }

    /// All-distinct values produce only Single entries.
    #[test]
    fn all_distinct_uses_singles(len in 3usize..30) {
        let row: Vec<u16> = (0..len as u16).collect();
        let table = dense_table(vec![row]);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        let all_single = compressed.data.iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(_)));
        prop_assert!(all_single, "All distinct values should produce Single entries");
        prop_assert_eq!(compressed.data.len(), len);
    }

    /// Total expanded elements equal the original table's total elements.
    #[test]
    fn total_expanded_elements_match(raw in dense_goto_strategy(6, 12)) {
        let total_orig: usize = raw.iter().map(|r| r.len()).sum();
        let table = dense_table(raw);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let total_expanded: usize = expand_compressed(&compressed)
            .iter().map(|r| r.len()).sum();
        prop_assert_eq!(total_expanded, total_orig);
    }

    /// RLE entry count is always ≤ total original entries.
    #[test]
    fn rle_entry_count_leq_original(raw in dense_goto_strategy(6, 15)) {
        let total_orig: usize = raw.iter().map(|r| r.len()).sum();
        let table = dense_table(raw);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        prop_assert!(compressed.data.len() <= total_orig,
            "RLE entries {} > original entries {}", compressed.data.len(), total_orig);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Goto table default entries — states with uniform targets
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// When every cell in a sparse row targets the same state, all lookups return it.
    #[test]
    fn sparse_uniform_row_default_target(n_syms in 2usize..20, target in 0u16..500) {
        let row: Vec<Option<u16>> = vec![Some(target); n_syms];
        let table = option_table(vec![row]);
        let compressed = compress_goto_table(&table);

        for sym in 0..n_syms {
            prop_assert_eq!(
                decompress_goto(&compressed, 0, sym),
                Some(StateId(target)),
                "Uniform row lookup failed at symbol {}", sym
            );
        }
        prop_assert_eq!(compressed.entries.len(), n_syms);
    }

    /// Sparse table where half the columns share one target and half share another.
    #[test]
    fn sparse_two_default_groups(n_syms in 4usize..20, t1 in 0u16..250, t2 in 250u16..500) {
        let half = n_syms / 2;
        let mut row = vec![Some(t1); half];
        row.extend(vec![Some(t2); n_syms - half]);
        let table = option_table(vec![row.clone()]);
        let compressed = compress_goto_table(&table);

        for sym in 0..half {
            prop_assert_eq!(decompress_goto(&compressed, 0, sym), Some(StateId(t1)));
        }
        for sym in half..n_syms {
            prop_assert_eq!(decompress_goto(&compressed, 0, sym), Some(StateId(t2)));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Goto table empty grammar — zero-dimension edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sparse_empty_table_no_states() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 0);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
}

#[test]
fn dense_empty_table_no_states() {
    let table: Vec<Vec<StateId>> = vec![];
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    assert_eq!(compressed.row_offsets.len(), 1); // sentinel only
    assert!(compressed.data.is_empty());
}

#[test]
fn sparse_single_cell_some() {
    let table = option_table(vec![vec![Some(42)]]);
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(42)));
    assert_eq!(compressed.entries.len(), 1);
}

#[test]
fn sparse_single_cell_none() {
    let table = option_table(vec![vec![None]]);
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(compressed.entries.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Goto table roundtrip — mixed patterns & boundary values
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Alternating values compress and expand correctly.
    #[test]
    fn alternating_values_roundtrip(a in 0u16..250, b in 250u16..500, half_len in 2usize..30) {
        let row: Vec<u16> = (0..half_len * 2).map(|i| if i % 2 == 0 { a } else { b }).collect();
        let table = dense_table(vec![row.clone()]);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let expanded = expand_compressed(&compressed);
        prop_assert_eq!(&expanded[0], &row);
    }

    /// Maximum u16 state values survive roundtrip through sparse compression.
    #[test]
    fn boundary_u16_sparse_roundtrip(val in (u16::MAX - 100)..=u16::MAX) {
        let raw = vec![vec![Some(val), None, Some(val)]];
        let table = option_table(raw);
        let compressed = compress_goto_table(&table);
        prop_assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(val)));
        prop_assert_eq!(decompress_goto(&compressed, 0, 1), None);
        prop_assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(val)));
    }

    /// Maximum u16 state values survive roundtrip through RLE compression.
    #[test]
    fn boundary_u16_dense_roundtrip(val in (u16::MAX - 100)..=u16::MAX, len in 3usize..20) {
        let row = vec![val; len];
        let table = dense_table(vec![row.clone()]);
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let expanded = expand_compressed(&compressed);
        prop_assert_eq!(&expanded[0], &row);
    }

    /// Fully-populated sparse table roundtrips correctly (no None cells).
    #[test]
    fn fully_populated_sparse_roundtrip(
        n_states in 1usize..8,
        n_syms in 1usize..8,
    ) {
        let raw: Vec<Vec<Option<u16>>> = (0..n_states)
            .map(|s| (0..n_syms).map(|sym| Some((s * n_syms + sym) as u16)).collect())
            .collect();
        let table = option_table(raw.clone());
        let compressed = compress_goto_table(&table);

        for s in 0..n_states {
            for sym in 0..n_syms {
                let expected = Some(StateId((s * n_syms + sym) as u16));
                prop_assert_eq!(decompress_goto(&compressed, s, sym), expected);
            }
        }
        prop_assert_eq!(compressed.entries.len(), n_states * n_syms);
    }

    /// Multi-row table with mixed run/single patterns roundtrips through RLE.
    #[test]
    fn mixed_multirow_rle_roundtrip(
        run_val in 0u16..500,
        run_len in 3usize..15,
        tail_len in 1usize..5,
    ) {
        let mut row1: Vec<u16> = vec![run_val; run_len];
        row1.extend((0..tail_len as u16).map(|i| run_val.wrapping_add(i + 1)));
        let row2: Vec<u16> = (0..row1.len() as u16).collect();
        let raw = vec![row1.clone(), row2.clone()];
        let table = dense_table(raw.clone());
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        let expanded = expand_compressed(&compressed);
        prop_assert_eq!(&expanded[0], &raw[0]);
        prop_assert_eq!(&expanded[1], &raw[1]);
    }
}
