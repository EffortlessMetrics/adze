//! Comprehensive tests for the GrammarLexer and GLRLexer in the runtime crate.
//!
//! Covers:
//! - Simple identifier tokenization
//! - Number tokenization
//! - String with quotes
//! - Operator tokenization
//! - Whitespace handling
//! - Multi-byte tokens
//! - Ambiguous tokens / longest match
//! - Token position tracking (byte offsets)
//! - Error handling for unknown characters
//! - Empty input
//! - Complex expression token streams
//! - Lookahead / priority-based matching

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::glr_lexer::GLRLexer;
use adze::lexer::{ErrorRecoveringLexer, ErrorRecoveryMode, GrammarLexer, Token};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::{Grammar, SymbolId, TokenPattern};

// ---------------------------------------------------------------------------
// Helper: collect all tokens from GrammarLexer (advancing position manually)
// ---------------------------------------------------------------------------
fn collect_grammar_tokens(lexer: &mut GrammarLexer, input: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        match lexer.next_token(input, pos) {
            Some(tok) if tok.symbol == SymbolId(0) => break, // EOF sentinel
            Some(tok) => {
                pos = tok.end;
                tokens.push(tok);
            }
            None => break,
        }
    }
    tokens
}

// ---------------------------------------------------------------------------
// 1. Tokenize simple identifier
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_tokenize_simple_identifier() {
    let patterns = vec![(
        SymbolId(1),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        0,
    )];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"foo", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, b"foo");
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 3);
}

#[test]
fn glr_lexer_tokenize_simple_identifier() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "ident".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "hello".to_string()).unwrap();
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.symbol_id, SymbolId(1));
    assert_eq!(tok.text, "hello");
    assert_eq!(tok.byte_offset, 0);
    assert_eq!(tok.byte_length, 5);
}

#[test]
fn grammar_lexer_identifier_with_underscores_and_digits() {
    let patterns = vec![(
        SymbolId(1),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        0,
    )];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"_my_var2", 0).unwrap();
    assert_eq!(tok.text, b"_my_var2");
    assert_eq!(tok.end, 8);
}

// ---------------------------------------------------------------------------
// 2. Tokenize number
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_tokenize_number() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"42", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, b"42");
}

#[test]
fn glr_lexer_tokenize_number() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "98765".to_string()).unwrap();
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.symbol_id, SymbolId(1));
    assert_eq!(tok.text, "98765");
    assert_eq!(tok.byte_length, 5);
}

#[test]
fn grammar_lexer_tokenize_multi_digit_number() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"00123", 0).unwrap();
    assert_eq!(tok.text, b"00123");
    assert_eq!(tok.end, 5);
}

// ---------------------------------------------------------------------------
// 3. Tokenize string with quotes
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_tokenize_quoted_string() {
    let patterns = vec![(
        SymbolId(1),
        TokenPattern::Regex(r#""[^"]*""#.to_string()),
        0,
    )];
    let mut lexer = GrammarLexer::new(&patterns);

    let input = br#""hello world""#;
    let tok = lexer.next_token(input, 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, br#""hello world""#.to_vec());
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 13);
}

#[test]
fn glr_lexer_tokenize_quoted_string() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, r#""abc""#.to_string()).unwrap();
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.symbol_id, SymbolId(1));
    assert_eq!(tok.text, r#""abc""#);
}

#[test]
fn grammar_lexer_tokenize_empty_quoted_string() {
    let patterns = vec![(
        SymbolId(1),
        TokenPattern::Regex(r#""[^"]*""#.to_string()),
        0,
    )];
    let mut lexer = GrammarLexer::new(&patterns);

    let input = br#""""#;
    let tok = lexer.next_token(input, 0).unwrap();
    assert_eq!(tok.text, br#""""#.to_vec());
    assert_eq!(tok.end, 2);
}

// ---------------------------------------------------------------------------
// 4. Tokenize operators (+, -, *, /)
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_tokenize_operators() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("+".to_string()), 0),
        (SymbolId(2), TokenPattern::String("-".to_string()), 0),
        (SymbolId(3), TokenPattern::String("*".to_string()), 0),
        (SymbolId(4), TokenPattern::String("/".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"+", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));

    let tok = lexer.next_token(b"-", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(2));

    let tok = lexer.next_token(b"*", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(3));

    let tok = lexer.next_token(b"/", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(4));
}

#[test]
fn glr_lexer_tokenize_operators() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        ir::Token {
            name: "minus".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        ir::Token {
            name: "star".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(4),
        ir::Token {
            name: "slash".to_string(),
            pattern: TokenPattern::String("/".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "+ - * /".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].text, "+");
    assert_eq!(tokens[1].text, "-");
    assert_eq!(tokens[2].text, "*");
    assert_eq!(tokens[3].text, "/");
}

// ---------------------------------------------------------------------------
// 5. Tokenize whitespace handling
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_skip_whitespace() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(2), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(2)]);

    // Leading whitespace is skipped
    let tok = lexer.next_token(b"   42", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, b"42");
    assert_eq!(tok.start, 3);
    assert_eq!(tok.end, 5);
}

#[test]
fn grammar_lexer_skip_tabs_and_newlines() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(2), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(2)]);

    let tok = lexer.next_token(b"\t\n\r 7", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, b"7");
    assert_eq!(tok.start, 4);
}

#[test]
fn glr_lexer_auto_skips_whitespace() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "   42   99   ".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "42");
    assert_eq!(tokens[1].text, "99");
}

// ---------------------------------------------------------------------------
// 6. Tokenize multi-byte tokens
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_multi_char_literal_tokens() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("==".to_string()), 0),
        (SymbolId(2), TokenPattern::String("!=".to_string()), 0),
        (SymbolId(3), TokenPattern::String("<=".to_string()), 0),
        (SymbolId(4), TokenPattern::String(">=".to_string()), 0),
        (SymbolId(5), TokenPattern::String("&&".to_string()), 0),
        (SymbolId(6), TokenPattern::String("||".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"==", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, b"==");
    assert_eq!(tok.end, 2);

    let tok = lexer.next_token(b"!=", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(2));

    let tok = lexer.next_token(b"&&", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(5));

    let tok = lexer.next_token(b"||", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(6));
}

#[test]
fn glr_lexer_multi_char_tokens() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "arrow".to_string(),
            pattern: TokenPattern::String("->".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        ir::Token {
            name: "fat_arrow".to_string(),
            pattern: TokenPattern::String("=>".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "-> =>".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].text, "->");
    assert_eq!(tokens[0].byte_length, 2);
    assert_eq!(tokens[1].text, "=>");
    assert_eq!(tokens[1].byte_length, 2);
}

// ---------------------------------------------------------------------------
// 7. Tokenize ambiguous tokens (longest match via priority)
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_priority_resolves_ambiguity() {
    let patterns = vec![
        (
            SymbolId(1),
            TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            1,
        ), // low priority
        (SymbolId(2), TokenPattern::String("if".to_string()), 10), // high priority
    ];
    let mut lexer = GrammarLexer::new(&patterns);

    // "if" matches both patterns; high-priority keyword wins
    let tok = lexer.next_token(b"if", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(2));
    assert_eq!(tok.text, b"if");
}

#[test]
fn grammar_lexer_longer_literal_preferred_over_shorter() {
    // When two literals could match, the one that actually matches first by
    // priority wins. Here we give the longer literal higher priority.
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("=".to_string()), 1),
        (SymbolId(2), TokenPattern::String("==".to_string()), 10),
    ];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"==", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(2));
    assert_eq!(tok.text, b"==");
    assert_eq!(tok.end, 2);
}

#[test]
fn glr_lexer_first_matching_pattern_wins() {
    // GLRLexer uses symbol ID order (ascending). SymbolId(1) is tried first.
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        ir::Token {
            name: "hex".to_string(),
            pattern: TokenPattern::Regex(r"[0-9a-fA-F]+".to_string()),
            fragile: false,
        },
    );

    // "123" matches both; SymbolId(1) is tried first
    let mut lexer = GLRLexer::new(&grammar, "123".to_string()).unwrap();
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.symbol_id, SymbolId(1));
}

// ---------------------------------------------------------------------------
// 8. Token position tracking (byte offsets)
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_position_tracking() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(2), TokenPattern::String("+".to_string()), 0),
        (SymbolId(3), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(3)]);

    let input = b"10 + 20";
    let tok1 = lexer.next_token(input, 0).unwrap();
    assert_eq!(tok1.start, 0);
    assert_eq!(tok1.end, 2);
    assert_eq!(tok1.text, b"10");

    let tok2 = lexer.next_token(input, tok1.end).unwrap();
    assert_eq!(tok2.start, 3);
    assert_eq!(tok2.end, 4);
    assert_eq!(tok2.text, b"+");

    let tok3 = lexer.next_token(input, tok2.end).unwrap();
    assert_eq!(tok3.start, 5);
    assert_eq!(tok3.end, 7);
    assert_eq!(tok3.text, b"20");
}

#[test]
fn glr_lexer_byte_offset_tracking() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "word".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "ab cd ef".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens[0].byte_offset, 0);
    assert_eq!(tokens[0].byte_length, 2);

    assert_eq!(tokens[1].byte_offset, 3);
    assert_eq!(tokens[1].byte_length, 2);

    assert_eq!(tokens[2].byte_offset, 6);
    assert_eq!(tokens[2].byte_length, 2);
}

#[test]
fn grammar_lexer_position_advances_correctly_across_stream() {
    let patterns = vec![
        (
            SymbolId(1),
            TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            0,
        ),
        (SymbolId(2), TokenPattern::String("=".to_string()), 0),
        (SymbolId(3), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(4), TokenPattern::String(";".to_string()), 0),
        (SymbolId(5), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(5)]);

    let input = b"x = 42;";
    let tokens = collect_grammar_tokens(&mut lexer, input);
    assert_eq!(tokens.len(), 4);

    // Verify positions are contiguous (modulo skipped whitespace)
    assert_eq!(tokens[0].start, 0); // x
    assert_eq!(tokens[0].end, 1);
    assert_eq!(tokens[1].start, 2); // =
    assert_eq!(tokens[1].end, 3);
    assert_eq!(tokens[2].start, 4); // 42
    assert_eq!(tokens[2].end, 6);
    assert_eq!(tokens[3].start, 6); // ;
    assert_eq!(tokens[3].end, 7);
}

// ---------------------------------------------------------------------------
// 9. Error handling for unknown characters
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_returns_none_for_unknown_char() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);

    // '@' doesn't match any pattern
    let result = lexer.next_token(b"@", 0);
    assert!(result.is_none());
}

#[test]
fn error_recovering_lexer_skip_char_mode() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0)];
    let base = GrammarLexer::new(&patterns);
    let error_sym = SymbolId(999);
    let mut lexer = ErrorRecoveringLexer::new(base, error_sym);

    let input = b"@123";
    let tok = lexer.next_token(input, 0).unwrap();
    assert_eq!(tok.symbol, error_sym);
    assert_eq!(tok.text, b"@");
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 1);

    // After skipping the error char, next token is the number
    let tok = lexer.next_token(input, 1).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, b"123");
}

#[test]
fn error_recovering_lexer_skip_to_known_mode() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0)];
    let base = GrammarLexer::new(&patterns);
    let error_sym = SymbolId(999);
    let mut lexer = ErrorRecoveringLexer::new(base, error_sym);
    lexer.set_recovery_mode(ErrorRecoveryMode::SkipToKnown);

    let input = b"@#$123";
    let tok = lexer.next_token(input, 0).unwrap();
    assert_eq!(tok.symbol, error_sym);
    // Should skip all unknown chars until we reach a known token
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 3); // skipped "@#$"
}

#[test]
fn error_recovering_lexer_fail_mode() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0)];
    let base = GrammarLexer::new(&patterns);
    let error_sym = SymbolId(999);
    let mut lexer = ErrorRecoveringLexer::new(base, error_sym);
    lexer.set_recovery_mode(ErrorRecoveryMode::Fail);

    let result = lexer.next_token(b"@", 0);
    assert!(result.is_none());
}

#[test]
fn glr_lexer_skips_unknown_and_continues() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    // '@' is not matched — GLRLexer skips it and continues
    let mut lexer = GLRLexer::new(&grammar, "@ 42".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].text, "42");
}

// ---------------------------------------------------------------------------
// 10. Tokenize empty input
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_empty_input_returns_eof() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"", 0).unwrap();
    // EOF sentinel: symbol 0, empty text
    assert_eq!(tok.symbol, SymbolId(0));
    assert!(tok.text.is_empty());
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 0);
}

#[test]
fn glr_lexer_empty_input_returns_no_tokens() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, String::new()).unwrap();
    let tokens = lexer.tokenize_all();
    assert!(tokens.is_empty());
}

#[test]
fn grammar_lexer_whitespace_only_input() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(2), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(2)]);

    // All whitespace, skipped → lands at EOF
    let tok = lexer.next_token(b"   ", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(0)); // EOF
}

// ---------------------------------------------------------------------------
// 11. Token stream for complex expressions
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_complex_expression_stream() {
    let patterns = vec![
        (
            SymbolId(1),
            TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            0,
        ),
        (SymbolId(2), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(3), TokenPattern::String("+".to_string()), 0),
        (SymbolId(4), TokenPattern::String("*".to_string()), 0),
        (SymbolId(5), TokenPattern::String("(".to_string()), 0),
        (SymbolId(6), TokenPattern::String(")".to_string()), 0),
        (SymbolId(7), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(7)]);

    let input = b"a + b * (2 + 3)";
    let tokens = collect_grammar_tokens(&mut lexer, input);

    let syms: Vec<SymbolId> = tokens.iter().map(|t| t.symbol).collect();
    let texts: Vec<&[u8]> = tokens.iter().map(|t| t.text.as_slice()).collect();

    assert_eq!(
        syms,
        vec![
            SymbolId(1), // a
            SymbolId(3), // +
            SymbolId(1), // b
            SymbolId(4), // *
            SymbolId(5), // (
            SymbolId(2), // 2
            SymbolId(3), // +
            SymbolId(2), // 3
            SymbolId(6), // )
        ]
    );
    assert_eq!(
        texts,
        vec![
            b"a" as &[u8],
            b"+",
            b"b",
            b"*",
            b"(",
            b"2",
            b"+",
            b"3",
            b")",
        ]
    );
}

#[test]
fn glr_lexer_complex_expression_stream() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        ir::Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        ir::Token {
            name: "star".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(4),
        ir::Token {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(5),
        ir::Token {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "1 + 2 * (3 + 4)".to_string()).unwrap();
    let tokens = lexer.tokenize_all();

    let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(texts, vec!["1", "+", "2", "*", "(", "3", "+", "4", ")"]);
}

#[test]
fn grammar_lexer_assignment_statement() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("let".to_string()), 10),
        (
            SymbolId(2),
            TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            1,
        ),
        (SymbolId(3), TokenPattern::String("=".to_string()), 0),
        (SymbolId(4), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(5), TokenPattern::String(";".to_string()), 0),
        (SymbolId(6), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(6)]);

    let input = b"let x = 42;";
    let tokens = collect_grammar_tokens(&mut lexer, input);

    let syms: Vec<SymbolId> = tokens.iter().map(|t| t.symbol).collect();
    assert_eq!(
        syms,
        vec![
            SymbolId(1), // let (keyword, high priority)
            SymbolId(2), // x
            SymbolId(3), // =
            SymbolId(4), // 42
            SymbolId(5), // ;
        ]
    );
}

// ---------------------------------------------------------------------------
// 12. Tokenize with lookahead (priority-based matching)
// ---------------------------------------------------------------------------

#[test]
fn grammar_lexer_keyword_vs_identifier_priority() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("if".to_string()), 10),
        (SymbolId(2), TokenPattern::String("else".to_string()), 10),
        (SymbolId(3), TokenPattern::String("while".to_string()), 10),
        (
            SymbolId(4),
            TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            1,
        ),
        (SymbolId(5), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(5)]);

    // "if" and "else" → keywords; "condition" → identifier
    let input = b"if condition else result";
    let tokens = collect_grammar_tokens(&mut lexer, input);

    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].symbol, SymbolId(1)); // if
    assert_eq!(tokens[1].symbol, SymbolId(4)); // condition (ident)
    assert_eq!(tokens[2].symbol, SymbolId(2)); // else
    assert_eq!(tokens[3].symbol, SymbolId(4)); // result (ident)
}

#[test]
fn grammar_lexer_operator_precedence_by_priority() {
    // "=" vs "==" — higher priority pattern for "==" should win
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("=".to_string()), 1),
        (SymbolId(2), TokenPattern::String("==".to_string()), 10),
    ];
    let mut lexer = GrammarLexer::new(&patterns);

    let tok = lexer.next_token(b"==x", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(2));
    assert_eq!(tok.end, 2);

    // Single "=" when only one char available
    let tok = lexer.next_token(b"=x", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.end, 1);
}

#[test]
fn glr_lexer_reset_and_retokenize() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let mut lexer = GLRLexer::new(&grammar, "1 2 3".to_string()).unwrap();
    let first_pass = lexer.tokenize_all();
    assert_eq!(first_pass.len(), 3);

    lexer.reset();
    let second_pass = lexer.tokenize_all();
    assert_eq!(second_pass.len(), 3);

    // Both passes produce identical tokens
    for (a, b) in first_pass.iter().zip(second_pass.iter()) {
        assert_eq!(a.symbol_id, b.symbol_id);
        assert_eq!(a.text, b.text);
        assert_eq!(a.byte_offset, b.byte_offset);
    }
}

#[test]
fn glr_lexer_invalid_regex_returns_error() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        ir::Token {
            name: "bad".to_string(),
            pattern: TokenPattern::Regex(r"[invalid".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(SymbolId(1), "bad".to_string());

    let result = GLRLexer::new(&grammar, "test".to_string());
    assert!(result.is_err());
    let err = match result {
        Err(e) => e,
        Ok(_) => unreachable!(),
    };
    assert!(err.contains("Invalid regex"));
}
