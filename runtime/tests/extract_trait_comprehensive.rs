#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for the `Extract` trait, `Spanned`, span validation,
//! `WithLeaf`, error types, and the built-in `Extract` implementations for
//! primitives, `String`, `Option`, `Box`, and `Vec`.

use adze::pure_parser::{ParsedNode, Point};
use adze::{Extract, ExtractDefault, SpanError, SpanErrorReason, Spanned, WithLeaf};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Build a `ParsedNode` using `MaybeUninit` to work around `language`
/// being `pub(crate)`.
fn make_node(
    symbol: u16,
    children: Vec<ParsedNode>,
    start: usize,
    end: usize,
    is_named: bool,
) -> ParsedNode {
    use std::mem::MaybeUninit;

    let mut uninit = MaybeUninit::<ParsedNode>::uninit();
    let ptr = uninit.as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 0, 1);
        std::ptr::addr_of_mut!((*ptr).symbol).write(symbol);
        std::ptr::addr_of_mut!((*ptr).children).write(children);
        std::ptr::addr_of_mut!((*ptr).start_byte).write(start);
        std::ptr::addr_of_mut!((*ptr).end_byte).write(end);
        std::ptr::addr_of_mut!((*ptr).start_point).write(pt(0, start as u32));
        std::ptr::addr_of_mut!((*ptr).end_point).write(pt(0, end as u32));
        std::ptr::addr_of_mut!((*ptr).is_extra).write(false);
        std::ptr::addr_of_mut!((*ptr).is_error).write(false);
        std::ptr::addr_of_mut!((*ptr).is_missing).write(false);
        std::ptr::addr_of_mut!((*ptr).is_named).write(is_named);
        std::ptr::addr_of_mut!((*ptr).field_id).write(None);
        uninit.assume_init()
    }
}

fn leaf(start: usize, end: usize) -> ParsedNode {
    make_node(1, vec![], start, end, true)
}

fn unnamed_leaf(start: usize, end: usize) -> ParsedNode {
    make_node(2, vec![], start, end, false)
}

// ---------------------------------------------------------------------------
// 1. Extract<String> – basic extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_string_from_node() {
    let source = b"hello world";
    let node = leaf(0, 5);
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result, "hello");
}

#[test]
fn extract_string_from_none_returns_empty() {
    let source = b"hello";
    let result = String::extract(None, source, 0, None);
    assert_eq!(result, "");
}

#[test]
fn extract_string_entire_source() {
    let source = b"foobar";
    let node = leaf(0, 6);
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result, "foobar");
}

#[test]
fn extract_string_empty_range() {
    let source = b"abc";
    let node = leaf(1, 1);
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result, "");
}

// ---------------------------------------------------------------------------
// 2. Extract<()> – unit extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_unit_from_some_node() {
    let source = b"x";
    let node = leaf(0, 1);
    // Should not panic; returns ()
    <() as Extract<()>>::extract(Some(&node), source, 0, None);
}

#[test]
fn extract_unit_from_none() {
    let source = b"";
    <() as Extract<()>>::extract(None, source, 0, None);
}

// ---------------------------------------------------------------------------
// 3. Numeric primitive extraction (i32, u64, f64, bool)
// ---------------------------------------------------------------------------

#[test]
fn extract_i32_positive() {
    let source = b"42";
    let node = leaf(0, 2);
    let val = i32::extract(Some(&node), source, 0, None);
    assert_eq!(val, 42);
}

#[test]
fn extract_i32_negative() {
    let source = b"-7";
    let node = leaf(0, 2);
    let val = i32::extract(Some(&node), source, 0, None);
    assert_eq!(val, -7);
}

#[test]
fn extract_i32_invalid_defaults_to_zero() {
    let source = b"notanumber";
    let node = leaf(0, 10);
    let val = i32::extract(Some(&node), source, 0, None);
    assert_eq!(val, 0);
}

#[test]
fn extract_i32_from_none_defaults_to_zero() {
    let source = b"";
    let val = i32::extract(None, source, 0, None);
    assert_eq!(val, 0);
}

#[test]
fn extract_u64_large_value() {
    let text = b"18446744073709551615"; // u64::MAX
    let node = leaf(0, text.len());
    let val = u64::extract(Some(&node), text, 0, None);
    assert_eq!(val, u64::MAX);
}

#[test]
fn extract_f64_with_decimals() {
    let source = b"3.14";
    let node = leaf(0, 4);
    let val = f64::extract(Some(&node), source, 0, None);
    assert!((val - 3.14).abs() < f64::EPSILON);
}

#[test]
fn extract_bool_true() {
    let source = b"true";
    let node = leaf(0, 4);
    let val = bool::extract(Some(&node), source, 0, None);
    assert!(val);
}

#[test]
fn extract_bool_false() {
    let source = b"false";
    let node = leaf(0, 5);
    let val = bool::extract(Some(&node), source, 0, None);
    assert!(!val);
}

// ---------------------------------------------------------------------------
// 4. Extract<Option<U>> – optional extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_option_some() {
    let source = b"99";
    let node = leaf(0, 2);
    let val = <Option<i32> as Extract<Option<i32>>>::extract(Some(&node), source, 0, None);
    assert_eq!(val, Some(99));
}

#[test]
fn extract_option_none() {
    let source = b"";
    let val = <Option<i32> as Extract<Option<i32>>>::extract(None, source, 0, None);
    assert_eq!(val, None);
}

// ---------------------------------------------------------------------------
// 5. Extract<Box<U>> – boxed extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_box_string() {
    let source = b"boxed";
    let node = leaf(0, 5);
    let val = <Box<String> as Extract<Box<String>>>::extract(Some(&node), source, 0, None);
    assert_eq!(*val, "boxed");
}

#[test]
fn extract_box_i32() {
    let source = b"123";
    let node = leaf(0, 3);
    let val = <Box<i32> as Extract<Box<i32>>>::extract(Some(&node), source, 0, None);
    assert_eq!(*val, 123);
}

// ---------------------------------------------------------------------------
// 6. Extract<Vec<U>> – vector extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_vec_from_none_returns_empty() {
    let source = b"";
    let val = <Vec<String> as Extract<Vec<String>>>::extract(None, source, 0, None);
    assert!(val.is_empty());
}

#[test]
fn extract_vec_with_named_children() {
    // Source: "ab"
    let source = b"ab";
    let child_a = leaf(0, 1); // "a"
    let child_b = leaf(1, 2); // "b"
    let parent = make_node(10, vec![child_a, child_b], 0, 2, true);
    let val = <Vec<String> as Extract<Vec<String>>>::extract(Some(&parent), source, 0, None);
    assert_eq!(val, vec!["a", "b"]);
}

// ---------------------------------------------------------------------------
// 7. Spanned – wrapping values with spans
// ---------------------------------------------------------------------------

#[test]
fn spanned_deref() {
    let s = Spanned {
        value: 42,
        span: (0, 2),
    };
    assert_eq!(*s, 42);
}

#[test]
fn spanned_extract_attaches_span() {
    let source = b"hello";
    let node = leaf(0, 5);
    let result =
        <Spanned<String> as Extract<Spanned<String>>>::extract(Some(&node), source, 0, None);
    assert_eq!(result.value, "hello");
    assert_eq!(result.span, (0, 5));
}

#[test]
fn spanned_extract_from_none_uses_last_idx() {
    let source = b"";
    let result = <Spanned<String> as Extract<Spanned<String>>>::extract(None, source, 7, None);
    assert_eq!(result.value, "");
    assert_eq!(result.span, (7, 7));
}

#[test]
fn spanned_index_into_str() {
    let source = "hello world";
    let span = Spanned {
        value: (),
        span: (6, 11),
    };
    assert_eq!(&source[span], "world");
}

#[test]
fn spanned_clone() {
    let s = Spanned {
        value: String::from("test"),
        span: (0, 4),
    };
    let cloned = s.clone();
    assert_eq!(cloned.value, "test");
    assert_eq!(cloned.span, (0, 4));
}

// ---------------------------------------------------------------------------
// 8. Span validation and SpanError
// ---------------------------------------------------------------------------

#[test]
fn span_error_start_greater_than_end() {
    let err = SpanError {
        span: (5, 2),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    assert_eq!(err.to_string(), "Invalid span 5..2: start (5) > end (2)");
}

#[test]
fn span_error_start_out_of_bounds() {
    let err = SpanError {
        span: (20, 25),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    assert!(err.to_string().contains("start (20) > source length (10)"));
}

#[test]
fn span_error_end_out_of_bounds() {
    let err = SpanError {
        span: (0, 100),
        source_len: 50,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    assert!(err.to_string().contains("end (100) > source length (50)"));
}

#[test]
fn span_error_implements_std_error() {
    let err = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    // Verify it implements std::error::Error by using it as a trait object
    let _: &dyn std::error::Error = &err;
}

#[test]
fn span_error_reason_eq() {
    assert_eq!(
        SpanErrorReason::StartGreaterThanEnd,
        SpanErrorReason::StartGreaterThanEnd
    );
    assert_ne!(
        SpanErrorReason::StartOutOfBounds,
        SpanErrorReason::EndOutOfBounds
    );
}

#[test]
fn span_error_clone() {
    let err = SpanError {
        span: (1, 2),
        source_len: 5,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
#[should_panic(expected = "Invalid span")]
fn spanned_index_panics_on_reversed_span() {
    let source = "hello";
    let span = Spanned {
        value: (),
        span: (4, 1),
    };
    let _ = &source[span];
}

#[test]
#[should_panic(expected = "Invalid span")]
fn spanned_index_panics_on_out_of_bounds() {
    let source = "ab";
    let span = Spanned {
        value: (),
        span: (0, 10),
    };
    let _ = &source[span];
}

// ---------------------------------------------------------------------------
// 9. WithLeaf – custom transform extraction
// ---------------------------------------------------------------------------

#[test]
fn with_leaf_extracts_via_transform() {
    let source = b"42";
    let node = leaf(0, 2);
    let transform: &dyn Fn(&str) -> i64 = &|s: &str| s.parse::<i64>().unwrap_or(-1);
    let val = <WithLeaf<i64> as Extract<i64>>::extract(Some(&node), source, 0, Some(transform));
    assert_eq!(val, 42);
}

#[test]
#[should_panic(expected = "Leaf extraction failed")]
fn with_leaf_panics_without_transform() {
    let source = b"hello";
    let node = leaf(0, 5);
    let _: String = <WithLeaf<String> as Extract<String>>::extract(Some(&node), source, 0, None);
}

#[test]
fn with_leaf_empty_node_passes_empty_string() {
    let source = b"abc";
    let transform: &dyn Fn(&str) -> usize = &|s: &str| s.len();
    // Node with empty byte range
    let node = leaf(1, 1);
    let val = <WithLeaf<usize> as Extract<usize>>::extract(Some(&node), source, 0, Some(transform));
    assert_eq!(val, 0);
}

// ---------------------------------------------------------------------------
// 10. ParsedNode API surface tests
// ---------------------------------------------------------------------------

#[test]
fn parsed_node_accessors() {
    let child = leaf(0, 3);
    let parent = make_node(5, vec![child], 0, 3, true);
    assert_eq!(parent.symbol(), 5);
    assert_eq!(parent.start_byte(), 0);
    assert_eq!(parent.end_byte(), 3);
    assert_eq!(parent.child_count(), 1);
    assert!(parent.child(0).is_some());
    assert!(parent.child(1).is_none());
    assert!(parent.is_named());
}

#[test]
fn parsed_node_utf8_text() {
    let source = b"hello";
    let node = leaf(0, 5);
    assert_eq!(node.utf8_text(source).unwrap(), "hello");
}

#[test]
fn parsed_node_child_walker() {
    let a = leaf(0, 1);
    let b = leaf(1, 2);
    let c = leaf(2, 3);
    let parent = make_node(10, vec![a, b, c], 0, 3, true);

    let mut walker = parent.walk();
    assert!(walker.goto_first_child());
    assert_eq!(walker.node().start_byte(), 0);
    assert!(walker.goto_next_sibling());
    assert_eq!(walker.node().start_byte(), 1);
    assert!(walker.goto_next_sibling());
    assert_eq!(walker.node().start_byte(), 2);
    assert!(!walker.goto_next_sibling());
}

// ---------------------------------------------------------------------------
// 11. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn extract_string_with_unicode() {
    let source = "héllo".as_bytes();
    let node = leaf(0, source.len());
    let val = String::extract(Some(&node), source, 0, None);
    assert_eq!(val, "héllo");
}

#[test]
fn spanned_index_empty_span_at_boundary() {
    let source = "abc";
    // Empty span at end of string is valid
    let span = Spanned {
        value: (),
        span: (3, 3),
    };
    assert_eq!(&source[span], "");
}

#[test]
fn extract_all_signed_integer_types() {
    let source = b"-128";
    let node = leaf(0, 4);
    assert_eq!(i8::extract(Some(&node), source, 0, None), -128);

    let source = b"32767";
    let node = leaf(0, 5);
    assert_eq!(i16::extract(Some(&node), source, 0, None), 32767);

    let source = b"100";
    let node = leaf(0, 3);
    assert_eq!(i128::extract(Some(&node), source, 0, None), 100);
    assert_eq!(isize::extract(Some(&node), source, 0, None), 100);
}

#[test]
fn extract_all_unsigned_integer_types() {
    let source = b"255";
    let node = leaf(0, 3);
    assert_eq!(u8::extract(Some(&node), source, 0, None), 255);

    let source = b"65535";
    let node = leaf(0, 5);
    assert_eq!(u16::extract(Some(&node), source, 0, None), 65535);

    let source = b"42";
    let node = leaf(0, 2);
    assert_eq!(u32::extract(Some(&node), source, 0, None), 42);
    assert_eq!(u128::extract(Some(&node), source, 0, None), 42);
    assert_eq!(usize::extract(Some(&node), source, 0, None), 42);
}

#[test]
fn extract_f32_value() {
    let source = b"2.5";
    let node = leaf(0, 3);
    let val = f32::extract(Some(&node), source, 0, None);
    assert!((val - 2.5).abs() < f32::EPSILON);
}

#[test]
fn spanned_index_mut() {
    let mut source = String::from("abcdef");
    let span = Spanned {
        value: (),
        span: (0, 3),
    };
    source.as_mut_str()[span].make_ascii_uppercase();
    assert_eq!(source, "ABCdef");
}

#[test]
fn spanned_debug_format() {
    let s = Spanned {
        value: 42,
        span: (0, 2),
    };
    let debug = format!("{:?}", s);
    assert!(debug.contains("42"));
    assert!(debug.contains("0"));
    assert!(debug.contains("2"));
}
