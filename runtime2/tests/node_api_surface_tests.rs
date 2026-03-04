//! Tests for the Node API surface.

use adze_runtime::tree::Tree;

#[test]
fn node_kind_id() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let _id = root.kind_id();
}

#[test]
fn node_byte_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let range = root.byte_range();
    assert!(range.start <= range.end);
}

#[test]
fn node_start_end() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.start_byte() <= root.end_byte());
}

#[test]
fn node_positions() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let start = root.start_position();
    let end = root.end_position();
    // Start position row should be <= end position row
    assert!(start.row <= end.row || start.column <= end.column);
}

#[test]
fn node_is_named() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // Just verify it doesn't panic
    let _named = root.is_named();
}

#[test]
fn node_is_missing() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let _missing = root.is_missing();
}

#[test]
fn node_is_error() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let _error = root.is_error();
}

#[test]
fn node_child_count() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let _count = root.child_count();
    let _named_count = root.named_child_count();
}

#[test]
fn node_child_by_index() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // May return None for stub tree
    let _child = root.child(0);
}

#[test]
fn node_child_by_field_name() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let _child = root.child_by_field_name("nonexistent");
}

#[test]
fn node_siblings() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // Root has no siblings
    let _next = root.next_sibling();
    let _prev = root.prev_sibling();
}

#[test]
fn node_debug() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let debug = format!("{root:?}");
    assert!(!debug.is_empty());
}
