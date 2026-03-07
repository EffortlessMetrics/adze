//! Comprehensive tests for ErrorRecoveryConfig, ErrorRecoveryConfigBuilder,
//! ErrorRecoveryState, RecoveryStrategy, and ErrorNode.

use adze::adze_ir as ir;

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};

// ── 1. ErrorRecoveryConfig::default() ──────────────────────────────────────

#[test]
fn default_config_max_panic_skip_is_50() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn default_config_sync_tokens_is_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn default_config_insert_candidates_is_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn default_config_deletable_tokens_is_empty() {
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
fn default_config_phrase_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn default_config_scope_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn default_config_scope_delimiters_is_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.scope_delimiters.is_empty());
}

#[test]
fn default_config_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
}

// ── 2. ErrorRecoveryConfigBuilder::new().build() ───────────────────────────

#[test]
fn builder_default_produces_same_as_default_config() {
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
fn builder_default_sync_tokens_empty() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn builder_default_insert_candidates_empty() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn builder_default_scope_delimiters_empty() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.scope_delimiters.is_empty());
}

// ── 3. Builder max_consecutive_errors ──────────────────────────────────────

#[test]
fn builder_max_consecutive_errors_sets_value() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 5);
}

#[test]
fn builder_max_consecutive_errors_zero() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 0);
}

#[test]
fn builder_max_consecutive_errors_large_value() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1_000_000)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 1_000_000);
}

#[test]
fn builder_max_consecutive_errors_overwrite() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .max_consecutive_errors(20)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 20);
}

// ── 4. Builder set_max_recovery_attempts ───────────────────────────────────

#[test]
fn builder_set_max_recovery_attempts_sets_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(7)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 7);
}

#[test]
fn builder_set_max_recovery_attempts_overrides_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .set_max_recovery_attempts(15)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 15);
}

// ── 5. Builder max_panic_skip ──────────────────────────────────────────────

#[test]
fn builder_max_panic_skip_sets_value() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .build();
    assert_eq!(cfg.max_panic_skip, 100);
}

#[test]
fn builder_max_panic_skip_zero() {
    let cfg = ErrorRecoveryConfigBuilder::new().max_panic_skip(0).build();
    assert_eq!(cfg.max_panic_skip, 0);
}

// ── 6. Builder add_sync_token ──────────────────────────────────────────────

#[test]
fn builder_add_single_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.sync_tokens[0].0, 10);
}

#[test]
fn builder_add_multiple_sync_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token(3)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 3);
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 1));
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 2));
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 3));
}

// ── 7. Builder add_insertable_token ────────────────────────────────────────

#[test]
fn builder_add_single_insertable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(42)
        .build();
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert_eq!(cfg.insert_candidates[0].0, 42);
}

#[test]
fn builder_add_multiple_insertable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .add_insertable_token(20)
        .build();
    assert_eq!(cfg.insert_candidates.len(), 2);
}

// ── 8. Builder add_deletable_token ─────────────────────────────────────────

#[test]
fn builder_add_single_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(7)
        .build();
    assert!(cfg.deletable_tokens.contains(&7));
    assert_eq!(cfg.deletable_tokens.len(), 1);
}

#[test]
fn builder_add_multiple_deletable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(7)
        .add_deletable_token(8)
        .add_deletable_token(9)
        .build();
    assert_eq!(cfg.deletable_tokens.len(), 3);
    assert!(cfg.deletable_tokens.contains(&7));
    assert!(cfg.deletable_tokens.contains(&8));
    assert!(cfg.deletable_tokens.contains(&9));
}

#[test]
fn builder_add_duplicate_deletable_token_deduplicates() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .add_deletable_token(5)
        .build();
    assert_eq!(cfg.deletable_tokens.len(), 1);
}

// ── 9. Builder add_scope_delimiter ─────────────────────────────────────────

#[test]
fn builder_add_single_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(10, 11)]);
}

#[test]
fn builder_add_multiple_scope_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .add_scope_delimiter(30, 31)
        .build();
    assert_eq!(cfg.scope_delimiters.len(), 3);
    assert_eq!(cfg.scope_delimiters[0], (10, 11));
    assert_eq!(cfg.scope_delimiters[1], (20, 21));
    assert_eq!(cfg.scope_delimiters[2], (30, 31));
}

// ── 10. Builder enable/disable toggles ─────────────────────────────────────

#[test]
fn builder_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_disable_indentation_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(false)
        .build();
    assert!(!cfg.enable_indentation_recovery);
}

#[test]
fn builder_disable_scope_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!cfg.enable_scope_recovery);
}

#[test]
fn builder_disable_phrase_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!cfg.enable_phrase_recovery);
}

#[test]
fn builder_enable_phrase_recovery_explicitly() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_phrase_recovery(true)
        .build();
    assert!(cfg.enable_phrase_recovery);
}

// ── 11. RecoveryStrategy variants ──────────────────────────────────────────

#[test]
fn recovery_strategy_panic_mode_eq() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
}

#[test]
fn recovery_strategy_token_insertion_eq() {
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn recovery_strategy_token_deletion_eq() {
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn recovery_strategy_token_substitution_eq() {
    assert_eq!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn recovery_strategy_phrase_level_eq() {
    assert_eq!(RecoveryStrategy::PhraseLevel, RecoveryStrategy::PhraseLevel);
}

#[test]
fn recovery_strategy_scope_recovery_eq() {
    assert_eq!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn recovery_strategy_indentation_recovery_eq() {
    assert_eq!(
        RecoveryStrategy::IndentationRecovery,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn recovery_strategy_distinct_variants_not_equal() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion);
    assert_ne!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::ScopeRecovery
    );
    assert_ne!(
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::IndentationRecovery
    );
    assert_ne!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PanicMode
    );
}

#[test]
fn recovery_strategy_debug_format() {
    let dbg = format!("{:?}", RecoveryStrategy::PanicMode);
    assert_eq!(dbg, "PanicMode");
}

#[test]
fn recovery_strategy_clone() {
    let original = RecoveryStrategy::TokenDeletion;
    let cloned = original;
    assert_eq!(original, cloned);
}

// ── 12. ErrorRecoveryState::new() ──────────────────────────────────────────

#[test]
fn state_new_default_config_no_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_new_custom_config() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .add_scope_delimiter(1, 2)
        .build();
    let state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
}

#[test]
fn state_new_error_nodes_empty() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

// ── 13. increment_error_count / reset_error_count ──────────────────────────

#[test]
fn state_increment_error_count_once() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_increment_error_count_multiple_times() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_error_count_clears() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());

    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_reset_then_increment_again() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    assert!(state.should_give_up());

    state.reset_error_count();
    assert!(!state.should_give_up());

    state.increment_error_count();
    assert!(state.should_give_up());
}

// ── 14. should_give_up ─────────────────────────────────────────────────────

#[test]
fn state_should_give_up_false_initially() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_should_give_up_at_exact_threshold() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_should_give_up_above_threshold() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn state_should_give_up_zero_threshold_gives_up_immediately() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    let state = ErrorRecoveryState::new(cfg);
    assert!(state.should_give_up());
}

// ── 15. push_scope with registered delimiter ───────────────────────────────

#[test]
fn state_push_scope_registered_open_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), Some(10));
}

#[test]
fn state_push_scope_multiple_registered_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    state.push_scope(20);
    assert_eq!(state.pop_scope_test(), Some(20));
    assert_eq!(state.pop_scope_test(), Some(10));
}

// ── 16. push_scope with unregistered delimiter (no effect) ─────────────────

#[test]
fn state_push_scope_unregistered_token_no_effect() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_push_scope_close_delimiter_no_effect() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(11); // close delimiter, not open
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_push_scope_no_delimiters_configured() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), None);
}

// ── 17. pop_scope_test LIFO order ──────────────────────────────────────────

#[test]
fn state_pop_scope_test_lifo_three_items() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .add_scope_delimiter(3, 4)
        .add_scope_delimiter(5, 6)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    state.push_scope(3);
    state.push_scope(5);
    assert_eq!(state.pop_scope_test(), Some(5));
    assert_eq!(state.pop_scope_test(), Some(3));
    assert_eq!(state.pop_scope_test(), Some(1));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_test_empty_returns_none() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    // Cannot call pop_scope_test on non-mut; need mut
    let mut state = state;
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_test_interleaved_push_pop() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    state.push_scope(20);
    assert_eq!(state.pop_scope_test(), Some(20));
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), None);
}

// ── 18. pop_scope (matching-based) ─────────────────────────────────────────

#[test]
fn state_pop_scope_matching_close_succeeds() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(state.pop_scope(11));
}

#[test]
fn state_pop_scope_nonmatching_close_fails() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(!state.pop_scope(21)); // 21 matches 20, but top is 10
}

#[test]
fn state_pop_scope_empty_stack_fails() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.pop_scope(11));
}

#[test]
fn state_pop_scope_unregistered_close_fails() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(!state.pop_scope(99));
}

// ── 19. ErrorNode construction and field access ────────────────────────────

#[test]
fn error_node_basic_construction() {
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
    assert_eq!(node.start_position, (0, 0));
    assert_eq!(node.end_position, (0, 5));
    assert_eq!(node.expected, vec![1, 2]);
    assert_eq!(node.actual, Some(3));
    assert_eq!(node.recovery, RecoveryStrategy::TokenDeletion);
    assert_eq!(node.skipped_tokens, vec![3]);
}

#[test]
fn error_node_empty_expected_list() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 10,
        start_position: (1, 0),
        end_position: (1, 0),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert!(node.expected.is_empty());
    assert!(node.actual.is_none());
    assert!(node.skipped_tokens.is_empty());
}

#[test]
fn error_node_multiple_expected_tokens() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![10, 20, 30, 40, 50],
        actual: Some(99),
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert_eq!(node.expected.len(), 5);
    assert_eq!(node.expected[0], 10);
    assert_eq!(node.expected[4], 50);
}

#[test]
fn error_node_with_skipped_tokens() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 20,
        start_position: (1, 5),
        end_position: (2, 3),
        expected: vec![1],
        actual: Some(7),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![7, 8, 9],
    };
    assert_eq!(node.skipped_tokens.len(), 3);
    assert_eq!(node.skipped_tokens[0], 7);
    assert_eq!(node.skipped_tokens[2], 9);
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 1,
        end_byte: 2,
        start_position: (0, 1),
        end_position: (0, 2),
        expected: vec![5],
        actual: Some(6),
        recovery: RecoveryStrategy::ScopeRecovery,
        skipped_tokens: vec![],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 1);
    assert_eq!(cloned.end_byte, 2);
    assert_eq!(cloned.recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn error_node_debug_format() {
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
    let dbg = format!("{node:?}");
    assert!(dbg.contains("ErrorNode"));
    assert!(dbg.contains("PanicMode"));
}

// ── 20. ErrorRecoveryState record_error / get_error_nodes ──────────────────

#[test]
fn state_record_error_and_retrieve() {
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
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
}

#[test]
fn state_record_multiple_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (i, 0),
            (i, 5),
            vec![1],
            Some(2),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn state_get_error_nodes_empty_initially() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

// ── 21. add_recent_token / update_recent_tokens ────────────────────────────

#[test]
fn state_add_recent_token_basic() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.add_recent_token(42);
    // No panic; token tracked internally
}

#[test]
fn state_update_recent_tokens_uses_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(ir::SymbolId(99));
    // No panic
}

// ── 22. can_delete_token / can_replace_token ───────────────────────────────

#[test]
fn config_can_delete_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(ir::SymbolId(10));
    assert!(cfg.can_delete_token(ir::SymbolId(5)));
}

#[test]
fn config_cannot_delete_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(ir::SymbolId(10));
    assert!(!cfg.can_delete_token(ir::SymbolId(10)));
}

#[test]
fn config_can_replace_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(ir::SymbolId(10));
    assert!(cfg.can_replace_token(ir::SymbolId(5)));
}

#[test]
fn config_cannot_replace_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(ir::SymbolId(10));
    assert!(!cfg.can_replace_token(ir::SymbolId(10)));
}

#[test]
fn config_can_delete_explicitly_deletable_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(ir::SymbolId(10));
    cfg.deletable_tokens.insert(10);
    // Explicitly deletable overrides sync token protection
    assert!(cfg.can_delete_token(ir::SymbolId(10)));
}

// ── 23. Static helper methods ──────────────────────────────────────────────

#[test]
fn is_scope_delimiter_open_token() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delimiters));
}

#[test]
fn is_scope_delimiter_close_token() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delimiters));
}

#[test]
fn is_scope_delimiter_unrelated_token() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delimiters));
}

#[test]
fn is_scope_delimiter_empty_list() {
    assert!(!ErrorRecoveryState::is_scope_delimiter(1, &[]));
}

#[test]
fn is_matching_delimiter_valid_pair() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_matching_delimiter(1, 2, &delimiters));
}

#[test]
fn is_matching_delimiter_invalid_pair() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        1,
        4,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_reversed_pair() {
    let delimiters = vec![(1, 2)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        2,
        1,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_empty_delimiters() {
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 2, &[]));
}

// ── 24. Complex builder combinations ───────────────────────────────────────

#[test]
fn builder_full_configuration() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .max_consecutive_errors(25)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(10)
        .add_insertable_token(11)
        .add_deletable_token(50)
        .add_scope_delimiter(100, 101)
        .add_scope_delimiter(200, 201)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .enable_indentation_recovery(true)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
    assert_eq!(cfg.max_consecutive_errors, 25);
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert_eq!(cfg.insert_candidates.len(), 2);
    assert_eq!(cfg.deletable_tokens.len(), 1);
    assert_eq!(cfg.scope_delimiters.len(), 2);
    assert!(cfg.enable_phrase_recovery);
    assert!(cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_minimal_configuration_all_disabled() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(0)
        .max_consecutive_errors(0)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(false)
        .build();
    assert_eq!(cfg.max_panic_skip, 0);
    assert_eq!(cfg.max_consecutive_errors, 0);
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(!cfg.enable_indentation_recovery);
}

// ── 25. ErrorRecoveryState determine_recovery_strategy ─────────────────────

#[test]
fn state_determine_strategy_insertion_when_insertable() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn state_determine_strategy_panic_when_over_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // First call increments to 1 (within limit)
    let _ = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    // Second call increments to 2 (over limit of 1)
    let strategy = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn state_determine_strategy_deletion_for_wrong_token() {
    let cfg = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn state_determine_strategy_substitution_for_single_expected() {
    // Token 99 is a sync token, so it's not "clearly wrong" — skips deletion,
    // falls through to substitution (single expected token).
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(ir::SymbolId(99));
    cfg.enable_phrase_recovery = false;
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

// ── 26. ErrorNode with each RecoveryStrategy variant ───────────────────────

#[test]
fn error_node_with_panic_mode() {
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
    assert_eq!(node.recovery, RecoveryStrategy::PanicMode);
}

#[test]
fn error_node_with_token_insertion() {
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
    assert_eq!(node.recovery, RecoveryStrategy::TokenInsertion);
}

#[test]
fn error_node_with_token_substitution() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![5],
        actual: Some(6),
        recovery: RecoveryStrategy::TokenSubstitution,
        skipped_tokens: vec![],
    };
    assert_eq!(node.recovery, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn error_node_with_scope_recovery() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 11,
        start_position: (1, 0),
        end_position: (1, 1),
        expected: vec![2],
        actual: Some(4),
        recovery: RecoveryStrategy::ScopeRecovery,
        skipped_tokens: vec![],
    };
    assert_eq!(node.recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn error_node_with_indentation_recovery() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 4,
        start_position: (2, 0),
        end_position: (2, 4),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::IndentationRecovery,
        skipped_tokens: vec![],
    };
    assert_eq!(node.recovery, RecoveryStrategy::IndentationRecovery);
}

// ── 27. Config clone ───────────────────────────────────────────────────────

#[test]
fn config_clone_preserves_all_fields() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(77)
        .max_consecutive_errors(12)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(4, 5)
        .enable_indentation_recovery(true)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 77);
    assert_eq!(cloned.max_consecutive_errors, 12);
    assert_eq!(cloned.sync_tokens.len(), 1);
    assert_eq!(cloned.insert_candidates.len(), 1);
    assert!(cloned.deletable_tokens.contains(&3));
    assert_eq!(cloned.scope_delimiters, vec![(4, 5)]);
    assert!(cloned.enable_indentation_recovery);
    assert!(!cloned.enable_phrase_recovery);
    assert!(!cloned.enable_scope_recovery);
}

// ── 28. Nested scope operations ────────────────────────────────────────────

#[test]
fn state_nested_scope_same_delimiter_type() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    state.push_scope(10);
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_matching_nested() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    state.push_scope(20);
    // Only matching close works
    assert!(state.pop_scope(21));
    assert!(!state.pop_scope(21)); // top is now 10, not 20
    assert!(state.pop_scope(11));
}

// ── 29. ErrorRecoveryConfigBuilder Default trait ───────────────────────────

#[test]
fn builder_default_trait_same_as_new() {
    let from_default: ErrorRecoveryConfigBuilder = Default::default();
    let from_new = ErrorRecoveryConfigBuilder::new();
    let cfg_d = from_default.build();
    let cfg_n = from_new.build();
    assert_eq!(cfg_d.max_panic_skip, cfg_n.max_panic_skip);
    assert_eq!(cfg_d.max_consecutive_errors, cfg_n.max_consecutive_errors);
}

// ── 30. Config debug format ────────────────────────────────────────────────

#[test]
fn config_debug_format_contains_field_names() {
    let cfg = ErrorRecoveryConfig::default();
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("max_panic_skip"));
    assert!(dbg.contains("max_consecutive_errors"));
}

// ── 31. clear_errors ───────────────────────────────────────────────────────

#[test]
fn state_clear_errors_removes_all() {
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
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 2);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// ── 32. reset_consecutive_errors ───────────────────────────────────────────

#[test]
fn state_reset_consecutive_errors_clears_count() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

// ── 33. ErrorNode actual field variants ────────────────────────────────────

#[test]
fn error_node_actual_none() {
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
    assert!(node.actual.is_none());
}

#[test]
fn error_node_actual_some() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 0,
        start_position: (0, 0),
        end_position: (0, 0),
        expected: vec![],
        actual: Some(42),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert_eq!(node.actual, Some(42));
}

// ── 34. ErrorNode positions ────────────────────────────────────────────────

#[test]
fn error_node_multiline_positions() {
    let node = ErrorNode {
        start_byte: 100,
        end_byte: 250,
        start_position: (5, 10),
        end_position: (8, 3),
        expected: vec![1, 2, 3],
        actual: Some(99),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![99, 100, 101],
    };
    assert_eq!(node.start_position.0, 5);
    assert_eq!(node.start_position.1, 10);
    assert_eq!(node.end_position.0, 8);
    assert_eq!(node.end_position.1, 3);
}

#[test]
fn error_node_zero_length_span() {
    let node = ErrorNode {
        start_byte: 50,
        end_byte: 50,
        start_position: (3, 7),
        end_position: (3, 7),
        expected: vec![1],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_byte, node.end_byte);
    assert_eq!(node.start_position, node.end_position);
}

// ── 35. Builder add_sync_token_sym ─────────────────────────────────────────

#[test]
fn builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(ir::SymbolId(55))
        .build();
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.sync_tokens[0], ir::SymbolId(55));
}

// ── 36. Builder add_insertable_token_sym ───────────────────────────────────

#[test]
fn builder_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(ir::SymbolId(77))
        .build();
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert_eq!(cfg.insert_candidates[0], ir::SymbolId(77));
}
