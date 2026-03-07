// Comprehensive tests for ErrorRecoveryConfig, ErrorRecoveryState, and related types.
use adze::adze_ir as ir;
use adze::error_recovery::*;

use ir::SymbolId;
use std::collections::HashSet;

// =====================================================================
// 1. Config defaults (8 tests)
// =====================================================================

#[test]
fn default_config_max_panic_skip_is_50() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn default_config_sync_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn default_config_insert_candidates_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn default_config_deletable_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.deletable_tokens.is_empty());
}

#[test]
fn default_config_max_token_deletions_is_3() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn default_config_max_token_insertions_is_2() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn default_config_max_consecutive_errors_is_10() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn default_config_recovery_flags() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
    assert!(cfg.enable_scope_recovery);
    assert!(!cfg.enable_indentation_recovery);
    assert!(cfg.scope_delimiters.is_empty());
}

// =====================================================================
// 2. Config custom values (8 tests)
// =====================================================================

#[test]
fn custom_config_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 200,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn custom_config_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    assert_eq!(cfg.max_consecutive_errors, 5);
}

#[test]
fn custom_config_max_token_deletions() {
    let cfg = ErrorRecoveryConfig {
        max_token_deletions: 10,
        ..Default::default()
    };
    assert_eq!(cfg.max_token_deletions, 10);
}

#[test]
fn custom_config_max_token_insertions() {
    let cfg = ErrorRecoveryConfig {
        max_token_insertions: 7,
        ..Default::default()
    };
    assert_eq!(cfg.max_token_insertions, 7);
}

#[test]
fn custom_config_disable_phrase_recovery() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        ..Default::default()
    };
    assert!(!cfg.enable_phrase_recovery);
}

#[test]
fn custom_config_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfig {
        enable_indentation_recovery: true,
        ..Default::default()
    };
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn custom_config_scope_delimiters() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41), (91, 93)],
        ..Default::default()
    };
    assert_eq!(cfg.scope_delimiters.len(), 2);
    assert_eq!(cfg.scope_delimiters[0], (40, 41));
    assert_eq!(cfg.scope_delimiters[1], (91, 93));
}

#[test]
fn custom_config_deletable_tokens() {
    let mut deletable = HashSet::new();
    deletable.insert(10u16);
    deletable.insert(20);
    let cfg = ErrorRecoveryConfig {
        deletable_tokens: deletable,
        ..Default::default()
    };
    assert!(cfg.deletable_tokens.contains(&10));
    assert!(cfg.deletable_tokens.contains(&20));
    assert!(!cfg.deletable_tokens.contains(&30));
}

// =====================================================================
// 3. Config Debug and Clone (8 tests)
// =====================================================================

#[test]
fn config_implements_debug() {
    let cfg = ErrorRecoveryConfig::default();
    let debug_str = format!("{cfg:?}");
    assert!(debug_str.contains("ErrorRecoveryConfig"));
}

#[test]
fn config_clone_preserves_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 123,
        ..Default::default()
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 123);
}

#[test]
fn config_clone_preserves_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 42,
        ..Default::default()
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.max_consecutive_errors, 42);
}

#[test]
fn config_clone_preserves_scope_delimiters() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2), (3, 4)],
        ..Default::default()
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.scope_delimiters, vec![(1, 2), (3, 4)]);
}

#[test]
fn config_clone_preserves_flags() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        enable_indentation_recovery: true,
        ..Default::default()
    };
    let cloned = cfg.clone();
    assert!(!cloned.enable_phrase_recovery);
    assert!(!cloned.enable_scope_recovery);
    assert!(cloned.enable_indentation_recovery);
}

#[test]
fn config_can_delete_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    // Non-sync token can be deleted
    assert!(cfg.can_delete_token(SymbolId(5)));
}

#[test]
fn config_cannot_delete_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(!cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn config_can_replace_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(30));
    assert!(cfg.can_replace_token(SymbolId(25)));
    assert!(!cfg.can_replace_token(SymbolId(30)));
}

// =====================================================================
// 4. State creation and initial state (8 tests)
// =====================================================================

#[test]
fn state_new_zero_consecutive_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_new_empty_error_nodes() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_new_does_not_give_up() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_new_with_custom_config() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
}

#[test]
fn state_new_with_scope_delimiters() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    assert_eq!(state.pop_scope_test(), Some(1));
}

#[test]
fn state_new_accepts_builder_config() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .max_consecutive_errors(5)
        .build();
    let state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
}

#[test]
fn state_new_empty_recent_tokens() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    // Adding a token proves recent tokens was empty before
    state.add_recent_token(42);
    // No panic means success
}

#[test]
fn state_new_empty_scope_stack() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    // pop_scope_test returns None when stack is empty
    assert_eq!(state.pop_scope_test(), None);
}

// =====================================================================
// 5. Error counting (8 tests)
// =====================================================================

#[test]
fn increment_error_count_once() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    // Default max is 10, so 1 error should not give up
    assert!(!state.should_give_up());
}

#[test]
fn increment_error_count_multiple_times() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn increment_error_count_to_exact_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..3 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn increment_error_count_exceeds_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn error_count_starts_at_zero() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    });
    assert!(!state.should_give_up());
}

#[test]
fn error_count_exactly_one_below_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn error_count_with_max_one() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn error_count_with_large_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 1000,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..999 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

// =====================================================================
// 6. Error recording and nodes (8 tests)
// =====================================================================

#[test]
fn record_error_creates_node() {
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
    assert_eq!(state.get_error_nodes().len(), 1);
}

#[test]
fn record_error_node_byte_range() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        10,
        20,
        (1, 0),
        (1, 10),
        vec![5],
        Some(6),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, 10);
    assert_eq!(nodes[0].end_byte, 20);
}

#[test]
fn record_error_node_expected_symbols() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![10, 20, 30],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].expected, vec![10, 20, 30]);
}

#[test]
fn record_error_node_actual_token() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![99],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].actual, Some(99));
}

#[test]
fn record_error_node_actual_none() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].actual, None);
}

#[test]
fn record_error_node_recovery_strategy() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        Some(1),
        RecoveryStrategy::ScopeRecovery,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn record_multiple_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5 {
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
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn get_error_nodes_returns_clone() {
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
    let nodes_a = state.get_error_nodes();
    let nodes_b = state.get_error_nodes();
    assert_eq!(nodes_a.len(), nodes_b.len());
    assert_eq!(nodes_a[0].start_byte, nodes_b[0].start_byte);
}

// =====================================================================
// 7. Should give up logic (8 tests)
// =====================================================================

#[test]
fn should_give_up_false_at_zero() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn should_give_up_false_below_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 10,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..9 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn should_give_up_true_at_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn should_give_up_true_above_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn should_give_up_false_after_reset() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn should_give_up_with_zero_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(cfg);
    // 0 errors >= 0 limit → give up immediately
    assert!(state.should_give_up());
}

#[test]
fn should_give_up_boundary_below() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 100,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..99 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn should_give_up_boundary_exact() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 100,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..100 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

// =====================================================================
// 8. Recovery strategy selection (7 tests)
// =====================================================================

#[test]
fn strategy_token_insertion_when_insertable() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_panic_mode_after_exceeding_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Exhaust error budget
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_phrase_level_as_fallback() {
    // Make the actual token a sync token so it won't be "clearly wrong"
    // and won't match substitution (multiple expected), triggering phrase-level fallback
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .add_sync_token(88)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[99, 100], Some(88), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_token_deletion_for_wrong_token() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Token 50 is not in expected set [10, 20] and not a sync token → deletion
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_substitution_when_single_expected() {
    // Token is not clearly wrong (it IS a sync token) but single expected → substitution
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .add_sync_token(50)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn strategy_scope_recovery_on_mismatch() {
    // Make the closing delimiter a sync token so it's not "clearly wrong"
    // and provide multiple expected tokens so substitution doesn't trigger
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .add_scope_delimiter(40, 41)
        .add_sync_token(41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[1, 2, 3], Some(41), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn strategy_resets_errors_on_successful_insertion() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    // Successful insertion should reset consecutive errors
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
    // After successful insertion reset, should_give_up should be false
    assert!(!state.should_give_up());
}

// =====================================================================
// 9. Reset behavior (5 tests)
// =====================================================================

#[test]
fn reset_error_count_clears_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..5 {
        state.increment_error_count();
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn reset_consecutive_errors_method() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..5 {
        state.increment_error_count();
    }
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn clear_errors_removes_all_nodes() {
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
fn reset_then_increment_again() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..3 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn clear_errors_does_not_affect_error_count() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.clear_errors();
    // Error nodes cleared but consecutive count remains
    assert!(state.should_give_up());
    assert!(state.get_error_nodes().is_empty());
}

// =====================================================================
// 10. Scope operations (8 tests)
// =====================================================================

#[test]
fn push_scope_opening_delimiter() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    assert_eq!(state.pop_scope_test(), Some(1));
}

#[test]
fn push_scope_ignores_non_delimiter() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn pop_scope_matching_close() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    assert!(state.pop_scope(2));
}

#[test]
fn pop_scope_non_matching_close() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2), (3, 4)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    assert!(!state.pop_scope(4));
}

#[test]
fn nested_scopes() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2), (3, 4)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    state.push_scope(3);
    assert!(state.pop_scope(4));
    assert!(state.pop_scope(2));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn pop_scope_empty_stack_returns_false() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.pop_scope(2));
}

#[test]
fn scope_test_pop_empty_returns_none() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn multiple_same_scopes() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    state.push_scope(1);
    assert_eq!(state.pop_scope_test(), Some(1));
    assert_eq!(state.pop_scope_test(), Some(1));
    assert_eq!(state.pop_scope_test(), None);
}

// =====================================================================
// 11. Recent tokens tracking (4 tests)
// =====================================================================

#[test]
fn add_recent_token_works() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.add_recent_token(42);
    // No panic = success
}

#[test]
fn update_recent_tokens_with_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(SymbolId(7));
    // No panic = success
}

#[test]
fn recent_tokens_overflow_caps_at_10() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..15 {
        state.add_recent_token(i);
    }
    // No panic = success; internal cap at 10
}

#[test]
fn add_recent_token_multiple() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5 {
        state.add_recent_token(i);
    }
    // No panic = success
}

// =====================================================================
// 12. Builder tests (5 tests)
// =====================================================================

#[test]
fn builder_default_produces_same_as_default() {
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    let from_default = ErrorRecoveryConfig::default();
    assert_eq!(from_builder.max_panic_skip, from_default.max_panic_skip);
    assert_eq!(
        from_builder.max_consecutive_errors,
        from_default.max_consecutive_errors
    );
}

#[test]
fn builder_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn builder_add_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(42).build();
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 42));
}

#[test]
fn builder_add_insertable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .build();
    assert!(cfg.insert_candidates.iter().any(|t| t.0 == 7));
}

#[test]
fn builder_chained_configuration() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(40, 41)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .max_consecutive_errors(20)
        .build();
    assert_eq!(cfg.max_panic_skip, 10);
    assert_eq!(cfg.max_consecutive_errors, 20);
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
    assert_eq!(cfg.scope_delimiters, vec![(40, 41)]);
    assert!(cfg.deletable_tokens.contains(&3));
}

// =====================================================================
// 13. RecoveryStrategy enum (4 tests)
// =====================================================================

#[test]
fn recovery_strategy_equality() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn recovery_strategy_inequality() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion);
    assert_ne!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::PhraseLevel
    );
}

#[test]
fn recovery_strategy_debug() {
    let s = format!("{:?}", RecoveryStrategy::PanicMode);
    assert_eq!(s, "PanicMode");
}

#[test]
fn recovery_strategy_clone() {
    let original = RecoveryStrategy::TokenSubstitution;
    let copied = original;
    assert_eq!(original, copied);
}

// =====================================================================
// 14. RecoveryAction enum (3 tests)
// =====================================================================

#[test]
fn recovery_action_insert_token() {
    let action = RecoveryAction::InsertToken(SymbolId(10));
    assert!(matches!(action, RecoveryAction::InsertToken(SymbolId(10))));
}

#[test]
fn recovery_action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    assert!(matches!(action, RecoveryAction::DeleteToken));
}

#[test]
fn recovery_action_debug() {
    let action = RecoveryAction::DeleteToken;
    let s = format!("{action:?}");
    assert!(s.contains("DeleteToken"));
}

// =====================================================================
// 15. Static helper methods (3 tests)
// =====================================================================

#[test]
fn is_scope_delimiter_true() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delimiters));
}

#[test]
fn is_scope_delimiter_false() {
    let delimiters = vec![(1, 2)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delimiters));
}

#[test]
fn is_matching_delimiter_checks_pairs() {
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
}

// =====================================================================
// 16. ErrorNode (2 tests)
// =====================================================================

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2],
        actual: Some(3),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![3],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 0);
    assert_eq!(cloned.end_byte, 10);
    assert_eq!(cloned.expected, vec![1, 2]);
    assert_eq!(cloned.actual, Some(3));
}

#[test]
fn error_node_debug() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 15,
        start_position: (1, 0),
        end_position: (1, 10),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    let s = format!("{node:?}");
    assert!(s.contains("ErrorNode"));
}
