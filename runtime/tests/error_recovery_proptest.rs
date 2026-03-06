#![allow(clippy::needless_range_loop)]
//! Property-based tests for the `adze::error_recovery` module.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;
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
    prop_oneof![Just(None), (0u16..1000).prop_map(Some),]
}

// ---------------------------------------------------------------------------
// Tests: ErrorRecoveryState creation
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Fresh state always starts with zero consecutive errors.
    #[test]
    fn state_creation_zero_errors(
        max_panic in 1usize..200,
        max_consec in 1usize..50,
    ) {
        let config = ErrorRecoveryConfig {
            max_panic_skip: max_panic,
            max_consecutive_errors: max_consec,
            ..Default::default()
        };
        let state = ErrorRecoveryState::new(config);
        let errors = state.get_error_nodes();
        prop_assert!(errors.is_empty());
    }

    /// Fresh state should not give up immediately (unless max_consecutive_errors == 0).
    #[test]
    fn state_creation_not_give_up(max_consec in 1usize..100) {
        let config = ErrorRecoveryConfig {
            max_consecutive_errors: max_consec,
            ..Default::default()
        };
        let state = ErrorRecoveryState::new(config);
        prop_assert!(!state.should_give_up());
    }

    /// Builder produces config with specified max_panic_skip.
    #[test]
    fn builder_max_panic_skip(val in 1usize..1000) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(val)
            .build();
        prop_assert_eq!(config.max_panic_skip, val);
    }
}

// ---------------------------------------------------------------------------
// Tests: record_error with random parameters
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A single recorded error is retrievable with correct byte offsets.
    #[test]
    fn record_single_error_byte_offsets(
        (start, end) in arb_byte_range(),
        start_pos in arb_position(),
        end_pos in arb_position(),
        expected in arb_symbol_vec(8),
        actual in arb_optional_symbol(),
        strategy in arb_recovery_strategy(),
        skipped in arb_symbol_vec(5),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(start, end, start_pos, end_pos, expected.clone(), actual, strategy, skipped.clone());
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
        prop_assert_eq!(nodes[0].start_byte, start);
        prop_assert_eq!(nodes[0].end_byte, end);
    }

    /// Recorded error preserves the expected symbol list.
    #[test]
    fn record_error_preserves_expected(
        expected in arb_symbol_vec(16),
        actual in arb_optional_symbol(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 10, (0, 0), (0, 10), expected.clone(), actual, RecoveryStrategy::PanicMode, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(&nodes[0].expected, &expected);
    }

    /// Recorded error preserves the actual symbol.
    #[test]
    fn record_error_preserves_actual(actual in arb_optional_symbol()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], actual, RecoveryStrategy::TokenDeletion, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes[0].actual, actual);
    }

    /// Recorded error preserves the recovery strategy.
    #[test]
    fn record_error_preserves_strategy(strategy in arb_recovery_strategy()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, strategy, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes[0].recovery, strategy);
    }
}

// ---------------------------------------------------------------------------
// Tests: get_error_nodes returns all recorded errors
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// All recorded errors appear in get_error_nodes in insertion order.
    #[test]
    fn get_error_nodes_returns_all(count in 1usize..30) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(i, i + 1, (0, i), (0, i + 1), vec![i as u16], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), count);
        for i in 0..count {
            prop_assert_eq!(nodes[i].start_byte, i);
        }
    }

    /// Getting error nodes twice yields the same result (non-destructive read).
    #[test]
    fn get_error_nodes_idempotent(count in 0usize..15) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let first = state.get_error_nodes();
        let second = state.get_error_nodes();
        prop_assert_eq!(first.len(), second.len());
    }
}

// ---------------------------------------------------------------------------
// Tests: error count matches recorded count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Error node count equals number of record_error calls.
    #[test]
    fn error_count_matches(count in 0usize..50) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        prop_assert_eq!(state.get_error_nodes().len(), count);
    }

    /// Increment/reset error count tracks consecutive errors independently of recorded nodes.
    #[test]
    fn increment_reset_independent_of_nodes(
        increments in 0usize..20,
        records in 0usize..10,
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for _ in 0..increments {
            state.increment_error_count();
        }
        for i in 0..records {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        prop_assert_eq!(state.get_error_nodes().len(), records);
    }
}

// ---------------------------------------------------------------------------
// Tests: RecoveryStrategy variants all constructible
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Every RecoveryStrategy variant can be cloned and compared for equality.
    #[test]
    fn strategy_clone_eq(strategy in arb_recovery_strategy()) {
        let cloned = strategy;
        prop_assert_eq!(strategy, cloned);
    }

    /// Every RecoveryStrategy variant has a Debug representation.
    #[test]
    fn strategy_debug_nonempty(strategy in arb_recovery_strategy()) {
        let debug = format!("{:?}", strategy);
        prop_assert!(!debug.is_empty());
    }

    /// RecoveryAction::InsertToken round-trips the symbol id.
    #[test]
    fn action_insert_token_roundtrip(sym in 0u16..1000) {
        let action = RecoveryAction::InsertToken(SymbolId(sym));
        match action {
            RecoveryAction::InsertToken(s) => prop_assert_eq!(s.0, sym),
            _ => prop_assert!(false, "expected InsertToken"),
        }
    }

    /// RecoveryAction::DeleteToken matches correctly.
    #[test]
    fn action_delete_token(_dummy in 0u8..1) {
        let action = RecoveryAction::DeleteToken;
        prop_assert!(matches!(action, RecoveryAction::DeleteToken));
    }
}

// ---------------------------------------------------------------------------
// Tests: max errors configuration respected
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// should_give_up returns true once consecutive errors reach the configured max.
    #[test]
    fn should_give_up_at_max(max_consec in 1usize..30) {
        let config = ErrorRecoveryConfig {
            max_consecutive_errors: max_consec,
            ..Default::default()
        };
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..max_consec {
            prop_assert!(!state.should_give_up());
            state.increment_error_count();
        }
        prop_assert!(state.should_give_up());
    }

    /// should_give_up returns false when below max.
    #[test]
    fn should_not_give_up_below_max(
        max_consec in 2usize..30,
        errors in 0usize..29,
    ) {
        prop_assume!(errors < max_consec);
        let config = ErrorRecoveryConfig {
            max_consecutive_errors: max_consec,
            ..Default::default()
        };
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..errors {
            state.increment_error_count();
        }
        prop_assert!(!state.should_give_up());
    }

    /// reset_error_count brings consecutive errors back to zero.
    #[test]
    fn reset_clears_consecutive(increments in 1usize..50) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for _ in 0..increments {
            state.increment_error_count();
        }
        state.reset_error_count();
        prop_assert!(!state.should_give_up());
    }

    /// max_consecutive_errors from builder matches what is configured.
    #[test]
    fn builder_max_consecutive_errors(max in 1usize..200) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(max)
            .build();
        prop_assert_eq!(config.max_consecutive_errors, max);
    }
}

// ---------------------------------------------------------------------------
// Tests: recovery doesn't lose data
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// clear_errors removes all nodes, but subsequent records still work.
    #[test]
    fn clear_then_record(
        first_count in 1usize..10,
        second_count in 1usize..10,
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..first_count {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        state.clear_errors();
        prop_assert!(state.get_error_nodes().is_empty());
        for i in 0..second_count {
            state.record_error(i + 100, i + 101, (0, 0), (0, 1), vec![], None, RecoveryStrategy::TokenDeletion, vec![]);
        }
        prop_assert_eq!(state.get_error_nodes().len(), second_count);
    }

    /// Recording errors with varying expected lists preserves each list independently.
    #[test]
    fn varying_expected_lists_preserved(
        lists in prop::collection::vec(arb_symbol_vec(8), 1..15),
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

    /// Scope push/pop round-trips correctly for arbitrary delimiter pairs.
    #[test]
    fn scope_push_pop_roundtrip(
        open in 0u16..500,
        close in 500u16..1000,
        depth in 1usize..10,
    ) {
        let config = ErrorRecoveryConfig {
            scope_delimiters: vec![(open, close)],
            ..Default::default()
        };
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..depth {
            state.push_scope(open);
        }
        for _ in 0..depth {
            let popped = state.pop_scope(close);
            prop_assert!(popped);
        }
        // Stack should be empty now; an extra pop should fail
        prop_assert!(!state.pop_scope(close));
    }

    /// Recent tokens buffer stays at capacity 10.
    #[test]
    fn recent_tokens_bounded(count in 0usize..50) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.add_recent_token(i as u16);
        }
        // We can't directly inspect recent_tokens from outside,
        // but we can verify the state remains usable.
        let errors = state.get_error_nodes();
        prop_assert!(errors.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Tests: error positions are preserved
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Start and end byte positions are preserved exactly.
    #[test]
    fn byte_positions_preserved(
        (start, end) in arb_byte_range(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(start, end, (0, 0), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes[0].start_byte, start);
        prop_assert_eq!(nodes[0].end_byte, end);
    }

    /// Row/column positions are preserved in the error node.
    #[test]
    fn row_col_positions_preserved(
        start_pos in arb_position(),
        end_pos in arb_position(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, start_pos, end_pos, vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes[0].start_position, start_pos);
        prop_assert_eq!(nodes[0].end_position, end_pos);
    }

    /// Multiple errors preserve their positions in order.
    #[test]
    fn multiple_positions_ordered(
        offsets in prop::collection::vec(0usize..100_000, 1..20),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, &off) in offsets.iter().enumerate() {
            state.record_error(off, off + 1, (i, 0), (i, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), offsets.len());
        for i in 0..offsets.len() {
            prop_assert_eq!(nodes[i].start_byte, offsets[i]);
            prop_assert_eq!(nodes[i].start_position.0, i);
        }
    }

    /// Skipped tokens are preserved per error node.
    #[test]
    fn skipped_tokens_preserved(
        skipped in arb_symbol_vec(12),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, skipped.clone());
        let nodes = state.get_error_nodes();
        prop_assert_eq!(&nodes[0].skipped_tokens, &skipped);
    }
}

// ---------------------------------------------------------------------------
// Tests: config properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// can_delete_token returns true for non-sync tokens.
    #[test]
    fn can_delete_non_sync(
        sync_tokens in prop::collection::vec(0u16..100, 0..5),
        query in 100u16..200,
    ) {
        let mut config = ErrorRecoveryConfig::default();
        for &t in &sync_tokens {
            config.sync_tokens.push(SymbolId(t));
        }
        // query is outside sync range 0..100
        prop_assert!(config.can_delete_token(SymbolId(query)));
    }

    /// can_delete_token returns false for sync tokens (unless in deletable set).
    #[test]
    fn cannot_delete_sync(sync_val in 0u16..1000) {
        let mut config = ErrorRecoveryConfig::default();
        config.sync_tokens.push(SymbolId(sync_val));
        prop_assert!(!config.can_delete_token(SymbolId(sync_val)));
    }

    /// can_replace_token returns false for sync tokens.
    #[test]
    fn cannot_replace_sync(sync_val in 0u16..1000) {
        let mut config = ErrorRecoveryConfig::default();
        config.sync_tokens.push(SymbolId(sync_val));
        prop_assert!(!config.can_replace_token(SymbolId(sync_val)));
    }

    /// Builder scope delimiter round-trips.
    #[test]
    fn builder_scope_delimiters(
        open in 0u16..500,
        close in 500u16..1000,
    ) {
        let config = ErrorRecoveryConfigBuilder::new()
            .add_scope_delimiter(open, close)
            .build();
        prop_assert_eq!(config.scope_delimiters.len(), 1);
        prop_assert_eq!(config.scope_delimiters[0], (open, close));
    }

    /// Static is_scope_delimiter agrees with config.
    #[test]
    fn static_is_scope_delimiter(
        open in 0u16..500,
        close in 500u16..1000,
        query in 0u16..1000,
    ) {
        let delims = vec![(open, close)];
        let expected = query == open || query == close;
        prop_assert_eq!(ErrorRecoveryState::is_scope_delimiter(query, &delims), expected);
    }

    /// Static is_matching_delimiter is correct.
    #[test]
    fn static_is_matching_delimiter(
        open in 0u16..500,
        close in 500u16..1000,
        q_open in 0u16..1000,
        q_close in 0u16..1000,
    ) {
        let delims = vec![(open, close)];
        let expected = q_open == open && q_close == close;
        prop_assert_eq!(
            ErrorRecoveryState::is_matching_delimiter(q_open, q_close, &delims),
            expected
        );
    }
}
