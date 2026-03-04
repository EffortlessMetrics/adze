//! Comprehensive TreeCursor traversal tests for adze-runtime (runtime2).
//!
//! Tests cursor movement, depth tracking, reset, and traversal patterns.

use adze_runtime::tree::{Tree, TreeCursor};

// ============================================================================
// Helpers
// ============================================================================

fn leaf(sym: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(sym, start, end, vec![])
}

fn branch(sym: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(sym, start, end, children)
}

// ============================================================================
// Tests: Basic cursor creation
// ============================================================================

#[test]
fn cursor_on_stub_tree() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    let node = cursor.node();
    assert_eq!(node.child_count(), 0);
}

#[test]
fn cursor_on_leaf_tree() {
    let tree = leaf(1, 0, 5);
    let cursor = TreeCursor::new(&tree);
    let node = cursor.node();
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 5);
}

#[test]
fn cursor_initial_depth_is_zero() {
    let tree = leaf(1, 0, 5);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// Tests: goto_first_child
// ============================================================================

#[test]
fn goto_first_child_on_leaf_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn goto_first_child_on_branch_returns_true() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
}

#[test]
fn goto_first_child_moves_to_first() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 5);
}

#[test]
fn goto_first_child_increases_depth() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
}

// ============================================================================
// Tests: goto_next_sibling
// ============================================================================

#[test]
fn goto_next_sibling_on_root_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn goto_next_sibling_moves_right() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().start_byte(), 5);
    assert_eq!(cursor.node().end_byte(), 10);
}

#[test]
fn goto_next_sibling_last_child_returns_false() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn goto_next_sibling_preserves_depth() {
    let tree = branch(
        1,
        0,
        15,
        vec![leaf(2, 0, 5), leaf(3, 5, 10), leaf(4, 10, 15)],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);
}

// ============================================================================
// Tests: goto_parent
// ============================================================================

#[test]
fn goto_parent_from_root_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn goto_parent_from_child_returns_true() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
}

#[test]
fn goto_parent_returns_to_root() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 10);
}

#[test]
fn goto_parent_from_deep_child() {
    let tree = branch(1, 0, 20, vec![branch(2, 0, 10, vec![leaf(3, 0, 5)])]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // depth 1
    cursor.goto_first_child(); // depth 2
    assert_eq!(cursor.depth(), 2);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// Tests: Full traversal
// ============================================================================

#[test]
fn preorder_traversal_visits_all_nodes() {
    let tree = branch(
        1,
        0,
        15,
        vec![leaf(2, 0, 5), leaf(3, 5, 10), leaf(4, 10, 15)],
    );
    let mut cursor = TreeCursor::new(&tree);
    let mut count = 0;
    let mut visit = true;

    // Preorder traversal
    loop {
        if visit {
            count += 1;
        }
        if visit && cursor.goto_first_child() {
            visit = true;
            continue;
        }
        if cursor.goto_next_sibling() {
            visit = true;
            continue;
        }
        if cursor.goto_parent() {
            visit = false;
            continue;
        }
        break;
    }
    assert_eq!(count, 4); // root + 3 children
}

#[test]
fn traversal_nested_tree() {
    let tree = branch(
        1,
        0,
        20,
        vec![
            branch(2, 0, 10, vec![leaf(3, 0, 5), leaf(4, 5, 10)]),
            leaf(5, 10, 20),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    let mut count = 0;
    let mut visit = true;

    loop {
        if visit {
            count += 1;
        }
        if visit && cursor.goto_first_child() {
            visit = true;
            continue;
        }
        if cursor.goto_next_sibling() {
            visit = true;
            continue;
        }
        if cursor.goto_parent() {
            visit = false;
            continue;
        }
        break;
    }
    assert_eq!(count, 5); // root + branch + 2 leaves + leaf
}

// ============================================================================
// Tests: Reset
// ============================================================================

#[test]
fn reset_returns_to_root() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
}

// ============================================================================
// Tests: Edge cases
// ============================================================================

#[test]
fn cursor_on_single_child_tree() {
    let tree = branch(1, 0, 5, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_wide_tree_traversal() {
    let children: Vec<Tree> = (0..50)
        .map(|i| leaf(i as u32 + 2, i * 2, (i + 1) * 2))
        .collect();
    let tree = branch(1, 0, 100, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut sibling_count = 1;
    while cursor.goto_next_sibling() {
        sibling_count += 1;
    }
    assert_eq!(sibling_count, 50);
}

#[test]
fn cursor_deep_tree_traversal() {
    let mut tree = leaf(10, 0, 5);
    for i in (1..10).rev() {
        tree = branch(i as u32, 0, 5, vec![tree]);
    }
    let mut cursor = TreeCursor::new(&tree);
    let mut depth = 0;
    while cursor.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, 9);
}
