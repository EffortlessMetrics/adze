//! Comprehensive tests for deep TreeCursor traversal patterns.
//!
//! Coverage areas:
//! 1. Cursor depth tracking through deep trees
//! 2. Full DFS traversal pattern
//! 3. Cursor on balanced binary tree
//! 4. Cursor on left-skewed tree
//! 5. Cursor parent after deep descent
//! 6. Cursor reset from deep position
//! 7. Wide tree cursor patterns
//! 8. Mixed deep/wide traversal
//! 9. Cursor on single node
//! 10. Multiple cursor operations in sequence

use adze_runtime::tree::{Tree, TreeCursor};

// ============================================================================
// Helpers
// ============================================================================

/// Leaf node helper.
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Full iterative DFS traversal returning (kind_id, depth) pairs.
fn dfs_with_depth(tree: &Tree) -> Vec<(u16, usize)> {
    let mut cursor = TreeCursor::new(tree);
    let mut result = Vec::new();
    let mut reached_root = false;

    loop {
        result.push((cursor.node().kind_id(), cursor.depth()));

        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                reached_root = true;
                break;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
        if reached_root {
            break;
        }
    }

    result
}

/// Full DFS returning only kind_ids.
fn dfs_kind_ids(tree: &Tree) -> Vec<u16> {
    dfs_with_depth(tree).into_iter().map(|(id, _)| id).collect()
}

/// Build a linear chain: root -> child -> grandchild -> ... with given depth.
fn build_linear_chain(depth: usize) -> Tree {
    assert!(depth >= 1);
    let mut current = leaf(depth as u32, 0, 1);
    for i in (1..depth).rev() {
        current = Tree::new_for_testing(i as u32, 0, 1, vec![current]);
    }
    current
}

/// Build a balanced binary tree of given depth (depth 0 = single leaf).
fn build_balanced_binary(depth: usize, next_sym: &mut u32) -> Tree {
    let sym = *next_sym;
    *next_sym += 1;
    if depth == 0 {
        return leaf(sym, 0, 1);
    }
    let left = build_balanced_binary(depth - 1, next_sym);
    let right = build_balanced_binary(depth - 1, next_sym);
    Tree::new_for_testing(sym, 0, 1, vec![left, right])
}

/// Build a left-skewed tree: each node has one child on the left plus a right leaf.
fn build_left_skewed(depth: usize, next_sym: &mut u32) -> Tree {
    let sym = *next_sym;
    *next_sym += 1;
    if depth == 0 {
        return leaf(sym, 0, 1);
    }
    let left = build_left_skewed(depth - 1, next_sym);
    let right_sym = *next_sym;
    *next_sym += 1;
    let right = leaf(right_sym, 0, 1);
    Tree::new_for_testing(sym, 0, 1, vec![left, right])
}

/// Count total nodes via DFS.
fn count_nodes(tree: &Tree) -> usize {
    dfs_kind_ids(tree).len()
}

// ============================================================================
// 1. Cursor depth tracking through deep trees
// ============================================================================

#[test]
fn depth_at_root_is_zero() {
    let tree = leaf(1, 0, 10);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn depth_increments_on_first_child() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn depth_tracks_through_chain_of_3() {
    let tree = build_linear_chain(3);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
    assert!(!cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn depth_tracks_through_chain_of_10() {
    let tree = build_linear_chain(10);
    let mut cursor = TreeCursor::new(&tree);
    for expected in 0..10 {
        assert_eq!(cursor.depth(), expected);
        if expected < 9 {
            assert!(cursor.goto_first_child());
        }
    }
}

#[test]
fn depth_tracks_through_chain_of_20() {
    let tree = build_linear_chain(20);
    let mut cursor = TreeCursor::new(&tree);
    for expected in 0..20 {
        assert_eq!(cursor.depth(), expected);
        if expected < 19 {
            assert!(cursor.goto_first_child());
        }
    }
}

#[test]
fn depth_decrements_on_parent() {
    let tree = build_linear_chain(5);
    let mut cursor = TreeCursor::new(&tree);
    // Go deep
    for _ in 0..4 {
        cursor.goto_first_child();
    }
    assert_eq!(cursor.depth(), 4);
    // Come back
    for expected in (0..4).rev() {
        assert!(cursor.goto_parent());
        assert_eq!(cursor.depth(), expected);
    }
}

#[test]
fn depth_unchanged_on_sibling_move() {
    let tree = Tree::new_for_testing(
        1,
        0,
        30,
        vec![leaf(2, 0, 10), leaf(3, 10, 20), leaf(4, 20, 30)],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);
}

// ============================================================================
// 2. Full DFS traversal pattern
// ============================================================================

#[test]
fn dfs_single_node() {
    let tree = leaf(42, 0, 5);
    assert_eq!(dfs_kind_ids(&tree), vec![42]);
}

#[test]
fn dfs_root_with_one_child() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![leaf(2, 0, 10)]);
    assert_eq!(dfs_kind_ids(&tree), vec![1, 2]);
}

#[test]
fn dfs_root_with_three_children() {
    let tree = Tree::new_for_testing(
        1,
        0,
        30,
        vec![leaf(2, 0, 10), leaf(3, 10, 20), leaf(4, 20, 30)],
    );
    assert_eq!(dfs_kind_ids(&tree), vec![1, 2, 3, 4]);
}

#[test]
fn dfs_two_level_tree() {
    let child_a = Tree::new_for_testing(2, 0, 10, vec![leaf(4, 0, 5), leaf(5, 5, 10)]);
    let child_b = Tree::new_for_testing(3, 10, 20, vec![leaf(6, 10, 15)]);
    let tree = Tree::new_for_testing(1, 0, 20, vec![child_a, child_b]);
    assert_eq!(dfs_kind_ids(&tree), vec![1, 2, 4, 5, 3, 6]);
}

#[test]
fn dfs_depths_match_expected() {
    let child_a = Tree::new_for_testing(2, 0, 10, vec![leaf(4, 0, 5), leaf(5, 5, 10)]);
    let child_b = leaf(3, 10, 20);
    let tree = Tree::new_for_testing(1, 0, 20, vec![child_a, child_b]);
    let depths: Vec<usize> = dfs_with_depth(&tree).into_iter().map(|(_, d)| d).collect();
    assert_eq!(depths, vec![0, 1, 2, 2, 1]);
}

#[test]
fn dfs_linear_chain_visits_in_order() {
    let tree = build_linear_chain(6);
    let ids = dfs_kind_ids(&tree);
    assert_eq!(ids, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn dfs_node_count_matches_linear_chain() {
    let tree = build_linear_chain(15);
    assert_eq!(count_nodes(&tree), 15);
}

// ============================================================================
// 3. Cursor on balanced binary tree
// ============================================================================

#[test]
fn balanced_depth_0_single_node() {
    let mut sym = 1;
    let tree = build_balanced_binary(0, &mut sym);
    assert_eq!(count_nodes(&tree), 1);
}

#[test]
fn balanced_depth_1_three_nodes() {
    let mut sym = 1;
    let tree = build_balanced_binary(1, &mut sym);
    assert_eq!(count_nodes(&tree), 3);
}

#[test]
fn balanced_depth_2_seven_nodes() {
    let mut sym = 1;
    let tree = build_balanced_binary(2, &mut sym);
    assert_eq!(count_nodes(&tree), 7);
}

#[test]
fn balanced_depth_3_fifteen_nodes() {
    let mut sym = 1;
    let tree = build_balanced_binary(3, &mut sym);
    assert_eq!(count_nodes(&tree), 15);
}

#[test]
fn balanced_depth_4_thirtyone_nodes() {
    let mut sym = 1;
    let tree = build_balanced_binary(4, &mut sym);
    assert_eq!(count_nodes(&tree), 31);
}

#[test]
fn balanced_tree_max_depth_matches() {
    let mut sym = 1;
    let tree = build_balanced_binary(3, &mut sym);
    let max_depth = dfs_with_depth(&tree)
        .into_iter()
        .map(|(_, d)| d)
        .max()
        .unwrap();
    assert_eq!(max_depth, 3);
}

#[test]
fn balanced_tree_dfs_visits_left_before_right() {
    let mut sym = 1;
    let tree = build_balanced_binary(2, &mut sym);
    let mut cursor = TreeCursor::new(&tree);
    // root
    assert_eq!(cursor.node().kind_id(), 1);
    // left subtree root
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 2);
    // left-left leaf
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 3);
}

#[test]
fn balanced_tree_right_subtree_reachable() {
    let mut sym = 1;
    let tree = build_balanced_binary(1, &mut sym);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 2); // left child
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3); // right child
}

#[test]
fn balanced_tree_leaf_has_no_children() {
    let mut sym = 1;
    let tree = build_balanced_binary(2, &mut sym);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // left subtree
    cursor.goto_first_child(); // left-left leaf
    assert!(!cursor.goto_first_child());
}

// ============================================================================
// 4. Cursor on left-skewed tree
// ============================================================================

#[test]
fn left_skewed_depth_3_node_count() {
    let mut sym = 1;
    let tree = build_left_skewed(3, &mut sym);
    // depth 3: 3 internal + 3 right leaves + 1 bottom leaf = 7
    assert_eq!(count_nodes(&tree), 7);
}

#[test]
fn left_skewed_max_depth_matches() {
    let mut sym = 1;
    let tree = build_left_skewed(4, &mut sym);
    let max_depth = dfs_with_depth(&tree)
        .into_iter()
        .map(|(_, d)| d)
        .max()
        .unwrap();
    assert_eq!(max_depth, 4);
}

#[test]
fn left_skewed_descend_left_path() {
    let mut sym = 1;
    let tree = build_left_skewed(5, &mut sym);
    let mut cursor = TreeCursor::new(&tree);
    for d in 0..5 {
        assert_eq!(cursor.depth(), d);
        assert!(cursor.goto_first_child());
    }
    assert_eq!(cursor.depth(), 5);
    // Bottom leaf has no children
    assert!(!cursor.goto_first_child());
}

#[test]
fn left_skewed_each_internal_has_sibling() {
    let mut sym = 1;
    let tree = build_left_skewed(4, &mut sym);
    let mut cursor = TreeCursor::new(&tree);
    // At each level (except bottom), first child should have a sibling
    for _ in 0..4 {
        cursor.goto_first_child();
        assert!(cursor.goto_next_sibling()); // right leaf sibling exists
        cursor.goto_parent(); // back to parent
        cursor.goto_first_child(); // down left again
    }
}

// ============================================================================
// 5. Cursor parent after deep descent
// ============================================================================

#[test]
fn parent_from_depth_5_returns_to_root() {
    let tree = build_linear_chain(6);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..5 {
        cursor.goto_first_child();
    }
    assert_eq!(cursor.depth(), 5);
    for _ in 0..5 {
        assert!(cursor.goto_parent());
    }
    assert_eq!(cursor.depth(), 0);
    assert!(!cursor.goto_parent());
}

#[test]
fn parent_preserves_correct_node_at_each_level() {
    let tree = build_linear_chain(4);
    let mut cursor = TreeCursor::new(&tree);
    // Descend collecting IDs
    let mut ids_down = Vec::new();
    ids_down.push(cursor.node().kind_id());
    while cursor.goto_first_child() {
        ids_down.push(cursor.node().kind_id());
    }
    // Ascend collecting IDs
    let mut ids_up = vec![cursor.node().kind_id()];
    while cursor.goto_parent() {
        ids_up.push(cursor.node().kind_id());
    }
    ids_up.reverse();
    assert_eq!(ids_down, ids_up);
}

#[test]
fn parent_after_sibling_move_returns_correct_parent() {
    let tree = Tree::new_for_testing(
        10,
        0,
        30,
        vec![leaf(11, 0, 10), leaf(12, 10, 20), leaf(13, 20, 30)],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling(); // at child 13
    assert_eq!(cursor.node().kind_id(), 13);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 10);
}

#[test]
fn parent_at_root_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn parent_called_repeatedly_stays_at_root() {
    let tree = build_linear_chain(3);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.goto_parent();
    cursor.goto_parent();
    assert!(!cursor.goto_parent());
    assert!(!cursor.goto_parent()); // extra call
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// 6. Cursor reset from deep position
// ============================================================================

#[test]
fn reset_from_deep_position() {
    let tree = build_linear_chain(8);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..7 {
        cursor.goto_first_child();
    }
    assert_eq!(cursor.depth(), 7);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn reset_to_different_tree() {
    let tree1 = Tree::new_for_testing(100, 0, 10, vec![leaf(101, 0, 5)]);
    let tree2 = Tree::new_for_testing(200, 0, 20, vec![leaf(201, 0, 10)]);
    let mut cursor = TreeCursor::new(&tree1);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 101);
    cursor.reset(&tree2);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 200);
}

#[test]
fn reset_allows_full_retraversal() {
    let tree = Tree::new_for_testing(
        1,
        0,
        30,
        vec![leaf(2, 0, 10), leaf(3, 10, 20), leaf(4, 20, 30)],
    );
    let first_ids = dfs_kind_ids(&tree);
    // Traverse partially
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    // Reset and re-traverse
    cursor.reset(&tree);
    let mut second_ids = Vec::new();
    let mut reached_root = false;
    loop {
        second_ids.push(cursor.node().kind_id());
        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                reached_root = true;
                break;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
        if reached_root {
            break;
        }
    }
    assert_eq!(first_ids, second_ids);
}

#[test]
fn reset_from_root_is_noop_equivalent() {
    let tree = leaf(5, 0, 10);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 5);
}

// ============================================================================
// 7. Wide tree cursor patterns
// ============================================================================

#[test]
fn wide_tree_10_children_all_reachable() {
    let children: Vec<Tree> = (0..10u32)
        .map(|i| leaf(i + 10, (i * 5) as usize, ((i + 1) * 5) as usize))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 50, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut visited = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        visited.push(cursor.node().kind_id());
    }
    let expected: Vec<u16> = (10..20).collect();
    assert_eq!(visited, expected);
}

#[test]
fn wide_tree_50_children_all_reachable() {
    let children: Vec<Tree> = (0..50).map(|i| leaf(i as u32 + 100, 0, 1)).collect();
    let tree = Tree::new_for_testing(1, 0, 50, children);
    let ids = dfs_kind_ids(&tree);
    assert_eq!(ids.len(), 51); // root + 50 children
}

#[test]
fn wide_tree_last_child_has_no_next_sibling() {
    let children: Vec<Tree> = (0..5).map(|i| leaf(i + 10, 0, 1)).collect();
    let tree = Tree::new_for_testing(1, 0, 5, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    for _ in 0..4 {
        assert!(cursor.goto_next_sibling());
    }
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn wide_tree_depth_always_1_for_children() {
    let children: Vec<Tree> = (0..8).map(|i| leaf(i + 10, 0, 1)).collect();
    let tree = Tree::new_for_testing(1, 0, 8, children);
    let depths: Vec<usize> = dfs_with_depth(&tree).into_iter().map(|(_, d)| d).collect();
    assert_eq!(depths[0], 0); // root
    for d in &depths[1..] {
        assert_eq!(*d, 1);
    }
}

#[test]
fn wide_tree_parent_from_any_child_returns_root() {
    let children: Vec<Tree> = (0..6).map(|i| leaf(i + 10, 0, 1)).collect();
    let tree = Tree::new_for_testing(1, 0, 6, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    // Move to 4th child
    for _ in 0..3 {
        cursor.goto_next_sibling();
    }
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// 8. Mixed deep/wide traversal
// ============================================================================

#[test]
fn mixed_tree_dfs_order() {
    //       1
    //      / \
    //     2   3
    //    /   / \
    //   4   5   6
    let n4 = leaf(4, 0, 2);
    let n2 = Tree::new_for_testing(2, 0, 4, vec![n4]);
    let n5 = leaf(5, 4, 6);
    let n6 = leaf(6, 6, 8);
    let n3 = Tree::new_for_testing(3, 4, 8, vec![n5, n6]);
    let tree = Tree::new_for_testing(1, 0, 8, vec![n2, n3]);
    assert_eq!(dfs_kind_ids(&tree), vec![1, 2, 4, 3, 5, 6]);
}

#[test]
fn mixed_tree_depth_tracking() {
    let n4 = leaf(4, 0, 2);
    let n2 = Tree::new_for_testing(2, 0, 4, vec![n4]);
    let n5 = leaf(5, 4, 6);
    let n6 = leaf(6, 6, 8);
    let n3 = Tree::new_for_testing(3, 4, 8, vec![n5, n6]);
    let tree = Tree::new_for_testing(1, 0, 8, vec![n2, n3]);
    let depths: Vec<usize> = dfs_with_depth(&tree).into_iter().map(|(_, d)| d).collect();
    assert_eq!(depths, vec![0, 1, 2, 1, 2, 2]);
}

#[test]
fn mixed_navigate_down_left_then_up_and_right() {
    let n4 = leaf(4, 0, 2);
    let n2 = Tree::new_for_testing(2, 0, 4, vec![n4]);
    let n5 = leaf(5, 4, 6);
    let n3 = Tree::new_for_testing(3, 4, 8, vec![n5]);
    let tree = Tree::new_for_testing(1, 0, 8, vec![n2, n3]);

    let mut cursor = TreeCursor::new(&tree);
    // Go down left branch
    cursor.goto_first_child(); // -> 2
    cursor.goto_first_child(); // -> 4
    assert_eq!(cursor.node().kind_id(), 4);
    assert_eq!(cursor.depth(), 2);
    // Go up to root
    cursor.goto_parent(); // -> 2
    cursor.goto_parent(); // -> 1
    assert_eq!(cursor.depth(), 0);
    // Go down right branch
    cursor.goto_first_child(); // -> 2
    cursor.goto_next_sibling(); // -> 3
    cursor.goto_first_child(); // -> 5
    assert_eq!(cursor.node().kind_id(), 5);
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn mixed_three_levels_deep_with_wide_middle() {
    //       1
    //    / | | \
    //   2  3  4  5
    //  /      |
    // 6       7
    let n6 = leaf(6, 0, 2);
    let n2 = Tree::new_for_testing(2, 0, 4, vec![n6]);
    let n3 = leaf(3, 4, 6);
    let n7 = leaf(7, 6, 8);
    let n4 = Tree::new_for_testing(4, 6, 10, vec![n7]);
    let n5 = leaf(5, 10, 12);
    let tree = Tree::new_for_testing(1, 0, 12, vec![n2, n3, n4, n5]);
    assert_eq!(dfs_kind_ids(&tree), vec![1, 2, 6, 3, 4, 7, 5]);
}

#[test]
fn mixed_asymmetric_left_heavy() {
    //     1
    //    / \
    //   2   3
    //  / \
    // 4   5
    //     |
    //     6
    let n4 = leaf(4, 0, 2);
    let n6 = leaf(6, 2, 4);
    let n5 = Tree::new_for_testing(5, 2, 6, vec![n6]);
    let n2 = Tree::new_for_testing(2, 0, 6, vec![n4, n5]);
    let n3 = leaf(3, 6, 8);
    let tree = Tree::new_for_testing(1, 0, 8, vec![n2, n3]);
    assert_eq!(dfs_kind_ids(&tree), vec![1, 2, 4, 5, 6, 3]);
    let max_depth = dfs_with_depth(&tree)
        .into_iter()
        .map(|(_, d)| d)
        .max()
        .unwrap();
    assert_eq!(max_depth, 3);
}

// ============================================================================
// 9. Cursor on single node
// ============================================================================

#[test]
fn single_node_no_children() {
    let tree = leaf(99, 0, 42);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn single_node_no_sibling() {
    let tree = leaf(99, 0, 42);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn single_node_no_parent() {
    let tree = leaf(99, 0, 42);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn single_node_dfs_returns_one() {
    let tree = leaf(77, 5, 15);
    assert_eq!(dfs_kind_ids(&tree), vec![77]);
}

#[test]
fn single_node_depth_is_zero() {
    let tree = leaf(33, 0, 1);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn single_node_reset_returns_to_same() {
    let tree = leaf(44, 0, 1);
    let mut cursor = TreeCursor::new(&tree);
    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 44);
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// 10. Multiple cursor operations in sequence
// ============================================================================

#[test]
fn sequential_descend_ascend_three_times() {
    let tree = build_linear_chain(5);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..3 {
        // Descend to bottom
        while cursor.goto_first_child() {}
        assert_eq!(cursor.depth(), 4);
        // Ascend to root
        while cursor.goto_parent() {}
        assert_eq!(cursor.depth(), 0);
    }
}

#[test]
fn sequential_traverse_then_reset_then_traverse() {
    let tree = Tree::new_for_testing(1, 0, 20, vec![leaf(2, 0, 10), leaf(3, 10, 20)]);
    let first = dfs_kind_ids(&tree);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.reset(&tree);
    // Manual second traversal
    let mut second = Vec::new();
    let mut done = false;
    loop {
        second.push(cursor.node().kind_id());
        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                done = true;
                break;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
        if done {
            break;
        }
    }
    assert_eq!(first, second);
}

#[test]
fn interleave_child_sibling_parent() {
    let n3 = leaf(3, 0, 2);
    let n4 = leaf(4, 2, 4);
    let n2 = Tree::new_for_testing(2, 0, 4, vec![n3, n4]);
    let n5 = leaf(5, 4, 6);
    let tree = Tree::new_for_testing(1, 0, 6, vec![n2, n5]);

    let mut cursor = TreeCursor::new(&tree);
    // child -> child -> sibling -> parent -> sibling
    assert!(cursor.goto_first_child()); // 2
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_first_child()); // 3
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(cursor.goto_next_sibling()); // 4
    assert_eq!(cursor.node().kind_id(), 4);
    assert!(cursor.goto_parent()); // 2
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_next_sibling()); // 5
    assert_eq!(cursor.node().kind_id(), 5);
}

#[test]
fn repeated_failed_operations_dont_corrupt_state() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    // All these should fail but not corrupt cursor
    for _ in 0..10 {
        assert!(!cursor.goto_first_child());
        assert!(!cursor.goto_next_sibling());
        assert!(!cursor.goto_parent());
    }
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn zigzag_traversal_pattern() {
    //       1
    //      / \
    //     2   3
    //    /     \
    //   4       5
    let n4 = leaf(4, 0, 2);
    let n2 = Tree::new_for_testing(2, 0, 4, vec![n4]);
    let n5 = leaf(5, 4, 6);
    let n3 = Tree::new_for_testing(3, 4, 8, vec![n5]);
    let tree = Tree::new_for_testing(1, 0, 8, vec![n2, n3]);

    let mut cursor = TreeCursor::new(&tree);
    // Down left
    cursor.goto_first_child(); // 2
    cursor.goto_first_child(); // 4
    assert_eq!(cursor.node().kind_id(), 4);
    // Up
    cursor.goto_parent(); // 2
    cursor.goto_parent(); // 1
    // Down right
    cursor.goto_first_child(); // 2
    cursor.goto_next_sibling(); // 3
    cursor.goto_first_child(); // 5
    assert_eq!(cursor.node().kind_id(), 5);
}

// ============================================================================
// Additional edge case tests
// ============================================================================

#[test]
fn stub_tree_cursor_depth() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn stub_tree_cursor_no_children() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn deep_chain_50_descend_and_ascend() {
    let tree = build_linear_chain(50);
    let mut cursor = TreeCursor::new(&tree);
    for i in 0..49 {
        assert_eq!(cursor.depth(), i);
        assert!(cursor.goto_first_child());
    }
    assert_eq!(cursor.depth(), 49);
    assert!(!cursor.goto_first_child());
    for i in (0..49).rev() {
        assert!(cursor.goto_parent());
        assert_eq!(cursor.depth(), i);
    }
    assert!(!cursor.goto_parent());
}

#[test]
fn node_properties_during_traversal() {
    let tree = Tree::new_for_testing(
        10,
        0,
        100,
        vec![
            Tree::new_for_testing(20, 0, 50, vec![leaf(30, 0, 25), leaf(31, 25, 50)]),
            leaf(21, 50, 100),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 100);

    cursor.goto_first_child();
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 50);

    cursor.goto_first_child();
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 25);

    cursor.goto_next_sibling();
    assert_eq!(cursor.node().start_byte(), 25);
    assert_eq!(cursor.node().end_byte(), 50);
}

#[test]
fn child_count_during_traversal() {
    let tree = Tree::new_for_testing(
        1,
        0,
        30,
        vec![
            Tree::new_for_testing(2, 0, 10, vec![leaf(4, 0, 5), leaf(5, 5, 10)]),
            leaf(3, 10, 30),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 2);
    cursor.goto_first_child();
    assert_eq!(cursor.node().child_count(), 2);
    cursor.goto_first_child();
    assert_eq!(cursor.node().child_count(), 0);
}

#[test]
fn reset_mid_sibling_walk() {
    let children: Vec<Tree> = (0..5).map(|i| leaf(i + 10, 0, 1)).collect();
    let tree = Tree::new_for_testing(1, 0, 5, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling(); // at third child
    assert_eq!(cursor.node().kind_id(), 12);
    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn two_cursors_independent() {
    let tree = Tree::new_for_testing(1, 0, 20, vec![leaf(2, 0, 10), leaf(3, 10, 20)]);
    let mut c1 = TreeCursor::new(&tree);
    let mut c2 = TreeCursor::new(&tree);
    c1.goto_first_child();
    assert_eq!(c1.node().kind_id(), 2);
    assert_eq!(c2.node().kind_id(), 1); // c2 still at root
    c2.goto_first_child();
    c2.goto_next_sibling();
    assert_eq!(c2.node().kind_id(), 3);
    assert_eq!(c1.node().kind_id(), 2); // c1 unchanged
}

#[test]
fn balanced_tree_depth_5_full_traversal() {
    let mut sym = 1;
    let tree = build_balanced_binary(5, &mut sym);
    let nodes = dfs_with_depth(&tree);
    assert_eq!(nodes.len(), 63); // 2^6 - 1
    let max_d = nodes.iter().map(|(_, d)| *d).max().unwrap();
    assert_eq!(max_d, 5);
}

#[test]
fn sibling_after_failed_child_returns_false_at_leaf() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![leaf(2, 0, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // at leaf 2
    assert!(!cursor.goto_first_child()); // no children
    assert!(!cursor.goto_next_sibling()); // no siblings
}

#[test]
fn dfs_collects_all_leaf_depths() {
    //       1
    //      / \
    //     2   3
    //    /
    //   4
    let n4 = leaf(4, 0, 2);
    let n2 = Tree::new_for_testing(2, 0, 4, vec![n4]);
    let n3 = leaf(3, 4, 8);
    let tree = Tree::new_for_testing(1, 0, 8, vec![n2, n3]);

    let leaf_depths: Vec<usize> = dfs_with_depth(&tree)
        .into_iter()
        .filter(|(id, _)| *id == 4 || *id == 3)
        .map(|(_, d)| d)
        .collect();
    assert_eq!(leaf_depths, vec![2, 1]); // leaf 4 at depth 2, leaf 3 at depth 1
}

#[test]
fn cursor_node_kind_id_matches_symbol() {
    let tree = Tree::new_for_testing(255, 0, 10, vec![leaf(128, 0, 5), leaf(64, 5, 10)]);
    let ids = dfs_kind_ids(&tree);
    assert_eq!(ids, vec![255, 128, 64]);
}

#[test]
fn repeated_reset_is_stable() {
    let tree = build_linear_chain(5);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..20 {
        cursor.reset(&tree);
        assert_eq!(cursor.depth(), 0);
        assert_eq!(cursor.node().kind_id(), 1);
    }
}
