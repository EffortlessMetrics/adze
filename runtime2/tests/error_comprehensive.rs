//! Comprehensive tests for ParseError, ParseErrorKind, and ErrorLocation.

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
}

#[test]
fn parse_error_with_msg() {
    let err = ParseError::with_msg("test error");
    let display = format!("{}", err);
    assert!(display.contains("test error"));
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
    assert!(err.location.is_some());
    assert_eq!(err.location.unwrap().byte_offset, 42);
}

#[test]
fn parse_error_with_location() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::no_language().with_location(loc);
    assert!(err.location.is_some());
    assert_eq!(err.location.unwrap().line, 1);
}

#[test]
fn error_location_display() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 5,
        column: 12,
    };
    let display = format!("{}", loc);
    assert!(display.contains("5") || display.contains("12"));
}

#[test]
fn parse_error_display_no_language() {
    let err = ParseError::no_language();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_display_timeout() {
    let err = ParseError::timeout();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_debug_impl() {
    let err = ParseError::no_language();
    let debug = format!("{:?}", err);
    assert!(debug.contains("ParseError"));
}

#[test]
fn error_location_zero() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 0,
        column: 0,
    };
    let _ = format!("{}", loc);
}

#[test]
fn error_location_large_values() {
    let loc = ErrorLocation {
        byte_offset: usize::MAX,
        line: usize::MAX,
        column: usize::MAX,
    };
    let _ = format!("{}", loc);
}

#[test]
fn parse_error_kind_variants() {
    let _ = ParseErrorKind::NoLanguage;
    let _ = ParseErrorKind::Timeout;
    let _ = ParseErrorKind::SyntaxError("test".to_string());
    let _ = ParseErrorKind::Other("other".to_string());
}

#[test]
fn parse_error_with_msg_empty() {
    let err = ParseError::with_msg("");
    let msg = format!("{}", err);
    // Should still work even with empty message
    let _ = msg;
}

#[test]
fn parse_error_syntax_error_with_location_chain() {
    let loc1 = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let loc2 = ErrorLocation {
        byte_offset: 20,
        line: 3,
        column: 8,
    };
    let err = ParseError::syntax_error("err", loc1).with_location(loc2);
    // with_location replaces the existing location
    assert_eq!(err.location.unwrap().byte_offset, 20);
}

#[test]
fn parse_error_from_glr_error_lex() {
    use adze_glr_core::driver::GlrError;
    let glr_err = GlrError::Lex("bad token".to_string());
    let parse_err: ParseError = glr_err.into();
    let msg = format!("{}", parse_err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_from_glr_error_parse() {
    use adze_glr_core::driver::GlrError;
    let glr_err = GlrError::Parse("unexpected symbol".to_string());
    let parse_err: ParseError = glr_err.into();
    let msg = format!("{}", parse_err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_from_glr_error_other() {
    use adze_glr_core::driver::GlrError;
    let glr_err = GlrError::Other("misc".to_string());
    let parse_err: ParseError = glr_err.into();
    let msg = format!("{}", parse_err);
    assert!(!msg.is_empty());
}
