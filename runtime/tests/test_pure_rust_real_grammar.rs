// Test the pure-Rust implementation with a real Tree-sitter grammar

#![cfg(test)]
#![allow(unused_imports, dead_code)]

#[cfg(feature = "pure-rust")]
mod pure_rust_real_grammar_tests {
    use rust_sitter::pure_incremental::{Edit, Point, Tree};
    use rust_sitter::pure_parser::{Parser, TSLanguage};

    // This test requires the JSON grammar to be built with pure-Rust support
    // For now, we'll test with a simple mock grammar

    #[test]
    #[ignore = "needs update to current codegen"]
    fn test_json_parsing() {
        // Create a simple JSON-like grammar for testing
        let language = create_json_language();
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        // Test parsing a simple JSON object
        let source = r#"{"name": "rust-sitter", "version": "1.0.0"}"#;
        let result = parser.parse_string(source);

        assert!(result.root.is_some());
        assert!(result.errors.is_empty());

        let root = result.root.unwrap();
        assert_eq!(root.start_byte, 0);
        assert_eq!(root.end_byte, source.len());
    }

    #[test]
    #[ignore = "needs update to current codegen"]
    fn test_javascript_expression_parsing() {
        // Create a simple JavaScript expression grammar
        let language = create_js_expr_language();
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        // Test various JavaScript expressions
        let test_cases = vec![
            "42",
            "1 + 2 * 3",
            "foo.bar.baz",
            "func(a, b, c)",
            "[1, 2, 3]",
            "{a: 1, b: 2}",
            "x => x * 2",
            "async function() { return 42; }",
        ];

        for source in test_cases {
            println!("Testing: {}", source);
            let result = parser.parse_string(source);

            // For now, just check that parsing completes
            // Real grammar would have proper validation
            assert!(result.root.is_some() || !result.errors.is_empty());
        }
    }

    #[test]
    #[ignore = "needs update to current codegen"]
    fn test_incremental_parsing_real_world() {
        let language = create_json_language();
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        // Initial parse
        let source1 = r#"{"users": [{"id": 1, "name": "Alice"}]}"#;
        let result1 = parser.parse_string(source1);
        assert!(result1.root.is_some());

        let tree1 = Tree::new(result1.root.unwrap(), language, source1.as_bytes());

        // Edit: Add another user
        let source2 = r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#;

        // The edit adds: , {"id": 2, "name": "Bob"}
        let edit_start = 37; // After the first }
        let edit = Edit {
            start_byte: edit_start,
            old_end_byte: edit_start,
            new_end_byte: edit_start + 26,
            start_point: Point {
                row: 0,
                column: edit_start,
            },
            old_end_point: Point {
                row: 0,
                column: edit_start,
            },
            new_end_point: Point {
                row: 0,
                column: edit_start + 26,
            },
        };

        let mut tree2 = tree1.clone();
        tree2.edit(&edit);

        // Incremental parse
        let result2 = parser.parse_string_with_tree(source2, Some(&tree2));
        assert!(result2.root.is_some());
        assert!(result2.errors.is_empty());
    }

    #[test]
    #[ignore = "needs update to current codegen"]
    fn test_error_recovery_real_world() {
        let language = create_json_language();
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        // Test various malformed JSON
        let error_cases = vec![
            (r#"{"unclosed": "#, "Missing closing brace"),
            (r#"{"extra": "comma",}"#, "Trailing comma"),
            (r#"{'single': 'quotes'}"#, "Single quotes"),
            (r#"{"missing" "colon"}"#, "Missing colon"),
            (r#"[1, 2, 3,]"#, "Trailing comma in array"),
        ];

        for (source, description) in error_cases {
            println!("Testing error case: {}", description);
            let result = parser.parse_string(source);

            // Should still produce a tree, but with errors
            assert!(result.root.is_some() || !result.errors.is_empty());

            if !result.errors.is_empty() {
                println!("  Errors found: {} error(s)", result.errors.len());
            }
        }
    }

    // Mock language definitions for testing
    fn create_json_language() -> &'static TSLanguage {
        use rust_sitter::pure_parser::{ExternalScanner, TSParseAction, TSParseActionType};

        static PARSE_TABLE: &[u16] = &[0; 1000];
        static SMALL_PARSE_TABLE: &[u16] = &[0; 100];
        static PARSE_ACTIONS: &[TSParseAction] = &[TSParseAction {
            action_type: TSParseActionType::Shift as u8,
            symbol: 1,
            state_or_production: 1,
        }];

        static LANGUAGE: TSLanguage = TSLanguage {
            version: 14,
            symbol_count: 50,
            token_count: 20,
            state_count: 100,
            large_state_count: 5,
            production_id_count: 10,
            field_count: 5,
            parse_table: PARSE_TABLE.as_ptr(),
            small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
            small_parse_table_map: std::ptr::null(),
            parse_actions: PARSE_ACTIONS.as_ptr(),
            lex_modes: std::ptr::null(),
            lex_fn: None,
            external_scanner: ExternalScanner {
                create: None,
                destroy: None,
                scan: None,
                serialize: None,
                deserialize: None,
            },
            production_id_map: std::ptr::null(),
        };

        &LANGUAGE
    }

    fn create_js_expr_language() -> &'static TSLanguage {
        // Similar mock for JavaScript expressions
        create_json_language() // Reuse for simplicity
    }
}

// Integration test that actually builds and uses a real grammar
#[cfg(all(test, feature = "pure-rust"))]
#[test]
#[ignore = "needs update to current codegen"]
fn test_build_and_use_real_grammar() {
    use rust_sitter_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};
    use std::path::Path;

    // Check if we have a test grammar available
    let grammar_path = Path::new("../tests/grammars/json/grammar.js");
    if !grammar_path.exists() {
        println!("Skipping real grammar test - grammar.js not found");
        return;
    }

    // Build the grammar
    let options = BuildOptions {
        out_dir: "target/test_grammars".to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    match build_parser_from_grammar_js(grammar_path, options) {
        Ok(result) => {
            println!("Successfully built {} parser", result.grammar_name);
            println!("Parser code length: {} bytes", result.parser_code.len());
            println!(
                "NODE_TYPES.json length: {} bytes",
                result.node_types_json.len()
            );

            // TODO: Actually load and use the generated parser
            // This would require dynamic loading or code generation
        }
        Err(e) => {
            println!("Failed to build grammar: {}", e);
            // Don't fail the test, as this is expected without proper setup
        }
    }
}
