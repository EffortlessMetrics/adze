//! Tests for runtime2 error types and constructors.

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};

#[test]
fn parse_error_no_language() {
    let err = ParseError::no_language();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    assert!(err.location.is_none());
}

#[test]
fn parse_error_timeout() {
    let err = ParseError::timeout();
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
    assert!(err.location.is_none());
}

#[test]
fn parse_error_syntax_error() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 10,
    };
    let err = ParseError::syntax_error("unexpected token", loc);
    assert!(matches!(err.kind, ParseErrorKind::SyntaxError(_)));
    let location = err.location.unwrap();
    assert_eq!(location.byte_offset, 42);
    assert_eq!(location.line, 3);
    assert_eq!(location.column, 10);
}

#[test]
fn parse_error_with_location() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 5,
        column: 20,
    };
    let err = ParseError::no_language().with_location(loc);
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    let location = err.location.unwrap();
    assert_eq!(location.byte_offset, 100);
}

#[test]
fn parse_error_display() {
    let err = ParseError::no_language();
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn parse_error_debug() {
    let err = ParseError::timeout();
    let debug = format!("{err:?}");
    assert!(debug.contains("Timeout") || debug.contains("timeout"));
}

#[test]
fn error_location_default() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 0,
        column: 0,
    };
    assert_eq!(loc.byte_offset, 0);
    assert_eq!(loc.line, 0);
    assert_eq!(loc.column, 0);
}

#[test]
fn parse_error_kind_variants() {
    let kinds = [
        ParseErrorKind::NoLanguage,
        ParseErrorKind::Timeout,
        ParseErrorKind::SyntaxError("test".to_string()),
    ];
    for kind in &kinds {
        let debug = format!("{kind:?}");
        assert!(!debug.is_empty());
    }
}

#[test]
fn error_location_zero() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 0,
    };
    let debug = format!("{loc:?}");
    assert!(!debug.is_empty());
}
