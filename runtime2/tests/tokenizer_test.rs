//! Tokenizer Unit and Integration Tests (Phase 3.2)
//!
//! Following TDD/BDD methodology - tests written before implementation.
//! Contract: docs/specs/PHASE_3.2_TOKENIZATION_FOREST_CONVERSION.md

#[cfg(feature = "pure-rust")]
mod tokenizer_unit_tests {
    use adze_glr_core::SymbolId;
    use adze_runtime::Token;
    use adze_runtime::tokenizer::{
        Matcher, TokenPattern, Tokenizer, TokenizerError, WhitespaceMode,
    };

    /// Test: Literal token matching
    ///
    /// Contract: Exact string matches produce tokens
    #[test]
    fn test_literal_tokens() {
        // Grammar: + → "+"
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1),
                matcher: Matcher::Literal("+".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(2),
                matcher: Matcher::Literal("-".to_string()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);
        let tokens = tokenizer.scan(b"+-+").unwrap();

        assert_eq!(tokens.len(), 4); // PLUS, MINUS, PLUS, EOF
        assert_eq!(tokens[0].kind, 1); // PLUS
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 1);

        assert_eq!(tokens[1].kind, 2); // MINUS
        assert_eq!(tokens[1].start, 1);
        assert_eq!(tokens[1].end, 2);

        assert_eq!(tokens[2].kind, 1); // PLUS
        assert_eq!(tokens[2].start, 2);
        assert_eq!(tokens[2].end, 3);

        assert_eq!(tokens[3].kind, 0); // EOF
        assert_eq!(tokens[3].start, 3);
        assert_eq!(tokens[3].end, 3);
    }

    /// Test: Regex token matching
    ///
    /// Contract: Regex patterns match sequences
    #[test]
    fn test_regex_tokens() {
        // Grammar: NUMBER → [0-9]+
        let patterns = vec![TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
            is_keyword: false,
        }];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);
        let tokens = tokenizer.scan(b"123").unwrap();

        assert_eq!(tokens.len(), 2); // NUMBER, EOF
        assert_eq!(tokens[0].kind, 1); // NUMBER
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 3);
    }

    /// Test: Maximal munch (longest match)
    ///
    /// Contract: Longer matches preferred over shorter
    #[test]
    fn test_maximal_munch() {
        // Grammar:
        //   IF → "if"
        //   IDENT → [a-z]+
        //
        // Input: "ifx" should match as IDENT (longer match)
        // Input: "if " should match as IF (exact match preferred when tied)
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1), // IF
                matcher: Matcher::Literal("if".to_string()),
                is_keyword: true,
            },
            TokenPattern {
                symbol_id: SymbolId(2), // IDENT
                matcher: Matcher::Regex(regex::Regex::new(r"^[a-z]+").unwrap()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);

        // "ifx" → IDENT (longer match: 3 chars vs 2 chars)
        let tokens = tokenizer.scan(b"ifx").unwrap();
        assert_eq!(tokens.len(), 2); // IDENT, EOF
        assert_eq!(tokens[0].kind, 2); // IDENT
        assert_eq!(tokens[0].end - tokens[0].start, 3);
    }

    /// Test: Keyword vs identifier disambiguation
    ///
    /// Contract: When lengths match, keywords have precedence
    #[test]
    fn test_keyword_vs_identifier() {
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1), // IF
                matcher: Matcher::Literal("if".to_string()),
                is_keyword: true,
            },
            TokenPattern {
                symbol_id: SymbolId(2), // IDENT
                matcher: Matcher::Regex(regex::Regex::new(r"^[a-z]+").unwrap()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);

        // "if" by itself → IF (keyword)
        let tokens = tokenizer.scan(b"if").unwrap();
        assert_eq!(tokens.len(), 2); // IF, EOF
        assert_eq!(tokens[0].kind, 1); // IF (keyword)

        // "ifx" → IDENT (longer match)
        let tokens2 = tokenizer.scan(b"ifx").unwrap();
        assert_eq!(tokens2[0].kind, 2); // IDENT
    }

    /// Test: Whitespace handling (skip mode)
    ///
    /// Contract: Whitespace skipped, positions adjusted correctly
    #[test]
    fn test_whitespace_skip() {
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1), // NUMBER
                matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(2), // PLUS
                matcher: Matcher::Literal("+".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(255), // WHITESPACE (special)
                matcher: Matcher::Regex(regex::Regex::new(r"^\s+").unwrap()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);
        let tokens = tokenizer.scan(b"1 + 2").unwrap();

        // Whitespace should be skipped
        assert_eq!(tokens.len(), 4); // NUM, PLUS, NUM, EOF
        assert_eq!(tokens[0].kind, 1); // NUMBER "1"
        assert_eq!(tokens[1].kind, 2); // PLUS "+"
        assert_eq!(tokens[2].kind, 1); // NUMBER "2"
    }

    /// Test: Error on invalid token
    ///
    /// Contract: Unrecognized characters produce error
    #[test]
    fn test_error_invalid_token() {
        let patterns = vec![TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Literal("+".to_string()),
            is_keyword: false,
        }];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);

        // "@" is not recognized
        let result = tokenizer.scan(b"@");
        assert!(result.is_err());

        if let Err(TokenizerError::InvalidToken { position, .. }) = result {
            assert_eq!(position, 0);
        } else {
            panic!("Expected InvalidToken error");
        }
    }

    /// Test: EOF token always present
    ///
    /// Contract: Last token is always EOF at input.len()
    #[test]
    fn test_eof_token() {
        let patterns = vec![TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
            is_keyword: false,
        }];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);

        // Empty input
        let tokens = tokenizer.scan(b"").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, 0); // EOF
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 0);

        // Non-empty input
        let tokens2 = tokenizer.scan(b"123").unwrap();
        assert_eq!(tokens2.last().unwrap().kind, 0); // EOF
        assert_eq!(tokens2.last().unwrap().start, 3);
    }

    /// Test: Token positions have no gaps or overlaps
    ///
    /// Contract: For all i: token[i].end == token[i+1].start
    #[test]
    fn test_token_positions_no_gaps() {
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1),
                matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(2),
                matcher: Matcher::Literal("+".to_string()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);
        let tokens = tokenizer.scan(b"1+2+3").unwrap();

        // Verify no gaps or overlaps
        for i in 0..tokens.len() - 1 {
            assert_eq!(
                tokens[i].end,
                tokens[i + 1].start,
                "Gap or overlap between token {} and {}",
                i,
                i + 1
            );
        }
    }
}

#[cfg(feature = "pure-rust")]
mod tokenizer_integration_tests {
    use adze_glr_core::SymbolId;
    use adze_runtime::tokenizer::{Matcher, TokenPattern, Tokenizer, WhitespaceMode};

    /// Integration Test: Arithmetic expression tokenization
    ///
    /// Grammar:
    ///   expr → NUMBER | expr + expr | expr * expr
    ///
    /// Tests complete pipeline with multiple token types
    #[test]
    fn test_arithmetic_expression() {
        // Setup grammar
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1), // NUMBER
                matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(2), // PLUS
                matcher: Matcher::Literal("+".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(3), // TIMES
                matcher: Matcher::Literal("*".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(255), // WHITESPACE
                matcher: Matcher::Regex(regex::Regex::new(r"^\s+").unwrap()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);

        // Parse: "1 + 2 * 3"
        let tokens = tokenizer.scan(b"1 + 2 * 3").unwrap();

        // Expected: [NUM(1), PLUS, NUM(2), TIMES, NUM(3), EOF]
        assert_eq!(tokens.len(), 6);

        assert_eq!(tokens[0].kind, 1); // NUMBER
        assert_eq!(tokens[1].kind, 2); // PLUS
        assert_eq!(tokens[2].kind, 1); // NUMBER
        assert_eq!(tokens[3].kind, 3); // TIMES
        assert_eq!(tokens[4].kind, 1); // NUMBER
        assert_eq!(tokens[5].kind, 0); // EOF
    }

    /// Integration Test: Nested parentheses
    ///
    /// Grammar:
    ///   expr → ( expr ) | NUMBER
    ///
    /// Tests delimiter tokens and nesting
    #[test]
    fn test_nested_parens() {
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1), // LPAREN
                matcher: Matcher::Literal("(".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(2), // RPAREN
                matcher: Matcher::Literal(")".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(3), // NUMBER
                matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);

        // Parse: "((1))"
        let tokens = tokenizer.scan(b"((1))").unwrap();

        // Expected: [LPAREN, LPAREN, NUM, RPAREN, RPAREN, EOF]
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].kind, 1); // LPAREN
        assert_eq!(tokens[1].kind, 1); // LPAREN
        assert_eq!(tokens[2].kind, 3); // NUMBER
        assert_eq!(tokens[3].kind, 2); // RPAREN
        assert_eq!(tokens[4].kind, 2); // RPAREN
        assert_eq!(tokens[5].kind, 0); // EOF
    }

    /// Integration Test: Complex expression with mixed operators
    ///
    /// Tests: operator precedence, associativity (via tokenization)
    #[test]
    fn test_complex_expression() {
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1), // NUMBER
                matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(2), // PLUS
                matcher: Matcher::Literal("+".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(3), // MINUS
                matcher: Matcher::Literal("-".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(4), // TIMES
                matcher: Matcher::Literal("*".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(5), // DIVIDE
                matcher: Matcher::Literal("/".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(6), // LPAREN
                matcher: Matcher::Literal("(".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(7), // RPAREN
                matcher: Matcher::Literal(")".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(255), // WHITESPACE
                matcher: Matcher::Regex(regex::Regex::new(r"^\s+").unwrap()),
                is_keyword: false,
            },
        ];

        let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);

        // Parse: "(1 + 2) * 3 - 4 / 5"
        let tokens = tokenizer.scan(b"(1 + 2) * 3 - 4 / 5").unwrap();

        // Should produce correct token sequence
        assert!(tokens.len() > 10); // Complex expression
        assert_eq!(tokens.last().unwrap().kind, 0); // EOF
    }
}

#[cfg(not(feature = "pure-rust"))]
#[test]
fn test_tokenizer_feature_not_enabled() {
    // Placeholder test when feature is disabled
}
