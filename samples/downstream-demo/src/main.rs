use downstream_demo::grammar::{self, Expr};

fn main() {
    let input = "1 + 2 * 3";
    let expr = grammar::parse(input).expect("demo input should parse");

    println!("input: {input}");
    println!("typed AST: {expr:?}");

    assert_eq!(
        expr,
        Expr::Add(
            Box::new(Expr::Number(1)),
            (),
            Box::new(Expr::Mul(
                Box::new(Expr::Number(2)),
                (),
                Box::new(Expr::Number(3)),
            )),
        )
    );
}
