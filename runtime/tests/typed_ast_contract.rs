#[test]
fn typed_ast_contract_left_associative_addition() {
    let parsed = adze_example::typed_ast_contract::grammar::parse("1 + 2 + 3")
        .expect("input should parse into Expr");

    assert_eq!(
        parsed,
        adze_example::typed_ast_contract::grammar::Expr::Add(
            Box::new(adze_example::typed_ast_contract::grammar::Expr::Add(
                Box::new(adze_example::typed_ast_contract::grammar::Expr::Number(1)),
                (),
                Box::new(adze_example::typed_ast_contract::grammar::Expr::Number(2)),
            )),
            (),
            Box::new(adze_example::typed_ast_contract::grammar::Expr::Number(3)),
        )
    );
}

#[test]
fn typed_ast_contract_repeated_parse_is_deterministic() {
    let source = "1 + 2 + 3";
    let expected = adze_example::typed_ast_contract::grammar::parse(source)
        .expect("baseline parse should succeed");

    for attempt in 0..16 {
        let parsed = adze_example::typed_ast_contract::grammar::parse(source)
            .unwrap_or_else(|errors| panic!("parse attempt {attempt} failed: {errors:?}"));

        assert_eq!(
            parsed, expected,
            "parse attempt {attempt} should return the same typed AST"
        );
    }
}

#[test]
fn typed_ast_contract_parse_document_ast_matches_parse() {
    let source = "1 + 2 + 3";
    let expected = adze_example::typed_ast_contract::grammar::parse(source)
        .expect("baseline parse should succeed");
    let document = adze_example::typed_ast_contract::grammar::parse_document(source)
        .expect("document parse should succeed");

    let parsed_from_document: adze_example::typed_ast_contract::grammar::Expr = document
        .ast()
        .expect("document should extract typed AST from its selected tree");

    assert_eq!(parsed_from_document, expected);
}

#[test]
fn typed_ast_contract_parse_document_ast_reports_document_diagnostics() {
    let source = "1 +";
    let document = adze_example::typed_ast_contract::grammar::parse_document(source)
        .expect("document parse should return partial parse facts");
    let diagnostic = document
        .diagnostics()
        .first()
        .expect("bad input should produce a document diagnostic");

    let errors = document
        .ast::<adze_example::typed_ast_contract::grammar::Expr>()
        .expect_err("typed AST extraction should reject diagnostic documents");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].start, diagnostic.start_byte);
    assert_eq!(errors[0].end, diagnostic.end_byte);
    assert_eq!(errors[0].expected, diagnostic.expected);
}
