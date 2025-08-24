#![cfg(test)]
#![allow(unused_imports, dead_code)]
// TODO: This test file needs to be updated to work with the new Grammar API
// The old API had get_or_add_symbol and different Rule structure
#![allow(unexpected_cfgs)]
#![cfg(skip_outdated_tests)]

mod tests {
    use rust_sitter::{
        parser_v3::{ParseNode, Parser},
        query::{Query, QueryCursor, compile_query},
    };
    use rust_sitter_ir::{Grammar, Rule, RuleExpr, Symbol, SymbolId};
    use std::collections::HashMap;

    /// Create a simple test grammar
    fn create_test_grammar() -> Grammar {
        let mut grammar = Grammar::new("test".to_string());

        // Define symbols
        let program_id = grammar.get_or_add_symbol("program");
        let identifier_id = grammar.get_or_add_symbol("identifier");
        let keyword_id = grammar.get_or_add_symbol("keyword");
        let string_id = grammar.get_or_add_symbol("string");

        // Define rules
        grammar.rules.push(Rule {
            name: program_id,
            expr: RuleExpr::Repeat(Box::new(RuleExpr::Choice(vec![
                RuleExpr::Symbol(identifier_id),
                RuleExpr::Symbol(keyword_id),
                RuleExpr::Symbol(string_id),
            ]))),
            is_public: true,
            precedence: None,
            associativity: None,
        });

        grammar.rules.push(Rule {
            name: identifier_id,
            expr: RuleExpr::Pattern("[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            is_public: true,
            precedence: None,
            associativity: None,
        });

        grammar.rules.push(Rule {
            name: keyword_id,
            expr: RuleExpr::Choice(vec![
                RuleExpr::String("if".to_string()),
                RuleExpr::String("else".to_string()),
                RuleExpr::String("while".to_string()),
                RuleExpr::String("for".to_string()),
                RuleExpr::String("return".to_string()),
            ]),
            is_public: true,
            precedence: None,
            associativity: None,
        });

        grammar.rules.push(Rule {
            name: string_id,
            expr: RuleExpr::Pattern(r#""[^"]*""#.to_string()),
            is_public: true,
            precedence: None,
            associativity: None,
        });

        grammar
    }

    /// Helper to create a parse node
    fn make_node(
        symbol: SymbolId,
        start: usize,
        end: usize,
        children: Vec<ParseNode>,
    ) -> ParseNode {
        ParseNode {
            symbol,
            children,
            start_byte: start,
            end_byte: end,
            field_name: None,
        }
    }

    #[test]
    #[ignore = "query engine incomplete"]
    fn test_eq_predicate_with_value() {
        let source = "if test else while";
        let grammar = create_test_grammar();

        // Mock parse tree
        let tree = make_node(
            grammar.get_or_add_symbol("program"),
            0,
            18,
            vec![
                make_node(grammar.get_or_add_symbol("keyword"), 0, 2, vec![]), // "if"
                make_node(grammar.get_or_add_symbol("identifier"), 3, 7, vec![]), // "test"
                make_node(grammar.get_or_add_symbol("keyword"), 8, 12, vec![]), // "else"
                make_node(grammar.get_or_add_symbol("keyword"), 13, 18, vec![]), // "while"
            ],
        );

        // Query that matches keywords equal to "if"
        let query_str = r#"
            (keyword) @kw
            (#eq? @kw "if")
        "#;

        // Test with the enhanced matcher
        use rust_sitter::query::matcher_v2::{QueryMatch, QueryMatcher};

        // Create a mock query
        let mut query = Query {
            source: query_str.to_string(),
            patterns: vec![],
            capture_names: HashMap::new(),
            property_settings: vec![],
            property_predicates: vec![],
        };

        query.capture_names.insert("kw".to_string(), 0);

        use rust_sitter::query::ast::{Pattern, PatternNode, Predicate, Quantifier};

        let pattern = Pattern {
            root: PatternNode {
                symbol: grammar.get_or_add_symbol("keyword"),
                children: vec![],
                fields: HashMap::new(),
                capture: Some(0),
                is_named: true,
                quantifier: Quantifier::One,
            },
            predicates: vec![Predicate::Eq {
                capture1: 0,
                capture2: None,
                value: Some("if".to_string()),
            }],
            start_byte: 0,
        };

        query.patterns.push(pattern);

        // Match with predicates
        let matcher = QueryMatcher::new(&query, source);
        let matches = matcher.matches(&tree);

        // Should match only the "if" keyword
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures[0].node.start_byte, 0);
        assert_eq!(matches[0].captures[0].node.end_byte, 2);
    }

    #[test]
    #[ignore = "query engine incomplete"]
    fn test_eq_predicate_between_captures() {
        let source = "test other test";
        let grammar = create_test_grammar();

        // Mock parse tree
        let tree = make_node(
            grammar.get_or_add_symbol("program"),
            0,
            15,
            vec![
                make_node(grammar.get_or_add_symbol("identifier"), 0, 4, vec![]), // "test"
                make_node(grammar.get_or_add_symbol("identifier"), 5, 10, vec![]), // "other"
                make_node(grammar.get_or_add_symbol("identifier"), 11, 15, vec![]), // "test"
            ],
        );

        // Query that matches consecutive identifiers that are equal
        let query_str = r#"
            (identifier) @first . (identifier) @second
            (#eq? @first @second)
        "#;

        // This would need a more sophisticated pattern matching for consecutive nodes
        // For now, test individual nodes
    }

    #[test]
    #[ignore = "query engine incomplete"]
    fn test_match_predicate() {
        let source = "test_var myFunction123 _private";
        let grammar = create_test_grammar();

        // Mock parse tree
        let tree = make_node(
            grammar.get_or_add_symbol("program"),
            0,
            31,
            vec![
                make_node(grammar.get_or_add_symbol("identifier"), 0, 8, vec![]), // "test_var"
                make_node(grammar.get_or_add_symbol("identifier"), 9, 22, vec![]), // "myFunction123"
                make_node(grammar.get_or_add_symbol("identifier"), 23, 31, vec![]), // "_private"
            ],
        );

        // Query that matches identifiers starting with underscore
        use rust_sitter::query::{
            ast::{Pattern, PatternNode, Predicate, Quantifier, Query},
            matcher_v2::QueryMatcher,
        };

        let mut query = Query {
            source: "".to_string(),
            patterns: vec![],
            capture_names: HashMap::new(),
            property_settings: vec![],
            property_predicates: vec![],
        };

        query.capture_names.insert("id".to_string(), 0);

        let pattern = Pattern {
            root: PatternNode {
                symbol: grammar.get_or_add_symbol("identifier"),
                children: vec![],
                fields: HashMap::new(),
                capture: Some(0),
                is_named: true,
                quantifier: Quantifier::One,
            },
            predicates: vec![Predicate::Match {
                capture: 0,
                regex: "^_".to_string(),
            }],
            start_byte: 0,
        };

        query.patterns.push(pattern);

        let matcher = QueryMatcher::new(&query, source);
        let matches = matcher.matches(&tree);

        // Should match only "_private"
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures[0].node.start_byte, 23);
    }

    #[test]
    #[ignore = "query engine incomplete"]
    fn test_any_of_predicate() {
        let source = "if test return while";
        let grammar = create_test_grammar();

        // Mock parse tree
        let tree = make_node(
            grammar.get_or_add_symbol("program"),
            0,
            20,
            vec![
                make_node(grammar.get_or_add_symbol("keyword"), 0, 2, vec![]), // "if"
                make_node(grammar.get_or_add_symbol("identifier"), 3, 7, vec![]), // "test"
                make_node(grammar.get_or_add_symbol("keyword"), 8, 14, vec![]), // "return"
                make_node(grammar.get_or_add_symbol("keyword"), 15, 20, vec![]), // "while"
            ],
        );

        // Query that matches control flow keywords
        use rust_sitter::query::{
            ast::{Pattern, PatternNode, Predicate, Quantifier, Query},
            matcher_v2::QueryMatcher,
        };

        let mut query = Query {
            source: "".to_string(),
            patterns: vec![],
            capture_names: HashMap::new(),
            property_settings: vec![],
            property_predicates: vec![],
        };

        query.capture_names.insert("flow".to_string(), 0);

        let pattern = Pattern {
            root: PatternNode {
                symbol: grammar.get_or_add_symbol("keyword"),
                children: vec![],
                fields: HashMap::new(),
                capture: Some(0),
                is_named: true,
                quantifier: Quantifier::One,
            },
            predicates: vec![Predicate::AnyOf {
                capture: 0,
                values: vec!["if".to_string(), "while".to_string(), "for".to_string()],
            }],
            start_byte: 0,
        };

        query.patterns.push(pattern);

        let matcher = QueryMatcher::new(&query, source);
        let matches = matcher.matches(&tree);

        // Should match "if" and "while" but not "return"
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].captures[0].node.start_byte, 0); // "if"
        assert_eq!(matches[1].captures[0].node.start_byte, 15); // "while"
    }

    #[test]
    #[ignore = "query engine incomplete"]
    fn test_not_predicates() {
        let source = "test if other";
        let grammar = create_test_grammar();

        // Mock parse tree
        let tree = make_node(
            grammar.get_or_add_symbol("program"),
            0,
            13,
            vec![
                make_node(grammar.get_or_add_symbol("identifier"), 0, 4, vec![]), // "test"
                make_node(grammar.get_or_add_symbol("keyword"), 5, 7, vec![]),    // "if"
                make_node(grammar.get_or_add_symbol("identifier"), 8, 13, vec![]), // "other"
            ],
        );

        // Query that matches identifiers NOT equal to "test"
        use rust_sitter::query::{
            ast::{Pattern, PatternNode, Predicate, Quantifier, Query},
            matcher_v2::QueryMatcher,
        };

        let mut query = Query {
            source: "".to_string(),
            patterns: vec![],
            capture_names: HashMap::new(),
            property_settings: vec![],
            property_predicates: vec![],
        };

        query.capture_names.insert("id".to_string(), 0);

        let pattern = Pattern {
            root: PatternNode {
                symbol: grammar.get_or_add_symbol("identifier"),
                children: vec![],
                fields: HashMap::new(),
                capture: Some(0),
                is_named: true,
                quantifier: Quantifier::One,
            },
            predicates: vec![Predicate::NotEq {
                capture1: 0,
                capture2: None,
                value: Some("test".to_string()),
            }],
            start_byte: 0,
        };

        query.patterns.push(pattern);

        let matcher = QueryMatcher::new(&query, source);
        let matches = matcher.matches(&tree);

        // Should match only "other"
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures[0].node.start_byte, 8);
    }
}
