#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;
use std::error::Error;
use std::fmt::Write as _;

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_error_location() -> impl Strategy<Value = ErrorLocation> {
    (any::<usize>(), 1..usize::MAX, 1..usize::MAX).prop_map(|(byte_offset, line, column)| {
        ErrorLocation {
            byte_offset,
            line,
            column,
        }
    })
}

fn arb_nonempty_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _.:;!?]{1,128}"
}

// ---------------------------------------------------------------------------
// 1 – ParseError construction and field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn no_language_has_correct_kind_and_no_location(_ in 0..1u8) {
        let err = ParseError::no_language();
        assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
        assert!(err.location.is_none());
    }

    #[test]
    fn timeout_has_correct_kind_and_no_location(_ in 0..1u8) {
        let err = ParseError::timeout();
        assert!(matches!(err.kind, ParseErrorKind::Timeout));
        assert!(err.location.is_none());
    }

    #[test]
    fn syntax_error_stores_message_and_location(
        msg in arb_nonempty_string(),
        loc in arb_error_location(),
    ) {
        let err = ParseError::syntax_error(msg.clone(), loc.clone());
        match &err.kind {
            ParseErrorKind::SyntaxError(s) => assert_eq!(s, &msg),
            other => panic!("expected SyntaxError, got {:?}", other),
        }
        assert_eq!(err.location.as_ref(), Some(&loc));
    }

    #[test]
    fn with_msg_stores_arbitrary_string(msg in ".*") {
        let err = ParseError::with_msg(&msg);
        match &err.kind {
            ParseErrorKind::Other(s) => assert_eq!(s, &msg),
            other => panic!("expected Other, got {:?}", other),
        }
        assert!(err.location.is_none());
    }

    #[test]
    fn with_location_attaches_location(loc in arb_error_location()) {
        let err = ParseError::no_language().with_location(loc.clone());
        assert_eq!(err.location.as_ref(), Some(&loc));
    }
}

// ---------------------------------------------------------------------------
// 2 – ParseErrorKind variants – Display output
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn no_language_display(_ in 0..1u8) {
        assert_eq!(ParseErrorKind::NoLanguage.to_string(), "no language set");
    }

    #[test]
    fn timeout_display(_ in 0..1u8) {
        assert_eq!(ParseErrorKind::Timeout.to_string(), "parse timeout exceeded");
    }

    #[test]
    fn invalid_encoding_display(_ in 0..1u8) {
        assert_eq!(
            ParseErrorKind::InvalidEncoding.to_string(),
            "invalid input encoding"
        );
    }

    #[test]
    fn cancelled_display(_ in 0..1u8) {
        assert_eq!(ParseErrorKind::Cancelled.to_string(), "parse cancelled");
    }

    #[test]
    fn allocation_error_display(_ in 0..1u8) {
        assert_eq!(
            ParseErrorKind::AllocationError.to_string(),
            "memory allocation failed"
        );
    }

    #[test]
    fn version_mismatch_display(expected in any::<u32>(), actual in any::<u32>()) {
        let kind = ParseErrorKind::VersionMismatch { expected, actual };
        let display = kind.to_string();
        assert_eq!(
            display,
            format!("language version mismatch: expected {expected}, got {actual}")
        );
    }

    #[test]
    fn syntax_error_kind_display(msg in arb_nonempty_string()) {
        let kind = ParseErrorKind::SyntaxError(msg.clone());
        assert_eq!(kind.to_string(), format!("syntax error at {msg}"));
    }

    #[test]
    fn other_kind_display(msg in ".*") {
        let kind = ParseErrorKind::Other(msg.clone());
        assert_eq!(kind.to_string(), msg);
    }
}

// ---------------------------------------------------------------------------
// 3 – ErrorLocation fields
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_fields_roundtrip(
        byte_offset in any::<usize>(),
        line in any::<usize>(),
        column in any::<usize>(),
    ) {
        let loc = ErrorLocation { byte_offset, line, column };
        assert_eq!(loc.byte_offset, byte_offset);
        assert_eq!(loc.line, line);
        assert_eq!(loc.column, column);
    }
}

// ---------------------------------------------------------------------------
// 4 – Display formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_display_format(
        line in any::<usize>(),
        column in any::<usize>(),
    ) {
        let loc = ErrorLocation { byte_offset: 0, line, column };
        assert_eq!(loc.to_string(), format!("{line}:{column}"));
    }

    #[test]
    fn parse_error_display_delegates_to_kind(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        assert_eq!(err.to_string(), msg);
    }

    #[test]
    fn parse_error_display_no_language(_ in 0..1u8) {
        assert_eq!(ParseError::no_language().to_string(), "no language set");
    }

    #[test]
    fn parse_error_display_timeout(_ in 0..1u8) {
        assert_eq!(ParseError::timeout().to_string(), "parse timeout exceeded");
    }
}

// ---------------------------------------------------------------------------
// 5 – Clone / Debug / PartialEq traits (ErrorLocation)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_clone_equals_original(loc in arb_error_location()) {
        let cloned = loc.clone();
        assert_eq!(loc, cloned);
    }

    #[test]
    fn error_location_partial_eq_reflexive(loc in arb_error_location()) {
        assert_eq!(loc, loc);
    }

    #[test]
    fn error_location_partial_eq_different(
        a_off in any::<usize>(),
        b_off in any::<usize>(),
        line in 1..1000usize,
        column in 1..1000usize,
    ) {
        let a = ErrorLocation { byte_offset: a_off, line, column };
        let b = ErrorLocation { byte_offset: b_off, line, column };
        if a_off == b_off {
            assert_eq!(a, b);
        } else {
            assert_ne!(a, b);
        }
    }

    #[test]
    fn error_location_debug_contains_fields(loc in arb_error_location()) {
        let dbg = format!("{:?}", loc);
        assert!(dbg.contains("ErrorLocation"));
        assert!(dbg.contains(&loc.byte_offset.to_string()));
        assert!(dbg.contains(&loc.line.to_string()));
        assert!(dbg.contains(&loc.column.to_string()));
    }

    #[test]
    fn parse_error_debug_is_nonempty(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let dbg = format!("{:?}", err);
        assert!(!dbg.is_empty());
        assert!(dbg.contains("ParseError"));
    }

    #[test]
    fn parse_error_kind_debug_is_nonempty(expected in any::<u32>(), actual in any::<u32>()) {
        let kind = ParseErrorKind::VersionMismatch { expected, actual };
        let dbg = format!("{:?}", kind);
        assert!(dbg.contains("VersionMismatch"));
    }
}

// ---------------------------------------------------------------------------
// 6 – Error trait implementation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_is_std_error(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let _: &dyn Error = &err;
        // thiserror-generated source is None for leaf variants
        assert!(err.source().is_none());
    }

    #[test]
    fn parse_error_kind_is_std_error(_ in 0..1u8) {
        let kind = ParseErrorKind::NoLanguage;
        let _: &dyn Error = &kind;
        assert!(kind.source().is_none());
    }
}

// ---------------------------------------------------------------------------
// 7 – Collection behaviour
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_locations_in_vec(
        count in 1..50usize,
        byte_offset in any::<usize>(),
        line in 1..10000usize,
        column in 1..10000usize,
    ) {
        let loc = ErrorLocation { byte_offset, line, column };
        let v: Vec<ErrorLocation> = vec![loc.clone(); count];
        assert_eq!(v.len(), count);
        for i in 0..v.len() {
            assert_eq!(v[i], loc);
        }
    }

    #[test]
    fn parse_errors_in_vec(count in 1..50usize) {
        let mut v: Vec<ParseError> = Vec::with_capacity(count);
        for i in 0..count {
            v.push(ParseError::with_msg(&format!("err-{i}")));
        }
        assert_eq!(v.len(), count);
        for i in 0..count {
            assert_eq!(v[i].to_string(), format!("err-{i}"));
        }
    }

    #[test]
    fn error_locations_dedup(
        line in 1..1000usize,
        column in 1..1000usize,
    ) {
        let loc = ErrorLocation { byte_offset: 0, line, column };
        let mut v = vec![loc.clone(), loc.clone(), loc.clone()];
        v.dedup();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], loc);
    }
}

// ---------------------------------------------------------------------------
// 8 – Edge cases
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn empty_message_syntax_error(loc in arb_error_location()) {
        let err = ParseError::syntax_error("", loc);
        assert_eq!(err.to_string(), "syntax error at ");
    }

    #[test]
    fn empty_message_other_kind(_ in 0..1u8) {
        let err = ParseError::with_msg("");
        assert_eq!(err.to_string(), "");
    }

    #[test]
    fn large_offset_error_location(
        byte_offset in (usize::MAX - 100)..=usize::MAX,
        line in (usize::MAX - 100)..=usize::MAX,
        column in (usize::MAX - 100)..=usize::MAX,
    ) {
        let loc = ErrorLocation { byte_offset, line, column };
        let cloned = loc.clone();
        assert_eq!(loc, cloned);
        // Display must not panic
        let _ = loc.to_string();
    }

    #[test]
    fn version_mismatch_boundary_values(
        expected in prop_oneof![Just(0u32), Just(u32::MAX), any::<u32>()],
        actual in prop_oneof![Just(0u32), Just(u32::MAX), any::<u32>()],
    ) {
        let kind = ParseErrorKind::VersionMismatch { expected, actual };
        let display = kind.to_string();
        assert!(display.contains(&expected.to_string()));
        assert!(display.contains(&actual.to_string()));
    }

    #[test]
    fn with_location_replaces_existing(
        loc1 in arb_error_location(),
        loc2 in arb_error_location(),
    ) {
        let err = ParseError::syntax_error("x", loc1)
            .with_location(loc2.clone());
        assert_eq!(err.location.as_ref(), Some(&loc2));
    }

    #[test]
    fn display_write_to_buffer(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let mut buf = String::new();
        write!(buf, "{err}").unwrap();
        assert_eq!(buf, msg);
    }
}
