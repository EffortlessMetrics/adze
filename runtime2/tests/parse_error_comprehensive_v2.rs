//! Comprehensive tests for ParseError and ParseErrorKind.

use adze_runtime::error::ErrorLocation;
use adze_runtime::{ParseError, ParseErrorKind};

// ─── ParseErrorKind variants ───

#[test]
fn no_language_error() {
    let e = ParseErrorKind::NoLanguage;
    assert_eq!(e.to_string(), "no language set");
}

#[test]
fn timeout_error() {
    let e = ParseErrorKind::Timeout;
    assert_eq!(e.to_string(), "parse timeout exceeded");
}

#[test]
fn invalid_encoding_error() {
    let e = ParseErrorKind::InvalidEncoding;
    assert_eq!(e.to_string(), "invalid input encoding");
}

#[test]
fn cancelled_error() {
    let e = ParseErrorKind::Cancelled;
    assert_eq!(e.to_string(), "parse cancelled");
}

#[test]
fn version_mismatch_error() {
    let e = ParseErrorKind::VersionMismatch {
        expected: 14,
        actual: 15,
    };
    let msg = e.to_string();
    assert!(msg.contains("14"));
    assert!(msg.contains("15"));
}

#[test]
fn syntax_error() {
    let e = ParseErrorKind::SyntaxError("unexpected token".to_string());
    let msg = e.to_string();
    assert!(msg.contains("unexpected token"));
}

#[test]
fn allocation_error() {
    let e = ParseErrorKind::AllocationError;
    assert_eq!(e.to_string(), "memory allocation failed");
}

#[test]
fn other_error() {
    let e = ParseErrorKind::Other("custom".to_string());
    assert_eq!(e.to_string(), "custom");
}

// ─── ParseError construction ───

#[test]
fn parse_error_no_language() {
    let e = ParseError::no_language();
    assert!(e.to_string().contains("no language"));
    assert!(e.location.is_none());
}

#[test]
fn parse_error_timeout() {
    let e = ParseError::timeout();
    assert!(e.to_string().contains("timeout"));
    assert!(e.location.is_none());
}

#[test]
fn parse_error_syntax_with_location() {
    let loc = ErrorLocation {
        line: 5,
        column: 10,
        byte_offset: 42,
    };
    let e = ParseError::syntax_error("unexpected }", loc);
    assert!(e.to_string().contains("unexpected }"));
    assert!(e.location.is_some());
    let loc = e.location.unwrap();
    assert_eq!(loc.line, 5);
    assert_eq!(loc.column, 10);
    assert_eq!(loc.byte_offset, 42);
}

#[test]
fn parse_error_with_msg() {
    let e = ParseError::with_msg("something went wrong");
    assert_eq!(e.to_string(), "something went wrong");
    assert!(e.location.is_none());
}

#[test]
fn parse_error_with_location() {
    let e = ParseError::no_language().with_location(ErrorLocation {
        line: 1,
        column: 0,
        byte_offset: 0,
    });
    assert!(e.location.is_some());
}

// ─── ErrorLocation ───

#[test]
fn error_location_display() {
    let loc = ErrorLocation {
        line: 3,
        column: 7,
        byte_offset: 20,
    };
    let s = format!("{}", loc);
    assert!(s.contains("3"));
    assert!(s.contains("7"));
}

#[test]
fn error_location_fields() {
    let loc = ErrorLocation {
        line: 10,
        column: 20,
        byte_offset: 100,
    };
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 20);
    assert_eq!(loc.byte_offset, 100);
}

#[test]
fn error_location_zero() {
    let loc = ErrorLocation {
        line: 0,
        column: 0,
        byte_offset: 0,
    };
    assert_eq!(loc.line, 0);
    assert_eq!(loc.column, 0);
    assert_eq!(loc.byte_offset, 0);
}

#[test]
fn error_location_large_values() {
    let loc = ErrorLocation {
        line: 1_000_000,
        column: 500_000,
        byte_offset: usize::MAX,
    };
    assert_eq!(loc.line, 1_000_000);
    assert_eq!(loc.byte_offset, usize::MAX);
}

// ─── Debug format ───

#[test]
fn parse_error_debug() {
    let e = ParseError::no_language();
    let d = format!("{:?}", e);
    assert!(d.contains("ParseError"));
}

#[test]
fn parse_error_kind_debug() {
    let variants: Vec<ParseErrorKind> = vec![
        ParseErrorKind::NoLanguage,
        ParseErrorKind::Timeout,
        ParseErrorKind::InvalidEncoding,
        ParseErrorKind::Cancelled,
        ParseErrorKind::AllocationError,
        ParseErrorKind::Other("test".to_string()),
        ParseErrorKind::SyntaxError("test".to_string()),
        ParseErrorKind::VersionMismatch {
            expected: 1,
            actual: 2,
        },
    ];
    for v in &variants {
        let d = format!("{:?}", v);
        assert!(!d.is_empty());
    }
}

// ─── Error trait ───

#[test]
fn parse_error_is_std_error() {
    let e = ParseError::no_language();
    let _: &dyn std::error::Error = &e;
}

#[test]
fn parse_error_display() {
    let e = ParseError::timeout();
    let msg = format!("{}", e);
    assert!(!msg.is_empty());
}

// ─── ParseError kind access ───

#[test]
fn parse_error_kind_field() {
    let e = ParseError::no_language();
    match &e.kind {
        ParseErrorKind::NoLanguage => {}
        _ => panic!("expected NoLanguage"),
    }
}

#[test]
fn parse_error_timeout_kind() {
    let e = ParseError::timeout();
    match &e.kind {
        ParseErrorKind::Timeout => {}
        _ => panic!("expected Timeout"),
    }
}

#[test]
fn parse_error_other_kind() {
    let e = ParseError::with_msg("custom");
    match &e.kind {
        ParseErrorKind::Other(msg) => assert_eq!(msg, "custom"),
        _ => panic!("expected Other"),
    }
}

// ─── with_location chaining ───

#[test]
fn with_location_replaces_none() {
    let e = ParseError::no_language().with_location(ErrorLocation {
        line: 1,
        column: 2,
        byte_offset: 3,
    });
    let loc = e.location.unwrap();
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 2);
    assert_eq!(loc.byte_offset, 3);
}

#[test]
fn with_location_replaces_existing() {
    let e = ParseError::syntax_error(
        "err",
        ErrorLocation {
            line: 1,
            column: 1,
            byte_offset: 0,
        },
    )
    .with_location(ErrorLocation {
        line: 2,
        column: 2,
        byte_offset: 10,
    });
    let loc = e.location.unwrap();
    assert_eq!(loc.line, 2);
    assert_eq!(loc.byte_offset, 10);
}

// ─── Syntax error variants ───

#[test]
fn syntax_error_empty_message() {
    let loc = ErrorLocation {
        line: 0,
        column: 0,
        byte_offset: 0,
    };
    let e = ParseError::syntax_error("", loc);
    let msg = e.to_string();
    assert!(msg.contains("syntax error"));
}

#[test]
fn syntax_error_long_message() {
    let long = "x".repeat(1000);
    let loc = ErrorLocation {
        line: 0,
        column: 0,
        byte_offset: 0,
    };
    let e = ParseError::syntax_error(&long, loc);
    let msg = e.to_string();
    assert!(msg.contains(&long));
}

// ─── Version mismatch details ───

#[test]
fn version_mismatch_fields() {
    let e = ParseErrorKind::VersionMismatch {
        expected: 14,
        actual: 15,
    };
    if let ParseErrorKind::VersionMismatch { expected, actual } = e {
        assert_eq!(expected, 14);
        assert_eq!(actual, 15);
    }
}

#[test]
fn version_mismatch_same_version() {
    let e = ParseErrorKind::VersionMismatch {
        expected: 15,
        actual: 15,
    };
    let msg = e.to_string();
    assert!(msg.contains("15"));
}

// ─── with_msg edge cases ───

#[test]
fn with_msg_empty() {
    let e = ParseError::with_msg("");
    assert_eq!(e.to_string(), "");
}

#[test]
fn with_msg_unicode() {
    let e = ParseError::with_msg("错误: 解析失败 🔥");
    assert!(e.to_string().contains("🔥"));
}

// ─── Multiple errors ───

#[test]
fn create_multiple_errors() {
    let errors: Vec<ParseError> = vec![
        ParseError::no_language(),
        ParseError::timeout(),
        ParseError::with_msg("a"),
        ParseError::with_msg("b"),
        ParseError::syntax_error(
            "c",
            ErrorLocation {
                line: 0,
                column: 0,
                byte_offset: 0,
            },
        ),
    ];
    assert_eq!(errors.len(), 5);
    for e in &errors {
        assert!(!e.to_string().is_empty());
    }
}
