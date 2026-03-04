//! Comprehensive tests for forest-to-tree builder patterns, Tree construction,
//! TreeCursor API, and various tree shapes.

use adze_runtime::Tree;
use adze_runtime::tree::TreeCursor;

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

/// Leaf node helper: symbol with byte range, no children.
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Internal node helper: symbol with byte range and children.
fn node(symbol: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(symbol, start, end, children)
}

// ===== Section 1: Tree::new_stub() =====

#[test]
fn stub_tree_root_has_zero_symbol() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
}

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
fn stub_tree_has_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn stub_tree_debug_format() {
    let tree = Tree::new_stub();
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"));
}

// ===== Section 2: Tree::new_for_testing — single node =====

#[test]
fn single_node_symbol() {
    let tree = leaf(42, 0, 10);
    assert_eq!(tree.root_node().kind_id(), 42);
}

#[test]
fn single_node_byte_range() {
    let tree = leaf(1, 5, 15);
    assert_eq!(tree.root_node().start_byte(), 5);
    assert_eq!(tree.root_node().end_byte(), 15);
}

#[test]
fn single_node_no_children() {
    let tree = leaf(1, 0, 1);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn zero_byte_node() {
    let tree = leaf(7, 0, 0);
    assert_eq!(tree.root_node().start_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 0);
    assert_eq!(tree.root_node().byte_range(), 0..0);
}

#[test]
fn adjacent_byte_range_node() {
    let tree = leaf(1, 10, 10);
    assert_eq!(tree.root_node().byte_range(), 10..10);
}

// ===== Section 3: Tree::new_for_testing — with children =====

#[test]
fn parent_with_one_child() {
    let child = leaf(2, 0, 3);
    let tree = node(1, 0, 3, vec![child]);
    assert_eq!(tree.root_node().child_count(), 1);
    assert_eq!(tree.root_node().child(0).unwrap().kind_id(), 2);
}

#[test]
fn parent_with_two_children() {
    let tree = node(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    assert_eq!(tree.root_node().child_count(), 2);
    assert_eq!(tree.root_node().child(0).unwrap().kind_id(), 2);
    assert_eq!(tree.root_node().child(1).unwrap().kind_id(), 3);
}

#[test]
fn child_out_of_bounds_returns_none() {
    let tree = leaf(1, 0, 1);
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(100).is_none());
}

#[test]
fn nested_children_two_levels() {
    let grandchild = leaf(3, 0, 2);
    let child = node(2, 0, 2, vec![grandchild]);
    let tree = node(1, 0, 2, vec![child]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let c = root.child(0).unwrap();
    assert_eq!(c.kind_id(), 2);
    assert_eq!(c.child_count(), 1);
    assert_eq!(c.child(0).unwrap().kind_id(), 3);
}

#[test]
fn children_adjacent_byte_ranges() {
    let tree = node(0, 0, 9, vec![leaf(1, 0, 3), leaf(2, 3, 6), leaf(3, 6, 9)]);
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().end_byte(), 3);
    assert_eq!(root.child(1).unwrap().start_byte(), 3);
    assert_eq!(root.child(1).unwrap().end_byte(), 6);
    assert_eq!(root.child(2).unwrap().start_byte(), 6);
}

// ===== Section 4: Wide trees (many children) =====

#[test]
fn wide_tree_ten_children() {
    let children: Vec<Tree> = (0..10)
        .map(|i| leaf(i + 1, i as usize, (i + 1) as usize))
        .collect();
    let tree = node(0, 0, 10, children);
    assert_eq!(tree.root_node().child_count(), 10);
    for i in 0..10 {
        assert_eq!(tree.root_node().child(i).unwrap().kind_id(), (i + 1) as u16);
    }
}

#[test]
fn wide_tree_hundred_children() {
    let children: Vec<Tree> = (0..100)
        .map(|i| leaf(i + 1, i as usize, (i + 1) as usize))
        .collect();
    let tree = node(0, 0, 100, children);
    assert_eq!(tree.root_node().child_count(), 100);
    assert_eq!(tree.root_node().child(99).unwrap().kind_id(), 100);
}

// ===== Section 5: Deeply nested trees =====

#[test]
fn deeply_nested_tree_depth_50() {
    let mut current = leaf(50, 0, 1);
    for sym in (0..50).rev() {
        current = node(sym, 0, 1, vec![current]);
    }
    // Walk down with cursor
    let mut cursor = TreeCursor::new(&current);
    for expected_depth in 0..50 {
        assert_eq!(cursor.depth(), expected_depth);
        assert_eq!(cursor.node().kind_id(), expected_depth as u16);
        assert!(cursor.goto_first_child());
    }
    assert_eq!(cursor.node().kind_id(), 50);
    assert!(!cursor.goto_first_child()); // leaf
}

#[test]
fn deeply_nested_tree_go_back_up() {
    let mut current = leaf(5, 0, 1);
    for sym in (0..5).rev() {
        current = node(sym, 0, 1, vec![current]);
    }
    let mut cursor = TreeCursor::new(&current);
    // Go all the way down
    while cursor.goto_first_child() {}
    assert_eq!(cursor.depth(), 5);
    // Go all the way back up
    while cursor.goto_parent() {}
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

// ===== Section 6: TreeCursor — basic navigation =====

#[test]
fn cursor_starts_at_root() {
    let tree = leaf(42, 0, 5);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 42);
}

#[test]
fn cursor_goto_first_child_on_leaf_returns_false() {
    let tree = leaf(1, 0, 1);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_next_sibling_at_root_returns_false() {
    let tree = leaf(1, 0, 1);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = leaf(1, 0, 1);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_depth_increases_on_child() {
    let tree = node(0, 0, 5, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_depth_decreases_on_parent() {
    let tree = node(0, 0, 5, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

// ===== Section 7: TreeCursor — sibling iteration =====

#[test]
fn cursor_sibling_iteration_visits_all() {
    let tree = node(0, 0, 3, vec![leaf(1, 0, 1), leaf(2, 1, 2), leaf(3, 2, 3)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());

    let mut visited = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        visited.push(cursor.node().kind_id());
    }
    assert_eq!(visited, vec![1, 2, 3]);
}

#[test]
fn cursor_no_more_siblings_returns_false() {
    let tree = node(0, 0, 2, vec![leaf(1, 0, 1), leaf(2, 1, 2)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling()); // to child 2
    assert!(!cursor.goto_next_sibling()); // no more
}

#[test]
fn cursor_sibling_then_parent_then_sibling_again() {
    let tree = node(
        0,
        0,
        6,
        vec![
            node(
                1,
                0,
                3,
                vec![leaf(10, 0, 1), leaf(11, 1, 2), leaf(12, 2, 3)],
            ),
            leaf(2, 3, 6),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);

    // Go to first child (symbol 1)
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);

    // Go to grandchild (symbol 10)
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 10);

    // Iterate siblings: 10 -> 11 -> 12
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 11);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 12);
    assert!(!cursor.goto_next_sibling());

    // Back to parent (symbol 1), then to sibling (symbol 2)
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

// ===== Section 8: TreeCursor — DFS traversal =====

fn dfs_collect(tree: &Tree) -> Vec<u16> {
    let mut result = vec![];
    let mut cursor = TreeCursor::new(tree);
    dfs_walk(&mut cursor, &mut result);
    result
}

fn dfs_walk(cursor: &mut TreeCursor<'_>, result: &mut Vec<u16>) {
    result.push(cursor.node().kind_id());
    if cursor.goto_first_child() {
        loop {
            dfs_walk(cursor, result);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

#[test]
fn dfs_traversal_single_node() {
    let tree = leaf(1, 0, 1);
    assert_eq!(dfs_collect(&tree), vec![1]);
}

#[test]
fn dfs_traversal_linear_chain() {
    let tree = node(0, 0, 1, vec![node(1, 0, 1, vec![leaf(2, 0, 1)])]);
    assert_eq!(dfs_collect(&tree), vec![0, 1, 2]);
}

#[test]
fn dfs_traversal_balanced_binary() {
    //       0
    //      / \
    //     1   2
    //    / \
    //   3   4
    let tree = node(
        0,
        0,
        10,
        vec![
            node(1, 0, 5, vec![leaf(3, 0, 2), leaf(4, 2, 5)]),
            leaf(2, 5, 10),
        ],
    );
    assert_eq!(dfs_collect(&tree), vec![0, 1, 3, 4, 2]);
}

#[test]
fn dfs_traversal_wide_flat() {
    let children: Vec<Tree> = (1..=5).map(|i| leaf(i, 0, 1)).collect();
    let tree = node(0, 0, 1, children);
    assert_eq!(dfs_collect(&tree), vec![0, 1, 2, 3, 4, 5]);
}

// ===== Section 9: TreeCursor — reset =====

#[test]
fn cursor_reset_returns_to_root() {
    let tree = node(0, 0, 5, vec![leaf(1, 0, 2), leaf(2, 2, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);

    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_reset_allows_fresh_traversal() {
    let tree = node(0, 0, 3, vec![leaf(1, 0, 1), leaf(2, 1, 2), leaf(3, 2, 3)]);
    let mut cursor = TreeCursor::new(&tree);

    // First traversal
    let first = dfs_collect(&tree);

    // Navigate partway
    cursor.goto_first_child();
    cursor.goto_next_sibling();

    // Reset and traverse again
    cursor.reset(&tree);
    let mut second = vec![];
    dfs_walk(&mut cursor, &mut second);
    assert_eq!(first, second);
}

#[test]
fn cursor_reset_to_different_tree() {
    let tree1 = leaf(10, 0, 5);
    let tree2 = leaf(20, 5, 10);
    let mut cursor = TreeCursor::new(&tree1);
    assert_eq!(cursor.node().kind_id(), 10);

    cursor.reset(&tree2);
    assert_eq!(cursor.node().kind_id(), 20);
    assert_eq!(cursor.depth(), 0);
}

// ===== Section 10: Tree clone =====

#[test]
fn clone_preserves_structure() {
    let tree = node(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let cloned = tree.clone();
    assert_eq!(dfs_collect(&tree), dfs_collect(&cloned));
}

#[test]
fn clone_preserves_byte_ranges() {
    let tree = node(0, 0, 20, vec![leaf(1, 0, 10), leaf(2, 10, 20)]);
    let cloned = tree.clone();
    let orig_root = tree.root_node();
    let clone_root = cloned.root_node();
    assert_eq!(orig_root.start_byte(), clone_root.start_byte());
    assert_eq!(orig_root.end_byte(), clone_root.end_byte());
    assert_eq!(
        orig_root.child(0).unwrap().byte_range(),
        clone_root.child(0).unwrap().byte_range(),
    );
}

#[test]
fn clone_is_independent() {
    let tree = leaf(5, 0, 10);
    let cloned = tree.clone();
    // Both should exist independently
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

#[test]
fn clone_deep_tree() {
    let mut current = leaf(20, 0, 1);
    for sym in (0..20).rev() {
        current = node(sym, 0, 1, vec![current]);
    }
    let cloned = current.clone();
    assert_eq!(dfs_collect(&current), dfs_collect(&cloned));
}

// ===== Section 11: Debug formatting =====

#[test]
fn debug_format_contains_tree_keyword() {
    let tree = leaf(1, 0, 5);
    let output = format!("{:?}", tree);
    assert!(output.contains("Tree"), "Debug output: {output}");
}

#[test]
fn debug_format_contains_range_info() {
    let tree = leaf(1, 3, 7);
    let output = format!("{:?}", tree);
    assert!(
        output.contains("3..7") || output.contains("range"),
        "Debug output: {output}"
    );
}

// ===== Section 12: Node API through tree =====

#[test]
fn node_is_named_returns_true() {
    let tree = leaf(1, 0, 1);
    assert!(tree.root_node().is_named());
}

#[test]
fn node_is_missing_returns_false() {
    let tree = leaf(1, 0, 1);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn node_is_error_returns_false() {
    let tree = leaf(1, 0, 1);
    assert!(!tree.root_node().is_error());
}

#[test]
fn node_named_child_count_equals_child_count() {
    let tree = node(0, 0, 3, vec![leaf(1, 0, 1), leaf(2, 1, 2), leaf(3, 2, 3)]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn node_named_child_same_as_child() {
    let tree = node(0, 0, 2, vec![leaf(1, 0, 1), leaf(2, 1, 2)]);
    let root = tree.root_node();
    assert_eq!(
        root.named_child(0).unwrap().kind_id(),
        root.child(0).unwrap().kind_id(),
    );
}

#[test]
fn node_parent_returns_none() {
    let tree = node(0, 0, 1, vec![leaf(1, 0, 1)]);
    assert!(tree.root_node().parent().is_none());
    assert!(tree.root_node().child(0).unwrap().parent().is_none());
}

#[test]
fn node_siblings_return_none() {
    let tree = node(0, 0, 2, vec![leaf(1, 0, 1), leaf(2, 1, 2)]);
    let first_child = tree.root_node().child(0).unwrap();
    assert!(first_child.next_sibling().is_none());
    assert!(first_child.prev_sibling().is_none());
    assert!(first_child.next_named_sibling().is_none());
    assert!(first_child.prev_named_sibling().is_none());
}

#[test]
fn node_child_by_field_name_returns_none() {
    let tree = node(0, 0, 1, vec![leaf(1, 0, 1)]);
    assert!(tree.root_node().child_by_field_name("left").is_none());
}

// ===== Section 13: utf8_text =====

#[test]
fn node_utf8_text_extracts_range() {
    let tree = leaf(1, 2, 5);
    let source = b"hello world";
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "llo");
}

#[test]
fn node_utf8_text_full_range() {
    let tree = leaf(1, 0, 5);
    let source = b"hello";
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "hello");
}

// ===== Section 14: Tree source_bytes =====

#[test]
fn tree_source_bytes_initially_none() {
    let tree = leaf(1, 0, 1);
    assert!(tree.source_bytes().is_none());
}

#[test]
fn stub_tree_source_bytes_none() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

// ===== Section 15: root_kind =====

#[test]
fn root_kind_returns_symbol_id() {
    let tree = leaf(99, 0, 1);
    assert_eq!(tree.root_kind(), 99);
}

#[test]
fn root_kind_of_stub_is_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

// ===== Section 16: Edge cases — overlapping and unusual ranges =====

#[test]
fn children_with_overlapping_byte_ranges() {
    // While unusual, the API doesn't enforce non-overlapping ranges.
    let tree = node(0, 0, 10, vec![leaf(1, 0, 7), leaf(2, 3, 10)]);
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().byte_range(), 0..7);
    assert_eq!(root.child(1).unwrap().byte_range(), 3..10);
}

#[test]
fn all_children_zero_byte_range() {
    let tree = node(0, 0, 0, vec![leaf(1, 0, 0), leaf(2, 0, 0)]);
    assert_eq!(tree.root_node().child_count(), 2);
    assert_eq!(tree.root_node().child(0).unwrap().byte_range(), 0..0);
}

#[test]
fn large_byte_range() {
    let big = usize::MAX / 2;
    let tree = leaf(1, 0, big);
    assert_eq!(tree.root_node().end_byte(), big);
}

// ===== Section 17: Multiple cursors / concurrent traversal =====

#[test]
fn two_cursors_on_same_tree() {
    let tree = node(0, 0, 3, vec![leaf(1, 0, 1), leaf(2, 1, 2), leaf(3, 2, 3)]);
    let mut c1 = TreeCursor::new(&tree);
    let mut c2 = TreeCursor::new(&tree);

    c1.goto_first_child(); // c1 at child 1
    c2.goto_first_child();
    c2.goto_next_sibling(); // c2 at child 2

    assert_eq!(c1.node().kind_id(), 1);
    assert_eq!(c2.node().kind_id(), 2);
}

#[test]
fn cursor_on_cloned_tree() {
    let tree = node(0, 0, 2, vec![leaf(1, 0, 1), leaf(2, 1, 2)]);
    let cloned = tree.clone();
    let c1_result = dfs_collect(&tree);
    let c2_result = dfs_collect(&cloned);
    assert_eq!(c1_result, c2_result);
}

// ===== Section 18: Complex tree shapes =====

#[test]
fn left_leaning_tree() {
    //     0
    //    /
    //   1
    //  /
    // 2
    let tree = node(0, 0, 3, vec![node(1, 0, 2, vec![leaf(2, 0, 1)])]);
    assert_eq!(dfs_collect(&tree), vec![0, 1, 2]);
}

#[test]
fn right_leaning_tree_via_last_child() {
    // Root with multiple children, only last has descendants
    let tree = node(
        0,
        0,
        10,
        vec![
            leaf(1, 0, 2),
            leaf(2, 2, 4),
            node(3, 4, 10, vec![leaf(4, 4, 7), leaf(5, 7, 10)]),
        ],
    );
    assert_eq!(dfs_collect(&tree), vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn diamond_shape_tree() {
    //       0
    //      / \
    //     1   2
    //    / \ / \
    //   3  4 5  6
    // (each subtree is independent; there's no shared node)
    let tree = node(
        0,
        0,
        12,
        vec![
            node(1, 0, 6, vec![leaf(3, 0, 3), leaf(4, 3, 6)]),
            node(2, 6, 12, vec![leaf(5, 6, 9), leaf(6, 9, 12)]),
        ],
    );
    assert_eq!(dfs_collect(&tree), vec![0, 1, 3, 4, 2, 5, 6]);
}

#[test]
fn mixed_depth_children() {
    //       0
    //     / | \
    //    1  2  3
    //   /      |
    //  4       5
    let tree = node(
        0,
        0,
        15,
        vec![
            node(1, 0, 5, vec![leaf(4, 0, 5)]),
            leaf(2, 5, 10),
            node(3, 10, 15, vec![leaf(5, 10, 15)]),
        ],
    );
    assert_eq!(dfs_collect(&tree), vec![0, 1, 4, 2, 3, 5]);
}

// ===== Section 19: Cursor depth at various positions =====

#[test]
fn cursor_depth_at_each_level() {
    let tree = node(
        0,
        0,
        5,
        vec![node(1, 0, 3, vec![leaf(2, 0, 3)]), leaf(3, 3, 5)],
    );
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0); // root

    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1); // child 1

    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2); // grandchild 2

    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1); // back to child 1

    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1); // sibling child 3

    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0); // root again
}

// ===== Section 20: Node byte_range consistency =====

#[test]
fn byte_range_matches_start_end() {
    let tree = leaf(1, 7, 42);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), root.start_byte()..root.end_byte());
}

#[test]
fn child_byte_ranges_via_cursor() {
    let tree = node(0, 0, 9, vec![leaf(1, 0, 3), leaf(2, 3, 6), leaf(3, 6, 9)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();

    let mut ranges = vec![cursor.node().byte_range()];
    while cursor.goto_next_sibling() {
        ranges.push(cursor.node().byte_range());
    }
    assert_eq!(ranges, vec![0..3, 3..6, 6..9]);
}

// ===== Section 21: Stress / boundary tests =====

#[test]
fn empty_children_vec_is_leaf() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn single_child_repeated_navigation() {
    let tree = node(0, 0, 1, vec![leaf(1, 0, 1)]);
    let mut cursor = TreeCursor::new(&tree);

    for _ in 0..10 {
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.node().kind_id(), 1);
        assert!(cursor.goto_parent());
        assert_eq!(cursor.node().kind_id(), 0);
    }
}

#[test]
fn symbol_id_max_u16() {
    let tree = leaf(u16::MAX as u32, 0, 1);
    assert_eq!(tree.root_node().kind_id(), u16::MAX);
}

#[test]
fn node_kind_without_language_is_unknown() {
    let tree = leaf(1, 0, 1);
    // No language set, so kind() should return "unknown"
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn start_position_and_end_position_are_zero() {
    // Phase 1 stubs: positions return (0,0)
    let tree = leaf(1, 5, 10);
    let root = tree.root_node();
    assert_eq!(root.start_position().row, 0);
    assert_eq!(root.start_position().column, 0);
    assert_eq!(root.end_position().row, 0);
    assert_eq!(root.end_position().column, 0);
}
