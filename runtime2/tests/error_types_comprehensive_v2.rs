//! Comprehensive error types tests v2 — expanded coverage.

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};

// === ErrorLocation edge cases ===

#[test]
fn location_display_1_1() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(loc.to_string(), "1:1");
}

#[test]
fn location_display_large() {
    let loc = ErrorLocation {
        byte_offset: 99999,
        line: 500,
        column: 80,
    };
    assert_eq!(loc.to_string(), "500:80");
}

#[test]
fn location_display_zero_zero() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 0,
        column: 0,
    };
    assert_eq!(loc.to_string(), "0:0");
}

#[test]
fn location_eq_same() {
    let a = ErrorLocation {
        byte_offset: 5,
        line: 2,
        column: 3,
    };
    let b = ErrorLocation {
        byte_offset: 5,
        line: 2,
        column: 3,
    };
    assert_eq!(a, b);
}

#[test]
fn location_ne_byte_offset() {
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
fn location_ne_line() {
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
fn location_ne_column() {
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
fn location_clone_eq() {
    let a = ErrorLocation {
        byte_offset: 10,
        line: 3,
        column: 7,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn location_debug_contains_fields() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 5,
        column: 10,
    };
    let d = format!("{:?}", loc);
    assert!(d.contains("42") || d.contains("ErrorLocation"));
}

#[test]
fn location_max_values() {
    let loc = ErrorLocation {
        byte_offset: usize::MAX,
        line: usize::MAX,
        column: usize::MAX,
    };
    let s = loc.to_string();
    assert!(!s.is_empty());
}

// === ParseErrorKind display ===

#[test]
fn kind_no_language_exact() {
    assert_eq!(ParseErrorKind::NoLanguage.to_string(), "no language set");
}

#[test]
fn kind_timeout_exact() {
    assert_eq!(
        ParseErrorKind::Timeout.to_string(),
        "parse timeout exceeded"
    );
}

#[test]
fn kind_invalid_encoding_exact() {
    assert_eq!(
        ParseErrorKind::InvalidEncoding.to_string(),
        "invalid input encoding"
    );
}

#[test]
fn kind_cancelled_exact() {
    assert_eq!(ParseErrorKind::Cancelled.to_string(), "parse cancelled");
}

#[test]
fn kind_allocation_exact() {
    assert_eq!(
        ParseErrorKind::AllocationError.to_string(),
        "memory allocation failed"
    );
}

#[test]
fn kind_version_mismatch_content() {
    let k = ParseErrorKind::VersionMismatch {
        expected: 14,
        actual: 15,
    };
    let s = k.to_string();
    assert!(s.contains("14") && s.contains("15"));
}

#[test]
fn kind_syntax_error_content() {
    let k = ParseErrorKind::SyntaxError("bad".into());
    assert!(k.to_string().contains("bad"));
}

#[test]
fn kind_other_content() {
    let k = ParseErrorKind::Other("custom msg".into());
    assert_eq!(k.to_string(), "custom msg");
}

#[test]
fn kind_other_empty_string() {
    let k = ParseErrorKind::Other(String::new());
    assert_eq!(k.to_string(), "");
}

// === ParseErrorKind debug ===

#[test]
fn kind_debug_all_variants() {
    let variants: Vec<ParseErrorKind> = vec![
        ParseErrorKind::NoLanguage,
        ParseErrorKind::Timeout,
        ParseErrorKind::InvalidEncoding,
        ParseErrorKind::Cancelled,
        ParseErrorKind::VersionMismatch {
            expected: 1,
            actual: 2,
        },
        ParseErrorKind::SyntaxError("x".into()),
        ParseErrorKind::AllocationError,
        ParseErrorKind::Other("y".into()),
    ];
    for v in &variants {
        let d = format!("{:?}", v);
        assert!(!d.is_empty());
    }
}

// === ParseError constructors ===

#[test]
fn error_no_language_no_location() {
    let e = ParseError::no_language();
    assert!(e.location.is_none());
}

#[test]
fn error_timeout_no_location() {
    let e = ParseError::timeout();
    assert!(e.location.is_none());
}

#[test]
fn error_syntax_has_location() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let e = ParseError::syntax_error("bad", loc.clone());
    assert_eq!(e.location, Some(loc));
}

#[test]
fn error_with_msg_no_location() {
    let e = ParseError::with_msg("test");
    assert!(e.location.is_none());
}

#[test]
fn error_with_location_adds() {
    let e = ParseError::no_language();
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let e2 = e.with_location(loc.clone());
    assert_eq!(e2.location, Some(loc));
}

#[test]
fn error_with_location_replaces() {
    let loc1 = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let loc2 = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let e = ParseError::syntax_error("test", loc1);
    let e2 = e.with_location(loc2.clone());
    assert_eq!(e2.location, Some(loc2));
}

// === ParseError Display ===

#[test]
fn error_display_no_language() {
    let e = ParseError::no_language();
    assert!(e.to_string().contains("no language"));
}

#[test]
fn error_display_timeout() {
    let e = ParseError::timeout();
    assert!(e.to_string().contains("timeout"));
}

#[test]
fn error_display_syntax() {
    let loc = ErrorLocation {
        byte_offset: 5,
        line: 1,
        column: 6,
    };
    let e = ParseError::syntax_error("EOF", loc);
    assert!(e.to_string().contains("EOF"));
}

#[test]
fn error_display_custom() {
    let e = ParseError::with_msg("foo bar");
    assert_eq!(e.to_string(), "foo bar");
}

#[test]
fn error_display_unicode() {
    let e = ParseError::with_msg("unexpected «λ» token");
    assert!(e.to_string().contains("λ"));
}

#[test]
fn error_display_long() {
    let msg = "x".repeat(10000);
    let e = ParseError::with_msg(&msg);
    assert_eq!(e.to_string().len(), 10000);
}

// === Error trait ===

#[test]
fn parse_error_std_error() {
    fn check<E: std::error::Error>(_e: &E) {}
    check(&ParseError::no_language());
}

#[test]
fn parse_error_kind_std_error() {
    fn check<E: std::error::Error>(_e: &E) {}
    check(&ParseErrorKind::NoLanguage);
}

// === Debug ===

#[test]
fn error_debug_has_type() {
    let e = ParseError::no_language();
    let d = format!("{:?}", e);
    assert!(d.contains("ParseError"));
}

#[test]
fn error_debug_with_location() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let e = ParseError::syntax_error("t", loc);
    let d = format!("{:?}", e);
    assert!(d.contains("ParseError"));
}

// === Version mismatch edge cases ===

#[test]
fn version_mismatch_zero_zero() {
    let k = ParseErrorKind::VersionMismatch {
        expected: 0,
        actual: 0,
    };
    let s = k.to_string();
    assert!(s.contains("0"));
}

#[test]
fn version_mismatch_large() {
    let k = ParseErrorKind::VersionMismatch {
        expected: u32::MAX,
        actual: u32::MAX,
    };
    let s = k.to_string();
    assert!(!s.is_empty());
}

// === Syntax error edge cases ===

#[test]
fn syntax_error_empty_msg() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let e = ParseError::syntax_error("", loc);
    let _ = e.to_string();
}

#[test]
fn syntax_error_string_arg() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let e = ParseError::syntax_error(String::from("owned msg"), loc);
    assert!(e.to_string().contains("owned msg"));
}
