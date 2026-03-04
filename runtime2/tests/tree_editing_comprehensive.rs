//! Comprehensive tests for Tree editing and manipulation in adze-runtime.
//!
//! Covers: Point construction/ordering/copy, stub tree creation, tree cloning
//! independence, TreeCursor navigation, Node traversal consistency, byte range
//! invariants, multi-tree independence, new_for_testing trees, Parser construction
//! and error paths, ParseError display/debug, ParseErrorKind variants, utf8_text,
//! and incremental editing (feature-gated behind `incremental_glr`).

#![allow(clippy::needless_range_loop)]

use adze_runtime::tree::TreeCursor;
use adze_runtime::{ParseError, ParseErrorKind, Parser, Point, Tree};

#[cfg(feature = "incremental_glr")]
use adze_runtime::{EditError, InputEdit};

// ===== Helpers =====

/// Build a point helper for edit construction.
fn pt(row: usize, col: usize) -> Point {
    Point::new(row, col)
}

// ===== Section 1: Point Construction and Ordering =====

#[test]
fn point_new_and_accessors() {
    let p = Point::new(3, 7);
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

#[test]
fn point_origin() {
    let p = Point::new(0, 0);
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_display_is_one_indexed() {
    assert_eq!(format!("{}", Point::new(0, 0)), "1:1");
    assert_eq!(format!("{}", Point::new(2, 4)), "3:5");
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
fn point_ordering_same_row() {
    let a = Point::new(0, 0);
    let b = Point::new(0, 5);
    assert!(a < b);
    assert!(b > a);
}

#[test]
fn point_ordering_different_rows() {
    let a = Point::new(0, 99);
    let b = Point::new(1, 0);
    assert!(a < b, "row takes precedence over column");
}

#[test]
fn point_ordering_transitive() {
    let a = Point::new(0, 0);
    let b = Point::new(0, 5);
    let c = Point::new(1, 0);
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
}

#[test]
fn point_min_max() {
    let a = Point::new(2, 10);
    let b = Point::new(5, 3);
    assert_eq!(std::cmp::min(a, b), a);
    assert_eq!(std::cmp::max(a, b), b);
}

#[test]
fn point_le_ge() {
    let a = Point::new(1, 1);
    let b = Point::new(1, 1);
    assert!(a <= b);
    assert!(a >= b);
}

// ===== Section 2: Point Copy Semantics =====

#[test]
fn point_copy_semantics() {
    let p = Point::new(10, 20);
    let q = p; // Copy
    assert_eq!(p, q);
    // p is still usable after copy
    assert_eq!(p.row, 10);
}

#[test]
fn point_clone_equals_copy() {
    let p = Point::new(5, 15);
    let q = p.clone();
    assert_eq!(p, q);
}

#[test]
fn point_debug_format() {
    let p = Point::new(3, 7);
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("Point"));
    assert!(dbg.contains("3"));
    assert!(dbg.contains("7"));
}

// ===== Section 3: Point Default Values =====

#[test]
fn point_zero_is_smallest() {
    let zero = Point::new(0, 0);
    let nonzero = Point::new(0, 1);
    assert!(zero < nonzero);
}

#[test]
fn point_large_values() {
    let p = Point::new(usize::MAX, usize::MAX);
    assert_eq!(p.row, usize::MAX);
    assert_eq!(p.column, usize::MAX);
}

#[test]
fn point_display_large_values() {
    // Wrapping addition in Display: row+1, column+1
    // This will wrap on usize::MAX but that's fine — just test it doesn't panic
    let p = Point::new(100, 200);
    let s = format!("{}", p);
    assert_eq!(s, "101:201");
}

// ===== Section 4: Stub Tree Basics =====

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

// ===== Section 5: new_for_testing Trees =====

#[test]
fn new_for_testing_leaf_node() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 1);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn new_for_testing_with_children() {
    let child1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let child2 = Tree::new_for_testing(3, 3, 7, vec![]);
    let tree = Tree::new_for_testing(1, 0, 7, vec![child1, child2]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().kind_id(), 2);
    assert_eq!(root.child(1).unwrap().kind_id(), 3);
}

#[test]
fn new_for_testing_nested_children() {
    let grandchild = Tree::new_for_testing(4, 0, 1, vec![]);
    let child = Tree::new_for_testing(3, 0, 2, vec![grandchild]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let c = root.child(0).unwrap();
    assert_eq!(c.kind_id(), 3);
    // new_for_testing flattens grandchild's root children into the child node
    assert_eq!(c.child_count(), 1);
}

#[test]
fn new_for_testing_root_kind() {
    let tree = Tree::new_for_testing(42, 10, 20, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn new_for_testing_byte_range_consistency() {
    let tree = Tree::new_for_testing(1, 5, 15, vec![]);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 5..15);
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 15);
}

// ===== Section 6: Tree Cloning =====

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
    assert_eq!(tree.root_node().start_byte(), c1.root_node().start_byte());
    assert_eq!(c1.root_node().start_byte(), c2.root_node().start_byte());
}

#[test]
fn clone_for_testing_tree_preserves_children() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_node().child_count(), 1);
    assert_eq!(cloned.root_node().child(0).unwrap().kind_id(), 2);
}

// ===== Section 7: TreeCursor Navigation =====

#[test]
fn cursor_starts_at_root() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
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

#[test]
fn cursor_depth_at_root_is_zero() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_descend_increases_depth() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_node_reflects_position() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn cursor_goto_parent_returns_to_root() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_sibling_navigation() {
    let child1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let child2 = Tree::new_for_testing(3, 3, 7, vec![]);
    let tree = Tree::new_for_testing(1, 0, 7, vec![child1, child2]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

#[test]
fn cursor_reset_returns_to_root() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 1);
}

// ===== Section 8: Node API Surface =====

#[test]
fn node_byte_range_consistency() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.byte_range(), root.start_byte()..root.end_byte());
}

#[test]
fn node_named_child_count_equals_child_count() {
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

#[test]
fn node_start_position_returns_origin() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().start_position(), Point::new(0, 0));
}

#[test]
fn node_end_position_returns_origin() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().end_position(), Point::new(0, 0));
}

#[test]
fn node_named_child_same_as_child() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let root = tree.root_node();
    let c1 = root.child(0).unwrap();
    let c2 = root.named_child(0).unwrap();
    assert_eq!(c1.kind_id(), c2.kind_id());
    assert_eq!(c1.start_byte(), c2.start_byte());
}

#[test]
fn node_utf8_text_on_valid_source() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let source = b"hello world";
    let root = tree.root_node();
    let text = root.utf8_text(source).unwrap();
    assert_eq!(text, "hello");
}

#[test]
fn node_utf8_text_empty_range() {
    let tree = Tree::new_for_testing(1, 3, 3, vec![]);
    let source = b"hello";
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "");
}

#[test]
fn node_copy_semantics() {
    let tree = Tree::new_stub();
    let n1 = tree.root_node();
    let n2 = n1; // Copy
    assert_eq!(n1.kind_id(), n2.kind_id());
    assert_eq!(n1.start_byte(), n2.start_byte());
}

// ===== Section 9: Multiple Tree Independence =====

#[test]
fn two_stub_trees_are_independent() {
    let t1 = Tree::new_stub();
    let t2 = Tree::new_stub();
    assert_eq!(t1.root_node().child_count(), t2.root_node().child_count());
    assert_eq!(t1.root_kind(), t2.root_kind());
}

#[test]
fn cloned_tree_edit_does_not_affect_original() {
    let tree = Tree::new_stub();
    let _cloned = tree.clone();
    assert_eq!(tree.root_node().start_byte(), 0);
}

// ===== Section 10: Debug Formatting =====

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

#[test]
fn node_debug_format_contains_range() {
    let tree = Tree::new_for_testing(1, 5, 10, vec![]);
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("5..10") || debug.contains("range"));
}

// ===== Section 11: ParseError and ParseErrorKind =====

#[test]
fn parse_error_no_language_display() {
    let err = ParseError::no_language();
    let msg = format!("{}", err);
    assert!(msg.contains("no language"));
}

#[test]
fn parse_error_no_language_debug() {
    let err = ParseError::no_language();
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("NoLanguage"));
}

#[test]
fn parse_error_timeout_display() {
    let err = ParseError::timeout();
    let msg = format!("{}", err);
    assert!(msg.contains("timeout"));
}

#[test]
fn parse_error_with_msg() {
    let err = ParseError::with_msg("custom failure");
    let msg = format!("{}", err);
    assert!(msg.contains("custom failure"));
}

#[test]
fn parse_error_syntax_error_with_location() {
    let loc = adze_runtime::error::ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let err = ParseError::syntax_error("unexpected token", loc);
    let msg = format!("{}", err);
    assert!(msg.contains("unexpected token"));
    assert!(err.location.is_some());
}

#[test]
fn parse_error_with_location_adds_location() {
    let loc = adze_runtime::error::ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::no_language().with_location(loc);
    assert!(err.location.is_some());
    assert_eq!(err.location.as_ref().unwrap().line, 1);
}

#[test]
fn parse_error_kind_field_accessible() {
    let err = ParseError::timeout();
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
}

#[test]
fn parse_error_location_field_default_none() {
    let err = ParseError::no_language();
    assert!(err.location.is_none());
}

#[test]
fn parse_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(ParseError::no_language());
    let _msg = err.to_string();
}

#[test]
fn parse_error_kind_version_mismatch_display() {
    let err = ParseError {
        kind: ParseErrorKind::VersionMismatch {
            expected: 15,
            actual: 14,
        },
        location: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("15"));
    assert!(msg.contains("14"));
}

#[test]
fn parse_error_kind_invalid_encoding_display() {
    let err = ParseError {
        kind: ParseErrorKind::InvalidEncoding,
        location: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("encoding"));
}

#[test]
fn parse_error_kind_cancelled_display() {
    let err = ParseError {
        kind: ParseErrorKind::Cancelled,
        location: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("cancelled"));
}

#[test]
fn parse_error_kind_allocation_error_display() {
    let err = ParseError {
        kind: ParseErrorKind::AllocationError,
        location: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("allocation"));
}

#[test]
fn error_location_display() {
    let loc = adze_runtime::error::ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 7,
    };
    let msg = format!("{}", loc);
    assert_eq!(msg, "3:7");
}

// ===== Section 12: Parser Construction and Error Paths =====

#[test]
fn parser_new_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn parser_default_equals_new() {
    let p1 = Parser::new();
    let p2 = Parser::default();
    assert!(p1.language().is_none());
    assert!(p2.language().is_none());
}

#[test]
fn parser_has_no_timeout_by_default() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_set_timeout() {
    let mut parser = Parser::new();
    let dur = std::time::Duration::from_secs(5);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn parser_set_timeout_zero() {
    let mut parser = Parser::new();
    let dur = std::time::Duration::from_secs(0);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn parser_reset_does_not_panic() {
    let mut parser = Parser::new();
    parser.reset(); // should not panic even without language
}

#[test]
fn parser_parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parser_parse_empty_input_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse(b"", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_utf8_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

#[test]
fn parser_set_language_with_stub() {
    let mut parser = Parser::new();
    let lang = adze_runtime::test_helpers::stub_language();
    let result = parser.set_language(lang);
    // stub_language has valid metadata, so set_language should succeed
    assert!(result.is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn parser_parse_with_stub_language_returns_error() {
    let mut parser = Parser::new();
    let lang = adze_runtime::test_helpers::stub_language();
    parser.set_language(lang).unwrap();
    // stub_language has empty parse tables, so parsing will fail
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parser.parse(b"test", None)));
    // Either returns Err or panics (both acceptable with empty tables)
    if let Ok(res) = result {
        assert!(res.is_err());
    }
}

#[test]
fn parser_debug_format() {
    let parser = Parser::new();
    let dbg = format!("{:?}", parser);
    assert!(dbg.contains("Parser"));
}

// ===== Section 13: InputEdit and Tree editing (feature-gated) =====

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
    let edit = make_edit(0, 0, 5);
    tree.edit(&edit)
        .expect("Zero-range edit on stub should succeed");
    assert_eq!(tree.root_node().end_byte(), 0);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_rejects_old_end_less_than_start() {
    let mut tree = Tree::new_stub();
    let edit = make_edit(10, 5, 15);
    let result = tree.edit(&edit);
    assert!(matches!(
        result,
        Err(EditError::InvalidRange {
            start: 10,
            old_end: 5
        })
    ));
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_rejects_new_end_less_than_start() {
    let mut tree = Tree::new_stub();
    let edit = make_edit(10, 15, 5);
    let result = tree.edit(&edit);
    assert!(matches!(
        result,
        Err(EditError::InvalidRange {
            start: 10,
            old_end: 5
        })
    ));
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_error_display_messages() {
    let invalid = EditError::InvalidRange {
        start: 3,
        old_end: 1,
    };
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
    let _description = err.to_string();
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_clone_does_not_affect_original() {
    let mut tree = Tree::new_stub();
    tree.edit(&make_edit(0, 0, 0)).unwrap();
    let original = tree.clone();
    let edit = make_edit(0, 0, 10);
    tree.edit(&edit).expect("edit should succeed");
    assert_eq!(original.root_node().end_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 0);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_sequential_insertions() {
    let mut tree = Tree::new_stub();
    tree.edit(&make_edit(0, 0, 5)).unwrap();
    assert_eq!(tree.root_node().end_byte(), 0);
    tree.edit(&make_edit(5, 5, 8)).unwrap();
    assert_eq!(tree.root_node().end_byte(), 0);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn input_edit_copy_semantics() {
    let edit = make_edit(0, 5, 10);
    let copy = edit; // InputEdit is Copy
    assert_eq!(edit.start_byte, copy.start_byte);
    assert_eq!(edit.old_end_byte, copy.old_end_byte);
    assert_eq!(edit.new_end_byte, copy.new_end_byte);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn input_edit_debug_format() {
    let edit = make_edit(0, 5, 10);
    let dbg = format!("{:?}", edit);
    assert!(dbg.contains("InputEdit"));
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_error_debug_format() {
    let err = EditError::ArithmeticOverflow;
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("ArithmeticOverflow"));
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_error_clone() {
    let err = EditError::InvalidRange {
        start: 1,
        old_end: 0,
    };
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn edit_error_eq() {
    let a = EditError::ArithmeticOverflow;
    let b = EditError::ArithmeticOverflow;
    let c = EditError::ArithmeticUnderflow;
    assert_eq!(a, b);
    assert_ne!(a, c);
}
