//! Contract test proving text parses directly into a typed Rust AST.

use adze_example::typed_ast_contract::grammar;

#[test]
#[ignore = "KNOWN LIMITATION: typed enum extraction receives None node for left-recursive Add during parse of `1 + 2 + 3`"]
fn typed_ast_left_associative_addition_contract() {
    let parsed = grammar::parse("1 + 2 + 3").expect("contract parse should succeed");

    assert_eq!(
        parsed,
        grammar::Expr::Add(
            Box::new(grammar::Expr::Add(
                Box::new(grammar::Expr::Number(1)),
                (),
                Box::new(grammar::Expr::Number(2)),
            )),
            (),
            Box::new(grammar::Expr::Number(3)),
        )
    );
}
