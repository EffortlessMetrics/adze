#![allow(clippy::needless_range_loop)]
//! Comprehensive edge-case tests for table compression in adze-tablegen.
//!
//! Covers empty tables, single-state tables, all-identical/all-unique rows,
//! large action cell values, sparse vs dense tables, semantic preservation,
//! and round-trip compress → decompress correctness.

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_tablegen::compress::{
    CompressedActionEntry, CompressedGotoEntry, CompressedParseTable, TableCompressor,
};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use std::collections::BTreeMap;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Wrap each action into a single-element GLR cell (empty for Error).
fn glr_table(rows: Vec<Vec<Action>>) -> Vec<Vec<Vec<Action>>> {
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .map(|a| {
                    if matches!(a, Action::Error) {
                        vec![]
                    } else {
                        vec![a]
                    }
                })
                .collect()
        })
        .collect()
}

/// Verify every cell of an action table survives roundtrip through compression.
fn assert_action_roundtrip(table: &[Vec<Vec<Action>>]) {
    let compressed = compress_action_table(table);
    for (state, row) in table.iter().enumerate() {
        for (sym, cell) in row.iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            let got = decompress_action(&compressed, state, sym);
            assert_eq!(got, expected, "action mismatch state={state} sym={sym}");
        }
    }
}

/// Verify every cell of a goto table survives roundtrip through compression.
fn assert_goto_roundtrip(table: &[Vec<Option<StateId>>]) {
    let compressed = compress_goto_table(table);
    for (state, row) in table.iter().enumerate() {
        for (sym, &expected) in row.iter().enumerate() {
            let got = decompress_goto(&compressed, state, sym);
            assert_eq!(got, expected, "goto mismatch state={state} sym={sym}");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1–2. Empty tables
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_table_completely_empty() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 0);
    assert_eq!(c.state_to_row.len(), 0);
    // States with zero symbol columns collapse to one unique row
    let table2: Vec<Vec<Vec<Action>>> = vec![vec![], vec![], vec![]];
    let c2 = compress_action_table(&table2);
    assert_eq!(c2.unique_rows.len(), 1, "all empty rows are identical");
    assert_eq!(c2.state_to_row.len(), 3);
}

#[test]
fn goto_table_completely_empty() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let c = compress_goto_table(&table);
    assert!(c.entries.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 3–5. Single-state tables
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_table_single_state_single_error() {
    let table = vec![vec![vec![]]];
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 1);
    assert_eq!(decompress_action(&c, 0, 0), Action::Error);
}

#[test]
fn action_table_single_state_shift_and_accept() {
    let table = vec![vec![vec![Action::Shift(StateId(42))]]];
    assert_action_roundtrip(&table);
    let table2 = vec![vec![vec![Action::Accept]]];
    assert_action_roundtrip(&table2);
}

#[test]
fn goto_table_single_state_none_and_some() {
    let t1 = vec![vec![None]];
    assert_goto_roundtrip(&t1);
    assert!(compress_goto_table(&t1).entries.is_empty());
    let t2 = vec![vec![Some(StateId(99))]];
    assert_goto_roundtrip(&t2);
    assert_eq!(compress_goto_table(&t2).entries.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6–8. All identical rows (maximum compression)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_all_identical_50_states() {
    let row = vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(0))],
        vec![],
    ];
    let table: Vec<Vec<Vec<Action>>> = vec![row; 50];
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 1);
    assert_eq!(c.state_to_row.len(), 50);
    assert!(c.state_to_row.iter().all(|&idx| idx == 0));
    assert_action_roundtrip(&table);
}

#[test]
fn goto_all_identical_rows() {
    let row = vec![Some(StateId(1)), None, Some(StateId(2))];
    let table: Vec<Vec<Option<StateId>>> = vec![row; 40];
    assert_goto_roundtrip(&table);
    assert_eq!(compress_goto_table(&table).entries.len(), 40 * 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9–11. All unique rows (minimum compression)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_all_unique_rows() {
    let table: Vec<Vec<Vec<Action>>> = (0u16..25)
        .map(|s| vec![vec![Action::Shift(StateId(s))]])
        .collect();
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 25);
    assert_action_roundtrip(&table);
}

#[test]
fn goto_all_unique_rows() {
    let table: Vec<Vec<Option<StateId>>> =
        (0u16..15).map(|s| vec![Some(StateId(s)), None]).collect();
    assert_goto_roundtrip(&table);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12–14. Very large action cell values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_max_ids_roundtrip() {
    let table = vec![vec![
        vec![Action::Shift(StateId(u16::MAX))],
        vec![Action::Reduce(RuleId(u16::MAX))],
    ]];
    assert_action_roundtrip(&table);
}

#[test]
fn goto_max_state_id() {
    let table = vec![vec![Some(StateId(u16::MAX))]];
    assert_goto_roundtrip(&table);
}

#[test]
fn encode_shift_boundary_values() {
    let compressor = TableCompressor::new();
    // 0x7FFE is valid
    assert_eq!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x7FFE)))
            .unwrap(),
        0x7FFE,
    );
    // 0x7FFF is still valid (< 0x8000)
    assert!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x7FFF)))
            .is_ok()
    );
    // 0x8000 is rejected
    assert!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x8000)))
            .is_err()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 15–18. Sparse vs dense tables
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_extremely_sparse_diagonal() {
    let n = 20;
    let table: Vec<Vec<Vec<Action>>> = (0..n)
        .map(|s| {
            (0..n)
                .map(|sym| {
                    if s == sym {
                        vec![Action::Shift(StateId(s as u16))]
                    } else {
                        vec![]
                    }
                })
                .collect()
        })
        .collect();
    assert_action_roundtrip(&table);
    assert_eq!(compress_action_table(&table).unique_rows.len(), n);
}

#[test]
fn action_fully_dense_table() {
    let table: Vec<Vec<Vec<Action>>> = (0u16..5)
        .map(|s| {
            (0u16..8)
                .map(|sym| vec![Action::Shift(StateId(s * 8 + sym))])
                .collect()
        })
        .collect();
    assert_action_roundtrip(&table);
}

#[test]
fn goto_extremely_sparse_diagonal() {
    let n = 30;
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
    assert_goto_roundtrip(&table);
    assert_eq!(compress_goto_table(&table).entries.len(), n);
}

#[test]
fn goto_fully_dense() {
    let table: Vec<Vec<Option<StateId>>> = (0u16..4)
        .map(|s| (0u16..6).map(|sym| Some(StateId(s * 6 + sym))).collect())
        .collect();
    assert_goto_roundtrip(&table);
    assert_eq!(compress_goto_table(&table).entries.len(), 4 * 6);
}

// ═══════════════════════════════════════════════════════════════════════════
// 19–21. Compression preserves action semantics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_semantics_mixed_row_all_variants() {
    let table = glr_table(vec![vec![
        Action::Error,
        Action::Shift(StateId(10)),
        Action::Reduce(RuleId(5)),
        Action::Accept,
        Action::Recover,
    ]]);
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Error);
    assert_eq!(decompress_action(&c, 0, 1), Action::Shift(StateId(10)));
    assert_eq!(decompress_action(&c, 0, 2), Action::Reduce(RuleId(5)));
    assert_eq!(decompress_action(&c, 0, 3), Action::Accept);
    assert_eq!(decompress_action(&c, 0, 4), Action::Recover);
}

#[test]
fn multi_action_cell_first_action_wins() {
    let table = vec![vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))],
        vec![Action::Reduce(RuleId(3)), Action::Accept],
    ]];
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Shift(StateId(1)));
    assert_eq!(decompress_action(&c, 0, 1), Action::Reduce(RuleId(3)));
}

#[test]
fn empty_cells_decompress_to_error() {
    let table = vec![vec![vec![], vec![], vec![]]];
    let c = compress_action_table(&table);
    for sym in 0..3 {
        assert_eq!(decompress_action(&c, 0, sym), Action::Error);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 22–25. Round-trip: compress → decompress gives same tables
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_roundtrip_many_states_many_symbols() {
    let table: Vec<Vec<Vec<Action>>> = (0u16..10)
        .map(|s| {
            (0u16..15)
                .map(|sym| match (s + sym) % 4 {
                    0 => vec![],
                    1 => vec![Action::Shift(StateId(sym))],
                    2 => vec![Action::Reduce(RuleId(s))],
                    _ => vec![Action::Accept],
                })
                .collect()
        })
        .collect();
    assert_action_roundtrip(&table);
}

#[test]
fn goto_roundtrip_many_states_many_symbols() {
    let table: Vec<Vec<Option<StateId>>> = (0u16..12)
        .map(|s| {
            (0u16..10)
                .map(|sym| {
                    if (s + sym) % 3 == 0 {
                        Some(StateId(s + sym))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    assert_goto_roundtrip(&table);
}

#[test]
fn action_roundtrip_wide_row_300_symbols() {
    let row: Vec<Action> = (0u16..300)
        .map(|i| match i % 5 {
            0 => Action::Shift(StateId(i % 100)),
            1 => Action::Reduce(RuleId(i % 50)),
            2 => Action::Accept,
            3 => Action::Error,
            _ => Action::Recover,
        })
        .collect();
    assert_action_roundtrip(&glr_table(vec![row]));
}

#[test]
fn goto_roundtrip_wide_row_400_symbols() {
    let row: Vec<Option<StateId>> = (0u16..400)
        .map(|i| {
            if i % 7 == 0 {
                Some(StateId(i % 200))
            } else {
                None
            }
        })
        .collect();
    assert_goto_roundtrip(&[row]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 26–27. Row deduplication edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_dedup_interleaved_duplicates() {
    let a = vec![vec![Action::Shift(StateId(0))]];
    let b = vec![vec![Action::Reduce(RuleId(0))]];
    let c = vec![vec![Action::Accept]];
    let table = vec![a.clone(), b.clone(), a, b, c];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 3);
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[2]);
    assert_eq!(compressed.state_to_row[1], compressed.state_to_row[3]);
}

#[test]
fn action_dedup_preserves_order_mapping() {
    let table: Vec<Vec<Vec<Action>>> = vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Shift(StateId(2))]],
        vec![vec![Action::Shift(StateId(1))]],
    ];
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 2);
    assert_eq!(c.state_to_row[0], c.state_to_row[2]);
    assert_ne!(c.state_to_row[0], c.state_to_row[1]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 28. BitPackedActionTable edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn bitpacked_all_errors_and_empty() {
    // All-error table
    let table = vec![vec![Action::Error; 10]; 5];
    let packed = BitPackedActionTable::from_table(&table);
    for state in 0..5 {
        for sym in 0..10 {
            assert_eq!(packed.decompress(state, sym), Action::Error);
        }
    }
    // Empty table doesn't panic
    let empty: Vec<Vec<Action>> = vec![];
    let _packed = BitPackedActionTable::from_table(&empty);
}

// ═══════════════════════════════════════════════════════════════════════════
// 29. CompressedParseTable dimension edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_parse_table_dimensions() {
    let t0 = CompressedParseTable::new_for_testing(0, 0);
    assert_eq!(t0.symbol_count(), 0);
    assert_eq!(t0.state_count(), 0);
    let t1 = CompressedParseTable::new_for_testing(500, 3);
    assert_eq!(t1.symbol_count(), 500);
    assert_eq!(t1.state_count(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 30. Small-table compressor row offsets
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn small_table_row_offsets_invariants() {
    let compressor = TableCompressor::new();
    // Test non-decreasing and length == n+1
    let action_table = vec![
        vec![vec![]; 4],
        vec![vec![Action::Shift(StateId(1))]; 4],
        vec![vec![]; 4],
        vec![vec![Action::Reduce(RuleId(0))]; 4],
    ];
    let sym_map = BTreeMap::new();
    let c = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    assert_eq!(c.row_offsets.len(), action_table.len() + 1);
    for pair in c.row_offsets.windows(2) {
        assert!(pair[1] >= pair[0], "offsets must be non-decreasing");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 31. Goto RLE edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_rle_short_and_long_runs() {
    let compressor = TableCompressor::new();
    // Run of 2 stays as singles
    let t1 = vec![vec![StateId(3), StateId(3)]];
    let c1 = compressor.compress_goto_table_small(&t1).unwrap();
    assert_eq!(c1.data.len(), 2);
    assert!(
        c1.data
            .iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(3)))
    );
    // Run of 100 compresses to single RunLength
    let t2 = vec![vec![StateId(7); 100]];
    let c2 = compressor.compress_goto_table_small(&t2).unwrap();
    assert_eq!(c2.data.len(), 1);
    assert!(matches!(
        c2.data[0],
        CompressedGotoEntry::RunLength {
            state: 7,
            count: 100
        }
    ));
    // Sentinel offset equals data length
    let t3 = vec![
        vec![StateId(1), StateId(2)],
        vec![StateId(3), StateId(3), StateId(3)],
    ];
    let c3 = compressor.compress_goto_table_small(&t3).unwrap();
    assert_eq!(*c3.row_offsets.last().unwrap() as usize, c3.data.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// 33. Encoding sentinels
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn encoding_sentinels_distinct_and_reduce_one_based() {
    let compressor = TableCompressor::new();
    let accept = compressor.encode_action_small(&Action::Accept).unwrap();
    let error = compressor.encode_action_small(&Action::Error).unwrap();
    let recover = compressor.encode_action_small(&Action::Recover).unwrap();
    let mut vals = vec![accept, error, recover];
    vals.sort();
    vals.dedup();
    assert_eq!(
        vals.len(),
        3,
        "Accept, Error, Recover must have distinct encodings"
    );
    // Reduce rule 0 is 1-based
    let r0 = compressor
        .encode_action_small(&Action::Reduce(RuleId(0)))
        .unwrap();
    assert_eq!(r0, 0x8001);
}

// ═══════════════════════════════════════════════════════════════════════════
// 34. Small-table skips Error actions and default is always Error
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn small_table_skips_errors_and_defaults_to_error() {
    let compressor = TableCompressor::new();
    let action_table = vec![vec![
        vec![Action::Error],
        vec![Action::Shift(StateId(1))],
        vec![Action::Error],
        vec![Action::Reduce(RuleId(2))],
    ]];
    let sym_map = BTreeMap::new();
    let c = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    assert_eq!(c.data.len(), 2);
    assert_eq!(c.data[0].action, Action::Shift(StateId(1)));
    assert_eq!(c.data[1].action, Action::Reduce(RuleId(2)));
    for d in &c.default_actions {
        assert_eq!(*d, Action::Error);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 35. CompressedActionEntry construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_action_entry_all_variants() {
    let variants = vec![
        Action::Shift(StateId(0)),
        Action::Shift(StateId(u16::MAX)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(u16::MAX)),
        Action::Accept,
        Action::Error,
        Action::Recover,
    ];
    for (i, action) in variants.iter().enumerate() {
        let entry = CompressedActionEntry::new(i as u16, action.clone());
        assert_eq!(entry.symbol, i as u16);
        assert_eq!(entry.action, *action);
    }
}
