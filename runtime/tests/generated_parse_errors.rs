use adze::errors::ParseErrorReason;
use std::ops::Range;

#[test]
fn generated_typed_parser_bad_token_reports_source_span() {
    let source = "1 + @";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("invalid token must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    assert_eq!(
        first.byte_span(),
        4..5,
        "bad token span should point at the invalid `@` byte"
    );

    let span = first.source_span(source.as_bytes());
    assert_eq!(span.start.line, 1);
    assert_eq!(span.start.column, 5);
    assert_eq!(span.end.line, 1);
    assert_eq!(span.end.column, 6);

    assert!(
        matches!(first.reason, ParseErrorReason::UnexpectedToken(_)),
        "bad generated-parser input should report an unexpected token: {:?}",
        first
    );

    let rendered = first.display_with_source(source).to_string();
    assert!(
        rendered.contains("bytes 4..5"),
        "rendered diagnostic should include byte span: {rendered}"
    );
    assert!(
        rendered.contains(source),
        "rendered diagnostic should include source excerpt: {rendered}"
    );
    assert!(
        rendered.contains("    ^"),
        "rendered diagnostic should place a caret under the invalid token: {rendered}"
    );
}

#[test]
fn generated_typed_parser_multibyte_bad_token_reports_utf8_byte_span() {
    let source = "1 + λ";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("invalid multibyte token must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    assert_eq!(
        first.byte_span(),
        4..6,
        "bad token span should cover the full UTF-8 byte width"
    );

    let span = first.source_span(source.as_bytes());
    assert_eq!(span.start.line, 1);
    assert_eq!(span.start.column, 5);
    assert_eq!(span.end.line, 1);
    assert_eq!(span.end.column, 7);

    let rendered = first.display_with_source(source).to_string();
    assert!(
        rendered.contains("bytes 4..6"),
        "rendered diagnostic should include full UTF-8 byte span: {rendered}"
    );
}

#[test]
fn generated_typed_parser_unexpected_eof_reports_zero_width_source_span() {
    let source = "1 +";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("truncated expression must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    assert_eq!(
        first.byte_span(),
        source.len()..source.len(),
        "unexpected EOF should point at the end-of-input insertion point"
    );

    let span = first.source_span(source.as_bytes());
    assert_eq!(span.start.line, 1);
    assert_eq!(span.start.column, source.len() + 1);
    assert_eq!(span.end.line, 1);
    assert_eq!(span.end.column, source.len() + 1);

    let rendered = first.display_with_source(source).to_string();
    assert!(
        rendered.contains("bytes 3..3"),
        "rendered diagnostic should include zero-width byte span: {rendered}"
    );
    assert!(
        rendered.contains("   ^"),
        "rendered diagnostic should place a caret at EOF: {rendered}"
    );
}

#[test]
fn generated_typed_parser_unexpected_eof_lists_expected_tokens() {
    let source = "1 +";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("truncated expression must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    let ParseErrorReason::UnexpectedToken(message) = &first.reason else {
        panic!(
            "truncated generated-parser input should report an unexpected token: {:?}",
            first.reason
        );
    };

    assert!(
        message.contains("expected one of:"),
        "unexpected-token detail should include normalized expected tokens: {message}"
    );
    assert!(
        message.contains(r"/\d+/"),
        "expected-token detail should use generated token names, not raw ids: {message}"
    );
    assert!(
        !message.contains("SymbolId") && !message.contains("symbol ") && !message.contains("_4"),
        "expected-token detail should not expose raw symbol ids or extra-token internals: {message}"
    );

    let rendered = first.display_with_source(source).to_string();
    assert!(
        rendered.contains("expected one of:"),
        "rendered diagnostic should include expected-token context: {rendered}"
    );
    assert!(
        rendered.contains(r"/\d+/"),
        "rendered diagnostic should include the expected token name: {rendered}"
    );
}

#[test]
fn generated_typed_parser_multiline_bad_token_reports_line_column_and_excerpt() {
    let source = "1 +\n@";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("multiline invalid token must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    assert_eq!(
        first.byte_span(),
        4..5,
        "bad token span should point at the invalid token on the second line"
    );

    let span = first.source_span(source.as_bytes());
    assert_eq!(span.start.line, 2);
    assert_eq!(span.start.column, 1);
    assert_eq!(span.end.line, 2);
    assert_eq!(span.end.column, 2);

    let rendered = first.display_with_source(source).to_string();
    assert!(
        rendered.contains("at 2:1 (bytes 4..5)"),
        "rendered diagnostic should include second-line location and byte span: {rendered}"
    );
    assert!(
        rendered.contains("@\n^"),
        "rendered diagnostic should include the second source line and caret: {rendered}"
    );
}

/// Canary: the public diagnostic contract for generated typed-parser errors
/// should not change when the GLR feature is enabled for the runtime crate.
///
/// The product-proof lane runs this exact test under both `pure-rust` and
/// `pure-rust,glr`. Keeping fixed byte spans, line/column positions, expected
/// token names, and rendered byte ranges here gives us a narrow LR/GLR
/// feature-parity receipt without claiming full parse-error stabilization.
#[test]
fn generated_typed_parser_error_contract_is_feature_stable() {
    struct Case {
        label: &'static str,
        source: &'static str,
        byte_span: Range<usize>,
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    }

    let cases = [
        Case {
            label: "unexpected EOF",
            source: "1 +",
            byte_span: 3..3,
            start_line: 1,
            start_column: 4,
            end_line: 1,
            end_column: 4,
        },
        Case {
            label: "invalid ASCII token",
            source: "1 + @",
            byte_span: 4..5,
            start_line: 1,
            start_column: 5,
            end_line: 1,
            end_column: 6,
        },
        Case {
            label: "invalid UTF-8 scalar",
            source: "1 + λ",
            byte_span: 4..6,
            start_line: 1,
            start_column: 5,
            end_line: 1,
            end_column: 7,
        },
    ];

    for case in cases {
        let errors = match adze_example::typed_ast_contract::grammar::parse(case.source) {
            Ok(ast) => panic!("{} unexpectedly parsed as {ast:?}", case.label),
            Err(errors) => errors,
        };

        let first = errors
            .first()
            .unwrap_or_else(|| panic!("{} should produce at least one parse error", case.label));

        assert_eq!(
            first.byte_span(),
            case.byte_span,
            "{} should keep its public byte-span contract",
            case.label
        );

        let span = first.source_span(case.source.as_bytes());
        assert_eq!(span.start.line, case.start_line, "{}", case.label);
        assert_eq!(span.start.column, case.start_column, "{}", case.label);
        assert_eq!(span.end.line, case.end_line, "{}", case.label);
        assert_eq!(span.end.column, case.end_column, "{}", case.label);

        assert!(
            !first.expected.is_empty(),
            "{} should expose structured expected-token names",
            case.label
        );
        assert!(
            first.expected.iter().any(|token| token == r"/\d+/"),
            "{} should keep the arithmetic digit token in expected names: {:?}",
            case.label,
            first.expected
        );
        for token in &first.expected {
            assert!(
                !token.contains("SymbolId") && !token.contains("symbol ") && !token.contains('_'),
                "{} should expose human-readable expected names, got {token}",
                case.label
            );
        }

        let rendered = first.display_with_source(case.source).to_string();
        let expected_byte_range = format!("bytes {}..{}", case.byte_span.start, case.byte_span.end);
        assert!(
            rendered.contains(&expected_byte_range),
            "{} should render byte range {expected_byte_range}: {rendered}",
            case.label
        );
    }
}

// ============================================================================
// Structured expected-token field tests
// ============================================================================

#[test]
fn generated_typed_parser_unexpected_eof_expected_field_is_populated() {
    let source = "1 +";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("truncated expression must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    // The structured `expected` field should contain meaningful token names
    assert!(
        !first.expected.is_empty(),
        "expected field should be populated for unexpected EOF, got: {:?}",
        first.expected
    );

    // Token names should be human-readable, not raw IDs
    for name in &first.expected {
        assert!(
            !name.contains("SymbolId"),
            "expected token names should not contain raw SymbolId, got: {name}"
        );
        assert!(
            !name.contains("symbol "),
            "expected token names should not contain 'symbol ' prefix, got: {name}"
        );
    }
}

#[test]
fn generated_typed_parser_unexpected_eof_expected_field_sorted_and_deduped() {
    let source = "1 +";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("truncated expression must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    // The expected list should be sorted
    let mut sorted = first.expected.clone();
    sorted.sort();
    assert_eq!(
        first.expected, sorted,
        "expected field should be sorted: {:?}",
        first.expected
    );

    // The expected list should be deduplicated
    let mut deduped = first.expected.clone();
    deduped.dedup();
    assert_eq!(
        first.expected.len(),
        deduped.len(),
        "expected field should not contain duplicates: {:?}",
        first.expected
    );
}

#[test]
fn generated_typed_parser_bad_token_expected_field_is_populated() {
    let source = "1 + @";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("invalid token must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    // Even for bad tokens, the expected field should be populated with what
    // the parser expected at that position.
    assert!(
        !first.expected.is_empty(),
        "expected field should be populated for bad token, got: {:?}",
        first.expected
    );

    // Token names should be human-readable
    for name in &first.expected {
        assert!(
            !name.contains("SymbolId"),
            "expected token names should not contain raw SymbolId, got: {name}"
        );
    }
}

#[test]
fn generated_typed_parser_expected_field_contains_digit_pattern() {
    let source = "1 +";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("truncated expression must fail through the generated typed parser");

    let first = errors
        .first()
        .expect("generated parser should return at least one parse error");

    // For the arithmetic grammar, EOF at this position should expect a number.
    assert!(
        first.expected.iter().any(|t| t == r"/\d+/"),
        "expected tokens should include a digit pattern for arithmetic expression: {:?}",
        first.expected
    );
}

#[test]
fn generated_typed_parser_bad_inputs_return_errors_without_panicking() {
    let cases = [
        ("empty input", ""),
        ("whitespace only", "   "),
        ("trailing operator", "1 +"),
        ("invalid ascii token", "1 + @"),
        ("invalid utf8 scalar", "1 + λ"),
        ("multiline invalid token", "1 +\n@"),
    ];

    for (label, source) in cases {
        let parsed =
            std::panic::catch_unwind(|| adze_example::typed_ast_contract::grammar::parse(source));

        let errors = match parsed {
            Ok(Err(errors)) => errors,
            Ok(Ok(ast)) => panic!("generated parser unexpectedly accepted {label}: {ast:?}"),
            Err(_) => panic!("generated parser panicked for {label}"),
        };

        assert!(
            !errors.is_empty(),
            "generated parser should return at least one structured error for {label}"
        );
    }
}

/// Canary: prove that generated parser errors expose structured expected-token
/// names (not opaque IDs) end-to-end.
#[test]
fn expected_token_sets_are_reported() {
    // Use a bare operator — the grammar expects a number first.
    let source = "+";
    let errors = adze_example::typed_ast_contract::grammar::parse(source)
        .expect_err("bare operator must fail");

    let first = errors
        .first()
        .expect("should produce at least one parse error");

    // The `expected` vec must be non-empty and contain human-readable names.
    assert!(
        !first.expected.is_empty(),
        "expected field must be populated, got: {:?}",
        first.expected
    );

    // Every entry must be a readable token name, not an opaque internal ID.
    for name in &first.expected {
        assert!(
            !name.contains("SymbolId") && !name.contains('_'),
            "expected token should be a human-readable name, not an opaque ID: {name}"
        );
    }

    // For the arithmetic grammar, at least one expected token must reference
    // the digit pattern — the only terminal that can start an expression.
    assert!(
        first.expected.iter().any(|t| t.contains("d")),
        "expected tokens should include the digit pattern for the arithmetic grammar: {:?}",
        first.expected
    );
}
