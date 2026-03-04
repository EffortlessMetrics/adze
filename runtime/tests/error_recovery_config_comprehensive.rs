// Comprehensive tests for ErrorRecoveryConfig and RecoveryStrategy
use adze::error_recovery::*;
use adze_ir::SymbolId;

// =====================================================================
// 1. ErrorRecoveryConfig default construction
// =====================================================================

#[test]
fn config_default_max_panic_skip() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn config_default_sync_tokens_empty() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.sync_tokens.is_empty());
}

#[test]
fn config_default_insert_candidates_empty() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.insert_candidates.is_empty());
}

#[test]
fn config_default_deletable_tokens_empty() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.deletable_tokens.is_empty());
}

#[test]
fn config_default_max_token_deletions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_deletions, 3);
}

#[test]
fn config_default_max_token_insertions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_insertions, 2);
}

#[test]
fn config_default_max_consecutive_errors() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn config_default_phrase_recovery_enabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.enable_phrase_recovery);
}

#[test]
fn config_default_scope_recovery_enabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.enable_scope_recovery);
}

#[test]
fn config_default_scope_delimiters_empty() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.scope_delimiters.is_empty());
}

#[test]
fn config_default_indentation_recovery_disabled() {
    let config = ErrorRecoveryConfig::default();
    assert!(!config.enable_indentation_recovery);
}

// =====================================================================
// 2. ErrorRecoveryConfig clone
// =====================================================================

#[test]
fn config_clone_preserves_max_panic_skip() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 200,
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.max_panic_skip, 200);
}

#[test]
fn config_clone_preserves_sync_tokens() {
    let mut config = ErrorRecoveryConfig::default();
    config.sync_tokens.push(SymbolId(42));
    config.sync_tokens.push(SymbolId(99));
    let cloned = config.clone();
    assert_eq!(cloned.sync_tokens.len(), 2);
    assert_eq!(cloned.sync_tokens[0], SymbolId(42));
    assert_eq!(cloned.sync_tokens[1], SymbolId(99));
}

#[test]
fn config_clone_preserves_scope_delimiters() {
    let mut config = ErrorRecoveryConfig::default();
    config.scope_delimiters.push((10, 11));
    let cloned = config.clone();
    assert_eq!(cloned.scope_delimiters, vec![(10, 11)]);
}

#[test]
fn config_clone_is_independent() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 100,
        ..Default::default()
    };
    let mut cloned = config.clone();
    cloned.max_panic_skip = 999;
    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(cloned.max_panic_skip, 999);
}

// =====================================================================
// 3. RecoveryStrategy variant construction
// =====================================================================

#[test]
fn strategy_panic_mode_construction() {
    let _s = RecoveryStrategy::PanicMode;
}

#[test]
fn strategy_token_insertion_construction() {
    let _s = RecoveryStrategy::TokenInsertion;
}

#[test]
fn strategy_token_deletion_construction() {
    let _s = RecoveryStrategy::TokenDeletion;
}

#[test]
fn strategy_token_substitution_construction() {
    let _s = RecoveryStrategy::TokenSubstitution;
}

#[test]
fn strategy_phrase_level_construction() {
    let _s = RecoveryStrategy::PhraseLevel;
}

#[test]
fn strategy_scope_recovery_construction() {
    let _s = RecoveryStrategy::ScopeRecovery;
}

#[test]
fn strategy_indentation_recovery_construction() {
    let _s = RecoveryStrategy::IndentationRecovery;
}

// =====================================================================
// 4. RecoveryStrategy clone
// =====================================================================

#[test]
fn strategy_clone_panic_mode() {
    let s = RecoveryStrategy::PanicMode;
    let c = s;
    assert_eq!(s, c);
}

#[test]
fn strategy_clone_token_insertion() {
    let s = RecoveryStrategy::TokenInsertion;
    let c = s;
    assert_eq!(s, c);
}

#[test]
fn strategy_clone_token_deletion() {
    let s = RecoveryStrategy::TokenDeletion;
    let c = s;
    assert_eq!(s, c);
}

#[test]
fn strategy_clone_phrase_level() {
    let s = RecoveryStrategy::PhraseLevel;
    let c = s;
    assert_eq!(s, c);
}

#[test]
fn strategy_clone_scope_recovery() {
    let s = RecoveryStrategy::ScopeRecovery;
    let c = s;
    assert_eq!(s, c);
}

// =====================================================================
// 5. RecoveryStrategy debug format
// =====================================================================

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

// =====================================================================
// 6. RecoveryStrategy equality
// =====================================================================

#[test]
fn strategy_eq_same_variant() {
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
}

#[test]
fn strategy_ne_different_variants() {
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
    assert_ne!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::PhraseLevel
    );
    assert_ne!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn strategy_ne_all_pairs_distinct() {
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

// =====================================================================
// 7. ErrorRecoveryState default
// =====================================================================

#[test]
fn state_new_not_giving_up() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(!state.should_give_up());
}

#[test]
fn state_new_no_error_nodes() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_increment_and_check() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_clears_error_count() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_reset_consecutive_errors_method() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.reset_consecutive_errors();
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_clear_errors_removes_recorded() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        2,
        3,
        (0, 2),
        (0, 3),
        vec![2],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 2);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// =====================================================================
// 8. Multiple strategies in collections
// =====================================================================

#[test]
fn strategies_in_vec() {
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
fn strategies_in_vec_contains() {
    let strategies = [RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion];
    assert!(strategies.contains(&RecoveryStrategy::PanicMode));
    assert!(strategies.contains(&RecoveryStrategy::TokenDeletion));
    assert!(!strategies.contains(&RecoveryStrategy::PhraseLevel));
}

#[test]
fn strategies_in_vec_filter() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::PanicMode,
    ];
    let panic_count = strategies
        .iter()
        .filter(|s| **s == RecoveryStrategy::PanicMode)
        .count();
    assert_eq!(panic_count, 2);
}

#[test]
fn strategies_dedup() {
    let mut strategies = vec![
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
    ];
    strategies.dedup();
    assert_eq!(strategies.len(), 3);
}

// =====================================================================
// 9. Strategy pattern matching
// =====================================================================

#[test]
fn pattern_match_all_variants() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for v in &variants {
        let label = match v {
            RecoveryStrategy::PanicMode => "panic",
            RecoveryStrategy::TokenInsertion => "insert",
            RecoveryStrategy::TokenDeletion => "delete",
            RecoveryStrategy::TokenSubstitution => "substitute",
            RecoveryStrategy::PhraseLevel => "phrase",
            RecoveryStrategy::ScopeRecovery => "scope",
            RecoveryStrategy::IndentationRecovery => "indent",
        };
        assert!(!label.is_empty());
    }
}

#[test]
fn pattern_match_with_matches_macro() {
    assert!(matches!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::PanicMode
    ));
    assert!(matches!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    ));
    assert!(!matches!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenDeletion
    ));
}

#[test]
fn pattern_match_or_patterns() {
    let s = RecoveryStrategy::TokenDeletion;
    let is_token_level = matches!(
        s,
        RecoveryStrategy::TokenInsertion
            | RecoveryStrategy::TokenDeletion
            | RecoveryStrategy::TokenSubstitution
    );
    assert!(is_token_level);

    let s2 = RecoveryStrategy::PhraseLevel;
    let is_token_level2 = matches!(
        s2,
        RecoveryStrategy::TokenInsertion
            | RecoveryStrategy::TokenDeletion
            | RecoveryStrategy::TokenSubstitution
    );
    assert!(!is_token_level2);
}

// =====================================================================
// 10. Config with different settings
// =====================================================================

#[test]
fn config_custom_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(config.max_panic_skip, 200);
}

#[test]
fn config_disable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!config.enable_phrase_recovery);
}

#[test]
fn config_disable_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!config.enable_scope_recovery);
}

#[test]
fn config_enable_indentation_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(config.enable_indentation_recovery);
}

#[test]
fn config_multiple_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token(3)
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
}

#[test]
fn config_multiple_insertable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .add_insertable_token(20)
        .build();
    assert_eq!(config.insert_candidates.len(), 2);
}

#[test]
fn config_multiple_deletable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .add_deletable_token(6)
        .add_deletable_token(7)
        .build();
    assert!(config.deletable_tokens.contains(&5));
    assert!(config.deletable_tokens.contains(&6));
    assert!(config.deletable_tokens.contains(&7));
    assert_eq!(config.deletable_tokens.len(), 3);
}

#[test]
fn config_multiple_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .add_scope_delimiter(123, 125)
        .build();
    assert_eq!(config.scope_delimiters.len(), 3);
}

#[test]
fn config_builder_chaining_all_options() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_sync_token_sym(SymbolId(2))
        .add_insertable_token(3)
        .add_insertable_token_sym(SymbolId(4))
        .add_deletable_token(5)
        .add_scope_delimiter(6, 7)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .enable_indentation_recovery(true)
        .max_consecutive_errors(50)
        .set_max_recovery_attempts(50)
        .build();
    assert_eq!(config.max_panic_skip, 10);
    assert_eq!(config.sync_tokens.len(), 2);
    assert_eq!(config.insert_candidates.len(), 2);
    assert!(config.deletable_tokens.contains(&5));
    assert_eq!(config.scope_delimiters, vec![(6, 7)]);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_phrase_recovery);
    assert!(config.enable_indentation_recovery);
    assert_eq!(config.max_consecutive_errors, 50);
}

#[test]
fn config_set_max_recovery_attempts_aliases_max_consecutive() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(77)
        .build();
    assert_eq!(config.max_consecutive_errors, 77);
}

// =====================================================================
// Config methods: can_delete_token and can_replace_token
// =====================================================================

#[test]
fn can_delete_non_sync_token() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.can_delete_token(SymbolId(42)));
}

#[test]
fn cannot_delete_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(99))
        .build();
    assert!(!config.can_delete_token(SymbolId(99)));
}

#[test]
fn can_delete_sync_token_if_in_deletable_set() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(10))
        .add_deletable_token(10)
        .build();
    assert!(config.can_delete_token(SymbolId(10)));
}

#[test]
fn can_replace_non_sync_token() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.can_replace_token(SymbolId(42)));
}

#[test]
fn cannot_replace_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(50))
        .build();
    assert!(!config.can_replace_token(SymbolId(50)));
}

// =====================================================================
// RecoveryAction variants
// =====================================================================

#[test]
fn recovery_action_insert_token() {
    let action = RecoveryAction::InsertToken(SymbolId(42));
    assert!(matches!(action, RecoveryAction::InsertToken(id) if id == SymbolId(42)));
}

#[test]
fn recovery_action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    assert!(matches!(action, RecoveryAction::DeleteToken));
}

#[test]
fn recovery_action_replace_token() {
    let action = RecoveryAction::ReplaceToken(SymbolId(7));
    assert!(matches!(action, RecoveryAction::ReplaceToken(id) if id == SymbolId(7)));
}

#[test]
fn recovery_action_create_error_node() {
    let action = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    match action {
        RecoveryAction::CreateErrorNode(ids) => assert_eq!(ids.len(), 2),
        _ => panic!("expected CreateErrorNode"),
    }
}

#[test]
fn recovery_action_debug_format() {
    let action = RecoveryAction::DeleteToken;
    let dbg = format!("{:?}", action);
    assert!(dbg.contains("DeleteToken"));
}

#[test]
fn recovery_action_clone() {
    let action = RecoveryAction::InsertToken(SymbolId(5));
    let cloned = action.clone();
    assert!(matches!(cloned, RecoveryAction::InsertToken(id) if id == SymbolId(5)));
}

// =====================================================================
// ErrorNode tests
// =====================================================================

#[test]
fn error_node_fields_accessible() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        10,
        20,
        (1, 5),
        (1, 15),
        vec![1, 2, 3],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![50, 51],
    );
    let nodes = state.get_error_nodes();
    let node = &nodes[0];
    assert_eq!(node.start_byte, 10);
    assert_eq!(node.end_byte, 20);
    assert_eq!(node.expected, vec![1, 2, 3]);
    assert_eq!(node.actual, Some(99));
    assert_eq!(node.recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn error_node_none_actual() {
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
    assert_eq!(state.get_error_nodes()[0].actual, None);
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![3],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 0);
    assert_eq!(cloned.end_byte, 5);
    assert_eq!(cloned.recovery, RecoveryStrategy::PhraseLevel);
}

#[test]
fn error_node_debug() {
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
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("ErrorNode"));
}

// =====================================================================
// Scope tracking
// =====================================================================

#[test]
fn scope_push_non_delimiter_ignored() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(99);
    // 99 is not an opening delimiter, should not be pushed
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn scope_pop_mismatched_close() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    // Try closing with wrong delimiter
    assert!(!state.pop_scope(93));
}

#[test]
fn scope_nested_deeply() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..10 {
        state.push_scope(40);
    }
    for _ in 0..10 {
        assert!(state.pop_scope(41));
    }
    assert!(!state.pop_scope(41));
}

// =====================================================================
// Static utility functions
// =====================================================================

#[test]
fn is_scope_delimiter_opening() {
    let delimiters = vec![(40, 41)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
}

#[test]
fn is_scope_delimiter_closing() {
    let delimiters = vec![(40, 41)];
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
}

#[test]
fn is_scope_delimiter_not_found() {
    let delimiters = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delimiters));
}

#[test]
fn is_matching_delimiter_correct() {
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
}

#[test]
fn is_matching_delimiter_swapped() {
    let delimiters = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        41,
        40,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_cross_pair() {
    let delimiters = vec![(40, 41), (91, 93)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        40,
        93,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_empty_delimiters() {
    let delimiters: Vec<(u16, u16)> = vec![];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        40,
        41,
        &delimiters
    ));
}

// =====================================================================
// determine_recovery_strategy
// =====================================================================

#[test]
fn determine_strategy_insertion_when_candidate_available() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn determine_strategy_panic_after_too_many_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // First call consumes the budget
    let _ = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    // Second call exceeds it
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn determine_strategy_deletion_for_wrong_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], Some(50), (0, 0), 0);
    // Wrong token that is not a sync token triggers deletion first
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn determine_strategy_phrase_level_fallback() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // Multiple expected tokens with a non-sync wrong actual triggers deletion
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

// =====================================================================
// Recent tokens
// =====================================================================

#[test]
fn add_recent_token_overflow() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..20 {
        state.add_recent_token(i);
    }
    // Internally capped at 10; just ensure no panic
}

#[test]
fn update_recent_tokens_via_symbol_id() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.update_recent_tokens(SymbolId(42));
    // Should not panic
}

// =====================================================================
// Edge cases
// =====================================================================

#[test]
fn zero_max_consecutive_errors_gives_up_immediately() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    let state = ErrorRecoveryState::new(config);
    assert!(state.should_give_up());
}

#[test]
fn large_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(usize::MAX)
        .build();
    assert_eq!(config.max_panic_skip, usize::MAX);
}

#[test]
fn many_sync_tokens_in_config() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in 0..100 {
        builder = builder.add_sync_token(i);
    }
    let config = builder.build();
    assert_eq!(config.sync_tokens.len(), 100);
}

#[test]
fn config_debug_format() {
    let config = ErrorRecoveryConfig::default();
    let dbg = format!("{:?}", config);
    assert!(dbg.contains("ErrorRecoveryConfig"));
}

#[test]
fn builder_default_trait() {
    let builder: ErrorRecoveryConfigBuilder = Default::default();
    let config = builder.build();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn record_error_with_every_strategy() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for (i, strat) in strategies.iter().enumerate() {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![i as u16],
            Some(i as u16 + 100),
            *strat,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 7);
    for (i, node) in nodes.iter().enumerate() {
        assert_eq!(node.recovery, strategies[i]);
    }
}
