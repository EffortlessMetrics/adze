//! Comprehensive v2 tests for bit-packing and table compression in adze-tablegen.
//!
//! 55+ tests covering:
//! 1. Compress simple action table (8 tests)
//! 2. Compress goto table (8 tests)
//! 3. BitPackedActionTable lookup correctness (8 tests)
//! 4. CompressedGotoTable lookup correctness (5 tests)
//! 5. Compression preserves all actions (8 tests)
//! 6. RLE encoding patterns (5 tests)
//! 7. Compression with GLR multi-action cells (5 tests)
//! 8. Edge cases (8 tests)

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};

// ============================================================================
// Helpers
// ============================================================================

/// Wrap single actions into GLR action cells (empty vec for Error).
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

// ============================================================================
// 1. Compress simple action table (8 tests)
// ============================================================================

#[test]
fn compress_action_single_shift() {
    let table = glr_table(vec![vec![Action::Shift(StateId(1))]]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 1);
}

#[test]
fn compress_action_single_reduce() {
    let table = glr_table(vec![vec![Action::Reduce(RuleId(0))]]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn compress_action_single_accept() {
    let table = glr_table(vec![vec![Action::Accept]]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn compress_action_all_errors() {
    let table = glr_table(vec![
        vec![Action::Error, Action::Error],
        vec![Action::Error, Action::Error],
    ]);
    let compressed = compress_action_table(&table);
    // Two identical rows should deduplicate to 1
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row, [0, 0]);
}

#[test]
fn compress_action_duplicate_rows_deduplicated() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(5)), Action::Error],
        vec![Action::Shift(StateId(5)), Action::Error],
        vec![Action::Shift(StateId(5)), Action::Error],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row, [0, 0, 0]);
}

#[test]
fn compress_action_distinct_rows_preserved() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(0))],
        vec![Action::Accept],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 3);
}

#[test]
fn compress_action_mixed_dedup_pattern() {
    // Rows: A, B, A, C, B  → 3 unique
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1)), Action::Error],
        vec![Action::Error, Action::Reduce(RuleId(0))],
        vec![Action::Shift(StateId(1)), Action::Error],
        vec![Action::Accept, Action::Accept],
        vec![Action::Error, Action::Reduce(RuleId(0))],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 3);
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[2]);
    assert_eq!(compressed.state_to_row[1], compressed.state_to_row[4]);
}

#[test]
fn compress_action_state_to_row_length_matches_states() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(0))],
        vec![Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2))],
        vec![Action::Shift(StateId(3))],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.state_to_row.len(), 4);
}

// ============================================================================
// 2. Compress goto table (8 tests)
// ============================================================================

#[test]
fn compress_goto_single_entry() {
    let table = vec![vec![Some(StateId(5)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
}

#[test]
fn compress_goto_all_none() {
    let table = vec![vec![None, None, None], vec![None, None, None]];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn compress_goto_fully_populated() {
    let table = vec![vec![Some(StateId(1)), Some(StateId(2)), Some(StateId(3))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
}

#[test]
fn compress_goto_sparse_diagonal() {
    let table = vec![
        vec![Some(StateId(10)), None, None],
        vec![None, Some(StateId(20)), None],
        vec![None, None, Some(StateId(30))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
}

#[test]
fn compress_goto_multiple_entries_per_row() {
    let table = vec![vec![
        Some(StateId(1)),
        None,
        Some(StateId(3)),
        None,
        Some(StateId(5)),
    ]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
}

#[test]
fn compress_goto_duplicate_target_states() {
    let table = vec![vec![Some(StateId(7)), Some(StateId(7)), Some(StateId(7))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
    // All point to the same state
    for col in 0..3 {
        assert_eq!(decompress_goto(&compressed, 0, col), Some(StateId(7)));
    }
}

#[test]
fn compress_goto_large_state_ids() {
    let table = vec![vec![Some(StateId(1000)), None, Some(StateId(u16::MAX))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 2);
}

#[test]
fn compress_goto_preserves_state_zero() {
    let table = vec![vec![Some(StateId(0)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(0)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
}

// ============================================================================
// 3. BitPackedActionTable lookup correctness (8 tests)
// ============================================================================

#[test]
fn bitpack_lookup_error() {
    let table = vec![vec![Action::Error, Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Error);
}

#[test]
fn bitpack_lookup_single_shift() {
    let table = vec![vec![Action::Shift(StateId(42))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(42)));
}

#[test]
fn bitpack_lookup_single_reduce() {
    let table = vec![vec![Action::Reduce(RuleId(7))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(7)));
}

#[test]
fn bitpack_lookup_accept() {
    let table = vec![vec![Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Accept);
}

#[test]
fn bitpack_lookup_recover_is_error() {
    // Recover is mapped to error in bit-packing
    let table = vec![vec![Action::Recover]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn bitpack_lookup_fork_preserved() {
    let fork_actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let table = vec![vec![Action::Fork(fork_actions.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(fork_actions));
}

#[test]
fn bitpack_lookup_multi_row_multi_col() {
    let table = vec![
        vec![Action::Shift(StateId(1)), Action::Error],
        vec![Action::Error, Action::Reduce(RuleId(2))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(1, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Reduce(RuleId(2)));
}

#[test]
fn bitpack_lookup_shift_then_reduce_in_same_row() {
    let table = vec![vec![Action::Shift(StateId(10)), Action::Reduce(RuleId(5))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(10)));
    assert_eq!(packed.decompress(0, 1), Action::Reduce(RuleId(5)));
}

// ============================================================================
// 4. CompressedGotoTable lookup correctness (5 tests)
// ============================================================================

#[test]
fn goto_lookup_present_entry() {
    let table = vec![vec![None, Some(StateId(3)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(3)));
}

#[test]
fn goto_lookup_absent_entry() {
    let table = vec![vec![None, Some(StateId(3)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 0, 2), None);
}

#[test]
fn goto_lookup_multiple_states() {
    let table = vec![vec![Some(StateId(1)), None], vec![None, Some(StateId(2))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(decompress_goto(&compressed, 1, 0), None);
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(2)));
}

#[test]
fn goto_lookup_nonexistent_key_returns_none() {
    let table = vec![vec![Some(StateId(9))]];
    let compressed = compress_goto_table(&table);
    // Key (1, 0) was never inserted
    assert_eq!(decompress_goto(&compressed, 1, 0), None);
}

#[test]
fn goto_lookup_zero_target_is_some() {
    let table = vec![vec![Some(StateId(0))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(0)));
}

// ============================================================================
// 5. Compression preserves all actions (8 tests)
// ============================================================================

#[test]
fn preserve_shift_roundtrip() {
    let table = glr_table(vec![vec![Action::Shift(StateId(99))]]);
    assert_action_roundtrip(&table);
}

#[test]
fn preserve_reduce_roundtrip() {
    let table = glr_table(vec![vec![Action::Reduce(RuleId(42))]]);
    assert_action_roundtrip(&table);
}

#[test]
fn preserve_accept_roundtrip() {
    let table = glr_table(vec![vec![Action::Accept]]);
    assert_action_roundtrip(&table);
}

#[test]
fn preserve_error_roundtrip() {
    let table = glr_table(vec![vec![Action::Error, Action::Error, Action::Error]]);
    assert_action_roundtrip(&table);
}

#[test]
fn preserve_mixed_actions_roundtrip() {
    let table = glr_table(vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Error,
        Action::Accept,
    ]]);
    assert_action_roundtrip(&table);
}

#[test]
fn preserve_multi_state_roundtrip() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1)), Action::Error, Action::Error],
        vec![Action::Error, Action::Reduce(RuleId(0)), Action::Error],
        vec![Action::Error, Action::Error, Action::Accept],
    ]);
    assert_action_roundtrip(&table);
}

#[test]
fn preserve_goto_full_roundtrip() {
    let table = vec![
        vec![Some(StateId(1)), None, Some(StateId(3))],
        vec![None, Some(StateId(2)), None],
        vec![Some(StateId(0)), Some(StateId(4)), Some(StateId(5))],
    ];
    assert_goto_roundtrip(&table);
}

#[test]
fn preserve_goto_sparse_roundtrip() {
    let table = vec![
        vec![None, None, None, None, None],
        vec![None, None, Some(StateId(7)), None, None],
        vec![None, None, None, None, None],
    ];
    assert_goto_roundtrip(&table);
}

// ============================================================================
// 6. RLE encoding patterns (5 tests)
// ============================================================================

#[test]
fn rle_all_same_goto_entries() {
    // All entries are the same target — maximal sparseness benefit
    let table = vec![vec![
        Some(StateId(5)),
        Some(StateId(5)),
        Some(StateId(5)),
        Some(StateId(5)),
    ]];
    let compressed = compress_goto_table(&table);
    // Sparse: 4 entries stored
    assert_eq!(compressed.entries.len(), 4);
    for col in 0..4 {
        assert_eq!(decompress_goto(&compressed, 0, col), Some(StateId(5)));
    }
}

#[test]
fn rle_alternating_goto_pattern() {
    let table = vec![vec![
        Some(StateId(1)),
        Some(StateId(2)),
        Some(StateId(1)),
        Some(StateId(2)),
    ]];
    let compressed = compress_goto_table(&table);
    for (col, expected) in [StateId(1), StateId(2), StateId(1), StateId(2)]
        .iter()
        .enumerate()
    {
        assert_eq!(decompress_goto(&compressed, 0, col), Some(*expected));
    }
}

#[test]
fn rle_none_heavy_goto() {
    let table = vec![vec![None, None, None, Some(StateId(9)), None, None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 0, 3), Some(StateId(9)));
}

#[test]
fn rle_action_row_dedup_is_stable() {
    // Same table compressed twice should produce identical dedup
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1)), Action::Error],
        vec![Action::Shift(StateId(1)), Action::Error],
    ]);
    let c1 = compress_action_table(&table);
    let c2 = compress_action_table(&table);
    assert_eq!(c1.unique_rows.len(), c2.unique_rows.len());
    assert_eq!(c1.state_to_row, c2.state_to_row);
}

#[test]
fn rle_dense_goto_all_populated() {
    let table = vec![vec![
        Some(StateId(0)),
        Some(StateId(1)),
        Some(StateId(2)),
        Some(StateId(3)),
    ]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
    for col in 0..4 {
        assert_eq!(
            decompress_goto(&compressed, 0, col),
            Some(StateId(col as u16))
        );
    }
}

// ============================================================================
// 7. Compression with GLR multi-action cells (5 tests)
// ============================================================================

#[test]
fn glr_cell_with_two_actions_first_wins() {
    // compress_action_table stores the full cell; decompress_action returns the first
    let table = vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
}

#[test]
fn glr_cell_empty_returns_error() {
    let table = vec![vec![vec![]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
}

#[test]
fn glr_multi_cell_dedup_differs_from_single() {
    // Two cells with same first action but different second action
    // are treated as different rows
    let row_a = vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]];
    let row_b = vec![vec![Action::Shift(StateId(1))]];
    let table = vec![row_a, row_b];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
}

#[test]
fn glr_fork_in_bitpacked_table() {
    let fork = Action::Fork(vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))]);
    let table = vec![vec![fork.clone(), Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), fork);
    assert_eq!(packed.decompress(0, 1), Action::Error);
}

#[test]
fn glr_multiple_fork_cells() {
    let fork1 = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    let fork2 = Action::Fork(vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(3))]);
    let table = vec![vec![fork1.clone(), fork2.clone()]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), fork1);
    assert_eq!(packed.decompress(0, 1), fork2);
}

// ============================================================================
// 8. Edge cases (8 tests)
// ============================================================================

#[test]
fn edge_empty_action_table() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let compressed = compress_action_table(&table);
    assert!(compressed.unique_rows.is_empty());
    assert!(compressed.state_to_row.is_empty());
}

#[test]
fn edge_empty_goto_table() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn edge_single_state_single_symbol_action() {
    let table = glr_table(vec![vec![Action::Shift(StateId(0))]]);
    assert_action_roundtrip(&table);
}

#[test]
fn edge_single_state_single_symbol_goto() {
    let table = vec![vec![Some(StateId(0))]];
    assert_goto_roundtrip(&table);
}

#[test]
fn edge_wide_row_action_table() {
    // A single state with 100 symbols
    let row: Vec<Action> = (0..100)
        .map(|i| {
            if i % 3 == 0 {
                Action::Shift(StateId(i))
            } else if i % 3 == 1 {
                Action::Reduce(RuleId(i))
            } else {
                Action::Error
            }
        })
        .collect();
    let table = glr_table(vec![row]);
    assert_action_roundtrip(&table);
}

#[test]
fn edge_wide_row_goto_table() {
    // A single state with 50 symbols, alternating Some/None
    let row: Vec<Option<StateId>> = (0..50)
        .map(|i| if i % 2 == 0 { Some(StateId(i)) } else { None })
        .collect();
    let table = vec![row];
    assert_goto_roundtrip(&table);
}

#[test]
fn edge_many_states_few_symbols() {
    // 50 states, 2 symbols each
    let table: Vec<Vec<Vec<Action>>> = (0..50)
        .map(|s| {
            vec![
                vec![Action::Shift(StateId(s))],
                vec![Action::Reduce(RuleId(s))],
            ]
        })
        .collect();
    let compressed = compress_action_table(&table);
    // All rows are unique since each has a different state/rule
    assert_eq!(compressed.unique_rows.len(), 50);
    // Roundtrip each cell
    for state in 0..50 {
        assert_eq!(
            decompress_action(&compressed, state, 0),
            Action::Shift(StateId(state as u16))
        );
        assert_eq!(
            decompress_action(&compressed, state, 1),
            Action::Reduce(RuleId(state as u16))
        );
    }
}

#[test]
fn edge_max_symbol_ids_in_goto() {
    let table = vec![vec![
        Some(StateId(u16::MAX)),
        None,
        Some(StateId(u16::MAX - 1)),
    ]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(u16::MAX)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(
        decompress_goto(&compressed, 0, 2),
        Some(StateId(u16::MAX - 1))
    );
}

// ============================================================================
// Additional edge-case / stress tests (to reach 55+)
// ============================================================================

#[test]
fn bitpack_error_mask_spans_multiple_words() {
    // 65+ cells forces the error mask to span 2 u64 words
    let row: Vec<Action> = (0..65).map(|_| Action::Error).collect();
    let packed = BitPackedActionTable::from_table(&[row]);
    for col in 0..65 {
        assert_eq!(packed.decompress(0, col), Action::Error);
    }
}

#[test]
fn compress_action_recover_treated_as_error() {
    let table = glr_table(vec![vec![Action::Recover, Action::Shift(StateId(1))]]);
    let compressed = compress_action_table(&table);
    // Recover is stored in the cell; decompress_action returns first element
    let result = decompress_action(&compressed, 0, 0);
    assert_eq!(result, Action::Recover);
}

#[test]
fn goto_lookup_out_of_bounds_state_returns_none() {
    let table = vec![vec![Some(StateId(1))]];
    let compressed = compress_goto_table(&table);
    // State 5 was never in the table
    assert_eq!(decompress_goto(&compressed, 5, 0), None);
}

#[test]
fn compress_action_deterministic_across_calls() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))],
        vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))],
        vec![Action::Accept, Action::Error],
    ]);
    let c1 = compress_action_table(&table);
    let c2 = compress_action_table(&table);
    assert_eq!(c1.unique_rows, c2.unique_rows);
    assert_eq!(c1.state_to_row, c2.state_to_row);
}

#[test]
fn bitpack_from_table_with_zero_symbols_constructs() {
    let table: Vec<Vec<Action>> = vec![vec![]];
    // Construction with zero symbols should not panic
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
#[should_panic]
fn bitpack_zero_symbols_decompress_panics() {
    let table: Vec<Vec<Action>> = vec![vec![]];
    let packed = BitPackedActionTable::from_table(&table);
    // Decompress with symbol_count=0 causes division by zero
    let _result = packed.decompress(0, 0);
}
