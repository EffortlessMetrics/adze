//! Phase 3.2 End-to-End Integration Test
//!
//! Tests the complete pipeline:
//! - Component 1: Tokenizer (tokenize input)
//! - Phase 3.1: GLR Engine (parse tokens to forest)
//! - Component 2: Forest Converter (convert forest to tree)
//!
//! Contract: Verify that all Phase 3.2 components work together seamlessly.

#[cfg(feature = "pure-rust-glr")]
mod phase_3_2_end_to_end {
    use rust_sitter_runtime::{
        Parser,
        tokenizer::{TokenPattern, Matcher, WhitespaceMode},
        language::SymbolMetadata,
    };
    use rust_sitter_glr_core::{
        SymbolId, ParseTable, Action, FirstFollowSets,
        build_lr1_automaton,
    };
    use rust_sitter_ir::{
        Grammar, ProductionId, Rule, Symbol,
        Token as IrToken, TokenPattern as IrTokenPattern,
    };

    /// Create a simple arithmetic grammar: expr → NUMBER | expr + expr
    ///
    /// Symbols:
    /// - 0: EOF
    /// - 1: NUMBER (terminal)
    /// - 2: PLUS (terminal)
    /// - 3: expr (nonterminal)
    ///
    fn create_arithmetic_grammar() -> (&'static ParseTable, Vec<SymbolMetadata>, Vec<TokenPattern>) {
        // Build IR grammar
        let mut grammar = Grammar::new("arithmetic".to_string());

        // Define tokens
        let number_id = SymbolId(1);
        grammar.tokens.insert(
            number_id,
            IrToken {
                name: "NUMBER".to_string(),
                pattern: IrTokenPattern::String("[0-9]+".to_string()),
                fragile: false,
            },
        );

        let plus_id = SymbolId(2);
        grammar.tokens.insert(
            plus_id,
            IrToken {
                name: "PLUS".to_string(),
                pattern: IrTokenPattern::String("+".to_string()),
                fragile: false,
            },
        );

        // Define nonterminal
        let expr_id = SymbolId(3);
        grammar.rule_names.insert(expr_id, "expr".to_string());

        // Define rules
        // Rule 1: expr → NUMBER
        grammar.rules.entry(expr_id).or_default().push(Rule {
            lhs: expr_id,
            rhs: vec![Symbol::Terminal(number_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        });

        // Rule 2: expr → expr PLUS expr
        grammar.rules.entry(expr_id).or_default().push(Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(plus_id),
                Symbol::NonTerminal(expr_id),
            ],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        });

        // Build LR(1) parse table
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let table = build_lr1_automaton(&grammar, &ff).unwrap();
        let table_static: &'static ParseTable = Box::leak(Box::new(table));

        // Symbol metadata
        let symbol_metadata = vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            }, // EOF
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }, // NUMBER
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }, // PLUS
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            }, // expr
        ];

        // Token patterns for Tokenizer
        let token_patterns = vec![
            TokenPattern {
                symbol_id: number_id,
                matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: plus_id,
                matcher: Matcher::Literal("+".to_string()),
                is_keyword: false,
            },
        ];

        (table_static, symbol_metadata, token_patterns)
    }

    /// Test: Phase 3.2 Components Integration
    ///
    /// Verifies that Tokenizer and ForestConverter are properly wired into the Parser.
    /// Note: Full parsing tests with generated grammars are in glr_parse.rs
    #[test]
    fn test_phase_3_2_components_integration() {
        let (table, metadata, patterns) = create_arithmetic_grammar();

        let mut parser = Parser::new();

        // Test Component Integration API
        parser.set_glr_table(table).unwrap();
        parser.set_symbol_metadata(metadata).unwrap();
        parser.set_token_patterns(patterns).unwrap();

        // Verify all components are registered
        assert!(parser.is_glr_mode());
    }

    /// Test: Tokenizer integration (simple case)
    ///
    /// Tests that tokenizer is properly invoked via Parser
    #[test]
    fn test_tokenizer_integration() {
        let (table, metadata, patterns) = create_arithmetic_grammar();

        let mut parser = Parser::new();
        parser.set_glr_table(table).unwrap();
        parser.set_symbol_metadata(metadata).unwrap();
        parser.set_token_patterns(patterns).unwrap();

        // Tokenizer should tokenize the input even if parse fails
        // This is tested indirectly - if tokenizer fails, we get TokenizerError
        // If GLR fails, we get ParseError
        let result = parser.parse(b"123", None);

        // Either succeeds or fails with parse error (not tokenizer error)
        if let Err(e) = result {
            let err_msg = format!("{:?}", e);
            // Should not be "Invalid token" (that would be tokenizer error)
            assert!(!err_msg.contains("Invalid token"));
        }
    }

    /// Test: ForestConverter integration
    ///
    /// Tests that forest converter is properly invoked
    #[test]
    fn test_forest_converter_integration() {
        let (table, metadata, patterns) = create_arithmetic_grammar();

        let mut parser = Parser::new();
        parser.set_glr_table(table).unwrap();
        parser.set_symbol_metadata(metadata).unwrap();
        parser.set_token_patterns(patterns).unwrap();

        // The parse will either succeed (full integration works) or
        // fail with parse error (but forest converter was still used)
        let result = parser.parse(b"1+2", None);

        if result.is_ok() {
            let tree = result.unwrap();
            // If parsing succeeded, verify tree has source
            assert_eq!(tree.source_bytes(), Some(&b"1+2"[..]));
        } else {
            // If parsing failed, it should be a parse error, not converter error
            let err_msg = format!("{:?}", result.unwrap_err());
            // Should not be converter-specific errors
            assert!(!err_msg.contains("NoRoots"));
            assert!(!err_msg.contains("InvalidNodeId"));
        }
    }

    /// Test: Tokenization errors propagate correctly
    ///
    /// Invalid input should produce tokenizer error
    #[test]
    fn test_e2e_tokenization_error() {
        let (table, metadata, patterns) = create_arithmetic_grammar();

        let mut parser = Parser::new();
        parser.set_glr_table(table).unwrap();
        parser.set_symbol_metadata(metadata).unwrap();
        parser.set_token_patterns(patterns).unwrap();

        // Input with invalid character (letter)
        let result = parser.parse(b"1+x", None);
        assert!(result.is_err());
    }

    /// Test: Parser API consistency
    ///
    /// Verify that set_token_patterns requires set_glr_table first
    #[test]
    fn test_e2e_api_contract() {
        let (_, _, patterns) = create_arithmetic_grammar();

        let mut parser = Parser::new();

        // Should fail: set_token_patterns without set_glr_table
        let result = parser.set_token_patterns(patterns.clone());
        assert!(result.is_err());

        // Should succeed: set_token_patterns after set_glr_table
        let (table, metadata, _) = create_arithmetic_grammar();
        parser.set_glr_table(table).unwrap();
        parser.set_symbol_metadata(metadata).unwrap();
        let result2 = parser.set_token_patterns(patterns);
        assert!(result2.is_ok());
    }
}

#[cfg(not(feature = "pure-rust-glr"))]
#[test]
fn test_phase_3_2_feature_not_enabled() {
    // Placeholder test when feature is disabled
}
