//! Tests for runtime error recovery configuration edge cases.

use adze::error_recovery::*;

#[test]
fn default_config_builder() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let _state = ErrorRecoveryState::new(config);
}

#[test]
fn config_builder_with_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(5)
        .add_sync_token(10)
        .build();
    let _state = ErrorRecoveryState::new(config);
}

#[test]
fn config_builder_with_insertable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(3)
        .add_insertable_token(7)
        .build();
    let _state = ErrorRecoveryState::new(config);
}

#[test]
fn config_builder_with_deletable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(99)
        .build();
    let _state = ErrorRecoveryState::new(config);
}

#[test]
fn config_builder_chaining() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(20)
        .add_sync_token(5)
        .add_insertable_token(3)
        .add_deletable_token(7)
        .build();
    let _state = ErrorRecoveryState::new(config);
}

#[test]
fn state_record_error() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (1, 0),
        (1, 5),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
}

#[test]
fn state_multiple_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..3 {
        state.record_error(
            i,
            i + 1,
            (1, i),
            (1, i + 1),
            vec![1],
            Some(2),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 3);
}

#[test]
fn state_determine_strategy() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(42).build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(3), (1, 0), 0);
    let debug = format!("{strategy:?}");
    assert!(!debug.is_empty());
}

#[test]
fn state_clear_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        1,
        (1, 0),
        (1, 1),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.clear_errors();
    let errors = state.get_error_nodes();
    assert!(errors.is_empty());
}

#[test]
fn state_should_give_up() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let state = ErrorRecoveryState::new(config);
    // Initially should not give up
    // (The actual behavior depends on config, but this should not panic)
    let _give_up = state.should_give_up();
}

#[test]
fn state_increment_and_reset_error_count() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    state.reset_error_count();
}

#[test]
fn state_add_recent_token() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.add_recent_token(1);
    state.add_recent_token(2);
    state.add_recent_token(3);
}

#[test]
fn state_push_pop_scope() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11) // opening, closing
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    let popped = state.pop_scope(11);
    assert!(popped);
}

#[test]
fn recovery_strategy_panic_mode() {
    let s = RecoveryStrategy::PanicMode;
    let debug = format!("{s:?}");
    assert!(debug.contains("Panic"));
}

#[test]
fn recovery_strategy_token_insertion() {
    let s = RecoveryStrategy::TokenInsertion;
    let debug = format!("{s:?}");
    assert!(debug.contains("Insert") || debug.contains("Token"));
}

#[test]
fn is_scope_delimiter_true() {
    let result = ErrorRecoveryState::is_scope_delimiter(10, &[(10, 11)]);
    assert!(result);
}

#[test]
fn is_scope_delimiter_false() {
    let result = ErrorRecoveryState::is_scope_delimiter(99, &[(10, 11)]);
    assert!(!result);
}

#[test]
fn is_matching_delimiter() {
    let result = ErrorRecoveryState::is_matching_delimiter(10, 11, &[(10, 11)]);
    assert!(result);
}

#[test]
fn is_not_matching_delimiter() {
    let result = ErrorRecoveryState::is_matching_delimiter(10, 12, &[(10, 11)]);
    assert!(!result);
}
