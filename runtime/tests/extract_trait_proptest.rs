#![allow(clippy::needless_range_loop)]

//! Property-based tests for the `Extract` trait in the adze runtime.
//!
//! Uses proptest to verify extraction invariants over randomly generated
//! `ParsedNode` instances and source byte slices.

use adze::pure_parser::{ParsedNode, Point};
use adze::{Extract, Spanned};
use proptest::prelude::*;
use std::mem::MaybeUninit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Build a `ParsedNode` via field-level pointer writes to avoid depending on
/// private fields that may not have a public constructor.
#[allow(clippy::too_many_arguments)]
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

/// Leaf node spanning `source[start..end]`.
fn leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(symbol, vec![], start, end, true)
}

/// Branch node with children.
fn branch(symbol: u16, start: usize, end: usize, children: Vec<ParsedNode>) -> ParsedNode {
    make_node(symbol, children, start, end, true)
}

// =========================================================================
// 1. Extract for basic types — String
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_string_from_valid_node(s in "[a-zA-Z0-9_ ]{1,64}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, s);
    }

    #[test]
    fn extract_string_none_yields_empty(_dummy in 0u8..1) {
        let result: String = String::extract(None, b"anything", 0, None);
        prop_assert_eq!(result, String::new());
    }

    #[test]
    fn extract_string_empty_span(_dummy in 0u8..1) {
        let source = b"hello";
        let node = leaf(1, 0, 0);
        let result: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, "");
    }
}

// =========================================================================
// 2. Extract for numeric primitives
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_i32_roundtrip(val in -10000i32..10000) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: i32 = i32::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, val);
    }

    #[test]
    fn extract_u64_roundtrip(val in 0u64..100_000) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: u64 = u64::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, val);
    }

    #[test]
    fn extract_f64_roundtrip(val in -1000.0f64..1000.0) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: f64 = f64::extract(Some(&node), source, 0, None);
        // Compare with epsilon for floating point
        prop_assert!((result - val).abs() < 1e-10,
            "expected {} but got {}", val, result);
    }

    #[test]
    fn extract_bool_values(_dummy in 0u8..1) {
        let node_t = leaf(1, 0, 4);
        prop_assert!(bool::extract(Some(&node_t), b"true", 0, None));
        let node_f = leaf(1, 0, 5);
        prop_assert!(!bool::extract(Some(&node_f), b"false", 0, None));
    }

    #[test]
    fn extract_i8_roundtrip(val in -128i8..=127) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: i8 = i8::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, val);
    }
}

// =========================================================================
// 3. Extract for Option types
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_option_string(s in "[a-z]{1,32}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let some: Option<String> =
            <Option<String> as Extract<Option<String>>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(some, Some(s));

        let none: Option<String> =
            <Option<String> as Extract<Option<String>>>::extract(None, b"", 0, None);
        prop_assert_eq!(none, None);
    }

    #[test]
    fn extract_option_i32(val in -1000i32..1000) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let some: Option<i32> =
            <Option<i32> as Extract<Option<i32>>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(some, Some(val));

        let none: Option<i32> =
            <Option<i32> as Extract<Option<i32>>>::extract(None, b"", 0, None);
        prop_assert_eq!(none, None);
    }
}

// =========================================================================
// 4. Extract for Vec types
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn extract_vec_none_yields_empty(_dummy in 0u8..1) {
        let result: Vec<String> =
            <Vec<String> as Extract<Vec<String>>>::extract(None, b"", 0, None);
        prop_assert!(result.is_empty());
    }

    #[test]
    fn extract_vec_single_child(s in "[a-z]{1,16}") {
        let source = s.as_bytes();
        let child = leaf(2, 0, source.len());
        let parent = branch(1, 0, source.len(), vec![child]);
        let result: Vec<String> =
            <Vec<String> as Extract<Vec<String>>>::extract(Some(&parent), source, 0, None);
        prop_assert_eq!(result.len(), 1);
        prop_assert_eq!(&result[0], &s);
    }

    #[test]
    fn extract_vec_multiple_children(
        a in "[a-z]{1,8}",
        b in "[a-z]{1,8}",
    ) {
        // Build source "a b"
        let combined = format!("{} {}", a, b);
        let source = combined.as_bytes();
        let a_end = a.len();
        let b_start = a.len() + 1;
        let b_end = combined.len();

        let child_a = leaf(2, 0, a_end);
        let child_b = leaf(2, b_start, b_end);
        let parent = branch(1, 0, b_end, vec![child_a, child_b]);

        let result: Vec<String> =
            <Vec<String> as Extract<Vec<String>>>::extract(Some(&parent), source, 0, None);
        prop_assert_eq!(result.len(), 2);
        prop_assert_eq!(&result[0], &a);
        prop_assert_eq!(&result[1], &b);
    }
}

// =========================================================================
// 5. Extract error handling — unparseable text
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_i32_from_non_numeric_defaults(s in "[a-zA-Z]{1,16}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: i32 = i32::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, 0i32);
    }

    #[test]
    fn extract_f64_from_non_numeric_defaults(s in "[a-zA-Z]{1,16}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: f64 = f64::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, 0.0f64);
    }

    #[test]
    fn extract_bool_from_garbage_defaults(s in "[0-9]{1,8}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: bool = bool::extract(Some(&node), source, 0, None);
        prop_assert!(!result);
    }
}

// =========================================================================
// 6. Extract with missing nodes (None)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn extract_numeric_none_defaults(_dummy in 0u8..1) {
        prop_assert_eq!(i32::extract(None, b"", 0, None), 0i32);
        prop_assert_eq!(u64::extract(None, b"", 0, None), 0u64);
        prop_assert_eq!(f32::extract(None, b"", 0, None), 0.0f32);
    }

    #[test]
    fn extract_string_none_defaults(_dummy in 0u8..1) {
        let result: String = String::extract(None, b"", 0, None);
        prop_assert_eq!(result, "");
    }

    #[test]
    fn extract_bool_none_defaults(_dummy in 0u8..1) {
        let result: bool = bool::extract(None, b"", 0, None);
        prop_assert!(!result);
    }
}

// =========================================================================
// 7. Extract idempotency
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_string_idempotent(s in "[a-zA-Z0-9]{1,32}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let r1: String = String::extract(Some(&node), source, 0, None);
        let r2: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn extract_i32_idempotent(val in -5000i32..5000) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let r1: i32 = i32::extract(Some(&node), source, 0, None);
        let r2: i32 = i32::extract(Some(&node), source, 0, None);
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn extract_option_idempotent(val in 0u32..1000) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let r1: Option<u32> =
            <Option<u32> as Extract<Option<u32>>>::extract(Some(&node), source, 0, None);
        let r2: Option<u32> =
            <Option<u32> as Extract<Option<u32>>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(r1, r2);
    }
}

// =========================================================================
// 8. Extract for Box and unit types
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn extract_box_string(s in "[a-z]{1,16}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: Box<String> =
            <Box<String> as Extract<Box<String>>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(*result, s);
    }

    #[test]
    fn extract_box_i32(val in -1000i32..1000) {
        let text = val.to_string();
        let source = text.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: Box<i32> =
            <Box<i32> as Extract<Box<i32>>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(*result, val);
    }

    #[test]
    fn extract_unit_with_and_without_node(_dummy in 0u8..1) {
        let node = leaf(1, 0, 5);
        <() as Extract<()>>::extract(Some(&node), b"hello", 0, None);
        <() as Extract<()>>::extract(None, b"", 0, None);
    }
}

// =========================================================================
// 9. Spanned extraction
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_spanned_string_has_correct_span(s in "[a-z]{1,32}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let result: Spanned<String> =
            <Spanned<String> as Extract<Spanned<String>>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result.span, (0, source.len()));
        prop_assert_eq!(result.value, s);
    }

    #[test]
    fn extract_spanned_none_uses_last_idx(idx in 0usize..100) {
        let result: Spanned<String> =
            <Spanned<String> as Extract<Spanned<String>>>::extract(None, b"", idx, None);
        prop_assert_eq!(result.span, (idx, idx));
        prop_assert_eq!(result.value, "");
    }
}

// =========================================================================
// 10. ParsedNode creation and access
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parsed_node_byte_range(start in 0usize..100, len in 1usize..100) {
        let end = start + len;
        let node = leaf(1, start, end);
        prop_assert_eq!(node.start_byte(), start);
        prop_assert_eq!(node.end_byte(), end);
    }

    #[test]
    fn parsed_node_child_count(n in 0usize..8) {
        let source_len = n.max(1);
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(2, i, i + 1)).collect();
        let parent = branch(1, 0, source_len, children);
        prop_assert_eq!(parent.child_count(), n);
    }

    #[test]
    fn parsed_node_child_access(n in 1usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(2, i, i + 1)).collect();
        let parent = branch(1, 0, n, children);
        for i in 0..n {
            let child = parent.child(i);
            prop_assert!(child.is_some());
            prop_assert_eq!(child.unwrap().start_byte(), i);
        }
        prop_assert!(parent.child(n).is_none());
    }

    #[test]
    fn parsed_node_is_named_flag(is_named in proptest::bool::ANY) {
        let node = make_node(1, vec![], 0, 1, is_named);
        prop_assert_eq!(node.is_named(), is_named);
    }

    #[test]
    fn parsed_node_children_slice_matches_count(n in 0usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(2, i, i + 1)).collect();
        let parent = branch(1, 0, n.max(1), children);
        prop_assert_eq!(parent.children().len(), parent.child_count());
    }

    #[test]
    fn parsed_node_utf8_text(s in "[a-zA-Z0-9]{1,32}") {
        let source = s.as_bytes();
        let node = leaf(1, 0, source.len());
        let text = node.utf8_text(source);
        prop_assert!(text.is_ok());
        prop_assert_eq!(text.unwrap(), s.as_str());
    }
}

// =========================================================================
// 11. Extract with partial/offset spans
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_string_from_middle(
        prefix in "[a-z]{1,8}",
        middle in "[A-Z]{1,8}",
        suffix in "[0-9]{1,8}",
    ) {
        let combined = format!("{}{}{}", prefix, middle, suffix);
        let source = combined.as_bytes();
        let start = prefix.len();
        let end = start + middle.len();
        let node = leaf(1, start, end);
        let result: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, middle);
    }

    #[test]
    fn extract_i32_from_offset(val in 0i32..999) {
        let text = format!("   {}", val);
        let source = text.as_bytes();
        let start = 3; // skip spaces
        let end = source.len();
        let node = leaf(1, start, end);
        let result: i32 = i32::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, val);
    }
}
