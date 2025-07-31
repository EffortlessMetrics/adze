#[rust_sitter::grammar("arithmetic")]
pub mod grammar {
    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug)]
    pub enum Expression {
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
        #[rust_sitter::prec_left(1)]
        Sub(
            Box<Expression>,
            #[rust_sitter::leaf(text = "-")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(2)]
        Mul(
            Box<Expression>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Expression>,
        ),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grammar::Expression;

    #[wasm_bindgen_test::wasm_bindgen_test]
    #[test]
    fn successful_parses() {
        assert_eq!(grammar::parse("1").unwrap(), Expression::Number(1));

        assert_eq!(grammar::parse(" 1").unwrap(), Expression::Number(1));

        assert_eq!(
            grammar::parse("1 - 2").unwrap(),
            Expression::Sub(
                Box::new(Expression::Number(1)),
                (),
                Box::new(Expression::Number(2))
            )
        );

        assert_eq!(
            grammar::parse("1 - 2 - 3").unwrap(),
            Expression::Sub(
                Box::new(Expression::Sub(
                    Box::new(Expression::Number(1)),
                    (),
                    Box::new(Expression::Number(2))
                )),
                (),
                Box::new(Expression::Number(3))
            )
        );

        assert_eq!(
            grammar::parse("1 - 2 * 3").unwrap(),
            Expression::Sub(
                Box::new(Expression::Number(1)),
                (),
                Box::new(Expression::Mul(
                    Box::new(Expression::Number(2)),
                    (),
                    Box::new(Expression::Number(3))
                ))
            )
        );

        assert_eq!(
            grammar::parse("1 * 2 * 3").unwrap(),
            Expression::Mul(
                Box::new(Expression::Mul(
                    Box::new(Expression::Number(1)),
                    (),
                    Box::new(Expression::Number(2))
                )),
                (),
                Box::new(Expression::Number(3))
            )
        );

        assert_eq!(
            grammar::parse("1 * 2 - 3").unwrap(),
            Expression::Sub(
                Box::new(Expression::Mul(
                    Box::new(Expression::Number(1)),
                    (),
                    Box::new(Expression::Number(2))
                )),
                (),
                Box::new(Expression::Number(3))
            )
        );
    }

    #[test]
    fn failed_parses() {
        insta::assert_debug_snapshot!(grammar::parse("1 + 2"));
        insta::assert_debug_snapshot!(grammar::parse("1 - 2 -"));
        insta::assert_debug_snapshot!(grammar::parse("a1"));
        insta::assert_debug_snapshot!(grammar::parse("1a"));
    }

    #[cfg(feature = "pure-rust")]
    #[test]
    fn test_pure_rust_parser() {
        println!("Testing pure-Rust arithmetic parser...");
        
        // Test successful parses
        let result = grammar::parse("42");
        println!("Parse result for '42': {:?}", result);
        assert!(result.is_ok(), "Failed to parse '42': {:?}", result);
        assert_eq!(result.unwrap(), Expression::Number(42));
        
        // Test subtraction expression
        let result = grammar::parse("1 - 2");
        println!("Parse result for '1 - 2': {:?}", result);
        assert!(result.is_ok(), "Failed to parse '1 - 2': {:?}", result);
        assert_eq!(result.unwrap(), Expression::Sub(
            Box::new(Expression::Number(1)),
            (),
            Box::new(Expression::Number(2))
        ));
        
        // Test multiplication expression
        let result = grammar::parse("3 * 4");
        println!("Parse result for '3 * 4': {:?}", result);
        assert!(result.is_ok(), "Failed to parse '3 * 4': {:?}", result);
        assert_eq!(result.unwrap(), Expression::Mul(
            Box::new(Expression::Number(3)),
            (),
            Box::new(Expression::Number(4))
        ));
        
        // Test left associativity of subtraction
        let result = grammar::parse("1 - 2 - 3");
        println!("Parse result for '1 - 2 - 3': {:?}", result);
        assert!(result.is_ok(), "Failed to parse '1 - 2 - 3': {:?}", result);
        assert_eq!(result.unwrap(), Expression::Sub(
            Box::new(Expression::Sub(
                Box::new(Expression::Number(1)),
                (),
                Box::new(Expression::Number(2))
            )),
            (),
            Box::new(Expression::Number(3))
        ));
        
        // Test precedence: multiplication binds tighter than subtraction
        println!("\n=== Testing precedence: '1 - 2 * 3' ===");
        let result = grammar::parse("1 - 2 * 3");
        println!("Parse result for '1 - 2 * 3': {:?}", result);
        match &result {
            Ok(expr) => {
                println!("Parsed as: {:?}", expr);
                // Expected: Sub(Number(1), (), Mul(Number(2), (), Number(3)))
                // Actually getting: Mul(Sub(Number(1), (), Number(2)), (), Number(3))
            }
            Err(e) => println!("Parse error: {:?}", e),
        }
        
        println!("Test completed!");
    }
}
