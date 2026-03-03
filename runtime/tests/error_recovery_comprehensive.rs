//! Comprehensive tests for the error recovery system.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};

// ── Builder tests ───────────────────────────────────────────────────────────

#[test]
fn builder_creates_config_with_defaults() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(config.max_panic_skip, 50);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_indentation_recovery);
    assert!(config.sync_tokens.is_empty());
    assert!(config.insert_candidates.is_empty());
    assert!(config.scope_delimiters.is_empty());
}

#[test]
fn builder_with_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    assert_eq!(config.max_consecutive_errors, 5);
}

#[test]
fn builder_with_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    assert_eq!(config.scope_delimiters.len(), 2);
    assert_eq!(config.scope_delimiters[0], (10, 11));
    assert_eq!(config.scope_delimiters[1], (20, 21));
}

#[test]
fn builder_with_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(config.max_panic_skip, 200);
}

#[test]
fn builder_with_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(42).build();
    assert_eq!(config.sync_tokens.len(), 1);
    assert!(config.sync_tokens.iter().any(|t| t.0 == 42));
}

#[test]
fn builder_with_insertable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .build();
    assert_eq!(config.insert_candidates.len(), 1);
    assert!(config.insert_candidates.iter().any(|t| t.0 == 7));
}

#[test]
fn builder_with_deletable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(99)
        .build();
    assert!(config.deletable_tokens.contains(&99));
}

#[test]
fn builder_chaining_all_options() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(30)
        .max_consecutive_errors(3)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(4, 5)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    assert_eq!(config.max_panic_skip, 30);
    assert_eq!(config.max_consecutive_errors, 3);
    assert!(!config.enable_phrase_recovery);
    assert!(!config.enable_scope_recovery);
    assert!(config.enable_indentation_recovery);
}

// ── State: initial conditions ───────────────────────────────────────────────

#[test]
fn state_starts_with_zero_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert_eq!(state.get_error_nodes().len(), 0);
}

#[test]
fn state_starts_not_giving_up() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

// ── Recording errors ────────────────────────────────────────────────────────

#[test]
fn recording_error_increments_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 1);
}

#[test]
fn multiple_errors_tracked() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (0, i * 10),
            (0, i * 10 + 5),
            vec![1],
            Some(2),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn reset_clears_consecutive_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn clear_errors_removes_all_error_nodes() {
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
    assert_eq!(state.get_error_nodes().len(), 0);
}

// ── Error node fields ───────────────────────────────────────────────────────

#[test]
fn error_node_preserves_byte_positions() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        42,
        99,
        (3, 10),
        (3, 67),
        vec![5, 6],
        Some(7),
        RecoveryStrategy::TokenSubstitution,
        vec![7],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, 42);
    assert_eq!(nodes[0].end_byte, 99);
}

#[test]
fn error_node_preserves_expected_and_actual() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![10, 20, 30],
        Some(40),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].expected, vec![10, 20, 30]);
    assert_eq!(nodes[0].actual, Some(40));
}

#[test]
fn error_node_with_no_actual_token() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].actual, None);
}

#[test]
fn error_at_different_byte_positions() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        100,
        110,
        (5, 0),
        (5, 10),
        vec![3],
        Some(4),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    state.record_error(
        500,
        505,
        (20, 0),
        (20, 5),
        vec![5],
        Some(6),
        RecoveryStrategy::PhraseLevel,
        vec![],
    );

    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[1].start_byte, 100);
    assert_eq!(nodes[2].start_byte, 500);
}

// ── Recovery strategy variants ──────────────────────────────────────────────

#[test]
fn recovery_strategy_panic_mode() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn recovery_strategy_token_insertion() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn recovery_strategy_token_deletion() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        Some(1),
        RecoveryStrategy::TokenDeletion,
        vec![1],
    );
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn recovery_strategy_phrase_level() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        None,
        RecoveryStrategy::PhraseLevel,
        vec![],
    );
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::PhraseLevel
    );
}

#[test]
fn recovery_strategy_token_substitution() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        Some(2),
        RecoveryStrategy::TokenSubstitution,
        vec![],
    );
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::TokenSubstitution
    );
}

// ── Scope push / pop ────────────────────────────────────────────────────────

#[test]
fn scope_push_and_pop() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.push_scope(10);
    assert!(state.pop_scope(11));
}

#[test]
fn scope_push_non_delimiter_ignored() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.push_scope(99); // not a delimiter
    // pop_scope_test reveals the raw stack; nothing was pushed
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn nested_scope_tracking() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.push_scope(10);
    state.push_scope(20);
    state.push_scope(10);

    // Must pop in LIFO order
    assert!(state.pop_scope(11)); // matches inner 10
    assert!(state.pop_scope(21)); // matches 20
    assert!(state.pop_scope(11)); // matches outer 10
}

#[test]
fn pop_scope_mismatched_delimiter_fails() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.push_scope(10);
    // Trying to pop with wrong closing delimiter
    assert!(!state.pop_scope(21));
}

// ── determine_recovery_strategy ─────────────────────────────────────────────

#[test]
fn strategy_selects_token_insertion_when_insertable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    let strategy = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_selects_token_deletion_for_wrong_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    // actual=50 is not in expected=[1,2] and not a sync token → deletion
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_selects_substitution_when_single_expected() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .add_sync_token(50) // make 50 a sync token so deletion is skipped
        .build();
    let mut state = ErrorRecoveryState::new(config);

    // actual=50 is sync so not deletable; single expected → substitution
    let strategy = state.determine_recovery_strategy(&[1], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn strategy_falls_back_to_phrase_level() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(50).build();
    let mut state = ErrorRecoveryState::new(config);

    // actual=50 is sync (not deletable), multiple expected (not substitutable),
    // no insertable → phrase level
    let strategy = state.determine_recovery_strategy(&[1, 2, 3], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_falls_back_to_panic_when_phrase_disabled() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .add_sync_token(50)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    // Everything disabled, multiple expected → panic mode
    let strategy = state.determine_recovery_strategy(&[1, 2, 3], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// ── Max errors / should_give_up ─────────────────────────────────────────────

#[test]
fn max_errors_limit_triggers_give_up() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn exceeding_max_errors_forces_panic_mode() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    // First call: insertion is available → TokenInsertion (resets counter)
    let s1 = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s1, RecoveryStrategy::TokenInsertion);

    // Artificially exceed the limit
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count(); // now at 3, limit is 2

    let s2 = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s2, RecoveryStrategy::PanicMode);
}

#[test]
fn reset_error_count_prevents_give_up() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());

    state.reset_error_count();
    assert!(!state.should_give_up());
}

// ── Config: can_delete_token / can_replace_token ────────────────────────────

#[test]
fn config_can_delete_non_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(config.can_delete_token(adze_ir::SymbolId(99)));
    assert!(!config.can_delete_token(adze_ir::SymbolId(10)));
}

#[test]
fn config_can_replace_non_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(config.can_replace_token(adze_ir::SymbolId(99)));
    assert!(!config.can_replace_token(adze_ir::SymbolId(10)));
}

// ── Static helpers ──────────────────────────────────────────────────────────

#[test]
fn is_scope_delimiter_detects_open_and_close() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delims));
    assert!(!ErrorRecoveryState::is_scope_delimiter(5, &delims));
}

#[test]
fn is_matching_delimiter_checks_pairs() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_matching_delimiter(1, 2, &delims));
    assert!(ErrorRecoveryState::is_matching_delimiter(3, 4, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 4, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(3, 2, &delims));
}

// ── Default config via Default trait ────────────────────────────────────────

#[test]
fn default_config_matches_builder_defaults() {
    let from_default = ErrorRecoveryConfig::default();
    let from_builder = ErrorRecoveryConfigBuilder::new().build();

    assert_eq!(from_default.max_panic_skip, from_builder.max_panic_skip);
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
