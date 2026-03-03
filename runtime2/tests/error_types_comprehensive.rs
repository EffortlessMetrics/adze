#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for error types in adze-runtime (runtime2 crate).

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};
use std::error::Error;
use std::fmt::Write;

// ---------------------------------------------------------------------------
// ParseError construction
// ---------------------------------------------------------------------------

#[test]
fn test_no_language_constructor() {
    let err = ParseError::no_language();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    assert!(err.location.is_none());
}

#[test]
fn test_timeout_constructor() {
    let err = ParseError::timeout();
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
    assert!(err.location.is_none());
}

#[test]
fn test_syntax_error_constructor() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let err = ParseError::syntax_error("unexpected token", loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::SyntaxError(_)));
    assert_eq!(err.location, Some(loc));
}

#[test]
fn test_with_msg_constructor() {
    let err = ParseError::with_msg("something broke");
    assert!(matches!(err.kind, ParseErrorKind::Other(ref s) if s == "something broke"));
    assert!(err.location.is_none());
}

#[test]
fn test_with_location_builder() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::timeout().with_location(loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
    assert_eq!(err.location, Some(loc));
}

// ---------------------------------------------------------------------------
// ParseErrorKind variants
// ---------------------------------------------------------------------------

#[test]
fn test_error_kind_no_language() {
    let kind = ParseErrorKind::NoLanguage;
    assert_eq!(kind.to_string(), "no language set");
}

#[test]
fn test_error_kind_timeout() {
    let kind = ParseErrorKind::Timeout;
    assert_eq!(kind.to_string(), "parse timeout exceeded");
}

#[test]
fn test_error_kind_invalid_encoding() {
    let kind = ParseErrorKind::InvalidEncoding;
    assert_eq!(kind.to_string(), "invalid input encoding");
}

#[test]
fn test_error_kind_cancelled() {
    let kind = ParseErrorKind::Cancelled;
    assert_eq!(kind.to_string(), "parse cancelled");
}

#[test]
fn test_error_kind_version_mismatch() {
    let kind = ParseErrorKind::VersionMismatch {
        expected: 14,
        actual: 13,
    };
    assert_eq!(
        kind.to_string(),
        "language version mismatch: expected 14, got 13"
    );
}

#[test]
fn test_error_kind_syntax_error() {
    let kind = ParseErrorKind::SyntaxError("unexpected '}'".to_string());
    assert_eq!(kind.to_string(), "syntax error at unexpected '}'");
}

#[test]
fn test_error_kind_allocation_error() {
    let kind = ParseErrorKind::AllocationError;
    assert_eq!(kind.to_string(), "memory allocation failed");
}

#[test]
fn test_error_kind_other() {
    let kind = ParseErrorKind::Other("custom problem".to_string());
    assert_eq!(kind.to_string(), "custom problem");
}

// ---------------------------------------------------------------------------
// ErrorLocation fields and Display
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_fields() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 7,
    };
    assert_eq!(loc.byte_offset, 42);
    assert_eq!(loc.line, 3);
    assert_eq!(loc.column, 7);
}

#[test]
fn test_error_location_display() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 10,
        column: 20,
    };
    assert_eq!(loc.to_string(), "10:20");
}

#[test]
fn test_error_location_clone_and_eq() {
    let loc = ErrorLocation {
        byte_offset: 5,
        line: 1,
        column: 6,
    };
    let loc2 = loc.clone();
    assert_eq!(loc, loc2);
}

// ---------------------------------------------------------------------------
// Display formatting for ParseError
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_display_delegates_to_kind() {
    let err = ParseError::no_language();
    assert_eq!(err.to_string(), "no language set");
}

#[test]
fn test_parse_error_display_syntax_error() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::syntax_error("missing semicolon", loc);
    assert_eq!(err.to_string(), "syntax error at missing semicolon");
}

#[test]
fn test_parse_error_display_with_msg() {
    let err = ParseError::with_msg("internal failure");
    assert_eq!(err.to_string(), "internal failure");
}

// ---------------------------------------------------------------------------
// Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_debug() {
    let err = ParseError::timeout();
    let debug = format!("{:?}", err);
    assert!(debug.contains("Timeout"));
    assert!(debug.contains("ParseError"));
}

#[test]
fn test_error_location_debug() {
    let loc = ErrorLocation {
        byte_offset: 99,
        line: 5,
        column: 12,
    };
    let debug = format!("{:?}", loc);
    assert!(debug.contains("99"));
    assert!(debug.contains("5"));
    assert!(debug.contains("12"));
}

#[test]
fn test_error_kind_debug() {
    let kind = ParseErrorKind::Cancelled;
    let debug = format!("{:?}", kind);
    assert!(debug.contains("Cancelled"));
}

// ---------------------------------------------------------------------------
// std::error::Error trait (source chain)
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_is_std_error() {
    let err = ParseError::no_language();
    // ParseError implements std::error::Error
    let _: &dyn Error = &err;
}

#[test]
fn test_parse_error_source_is_none() {
    let err = ParseError::timeout();
    // #[error("{kind}")] formats via kind but does not set source()
    let source = err.source();
    assert!(source.is_none());
}

// ---------------------------------------------------------------------------
// Error propagation patterns
// ---------------------------------------------------------------------------

fn fallible_parse() -> Result<(), ParseError> {
    Err(ParseError::no_language())
}

#[test]
fn test_error_propagation_with_question_mark() {
    fn wrapper() -> Result<(), ParseError> {
        fallible_parse()?;
        Ok(())
    }
    let result = wrapper();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().kind,
        ParseErrorKind::NoLanguage
    ));
}

#[test]
fn test_error_in_result_map_err() {
    let result: Result<(), &str> = Err("bad input");
    let mapped: Result<(), ParseError> = result.map_err(ParseError::with_msg);
    assert!(mapped.is_err());
    assert_eq!(mapped.unwrap_err().to_string(), "bad input");
}

// ---------------------------------------------------------------------------
// Multiple errors collection
// ---------------------------------------------------------------------------

#[test]
fn test_collect_multiple_errors() {
    let errors: Vec<ParseError> = vec![
        ParseError::no_language(),
        ParseError::timeout(),
        ParseError::with_msg("error 3"),
    ];
    assert_eq!(errors.len(), 3);
    assert!(matches!(errors[0].kind, ParseErrorKind::NoLanguage));
    assert!(matches!(errors[1].kind, ParseErrorKind::Timeout));
    assert!(matches!(errors[2].kind, ParseErrorKind::Other(_)));
}

#[test]
fn test_filter_errors_by_kind() {
    let errors: Vec<ParseError> = vec![
        ParseError::no_language(),
        ParseError::timeout(),
        ParseError::no_language(),
        ParseError::with_msg("x"),
    ];
    let no_lang_count = errors
        .iter()
        .filter(|e| matches!(e.kind, ParseErrorKind::NoLanguage))
        .count();
    assert_eq!(no_lang_count, 2);
}

// ---------------------------------------------------------------------------
// Error comparison / matching
// ---------------------------------------------------------------------------

#[test]
fn test_error_kind_discriminant_matching() {
    let err = ParseError::timeout();
    let is_timeout = matches!(err.kind, ParseErrorKind::Timeout);
    assert!(is_timeout);
    let is_no_lang = matches!(err.kind, ParseErrorKind::NoLanguage);
    assert!(!is_no_lang);
}

#[test]
fn test_version_mismatch_fields() {
    let err = ParseError {
        kind: ParseErrorKind::VersionMismatch {
            expected: 15,
            actual: 12,
        },
        location: None,
    };
    if let ParseErrorKind::VersionMismatch { expected, actual } = err.kind {
        assert_eq!(expected, 15);
        assert_eq!(actual, 12);
    } else {
        panic!("expected VersionMismatch");
    }
}

// ---------------------------------------------------------------------------
// Error chaining
// ---------------------------------------------------------------------------

#[test]
fn test_with_location_chains_onto_no_language() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 10,
        column: 1,
    };
    let err = ParseError::no_language().with_location(loc.clone());
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    assert_eq!(err.location.unwrap(), loc);
}

#[test]
fn test_with_location_replaces_existing_location() {
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
    let err = ParseError::syntax_error("tok", loc1).with_location(loc2.clone());
    assert_eq!(err.location.unwrap(), loc2);
}

#[test]
fn test_display_write_to_buffer() {
    let err = ParseError::with_msg("buf test");
    let mut buf = String::new();
    write!(&mut buf, "Error: {err}").unwrap();
    assert_eq!(buf, "Error: buf test");
}

#[test]
fn test_error_location_inequality() {
    let a = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let b = ErrorLocation {
        byte_offset: 1,
        line: 1,
        column: 2,
    };
    assert_ne!(a, b);
}
