use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze_ir::SymbolId;

// ---------- ErrorRecoveryConfigBuilder tests ----------

#[test]
fn test_builder_default_creates_sane_defaults() {
    let config = ErrorRecoveryConfigBuilder::new().build();

    assert_eq!(config.max_panic_skip, 50);
    assert!(config.sync_tokens.is_empty());
    assert!(config.insert_candidates.is_empty());
    assert!(config.deletable_tokens.is_empty());
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(config.scope_delimiters.is_empty());
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn test_builder_max_panic_skip_and_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .add_sync_token(10)
        .add_sync_token(20)
        .add_sync_token(30)
        .build();

    assert_eq!(config.max_panic_skip, 200);
    assert_eq!(config.sync_tokens.len(), 3);
    assert!(config.sync_tokens.iter().any(|t| t.0 == 10));
    assert!(config.sync_tokens.iter().any(|t| t.0 == 20));
    assert!(config.sync_tokens.iter().any(|t| t.0 == 30));
}

#[test]
fn test_builder_insertable_and_deletable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .add_insertable_token(6)
        .add_deletable_token(7)
        .add_deletable_token(8)
        .build();

    assert_eq!(config.insert_candidates.len(), 2);
    assert!(config.insert_candidates.iter().any(|t| t.0 == 5));
    assert!(config.insert_candidates.iter().any(|t| t.0 == 6));
    assert!(config.deletable_tokens.contains(&7));
    assert!(config.deletable_tokens.contains(&8));
}

#[test]
fn test_builder_scope_delimiters_and_feature_flags() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41) // ( )
        .add_scope_delimiter(60, 62) // { }
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .enable_indentation_recovery(true)
        .build();

    assert_eq!(config.scope_delimiters, vec![(40, 41), (60, 62)]);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_phrase_recovery);
    assert!(config.enable_indentation_recovery);
}

#[test]
fn test_builder_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();

    assert_eq!(config.max_consecutive_errors, 5);
}

// ---------- ErrorRecoveryState creation and lifecycle ----------

#[test]
fn test_state_new_starts_with_zero_errors() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);

    assert!(!state.should_give_up());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_state_increment_and_reset_error_count() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());

    state.increment_error_count();
    assert!(state.should_give_up());

    state.reset_error_count();
    assert!(!state.should_give_up());
}

// ---------- RecoveryStrategy enum variants ----------

#[test]
fn test_recovery_strategy_variants_are_distinct() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ];

    // Every variant must be equal to itself and different from the others.
    for (i, a) in variants.iter().enumerate() {
        assert_eq!(a, a);
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "variants at {i} and {j} should differ");
            }
        }
    }
}

#[test]
fn test_recovery_action_insert_and_delete() {
    let insert = RecoveryAction::InsertToken(SymbolId(42));
    assert!(matches!(insert, RecoveryAction::InsertToken(id) if id == SymbolId(42)));

    let delete = RecoveryAction::DeleteToken;
    assert!(matches!(delete, RecoveryAction::DeleteToken));
}

// ---------- Error tracking via record_error ----------

#[test]
fn test_record_error_stores_error_nodes() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1, 2],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![99],
    );

    state.record_error(
        10,
        15,
        (1, 0),
        (1, 5),
        vec![3],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 2);

    assert_eq!(errors[0].start_byte, 0);
    assert_eq!(errors[0].end_byte, 5);
    assert_eq!(errors[0].expected, vec![1, 2]);
    assert_eq!(errors[0].actual, Some(99));
    assert_eq!(errors[0].recovery, RecoveryStrategy::TokenDeletion);

    assert_eq!(errors[1].start_byte, 10);
    assert_eq!(errors[1].actual, None);
    assert_eq!(errors[1].recovery, RecoveryStrategy::TokenInsertion);
}

// ---------- determine_recovery_strategy ----------

#[test]
fn test_determine_strategy_token_insertion_when_insertable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    // Expected list includes insertable token 10, actual is None → TokenInsertion.
    let strategy = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn test_determine_strategy_panic_mode_after_exceeding_max_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    // Exhaust the error budget.
    let _ = state.determine_recovery_strategy(&[50], Some(60), (0, 0), 0);
    let _ = state.determine_recovery_strategy(&[50], Some(60), (0, 0), 5);

    // Third call exceeds max_consecutive_errors(2).
    let strategy = state.determine_recovery_strategy(&[50], Some(60), (0, 0), 10);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn test_determine_strategy_phrase_level_as_fallback() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .add_sync_token(60) // Mark actual token as sync so deletion is skipped
        .build();
    let mut state = ErrorRecoveryState::new(config);

    // actual token is a sync token → not "clearly wrong" → skips deletion → PhraseLevel.
    let strategy = state.determine_recovery_strategy(&[50, 51], Some(60), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

// ---------- Scope tracking ----------

#[test]
fn test_scope_push_pop_lifecycle() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2) // ( )
        .add_scope_delimiter(3, 4) // [ ]
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.push_scope(1);
    state.push_scope(3);

    // Pop matching close for [ ]
    assert!(state.pop_scope(4));
    // Pop matching close for ( )
    assert!(state.pop_scope(2));

    // Stack is empty, non-matching pop fails.
    assert!(!state.pop_scope(2));
}

// ---------- Config helper methods ----------

#[test]
fn test_config_can_delete_and_replace_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(10)
        .add_deletable_token(20)
        .build();

    // Sync tokens cannot be deleted (unless also explicitly deletable).
    assert!(!config.can_delete_token(SymbolId(10)));

    // Explicitly deletable tokens can be deleted.
    assert!(config.can_delete_token(SymbolId(20)));

    // Non-sync, non-deletable tokens can still be deleted (not a sync token).
    assert!(config.can_delete_token(SymbolId(99)));

    // Sync tokens cannot be replaced.
    assert!(!config.can_replace_token(SymbolId(10)));

    // Non-sync tokens can be replaced.
    assert!(config.can_replace_token(SymbolId(99)));
}

// ---------- Static helpers ----------

#[test]
fn test_static_delimiter_helpers() {
    let delimiters = vec![(1, 2), (3, 4)];

    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(5, &delimiters));

    assert!(ErrorRecoveryState::is_matching_delimiter(1, 2, &delimiters));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        1,
        4,
        &delimiters
    ));
}
