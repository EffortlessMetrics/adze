//! Comprehensive tests for error recovery types: ErrorNode, RecoveryStrategy,
//! ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, and ErrorRecoveryState.

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze_ir::SymbolId;

// =========================================================================
// 1. ErrorNode construction (8 tests)
// =========================================================================

#[test]
fn test_error_node_basic_construction() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![1, 2],
        actual: Some(3),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![3],
    };
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 5);
    assert_eq!(node.expected, [1, 2]);
    assert_eq!(node.actual, Some(3));
}

#[test]
fn test_error_node_no_actual_token() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 10,
        start_position: (1, 0),
        end_position: (1, 0),
        expected: vec![42],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert!(node.actual.is_none());
    assert_eq!(node.expected, [42]);
}

#[test]
fn test_error_node_empty_expected() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![],
        actual: Some(7),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert!(node.expected.is_empty());
}

#[test]
fn test_error_node_zero_length_span() {
    let node = ErrorNode {
        start_byte: 42,
        end_byte: 42,
        start_position: (3, 10),
        end_position: (3, 10),
        expected: vec![1],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_byte, node.end_byte);
}

#[test]
fn test_error_node_with_skipped_tokens() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 20,
        start_position: (0, 5),
        end_position: (0, 20),
        expected: vec![10],
        actual: Some(99),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![99, 100, 101],
    };
    assert_eq!(node.skipped_tokens.len(), 3);
    assert_eq!(node.skipped_tokens[0], 99);
}

#[test]
fn test_error_node_multiline_position() {
    let node = ErrorNode {
        start_byte: 50,
        end_byte: 80,
        start_position: (2, 10),
        end_position: (4, 3),
        expected: vec![1, 2, 3],
        actual: Some(5),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_position, (2, 10));
    assert_eq!(node.end_position, (4, 3));
}

#[test]
fn test_error_node_clone() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2, 3],
        actual: Some(4),
        recovery: RecoveryStrategy::TokenSubstitution,
        skipped_tokens: vec![4],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 0);
    assert_eq!(cloned.end_byte, 10);
    assert_eq!(cloned.expected, [1, 2, 3]);
    assert_eq!(cloned.actual, Some(4));
}

#[test]
fn test_error_node_debug_format() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("ErrorNode"));
    assert!(debug_str.contains("start_byte"));
}

// =========================================================================
// 2. RecoveryStrategy variants (5 tests)
// =========================================================================

#[test]
fn test_recovery_strategy_all_variants_exist() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    assert_eq!(strategies.len(), 7);
}

#[test]
fn test_recovery_strategy_equality() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
    assert_ne!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn test_recovery_strategy_copy() {
    let s = RecoveryStrategy::PhraseLevel;
    let s2 = s; // Copy
    assert_eq!(s, s2);
}

#[test]
fn test_recovery_strategy_debug() {
    let s = RecoveryStrategy::ScopeRecovery;
    let debug_str = format!("{s:?}");
    assert_eq!(debug_str, "ScopeRecovery");
}

#[test]
fn test_recovery_strategy_pattern_matching() {
    let s = RecoveryStrategy::TokenInsertion;
    let label = match s {
        RecoveryStrategy::PanicMode => "panic",
        RecoveryStrategy::TokenInsertion => "insert",
        RecoveryStrategy::TokenDeletion => "delete",
        RecoveryStrategy::TokenSubstitution => "substitute",
        RecoveryStrategy::PhraseLevel => "phrase",
        RecoveryStrategy::ScopeRecovery => "scope",
        RecoveryStrategy::IndentationRecovery => "indent",
    };
    assert_eq!(label, "insert");
}

// =========================================================================
// 3. RecoveryAction variants (8 tests)
// =========================================================================

#[test]
fn test_recovery_action_insert_token() {
    let action = RecoveryAction::InsertToken(SymbolId(10));
    assert!(matches!(action, RecoveryAction::InsertToken(SymbolId(10))));
}

#[test]
fn test_recovery_action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    assert!(matches!(action, RecoveryAction::DeleteToken));
}

#[test]
fn test_recovery_action_replace_token() {
    let action = RecoveryAction::ReplaceToken(SymbolId(5));
    assert!(matches!(action, RecoveryAction::ReplaceToken(SymbolId(5))));
}

#[test]
fn test_recovery_action_create_error_node() {
    let action = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    if let RecoveryAction::CreateErrorNode(ref ids) = action {
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], SymbolId(1));
    } else {
        panic!("Expected CreateErrorNode");
    }
}

#[test]
fn test_recovery_action_equality() {
    assert_eq!(RecoveryAction::DeleteToken, RecoveryAction::DeleteToken);
    assert_eq!(
        RecoveryAction::InsertToken(SymbolId(1)),
        RecoveryAction::InsertToken(SymbolId(1))
    );
    assert_ne!(
        RecoveryAction::InsertToken(SymbolId(1)),
        RecoveryAction::InsertToken(SymbolId(2))
    );
}

#[test]
fn test_recovery_action_debug() {
    let action = RecoveryAction::InsertToken(SymbolId(42));
    let debug_str = format!("{action:?}");
    assert!(debug_str.contains("InsertToken"));
}

#[test]
fn test_recovery_action_clone() {
    let action = RecoveryAction::CreateErrorNode(vec![SymbolId(1)]);
    let cloned = action.clone();
    assert_eq!(action, cloned);
}

#[test]
fn test_recovery_action_all_variants_distinct() {
    let actions: Vec<RecoveryAction> = vec![
        RecoveryAction::InsertToken(SymbolId(0)),
        RecoveryAction::DeleteToken,
        RecoveryAction::ReplaceToken(SymbolId(0)),
        RecoveryAction::CreateErrorNode(vec![]),
    ];
    for i in 0..actions.len() {
        for j in (i + 1)..actions.len() {
            assert_ne!(actions[i], actions[j]);
        }
    }
}

// =========================================================================
// 4. ErrorRecoveryConfig (8 tests)
// =========================================================================

#[test]
fn test_config_default_values() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
    assert!(config.sync_tokens.is_empty());
    assert!(config.insert_candidates.is_empty());
    assert!(config.deletable_tokens.is_empty());
    assert_eq!(config.max_token_deletions, 3);
    assert_eq!(config.max_token_insertions, 2);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(config.scope_delimiters.is_empty());
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn test_config_can_delete_non_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    // Non-sync token → can delete
    assert!(config.can_delete_token(SymbolId(99)));
}

#[test]
fn test_config_cannot_delete_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    // Sync token → cannot delete (unless explicitly deletable)
    assert!(!config.can_delete_token(SymbolId(10)));
}

#[test]
fn test_config_can_delete_explicitly_deletable_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    config.deletable_tokens.insert(10);
    // Explicitly deletable overrides sync
    assert!(config.can_delete_token(SymbolId(10)));
}

#[test]
fn test_config_can_replace_non_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(20));
    assert!(config.can_replace_token(SymbolId(5)));
}

#[test]
fn test_config_cannot_replace_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(20));
    assert!(!config.can_replace_token(SymbolId(20)));
}

#[test]
fn test_config_clone() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 100,
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.max_panic_skip, 100);
    assert_eq!(cloned.scope_delimiters, [(1, 2)]);
}

#[test]
fn test_config_debug() {
    let config = ErrorRecoveryConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("ErrorRecoveryConfig"));
}

// =========================================================================
// 5. ErrorRecoveryConfigBuilder (5 tests)
// =========================================================================

#[test]
fn test_builder_default_produces_default_config() {
    let built = ErrorRecoveryConfigBuilder::default().build();
    let default_config = ErrorRecoveryConfig::default();
    assert_eq!(built.max_panic_skip, default_config.max_panic_skip);
    assert_eq!(
        built.max_consecutive_errors,
        default_config.max_consecutive_errors
    );
    assert_eq!(
        built.enable_phrase_recovery,
        default_config.enable_phrase_recovery
    );
}

#[test]
fn test_builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(config.max_panic_skip, 200);
}

#[test]
fn test_builder_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(10)
        .add_sync_token(20)
        .add_sync_token_sym(SymbolId(30))
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
    assert!(config.sync_tokens.contains(&SymbolId(10)));
    assert!(config.sync_tokens.contains(&SymbolId(20)));
    assert!(config.sync_tokens.contains(&SymbolId(30)));
}

#[test]
fn test_builder_insertable_and_deletable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .add_insertable_token_sym(SymbolId(6))
        .add_deletable_token(7)
        .build();
    assert_eq!(config.insert_candidates.len(), 2);
    assert!(config.insert_candidates.contains(&SymbolId(5)));
    assert!(config.insert_candidates.contains(&SymbolId(6)));
    assert!(config.deletable_tokens.contains(&7));
}

#[test]
fn test_builder_full_chain() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(75)
        .max_consecutive_errors(5)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(40, 41)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    assert_eq!(config.max_panic_skip, 75);
    assert_eq!(config.max_consecutive_errors, 5);
    assert!(config.sync_tokens.contains(&SymbolId(1)));
    assert!(config.insert_candidates.contains(&SymbolId(2)));
    assert!(config.deletable_tokens.contains(&3));
    assert_eq!(config.scope_delimiters, [(40, 41)]);
    assert!(!config.enable_phrase_recovery);
    assert!(!config.enable_scope_recovery);
    assert!(config.enable_indentation_recovery);
}

// =========================================================================
// 6. ErrorRecoveryState operations (8 tests)
// =========================================================================

#[test]
fn test_state_new_initial_values() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_state_increment_error_count() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.should_give_up());
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn test_state_reset_error_count() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn test_state_should_give_up_below_threshold() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.should_give_up());
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn test_state_should_give_up_at_threshold() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn test_state_should_give_up_above_threshold() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn test_state_add_recent_token() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.add_recent_token(42);
    // Verify via update_recent_tokens which also uses add_recent_token
    state.update_recent_tokens(SymbolId(43));
    // No panic = tokens accepted correctly
}

#[test]
fn test_state_recent_tokens_max_capacity() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    // Add 15 tokens; internal buffer capped at 10
    for i in 0..15 {
        state.add_recent_token(i);
    }
    // No public accessor for recent_tokens, so just verify no panic
    // and that further additions still work
    state.add_recent_token(100);
}

// =========================================================================
// 7. State push, record, and clear (5 tests)
// =========================================================================

#[test]
fn test_state_record_single_error() {
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
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].start_byte, 0);
    assert_eq!(errors[0].end_byte, 5);
    assert_eq!(errors[0].expected, [1, 2]);
    assert_eq!(errors[0].actual, Some(3));
}

#[test]
fn test_state_record_multiple_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (i, 0),
            (i, 5),
            vec![1],
            Some(i as u16),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn test_state_clear_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        1,
        2,
        (0, 1),
        (0, 2),
        vec![3],
        Some(4),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 2);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_state_scope_push_and_pop() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    assert!(state.pop_scope(11));
    // Stack should be empty now; pop_scope_test returns None
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_state_scope_pop_mismatch() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    // Try to pop with wrong closer
    assert!(!state.pop_scope(21));
    // Stack unchanged — original open still there
    assert_eq!(state.pop_scope_test(), Some(10));
}

// =========================================================================
// 8. Edge cases (8 tests)
// =========================================================================

#[test]
fn test_edge_empty_config_delete_any() {
    let config = ErrorRecoveryConfig::default();
    // With no sync tokens, any token can be deleted
    assert!(config.can_delete_token(SymbolId(0)));
    assert!(config.can_delete_token(SymbolId(u16::MAX)));
}

#[test]
fn test_edge_empty_config_replace_any() {
    let config = ErrorRecoveryConfig::default();
    // With no sync tokens, any token can be replaced
    assert!(config.can_replace_token(SymbolId(0)));
    assert!(config.can_replace_token(SymbolId(u16::MAX)));
}

#[test]
fn test_edge_push_non_delimiter_ignored() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(99); // not a delimiter
    // Stack should be empty; pop_scope_test returns None
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_edge_pop_scope_on_empty_stack() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.pop_scope(11));
}

#[test]
fn test_edge_pop_scope_test_on_empty_stack() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_edge_update_recent_tokens_via_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(SymbolId(100));
    // No public accessor; verify no panic on repeated calls
    for i in 0..20 {
        state.update_recent_tokens(SymbolId(i));
    }
}

#[test]
fn test_edge_is_scope_delimiter_static() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(5, &delimiters));
}

#[test]
fn test_edge_is_matching_delimiter_static() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_matching_delimiter(1, 2, &delimiters));
    assert!(ErrorRecoveryState::is_matching_delimiter(3, 4, &delimiters));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        1,
        4,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        3,
        2,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        5,
        6,
        &delimiters
    ));
}

// =========================================================================
// 9. Determine recovery strategy (5 tests)
// =========================================================================

#[test]
fn test_determine_strategy_insertion_when_insertable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn test_determine_strategy_panic_mode_on_error_overflow() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // Push past the limit using increment_error_count
    for _ in 0..3 {
        state.increment_error_count();
    }
    let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn test_determine_strategy_deletion_for_wrong_token() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    // actual=50 not in expected=[1,2] and not a sync token → deletion
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(50), (0, 0), 0);
    // With default config (no insertable, no sync), deletion applies if clearly wrong
    assert!(matches!(
        strategy,
        RecoveryStrategy::TokenDeletion
            | RecoveryStrategy::TokenSubstitution
            | RecoveryStrategy::PhraseLevel
    ));
}

#[test]
fn test_determine_strategy_substitution_single_expected() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(50)); // make actual a sync token so deletion is skipped
    let mut state = ErrorRecoveryState::new(config);
    // Single expected token + actual is sync → substitution
    let strategy = state.determine_recovery_strategy(&[1], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn test_determine_strategy_scope_recovery_on_mismatch() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        enable_scope_recovery: true,
        enable_phrase_recovery: false,
        // Make token 11 a sync token so deletion is skipped
        sync_tokens: smallvec::smallvec![SymbolId(11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // Close delimiter 11 without matching open → scope mismatch
    // expected is empty, actual=11 is sync so not "clearly wrong"
    // Single expected? No — empty, so substitution also skipped
    let strategy = state.determine_recovery_strategy(&[], Some(11), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

// =========================================================================
// 10. Builder set_max_recovery_attempts alias (2 tests)
// =========================================================================

#[test]
fn test_builder_set_max_recovery_attempts_alias() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(7)
        .build();
    assert_eq!(config.max_consecutive_errors, 7);
}

#[test]
fn test_builder_set_max_recovery_attempts_overrides_max_consecutive() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .set_max_recovery_attempts(9)
        .build();
    assert_eq!(config.max_consecutive_errors, 9);
}

// =========================================================================
// 11. Reset consecutive errors (2 tests)
// =========================================================================

#[test]
fn test_state_reset_consecutive_errors_method() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn test_state_reset_consecutive_errors_then_should_not_give_up() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}
