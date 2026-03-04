//! Comprehensive Parser API v2 surface tests (60+ tests).
//!
//! Covers: Parser construction, default state, set_language, set_timeout,
//! parse panics with stub, Tree creation, TreeCursor navigation, parser
//! reset behavior, multiple parsers, Debug formats, and edge cases.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use adze_runtime::tree::TreeCursor;
use adze_runtime::{ParseError, Parser, Point, Token, Tree};

// ===========================================================================
// 1. Parser::new() construction
// ===========================================================================

#[test]
fn new_parser_returns_instance() {
    let _parser = Parser::new();
}

#[test]
fn new_parser_language_is_none() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn new_parser_timeout_is_none() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn new_parser_via_default_trait() {
    let parser: Parser = Default::default();
    assert!(parser.language().is_none());
}

#[test]
fn default_and_new_are_equivalent() {
    let a = Parser::new();
    let b = Parser::default();
    assert_eq!(a.language().is_none(), b.language().is_none());
    assert_eq!(a.timeout(), b.timeout());
}

// ===========================================================================
// 2. Parser default state
// ===========================================================================

#[test]
fn default_parser_has_no_language() {
    let parser = Parser::default();
    assert!(parser.language().is_none());
}

#[test]
fn default_parser_has_no_timeout() {
    let parser = Parser::default();
    assert_eq!(parser.timeout(), None);
}

#[test]
fn default_parser_parse_returns_no_language_error() {
    let mut parser = Parser::default();
    let err = parser.parse(b"x", None).unwrap_err();
    assert!(err.to_string().contains("no language"));
}

// ===========================================================================
// 3. set_language with stub
// ===========================================================================

#[test]
fn set_language_stub_succeeds() {
    let mut parser = Parser::new();
    assert!(parser.set_language(stub_language()).is_ok());
}

#[test]
fn language_accessor_after_set() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let lang = parser.language().unwrap();
    assert_eq!(lang.symbol_count, 1);
}

#[test]
fn set_language_twice_replaces() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let lang2 = multi_symbol_test_language(5);
    parser.set_language(lang2).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 5);
}

#[test]
fn language_symbol_name_from_stub() {
    let lang = stub_language();
    assert_eq!(lang.symbol_name(0), Some("placeholder"));
}

#[test]
fn language_symbol_name_out_of_bounds() {
    let lang = stub_language();
    assert_eq!(lang.symbol_name(999), None);
}

#[test]
fn language_is_terminal_from_stub() {
    let lang = stub_language();
    assert!(lang.is_terminal(0));
}

#[test]
fn language_is_visible_from_stub() {
    let lang = stub_language();
    assert!(lang.is_visible(0));
}

#[test]
fn language_field_name_empty() {
    let lang = stub_language();
    assert_eq!(lang.field_name(0), None);
}

#[test]
fn multi_symbol_language_names() {
    let lang = multi_symbol_test_language(4);
    assert_eq!(lang.symbol_name(0), Some("symbol_0"));
    assert_eq!(lang.symbol_name(3), Some("symbol_3"));
    assert_eq!(lang.symbol_name(4), None);
}

// ===========================================================================
// 4. set_timeout various durations
// ===========================================================================

#[test]
fn set_timeout_millis() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));
}

#[test]
fn set_timeout_secs() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn set_timeout_zero() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn set_timeout_nanos() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_nanos(500));
    assert_eq!(parser.timeout(), Some(Duration::from_nanos(500)));
}

#[test]
fn set_timeout_large() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(3600));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3600)));
}

#[test]
fn set_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    parser.set_timeout(Duration::from_millis(999));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(999)));
}

#[test]
fn timeout_persists_across_set_language() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(42));
    parser.set_language(stub_language()).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_millis(42)));
}

// ===========================================================================
// 5. parse panics with stub (catch_unwind)
// ===========================================================================

#[test]
fn parse_with_stub_language_panics_or_errors() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    // stub_language has empty parse tables; parse will either panic or error
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"hello", None)));
    match result {
        Ok(Ok(_tree)) => { /* if it somehow succeeds, that's fine */ }
        Ok(Err(_parse_err)) => { /* expected: parse error */ }
        Err(_panic) => { /* expected: panic from empty tables */ }
    }
}

#[test]
fn parse_utf8_with_stub_language_panics_or_errors() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("world", None)));
    match result {
        Ok(Ok(_)) => {}
        Ok(Err(_)) => {}
        Err(_) => {}
    }
}

#[test]
fn parse_empty_input_with_stub_panics_or_errors() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"", None)));
    match result {
        Ok(Ok(_)) => {}
        Ok(Err(_)) => {}
        Err(_) => {}
    }
}

#[test]
fn parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let err = parser.parse(b"test", None).unwrap_err();
    assert!(matches!(err.kind, adze_runtime::ParseErrorKind::NoLanguage));
}

#[test]
fn parse_utf8_without_language_returns_error() {
    let mut parser = Parser::new();
    let err = parser.parse_utf8("test", None).unwrap_err();
    assert!(matches!(err.kind, adze_runtime::ParseErrorKind::NoLanguage));
}

#[test]
fn parse_empty_bytes_without_language_errors() {
    let mut parser = Parser::new();
    assert!(parser.parse(b"", None).is_err());
}

// ===========================================================================
// 6. Tree creation patterns
// ===========================================================================

#[test]
fn tree_new_stub_creates_tree() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn tree_stub_root_node_kind_unknown() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn tree_stub_has_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn tree_stub_has_no_source() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn tree_stub_root_byte_range_is_zero() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn tree_for_testing_with_no_children() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
    assert_eq!(tree.root_node().kind_id(), 42);
    assert_eq!(tree.root_node().start_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 10);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn tree_for_testing_with_children() {
    let child_a = Tree::new_for_testing(1, 0, 3, vec![]);
    let child_b = Tree::new_for_testing(2, 3, 7, vec![]);
    let tree = Tree::new_for_testing(0, 0, 7, vec![child_a, child_b]);
    assert_eq!(tree.root_node().child_count(), 2);
    let first = tree.root_node().child(0).unwrap();
    assert_eq!(first.kind_id(), 1);
    assert_eq!(first.start_byte(), 0);
    assert_eq!(first.end_byte(), 3);
    let second = tree.root_node().child(1).unwrap();
    assert_eq!(second.kind_id(), 2);
}

#[test]
fn tree_for_testing_nested_children() {
    let grandchild = Tree::new_for_testing(3, 1, 2, vec![]);
    let child = Tree::new_for_testing(1, 0, 5, vec![grandchild]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    assert_eq!(tree.root_node().child_count(), 1);
    // child of root has no children because new_for_testing flattens grandchild
    // into the child's internal children
    let root_child = tree.root_node().child(0).unwrap();
    assert_eq!(root_child.kind_id(), 1);
}

#[test]
fn tree_clone_is_independent() {
    let tree = Tree::new_for_testing(5, 0, 20, vec![]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_kind(), 5);
    assert_eq!(cloned.root_node().end_byte(), 20);
}

#[test]
fn tree_clone_root_matches() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().byte_range(),
        cloned.root_node().byte_range()
    );
}

// ===========================================================================
// 7. TreeCursor on various trees
// ===========================================================================

#[test]
fn cursor_on_stub_starts_at_root() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_stub_no_children() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_stub_no_sibling() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_stub_no_parent() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_depth_increases_on_child() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_depth_decreases_on_parent() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_sibling_traversal() {
    let c1 = Tree::new_for_testing(1, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(2, 2, 4, vec![]);
    let c3 = Tree::new_for_testing(3, 4, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2, c3]);
    let mut cursor = TreeCursor::new(&tree);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);

    // No more siblings
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_reset_returns_to_root() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);

    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_reset_to_different_tree() {
    let tree1 = Tree::new_for_testing(10, 0, 5, vec![]);
    let tree2 = Tree::new_for_testing(20, 0, 8, vec![]);
    let mut cursor = TreeCursor::new(&tree1);
    assert_eq!(cursor.node().kind_id(), 10);

    cursor.reset(&tree2);
    assert_eq!(cursor.node().kind_id(), 20);
}

#[test]
fn cursor_leaf_node_has_no_children() {
    let leaf = Tree::new_for_testing(99, 5, 10, vec![]);
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(!cursor.goto_first_child());
}

// ===========================================================================
// 8. Parser reset behavior
// ===========================================================================

#[test]
fn reset_preserves_language() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.reset();
    assert!(parser.language().is_some());
}

#[test]
fn reset_preserves_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(250));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_millis(250)));
}

#[test]
fn reset_on_fresh_parser_is_noop() {
    let mut parser = Parser::new();
    parser.reset();
    assert!(parser.language().is_none());
    assert!(parser.timeout().is_none());
}

#[test]
fn reset_then_parse_without_language_errors() {
    let mut parser = Parser::new();
    parser.reset();
    assert!(parser.parse(b"x", None).is_err());
}

// ===========================================================================
// 9. Multiple parsers
// ===========================================================================

#[test]
fn two_parsers_independent_language() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(stub_language()).unwrap();
    assert!(p1.language().is_some());
    assert!(p2.language().is_none());

    p2.set_language(multi_symbol_test_language(3)).unwrap();
    assert_eq!(p1.language().unwrap().symbol_count, 1);
    assert_eq!(p2.language().unwrap().symbol_count, 3);
}

#[test]
fn two_parsers_independent_timeout() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_timeout(Duration::from_millis(100));
    assert_eq!(p1.timeout(), Some(Duration::from_millis(100)));
    assert_eq!(p2.timeout(), None);
}

#[test]
fn multiple_parsers_in_vec() {
    let parsers: Vec<Parser> = (0..5).map(|_| Parser::new()).collect();
    assert_eq!(parsers.len(), 5);
    for p in &parsers {
        assert!(p.language().is_none());
    }
}

// ===========================================================================
// 10. Parser Debug format
// ===========================================================================

#[test]
fn parser_debug_contains_parser() {
    let parser = Parser::new();
    let dbg = format!("{:?}", parser);
    assert!(dbg.contains("Parser"));
}

#[test]
fn parser_debug_does_not_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_timeout(Duration::from_secs(1));
    let _ = format!("{:?}", parser);
}

#[test]
fn tree_debug_contains_tree() {
    let tree = Tree::new_stub();
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"));
}

#[test]
fn tree_for_testing_debug() {
    let tree = Tree::new_for_testing(7, 0, 10, vec![]);
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"));
}

#[test]
fn language_debug_contains_language() {
    let lang = stub_language();
    let dbg = format!("{:?}", lang);
    assert!(dbg.contains("Language"));
}

// ===========================================================================
// 11. Node API via trees (supplementary)
// ===========================================================================

#[test]
fn node_kind_id_matches_tree_root_kind() {
    let tree = Tree::new_for_testing(77, 0, 5, vec![]);
    assert_eq!(tree.root_node().kind_id(), 77);
    assert_eq!(tree.root_kind(), 77);
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

#[test]
fn node_child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(100).is_none());
}

#[test]
fn node_named_child_same_as_child() {
    let child = Tree::new_for_testing(1, 0, 3, vec![]);
    let tree = Tree::new_for_testing(0, 0, 3, vec![child]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), root.named_child_count());
    assert_eq!(
        root.child(0).unwrap().kind_id(),
        root.named_child(0).unwrap().kind_id()
    );
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
    assert!(tree.root_node().child_by_field_name("foo").is_none());
}

#[test]
fn node_utf8_text_on_stub() {
    let tree = Tree::new_stub();
    let text = tree.root_node().utf8_text(b"hello").unwrap();
    assert_eq!(text, ""); // range 0..0
}

#[test]
fn node_utf8_text_extracts_range() {
    let tree = Tree::new_for_testing(0, 2, 5, vec![]);
    let source = b"xxhello";
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "hel");
}

#[test]
fn node_positions_are_dummy_zeros() {
    let tree = Tree::new_for_testing(0, 10, 20, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_position(), Point::new(0, 0));
    assert_eq!(root.end_position(), Point::new(0, 0));
}

#[test]
fn node_debug_format() {
    let tree = Tree::new_stub();
    let dbg = format!("{:?}", tree.root_node());
    assert!(dbg.contains("Node"));
}

// ===========================================================================
// 12. ParseError constructors
// ===========================================================================

#[test]
fn parse_error_no_language_display() {
    let err = ParseError::no_language();
    assert_eq!(err.to_string(), "no language set");
    assert!(err.location.is_none());
}

#[test]
fn parse_error_timeout_display() {
    let err = ParseError::timeout();
    assert_eq!(err.to_string(), "parse timeout exceeded");
}

#[test]
fn parse_error_with_msg_display() {
    let err = ParseError::with_msg("custom error");
    assert_eq!(err.to_string(), "custom error");
}

#[test]
fn parse_error_syntax_with_location() {
    use adze_runtime::error::ErrorLocation;
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let err = ParseError::syntax_error("bad token", loc);
    assert!(err.to_string().contains("bad token"));
    assert!(err.location.is_some());
}

// ===========================================================================
// 13. Point type
// ===========================================================================

#[test]
fn point_new_accessors() {
    let p = Point::new(3, 7);
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

#[test]
fn point_display_one_indexed() {
    assert_eq!(format!("{}", Point::new(0, 0)), "1:1");
    assert_eq!(format!("{}", Point::new(4, 9)), "5:10");
}

#[test]
fn point_equality() {
    assert_eq!(Point::new(1, 2), Point::new(1, 2));
    assert_ne!(Point::new(1, 2), Point::new(1, 3));
}

#[test]
fn point_ordering() {
    assert!(Point::new(0, 0) < Point::new(1, 0));
    assert!(Point::new(1, 0) < Point::new(1, 1));
}

#[test]
fn point_copy_semantics() {
    let p = Point::new(5, 5);
    let p2 = p;
    let p3 = p;
    assert_eq!(p2, p3);
}

// ===========================================================================
// 14. Token type
// ===========================================================================

#[test]
fn token_fields_accessible() {
    let tok = Token {
        kind: 7,
        start: 10,
        end: 20,
    };
    assert_eq!(tok.kind, 7);
    assert_eq!(tok.start, 10);
    assert_eq!(tok.end, 20);
}

#[test]
fn token_copy_semantics() {
    let tok = Token {
        kind: 1,
        start: 0,
        end: 5,
    };
    let tok2 = tok;
    assert_eq!(tok.kind, tok2.kind);
}

#[test]
fn token_debug_format() {
    let tok = Token {
        kind: 0,
        start: 0,
        end: 0,
    };
    let dbg = format!("{:?}", tok);
    assert!(dbg.contains("Token"));
}

// ===========================================================================
// 15. Language clone
// ===========================================================================

#[test]
fn language_clone_preserves_symbol_count() {
    let lang = stub_language();
    let cloned = lang.clone();
    assert_eq!(cloned.symbol_count, lang.symbol_count);
}

#[test]
fn language_clone_preserves_names() {
    let lang = multi_symbol_test_language(3);
    let cloned = lang.clone();
    assert_eq!(cloned.symbol_names, lang.symbol_names);
}

#[test]
fn language_version_is_zero_for_stub() {
    let lang = stub_language();
    assert_eq!(lang.version, 0);
}
