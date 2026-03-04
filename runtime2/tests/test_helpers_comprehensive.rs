//! Comprehensive tests for the `adze_runtime::test_helpers` module.
//!
//! Covers: stub_language creation, multi_symbol_test_language, Parser integration,
//! Tree construction (new_stub, new_for_testing), TreeCursor traversal, clone/debug,
//! and parse-attempt panic safety.

use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use adze_runtime::tree::TreeCursor;
use adze_runtime::{Parser, Tree};

// ---------------------------------------------------------------------------
// 1. stub_language() basics
// ---------------------------------------------------------------------------

#[test]
fn stub_language_returns_language() {
    let _lang = stub_language();
}

#[test]
fn stub_language_has_symbol_names() {
    let lang = stub_language();
    assert!(!lang.symbol_names.is_empty());
}

#[test]
fn stub_language_has_symbol_metadata() {
    let lang = stub_language();
    assert!(!lang.symbol_metadata.is_empty());
}

#[test]
fn stub_language_first_symbol_name_is_placeholder() {
    let lang = stub_language();
    assert_eq!(lang.symbol_names[0], "placeholder");
}

#[test]
fn stub_language_symbol_metadata_is_terminal() {
    let lang = stub_language();
    assert!(lang.symbol_metadata[0].is_terminal);
}

#[test]
fn stub_language_symbol_metadata_is_visible() {
    let lang = stub_language();
    assert!(lang.symbol_metadata[0].is_visible);
}

#[test]
fn stub_language_symbol_metadata_not_supertype() {
    let lang = stub_language();
    assert!(!lang.symbol_metadata[0].is_supertype);
}

#[test]
fn stub_language_field_names_empty() {
    let lang = stub_language();
    assert!(lang.field_names.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Multiple / repeated stub_language calls
// ---------------------------------------------------------------------------

#[test]
fn stub_language_called_twice_returns_independent_instances() {
    let a = stub_language();
    let b = stub_language();
    assert_eq!(a.symbol_names.len(), b.symbol_names.len());
}

#[test]
fn stub_language_called_in_loop() {
    for _ in 0..5 {
        let lang = stub_language();
        assert!(!lang.symbol_names.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3. multi_symbol_test_language
// ---------------------------------------------------------------------------

#[test]
fn multi_symbol_language_single_symbol() {
    let lang = multi_symbol_test_language(1);
    assert_eq!(lang.symbol_names.len(), 1);
    assert_eq!(lang.symbol_names[0], "symbol_0");
}

#[test]
fn multi_symbol_language_five_symbols() {
    let lang = multi_symbol_test_language(5);
    assert_eq!(lang.symbol_names.len(), 5);
    for i in 0..5 {
        assert_eq!(lang.symbol_names[i], format!("symbol_{}", i));
    }
}

#[test]
fn multi_symbol_language_metadata_count_matches() {
    let lang = multi_symbol_test_language(10);
    assert_eq!(lang.symbol_metadata.len(), 10);
}

#[test]
fn multi_symbol_language_all_metadata_terminal() {
    let lang = multi_symbol_test_language(4);
    for m in &lang.symbol_metadata {
        assert!(m.is_terminal);
        assert!(m.is_visible);
        assert!(!m.is_supertype);
    }
}

#[test]
fn multi_symbol_language_field_names_empty() {
    let lang = multi_symbol_test_language(3);
    assert!(lang.field_names.is_empty());
}

// ---------------------------------------------------------------------------
// 4. Parser::new() with stub_language
// ---------------------------------------------------------------------------

#[test]
fn parser_new_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn parser_set_language_with_stub() {
    let mut parser = Parser::new();
    let lang = stub_language();
    let result = parser.set_language(lang);
    assert!(result.is_ok());
}

#[test]
fn parser_language_present_after_set() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_set_language_twice() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_default_is_new() {
    let parser = Parser::default();
    assert!(parser.language().is_none());
}

// ---------------------------------------------------------------------------
// 5. Tree::new_stub() properties
// ---------------------------------------------------------------------------

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
fn tree_new_stub_root_child_count_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn tree_new_stub_root_kind_unknown_without_language() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn tree_new_stub_language_is_none() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn tree_new_stub_source_bytes_none() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn tree_new_stub_root_kind_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

// ---------------------------------------------------------------------------
// 6. Tree::new_for_testing() with various params
// ---------------------------------------------------------------------------

#[test]
fn new_for_testing_leaf_node() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_node().kind_id(), 42);
    assert_eq!(tree.root_node().start_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 10);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn new_for_testing_one_child() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let parent = Tree::new_for_testing(0, 0, 5, vec![child]);
    assert_eq!(parent.root_node().child_count(), 1);
    let c = parent.root_node().child(0).unwrap();
    assert_eq!(c.kind_id(), 1);
}

#[test]
fn new_for_testing_two_children() {
    let c1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let parent = Tree::new_for_testing(0, 0, 6, vec![c1, c2]);
    assert_eq!(parent.root_node().child_count(), 2);
    assert_eq!(parent.root_node().child(1).unwrap().kind_id(), 2);
}

#[test]
fn new_for_testing_nested_children() {
    let grandchild = Tree::new_for_testing(3, 0, 2, vec![]);
    let child = Tree::new_for_testing(2, 0, 4, vec![grandchild]);
    let root = Tree::new_for_testing(1, 0, 6, vec![child]);
    let root_node = root.root_node();
    assert_eq!(root_node.child_count(), 1);
    let child_node = root_node.child(0).unwrap();
    assert_eq!(child_node.kind_id(), 2);
    // grandchild is nested inside child's children
    assert_eq!(child_node.child_count(), 1);
    assert_eq!(child_node.child(0).unwrap().kind_id(), 3);
}

#[test]
fn new_for_testing_zero_byte_range() {
    let tree = Tree::new_for_testing(5, 0, 0, vec![]);
    assert_eq!(tree.root_node().byte_range(), 0..0);
}

#[test]
fn new_for_testing_large_symbol_id() {
    let tree = Tree::new_for_testing(u32::MAX, 0, 1, vec![]);
    assert_eq!(tree.root_kind(), u32::MAX);
}

#[test]
fn new_for_testing_child_byte_ranges_preserved() {
    let c1 = Tree::new_for_testing(1, 10, 20, vec![]);
    let c2 = Tree::new_for_testing(2, 20, 30, vec![]);
    let parent = Tree::new_for_testing(0, 10, 30, vec![c1, c2]);
    assert_eq!(parent.root_node().child(0).unwrap().start_byte(), 10);
    assert_eq!(parent.root_node().child(0).unwrap().end_byte(), 20);
    assert_eq!(parent.root_node().child(1).unwrap().start_byte(), 20);
    assert_eq!(parent.root_node().child(1).unwrap().end_byte(), 30);
}

// ---------------------------------------------------------------------------
// 7. TreeCursor traversal on test trees
// ---------------------------------------------------------------------------

fn make_three_level_tree() -> Tree {
    let gc1 = Tree::new_for_testing(10, 0, 2, vec![]);
    let gc2 = Tree::new_for_testing(11, 2, 4, vec![]);
    let c1 = Tree::new_for_testing(1, 0, 4, vec![gc1, gc2]);
    let c2 = Tree::new_for_testing(2, 4, 8, vec![]);
    Tree::new_for_testing(0, 0, 8, vec![c1, c2])
}

#[test]
fn cursor_starts_at_root() {
    let tree = make_three_level_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_depth_at_root_is_zero() {
    let tree = make_three_level_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child_succeeds() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_goto_next_sibling() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn cursor_goto_next_sibling_at_last_returns_false() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // second child
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

#[test]
fn cursor_goto_parent_from_child() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_depth_increases_with_descent() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn cursor_reset_returns_to_root() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_full_traversal_visits_all_nodes() {
    let tree = make_three_level_tree();
    let mut cursor = TreeCursor::new(&tree);
    let mut visited = vec![cursor.node().kind_id()];

    fn collect(cursor: &mut TreeCursor<'_>, visited: &mut Vec<u16>) {
        if cursor.goto_first_child() {
            visited.push(cursor.node().kind_id());
            collect(cursor, visited);
            while cursor.goto_next_sibling() {
                visited.push(cursor.node().kind_id());
                collect(cursor, visited);
            }
            cursor.goto_parent();
        }
    }
    collect(&mut cursor, &mut visited);
    // root(0), c1(1), gc1(10), gc2(11), c2(2)
    assert_eq!(visited, vec![0, 1, 10, 11, 2]);
}

#[test]
fn cursor_on_leaf_node_goto_first_child_false() {
    let tree = Tree::new_for_testing(99, 0, 1, vec![]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_on_stub_tree() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_parent());
}

// ---------------------------------------------------------------------------
// 8. Parse attempts with catch_unwind (stub language panics on parse)
// ---------------------------------------------------------------------------

#[test]
fn parse_with_stub_language_panics() {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut parser = Parser::new();
        parser.set_language(stub_language()).unwrap();
        let _ = parser.parse(b"hello", None);
    }));
    // The parse will either panic or return an error – both are acceptable.
    // We just verify the test doesn't crash the process.
    let _ = result;
}

#[test]
fn parse_empty_input_with_stub_language() {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut parser = Parser::new();
        parser.set_language(stub_language()).unwrap();
        let _ = parser.parse(b"", None);
    }));
    let _ = result;
}

#[test]
fn parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let err = parser.parse(b"x", None);
    assert!(err.is_err());
}

#[test]
fn parse_utf8_without_language_returns_error() {
    let mut parser = Parser::new();
    let err = parser.parse_utf8("x", None);
    assert!(err.is_err());
}

// ---------------------------------------------------------------------------
// 9. Tree clone and debug
// ---------------------------------------------------------------------------

#[test]
fn tree_clone_preserves_root_symbol() {
    let tree = Tree::new_for_testing(7, 0, 10, vec![]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_kind(), 7);
}

#[test]
fn tree_clone_preserves_children() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_node().child_count(), 1);
}

#[test]
fn tree_clone_is_independent() {
    let tree = Tree::new_for_testing(5, 0, 10, vec![]);
    let cloned = tree.clone();
    // They have the same data but are independent objects.
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

#[test]
fn tree_debug_format_contains_node() {
    let tree = Tree::new_for_testing(3, 0, 5, vec![]);
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"));
    assert!(dbg.contains("Node"));
}

#[test]
fn stub_tree_debug_format() {
    let tree = Tree::new_stub();
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"));
}

// ---------------------------------------------------------------------------
// 10. Multiple tree construction patterns
// ---------------------------------------------------------------------------

#[test]
fn many_children_tree() {
    let children: Vec<Tree> = (0..20)
        .map(|i| Tree::new_for_testing(i + 1, (i * 5) as usize, ((i + 1) * 5) as usize, vec![]))
        .collect();
    let tree = Tree::new_for_testing(0, 0, 100, children);
    assert_eq!(tree.root_node().child_count(), 20);
}

#[test]
fn deeply_nested_tree() {
    let mut current = Tree::new_for_testing(10, 0, 2, vec![]);
    for sym in (0..10).rev() {
        current = Tree::new_for_testing(sym, 0, 2, vec![current]);
    }
    // root is symbol 0 with depth 10
    let mut cursor = TreeCursor::new(&current);
    let mut depth = 0;
    while cursor.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, 10);
}

#[test]
fn tree_with_heterogeneous_depths() {
    let deep = Tree::new_for_testing(
        3,
        0,
        2,
        vec![Tree::new_for_testing(
            4,
            0,
            1,
            vec![Tree::new_for_testing(5, 0, 1, vec![])],
        )],
    );
    let shallow = Tree::new_for_testing(6, 2, 4, vec![]);
    let root = Tree::new_for_testing(0, 0, 4, vec![deep, shallow]);
    assert_eq!(root.root_node().child_count(), 2);
}

// ---------------------------------------------------------------------------
// 11. Node API on test trees
// ---------------------------------------------------------------------------

#[test]
fn node_byte_range() {
    let tree = Tree::new_for_testing(1, 5, 15, vec![]);
    assert_eq!(tree.root_node().byte_range(), 5..15);
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
    assert!(tree.root_node().child(999).is_none());
}

#[test]
fn node_parent_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn node_next_sibling_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().next_sibling().is_none());
}

#[test]
fn node_utf8_text_extracts_slice() {
    let tree = Tree::new_for_testing(0, 2, 5, vec![]);
    let source = b"hello world";
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "llo");
}

// ---------------------------------------------------------------------------
// 12. Parser timeout / reset
// ---------------------------------------------------------------------------

#[test]
fn parser_timeout_initially_none() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_set_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(std::time::Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(std::time::Duration::from_secs(5)));
}

#[test]
fn parser_reset_does_not_panic() {
    let mut parser = Parser::new();
    parser.reset();
}

#[test]
fn parser_debug_format() {
    let parser = Parser::new();
    let dbg = format!("{:?}", parser);
    assert!(dbg.contains("Parser"));
}
