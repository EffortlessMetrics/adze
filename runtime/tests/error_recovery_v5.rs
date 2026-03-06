//! Tests for error recovery configuration and state management (v5).
//!
//! Categories:
//!   1. Config default values
//!   2. Config builder pattern
//!   3. State accumulates errors correctly
//!   4. Error count limits (max_consecutive_errors)
//!   5. Sync token recognition
//!   6. Scope delimiter matching
//!   7. ErrorNode field preservation
//!   8. Edge cases

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};
use adze_ir::SymbolId;

// ===========================================================================
// 1. Config default values (7 tests)
// ===========================================================================

#[test]
fn config_default_max_panic_skip() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn config_default_sync_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn config_default_insert_candidates_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn config_default_deletable_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.deletable_tokens.is_empty());
}

#[test]
fn config_default_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn config_default_phrase_and_scope_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn config_default_indentation_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
}

// ===========================================================================
// 2. Config builder pattern (10 tests)
// ===========================================================================

#[test]
fn builder_default_matches_config_default() {
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    let from_default = ErrorRecoveryConfig::default();
    assert_eq!(from_builder.max_panic_skip, from_default.max_panic_skip);
    assert_eq!(
        from_builder.max_consecutive_errors,
        from_default.max_consecutive_errors
    );
}

#[test]
fn builder_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .build();
    assert_eq!(cfg.max_panic_skip, 100);
}

#[test]
fn builder_add_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(7)
        .add_sync_token(8)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 7));
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 8));
}

#[test]
fn builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(42))
        .build();
    assert!(cfg.sync_tokens.contains(&SymbolId(42)));
}

#[test]
fn builder_add_insertable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(3)
        .build();
    assert!(cfg.insert_candidates.iter().any(|t| t.0 == 3));
}

#[test]
fn builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(99)
        .build();
    assert!(cfg.deletable_tokens.contains(&99));
}

#[test]
fn builder_add_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(10, 11)]);
}

#[test]
fn builder_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_chained_multiple_options() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(25)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(4, 5)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .max_consecutive_errors(5)
        .build();
    assert_eq!(cfg.max_panic_skip, 25);
    assert_eq!(cfg.max_consecutive_errors, 5);
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
}

#[test]
fn builder_set_max_recovery_attempts_aliases_consecutive() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(77)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 77);
}

// ===========================================================================
// 3. State accumulates errors correctly (8 tests)
// ===========================================================================

fn default_state() -> ErrorRecoveryState {
    ErrorRecoveryState::new(ErrorRecoveryConfig::default())
}

#[test]
fn state_starts_with_no_errors() {
    let state = default_state();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_record_single_error() {
    let mut state = default_state();
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 1);
}

#[test]
fn state_record_multiple_errors() {
    let mut state = default_state();
    for i in 0..5 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![10],
            Some(20),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn state_clear_errors() {
    let mut state = default_state();
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
    assert!(!state.get_error_nodes().is_empty());
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_increment_error_count() {
    let mut state = default_state();
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_reset_error_count() {
    let mut state = default_state();
    for _ in 0..8 {
        state.increment_error_count();
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_add_recent_token_within_capacity() {
    let mut state = default_state();
    for i in 0..10 {
        state.add_recent_token(i);
    }
    // Adding one more should evict the oldest
    state.add_recent_token(99);
    // No panic — capacity stays bounded
}

#[test]
fn state_update_recent_tokens_via_symbol() {
    let mut state = default_state();
    state.update_recent_tokens(SymbolId(55));
    // No panic — exercises the SymbolId wrapper path
}

// ===========================================================================
// 4. Error count limits / max_consecutive_errors (6 tests)
// ===========================================================================

#[test]
fn should_give_up_below_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn should_give_up_at_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn should_give_up_above_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn determine_strategy_returns_panic_when_over_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Consume 2 errors
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn reset_consecutive_errors_resets_counter() {
    let mut state = default_state();
    for _ in 0..8 {
        state.increment_error_count();
    }
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn max_consecutive_errors_one_gives_up_after_first() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    assert!(state.should_give_up());
}

// ===========================================================================
// 5. Sync token recognition (5 tests)
// ===========================================================================

#[test]
fn can_delete_non_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(cfg.can_delete_token(SymbolId(99)));
}

#[test]
fn cannot_delete_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(!cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn can_delete_explicitly_deletable_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    cfg.deletable_tokens.insert(10);
    // Explicitly deletable overrides sync token protection
    assert!(cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn can_replace_non_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(20).build();
    assert!(cfg.can_replace_token(SymbolId(5)));
}

#[test]
fn cannot_replace_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(20).build();
    assert!(!cfg.can_replace_token(SymbolId(20)));
}

// ===========================================================================
// 6. Scope delimiter matching (8 tests)
// ===========================================================================

#[test]
fn push_opening_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    // Pop should succeed
    assert!(state.pop_scope(2));
}

#[test]
fn push_non_delimiter_is_ignored() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99);
    // Pop test should give None since 99 was never pushed
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn pop_matching_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(state.pop_scope(11));
}

#[test]
fn pop_mismatched_delimiter_fails() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(!state.pop_scope(21));
}

#[test]
fn nested_scope_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .add_scope_delimiter(3, 4)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    state.push_scope(3);
    assert!(state.pop_scope(4));
    assert!(state.pop_scope(2));
}

#[test]
fn pop_scope_test_returns_top() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(5, 6)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(5);
    assert_eq!(state.pop_scope_test(), Some(5));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn is_scope_delimiter_static() {
    let delims = vec![(1u16, 2u16), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delims));
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delims));
}

#[test]
fn is_matching_delimiter_static() {
    let delims = vec![(1u16, 2u16), (3, 4)];
    assert!(ErrorRecoveryState::is_matching_delimiter(1, 2, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 4, &delims));
    assert!(ErrorRecoveryState::is_matching_delimiter(3, 4, &delims));
}

// ===========================================================================
// 7. ErrorNode field preservation (6 tests)
// ===========================================================================

#[test]
fn error_node_start_byte_preserved() {
    let mut state = default_state();
    state.record_error(
        42,
        50,
        (1, 0),
        (1, 8),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, 42);
}

#[test]
fn error_node_end_byte_preserved() {
    let mut state = default_state();
    state.record_error(
        0,
        99,
        (0, 0),
        (0, 99),
        vec![],
        None,
        RecoveryStrategy::PhraseLevel,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].end_byte, 99);
}

#[test]
fn error_node_positions_preserved() {
    let mut state = default_state();
    state.record_error(
        0,
        5,
        (3, 7),
        (4, 2),
        vec![10],
        Some(20),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_position, (3, 7));
    assert_eq!(nodes[0].end_position, (4, 2));
}

#[test]
fn error_node_expected_symbols_preserved() {
    let mut state = default_state();
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![5, 10, 15],
        Some(99),
        RecoveryStrategy::TokenSubstitution,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].expected, vec![5, 10, 15]);
}

#[test]
fn error_node_actual_none_preserved() {
    let mut state = default_state();
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
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].actual, None);
}

#[test]
fn error_node_skipped_tokens_preserved() {
    let mut state = default_state();
    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![2, 3, 4],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].skipped_tokens, vec![2, 3, 4]);
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn config_zero_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    let state = ErrorRecoveryState::new(cfg);
    assert!(state.should_give_up());
}

#[test]
fn config_empty_sync_tokens_all_deletable() {
    let cfg = ErrorRecoveryConfig::default();
    // With no sync tokens, every token can be deleted
    assert!(cfg.can_delete_token(SymbolId(0)));
    assert!(cfg.can_delete_token(SymbolId(u16::MAX)));
}

#[test]
fn config_empty_scope_delimiters_no_push() {
    let cfg = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    // Nothing pushed because 1 is not a known opening delimiter
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn recovery_strategy_all_variants_are_copy() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for v in variants {
        let copied = v;
        assert_eq!(v, copied);
    }
}

#[test]
fn error_node_with_empty_expected() {
    let mut state = default_state();
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![],
        Some(1),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert!(nodes[0].expected.is_empty());
}

#[test]
fn error_node_zero_byte_span() {
    let mut state = default_state();
    state.record_error(
        5,
        5,
        (1, 3),
        (1, 3),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, nodes[0].end_byte);
}

#[test]
fn builder_default_trait_same_as_new() {
    let from_default = ErrorRecoveryConfigBuilder::default().build();
    let from_new = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(from_default.max_panic_skip, from_new.max_panic_skip);
    assert_eq!(
        from_default.max_consecutive_errors,
        from_new.max_consecutive_errors,
    );
}

#[test]
fn determine_strategy_insertion_preferred_when_available() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[5], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}
