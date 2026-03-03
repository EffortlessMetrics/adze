//! Comprehensive tests for the Parser API surface.
//!
//! Covers: Parser::new, set_language, language, parse, parse_utf8,
//! set_timeout, timeout, reset, error handling, and mode switching.

use adze_runtime::error::ParseError;
use adze_runtime::language::{Language, SymbolMetadata};
use adze_runtime::parser::Parser;
use std::time::Duration;

fn minimal_language() -> Language {
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        }])
        .tokenizer(|_input: &[u8]| {
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = adze_runtime::Token>>
        })
        .build()
        .unwrap()
}

#[test]
fn parser_new_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn parser_new_has_no_timeout() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_set_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn parser_set_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    parser.set_timeout(Duration::from_millis(100));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));
}

#[test]
fn parser_set_language_success() {
    let mut parser = Parser::new();
    let lang = minimal_language();
    let result = parser.set_language(lang);
    assert!(result.is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn parser_set_language_empty_metadata_fails() {
    let mut parser = Parser::new();
    // Build with empty metadata succeeds at builder level, but set_language rejects it
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    let lang = Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_metadata(vec![])
        .tokenizer(|_input: &[u8]| {
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = adze_runtime::Token>>
        })
        .build();
    // Builder may reject empty metadata
    if let Ok(lang) = lang {
        let result = parser.set_language(lang);
        assert!(result.is_err());
    }
}

#[test]
fn parser_set_language_no_tokenizer_check() {
    let mut parser = Parser::new();
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    let lang = Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        }])
        .build()
        .unwrap();
    // Parser requires tokenizer for set_language
    let result = parser.set_language(lang);
    assert!(result.is_err());
}

#[test]
fn parser_parse_without_language_fails() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_utf8_without_language_fails() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

#[test]
fn parser_language_returns_ref_after_set() {
    let mut parser = Parser::new();
    let lang = minimal_language();
    parser.set_language(lang).unwrap();
    let lang_ref = parser.language().unwrap();
    assert!(!lang_ref.symbol_metadata.is_empty());
}

#[test]
fn parser_reset_does_not_clear_language() {
    let mut parser = Parser::new();
    let lang = minimal_language();
    parser.set_language(lang).unwrap();
    parser.reset();
    assert!(parser.language().is_some());
}

#[test]
fn parser_reset_does_not_clear_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(10)));
}

#[test]
fn parser_debug_impl() {
    let parser = Parser::new();
    let debug = format!("{:?}", parser);
    assert!(debug.contains("Parser"));
}

#[test]
fn parser_set_language_twice_replaces() {
    let mut parser = Parser::new();
    let lang1 = minimal_language();
    let lang2 = {
        let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
        Language::builder()
            .version(14)
            .parse_table(table)
            .symbol_metadata(vec![
                SymbolMetadata {
                    is_terminal: false,
                    is_visible: true,
                    is_supertype: false,
                },
                SymbolMetadata {
                    is_terminal: true,
                    is_visible: true,
                    is_supertype: false,
                },
            ])
            .tokenizer(|_input: &[u8]| {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = adze_runtime::Token>>
            })
            .build()
            .unwrap()
    };
    parser.set_language(lang1).unwrap();
    assert_eq!(parser.language().unwrap().symbol_metadata.len(), 1);
    parser.set_language(lang2).unwrap();
    assert_eq!(parser.language().unwrap().symbol_metadata.len(), 2);
}

#[test]
fn parse_error_display() {
    let err = ParseError::no_language();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_with_msg() {
    let err = ParseError::with_msg("custom error");
    let msg = format!("{}", err);
    assert!(msg.contains("custom"));
}

#[test]
fn parser_zero_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn parser_large_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(3600));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3600)));
}

#[test]
fn parser_parse_empty_input_without_language() {
    let mut parser = Parser::new();
    let result = parser.parse(b"", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_utf8_empty_input_without_language() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("", None);
    assert!(result.is_err());
}
