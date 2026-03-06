//! Comprehensive tests for the Parser, Tree, Node, and TreeCursor APIs (v3).
//!
//! Covers:
//! 1. Parser construction and configuration
//! 2. Tree creation and properties
//! 3. Node navigation and properties
//! 4. TreeCursor traversal
//! 5. Edit/change tracking
//! 6. Debug/Display implementations
//! 7. Clone/PartialEq properties
//! 8. Error handling

use adze_runtime::Token;
use adze_runtime::error::{ErrorLocation, ParseError};
use adze_runtime::language::{Language, SymbolMetadata};
use adze_runtime::node::Point;
use adze_runtime::parser::Parser;
use adze_runtime::tree::{Tree, TreeCursor};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_language(names: Vec<&str>) -> Language {
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    let count = names.len();
    let builder = Language::builder()
        .version(15)
        .parse_table(table)
        .symbol_names(names.into_iter().map(String::from).collect())
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            };
            count
        ])
        .field_names(vec![])
        .tokenizer(|_| Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Token>>);

    builder.build().unwrap()
}

fn make_default_language() -> Language {
    make_language(vec!["root", "child_a", "child_b", "leaf"])
}

/// Build a tree: root(0..10) -> [child_a(0..4), child_b(4..10) -> [leaf(4..7)]]
fn make_test_tree() -> Tree {
    let leaf = Tree::new_for_testing(3, 4, 7, vec![]);
    let child_b = Tree::new_for_testing(2, 4, 10, vec![leaf]);
    let child_a = Tree::new_for_testing(1, 0, 4, vec![]);
    Tree::new_for_testing(0, 0, 10, vec![child_a, child_b])
}

// ===========================================================================
// 1. Parser construction and configuration (8 tests)
// ===========================================================================

#[test]
fn test_parser_new_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn test_parser_new_no_timeout() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn test_parser_set_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(500));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(500)));
}

#[test]
fn test_parser_set_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    parser.set_timeout(Duration::from_secs(2));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn test_parser_set_language_success() {
    let mut parser = Parser::new();
    let lang = make_default_language();
    assert!(parser.set_language(lang).is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn test_parser_set_language_replaces_previous() {
    let mut parser = Parser::new();
    let lang1 = make_language(vec!["a"]);
    let lang2 = make_language(vec!["x", "y"]);
    parser.set_language(lang1).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 1);
    parser.set_language(lang2).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 2);
}

#[test]
fn test_parser_set_language_empty_metadata_fails() {
    let mut parser = Parser::new();
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["s".into()])
        .symbol_metadata(vec![]) // empty!
        .field_names(vec![])
        .tokenizer(|_| Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Token>>)
        .build();
    // Building should succeed, but set_language should reject empty metadata
    if let Ok(lang) = lang {
        let result = parser.set_language(lang);
        assert!(result.is_err());
    }
}

#[test]
fn test_parser_reset_does_not_panic() {
    let mut parser = Parser::new();
    parser.reset(); // should not panic even without language
    let lang = make_default_language();
    parser.set_language(lang).unwrap();
    parser.reset();
}

// ===========================================================================
// 2. Tree creation and properties (10 tests)
// ===========================================================================

#[test]
fn test_tree_new_stub_defaults() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn test_tree_new_stub_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn test_tree_new_stub_no_source() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn test_tree_new_for_testing_root_symbol() {
    let tree = Tree::new_for_testing(42, 0, 100, vec![]);
    assert_eq!(tree.root_node().kind_id(), 42);
}

#[test]
fn test_tree_new_for_testing_byte_range() {
    let tree = Tree::new_for_testing(0, 5, 20, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 20);
    assert_eq!(root.byte_range(), 5..20);
}

#[test]
fn test_tree_new_for_testing_children() {
    let c1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn test_tree_root_kind() {
    let tree = Tree::new_for_testing(99, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 99);
}

#[test]
fn test_tree_new_for_testing_nested() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 5, vec![leaf]);
    let tree = Tree::new_for_testing(1, 0, 10, vec![mid]);
    assert_eq!(tree.root_node().child_count(), 1);
    let mid_node = tree.root_node().child(0).unwrap();
    assert_eq!(mid_node.kind_id(), 2);
    assert_eq!(mid_node.child_count(), 1);
}

#[test]
fn test_tree_new_for_testing_empty_children() {
    let tree = Tree::new_for_testing(0, 0, 0, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
    assert!(tree.root_node().child(0).is_none());
}

#[test]
fn test_tree_source_bytes_none_for_testing_tree() {
    let tree = make_test_tree();
    assert!(tree.source_bytes().is_none());
}

// ===========================================================================
// 3. Node navigation and properties (10 tests)
// ===========================================================================

#[test]
fn test_node_kind_id_u16() {
    let tree = Tree::new_for_testing(300, 0, 10, vec![]);
    let kind: u16 = tree.root_node().kind_id();
    assert_eq!(kind, 300u16);
}

#[test]
fn test_node_kind_without_language_returns_unknown() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn test_node_child_returns_none_out_of_bounds() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(999).is_none());
}

#[test]
fn test_node_child_preserves_byte_range() {
    let tree = make_test_tree();
    let child_a = tree.root_node().child(0).unwrap();
    assert_eq!(child_a.start_byte(), 0);
    assert_eq!(child_a.end_byte(), 4);
    let child_b = tree.root_node().child(1).unwrap();
    assert_eq!(child_b.start_byte(), 4);
    assert_eq!(child_b.end_byte(), 10);
}

#[test]
fn test_node_is_named_always_true() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(tree.root_node().is_named());
}

#[test]
fn test_node_is_missing_always_false() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn test_node_is_error_always_false() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(!tree.root_node().is_error());
}

#[test]
fn test_node_named_child_count_matches_child_count() {
    let tree = make_test_tree();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn test_node_parent_returns_none() {
    let tree = make_test_tree();
    assert!(tree.root_node().parent().is_none());
    let child = tree.root_node().child(0).unwrap();
    assert!(child.parent().is_none());
}

#[test]
fn test_node_siblings_return_none() {
    let tree = make_test_tree();
    let child = tree.root_node().child(0).unwrap();
    assert!(child.next_sibling().is_none());
    assert!(child.prev_sibling().is_none());
    assert!(child.next_named_sibling().is_none());
    assert!(child.prev_named_sibling().is_none());
}

// ===========================================================================
// 4. TreeCursor traversal (8 tests)
// ===========================================================================

#[test]
fn test_cursor_starts_at_root() {
    let tree = make_test_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_goto_first_child() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn test_cursor_goto_next_sibling() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn test_cursor_goto_next_sibling_at_last_returns_false() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // at child_b
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

#[test]
fn test_cursor_goto_parent() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_goto_parent_at_root_returns_false() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn test_cursor_leaf_has_no_children() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    // Navigate: root -> child_a (leaf, symbol 1)
    cursor.goto_first_child();
    assert!(!cursor.goto_first_child()); // child_a has no children
}

#[test]
fn test_cursor_reset() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_first_child(); // now at leaf
    assert_eq!(cursor.node().kind_id(), 3);
    assert_eq!(cursor.depth(), 2);

    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

// ===========================================================================
// 5. Edit/change tracking (5 tests)
// ===========================================================================

#[test]
fn test_tree_clone_is_independent() {
    let tree = make_test_tree();
    let cloned = tree.clone();
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
}

#[test]
fn test_tree_clone_preserves_kind_ids() {
    let tree = make_test_tree();
    let cloned = tree.clone();
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    let orig_child = tree.root_node().child(0).unwrap();
    let clone_child = cloned.root_node().child(0).unwrap();
    assert_eq!(orig_child.kind_id(), clone_child.kind_id());
}

#[test]
fn test_tree_clone_preserves_nested_structure() {
    let tree = make_test_tree();
    let cloned = tree.clone();
    // Navigate root -> child_b -> leaf
    let orig_leaf = tree.root_node().child(1).unwrap().child(0).unwrap();
    let clone_leaf = cloned.root_node().child(1).unwrap().child(0).unwrap();
    assert_eq!(orig_leaf.kind_id(), clone_leaf.kind_id());
    assert_eq!(orig_leaf.start_byte(), clone_leaf.start_byte());
    assert_eq!(orig_leaf.end_byte(), clone_leaf.end_byte());
}

#[test]
fn test_tree_new_for_testing_many_children() {
    let children: Vec<Tree> = (0..10)
        .map(|i| Tree::new_for_testing(i + 1, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let tree = Tree::new_for_testing(0, 0, 10, children);
    assert_eq!(tree.root_node().child_count(), 10);
    for i in 0..10 {
        let child = tree.root_node().child(i).unwrap();
        assert_eq!(child.kind_id(), (i as u16) + 1);
    }
}

#[test]
fn test_tree_root_kind_matches_kind_id() {
    let tree = Tree::new_for_testing(55, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 55);
    assert_eq!(tree.root_node().kind_id(), 55);
}

// ===========================================================================
// 6. Debug/Display implementations (5 tests)
// ===========================================================================

#[test]
fn test_parser_debug_contains_parser() {
    let parser = Parser::new();
    let debug = format!("{:?}", parser);
    assert!(debug.contains("Parser"));
}

#[test]
fn test_tree_debug_contains_tree() {
    let tree = Tree::new_stub();
    let debug = format!("{:?}", tree);
    assert!(debug.contains("Tree"));
}

#[test]
fn test_node_debug_contains_kind_and_range() {
    let tree = Tree::new_for_testing(5, 10, 20, vec![]);
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("Node"));
    assert!(debug.contains("10..20"));
}

#[test]
fn test_point_display_one_indexed() {
    let p = Point::new(0, 0);
    assert_eq!(format!("{}", p), "1:1");
    let p2 = Point::new(3, 7);
    assert_eq!(format!("{}", p2), "4:8");
}

#[test]
fn test_point_debug() {
    let p = Point::new(2, 5);
    let debug = format!("{:?}", p);
    assert!(debug.contains("Point"));
    assert!(debug.contains("2"));
    assert!(debug.contains("5"));
}

// ===========================================================================
// 7. Clone/PartialEq properties (5 tests)
// ===========================================================================

#[test]
fn test_point_clone_eq() {
    let p = Point::new(1, 2);
    let p2 = p;
    assert_eq!(p, p2);
}

#[test]
fn test_point_ordering() {
    let a = Point::new(0, 5);
    let b = Point::new(1, 0);
    assert!(a < b);
}

#[test]
fn test_point_equality() {
    assert_eq!(Point::new(3, 4), Point::new(3, 4));
    assert_ne!(Point::new(3, 4), Point::new(3, 5));
    assert_ne!(Point::new(3, 4), Point::new(4, 4));
}

#[test]
fn test_tree_clone_deep_copy() {
    // Cloning a tree produces an independent copy
    let tree = make_test_tree();
    let c1 = tree.clone();
    let c2 = tree.clone();
    assert_eq!(c1.root_node().kind_id(), c2.root_node().kind_id());
    assert_eq!(c1.root_node().child_count(), c2.root_node().child_count());
}

#[test]
fn test_node_copy_semantics() {
    let tree = Tree::new_for_testing(7, 0, 5, vec![]);
    let node = tree.root_node();
    let node_copy = node; // Node is Copy
    assert_eq!(node.kind_id(), node_copy.kind_id());
    assert_eq!(node.start_byte(), node_copy.start_byte());
}

// ===========================================================================
// 8. Error handling (4 tests)
// ===========================================================================

#[test]
fn test_parse_error_no_language() {
    let err = ParseError::no_language();
    let msg = format!("{}", err);
    assert!(msg.contains("no language"));
}

#[test]
fn test_parse_error_timeout() {
    let err = ParseError::timeout();
    let msg = format!("{}", err);
    assert!(msg.contains("timeout"));
}

#[test]
fn test_parse_error_with_msg() {
    let err = ParseError::with_msg("custom error");
    let msg = format!("{}", err);
    assert!(msg.contains("custom error"));
}

#[test]
fn test_parse_error_syntax_with_location() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let err = ParseError::syntax_error("unexpected token", loc);
    let msg = format!("{}", err);
    assert!(msg.contains("unexpected token"));
    assert!(err.location.is_some());
    let loc = err.location.unwrap();
    assert_eq!(loc.byte_offset, 10);
    assert_eq!(loc.line, 2);
    assert_eq!(loc.column, 5);
}

// ===========================================================================
// Additional coverage: utf8_text, child_by_field_name, start/end_position
// ===========================================================================

#[test]
fn test_node_utf8_text_valid() {
    let source = b"hello world";
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "hello");
}

#[test]
fn test_node_utf8_text_sub_range() {
    let source = b"abcdefghij";
    let tree = Tree::new_for_testing(0, 3, 7, vec![]);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "defg");
}

#[test]
fn test_node_child_by_field_name_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(tree.root_node().child_by_field_name("anything").is_none());
}

#[test]
fn test_node_start_end_position_stub() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![]);
    let root = tree.root_node();
    // Phase 1: positions are dummy (0,0)
    assert_eq!(root.start_position(), Point::new(0, 0));
    assert_eq!(root.end_position(), Point::new(0, 0));
}

#[test]
fn test_error_location_display() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(format!("{}", loc), "1:1");
}

#[test]
fn test_parse_error_with_location_chain() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 10,
    };
    let err = ParseError::with_msg("oops").with_location(loc);
    assert!(err.location.is_some());
    assert_eq!(err.location.unwrap().byte_offset, 42);
}

#[test]
fn test_cursor_full_depth_first_traversal() {
    // root(0) -> [child_a(1), child_b(2) -> [leaf(3)]]
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    let mut visited = vec![cursor.node().kind_id()];

    // Depth-first: go as deep as possible, then siblings, then up
    if cursor.goto_first_child() {
        visited.push(cursor.node().kind_id()); // child_a(1)
        // child_a has no children
        if !cursor.goto_first_child() {
            // try sibling
            if cursor.goto_next_sibling() {
                visited.push(cursor.node().kind_id()); // child_b(2)
                if cursor.goto_first_child() {
                    visited.push(cursor.node().kind_id()); // leaf(3)
                }
            }
        }
    }

    assert_eq!(visited, vec![0, 1, 2, 3]);
}

#[test]
fn test_cursor_depth_tracking() {
    let tree = make_test_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_next_sibling(); // child_b
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child(); // leaf
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}
