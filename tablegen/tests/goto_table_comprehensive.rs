#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for GOTO table generation and compression in adze-tablegen.
//!
//! The GOTO table maps (state, nonterminal) → next_state.
//! Covers: construction, empty/single-state tables, sparse/dense patterns,
//! RLE compression, multi-nonterminal handling, and lookup after compression.

use adze_ir::StateId;
use adze_tablegen::compress::{CompressedGotoEntry, CompressedGotoTable, TableCompressor};
use adze_tablegen::compression::{compress_goto_table, decompress_goto};

// ── helpers ─────────────────────────────────────────────────────────────────

/// Build an Option-based goto table for the `compression` module functions.
fn option_table(rows: Vec<Vec<Option<u16>>>) -> Vec<Vec<Option<StateId>>> {
    rows.into_iter()
        .map(|row| row.into_iter().map(|v| v.map(StateId)).collect())
        .collect()
}

/// Build a dense StateId goto table for `compress` module (TableCompressor).
fn dense_table(rows: Vec<Vec<u16>>) -> Vec<Vec<StateId>> {
    rows.into_iter()
        .map(|row| row.into_iter().map(StateId).collect())
        .collect()
}

/// Decompress an RLE-compressed goto table back to a flat vector per row.
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

// ═══════════════════════════════════════════════════════════════════════════
// 1. GOTO table construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_construction_basic() {
    let table = option_table(vec![
        vec![None, Some(1), None],
        vec![Some(2), None, None],
        vec![None, None, Some(3)],
    ]);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
}

#[test]
fn goto_construction_preserves_all_entries() {
    let table = option_table(vec![
        vec![Some(10), Some(20)],
        vec![Some(30), Some(40)],
    ]);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(10)));
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(20)));
    assert_eq!(decompress_goto(&compressed, 1, 0), Some(StateId(30)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(40)));
}

#[test]
fn goto_construction_none_entries_not_stored() {
    let table = option_table(vec![vec![None, None, None]]);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 0);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(decompress_goto(&compressed, 0, 2), None);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Empty GOTO table
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_empty_table_sparse() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 0);
}

#[test]
fn goto_empty_table_compressor() {
    let table: Vec<Vec<StateId>> = vec![];
    let compressor = TableCompressor::new();
    let result = compressor.compress_goto_table_small(&table);
    assert!(result.is_ok());
    let compressed = result.unwrap();
    assert!(compressed.data.is_empty());
    assert_eq!(compressed.row_offsets.len(), 1); // sentinel only
}

#[test]
fn goto_empty_rows() {
    let table: Vec<Vec<StateId>> = vec![vec![], vec![], vec![]];
    let compressor = TableCompressor::new();
    let result = compressor.compress_goto_table_small(&table).unwrap();
    assert!(result.data.is_empty());
    assert_eq!(result.row_offsets.len(), 4); // 3 rows + 1 sentinel
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Single-state GOTO
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_single_state_single_nt() {
    let table = option_table(vec![vec![Some(5)]]);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(5)));
}

#[test]
fn goto_single_state_dense_compressor() {
    let table = dense_table(vec![vec![7, 8, 9]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded.len(), 1);
    assert_eq!(expanded[0], vec![7, 8, 9]);
}

#[test]
fn goto_single_state_all_same() {
    let table = dense_table(vec![vec![4, 4, 4, 4]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    // 4 identical values → RLE (count > 2)
    let has_rle = compressed
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 4, count: 4 }));
    assert!(has_rle, "Expected RLE for 4 identical values");
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded[0], vec![4, 4, 4, 4]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Sparse GOTO table compression
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_sparse_diagonal() {
    // Only diagonal entries are populated
    let table = option_table(vec![
        vec![Some(1), None, None, None],
        vec![None, Some(2), None, None],
        vec![None, None, Some(3), None],
        vec![None, None, None, Some(4)],
    ]);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
    for i in 0..4 {
        assert_eq!(
            decompress_goto(&compressed, i, i),
            Some(StateId((i + 1) as u16))
        );
        // off-diagonal is None
        if i + 1 < 4 {
            assert_eq!(decompress_goto(&compressed, i, i + 1), None);
        }
    }
}

#[test]
fn goto_sparse_single_column() {
    let table = option_table(vec![
        vec![None, Some(10), None],
        vec![None, Some(20), None],
        vec![None, Some(30), None],
    ]);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
    for i in 0..3 {
        assert_eq!(
            decompress_goto(&compressed, i, 1),
            Some(StateId(((i + 1) * 10) as u16))
        );
    }
}

#[test]
fn goto_sparse_very_sparse() {
    // 10 states × 10 nonterminals but only 2 entries
    let mut rows = vec![vec![None; 10]; 10];
    rows[0][5] = Some(StateId(42));
    rows[9][0] = Some(StateId(99));
    let compressed = compress_goto_table(&rows);
    assert_eq!(compressed.entries.len(), 2);
    assert_eq!(decompress_goto(&compressed, 0, 5), Some(StateId(42)));
    assert_eq!(decompress_goto(&compressed, 9, 0), Some(StateId(99)));
    assert_eq!(decompress_goto(&compressed, 5, 5), None);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Dense GOTO table handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_dense_all_populated() {
    let table = option_table(vec![
        vec![Some(1), Some(2), Some(3)],
        vec![Some(4), Some(5), Some(6)],
    ]);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 6);
    for state in 0..2 {
        for sym in 0..3 {
            let expected = (state * 3 + sym + 1) as u16;
            assert_eq!(
                decompress_goto(&compressed, state, sym),
                Some(StateId(expected))
            );
        }
    }
}

#[test]
fn goto_dense_compressor_roundtrip() {
    let table = dense_table(vec![
        vec![1, 2, 3, 4, 5],
        vec![6, 7, 8, 9, 10],
        vec![11, 12, 13, 14, 15],
    ]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded.len(), 3);
    assert_eq!(expanded[0], vec![1, 2, 3, 4, 5]);
    assert_eq!(expanded[1], vec![6, 7, 8, 9, 10]);
    assert_eq!(expanded[2], vec![11, 12, 13, 14, 15]);
}

#[test]
fn goto_dense_row_offsets_monotonic() {
    let table = dense_table(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    for i in 1..compressed.row_offsets.len() {
        assert!(
            compressed.row_offsets[i] >= compressed.row_offsets[i - 1],
            "row_offsets must be monotonically increasing"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. GOTO with many nonterminals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_many_nonterminals_sparse() {
    // 5 states × 50 nonterminals
    let mut rows = vec![vec![None; 50]; 5];
    for state in 0..5 {
        rows[state][state * 10] = Some(StateId(state as u16));
    }
    let compressed = compress_goto_table(&rows);
    assert_eq!(compressed.entries.len(), 5);
    for state in 0..5 {
        assert_eq!(
            decompress_goto(&compressed, state, state * 10),
            Some(StateId(state as u16))
        );
    }
}

#[test]
fn goto_many_nonterminals_dense_compressor() {
    // 3 states × 20 nonterminals, all different
    let table = dense_table(
        (0..3)
            .map(|s| (0..20).map(|n| (s * 20 + n) as u16).collect())
            .collect(),
    );
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded.len(), 3);
    for s in 0..3 {
        for n in 0..20 {
            assert_eq!(expanded[s][n], (s * 20 + n) as u16);
        }
    }
}

#[test]
fn goto_100_nonterminals_alternating() {
    // Dense table: 2 states, 100 nonterminals, alternating between 2 values
    let table = dense_table(vec![
        (0..100).map(|i| if i % 2 == 0 { 1 } else { 2 }).collect(),
        (0..100).map(|i| if i % 2 == 0 { 3 } else { 4 }).collect(),
    ]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    for i in 0..100 {
        assert_eq!(expanded[0][i], if i % 2 == 0 { 1 } else { 2 });
        assert_eq!(expanded[1][i], if i % 2 == 0 { 3 } else { 4 });
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. RLE compression of GOTO
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_rle_long_run() {
    // A run of 10 identical states → should use RunLength
    let table = dense_table(vec![vec![5; 10]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let has_rle = compressed
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 5, count: 10 }));
    assert!(has_rle, "Expected RLE for 10 identical values");
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded[0], vec![5; 10]);
}

#[test]
fn goto_rle_run_of_two_uses_singles() {
    // Runs of exactly 2 should use Single entries (more efficient)
    let table = dense_table(vec![vec![3, 3]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let all_single = compressed
        .data
        .iter()
        .all(|e| matches!(e, CompressedGotoEntry::Single(_)));
    assert!(all_single, "Run of 2 should use Single entries, not RLE");
    assert_eq!(compressed.data.len(), 2);
}

#[test]
fn goto_rle_run_of_three_uses_rle() {
    let table = dense_table(vec![vec![7, 7, 7]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let has_rle = compressed
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 7, count: 3 }));
    assert!(has_rle, "Run of 3 should use RunLength");
}

#[test]
fn goto_rle_mixed_runs_and_singles() {
    // Pattern: [1, 1, 1, 2, 3, 3, 3, 3]
    let table = dense_table(vec![vec![1, 1, 1, 2, 3, 3, 3, 3]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded[0], vec![1, 1, 1, 2, 3, 3, 3, 3]);
    // Verify RLE entries exist for the runs
    let rle_count = compressed
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::RunLength { .. }))
        .count();
    assert!(rle_count >= 2, "Should have at least 2 RLE entries");
}

#[test]
fn goto_rle_no_runs_all_distinct() {
    let table = dense_table(vec![vec![1, 2, 3, 4, 5]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    // All entries should be Single since no adjacent duplicates
    let all_single = compressed
        .data
        .iter()
        .all(|e| matches!(e, CompressedGotoEntry::Single(_)));
    assert!(all_single, "All distinct values should produce Single entries");
    assert_eq!(compressed.data.len(), 5);
}

#[test]
fn goto_rle_multiple_rows_independent() {
    // Each row's RLE is independent
    let table = dense_table(vec![
        vec![1, 1, 1, 1],  // one RLE(1,4)
        vec![2, 3, 4, 5],  // four Singles
        vec![9, 9, 9, 9],  // one RLE(9,4)
    ]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded[0], vec![1, 1, 1, 1]);
    assert_eq!(expanded[1], vec![2, 3, 4, 5]);
    assert_eq!(expanded[2], vec![9, 9, 9, 9]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. GOTO lookup after compression
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_lookup_sparse_roundtrip() {
    let table = option_table(vec![
        vec![Some(10), None, Some(20), None, Some(30)],
        vec![None, Some(40), None, Some(50), None],
    ]);
    let compressed = compress_goto_table(&table);
    // Verify all populated entries
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(10)));
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(20)));
    assert_eq!(decompress_goto(&compressed, 0, 4), Some(StateId(30)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(40)));
    assert_eq!(decompress_goto(&compressed, 1, 3), Some(StateId(50)));
    // Verify None entries
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(decompress_goto(&compressed, 0, 3), None);
    assert_eq!(decompress_goto(&compressed, 1, 0), None);
    assert_eq!(decompress_goto(&compressed, 1, 2), None);
    assert_eq!(decompress_goto(&compressed, 1, 4), None);
}

#[test]
fn goto_lookup_missing_key_returns_none() {
    let table = option_table(vec![vec![Some(1)]]);
    let compressed = compress_goto_table(&table);
    // Out-of-range lookups
    assert_eq!(decompress_goto(&compressed, 5, 0), None);
    assert_eq!(decompress_goto(&compressed, 0, 5), None);
    assert_eq!(decompress_goto(&compressed, 100, 100), None);
}

#[test]
fn goto_lookup_dense_compressor_expanded_matches_original() {
    let original = vec![
        vec![10u16, 20, 30, 40],
        vec![50, 60, 70, 80],
        vec![90, 100, 110, 120],
    ];
    let table = dense_table(original.clone());
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded, original);
}

#[test]
fn goto_lookup_after_rle_roundtrip() {
    // Large runs then individual values
    let mut row = vec![0u16; 50];
    for i in 0..20 {
        row[i] = 1;
    }
    for i in 20..35 {
        row[i] = 2;
    }
    for i in 35..50 {
        row[i] = (i - 34) as u16; // 1,2,3,...,15
    }
    let table = dense_table(vec![row.clone()]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    let expanded = expand_compressed(&compressed);
    assert_eq!(expanded[0], row);
}

#[test]
fn goto_row_offsets_sentinel() {
    // row_offsets should have len == n_rows + 1
    let table = dense_table(vec![
        vec![1, 2],
        vec![3, 4],
        vec![5, 6],
    ]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    assert_eq!(compressed.row_offsets.len(), 4); // 3 rows + sentinel
    // Last sentinel must equal total entries
    let total_expanded: usize = expand_compressed(&compressed)
        .iter()
        .map(|r| r.len())
        .sum();
    assert_eq!(total_expanded, 6);
}

#[test]
fn goto_compressor_default_matches_new() {
    let compressor_new = TableCompressor::new();
    let compressor_default = TableCompressor::default();
    let table = dense_table(vec![vec![1, 2, 3]]);
    let r1 = compressor_new.compress_goto_table_small(&table).unwrap();
    let r2 = compressor_default.compress_goto_table_small(&table).unwrap();
    let e1 = expand_compressed(&r1);
    let e2 = expand_compressed(&r2);
    assert_eq!(e1, e2);
}

#[test]
fn goto_large_state_ids() {
    // Test with large StateId values near u16::MAX
    let table = option_table(vec![
        vec![Some(65000), None, Some(65534)],
        vec![None, Some(32000), None],
    ]);
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(65000)));
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(65534)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(32000)));
}

#[test]
fn goto_rle_boundary_exactly_three() {
    // Boundary: exactly 3 identical → should use RunLength (threshold is > 2)
    let table = dense_table(vec![vec![42, 42, 42]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    assert_eq!(compressed.data.len(), 1);
    assert!(matches!(
        compressed.data[0],
        CompressedGotoEntry::RunLength {
            state: 42,
            count: 3
        }
    ));
}

#[test]
fn goto_rle_boundary_exactly_two() {
    // Boundary: exactly 2 identical → should use 2 Singles
    let table = dense_table(vec![vec![42, 42]]);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress_goto_table_small(&table).unwrap();
    assert_eq!(compressed.data.len(), 2);
    assert!(matches!(compressed.data[0], CompressedGotoEntry::Single(42)));
    assert!(matches!(compressed.data[1], CompressedGotoEntry::Single(42)));
}

#[test]
fn goto_sparse_compression_ratio() {
    // A 20×20 table with only 5 entries should compress well
    let mut rows = vec![vec![None; 20]; 20];
    rows[0][0] = Some(StateId(1));
    rows[5][10] = Some(StateId(2));
    rows[10][5] = Some(StateId(3));
    rows[15][15] = Some(StateId(4));
    rows[19][19] = Some(StateId(5));
    let compressed = compress_goto_table(&rows);
    assert_eq!(compressed.entries.len(), 5);
    // Full table would be 400 cells, but we only store 5
    assert!(compressed.entries.len() < 20 * 20);
}
