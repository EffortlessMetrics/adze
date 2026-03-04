//! Comprehensive tests for TableCompressor in adze-tablegen
//!
//! Tests the full compression pipeline including:
//! - Empty and minimal tables
//! - Various action types (shifts, reduces, accept, error)
//! - Goto table compression with run-length encoding
//! - Row offset validation
//! - Deterministic compression
//! - Large table handling
//! - CompressedActionEntry and CompressedGotoEntry creation
//! - Compression semantics preservation

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, StateId, SymbolId};
use adze_tablegen::compress::{
    CompressedActionEntry, CompressedGotoEntry, CompressedTables, TableCompressor,
};
use std::collections::BTreeMap;

// ============================================================================
// Helper Functions
// ============================================================================

/// Build a simple grammar: S → a
fn build_simple_grammar() -> ParseTable {
    let mut grammar = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("Failed to compute first/follow");
    build_lr1_automaton(&grammar, &ff).expect("Failed to build automaton")
}

/// Build a grammar with multiple rules: S → A | B; A → a; B → b
fn build_multi_rule_grammar() -> ParseTable {
    let mut grammar = GrammarBuilder::new("multi_rule")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .start("start")
        .build();

    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("Failed to compute first/follow");
    build_lr1_automaton(&grammar, &ff).expect("Failed to build automaton")
}

/// Build a grammar with sequences: S → A b; A → a a
fn build_sequence_grammar() -> ParseTable {
    let mut grammar = GrammarBuilder::new("sequence")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["A", "b"])
        .rule("A", vec!["a", "a"])
        .start("start")
        .build();

    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("Failed to compute first/follow");
    build_lr1_automaton(&grammar, &ff).expect("Failed to build automaton")
}

/// Build a grammar with repetition: S → A; A → a A | ε
fn build_repetition_grammar() -> ParseTable {
    let mut grammar = GrammarBuilder::new("repetition")
        .token("a", "a")
        .rule("start", vec!["A"])
        .rule("A", vec!["a", "A"])
        .rule("A", vec![])
        .start("start")
        .build();

    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("Failed to compute first/follow");
    build_lr1_automaton(&grammar, &ff).expect("Failed to build automaton")
}

/// Get token indices from a parse table including EOF
fn get_token_indices(parse_table: &ParseTable) -> Vec<usize> {
    let mut indices: Vec<usize> = parse_table.symbol_to_index.values().copied().collect();
    indices.sort();
    indices.dedup();
    indices
}

// ============================================================================
// Tests: Basic Compression
// ============================================================================

#[test]
fn test_01_compressor_creation() {
    let compressor = TableCompressor::new();
    // Just verify it can be created successfully
    let _ = compressor;
}

#[test]
fn test_02_compress_simple_table() {
    let parse_table = build_simple_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let result = compressor.compress(&parse_table, &token_indices, false);
    assert!(result.is_ok(), "Failed to compress simple table");

    let compressed = result.unwrap();
    assert!(!compressed.action_table.row_offsets.is_empty());
    assert!(!compressed.goto_table.row_offsets.is_empty());
}

#[test]
fn test_03_compress_multi_rule_table() {
    let parse_table = build_multi_rule_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let result = compressor.compress(&parse_table, &token_indices, false);
    assert!(result.is_ok(), "Failed to compress multi-rule table");
}

#[test]
fn test_04_compress_sequence_table() {
    let parse_table = build_sequence_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let result = compressor.compress(&parse_table, &token_indices, false);
    assert!(result.is_ok(), "Failed to compress sequence table");
}

#[test]
fn test_05_compress_with_nullable_start() {
    let parse_table = build_repetition_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    // start_can_be_empty = true because A can derive ε
    let result = compressor.compress(&parse_table, &token_indices, true);
    assert!(result.is_ok(), "Failed to compress with nullable start");
}

// ============================================================================
// Tests: Action Table Compression
// ============================================================================

#[test]
fn test_06_action_table_empty() {
    let compressor = TableCompressor::new();
    let action_table = vec![vec![vec![]; 5]; 3]; // 3 states, 5 symbols, all empty
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 4); // 3 states + 1 sentinel
    assert_eq!(compressed.default_actions.len(), 3);
    assert!(compressed.data.is_empty());
}

#[test]
fn test_07_action_table_with_shifts() {
    let compressor = TableCompressor::new();
    // State 0: shift(1) on symbol 0, nothing on symbol 1
    let action_table = vec![vec![vec![Action::Shift(StateId(1))], vec![]]];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 2); // 1 state + 1 sentinel
    assert!(
        compressed
            .data
            .iter()
            .any(|entry| { matches!(entry.action, Action::Shift(StateId(1))) })
    );
}

#[test]
fn test_08_action_table_with_reduces() {
    let compressor = TableCompressor::new();
    // State 0: reduce(1) on symbol 0, reduce(2) on symbol 1
    let action_table = vec![vec![
        vec![Action::Reduce(RuleId(1))],
        vec![Action::Reduce(RuleId(2))],
    ]];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.data.len(), 2);
    assert!(
        compressed
            .data
            .iter()
            .any(|entry| matches!(entry.action, Action::Reduce(RuleId(1))))
    );
    assert!(
        compressed
            .data
            .iter()
            .any(|entry| matches!(entry.action, Action::Reduce(RuleId(2))))
    );
}

#[test]
fn test_09_action_table_with_accept() {
    let compressor = TableCompressor::new();
    let action_table = vec![vec![vec![Action::Accept], vec![]]];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert!(
        compressed
            .data
            .iter()
            .any(|entry| matches!(entry.action, Action::Accept))
    );
}

#[test]
fn test_10_action_table_with_mixed_actions() {
    let compressor = TableCompressor::new();
    // Mix of shifts, reduces, and accept
    let action_table = vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(1))],
        vec![Action::Accept],
    ]];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.data.len(), 3);
}

#[test]
fn test_11_action_table_row_offsets_validity() {
    let compressor = TableCompressor::new();
    // Multiple states with varying action counts
    let action_table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![]], // 1 action
        vec![vec![Action::Reduce(RuleId(1))], vec![]], // 1 action
        vec![vec![Action::Accept], vec![Action::Reduce(RuleId(2))]], // 2 actions
    ];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    // Row offsets: [0, 1, 2, 4]
    assert_eq!(compressed.row_offsets.len(), 4);
    assert_eq!(compressed.row_offsets[0], 0);
    assert!(compressed.row_offsets[1] <= compressed.row_offsets[2]);
    assert!(compressed.row_offsets[2] <= compressed.row_offsets[3]);
}

#[test]
fn test_12_action_table_default_actions() {
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![vec![Action::Reduce(RuleId(1))]; 10],
        vec![vec![Action::Shift(StateId(2))]; 10],
    ];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    // Default action optimization is disabled, so defaults should be Error
    assert_eq!(compressed.default_actions.len(), 2);
    for default_action in &compressed.default_actions {
        assert_eq!(*default_action, Action::Error);
    }
}

// ============================================================================
// Tests: Goto Table Compression
// ============================================================================

#[test]
fn test_13_goto_table_empty() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(0xFFFF); 5]; 3]; // 3 states, 5 symbols, all invalid

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 4); // 3 states + 1 sentinel
}

#[test]
fn test_14_goto_table_single_entries() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(1), StateId(2), StateId(3)]];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.data.len(), 3);
    assert!(
        compressed
            .data
            .iter()
            .all(|entry| matches!(entry, CompressedGotoEntry::Single(_)))
    );
}

#[test]
fn test_15_goto_table_run_length_long() {
    let compressor = TableCompressor::new();
    // Run of 5 StateId(1)s should trigger run-length encoding (count > 2)
    let goto_table = vec![vec![
        StateId(1),
        StateId(1),
        StateId(1),
        StateId(1),
        StateId(1),
    ]];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert!(
        compressed.data.iter().any(|entry| {
            matches!(entry, CompressedGotoEntry::RunLength { state: 1, count: 5 })
        })
    );
}

#[test]
fn test_16_goto_table_run_length_short() {
    let compressor = TableCompressor::new();
    // Runs of 1-2 should use individual Single entries, not RunLength
    let goto_table = vec![vec![StateId(1), StateId(1), StateId(2)]];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    // Should have two single entries for the pair of 1s, then one for 2
    assert_eq!(compressed.data.len(), 3);
    assert!(
        compressed
            .data
            .iter()
            .filter(|e| matches!(e, CompressedGotoEntry::Single(1)))
            .count()
            == 2
    );
}

#[test]
fn test_17_goto_table_mixed_runs() {
    let compressor = TableCompressor::new();
    // Mix of different runs
    let goto_table = vec![vec![
        StateId(1),
        StateId(1),
        StateId(2),
        StateId(2),
        StateId(2),
        StateId(2),
        StateId(3),
    ]];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    // Should have: 2 single(1), 1 runlength(2,4), 1 single(3)
    let single_1_count = compressed
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::Single(1)))
        .count();
    let runlength_2_count = compressed
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::RunLength { state: 2, count: 4 }))
        .count();
    let single_3_count = compressed
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::Single(3)))
        .count();

    assert_eq!(single_1_count, 2);
    assert_eq!(runlength_2_count, 1);
    assert_eq!(single_3_count, 1);
}

#[test]
fn test_18_goto_table_multiple_states() {
    let compressor = TableCompressor::new();
    let goto_table = vec![
        vec![StateId(1), StateId(2)],
        vec![StateId(3), StateId(3), StateId(3)],
        vec![StateId(4), StateId(4)],
    ];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 4); // 3 states + 1 sentinel
    assert!(compressed.row_offsets[0] < compressed.row_offsets[1]);
    assert!(compressed.row_offsets[1] < compressed.row_offsets[2]);
    assert!(compressed.row_offsets[2] < compressed.row_offsets[3]);
}

// ============================================================================
// Tests: Row Offsets Correctness
// ============================================================================

#[test]
fn test_19_row_offsets_strictly_increasing() {
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![]],
        vec![
            vec![Action::Reduce(RuleId(1))],
            vec![Action::Reduce(RuleId(2))],
        ],
        vec![vec![], vec![], vec![]],
    ];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    for i in 1..compressed.row_offsets.len() {
        assert!(
            compressed.row_offsets[i] >= compressed.row_offsets[i - 1],
            "Row offsets not monotonically increasing at index {}",
            i
        );
    }
}

#[test]
fn test_20_row_offsets_length_matches_state_count() {
    let compressor = TableCompressor::new();
    let num_states = 5;
    let action_table = vec![vec![vec![Action::Shift(StateId(1))]; 3]; num_states];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(
        compressed.row_offsets.len(),
        num_states + 1,
        "Row offsets should have (state_count + 1) entries"
    );
}

#[test]
fn test_21_goto_row_offsets_length_matches_state_count() {
    let compressor = TableCompressor::new();
    let num_states = 4;
    let goto_table = vec![vec![StateId(1), StateId(2)]; num_states];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), num_states + 1);
}

// ============================================================================
// Tests: CompressedActionEntry
// ============================================================================

#[test]
fn test_22_compressed_action_entry_shift() {
    let entry = CompressedActionEntry::new(42, Action::Shift(StateId(10)));
    assert_eq!(entry.symbol, 42);
    match entry.action {
        Action::Shift(StateId(10)) => {}
        _ => panic!("Expected shift action with state 10"),
    }
}

#[test]
fn test_23_compressed_action_entry_reduce() {
    let entry = CompressedActionEntry::new(5, Action::Reduce(RuleId(3)));
    assert_eq!(entry.symbol, 5);
    match entry.action {
        Action::Reduce(RuleId(3)) => {}
        _ => panic!("Expected reduce action with rule 3"),
    }
}

#[test]
fn test_24_compressed_action_entry_accept() {
    let entry = CompressedActionEntry::new(1, Action::Accept);
    assert_eq!(entry.symbol, 1);
    assert_eq!(entry.action, Action::Accept);
}

#[test]
fn test_25_compressed_action_entry_error() {
    let entry = CompressedActionEntry::new(0, Action::Error);
    assert_eq!(entry.symbol, 0);
    assert_eq!(entry.action, Action::Error);
}

// ============================================================================
// Tests: CompressedGotoEntry
// ============================================================================

#[test]
fn test_26_compressed_goto_entry_single() {
    let entry = CompressedGotoEntry::Single(42);
    match entry {
        CompressedGotoEntry::Single(state) => assert_eq!(state, 42),
        _ => panic!("Expected Single variant"),
    }
}

#[test]
fn test_27_compressed_goto_entry_run_length() {
    let entry = CompressedGotoEntry::RunLength {
        state: 5,
        count: 10,
    };
    match entry {
        CompressedGotoEntry::RunLength { state, count } => {
            assert_eq!(state, 5);
            assert_eq!(count, 10);
        }
        _ => panic!("Expected RunLength variant"),
    }
}

#[test]
fn test_28_compressed_goto_entry_clone() {
    let entry1 = CompressedGotoEntry::RunLength { state: 7, count: 3 };
    let entry2 = entry1.clone();

    match (entry1, entry2) {
        (
            CompressedGotoEntry::RunLength {
                state: s1,
                count: c1,
            },
            CompressedGotoEntry::RunLength {
                state: s2,
                count: c2,
            },
        ) => {
            assert_eq!(s1, s2);
            assert_eq!(c1, c2);
        }
        _ => panic!("Expected both to be RunLength"),
    }
}

// ============================================================================
// Tests: CompressedTables Field Validation
// ============================================================================

#[test]
fn test_29_compressed_tables_fields_present() {
    let parse_table = build_simple_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    // Verify all fields are present
    assert!(!compressed.action_table.data.is_empty() || parse_table.action_table.is_empty());
    assert!(!compressed.action_table.row_offsets.is_empty());
    assert!(!compressed.action_table.default_actions.is_empty());
    assert!(!compressed.goto_table.row_offsets.is_empty());
    assert_eq!(
        compressed.small_table_threshold, 32768,
        "Threshold should match compressor"
    );
}

#[test]
fn test_30_compressed_tables_validate_ok() {
    let parse_table = build_simple_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    let result = compressed.validate(&parse_table);
    assert!(result.is_ok(), "Validation should succeed");
}

// ============================================================================
// Tests: Deterministic Compression
// ============================================================================

#[test]
fn test_31_deterministic_compression_simple() {
    let parse_table = build_simple_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let result1 = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();
    let result2 = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    // Same inputs should produce same row_offsets
    assert_eq!(
        result1.action_table.row_offsets, result2.action_table.row_offsets,
        "Action row offsets should be deterministic"
    );
    assert_eq!(
        result1.goto_table.row_offsets, result2.goto_table.row_offsets,
        "Goto row offsets should be deterministic"
    );
}

#[test]
fn test_32_deterministic_compression_multi_rule() {
    let parse_table = build_multi_rule_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let result1 = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();
    let result2 = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    assert_eq!(
        result1.action_table.data.len(),
        result2.action_table.data.len(),
        "Compressed action data should be deterministic"
    );
    assert_eq!(
        result1.goto_table.data.len(),
        result2.goto_table.data.len(),
        "Compressed goto data should be deterministic"
    );
}

// ============================================================================
// Tests: Large Table Compression
// ============================================================================

#[test]
fn test_33_compress_large_action_table() {
    let compressor = TableCompressor::new();
    // Create a table with 100 states and 50 symbols
    let action_table = vec![vec![vec![Action::Shift(StateId(1))]; 50]; 100];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 101); // 100 + 1
    assert_eq!(compressed.default_actions.len(), 100);
}

#[test]
fn test_34_compress_large_goto_table() {
    let compressor = TableCompressor::new();
    // Create a table with 100 states and 50 symbols
    let goto_table = vec![vec![StateId(1); 50]; 100];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 101); // 100 + 1
}

// ============================================================================
// Tests: Compression Preserves Semantics
// ============================================================================

#[test]
fn test_35_compress_action_semantics_shifts() {
    let parse_table = build_simple_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    // Verify that shift actions are preserved
    let has_shift = compressed
        .action_table
        .data
        .iter()
        .any(|entry| matches!(entry.action, Action::Shift(_)));
    assert!(has_shift, "Compression should preserve shift actions");
}

#[test]
fn test_36_compress_action_semantics_reduces() {
    let parse_table = build_multi_rule_grammar();
    let compressor = TableCompressor::new();
    let token_indices = get_token_indices(&parse_table);

    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    // Verify that reduce actions are preserved
    let has_reduce = compressed
        .action_table
        .data
        .iter()
        .any(|entry| matches!(entry.action, Action::Reduce(_)));
    assert!(has_reduce, "Compression should preserve reduce actions");
}

#[test]
fn test_37_compress_goto_semantics_states() {
    let parse_table = build_sequence_grammar();
    let compressor = TableCompressor::new();

    let compressed = compressor
        .compress_goto_table_small(&parse_table.goto_table)
        .unwrap();

    // Verify that goto states are preserved (not all invalid)
    let has_valid_state = compressed.data.iter().any(|entry| match entry {
        CompressedGotoEntry::Single(state) => *state != 0xFFFF,
        CompressedGotoEntry::RunLength { state, count: _ } => *state != 0xFFFF,
    });
    assert!(
        has_valid_state,
        "Compression should preserve valid goto states"
    );
}

// ============================================================================
// Tests: Edge Cases and Boundary Conditions
// ============================================================================

#[test]
fn test_38_action_table_single_state() {
    let compressor = TableCompressor::new();
    let action_table = vec![vec![vec![Action::Accept]; 5]];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 2); // 1 + 1
}

#[test]
fn test_39_action_table_single_symbol() {
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Reduce(RuleId(1))]],
    ];
    let symbol_to_index = BTreeMap::new();

    let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    assert_eq!(compressed.data.len(), 2);
}

#[test]
fn test_40_goto_table_all_same_state() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(42); 100]];

    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());

    let compressed = result.unwrap();
    // Should have a run-length entry for 100 copies of state 42
    assert!(compressed.data.iter().any(|entry| {
        matches!(
            entry,
            CompressedGotoEntry::RunLength {
                state: 42,
                count: 100
            }
        )
    }));
}

#[test]
fn test_41_encode_action_small_shift() {
    let compressor = TableCompressor::new();
    let action = Action::Shift(StateId(100));
    let result = compressor.encode_action_small(&action);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
}

#[test]
fn test_42_encode_action_small_reduce() {
    let compressor = TableCompressor::new();
    let action = Action::Reduce(RuleId(50));
    let result = compressor.encode_action_small(&action);

    assert!(result.is_ok());
    // Reduce encoding: 0x8000 | (rule_id + 1)
    assert_eq!(result.unwrap(), 0x8000 | 51);
}

#[test]
fn test_43_encode_action_small_accept() {
    let compressor = TableCompressor::new();
    let action = Action::Accept;
    let result = compressor.encode_action_small(&action);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0xFFFF);
}

#[test]
fn test_44_encode_action_small_error() {
    let compressor = TableCompressor::new();
    let action = Action::Error;
    let result = compressor.encode_action_small(&action);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0xFFFE);
}
