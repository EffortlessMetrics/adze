//! Tree and Node API tests.
//!
//! Covers: node metadata, kind resolution with language, byte ranges,
//! child access, text extraction, tree cloning, source bytes, walk,
//! cursor edge cases, and InputEdit/Point types.

use adze_runtime::{Point, Tree, tree::TreeCursor};

// ---------------------------------------------------------------------------
// Helper: build a tree with known structure and a language.
// set_language/set_source are pub(crate), so we use Parser::parse to get
// a tree with language set when needed (glr-core only).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Node basic metadata
// ---------------------------------------------------------------------------

#[test]
fn stub_root_node_kind_without_language() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind(), "unknown");
}

// kind resolution with language is tested in builder_tests.rs via Parser::parse

#[test]
fn node_kind_id() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
}

#[test]
fn node_byte_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 0..0);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn node_positions_are_dummy() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_position(), Point::new(0, 0));
    assert_eq!(root.end_position(), Point::new(0, 0));
}

#[test]
fn node_is_named_always_true() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().is_named());
}

#[test]
fn node_is_missing_always_false() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_missing());
}

#[test]
fn node_is_error_always_false() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_error());
}

// ---------------------------------------------------------------------------
// Node child access
// ---------------------------------------------------------------------------

#[test]
fn stub_node_has_no_children() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
    assert_eq!(root.named_child_count(), 0);
    assert!(root.child(0).is_none());
    assert!(root.named_child(0).is_none());
}

#[test]
fn child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.child(100).is_none());
}

// ---------------------------------------------------------------------------
// Node sibling/parent stubs
// ---------------------------------------------------------------------------

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
    assert!(tree.root_node().child_by_field_name("foo").is_none());
}

// ---------------------------------------------------------------------------
// Node text extraction
// ---------------------------------------------------------------------------

#[test]
fn utf8_text_on_stub_returns_empty() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let text = root.utf8_text(b"hello world").unwrap();
    assert_eq!(text, ""); // 0..0 range
}

// ---------------------------------------------------------------------------
// Node Debug
// ---------------------------------------------------------------------------

#[test]
fn node_debug_format() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let debug = format!("{:?}", root);
    assert!(debug.contains("Node"));
    assert!(debug.contains("kind"));
    assert!(debug.contains("range"));
}

// ---------------------------------------------------------------------------
// Point
// ---------------------------------------------------------------------------

#[test]
fn point_new_and_accessors() {
    let p = Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_display_is_one_indexed() {
    let p = Point::new(0, 0);
    assert_eq!(format!("{}", p), "1:1");

    let p2 = Point::new(3, 7);
    assert_eq!(format!("{}", p2), "4:8");
}

#[test]
fn point_equality_and_ordering() {
    let a = Point::new(1, 5);
    let b = Point::new(1, 5);
    let c = Point::new(2, 0);
    assert_eq!(a, b);
    assert!(a < c);
}

#[test]
fn point_clone_and_copy() {
    let p = Point::new(1, 2);
    let p2 = p; // Copy
    let p3 = p;
    assert_eq!(p, p2);
    assert_eq!(p, p3);
}

// ---------------------------------------------------------------------------
// Tree: source_bytes
// ---------------------------------------------------------------------------

#[test]
fn tree_source_bytes_none_by_default() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

// source_bytes after set is tested via Parser::parse in builder_tests.rs

// ---------------------------------------------------------------------------
// Tree: language accessor
// ---------------------------------------------------------------------------

#[test]
fn tree_language_none_by_default() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

// language_after_set is tested via Parser::parse in builder_tests.rs

// ---------------------------------------------------------------------------
// Tree: root_kind
// ---------------------------------------------------------------------------

#[test]
fn tree_root_kind_returns_symbol_id() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

// ---------------------------------------------------------------------------
// Tree: clone
// ---------------------------------------------------------------------------

#[test]
fn tree_clone_is_independent() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(cloned.source_bytes(), tree.source_bytes());
    assert_eq!(cloned.root_kind(), tree.root_kind());
}

// ---------------------------------------------------------------------------
// Tree Debug
// ---------------------------------------------------------------------------

#[test]
fn tree_debug_does_not_panic() {
    let tree = Tree::new_stub();
    let debug = format!("{:?}", tree);
    assert!(debug.contains("Tree"));
}

// ---------------------------------------------------------------------------
// TreeCursor: edge cases
// ---------------------------------------------------------------------------

#[test]
fn cursor_on_stub_tree_has_no_children() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    // Already at root
    assert!(!cursor.goto_parent());
    // Still at root, operations still work
    assert!(!cursor.goto_first_child());
}

// ---------------------------------------------------------------------------
// InputEdit
// ---------------------------------------------------------------------------

#[test]
fn input_edit_clone_and_eq() {
    use adze_runtime::InputEdit;
    let edit = InputEdit {
        start_byte: 0,
        old_end_byte: 5,
        new_end_byte: 10,
        start_position: Point::new(0, 0),
        old_end_position: Point::new(0, 5),
        new_end_position: Point::new(0, 10),
    };
    let edit2 = edit;
    assert_eq!(edit.start_byte, edit2.start_byte);
    assert_eq!(edit.new_end_byte, edit2.new_end_byte);
}

// ---------------------------------------------------------------------------
// Node kind resolution with multi-symbol language
// ---------------------------------------------------------------------------

// node_kind with multi-symbol language is tested via builder_tests.rs

// ---------------------------------------------------------------------------
// Token type
// ---------------------------------------------------------------------------

#[test]
fn token_fields() {
    use adze_runtime::Token;
    let tok = Token {
        kind: 3,
        start: 10,
        end: 15,
    };
    assert_eq!(tok.kind, 3);
    assert_eq!(tok.start, 10);
    assert_eq!(tok.end, 15);
}

#[test]
fn token_clone_and_debug() {
    use adze_runtime::Token;
    let tok = Token {
        kind: 1,
        start: 0,
        end: 5,
    };
    let tok2 = tok;
    assert_eq!(tok.kind, tok2.kind);
    let debug = format!("{:?}", tok);
    assert!(debug.contains("Token"));
}
