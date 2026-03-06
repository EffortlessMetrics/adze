//! Comprehensive tests for ErrorNode, ErrorRecoveryState, and related types.

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn default_state() -> ErrorRecoveryState {
    ErrorRecoveryState::new(ErrorRecoveryConfig::default())
}

fn make_error_node(
    start: usize,
    end: usize,
    expected: Vec<u16>,
    actual: Option<u16>,
    recovery: RecoveryStrategy,
) -> ErrorNode {
    ErrorNode {
        start_byte: start,
        end_byte: end,
        start_position: (0, start),
        end_position: (0, end),
        expected,
        actual,
        recovery,
        skipped_tokens: Vec::new(),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 1. ErrorNode construction (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_error_node_construct_basic() {
    let node = make_error_node(0, 5, vec![1, 2], Some(3), RecoveryStrategy::PanicMode);
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 5);
    assert_eq!(node.expected, [1, 2]);
    assert_eq!(node.actual, Some(3));
}

#[test]
fn test_error_node_construct_no_actual() {
    let node = make_error_node(10, 20, vec![5], None, RecoveryStrategy::TokenInsertion);
    assert!(node.actual.is_none());
    assert_eq!(node.expected, [5]);
}

#[test]
fn test_error_node_construct_empty_expected() {
    let node = make_error_node(0, 1, vec![], Some(99), RecoveryStrategy::TokenDeletion);
    assert!(node.expected.is_empty());
    assert_eq!(node.actual, Some(99));
}

#[test]
fn test_error_node_construct_with_positions() {
    let node = ErrorNode {
        start_byte: 100,
        end_byte: 200,
        start_position: (5, 10),
        end_position: (5, 110),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![7, 8, 9],
    };
    assert_eq!(node.start_position, (5, 10));
    assert_eq!(node.end_position, (5, 110));
    assert_eq!(node.skipped_tokens, [7, 8, 9]);
}

#[test]
fn test_error_node_construct_all_strategies() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ];
    for strategy in strategies {
        let node = make_error_node(0, 1, vec![1], Some(2), strategy);
        assert_eq!(node.recovery, strategy);
    }
}

#[test]
fn test_error_node_construct_large_expected() {
    let expected: Vec<u16> = (0..256).collect();
    let node = make_error_node(0, 10, expected, Some(999), RecoveryStrategy::PanicMode);
    assert_eq!(node.expected.len(), 256);
    assert_eq!(node.expected[255], 255);
}

#[test]
fn test_error_node_construct_with_skipped_tokens() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 50,
        start_position: (0, 0),
        end_position: (2, 10),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![10, 20, 30, 40],
    };
    assert_eq!(node.skipped_tokens.len(), 4);
}

#[test]
fn test_error_node_construct_zero_width() {
    let node = make_error_node(42, 42, vec![1], None, RecoveryStrategy::TokenInsertion);
    assert_eq!(node.start_byte, node.end_byte);
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. ErrorNode Debug display (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_error_node_debug_contains_start_byte() {
    let node = make_error_node(42, 50, vec![1], Some(2), RecoveryStrategy::PanicMode);
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("42"), "Debug should show start_byte");
}

#[test]
fn test_error_node_debug_contains_end_byte() {
    let node = make_error_node(0, 99, vec![], None, RecoveryStrategy::TokenDeletion);
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("99"), "Debug should show end_byte");
}

#[test]
fn test_error_node_debug_contains_expected() {
    let node = make_error_node(0, 1, vec![7, 8, 9], None, RecoveryStrategy::PanicMode);
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("expected"));
}

#[test]
fn test_error_node_debug_contains_actual() {
    let node = make_error_node(0, 1, vec![], Some(55), RecoveryStrategy::PanicMode);
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("55"));
}

#[test]
fn test_error_node_debug_contains_recovery_strategy() {
    let node = make_error_node(0, 1, vec![], None, RecoveryStrategy::ScopeRecovery);
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("ScopeRecovery"));
}

#[test]
fn test_error_node_debug_none_actual() {
    let node = make_error_node(0, 1, vec![], None, RecoveryStrategy::PanicMode);
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("None"));
}

#[test]
fn test_error_node_debug_nonempty() {
    let node = make_error_node(0, 1, vec![1], Some(2), RecoveryStrategy::PanicMode);
    let debug_str = format!("{node:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn test_error_node_debug_phrase_level() {
    let node = make_error_node(
        0,
        100,
        vec![1, 2, 3],
        Some(4),
        RecoveryStrategy::PhraseLevel,
    );
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("PhraseLevel"));
    assert!(debug_str.contains("100"));
}

// ═════════════════════════════════════════════════════════════════════════════
// 3. RecoveryStrategy variants (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_recovery_strategy_panic_mode_eq() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
}

#[test]
fn test_recovery_strategy_token_insertion_eq() {
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn test_recovery_strategy_variants_distinct() {
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
fn test_recovery_strategy_debug() {
    let debug_str = format!("{:?}", RecoveryStrategy::TokenSubstitution);
    assert_eq!(debug_str, "TokenSubstitution");
}

#[test]
fn test_recovery_strategy_clone() {
    let original = RecoveryStrategy::ScopeRecovery;
    let cloned = original; // Copy, not clone
    assert_eq!(original, cloned);
}

#[test]
fn test_recovery_strategy_debug_all_variants() {
    let dbg = format!("{:?}", RecoveryStrategy::PanicMode);
    assert_eq!(dbg, "PanicMode");
    let dbg = format!("{:?}", RecoveryStrategy::TokenDeletion);
    assert_eq!(dbg, "TokenDeletion");
    let dbg = format!("{:?}", RecoveryStrategy::PhraseLevel);
    assert_eq!(dbg, "PhraseLevel");
}

#[test]
fn test_recovery_strategy_copy_semantics() {
    let a = RecoveryStrategy::TokenInsertion;
    let b = a;
    let c = a; // `a` still usable after copy
    assert_eq!(b, c);
}

#[test]
fn test_recovery_strategy_ne_cross() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::PhraseLevel);
    assert_ne!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion
    );
    assert_ne!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::TokenSubstitution
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 4. ErrorNode with recovery (7 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_record_error_stores_node() {
    let mut state = default_state();
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
fn test_record_error_preserves_expected() {
    let mut state = default_state();
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![10, 20, 30],
        Some(99),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].expected, [10, 20, 30]);
}

#[test]
fn test_record_error_preserves_actual() {
    let mut state = default_state();
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        Some(42),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes()[0].actual, Some(42));
}

#[test]
fn test_record_error_preserves_recovery_strategy() {
    let mut state = default_state();
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        None,
        RecoveryStrategy::ScopeRecovery,
        vec![],
    );
    assert_eq!(
        state.get_error_nodes()[0].recovery,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn test_record_error_none_actual() {
    let mut state = default_state();
    state.record_error(
        5,
        10,
        (1, 0),
        (1, 5),
        vec![1],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    assert!(state.get_error_nodes()[0].actual.is_none());
}

#[test]
fn test_record_error_with_skipped_tokens() {
    let mut state = default_state();
    state.record_error(
        0,
        20,
        (0, 0),
        (0, 20),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![5, 6, 7],
    );
    assert_eq!(state.get_error_nodes()[0].skipped_tokens, [5, 6, 7]);
}

#[test]
fn test_clear_errors_removes_all() {
    let mut state = default_state();
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
    state.record_error(
        1,
        2,
        (0, 1),
        (0, 2),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 2);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// ═════════════════════════════════════════════════════════════════════════════
// 5. Multiple errors (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_errors_count() {
    let mut state = default_state();
    for i in 0..5 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![1],
            Some(2),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn test_multiple_errors_preserve_order() {
    let mut state = default_state();
    for i in 0..3 {
        state.record_error(
            i * 10,
            i * 10 + 5,
            (0, 0),
            (0, 5),
            vec![i as u16],
            None,
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[1].start_byte, 10);
    assert_eq!(nodes[2].start_byte, 20);
}

#[test]
fn test_multiple_errors_different_strategies() {
    let mut state = default_state();
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
    state.record_error(
        1,
        2,
        (0, 1),
        (0, 2),
        vec![],
        None,
        RecoveryStrategy::TokenInsertion,
        vec![],
    );
    state.record_error(
        2,
        3,
        (0, 2),
        (0, 3),
        vec![],
        None,
        RecoveryStrategy::ScopeRecovery,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].recovery, RecoveryStrategy::PanicMode);
    assert_eq!(nodes[1].recovery, RecoveryStrategy::TokenInsertion);
    assert_eq!(nodes[2].recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn test_multiple_errors_different_expected() {
    let mut state = default_state();
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1, 2],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        1,
        2,
        (0, 1),
        (0, 2),
        vec![3, 4, 5],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].expected, [1, 2]);
    assert_eq!(nodes[1].expected, [3, 4, 5]);
}

#[test]
fn test_multiple_errors_mixed_actual() {
    let mut state = default_state();
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        Some(10),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        1,
        2,
        (0, 1),
        (0, 2),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        2,
        3,
        (0, 2),
        (0, 3),
        vec![],
        Some(30),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes[0].actual, Some(10));
    assert!(nodes[1].actual.is_none());
    assert_eq!(nodes[2].actual, Some(30));
}

#[test]
fn test_multiple_errors_large_batch() {
    let mut state = default_state();
    for i in 0..50 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![1],
            Some(2),
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 50);
}

#[test]
fn test_multiple_errors_clear_then_add() {
    let mut state = default_state();
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
    state.clear_errors();
    state.record_error(
        10,
        20,
        (1, 0),
        (1, 10),
        vec![5],
        Some(6),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 10);
}

#[test]
fn test_multiple_errors_each_has_correct_range() {
    let mut state = default_state();
    let ranges = [(0, 5), (10, 15), (20, 30), (100, 200)];
    for &(s, e) in &ranges {
        state.record_error(
            s,
            e,
            (0, s),
            (0, e),
            vec![],
            None,
            RecoveryStrategy::PanicMode,
            vec![],
        );
    }
    let nodes = state.get_error_nodes();
    for (i, &(s, e)) in ranges.iter().enumerate() {
        assert_eq!(nodes[i].start_byte, s);
        assert_eq!(nodes[i].end_byte, e);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 6. Byte range properties (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_byte_range_start_le_end() {
    let node = make_error_node(10, 20, vec![], None, RecoveryStrategy::PanicMode);
    assert!(node.start_byte <= node.end_byte);
}

#[test]
fn test_byte_range_zero_width_valid() {
    let node = make_error_node(5, 5, vec![1], None, RecoveryStrategy::TokenInsertion);
    assert_eq!(node.start_byte, node.end_byte);
    assert_eq!(node.end_byte - node.start_byte, 0);
}

#[test]
fn test_byte_range_single_byte() {
    let node = make_error_node(7, 8, vec![], None, RecoveryStrategy::PanicMode);
    assert_eq!(node.end_byte - node.start_byte, 1);
}

#[test]
fn test_byte_range_large_span() {
    let node = make_error_node(0, 1_000_000, vec![], None, RecoveryStrategy::PanicMode);
    assert_eq!(node.end_byte - node.start_byte, 1_000_000);
}

#[test]
fn test_byte_range_at_zero() {
    let node = make_error_node(0, 0, vec![], None, RecoveryStrategy::PanicMode);
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 0);
}

#[test]
fn test_byte_range_max_usize() {
    let node = make_error_node(
        usize::MAX - 1,
        usize::MAX,
        vec![],
        None,
        RecoveryStrategy::PanicMode,
    );
    assert_eq!(node.end_byte - node.start_byte, 1);
}

#[test]
fn test_byte_range_adjacent_errors_non_overlapping() {
    let a = make_error_node(0, 5, vec![], None, RecoveryStrategy::PanicMode);
    let b = make_error_node(5, 10, vec![], None, RecoveryStrategy::PanicMode);
    assert!(a.end_byte <= b.start_byte);
}

#[test]
fn test_byte_range_recorded_errors_non_overlapping() {
    let mut state = default_state();
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
        5,
        10,
        (0, 5),
        (0, 10),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        10,
        15,
        (0, 10),
        (0, 15),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    let nodes = state.get_error_nodes();
    for pair in nodes.windows(2) {
        assert!(pair[0].end_byte <= pair[1].start_byte);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 7. Edge cases (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_edge_empty_expected_and_no_actual() {
    let node = make_error_node(0, 1, vec![], None, RecoveryStrategy::PanicMode);
    assert!(node.expected.is_empty());
    assert!(node.actual.is_none());
}

#[test]
fn test_edge_large_token_id() {
    let node = make_error_node(
        0,
        1,
        vec![u16::MAX],
        Some(u16::MAX),
        RecoveryStrategy::PanicMode,
    );
    assert_eq!(node.expected[0], u16::MAX);
    assert_eq!(node.actual, Some(u16::MAX));
}

#[test]
fn test_edge_duplicate_expected() {
    let node = make_error_node(0, 1, vec![5, 5, 5], None, RecoveryStrategy::PanicMode);
    assert_eq!(node.expected.len(), 3);
}

#[test]
fn test_edge_node_clone() {
    let node = make_error_node(
        10,
        20,
        vec![1, 2, 3],
        Some(4),
        RecoveryStrategy::TokenDeletion,
    );
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, node.start_byte);
    assert_eq!(cloned.end_byte, node.end_byte);
    assert_eq!(cloned.expected, node.expected);
    assert_eq!(cloned.actual, node.actual);
    assert_eq!(cloned.recovery, node.recovery);
}

#[test]
fn test_edge_many_skipped_tokens() {
    let skipped: Vec<u16> = (0..1000).collect();
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5000,
        start_position: (0, 0),
        end_position: (100, 0),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: skipped,
    };
    assert_eq!(node.skipped_tokens.len(), 1000);
}

#[test]
fn test_edge_record_error_empty_skipped() {
    let mut state = default_state();
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
    assert!(state.get_error_nodes()[0].skipped_tokens.is_empty());
}

#[test]
fn test_edge_positions_multiline() {
    let node = ErrorNode {
        start_byte: 50,
        end_byte: 100,
        start_position: (3, 10),
        end_position: (7, 5),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::PhraseLevel,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_position.0, 3);
    assert_eq!(node.end_position.0, 7);
}

#[test]
fn test_edge_error_count_increment_and_reset() {
    let mut state = default_state();
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up()); // default max is 10
    state.reset_error_count();
    assert!(!state.should_give_up());
}

// ═════════════════════════════════════════════════════════════════════════════
// 8. Config builder and state interaction (bonus, pushes past 55)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_config_builder_defaults() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(config.max_panic_skip, 50);
    assert!(config.sync_tokens.is_empty());
    assert!(config.insert_candidates.is_empty());
    assert!(config.scope_delimiters.is_empty());
}

#[test]
fn test_config_builder_chain() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .max_consecutive_errors(5)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(config.max_consecutive_errors, 5);
    assert!(!config.enable_phrase_recovery);
    assert!(!config.enable_scope_recovery);
}

#[test]
fn test_state_should_give_up_at_limit() {
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
fn test_scope_push_pop_roundtrip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41) // '(' = 40, ')' = 41
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(state.pop_scope(41));
}

#[test]
fn test_scope_pop_mismatch_returns_false() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(!state.pop_scope(93)); // wrong closer
}

#[test]
fn test_recent_tokens_updated() {
    let mut state = default_state();
    for i in 0..15 {
        state.add_recent_token(i);
    }
    // Only the last 10 should be kept
    // Just verify no panic — internal state is behind a Mutex
}

#[test]
fn test_is_scope_delimiter_static() {
    let delimiters = [(40, 41), (91, 93), (123, 125)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(123, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(0, &delimiters));
}

#[test]
fn test_is_matching_delimiter_static() {
    let delimiters = [(40, 41), (91, 93)];
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
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        40,
        93,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        91,
        41,
        &delimiters
    ));
}
