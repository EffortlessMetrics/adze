#![allow(clippy::needless_range_loop)]

//! Property-based tests for `Spanned`, `Point`, `ParsedNode`, and `ChildWalker`
//! from the adze runtime public API.

use adze::Spanned;
use adze::pure_parser::{ParsedNode, Point};
use std::mem::MaybeUninit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

fn spanned<T>(value: T, start: usize, end: usize) -> Spanned<T> {
    Spanned {
        value,
        span: (start, end),
    }
}

#[allow(clippy::too_many_arguments)]
fn make_node(
    symbol: u16,
    children: Vec<ParsedNode>,
    start: usize,
    end: usize,
    start_pt: Point,
    end_pt: Point,
    is_extra: bool,
    is_error: bool,
    is_missing: bool,
    is_named: bool,
    field_id: Option<u16>,
) -> ParsedNode {
    let mut uninit = MaybeUninit::<ParsedNode>::uninit();
    let ptr = uninit.as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 0, 1);
        std::ptr::addr_of_mut!((*ptr).symbol).write(symbol);
        std::ptr::addr_of_mut!((*ptr).children).write(children);
        std::ptr::addr_of_mut!((*ptr).start_byte).write(start);
        std::ptr::addr_of_mut!((*ptr).end_byte).write(end);
        std::ptr::addr_of_mut!((*ptr).start_point).write(start_pt);
        std::ptr::addr_of_mut!((*ptr).end_point).write(end_pt);
        std::ptr::addr_of_mut!((*ptr).is_extra).write(is_extra);
        std::ptr::addr_of_mut!((*ptr).is_error).write(is_error);
        std::ptr::addr_of_mut!((*ptr).is_missing).write(is_missing);
        std::ptr::addr_of_mut!((*ptr).is_named).write(is_named);
        std::ptr::addr_of_mut!((*ptr).field_id).write(field_id);
        uninit.assume_init()
    }
}

fn leaf(sym: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        sym,
        vec![],
        start,
        end,
        pt(0, start as u32),
        pt(0, end as u32),
        false,
        false,
        false,
        true,
        None,
    )
}

fn branch(sym: u16, start: usize, end: usize, children: Vec<ParsedNode>) -> ParsedNode {
    make_node(
        sym,
        children,
        start,
        end,
        pt(0, start as u32),
        pt(0, end as u32),
        false,
        false,
        false,
        true,
        None,
    )
}

// ===================================================================
// 1. Point creation preserves row and column
// ===================================================================
#[test]
fn point_creation_preserves_fields() {
    let p = pt(42, 99);
    assert_eq!(p.row, 42);
    assert_eq!(p.column, 99);
}

// ===================================================================
// 2. Point default is (0, 0)
// ===================================================================
#[test]
fn point_default_is_origin() {
    let p = Point::default();
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

// ===================================================================
// 3. Point equality – reflexive
// ===================================================================
#[test]
fn point_eq_reflexive() {
    let p = pt(10, 20);
    assert_eq!(p, p);
}

// ===================================================================
// 4. Point equality – symmetric
// ===================================================================
#[test]
fn point_eq_symmetric() {
    let a = pt(5, 7);
    let b = pt(5, 7);
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ===================================================================
// 5. Point inequality when row differs
// ===================================================================
#[test]
fn point_ne_different_row() {
    assert_ne!(pt(1, 0), pt(2, 0));
}

// ===================================================================
// 6. Point inequality when column differs
// ===================================================================
#[test]
fn point_ne_different_column() {
    assert_ne!(pt(0, 1), pt(0, 2));
}

// ===================================================================
// 7. Point copy semantics
// ===================================================================
#[test]
fn point_is_copy() {
    let a = pt(3, 4);
    let b = a; // copy
    assert_eq!(a, b);
    assert_eq!(a.row, 3);
}

// ===================================================================
// 8. Point clone matches original
// ===================================================================
#[test]
fn point_clone_matches() {
    let a = pt(100, 200);
    let b = a;
    assert_eq!(a, b);
}

// ===================================================================
// 9. Point debug contains row and column
// ===================================================================
#[test]
fn point_debug_contains_fields() {
    let p = pt(7, 13);
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("7"), "debug should contain row");
    assert!(dbg.contains("13"), "debug should contain column");
}

// ===================================================================
// 10. Point large values
// ===================================================================
#[test]
fn point_large_values() {
    let p = pt(u32::MAX, u32::MAX);
    assert_eq!(p.row, u32::MAX);
    assert_eq!(p.column, u32::MAX);
}

// ===================================================================
// 11. Spanned wraps value and provides deref
// ===================================================================
#[test]
fn spanned_deref_returns_value() {
    let s = spanned(42i32, 0, 10);
    assert_eq!(*s, 42);
}

// ===================================================================
// 12. Spanned span field stores start and end
// ===================================================================
#[test]
fn spanned_span_stores_range() {
    let s = spanned("hello", 5, 10);
    assert_eq!(s.span.0, 5);
    assert_eq!(s.span.1, 10);
}

// ===================================================================
// 13. Spanned clone preserves both value and span
// ===================================================================
#[test]
fn spanned_clone_preserves_all() {
    let s = spanned(String::from("abc"), 1, 4);
    let c = s.clone();
    assert_eq!(*c, "abc");
    assert_eq!(c.span, (1, 4));
}

// ===================================================================
// 14. Spanned with zero-length span
// ===================================================================
#[test]
fn spanned_zero_length_span() {
    let s = spanned(0u8, 7, 7);
    assert_eq!(s.span.0, s.span.1);
}

// ===================================================================
// 15. Spanned value field direct access
// ===================================================================
#[test]
fn spanned_value_field_access() {
    let s = spanned(vec![1, 2, 3], 0, 3);
    assert_eq!(s.value, vec![1, 2, 3]);
}

// ===================================================================
// 16. Spanned debug contains value and span info
// ===================================================================
#[test]
fn spanned_debug_format() {
    let s = spanned(99i32, 10, 20);
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("99"));
    assert!(dbg.contains("10"));
    assert!(dbg.contains("20"));
}

// ===================================================================
// 17. Spanned start < end convention
// ===================================================================
#[test]
fn spanned_start_le_end() {
    let s = spanned((), 3, 8);
    assert!(s.span.0 <= s.span.1);
}

// ===================================================================
// 18. ParsedNode byte range: start_byte <= end_byte
// ===================================================================
#[test]
fn parsed_node_byte_range_consistent() {
    let n = leaf(1, 5, 15);
    assert!(n.start_byte() <= n.end_byte());
}

// ===================================================================
// 19. ParsedNode accessors match construction
// ===================================================================
#[test]
fn parsed_node_accessor_roundtrip() {
    let n = make_node(
        42,
        vec![],
        10,
        20,
        pt(1, 0),
        pt(1, 10),
        false,
        false,
        false,
        true,
        Some(5),
    );
    assert_eq!(n.symbol(), 42);
    assert_eq!(n.start_byte(), 10);
    assert_eq!(n.end_byte(), 20);
    assert_eq!(n.start_point(), pt(1, 0));
    assert_eq!(n.end_point(), pt(1, 10));
    assert!(n.is_named());
    assert!(!n.is_error());
    assert!(!n.is_missing());
    assert!(!n.is_extra());
}

// ===================================================================
// 20. ParsedNode child_count on leaf is zero
// ===================================================================
#[test]
fn parsed_node_leaf_has_no_children() {
    let n = leaf(1, 0, 5);
    assert_eq!(n.child_count(), 0);
    assert!(n.child(0).is_none());
}

// ===================================================================
// 21. ParsedNode children slice length matches child_count
// ===================================================================
#[test]
fn parsed_node_children_slice_len() {
    let kids = vec![leaf(1, 0, 1), leaf(2, 1, 2), leaf(3, 2, 3)];
    let parent = branch(99, 0, 3, kids);
    assert_eq!(parent.child_count(), 3);
    assert_eq!(parent.children().len(), 3);
}

// ===================================================================
// 22. ParsedNode child(i) returns correct node
// ===================================================================
#[test]
fn parsed_node_child_by_index() {
    let kids = vec![leaf(10, 0, 1), leaf(20, 1, 2)];
    let parent = branch(99, 0, 2, kids);
    assert_eq!(parent.child(0).unwrap().symbol(), 10);
    assert_eq!(parent.child(1).unwrap().symbol(), 20);
    assert!(parent.child(2).is_none());
}

// ===================================================================
// 23. ParsedNode has_error propagates
// ===================================================================
#[test]
fn parsed_node_has_error_propagates() {
    let err_child = make_node(
        1,
        vec![],
        0,
        1,
        pt(0, 0),
        pt(0, 1),
        false,
        true,
        false,
        true,
        None,
    );
    let parent = branch(99, 0, 1, vec![err_child]);
    assert!(!parent.is_error());
    assert!(parent.has_error());
}

// ===================================================================
// 24. ParsedNode utf8_text on valid source
// ===================================================================
#[test]
fn parsed_node_utf8_text_valid() {
    let n = leaf(1, 2, 5);
    let source = b"hello world";
    assert_eq!(n.utf8_text(source).unwrap(), "llo");
}

// ===================================================================
// 25. ParsedNode utf8_text on out-of-bounds returns error
// ===================================================================
#[test]
fn parsed_node_utf8_text_out_of_bounds() {
    let n = leaf(1, 0, 100);
    let source = b"short";
    assert!(n.utf8_text(source).is_err());
}

// ===================================================================
// 26. ChildWalker on empty node returns false
// ===================================================================
#[test]
fn child_walker_empty_returns_false() {
    let n = leaf(1, 0, 1);
    let mut w = n.walk();
    assert!(!w.goto_first_child());
}

// ===================================================================
// 27. ChildWalker iterates all children in order
// ===================================================================
#[test]
fn child_walker_iterates_all_in_order() {
    let kids = vec![leaf(10, 0, 1), leaf(20, 1, 2), leaf(30, 2, 3)];
    let parent = branch(99, 0, 3, kids);
    let mut w = parent.walk();
    assert!(w.goto_first_child());
    assert_eq!(w.node().symbol(), 10);
    assert!(w.goto_next_sibling());
    assert_eq!(w.node().symbol(), 20);
    assert!(w.goto_next_sibling());
    assert_eq!(w.node().symbol(), 30);
    assert!(!w.goto_next_sibling());
}

// ===================================================================
// 28. ChildWalker goto_first_child resets position
// ===================================================================
#[test]
fn child_walker_goto_first_resets() {
    let kids = vec![leaf(10, 0, 1), leaf(20, 1, 2)];
    let parent = branch(99, 0, 2, kids);
    let mut w = parent.walk();
    assert!(w.goto_first_child());
    assert!(w.goto_next_sibling());
    assert_eq!(w.node().symbol(), 20);
    // Reset
    assert!(w.goto_first_child());
    assert_eq!(w.node().symbol(), 10);
}

// ===================================================================
// 29. ChildWalker single-child node
// ===================================================================
#[test]
fn child_walker_single_child() {
    let parent = branch(99, 0, 1, vec![leaf(7, 0, 1)]);
    let mut w = parent.walk();
    assert!(w.goto_first_child());
    assert_eq!(w.node().symbol(), 7);
    assert!(!w.goto_next_sibling());
}

// ===================================================================
// 30. ChildWalker node byte ranges match children
// ===================================================================
#[test]
fn child_walker_node_byte_ranges() {
    let kids = vec![leaf(1, 0, 3), leaf(2, 3, 7), leaf(3, 7, 10)];
    let parent = branch(99, 0, 10, kids);
    let mut w = parent.walk();
    assert!(w.goto_first_child());
    assert_eq!(w.node().start_byte(), 0);
    assert_eq!(w.node().end_byte(), 3);
    assert!(w.goto_next_sibling());
    assert_eq!(w.node().start_byte(), 3);
    assert_eq!(w.node().end_byte(), 7);
    assert!(w.goto_next_sibling());
    assert_eq!(w.node().start_byte(), 7);
    assert_eq!(w.node().end_byte(), 10);
}

// ===================================================================
// 31. ParsedNode points match construction
// ===================================================================
#[test]
fn parsed_node_points_match_construction() {
    let n = make_node(
        1,
        vec![],
        0,
        10,
        pt(2, 5),
        pt(3, 8),
        false,
        false,
        false,
        true,
        None,
    );
    assert_eq!(n.start_point().row, 2);
    assert_eq!(n.start_point().column, 5);
    assert_eq!(n.end_point().row, 3);
    assert_eq!(n.end_point().column, 8);
}

// ===================================================================
// 32. Spanned with nested Spanned
// ===================================================================
#[test]
fn spanned_nested() {
    let inner = spanned(10u32, 2, 5);
    let outer = spanned(inner, 0, 10);
    assert_eq!(*outer.value, 10);
    assert_eq!(outer.value.span, (2, 5));
    assert_eq!(outer.span, (0, 10));
}

// ===================================================================
// 33. ParsedNode is_missing flag
// ===================================================================
#[test]
fn parsed_node_is_missing_flag() {
    let n = make_node(
        1,
        vec![],
        0,
        0,
        pt(0, 0),
        pt(0, 0),
        false,
        false,
        true,
        true,
        None,
    );
    assert!(n.is_missing());
    assert!(!n.is_error());
}

// ===================================================================
// 34. ParsedNode is_extra flag
// ===================================================================
#[test]
fn parsed_node_is_extra_flag() {
    let n = make_node(
        1,
        vec![],
        0,
        1,
        pt(0, 0),
        pt(0, 1),
        true,
        false,
        false,
        false,
        None,
    );
    assert!(n.is_extra());
    assert!(!n.is_named());
}

// ===================================================================
// 35. Point struct size is 8 bytes (two u32s)
// ===================================================================
#[test]
fn point_struct_layout() {
    assert_eq!(std::mem::size_of::<Point>(), 8);
}
