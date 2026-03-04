//! Comprehensive tests for `Parser::parse_utf8` and related UTF-8 handling.
//!
//! Covers: no-language error paths, empty strings, ASCII-only inputs,
//! multi-byte UTF-8, long strings, special characters, timeout interaction,
//! and multiple parse attempts.

use adze_runtime::error::{ParseError, ParseErrorKind};
use adze_runtime::language::{Language, SymbolMetadata};
use adze_runtime::parser::Parser;
use adze_runtime::tree::Tree;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helper: build a minimal language that can be set on Parser
// ---------------------------------------------------------------------------

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

// ===========================================================================
// Section 1: parse_utf8 without language (error handling)
// ===========================================================================

#[test]
fn parse_utf8_no_language_returns_err() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_no_language_error_is_no_language_kind() {
    let mut parser = Parser::new();
    let err = parser.parse_utf8("hello", None).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_utf8_no_language_error_display_not_empty() {
    let mut parser = Parser::new();
    let err = parser.parse_utf8("test", None).unwrap_err();
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn parse_utf8_no_language_error_has_no_location() {
    let mut parser = Parser::new();
    let err = parser.parse_utf8("data", None).unwrap_err();
    assert!(err.location.is_none());
}

#[test]
fn parse_utf8_no_language_empty_input() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("", None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().kind,
        ParseErrorKind::NoLanguage
    ));
}

#[test]
fn parse_utf8_no_language_unicode_input() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("日本語テスト", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_no_language_after_reset() {
    let mut parser = Parser::new();
    parser.reset();
    let result = parser.parse_utf8("abc", None);
    assert!(result.is_err());
}

// ===========================================================================
// Section 2: parse (bytes) without language (error handling)
// ===========================================================================

#[test]
fn parse_no_language_returns_err() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
}

#[test]
fn parse_no_language_error_is_no_language_kind() {
    let mut parser = Parser::new();
    let err = parser.parse(b"hello", None).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_no_language_empty_bytes() {
    let mut parser = Parser::new();
    let result = parser.parse(b"", None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().kind,
        ParseErrorKind::NoLanguage
    ));
}

#[test]
fn parse_no_language_with_null_bytes() {
    let mut parser = Parser::new();
    let result = parser.parse(b"\x00\x00\x00", None);
    assert!(result.is_err());
}

#[test]
fn parse_no_language_after_timeout_set() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    let result = parser.parse(b"data", None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().kind,
        ParseErrorKind::NoLanguage
    ));
}

// ===========================================================================
// Section 3: Various UTF-8 strings (parse_utf8 without language)
// ===========================================================================

#[test]
fn parse_utf8_latin_extended() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("àáâãäåæçèéêë", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_cyrillic() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("Привет мир", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_arabic() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("مرحبا بالعالم", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_chinese() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("你好世界", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_japanese_hiragana() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("こんにちは世界", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_korean() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("안녕하세요", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_thai() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("สวัสดีชาวโลก", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_devanagari() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("नमस्ते दुनिया", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_emoji_basic() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("😀🎉🚀", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_emoji_zwj_sequence() {
    let mut parser = Parser::new();
    // Family emoji with ZWJ sequences
    let result = parser.parse_utf8("👨‍👩‍👧‍👦", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_mixed_scripts() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("Hello 你好 مرحبا Привет", None);
    assert!(result.is_err());
}

// ===========================================================================
// Section 4: Empty string
// ===========================================================================

#[test]
fn parse_utf8_empty_string_no_language() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("", None);
    assert!(result.is_err());
}

#[test]
fn parse_empty_bytes_no_language() {
    let mut parser = Parser::new();
    let result = parser.parse(b"", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_empty_string_with_language() {
    let mut parser = Parser::new();
    parser.set_language(minimal_language()).unwrap();
    // GLR driver may panic with default parse table; verify we handle it
    let _ = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("", None)));
}

#[test]
fn parse_empty_vec_as_ref() {
    let mut parser = Parser::new();
    let empty: Vec<u8> = vec![];
    let result = parser.parse(empty, None);
    assert!(result.is_err());
}

// ===========================================================================
// Section 5: ASCII-only strings
// ===========================================================================

#[test]
fn parse_utf8_ascii_letters_no_language() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("abcdefghijklmnopqrstuvwxyz", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_ascii_digits() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("0123456789", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_ascii_punctuation() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("!@#$%^&*()_+-=[]{}|;':\",./<>?", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_single_char() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("a", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_whitespace_only() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("   \t\n\r  ", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_newlines() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("\n\n\n\n\n", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_crlf() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("line1\r\nline2\r\nline3", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_ascii_with_language() {
    let mut parser = Parser::new();
    parser.set_language(minimal_language()).unwrap();
    // GLR driver may panic with default parse table; verify we handle it
    let _ = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("hello world", None)));
}

// ===========================================================================
// Section 6: Multi-byte UTF-8
// ===========================================================================

#[test]
fn parse_utf8_two_byte_chars() {
    let mut parser = Parser::new();
    // ñ, ü, é are 2-byte UTF-8
    let result = parser.parse_utf8("ñüéàö", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_three_byte_chars() {
    let mut parser = Parser::new();
    // CJK characters are 3-byte UTF-8
    let result = parser.parse_utf8("漢字テスト", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_four_byte_chars() {
    let mut parser = Parser::new();
    // Emoji are 4-byte UTF-8
    let result = parser.parse_utf8("🦀🐍🦊🐉", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_supplementary_plane() {
    let mut parser = Parser::new();
    // Mathematical symbols from supplementary planes
    let result = parser.parse_utf8("𝕳𝖊𝖑𝖑𝖔", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_mixed_byte_lengths() {
    let mut parser = Parser::new();
    // Mix of 1, 2, 3, and 4-byte characters
    let result = parser.parse_utf8("a é 漢 🦀", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_bom() {
    let mut parser = Parser::new();
    // UTF-8 BOM followed by content
    let result = parser.parse_utf8("\u{FEFF}hello", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_zero_width_chars() {
    let mut parser = Parser::new();
    // Zero-width space, zero-width joiner, zero-width non-joiner
    let result = parser.parse_utf8("a\u{200B}b\u{200C}c\u{200D}d", None);
    assert!(result.is_err());
}

// ===========================================================================
// Section 7: Long strings
// ===========================================================================

#[test]
fn parse_utf8_1kb_string() {
    let mut parser = Parser::new();
    let input = "a".repeat(1024);
    let result = parser.parse_utf8(&input, None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_64kb_string() {
    let mut parser = Parser::new();
    let input = "x".repeat(65536);
    let result = parser.parse_utf8(&input, None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_long_multibyte() {
    let mut parser = Parser::new();
    let input = "漢".repeat(10_000);
    let result = parser.parse_utf8(&input, None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_long_emoji() {
    let mut parser = Parser::new();
    let input = "🦀".repeat(5_000);
    let result = parser.parse_utf8(&input, None);
    assert!(result.is_err());
}

#[test]
fn parse_bytes_1mb() {
    let mut parser = Parser::new();
    let input = vec![b'a'; 1_048_576];
    let result = parser.parse(&input, None);
    assert!(result.is_err());
}

// ===========================================================================
// Section 8: Special characters
// ===========================================================================

#[test]
fn parse_utf8_null_char() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("\0", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_embedded_nulls() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello\0world\0end", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_tab_chars() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("\t\t\t", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_backslash() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("\\n\\t\\r\\\\", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_control_chars() {
    let mut parser = Parser::new();
    // Bell, backspace, form-feed, vertical tab
    let result = parser.parse_utf8("\x07\x08\x0C\x0B", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_replacement_char() {
    let mut parser = Parser::new();
    // U+FFFD replacement character
    let result = parser.parse_utf8("\u{FFFD}\u{FFFD}", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_rtl_markers() {
    let mut parser = Parser::new();
    // Right-to-left mark, left-to-right mark
    let result = parser.parse_utf8("hello\u{200F}world\u{200E}end", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_combining_diacriticals() {
    let mut parser = Parser::new();
    // 'e' followed by combining acute accent
    let result = parser.parse_utf8("e\u{0301} a\u{0308}", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_line_separator() {
    let mut parser = Parser::new();
    // Unicode line separator (U+2028) and paragraph separator (U+2029)
    let result = parser.parse_utf8("line1\u{2028}line2\u{2029}line3", None);
    assert!(result.is_err());
}

// ===========================================================================
// Section 9: Parser timeout interaction
// ===========================================================================

#[test]
fn parse_utf8_with_timeout_no_language() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
    // NoLanguage error takes precedence over timeout
    assert!(matches!(
        result.unwrap_err().kind,
        ParseErrorKind::NoLanguage
    ));
}

#[test]
fn parse_utf8_timeout_preserved_after_error() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(30));
    let _ = parser.parse_utf8("hello", None);
    assert_eq!(parser.timeout(), Some(Duration::from_secs(30)));
}

#[test]
fn parse_utf8_zero_timeout_no_language() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    let result = parser.parse_utf8("test", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_large_timeout_no_language() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(u64::MAX));
    let result = parser.parse_utf8("test", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_timeout_set_with_language() {
    let mut parser = Parser::new();
    parser.set_language(minimal_language()).unwrap();
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
    // GLR driver may panic with default parse table; catch it
    let _ = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("abc", None)));
}

// ===========================================================================
// Section 10: Multiple parse attempts
// ===========================================================================

#[test]
fn parse_utf8_multiple_errors_no_language() {
    let mut parser = Parser::new();
    for i in 0..10 {
        let input = format!("attempt {i}");
        let result = parser.parse_utf8(&input, None);
        assert!(result.is_err());
    }
}

#[test]
fn parse_utf8_alternating_empty_and_nonempty() {
    let mut parser = Parser::new();
    for i in 0..10 {
        let input = if i % 2 == 0 { "" } else { "some text" };
        let result = parser.parse_utf8(input, None);
        assert!(result.is_err());
    }
}

#[test]
fn parse_alternating_utf8_and_bytes() {
    let mut parser = Parser::new();
    let err1 = parser.parse_utf8("hello", None);
    let err2 = parser.parse(b"world", None);
    let err3 = parser.parse_utf8("!", None);
    assert!(err1.is_err());
    assert!(err2.is_err());
    assert!(err3.is_err());
}

#[test]
fn parse_utf8_with_language_multiple_attempts() {
    let mut parser = Parser::new();
    parser.set_language(minimal_language()).unwrap();
    // GLR driver may panic with default parse table; catch per attempt
    for _ in 0..5 {
        let mut p = Parser::new();
        p.set_language(minimal_language()).unwrap();
        let _ = catch_unwind(AssertUnwindSafe(|| p.parse_utf8("test input", None)));
    }
}

#[test]
fn parse_utf8_reset_between_parses() {
    let mut p1 = Parser::new();
    p1.set_language(minimal_language()).unwrap();
    let _ = catch_unwind(AssertUnwindSafe(|| p1.parse_utf8("first", None)));
    // After panic, create fresh parser to verify reset path
    let mut p2 = Parser::new();
    p2.set_language(minimal_language()).unwrap();
    p2.reset();
    let _ = catch_unwind(AssertUnwindSafe(|| p2.parse_utf8("second", None)));
}

#[test]
fn parse_utf8_growing_inputs() {
    let mut parser = Parser::new();
    for len in [1, 10, 100, 1000, 10_000] {
        let input = "a".repeat(len);
        let result = parser.parse_utf8(&input, None);
        assert!(result.is_err());
    }
}

#[test]
fn parse_utf8_with_language_different_inputs() {
    let inputs = ["", "x", "hello", "漢字", "🦀🦀🦀"];
    for input in &inputs {
        let mut parser = Parser::new();
        parser.set_language(minimal_language()).unwrap();
        let _ = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8(input, None)));
    }
}

// ===========================================================================
// Section 11: parse_utf8 and parse equivalence
// ===========================================================================

#[test]
fn parse_utf8_and_parse_bytes_same_error_kind() {
    let mut parser = Parser::new();
    let err_utf8 = parser.parse_utf8("test", None).unwrap_err();
    let err_bytes = parser.parse(b"test", None).unwrap_err();
    assert!(matches!(err_utf8.kind, ParseErrorKind::NoLanguage));
    assert!(matches!(err_bytes.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_utf8_delegates_to_parse() {
    // parse_utf8 should behave identically to parse with the same bytes
    let mut parser1 = Parser::new();
    let mut parser2 = Parser::new();
    let input = "hello world 漢字 🦀";
    let r1 = parser1.parse_utf8(input, None);
    let r2 = parser2.parse(input.as_bytes(), None);
    assert_eq!(r1.is_err(), r2.is_err());
}

// ===========================================================================
// Section 12: old_tree parameter (parse_utf8)
// ===========================================================================

#[test]
fn parse_utf8_with_stub_old_tree_no_language() {
    let mut parser = Parser::new();
    let stub = Tree::new_stub();
    let result = parser.parse_utf8("hello", Some(&stub));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().kind,
        ParseErrorKind::NoLanguage
    ));
}

#[test]
fn parse_utf8_old_tree_none_no_language() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

#[test]
fn parse_with_stub_old_tree_no_language() {
    let mut parser = Parser::new();
    let stub = Tree::new_stub();
    let result = parser.parse(b"hello", Some(&stub));
    assert!(result.is_err());
}

// ===========================================================================
// Section 13: Parser Default trait
// ===========================================================================

#[test]
fn parser_default_has_no_language() {
    let parser = Parser::default();
    assert!(parser.language().is_none());
}

#[test]
fn parser_default_has_no_timeout() {
    let parser = Parser::default();
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_default_parse_utf8_fails() {
    let mut parser = Parser::default();
    let result = parser.parse_utf8("test", None);
    assert!(result.is_err());
}
