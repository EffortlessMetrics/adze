// Simplified Python grammar for testing pure-rust implementation
#[rust_sitter::grammar("python_simple")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Module {
        #[rust_sitter::repeat(non_empty = false)]
        pub body: Vec<Statement>,
    }

    #[rust_sitter::language]
    pub enum Statement {
        Expression(ExpressionStatement),
        Assignment(AssignmentStatement),
    }

    #[rust_sitter::language]
    pub struct ExpressionStatement {
        pub expression: Expression,
    }

    #[rust_sitter::language]
    pub struct AssignmentStatement {
        pub target: Identifier,
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub value: Expression,
    }

    #[rust_sitter::language]
    #[rust_sitter::prec(1)]
    pub enum Expression {
        #[rust_sitter::prec(3)]
        Primary(PrimaryExpression),
        #[rust_sitter::prec_left(1)]
        Add(Box<Expression>, #[rust_sitter::leaf(text = "+")] (), Box<Expression>),
        #[rust_sitter::prec_left(2)]
        Multiply(Box<Expression>, #[rust_sitter::leaf(text = "*")] (), Box<Expression>),
    }

    #[rust_sitter::language]
    pub enum PrimaryExpression {
        Number(NumberLiteral),
        String(StringLiteral),
        Identifier(Identifier),
    }

    #[rust_sitter::language]
    pub struct NumberLiteral {
        #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse::<i32>().unwrap())]
        pub value: i32,
    }

    #[rust_sitter::language]
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""[^"]*"|'[^']*'"#)]
        pub value: String,
    }

    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

pub use grammar::*;


#[cfg(test)]
mod tests {
    use super::*;

    fn print_tree(node: &rust_sitter::pure_parser::ParsedNode, source: &[u8], indent: usize) {
        let text = std::str::from_utf8(&source[node.start_byte..node.end_byte]).unwrap_or("<invalid>");
        eprintln!("{:indent$}Node: symbol={} kind='{}' range={}..{} text='{}'",
            "", node.symbol, node.kind(), node.start_byte, node.end_byte, text, indent = indent);
        for child in &node.children {
            print_tree(child, source, indent + 2);
        }
    }

    #[test]
    fn test_primary_expression() {
        use rust_sitter::Extract;
        
        // First, let's debug what symbols are available
        eprintln!("\n=== Available symbols in language ===");
        let lang = language();
        unsafe {
            let symbol_count = lang.symbol_count;
            eprintln!("Total symbols: {}", symbol_count);
            
            // Print first 50 symbol names
            let symbol_names = std::slice::from_raw_parts(lang.symbol_names, symbol_count.min(50) as usize);
            for (i, &name_ptr) in symbol_names.iter().enumerate() {
                if !name_ptr.is_null() {
                    let c_str = std::ffi::CStr::from_ptr(name_ptr as *const i8);
                    let name = c_str.to_string_lossy();
                    eprintln!("  Symbol {}: '{}'", i, name);
                }
            }
        }
        
        // Test parsing "42" as a PrimaryExpression
        let input = "42";
        
        // Parse with debug output
        use rust_sitter::pure_parser::Parser;
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        let parse_result = parser.parse_bytes(input.as_bytes());
        
        eprintln!("\n=== Parse result ===");
        eprintln!("Has root: {}", parse_result.root.is_some());
        eprintln!("Errors: {:?}", parse_result.errors);
        
        if let Some(tree) = &parse_result.root {
            eprintln!("\n=== Parse tree ===");
            print_tree(tree, input.as_bytes(), 0);
        } else {
            eprintln!("\n=== Parse failed ===");
        }
        
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse '42'");
        
        let module = result.unwrap();
        
        // The module should contain a single expression statement
        assert_eq!(module.body.len(), 1, "Expected 1 statement in module body");
        
        match &module.body[0] {
            Statement::Expression(expr_stmt) => {
                // The expression should be a primary expression
                match &expr_stmt.expression {
                    Expression::Primary(primary) => {
                        // The primary expression should be a Number variant
                        eprintln!("DEBUG: Successfully extracted primary expression");
                        match primary {
                            PrimaryExpression::Number(num) => {
                                eprintln!("DEBUG: Found Number variant with value: {}", num.value);
                                assert_eq!(num.value, 42, "Expected number value to be 42");
                            },
                            PrimaryExpression::String(s) => {
                                panic!("Expected Number variant but got String: {:?}", s.value);
                            },
                            PrimaryExpression::Identifier(id) => {
                                panic!("Expected Number variant but got Identifier: {}", id.name);
                            }
                        }
                    },
                    _ => panic!("Expected Primary expression but got something else"),
                }
            },
            _ => panic!("Expected Expression statement but got something else"),
        }
    }

    #[test]
    fn test_extract_string() {
        use rust_sitter::Extract;
        
        // Test parsing "hello" as a string literal
        let input = r#""hello""#;
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse string");
        
        let module = result.unwrap();
        assert_eq!(module.body.len(), 1, "Expected 1 statement in module body");
        
        match &module.body[0] {
            Statement::Expression(expr_stmt) => {
                match &expr_stmt.expression {
                    Expression::Primary(primary) => {
                        match primary {
                            PrimaryExpression::String(s) => {
                                assert_eq!(s.value, "\"hello\"", "Expected string value to be \"hello\"");
                            },
                            _ => panic!("Expected String variant"),
                        }
                    },
                    _ => panic!("Expected Primary expression"),
                }
            },
            _ => panic!("Expected Expression statement"),
        }
    }

    #[test]
    fn test_extract_identifier() {
        use rust_sitter::Extract;
        
        // Test parsing "x" as an identifier
        let input = "x";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse identifier");
        
        let module = result.unwrap();
        assert_eq!(module.body.len(), 1, "Expected 1 statement in module body");
        
        match &module.body[0] {
            Statement::Expression(expr_stmt) => {
                match &expr_stmt.expression {
                    Expression::Primary(primary) => {
                        match primary {
                            PrimaryExpression::Identifier(id) => {
                                assert_eq!(id.name, "x", "Expected identifier name to be 'x'");
                            },
                            _ => panic!("Expected Identifier variant"),
                        }
                    },
                    _ => panic!("Expected Primary expression"),
                }
            },
            _ => panic!("Expected Expression statement"),
        }
    }

    #[test]
    fn test_simple_addition() {
        let input = "1 + 2";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse simple addition");
    }

    #[test]
    fn test_operator_precedence() {
        let input = "1 + 2 * 3";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse expression with precedence");
        // Should parse as 1 + (2 * 3), not (1 + 2) * 3
    }

    #[test]
    fn test_assignment() {
        let input = "x = 42";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse assignment");
    }
}


