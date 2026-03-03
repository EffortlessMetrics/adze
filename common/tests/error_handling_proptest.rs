#![allow(clippy::needless_range_loop)]

//! Property-based tests for error handling in adze-common.
//!
//! Tests syn::Error properties produced by parsing failures in NameValueExpr
//! and FieldThenParams, including error creation, display formatting, debug
//! output, clone, equality, from conversions, useful messages, and source context.

use adze_common::{FieldThenParams, NameValueExpr};
use proptest::prelude::*;
use std::error::Error as StdError;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers (lowercase start).
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Simple leaf type names.
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Integer literal values.
fn int_value() -> impl Strategy<Value = i64> {
    -1000i64..1000
}

/// Strings that are definitely not valid Rust identifiers.
fn invalid_ident_start() -> impl Strategy<Value = String> {
    prop::sample::select(
        &[
            "123abc", "42", "0x1F", "-foo", "+bar", "!bang", "3_trail", "999", "0", "00abc",
        ][..],
    )
    .prop_map(|s| s.to_string())
}

/// Strings that are definitely not valid as NameValueExpr.
fn garbage_input() -> impl Strategy<Value = String> {
    prop::sample::select(
        &[
            "", "   ", "=", "= =", "===", ":::", ",,", ";;", "{{}", "()", "[]", "->", "<>", "/**/",
            "//",
        ][..],
    )
    .prop_map(|s| s.to_string())
}

/// Incomplete NVE inputs (ident but missing value after `=`).
fn incomplete_nve() -> impl Strategy<Value = String> {
    ident_strategy().prop_map(|name| format!("{name} ="))
}

// ===========================================================================
// 1. Error types creation — syn::Error from parse failures
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 1. Parsing empty string as NVE produces an error.
    #[test]
    fn nve_empty_input_produces_error(_dummy in 0..1u8) {
        let result = syn::parse_str::<NameValueExpr>("");
        prop_assert!(result.is_err());
    }

    /// 2. Parsing empty string as FTP produces an error.
    #[test]
    fn ftp_empty_input_produces_error(_dummy in 0..1u8) {
        let result = syn::parse_str::<FieldThenParams>("");
        prop_assert!(result.is_err());
    }

    /// 3. Parsing invalid identifier start as NVE produces an error.
    #[test]
    fn nve_invalid_ident_produces_error(bad in invalid_ident_start(), val in int_value()) {
        let src = format!("{bad} = {val}");
        let result = syn::parse_str::<NameValueExpr>(&src);
        prop_assert!(result.is_err());
    }

    /// 4. Parsing NVE with missing equals sign produces an error.
    #[test]
    fn nve_missing_equals_produces_error(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} {val}");
        let result = syn::parse_str::<NameValueExpr>(&src);
        prop_assert!(result.is_err());
    }

    /// 5. Parsing NVE with incomplete value produces an error.
    #[test]
    fn nve_incomplete_value_produces_error(src in incomplete_nve()) {
        let result = syn::parse_str::<NameValueExpr>(&src);
        prop_assert!(result.is_err());
    }
}

// ===========================================================================
// 2. Error display formatting
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 6. Display of NVE parse error is non-empty.
    #[test]
    fn nve_error_display_non_empty(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let display = format!("{e}");
            prop_assert!(!display.is_empty(), "error display should be non-empty");
        }
    }

    /// 7. Display of FTP parse error is non-empty.
    #[test]
    fn ftp_error_display_non_empty(_dummy in 0..1u8) {
        if let Err(e) = syn::parse_str::<FieldThenParams>("") {
            let display = format!("{e}");
            prop_assert!(!display.is_empty(), "error display should be non-empty");
        }
    }

    /// 8. Display of NVE error contains "expected" — syn errors describe expectations.
    #[test]
    fn nve_error_display_contains_expected(name in ident_strategy(), val in int_value()) {
        // Missing `=` between ident and value
        let src = format!("{name} {val}");
        if let Err(e) = syn::parse_str::<NameValueExpr>(&src) {
            let display = format!("{e}");
            prop_assert!(
                display.contains("expected") || display.contains("unexpected"),
                "error display should mention expectation: {display}"
            );
        }
    }

    /// 9. Display of NVE error for empty input mentions "expected".
    #[test]
    fn nve_empty_error_display_mentions_expected(_dummy in 0..1u8) {
        if let Err(e) = syn::parse_str::<NameValueExpr>("") {
            let display = format!("{e}");
            prop_assert!(
                display.contains("expected") || display.contains("unexpected") || display.contains("end of input"),
                "error display should describe what went wrong: {display}"
            );
        }
    }

    /// 10. Display formatting is consistent across two calls on same input.
    #[test]
    fn nve_error_display_deterministic(name in ident_strategy()) {
        let src = format!("{name} =");
        let e1 = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        let e2 = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        prop_assert_eq!(format!("{e1}"), format!("{e2}"));
    }
}

// ===========================================================================
// 3. Error debug output
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 11. Debug output of NVE parse error is non-empty.
    #[test]
    fn nve_error_debug_non_empty(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let debug = format!("{e:?}");
            prop_assert!(!debug.is_empty(), "error debug should be non-empty");
        }
    }

    /// 12. Debug output of FTP parse error is non-empty.
    #[test]
    fn ftp_error_debug_non_empty(_dummy in 0..1u8) {
        if let Err(e) = syn::parse_str::<FieldThenParams>("") {
            let debug = format!("{e:?}");
            prop_assert!(!debug.is_empty(), "error debug should be non-empty");
        }
    }

    /// 13. Debug and Display produce different representations.
    #[test]
    fn nve_error_debug_differs_from_display(name in ident_strategy()) {
        let src = format!("{name} =");
        let err = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        let display = format!("{err}");
        let debug = format!("{err:?}");
        // Debug typically wraps in Error(...) or similar
        prop_assert_ne!(display, debug, "Debug and Display should differ");
    }

    /// 14. Debug output contains "Error" keyword.
    #[test]
    fn nve_error_debug_contains_error_keyword(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let debug = format!("{e:?}");
            prop_assert!(
                debug.contains("Error") || debug.contains("error"),
                "debug should reference Error: {debug}"
            );
        }
    }
}

// ===========================================================================
// 4. Error clone
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 15. Cloned NVE error has same display as original.
    #[test]
    fn nve_error_clone_display_matches(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let cloned = e.clone();
            prop_assert_eq!(format!("{e}"), format!("{cloned}"));
        }
    }

    /// 16. Cloned FTP error has same display as original.
    #[test]
    fn ftp_error_clone_display_matches(_dummy in 0..1u8) {
        let err = syn::parse_str::<FieldThenParams>("").unwrap_err();
        let cloned = err.clone();
        prop_assert_eq!(format!("{err}"), format!("{cloned}"));
    }

    /// 17. Cloned error has same debug as original.
    #[test]
    fn nve_error_clone_debug_matches(name in ident_strategy()) {
        let src = format!("{name} =");
        let err = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        let cloned = err.clone();
        prop_assert_eq!(format!("{err:?}"), format!("{cloned:?}"));
    }

    /// 18. Cloned error span matches original error span.
    #[test]
    fn nve_error_clone_span_matches(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let cloned = e.clone();
            let orig_span = format!("{:?}", e.span());
            let clone_span = format!("{:?}", cloned.span());
            prop_assert_eq!(orig_span, clone_span);
        }
    }
}

// ===========================================================================
// 5. Error equality (via display/debug comparison since syn::Error has no Eq)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 19. Same invalid input produces same error message (display equality).
    #[test]
    fn nve_same_input_same_error_display(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} {val}");
        let e1 = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        let e2 = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        prop_assert_eq!(format!("{e1}"), format!("{e2}"));
    }

    /// 20. Same invalid input produces same error debug (debug equality).
    #[test]
    fn nve_same_input_same_error_debug(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} {val}");
        let e1 = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        let e2 = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        prop_assert_eq!(format!("{e1:?}"), format!("{e2:?}"));
    }

    /// 21. Different invalid inputs can produce different error messages.
    #[test]
    fn different_errors_can_differ(_dummy in 0..1u8) {
        let e_empty = syn::parse_str::<NameValueExpr>("").unwrap_err();
        let e_partial = syn::parse_str::<NameValueExpr>("x =").unwrap_err();
        let d1 = format!("{e_empty}");
        let d2 = format!("{e_partial}");
        // They may or may not differ, but both should be non-empty
        prop_assert!(!d1.is_empty());
        prop_assert!(!d2.is_empty());
    }
}

// ===========================================================================
// 6. Error from conversions (syn::Error <-> std::io::Error, into_compile_error)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 22. syn::Error can be converted to a compile error token stream.
    #[test]
    fn nve_error_into_compile_error(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let tokens = e.into_compile_error();
            let token_str = tokens.to_string();
            prop_assert!(
                !token_str.is_empty(),
                "compile_error tokens should be non-empty"
            );
            prop_assert!(
                token_str.contains("compile_error"),
                "should contain compile_error!: {token_str}"
            );
        }
    }

    /// 23. syn::Error implements std::error::Error — source() returns None for simple errors.
    #[test]
    fn nve_error_source_is_none(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let source = StdError::source(&e);
            prop_assert!(source.is_none(), "simple syn errors have no source");
        }
    }

    /// 24. syn::Error can be created from a custom message and used like a parse error.
    #[test]
    fn syn_error_from_custom_message(msg in "[a-z ]{1,40}") {
        let span = proc_macro2::Span::call_site();
        let err = syn::Error::new(span, &msg);
        let display = format!("{err}");
        prop_assert!(
            display.contains(&msg),
            "custom error display should contain message: {display}"
        );
    }

    /// 25. Custom syn::Error clone preserves message.
    #[test]
    fn syn_error_custom_clone_preserves_msg(msg in "[a-z ]{1,40}") {
        let span = proc_macro2::Span::call_site();
        let err = syn::Error::new(span, &msg);
        let cloned = err.clone();
        prop_assert_eq!(format!("{err}"), format!("{cloned}"));
    }
}

// ===========================================================================
// 7. Error messages contain useful info
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 26. NVE error for missing `=` mentions `=` or expectation.
    #[test]
    fn nve_missing_eq_error_is_informative(name in ident_strategy()) {
        let src = format!("{name} 42");
        let err = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        let display = format!("{err}");
        prop_assert!(
            display.contains('=') || display.contains("expected") || display.contains("unexpected"),
            "error for missing `=` should be informative: {display}"
        );
    }

    /// 27. FTP error for empty input gives an informative message.
    #[test]
    fn ftp_empty_error_informative(_dummy in 0..1u8) {
        let err = syn::parse_str::<FieldThenParams>("").unwrap_err();
        let display = format!("{err}");
        prop_assert!(
            display.len() > 5,
            "error message should be descriptive, got: {display}"
        );
    }

    /// 28. NVE error message length is reasonable (not absurdly long).
    #[test]
    fn nve_error_message_reasonable_length(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let display = format!("{e}");
            prop_assert!(display.len() < 1000, "error too long: {} chars", display.len());
        }
    }

    /// 29. Valid NVE input does NOT produce an error (positive control).
    #[test]
    fn nve_valid_input_no_error(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let result = syn::parse_str::<NameValueExpr>(&src);
        prop_assert!(result.is_ok(), "valid input should parse: {src}");
    }

    /// 30. Valid FTP input does NOT produce an error (positive control).
    #[test]
    fn ftp_valid_input_no_error(ty in leaf_type_name()) {
        let result = syn::parse_str::<FieldThenParams>(ty);
        prop_assert!(result.is_ok(), "valid bare type should parse: {ty}");
    }
}

// ===========================================================================
// 8. Error with source context (spans and combined errors)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 31. Error span is accessible (does not panic).
    #[test]
    fn nve_error_span_accessible(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            // Accessing span should not panic
            let _span = e.span();
            let _debug = format!("{:?}", e.span());
            prop_assert!(!_debug.is_empty());
        }
    }

    /// 32. Combined errors via `combine` produce multi-message output.
    #[test]
    fn syn_error_combine_produces_multi_error(
        msg_a in "[a-z]{3,15}",
        msg_b in "[a-z]{3,15}",
    ) {
        let span = proc_macro2::Span::call_site();
        let mut err_a = syn::Error::new(span, &msg_a);
        let err_b = syn::Error::new(span, &msg_b);
        err_a.combine(err_b);
        let compile_tokens = err_a.into_compile_error().to_string();
        // Combined error should mention both messages
        prop_assert!(
            compile_tokens.contains(&msg_a) && compile_tokens.contains(&msg_b),
            "combined error should contain both messages: {compile_tokens}"
        );
    }

    /// 33. Error iterator yields at least one error for simple parse failures.
    #[test]
    fn nve_error_iter_non_empty(bad in garbage_input()) {
        if let Err(e) = syn::parse_str::<NameValueExpr>(&bad) {
            let count = e.into_iter().count();
            prop_assert!(count >= 1, "error iterator should yield at least one error");
        }
    }

    /// 34. Combined custom errors iterate to yield multiple entries.
    #[test]
    fn syn_error_combined_iter_count(
        msg_a in "[a-z]{3,15}",
        msg_b in "[a-z]{3,15}",
    ) {
        let span = proc_macro2::Span::call_site();
        let mut err_a = syn::Error::new(span, &msg_a);
        let err_b = syn::Error::new(span, &msg_b);
        err_a.combine(err_b);
        let count = err_a.into_iter().count();
        prop_assert!(count >= 2, "combined errors should iterate to >= 2 entries, got {count}");
    }

    /// 35. Error span debug representation is stable across clone.
    #[test]
    fn nve_error_span_debug_stable_across_clone(name in ident_strategy()) {
        let src = format!("{name} =");
        let err = syn::parse_str::<NameValueExpr>(&src).unwrap_err();
        let cloned = err.clone();
        let span_dbg = format!("{:?}", err.span());
        let clone_span_dbg = format!("{:?}", cloned.span());
        prop_assert_eq!(span_dbg, clone_span_dbg);
    }
}
