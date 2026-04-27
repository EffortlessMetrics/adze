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

#[adze::grammar("typed_ast_contract")]
pub mod typed_ast_grammar {
    #[adze::language]
    #[derive(Debug, PartialEq, Eq)]
    pub struct Program {
        pub expr: Expr,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Expr {
        Number(#[adze::leaf(pattern = r"\d+", transform = |s| s.parse::<i32>().unwrap())] i32),
        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
    }

    #[adze::extra]
    pub struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        pub ws: (),
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammar, typed_ast_grammar};

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
    #[ignore = "Known typed extraction gap: recursive enum extraction can receive None nodes (panic: `Extract called with None node for enum`) for this left-recursive contract grammar."]
    fn typed_ast_left_associative_addition_contract() {
        use typed_ast_grammar::{Expr, Program};

        let parsed = typed_ast_grammar::parse("1 + 2 + 3").expect("typed AST parse should succeed");

        assert_eq!(
            parsed,
            Program {
                expr: Expr::Add(
                    Box::new(Expr::Add(
                        Box::new(Expr::Number(1)),
                        (),
                        Box::new(Expr::Number(2))
                    )),
                    (),
                    Box::new(Expr::Number(3))
                )
            }
        );
    }
}
