//! Error recovery configuration and state management tests.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};

fn record_simple_error(state: &mut ErrorRecoveryState, start: usize, end: usize) {
    state.record_error(
        start,
        end,
        (0, start),
        (0, end),
        vec![1, 2],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
}

#[test]
fn default_config() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
    assert_eq!(config.max_token_deletions, 3);
    assert_eq!(config.max_token_insertions, 2);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn config_builder() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .build();
    assert_eq!(config.max_panic_skip, 100);
}

#[test]
fn recovery_state_new() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);
    assert!(state.get_error_nodes().is_empty());
    assert!(!state.should_give_up());
}

#[test]
fn recovery_state_record_and_get_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    record_simple_error(&mut state, 0, 5);
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].start_byte, 0);
    assert_eq!(errors[0].end_byte, 5);
}

#[test]
fn recovery_state_clear_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    record_simple_error(&mut state, 0, 5);
    record_simple_error(&mut state, 5, 10);
    assert_eq!(state.get_error_nodes().len(), 2);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn recovery_state_scope_push_pop() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)], // '(' and ')'
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(state.pop_scope(41));
}

#[test]
fn recovery_state_scope_pop_mismatch() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(!state.pop_scope(99));
}

#[test]
fn recovery_state_recent_tokens() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..100 {
        state.add_recent_token(i);
    }
}

#[test]
fn recovery_state_increment_and_reset_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..100 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn recovery_state_reset_consecutive() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    record_simple_error(&mut state, 0, 1);
    state.reset_consecutive_errors();
    record_simple_error(&mut state, 1, 2);
    assert_eq!(state.get_error_nodes().len(), 2);
}

#[test]
fn determine_recovery_strategy() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    let _strategy = state.determine_recovery_strategy(&[1, 2, 3], Some(4), (0, 0), 0);
}

#[test]
fn recovery_state_pop_scope_test() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    assert!(state.pop_scope_test().is_none());
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), Some(10));
    assert!(state.pop_scope_test().is_none());
}

#[test]
fn recovery_state_multiple_scopes() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 4), (2, 5), (3, 6)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(1);
    state.push_scope(2);
    state.push_scope(3);
    assert!(state.pop_scope(6));
    assert!(state.pop_scope(5));
    assert!(state.pop_scope(4));
}

#[test]
fn recovery_strategy_variants() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ];
    for v in &variants {
        let _ = format!("{v:?}");
    }
}
