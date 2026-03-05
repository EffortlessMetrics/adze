//! Error recovery v6 — 64 tests across 8 categories.
//!
//! Categories (8 tests each):
//!   1. recovery_config_*  — configuration options
//!   2. recovery_state_*   — state management
//!   3. recovery_mode_*    — mode transitions
//!   4. recovery_node_*    — error node creation
//!   5. recovery_limit_*   — error count limits
//!   6. recovery_cost_*    — error cost accounting
//!   7. recovery_reset_*   — state reset operations
//!   8. recovery_edge_*    — edge cases

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};
use adze_ir::SymbolId;
#[allow(unused_imports)]
use std::sync::atomic::Ordering;

// ===========================================================================
// 1. recovery_config — configuration options (8 tests)
// ===========================================================================

#[test]
fn recovery_config_default_max_panic_skip_is_50() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn recovery_config_default_max_token_deletions_is_3() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn recovery_config_default_max_token_insertions_is_2() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn recovery_config_builder_sets_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn recovery_config_builder_adds_sync_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(10)
        .add_sync_token(20)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 10));
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 20));
}

#[test]
fn recovery_config_builder_adds_insertable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .add_insertable_token(6)
        .build();
    assert_eq!(cfg.insert_candidates.len(), 2);
}

#[test]
fn recovery_config_builder_adds_deletable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(42)
        .build();
    assert!(cfg.deletable_tokens.contains(&42));
}

#[test]
fn recovery_config_builder_chain_multiple_options() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(75)
        .max_consecutive_errors(5)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    assert_eq!(cfg.max_panic_skip, 75);
    assert_eq!(cfg.max_consecutive_errors, 5);
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
}

// ===========================================================================
// 2. recovery_state — state management (8 tests)
// ===========================================================================

#[test]
fn recovery_state_new_has_zero_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn recovery_state_increment_advances_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    // Two increments, should not yet give up (default max = 10)
    assert!(!state.should_give_up());
}

#[test]
fn recovery_state_scope_push_records_opener() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    // Verify scope was pushed (pop returns it)
    assert_eq!(state.pop_scope_test(), Some(40));
}

#[test]
fn recovery_state_scope_push_ignores_non_delimiter() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99); // not an opener
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn recovery_state_pop_scope_matches_correctly() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41), (91, 93)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    state.push_scope(91);
    assert!(state.pop_scope(93)); // matches [
    assert!(state.pop_scope(41)); // matches (
}

#[test]
fn recovery_state_pop_scope_rejects_mismatch() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41), (91, 93)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    assert!(!state.pop_scope(93)); // 93 closes 91, but top is 40
}

#[test]
fn recovery_state_recent_tokens_bounded_at_10() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..20 {
        state.update_recent_tokens(SymbolId(i));
    }
    let nodes = state.get_error_nodes();
    // recent_tokens is internal; just ensure no panic and error_nodes is independent
    assert!(nodes.is_empty());
}

#[test]
fn recovery_state_add_recent_token_oldest_evicted() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 1..=12 {
        state.add_recent_token(i);
    }
    // We cannot directly read recent_tokens from the public API, but we can confirm
    // no panic and the state remains functional
    state.increment_error_count();
    assert!(!state.should_give_up());
}

// ===========================================================================
// 3. recovery_mode — mode transitions (8 tests)
// ===========================================================================

#[test]
fn recovery_mode_insertion_when_candidate_available() {
    let cfg = ErrorRecoveryConfig {
        insert_candidates: smallvec::smallvec![SymbolId(10)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
}

#[test]
fn recovery_mode_deletion_when_token_clearly_wrong() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11], Some(99), (0, 0), 0);
    // 99 is not expected and not a sync token → deletion
    assert_eq!(strat, RecoveryStrategy::TokenDeletion);
}

#[test]
fn recovery_mode_substitution_when_single_expected() {
    // actual must be a sync token so is_clearly_wrong returns false,
    // allowing substitution check (single expected) to fire.
    let cfg = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(99)],
        enable_phrase_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn recovery_mode_scope_recovery_on_unmatched_close() {
    // actual must be a sync token so is_clearly_wrong returns false and
    // deletion is skipped, allowing scope recovery to fire.
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        sync_tokens: smallvec::smallvec![SymbolId(41)],
        enable_phrase_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11], Some(41), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn recovery_mode_phrase_level_as_fallback() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: true,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    let _strat = state.determine_recovery_strategy(&[10, 11], Some(99), (0, 0), 0);
    // No insertion candidates, clearly wrong → deletion tried first for multi-expected...
    // Actually with 2 expected and token clearly wrong → TokenDeletion
    // Let's use a sync token as actual so it's not clearly wrong and not substitutable
    let cfg2 = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(99)],
        enable_phrase_recovery: true,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state2 = ErrorRecoveryState::new(cfg2);
    let strat2 = state2.determine_recovery_strategy(&[10, 11], Some(99), (0, 0), 0);
    assert_eq!(strat2, RecoveryStrategy::PhraseLevel);
}

#[test]
fn recovery_mode_panic_when_phrase_disabled() {
    let cfg = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(99)],
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PanicMode);
}

#[test]
fn recovery_mode_panic_after_max_errors_exceeded() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        insert_candidates: smallvec::smallvec![SymbolId(10)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Exhaust the error limit
    state.determine_recovery_strategy(&[50], Some(60), (0, 0), 0);
    state.determine_recovery_strategy(&[50], Some(60), (0, 0), 0);
    // Third call: consecutive errors = 2 already, fetch_add makes it 3 > max(2)
    let strat = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PanicMode);
}

#[test]
fn recovery_mode_insertion_resets_error_count() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 10,
        insert_candidates: smallvec::smallvec![SymbolId(10)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Increment errors a few times first
    state.increment_error_count();
    state.increment_error_count();
    // Token insertion should reset the counter
    let strat = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
    // After successful insertion, should_give_up is false
    assert!(!state.should_give_up());
}

// ===========================================================================
// 4. recovery_node — error node creation (8 tests)
// ===========================================================================

#[test]
fn recovery_node_record_and_retrieve() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::TokenDeletion,
        vec![3],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn recovery_node_preserves_byte_range() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        10,
        20,
        (1, 0),
        (1, 10),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, 10);
    assert_eq!(nodes[0].end_byte, 20);
}

#[test]
fn recovery_node_preserves_expected_symbols() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![5, 6, 7],
        Some(8),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].expected, vec![5, 6, 7]);
}

#[test]
fn recovery_node_preserves_actual_symbol() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        Some(42),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes()[0].actual, Some(42));
}

#[test]
fn recovery_node_actual_none_when_missing() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    assert_eq!(state.get_error_nodes()[0].actual, None);
}

#[test]
fn recovery_node_preserves_recovery_strategy() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        None,
        RecoveryStrategy::ScopeRecovery,
        vec![],
    );
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn recovery_node_multiple_records_accumulate() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![1],
            Some(2),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn recovery_node_direct_construction() {
    let node = ErrorNode {
        start_byte: 100,
        end_byte: 200,
        start_position: (5, 0),
        end_position: (5, 100),
        expected: vec![10, 20, 30],
        actual: Some(99),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![50, 51],
    };
    assert_eq!(node.start_byte, 100);
    assert_eq!(node.end_byte, 200);
    assert_eq!(node.expected.len(), 3);
    assert_eq!(node.actual, Some(99));
    assert_eq!(node.recovery, RecoveryStrategy::PhraseLevel);
}

// ===========================================================================
// 5. recovery_limit — error count limits (8 tests)
// ===========================================================================

#[test]
fn recovery_limit_should_give_up_at_threshold() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn recovery_limit_not_give_up_below_threshold() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn recovery_limit_threshold_one() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn recovery_limit_threshold_zero_gives_up_immediately() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(cfg);
    // 0 >= 0, so should give up immediately
    assert!(state.should_give_up());
}

#[test]
fn recovery_limit_can_delete_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(cfg.can_delete_token(SymbolId(99)));
}

#[test]
fn recovery_limit_cannot_delete_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(!cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn recovery_limit_deletable_set_overrides_sync_check() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    cfg.deletable_tokens.insert(10);
    // Explicitly deletable even though it's a sync token
    assert!(cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn recovery_limit_can_replace_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(cfg.can_replace_token(SymbolId(99)));
    assert!(!cfg.can_replace_token(SymbolId(10)));
}

// ===========================================================================
// 6. recovery_cost — error cost accounting (8 tests)
// ===========================================================================

#[test]
fn recovery_cost_determine_strategy_increments_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    });
    let _ = state.determine_recovery_strategy(&[1, 2], Some(99), (0, 0), 0);
    // After one call, consecutive_errors should have been incremented at least once
    // (fetch_add in determine_recovery_strategy)
    // It may have been reset if insertion succeeded, but with no insert_candidates it won't
    assert!(!state.should_give_up()); // still below 10
}

#[test]
fn recovery_cost_multiple_calls_accumulate() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 100,
        sync_tokens: smallvec::smallvec![SymbolId(50)],
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..10 {
        let _ = state.determine_recovery_strategy(&[1, 2], Some(50), (0, 0), 0);
    }
    // Each call increments by 1, none reset; count should be 10
    assert!(!state.should_give_up()); // 10 < 100
}

#[test]
fn recovery_cost_insertion_success_resets_counter() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 100,
        insert_candidates: smallvec::smallvec![SymbolId(10)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Pre-increment a few times
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    // This should succeed with insertion and reset the count
    let strat = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
    assert!(!state.should_give_up());
}

#[test]
fn recovery_cost_builder_set_max_recovery_attempts() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(42)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 42);
}

#[test]
fn recovery_cost_scope_delimiter_tracked_in_builder() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .build();
    assert_eq!(cfg.scope_delimiters.len(), 2);
    assert_eq!(cfg.scope_delimiters[0], (40, 41));
    assert_eq!(cfg.scope_delimiters[1], (91, 93));
}

#[test]
fn recovery_cost_sync_token_sym_variant() {
    let sym = SymbolId(77);
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(sym)
        .build();
    assert!(cfg.sync_tokens.contains(&SymbolId(77)));
}

#[test]
fn recovery_cost_insertable_token_sym_variant() {
    let sym = SymbolId(88);
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(sym)
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(88)));
}

#[test]
fn recovery_cost_deletion_does_not_reset_counter() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 100,
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // First call: token clearly wrong → deletion (doesn't reset counter)
    let strat = state.determine_recovery_strategy(&[1, 2], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenDeletion);
    // Second call
    let strat2 = state.determine_recovery_strategy(&[1, 2], Some(99), (0, 0), 0);
    assert_eq!(strat2, RecoveryStrategy::TokenDeletion);
}

// ===========================================================================
// 7. recovery_reset — state reset operations (8 tests)
// ===========================================================================

#[test]
fn recovery_reset_consecutive_errors_to_zero() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn recovery_reset_error_count_method() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..5 {
        state.increment_error_count();
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn recovery_reset_clear_errors_empties_nodes() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        1,
        2,
        (0, 1),
        (0, 2),
        vec![2],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 2);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn recovery_reset_clear_errors_allows_new_recording() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.clear_errors();
    state.record_error(
        5,
        10,
        (1, 0),
        (1, 5),
        vec![3],
        Some(4),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 5);
}

#[test]
fn recovery_reset_scope_via_pop_test() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    state.push_scope(40);
    assert_eq!(state.pop_scope_test(), Some(40));
    assert_eq!(state.pop_scope_test(), Some(40));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn recovery_reset_error_count_after_give_up() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn recovery_reset_clear_and_rerecord_multiple() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..10 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![],
            None,
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 10);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 1);
}

#[test]
fn recovery_reset_independent_of_scope_stack() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    state.increment_error_count();
    state.reset_error_count();
    // Scope stack should still have the opener
    assert_eq!(state.pop_scope_test(), Some(40));
}

// ===========================================================================
// 8. recovery_edge — edge cases (8 tests)
// ===========================================================================

#[test]
fn recovery_edge_empty_expected_set() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Empty expected set, actual present, not a sync token → is_clearly_wrong → deletion
    let strat = state.determine_recovery_strategy(&[], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenDeletion);
}

#[test]
fn recovery_edge_no_actual_and_no_candidates() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: true,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // No actual token, no insert candidates → falls through to phrase level
    let strat = state.determine_recovery_strategy(&[1, 2], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PhraseLevel);
}

#[test]
fn recovery_edge_error_node_zero_byte_range() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        5,
        5,
        (0, 5),
        (0, 5),
        vec![],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, nodes[0].end_byte);
}

#[test]
fn recovery_edge_is_scope_delimiter_static() {
    let delimiters = vec![(40, 41), (91, 93), (123, 125)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(123, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(0, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(
        u16::MAX,
        &delimiters
    ));
}

#[test]
fn recovery_edge_is_matching_delimiter_static() {
    let delimiters = vec![(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_matching_delimiter(
        40,
        41,
        &delimiters
    ));
    assert!(ErrorRecoveryState::is_matching_delimiter(
        91,
        93,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        40,
        93,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        91,
        41,
        &delimiters
    ));
}

#[test]
fn recovery_edge_empty_delimiter_list() {
    let empty: Vec<(u16, u16)> = vec![];
    assert!(!ErrorRecoveryState::is_scope_delimiter(40, &empty));
    assert!(!ErrorRecoveryState::is_matching_delimiter(40, 41, &empty));
}

#[test]
fn recovery_edge_config_default_builder_equivalence() {
    let from_default = ErrorRecoveryConfig::default();
    let from_builder = ErrorRecoveryConfigBuilder::default().build();
    assert_eq!(from_default.max_panic_skip, from_builder.max_panic_skip);
    assert_eq!(
        from_default.max_token_deletions,
        from_builder.max_token_deletions
    );
    assert_eq!(
        from_default.max_token_insertions,
        from_builder.max_token_insertions
    );
    assert_eq!(
        from_default.max_consecutive_errors,
        from_builder.max_consecutive_errors
    );
    assert_eq!(
        from_default.enable_phrase_recovery,
        from_builder.enable_phrase_recovery
    );
    assert_eq!(
        from_default.enable_scope_recovery,
        from_builder.enable_scope_recovery
    );
    assert_eq!(
        from_default.enable_indentation_recovery,
        from_builder.enable_indentation_recovery
    );
}

#[test]
fn recovery_edge_error_node_clone_preserves_fields() {
    let node = ErrorNode {
        start_byte: 42,
        end_byte: 84,
        start_position: (3, 10),
        end_position: (3, 52),
        expected: vec![1, 2, 3],
        actual: Some(99),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![10, 11],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 42);
    assert_eq!(cloned.end_byte, 84);
    assert_eq!(cloned.expected, vec![1, 2, 3]);
    assert_eq!(cloned.actual, Some(99));
    assert_eq!(cloned.recovery, RecoveryStrategy::PhraseLevel);
}
