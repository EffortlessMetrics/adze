// Property tests for ErrorRecoveryConfigBuilder and ErrorRecoveryState
use adze::error_recovery::*;
use adze_ir::SymbolId;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Builder defaults
// ---------------------------------------------------------------------------

#[test]
fn builder_default_matches_config_default() {
    let builder_config = ErrorRecoveryConfigBuilder::new().build();
    let default_config = ErrorRecoveryConfig::default();
    assert_eq!(builder_config.max_panic_skip, default_config.max_panic_skip);
    assert_eq!(
        builder_config.max_consecutive_errors,
        default_config.max_consecutive_errors
    );
}

#[test]
fn builder_default_empty_collections() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.sync_tokens.is_empty());
    assert!(config.insert_candidates.is_empty());
    assert!(config.deletable_tokens.is_empty());
    assert!(config.scope_delimiters.is_empty());
}

#[test]
fn default_config_values() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_indentation_recovery);
}

// ---------------------------------------------------------------------------
// Builder fluent chaining
// ---------------------------------------------------------------------------

#[test]
fn builder_chain_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token(3)
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
    assert!(config.sync_tokens.contains(&SymbolId(1)));
    assert!(config.sync_tokens.contains(&SymbolId(2)));
    assert!(config.sync_tokens.contains(&SymbolId(3)));
}

#[test]
fn builder_chain_insertable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .add_insertable_token(20)
        .build();
    assert_eq!(config.insert_candidates.len(), 2);
    assert!(config.insert_candidates.contains(&SymbolId(10)));
}

#[test]
fn builder_chain_deletable_tokens() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .build();
    assert!(config.deletable_tokens.contains(&5));
}

#[test]
fn builder_add_sync_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(42))
        .build();
    assert!(config.sync_tokens.contains(&SymbolId(42)));
}

#[test]
fn builder_add_insertable_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(99))
        .build();
    assert!(config.insert_candidates.contains(&SymbolId(99)));
}

#[test]
fn builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new().max_panic_skip(50).build();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn builder_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(10)
        .build();
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn builder_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .build();
    assert_eq!(config.scope_delimiters.len(), 2);
    assert!(config.scope_delimiters.contains(&(10, 11)));
    assert!(config.scope_delimiters.contains(&(20, 21)));
}

#[test]
fn builder_enable_indentation() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(config.enable_indentation_recovery);
}

#[test]
fn builder_disable_indentation() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(false)
        .build();
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn builder_enable_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(true)
        .build();
    assert!(config.enable_scope_recovery);
}

#[test]
fn builder_disable_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!config.enable_scope_recovery);
}

#[test]
fn builder_enable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .build();
    assert!(config.enable_phrase_recovery);
}

#[test]
fn builder_disable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!config.enable_phrase_recovery);
}

// ---------------------------------------------------------------------------
// ErrorRecoveryConfig method tests
// ---------------------------------------------------------------------------

#[test]
fn can_delete_token_explicit() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .build();
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn can_delete_token_non_sync() {
    // Non-sync tokens are also considered deletable
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(1).build();
    // Token 99 is not a sync token, so it's deletable
    assert!(config.can_delete_token(SymbolId(99)));
}

#[test]
fn can_replace_token_non_sync() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(1).build();
    // Non-sync tokens can be replaced
    assert!(config.can_replace_token(SymbolId(99)));
    // Sync tokens cannot be replaced
    assert!(!config.can_replace_token(SymbolId(1)));
}

// ---------------------------------------------------------------------------
// ErrorRecoveryState tests
// ---------------------------------------------------------------------------

#[test]
fn state_new_has_zero_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let state = ErrorRecoveryState::new(config);
    assert!(state.get_error_nodes().is_empty());
    assert!(!state.should_give_up());
}

#[test]
fn state_record_error() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
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
fn state_record_multiple_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        10,
        15,
        (1, 0),
        (1, 5),
        vec![],
        None,
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 2);
}

#[test]
fn state_clear_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_add_recent_token() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..15 {
        state.add_recent_token(i);
    }
    // Should not panic; recent tokens are bounded to 10
}

#[test]
fn state_push_pop_scope() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    assert!(state.pop_scope(11)); // close with matching delimiter
}

#[test]
fn state_pop_scope_empty_returns_false() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    assert!(!state.pop_scope(11));
}

#[test]
fn state_increment_and_should_give_up() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_error_count() {
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
fn state_reset_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn state_update_recent_tokens() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.update_recent_tokens(SymbolId(5));
    state.update_recent_tokens(SymbolId(10));
}

#[test]
fn state_pop_scope_test() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    let popped = state.pop_scope_test();
    assert_eq!(popped, Some(10));
    assert_eq!(state.pop_scope_test(), None);
}

// ---------------------------------------------------------------------------
// ErrorNode tests
// ---------------------------------------------------------------------------

#[test]
fn error_node_debug() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![1, 2],
        actual: Some(3),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let debug = format!("{:?}", node);
    assert!(debug.contains("ErrorNode"));
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 20,
        start_position: (1, 0),
        end_position: (1, 10),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![5, 6],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 10);
    assert_eq!(cloned.end_byte, 20);
    assert_eq!(cloned.skipped_tokens.len(), 2);
}

// ---------------------------------------------------------------------------
// RecoveryStrategy / RecoveryAction enum tests
// ---------------------------------------------------------------------------

#[test]
fn recovery_strategy_variants_debug() {
    let strategies = vec![
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
    ];
    for s in &strategies {
        let debug = format!("{:?}", s);
        assert!(!debug.is_empty());
    }
}

#[test]
fn recovery_action_insert_token() {
    let action = RecoveryAction::InsertToken(SymbolId(5));
    let debug = format!("{:?}", action);
    assert!(debug.contains("InsertToken"));
}

#[test]
fn recovery_action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    let debug = format!("{:?}", action);
    assert!(debug.contains("DeleteToken"));
}

#[test]
fn recovery_action_replace_token() {
    let action = RecoveryAction::ReplaceToken(SymbolId(10));
    let debug = format!("{:?}", action);
    assert!(debug.contains("ReplaceToken"));
}

#[test]
fn recovery_action_create_error_node() {
    let action = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    let debug = format!("{:?}", action);
    assert!(debug.contains("CreateErrorNode"));
}

// ---------------------------------------------------------------------------
// Static delimiter helpers
// ---------------------------------------------------------------------------

#[test]
fn is_scope_delimiter_positive() {
    let delimiters = vec![(10u16, 11u16)];
    assert!(ErrorRecoveryState::is_scope_delimiter(10, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(11, &delimiters));
}

#[test]
fn is_scope_delimiter_negative() {
    let delimiters = vec![(10u16, 11u16)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(12, &delimiters));
}

#[test]
fn is_matching_delimiter_positive() {
    let delimiters = vec![(10u16, 11u16)];
    assert!(ErrorRecoveryState::is_matching_delimiter(
        10,
        11,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_reversed_false() {
    let delimiters = vec![(10u16, 11u16)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        11,
        10,
        &delimiters
    ));
}

#[test]
fn is_matching_delimiter_wrong_pair() {
    let delimiters = vec![(10u16, 11u16), (20u16, 21u16)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        10,
        21,
        &delimiters
    ));
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn builder_max_panic_skip_preserved(max in 1usize..1000) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(max)
            .build();
        prop_assert_eq!(config.max_panic_skip, max);
    }

    #[test]
    fn builder_max_consecutive_errors_preserved(max in 1usize..100) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(max)
            .build();
        prop_assert_eq!(config.max_consecutive_errors, max);
    }

    #[test]
    fn builder_sync_token_count(tokens in proptest::collection::vec(0u16..1000, 0..20)) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for t in &tokens {
            builder = builder.add_sync_token(*t);
        }
        let config = builder.build();
        // SmallVec allows duplicates
        prop_assert_eq!(config.sync_tokens.len(), tokens.len());
    }

    #[test]
    fn builder_insertable_token_roundtrip(id in 0u16..1000) {
        let config = ErrorRecoveryConfigBuilder::new()
            .add_insertable_token(id)
            .build();
        prop_assert!(config.insert_candidates.contains(&SymbolId(id)));
    }

    #[test]
    fn builder_deletable_token_roundtrip(id in 0u16..1000) {
        let config = ErrorRecoveryConfigBuilder::new()
            .add_deletable_token(id)
            .build();
        prop_assert!(config.deletable_tokens.contains(&id));
    }

    #[test]
    fn give_up_respects_threshold(threshold in 1usize..20, count in 0usize..30) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(threshold)
            .build();
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..count {
            state.increment_error_count();
        }
        prop_assert_eq!(state.should_give_up(), count >= threshold);
    }

    #[test]
    fn error_node_byte_range_consistency(start in 0usize..1000, len in 0usize..1000) {
        let end = start + len;
        let config = ErrorRecoveryConfigBuilder::new().build();
        let mut state = ErrorRecoveryState::new(config);
        state.record_error(start, end, (0, 0), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
        prop_assert_eq!(nodes[0].start_byte, start);
        prop_assert_eq!(nodes[0].end_byte, end);
    }

    #[test]
    fn scope_push_pop_test_symmetry(token in 0u16..100) {
        let config = ErrorRecoveryConfigBuilder::new()
            .add_scope_delimiter(token, token.wrapping_add(1))
            .build();
        let mut state = ErrorRecoveryState::new(config);
        state.push_scope(token);
        let popped = state.pop_scope_test();
        prop_assert_eq!(popped, Some(token));
        prop_assert_eq!(state.pop_scope_test(), None);
    }

    #[test]
    fn is_scope_delimiter_symmetry(open in 0u16..100, close in 100u16..200) {
        let delimiters = vec![(open, close)];
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(open, &delimiters));
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(close, &delimiters));
        prop_assert!(!ErrorRecoveryState::is_scope_delimiter(close + 1, &delimiters));
    }

    #[test]
    fn is_matching_delimiter_exact(open in 0u16..100, close in 100u16..200) {
        let delimiters = vec![(open, close)];
        prop_assert!(ErrorRecoveryState::is_matching_delimiter(open, close, &delimiters));
        prop_assert!(!ErrorRecoveryState::is_matching_delimiter(close, open, &delimiters));
    }
}

// ---------------------------------------------------------------------------
// determine_recovery_strategy
// ---------------------------------------------------------------------------

#[test]
fn determine_strategy_returns_strategy() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(99), (0, 0), 0);
    // Should return some valid strategy variant
    let debug = format!("{:?}", strategy);
    assert!(!debug.is_empty());
}

#[test]
fn determine_strategy_with_no_expected() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[], None, (0, 0), 0);
    let debug = format!("{:?}", strategy);
    assert!(!debug.is_empty());
}

// ---------------------------------------------------------------------------
// Full builder chain test
// ---------------------------------------------------------------------------

#[test]
fn builder_full_chain() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .max_consecutive_errors(5)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(10)
        .add_insertable_token(11)
        .add_deletable_token(20)
        .add_scope_delimiter(30, 31)
        .enable_indentation_recovery(true)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .build();
    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(config.max_consecutive_errors, 5);
    assert_eq!(config.sync_tokens.len(), 2);
    assert_eq!(config.insert_candidates.len(), 2);
    assert_eq!(config.deletable_tokens.len(), 1);
    assert_eq!(config.scope_delimiters.len(), 1);
    assert!(config.enable_indentation_recovery);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_phrase_recovery);
}
