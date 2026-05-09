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
