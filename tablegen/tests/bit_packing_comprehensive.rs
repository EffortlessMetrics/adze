#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for bit-packing and encoding in adze-tablegen.
//!
//! Covers: BitPackedActionTable encoding / decoding, action roundtrips,
//! boundary values, all action type encodings, packed table integrity,
//! row deduplication, goto compression, and compression ratios.

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};

// ── BitPackedActionTable: encoding ──────────────────────────────────────────

#[test]
fn bitpack_error_only_table() {
    let table = vec![
        vec![Action::Error, Action::Error],
        vec![Action::Error, Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    for state in 0..2 {
        for sym in 0..2 {
            assert_eq!(packed.decompress(state, sym), Action::Error);
        }
    }
}

#[test]
fn bitpack_shift_only_table() {
    let table = vec![vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(2)));
}

#[test]
fn bitpack_reduce_only_table() {
    let table = vec![vec![Action::Reduce(RuleId(10)), Action::Reduce(RuleId(20))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(10)));
    assert_eq!(packed.decompress(0, 1), Action::Reduce(RuleId(20)));
}

#[test]
fn bitpack_accept_encoding() {
    // Accept is stored as u32::MAX in reduce_data
    let table = vec![vec![Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Accept);
}

#[test]
fn bitpack_recover_treated_as_error() {
    let table = vec![vec![Action::Recover]];
    let packed = BitPackedActionTable::from_table(&table);
    // Recover is mapped into the error mask, decompresses as Error
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn bitpack_fork_encoding() {
    let fork_actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    let table = vec![vec![Action::Fork(fork_actions.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(fork_actions));
}

// ── roundtrips: shifts before reduces ───────────────────────────────────────

#[test]
fn bitpack_roundtrip_shift_then_reduce() {
    let table = vec![vec![
        Action::Shift(StateId(5)),
        Action::Error,
        Action::Reduce(RuleId(3)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(5)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Reduce(RuleId(3)));
}

#[test]
fn bitpack_roundtrip_multirow_shifts_then_reduces() {
    // All shifts in earlier rows, all reduces in later rows
    let table = vec![
        vec![Action::Shift(StateId(0)), Action::Shift(StateId(100))],
        vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(50))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(0)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(100)));
    assert_eq!(packed.decompress(1, 0), Action::Reduce(RuleId(0)));
    assert_eq!(packed.decompress(1, 1), Action::Reduce(RuleId(50)));
}

#[test]
fn bitpack_roundtrip_shift_error_accept() {
    let table = vec![vec![
        Action::Shift(StateId(7)),
        Action::Error,
        Action::Accept,
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(7)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Accept);
}

// ── boundary values ─────────────────────────────────────────────────────────

#[test]
fn bitpack_state_id_zero() {
    let table = vec![vec![Action::Shift(StateId(0))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(0)));
}

#[test]
fn bitpack_state_id_max_u16() {
    let table = vec![vec![Action::Shift(StateId(u16::MAX))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(u16::MAX)));
}

#[test]
fn bitpack_rule_id_zero() {
    let table = vec![vec![Action::Reduce(RuleId(0))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(0)));
}

#[test]
fn bitpack_rule_id_max_u16() {
    // u16::MAX (65535) as u32 is 65535, not u32::MAX, so no Accept confusion
    let table = vec![vec![Action::Reduce(RuleId(u16::MAX))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(u16::MAX)));
}

// ── error mask spanning multiple u64 words ──────────────────────────────────

#[test]
fn bitpack_error_mask_spans_two_words() {
    // 65 cells → 2 u64 words for the error mask
    let mut row = vec![Action::Error; 65];
    row[64] = Action::Shift(StateId(42));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);

    for sym in 0..64 {
        assert_eq!(
            packed.decompress(0, sym),
            Action::Error,
            "cell {sym} should be Error"
        );
    }
    assert_eq!(packed.decompress(0, 64), Action::Shift(StateId(42)));
}

#[test]
fn bitpack_error_mask_exact_word_boundary() {
    // Exactly 64 cells (one full u64 word)
    let table = vec![vec![Action::Error; 64]];
    let packed = BitPackedActionTable::from_table(&table);
    for sym in 0..64 {
        assert_eq!(packed.decompress(0, sym), Action::Error);
    }
}

// ── all action type encodings ───────────────────────────────────────────────

#[test]
fn bitpack_all_action_types_in_one_table() {
    let fork = vec![Action::Shift(StateId(99)), Action::Reduce(RuleId(99))];
    let table = vec![
        vec![
            Action::Shift(StateId(1)),
            Action::Shift(StateId(2)),
            Action::Error,
            Action::Fork(fork.clone()),
        ],
        vec![
            Action::Reduce(RuleId(10)),
            Action::Accept,
            Action::Recover,
            Action::Error,
        ],
    ];
    let packed = BitPackedActionTable::from_table(&table);

    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(0, 2), Action::Error);
    assert_eq!(packed.decompress(0, 3), Action::Fork(fork));
    assert_eq!(packed.decompress(1, 0), Action::Reduce(RuleId(10)));
    assert_eq!(packed.decompress(1, 1), Action::Accept);
    assert_eq!(packed.decompress(1, 2), Action::Error); // Recover → Error
    assert_eq!(packed.decompress(1, 3), Action::Error);
}

// ── empty / single-cell tables ──────────────────────────────────────────────

#[test]
fn bitpack_empty_table() {
    let table: Vec<Vec<Action>> = vec![];
    let _packed = BitPackedActionTable::from_table(&table);
    // Construction must not panic
}

#[test]
fn bitpack_single_cell_shift() {
    let table = vec![vec![Action::Shift(StateId(42))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(42)));
}

#[test]
fn bitpack_single_cell_error() {
    let table = vec![vec![Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

// ── multiple forks ──────────────────────────────────────────────────────────

#[test]
fn bitpack_multiple_fork_cells() {
    let fork_a = vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))];
    let fork_b = vec![Action::Reduce(RuleId(3)), Action::Reduce(RuleId(4))];
    let table = vec![vec![
        Action::Fork(fork_a.clone()),
        Action::Fork(fork_b.clone()),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(fork_a));
    assert_eq!(packed.decompress(0, 1), Action::Fork(fork_b));
}

// ── row deduplication (CompressedActionTable) ───────────────────────────────

#[test]
fn row_dedup_identical_rows() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![]],
        vec![vec![Action::Shift(StateId(1))], vec![]],
        vec![vec![Action::Shift(StateId(1))], vec![]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row, vec![0, 0, 0]);
}

#[test]
fn row_dedup_all_unique() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![]],
        vec![vec![Action::Shift(StateId(2))], vec![]],
        vec![vec![Action::Shift(StateId(3))], vec![]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 3);
}

#[test]
fn row_dedup_mixed_pattern() {
    let row_a = vec![vec![Action::Shift(StateId(1))], vec![Action::Error]];
    let row_b = vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]];
    let table = vec![row_a.clone(), row_b.clone(), row_a, row_b];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[2]);
    assert_eq!(compressed.state_to_row[1], compressed.state_to_row[3]);
}

#[test]
fn row_dedup_preserves_action_lookup() {
    let table = vec![
        vec![vec![Action::Shift(StateId(5))], vec![]],
        vec![vec![Action::Shift(StateId(5))], vec![]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(5))
    );
    assert_eq!(
        decompress_action(&compressed, 1, 0),
        Action::Shift(StateId(5))
    );
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
    assert_eq!(decompress_action(&compressed, 1, 1), Action::Error);
}

#[test]
fn row_dedup_empty_table() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 0);
    assert!(compressed.state_to_row.is_empty());
}

// ── goto table compression ──────────────────────────────────────────────────

#[test]
fn goto_sparse_roundtrip() {
    let table = vec![
        vec![None, Some(StateId(10)), None],
        vec![Some(StateId(20)), None, None],
        vec![None, None, Some(StateId(30))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);

    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(10)));
    assert_eq!(decompress_goto(&compressed, 1, 0), Some(StateId(20)));
    assert_eq!(decompress_goto(&compressed, 2, 2), Some(StateId(30)));
}

#[test]
fn goto_all_none() {
    let table = vec![vec![None; 3]; 3];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 0);
    for state in 0..3 {
        for sym in 0..3 {
            assert_eq!(decompress_goto(&compressed, state, sym), None);
        }
    }
}

#[test]
fn goto_fully_populated() {
    let table = vec![
        vec![Some(StateId(1)), Some(StateId(2))],
        vec![Some(StateId(3)), Some(StateId(4))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(2)));
    assert_eq!(decompress_goto(&compressed, 1, 0), Some(StateId(3)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(4)));
}

#[test]
fn goto_empty_table() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 0);
}

// ── compression ratios ──────────────────────────────────────────────────────

#[test]
fn compression_ratio_all_duplicate_rows() {
    let row = vec![vec![Action::Shift(StateId(1))], vec![]];
    let table: Vec<Vec<Vec<Action>>> = vec![row; 100];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    let ratio = table.len() as f64 / compressed.unique_rows.len() as f64;
    assert!(ratio >= 100.0);
}

#[test]
fn compression_ratio_sparse_goto() {
    let mut table = vec![vec![None; 10]; 10];
    table[0][0] = Some(StateId(1));
    table[2][3] = Some(StateId(2));
    table[5][5] = Some(StateId(3));
    table[7][9] = Some(StateId(4));
    table[9][0] = Some(StateId(5));

    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 5);
    let density = compressed.entries.len() as f64 / 100.0;
    assert!(density < 0.1, "expected <10% density, got {density}");
}

// ── packed table integrity ──────────────────────────────────────────────────

#[test]
fn bitpack_large_table_integrity() {
    // 8 states × 8 symbols: first 4 rows shifts, last 4 rows reduces
    let mut table = Vec::new();
    for state in 0..4u16 {
        let row: Vec<Action> = (0..8u16)
            .map(|sym| Action::Shift(StateId(state * 8 + sym)))
            .collect();
        table.push(row);
    }
    for rule_base in 0..4u16 {
        let row: Vec<Action> = (0..8u16)
            .map(|sym| Action::Reduce(RuleId(rule_base * 8 + sym)))
            .collect();
        table.push(row);
    }

    let packed = BitPackedActionTable::from_table(&table);

    for state in 0..4 {
        for sym in 0..8 {
            let expected = Action::Shift(StateId((state * 8 + sym) as u16));
            assert_eq!(
                packed.decompress(state, sym),
                expected,
                "shift mismatch at state={state} sym={sym}"
            );
        }
    }
    for state in 4..8 {
        for sym in 0..8 {
            let rule_base = (state - 4) as u16;
            let expected = Action::Reduce(RuleId(rule_base * 8 + sym as u16));
            assert_eq!(
                packed.decompress(state, sym),
                expected,
                "reduce mismatch at state={state} sym={sym}"
            );
        }
    }
}

#[test]
fn bitpack_fork_does_not_corrupt_indexing() {
    let fork = vec![Action::Shift(StateId(10)), Action::Reduce(RuleId(20))];
    let table = vec![vec![
        Action::Shift(StateId(1)),
        Action::Fork(fork.clone()),
        Action::Reduce(RuleId(5)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Fork(fork));
    assert_eq!(packed.decompress(0, 2), Action::Reduce(RuleId(5)));
}

#[test]
fn bitpack_interleaved_errors_preserve_indexing() {
    let table = vec![vec![
        Action::Shift(StateId(1)),
        Action::Error,
        Action::Shift(StateId(2)),
        Action::Error,
        Action::Reduce(RuleId(3)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(0, 3), Action::Error);
    assert_eq!(packed.decompress(0, 4), Action::Reduce(RuleId(3)));
}
