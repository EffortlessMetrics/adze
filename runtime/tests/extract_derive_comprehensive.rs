#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for `Extract` trait implementations in the adze runtime.
//!
//! Covers: primitive types (String, bool, numerics), Option<T>, Vec<T>, Box<T>,
//! trait object safety, custom Extract implementations, error handling,
//! Spanned wrapper, and ParsedNode as Extract input.

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

fn named_parent(children: Vec<ParsedNode>, start: usize, end: usize) -> ParsedNode {
    make_node(2, children, start, end, true)
}

// ---------------------------------------------------------------------------
// 1. Extract for primitive types — String
// ---------------------------------------------------------------------------

#[test]
fn extract_string_yields_text_from_node() {
    let src = b"hello world";
    let node = leaf(0, 5);
    assert_eq!(String::extract(Some(&node), src, 0, None), "hello");
}

#[test]
fn extract_string_none_node_yields_empty() {
    assert_eq!(String::extract(None, b"abc", 0, None), "");
}

#[test]
fn extract_string_mid_range() {
    let src = b"abcdef";
    let node = leaf(2, 5);
    assert_eq!(String::extract(Some(&node), src, 0, None), "cde");
}

// ---------------------------------------------------------------------------
// 2. Extract for primitive types — bool
// ---------------------------------------------------------------------------

#[test]
fn extract_bool_true() {
    let src = b"true";
    let node = leaf(0, 4);
    assert!(bool::extract(Some(&node), src, 0, None));
}

#[test]
fn extract_bool_false() {
    let src = b"false";
    let node = leaf(0, 5);
    assert!(!bool::extract(Some(&node), src, 0, None));
}

#[test]
fn extract_bool_none_yields_default() {
    // Default for bool is false
    assert!(!bool::extract(None, b"", 0, None));
}

#[test]
fn extract_bool_invalid_text_yields_default() {
    let src = b"notabool";
    let node = leaf(0, 8);
    assert!(!bool::extract(Some(&node), src, 0, None));
}

// ---------------------------------------------------------------------------
// 3. Extract for numeric primitives
// ---------------------------------------------------------------------------

#[test]
fn extract_i32_from_node() {
    let src = b"42";
    let node = leaf(0, 2);
    assert_eq!(i32::extract(Some(&node), src, 0, None), 42);
}

#[test]
fn extract_i32_negative() {
    let src = b"-7";
    let node = leaf(0, 2);
    assert_eq!(i32::extract(Some(&node), src, 0, None), -7);
}

#[test]
fn extract_f64_from_node() {
    let src = b"3.14";
    let node = leaf(0, 4);
    let val = f64::extract(Some(&node), src, 0, None);
    assert!((val - 3.14).abs() < f64::EPSILON);
}

#[test]
fn extract_u64_none_yields_zero() {
    assert_eq!(u64::extract(None, b"", 0, None), 0);
}

#[test]
fn extract_i32_unparseable_yields_default() {
    let src = b"xyz";
    let node = leaf(0, 3);
    assert_eq!(i32::extract(Some(&node), src, 0, None), 0);
}

// ---------------------------------------------------------------------------
// 4. Extract for Option<T>
// ---------------------------------------------------------------------------

#[test]
fn extract_option_string_some() {
    let src = b"hello";
    let node = leaf(0, 5);
    let result = Option::<String>::extract(Some(&node), src, 0, None);
    assert_eq!(result, Some("hello".to_string()));
}

#[test]
fn extract_option_string_none_node() {
    let result = Option::<String>::extract(None, b"abc", 0, None);
    assert_eq!(result, None);
}

#[test]
fn extract_option_i32_some() {
    let src = b"99";
    let node = leaf(0, 2);
    let result = Option::<i32>::extract(Some(&node), src, 0, None);
    assert_eq!(result, Some(99));
}

#[test]
fn extract_option_i32_none_node() {
    let result = Option::<i32>::extract(None, b"", 0, None);
    assert_eq!(result, None);
}

// ---------------------------------------------------------------------------
// 5. Extract for Vec<T>
// ---------------------------------------------------------------------------

#[test]
fn extract_vec_string_with_children() {
    // Source: "ab cd"
    let src = b"ab cd";
    let child0 = leaf(0, 2); // "ab"
    let child1 = leaf(3, 5); // "cd"
    let parent = named_parent(vec![child0, child1], 0, 5);
    let result = Vec::<String>::extract(Some(&parent), src, 0, None);
    assert_eq!(result, vec!["ab".to_string(), "cd".to_string()]);
}

#[test]
fn extract_vec_none_yields_empty() {
    let result = Vec::<String>::extract(None, b"", 0, None);
    assert!(result.is_empty());
}

#[test]
fn extract_vec_no_children_yields_empty() {
    let src = b"abc";
    let parent = named_parent(vec![], 0, 3);
    let result = Vec::<String>::extract(Some(&parent), src, 0, None);
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// 6. Extract for Box<T>
// ---------------------------------------------------------------------------

#[test]
fn extract_box_string() {
    let src = b"boxed";
    let node = leaf(0, 5);
    let result = Box::<String>::extract(Some(&node), src, 0, None);
    assert_eq!(*result, "boxed");
}

#[test]
fn extract_box_i32() {
    let src = b"123";
    let node = leaf(0, 3);
    let result = Box::<i32>::extract(Some(&node), src, 0, None);
    assert_eq!(*result, 123);
}

#[test]
fn extract_box_none_node_yields_default() {
    let result = Box::<String>::extract(None, b"", 0, None);
    assert_eq!(*result, "");
}

// ---------------------------------------------------------------------------
// 7. Trait object safety — Extract is not object-safe (has generic param
//    and associated types), but we verify trait bounds compile correctly
// ---------------------------------------------------------------------------

fn accepts_extract_string<T: Extract<String>>(_node: Option<&ParsedNode>, _src: &[u8]) {}
fn accepts_extract_i32<T: Extract<i32>>(_node: Option<&ParsedNode>, _src: &[u8]) {}

#[test]
fn trait_bound_string_compiles() {
    let src = b"test";
    let node = leaf(0, 4);
    accepts_extract_string::<String>(Some(&node), src);
}

#[test]
fn trait_bound_i32_compiles() {
    let src = b"42";
    let node = leaf(0, 2);
    accepts_extract_i32::<i32>(Some(&node), src);
}

#[test]
fn trait_bound_option_compiles() {
    fn accepts_extract_option<T: Extract<Option<String>>>(
        _node: Option<&ParsedNode>,
        _src: &[u8],
    ) {
    }
    accepts_extract_option::<Option<String>>(None, b"");
}

// ---------------------------------------------------------------------------
// 8. Custom Extract implementations — WithLeaf
// ---------------------------------------------------------------------------

#[test]
fn with_leaf_custom_transform() {
    let src = b"42";
    let node = leaf(0, 2);
    let transform: Box<dyn Fn(&str) -> i64> = Box::new(|s: &str| s.parse::<i64>().unwrap() * 10);
    let result = WithLeaf::<i64>::extract(Some(&node), src, 0, Some(&*transform));
    assert_eq!(result, 420);
}

#[test]
fn with_leaf_string_uppercase() {
    let src = b"hello";
    let node = leaf(0, 5);
    let transform: Box<dyn Fn(&str) -> String> = Box::new(|s: &str| s.to_uppercase());
    let result = WithLeaf::<String>::extract(Some(&node), src, 0, Some(&*transform));
    assert_eq!(result, "HELLO");
}

#[test]
#[should_panic(expected = "Leaf extraction failed")]
fn with_leaf_panics_without_transform() {
    let src = b"data";
    let node = leaf(0, 4);
    let _ = WithLeaf::<String>::extract(Some(&node), src, 0, None);
}

// ---------------------------------------------------------------------------
// 9. Extract error handling
// ---------------------------------------------------------------------------

#[test]
fn parse_error_unexpected_token_fields() {
    let err = ParseError {
        reason: ParseErrorReason::UnexpectedToken("@".to_string()),
        start: 5,
        end: 6,
    };
    assert_eq!(err.start, 5);
    assert_eq!(err.end, 6);
    match &err.reason {
        ParseErrorReason::UnexpectedToken(tok) => assert_eq!(tok, "@"),
        _ => panic!("expected UnexpectedToken"),
    }
}

#[test]
fn parse_error_missing_token_fields() {
    let err = ParseError {
        reason: ParseErrorReason::MissingToken("identifier".to_string()),
        start: 0,
        end: 0,
    };
    match &err.reason {
        ParseErrorReason::MissingToken(tok) => assert_eq!(tok, "identifier"),
        _ => panic!("expected MissingToken"),
    }
}

#[test]
fn parse_error_failed_node_empty() {
    let err = ParseError {
        reason: ParseErrorReason::FailedNode(vec![]),
        start: 0,
        end: 10,
    };
    match &err.reason {
        ParseErrorReason::FailedNode(inner) => assert!(inner.is_empty()),
        _ => panic!("expected FailedNode"),
    }
}

#[test]
fn parse_error_debug_format() {
    let err = ParseError {
        reason: ParseErrorReason::UnexpectedToken("!!".to_string()),
        start: 3,
        end: 5,
    };
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("UnexpectedToken"));
    assert!(dbg.contains("!!"));
}

// ---------------------------------------------------------------------------
// 10. Extract with Spanned wrapper
// ---------------------------------------------------------------------------

#[test]
fn spanned_string_extract() {
    let src = b"hello world";
    let node = leaf(6, 11);
    let result = Spanned::<String>::extract(Some(&node), src, 0, None);
    assert_eq!(result.value, "world");
    assert_eq!(result.span, (6, 11));
}

#[test]
fn spanned_none_uses_last_idx() {
    let result = Spanned::<String>::extract(None, b"abc", 7, None);
    assert_eq!(result.value, "");
    assert_eq!(result.span, (7, 7));
}

#[test]
fn spanned_deref_to_inner() {
    let spanned = Spanned {
        value: "test".to_string(),
        span: (0, 4),
    };
    // Deref coercion
    let s: &str = &spanned;
    assert_eq!(s, "test");
}

#[test]
fn spanned_index_into_source() {
    let source = "abcdefgh";
    let spanned = Spanned {
        value: (),
        span: (2, 5),
    };
    assert_eq!(&source[spanned], "cde");
}

// ---------------------------------------------------------------------------
// 11. SpanError validation
// ---------------------------------------------------------------------------

#[test]
fn span_error_start_greater_than_end() {
    let err = SpanError {
        span: (5, 3),
        source_len: 10,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("start (5) > end (3)"));
}

#[test]
fn span_error_start_out_of_bounds() {
    let err = SpanError {
        span: (20, 25),
        source_len: 10,
        reason: SpanErrorReason::StartOutOfBounds,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("start (20) > source length (10)"));
}

#[test]
fn span_error_end_out_of_bounds() {
    let err = SpanError {
        span: (0, 50),
        source_len: 10,
        reason: SpanErrorReason::EndOutOfBounds,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("end (50) > source length (10)"));
}

#[test]
fn span_error_implements_std_error() {
    let err = SpanError {
        span: (0, 0),
        source_len: 0,
        reason: SpanErrorReason::StartGreaterThanEnd,
    };
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

// ---------------------------------------------------------------------------
// 12. ParsedNode as Extract input — field verification
// ---------------------------------------------------------------------------

#[test]
fn parsed_node_fields_accessible() {
    let node = make_node(42, vec![], 10, 20, true);
    assert_eq!(node.symbol, 42);
    assert_eq!(node.start_byte, 10);
    assert_eq!(node.end_byte, 20);
    assert!(node.is_named);
    assert!(!node.is_error);
    assert!(!node.is_missing);
    assert!(!node.is_extra);
    assert!(node.children.is_empty());
    assert_eq!(node.field_id, None);
}

#[test]
fn parsed_node_with_children() {
    let c1 = leaf(0, 3);
    let c2 = leaf(4, 7);
    let parent = named_parent(vec![c1, c2], 0, 7);
    assert_eq!(parent.children.len(), 2);
    assert_eq!(parent.children[0].start_byte, 0);
    assert_eq!(parent.children[1].start_byte, 4);
}

// ---------------------------------------------------------------------------
// 13. Extract for unit type
// ---------------------------------------------------------------------------

#[test]
fn extract_unit_from_some() {
    let node = leaf(0, 1);
    <()>::extract(Some(&node), b"x", 0, None);
    // No panic = success
}

#[test]
fn extract_unit_from_none() {
    <()>::extract(None, b"", 0, None);
    // No panic = success
}

// ---------------------------------------------------------------------------
// 14. Nested generic extraction
// ---------------------------------------------------------------------------

#[test]
fn extract_option_box_string() {
    let src = b"nested";
    let node = leaf(0, 6);
    let result = Option::<Box<String>>::extract(Some(&node), src, 0, None);
    assert_eq!(result.as_deref(), Some(&"nested".to_string()));
}

#[test]
fn extract_box_option_string_none() {
    let result = Box::<Option<String>>::extract(None, b"", 0, None);
    assert_eq!(*result, None);
}
