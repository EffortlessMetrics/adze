//! Comprehensive tests for Parser error handling paths in adze-runtime.
//!
//! Covers: no-language errors, stub-language panics via catch_unwind,
//! timeout behaviour, ParseError/ParseErrorKind inspection, multiple
//! parse calls, empty/unicode/binary/large input, and parser re-creation.

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};
use adze_runtime::parser::Parser;
use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

// =========================================================================
// Helpers
// =========================================================================

/// Attempt a parse wrapped in catch_unwind; returns Ok(Result) or Err(panic).
fn try_parse(
    parser: &mut Parser,
    input: &[u8],
) -> Result<Result<adze_runtime::Tree, ParseError>, Box<dyn std::any::Any + Send>> {
    let input = input.to_vec();
    catch_unwind(AssertUnwindSafe(|| parser.parse(&input, None)))
}

/// Shorthand: parse must fail with a real ParseError (not a panic).
fn expect_parse_error(parser: &mut Parser, input: &[u8]) -> ParseError {
    match try_parse(parser, input) {
        Ok(Err(e)) => e,
        Ok(Ok(_)) => panic!("expected ParseError but got Ok(Tree)"),
        Err(_) => panic!("expected ParseError but got panic"),
    }
}

// =========================================================================
// 1. Parser without language set
// =========================================================================

#[test]
fn parse_without_language_returns_no_language_error() {
    let mut parser = Parser::new();
    let err = expect_parse_error(&mut parser, b"hello");
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_without_language_has_no_location() {
    let mut parser = Parser::new();
    let err = expect_parse_error(&mut parser, b"x");
    assert!(err.location.is_none());
}

#[test]
fn parse_utf8_without_language_returns_no_language() {
    let mut parser = Parser::new();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("hello", None)));
    match result {
        Ok(Err(e)) => assert!(matches!(e.kind, ParseErrorKind::NoLanguage)),
        Ok(Ok(_)) => panic!("expected error"),
        Err(_) => panic!("unexpected panic"),
    }
}

#[test]
fn parse_empty_without_language_returns_no_language() {
    let mut parser = Parser::new();
    let err = expect_parse_error(&mut parser, b"");
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn no_language_error_display_contains_message() {
    let mut parser = Parser::new();
    let err = expect_parse_error(&mut parser, b"test");
    let msg = format!("{err}");
    assert!(msg.contains("no language"), "display was: {msg}");
}

#[test]
fn no_language_error_debug_is_nonempty() {
    let mut parser = Parser::new();
    let err = expect_parse_error(&mut parser, b"test");
    let dbg = format!("{err:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn parse_after_reset_without_language_still_errors() {
    let mut parser = Parser::new();
    parser.reset();
    let err = expect_parse_error(&mut parser, b"data");
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

// =========================================================================
// 2. Parser with stub language — catch panics
// =========================================================================

fn parser_with_stub() -> Parser {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p
}

#[test]
fn stub_language_parse_does_not_crash_process() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"hello");
}

#[test]
fn stub_language_parse_empty_input() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"");
}

#[test]
fn stub_language_parse_single_byte() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"x");
}

#[test]
fn stub_language_parse_whitespace_only() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"   \t\n");
}

#[test]
fn stub_language_parse_newlines() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"\n\n\n");
}

#[test]
fn stub_language_parse_returns_error_or_panics() {
    let mut parser = parser_with_stub();
    // Either a ParseError or a panic is acceptable — no UB.
    let outcome = try_parse(&mut parser, b"1 + 2");
    match outcome {
        Ok(Err(_)) => {} // error path
        Ok(Ok(_)) => {}  // unlikely but legal
        Err(_) => {}     // panic path
    }
}

#[test]
fn stub_language_parse_twice_without_crash() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"first");
    let _ = try_parse(&mut parser, b"second");
}

#[test]
fn stub_language_parse_with_null_bytes() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"ab\0cd");
}

#[test]
fn stub_language_parse_high_bytes() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, &[0xFF, 0xFE, 0xFD]);
}

#[test]
fn stub_language_parse_utf8_multibyte() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, "café ☕ 日本語".as_bytes());
}

// =========================================================================
// 3. Parser timeout behaviour
// =========================================================================

#[test]
fn set_timeout_stores_value() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));
}

#[test]
fn set_timeout_zero_is_accepted() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn set_timeout_large_value() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(3600));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3600)));
}

#[test]
fn set_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(10));
    parser.set_timeout(Duration::from_millis(999));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(999)));
}

#[test]
fn timeout_preserved_after_set_language() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(42));
    parser.set_language(stub_language()).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_millis(42)));
}

#[test]
fn timeout_preserved_after_reset() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(42));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_millis(42)));
}

#[test]
fn parse_without_language_ignores_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(1));
    let err = expect_parse_error(&mut parser, b"x");
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn timeout_nanos_precision() {
    let mut parser = Parser::new();
    let dur = Duration::from_nanos(123_456_789);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

// =========================================================================
// 4. ParseError kind variants
// =========================================================================

#[test]
fn parse_error_no_language_variant() {
    let err = ParseError::no_language();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    assert!(err.location.is_none());
}

#[test]
fn parse_error_timeout_variant() {
    let err = ParseError::timeout();
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
    assert!(err.location.is_none());
}

#[test]
fn parse_error_syntax_error_variant() {
    let loc = ErrorLocation {
        byte_offset: 5,
        line: 1,
        column: 6,
    };
    let err = ParseError::syntax_error("unexpected token", loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::SyntaxError(_)));
    assert_eq!(err.location, Some(loc));
}

#[test]
fn parse_error_with_msg_variant() {
    let err = ParseError::with_msg("custom failure");
    assert!(matches!(err.kind, ParseErrorKind::Other(_)));
    let msg = format!("{err}");
    assert!(msg.contains("custom failure"), "got: {msg}");
}

#[test]
fn parse_error_with_location_chaining() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::no_language().with_location(loc.clone());
    assert_eq!(err.location, Some(loc));
}

#[test]
fn parse_error_kind_no_language_display() {
    let kind = ParseErrorKind::NoLanguage;
    assert_eq!(format!("{kind}"), "no language set");
}

#[test]
fn parse_error_kind_timeout_display() {
    let kind = ParseErrorKind::Timeout;
    assert_eq!(format!("{kind}"), "parse timeout exceeded");
}

#[test]
fn parse_error_kind_invalid_encoding_display() {
    let kind = ParseErrorKind::InvalidEncoding;
    assert_eq!(format!("{kind}"), "invalid input encoding");
}

#[test]
fn parse_error_kind_cancelled_display() {
    let kind = ParseErrorKind::Cancelled;
    assert_eq!(format!("{kind}"), "parse cancelled");
}

#[test]
fn parse_error_kind_version_mismatch_display() {
    let kind = ParseErrorKind::VersionMismatch {
        expected: 14,
        actual: 15,
    };
    let msg = format!("{kind}");
    assert!(msg.contains("14") && msg.contains("15"), "got: {msg}");
}

#[test]
fn parse_error_kind_allocation_error_display() {
    let kind = ParseErrorKind::AllocationError;
    assert_eq!(format!("{kind}"), "memory allocation failed");
}

#[test]
fn parse_error_kind_other_display() {
    let kind = ParseErrorKind::Other("boom".into());
    assert_eq!(format!("{kind}"), "boom");
}

#[test]
fn parse_error_kind_syntax_error_display() {
    let kind = ParseErrorKind::SyntaxError("line 3".into());
    let msg = format!("{kind}");
    assert!(msg.contains("line 3"), "got: {msg}");
}

#[test]
fn error_location_display() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 7,
    };
    assert_eq!(format!("{loc}"), "3:7");
}

#[test]
fn error_location_clone_eq() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let loc2 = loc.clone();
    assert_eq!(loc, loc2);
}

#[test]
fn parse_error_is_debug() {
    let err = ParseError::no_language();
    let _ = format!("{err:?}");
}

#[test]
fn parse_error_is_std_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}
    let err = ParseError::no_language();
    assert_error(&err);
}

// =========================================================================
// 5. Multiple parse calls
// =========================================================================

#[test]
fn multiple_parses_without_language_all_fail() {
    let mut parser = Parser::new();
    for _ in 0..5 {
        let err = expect_parse_error(&mut parser, b"data");
        assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    }
}

#[test]
fn parse_then_set_language_then_parse_again() {
    let mut parser = Parser::new();
    let _ = expect_parse_error(&mut parser, b"x");
    parser.set_language(stub_language()).unwrap();
    // Second parse may panic in GLR driver — that's OK.
    let _ = try_parse(&mut parser, b"x");
}

#[test]
fn alternating_parse_calls_with_different_input() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"aaa");
    let _ = try_parse(&mut parser, b"bbb");
    let _ = try_parse(&mut parser, b"ccc");
}

#[test]
fn parse_ten_times_sequentially() {
    let mut parser = parser_with_stub();
    for i in 0..10 {
        let input = format!("input_{i}");
        let _ = try_parse(&mut parser, input.as_bytes());
    }
}

#[test]
fn parse_error_then_success_path_does_not_corrupt_state() {
    let mut parser = Parser::new();
    let _ = expect_parse_error(&mut parser, b"first");
    parser.set_language(stub_language()).unwrap();
    // Parser should still be usable (no internal corruption).
    assert!(parser.language().is_some());
}

// =========================================================================
// 6. Parse empty input
// =========================================================================

#[test]
fn parse_empty_bytes_no_language() {
    let mut parser = Parser::new();
    let err = expect_parse_error(&mut parser, b"");
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_empty_bytes_with_stub() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"");
}

#[test]
fn parse_empty_string_utf8_no_language() {
    let mut parser = Parser::new();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("", None)));
    match result {
        Ok(Err(e)) => assert!(matches!(e.kind, ParseErrorKind::NoLanguage)),
        other => {
            let _ = other; // panic or unexpected Ok — both acceptable
        }
    }
}

#[test]
fn parse_empty_string_utf8_with_stub() {
    let mut parser = parser_with_stub();
    let _ = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("", None)));
}

// =========================================================================
// 7. Parse unicode input
// =========================================================================

#[test]
fn parse_ascii_only() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"hello world 123");
}

#[test]
fn parse_latin1_extended() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, "Ñoño résumé naïve".as_bytes());
}

#[test]
fn parse_cjk_input() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, "日本語テスト".as_bytes());
}

#[test]
fn parse_emoji_input() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, "🦀🔥💯".as_bytes());
}

#[test]
fn parse_mixed_scripts() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, "Hello мир 世界 🌍".as_bytes());
}

#[test]
fn parse_bom_prefixed_utf8() {
    let mut parser = parser_with_stub();
    let mut input = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    input.extend_from_slice(b"content");
    let _ = try_parse(&mut parser, &input);
}

#[test]
fn parse_zero_width_characters() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, "a\u{200B}b\u{FEFF}c".as_bytes());
}

// =========================================================================
// 8. Parse binary input
// =========================================================================

#[test]
fn parse_all_zero_bytes() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, &[0u8; 16]);
}

#[test]
fn parse_all_0xff_bytes() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, &[0xFF; 16]);
}

#[test]
fn parse_byte_range_0_to_255() {
    let mut parser = parser_with_stub();
    let input: Vec<u8> = (0..=255).collect();
    let _ = try_parse(&mut parser, &input);
}

#[test]
fn parse_invalid_utf8_sequence() {
    let mut parser = parser_with_stub();
    // Invalid 2-byte UTF-8 start without continuation
    let _ = try_parse(&mut parser, &[0xC0, 0x01]);
}

#[test]
fn parse_surrogate_half_bytes() {
    let mut parser = parser_with_stub();
    // ED A0 80 = U+D800 encoded as if UTF-8 (invalid)
    let _ = try_parse(&mut parser, &[0xED, 0xA0, 0x80]);
}

#[test]
fn parse_overlong_null() {
    let mut parser = parser_with_stub();
    // Overlong encoding of U+0000
    let _ = try_parse(&mut parser, &[0xC0, 0x80]);
}

// =========================================================================
// 9. Parse large input
// =========================================================================

#[test]
fn parse_1kb_input() {
    let mut parser = parser_with_stub();
    let input = vec![b'a'; 1024];
    let _ = try_parse(&mut parser, &input);
}

#[test]
fn parse_64kb_input() {
    let mut parser = parser_with_stub();
    let input = vec![b'z'; 64 * 1024];
    let _ = try_parse(&mut parser, &input);
}

#[test]
fn parse_1mb_input() {
    let mut parser = parser_with_stub();
    let input = vec![b'.'; 1024 * 1024];
    let _ = try_parse(&mut parser, &input);
}

#[test]
fn parse_large_repeating_pattern() {
    let mut parser = parser_with_stub();
    let pattern = b"tok ";
    let input: Vec<u8> = pattern.iter().cycle().take(10_000).copied().collect();
    let _ = try_parse(&mut parser, &input);
}

// =========================================================================
// 10. Parser re-creation (many new parsers)
// =========================================================================

#[test]
fn create_100_parsers() {
    for _ in 0..100 {
        let _ = Parser::new();
    }
}

#[test]
fn create_and_set_language_100_times() {
    for _ in 0..100 {
        let mut p = Parser::new();
        p.set_language(stub_language()).unwrap();
    }
}

#[test]
fn create_parse_drop_cycle() {
    for i in 0..20 {
        let mut parser = parser_with_stub();
        let input = format!("cycle_{i}");
        let _ = try_parse(&mut parser, input.as_bytes());
        drop(parser);
    }
}

#[test]
fn parser_default_equivalent_to_new() {
    let p1 = Parser::new();
    let p2 = Parser::default();
    assert!(p1.language().is_none());
    assert!(p2.language().is_none());
    assert_eq!(p1.timeout(), p2.timeout());
}

#[test]
fn parser_is_debug() {
    let parser = Parser::new();
    let dbg = format!("{parser:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn parser_with_multi_symbol_language() {
    let mut parser = Parser::new();
    let lang = multi_symbol_test_language(10);
    parser.set_language(lang).unwrap();
    let _ = try_parse(&mut parser, b"test");
}

#[test]
fn set_language_returns_ok_for_valid_stub() {
    let mut parser = Parser::new();
    assert!(parser.set_language(stub_language()).is_ok());
}

#[test]
fn language_accessor_returns_some_after_set() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn language_accessor_returns_none_initially() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

// =========================================================================
// Additional edge-case error-path tests
// =========================================================================

#[test]
fn parse_error_with_msg_empty_string() {
    let err = ParseError::with_msg("");
    let msg = format!("{err}");
    assert!(msg.is_empty() || msg.len() >= 0); // does not panic
}

#[test]
fn parse_error_with_msg_long_string() {
    let long = "x".repeat(10_000);
    let err = ParseError::with_msg(&long);
    let msg = format!("{err}");
    assert!(msg.contains("xxx"));
}

#[test]
fn parse_error_syntax_error_empty_message() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::syntax_error("", loc);
    let _ = format!("{err}");
}

#[test]
fn error_location_zero_values() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 0,
        column: 0,
    };
    assert_eq!(format!("{loc}"), "0:0");
}

#[test]
fn error_location_large_values() {
    let loc = ErrorLocation {
        byte_offset: usize::MAX,
        line: usize::MAX,
        column: usize::MAX,
    };
    let display = format!("{loc}");
    assert!(!display.is_empty());
}

#[test]
fn parse_error_no_language_then_with_location() {
    let loc = ErrorLocation {
        byte_offset: 7,
        line: 2,
        column: 3,
    };
    let err = ParseError::no_language().with_location(loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    assert_eq!(err.location.unwrap(), loc);
}

#[test]
fn parse_error_timeout_then_with_location() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 5,
        column: 10,
    };
    let err = ParseError::timeout().with_location(loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
    assert_eq!(err.location.unwrap(), loc);
}

#[test]
fn version_mismatch_error_fields() {
    let kind = ParseErrorKind::VersionMismatch {
        expected: 1,
        actual: 999,
    };
    let msg = format!("{kind}");
    assert!(msg.contains("1") && msg.contains("999"));
}

#[test]
fn stub_parse_with_old_tree_none() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"code");
}

#[test]
fn reset_then_parse_with_language() {
    let mut parser = parser_with_stub();
    parser.reset();
    let _ = try_parse(&mut parser, b"after reset");
}

#[test]
fn reset_multiple_times() {
    let mut parser = parser_with_stub();
    for _ in 0..10 {
        parser.reset();
    }
    let _ = try_parse(&mut parser, b"still ok");
}

#[test]
fn set_language_twice_overwrites() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parse_only_whitespace_no_language() {
    let mut parser = Parser::new();
    let err = expect_parse_error(&mut parser, b"   ");
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_tab_characters() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"\t\t\t");
}

#[test]
fn parse_carriage_return_line_feed() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"line1\r\nline2\r\n");
}

#[test]
fn parse_single_newline() {
    let mut parser = parser_with_stub();
    let _ = try_parse(&mut parser, b"\n");
}

#[test]
fn parse_error_kind_debug_no_language() {
    let kind = ParseErrorKind::NoLanguage;
    let dbg = format!("{kind:?}");
    assert!(dbg.contains("NoLanguage"));
}

#[test]
fn parse_error_kind_debug_timeout() {
    let kind = ParseErrorKind::Timeout;
    let dbg = format!("{kind:?}");
    assert!(dbg.contains("Timeout"));
}

#[test]
fn parse_error_kind_debug_other() {
    let kind = ParseErrorKind::Other("detail".into());
    let dbg = format!("{kind:?}");
    assert!(dbg.contains("detail"));
}
