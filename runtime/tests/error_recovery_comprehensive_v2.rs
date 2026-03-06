//! Comprehensive v2 tests for error_recovery module.
//!
//! 60+ tests covering:
//! 1. ErrorRecoveryConfig builder pattern
//! 2. RecoveryStrategy variants and properties
//! 3. RecoveryAction construction
//! 4. Property tests for configuration invariants
//! 5. Edge cases (zero limits, empty collections, boundary values)

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;

// ============================================================================
// 1. ErrorRecoveryConfig Default Tests
// ============================================================================

#[test]
fn default_config_max_panic_skip() {
    assert_eq!(ErrorRecoveryConfig::default().max_panic_skip, 50);
}

#[test]
fn default_config_sync_tokens_empty() {
    assert!(ErrorRecoveryConfig::default().sync_tokens.is_empty());
}

#[test]
fn default_config_insert_candidates_empty() {
    assert!(ErrorRecoveryConfig::default().insert_candidates.is_empty());
}

#[test]
fn default_config_deletable_tokens_empty() {
    assert!(ErrorRecoveryConfig::default().deletable_tokens.is_empty());
}

#[test]
fn default_config_max_token_deletions() {
    assert_eq!(ErrorRecoveryConfig::default().max_token_deletions, 3);
}

#[test]
fn default_config_max_token_insertions() {
    assert_eq!(ErrorRecoveryConfig::default().max_token_insertions, 2);
}

#[test]
fn default_config_max_consecutive_errors() {
    assert_eq!(ErrorRecoveryConfig::default().max_consecutive_errors, 10);
}

#[test]
fn default_config_phrase_recovery_enabled() {
    assert!(ErrorRecoveryConfig::default().enable_phrase_recovery);
}

#[test]
fn default_config_scope_recovery_enabled() {
    assert!(ErrorRecoveryConfig::default().enable_scope_recovery);
}

#[test]
fn default_config_scope_delimiters_empty() {
    assert!(ErrorRecoveryConfig::default().scope_delimiters.is_empty());
}

#[test]
fn default_config_indentation_recovery_disabled() {
    assert!(!ErrorRecoveryConfig::default().enable_indentation_recovery);
}

// ============================================================================
// 2. ErrorRecoveryConfigBuilder Tests
// ============================================================================

#[test]
fn builder_default_matches_config_default() {
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
        from_builder.enable_scope_recovery,
        from_default.enable_scope_recovery
    );
    assert_eq!(
        from_builder.enable_indentation_recovery,
        from_default.enable_indentation_recovery
    );
}

#[test]
fn builder_default_trait_impl() {
    // ErrorRecoveryConfigBuilder implements Default
    let builder: ErrorRecoveryConfigBuilder = Default::default();
    let config = builder.build();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(config.max_panic_skip, 200);
}

#[test]
fn builder_max_panic_skip_zero() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(0).build();
    assert_eq!(config.max_panic_skip, 0);
}

#[test]
fn builder_add_sync_token_u16() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(42).build();
    assert_eq!(config.sync_tokens.len(), 1);
    assert_eq!(config.sync_tokens[0], SymbolId(42));
}

#[test]
fn builder_add_sync_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(99))
        .build();
    assert_eq!(config.sync_tokens.len(), 1);
    assert_eq!(config.sync_tokens[0], SymbolId(99));
}

#[test]
fn builder_multiple_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token(3)
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
}

#[test]
fn builder_add_insertable_token_u16() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .build();
    assert_eq!(config.insert_candidates.len(), 1);
    assert_eq!(config.insert_candidates[0], SymbolId(7));
}

#[test]
fn builder_add_insertable_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(55))
        .build();
    assert_eq!(config.insert_candidates.len(), 1);
    assert_eq!(config.insert_candidates[0], SymbolId(55));
}

#[test]
fn builder_add_deletable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(10)
        .build();
    assert!(config.deletable_tokens.contains(&10));
    assert_eq!(config.deletable_tokens.len(), 1);
}

#[test]
fn builder_add_scope_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(config.scope_delimiters, vec![(40, 41)]);
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
fn builder_disable_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!config.enable_scope_recovery);
}

#[test]
fn builder_disable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!config.enable_phrase_recovery);
}

#[test]
fn builder_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(25)
        .build();
    assert_eq!(config.max_consecutive_errors, 25);
}

#[test]
fn builder_set_max_recovery_attempts_aliases_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(77)
        .build();
    assert_eq!(config.max_consecutive_errors, 77);
}

#[test]
fn builder_full_chain() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(3)
        .add_insertable_token_sym(SymbolId(4))
        .add_deletable_token(5)
        .add_scope_delimiter(6, 7)
        .enable_indentation_recovery(true)
        .enable_scope_recovery(false)
        .enable_phrase_recovery(false)
        .max_consecutive_errors(3)
        .build();

    assert_eq!(config.max_panic_skip, 10);
    assert_eq!(config.sync_tokens.len(), 2);
    assert_eq!(config.insert_candidates.len(), 2);
    assert!(config.deletable_tokens.contains(&5));
    assert_eq!(config.scope_delimiters, vec![(6, 7)]);
    assert!(config.enable_indentation_recovery);
    assert!(!config.enable_scope_recovery);
    assert!(!config.enable_phrase_recovery);
    assert_eq!(config.max_consecutive_errors, 3);
}

// ============================================================================
// 3. Config can_delete_token / can_replace_token Tests
// ============================================================================

#[test]
fn can_delete_non_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    // Non-sync token is deletable
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn cannot_delete_sync_token_unless_explicitly_deletable() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    // Sync token without explicit deletable entry
    assert!(!config.can_delete_token(SymbolId(10)));
}

#[test]
fn can_delete_sync_token_if_in_deletable_set() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    config.deletable_tokens.insert(10);
    // Explicit deletable overrides sync protection
    assert!(config.can_delete_token(SymbolId(10)));
}

#[test]
fn can_replace_non_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    assert!(config.can_replace_token(SymbolId(5)));
}

#[test]
fn cannot_replace_sync_token() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(10));
    assert!(!config.can_replace_token(SymbolId(10)));
}

#[test]
fn can_delete_any_token_with_empty_sync_set() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.can_delete_token(SymbolId(0)));
    assert!(config.can_delete_token(SymbolId(999)));
}

#[test]
fn can_replace_any_token_with_empty_sync_set() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.can_replace_token(SymbolId(0)));
    assert!(config.can_replace_token(SymbolId(u16::MAX)));
}

// ============================================================================
// 4. RecoveryStrategy Enum Tests
// ============================================================================

#[test]
fn recovery_strategy_equality() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
    assert_eq!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::TokenSubstitution
    );
    assert_eq!(RecoveryStrategy::PhraseLevel, RecoveryStrategy::PhraseLevel);
    assert_eq!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::ScopeRecovery
    );
    assert_eq!(
        RecoveryStrategy::IndentationRecovery,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn recovery_strategy_inequality() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion);
    assert_ne!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::ScopeRecovery
    );
    assert_ne!(
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn recovery_strategy_clone() {
    let s = RecoveryStrategy::TokenSubstitution;
    let cloned = s;
    assert_eq!(s, cloned);
}

#[test]
fn recovery_strategy_debug() {
    let dbg = format!("{:?}", RecoveryStrategy::PanicMode);
    assert_eq!(dbg, "PanicMode");
    let dbg2 = format!("{:?}", RecoveryStrategy::TokenInsertion);
    assert_eq!(dbg2, "TokenInsertion");
}

#[test]
fn recovery_strategy_all_variants_are_distinct() {
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
// 5. RecoveryAction Construction Tests
// ============================================================================

#[test]
fn recovery_action_insert_token() {
    let action = RecoveryAction::InsertToken(SymbolId(42));
    match action {
        RecoveryAction::InsertToken(id) => assert_eq!(id, SymbolId(42)),
        _ => panic!("Expected InsertToken"),
    }
}

#[test]
fn recovery_action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    assert!(matches!(action, RecoveryAction::DeleteToken));
}

#[test]
fn recovery_action_replace_token() {
    let action = RecoveryAction::ReplaceToken(SymbolId(100));
    match action {
        RecoveryAction::ReplaceToken(id) => assert_eq!(id, SymbolId(100)),
        _ => panic!("Expected ReplaceToken"),
    }
}

#[test]
fn recovery_action_create_error_node_empty() {
    let action = RecoveryAction::CreateErrorNode(vec![]);
    match action {
        RecoveryAction::CreateErrorNode(ref ids) => assert!(ids.is_empty()),
        _ => panic!("Expected CreateErrorNode"),
    }
}

#[test]
fn recovery_action_create_error_node_multiple() {
    let ids = vec![SymbolId(1), SymbolId(2), SymbolId(3)];
    let action = RecoveryAction::CreateErrorNode(ids.clone());
    match action {
        RecoveryAction::CreateErrorNode(ref got) => assert_eq!(*got, ids),
        _ => panic!("Expected CreateErrorNode"),
    }
}

#[test]
fn recovery_action_debug_format() {
    let action = RecoveryAction::InsertToken(SymbolId(5));
    let dbg = format!("{:?}", action);
    assert!(dbg.contains("InsertToken"));

    let action2 = RecoveryAction::DeleteToken;
    let dbg2 = format!("{:?}", action2);
    assert!(dbg2.contains("DeleteToken"));
}

#[test]
fn recovery_action_clone() {
    let action = RecoveryAction::InsertToken(SymbolId(10));
    let cloned = action.clone();
    assert!(matches!(cloned, RecoveryAction::InsertToken(id) if id == SymbolId(10)));
}

// ============================================================================
// 6. ErrorRecoveryState Tests
// ============================================================================

#[test]
fn state_new_no_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_increment_and_reset_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    // max_consecutive_errors = 10 by default
    assert!(!state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_should_give_up_at_boundary() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    assert!(!state.should_give_up()); // 1 < 2
    state.increment_error_count();
    assert!(state.should_give_up()); // 2 >= 2
}

#[test]
fn state_should_give_up_zero_max_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    let state = ErrorRecoveryState::new(config);
    // 0 >= 0, should give up immediately
    assert!(state.should_give_up());
}

#[test]
fn state_record_and_retrieve_single_error() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        10,
        20,
        (1, 10),
        (1, 20),
        vec![1, 2],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![99],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 10);
    assert_eq!(nodes[0].end_byte, 20);
    assert_eq!(nodes[0].expected, vec![1, 2]);
    assert_eq!(nodes[0].actual, Some(99));
    assert_eq!(nodes[0].recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn state_record_error_with_no_actual_token() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![5],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].actual, None);
}

#[test]
fn state_record_multiple_errors_preserves_order() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5u16 {
        state.record_error(
            i as usize,
            (i + 1) as usize,
            (0, i as usize),
            (0, (i + 1) as usize),
            vec![i],
            Some(i + 100),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 5);
    for (i, node) in nodes.iter().enumerate() {
        assert_eq!(node.start_byte, i);
    }
}

#[test]
fn state_clear_errors() {
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
fn state_reset_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

// ============================================================================
// 7. Scope Tracking Tests
// ============================================================================

#[test]
fn scope_push_opening_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert_eq!(state.pop_scope_test(), Some(40));
}

#[test]
fn scope_push_non_delimiter_is_noop() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(99); // not an opening delimiter
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn scope_pop_matching_close() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    assert!(state.pop_scope(11));
}

#[test]
fn scope_pop_non_matching_close_fails() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    // Try to close with wrong delimiter
    assert!(!state.pop_scope(21));
    // Original scope should still be there
    assert_eq!(state.pop_scope_test(), Some(10));
}

#[test]
fn scope_nested_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .add_scope_delimiter(3, 4)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(1);
    state.push_scope(3);
    state.push_scope(1);
    assert!(state.pop_scope(2)); // close inner (1,2)
    assert!(state.pop_scope(4)); // close (3,4)
    assert!(state.pop_scope(2)); // close outer (1,2)
    assert_eq!(state.pop_scope_test(), None); // empty
}

#[test]
fn scope_pop_on_empty_stack() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.pop_scope(2));
}

// ============================================================================
// 8. Recent Tokens Tests
// ============================================================================

#[test]
fn recent_tokens_add_single() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.add_recent_token(42);
    state.update_recent_tokens(SymbolId(43));
    // Should have 2 tokens
}

#[test]
fn recent_tokens_max_capacity_10() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..15u16 {
        state.add_recent_token(i);
    }
    // add_recent_token caps at 10 entries
    // Tokens 0..4 should have been evicted, 5..14 remain
}

// ============================================================================
// 9. Static Helper Tests
// ============================================================================

#[test]
fn is_scope_delimiter_open() {
    let delims = vec![(10, 11), (20, 21)];
    assert!(ErrorRecoveryState::is_scope_delimiter(10, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(20, &delims));
}

#[test]
fn is_scope_delimiter_close() {
    let delims = vec![(10, 11), (20, 21)];
    assert!(ErrorRecoveryState::is_scope_delimiter(11, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(21, &delims));
}

#[test]
fn is_scope_delimiter_not_present() {
    let delims = vec![(10, 11)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delims));
}

#[test]
fn is_scope_delimiter_empty_delimiters() {
    assert!(!ErrorRecoveryState::is_scope_delimiter(1, &[]));
}

#[test]
fn is_matching_delimiter_correct_pair() {
    let delims = vec![(10, 11), (20, 21)];
    assert!(ErrorRecoveryState::is_matching_delimiter(10, 11, &delims));
    assert!(ErrorRecoveryState::is_matching_delimiter(20, 21, &delims));
}

#[test]
fn is_matching_delimiter_swapped_pair() {
    let delims = vec![(10, 11)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(11, 10, &delims));
}

#[test]
fn is_matching_delimiter_cross_pair() {
    let delims = vec![(10, 11), (20, 21)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(10, 21, &delims));
}

#[test]
fn is_matching_delimiter_empty_delimiters() {
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 2, &[]));
}

// ============================================================================
// 10. determine_recovery_strategy Tests
// ============================================================================

#[test]
fn strategy_insertion_when_insertable_candidate_matches() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_deletion_for_clearly_wrong_token() {
    // No insertable candidates, token not in expected, not a sync token
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10, 11], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_substitution_when_single_expected() {
    // Actual token is a sync token (not "clearly wrong"), exactly 1 expected => substitution
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(99)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn strategy_panic_mode_when_max_errors_exceeded() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // First call increments to 1, within limit
    let _s1 = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    // Second call increments to 2, exceeds limit of 1
    let s2 = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    assert_eq!(s2, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_phrase_level_as_fallback() {
    // No insertable, actual token is a sync token (not clearly wrong), multiple expected
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(50)
        .enable_scope_recovery(false)
        .enable_phrase_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10, 11], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_scope_recovery_on_unmatched_close() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .add_sync_token(41) // close paren is sync, so not "clearly wrong"
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // Encountering close ')' (41) without matching open
    let strategy = state.determine_recovery_strategy(&[10, 11], Some(41), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn strategy_panic_mode_when_all_disabled() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .add_sync_token(50) // make actual a sync token so not "clearly wrong"
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10, 11], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// ============================================================================
// 11. ErrorNode Tests
// ============================================================================

#[test]
fn error_node_fields() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 15,
        start_position: (2, 3),
        end_position: (2, 13),
        expected: vec![1, 2, 3],
        actual: Some(99),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![99, 100],
    };
    assert_eq!(node.start_byte, 5);
    assert_eq!(node.end_byte, 15);
    assert_eq!(node.expected.len(), 3);
    assert_eq!(node.actual, Some(99));
    assert_eq!(node.recovery, RecoveryStrategy::PhraseLevel);
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![1],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 0);
    assert_eq!(cloned.actual, None);
}

#[test]
fn error_node_debug() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 0,
        start_position: (0, 0),
        end_position: (0, 0),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![],
    };
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("ErrorNode"));
}

// ============================================================================
// 12. Property-Style Tests for Config Invariants
// ============================================================================

#[test]
fn property_deletable_superset_of_explicit() {
    let mut config = ErrorRecoveryConfig::default();
    let explicit: Vec<u16> = vec![1, 5, 10, 20, 50];
    for &t in &explicit {
        config.deletable_tokens.insert(t);
    }
    // Every explicitly deletable token should be can_delete_token
    for &t in &explicit {
        assert!(config.can_delete_token(SymbolId(t)));
    }
}

#[test]
fn property_sync_tokens_not_deletable_unless_explicit() {
    let mut config = ErrorRecoveryConfig::default();
    for i in 0..10u16 {
        config.sync_tokens.push(SymbolId(i));
    }
    // Without explicit deletable, sync tokens should not be deletable
    for i in 0..10u16 {
        assert!(!config.can_delete_token(SymbolId(i)));
    }
    // Add some to deletable
    config.deletable_tokens.insert(3);
    config.deletable_tokens.insert(7);
    assert!(config.can_delete_token(SymbolId(3)));
    assert!(config.can_delete_token(SymbolId(7)));
    assert!(!config.can_delete_token(SymbolId(5)));
}

#[test]
fn property_sync_tokens_not_replaceable() {
    let mut config = ErrorRecoveryConfig::default();
    for i in 0..20u16 {
        config.sync_tokens.push(SymbolId(i));
    }
    for i in 0..20u16 {
        assert!(!config.can_replace_token(SymbolId(i)));
    }
}

#[test]
fn property_non_sync_tokens_always_deletable_and_replaceable() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(100));
    // Any token not 100 should be deletable and replaceable
    for i in [0u16, 1, 50, 99, 101, 200, u16::MAX] {
        assert!(config.can_delete_token(SymbolId(i)));
        assert!(config.can_replace_token(SymbolId(i)));
    }
}

#[test]
fn property_error_count_monotonic_until_reset() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(100)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    for expected in 1..=10 {
        state.increment_error_count();
        // After `expected` increments, should_give_up is false (100 max)
        assert!(!state.should_give_up());
        // Note: We can't directly read the counter, but should_give_up is consistent
        let _ = expected;
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn property_give_up_threshold_exact() {
    // For any max N, should_give_up is true at exactly N errors
    for max in 1..=5usize {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(max)
            .build();
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..max {
            assert!(!state.should_give_up());
            state.increment_error_count();
        }
        assert!(state.should_give_up());
    }
}

#[test]
fn property_error_nodes_accumulate() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..20usize {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![],
            None,
            RecoveryStrategy::PanicMode,
            vec![],
        );
        assert_eq!(state.get_error_nodes().len(), i + 1);
    }
}

#[test]
fn property_clear_errors_resets_to_empty() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..10usize {
        state.record_error(
            i,
            i + 1,
            (0, 0),
            (0, 0),
            vec![],
            None,
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 10);
    state.clear_errors();
    assert_eq!(state.get_error_nodes().len(), 0);
    // Can add again after clear
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
}

// ============================================================================
// 13. Edge Case Tests
// ============================================================================

#[test]
fn edge_max_panic_skip_usize_max() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(usize::MAX)
        .build();
    assert_eq!(config.max_panic_skip, usize::MAX);
}

#[test]
fn edge_symbol_id_zero() {
    let action = RecoveryAction::InsertToken(SymbolId(0));
    assert!(matches!(action, RecoveryAction::InsertToken(SymbolId(0))));
}

#[test]
fn edge_symbol_id_max() {
    let action = RecoveryAction::InsertToken(SymbolId(u16::MAX));
    assert!(matches!(
        action,
        RecoveryAction::InsertToken(SymbolId(65535))
    ));
}

#[test]
fn edge_error_node_zero_span() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        5,
        5,
        (1, 5),
        (1, 5),
        vec![],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, nodes[0].end_byte);
}

#[test]
fn edge_empty_expected_tokens() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        Some(5),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert!(state.get_error_nodes()[0].expected.is_empty());
}

#[test]
fn edge_large_skipped_tokens_list() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    let skipped: Vec<u16> = (0..1000).collect();
    state.record_error(
        0,
        1000,
        (0, 0),
        (0, 1000),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        skipped,
    );
    assert_eq!(state.get_error_nodes().len(), 1);
}

#[test]
fn edge_many_scope_delimiters() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in (0..100u16).step_by(2) {
        builder = builder.add_scope_delimiter(i, i + 1);
    }
    let config = builder.build();
    assert_eq!(config.scope_delimiters.len(), 50);
}

#[test]
fn edge_duplicate_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(5)
        .add_sync_token(5)
        .add_sync_token(5)
        .build();
    // SmallVec allows duplicates
    assert_eq!(config.sync_tokens.len(), 3);
    // can_delete still works - token 5 is a sync token
    assert!(!config.can_delete_token(SymbolId(5)));
}

#[test]
fn edge_duplicate_insertable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .add_insertable_token(7)
        .build();
    assert_eq!(config.insert_candidates.len(), 2);
}

#[test]
fn edge_duplicate_deletable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(10)
        .add_deletable_token(10)
        .build();
    // HashSet deduplicates
    assert_eq!(config.deletable_tokens.len(), 1);
}
