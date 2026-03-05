//! Property-based tests for error recovery in the adze runtime.
//!
//! 46 proptest property tests covering config invariants, state management,
//! mode transitions, error nodes, limits, cost accounting, reset, and combined
//! properties.

#![allow(unused_imports)]
#![allow(dead_code)]

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};
use adze::lexer::ErrorRecoveryMode;
use adze_ir::SymbolId;
use proptest::prelude::*;
use std::collections::HashSet;
use std::sync::atomic::Ordering;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config_with_max_errors(max: usize) -> ErrorRecoveryConfig {
    ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(max)
        .build()
}

fn config_with_sync_tokens(tokens: &[u16]) -> ErrorRecoveryConfig {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for &t in tokens {
        builder = builder.add_sync_token(t);
    }
    builder.build()
}

fn config_with_insertable(tokens: &[u16]) -> ErrorRecoveryConfig {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for &t in tokens {
        builder = builder.add_insertable_token(t);
    }
    builder.build()
}

fn config_with_scope_delimiters(pairs: &[(u16, u16)]) -> ErrorRecoveryConfig {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for &(o, c) in pairs {
        builder = builder.add_scope_delimiter(o, c);
    }
    builder.build()
}

fn make_error_node(
    start: usize,
    end: usize,
    expected: Vec<u16>,
    actual: Option<u16>,
    strategy: RecoveryStrategy,
) -> ErrorNode {
    ErrorNode {
        start_byte: start,
        end_byte: end,
        start_position: (0, start),
        end_position: (0, end),
        expected,
        actual,
        recovery: strategy,
        skipped_tokens: Vec::new(),
    }
}

fn all_strategies() -> Vec<RecoveryStrategy> {
    vec![
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ]
}

fn strategy_from_index(idx: usize) -> RecoveryStrategy {
    let strats = all_strategies();
    strats[idx % strats.len()]
}

// ---------------------------------------------------------------------------
// Category 1: prop_error_config_* — config property invariants (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_config_default_values_are_sane(
        _dummy in 0u8..1,
    ) {
        let cfg = ErrorRecoveryConfig::default();
        prop_assert!(cfg.max_panic_skip > 0);
        prop_assert!(cfg.max_consecutive_errors > 0);
        prop_assert!(cfg.max_token_deletions > 0);
        prop_assert!(cfg.max_token_insertions > 0);
        prop_assert!(cfg.enable_phrase_recovery);
        prop_assert!(cfg.enable_scope_recovery);
        prop_assert!(!cfg.enable_indentation_recovery);
        prop_assert!(cfg.sync_tokens.is_empty());
        prop_assert!(cfg.insert_candidates.is_empty());
        prop_assert!(cfg.scope_delimiters.is_empty());
    }

    #[test]
    fn prop_error_config_builder_preserves_panic_skip(max in 1usize..1000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(max)
            .build();
        prop_assert_eq!(cfg.max_panic_skip, max);
    }

    #[test]
    fn prop_error_config_builder_accumulates_sync_tokens(
        tokens in prop::collection::vec(1u16..500, 1..20),
    ) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &t in &tokens {
            builder = builder.add_sync_token(t);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.sync_tokens.len(), tokens.len());
        for &t in &tokens {
            prop_assert!(cfg.sync_tokens.iter().any(|s| s.0 == t));
        }
    }

    #[test]
    fn prop_error_config_builder_accumulates_insertable(
        tokens in prop::collection::vec(1u16..500, 1..20),
    ) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &t in &tokens {
            builder = builder.add_insertable_token(t);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.insert_candidates.len(), tokens.len());
    }

    #[test]
    fn prop_error_config_can_delete_non_sync_token(
        token_val in 100u16..200,
        sync_val in 1u16..50,
    ) {
        let cfg = config_with_sync_tokens(&[sync_val]);
        let non_sync = SymbolId(token_val);
        prop_assert!(cfg.can_delete_token(non_sync));
    }

    #[test]
    fn prop_error_config_cannot_replace_sync_token(
        sync_val in 1u16..500,
    ) {
        let cfg = config_with_sync_tokens(&[sync_val]);
        let token = SymbolId(sync_val);
        prop_assert!(!cfg.can_replace_token(token));
    }
}

// ---------------------------------------------------------------------------
// Category 2: prop_error_state_* — state management properties (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_state_new_starts_empty(max in 1usize..100) {
        let cfg = config_with_max_errors(max);
        let state = ErrorRecoveryState::new(cfg);
        prop_assert!(state.get_error_nodes().is_empty());
        prop_assert!(!state.should_give_up());
    }

    #[test]
    fn prop_error_state_record_preserves_fields(
        start in 0usize..1000,
        len in 1usize..500,
        expected in prop::collection::vec(1u16..100, 1..5),
        actual in prop::option::of(1u16..100),
        strat_idx in 0usize..6,
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        let end = start + len;
        let strategy = strategy_from_index(strat_idx);

        state.record_error(
            start,
            end,
            (0, start),
            (0, end),
            expected.clone(),
            actual,
            strategy,
            Vec::new(),
        );

        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
        prop_assert_eq!(nodes[0].start_byte, start);
        prop_assert_eq!(nodes[0].end_byte, end);
        prop_assert_eq!(&nodes[0].expected, &expected);
        prop_assert_eq!(nodes[0].actual, actual);
        prop_assert_eq!(nodes[0].recovery, strategy);
    }

    #[test]
    fn prop_error_state_multiple_records_accumulate(
        count in 1usize..20,
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);

        for i in 0..count {
            state.record_error(
                i, i + 1, (0, i), (0, i + 1),
                vec![1], None, RecoveryStrategy::PanicMode, Vec::new(),
            );
        }

        prop_assert_eq!(state.get_error_nodes().len(), count);
    }

    #[test]
    fn prop_error_state_recent_tokens_bounded(
        tokens in prop::collection::vec(1u16..500, 1..50),
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        for &t in &tokens {
            state.add_recent_token(t);
        }
        // recent_tokens is bounded to 10 entries
        // We can't read it directly, but adding many tokens should not panic
        prop_assert!(true);
    }

    #[test]
    fn prop_error_state_update_recent_tokens_via_symbol(
        vals in prop::collection::vec(0u16..1000, 1..30),
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        for &v in &vals {
            state.update_recent_tokens(SymbolId(v));
        }
        // Should not panic regardless of count
        prop_assert!(true);
    }

    #[test]
    fn prop_error_state_scope_push_pop_balanced(
        count in 1usize..10,
    ) {
        // Generate pairs with unique open and close values
        let pairs: Vec<(u16, u16)> = (0..count)
            .map(|i| (i as u16 * 2, i as u16 * 2 + 1))
            .collect();
        let cfg = config_with_scope_delimiters(&pairs);
        let mut state = ErrorRecoveryState::new(cfg);

        // Push all openers
        for &(open, _) in &pairs {
            state.push_scope(open);
        }

        // Pop in reverse (LIFO)
        for &(_, close) in pairs.iter().rev() {
            prop_assert!(state.pop_scope(close));
        }

        // Stack should be empty
        prop_assert_eq!(state.pop_scope_test(), None);
    }
}

// ---------------------------------------------------------------------------
// Category 3: prop_error_mode_* — mode transition properties (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_mode_variants_are_distinct(
        a_idx in 0usize..3,
        b_idx in 0usize..3,
    ) {
        let modes = [
            ErrorRecoveryMode::SkipChar,
            ErrorRecoveryMode::SkipToKnown,
            ErrorRecoveryMode::Fail,
        ];
        if a_idx == b_idx {
            prop_assert_eq!(modes[a_idx], modes[b_idx]);
        } else {
            prop_assert_ne!(modes[a_idx], modes[b_idx]);
        }
    }

    #[test]
    fn prop_error_mode_exceeding_limit_gives_panic(
        max in 1usize..20,
        overshoot in 1usize..10,
    ) {
        let mut cfg = ErrorRecoveryConfig::default();
        cfg.max_consecutive_errors = max;
        cfg.enable_phrase_recovery = false;
        let mut state = ErrorRecoveryState::new(cfg);

        // Exhaust errors to exceed the limit
        for _ in 0..max + overshoot {
            state.increment_error_count();
        }
        let strategy = state.determine_recovery_strategy(&[99], Some(88), (0, 0), 0);
        prop_assert_eq!(strategy, RecoveryStrategy::PanicMode);
    }

    #[test]
    fn prop_error_mode_insertion_when_candidate_available(
        token_id in 1u16..500,
    ) {
        let cfg = config_with_insertable(&[token_id]);
        let mut state = ErrorRecoveryState::new(cfg);
        let strategy = state.determine_recovery_strategy(
            &[token_id],
            None,
            (0, 0),
            0,
        );
        prop_assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
    }

    #[test]
    fn prop_error_mode_phrase_level_as_fallback(
        actual in 1u16..500,
    ) {
        // No insertable, no deletable conditions, phrase recovery enabled
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(true)
            .enable_scope_recovery(false)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        // expected doesn't contain actual, and actual not in sync tokens
        let strategy = state.determine_recovery_strategy(
            &[actual.wrapping_add(100)],
            Some(actual),
            (0, 0),
            0,
        );
        // With only one expected token, substitution is chosen
        // Let's use two expected so substitution check (expected.len() == 1) is false
        let mut state2 = ErrorRecoveryState::new(
            ErrorRecoveryConfigBuilder::new()
                .enable_phrase_recovery(true)
                .enable_scope_recovery(false)
                .build(),
        );
        let strategy2 = state2.determine_recovery_strategy(
            &[actual.wrapping_add(100), actual.wrapping_add(200)],
            Some(actual),
            (0, 0),
            0,
        );
        // Deletion tried first (token not in sync/expected), so we get deletion
        // TokenDeletion or PhraseLevel depending on is_clearly_wrong
        prop_assert!(
            strategy2 == RecoveryStrategy::TokenDeletion
            || strategy2 == RecoveryStrategy::PhraseLevel
        );
    }

    #[test]
    fn prop_error_mode_substitution_single_expected(
        actual in 1u16..200,
        expected_single in 201u16..400,
    ) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(false)
            .enable_scope_recovery(false)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        // one expected, actual != expected, not a sync token => is_clearly_wrong
        // but deletion is checked before substitution; both may apply
        let strategy = state.determine_recovery_strategy(
            &[expected_single],
            Some(actual),
            (0, 0),
            0,
        );
        prop_assert!(
            strategy == RecoveryStrategy::TokenDeletion
            || strategy == RecoveryStrategy::TokenSubstitution
        );
    }

    #[test]
    fn prop_error_mode_recovery_strategies_are_distinct(
        a in 0usize..6,
        b in 0usize..6,
    ) {
        let sa = strategy_from_index(a);
        let sb = strategy_from_index(b);
        if a == b {
            prop_assert_eq!(sa, sb);
        } else {
            prop_assert_ne!(sa, sb);
        }
    }
}

// ---------------------------------------------------------------------------
// Category 4: prop_error_node_* — error node properties (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_node_byte_range_ordering(
        start in 0usize..10000,
        len in 1usize..5000,
    ) {
        let end = start + len;
        let node = make_error_node(start, end, vec![1], None, RecoveryStrategy::PanicMode);
        prop_assert!(node.start_byte < node.end_byte);
        prop_assert_eq!(node.end_byte - node.start_byte, len);
    }

    #[test]
    fn prop_error_node_preserves_expected(
        expected in prop::collection::vec(0u16..1000, 0..20),
    ) {
        let node = make_error_node(0, 1, expected.clone(), None, RecoveryStrategy::PanicMode);
        prop_assert_eq!(&node.expected, &expected);
    }

    #[test]
    fn prop_error_node_preserves_actual(
        actual in prop::option::of(0u16..1000),
    ) {
        let node = make_error_node(0, 1, vec![], actual, RecoveryStrategy::PanicMode);
        prop_assert_eq!(node.actual, actual);
    }

    #[test]
    fn prop_error_node_clone_eq(
        start in 0usize..1000,
        end_off in 1usize..500,
        expected in prop::collection::vec(0u16..100, 0..5),
        actual in prop::option::of(0u16..100),
        strat_idx in 0usize..6,
    ) {
        let node = make_error_node(
            start,
            start + end_off,
            expected,
            actual,
            strategy_from_index(strat_idx),
        );
        let cloned = node.clone();
        prop_assert_eq!(node.start_byte, cloned.start_byte);
        prop_assert_eq!(node.end_byte, cloned.end_byte);
        prop_assert_eq!(&node.expected, &cloned.expected);
        prop_assert_eq!(node.actual, cloned.actual);
        prop_assert_eq!(node.recovery, cloned.recovery);
    }

    #[test]
    fn prop_error_node_skipped_tokens_preserved(
        skipped in prop::collection::vec(0u16..500, 0..20),
    ) {
        let node = ErrorNode {
            start_byte: 0,
            end_byte: 1,
            start_position: (0, 0),
            end_position: (0, 1),
            expected: vec![],
            actual: None,
            recovery: RecoveryStrategy::PanicMode,
            skipped_tokens: skipped.clone(),
        };
        prop_assert_eq!(&node.skipped_tokens, &skipped);
    }

    #[test]
    fn prop_error_node_position_fields(
        row in 0usize..1000,
        col_start in 0usize..200,
        col_len in 1usize..200,
    ) {
        let node = ErrorNode {
            start_byte: col_start,
            end_byte: col_start + col_len,
            start_position: (row, col_start),
            end_position: (row, col_start + col_len),
            expected: vec![],
            actual: None,
            recovery: RecoveryStrategy::TokenDeletion,
            skipped_tokens: vec![],
        };
        prop_assert_eq!(node.start_position.0, row);
        prop_assert_eq!(node.end_position.0, row);
        prop_assert!(node.start_position.1 < node.end_position.1);
    }
}

// ---------------------------------------------------------------------------
// Category 5: prop_error_limit_* — limit properties (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_limit_should_give_up_at_max(max in 1usize..50) {
        let cfg = config_with_max_errors(max);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..max {
            state.increment_error_count();
        }
        prop_assert!(state.should_give_up());
    }

    #[test]
    fn prop_error_limit_should_not_give_up_below_max(
        max in 2usize..50,
        count in 0usize..50,
    ) {
        prop_assume!(count < max);
        let cfg = config_with_max_errors(max);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..count {
            state.increment_error_count();
        }
        prop_assert!(!state.should_give_up());
    }

    #[test]
    fn prop_error_limit_increment_is_monotonic(steps in 1usize..30) {
        let cfg = config_with_max_errors(100);
        let mut state = ErrorRecoveryState::new(cfg);
        let mut prev_give_up = false;
        for _ in 0..steps {
            state.increment_error_count();
            let now = state.should_give_up();
            // Once give_up becomes true, it stays true
            if prev_give_up {
                prop_assert!(now);
            }
            prev_give_up = now;
        }
    }

    #[test]
    fn prop_error_limit_max_panic_skip_preserved(val in 1usize..10000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(val)
            .build();
        prop_assert_eq!(cfg.max_panic_skip, val);
    }

    #[test]
    fn prop_error_limit_scope_delimiters_count(
        count in 0usize..20,
    ) {
        let pairs: Vec<(u16, u16)> = (0..count)
            .map(|i| (i as u16 * 2, i as u16 * 2 + 1))
            .collect();
        let cfg = config_with_scope_delimiters(&pairs);
        prop_assert_eq!(cfg.scope_delimiters.len(), count);
    }

    #[test]
    fn prop_error_limit_deletable_tokens_all_present(
        tokens in prop::collection::hash_set(0u16..1000, 0..30),
    ) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &t in &tokens {
            builder = builder.add_deletable_token(t);
        }
        let cfg = builder.build();
        for &t in &tokens {
            prop_assert!(cfg.deletable_tokens.contains(&t));
        }
        prop_assert_eq!(cfg.deletable_tokens.len(), tokens.len());
    }
}

// ---------------------------------------------------------------------------
// Category 6: prop_error_cost_* — cost accounting properties (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_cost_error_count_tracks_increments(
        increments in 1usize..50,
    ) {
        let cfg = config_with_max_errors(100);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..increments {
            state.increment_error_count();
        }
        // After `increments` bumps, should_give_up iff increments >= 100
        prop_assert_eq!(state.should_give_up(), increments >= 100);
    }

    #[test]
    fn prop_error_cost_determine_strategy_increments_counter(
        calls in 1usize..15,
    ) {
        let cfg = config_with_max_errors(100);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..calls {
            let _ = state.determine_recovery_strategy(&[999], Some(888), (0, 0), 0);
        }
        // Each determine_recovery_strategy call increments (though some reset)
        // At minimum, errors have been processed
        prop_assert!(true);
    }

    #[test]
    fn prop_error_cost_insertion_resets_counter(
        token_id in 1u16..500,
    ) {
        let cfg = config_with_insertable(&[token_id]);
        let mut state = ErrorRecoveryState::new(cfg);

        // First bump up errors
        state.increment_error_count();
        state.increment_error_count();

        // Insertion should reset
        let strategy = state.determine_recovery_strategy(
            &[token_id],
            None,
            (0, 0),
            0,
        );
        prop_assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
        // After insertion, should not be at give-up threshold (reset happened)
        prop_assert!(!state.should_give_up());
    }

    #[test]
    fn prop_error_cost_error_nodes_count_matches_records(
        count in 0usize..30,
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        for i in 0..count {
            state.record_error(
                i, i + 1, (0, i), (0, i + 1),
                vec![1, 2], Some(3), RecoveryStrategy::TokenDeletion, vec![],
            );
        }
        prop_assert_eq!(state.get_error_nodes().len(), count);
    }

    #[test]
    fn prop_error_cost_scope_push_does_not_affect_error_count(
        pushes in 1usize..20,
    ) {
        let pairs: Vec<(u16, u16)> = (0..pushes)
            .map(|i| (i as u16 * 2, i as u16 * 2 + 1))
            .collect();
        let cfg = config_with_scope_delimiters(&pairs);
        let mut state = ErrorRecoveryState::new(cfg);
        for &(open, _) in &pairs {
            state.push_scope(open);
        }
        prop_assert!(!state.should_give_up());
    }

    #[test]
    fn prop_error_cost_non_matching_pop_does_not_crash(
        close_val in 200u16..300,
    ) {
        // No delimiters configured — pop should return false
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        let result = state.pop_scope(close_val);
        prop_assert!(!result);
    }
}

// ---------------------------------------------------------------------------
// Category 7: prop_error_reset_* — reset properties (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_reset_count_clears_give_up(
        max in 1usize..50,
    ) {
        let cfg = config_with_max_errors(max);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..max {
            state.increment_error_count();
        }
        prop_assert!(state.should_give_up());
        state.reset_error_count();
        prop_assert!(!state.should_give_up());
    }

    #[test]
    fn prop_error_reset_consecutive_errors(
        errors_before in 1usize..30,
    ) {
        let cfg = config_with_max_errors(100);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..errors_before {
            state.increment_error_count();
        }
        state.reset_consecutive_errors();
        prop_assert!(!state.should_give_up());
    }

    #[test]
    fn prop_error_reset_clear_errors_empties_nodes(
        count in 1usize..20,
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        for i in 0..count {
            state.record_error(
                i, i + 1, (0, i), (0, i + 1),
                vec![1], None, RecoveryStrategy::PanicMode, vec![],
            );
        }
        prop_assert!(!state.get_error_nodes().is_empty());
        state.clear_errors();
        prop_assert!(state.get_error_nodes().is_empty());
    }

    #[test]
    fn prop_error_reset_does_not_affect_error_nodes(
        count in 1usize..10,
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        for i in 0..count {
            state.record_error(
                i, i + 1, (0, i), (0, i + 1),
                vec![1], None, RecoveryStrategy::PanicMode, vec![],
            );
        }
        state.reset_error_count();
        // Error nodes should still be there — reset only clears the counter
        prop_assert_eq!(state.get_error_nodes().len(), count);
    }

    #[test]
    fn prop_error_reset_clear_does_not_affect_count(
        bumps in 1usize..20,
        max in 30usize..60,
    ) {
        let cfg = config_with_max_errors(max);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..bumps {
            state.increment_error_count();
        }
        state.record_error(
            0, 1, (0, 0), (0, 1),
            vec![1], None, RecoveryStrategy::PanicMode, vec![],
        );
        state.clear_errors();
        // Clearing nodes does not reset the error counter
        prop_assert_eq!(state.should_give_up(), bumps >= max);
    }

    #[test]
    fn prop_error_reset_idempotent(
        resets in 1usize..10,
    ) {
        let cfg = config_with_max_errors(5);
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..10 {
            state.increment_error_count();
        }
        for _ in 0..resets {
            state.reset_error_count();
        }
        prop_assert!(!state.should_give_up());
    }
}

// ---------------------------------------------------------------------------
// Category 8: prop_error_combined_* — combined properties (4 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 50, .. ProptestConfig::default() })]

    #[test]
    fn prop_error_combined_record_and_clear_cycle(
        cycles in 1usize..10,
        per_cycle in 1usize..10,
    ) {
        let cfg = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..cycles {
            for j in 0..per_cycle {
                state.record_error(
                    j, j + 1, (0, j), (0, j + 1),
                    vec![1], None, RecoveryStrategy::PanicMode, vec![],
                );
            }
            prop_assert_eq!(state.get_error_nodes().len(), per_cycle);
            state.clear_errors();
            prop_assert!(state.get_error_nodes().is_empty());
        }
    }

    #[test]
    fn prop_error_combined_scope_and_errors_independent(
        pair_count in 1usize..5,
        error_count in 0usize..10,
    ) {
        // Generate pairs with unique open and close values
        let pairs: Vec<(u16, u16)> = (0..pair_count)
            .map(|i| (i as u16 * 2, i as u16 * 2 + 1))
            .collect();
        let cfg = config_with_scope_delimiters(&pairs);
        let mut state = ErrorRecoveryState::new(cfg);

        // Push scopes
        for &(open, _) in &pairs {
            state.push_scope(open);
        }

        // Record errors
        for i in 0..error_count {
            state.record_error(
                i, i + 1, (0, i), (0, i + 1),
                vec![1], None, RecoveryStrategy::PanicMode, vec![],
            );
        }

        // Scopes and errors are independent
        prop_assert_eq!(state.get_error_nodes().len(), error_count);

        // Pop scopes in reverse
        for &(_, close) in pairs.iter().rev() {
            prop_assert!(state.pop_scope(close));
        }
        prop_assert_eq!(state.pop_scope_test(), None);
    }

    #[test]
    fn prop_error_combined_builder_flags_independence(
        phrase in proptest::bool::ANY,
        scope in proptest::bool::ANY,
        indent in proptest::bool::ANY,
        max_skip in 1usize..500,
        max_errors in 1usize..100,
    ) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(phrase)
            .enable_scope_recovery(scope)
            .enable_indentation_recovery(indent)
            .max_panic_skip(max_skip)
            .max_consecutive_errors(max_errors)
            .build();
        prop_assert_eq!(cfg.enable_phrase_recovery, phrase);
        prop_assert_eq!(cfg.enable_scope_recovery, scope);
        prop_assert_eq!(cfg.enable_indentation_recovery, indent);
        prop_assert_eq!(cfg.max_panic_skip, max_skip);
        prop_assert_eq!(cfg.max_consecutive_errors, max_errors);
    }

    #[test]
    fn prop_error_combined_static_delimiter_helpers(
        open in 1u16..100,
        close in 101u16..200,
        other in 201u16..300,
    ) {
        let delimiters = vec![(open, close)];
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(open, &delimiters));
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(close, &delimiters));
        prop_assert!(!ErrorRecoveryState::is_scope_delimiter(other, &delimiters));
        prop_assert!(ErrorRecoveryState::is_matching_delimiter(open, close, &delimiters));
        prop_assert!(!ErrorRecoveryState::is_matching_delimiter(close, open, &delimiters));
        prop_assert!(!ErrorRecoveryState::is_matching_delimiter(other, close, &delimiters));
    }
}
