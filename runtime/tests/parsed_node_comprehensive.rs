#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the `ParsedNode` API and related types
//! (`ChildWalker`, `Point`, `ParseResult`).

use adze::pure_parser::{ParsedNode, Point};
use std::mem::MaybeUninit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Create a `ParsedNode` without naming the `pub(crate)` `language` field.
/// Zero-init sets `language: Option<*const TSLanguage>` to `None`, then we
/// overwrite every public field.
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
    // SAFETY: zeroed memory gives `language = None`. Every public field is
    // then overwritten. A zeroed Vec is immediately replaced, so no
    // double-free.
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

/// Build a leaf `ParsedNode` spanning `[start, end)` on row 0.
fn leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        symbol,
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

/// Build a branch node whose range is `[start, end)` with the given children.
fn branch(symbol: u16, start: usize, end: usize, children: Vec<ParsedNode>) -> ParsedNode {
    make_node(
        symbol,
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

/// Build a leaf with custom flags.
fn leaf_with_flags(
    symbol: u16,
    start: usize,
    end: usize,
    is_extra: bool,
    is_error: bool,
    is_missing: bool,
    is_named: bool,
) -> ParsedNode {
    make_node(
        symbol,
        vec![],
        start,
        end,
        pt(0, start as u32),
        pt(0, end as u32),
        is_extra,
        is_error,
        is_missing,
        is_named,
        None,
    )
}

/// Build a simple expression tree: `expr(num, op, num)` over source "3+5".
fn sample_expr_tree() -> ParsedNode {
    let num1 = leaf(1, 0, 1); // "3"
    let op = leaf(2, 1, 2); // "+"
    let num2 = leaf(1, 2, 3); // "5"
    branch(10, 0, 3, vec![num1, op, num2])
}

/// Build a multiline node spanning rows 0‒2.
fn multiline_node() -> ParsedNode {
    make_node(
        42,
        vec![],
        0,
        20,
        pt(0, 0),
        pt(2, 5),
        false,
        false,
        false,
        true,
        None,
    )
}

// ===================================================================
// Tests — basic accessors
// ===================================================================

#[test]
fn test_symbol_accessor() {
    let node = leaf(7, 0, 3);
    assert_eq!(node.symbol(), 7);
}

#[test]
fn test_start_byte_accessor() {
    let node = leaf(1, 5, 10);
    assert_eq!(node.start_byte(), 5);
}

#[test]
fn test_end_byte_accessor() {
    let node = leaf(1, 5, 10);
    assert_eq!(node.end_byte(), 10);
}

#[test]
fn test_start_point_accessor() {
    let node = leaf(1, 3, 6);
    assert_eq!(node.start_point(), Point { row: 0, column: 3 });
}

#[test]
fn test_end_point_accessor() {
    let node = multiline_node();
    assert_eq!(node.end_point(), Point { row: 2, column: 5 });
}

// ===================================================================
// Tests — flag queries
// ===================================================================

#[test]
fn test_is_named_true() {
    let node = leaf(1, 0, 1);
    assert!(node.is_named());
}

#[test]
fn test_is_named_false() {
    let node = leaf_with_flags(1, 0, 1, false, false, false, false);
    assert!(!node.is_named());
}

#[test]
fn test_is_extra_flag() {
    let node = leaf_with_flags(1, 0, 1, true, false, false, true);
    assert!(node.is_extra());
}

#[test]
fn test_is_error_flag() {
    let node = leaf_with_flags(1, 0, 1, false, true, false, true);
    assert!(node.is_error());
}

#[test]
fn test_is_missing_flag() {
    let node = leaf_with_flags(1, 0, 1, false, false, true, true);
    assert!(node.is_missing());
}

// ===================================================================
// Tests — has_error (recursive)
// ===================================================================

#[test]
fn test_has_error_on_clean_tree() {
    let tree = sample_expr_tree();
    assert!(!tree.has_error());
}

#[test]
fn test_has_error_on_error_leaf() {
    let error = leaf_with_flags(99, 0, 1, false, true, false, true);
    assert!(error.has_error());
}

#[test]
fn test_has_error_propagates_through_children() {
    let error_leaf = leaf_with_flags(99, 0, 1, false, true, false, true);
    let parent = branch(10, 0, 5, vec![leaf(1, 0, 1), error_leaf]);
    assert!(parent.has_error());
}

#[test]
fn test_has_error_propagates_deeply() {
    let error_leaf = leaf_with_flags(99, 2, 3, false, true, false, true);
    let inner = branch(5, 0, 5, vec![leaf(1, 0, 2), error_leaf]);
    let outer = branch(10, 0, 10, vec![inner]);
    assert!(outer.has_error());
}

// ===================================================================
// Tests — children accessors
// ===================================================================

#[test]
fn test_child_count_leaf() {
    let node = leaf(1, 0, 1);
    assert_eq!(node.child_count(), 0);
}

#[test]
fn test_child_count_branch() {
    let tree = sample_expr_tree();
    assert_eq!(tree.child_count(), 3);
}

#[test]
fn test_child_valid_index() {
    let tree = sample_expr_tree();
    let first = tree.child(0).unwrap();
    assert_eq!(first.symbol(), 1);
    assert_eq!(first.start_byte(), 0);
    assert_eq!(first.end_byte(), 1);
}

#[test]
fn test_child_out_of_bounds() {
    let tree = sample_expr_tree();
    assert!(tree.child(100).is_none());
}

#[test]
fn test_children_slice() {
    let tree = sample_expr_tree();
    let kids = tree.children();
    assert_eq!(kids.len(), 3);
    assert_eq!(kids[1].symbol(), 2); // operator "+"
}

// ===================================================================
// Tests — utf8_text
// ===================================================================

#[test]
fn test_utf8_text_simple() {
    let source = b"3+5";
    let tree = sample_expr_tree();
    assert_eq!(tree.utf8_text(source).unwrap(), "3+5");
}

#[test]
fn test_utf8_text_child() {
    let source = b"3+5";
    let tree = sample_expr_tree();
    let op = tree.child(1).unwrap();
    assert_eq!(op.utf8_text(source).unwrap(), "+");
}

#[test]
fn test_utf8_text_empty_range() {
    let node = leaf(1, 3, 3);
    let source = b"hello";
    assert_eq!(node.utf8_text(source).unwrap(), "");
}

#[test]
fn test_utf8_text_out_of_bounds() {
    let node = leaf(1, 0, 100);
    let source = b"short";
    assert!(node.utf8_text(source).is_err());
}

#[test]
fn test_utf8_text_unicode() {
    let source = "héllo".as_bytes();
    let node = leaf(1, 0, source.len());
    assert_eq!(node.utf8_text(source).unwrap(), "héllo");
}

#[test]
fn test_utf8_text_multibyte_substring() {
    // "café" has 'é' as 2 bytes at positions 3-4
    let source = "café".as_bytes();
    // Extract just "caf"
    let node = leaf(1, 0, 3);
    assert_eq!(node.utf8_text(source).unwrap(), "caf");
}

// ===================================================================
// Tests — ChildWalker
// ===================================================================

#[test]
fn test_walker_goto_first_child_on_branch() {
    let tree = sample_expr_tree();
    let mut walker = tree.walk();
    assert!(walker.goto_first_child());
    assert_eq!(walker.node().symbol(), 1);
}

#[test]
fn test_walker_goto_first_child_on_leaf() {
    let node = leaf(1, 0, 1);
    let mut walker = node.walk();
    assert!(!walker.goto_first_child());
}

#[test]
fn test_walker_goto_next_sibling() {
    let tree = sample_expr_tree();
    let mut walker = tree.walk();
    assert!(walker.goto_first_child());
    assert!(walker.goto_next_sibling());
    assert_eq!(walker.node().symbol(), 2); // operator "+"
}

#[test]
fn test_walker_goto_next_sibling_at_end() {
    let tree = sample_expr_tree();
    let mut walker = tree.walk();
    assert!(walker.goto_first_child());
    assert!(walker.goto_next_sibling()); // -> op
    assert!(walker.goto_next_sibling()); // -> num2
    assert!(!walker.goto_next_sibling()); // no more
}

#[test]
fn test_walker_full_traversal_collects_all_children() {
    let tree = sample_expr_tree();
    let mut walker = tree.walk();
    let mut symbols = Vec::new();
    if walker.goto_first_child() {
        symbols.push(walker.node().symbol());
        while walker.goto_next_sibling() {
            symbols.push(walker.node().symbol());
        }
    }
    assert_eq!(symbols, vec![1, 2, 1]);
}

#[test]
fn test_walker_reset_via_goto_first_child() {
    let tree = sample_expr_tree();
    let mut walker = tree.walk();
    walker.goto_first_child();
    walker.goto_next_sibling(); // now at child 1
    // Reset back to first child
    walker.goto_first_child();
    assert_eq!(walker.node().symbol(), 1);
    assert_eq!(walker.node().start_byte(), 0);
}

// ===================================================================
// Tests — clone / Debug
// ===================================================================

#[test]
fn test_parsed_node_clone() {
    let tree = sample_expr_tree();
    let cloned = tree.clone();
    assert_eq!(cloned.symbol(), tree.symbol());
    assert_eq!(cloned.child_count(), tree.child_count());
    assert_eq!(cloned.start_byte(), tree.start_byte());
    assert_eq!(cloned.end_byte(), tree.end_byte());
}

#[test]
fn test_parsed_node_debug() {
    let node = leaf(5, 0, 3);
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("ParsedNode"));
    assert!(dbg.contains("symbol: 5"));
}

// ===================================================================
// Tests — kind fallback (no language pointer)
// ===================================================================

#[test]
fn test_kind_fallback_known_symbol() {
    // symbol 0 → "end" in the hard-coded fallback table
    let node = leaf(0, 0, 1);
    assert_eq!(node.kind(), "end");
}

#[test]
fn test_kind_fallback_unknown_symbol() {
    let node = leaf(200, 0, 1);
    assert_eq!(node.kind(), "unknown");
}

// ===================================================================
// Tests — field_id
// ===================================================================

#[test]
fn test_field_id_none_by_default() {
    let node = leaf(1, 0, 1);
    assert_eq!(node.field_id, None);
}

#[test]
fn test_field_id_set() {
    let mut node = leaf(1, 0, 1);
    node.field_id = Some(42);
    assert_eq!(node.field_id, Some(42));
}

// ===================================================================
// Tests — Point
// ===================================================================

#[test]
fn test_point_default() {
    let p = Point::default();
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn test_point_equality() {
    let a = Point { row: 1, column: 5 };
    let b = Point { row: 1, column: 5 };
    let c = Point { row: 2, column: 0 };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ===================================================================
// Tests — deep tree
// ===================================================================

#[test]
fn test_deeply_nested_tree() {
    // Build a chain: root -> a -> b -> c (leaf)
    let c = leaf(3, 2, 3);
    let b = branch(2, 1, 3, vec![c]);
    let a = branch(1, 0, 3, vec![b]);
    let root = branch(0, 0, 3, vec![a]);

    assert_eq!(root.child_count(), 1);
    let level1 = root.child(0).unwrap();
    assert_eq!(level1.symbol(), 1);
    let level2 = level1.child(0).unwrap();
    assert_eq!(level2.symbol(), 2);
    let level3 = level2.child(0).unwrap();
    assert_eq!(level3.symbol(), 3);
    assert_eq!(level3.child_count(), 0);
}
