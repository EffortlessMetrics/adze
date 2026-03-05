//! Compression v5 tests for adze-tablegen.
//!
//! 60 tests covering:
//! 1. Action table compression roundtrip (10 tests)
//! 2. Goto table compression roundtrip (8 tests)
//! 3. BitPacked action table roundtrip (10 tests)
//! 4. Size reduction properties (8 tests)
//! 5. Various table shapes via real grammars (10 tests)
//! 6. TableCompressor pipeline integration (6 tests)
//! 7. Edge cases (8 tests)

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};
use adze_tablegen::compress::{CompressedTables, TableCompressor};
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

fn compress_pipeline(grammar: &Grammar) -> CompressedTables {
    let pt = build_table(grammar);
    let ti = collect_token_indices(grammar, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    TableCompressor::new().compress(&pt, &ti, sce).unwrap()
}

/// Convert single-action rows into GLR multi-action cells.
#[allow(dead_code)]
fn to_glr_cells(rows: Vec<Vec<Action>>) -> Vec<Vec<Vec<Action>>> {
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

fn diamond_grammar() -> Grammar {
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

fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .rule("A", vec![])
        .start("S")
        .build()
}

// ============================================================================
// Section 1: Action table compression roundtrip (10 tests)
// ============================================================================

#[test]
fn v5_action_01_all_error_roundtrip() {
    let table = vec![
        vec![vec![], vec![]],
        vec![vec![], vec![]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
    assert_eq!(decompress_action(&compressed, 1, 0), Action::Error);
    assert_eq!(decompress_action(&compressed, 1, 1), Action::Error);
}

#[test]
fn v5_action_02_shift_roundtrip() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Shift(StateId(2))]],
        vec![vec![Action::Error], vec![Action::Shift(StateId(3))]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Shift(StateId(1)));
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Shift(StateId(2)));
    assert_eq!(decompress_action(&compressed, 1, 0), Action::Error);
    assert_eq!(decompress_action(&compressed, 1, 1), Action::Shift(StateId(3)));
}

#[test]
fn v5_action_03_reduce_roundtrip() {
    let table = vec![
        vec![vec![Action::Reduce(RuleId(0))], vec![Action::Reduce(RuleId(1))]],
        vec![vec![Action::Reduce(RuleId(2))], vec![Action::Error]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Reduce(RuleId(0)));
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Reduce(RuleId(1)));
    assert_eq!(decompress_action(&compressed, 1, 0), Action::Reduce(RuleId(2)));
    assert_eq!(decompress_action(&compressed, 1, 1), Action::Error);
}

#[test]
fn v5_action_04_accept_roundtrip() {
    let table = vec![
        vec![vec![Action::Accept], vec![Action::Error]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
}

#[test]
fn v5_action_05_mixed_actions_roundtrip() {
    let table = vec![
        vec![
            vec![Action::Shift(StateId(5))],
            vec![Action::Reduce(RuleId(3))],
            vec![Action::Accept],
        ],
        vec![
            vec![Action::Error],
            vec![Action::Shift(StateId(7))],
            vec![Action::Reduce(RuleId(0))],
        ],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Shift(StateId(5)));
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Reduce(RuleId(3)));
    assert_eq!(decompress_action(&compressed, 0, 2), Action::Accept);
    assert_eq!(decompress_action(&compressed, 1, 0), Action::Error);
    assert_eq!(decompress_action(&compressed, 1, 1), Action::Shift(StateId(7)));
    assert_eq!(decompress_action(&compressed, 1, 2), Action::Reduce(RuleId(0)));
}

#[test]
fn v5_action_06_duplicate_rows_deduplicated() {
    let row = vec![vec![Action::Shift(StateId(1))], vec![Action::Error]];
    let table = vec![row.clone(), row.clone(), row];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 3);
    // All states map to the same unique row
    assert_eq!(compressed.state_to_row[0], compressed.state_to_row[1]);
    assert_eq!(compressed.state_to_row[1], compressed.state_to_row[2]);
}

#[test]
fn v5_action_07_no_duplicates_preserves_all_rows() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Error]],
        vec![vec![Action::Error], vec![Action::Shift(StateId(2))]],
        vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 3);
}

#[test]
fn v5_action_08_glr_multi_action_cell() {
    let table = vec![
        vec![
            vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
            vec![Action::Error],
        ],
    ];
    let compressed = compress_action_table(&table);
    // decompress_action returns first action from the cell
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Shift(StateId(1)));
}

#[test]
fn v5_action_09_single_cell_table() {
    let table = vec![vec![vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 1);
}

#[test]
fn v5_action_10_recover_action_roundtrip() {
    let table = vec![
        vec![vec![Action::Recover], vec![Action::Shift(StateId(1))]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Recover);
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Shift(StateId(1)));
}

// ============================================================================
// Section 2: Goto table compression roundtrip (8 tests)
// ============================================================================

#[test]
fn v5_goto_01_all_none_roundtrip() {
    let table: Vec<Vec<Option<StateId>>> = vec![
        vec![None, None],
        vec![None, None],
    ];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 1, 1), None);
}

#[test]
fn v5_goto_02_sparse_roundtrip() {
    let table = vec![
        vec![None, Some(StateId(1)), None],
        vec![Some(StateId(2)), None, None],
        vec![None, None, Some(StateId(3))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(1)));
    assert_eq!(decompress_goto(&compressed, 1, 0), Some(StateId(2)));
    assert_eq!(decompress_goto(&compressed, 2, 2), Some(StateId(3)));
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 2, 0), None);
}

#[test]
fn v5_goto_03_fully_populated_roundtrip() {
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
fn v5_goto_04_single_entry_roundtrip() {
    let table = vec![
        vec![None, None, None],
        vec![None, Some(StateId(42)), None],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(42)));
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
}

#[test]
fn v5_goto_05_large_state_ids() {
    let table = vec![
        vec![Some(StateId(1000)), None],
        vec![None, Some(StateId(65534))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(1000)));
    assert_eq!(decompress_goto(&compressed, 1, 1), Some(StateId(65534)));
}

#[test]
fn v5_goto_06_single_row_roundtrip() {
    let table = vec![
        vec![Some(StateId(10)), None, Some(StateId(20))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 2);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(10)));
    assert_eq!(decompress_goto(&compressed, 0, 1), None);
    assert_eq!(decompress_goto(&compressed, 0, 2), Some(StateId(20)));
}

#[test]
fn v5_goto_07_diagonal_pattern() {
    let table = vec![
        vec![Some(StateId(0)), None, None, None],
        vec![None, Some(StateId(1)), None, None],
        vec![None, None, Some(StateId(2)), None],
        vec![None, None, None, Some(StateId(3))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
    for i in 0..4 {
        assert_eq!(decompress_goto(&compressed, i, i), Some(StateId(i as u16)));
        if i + 1 < 4 {
            assert_eq!(decompress_goto(&compressed, i, i + 1), None);
        }
    }
}

#[test]
fn v5_goto_08_first_column_populated() {
    let table = vec![
        vec![Some(StateId(1)), None, None],
        vec![Some(StateId(2)), None, None],
        vec![Some(StateId(3)), None, None],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
    for i in 0..3 {
        assert_eq!(
            decompress_goto(&compressed, i, 0),
            Some(StateId((i + 1) as u16))
        );
        assert_eq!(decompress_goto(&compressed, i, 1), None);
    }
}

// ============================================================================
// Section 3: BitPacked action table roundtrip (10 tests)
// ============================================================================

#[test]
fn v5_bitpack_01_all_error() {
    let table = vec![
        vec![Action::Error, Action::Error],
        vec![Action::Error, Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(1, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Error);
}

#[test]
fn v5_bitpack_02_all_shift() {
    let table = vec![
        vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(2)));
}

#[test]
fn v5_bitpack_03_all_reduce() {
    let table = vec![
        vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Reduce(RuleId(0)));
    assert_eq!(packed.decompress(0, 1), Action::Reduce(RuleId(1)));
}

#[test]
fn v5_bitpack_04_error_then_shift() {
    let table = vec![
        vec![Action::Error, Action::Shift(StateId(5))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(5)));
}

#[test]
fn v5_bitpack_05_shift_then_error() {
    let table = vec![
        vec![Action::Shift(StateId(3)), Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(3)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
}

#[test]
fn v5_bitpack_06_shift_then_reduce() {
    let table = vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Reduce(RuleId(0)));
}

#[test]
fn v5_bitpack_07_accept_roundtrip() {
    // Accept is encoded as special reduce with u32::MAX
    let table = vec![
        vec![Action::Shift(StateId(1)), Action::Accept],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Accept);
}

#[test]
fn v5_bitpack_08_fork_roundtrip() {
    let fork_actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let table = vec![
        vec![Action::Fork(fork_actions.clone()), Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Fork(fork_actions));
    assert_eq!(packed.decompress(0, 1), Action::Error);
}

#[test]
fn v5_bitpack_09_recover_treated_as_error() {
    let table = vec![
        vec![Action::Recover, Action::Shift(StateId(1))],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    // Recover is stored as error in bit-packed format
    assert_eq!(packed.decompress(0, 0), Action::Error);
    assert_eq!(packed.decompress(0, 1), Action::Shift(StateId(1)));
}

#[test]
fn v5_bitpack_10_multi_row_mixed() {
    let table = vec![
        vec![Action::Shift(StateId(1)), Action::Error, Action::Shift(StateId(2))],
        vec![Action::Error, Action::Reduce(RuleId(0)), Action::Error],
    ];
    let packed = BitPackedActionTable::from_table(&table);
    assert_eq!(packed.decompress(0, 0), Action::Shift(StateId(1)));
    assert_eq!(packed.decompress(0, 1), Action::Error);
    assert_eq!(packed.decompress(0, 2), Action::Shift(StateId(2)));
    assert_eq!(packed.decompress(1, 0), Action::Error);
    assert_eq!(packed.decompress(1, 1), Action::Reduce(RuleId(0)));
    assert_eq!(packed.decompress(1, 2), Action::Error);
}

// ============================================================================
// Section 4: Size reduction properties (8 tests)
// ============================================================================

#[test]
fn v5_size_01_duplicate_rows_reduce_unique_count() {
    let row = vec![vec![Action::Shift(StateId(1))], vec![Action::Error]];
    let table = vec![row.clone(), row.clone(), row.clone(), row.clone(), row];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert!(compressed.unique_rows.len() < table.len());
}

#[test]
fn v5_size_02_sparse_goto_fewer_entries_than_cells() {
    let table = vec![
        vec![None, None, None, None, None],
        vec![None, Some(StateId(1)), None, None, None],
        vec![None, None, None, None, None],
    ];
    let compressed = compress_goto_table(&table);
    let total_cells = 3 * 5;
    assert!(compressed.entries.len() < total_cells);
    assert_eq!(compressed.entries.len(), 1);
}

#[test]
fn v5_size_03_all_same_rows_maximal_dedup() {
    let row = vec![
        vec![Action::Reduce(RuleId(0))],
        vec![Action::Reduce(RuleId(0))],
        vec![Action::Reduce(RuleId(0))],
    ];
    let table = vec![row.clone(); 20];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 20);
}

#[test]
fn v5_size_04_all_unique_rows_no_dedup() {
    let table: Vec<Vec<Vec<Action>>> = (0..5)
        .map(|i| {
            vec![vec![Action::Shift(StateId(i))], vec![Action::Error]]
        })
        .collect();
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 5);
}

#[test]
fn v5_size_05_half_duplicate_rows() {
    let row_a = vec![vec![Action::Shift(StateId(1))], vec![Action::Error]];
    let row_b = vec![vec![Action::Error], vec![Action::Reduce(RuleId(0))]];
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
fn v5_size_06_empty_goto_zero_entries() {
    let table: Vec<Vec<Option<StateId>>> = vec![
        vec![None; 10],
        vec![None; 10],
        vec![None; 10],
    ];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn v5_size_07_dense_goto_entry_count_equals_populated() {
    let table = vec![
        vec![Some(StateId(1)), Some(StateId(2)), None],
        vec![None, Some(StateId(3)), Some(StateId(4))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
}

#[test]
fn v5_size_08_action_dedup_ratio_with_pattern() {
    // Create a table where every even row is the same and every odd row is the same
    let row_even = vec![vec![Action::Shift(StateId(0))], vec![Action::Error]];
    let row_odd = vec![vec![Action::Error], vec![Action::Reduce(RuleId(1))]];
    let table: Vec<Vec<Vec<Action>>> = (0..100)
        .map(|i| if i % 2 == 0 { row_even.clone() } else { row_odd.clone() })
        .collect();
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
    assert_eq!(compressed.state_to_row.len(), 100);
}

// ============================================================================
// Section 5: Various table shapes via real grammars (10 tests)
// ============================================================================

#[test]
fn v5_grammar_01_single_token_pipeline_valid() {
    let ct = compress_pipeline(&single_token_grammar());
    assert!(!ct.action_table.data.is_empty());
    assert!(!ct.action_table.row_offsets.is_empty());
}

#[test]
fn v5_grammar_02_two_token_pipeline_valid() {
    let ct = compress_pipeline(&two_token_grammar());
    assert!(!ct.action_table.data.is_empty());
    assert!(!ct.action_table.default_actions.is_empty());
}

#[test]
fn v5_grammar_03_alternatives_pipeline_valid() {
    let ct = compress_pipeline(&alternatives_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v5_grammar_04_nested_pipeline_valid() {
    let ct = compress_pipeline(&nested_grammar());
    assert!(!ct.action_table.data.is_empty());
    assert!(!ct.goto_table.data.is_empty());
}

#[test]
fn v5_grammar_05_deep_chain_pipeline_valid() {
    let ct = compress_pipeline(&deep_chain_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v5_grammar_06_left_recursive_pipeline_valid() {
    let ct = compress_pipeline(&left_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v5_grammar_07_right_recursive_pipeline_valid() {
    let ct = compress_pipeline(&right_recursive_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v5_grammar_08_precedence_pipeline_valid() {
    let ct = compress_pipeline(&precedence_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v5_grammar_09_wide_alternatives_pipeline_valid() {
    let ct = compress_pipeline(&wide_alternatives_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn v5_grammar_10_long_sequence_pipeline_valid() {
    let ct = compress_pipeline(&long_sequence_grammar());
    assert!(!ct.action_table.data.is_empty());
}

// ============================================================================
// Section 6: TableCompressor pipeline integration (6 tests)
// ============================================================================

#[test]
fn v5_pipeline_01_compressor_validates_single_token() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    assert!(ct.validate(&pt).is_ok());
}

#[test]
fn v5_pipeline_02_compressor_validates_alternatives() {
    let g = alternatives_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    assert!(ct.validate(&pt).is_ok());
}

#[test]
fn v5_pipeline_03_compressor_validates_nested() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    assert!(ct.validate(&pt).is_ok());
}

#[test]
fn v5_pipeline_04_compressor_validates_diamond() {
    let g = diamond_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    assert!(ct.validate(&pt).is_ok());
}

#[test]
fn v5_pipeline_05_compressor_validates_multi_level() {
    let g = multi_level_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    assert!(ct.validate(&pt).is_ok());
}

#[test]
fn v5_pipeline_06_compressor_validates_nullable() {
    let g = nullable_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    assert!(ct.validate(&pt).is_ok());
}

// ============================================================================
// Section 7: Edge cases (8 tests)
// ============================================================================

#[test]
fn v5_edge_01_empty_action_table() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let compressed = compress_action_table(&table);
    assert!(compressed.unique_rows.is_empty());
    assert!(compressed.state_to_row.is_empty());
}

#[test]
fn v5_edge_02_empty_goto_table() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
}

#[test]
fn v5_edge_03_single_state_single_symbol_action() {
    let table = vec![vec![vec![Action::Shift(StateId(0))]]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Shift(StateId(0)));
}

#[test]
fn v5_edge_04_single_state_single_symbol_goto() {
    let table = vec![vec![Some(StateId(99))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(99)));
}

#[test]
fn v5_edge_05_state_id_zero_preserved() {
    let table = vec![
        vec![None, Some(StateId(0))],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), None);
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(0)));
}

#[test]
fn v5_edge_06_large_state_count_dedup() {
    let row = vec![vec![Action::Error], vec![Action::Shift(StateId(1))]];
    let table = vec![row; 200];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row.len(), 200);
}

#[test]
fn v5_edge_07_wide_row_roundtrip() {
    let wide_row: Vec<Vec<Action>> = (0..50)
        .map(|i| {
            if i % 3 == 0 {
                vec![Action::Shift(StateId(i))]
            } else {
                vec![]
            }
        })
        .collect();
    let table = vec![wide_row];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Shift(StateId(0)));
    assert_eq!(decompress_action(&compressed, 0, 1), Action::Error);
    assert_eq!(decompress_action(&compressed, 0, 3), Action::Shift(StateId(3)));
}

#[test]
fn v5_edge_08_bitpack_empty_table() {
    let table: Vec<Vec<Action>> = vec![];
    let packed = BitPackedActionTable::from_table(&table);
    // No panics on construction with empty table
    let _ = packed;
}
