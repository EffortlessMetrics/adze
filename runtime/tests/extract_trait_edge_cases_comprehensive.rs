#![allow(clippy::needless_range_loop)]
//! Edge-case tests for the `Extract` trait and its built-in implementations.
//!
//! Covers: empty input, None nodes, nested generics (`Vec<Option<T>>`,
//! `Option<Box<T>>`), `Box<T>` recursion, `WithLeaf` panics, `Spanned`
//! edge cases, numeric overflow/underflow, Unicode boundaries, large inputs,
//! error type formatting, and trait-level properties.

use adze::errors::{ParseError, ParseErrorReason};
use adze::pure_parser::{ParsedNode, Point};
use adze::{Extract, SpanError, SpanErrorReason, Spanned, WithLeaf};
use std::mem::MaybeUninit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Build a `ParsedNode` via `MaybeUninit` to work around `language`
/// being `pub(crate)`.
fn make_node(
    symbol: u16,
    children: Vec<ParsedNode>,
    start: usize,
    end: usize,
    is_named: bool,
) -> ParsedNode {
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

// ---------------------------------------------------------------------------
// 1. Extract from empty input
// ---------------------------------------------------------------------------

#[test]
fn extract_string_empty_source_none_node() {
    let result = String::extract(None, b"", 0, None);
    assert_eq!(result, "");
}

#[test]
fn extract_string_empty_source_with_zero_range_node() {
    let node = leaf(0, 0);
    let result = String::extract(Some(&node), b"", 0, None);
    assert_eq!(result, "");
}

#[test]
fn extract_i32_empty_source_none_node() {
    let result = i32::extract(None, b"", 0, None);
    assert_eq!(result, 0);
}

#[test]
fn extract_unit_empty_source() {
    <() as Extract<()>>::extract(None, b"", 0, None);
    // should not panic
}

#[test]
fn extract_vec_empty_source_none() {
    let result = <Vec<String> as Extract<Vec<String>>>::extract(None, b"", 0, None);
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Extract with mismatched / unparseable content
// ---------------------------------------------------------------------------

#[test]
fn extract_i32_from_float_text_returns_default() {
    let source = b"3.14";
    let node = leaf(0, 4);
    let val = i32::extract(Some(&node), source, 0, None);
    assert_eq!(val, 0, "i32 cannot parse '3.14', should default to 0");
}

#[test]
fn extract_u64_from_negative_returns_default() {
    let source = b"-1";
    let node = leaf(0, 2);
    let val = u64::extract(Some(&node), source, 0, None);
    assert_eq!(val, 0, "u64 cannot parse '-1', should default to 0");
}

#[test]
fn extract_bool_from_garbage_returns_default() {
    let source = b"maybe";
    let node = leaf(0, 5);
    let val = bool::extract(Some(&node), source, 0, None);
    assert!(!val, "bool cannot parse 'maybe', should default to false");
}

#[test]
fn extract_f64_from_text_returns_default() {
    let source = b"not_a_number";
    let node = leaf(0, 12);
    let val = f64::extract(Some(&node), source, 0, None);
    assert_eq!(val, 0.0);
}

#[test]
fn extract_i8_overflow_returns_default() {
    let source = b"256";
    let node = leaf(0, 3);
    let val = i8::extract(Some(&node), source, 0, None);
    assert_eq!(val, 0, "256 overflows i8, should default to 0");
}

#[test]
fn extract_u8_overflow_returns_default() {
    let source = b"999";
    let node = leaf(0, 3);
    let val = u8::extract(Some(&node), source, 0, None);
    assert_eq!(val, 0, "999 overflows u8, should default to 0");
}

// ---------------------------------------------------------------------------
// 3. Extract with partial matches / substrings
// ---------------------------------------------------------------------------

#[test]
fn extract_string_partial_range() {
    let source = b"hello world";
    let node = leaf(6, 11);
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result, "world");
}

#[test]
fn extract_i32_with_leading_whitespace_returns_default() {
    let source = b" 42";
    let node = leaf(0, 3);
    // " 42" includes leading space — str::parse::<i32>(" 42") fails
    let val = i32::extract(Some(&node), source, 0, None);
    assert_eq!(val, 0);
}

#[test]
fn extract_i32_from_middle_of_source() {
    let source = b"abc42def";
    let node = leaf(3, 5);
    let val = i32::extract(Some(&node), source, 0, None);
    assert_eq!(val, 42);
}

// ---------------------------------------------------------------------------
// 4. Nested types: Vec<Option<T>>, Option<Box<T>>, Box<Option<T>>
// ---------------------------------------------------------------------------

#[test]
fn extract_option_string_none_node() {
    let result =
        <Option<String> as Extract<Option<String>>>::extract(None, b"anything", 0, None);
    assert_eq!(result, None);
}

#[test]
fn extract_option_string_some_node() {
    let source = b"hi";
    let node = leaf(0, 2);
    let result =
        <Option<String> as Extract<Option<String>>>::extract(Some(&node), source, 0, None);
    assert_eq!(result, Some("hi".to_string()));
}

#[test]
fn extract_option_i32_unparseable_gives_some_zero() {
    // Option wrapping: node is Some, but inner parse fails → Some(0)
    let source = b"xyz";
    let node = leaf(0, 3);
    let result =
        <Option<i32> as Extract<Option<i32>>>::extract(Some(&node), source, 0, None);
    assert_eq!(result, Some(0));
}

#[test]
fn extract_box_string_from_none() {
    let result =
        <Box<String> as Extract<Box<String>>>::extract(None, b"", 0, None);
    assert_eq!(*result, "");
}

#[test]
fn extract_box_option_string_none_node() {
    let result = <Box<Option<String>> as Extract<Box<Option<String>>>>::extract(
        None, b"", 0, None,
    );
    assert_eq!(*result, None);
}

#[test]
fn extract_box_option_string_some_node() {
    let source = b"boxed";
    let node = leaf(0, 5);
    let result = <Box<Option<String>> as Extract<Box<Option<String>>>>::extract(
        Some(&node), source, 0, None,
    );
    assert_eq!(*result, Some("boxed".to_string()));
}

#[test]
fn extract_option_box_i32_some() {
    let source = b"7";
    let node = leaf(0, 1);
    let result = <Option<Box<i32>> as Extract<Option<Box<i32>>>>::extract(
        Some(&node), source, 0, None,
    );
    assert_eq!(result, Some(Box::new(7)));
}

#[test]
fn extract_option_box_i32_none() {
    let result = <Option<Box<i32>> as Extract<Option<Box<i32>>>>::extract(
        None, b"", 0, None,
    );
    assert_eq!(result, None);
}

// ---------------------------------------------------------------------------
// 5. Box<T> recursion depth
// ---------------------------------------------------------------------------

#[test]
fn extract_deeply_nested_boxes() {
    // Box<Box<Box<String>>>
    let source = b"deep";
    let node = leaf(0, 4);
    let result = <Box<Box<Box<String>>> as Extract<Box<Box<Box<String>>>>>::extract(
        Some(&node), source, 0, None,
    );
    assert_eq!(***result, "deep");
}

#[test]
fn extract_deeply_nested_boxes_none() {
    let result = <Box<Box<Box<i32>>> as Extract<Box<Box<Box<i32>>>>>::extract(
        None, b"", 0, None,
    );
    assert_eq!(***result, 0);
}

// ---------------------------------------------------------------------------
// 6. Vec edge cases
// ---------------------------------------------------------------------------

#[test]
fn extract_vec_single_child() {
    let source = b"x";
    let child = leaf(0, 1);
    let parent = make_node(10, vec![child], 0, 1, true);
    let result =
        <Vec<String> as Extract<Vec<String>>>::extract(Some(&parent), source, 0, None);
    assert_eq!(result, vec!["x"]);
}

#[test]
fn extract_vec_no_children() {
    let source = b"";
    let parent = make_node(10, vec![], 0, 0, true);
    let result =
        <Vec<String> as Extract<Vec<String>>>::extract(Some(&parent), source, 0, None);
    assert!(result.is_empty());
}

#[test]
fn extract_vec_of_option_string() {
    // Vec<Option<String>> with two children
    let source = b"ab";
    let child_a = leaf(0, 1);
    let child_b = leaf(1, 2);
    let parent = make_node(10, vec![child_a, child_b], 0, 2, true);
    let result = <Vec<Option<String>> as Extract<Vec<Option<String>>>>::extract(
        Some(&parent), source, 0, None,
    );
    assert_eq!(result, vec![Some("a".to_string()), Some("b".to_string())]);
}

// ---------------------------------------------------------------------------
// 7. WithLeaf extraction and panic
// ---------------------------------------------------------------------------

#[test]
fn with_leaf_extract_with_transform() {
    let source = b"42";
    let node = leaf(0, 2);
    let transform: Box<dyn Fn(&str) -> usize> = Box::new(|s: &str| s.len());
    let result = <WithLeaf<usize> as Extract<usize>>::extract(
        Some(&node),
        source,
        0,
        Some(transform.as_ref()),
    );
    assert_eq!(result, 2);
}

#[test]
#[should_panic(expected = "Leaf extraction failed")]
fn with_leaf_extract_without_transform_panics() {
    let source = b"text";
    let node = leaf(0, 4);
    let _: String = <WithLeaf<String> as Extract<String>>::extract(
        Some(&node), source, 0, None,
    );
}

#[test]
fn with_leaf_extract_from_none_with_transform() {
    let source = b"";
    let transform: Box<dyn Fn(&str) -> String> =
        Box::new(|s: &str| format!("got:{}", s));
    let result = <WithLeaf<String> as Extract<String>>::extract(
        None,
        source,
        0,
        Some(transform.as_ref()),
    );
    // None node → unwrap_or_default → empty string → transform applied
    assert_eq!(result, "got:");
}

// ---------------------------------------------------------------------------
// 8. Spanned edge cases
// ---------------------------------------------------------------------------

#[test]
fn spanned_extract_none_uses_last_idx() {
    let source = b"hello";
    let result = <Spanned<String> as Extract<Spanned<String>>>::extract(
        None, source, 3, None,
    );
    assert_eq!(result.value, "");
    assert_eq!(result.span, (3, 3));
}

#[test]
fn spanned_extract_zero_width_node() {
    let source = b"abc";
    let node = leaf(2, 2);
    let result = <Spanned<String> as Extract<Spanned<String>>>::extract(
        Some(&node), source, 0, None,
    );
    assert_eq!(result.value, "");
    assert_eq!(result.span, (2, 2));
}

#[test]
fn spanned_clone() {
    let s = Spanned {
        value: "hello".to_string(),
        span: (0, 5),
    };
    let cloned = s.clone();
    assert_eq!(cloned.value, "hello");
    assert_eq!(cloned.span, (0, 5));
}

// ---------------------------------------------------------------------------
// 9. Error types and formatting
// ---------------------------------------------------------------------------

#[test]
fn parse_error_unexpected_token_debug() {
    let err = ParseError {
        reason: ParseErrorReason::UnexpectedToken("@".to_string()),
        start: 0,
        end: 1,
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("UnexpectedToken"));
    assert!(debug.contains("@"));
}

#[test]
fn parse_error_missing_token_debug() {
    let err = ParseError {
        reason: ParseErrorReason::MissingToken("identifier".to_string()),
        start: 5,
        end: 5,
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("MissingToken"));
    assert!(debug.contains("identifier"));
}

#[test]
fn parse_error_failed_node_with_children() {
    let inner = ParseError {
        reason: ParseErrorReason::UnexpectedToken("!".to_string()),
        start: 0,
        end: 1,
    };
    let outer = ParseError {
        reason: ParseErrorReason::FailedNode(vec![inner]),
        start: 0,
        end: 5,
    };
    let debug = format!("{:?}", outer);
    assert!(debug.contains("FailedNode"));
    assert!(debug.contains("UnexpectedToken"));
}

#[test]
fn parse_error_failed_node_empty_children() {
    let err = ParseError {
        reason: ParseErrorReason::FailedNode(vec![]),
        start: 0,
        end: 0,
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("FailedNode"));
}

#[test]
fn span_error_display_all_variants() {
    let e1 = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    assert!(e1.to_string().contains("start (5) > end (3)"));

    let e2 = SpanError {
        span: (20, 25),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    assert!(e2.to_string().contains("start (20) > source length (10)"));

    let e3 = SpanError {
        span: (0, 100),
        source_len: 50,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    assert!(e3.to_string().contains("end (100) > source length (50)"));
}

#[test]
fn span_error_is_std_error() {
    let e = SpanError {
        span: (1, 0),
        source_len: 5,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    // Verify it implements std::error::Error (compile-time check via trait object)
    let _: &dyn std::error::Error = &e;
}

// ---------------------------------------------------------------------------
// 10. Unicode boundary extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_string_multibyte_utf8() {
    let source = "héllo".as_bytes();
    // 'é' is 2 bytes in UTF-8, so "héllo" = [h, 0xC3, 0xA9, l, l, o] = 6 bytes
    let node = leaf(0, 6);
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result, "héllo");
}

#[test]
fn extract_string_emoji() {
    let source = "🦀".as_bytes(); // 4 bytes in UTF-8
    let node = leaf(0, 4);
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result, "🦀");
}

#[test]
fn extract_string_cjk() {
    let source = "你好".as_bytes(); // 3 bytes each = 6 bytes
    let node = leaf(0, 6);
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result, "你好");
}

// ---------------------------------------------------------------------------
// 11. Performance with large inputs
// ---------------------------------------------------------------------------

#[test]
fn extract_string_large_input() {
    let large = "x".repeat(100_000);
    let source = large.as_bytes();
    let node = leaf(0, source.len());
    let result = String::extract(Some(&node), source, 0, None);
    assert_eq!(result.len(), 100_000);
}

#[test]
fn extract_i64_max_value() {
    let text = i64::MAX.to_string();
    let source = text.as_bytes();
    let node = leaf(0, source.len());
    let val = i64::extract(Some(&node), source, 0, None);
    assert_eq!(val, i64::MAX);
}

#[test]
fn extract_i64_min_value() {
    let text = i64::MIN.to_string();
    let source = text.as_bytes();
    let node = leaf(0, source.len());
    let val = i64::extract(Some(&node), source, 0, None);
    assert_eq!(val, i64::MIN);
}

// ---------------------------------------------------------------------------
// 12. All primitive types with None
// ---------------------------------------------------------------------------

#[test]
fn extract_all_primitives_none_return_default() {
    assert_eq!(i8::extract(None, b"", 0, None), 0i8);
    assert_eq!(i16::extract(None, b"", 0, None), 0i16);
    assert_eq!(i32::extract(None, b"", 0, None), 0i32);
    assert_eq!(i64::extract(None, b"", 0, None), 0i64);
    assert_eq!(i128::extract(None, b"", 0, None), 0i128);
    assert_eq!(isize::extract(None, b"", 0, None), 0isize);
    assert_eq!(u8::extract(None, b"", 0, None), 0u8);
    assert_eq!(u16::extract(None, b"", 0, None), 0u16);
    assert_eq!(u32::extract(None, b"", 0, None), 0u32);
    assert_eq!(u64::extract(None, b"", 0, None), 0u64);
    assert_eq!(u128::extract(None, b"", 0, None), 0u128);
    assert_eq!(usize::extract(None, b"", 0, None), 0usize);
    assert_eq!(f32::extract(None, b"", 0, None), 0.0f32);
    assert_eq!(f64::extract(None, b"", 0, None), 0.0f64);
    assert!(!bool::extract(None, b"", 0, None));
}

// ---------------------------------------------------------------------------
// 13. Extract trait associated types
// ---------------------------------------------------------------------------

#[test]
fn extract_has_conflicts_default_is_false() {
    // The default for HAS_CONFLICTS is false
    assert!(!<String as Extract<String>>::HAS_CONFLICTS);
    assert!(!<i32 as Extract<i32>>::HAS_CONFLICTS);
    assert!(!<() as Extract<()>>::HAS_CONFLICTS);
}

// ---------------------------------------------------------------------------
// 14. Spanned indexing into source
// ---------------------------------------------------------------------------

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
#[should_panic(expected = "Invalid span")]
fn spanned_index_start_exceeds_end() {
    let source = "hello";
    let span = Spanned {
        value: (),
        span: (3, 1),
    };
    let _ = &source[span];
}

#[test]
fn extract_f32_scientific_notation() {
    let source = b"1.5e2";
    let node = leaf(0, 5);
    let val = f32::extract(Some(&node), source, 0, None);
    assert!((val - 150.0).abs() < f32::EPSILON);
}
