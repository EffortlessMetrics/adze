// Auto-generated parser code
mod grammar {
    include!(concat!(env!("OUT_DIR"), "/grammar_python_simple/parser_python_simple.rs"));
}

pub use grammar::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imports() {
        let input = "import math";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse simple import");
    }

    #[test]
    fn test_simple_assignment() {
        let input = "x = 42";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse simple assignment");
    }

    #[test]
    fn test_simple_binary_expression() {
        let input = "1 + 2";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse simple binary expression");
    }

    #[test]
    fn test_operator_precedence() {
        let input = "1 + 2 * 3";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse expression with operator precedence");
    }

    #[test]
    fn test_function_definition() {
        let input = "def add(a, b): return a + b";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse function definition");
    }

    #[test]
    fn test_empty_list() {
        let input = "[]";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse empty list");
    }

    #[test]
    fn test_list_with_elements() {
        let input = "[1, 2, 3]";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse list with elements");
    }

    #[test]
    fn test_mixed_operators() {
        let input = "a + b * c - d / e";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse expression with mixed operators");
    }

    #[test]
    fn test_string_literal() {
        let input = r#""hello world""#;
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse string literal");
    }

    #[test]
    fn test_single_quoted_string() {
        let input = "'hello world'";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse single-quoted string");
    }

    #[test]
    fn test_primary_expression() {
        use rust_sitter::Extract;
        
        // Test parsing "42" as a PrimaryExpression
        let input = "42";
        let result = grammar::parse(input);
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
                            },
                            _ => {
                                panic!("Expected Number variant but got something else");
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
        let result = grammar::parse(input);
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
        let result = grammar::parse(input);
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
    fn test_function_call_precedence() {
        // Test that a + b(c) is parsed as a + (b(c)), not (a + b)(c)
        let input = "a + b(c)";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse 'a + b(c)'");
        
        // Test empty function call
        let input2 = "func()";
        let result2 = grammar::parse(input2);
        assert!(result2.is_ok(), "Failed to parse 'func()'");
        
        // Test with multiple arguments
        let input3 = "func(x, y, z)";
        let result3 = grammar::parse(input3);
        assert!(result3.is_ok(), "Failed to parse 'func(x, y, z)'");
    }

    #[test]
    fn test_if_statement() {
        let input = "if x > 0: print(x)";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse if statement");
    }

    #[test]
    fn test_while_loop() {
        let input = "while x > 0: x = x - 1";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse while loop");
    }

    #[test]
    fn test_for_loop() {
        let input = "for i in range(10): print(i)";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse for loop");
    }

    #[test]
    fn test_class_definition() {
        let input = "class MyClass: pass";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse class definition");
    }

    #[test]
    fn test_attribute_access() {
        let input = "obj.attribute";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse attribute access");
    }

    #[test]
    fn test_method_call() {
        let input = "obj.method()";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse method call");
    }

    #[test]
    fn test_assignment_with_expression() {
        let input = "x = a + b * c";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse assignment with expression");
    }

    #[test]
    fn test_dictionary() {
        let input = "{'key': 'value'}";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse dictionary");
    }

    #[test]
    fn test_comparison_operators() {
        let input = "a < b and c >= d or e == f";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse comparison operators");
    }

    #[test]
    fn test_list_comprehension() {
        let input = "[x * 2 for x in range(10)]";
        let result = grammar::parse(input);
        assert!(result.is_ok(), "Failed to parse list comprehension");
    }

    #[test]
    fn test_complex_expression() {
        // Test that a + b(c) is parsed as a + (b(c)), not (a + b)(c)
        let input2 = "a + b(c)";
        let result2 = grammar::parse(input2);
        assert!(result2.is_ok(), "Failed to parse 'a + b(c)'");
        
        // Test empty function call
        let input3 = "func()";
        let result3 = grammar::parse(input3);
        assert!(result3.is_ok(), "Failed to parse 'func()'");
        
        // Test operator precedence: a + b * c should be a + (b * c)
        let input3 = "a + b * c";
        let result3 = grammar::parse(input3);
        assert!(result3.is_ok(), "Failed to parse 'a + b * c'");
    }
}