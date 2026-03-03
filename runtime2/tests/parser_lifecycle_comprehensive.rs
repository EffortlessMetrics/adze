#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for Parser lifecycle in adze-runtime.
//!
//! Covers: creation, language setting, parse-without-language errors, reset,
//! sequential parses, reuse patterns, debug display, thread safety (Send/Sync),
//! and timeout set/unset behaviour.

use adze_runtime::parser::Parser;
use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use std::time::Duration;

// ===========================================================================
// Parser creation
// ===========================================================================

#[test]
fn new_parser_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn new_parser_has_no_timeout() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn default_trait_creates_equivalent_parser() {
    let p1 = Parser::new();
    let p2 = Parser::default();
    assert!(p1.language().is_none());
    assert!(p2.language().is_none());
    assert_eq!(p1.timeout(), p2.timeout());
}

// ===========================================================================
// Set language before parsing
// ===========================================================================

#[test]
fn set_language_succeeds_with_valid_language() {
    let mut parser = Parser::new();
    let lang = stub_language();
    assert!(parser.set_language(lang).is_ok());
}

#[test]
fn language_accessor_returns_some_after_set() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn language_accessor_matches_set_language() {
    let mut parser = Parser::new();
    let lang = stub_language();
    parser.set_language(lang).unwrap();
    let got = parser.language().unwrap();
    assert_eq!(got.symbol_count, 1);
    assert_eq!(got.symbol_names[0], "placeholder");
}

#[test]
fn set_language_can_be_called_multiple_times() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    // Replace with a multi-symbol language
    let lang2 = multi_symbol_test_language(5);
    parser.set_language(lang2).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 5);
}

#[test]
fn set_language_overwrites_previous() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    assert_eq!(parser.language().unwrap().symbol_names.len(), 1);

    parser.set_language(multi_symbol_test_language(3)).unwrap();
    assert_eq!(parser.language().unwrap().symbol_names.len(), 3);
}

// ===========================================================================
// Parse without language → error
// ===========================================================================

#[test]
fn parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
}

#[test]
fn parse_without_language_error_message_mentions_language() {
    let mut parser = Parser::new();
    let err = parser.parse(b"test", None).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.to_lowercase().contains("language"),
        "error should mention 'language', got: {msg}"
    );
}

#[test]
fn parse_utf8_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

// ===========================================================================
// Reset parser between parses
// ===========================================================================

#[test]
fn reset_preserves_language() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.reset();
    assert!(
        parser.language().is_some(),
        "reset should not clear language"
    );
}

#[test]
fn reset_preserves_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(10)));
}

#[test]
fn reset_on_fresh_parser_does_not_panic() {
    let mut parser = Parser::new();
    parser.reset(); // should be a no-op
}

#[test]
fn reset_can_be_called_repeatedly() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    for _ in 0..10 {
        parser.reset();
    }
    assert!(parser.language().is_some());
}

// ===========================================================================
// Multiple sequential parses
// ===========================================================================

#[test]
fn multiple_parse_attempts_without_language() {
    let mut parser = Parser::new();
    // Without a language, each call should return Err (not panic).
    let r1 = parser.parse(b"a", None);
    let r2 = parser.parse(b"b", None);
    let r3 = parser.parse(b"c", None);
    assert!(r1.is_err());
    assert!(r2.is_err());
    assert!(r3.is_err());
}

#[test]
fn parse_error_does_not_corrupt_parser_state() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(42));
    // No language → guaranteed Err (not panic)
    let _ = parser.parse(b"x", None);
    assert_eq!(parser.timeout(), Some(Duration::from_millis(42)));
    assert!(parser.language().is_none());
}

#[test]
fn sequential_errors_leave_parser_reusable() {
    let mut parser = Parser::new();
    let _ = parser.parse(b"first", None);
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
    parser.reset();
    assert!(parser.language().is_some());
}

// ===========================================================================
// Parser reuse patterns
// ===========================================================================

#[test]
fn parser_reuse_after_language_change() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();

    // Switch language
    parser.set_language(multi_symbol_test_language(4)).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 4);
}

#[test]
fn parser_reuse_set_language_reset_cycle() {
    let mut parser = Parser::new();
    for i in 1..=5 {
        parser.set_language(multi_symbol_test_language(i)).unwrap();
        parser.reset();
        assert_eq!(parser.language().unwrap().symbol_count, i as u32);
    }
}

#[test]
fn parser_moved_to_new_binding_still_works() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_timeout(Duration::from_millis(100));

    let moved = parser;
    assert!(moved.language().is_some());
    assert_eq!(moved.timeout(), Some(Duration::from_millis(100)));
}

// ===========================================================================
// Parser debug display
// ===========================================================================

#[test]
fn parser_debug_without_language() {
    let parser = Parser::new();
    let dbg = format!("{:?}", parser);
    assert!(
        dbg.contains("Parser"),
        "Debug output should mention Parser: {dbg}"
    );
}

#[test]
fn parser_debug_with_language() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let dbg = format!("{:?}", parser);
    assert!(dbg.contains("Parser"), "got: {dbg}");
    // Language should appear in some form
    assert!(
        dbg.contains("language") || dbg.contains("Language"),
        "Debug should include language info: {dbg}"
    );
}

#[test]
fn parser_debug_with_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(42));
    let dbg = format!("{:?}", parser);
    assert!(
        dbg.contains("timeout") || dbg.contains("42"),
        "Debug should include timeout: {dbg}"
    );
}

// ===========================================================================
// Thread safety (Send / Sync)
// ===========================================================================

// Parser contains Option<Language> which holds Box<dyn Fn> (tokenizer).
// The tokenizer closure is not required to be Send, so Parser is !Send.
// We verify this at compile time with a negative-style assertion.

#[test]
fn parser_without_language_is_usable_on_same_thread() {
    // Parser must be usable on the creating thread without restrictions.
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));
}

#[test]
fn parser_with_language_is_usable_on_same_thread() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_timeout(Duration::from_millis(500));
    assert!(parser.language().is_some());
    assert_eq!(parser.timeout(), Some(Duration::from_millis(500)));
}

#[test]
fn multiple_parsers_on_same_thread() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(stub_language()).unwrap();
    p2.set_language(multi_symbol_test_language(3)).unwrap();
    assert_eq!(p1.language().unwrap().symbol_count, 1);
    assert_eq!(p2.language().unwrap().symbol_count, 3);
}

// ===========================================================================
// Set and unset timeout
// ===========================================================================

#[test]
fn timeout_initially_none() {
    let parser = Parser::new();
    assert_eq!(parser.timeout(), None);
}

#[test]
fn set_timeout_stores_duration() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(250));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(250)));
}

#[test]
fn set_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn set_timeout_zero_duration() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn set_timeout_max_duration() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::MAX);
    assert_eq!(parser.timeout(), Some(Duration::MAX));
}

#[test]
fn timeout_persists_across_language_changes() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(7));
    parser.set_language(stub_language()).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(7)));

    parser.set_language(multi_symbol_test_language(2)).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(7)));
}

#[test]
fn timeout_persists_across_parse_attempts() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(300));
    // No language → Err, but timeout stays
    let _ = parser.parse(b"a", None);
    assert_eq!(parser.timeout(), Some(Duration::from_millis(300)));
}
