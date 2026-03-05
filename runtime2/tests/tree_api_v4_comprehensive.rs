//! Comprehensive tests for runtime2 Tree/Node/TreeCursor API.

use adze_runtime::tree::{Tree, TreeCursor};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Leaf node with symbol, start, end, no children.
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Branch node with symbol, start, end, and children.
fn branch(symbol: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(symbol, start, end, children)
}

// ===========================================================================
// 1. Tree construction (8 tests)
// ===========================================================================

#[test]
fn test_construct_leaf_node() {
    let tree = leaf(1, 0, 5);
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 1);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
}

#[test]
fn test_construct_tree_with_children() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn test_construct_stub_tree() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
    assert_eq!(root.child_count(), 0);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn test_construct_deeply_nested_tree() {
    let inner = leaf(3, 2, 3);
    let mid = branch(2, 1, 4, vec![inner]);
    let outer = branch(1, 0, 5, vec![mid]);
    assert_eq!(outer.root_node().child_count(), 1);
}

#[test]
fn test_construct_tree_preserves_byte_ranges() {
    let tree = branch(0, 10, 20, vec![leaf(1, 10, 15), leaf(2, 15, 20)]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 20);
}

#[test]
fn test_construct_tree_zero_length_node() {
    let tree = leaf(5, 3, 3);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), root.end_byte());
}

#[test]
fn test_construct_tree_large_symbol_id() {
    let tree = leaf(65535, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 65535);
}

#[test]
fn test_construct_tree_symbol_id_truncation() {
    // kind_id() returns u16 — symbol values > u16::MAX are truncated.
    let tree = leaf(0x1_0001, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 1); // low 16 bits
}

// ===========================================================================
// 2. Root node properties (8 tests)
// ===========================================================================

#[test]
fn test_root_node_kind_id() {
    let tree = leaf(42, 0, 10);
    assert_eq!(tree.root_node().kind_id(), 42);
}

#[test]
fn test_root_node_start_byte() {
    let tree = leaf(0, 5, 15);
    assert_eq!(tree.root_node().start_byte(), 5);
}

#[test]
fn test_root_node_end_byte() {
    let tree = leaf(0, 5, 15);
    assert_eq!(tree.root_node().end_byte(), 15);
}

#[test]
fn test_root_node_byte_range() {
    let tree = leaf(0, 3, 7);
    assert_eq!(tree.root_node().byte_range(), 3..7);
}

#[test]
fn test_root_node_is_named() {
    let tree = leaf(0, 0, 1);
    assert!(tree.root_node().is_named());
}

#[test]
fn test_root_node_is_not_missing() {
    let tree = leaf(0, 0, 1);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn test_root_node_is_not_error() {
    let tree = leaf(0, 0, 1);
    assert!(!tree.root_node().is_error());
}

#[test]
fn test_root_node_kind_without_language() {
    let tree = leaf(0, 0, 1);
    // Without language, kind() falls back to "unknown"
    assert_eq!(tree.root_node().kind(), "unknown");
}

// ===========================================================================
// 3. Child access (8 tests)
// ===========================================================================

#[test]
fn test_child_access_first() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let first = tree.root_node().child(0).unwrap();
    assert_eq!(first.kind_id(), 1);
}

#[test]
fn test_child_access_second() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let second = tree.root_node().child(1).unwrap();
    assert_eq!(second.kind_id(), 2);
}

#[test]
fn test_child_access_out_of_bounds() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    assert!(tree.root_node().child(1).is_none());
}

#[test]
fn test_child_access_on_leaf_returns_none() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().child(0).is_none());
}

#[test]
fn test_child_count_matches_children() {
    let tree = branch(
        0,
        0,
        15,
        vec![leaf(1, 0, 5), leaf(2, 5, 10), leaf(3, 10, 15)],
    );
    assert_eq!(tree.root_node().child_count(), 3);
}

#[test]
fn test_child_byte_ranges_preserved() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 3), leaf(2, 3, 7), leaf(3, 7, 10)]);
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    let c2 = root.child(2).unwrap();
    assert_eq!(c0.byte_range(), 0..3);
    assert_eq!(c2.byte_range(), 7..10);
}

#[test]
fn test_child_of_child() {
    let grandchild = leaf(3, 2, 4);
    let child = branch(2, 0, 5, vec![grandchild]);
    let tree = branch(1, 0, 5, vec![child]);
    let gc = tree.root_node().child(0).unwrap().child(0).unwrap();
    assert_eq!(gc.kind_id(), 3);
}

#[test]
fn test_named_child_count_equals_child_count() {
    // Phase 1: named_child_count == child_count
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

// ===========================================================================
// 4. TreeCursor navigation (8 tests)
// ===========================================================================

#[test]
fn test_cursor_starts_at_root() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn test_cursor_goto_first_child() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn test_cursor_goto_next_sibling() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn test_cursor_goto_parent() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn test_cursor_parent_at_root_returns_false() {
    let tree = leaf(0, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn test_cursor_first_child_on_leaf_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn test_cursor_next_sibling_at_last_child_returns_false() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn test_cursor_depth_tracking() {
    let gc = leaf(3, 2, 3);
    let child = branch(2, 0, 5, vec![gc]);
    let tree = branch(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);
}

// ===========================================================================
// 5. Tree with text content (5 tests)
// ===========================================================================

#[test]
fn test_utf8_text_extraction() {
    let source = b"hello world";
    let tree = leaf(0, 0, 5);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "hello");
}

#[test]
fn test_utf8_text_child_extraction() {
    let source = b"hello world";
    let tree = branch(0, 0, 11, vec![leaf(1, 0, 5), leaf(2, 6, 11)]);
    let c1 = tree.root_node().child(1).unwrap();
    assert_eq!(c1.utf8_text(source).unwrap(), "world");
}

#[test]
fn test_utf8_text_empty_range() {
    let source = b"abc";
    let tree = leaf(0, 1, 1);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "");
}

#[test]
fn test_utf8_text_full_source() {
    let source = b"full";
    let tree = leaf(0, 0, 4);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "full");
}

#[test]
fn test_utf8_text_unicode() {
    let source = "café".as_bytes();
    // 'c' 'a' 'f' 'é' — 'é' is 2 bytes in UTF-8
    let tree = leaf(0, 0, source.len());
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "café");
}

// ===========================================================================
// 6. Nested tree structures (5 tests)
// ===========================================================================

#[test]
fn test_three_level_nesting() {
    let l2 = leaf(3, 4, 6);
    let l1 = branch(2, 2, 8, vec![l2]);
    let tree = branch(1, 0, 10, vec![l1]);

    let n1 = tree.root_node().child(0).unwrap();
    assert_eq!(n1.kind_id(), 2);
    let n2 = n1.child(0).unwrap();
    assert_eq!(n2.kind_id(), 3);
    assert_eq!(n2.child_count(), 0);
}

#[test]
fn test_cursor_traverses_three_levels() {
    let l2 = leaf(3, 4, 6);
    let l1 = branch(2, 2, 8, vec![l2]);
    let tree = branch(1, 0, 10, vec![l1]);

    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(!cursor.goto_first_child()); // leaf
}

#[test]
fn test_nested_sibling_navigation() {
    let c0 = branch(2, 0, 5, vec![leaf(4, 0, 2), leaf(5, 2, 5)]);
    let c1 = leaf(3, 5, 10);
    let tree = branch(1, 0, 10, vec![c0, c1]);

    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // c0
    cursor.goto_first_child(); // leaf(4)
    assert_eq!(cursor.node().kind_id(), 4);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 5);
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

#[test]
fn test_parent_child_parent_roundtrip() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_parent();
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_nested_child_byte_ranges() {
    let gc = leaf(3, 3, 5);
    let child = branch(2, 1, 7, vec![gc]);
    let tree = branch(1, 0, 10, vec![child]);

    let n = tree.root_node().child(0).unwrap().child(0).unwrap();
    assert_eq!(n.byte_range(), 3..5);
}

// ===========================================================================
// 7. Named vs anonymous nodes (5 tests)
// ===========================================================================

#[test]
fn test_all_nodes_are_named_phase1() {
    // Phase 1: is_named() always returns true.
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    for i in 0..tree.root_node().child_count() {
        assert!(tree.root_node().child(i).unwrap().is_named());
    }
}

#[test]
fn test_named_child_returns_same_as_child() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let root = tree.root_node();
    let c = root.child(0).unwrap();
    let nc = root.named_child(0).unwrap();
    assert_eq!(c.kind_id(), nc.kind_id());
    assert_eq!(c.byte_range(), nc.byte_range());
}

#[test]
fn test_named_child_out_of_bounds() {
    let tree = leaf(0, 0, 5);
    assert!(tree.root_node().named_child(0).is_none());
}

#[test]
fn test_child_by_field_name_returns_none() {
    // Phase 1: field access not implemented.
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    assert!(tree.root_node().child_by_field_name("name").is_none());
}

#[test]
fn test_parent_and_sibling_links_not_stored() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let first = tree.root_node().child(0).unwrap();
    // Sibling/parent links are not stored in Phase 1.
    assert!(first.parent().is_none());
    assert!(first.next_sibling().is_none());
    assert!(first.prev_sibling().is_none());
    assert!(first.next_named_sibling().is_none());
    assert!(first.prev_named_sibling().is_none());
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn test_empty_tree_stub() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
    assert_eq!(tree.root_node().byte_range(), 0..0);
}

#[test]
fn test_single_node_tree() {
    let tree = leaf(99, 0, 42);
    assert_eq!(tree.root_node().kind_id(), 99);
    assert_eq!(tree.root_node().child_count(), 0);
    assert_eq!(tree.root_node().end_byte(), 42);
}

#[test]
fn test_deep_tree_10_levels() {
    // Build a chain: root -> child -> ... -> leaf (10 levels)
    let mut current = leaf(10, 0, 1);
    for sym in (1..10).rev() {
        current = branch(sym, 0, 1, vec![current]);
    }
    let tree = current;

    // Walk down via cursor
    let mut cursor = TreeCursor::new(&tree);
    for expected_depth in 0..9 {
        assert_eq!(cursor.depth(), expected_depth);
        assert!(cursor.goto_first_child());
    }
    assert_eq!(cursor.depth(), 9);
    assert_eq!(cursor.node().kind_id(), 10);
    assert!(!cursor.goto_first_child()); // leaf
}

#[test]
fn test_wide_tree_20_children() {
    let children: Vec<Tree> = (0..20)
        .map(|i| leaf(i + 1, i as usize, (i + 1) as usize))
        .collect();
    let tree = branch(0, 0, 20, children);
    assert_eq!(tree.root_node().child_count(), 20);

    let last = tree.root_node().child(19).unwrap();
    assert_eq!(last.kind_id(), 20);
}

#[test]
fn test_cursor_reset() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);

    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_full_dfs_traversal() {
    // Tree:  0 -> [1, 2 -> [3, 4]]
    let c1 = leaf(1, 0, 2);
    let c2 = branch(2, 2, 8, vec![leaf(3, 2, 5), leaf(4, 5, 8)]);
    let tree = branch(0, 0, 8, vec![c1, c2]);

    let mut cursor = TreeCursor::new(&tree);
    let mut visited = vec![cursor.node().kind_id()];

    // DFS using cursor: try child, else sibling, else parent-sibling
    fn dfs(cursor: &mut TreeCursor<'_>, visited: &mut Vec<u16>) {
        if cursor.goto_first_child() {
            visited.push(cursor.node().kind_id());
            dfs(cursor, visited);
            while cursor.goto_next_sibling() {
                visited.push(cursor.node().kind_id());
                dfs(cursor, visited);
            }
            cursor.goto_parent();
        }
    }
    dfs(&mut cursor, &mut visited);
    assert_eq!(visited, [0, 1, 2, 3, 4]);
}

#[test]
fn test_tree_clone_independence() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let cloned = tree.clone();
    // Both trees should have same structure.
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
}

#[test]
fn test_tree_debug_format() {
    let tree = leaf(0, 0, 5);
    let dbg = format!("{tree:?}");
    assert!(dbg.contains("Tree"));
}

// ===========================================================================
// Additional coverage for 55+ total
// ===========================================================================

#[test]
fn test_root_kind_returns_u32() {
    let tree = leaf(0x1_ABCD, 0, 1);
    assert_eq!(tree.root_kind(), 0x1_ABCD);
}

#[test]
fn test_source_bytes_none_by_default() {
    let tree = leaf(0, 0, 1);
    assert!(tree.source_bytes().is_none());
}

#[test]
fn test_language_none_by_default() {
    let tree = leaf(0, 0, 1);
    assert!(tree.language().is_none());
}

#[test]
fn test_start_position_returns_zero() {
    let tree = leaf(0, 10, 20);
    let pos = tree.root_node().start_position();
    assert_eq!(pos.row, 0);
    assert_eq!(pos.column, 0);
}

#[test]
fn test_end_position_returns_zero() {
    let tree = leaf(0, 10, 20);
    let pos = tree.root_node().end_position();
    assert_eq!(pos.row, 0);
    assert_eq!(pos.column, 0);
}

#[test]
fn test_cursor_sibling_iteration_all() {
    let children: Vec<Tree> = (1..=5).map(|i| leaf(i, 0, 1)).collect();
    let tree = branch(0, 0, 5, children);

    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut ids = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        ids.push(cursor.node().kind_id());
    }
    assert_eq!(ids, [1, 2, 3, 4, 5]);
}

#[test]
fn test_cursor_next_sibling_at_root_returns_false() {
    let tree = leaf(0, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}
