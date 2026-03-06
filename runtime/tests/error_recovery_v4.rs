//! Tests for the error recovery module (v4).
//!
//! Covers: config defaults, config customization, recovery strategy transitions,
//! state tracking, error node construction, config validation, and edge cases.

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze_ir::SymbolId;
use smallvec::SmallVec;

// ---------------------------------------------------------------------------
// 1. ErrorRecoveryConfig defaults
// ---------------------------------------------------------------------------

#[test]
fn test_default_config_max_panic_skip() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn test_default_config_max_token_deletions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn test_default_config_max_token_insertions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn test_default_config_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn test_default_config_phrase_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn test_default_config_scope_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn test_default_config_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
}

#[test]
fn test_default_config_empty_sync_tokens() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn test_default_config_empty_insert_candidates() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn test_default_config_empty_deletable_tokens() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.deletable_tokens.is_empty());
}

#[test]
fn test_default_config_empty_scope_delimiters() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.scope_delimiters.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Config customization
// ---------------------------------------------------------------------------

#[test]
fn test_config_set_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 100,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, 100);
}

#[test]
fn test_config_set_max_token_deletions() {
    let cfg = ErrorRecoveryConfig {
        max_token_deletions: 10,
        ..Default::default()
    };
    assert_eq!(cfg.max_token_deletions, 10);
}

#[test]
fn test_config_set_max_token_insertions() {
    let cfg = ErrorRecoveryConfig {
        max_token_insertions: 5,
        ..Default::default()
    };
    assert_eq!(cfg.max_token_insertions, 5);
}

#[test]
fn test_config_set_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 25,
        ..Default::default()
    };
    assert_eq!(cfg.max_consecutive_errors, 25);
}

#[test]
fn test_config_disable_phrase_recovery() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        ..Default::default()
    };
    assert!(!cfg.enable_phrase_recovery);
}

#[test]
fn test_config_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfig {
        enable_indentation_recovery: true,
        ..Default::default()
    };
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn test_config_add_sync_token() {
    let cfg = ErrorRecoveryConfig {
        sync_tokens: SmallVec::from_vec(vec![SymbolId(42)]),
        ..Default::default()
    };
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.sync_tokens[0], SymbolId(42));
}

#[test]
fn test_config_add_scope_delimiter() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    assert_eq!(cfg.scope_delimiters.len(), 1);
    assert_eq!(cfg.scope_delimiters[0], (1, 2));
}

#[test]
fn test_config_can_delete_non_sync_token() {
    let cfg = ErrorRecoveryConfig::default();
    // No sync tokens configured, so any token can be deleted
    assert!(cfg.can_delete_token(SymbolId(99)));
}

#[test]
fn test_config_can_delete_explicitly_deletable() {
    let cfg = ErrorRecoveryConfig {
        sync_tokens: SmallVec::from_vec(vec![SymbolId(5)]),
        deletable_tokens: [5].into_iter().collect(),
        ..Default::default()
    };
    // Explicitly deletable overrides sync token restriction
    assert!(cfg.can_delete_token(SymbolId(5)));
}

#[test]
fn test_config_cannot_delete_sync_token_not_in_deletable() {
    let cfg = ErrorRecoveryConfig {
        sync_tokens: SmallVec::from_vec(vec![SymbolId(7)]),
        ..Default::default()
    };
    // Token 7 is a sync token but not in deletable set
    assert!(!cfg.can_delete_token(SymbolId(7)));
}

#[test]
fn test_config_can_replace_non_sync_token() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.can_replace_token(SymbolId(42)));
}

#[test]
fn test_config_cannot_replace_sync_token() {
    let cfg = ErrorRecoveryConfig {
        sync_tokens: SmallVec::from_vec(vec![SymbolId(10)]),
        ..Default::default()
    };
    assert!(!cfg.can_replace_token(SymbolId(10)));
}

// ---------------------------------------------------------------------------
// 3. ErrorRecoveryMode transitions (via strategy determination)
// ---------------------------------------------------------------------------

#[test]
fn test_strategy_panic_mode_after_max_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    // First two errors won't trigger PanicMode by exceeding limit
    let _ = state.determine_recovery_strategy(&[100], Some(200), (0, 0), 0);
    let _ = state.determine_recovery_strategy(&[100], Some(200), (0, 0), 0);
    // Third exceeds max_consecutive_errors (2)
    let strategy = state.determine_recovery_strategy(&[100], Some(200), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn test_strategy_token_insertion_when_candidate_available() {
    let cfg = ErrorRecoveryConfig {
        insert_candidates: SmallVec::from_vec(vec![SymbolId(5)]),
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    let strategy = state.determine_recovery_strategy(&[5], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn test_strategy_token_deletion_for_unexpected() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    // Token 99 is not in expected [1,2,3] and not a sync token
    let strategy = state.determine_recovery_strategy(&[1, 2, 3], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn test_strategy_substitution_with_single_expected() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        sync_tokens: SmallVec::from_vec(vec![SymbolId(50)]),
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    // Single expected token + actual is sync token → substitution
    let strategy = state.determine_recovery_strategy(&[10], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn test_strategy_phrase_level_as_fallback() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: true,
        enable_scope_recovery: false,
        sync_tokens: SmallVec::from_vec(vec![SymbolId(50)]),
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    let strategy = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn test_strategy_scope_recovery_on_mismatch() {
    let cfg = ErrorRecoveryConfig {
        enable_scope_recovery: true,
        enable_phrase_recovery: false,
        scope_delimiters: vec![(1, 2)],
        sync_tokens: SmallVec::from_vec(vec![SymbolId(2)]),
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    // Token 2 is a closing delimiter without matching open → scope mismatch
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn test_strategy_defaults_to_panic_when_all_disabled() {
    let cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        sync_tokens: SmallVec::from_vec(vec![SymbolId(50)]),
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    let strategy = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// ---------------------------------------------------------------------------
// 4. ErrorRecoveryState tracking
// ---------------------------------------------------------------------------

#[test]
fn test_state_initial_error_count_zero() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn test_state_increment_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    // max_consecutive_errors default is 10, so 2 errors won't trigger give up
    assert!(!state.should_give_up());
}

#[test]
fn test_state_should_give_up_at_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn test_state_reset_error_count() {
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
fn test_state_record_error_and_retrieve() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());

    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
}

#[test]
fn test_state_record_multiple_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());

    state.record_error(
        0,
        3,
        (0, 0),
        (0, 3),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    state.record_error(
        5,
        8,
        (0, 5),
        (0, 8),
        vec![2],
        Some(9),
        RecoveryStrategy::TokenDeletion,
        vec![9],
    );
    state.record_error(
        10,
        15,
        (1, 0),
        (1, 5),
        vec![],
        Some(7),
        RecoveryStrategy::PhraseLevel,
        vec![],
    );

    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 3);
}

#[test]
fn test_state_clear_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());

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
    assert_eq!(state.get_error_nodes().len(), 1);

    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_state_add_recent_token() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.add_recent_token(42);
    // No public accessor for recent_tokens, but this should not panic
}

#[test]
fn test_state_recent_tokens_capacity_limit() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..15 {
        state.add_recent_token(i);
    }
    // Internal deque caps at 10 — should not panic
}

#[test]
fn test_state_push_scope() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    state.push_scope(1); // opening delimiter
    let popped = state.pop_scope_test();
    assert_eq!(popped, Some(1));
}

#[test]
fn test_state_push_scope_ignores_non_delimiter() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    state.push_scope(99); // not a delimiter
    let popped = state.pop_scope_test();
    assert_eq!(popped, None);
}

#[test]
fn test_state_pop_scope_matching() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 20)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    state.push_scope(10);
    assert!(state.pop_scope(20));
}

#[test]
fn test_state_pop_scope_non_matching() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 20), (30, 40)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    state.push_scope(10);
    // Try to pop with wrong closing delimiter
    assert!(!state.pop_scope(40));
}

#[test]
fn test_state_update_recent_tokens_via_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(SymbolId(7));
    // Should not panic — exercises the SymbolId wrapper path
}

#[test]
fn test_state_reset_consecutive_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

// ---------------------------------------------------------------------------
// 5. ErrorNode construction
// ---------------------------------------------------------------------------

#[test]
fn test_error_node_basic_construction() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 20,
        start_position: (1, 0),
        end_position: (1, 10),
        expected: vec![1, 2, 3],
        actual: Some(99),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![4, 5],
    };
    assert_eq!(node.start_byte, 10);
    assert_eq!(node.end_byte, 20);
    assert_eq!(node.expected.len(), 3);
    assert_eq!(node.actual, Some(99));
}

#[test]
fn test_error_node_no_actual_token() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 0,
        start_position: (0, 0),
        end_position: (0, 0),
        expected: vec![1],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert!(node.actual.is_none());
}

#[test]
fn test_error_node_clone() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 15,
        start_position: (0, 5),
        end_position: (0, 15),
        expected: vec![10],
        actual: Some(20),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![20],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 5);
    assert_eq!(cloned.end_byte, 15);
    assert_eq!(cloned.expected, vec![10]);
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
    assert!(!debug_str.is_empty());
}

#[test]
fn test_error_node_all_recovery_strategies() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for strategy in strategies {
        let node = ErrorNode {
            start_byte: 0,
            end_byte: 0,
            start_position: (0, 0),
            end_position: (0, 0),
            expected: vec![],
            actual: None,
            recovery: strategy,
            skipped_tokens: vec![],
        };
        assert_eq!(node.recovery, strategy);
    }
}

// ---------------------------------------------------------------------------
// 6. Config validation — limits and boundaries
// ---------------------------------------------------------------------------

#[test]
fn test_config_builder_default_matches_config_default() {
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    let from_default = ErrorRecoveryConfig::default();
    assert_eq!(from_builder.max_panic_skip, from_default.max_panic_skip);
    assert_eq!(
        from_builder.max_consecutive_errors,
        from_default.max_consecutive_errors
    );
    assert_eq!(
        from_builder.max_token_deletions,
        from_default.max_token_deletions
    );
    assert_eq!(
        from_builder.max_token_insertions,
        from_default.max_token_insertions
    );
    assert_eq!(
        from_builder.enable_phrase_recovery,
        from_default.enable_phrase_recovery
    );
    assert_eq!(
        from_builder.enable_scope_recovery,
        from_default.enable_scope_recovery
    );
    assert_eq!(
        from_builder.enable_indentation_recovery,
        from_default.enable_indentation_recovery
    );
}

#[test]
fn test_builder_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn test_builder_add_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(5)
        .add_sync_token(10)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 2);
}

#[test]
fn test_builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(42))
        .build();
    assert_eq!(cfg.sync_tokens[0], SymbolId(42));
}

#[test]
fn test_builder_add_insertable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .build();
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert_eq!(cfg.insert_candidates[0], SymbolId(7));
}

#[test]
fn test_builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(3)
        .build();
    assert!(cfg.deletable_tokens.contains(&3));
}

#[test]
fn test_builder_add_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(40, 41)]);
}

#[test]
fn test_builder_enable_flags() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn test_builder_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(50)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 50);
}

#[test]
fn test_builder_set_max_recovery_attempts() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(77)
        .build();
    // set_max_recovery_attempts maps to max_consecutive_errors
    assert_eq!(cfg.max_consecutive_errors, 77);
}

#[test]
fn test_builder_default_trait() {
    let builder = ErrorRecoveryConfigBuilder::default();
    let cfg = builder.build();
    assert_eq!(cfg.max_panic_skip, 50);
}

// ---------------------------------------------------------------------------
// 7. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_zero_max_consecutive_errors_gives_up_immediately() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(cfg);
    // With 0 limit, should give up before any errors
    assert!(state.should_give_up());
}

#[test]
fn test_zero_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 0,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, 0);
}

#[test]
fn test_large_max_values() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: usize::MAX,
        max_token_deletions: usize::MAX,
        max_token_insertions: usize::MAX,
        max_consecutive_errors: usize::MAX,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, usize::MAX);
    assert_eq!(cfg.max_token_deletions, usize::MAX);
    assert_eq!(cfg.max_token_insertions, usize::MAX);
    assert_eq!(cfg.max_consecutive_errors, usize::MAX);
}

#[test]
fn test_error_node_zero_byte_span() {
    let node = ErrorNode {
        start_byte: 42,
        end_byte: 42,
        start_position: (3, 10),
        end_position: (3, 10),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_byte, node.end_byte);
}

#[test]
fn test_error_node_empty_expected_and_skipped() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 100,
        start_position: (0, 0),
        end_position: (5, 0),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert!(node.expected.is_empty());
    assert!(node.skipped_tokens.is_empty());
}

#[test]
fn test_recovery_strategy_copy_semantics() {
    let s1 = RecoveryStrategy::PanicMode;
    let s2 = s1; // Copy
    assert_eq!(s1, s2);
}

#[test]
fn test_recovery_strategy_equality() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn test_recovery_strategy_debug() {
    let debug_str = format!("{:?}", RecoveryStrategy::ScopeRecovery);
    assert!(debug_str.contains("ScopeRecovery"));
}

#[test]
fn test_recovery_action_insert_token() {
    let action = RecoveryAction::InsertToken(SymbolId(5));
    assert_eq!(action, RecoveryAction::InsertToken(SymbolId(5)));
}

#[test]
fn test_recovery_action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    assert_eq!(action, RecoveryAction::DeleteToken);
}

#[test]
fn test_recovery_action_clone() {
    let action = RecoveryAction::InsertToken(SymbolId(10));
    let cloned = action.clone();
    assert_eq!(action, cloned);
}

#[test]
fn test_state_empty_error_nodes_initially() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_state_pop_scope_empty_stack() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Pop on empty stack returns false
    assert!(!state.pop_scope(2));
}

#[test]
fn test_state_pop_scope_test_empty() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_is_scope_delimiter_static() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delimiters));
}

#[test]
fn test_is_matching_delimiter_static() {
    let delimiters = vec![(10, 20), (30, 40)];
    assert!(ErrorRecoveryState::is_matching_delimiter(
        10,
        20,
        &delimiters
    ));
    assert!(ErrorRecoveryState::is_matching_delimiter(
        30,
        40,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        10,
        40,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        30,
        20,
        &delimiters
    ));
}

#[test]
fn test_is_scope_delimiter_empty_list() {
    assert!(!ErrorRecoveryState::is_scope_delimiter(1, &[]));
}

#[test]
fn test_is_matching_delimiter_empty_list() {
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 2, &[]));
}

#[test]
fn test_multiple_scope_push_pop() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2), (3, 4)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    state.push_scope(1);
    state.push_scope(3);
    // Pop innermost first
    assert!(state.pop_scope(4));
    assert!(state.pop_scope(2));
    // Stack now empty
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn test_builder_chaining() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(40, 41)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .enable_indentation_recovery(false)
        .max_consecutive_errors(5)
        .build();

    assert_eq!(cfg.max_panic_skip, 10);
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert!(cfg.deletable_tokens.contains(&3));
    assert_eq!(cfg.scope_delimiters.len(), 1);
    assert_eq!(cfg.max_consecutive_errors, 5);
}

#[test]
fn test_error_node_large_expected_set() {
    let expected: Vec<u16> = (0..1000).collect();
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected,
        actual: Some(9999),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert_eq!(node.expected.len(), 1000);
}

#[test]
fn test_state_give_up_boundary() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    for _ in 0..4 {
        state.increment_error_count();
        assert!(!state.should_give_up());
    }
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn test_config_clone() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 99,
        sync_tokens: SmallVec::from_vec(vec![SymbolId(1)]),
        ..Default::default()
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 99);
    assert_eq!(cloned.sync_tokens.len(), 1);
}
