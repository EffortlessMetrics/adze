#![allow(clippy::needless_range_loop)]

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};
use std::fmt::Write as _;

// === ErrorLocation creation ===

#[test]
fn error_location_basic_creation() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(loc.byte_offset, 0);
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 1);
}

#[test]
fn error_location_nonzero_offset() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 10,
    };
    assert_eq!(loc.byte_offset, 42);
    assert_eq!(loc.line, 3);
    assert_eq!(loc.column, 10);
}

#[test]
fn error_location_large_values() {
    let loc = ErrorLocation {
        byte_offset: usize::MAX,
        line: usize::MAX,
        column: usize::MAX,
    };
    assert_eq!(loc.byte_offset, usize::MAX);
    assert_eq!(loc.line, usize::MAX);
    assert_eq!(loc.column, usize::MAX);
}

// === ErrorLocation Display ===

#[test]
fn error_location_display_format() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 5,
        column: 3,
    };
    assert_eq!(format!("{loc}"), "5:3");
}

#[test]
fn error_location_display_one_one() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(loc.to_string(), "1:1");
}

// === ErrorLocation Debug ===

#[test]
fn error_location_debug_contains_fields() {
    let loc = ErrorLocation {
        byte_offset: 7,
        line: 2,
        column: 4,
    };
    let dbg = format!("{loc:?}");
    assert!(dbg.contains("byte_offset"));
    assert!(dbg.contains("7"));
    assert!(dbg.contains("line"));
    assert!(dbg.contains("2"));
    assert!(dbg.contains("column"));
    assert!(dbg.contains("4"));
}

// === ErrorLocation Clone and Eq ===

#[test]
fn error_location_clone() {
    let loc = ErrorLocation {
        byte_offset: 5,
        line: 2,
        column: 3,
    };
    let cloned = loc.clone();
    assert_eq!(loc, cloned);
}

#[test]
fn error_location_equality() {
    let a = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let b = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    assert_eq!(a, b);
}

#[test]
fn error_location_inequality_byte_offset() {
    let a = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let b = ErrorLocation {
        byte_offset: 11,
        line: 2,
        column: 5,
    };
    assert_ne!(a, b);
}

#[test]
fn error_location_inequality_line() {
    let a = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let b = ErrorLocation {
        byte_offset: 10,
        line: 3,
        column: 5,
    };
    assert_ne!(a, b);
}

#[test]
fn error_location_inequality_column() {
    let a = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let b = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 6,
    };
    assert_ne!(a, b);
}

// === ParseError constructors ===

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
        byte_offset: 20,
        line: 4,
        column: 8,
    };
    let err = ParseError::syntax_error("unexpected token", loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::SyntaxError(_)));
    assert_eq!(err.location, Some(loc));
}

#[test]
fn parse_error_with_msg() {
    let err = ParseError::with_msg("something broke");
    assert!(matches!(err.kind, ParseErrorKind::Other(ref s) if s == "something broke"));
    assert!(err.location.is_none());
}

// === ParseError::with_location ===

#[test]
fn parse_error_with_location_chain() {
    let loc = ErrorLocation {
        byte_offset: 50,
        line: 10,
        column: 1,
    };
    let err = ParseError::no_language().with_location(loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    assert_eq!(err.location, Some(loc));
}

#[test]
fn parse_error_with_location_replaces_existing() {
    let loc1 = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 3,
    };
    let loc2 = ErrorLocation {
        byte_offset: 20,
        line: 4,
        column: 5,
    };
    let err = ParseError::syntax_error("err", loc1).with_location(loc2.clone());
    assert_eq!(err.location, Some(loc2));
}

// === ParseErrorKind Display ===

#[test]
fn parse_error_kind_no_language_display() {
    let err = ParseError::no_language();
    assert_eq!(err.to_string(), "no language set");
}

#[test]
fn parse_error_kind_timeout_display() {
    let err = ParseError::timeout();
    assert_eq!(err.to_string(), "parse timeout exceeded");
}

#[test]
fn parse_error_kind_invalid_encoding_display() {
    let err = ParseError {
        kind: ParseErrorKind::InvalidEncoding,
        location: None,
    };
    assert_eq!(err.to_string(), "invalid input encoding");
}

#[test]
fn parse_error_kind_cancelled_display() {
    let err = ParseError {
        kind: ParseErrorKind::Cancelled,
        location: None,
    };
    assert_eq!(err.to_string(), "parse cancelled");
}

#[test]
fn parse_error_kind_version_mismatch_display() {
    let err = ParseError {
        kind: ParseErrorKind::VersionMismatch {
            expected: 14,
            actual: 13,
        },
        location: None,
    };
    assert_eq!(
        err.to_string(),
        "language version mismatch: expected 14, got 13"
    );
}

#[test]
fn parse_error_kind_syntax_error_display() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::syntax_error("unexpected '}'", loc);
    assert_eq!(err.to_string(), "syntax error at unexpected '}'");
}

#[test]
fn parse_error_kind_allocation_error_display() {
    let err = ParseError {
        kind: ParseErrorKind::AllocationError,
        location: None,
    };
    assert_eq!(err.to_string(), "memory allocation failed");
}

#[test]
fn parse_error_kind_other_display() {
    let err = ParseError::with_msg("custom problem");
    assert_eq!(err.to_string(), "custom problem");
}

// === ParseError Debug ===

#[test]
fn parse_error_debug_contains_kind() {
    let err = ParseError::no_language();
    let dbg = format!("{err:?}");
    assert!(dbg.contains("NoLanguage"));
}

#[test]
fn parse_error_debug_with_location() {
    let loc = ErrorLocation {
        byte_offset: 5,
        line: 1,
        column: 6,
    };
    let err = ParseError::timeout().with_location(loc);
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Timeout"));
    assert!(dbg.contains("byte_offset"));
}

// === Multiple errors collection ===

#[test]
fn collect_multiple_errors() {
    let errors: Vec<ParseError> = vec![
        ParseError::no_language(),
        ParseError::timeout(),
        ParseError::with_msg("first"),
        ParseError::with_msg("second"),
    ];
    assert_eq!(errors.len(), 4);
    assert!(matches!(errors[0].kind, ParseErrorKind::NoLanguage));
    assert!(matches!(errors[1].kind, ParseErrorKind::Timeout));
    assert!(matches!(errors[2].kind, ParseErrorKind::Other(ref s) if s == "first"));
    assert!(matches!(errors[3].kind, ParseErrorKind::Other(ref s) if s == "second"));
}

#[test]
fn errors_with_mixed_locations() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 10,
        column: 20,
    };
    let errors: Vec<ParseError> = vec![
        ParseError::no_language(),
        ParseError::syntax_error("bad token", loc.clone()),
    ];
    assert!(errors[0].location.is_none());
    assert_eq!(errors[1].location.as_ref(), Some(&loc));
}

// === Error as std::error::Error ===

#[test]
fn parse_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(ParseError::no_language());
    assert_eq!(err.to_string(), "no language set");
}

// === Display via Write trait ===

#[test]
fn error_location_write_to_string() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 99,
        column: 42,
    };
    let mut buf = String::new();
    write!(buf, "error at {loc}").unwrap();
    assert_eq!(buf, "error at 99:42");
}

// === Byte offset edge cases ===

#[test]
fn error_location_zero_byte_offset() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(loc.byte_offset, 0);
    assert_eq!(loc.to_string(), "1:1");
}
