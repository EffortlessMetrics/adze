//! Comprehensive tests for the Parser API surface.
//!
//! Covers: Parser construction, language setting, parse errors,
//! Tree building/inspection, Node API, and TreeCursor traversal.

use adze_runtime::error::ParseError;
use adze_runtime::language::{Language, SymbolMetadata};
use adze_runtime::node::Point;
use adze_runtime::parser::Parser;
use adze_runtime::tree::{Tree, TreeCursor};
use std::time::Duration;

fn minimal_language() -> Language {
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        }])
        .tokenizer(|_input: &[u8]| {
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = adze_runtime::Token>>
        })
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// 1. Parser construction
// ---------------------------------------------------------------------------

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
fn parser_debug_impl() {
    let parser = Parser::new();
    let debug = format!("{:?}", parser);
    assert!(debug.contains("Parser"));
}

// ---------------------------------------------------------------------------
// 2. Language setting
// ---------------------------------------------------------------------------

#[test]
fn parser_set_language_success() {
    let mut parser = Parser::new();
    let lang = minimal_language();
    let result = parser.set_language(lang);
    assert!(result.is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn parser_language_returns_ref_after_set() {
    let mut parser = Parser::new();
    let lang = minimal_language();
    parser.set_language(lang).unwrap();
    let lang_ref = parser.language().unwrap();
    assert!(!lang_ref.symbol_metadata.is_empty());
}

#[test]
fn parser_set_language_twice_replaces() {
    let mut parser = Parser::new();
    let lang1 = minimal_language();
    let lang2 = {
        let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
        Language::builder()
            .version(14)
            .parse_table(table)
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
            .tokenizer(|_input: &[u8]| {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = adze_runtime::Token>>
            })
            .build()
            .unwrap()
    };
    parser.set_language(lang1).unwrap();
    assert_eq!(parser.language().unwrap().symbol_metadata.len(), 1);
    parser.set_language(lang2).unwrap();
    assert_eq!(parser.language().unwrap().symbol_metadata.len(), 2);
}

#[test]
fn parser_set_language_empty_metadata_fails() {
    let mut parser = Parser::new();
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    let lang = Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_metadata(vec![])
        .tokenizer(|_input: &[u8]| {
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = adze_runtime::Token>>
        })
        .build();
    // Builder may reject empty metadata
    if let Ok(lang) = lang {
        let result = parser.set_language(lang);
        assert!(result.is_err());
    }
}

#[test]
fn parser_set_language_no_tokenizer_check() {
    let mut parser = Parser::new();
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    let lang = Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        }])
        .build()
        .unwrap();
    // Parser requires tokenizer for set_language
    let result = parser.set_language(lang);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 3. Parse without language (error paths)
// ---------------------------------------------------------------------------

#[test]
fn parser_parse_without_language_fails() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_utf8_without_language_fails() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_empty_input_without_language() {
    let mut parser = Parser::new();
    let result = parser.parse(b"", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_utf8_empty_input_without_language() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("", None);
    assert!(result.is_err());
}

#[test]
fn parse_error_display() {
    let err = ParseError::no_language();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_with_msg() {
    let err = ParseError::with_msg("custom error");
    let msg = format!("{}", err);
    assert!(msg.contains("custom"));
}

// ---------------------------------------------------------------------------
// 4. Timeout behaviour
// ---------------------------------------------------------------------------

#[test]
fn parser_set_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn parser_set_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    parser.set_timeout(Duration::from_millis(100));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));
}

#[test]
fn parser_zero_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn parser_large_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(3600));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3600)));
}

// ---------------------------------------------------------------------------
// 5. Reset behaviour
// ---------------------------------------------------------------------------

#[test]
fn parser_reset_does_not_clear_language() {
    let mut parser = Parser::new();
    let lang = minimal_language();
    parser.set_language(lang).unwrap();
    parser.reset();
    assert!(parser.language().is_some());
}

#[test]
fn parser_reset_does_not_clear_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(10)));
}

// ---------------------------------------------------------------------------
// 6. Tree building and inspection
// ---------------------------------------------------------------------------

#[test]
fn tree_stub_has_zero_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_new_for_testing_leaf() {
    let tree = Tree::new_for_testing(42, 0, 5, vec![]);
    assert_eq!(tree.root_kind(), 42);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_new_for_testing_with_children() {
    let child_a = Tree::new_for_testing(1, 0, 3, vec![]);
    let child_b = Tree::new_for_testing(2, 3, 7, vec![]);
    let tree = Tree::new_for_testing(0, 0, 7, vec![child_a, child_b]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().kind_id(), 1);
    assert_eq!(root.child(1).unwrap().kind_id(), 2);
}

#[test]
fn tree_language_is_none_for_test_tree() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![]);
    assert!(tree.language().is_none());
}

#[test]
fn tree_clone_is_independent() {
    let tree = Tree::new_for_testing(5, 0, 10, vec![]);
    let cloned = tree.clone();
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(tree.root_kind(), cloned.root_kind());
}

// ---------------------------------------------------------------------------
// 7. Node API
// ---------------------------------------------------------------------------

#[test]
fn node_kind_without_language_is_unknown() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn node_kind_id_matches_symbol() {
    let tree = Tree::new_for_testing(99, 0, 10, vec![]);
    assert_eq!(tree.root_node().kind_id(), 99);
}

#[test]
fn node_byte_range() {
    let tree = Tree::new_for_testing(0, 3, 15, vec![]);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 3..15);
}

#[test]
fn node_is_named_returns_true() {
    let tree = Tree::new_for_testing(0, 0, 1, vec![]);
    assert!(tree.root_node().is_named());
}

#[test]
fn node_is_error_returns_false() {
    let tree = Tree::new_for_testing(0, 0, 1, vec![]);
    assert!(!tree.root_node().is_error());
}

#[test]
fn node_is_missing_returns_false() {
    let tree = Tree::new_for_testing(0, 0, 1, vec![]);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn node_child_out_of_bounds_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(100).is_none());
}

#[test]
fn node_parent_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn node_siblings_return_none() {
    let child = Tree::new_for_testing(1, 0, 3, vec![]);
    let tree = Tree::new_for_testing(0, 0, 3, vec![child]);
    let first_child = tree.root_node().child(0).unwrap();
    assert!(first_child.next_sibling().is_none());
    assert!(first_child.prev_sibling().is_none());
    assert!(first_child.next_named_sibling().is_none());
    assert!(first_child.prev_named_sibling().is_none());
}

#[test]
fn node_child_by_field_name_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    assert!(tree.root_node().child_by_field_name("body").is_none());
}

#[test]
fn node_utf8_text_extracts_slice() {
    let source = b"hello world";
    let tree = Tree::new_for_testing(0, 6, 11, vec![]);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "world");
}

#[test]
fn node_start_end_positions_are_zero() {
    let tree = Tree::new_for_testing(0, 10, 20, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_position(), Point::new(0, 0));
    assert_eq!(root.end_position(), Point::new(0, 0));
}

#[test]
fn node_named_child_count_equals_child_count() {
    let c1 = Tree::new_for_testing(1, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(2, 2, 4, vec![]);
    let tree = Tree::new_for_testing(0, 0, 4, vec![c1, c2]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn node_debug_format() {
    let tree = Tree::new_for_testing(7, 2, 8, vec![]);
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("Node"));
    assert!(debug.contains("2..8"));
}

// ---------------------------------------------------------------------------
// 8. TreeCursor traversal
// ---------------------------------------------------------------------------

#[test]
fn cursor_starts_at_root() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_goto_first_child_on_leaf_returns_false() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_goto_next_sibling_at_root_returns_false() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_descend_and_traverse_siblings() {
    let c1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let c3 = Tree::new_for_testing(3, 6, 9, vec![]);
    let tree = Tree::new_for_testing(0, 0, 9, vec![c1, c2, c3]);

    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().kind_id(), 1);

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);

    // No more siblings
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent_returns_to_root() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);

    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);

    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
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
fn cursor_nested_traversal() {
    let grandchild = Tree::new_for_testing(2, 0, 3, vec![]);
    let child = Tree::new_for_testing(1, 0, 3, vec![grandchild]);
    let tree = Tree::new_for_testing(0, 0, 3, vec![child]);

    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // depth 1: symbol 1
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().kind_id(), 1);

    assert!(cursor.goto_first_child()); // depth 2: symbol 2
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().kind_id(), 2);

    assert!(!cursor.goto_first_child()); // leaf — no children

    assert!(cursor.goto_parent()); // back to depth 1
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_parent()); // back to root
    assert_eq!(cursor.depth(), 0);
}
