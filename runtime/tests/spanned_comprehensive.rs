#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the `Spanned<T>` wrapper in the adze runtime.
//!
//! Covers creation, deref, byte ranges, span indexing, clone, debug,
//! comparison, error handling, and usage with various inner types.

use adze::{SpanError, SpanErrorReason, Spanned};
use std::ops::Deref;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn mk<T>(value: T, start: usize, end: usize) -> Spanned<T> {
    Spanned {
        value,
        span: (start, end),
    }
}

// ===========================================================================
// 1. Creation
// ===========================================================================

#[test]
fn creation_with_i32_preserves_value_and_span() {
    let s = mk(42, 0, 5);
    assert_eq!(s.value, 42);
    assert_eq!(s.span, (0, 5));
}

#[test]
fn creation_with_string_preserves_value_and_span() {
    let s = mk(String::from("hello"), 3, 8);
    assert_eq!(s.value, "hello");
    assert_eq!(s.span, (3, 8));
}

#[test]
fn creation_with_zero_width_span() {
    let s = mk(true, 7, 7);
    assert_eq!(s.value, true);
    assert_eq!(s.span.0, s.span.1);
}

// ===========================================================================
// 2. Deref to inner value
// ===========================================================================

#[test]
fn deref_returns_inner_i32() {
    let s = mk(99, 0, 2);
    let inner: &i32 = s.deref();
    assert_eq!(*inner, 99);
}

#[test]
fn deref_with_star_operator() {
    let s = mk(3.14_f64, 0, 4);
    assert!((*s - 3.14).abs() < f64::EPSILON);
}

#[test]
fn deref_string_allows_str_methods() {
    let s = mk(String::from("HELLO"), 0, 5);
    // Deref to String, then auto-deref to str for .to_lowercase()
    assert_eq!(s.to_lowercase(), "hello");
}

#[test]
fn deref_vec_allows_slice_methods() {
    let s = mk(vec![1, 2, 3], 0, 10);
    assert_eq!(s.len(), 3);
    assert!(s.contains(&2));
}

// ===========================================================================
// 3. Byte range
// ===========================================================================

#[test]
fn byte_range_start_end() {
    let s = mk("token", 10, 15);
    assert_eq!(s.span.0, 10);
    assert_eq!(s.span.1, 15);
}

#[test]
fn byte_range_length() {
    let s = mk((), 5, 12);
    let len = s.span.1 - s.span.0;
    assert_eq!(len, 7);
}

#[test]
fn byte_range_at_origin() {
    let s = mk('a', 0, 1);
    assert_eq!(s.span, (0, 1));
}

#[test]
fn byte_range_large_offsets() {
    let s = mk(0u8, 1_000_000, 2_000_000);
    assert_eq!(s.span.0, 1_000_000);
    assert_eq!(s.span.1, 2_000_000);
}

// ===========================================================================
// 4. Span indexing into source str
// ===========================================================================

#[test]
fn index_str_with_spanned() {
    let source = "hello world";
    let s = mk((), 6, 11);
    assert_eq!(&source[s], "world");
}

#[test]
fn index_str_with_zero_width_span() {
    let source = "abc";
    let s = mk(0, 1, 1);
    assert_eq!(&source[s], "");
}

#[test]
fn index_str_full_range() {
    let source = "full";
    let s = mk(true, 0, 4);
    assert_eq!(&source[s], "full");
}

#[test]
fn index_str_single_char() {
    let source = "abcdef";
    let s = mk(42, 2, 3);
    assert_eq!(&source[s], "c");
}

#[test]
fn index_str_unicode_boundary() {
    // "café" is 5 bytes: c(1) a(1) f(1) é(2)
    let source = "café";
    let s = mk((), 0, 3);
    assert_eq!(&source[s], "caf");
}

// ===========================================================================
// 5. PartialEq — Spanned does NOT derive PartialEq, so we test field equality
// ===========================================================================

#[test]
fn field_equality_same_value_same_span() {
    let a = mk(10, 0, 5);
    let b = mk(10, 0, 5);
    assert_eq!(a.value, b.value);
    assert_eq!(a.span, b.span);
}

#[test]
fn field_equality_same_value_different_span() {
    let a = mk(10, 0, 5);
    let b = mk(10, 3, 8);
    assert_eq!(a.value, b.value);
    assert_ne!(a.span, b.span);
}

#[test]
fn field_equality_different_value_same_span() {
    let a = mk(10, 0, 5);
    let b = mk(20, 0, 5);
    assert_ne!(a.value, b.value);
    assert_eq!(a.span, b.span);
}

// ===========================================================================
// 6. Clone
// ===========================================================================

#[test]
fn clone_preserves_value_and_span() {
    let original = mk(42, 3, 7);
    let cloned = original.clone();
    assert_eq!(cloned.value, 42);
    assert_eq!(cloned.span, (3, 7));
}

#[test]
fn clone_is_independent_of_original() {
    let original = mk(String::from("test"), 0, 4);
    let mut cloned = original.clone();
    cloned.value.push_str("_extra");
    // original is unaffected
    assert_eq!(original.value, "test");
    assert_eq!(cloned.value, "test_extra");
}

#[test]
fn clone_span_is_independent() {
    let original = mk(1, 0, 5);
    let mut cloned = original.clone();
    cloned.span = (10, 20);
    assert_eq!(original.span, (0, 5));
    assert_eq!(cloned.span, (10, 20));
}

// ===========================================================================
// 7. Debug format
// ===========================================================================

#[test]
fn debug_format_contains_value() {
    let s = mk(42, 0, 2);
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("42"), "debug should contain value: {}", dbg);
}

#[test]
fn debug_format_contains_span() {
    let s = mk("x", 3, 5);
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("3"), "debug should contain start: {}", dbg);
    assert!(dbg.contains("5"), "debug should contain end: {}", dbg);
}

#[test]
fn debug_format_contains_struct_name() {
    let s = mk(0, 0, 0);
    let dbg = format!("{:?}", s);
    assert!(
        dbg.contains("Spanned"),
        "debug should contain 'Spanned': {}",
        dbg
    );
}

// ===========================================================================
// 8. Different inner types
// ===========================================================================

#[test]
fn spanned_bool() {
    let s = mk(true, 0, 4);
    assert!(*s);
}

#[test]
fn spanned_f32() {
    let s = mk(2.5_f32, 0, 3);
    assert!((*s - 2.5).abs() < f32::EPSILON);
}

#[test]
fn spanned_option() {
    let s = mk(Some(42), 0, 2);
    assert_eq!(*s, Some(42));
}

#[test]
fn spanned_tuple() {
    let s = mk((1, "two"), 0, 5);
    assert_eq!(s.value.0, 1);
    assert_eq!(s.value.1, "two");
}

#[test]
fn spanned_nested_spanned() {
    let inner = mk(99, 0, 2);
    let outer = mk(inner, 0, 10);
    // Deref outer → Spanned<i32>, deref that → i32
    assert_eq!(**outer, 99);
}

#[test]
fn spanned_unit() {
    let s = mk((), 5, 5);
    assert_eq!(s.value, ());
    assert_eq!(s.span, (5, 5));
}

// ===========================================================================
// 9. SpanError and validation via panicking Index
// ===========================================================================

#[test]
#[should_panic(expected = "Invalid span")]
fn index_panics_on_start_greater_than_end() {
    let source = "hello";
    let s = mk((), 3, 1); // start > end
    let _ = &source[s];
}

#[test]
#[should_panic(expected = "Invalid span")]
fn index_panics_on_end_out_of_bounds() {
    let source = "hi";
    let s = mk((), 0, 100);
    let _ = &source[s];
}

#[test]
#[should_panic(expected = "Invalid span")]
fn index_panics_on_start_out_of_bounds() {
    let source = "ab";
    let s = mk((), 50, 100);
    let _ = &source[s];
}

#[test]
fn span_error_display_start_greater_than_end() {
    let err = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("start"));
    assert!(msg.contains("end"));
}

#[test]
fn span_error_display_end_out_of_bounds() {
    let err = SpanError {
        span: (0, 20),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("20"));
    assert!(msg.contains("10"));
}

#[test]
fn span_error_is_std_error() {
    let err = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &err;
}
