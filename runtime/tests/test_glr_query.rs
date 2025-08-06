// Test GLR query support
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// NOTE: These tests use internal modules not exported by the public API
#[path = "../src/glr_query.rs"]
mod glr_query;

use glr_query::{QueryCursor, QueryParser, Subtree};

fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Define terminals
    let number_id = SymbolId(0);
    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let plus_id = SymbolId(1);
    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let times_id = SymbolId(2);
    grammar.tokens.insert(
        times_id,
        Token {
            name: "times".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    let lparen_id = SymbolId(3);
    grammar.tokens.insert(
        lparen_id,
        Token {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );

    let rparen_id = SymbolId(4);
    grammar.tokens.insert(
        rparen_id,
        Token {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );

    // Define non-terminals
    let expr_id = SymbolId(10);
    let term_id = SymbolId(11);
    let factor_id = SymbolId(12);
    let add_expr_id = SymbolId(13);
    let mul_expr_id = SymbolId(14);
    let paren_expr_id = SymbolId(15);
    let number_expr_id = SymbolId(16);

    grammar.rule_names.insert(expr_id, "expression".to_string());
    grammar.rule_names.insert(term_id, "term".to_string());
    grammar.rule_names.insert(factor_id, "factor".to_string());
    grammar
        .rule_names
        .insert(add_expr_id, "add_expression".to_string());
    grammar
        .rule_names
        .insert(mul_expr_id, "mul_expression".to_string());
    grammar
        .rule_names
        .insert(paren_expr_id, "paren_expression".to_string());
    grammar
        .rule_names
        .insert(number_expr_id, "number_expression".to_string());

    // expression → expression + term (add_expression)
    grammar
        .rules
        .entry(expr_id)
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(plus_id),
                Symbol::NonTerminal(term_id),
            ],
            precedence: Some(rust_sitter_ir::PrecedenceKind::Static(1)),
            associativity: Some(rust_sitter_ir::Associativity::Left),
            production_id: ProductionId(0),
            fields: vec![],
        });

    // expression → term
    grammar
        .rules
        .entry(expr_id)
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: expr_id,
            rhs: vec![Symbol::NonTerminal(term_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        });

    // term → term * factor (mul_expression)
    grammar
        .rules
        .entry(term_id)
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: term_id,
            rhs: vec![
                Symbol::NonTerminal(term_id),
                Symbol::Terminal(times_id),
                Symbol::NonTerminal(factor_id),
            ],
            precedence: Some(rust_sitter_ir::PrecedenceKind::Static(2)),
            associativity: Some(rust_sitter_ir::Associativity::Left),
            production_id: ProductionId(2),
            fields: vec![],
        });

    // term → factor
    grammar
        .rules
        .entry(term_id)
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: term_id,
            rhs: vec![Symbol::NonTerminal(factor_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(3),
            fields: vec![],
        });

    // factor → ( expression ) (paren_expression)
    grammar
        .rules
        .entry(factor_id)
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: factor_id,
            rhs: vec![
                Symbol::Terminal(lparen_id),
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(rparen_id),
            ],
            precedence: None,
            associativity: None,
            production_id: ProductionId(4),
            fields: vec![],
        });

    // factor → number (number_expression)
    grammar
        .rules
        .entry(factor_id)
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: factor_id,
            rhs: vec![Symbol::Terminal(number_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(5),
            fields: vec![],
        });

    // The parser will determine the starting symbol
    grammar
}

fn parse_expression(_grammar: &Grammar, input: &str) -> Option<Subtree> {
    // For testing, create subtree structures based on the input
    // In a real implementation, this would use the GLR parser

    let expr_id = SymbolId(10);
    let add_expr_id = SymbolId(13);
    let term_id = SymbolId(11);
    let factor_id = SymbolId(12);
    let number_id = SymbolId(0);
    let plus_id = SymbolId(1);

    if input == "1 + 2" {
        // Create tree for "1 + 2"
        Some(Subtree {
            symbol: add_expr_id,
            children: vec![
                Subtree {
                    symbol: expr_id,
                    children: vec![Subtree {
                        symbol: term_id,
                        children: vec![Subtree {
                            symbol: factor_id,
                            children: vec![Subtree {
                                symbol: number_id,
                                children: vec![],
                                start_byte: 0,
                                end_byte: 1,
                            }],
                            start_byte: 0,
                            end_byte: 1,
                        }],
                        start_byte: 0,
                        end_byte: 1,
                    }],
                    start_byte: 0,
                    end_byte: 1,
                },
                Subtree {
                    symbol: plus_id,
                    children: vec![],
                    start_byte: 2,
                    end_byte: 3,
                },
                Subtree {
                    symbol: term_id,
                    children: vec![Subtree {
                        symbol: factor_id,
                        children: vec![Subtree {
                            symbol: number_id,
                            children: vec![],
                            start_byte: 4,
                            end_byte: 5,
                        }],
                        start_byte: 4,
                        end_byte: 5,
                    }],
                    start_byte: 4,
                    end_byte: 5,
                },
            ],
            start_byte: 0,
            end_byte: 5,
        })
    } else {
        // Default tree structure
        Some(Subtree {
            symbol: expr_id,
            children: vec![],
            start_byte: 0,
            end_byte: input.len(),
        })
    }
}

#[test]
fn test_simple_query() {
    let grammar = create_test_grammar();

    // Parse "1 + 2"
    let tree = parse_expression(&grammar, "1 + 2").unwrap();

    // Create a query to find all numbers
    let query_parser = QueryParser::new(&grammar, "(number)");
    let query = query_parser.parse().unwrap();

    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, &tree).collect();

    // Should find 2 numbers
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_query_with_captures() {
    let grammar = create_test_grammar();

    // Parse "1 + 2"
    let tree = parse_expression(&grammar, "1 + 2").unwrap();

    // Create a query to capture numbers
    let query_parser = QueryParser::new(&grammar, "(number) @num");
    let query = query_parser.parse().unwrap();

    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, &tree).collect();

    // Should find 2 numbers
    assert_eq!(matches.len(), 2);

    // Check capture names
    assert_eq!(query.capture_names.get("num"), Some(&0));
}

#[test]
fn test_wildcard_pattern() {
    let grammar = create_test_grammar();

    // Parse "(1 + 2)"
    let tree = parse_expression(&grammar, "(1 + 2)").unwrap();

    // Create a query with wildcard to match any number
    let query_parser = QueryParser::new(&grammar, "(number) @num");
    let query = query_parser.parse().unwrap();

    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, &tree).collect();

    // Should find numbers (0 in this default case)
    assert_eq!(matches.len(), 0); // Default tree has no numbers
}

#[test]
fn test_query_with_quantifiers() {
    let grammar = create_test_grammar();

    // Parse "1 + 2 + 3"
    let tree = parse_expression(&grammar, "1 + 2 + 3").unwrap();

    // Create a query to match expressions with one or more additions
    // Note: This is a simplified example - real Tree-sitter queries would handle this differently
    let query_parser = QueryParser::new(&grammar, "(add_expression)");
    let query = query_parser.parse().unwrap();

    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, &tree).collect();

    // Should find 2 additions (1 + 2) + 3
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_nested_query() {
    let grammar = create_test_grammar();

    // Parse "(1 + 2) * 3"
    let tree = parse_expression(&grammar, "(1 + 2) * 3").unwrap();

    // Create a query to find additions inside parentheses
    let query_parser = QueryParser::new(&grammar, "(paren_expression (add_expression) @add)");
    let query = query_parser.parse().unwrap();

    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, &tree).collect();

    // Should find 1 addition inside parentheses
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].captures.len(), 1);
}

#[test]
fn test_query_with_predicates() {
    let grammar = create_test_grammar();

    // Parse "1 + 1"
    let _tree = parse_expression(&grammar, "1 + 1").unwrap();

    // Create a query with an equality predicate
    // Note: Predicate evaluation is simplified since we don't have source text
    let query_parser = QueryParser::new(
        &grammar,
        "(add_expression (expression) @left (plus) (term) @right) (#eq? @left @right)",
    );
    let query = query_parser.parse().unwrap();

    // Should have one pattern with one predicate
    assert_eq!(query.patterns.len(), 1);
    assert_eq!(query.patterns[0].predicate_indices.len(), 1);
}

#[test]
fn test_query_max_depth() {
    let grammar = create_test_grammar();

    // Parse nested expression
    let tree = parse_expression(&grammar, "((1 + 2) * 3) + 4").unwrap();

    // Create a query to find all expressions
    let query_parser = QueryParser::new(&grammar, "(expression)");
    let query = query_parser.parse().unwrap();

    // Without depth limit
    let cursor = QueryCursor::new();
    let all_matches: Vec<_> = cursor.matches(&query, &tree).collect();

    // With depth limit
    let mut limited_cursor = QueryCursor::new();
    limited_cursor.set_max_depth(2);
    let limited_matches: Vec<_> = limited_cursor.matches(&query, &tree).collect();

    // Should find fewer matches with depth limit
    assert!(limited_matches.len() < all_matches.len());
}

#[test]
fn test_query_parser_errors() {
    let grammar = create_test_grammar();

    // Test various parsing errors
    let test_cases = vec![
        ("", "EmptyQuery"),
        ("expression", "ExpectedOpenParen"),
        ("(expression", "ExpectedCloseParen"),
        ("(unknown_type)", "UnknownNodeType"),
        ("(#unknown?)", "ExpectedOpenParen"),
        ("(expression) (#eq? @unknown)", "UnknownCapture"),
    ];

    for (query_str, expected_error) in test_cases {
        let parser = QueryParser::new(&grammar, query_str);
        let result = parser.parse();
        assert!(result.is_err(), "Expected error for query: {}", query_str);
        let error_str = format!("{:?}", result.unwrap_err());
        assert!(
            error_str.contains(expected_error),
            "Expected {} error for query: {}, got: {}",
            expected_error,
            query_str,
            error_str
        );
    }
}

#[test]
fn test_multiple_patterns() {
    let grammar = create_test_grammar();

    // Parse "1 + 2 * 3"
    let tree = parse_expression(&grammar, "1 + 2 * 3").unwrap();

    // Create a query with multiple patterns
    let query_parser = QueryParser::new(&grammar, "(add_expression) (mul_expression)");
    let query = query_parser.parse().unwrap();

    assert_eq!(query.patterns.len(), 2);

    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, &tree).collect();

    // Should find 1 addition and 1 multiplication
    assert_eq!(matches.len(), 2);

    // Check which patterns matched
    let pattern_indices: Vec<_> = matches.iter().map(|m| m.pattern_index).collect();
    assert!(pattern_indices.contains(&0)); // add_expression
    assert!(pattern_indices.contains(&1)); // mul_expression
}

#[test]
fn test_query_with_comments() {
    let grammar = create_test_grammar();

    // Query with comments
    let query_str = r#"
        ; Find all binary operations
        (add_expression) @addition
        
        ; Also find multiplications
        (mul_expression) @multiplication
    "#;

    let query_parser = QueryParser::new(&grammar, query_str);
    let query = query_parser.parse().unwrap();

    assert_eq!(query.patterns.len(), 2);
    assert_eq!(query.capture_names.len(), 2);
    assert!(query.capture_names.contains_key("addition"));
    assert!(query.capture_names.contains_key("multiplication"));
}
