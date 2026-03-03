#![allow(clippy::needless_range_loop)]

//! Comprehensive error scenario tests for adze-runtime (runtime2).
//!
//! Tests various error construction, formatting, comparison, and recovery patterns.

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};
use adze_runtime::Parser;

// ---------------------------------------------------------------------------
// 1. Error kind construction
// ---------------------------------------------------------------------------

#[test]
fn error_kind_no_language_display() {
    let err = ParseError::no_language();
    let msg = format!("{}", err);
    assert_eq!(msg, "no language set");
    assert!(err.location.is_none());
}

#[test]
fn error_kind_timeout_display() {
    let err = ParseError::timeout();
    let msg = format!("{}", err);
    assert_eq!(msg, "parse timeout exceeded");
    assert!(err.location.is_none());
}

#[test]
fn error_kind_invalid_encoding() {
    let err = ParseError {
        kind: ParseErrorKind::InvalidEncoding,
        location: None,
    };
    let msg = format!("{}", err);
    assert_eq!(msg, "invalid input encoding");
}

#[test]
fn error_kind_cancelled() {
    let err = ParseError {
        kind: ParseErrorKind::Cancelled,
        location: None,
    };
    let msg = format!("{}", err);
    assert_eq!(msg, "parse cancelled");
}

#[test]
fn error_kind_version_mismatch_display() {
    let err = ParseError {
        kind: ParseErrorKind::VersionMismatch {
            expected: 15,
            actual: 14,
        },
        location: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("15"));
    assert!(msg.contains("14"));
    assert!(msg.contains("version mismatch"));
}

#[test]
fn error_kind_syntax_error_display() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let err = ParseError::syntax_error("unexpected '}'", loc);
    let msg = format!("{}", err);
    assert!(msg.contains("unexpected '}'"));
}

#[test]
fn error_kind_allocation_error() {
    let err = ParseError {
        kind: ParseErrorKind::AllocationError,
        location: None,
    };
    let msg = format!("{}", err);
    assert_eq!(msg, "memory allocation failed");
}

#[test]
fn error_kind_other_custom_message() {
    let err = ParseError::with_msg("custom failure reason");
    let msg = format!("{}", err);
    assert_eq!(msg, "custom failure reason");
}

// ---------------------------------------------------------------------------
// 2. Error location tracking
// ---------------------------------------------------------------------------

#[test]
fn error_location_display_format() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 7,
    };
    let display = format!("{}", loc);
    assert_eq!(display, "3:7");
}

#[test]
fn error_location_equality() {
    let a = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let b = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(a, b);
}

#[test]
fn error_location_inequality_byte_offset() {
    let a = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let b = ErrorLocation {
        byte_offset: 1,
        line: 1,
        column: 1,
    };
    assert_ne!(a, b);
}

#[test]
fn error_location_inequality_line() {
    let a = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let b = ErrorLocation {
        byte_offset: 0,
        line: 2,
        column: 1,
    };
    assert_ne!(a, b);
}

#[test]
fn error_location_inequality_column() {
    let a = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let b = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 2,
    };
    assert_ne!(a, b);
}

#[test]
fn error_location_clone() {
    let loc = ErrorLocation {
        byte_offset: 99,
        line: 10,
        column: 20,
    };
    let cloned = loc.clone();
    assert_eq!(loc, cloned);
}

#[test]
fn error_location_at_origin() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(format!("{}", loc), "1:1");
}

#[test]
fn error_location_large_values() {
    let loc = ErrorLocation {
        byte_offset: usize::MAX,
        line: usize::MAX,
        column: usize::MAX,
    };
    let display = format!("{}", loc);
    assert!(display.contains(&usize::MAX.to_string()));
}

// ---------------------------------------------------------------------------
// 3. with_location chaining
// ---------------------------------------------------------------------------

#[test]
fn with_location_attaches_to_no_language() {
    let loc = ErrorLocation {
        byte_offset: 5,
        line: 1,
        column: 6,
    };
    let err = ParseError::no_language().with_location(loc.clone());
    assert_eq!(err.location.as_ref(), Some(&loc));
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn with_location_overwrites_existing() {
    let loc1 = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let loc2 = ErrorLocation {
        byte_offset: 50,
        line: 5,
        column: 10,
    };
    let err = ParseError::syntax_error("err", loc1).with_location(loc2.clone());
    assert_eq!(err.location.as_ref(), Some(&loc2));
}

// ---------------------------------------------------------------------------
// 4. Syntax error with location details
// ---------------------------------------------------------------------------

#[test]
fn syntax_error_stores_location() {
    let loc = ErrorLocation {
        byte_offset: 15,
        line: 3,
        column: 4,
    };
    let err = ParseError::syntax_error("missing semicolon", loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::SyntaxError(ref s) if s == "missing semicolon"));
    assert_eq!(err.location, Some(loc));
}

#[test]
fn syntax_error_empty_message() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::syntax_error("", loc);
    let msg = format!("{}", err);
    assert!(msg.contains("syntax error"));
}

// ---------------------------------------------------------------------------
// 5. Unicode content in error messages
// ---------------------------------------------------------------------------

#[test]
fn unicode_in_syntax_error_message() {
    let loc = ErrorLocation {
        byte_offset: 6,
        line: 1,
        column: 3,
    };
    let err = ParseError::syntax_error("unexpected token '日本語'", loc);
    let msg = format!("{}", err);
    assert!(msg.contains("日本語"));
}

#[test]
fn unicode_in_other_error_message() {
    let err = ParseError::with_msg("ошибка парсера: неожиданный символ");
    let msg = format!("{}", err);
    assert!(msg.contains("ошибка"));
}

#[test]
fn emoji_in_error_message() {
    let err = ParseError::with_msg("parse failed 🚫 at token ➡️");
    let msg = format!("{}", err);
    assert!(msg.contains("🚫"));
    assert!(msg.contains("➡️"));
}

// ---------------------------------------------------------------------------
// 6. Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn error_debug_includes_kind() {
    let err = ParseError::timeout();
    let debug = format!("{:?}", err);
    assert!(debug.contains("Timeout"));
}

#[test]
fn error_debug_includes_location_when_present() {
    let loc = ErrorLocation {
        byte_offset: 7,
        line: 2,
        column: 3,
    };
    let err = ParseError::no_language().with_location(loc);
    let debug = format!("{:?}", err);
    assert!(debug.contains("ErrorLocation"));
    assert!(debug.contains("byte_offset: 7"));
}

#[test]
fn error_location_debug() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 10,
        column: 5,
    };
    let debug = format!("{:?}", loc);
    assert!(debug.contains("42"));
    assert!(debug.contains("10"));
    assert!(debug.contains("5"));
}

// ---------------------------------------------------------------------------
// 7. Multiple error construction (simulated cascading)
// ---------------------------------------------------------------------------

#[test]
fn collect_multiple_errors() {
    let errors: Vec<ParseError> = vec![
        ParseError::syntax_error(
            "unexpected token",
            ErrorLocation {
                byte_offset: 0,
                line: 1,
                column: 1,
            },
        ),
        ParseError::syntax_error(
            "missing semicolon",
            ErrorLocation {
                byte_offset: 20,
                line: 2,
                column: 5,
            },
        ),
        ParseError::syntax_error(
            "unclosed brace",
            ErrorLocation {
                byte_offset: 50,
                line: 4,
                column: 1,
            },
        ),
    ];
    assert_eq!(errors.len(), 3);
    for i in 0..errors.len() {
        assert!(errors[i].location.is_some());
    }
    // Verify ordering by byte_offset
    let offsets: Vec<usize> = errors
        .iter()
        .map(|e| e.location.as_ref().unwrap().byte_offset)
        .collect();
    assert!(offsets.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn cascading_errors_different_lines() {
    let mut errors = Vec::new();
    let lines = [
        (1, "unexpected 'if'"),
        (2, "expected expression"),
        (3, "missing '}'"),
        (5, "unexpected EOF"),
    ];
    for (line, msg) in &lines {
        errors.push(ParseError::syntax_error(
            *msg,
            ErrorLocation {
                byte_offset: line * 20,
                line: *line,
                column: 1,
            },
        ));
    }
    assert_eq!(errors.len(), 4);
    for i in 0..errors.len() {
        let loc = errors[i].location.as_ref().unwrap();
        assert_eq!(loc.line, lines[i].0);
    }
}

// ---------------------------------------------------------------------------
// 8. Parser error: no language set
// ---------------------------------------------------------------------------

#[test]
fn parser_parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parser_parse_utf8_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 9. Error from std::error::Error trait
// ---------------------------------------------------------------------------

#[test]
fn parse_error_implements_std_error() {
    let err = ParseError::timeout();
    let std_err: &dyn std::error::Error = &err;
    let msg = std_err.to_string();
    assert_eq!(msg, "parse timeout exceeded");
}

#[test]
fn parse_error_source_chain() {
    let err = ParseError::with_msg("top-level error");
    let std_err: &dyn std::error::Error = &err;
    // ParseError doesn't wrap a source, so source() should be None
    assert!(std_err.source().is_none());
}

// ---------------------------------------------------------------------------
// 10. VersionMismatch field access
// ---------------------------------------------------------------------------

#[test]
fn version_mismatch_fields() {
    let kind = ParseErrorKind::VersionMismatch {
        expected: 15,
        actual: 13,
    };
    if let ParseErrorKind::VersionMismatch { expected, actual } = kind {
        assert_eq!(expected, 15);
        assert_eq!(actual, 13);
    } else {
        panic!("expected VersionMismatch");
    }
}

#[test]
fn version_mismatch_zero_versions() {
    let err = ParseError {
        kind: ParseErrorKind::VersionMismatch {
            expected: 0,
            actual: 0,
        },
        location: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("0"));
}

// ---------------------------------------------------------------------------
// 11. Error recovery: parser reset after error
// ---------------------------------------------------------------------------

#[test]
fn parser_reset_after_error() {
    let mut parser = Parser::new();
    let _ = parser.parse(b"test", None); // fails: no language
    parser.reset();
    // Parser should still be usable after reset
    let result = parser.parse(b"test", None);
    assert!(result.is_err()); // still no language, but no panic
}

// ---------------------------------------------------------------------------
// 12. Default parser state
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 13. Timeout configuration doesn't cause errors by itself
// ---------------------------------------------------------------------------

#[test]
fn parser_set_timeout_does_not_error() {
    let mut parser = Parser::new();
    parser.set_timeout(std::time::Duration::from_secs(5));
    assert_eq!(
        parser.timeout(),
        Some(std::time::Duration::from_secs(5))
    );
}
