#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for the `Extract` trait, `Spanned`, span validation,
//! `WithLeaf`, error types, and the built-in `Extract` implementations for
//! primitives, `String`, `Option`, `Box`, and `Vec`.

use adze::pure_parser::{ParsedNode, Point};
use adze::{Extract, SpanError, SpanErrorReason, Spanned, WithLeaf};

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
    let pi_text = std::f64::consts::PI.to_string();
    let source = pi_text.as_bytes();
    let node = leaf(0, source.len());
    let val = f64::extract(Some(&node), source, 0, None);
    assert!((val - std::f64::consts::PI).abs() < f64::EPSILON);
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

// ---------------------------------------------------------------------------
// 12. Trait object patterns – SpanError as dyn Error
// ---------------------------------------------------------------------------

#[test]
fn span_error_as_dyn_error_source_is_none() {
    use std::error::Error;
    let err = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    let dyn_err: &dyn Error = &err;
    assert!(dyn_err.source().is_none());
}

#[test]
fn span_error_debug_contains_fields() {
    let err = SpanError {
        span: (1, 2),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("SpanError"));
    assert!(debug.contains("EndOutOfBounds"));
}

// ---------------------------------------------------------------------------
// 13. Error types – ParseError and ParseErrorReason
// ---------------------------------------------------------------------------

#[test]
fn parse_error_unexpected_token_debug() {
    use adze::errors::{ParseError, ParseErrorReason};
    let err = ParseError {
        reason: ParseErrorReason::UnexpectedToken("foo".into()),
        start: 0,
        end: 3,
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("UnexpectedToken"));
    assert!(debug.contains("foo"));
}

#[test]
fn parse_error_missing_token_debug() {
    use adze::errors::{ParseError, ParseErrorReason};
    let err = ParseError {
        reason: ParseErrorReason::MissingToken(";".into()),
        start: 5,
        end: 5,
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("MissingToken"));
}

#[test]
fn parse_error_failed_node_nested() {
    use adze::errors::{ParseError, ParseErrorReason};
    let inner = ParseError {
        reason: ParseErrorReason::UnexpectedToken("x".into()),
        start: 0,
        end: 1,
    };
    let outer = ParseError {
        reason: ParseErrorReason::FailedNode(vec![inner]),
        start: 0,
        end: 5,
    };
    match &outer.reason {
        ParseErrorReason::FailedNode(v) => assert_eq!(v.len(), 1),
        _ => panic!("expected FailedNode"),
    }
}

// ---------------------------------------------------------------------------
// 14. Nested generic extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_option_box_string() {
    let source = b"nested";
    let node = leaf(0, 6);
    let val = <Option<Box<String>> as Extract<Option<Box<String>>>>::extract(
        Some(&node),
        source,
        0,
        None,
    );
    assert_eq!(**val.as_ref().unwrap(), "nested");
}

#[test]
fn extract_option_box_none() {
    let source = b"";
    let val = <Option<Box<String>> as Extract<Option<Box<String>>>>::extract(None, source, 0, None);
    assert!(val.is_none());
}

#[test]
fn extract_box_option_some() {
    let source = b"77";
    let node = leaf(0, 2);
    let val =
        <Box<Option<i32>> as Extract<Box<Option<i32>>>>::extract(Some(&node), source, 0, None);
    assert_eq!(*val, Some(77));
}

#[test]
fn extract_box_box_string() {
    let source = b"deep";
    let node = leaf(0, 4);
    let val =
        <Box<Box<String>> as Extract<Box<Box<String>>>>::extract(Some(&node), source, 0, None);
    assert_eq!(**val, "deep");
}

// ---------------------------------------------------------------------------
// 15. Unicode edge cases
// ---------------------------------------------------------------------------

#[test]
fn extract_string_multibyte_emoji() {
    let source = "🦀🦀🦀".as_bytes();
    let node = leaf(0, source.len());
    let val = String::extract(Some(&node), source, 0, None);
    assert_eq!(val, "🦀🦀🦀");
}

#[test]
fn extract_string_cjk() {
    let source = "漢字テスト".as_bytes();
    let node = leaf(0, source.len());
    let val = String::extract(Some(&node), source, 0, None);
    assert_eq!(val, "漢字テスト");
}

#[test]
fn extract_string_mixed_ascii_unicode() {
    let source = "abc_αβγ_🎉".as_bytes();
    let node = leaf(0, source.len());
    let val = String::extract(Some(&node), source, 0, None);
    assert_eq!(val, "abc_αβγ_🎉");
}

// ---------------------------------------------------------------------------
// 16. Empty input edge cases
// ---------------------------------------------------------------------------

#[test]
fn extract_string_empty_source() {
    let source = b"";
    let node = leaf(0, 0);
    let val = String::extract(Some(&node), source, 0, None);
    assert_eq!(val, "");
}

#[test]
fn extract_vec_empty_parent_no_children() {
    let source = b"";
    let parent = make_node(10, vec![], 0, 0, true);
    let val = <Vec<String> as Extract<Vec<String>>>::extract(Some(&parent), source, 0, None);
    assert!(val.is_empty());
}

// ---------------------------------------------------------------------------
// 17. Re-exports smoke tests
// ---------------------------------------------------------------------------

#[test]
fn reexport_extract_trait_accessible() {
    // Verify Extract is accessible from adze root
    fn _assert_extract<T: Extract<String>>() {}
    _assert_extract::<String>();
}

#[test]
fn reexport_spanned_accessible() {
    let _: Spanned<i32> = Spanned {
        value: 0,
        span: (0, 0),
    };
}

#[test]
fn reexport_with_leaf_accessible() {
    // WithLeaf is accessible from adze root
    let _phantom: std::marker::PhantomData<WithLeaf<String>> = std::marker::PhantomData;
}

#[test]
fn reexport_span_error_types_accessible() {
    let _: SpanError = SpanError {
        span: (0, 0),
        source_len: 0,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
}

// ---------------------------------------------------------------------------
// 18. HAS_CONFLICTS default
// ---------------------------------------------------------------------------

#[test]
fn has_conflicts_default_is_false() {
    assert!(!<String as Extract<String>>::HAS_CONFLICTS);
    assert!(!<i32 as Extract<i32>>::HAS_CONFLICTS);
    assert!(!<() as Extract<()>>::HAS_CONFLICTS);
}

// ---------------------------------------------------------------------------
// 19. Spanned with various inner types
// ---------------------------------------------------------------------------

#[test]
fn spanned_extract_i32() {
    let source = b"99";
    let node = leaf(0, 2);
    let result = <Spanned<i32> as Extract<Spanned<i32>>>::extract(Some(&node), source, 0, None);
    assert_eq!(result.value, 99);
    assert_eq!(result.span, (0, 2));
}

#[test]
fn spanned_extract_option_some() {
    let source = b"hi";
    let node = leaf(0, 2);
    let result = <Spanned<Option<String>> as Extract<Spanned<Option<String>>>>::extract(
        Some(&node),
        source,
        0,
        None,
    );
    assert_eq!(result.value, Some("hi".to_string()));
    assert_eq!(result.span, (0, 2));
}

#[test]
fn spanned_deref_to_inner() {
    let s = Spanned {
        value: String::from("abc"),
        span: (0, 3),
    };
    // Deref gives &String
    assert_eq!(s.len(), 3);
}

// ---------------------------------------------------------------------------
// 20. WithLeaf edge cases
// ---------------------------------------------------------------------------

#[test]
fn with_leaf_transform_returns_custom_type() {
    let source = b"hello world";
    let node = leaf(0, 5);
    let transform: &dyn Fn(&str) -> Vec<char> = &|s: &str| s.chars().collect();
    let val = <WithLeaf<Vec<char>> as Extract<Vec<char>>>::extract(
        Some(&node),
        source,
        0,
        Some(transform),
    );
    assert_eq!(val, vec!['h', 'e', 'l', 'l', 'o']);
}

#[test]
fn with_leaf_transform_on_numeric_string() {
    let source = b"  42  ";
    let node = leaf(0, 6);
    let transform: &dyn Fn(&str) -> i32 = &|s: &str| s.trim().parse().unwrap_or(0);
    let val = <WithLeaf<i32> as Extract<i32>>::extract(Some(&node), source, 0, Some(transform));
    assert_eq!(val, 42);
}

// ---------------------------------------------------------------------------
// 21. ParsedNode deeper integration
// ---------------------------------------------------------------------------

#[test]
fn parsed_node_nested_children() {
    let grandchild = leaf(0, 1);
    let child = make_node(2, vec![grandchild], 0, 1, true);
    let parent = make_node(3, vec![child], 0, 1, true);
    assert_eq!(parent.child_count(), 1);
    let c = parent.child(0).unwrap();
    assert_eq!(c.child_count(), 1);
}

#[test]
fn parsed_node_is_not_error_by_default() {
    let node = leaf(0, 1);
    assert!(!node.is_error);
    assert!(!node.is_missing);
    assert!(!node.is_extra);
}

#[test]
fn parsed_node_start_end_points() {
    let node = leaf(5, 10);
    assert_eq!(node.start_point.row, 0);
    assert_eq!(node.start_point.column, 5);
    assert_eq!(node.end_point.row, 0);
    assert_eq!(node.end_point.column, 10);
}

#[test]
fn parsed_node_symbol_accessor() {
    let node = make_node(42, vec![], 0, 0, false);
    assert_eq!(node.symbol(), 42);
    assert!(!node.is_named());
}

// ---------------------------------------------------------------------------
// 22. Extract with offset (last_idx) parameter
// ---------------------------------------------------------------------------

#[test]
fn extract_string_ignores_last_idx() {
    let source = b"abcdef";
    let node = leaf(2, 5);
    // last_idx shouldn't affect String extraction when node is present
    let val = String::extract(Some(&node), source, 99, None);
    assert_eq!(val, "cde");
}

#[test]
fn spanned_none_uses_last_idx_offset() {
    let source = b"hello";
    let result = <Spanned<String> as Extract<Spanned<String>>>::extract(None, source, 42, None);
    assert_eq!(result.span, (42, 42));
}

// ---------------------------------------------------------------------------
// 23. Vec extraction with single child
// ---------------------------------------------------------------------------

#[test]
fn extract_vec_single_child() {
    let source = b"x";
    let child = leaf(0, 1);
    let parent = make_node(10, vec![child], 0, 1, true);
    let val = <Vec<String> as Extract<Vec<String>>>::extract(Some(&parent), source, 0, None);
    assert_eq!(val, vec!["x"]);
}
