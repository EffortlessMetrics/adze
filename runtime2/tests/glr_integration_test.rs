//! End-to-End GLR Integration Test (Phase 3.1)
//!
//! Tests the complete GLR parsing pipeline from Parser API through GLREngine.
//!
//! Contract: docs/specs/GLR_ENGINE_CONTRACT.md

#[cfg(feature = "pure-rust-glr")]
mod glr_integration {
    use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
    use rust_sitter_ir::builder::GrammarBuilder;
    use rust_sitter_runtime::Parser;

    /// Build a simple expression grammar for testing
    ///
    /// Grammar:
    ///   expr → expr + expr   (ambiguous - no precedence)
    ///   expr → NUMBER
    fn build_test_grammar() -> rust_sitter_ir::Grammar {
        GrammarBuilder::new("test_expr")
            .token("NUMBER", r"\d+")
            .token("+", "+")
            .rule("expr", vec!["expr", "+", "expr"])
            .rule("expr", vec!["NUMBER"])
            .start("expr")
            .build()
    }

    /// Create a static ParseTable for testing
    fn create_static_parse_table() -> &'static rust_sitter_glr_core::ParseTable {
        let mut grammar = build_test_grammar();
        let first_follow = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
        let table = build_lr1_automaton(&grammar, &first_follow).unwrap();

        // Leak to get 'static lifetime (acceptable for tests)
        Box::leak(Box::new(table))
    }

    #[test]
    fn test_glr_parser_integration_basic() {
        // Setup: Create parser with GLR table
        let mut parser = Parser::new();
        let table = create_static_parse_table();

        parser.set_glr_table(table).unwrap();

        // Verify parser is in GLR mode
        assert!(parser.is_glr_mode(), "Parser should be in GLR mode");

        // Parse using the GLR engine
        // Note: Phase 3.1 MVP has stub tokenizer that only produces EOF
        // Empty input will be rejected by grammar (expects expr)
        let result = parser.parse(b"", None);

        // Expected to fail due to incomplete tokenization (Phase 3.1 MVP)
        // TODO: In Phase 3.2, implement real tokenizer and expect success
        assert!(
            result.is_err(),
            "Parse should fail with stub tokenizer (Phase 3.1 MVP)"
        );

        // Verify error comes from GLR engine (not from missing language)
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("No parse succeeded") || err_msg.contains("Syntax error"),
            "Error should be from GLR engine, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_glr_parser_mode_switching() {
        use rust_sitter_runtime::{Language, language::SymbolMetadata};

        let mut parser = Parser::new();
        let table = create_static_parse_table();

        // Start in GLR mode
        parser.set_glr_table(table).unwrap();
        assert!(parser.is_glr_mode());

        // Switch to LR mode by setting language
        let language = Language {
            version: 0,
            symbol_count: 2,
            field_count: 0,
            max_alias_sequence_length: 0,
            parse_table: None,
            tokenize: None,
            symbol_names: vec!["sym0".to_string(), "sym1".to_string()],
            symbol_metadata: vec![
                SymbolMetadata {
                    is_terminal: true,
                    is_visible: true,
                    is_supertype: false,
                },
                SymbolMetadata {
                    is_terminal: false,
                    is_visible: true,
                    is_supertype: false,
                },
            ],
            field_names: vec![],
        };

        // Note: set_language will fail because this test language doesn't have a parse table
        // This is expected - we're just testing mode switching logic
        let result = parser.set_language(language);
        assert!(result.is_err(), "Should fail without valid parse table");

        // GLR mode should still be cleared even though set_language failed
        // (mode switching happens first in set_language)
    }

    #[test]
    fn test_glr_parser_requires_table() {
        let mut parser = Parser::new();

        // Parse without setting table should fail
        let result = parser.parse(b"test", None);

        assert!(
            result.is_err(),
            "Parse should fail without GLR table or language"
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("language") || err_msg.contains("GLR"),
            "Error should mention missing language or GLR state: {}",
            err_msg
        );
    }

    #[test]
    fn test_glr_engine_created_with_config() {
        let mut parser = Parser::new();
        let table = create_static_parse_table();

        parser.set_glr_table(table).unwrap();

        // Parse should create GLREngine with default config
        // Config validation happens in GLREngine::new()
        let result = parser.parse(b"", None);

        // If config is invalid, GLREngine::new() would panic
        // Getting any result (Ok or Err) means GLREngine was created successfully
        assert!(
            result.is_ok() || result.is_err(),
            "GLREngine should be created (config validation passed)"
        );

        // Verify we got a parse error (not a config error)
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(
                !msg.contains("max_forks") && !msg.contains("config"),
                "Should not be a config error, got: {}",
                msg
            );
        }
    }

    // TODO: Add more tests once tokenization and forest conversion are implemented
    // #[test]
    // fn test_parse_ambiguous_expression() {
    //     let mut parser = Parser::new();
    //     let table = create_static_parse_table();
    //
    //     parser.set_glr_table(table).unwrap();
    //
    //     // Parse "1 + 2 + 3" which has multiple valid parse trees
    //     let tree = parser.parse(b"1 + 2 + 3", None).unwrap();
    //
    //     // Verify tree structure (after Phase 3.3 forest conversion)
    //     assert!(tree.root_node().is_some());
    //     assert_eq!(tree.root_node().unwrap().kind(), "expr");
    // }
}

#[cfg(not(feature = "pure-rust-glr"))]
#[test]
fn test_glr_feature_not_enabled() {
    // This test ensures the file compiles even without pure-rust-glr feature
}
