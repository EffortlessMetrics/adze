#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for the error recovery system.
//!
//! Covers: construction, recording with every strategy variant, retrieval,
//! multiple errors, position tracking, expected/actual info, skipped tokens,
//! strategy selection, edge cases, and state reset/clear.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};

// ── 1. ErrorRecoveryState construction ──────────────────────────────────────

#[test]
fn state_default_has_no_errors() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
    assert!(!state.should_give_up());
}

#[test]
fn state_with_custom_config_retains_limits() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .max_panic_skip(100)
        .add_sync_token(42)
        .add_insertable_token(7)
        .add_deletable_token(99)
        .add_scope_delimiter(10, 11)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    let state = ErrorRecoveryState::new(config);
    // Should start clean regardless of config
    assert!(state.get_error_nodes().is_empty());
    assert!(!state.should_give_up());
}

// ── 2. Recording errors with all RecoveryStrategy variants ──────────────────

fn record_one(state: &mut ErrorRecoveryState, strategy: RecoveryStrategy) {
    state.record_error(0, 1, (0, 0), (0, 1), vec![1], Some(2), strategy, vec![]);
}

#[test]
fn record_error_panic_mode() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    record_one(&mut state, RecoveryStrategy::PanicMode);
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::PanicMode
    );
}

#[test]
fn record_error_token_insertion() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    record_one(&mut state, RecoveryStrategy::TokenInsertion);
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn record_error_token_deletion() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    record_one(&mut state, RecoveryStrategy::TokenDeletion);
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn record_error_token_substitution() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    record_one(&mut state, RecoveryStrategy::TokenSubstitution);
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn record_error_phrase_level() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    record_one(&mut state, RecoveryStrategy::PhraseLevel);
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::PhraseLevel
    );
}

// ── 3. Getting error nodes back ─────────────────────────────────────────────

#[test]
fn get_error_nodes_returns_all_recorded() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
    ];
    for (i, &strat) in strategies.iter().enumerate() {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![1],
            Some(2),
            strat,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 3);
    for i in 0..strategies.len() {
        assert_eq!(nodes[i].recovery, strategies[i]);
    }
}

#[test]
fn get_error_nodes_empty_when_none_recorded() {
    let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    assert!(state.get_error_nodes().is_empty());
}

// ── 4. Multiple errors recorded ─────────────────────────────────────────────

#[test]
fn ten_errors_all_retrievable() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    for i in 0..10 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (i, 0),
            (i, 5),
            vec![i as u16],
            Some((i + 100) as u16),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 10);
    for i in 0..10 {
        assert_eq!(nodes[i].start_byte, i * 10);
        assert_eq!(nodes[i].expected, vec![i as u16]);
        assert_eq!(nodes[i].actual, Some((i + 100) as u16));
    }
}

// ── 5. Error position tracking ──────────────────────────────────────────────

#[test]
fn byte_positions_preserved() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        42,
        99,
        (3, 10),
        (3, 67),
        vec![5],
        Some(6),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let n = &state.get_error_nodes()[0];
    assert_eq!(n.start_byte, 42);
    assert_eq!(n.end_byte, 99);
}

#[test]
fn multiple_positions_ordered() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    let positions: [(usize, usize); 3] = [(0, 5), (100, 110), (500, 505)];
    for &(s, e) in &positions {
        state.record_error(
            s,
            e,
            (0, 0),
            (0, 0),
            vec![1],
            Some(2),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    for i in 0..positions.len() {
        assert_eq!(nodes[i].start_byte, positions[i].0);
        assert_eq!(nodes[i].end_byte, positions[i].1);
    }
}

// ── 6. Expected/actual token info ───────────────────────────────────────────

#[test]
fn expected_tokens_stored_faithfully() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![10, 20, 30, 40, 50],
        Some(99),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(
        state.get_error_nodes()[0].expected,
        vec![10, 20, 30, 40, 50]
    );
    assert_eq!(state.get_error_nodes()[0].actual, Some(99));
}

#[test]
fn actual_none_for_missing_token() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
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
    assert_eq!(state.get_error_nodes()[0].actual, None);
}

// ── 7. Skipped token tracking ───────────────────────────────────────────────

#[test]
fn skipped_tokens_preserved() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![3, 4, 5, 6],
    );
    assert_eq!(state.get_error_nodes()[0].skipped_tokens, vec![3, 4, 5, 6]);
}

#[test]
fn empty_skipped_tokens() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        Some(2),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    assert!(state.get_error_nodes()[0].skipped_tokens.is_empty());
}

// ── 8. Recovery strategy selection ──────────────────────────────────────────

#[test]
fn strategy_prefers_insertion_when_insertable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let s = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenInsertion);
}

#[test]
fn strategy_deletion_for_clearly_wrong_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let s = state.determine_recovery_strategy(&[1, 2], Some(50), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenDeletion);
}

#[test]
fn strategy_substitution_single_expected() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .add_sync_token(50)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // actual=50 is sync → not deletable; single expected → substitution
    let s = state.determine_recovery_strategy(&[1], Some(50), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn strategy_phrase_level_fallback() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(50).build();
    let mut state = ErrorRecoveryState::new(config);
    let s = state.determine_recovery_strategy(&[1, 2, 3], Some(50), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_panic_when_all_disabled() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .add_sync_token(50)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let s = state.determine_recovery_strategy(&[1, 2, 3], Some(50), (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_forced_panic_after_max_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // Exhaust the limit
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    let s = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

// ── 9. Edge cases ───────────────────────────────────────────────────────────

#[test]
fn zero_length_error_at_start() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let n = &state.get_error_nodes()[0];
    assert_eq!(n.start_byte, 0);
    assert_eq!(n.end_byte, 0);
    assert!(n.expected.is_empty());
    assert_eq!(n.actual, None);
}

#[test]
fn error_at_large_offset() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    let big = usize::MAX - 1;
    state.record_error(
        big,
        big + 1,
        (999_999, 0),
        (999_999, 1),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let n = &state.get_error_nodes()[0];
    assert_eq!(n.start_byte, big);
    assert_eq!(n.end_byte, big + 1);
}

#[test]
fn empty_expected_list() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    state.record_error(
        5,
        10,
        (0, 5),
        (0, 10),
        vec![],
        Some(1),
        RecoveryStrategy::PhraseLevel,
        vec![],
    );
    assert!(state.get_error_nodes()[0].expected.is_empty());
}

// ── 10. State reset/clear ───────────────────────────────────────────────────

#[test]
fn clear_errors_then_record_again() {
    let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    record_one(&mut state, RecoveryStrategy::PanicMode);
    record_one(&mut state, RecoveryStrategy::TokenDeletion);
    assert_eq!(state.get_error_nodes().len(), 2);

    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());

    record_one(&mut state, RecoveryStrategy::PhraseLevel);
    assert_eq!(state.get_error_nodes().len(), 1);
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::PhraseLevel
    );
}

#[test]
fn reset_consecutive_errors_allows_recovery_again() {
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
fn default_config_matches_builder_defaults() {
    let d = ErrorRecoveryConfig::default();
    let b = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(d.max_panic_skip, b.max_panic_skip);
    assert_eq!(d.max_consecutive_errors, b.max_consecutive_errors);
    assert_eq!(d.enable_phrase_recovery, b.enable_phrase_recovery);
    assert_eq!(d.enable_scope_recovery, b.enable_scope_recovery);
    assert_eq!(d.enable_indentation_recovery, b.enable_indentation_recovery);
}
