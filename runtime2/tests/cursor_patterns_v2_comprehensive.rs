//! Comprehensive tests for TreeCursor traversal patterns.

use adze_runtime::tree::{Tree, TreeCursor};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Build a simple binary tree:
///   root(0) [0..10]
///     left(1) [0..5]
///       ll(3) [0..2]
///       lr(4) [2..5]
///     right(2) [5..10]
fn binary_tree() -> Tree {
    let ll = leaf(3, 0, 2);
    let lr = leaf(4, 2, 5);
    let left = Tree::new_for_testing(1, 0, 5, vec![ll, lr]);
    let right = leaf(2, 5, 10);
    Tree::new_for_testing(0, 0, 10, vec![left, right])
}

/// Build a wide tree with `n` children.
fn wide_tree(n: usize) -> Tree {
    let children: Vec<Tree> = (0..n)
        .map(|i| leaf(i as u32 + 1, i * 10, (i + 1) * 10))
        .collect();
    Tree::new_for_testing(0, 0, n * 10, children)
}

/// Build a deeply nested linear chain: depth levels, one child per node.
fn deep_chain(depth: usize) -> Tree {
    let mut current = leaf(depth as u32, 0, 1);
    for d in (0..depth).rev() {
        current = Tree::new_for_testing(d as u32, 0, (depth - d) as usize, vec![current]);
    }
    current
}

// ---------------------------------------------------------------------------
// 1. Basic construction & cursor creation
// ---------------------------------------------------------------------------

#[test]
fn test_cursor_new_positions_at_root() {
    let tree = binary_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_on_single_leaf() {
    let tree = leaf(42, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 42);
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
}

#[test]
fn test_cursor_on_stub_tree() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert!(!cursor.goto_first_child());
}

// ---------------------------------------------------------------------------
// 2. goto_first_child
// ---------------------------------------------------------------------------

#[test]
fn test_goto_first_child_success() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn test_goto_first_child_leaf_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn test_goto_first_child_depth_increment() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
}

// ---------------------------------------------------------------------------
// 3. goto_next_sibling
// ---------------------------------------------------------------------------

#[test]
fn test_goto_next_sibling_success() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // left(1)
    assert!(cursor.goto_next_sibling()); // right(2)
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn test_goto_next_sibling_at_last_child_returns_false() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // right(2) — last child
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn test_goto_next_sibling_at_root_returns_false() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn test_goto_next_sibling_stays_at_same_depth() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let d = cursor.depth();
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), d);
}

// ---------------------------------------------------------------------------
// 4. goto_parent
// ---------------------------------------------------------------------------

#[test]
fn test_goto_parent_from_child() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_goto_parent_from_root_returns_false() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn test_goto_parent_from_grandchild() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // left
    cursor.goto_first_child(); // ll
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
}

// ---------------------------------------------------------------------------
// 5. depth
// ---------------------------------------------------------------------------

#[test]
fn test_depth_at_root_is_zero() {
    let tree = binary_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_depth_increases_with_descent() {
    let tree = deep_chain(5);
    let mut cursor = TreeCursor::new(&tree);
    for expected in 0..=5 {
        assert_eq!(cursor.depth(), expected);
        if expected < 5 {
            assert!(cursor.goto_first_child());
        }
    }
}

#[test]
fn test_depth_decreases_with_ascent() {
    let tree = deep_chain(3);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 3);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);
}

// ---------------------------------------------------------------------------
// 6. reset
// ---------------------------------------------------------------------------

#[test]
fn test_reset_returns_to_root() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn test_reset_to_different_tree() {
    let tree1 = binary_tree();
    let tree2 = leaf(99, 0, 1);
    let mut cursor = TreeCursor::new(&tree1);
    cursor.goto_first_child();
    cursor.reset(&tree2);
    assert_eq!(cursor.node().kind_id(), 99);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_reset_is_idempotent() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.reset(&tree);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

// ---------------------------------------------------------------------------
// 7. node() method
// ---------------------------------------------------------------------------

#[test]
fn test_node_kind_id() {
    let tree = binary_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn test_node_byte_range() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 10);
    cursor.goto_first_child();
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 5);
}

#[test]
fn test_node_child_count_via_cursor() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 2);
    cursor.goto_first_child(); // left, which has 2 children
    assert_eq!(cursor.node().child_count(), 2);
    cursor.goto_first_child(); // ll, a leaf
    assert_eq!(cursor.node().child_count(), 0);
}

#[test]
fn test_node_is_named() {
    let tree = binary_tree();
    let cursor = TreeCursor::new(&tree);
    assert!(cursor.node().is_named());
}

#[test]
fn test_node_is_not_error() {
    let tree = binary_tree();
    let cursor = TreeCursor::new(&tree);
    assert!(!cursor.node().is_error());
    assert!(!cursor.node().is_missing());
}

// ---------------------------------------------------------------------------
// 8. field_name / child_by_field_name on node
// ---------------------------------------------------------------------------

#[test]
fn test_child_by_field_name_returns_none() {
    let tree = binary_tree();
    let cursor = TreeCursor::new(&tree);
    // field-name access is currently unimplemented
    assert!(cursor.node().child_by_field_name("left").is_none());
}

#[test]
fn test_node_parent_returns_none() {
    // parent links are not stored in this implementation
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.node().parent().is_none());
}

// ---------------------------------------------------------------------------
// 9. DFS traversal pattern
// ---------------------------------------------------------------------------

fn collect_dfs(tree: &Tree) -> Vec<u16> {
    let mut result = Vec::new();
    let mut cursor = TreeCursor::new(tree);
    dfs_walk(&mut cursor, &mut result);
    result
}

fn dfs_walk(cursor: &mut TreeCursor<'_>, out: &mut Vec<u16>) {
    out.push(cursor.node().kind_id());
    if cursor.goto_first_child() {
        loop {
            dfs_walk(cursor, out);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

#[test]
fn test_dfs_binary_tree() {
    let tree = binary_tree();
    let order = collect_dfs(&tree);
    // root(0), left(1), ll(3), lr(4), right(2)
    assert_eq!(order, vec![0, 1, 3, 4, 2]);
}

#[test]
fn test_dfs_single_node() {
    let tree = leaf(7, 0, 1);
    let order = collect_dfs(&tree);
    assert_eq!(order, vec![7]);
}

#[test]
fn test_dfs_deep_chain() {
    let tree = deep_chain(4);
    let order = collect_dfs(&tree);
    assert_eq!(order, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_dfs_wide_tree() {
    let tree = wide_tree(5);
    let order = collect_dfs(&tree);
    // root(0), then children 1..5
    assert_eq!(order, vec![0, 1, 2, 3, 4, 5]);
}

// ---------------------------------------------------------------------------
// 10. BFS (manual breadth-first) traversal pattern
// ---------------------------------------------------------------------------

fn collect_bfs(tree: &Tree) -> Vec<u16> {
    let mut result = Vec::new();
    let mut queue: Vec<Vec<u16>> = Vec::new(); // collect per-level
    // We'll do BFS by repeatedly collecting children at each depth level.
    // Use the cursor for a level-order traversal.
    let mut cursor = TreeCursor::new(tree);
    result.push(cursor.node().kind_id());

    // Level-by-level: descend to each child, collect siblings, then go to their children.
    let mut has_children = cursor.goto_first_child();
    while has_children {
        let mut level = Vec::new();
        loop {
            level.push(cursor.node().kind_id());
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        result.extend_from_slice(&level);
        queue.push(level);

        // Reset to first child of this level and go to its children
        // For a proper BFS with cursor, we need to restart at each node.
        // Since cursor doesn't support random positioning, we use a simplified approach.
        cursor.goto_parent();
        cursor.goto_first_child();
        has_children = cursor.goto_first_child();
    }
    result
}

#[test]
fn test_bfs_binary_tree_first_two_levels() {
    // BFS on the binary tree captures at least root + first children level
    let tree = binary_tree();
    let order = collect_bfs(&tree);
    // Simplified BFS: root, then children of root, then children of first child
    assert!(order.contains(&0)); // root
    assert!(order.contains(&1)); // left
    assert!(order.contains(&2)); // right
}

// ---------------------------------------------------------------------------
// 11. Zigzag traversal pattern
// ---------------------------------------------------------------------------

#[test]
fn test_zigzag_down_up_down() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    // Down
    assert!(cursor.goto_first_child()); // left(1)
    assert_eq!(cursor.node().kind_id(), 1);
    // Up
    assert!(cursor.goto_parent()); // root(0)
    assert_eq!(cursor.node().kind_id(), 0);
    // Down again
    assert!(cursor.goto_first_child()); // left(1) — back to first child
    assert_eq!(cursor.node().kind_id(), 1);
    // Sibling
    assert!(cursor.goto_next_sibling()); // right(2)
    assert_eq!(cursor.node().kind_id(), 2);
    // Down to leaf
    assert!(!cursor.goto_first_child());
    // Up to root
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn test_zigzag_deep() {
    let tree = deep_chain(4);
    let mut cursor = TreeCursor::new(&tree);
    // Go all the way down
    for _ in 0..4 {
        cursor.goto_first_child();
    }
    assert_eq!(cursor.depth(), 4);
    // Zigzag: up 2, then try going down
    cursor.goto_parent();
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 2);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 3);
}

// ---------------------------------------------------------------------------
// 12. Deep trees (10+ levels)
// ---------------------------------------------------------------------------

#[test]
fn test_deep_chain_12_levels() {
    let tree = deep_chain(12);
    let mut cursor = TreeCursor::new(&tree);
    for d in 0..=12 {
        assert_eq!(cursor.depth(), d);
        if d < 12 {
            assert!(cursor.goto_first_child());
        }
    }
    assert_eq!(cursor.node().kind_id(), 12);
    // Walk all the way back up
    for d in (0..12).rev() {
        assert!(cursor.goto_parent());
        assert_eq!(cursor.depth(), d);
    }
}

#[test]
fn test_deep_chain_20_levels_dfs() {
    let tree = deep_chain(20);
    let order = collect_dfs(&tree);
    let expected: Vec<u16> = (0..=20).map(|i| i as u16).collect();
    assert_eq!(order, expected);
}

#[test]
fn test_deep_chain_node_properties_at_bottom() {
    let tree = deep_chain(15);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..15 {
        cursor.goto_first_child();
    }
    assert_eq!(cursor.node().kind_id(), 15);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 1);
    assert_eq!(cursor.node().child_count(), 0);
}

// ---------------------------------------------------------------------------
// 13. Wide trees (100+ children)
// ---------------------------------------------------------------------------

#[test]
fn test_wide_tree_100_children() {
    let tree = wide_tree(100);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 100);

    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 100);
}

#[test]
fn test_wide_tree_200_all_reachable() {
    let tree = wide_tree(200);
    let order = collect_dfs(&tree);
    assert_eq!(order.len(), 201); // root + 200 children
}

#[test]
fn test_wide_tree_last_child_properties() {
    let n = 150;
    let tree = wide_tree(n);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    for _ in 1..n {
        assert!(cursor.goto_next_sibling());
    }
    assert!(!cursor.goto_next_sibling());
    // Last child: symbol = n, range = [(n-1)*10, n*10)
    assert_eq!(cursor.node().kind_id(), n as u16);
    assert_eq!(cursor.node().start_byte(), (n - 1) * 10);
    assert_eq!(cursor.node().end_byte(), n * 10);
}

#[test]
fn test_wide_tree_depth_stays_one() {
    let tree = wide_tree(50);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    for _ in 1..50 {
        cursor.goto_next_sibling();
        assert_eq!(cursor.depth(), 1);
    }
}

// ---------------------------------------------------------------------------
// 14. Cursor after tree clone + modification
// ---------------------------------------------------------------------------

#[test]
fn test_cursor_on_cloned_tree() {
    let tree = binary_tree();
    let cloned = tree.clone();
    let mut cursor = TreeCursor::new(&cloned);
    assert_eq!(cursor.node().kind_id(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn test_cursor_independent_on_original_and_clone() {
    let tree = binary_tree();
    let cloned = tree.clone();
    let cursor_orig = TreeCursor::new(&tree);
    let cursor_clone = TreeCursor::new(&cloned);
    assert_eq!(cursor_orig.node().kind_id(), cursor_clone.node().kind_id());
    assert_eq!(
        cursor_orig.node().start_byte(),
        cursor_clone.node().start_byte()
    );
}

// ---------------------------------------------------------------------------
// 15. Cursor reuse patterns
// ---------------------------------------------------------------------------

#[test]
fn test_cursor_reuse_after_full_traversal() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    // Full DFS traversal
    let order1 = {
        let mut v = Vec::new();
        dfs_walk(&mut cursor, &mut v);
        v
    };
    // Reset and traverse again
    cursor.reset(&tree);
    let order2 = {
        let mut v = Vec::new();
        dfs_walk(&mut cursor, &mut v);
        v
    };
    assert_eq!(order1, order2);
}

#[test]
fn test_cursor_reuse_across_different_trees() {
    let tree1 = binary_tree();
    let tree2 = wide_tree(3);
    let mut cursor = TreeCursor::new(&tree1);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.reset(&tree2);
    assert_eq!(cursor.node().kind_id(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn test_cursor_multiple_resets() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..10 {
        cursor.goto_first_child();
        cursor.reset(&tree);
        assert_eq!(cursor.depth(), 0);
        assert_eq!(cursor.node().kind_id(), 0);
    }
}

// ---------------------------------------------------------------------------
// 16. Complex traversal: collect all leaves
// ---------------------------------------------------------------------------

fn collect_leaves(tree: &Tree) -> Vec<u16> {
    let mut result = Vec::new();
    let mut cursor = TreeCursor::new(tree);
    collect_leaves_rec(&mut cursor, &mut result);
    result
}

fn collect_leaves_rec(cursor: &mut TreeCursor<'_>, out: &mut Vec<u16>) {
    if !cursor.goto_first_child() {
        // This is a leaf
        out.push(cursor.node().kind_id());
        return;
    }
    loop {
        collect_leaves_rec(cursor, out);
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    cursor.goto_parent();
}

#[test]
fn test_collect_leaves_binary_tree() {
    let tree = binary_tree();
    let leaves = collect_leaves(&tree);
    assert_eq!(leaves, vec![3, 4, 2]); // ll, lr, right
}

#[test]
fn test_collect_leaves_wide_tree() {
    let tree = wide_tree(5);
    let leaves = collect_leaves(&tree);
    let expected: Vec<u16> = (1..=5).collect();
    assert_eq!(leaves, expected);
}

#[test]
fn test_collect_leaves_deep_chain() {
    let tree = deep_chain(10);
    let leaves = collect_leaves(&tree);
    assert_eq!(leaves, vec![10]); // only the deepest node
}

// ---------------------------------------------------------------------------
// 17. Complex traversal: count nodes at each depth
// ---------------------------------------------------------------------------

fn nodes_per_depth(tree: &Tree) -> Vec<usize> {
    let mut counts = Vec::new();
    let mut cursor = TreeCursor::new(tree);
    count_at_depth(&mut cursor, &mut counts);
    counts
}

fn count_at_depth(cursor: &mut TreeCursor<'_>, counts: &mut Vec<usize>) {
    let d = cursor.depth();
    while counts.len() <= d {
        counts.push(0);
    }
    counts[d] += 1;
    if cursor.goto_first_child() {
        loop {
            count_at_depth(cursor, counts);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

#[test]
fn test_nodes_per_depth_binary() {
    let tree = binary_tree();
    let counts = nodes_per_depth(&tree);
    assert_eq!(counts, vec![1, 2, 2]); // root, 2 children, 2 grandchildren
}

#[test]
fn test_nodes_per_depth_wide() {
    let tree = wide_tree(10);
    let counts = nodes_per_depth(&tree);
    assert_eq!(counts, vec![1, 10]);
}

// ---------------------------------------------------------------------------
// 18. Mixed nested tree
// ---------------------------------------------------------------------------

fn mixed_tree() -> Tree {
    // root(0) [0..30]
    //   a(1) [0..10]
    //     a1(10) [0..5]
    //     a2(11) [5..10]
    //   b(2) [10..20]
    //   c(3) [20..30]
    //     c1(30) [20..25]
    //       c1a(31) [20..22]
    //       c1b(32) [22..25]
    //     c2(33) [25..30]
    let a1 = leaf(10, 0, 5);
    let a2 = leaf(11, 5, 10);
    let a = Tree::new_for_testing(1, 0, 10, vec![a1, a2]);
    let b = leaf(2, 10, 20);
    let c1a = leaf(31, 20, 22);
    let c1b = leaf(32, 22, 25);
    let c1 = Tree::new_for_testing(30, 20, 25, vec![c1a, c1b]);
    let c2 = leaf(33, 25, 30);
    let c = Tree::new_for_testing(3, 20, 30, vec![c1, c2]);
    Tree::new_for_testing(0, 0, 30, vec![a, b, c])
}

#[test]
fn test_mixed_tree_dfs_order() {
    let tree = mixed_tree();
    let order = collect_dfs(&tree);
    assert_eq!(order, vec![0, 1, 10, 11, 2, 3, 30, 31, 32, 33]);
}

#[test]
fn test_mixed_tree_leaves() {
    let tree = mixed_tree();
    let leaves = collect_leaves(&tree);
    assert_eq!(leaves, vec![10, 11, 2, 31, 32, 33]);
}

#[test]
fn test_mixed_tree_depth_counts() {
    let tree = mixed_tree();
    let counts = nodes_per_depth(&tree);
    // depth 0: root(1), depth 1: a,b,c(3), depth 2: a1,a2,c1,c2(4), depth 3: c1a,c1b(2)
    assert_eq!(counts, vec![1, 3, 4, 2]);
}

#[test]
fn test_navigate_to_specific_node_in_mixed_tree() {
    let tree = mixed_tree();
    let mut cursor = TreeCursor::new(&tree);
    // Navigate to c1b(32): root -> c -> c1 -> c1b
    cursor.goto_first_child(); // a
    cursor.goto_next_sibling(); // b
    cursor.goto_next_sibling(); // c
    assert_eq!(cursor.node().kind_id(), 3);
    cursor.goto_first_child(); // c1
    assert_eq!(cursor.node().kind_id(), 30);
    cursor.goto_first_child(); // c1a
    cursor.goto_next_sibling(); // c1b
    assert_eq!(cursor.node().kind_id(), 32);
    assert_eq!(cursor.node().start_byte(), 22);
    assert_eq!(cursor.node().end_byte(), 25);
}

// ---------------------------------------------------------------------------
// 19. Sibling iteration pattern
// ---------------------------------------------------------------------------

fn collect_siblings_from_first(tree: &Tree) -> Vec<u16> {
    let mut result = Vec::new();
    let mut cursor = TreeCursor::new(tree);
    if cursor.goto_first_child() {
        result.push(cursor.node().kind_id());
        while cursor.goto_next_sibling() {
            result.push(cursor.node().kind_id());
        }
    }
    result
}

#[test]
fn test_sibling_iteration_binary() {
    let tree = binary_tree();
    assert_eq!(collect_siblings_from_first(&tree), vec![1, 2]);
}

#[test]
fn test_sibling_iteration_wide_50() {
    let tree = wide_tree(50);
    let siblings = collect_siblings_from_first(&tree);
    assert_eq!(siblings.len(), 50);
    for (i, &s) in siblings.iter().enumerate() {
        assert_eq!(s, (i + 1) as u16);
    }
}

// ---------------------------------------------------------------------------
// 20. Node byte_range via cursor
// ---------------------------------------------------------------------------

#[test]
fn test_byte_range_via_cursor() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().byte_range(), 0..10);
    cursor.goto_first_child();
    assert_eq!(cursor.node().byte_range(), 0..5);
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().byte_range(), 5..10);
}

// ---------------------------------------------------------------------------
// 21. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_goto_parent_then_first_child_revisits() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // left
    cursor.goto_next_sibling(); // right
    cursor.goto_parent(); // root
    cursor.goto_first_child(); // left again (always first child)
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn test_repeated_goto_first_child_no_crash() {
    let tree = deep_chain(3);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert!(!cursor.goto_first_child()); // at leaf
}

#[test]
fn test_repeated_goto_parent_no_crash() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent()); // already at root
    assert!(!cursor.goto_parent()); // still at root
    assert!(!cursor.goto_parent());
}

#[test]
fn test_goto_next_sibling_no_child_no_crash() {
    let tree = leaf(0, 0, 1);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_next_sibling());
}

// ---------------------------------------------------------------------------
// 22. Cursor preserves position after failed navigation
// ---------------------------------------------------------------------------

#[test]
fn test_failed_first_child_preserves_position() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child(); // at ll(3)
    let id_before = cursor.node().kind_id();
    assert!(!cursor.goto_first_child()); // leaf, fails
    assert_eq!(cursor.node().kind_id(), id_before);
}

#[test]
fn test_failed_sibling_preserves_position() {
    let tree = binary_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // right(2)
    let id_before = cursor.node().kind_id();
    assert!(!cursor.goto_next_sibling()); // no more siblings
    assert_eq!(cursor.node().kind_id(), id_before);
}

#[test]
fn test_failed_parent_preserves_position() {
    let tree = binary_tree();
    let cursor = TreeCursor::new(&tree);
    let id_before = cursor.node().kind_id();
    let mut cursor = cursor;
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), id_before);
}

// ---------------------------------------------------------------------------
// 23. Tree with single child at each level (unary chain)
// ---------------------------------------------------------------------------

#[test]
fn test_unary_chain_no_siblings() {
    let tree = deep_chain(5);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..5 {
        assert!(cursor.goto_first_child());
        assert!(!cursor.goto_next_sibling()); // only one child per level
    }
}

// ---------------------------------------------------------------------------
// 24. Complex: collect (depth, kind_id) pairs
// ---------------------------------------------------------------------------

fn collect_depth_kind(tree: &Tree) -> Vec<(usize, u16)> {
    let mut result = Vec::new();
    let mut cursor = TreeCursor::new(tree);
    collect_dk_rec(&mut cursor, &mut result);
    result
}

fn collect_dk_rec(cursor: &mut TreeCursor<'_>, out: &mut Vec<(usize, u16)>) {
    out.push((cursor.depth(), cursor.node().kind_id()));
    if cursor.goto_first_child() {
        loop {
            collect_dk_rec(cursor, out);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

#[test]
fn test_depth_kind_binary_tree() {
    let tree = binary_tree();
    let pairs = collect_depth_kind(&tree);
    assert_eq!(pairs, vec![(0, 0), (1, 1), (2, 3), (2, 4), (1, 2)]);
}

#[test]
fn test_depth_kind_deep_chain() {
    let tree = deep_chain(3);
    let pairs = collect_depth_kind(&tree);
    assert_eq!(pairs, vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
}
