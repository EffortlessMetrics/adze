//! Comprehensive v2 tests for RLE (run-length encoding) compression in adze-tablegen.
//!
//! 55+ tests covering:
//! 1. Compression produces valid output (8 tests)
//! 2. Decompression recovers original values (10 tests)
//! 3. RLE segments are non-empty (5 tests)
//! 4. Bit packing properties (8 tests)
//! 5. Compression ratio (5 tests)
//! 6. Various grammar topologies (8 tests)
//! 7. Determinism (5 tests)
//! 8. Edge cases (6 tests)

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};
use adze_tablegen::compress::{CompressedGotoEntry, CompressedTables, TableCompressor};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use adze_tablegen::{collect_token_indices, eof_accepts_or_reduces};
use std::collections::BTreeMap;

// ============================================================================
// Helpers
// ============================================================================

fn build_table(mut g: Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("LR(1)")
}

fn compress_grammar(g: &Grammar) -> CompressedTables {
    let pt = build_table(g.clone());
    let ti = collect_token_indices(g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    TableCompressor::new().compress(&pt, &ti, sce).unwrap()
}

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

// ── Grammar constructors ────────────────────────────────────────────────────

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("rle_single_tok")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build()
}

fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("rle_two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build()
}

fn five_alt_grammar() -> Grammar {
    GrammarBuilder::new("rle_five_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .rule("S", vec!["d"])
        .rule("S", vec!["e"])
        .start("S")
        .build()
}

fn long_sequence_grammar() -> Grammar {
    GrammarBuilder::new("rle_long_seq")
        .token("t1", "a")
        .token("t2", "b")
        .token("t3", "c")
        .token("t4", "d")
        .token("t5", "e")
        .token("t6", "f")
        .token("t7", "g")
        .rule("S", vec!["t1", "t2", "t3", "t4", "t5", "t6", "t7"])
        .start("S")
        .build()
}

fn nested_rules_grammar() -> Grammar {
    GrammarBuilder::new("rle_nested")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("rle_deep_chain")
        .token("z", "z")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["D"])
        .rule("D", vec!["z"])
        .start("S")
        .build()
}

fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rle_left_rec")
        .token("a", "a")
        .rule("S", vec!["a"])
        .rule("S", vec!["S", "a"])
        .start("S")
        .build()
}

fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rle_right_rec")
        .token("a", "a")
        .rule("S", vec!["a"])
        .rule("S", vec!["a", "S"])
        .start("S")
        .build()
}

fn nullable_start_grammar() -> Grammar {
    GrammarBuilder::new("rle_nullable_start")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .rule("A", vec![])
        .start("S")
        .build()
}

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("rle_prec")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn wide_alt_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new("rle_wide_alt");
    for i in 0..8 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + i as u8) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("S", vec![&name]);
    }
    gb.start("S").build()
}

fn diamond_grammar() -> Grammar {
    GrammarBuilder::new("rle_diamond")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build()
}

fn multi_level_grammar() -> Grammar {
    GrammarBuilder::new("rle_multi_level")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A"])
        .rule("A", vec!["B", "c"])
        .rule("B", vec!["a"])
        .rule("B", vec!["b"])
        .start("S")
        .build()
}

// ============================================================================
// 1. Compression produces valid output (8 tests)
// ============================================================================

#[test]
fn valid_output_single_token_no_panic() {
    let _ct = compress_grammar(&single_token_grammar());
}

#[test]
fn valid_output_action_table_has_data() {
    let ct = compress_grammar(&single_token_grammar());
    assert!(
        !ct.action_table.data.is_empty(),
        "action data must be non-empty"
    );
}

#[test]
fn valid_output_action_row_offsets_start_at_zero() {
    let ct = compress_grammar(&single_token_grammar());
    assert_eq!(
        ct.action_table.row_offsets[0], 0,
        "first row offset must be zero"
    );
}

#[test]
fn valid_output_goto_row_offsets_start_at_zero() {
    let ct = compress_grammar(&single_token_grammar());
    assert_eq!(ct.goto_table.row_offsets[0], 0);
}

#[test]
fn valid_output_sentinel_matches_action_data_len() {
    let ct = compress_grammar(&two_alt_grammar());
    let sentinel = *ct.action_table.row_offsets.last().unwrap();
    assert_eq!(sentinel as usize, ct.action_table.data.len());
}

#[test]
fn valid_output_sentinel_matches_goto_data_len() {
    let ct = compress_grammar(&nested_rules_grammar());
    let sentinel = *ct.goto_table.row_offsets.last().unwrap();
    assert_eq!(sentinel as usize, ct.goto_table.data.len());
}

#[test]
fn valid_output_action_offsets_monotonically_nondecreasing() {
    let ct = compress_grammar(&long_sequence_grammar());
    for w in ct.action_table.row_offsets.windows(2) {
        assert!(w[1] >= w[0], "row offsets must be non-decreasing");
    }
}

#[test]
fn valid_output_goto_offsets_monotonically_nondecreasing() {
    let ct = compress_grammar(&deep_chain_grammar());
    for w in ct.goto_table.row_offsets.windows(2) {
        assert!(w[1] >= w[0], "goto offsets must be non-decreasing");
    }
}

// ============================================================================
// 2. Decompression recovers original values (10 tests)
// ============================================================================

#[test]
fn decompress_action_shift_roundtrip() {
    let table = glr_table(vec![vec![Action::Shift(StateId(7)), Action::Error]]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(7))
    );
}

#[test]
fn decompress_action_reduce_roundtrip() {
    let table = glr_table(vec![vec![Action::Error, Action::Reduce(RuleId(3))]]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 1),
        Action::Reduce(RuleId(3))
    );
}

#[test]
fn decompress_action_accept_roundtrip() {
    let table = glr_table(vec![vec![Action::Accept]]);
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
}

#[test]
fn decompress_action_error_roundtrip() {
    let table = glr_table(vec![vec![Action::Error, Action::Error]]);
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
}

#[test]
fn decompress_action_mixed_row() {
    let table = glr_table(vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Error,
        Action::Accept,
    ]]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
    assert_eq!(
        decompress_action(&compressed, 0, 1),
        Action::Reduce(RuleId(0))
    );
    assert_eq!(decompress_action(&compressed, 0, 2), Action::Error);
    assert_eq!(decompress_action(&compressed, 0, 3), Action::Accept);
}

#[test]
fn decompress_action_multistate_table() {
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1)), Action::Error],
        vec![Action::Error, Action::Reduce(RuleId(0))],
        vec![Action::Accept, Action::Error],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
    assert_eq!(decompress_action(&compressed, 1, 0), Action::Error);
    assert_eq!(
        decompress_action(&compressed, 1, 1),
        Action::Reduce(RuleId(0))
    );
    assert_eq!(decompress_action(&compressed, 2, 0), Action::Accept);
    assert_eq!(decompress_action(&compressed, 2, 1), Action::Error);
}

#[test]
fn decompress_goto_some_roundtrip() {
    let table = vec![vec![None, Some(StateId(5)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(5)));
    assert_eq!(decompress_goto(&compressed, 0, 2), None);
}

#[test]
fn decompress_goto_all_some() {
    let table = vec![vec![Some(StateId(1)), Some(StateId(2)), Some(StateId(3))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(2)));
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(3)));
}

#[test]
fn decompress_goto_all_none() {
    let table: Vec<Vec<Option<StateId>>> = vec![vec![None; 4]; 3];
    let compressed = compress_goto_table(&table);
    for s in 0..3 {
        for sym in 0..4 {
            assert_eq!(decompress_goto(&compressed, s, sym), None);
        }
    }
}

#[test]
fn decompress_goto_multirow_sparse() {
    let table = vec![
        vec![Some(StateId(10)), None, None],
        vec![None, Some(StateId(20)), None],
        vec![None, None, Some(StateId(30))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(10)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(20)));
    assert_eq!(decompress_goto(&compressed, 2, 2), Some(StateId(30)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(decompress_goto(&compressed, 1, 0), None);
    assert_eq!(decompress_goto(&compressed, 2, 0), None);
}

// ============================================================================
// 3. RLE segments are non-empty (5 tests)
// ============================================================================

#[test]
fn rle_goto_single_entry_produces_output() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(1)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(!res.data.is_empty(), "single entry must produce output");
}

#[test]
fn rle_goto_run_of_3_produces_single_rle() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(5); 3]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(
        res.data
            .iter()
            .any(|e| matches!(e, CompressedGotoEntry::RunLength { count, .. } if *count == 3)),
        "run of 3 should produce RunLength entry"
    );
}

#[test]
fn rle_goto_run_of_4_produces_single_rle() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(9); 4]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(
        res.data
            .iter()
            .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 9, count: 4 }))
    );
}

#[test]
fn rle_goto_distinct_values_produce_singles() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(1), StateId(2), StateId(3)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert_eq!(res.data.len(), 3, "3 distinct values => 3 Single entries");
    assert!(
        res.data
            .iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(_)))
    );
}

#[test]
fn rle_goto_every_row_has_entries() {
    let c = TableCompressor::new();
    let gt = vec![
        vec![StateId(1), StateId(2)],
        vec![StateId(3), StateId(4)],
        vec![StateId(5), StateId(6)],
    ];
    let res = c.compress_goto_table_small(&gt).unwrap();
    // 3 rows + sentinel = 4 offsets; each row has at least 1 entry
    assert_eq!(res.row_offsets.len(), 4);
    for w in res.row_offsets.windows(2) {
        assert!(w[1] > w[0], "each row must have at least one entry");
    }
}

// ============================================================================
// 4. Bit packing properties (8 tests)
// ============================================================================

#[test]
fn bitpack_error_mask_all_errors() {
    let table = vec![vec![Action::Error; 3]; 2];
    let packed = BitPackedActionTable::from_table(&table);
    for s in 0..2 {
        for sym in 0..3 {
            assert_eq!(packed.decompress(s, sym), Action::Error);
        }
    }
}

#[test]
fn bitpack_shift_only_roundtrip() {
    let table = vec![vec![Action::Shift(StateId(10)), Action::Shift(StateId(20))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(10)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(20)));
}

#[test]
fn bitpack_reduce_only_roundtrip() {
    let table = vec![vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(0)));
    assert_eq!(packed.decompress(0, 1), Action::Reduce(RuleId(1)));
}

#[test]
fn bitpack_accept_roundtrip() {
    let table = vec![vec![Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Accept);
}

#[test]
fn bitpack_recover_maps_to_error() {
    let table = vec![vec![Action::Recover]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn bitpack_fork_roundtrip() {
    let actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let table = vec![vec![Action::Fork(actions.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(actions));
}

#[test]
fn bitpack_mixed_shift_then_reduce_row() {
    // BitPackedActionTable requires shifts before reduces in scan order
    let table = vec![
        vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))],
        vec![Action::Reduce(RuleId(0)), Action::Accept],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(1, 0), Action::Reduce(RuleId(0)));
    assert_eq!(packed.decompress(1, 1), Action::Accept);
}

#[test]
fn bitpack_error_interspersed_with_actions() {
    // Shifts before reduces in scan order, errors interspersed
    let table = vec![vec![
        Action::Shift(StateId(1)),
        Action::Error,
        Action::Shift(StateId(3)),
        Action::Error,
        Action::Reduce(RuleId(0)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(3)));
    assert_eq!(packed.decompress(0, 3), Action::Error);
    assert_eq!(packed.decompress(0, 4), Action::Reduce(RuleId(0)));
}

// ============================================================================
// 5. Compression ratio (5 tests)
// ============================================================================

#[test]
fn ratio_compressed_not_larger_than_raw_simple() {
    let g = single_token_grammar();
    let pt = build_table(g.clone());
    let ct = compress_grammar(&g);
    let raw_cells: usize = pt.action_table.iter().map(|row| row.len()).sum();
    assert!(
        ct.action_table.data.len() <= raw_cells,
        "compressed should not exceed raw cell count"
    );
}

#[test]
fn ratio_compressed_not_larger_than_raw_wide() {
    let g = wide_alt_grammar();
    let pt = build_table(g.clone());
    let ct = compress_grammar(&g);
    let raw_cells: usize = pt.action_table.iter().map(|row| row.len()).sum();
    assert!(ct.action_table.data.len() <= raw_cells);
}

#[test]
fn ratio_row_dedup_reduces_count() {
    // Two identical rows should deduplicate to 1 unique row
    let table = glr_table(vec![
        vec![Action::Shift(StateId(1)), Action::Error],
        vec![Action::Shift(StateId(1)), Action::Error],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        compressed.unique_rows.len(),
        1,
        "duplicate rows should be deduplicated"
    );
}

#[test]
fn ratio_goto_rle_reduces_entries_for_runs() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(7); 10]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(
        res.data.len() < 10,
        "run of 10 identical values must compress to fewer entries"
    );
}

#[test]
fn ratio_sparse_goto_only_stores_nonempty() {
    let table = vec![
        vec![None, None, None, Some(StateId(1)), None],
        vec![None, None, None, None, None],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(
        compressed.entries.len(),
        1,
        "only one non-None entry should be stored"
    );
}

// ============================================================================
// 6. Various grammar topologies (8 tests)
// ============================================================================

#[test]
fn topo_left_recursive_compresses() {
    let ct = compress_grammar(&left_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
    assert!(ct.action_table.row_offsets.len() >= 2);
}

#[test]
fn topo_right_recursive_compresses() {
    let ct = compress_grammar(&right_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn topo_nullable_start_compresses() {
    let ct = compress_grammar(&nullable_start_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn topo_precedence_compresses() {
    let ct = compress_grammar(&precedence_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn topo_deep_chain_has_goto_entries() {
    let ct = compress_grammar(&deep_chain_grammar());
    assert!(!ct.goto_table.data.is_empty(), "chain rules need gotos");
}

#[test]
fn topo_diamond_compresses() {
    let ct = compress_grammar(&diamond_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn topo_multi_level_compresses() {
    let ct = compress_grammar(&multi_level_grammar());
    assert!(!ct.action_table.data.is_empty());
    assert!(!ct.goto_table.data.is_empty());
}

#[test]
fn topo_wide_alt_has_shifts_and_reduces() {
    let ct = compress_grammar(&wide_alt_grammar());
    let has_shift = ct
        .action_table
        .data
        .iter()
        .any(|e| matches!(e.action, Action::Shift(_)));
    let has_reduce = ct
        .action_table
        .data
        .iter()
        .any(|e| matches!(e.action, Action::Reduce(_)));
    assert!(has_shift, "wide alt should have shifts");
    assert!(has_reduce, "wide alt should have reduces");
}

// ============================================================================
// 7. Determinism (5 tests)
// ============================================================================

#[test]
fn determinism_action_data_length_stable() {
    let g = five_alt_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.action_table.data.len(), b.action_table.data.len());
}

#[test]
fn determinism_goto_data_length_stable() {
    let g = nested_rules_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.goto_table.data.len(), b.goto_table.data.len());
}

#[test]
fn determinism_action_row_offsets_stable() {
    let g = long_sequence_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.action_table.row_offsets, b.action_table.row_offsets);
}

#[test]
fn determinism_goto_row_offsets_stable() {
    let g = deep_chain_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.goto_table.row_offsets, b.goto_table.row_offsets);
}

#[test]
fn determinism_triple_compress_all_fields() {
    let g = precedence_grammar();
    let runs: Vec<_> = (0..3).map(|_| compress_grammar(&g)).collect();
    for i in 1..3 {
        assert_eq!(
            runs[0].action_table.data.len(),
            runs[i].action_table.data.len()
        );
        assert_eq!(
            runs[0].action_table.row_offsets,
            runs[i].action_table.row_offsets
        );
        assert_eq!(
            runs[0].goto_table.row_offsets,
            runs[i].goto_table.row_offsets
        );
        assert_eq!(
            runs[0].action_table.default_actions,
            runs[i].action_table.default_actions
        );
    }
}

// ============================================================================
// 8. Edge cases (6 tests)
// ============================================================================

#[test]
fn edge_all_error_action_table() {
    let table = glr_table(vec![vec![Action::Error; 5]; 3]);
    let compressed = compress_action_table(&table);
    for s in 0..3 {
        for sym in 0..5 {
            assert_eq!(decompress_action(&compressed, s, sym), Action::Error);
        }
    }
}

#[test]
fn edge_single_cell_table() {
    let table = glr_table(vec![vec![Action::Shift(StateId(0))]]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(0))
    );
}

#[test]
fn edge_goto_rle_boundary_run_of_2_is_singles() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(3), StateId(3)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert_eq!(res.data.len(), 2, "run of 2 => 2 Singles, not RunLength");
    assert!(
        res.data
            .iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(3)))
    );
}

#[test]
fn edge_goto_rle_alternating_values() {
    let c = TableCompressor::new();
    let gt = vec![vec![
        StateId(1),
        StateId(2),
        StateId(1),
        StateId(2),
        StateId(1),
    ]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert_eq!(
        res.data.len(),
        5,
        "alternating values cannot be run-length compressed"
    );
}

#[test]
fn edge_compress_action_table_small_empty_cells() {
    let c = TableCompressor::new();
    let at: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; 4]; 2];
    let s2i = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &s2i).unwrap();
    assert!(res.data.is_empty(), "all-empty cells => no entries");
    assert_eq!(res.row_offsets.len(), 3, "2 states + sentinel");
}

#[test]
fn edge_goto_empty_table() {
    let c = TableCompressor::new();
    let gt: Vec<Vec<StateId>> = vec![];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(res.data.is_empty());
    assert_eq!(res.row_offsets.len(), 1, "just the sentinel");
}
