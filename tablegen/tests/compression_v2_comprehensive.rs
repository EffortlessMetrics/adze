//! Comprehensive v2 tests for table compression in adze-tablegen.
//!
//! 50+ tests covering: simple/larger grammars, determinism, non-empty output,
//! grammar shapes, token stream length, multiple compressions, many states,
//! alternatives, edge cases, and semantic equivalence of compressed output.

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};
use adze_tablegen::compress::{
    CompressedActionEntry, CompressedActionTable, CompressedGotoEntry, CompressedGotoTable,
    CompressedParseTable, CompressedTables, TableCompressor,
};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use adze_tablegen::node_types::NodeTypesGenerator;
use adze_tablegen::{AbiLanguageBuilder, StaticLanguageGenerator, TableGenError};
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

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single_tok")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build()
}

fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build()
}

fn five_alt_grammar() -> Grammar {
    GrammarBuilder::new("five_alt")
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
    GrammarBuilder::new("long_seq")
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
    GrammarBuilder::new("nested")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep_chain")
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
    GrammarBuilder::new("left_rec")
        .token("a", "a")
        .rule("S", vec!["a"])
        .rule("S", vec!["S", "a"])
        .start("S")
        .build()
}

fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("S", vec!["a"])
        .rule("S", vec!["a", "S"])
        .start("S")
        .build()
}

fn nullable_start_grammar() -> Grammar {
    GrammarBuilder::new("nullable_start")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .rule("A", vec![])
        .start("S")
        .build()
}

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec_v2")
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
    let mut gb = GrammarBuilder::new("wide_alt");
    for i in 0..8 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + i as u8) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("S", vec![&name]);
    }
    gb.start("S").build()
}

// ============================================================================
// 1. Compression with simple grammar
// ============================================================================

#[test]
fn simple_grammar_compresses_without_error() {
    let g = single_token_grammar();
    let _ct = compress_grammar(&g);
}

#[test]
fn simple_grammar_action_table_has_entries() {
    let ct = compress_grammar(&single_token_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn simple_grammar_goto_table_has_row_offsets() {
    let ct = compress_grammar(&single_token_grammar());
    assert!(ct.goto_table.row_offsets.len() >= 2);
}

#[test]
fn simple_grammar_has_accept_action() {
    let ct = compress_grammar(&single_token_grammar());
    assert!(
        ct.action_table
            .data
            .iter()
            .any(|e| matches!(e.action, Action::Accept)),
        "must contain Accept"
    );
}

#[test]
fn simple_grammar_has_shift_action() {
    let ct = compress_grammar(&single_token_grammar());
    assert!(
        ct.action_table
            .data
            .iter()
            .any(|e| matches!(e.action, Action::Shift(_))),
        "must contain Shift"
    );
}

// ============================================================================
// 2. Compression with larger grammar
// ============================================================================

#[test]
fn larger_grammar_compresses_without_error() {
    let _ct = compress_grammar(&long_sequence_grammar());
}

#[test]
fn larger_grammar_more_states_than_simple() {
    let simple_ct = compress_grammar(&single_token_grammar());
    let large_ct = compress_grammar(&long_sequence_grammar());
    assert!(
        large_ct.action_table.row_offsets.len() > simple_ct.action_table.row_offsets.len(),
        "longer sequence needs more states"
    );
}

#[test]
fn larger_grammar_action_data_grows() {
    let simple_ct = compress_grammar(&single_token_grammar());
    let large_ct = compress_grammar(&long_sequence_grammar());
    assert!(large_ct.action_table.data.len() > simple_ct.action_table.data.len());
}

#[test]
fn deep_chain_grammar_compresses() {
    let ct = compress_grammar(&deep_chain_grammar());
    assert!(ct.action_table.row_offsets.len() >= 2);
    assert!(ct.goto_table.row_offsets.len() >= 2);
}

#[test]
fn nested_rules_grammar_compresses() {
    let ct = compress_grammar(&nested_rules_grammar());
    assert!(!ct.action_table.data.is_empty());
}

// ============================================================================
// 3. Compression determinism
// ============================================================================

#[test]
fn determinism_action_data_length() {
    let g = five_alt_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.action_table.data.len(), b.action_table.data.len());
}

#[test]
fn determinism_goto_data_length() {
    let g = five_alt_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.goto_table.data.len(), b.goto_table.data.len());
}

#[test]
fn determinism_action_row_offsets() {
    let g = nested_rules_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.action_table.row_offsets, b.action_table.row_offsets);
}

#[test]
fn determinism_goto_row_offsets() {
    let g = nested_rules_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.goto_table.row_offsets, b.goto_table.row_offsets);
}

#[test]
fn determinism_default_actions() {
    let g = long_sequence_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(
        a.action_table.default_actions,
        b.action_table.default_actions
    );
}

#[test]
fn determinism_action_entries_bitwise() {
    let g = two_alt_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    for (ea, eb) in a.action_table.data.iter().zip(b.action_table.data.iter()) {
        assert_eq!(ea.symbol, eb.symbol);
        assert_eq!(ea.action, eb.action);
    }
}

// ============================================================================
// 4. Compressed output is non-empty
// ============================================================================

#[test]
fn nonempty_action_data_single_token() {
    let ct = compress_grammar(&single_token_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn nonempty_action_data_two_alt() {
    let ct = compress_grammar(&two_alt_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn nonempty_goto_offsets_two_alt() {
    let ct = compress_grammar(&two_alt_grammar());
    assert!(!ct.goto_table.row_offsets.is_empty());
}

#[test]
fn nonempty_default_actions() {
    let ct = compress_grammar(&single_token_grammar());
    assert!(!ct.action_table.default_actions.is_empty());
}

#[test]
fn nonempty_action_row_offsets() {
    let ct = compress_grammar(&five_alt_grammar());
    assert!(ct.action_table.row_offsets.len() > 1);
}

// ============================================================================
// 5. Compression with different grammar shapes
// ============================================================================

#[test]
fn shape_left_recursive() {
    let ct = compress_grammar(&left_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn shape_right_recursive() {
    let ct = compress_grammar(&right_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn shape_nullable_start() {
    let ct = compress_grammar(&nullable_start_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn shape_precedence() {
    let ct = compress_grammar(&precedence_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn shape_wide_alternation() {
    let ct = compress_grammar(&wide_alt_grammar());
    assert!(
        ct.action_table.data.len() >= 8,
        "8 alternatives ⇒ 8+ actions"
    );
}

// ============================================================================
// 6. Compression preserves token stream length
// ============================================================================

#[test]
fn token_stream_nonempty_simple() {
    let g = single_token_grammar();
    let pt = build_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn token_stream_nonempty_large() {
    let g = long_sequence_grammar();
    let pt = build_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn token_stream_larger_grammar_has_more_tokens() {
    let g1 = single_token_grammar();
    let pt1 = build_table(g1.clone());
    let code1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();

    let g2 = long_sequence_grammar();
    let pt2 = build_table(g2.clone());
    let code2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();

    assert!(
        code2.len() > code1.len(),
        "larger grammar produces more code"
    );
}

#[test]
fn token_stream_contains_tslanguage() {
    let g = two_alt_grammar();
    let pt = build_table(g.clone());
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(code.contains("TSLanguage"));
}

// ============================================================================
// 7. Multiple compressions produce same output
// ============================================================================

#[test]
fn triple_compress_action_offsets_stable() {
    let g = left_recursive_grammar();
    let results: Vec<_> = (0..3).map(|_| compress_grammar(&g)).collect();
    assert_eq!(
        results[0].action_table.row_offsets,
        results[1].action_table.row_offsets,
    );
    assert_eq!(
        results[1].action_table.row_offsets,
        results[2].action_table.row_offsets,
    );
}

#[test]
fn triple_compress_goto_offsets_stable() {
    let g = deep_chain_grammar();
    let results: Vec<_> = (0..3).map(|_| compress_grammar(&g)).collect();
    assert_eq!(
        results[0].goto_table.row_offsets,
        results[1].goto_table.row_offsets,
    );
    assert_eq!(
        results[1].goto_table.row_offsets,
        results[2].goto_table.row_offsets,
    );
}

#[test]
fn five_compress_data_length_stable() {
    let g = wide_alt_grammar();
    let lengths: Vec<_> = (0..5)
        .map(|_| compress_grammar(&g).action_table.data.len())
        .collect();
    assert!(lengths.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn repeated_compress_same_small_table_threshold() {
    let g = single_token_grammar();
    let a = compress_grammar(&g);
    let b = compress_grammar(&g);
    assert_eq!(a.small_table_threshold, b.small_table_threshold);
}

// ============================================================================
// 8. Compression with many states
// ============================================================================

#[test]
fn many_states_long_sequence() {
    let g = long_sequence_grammar();
    let pt = build_table(g.clone());
    let ct = compress_grammar(&g);
    // 7-token sequence ⇒ at least 8 states
    assert!(
        ct.action_table.row_offsets.len() >= 8,
        "state count {}, expected >= 8",
        ct.action_table.row_offsets.len() - 1
    );
    assert_eq!(ct.action_table.row_offsets.len(), pt.state_count + 1);
}

#[test]
fn many_states_deep_chain() {
    let g = deep_chain_grammar();
    let pt = build_table(g.clone());
    let ct = compress_grammar(&g);
    assert_eq!(ct.action_table.row_offsets.len(), pt.state_count + 1);
    assert_eq!(ct.goto_table.row_offsets.len(), pt.state_count + 1);
}

#[test]
fn many_states_wide_alt() {
    let g = wide_alt_grammar();
    let pt = build_table(g.clone());
    let ct = compress_grammar(&g);
    assert_eq!(
        ct.action_table.default_actions.len(),
        pt.state_count,
        "one default action per state"
    );
}

#[test]
fn many_states_row_offsets_monotonic() {
    let g = long_sequence_grammar();
    let ct = compress_grammar(&g);
    for w in ct.action_table.row_offsets.windows(2) {
        assert!(
            w[1] >= w[0],
            "row offsets must be monotonically non-decreasing"
        );
    }
}

#[test]
fn many_states_goto_row_offsets_monotonic() {
    let g = deep_chain_grammar();
    let ct = compress_grammar(&g);
    for w in ct.goto_table.row_offsets.windows(2) {
        assert!(w[1] >= w[0]);
    }
}

// ============================================================================
// 9. Compression with alternatives
// ============================================================================

#[test]
fn alternatives_two_alts_both_reduce() {
    let ct = compress_grammar(&two_alt_grammar());
    let reduce_count = ct
        .action_table
        .data
        .iter()
        .filter(|e| matches!(e.action, Action::Reduce(_)))
        .count();
    assert!(reduce_count >= 2, "two alternatives ⇒ two reduces");
}

#[test]
fn alternatives_five_alts_all_reduce() {
    let ct = compress_grammar(&five_alt_grammar());
    let reduce_count = ct
        .action_table
        .data
        .iter()
        .filter(|e| matches!(e.action, Action::Reduce(_)))
        .count();
    assert!(reduce_count >= 5);
}

#[test]
fn alternatives_wide_alt_has_shifts_for_each_token() {
    let ct = compress_grammar(&wide_alt_grammar());
    let shift_count = ct
        .action_table
        .data
        .iter()
        .filter(|e| matches!(e.action, Action::Shift(_)))
        .count();
    assert!(
        shift_count >= 8,
        "8 alternatives ⇒ 8 shift actions in initial state"
    );
}

#[test]
fn alternatives_nested_structure() {
    let g = GrammarBuilder::new("nested_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["X"])
        .rule("X", vec!["a"])
        .rule("X", vec!["b"])
        .rule("X", vec!["c"])
        .start("S")
        .build();
    let ct = compress_grammar(&g);
    assert!(ct.action_table.data.len() >= 3);
}

// ============================================================================
// 10. Compression edge cases
// ============================================================================

#[test]
fn edge_case_compress_action_all_empty() {
    let c = TableCompressor::new();
    let at: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; 5]; 3];
    let s2i = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &s2i).unwrap();
    assert!(
        res.data.is_empty(),
        "all-empty table ⇒ no compressed entries"
    );
    assert_eq!(res.row_offsets.len(), 4); // 3 states + sentinel
}

#[test]
fn edge_case_compress_goto_empty_input() {
    let c = TableCompressor::new();
    let gt: Vec<Vec<StateId>> = vec![];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(res.data.is_empty());
}

#[test]
fn edge_case_compress_goto_single_zero_state() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(0)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert_eq!(res.data.len(), 1);
}

#[test]
fn edge_case_single_accept_action() {
    let c = TableCompressor::new();
    let at = vec![vec![vec![Action::Accept]]];
    let s2i = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &s2i).unwrap();
    assert_eq!(res.data.len(), 1);
    assert!(matches!(res.data[0].action, Action::Accept));
}

#[test]
fn edge_case_single_reduce_action() {
    let c = TableCompressor::new();
    let at = vec![vec![vec![Action::Reduce(RuleId(0))]]];
    let s2i = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &s2i).unwrap();
    assert_eq!(res.data.len(), 1);
    assert!(matches!(res.data[0].action, Action::Reduce(RuleId(0))));
}

#[test]
fn edge_case_encode_small_shift_zero() {
    let c = TableCompressor::new();
    let v = c.encode_action_small(&Action::Shift(StateId(0))).unwrap();
    assert_eq!(v, 0);
}

#[test]
fn edge_case_encode_small_reduce_zero() {
    let c = TableCompressor::new();
    let v = c.encode_action_small(&Action::Reduce(RuleId(0))).unwrap();
    assert_eq!(v, 0x8000 | 1); // 1-based encoding
}

#[test]
fn edge_case_goto_rle_boundary_run_of_2() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(3), StateId(3)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    // Run of 2 → Singles, not RunLength
    assert_eq!(res.data.len(), 2);
    assert!(
        res.data
            .iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(3)))
    );
}

#[test]
fn edge_case_goto_rle_run_of_4() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(10); 4]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(res.data.iter().any(|e| matches!(
        e,
        CompressedGotoEntry::RunLength {
            state: 10,
            count: 4
        }
    )));
}

#[test]
fn edge_case_mixed_goto_entries() {
    let c = TableCompressor::new();
    let gt = vec![vec![
        StateId(1),
        StateId(2),
        StateId(2),
        StateId(2),
        StateId(3),
    ]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(!res.data.is_empty());
}

// ============================================================================
// Additional: compressed vs uncompressed semantic equivalence
// ============================================================================

#[test]
fn compressed_action_not_larger_than_raw_simple() {
    let g = single_token_grammar();
    let pt = build_table(g.clone());
    let ct = compress_grammar(&g);
    let raw_cells: usize = pt.action_table.iter().map(|row| row.len()).sum();
    assert!(ct.action_table.data.len() <= raw_cells);
}

#[test]
fn compressed_action_not_larger_than_raw_wide() {
    let g = wide_alt_grammar();
    let pt = build_table(g.clone());
    let ct = compress_grammar(&g);
    let raw_cells: usize = pt.action_table.iter().map(|row| row.len()).sum();
    assert!(ct.action_table.data.len() <= raw_cells);
}

#[test]
fn sentinel_offset_equals_action_data_len() {
    let g = five_alt_grammar();
    let ct = compress_grammar(&g);
    let last = *ct.action_table.row_offsets.last().unwrap();
    assert_eq!(last as usize, ct.action_table.data.len());
}

#[test]
fn sentinel_offset_equals_goto_data_len() {
    let g = deep_chain_grammar();
    let ct = compress_grammar(&g);
    let last = *ct.goto_table.row_offsets.last().unwrap();
    assert_eq!(last as usize, ct.goto_table.data.len());
}

// ============================================================================
// Additional: compression module roundtrip (compress/decompress)
// ============================================================================

#[test]
fn roundtrip_action_shift_reduce_mixed() {
    let table = glr_table(vec![vec![
        Action::Shift(StateId(5)),
        Action::Reduce(RuleId(2)),
        Action::Error,
        Action::Accept,
    ]]);
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(5))
    );
    assert_eq!(
        decompress_action(&compressed, 0, 1),
        Action::Reduce(RuleId(2))
    );
    assert_eq!(decompress_action(&compressed, 0, 2), Action::Error);
    assert_eq!(decompress_action(&compressed, 0, 3), Action::Accept);
}

#[test]
fn roundtrip_goto_with_none_entries() {
    let table = vec![
        vec![Some(StateId(1)), None, Some(StateId(3))],
        vec![None, Some(StateId(2)), None],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(3)));
    assert_eq!(decompress_goto(&compressed, 1, 0), None);
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(2)));
    assert_eq!(decompress_goto(&compressed, 1, 2), None);
}

#[test]
fn roundtrip_action_all_error() {
    let table = glr_table(vec![vec![Action::Error; 4]; 3]);
    let compressed = compress_action_table(&table);
    for state in 0..3 {
        for sym in 0..4 {
            assert_eq!(decompress_action(&compressed, state, sym), Action::Error);
        }
    }
}

#[test]
fn roundtrip_goto_all_none() {
    let table: Vec<Vec<Option<StateId>>> = vec![vec![None; 3]; 2];
    let compressed = compress_goto_table(&table);
    for state in 0..2 {
        for sym in 0..3 {
            assert_eq!(decompress_goto(&compressed, state, sym), None);
        }
    }
}

// ============================================================================
// Additional: BitPackedActionTable
// ============================================================================

#[test]
fn bitpacked_roundtrip_shift_reduce() {
    // BitPackedActionTable uses a positional heuristic: all shifts in scan order
    // must appear before all reduces/accepts.
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
fn bitpacked_all_errors() {
    let table = vec![vec![Action::Error; 5]; 3];
    let packed = BitPackedActionTable::from_table(&table);
    for s in 0..3 {
        for sym in 0..5 {
            assert_eq!(packed.decompress(s, sym), Action::Error);
        }
    }
}

// ============================================================================
// Additional: AbiLanguageBuilder integration
// ============================================================================

#[test]
fn abi_builder_generates_nonempty_code() {
    let g = single_token_grammar();
    let pt = build_table(g.clone());
    let builder = AbiLanguageBuilder::new(&g, &pt);
    let code = builder.generate();
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_with_compressed_tables() {
    let g = two_alt_grammar();
    let pt = build_table(g.clone());
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    let builder = AbiLanguageBuilder::new(&g, &pt).with_compressed_tables(&ct);
    let code = builder.generate();
    assert!(!code.is_empty());
}

// ============================================================================
// Additional: compress_tables method on StaticLanguageGenerator
// ============================================================================

#[test]
fn static_gen_compress_tables_populates_field() {
    let g = single_token_grammar();
    let pt = build_table(g.clone());
    let mut slg = StaticLanguageGenerator::new(g, pt);
    assert!(slg.compressed_tables.is_none());
    slg.compress_tables().unwrap();
    assert!(slg.compressed_tables.is_some());
}

#[test]
fn static_gen_compress_tables_nullable() {
    let g = nullable_start_grammar();
    let pt = build_table(g.clone());
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.compress_tables().unwrap();
    assert!(slg.compressed_tables.is_some());
    assert!(
        slg.start_can_be_empty,
        "eof_accepts_or_reduces should detect nullable start"
    );
}
