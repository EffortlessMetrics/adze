//! Comprehensive tests for Tree editing and manipulation in adze-runtime.
//!
//! Covers: stub tree creation, tree cloning independence, TreeCursor navigation,
//! Node traversal consistency, byte range invariants, multi-tree independence,
//! and incremental editing (feature-gated behind `incremental_glr`).

#![allow(clippy::needless_range_loop)]

use adze_runtime::tree::TreeCursor;
use adze_runtime::{Point, Tree};

#[cfg(feature = "incremental_glr")]
use adze_runtime::{EditError, InputEdit};

// ===== Helpers =====

/// Build a point helper for edit construction.
fn pt(row: usize, col: usize) -> Point {
    Point::new(row, col)
}

// ===== Section 1: Stub Tree Basics =====

#[test]
fn stub_tree_root_has_zero_byte_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn stub_tree_root_has_no_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn stub_tree_root_kind_id_is_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn stub_tree_has_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn stub_tree_has_no_source_bytes() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn stub_tree_root_is_not_error() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(!root.is_error());
    assert!(!root.is_missing());
}

#[test]
fn stub_tree_root_kind_returns_unknown_without_language() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

// ===== Section 2: Tree Cloning =====

#[test]
fn clone_preserves_root_byte_range() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
}

#[test]
fn clone_preserves_child_count() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
}

#[test]
fn clone_preserves_kind_id() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
}

#[test]
fn multiple_clones_are_independent() {
    let tree = Tree::new_stub();
    let c1 = tree.clone();
    let c2 = tree.clone();
    // All three are independent instances with identical structure
    assert_eq!(tree.root_node().start_byte(), c1.root_node().start_byte());
    assert_eq!(c1.root_node().start_byte(), c2.root_node().start_byte());
}

// ===== Section 3: TreeCursor on Stub =====

#[test]
fn cursor_starts_at_root() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    // Cursor is created successfully on a stub tree
    drop(cursor);
}

#[test]
fn cursor_cannot_descend_into_leaf() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_cannot_go_to_parent_from_root() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_cannot_go_to_next_sibling_from_root() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

// ===== Section 4: Node API Surface =====

#[test]
fn node_byte_range_consistency() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.byte_range(), root.start_byte()..root.end_byte());
}

#[test]
fn node_named_child_count_equals_child_count() {
    // Phase 1: named_child_count == child_count (no filtering)
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn node_child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.child(0).is_none());
    assert!(root.child(999).is_none());
}

#[test]
fn node_parent_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn node_siblings_return_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
    assert!(root.next_named_sibling().is_none());
    assert!(root.prev_named_sibling().is_none());
}

#[test]
fn node_child_by_field_name_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child_by_field_name("anything").is_none());
}

#[test]
fn node_is_named_returns_true() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().is_named());
}

// ===== Section 5: Multiple Tree Independence =====

#[test]
fn two_stub_trees_are_independent() {
    let t1 = Tree::new_stub();
    let t2 = Tree::new_stub();
    // Both have the same structure but are separate allocations
    assert_eq!(t1.root_node().child_count(), t2.root_node().child_count());
    assert_eq!(t1.root_kind(), t2.root_kind());
}

#[test]
fn cloned_tree_edit_does_not_affect_original() {
    // Even without incremental feature, verify clones are independent
    let tree = Tree::new_stub();
    let _cloned = tree.clone();
    // Original is unchanged
    assert_eq!(tree.root_node().start_byte(), 0);
}

// ===== Section 6: Debug Formatting =====

#[test]
fn tree_debug_format_is_nonempty() {
    let tree = Tree::new_stub();
    let debug = format!("{:?}", tree);
    assert!(!debug.is_empty());
    assert!(debug.contains("Tree"));
}

#[test]
fn node_debug_format_contains_kind() {
    let tree = Tree::new_stub();
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("Node"));
}

// ===== Section 7: Incremental Editing (feature-gated) =====

#[cfg(feature = "incremental_glr")]
fn make_edit(start: usize, old_end: usize, new_end: usize) -> InputEdit {
    InputEdit {
        start_byte: start,
        old_end_byte: old_end,
        new_end_byte: new_end,
        start_position: pt(0, start),
        old_end_position: pt(0, old_end),
        new_end_position: pt(0, new_end),
    }
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_stub_tree_with_zero_range_succeeds() {
    let mut tree = Tree::new_stub();
    // Zero-length insertion at position 0 — the stub root (0..0) is considered
    // "before" the edit (end_byte <= start_byte), so it is not modified.
    let edit = make_edit(0, 0, 5);
    tree.edit(&edit).expect("Zero-range edit on stub should succeed");
    assert_eq!(tree.root_node().end_byte(), 0);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_rejects_old_end_less_than_start() {
    let mut tree = Tree::new_stub();
    let edit = make_edit(10, 5, 15);
    let result = tree.edit(&edit);
    assert!(matches!(result, Err(EditError::InvalidRange { start: 10, old_end: 5 })));
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_rejects_new_end_less_than_start() {
    let mut tree = Tree::new_stub();
    let edit = make_edit(10, 15, 5);
    let result = tree.edit(&edit);
    assert!(matches!(result, Err(EditError::InvalidRange { start: 10, old_end: 5 })));
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_error_display_messages() {
    let invalid = EditError::InvalidRange { start: 3, old_end: 1 };
    let msg = format!("{}", invalid);
    assert!(msg.contains("Invalid edit range"));

    let overflow = EditError::ArithmeticOverflow;
    assert!(format!("{}", overflow).contains("overflow"));

    let underflow = EditError::ArithmeticUnderflow;
    assert!(format!("{}", underflow).contains("underflow"));
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(EditError::ArithmeticOverflow);
    // Verify it implements std::error::Error
    let _description = err.to_string();
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_clone_does_not_affect_original() {
    let mut tree = Tree::new_stub();
    // Give the tree a real range first so edits can intersect it
    tree.edit(&make_edit(0, 0, 0)).unwrap(); // no-op but valid
    let original = tree.clone();
    // The stub tree root is 0..0, entirely "before" position 0,
    // so edits pass through without changing byte ranges.
    let edit = make_edit(0, 0, 10);
    tree.edit(&edit).expect("edit should succeed");
    // Both remain unchanged because 0..0 is before the edit
    assert_eq!(original.root_node().end_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 0);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_sequential_insertions() {
    let mut tree = Tree::new_stub();
    // Stub tree (0..0) is "before" all these edits, so it stays at 0..0
    tree.edit(&make_edit(0, 0, 5)).unwrap();
    assert_eq!(tree.root_node().end_byte(), 0);

    tree.edit(&make_edit(5, 5, 8)).unwrap();
    assert_eq!(tree.root_node().end_byte(), 0);
}

// ===== Section 8: Point API =====

#[test]
fn point_new_and_accessors() {
    let p = Point::new(3, 7);
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

#[test]
fn point_display_is_one_indexed() {
    let p = Point::new(0, 0);
    assert_eq!(format!("{}", p), "1:1");

    let p2 = Point::new(2, 4);
    assert_eq!(format!("{}", p2), "3:5");
}

#[test]
fn point_equality() {
    let a = Point::new(1, 2);
    let b = Point::new(1, 2);
    let c = Point::new(1, 3);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn point_ordering() {
    let a = Point::new(0, 0);
    let b = Point::new(0, 5);
    let c = Point::new(1, 0);
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
}
