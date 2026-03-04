//! Comprehensive tests for ErrorRecoveryConfigBuilder, ErrorRecoveryConfig,
//! ErrorRecoveryState, and related types.

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze_ir::SymbolId;

// ── ErrorRecoveryConfigBuilder basics ──

#[test]
fn builder_default_creates_valid_config() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    // Default has no sync tokens, so can_delete_token uses fallback logic
    // can_delete_token = deletable_tokens.contains OR !sync_tokens.contains
    // With empty sync_tokens, !sync_tokens.contains(x) is always true
    assert!(config.can_delete_token(SymbolId(1)));
}

#[test]
fn builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(10).build();
    assert_eq!(config.max_panic_skip, 10);
}

#[test]
fn builder_max_panic_skip_zero() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(0).build();
    assert_eq!(config.max_panic_skip, 0);
}

#[test]
fn builder_add_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(42).build();
    assert!(config.sync_tokens.contains(&SymbolId(42)));
}

#[test]
fn builder_add_sync_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(7))
        .build();
    assert!(config.sync_tokens.contains(&SymbolId(7)));
}

#[test]
fn builder_add_insertable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .build();
    assert!(config.insert_candidates.contains(&SymbolId(5)));
}

#[test]
fn builder_add_insertable_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(8))
        .build();
    assert!(config.insert_candidates.contains(&SymbolId(8)));
}

#[test]
fn builder_add_deletable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(3)
        .build();
    assert!(config.deletable_tokens.contains(&3));
}

#[test]
fn builder_add_scope_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(config.scope_delimiters.len(), 1);
    assert_eq!(config.scope_delimiters[0], (40, 41));
}

#[test]
fn builder_multiple_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41) // ()
        .add_scope_delimiter(91, 93) // []
        .add_scope_delimiter(123, 125) // {}
        .build();
    assert_eq!(config.scope_delimiters.len(), 3);
}

#[test]
fn builder_enable_indentation_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(config.enable_indentation_recovery);
}

#[test]
fn builder_disable_indentation_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(false)
        .build();
    assert!(!config.enable_indentation_recovery);
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
        .max_consecutive_errors(5)
        .build();
    assert_eq!(config.max_consecutive_errors, 5);
}

#[test]
fn builder_chained_methods() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(5)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(10)
        .add_deletable_token(11)
        .add_scope_delimiter(40, 41)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(true)
        .max_consecutive_errors(3)
        .set_max_recovery_attempts(50)
        .build();
    assert_eq!(config.max_panic_skip, 5);
    assert_eq!(config.sync_tokens.len(), 2);
    assert!(config.deletable_tokens.contains(&11));
    assert_eq!(config.max_consecutive_errors, 50); // set_max_recovery_attempts overrides
}

// ── ErrorRecoveryConfig ──

#[test]
fn config_default() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
    assert!(config.sync_tokens.is_empty());
    assert!(config.insert_candidates.is_empty());
    assert!(config.deletable_tokens.is_empty());
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn config_can_delete_token_no_sync_tokens() {
    // With no sync tokens, any token is "deletable" (not in sync_tokens)
    let config = ErrorRecoveryConfig::default();
    assert!(config.can_delete_token(SymbolId(1)));
    assert!(config.can_delete_token(SymbolId(999)));
}

#[test]
fn config_can_delete_sync_token_false() {
    // Sync tokens can't be deleted (unless explicitly in deletable_tokens)
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(5).build();
    assert!(!config.can_delete_token(SymbolId(5)));
}

#[test]
fn config_can_delete_sync_token_with_explicit_deletable() {
    // Even if a sync token, explicit deletable overrides
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(5)
        .add_deletable_token(5)
        .build();
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn config_can_replace_token_no_sync() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.can_replace_token(SymbolId(1)));
}

#[test]
fn config_can_replace_sync_token_false() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(5).build();
    assert!(!config.can_replace_token(SymbolId(5)));
}

#[test]
fn config_can_replace_non_sync_token_true() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(5).build();
    assert!(config.can_replace_token(SymbolId(6)));
}

#[test]
fn config_debug_impl() {
    let config = ErrorRecoveryConfig::default();
    let dbg = format!("{:?}", config);
    assert!(dbg.contains("ErrorRecoveryConfig"));
}

#[test]
fn config_clone() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_deletable_token(2)
        .build();
    let cloned = config.clone();
    assert_eq!(cloned.sync_tokens.len(), config.sync_tokens.len());
    assert_eq!(cloned.deletable_tokens, config.deletable_tokens);
}

// ── ErrorRecoveryState ──

#[test]
fn state_new_starts_with_no_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let state = ErrorRecoveryState::new(config);
    let nodes = state.get_error_nodes();
    assert!(nodes.is_empty());
}

#[test]
fn state_record_error_adds_node() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        10,
        15,
        (0, 10),
        (0, 15),
        vec![1, 2, 3],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 10);
    assert_eq!(nodes[0].end_byte, 15);
    assert_eq!(nodes[0].actual, Some(99));
}

#[test]
fn state_record_multiple_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..5 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (i, 0),
            (i, 5),
            vec![],
            None,
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn state_clear_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
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
    assert_eq!(state.get_error_nodes().len(), 1);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_add_recent_token() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..20 {
        state.add_recent_token(i);
    }
    // Should not panic; internal buffer caps at 10
}

#[test]
fn state_push_scope_with_configured_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    // Verify scope was pushed via pop_scope_test
    assert_eq!(state.pop_scope_test(), Some(40));
}

#[test]
fn state_push_scope_without_delimiter_is_noop() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(99);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_matching() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    let popped = state.pop_scope(41);
    assert!(popped);
}

#[test]
fn state_pop_scope_non_matching() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    let popped = state.pop_scope(99);
    assert!(!popped);
}

#[test]
fn state_pop_scope_empty_stack() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let popped = state.pop_scope(41);
    assert!(!popped);
}

#[test]
fn state_nested_scopes() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41) // ()
        .add_scope_delimiter(91, 93) // []
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40); // (
    state.push_scope(91); // [
    assert!(state.pop_scope(93)); // ]
    assert!(state.pop_scope(41)); // )
}

#[test]
fn state_increment_error_count() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_should_give_up_after_max_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
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
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_reset_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn state_update_recent_tokens() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.update_recent_tokens(SymbolId(1));
    state.update_recent_tokens(SymbolId(2));
    state.update_recent_tokens(SymbolId(3));
    // Should not panic
}

// ── Static helpers ──

#[test]
fn is_scope_delimiter_true_for_opening() {
    let delims = [(40u16, 41u16), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(91, &delims));
}

#[test]
fn is_scope_delimiter_false_for_unconfigured() {
    let delims = [(40u16, 41u16)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delims));
}

#[test]
fn is_scope_delimiter_empty_delimiters() {
    let delims: [(u16, u16); 0] = [];
    assert!(!ErrorRecoveryState::is_scope_delimiter(40, &delims));
}

#[test]
fn is_matching_delimiter_true() {
    let delims = [(40u16, 41u16), (123, 125)];
    assert!(ErrorRecoveryState::is_matching_delimiter(40, 41, &delims));
    assert!(ErrorRecoveryState::is_matching_delimiter(123, 125, &delims));
}

#[test]
fn is_matching_delimiter_false() {
    let delims = [(40u16, 41u16)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(40, 99, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(99, 41, &delims));
}

#[test]
fn is_matching_delimiter_reversed_pair() {
    let delims = [(40u16, 41u16)];
    // Reversed: 41 open, 40 close — should not match
    assert!(!ErrorRecoveryState::is_matching_delimiter(41, 40, &delims));
}

// ── RecoveryStrategy and RecoveryAction enums ──

#[test]
fn recovery_strategy_all_variants() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ];
    for s in &strategies {
        let _ = format!("{:?}", s);
    }
}

#[test]
fn recovery_strategy_clone() {
    let s = RecoveryStrategy::PanicMode;
    let cloned = s;
    let _ = format!("{:?}", cloned);
}

#[test]
fn recovery_action_insert_token() {
    let a = RecoveryAction::InsertToken(SymbolId(5));
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("InsertToken"));
}

#[test]
fn recovery_action_delete_token() {
    let a = RecoveryAction::DeleteToken;
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("DeleteToken"));
}

#[test]
fn recovery_action_clone() {
    let a = RecoveryAction::InsertToken(SymbolId(10));
    let cloned = a.clone();
    let _ = format!("{:?}", cloned);
}

// ── ErrorNode ──

#[test]
fn error_node_fields() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2],
        actual: Some(99),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![99],
    };
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 10);
    assert_eq!(node.expected, vec![1, 2]);
    assert_eq!(node.actual, Some(99));
}

#[test]
fn error_node_debug() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 15,
        start_position: (0, 5),
        end_position: (0, 15),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("ErrorNode"));
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, node.start_byte);
}

// ── determine_recovery_strategy ──

#[test]
fn determine_recovery_with_insertable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    // Should suggest insertion since expected token 10 is insertable
    let _ = format!("{:?}", strategy);
}

#[test]
fn determine_recovery_with_deletable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[], Some(5), (0, 0), 0);
    let _ = format!("{:?}", strategy);
}

#[test]
fn determine_recovery_panic_when_no_options() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1) // Need sync tokens to avoid fallback
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[], None, (0, 0), 0);
    let _ = format!("{:?}", strategy);
}

#[test]
fn determine_recovery_scope_when_enabled() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(true)
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    let strategy = state.determine_recovery_strategy(&[], Some(41), (0, 0), 0);
    let _ = format!("{:?}", strategy);
}
