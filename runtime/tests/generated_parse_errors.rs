use adze::errors::ParseErrorReason;

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
