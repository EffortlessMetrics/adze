//! Tests for the tokenizer module — scanning, patterns, whitespace modes.
#![cfg(feature = "pure-rust")]

use adze_ir::SymbolId;
use adze_runtime::Token;
use adze_runtime::tokenizer::*;

fn ws_pattern() -> TokenPattern {
    TokenPattern {
        symbol_id: SymbolId(255),
        matcher: Matcher::Regex(regex::Regex::new(r"[ \t\n]+").unwrap()),
        is_keyword: false,
    }
}

#[test]
fn tokenizer_empty_input() {
    let tokenizer = Tokenizer::new(
        vec![TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Literal("hello".to_string()),
            is_keyword: false,
        }],
        WhitespaceMode::Skip,
    );
    let tokens = tokenizer.scan(b"").unwrap();
    // Only EOF token
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, 0);
}

#[test]
fn tokenizer_single_literal() {
    let tokenizer = Tokenizer::new(
        vec![TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Literal("hello".to_string()),
            is_keyword: false,
        }],
        WhitespaceMode::Skip,
    );
    let tokens = tokenizer.scan(b"hello").unwrap();
    // hello + EOF
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 5);
    assert_eq!(tokens[1].kind, 0); // EOF
}

#[test]
fn tokenizer_multiple_literals() {
    let tokenizer = Tokenizer::new(
        vec![
            TokenPattern {
                symbol_id: SymbolId(1),
                matcher: Matcher::Literal("hello".to_string()),
                is_keyword: false,
            },
            TokenPattern {
                symbol_id: SymbolId(2),
                matcher: Matcher::Literal("world".to_string()),
                is_keyword: false,
            },
            ws_pattern(),
        ],
        WhitespaceMode::Skip,
    );
    let tokens = tokenizer.scan(b"hello world").unwrap();
    // hello + world + EOF
    assert_eq!(tokens.len(), 3);
}

#[test]
fn tokenizer_regex_pattern() {
    let tokenizer = Tokenizer::new(
        vec![TokenPattern {
            symbol_id: SymbolId(10),
            matcher: Matcher::Regex(regex::Regex::new(r"[0-9]+").unwrap()),
            is_keyword: false,
        }],
        WhitespaceMode::Skip,
    );
    let tokens = tokenizer.scan(b"123").unwrap();
    // number + EOF
    assert_eq!(tokens.len(), 2);
}

#[test]
fn tokenizer_whitespace_skip() {
    let tokenizer = Tokenizer::new(
        vec![
            TokenPattern {
                symbol_id: SymbolId(1),
                matcher: Matcher::Literal("a".to_string()),
                is_keyword: false,
            },
            ws_pattern(),
        ],
        WhitespaceMode::Skip,
    );
    let tokens = tokenizer.scan(b"a  a  a").unwrap();
    // 3 a's + EOF
    assert_eq!(tokens.len(), 4);
}

#[test]
fn tokenizer_invalid_token() {
    let tokenizer = Tokenizer::new(
        vec![TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Literal("hello".to_string()),
            is_keyword: false,
        }],
        WhitespaceMode::Skip,
    );
    let result = tokenizer.scan(b"goodbye");
    assert!(result.is_err());
}

#[test]
fn tokenizer_error_debug() {
    let tokenizer = Tokenizer::new(
        vec![TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Literal("hello".to_string()),
            is_keyword: false,
        }],
        WhitespaceMode::Skip,
    );
    if let Err(err) = tokenizer.scan(b"xyz") {
        let debug = format!("{err:?}");
        assert!(!debug.is_empty());
    }
}

#[test]
fn token_struct() {
    let t = Token {
        kind: 5,
        start: 10,
        end: 20,
    };
    assert_eq!(t.kind, 5);
    assert_eq!(t.start, 10);
    assert_eq!(t.end, 20);
}

#[test]
fn matcher_debug() {
    let lit = Matcher::Literal("test".to_string());
    let debug = format!("{lit:?}");
    assert!(debug.contains("Literal"));

    let re = Matcher::Regex(regex::Regex::new(r"\d+").unwrap());
    let debug = format!("{re:?}");
    assert!(debug.contains("Regex"));
}

#[test]
fn whitespace_mode_eq() {
    assert_eq!(WhitespaceMode::Skip, WhitespaceMode::Skip);
    assert_eq!(WhitespaceMode::Preserve, WhitespaceMode::Preserve);
    assert_ne!(WhitespaceMode::Skip, WhitespaceMode::Preserve);
}
