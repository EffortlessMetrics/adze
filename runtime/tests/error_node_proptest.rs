#![allow(clippy::needless_range_loop)]
//! Property-based tests for error node handling in the adze runtime.

use adze::error_recovery::{ErrorRecoveryConfig, ErrorRecoveryState, RecoveryStrategy};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_recovery_strategy() -> impl Strategy<Value = RecoveryStrategy> {
    prop_oneof![
        Just(RecoveryStrategy::PanicMode),
        Just(RecoveryStrategy::TokenInsertion),
        Just(RecoveryStrategy::TokenDeletion),
        Just(RecoveryStrategy::TokenSubstitution),
        Just(RecoveryStrategy::PhraseLevel),
        Just(RecoveryStrategy::ScopeRecovery),
        Just(RecoveryStrategy::IndentationRecovery),
    ]
}

fn arb_position() -> impl Strategy<Value = (usize, usize)> {
    (0usize..10_000, 0usize..500)
}

fn arb_byte_range() -> impl Strategy<Value = (usize, usize)> {
    (0usize..100_000).prop_flat_map(|start| (Just(start), start..start + 10_000))
}

fn arb_symbol_vec(max_len: usize) -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..1000, 0..max_len)
}

fn arb_optional_symbol() -> impl Strategy<Value = Option<u16>> {
    prop_oneof![Just(None), (0u16..1000).prop_map(Some)]
}

// ---------------------------------------------------------------------------
// Tests: Error node creation via record_error
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// record_error creates exactly one error node per call.
    #[test]
    fn creation_single_record(
        (start, end) in arb_byte_range(),
        strategy in arb_recovery_strategy(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        prop_assert!(state.get_error_nodes().is_empty());
        state.record_error(start, end, (0, 0), (0, 1), vec![], None, strategy, vec![]);
        prop_assert_eq!(state.get_error_nodes().len(), 1);
    }

    /// record_error with all-zero arguments produces valid node.
    #[test]
    fn creation_zero_args(_dummy in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 0, (0, 0), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
        prop_assert_eq!(nodes[0].start_byte, 0);
        prop_assert_eq!(nodes[0].end_byte, 0);
    }

    /// record_error with maximum-like byte values still produces node.
    #[test]
    fn creation_large_byte_offsets(
        start in 1_000_000usize..10_000_000,
    ) {
        let end = start + 1;
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(start, end, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes[0].start_byte, start);
        prop_assert_eq!(nodes[0].end_byte, end);
    }
}

// ---------------------------------------------------------------------------
// Tests: Error node position tracking
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Byte range (start_byte, end_byte) is preserved exactly.
    #[test]
    fn position_byte_range_preserved(
        (start, end) in arb_byte_range(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(start, end, (0, 0), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.start_byte, start);
        prop_assert_eq!(node.end_byte, end);
        prop_assert!(node.end_byte >= node.start_byte);
    }

    /// Row/column start_position is preserved.
    #[test]
    fn position_start_row_col_preserved(
        start_pos in arb_position(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, start_pos, (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.start_position, start_pos);
    }

    /// Row/column end_position is preserved.
    #[test]
    fn position_end_row_col_preserved(
        end_pos in arb_position(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), end_pos, vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.end_position, end_pos);
    }

    /// Both start and end positions are independently preserved.
    #[test]
    fn position_start_end_independent(
        (sb, eb) in arb_byte_range(),
        sp in arb_position(),
        ep in arb_position(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(sb, eb, sp, ep, vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.start_byte, sb);
        prop_assert_eq!(node.end_byte, eb);
        prop_assert_eq!(node.start_position, sp);
        prop_assert_eq!(node.end_position, ep);
    }
}

// ---------------------------------------------------------------------------
// Tests: Error node expected tokens
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Empty expected list is preserved.
    #[test]
    fn expected_empty(_dummy in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert!(state.get_error_nodes()[0].expected.is_empty());
    }

    /// Arbitrary expected list is preserved in full.
    #[test]
    fn expected_arbitrary_preserved(expected in arb_symbol_vec(20)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), expected.clone(), None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(&state.get_error_nodes()[0].expected, &expected);
    }

    /// Expected list length matches what was provided.
    #[test]
    fn expected_length_matches(expected in arb_symbol_vec(16)) {
        let len = expected.len();
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), expected, None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].expected.len(), len);
    }

    /// Expected list element order is preserved.
    #[test]
    fn expected_order_preserved(expected in prop::collection::vec(0u16..500, 2..10)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), expected.clone(), None, RecoveryStrategy::PanicMode, vec![]);
        let node = &state.get_error_nodes()[0];
        for i in 0..expected.len() {
            prop_assert_eq!(node.expected[i], expected[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Error node actual token
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// None actual is preserved.
    #[test]
    fn actual_none_preserved(_dummy in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].actual, None);
    }

    /// Some(symbol) actual is preserved.
    #[test]
    fn actual_some_preserved(sym in 0u16..1000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], Some(sym), RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].actual, Some(sym));
    }

    /// Actual token round-trips through arbitrary optional.
    #[test]
    fn actual_roundtrip(actual in arb_optional_symbol()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], actual, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].actual, actual);
    }
}

// ---------------------------------------------------------------------------
// Tests: Error node recovery strategy
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Recovery strategy is preserved on the node.
    #[test]
    fn strategy_preserved(strategy in arb_recovery_strategy()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, strategy, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].recovery, strategy);
    }

    /// Different strategies on different nodes are each preserved.
    #[test]
    fn strategy_per_node(
        s1 in arb_recovery_strategy(),
        s2 in arb_recovery_strategy(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, s1, vec![]);
        state.record_error(1, 2, (0, 1), (0, 2), vec![], None, s2, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes[0].recovery, s1);
        prop_assert_eq!(nodes[1].recovery, s2);
    }
}

// ---------------------------------------------------------------------------
// Tests: Error node skipped token count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Empty skipped_tokens list is preserved.
    #[test]
    fn skipped_empty(_dummy in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert!(state.get_error_nodes()[0].skipped_tokens.is_empty());
    }

    /// Skipped tokens count matches what was provided.
    #[test]
    fn skipped_count_matches(skipped in arb_symbol_vec(15)) {
        let count = skipped.len();
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, skipped);
        prop_assert_eq!(state.get_error_nodes()[0].skipped_tokens.len(), count);
    }

    /// Skipped tokens content is preserved exactly.
    #[test]
    fn skipped_content_preserved(skipped in arb_symbol_vec(12)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, skipped.clone());
        prop_assert_eq!(&state.get_error_nodes()[0].skipped_tokens, &skipped);
    }

    /// Skipped tokens order is preserved.
    #[test]
    fn skipped_order_preserved(skipped in prop::collection::vec(0u16..500, 2..10)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, skipped.clone());
        let node = &state.get_error_nodes()[0];
        for i in 0..skipped.len() {
            prop_assert_eq!(node.skipped_tokens[i], skipped[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Multiple error nodes
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// N calls to record_error produce exactly N error nodes.
    #[test]
    fn multiple_count_matches(count in 1usize..40) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(i, i + 1, (0, i), (0, i + 1), vec![i as u16], None, RecoveryStrategy::PanicMode, vec![]);
        }
        prop_assert_eq!(state.get_error_nodes().len(), count);
    }

    /// Each node in a batch has its own expected list.
    #[test]
    fn multiple_independent_expected(
        lists in prop::collection::vec(arb_symbol_vec(6), 2..12),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, list) in lists.iter().enumerate() {
            state.record_error(i, i + 1, (0, 0), (0, 1), list.clone(), None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..lists.len() {
            prop_assert_eq!(&nodes[i].expected, &lists[i]);
        }
    }

    /// Each node in a batch has its own actual token.
    #[test]
    fn multiple_independent_actual(
        actuals in prop::collection::vec(arb_optional_symbol(), 2..12),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, &actual) in actuals.iter().enumerate() {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], actual, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..actuals.len() {
            prop_assert_eq!(nodes[i].actual, actuals[i]);
        }
    }

    /// Each node in a batch has its own recovery strategy.
    #[test]
    fn multiple_independent_strategy(
        strategies in prop::collection::vec(arb_recovery_strategy(), 2..12),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, &strat) in strategies.iter().enumerate() {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, strat, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..strategies.len() {
            prop_assert_eq!(nodes[i].recovery, strategies[i]);
        }
    }

    /// Each node in a batch has its own skipped tokens.
    #[test]
    fn multiple_independent_skipped(
        skip_lists in prop::collection::vec(arb_symbol_vec(5), 2..12),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, list) in skip_lists.iter().enumerate() {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, list.clone());
        }
        let nodes = state.get_error_nodes();
        for i in 0..skip_lists.len() {
            prop_assert_eq!(&nodes[i].skipped_tokens, &skip_lists[i]);
        }
    }

    /// clear_errors empties the list, then new records start fresh.
    #[test]
    fn multiple_clear_and_rerecord(
        first in 1usize..10,
        second in 1usize..10,
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..first {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        state.clear_errors();
        prop_assert!(state.get_error_nodes().is_empty());
        for i in 0..second {
            state.record_error(i + 100, i + 101, (0, 0), (0, 1), vec![], None, RecoveryStrategy::TokenDeletion, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), second);
        prop_assert_eq!(nodes[0].start_byte, 100);
    }

    /// get_error_nodes is non-destructive: two reads yield same count.
    #[test]
    fn multiple_read_idempotent(count in 0usize..20) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let a = state.get_error_nodes();
        let b = state.get_error_nodes();
        prop_assert_eq!(a.len(), b.len());
        for i in 0..a.len() {
            prop_assert_eq!(a[i].start_byte, b[i].start_byte);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Error node ordering by position
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Nodes appear in insertion order (ascending start_byte when inserted that way).
    #[test]
    fn ordering_insertion_order_ascending(count in 2usize..25) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            let start = i * 10;
            state.record_error(start, start + 5, (i, 0), (i, 5), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 1..nodes.len() {
            prop_assert!(nodes[i].start_byte > nodes[i - 1].start_byte);
        }
    }

    /// Nodes inserted in descending order preserve that insertion order.
    #[test]
    fn ordering_insertion_order_descending(count in 2usize..25) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in (0..count).rev() {
            let start = i * 10;
            state.record_error(start, start + 5, (i, 0), (i, 5), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        // Insertion order preserved: first inserted had highest start_byte
        for i in 1..nodes.len() {
            prop_assert!(nodes[i].start_byte < nodes[i - 1].start_byte);
        }
    }

    /// Nodes can be sorted by start_byte after retrieval.
    #[test]
    fn ordering_sortable_by_start_byte(
        offsets in prop::collection::vec(0usize..100_000, 2..20),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, &off) in offsets.iter().enumerate() {
            state.record_error(off, off + 1, (i, 0), (i, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let mut nodes = state.get_error_nodes();
        nodes.sort_by_key(|n| n.start_byte);
        for i in 1..nodes.len() {
            prop_assert!(nodes[i].start_byte >= nodes[i - 1].start_byte);
        }
    }

    /// Sorted nodes with unique offsets are strictly ascending.
    #[test]
    fn ordering_unique_offsets_strictly_ascending(count in 2usize..20) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        // Use unique, shuffled offsets
        let mut offsets: Vec<usize> = (0..count).map(|i| i * 7 + 3).collect();
        offsets.reverse(); // insert in reverse
        for (idx, &off) in offsets.iter().enumerate() {
            state.record_error(off, off + 1, (idx, 0), (idx, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let mut nodes = state.get_error_nodes();
        nodes.sort_by_key(|n| n.start_byte);
        for i in 1..nodes.len() {
            prop_assert!(nodes[i].start_byte > nodes[i - 1].start_byte);
        }
    }

    /// Nodes at the same position preserve insertion order.
    #[test]
    fn ordering_same_position_insertion_order(count in 2usize..15) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(42, 43, (0, 0), (0, 1), vec![i as u16], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), count);
        for i in 0..count {
            prop_assert_eq!(&nodes[i].expected, &vec![i as u16]);
        }
    }
}
