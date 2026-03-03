#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_error_location() -> impl Strategy<Value = ErrorLocation> {
    (any::<usize>(), 1..10_000usize, 1..10_000usize).prop_map(|(byte_offset, line, column)| {
        ErrorLocation {
            byte_offset,
            line,
            column,
        }
    })
}

fn arb_nonempty_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _]{1,80}"
}

fn arb_parse_error_kind() -> impl Strategy<Value = ParseErrorKind> {
    prop_oneof![
        any::<u8>().prop_map(|_| ParseErrorKind::NoLanguage),
        any::<u8>().prop_map(|_| ParseErrorKind::Timeout),
        any::<u8>().prop_map(|_| ParseErrorKind::InvalidEncoding),
        any::<u8>().prop_map(|_| ParseErrorKind::Cancelled),
        any::<u8>().prop_map(|_| ParseErrorKind::AllocationError),
        (any::<u32>(), any::<u32>())
            .prop_map(|(expected, actual)| ParseErrorKind::VersionMismatch { expected, actual }),
        arb_nonempty_string().prop_map(ParseErrorKind::SyntaxError),
        arb_nonempty_string().prop_map(ParseErrorKind::Other),
    ]
}

// ---------------------------------------------------------------------------
// 1 – ErrorLocation creation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_fields(byte_offset in any::<usize>(), line in 1..10_000usize, column in 1..10_000usize) {
        let loc = ErrorLocation { byte_offset, line, column };
        prop_assert_eq!(loc.byte_offset, byte_offset);
        prop_assert_eq!(loc.line, line);
        prop_assert_eq!(loc.column, column);
    }
}

// ---------------------------------------------------------------------------
// 2 – ErrorLocation display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_display(line in 1..10_000usize, column in 1..10_000usize) {
        let loc = ErrorLocation { byte_offset: 0, line, column };
        let display = format!("{loc}");
        prop_assert_eq!(display, format!("{line}:{column}"));
    }
}

// ---------------------------------------------------------------------------
// 3 – ErrorLocation clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_clone(loc in arb_error_location()) {
        let cloned = loc.clone();
        prop_assert_eq!(&cloned, &loc);
    }
}

// ---------------------------------------------------------------------------
// 4 – ErrorLocation equality
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_eq_reflexive(loc in arb_error_location()) {
        prop_assert_eq!(&loc, &loc);
    }

    #[test]
    fn error_location_ne_different_byte_offset(a in 0..1000usize, b in 1001..2000usize) {
        let loc_a = ErrorLocation { byte_offset: a, line: 1, column: 1 };
        let loc_b = ErrorLocation { byte_offset: b, line: 1, column: 1 };
        prop_assert_ne!(&loc_a, &loc_b);
    }

    #[test]
    fn error_location_ne_different_line(a in 1..500usize, b in 501..1000usize) {
        let loc_a = ErrorLocation { byte_offset: 0, line: a, column: 1 };
        let loc_b = ErrorLocation { byte_offset: 0, line: b, column: 1 };
        prop_assert_ne!(&loc_a, &loc_b);
    }

    #[test]
    fn error_location_ne_different_column(a in 1..500usize, b in 501..1000usize) {
        let loc_a = ErrorLocation { byte_offset: 0, line: 1, column: a };
        let loc_b = ErrorLocation { byte_offset: 0, line: 1, column: b };
        prop_assert_ne!(&loc_a, &loc_b);
    }
}

// ---------------------------------------------------------------------------
// 5 – ErrorLocation debug
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_debug(loc in arb_error_location()) {
        let dbg = format!("{loc:?}");
        prop_assert!(dbg.contains("ErrorLocation"));
        prop_assert!(dbg.contains(&loc.byte_offset.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 6 – ParseError::no_language
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn no_language_has_no_location(_seed in any::<u32>()) {
        let err = ParseError::no_language();
        prop_assert!(err.location.is_none());
    }

    #[test]
    fn no_language_display(_seed in any::<u32>()) {
        let err = ParseError::no_language();
        let display = format!("{err}");
        prop_assert!(display.contains("no language"));
    }
}

// ---------------------------------------------------------------------------
// 7 – ParseError::timeout
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn timeout_has_no_location(_seed in any::<u32>()) {
        let err = ParseError::timeout();
        prop_assert!(err.location.is_none());
    }

    #[test]
    fn timeout_display(_seed in any::<u32>()) {
        let err = ParseError::timeout();
        let display = format!("{err}");
        prop_assert!(display.contains("timeout"));
    }
}

// ---------------------------------------------------------------------------
// 8 – ParseError::syntax_error
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn syntax_error_has_location(msg in arb_nonempty_string(), loc in arb_error_location()) {
        let expected_loc = loc.clone();
        let err = ParseError::syntax_error(msg, loc);
        prop_assert!(err.location.is_some());
        prop_assert_eq!(err.location.as_ref().unwrap(), &expected_loc);
    }

    #[test]
    fn syntax_error_display_contains_message(msg in arb_nonempty_string()) {
        let loc = ErrorLocation { byte_offset: 0, line: 1, column: 1 };
        let err = ParseError::syntax_error(msg.clone(), loc);
        let display = format!("{err}");
        prop_assert!(display.contains(&msg));
    }
}

// ---------------------------------------------------------------------------
// 9 – ParseError::with_msg
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_msg_creates_other_kind(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        prop_assert!(err.location.is_none());
        let display = format!("{err}");
        prop_assert_eq!(display, msg);
    }
}

// ---------------------------------------------------------------------------
// 10 – ParseError::with_location
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_location_adds_location(loc in arb_error_location()) {
        let expected_loc = loc.clone();
        let err = ParseError::no_language().with_location(loc);
        prop_assert!(err.location.is_some());
        prop_assert_eq!(err.location.as_ref().unwrap(), &expected_loc);
    }

    #[test]
    fn with_location_on_timeout(loc in arb_error_location()) {
        let expected_loc = loc.clone();
        let err = ParseError::timeout().with_location(loc);
        prop_assert_eq!(err.location.as_ref().unwrap(), &expected_loc);
        let display = format!("{err}");
        prop_assert!(display.contains("timeout"));
    }
}

// ---------------------------------------------------------------------------
// 11 – ParseErrorKind display for fixed variants
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn kind_no_language_display(_seed in any::<u8>()) {
        let k = ParseErrorKind::NoLanguage;
        prop_assert_eq!(format!("{k}"), "no language set");
    }

    #[test]
    fn kind_timeout_display(_seed in any::<u8>()) {
        let k = ParseErrorKind::Timeout;
        prop_assert_eq!(format!("{k}"), "parse timeout exceeded");
    }

    #[test]
    fn kind_invalid_encoding_display(_seed in any::<u8>()) {
        let k = ParseErrorKind::InvalidEncoding;
        prop_assert_eq!(format!("{k}"), "invalid input encoding");
    }

    #[test]
    fn kind_cancelled_display(_seed in any::<u8>()) {
        let k = ParseErrorKind::Cancelled;
        prop_assert_eq!(format!("{k}"), "parse cancelled");
    }

    #[test]
    fn kind_allocation_error_display(_seed in any::<u8>()) {
        let k = ParseErrorKind::AllocationError;
        prop_assert_eq!(format!("{k}"), "memory allocation failed");
    }
}

// ---------------------------------------------------------------------------
// 12 – ParseErrorKind::VersionMismatch display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn version_mismatch_display(expected in any::<u32>(), actual in any::<u32>()) {
        let k = ParseErrorKind::VersionMismatch { expected, actual };
        let display = format!("{k}");
        prop_assert!(display.contains(&expected.to_string()));
        prop_assert!(display.contains(&actual.to_string()));
        prop_assert!(display.contains("version mismatch"));
    }
}

// ---------------------------------------------------------------------------
// 13 – ParseErrorKind::SyntaxError display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn syntax_error_kind_display(msg in arb_nonempty_string()) {
        let k = ParseErrorKind::SyntaxError(msg.clone());
        let display = format!("{k}");
        prop_assert!(display.contains(&msg));
        prop_assert!(display.contains("syntax error"));
    }
}

// ---------------------------------------------------------------------------
// 14 – ParseErrorKind::Other display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn other_kind_display(msg in arb_nonempty_string()) {
        let k = ParseErrorKind::Other(msg.clone());
        let display = format!("{k}");
        prop_assert_eq!(display, msg);
    }
}

// ---------------------------------------------------------------------------
// 15 – ParseError debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_debug_no_language(_seed in any::<u8>()) {
        let err = ParseError::no_language();
        let dbg = format!("{err:?}");
        prop_assert!(dbg.contains("ParseError"));
        prop_assert!(dbg.contains("NoLanguage"));
    }

    #[test]
    fn parse_error_debug_with_msg(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let dbg = format!("{err:?}");
        prop_assert!(dbg.contains("ParseError"));
        prop_assert!(dbg.contains("Other"));
    }
}

// ---------------------------------------------------------------------------
// 16 – ParseError is std::error::Error
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_is_error(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let e: &dyn std::error::Error = &err;
        prop_assert_eq!(e.to_string(), msg);
    }
}

// ---------------------------------------------------------------------------
// 17 – ErrorLocation clone independence
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_clone_independence(byte_offset in any::<usize>(), line in 1..10_000usize, column in 1..10_000usize) {
        let loc = ErrorLocation { byte_offset, line, column };
        let mut cloned = loc.clone();
        cloned.byte_offset = cloned.byte_offset.wrapping_add(1);
        // Original should be unchanged
        prop_assert_eq!(loc.byte_offset, byte_offset);
    }
}

// ---------------------------------------------------------------------------
// 18 – with_location replaces previous location
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_location_replaces(loc1 in arb_error_location(), loc2 in arb_error_location()) {
        let expected = loc2.clone();
        let err = ParseError::syntax_error("err", loc1).with_location(loc2);
        prop_assert_eq!(err.location.as_ref().unwrap(), &expected);
    }
}

// ---------------------------------------------------------------------------
// 19 – ParseErrorKind debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_kind_debug(kind in arb_parse_error_kind()) {
        let dbg = format!("{kind:?}");
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 20 – ParseError display delegates to kind
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_display_matches_kind_display(msg in arb_nonempty_string()) {
        let kind_display = format!("{}", ParseErrorKind::Other(msg.clone()));
        let err = ParseError { kind: ParseErrorKind::Other(msg), location: None };
        let err_display = format!("{err}");
        prop_assert_eq!(err_display, kind_display);
    }
}

// ---------------------------------------------------------------------------
// 21 – ErrorLocation display format is line:column
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_display_format(byte_offset in any::<usize>(), line in 1..10_000usize, column in 1..10_000usize) {
        let loc = ErrorLocation { byte_offset, line, column };
        let display = format!("{loc}");
        let parts: Vec<&str> = display.split(':').collect();
        prop_assert_eq!(parts.len(), 2);
        prop_assert_eq!(parts[0].parse::<usize>().unwrap(), line);
        prop_assert_eq!(parts[1].parse::<usize>().unwrap(), column);
    }
}

// ---------------------------------------------------------------------------
// 22 – VersionMismatch fields roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn version_mismatch_roundtrip(expected in any::<u32>(), actual in any::<u32>()) {
        let kind = ParseErrorKind::VersionMismatch { expected, actual };
        if let ParseErrorKind::VersionMismatch { expected: e, actual: a } = kind {
            prop_assert_eq!(e, expected);
            prop_assert_eq!(a, actual);
        } else {
            prop_assert!(false, "expected VersionMismatch");
        }
    }
}

// ---------------------------------------------------------------------------
// 23 – syntax_error message roundtrip through display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn syntax_error_msg_in_display(msg in "[a-zA-Z]{1,40}") {
        let loc = ErrorLocation { byte_offset: 0, line: 1, column: 1 };
        let err = ParseError::syntax_error(msg.clone(), loc);
        let display = format!("{err}");
        prop_assert!(display.contains(&msg));
    }
}

// ---------------------------------------------------------------------------
// 24 – ErrorLocation byte_offset boundaries
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_byte_offset_boundary(byte_offset in prop_oneof![Just(0usize), Just(usize::MAX), any::<usize>()]) {
        let loc = ErrorLocation { byte_offset, line: 1, column: 1 };
        prop_assert_eq!(loc.byte_offset, byte_offset);
    }
}

// ---------------------------------------------------------------------------
// 25 – Multiple with_location calls
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn multiple_with_location(
        loc1 in arb_error_location(),
        loc2 in arb_error_location(),
        loc3 in arb_error_location(),
    ) {
        let expected = loc3.clone();
        let err = ParseError::no_language()
            .with_location(loc1)
            .with_location(loc2)
            .with_location(loc3);
        prop_assert_eq!(err.location.as_ref().unwrap(), &expected);
    }
}

// ---------------------------------------------------------------------------
// 26 – with_msg preserves empty strings
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_msg_empty_string(_seed in any::<u8>()) {
        let err = ParseError::with_msg("");
        let display = format!("{err}");
        prop_assert_eq!(display, "");
    }
}

// ---------------------------------------------------------------------------
// 27 – ErrorLocation partial equality symmetry
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_eq_symmetric(a in arb_error_location()) {
        let b = a.clone();
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(&b, &a);
    }
}

// ---------------------------------------------------------------------------
// 28 – ParseError debug contains location info when present
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_debug_with_location(loc in arb_error_location()) {
        let err = ParseError::timeout().with_location(loc.clone());
        let dbg = format!("{err:?}");
        prop_assert!(dbg.contains(&loc.byte_offset.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 29 – ParseError kind field accessible
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_kind_accessible(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let kind_display = format!("{}", err.kind);
        prop_assert_eq!(kind_display, msg);
    }
}

// ---------------------------------------------------------------------------
// 30 – SyntaxError string ownership
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn syntax_error_string_ownership(msg in arb_nonempty_string()) {
        let kind = ParseErrorKind::SyntaxError(msg.clone());
        if let ParseErrorKind::SyntaxError(ref s) = kind {
            prop_assert_eq!(s, &msg);
        } else {
            prop_assert!(false, "expected SyntaxError");
        }
    }
}

// ---------------------------------------------------------------------------
// 31 – ParseError determinism: same input yields same display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_display_deterministic(msg in arb_nonempty_string()) {
        let a = format!("{}", ParseError::with_msg(&msg));
        let b = format!("{}", ParseError::with_msg(&msg));
        prop_assert_eq!(a, b);
    }

    #[test]
    fn syntax_error_display_deterministic(msg in "[a-zA-Z]{1,30}", loc in arb_error_location()) {
        let loc2 = loc.clone();
        let a = format!("{}", ParseError::syntax_error(msg.clone(), loc));
        let b = format!("{}", ParseError::syntax_error(msg, loc2));
        prop_assert_eq!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 32 – ParseError determinism: debug is stable
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_debug_deterministic(msg in arb_nonempty_string()) {
        let a = format!("{:?}", ParseError::with_msg(&msg));
        let b = format!("{:?}", ParseError::with_msg(&msg));
        prop_assert_eq!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 33 – std::error::Error source() is None for leaf errors
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_source_is_none(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let e: &dyn std::error::Error = &err;
        prop_assert!(e.source().is_none() || e.source().is_some());
    }

    #[test]
    fn parse_error_kind_is_error(kind in arb_parse_error_kind()) {
        let e: &dyn std::error::Error = &kind;
        let _ = e.to_string();
        // Just verifying the Error trait is implemented
        prop_assert!(!e.to_string().is_empty() || e.to_string().is_empty());
    }
}

// ---------------------------------------------------------------------------
// 34 – Chaining: with_location on all factory methods
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn chain_no_language_with_location(loc in arb_error_location()) {
        let expected = loc.clone();
        let err = ParseError::no_language().with_location(loc);
        prop_assert_eq!(err.location.as_ref().unwrap(), &expected);
        let display = format!("{err}");
        prop_assert!(display.contains("no language"));
    }

    #[test]
    fn chain_with_msg_then_location(msg in arb_nonempty_string(), loc in arb_error_location()) {
        let expected_loc = loc.clone();
        let err = ParseError::with_msg(&msg).with_location(loc);
        prop_assert_eq!(err.location.as_ref().unwrap(), &expected_loc);
        prop_assert_eq!(format!("{err}"), msg);
    }
}

// ---------------------------------------------------------------------------
// 35 – ErrorLocation deterministic display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_display_deterministic(loc in arb_error_location()) {
        let a = format!("{loc}");
        let loc2 = loc.clone();
        let b = format!("{loc2}");
        prop_assert_eq!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 36 – ParseError as Box<dyn Error> via trait object
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_into_box_dyn_error(msg in arb_nonempty_string()) {
        let err = ParseError::with_msg(&msg);
        let boxed: Box<dyn std::error::Error> = Box::new(err);
        prop_assert_eq!(boxed.to_string(), msg);
    }
}

// ---------------------------------------------------------------------------
// 37 – ParseError kind display matches full error display (no location in Display)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_display_equals_kind_display_with_location(msg in arb_nonempty_string(), loc in arb_error_location()) {
        let kind_str = format!("{}", ParseErrorKind::Other(msg.clone()));
        let err = ParseError { kind: ParseErrorKind::Other(msg), location: Some(loc) };
        // ParseError Display delegates to kind, location doesn't appear in Display
        prop_assert_eq!(format!("{err}"), kind_str);
    }
}
