#![cfg(feature = "pure-rust")]

//! Comprehensive tests for the tokenizer module.

use adze_glr_core::SymbolId;
use adze_runtime::tokenizer::{Matcher, TokenPattern, Tokenizer, TokenizerError, WhitespaceMode};

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

fn literal(id: u16, s: &str) -> TokenPattern {
    TokenPattern {
        symbol_id: SymbolId(id),
        matcher: Matcher::Literal(s.to_string()),
        is_keyword: false,
    }
}

fn keyword(id: u16, s: &str) -> TokenPattern {
    TokenPattern {
        symbol_id: SymbolId(id),
        matcher: Matcher::Literal(s.to_string()),
        is_keyword: true,
    }
}

fn regex_pat(id: u16, re: &str) -> TokenPattern {
    TokenPattern {
        symbol_id: SymbolId(id),
        matcher: Matcher::Regex(regex::Regex::new(re).unwrap()),
        is_keyword: false,
    }
}

fn whitespace_pattern() -> TokenPattern {
    TokenPattern {
        symbol_id: SymbolId(255),
        matcher: Matcher::Regex(regex::Regex::new(r"^\s+").unwrap()),
        is_keyword: false,
    }
}

fn kinds(tokens: &[adze_runtime::Token]) -> Vec<u32> {
    tokens.iter().map(|t| t.kind).collect()
}

// ---------------------------------------------------------------------------
// 1. Empty input always yields a single EOF token
// ---------------------------------------------------------------------------
#[test]
fn empty_input_produces_only_eof() {
    let tok = Tokenizer::new(vec![literal(1, "+")], WhitespaceMode::Skip);
    let tokens = tok.scan(b"").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, 0);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 0);
}

// ---------------------------------------------------------------------------
// 2. Single literal token followed by EOF
// ---------------------------------------------------------------------------
#[test]
fn single_literal_token() {
    let tok = Tokenizer::new(vec![literal(5, "abc")], WhitespaceMode::Skip);
    let tokens = tok.scan(b"abc").unwrap();
    assert_eq!(kinds(&tokens), vec![5, 0]);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 3);
}

// ---------------------------------------------------------------------------
// 3. Multiple consecutive literals
// ---------------------------------------------------------------------------
#[test]
fn multiple_literals_in_sequence() {
    let tok = Tokenizer::new(vec![literal(1, "+"), literal(2, "-")], WhitespaceMode::Skip);
    let tokens = tok.scan(b"+-+").unwrap();
    assert_eq!(kinds(&tokens), vec![1, 2, 1, 0]);
}

// ---------------------------------------------------------------------------
// 4. Regex pattern matches digits
// ---------------------------------------------------------------------------
#[test]
fn regex_matches_digits() {
    let tok = Tokenizer::new(vec![regex_pat(1, r"^\d+")], WhitespaceMode::Skip);
    let tokens = tok.scan(b"42").unwrap();
    assert_eq!(kinds(&tokens), vec![1, 0]);
    assert_eq!(tokens[0].end, 2);
}

// ---------------------------------------------------------------------------
// 5. Maximal munch: longer match wins over shorter
// ---------------------------------------------------------------------------
#[test]
fn maximal_munch_longer_match_wins() {
    let tok = Tokenizer::new(
        vec![literal(1, "+"), literal(2, "++")],
        WhitespaceMode::Skip,
    );
    let tokens = tok.scan(b"++").unwrap();
    assert_eq!(kinds(&tokens), vec![2, 0]);
}

// ---------------------------------------------------------------------------
// 6. Keyword wins over identifier at same length
// ---------------------------------------------------------------------------
#[test]
fn keyword_wins_tie_over_identifier() {
    let tok = Tokenizer::new(
        vec![regex_pat(2, r"^[a-z]+"), keyword(1, "if")],
        WhitespaceMode::Skip,
    );
    let tokens = tok.scan(b"if").unwrap();
    assert_eq!(tokens[0].kind, 1); // keyword
}

// ---------------------------------------------------------------------------
// 7. Identifier wins when it is longer than keyword
// ---------------------------------------------------------------------------
#[test]
fn identifier_wins_when_longer_than_keyword() {
    let tok = Tokenizer::new(
        vec![keyword(1, "if"), regex_pat(2, r"^[a-z]+")],
        WhitespaceMode::Skip,
    );
    let tokens = tok.scan(b"iffy").unwrap();
    assert_eq!(tokens[0].kind, 2); // identifier (4 chars > 2)
    assert_eq!(tokens[0].end, 4);
}

// ---------------------------------------------------------------------------
// 8. Whitespace skip mode omits whitespace tokens
// ---------------------------------------------------------------------------
#[test]
fn whitespace_skip_mode() {
    let tok = Tokenizer::new(
        vec![regex_pat(1, r"^\d+"), whitespace_pattern()],
        WhitespaceMode::Skip,
    );
    let tokens = tok.scan(b"  42  ").unwrap();
    assert_eq!(kinds(&tokens), vec![1, 0]);
    assert_eq!(tokens[0].start, 2);
    assert_eq!(tokens[0].end, 4);
}

// ---------------------------------------------------------------------------
// 9. Whitespace preserve mode emits whitespace tokens
// ---------------------------------------------------------------------------
#[test]
fn whitespace_preserve_mode() {
    let tok = Tokenizer::new(
        vec![regex_pat(1, r"^\d+"), whitespace_pattern()],
        WhitespaceMode::Preserve,
    );
    let tokens = tok.scan(b" 7 ").unwrap();
    assert_eq!(kinds(&tokens), vec![255, 1, 255, 0]);
}

// ---------------------------------------------------------------------------
// 10. Invalid token error at correct position
// ---------------------------------------------------------------------------
#[test]
fn invalid_token_error_position() {
    let tok = Tokenizer::new(vec![literal(1, "a")], WhitespaceMode::Skip);
    let err = tok.scan(b"a!").unwrap_err();
    match err {
        TokenizerError::InvalidToken { position, .. } => assert_eq!(position, 1),
    }
}

// ---------------------------------------------------------------------------
// 11. Invalid token error snippet content
// ---------------------------------------------------------------------------
#[test]
fn invalid_token_error_snippet() {
    let tok = Tokenizer::new(vec![], WhitespaceMode::Skip);
    let err = tok.scan(b"xyz").unwrap_err();
    match err {
        TokenizerError::InvalidToken { snippet, .. } => assert!(snippet.starts_with("xyz")),
    }
}

// ---------------------------------------------------------------------------
// 12. Snippet truncated to at most 20 bytes
// ---------------------------------------------------------------------------
#[test]
fn error_snippet_truncated_to_20_bytes() {
    let tok = Tokenizer::new(vec![], WhitespaceMode::Skip);
    let long_input = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let err = tok.scan(long_input).unwrap_err();
    match err {
        TokenizerError::InvalidToken { snippet, .. } => {
            assert!(snippet.len() <= 20);
        }
    }
}

// ---------------------------------------------------------------------------
// 13. EOF token start == end == input length
// ---------------------------------------------------------------------------
#[test]
fn eof_token_position_equals_input_length() {
    let tok = Tokenizer::new(vec![literal(1, "x")], WhitespaceMode::Skip);
    let tokens = tok.scan(b"xx").unwrap();
    let eof = tokens.last().unwrap();
    assert_eq!(eof.kind, 0);
    assert_eq!(eof.start, 2);
    assert_eq!(eof.end, 2);
}

// ---------------------------------------------------------------------------
// 14. Tokens have no gaps or overlaps
// ---------------------------------------------------------------------------
#[test]
fn no_gaps_or_overlaps_between_tokens() {
    let tok = Tokenizer::new(
        vec![
            regex_pat(1, r"^[a-z]+"),
            literal(2, "+"),
            whitespace_pattern(),
        ],
        WhitespaceMode::Preserve,
    );
    let tokens = tok.scan(b"abc + def").unwrap();
    for pair in tokens.windows(2) {
        assert_eq!(
            pair[0].end, pair[1].start,
            "gap/overlap between {:?} and {:?}",
            pair[0], pair[1]
        );
    }
}

// ---------------------------------------------------------------------------
// 15. Token is Copy
// ---------------------------------------------------------------------------
#[test]
fn token_is_copy() {
    let t = adze_runtime::Token {
        kind: 1,
        start: 0,
        end: 5,
    };
    let t2 = t; // copy
    assert_eq!(t.kind, t2.kind);
    assert_eq!(t.start, t2.start);
    assert_eq!(t.end, t2.end);
}

// ---------------------------------------------------------------------------
// 16. TokenizerError implements Display
// ---------------------------------------------------------------------------
#[test]
fn tokenizer_error_display() {
    let err = TokenizerError::InvalidToken {
        position: 42,
        snippet: "bad".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("42"));
    assert!(msg.contains("bad"));
}

// ---------------------------------------------------------------------------
// 17. TokenizerError implements std::error::Error
// ---------------------------------------------------------------------------
#[test]
fn tokenizer_error_is_std_error() {
    let err = TokenizerError::InvalidToken {
        position: 0,
        snippet: String::new(),
    };
    let _: &dyn std::error::Error = &err;
}

// ---------------------------------------------------------------------------
// 18. Pattern precedence: first keyword wins when two keywords tie
// ---------------------------------------------------------------------------
#[test]
fn first_keyword_wins_when_two_keywords_tie() {
    let tok = Tokenizer::new(
        vec![keyword(1, "do"), keyword(2, "do")],
        WhitespaceMode::Skip,
    );
    let tokens = tok.scan(b"do").unwrap();
    // Both are keywords with same length; first registered keyword already
    // matched, so the second is_keyword && !best_is_keyword branch is false.
    assert_eq!(tokens[0].kind, 1);
}

// ---------------------------------------------------------------------------
// 19. Literal pattern at non-zero offset
// ---------------------------------------------------------------------------
#[test]
fn literal_match_at_nonzero_offset() {
    let tok = Tokenizer::new(vec![literal(1, "a"), literal(2, "b")], WhitespaceMode::Skip);
    let tokens = tok.scan(b"ab").unwrap();
    assert_eq!(tokens[0].kind, 1);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[1].kind, 2);
    assert_eq!(tokens[1].start, 1);
}

// ---------------------------------------------------------------------------
// 20. Multi-character literal
// ---------------------------------------------------------------------------
#[test]
fn multi_char_literal() {
    let tok = Tokenizer::new(vec![literal(1, "===")], WhitespaceMode::Skip);
    let tokens = tok.scan(b"===").unwrap();
    assert_eq!(tokens[0].end - tokens[0].start, 3);
}

// ---------------------------------------------------------------------------
// 21. Unicode input handled correctly
// ---------------------------------------------------------------------------
#[test]
fn unicode_input_byte_positions() {
    // 'é' is 2 bytes in UTF-8
    let tok = Tokenizer::new(vec![regex_pat(1, r"^[a-zé]+")], WhitespaceMode::Skip);
    let tokens = tok.scan("café".as_bytes()).unwrap();
    assert_eq!(tokens[0].kind, 1);
    assert_eq!(tokens[0].start, 0);
    // "café" = 5 bytes (c=1, a=1, f=1, é=2)
    assert_eq!(tokens[0].end, 5);
}

// ---------------------------------------------------------------------------
// 22. Whitespace-only input in skip mode yields EOF only
// ---------------------------------------------------------------------------
#[test]
fn whitespace_only_input_skip_mode() {
    let tok = Tokenizer::new(vec![whitespace_pattern()], WhitespaceMode::Skip);
    let tokens = tok.scan(b"   \t\n  ").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, 0);
}

// ---------------------------------------------------------------------------
// 23. Multiple regex patterns with different precedence
// ---------------------------------------------------------------------------
#[test]
fn multiple_regex_patterns_maximal_munch() {
    let tok = Tokenizer::new(
        vec![
            regex_pat(1, r"^[0-9]"),          // single digit
            regex_pat(2, r"^[0-9]+\.[0-9]+"), // float
        ],
        WhitespaceMode::Skip,
    );
    let tokens = tok.scan(b"3.14").unwrap();
    // float pattern (4 chars) is longer than single digit (1 char)
    assert_eq!(tokens[0].kind, 2);
    assert_eq!(tokens[0].end, 4);
}

// ---------------------------------------------------------------------------
// 24. WhitespaceMode is Copy and PartialEq
// ---------------------------------------------------------------------------
#[test]
fn whitespace_mode_traits() {
    let a = WhitespaceMode::Skip;
    let b = a; // copy
    assert_eq!(a, b);
    assert_ne!(WhitespaceMode::Skip, WhitespaceMode::Preserve);
}

// ---------------------------------------------------------------------------
// 25. Tokenizer is Send + Sync (compile-time check)
// ---------------------------------------------------------------------------
#[test]
fn tokenizer_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Tokenizer>();
}
