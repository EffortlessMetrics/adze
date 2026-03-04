//! Comprehensive tests for table compression algorithms in adze-tablegen.
//!
//! Covers: roundtrip correctness, compressed size vs original, edge cases
//! (empty, single-entry, large tables), identity/sparse/dense patterns,
//! lookup correctness after compression, and determinism of output.

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

/// Wrap single actions into GLR cells (empty vec for Error).
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

/// Assert every cell roundtrips through action table compression.
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

/// Assert every cell roundtrips through goto table compression.
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
// 1. Compress/decompress roundtrip correctness
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn roundtrip_action_shift_only() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))],
        vec![Action::Shift(StateId(3)), Action::Shift(StateId(4))],
    ]);
    assert_action_roundtrip(&table);
}

#[test]
fn roundtrip_action_reduce_only() {
    let table = glr_table(vec![
        vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
        vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(3))],
    ]);
    assert_action_roundtrip(&table);
}

#[test]
fn roundtrip_action_mixed_actions() {
    let table = glr_table(vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
    ]]);
    assert_action_roundtrip(&table);
}

#[test]
fn roundtrip_action_accept_cells() {
    let table = glr_table(vec![
        vec![Action::Accept, Action::Error],
        vec![Action::Error, Action::Accept],
    ]);
    assert_action_roundtrip(&table);
}

#[test]
fn roundtrip_goto_mixed() {
    let table = vec![
        vec![Some(StateId(1)), None, Some(StateId(3))],
        vec![None, Some(StateId(2)), None],
        vec![Some(StateId(0)), Some(StateId(4)), Some(StateId(5))],
    ];
    assert_goto_roundtrip(&table);
}

#[test]
fn roundtrip_goto_all_present() {
    let table = vec![vec![
        Some(StateId(0)),
        Some(StateId(1)),
        Some(StateId(2)),
        Some(StateId(3)),
    ]];
    assert_goto_roundtrip(&table);
}

#[test]
fn roundtrip_action_glr_multi_action_cells() {
    // GLR tables can have multiple actions per cell; roundtrip returns the first.
    let table = vec![vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        vec![Action::Reduce(RuleId(1))],
    ]];
    let compressed = compress_action_table(&table);
    // First action should be returned
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
    assert_eq!(
        decompress_action(&compressed, 0, 1),
        Action::Reduce(RuleId(1))
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Compressed table size vs original
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_dedup_reduces_unique_rows() {
    // 10 identical rows should compress to 1 unique row.
    let row = vec![
        vec![Action::Shift(StateId(5))],
        vec![Action::Reduce(RuleId(2))],
    ];
    let table: Vec<Vec<Vec<Action>>> = vec![row; 10];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 10);
}

#[test]
fn action_dedup_two_distinct_patterns() {
    let row_a = vec![vec![Action::Shift(StateId(0))], vec![]];
    let row_b = vec![vec![], vec![Action::Reduce(RuleId(0))]];
    let table = vec![
        row_a.clone(),
        row_b.clone(),
        row_a.clone(),
        row_b.clone(),
        row_a,
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
    assert_eq!(compressed.state_to_row.len(), 5);
}

#[test]
fn goto_sparse_has_fewer_entries_than_cells() {
    // 5x5 table with only 3 non-None entries.
    let mut table = vec![vec![None; 5]; 5];
    table[0][1] = Some(StateId(10));
    table[2][3] = Some(StateId(20));
    table[4][0] = Some(StateId(30));
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
}

#[test]
fn goto_dense_has_all_entries() {
    let table = vec![vec![Some(StateId(1)), Some(StateId(2)), Some(StateId(3))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Edge cases: empty tables, single-entry, very large tables
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_empty_table_zero_states() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let compressed = compress_action_table(&table);
    assert!(compressed.unique_rows.is_empty());
    assert!(compressed.state_to_row.is_empty());
}

#[test]
fn goto_empty_table_zero_states() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn action_single_error_cell() {
    let table = vec![vec![vec![]]]; // one state, one symbol, empty cell = Error
    assert_action_roundtrip(&table);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn goto_single_none_cell() {
    let table = vec![vec![None]];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
}

#[test]
fn action_large_table_50_states_100_symbols() {
    let table: Vec<Vec<Vec<Action>>> = (0u16..50)
        .map(|s| {
            (0u16..100)
                .map(|sym| {
                    if (s + sym) % 5 == 0 {
                        vec![Action::Shift(StateId(sym))]
                    } else if (s + sym) % 7 == 0 {
                        vec![Action::Reduce(RuleId(s))]
                    } else {
                        vec![]
                    }
                })
                .collect()
        })
        .collect();
    assert_action_roundtrip(&table);
}

#[test]
fn goto_large_table_50_states_50_symbols() {
    let table: Vec<Vec<Option<StateId>>> = (0u16..50)
        .map(|s| {
            (0u16..50)
                .map(|sym| {
                    if (s + sym) % 3 == 0 {
                        Some(StateId((s * 50 + sym) % 100))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    assert_goto_roundtrip(&table);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Identity matrices, sparse vs dense tables
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_identity_matrix() {
    // Diagonal pattern: state i goes to state i at symbol i.
    let n = 8;
    let table: Vec<Vec<Option<StateId>>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i == j {
                        Some(StateId(i as u16))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    assert_goto_roundtrip(&table);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), n);
}

#[test]
fn action_identity_shift_diagonal() {
    // Only diagonal cells have Shift; rest are Error.
    let n = 6;
    let table: Vec<Vec<Vec<Action>>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i == j {
                        vec![Action::Shift(StateId(i as u16))]
                    } else {
                        vec![]
                    }
                })
                .collect()
        })
        .collect();
    assert_action_roundtrip(&table);
    // All rows are distinct in the diagonal case.
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), n);
}

#[test]
fn goto_fully_dense_table() {
    // Every cell has a value.
    let table: Vec<Vec<Option<StateId>>> = (0u16..4)
        .map(|s| (0u16..4).map(|sym| Some(StateId(s * 4 + sym))).collect())
        .collect();
    assert_goto_roundtrip(&table);
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 16);
}

#[test]
fn goto_fully_sparse_table() {
    // Every cell is None.
    let table = vec![vec![None; 10]; 10];
    assert_goto_roundtrip(&table);
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn action_fully_error_table() {
    // Every cell is Error (empty vec).
    let table: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; 8]; 8];
    assert_action_roundtrip(&table);
    let compressed = compress_action_table(&table);
    // All rows are identical empty rows.
    assert_eq!(compressed.unique_rows.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Lookup correctness after compression
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_lookup_specific_cells() {
    let table = glr_table(vec![
        vec![
            Action::Shift(StateId(10)),
            Action::Error,
            Action::Reduce(RuleId(5)),
        ],
        vec![Action::Error, Action::Accept, Action::Shift(StateId(20))],
    ]);
    let compressed = compress_action_table(&table);

    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(10))
    );
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
    assert_eq!(
        decompress_action(&compressed, 0, 2),
        Action::Reduce(RuleId(5))
    );
    assert_eq!(decompress_action(&compressed, 1, 0), Action::Error);
    assert_eq!(decompress_action(&compressed, 1, 1), Action::Accept);
    assert_eq!(
        decompress_action(&compressed, 1, 2),
        Action::Shift(StateId(20))
    );
}

#[test]
fn goto_lookup_specific_cells() {
    let table = vec![
        vec![None, Some(StateId(5)), None, Some(StateId(7))],
        vec![Some(StateId(1)), None, Some(StateId(3)), None],
    ];
    let compressed = compress_goto_table(&table);

    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(5)));
    assert_eq!(decompress_goto(&compressed, 0, 2), None);
    assert_eq!(decompress_goto(&compressed, 0, 3), Some(StateId(7)));
    assert_eq!(decompress_goto(&compressed, 1, 0), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 1, 1), None);
    assert_eq!(decompress_goto(&compressed, 1, 2), Some(StateId(3)));
    assert_eq!(decompress_goto(&compressed, 1, 3), None);
}

#[test]
fn goto_lookup_out_of_bounds_returns_none() {
    let table = vec![vec![Some(StateId(1)), None]];
    let compressed = compress_goto_table(&table);
    // Querying a coordinate with no entry returns None.
    assert_eq!(decompress_goto(&compressed, 99, 99), None);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Determinism of compression output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_compression_is_deterministic() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))],
        vec![Action::Error, Action::Accept],
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))], // dup of row 0
    ]);
    let c1 = compress_action_table(&table);
    let c2 = compress_action_table(&table);

    assert_eq!(c1.unique_rows.len(), c2.unique_rows.len());
    assert_eq!(c1.state_to_row, c2.state_to_row);
    for (r1, r2) in c1.unique_rows.iter().zip(c2.unique_rows.iter()) {
        assert_eq!(r1, r2);
    }
}

#[test]
fn goto_compression_is_deterministic() {
    let table = vec![
        vec![None, Some(StateId(1)), None],
        vec![Some(StateId(2)), None, Some(StateId(3))],
    ];
    let c1 = compress_goto_table(&table);
    let c2 = compress_goto_table(&table);

    assert_eq!(c1.entries.len(), c2.entries.len());
    for (key, val) in &c1.entries {
        assert_eq!(c2.entries.get(key), Some(val));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. TableCompressor (compress.rs) - small table encoding
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn encode_action_shift_roundtrips() {
    let compressor = TableCompressor::new();
    let encoded = compressor
        .encode_action_small(&Action::Shift(StateId(42)))
        .unwrap();
    assert_eq!(encoded, 42);
}

#[test]
fn encode_action_reduce_roundtrips() {
    let compressor = TableCompressor::new();
    let encoded = compressor
        .encode_action_small(&Action::Reduce(RuleId(7)))
        .unwrap();
    // Reduce encoded as 0x8000 | (rule_id + 1)
    assert_eq!(encoded, 0x8000 | 8);
}

#[test]
fn encode_action_accept_value() {
    let compressor = TableCompressor::new();
    let encoded = compressor.encode_action_small(&Action::Accept).unwrap();
    assert_eq!(encoded, 0xFFFF);
}

#[test]
fn encode_action_error_value() {
    let compressor = TableCompressor::new();
    let encoded = compressor.encode_action_small(&Action::Error).unwrap();
    assert_eq!(encoded, 0xFFFE);
}

#[test]
fn encode_action_recover_value() {
    let compressor = TableCompressor::new();
    let encoded = compressor.encode_action_small(&Action::Recover).unwrap();
    assert_eq!(encoded, 0xFFFD);
}

#[test]
fn encode_shift_state_too_large() {
    let compressor = TableCompressor::new();
    assert!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x8000)))
            .is_err()
    );
}

#[test]
fn encode_reduce_rule_too_large() {
    let compressor = TableCompressor::new();
    assert!(
        compressor
            .encode_action_small(&Action::Reduce(RuleId(0x4000)))
            .is_err()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. CompressedParseTable API
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_parse_table_accessors() {
    let cpt = CompressedParseTable::new_for_testing(42, 99);
    assert_eq!(cpt.symbol_count(), 42);
    assert_eq!(cpt.state_count(), 99);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. compress_action_table_small (TableCompressor)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_action_table_small_row_offsets_correct() {
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![],
            vec![Action::Reduce(RuleId(0))],
        ],
        vec![vec![], vec![], vec![]],
    ];
    let symbol_to_index = BTreeMap::new();
    let result = compressor
        .compress_action_table_small(&action_table, &symbol_to_index)
        .unwrap();

    // row_offsets should have state_count + 1 entries
    assert_eq!(result.row_offsets.len(), 3);
    // First state has 2 non-error actions
    assert_eq!(result.row_offsets[0], 0);
    assert_eq!(result.row_offsets[1], 2);
    // Second state has 0 non-error actions
    assert_eq!(result.row_offsets[2], 2);
}

#[test]
fn compress_action_table_small_default_is_error() {
    let compressor = TableCompressor::new();
    let action_table = vec![vec![vec![Action::Reduce(RuleId(0))]; 5]];
    let symbol_to_index = BTreeMap::new();
    let result = compressor
        .compress_action_table_small(&action_table, &symbol_to_index)
        .unwrap();
    // Default action optimization is disabled; default should be Error.
    assert_eq!(result.default_actions[0], Action::Error);
}

#[test]
fn compress_action_table_small_empty_states() {
    let compressor = TableCompressor::new();
    let action_table: Vec<Vec<Vec<Action>>> = vec![vec![]; 3];
    let symbol_to_index = BTreeMap::new();
    let result = compressor
        .compress_action_table_small(&action_table, &symbol_to_index)
        .unwrap();
    assert!(result.data.is_empty());
    assert_eq!(result.row_offsets.len(), 4); // 3 states + 1 sentinel
    assert_eq!(result.default_actions.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. compress_goto_table_small (TableCompressor) — run-length encoding
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_small_run_length_for_long_runs() {
    let compressor = TableCompressor::new();
    // A run of 5 identical StateIds should produce a RunLength entry.
    let goto_table = vec![vec![StateId(7); 5]];
    let result = compressor.compress_goto_table_small(&goto_table).unwrap();
    let has_rl = result
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 7, count: 5 }));
    assert!(has_rl, "Expected RunLength entry for 5 identical values");
}

#[test]
fn goto_small_singles_for_short_runs() {
    let compressor = TableCompressor::new();
    // A run of 2 should produce 2 Single entries, not RunLength.
    let goto_table = vec![vec![StateId(3), StateId(3)]];
    let result = compressor.compress_goto_table_small(&goto_table).unwrap();
    let singles: Vec<_> = result
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::Single(3)))
        .collect();
    assert_eq!(singles.len(), 2);
}

#[test]
fn goto_small_alternating_no_runs() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(1), StateId(2), StateId(1), StateId(2)]];
    let result = compressor.compress_goto_table_small(&goto_table).unwrap();
    // No RunLength entries for alternating values.
    let rl_count = result
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::RunLength { .. }))
        .count();
    assert_eq!(rl_count, 0);
    assert_eq!(result.data.len(), 4);
}

#[test]
fn goto_small_empty_rows() {
    let compressor = TableCompressor::new();
    let goto_table: Vec<Vec<StateId>> = vec![vec![], vec![], vec![]];
    let result = compressor.compress_goto_table_small(&goto_table).unwrap();
    assert!(result.data.is_empty());
    assert_eq!(result.row_offsets.len(), 4); // 3 rows + 1 sentinel
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. CompressedActionEntry construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compressed_action_entry_new() {
    let entry = CompressedActionEntry::new(99, Action::Accept);
    assert_eq!(entry.symbol, 99);
    assert_eq!(entry.action, Action::Accept);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. BitPackedActionTable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn bitpacked_all_errors() {
    let table: Vec<Vec<Action>> = vec![vec![Action::Error; 4]; 3];
    let bp = BitPackedActionTable::from_table(&table);
    for s in 0..3 {
        for sym in 0..4 {
            assert_eq!(bp.decompress(s, sym), Action::Error);
        }
    }
}

#[test]
fn bitpacked_empty_table() {
    let table: Vec<Vec<Action>> = vec![];
    let bp = BitPackedActionTable::from_table(&table);
    // No cells to decompress; just verify construction succeeds.
    let _ = bp;
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Row deduplication: ordering and indices
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_dedup_preserves_first_occurrence_index() {
    let row_a = vec![vec![Action::Shift(StateId(1))]];
    let row_b = vec![vec![Action::Reduce(RuleId(0))]];
    let table = vec![row_a.clone(), row_b.clone(), row_a, row_b];
    let compressed = compress_action_table(&table);
    // row_a first seen at index 0, row_b at index 1
    assert_eq!(compressed.state_to_row[0], 0);
    assert_eq!(compressed.state_to_row[1], 1);
    assert_eq!(compressed.state_to_row[2], 0);
    assert_eq!(compressed.state_to_row[3], 1);
}

#[test]
fn action_dedup_single_row_maps_to_zero() {
    let table = vec![vec![vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.state_to_row.len(), 1);
    assert_eq!(compressed.state_to_row[0], 0);
    assert_eq!(compressed.unique_rows.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Stress: many distinct rows
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_100_distinct_rows_no_dedup() {
    let table: Vec<Vec<Vec<Action>>> = (0u16..100)
        .map(|i| vec![vec![Action::Shift(StateId(i))]])
        .collect();
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 100);
    assert_action_roundtrip(&table);
}

#[test]
fn goto_100_states_sparse() {
    let table: Vec<Vec<Option<StateId>>> = (0u16..100)
        .map(|s| {
            (0u16..20)
                .map(|sym| {
                    if sym == s % 20 {
                        Some(StateId(s))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    assert_goto_roundtrip(&table);
    let compressed = compress_goto_table(&table);
    // Exactly one entry per state.
    assert_eq!(compressed.entries.len(), 100);
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Goto table: repeated compress yields same entries
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_determinism_repeated_calls() {
    let table = vec![
        vec![Some(StateId(1)), None, Some(StateId(2))],
        vec![None, Some(StateId(3)), None],
    ];
    for _ in 0..5 {
        let c = compress_goto_table(&table);
        assert_eq!(c.entries.len(), 3);
        assert_eq!(c.entries.get(&(0, 0)), Some(&StateId(1)));
        assert_eq!(c.entries.get(&(0, 2)), Some(&StateId(2)));
        assert_eq!(c.entries.get(&(1, 1)), Some(&StateId(3)));
    }
}
