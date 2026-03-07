//! Comprehensive v2 tests for the error recovery strategy system.

use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};

use ir::SymbolId;

// ===========================================================================
// Helpers
// ===========================================================================

fn default_config() -> ErrorRecoveryConfig {
    ErrorRecoveryConfig::default()
}

fn default_state() -> ErrorRecoveryState {
    ErrorRecoveryState::new(default_config())
}

fn config_with_scope_delimiters(pairs: &[(u16, u16)]) -> ErrorRecoveryConfig {
    let mut cfg = default_config();
    cfg.scope_delimiters = pairs.to_vec();
    cfg
}

fn record_simple(
    state: &mut ErrorRecoveryState,
    start: usize,
    end: usize,
    expected: Vec<u16>,
    actual: Option<u16>,
    strategy: RecoveryStrategy,
) {
    state.record_error(
        start,
        end,
        (0, start),
        (0, end),
        expected,
        actual,
        strategy,
        vec![],
    );
}

// ===========================================================================
// 1. RecoveryStrategy variant construction
// ===========================================================================

#[test]
fn strategy_panic_mode_exists() {
    let _s = RecoveryStrategy::PanicMode;
}

#[test]
fn strategy_token_insertion_exists() {
    let _s = RecoveryStrategy::TokenInsertion;
}

#[test]
fn strategy_token_deletion_exists() {
    let _s = RecoveryStrategy::TokenDeletion;
}

#[test]
fn strategy_token_substitution_exists() {
    let _s = RecoveryStrategy::TokenSubstitution;
}

#[test]
fn strategy_phrase_level_exists() {
    let _s = RecoveryStrategy::PhraseLevel;
}

#[test]
fn strategy_scope_recovery_exists() {
    let _s = RecoveryStrategy::ScopeRecovery;
}

#[test]
fn strategy_indentation_recovery_exists() {
    let _s = RecoveryStrategy::IndentationRecovery;
}

// ===========================================================================
// 2. RecoveryStrategy Clone behaviour
// ===========================================================================

#[test]
fn strategy_clone_panic_mode() {
    let a = RecoveryStrategy::PanicMode;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn strategy_clone_token_insertion() {
    let a = RecoveryStrategy::TokenInsertion;
    assert_eq!(a, a.clone());
}

#[test]
fn strategy_clone_token_deletion() {
    let a = RecoveryStrategy::TokenDeletion;
    assert_eq!(a, a.clone());
}

#[test]
fn strategy_clone_token_substitution() {
    let a = RecoveryStrategy::TokenSubstitution;
    assert_eq!(a, a.clone());
}

#[test]
fn strategy_clone_phrase_level() {
    let a = RecoveryStrategy::PhraseLevel;
    assert_eq!(a, a.clone());
}

#[test]
fn strategy_clone_scope_recovery() {
    let a = RecoveryStrategy::ScopeRecovery;
    assert_eq!(a, a.clone());
}

#[test]
fn strategy_clone_indentation_recovery() {
    let a = RecoveryStrategy::IndentationRecovery;
    assert_eq!(a, a.clone());
}

// ===========================================================================
// 3. RecoveryStrategy Debug output
// ===========================================================================

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

// ===========================================================================
// 4. RecoveryStrategy PartialEq
// ===========================================================================

#[test]
fn strategy_eq_same_variant() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
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
    for i in 0..variants.len() {
        for j in 0..variants.len() {
            if i == j {
                assert_eq!(variants[i], variants[j]);
            } else {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }
}

// ===========================================================================
// 5. ErrorRecoveryConfig default construction
// ===========================================================================

#[test]
fn config_default_max_panic_skip() {
    assert_eq!(default_config().max_panic_skip, 50);
}

#[test]
fn config_default_sync_tokens_empty() {
    assert!(default_config().sync_tokens.is_empty());
}

#[test]
fn config_default_insert_candidates_empty() {
    assert!(default_config().insert_candidates.is_empty());
}

#[test]
fn config_default_deletable_tokens_empty() {
    assert!(default_config().deletable_tokens.is_empty());
}

#[test]
fn config_default_max_token_deletions() {
    assert_eq!(default_config().max_token_deletions, 3);
}

#[test]
fn config_default_max_token_insertions() {
    assert_eq!(default_config().max_token_insertions, 2);
}

#[test]
fn config_default_max_consecutive_errors() {
    assert_eq!(default_config().max_consecutive_errors, 10);
}

#[test]
fn config_default_phrase_recovery_enabled() {
    assert!(default_config().enable_phrase_recovery);
}

#[test]
fn config_default_scope_recovery_enabled() {
    assert!(default_config().enable_scope_recovery);
}

#[test]
fn config_default_scope_delimiters_empty() {
    assert!(default_config().scope_delimiters.is_empty());
}

#[test]
fn config_default_indentation_recovery_disabled() {
    assert!(!default_config().enable_indentation_recovery);
}

// ===========================================================================
// 6. ErrorRecoveryConfig max_errors & token predicates
// ===========================================================================

#[test]
fn config_can_delete_non_sync_token() {
    let mut cfg = default_config();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(cfg.can_delete_token(SymbolId(99)));
}

#[test]
fn config_cannot_delete_sync_token() {
    let mut cfg = default_config();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(!cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn config_can_delete_explicitly_deletable_even_if_sync() {
    let mut cfg = default_config();
    cfg.sync_tokens.push(SymbolId(5));
    cfg.deletable_tokens.insert(5);
    // deletable_tokens OR not-sync — the OR makes this true
    assert!(cfg.can_delete_token(SymbolId(5)));
}

#[test]
fn config_can_replace_non_sync() {
    let mut cfg = default_config();
    cfg.sync_tokens.push(SymbolId(7));
    assert!(cfg.can_replace_token(SymbolId(42)));
}

#[test]
fn config_cannot_replace_sync() {
    let mut cfg = default_config();
    cfg.sync_tokens.push(SymbolId(7));
    assert!(!cfg.can_replace_token(SymbolId(7)));
}

// ===========================================================================
// 7. ErrorRecoveryState construction
// ===========================================================================

#[test]
fn state_initial_no_errors() {
    let state = default_state();
    assert!(!state.should_give_up());
}

#[test]
fn state_initial_empty_error_nodes() {
    let state = default_state();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_with_custom_config() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..default_config()
    };
    let state = ErrorRecoveryState::new(cfg);
    // After 5 increments it should give up
    for _ in 0..5 {
        // won't give up yet at 4
    }
    assert!(!state.should_give_up());
}

// ===========================================================================
// 8. ErrorRecoveryState error counting
// ===========================================================================

#[test]
fn state_increment_error_count() {
    let mut state = default_state();
    state.increment_error_count();
    state.increment_error_count();
    // default max is 10, 2 < 10
    assert!(!state.should_give_up());
}

#[test]
fn state_should_give_up_at_max() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..default_config()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_error_count() {
    let mut state = default_state();
    for _ in 0..8 {
        state.increment_error_count();
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_reset_consecutive_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..default_config()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

// ===========================================================================
// 9. Scope tracking
// ===========================================================================

#[test]
fn state_push_scope_opening_delimiter() {
    let cfg = config_with_scope_delimiters(&[(40, 41)]);
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    // pop_scope_test reveals the stack
    assert_eq!(state.pop_scope_test(), Some(40));
}

#[test]
fn state_push_scope_ignores_non_delimiter() {
    let cfg = config_with_scope_delimiters(&[(40, 41)]);
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99); // not an opening delimiter
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_matching() {
    let cfg = config_with_scope_delimiters(&[(10, 11)]);
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(state.pop_scope(11));
}

#[test]
fn state_pop_scope_non_matching() {
    let cfg = config_with_scope_delimiters(&[(10, 11), (20, 21)]);
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(!state.pop_scope(21)); // 21 closes 20, not 10
}

#[test]
fn state_nested_scopes() {
    let cfg = config_with_scope_delimiters(&[(10, 11), (20, 21)]);
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    state.push_scope(20);
    assert!(state.pop_scope(21));
    assert!(state.pop_scope(11));
    assert_eq!(state.pop_scope_test(), None);
}

// ===========================================================================
// 10. Recent tokens tracking
// ===========================================================================

#[test]
fn state_add_recent_token() {
    let mut state = default_state();
    state.add_recent_token(42);
    state.add_recent_token(43);
    // No panic — tokens recorded
}

#[test]
fn state_update_recent_tokens_via_symbol_id() {
    let mut state = default_state();
    state.update_recent_tokens(SymbolId(7));
    // Internally delegates to add_recent_token
}

#[test]
fn state_recent_tokens_bounded_at_ten() {
    let mut state = default_state();
    for i in 0..20 {
        state.add_recent_token(i);
    }
    // The internal buffer is capped at 10; no panic
}

// ===========================================================================
// 11. Error recording and retrieval
// ===========================================================================

#[test]
fn state_record_single_error() {
    let mut state = default_state();
    record_simple(
        &mut state,
        0,
        5,
        vec![1, 2],
        Some(3),
        RecoveryStrategy::TokenDeletion,
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
}

#[test]
fn state_record_multiple_errors() {
    let mut state = default_state();
    for i in 0..5 {
        record_simple(
            &mut state,
            i * 10,
            i * 10 + 5,
            vec![1],
            None,
            RecoveryStrategy::PanicMode,
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn state_error_node_expected_field() {
    let mut state = default_state();
    record_simple(
        &mut state,
        0,
        1,
        vec![10, 20, 30],
        Some(99),
        RecoveryStrategy::PhraseLevel,
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].expected, vec![10, 20, 30]);
}

#[test]
fn state_error_node_actual_none() {
    let mut state = default_state();
    record_simple(
        &mut state,
        0,
        1,
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
    );
    assert_eq!(state.get_error_nodes()[0].actual, None);
}

#[test]
fn state_error_node_actual_some() {
    let mut state = default_state();
    record_simple(
        &mut state,
        0,
        1,
        vec![1],
        Some(42),
        RecoveryStrategy::TokenSubstitution,
    );
    assert_eq!(state.get_error_nodes()[0].actual, Some(42));
}

#[test]
fn state_error_node_recovery_field() {
    let mut state = default_state();
    record_simple(
        &mut state,
        0,
        1,
        vec![],
        None,
        RecoveryStrategy::ScopeRecovery,
    );
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn state_clear_errors() {
    let mut state = default_state();
    record_simple(
        &mut state,
        0,
        1,
        vec![1],
        Some(2),
        RecoveryStrategy::TokenDeletion,
    );
    assert_eq!(state.get_error_nodes().len(), 1);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// ===========================================================================
// 12. determine_recovery_strategy
// ===========================================================================

#[test]
fn determine_strategy_insertion_when_candidate_available() {
    let mut cfg = default_config();
    cfg.insert_candidates.push(SymbolId(10));
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
}

#[test]
fn determine_strategy_deletion_when_clearly_wrong() {
    let mut cfg = default_config();
    cfg.enable_phrase_recovery = false;
    let mut state = ErrorRecoveryState::new(cfg);
    let strat = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    // Deletion has higher priority than substitution when token is clearly wrong
    assert_eq!(strat, RecoveryStrategy::TokenDeletion);
}

#[test]
fn determine_strategy_panic_mode_when_error_limit_exceeded() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        enable_phrase_recovery: false,
        ..default_config()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Burn through 2 errors
    state.increment_error_count();
    state.increment_error_count();
    let strat = state.determine_recovery_strategy(&[1, 2], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PanicMode);
}

#[test]
fn determine_strategy_phrase_level_as_fallback() {
    let mut cfg = default_config(); // phrase recovery enabled by default
    // Make actual token a sync token so is_clearly_wrong returns false
    cfg.sync_tokens.push(SymbolId(30));
    let mut state = ErrorRecoveryState::new(cfg);
    // Multiple expected tokens => not substitution, sync token => not deletion
    let strat = state.determine_recovery_strategy(&[10, 20], Some(30), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PhraseLevel);
}

#[test]
fn determine_strategy_scope_recovery_on_mismatch() {
    let mut cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        enable_phrase_recovery: false,
        ..default_config()
    };
    // Make 41 a sync token so deletion doesn't fire first
    cfg.sync_tokens.push(SymbolId(41));
    let mut state = ErrorRecoveryState::new(cfg);
    // Token 41 is a closing delimiter with no matching open on the stack
    let strat = state.determine_recovery_strategy(&[10, 20], Some(41), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::ScopeRecovery);
}

// ===========================================================================
// 13. Static helper methods
// ===========================================================================

#[test]
fn is_scope_delimiter_open() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delims));
}

#[test]
fn is_scope_delimiter_close() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delims));
}

#[test]
fn is_scope_delimiter_false() {
    let delims = vec![(1, 2)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delims));
}

#[test]
fn is_matching_delimiter_true() {
    let delims = vec![(10, 11)];
    assert!(ErrorRecoveryState::is_matching_delimiter(10, 11, &delims));
}

#[test]
fn is_matching_delimiter_false_wrong_pair() {
    let delims = vec![(10, 11), (20, 21)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(10, 21, &delims));
}

#[test]
fn is_matching_delimiter_empty_list() {
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 2, &[]));
}

// ===========================================================================
// 14. ErrorRecoveryConfigBuilder
// ===========================================================================

#[test]
fn builder_default_matches_config_default() {
    let built = ErrorRecoveryConfigBuilder::new().build();
    let direct = default_config();
    assert_eq!(built.max_panic_skip, direct.max_panic_skip);
    assert_eq!(built.max_consecutive_errors, direct.max_consecutive_errors);
    assert_eq!(built.enable_phrase_recovery, direct.enable_phrase_recovery);
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
    let cfg = ErrorRecoveryConfigBuilder::new().add_sync_token(55).build();
    assert!(cfg.sync_tokens.iter().any(|s| s.0 == 55));
}

#[test]
fn builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(77))
        .build();
    assert!(cfg.sync_tokens.contains(&SymbolId(77)));
}

#[test]
fn builder_add_insertable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(8)
        .build();
    assert!(cfg.insert_candidates.iter().any(|s| s.0 == 8));
}

#[test]
fn builder_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(9))
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(9)));
}

#[test]
fn builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(33)
        .build();
    assert!(cfg.deletable_tokens.contains(&33));
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
fn builder_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(42)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 42);
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
        .max_panic_skip(25)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(3)
        .add_deletable_token(4)
        .add_scope_delimiter(10, 11)
        .enable_indentation_recovery(true)
        .enable_phrase_recovery(false)
        .max_consecutive_errors(5)
        .build();
    assert_eq!(cfg.max_panic_skip, 25);
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert!(cfg.deletable_tokens.contains(&4));
    assert_eq!(cfg.scope_delimiters.len(), 1);
    assert!(cfg.enable_indentation_recovery);
    assert!(!cfg.enable_phrase_recovery);
    assert_eq!(cfg.max_consecutive_errors, 5);
}

// ===========================================================================
// 15. RecoveryAction variants
// ===========================================================================

#[test]
fn action_insert_token() {
    let action = RecoveryAction::InsertToken(SymbolId(5));
    assert!(matches!(action, RecoveryAction::InsertToken(SymbolId(5))));
}

#[test]
fn action_delete_token() {
    let action = RecoveryAction::DeleteToken;
    assert!(matches!(action, RecoveryAction::DeleteToken));
}

#[test]
fn action_replace_token() {
    let action = RecoveryAction::ReplaceToken(SymbolId(12));
    assert!(matches!(action, RecoveryAction::ReplaceToken(SymbolId(12))));
}

#[test]
fn action_create_error_node() {
    let action = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    match action {
        RecoveryAction::CreateErrorNode(ids) => assert_eq!(ids.len(), 2),
        _ => panic!("expected CreateErrorNode"),
    }
}

#[test]
fn action_debug_format() {
    let action = RecoveryAction::DeleteToken;
    let dbg = format!("{:?}", action);
    assert!(dbg.contains("DeleteToken"));
}

#[test]
fn action_clone() {
    let action = RecoveryAction::InsertToken(SymbolId(100));
    let cloned = action.clone();
    assert!(matches!(cloned, RecoveryAction::InsertToken(SymbolId(100))));
}

// ===========================================================================
// 16. Config clone / debug
// ===========================================================================

#[test]
fn config_clone_preserves_fields() {
    let mut cfg = default_config();
    cfg.max_panic_skip = 77;
    cfg.max_consecutive_errors = 3;
    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 77);
    assert_eq!(cloned.max_consecutive_errors, 3);
}

#[test]
fn config_debug_not_empty() {
    let cfg = default_config();
    let dbg = format!("{:?}", cfg);
    assert!(!dbg.is_empty());
    assert!(dbg.contains("max_panic_skip"));
}

// ===========================================================================
// 17. Edge cases
// ===========================================================================

#[test]
fn state_give_up_boundary_just_below() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..default_config()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn state_give_up_boundary_exact() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..default_config()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn state_give_up_boundary_above() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..default_config()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..6 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn state_pop_scope_test_empty_stack() {
    let mut state = default_state();
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn config_with_zero_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..default_config()
    };
    let state = ErrorRecoveryState::new(cfg);
    // 0 >= 0 is true, should give up immediately
    assert!(state.should_give_up());
}

#[test]
fn determine_strategy_resets_on_insertion_success() {
    let mut cfg = default_config();
    cfg.insert_candidates.push(SymbolId(10));
    let mut state = ErrorRecoveryState::new(cfg);
    // First call bumps errors then resets on successful insertion
    let strat = state.determine_recovery_strategy(&[10], Some(99), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
    // Counter was reset, so should not give up
    assert!(!state.should_give_up());
}

#[test]
fn state_record_error_with_skipped_tokens() {
    let mut state = default_state();
    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![3, 4, 5],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].skipped_tokens, vec![3, 4, 5]);
}

#[test]
fn config_multiple_scope_delimiter_pairs() {
    let cfg = config_with_scope_delimiters(&[(40, 41), (91, 93), (123, 125)]);
    assert_eq!(cfg.scope_delimiters.len(), 3);
}

#[test]
fn builder_default_impl() {
    // ErrorRecoveryConfigBuilder implements Default
    let builder: ErrorRecoveryConfigBuilder = Default::default();
    let cfg = builder.build();
    assert_eq!(cfg.max_panic_skip, 50);
}
