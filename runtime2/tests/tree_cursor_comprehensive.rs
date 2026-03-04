//! Comprehensive tests for TreeCursor navigation and traversal
//!
//! Tests cover:
//! - TreeCursor creation and initialization
//! - Navigation methods (goto_first_child, goto_next_sibling, goto_parent, reset)
//! - Node access at each cursor position
//! - Depth tracking
//! - Full tree traversals (depth-first)
//! - Edge cases (leaf nodes, single children, deep/wide trees)
//! - Navigation state assertions

use adze_runtime::tree::{Tree, TreeCursor};

// ============================================================================
// Helper Functions to Build Test Trees
// ============================================================================

/// Build a simple tree with 2 children:
/// ```
///     root(0)
///    /        \
///  child1(1) child2(2)
/// ```
fn simple_two_child_tree() -> Tree {
    Tree::new_for_testing(
        0,  // root symbol
        0,  // start_byte
        10, // end_byte
        vec![
            Tree::new_for_testing(1, 0, 5, vec![]),  // child1
            Tree::new_for_testing(2, 5, 10, vec![]), // child2
        ],
    )
}

/// Build a tree with nested children:
/// ```
///       root(0)
///      /        \
///    child1(1) child2(2)
///    /
///  grandchild(3)
/// ```
fn nested_tree() -> Tree {
    Tree::new_for_testing(
        0,
        0,
        20,
        vec![
            Tree::new_for_testing(1, 0, 10, vec![Tree::new_for_testing(3, 0, 5, vec![])]),
            Tree::new_for_testing(2, 10, 20, vec![]),
        ],
    )
}

/// Build a single node tree (just root, no children)
fn single_node_tree() -> Tree {
    Tree::new_for_testing(0, 0, 5, vec![])
}

/// Build a deep tree (many levels):
/// ```
/// root(0)
///   -> child1(1)
///     -> child1_1(11)
///       -> child1_1_1(111)
/// ```
fn deep_tree() -> Tree {
    Tree::new_for_testing(
        0,
        0,
        100,
        vec![Tree::new_for_testing(
            1,
            0,
            75,
            vec![Tree::new_for_testing(
                11,
                0,
                50,
                vec![Tree::new_for_testing(
                    111,
                    0,
                    25,
                    vec![Tree::new_for_testing(1111, 0, 10, vec![])],
                )],
            )],
        )],
    )
}

/// Build a wide tree (many siblings):
/// ```
///            root(0)
///   /    /    |    \    \
///  c1(1) c2(2) c3(3) c4(4) c5(5)
/// ```
fn wide_tree() -> Tree {
    Tree::new_for_testing(
        0,
        0,
        50,
        vec![
            Tree::new_for_testing(1, 0, 10, vec![]),
            Tree::new_for_testing(2, 10, 20, vec![]),
            Tree::new_for_testing(3, 20, 30, vec![]),
            Tree::new_for_testing(4, 30, 40, vec![]),
            Tree::new_for_testing(5, 40, 50, vec![]),
        ],
    )
}

/// Build a complex tree with mixed depth and width
fn complex_tree() -> Tree {
    Tree::new_for_testing(
        0,
        0,
        100,
        vec![
            Tree::new_for_testing(
                1,
                0,
                30,
                vec![
                    Tree::new_for_testing(11, 0, 10, vec![]),
                    Tree::new_for_testing(12, 10, 20, vec![]),
                    Tree::new_for_testing(13, 20, 30, vec![]),
                ],
            ),
            Tree::new_for_testing(
                2,
                30,
                65,
                vec![
                    Tree::new_for_testing(
                        21,
                        30,
                        47,
                        vec![
                            Tree::new_for_testing(211, 30, 38, vec![]),
                            Tree::new_for_testing(212, 38, 47, vec![]),
                        ],
                    ),
                    Tree::new_for_testing(22, 47, 65, vec![]),
                ],
            ),
            Tree::new_for_testing(3, 65, 100, vec![]),
        ],
    )
}

// ============================================================================
// Tests: 1-6 TreeCursor Creation and Basic Navigation
// ============================================================================

#[test]
fn test_cursor_creation_at_root() {
    let tree = simple_two_child_tree();
    let cursor = TreeCursor::new(&tree);

    // Cursor should start at root
    let node = cursor.node();
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 10);
    assert_eq!(node.child_count(), 2);
}

#[test]
fn test_cursor_depth_at_root() {
    let tree = simple_two_child_tree();
    let cursor = TreeCursor::new(&tree);

    // Depth at root should be 0
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_node_returns_correct_node() {
    let tree = simple_two_child_tree();
    let cursor = TreeCursor::new(&tree);
    let node = cursor.node();

    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 10);
    assert_eq!(node.child_count(), 2);
}

#[test]
fn test_cursor_creation_from_stub_tree() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);

    // Should not panic and should be at root
    assert_eq!(cursor.depth(), 0);
    let node = cursor.node();
    assert_eq!(node.child_count(), 0);
}

#[test]
fn test_cursor_multiple_independent_cursors() {
    let tree = simple_two_child_tree();
    let cursor1 = TreeCursor::new(&tree);
    let mut cursor2 = TreeCursor::new(&tree);

    // Both start at root, but can navigate independently
    assert_eq!(cursor1.depth(), 0);
    assert_eq!(cursor2.depth(), 0);

    cursor2.goto_first_child();
    assert_eq!(cursor2.depth(), 1);
    // cursor1 should still be at root
    assert_eq!(cursor1.depth(), 0);
}

#[test]
fn test_cursor_from_different_tree_types() {
    let single = single_node_tree();
    let nested = nested_tree();
    let deep = deep_tree();

    let c1 = TreeCursor::new(&single);
    let c2 = TreeCursor::new(&nested);
    let c3 = TreeCursor::new(&deep);

    assert_eq!(c1.depth(), 0);
    assert_eq!(c2.depth(), 0);
    assert_eq!(c3.depth(), 0);
}

// ============================================================================
// Tests: 7-12 goto_first_child Navigation
// ============================================================================

#[test]
fn test_goto_first_child_returns_true() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    assert!(cursor.goto_first_child());
}

#[test]
fn test_goto_first_child_advances_depth() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn test_goto_first_child_updates_node() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    let node = cursor.node();
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 5);
}

#[test]
fn test_goto_first_child_on_leaf_returns_false() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    // First child is a leaf (has no children)
    assert!(!cursor.goto_first_child());
    // Depth should not change
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn test_goto_first_child_on_single_child() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);

    // First child has one child
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);

    // Can navigate to grandchild
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);

    let node = cursor.node();
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 5);
}

#[test]
fn test_goto_first_child_on_empty_tree() {
    let tree = single_node_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Single node tree has no children
    assert!(!cursor.goto_first_child());
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// Tests: 13-18 goto_next_sibling Navigation
// ============================================================================

#[test]
fn test_goto_next_sibling_returns_true() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
}

#[test]
fn test_goto_next_sibling_updates_node() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    let first_child = cursor.node();
    let first_end = first_child.end_byte();

    cursor.goto_next_sibling();
    let second_child = cursor.node();
    assert_eq!(second_child.start_byte(), first_end);
}

#[test]
fn test_goto_next_sibling_depth_unchanged() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    let depth_before = cursor.depth();

    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), depth_before);
}

#[test]
fn test_goto_next_sibling_on_last_sibling_returns_false() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    cursor.goto_next_sibling(); // Move to second (last) child

    // No more siblings
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn test_goto_next_sibling_at_root_returns_false() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Root has no siblings
    assert!(!cursor.goto_next_sibling());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_goto_next_sibling_multiple_steps() {
    let tree = wide_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    let mut count = 1;

    while cursor.goto_next_sibling() {
        count += 1;
    }

    assert_eq!(count, 5); // Wide tree has 5 children
}

// ============================================================================
// Tests: 19-24 goto_parent Navigation
// ============================================================================

#[test]
fn test_goto_parent_returns_true() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    assert!(cursor.goto_parent());
}

#[test]
fn test_goto_parent_updates_depth() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);

    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_goto_parent_restores_node() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    let root_node = cursor.node();
    let root_start = root_node.start_byte();
    let root_end = root_node.end_byte();

    cursor.goto_first_child();
    cursor.goto_parent();

    let restored = cursor.node();
    assert_eq!(restored.start_byte(), root_start);
    assert_eq!(restored.end_byte(), root_end);
}

#[test]
fn test_goto_parent_from_grandchild() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Navigate down two levels
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);

    // Go back up
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);

    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_goto_parent_at_root_returns_false() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Already at root
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// Tests: 25-28 Depth Tracking
// ============================================================================

#[test]
fn test_depth_increases_with_goto_first_child() {
    let tree = deep_tree();
    let mut cursor = TreeCursor::new(&tree);

    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 3);
}

#[test]
fn test_depth_unchanged_with_goto_next_sibling() {
    let tree = wide_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    let depth = cursor.depth();

    for _ in 0..4 {
        cursor.goto_next_sibling();
        assert_eq!(cursor.depth(), depth);
    }
}

#[test]
fn test_depth_after_parent_navigation() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    cursor.goto_first_child();
    let depth = cursor.depth();

    cursor.goto_parent();
    assert_eq!(cursor.depth(), depth - 1);

    cursor.goto_parent();
    assert_eq!(cursor.depth(), depth - 2);
}

#[test]
fn test_depth_with_mixed_navigation() {
    let tree = complex_tree();
    let mut cursor = TreeCursor::new(&tree);

    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child(); // depth 1
    assert_eq!(cursor.depth(), 1);

    cursor.goto_first_child(); // depth 2
    assert_eq!(cursor.depth(), 2);

    cursor.goto_next_sibling(); // depth still 2
    assert_eq!(cursor.depth(), 2);

    cursor.goto_parent(); // depth 1
    assert_eq!(cursor.depth(), 1);

    cursor.goto_next_sibling(); // depth still 1
    assert_eq!(cursor.depth(), 1);
}

// ============================================================================
// Tests: 29-32 Cursor Reset
// ============================================================================

#[test]
fn test_cursor_reset_returns_to_root() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);

    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);

    let node = cursor.node();
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 20);
}

#[test]
fn test_cursor_reset_multiple_times() {
    let tree = deep_tree();
    let mut cursor = TreeCursor::new(&tree);

    for _ in 0..3 {
        cursor.goto_first_child();
    }
    assert_eq!(cursor.depth(), 3);

    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);

    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);

    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_reset_can_navigate_again() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    cursor.reset(&tree);

    // Should be able to navigate again
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn test_cursor_reset_between_different_trees() {
    let tree1 = simple_two_child_tree();
    let tree2 = wide_tree();

    let mut cursor = TreeCursor::new(&tree1);
    cursor.goto_first_child();

    cursor.reset(&tree2);
    // Should now be at root of tree2
    assert_eq!(cursor.depth(), 0);
    let node = cursor.node();
    assert_eq!(node.child_count(), 5); // tree2 has 5 children
}

// ============================================================================
// Tests: 33-35 Full Tree Traversal (Depth-First)
// ============================================================================

#[test]
fn test_full_depth_first_traversal_simple() {
    let tree = simple_two_child_tree();
    let mut cursor = TreeCursor::new(&tree);

    let mut visited_symbols = vec![];

    // Visit root
    visited_symbols.push(cursor.node().kind_id());

    // Visit first child
    cursor.goto_first_child();
    visited_symbols.push(cursor.node().kind_id());

    // Try to go deeper
    assert!(!cursor.goto_first_child());

    // Try to go to sibling
    cursor.goto_parent();
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    visited_symbols.push(cursor.node().kind_id());

    // Should have visited: root(0), child1(1), child2(2)
    assert_eq!(visited_symbols, vec![0, 1, 2]);
}

#[test]
fn test_full_depth_first_traversal_nested() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);

    let mut visited = vec![];

    // Visit root
    visited.push(cursor.node().kind_id());

    // Visit first child (1)
    cursor.goto_first_child();
    visited.push(cursor.node().kind_id());

    // Visit grandchild (3)
    cursor.goto_first_child();
    visited.push(cursor.node().kind_id());

    // Grandchild has no siblings, so go back to parent
    assert!(!cursor.goto_next_sibling());
    cursor.goto_parent();

    // Parent (child 1) HAS a sibling (child 2), so we CAN move to it
    assert!(cursor.goto_next_sibling());
    visited.push(cursor.node().kind_id());

    assert_eq!(visited, vec![0, 1, 3, 2]);
}

#[test]
fn test_full_depth_first_traversal_wide() {
    let tree = wide_tree();
    let mut cursor = TreeCursor::new(&tree);

    let mut visited = vec![];
    visited.push(cursor.node().kind_id());

    // Visit all children
    if cursor.goto_first_child() {
        loop {
            visited.push(cursor.node().kind_id());
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    // Should have visited: root(0), children 1,2,3,4,5
    assert_eq!(visited, vec![0, 1, 2, 3, 4, 5]);
}

// ============================================================================
// Tests: 36+ Mixed Navigation Patterns and Edge Cases
// ============================================================================

#[test]
fn test_navigation_pattern_down_and_up() {
    let tree = deep_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Go down 3 levels
    for _ in 0..3 {
        assert!(cursor.goto_first_child());
    }
    let max_depth = cursor.depth();
    assert_eq!(max_depth, 3);

    // Go back up all the way
    for _ in 0..3 {
        assert!(cursor.goto_parent());
    }

    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_navigation_sibling_then_child() {
    let tree = complex_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    let node1 = cursor.node();
    let node1_start = node1.start_byte();

    cursor.goto_next_sibling();
    let node2 = cursor.node();
    let node2_start = node2.start_byte();

    assert!(node2_start >= node1_start);

    // Can navigate into second sibling
    if node2.child_count() > 0 {
        assert!(cursor.goto_first_child());
    }
}

#[test]
fn test_node_byte_ranges_consistent_during_traversal() {
    let tree = complex_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Root node
    let root = cursor.node();
    let root_start = root.start_byte();
    let root_end = root.end_byte();
    assert!(root_start <= root_end);

    // Explore children
    if cursor.goto_first_child() {
        let child1 = cursor.node();
        assert!(child1.start_byte() >= root_start);
        assert!(child1.end_byte() <= root_end);

        if cursor.goto_next_sibling() {
            let child2 = cursor.node();
            assert!(child2.start_byte() >= root_start);
            assert!(child2.end_byte() <= root_end);
        }
    }
}

#[test]
fn test_cursor_child_count_matches_navigation() {
    let tree = wide_tree();
    let mut cursor = TreeCursor::new(&tree);

    let node = cursor.node();
    let child_count = node.child_count();
    assert_eq!(child_count, 5);

    // Count children via navigation
    let mut actual_count = 0;
    if cursor.goto_first_child() {
        actual_count = 1;
        while cursor.goto_next_sibling() {
            actual_count += 1;
        }
    }

    assert_eq!(actual_count, child_count);
}

#[test]
fn test_traverse_all_nodes_in_complex_tree() {
    let tree = complex_tree();
    let mut cursor = TreeCursor::new(&tree);

    let mut node_count = 0;

    // Simple traversal: visit root and all first-level children
    node_count += 1;
    if cursor.goto_first_child() {
        loop {
            node_count += 1;

            // Try to go deeper
            if cursor.goto_first_child() {
                loop {
                    node_count += 1;
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    // Complex tree should have multiple nodes
    assert!(node_count > 5);
}

#[test]
fn test_navigate_to_specific_node() {
    let tree = complex_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Navigate to second child's first grandchild
    cursor.goto_first_child(); // to first child
    cursor.goto_next_sibling(); // to second child
    assert_eq!(cursor.depth(), 1);

    if cursor.node().child_count() > 0 {
        cursor.goto_first_child();
        assert_eq!(cursor.depth(), 2);
    }
}

#[test]
fn test_multiple_cursor_independence() {
    let tree = complex_tree();
    let mut cursor1 = TreeCursor::new(&tree);
    let mut cursor2 = TreeCursor::new(&tree);

    // Cursor1 goes deep
    cursor1.goto_first_child();
    cursor1.goto_first_child();

    // Cursor2 navigates siblings
    cursor2.goto_first_child();
    cursor2.goto_next_sibling();

    // They should be at different positions
    assert_eq!(cursor1.depth(), 2);
    assert_eq!(cursor2.depth(), 1);
    assert_ne!(cursor1.node().kind_id(), cursor2.node().kind_id());
}

#[test]
fn test_cursor_operations_sequence() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Sequence: goto_first_child, goto_parent, goto_first_child, goto_next_sibling
    assert!(cursor.goto_first_child()); // depth 1
    cursor.goto_parent(); // depth 0

    assert!(cursor.goto_first_child()); // depth 1
    let node_at_depth_1 = cursor.node().kind_id();

    cursor.goto_next_sibling(); // Still depth 1
    let sibling_id = cursor.node().kind_id();

    assert_ne!(node_at_depth_1, sibling_id);
}

#[test]
fn test_cursor_with_very_deep_tree() {
    let tree = deep_tree();
    let mut cursor = TreeCursor::new(&tree);

    let mut depth = 0;
    while cursor.goto_first_child() {
        depth += 1;
        assert_eq!(cursor.depth(), depth);
    }

    assert!(depth > 0);

    // Navigate back up
    while cursor.goto_parent() {
        depth -= 1;
        assert_eq!(cursor.depth(), depth);
    }

    assert_eq!(depth, 0);
}

#[test]
fn test_node_method_after_navigation() {
    let tree = complex_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Get node properties after different navigation positions
    let root = cursor.node();
    let root_count = root.child_count();
    assert!(root_count > 0);

    cursor.goto_first_child();
    let child = cursor.node();
    assert!(child.start_byte() <= root.end_byte());
    assert!(child.end_byte() <= root.end_byte());

    cursor.goto_parent();
    let restored = cursor.node();
    assert_eq!(restored.child_count(), root_count);
}

#[test]
fn test_alternating_first_child_and_sibling() {
    let tree = complex_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Go to first child
    cursor.goto_first_child();

    // Repeatedly: try first child, then sibling
    let mut iteration = 0;
    loop {
        iteration += 1;

        // Can we go deeper?
        if cursor.goto_first_child() {
            // Go back and try sibling instead
            cursor.goto_parent();
        }

        // Try to go to sibling
        if !cursor.goto_next_sibling() {
            // No more siblings
            cursor.goto_parent();
            break;
        }

        if iteration > 20 {
            break; // Safety limit
        }
    }

    // Should have explored the tree structure
    assert!(iteration > 0);
}

#[test]
fn test_cursor_node_byte_consistency() {
    let tree = wide_tree();
    let mut cursor = TreeCursor::new(&tree);

    cursor.goto_first_child();
    loop {
        let node = cursor.node();
        let start = node.start_byte();
        let end = node.end_byte();

        // Byte ranges should always be valid
        assert!(start <= end);

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}
