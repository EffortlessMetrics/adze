#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for TreeCursor navigation in adze-runtime.

use adze_runtime::tree::{Tree, TreeCursor};

// ---------------------------------------------------------------------------
// Helpers — build trees from (symbol, start, end, children) descriptions
// ---------------------------------------------------------------------------

/// Minimal tree-building DSL using the crate-public `Tree::new_stub` plus
/// a manually-constructed `TreeNode` hierarchy.  Because `TreeNode` is
/// `pub(crate)`, we build trees through the public `Tree` API that already
/// exists (new_stub), plus a small helper that builds richer trees via a
/// dedicated builder module exposed for tests.
///
/// For these tests we only need symbol IDs and byte ranges visible through
/// `Node::kind_id()`, `Node::start_byte()`, `Node::end_byte()`, and
/// `Node::child_count()`.

// -- Single-node tree (stub) ------------------------------------------------

fn single_node_tree() -> Tree {
    Tree::new_stub()
}

// -- We cannot directly construct TreeNode from integration tests (pub(crate)),
//    but `Tree::new_stub()` gives us a single-node tree.  For richer trees we
//    parse using the test-utils language builder exposed in the crate.  However,
//    to keep tests self-contained we rely on a trick: `Tree` implements `Clone`,
//    so we can build trees by composing stubs.  Unfortunately, the fields on
//    Tree are pub(crate) so we cannot add children.
//
//    The simplest approach is to use the *unit-test* helper already present in
//    `tree.rs` (build_test_tree) but that is `#[cfg(test)]` and private.
//
//    The cleanest solution for integration tests: expose a `Tree::new_test`
//    builder in the public API behind a feature gate.  Since we are allowed to
//    make minimal changes to the source, let's add a small test-only builder
//    method.

// We'll use the test-helper provided by the crate (see below).

// ═══════════════════════════════════════════════════════════════════════════
// 1. CURSOR INITIAL POSITION
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn initial_position_is_root() {
    let tree = single_node_tree();
    let cursor = TreeCursor::new(&tree);
    let node = cursor.node();
    assert_eq!(node.kind_id(), 0);
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 0);
}

#[test]
fn initial_depth_is_zero() {
    let tree = single_node_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn initial_node_child_count_stub() {
    let tree = single_node_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().child_count(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. CURSOR ON SINGLE-NODE TREE
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn single_node_no_first_child() {
    let tree = single_node_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn single_node_no_next_sibling() {
    let tree = single_node_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn single_node_no_parent() {
    let tree = single_node_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn single_node_depth_stays_zero() {
    let tree = single_node_tree();
    let mut cursor = TreeCursor::new(&tree);
    // All navigation attempts should leave depth at 0
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 0);
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 0);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Richer tree construction via `Tree::new_for_testing`
// ═══════════════════════════════════════════════════════════════════════════

/// Build a tree with the following shape:
///
/// ```text
///         root (sym=0, 0..100)
///        /    |    \
///   c0(1)  c1(2)  c2(3)
///   0..30  30..60  60..100
///   / \
/// g0(4) g1(5)
/// 0..15  15..30
/// ```
fn rich_tree() -> Tree {
    Tree::new_for_testing(0, 0, 100, vec![
        Tree::new_for_testing(1, 0, 30, vec![
            Tree::new_for_testing(4, 0, 15, vec![]),
            Tree::new_for_testing(5, 15, 30, vec![]),
        ]),
        Tree::new_for_testing(2, 30, 60, vec![]),
        Tree::new_for_testing(3, 60, 100, vec![]),
    ])
}

/// Build a deep chain: root -> c0 -> c1 -> ... -> c(depth-1)
fn deep_tree(depth: usize) -> Tree {
    assert!(depth >= 1);
    // Build from the leaf up
    let mut current = Tree::new_for_testing(depth as u32, 0, 10, vec![]);
    for i in (1..depth).rev() {
        current = Tree::new_for_testing(i as u32, 0, 10, vec![current]);
    }
    // root is symbol 0
    Tree::new_for_testing(0, 0, 10, vec![current])
}

/// Build a wide tree: root with N children, each a leaf.
fn wide_tree(width: usize) -> Tree {
    let children: Vec<Tree> = (0..width)
        .map(|i| {
            let sym = (i + 1) as u32;
            let start = i * 10;
            let end = start + 10;
            Tree::new_for_testing(sym, start, end, vec![])
        })
        .collect();
    Tree::new_for_testing(0, 0, width * 10, children)
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. CHILD NAVIGATION
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_first_child_reaches_leftmost() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn goto_first_child_on_leaf_returns_false() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    // Go to c0's grandchild g0 (leaf)
    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_first_child()); // g0
    assert!(!cursor.goto_first_child()); // g0 is a leaf
}

#[test]
fn child_byte_ranges_are_correct() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // c0
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 30);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. SIBLING NAVIGATION
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_next_sibling_walks_all_children() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // c0

    let expected_syms: &[u16] = &[1, 2, 3];
    let mut collected = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        collected.push(cursor.node().kind_id());
    }
    assert_eq!(collected, expected_syms);
}

#[test]
fn goto_next_sibling_at_last_child_returns_false() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_next_sibling()); // c1
    assert!(cursor.goto_next_sibling()); // c2
    assert!(!cursor.goto_next_sibling()); // no more siblings
    // Cursor should still point at c2
    assert_eq!(cursor.node().kind_id(), 3);
}

#[test]
fn goto_next_sibling_on_root_returns_false() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
    // Still at root
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn sibling_byte_ranges_advance() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());

    let mut starts = vec![cursor.node().start_byte()];
    while cursor.goto_next_sibling() {
        starts.push(cursor.node().start_byte());
    }
    // Should be monotonically non-decreasing
    for i in 1..starts.len() {
        assert!(starts[i] >= starts[i - 1]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. PARENT NAVIGATION
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_parent_returns_to_parent_node() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0); // back to root
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn goto_parent_from_grandchild_returns_to_child() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_first_child()); // g0
    assert_eq!(cursor.depth(), 2);
    assert!(cursor.goto_parent()); // back to c0
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn goto_parent_at_root_returns_false() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn parent_after_sibling_returns_correct_parent() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_next_sibling()); // c1
    assert!(cursor.goto_parent()); // root
    assert_eq!(cursor.node().kind_id(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. CURSOR RESET
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn reset_returns_to_root() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    // Navigate deep
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);

    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn reset_allows_fresh_traversal() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);

    // First traversal: collect root's children
    assert!(cursor.goto_first_child());
    let mut first_pass = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        first_pass.push(cursor.node().kind_id());
    }

    // Reset and repeat
    cursor.reset(&tree);
    assert!(cursor.goto_first_child());
    let mut second_pass = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        second_pass.push(cursor.node().kind_id());
    }

    assert_eq!(first_pass, second_pass);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. DEEP TRAVERSAL PATTERNS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deep_traversal_reaches_leaf() {
    let tree = deep_tree(10);
    let mut cursor = TreeCursor::new(&tree);

    let mut depth = 0;
    while cursor.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, 10); // root + 10 levels
    assert_eq!(cursor.depth(), 10);
}

#[test]
fn deep_traversal_unwind_to_root() {
    let tree = deep_tree(10);
    let mut cursor = TreeCursor::new(&tree);

    // Go all the way down
    while cursor.goto_first_child() {}

    // Come all the way back up
    while cursor.goto_parent() {}
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn deep_traversal_symbol_ids_increase() {
    let tree = deep_tree(5);
    let mut cursor = TreeCursor::new(&tree);

    let mut syms = vec![cursor.node().kind_id()];
    while cursor.goto_first_child() {
        syms.push(cursor.node().kind_id());
    }
    // Symbols should be 0, 1, 2, 3, 4, 5
    let expected: Vec<u16> = (0..=5).collect();
    assert_eq!(syms, expected);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. WIDE TRAVERSAL PATTERNS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wide_tree_visits_all_children() {
    let tree = wide_tree(20);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());

    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 20);
}

#[test]
fn wide_tree_children_have_sequential_symbols() {
    let tree = wide_tree(10);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());

    let mut ids = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        ids.push(cursor.node().kind_id());
    }
    let expected: Vec<u16> = (1..=10).collect();
    assert_eq!(ids, expected);
}

#[test]
fn wide_tree_none_have_children() {
    let tree = wide_tree(5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());

    loop {
        assert_eq!(cursor.node().child_count(), 0);
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. NAVIGATION BEYOND BOUNDARIES
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn repeated_parent_at_root_is_idempotent() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..5 {
        assert!(!cursor.goto_parent());
    }
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn repeated_sibling_past_end_is_idempotent() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    // Move past end
    while cursor.goto_next_sibling() {}
    let last_sym = cursor.node().kind_id();
    // Repeated calls should not change position
    for _ in 0..5 {
        assert!(!cursor.goto_next_sibling());
        assert_eq!(cursor.node().kind_id(), last_sym);
    }
}

#[test]
fn first_child_on_leaf_repeated_is_idempotent() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_first_child()); // g0 (leaf)
    let leaf_sym = cursor.node().kind_id();
    for _ in 0..5 {
        assert!(!cursor.goto_first_child());
        assert_eq!(cursor.node().kind_id(), leaf_sym);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. MIXED / COMBINED NAVIGATION
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn child_sibling_parent_roundtrip() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);

    // root -> c0 -> sibling c1 -> parent (root)
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn depth_first_preorder_traversal() {
    // Perform a full DFS pre-order and collect symbol IDs.
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    let mut visited = Vec::new();

    // Iterative DFS using cursor
    loop {
        visited.push(cursor.node().kind_id());

        // Try going deeper
        if cursor.goto_first_child() {
            continue;
        }
        // Try going to sibling
        if cursor.goto_next_sibling() {
            continue;
        }
        // Backtrack until we find a sibling or reach root
        loop {
            if !cursor.goto_parent() {
                // Back at root with nowhere to go
                // Expected order: root(0), c0(1), g0(4), g1(5), c1(2), c2(3)
                assert_eq!(visited, vec![0, 1, 4, 5, 2, 3]);
                return;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[test]
fn zigzag_navigation() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);

    // Down to g0, up, down again
    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_first_child()); // g0
    assert_eq!(cursor.node().kind_id(), 4);
    assert!(cursor.goto_parent()); // c0
    assert!(cursor.goto_first_child()); // g0 again
    assert_eq!(cursor.node().kind_id(), 4);
}

#[test]
fn navigate_to_second_grandchild() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);

    assert!(cursor.goto_first_child()); // c0
    assert!(cursor.goto_first_child()); // g0
    assert!(cursor.goto_next_sibling()); // g1
    assert_eq!(cursor.node().kind_id(), 5);
    assert_eq!(cursor.node().start_byte(), 15);
    assert_eq!(cursor.node().end_byte(), 30);
}

#[test]
fn depth_tracks_correctly_through_mixed_navigation() {
    let tree = rich_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);

    assert!(cursor.goto_first_child()); // depth 1
    assert_eq!(cursor.depth(), 1);

    assert!(cursor.goto_first_child()); // depth 2
    assert_eq!(cursor.depth(), 2);

    assert!(cursor.goto_next_sibling()); // still depth 2
    assert_eq!(cursor.depth(), 2);

    assert!(cursor.goto_parent()); // depth 1
    assert_eq!(cursor.depth(), 1);

    assert!(cursor.goto_next_sibling()); // still depth 1
    assert_eq!(cursor.depth(), 1);

    assert!(cursor.goto_parent()); // depth 0
    assert_eq!(cursor.depth(), 0);
}
