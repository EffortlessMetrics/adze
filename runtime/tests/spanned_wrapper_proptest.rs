#![allow(clippy::needless_range_loop)]

//! Property-based tests for the `Spanned<T>` wrapper in the adze runtime.
//!
//! Covers construction, position validity, comparison, display, clone/debug
//! behaviour, position arithmetic, and usage with various inner types.

use std::panic;

use adze::{SpanError, SpanErrorReason, Spanned};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn mk<T>(value: T, start: usize, end: usize) -> Spanned<T> {
    Spanned {
        value,
        span: (start, end),
    }
}

fn ascii_source() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,128}"
}

// ---------------------------------------------------------------------------
// 1. Construction with random i32 preserves value and span
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn construction_preserves_value_and_span(
        v in any::<i32>(),
        s in 0usize..500,
        e in 0usize..500,
    ) {
        let sp = mk(v, s, e);
        prop_assert_eq!(sp.value, v);
        prop_assert_eq!(sp.span, (s, e));
    }
}

// ---------------------------------------------------------------------------
// 2. Construction with String preserves value and span
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn construction_string_preserves(
        v in ".*{0,64}",
        s in 0usize..500,
        e in 0usize..500,
    ) {
        let sp = mk(v.clone(), s, e);
        prop_assert_eq!(&sp.value, &v);
        prop_assert_eq!(sp.span, (s, e));
    }
}

// ---------------------------------------------------------------------------
// 3. Start <= end is a valid span for indexing
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn valid_span_start_le_end(src in ascii_source()) {
        let len = src.len();
        if len > 0 {
            let start = 0;
            let end = len;
            let sp = mk((), start, end);
            let slice = &src.as_str()[sp];
            prop_assert_eq!(slice, src.as_str());
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Start > end always panics on index
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn start_gt_end_always_panics(
        src in ascii_source(),
        delta in 1usize..50,
    ) {
        let len = src.len();
        if len > 0 {
            let start = len;
            let end = start.saturating_sub(delta).min(start - 1);
            let sp = mk((), start, end);
            let result = panic::catch_unwind(move || {
                let _ = &src.as_str()[sp];
            });
            prop_assert!(result.is_err());
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Spanned<i32> deref equals inner value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_i32(v in any::<i32>(), s in 0usize..100, e in 0usize..100) {
        let sp = mk(v, s, e);
        prop_assert_eq!(*sp, v);
    }
}

// ---------------------------------------------------------------------------
// 6. Spanned<u8> deref equals inner value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_u8(v in any::<u8>(), s in 0usize..100, e in 0usize..100) {
        let sp = mk(v, s, e);
        prop_assert_eq!(*sp, v);
    }
}

// ---------------------------------------------------------------------------
// 7. Spanned<u64> deref equals inner value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_u64(v in any::<u64>(), s in 0usize..100, e in 0usize..100) {
        let sp = mk(v, s, e);
        prop_assert_eq!(*sp, v);
    }
}

// ---------------------------------------------------------------------------
// 8. Spanned<bool> deref equals inner value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_bool(v in any::<bool>(), s in 0usize..100, e in 0usize..100) {
        let sp = mk(v, s, e);
        prop_assert_eq!(*sp, v);
    }
}

// ---------------------------------------------------------------------------
// 9. Spanned<Vec<u8>> deref equals inner value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_vec_u8(v in proptest::collection::vec(any::<u8>(), 0..32)) {
        let sp = mk(v.clone(), 0, 1);
        prop_assert_eq!(&*sp, &v);
    }
}

// ---------------------------------------------------------------------------
// 10. Spanned<Option<i32>> deref equals inner value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_option_i32(v in any::<Option<i32>>()) {
        let sp = mk(v, 0, 1);
        prop_assert_eq!(*sp, v);
    }
}

// ---------------------------------------------------------------------------
// 11. Clone produces identical value and span
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn clone_identical(
        v in any::<i64>(),
        s in 0usize..500,
        e in 0usize..500,
    ) {
        let sp = mk(v, s, e);
        let cl = sp.clone();
        prop_assert_eq!(cl.value, sp.value);
        prop_assert_eq!(cl.span, sp.span);
    }
}

// ---------------------------------------------------------------------------
// 12. Clone is independent — mutating clone does not affect original
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn clone_independence(v in any::<i32>(), s in 0usize..100, e in 0usize..100) {
        let sp = mk(v, s, e);
        let mut cl = sp.clone();
        cl.value = v.wrapping_add(1);
        cl.span = (s.wrapping_add(1), e.wrapping_add(1));
        // Clone was mutated independently
        prop_assert_eq!(cl.value, v.wrapping_add(1));
        prop_assert_eq!(cl.span, (s.wrapping_add(1), e.wrapping_add(1)));
        // Original is unchanged
        prop_assert_eq!(sp.value, v);
        prop_assert_eq!(sp.span, (s, e));
    }
}

// ---------------------------------------------------------------------------
// 13. Debug output contains value
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn debug_shows_value(v in 0i32..100_000) {
        let sp = mk(v, 0, 1);
        let dbg = format!("{:?}", sp);
        prop_assert!(dbg.contains(&v.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 14. Debug output contains span start and end
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn debug_shows_span(s in 0usize..1000, e in 0usize..1000) {
        let sp = mk(0u8, s, e);
        let dbg = format!("{:?}", sp);
        prop_assert!(dbg.contains(&s.to_string()));
        prop_assert!(dbg.contains(&e.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 15. Debug output starts with "Spanned"
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn debug_starts_with_spanned(v in any::<u32>()) {
        let sp = mk(v, 0, 1);
        let dbg = format!("{:?}", sp);
        prop_assert!(dbg.starts_with("Spanned"));
    }
}

// ---------------------------------------------------------------------------
// 16. Span length equals end - start when valid
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_length_is_end_minus_start(s in 0usize..500, delta in 0usize..500) {
        let e = s + delta;
        let sp = mk((), s, e);
        prop_assert_eq!(sp.span.1 - sp.span.0, delta);
    }
}

// ---------------------------------------------------------------------------
// 17. Adjacent spans cover the full source
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn adjacent_spans_cover_source(src in "[a-z]{2,64}") {
        let len = src.len();
        let mid = len / 2;
        let left = &src.as_str()[mk((), 0, mid)];
        let right = &src.as_str()[mk((), mid, len)];
        prop_assert_eq!(format!("{}{}", left, right), src);
    }
}

// ---------------------------------------------------------------------------
// 18. Three-way split and reassemble
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn three_way_split(src in "[a-z]{3,96}") {
        let len = src.len();
        let a = len / 3;
        let b = 2 * len / 3;
        let p1 = src.as_str()[mk((), 0, a)].to_owned();
        let p2 = src.as_str()[mk((), a, b)].to_owned();
        let p3 = src.as_str()[mk((), b, len)].to_owned();
        prop_assert_eq!(format!("{}{}{}", p1, p2, p3), src);
    }
}

// ---------------------------------------------------------------------------
// 19. Empty span at every position returns empty string
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn empty_span_everywhere(src in ascii_source()) {
        let len = src.len();
        for i in 0..=len {
            let sp = mk((), i, i);
            prop_assert_eq!(&src.as_str()[sp], "");
        }
    }
}

// ---------------------------------------------------------------------------
// 20. Single-byte spans yield single chars
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn single_byte_spans(src in "[a-z]{1,64}") {
        let bytes = src.as_bytes();
        for i in 0..bytes.len() {
            let sp = mk((), i, i + 1);
            let ch = &src.as_str()[sp];
            prop_assert_eq!(ch.len(), 1);
            prop_assert_eq!(ch.as_bytes()[0], bytes[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 21. Position arithmetic — shifting start/end by offset
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn position_shift(
        s in 0usize..200,
        e in 0usize..200,
        offset in 0usize..100,
    ) {
        let sp = mk(42i32, s, e);
        let shifted = mk(42i32, s + offset, e + offset);
        prop_assert_eq!(shifted.span.0, sp.span.0 + offset);
        prop_assert_eq!(shifted.span.1, sp.span.1 + offset);
        // Length preserved
        if e >= s {
            prop_assert_eq!(shifted.span.1 - shifted.span.0, sp.span.1 - sp.span.0);
        }
    }
}

// ---------------------------------------------------------------------------
// 22. Span contains check — midpoint lies within span
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn midpoint_within_span(s in 0usize..500, delta in 1usize..500) {
        let e = s + delta;
        let mid = s + delta / 2;
        prop_assert!(mid >= s);
        prop_assert!(mid < e);
    }
}

// ---------------------------------------------------------------------------
// 23. SpanError Display for StartGreaterThanEnd format
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_sgte(s in 1usize..500, delta in 1usize..500) {
        let e = s.saturating_sub(delta);
        let err = SpanError {
            span: (s, e),
            source_len: 100,
            reason: SpanErrorReason::StartGreaterThanEnd,
        };
        let msg = format!("{}", err);
        prop_assert!(msg.contains("Invalid span"));
        prop_assert!(msg.contains(&s.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 24. SpanError Display for EndOutOfBounds includes source length
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_display_eoob(slen in 1usize..500, extra in 1usize..500) {
        let end = slen + extra;
        let err = SpanError {
            span: (0, end),
            source_len: slen,
            reason: SpanErrorReason::EndOutOfBounds,
        };
        let msg = format!("{}", err);
        prop_assert!(msg.contains("source length"));
        prop_assert!(msg.contains(&slen.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 25. SpanError Debug output is non-empty
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_debug_non_empty(s in 0usize..100, e in 0usize..100, slen in 0usize..100) {
        let err = SpanError {
            span: (s, e),
            source_len: slen,
            reason: SpanErrorReason::StartOutOfBounds,
        };
        let dbg = format!("{:?}", err);
        prop_assert!(!dbg.is_empty());
        prop_assert!(dbg.contains("SpanError"));
    }
}

// ---------------------------------------------------------------------------
// 26. SpanError implements std::error::Error (Display + Debug)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_is_error_trait(s in 0usize..100, e in 0usize..100, slen in 0usize..100) {
        let err = SpanError {
            span: (s, e),
            source_len: slen,
            reason: SpanErrorReason::StartGreaterThanEnd,
        };
        let dyn_err: &dyn std::error::Error = &err;
        // Display produces non-empty string
        prop_assert!(!dyn_err.to_string().is_empty());
        // No underlying source
        prop_assert!(dyn_err.source().is_none());
    }
}

// ---------------------------------------------------------------------------
// 27. SpanErrorReason Clone roundtrip preserves equality
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reason_clone_eq(idx in 0u8..3) {
        let reason = match idx {
            0 => SpanErrorReason::StartGreaterThanEnd,
            1 => SpanErrorReason::StartOutOfBounds,
            _ => SpanErrorReason::EndOutOfBounds,
        };
        prop_assert_eq!(reason.clone(), reason);
    }
}

// ---------------------------------------------------------------------------
// 28. SpanError equality is symmetric
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_eq_symmetric(s in 0usize..200, e in 0usize..200, slen in 0usize..200) {
        let a = SpanError {
            span: (s, e),
            source_len: slen,
            reason: SpanErrorReason::EndOutOfBounds,
        };
        let b = a.clone();
        prop_assert!(a == b);
        prop_assert!(b == a);
    }
}

// ---------------------------------------------------------------------------
// 29. Different source_len makes SpanErrors unequal
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_error_different_slen_not_eq(
        s in 0usize..200,
        e in 0usize..200,
        slen1 in 0usize..200,
        slen2 in 0usize..200,
    ) {
        prop_assume!(slen1 != slen2);
        let a = SpanError {
            span: (s, e),
            source_len: slen1,
            reason: SpanErrorReason::StartOutOfBounds,
        };
        let b = SpanError {
            span: (s, e),
            source_len: slen2,
            reason: SpanErrorReason::StartOutOfBounds,
        };
        prop_assert!(a != b);
    }
}

// ---------------------------------------------------------------------------
// 30. IndexMut works on valid spans
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn index_mut_uppercases(src in "[a-z]{1,64}") {
        let len = src.len();
        let mut s = src.clone();
        let sp = mk((), 0, len);
        s.as_mut_str()[sp].make_ascii_uppercase();
        prop_assert_eq!(s, src.to_ascii_uppercase());
    }
}

// ---------------------------------------------------------------------------
// 31. Spanned with tuple inner type
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn spanned_tuple(a in any::<u16>(), b in any::<u16>(), s in 0usize..100, e in 0usize..100) {
        let sp = mk((a, b), s, e);
        prop_assert_eq!(sp.value, (a, b));
        prop_assert_eq!(sp.span, (s, e));
        let cl = sp.clone();
        prop_assert_eq!(cl.value, (a, b));
    }
}

// ---------------------------------------------------------------------------
// 32. Spanned with nested Spanned
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn nested_spanned(
        v in any::<u32>(),
        s1 in 0usize..100,
        e1 in 0usize..100,
        s2 in 0usize..100,
        e2 in 0usize..100,
    ) {
        let inner = mk(v, s1, e1);
        let outer = mk(inner.clone(), s2, e2);
        prop_assert_eq!(outer.value.value, v);
        prop_assert_eq!(outer.value.span, (s1, e1));
        prop_assert_eq!(outer.span, (s2, e2));
    }
}

// ---------------------------------------------------------------------------
// 33. Span width consistency across clone
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn span_width_preserved_by_clone(s in 0usize..500, delta in 0usize..500) {
        let e = s + delta;
        let sp = mk((), s, e);
        let cl = sp.clone();
        prop_assert_eq!(cl.span.1 - cl.span.0, delta);
    }
}

// ---------------------------------------------------------------------------
// 34. Spanned<f32> deref comparison via to_bits
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn deref_f32(v in any::<f32>()) {
        let sp = mk(v, 0, 4);
        prop_assert_eq!((*sp).to_bits(), v.to_bits());
    }
}

// ---------------------------------------------------------------------------
// 35. Spanned with zero-sized type
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn spanned_zst(s in 0usize..1000, e in 0usize..1000) {
        let sp = mk((), s, e);
        prop_assert_eq!(sp.value, ());
        prop_assert_eq!(sp.span, (s, e));
        let dbg = format!("{:?}", sp);
        prop_assert!(dbg.contains("Spanned"));
    }
}
