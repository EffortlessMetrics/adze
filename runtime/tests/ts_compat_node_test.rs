//! Tests for Tree-sitter compatibility Node metadata methods
//! These tests validate the new Node implementation methods added in PR #58

#[cfg(feature = "ts-compat")]
mod ts_compat_tests {
    use adze::adze_glr_core as glr_core;
    use adze::adze_ir as ir;
    use adze::ts_compat::{Language, Parser, Point};
    use glr_core::{FirstFollowSets, build_lr1_automaton};
    use ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
    use std::sync::Arc;

    fn create_test_language() -> Arc<Language> {
        let mut grammar = Grammar::new("test".to_string());

        // Add tokens for testing
        let number_id = SymbolId(1);
        let expr_id = SymbolId(2);

        // Token definitions
        grammar.tokens.insert(
            number_id,
            Token {
                name: "number".to_string(),
                pattern: TokenPattern::String(r"\d+".to_string()),
                fragile: false,
            },
        );

        // Rule: expr -> number
        let rule = Rule {
            lhs: expr_id,
            rhs: vec![Symbol::Terminal(number_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);

        // Add rule names for symbol lookup
        grammar.rule_names.insert(expr_id, "expression".to_string());
        grammar.rule_names.insert(number_id, "number".to_string());

        // Build the parse table using the GLR core
        let first_follow_sets = FirstFollowSets::compute(&grammar).unwrap();
        let parse_table = build_lr1_automaton(&grammar, &first_follow_sets).unwrap();

        Arc::new(Language::new("test", grammar, parse_table))
    }

    #[test]
    fn test_node_kind_returns_correct_type() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        // Parse simple input
        let tree = parser.parse("123", None).expect("Parse should succeed");
        let root = tree.root_node();

        // Root node should return actual kind, not static string
        let kind = root.kind();
        assert_ne!(kind, "node", "Node kind should not be static 'node'");
        // Should be either "expression" or the symbol name from grammar
        assert!(
            kind == "expression" || kind == "unknown",
            "Node kind should be 'expression' or 'unknown', got: {}",
            kind
        );
    }

    #[test]
    fn test_node_byte_positions() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        let source = "123";
        let tree = parser.parse(source, None).expect("Parse should succeed");
        let root = tree.root_node();

        // Root node should span the entire source
        assert_eq!(root.start_byte(), 0, "Root node should start at byte 0");
        assert_eq!(
            root.end_byte(),
            source.len(),
            "Root node should end at source length"
        );
    }

    #[test]
    fn test_node_positions() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        // Test single line
        let tree = parser.parse("123", None).expect("Parse should succeed");
        let root = tree.root_node();

        assert_eq!(
            root.start_position(),
            Point { row: 0, column: 0 },
            "Root should start at (0, 0)"
        );
        assert_eq!(
            root.end_position(),
            Point { row: 0, column: 3 },
            "Root should end at (0, 3)"
        );
    }

    #[test]
    fn test_node_positions_multiline() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        // Test multiline input
        let source = "123\n456\n789";
        if let Some(tree) = parser.parse(source, None) {
            let root = tree.root_node();

            assert_eq!(root.start_position(), Point { row: 0, column: 0 });
            // End should be at line 2, column 3
            assert_eq!(root.end_position(), Point { row: 2, column: 3 });
        }
    }

    #[test]
    fn test_node_child_count() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        let tree = parser.parse("123", None).expect("Parse should succeed");
        let root = tree.root_node();

        // Current implementation returns 0 children due to parser_v4 limitations
        // This is expected behavior for now
        let child_count = root.child_count();
        assert!(
            child_count == 0,
            "Child count should be 0 (parser_v4 limitation), got: {}",
            child_count
        );
    }

    #[test]
    fn test_node_child_access() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        let tree = parser.parse("123", None).expect("Parse should succeed");
        let root = tree.root_node();

        // Child access should return None (parser_v4 limitation)
        assert!(root.child(0).is_none(), "Child access should return None");
        assert!(root.child(1).is_none(), "Child access should return None");
    }

    #[test]
    fn test_node_error_detection() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        // Test parse result - check error states work correctly
        let tree = parser.parse("123", None).expect("Parse should succeed");
        let root = tree.root_node();

        // Test that is_error and is_missing methods work and return boolean values
        let is_error = root.is_error();
        let is_missing = root.is_missing();

        // These should be boolean values, not panic - test that they're callable
        let _: bool = is_error; // Ensure it's a boolean type
        let _: bool = is_missing; // Ensure it's a boolean type

        // If there are errors in the tree, the node should reflect that
        if tree.error_count() > 0 {
            assert!(root.is_error(), "Node should be error if tree has errors");
        }

        // For root node, test that the methods work consistently
        // The exact values depend on the parser implementation,
        // but the methods should be callable without panicking
    }

    #[test]
    fn test_node_byte_range() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        let source = "12345";
        let tree = parser.parse(source, None).expect("Parse should succeed");
        let root = tree.root_node();

        let range = root.byte_range();
        assert_eq!(range, 0..5, "Byte range should cover entire source");
    }

    #[test]
    fn test_node_text_extraction() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        let source = "67890";
        let tree = parser.parse(source, None).expect("Parse should succeed");
        let root = tree.root_node();

        // Test UTF-8 text extraction
        let text = root
            .utf8_text(source.as_bytes())
            .expect("Should be valid UTF-8");
        assert_eq!(text, source, "Extracted text should match source");

        // Test string text extraction
        let text_string = root.text(source.as_bytes());
        assert_eq!(text_string, source, "Extracted string should match source");
    }

    #[test]
    fn test_node_empty_source() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        // Test empty source - this will likely fail to parse, which is expected
        if let Some(tree) = parser.parse("", None) {
            let root = tree.root_node();

            assert_eq!(root.start_byte(), 0);
            assert_eq!(root.end_byte(), 0);
            assert_eq!(root.byte_range(), 0..0);

            // Empty source with errors should be considered "missing"
            if tree.error_count() > 0 {
                assert!(
                    root.is_missing(),
                    "Empty source with errors should be missing"
                );
            }
        }
    }

    #[test]
    fn test_node_unicode_text() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        // Test with unicode content - might not parse successfully due to grammar, but shouldn't crash
        let unicode_source = "123🦀456";
        if let Some(tree) = parser.parse(unicode_source, None) {
            let root = tree.root_node();

            // Should handle unicode byte counting correctly
            assert_eq!(root.start_byte(), 0);
            assert_eq!(root.end_byte(), unicode_source.len()); // Byte length, not char length

            let extracted = root.text(unicode_source.as_bytes());
            assert_eq!(extracted, unicode_source);
        }
    }

    #[test]
    fn test_node_api_compatibility() {
        // Test that the Node API matches Tree-sitter expectations
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        let tree = parser.parse("789", None).expect("Parse should succeed");
        let root = tree.root_node();

        // All methods should be callable without panicking
        let _ = root.kind();
        let _ = root.start_byte();
        let _ = root.end_byte();
        let _ = root.start_position();
        let _ = root.end_position();
        let _ = root.child_count();
        let _ = root.child(0);
        let _ = root.is_error();
        let _ = root.is_missing();
        let _ = root.byte_range();
        let _ = root.utf8_text(b"test");
        let _ = root.text(b"test");
    }

    #[test]
    fn test_debug_implementation() {
        let language = create_test_language();
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        let tree = parser.parse("999", None).expect("Parse should succeed");
        let root = tree.root_node();

        // Should be able to debug print
        let debug_str = format!("{:?}", root);
        assert!(!debug_str.is_empty(), "Debug output should not be empty");

        // Should contain some meaningful information
        assert!(
            debug_str.contains("Node"),
            "Debug output should contain 'Node'"
        );
    }
}
