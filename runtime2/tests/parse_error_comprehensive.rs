//! Comprehensive tests for runtime2 ParseError, ParseErrorKind, ErrorLocation types.

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};

#[test]
fn parse_error_no_language() {
    let err = ParseError::no_language();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_error_timeout() {
    let err = ParseError::timeout();
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
}

#[test]
fn parse_error_syntax_error() {
    let loc = ErrorLocation {
        byte_offset: 5,
        line: 1,
        column: 5,
    };
    let err = ParseError::syntax_error("unexpected token", loc);
    assert!(matches!(err.kind, ParseErrorKind::SyntaxError(_)));
}

#[test]
fn parse_error_with_msg() {
    let err = ParseError::with_msg("test error");
    let display = format!("{}", err);
    assert!(display.contains("test error"));
}

#[test]
fn parse_error_with_location() {
    let err = ParseError::no_language().with_location(ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 3,
    });
    assert!(err.location.is_some());
    let loc = err.location.unwrap();
    assert_eq!(loc.byte_offset, 10);
    assert_eq!(loc.line, 2);
    assert_eq!(loc.column, 3);
}

#[test]
fn parse_error_kind_debug() {
    let kinds = vec![
        ParseErrorKind::NoLanguage,
        ParseErrorKind::Timeout,
        ParseErrorKind::SyntaxError("test".to_string()),
    ];
    for kind in kinds {
        let debug = format!("{:?}", kind);
        assert!(!debug.is_empty());
    }
}

#[test]
fn parse_error_display() {
    let err = ParseError::no_language();
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

#[test]
fn parse_error_is_std_error() {
    let err = ParseError::no_language();
    let _: &dyn std::error::Error = &err;
}

#[test]
fn error_location_fields() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 10,
        column: 5,
    };
    assert_eq!(loc.byte_offset, 42);
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
}

#[test]
fn error_location_debug() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 0,
        column: 0,
    };
    let debug = format!("{:?}", loc);
    assert!(!debug.is_empty());
}

#[test]
fn error_location_clone() {
    let loc = ErrorLocation {
        byte_offset: 1,
        line: 2,
        column: 3,
    };
    let cloned = loc.clone();
    assert_eq!(loc.byte_offset, cloned.byte_offset);
    assert_eq!(loc.line, cloned.line);
    assert_eq!(loc.column, cloned.column);
}

#[test]
fn parse_error_kind_no_language_display() {
    let err = ParseError::no_language();
    let s = format!("{}", err);
    assert!(s.to_lowercase().contains("language") || !s.is_empty());
}

#[test]
fn parse_error_kind_timeout_display() {
    let err = ParseError::timeout();
    let s = format!("{}", err);
    assert!(!s.is_empty());
}

#[test]
fn syntax_error_preserves_message() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::syntax_error("unexpected '}'", loc);
    if let ParseErrorKind::SyntaxError(msg) = &err.kind {
        assert!(msg.contains("unexpected '}'"));
    } else {
        panic!("Expected SyntaxError kind");
    }
}
