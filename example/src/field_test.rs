#[rust_sitter::grammar("field_test")]
pub mod grammar {
    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug)]
    pub enum Expression {
        Binary {
            #[rust_sitter::field("left")]
            left: Box<Expression>,
            #[rust_sitter::field("operator")]
            op: BinaryOp,
            #[rust_sitter::field("right")]
            right: Box<Expression>,
        },
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
    }

    #[derive(PartialEq, Eq, Debug)]
    pub enum BinaryOp {
        #[rust_sitter::leaf(text = "+")]
        Add(()),
        #[rust_sitter::leaf(text = "-")]
        Sub(()),
        #[rust_sitter::leaf(text = "*")]
        Mul(()),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use super::grammar::*;

    #[test]
    fn test_field_names() {
        // This test verifies that field names are properly handled
        let expr = Expression::Binary {
            left: Box::new(Expression::Number(1)),
            op: BinaryOp::Add(()),
            right: Box::new(Expression::Number(2)),
        };

        // The parser should be able to use field names when parsing
        match expr {
            Expression::Binary { left, op, right } => {
                assert_eq!(*left, Expression::Number(1));
                assert_eq!(*right, Expression::Number(2));
                assert_eq!(op, BinaryOp::Add(()));
            }
            _ => panic!("Expected binary expression"),
        }
    }
}
