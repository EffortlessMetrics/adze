#[test]
#[ignore = "Known typed extraction gap: parse succeeds but extraction panics with `Extract called with None node for enum` for this minimal left-recursive contract"]
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
