#![allow(clippy::needless_range_loop)]

//! Property-based tests for error types in adze-runtime (runtime2/).

use proptest::prelude::*;

use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_location() -> impl Strategy<Value = ErrorLocation> {
    (any::<usize>(), any::<usize>(), any::<usize>()).prop_map(|(b, l, c)| ErrorLocation {
        byte_offset: b,
        line: l,
        column: c,
    })
}

fn arb_ascii_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _]{1,64}"
}

// ---------------------------------------------------------------------------
// 1 – syntax_error accepts owned String via Into<String>
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn syntax_error_accepts_owned_string(msg in arb_ascii_string()) {
        let owned: String = msg.clone();
        let loc = ErrorLocation { byte_offset: 0, line: 1, column: 1 };
        let err = ParseError::syntax_error(owned, loc);
        match &err.kind {
            ParseErrorKind::SyntaxError(s) => prop_assert_eq!(s, &msg),
            other => prop_assert!(false, "expected SyntaxError, got {:?}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// 2 – with_location preserves kind unchanged
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_location_preserves_no_language_kind(loc in arb_location()) {
        let err = ParseError::no_language().with_location(loc);
        prop_assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    }

    #[test]
    fn with_location_preserves_timeout_kind(loc in arb_location()) {
        let err = ParseError::timeout().with_location(loc);
        prop_assert!(matches!(err.kind, ParseErrorKind::Timeout));
    }

    #[test]
    fn with_location_preserves_with_msg_kind(msg in arb_ascii_string(), loc in arb_location()) {
        let err = ParseError::with_msg(&msg).with_location(loc);
        match &err.kind {
            ParseErrorKind::Other(s) => prop_assert_eq!(s, &msg),
            other => prop_assert!(false, "expected Other, got {:?}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// 3 – ErrorLocation Eq is transitive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_eq_transitive(loc in arb_location()) {
        let a = loc.clone();
        let b = a.clone();
        let c = b.clone();
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(&b, &c);
        prop_assert_eq!(&a, &c);
    }
}

// ---------------------------------------------------------------------------
// 4 – ParseError can be used as Box<dyn Error>
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_into_box_dyn_error(msg in arb_ascii_string()) {
        let err = ParseError::with_msg(&msg);
        let boxed: Box<dyn std::error::Error> = Box::new(err);
        prop_assert_eq!(boxed.to_string(), msg);
    }
}

// ---------------------------------------------------------------------------
// 5 – no_language and timeout are distinguishable via Display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn no_language_and_timeout_differ_in_display(_ in 0..1u8) {
        let a = ParseError::no_language().to_string();
        let b = ParseError::timeout().to_string();
        prop_assert_ne!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 6 – with_msg then with_location produces error with both
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_msg_then_with_location(msg in arb_ascii_string(), loc in arb_location()) {
        let expected_loc = loc.clone();
        let err = ParseError::with_msg(&msg).with_location(loc);
        prop_assert_eq!(err.to_string(), msg);
        prop_assert_eq!(err.location.as_ref(), Some(&expected_loc));
    }
}

// ---------------------------------------------------------------------------
// 7 – ParseError Display length equals kind Display length
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_display_len_matches_kind(msg in arb_ascii_string()) {
        let kind_display = ParseErrorKind::Other(msg.clone()).to_string();
        let err_display = ParseError::with_msg(&msg).to_string();
        prop_assert_eq!(err_display.len(), kind_display.len());
    }
}

// ---------------------------------------------------------------------------
// 8 – ParseErrorKind Display is deterministic
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn kind_display_deterministic(msg in arb_ascii_string()) {
        let k1 = ParseErrorKind::SyntaxError(msg.clone());
        let k2 = ParseErrorKind::SyntaxError(msg);
        prop_assert_eq!(k1.to_string(), k2.to_string());
    }
}

// ---------------------------------------------------------------------------
// 9 – with_location idempotent with same location
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_location_idempotent(loc in arb_location()) {
        let loc2 = loc.clone();
        let err1 = ParseError::no_language().with_location(loc.clone());
        let err2 = ParseError::no_language().with_location(loc2);
        prop_assert_eq!(err1.location, err2.location);
    }
}

// ---------------------------------------------------------------------------
// 10 – syntax_error message preserved after with_location override
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn syntax_error_msg_preserved_after_with_location(
        msg in arb_ascii_string(),
        loc1 in arb_location(),
        loc2 in arb_location(),
    ) {
        let err = ParseError::syntax_error(msg.clone(), loc1).with_location(loc2);
        let display = err.to_string();
        prop_assert!(display.contains(&msg));
    }
}

// ---------------------------------------------------------------------------
// 11 – ParseError Debug always contains "ParseError"
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_always_contains_struct_name(msg in arb_ascii_string()) {
        let variants: Vec<ParseError> = vec![
            ParseError::no_language(),
            ParseError::timeout(),
            ParseError::with_msg(&msg),
        ];
        for err in &variants {
            let dbg = format!("{:?}", err);
            prop_assert!(dbg.contains("ParseError"), "missing ParseError in: {}", dbg);
        }
    }
}

// ---------------------------------------------------------------------------
// 12 – ErrorLocation display uses exactly one colon separator
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_display_single_colon(line in 1..10_000usize, column in 1..10_000usize) {
        let loc = ErrorLocation { byte_offset: 0, line, column };
        let display = loc.to_string();
        let colon_count = display.chars().filter(|&c| c == ':').count();
        prop_assert_eq!(colon_count, 1);
    }
}

// ---------------------------------------------------------------------------
// 13 – Multiple ParseError instances are independent
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn multiple_errors_independent(msg1 in arb_ascii_string(), msg2 in arb_ascii_string()) {
        let e1 = ParseError::with_msg(&msg1);
        let e2 = ParseError::with_msg(&msg2);
        prop_assert_eq!(e1.to_string(), msg1);
        prop_assert_eq!(e2.to_string(), msg2);
    }
}

// ---------------------------------------------------------------------------
// 14 – ParseError::with_msg with whitespace-only strings
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_msg_whitespace(n in 1..20usize) {
        let spaces = " ".repeat(n);
        let err = ParseError::with_msg(&spaces);
        prop_assert_eq!(err.to_string(), spaces);
    }
}

// ---------------------------------------------------------------------------
// 15 – ParseErrorKind::Other with newlines
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn other_kind_with_newlines(prefix in arb_ascii_string(), suffix in arb_ascii_string()) {
        let msg = format!("{}\n{}", prefix, suffix);
        let kind = ParseErrorKind::Other(msg.clone());
        prop_assert_eq!(kind.to_string(), msg);
    }
}

// ---------------------------------------------------------------------------
// 16 – VersionMismatch display through ParseError
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn version_mismatch_through_parse_error(expected in any::<u32>(), actual in any::<u32>()) {
        let err = ParseError {
            kind: ParseErrorKind::VersionMismatch { expected, actual },
            location: None,
        };
        let display = err.to_string();
        prop_assert_eq!(
            display,
            format!("language version mismatch: expected {expected}, got {actual}")
        );
    }
}

// ---------------------------------------------------------------------------
// 17 – ParseError Debug changes when location added
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_differs_with_and_without_location(loc in arb_location()) {
        let without = format!("{:?}", ParseError::timeout());
        let with = format!("{:?}", ParseError::timeout().with_location(loc));
        prop_assert_ne!(without, with);
    }
}

// ---------------------------------------------------------------------------
// 18 – ErrorLocation clone then mutate shows independence
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_clone_mutate_independent(
        byte_offset in any::<usize>(),
        line in 1..10_000usize,
        column in 1..10_000usize,
    ) {
        let original = ErrorLocation { byte_offset, line, column };
        let mut cloned = original.clone();
        cloned.line = cloned.line.wrapping_add(1);
        cloned.column = cloned.column.wrapping_add(1);
        prop_assert_ne!(cloned.line, line);
        prop_assert_ne!(cloned.column, column);
        prop_assert_eq!(original.byte_offset, byte_offset);
        prop_assert_eq!(original.line, line);
        prop_assert_eq!(original.column, column);
    }
}

// ---------------------------------------------------------------------------
// 19 – Result<(), ParseError> with ? operator
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn result_question_mark_propagation(msg in arb_ascii_string()) {
        fn fallible(m: &str) -> Result<(), ParseError> {
            Err(ParseError::with_msg(m))
        }
        fn wrapper(m: &str) -> Result<(), ParseError> {
            fallible(m)?;
            Ok(())
        }
        let res = wrapper(&msg);
        prop_assert!(res.is_err());
        prop_assert_eq!(res.unwrap_err().to_string(), msg);
    }
}

// ---------------------------------------------------------------------------
// 20 – Result::map_err with ParseError::with_msg
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn map_err_with_parse_error(msg in arb_ascii_string()) {
        let result: Result<(), &str> = Err(&*msg);
        let mapped: Result<(), ParseError> = result.map_err(ParseError::with_msg);
        prop_assert!(mapped.is_err());
        prop_assert_eq!(mapped.unwrap_err().to_string(), msg);
    }
}

// ---------------------------------------------------------------------------
// 21 – All fixed ParseErrorKind variants have non-empty Display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_fixed_variants_nonempty_display(_ in 0..1u8) {
        let kinds: Vec<ParseErrorKind> = vec![
            ParseErrorKind::NoLanguage,
            ParseErrorKind::Timeout,
            ParseErrorKind::InvalidEncoding,
            ParseErrorKind::Cancelled,
            ParseErrorKind::AllocationError,
        ];
        for k in &kinds {
            prop_assert!(!k.to_string().is_empty(), "empty display for {:?}", k);
        }
    }
}

// ---------------------------------------------------------------------------
// 22 – ParseErrorKind source() is None for all leaf variants
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_kind_variants_source_is_none(msg in arb_ascii_string()) {
        use std::error::Error;
        let kinds: Vec<ParseErrorKind> = vec![
            ParseErrorKind::NoLanguage,
            ParseErrorKind::Timeout,
            ParseErrorKind::InvalidEncoding,
            ParseErrorKind::Cancelled,
            ParseErrorKind::AllocationError,
            ParseErrorKind::VersionMismatch { expected: 1, actual: 2 },
            ParseErrorKind::SyntaxError(msg.clone()),
            ParseErrorKind::Other(msg),
        ];
        for k in &kinds {
            prop_assert!(k.source().is_none(), "non-None source for {:?}", k);
        }
    }
}

// ---------------------------------------------------------------------------
// 23 – ParseError location field is None by default for constructors
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn constructors_without_location_have_none(msg in arb_ascii_string()) {
        let errors = vec![
            ParseError::no_language(),
            ParseError::timeout(),
            ParseError::with_msg(&msg),
        ];
        for err in &errors {
            prop_assert!(err.location.is_none());
        }
    }
}

// ---------------------------------------------------------------------------
// 24 – syntax_error always has Some(location)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn syntax_error_always_has_location(msg in arb_ascii_string(), loc in arb_location()) {
        let err = ParseError::syntax_error(msg, loc);
        prop_assert!(err.location.is_some());
    }
}

// ---------------------------------------------------------------------------
// 25 – ErrorLocation zero fields are valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_zero_fields(_ in 0..1u8) {
        let loc = ErrorLocation { byte_offset: 0, line: 0, column: 0 };
        prop_assert_eq!(loc.to_string(), "0:0");
        prop_assert_eq!(loc.byte_offset, 0);
    }
}

// ---------------------------------------------------------------------------
// 26 – ParseError struct fields are publicly accessible
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_error_fields_accessible(msg in arb_ascii_string(), loc in arb_location()) {
        let expected_loc = loc.clone();
        let err = ParseError::syntax_error(msg.clone(), loc);
        // Access kind field
        let _kind_str = err.kind.to_string();
        // Access location field
        let loc_ref = err.location.as_ref().unwrap();
        prop_assert_eq!(loc_ref, &expected_loc);
    }
}

// ---------------------------------------------------------------------------
// 27 – ErrorLocation inequality when any field differs
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_location_ne_when_line_differs(
        byte_offset in any::<usize>(),
        col in any::<usize>(),
        l1 in 0..1000usize,
        l2 in 1001..2000usize,
    ) {
        let a = ErrorLocation { byte_offset, line: l1, column: col };
        let b = ErrorLocation { byte_offset, line: l2, column: col };
        prop_assert_ne!(a, b);
    }

    #[test]
    fn error_location_ne_when_column_differs(
        byte_offset in any::<usize>(),
        line in any::<usize>(),
        c1 in 0..1000usize,
        c2 in 1001..2000usize,
    ) {
        let a = ErrorLocation { byte_offset, line, column: c1 };
        let b = ErrorLocation { byte_offset, line, column: c2 };
        prop_assert_ne!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 28 – InvalidEncoding and Cancelled through ParseError wrapper
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn invalid_encoding_through_parse_error(_ in 0..1u8) {
        let err = ParseError {
            kind: ParseErrorKind::InvalidEncoding,
            location: None,
        };
        prop_assert_eq!(err.to_string(), "invalid input encoding");
    }

    #[test]
    fn cancelled_through_parse_error(_ in 0..1u8) {
        let err = ParseError {
            kind: ParseErrorKind::Cancelled,
            location: None,
        };
        prop_assert_eq!(err.to_string(), "parse cancelled");
    }

    #[test]
    fn allocation_error_through_parse_error(_ in 0..1u8) {
        let err = ParseError {
            kind: ParseErrorKind::AllocationError,
            location: None,
        };
        prop_assert_eq!(err.to_string(), "memory allocation failed");
    }
}

// ---------------------------------------------------------------------------
// 29 – ParseError direct construction with all kinds
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn direct_construction_with_location(
        loc in arb_location(),
        expected in any::<u32>(),
        actual in any::<u32>(),
    ) {
        let expected_loc = loc.clone();
        let err = ParseError {
            kind: ParseErrorKind::VersionMismatch { expected, actual },
            location: Some(loc),
        };
        prop_assert!(err.to_string().contains("version mismatch"));
        prop_assert_eq!(err.location.as_ref(), Some(&expected_loc));
    }
}

// ---------------------------------------------------------------------------
// 30 – Chaining three with_location calls keeps only the last
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn triple_with_location_keeps_last(
        l1 in arb_location(),
        l2 in arb_location(),
        l3 in arb_location(),
    ) {
        let expected = l3.clone();
        let err = ParseError::with_msg("chain")
            .with_location(l1)
            .with_location(l2)
            .with_location(l3);
        prop_assert_eq!(err.location.as_ref(), Some(&expected));
        prop_assert_eq!(err.to_string(), "chain");
    }
}
