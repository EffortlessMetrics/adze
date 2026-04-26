//! Comprehensive tests for compression boundary conditions in `adze-tablegen`.
//!
//! Covers 85+ tests across categories:
//! 1. Compress minimal grammar → succeeds
//! 2. Compress 2-token grammar → succeeds
//! 3. Compress 5-token grammar → succeeds
//! 4. Compress 10-token grammar → succeeds
//! 5. Compress arithmetic grammar → succeeds
//! 6. Compressed size < uncompressed
//! 7. Compressed is deterministic
//! 8. Compress twice → same output
//! 9. Compress then decompress → roundtrip
//! 10. Bit-packed lookup matches original
//! 11. Compress empty-ish grammar → small output
//! 12. Compress chain grammar → succeeds
//! 13. Compress alternatives grammar → succeeds
//! 14. Compress precedence grammar → succeeds
//! 15. Compress with extras → succeeds
//! 16. Compress with inline → succeeds
//! 17. Compress with externals → succeeds
//! 18. Compression ratio improves with sparsity
//! 19. Action table compression → valid
//! 20. Goto table compression → valid

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};
use adze_tablegen::compress::{CompressedParseTable, CompressedTables, TableCompressor};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use adze_tablegen::{collect_token_indices, eof_accepts_or_reduces};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Full pipeline: Grammar → FIRST/FOLLOW → LR(1) → CompressedTables.
fn cb_v9_compress_pipeline(grammar: &mut Grammar) -> (adze_glr_core::ParseTable, CompressedTables) {
    let ff = FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(grammar, &ff).expect("LR(1) automaton construction failed");
    let token_indices = collect_token_indices(grammar, &table);
    let start_empty = eof_accepts_or_reduces(&table);
    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &token_indices, start_empty)
        .expect("Table compression failed");
    (table, compressed)
}

/// Minimal grammar: start → a.
fn cb_v9_minimal() -> Grammar {
    GrammarBuilder::new("cb_v9_minimal")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Two-token grammar: start → a | b.
fn cb_v9_two_token() -> Grammar {
    GrammarBuilder::new("cb_v9_two")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

/// N-token alternatives grammar: start → t0 | t1 | … | tN-1.
fn cb_v9_n_token(n: usize) -> Grammar {
    let name: &str = Box::leak(format!("cb_v9_{n}tok").into_boxed_str());
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok: &str = Box::leak(format!("t{i}").into_boxed_str());
        b = b.token(tok, tok).rule("start", vec![tok]);
    }
    b.start("start").build()
}

/// Arithmetic grammar with precedence.
fn cb_v9_arithmetic() -> Grammar {
    GrammarBuilder::new("cb_v9_arith")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

/// Chain grammar: r0 → x, r1 → r0, …, start → rN-1.
fn cb_v9_chain(n: usize) -> Grammar {
    let name: &str = Box::leak(format!("cb_v9_chain{n}").into_boxed_str());
    let mut b = GrammarBuilder::new(name).token("x", "x");
    let names: Vec<String> = (0..n).map(|i| format!("r{i}")).collect();
    let first: &str = Box::leak(names[0].clone().into_boxed_str());
    b = b.rule(first, vec!["x"]);
    for i in 1..n {
        let lhs: &str = Box::leak(names[i].clone().into_boxed_str());
        let rhs: &str = Box::leak(names[i - 1].clone().into_boxed_str());
        b = b.rule(lhs, vec![rhs]);
    }
    let last: &str = Box::leak(names[n - 1].clone().into_boxed_str());
    b = b.rule("start", vec![last]);
    b.start("start").build()
}

/// Left-recursive grammar: list → list a | a, start → list.
fn cb_v9_left_recursive() -> Grammar {
    GrammarBuilder::new("cb_v9_leftrec")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .rule("start", vec!["list"])
        .start("start")
        .build()
}

/// Right-recursive grammar: list → a list | a, start → list.
fn cb_v9_right_recursive() -> Grammar {
    GrammarBuilder::new("cb_v9_rightrec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .rule("start", vec!["list"])
        .start("start")
        .build()
}

/// Grammar with extras: start → a, whitespace is extra.
fn cb_v9_extras() -> Grammar {
    GrammarBuilder::new("cb_v9_extras")
        .token("a", "a")
        .token("ws", r"\s+")
        .rule("start", vec!["a"])
        .start("start")
        .extra("ws")
        .build()
}

/// Grammar with inline rule: inner inlined into start.
fn cb_v9_inline() -> Grammar {
    GrammarBuilder::new("cb_v9_inline")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .inline("inner")
        .build()
}

/// Grammar with externals.
fn cb_v9_external() -> Grammar {
    GrammarBuilder::new("cb_v9_ext")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .external("ext_tok")
        .build()
}

/// Grammar with right-associative precedence.
fn cb_v9_right_assoc() -> Grammar {
    GrammarBuilder::new("cb_v9_rassoc")
        .token("num", r"\d+")
        .token("pow", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 1, Associativity::Right)
        .start("expr")
        .build()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Compress minimal grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_minimal_compress_succeeds() {
    let mut g = cb_v9_minimal();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_minimal_compress_goto_nonempty() {
    let mut g = cb_v9_minimal();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cb_v9_minimal_has_row_offsets() {
    let mut g = cb_v9_minimal();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.row_offsets.is_empty());
    assert!(!compressed.goto_table.row_offsets.is_empty());
}

#[test]
fn cb_v9_minimal_validate_passes() {
    let mut g = cb_v9_minimal();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Compress 2-token grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_two_token_compress_succeeds() {
    let mut g = cb_v9_two_token();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_two_token_goto_nonempty() {
    let mut g = cb_v9_two_token();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cb_v9_two_token_validate_passes() {
    let mut g = cb_v9_two_token();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_two_token_more_states_than_minimal() {
    let mut g_min = cb_v9_minimal();
    let (pt_min, _) = cb_v9_compress_pipeline(&mut g_min);
    let mut g_two = cb_v9_two_token();
    let (pt_two, _) = cb_v9_compress_pipeline(&mut g_two);
    assert!(pt_two.state_count >= pt_min.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Compress 5-token grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_five_token_compress_succeeds() {
    let mut g = cb_v9_n_token(5);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_five_token_validate_passes() {
    let mut g = cb_v9_n_token(5);
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_five_token_has_default_actions() {
    let mut g = cb_v9_n_token(5);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.default_actions.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Compress 10-token grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_ten_token_compress_succeeds() {
    let mut g = cb_v9_n_token(10);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_ten_token_validate_passes() {
    let mut g = cb_v9_n_token(10);
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_ten_token_row_offsets_match_state_count() {
    let mut g = cb_v9_n_token(10);
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cb_v9_ten_token_more_entries_than_five() {
    let mut g5 = cb_v9_n_token(5);
    let (_, c5) = cb_v9_compress_pipeline(&mut g5);
    let mut g10 = cb_v9_n_token(10);
    let (_, c10) = cb_v9_compress_pipeline(&mut g10);
    assert!(c10.action_table.data.len() >= c5.action_table.data.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Compress arithmetic grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_arithmetic_compress_succeeds() {
    let mut g = cb_v9_arithmetic();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cb_v9_arithmetic_validate_passes() {
    let mut g = cb_v9_arithmetic();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_arithmetic_row_offsets_monotonic() {
    let mut g = cb_v9_arithmetic();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    for pair in compressed.action_table.row_offsets.windows(2) {
        assert!(pair[0] <= pair[1]);
    }
}

#[test]
fn cb_v9_arithmetic_goto_row_offsets_monotonic() {
    let mut g = cb_v9_arithmetic();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    for pair in compressed.goto_table.row_offsets.windows(2) {
        assert!(pair[0] <= pair[1]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Compressed size < uncompressed
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_action_dedup_reduces_rows() {
    // All-error table: 10 identical rows → 1 unique row.
    let table = vec![vec![vec![Action::Error]; 5]; 10];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert!(compressed.unique_rows.len() < table.len());
}

#[test]
fn cb_v9_goto_sparse_reduces_entries() {
    // 10x10 with only 2 entries.
    let mut table = vec![vec![None; 10]; 10];
    table[0][0] = Some(StateId(1));
    table[9][9] = Some(StateId(2));
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 2);
    assert!(compressed.entries.len() < 100);
}

#[test]
fn cb_v9_half_duplicate_rows_compress() {
    let a = vec![vec![Action::Shift(StateId(0))]; 4];
    let b = vec![vec![Action::Reduce(RuleId(0))]; 4];
    let table = vec![a.clone(), b.clone(), a, b];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
}

#[test]
fn cb_v9_compressed_parse_table_size_coherent() {
    let mut g = cb_v9_n_token(8);
    let (pt, _) = cb_v9_compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
    assert_eq!(cpt.state_count(), pt.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Compressed is deterministic
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_determinism_action_data_len() {
    let mut g1 = cb_v9_arithmetic();
    let (_, c1) = cb_v9_compress_pipeline(&mut g1);
    let mut g2 = cb_v9_arithmetic();
    let (_, c2) = cb_v9_compress_pipeline(&mut g2);
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
}

#[test]
fn cb_v9_determinism_goto_data_len() {
    let mut g1 = cb_v9_arithmetic();
    let (_, c1) = cb_v9_compress_pipeline(&mut g1);
    let mut g2 = cb_v9_arithmetic();
    let (_, c2) = cb_v9_compress_pipeline(&mut g2);
    assert_eq!(c1.goto_table.data.len(), c2.goto_table.data.len());
}

#[test]
fn cb_v9_determinism_action_row_offsets() {
    let mut g1 = cb_v9_minimal();
    let (_, c1) = cb_v9_compress_pipeline(&mut g1);
    let mut g2 = cb_v9_minimal();
    let (_, c2) = cb_v9_compress_pipeline(&mut g2);
    assert_eq!(c1.action_table.row_offsets, c2.action_table.row_offsets);
}

#[test]
fn cb_v9_determinism_goto_row_offsets() {
    let mut g1 = cb_v9_minimal();
    let (_, c1) = cb_v9_compress_pipeline(&mut g1);
    let mut g2 = cb_v9_minimal();
    let (_, c2) = cb_v9_compress_pipeline(&mut g2);
    assert_eq!(c1.goto_table.row_offsets, c2.goto_table.row_offsets);
}

#[test]
fn cb_v9_determinism_threshold() {
    let mut g1 = cb_v9_arithmetic();
    let (_, c1) = cb_v9_compress_pipeline(&mut g1);
    let mut g2 = cb_v9_arithmetic();
    let (_, c2) = cb_v9_compress_pipeline(&mut g2);
    assert_eq!(c1.small_table_threshold, c2.small_table_threshold);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Compress twice → same output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_double_compress_action_stable() {
    let table = vec![
        vec![vec![Action::Shift(StateId(0))], vec![Action::Error]],
        vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]],
    ];
    let c1 = compress_action_table(&table);
    let c2 = compress_action_table(&table);
    assert_eq!(c1.unique_rows.len(), c2.unique_rows.len());
    assert_eq!(c1.state_to_row, c2.state_to_row);
}

#[test]
fn cb_v9_double_compress_goto_stable() {
    let mut table = vec![vec![None; 3]; 3];
    table[0][1] = Some(StateId(5));
    table[2][0] = Some(StateId(7));
    let c1 = compress_goto_table(&table);
    let c2 = compress_goto_table(&table);
    assert_eq!(c1.entries.len(), c2.entries.len());
}

#[test]
fn cb_v9_double_pipeline_compress_stable() {
    let mut g1 = cb_v9_two_token();
    let (_, c1) = cb_v9_compress_pipeline(&mut g1);
    let mut g2 = cb_v9_two_token();
    let (_, c2) = cb_v9_compress_pipeline(&mut g2);
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
    assert_eq!(c1.goto_table.data.len(), c2.goto_table.data.len());
    assert_eq!(
        c1.action_table.default_actions.len(),
        c2.action_table.default_actions.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Compress then decompress → roundtrip
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_roundtrip_action_shift() {
    let table = vec![vec![vec![Action::Shift(StateId(42))]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(42))
    );
}

#[test]
fn cb_v9_roundtrip_action_reduce() {
    let table = vec![vec![vec![Action::Reduce(RuleId(7))]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Reduce(RuleId(7))
    );
}

#[test]
fn cb_v9_roundtrip_action_accept() {
    let table = vec![vec![vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
}

#[test]
fn cb_v9_roundtrip_action_error() {
    let table = vec![vec![vec![Action::Error]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
}

#[test]
fn cb_v9_roundtrip_action_recover() {
    let table = vec![vec![vec![Action::Recover]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Recover);
}

#[test]
fn cb_v9_roundtrip_action_glr_first() {
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
fn cb_v9_roundtrip_goto_present() {
    let table = vec![vec![Some(StateId(99))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(99)));
}

#[test]
fn cb_v9_roundtrip_goto_absent() {
    let table = vec![vec![None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
}

#[test]
fn cb_v9_roundtrip_action_mixed_row() {
    let table = vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(2))],
        vec![Action::Accept],
        vec![Action::Error],
    ]];
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Shift(StateId(1)));
    assert_eq!(decompress_action(&c, 0, 1), Action::Reduce(RuleId(2)));
    assert_eq!(decompress_action(&c, 0, 2), Action::Accept);
    assert_eq!(decompress_action(&c, 0, 3), Action::Error);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Bit-packed lookup matches original
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_bitpack_single_shift() {
    let table = vec![vec![Action::Shift(StateId(5))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(5)));
}

#[test]
fn cb_v9_bitpack_single_reduce() {
    let table = vec![vec![Action::Reduce(RuleId(3))]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(3)));
}

#[test]
fn cb_v9_bitpack_single_accept() {
    let table = vec![vec![Action::Accept]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Accept);
}

#[test]
fn cb_v9_bitpack_error_roundtrip() {
    let table = vec![vec![Action::Error]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn cb_v9_bitpack_recover_as_error() {
    let table = vec![vec![Action::Recover]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
}

#[test]
fn cb_v9_bitpack_fork_roundtrip() {
    let fork = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let table = vec![vec![Action::Fork(fork.clone())]];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(fork));
}

#[test]
fn cb_v9_bitpack_all_error_row() {
    let table = vec![vec![Action::Error; 6]];
    let packed = BitPackedActionTable::from_table(&table);
    for col in 0..6 {
        assert_eq!(packed.decompress(0, col), Action::Error);
    }
}

#[test]
fn cb_v9_bitpack_all_shift_row() {
    let table = vec![
        (0..4)
            .map(|i| Action::Shift(StateId(i)))
            .collect::<Vec<_>>(),
    ];
    let packed = BitPackedActionTable::from_table(&table);
    for col in 0..4 {
        assert_eq!(
            packed.decompress(0, col),
            Action::Shift(StateId(col as u16))
        );
    }
}

#[test]
fn cb_v9_bitpack_3x3_error_positions() {
    let table = vec![
        vec![
            Action::Error,
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
        ],
        vec![Action::Shift(StateId(2)), Action::Error, Action::Accept],
        vec![Action::Reduce(RuleId(1)), Action::Accept, Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    // Error cells must roundtrip.
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Error);
    assert_eq!(packed.decompress(2, 2), Action::Error);
    // Non-error cells must not be Error.
    assert_ne!(packed.decompress(0, 1), Action::Error);
    assert_ne!(packed.decompress(0, 2), Action::Error);
    assert_ne!(packed.decompress(1, 0), Action::Error);
    assert_ne!(packed.decompress(1, 2), Action::Error);
    assert_ne!(packed.decompress(2, 0), Action::Error);
    assert_ne!(packed.decompress(2, 1), Action::Error);
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Compress empty-ish grammar → small output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_emptyish_action_table() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let compressed = compress_action_table(&table);
    assert!(compressed.unique_rows.is_empty());
    assert!(compressed.state_to_row.is_empty());
}

#[test]
fn cb_v9_emptyish_goto_table() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn cb_v9_all_none_goto() {
    let table = vec![vec![None; 5]; 5];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn cb_v9_all_error_action() {
    let table = vec![vec![vec![Action::Error]; 5]; 5];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

#[test]
fn cb_v9_single_cell_action() {
    let table = vec![vec![vec![Action::Shift(StateId(0))]]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Compress chain grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_chain_3_compress_succeeds() {
    let mut g = cb_v9_chain(3);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_chain_5_validate_passes() {
    let mut g = cb_v9_chain(5);
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_chain_8_row_offsets_bounded() {
    let mut g = cb_v9_chain(8);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    let data_len = compressed.action_table.data.len();
    for &offset in &compressed.action_table.row_offsets {
        assert!(usize::from(offset) <= data_len);
    }
}

#[test]
fn cb_v9_chain_depth_increases_states() {
    let mut g3 = cb_v9_chain(3);
    let (pt3, _) = cb_v9_compress_pipeline(&mut g3);
    let mut g8 = cb_v9_chain(8);
    let (pt8, _) = cb_v9_compress_pipeline(&mut g8);
    assert!(pt8.state_count >= pt3.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Compress alternatives grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_alternatives_3_compress_succeeds() {
    let mut g = cb_v9_n_token(3);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_alternatives_8_validate_passes() {
    let mut g = cb_v9_n_token(8);
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_alternatives_15_compress_succeeds() {
    let mut g = cb_v9_n_token(15);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_alternatives_scaling_more_tokens_more_data() {
    let mut g3 = cb_v9_n_token(3);
    let (_, c3) = cb_v9_compress_pipeline(&mut g3);
    let mut g10 = cb_v9_n_token(10);
    let (_, c10) = cb_v9_compress_pipeline(&mut g10);
    assert!(c10.action_table.data.len() >= c3.action_table.data.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Compress precedence grammar → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_precedence_left_assoc_compress_succeeds() {
    let mut g = cb_v9_arithmetic();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_precedence_right_assoc_compress_succeeds() {
    let mut g = cb_v9_right_assoc();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_precedence_validate_passes() {
    let mut g = cb_v9_right_assoc();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_precedence_preserves_state_count() {
    let mut g = cb_v9_arithmetic();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Compress with extras → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_extras_compress_succeeds() {
    let mut g = cb_v9_extras();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_extras_validate_passes() {
    let mut g = cb_v9_extras();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Compress with inline → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_inline_compress_succeeds() {
    let mut g = cb_v9_inline();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_inline_validate_passes() {
    let mut g = cb_v9_inline();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Compress with externals → succeeds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_external_compress_succeeds() {
    let mut g = cb_v9_external();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_external_validate_passes() {
    let mut g = cb_v9_external();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Compression ratio improves with sparsity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_sparse_goto_fewer_entries() {
    // Sparse: only 3 entries in 10x10 vs dense: all 100.
    let mut sparse = vec![vec![None; 10]; 10];
    sparse[0][0] = Some(StateId(1));
    sparse[5][5] = Some(StateId(2));
    sparse[9][9] = Some(StateId(3));
    let c_sparse = compress_goto_table(&sparse);

    let dense: Vec<Vec<Option<StateId>>> = (0..10)
        .map(|s| {
            (0..10)
                .map(|c| Some(StateId((s * 10 + c) as u16)))
                .collect()
        })
        .collect();
    let c_dense = compress_goto_table(&dense);
    assert!(c_sparse.entries.len() < c_dense.entries.len());
}

#[test]
fn cb_v9_homogeneous_rows_compress_better() {
    // All identical rows → 1 unique row.
    let homogeneous = vec![vec![vec![Action::Error]; 8]; 20];
    let c_homo = compress_action_table(&homogeneous);

    // All different rows → 20 unique rows.
    let heterogeneous: Vec<Vec<Vec<Action>>> = (0..20)
        .map(|i| vec![vec![Action::Shift(StateId(i as u16))]; 8])
        .collect();
    let c_het = compress_action_table(&heterogeneous);
    assert!(c_homo.unique_rows.len() < c_het.unique_rows.len());
}

#[test]
fn cb_v9_sparsity_50pct_vs_100pct_goto() {
    // 50% sparse.
    let mut half = vec![vec![None; 4]; 4];
    half[0][0] = Some(StateId(1));
    half[1][1] = Some(StateId(2));
    half[2][2] = Some(StateId(3));
    half[3][3] = Some(StateId(4));
    // diag only: 4 entries.
    let c_half = compress_goto_table(&half);

    // 100% full.
    let full: Vec<Vec<Option<StateId>>> = (0..4)
        .map(|s| (0..4).map(|c| Some(StateId((s * 4 + c) as u16))).collect())
        .collect();
    let c_full = compress_goto_table(&full);
    assert!(c_half.entries.len() < c_full.entries.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. Action table compression → valid
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_action_state_to_row_indices_valid() {
    let table = vec![
        vec![vec![Action::Shift(StateId(0))]],
        vec![vec![Action::Reduce(RuleId(0))]],
        vec![vec![Action::Shift(StateId(0))]],
    ];
    let compressed = compress_action_table(&table);
    for &idx in &compressed.state_to_row {
        assert!(idx < compressed.unique_rows.len());
    }
}

#[test]
fn cb_v9_action_state_to_row_len_matches_states() {
    let table = vec![vec![vec![Action::Error]; 3], vec![vec![Action::Accept]; 3]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.state_to_row.len(), 2);
}

#[test]
fn cb_v9_action_unique_rows_preserve_columns() {
    let table = vec![vec![
        vec![Action::Shift(StateId(0))],
        vec![Action::Reduce(RuleId(1))],
        vec![Action::Error],
    ]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows[0].len(), 3);
}

#[test]
fn cb_v9_action_empty_cells_decompress_to_error() {
    let table = vec![vec![vec![]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
}

#[test]
fn cb_v9_action_large_state_id_preserved() {
    let table = vec![vec![vec![Action::Shift(StateId(0x7FFF))]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(0x7FFF))
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. Goto table compression → valid
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_goto_single_entry_roundtrip() {
    let mut table = vec![vec![None; 5]; 5];
    table[2][3] = Some(StateId(42));
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 2, 3), Some(StateId(42)));
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
}

#[test]
fn cb_v9_goto_diagonal_roundtrip() {
    let mut table = vec![vec![None; 4]; 4];
    for (i, row) in table.iter_mut().enumerate().take(4) {
        row[i] = Some(StateId(i as u16));
    }
    let compressed = compress_goto_table(&table);
    for i in 0..4 {
        assert_eq!(decompress_goto(&compressed, i, i), Some(StateId(i as u16)));
    }
}

#[test]
fn cb_v9_goto_dense_preserves_all() {
    let table: Vec<Vec<Option<StateId>>> = (0..3)
        .map(|s| (0..3).map(|c| Some(StateId((s * 3 + c) as u16))).collect())
        .collect();
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 9);
    for s in 0..3 {
        for c in 0..3 {
            assert_eq!(
                decompress_goto(&compressed, s, c),
                Some(StateId((s * 3 + c) as u16))
            );
        }
    }
}

#[test]
fn cb_v9_goto_1x1_roundtrip() {
    let table = vec![vec![Some(StateId(77))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(77)));
}

#[test]
fn cb_v9_goto_large_state_id() {
    let table = vec![vec![Some(StateId(0xFFFE))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(0xFFFE)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional boundary tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cb_v9_left_recursive_compress_succeeds() {
    let mut g = cb_v9_left_recursive();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_left_recursive_validate_passes() {
    let mut g = cb_v9_left_recursive();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_right_recursive_compress_succeeds() {
    let mut g = cb_v9_right_recursive();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_right_recursive_validate_passes() {
    let mut g = cb_v9_right_recursive();
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_different_grammars_different_compressed() {
    let mut g_min = cb_v9_minimal();
    let (_, c_min) = cb_v9_compress_pipeline(&mut g_min);
    let mut g_arith = cb_v9_arithmetic();
    let (_, c_arith) = cb_v9_compress_pipeline(&mut g_arith);
    assert_ne!(
        c_min.action_table.data.len(),
        c_arith.action_table.data.len()
    );
}

#[test]
fn cb_v9_cpt_from_parse_table_symbol_count() {
    let mut g = cb_v9_n_token(6);
    let (pt, _) = cb_v9_compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

#[test]
fn cb_v9_cpt_from_parse_table_state_count() {
    let mut g = cb_v9_n_token(6);
    let (pt, _) = cb_v9_compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
}

#[test]
fn cb_v9_row_offsets_bounded_by_data_len() {
    let mut g = cb_v9_arithmetic();
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    let data_len = compressed.action_table.data.len();
    for &offset in &compressed.action_table.row_offsets {
        assert!(usize::from(offset) <= data_len);
    }
    let goto_data_len = compressed.goto_table.data.len();
    for &offset in &compressed.goto_table.row_offsets {
        assert!(usize::from(offset) <= goto_data_len);
    }
}

#[test]
fn cb_v9_20_token_compress_succeeds() {
    let mut g = cb_v9_n_token(20);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cb_v9_20_token_validate_passes() {
    let mut g = cb_v9_n_token(20);
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    compressed.validate(&pt).unwrap();
}

#[test]
fn cb_v9_bitpack_empty_table_no_panic() {
    let table: Vec<Vec<Action>> = vec![];
    let _packed = BitPackedActionTable::from_table(&table);
}

#[test]
fn cb_v9_action_dedup_preserves_order() {
    let a = vec![vec![Action::Shift(StateId(0))]];
    let b = vec![vec![Action::Reduce(RuleId(0))]];
    let table = vec![a.clone(), b.clone(), a, b];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.state_to_row, vec![0, 1, 0, 1]);
}

#[test]
fn cb_v9_goto_offsets_match_state_count_pipeline() {
    let mut g = cb_v9_chain(4);
    let (pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert_eq!(compressed.goto_table.row_offsets.len(), pt.state_count + 1);
}

#[test]
fn cb_v9_chain_10_compress_succeeds() {
    let mut g = cb_v9_chain(10);
    let (_pt, compressed) = cb_v9_compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cb_v9_bitpack_all_reduce_row() {
    let table = vec![
        (0..4)
            .map(|i| Action::Reduce(RuleId(i)))
            .collect::<Vec<_>>(),
    ];
    let packed = BitPackedActionTable::from_table(&table);
    for col in 0..4 {
        assert_eq!(
            packed.decompress(0, col),
            Action::Reduce(RuleId(col as u16))
        );
    }
}

#[test]
fn cb_v9_rejects_action_symbol_id_over_u16() {
    let compressor = TableCompressor::new();
    let action_table = vec![vec![vec![Action::Shift(StateId(1))]]];
    let symbol_to_index = std::collections::BTreeMap::from([(adze_ir::SymbolId(1), 70000usize)]);

    let err = compressor
        .compress_action_table_small(&action_table, &symbol_to_index)
        .expect_err("compression should reject symbol ids above u16 width");

    let msg = err.to_string();
    assert!(msg.contains("symbol id 70000 exceeds u16::MAX"), "{msg}");
}

#[test]
fn cb_v9_rejects_action_row_offset_over_u16() {
    let compressor = TableCompressor::new();
    let mut action_row = Vec::with_capacity(65536);
    for _ in 0..65536 {
        action_row.push(vec![Action::Shift(StateId(1))]);
    }
    let action_table = vec![action_row];
    let symbol_to_index = std::collections::BTreeMap::new();

    let err = compressor
        .compress_action_table_small(&action_table, &symbol_to_index)
        .expect_err("compression should reject row_offsets above u16 width");

    let msg = err.to_string();
    assert!(msg.contains("row offset 65536 exceeds u16::MAX"), "{msg}");
}
