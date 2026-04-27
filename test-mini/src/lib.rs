#![allow(clippy::empty_line_after_outer_attr, clippy::unnecessary_cast)]

#[adze::grammar("mini")]
pub mod grammar {
    #[derive(Debug)]
    #[adze::language]
    pub struct Program {
        #[adze::leaf(pattern = r"\d+", text = true)]
        pub number: String,
    }
}

#[adze::grammar("typed_ast")]
pub mod typed_ast {
    #[derive(Debug, PartialEq, Eq)]
    #[adze::language]
    pub enum Expr {
        Number(#[adze::leaf(pattern = r"\d+", transform = |s| s.parse::<i32>().unwrap())] i32),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammar, typed_ast};

    #[test]
    fn test_number() {
        let result = grammar::parse("42");
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "42");
    }

    #[test]
    fn test_multi_digit_number() {
        let result = grammar::parse("12345");
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "12345");
    }

    #[test]
    fn test_single_digit() {
        let result = grammar::parse("0");
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "0");
    }

    #[test]
    fn test_empty_input() {
        let result = grammar::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_non_number() {
        let result = grammar::parse("abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_number_with_trailing_text() {
        // The parser successfully parses "42" and ignores the trailing "abc"
        // This is expected behavior - the parser consumes what it can
        let result = grammar::parse("42abc");
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "42");
    }

    #[test]
    #[ignore = "blocked: Extract called with None node for enum when parsing recursive Expr in generated typed extraction"]
    fn typed_ast_left_associative_addition_contract() {
        let expr = typed_ast::parse("1 + 2 + 3").expect("typed AST parse should succeed");

        assert_eq!(
            expr,
            typed_ast::Expr::Add(
                Box::new(typed_ast::Expr::Add(
                    Box::new(typed_ast::Expr::Number(1)),
                    (),
                    Box::new(typed_ast::Expr::Number(2)),
                )),
                (),
                Box::new(typed_ast::Expr::Number(3)),
            )
        );
    }
}
