//! Comprehensive v3 tests for the lexer module.
//!
//! Covers:
//! 1. Token construction
//! 2. Token properties
//! 3. Token Debug/Clone
//! 4. LexerConfig (ErrorRecoveryMode) construction and defaults
//! 5. Token range validation
//! 6. Token ordering
//! 7. Various token patterns
//! 8. Edge cases

use adze::lexer::{ErrorRecoveringLexer, ErrorRecoveryMode, GrammarLexer, Token};
use adze_ir::{SymbolId, TokenPattern};

// =========================================================================
// Helpers
// =========================================================================

fn make_token(symbol: u16, text: &[u8], start: usize, end: usize) -> Token {
    Token {
        symbol: SymbolId(symbol),
        text: text.to_vec(),
        start,
        end,
    }
}

fn collect_tokens(lexer: &mut GrammarLexer, input: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        match lexer.next_token(input, pos) {
            Some(tok) if tok.symbol == SymbolId(0) => break,
            Some(tok) => {
                pos = tok.end;
                tokens.push(tok);
            }
            None => break,
        }
    }
    tokens
}

// =========================================================================
// 1. Token construction (8 tests)
// =========================================================================

#[test]
fn token_construction_basic() {
    let tok = make_token(1, b"hello", 0, 5);
    assert_eq!(tok.symbol, SymbolId(1));
    assert_eq!(tok.text, b"hello");
}

#[test]
fn token_construction_empty_text() {
    let tok = make_token(0, b"", 0, 0);
    assert_eq!(tok.text.len(), 0);
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 0);
}

#[test]
fn token_construction_single_byte() {
    let tok = make_token(5, b"+", 10, 11);
    assert_eq!(tok.text, b"+");
    assert_eq!(tok.end - tok.start, 1);
}

#[test]
fn token_construction_multibyte_utf8() {
    let tok = make_token(3, "λ".as_bytes(), 0, 2);
    assert_eq!(tok.text, "λ".as_bytes());
}

#[test]
fn token_construction_large_symbol_id() {
    let tok = make_token(u16::MAX, b"x", 0, 1);
    assert_eq!(tok.symbol, SymbolId(u16::MAX));
}

#[test]
fn token_construction_zero_symbol_id() {
    let tok = make_token(0, b"eof", 100, 103);
    assert_eq!(tok.symbol, SymbolId(0));
}

#[test]
fn token_construction_binary_text() {
    let tok = make_token(7, &[0x00, 0xFF, 0x80], 0, 3);
    assert_eq!(tok.text, &[0x00, 0xFF, 0x80]);
}

#[test]
fn token_construction_long_text() {
    let long = vec![b'a'; 1024];
    let tok = make_token(1, &long, 0, 1024);
    assert_eq!(tok.text.len(), 1024);
}

// =========================================================================
// 2. Token properties (8 tests)
// =========================================================================

#[test]
fn token_text_length_matches_span() {
    let tok = make_token(1, b"abc", 5, 8);
    assert_eq!(tok.text.len(), tok.end - tok.start);
}

#[test]
fn token_symbol_equality() {
    let tok1 = make_token(42, b"x", 0, 1);
    let tok2 = make_token(42, b"y", 0, 1);
    assert_eq!(tok1.symbol, tok2.symbol);
}

#[test]
fn token_symbol_inequality() {
    let tok1 = make_token(1, b"x", 0, 1);
    let tok2 = make_token(2, b"x", 0, 1);
    assert_ne!(tok1.symbol, tok2.symbol);
}

#[test]
fn token_start_before_end() {
    let tok = make_token(1, b"test", 10, 14);
    assert!(tok.start < tok.end);
}

#[test]
fn token_span_size() {
    let tok = make_token(1, b"hello", 3, 8);
    assert_eq!(tok.end - tok.start, 5);
}

#[test]
fn token_text_is_owned() {
    let data = b"owned".to_vec();
    let tok = Token {
        symbol: SymbolId(1),
        text: data.clone(),
        start: 0,
        end: 5,
    };
    // Modifying original doesn't affect token
    assert_eq!(tok.text, b"owned");
}

#[test]
fn token_different_texts_different_tokens() {
    let tok1 = make_token(1, b"abc", 0, 3);
    let tok2 = make_token(1, b"def", 0, 3);
    assert_ne!(tok1, tok2);
}

#[test]
fn token_equality_requires_all_fields() {
    let tok1 = make_token(1, b"x", 0, 1);
    let tok2 = make_token(1, b"x", 0, 1);
    let tok3 = make_token(1, b"x", 1, 2);
    assert_eq!(tok1, tok2);
    assert_ne!(tok1, tok3);
}

// =========================================================================
// 3. Token Debug/Clone (5 tests)
// =========================================================================

#[test]
fn token_debug_format() {
    let tok = make_token(1, b"hi", 0, 2);
    let dbg = format!("{:?}", tok);
    assert!(dbg.contains("Token"));
    assert!(dbg.contains("symbol"));
}

#[test]
fn token_clone_is_equal() {
    let tok = make_token(5, b"clone_me", 10, 18);
    let cloned = tok.clone();
    assert_eq!(tok, cloned);
}

#[test]
fn token_clone_is_independent() {
    let tok = make_token(1, b"independent", 0, 11);
    let mut cloned = tok.clone();
    cloned.start = 99;
    assert_ne!(tok.start, cloned.start);
}

#[test]
fn token_debug_contains_positions() {
    let tok = make_token(3, b"pos", 42, 45);
    let dbg = format!("{:?}", tok);
    assert!(dbg.contains("42"));
    assert!(dbg.contains("45"));
}

#[test]
fn error_recovery_mode_debug() {
    let mode = ErrorRecoveryMode::SkipChar;
    let dbg = format!("{:?}", mode);
    assert!(dbg.contains("SkipChar"));
}

// =========================================================================
// 4. ErrorRecoveryMode construction and defaults (5 tests)
// =========================================================================

#[test]
fn error_recovery_mode_skip_char_variant() {
    let mode = ErrorRecoveryMode::SkipChar;
    assert_eq!(mode, ErrorRecoveryMode::SkipChar);
}

#[test]
fn error_recovery_mode_skip_to_known_variant() {
    let mode = ErrorRecoveryMode::SkipToKnown;
    assert_eq!(mode, ErrorRecoveryMode::SkipToKnown);
}

#[test]
fn error_recovery_mode_fail_variant() {
    let mode = ErrorRecoveryMode::Fail;
    assert_eq!(mode, ErrorRecoveryMode::Fail);
}

#[test]
fn error_recovery_mode_clone() {
    let mode = ErrorRecoveryMode::SkipChar;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn error_recovery_mode_variants_distinct() {
    assert_ne!(ErrorRecoveryMode::SkipChar, ErrorRecoveryMode::Fail);
    assert_ne!(ErrorRecoveryMode::SkipToKnown, ErrorRecoveryMode::Fail);
    assert_ne!(ErrorRecoveryMode::SkipChar, ErrorRecoveryMode::SkipToKnown);
}

// =========================================================================
// 5. Token range validation (8 tests)
// =========================================================================

#[test]
fn lexer_token_start_end_match_literal() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("abc".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"abc", 0).unwrap();
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 3);
}

#[test]
fn lexer_token_range_at_offset() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("xy".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"__xy__", 2).unwrap();
    assert_eq!(tok.start, 2);
    assert_eq!(tok.end, 4);
}

#[test]
fn lexer_token_range_regex_variable_length() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[0-9]+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"12345", 0).unwrap();
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 5);
}

#[test]
fn lexer_token_range_single_char_literal() {
    let patterns = vec![(SymbolId(1), TokenPattern::String(";".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b";", 0).unwrap();
    assert_eq!(tok.end - tok.start, 1);
}

#[test]
fn lexer_token_range_multi_char_literal() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("===".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"===", 0).unwrap();
    assert_eq!(tok.end - tok.start, 3);
}

#[test]
fn lexer_eof_token_has_zero_length() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("a".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    // At position past input — returns EOF sentinel
    let tok = lexer.next_token(b"", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(0));
    assert_eq!(tok.start, tok.end);
}

#[test]
fn lexer_token_text_bytes_match_range() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[a-z]+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let input = b"hello world";
    let tok = lexer.next_token(input, 0).unwrap();
    assert_eq!(&input[tok.start..tok.end], tok.text.as_slice());
}

#[test]
fn lexer_skipped_whitespace_adjusts_range() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"[a-z]+".to_string()), 0),
        (SymbolId(99), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(99)]);
    let tok = lexer.next_token(b"   abc", 0).unwrap();
    assert_eq!(tok.start, 3);
    assert_eq!(tok.end, 6);
}

// =========================================================================
// 6. Token ordering (5 tests)
// =========================================================================

#[test]
fn tokens_sequential_non_overlapping() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[a-z]+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let toks = collect_tokens(&mut lexer, b"abcdef");
    // Single token covering the entire word
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].start, 0);
    assert_eq!(toks[0].end, 6);
}

#[test]
fn multiple_tokens_sequential_order() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"[a-z]+".to_string()), 0),
        (SymbolId(2), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    let toks = collect_tokens(&mut lexer, b"aa bb");
    // aa, " ", bb
    assert_eq!(toks.len(), 3);
    assert!(toks[0].end <= toks[1].start);
    assert!(toks[1].end <= toks[2].start);
}

#[test]
fn token_positions_are_monotonically_increasing() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("+".to_string()), 0),
        (SymbolId(2), TokenPattern::Regex(r"[0-9]+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    let toks = collect_tokens(&mut lexer, b"1+2+3");
    for i in 1..toks.len() {
        assert!(toks[i].start >= toks[i - 1].end);
    }
}

#[test]
fn priority_ordering_keyword_over_ident() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"[a-z]+".to_string()), 1),
        (SymbolId(2), TokenPattern::String("for".to_string()), 10),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"for", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(2));
}

#[test]
fn priority_ordering_longest_literal() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("=".to_string()), 0),
        (SymbolId(2), TokenPattern::String("==".to_string()), 10),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"==", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(2));
}

// =========================================================================
// 7. Various token patterns (8 tests)
// =========================================================================

#[test]
fn pattern_literal_string_exact_match() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("let".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"let", 0).unwrap();
    assert_eq!(tok.text, b"let");
}

#[test]
fn pattern_literal_string_no_partial() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("let".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    // "le" does not match "let"
    let tok = lexer.next_token(b"le", 0);
    // Should be EOF (position 0 >= 0... input has length 2, no match for "let")
    assert!(tok.is_none() || tok.as_ref().unwrap().symbol == SymbolId(0));
}

#[test]
fn pattern_regex_digits() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[0-9]+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"42", 0).unwrap();
    assert_eq!(tok.text, b"42");
}

#[test]
fn pattern_regex_identifier() {
    let patterns = vec![(
        SymbolId(1),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        0,
    )];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"_foo123", 0).unwrap();
    assert_eq!(tok.text, b"_foo123");
}

#[test]
fn pattern_regex_float() {
    let patterns = vec![(
        SymbolId(1),
        TokenPattern::Regex(r"[0-9]+\.[0-9]+".to_string()),
        0,
    )];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"3.14", 0).unwrap();
    assert_eq!(tok.text, b"3.14");
}

#[test]
fn pattern_regex_quoted_string() {
    let patterns = vec![(
        SymbolId(1),
        TokenPattern::Regex(r#""[^"]*""#.to_string()),
        0,
    )];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(br#""hello""#, 0).unwrap();
    assert_eq!(tok.text, br#""hello""#);
}

#[test]
fn pattern_multiple_literals() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("(".to_string()), 0),
        (SymbolId(2), TokenPattern::String(")".to_string()), 0),
        (SymbolId(3), TokenPattern::String(",".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    let toks = collect_tokens(&mut lexer, b"(,)");
    assert_eq!(toks.len(), 3);
    assert_eq!(toks[0].symbol, SymbolId(1));
    assert_eq!(toks[1].symbol, SymbolId(3));
    assert_eq!(toks[2].symbol, SymbolId(2));
}

#[test]
fn pattern_mixed_literal_and_regex() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("+".to_string()), 0),
        (SymbolId(2), TokenPattern::Regex(r"[0-9]+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    let toks = collect_tokens(&mut lexer, b"1+2");
    assert_eq!(toks.len(), 3);
    assert_eq!(toks[0].symbol, SymbolId(2));
    assert_eq!(toks[1].symbol, SymbolId(1));
    assert_eq!(toks[2].symbol, SymbolId(2));
}

// =========================================================================
// 8. Edge cases (8 tests)
// =========================================================================

#[test]
fn edge_empty_input_returns_eof() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("a".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(0));
}

#[test]
fn edge_no_matching_pattern_returns_none() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[0-9]+".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let result = lexer.next_token(b"abc", 0);
    assert!(result.is_none());
}

#[test]
fn edge_error_recovery_fail_mode() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[0-9]+".to_string()), 0)];
    let base = GrammarLexer::new(&patterns);
    let mut lexer = ErrorRecoveringLexer::new(base, SymbolId(999));
    lexer.set_recovery_mode(ErrorRecoveryMode::Fail);
    let result = lexer.next_token(b"abc", 0);
    assert!(result.is_none());
}

#[test]
fn edge_error_recovery_skip_char_produces_error_token() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[0-9]+".to_string()), 0)];
    let base = GrammarLexer::new(&patterns);
    let mut lexer = ErrorRecoveringLexer::new(base, SymbolId(999));
    lexer.set_recovery_mode(ErrorRecoveryMode::SkipChar);
    let tok = lexer.next_token(b"@1", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(999));
    assert_eq!(tok.text, b"@");
}

#[test]
fn edge_error_recovery_skip_to_known() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"[0-9]+".to_string()), 0)];
    let base = GrammarLexer::new(&patterns);
    let mut lexer = ErrorRecoveringLexer::new(base, SymbolId(999));
    lexer.set_recovery_mode(ErrorRecoveryMode::SkipToKnown);
    let tok = lexer.next_token(b"abc123", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(999));
    // Should skip "abc" (3 chars) to reach "123"
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 3);
}

#[test]
fn edge_position_past_input_returns_eof() {
    let patterns = vec![(SymbolId(1), TokenPattern::String("a".to_string()), 0)];
    let mut lexer = GrammarLexer::new(&patterns);
    let tok = lexer.next_token(b"a", 5).unwrap();
    assert_eq!(tok.symbol, SymbolId(0));
}

#[test]
fn edge_skip_only_whitespace_input() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"[a-z]+".to_string()), 0),
        (SymbolId(99), TokenPattern::Regex(r"\s+".to_string()), 0),
    ];
    let mut lexer = GrammarLexer::new(&patterns);
    lexer.set_skip_symbols(vec![SymbolId(99)]);
    // Only whitespace — after skipping, should get EOF
    let tok = lexer.next_token(b"   ", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(0));
}

#[test]
fn edge_lexer_no_patterns() {
    let patterns: Vec<(SymbolId, TokenPattern, i32)> = vec![];
    let mut lexer = GrammarLexer::new(&patterns);
    // No patterns means nothing matches — EOF on empty
    let tok = lexer.next_token(b"", 0).unwrap();
    assert_eq!(tok.symbol, SymbolId(0));
    // Non-empty returns None
    let result = lexer.next_token(b"x", 0);
    assert!(result.is_none());
}
