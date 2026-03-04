//! Comprehensive tests for the Parser API (v2).
//!
//! 60+ tests covering Parser construction, language management, timeout,
//! reset, parse dispatch, error handling, Tree/Node/TreeCursor inspection,
//! and edge cases.

use adze_runtime::Token;
use adze_runtime::error::{ErrorLocation, ParseError, ParseErrorKind};
use adze_runtime::language::{Language, SymbolMetadata};
use adze_runtime::node::Point;
use adze_runtime::parser::Parser;
use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use adze_runtime::tree::{Tree, TreeCursor};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helper: build a minimal valid Language (needs glr-core features)
// ---------------------------------------------------------------------------

fn minimal_language() -> Language {
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    Language::builder()
        .version(1)
        .parse_table(table)
        .symbol_names(vec!["root".into(), "tok_a".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|_input: &[u8]| Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Token>>)
        .build()
        .unwrap()
}

fn language_with_n_symbols(n: usize) -> Language {
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    Language::builder()
        .version(1)
        .parse_table(table)
        .symbol_names((0..n).map(|i| format!("sym_{i}")).collect())
        .symbol_metadata(
            (0..n)
                .map(|i| SymbolMetadata {
                    is_terminal: i % 2 == 0,
                    is_visible: true,
                    is_supertype: false,
                })
                .collect(),
        )
        .tokenizer(|_: &[u8]| Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Token>>)
        .build()
        .unwrap()
}

// ===================================================================
// 1. Parser construction
// ===================================================================

#[test]
fn new_parser_has_no_language() {
    let p = Parser::new();
    assert!(p.language().is_none());
}

#[test]
fn new_parser_has_no_timeout() {
    let p = Parser::new();
    assert!(p.timeout().is_none());
}

#[test]
fn default_is_equivalent_to_new() {
    let p1 = Parser::new();
    let p2 = Parser::default();
    assert!(p1.language().is_none());
    assert!(p2.language().is_none());
    assert_eq!(p1.timeout(), p2.timeout());
}

#[test]
fn parser_implements_debug() {
    let p = Parser::new();
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("Parser"));
}

// ===================================================================
// 2. set_language – success paths
// ===================================================================

#[test]
fn set_language_succeeds_with_valid_language() {
    let mut p = Parser::new();
    assert!(p.set_language(minimal_language()).is_ok());
    assert!(p.language().is_some());
}

#[test]
fn set_language_returns_unit_on_success() {
    let mut p = Parser::new();
    let res: Result<(), ParseError> = p.set_language(minimal_language());
    assert_eq!(res.unwrap(), ());
}

#[test]
fn language_ref_after_set_has_correct_metadata_len() {
    let mut p = Parser::new();
    p.set_language(minimal_language()).unwrap();
    assert_eq!(p.language().unwrap().symbol_metadata.len(), 2);
}

#[test]
fn set_language_twice_replaces_previous() {
    let mut p = Parser::new();
    p.set_language(minimal_language()).unwrap();
    assert_eq!(p.language().unwrap().symbol_metadata.len(), 2);

    p.set_language(language_with_n_symbols(5)).unwrap();
    assert_eq!(p.language().unwrap().symbol_metadata.len(), 5);
}

#[test]
fn set_language_with_stub_helper() {
    let mut p = Parser::new();
    assert!(p.set_language(stub_language()).is_ok());
}

#[test]
fn set_language_with_multi_symbol_helper() {
    let mut p = Parser::new();
    let lang = multi_symbol_test_language(10);
    assert!(p.set_language(lang).is_ok());
    assert_eq!(p.language().unwrap().symbol_count, 10);
}

// ===================================================================
// 3. set_language – error paths
// ===================================================================

#[test]
fn set_language_rejects_empty_symbol_metadata() {
    let mut p = Parser::new();
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    // builder will succeed (metadata vec is present), but set_language should reject
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![])
        .tokenizer(|_: &[u8]| Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Token>>)
        .build();
    if let Ok(lang) = lang {
        let res = p.set_language(lang);
        assert!(res.is_err(), "empty metadata should be rejected");
    }
}

#[test]
fn set_language_rejects_missing_tokenizer() {
    let mut p = Parser::new();
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .build()
        .unwrap();
    let res = p.set_language(lang);
    assert!(res.is_err(), "missing tokenizer should be rejected");
}

// ===================================================================
// 4. language() accessor
// ===================================================================

#[test]
fn language_is_none_before_set() {
    assert!(Parser::new().language().is_none());
}

#[test]
fn language_version_matches_after_set() {
    let mut p = Parser::new();
    p.set_language(minimal_language()).unwrap();
    assert_eq!(p.language().unwrap().version, 1);
}

#[test]
fn language_symbol_names_preserved() {
    let mut p = Parser::new();
    p.set_language(minimal_language()).unwrap();
    let names = &p.language().unwrap().symbol_names;
    assert_eq!(names[0], "root");
    assert_eq!(names[1], "tok_a");
}

// ===================================================================
// 5. Timeout
// ===================================================================

#[test]
fn set_timeout_and_read_back() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(250));
    assert_eq!(p.timeout(), Some(Duration::from_millis(250)));
}

#[test]
fn set_timeout_overrides_previous() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(100));
    p.set_timeout(Duration::from_secs(2));
    assert_eq!(p.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn timeout_zero_is_allowed() {
    let mut p = Parser::new();
    p.set_timeout(Duration::ZERO);
    assert_eq!(p.timeout(), Some(Duration::ZERO));
}

#[test]
fn timeout_large_value() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(86400));
    assert_eq!(p.timeout().unwrap().as_secs(), 86400);
}

// ===================================================================
// 6. reset()
// ===================================================================

#[test]
fn reset_does_not_panic_on_fresh_parser() {
    let mut p = Parser::new();
    p.reset(); // no-op, must not crash
}

#[test]
fn reset_preserves_language() {
    let mut p = Parser::new();
    p.set_language(minimal_language()).unwrap();
    p.reset();
    // Language should still be present after reset
    assert!(p.language().is_some());
}

#[test]
fn reset_preserves_timeout() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(42));
    p.reset();
    assert_eq!(p.timeout(), Some(Duration::from_millis(42)));
}

#[test]
fn reset_after_multiple_configurations() {
    let mut p = Parser::new();
    p.set_language(minimal_language()).unwrap();
    p.set_timeout(Duration::from_secs(1));
    p.reset();
    // State is preserved
    assert!(p.language().is_some());
    assert!(p.timeout().is_some());
}

// ===================================================================
// 7. parse / parse_utf8 – error paths (no language)
// ===================================================================

#[test]
fn parse_without_language_returns_no_language_error() {
    let mut p = Parser::new();
    let err = p.parse(b"hello", None).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_utf8_without_language_returns_error() {
    let mut p = Parser::new();
    assert!(p.parse_utf8("world", None).is_err());
}

#[test]
fn parse_empty_bytes_without_language_fails() {
    let mut p = Parser::new();
    assert!(p.parse(b"", None).is_err());
}

#[test]
fn parse_utf8_empty_string_without_language_fails() {
    let mut p = Parser::new();
    assert!(p.parse_utf8("", None).is_err());
}

// ===================================================================
// 8. ParseError construction and display
// ===================================================================

#[test]
fn parse_error_no_language_display() {
    let e = ParseError::no_language();
    let msg = e.to_string();
    assert!(msg.contains("no language"), "got: {msg}");
}

#[test]
fn parse_error_timeout_display() {
    let e = ParseError::timeout();
    assert!(e.to_string().contains("timeout"));
}

#[test]
fn parse_error_with_msg_display() {
    let e = ParseError::with_msg("something broke");
    assert!(e.to_string().contains("something broke"));
}

#[test]
fn parse_error_syntax_error_with_location() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let e = ParseError::syntax_error("unexpected token", loc.clone());
    assert!(e.to_string().contains("unexpected token"));
    assert_eq!(e.location.unwrap(), loc);
}

#[test]
fn parse_error_with_location_chaining() {
    let e = ParseError::with_msg("err").with_location(ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    });
    assert!(e.location.is_some());
}

#[test]
fn parse_error_no_language_has_no_location() {
    let e = ParseError::no_language();
    assert!(e.location.is_none());
}

#[test]
fn error_location_display() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 3,
        column: 7,
    };
    assert_eq!(format!("{loc}"), "3:7");
}

#[test]
fn parse_error_kind_variants_are_distinct() {
    let e1 = ParseError::no_language();
    let e2 = ParseError::timeout();
    let e3 = ParseError::with_msg("other");
    assert_ne!(e1.to_string(), e2.to_string());
    assert_ne!(e2.to_string(), e3.to_string());
}

// ===================================================================
// 9. Tree – construction helpers and basic API
// ===================================================================

#[test]
fn tree_new_stub_has_zero_range() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn tree_new_stub_has_no_language() {
    let t = Tree::new_stub();
    assert!(t.language().is_none());
}

#[test]
fn tree_new_stub_has_no_source() {
    let t = Tree::new_stub();
    assert!(t.source_bytes().is_none());
}

#[test]
fn tree_new_stub_root_kind_is_zero() {
    let t = Tree::new_stub();
    assert_eq!(t.root_kind(), 0);
}

#[test]
fn tree_new_for_testing_preserves_range() {
    let t = Tree::new_for_testing(42, 10, 20, vec![]);
    let root = t.root_node();
    assert_eq!(root.kind_id(), 42);
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 20);
}

#[test]
fn tree_new_for_testing_with_children() {
    let child = Tree::new_for_testing(1, 0, 3, vec![]);
    let parent = Tree::new_for_testing(0, 0, 5, vec![child]);
    assert_eq!(parent.root_node().child_count(), 1);
}

#[test]
fn tree_clone_is_deep() {
    let t = Tree::new_for_testing(0, 0, 10, vec![Tree::new_for_testing(1, 0, 5, vec![])]);
    let cloned = t.clone();
    // Verify independence: the cloned tree has the same structure
    assert_eq!(cloned.root_node().child_count(), 1);
    assert_eq!(cloned.root_node().start_byte(), 0);
    assert_eq!(cloned.root_node().end_byte(), 10);
}

#[test]
fn tree_debug_contains_tree() {
    let t = Tree::new_stub();
    let dbg = format!("{:?}", t);
    assert!(dbg.contains("Tree"));
}

// ===================================================================
// 10. Node API
// ===================================================================

#[test]
fn node_kind_without_language_is_unknown() {
    let t = Tree::new_stub();
    assert_eq!(t.root_node().kind(), "unknown");
}

#[test]
fn node_kind_id_returns_symbol() {
    let t = Tree::new_for_testing(7, 0, 5, vec![]);
    assert_eq!(t.root_node().kind_id(), 7);
}

#[test]
fn node_byte_range_matches_start_end() {
    let t = Tree::new_for_testing(0, 3, 8, vec![]);
    let root = t.root_node();
    assert_eq!(root.byte_range(), 3..8);
    assert_eq!(root.start_byte(), 3);
    assert_eq!(root.end_byte(), 8);
}

#[test]
fn node_start_position_is_placeholder() {
    let t = Tree::new_stub();
    assert_eq!(t.root_node().start_position(), Point::new(0, 0));
}

#[test]
fn node_end_position_is_placeholder() {
    let t = Tree::new_stub();
    assert_eq!(t.root_node().end_position(), Point::new(0, 0));
}

#[test]
fn node_is_named_always_true() {
    let t = Tree::new_stub();
    assert!(t.root_node().is_named());
}

#[test]
fn node_is_missing_always_false() {
    let t = Tree::new_stub();
    assert!(!t.root_node().is_missing());
}

#[test]
fn node_is_error_always_false() {
    let t = Tree::new_stub();
    assert!(!t.root_node().is_error());
}

#[test]
fn node_child_count_zero_for_leaf() {
    let t = Tree::new_for_testing(0, 0, 1, vec![]);
    assert_eq!(t.root_node().child_count(), 0);
}

#[test]
fn node_child_returns_none_out_of_bounds() {
    let t = Tree::new_for_testing(0, 0, 1, vec![]);
    assert!(t.root_node().child(0).is_none());
    assert!(t.root_node().child(99).is_none());
}

#[test]
fn node_child_returns_correct_child() {
    let c0 = Tree::new_for_testing(1, 0, 2, vec![]);
    let c1 = Tree::new_for_testing(2, 2, 5, vec![]);
    let parent = Tree::new_for_testing(0, 0, 5, vec![c0, c1]);
    let root = parent.root_node();
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().kind_id(), 1);
    assert_eq!(root.child(1).unwrap().kind_id(), 2);
}

#[test]
fn node_named_child_same_as_child() {
    let c = Tree::new_for_testing(1, 0, 2, vec![]);
    let parent = Tree::new_for_testing(0, 0, 2, vec![c]);
    let root = parent.root_node();
    assert_eq!(root.named_child(0).unwrap().kind_id(), 1);
}

#[test]
fn node_named_child_count_equals_child_count() {
    let c = Tree::new_for_testing(1, 0, 2, vec![]);
    let parent = Tree::new_for_testing(0, 0, 2, vec![c]);
    assert_eq!(
        parent.root_node().named_child_count(),
        parent.root_node().child_count()
    );
}

#[test]
fn node_child_by_field_name_returns_none() {
    let t = Tree::new_stub();
    assert!(t.root_node().child_by_field_name("any").is_none());
}

#[test]
fn node_parent_returns_none() {
    let t = Tree::new_stub();
    assert!(t.root_node().parent().is_none());
}

#[test]
fn node_siblings_return_none() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
    assert!(root.next_named_sibling().is_none());
    assert!(root.prev_named_sibling().is_none());
}

#[test]
fn node_utf8_text_extracts_slice() {
    let source = b"hello world";
    let t = Tree::new_for_testing(0, 6, 11, vec![]);
    let text = t.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "world");
}

#[test]
fn node_utf8_text_empty_range() {
    let source = b"abc";
    let t = Tree::new_for_testing(0, 2, 2, vec![]);
    assert_eq!(t.root_node().utf8_text(source).unwrap(), "");
}

#[test]
fn node_debug_format() {
    let t = Tree::new_for_testing(0, 0, 5, vec![]);
    let dbg = format!("{:?}", t.root_node());
    assert!(dbg.contains("Node"));
    assert!(dbg.contains("range"));
}

#[test]
fn node_is_copy() {
    let t = Tree::new_for_testing(0, 0, 5, vec![]);
    let n = t.root_node();
    let n2 = n; // Copy
    assert_eq!(n.kind_id(), n2.kind_id());
}

// ===================================================================
// 11. TreeCursor
// ===================================================================

fn tree_with_children() -> Tree {
    let gc = Tree::new_for_testing(3, 0, 1, vec![]);
    let c0 = Tree::new_for_testing(1, 0, 3, vec![gc]);
    let c1 = Tree::new_for_testing(2, 3, 5, vec![]);
    Tree::new_for_testing(0, 0, 5, vec![c0, c1])
}

#[test]
fn cursor_starts_at_root() {
    let t = tree_with_children();
    let cursor = TreeCursor::new(&t);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_goto_first_child_on_leaf_returns_false() {
    let t = Tree::new_for_testing(0, 0, 1, vec![]);
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_goto_next_sibling() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn cursor_goto_next_sibling_at_last_child_returns_false() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // now at c1 (last child)
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_traverse_to_grandchild_and_back() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child(); // symbol 1
    cursor.goto_first_child(); // symbol 3 (grandchild)
    assert_eq!(cursor.node().kind_id(), 3);
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent(); // back to symbol 1
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.goto_parent(); // back to root
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_reset_moves_to_root() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.reset(&t);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_depth_increases_with_descent() {
    let t = tree_with_children();
    let mut cursor = TreeCursor::new(&t);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
}

// ===================================================================
// 12. Point type
// ===================================================================

#[test]
fn point_new_constructor() {
    let p = Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_equality() {
    assert_eq!(Point::new(1, 2), Point::new(1, 2));
    assert_ne!(Point::new(1, 2), Point::new(1, 3));
}

#[test]
fn point_ordering() {
    assert!(Point::new(0, 5) < Point::new(1, 0));
    assert!(Point::new(1, 0) < Point::new(1, 1));
}

#[test]
fn point_display() {
    // Display is 1-indexed
    let p = Point::new(0, 0);
    assert_eq!(format!("{p}"), "1:1");
    let p2 = Point::new(3, 7);
    assert_eq!(format!("{p2}"), "4:8");
}

#[test]
fn point_clone_and_copy() {
    let p = Point::new(2, 4);
    let p2 = p;
    assert_eq!(p, p2);
}

// ===================================================================
// 13. Language query methods
// ===================================================================

#[test]
fn language_symbol_name_by_id() {
    let lang = minimal_language();
    assert_eq!(lang.symbol_name(0), Some("root"));
    assert_eq!(lang.symbol_name(1), Some("tok_a"));
    assert_eq!(lang.symbol_name(99), None);
}

#[test]
fn language_is_terminal() {
    let lang = minimal_language();
    assert!(!lang.is_terminal(0)); // root is non-terminal
    assert!(lang.is_terminal(1)); // tok_a is terminal
}

#[test]
fn language_is_visible() {
    let lang = minimal_language();
    assert!(lang.is_visible(0));
    assert!(lang.is_visible(1));
}

#[test]
fn language_field_name_empty() {
    let lang = minimal_language();
    assert!(lang.field_name(0).is_none());
}

#[test]
fn language_symbol_for_name_found() {
    let lang = minimal_language();
    // is_named=true means is_visible=true
    assert_eq!(lang.symbol_for_name("root", true), Some(0));
    assert_eq!(lang.symbol_for_name("tok_a", true), Some(1));
}

#[test]
fn language_symbol_for_name_not_found() {
    let lang = minimal_language();
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
}

// ===================================================================
// 14. Token type
// ===================================================================

#[test]
fn token_fields() {
    let t = Token {
        kind: 5,
        start: 10,
        end: 15,
    };
    assert_eq!(t.kind, 5);
    assert_eq!(t.start, 10);
    assert_eq!(t.end, 15);
}

#[test]
fn token_clone() {
    let t = Token {
        kind: 1,
        start: 0,
        end: 3,
    };
    let t2 = t;
    assert_eq!(t.kind, t2.kind);
}

// ===================================================================
// 15. InputEdit type
// ===================================================================

#[test]
fn input_edit_fields() {
    let edit = adze_runtime::InputEdit {
        start_byte: 5,
        old_end_byte: 10,
        new_end_byte: 15,
        start_position: Point::new(0, 5),
        old_end_position: Point::new(0, 10),
        new_end_position: Point::new(0, 15),
    };
    assert_eq!(edit.start_byte, 5);
    assert_eq!(edit.old_end_byte, 10);
    assert_eq!(edit.new_end_byte, 15);
}

#[test]
fn input_edit_is_copy() {
    let edit = adze_runtime::InputEdit {
        start_byte: 0,
        old_end_byte: 1,
        new_end_byte: 2,
        start_position: Point::new(0, 0),
        old_end_position: Point::new(0, 1),
        new_end_position: Point::new(0, 2),
    };
    let edit2 = edit;
    assert_eq!(edit, edit2);
}

// ===================================================================
// 16. Edge cases
// ===================================================================

#[test]
fn tree_for_testing_deeply_nested() {
    let leaf = Tree::new_for_testing(5, 0, 1, vec![]);
    let mid = Tree::new_for_testing(4, 0, 1, vec![leaf]);
    let top = Tree::new_for_testing(3, 0, 1, vec![mid]);
    let root = top.root_node();
    let c = root.child(0).unwrap();
    assert_eq!(c.kind_id(), 4);
    let gc = c.child(0).unwrap();
    assert_eq!(gc.kind_id(), 5);
    assert_eq!(gc.child_count(), 0);
}

#[test]
fn tree_for_testing_wide_tree() {
    let children: Vec<Tree> = (0..20)
        .map(|i| Tree::new_for_testing(i + 1, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let root = Tree::new_for_testing(0, 0, 20, children);
    assert_eq!(root.root_node().child_count(), 20);
    assert_eq!(root.root_node().child(19).unwrap().kind_id(), 20);
}

#[test]
fn multiple_parsers_independent() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(minimal_language()).unwrap();
    p1.set_timeout(Duration::from_secs(1));
    // p2 is unaffected
    assert!(p2.language().is_none());
    assert!(p2.timeout().is_none());
    // p2 can be configured independently
    p2.set_timeout(Duration::from_millis(500));
    assert_eq!(p1.timeout().unwrap().as_secs(), 1);
    assert_eq!(p2.timeout().unwrap().as_millis(), 500);
}

#[test]
fn node_utf8_text_with_unicode() {
    let source = "café ☕".as_bytes();
    let t = Tree::new_for_testing(0, 0, source.len(), vec![]);
    let text = t.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "café ☕");
}

#[test]
fn node_utf8_text_partial_unicode() {
    let source = "café ☕".as_bytes();
    // "café" is 5 bytes in UTF-8 (é = 2 bytes)
    let t = Tree::new_for_testing(0, 0, 5, vec![]);
    let text = t.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "café");
}
