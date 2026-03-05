//! Advanced tests for `BitPackedActionTable` in `adze-tablegen`.
//!
//! 80+ tests covering:
//! - Compression roundtrips for action and goto tables
//! - Deterministic compression
//! - Same/different table identity
//! - State and symbol count preservation
//! - Size characteristics of compressed output
//! - Grammar-driven compression (precedence, alternatives, chains, recursion)
//! - TableCompressor full pipeline
//! - BitPackedActionTable creation and access patterns

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};
use adze_tablegen::compress::CompressedParseTable;
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use adze_tablegen::{TableCompressor, helpers};

// =============================================================================
// Helpers
// =============================================================================

/// Build a GLR action table (multi-action cells) from a flat single-action table.
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

/// Build grammar → first/follow → LR(1) parse table.
fn build_table(
    name: &str,
    build_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> adze_glr_core::ParseTable {
    let gb = GrammarBuilder::new(name);
    let g = build_fn(gb).build();
    let ff = FirstFollowSets::compute(&g).expect("first/follow");
    build_lr1_automaton(&g, &ff).expect("lr1 automaton")
}

/// Build grammar and return both Grammar and ParseTable.
fn build_grammar_and_table(
    name: &str,
    build_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> (Grammar, adze_glr_core::ParseTable) {
    let gb = GrammarBuilder::new(name);
    let g = build_fn(gb).build();
    let ff = FirstFollowSets::compute(&g).expect("first/follow");
    let pt = build_lr1_automaton(&g, &ff).expect("lr1 automaton");
    (g, pt)
}

/// Compress via the full TableCompressor pipeline.
fn compress_full(g: &Grammar, pt: &adze_glr_core::ParseTable) -> adze_tablegen::CompressedTables {
    let tc = TableCompressor::new();
    let token_idx = helpers::collect_token_indices(g, pt);
    let nullable = helpers::eof_accepts_or_reduces(pt);
    tc.compress(pt, &token_idx, nullable).unwrap()
}

// =============================================================================
// 1. Compress action table → non-empty result (tests 1-3)
// =============================================================================

#[test]
fn bpa_v9_compress_action_single_shift_non_empty() {
    let table = to_glr(vec![vec![Action::Shift(StateId(1))]]);
    let compressed = compress_action_table(&table);
    assert!(!compressed.unique_rows.is_empty());
}

#[test]
fn bpa_v9_compress_action_mixed_non_empty() {
    let table = to_glr(vec![
        vec![Action::Shift(StateId(1)), Action::Error],
        vec![Action::Error, Action::Reduce(RuleId(0))],
    ]);
    let compressed = compress_action_table(&table);
    assert!(!compressed.unique_rows.is_empty());
    assert!(!compressed.state_to_row.is_empty());
}

#[test]
fn bpa_v9_compress_action_all_error_non_empty() {
    let table = to_glr(vec![vec![Action::Error; 5]; 5]);
    let compressed = compress_action_table(&table);
    assert!(!compressed.unique_rows.is_empty());
}

// =============================================================================
// 2. Compress goto table → non-empty result (tests 4-6)
// =============================================================================

#[test]
fn bpa_v9_compress_goto_single_entry_non_empty() {
    let table = vec![vec![Some(StateId(3)), None]];
    let compressed = compress_goto_table(&table);
    assert!(!compressed.entries.is_empty());
}

#[test]
fn bpa_v9_compress_goto_diagonal_non_empty() {
    let mut table = vec![vec![None; 4]; 4];
    for (i, row) in table.iter_mut().enumerate() {
        row[i] = Some(StateId(i as u16));
    }
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
}

#[test]
fn bpa_v9_compress_goto_all_none_is_empty() {
    let table = vec![vec![None; 3]; 3];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

// =============================================================================
// 3. Compression is deterministic (tests 7-9)
// =============================================================================

#[test]
fn bpa_v9_action_compression_deterministic() {
    let table = to_glr(vec![
        vec![
            Action::Shift(StateId(1)),
            Action::Error,
            Action::Reduce(RuleId(0)),
        ],
        vec![Action::Error, Action::Shift(StateId(2)), Action::Accept],
    ]);
    let c1 = compress_action_table(&table);
    let c2 = compress_action_table(&table);
    assert_eq!(c1.unique_rows.len(), c2.unique_rows.len());
    assert_eq!(c1.state_to_row, c2.state_to_row);
}

#[test]
fn bpa_v9_goto_compression_deterministic() {
    let table = vec![
        vec![Some(StateId(1)), None, Some(StateId(3))],
        vec![None, Some(StateId(2)), None],
    ];
    let c1 = compress_goto_table(&table);
    let c2 = compress_goto_table(&table);
    assert_eq!(c1.entries.len(), c2.entries.len());
    for (key, val) in &c1.entries {
        assert_eq!(c2.entries.get(key), Some(val));
    }
}

#[test]
fn bpa_v9_bitpacked_deterministic() {
    let table = vec![
        vec![
            Action::Shift(StateId(1)),
            Action::Error,
            Action::Reduce(RuleId(2)),
        ],
        vec![Action::Error, Action::Shift(StateId(3)), Action::Accept],
    ];
    let p1 = BitPackedActionTable::from_table(&table);
    let p2 = BitPackedActionTable::from_table(&table);
    for s in 0..2 {
        for sym in 0..3 {
            assert_eq!(p1.decompress(s, sym), p2.decompress(s, sym));
        }
    }
}

// =============================================================================
// 4. Same table → same compressed form (tests 10-12)
// =============================================================================

#[test]
fn bpa_v9_same_action_table_same_unique_rows() {
    let row = vec![Action::Shift(StateId(1)), Action::Error];
    let table = to_glr(vec![row.clone(), row]);
    let c1 = compress_action_table(&table);
    let c2 = compress_action_table(&table);
    assert_eq!(c1.unique_rows, c2.unique_rows);
}

#[test]
fn bpa_v9_same_goto_table_same_entries() {
    let table = vec![vec![Some(StateId(10)), None, Some(StateId(20))]];
    let c1 = compress_goto_table(&table);
    let c2 = compress_goto_table(&table);
    assert_eq!(c1.entries, c2.entries);
}

#[test]
fn bpa_v9_same_bitpacked_same_decompress() {
    let table = vec![vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))]];
    let p1 = BitPackedActionTable::from_table(&table);
    let p2 = BitPackedActionTable::from_table(&table);
    assert_eq!(p1.decompress(0, 0), p2.decompress(0, 0));
    assert_eq!(p1.decompress(0, 1), p2.decompress(0, 1));
}

// =============================================================================
// 5. Different tables → different compressed (tests 13-15)
// =============================================================================

#[test]
fn bpa_v9_different_action_tables_different_unique_rows() {
    let t1 = to_glr(vec![vec![Action::Shift(StateId(1))]]);
    let t2 = to_glr(vec![vec![Action::Reduce(RuleId(0))]]);
    let c1 = compress_action_table(&t1);
    let c2 = compress_action_table(&t2);
    assert_ne!(c1.unique_rows, c2.unique_rows);
}

#[test]
fn bpa_v9_different_goto_tables_different_entries() {
    let t1 = vec![vec![Some(StateId(1)), None]];
    let t2 = vec![vec![None, Some(StateId(2))]];
    let c1 = compress_goto_table(&t1);
    let c2 = compress_goto_table(&t2);
    assert_ne!(c1.entries, c2.entries);
}

#[test]
fn bpa_v9_different_bitpacked_different_decompress() {
    let t1 = vec![vec![Action::Shift(StateId(1))]];
    let t2 = vec![vec![Action::Reduce(RuleId(0))]];
    let p1 = BitPackedActionTable::from_table(&t1);
    let p2 = BitPackedActionTable::from_table(&t2);
    assert_ne!(p1.decompress(0, 0), p2.decompress(0, 0));
}

// =============================================================================
// 6. Compressed table preserves state count (tests 16-18)
// =============================================================================

#[test]
fn bpa_v9_action_preserves_state_count_1() {
    let table = to_glr(vec![vec![Action::Shift(StateId(0))]]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.state_to_row.len(), 1);
}

#[test]
fn bpa_v9_action_preserves_state_count_5() {
    let table = to_glr(vec![
        vec![Action::Shift(StateId(0))],
        vec![Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2))],
        vec![Action::Shift(StateId(3))],
        vec![Action::Shift(StateId(4))],
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.state_to_row.len(), 5);
}

#[test]
fn bpa_v9_action_preserves_state_count_with_dedup() {
    let row = vec![Action::Shift(StateId(1)), Action::Error];
    let table = to_glr(vec![row.clone(), row.clone(), row]);
    let compressed = compress_action_table(&table);
    // state_to_row has one entry per state even when rows are deduped
    assert_eq!(compressed.state_to_row.len(), 3);
    assert_eq!(compressed.unique_rows.len(), 1);
}

// =============================================================================
// 7. Compressed table preserves symbol count (tests 19-21)
// =============================================================================

#[test]
fn bpa_v9_action_preserves_symbol_count_1() {
    let table = to_glr(vec![vec![Action::Error]]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows[0].len(), 1);
}

#[test]
fn bpa_v9_action_preserves_symbol_count_10() {
    let table = to_glr(vec![vec![Action::Error; 10]]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows[0].len(), 10);
}

#[test]
fn bpa_v9_goto_preserves_entry_values() {
    let table = vec![
        vec![Some(StateId(7)), None, Some(StateId(99))],
        vec![None, Some(StateId(42)), None],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(7)));
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(99)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(42)));
}

// =============================================================================
// 8. Compress then decompress → same actions (tests 22-26)
// =============================================================================

#[test]
fn bpa_v9_action_roundtrip_shift_reduce_accept() {
    let table = to_glr(vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ]]);
    assert_action_rt(&table);
}

#[test]
fn bpa_v9_action_roundtrip_multi_row() {
    let table = to_glr(vec![
        vec![Action::Shift(StateId(0)), Action::Error],
        vec![Action::Error, Action::Reduce(RuleId(1))],
        vec![Action::Accept, Action::Error],
    ]);
    assert_action_rt(&table);
}

#[test]
fn bpa_v9_goto_roundtrip_sparse() {
    let mut table = vec![vec![None; 5]; 5];
    table[0][1] = Some(StateId(10));
    table[2][3] = Some(StateId(20));
    table[4][4] = Some(StateId(30));
    assert_goto_rt(&table);
}

#[test]
fn bpa_v9_bitpacked_roundtrip_mixed() {
    let table = vec![
        vec![
            Action::Shift(StateId(1)),
            Action::Error,
            Action::Shift(StateId(2)),
        ],
        vec![Action::Error, Action::Error, Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(1, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Error);
    assert_eq!(packed.decompress(1, 2), Action::Error);
}

#[test]
fn bpa_v9_bitpacked_roundtrip_fork() {
    let inner = vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(4))];
    let table = vec![vec![Action::Fork(inner.clone()), Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(inner));
    assert_eq!(packed.decompress(0, 1), Action::Error);
}

// =============================================================================
// 9. Compression reduces size (or at least doesn't grow) (tests 27-29)
// =============================================================================

#[test]
fn bpa_v9_dedup_reduces_unique_rows() {
    let row = vec![
        Action::Shift(StateId(1)),
        Action::Error,
        Action::Reduce(RuleId(0)),
    ];
    let table = to_glr(vec![row.clone(), row.clone(), row.clone(), row]);
    let compressed = compress_action_table(&table);
    assert!(compressed.unique_rows.len() < 4);
}

#[test]
fn bpa_v9_all_identical_rows_one_unique() {
    let row = vec![Action::Error; 20];
    let table = to_glr(vec![
        row.clone(),
        row.clone(),
        row.clone(),
        row.clone(),
        row,
    ]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn bpa_v9_goto_sparse_fewer_entries_than_cells() {
    let mut table = vec![vec![None; 10]; 10];
    table[0][0] = Some(StateId(1));
    table[5][5] = Some(StateId(2));
    let compressed = compress_goto_table(&table);
    // 2 entries vs 100 total cells
    assert!(compressed.entries.len() < 100);
    assert_eq!(compressed.entries.len(), 2);
}

// =============================================================================
// 10. Minimal grammar → small compressed table (tests 30-32)
// =============================================================================

#[test]
fn bpa_v9_minimal_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_min", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_minimal_grammar_small_action_table() {
    let (g, pt) = build_grammar_and_table("bpa_v9_min2", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    // A grammar with one token should produce a small table
    assert!(compressed.action_table.data.len() <= 20);
}

#[test]
fn bpa_v9_minimal_grammar_few_row_offsets() {
    let (g, pt) = build_grammar_and_table("bpa_v9_min3", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(compressed.action_table.row_offsets.len() <= 10);
}

// =============================================================================
// 11. Larger grammar → larger compressed table (tests 33-35)
// =============================================================================

#[test]
fn bpa_v9_larger_grammar_more_action_entries() {
    let (g_small, pt_small) = build_grammar_and_table("bpa_v9_sm", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let (g_large, pt_large) = build_grammar_and_table("bpa_v9_lg", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["x"])
            .rule("start", vec!["y"])
            .rule("x", vec!["a", "b"])
            .rule("y", vec!["c", "d"])
            .start("start")
    });
    let c_small = compress_full(&g_small, &pt_small);
    let c_large = compress_full(&g_large, &pt_large);
    assert!(c_large.action_table.data.len() >= c_small.action_table.data.len());
}

#[test]
fn bpa_v9_larger_grammar_more_states() {
    let pt_small = build_table("bpa_v9_s2", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let pt_large = build_table("bpa_v9_l2", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start")
    });
    assert!(pt_large.state_count >= pt_small.state_count);
}

#[test]
fn bpa_v9_larger_grammar_more_row_offsets() {
    let (g_small, pt_small) = build_grammar_and_table("bpa_v9_s3", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let (g_large, pt_large) = build_grammar_and_table("bpa_v9_l3", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["w"])
            .rule("w", vec!["a", "b", "c", "d"])
            .start("start")
    });
    let c_small = compress_full(&g_small, &pt_small);
    let c_large = compress_full(&g_large, &pt_large);
    assert!(c_large.action_table.row_offsets.len() >= c_small.action_table.row_offsets.len());
}

// =============================================================================
// 12. Grammar with precedence → compressible (tests 36-38)
// =============================================================================

#[test]
fn bpa_v9_precedence_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_prec", |gb| {
        gb.token("n", "[0-9]+")
            .token("plus", "\\+")
            .rule("start", vec!["e"])
            .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
            .rule("e", vec!["n"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_precedence_grammar_validate() {
    let (g, pt) = build_grammar_and_table("bpa_v9_prec2", |gb| {
        gb.token("n", "[0-9]+")
            .token("plus", "\\+")
            .rule("start", vec!["e"])
            .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
            .rule("e", vec!["n"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    compressed.validate(&pt).unwrap();
}

#[test]
fn bpa_v9_right_assoc_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_rprec", |gb| {
        gb.token("n", "[0-9]+")
            .token("star", "\\*")
            .rule("start", vec!["e"])
            .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Right)
            .rule("e", vec!["n"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

// =============================================================================
// 13. Grammar with alternatives → compressible (tests 39-41)
// =============================================================================

#[test]
fn bpa_v9_two_alt_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_alt2", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_three_alt_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_alt3", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_alt_grammar_has_default_actions() {
    let (g, pt) = build_grammar_and_table("bpa_v9_alt4", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.default_actions.is_empty());
}

// =============================================================================
// 14. Grammar with chain rules → compressible (tests 42-44)
// =============================================================================

#[test]
fn bpa_v9_chain_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_ch1", |gb| {
        gb.token("a", "a")
            .rule("start", vec!["middle"])
            .rule("middle", vec!["a"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_deep_chain_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_ch2", |gb| {
        gb.token("a", "a")
            .rule("start", vec!["level1"])
            .rule("level1", vec!["level2"])
            .rule("level2", vec!["a"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn bpa_v9_chain_with_alt_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_ch3", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["x"])
            .rule("start", vec!["y"])
            .rule("x", vec!["a"])
            .rule("y", vec!["b"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

// =============================================================================
// 15. Grammar with recursion → compressible (tests 45-47)
// =============================================================================

#[test]
fn bpa_v9_right_recursive_grammar_compresses() {
    let (g, pt) = build_grammar_and_table("bpa_v9_rr1", |gb| {
        gb.token("a", "a")
            .rule("start", vec!["list"])
            .rule("list", vec!["a", "list"])
            .rule("list", vec!["a"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_recursive_grammar_validates() {
    let (g, pt) = build_grammar_and_table("bpa_v9_rr2", |gb| {
        gb.token("a", "a")
            .rule("start", vec!["list"])
            .rule("list", vec!["a", "list"])
            .rule("list", vec!["a"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    compressed.validate(&pt).unwrap();
}

#[test]
fn bpa_v9_recursive_with_multiple_tokens() {
    let (g, pt) = build_grammar_and_table("bpa_v9_rr3", |gb| {
        gb.token("a", "a")
            .token("comma", ",")
            .rule("start", vec!["list"])
            .rule("list", vec!["a", "comma", "list"])
            .rule("list", vec!["a"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

// =============================================================================
// 16. TableCompressor from parse table (tests 48-50)
// =============================================================================

#[test]
fn bpa_v9_table_compressor_new_does_not_panic() {
    let _tc = TableCompressor::new();
}

#[test]
fn bpa_v9_table_compressor_default_does_not_panic() {
    let _tc: TableCompressor = Default::default();
}

#[test]
fn bpa_v9_compressed_parse_table_from_parse_table() {
    let pt = build_table("bpa_v9_cpt", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

// =============================================================================
// 17. TableCompressor compress method (tests 51-54)
// =============================================================================

#[test]
fn bpa_v9_compressor_compress_succeeds() {
    let (g, pt) = build_grammar_and_table("bpa_v9_cc1", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_compressor_compress_alt_grammar() {
    let (g, pt) = build_grammar_and_table("bpa_v9_cc2", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn bpa_v9_compressor_encode_shift() {
    let tc = TableCompressor::new();
    let encoded = tc.encode_action_small(&Action::Shift(StateId(42))).unwrap();
    assert_eq!(encoded, 42);
}

#[test]
fn bpa_v9_compressor_encode_reduce() {
    let tc = TableCompressor::new();
    let encoded = tc.encode_action_small(&Action::Reduce(RuleId(5))).unwrap();
    // Reduce: bit 15 set, 1-based rule ID
    assert_eq!(encoded, 0x8000 | 6);
}

// =============================================================================
// 18. Compressed table Debug format (tests 55-57)
// =============================================================================

#[test]
fn bpa_v9_compressed_action_table_debug() {
    let (g, pt) = build_grammar_and_table("bpa_v9_dbg1", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    let debug_str = format!("{:?}", compressed.action_table);
    assert!(!debug_str.is_empty());
}

#[test]
fn bpa_v9_compressed_goto_table_debug() {
    let (g, pt) = build_grammar_and_table("bpa_v9_dbg2", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    let debug_str = format!("{:?}", compressed.goto_table);
    assert!(!debug_str.is_empty());
}

#[test]
fn bpa_v9_compressed_action_entry_debug() {
    let (g, pt) = build_grammar_and_table("bpa_v9_dbg3", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    if let Some(entry) = compressed.action_table.data.first() {
        let debug_str = format!("{:?}", entry);
        assert!(!debug_str.is_empty());
    }
}

// =============================================================================
// 19. Various grammar sizes (tests 58-62)
// =============================================================================

#[test]
fn bpa_v9_single_token_grammar() {
    let (g, pt) = build_grammar_and_table("bpa_v9_sz1", |gb| {
        gb.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.row_offsets.is_empty());
}

#[test]
fn bpa_v9_two_token_sequence_grammar() {
    let (g, pt) = build_grammar_and_table("bpa_v9_sz2", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(compressed.action_table.row_offsets.len() >= 2);
}

#[test]
fn bpa_v9_three_token_sequence_grammar() {
    let (g, pt) = build_grammar_and_table("bpa_v9_sz3", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(compressed.action_table.row_offsets.len() >= 3);
}

#[test]
fn bpa_v9_four_alt_grammar() {
    let (g, pt) = build_grammar_and_table("bpa_v9_sz4", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .rule("start", vec!["d"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn bpa_v9_mixed_alts_and_sequence() {
    let (g, pt) = build_grammar_and_table("bpa_v9_sz5", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["x"])
            .rule("start", vec!["y"])
            .rule("x", vec!["a", "b"])
            .rule("y", vec!["c"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

// =============================================================================
// 20. BitPackedActionTable creation and access (tests 63-70)
// =============================================================================

#[test]
fn bpa_v9_bitpacked_from_empty_table() {
    let table: Vec<Vec<Action>> = vec![];
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
fn bpa_v9_bitpacked_from_single_error() {
    let table = vec![vec![Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn bpa_v9_bitpacked_from_single_shift() {
    let table = vec![vec![Action::Shift(StateId(42))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(42)));
}

#[test]
fn bpa_v9_bitpacked_from_single_reduce() {
    let table = vec![vec![Action::Reduce(RuleId(7))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(7)));
}

#[test]
fn bpa_v9_bitpacked_from_single_accept() {
    let table = vec![vec![Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Accept);
}

#[test]
fn bpa_v9_bitpacked_recover_maps_to_error() {
    let table = vec![vec![Action::Recover]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn bpa_v9_bitpacked_fork_roundtrip() {
    let inner = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    let table = vec![vec![Action::Fork(inner.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(inner));
}

#[test]
fn bpa_v9_bitpacked_multi_row_multi_col() {
    let table = vec![
        vec![
            Action::Shift(StateId(0)),
            Action::Error,
            Action::Shift(StateId(1)),
        ],
        vec![Action::Error, Action::Shift(StateId(2)), Action::Error],
        vec![Action::Error, Action::Error, Action::Accept],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(0)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(1, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(1, 2), Action::Error);
    assert_eq!(packed.decompress(2, 0), Action::Error);
    assert_eq!(packed.decompress(2, 1), Action::Error);
    assert_eq!(packed.decompress(2, 2), Action::Accept);
}

// =============================================================================
// Additional coverage: boundary, encoding, pipeline (tests 71-85)
// =============================================================================

#[test]
fn bpa_v9_bitpacked_65_cells_boundary() {
    let mut row = vec![Action::Error; 65];
    row[64] = Action::Shift(StateId(99));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 63), Action::Error);
    assert_eq!(packed.decompress(0, 64), Action::Shift(StateId(99)));
}

#[test]
fn bpa_v9_bitpacked_128_cells_all_errors() {
    let table = vec![vec![Action::Error; 128]];
    let packed = BitPackedActionTable::from_table(&table);
    for sym in 0..128 {
        assert_eq!(packed.decompress(0, sym), Action::Error);
    }
}

#[test]
fn bpa_v9_bitpacked_shift_at_word_boundaries() {
    let mut row = vec![Action::Error; 129];
    row[0] = Action::Shift(StateId(10));
    row[63] = Action::Shift(StateId(20));
    row[64] = Action::Shift(StateId(30));
    row[128] = Action::Shift(StateId(40));
    let table = vec![row];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(10)));
    assert_eq!(packed.decompress(0, 63), Action::Shift(StateId(20)));
    assert_eq!(packed.decompress(0, 64), Action::Shift(StateId(30)));
    assert_eq!(packed.decompress(0, 128), Action::Shift(StateId(40)));
}

#[test]
fn bpa_v9_encode_accept_is_ffff() {
    let tc = TableCompressor::new();
    assert_eq!(tc.encode_action_small(&Action::Accept).unwrap(), 0xFFFF);
}

#[test]
fn bpa_v9_encode_error_is_fffe() {
    let tc = TableCompressor::new();
    assert_eq!(tc.encode_action_small(&Action::Error).unwrap(), 0xFFFE);
}

#[test]
fn bpa_v9_encode_recover_is_fffd() {
    let tc = TableCompressor::new();
    assert_eq!(tc.encode_action_small(&Action::Recover).unwrap(), 0xFFFD);
}

#[test]
fn bpa_v9_encode_shift_overflow_errors() {
    let tc = TableCompressor::new();
    assert!(
        tc.encode_action_small(&Action::Shift(StateId(0x8000)))
            .is_err()
    );
}

#[test]
fn bpa_v9_encode_reduce_overflow_errors() {
    let tc = TableCompressor::new();
    assert!(
        tc.encode_action_small(&Action::Reduce(RuleId(0x4000)))
            .is_err()
    );
}

#[test]
fn bpa_v9_pipeline_deterministic_two_runs() {
    let (g, pt) = build_grammar_and_table("bpa_v9_det", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    let c1 = compress_full(&g, &pt);
    let c2 = compress_full(&g, &pt);
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
    assert_eq!(c1.goto_table.data.len(), c2.goto_table.data.len());
    assert_eq!(c1.action_table.row_offsets, c2.action_table.row_offsets);
    assert_eq!(c1.goto_table.row_offsets, c2.goto_table.row_offsets);
}

#[test]
fn bpa_v9_bitpacked_large_state_ids() {
    let table = vec![vec![
        Action::Shift(StateId(1000)),
        Action::Shift(StateId(5000)),
        Action::Shift(StateId(30000)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1000)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(5000)));
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(30000)));
}

#[test]
fn bpa_v9_action_dedup_maps_identical_rows() {
    let row = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let table = to_glr(vec![row.clone(), row.clone(), row]);
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[1]);
    assert_eq!(compressed.state_to_row[1], compressed.state_to_row[2]);
}

#[test]
fn bpa_v9_goto_roundtrip_large_state_ids() {
    let table = vec![vec![
        Some(StateId(0)),
        Some(StateId(u16::MAX)),
        None,
        Some(StateId(1000)),
    ]];
    assert_goto_rt(&table);
}

#[test]
fn bpa_v9_bitpacked_two_forks_same_row() {
    let f1 = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let f2 = vec![Action::Reduce(RuleId(2)), Action::Accept];
    let table = vec![vec![Action::Fork(f1.clone()), Action::Fork(f2.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(f1));
    assert_eq!(packed.decompress(0, 1), Action::Fork(f2));
}

#[test]
fn bpa_v9_full_pipeline_sequence_grammar() {
    let (g, pt) = build_grammar_and_table("bpa_v9_seq", |gb| {
        gb.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start")
    });
    let compressed = compress_full(&g, &pt);
    compressed.validate(&pt).unwrap();
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}
