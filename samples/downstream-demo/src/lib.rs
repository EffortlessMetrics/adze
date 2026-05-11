#[adze::grammar("downstream_arithmetic")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq)]
    pub enum Expr {
        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),

        #[adze::prec_left(2)]
        Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
    }

    #[adze::extra]
    #[allow(dead_code)]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s+")]
        _ws: (),
    }
}

#[cfg(test)]
mod tests {
    use super::grammar::{self, Expr};

    #[test]
    fn downstream_demo_parses_typed_ast_with_precedence() {
        let expr = grammar::parse("1 + 2 * 3").expect("expression should parse");

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

    #[test]
    fn downstream_demo_reports_expected_tokens_for_bad_input() {
        let source = "1 + @";
        let errors = grammar::parse(source).expect_err("bad input should fail clearly");
        let first = errors
            .first()
            .expect("parse should report at least one error");

        assert_eq!(first.byte_span(), 4..5);
        assert!(
            first.expected.iter().any(|name| name == r"/\d+/"),
            "expected token set should name the number token, got {:?}",
            first.expected
        );

        let rendered = first.display_with_source(source).to_string();
        assert!(rendered.contains("bytes 4..5"));
        assert!(rendered.contains("expected one of:"));
        assert!(rendered.contains("    ^"));
    }
}
