//! Comprehensive tests for the SIMD-accelerated lexer module.
//!
//! This test suite covers:
//! - Basic tokenization with different pattern types
//! - Multiple token types in sequence
//! - Edge cases (empty input, single character, long strings)
//! - Error handling and boundary conditions
//! - SIMD-optimized pattern matching (whitespace, digits, identifiers)
//! - Literal string matching with greedy behavior

use adze::simd_lexer::SimdLexer;
use adze_ir::{SymbolId, TokenPattern};

// Test basic whitespace tokenization
#[test]
fn test_whitespace_single_space() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b" ";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));
    assert_eq!(token.start, 0);
    assert_eq!(token.end, 1);
    assert_eq!(token.text, b" ");
}

#[test]
fn test_whitespace_multiple_types() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"    \t\n\r  ";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));
    assert_eq!(token.start, 0);
    assert_eq!(token.end, 9);
}

#[test]
fn test_whitespace_followed_by_text() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"    \t\n  hello";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));
    assert_eq!(token.end, 8);
}

#[test]
fn test_whitespace_long_sequence() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    // Create a long whitespace sequence to test SIMD chunks (>32 bytes)
    let input = vec![b' '; 100];
    let token = lexer.scan(&input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));
    assert_eq!(token.end, 100);
}

// Test digit tokenization
#[test]
fn test_digits_single() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"5";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 1);
}

#[test]
fn test_digits_multiple() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"12345abc";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 5);
    assert_eq!(token.text, b"12345");
}

#[test]
fn test_digits_long_sequence() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    // Create a long digit sequence to test SIMD chunks (>32 bytes)
    let input = b"1234567890123456789012345678901234567890123456789012345678901234567890";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 70);
}

#[test]
fn test_digits_zero_prefixed() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"00123";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 5);
}

// Test identifier tokenization
#[test]
fn test_identifier_simple() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = b"hello";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 5);
}

#[test]
fn test_identifier_with_underscore() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = b"hello_world123 ";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 14);
}

#[test]
fn test_identifier_starting_with_underscore() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = b"_private";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 8);
}

#[test]
fn test_identifier_mixed_case() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = b"CamelCaseVar123";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 15);
}

#[test]
fn test_identifier_long_name() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    // Create a long identifier to test SIMD chunks (>32 bytes)
    let input = b"this_is_a_very_long_identifier_name_that_exceeds_simd_lane_width";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 64);
}

// Test literal string matching
#[test]
fn test_literal_exact_match() {
    let patterns = vec![(SymbolId(4), TokenPattern::String("function".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"function";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(4));
    assert_eq!(token.end, 8);
}

#[test]
fn test_literal_greedy_matching() {
    let patterns = vec![
        (SymbolId(4), TokenPattern::String("function".to_string())),
        (SymbolId(5), TokenPattern::String("func".to_string())),
        (SymbolId(6), TokenPattern::String("fn".to_string())),
    ];
    let lexer = SimdLexer::new(&patterns);

    // Should match "function" (longest) not "func" or "fn"
    let input = b"function";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(4));
    assert_eq!(token.end, 8);
}

#[test]
fn test_literal_partial_match() {
    let patterns = vec![
        (SymbolId(4), TokenPattern::String("function".to_string())),
        (SymbolId(5), TokenPattern::String("func".to_string())),
    ];
    let lexer = SimdLexer::new(&patterns);

    // Should match "func" since "function" doesn't fully match
    let input = b"func(";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(5));
    assert_eq!(token.end, 4);
}

#[test]
fn test_literal_short_string() {
    let patterns = vec![(SymbolId(6), TokenPattern::String("x".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"x";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(6));
    assert_eq!(token.end, 1);
}

#[test]
fn test_literal_long_string() {
    let patterns = vec![(
        SymbolId(7),
        TokenPattern::String("this_is_a_very_long_literal_string_for_testing".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = b"this_is_a_very_long_literal_string_for_testing";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(7));
    assert_eq!(token.end, 46);
}

// Test multiple token types
#[test]
fn test_multiple_patterns_whitespace_first() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\s+".to_string())),
        (SymbolId(2), TokenPattern::Regex(r"\d+".to_string())),
        (
            SymbolId(3),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ),
    ];
    let lexer = SimdLexer::new(&patterns);

    let input = b"   123";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));
    assert_eq!(token.end, 3);

    // Scan next token
    let token = lexer.scan(input, 3).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 6);
}

#[test]
fn test_multiple_patterns_identifier_first() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\s+".to_string())),
        (SymbolId(2), TokenPattern::Regex(r"\d+".to_string())),
        (
            SymbolId(3),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ),
    ];
    let lexer = SimdLexer::new(&patterns);

    let input = b"hello 123";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 5);
}

#[test]
fn test_tokenize_sequence() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\s+".to_string())),
        (SymbolId(2), TokenPattern::Regex(r"\d+".to_string())),
        (
            SymbolId(3),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ),
    ];
    let lexer = SimdLexer::new(&patterns);

    let input = b"hello 123 world 456";
    let mut tokens = Vec::new();
    let mut pos = 0;

    while let Some(token) = lexer.scan(input, pos) {
        tokens.push((token.symbol, token.start, token.end));
        pos = token.end;
    }

    assert_eq!(tokens.len(), 7);
    assert_eq!(tokens[0], (SymbolId(3), 0, 5)); // hello
    assert_eq!(tokens[1], (SymbolId(1), 5, 6)); // space
    assert_eq!(tokens[2], (SymbolId(2), 6, 9)); // 123
    assert_eq!(tokens[3], (SymbolId(1), 9, 10)); // space
    assert_eq!(tokens[4], (SymbolId(3), 10, 15)); // world
    assert_eq!(tokens[5], (SymbolId(1), 15, 16)); // space
    assert_eq!(tokens[6], (SymbolId(2), 16, 19)); // 456
}

// Test edge cases
#[test]
fn test_empty_input() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"";
    let token = lexer.scan(input, 0);
    assert!(token.is_none());
}

#[test]
fn test_scan_at_end() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"hello";
    let token = lexer.scan(input, 5);
    assert!(token.is_none());
}

#[test]
fn test_scan_past_end() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"hello";
    let token = lexer.scan(input, 10);
    assert!(token.is_none());
}

#[test]
fn test_no_match() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"abc";
    let token = lexer.scan(input, 0);
    assert!(token.is_none());
}

#[test]
fn test_single_character_input() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = b"x";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 1);
}

#[test]
fn test_identifier_cannot_start_with_digit() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = b"123abc";
    let token = lexer.scan(input, 0);
    assert!(token.is_none());
}

#[test]
fn test_whitespace_not_at_start() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"hello   world";
    let token = lexer.scan(input, 0);
    assert!(token.is_none()); // No whitespace at position 0

    let token = lexer.scan(input, 5).unwrap();
    assert_eq!(token.symbol, SymbolId(1));
    assert_eq!(token.end, 8);
}

// Test mixed patterns with literals and regexes
#[test]
fn test_mixed_literals_and_patterns() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("if".to_string())),
        (SymbolId(2), TokenPattern::String("else".to_string())),
        (SymbolId(3), TokenPattern::Regex(r"\s+".to_string())),
        (
            SymbolId(4),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ),
    ];
    let lexer = SimdLexer::new(&patterns);

    let input = b"if condition else";
    let mut tokens = Vec::new();
    let mut pos = 0;

    while let Some(token) = lexer.scan(input, pos) {
        tokens.push((token.symbol, token.start, token.end));
        pos = token.end;
    }

    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0], (SymbolId(1), 0, 2)); // if
    assert_eq!(tokens[1], (SymbolId(3), 2, 3)); // space
    assert_eq!(tokens[2], (SymbolId(4), 3, 12)); // condition
    assert_eq!(tokens[3], (SymbolId(3), 12, 13)); // space
    assert_eq!(tokens[4], (SymbolId(2), 13, 17)); // else
}

#[test]
#[ignore = "KNOWN BUG: keyword boundary detection - lexer matches keyword prefix instead of full identifier"]
fn test_keyword_vs_identifier() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("function".to_string())),
        (
            SymbolId(2),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ),
    ];
    let lexer = SimdLexer::new(&patterns);

    // "function" should match as keyword
    let input = b"function";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));

    // "functions" should match as identifier (keyword doesn't match whole word)
    let input = b"functions";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 9);
}

// Test scan position handling
#[test]
fn test_scan_from_middle() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"abc123def456";
    let token = lexer.scan(input, 3).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.start, 3);
    assert_eq!(token.end, 6);
    assert_eq!(token.text, b"123");
}

#[test]
fn test_scan_consecutive_positions() {
    let patterns = vec![(SymbolId(3), TokenPattern::String("x".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"xxx";

    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.start, 0);
    assert_eq!(token.end, 1);

    let token = lexer.scan(input, 1).unwrap();
    assert_eq!(token.start, 1);
    assert_eq!(token.end, 2);

    let token = lexer.scan(input, 2).unwrap();
    assert_eq!(token.start, 2);
    assert_eq!(token.end, 3);
}

// Test UTF-8 handling
#[test]
fn test_ascii_only() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = "hello_world".as_bytes();
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    assert_eq!(token.end, 11);
}

#[test]
fn test_non_ascii_stops_identifier() {
    let patterns = vec![(
        SymbolId(3),
        TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
    )];
    let lexer = SimdLexer::new(&patterns);

    let input = "hello_wörld".as_bytes();
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));
    // Should stop at the ö character (ö is 2 bytes in UTF-8, starts at position 7)
    // The regex [a-zA-Z_][a-zA-Z0-9_]* matches "hello_w" (7 bytes) before hitting
    // the non-ASCII byte of ö
    assert_eq!(token.end, 7); // "hello_w"
}

// Test pattern priority
#[test]
fn test_pattern_order_matters() {
    // Literals are checked first (sorted by length), then patterns in order
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\d+".to_string())),
        (
            SymbolId(2),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ),
    ];
    let lexer = SimdLexer::new(&patterns);

    // "123abc" should match digits first
    let input = b"123abc";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));
    assert_eq!(token.end, 3);
}

#[test]
fn test_zero_width_match_prevented() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"\s+".to_string())),
        (SymbolId(2), TokenPattern::Regex(r"\d+".to_string())),
    ];
    let lexer = SimdLexer::new(&patterns);

    // Input with no matching patterns should return None
    let input = b"abc";
    let token = lexer.scan(input, 0);
    assert!(token.is_none());
}

// Test boundary conditions
#[test]
fn test_exact_simd_lane_width() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    // Exactly 32 bytes (SIMD lane width)
    let input = b"12345678901234567890123456789012";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 32);
}

#[test]
fn test_just_over_simd_lane_width() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    // 33 bytes (one over SIMD lane width)
    let input = b"123456789012345678901234567890123";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 33);
}

#[test]
fn test_just_under_simd_lane_width() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    // 31 bytes (one under SIMD lane width)
    let input = b"1234567890123456789012345678901";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 31);
}

// Test complete tokenization
#[test]
fn test_complete_tokenization() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("let".to_string())),
        (SymbolId(2), TokenPattern::String("=".to_string())),
        (SymbolId(3), TokenPattern::String(";".to_string())),
        (SymbolId(4), TokenPattern::Regex(r"\s+".to_string())),
        (
            SymbolId(5),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ),
        (SymbolId(6), TokenPattern::Regex(r"\d+".to_string())),
    ];
    let lexer = SimdLexer::new(&patterns);

    let input = b"let x = 42;";
    let mut tokens = Vec::new();
    let mut pos = 0;

    while let Some(token) = lexer.scan(input, pos) {
        tokens.push(token.symbol);
        pos = token.end;
    }

    assert_eq!(
        tokens,
        vec![
            SymbolId(1), // let
            SymbolId(4), // space
            SymbolId(5), // x
            SymbolId(4), // space
            SymbolId(2), // =
            SymbolId(4), // space
            SymbolId(6), // 42
            SymbolId(3), // ;
        ]
    );
}

#[test]
fn test_all_digit_characters() {
    let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    let input = b"0123456789";
    let token = lexer.scan(input, 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));
    assert_eq!(token.end, 10);
}

#[test]
fn test_all_whitespace_types_individually() {
    let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
    let lexer = SimdLexer::new(&patterns);

    // Space
    let token = lexer.scan(b" ", 0).unwrap();
    assert_eq!(token.end, 1);

    // Tab
    let token = lexer.scan(b"\t", 0).unwrap();
    assert_eq!(token.end, 1);

    // Newline
    let token = lexer.scan(b"\n", 0).unwrap();
    assert_eq!(token.end, 1);

    // Carriage return
    let token = lexer.scan(b"\r", 0).unwrap();
    assert_eq!(token.end, 1);
}

#[test]
fn test_literal_with_special_characters() {
    let patterns = vec![
        (SymbolId(1), TokenPattern::String("==".to_string())),
        (SymbolId(2), TokenPattern::String("!=".to_string())),
        (SymbolId(3), TokenPattern::String("<=".to_string())),
        (SymbolId(4), TokenPattern::String(">=".to_string())),
    ];
    let lexer = SimdLexer::new(&patterns);

    let token = lexer.scan(b"==", 0).unwrap();
    assert_eq!(token.symbol, SymbolId(1));

    let token = lexer.scan(b"!=", 0).unwrap();
    assert_eq!(token.symbol, SymbolId(2));

    let token = lexer.scan(b"<=", 0).unwrap();
    assert_eq!(token.symbol, SymbolId(3));

    let token = lexer.scan(b">=", 0).unwrap();
    assert_eq!(token.symbol, SymbolId(4));
}
