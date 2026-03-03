#![allow(clippy::needless_range_loop)]

//! Property-based tests for `Spanned<T>`, `SpanError`, `SpanErrorReason`,
//! and the `str` indexing implementations.

use std::panic;

use adze::{SpanError, SpanErrorReason, Spanned};
use proptest::prelude::*;
use proptest::strategy::ValueTree;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `Spanned<T>` with given value and span.
fn spanned<T>(value: T, start: usize, end: usize) -> Spanned<T> {
    Spanned {
        value,
        span: (start, end),
    }
}

/// Strategy for a non-empty ASCII source string (1..=256 bytes).
fn source_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,256}"
}

// ---------------------------------------------------------------------------
// 1. Deref returns inner value (i32)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_returns_inner_value_i32(v in any::<i32>(), s in 0usize..100, e in 0usize..100) {
        let sp = spanned(v, s, e);
        prop_assert_eq!(*sp, v);
    }
}

// ---------------------------------------------------------------------------
// 2. Deref returns inner value (String)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_returns_inner_value_string(v in ".*", s in 0usize..100, e in 0usize..100) {
        let sp = spanned(v.clone(), s, e);
        prop_assert_eq!(&*sp, &v);
    }
}

// ---------------------------------------------------------------------------
// 3. Clone preserves value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn clone_preserves_value(v in any::<i64>(), s in 0usize..100, e in 0usize..100) {
        let sp = spanned(v, s, e);
        let cloned = sp.clone();
        prop_assert_eq!(*cloned, v);
    }
}

// ---------------------------------------------------------------------------
// 4. Clone preserves span
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn clone_preserves_span(v in any::<u32>(), s in 0usize..100, e in 0usize..100) {
        let sp = spanned(v, s, e);
        let cloned = sp.clone();
        prop_assert_eq!(cloned.span, (s, e));
    }
}

// ---------------------------------------------------------------------------
// 5. Debug output contains the value representation
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn debug_contains_value(v in 0i32..10000) {
        let sp = spanned(v, 0, 1);
        let dbg = format!("{:?}", sp);
        prop_assert!(dbg.contains(&v.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 6. Debug output contains span info
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn debug_contains_span(s in 0usize..50, e in 0usize..50) {
        let sp = spanned((), s, e);
        let dbg = format!("{:?}", sp);
        prop_assert!(dbg.contains(&s.to_string()));
        prop_assert!(dbg.contains(&e.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 7. Valid span indexing returns correct slice
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn valid_span_index_returns_correct_slice(src in source_strategy()) {
        let len = src.len();
        let mut runner = proptest::test_runner::TestRunner::deterministic();
        let (start, end) = (0..=len)
            .prop_flat_map(move |s| (Just(s), s..=len))
            .new_tree(&mut runner)
            .unwrap()
            .current();
        let sp = spanned((), start, end);
        prop_assert_eq!(&src.as_str()[sp], &src[start..end]);
    }
}

// ---------------------------------------------------------------------------
// 8. Empty span at any valid position returns empty string
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn empty_span_at_valid_position(src in source_strategy()) {
        let len = src.len();
        for pos in 0..=len {
            let sp = spanned((), pos, pos);
            prop_assert_eq!(&src.as_str()[sp], "");
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Full span returns entire source
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn full_span_returns_entire_source(src in source_strategy()) {
        let len = src.len();
        let sp = spanned((), 0, len);
        prop_assert_eq!(&src.as_str()[sp], src.as_str());
    }
}

// ---------------------------------------------------------------------------
// 10. Value type does not affect indexing
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn value_type_does_not_affect_indexing(src in source_strategy(), v in any::<u64>()) {
        let len = src.len();
        let sp_unit = spanned((), 0, len);
        let sp_u64 = spanned(v, 0, len);
        let a = &src.as_str()[sp_unit];
        let b = &src.as_str()[sp_u64];
        prop_assert_eq!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 11. start > end panics via Index
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn start_greater_than_end_panics(src in source_strategy(), start in 1usize..256) {
        let len = src.len();
        let start = start.min(len);
        if start > 0 {
            let end = start - 1;
            let sp = spanned((), start, end);
            let result = panic::catch_unwind(move || {
                let _ = &src.as_str()[sp];
            });
            prop_assert!(result.is_err());
        }
    }
}

// ---------------------------------------------------------------------------
// 12. end > source_len panics via Index
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn end_out_of_bounds_panics(src in source_strategy(), extra in 1usize..100) {
        let len = src.len();
        let sp = spanned((), 0, len + extra);
        let result = panic::catch_unwind(move || {
            let _ = &src.as_str()[sp];
        });
        prop_assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// 13. start > source_len panics via Index
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn start_out_of_bounds_panics(src in source_strategy(), extra in 1usize..100) {
        let len = src.len();
        let sp = spanned((), len + extra, len + extra + 1);
        let result = panic::catch_unwind(move || {
            let _ = &src.as_str()[sp];
        });
        prop_assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// 14. Adjacent spans concatenate to parent span
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn adjacent_spans_concatenate(src in source_strategy()) {
        let len = src.len();
        if len >= 2 {
            let mid = len / 2;
            let a = src.as_str()[spanned((), 0, mid)].to_owned();
            let b = src.as_str()[spanned((), mid, len)].to_owned();
            let full = &src.as_str()[spanned((), 0, len)];
            let combined = format!("{}{}", a, b);
            prop_assert_eq!(combined.as_str(), full);
        }
    }
}

// ---------------------------------------------------------------------------
// 15. IndexMut with valid span works
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn index_mut_valid_span(src in "[a-z]{1,64}") {
        let mut s = src.clone();
        let len = s.len();
        let sp = spanned((), 0, len);
        s.as_mut_str()[sp].make_ascii_uppercase();
        prop_assert_eq!(s, src.to_ascii_uppercase());
    }
}

// ---------------------------------------------------------------------------
// 16. IndexMut invalid span panics
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn index_mut_invalid_panics(src in source_strategy(), extra in 1usize..50) {
        let len = src.len();
        let mut s = src;
        let sp = spanned((), 0, len + extra);
        let result = panic::catch_unwind(move || {
            let _ = &mut s.as_mut_str()[sp];
        });
        prop_assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// 17. SpanError Display — StartGreaterThanEnd format
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_start_gt_end(
        start in 1usize..1000,
        delta in 1usize..1000,
        slen in 0usize..2000,
    ) {
        let end = start.saturating_sub(delta).min(start - 1);
        let err = SpanError {
            span: (start, end),
            source_len: slen,
            reason: SpanErrorReason::StartGreaterThanEnd,
        };
        let msg = err.to_string();
        prop_assert!(msg.contains("start"));
        prop_assert!(msg.contains("end"));
        prop_assert!(msg.contains(&start.to_string()));
        prop_assert!(msg.contains(&end.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 18. SpanError Display — StartOutOfBounds format
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_start_oob(
        start in 0usize..1000,
        end in 0usize..1000,
        slen in 0usize..1000,
    ) {
        let err = SpanError {
            span: (start, end),
            source_len: slen,
            reason: SpanErrorReason::StartOutOfBounds,
        };
        let msg = err.to_string();
        prop_assert!(msg.contains("start"));
        prop_assert!(msg.contains("source length"));
        prop_assert!(msg.contains(&slen.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 19. SpanError Display — EndOutOfBounds format
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_end_oob(
        start in 0usize..1000,
        end in 0usize..1000,
        slen in 0usize..1000,
    ) {
        let err = SpanError {
            span: (start, end),
            source_len: slen,
            reason: SpanErrorReason::EndOutOfBounds,
        };
        let msg = err.to_string();
        prop_assert!(msg.contains("end"));
        prop_assert!(msg.contains("source length"));
        prop_assert!(msg.contains(&slen.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 20. SpanError Clone roundtrip preserves equality
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_clone_roundtrip(
        start in 0usize..500,
        end in 0usize..500,
        slen in 0usize..500,
    ) {
        let err = SpanError {
            span: (start, end),
            source_len: slen,
            reason: SpanErrorReason::StartGreaterThanEnd,
        };
        let cloned = err.clone();
        prop_assert_eq!(err, cloned);
    }
}

// ---------------------------------------------------------------------------
// 21. SpanError PartialEq is reflexive
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_eq_reflexive(
        start in 0usize..500,
        end in 0usize..500,
        slen in 0usize..500,
    ) {
        let err = SpanError {
            span: (start, end),
            source_len: slen,
            reason: SpanErrorReason::EndOutOfBounds,
        };
        prop_assert!(err == err.clone());
    }
}

// ---------------------------------------------------------------------------
// 22. SpanErrorReason variants are distinct
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_reason_variants_distinct(_dummy in 0u8..1) {
        let a = SpanErrorReason::StartGreaterThanEnd;
        let b = SpanErrorReason::StartOutOfBounds;
        let c = SpanErrorReason::EndOutOfBounds;
        prop_assert!(a != b);
        prop_assert!(a.clone() != c);
        prop_assert!(b != c);
    }
}

// ---------------------------------------------------------------------------
// 23. SpanError implements std::error::Error (source is None)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_is_std_error(
        start in 0usize..100,
        end in 0usize..100,
        slen in 0usize..100,
    ) {
        let err = SpanError {
            span: (start, end),
            source_len: slen,
            reason: SpanErrorReason::StartGreaterThanEnd,
        };
        let dyn_err: &dyn std::error::Error = &err;
        prop_assert!(dyn_err.source().is_none());
        prop_assert!(!dyn_err.to_string().is_empty());
    }
}

// ---------------------------------------------------------------------------
// 24. SpanError fields are accessible and correct
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_fields_accessible(
        s in 0usize..1000,
        e in 0usize..1000,
        slen in 0usize..1000,
    ) {
        let err = SpanError {
            span: (s, e),
            source_len: slen,
            reason: SpanErrorReason::EndOutOfBounds,
        };
        prop_assert_eq!(err.span, (s, e));
        prop_assert_eq!(err.source_len, slen);
        prop_assert_eq!(err.reason, SpanErrorReason::EndOutOfBounds);
    }
}

// ---------------------------------------------------------------------------
// 25. SpanErrorReason Clone roundtrip
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_reason_clone(idx in 0u8..3) {
        let reason = match idx {
            0 => SpanErrorReason::StartGreaterThanEnd,
            1 => SpanErrorReason::StartOutOfBounds,
            _ => SpanErrorReason::EndOutOfBounds,
        };
        let cloned = reason.clone();
        prop_assert_eq!(reason, cloned);
    }
}

// ---------------------------------------------------------------------------
// 26. Zero-length source only allows (0,0) span
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn zero_length_source_valid_span(_dummy in 0u8..1) {
        let src = "";
        let sp = spanned((), 0, 0);
        prop_assert_eq!(&src[sp], "");
    }
}

// ---------------------------------------------------------------------------
// 27. Zero-length source rejects non-zero spans
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn zero_length_source_rejects_nonzero(end in 1usize..100) {
        let src = String::new();
        let sp = spanned((), 0, end);
        let result = panic::catch_unwind(move || {
            let _ = &src.as_str()[sp];
        });
        prop_assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// 28. Span at exact boundary: start == len, end == len
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_at_exact_boundary(src in source_strategy()) {
        let len = src.len();
        let sp = spanned((), len, len);
        prop_assert_eq!(&src.as_str()[sp], "");
    }
}

// ---------------------------------------------------------------------------
// 29. Deref with bool type
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_with_bool(v in any::<bool>()) {
        let sp = spanned(v, 0, 1);
        prop_assert_eq!(*sp, v);
    }
}

// ---------------------------------------------------------------------------
// 30. Multiple non-overlapping spans from same source
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn multiple_non_overlapping_spans(src in "[a-z]{4,64}") {
        let len = src.len();
        let quarter = len / 4;
        if quarter > 0 {
            let a = src.as_str()[spanned((), 0, quarter)].to_owned();
            let b = src.as_str()[spanned((), quarter, quarter * 2)].to_owned();
            let c = src.as_str()[spanned((), quarter * 2, quarter * 3)].to_owned();
            let d = src.as_str()[spanned((), quarter * 3, len)].to_owned();
            let combined = format!("{}{}{}{}", a, b, c, d);
            prop_assert_eq!(combined.as_str(), src.as_str());
        }
    }
}

// ---------------------------------------------------------------------------
// 31. Panic message includes "Invalid span"
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn panic_message_contains_invalid_span(src in source_strategy(), extra in 1usize..50) {
        let len = src.len();
        let sp = spanned((), 0, len + extra);
        let result = panic::catch_unwind(move || {
            let _ = &src.as_str()[sp];
        });
        if let Err(payload) = result && let Some(msg) = payload.downcast_ref::<String>() {
            prop_assert!(msg.contains("Invalid span"));
        }
    }
}

// ---------------------------------------------------------------------------
// 32. Deref with f64 type
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_with_f64(v in any::<f64>()) {
        let sp = spanned(v, 0, 8);
        prop_assert_eq!((*sp).to_bits(), v.to_bits());
    }
}

// ---------------------------------------------------------------------------
// 33. Spanned span field is publicly readable
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_field_is_correct(s in 0usize..1000, e in 0usize..1000) {
        let sp = spanned(42u8, s, e);
        prop_assert_eq!(sp.span.0, s);
        prop_assert_eq!(sp.span.1, e);
    }
}

// ---------------------------------------------------------------------------
// 34. SpanError with different reasons are not equal
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_different_reasons_not_equal(
        s in 0usize..100,
        e in 0usize..100,
        slen in 0usize..100,
    ) {
        let e1 = SpanError { span: (s, e), source_len: slen, reason: SpanErrorReason::StartGreaterThanEnd };
        let e2 = SpanError { span: (s, e), source_len: slen, reason: SpanErrorReason::StartOutOfBounds };
        let e3 = SpanError { span: (s, e), source_len: slen, reason: SpanErrorReason::EndOutOfBounds };
        prop_assert!(e1 != e2);
        prop_assert!(e1.clone() != e3);
        prop_assert!(e2 != e3);
    }
}

// ---------------------------------------------------------------------------
// 35. Single-byte spans extract correct character
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn single_byte_span_correct(src in "[a-z]{1,128}") {
        let len = src.len();
        for i in 0..len {
            let sp = spanned((), i, i + 1);
            let expected = &src[i..i + 1];
            prop_assert_eq!(&src.as_str()[sp], expected);
        }
    }
}
