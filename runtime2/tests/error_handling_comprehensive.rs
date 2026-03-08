#![allow(clippy::needless_range_loop)]

//! Comprehensive error handling tests for adze-runtime (runtime2 crate).
//! Tests cover ParseError, ParseErrorKind, and ErrorLocation types with 25+ test cases.

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};
use std::error::Error;
use std::fmt::Write;

// ---------------------------------------------------------------------------
// 1. ParseError creation with basic message
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_with_basic_message() {
    let err = ParseError::with_msg("basic error message");
    assert!(matches!(err.kind, ParseErrorKind::Other(ref s) if s == "basic error message"));
    assert!(err.location.is_none());
    assert_eq!(err.to_string(), "basic error message");
}

// ---------------------------------------------------------------------------
// 2. ParseError with all kinds
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_no_language_kind() {
    let err = ParseError::no_language();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn test_parse_error_timeout_kind() {
    let err = ParseError::timeout();
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
}

#[test]
fn test_parse_error_invalid_encoding_kind() {
    let kind = ParseErrorKind::InvalidEncoding;
    assert_eq!(kind.to_string(), "invalid input encoding");
}

#[test]
fn test_parse_error_cancelled_kind() {
    let kind = ParseErrorKind::Cancelled;
    assert_eq!(kind.to_string(), "parse cancelled");
}

#[test]
fn test_parse_error_version_mismatch_kind() {
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
fn test_parse_error_syntax_error_kind() {
    let kind = ParseErrorKind::SyntaxError("unexpected closing brace".to_string());
    assert_eq!(kind.to_string(), "syntax error at unexpected closing brace");
}

#[test]
fn test_parse_error_allocation_error_kind() {
    let kind = ParseErrorKind::AllocationError;
    assert_eq!(kind.to_string(), "memory allocation failed");
}

#[test]
fn test_parse_error_other_kind() {
    let kind = ParseErrorKind::Other("custom problem".to_string());
    assert_eq!(kind.to_string(), "custom problem");
}

// ---------------------------------------------------------------------------
// 3. ParseError display formatting
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_display_formatting() {
    let err = ParseError::timeout();
    let display = format!("{}", err);
    assert_eq!(display, "parse timeout exceeded");
}

#[test]
fn test_parse_error_display_syntax_error() {
    let err = ParseError::syntax_error(
        "missing semicolon",
        ErrorLocation {
            byte_offset: 42,
            line: 3,
            column: 10,
        },
    );
    assert_eq!(err.to_string(), "syntax error at missing semicolon");
}

// ---------------------------------------------------------------------------
// 4. ParseError debug formatting
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_debug_formatting() {
    let err = ParseError::no_language();
    let debug = format!("{:?}", err);
    assert!(debug.contains("ParseError") || debug.contains("NoLanguage"));
}

#[test]
fn test_error_location_debug_formatting() {
    let loc = ErrorLocation {
        byte_offset: 99,
        line: 5,
        column: 12,
    };
    let debug = format!("{:?}", loc);
    assert!(debug.contains("99") || debug.contains("5") || debug.contains("12"));
}

#[test]
fn test_parse_error_kind_debug_formatting() {
    let kind = ParseErrorKind::Timeout;
    let debug = format!("{:?}", kind);
    assert!(debug.contains("Timeout"));
}

// ---------------------------------------------------------------------------
// 5. ParseErrorKind variants enumeration
// ---------------------------------------------------------------------------

#[test]
fn test_all_parse_error_kind_variants_exist() {
    let _no_lang = ParseErrorKind::NoLanguage;
    let _timeout = ParseErrorKind::Timeout;
    let _invalid_enc = ParseErrorKind::InvalidEncoding;
    let _cancelled = ParseErrorKind::Cancelled;
    let _version = ParseErrorKind::VersionMismatch {
        expected: 1,
        actual: 2,
    };
    let _syntax = ParseErrorKind::SyntaxError("test".to_string());
    let _alloc = ParseErrorKind::AllocationError;
    let _other = ParseErrorKind::Other("test".to_string());
}

// ---------------------------------------------------------------------------
// 6. ErrorLocation with byte offsets
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_byte_offset() {
    let loc = ErrorLocation {
        byte_offset: 256,
        line: 10,
        column: 5,
    };
    assert_eq!(loc.byte_offset, 256);
}

#[test]
fn test_error_location_byte_offset_zero() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(loc.byte_offset, 0);
}

#[test]
fn test_error_location_byte_offset_large() {
    let loc = ErrorLocation {
        byte_offset: 1_000_000,
        line: 100,
        column: 50,
    };
    assert_eq!(loc.byte_offset, 1_000_000);
}

// ---------------------------------------------------------------------------
// 7. ErrorLocation with point positions
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_line_and_column() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 42,
        column: 17,
    };
    assert_eq!(loc.line, 42);
    assert_eq!(loc.column, 17);
}

#[test]
fn test_error_location_display_format() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 5,
        column: 10,
    };
    assert_eq!(loc.to_string(), "5:10");
}

#[test]
fn test_error_location_display_with_large_numbers() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 999,
        column: 888,
    };
    assert_eq!(loc.to_string(), "999:888");
}

// ---------------------------------------------------------------------------
// 8. ParseError chain with location (with_location method)
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_with_location_chaining() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 10,
        column: 1,
    };
    let err = ParseError::no_language().with_location(loc.clone());
    assert_eq!(err.location.unwrap(), loc);
}

#[test]
fn test_parse_error_location_override() {
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
    assert_eq!(err.location.unwrap(), loc2);
}

// ---------------------------------------------------------------------------
// 9. ParseError from string conversion / with_msg
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_from_static_str() {
    let err = ParseError::with_msg("static string");
    assert_eq!(err.to_string(), "static string");
}

#[test]
fn test_parse_error_from_owned_string() {
    let msg = String::from("owned string message");
    let err = ParseError::with_msg(&msg);
    assert_eq!(err.to_string(), "owned string message");
}

// ---------------------------------------------------------------------------
// 10. ParseError equality comparison (via kind and location)
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_kind_matching() {
    let err1 = ParseError::timeout();
    let err2 = ParseError::timeout();
    // Both are timeouts, can match via pattern
    assert!(matches!(err1.kind, ParseErrorKind::Timeout));
    assert!(matches!(err2.kind, ParseErrorKind::Timeout));
}

#[test]
fn test_parse_error_different_kinds() {
    let err1 = ParseError::no_language();
    let err2 = ParseError::timeout();
    // Different kinds
    assert!(!matches!(err1.kind, ParseErrorKind::Timeout));
    assert!(!matches!(err2.kind, ParseErrorKind::NoLanguage));
}

// ---------------------------------------------------------------------------
// 11. ParseError clone
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_clone() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 7,
    };
    let loc_clone = loc.clone();
    assert_eq!(loc, loc_clone);
}

#[test]
fn test_parse_error_kind_cloning() {
    let kind = ParseErrorKind::SyntaxError("test error".to_string());
    let kind2 = ParseErrorKind::SyntaxError("test error".to_string());
    assert_eq!(kind.to_string(), kind2.to_string());
}

// ---------------------------------------------------------------------------
// 12. Multiple errors in sequence
// ---------------------------------------------------------------------------

#[test]
fn test_collect_multiple_parse_errors() {
    let errors = [
        ParseError::no_language(),
        ParseError::timeout(),
        ParseError::with_msg("error 3"),
        ParseError::with_msg("error 4"),
    ];
    assert_eq!(errors.len(), 4);
    assert!(matches!(errors[0].kind, ParseErrorKind::NoLanguage));
    assert!(matches!(errors[1].kind, ParseErrorKind::Timeout));
}

#[test]
fn test_filter_errors_by_kind_match() {
    let errors = [
        ParseError::no_language(),
        ParseError::timeout(),
        ParseError::no_language(),
        ParseError::with_msg("x"),
    ];
    let no_lang_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e.kind, ParseErrorKind::NoLanguage))
        .collect();
    assert_eq!(no_lang_errors.len(), 2);
}

#[test]
fn test_error_sequences_with_locations() {
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
    let errors = [
        ParseError::syntax_error("error1", loc1.clone()),
        ParseError::syntax_error("error2", loc2.clone()),
    ];
    assert!(errors[0].location.is_some());
    assert!(errors[1].location.is_some());
}

// ---------------------------------------------------------------------------
// 13. Error with empty message
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_empty_message() {
    let err = ParseError::with_msg("");
    let msg = format!("{}", err);
    assert_eq!(msg, "");
}

#[test]
fn test_parse_error_syntax_error_empty_detail() {
    let err = ParseError::syntax_error(
        "",
        ErrorLocation {
            byte_offset: 0,
            line: 1,
            column: 1,
        },
    );
    assert_eq!(err.to_string(), "syntax error at ");
}

// ---------------------------------------------------------------------------
// 14. Error with very long message
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_long_message() {
    let long_msg = "x".repeat(1000);
    let err = ParseError::with_msg(&long_msg);
    assert_eq!(err.to_string(), long_msg);
}

#[test]
fn test_parse_error_syntax_error_long_detail() {
    let long_detail = "unexpected token at position: ".to_string() + &"y".repeat(500);
    let err = ParseError::syntax_error(
        &long_detail,
        ErrorLocation {
            byte_offset: 0,
            line: 1,
            column: 1,
        },
    );
    assert!(err.to_string().contains(&long_detail));
}

// ---------------------------------------------------------------------------
// 15. Error with unicode in message
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_unicode_message() {
    let err = ParseError::with_msg("Error: unexpected 你好 token");
    assert_eq!(err.to_string(), "Error: unexpected 你好 token");
}

#[test]
fn test_parse_error_unicode_emoji() {
    let err = ParseError::with_msg("Parse error 😱");
    assert_eq!(err.to_string(), "Parse error 😱");
}

#[test]
fn test_parse_error_syntax_error_unicode() {
    let err = ParseError::syntax_error(
        "syntaxfehler: ä ö ü",
        ErrorLocation {
            byte_offset: 0,
            line: 1,
            column: 1,
        },
    );
    assert!(err.to_string().contains("ä") || err.to_string().contains("ö"));
}

// ---------------------------------------------------------------------------
// 16. Error location at start of input
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_at_start() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::syntax_error("error at start", loc.clone());
    assert_eq!(err.location.as_ref().unwrap().byte_offset, 0);
    assert_eq!(err.location.as_ref().unwrap().line, 1);
    assert_eq!(err.location.as_ref().unwrap().column, 1);
}

// ---------------------------------------------------------------------------
// 17. Error location at end of input
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_at_end() {
    let loc = ErrorLocation {
        byte_offset: 10000,
        line: 500,
        column: 80,
    };
    let err = ParseError::syntax_error("error at end", loc.clone());
    assert_eq!(err.location.as_ref().unwrap().byte_offset, 10000);
    assert_eq!(err.location.as_ref().unwrap().line, 500);
    assert_eq!(err.location.as_ref().unwrap().column, 80);
}

// ---------------------------------------------------------------------------
// 18. Error location in middle of input
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_in_middle() {
    let loc = ErrorLocation {
        byte_offset: 250,
        line: 25,
        column: 15,
    };
    let err = ParseError::syntax_error("error in middle", loc.clone());
    let unwrapped = err.location.as_ref().unwrap();
    assert_eq!(unwrapped.byte_offset, 250);
    assert_eq!(unwrapped.line, 25);
    assert_eq!(unwrapped.column, 15);
}

// ---------------------------------------------------------------------------
// 19. ErrorLocation comparison/ordering
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_equality() {
    let loc1 = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 5,
    };
    let loc2 = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 5,
    };
    assert_eq!(loc1, loc2);
}

#[test]
fn test_error_location_inequality() {
    let loc1 = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 5,
    };
    let loc2 = ErrorLocation {
        byte_offset: 43,
        line: 3,
        column: 5,
    };
    assert_ne!(loc1, loc2);
}

#[test]
fn test_error_location_inequality_line_diff() {
    let loc1 = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 5,
    };
    let loc2 = ErrorLocation {
        byte_offset: 42,
        line: 4,
        column: 5,
    };
    assert_ne!(loc1, loc2);
}

#[test]
fn test_error_location_inequality_column_diff() {
    let loc1 = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 5,
    };
    let loc2 = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 6,
    };
    assert_ne!(loc1, loc2);
}

// ---------------------------------------------------------------------------
// 20. Error kind categorization / matching
// ---------------------------------------------------------------------------

#[test]
fn test_match_all_error_kinds() {
    let base_kinds: Vec<ParseErrorKind> = vec![
        ParseErrorKind::NoLanguage,
        ParseErrorKind::Timeout,
        ParseErrorKind::InvalidEncoding,
        ParseErrorKind::Cancelled,
        ParseErrorKind::VersionMismatch {
            expected: 1,
            actual: 2,
        },
        ParseErrorKind::SyntaxError("err".to_string()),
        ParseErrorKind::AllocationError,
        ParseErrorKind::Other("test".to_string()),
    ];
    #[cfg(feature = "external_scanners")]
    let kinds = {
        let mut kinds = base_kinds;
        kinds.push(ParseErrorKind::ExternalScannerError("scanner".to_string()));
        kinds
    };
    #[cfg(not(feature = "external_scanners"))]
    let kinds = base_kinds;

    let expected_count = kinds.len();
    let mut matched_count = 0;
    for kind in kinds {
        match kind {
            ParseErrorKind::NoLanguage
            | ParseErrorKind::Timeout
            | ParseErrorKind::InvalidEncoding
            | ParseErrorKind::Cancelled
            | ParseErrorKind::VersionMismatch { .. }
            | ParseErrorKind::SyntaxError(_)
            | ParseErrorKind::AllocationError
            | ParseErrorKind::Other(_) => matched_count += 1,
            #[cfg(feature = "external_scanners")]
            ParseErrorKind::ExternalScannerError(_) => matched_count += 1,
        }
    }
    assert_eq!(matched_count, expected_count);
}

// ---------------------------------------------------------------------------
// 21. Error is std::error::Error trait object
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_implements_std_error() {
    let err = ParseError::no_language();
    let _: &dyn Error = &err;
}

#[test]
fn test_parse_error_source_method() {
    let err = ParseError::timeout();
    // The #[error] derive doesn't automatically set source()
    let source = err.source();
    // source() should be None for ParseError
    assert!(source.is_none());
}

// ---------------------------------------------------------------------------
// 22. Error into Box<dyn Error> conversion
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_as_boxed_error() {
    let err = ParseError::with_msg("boxed error");
    let boxed: Box<dyn Error> = Box::new(err);
    assert_eq!(boxed.to_string(), "boxed error");
}

#[test]
fn test_parse_error_in_result_with_box() {
    fn returns_error() -> Result<(), Box<dyn Error>> {
        Err(Box::new(ParseError::no_language()))
    }
    let result = returns_error();
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 23. Error context/cause chain (if supported)
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_no_implicit_cause() {
    let err = ParseError::timeout();
    // thiserror's #[error] doesn't automatically chain sources
    let cause = err.source();
    assert!(cause.is_none());
}

#[test]
fn test_parse_error_display_in_context() {
    let err = ParseError::with_msg("root cause");
    let formatted = format!("Parsing failed: {}", err);
    assert!(formatted.contains("root cause"));
}

// ---------------------------------------------------------------------------
// 24. Error with zero-length span
// ---------------------------------------------------------------------------

#[test]
fn test_error_location_zero_byte_span() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 5,
        column: 10,
    };
    let err = ParseError::syntax_error("zero-span error", loc);
    // The location itself is not zero-width, but represents a single point
    assert_eq!(err.location.unwrap().byte_offset, 42);
}

#[test]
fn test_error_at_same_byte_and_position() {
    let loc1 = ErrorLocation {
        byte_offset: 100,
        line: 10,
        column: 5,
    };
    let loc2 = ErrorLocation {
        byte_offset: 100,
        line: 10,
        column: 5,
    };
    assert_eq!(loc1, loc2);
}

// ---------------------------------------------------------------------------
// 25. Error recovery information / additional metadata
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_version_mismatch_fields() {
    let kind = ParseErrorKind::VersionMismatch {
        expected: 15,
        actual: 12,
    };
    if let ParseErrorKind::VersionMismatch { expected, actual } = kind {
        assert_eq!(expected, 15);
        assert_eq!(actual, 12);
    } else {
        panic!("expected VersionMismatch variant");
    }
}

#[test]
fn test_parse_error_syntax_error_message_extraction() {
    let msg = "unexpected closing paren";
    let kind = ParseErrorKind::SyntaxError(msg.to_string());
    if let ParseErrorKind::SyntaxError(extracted) = kind {
        assert_eq!(extracted, msg);
    } else {
        panic!("expected SyntaxError variant");
    }
}

// ---------------------------------------------------------------------------
// Additional comprehensive tests
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_result_propagation() {
    fn may_fail() -> Result<(), ParseError> {
        Err(ParseError::with_msg("allocation failed"))
    }

    fn wrapper() -> Result<(), ParseError> {
        may_fail()?;
        Ok(())
    }

    let result = wrapper();
    assert!(result.is_err());
}

#[test]
fn test_parse_error_allocation_error_constructor() {
    let err = ParseError {
        kind: ParseErrorKind::AllocationError,
        location: None,
    };
    assert!(matches!(err.kind, ParseErrorKind::AllocationError));
}

#[test]
fn test_error_location_write_to_string_buffer() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 7,
        column: 3,
    };
    let mut buf = String::new();
    write!(&mut buf, "Error at {}", loc).unwrap();
    assert_eq!(buf, "Error at 7:3");
}

#[test]
fn test_parse_error_syntax_error_with_location() {
    let loc = ErrorLocation {
        byte_offset: 50,
        line: 5,
        column: 20,
    };
    let err = ParseError::syntax_error("detail message", loc.clone());
    assert_eq!(err.location.as_ref().unwrap(), &loc);
    assert!(err.to_string().contains("detail message"));
}

#[test]
fn test_error_kind_option_wrapping() {
    let err_no_loc = ParseError::no_language();
    let err_with_loc = ParseError::syntax_error(
        "msg",
        ErrorLocation {
            byte_offset: 0,
            line: 1,
            column: 1,
        },
    );

    assert!(err_no_loc.location.is_none());
    assert!(err_with_loc.location.is_some());
}

#[test]
fn test_multiple_errors_distinct_kinds() {
    let errors = [
        ParseError::no_language(),
        ParseError::timeout(),
        ParseError::with_msg("custom"),
    ];

    let kinds_match = errors.iter().all(|e| {
        matches!(
            e.kind,
            ParseErrorKind::NoLanguage | ParseErrorKind::Timeout | ParseErrorKind::Other(_)
        )
    });
    assert!(kinds_match);
}

#[test]
fn test_error_location_display_consistency() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 42,
        column: 17,
    };
    let display1 = format!("{}", loc);
    let display2 = format!("{}", loc);
    assert_eq!(display1, display2);
    assert_eq!(display1, "42:17");
}
