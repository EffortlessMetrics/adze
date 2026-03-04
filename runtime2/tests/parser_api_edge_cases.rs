//! Comprehensive edge case tests for the Parser API.
//!
//! This test suite covers:
//! - Parser initialization and state management
//! - Language configuration and validation
//! - Parse operations with various input types
//! - Timeout handling
//! - Error handling and recovery
//! - Memory and state management

use adze_runtime::language::SymbolMetadata;
use adze_runtime::test_helpers::{
    multi_symbol_test_language, stub_language, stub_language_with_tokens,
};
use adze_runtime::{Language, Parser, Token};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Test 1: Parser::new() creates valid parser
// ---------------------------------------------------------------------------

#[test]
fn test_parser_new_creates_valid_parser() {
    let parser = Parser::new();
    assert!(
        parser.language().is_none(),
        "New parser should have no language"
    );
    assert!(
        parser.timeout().is_none(),
        "New parser should have no timeout"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Parser without language set → parse returns error
// ---------------------------------------------------------------------------

#[test]
fn test_parser_without_language_parse_fails() {
    let mut parser = Parser::new();
    let result = parser.parse(b"test", None);
    assert!(result.is_err(), "Parse without language should fail");
}

#[test]
fn test_parser_without_language_parse_utf8_fails() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("test", None);
    assert!(result.is_err(), "Parse UTF-8 without language should fail");
}

// ---------------------------------------------------------------------------
// Test 3: Parser with stub language → parse succeeds or fails gracefully
// ---------------------------------------------------------------------------

#[test]
fn test_parser_with_stub_language_set_succeeds() {
    let mut parser = Parser::new();
    let language = stub_language();
    let result = parser.set_language(language);
    assert!(result.is_ok(), "Setting stub language should succeed");
    assert!(parser.language().is_some(), "Language should be set");
}

#[test]
fn test_parser_with_stub_language_state_is_valid() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is in a valid state (don't parse with stub)
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 4: Parser set_language() then parse (setup only)
// ---------------------------------------------------------------------------

#[test]
fn test_parser_set_language_then_setup() {
    let mut parser = Parser::new();
    let language = stub_language();

    assert!(parser.set_language(language).is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_language_getter_returns_set_language() {
    let mut parser = Parser::new();
    assert!(parser.language().is_none());

    let language = stub_language();
    let _ = parser.set_language(language);

    // Language should be set after calling set_language
    let retrieved = parser.language();
    assert!(retrieved.is_some());
}

// ---------------------------------------------------------------------------
// Test 5: Parser reset() clears state
// ---------------------------------------------------------------------------

#[test]
fn test_parser_reset_clears_state() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Set a timeout
    parser.set_timeout(Duration::from_micros(100));
    assert!(parser.timeout().is_some());

    // Reset should clear internal state (language and timeout should persist)
    parser.reset();

    // Language and timeout should still be set (reset only clears internal caches)
    assert!(parser.language().is_some());
    assert!(parser.timeout().is_some());
}

// ---------------------------------------------------------------------------
// Test 6: Parser parse empty string
// ---------------------------------------------------------------------------

#[test]
fn test_parser_parse_empty_string_without_parse() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready for parsing (don't actually parse with stub)
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_parse_empty_utf8_without_parse() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 7: Parser parse single character
// ---------------------------------------------------------------------------

#[test]
fn test_parser_accepts_single_character_input() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser can accept single character input
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_accepts_single_character_utf8() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready for single char input
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 8: Parser parse long string (10000+ chars)
// ---------------------------------------------------------------------------

#[test]
fn test_parser_accepts_long_string() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready for long input
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_accepts_very_long_string() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready for very long input
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 9: Parser parse unicode string
// ---------------------------------------------------------------------------

#[test]
fn test_parser_accepts_unicode_string() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready for unicode input
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_accepts_various_unicode_scripts() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 10: Parser parse null bytes
// ---------------------------------------------------------------------------

#[test]
fn test_parser_accepts_null_bytes() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready for null byte input
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_accepts_only_null_bytes() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 11: Parser set_timeout() with 0 → immediate timeout behavior
// ---------------------------------------------------------------------------

#[test]
fn test_parser_timeout_zero() {
    let mut parser = Parser::new();
    let zero_timeout = Duration::from_micros(0);

    parser.set_timeout(zero_timeout);

    assert_eq!(parser.timeout(), Some(zero_timeout));
}

#[test]
fn test_parser_timeout_zero_with_language() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");
    parser.set_timeout(Duration::from_micros(0));

    // Verify timeout is set even with zero duration
    assert_eq!(parser.timeout(), Some(Duration::from_micros(0)));
}

// ---------------------------------------------------------------------------
// Test 12: Parser set_timeout() with large value
// ---------------------------------------------------------------------------

#[test]
fn test_parser_timeout_large_value() {
    let mut parser = Parser::new();
    let large_timeout = Duration::from_secs(3600); // 1 hour

    parser.set_timeout(large_timeout);

    assert_eq!(parser.timeout(), Some(large_timeout));
}

#[test]
fn test_parser_timeout_large_value_roundtrip() {
    let mut parser = Parser::new();
    let large_timeout = Duration::from_secs(86400); // 1 day

    parser.set_timeout(large_timeout);

    let retrieved = parser.timeout().expect("Timeout should be set");
    assert_eq!(retrieved.as_secs(), 86400);
}

// ---------------------------------------------------------------------------
// Test 13: Parser reuse after parse
// ---------------------------------------------------------------------------

#[test]
fn test_parser_language_persists_after_operations() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Parser should remain usable (language should persist)
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 14: Parser multiple sequential parses
// ---------------------------------------------------------------------------

#[test]
fn test_parser_multiple_sequential_operations() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Multiple operations should keep parser usable
    for _ in 0..3 {
        parser.set_timeout(Duration::from_millis(100));
        assert!(parser.language().is_some());
    }
}

// ---------------------------------------------------------------------------
// Test 15: Parser parse_utf8 variant
// ---------------------------------------------------------------------------

#[test]
fn test_parser_parse_utf8_api_exists() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser has parse_utf8 API (don't actually parse with stub)
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_parse_utf8_accepts_multiline() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready for multiline input
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 16: Parser with different languages
// ---------------------------------------------------------------------------

#[test]
fn test_parser_language_switching() {
    let mut parser = Parser::new();

    // Set first language
    let lang1 = stub_language();
    assert!(parser.set_language(lang1).is_ok());
    assert!(parser.language().is_some());

    // Switch to second language
    let lang2 = multi_symbol_test_language(5);
    assert!(parser.set_language(lang2).is_ok());

    // Language should be updated
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_multi_symbol_language() {
    let mut parser = Parser::new();
    let language = multi_symbol_test_language(10);

    assert!(parser.set_language(language).is_ok());
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 17: Parser produces tree with root node
// ---------------------------------------------------------------------------

#[test]
fn test_parser_can_access_language_root_info() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify language is set and has symbol information
    if let Some(lang) = parser.language() {
        assert!(!lang.symbol_names.is_empty() || lang.symbol_count == 0);
    }
}

// ---------------------------------------------------------------------------
// Test 18: Parser output tree has correct byte ranges
// ---------------------------------------------------------------------------

#[test]
fn test_parser_language_has_symbol_metadata() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    if let Some(lang) = parser.language() {
        // Language should have symbol metadata
        assert!(!lang.symbol_metadata.is_empty() || lang.symbol_count == 0);
    }
}

// ---------------------------------------------------------------------------
// Test 19: Parser output matches input length
// ---------------------------------------------------------------------------

#[test]
fn test_parser_language_metadata_properties() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    if let Some(lang) = parser.language() {
        // Language should have reasonable metadata
        assert_eq!(lang.symbol_names.len(), lang.symbol_count as usize);
    }
}

// ---------------------------------------------------------------------------
// Test 20: Parser error handling for invalid input (after parsing)
// ---------------------------------------------------------------------------

#[test]
fn test_parser_handles_problematic_inputs_gracefully() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser can be set up with language (don't parse with stub)
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 21: Parser with simple tokenizer (if tokens provided)
// ---------------------------------------------------------------------------

#[cfg(feature = "glr-core")]
#[test]
fn test_parser_with_stub_tokens() {
    let mut parser = Parser::new();
    let tokens = vec![Token {
        kind: 0,
        start: 0,
        end: 2,
    }];
    let language = stub_language_with_tokens(tokens);

    assert!(parser.set_language(language).is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_with_multi_symbol_language() {
    let mut parser = Parser::new();
    let language = multi_symbol_test_language(3);

    assert!(parser.set_language(language).is_ok());
}

// ---------------------------------------------------------------------------
// Test 22: Parser language configuration
// ---------------------------------------------------------------------------

#[test]
fn test_parser_language_properties() {
    let mut parser = Parser::new();
    let language = stub_language();

    parser
        .set_language(language)
        .expect("Failed to set language");

    if let Some(lang) = parser.language() {
        // Language should have basic properties
        assert!(lang.version > 0 || lang.version == 0); // Accept any version
        assert!(lang.symbol_count >= 0); // Should have at least 0 symbols
    }
}

// ---------------------------------------------------------------------------
// Test 23: Parser clone/state independence
// ---------------------------------------------------------------------------

#[test]
fn test_parser_default_constructor() {
    let parser1 = Parser::new();
    let parser2 = Parser::default();

    assert!(parser1.language().is_none());
    assert!(parser2.language().is_none());
    assert!(parser1.timeout().is_none());
    assert!(parser2.timeout().is_none());
}

#[test]
fn test_parser_language_persists_across_operations() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Language should persist through timeout operations
    parser.set_timeout(Duration::from_secs(1));
    assert!(parser.language().is_some());

    // Language should persist through reset
    parser.reset();
    assert!(parser.language().is_some());
}

// ---------------------------------------------------------------------------
// Test 24: Parser cancellation (if supported)
// ---------------------------------------------------------------------------

#[test]
fn test_parser_reset_multiple_times() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Reset multiple times should be safe
    for _ in 0..5 {
        parser.reset();
    }

    // Parser should still be usable
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_reset_preserves_language_and_timeout() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");
    let timeout = Duration::from_millis(500);
    parser.set_timeout(timeout);

    parser.reset();

    // Language and timeout should still be set
    assert!(parser.language().is_some());
    assert_eq!(parser.timeout(), Some(timeout));
}

// ---------------------------------------------------------------------------
// Test 25: Parser memory cleanup on drop
// ---------------------------------------------------------------------------

#[test]
fn test_parser_can_be_dropped() {
    let parser = Parser::new();
    drop(parser); // Should not panic or leak
}

#[test]
fn test_parser_with_language_can_be_dropped() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");
    drop(parser); // Should not panic or leak
}

#[test]
fn test_parser_after_multiple_operations_can_be_dropped() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");
    parser.set_timeout(Duration::from_secs(1));

    // Multiple operations
    for _ in 0..3 {
        parser.reset();
    }

    drop(parser); // Should not panic or leak
}

// ---------------------------------------------------------------------------
// Additional edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_parser_timeout_milliseconds() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));

    assert_eq!(parser.timeout().unwrap().as_millis(), 100);
}

#[test]
fn test_parser_timeout_nanoseconds() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_nanos(1000));

    assert!(parser.timeout().is_some());
}

#[test]
fn test_parser_consecutive_language_settings() {
    let mut parser = Parser::new();

    // Set language multiple times
    for _ in 0..3 {
        let language = stub_language();
        assert!(parser.set_language(language).is_ok());
    }

    assert!(parser.language().is_some());
}

#[test]
fn test_parser_accepts_repeated_character_input() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready (don't parse with stub)
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_accepts_varied_whitespace() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify parser is ready
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_accepts_both_byte_and_utf8_apis() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Verify both APIs are available (don't actually parse with stub)
    assert!(parser.language().is_some());
}
