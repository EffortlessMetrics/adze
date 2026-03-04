#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::tree::{Tree, TreeCursor};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Build a flat tree with leaf children occupying consecutive byte sub-ranges.
fn flat_tree(symbol: u32, start: usize, end: usize, num_children: usize) -> Tree {
    if num_children == 0 || end <= start {
        return Tree::new_for_testing(symbol, start, end, vec![]);
    }
    let span = end - start;
    let children: Vec<Tree> = (0..num_children)
        .map(|i| {
            let cs = start + (span * i) / num_children;
            let ce = start + (span * (i + 1)) / num_children;
            Tree::new_for_testing((i as u32) + 1, cs, ce, vec![])
        })
        .collect();
    Tree::new_for_testing(symbol, start, end, children)
}

/// Build a deep linear tree (each node has exactly one child, except the leaf).
fn deep_tree(depth: usize, start: usize, end: usize) -> Tree {
    if depth == 0 {
        return Tree::new_for_testing(depth as u32, start, end, vec![]);
    }
    let child = deep_tree(depth - 1, start, end);
    Tree::new_for_testing(depth as u32, start, end, vec![child])
}

/// Strategy for a flat tree with random children count.
fn arb_flat_tree() -> impl Strategy<Value = Tree> {
    (0u32..500, 0usize..5_000, 1usize..5_000, 0usize..8).prop_map(
        |(sym, start, span, nchildren)| {
            let end = start + span;
            flat_tree(sym, start, end, nchildren)
        },
    )
}

/// Strategy for a flat tree guaranteed to have at least one child.
fn arb_flat_tree_with_children() -> impl Strategy<Value = Tree> {
    (0u32..500, 0usize..5_000, 1usize..5_000, 1usize..8).prop_map(
        |(sym, start, span, nchildren)| {
            let end = start + span;
            flat_tree(sym, start, end, nchildren)
        },
    )
}

/// Strategy for a deep linear tree.
fn arb_deep_tree() -> impl Strategy<Value = (Tree, usize)> {
    (1usize..15, 0usize..5_000, 1usize..5_000).prop_map(|(depth, start, span)| {
        let end = start + span;
        (deep_tree(depth, start, end), depth)
    })
}

// ===========================================================================
// 1 – Cursor creation: cursor starts at root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn cursor_starts_at_root(tree in arb_flat_tree()) {
        let cursor = TreeCursor::new(&tree);
        let node = cursor.node();
        let root = tree.root_node();
        prop_assert_eq!(node.kind_id(), root.kind_id());
        prop_assert_eq!(node.start_byte(), root.start_byte());
        prop_assert_eq!(node.end_byte(), root.end_byte());
    }
}

// ===========================================================================
// 2 – Cursor depth at root is 0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn cursor_depth_at_root_is_zero(tree in arb_flat_tree()) {
        let cursor = TreeCursor::new(&tree);
        prop_assert_eq!(cursor.depth(), 0);
    }
}

// ===========================================================================
// 3 – goto_first_child returns true iff root has children
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn goto_first_child_matches_child_count(tree in arb_flat_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        let has_children = tree.root_node().child_count() > 0;
        prop_assert_eq!(cursor.goto_first_child(), has_children);
    }
}

// ===========================================================================
// 4 – After goto_first_child, depth is 1
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn depth_after_first_child_is_one(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        prop_assert!(cursor.goto_first_child());
        prop_assert_eq!(cursor.depth(), 1);
    }
}

// ===========================================================================
// 5 – First child node matches tree.root_node().child(0)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn first_child_node_matches(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        cursor.goto_first_child();
        let cursor_node = cursor.node();
        let expected = tree.root_node().child(0).unwrap();
        prop_assert_eq!(cursor_node.kind_id(), expected.kind_id());
        prop_assert_eq!(cursor_node.start_byte(), expected.start_byte());
        prop_assert_eq!(cursor_node.end_byte(), expected.end_byte());
    }
}

// ===========================================================================
// 6 – Sibling traversal visits all children
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn sibling_traversal_visits_all_children(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        let child_count = tree.root_node().child_count();
        cursor.goto_first_child();
        let mut visited = 1usize;
        while cursor.goto_next_sibling() {
            visited += 1;
        }
        prop_assert_eq!(visited, child_count);
    }
}

// ===========================================================================
// 7 – Sibling traversal matches child() access
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn sibling_nodes_match_indexed_children(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        let root = tree.root_node();
        cursor.goto_first_child();
        let mut idx = 0usize;
        loop {
            let expected = root.child(idx).unwrap();
            let node = cursor.node();
            prop_assert_eq!(node.kind_id(), expected.kind_id());
            prop_assert_eq!(node.start_byte(), expected.start_byte());
            idx += 1;
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        prop_assert_eq!(idx, root.child_count());
    }
}

// ===========================================================================
// 8 – goto_next_sibling on last child returns false
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn next_sibling_false_at_last_child(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        cursor.goto_first_child();
        while cursor.goto_next_sibling() {}
        // One more call should still return false
        prop_assert!(!cursor.goto_next_sibling());
    }
}

// ===========================================================================
// 9 – goto_parent returns to root after first_child
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn parent_after_first_child_returns_to_root(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        cursor.goto_first_child();
        prop_assert!(cursor.goto_parent());
        prop_assert_eq!(cursor.depth(), 0);
        prop_assert_eq!(cursor.node().kind_id(), tree.root_node().kind_id());
    }
}

// ===========================================================================
// 10 – goto_parent at root returns false
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn parent_at_root_returns_false(tree in arb_flat_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        prop_assert!(!cursor.goto_parent());
    }
}

// ===========================================================================
// 11 – Roundtrip: child then parent preserves node
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn child_then_parent_roundtrip(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        let root_kind = cursor.node().kind_id();
        let root_start = cursor.node().start_byte();
        cursor.goto_first_child();
        cursor.goto_parent();
        prop_assert_eq!(cursor.node().kind_id(), root_kind);
        prop_assert_eq!(cursor.node().start_byte(), root_start);
    }
}

// ===========================================================================
// 12 – Roundtrip: sibling then parent preserves parent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn sibling_then_parent_roundtrip(
        tree in (0u32..500, 0usize..5_000, 1usize..5_000)
            .prop_map(|(sym, start, span)| {
                let end = start + span;
                flat_tree(sym, start, end, 3)
            })
    ) {
        let mut cursor = TreeCursor::new(&tree);
        let root_kind = cursor.node().kind_id();
        cursor.goto_first_child();
        cursor.goto_next_sibling();
        cursor.goto_parent();
        prop_assert_eq!(cursor.node().kind_id(), root_kind);
        prop_assert_eq!(cursor.depth(), 0);
    }
}

// ===========================================================================
// 13 – Reset returns cursor to root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn reset_returns_to_root(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        cursor.goto_first_child();
        cursor.goto_next_sibling();
        cursor.reset(&tree);
        prop_assert_eq!(cursor.depth(), 0);
        prop_assert_eq!(cursor.node().kind_id(), tree.root_node().kind_id());
    }
}

// ===========================================================================
// 14 – Reset after deep traversal
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn reset_after_deep_traversal((tree, _depth) in arb_deep_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        while cursor.goto_first_child() {}
        cursor.reset(&tree);
        prop_assert_eq!(cursor.depth(), 0);
        prop_assert_eq!(cursor.node().start_byte(), tree.root_node().start_byte());
    }
}

// ===========================================================================
// 15 – Leaf node: goto_first_child returns false
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn leaf_node_has_no_children(sym in 0u32..500, start in 0usize..5_000, span in 1usize..5_000) {
        let tree = Tree::new_for_testing(sym, start, start + span, vec![]);
        let mut cursor = TreeCursor::new(&tree);
        prop_assert!(!cursor.goto_first_child());
    }
}

// ===========================================================================
// 16 – Leaf node: goto_next_sibling returns false
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn leaf_root_has_no_sibling(sym in 0u32..500, start in 0usize..5_000, span in 1usize..5_000) {
        let tree = Tree::new_for_testing(sym, start, start + span, vec![]);
        let mut cursor = TreeCursor::new(&tree);
        prop_assert!(!cursor.goto_next_sibling());
    }
}

// ===========================================================================
// 17 – Deep tree: depth tracks correctly on descent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn depth_tracks_on_descent((tree, depth) in arb_deep_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        let mut d = 0usize;
        while cursor.goto_first_child() {
            d += 1;
            prop_assert_eq!(cursor.depth(), d);
        }
        prop_assert_eq!(d, depth);
    }
}

// ===========================================================================
// 18 – Deep tree: depth tracks correctly on ascent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn depth_tracks_on_ascent((tree, depth) in arb_deep_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        // Descend to leaf
        while cursor.goto_first_child() {}
        prop_assert_eq!(cursor.depth(), depth);
        // Ascend back
        let mut d = depth;
        while cursor.goto_parent() {
            d -= 1;
            prop_assert_eq!(cursor.depth(), d);
        }
        prop_assert_eq!(d, 0);
    }
}

// ===========================================================================
// 19 – Deep tree: node at leaf has correct symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn deep_tree_leaf_symbol((tree, _depth) in arb_deep_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        while cursor.goto_first_child() {}
        // Leaf symbol should be 0 (depth 0 in our deep_tree builder)
        prop_assert_eq!(cursor.node().kind_id(), 0);
    }
}

// ===========================================================================
// 20 – node() is idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn node_is_idempotent(tree in arb_flat_tree()) {
        let cursor = TreeCursor::new(&tree);
        let n1 = cursor.node();
        let n2 = cursor.node();
        prop_assert_eq!(n1.kind_id(), n2.kind_id());
        prop_assert_eq!(n1.start_byte(), n2.start_byte());
        prop_assert_eq!(n1.end_byte(), n2.end_byte());
    }
}

// ===========================================================================
// 21 – node() after child navigation returns child info
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn node_after_child_is_child(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        let root_kind = cursor.node().kind_id();
        cursor.goto_first_child();
        let child_kind = cursor.node().kind_id();
        // Child should differ from root (different symbol IDs in our builder)
        prop_assert_ne!(child_kind, root_kind);
    }
}

// ===========================================================================
// 22 – Stub tree cursor: all navigation returns false
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn stub_tree_cursor_all_false(_dummy in 0u32..1) {
        let tree = Tree::new_stub();
        let mut cursor = TreeCursor::new(&tree);
        prop_assert!(!cursor.goto_first_child());
        prop_assert!(!cursor.goto_next_sibling());
        prop_assert!(!cursor.goto_parent());
        prop_assert_eq!(cursor.depth(), 0);
    }
}

// ===========================================================================
// 23 – Full traversal: child → siblings → parent cycle
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn full_child_sibling_parent_cycle(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        let root_kind = cursor.node().kind_id();
        // go to first child
        cursor.goto_first_child();
        // walk all siblings
        while cursor.goto_next_sibling() {}
        // go back to parent
        cursor.goto_parent();
        prop_assert_eq!(cursor.node().kind_id(), root_kind);
        prop_assert_eq!(cursor.depth(), 0);
    }
}

// ===========================================================================
// 24 – Multiple resets are stable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn multiple_resets_stable(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        for _ in 0..5 {
            cursor.goto_first_child();
            cursor.reset(&tree);
            prop_assert_eq!(cursor.depth(), 0);
            prop_assert_eq!(cursor.node().kind_id(), tree.root_node().kind_id());
        }
    }
}

// ===========================================================================
// 25 – Reset to a different tree
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn reset_to_different_tree(
        tree_a in arb_flat_tree(),
        tree_b in arb_flat_tree(),
    ) {
        let mut cursor = TreeCursor::new(&tree_a);
        cursor.reset(&tree_b);
        prop_assert_eq!(cursor.depth(), 0);
        prop_assert_eq!(cursor.node().kind_id(), tree_b.root_node().kind_id());
        prop_assert_eq!(cursor.node().start_byte(), tree_b.root_node().start_byte());
    }
}

// ===========================================================================
// 26 – Depth never exceeds tree height
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn depth_bounded_by_tree_height((tree, depth) in arb_deep_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        let mut max_depth = 0usize;
        while cursor.goto_first_child() {
            max_depth += 1;
        }
        prop_assert!(max_depth <= depth);
        prop_assert!(cursor.depth() <= depth);
    }
}

// ===========================================================================
// 27 – goto_first_child on already-visited child is idempotent leaf behavior
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn first_child_of_leaf_child_is_false(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        cursor.goto_first_child();
        // Children in flat_tree are leaves
        prop_assert!(!cursor.goto_first_child());
        prop_assert_eq!(cursor.depth(), 1);
    }
}

// ===========================================================================
// 28 – goto_parent twice from depth-2 returns to root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn double_parent_from_depth_2((tree, depth) in arb_deep_tree().prop_filter(
        "need depth >= 2", |(_, d)| *d >= 2
    )) {
        let mut cursor = TreeCursor::new(&tree);
        cursor.goto_first_child();
        cursor.goto_first_child();
        prop_assert_eq!(cursor.depth(), 2);
        cursor.goto_parent();
        prop_assert_eq!(cursor.depth(), 1);
        cursor.goto_parent();
        prop_assert_eq!(cursor.depth(), 0);
        prop_assert_eq!(cursor.node().kind_id(), tree.root_node().kind_id());
        let _ = depth;
    }
}

// ===========================================================================
// 29 – Cursor node byte range is within parent range
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn child_byte_range_within_parent(tree in arb_flat_tree_with_children()) {
        let mut cursor = TreeCursor::new(&tree);
        let parent_start = cursor.node().start_byte();
        let parent_end = cursor.node().end_byte();
        cursor.goto_first_child();
        let child_start = cursor.node().start_byte();
        let child_end = cursor.node().end_byte();
        prop_assert!(child_start >= parent_start,
            "child start {} < parent start {}", child_start, parent_start);
        prop_assert!(child_end <= parent_end,
            "child end {} > parent end {}", child_end, parent_end);
    }
}

// ===========================================================================
// 30 – Depth after full descent and ascent is 0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn full_descent_ascent_returns_to_zero((tree, _depth) in arb_deep_tree()) {
        let mut cursor = TreeCursor::new(&tree);
        while cursor.goto_first_child() {}
        while cursor.goto_parent() {}
        prop_assert_eq!(cursor.depth(), 0);
    }
}

// ===========================================================================
// 31 – Cursor on zero-span tree works
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn zero_span_tree_cursor(sym in 0u32..500, pos in 0usize..10_000) {
        let tree = Tree::new_for_testing(sym, pos, pos, vec![]);
        let mut cursor = TreeCursor::new(&tree);
        prop_assert_eq!(cursor.depth(), 0);
        prop_assert_eq!(cursor.node().start_byte(), pos);
        prop_assert_eq!(cursor.node().end_byte(), pos);
        prop_assert!(!cursor.goto_first_child());
        prop_assert!(!cursor.goto_parent());
    }
}

// ===========================================================================
// 32 – Repeated goto_parent at root stays at root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn repeated_parent_at_root_stable(tree in arb_flat_tree(), n in 1usize..10) {
        let mut cursor = TreeCursor::new(&tree);
        for _ in 0..n {
            let _ = cursor.goto_parent();
        }
        prop_assert_eq!(cursor.depth(), 0);
        prop_assert_eq!(cursor.node().kind_id(), tree.root_node().kind_id());
    }
}

// ===========================================================================
// 33 – node().child_count() matches navigation count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn node_child_count_matches_navigation(tree in arb_flat_tree()) {
        let cursor = TreeCursor::new(&tree);
        let reported = cursor.node().child_count();
        let mut nav_cursor = TreeCursor::new(&tree);
        let navigated = if nav_cursor.goto_first_child() {
            let mut count = 1usize;
            while nav_cursor.goto_next_sibling() {
                count += 1;
            }
            count
        } else {
            0
        };
        prop_assert_eq!(reported, navigated);
    }
}

// ===========================================================================
// 34 – Full DFS traversal visits every node exactly once
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn full_dfs_visits_all_nodes(tree in arb_flat_tree()) {
        // Count nodes via child() API
        fn count_nodes(tree: &Tree) -> usize {
            fn walk(node: adze_runtime::node::Node<'_>) -> usize {
                let mut n = 1;
                for i in 0..node.child_count() {
                    n += walk(node.child(i).unwrap());
                }
                n
            }
            walk(tree.root_node())
        }
        let expected = count_nodes(&tree);

        // Count nodes via cursor DFS
        let mut cursor = TreeCursor::new(&tree);
        let mut visited = 0usize;
        let mut reached_end = false;
        loop {
            visited += 1;
            // Try deeper first
            if cursor.goto_first_child() {
                continue;
            }
            // Try next sibling
            if cursor.goto_next_sibling() {
                continue;
            }
            // Backtrack until we find a sibling or exhaust the tree
            loop {
                if !cursor.goto_parent() {
                    reached_end = true;
                    break;
                }
                if cursor.goto_next_sibling() {
                    break;
                }
            }
            if reached_end {
                break;
            }
        }
        prop_assert_eq!(visited, expected);
    }
}

// ===========================================================================
// 35 – Tree::root_node() returns Node directly with correct construction data
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn root_node_returns_node_with_construction_data(
        sym in 0u32..500,
        start in 0usize..5_000,
        span in 1usize..5_000,
        n_children in 0usize..6,
    ) {
        let end = start + span;
        let tree = flat_tree(sym, start, end, n_children);

        // root_node() returns Node directly (not Option)
        let root = tree.root_node();
        prop_assert_eq!(root.kind_id(), sym as u16);
        prop_assert_eq!(root.start_byte(), start);
        prop_assert_eq!(root.end_byte(), end);
        prop_assert_eq!(root.child_count(), n_children);
    }
}
