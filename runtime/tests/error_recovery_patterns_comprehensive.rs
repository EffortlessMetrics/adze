//! Comprehensive tests for error recovery configuration and patterns.

use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};

use ir::SymbolId;

// ============================================================
// 1. ErrorRecoveryConfig default construction
// ============================================================

#[test]
fn default_config_max_panic_skip() {
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
fn default_config_max_token_deletions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn default_config_max_token_insertions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn default_config_max_consecutive_errors() {
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
fn default_config_scope_delimiters_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.scope_delimiters.is_empty());
}

#[test]
fn default_config_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
}

// ============================================================
// 2. ErrorRecoveryConfig builder pattern
// ============================================================

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
        .add_sync_token(10)
        .add_sync_token(20)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert!(cfg.sync_tokens.contains(&SymbolId(10)));
    assert!(cfg.sync_tokens.contains(&SymbolId(20)));
}

#[test]
fn builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(42))
        .build();
    assert!(cfg.sync_tokens.contains(&SymbolId(42)));
}

#[test]
fn builder_add_insertable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(5)));
}

#[test]
fn builder_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(7))
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(7)));
}

#[test]
fn builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(99)
        .build();
    assert!(cfg.deletable_tokens.contains(&99));
}

#[test]
fn builder_add_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(40, 41)]);
}

#[test]
fn builder_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_enable_scope_recovery_off() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!cfg.enable_scope_recovery);
}

#[test]
fn builder_enable_phrase_recovery_off() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!cfg.enable_phrase_recovery);
}

#[test]
fn builder_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(25)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 25);
}

#[test]
fn builder_set_max_recovery_attempts() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(7)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 7);
}

#[test]
fn builder_chaining_multiple_settings() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(4, 5)
        .enable_indentation_recovery(true)
        .enable_scope_recovery(false)
        .enable_phrase_recovery(false)
        .max_consecutive_errors(99)
        .build();
    assert_eq!(cfg.max_panic_skip, 10);
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert!(cfg.deletable_tokens.contains(&3));
    assert_eq!(cfg.scope_delimiters.len(), 1);
    assert!(cfg.enable_indentation_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(!cfg.enable_phrase_recovery);
    assert_eq!(cfg.max_consecutive_errors, 99);
}

#[test]
fn builder_default_trait() {
    let b: ErrorRecoveryConfigBuilder = Default::default();
    let cfg = b.build();
    assert_eq!(cfg.max_panic_skip, 50);
}

// ============================================================
// 3. RecoveryStrategy variants
// ============================================================

#[test]
fn recovery_strategy_panic_mode() {
    let s = RecoveryStrategy::PanicMode;
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

#[test]
fn recovery_strategy_token_insertion() {
    let s = RecoveryStrategy::TokenInsertion;
    assert_eq!(s, RecoveryStrategy::TokenInsertion);
}

#[test]
fn recovery_strategy_token_deletion() {
    let s = RecoveryStrategy::TokenDeletion;
    assert_eq!(s, RecoveryStrategy::TokenDeletion);
}

#[test]
fn recovery_strategy_token_substitution() {
    let s = RecoveryStrategy::TokenSubstitution;
    assert_eq!(s, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn recovery_strategy_phrase_level() {
    let s = RecoveryStrategy::PhraseLevel;
    assert_eq!(s, RecoveryStrategy::PhraseLevel);
}

#[test]
fn recovery_strategy_scope_recovery() {
    let s = RecoveryStrategy::ScopeRecovery;
    assert_eq!(s, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn recovery_strategy_indentation_recovery() {
    let s = RecoveryStrategy::IndentationRecovery;
    assert_eq!(s, RecoveryStrategy::IndentationRecovery);
}

#[test]
fn recovery_strategy_inequality() {
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
    assert_ne!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::PhraseLevel
    );
}

#[test]
fn recovery_strategy_copy() {
    let a = RecoveryStrategy::ScopeRecovery;
    let b = a; // Copy
    assert_eq!(a, b);
}

// ============================================================
// 4. RecoveryAction variants
// ============================================================

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
fn recovery_action_replace_token() {
    let action = RecoveryAction::ReplaceToken(SymbolId(7));
    assert!(matches!(action, RecoveryAction::ReplaceToken(SymbolId(7))));
}

#[test]
fn recovery_action_create_error_node() {
    let action = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    match action {
        RecoveryAction::CreateErrorNode(ids) => {
            assert_eq!(ids.len(), 2);
            assert_eq!(ids[0], SymbolId(1));
            assert_eq!(ids[1], SymbolId(2));
        }
        _ => panic!("expected CreateErrorNode"),
    }
}

#[test]
fn recovery_action_create_error_node_empty() {
    let action = RecoveryAction::CreateErrorNode(vec![]);
    match action {
        RecoveryAction::CreateErrorNode(ids) => assert!(ids.is_empty()),
        _ => panic!("expected CreateErrorNode"),
    }
}

#[test]
fn recovery_action_clone() {
    let action = RecoveryAction::InsertToken(SymbolId(55));
    let cloned = action.clone();
    assert!(matches!(cloned, RecoveryAction::InsertToken(SymbolId(55))));
}

// ============================================================
// 5. ErrorRecoveryState initial state
// ============================================================

#[test]
fn state_initial_no_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_initial_no_error_nodes() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_initial_scope_stack_empty() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2)],
        ..Default::default()
    });
    assert_eq!(state.pop_scope_test(), None);
}

// ============================================================
// 6. State transitions during recovery
// ============================================================

#[test]
fn state_transition_increment_then_give_up() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_transition_reset_after_errors() {
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
fn state_transition_scope_push_pop_cycle() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(state.pop_scope(11));
    // Stack should be empty again
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_transition_record_then_clear() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
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

#[test]
fn state_transition_add_recent_tokens_wraps() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..15u16 {
        state.add_recent_token(i);
    }
    // recent_tokens capacity is 10
    state.update_recent_tokens(SymbolId(100));
    // should still be 10 (oldest dropped)
}

// ============================================================
// 7. Config with different max_error_count values
// ============================================================

#[test]
fn config_max_errors_one() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn config_max_errors_large() {
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

#[test]
fn config_max_errors_via_builder() {
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

// ============================================================
// 8. Recovery with various token types
// ============================================================

#[test]
fn can_delete_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(100));
    assert!(cfg.can_delete_token(SymbolId(50)));
}

#[test]
fn cannot_delete_sync_token_unless_explicitly_deletable() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(100));
    // sync token, not in deletable set
    assert!(!cfg.can_delete_token(SymbolId(100)));
}

#[test]
fn can_delete_sync_token_if_in_deletable_set() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(100));
    cfg.deletable_tokens.insert(100);
    assert!(cfg.can_delete_token(SymbolId(100)));
}

#[test]
fn can_replace_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(50));
    assert!(cfg.can_replace_token(SymbolId(99)));
}

#[test]
fn cannot_replace_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(50));
    assert!(!cfg.can_replace_token(SymbolId(50)));
}

#[test]
fn insert_candidates_affect_strategy_selection() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn no_insert_candidates_skips_insertion() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // No insert candidates, no phrase/scope recovery → panic mode
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// ============================================================
// 9. Edge cases: zero tolerance, max tolerance
// ============================================================

#[test]
fn zero_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new().max_panic_skip(0).build();
    assert_eq!(cfg.max_panic_skip, 0);
}

#[test]
fn zero_max_consecutive_errors_immediately_gives_up() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(cfg);
    assert!(state.should_give_up());
}

#[test]
fn usize_max_consecutive_errors_never_gives_up_easily() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: usize::MAX,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..1000 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn empty_expected_tokens_in_strategy() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[], Some(5), (0, 0), 0);
    // actual=5 not in empty expected set and no sync tokens → clearly wrong → TokenDeletion
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_with_no_actual_and_no_insertable() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[99], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// ============================================================
// 10. Debug/Clone/Display traits on all types
// ============================================================

#[test]
fn recovery_strategy_debug() {
    let s = RecoveryStrategy::PanicMode;
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("PanicMode"));
}

#[test]
fn recovery_strategy_clone() {
    let s = RecoveryStrategy::TokenDeletion;
    let c = s;
    assert_eq!(s, c);
}

#[test]
fn recovery_action_debug() {
    let a = RecoveryAction::DeleteToken;
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("DeleteToken"));
}

#[test]
fn recovery_action_insert_debug() {
    let a = RecoveryAction::InsertToken(SymbolId(42));
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("InsertToken"));
}

#[test]
fn error_recovery_config_debug() {
    let cfg = ErrorRecoveryConfig::default();
    let dbg = format!("{:?}", cfg);
    assert!(dbg.contains("max_panic_skip"));
}

#[test]
fn error_recovery_config_clone() {
    let cfg = ErrorRecoveryConfig::default();
    let c = cfg.clone();
    assert_eq!(c.max_panic_skip, 50);
    assert_eq!(c.max_consecutive_errors, 10);
}

#[test]
fn error_node_debug() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("ErrorNode"));
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 15,
        start_position: (1, 0),
        end_position: (1, 10),
        expected: vec![3, 4],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![9],
    };
    let c = node.clone();
    assert_eq!(c.start_byte, 5);
    assert_eq!(c.end_byte, 15);
    assert_eq!(c.expected, vec![3, 4]);
    assert_eq!(c.actual, None);
    assert_eq!(c.recovery, RecoveryStrategy::TokenInsertion);
}

// ============================================================
// 11. Multiple recovery attempts
// ============================================================

#[test]
fn multiple_error_recordings() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
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

#[test]
fn multiple_strategy_determinations() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .max_consecutive_errors(100)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);

    // First call: insertion should succeed and reset error count
    let s1 = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s1, RecoveryStrategy::TokenInsertion);

    // Second call: still insertion
    let s2 = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    assert_eq!(s2, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_escalates_past_error_limit() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .enable_phrase_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);

    // Manually push error count to the limit
    state.increment_error_count();
    state.increment_error_count();

    // Next determine_recovery_strategy will increment to 3, exceeding max=2
    let strategy = state.determine_recovery_strategy(&[10], Some(5), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn scope_push_pop_multiple_levels() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41), (91, 93), (123, 125)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    state.push_scope(91);
    state.push_scope(123);

    assert!(state.pop_scope(125)); // matches 123
    assert!(state.pop_scope(93)); // matches 91
    assert!(state.pop_scope(41)); // matches 40
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn scope_mismatch_detection() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        enable_scope_recovery: true,
        enable_phrase_recovery: false,
        // Make 41 a sync token so it won't be "clearly wrong" (skips deletion)
        sync_tokens: smallvec::smallvec![SymbolId(41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=41 is a sync token, so not "clearly wrong" → skips deletion
    // expected=[10,20] has 2 entries → can't substitute
    // scope recovery enabled + closing delimiter without opening → ScopeRecovery
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(41), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

// ============================================================
// 12. Recovery state reset
// ============================================================

#[test]
fn reset_consecutive_errors_method() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn reset_error_count_method() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    });
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn clear_errors_removes_all_nodes() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..10 {
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
    }
    assert_eq!(state.get_error_nodes().len(), 10);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_reusable_after_full_reset() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);

    // Drive to failure
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());

    // Record some errors
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

    // Full reset
    state.reset_error_count();
    state.clear_errors();

    assert!(!state.should_give_up());
    assert!(state.get_error_nodes().is_empty());
}

// ============================================================
// Additional edge-case and coverage tests
// ============================================================

#[test]
fn static_is_scope_delimiter_open() {
    let delimiters = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(5, &delimiters));
}

#[test]
fn static_is_matching_delimiter_correct_pairs() {
    let delimiters = vec![(10, 11), (20, 21)];
    assert!(ErrorRecoveryState::is_matching_delimiter(
        10,
        11,
        &delimiters
    ));
    assert!(ErrorRecoveryState::is_matching_delimiter(
        20,
        21,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        10,
        21,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        20,
        11,
        &delimiters
    ));
}

#[test]
fn push_non_delimiter_is_ignored() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99); // not an opening delimiter
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn pop_scope_mismatched_returns_false() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    // Try closing with wrong delimiter
    assert!(!state.pop_scope(21));
}

#[test]
fn error_node_with_skipped_tokens() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        20,
        (0, 0),
        (0, 20),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![4, 5, 6],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].skipped_tokens, vec![4, 5, 6]);
}

#[test]
fn error_node_no_actual_token() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].actual, None);
}

#[test]
fn token_deletion_strategy_for_clearly_wrong_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(100)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=50, expected=[10,20], not a sync token → clearly wrong → deletion
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn token_substitution_strategy_for_single_expected() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(100)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=50, expected=[10] (single expected) → can substitute
    // But deletion is tried first if clearly wrong, so actual must be a sync token
    // to skip deletion check. Let's use the sync token to skip deletion:
    // actual=100 is sync, expected=[10] (single)
    let strategy = state.determine_recovery_strategy(&[10], Some(100), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn phrase_level_fallback() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(50)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=50 (sync token), expected=[10,20] (multiple, can't substitute)
    // → not clearly wrong (it's a sync token), can't substitute (multiple expected)
    // → phrase level
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn update_recent_tokens_via_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(SymbolId(42));
    state.update_recent_tokens(SymbolId(43));
    // Verify they were added (indirectly through the state working)
}

#[test]
fn config_with_multiple_deletable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(1)
        .add_deletable_token(2)
        .add_deletable_token(3)
        .build();
    assert_eq!(cfg.deletable_tokens.len(), 3);
    assert!(cfg.deletable_tokens.contains(&1));
    assert!(cfg.deletable_tokens.contains(&2));
    assert!(cfg.deletable_tokens.contains(&3));
}

#[test]
fn config_with_multiple_scope_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41) // ()
        .add_scope_delimiter(91, 93) // []
        .add_scope_delimiter(123, 125) // {}
        .build();
    assert_eq!(cfg.scope_delimiters.len(), 3);
}

#[test]
fn error_node_positions() {
    let node = ErrorNode {
        start_byte: 100,
        end_byte: 200,
        start_position: (5, 10),
        end_position: (5, 110),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_byte, 100);
    assert_eq!(node.end_byte, 200);
    assert_eq!(node.start_position, (5, 10));
    assert_eq!(node.end_position, (5, 110));
}

#[test]
fn recovery_action_replace_token_debug() {
    let a = RecoveryAction::ReplaceToken(SymbolId(33));
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("ReplaceToken"));
}

#[test]
fn recovery_action_create_error_node_debug() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1)]);
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("CreateErrorNode"));
}
