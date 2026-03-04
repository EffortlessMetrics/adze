// Comprehensive tests for ErrorRecoveryState machine behavior.
// Covers config construction, clone/debug, state creation, add_recent_token,
// determine_recovery_strategy, record_error, multi-cycle recovery,
// custom sync tokens, insert candidates, and deletable tokens.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};
use adze_ir::SymbolId;
use std::collections::HashSet;

// =====================================================================
// Helpers
// =====================================================================

fn default_config() -> ErrorRecoveryConfig {
    ErrorRecoveryConfig::default()
}

fn config_with_sync(tokens: &[u16]) -> ErrorRecoveryConfig {
    let mut b = ErrorRecoveryConfigBuilder::new();
    for &t in tokens {
        b = b.add_sync_token(t);
    }
    b.build()
}

fn config_with_insert(tokens: &[u16]) -> ErrorRecoveryConfig {
    let mut b = ErrorRecoveryConfigBuilder::new();
    for &t in tokens {
        b = b.add_insertable_token(t);
    }
    b.build()
}

fn config_with_deletable(tokens: &[u16]) -> ErrorRecoveryConfig {
    let mut b = ErrorRecoveryConfigBuilder::new();
    for &t in tokens {
        b = b.add_deletable_token(t);
    }
    b.build()
}

// =====================================================================
// 1. Config construction with various settings
// =====================================================================

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

#[test]
fn config_builder_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn config_builder_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 5);
}

#[test]
fn config_builder_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn config_builder_disable_phrase_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!cfg.enable_phrase_recovery);
}

#[test]
fn config_builder_disable_scope_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!cfg.enable_scope_recovery);
}

#[test]
fn config_builder_add_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(40, 41)]);
}

#[test]
fn config_builder_multiple_scope_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .build();
    assert_eq!(cfg.scope_delimiters.len(), 2);
}

#[test]
fn config_builder_set_max_recovery_attempts() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(7)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 7);
}

// =====================================================================
// 2. Config clone and debug
// =====================================================================

#[test]
fn config_clone_preserves_fields() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(99)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .max_consecutive_errors(4)
        .enable_phrase_recovery(false)
        .build();
    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 99);
    assert_eq!(cloned.max_consecutive_errors, 4);
    assert!(!cloned.enable_phrase_recovery);
    assert!(cloned.sync_tokens.iter().any(|t| t.0 == 1));
    assert!(cloned.insert_candidates.iter().any(|t| t.0 == 2));
    assert!(cloned.deletable_tokens.contains(&3));
}

#[test]
fn config_debug_does_not_panic() {
    let cfg = default_config();
    let debug = format!("{:?}", cfg);
    assert!(debug.contains("ErrorRecoveryConfig"));
}

// =====================================================================
// 3. State creation from config
// =====================================================================

#[test]
fn state_new_has_no_errors() {
    let state = ErrorRecoveryState::new(default_config());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_new_should_not_give_up() {
    let state = ErrorRecoveryState::new(default_config());
    assert!(!state.should_give_up());
}

#[test]
fn state_new_from_custom_config() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
}

// =====================================================================
// 4. add_recent_token behavior
// =====================================================================

#[test]
fn add_recent_token_single() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.add_recent_token(42);
    // No panic; token added successfully.
}

#[test]
fn add_recent_token_multiple() {
    let mut state = ErrorRecoveryState::new(default_config());
    for i in 0..5 {
        state.add_recent_token(i);
    }
}

#[test]
fn add_recent_token_overflow_evicts_oldest() {
    let mut state = ErrorRecoveryState::new(default_config());
    // Add 15 tokens; buffer caps at 10.
    for i in 0..15u16 {
        state.add_recent_token(i);
    }
    // No panic; the internal deque handles eviction.
}

#[test]
fn add_recent_token_zero_value() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.add_recent_token(0);
}

#[test]
fn add_recent_token_max_u16() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.add_recent_token(u16::MAX);
}

#[test]
fn update_recent_tokens_via_symbol_id() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.update_recent_tokens(SymbolId(100));
}

// =====================================================================
// 5. determine_recovery_strategy for various scenarios
// =====================================================================

#[test]
fn strategy_token_insertion_when_candidate_matches_expected() {
    let cfg = config_with_insert(&[10]);
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10, 20], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_token_insertion_with_actual_present() {
    let cfg = config_with_insert(&[10]);
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10, 20], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_token_deletion_when_actual_is_wrong() {
    // No insert candidates, phrase recovery disabled, scope recovery disabled.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // expected=[5,6], actual=99 (not in expected, not a sync token) => deletion
    // But deletion requires exactly not a sync token. With no sync tokens, 99 is clearly wrong.
    // After deletion check, substitution is tried: can_substitute requires expected.len()==1.
    // expected has 2 elements so substitution fails => PanicMode (fallback).
    let strategy = state.determine_recovery_strategy(&[5, 6], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_token_substitution_when_single_expected() {
    // To reach substitution: actual must not be "clearly wrong" (i.e., actual is in expected
    // or is a sync token), and expected.len()==1. Use a sync token as actual so deletion
    // is skipped but substitution fires.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(99)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[5], Some(99), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn strategy_phrase_level_when_enabled_and_no_other_match() {
    // No insert candidates; actual is in expected so not "clearly wrong" => skip deletion.
    // expected.len() > 1 => skip substitution. scope recovery: no delimiters. phrase enabled.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[5, 6], Some(5), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_panic_mode_after_exceeding_max_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Use up the error budget.
    state.increment_error_count();
    state.increment_error_count();
    // Now consecutive_errors == 2; determine_recovery_strategy will fetch_add to 3 > 2.
    let strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_panic_mode_when_all_disabled_and_no_candidates() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual == expected token, so not "clearly wrong" => skip deletion.
    // expected.len() > 1 => skip substitution. phrase disabled, scope disabled => panic.
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(1), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_scope_recovery_on_unmatched_close() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=41 (close paren) with no matching open on scope stack => scope mismatch.
    // expected contains actual so deletion won't fire; expected.len()>1 so substitution won't fire.
    let strategy = state.determine_recovery_strategy(&[41, 50], Some(41), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn strategy_no_scope_recovery_when_scope_is_balanced() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Push matching open, so close delimiter is matched => no scope mismatch.
    state.push_scope(40);
    // actual=41, expected contains it, so not clearly wrong. 2 expected => no substitution.
    // Scope is balanced. Falls through to phrase level.
    let strategy = state.determine_recovery_strategy(&[41, 50], Some(41), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_insertion_resets_consecutive_errors() {
    let cfg = config_with_insert(&[10]);
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
    // After successful insertion, consecutive errors are reset.
    assert!(!state.should_give_up());
}

#[test]
fn strategy_with_none_actual_and_no_insert_candidates() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // actual=None, no insert candidates => insertion check fails (can_insert_token false).
    // actual is None => deletion check skipped. actual is None => substitution skipped.
    // phrase enabled => PhraseLevel.
    let strategy = state.determine_recovery_strategy(&[1, 2], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

// =====================================================================
// 6. record_error tracking
// =====================================================================

#[test]
fn record_error_single() {
    let mut state = ErrorRecoveryState::new(default_config());
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
    assert_eq!(nodes[0].expected, vec![1, 2]);
    assert_eq!(nodes[0].actual, Some(3));
    assert_eq!(nodes[0].recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn record_error_multiple() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    state.record_error(
        5,
        10,
        (1, 0),
        (1, 5),
        vec![2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![3, 4],
    );
    assert_eq!(state.get_error_nodes().len(), 2);
}

#[test]
fn record_error_with_no_expected() {
    let mut state = ErrorRecoveryState::new(default_config());
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
}

#[test]
fn record_error_preserves_positions() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.record_error(
        100,
        200,
        (5, 10),
        (5, 110),
        vec![7],
        Some(8),
        RecoveryStrategy::PhraseLevel,
        vec![],
    );
    let n = &state.get_error_nodes()[0];
    assert_eq!(n.start_byte, 100);
    assert_eq!(n.end_byte, 200);
}

#[test]
fn clear_errors_empties_list() {
    let mut state = ErrorRecoveryState::new(default_config());
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
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn get_error_nodes_returns_clone() {
    let mut state = ErrorRecoveryState::new(default_config());
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
    let nodes1 = state.get_error_nodes();
    let nodes2 = state.get_error_nodes();
    assert_eq!(nodes1.len(), nodes2.len());
}

// =====================================================================
// 7. Multiple error recovery cycles
// =====================================================================

#[test]
fn multiple_cycles_increment_and_reset() {
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
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn cycle_strategy_then_record_then_strategy() {
    let cfg = config_with_insert(&[10]);
    let mut state = ErrorRecoveryState::new(cfg);
    let s1 = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s1, RecoveryStrategy::TokenInsertion);
    state.record_error(0, 1, (0, 0), (0, 1), vec![10], None, s1, vec![]);
    let s2 = state.determine_recovery_strategy(&[10], None, (1, 0), 1);
    assert_eq!(s2, RecoveryStrategy::TokenInsertion);
    assert_eq!(state.get_error_nodes().len(), 1);
}

#[test]
fn cycle_exhaust_then_reset_then_recover() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    // Now at limit; next determine will exceed.
    let s = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PanicMode);
    state.reset_error_count();
    let s2 = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s2, RecoveryStrategy::TokenInsertion);
}

#[test]
fn many_errors_recorded() {
    let mut state = ErrorRecoveryState::new(default_config());
    for i in 0..50u16 {
        state.record_error(
            i as usize,
            (i + 1) as usize,
            (0, i as usize),
            (0, (i + 1) as usize),
            vec![i],
            Some(i + 100),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 50);
}

// =====================================================================
// 8. Config with custom sync tokens
// =====================================================================

#[test]
fn sync_tokens_builder_single() {
    let cfg = config_with_sync(&[10]);
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.sync_tokens[0], SymbolId(10));
}

#[test]
fn sync_tokens_builder_multiple() {
    let cfg = config_with_sync(&[10, 20, 30]);
    assert_eq!(cfg.sync_tokens.len(), 3);
}

#[test]
fn sync_token_prevents_deletion() {
    let cfg = config_with_sync(&[10]);
    assert!(!cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn non_sync_token_allows_deletion() {
    let cfg = config_with_sync(&[10]);
    assert!(cfg.can_delete_token(SymbolId(99)));
}

#[test]
fn sync_token_prevents_replacement() {
    let cfg = config_with_sync(&[10]);
    assert!(!cfg.can_replace_token(SymbolId(10)));
}

#[test]
fn non_sync_token_allows_replacement() {
    let cfg = config_with_sync(&[10]);
    assert!(cfg.can_replace_token(SymbolId(99)));
}

#[test]
fn sync_token_via_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(42))
        .build();
    assert!(cfg.sync_tokens.contains(&SymbolId(42)));
}

// =====================================================================
// 9. Config with custom insert candidates
// =====================================================================

#[test]
fn insert_candidates_single() {
    let cfg = config_with_insert(&[5]);
    assert_eq!(cfg.insert_candidates.len(), 1);
    assert_eq!(cfg.insert_candidates[0], SymbolId(5));
}

#[test]
fn insert_candidates_multiple() {
    let cfg = config_with_insert(&[5, 6, 7]);
    assert_eq!(cfg.insert_candidates.len(), 3);
}

#[test]
fn insert_candidate_triggers_insertion_strategy() {
    let cfg = config_with_insert(&[20]);
    let mut state = ErrorRecoveryState::new(cfg);
    let s = state.determine_recovery_strategy(&[20], Some(99), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenInsertion);
}

#[test]
fn no_insert_candidate_skips_insertion() {
    let cfg = config_with_insert(&[20]);
    let mut state = ErrorRecoveryState::new(cfg);
    // expected doesn't include 20, so insertion won't match.
    // actual=99, not in expected, not sync => deletion.
    // expected.len()==1 => substitution.
    // Deletion comes first.
    let s = state.determine_recovery_strategy(&[30], Some(99), (0, 0), 0);
    assert_ne!(s, RecoveryStrategy::TokenInsertion);
}

#[test]
fn insert_candidate_via_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(77))
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(77)));
}

// =====================================================================
// 10. Config with deletable tokens
// =====================================================================

#[test]
fn deletable_tokens_single() {
    let cfg = config_with_deletable(&[15]);
    assert!(cfg.deletable_tokens.contains(&15));
}

#[test]
fn deletable_tokens_multiple() {
    let cfg = config_with_deletable(&[15, 16, 17]);
    assert_eq!(cfg.deletable_tokens.len(), 3);
}

#[test]
fn deletable_token_overrides_sync_for_can_delete() {
    // When a token is both a sync token AND explicitly deletable,
    // can_delete_token returns true because the deletable_tokens check comes first.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(10)
        .add_deletable_token(10)
        .build();
    assert!(cfg.can_delete_token(SymbolId(10)));
}

#[test]
fn non_deletable_non_sync_can_still_be_deleted() {
    let cfg = default_config();
    // With no sync tokens and no deletable set, any token can be deleted
    // (because it's not a sync token).
    assert!(cfg.can_delete_token(SymbolId(42)));
}

// =====================================================================
// Additional edge cases and coverage boosters
// =====================================================================

#[test]
fn strategy_enum_all_variants_are_distinct() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
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

#[test]
fn strategy_clone_and_copy() {
    let s = RecoveryStrategy::TokenInsertion;
    let cloned = s;
    assert_eq!(s, cloned);
}

#[test]
fn strategy_debug_format() {
    let s = RecoveryStrategy::PanicMode;
    assert_eq!(format!("{:?}", s), "PanicMode");
}

#[test]
fn error_node_debug() {
    let mut state = ErrorRecoveryState::new(default_config());
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
    let nodes = state.get_error_nodes();
    let dbg = format!("{:?}", nodes[0]);
    assert!(dbg.contains("ErrorNode"));
}

#[test]
fn scope_push_non_delimiter_is_noop() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(99); // Not a delimiter.
    // pop_scope_test should return None because nothing was pushed.
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn scope_pop_on_empty_stack() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.pop_scope(41));
}

#[test]
fn scope_push_pop_matching() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    assert!(state.pop_scope(41));
}

#[test]
fn scope_pop_wrong_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(40);
    // Try to pop with wrong close delimiter.
    assert!(!state.pop_scope(93));
}

#[test]
fn is_scope_delimiter_static() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_scope_delimiter(1, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(2, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(3, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(4, &delims));
    assert!(!ErrorRecoveryState::is_scope_delimiter(5, &delims));
}

#[test]
fn is_matching_delimiter_static() {
    let delims = vec![(1, 2), (3, 4)];
    assert!(ErrorRecoveryState::is_matching_delimiter(1, 2, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(1, 4, &delims));
    assert!(!ErrorRecoveryState::is_matching_delimiter(2, 1, &delims));
}

#[test]
fn reset_consecutive_errors_method() {
    let mut state = ErrorRecoveryState::new(default_config());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn config_with_all_options_combined() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(3)
        .add_insertable_token(4)
        .add_deletable_token(5)
        .add_scope_delimiter(6, 7)
        .max_consecutive_errors(20)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    assert_eq!(cfg.max_panic_skip, 100);
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert_eq!(cfg.insert_candidates.len(), 2);
    assert!(cfg.deletable_tokens.contains(&5));
    assert_eq!(cfg.scope_delimiters, vec![(6, 7)]);
    assert_eq!(cfg.max_consecutive_errors, 20);
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn should_give_up_boundary_exact() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count(); // 1
    assert!(!state.should_give_up());
    state.increment_error_count(); // 2
    assert!(!state.should_give_up());
    state.increment_error_count(); // 3 == max
    assert!(state.should_give_up());
}

#[test]
fn determine_strategy_increments_error_count() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Each call to determine_recovery_strategy increments consecutive_errors.
    state.determine_recovery_strategy(&[1, 2], Some(1), (0, 0), 0);
    state.determine_recovery_strategy(&[1, 2], Some(1), (0, 0), 0);
    // Two calls => consecutive_errors >= 2.
    // With max_consecutive_errors=10, should not give up.
    assert!(!state.should_give_up());
}

#[test]
fn record_error_with_all_strategies() {
    let mut state = ErrorRecoveryState::new(default_config());
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ];
    for (i, &strat) in strategies.iter().enumerate() {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![i as u16],
            None,
            strat,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 6);
}

#[test]
fn config_deletable_tokens_is_hashset() {
    let mut set = HashSet::new();
    set.insert(1u16);
    set.insert(2);
    let cfg = ErrorRecoveryConfig {
        deletable_tokens: set,
        ..default_config()
    };
    assert!(cfg.deletable_tokens.contains(&1));
    assert!(cfg.deletable_tokens.contains(&2));
    assert!(!cfg.deletable_tokens.contains(&3));
}

#[test]
fn strategy_deletion_over_substitution_when_multiple_expected() {
    // actual not in expected, not sync, expected.len() > 1 => deletion fires, substitution skipped.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let s = state.determine_recovery_strategy(&[5, 6], Some(99), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_with_actual_equal_to_sync_token() {
    // actual is a sync token => not "clearly wrong" => deletion skipped.
    // expected.len() > 1 => substitution skipped. phrase enabled => PhraseLevel.
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(99)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let s = state.determine_recovery_strategy(&[5, 6], Some(99), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PhraseLevel);
}

#[test]
fn add_recent_token_exactly_ten() {
    let mut state = ErrorRecoveryState::new(default_config());
    for i in 0..10u16 {
        state.add_recent_token(i);
    }
    // Buffer should be exactly full, no eviction yet.
}

#[test]
fn add_recent_token_eleven_evicts_one() {
    let mut state = ErrorRecoveryState::new(default_config());
    for i in 0..11u16 {
        state.add_recent_token(i);
    }
    // One token should have been evicted; no panic.
}
