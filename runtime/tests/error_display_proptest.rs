#![allow(clippy::needless_range_loop)]

//! Property-based tests for error display formatting in the adze runtime.
//!
//! Covers `ParseErrorReason` (Debug), `SpanError` (Display),
//! `error_reporting::ParseError` (Display), and related formatting.

use adze::error_reporting::ParseError as ReportingParseError;
use adze::errors::{ParseError, ParseErrorReason};
use adze::{SpanError, SpanErrorReason};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn token_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}"
}

fn expected_tokens_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(token_strategy(), 0..=8)
}

fn context_strategy() -> impl Strategy<Value = String> {
    prop_oneof![Just(String::new()), "[a-z ]{1,40}",]
}

fn span_error_reason_strategy() -> impl Strategy<Value = SpanErrorReason> {
    prop_oneof![
        Just(SpanErrorReason::StartGreaterThanEnd),
        Just(SpanErrorReason::StartOutOfBounds),
        Just(SpanErrorReason::EndOutOfBounds),
    ]
}

// ---------------------------------------------------------------------------
// 1. SpanError Display — StartGreaterThanEnd contains start and end
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_start_gt_end_contains_positions(
        start in 1usize..=10000,
        delta in 1usize..=10000,
        source_len in 0usize..=20000,
    ) {
        let end = start.saturating_sub(delta).min(start - 1); // end < start
        let err = SpanError {
            span: (start, end),
            source_len,
            reason: SpanErrorReason::StartGreaterThanEnd,
        };
        let display = format!("{err}");
        prop_assert!(display.contains(&start.to_string()));
        prop_assert!(display.contains(&end.to_string()));
        prop_assert!(display.contains("Invalid span"));
    }
}

// ---------------------------------------------------------------------------
// 2. SpanError Display — StartOutOfBounds contains source_len
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_start_oob_contains_source_len(
        start in 1usize..=10000,
        end in 1usize..=10000,
        source_len in 0usize..=10000,
    ) {
        let err = SpanError {
            span: (start, end),
            source_len,
            reason: SpanErrorReason::StartOutOfBounds,
        };
        let display = format!("{err}");
        prop_assert!(display.contains(&source_len.to_string()));
        prop_assert!(display.contains("start"));
        prop_assert!(display.contains("source length"));
    }
}

// ---------------------------------------------------------------------------
// 3. SpanError Display — EndOutOfBounds contains source_len
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_end_oob_contains_source_len(
        start in 0usize..=10000,
        end in 1usize..=10000,
        source_len in 0usize..=10000,
    ) {
        let err = SpanError {
            span: (start, end),
            source_len,
            reason: SpanErrorReason::EndOutOfBounds,
        };
        let display = format!("{err}");
        prop_assert!(display.contains(&source_len.to_string()));
        prop_assert!(display.contains("end"));
        prop_assert!(display.contains("source length"));
    }
}

// ---------------------------------------------------------------------------
// 4. SpanError Display never panics for any reason variant
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn span_error_display_never_panics(
        start in 0usize..=usize::MAX / 2,
        end in 0usize..=usize::MAX / 2,
        source_len in 0usize..=usize::MAX / 2,
        reason in span_error_reason_strategy(),
    ) {
        let err = SpanError { span: (start, end), source_len, reason };
        let _ = format!("{err}");
    }
}

// ---------------------------------------------------------------------------
// 5. SpanError Display is non-empty
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_is_non_empty(
        start in 0usize..=5000,
        end in 0usize..=5000,
        source_len in 0usize..=5000,
        reason in span_error_reason_strategy(),
    ) {
        let err = SpanError { span: (start, end), source_len, reason };
        let display = format!("{err}");
        prop_assert!(!display.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 6. SpanError Display starts with "Invalid span"
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_starts_with_invalid_span(
        start in 0usize..=5000,
        end in 0usize..=5000,
        source_len in 0usize..=5000,
        reason in span_error_reason_strategy(),
    ) {
        let err = SpanError { span: (start, end), source_len, reason };
        let display = format!("{err}");
        prop_assert!(display.starts_with("Invalid span"));
    }
}

// ---------------------------------------------------------------------------
// 7. SpanError Display contains the span range notation
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_contains_range_notation(
        start in 0usize..=5000,
        end in 0usize..=5000,
        source_len in 0usize..=5000,
        reason in span_error_reason_strategy(),
    ) {
        let err = SpanError { span: (start, end), source_len, reason };
        let display = format!("{err}");
        let expected_range = format!("{start}..{end}");
        prop_assert!(display.contains(&expected_range),
            "Expected display to contain '{}', got: {}", expected_range, display);
    }
}

// ---------------------------------------------------------------------------
// 8. SpanError implements std::error::Error
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_is_std_error(
        start in 0usize..=5000,
        end in 0usize..=5000,
        source_len in 0usize..=5000,
        reason in span_error_reason_strategy(),
    ) {
        let err = SpanError { span: (start, end), source_len, reason };
        // Verify it implements std::error::Error by calling the trait method
        let err_ref: &dyn std::error::Error = &err;
        let _ = err_ref.to_string();
        // source() should be None for SpanError
        prop_assert!(err_ref.source().is_none());
    }
}

// ---------------------------------------------------------------------------
// 9. ParseErrorReason::UnexpectedToken Debug contains token text
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parse_error_reason_unexpected_token_debug_contains_text(
        token in token_strategy(),
    ) {
        let reason = ParseErrorReason::UnexpectedToken(token.clone());
        let dbg = format!("{reason:?}");
        prop_assert!(dbg.contains(&token),
            "Debug should contain token '{}', got: {}", token, dbg);
        prop_assert!(dbg.contains("UnexpectedToken"));
    }
}

// ---------------------------------------------------------------------------
// 10. ParseErrorReason::MissingToken Debug contains token name
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parse_error_reason_missing_token_debug_contains_text(
        token in token_strategy(),
    ) {
        let reason = ParseErrorReason::MissingToken(token.clone());
        let dbg = format!("{reason:?}");
        prop_assert!(dbg.contains(&token),
            "Debug should contain token '{}', got: {}", token, dbg);
        prop_assert!(dbg.contains("MissingToken"));
    }
}

// ---------------------------------------------------------------------------
// 11. ParseErrorReason::FailedNode Debug contains "FailedNode"
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parse_error_reason_failed_node_debug(
        count in 0usize..=5,
    ) {
        let inner: Vec<ParseError> = (0..count)
            .map(|i| ParseError {
                reason: ParseErrorReason::UnexpectedToken(format!("tok{i}")),
                start: i,
                end: i + 1,
            })
            .collect();
        let reason = ParseErrorReason::FailedNode(inner);
        let dbg = format!("{reason:?}");
        prop_assert!(dbg.contains("FailedNode"));
    }
}

// ---------------------------------------------------------------------------
// 12. ParseError Debug contains start and end positions
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parse_error_debug_contains_positions(
        start in 0usize..=10000,
        end in 0usize..=10000,
        token in token_strategy(),
    ) {
        let err = ParseError {
            reason: ParseErrorReason::UnexpectedToken(token),
            start,
            end,
        };
        let dbg = format!("{err:?}");
        prop_assert!(dbg.contains(&start.to_string()));
        prop_assert!(dbg.contains(&end.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 13. ParseError Debug never panics
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parse_error_debug_never_panics(
        start in 0usize..=usize::MAX / 2,
        end in 0usize..=usize::MAX / 2,
        token in ".*",
    ) {
        let err = ParseError {
            reason: ParseErrorReason::UnexpectedToken(token),
            start,
            end,
        };
        let _ = format!("{err:?}");
    }
}

// ---------------------------------------------------------------------------
// 14. ReportingParseError Display contains line:column
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_display_contains_line_column(
        line in 1usize..=10000,
        column in 1usize..=10000,
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: None,
            expected: vec![],
            context: String::new(),
        };
        let display = format!("{err}");
        let pos = format!("{line}:{column}");
        prop_assert!(display.contains(&pos),
            "Expected '{}' in: {}", pos, display);
    }
}

// ---------------------------------------------------------------------------
// 15. ReportingParseError Display with unexpected token
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_display_unexpected_token(
        line in 1usize..=1000,
        column in 1usize..=1000,
        token in token_strategy(),
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: Some(token.clone()),
            expected: vec![],
            context: String::new(),
        };
        let display = format!("{err}");
        prop_assert!(display.contains(&token),
            "Expected token '{}' in: {}", token, display);
        prop_assert!(display.contains("unexpected token"));
    }
}

// ---------------------------------------------------------------------------
// 16. ReportingParseError Display without token says "unexpected end of input"
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn reporting_error_display_end_of_input(
        line in 1usize..=1000,
        column in 1usize..=1000,
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: None,
            expected: vec![],
            context: String::new(),
        };
        let display = format!("{err}");
        prop_assert!(display.contains("unexpected end of input"),
            "Expected 'unexpected end of input' in: {}", display);
    }
}

// ---------------------------------------------------------------------------
// 17. ReportingParseError Display with expected tokens
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_display_expected_tokens(
        line in 1usize..=1000,
        column in 1usize..=1000,
        expected in prop::collection::vec(token_strategy(), 1..=5),
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: None,
            expected: expected.clone(),
            context: String::new(),
        };
        let display = format!("{err}");
        prop_assert!(display.contains("expected one of:"),
            "Expected 'expected one of:' in: {}", display);
        for tok in &expected {
            prop_assert!(display.contains(tok.as_str()),
                "Expected token '{}' in: {}", tok, display);
        }
    }
}

// ---------------------------------------------------------------------------
// 18. ReportingParseError Display with context
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_display_with_context(
        line in 1usize..=1000,
        column in 1usize..=1000,
        ctx in "[a-z ]{1,40}",
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: None,
            expected: vec![],
            context: ctx.clone(),
        };
        let display = format!("{err}");
        prop_assert!(display.contains(&ctx),
            "Expected context '{}' in: {}", ctx, display);
        // Context is wrapped in parentheses
        let wrapped = format!("({ctx})");
        prop_assert!(display.contains(&wrapped),
            "Expected '({})' in: {}", ctx, display);
    }
}

// ---------------------------------------------------------------------------
// 19. ReportingParseError Display empty context is not shown
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn reporting_error_no_empty_context_parens(
        line in 1usize..=1000,
        column in 1usize..=1000,
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: None,
            expected: vec![],
            context: String::new(),
        };
        let display = format!("{err}");
        // Should not contain empty context markers
        prop_assert!(!display.contains("()"),
            "Should not contain empty parens: {}", display);
    }
}

// ---------------------------------------------------------------------------
// 20. ReportingParseError Display never panics
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_display_never_panics(
        line in 0usize..=usize::MAX / 2,
        column in 0usize..=usize::MAX / 2,
        token in proptest::option::of(token_strategy()),
        expected in expected_tokens_strategy(),
        context in context_strategy(),
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: token,
            expected,
            context,
        };
        let _ = format!("{err}");
    }
}

// ---------------------------------------------------------------------------
// 21. ReportingParseError Display is non-empty
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_display_is_non_empty(
        line in 1usize..=1000,
        column in 1usize..=1000,
        token in proptest::option::of(token_strategy()),
        expected in expected_tokens_strategy(),
        context in context_strategy(),
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: token,
            expected,
            context,
        };
        let display = format!("{err}");
        prop_assert!(!display.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 22. ReportingParseError Display starts with "Parse error at"
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_display_starts_with_prefix(
        line in 1usize..=1000,
        column in 1usize..=1000,
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: None,
            expected: vec![],
            context: String::new(),
        };
        let display = format!("{err}");
        prop_assert!(display.starts_with("Parse error at"),
            "Expected prefix 'Parse error at' in: {}", display);
    }
}

// ---------------------------------------------------------------------------
// 23. Multiple parse errors Debug independently
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn multiple_parse_errors_debug_independently(
        count in 1usize..=10,
        base_start in 0usize..=1000,
    ) {
        let errors: Vec<ParseError> = (0..count)
            .map(|i| ParseError {
                reason: ParseErrorReason::UnexpectedToken(format!("tok_{i}")),
                start: base_start + i * 10,
                end: base_start + i * 10 + 5,
            })
            .collect();

        for (i, err) in errors.iter().enumerate() {
            let dbg = format!("{err:?}");
            let token = format!("tok_{i}");
            prop_assert!(dbg.contains(&token),
                "Error {} debug should contain '{}', got: {}", i, token, dbg);
        }
    }
}

// ---------------------------------------------------------------------------
// 24. Nested FailedNode preserves inner error info
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn nested_failed_node_preserves_info(
        inner_token in token_strategy(),
        outer_start in 0usize..=1000,
        outer_end in 0usize..=1000,
    ) {
        let inner = ParseError {
            reason: ParseErrorReason::UnexpectedToken(inner_token.clone()),
            start: 0,
            end: 1,
        };
        let outer = ParseError {
            reason: ParseErrorReason::FailedNode(vec![inner]),
            start: outer_start,
            end: outer_end,
        };
        let dbg = format!("{outer:?}");
        prop_assert!(dbg.contains(&inner_token),
            "Nested debug should contain inner token '{}', got: {}", inner_token, dbg);
        prop_assert!(dbg.contains("FailedNode"));
        prop_assert!(dbg.contains("UnexpectedToken"));
    }
}

// ---------------------------------------------------------------------------
// 25. ReportingParseError Display with all fields populated
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_full_display(
        line in 1usize..=1000,
        column in 1usize..=1000,
        token in token_strategy(),
        expected in prop::collection::vec(token_strategy(), 1..=3),
        ctx in "[a-z ]{1,20}",
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: Some(token.clone()),
            expected: expected.clone(),
            context: ctx.clone(),
        };
        let display = format!("{err}");

        // All parts should be present
        let pos = format!("{line}:{column}");
        prop_assert!(display.contains(&pos));
        prop_assert!(display.contains(&token));
        prop_assert!(display.contains("expected one of:"));
        let wrapped = format!("({})", ctx);
        prop_assert!(display.contains(&wrapped));
    }
}

// ---------------------------------------------------------------------------
// 26. SpanError Debug output is non-empty and contains type name
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_debug_contains_type_name(
        start in 0usize..=5000,
        end in 0usize..=5000,
        source_len in 0usize..=5000,
        reason in span_error_reason_strategy(),
    ) {
        let err = SpanError { span: (start, end), source_len, reason };
        let dbg = format!("{err:?}");
        prop_assert!(dbg.contains("SpanError"));
    }
}

// ---------------------------------------------------------------------------
// 27. SpanErrorReason Debug output matches variant name
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_reason_debug_matches_variant(
        reason in span_error_reason_strategy(),
    ) {
        let dbg = format!("{reason:?}");
        let expected = match reason {
            SpanErrorReason::StartGreaterThanEnd => "StartGreaterThanEnd",
            SpanErrorReason::StartOutOfBounds => "StartOutOfBounds",
            SpanErrorReason::EndOutOfBounds => "EndOutOfBounds",
        };
        prop_assert!(dbg.contains(expected));
    }
}

// ---------------------------------------------------------------------------
// 28. ReportingParseError Display expected tokens are comma-separated
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reporting_error_expected_comma_separated(
        line in 1usize..=100,
        column in 1usize..=100,
        expected in prop::collection::vec(token_strategy(), 2..=5),
    ) {
        let err = ReportingParseError {
            line,
            column,
            unexpected_token: None,
            expected: expected.clone(),
            context: String::new(),
        };
        let display = format!("{err}");
        let joined = expected.join(", ");
        prop_assert!(display.contains(&joined),
            "Expected comma-separated '{}' in: {}", joined, display);
    }
}

// ---------------------------------------------------------------------------
// 29. ParseError with MissingToken Debug includes token
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parse_error_missing_token_debug(
        token in token_strategy(),
        start in 0usize..=1000,
        end in 0usize..=1000,
    ) {
        let err = ParseError {
            reason: ParseErrorReason::MissingToken(token.clone()),
            start,
            end,
        };
        let dbg = format!("{err:?}");
        prop_assert!(dbg.contains(&token));
        prop_assert!(dbg.contains("MissingToken"));
    }
}

// ---------------------------------------------------------------------------
// 30. SpanError Display and Debug are consistent (both non-empty)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_and_debug_both_non_empty(
        start in 0usize..=5000,
        end in 0usize..=5000,
        source_len in 0usize..=5000,
        reason in span_error_reason_strategy(),
    ) {
        let err = SpanError { span: (start, end), source_len, reason };
        let display = format!("{err}");
        let debug = format!("{err:?}");
        prop_assert!(!display.is_empty());
        prop_assert!(!debug.is_empty());
        // Display and Debug should differ (Display is human-readable)
        prop_assert_ne!(display, debug);
    }
}
