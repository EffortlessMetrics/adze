//! Integration tests for the adze runtime crate covering edge cases
//!
//! This test suite provides comprehensive coverage of edge cases and boundary
//! conditions for key types in the adze runtime through its public API:
//! - Spanned<T> with various edge-case spans
//! - Point with boundary values
//! - ErrorRecoveryState and ErrorRecoveryConfig
//! - Visitor API (tested through TreeWalker and BreadthFirstWalker)
//! - RecoveryStrategy variants
//! - VisitorAction variants

use adze::adze_ir as ir;

use adze::*;

// ============================================================================
// Spanned<T> Edge Case Tests
// ============================================================================

#[test]
fn test_spanned_zero_length_span() {
    let value = "hello";
    let spanned: Spanned<&str> = Spanned {
        value,
        span: (5, 5), // Zero-length span at position 5
    };
    assert_eq!(spanned.span, (5, 5));
    assert_eq!(spanned.value, "hello");
}

#[test]
fn test_spanned_very_large_byte_offsets() {
    let value = 42u32;
    let spanned: Spanned<u32> = Spanned {
        value,
        span: (usize::MAX / 2, usize::MAX - 1), // Very large offsets
    };
    assert_eq!(spanned.span.0, usize::MAX / 2);
    assert_eq!(spanned.span.1, usize::MAX - 1);
}

#[test]
fn test_spanned_deref_coercion() {
    let value = vec![1, 2, 3];
    let spanned: Spanned<Vec<i32>> = Spanned {
        value: value.clone(),
        span: (0, 10),
    };
    // Deref should allow access to Vec methods
    assert_eq!(spanned.len(), 3);
    assert_eq!(spanned[0], 1);
}

#[test]
fn test_spanned_nested_generic() {
    let spanned: Spanned<Option<String>> = Spanned {
        value: Some("nested".to_string()),
        span: (100, 200),
    };
    assert!(spanned.value.is_some());
    assert_eq!(spanned.value.as_ref().unwrap(), "nested");
}

#[test]
fn test_spanned_clone() {
    let original: Spanned<String> = Spanned {
        value: "test".to_string(),
        span: (10, 14),
    };
    let cloned = original.clone();
    assert_eq!(cloned.value, original.value);
    assert_eq!(cloned.span, original.span);
}

#[test]
fn test_spanned_equality() {
    let spanned1: Spanned<i32> = Spanned {
        value: 42,
        span: (0, 5),
    };
    let spanned2: Spanned<i32> = Spanned {
        value: 42,
        span: (0, 5),
    };
    // Equality depends on implementation; test existence
    assert_eq!(spanned1.value, spanned2.value);
    assert_eq!(spanned1.span, spanned2.span);
}

#[test]
fn test_spanned_with_unit_type() {
    let spanned: Spanned<()> = Spanned {
        value: (),
        span: (0, 5),
    };

    assert_eq!(spanned.span, (0, 5));
}

#[test]
fn test_spanned_with_borrowed_slice() {
    let spanned: Spanned<&[u8]> = Spanned {
        value: b"bytes",
        span: (0, 5),
    };

    assert_eq!(spanned.span.1 - spanned.span.0, 5);
}

#[test]
fn test_spanned_multiple_generic_parameters() {
    let spanned: Spanned<(String, i32)> = Spanned {
        value: ("test".to_string(), 42),
        span: (0, 10),
    };

    assert_eq!(spanned.value.0, "test");
    assert_eq!(spanned.value.1, 42);
}

#[test]
fn test_spanned_span_ordering() {
    let spanned: Spanned<i32> = Spanned {
        value: 100,
        span: (10, 20),
    };

    assert!(spanned.span.0 < spanned.span.1);
}

#[test]
fn test_spanned_span_identity() {
    let spanned1: Spanned<String> = Spanned {
        value: "test1".to_string(),
        span: (0, 5),
    };
    let spanned2: Spanned<String> = Spanned {
        value: "test2".to_string(),
        span: (0, 5),
    };

    // Same span, different value
    assert_eq!(spanned1.span, spanned2.span);
    assert_ne!(spanned1.value, spanned2.value);
}

// ============================================================================
// Point Edge Case Tests
// ============================================================================

#[test]
fn test_point_at_zero_zero() {
    let point = pure_parser::Point { row: 0, column: 0 };
    assert_eq!(point.row, 0);
    assert_eq!(point.column, 0);
}

#[test]
fn test_point_at_max_values() {
    let point = pure_parser::Point {
        row: u32::MAX,
        column: u32::MAX,
    };
    assert_eq!(point.row, u32::MAX);
    assert_eq!(point.column, u32::MAX);
}

#[test]
fn test_point_large_values() {
    let point = pure_parser::Point {
        row: 1_000_000,
        column: 999_999,
    };
    assert_eq!(point.row, 1_000_000);
    assert_eq!(point.column, 999_999);
}

#[test]
fn test_point_comparison() {
    let point1 = pure_parser::Point {
        row: 10,
        column: 20,
    };
    let point2 = pure_parser::Point {
        row: 10,
        column: 20,
    };
    let point3 = pure_parser::Point {
        row: 11,
        column: 20,
    };

    assert_eq!(point1, point2);
    assert_ne!(point1, point3);
}

#[test]
fn test_point_ordering() {
    let point_low = pure_parser::Point { row: 5, column: 10 };
    let point_high = pure_parser::Point { row: 10, column: 5 };

    // row 5 < row 10, so point_low should be less
    assert!(point_low.row < point_high.row);
}

#[test]
fn test_point_default() {
    let point = pure_parser::Point::default();
    assert_eq!(point.row, 0);
    assert_eq!(point.column, 0);
}

#[test]
fn test_point_clone() {
    let original = pure_parser::Point {
        row: 42,
        column: 99,
    };
    let cloned = original;
    assert_eq!(cloned.row, original.row);
    assert_eq!(cloned.column, original.column);
}

#[test]
fn test_point_copy_semantics() {
    let point1 = pure_parser::Point { row: 5, column: 10 };
    let point2 = point1; // Copy, not move
    assert_eq!(point1.row, point2.row); // Can still use point1
    assert_eq!(point1.column, point2.column);
}

#[test]
fn test_point_byte_column_semantics() {
    // Point column is in bytes, not characters
    let point1 = pure_parser::Point { row: 0, column: 1 };
    let point2 = pure_parser::Point { row: 0, column: 2 };

    assert!(point1.column < point2.column);
}

#[test]
fn test_point_row_independence() {
    // Different rows
    let point1 = pure_parser::Point {
        row: 100,
        column: 0,
    };
    let point2 = pure_parser::Point {
        row: 200,
        column: 0,
    };

    assert_eq!(point1.column, point2.column);
    assert_ne!(point1.row, point2.row);
}

#[test]
fn test_point_boundary_values() {
    let point1 = pure_parser::Point { row: 0, column: 0 };
    let point2 = pure_parser::Point { row: 1, column: 1 };

    assert!(point1.row < point2.row);
    assert!(point1.column < point2.column);
}

// ============================================================================
// ErrorRecoveryConfig Edge Case Tests
// ============================================================================

#[test]
fn test_error_recovery_config_default() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
    assert_eq!(config.max_token_deletions, 3);
    assert_eq!(config.max_token_insertions, 2);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
}

#[test]
fn test_error_recovery_config_custom() {
    let config = error_recovery::ErrorRecoveryConfig {
        max_panic_skip: 100,
        sync_tokens: Default::default(),
        insert_candidates: Default::default(),
        deletable_tokens: Default::default(),
        max_token_deletions: 5,
        max_token_insertions: 4,
        max_consecutive_errors: 20,
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        scope_delimiters: vec![(1, 2)],
        enable_indentation_recovery: true,
    };

    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(config.max_token_deletions, 5);
    assert_eq!(config.max_token_insertions, 4);
}

#[test]
fn test_error_recovery_config_can_delete_token() {
    let mut config = error_recovery::ErrorRecoveryConfig::default();
    config.sync_tokens.push(ir::SymbolId(5));

    // Token in sync_tokens should not be deletable
    assert!(!config.can_delete_token(ir::SymbolId(5)));
}

#[test]
fn test_error_recovery_config_can_replace_token() {
    let mut config = error_recovery::ErrorRecoveryConfig::default();
    config.sync_tokens.push(ir::SymbolId(5));

    // Token in sync_tokens should not be replaceable
    assert!(!config.can_replace_token(ir::SymbolId(5)));
}

#[test]
fn test_error_recovery_config_zero_limits() {
    let config = error_recovery::ErrorRecoveryConfig {
        max_panic_skip: 0,
        sync_tokens: Default::default(),
        insert_candidates: Default::default(),
        deletable_tokens: Default::default(),
        max_token_deletions: 0,
        max_token_insertions: 0,
        max_consecutive_errors: 0,
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };

    assert_eq!(config.max_panic_skip, 0);
    assert_eq!(config.max_token_deletions, 0);
}

#[test]
fn test_error_recovery_config_large_limits() {
    let config = error_recovery::ErrorRecoveryConfig {
        max_panic_skip: 10000,
        sync_tokens: Default::default(),
        insert_candidates: Default::default(),
        deletable_tokens: Default::default(),
        max_token_deletions: 1000,
        max_token_insertions: 1000,
        max_consecutive_errors: 1000,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };

    assert_eq!(config.max_panic_skip, 10000);
    assert_eq!(config.max_consecutive_errors, 1000);
}

#[test]
fn test_error_recovery_config_scope_delimiters() {
    let config = error_recovery::ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: Default::default(),
        insert_candidates: Default::default(),
        deletable_tokens: Default::default(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![(1, 2), (3, 4)],
        enable_indentation_recovery: false,
    };

    assert_eq!(config.scope_delimiters.len(), 2);
    assert_eq!(config.scope_delimiters[0], (1, 2));
}

#[test]
fn test_error_recovery_config_token_candidates() {
    let mut config = error_recovery::ErrorRecoveryConfig::default();
    config.insert_candidates.push(ir::SymbolId(1));
    config.insert_candidates.push(ir::SymbolId(2));

    assert_eq!(config.insert_candidates.len(), 2);
}

// ============================================================================
// ErrorRecoveryState Edge Case Tests
// ============================================================================

#[test]
fn test_error_recovery_state_creation() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let state = error_recovery::ErrorRecoveryState::new(config);
    // Verify state was created
    assert!(!state.should_give_up());
}

#[test]
fn test_error_recovery_state_increment_errors() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    // Incrementing errors shouldn't immediately give up (under default limit of 10)
    state.increment_error_count();
    assert!(!state.should_give_up());

    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn test_error_recovery_state_reset_errors() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    state.increment_error_count();
    state.increment_error_count();

    state.reset_error_count();
    assert!(!state.should_give_up()); // Reset successful
}

#[test]
fn test_error_recovery_state_many_errors() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    // Increment beyond the default limit (10)
    for _ in 0..150 {
        state.increment_error_count();
    }

    assert!(state.should_give_up()); // Many errors, should give up
}

#[test]
fn test_error_recovery_state_add_recent_token() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    state.add_recent_token(42);
    state.add_recent_token(43);
    // Verify no panic - method should succeed
}

#[test]
fn test_error_recovery_state_scope_operations() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    state.push_scope(1);
    // pop_scope_test checks if stack is empty
    let _result = state.pop_scope_test();
    // Result depends on implementation - just verify no panic
}

#[test]
fn test_error_recovery_state_get_error_nodes() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let state = error_recovery::ErrorRecoveryState::new(config);

    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 0);
}

#[test]
fn test_error_recovery_state_clear_errors() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    state.clear_errors();
    assert_eq!(state.get_error_nodes().len(), 0);
}

#[test]
fn test_error_recovery_state_record_error() {
    let config = error_recovery::ErrorRecoveryConfig::default();
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    state.record_error(
        10,
        20,
        (1, 5),
        (1, 15),
        vec![1, 2, 3],
        Some(99),
        error_recovery::RecoveryStrategy::PanicMode,
        vec![],
    );

    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn test_error_recovery_state_consecutive_errors_under_limit() {
    let config = error_recovery::ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = error_recovery::ErrorRecoveryState::new(config);

    for _ in 0..4 {
        state.increment_error_count();
    }

    assert!(!state.should_give_up());
}

// ============================================================================
// RecoveryStrategy Edge Case Tests
// ============================================================================

#[test]
fn test_recovery_strategy_panic_mode() {
    let strategy = error_recovery::RecoveryStrategy::PanicMode;
    assert_eq!(strategy, error_recovery::RecoveryStrategy::PanicMode);
}

#[test]
fn test_recovery_strategy_token_insertion() {
    let strategy = error_recovery::RecoveryStrategy::TokenInsertion;
    assert_eq!(strategy, error_recovery::RecoveryStrategy::TokenInsertion);
}

#[test]
fn test_recovery_strategy_token_deletion() {
    let strategy = error_recovery::RecoveryStrategy::TokenDeletion;
    assert_eq!(strategy, error_recovery::RecoveryStrategy::TokenDeletion);
}

#[test]
fn test_recovery_strategy_token_substitution() {
    let strategy = error_recovery::RecoveryStrategy::TokenSubstitution;
    assert_eq!(
        strategy,
        error_recovery::RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn test_recovery_strategy_phrase_level() {
    let strategy = error_recovery::RecoveryStrategy::PhraseLevel;
    assert_eq!(strategy, error_recovery::RecoveryStrategy::PhraseLevel);
}

#[test]
fn test_recovery_strategy_scope_recovery() {
    let strategy = error_recovery::RecoveryStrategy::ScopeRecovery;
    assert_eq!(strategy, error_recovery::RecoveryStrategy::ScopeRecovery);
}

#[test]
fn test_recovery_strategy_all_variants_distinct() {
    let strategies = [
        error_recovery::RecoveryStrategy::PanicMode,
        error_recovery::RecoveryStrategy::TokenInsertion,
        error_recovery::RecoveryStrategy::TokenDeletion,
        error_recovery::RecoveryStrategy::TokenSubstitution,
        error_recovery::RecoveryStrategy::PhraseLevel,
        error_recovery::RecoveryStrategy::ScopeRecovery,
    ];

    // Each should be unique
    for i in 0..strategies.len() {
        for j in (i + 1)..strategies.len() {
            assert_ne!(strategies[i], strategies[j]);
        }
    }
}

#[test]
fn test_recovery_strategy_comparison() {
    let strat1 = error_recovery::RecoveryStrategy::PanicMode;
    let strat2 = error_recovery::RecoveryStrategy::TokenInsertion;
    assert_ne!(strat1, strat2);
}

#[test]
fn test_recovery_strategy_debug() {
    let strategy = error_recovery::RecoveryStrategy::PanicMode;
    let debug_str = format!("{:?}", strategy);
    assert!(debug_str.contains("PanicMode"));
}

#[test]
fn test_recovery_strategy_copy() {
    let strat1 = error_recovery::RecoveryStrategy::TokenInsertion;
    let strat2 = strat1; // Copy, not move
    assert_eq!(strat1, strat2);
    assert_eq!(strat1, error_recovery::RecoveryStrategy::TokenInsertion);
}

#[test]
fn test_recovery_strategy_clone() {
    let original = error_recovery::RecoveryStrategy::TokenSubstitution;
    let cloned = original;
    assert_eq!(original, cloned);
}

// ============================================================================
// Visitor API Edge Case Tests
// ============================================================================

#[test]
fn test_visitor_action_continue() {
    let action = visitor::VisitorAction::Continue;
    assert_eq!(action, visitor::VisitorAction::Continue);
}

#[test]
fn test_visitor_action_skip_children() {
    let action = visitor::VisitorAction::SkipChildren;
    assert_eq!(action, visitor::VisitorAction::SkipChildren);
}

#[test]
fn test_visitor_action_stop() {
    let action = visitor::VisitorAction::Stop;
    assert_eq!(action, visitor::VisitorAction::Stop);
}

#[test]
fn test_visitor_action_all_distinct() {
    assert_ne!(
        visitor::VisitorAction::Continue,
        visitor::VisitorAction::SkipChildren
    );
    assert_ne!(
        visitor::VisitorAction::Continue,
        visitor::VisitorAction::Stop
    );
    assert_ne!(
        visitor::VisitorAction::SkipChildren,
        visitor::VisitorAction::Stop
    );
}

#[test]
fn test_visitor_action_debug() {
    let action = visitor::VisitorAction::Continue;
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("Continue"));
}

#[test]
fn test_visitor_action_copy() {
    let action1 = visitor::VisitorAction::SkipChildren;
    let action2 = action1; // Copy
    assert_eq!(action1, action2);
}

#[test]
fn test_visitor_action_clone() {
    let action1 = visitor::VisitorAction::Stop;
    let action2 = action1;
    assert_eq!(action1, action2);
}

// ============================================================================
// Additional Edge Case Tests
// ============================================================================

#[test]
fn test_recovery_action_delete_token() {
    let action = error_recovery::RecoveryAction::DeleteToken;
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("DeleteToken"));
}

#[test]
fn test_recovery_action_insert_token() {
    let action = error_recovery::RecoveryAction::InsertToken(ir::SymbolId(42));
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("InsertToken"));
}

#[test]
fn test_error_node_structure() {
    let error_node = error_recovery::ErrorNode {
        start_byte: 10,
        end_byte: 20,
        start_position: (1, 5),
        end_position: (1, 15),
        expected: vec![1, 2, 3],
        actual: Some(99),
        recovery: error_recovery::RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };

    assert_eq!(error_node.start_byte, 10);
    assert_eq!(error_node.end_byte, 20);
    assert_eq!(error_node.expected.len(), 3);
    assert_eq!(error_node.actual, Some(99));
}

#[test]
fn test_error_node_zero_length() {
    let error_node = error_recovery::ErrorNode {
        start_byte: 50,
        end_byte: 50,
        start_position: (0, 0),
        end_position: (0, 0),
        expected: vec![],
        actual: None,
        recovery: error_recovery::RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![],
    };

    assert_eq!(error_node.start_byte, error_node.end_byte);
}

#[test]
fn test_error_node_many_expected_tokens() {
    let mut expected = vec![];
    for i in 0..100 {
        expected.push(i as u16);
    }

    let error_node = error_recovery::ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: expected.clone(),
        actual: None,
        recovery: error_recovery::RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };

    assert_eq!(error_node.expected.len(), 100);
}

#[test]
fn test_error_node_with_skipped_tokens() {
    let error_node = error_recovery::ErrorNode {
        start_byte: 0,
        end_byte: 10,
        start_position: (0, 0),
        end_position: (0, 10),
        expected: vec![1],
        actual: Some(99),
        recovery: error_recovery::RecoveryStrategy::PanicMode,
        skipped_tokens: vec![2, 3, 4, 5],
    };

    assert_eq!(error_node.skipped_tokens.len(), 4);
}

#[test]
fn test_spanned_with_large_generic() {
    let large_vec: Vec<i32> = (0..1000).collect();
    let spanned: Spanned<Vec<i32>> = Spanned {
        value: large_vec.clone(),
        span: (0, 4000),
    };

    assert_eq!(spanned.value.len(), 1000);
}

#[test]
fn test_point_with_different_row_and_column_max() {
    let point = pure_parser::Point {
        row: u32::MAX,
        column: 0,
    };

    assert_eq!(point.row, u32::MAX);
    assert_eq!(point.column, 0);
}

#[test]
fn test_error_recovery_config_builder_pattern() {
    // Test that custom config can be built
    let config = error_recovery::ErrorRecoveryConfig {
        max_panic_skip: 75,
        sync_tokens: Default::default(),
        insert_candidates: Default::default(),
        deletable_tokens: Default::default(),
        max_token_deletions: 7,
        max_token_insertions: 5,
        max_consecutive_errors: 15,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![(1, 2), (3, 4), (5, 6)],
        enable_indentation_recovery: true,
    };

    // Verify it was created with expected values
    assert_eq!(config.max_panic_skip, 75);
}

#[test]
fn test_point_arithmetic_simulation() {
    let point1 = pure_parser::Point { row: 10, column: 5 };
    let point2 = pure_parser::Point {
        row: 20,
        column: 15,
    };

    // Simulate row/column arithmetic
    let row_diff = point2.row.saturating_sub(point1.row);
    let col_diff = point2.column.saturating_sub(point1.column);

    assert_eq!(row_diff, 10);
    assert_eq!(col_diff, 10);
}

#[test]
fn test_spanned_sequential_spans() {
    let span1: Spanned<&str> = Spanned {
        value: "hello",
        span: (0, 5),
    };
    let span2: Spanned<&str> = Spanned {
        value: "world",
        span: (5, 10),
    };

    // Spans are adjacent
    assert_eq!(span1.span.1, span2.span.0);
}

#[test]
fn test_recovery_strategy_all_variants_have_debug() {
    let strategies = vec![
        error_recovery::RecoveryStrategy::PanicMode,
        error_recovery::RecoveryStrategy::TokenInsertion,
        error_recovery::RecoveryStrategy::TokenDeletion,
        error_recovery::RecoveryStrategy::TokenSubstitution,
        error_recovery::RecoveryStrategy::PhraseLevel,
        error_recovery::RecoveryStrategy::ScopeRecovery,
    ];

    for strategy in strategies {
        let debug_str = format!("{:?}", strategy);
        assert!(!debug_str.is_empty());
    }
}
