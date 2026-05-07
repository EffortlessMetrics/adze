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
