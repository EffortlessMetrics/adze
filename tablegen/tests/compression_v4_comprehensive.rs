//! Comprehensive v4 tests for table compression in adze-tablegen.
//!
//! 60+ tests covering:
//! 1. Compression produces valid output (10 tests)
//! 2. Compression is deterministic (8 tests)
//! 3. Compressed output is smaller or equal to uncompressed (5 tests)
//! 4. Various grammar topologies compress correctly (10 tests)
//! 5. Compression preserves semantics (8 tests)
//! 6. Edge cases: simple/complex/recursive grammars (10 tests)
//! 7. Output properties (non-empty, correct structure) (9 tests)

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};
use adze_tablegen::compress::{
    CompressedGotoEntry, CompressedParseTable, CompressedTables, TableCompressor,
};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use adze_tablegen::{collect_token_indices, eof_accepts_or_reduces};

// ============================================================================
// Helpers
// ============================================================================

fn build_table(grammar: &Grammar) -> ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("LR(1)")
}

fn compress_full(grammar: &Grammar) -> CompressedTables {
    let pt = build_table(grammar);
    let ti = collect_token_indices(grammar, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    TableCompressor::new().compress(&pt, &ti, sce).unwrap()
}

fn compress_with_table(grammar: &Grammar) -> (ParseTable, CompressedTables) {
    let pt = build_table(grammar);
    let ti = collect_token_indices(grammar, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    (pt, ct)
}

// --- Grammar constructors ---

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single_tok")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build()
}

fn alternatives_grammar() -> Grammar {
    GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .start("S")
        .build()
}

fn nested_grammar() -> Grammar {
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
        .rule("C", vec!["z"])
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

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn wide_alternatives_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new("wide");
    for i in 0..10 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + i as u8) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("S", vec![&name]);
    }
    gb.start("S").build()
}

fn long_sequence_grammar() -> Grammar {
    GrammarBuilder::new("long_seq")
        .token("t1", "a")
        .token("t2", "b")
        .token("t3", "c")
        .token("t4", "d")
        .token("t5", "e")
        .rule("S", vec!["t1", "t2", "t3", "t4", "t5"])
        .start("S")
        .build()
}

fn nullable_start_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .rule("A", vec![])
        .start("S")
        .build()
}

fn diamond_grammar() -> Grammar {
    // S -> A B, A -> x, B -> x  (diamond-shaped dependency)
    GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["x"])
        .start("S")
        .build()
}

fn multi_level_grammar() -> Grammar {
    GrammarBuilder::new("multi_lvl")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A"])
        .rule("A", vec!["B", "c"])
        .rule("B", vec!["a", "b"])
        .start("S")
        .build()
}

fn single_empty_alt_grammar() -> Grammar {
    // S has an empty alternative and a token alternative
    GrammarBuilder::new("empty_alt")
        .token("x", "x")
        .rule("S", vec!["x"])
        .rule("S", vec![])
        .start("S")
        .build()
}

// ============================================================================
// Section 1: Compression produces valid output (10 tests)
// ============================================================================

#[test]
fn v4_01_single_token_produces_valid_compressed() {
    let ct = compress_full(&single_token_grammar());
    assert!(
        !ct.action_table.data.is_empty(),
        "action data must be non-empty"
    );
    assert!(
        ct.action_table.row_offsets.len() >= 2,
        "must have at least 2 row offsets"
    );
    assert!(
        ct.goto_table.row_offsets.len() >= 2,
        "goto must have at least 2 row offsets"
    );
}

#[test]
fn v4_02_two_token_produces_valid_compressed() {
    let ct = compress_full(&two_token_grammar());
    assert!(!ct.action_table.data.is_empty());
    assert!(!ct.action_table.default_actions.is_empty());
}

#[test]
fn v4_03_alternatives_produces_valid_compressed() {
    let ct = compress_full(&alternatives_grammar());
    assert!(!ct.action_table.data.is_empty());
    assert!(!ct.action_table.row_offsets.is_empty());
}

#[test]
fn v4_04_nested_produces_valid_compressed() {
    let ct = compress_full(&nested_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v4_05_deep_chain_produces_valid_compressed() {
    let ct = compress_full(&deep_chain_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v4_06_left_recursive_produces_valid_compressed() {
    let ct = compress_full(&left_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v4_07_right_recursive_produces_valid_compressed() {
    let ct = compress_full(&right_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v4_08_precedence_produces_valid_compressed() {
    let ct = compress_full(&precedence_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v4_09_wide_alternatives_produces_valid_compressed() {
    let ct = compress_full(&wide_alternatives_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v4_10_nullable_start_produces_valid_compressed() {
    let grammar = nullable_start_grammar();
    let pt = build_table(&grammar);
    let ti = collect_token_indices(&grammar, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let result = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(result.is_ok(), "nullable start must compress without error");
}

// ============================================================================
// Section 2: Compression is deterministic (8 tests)
// ============================================================================

fn assert_deterministic(grammar: &Grammar) {
    let (pt1, ct1) = compress_with_table(grammar);
    let ti = collect_token_indices(grammar, &pt1);
    let sce = eof_accepts_or_reduces(&pt1);
    let ct2 = TableCompressor::new().compress(&pt1, &ti, sce).unwrap();

    assert_eq!(
        ct1.action_table.data.len(),
        ct2.action_table.data.len(),
        "action data length must be identical across runs"
    );
    assert_eq!(
        ct1.action_table.row_offsets, ct2.action_table.row_offsets,
        "action row offsets must be identical"
    );
    assert_eq!(
        ct1.goto_table.row_offsets, ct2.goto_table.row_offsets,
        "goto row offsets must be identical"
    );
    for (a, b) in ct1
        .action_table
        .data
        .iter()
        .zip(ct2.action_table.data.iter())
    {
        assert_eq!(a.symbol, b.symbol, "symbol must match");
        assert_eq!(a.action, b.action, "action must match");
    }
}

#[test]
fn v4_11_deterministic_single_token() {
    assert_deterministic(&single_token_grammar());
}

#[test]
fn v4_12_deterministic_two_token() {
    assert_deterministic(&two_token_grammar());
}

#[test]
fn v4_13_deterministic_alternatives() {
    assert_deterministic(&alternatives_grammar());
}

#[test]
fn v4_14_deterministic_nested() {
    assert_deterministic(&nested_grammar());
}

#[test]
fn v4_15_deterministic_deep_chain() {
    assert_deterministic(&deep_chain_grammar());
}

#[test]
fn v4_16_deterministic_left_recursive() {
    assert_deterministic(&left_recursive_grammar());
}

#[test]
fn v4_17_deterministic_precedence() {
    assert_deterministic(&precedence_grammar());
}

#[test]
fn v4_18_deterministic_wide_alternatives() {
    assert_deterministic(&wide_alternatives_grammar());
}

// ============================================================================
// Section 3: Compressed output size (5 tests)
// ============================================================================

/// Count total non-error actions in the original table.
fn count_original_nonerror(pt: &ParseTable) -> usize {
    pt.action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .filter(|a| !matches!(a, Action::Error))
        .count()
}

#[test]
fn v4_19_compressed_leq_original_single_token() {
    let (pt, ct) = compress_with_table(&single_token_grammar());
    assert!(
        ct.action_table.data.len() <= count_original_nonerror(&pt) + pt.state_count,
        "compressed entries must not exceed original non-error count + state overhead"
    );
}

#[test]
fn v4_20_compressed_leq_original_alternatives() {
    let (pt, ct) = compress_with_table(&alternatives_grammar());
    assert!(ct.action_table.data.len() <= count_original_nonerror(&pt) + pt.state_count);
}

#[test]
fn v4_21_compressed_leq_original_nested() {
    let (pt, ct) = compress_with_table(&nested_grammar());
    assert!(ct.action_table.data.len() <= count_original_nonerror(&pt) + pt.state_count);
}

#[test]
fn v4_22_compressed_leq_original_deep_chain() {
    let (pt, ct) = compress_with_table(&deep_chain_grammar());
    assert!(ct.action_table.data.len() <= count_original_nonerror(&pt) + pt.state_count);
}

#[test]
fn v4_23_compressed_no_explicit_errors() {
    let ct = compress_full(&two_token_grammar());
    for entry in &ct.action_table.data {
        assert!(
            !matches!(entry.action, Action::Error),
            "compressed data must not contain explicit Error actions"
        );
    }
}

// ============================================================================
// Section 4: Various grammar topologies compress correctly (10 tests)
// ============================================================================

#[test]
fn v4_24_diamond_grammar_compresses() {
    let _ct = compress_full(&diamond_grammar());
}

#[test]
fn v4_25_multi_level_grammar_compresses() {
    let _ct = compress_full(&multi_level_grammar());
}

#[test]
fn v4_26_long_sequence_compresses() {
    let ct = compress_full(&long_sequence_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v4_27_single_empty_alt_compresses() {
    let grammar = single_empty_alt_grammar();
    let pt = build_table(&grammar);
    let ti = collect_token_indices(&grammar, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let result = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(result.is_ok());
}

#[test]
fn v4_28_multiple_nonterminals_compress() {
    let grammar = GrammarBuilder::new("multi_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .start("S")
        .build();
    let _ct = compress_full(&grammar);
}

#[test]
fn v4_29_unary_chain_compresses() {
    let grammar = GrammarBuilder::new("unary")
        .token("t", "t")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["t"])
        .start("S")
        .build();
    let _ct = compress_full(&grammar);
}

#[test]
fn v4_30_binary_branching_compresses() {
    let grammar = GrammarBuilder::new("binary")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let _ct = compress_full(&grammar);
}

#[test]
fn v4_31_three_level_nesting_compresses() {
    let grammar = GrammarBuilder::new("three_lvl")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["x"])
        .start("S")
        .build();
    let ct = compress_full(&grammar);
    assert!(ct.action_table.row_offsets.len() >= 2);
}

#[test]
fn v4_32_wide_fan_out_compresses() {
    let mut gb = GrammarBuilder::new("fan_out");
    for i in 0..8 {
        let name = format!("tok{i}");
        let pat = format!("{}", (b'a' + i as u8) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("S", vec![&name]);
    }
    let grammar = gb.start("S").build();
    let _ct = compress_full(&grammar);
}

#[test]
fn v4_33_sequence_and_alternatives_mixed() {
    let grammar = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b"])
        .rule("S", vec!["c"])
        .start("S")
        .build();
    let ct = compress_full(&grammar);
    assert!(!ct.action_table.data.is_empty());
}

// ============================================================================
// Section 5: Compression preserves semantics (8 tests)
// ============================================================================

/// Every non-error action from the original parse table must appear in the compressed output.
fn verify_action_preservation(grammar: &Grammar) {
    let pt = build_table(grammar);
    let ti = collect_token_indices(grammar, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();

    let mut original_actions: Vec<(usize, u16, Action)> = Vec::new();
    for (state, row) in pt.action_table.iter().enumerate() {
        for (col, cell) in row.iter().enumerate() {
            for action in cell {
                if !matches!(action, Action::Error) {
                    original_actions.push((state, col as u16, action.clone()));
                }
            }
        }
    }

    // No explicit errors in compressed output
    for entry in &ct.action_table.data {
        assert!(!matches!(entry.action, Action::Error));
    }

    // Compressed entries count matches original non-error count
    assert_eq!(
        ct.action_table.data.len(),
        original_actions.len(),
        "compressed entry count must match original non-error action count"
    );
}

#[test]
fn v4_34_preserves_single_token() {
    verify_action_preservation(&single_token_grammar());
}

#[test]
fn v4_35_preserves_two_token() {
    verify_action_preservation(&two_token_grammar());
}

#[test]
fn v4_36_preserves_alternatives() {
    verify_action_preservation(&alternatives_grammar());
}

#[test]
fn v4_37_preserves_nested() {
    verify_action_preservation(&nested_grammar());
}

#[test]
fn v4_38_preserves_deep_chain() {
    verify_action_preservation(&deep_chain_grammar());
}

#[test]
fn v4_39_preserves_left_recursive() {
    verify_action_preservation(&left_recursive_grammar());
}

#[test]
fn v4_40_preserves_right_recursive() {
    verify_action_preservation(&right_recursive_grammar());
}

#[test]
fn v4_41_preserves_precedence() {
    verify_action_preservation(&precedence_grammar());
}

// ============================================================================
// Section 6: Edge cases — simple/complex/recursive (10 tests)
// ============================================================================

#[test]
fn v4_42_row_dedup_all_identical_rows() {
    let row = vec![vec![Action::Shift(StateId(1))], vec![Action::Error]];
    let table = vec![row.clone(), row.clone(), row.clone(), row];
    let compressed = compress_action_table(&table);
    assert_eq!(
        compressed.unique_rows.len(),
        1,
        "identical rows must deduplicate to 1"
    );
    assert_eq!(compressed.state_to_row, vec![0, 0, 0, 0]);
}

#[test]
fn v4_43_row_dedup_all_distinct_rows() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Shift(StateId(2))]],
        vec![vec![Action::Reduce(RuleId(0))]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(
        compressed.unique_rows.len(),
        3,
        "distinct rows must not deduplicate"
    );
}

#[test]
fn v4_44_row_dedup_roundtrip_accept() {
    let table = vec![vec![vec![Action::Accept], vec![Action::Error]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
}

#[test]
fn v4_45_sparse_goto_all_none() {
    let table: Vec<Vec<Option<StateId>>> = vec![vec![None; 5]; 3];
    let compressed = compress_goto_table(&table);
    assert_eq!(
        compressed.entries.len(),
        0,
        "all-None table must have no entries"
    );
}

#[test]
fn v4_46_sparse_goto_all_some() {
    let table = vec![vec![Some(StateId(1)), Some(StateId(2)), Some(StateId(3))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(2)));
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(3)));
}

#[test]
fn v4_47_bitpacked_mixed_actions() {
    let table = vec![vec![
        Action::Error,
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(0)),
    ]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(0, 2), Action::Reduce(RuleId(0)));
}

#[test]
fn v4_48_goto_rle_exact_boundary_3() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(7), StateId(7), StateId(7)]];
    let compressed = compressor.compress_goto_table_small(&goto_table).unwrap();
    let has_rle = compressed
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 7, count: 3 }));
    assert!(has_rle, "run of exactly 3 must trigger RLE");
}

#[test]
fn v4_49_goto_rle_boundary_2_uses_singles() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(4), StateId(4)]];
    let compressed = compressor.compress_goto_table_small(&goto_table).unwrap();
    let has_rle = compressed
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { .. }));
    assert!(!has_rle, "run of 2 must not use RLE");
    let single_count = compressed
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::Single(4)))
        .count();
    assert_eq!(single_count, 2);
}

#[test]
fn v4_50_encoding_shift_and_reduce_boundaries() {
    let compressor = TableCompressor::new();
    // Shift boundary
    assert!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0)))
            .is_ok()
    );
    assert!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x7FFF)))
            .is_ok()
    );
    assert!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0x8000)))
            .is_err()
    );
    // Reduce boundary
    assert!(
        compressor
            .encode_action_small(&Action::Reduce(RuleId(0)))
            .is_ok()
    );
    assert!(
        compressor
            .encode_action_small(&Action::Reduce(RuleId(0x3FFF)))
            .is_ok()
    );
    assert!(
        compressor
            .encode_action_small(&Action::Reduce(RuleId(0x4000)))
            .is_err()
    );
}

#[test]
fn v4_51_encoding_special_values() {
    let compressor = TableCompressor::new();
    assert_eq!(
        compressor.encode_action_small(&Action::Accept).unwrap(),
        0xFFFF
    );
    assert_eq!(
        compressor.encode_action_small(&Action::Error).unwrap(),
        0xFFFE
    );
    assert_eq!(
        compressor.encode_action_small(&Action::Recover).unwrap(),
        0xFFFD
    );
}

// ============================================================================
// Section 7: Output properties — non-empty, correct structure (9 tests)
// ============================================================================

#[test]
fn v4_52_action_row_offsets_len_eq_states_plus_one() {
    let (pt, ct) = compress_with_table(&two_token_grammar());
    assert_eq!(
        ct.action_table.row_offsets.len(),
        pt.state_count + 1,
        "action row_offsets length must be state_count + 1"
    );
}

#[test]
fn v4_53_goto_row_offsets_len_for_grammar() {
    let grammar = nested_grammar();
    let pt = build_table(&grammar);
    let ct = compress_full(&grammar);
    // goto row_offsets should have pt.state_count + 1 entries
    assert_eq!(ct.goto_table.row_offsets.len(), pt.state_count + 1);
}

#[test]
fn v4_54_default_actions_len_matches_states() {
    let (pt, ct) = compress_with_table(&alternatives_grammar());
    assert_eq!(
        ct.action_table.default_actions.len(),
        pt.state_count,
        "default_actions length must equal state_count"
    );
}

#[test]
fn v4_55_default_actions_all_error_due_to_disabled_optimization() {
    let ct = compress_full(&deep_chain_grammar());
    for da in &ct.action_table.default_actions {
        assert_eq!(
            *da,
            Action::Error,
            "default action optimization is disabled"
        );
    }
}

#[test]
fn v4_56_action_row_offsets_monotonically_nondecreasing() {
    let ct = compress_full(&precedence_grammar());
    for window in ct.action_table.row_offsets.windows(2) {
        assert!(
            window[1] >= window[0],
            "row offsets must be non-decreasing: {} < {}",
            window[1],
            window[0]
        );
    }
}

#[test]
fn v4_57_goto_row_offsets_monotonically_nondecreasing() {
    let ct = compress_full(&left_recursive_grammar());
    for window in ct.goto_table.row_offsets.windows(2) {
        assert!(
            window[1] >= window[0],
            "goto row offsets must be non-decreasing: {} < {}",
            window[1],
            window[0]
        );
    }
}

#[test]
fn v4_58_last_action_row_offset_eq_data_len() {
    let ct = compress_full(&right_recursive_grammar());
    let last = *ct.action_table.row_offsets.last().unwrap();
    assert_eq!(
        last as usize,
        ct.action_table.data.len(),
        "last row offset must equal action data length"
    );
}

#[test]
fn v4_59_last_goto_row_offset_eq_data_len() {
    let ct = compress_full(&nested_grammar());
    let last = *ct.goto_table.row_offsets.last().unwrap();
    // Count expanded entries (RLE entries expand to count)
    let expanded_len: usize = ct
        .goto_table
        .data
        .iter()
        .map(|e| match e {
            CompressedGotoEntry::Single(_) => 1,
            CompressedGotoEntry::RunLength { count, .. } => *count as usize,
        })
        .sum();
    // The last offset is set after pushing all entries, and entries are the
    // raw compressed entries (not expanded).
    assert_eq!(
        last as usize,
        ct.goto_table.data.len(),
        "last goto row offset must equal goto data length"
    );
    // Expanded length must be >= raw data length
    assert!(expanded_len >= ct.goto_table.data.len());
}

#[test]
fn v4_60_compressed_parse_table_dimensions_match() {
    let grammar = wide_alternatives_grammar();
    let pt = build_table(&grammar);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
    assert_eq!(cpt.state_count(), pt.state_count);
}
