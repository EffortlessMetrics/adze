//! Comprehensive tests for the `ErrorRecoveryConfigBuilder` API — builder
//! pattern, configuration validation, and type trait coverage.
//!
//! 80+ tests organised into logical sections:
//!   1.  Builder defaults
//!   2.  `max_panic_skip`
//!   3.  `max_consecutive_errors` / `set_max_recovery_attempts`
//!   4.  `add_sync_token` / `add_sync_token_sym`
//!   5.  `add_insertable_token` / `add_insertable_token_sym`
//!   6.  `add_deletable_token`
//!   7.  `add_scope_delimiter`
//!   8.  Enable/disable toggles
//!   9.  Chained builder calls
//!  10.  `ErrorRecoveryConfig` traits (Debug, Clone, Default)
//!  11.  `RecoveryStrategy` traits (Debug, Clone, Copy, PartialEq, Eq)
//!  12.  `ErrorNode` creation and field access
//!  13.  `ErrorRecoveryState` basic operations
//!  14.  `can_delete_token` / `can_replace_token`
//!  15.  Static helpers (`is_scope_delimiter`, `is_matching_delimiter`)

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};

// ============================================================================
// 1. Builder::new() defaults
// ============================================================================

#[test]
fn builder_new_default_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn builder_new_default_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn builder_new_default_max_token_deletions() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn builder_new_default_max_token_insertions() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn builder_new_default_sync_tokens_empty() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn builder_new_default_insert_candidates_empty() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn builder_new_default_deletable_tokens_empty() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.deletable_tokens.is_empty());
}

#[test]
fn builder_new_default_scope_delimiters_empty() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.scope_delimiters.is_empty());
}

#[test]
fn builder_new_default_phrase_recovery_enabled() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn builder_new_default_scope_recovery_enabled() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn builder_new_default_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfigBuilder::new().build();
    assert!(!cfg.enable_indentation_recovery);
}

// ============================================================================
// 2. max_panic_skip
// ============================================================================

#[test]
fn builder_max_panic_skip_sets_value() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn builder_max_panic_skip_zero() {
    let cfg = ErrorRecoveryConfigBuilder::new().max_panic_skip(0).build();
    assert_eq!(cfg.max_panic_skip, 0);
}

#[test]
fn builder_max_panic_skip_large() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(usize::MAX)
        .build();
    assert_eq!(cfg.max_panic_skip, usize::MAX);
}

#[test]
fn builder_max_panic_skip_last_wins() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .max_panic_skip(99)
        .build();
    assert_eq!(cfg.max_panic_skip, 99);
}

// ============================================================================
// 3. max_consecutive_errors / set_max_recovery_attempts
// ============================================================================

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
fn builder_set_max_recovery_attempts_maps_to_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(10)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn builder_set_max_recovery_attempts_overrides_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .set_max_recovery_attempts(15)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 15);
}

#[test]
fn builder_max_consecutive_errors_overrides_set_max_recovery_attempts() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(15)
        .max_consecutive_errors(3)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 3);
}

// ============================================================================
// 4. add_sync_token / add_sync_token_sym
// ============================================================================

#[test]
fn builder_add_sync_token_single() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(1).build();
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.sync_tokens[0].0, 1);
}

#[test]
fn builder_add_sync_token_multiple() {
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

#[test]
fn builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(adze_ir::SymbolId(42))
        .build();
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.sync_tokens[0].0, 42);
}

#[test]
fn builder_add_sync_token_duplicates_not_deduplicated() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(7)
        .add_sync_token(7)
        .build();
    // SmallVec does not deduplicate
    assert_eq!(cfg.sync_tokens.len(), 2);
}

// ============================================================================
// 5. add_insertable_token / add_insertable_token_sym
// ============================================================================

#[test]
fn builder_add_insertable_token_single() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(42)
        .build();
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert_eq!(cfg.insert_candidates[0].0, 42);
}

#[test]
fn builder_add_insertable_token_multiple() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .add_insertable_token(20)
        .add_insertable_token(30)
        .build();
    assert_eq!(cfg.insert_candidates.len(), 3);
}

#[test]
fn builder_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(adze_ir::SymbolId(99))
        .build();
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert_eq!(cfg.insert_candidates[0].0, 99);
}

// ============================================================================
// 6. add_deletable_token
// ============================================================================

#[test]
fn builder_add_deletable_token_single() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .build();
    assert!(cfg.deletable_tokens.contains(&5));
    assert_eq!(cfg.deletable_tokens.len(), 1);
}

#[test]
fn builder_add_deletable_token_multiple() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(1)
        .add_deletable_token(2)
        .add_deletable_token(3)
        .build();
    assert_eq!(cfg.deletable_tokens.len(), 3);
}

#[test]
fn builder_add_deletable_token_duplicate_deduplicates() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .add_deletable_token(5)
        .build();
    assert_eq!(cfg.deletable_tokens.len(), 1);
}

// ============================================================================
// 7. add_scope_delimiter
// ============================================================================

#[test]
fn builder_add_scope_delimiter_single() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(40, 41)]);
}

#[test]
fn builder_add_scope_delimiter_multiple() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .add_scope_delimiter(123, 125)
        .build();
    assert_eq!(cfg.scope_delimiters.len(), 3);
    assert_eq!(cfg.scope_delimiters[0], (40, 41));
    assert_eq!(cfg.scope_delimiters[1], (91, 93));
    assert_eq!(cfg.scope_delimiters[2], (123, 125));
}

#[test]
fn builder_add_scope_delimiter_preserves_order() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(3, 4)
        .add_scope_delimiter(1, 2)
        .build();
    assert_eq!(cfg.scope_delimiters[0], (3, 4));
    assert_eq!(cfg.scope_delimiters[1], (1, 2));
}

// ============================================================================
// 8. Enable/disable toggles
// ============================================================================

#[test]
fn builder_enable_indentation_recovery_true() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_enable_indentation_recovery_false() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(false)
        .build();
    assert!(!cfg.enable_indentation_recovery);
}

#[test]
fn builder_disable_phrase_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!cfg.enable_phrase_recovery);
}

#[test]
fn builder_disable_scope_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!cfg.enable_scope_recovery);
}

#[test]
fn builder_toggle_phrase_recovery_last_wins() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_phrase_recovery(true)
        .build();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn builder_toggle_scope_recovery_last_wins() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .enable_scope_recovery(true)
        .build();
    assert!(cfg.enable_scope_recovery);
}

// ============================================================================
// 9. Chained builder calls — single build()
// ============================================================================

#[test]
fn builder_full_chain_single_build() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .max_consecutive_errors(20)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(10)
        .add_deletable_token(50)
        .add_scope_delimiter(40, 41)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .enable_indentation_recovery(true)
        .build();
    assert_eq!(cfg.max_panic_skip, 100);
    assert_eq!(cfg.max_consecutive_errors, 20);
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert_eq!(cfg.deletable_tokens.len(), 1);
    assert_eq!(cfg.scope_delimiters.len(), 1);
    assert!(cfg.enable_phrase_recovery);
    assert!(cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_minimal_all_disabled() {
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

#[test]
fn builder_only_tokens_no_toggles() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert!(cfg.deletable_tokens.contains(&3));
    // Defaults preserved
    assert!(cfg.enable_phrase_recovery);
    assert!(cfg.enable_scope_recovery);
}

// ============================================================================
// 10. ErrorRecoveryConfig traits — Debug, Clone, Default
// ============================================================================

#[test]
fn config_debug_format_contains_type_name() {
    let cfg = ErrorRecoveryConfig::default();
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("ErrorRecoveryConfig"));
}

#[test]
fn config_debug_format_contains_field_values() {
    let cfg = ErrorRecoveryConfig::default();
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("max_panic_skip"));
    assert!(dbg.contains("50"));
}

#[test]
fn config_clone_independence() {
    let mut original = ErrorRecoveryConfig::default();
    let cloned = original.clone();
    original.max_panic_skip = 999;
    assert_eq!(cloned.max_panic_skip, 50);
}

#[test]
fn config_clone_preserves_scope_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .build();
    let cloned = cfg.clone();
    assert_eq!(cloned.scope_delimiters, vec![(1, 2)]);
}

#[test]
fn config_default_matches_builder_new() {
    let from_default = ErrorRecoveryConfig::default();
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(from_default.max_panic_skip, from_builder.max_panic_skip);
    assert_eq!(
        from_default.max_consecutive_errors,
        from_builder.max_consecutive_errors
    );
    assert_eq!(
        from_default.enable_phrase_recovery,
        from_builder.enable_phrase_recovery
    );
    assert_eq!(
        from_default.enable_scope_recovery,
        from_builder.enable_scope_recovery
    );
    assert_eq!(
        from_default.enable_indentation_recovery,
        from_builder.enable_indentation_recovery
    );
}

// ============================================================================
// 11. RecoveryStrategy traits — Debug, Clone, Copy, PartialEq, Eq
// ============================================================================

#[test]
fn strategy_debug_panic_mode() {
    assert_eq!(format!("{:?}", RecoveryStrategy::PanicMode), "PanicMode");
}

#[test]
fn strategy_debug_token_insertion() {
    assert_eq!(
        format!("{:?}", RecoveryStrategy::TokenInsertion),
        "TokenInsertion"
    );
}

#[test]
fn strategy_debug_token_deletion() {
    assert_eq!(
        format!("{:?}", RecoveryStrategy::TokenDeletion),
        "TokenDeletion"
    );
}

#[test]
fn strategy_debug_token_substitution() {
    assert_eq!(
        format!("{:?}", RecoveryStrategy::TokenSubstitution),
        "TokenSubstitution"
    );
}

#[test]
fn strategy_debug_phrase_level() {
    assert_eq!(
        format!("{:?}", RecoveryStrategy::PhraseLevel),
        "PhraseLevel"
    );
}

#[test]
fn strategy_debug_scope_recovery() {
    assert_eq!(
        format!("{:?}", RecoveryStrategy::ScopeRecovery),
        "ScopeRecovery"
    );
}

#[test]
fn strategy_debug_indentation_recovery() {
    assert_eq!(
        format!("{:?}", RecoveryStrategy::IndentationRecovery),
        "IndentationRecovery"
    );
}

#[test]
fn strategy_copy_semantics() {
    let a = RecoveryStrategy::PanicMode;
    let b = a; // Copy, not move
    assert_eq!(a, b);
}

#[test]
fn strategy_clone_all_variants() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for v in variants {
        let cloned = v; // Copy
        assert_eq!(v, cloned);
    }
}

#[test]
fn strategy_eq_same_variant() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn strategy_ne_different_variants() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion);
    assert_ne!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn strategy_all_seven_variants_distinct() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
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

// ============================================================================
// 12. ErrorNode creation and field access
// ============================================================================

#[test]
fn error_node_basic_field_access() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 10,
        start_position: (1, 5),
        end_position: (1, 10),
        expected: vec![1, 2, 3],
        actual: Some(99),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![99],
    };
    assert_eq!(node.start_byte, 5);
    assert_eq!(node.end_byte, 10);
    assert_eq!(node.start_position, (1, 5));
    assert_eq!(node.end_position, (1, 10));
    assert_eq!(node.expected, vec![1, 2, 3]);
    assert_eq!(node.actual, Some(99));
    assert_eq!(node.recovery, RecoveryStrategy::TokenDeletion);
    assert_eq!(node.skipped_tokens, vec![99]);
}

#[test]
fn error_node_actual_none() {
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
fn error_node_empty_expected() {
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
}

#[test]
fn error_node_clone_independence() {
    let node = ErrorNode {
        start_byte: 1,
        end_byte: 2,
        start_position: (0, 1),
        end_position: (0, 2),
        expected: vec![10],
        actual: Some(20),
        recovery: RecoveryStrategy::ScopeRecovery,
        skipped_tokens: vec![],
    };
    let mut cloned = node.clone();
    cloned.start_byte = 100;
    assert_eq!(node.start_byte, 1);
    assert_eq!(cloned.start_byte, 100);
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

#[test]
fn error_node_with_each_strategy_variant() {
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

#[test]
fn error_node_large_byte_range() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1_000_000,
        start_position: (0, 0),
        end_position: (5000, 0),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: (0..100).collect(),
    };
    assert_eq!(node.end_byte, 1_000_000);
    assert_eq!(node.skipped_tokens.len(), 100);
}

// ============================================================================
// 13. ErrorRecoveryState basic operations
// ============================================================================

#[test]
fn state_new_error_nodes_empty() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_new_does_not_give_up() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_increment_and_check_threshold() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_error_count_clears() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_push_scope_registered_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    assert_eq!(state.pop_scope_test(), Some(40));
}

#[test]
fn state_push_scope_unregistered_no_effect() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_matching_close() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    assert!(state.pop_scope(41));
}

#[test]
fn state_pop_scope_empty_stack_fails() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.pop_scope(41));
}

#[test]
fn state_record_error_and_retrieve() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1],
        Some(2),
        RecoveryStrategy::TokenDeletion,
        vec![2],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
}

#[test]
fn state_add_recent_token_no_panic() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..20 {
        state.add_recent_token(i);
    }
}

#[test]
fn state_update_recent_tokens_via_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(adze_ir::SymbolId(42));
}

#[test]
fn state_clear_errors_empties_nodes() {
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
    assert_eq!(state.get_error_nodes().len(), 1);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// ============================================================================
// 14. can_delete_token / can_replace_token
// ============================================================================

#[test]
fn config_can_delete_non_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(cfg.can_delete_token(adze_ir::SymbolId(5)));
}

#[test]
fn config_cannot_delete_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(!cfg.can_delete_token(adze_ir::SymbolId(10)));
}

#[test]
fn config_can_delete_explicitly_deletable_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(10)
        .add_deletable_token(10)
        .build();
    assert!(cfg.can_delete_token(adze_ir::SymbolId(10)));
}

#[test]
fn config_can_replace_non_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(cfg.can_replace_token(adze_ir::SymbolId(5)));
}

#[test]
fn config_cannot_replace_sync_token() {
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(10).build();
    assert!(!cfg.can_replace_token(adze_ir::SymbolId(10)));
}

// ============================================================================
// 15. Static helpers
// ============================================================================

#[test]
fn is_scope_delimiter_open() {
    let delimiters = vec![(1, 2)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delimiters));
}

#[test]
fn is_scope_delimiter_close() {
    let delimiters = vec![(1, 2)];
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delimiters));
}

#[test]
fn is_scope_delimiter_unrelated() {
    let delimiters = vec![(1, 2)];
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
    assert!(ErrorRecoveryState::is_matching_delimiter(3, 4, &delimiters));
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
fn is_matching_delimiter_reversed() {
    let delimiters = vec![(1, 2)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        2,
        1,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_empty() {
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 2, &[]));
}

// ============================================================================
// 16. Builder Default trait impl
// ============================================================================

#[test]
fn builder_default_trait_matches_new() {
    let from_new = ErrorRecoveryConfigBuilder::new().build();
    let from_default = ErrorRecoveryConfigBuilder::default().build();
    assert_eq!(from_new.max_panic_skip, from_default.max_panic_skip);
    assert_eq!(
        from_new.max_consecutive_errors,
        from_default.max_consecutive_errors
    );
}

// ============================================================================
// 17. State determine_recovery_strategy
// ============================================================================

#[test]
fn state_strategy_insertion_when_insertable() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn state_strategy_panic_when_over_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let _ = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    let strategy = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn state_strategy_deletion_for_wrong_token() {
    let cfg = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

// ============================================================================
// 18. Scope stack LIFO ordering
// ============================================================================

#[test]
fn state_scope_stack_lifo_order() {
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

// ============================================================================
// 19. Zero-threshold gives up immediately
// ============================================================================

#[test]
fn state_zero_threshold_gives_up_immediately() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    let state = ErrorRecoveryState::new(cfg);
    assert!(state.should_give_up());
}
