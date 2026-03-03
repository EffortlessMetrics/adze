#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for error handling boundaries in the adze runtime.
//!
//! Covers: errors at start/end/middle of input, multiple errors at one position,
//! errors spanning entire input, zero-length errors, u32::MAX positions,
//! overlapping spans, error node creation, and display formatting.

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};
use adze::error_reporting::ParseError as ReportingParseError;
use adze::{SpanError, SpanErrorReason};

// ── helpers ─────────────────────────────────────────────────────────────────

fn make_state() -> ErrorRecoveryState {
    ErrorRecoveryState::new(ErrorRecoveryConfig::default())
}

fn record(
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

// ── 1. Error at start of input ──────────────────────────────────────────────

#[test]
fn error_at_start_byte_zero() {
    let mut state = make_state();
    record(&mut state, 0, 1, vec![1, 2], Some(99), RecoveryStrategy::TokenDeletion);
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 1);
}

#[test]
fn error_at_start_position_zero_zero() {
    let mut state = make_state();
    state.record_error(0, 3, (0, 0), (0, 3), vec![10], Some(20), RecoveryStrategy::PanicMode, vec![20]);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.expected, vec![10]);
    assert_eq!(node.actual, Some(20));
}

#[test]
fn error_at_start_with_no_actual_token() {
    let mut state = make_state();
    record(&mut state, 0, 0, vec![5], None, RecoveryStrategy::TokenInsertion);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 0);
    assert!(node.actual.is_none());
}

// ── 2. Error at end of input ────────────────────────────────────────────────

#[test]
fn error_at_end_of_input() {
    let input_len = 100;
    let mut state = make_state();
    record(&mut state, input_len, input_len, vec![1], None, RecoveryStrategy::TokenInsertion);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, input_len);
    assert_eq!(node.end_byte, input_len);
}

#[test]
fn error_spanning_last_byte() {
    let mut state = make_state();
    record(&mut state, 99, 100, vec![3, 4], Some(7), RecoveryStrategy::TokenDeletion);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 99);
    assert_eq!(node.end_byte, 100);
}

// ── 3. Error in middle ──────────────────────────────────────────────────────

#[test]
fn error_in_middle_of_input() {
    let mut state = make_state();
    record(&mut state, 50, 55, vec![1], Some(2), RecoveryStrategy::TokenSubstitution);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 50);
    assert_eq!(node.end_byte, 55);
    assert_eq!(node.recovery, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn error_in_middle_preserves_expected_list() {
    let mut state = make_state();
    let expected: Vec<u16> = (0..20).collect();
    record(&mut state, 10, 15, expected.clone(), Some(99), RecoveryStrategy::PhraseLevel);
    assert_eq!(state.get_error_nodes()[0].expected, expected);
}

// ── 4. Multiple errors at same position ─────────────────────────────────────

#[test]
fn multiple_errors_at_same_position() {
    let mut state = make_state();
    for i in 0..5 {
        record(&mut state, 10, 12, vec![i], Some(100), RecoveryStrategy::TokenDeletion);
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 5);
    for node in &nodes {
        assert_eq!(node.start_byte, 10);
        assert_eq!(node.end_byte, 12);
    }
}

#[test]
fn multiple_errors_same_position_different_strategies() {
    let mut state = make_state();
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
    ];
    for strategy in &strategies {
        record(&mut state, 5, 5, vec![1], Some(2), *strategy);
    }
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), strategies.len());
    for i in 0..strategies.len() {
        assert_eq!(nodes[i].recovery, strategies[i]);
    }
}

// ── 5. Error spanning entire input ──────────────────────────────────────────

#[test]
fn error_spanning_entire_input() {
    let input_len = 500;
    let mut state = make_state();
    record(&mut state, 0, input_len, vec![1, 2, 3], Some(99), RecoveryStrategy::PanicMode);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, input_len);
}

#[test]
fn error_spanning_single_byte_input() {
    let mut state = make_state();
    record(&mut state, 0, 1, vec![1], Some(2), RecoveryStrategy::TokenDeletion);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 1);
    assert_eq!(node.end_byte - node.start_byte, 1);
}

// ── 6. Zero-length error ────────────────────────────────────────────────────

#[test]
fn zero_length_error_at_start() {
    let mut state = make_state();
    record(&mut state, 0, 0, vec![1], None, RecoveryStrategy::TokenInsertion);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, node.end_byte);
    assert_eq!(node.start_byte, 0);
}

#[test]
fn zero_length_error_in_middle() {
    let mut state = make_state();
    record(&mut state, 42, 42, vec![10, 20], None, RecoveryStrategy::TokenInsertion);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, 42);
    assert_eq!(node.end_byte, 42);
}

#[test]
fn zero_length_error_at_end() {
    let mut state = make_state();
    record(&mut state, 1000, 1000, vec![1], None, RecoveryStrategy::PhraseLevel);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.end_byte - node.start_byte, 0);
}

// ── 7. Error with u32::MAX positions ────────────────────────────────────────

#[test]
fn error_with_u32_max_start_byte() {
    let big = u32::MAX as usize;
    let mut state = make_state();
    record(&mut state, big, big + 1, vec![1], Some(2), RecoveryStrategy::PanicMode);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, big);
    assert_eq!(node.end_byte, big + 1);
}

#[test]
fn error_with_u32_max_end_byte() {
    let big = u32::MAX as usize;
    let mut state = make_state();
    record(&mut state, big - 10, big, vec![1, 2], Some(3), RecoveryStrategy::TokenDeletion);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.end_byte, big);
    assert_eq!(node.end_byte - node.start_byte, 10);
}

#[test]
fn zero_length_error_at_u32_max() {
    let big = u32::MAX as usize;
    let mut state = make_state();
    record(&mut state, big, big, vec![], None, RecoveryStrategy::PanicMode);
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.start_byte, big);
    assert_eq!(node.end_byte, big);
}

// ── 8. Error with overlapping spans ─────────────────────────────────────────

#[test]
fn overlapping_error_spans_both_recorded() {
    let mut state = make_state();
    record(&mut state, 5, 15, vec![1], Some(2), RecoveryStrategy::PanicMode);
    record(&mut state, 10, 20, vec![3], Some(4), RecoveryStrategy::TokenDeletion);
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 2);
    // Overlap region: 10..15
    assert!(nodes[0].end_byte > nodes[1].start_byte);
}

#[test]
fn nested_error_spans() {
    let mut state = make_state();
    record(&mut state, 0, 100, vec![1], Some(2), RecoveryStrategy::PanicMode);
    record(&mut state, 20, 50, vec![3], Some(4), RecoveryStrategy::TokenDeletion);
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 2);
    // Inner error is fully contained in outer
    assert!(nodes[0].start_byte <= nodes[1].start_byte);
    assert!(nodes[0].end_byte >= nodes[1].end_byte);
}

#[test]
fn identical_error_spans() {
    let mut state = make_state();
    record(&mut state, 10, 20, vec![1], Some(2), RecoveryStrategy::PanicMode);
    record(&mut state, 10, 20, vec![3], Some(4), RecoveryStrategy::TokenInsertion);
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].start_byte, nodes[1].start_byte);
    assert_eq!(nodes[0].end_byte, nodes[1].end_byte);
    assert_ne!(nodes[0].recovery, nodes[1].recovery);
}

// ── 9. Error node creation ──────────────────────────────────────────────────

#[test]
fn error_node_preserves_all_fields() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 20,
        start_position: (1, 5),
        end_position: (1, 15),
        expected: vec![1, 2, 3],
        actual: Some(42),
        recovery: RecoveryStrategy::ScopeRecovery,
        skipped_tokens: vec![7, 8, 9],
    };
    assert_eq!(node.start_byte, 10);
    assert_eq!(node.end_byte, 20);
    assert_eq!(node.expected, vec![1, 2, 3]);
    assert_eq!(node.actual, Some(42));
    assert_eq!(node.recovery, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn error_node_clone_is_independent() {
    let node = ErrorNode {
        start_byte: 5,
        end_byte: 10,
        start_position: (0, 5),
        end_position: (0, 10),
        expected: vec![1],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let clone = node.clone();
    assert_eq!(clone.start_byte, node.start_byte);
    assert_eq!(clone.end_byte, node.end_byte);
    assert_eq!(clone.expected, node.expected);
    assert_eq!(clone.actual, node.actual);
    assert_eq!(clone.recovery, node.recovery);
}

#[test]
fn error_node_debug_format() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![1],
        actual: Some(2),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![],
    };
    let debug = format!("{:?}", node);
    assert!(debug.contains("ErrorNode"));
    assert!(debug.contains("start_byte"));
    assert!(debug.contains("end_byte"));
}

#[test]
fn error_node_with_empty_expected() {
    let mut state = make_state();
    record(&mut state, 5, 10, vec![], Some(1), RecoveryStrategy::PanicMode);
    let node = &state.get_error_nodes()[0];
    assert!(node.expected.is_empty());
    assert_eq!(node.actual, Some(1));
}

#[test]
fn error_node_with_many_skipped_tokens() {
    let mut state = make_state();
    let skipped: Vec<u16> = (0..100).collect();
    state.record_error(
        0, 100,
        (0, 0), (0, 100),
        vec![1], Some(2),
        RecoveryStrategy::PanicMode,
        skipped.clone(),
    );
    let node = &state.get_error_nodes()[0];
    assert_eq!(node.skipped_tokens.len(), 100);
}

// ── 10. Error display formatting ────────────────────────────────────────────

#[test]
fn span_error_display_start_greater_than_end() {
    let err = SpanError {
        span: (10, 5),
        source_len: 20,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    let msg = format!("{err}");
    assert!(msg.contains("10"));
    assert!(msg.contains("5"));
    assert!(msg.contains("start"));
    assert!(msg.contains("end"));
}

#[test]
fn span_error_display_start_out_of_bounds() {
    let err = SpanError {
        span: (30, 35),
        source_len: 20,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    let msg = format!("{err}");
    assert!(msg.contains("30"));
    assert!(msg.contains("20"), "should contain source_len: {msg}");
}

#[test]
fn span_error_display_end_out_of_bounds() {
    let err = SpanError {
        span: (5, 50),
        source_len: 20,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let msg = format!("{err}");
    assert!(msg.contains("50"));
    assert!(msg.contains("20"), "should contain source_len: {msg}");
}

#[test]
fn reporting_parse_error_unexpected_eof() {
    let err = ReportingParseError {
        line: 1,
        column: 1,
        unexpected_token: None,
        expected: vec!["number".to_string()],
        context: String::new(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("1:1"));
    assert!(msg.contains("unexpected end of input"));
    assert!(msg.contains("number"));
}

#[test]
fn reporting_parse_error_with_context() {
    let err = ReportingParseError {
        line: 10,
        column: 5,
        unexpected_token: Some("}".to_string()),
        expected: vec!["identifier".to_string(), "number".to_string()],
        context: "inside function body".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("10:5"));
    assert!(msg.contains("}"));
    assert!(msg.contains("identifier"));
    assert!(msg.contains("inside function body"));
}

#[test]
fn span_error_is_std_error() {
    let err = SpanError {
        span: (10, 5),
        source_len: 20,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    // SpanError implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

#[test]
fn recovery_strategy_debug_all_variants() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for strategy in &strategies {
        let debug = format!("{:?}", strategy);
        assert!(!debug.is_empty());
    }
}

#[test]
fn recovery_strategy_clone_eq() {
    let a = RecoveryStrategy::PanicMode;
    let b = a;
    assert_eq!(a, b);

    let c = RecoveryStrategy::TokenInsertion;
    assert_ne!(a, c);
}
