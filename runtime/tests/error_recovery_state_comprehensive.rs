// Wave 133: Comprehensive ErrorRecoveryState and scope/delimiter tests
use adze::error_recovery::*;
use adze_ir::SymbolId;

fn default_config() -> ErrorRecoveryConfig {
    ErrorRecoveryConfigBuilder::new().build()
}

// =====================================================================
// ErrorRecoveryState construction
// =====================================================================

#[test]
fn state_new() {
    let state = ErrorRecoveryState::new(default_config());
    assert!(state.get_error_nodes().is_empty());
}

// =====================================================================
// Error counting
// =====================================================================

#[test]
fn increment_error_count() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn reset_error_count() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn should_give_up_after_many_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..100 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

// =====================================================================
// Scope tracking
// =====================================================================

#[test]
fn push_scope() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.push_scope(40);
}

#[test]
fn pop_scope_matching() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    let ok = state.pop_scope(41);
    assert!(ok);
}

#[test]
fn pop_scope_empty_stack() {
    let mut state = ErrorRecoveryState::new(default_config());
    let ok = state.pop_scope(41);
    assert!(!ok);
}

#[test]
fn pop_scope_test() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    let token = state.pop_scope_test();
    assert_eq!(token, Some(40));
}

#[test]
fn pop_scope_test_empty() {
    let mut state = ErrorRecoveryState::new(default_config());
    let token = state.pop_scope_test();
    assert_eq!(token, None);
}

// =====================================================================
// Recent tokens
// =====================================================================

#[test]
fn add_recent_token() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.add_recent_token(1);
    state.add_recent_token(2);
    state.add_recent_token(3);
}

#[test]
fn update_recent_tokens() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.update_recent_tokens(SymbolId(1));
    state.update_recent_tokens(SymbolId(2));
}

// =====================================================================
// Error recording
// =====================================================================

#[test]
fn record_single_error() {
    let mut state = ErrorRecoveryState::new(default_config());
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
    assert!(!state.get_error_nodes().is_empty());
}

#[test]
fn record_multiple_errors() {
    let mut state = ErrorRecoveryState::new(default_config());
    for i in 0..5u16 {
        state.record_error(
            (i as usize) * 10,
            (i as usize) * 10 + 5,
            (0, (i as usize) * 10),
            (0, (i as usize) * 10 + 5),
            vec![1],
            Some(i),
            RecoveryStrategy::TokenDeletion,
            vec![i],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn clear_errors() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn reset_consecutive_errors() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

// =====================================================================
// Recovery strategy determination
// =====================================================================

#[test]
fn determine_recovery_basic() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(10).build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(3), (0, 0), 0);
    let _ = strategy;
}

#[test]
fn determine_recovery_no_expected() {
    let mut state = ErrorRecoveryState::new(default_config());
    let strategy = state.determine_recovery_strategy(&[], None, (0, 0), 0);
    let _ = strategy;
}

#[test]
fn determine_recovery_repeated_calls() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..10 {
        let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
        let _ = strategy;
    }
}

// =====================================================================
// Static helper functions
// =====================================================================

#[test]
fn is_scope_delimiter_true() {
    let delimiters = vec![(40, 41), (91, 93), (123, 125)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(91, &delimiters));
}

#[test]
fn is_scope_delimiter_false() {
    let delimiters = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(42, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(0, &delimiters));
}

#[test]
fn is_matching_delimiter_true() {
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
}

#[test]
fn is_matching_delimiter_false() {
    let delimiters = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        40,
        93,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_empty() {
    assert!(!ErrorRecoveryState::is_matching_delimiter(40, 41, &[]));
}

// =====================================================================
// ErrorNode properties
// =====================================================================

#[test]
fn error_node_from_record() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.record_error(
        10,
        20,
        (1, 0),
        (1, 10),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::TokenSubstitution,
        vec![3],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];
    assert_eq!(node.start_byte, 10);
    assert_eq!(node.end_byte, 20);
}

// =====================================================================
// Config builder defaults
// =====================================================================

#[test]
fn config_builder_default() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let _ = config;
}

#[test]
fn config_builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(5).build();
    let _ = config;
}

#[test]
fn config_builder_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(10)
        .build();
    let _ = config;
}

#[test]
fn config_builder_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(1))
        .add_sync_token_sym(SymbolId(2))
        .build();
    assert!(!config.can_delete_token(SymbolId(1)));
    assert!(!config.can_delete_token(SymbolId(2)));
}

#[test]
fn config_builder_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let _ = config;
}

// =====================================================================
// RecoveryStrategy enum
// =====================================================================

#[test]
fn recovery_strategy_variants() {
    let _ = RecoveryStrategy::PanicMode;
    let _ = RecoveryStrategy::TokenInsertion;
    let _ = RecoveryStrategy::TokenDeletion;
    let _ = RecoveryStrategy::TokenSubstitution;
    let _ = RecoveryStrategy::PhraseLevel;
    let _ = RecoveryStrategy::ScopeRecovery;
}

#[test]
fn recovery_strategy_debug() {
    assert!(!format!("{:?}", RecoveryStrategy::PanicMode).is_empty());
    assert!(!format!("{:?}", RecoveryStrategy::TokenDeletion).is_empty());
}
