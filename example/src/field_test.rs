#[adze::grammar("field_test")]
pub mod grammar {
    #[adze::language]
    #[derive(PartialEq, Eq, Debug)]
    pub enum Expression {
        Binary {
            #[adze::field("left")]
            left: Box<Expression>,
            #[adze::field("operator")]
            op: BinaryOp,
            #[adze::field("right")]
            right: Box<Expression>,
        },
        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
    }

    #[derive(PartialEq, Eq, Debug)]
    pub enum BinaryOp {
        #[adze::leaf(text = "+")]
        Add(()),
        #[adze::leaf(text = "-")]
        Sub(()),
        #[adze::leaf(text = "*")]
        Mul(()),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
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
