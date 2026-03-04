// Wave 132: Comprehensive tests for error recovery configuration and state
use adze::error_recovery::*;
use adze_ir::SymbolId;

// =====================================================================
// ErrorRecoveryConfigBuilder tests
// =====================================================================

#[test]
fn builder_default() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.max_panic_skip > 0);
    assert!(config.sync_tokens.is_empty());
}

#[test]
fn builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(50).build();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn builder_add_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .build();
    assert_eq!(config.sync_tokens.len(), 2);
}

#[test]
fn builder_add_sync_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(10))
        .build();
    assert_eq!(config.sync_tokens.len(), 1);
    assert_eq!(config.sync_tokens[0], SymbolId(10));
}

#[test]
fn builder_add_insertable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .build();
    assert_eq!(config.insert_candidates.len(), 1);
}

#[test]
fn builder_add_insertable_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(20))
        .build();
    assert_eq!(config.insert_candidates.len(), 1);
}

#[test]
fn builder_add_deletable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(3)
        .build();
    assert!(config.deletable_tokens.contains(&3));
}

#[test]
fn builder_scope_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(config.scope_delimiters.len(), 1);
    assert_eq!(config.scope_delimiters[0], (40, 41));
}

#[test]
fn builder_enable_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(true)
        .build();
    assert!(config.enable_scope_recovery);
}

#[test]
fn builder_enable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .build();
    assert!(config.enable_phrase_recovery);
}

#[test]
fn builder_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(100)
        .build();
    assert_eq!(config.max_consecutive_errors, 100);
}

#[test]
fn builder_chaining() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(3)
        .add_deletable_token(4)
        .add_scope_delimiter(5, 6)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(true)
        .max_consecutive_errors(50)
        .build();
    assert_eq!(config.max_panic_skip, 10);
    assert_eq!(config.sync_tokens.len(), 2);
    assert_eq!(config.insert_candidates.len(), 1);
    assert!(config.deletable_tokens.contains(&4));
    assert_eq!(config.scope_delimiters.len(), 1);
    assert!(config.enable_scope_recovery);
    assert!(config.enable_phrase_recovery);
    assert_eq!(config.max_consecutive_errors, 50);
}

// =====================================================================
// ErrorRecoveryConfig methods
// =====================================================================

#[test]
fn can_delete_token_in_set() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .build();
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn can_delete_non_sync_token_even_if_not_explicitly_deletable() {
    // Tokens are deletable if they're explicitly marked OR if they're not sync tokens
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .add_sync_token_sym(SymbolId(99))
        .build();
    // Non-sync, non-explicit is still deletable (not in sync set)
    assert!(config.can_delete_token(SymbolId(50)));
    // Sync token is NOT deletable unless explicitly added
    assert!(!config.can_delete_token(SymbolId(99)));
}

#[test]
fn can_delete_sync_token_if_explicitly_deletable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(10)
        .add_sync_token_sym(SymbolId(10))
        .build();
    // Explicitly deletable overrides sync
    assert!(config.can_delete_token(SymbolId(10)));
}

// =====================================================================
// ErrorRecoveryState tests
// =====================================================================

#[test]
fn state_new() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let state = ErrorRecoveryState::new(config);
    assert!(!state.should_give_up());
}

#[test]
fn state_error_nodes_initially_empty() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let state = ErrorRecoveryState::new(config);
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_increment_error_count() {
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
fn state_reset_error_count() {
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

#[test]
fn state_reset_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.reset_consecutive_errors();
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_clear_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.clear_errors();
    assert!(!state.should_give_up());
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
        .add_scope_delimiter(40, 41)
        .enable_scope_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(state.pop_scope(41));
}

#[test]
fn state_pop_scope_no_matching() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .enable_scope_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.pop_scope(41));
}

#[test]
fn state_nested_scopes() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .enable_scope_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    state.push_scope(91);
    assert!(state.pop_scope(93));
    assert!(state.pop_scope(41));
}

// =====================================================================
// Static utility functions
// =====================================================================

#[test]
fn is_scope_delimiter_match() {
    let delimiters = vec![(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(91, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(93, &delimiters));
}

#[test]
fn is_scope_delimiter_no_match() {
    let delimiters = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delimiters));
}

#[test]
fn is_matching_delimiter() {
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
fn is_matching_delimiter_wrong_pair() {
    let delimiters = vec![(40, 41), (91, 93)];
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
fn is_matching_delimiter_empty() {
    let delimiters: Vec<(u16, u16)> = vec![];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        40,
        41,
        &delimiters
    ));
}

// =====================================================================
// RecoveryStrategy enum tests
// =====================================================================

#[test]
fn recovery_strategy_panic_mode() {
    let s = RecoveryStrategy::PanicMode;
    let debug = format!("{:?}", s);
    assert!(debug.contains("PanicMode"));
}

// =====================================================================
// Recovery state with record_error
// =====================================================================

#[test]
fn record_error_basic() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        5,
        6,
        (0, 5),
        (0, 6),
        vec![1, 2],
        Some(99),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].start_byte, 5);
    assert_eq!(errors[0].end_byte, 6);
}

#[test]
fn record_multiple_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
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
        5,
        7,
        (0, 5),
        (0, 7),
        vec![2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        10,
        11,
        (1, 0),
        (1, 1),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 3);
}

#[test]
fn clear_errors_removes_all() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
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
        5,
        6,
        (0, 5),
        (0, 6),
        vec![2],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// =====================================================================
// Edge cases
// =====================================================================

#[test]
fn zero_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    let state = ErrorRecoveryState::new(config);
    assert!(state.should_give_up());
}

#[test]
fn many_sync_tokens() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in 0..100 {
        builder = builder.add_sync_token(i);
    }
    let config = builder.build();
    assert_eq!(config.sync_tokens.len(), 100);
}

#[test]
fn many_scope_delimiters() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in 0..50u16 {
        builder = builder.add_scope_delimiter(i * 2, i * 2 + 1);
    }
    let config = builder.build();
    assert_eq!(config.scope_delimiters.len(), 50);
}

#[test]
fn update_recent_tokens() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..100 {
        state.update_recent_tokens(SymbolId(i));
    }
}

#[test]
fn pop_scope_test_with_scope() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .enable_scope_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    let popped = state.pop_scope_test();
    assert!(popped.is_some());
    assert_eq!(popped.unwrap(), 40);
}

#[test]
fn pop_scope_test_empty() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    let popped = state.pop_scope_test();
    assert!(popped.is_none());
}

// =====================================================================
// ErrorNode properties
// =====================================================================

#[test]
fn error_node_with_skipped_tokens() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1, 2, 3],
        Some(99),
        RecoveryStrategy::PanicMode,
        vec![50, 51, 52],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].expected, vec![1, 2, 3]);
    assert_eq!(errors[0].actual, Some(99));
}

#[test]
fn error_node_no_actual_token() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors[0].actual, None);
    assert!(errors[0].expected.is_empty());
}

#[test]
fn determine_recovery_strategy_basic() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(5), (0, 0), 0);
    let _ = strategy;
}
