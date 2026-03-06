//! Comprehensive tests for parse error types and error handling in the adze runtime.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze::errors::{ParseError, ParseErrorReason};
use adze::{SpanError, SpanErrorReason};

// ── 1. ParseErrorReason variants ────────────────────────────────────────

#[test]
fn test_unexpected_token_variant_constructable() {
    let reason = ParseErrorReason::UnexpectedToken("foo".to_string());
    assert!(matches!(reason, ParseErrorReason::UnexpectedToken(ref s) if s == "foo"));
}

#[test]
fn test_missing_token_variant_constructable() {
    let reason = ParseErrorReason::MissingToken("semicolon".to_string());
    assert!(matches!(reason, ParseErrorReason::MissingToken(ref s) if s == "semicolon"));
}

#[test]
fn test_failed_node_variant_constructable_empty() {
    let reason = ParseErrorReason::FailedNode(vec![]);
    assert!(matches!(reason, ParseErrorReason::FailedNode(ref v) if v.is_empty()));
}

#[test]
fn test_failed_node_variant_constructable_with_children() {
    let child = ParseError {
        reason: ParseErrorReason::UnexpectedToken("x".to_string()),
        start: 0,
        end: 1,
    };
    let reason = ParseErrorReason::FailedNode(vec![child]);
    assert!(matches!(reason, ParseErrorReason::FailedNode(ref v) if v.len() == 1));
}

#[test]
fn test_variants_are_distinct_unexpected_vs_missing() {
    let a = ParseErrorReason::UnexpectedToken("t".to_string());
    let b = ParseErrorReason::MissingToken("t".to_string());
    // They carry the same payload but are different variants
    assert!(!matches!(a, ParseErrorReason::MissingToken(_)));
    assert!(!matches!(b, ParseErrorReason::UnexpectedToken(_)));
}

#[test]
fn test_variants_are_distinct_unexpected_vs_failed() {
    let a = ParseErrorReason::UnexpectedToken("t".to_string());
    let b = ParseErrorReason::FailedNode(vec![]);
    assert!(!matches!(a, ParseErrorReason::FailedNode(_)));
    assert!(!matches!(b, ParseErrorReason::UnexpectedToken(_)));
}

#[test]
fn test_variants_are_distinct_missing_vs_failed() {
    let a = ParseErrorReason::MissingToken("t".to_string());
    let b = ParseErrorReason::FailedNode(vec![]);
    assert!(!matches!(a, ParseErrorReason::FailedNode(_)));
    assert!(!matches!(b, ParseErrorReason::MissingToken(_)));
}

#[test]
fn test_parse_error_reason_debug_is_implemented() {
    let reason = ParseErrorReason::UnexpectedToken("bad".to_string());
    let debug = format!("{:?}", reason);
    assert!(debug.contains("UnexpectedToken"));
}

// ── 2. SpanError properties ─────────────────────────────────────────────

#[test]
fn test_span_error_stores_span() {
    let err = SpanError {
        span: (3, 7),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    assert_eq!(err.span, (3, 7));
}

#[test]
fn test_span_error_stores_source_len() {
    let err = SpanError {
        span: (0, 5),
        source_len: 4,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    assert_eq!(err.source_len, 4);
}

#[test]
fn test_span_error_start_greater_than_end() {
    let err = SpanError {
        span: (5, 2),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    assert_eq!(err.reason, SpanErrorReason::StartGreaterThanEnd);
}

#[test]
fn test_span_error_start_out_of_bounds() {
    let err = SpanError {
        span: (20, 25),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    assert_eq!(err.reason, SpanErrorReason::StartOutOfBounds);
}

#[test]
fn test_span_error_end_out_of_bounds() {
    let err = SpanError {
        span: (0, 100),
        source_len: 50,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    assert_eq!(err.reason, SpanErrorReason::EndOutOfBounds);
}

#[test]
fn test_span_error_is_std_error() {
    let err = SpanError {
        span: (5, 2),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    // SpanError implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_span_error_clone_preserves_fields() {
    let err = SpanError {
        span: (1, 3),
        source_len: 5,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn test_span_error_reason_variants_eq() {
    assert_eq!(
        SpanErrorReason::StartGreaterThanEnd,
        SpanErrorReason::StartGreaterThanEnd
    );
    assert_ne!(
        SpanErrorReason::StartGreaterThanEnd,
        SpanErrorReason::StartOutOfBounds
    );
    assert_ne!(
        SpanErrorReason::EndOutOfBounds,
        SpanErrorReason::StartOutOfBounds
    );
}

// ── 3. Error Display formatting ─────────────────────────────────────────

#[test]
fn test_span_error_display_start_greater_than_end() {
    let err = SpanError {
        span: (5, 2),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    let msg = format!("{err}");
    assert!(msg.contains("5"));
    assert!(msg.contains("2"));
    assert!(msg.contains("start"));
    assert!(msg.contains("end"));
}

#[test]
fn test_span_error_display_start_out_of_bounds() {
    let err = SpanError {
        span: (20, 25),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    let msg = format!("{err}");
    assert!(msg.contains("20"));
    assert!(msg.contains("source length"));
    assert!(msg.contains("10"));
}

#[test]
fn test_span_error_display_end_out_of_bounds() {
    let err = SpanError {
        span: (0, 100),
        source_len: 50,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let msg = format!("{err}");
    assert!(msg.contains("100"));
    assert!(msg.contains("source length"));
    assert!(msg.contains("50"));
}

#[test]
fn test_error_reporting_parse_error_display_with_token() {
    let err = adze::error_reporting::ParseError {
        line: 3,
        column: 15,
        unexpected_token: Some("foo".to_string()),
        expected: vec!["number".to_string(), "string".to_string()],
        context: "in expression".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("3:15"));
    assert!(msg.contains("unexpected token 'foo'"));
    assert!(msg.contains("number"));
    assert!(msg.contains("string"));
    assert!(msg.contains("in expression"));
}

#[test]
fn test_error_reporting_parse_error_display_eof() {
    let err = adze::error_reporting::ParseError {
        line: 1,
        column: 1,
        unexpected_token: None,
        expected: vec![],
        context: String::new(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("unexpected end of input"));
}

#[test]
fn test_error_reporting_parse_error_display_no_expected() {
    let err = adze::error_reporting::ParseError {
        line: 2,
        column: 4,
        unexpected_token: Some("@".to_string()),
        expected: vec![],
        context: String::new(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("unexpected token '@'"));
    assert!(!msg.contains("expected one of"));
}

#[test]
fn test_error_reporting_parse_error_display_no_context() {
    let err = adze::error_reporting::ParseError {
        line: 5,
        column: 10,
        unexpected_token: Some("x".to_string()),
        expected: vec!["y".to_string()],
        context: String::new(),
    };
    let msg = format!("{err}");
    // Empty context should not produce "()" in output
    assert!(!msg.ends_with("()"));
}

#[test]
fn test_span_error_display_contains_span_range() {
    let err = SpanError {
        span: (10, 20),
        source_len: 15,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let msg = format!("{err}");
    assert!(msg.contains("10..20"));
}

// ── 4. Error comparison ─────────────────────────────────────────────────

#[test]
fn test_span_error_eq_same() {
    let a = SpanError {
        span: (0, 5),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let b = SpanError {
        span: (0, 5),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    assert_eq!(a, b);
}

#[test]
fn test_span_error_ne_different_span() {
    let a = SpanError {
        span: (0, 5),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let b = SpanError {
        span: (1, 5),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    assert_ne!(a, b);
}

#[test]
fn test_span_error_ne_different_source_len() {
    let a = SpanError {
        span: (0, 5),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let b = SpanError {
        span: (0, 5),
        source_len: 20,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    assert_ne!(a, b);
}

#[test]
fn test_span_error_ne_different_reason() {
    let a = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    let b = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    assert_ne!(a, b);
}

#[test]
fn test_recovery_strategy_eq() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn test_recovery_strategy_ne() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion);
    assert_ne!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn test_recovery_action_eq() {
    let a = RecoveryAction::DeleteToken;
    let b = RecoveryAction::DeleteToken;
    assert_eq!(a, b);

    let c = RecoveryAction::InsertToken(ir::SymbolId(5));
    let d = RecoveryAction::InsertToken(ir::SymbolId(5));
    assert_eq!(c, d);
}

// ── 5. Error Debug ──────────────────────────────────────────────────────

#[test]
fn test_parse_error_debug_contains_reason() {
    let err = ParseError {
        reason: ParseErrorReason::UnexpectedToken("abc".to_string()),
        start: 0,
        end: 3,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("UnexpectedToken"));
    assert!(debug.contains("abc"));
}

#[test]
fn test_parse_error_debug_contains_span() {
    let err = ParseError {
        reason: ParseErrorReason::MissingToken("if".to_string()),
        start: 10,
        end: 12,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("10"));
    assert!(debug.contains("12"));
}

#[test]
fn test_parse_error_reason_debug_missing_token() {
    let reason = ParseErrorReason::MissingToken("semicolon".to_string());
    let debug = format!("{reason:?}");
    assert!(debug.contains("MissingToken"));
    assert!(debug.contains("semicolon"));
}

#[test]
fn test_parse_error_reason_debug_failed_node() {
    let reason = ParseErrorReason::FailedNode(vec![]);
    let debug = format!("{reason:?}");
    assert!(debug.contains("FailedNode"));
}

#[test]
fn test_span_error_debug_contains_fields() {
    let err = SpanError {
        span: (1, 5),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("SpanError"));
    assert!(debug.contains("EndOutOfBounds"));
}

#[test]
fn test_span_error_reason_debug() {
    let r1 = SpanErrorReason::StartGreaterThanEnd;
    let r2 = SpanErrorReason::StartOutOfBounds;
    let r3 = SpanErrorReason::EndOutOfBounds;
    assert!(format!("{r1:?}").contains("StartGreaterThanEnd"));
    assert!(format!("{r2:?}").contains("StartOutOfBounds"));
    assert!(format!("{r3:?}").contains("EndOutOfBounds"));
}

#[test]
fn test_recovery_strategy_debug() {
    let s = RecoveryStrategy::PhraseLevel;
    let debug = format!("{s:?}");
    assert!(debug.contains("PhraseLevel"));
}

#[test]
fn test_error_node_debug() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (1, 0),
        end_position: (1, 5),
        expected: vec![1, 2],
        actual: Some(3),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![],
    };
    let debug = format!("{node:?}");
    assert!(debug.contains("ErrorNode"));
    assert!(debug.contains("TokenDeletion"));
}

// ── 6. Error from parse patterns ────────────────────────────────────────

#[test]
fn test_parse_error_with_unexpected_token() {
    let err = ParseError {
        reason: ParseErrorReason::UnexpectedToken("@#$".to_string()),
        start: 5,
        end: 8,
    };
    assert_eq!(err.start, 5);
    assert_eq!(err.end, 8);
    assert!(matches!(err.reason, ParseErrorReason::UnexpectedToken(ref s) if s == "@#$"));
}

#[test]
fn test_parse_error_with_missing_token() {
    let err = ParseError {
        reason: ParseErrorReason::MissingToken("}".to_string()),
        start: 20,
        end: 20,
    };
    // Missing tokens can have zero-length spans
    assert_eq!(err.start, err.end);
}

#[test]
fn test_parse_error_with_nested_failures() {
    let inner1 = ParseError {
        reason: ParseErrorReason::UnexpectedToken("a".to_string()),
        start: 0,
        end: 1,
    };
    let inner2 = ParseError {
        reason: ParseErrorReason::MissingToken("b".to_string()),
        start: 1,
        end: 1,
    };
    let outer = ParseError {
        reason: ParseErrorReason::FailedNode(vec![inner1, inner2]),
        start: 0,
        end: 5,
    };
    if let ParseErrorReason::FailedNode(ref children) = outer.reason {
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected FailedNode");
    }
}

#[test]
fn test_collect_parsing_errors_function_exists() {
    // Verify the function is accessible; actual parse trees would come from parsing
    use adze::errors::collect_parsing_errors;
    let _fn_ptr: fn(&adze::pure_parser::ParsedNode, &[u8], &mut Vec<ParseError>) =
        collect_parsing_errors;
}

#[test]
fn test_parsed_node_type_exists_and_is_public() {
    // ParsedNode is re-exported and usable as a type
    fn accept_node(_node: &adze::pure_parser::ParsedNode) {}
    let _ = accept_node;
}

#[test]
fn test_error_recovery_state_tracks_errors() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (1, 0),
        (1, 5),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
}

#[test]
fn test_error_recovery_strategy_selection_panic_on_excess() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // Exceed the limit
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();

    let strategy = state.determine_recovery_strategy(&[10], Some(99), (1, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn test_error_recovery_config_builder_chaining() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .add_sync_token(1)
        .add_insertable_token(3)
        .add_deletable_token(5)
        .add_scope_delimiter(40, 41)
        .max_consecutive_errors(5)
        .build();

    assert_eq!(config.max_panic_skip, 100);
    assert!(config.sync_tokens.iter().any(|t| t.0 == 1));
    assert!(config.insert_candidates.iter().any(|t| t.0 == 3));
    assert!(config.deletable_tokens.contains(&5));
    assert_eq!(config.scope_delimiters, [(40, 41)]);
    assert_eq!(config.max_consecutive_errors, 5);
}

// ── 7. Edge cases ───────────────────────────────────────────────────────

#[test]
fn test_parse_error_empty_unexpected_token() {
    let err = ParseError {
        reason: ParseErrorReason::UnexpectedToken(String::new()),
        start: 0,
        end: 0,
    };
    assert!(matches!(err.reason, ParseErrorReason::UnexpectedToken(ref s) if s.is_empty()));
}

#[test]
fn test_parse_error_zero_length_span() {
    let err = ParseError {
        reason: ParseErrorReason::MissingToken("x".to_string()),
        start: 42,
        end: 42,
    };
    assert_eq!(err.start, err.end);
}

#[test]
fn test_span_error_zero_length_valid_span() {
    // A zero-length span where start == end is valid (insertion point)
    let err = SpanError {
        span: (5, 5),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    assert_eq!(err.span.0, err.span.1);
}

#[test]
fn test_failed_node_deeply_nested() {
    let leaf = ParseError {
        reason: ParseErrorReason::UnexpectedToken("!".to_string()),
        start: 0,
        end: 1,
    };
    let mid = ParseError {
        reason: ParseErrorReason::FailedNode(vec![leaf]),
        start: 0,
        end: 5,
    };
    let outer = ParseError {
        reason: ParseErrorReason::FailedNode(vec![mid]),
        start: 0,
        end: 10,
    };
    // Verify 3 levels of nesting
    if let ParseErrorReason::FailedNode(ref level1) = outer.reason {
        if let ParseErrorReason::FailedNode(ref level2) = level1[0].reason {
            assert!(matches!(
                level2[0].reason,
                ParseErrorReason::UnexpectedToken(_)
            ));
        } else {
            panic!("Expected FailedNode at level 1");
        }
    } else {
        panic!("Expected FailedNode at level 0");
    }
}

#[test]
fn test_error_reporting_parse_error_empty_everything() {
    let err = adze::error_reporting::ParseError {
        line: 0,
        column: 0,
        unexpected_token: None,
        expected: vec![],
        context: String::new(),
    };
    let msg = format!("{err}");
    // Should still produce some output
    assert!(!msg.is_empty());
    assert!(msg.contains("0:0"));
}

#[test]
fn test_error_recovery_scope_delimiter_matching() {
    assert!(ErrorRecoveryState::is_scope_delimiter(
        40,
        &[(40, 41), (91, 93)]
    ));
    assert!(ErrorRecoveryState::is_scope_delimiter(
        41,
        &[(40, 41), (91, 93)]
    ));
    assert!(!ErrorRecoveryState::is_scope_delimiter(
        50,
        &[(40, 41), (91, 93)]
    ));
}

#[test]
fn test_error_recovery_matching_delimiter_pair() {
    assert!(ErrorRecoveryState::is_matching_delimiter(
        40,
        41,
        &[(40, 41)]
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        40,
        93,
        &[(40, 41)]
    ));
}

#[test]
fn test_error_recovery_give_up_after_max_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);

    assert!(!state.should_give_up());
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}
