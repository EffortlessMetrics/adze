//! Tests for Tree::new_stub() and Node/TreeCursor public API.

use adze_runtime::tree::{Tree, TreeCursor};

#[test]
fn stub_tree_root_node_exists() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // Stub tree should have a root node
    let _ = root.kind();
}

#[test]
fn stub_tree_root_kind() {
    let tree = Tree::new_stub();
    let kind = tree.root_kind();
    // Should return some kind value
    let _ = kind;
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
fn stub_tree_root_node_kind_id() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let _ = root.kind_id();
}

#[test]
fn stub_tree_root_byte_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let start = root.start_byte();
    let end = root.end_byte();
    assert!(start <= end);
}

#[test]
fn stub_tree_root_is_named() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let _ = root.is_named();
}

#[test]
fn stub_tree_root_child_count() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let count = root.child_count();
    let _ = count;
}

#[test]
fn stub_tree_cursor_creation() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    let _ = cursor;
}

#[test]
fn stub_tree_debug_display() {
    let tree = Tree::new_stub();
    let debug = format!("{:?}", tree);
    assert!(!debug.is_empty());
}

#[test]
fn stub_tree_root_node_debug() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let debug = format!("{:?}", root);
    assert!(!debug.is_empty());
}

#[test]
fn stub_tree_root_positions() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let start = root.start_position();
    let end = root.end_position();
    let _ = (start, end);
}

#[test]
fn stub_tree_clone() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
}
