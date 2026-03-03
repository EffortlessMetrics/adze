//! Tests for Tree and TreeCursor API.

use adze_runtime::tree::Tree;

#[test]
fn tree_new_stub() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // Stub tree should have a root node
    let debug = format!("{root:?}");
    assert!(!debug.is_empty());
}

#[test]
fn tree_root_kind() {
    let tree = Tree::new_stub();
    let _kind = tree.root_kind();
    // Just verifying it doesn't panic
}

#[test]
fn tree_language_stub() {
    let tree = Tree::new_stub();
    // Stub tree may or may not have a language
    let _lang = tree.language();
}

#[test]
fn tree_source_bytes_stub() {
    let tree = Tree::new_stub();
    let _src = tree.source_bytes();
}

#[test]
fn tree_cursor_creation() {
    let tree = Tree::new_stub();
    let _cursor = adze_runtime::tree::TreeCursor::new(&tree);
}

#[test]
fn tree_cursor_navigation() {
    let tree = Tree::new_stub();
    let mut cursor = adze_runtime::tree::TreeCursor::new(&tree);
    // These should not panic even on a stub tree
    let _has_child = cursor.goto_first_child();
    let _has_sibling = cursor.goto_next_sibling();
}

#[test]
fn tree_debug() {
    let tree = Tree::new_stub();
    let debug = format!("{tree:?}");
    assert!(!debug.is_empty());
}
