//! Comprehensive v5 tests for error_recovery module:
//! strategies, actions, config, ordering, clone/debug/eq, and edge cases.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;

// ===========================================================================
// 1. RecoveryStrategy construction (8 tests)
// ===========================================================================

#[test]
fn v5_strategy_panic_mode_variant() {
    let s = RecoveryStrategy::PanicMode;
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

#[test]
fn v5_strategy_token_insertion_variant() {
    let s = RecoveryStrategy::TokenInsertion;
    assert_eq!(s, RecoveryStrategy::TokenInsertion);
}

#[test]
fn v5_strategy_token_deletion_variant() {
    let s = RecoveryStrategy::TokenDeletion;
    assert_eq!(s, RecoveryStrategy::TokenDeletion);
}

#[test]
fn v5_strategy_token_substitution_variant() {
    let s = RecoveryStrategy::TokenSubstitution;
    assert_eq!(s, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn v5_strategy_phrase_level_variant() {
    let s = RecoveryStrategy::PhraseLevel;
    assert_eq!(s, RecoveryStrategy::PhraseLevel);
}

#[test]
fn v5_strategy_scope_recovery_variant() {
    let s = RecoveryStrategy::ScopeRecovery;
    assert_eq!(s, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn v5_strategy_indentation_recovery_variant() {
    let s = RecoveryStrategy::IndentationRecovery;
    assert_eq!(s, RecoveryStrategy::IndentationRecovery);
}

#[test]
fn v5_strategy_copy_semantics() {
    let a = RecoveryStrategy::PanicMode;
    let b = a; // Copy
    assert_eq!(a, b);
}

// ===========================================================================
// 2. RecoveryAction construction and properties (10 tests)
// ===========================================================================

#[test]
fn v5_action_insert_token_basic() {
    let action = RecoveryAction::InsertToken(SymbolId(42));
    assert_eq!(action, RecoveryAction::InsertToken(SymbolId(42)));
}

#[test]
fn v5_action_insert_token_zero() {
    let action = RecoveryAction::InsertToken(SymbolId(0));
    assert_eq!(action, RecoveryAction::InsertToken(SymbolId(0)));
}

#[test]
fn v5_action_insert_token_max() {
    let action = RecoveryAction::InsertToken(SymbolId(u16::MAX));
    assert_eq!(action, RecoveryAction::InsertToken(SymbolId(u16::MAX)));
}

#[test]
fn v5_action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    assert_eq!(action, RecoveryAction::DeleteToken);
}

#[test]
fn v5_action_replace_token() {
    let action = RecoveryAction::ReplaceToken(SymbolId(7));
    assert_eq!(action, RecoveryAction::ReplaceToken(SymbolId(7)));
}

#[test]
fn v5_action_create_error_node_empty() {
    let action = RecoveryAction::CreateErrorNode(vec![]);
    assert_eq!(action, RecoveryAction::CreateErrorNode(vec![]));
}

#[test]
fn v5_action_create_error_node_single() {
    let action = RecoveryAction::CreateErrorNode(vec![SymbolId(1)]);
    assert_eq!(action, RecoveryAction::CreateErrorNode(vec![SymbolId(1)]));
}

#[test]
fn v5_action_create_error_node_multiple() {
    let syms = vec![SymbolId(1), SymbolId(2), SymbolId(3)];
    let action = RecoveryAction::CreateErrorNode(syms.clone());
    assert_eq!(action, RecoveryAction::CreateErrorNode(syms));
}

#[test]
fn v5_action_insert_ne_delete() {
    let insert = RecoveryAction::InsertToken(SymbolId(1));
    let delete = RecoveryAction::DeleteToken;
    assert_ne!(insert, delete);
}

#[test]
fn v5_action_insert_different_symbols_ne() {
    let a = RecoveryAction::InsertToken(SymbolId(1));
    let b = RecoveryAction::InsertToken(SymbolId(2));
    assert_ne!(a, b);
}

// ===========================================================================
// 3. ErrorRecoveryConfig defaults and customization (8 tests)
// ===========================================================================

#[test]
fn v5_config_default_max_panic_skip() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn v5_config_default_max_token_deletions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn v5_config_default_max_token_insertions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn v5_config_default_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn v5_config_default_phrase_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn v5_config_default_scope_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn v5_config_default_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
}

#[test]
fn v5_config_default_empty_collections() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
    assert!(cfg.insert_candidates.is_empty());
    assert!(cfg.deletable_tokens.is_empty());
    assert!(cfg.scope_delimiters.is_empty());
}

// ===========================================================================
// 4. RecoveryAction ordering/comparison (8 tests)
// ===========================================================================

#[test]
fn v5_action_eq_reflexive_insert() {
    let a = RecoveryAction::InsertToken(SymbolId(5));
    assert_eq!(a, a.clone());
}

#[test]
fn v5_action_eq_reflexive_delete() {
    let a = RecoveryAction::DeleteToken;
    assert_eq!(a, a.clone());
}

#[test]
fn v5_action_eq_reflexive_replace() {
    let a = RecoveryAction::ReplaceToken(SymbolId(10));
    assert_eq!(a, a.clone());
}

#[test]
fn v5_action_eq_reflexive_error_node() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    assert_eq!(a, a.clone());
}

#[test]
fn v5_action_ne_insert_vs_replace_same_sym() {
    let insert = RecoveryAction::InsertToken(SymbolId(5));
    let replace = RecoveryAction::ReplaceToken(SymbolId(5));
    assert_ne!(insert, replace);
}

#[test]
fn v5_action_ne_delete_vs_error_node() {
    let delete = RecoveryAction::DeleteToken;
    let error = RecoveryAction::CreateErrorNode(vec![]);
    assert_ne!(delete, error);
}

#[test]
fn v5_action_ne_error_nodes_different_contents() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1)]);
    let b = RecoveryAction::CreateErrorNode(vec![SymbolId(2)]);
    assert_ne!(a, b);
}

#[test]
fn v5_action_ne_error_nodes_different_lengths() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1)]);
    let b = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    assert_ne!(a, b);
}

// ===========================================================================
// 5. Strategy enumeration coverage (5 tests)
// ===========================================================================

#[test]
fn v5_strategy_all_variants_distinct() {
    let variants: Vec<RecoveryStrategy> = vec![
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(
                variants[i], variants[j],
                "variants {i} and {j} should differ"
            );
        }
    }
}

#[test]
fn v5_strategy_count_is_seven() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    assert_eq!(variants.len(), 7);
}

#[test]
fn v5_strategy_clone_preserves_value() {
    let original = RecoveryStrategy::ScopeRecovery;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn v5_strategy_ne_cross_variant() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion);
    assert_ne!(
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery
    );
    assert_ne!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn v5_strategy_used_in_error_node() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2],
        actual: Some(3),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![3],
    };
    assert_eq!(node.recovery, RecoveryStrategy::TokenDeletion);
}

// ===========================================================================
// 6. Config with various strategies (8 tests)
// ===========================================================================

#[test]
fn v5_builder_default_matches_config_default() {
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
}

#[test]
fn v5_builder_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn v5_builder_add_sync_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(10)
        .add_sync_token(20)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert!(cfg.sync_tokens.contains(&SymbolId(10)));
    assert!(cfg.sync_tokens.contains(&SymbolId(20)));
}

#[test]
fn v5_builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(99))
        .build();
    assert!(cfg.sync_tokens.contains(&SymbolId(99)));
}

#[test]
fn v5_builder_add_insertable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .add_insertable_token_sym(SymbolId(6))
        .build();
    assert_eq!(cfg.insert_candidates.len(), 2);
    assert!(cfg.insert_candidates.contains(&SymbolId(5)));
    assert!(cfg.insert_candidates.contains(&SymbolId(6)));
}

#[test]
fn v5_builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(15)
        .build();
    assert!(cfg.deletable_tokens.contains(&15));
}

#[test]
fn v5_builder_scope_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41) // ( )
        .add_scope_delimiter(91, 93) // [ ]
        .build();
    assert_eq!(cfg.scope_delimiters.len(), 2);
    assert_eq!(cfg.scope_delimiters[0], (40, 41));
    assert_eq!(cfg.scope_delimiters[1], (91, 93));
}

#[test]
fn v5_builder_toggle_recovery_modes() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
}

// ===========================================================================
// 7. Clone/Debug/PartialEq (5 tests)
// ===========================================================================

#[test]
fn v5_strategy_debug_format() {
    let s = RecoveryStrategy::PanicMode;
    let dbg = format!("{s:?}");
    assert!(dbg.contains("PanicMode"), "debug was: {dbg}");
}

#[test]
fn v5_action_debug_format_insert() {
    let a = RecoveryAction::InsertToken(SymbolId(42));
    let dbg = format!("{a:?}");
    assert!(dbg.contains("InsertToken"), "debug was: {dbg}");
}

#[test]
fn v5_action_debug_format_delete() {
    let a = RecoveryAction::DeleteToken;
    let dbg = format!("{a:?}");
    assert!(dbg.contains("DeleteToken"), "debug was: {dbg}");
}

#[test]
fn v5_action_clone_deep_equality() {
    let original = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2), SymbolId(3)]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn v5_config_clone_independence() {
    let mut cfg = ErrorRecoveryConfig {
        max_panic_skip: 100,
        ..Default::default()
    };
    cfg.sync_tokens.push(SymbolId(5));

    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 100);
    assert!(cloned.sync_tokens.contains(&SymbolId(5)));

    // Mutate original; clone should be independent
    cfg.max_panic_skip = 999;
    assert_eq!(cloned.max_panic_skip, 100);
}

// ===========================================================================
// 8. Edge cases (3+ tests)
// ===========================================================================

#[test]
fn v5_config_can_delete_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    // Token 20 is not a sync token, so it can be deleted
    assert!(cfg.can_delete_token(SymbolId(20)));
}

#[test]
fn v5_config_can_delete_explicitly_deletable_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    cfg.deletable_tokens.insert(10);
    // Explicitly deletable even though it's a sync token
    assert!(cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn v5_config_cannot_replace_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(!cfg.can_replace_token(SymbolId(10)));
}

#[test]
fn v5_config_can_replace_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(cfg.can_replace_token(SymbolId(20)));
}

// ===========================================================================
// Additional: ErrorRecoveryState tests
// ===========================================================================

#[test]
fn v5_state_new_zero_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn v5_state_increment_and_give_up() {
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
fn v5_state_reset_error_count() {
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
fn v5_state_scope_push_pop_round_trip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    assert!(state.pop_scope(2));
}

#[test]
fn v5_state_scope_mismatch_pop_fails() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .add_scope_delimiter(3, 4)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    // Try to close with wrong delimiter
    assert!(!state.pop_scope(4));
}

#[test]
fn v5_state_record_and_retrieve_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());

    state.record_error(
        10,
        20,
        (1, 0),
        (1, 10),
        vec![5, 6],
        Some(7),
        RecoveryStrategy::PhraseLevel,
        vec![7],
    );
    state.record_error(
        30,
        40,
        (2, 0),
        (2, 10),
        vec![8],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );

    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].start_byte, 10);
    assert_eq!(nodes[0].end_byte, 20);
    assert_eq!(nodes[0].actual, Some(7));
    assert_eq!(nodes[1].start_byte, 30);
    assert_eq!(nodes[1].actual, None);
    assert_eq!(nodes[1].recovery, RecoveryStrategy::TokenInsertion);
}

#[test]
fn v5_state_recent_tokens_tracking() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.add_recent_token(1);
    state.add_recent_token(2);
    state.add_recent_token(3);
    state.update_recent_tokens(SymbolId(4));
    // No panic; just confirm it doesn't crash
}

#[test]
fn v5_state_determine_strategy_insertion() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn v5_state_determine_strategy_panic_after_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Burn through error budget
    state.increment_error_count();
    state.increment_error_count();
    let strategy = state.determine_recovery_strategy(&[1], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn v5_state_is_scope_delimiter_static() {
    let delimiters = vec![(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(91, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(42, &delimiters));
}

#[test]
fn v5_state_is_matching_delimiter_static() {
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
fn v5_builder_set_max_recovery_attempts() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(5)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 5);
}

#[test]
fn v5_builder_default_trait() {
    let builder: ErrorRecoveryConfigBuilder = Default::default();
    let cfg = builder.build();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn v5_error_node_fields() {
    let node = ErrorNode {
        start_byte: 100,
        end_byte: 200,
        start_position: (5, 10),
        end_position: (5, 20),
        expected: vec![1, 2, 3],
        actual: None,
        recovery: RecoveryStrategy::ScopeRecovery,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_byte, 100);
    assert_eq!(node.end_byte, 200);
    assert_eq!(node.expected, vec![1, 2, 3]);
    assert!(node.actual.is_none());
    assert!(node.skipped_tokens.is_empty());
}

#[test]
fn v5_error_node_debug_format() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![],
        actual: Some(99),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let dbg = format!("{node:?}");
    assert!(dbg.contains("ErrorNode"), "debug was: {dbg}");
}

#[test]
fn v5_state_pop_scope_test_helper() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    assert_eq!(state.pop_scope_test(), None);
    state.push_scope(1);
    assert_eq!(state.pop_scope_test(), Some(1));
    assert_eq!(state.pop_scope_test(), None);
}
