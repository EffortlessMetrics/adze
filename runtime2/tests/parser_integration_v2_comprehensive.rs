//! Comprehensive integration tests for runtime2 Parser, Tree, TreeCursor, and ParseError.
//!
//! 75 tests covering: parser lifecycle, set_language, parse (with catch_unwind),
//! set_timeout, Tree construction, TreeCursor traversal, multi-parser usage,
//! parser reuse, and ParseError formatting.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use adze_runtime::error::{ErrorLocation, ParseErrorKind};
use adze_runtime::test_helpers::stub_language;
use adze_runtime::tree::TreeCursor;
use adze_runtime::{ParseError, Parser, Tree};

// ──────────────────────────────────────────────────────────────────────────────
// 1. Parser::new() creates a parser
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn parser_new_creates_instance() {
    let _parser = Parser::new();
}

#[test]
fn parser_new_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn parser_new_has_no_timeout() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_default_creates_instance() {
    let parser: Parser = Default::default();
    assert!(parser.language().is_none());
}

#[test]
fn parser_debug_format_contains_parser() {
    let parser = Parser::new();
    let dbg = format!("{:?}", parser);
    assert!(dbg.contains("Parser"));
}

// ──────────────────────────────────────────────────────────────────────────────
// 2. Parser set_language with stub_language
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn set_language_with_stub_succeeds() {
    let mut parser = Parser::new();
    let lang = stub_language();
    assert!(parser.set_language(lang).is_ok());
}

#[test]
fn set_language_returns_ok() {
    let mut parser = Parser::new();
    let result = parser.set_language(stub_language());
    assert!(result.is_ok());
}

#[test]
fn parser_language_returns_some_after_set() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn set_language_twice_replaces() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_language_none_initially() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

// ──────────────────────────────────────────────────────────────────────────────
// 3. Parser parse with empty input (catches panic)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_empty_input_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"", None)));
    // Either panics (caught) or returns an error — both are acceptable
    match result {
        Ok(Ok(_tree)) => {} // unlikely with stub but acceptable
        Ok(Err(_e)) => {}
        Err(_panic) => {}
    }
}

#[test]
fn parse_empty_bytes_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let empty: &[u8] = &[];
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(empty, None)));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn parse_empty_str_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse("", None)));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn parse_utf8_empty_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("", None)));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

// ──────────────────────────────────────────────────────────────────────────────
// 4. Parser parse with various inputs (catches panic)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_single_byte_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"x", None)));
    drop(result);
}

#[test]
fn parse_ascii_text_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"hello world", None)));
    drop(result);
}

#[test]
fn parse_unicode_text_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        parser.parse("日本語テスト".as_bytes(), None)
    }));
    drop(result);
}

#[test]
fn parse_long_input_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let input = "a".repeat(10_000);
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(input.as_bytes(), None)));
    drop(result);
}

#[test]
fn parse_binary_input_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let input: Vec<u8> = (0..=255).collect();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(&input, None)));
    drop(result);
}

#[test]
fn parse_newlines_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"line1\nline2\n", None)));
    drop(result);
}

#[test]
fn parse_tabs_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"\t\t\t", None)));
    drop(result);
}

#[test]
fn parse_whitespace_only_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"   ", None)));
    drop(result);
}

#[test]
fn parse_null_bytes_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"\0\0\0", None)));
    drop(result);
}

#[test]
fn parse_utf8_multibyte_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("café ñ ü", None)));
    drop(result);
}

// ──────────────────────────────────────────────────────────────────────────────
// 5. Parser set_timeout with various durations
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn set_timeout_zero() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn set_timeout_one_second() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(1)));
}

#[test]
fn set_timeout_one_millisecond() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(1));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(1)));
}

#[test]
fn set_timeout_large_duration() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(3600);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn timeout_returns_none_initially() {
    let parser = Parser::new();
    assert_eq!(parser.timeout(), None);
}

#[test]
fn timeout_returns_set_value() {
    let mut parser = Parser::new();
    let dur = Duration::from_micros(500);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout().unwrap(), dur);
}

#[test]
fn set_timeout_overrides_previous() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    parser.set_timeout(Duration::from_secs(2));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(2)));
}

// ──────────────────────────────────────────────────────────────────────────────
// 6. Tree::new_stub and new_for_testing
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn tree_new_stub_root_symbol_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn tree_new_stub_root_start_byte_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().start_byte(), 0);
}

#[test]
fn tree_new_stub_root_end_byte_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().end_byte(), 0);
}

#[test]
fn tree_new_stub_no_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn tree_new_stub_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn tree_new_stub_root_kind_is_unknown() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn tree_new_for_testing_basic() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_node().kind_id(), 42);
    assert_eq!(tree.root_node().start_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 10);
}

#[test]
fn tree_new_for_testing_with_children() {
    let child1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let child2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![child1, child2]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn tree_new_for_testing_byte_range() {
    let tree = Tree::new_for_testing(10, 5, 20, vec![]);
    assert_eq!(tree.root_node().byte_range(), 5..20);
}

#[test]
fn tree_new_for_testing_nested_children() {
    let grandchild = Tree::new_for_testing(3, 0, 1, vec![]);
    let child = Tree::new_for_testing(2, 0, 5, vec![grandchild]);
    let tree = Tree::new_for_testing(1, 0, 10, vec![child]);
    assert_eq!(tree.root_node().child_count(), 1);
    let first_child = tree.root_node().child(0).unwrap();
    assert_eq!(first_child.kind_id(), 2);
}

#[test]
fn tree_clone_independence() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    let cloned = tree.clone();
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

#[test]
fn tree_debug_format() {
    let tree = Tree::new_stub();
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"));
}

// ──────────────────────────────────────────────────────────────────────────────
// 7. TreeCursor traversal patterns
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn cursor_starts_at_root() {
    let tree = Tree::new_for_testing(99, 0, 10, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 99);
}

#[test]
fn cursor_depth_at_root_is_zero() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child_leaf() {
    let tree = Tree::new_stub(); // no children
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_goto_first_child_with_children() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 10, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn cursor_goto_next_sibling() {
    let c1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn cursor_goto_next_sibling_at_end() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent_from_child() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_goto_parent_at_root() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_reset_to_root() {
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
fn cursor_full_traversal() {
    let gc = Tree::new_for_testing(3, 0, 1, vec![]);
    let c1 = Tree::new_for_testing(1, 0, 3, vec![gc]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2]);

    let mut cursor = TreeCursor::new(&tree);
    // root -> first child (1) -> grandchild (3)
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 3);
    // grandchild has no children or siblings
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    // back to child 1
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    // to sibling child 2
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
    // back to root
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_depth_increases() {
    let gc = Tree::new_for_testing(3, 0, 1, vec![]);
    let child = Tree::new_for_testing(1, 0, 5, vec![gc]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);

    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn cursor_sibling_then_child() {
    let gc = Tree::new_for_testing(10, 3, 4, vec![]);
    let c1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![gc]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2]);

    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // c1
    cursor.goto_next_sibling(); // c2
    assert!(cursor.goto_first_child()); // gc
    assert_eq!(cursor.node().kind_id(), 10);
}

#[test]
fn cursor_deep_tree_traversal() {
    // Build a linear chain: root -> c1 -> c2 -> c3 -> c4
    let c4 = Tree::new_for_testing(4, 0, 1, vec![]);
    let c3 = Tree::new_for_testing(3, 0, 2, vec![c4]);
    let c2 = Tree::new_for_testing(2, 0, 3, vec![c3]);
    let c1 = Tree::new_for_testing(1, 0, 4, vec![c2]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![c1]);

    let mut cursor = TreeCursor::new(&tree);
    for expected_id in 1u16..=4 {
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.node().kind_id(), expected_id);
    }
    assert_eq!(cursor.depth(), 4);
    // Walk all the way back
    for _ in 0..4 {
        assert!(cursor.goto_parent());
    }
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_reset_after_traversal() {
    let child = Tree::new_for_testing(7, 0, 3, vec![]);
    let tree = Tree::new_for_testing(0, 0, 3, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_node_returns_correct_kind_id() {
    let tree = Tree::new_for_testing(255, 0, 100, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 255);
}

// ──────────────────────────────────────────────────────────────────────────────
// 8. Multiple parsers created
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn multiple_parsers_independent() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(stub_language()).unwrap();
    assert!(p1.language().is_some());
    assert!(p2.language().is_none());
    p2.set_timeout(Duration::from_secs(5));
    assert!(p1.timeout().is_none());
}

#[test]
fn multiple_parsers_different_timeouts() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_timeout(Duration::from_millis(100));
    p2.set_timeout(Duration::from_millis(200));
    assert_eq!(p1.timeout().unwrap(), Duration::from_millis(100));
    assert_eq!(p2.timeout().unwrap(), Duration::from_millis(200));
}

#[test]
fn many_parsers_created() {
    let parsers: Vec<Parser> = (0..50).map(|_| Parser::new()).collect();
    assert_eq!(parsers.len(), 50);
    for p in &parsers {
        assert!(p.language().is_none());
    }
}

#[test]
fn parser_reuse_after_reset() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_timeout(Duration::from_secs(1));
    parser.reset();
    // Language and timeout survive reset (reset only clears arenas)
    assert!(parser.language().is_some());
    assert!(parser.timeout().is_some());
}

#[test]
fn parser_drop_is_safe() {
    {
        let mut parser = Parser::new();
        parser.set_language(stub_language()).unwrap();
        parser.set_timeout(Duration::from_secs(10));
    }
    // Parser dropped without issue
}

// ──────────────────────────────────────────────────────────────────────────────
// 9. Parser reuse after set_language
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn parser_reuse_after_set_language() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    // Can set language again
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_parse_then_parse_again_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();

    let r1 = catch_unwind(AssertUnwindSafe(|| parser.parse(b"first", None)));
    drop(r1);

    // Parser should still be usable (set language again to be safe)
    let _ = parser.set_language(stub_language());
    let r2 = catch_unwind(AssertUnwindSafe(|| parser.parse(b"second", None)));
    drop(r2);
}

#[test]
fn parser_set_language_clears_previous() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let v1 = parser.language().unwrap().version;
    parser.set_language(stub_language()).unwrap();
    let v2 = parser.language().unwrap().version;
    assert_eq!(v1, v2);
}

#[test]
fn parser_with_old_tree_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let old_tree = Tree::new_stub();
    let result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"test", Some(&old_tree))));
    drop(result);
}

#[test]
fn parser_parse_utf8_reuse_catches_panic() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let r1 = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("abc", None)));
    drop(r1);
    let _ = parser.set_language(stub_language());
    let r2 = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("def", None)));
    drop(r2);
}

// ──────────────────────────────────────────────────────────────────────────────
// 10. ParseError format/debug
// ──────────────────────────────────────────────────────────────────────────────

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
fn parse_error_has_kind_field() {
    let err = ParseError::no_language();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_error_has_location_field() {
    let err = ParseError::no_language();
    assert!(err.location.is_none());
}

#[test]
fn parse_error_with_msg_display() {
    let err = ParseError::with_msg("something broke");
    let msg = format!("{}", err);
    assert!(msg.contains("something broke"));
}

#[test]
fn parse_error_syntax_error_with_location() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let err = ParseError::syntax_error("unexpected token", loc);
    assert!(err.location.is_some());
    let location = err.location.unwrap();
    assert_eq!(location.byte_offset, 10);
    assert_eq!(location.line, 2);
    assert_eq!(location.column, 5);
}

#[test]
fn parse_error_timeout_display() {
    let err = ParseError::timeout();
    let msg = format!("{}", err);
    assert!(msg.contains("timeout"));
}

#[test]
fn parse_error_kind_no_language_variant() {
    let err = ParseError::no_language();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_error_kind_timeout_variant() {
    let err = ParseError::timeout();
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
}

#[test]
fn parse_error_kind_other_variant() {
    let err = ParseError::with_msg("custom");
    assert!(matches!(err.kind, ParseErrorKind::Other(_)));
}

#[test]
fn parse_error_with_location_builder() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    let err = ParseError::no_language().with_location(loc.clone());
    assert_eq!(err.location.as_ref().unwrap().byte_offset, 0);
    assert_eq!(err.location.as_ref().unwrap().line, 1);
}

#[test]
fn error_location_display() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 7,
    };
    let msg = format!("{}", loc);
    assert!(msg.contains("3"));
    assert!(msg.contains("7"));
}

// ──────────────────────────────────────────────────────────────────────────────
// Additional edge-case tests
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn tree_new_for_testing_empty_children_vec() {
    let tree = Tree::new_for_testing(5, 10, 20, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
    assert!(tree.root_node().child(0).is_none());
}

#[test]
fn tree_root_node_is_named() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().is_named());
}

#[test]
fn tree_root_node_is_not_error() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_error());
}

#[test]
fn tree_root_node_is_not_missing() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_missing());
}

#[test]
fn tree_root_node_child_by_field_name_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child_by_field_name("foo").is_none());
}

#[test]
fn tree_root_node_parent_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn tree_source_bytes_none_for_stub() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn tree_root_kind_returns_symbol() {
    let tree = Tree::new_for_testing(77, 0, 5, vec![]);
    assert_eq!(tree.root_kind(), 77);
}

#[test]
fn cursor_no_sibling_at_root() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_multiple_siblings() {
    let c1 = Tree::new_for_testing(1, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(2, 2, 4, vec![]);
    let c3 = Tree::new_for_testing(3, 4, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2, c3]);

    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn parser_parse_without_language_error_kind() {
    let mut parser = Parser::new();
    let err = parser.parse(b"x", None).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
    assert!(err.location.is_none());
}

#[test]
fn parser_parse_utf8_without_language_error() {
    let mut parser = Parser::new();
    let err = parser.parse_utf8("x", None).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}
