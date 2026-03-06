//! Comprehensive tests for ErrorRecoveryState and error recovery functionality.
//!
//! Tests cover:
//! - State creation and default values
//! - Recording errors with various RecoveryStrategy variants
//! - Error node position tracking and details
//! - Multiple error recording and ordering
//! - State reset and clear operations
//! - Error node retrieval and counting
//! - Edge cases and stress tests

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};

// ============================================================================
// 1. ErrorRecoveryState Creation Tests
// ============================================================================

#[test]
fn test_error_recovery_state_default_creation() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);

    assert!(!state.should_give_up());
    assert_eq!(state.get_error_nodes().len(), 0);
}

#[test]
fn test_error_recovery_state_with_custom_config() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .max_consecutive_errors(5)
        .build();

    let state = ErrorRecoveryState::new(config);
    assert!(!state.should_give_up());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_error_recovery_state_empty_error_nodes() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 0);
    assert!(errors.is_empty());
}

// ============================================================================
// 2. Recording Errors with Each RecoveryStrategy Variant
// ============================================================================

#[test]
fn test_record_error_panic_mode() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        10,
        20,
        (0, 10),
        (0, 20),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![3],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::PanicMode);
}

#[test]
fn test_record_error_token_insertion() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        5,
        5,
        (0, 5),
        (0, 5),
        vec![10],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::TokenInsertion);
    assert_eq!(errors[0].actual, None);
}

#[test]
fn test_record_error_token_deletion() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1, 2, 3],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![99],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::TokenDeletion);
}

#[test]
fn test_record_error_token_substitution() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        5,
        15,
        (1, 0),
        (1, 10),
        vec![42],
        Some(99),
        RecoveryStrategy::TokenSubstitution,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::TokenSubstitution);
    assert_eq!(errors[0].actual, Some(99));
}

#[test]
fn test_record_error_phrase_level() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        20,
        50,
        (2, 0),
        (2, 30),
        vec![100, 101],
        Some(200),
        RecoveryStrategy::PhraseLevel,
        vec![200, 201],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::PhraseLevel);
}

#[test]
fn test_record_error_scope_recovery() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        100,
        110,
        (5, 0),
        (5, 10),
        vec![7, 8],
        Some(9),
        RecoveryStrategy::ScopeRecovery,
        vec![9],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn test_record_error_indentation_recovery() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        15,
        25,
        (1, 5),
        (1, 15),
        vec![11, 12, 13],
        Some(14),
        RecoveryStrategy::IndentationRecovery,
        vec![14, 15],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::IndentationRecovery);
}

// ============================================================================
// 3. Error Node Position Tracking
// ============================================================================

#[test]
fn test_error_position_tracking_byte_positions() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        42,
        100,
        (0, 0),
        (0, 0),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_byte, 42);
    assert_eq!(errors[0].end_byte, 100);
}

#[test]
fn test_error_position_tracking_line_col_positions() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        5,
        (3, 7),
        (3, 12),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_position, (3, 7));
    assert_eq!(errors[0].end_position, (3, 12));
}

#[test]
fn test_error_position_multiline_tracking() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        100,
        250,
        (5, 10),
        (8, 5),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_position.0, 5);
    assert_eq!(errors[0].end_position.0, 8);
}

#[test]
fn test_error_expected_symbols_tracking() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let expected = vec![10, 20, 30];
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        expected.clone(),
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].expected, expected);
}

#[test]
fn test_error_actual_symbol_tracking() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1, 2],
        Some(42),
        RecoveryStrategy::TokenSubstitution,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].actual, Some(42));
}

#[test]
fn test_error_skipped_tokens_tracking() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let skipped = vec![5, 10, 15, 20];
    state.record_error(
        0,
        40,
        (0, 0),
        (0, 40),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        skipped.clone(),
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].skipped_tokens, skipped);
}

// ============================================================================
// 4. Multiple Error Recording
// ============================================================================

#[test]
fn test_record_multiple_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![2],
    );

    state.record_error(
        20,
        30,
        (1, 0),
        (1, 10),
        vec![3],
        Some(4),
        RecoveryStrategy::TokenDeletion,
        vec![4],
    );

    state.record_error(
        40,
        50,
        (2, 0),
        (2, 10),
        vec![5],
        Some(6),
        RecoveryStrategy::TokenInsertion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 3);
}

#[test]
fn test_record_many_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    const NUM_ERRORS: usize = 50;
    for i in 0..NUM_ERRORS {
        let start_byte = i * 20;
        let end_byte = start_byte + 10;
        state.record_error(
            start_byte,
            end_byte,
            (i, 0),
            (i, 10),
            vec![1, 2],
            Some(99),
            RecoveryStrategy::PanicMode,
            vec![99],
        );
    }

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), NUM_ERRORS);
}

// ============================================================================
// 5. Error State Clear/Reset
// ============================================================================

#[test]
fn test_clear_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    assert_eq!(state.get_error_nodes().len(), 1);

    state.clear_errors();
    assert_eq!(state.get_error_nodes().len(), 0);
}

#[test]
fn test_clear_errors_multiple_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    for i in 0..10 {
        state.record_error(
            i * 10,
            (i + 1) * 10,
            (i, 0),
            (i, 10),
            vec![1],
            Some(2),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }

    assert_eq!(state.get_error_nodes().len(), 10);
    state.clear_errors();
    assert_eq!(state.get_error_nodes().len(), 0);
}

#[test]
fn test_reset_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());

    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn test_reset_error_count_method() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

// ============================================================================
// 6. get_error_nodes() Returns Correct Count and Details
// ============================================================================

#[test]
fn test_get_error_nodes_empty() {
    let config = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(config);

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 0);
}

#[test]
fn test_get_error_nodes_single_error() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        5,
        15,
        (0, 5),
        (0, 15),
        vec![1, 2, 3],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![99],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].start_byte, 5);
    assert_eq!(errors[0].end_byte, 15);
    assert_eq!(errors[0].expected, vec![1, 2, 3]);
    assert_eq!(errors[0].actual, Some(99));
}

#[test]
fn test_get_error_nodes_multiple_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    for i in 0..5 {
        state.record_error(
            i * 10,
            (i + 1) * 10,
            (i, 0),
            (i, 10),
            vec![i as u16],
            Some((i + 100) as u16),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 5);

    // Verify each error
    for (idx, error) in errors.iter().enumerate() {
        assert_eq!(error.start_byte, idx * 10);
        assert_eq!(error.end_byte, (idx + 1) * 10);
    }
}

#[test]
fn test_get_error_nodes_preserves_details() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let expected = vec![10, 20, 30];
    let skipped = vec![40, 50, 60];

    state.record_error(
        100,
        200,
        (2, 5),
        (2, 105),
        expected.clone(),
        Some(99),
        RecoveryStrategy::TokenSubstitution,
        skipped.clone(),
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_byte, 100);
    assert_eq!(errors[0].end_byte, 200);
    assert_eq!(errors[0].start_position, (2, 5));
    assert_eq!(errors[0].end_position, (2, 105));
    assert_eq!(errors[0].expected, expected);
    assert_eq!(errors[0].actual, Some(99));
    assert_eq!(errors[0].recovery, RecoveryStrategy::TokenSubstitution);
    assert_eq!(errors[0].skipped_tokens, skipped);
}

// ============================================================================
// 7. Error Ordering (by position)
// ============================================================================

#[test]
fn test_error_ordering_by_start_byte() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // Record in non-sequential order
    state.record_error(
        100,
        110,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        10,
        20,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        50,
        60,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    // Errors should be returned in order they were added (not sorted)
    assert_eq!(errors[0].start_byte, 100);
    assert_eq!(errors[1].start_byte, 10);
    assert_eq!(errors[2].start_byte, 50);
}

#[test]
fn test_error_ordering_sequential() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // Record in sequential order
    for i in 0..10 {
        state.record_error(
            i * 100,
            i * 100 + 10,
            (i, 0),
            (i, 10),
            vec![1],
            Some(2),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }

    let errors = state.get_error_nodes();
    for (idx, error) in errors.iter().enumerate() {
        assert_eq!(error.start_byte, idx * 100);
    }
}

// ============================================================================
// 8. Edge Cases
// ============================================================================

#[test]
fn test_edge_case_zero_length_error() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // Error with start_byte == end_byte
    state.record_error(
        50,
        50,
        (0, 50),
        (0, 50),
        vec![1, 2],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].start_byte, 50);
    assert_eq!(errors[0].end_byte, 50);
}

#[test]
fn test_edge_case_overlapping_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // First error: 0-30
    state.record_error(
        0,
        30,
        (0, 0),
        (0, 30),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    // Second error: 20-50 (overlaps)
    state.record_error(
        20,
        50,
        (0, 20),
        (0, 50),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 2);
}

#[test]
fn test_edge_case_max_position_values() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let max_byte = usize::MAX / 2; // Use large but not max to avoid overflow
    let max_line = 1000000;
    let max_col = 1000000;

    state.record_error(
        max_byte - 100,
        max_byte,
        (max_line, max_col),
        (max_line + 1, 0),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].start_byte, max_byte - 100);
    assert_eq!(errors[0].end_byte, max_byte);
}

#[test]
fn test_edge_case_empty_expected_symbols() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].expected.len(), 0);
}

#[test]
fn test_edge_case_none_actual_symbol() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1, 2, 3],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].actual, None);
}

#[test]
fn test_edge_case_large_expected_list() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let mut expected = Vec::new();
    for i in 0..1000 {
        expected.push(i as u16);
    }

    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        expected.clone(),
        Some(9999),
        RecoveryStrategy::TokenSubstitution,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].expected.len(), 1000);
    assert_eq!(errors[0].expected, expected);
}

#[test]
fn test_edge_case_large_skipped_tokens() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let mut skipped = Vec::new();
    for i in 0..500 {
        skipped.push(i as u16);
    }

    state.record_error(
        0,
        5000,
        (0, 0),
        (0, 5000),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        skipped.clone(),
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].skipped_tokens.len(), 500);
}

// ============================================================================
// 9. RecoveryStrategy Display and Comparison
// ============================================================================

#[test]
fn test_recovery_strategy_equality() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn test_recovery_strategy_all_variants_distinct() {
    let panic = RecoveryStrategy::PanicMode;
    let insert = RecoveryStrategy::TokenInsertion;
    let delete = RecoveryStrategy::TokenDeletion;
    let subst = RecoveryStrategy::TokenSubstitution;
    let phrase = RecoveryStrategy::PhraseLevel;
    let scope = RecoveryStrategy::ScopeRecovery;
    let indent = RecoveryStrategy::IndentationRecovery;

    assert_ne!(panic, insert);
    assert_ne!(insert, delete);
    assert_ne!(delete, subst);
    assert_ne!(subst, phrase);
    assert_ne!(phrase, scope);
    assert_ne!(scope, indent);
}

#[test]
fn test_recovery_strategy_copy_clone() {
    let original = RecoveryStrategy::TokenDeletion;
    let copied = original;
    let cloned = original;

    assert_eq!(copied, original);
    assert_eq!(cloned, original);
}

#[test]
fn test_recovery_strategy_debug_format() {
    let strategies = vec![
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];

    for strategy in strategies {
        let debug_str = format!("{:?}", strategy);
        assert!(!debug_str.is_empty());
    }
}

// ============================================================================
// 10. Error at Start/End of Input
// ============================================================================

#[test]
fn test_error_at_start_of_input() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1, 2],
        Some(99),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_byte, 0);
}

#[test]
fn test_error_at_end_of_input() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let large_offset = 1_000_000;
    state.record_error(
        large_offset,
        large_offset + 10,
        (100, 0),
        (100, 10),
        vec![1, 2],
        Some(99),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_byte, large_offset);
    assert_eq!(errors[0].end_byte, large_offset + 10);
}

#[test]
fn test_error_at_start_line() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![],
        Some(1),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_position.0, 0);
}

#[test]
fn test_error_at_line_boundary() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        500,
        510,
        (10, 0),
        (11, 0),
        vec![],
        Some(1),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].start_position.0, 10);
    assert_eq!(errors[0].end_position.0, 11);
}

// ============================================================================
// 11. Stress Tests - Many Errors
// ============================================================================

#[test]
fn test_stress_many_errors_1000() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    const NUM_ERRORS: usize = 1000;
    for i in 0..NUM_ERRORS {
        state.record_error(
            i * 100,
            i * 100 + 50,
            (i / 100, i % 100),
            (i / 100, (i % 100) + 50),
            vec![(i as u16) % 256],
            Some((i as u16) + 256),
            match i % 7 {
                0 => RecoveryStrategy::PanicMode,
                1 => RecoveryStrategy::TokenInsertion,
                2 => RecoveryStrategy::TokenDeletion,
                3 => RecoveryStrategy::TokenSubstitution,
                4 => RecoveryStrategy::PhraseLevel,
                5 => RecoveryStrategy::ScopeRecovery,
                _ => RecoveryStrategy::IndentationRecovery,
            },
            vec![],
        );
    }

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), NUM_ERRORS);

    // Verify all errors are distinct
    for (i, error) in errors.iter().enumerate() {
        assert_eq!(error.start_byte, i * 100);
    }
}

#[test]
fn test_stress_clear_and_refill() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // First fill
    for i in 0..100 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (i, 0),
            (i, 5),
            vec![1],
            Some(2),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 100);

    // Clear
    state.clear_errors();
    assert_eq!(state.get_error_nodes().len(), 0);

    // Refill
    for i in 0..50 {
        state.record_error(
            i * 20,
            i * 20 + 10,
            (i, 0),
            (i, 10),
            vec![1],
            Some(2),
            RecoveryStrategy::TokenDeletion,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 50);
}

#[test]
fn test_stress_varying_error_sizes() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // Vary the expected symbols and skipped tokens
    for i in 0..100 {
        let mut expected = Vec::new();
        let mut skipped = Vec::new();

        for j in 0..=(i % 20) {
            expected.push(j as u16);
            if j % 2 == 0 {
                skipped.push(j as u16);
            }
        }

        state.record_error(
            i * 50,
            i * 50 + 25,
            (i / 10, i % 10),
            (i / 10, (i % 10) + 25),
            expected,
            Some(99),
            RecoveryStrategy::PanicMode,
            skipped,
        );
    }

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 100);
}

// ============================================================================
// 12. Configuration and Consecutive Error Count
// ============================================================================

#[test]
fn test_increment_error_count() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    assert!(!state.should_give_up());

    state.increment_error_count();
    assert!(!state.should_give_up()); // 1 < 10

    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up()); // 3 < 10
}

#[test]
fn test_should_give_up_threshold() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);

    assert!(!state.should_give_up());

    // Fill up to 4 errors (just below threshold)
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());

    // Add one more to reach threshold
    state.increment_error_count();
    assert!(state.should_give_up());

    // Add more
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn test_custom_max_consecutive_errors_config() {
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

// ============================================================================
// 13. Error Node Cloning and Independence
// ============================================================================

#[test]
fn test_error_node_cloning() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        10,
        20,
        (1, 5),
        (1, 15),
        vec![1, 2, 3],
        Some(99),
        RecoveryStrategy::TokenSubstitution,
        vec![99],
    );

    let errors1 = state.get_error_nodes();
    let errors2 = state.get_error_nodes();

    assert_eq!(errors1.len(), errors2.len());
    assert_eq!(errors1[0].start_byte, errors2[0].start_byte);
}

#[test]
fn test_error_nodes_independent_snapshots() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );

    let errors_before = state.get_error_nodes();
    assert_eq!(errors_before.len(), 1);

    // Add more errors
    state.record_error(
        20,
        30,
        (1, 0),
        (1, 10),
        vec![3],
        Some(4),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );

    let errors_after = state.get_error_nodes();
    assert_eq!(errors_after.len(), 2);

    // Original snapshot should still have 1 error
    assert_eq!(errors_before.len(), 1);
}

// ============================================================================
// 14. Config Builder Combinations
// ============================================================================

#[test]
fn test_config_builder_chaining() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(75)
        .add_insertable_token(10)
        .add_insertable_token(20)
        .max_consecutive_errors(7)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();

    assert_eq!(config.max_panic_skip, 75);
    assert_eq!(config.max_consecutive_errors, 7);
    assert!(!config.enable_phrase_recovery);
    assert!(!config.enable_scope_recovery);
}

#[test]
fn test_config_default_vs_custom() {
    let default_config = ErrorRecoveryConfig::default();
    let custom_config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .max_consecutive_errors(15)
        .build();

    assert_eq!(default_config.max_panic_skip, 50);
    assert_eq!(custom_config.max_panic_skip, 100);

    assert_eq!(default_config.max_consecutive_errors, 10);
    assert_eq!(custom_config.max_consecutive_errors, 15);
}

// ============================================================================
// 15. Comprehensive Scenario Tests
// ============================================================================

#[test]
fn test_realistic_error_recovery_scenario() {
    // Simulate a realistic parsing scenario with multiple errors of different types
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(50)
        .add_insertable_token(10)
        .add_insertable_token(11)
        .max_consecutive_errors(10)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .add_scope_delimiter(1, 2) // { }
        .add_scope_delimiter(3, 4) // ( )
        .build();

    let mut state = ErrorRecoveryState::new(config);

    // Simulate parsing with various error recovery strategies
    state.record_error(
        10,
        15,
        (0, 10),
        (0, 15),
        vec![10, 11],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );

    state.record_error(
        30,
        40,
        (1, 0),
        (1, 10),
        vec![20],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![99],
    );

    state.record_error(
        60,
        70,
        (2, 5),
        (2, 15),
        vec![30, 31],
        Some(100),
        RecoveryStrategy::TokenSubstitution,
        vec![],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 3);
    assert_eq!(errors[0].recovery, RecoveryStrategy::TokenInsertion);
    assert_eq!(errors[1].recovery, RecoveryStrategy::TokenDeletion);
    assert_eq!(errors[2].recovery, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn test_full_error_lifecycle() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // Start clean
    assert_eq!(state.get_error_nodes().len(), 0);
    assert!(!state.should_give_up());

    // Record errors
    for i in 0..5 {
        state.record_error(
            i * 100,
            i * 100 + 50,
            (i, 0),
            (i, 50),
            vec![1, 2],
            Some(99),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }

    assert_eq!(state.get_error_nodes().len(), 5);

    // Increment consecutive errors
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());

    // Reset consecutive errors
    state.reset_error_count();
    assert!(!state.should_give_up());

    // Clear all errors
    state.clear_errors();
    assert_eq!(state.get_error_nodes().len(), 0);
}

// ============================================================================
// 16. Additional Comprehensive Tests
// ============================================================================

#[test]
fn test_error_recovery_state_with_all_strategy_types() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];

    for (idx, strategy) in strategies.iter().enumerate() {
        state.record_error(
            idx * 10,
            idx * 10 + 5,
            (0, 0),
            (0, 5),
            vec![1],
            Some(2),
            *strategy,
            vec![],
        );
    }

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 7);

    for (idx, error) in errors.iter().enumerate() {
        assert_eq!(error.recovery, strategies[idx]);
    }
}

#[test]
fn test_consecutive_errors_increment_and_threshold() {
    let config = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(config);

    for i in 0..5 {
        state.increment_error_count();
        // i is 0-based so after increment we're at 1..6
        if i < 4 {
            assert!(!state.should_give_up());
        } else {
            // i=4, after increment we're at 5, so should give up
            assert!(state.should_give_up());
        }
    }
}

#[test]
fn test_multiple_clear_and_record_cycles() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    for cycle in 0..5 {
        // Record errors
        for i in 0..3 {
            state.record_error(
                i * 10,
                i * 10 + 5,
                (cycle * 3 + i, 0),
                (cycle * 3 + i, 5),
                vec![1],
                Some(2),
                RecoveryStrategy::PanicMode,
                vec![],
            );
        }

        assert_eq!(state.get_error_nodes().len(), 3);

        // Clear errors
        state.clear_errors();
        assert_eq!(state.get_error_nodes().len(), 0);
    }
}

#[test]
fn test_complex_position_tracking() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // Record errors with various positions
    let positions = [
        (0, 0, (0, 0), (0, 10)),
        (10, 20, (1, 0), (1, 10)),
        (30, 50, (2, 5), (2, 25)),
        (100, 150, (5, 10), (6, 5)),
    ];

    for (idx, (start_byte, end_byte, start_pos, end_pos)) in positions.iter().enumerate() {
        state.record_error(
            *start_byte,
            *end_byte,
            *start_pos,
            *end_pos,
            vec![idx as u16],
            Some((idx + 100) as u16),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }

    let errors = state.get_error_nodes();
    for (idx, error) in errors.iter().enumerate() {
        assert_eq!(error.start_byte, positions[idx].0);
        assert_eq!(error.end_byte, positions[idx].1);
        assert_eq!(error.start_position, positions[idx].2);
        assert_eq!(error.end_position, positions[idx].3);
    }
}

#[test]
fn test_error_recovery_with_empty_collections() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    // Record error with all empty collections
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

    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].expected.is_empty());
    assert_eq!(errors[0].actual, None);
    assert!(errors[0].skipped_tokens.is_empty());
}

#[test]
fn test_error_nodes_with_max_u16_values() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);

    state.record_error(
        0,
        100,
        (0, 0),
        (0, 100),
        vec![u16::MAX, u16::MAX - 1],
        Some(u16::MAX),
        RecoveryStrategy::TokenDeletion,
        vec![u16::MAX],
    );

    let errors = state.get_error_nodes();
    assert_eq!(errors[0].expected[0], u16::MAX);
    assert_eq!(errors[0].actual, Some(u16::MAX));
}
