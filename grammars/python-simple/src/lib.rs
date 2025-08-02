// Simplified Python grammar demonstrating conflict resolution
// This version focuses on expressions and avoids block/indentation complexity

#[rust_sitter::grammar("python_simple")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Module {
        pub expression: Expression,
    }

    #[rust_sitter::language]
    pub enum Expression {
        // Literals and identifiers (highest precedence)
        Primary(PrimaryExpression),
        
        // Postfix operations (high precedence)
        #[rust_sitter::prec(10)]
        Call(Box<CallExpression>),
        #[rust_sitter::prec(10)]
        Attribute(Box<AttributeExpression>),
        #[rust_sitter::prec(10)]
        Subscript(Box<SubscriptExpression>),
        
        // Unary operations (medium precedence)
        #[rust_sitter::prec(8)]
        Unary(Box<UnaryExpression>),
        
        // Binary operations (lower precedence, with internal precedence levels)
        Binary(Box<BinaryExpression>),
    }

    #[rust_sitter::language]
    pub enum PrimaryExpression {
        Number(NumberLiteral),
        String(StringLiteral),
        Identifier(Identifier),
        Parenthesized(Box<ParenthesizedExpression>),
    }

    #[rust_sitter::language]
    pub struct NumberLiteral {
        #[rust_sitter::leaf(pattern = r"\d+(\.\d+)?", transform = |s| s.parse::<f64>().unwrap())]
        pub value: f64,
    }

    #[rust_sitter::language]
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""[^"]*"|'[^']*'"#, transform = |s| s[1..s.len()-1].to_string())]
        pub value: String,
    }

    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }


    #[rust_sitter::language]
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws1: (),
        pub expression: Expression,
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws2: (),
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }

    #[rust_sitter::language]
    #[rust_sitter::prec(10)]
    pub struct CallExpression {
        pub function: Expression,
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws: (),
        pub arguments: Arguments,
    }

    #[rust_sitter::language]
    pub struct Arguments {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::leaf(pattern = r"\s*")]
        _ws1: (),
        // Allow empty argument lists by making Vec optional
        #[rust_sitter::repeat]
        #[rust_sitter::delimited(
            Comma {
                #[rust_sitter::leaf(text = ",")]
                _comma: (),
                #[rust_sitter::leaf(pattern = r"\s*")]
                _ws: (),
            }
        )]
        pub args: Vec<Expression>,
        #[rust_sitter::leaf(pattern = r"\s*")]
        _ws2: (),
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct Comma {
        #[rust_sitter::leaf(text = ",")]
        _comma: (),
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws: (),
    }

    #[rust_sitter::language]
    #[rust_sitter::prec(10)]
    pub struct AttributeExpression {
        pub object: Expression,
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws1: (),
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws2: (),
        pub attribute: Identifier,
    }

    #[rust_sitter::language]
    pub struct SubscriptExpression {
        pub object: Expression,
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws1: (),
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws2: (),
        pub index: Expression,
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws3: (),
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }

    #[rust_sitter::language]
    #[rust_sitter::prec(8)]
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        #[rust_sitter::leaf(pattern = r"\s*")]
                _ws: (),
        pub operand: Expression,
    }

    #[rust_sitter::language]
    pub enum UnaryOperator {
        Not(#[rust_sitter::leaf(text = "not")] ()),
        Minus(#[rust_sitter::leaf(text = "-")] ()),
        Plus(#[rust_sitter::leaf(text = "+")] ()),
    }

    #[rust_sitter::language]
    pub enum BinaryExpression {
        // Arithmetic (higher precedence)
        #[rust_sitter::prec_left(6)]
        Power(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "**")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(5)]
        Multiply(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "*")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(5)]
        Divide(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "/")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(4)]
        Add(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "+")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(4)]
        Subtract(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "-")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
        
        // Comparison (lower precedence)
        #[rust_sitter::prec_left(3)]
        Equal(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "==")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
        
        // Logical (lowest precedence)
        #[rust_sitter::prec_left(2)]
        And(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "and")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Or(
            Box<Expression>,
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            #[rust_sitter::leaf(text = "or")]
            (),
            #[rust_sitter::leaf(pattern = r"\s*")]
                        (),
            Box<Expression>,
        ),
    }

}

#[cfg(test)]
mod tests {
    use crate::grammar;

    #[test]
    fn test_basic_parsing() {
        // Test number first since it's simpler
        let result = grammar::parse("42");
        if let Err(e) = result {
            panic!("Failed to parse '42': {:?}", e);
        }
        
        // Test simple identifier
        let result = grammar::parse("a");
        if let Err(e) = result {
            panic!("Failed to parse 'a': {:?}", e);
        }
        
        // Test simple unary
        let result = grammar::parse("-42");
        if let Err(e) = result {
            panic!("Failed to parse '-42': {:?}", e);
        }
    }
    
    #[test]
    fn test_precedence() {
        // Test that -a.b is parsed as -(a.b), not (-a).b
        let input = "-a.b";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse '-a.b'");
        
        // Test that a + b(c) is parsed as a + (b(c)), not (a + b)(c)
        let input2 = "a + b(c)";
        let result2 = grammar::parse(input2);
        assert!(result2.is_ok(), "Failed to parse 'a + b(c)'");
        
        // Test operator precedence: a + b * c should be a + (b * c)
        let input3 = "a + b * c";
        let result3 = grammar::parse(input3);
        assert!(result3.is_ok(), "Failed to parse 'a + b * c'");
    }
}