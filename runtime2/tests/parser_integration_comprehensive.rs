//! Comprehensive tests for the Parser API in runtime2 (adze-runtime).

use adze_runtime::ParseError;
use adze_runtime::language::{Language, SymbolMetadata};
use adze_runtime::parser::Parser;
use adze_runtime::tree::{Tree, TreeCursor};

fn leak_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
}

fn one_meta() -> Vec<SymbolMetadata> {
    vec![SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    }]
}

fn build_lang() -> Language {
    Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(one_meta())
        .tokenizer(
            |_input: &[u8]| -> Box<dyn Iterator<Item = adze_runtime::Token> + '_> {
                Box::new(std::iter::empty())
            },
        )
        .build()
        .expect("build language")
}

// =============================================================================
// Parser creation
// =============================================================================

#[test]
fn parser_new_creates_instance() {
    let _parser = Parser::new();
}

#[test]
fn parser_default_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

// =============================================================================
// Language setting
// =============================================================================

#[test]
fn parser_set_language_accepts_language() {
    let mut parser = Parser::new();
    let result = parser.set_language(build_lang());
    assert!(result.is_ok(), "set_language failed: {:?}", result.err());
}

#[test]
fn parser_language_returns_set_language() {
    let mut parser = Parser::new();
    parser.set_language(build_lang()).unwrap();
    assert!(parser.language().is_some());
}

// =============================================================================
// Parse operations
// =============================================================================

#[test]
fn parser_parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse("hello", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_empty_string_without_language() {
    let mut parser = Parser::new();
    let result = parser.parse("", None);
    assert!(result.is_err());
}

// =============================================================================
// Language builder
// =============================================================================

#[test]
fn language_builder_requires_parse_table() {
    let result = Language::builder().symbol_metadata(one_meta()).build();
    assert!(result.is_err());
}

#[test]
fn language_builder_requires_symbol_metadata() {
    let result = Language::builder().parse_table(leak_table()).build();
    assert!(result.is_err());
}

#[test]
fn language_builder_with_all_required_fields() {
    let result = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(one_meta())
        .build();
    assert!(result.is_ok());
}

#[test]
fn language_clone() {
    let lang = build_lang();
    let lang2 = lang.clone();
    drop(lang);
    drop(lang2);
}

#[test]
fn language_symbol_for_name_nonexistent() {
    let lang = build_lang();
    assert!(lang.symbol_for_name("nonexistent", true).is_none());
}

// =============================================================================
// Tree API
// =============================================================================

#[test]
fn tree_new_stub_creates_tree() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
}

#[test]
fn tree_root_node_returns_node() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn tree_for_testing_basic() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 10);
}

#[test]
fn tree_for_testing_with_children() {
    let c1 = Tree::new_for_testing(2, 0, 5, vec![]);
    let c2 = Tree::new_for_testing(3, 5, 10, vec![]);
    let tree = Tree::new_for_testing(1, 0, 10, vec![c1, c2]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn tree_node_child_access() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    assert!(tree.root_node().child(0).is_some());
}

#[test]
fn tree_node_child_out_of_bounds() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(100).is_none());
}

// =============================================================================
// TreeCursor
// =============================================================================

#[test]
fn tree_cursor_creation() {
    let tree = Tree::new_stub();
    let _cursor = TreeCursor::new(&tree);
}

#[test]
fn tree_cursor_node_at_root() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 10);
}

#[test]
fn tree_cursor_leaf_has_no_children() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn tree_cursor_root_has_no_parent() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn tree_cursor_child_and_back() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().start_byte(), 0);
}

#[test]
fn tree_cursor_sibling_navigation() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let tree = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().start_byte(), 3);
    assert!(!cursor.goto_next_sibling());
}

// =============================================================================
// Multiple parsers
// =============================================================================

#[test]
fn multiple_parsers_independent() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(build_lang()).unwrap();
    assert!(p1.language().is_some());
    assert!(p2.language().is_none());
    let _ = p2.parse("x", None);
}

// =============================================================================
// ParseError
// =============================================================================

#[test]
fn parse_error_no_language() {
    let err = ParseError::no_language();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ParseError>();
}

// =============================================================================
// Language Debug
// =============================================================================

#[test]
fn language_debug_output() {
    let lang = build_lang();
    let debug = format!("{:?}", lang);
    assert!(debug.contains("Language"));
}

// =============================================================================
// Tree structure tests
// =============================================================================

#[test]
fn tree_deep_nesting() {
    let mut tree = Tree::new_for_testing(100, 0, 1, vec![]);
    for i in (1..=20).rev() {
        tree = Tree::new_for_testing(i, 0, 1, vec![tree]);
    }
    assert_eq!(tree.root_node().child_count(), 1);
}

#[test]
fn tree_wide_children() {
    let children: Vec<Tree> = (0..50)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 50, children);
    assert_eq!(tree.root_node().child_count(), 50);
}

// =============================================================================
// Language builder with symbol names
// =============================================================================

#[test]
fn language_with_symbol_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(one_meta())
        .symbol_names(vec!["end".to_string()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn language_with_field_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(one_meta())
        .field_names(vec!["name".to_string()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn language_builder_chaining() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(one_meta())
        .symbol_names(vec!["end".to_string()])
        .field_names(vec!["name".to_string()])
        .build();
    assert!(lang.is_ok());
}
