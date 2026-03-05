//! Comprehensive tests for `TreeCursor` navigation and node access.

use adze_runtime::Tree;
use adze_runtime::tree::TreeCursor;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// root(100, 0..10) -> [child(1, 0..5), child(2, 5..10)]
fn two_child_tree() -> Tree {
    Tree::new_for_testing(100, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)])
}

/// root(100, 0..20) -> [A(10, 0..10) -> [A1(11, 0..5), A2(12, 5..10)], B(20, 10..20)]
fn nested_tree() -> Tree {
    Tree::new_for_testing(
        100,
        0,
        20,
        vec![
            Tree::new_for_testing(10, 0, 10, vec![leaf(11, 0, 5), leaf(12, 5, 10)]),
            leaf(20, 10, 20),
        ],
    )
}

/// Builds a linear chain: depth-N tree where each node has one child.
/// Symbols: 0, 1, 2, …, depth. Byte range: 0..depth+1.
fn chain_tree(depth: usize) -> Tree {
    if depth == 0 {
        return leaf(0, 0, 1);
    }
    let mut current = leaf(depth as u32, 0, (depth + 1) as usize);
    for i in (0..depth).rev() {
        current = Tree::new_for_testing(i as u32, 0, (depth + 1) as usize, vec![current]);
    }
    current
}

/// root -> N leaf children with symbols 1..=N.
fn wide_tree(n: usize) -> Tree {
    let children: Vec<Tree> = (0..n)
        .map(|i| leaf((i + 1) as u32, i * 10, (i + 1) * 10))
        .collect();
    Tree::new_for_testing(0, 0, n * 10, children)
}

// ===========================================================================
// 1. Cursor creation at root (10 tests)
// ===========================================================================

#[test]
fn cursor_starts_at_root_node() {
    let tree = two_child_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 100);
}

#[test]
fn cursor_root_has_correct_symbol() {
    let tree = Tree::new_for_testing(42, 0, 5, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 42);
}

#[test]
fn cursor_root_has_correct_start_byte() {
    let tree = Tree::new_for_testing(0, 3, 10, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().start_byte(), 3);
}

#[test]
fn cursor_root_has_correct_end_byte() {
    let tree = Tree::new_for_testing(0, 3, 10, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().end_byte(), 10);
}

#[test]
fn cursor_root_has_correct_child_count() {
    let tree = two_child_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 2);
}

#[test]
fn cursor_starts_at_depth_zero() {
    let tree = nested_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_on_stub_tree_starts_at_root() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_on_single_leaf_starts_at_root() {
    let tree = leaf(77, 0, 5);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 77);
    assert_eq!(cursor.node().child_count(), 0);
}

#[test]
fn cursor_root_kind_is_unknown_without_language() {
    let tree = two_child_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind(), "unknown");
}

#[test]
fn cursor_root_node_is_named() {
    let tree = two_child_tree();
    let cursor = TreeCursor::new(&tree);
    assert!(cursor.node().is_named());
}

// ===========================================================================
// 2. Navigation: first_child, next_sibling, parent (15 tests)
// ===========================================================================

#[test]
fn goto_first_child_returns_true_when_children_exist() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
}

#[test]
fn goto_first_child_moves_to_first_child() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 5);
}

#[test]
fn goto_first_child_on_leaf_returns_false() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // at child(1)
    assert!(!cursor.goto_first_child());
}

#[test]
fn goto_next_sibling_returns_true_when_sibling_exists() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
}

#[test]
fn goto_next_sibling_moves_to_next_child() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);
    assert_eq!(cursor.node().start_byte(), 5);
}

#[test]
fn goto_next_sibling_on_last_child_returns_false() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // at child(2), last sibling
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn goto_parent_returns_true_from_child() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
}

#[test]
fn goto_parent_restores_parent_node() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_parent();
    assert_eq!(cursor.node().kind_id(), 100);
}

#[test]
fn goto_parent_at_root_returns_false() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn goto_first_child_increments_depth() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn goto_parent_decrements_depth() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn goto_next_sibling_keeps_same_depth() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn consecutive_next_sibling_visits_all_children() {
    let tree = Tree::new_for_testing(
        0,
        0,
        30,
        vec![leaf(1, 0, 10), leaf(2, 10, 20), leaf(3, 20, 30)],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn goto_first_child_then_parent_round_trip() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    let root_id = cursor.node().kind_id();
    cursor.goto_first_child();
    cursor.goto_parent();
    assert_eq!(cursor.node().kind_id(), root_id);
}

#[test]
fn goto_first_child_twice_reaches_grandchild() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A(10)
    cursor.goto_first_child(); // A1(11)
    assert_eq!(cursor.node().kind_id(), 11);
}

// ===========================================================================
// 3. Navigation sequences and round-trips (10 tests)
// ===========================================================================

#[test]
fn full_depth_first_traversal() {
    // root(100) -> [A(10) -> [A1(11), A2(12)], B(20)]
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    let mut visited: Vec<u16> = vec![cursor.node().kind_id()];

    // Iterative DFS
    let mut reached_end = false;
    // go to first child to start traversal
    if cursor.goto_first_child() {
        visited.push(cursor.node().kind_id());
        loop {
            // try going deeper
            if cursor.goto_first_child() {
                visited.push(cursor.node().kind_id());
                continue;
            }
            // try going right
            if cursor.goto_next_sibling() {
                visited.push(cursor.node().kind_id());
                continue;
            }
            // backtrack until we can go right or reach root
            loop {
                if !cursor.goto_parent() {
                    reached_end = true;
                    break;
                }
                if cursor.goto_next_sibling() {
                    visited.push(cursor.node().kind_id());
                    break;
                }
            }
            if reached_end {
                break;
            }
        }
    }

    assert_eq!(visited, vec![100, 10, 11, 12, 20]);
}

#[test]
fn navigate_to_last_sibling_and_back() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // child(2)
    assert_eq!(cursor.node().kind_id(), 2);
    cursor.goto_parent();
    cursor.goto_first_child(); // back to child(1)
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn parent_then_first_child_revisits_first_child() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A
    cursor.goto_first_child(); // A1
    cursor.goto_next_sibling(); // A2
    cursor.goto_parent(); // A
    cursor.goto_first_child(); // A1 again
    assert_eq!(cursor.node().kind_id(), 11);
}

#[test]
fn zigzag_child_sibling_parent() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A(10)
    assert_eq!(cursor.node().kind_id(), 10);
    cursor.goto_next_sibling(); // B(20)
    assert_eq!(cursor.node().kind_id(), 20);
    cursor.goto_parent(); // root
    assert_eq!(cursor.node().kind_id(), 100);
}

#[test]
fn navigate_right_then_down() {
    // root -> [A -> [A1], B -> [B1]]
    let tree = Tree::new_for_testing(
        0,
        0,
        20,
        vec![
            Tree::new_for_testing(10, 0, 10, vec![leaf(11, 0, 10)]),
            Tree::new_for_testing(20, 10, 20, vec![leaf(21, 10, 20)]),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A
    cursor.goto_next_sibling(); // B
    cursor.goto_first_child(); // B1
    assert_eq!(cursor.node().kind_id(), 21);
}

#[test]
fn repeated_parent_reaches_root() {
    let tree = chain_tree(4); // depth-4 chain
    let mut cursor = TreeCursor::new(&tree);
    // Go all the way down
    while cursor.goto_first_child() {}
    assert_eq!(cursor.depth(), 4);
    // Come all the way back
    while cursor.goto_parent() {}
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn first_child_parent_first_child_is_same_node() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let id1 = cursor.node().kind_id();
    let start1 = cursor.node().start_byte();
    cursor.goto_parent();
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), id1);
    assert_eq!(cursor.node().start_byte(), start1);
}

#[test]
fn navigate_to_deepest_then_back_to_root() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A(10)
    cursor.goto_first_child(); // A1(11)
    assert_eq!(cursor.node().kind_id(), 11);
    cursor.goto_parent(); // A
    cursor.goto_parent(); // root
    assert_eq!(cursor.node().kind_id(), 100);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn sibling_then_child_navigation() {
    // root -> [A(10) -> [], B(20) -> [B1(21), B2(22)]]
    let tree = Tree::new_for_testing(
        0,
        0,
        20,
        vec![
            leaf(10, 0, 10),
            Tree::new_for_testing(20, 10, 20, vec![leaf(21, 10, 15), leaf(22, 15, 20)]),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A
    cursor.goto_next_sibling(); // B
    cursor.goto_first_child(); // B1
    assert_eq!(cursor.node().kind_id(), 21);
    cursor.goto_next_sibling(); // B2
    assert_eq!(cursor.node().kind_id(), 22);
}

#[test]
fn complex_navigation_sequence() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);

    // root -> A -> A1 -> (no child) -> parent(A) -> A2 -> parent(A) -> sibling(B) -> parent(root)
    cursor.goto_first_child(); // A
    cursor.goto_first_child(); // A1
    assert!(!cursor.goto_first_child()); // A1 is leaf
    cursor.goto_parent(); // A
    cursor.goto_first_child(); // A1 (first child again)
    cursor.goto_next_sibling(); // A2
    assert_eq!(cursor.node().kind_id(), 12);
    cursor.goto_parent(); // A
    cursor.goto_next_sibling(); // B
    assert_eq!(cursor.node().kind_id(), 20);
    cursor.goto_parent(); // root
    assert_eq!(cursor.node().kind_id(), 100);
}

// ===========================================================================
// 4. Leaf node behavior (5 tests)
// ===========================================================================

#[test]
fn leaf_has_no_children() {
    let tree = leaf(5, 0, 3);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 0);
}

#[test]
fn leaf_first_child_returns_false() {
    let tree = leaf(5, 0, 3);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn leaf_child_count_is_zero() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // leaf child
    assert_eq!(cursor.node().child_count(), 0);
}

#[test]
fn leaf_node_has_correct_byte_range() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 5);
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().start_byte(), 5);
    assert_eq!(cursor.node().end_byte(), 10);
}

#[test]
fn leaf_next_sibling_after_parent_works() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A
    cursor.goto_first_child(); // A1 (leaf)
    assert!(!cursor.goto_first_child()); // can't go deeper
    cursor.goto_parent(); // A
    cursor.goto_next_sibling(); // B (leaf)
    assert_eq!(cursor.node().kind_id(), 20);
    assert!(!cursor.goto_first_child());
}

// ===========================================================================
// 5. Deep tree traversal (8 tests)
// ===========================================================================

#[test]
fn traverse_depth_5_tree() {
    let tree = chain_tree(5);
    let mut cursor = TreeCursor::new(&tree);
    for expected in 0..=5u16 {
        assert_eq!(cursor.node().kind_id(), expected);
        if expected < 5 {
            assert!(cursor.goto_first_child());
        }
    }
}

#[test]
fn depth_increases_going_down() {
    let tree = chain_tree(3);
    let mut cursor = TreeCursor::new(&tree);
    for d in 0..=3 {
        assert_eq!(cursor.depth(), d);
        if d < 3 {
            cursor.goto_first_child();
        }
    }
}

#[test]
fn depth_returns_to_zero_at_root() {
    let tree = chain_tree(4);
    let mut cursor = TreeCursor::new(&tree);
    while cursor.goto_first_child() {}
    while cursor.goto_parent() {}
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn deep_tree_leaf_has_correct_depth() {
    let tree = chain_tree(6);
    let mut cursor = TreeCursor::new(&tree);
    while cursor.goto_first_child() {}
    assert_eq!(cursor.depth(), 6);
}

#[test]
fn navigate_deep_then_sideways() {
    // root -> [A -> [A1], B]
    let tree = Tree::new_for_testing(
        0,
        0,
        20,
        vec![
            Tree::new_for_testing(10, 0, 10, vec![leaf(11, 0, 10)]),
            leaf(20, 10, 20),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A
    cursor.goto_first_child(); // A1
    assert_eq!(cursor.depth(), 2);
    // Can't go sideways from A1 (only child)
    assert!(!cursor.goto_next_sibling());
    cursor.goto_parent(); // A
    cursor.goto_next_sibling(); // B
    assert_eq!(cursor.node().kind_id(), 20);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn depth_4_first_child_chain() {
    // Build a 4-level tree manually
    let tree = Tree::new_for_testing(
        0,
        0,
        100,
        vec![Tree::new_for_testing(
            1,
            0,
            100,
            vec![Tree::new_for_testing(
                2,
                0,
                100,
                vec![Tree::new_for_testing(3, 0, 100, vec![leaf(4, 0, 100)])],
            )],
        )],
    );
    let mut cursor = TreeCursor::new(&tree);
    for expected_symbol in 0u16..=4 {
        assert_eq!(cursor.node().kind_id(), expected_symbol);
        if expected_symbol < 4 {
            assert!(cursor.goto_first_child());
        }
    }
    assert!(!cursor.goto_first_child()); // leaf
}

#[test]
fn deep_tree_parent_chain_back_to_root() {
    let tree = chain_tree(5);
    let mut cursor = TreeCursor::new(&tree);
    // Go to the bottom
    while cursor.goto_first_child() {}
    assert_eq!(cursor.node().kind_id(), 5);
    // Walk back verifying each level
    for expected in (0..5).rev() {
        assert!(cursor.goto_parent());
        assert_eq!(cursor.node().kind_id(), expected as u16);
    }
    assert!(!cursor.goto_parent());
}

#[test]
fn deep_tree_node_properties_correct() {
    let tree = chain_tree(3); // symbols 0,1,2,3 all with range 0..4
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // symbol 1
    cursor.goto_first_child(); // symbol 2
    assert_eq!(cursor.node().kind_id(), 2);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 4);
    assert_eq!(cursor.node().child_count(), 1);
}

// ===========================================================================
// 6. Wide tree with many siblings (8 tests)
// ===========================================================================

#[test]
fn wide_tree_has_correct_child_count() {
    let tree = wide_tree(5);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 5);
}

#[test]
fn iterate_all_siblings_in_wide_tree() {
    let tree = wide_tree(5);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut ids = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        ids.push(cursor.node().kind_id());
    }
    assert_eq!(ids, vec![1, 2, 3, 4, 5]);
}

#[test]
fn wide_tree_first_and_last_sibling_properties() {
    let tree = wide_tree(4);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.node().start_byte(), 0);
    // Navigate to last
    while cursor.goto_next_sibling() {}
    assert_eq!(cursor.node().kind_id(), 4);
    assert_eq!(cursor.node().start_byte(), 30);
    assert_eq!(cursor.node().end_byte(), 40);
}

#[test]
fn wide_tree_sibling_count_matches() {
    let tree = wide_tree(7);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 7);
}

#[test]
fn wide_tree_middle_sibling_properties() {
    let tree = wide_tree(5);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // 1
    cursor.goto_next_sibling(); // 2
    cursor.goto_next_sibling(); // 3 (middle)
    assert_eq!(cursor.node().kind_id(), 3);
    assert_eq!(cursor.node().start_byte(), 20);
    assert_eq!(cursor.node().end_byte(), 30);
}

#[test]
fn wide_tree_goto_parent_from_any_sibling() {
    let tree = wide_tree(4);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // child 2
    cursor.goto_next_sibling(); // child 3
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn wide_tree_no_children_on_leaves() {
    let tree = wide_tree(3);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    // Check each sibling is a leaf
    loop {
        assert_eq!(cursor.node().child_count(), 0);
        assert!(!cursor.goto_first_child());
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

#[test]
fn wide_tree_ten_children() {
    let tree = wide_tree(10);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 10);
    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 10);
    assert_eq!(cursor.node().kind_id(), 10);
}

// ===========================================================================
// 7. Node properties through cursor (6 tests)
// ===========================================================================

#[test]
fn node_kind_id_matches_symbol() {
    let tree = Tree::new_for_testing(255, 0, 1, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 255);
}

#[test]
fn node_byte_range_correct() {
    let tree = Tree::new_for_testing(0, 10, 50, vec![]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().byte_range(), 10..50);
}

#[test]
fn node_start_end_byte_correct_through_cursor() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // A(10, 0..10)
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 10);
    cursor.goto_next_sibling(); // B(20, 10..20)
    assert_eq!(cursor.node().start_byte(), 10);
    assert_eq!(cursor.node().end_byte(), 20);
}

#[test]
fn node_is_error_returns_false() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.node().is_error());
    cursor.goto_first_child();
    assert!(!cursor.node().is_error());
}

#[test]
fn node_child_via_cursor_matches_direct_access() {
    let tree = nested_tree();
    let root = tree.root_node();
    let direct_child = root.child(0).unwrap();

    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let cursor_child = cursor.node();

    assert_eq!(direct_child.kind_id(), cursor_child.kind_id());
    assert_eq!(direct_child.start_byte(), cursor_child.start_byte());
    assert_eq!(direct_child.end_byte(), cursor_child.end_byte());
    assert_eq!(direct_child.child_count(), cursor_child.child_count());
}

#[test]
fn node_child_count_consistent_through_cursor() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 2); // root has 2
    cursor.goto_first_child(); // A has 2 children
    assert_eq!(cursor.node().child_count(), 2);
    cursor.goto_first_child(); // A1 has 0
    assert_eq!(cursor.node().child_count(), 0);
}

// ===========================================================================
// 8. depth() and reset() behavior (4 tests)
// ===========================================================================

#[test]
fn depth_at_root_is_zero() {
    let tree = nested_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn depth_tracks_navigation_correctly() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    cursor.goto_next_sibling(); // sibling at same depth
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn reset_returns_cursor_to_root() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 100);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn reset_restores_depth_to_zero() {
    let tree = chain_tree(5);
    let mut cursor = TreeCursor::new(&tree);
    while cursor.goto_first_child() {}
    assert_eq!(cursor.depth(), 5);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
    // Can navigate again after reset
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

// ===========================================================================
// 9. Edge cases (6 tests)
// ===========================================================================

#[test]
fn root_goto_parent_returns_false() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
    // Cursor should still be at root
    assert_eq!(cursor.node().kind_id(), 100);
}

#[test]
fn root_goto_next_sibling_returns_false() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 100);
}

#[test]
fn last_sibling_goto_next_returns_false_stays_put() {
    let tree = wide_tree(3);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling(); // last child
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(!cursor.goto_next_sibling());
    // Should still be at last sibling
    assert_eq!(cursor.node().kind_id(), 3);
}

#[test]
fn empty_stub_tree_cursor_at_root() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn single_child_no_next_sibling() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(!cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn multiple_goto_parent_at_root_stays_at_root() {
    let tree = two_child_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
    assert!(!cursor.goto_parent());
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 100);
    assert_eq!(cursor.depth(), 0);
}
