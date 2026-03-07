//! Comprehensive tests for error recovery strategies in `adze::error_recovery`.
//!
//! Covers: ErrorRecoveryConfig, RecoveryStrategy, RecoveryAction,
//! ErrorRecoveryConfigBuilder, ErrorRecoveryState, ErrorNode, and helper methods.

use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};

use ir::SymbolId;

// =========================================================================
// 1. RecoveryStrategy — trait impls and variant coverage
// =========================================================================

#[test]
fn strategy_debug_all_variants() {
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
        let dbg = format!("{v:?}");
        assert!(!dbg.is_empty());
    }
}

#[test]
fn strategy_clone() {
    let s = RecoveryStrategy::PanicMode;
    let s2 = s;
    assert_eq!(s, s2);
}

#[test]
fn strategy_copy() {
    let s = RecoveryStrategy::TokenInsertion;
    let s2 = s;
    // Both copies should be equal and usable independently.
    assert_eq!(s, s2);
}

#[test]
fn strategy_eq_same() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn strategy_ne_different() {
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
    assert_ne!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::PhraseLevel
    );
}

#[test]
fn strategy_all_variants_are_distinct() {
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
                assert_ne!(a, b, "variants at {i} and {j} should differ");
            }
        }
    }
}

// =========================================================================
// 2. RecoveryAction — construction, pattern matching, Debug, Clone
// =========================================================================

#[test]
fn action_insert_token() {
    let a = RecoveryAction::InsertToken(SymbolId(42));
    match &a {
        RecoveryAction::InsertToken(id) => assert_eq!(*id, SymbolId(42)),
        _ => panic!("expected InsertToken"),
    }
}

#[test]
fn action_delete_token() {
    let a = RecoveryAction::DeleteToken;
    assert!(matches!(a, RecoveryAction::DeleteToken));
}

#[test]
fn action_replace_token() {
    let a = RecoveryAction::ReplaceToken(SymbolId(7));
    match &a {
        RecoveryAction::ReplaceToken(id) => assert_eq!(*id, SymbolId(7)),
        _ => panic!("expected ReplaceToken"),
    }
}

#[test]
fn action_create_error_node() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    match &a {
        RecoveryAction::CreateErrorNode(ids) => {
            assert_eq!(ids.len(), 2);
            assert_eq!(ids[0], SymbolId(1));
        }
        _ => panic!("expected CreateErrorNode"),
    }
}

#[test]
fn action_create_error_node_empty() {
    let a = RecoveryAction::CreateErrorNode(vec![]);
    match &a {
        RecoveryAction::CreateErrorNode(ids) => assert!(ids.is_empty()),
        _ => panic!("expected CreateErrorNode"),
    }
}

#[test]
fn action_debug() {
    let a = RecoveryAction::DeleteToken;
    let dbg = format!("{a:?}");
    assert!(dbg.contains("DeleteToken"));
}

#[test]
fn action_clone() {
    let a = RecoveryAction::InsertToken(SymbolId(5));
    let b = a.clone();
    match (&a, &b) {
        (RecoveryAction::InsertToken(x), RecoveryAction::InsertToken(y)) => {
            assert_eq!(x, y);
        }
        _ => panic!("clone mismatch"),
    }
}

#[test]
fn action_clone_error_node_is_deep() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(10)]);
    let b = a.clone();
    // Cloned vec is independent.
    match (a, b) {
        (RecoveryAction::CreateErrorNode(v1), RecoveryAction::CreateErrorNode(v2)) => {
            assert_eq!(v1, v2);
        }
        _ => panic!("clone mismatch"),
    }
}

// =========================================================================
// 3. ErrorRecoveryConfig — defaults
// =========================================================================

#[test]
fn config_default_max_panic_skip() {
    assert_eq!(ErrorRecoveryConfig::default().max_panic_skip, 50);
}

#[test]
fn config_default_sync_tokens_empty() {
    assert!(ErrorRecoveryConfig::default().sync_tokens.is_empty());
}

#[test]
fn config_default_insert_candidates_empty() {
    assert!(ErrorRecoveryConfig::default().insert_candidates.is_empty());
}

#[test]
fn config_default_deletable_tokens_empty() {
    assert!(ErrorRecoveryConfig::default().deletable_tokens.is_empty());
}

#[test]
fn config_default_max_token_deletions() {
    assert_eq!(ErrorRecoveryConfig::default().max_token_deletions, 3);
}

#[test]
fn config_default_max_token_insertions() {
    assert_eq!(ErrorRecoveryConfig::default().max_token_insertions, 2);
}

#[test]
fn config_default_max_consecutive_errors() {
    assert_eq!(ErrorRecoveryConfig::default().max_consecutive_errors, 10);
}

#[test]
fn config_default_phrase_recovery_enabled() {
    assert!(ErrorRecoveryConfig::default().enable_phrase_recovery);
}

#[test]
fn config_default_scope_recovery_enabled() {
    assert!(ErrorRecoveryConfig::default().enable_scope_recovery);
}

#[test]
fn config_default_scope_delimiters_empty() {
    assert!(ErrorRecoveryConfig::default().scope_delimiters.is_empty());
}

#[test]
fn config_default_indentation_recovery_disabled() {
    assert!(!ErrorRecoveryConfig::default().enable_indentation_recovery);
}

// =========================================================================
// 4. ErrorRecoveryConfig — can_delete_token / can_replace_token
// =========================================================================

#[test]
fn config_can_delete_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(cfg.can_delete_token(SymbolId(5)));
}

#[test]
fn config_cannot_delete_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    assert!(!cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn config_can_delete_explicitly_deletable_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(10));
    cfg.deletable_tokens.insert(10);
    // Explicitly deletable trumps sync-token check.
    assert!(cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn config_can_delete_with_empty_sync_tokens() {
    let cfg = ErrorRecoveryConfig::default();
    // No sync tokens → any token can be deleted.
    assert!(cfg.can_delete_token(SymbolId(999)));
}

#[test]
fn config_can_replace_non_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(20));
    assert!(cfg.can_replace_token(SymbolId(15)));
}

#[test]
fn config_cannot_replace_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(20));
    assert!(!cfg.can_replace_token(SymbolId(20)));
}

#[test]
fn config_can_replace_with_empty_sync_tokens() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.can_replace_token(SymbolId(0)));
}

// =========================================================================
// 5. ErrorRecoveryConfig — Clone and Debug
// =========================================================================

#[test]
fn config_clone() {
    let mut cfg = ErrorRecoveryConfig {
        max_panic_skip: 77,
        ..Default::default()
    };
    cfg.sync_tokens.push(SymbolId(1));
    let cfg2 = cfg.clone();
    assert_eq!(cfg2.max_panic_skip, 77);
    assert_eq!(cfg2.sync_tokens.len(), 1);
}

#[test]
fn config_debug() {
    let cfg = ErrorRecoveryConfig::default();
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("max_panic_skip"));
}

// =========================================================================
// 6. ErrorRecoveryConfigBuilder — builder pattern
// =========================================================================

#[test]
fn builder_default_produces_default_config() {
    let built = ErrorRecoveryConfigBuilder::new().build();
    let def = ErrorRecoveryConfig::default();
    assert_eq!(built.max_panic_skip, def.max_panic_skip);
    assert_eq!(built.max_consecutive_errors, def.max_consecutive_errors);
}

#[test]
fn builder_default_trait() {
    let b = ErrorRecoveryConfigBuilder::default();
    let cfg = b.build();
    assert_eq!(cfg.max_panic_skip, 50);
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
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 7));
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 8));
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
        .add_insertable_token(3)
        .build();
    assert!(cfg.insert_candidates.iter().any(|t| t.0 == 3));
}

#[test]
fn builder_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(99))
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(99)));
}

#[test]
fn builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(15)
        .build();
    assert!(cfg.deletable_tokens.contains(&15));
}

#[test]
fn builder_add_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .build();
    assert_eq!(cfg.scope_delimiters.len(), 2);
    assert_eq!(cfg.scope_delimiters[0], (40, 41));
    assert_eq!(cfg.scope_delimiters[1], (91, 93));
}

#[test]
fn builder_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_enable_scope_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!cfg.enable_scope_recovery);
}

#[test]
fn builder_enable_phrase_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!cfg.enable_phrase_recovery);
}

#[test]
fn builder_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(55)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 55);
}

#[test]
fn builder_set_max_recovery_attempts() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(33)
        .build();
    // set_max_recovery_attempts sets max_consecutive_errors
    assert_eq!(cfg.max_consecutive_errors, 33);
}

#[test]
fn builder_chaining_all_methods() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_sync_token_sym(SymbolId(2))
        .add_insertable_token(3)
        .add_insertable_token_sym(SymbolId(4))
        .add_deletable_token(5)
        .add_scope_delimiter(6, 7)
        .enable_indentation_recovery(true)
        .enable_scope_recovery(false)
        .enable_phrase_recovery(false)
        .max_consecutive_errors(99)
        .build();

    assert_eq!(cfg.max_panic_skip, 10);
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert_eq!(cfg.insert_candidates.len(), 2);
    assert!(cfg.deletable_tokens.contains(&5));
    assert_eq!(cfg.scope_delimiters, vec![(6, 7)]);
    assert!(cfg.enable_indentation_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(!cfg.enable_phrase_recovery);
    assert_eq!(cfg.max_consecutive_errors, 99);
}

// =========================================================================
// 7. ErrorRecoveryState — creation and basic operations
// =========================================================================

#[test]
fn state_new_no_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
    assert!(!state.should_give_up());
}

#[test]
fn state_increment_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.increment_error_count();
    state.increment_error_count();
    // Default max_consecutive_errors is 10, so 2 shouldn't give up.
    assert!(!state.should_give_up());
}

#[test]
fn state_should_give_up_at_limit() {
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
fn state_should_give_up_over_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    assert!(state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_error_count() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..20 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_reset_consecutive_errors() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for _ in 0..15 {
        state.increment_error_count();
    }
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

// =========================================================================
// 8. ErrorRecoveryState — error recording
// =========================================================================

fn make_state() -> ErrorRecoveryState {
    ErrorRecoveryState::new(ErrorRecoveryConfig::default())
}

fn record_error(state: &mut ErrorRecoveryState, start: usize, end: usize) {
    state.record_error(
        start,
        end,
        (0, start),
        (0, end),
        vec![1, 2],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
}

#[test]
fn state_record_one_error() {
    let mut state = make_state();
    record_error(&mut state, 0, 5);
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
}

#[test]
fn state_record_multiple_errors() {
    let mut state = make_state();
    for i in 0..5 {
        record_error(&mut state, i * 10, i * 10 + 5);
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn state_clear_errors() {
    let mut state = make_state();
    record_error(&mut state, 0, 1);
    record_error(&mut state, 1, 2);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_clear_errors_idempotent() {
    let mut state = make_state();
    state.clear_errors();
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_record_error_fields() {
    let mut state = make_state();
    state.record_error(
        10,
        20,
        (1, 5),
        (1, 15),
        vec![100, 200],
        Some(300),
        RecoveryStrategy::PanicMode,
        vec![301, 302],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, 10);
    assert_eq!(nodes[0].end_byte, 20);
    assert_eq!(nodes[0].expected, vec![100, 200]);
    assert_eq!(nodes[0].actual, Some(300));
    assert_eq!(nodes[0].recovery, RecoveryStrategy::PanicMode);
}

#[test]
fn state_record_error_no_actual() {
    let mut state = make_state();
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].actual, None);
}

// =========================================================================
// 9. ErrorRecoveryState — scope management
// =========================================================================

#[test]
fn state_push_scope_opening_delimiter() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    assert_eq!(state.pop_scope_test(), Some(40));
}

#[test]
fn state_push_scope_non_delimiter_ignored() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(40, 41)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99); // not an opening delimiter
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn state_pop_scope_matching() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(state.pop_scope(11));
}

#[test]
fn state_pop_scope_mismatched() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    // Trying to pop with wrong close delimiter.
    assert!(!state.pop_scope(21));
}

#[test]
fn state_pop_scope_empty_stack() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.pop_scope(11));
}

#[test]
fn state_nested_scopes() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(10, 11), (20, 21)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    state.push_scope(20);
    state.push_scope(10);
    // Pop innermost first.
    assert!(state.pop_scope(11));
    assert!(state.pop_scope(21));
    assert!(state.pop_scope(11));
}

#[test]
fn state_pop_scope_test_returns_raw_stack() {
    let cfg = ErrorRecoveryConfig {
        scope_delimiters: vec![(1, 2), (3, 4)],
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(1);
    state.push_scope(3);
    assert_eq!(state.pop_scope_test(), Some(3));
    assert_eq!(state.pop_scope_test(), Some(1));
    assert_eq!(state.pop_scope_test(), None);
}

// =========================================================================
// 10. ErrorRecoveryState — recent tokens
// =========================================================================

#[test]
fn state_add_recent_token_single() {
    let mut state = make_state();
    state.add_recent_token(42);
    // No public accessor for recent_tokens, but we can exercise the path.
}

#[test]
fn state_add_recent_tokens_overflow_cap() {
    let mut state = make_state();
    for i in 0..20u16 {
        state.add_recent_token(i);
    }
    // Internal buffer is capped at 10; no panic.
}

#[test]
fn state_update_recent_tokens_via_symbol_id() {
    let mut state = make_state();
    state.update_recent_tokens(SymbolId(7));
    state.update_recent_tokens(SymbolId(8));
}

// =========================================================================
// 11. ErrorRecoveryState — static helpers
// =========================================================================

#[test]
fn is_scope_delimiter_open() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delims));
}

#[test]
fn is_scope_delimiter_close() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delims));
}

#[test]
fn is_scope_delimiter_negative() {
    let delims = vec![(1, 2)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delims));
}

#[test]
fn is_scope_delimiter_empty_list() {
    assert!(!ErrorRecoveryState::is_scope_delimiter(1, &[]));
}

#[test]
fn is_matching_delimiter_true() {
    let delims = vec![(10, 11), (20, 21)];
    assert!(ErrorRecoveryState::is_matching_delimiter(10, 11, &delims));
    assert!(ErrorRecoveryState::is_matching_delimiter(20, 21, &delims));
}

#[test]
fn is_matching_delimiter_false_swapped() {
    let delims = vec![(10, 11)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(11, 10, &delims));
}

#[test]
fn is_matching_delimiter_false_cross() {
    let delims = vec![(10, 11), (20, 21)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(10, 21, &delims));
}

#[test]
fn is_matching_delimiter_empty_list() {
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 2, &[]));
}

// =========================================================================
// 12. ErrorRecoveryState — determine_recovery_strategy
// =========================================================================

#[test]
fn determine_strategy_insertion_when_insertable() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let s = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenInsertion);
}

#[test]
fn determine_strategy_deletion_when_clearly_wrong() {
    // No insertable tokens, token not in expected set and not a sync token.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let s = state.determine_recovery_strategy(&[1, 2, 3], Some(99), (0, 0), 0);
    // With phrase & scope disabled, deletion should apply since 99 is clearly wrong.
    assert_eq!(s, RecoveryStrategy::TokenDeletion);
}

#[test]
fn determine_strategy_substitution_single_expected() {
    // Exactly one expected token → substitution.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(50) // make actual a sync token so deletion won't fire
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let s = state.determine_recovery_strategy(&[7], Some(50), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn determine_strategy_phrase_level_fallback() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(50)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Multiple expected tokens and actual is a sync token → falls through to phrase-level.
    let s = state.determine_recovery_strategy(&[1, 2, 3], Some(50), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PhraseLevel);
}

#[test]
fn determine_strategy_panic_mode_when_all_disabled() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(50)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Actual is sync token, multiple expected → can't delete, can't substitute, no phrase/scope
    let s = state.determine_recovery_strategy(&[1, 2, 3], Some(50), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

#[test]
fn determine_strategy_panic_mode_over_error_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Burn through the limit.
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    let s = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

#[test]
fn determine_strategy_scope_recovery_on_unmatched_close() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_sync_token(41) // make close paren a sync token so deletion won't apply
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Actual is a closing delimiter with nothing on the scope stack.
    let s = state.determine_recovery_strategy(&[1, 2], Some(41), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::ScopeRecovery);
}

// =========================================================================
// 13. ErrorNode — construction and fields
// =========================================================================

#[test]
fn error_node_fields() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 10,
        start_position: (1, 0),
        end_position: (1, 5),
        expected: vec![1, 2, 3],
        actual: Some(4),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![4, 5],
    };
    assert_eq!(node.start_byte, 5);
    assert_eq!(node.end_byte, 10);
    assert_eq!(node.expected, vec![1, 2, 3]);
    assert_eq!(node.actual, Some(4));
    assert_eq!(node.recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn error_node_no_actual() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 0,
        start_position: (0, 0),
        end_position: (0, 0),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert!(node.actual.is_none());
    assert!(node.expected.is_empty());
}

#[test]
fn error_node_clone() {
    let node = ErrorNode {
        start_byte: 1,
        end_byte: 2,
        start_position: (0, 1),
        end_position: (0, 2),
        expected: vec![10],
        actual: Some(20),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![20],
    };
    let node2 = node.clone();
    assert_eq!(node2.start_byte, 1);
    assert_eq!(node2.recovery, RecoveryStrategy::PhraseLevel);
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
    let dbg = format!("{node:?}");
    assert!(dbg.contains("ErrorNode"));
}

// =========================================================================
// 14. Configuration edge cases
// =========================================================================

#[test]
fn config_zero_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 0,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, 0);
}

#[test]
fn config_zero_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    // With 0 limit, should_give_up is true from the start.
    let state = ErrorRecoveryState::new(cfg);
    assert!(state.should_give_up());
}

#[test]
fn config_large_max_values() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(usize::MAX)
        .max_consecutive_errors(usize::MAX)
        .build();
    assert_eq!(cfg.max_panic_skip, usize::MAX);
    assert_eq!(cfg.max_consecutive_errors, usize::MAX);
}

#[test]
fn config_many_sync_tokens() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in 0..100 {
        builder = builder.add_sync_token(i);
    }
    let cfg = builder.build();
    assert_eq!(cfg.sync_tokens.len(), 100);
}

#[test]
fn config_many_insertable_tokens() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in 0..50 {
        builder = builder.add_insertable_token(i);
    }
    let cfg = builder.build();
    assert_eq!(cfg.insert_candidates.len(), 50);
}

#[test]
fn config_many_scope_delimiters() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in 0..25u16 {
        builder = builder.add_scope_delimiter(i * 2, i * 2 + 1);
    }
    let cfg = builder.build();
    assert_eq!(cfg.scope_delimiters.len(), 25);
}

#[test]
fn config_duplicate_deletable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .add_deletable_token(5)
        .add_deletable_token(5)
        .build();
    // HashSet deduplicates.
    assert_eq!(cfg.deletable_tokens.len(), 1);
}

#[test]
fn config_deletable_tokens_set_operations() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.deletable_tokens.insert(1);
    cfg.deletable_tokens.insert(2);
    cfg.deletable_tokens.insert(3);
    assert!(cfg.deletable_tokens.contains(&2));
    assert!(!cfg.deletable_tokens.contains(&4));
}

// =========================================================================
// 15. determine_recovery_strategy — resets on successful insertion
// =========================================================================

#[test]
fn determine_strategy_resets_errors_on_insertion() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .max_consecutive_errors(5)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Accumulate some errors.
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    // Insertion resets the counter.
    let s = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenInsertion);
    // After insertion, should not give up.
    assert!(!state.should_give_up());
}

// =========================================================================
// 16. Multiple error recordings with different strategies
// =========================================================================

#[test]
fn record_errors_with_various_strategies() {
    let mut state = make_state();
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ];
    for (i, strat) in strategies.iter().enumerate() {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![1],
            Some(2),
            *strat,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), strategies.len());
    for (i, node) in nodes.iter().enumerate() {
        assert_eq!(node.recovery, strategies[i]);
    }
}

// =========================================================================
// 17. Config with all recovery modes disabled
// =========================================================================

#[test]
fn all_recovery_disabled_falls_to_panic() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1) // actual is a sync token so deletion/clearly-wrong won't fire
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Multiple expected → substitution won't fire (needs exactly 1 expected).
    let s = state.determine_recovery_strategy(&[2, 3], Some(1), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

// =========================================================================
// 18. Config with all recovery modes enabled
// =========================================================================

#[test]
fn all_recovery_enabled_config() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_phrase_recovery);
    assert!(cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
}

// =========================================================================
// 19. get_error_nodes returns cloned data
// =========================================================================

#[test]
fn get_error_nodes_returns_independent_clone() {
    let mut state = make_state();
    record_error(&mut state, 0, 5);
    let nodes1 = state.get_error_nodes();
    record_error(&mut state, 5, 10);
    let nodes2 = state.get_error_nodes();
    // First snapshot should be unaffected by subsequent recording.
    assert_eq!(nodes1.len(), 1);
    assert_eq!(nodes2.len(), 2);
}
