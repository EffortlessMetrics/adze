//! Comprehensive tests for error_recovery module: strategies, actions, config, state, and edge cases.

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze_ir::SymbolId;

// ---------------------------------------------------------------------------
// RecoveryStrategy enum tests
// ---------------------------------------------------------------------------

#[test]
fn strategy_panic_mode_eq() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_token_insertion_eq() {
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn strategy_token_deletion_eq() {
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn strategy_token_substitution_eq() {
    assert_eq!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn strategy_phrase_level_eq() {
    assert_eq!(RecoveryStrategy::PhraseLevel, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_scope_recovery_eq() {
    assert_eq!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn strategy_variants_not_equal() {
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
    assert_ne!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::PhraseLevel
    );
    assert_ne!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn strategy_debug_format() {
    let dbg = format!("{:?}", RecoveryStrategy::PanicMode);
    assert_eq!(dbg, "PanicMode");
    assert_eq!(
        format!("{:?}", RecoveryStrategy::TokenInsertion),
        "TokenInsertion"
    );
}

#[test]
fn strategy_clone() {
    let s = RecoveryStrategy::TokenDeletion;
    let s2 = s;
    assert_eq!(s, s2);
}

#[test]
fn strategy_copy_semantics() {
    let s = RecoveryStrategy::PhraseLevel;
    let s2 = s;
    // Both usable after copy
    assert_eq!(s, s2);
}

// ---------------------------------------------------------------------------
// RecoveryAction tests (Debug, Clone, PartialEq)
// ---------------------------------------------------------------------------

#[test]
fn action_insert_token_eq() {
    let a = RecoveryAction::InsertToken(SymbolId(5));
    let b = RecoveryAction::InsertToken(SymbolId(5));
    assert_eq!(a, b);
}

#[test]
fn action_insert_token_ne() {
    let a = RecoveryAction::InsertToken(SymbolId(5));
    let b = RecoveryAction::InsertToken(SymbolId(6));
    assert_ne!(a, b);
}

#[test]
fn action_delete_token_eq() {
    assert_eq!(RecoveryAction::DeleteToken, RecoveryAction::DeleteToken);
}

#[test]
fn action_replace_token_eq() {
    let a = RecoveryAction::ReplaceToken(SymbolId(10));
    let b = RecoveryAction::ReplaceToken(SymbolId(10));
    assert_eq!(a, b);
}

#[test]
fn action_create_error_node_eq() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    let b = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    assert_eq!(a, b);
}

#[test]
fn action_different_variants_ne() {
    assert_ne!(
        RecoveryAction::InsertToken(SymbolId(1)),
        RecoveryAction::DeleteToken
    );
    assert_ne!(
        RecoveryAction::DeleteToken,
        RecoveryAction::ReplaceToken(SymbolId(1))
    );
}

#[test]
fn action_debug_format() {
    let dbg = format!("{:?}", RecoveryAction::DeleteToken);
    assert_eq!(dbg, "DeleteToken");
    let dbg2 = format!("{:?}", RecoveryAction::InsertToken(SymbolId(42)));
    assert!(dbg2.contains("InsertToken"));
    assert!(dbg2.contains("42"));
}

#[test]
fn action_clone() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(7)]);
    let b = a.clone();
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// ErrorRecoveryConfig construction, defaults, customization
// ---------------------------------------------------------------------------

#[test]
fn config_default_max_panic_skip() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn config_default_sync_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn config_default_insert_candidates_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn config_default_deletable_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.deletable_tokens.is_empty());
}

#[test]
fn config_default_max_token_deletions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn config_default_max_token_insertions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn config_default_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn config_default_phrase_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn config_default_scope_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn config_default_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
}

#[test]
fn config_can_delete_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(cfg.can_delete_token(SymbolId(99)));
}

#[test]
fn config_cannot_delete_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(!cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn config_can_delete_explicitly_deletable() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    cfg.deletable_tokens.insert(10);
    // Explicitly deletable overrides sync
    assert!(cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn config_can_replace_non_sync() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(20));
    assert!(cfg.can_replace_token(SymbolId(5)));
}

#[test]
fn config_cannot_replace_sync() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(20));
    assert!(!cfg.can_replace_token(SymbolId(20)));
}

// ---------------------------------------------------------------------------
// ErrorRecoveryConfigBuilder tests
// ---------------------------------------------------------------------------

#[test]
fn builder_default_produces_default_config() {
    let built = ErrorRecoveryConfigBuilder::default().build();
    let dflt = ErrorRecoveryConfig::default();
    assert_eq!(built.max_panic_skip, dflt.max_panic_skip);
    assert_eq!(built.max_consecutive_errors, dflt.max_consecutive_errors);
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
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(7)
        .add_sync_token(8)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert!(cfg.sync_tokens.contains(&SymbolId(7)));
}

#[test]
fn builder_add_insertable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(42)
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(42)));
}

#[test]
fn builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(99)
        .build();
    assert!(cfg.deletable_tokens.contains(&99));
}

#[test]
fn builder_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(40, 41)]);
}

#[test]
fn builder_enable_indentation() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
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
fn builder_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 5);
}

#[test]
fn builder_set_max_recovery_attempts_aliases_consecutive() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(3)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 3);
}

// ---------------------------------------------------------------------------
// ErrorRecoveryState creation and tracking
// ---------------------------------------------------------------------------

#[test]
fn state_new_zero_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_increment_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    // Default max is 10, so not giving up yet
    assert!(!state.should_give_up());
}

#[test]
fn state_should_give_up_at_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_recent_tokens_up_to_ten() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..15 {
        state.add_recent_token(i);
    }
    // We can't inspect the internal deque, but we verify it doesn't panic
    // Adding 15 tokens when capacity is 10 should silently drop old ones.
    state.add_recent_token(100);
}

#[test]
fn state_push_pop_scope() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    // Verify via pop_scope_test that the stack has content
    assert!(state.pop_scope(11));
    // After popping, a second pop should fail
    assert!(!state.pop_scope(11));
}

#[test]
fn state_push_non_delimiter_ignored() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99); // not a delimiter
    // Stack should be empty; pop_scope_test returns None
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_mismatch_returns_false() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(!state.pop_scope(21)); // wrong closer for open=10
}

#[test]
fn state_pop_scope_test_raw() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), None);
}

// ---------------------------------------------------------------------------
// Error recording and retrieval
// ---------------------------------------------------------------------------

#[test]
fn state_record_and_get_errors() {
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
    assert_eq!(nodes[0].expected, vec![1]);
    assert_eq!(nodes[0].actual, Some(2));
    assert_eq!(nodes[0].recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn state_record_multiple_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..5 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![1],
            None,
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
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
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_reset_consecutive_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..7 {
        state.increment_error_count();
    }
    state.reset_consecutive_errors();
    // After reset, should not give up
    assert!(!state.should_give_up());
}

// ---------------------------------------------------------------------------
// Strategy selection logic
// ---------------------------------------------------------------------------

#[test]
fn strategy_selection_token_insertion_when_insertable() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_selection_token_deletion_when_wrong() {
    // No insertable tokens, actual token not in expected and not a sync token
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11, 12], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_selection_token_substitution_single_expected() {
    // To reach substitution, the actual token must NOT trigger deletion first.
    // Deletion fires when actual is "clearly wrong" (not in expected AND not sync).
    // So make the actual token a sync token to bypass deletion.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(99)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=99 is sync (not clearly wrong), exactly one expected → substitution
    let strat = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn strategy_selection_panic_mode_after_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Push past the limit via increment
    state.increment_error_count();
    state.increment_error_count();
    let strat = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_selection_scope_recovery_on_mismatch() {
    // To reach scope recovery, deletion and substitution must not trigger first.
    // Make actual=41 a sync token so it's not "clearly wrong" and have multiple expected.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_sync_token(41)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=41 is sync → not clearly wrong; multi-expected → no substitution; scope mismatch
    let strat = state.determine_recovery_strategy(&[10, 11, 12], Some(41), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn strategy_selection_phrase_level_fallback() {
    // No insertable, actual is sync token (not clearly wrong), multi-expected (no sub)
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(99)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_selection_panic_mode_when_all_disabled() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .add_sync_token(99)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=99 is sync token, not clearly wrong; multi-expected, no sub; both disabled
    let strat = state.determine_recovery_strategy(&[10, 11], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_resets_errors_on_insertion() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..5 {
        state.increment_error_count();
    }
    let strat = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
    // Error count should be reset after successful insertion strategy
    assert!(!state.should_give_up());
}

// ---------------------------------------------------------------------------
// Static helpers
// ---------------------------------------------------------------------------

#[test]
fn is_scope_delimiter_open() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delims));
    assert!(!ErrorRecoveryState::is_scope_delimiter(5, &delims));
}

#[test]
fn is_matching_delimiter_correct() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_matching_delimiter(1, 2, &delims));
    assert!(ErrorRecoveryState::is_matching_delimiter(3, 4, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 4, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(3, 2, &delims));
}

// ---------------------------------------------------------------------------
// ErrorNode tests
// ---------------------------------------------------------------------------

#[test]
fn error_node_debug() {
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
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("ErrorNode"));
    assert!(dbg.contains("start_byte: 0"));
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 15,
        start_position: (1, 0),
        end_position: (1, 10),
        expected: vec![10],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 5);
    assert_eq!(cloned.actual, None);
    assert_eq!(cloned.recovery, RecoveryStrategy::PanicMode);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn edge_empty_expected_no_actual() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[], None, (0, 0), 0);
    // No insertable found, no actual → falls through to phrase level
    assert_eq!(strat, RecoveryStrategy::PhraseLevel);
}

#[test]
fn edge_empty_expected_with_actual() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[], Some(50), (0, 0), 0);
    // actual not in expected (empty), not sync → deletion
    assert_eq!(strat, RecoveryStrategy::TokenDeletion);
}

#[test]
fn edge_max_consecutive_errors_zero() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(cfg);
    // 0 >= 0 → should give up immediately
    assert!(state.should_give_up());
}

#[test]
fn edge_update_recent_tokens_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(SymbolId(42));
    // Verify via a second update that it doesn't panic
    state.update_recent_tokens(SymbolId(43));
}

#[test]
fn edge_scope_stack_nested() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    state.push_scope(10);
    state.push_scope(10);
    // Verify 3 items via pop_scope_test
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), Some(10));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn edge_record_error_with_empty_fields() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].expected.is_empty());
    assert!(nodes[0].actual.is_none());
}

#[test]
fn edge_builder_chain_all_options() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_sync_token_sym(SymbolId(2))
        .add_insertable_token(3)
        .add_insertable_token_sym(SymbolId(4))
        .add_deletable_token(5)
        .add_scope_delimiter(6, 7)
        .enable_indentation_recovery(true)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .max_consecutive_errors(20)
        .build();
    assert_eq!(cfg.max_panic_skip, 10);
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert_eq!(cfg.insert_candidates.len(), 2);
    assert!(cfg.deletable_tokens.contains(&5));
    assert_eq!(cfg.scope_delimiters, vec![(6, 7)]);
    assert!(cfg.enable_indentation_recovery);
    assert!(cfg.enable_scope_recovery);
    assert!(!cfg.enable_phrase_recovery);
    assert_eq!(cfg.max_consecutive_errors, 20);
}

#[test]
fn edge_many_errors_then_reset() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn edge_no_scope_delimiters_configured() {
    // Default has no scope delimiters, push_scope should be no-op
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.push_scope(10);
    // Stack should be empty
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn edge_pop_scope_on_empty_stack() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.pop_scope(11));
}
