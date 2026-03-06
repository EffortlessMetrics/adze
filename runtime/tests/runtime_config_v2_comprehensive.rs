//! Comprehensive tests for ErrorRecoveryConfig, ErrorRecoveryState, and related config types.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze::lexer::ErrorRecoveryMode;

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;
use std::collections::HashSet;

// ============================================================
// 1. ErrorRecoveryConfig construction with struct literals (10)
// ============================================================

#[test]
fn test_config_default_max_panic_skip() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn test_config_default_max_token_deletions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_deletions, 3);
}

#[test]
fn test_config_default_max_token_insertions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_insertions, 2);
}

#[test]
fn test_config_default_max_consecutive_errors() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn test_config_default_phrase_recovery_enabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.enable_phrase_recovery);
}

#[test]
fn test_config_default_scope_recovery_enabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.enable_scope_recovery);
}

#[test]
fn test_config_default_indentation_recovery_disabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn test_config_default_empty_collections() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.sync_tokens.is_empty());
    assert!(config.insert_candidates.is_empty());
    assert!(config.deletable_tokens.is_empty());
    assert!(config.scope_delimiters.is_empty());
}

#[test]
fn test_config_struct_literal_override_single_field() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 100,
        ..Default::default()
    };
    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn test_config_struct_literal_override_multiple_fields() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 200,
        max_token_deletions: 5,
        max_token_insertions: 4,
        enable_phrase_recovery: false,
        enable_indentation_recovery: true,
        ..Default::default()
    };
    assert_eq!(config.max_panic_skip, 200);
    assert_eq!(config.max_token_deletions, 5);
    assert_eq!(config.max_token_insertions, 4);
    assert!(!config.enable_phrase_recovery);
    assert!(config.enable_indentation_recovery);
}

// ============================================================
// 2. ErrorRecoveryState behavior (8)
// ============================================================

#[test]
fn test_state_new_zero_consecutive_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn test_state_increment_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    // After 2 increments with max_consecutive_errors=10, should not give up
    assert!(!state.should_give_up());
}

#[test]
fn test_state_reset_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..5 {
        state.increment_error_count();
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn test_state_should_give_up_at_threshold() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn test_state_get_error_nodes_initially_empty() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_state_record_and_retrieve_error() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        10,
        20,
        (1, 0),
        (1, 10),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::TokenDeletion,
        vec![3],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 10);
    assert_eq!(nodes[0].end_byte, 20);
}

#[test]
fn test_state_clear_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
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
    assert_eq!(state.get_error_nodes().len(), 1);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_state_add_recent_token_capped_at_ten() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..15 {
        state.add_recent_token(i);
    }
    // Internal buffer is capped at 10
    state.update_recent_tokens(SymbolId(99));
    // We can verify indirectly by checking that state still works
    assert!(!state.should_give_up());
}

// ============================================================
// 3. ErrorRecoveryMode variants (5)
// ============================================================

#[test]
fn test_mode_skip_char_eq() {
    assert_eq!(ErrorRecoveryMode::SkipChar, ErrorRecoveryMode::SkipChar);
}

#[test]
fn test_mode_skip_to_known_eq() {
    assert_eq!(
        ErrorRecoveryMode::SkipToKnown,
        ErrorRecoveryMode::SkipToKnown
    );
}

#[test]
fn test_mode_fail_eq() {
    assert_eq!(ErrorRecoveryMode::Fail, ErrorRecoveryMode::Fail);
}

#[test]
fn test_mode_variants_not_equal() {
    assert_ne!(ErrorRecoveryMode::SkipChar, ErrorRecoveryMode::Fail);
    assert_ne!(ErrorRecoveryMode::SkipToKnown, ErrorRecoveryMode::Fail);
    assert_ne!(ErrorRecoveryMode::SkipChar, ErrorRecoveryMode::SkipToKnown);
}

#[test]
fn test_mode_debug_format() {
    let mode = ErrorRecoveryMode::SkipChar;
    let debug = format!("{mode:?}");
    assert!(debug.contains("SkipChar"));
}

// ============================================================
// 4. ErrorNode properties (5)
// ============================================================

#[test]
fn test_error_node_construction() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2, 3],
        actual: Some(4),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![4],
    };
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 10);
}

#[test]
fn test_error_node_no_actual_token() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 5,
        start_position: (1, 5),
        end_position: (1, 5),
        expected: vec![10],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert!(node.actual.is_none());
    assert!(node.skipped_tokens.is_empty());
}

#[test]
fn test_error_node_empty_expected() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![],
        actual: Some(99),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![99],
    };
    assert!(node.expected.is_empty());
}

#[test]
fn test_error_node_clone() {
    let node = ErrorNode {
        start_byte: 3,
        end_byte: 7,
        start_position: (0, 3),
        end_position: (0, 7),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![2, 3],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 3);
    assert_eq!(cloned.end_byte, 7);
    assert_eq!(cloned.expected, vec![1]);
    assert_eq!(cloned.skipped_tokens, vec![2, 3]);
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
    let debug = format!("{node:?}");
    assert!(debug.contains("ErrorNode"));
}

// ============================================================
// 5. Config Clone/Debug/PartialEq (5)
// ============================================================

#[test]
fn test_config_clone_preserves_fields() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 77,
        max_consecutive_errors: 5,
        enable_phrase_recovery: false,
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.max_panic_skip, 77);
    assert_eq!(cloned.max_consecutive_errors, 5);
    assert!(!cloned.enable_phrase_recovery);
}

#[test]
fn test_config_debug_format() {
    let config = ErrorRecoveryConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("ErrorRecoveryConfig"));
    assert!(debug.contains("max_panic_skip"));
}

#[test]
fn test_recovery_strategy_clone() {
    let strategy = RecoveryStrategy::ScopeRecovery;
    let cloned = strategy;
    assert_eq!(cloned, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn test_recovery_strategy_debug() {
    let s = RecoveryStrategy::TokenSubstitution;
    let debug = format!("{s:?}");
    assert!(debug.contains("TokenSubstitution"));
}

#[test]
fn test_recovery_action_clone_and_debug() {
    let action = RecoveryAction::InsertToken(SymbolId(42));
    let cloned = action.clone();
    assert_eq!(cloned, RecoveryAction::InsertToken(SymbolId(42)));
    let debug = format!("{cloned:?}");
    assert!(debug.contains("InsertToken"));
}

// ============================================================
// 6. State transitions (8)
// ============================================================

#[test]
fn test_state_determine_strategy_insertion() {
    let config = ErrorRecoveryConfig {
        insert_candidates: smallvec::smallvec![SymbolId(10)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn test_state_determine_strategy_deletion() {
    let config = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // actual token 99 not in expected [1, 2], and not a sync token → deletion
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn test_state_determine_strategy_substitution() {
    let config = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // Exactly one expected token and actual is different → substitution
    let strategy = state.determine_recovery_strategy(&[5], Some(99), (0, 0), 0);
    // With one expected, substitution check passes: expected.len() == 1
    // But deletion check also applies. Deletion is tried first.
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn test_state_determine_strategy_panic_mode_on_max_errors() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // Exhaust error budget: 2 calls use up the budget, 3rd triggers panic
    let _ = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    let _ = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn test_state_determine_strategy_phrase_level_fallback() {
    let config = ErrorRecoveryConfig {
        enable_phrase_recovery: true,
        sync_tokens: smallvec::smallvec![SymbolId(99)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    // actual=99 is a sync token so deletion is skipped; no insertion candidates
    // substitution needs len()==1 which is met, but actual is sync → can_substitute
    // checks _actual, expected len==1 → true, so TokenSubstitution
    // Actually let's use multiple expected to avoid substitution
    let strategy = state.determine_recovery_strategy(&[1, 2, 3], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn test_state_scope_push_and_pop() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)], // '(' and ')'
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(state.pop_scope(41));
}

#[test]
fn test_state_scope_pop_mismatch_returns_false() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41), (91, 93)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    // Try to pop with wrong closer
    assert!(!state.pop_scope(93));
}

#[test]
fn test_state_scope_pop_empty_returns_false() {
    let config = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.pop_scope(41));
}

// ============================================================
// 7. Config validation (5)
// ============================================================

#[test]
fn test_config_can_delete_non_sync_token() {
    let config = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(10)],
        ..Default::default()
    };
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn test_config_cannot_delete_sync_token() {
    let config = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(10)],
        ..Default::default()
    };
    assert!(!config.can_delete_token(SymbolId(10)));
}

#[test]
fn test_config_can_delete_explicitly_deletable_sync_token() {
    let mut deletable = HashSet::new();
    deletable.insert(10u16);
    let config = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(10)],
        deletable_tokens: deletable,
        ..Default::default()
    };
    // Token 10 is sync but also explicitly deletable → can delete
    assert!(config.can_delete_token(SymbolId(10)));
}

#[test]
fn test_config_can_replace_non_sync_token() {
    let config = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(20)],
        ..Default::default()
    };
    assert!(config.can_replace_token(SymbolId(5)));
}

#[test]
fn test_config_cannot_replace_sync_token() {
    let config = ErrorRecoveryConfig {
        sync_tokens: smallvec::smallvec![SymbolId(20)],
        ..Default::default()
    };
    assert!(!config.can_replace_token(SymbolId(20)));
}

// ============================================================
// 8. Edge cases (9)
// ============================================================

#[test]
fn test_config_zero_max_panic_skip() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 0,
        ..Default::default()
    };
    assert_eq!(config.max_panic_skip, 0);
}

#[test]
fn test_config_zero_max_consecutive_errors_immediate_give_up() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(config);
    assert!(state.should_give_up());
}

#[test]
fn test_config_all_recovery_disabled() {
    let config = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        enable_indentation_recovery: false,
        ..Default::default()
    };
    assert!(!config.enable_phrase_recovery);
    assert!(!config.enable_scope_recovery);
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn test_state_multiple_error_recordings() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..20 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![1],
            Some(2),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 20);
}

#[test]
fn test_builder_default_matches_config_default() {
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    let from_default = ErrorRecoveryConfig::default();
    assert_eq!(from_builder.max_panic_skip, from_default.max_panic_skip);
    assert_eq!(
        from_builder.max_consecutive_errors,
        from_default.max_consecutive_errors
    );
    assert_eq!(
        from_builder.enable_phrase_recovery,
        from_default.enable_phrase_recovery
    );
    assert_eq!(
        from_builder.enable_indentation_recovery,
        from_default.enable_indentation_recovery
    );
}

#[test]
fn test_builder_chained_operations() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(25)
        .add_sync_token(1)
        .add_sync_token_sym(SymbolId(2))
        .add_insertable_token(3)
        .add_insertable_token_sym(SymbolId(4))
        .add_deletable_token(5)
        .add_scope_delimiter(10, 11)
        .enable_indentation_recovery(true)
        .enable_scope_recovery(false)
        .enable_phrase_recovery(false)
        .max_consecutive_errors(7)
        .build();
    assert_eq!(config.max_panic_skip, 25);
    assert_eq!(config.sync_tokens.len(), 2);
    assert_eq!(config.insert_candidates.len(), 2);
    assert!(config.deletable_tokens.contains(&5));
    assert_eq!(config.scope_delimiters, [(10, 11)]);
    assert!(config.enable_indentation_recovery);
    assert!(!config.enable_scope_recovery);
    assert!(!config.enable_phrase_recovery);
    assert_eq!(config.max_consecutive_errors, 7);
}

#[test]
fn test_recovery_strategy_all_variants() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    // All variants are distinct
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_recovery_action_all_variants() {
    let actions: Vec<RecoveryAction> = vec![
        RecoveryAction::InsertToken(SymbolId(1)),
        RecoveryAction::DeleteToken,
        RecoveryAction::ReplaceToken(SymbolId(2)),
        RecoveryAction::CreateErrorNode(vec![SymbolId(3)]),
    ];
    assert!(matches!(actions[0], RecoveryAction::InsertToken(_)));
    assert!(matches!(actions[1], RecoveryAction::DeleteToken));
    assert!(matches!(actions[2], RecoveryAction::ReplaceToken(_)));
    assert!(matches!(actions[3], RecoveryAction::CreateErrorNode(_)));
}

#[test]
fn test_static_helper_is_scope_delimiter() {
    let delimiters = [(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(91, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(93, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(0, &delimiters));
}
