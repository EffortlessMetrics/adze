//! Error recovery v9 — 85 comprehensive tests for error recovery mechanisms.
//!
//! Covers: default config, builder patterns, state management, scope tracking,
//! strategy variants, error nodes, reset operations, and edge cases.

use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};

use ir::SymbolId;

// ============================================================================
// 1. Default config values
// ============================================================================

#[test]
fn test_v9_default_config_max_panic_skip() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn test_v9_default_config_max_consecutive_errors() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn test_v9_default_config_max_token_deletions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_deletions, 3);
}

#[test]
fn test_v9_default_config_max_token_insertions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_insertions, 2);
}

#[test]
fn test_v9_default_config_phrase_recovery_enabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.enable_phrase_recovery);
}

#[test]
fn test_v9_default_config_scope_recovery_enabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.enable_scope_recovery);
}

#[test]
fn test_v9_default_config_indentation_recovery_disabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn test_v9_default_config_empty_sync_tokens() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.sync_tokens.is_empty());
}

#[test]
fn test_v9_default_config_empty_insert_candidates() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.insert_candidates.is_empty());
}

#[test]
fn test_v9_default_config_empty_deletable_tokens() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.deletable_tokens.is_empty());
}

#[test]
fn test_v9_default_config_empty_scope_delimiters() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.scope_delimiters.is_empty());
}

// ============================================================================
// 2. Builder — max_panic_skip
// ============================================================================

#[test]
fn test_v9_builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .build();
    assert_eq!(config.max_panic_skip, 100);
}

#[test]
fn test_v9_builder_max_panic_skip_zero() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(0).build();
    assert_eq!(config.max_panic_skip, 0);
}

// ============================================================================
// 3. Builder — add strategies (sync tokens, insertable, deletable)
// ============================================================================

#[test]
fn test_v9_builder_add_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(42).build();
    assert!(config.sync_tokens.iter().any(|t| t.0 == 42));
}

#[test]
fn test_v9_builder_add_sync_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(99))
        .build();
    assert!(config.sync_tokens.contains(&SymbolId(99)));
}

#[test]
fn test_v9_builder_add_insertable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .build();
    assert!(config.insert_candidates.iter().any(|t| t.0 == 7));
}

#[test]
fn test_v9_builder_add_insertable_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(88))
        .build();
    assert!(config.insert_candidates.contains(&SymbolId(88)));
}

#[test]
fn test_v9_builder_add_deletable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(15)
        .build();
    assert!(config.deletable_tokens.contains(&15));
}

#[test]
fn test_v9_builder_multiple_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token(3)
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
}

// ============================================================================
// 4. Builder — scope delimiters
// ============================================================================

#[test]
fn test_v9_builder_add_scope_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(config.scope_delimiters, vec![(40, 41)]);
}

#[test]
fn test_v9_builder_multiple_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .add_scope_delimiter(123, 125)
        .build();
    assert_eq!(config.scope_delimiters.len(), 3);
    assert_eq!(config.scope_delimiters[0], (40, 41));
    assert_eq!(config.scope_delimiters[1], (91, 93));
    assert_eq!(config.scope_delimiters[2], (123, 125));
}

// ============================================================================
// 5. Builder — enable/disable flags and max_consecutive_errors
// ============================================================================

#[test]
fn test_v9_builder_enable_indentation_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(config.enable_indentation_recovery);
}

#[test]
fn test_v9_builder_disable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!config.enable_phrase_recovery);
}

#[test]
fn test_v9_builder_disable_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!config.enable_scope_recovery);
}

#[test]
fn test_v9_builder_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(25)
        .build();
    assert_eq!(config.max_consecutive_errors, 25);
}

#[test]
fn test_v9_builder_set_max_recovery_attempts() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(5)
        .build();
    assert_eq!(config.max_consecutive_errors, 5);
}

// ============================================================================
// 6. State — initial error count is 0
// ============================================================================

#[test]
fn test_v9_state_initial_error_count_zero() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);
    assert!(!state.should_give_up());
}

#[test]
fn test_v9_state_initial_scope_stack_empty() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_v9_state_initial_error_nodes_empty() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);
    assert!(state.get_error_nodes().is_empty());
}

// ============================================================================
// 7. increment_error_count increments count
// ============================================================================

#[test]
fn test_v9_increment_error_count_once() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn test_v9_increment_error_count_to_limit() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn test_v9_increment_error_count_beyond_limit() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

// ============================================================================
// 8. push_scope with registered delimiter works
// ============================================================================

#[test]
fn test_v9_push_scope_registered_delimiter() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), Some(10));
}

#[test]
fn test_v9_push_scope_multiple_registered() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    state.push_scope(20);
    assert_eq!(state.pop_scope_test(), Some(20));
    assert_eq!(state.pop_scope_test(), Some(10));
}

// ============================================================================
// 9. push_scope with unregistered delimiter → ignored
// ============================================================================

#[test]
fn test_v9_push_scope_unregistered_delimiter_ignored() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(99);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_v9_push_scope_close_delimiter_ignored() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // Pushing the close delimiter should be ignored
    state.push_scope(11);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_v9_push_scope_empty_delimiters_ignored() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), None);
}

// ============================================================================
// 10. pop_scope returns matching result
// ============================================================================

#[test]
fn test_v9_pop_scope_matching_close_returns_true() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    assert!(state.pop_scope(11));
}

#[test]
fn test_v9_pop_scope_mismatched_close_returns_false() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    assert!(!state.pop_scope(21));
}

// ============================================================================
// 11. pop_scope on empty → None / false
// ============================================================================

#[test]
fn test_v9_pop_scope_empty_returns_false() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.pop_scope(11));
}

#[test]
fn test_v9_pop_scope_test_empty_returns_none() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    assert_eq!(state.pop_scope_test(), None);
}

// ============================================================================
// 12. reset clears error count
// ============================================================================

#[test]
fn test_v9_reset_error_count_clears() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn test_v9_reset_consecutive_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..8 {
        state.increment_error_count();
    }
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn test_v9_clear_errors_removes_nodes() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
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
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// ============================================================================
// 13. Multiple recovery attempts
// ============================================================================

#[test]
fn test_v9_multiple_increments_tracked() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 100,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..50 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn test_v9_increment_then_reset_then_increment() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    state.reset_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn test_v9_multiple_record_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..5 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (i, 0),
            (i, 5),
            vec![1],
            Some(2),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

// ============================================================================
// 14. All 7 strategy variants work
// ============================================================================

#[test]
fn test_v9_strategy_panic_mode_eq() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
}

#[test]
fn test_v9_strategy_token_insertion_eq() {
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn test_v9_strategy_token_deletion_eq() {
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn test_v9_strategy_token_substitution_eq() {
    assert_eq!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn test_v9_strategy_phrase_level_eq() {
    assert_eq!(RecoveryStrategy::PhraseLevel, RecoveryStrategy::PhraseLevel);
}

#[test]
fn test_v9_strategy_scope_recovery_eq() {
    assert_eq!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn test_v9_strategy_indentation_recovery_eq() {
    assert_eq!(
        RecoveryStrategy::IndentationRecovery,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn test_v9_all_strategies_distinct() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for i in 0..strategies.len() {
        for j in (i + 1)..strategies.len() {
            assert_ne!(strategies[i], strategies[j]);
        }
    }
}

// ============================================================================
// 15. ErrorNode creation and access
// ============================================================================

#[test]
fn test_v9_error_node_fields() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 20,
        start_position: (1, 5),
        end_position: (1, 15),
        expected: vec![3, 4, 5],
        actual: Some(99),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![99],
    };
    assert_eq!(node.start_byte, 10);
    assert_eq!(node.end_byte, 20);
    assert_eq!(node.expected, vec![3, 4, 5]);
    assert_eq!(node.actual, Some(99));
    assert_eq!(node.recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn test_v9_error_node_no_actual() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![1],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert_eq!(node.actual, None);
}

#[test]
fn test_v9_error_node_clone() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 10,
        start_position: (0, 5),
        end_position: (0, 10),
        expected: vec![1, 2],
        actual: Some(3),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![3, 4],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, node.start_byte);
    assert_eq!(cloned.end_byte, node.end_byte);
    assert_eq!(cloned.expected, node.expected);
}

#[test]
fn test_v9_error_node_debug_format() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![],
    };
    let debug_str = format!("{node:?}");
    assert!(!debug_str.is_empty());
}

// ============================================================================
// 16. Config with multiple delimiters
// ============================================================================

#[test]
fn test_v9_config_multiple_delimiters_direct() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41), (91, 93), (123, 125)],
        ..Default::default()
    };
    assert_eq!(config.scope_delimiters.len(), 3);
}

#[test]
fn test_v9_is_scope_delimiter_open() {
    let delims = vec![(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(91, &delims));
}

#[test]
fn test_v9_is_scope_delimiter_close() {
    let delims = vec![(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(93, &delims));
}

#[test]
fn test_v9_is_scope_delimiter_not_found() {
    let delims = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(50, &delims));
}

#[test]
fn test_v9_is_matching_delimiter_correct() {
    let delims = vec![(40, 41), (91, 93), (123, 125)];
    assert!(ErrorRecoveryState::is_matching_delimiter(40, 41, &delims));
    assert!(ErrorRecoveryState::is_matching_delimiter(123, 125, &delims));
}

#[test]
fn test_v9_is_matching_delimiter_wrong_pair() {
    let delims = vec![(40, 41), (91, 93)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(40, 93, &delims));
}

#[test]
fn test_v9_is_matching_delimiter_reversed() {
    let delims = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(41, 40, &delims));
}

// ============================================================================
// 17. State with multiple scopes
// ============================================================================

#[test]
fn test_v9_nested_scopes() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    state.push_scope(20);
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), Some(20));
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_v9_scope_pop_with_close_token() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    state.push_scope(20);
    assert!(state.pop_scope(21));
    assert!(state.pop_scope(11));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_v9_scope_deep_nesting_same_delimiter() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..10 {
        state.push_scope(10);
    }
    for _ in 0..10 {
        assert!(state.pop_scope(11));
    }
    assert!(!state.pop_scope(11));
}

// ============================================================================
// 18. Recovery until max attempts exceeded
// ============================================================================

#[test]
fn test_v9_should_give_up_at_boundary() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn test_v9_should_give_up_zero_limit() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(config);
    assert!(state.should_give_up());
}

#[test]
fn test_v9_determine_strategy_exceeds_max() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // First call consumes the one allowed error
    let _ = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    // Second call should exceed max
    let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// ============================================================================
// 19. can_delete_token / can_replace_token
// ============================================================================

#[test]
fn test_v9_can_delete_nonsync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn test_v9_cannot_delete_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    assert!(!config.can_delete_token(SymbolId(10)));
}

#[test]
fn test_v9_can_delete_explicitly_deletable_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    config.deletable_tokens.insert(10);
    assert!(config.can_delete_token(SymbolId(10)));
}

#[test]
fn test_v9_can_replace_nonsync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    assert!(config.can_replace_token(SymbolId(5)));
}

#[test]
fn test_v9_cannot_replace_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    assert!(!config.can_replace_token(SymbolId(10)));
}

// ============================================================================
// 20. Builder chain patterns
// ============================================================================

#[test]
fn test_v9_builder_full_chain() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(3)
        .add_deletable_token(4)
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .enable_indentation_recovery(false)
        .max_consecutive_errors(50)
        .build();
    assert_eq!(config.max_panic_skip, 200);
    assert_eq!(config.sync_tokens.len(), 2);
    assert_eq!(config.insert_candidates.len(), 1);
    assert!(config.deletable_tokens.contains(&4));
    assert_eq!(config.scope_delimiters.len(), 2);
    assert_eq!(config.max_consecutive_errors, 50);
}

#[test]
fn test_v9_builder_default_creates_valid_config() {
    let config = ErrorRecoveryConfigBuilder::default().build();
    assert_eq!(config.max_panic_skip, 50);
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn test_v9_builder_override_defaults() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(1)
        .max_consecutive_errors(1)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    assert_eq!(config.max_panic_skip, 1);
    assert_eq!(config.max_consecutive_errors, 1);
    assert!(!config.enable_phrase_recovery);
    assert!(!config.enable_scope_recovery);
}

// ============================================================================
// Additional: recent tokens tracking
// ============================================================================

#[test]
fn test_v9_add_recent_token() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.add_recent_token(42);
    state.add_recent_token(43);
    // Verify via update_recent_tokens which delegates to add_recent_token
    state.update_recent_tokens(SymbolId(44));
    // No panic means success — internal state is private
}

#[test]
fn test_v9_recent_tokens_max_capacity() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..20 {
        state.add_recent_token(i);
    }
    // Internal capacity is 10 — no way to observe directly, but no panic
}

// ============================================================================
// Additional: record_error and get_error_nodes
// ============================================================================

#[test]
fn test_v9_record_and_retrieve_error() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        100,
        200,
        (5, 0),
        (5, 100),
        vec![10, 20, 30],
        Some(99),
        RecoveryStrategy::ScopeRecovery,
        vec![99, 98],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 100);
    assert_eq!(nodes[0].end_byte, 200);
    assert_eq!(nodes[0].expected, vec![10, 20, 30]);
    assert_eq!(nodes[0].actual, Some(99));
    assert_eq!(nodes[0].recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn test_v9_record_multiple_errors_preserves_order() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        10,
        15,
        (1, 0),
        (1, 5),
        vec![2],
        Some(3),
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    state.record_error(
        20,
        25,
        (2, 0),
        (2, 5),
        vec![4],
        Some(5),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[1].start_byte, 10);
    assert_eq!(nodes[2].start_byte, 20);
}

// ============================================================================
// Additional: determine_recovery_strategy scenarios
// ============================================================================

#[test]
fn test_v9_determine_strategy_insertion_when_insertable() {
    let mut config = ErrorRecoveryConfig::default();
    config.insert_candidates.push(SymbolId(10));
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn test_v9_determine_strategy_token_deletion_for_wrong_token() {
    let config = ErrorRecoveryConfig {
        enable_phrase_recovery: true,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // Token 2 is not in expected [1] and is not a sync token → deletion
    let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn test_v9_determine_strategy_deletion_when_recovery_disabled() {
    let config = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // Token 2 is clearly wrong (not expected, not sync) → deletion
    let strategy = state.determine_recovery_strategy(&[], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

// ============================================================================
// Additional: config clone
// ============================================================================

#[test]
fn test_v9_config_clone_preserves_fields() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 77,
        max_consecutive_errors: 33,
        scope_delimiters: vec![(1, 2), (3, 4)],
        enable_indentation_recovery: true,
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.max_panic_skip, 77);
    assert_eq!(cloned.max_consecutive_errors, 33);
    assert_eq!(cloned.scope_delimiters.len(), 2);
    assert!(cloned.enable_indentation_recovery);
}

// ============================================================================
// Additional: strategy debug/clone
// ============================================================================

#[test]
fn test_v9_strategy_debug_format() {
    let strategy = RecoveryStrategy::TokenSubstitution;
    let debug_str = format!("{strategy:?}");
    assert!(debug_str.contains("TokenSubstitution"));
}

#[test]
fn test_v9_strategy_copy_semantics() {
    let s1 = RecoveryStrategy::PanicMode;
    let s2 = s1; // Copy
    assert_eq!(s1, s2);
}

// ============================================================================
// Additional: combined scope and error operations
// ============================================================================

#[test]
fn test_v9_scope_and_error_combined() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    assert!(state.pop_scope(11));
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn test_v9_error_node_with_empty_expected() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 0,
        start_position: (0, 0),
        end_position: (0, 0),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert!(node.expected.is_empty());
    assert_eq!(node.actual, None);
}

#[test]
fn test_v9_state_from_builder_config() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(7)
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(state.pop_scope(41));
    for _ in 0..7 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn test_v9_record_error_with_all_strategies() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for (i, strategy) in strategies.iter().enumerate() {
        state.record_error(
            i,
            i + 1,
            (i, 0),
            (i, 1),
            vec![1],
            Some(2),
            *strategy,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 7);
    for (i, strategy) in strategies.iter().enumerate() {
        assert_eq!(nodes[i].recovery, *strategy);
    }
}
