#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for error recovery strategies in the adze runtime.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, ErrorNode,
    RecoveryStrategy,
};
use adze_ir::SymbolId;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_state() -> ErrorRecoveryState {
    ErrorRecoveryState::new(ErrorRecoveryConfig::default())
}

fn record(
    state: &mut ErrorRecoveryState,
    start_byte: usize,
    end_byte: usize,
    expected: Vec<u16>,
    actual: Option<u16>,
    recovery: RecoveryStrategy,
    skipped: Vec<u16>,
) {
    state.record_error(
        start_byte,
        end_byte,
        (0, start_byte),
        (0, end_byte),
        expected,
        actual,
        recovery,
        skipped,
    );
}

// =========================================================================
// 1. RecoveryStrategy variant tests
// =========================================================================

#[test]
fn strategy_panic_mode_equality() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_token_insertion_equality() {
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn strategy_token_deletion_equality() {
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn strategy_token_substitution_equality() {
    assert_eq!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn strategy_phrase_level_equality() {
    assert_eq!(RecoveryStrategy::PhraseLevel, RecoveryStrategy::PhraseLevel);
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
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(variants[i], variants[j], "variants {i} and {j} should differ");
        }
    }
}

// =========================================================================
// 2. Strategy Debug / Clone
// =========================================================================

#[test]
fn strategy_debug_format() {
    let dbg = format!("{:?}", RecoveryStrategy::PanicMode);
    assert!(dbg.contains("PanicMode"), "Debug should contain variant name");
}

#[test]
fn strategy_clone_preserves_value() {
    let orig = RecoveryStrategy::TokenSubstitution;
    let cloned = orig;
    assert_eq!(orig, cloned);
}

// =========================================================================
// 3. Error recording – basic
// =========================================================================

#[test]
fn record_single_error_and_retrieve() {
    let mut state = default_state();
    record(
        &mut state,
        0,
        5,
        vec![1, 2],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![99],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
    assert_eq!(nodes[0].expected, vec![1, 2]);
    assert_eq!(nodes[0].actual, Some(99));
    assert_eq!(nodes[0].recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn record_error_with_no_actual_token() {
    let mut state = default_state();
    record(
        &mut state,
        10,
        10,
        vec![3],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].actual, None);
}

#[test]
fn record_error_preserves_positions() {
    let mut state = default_state();
    state.record_error(5, 12, (1, 5), (1, 12), vec![7], Some(8), RecoveryStrategy::PanicMode, vec![]);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 5);
    assert_eq!(node.end_byte, 12);
}

// =========================================================================
// 4. Multiple errors in sequence
// =========================================================================

#[test]
fn record_multiple_errors_in_sequence() {
    let mut state = default_state();
    for i in 0..5 {
        record(
            &mut state,
            i * 10,
            i * 10 + 5,
            vec![i as u16],
            Some(100 + i as u16),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 5);
    for i in 0..5 {
        assert_eq!(nodes[i].start_byte, i * 10);
        assert_eq!(nodes[i].expected, vec![i as u16]);
    }
}

#[test]
fn errors_with_different_strategies() {
    let mut state = default_state();
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
    ];
    for (i, &strat) in strategies.iter().enumerate() {
        record(&mut state, i, i + 1, vec![1], Some(2), strat, vec![]);
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), strategies.len());
    for i in 0..strategies.len() {
        assert_eq!(nodes[i].recovery, strategies[i]);
    }
}

// =========================================================================
// 5. Error node retrieval
// =========================================================================

#[test]
fn get_error_nodes_returns_empty_initially() {
    let state = default_state();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn get_error_nodes_returns_clone() {
    let mut state = default_state();
    record(&mut state, 0, 1, vec![1], Some(2), RecoveryStrategy::PanicMode, vec![]);
    let first = state.get_error_nodes();
    let second = state.get_error_nodes();
    assert_eq!(first.len(), second.len());
    // Adding more errors should not affect previously returned vectors.
    record(&mut state, 2, 3, vec![3], Some(4), RecoveryStrategy::PanicMode, vec![]);
    assert_eq!(first.len(), 1);
    assert_eq!(state.get_error_nodes().len(), 2);
}

#[test]
fn clear_errors_removes_all() {
    let mut state = default_state();
    for i in 0..3 {
        record(&mut state, i, i + 1, vec![1], None, RecoveryStrategy::PanicMode, vec![]);
    }
    assert_eq!(state.get_error_nodes().len(), 3);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// =========================================================================
// 6. Edge cases – empty expected tokens
// =========================================================================

#[test]
fn record_error_with_empty_expected() {
    let mut state = default_state();
    record(&mut state, 0, 1, vec![], Some(5), RecoveryStrategy::PanicMode, vec![]);
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].expected.is_empty());
}

#[test]
fn record_error_with_large_expected_set() {
    let mut state = default_state();
    let expected: Vec<u16> = (0..256).collect();
    record(&mut state, 0, 10, expected.clone(), Some(999), RecoveryStrategy::PhraseLevel, vec![]);
    assert_eq!(state.get_error_nodes()[0].expected, expected);
}

// =========================================================================
// 7. Edge cases – zero-length spans
// =========================================================================

#[test]
fn record_error_zero_length_span() {
    let mut state = default_state();
    record(&mut state, 42, 42, vec![1], None, RecoveryStrategy::TokenInsertion, vec![]);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 42);
    assert_eq!(node.end_byte, 42);
}

// =========================================================================
// 8. Edge cases – overlapping errors
// =========================================================================

#[test]
fn record_overlapping_errors() {
    let mut state = default_state();
    record(&mut state, 0, 10, vec![1], Some(2), RecoveryStrategy::TokenDeletion, vec![]);
    record(&mut state, 5, 15, vec![3], Some(4), RecoveryStrategy::TokenSubstitution, vec![]);
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 2);
    // The overlapping region [5,10) is covered by both.
    assert_eq!(nodes[0].end_byte, 10);
    assert_eq!(nodes[1].start_byte, 5);
}

// =========================================================================
// 9. Skipped tokens tracking
// =========================================================================

#[test]
fn record_error_with_skipped_tokens() {
    let mut state = default_state();
    record(
        &mut state,
        0,
        20,
        vec![1, 2],
        Some(50),
        RecoveryStrategy::PanicMode,
        vec![50, 51, 52],
    );
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.skipped_tokens, vec![50, 51, 52]);
}

#[test]
fn record_error_with_empty_skipped_tokens() {
    let mut state = default_state();
    record(&mut state, 0, 1, vec![1], Some(2), RecoveryStrategy::TokenDeletion, vec![]);
    assert!(state.get_error_nodes()[0].skipped_tokens.is_empty());
}

// =========================================================================
// 10. Consecutive error tracking and give-up
// =========================================================================

#[test]
fn should_give_up_respects_threshold() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn reset_error_count_clears_consecutive() {
    let mut state = default_state();
    for _ in 0..5 {
        state.increment_error_count();
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

// =========================================================================
// 11. determine_recovery_strategy
// =========================================================================

#[test]
fn determine_strategy_insertion_when_insertable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strat = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenInsertion);
}

#[test]
fn determine_strategy_deletion_for_wrong_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // actual=50 is not in expected=[10,20] and not a sync token → deletion
    let strat = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenDeletion);
}

#[test]
fn determine_strategy_substitution_single_expected() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(50) // make 50 a sync token so deletion is skipped
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // Exactly one expected token + actual is sync → substitution
    let strat = state.determine_recovery_strategy(&[10], Some(50), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn determine_strategy_panic_after_max_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // Consume up to max
    state.determine_recovery_strategy(&[], Some(1), (0, 0), 0);
    state.determine_recovery_strategy(&[], Some(1), (0, 0), 0);
    // Next should be PanicMode
    let strat = state.determine_recovery_strategy(&[], Some(1), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PanicMode);
}

#[test]
fn determine_strategy_phrase_level_fallback() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(50) // keep actual as sync to avoid deletion
        .enable_phrase_recovery(true)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // Multiple expected, no insertable, sync actual → phrase level
    let strat = state.determine_recovery_strategy(&[10, 20], Some(50), (0, 0), 0);
    assert_eq!(strat, RecoveryStrategy::PhraseLevel);
}

// =========================================================================
// 12. ErrorNode struct field access
// =========================================================================

#[test]
fn error_node_debug_format() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![],
    };
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("ErrorNode"));
    assert!(dbg.contains("TokenDeletion"));
}

#[test]
fn error_node_clone_is_independent() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1, 2, 3],
        actual: Some(99),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![4, 5],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, node.start_byte);
    assert_eq!(cloned.expected, node.expected);
    assert_eq!(cloned.recovery, node.recovery);
    assert_eq!(cloned.skipped_tokens, node.skipped_tokens);
}

// =========================================================================
// 13. Config builder chaining
// =========================================================================

#[test]
fn config_builder_full_chain() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(3)
        .add_deletable_token(4)
        .add_scope_delimiter(10, 11)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .max_consecutive_errors(5)
        .build();

    assert_eq!(config.max_panic_skip, 200);
    assert!(config.sync_tokens.iter().any(|s| s.0 == 1));
    assert!(config.sync_tokens.iter().any(|s| s.0 == 2));
    assert!(config.insert_candidates.iter().any(|s| s.0 == 3));
    assert!(config.deletable_tokens.contains(&4));
    assert_eq!(config.scope_delimiters, vec![(10, 11)]);
    assert!(!config.enable_phrase_recovery);
    assert!(!config.enable_scope_recovery);
    assert!(config.enable_indentation_recovery);
    assert_eq!(config.max_consecutive_errors, 5);
}
