//! Comprehensive tests for Tree construction and TreeCursor navigation.
//!
//! Coverage areas:
//! 1. Tree construction via `Tree::new_for_testing` and `Tree::new_stub`
//! 2. TreeCursor navigation: goto_first_child, goto_next_sibling, goto_parent, reset
//! 3. Node properties: kind(), start_byte(), end_byte(), child_count(), is_named(), etc.
//! 4. TreeCursor field_name() / child_by_field_name() behavior
//! 5. Deep tree traversal patterns
//! 6. Wide tree navigation
//! 7. Property-based tests for tree invariants (proptest)

use adze_runtime::tree::{Tree, TreeCursor};

// ============================================================================
// Helpers
// ============================================================================

/// Leaf node helper.
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Perform a full iterative DFS traversal with a cursor, returning kind_ids in visit order.
fn dfs_kind_ids(tree: &Tree) -> Vec<u16> {
    let mut cursor = TreeCursor::new(tree);
    let mut ids = Vec::new();
    let mut reached_root = false;

    loop {
        ids.push(cursor.node().kind_id());

        // Try going deeper first.
        if cursor.goto_first_child() {
            continue;
        }

        // Try going to sibling.
        if cursor.goto_next_sibling() {
            continue;
        }

        // Walk up until we can take a sibling, or we're back at root.
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

    ids
}

/// Count total nodes in a tree via DFS.
fn count_nodes(tree: &Tree) -> usize {
    dfs_kind_ids(tree).len()
}

// ============================================================================
// 1. Tree construction with Tree::new_for_testing
// ============================================================================

#[test]
fn construct_leaf_node() {
    let tree = leaf(42, 0, 5);
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 42);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn construct_one_child() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let child = root.child(0).unwrap();
    assert_eq!(child.kind_id(), 1);
}

#[test]
fn construct_multiple_children() {
    let tree = Tree::new_for_testing(
        0,
        0,
        30,
        vec![leaf(1, 0, 10), leaf(2, 10, 20), leaf(3, 20, 30)],
    );
    assert_eq!(tree.root_node().child_count(), 3);
}

#[test]
fn construct_nested_children() {
    let inner = Tree::new_for_testing(2, 0, 5, vec![leaf(3, 0, 5)]);
    let tree = Tree::new_for_testing(0, 0, 10, vec![inner]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let child = root.child(0).unwrap();
    assert_eq!(child.kind_id(), 2);
    assert_eq!(child.child_count(), 1);
    let grandchild = child.child(0).unwrap();
    assert_eq!(grandchild.kind_id(), 3);
}

#[test]
fn construct_preserves_byte_ranges() {
    let tree = Tree::new_for_testing(0, 10, 99, vec![leaf(1, 15, 50), leaf(2, 50, 90)]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 99);
    assert_eq!(root.child(0).unwrap().start_byte(), 15);
    assert_eq!(root.child(1).unwrap().end_byte(), 90);
}

#[test]
fn construct_zero_length_node() {
    let tree = leaf(7, 5, 5);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), root.end_byte());
}

#[test]
fn new_stub_creates_empty_tree() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn construct_deeply_nested() {
    // 6 levels deep: 0 -> 1 -> 2 -> 3 -> 4 -> 5
    let mut tree = leaf(5, 0, 2);
    for sym in (0..5).rev() {
        tree = Tree::new_for_testing(sym, 0, 10, vec![tree]);
    }
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
    // Walk down and verify
    let c1 = root.child(0).unwrap();
    assert_eq!(c1.kind_id(), 1);
    let c2 = c1.child(0).unwrap();
    assert_eq!(c2.kind_id(), 2);
}

// ============================================================================
// 2. TreeCursor navigation
// ============================================================================

#[test]
fn cursor_starts_at_root() {
    let tree = Tree::new_for_testing(99, 0, 50, vec![leaf(1, 0, 25)]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 99);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn goto_first_child_returns_true_when_children_exist() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn goto_first_child_returns_false_on_leaf() {
    let tree = leaf(0, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn goto_first_child_selects_first_child() {
    let tree = Tree::new_for_testing(0, 0, 20, vec![leaf(10, 0, 10), leaf(20, 10, 20)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 10);
}

#[test]
fn goto_next_sibling_advances() {
    let tree = Tree::new_for_testing(0, 0, 20, vec![leaf(1, 0, 10), leaf(2, 10, 20)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn goto_next_sibling_false_at_last_child() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // at child 2
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn goto_next_sibling_false_at_root() {
    let tree = leaf(0, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn goto_parent_from_child() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn goto_parent_false_at_root() {
    let tree = leaf(0, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn goto_parent_restores_correct_node() {
    let tree = Tree::new_for_testing(
        0,
        0,
        30,
        vec![
            Tree::new_for_testing(1, 0, 15, vec![leaf(11, 0, 8), leaf(12, 8, 15)]),
            leaf(2, 15, 30),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // node 1
    cursor.goto_first_child(); // node 11
    cursor.goto_next_sibling(); // node 12
    cursor.goto_parent(); // back to node 1
    assert_eq!(cursor.node().kind_id(), 1);
    // And we should still be able to reach sibling 2
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn reset_returns_to_root() {
    let tree = Tree::new_for_testing(0, 0, 20, vec![leaf(1, 0, 10), leaf(2, 10, 20)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn reset_after_deep_traversal() {
    let mut t = leaf(5, 0, 1);
    for i in (0..5).rev() {
        t = Tree::new_for_testing(i, 0, 10, vec![t]);
    }
    let mut cursor = TreeCursor::new(&t);
    while cursor.goto_first_child() {}
    assert!(cursor.depth() > 0);
    cursor.reset(&t);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn depth_tracks_through_navigation() {
    let tree = Tree::new_for_testing(
        0,
        0,
        20,
        vec![Tree::new_for_testing(1, 0, 10, vec![leaf(2, 0, 5)])],
    );
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn multiple_cursors_are_independent() {
    let tree = Tree::new_for_testing(0, 0, 20, vec![leaf(1, 0, 10), leaf(2, 10, 20)]);
    let mut c1 = TreeCursor::new(&tree);
    let c2 = TreeCursor::new(&tree);
    c1.goto_first_child();
    assert_eq!(c1.depth(), 1);
    assert_eq!(c2.depth(), 0);
}

#[test]
fn goto_first_child_then_immediate_parent() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..10 {
        assert!(cursor.goto_first_child());
        assert!(cursor.goto_parent());
        assert_eq!(cursor.depth(), 0);
    }
}

#[test]
fn repeated_goto_parent_at_root_is_idempotent() {
    let tree = leaf(0, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..5 {
        assert!(!cursor.goto_parent());
        assert_eq!(cursor.depth(), 0);
    }
}

#[test]
fn cursor_depth_unchanged_across_siblings() {
    let tree = Tree::new_for_testing(
        0,
        0,
        50,
        vec![
            leaf(1, 0, 10),
            leaf(2, 10, 20),
            leaf(3, 20, 30),
            leaf(4, 30, 40),
            leaf(5, 40, 50),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    for _ in 0..4 {
        assert_eq!(cursor.depth(), 1);
        cursor.goto_next_sibling();
    }
    assert_eq!(cursor.depth(), 1);
}

// ============================================================================
// 3. Node properties
// ============================================================================

#[test]
fn node_kind_returns_unknown_without_language() {
    let tree = leaf(42, 0, 5);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn node_kind_id_returns_symbol() {
    let tree = leaf(123, 0, 5);
    assert_eq!(tree.root_node().kind_id(), 123);
}

#[test]
fn node_start_and_end_byte() {
    let tree = leaf(0, 7, 42);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 7);
    assert_eq!(root.end_byte(), 42);
}

#[test]
fn node_child_count_matches_children() {
    let tree = Tree::new_for_testing(
        0,
        0,
        40,
        vec![
            leaf(1, 0, 10),
            leaf(2, 10, 20),
            leaf(3, 20, 30),
            leaf(4, 30, 40),
        ],
    );
    assert_eq!(tree.root_node().child_count(), 4);
}

#[test]
fn node_is_named_always_true() {
    let tree = leaf(0, 0, 5);
    assert!(tree.root_node().is_named());
}

#[test]
fn node_is_missing_always_false() {
    let tree = leaf(0, 0, 5);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn node_is_error_always_false() {
    let tree = leaf(0, 0, 5);
    assert!(!tree.root_node().is_error());
}

#[test]
fn node_byte_range_matches_start_end() {
    let tree = leaf(0, 3, 17);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 3..17);
}

#[test]
fn node_named_child_count_equals_child_count() {
    let tree = Tree::new_for_testing(0, 0, 20, vec![leaf(1, 0, 10), leaf(2, 10, 20)]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn node_child_by_index() {
    let tree = Tree::new_for_testing(0, 0, 20, vec![leaf(10, 0, 10), leaf(20, 10, 20)]);
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().kind_id(), 10);
    assert_eq!(root.child(1).unwrap().kind_id(), 20);
    assert!(root.child(2).is_none());
}

#[test]
fn node_child_out_of_bounds_returns_none() {
    let tree = leaf(0, 0, 5);
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(999).is_none());
}

#[test]
fn node_utf8_text() {
    let tree = Tree::new_for_testing(0, 0, 11, vec![leaf(1, 0, 5), leaf(2, 6, 11)]);
    let source = b"hello world";
    let root = tree.root_node();
    assert_eq!(root.utf8_text(source).unwrap(), "hello world");
    assert_eq!(root.child(0).unwrap().utf8_text(source).unwrap(), "hello");
    assert_eq!(root.child(1).unwrap().utf8_text(source).unwrap(), "world");
}

#[test]
fn node_start_and_end_position_are_zero() {
    // Phase 1 stubs return (0,0)
    let tree = leaf(0, 10, 20);
    let root = tree.root_node();
    assert_eq!(root.start_position(), adze_runtime::Point::new(0, 0));
    assert_eq!(root.end_position(), adze_runtime::Point::new(0, 0));
}

// ============================================================================
// 4. field_name / child_by_field_name behavior
// ============================================================================

#[test]
fn child_by_field_name_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    assert!(tree.root_node().child_by_field_name("left").is_none());
    assert!(tree.root_node().child_by_field_name("right").is_none());
}

#[test]
fn child_by_field_name_empty_string_returns_none() {
    let tree = leaf(0, 0, 5);
    assert!(tree.root_node().child_by_field_name("").is_none());
}

#[test]
fn node_parent_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    assert!(tree.root_node().parent().is_none());
    assert!(tree.root_node().child(0).unwrap().parent().is_none());
}

#[test]
fn node_sibling_links_return_none() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let first = tree.root_node().child(0).unwrap();
    assert!(first.next_sibling().is_none());
    assert!(first.prev_sibling().is_none());
    assert!(first.next_named_sibling().is_none());
    assert!(first.prev_named_sibling().is_none());
}

// ============================================================================
// 5. Deep tree traversal patterns
// ============================================================================

fn make_deep_chain(depth: u32) -> Tree {
    let mut t = leaf(depth, 0, 1);
    for i in (0..depth).rev() {
        t = Tree::new_for_testing(i, 0, 10, vec![t]);
    }
    t
}

#[test]
fn traverse_deep_chain_to_bottom() {
    let tree = make_deep_chain(10);
    let mut cursor = TreeCursor::new(&tree);
    let mut max_depth = 0;
    while cursor.goto_first_child() {
        max_depth += 1;
    }
    assert_eq!(max_depth, 10);
    assert_eq!(cursor.node().kind_id(), 10);
}

#[test]
fn deep_tree_round_trip() {
    let tree = make_deep_chain(8);
    let mut cursor = TreeCursor::new(&tree);
    let mut depth = 0;
    while cursor.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, 8);
    while cursor.goto_parent() {
        depth -= 1;
    }
    assert_eq!(depth, 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn dfs_visits_all_nodes_in_deep_chain() {
    let tree = make_deep_chain(5);
    let ids = dfs_kind_ids(&tree);
    // symbols 0,1,2,3,4,5
    assert_eq!(ids, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn dfs_complex_tree_order() {
    //       0
    //      / \
    //     1   2
    //    / \
    //   3   4
    let tree = Tree::new_for_testing(
        0,
        0,
        50,
        vec![
            Tree::new_for_testing(1, 0, 25, vec![leaf(3, 0, 12), leaf(4, 12, 25)]),
            leaf(2, 25, 50),
        ],
    );
    let ids = dfs_kind_ids(&tree);
    assert_eq!(ids, vec![0, 1, 3, 4, 2]);
}

#[test]
fn dfs_single_node_tree() {
    let tree = leaf(77, 0, 1);
    let ids = dfs_kind_ids(&tree);
    assert_eq!(ids, vec![77]);
}

#[test]
fn dfs_asymmetric_tree() {
    //        0
    //      / | \
    //     1  2  3
    //    /      |
    //   4       5
    //          / \
    //         6   7
    let tree = Tree::new_for_testing(
        0,
        0,
        70,
        vec![
            Tree::new_for_testing(1, 0, 20, vec![leaf(4, 0, 10)]),
            leaf(2, 20, 30),
            Tree::new_for_testing(
                3,
                30,
                70,
                vec![Tree::new_for_testing(
                    5,
                    30,
                    70,
                    vec![leaf(6, 30, 50), leaf(7, 50, 70)],
                )],
            ),
        ],
    );
    let ids = dfs_kind_ids(&tree);
    assert_eq!(ids, vec![0, 1, 4, 2, 3, 5, 6, 7]);
}

// ============================================================================
// 6. Wide tree navigation
// ============================================================================

fn make_wide_tree(width: u32) -> Tree {
    let children: Vec<Tree> = (0..width)
        .map(|i| {
            let start = (i * 10) as usize;
            leaf(i + 1, start, start + 10)
        })
        .collect();
    Tree::new_for_testing(0, 0, (width * 10) as usize, children)
}

#[test]
fn iterate_all_siblings_wide() {
    let tree = make_wide_tree(10);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 10);
}

#[test]
fn sibling_count_matches_child_count() {
    let tree = make_wide_tree(7);
    let root = tree.root_node();
    let expected = root.child_count();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, expected);
}

#[test]
fn wide_tree_siblings_byte_ranges_non_overlapping() {
    let tree = make_wide_tree(5);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut prev_end = cursor.node().start_byte();
    loop {
        let node = cursor.node();
        assert!(node.start_byte() >= prev_end, "siblings must not overlap");
        prev_end = node.end_byte();
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

#[test]
fn navigate_to_last_sibling() {
    let tree = make_wide_tree(6);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    while cursor.goto_next_sibling() {}
    // Last child symbol is width (6)
    assert_eq!(cursor.node().kind_id(), 6);
}

#[test]
fn wide_tree_dfs_visits_all() {
    let tree = make_wide_tree(8);
    let count = count_nodes(&tree);
    // root + 8 children = 9
    assert_eq!(count, 9);
}

// ============================================================================
// 7. Additional edge cases & combined patterns
// ============================================================================

#[test]
fn cursor_node_after_failed_goto_first_child() {
    let tree = leaf(42, 3, 9);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    // Current node unchanged
    assert_eq!(cursor.node().kind_id(), 42);
    assert_eq!(cursor.node().start_byte(), 3);
}

#[test]
fn cursor_node_after_failed_goto_next_sibling() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // at 1
    assert!(!cursor.goto_next_sibling()); // only child
    assert_eq!(cursor.node().kind_id(), 1); // still at 1
}

#[test]
fn tree_clone_is_independent() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let cloned = tree.clone();
    // Both trees work independently
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
}

#[test]
fn tree_debug_does_not_panic() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 5)]);
    let debug_str = format!("{:?}", tree);
    assert!(!debug_str.is_empty());
}

#[test]
fn tree_language_is_none_for_testing_tree() {
    let tree = leaf(0, 0, 5);
    assert!(tree.language().is_none());
}

#[test]
fn tree_source_bytes_is_none_for_testing_tree() {
    let tree = leaf(0, 0, 5);
    assert!(tree.source_bytes().is_none());
}

#[test]
fn tree_root_kind() {
    let tree = Tree::new_for_testing(55, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 55);
}

#[test]
fn reset_to_different_tree() {
    let tree1 = leaf(1, 0, 5);
    let tree2 = Tree::new_for_testing(2, 0, 20, vec![leaf(3, 0, 10), leaf(4, 10, 20)]);
    let mut cursor = TreeCursor::new(&tree1);
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.reset(&tree2);
    assert_eq!(cursor.node().kind_id(), 2);
    assert_eq!(cursor.node().child_count(), 2);
}

#[test]
fn navigate_complex_pattern_down_sibling_down() {
    //       0
    //      / \
    //     1   2
    //    /   / \
    //   3   4   5
    let tree = Tree::new_for_testing(
        0,
        0,
        60,
        vec![
            Tree::new_for_testing(1, 0, 20, vec![leaf(3, 0, 20)]),
            Tree::new_for_testing(2, 20, 60, vec![leaf(4, 20, 40), leaf(5, 40, 60)]),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);

    // Down to 1
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);

    // Sibling to 2
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);

    // Down into 2's first child (4)
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 4);

    // Sibling to 5
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 5);

    // Back to 2
    cursor.goto_parent();
    assert_eq!(cursor.node().kind_id(), 2);

    // Back to root
    cursor.goto_parent();
    assert_eq!(cursor.node().kind_id(), 0);
}

// ============================================================================
// 7. Property-based tests (proptest)
// ============================================================================

mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for a simple well-structured tree where we know exact counts.
    fn arb_simple_tree() -> impl Strategy<Value = Tree> {
        (1u32..=8).prop_map(|width| {
            let children: Vec<Tree> = (0..width)
                .map(|i| leaf(i + 1, (i * 10) as usize, ((i + 1) * 10) as usize))
                .collect();
            Tree::new_for_testing(0, 0, (width * 10) as usize, children)
        })
    }

    proptest! {
        #[test]
        fn prop_root_start_lte_end(sym in 0u32..500, start in 0usize..1000, len in 0usize..500) {
            let tree = leaf(sym, start, start + len);
            let root = tree.root_node();
            prop_assert!(root.start_byte() <= root.end_byte());
        }

        #[test]
        fn prop_child_ranges_within_parent(width in 1u32..=6) {
            let tree = make_wide_tree(width);
            let root = tree.root_node();
            for i in 0..root.child_count() {
                let child = root.child(i).unwrap();
                prop_assert!(child.start_byte() >= root.start_byte());
                prop_assert!(child.end_byte() <= root.end_byte());
            }
        }

        #[test]
        fn prop_depth_equals_goto_first_child_count(depth in 1u32..=12) {
            let tree = make_deep_chain(depth);
            let mut cursor = TreeCursor::new(&tree);
            let mut actual_depth = 0;
            while cursor.goto_first_child() {
                actual_depth += 1;
            }
            prop_assert_eq!(actual_depth, depth);
        }

        #[test]
        fn prop_sibling_count_equals_child_count(tree in arb_simple_tree()) {
            let root = tree.root_node();
            let expected = root.child_count();
            let mut cursor = TreeCursor::new(&tree);
            if !cursor.goto_first_child() {
                prop_assert_eq!(expected, 0);
                return Ok(());
            }
            let mut count = 1;
            while cursor.goto_next_sibling() {
                count += 1;
            }
            prop_assert_eq!(count, expected);
        }

        #[test]
        fn prop_reset_always_returns_to_depth_zero(tree in arb_simple_tree()) {
            let mut cursor = TreeCursor::new(&tree);
            cursor.goto_first_child();
            cursor.goto_next_sibling();
            cursor.reset(&tree);
            prop_assert_eq!(cursor.depth(), 0);
        }

        #[test]
        fn prop_dfs_count_matches_wide(width in 1u32..=10) {
            let tree = make_wide_tree(width);
            let count = count_nodes(&tree);
            // root + width children
            prop_assert_eq!(count, (width + 1) as usize);
        }

        #[test]
        fn prop_dfs_count_matches_deep(depth in 1u32..=15) {
            let tree = make_deep_chain(depth);
            let count = count_nodes(&tree);
            // depth + 1 nodes (0..=depth)
            prop_assert_eq!(count, (depth + 1) as usize);
        }

        #[test]
        fn prop_goto_parent_after_first_child_restores_depth(tree in arb_simple_tree()) {
            let mut cursor = TreeCursor::new(&tree);
            let before = cursor.depth();
            if cursor.goto_first_child() {
                prop_assert_eq!(cursor.depth(), before + 1);
                prop_assert!(cursor.goto_parent());
                prop_assert_eq!(cursor.depth(), before);
            }
        }

        #[test]
        fn prop_kind_id_matches_construction(sym in 0u32..500) {
            let tree = leaf(sym, 0, 1);
            prop_assert_eq!(tree.root_node().kind_id(), sym as u16);
        }

        #[test]
        fn prop_byte_range_consistent(start in 0usize..1000, len in 0usize..500) {
            let end = start + len;
            let tree = leaf(0, start, end);
            let root = tree.root_node();
            prop_assert_eq!(root.byte_range(), start..end);
            prop_assert_eq!(root.start_byte(), start);
            prop_assert_eq!(root.end_byte(), end);
        }
    }
}
