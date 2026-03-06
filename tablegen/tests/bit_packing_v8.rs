//! Comprehensive tests for `BitPackedActionTable` and bit-level compression
//! in `adze-tablegen`.
//!
//! 80+ tests covering:
//! - BitPackedActionTable construction & decompress roundtrips
//! - Error mask bit-packing correctness
//! - Shift / Reduce / Accept / Recover / Fork encoding
//! - Row-deduplication in `compress_action_table`
//! - Sparse goto compression via `compress_goto_table`
//! - Boundary conditions (first cell, last cell, 64-bit word boundaries)
//! - Various table dimensions (1×1 … 100×100)
//! - GLR multi-action cells
//! - Mixed action patterns

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a GLR action table (multi-action cells) from a flat table of single
/// actions.  `Action::Error` maps to an empty cell.
fn to_glr(rows: Vec<Vec<Action>>) -> Vec<Vec<Vec<Action>>> {
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

/// Assert roundtrip through `compress_action_table` / `decompress_action`.
fn assert_action_rt(table: &[Vec<Vec<Action>>]) {
    let compressed = compress_action_table(table);
    for (state, row) in table.iter().enumerate() {
        for (sym, cell) in row.iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            let got = decompress_action(&compressed, state, sym);
            assert_eq!(got, expected, "state={state} sym={sym}");
        }
    }
}

/// Assert roundtrip through `compress_goto_table` / `decompress_goto`.
fn assert_goto_rt(table: &[Vec<Option<StateId>>]) {
    let compressed = compress_goto_table(table);
    for (state, row) in table.iter().enumerate() {
        for (sym, &expected) in row.iter().enumerate() {
            let got = decompress_goto(&compressed, state, sym);
            assert_eq!(got, expected, "state={state} sym={sym}");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. BitPackedActionTable — construction basics (tests 1-6)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_from_table_empty_does_not_panic() {
    let table: Vec<Vec<Action>> = vec![];
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
fn bp_v8_from_table_1x1_does_not_panic() {
    let table = vec![vec![Action::Error]];
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
fn bp_v8_from_table_10x10_does_not_panic() {
    let table = vec![vec![Action::Error; 10]; 10];
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
fn bp_v8_from_table_100x100_does_not_panic() {
    let table = vec![vec![Action::Error; 100]; 100];
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
fn bp_v8_from_table_1x100_does_not_panic() {
    let table = vec![vec![Action::Error; 100]];
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
fn bp_v8_from_table_100x1_does_not_panic() {
    let table = vec![vec![Action::Error]; 100];
    let _packed = BitPackedActionTable::from_table(&table);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Single-action decompress roundtrips (tests 7-14)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_roundtrip_error() {
    let table = vec![vec![Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn bp_v8_roundtrip_shift() {
    let table = vec![vec![Action::Shift(StateId(7))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(7)));
}

#[test]
fn bp_v8_roundtrip_reduce() {
    let table = vec![vec![Action::Reduce(RuleId(3))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(3)));
}

#[test]
fn bp_v8_roundtrip_accept() {
    let table = vec![vec![Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Accept);
}

#[test]
fn bp_v8_roundtrip_recover_maps_to_error() {
    let table = vec![vec![Action::Recover]];
    let packed = BitPackedActionTable::from_table(&table);
    // Recover is stored in the error mask → decompresses as Error
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn bp_v8_roundtrip_fork() {
    let inner = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    let table = vec![vec![Action::Fork(inner.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(inner));
}

#[test]
fn bp_v8_roundtrip_shift_zero() {
    let table = vec![vec![Action::Shift(StateId(0))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(0)));
}

#[test]
fn bp_v8_roundtrip_reduce_zero() {
    let table = vec![vec![Action::Reduce(RuleId(0))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(0)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Multi-cell tables — shifts before reduces (tests 15-24)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_two_shifts_one_row() {
    let table = vec![vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(2)));
}

#[test]
fn bp_v8_two_reduces_one_row() {
    let table = vec![vec![Action::Reduce(RuleId(10)), Action::Reduce(RuleId(20))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(10)));
    assert_eq!(packed.decompress(0, 1), Action::Reduce(RuleId(20)));
}

#[test]
fn bp_v8_shift_then_reduce_one_row() {
    let table = vec![vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(5)));
    assert_eq!(packed.decompress(0, 1), Action::Reduce(RuleId(3)));
}

#[test]
fn bp_v8_shift_error_reduce() {
    let table = vec![vec![
        Action::Shift(StateId(4)),
        Action::Error,
        Action::Reduce(RuleId(2)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(4)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Reduce(RuleId(2)));
}

#[test]
fn bp_v8_shift_then_accept() {
    let table = vec![vec![Action::Shift(StateId(1)), Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Accept);
}

#[test]
fn bp_v8_all_errors_3x3() {
    let table = vec![vec![Action::Error; 3]; 3];
    let packed = BitPackedActionTable::from_table(&table);
    for s in 0..3 {
        for sym in 0..3 {
            assert_eq!(packed.decompress(s, sym), Action::Error);
        }
    }
}

#[test]
fn bp_v8_shift_per_row() {
    // Each row has exactly one shift; all cells shift before any reduce.
    let table = vec![
        vec![Action::Shift(StateId(10))],
        vec![Action::Shift(StateId(20))],
        vec![Action::Shift(StateId(30))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(10)));
    assert_eq!(packed.decompress(1, 0), Action::Shift(StateId(20)));
    assert_eq!(packed.decompress(2, 0), Action::Shift(StateId(30)));
}

#[test]
fn bp_v8_two_rows_shifts_only() {
    let table = vec![
        vec![Action::Shift(StateId(0)), Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2)), Action::Shift(StateId(3))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(0)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(1, 0), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(1, 1), Action::Shift(StateId(3)));
}

#[test]
fn bp_v8_errors_then_shift_row() {
    let table = vec![
        vec![Action::Error, Action::Error],
        vec![Action::Error, Action::Shift(StateId(9))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(1, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Shift(StateId(9)));
}

#[test]
fn bp_v8_multiple_shifts_interspersed_errors() {
    let table = vec![vec![
        Action::Shift(StateId(1)),
        Action::Error,
        Action::Shift(StateId(2)),
        Action::Error,
        Action::Shift(StateId(3)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(0, 3), Action::Error);
    assert_eq!(packed.decompress(0, 4), Action::Shift(StateId(3)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Fork actions in bit-packed table (tests 25-32)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_fork_empty_inner() {
    let table = vec![vec![Action::Fork(vec![])]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(vec![]));
}

#[test]
fn bp_v8_fork_single_shift() {
    let table = vec![vec![Action::Fork(vec![Action::Shift(StateId(5))])]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(
        packed.decompress(0, 0),
        Action::Fork(vec![Action::Shift(StateId(5))])
    );
}

#[test]
fn bp_v8_fork_two_reduces() {
    let inner = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    let table = vec![vec![Action::Fork(inner.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(inner));
}

#[test]
fn bp_v8_fork_shift_reduce_accept() {
    let inner = vec![
        Action::Shift(StateId(3)),
        Action::Reduce(RuleId(4)),
        Action::Accept,
    ];
    let table = vec![vec![Action::Fork(inner.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(inner));
}

#[test]
fn bp_v8_fork_beside_shift() {
    let inner = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    let table = vec![vec![Action::Shift(StateId(1)), Action::Fork(inner.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Fork(inner));
}

#[test]
fn bp_v8_fork_beside_error() {
    let inner = vec![Action::Shift(StateId(2)), Action::Shift(StateId(3))];
    let table = vec![vec![Action::Error, Action::Fork(inner.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Fork(inner));
}

#[test]
fn bp_v8_two_forks_different_cells() {
    let f1 = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let f2 = vec![Action::Reduce(RuleId(2)), Action::Accept];
    let table = vec![vec![Action::Fork(f1.clone()), Action::Fork(f2.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(f1));
    assert_eq!(packed.decompress(0, 1), Action::Fork(f2));
}

#[test]
fn bp_v8_fork_in_second_row() {
    let inner = vec![Action::Shift(StateId(7)), Action::Reduce(RuleId(8))];
    let table = vec![
        vec![Action::Error, Action::Error],
        vec![Action::Error, Action::Fork(inner.clone())],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Fork(inner));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Boundary conditions — 64-bit word edges (tests 33-40)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_exactly_64_error_cells() {
    let table = vec![vec![Action::Error; 64]];
    let packed = BitPackedActionTable::from_table(&table);
    for sym in 0..64 {
        assert_eq!(packed.decompress(0, sym), Action::Error);
    }
}

#[test]
fn bp_v8_65th_cell_shift() {
    // 64 errors then 1 shift → spans two u64 words
    let mut row = vec![Action::Error; 64];
    row.push(Action::Shift(StateId(99)));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 63), Action::Error);
    assert_eq!(packed.decompress(0, 64), Action::Shift(StateId(99)));
}

#[test]
fn bp_v8_cell_index_63_is_shift() {
    // Last bit of first u64 word
    let mut row = vec![Action::Error; 63];
    row.push(Action::Shift(StateId(42)));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 62), Action::Error);
    assert_eq!(packed.decompress(0, 63), Action::Shift(StateId(42)));
}

#[test]
fn bp_v8_128_cells_all_errors() {
    let table = vec![vec![Action::Error; 128]];
    let packed = BitPackedActionTable::from_table(&table);
    for sym in 0..128 {
        assert_eq!(packed.decompress(0, sym), Action::Error);
    }
}

#[test]
fn bp_v8_first_cell_shift_rest_errors() {
    let mut row = vec![Action::Error; 65];
    row[0] = Action::Shift(StateId(1));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    for sym in 1..65 {
        assert_eq!(packed.decompress(0, sym), Action::Error);
    }
}

#[test]
fn bp_v8_last_cell_only_non_error() {
    let mut row = vec![Action::Error; 65];
    row[64] = Action::Shift(StateId(77));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    for sym in 0..64 {
        assert_eq!(packed.decompress(0, sym), Action::Error);
    }
    assert_eq!(packed.decompress(0, 64), Action::Shift(StateId(77)));
}

#[test]
fn bp_v8_shift_at_word_boundary_0_and_64() {
    let mut row = vec![Action::Error; 128];
    row[0] = Action::Shift(StateId(10));
    row[64] = Action::Shift(StateId(20));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(10)));
    assert_eq!(packed.decompress(0, 64), Action::Shift(StateId(20)));
}

#[test]
fn bp_v8_fork_at_word_boundary() {
    let mut row = vec![Action::Error; 65];
    let inner = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    row[64] = Action::Fork(inner.clone());
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 64), Action::Fork(inner));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. compress_action_table — row deduplication (tests 41-50)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_compress_action_does_not_panic() {
    let table = to_glr(vec![vec![Action::Error]]);
    let _compressed = compress_action_table(&table);
}

#[test]
fn bp_v8_compress_action_single_row() {
    let table = to_glr(vec![vec![Action::Shift(StateId(1))]]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 1);
}

#[test]
fn bp_v8_compress_action_duplicate_rows() {
    let row = vec![Action::Shift(StateId(1)), Action::Error];
    let table = to_glr(vec![row.clone(), row]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row, vec![0, 0]);
}

#[test]
fn bp_v8_compress_action_two_distinct_rows() {
    let table = to_glr(vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(0))],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
}

#[test]
fn bp_v8_compress_action_three_rows_two_unique() {
    let r1 = vec![Action::Error, Action::Shift(StateId(1))];
    let r2 = vec![Action::Reduce(RuleId(0)), Action::Error];
    let table = to_glr(vec![r1.clone(), r2, r1]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[2]);
}

#[test]
fn bp_v8_compress_action_all_error_rows_dedup() {
    let table = to_glr(vec![
        vec![Action::Error; 5],
        vec![Action::Error; 5],
        vec![Action::Error; 5],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn bp_v8_decompress_action_shift() {
    let table = to_glr(vec![vec![Action::Shift(StateId(42))]]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(42))
    );
}

#[test]
fn bp_v8_decompress_action_empty_cell_is_error() {
    let table = vec![vec![vec![]]]; // empty cell
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
}

#[test]
fn bp_v8_decompress_action_multi_action_returns_first() {
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
fn bp_v8_action_roundtrip_mixed_table() {
    let table = to_glr(vec![
        vec![Action::Error, Action::Shift(StateId(1)), Action::Error],
        vec![Action::Reduce(RuleId(0)), Action::Error, Action::Error],
        vec![Action::Error, Action::Error, Action::Accept],
    ]);
    assert_action_rt(&table);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. compress_goto_table — sparse representation (tests 51-60)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_compress_goto_does_not_panic() {
    let table: Vec<Vec<Option<StateId>>> = vec![vec![None]];
    let _compressed = compress_goto_table(&table);
}

#[test]
fn bp_v8_compress_goto_empty_table() {
    let table: Vec<Vec<Option<StateId>>> = vec![vec![None; 3]; 3];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn bp_v8_compress_goto_single_entry() {
    let table = vec![vec![Some(StateId(5)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(5)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
}

#[test]
fn bp_v8_compress_goto_diagonal() {
    let table = vec![
        vec![Some(StateId(1)), None, None],
        vec![None, Some(StateId(2)), None],
        vec![None, None, Some(StateId(3))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(2)));
    assert_eq!(decompress_goto(&compressed, 2, 2), Some(StateId(3)));
}

#[test]
fn bp_v8_compress_goto_dense_row() {
    let table = vec![vec![Some(StateId(1)), Some(StateId(2)), Some(StateId(3))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
    for sym in 0..3 {
        assert_eq!(
            decompress_goto(&compressed, 0, sym),
            Some(StateId((sym as u16) + 1))
        );
    }
}

#[test]
fn bp_v8_goto_roundtrip_mixed() {
    let table = vec![
        vec![None, Some(StateId(10)), None],
        vec![Some(StateId(20)), None, Some(StateId(30))],
    ];
    assert_goto_rt(&table);
}

#[test]
fn bp_v8_goto_roundtrip_all_some() {
    let table = vec![
        vec![Some(StateId(0)), Some(StateId(1))],
        vec![Some(StateId(2)), Some(StateId(3))],
    ];
    assert_goto_rt(&table);
}

#[test]
fn bp_v8_goto_roundtrip_all_none() {
    let table = vec![vec![None; 4]; 4];
    assert_goto_rt(&table);
}

#[test]
fn bp_v8_goto_state_id_zero() {
    let table = vec![vec![Some(StateId(0))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(0)));
}

#[test]
fn bp_v8_goto_large_state_id() {
    let table = vec![vec![Some(StateId(u16::MAX))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(u16::MAX)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Various table dimensions (tests 61-68)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_dim_1x1_action_roundtrip() {
    let table = to_glr(vec![vec![Action::Shift(StateId(0))]]);
    assert_action_rt(&table);
}

#[test]
fn bp_v8_dim_5x5_all_error() {
    let table = to_glr(vec![vec![Action::Error; 5]; 5]);
    assert_action_rt(&table);
}

#[test]
fn bp_v8_dim_10x10_shift_diagonal() {
    let mut rows = vec![vec![Action::Error; 10]; 10];
    // Only cells on the diagonal are shifts; shifts come before reduces
    // since there are NO reduces, the heuristic works fine.
    for (i, row) in rows.iter_mut().enumerate() {
        row[i] = Action::Shift(StateId(i as u16));
    }
    let table = to_glr(rows);
    assert_action_rt(&table);
}

#[test]
fn bp_v8_dim_50x1_shifts() {
    let table = to_glr((0..50).map(|i| vec![Action::Shift(StateId(i))]).collect());
    assert_action_rt(&table);
}

#[test]
fn bp_v8_dim_1x50_shifts() {
    let table = to_glr(vec![(0..50).map(|i| Action::Shift(StateId(i))).collect()]);
    assert_action_rt(&table);
}

#[test]
fn bp_v8_goto_dim_1x1() {
    let table = vec![vec![Some(StateId(5))]];
    assert_goto_rt(&table);
}

#[test]
fn bp_v8_goto_dim_5x5_sparse() {
    let mut table = vec![vec![None; 5]; 5];
    table[0][2] = Some(StateId(7));
    table[3][4] = Some(StateId(8));
    assert_goto_rt(&table);
}

#[test]
fn bp_v8_goto_dim_50x50_diagonal() {
    let mut table = vec![vec![None; 50]; 50];
    for (i, row) in table.iter_mut().enumerate() {
        row[i] = Some(StateId(i as u16));
    }
    assert_goto_rt(&table);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. GLR multi-action cells in compressed table (tests 69-76)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_glr_cell_two_actions() {
    let table = vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]];
    let compressed = compress_action_table(&table);
    // decompress_action returns first action
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
}

#[test]
fn bp_v8_glr_cell_three_actions() {
    let table = vec![vec![vec![
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
        Action::Reduce(RuleId(3)),
    ]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Reduce(RuleId(1))
    );
}

#[test]
fn bp_v8_glr_mixed_cells() {
    let table = vec![vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        vec![Action::Reduce(RuleId(2))],
        vec![],
    ]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
    assert_eq!(
        decompress_action(&compressed, 0, 1),
        Action::Reduce(RuleId(2))
    );
    assert_eq!(decompress_action(&compressed, 0, 2), Action::Error);
}

#[test]
fn bp_v8_glr_accept_in_multi_cell() {
    let table = vec![vec![vec![Action::Accept, Action::Reduce(RuleId(0))]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
}

#[test]
fn bp_v8_glr_row_dedup_multi_action() {
    let row = vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        vec![],
    ];
    let table = vec![row.clone(), row];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn bp_v8_glr_empty_table() {
    let table: Vec<Vec<Vec<Action>>> = vec![vec![vec![]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
}

#[test]
fn bp_v8_glr_single_action_accept() {
    let table = vec![vec![vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
}

#[test]
fn bp_v8_glr_two_rows_distinct_multi() {
    let table = vec![
        vec![
            vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
            vec![Action::Error],
        ],
        vec![
            vec![Action::Error],
            vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))],
        ],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. Edge cases & misc (tests 77-85)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_max_state_id_shift() {
    let table = vec![vec![Action::Shift(StateId(u16::MAX - 1))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(
        packed.decompress(0, 0),
        Action::Shift(StateId(u16::MAX - 1))
    );
}

#[test]
fn bp_v8_max_rule_id_reduce() {
    let table = vec![vec![Action::Reduce(RuleId(u16::MAX - 1))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(
        packed.decompress(0, 0),
        Action::Reduce(RuleId(u16::MAX - 1))
    );
}

#[test]
fn bp_v8_single_cell_overwrite_semantics() {
    // from_table is called once; later table replaces earlier
    let table_a = vec![vec![Action::Shift(StateId(1))]];
    let table_b = vec![vec![Action::Reduce(RuleId(2))]];
    let packed_a = BitPackedActionTable::from_table(&table_a);
    let packed_b = BitPackedActionTable::from_table(&table_b);
    assert_eq!(packed_a.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed_b.decompress(0, 0), Action::Reduce(RuleId(2)));
}

#[test]
fn bp_v8_error_mask_consistency() {
    // A table with interspersed errors: verify every error cell decompresses.
    let table = vec![vec![
        Action::Error,
        Action::Shift(StateId(0)),
        Action::Error,
        Action::Shift(StateId(1)),
        Action::Error,
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Error);
    assert_eq!(packed.decompress(0, 4), Action::Error);
}

#[test]
fn bp_v8_large_shift_id_values() {
    let table = vec![vec![
        Action::Shift(StateId(1000)),
        Action::Shift(StateId(2000)),
        Action::Shift(StateId(30000)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1000)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(2000)));
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(30000)));
}

#[test]
fn bp_v8_goto_missing_key_returns_none() {
    let table = vec![vec![Some(StateId(1)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
}

#[test]
fn bp_v8_action_dedup_preserves_state_map() {
    let r1 = vec![Action::Shift(StateId(0))];
    let r2 = vec![Action::Shift(StateId(1))];
    let table = to_glr(vec![r1.clone(), r2, r1]);
    let compressed = compress_action_table(&table);
    // States 0 and 2 should map to the same row
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[2]);
    assert_ne!(compressed.state_to_row[0], compressed.state_to_row[1]);
}

#[test]
fn bp_v8_action_dedup_4_identical_rows() {
    let row = vec![Action::Error, Action::Shift(StateId(5))];
    let table = to_glr(vec![row.clone(), row.clone(), row.clone(), row]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 4);
}

#[test]
fn bp_v8_compress_goto_only_last_row_has_entries() {
    let mut table = vec![vec![None; 3]; 3];
    table[2][0] = Some(StateId(10));
    table[2][1] = Some(StateId(11));
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 2);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 2, 0), Some(StateId(10)));
    assert_eq!(decompress_goto(&compressed, 2, 1), Some(StateId(11)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. Additional coverage (tests 86-90)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_v8_bitpack_accept_beside_errors() {
    let table = vec![vec![Action::Error, Action::Error, Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Accept);
}

#[test]
fn bp_v8_bitpack_shift_id_zero_beside_error() {
    let table = vec![vec![Action::Shift(StateId(0)), Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(0)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
}

#[test]
fn bp_v8_goto_roundtrip_10x10_sparse() {
    let mut table = vec![vec![None; 10]; 10];
    table[0][5] = Some(StateId(1));
    table[3][7] = Some(StateId(2));
    table[9][0] = Some(StateId(3));
    table[9][9] = Some(StateId(4));
    assert_goto_rt(&table);
}

#[test]
fn bp_v8_action_roundtrip_5x5_shifts() {
    let table = to_glr(vec![
        vec![
            Action::Shift(StateId(0)),
            Action::Shift(StateId(1)),
            Action::Shift(StateId(2)),
            Action::Shift(StateId(3)),
            Action::Shift(StateId(4)),
        ];
        5
    ]);
    assert_action_rt(&table);
}

#[test]
fn bp_v8_action_roundtrip_reduces_only() {
    let table = to_glr(vec![vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ]]);
    assert_action_rt(&table);
}
