// Property-based tests for the code generation / Tree-sitter encoding module
// (`compress.rs`: `TableCompressor`, `CompressedActionTable`, `CompressedGotoTable`).
//
// Properties verified:
//  1. encode_action_small roundtrips for Shift actions
//  2. encode_action_small roundtrips for Reduce actions
//  3. encode_action_small sentinel values are distinct
//  4. Shift encoding keeps high bit clear
//  5. Reduce encoding sets high bit
//  6. compress_action_table_small row_offsets length == states + 1
//  7. compress_action_table_small row_offsets are non-decreasing
//  8. compress_action_table_small last offset == data.len()
//  9. compress_action_table_small default_actions length == states
// 10. compress_action_table_small non-error actions are all encoded
// 11. compress_goto_table_small row_offsets length == states + 1
// 12. compress_goto_table_small row_offsets are non-decreasing
// 13. compress_goto_table_small total elements conserved
// 14. compress_goto_table_small run-length entries expand correctly
// 15. compress_goto_table_small is deterministic
// 16. encode_action_small rejects oversized shift states
// 17. encode_action_small rejects oversized reduce rules

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId, SymbolId};
use adze_tablegen::compress::{CompressedGotoEntry, TableCompressor};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn small_state_id() -> impl Strategy<Value = StateId> {
    (0u16..0x7FFF).prop_map(StateId)
}

fn small_rule_id() -> impl Strategy<Value = RuleId> {
    (0u16..0x3FFF).prop_map(RuleId)
}

/// Actions that are valid for small-table encoding (no Fork).
fn encodable_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        small_state_id().prop_map(Action::Shift),
        small_rule_id().prop_map(Action::Reduce),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Action cell (GLR: 0..3 actions per cell).
fn action_cell() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(encodable_action(), 0..=3)
}

/// Rectangular action table with a symbol_to_index map that covers all columns.
fn action_table_with_map(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = (Vec<Vec<Vec<Action>>>, BTreeMap<SymbolId, usize>)> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(action_cell(), symbols..=symbols),
            states..=states,
        )
        .prop_map(move |table| {
            let map: BTreeMap<SymbolId, usize> =
                (0..symbols).map(|i| (SymbolId(i as u16), i)).collect();
            (table, map)
        })
    })
}

/// Goto table: each row is a Vec<StateId>.
fn goto_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<StateId>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec((0u16..20).prop_map(StateId), symbols..=symbols),
            states..=states,
        )
    })
}

// ---------------------------------------------------------------------------
// 1. encode_action_small roundtrips for Shift actions
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn shift_encoding_roundtrip(state in 0u16..0x7FFF) {
        let compressor = TableCompressor::new();
        let action = Action::Shift(StateId(state));
        let encoded = compressor.encode_action_small(&action).unwrap();
        // Shift: encoded value == state id directly
        prop_assert_eq!(encoded, state, "Shift({}) encoded as {}", state, encoded);
    }
}

// ---------------------------------------------------------------------------
// 2. encode_action_small roundtrips for Reduce actions
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn reduce_encoding_roundtrip(rule in 0u16..0x3FFF) {
        let compressor = TableCompressor::new();
        let action = Action::Reduce(RuleId(rule));
        let encoded = compressor.encode_action_small(&action).unwrap();
        // Reduce: high bit set, lower bits = rule_id + 1 (1-based)
        let decoded_rule = (encoded & 0x7FFF) - 1;
        prop_assert_eq!(
            decoded_rule, rule,
            "Reduce({}) encoded as {:#06x}, decoded rule = {}",
            rule, encoded, decoded_rule
        );
    }
}

// ---------------------------------------------------------------------------
// 3. encode_action_small sentinel values are distinct
// ---------------------------------------------------------------------------

#[test]
fn sentinel_values_are_distinct() {
    let compressor = TableCompressor::new();
    let accept = compressor.encode_action_small(&Action::Accept).unwrap();
    let error = compressor.encode_action_small(&Action::Error).unwrap();
    let recover = compressor.encode_action_small(&Action::Recover).unwrap();

    assert_ne!(accept, error, "Accept and Error must differ");
    assert_ne!(accept, recover, "Accept and Recover must differ");
    assert_ne!(error, recover, "Error and Recover must differ");

    // Well-known values
    assert_eq!(accept, 0xFFFF);
    assert_eq!(error, 0xFFFE);
    assert_eq!(recover, 0xFFFD);
}

// ---------------------------------------------------------------------------
// 4. Shift encoding keeps high bit clear
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn shift_high_bit_clear(state in 0u16..0x7FFF) {
        let compressor = TableCompressor::new();
        let encoded = compressor
            .encode_action_small(&Action::Shift(StateId(state)))
            .unwrap();
        prop_assert_eq!(
            encoded & 0x8000, 0,
            "Shift({}) encoding {:#06x} has high bit set", state, encoded
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Reduce encoding sets high bit
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn reduce_high_bit_set(rule in 0u16..0x3FFF) {
        let compressor = TableCompressor::new();
        let encoded = compressor
            .encode_action_small(&Action::Reduce(RuleId(rule)))
            .unwrap();
        prop_assert_ne!(
            encoded & 0x8000, 0,
            "Reduce({}) encoding {:#06x} should have high bit set", rule, encoded
        );
    }
}

// ---------------------------------------------------------------------------
// 6. compress_action_table_small row_offsets length == states + 1
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn action_row_offsets_length(
        (table, map) in action_table_with_map(10, 8)
    ) {
        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &map)
            .unwrap();
        prop_assert_eq!(
            compressed.row_offsets.len(),
            table.len() + 1,
            "row_offsets.len() should be states + 1"
        );
    }
}

// ---------------------------------------------------------------------------
// 7. compress_action_table_small row_offsets are non-decreasing
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn action_row_offsets_non_decreasing(
        (table, map) in action_table_with_map(10, 8)
    ) {
        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &map)
            .unwrap();
        for w in compressed.row_offsets.windows(2) {
            prop_assert!(
                w[0] <= w[1],
                "row_offsets not non-decreasing: {} > {}", w[0], w[1]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 8. compress_action_table_small last offset == data.len()
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn action_last_offset_equals_data_len(
        (table, map) in action_table_with_map(10, 8)
    ) {
        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &map)
            .unwrap();
        prop_assert_eq!(
            *compressed.row_offsets.last().unwrap() as usize,
            compressed.data.len(),
            "Last row offset must equal data array length"
        );
    }
}

// ---------------------------------------------------------------------------
// 9. compress_action_table_small default_actions length == states
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn default_actions_length_matches_states(
        (table, map) in action_table_with_map(10, 8)
    ) {
        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &map)
            .unwrap();
        prop_assert_eq!(
            compressed.default_actions.len(),
            table.len(),
            "default_actions.len() should equal number of states"
        );
    }
}

// ---------------------------------------------------------------------------
// 10. compress_action_table_small: non-error actions are all encoded
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn all_non_error_actions_encoded(
        (table, map) in action_table_with_map(8, 6)
    ) {
        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&table, &map)
            .unwrap();

        // Count non-error actions in the original table
        let expected_non_error: usize = table
            .iter()
            .flat_map(|row| row.iter())
            .flat_map(|cell| cell.iter())
            .filter(|a| !matches!(a, Action::Error))
            .count();

        prop_assert_eq!(
            compressed.data.len(),
            expected_non_error,
            "Compressed data should contain exactly the non-error actions"
        );
    }
}

// ---------------------------------------------------------------------------
// 11. compress_goto_table_small row_offsets length == states + 1
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn goto_row_offsets_length(table in goto_table_strategy(10, 8)) {
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        prop_assert_eq!(
            compressed.row_offsets.len(),
            table.len() + 1,
            "goto row_offsets.len() should be states + 1"
        );
    }
}

// ---------------------------------------------------------------------------
// 12. compress_goto_table_small row_offsets are non-decreasing
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn goto_row_offsets_non_decreasing(table in goto_table_strategy(10, 8)) {
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();
        for w in compressed.row_offsets.windows(2) {
            prop_assert!(
                w[0] <= w[1],
                "goto row_offsets not non-decreasing: {} > {}", w[0], w[1]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 13. compress_goto_table_small total elements conserved
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn goto_total_elements_conserved(table in goto_table_strategy(8, 8)) {
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        // Expand all compressed entries and count total elements
        let expanded_count: u16 = compressed
            .data
            .iter()
            .map(|entry| match entry {
                CompressedGotoEntry::Single(_) => 1u16,
                CompressedGotoEntry::RunLength { count, .. } => *count,
            })
            .sum();

        let original_count: usize = table.iter().map(|row| row.len()).sum();

        prop_assert_eq!(
            expanded_count as usize, original_count,
            "Expanded goto entries ({}) must equal original element count ({})",
            expanded_count, original_count
        );
    }
}

// ---------------------------------------------------------------------------
// 14. compress_goto_table_small run-length entries expand correctly
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn goto_run_length_expansion(table in goto_table_strategy(8, 8)) {
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        // Expand all compressed data back to a flat list per row, then compare
        for (state_idx, row) in table.iter().enumerate() {
            let start = compressed.row_offsets[state_idx] as usize;
            let end = compressed.row_offsets[state_idx + 1] as usize;
            let entries = &compressed.data[start..end];

            let mut expanded: Vec<u16> = Vec::new();
            for entry in entries {
                match entry {
                    CompressedGotoEntry::Single(s) => expanded.push(*s),
                    CompressedGotoEntry::RunLength { state, count } => {
                        for _ in 0..*count {
                            expanded.push(*state);
                        }
                    }
                }
            }

            let original: Vec<u16> = row.iter().map(|s| s.0).collect();
            prop_assert_eq!(
                expanded, original,
                "State {} expansion mismatch", state_idx
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 15. compress_goto_table_small is deterministic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn goto_compression_deterministic(table in goto_table_strategy(8, 8)) {
        let compressor = TableCompressor::new();
        let c1 = compressor.compress_goto_table_small(&table).unwrap();
        let c2 = compressor.compress_goto_table_small(&table).unwrap();

        prop_assert_eq!(c1.row_offsets, c2.row_offsets, "Row offsets differ");
        prop_assert_eq!(c1.data.len(), c2.data.len(), "Data lengths differ");
        // Compare element-wise using Debug representation
        for (i, (a, b)) in c1.data.iter().zip(c2.data.iter()).enumerate() {
            prop_assert_eq!(
                format!("{a:?}"), format!("{b:?}"),
                "Data entry {} differs", i
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 16. encode_action_small rejects oversized shift states
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn shift_overflow_rejected(state in 0x8000u16..=u16::MAX) {
        let compressor = TableCompressor::new();
        let result = compressor.encode_action_small(&Action::Shift(StateId(state)));
        prop_assert!(
            result.is_err(),
            "Shift({}) should be rejected as too large", state
        );
    }
}

// ---------------------------------------------------------------------------
// 17. encode_action_small rejects oversized reduce rules
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn reduce_overflow_rejected(rule in 0x4000u16..=u16::MAX) {
        let compressor = TableCompressor::new();
        let result = compressor.encode_action_small(&Action::Reduce(RuleId(rule)));
        prop_assert!(
            result.is_err(),
            "Reduce({}) should be rejected as too large", rule
        );
    }
}

// ---------------------------------------------------------------------------
// 18. Shift and Reduce encodings never collide
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn shift_reduce_encodings_disjoint(
        state in 0u16..0x7FFF,
        rule in 0u16..0x3FFF,
    ) {
        let compressor = TableCompressor::new();
        let shift_enc = compressor
            .encode_action_small(&Action::Shift(StateId(state)))
            .unwrap();
        let reduce_enc = compressor
            .encode_action_small(&Action::Reduce(RuleId(rule)))
            .unwrap();
        prop_assert_ne!(
            shift_enc, reduce_enc,
            "Shift({}) and Reduce({}) must not encode to the same value",
            state, rule
        );
    }
}

// ---------------------------------------------------------------------------
// 19. compress_action_table_small is deterministic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn action_compression_deterministic(
        (table, map) in action_table_with_map(8, 6)
    ) {
        let compressor = TableCompressor::new();
        let c1 = compressor.compress_action_table_small(&table, &map).unwrap();
        let c2 = compressor.compress_action_table_small(&table, &map).unwrap();

        prop_assert_eq!(c1.row_offsets, c2.row_offsets);
        prop_assert_eq!(c1.default_actions, c2.default_actions);
        prop_assert_eq!(c1.data.len(), c2.data.len());
        for (i, (a, b)) in c1.data.iter().zip(c2.data.iter()).enumerate() {
            prop_assert_eq!(a.symbol, b.symbol, "Entry {} symbol differs", i);
            prop_assert_eq!(a.action.clone(), b.action.clone(), "Entry {} action differs", i);
        }
    }
}

// ---------------------------------------------------------------------------
// 20. Empty action table compresses without panic
// ---------------------------------------------------------------------------

#[test]
fn empty_action_table_compresses() {
    let compressor = TableCompressor::new();
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let map = BTreeMap::new();
    let result = compressor.compress_action_table_small(&table, &map);
    assert!(result.is_ok());
    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets, vec![0]);
    assert!(compressed.default_actions.is_empty());
    assert!(compressed.data.is_empty());
}

// ---------------------------------------------------------------------------
// 21. Empty goto table compresses without panic
// ---------------------------------------------------------------------------

#[test]
fn empty_goto_table_compresses() {
    let compressor = TableCompressor::new();
    let table: Vec<Vec<StateId>> = vec![];
    let result = compressor.compress_goto_table_small(&table);
    assert!(result.is_ok());
    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets, vec![0]);
    assert!(compressed.data.is_empty());
}

// ---------------------------------------------------------------------------
// 22. Uniform goto rows produce run-length entries
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn uniform_goto_row_uses_run_length(
        state_val in 0u16..20,
        width in 4usize..=16,
    ) {
        let compressor = TableCompressor::new();
        let table = vec![vec![StateId(state_val); width]];
        let compressed = compressor.compress_goto_table_small(&table).unwrap();

        // A uniform row of width >= 4 should have exactly one RunLength entry
        let start = compressed.row_offsets[0] as usize;
        let end = compressed.row_offsets[1] as usize;
        let entries = &compressed.data[start..end];

        prop_assert_eq!(
            entries.len(), 1,
            "Uniform row of width {} should compress to 1 RunLength entry, got {}",
            width, entries.len()
        );
        match &entries[0] {
            CompressedGotoEntry::RunLength { state, count } => {
                prop_assert_eq!(*state, state_val);
                prop_assert_eq!(*count, width as u16);
            }
            other => prop_assert!(false, "Expected RunLength, got {:?}", other),
        }
    }
}
