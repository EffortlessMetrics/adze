// Wave 131: Comprehensive property tests for runtime2 tree operations
use adze_runtime::tree::*;

// =====================================================================
// Tree construction tests
// =====================================================================

#[test]
fn create_leaf_node() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = t.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn create_tree_with_children() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let t = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    let root = t.root_node();
    assert_eq!(root.child_count(), 2);
}

#[test]
fn tree_child_access() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let t = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    let root = t.root_node();
    let child0 = root.child(0).expect("Should have child 0");
    let child1 = root.child(1).expect("Should have child 1");
    assert_eq!(child0.start_byte(), 0);
    assert_eq!(child0.end_byte(), 3);
    assert_eq!(child1.start_byte(), 3);
    assert_eq!(child1.end_byte(), 6);
}

#[test]
fn tree_child_out_of_bounds() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = t.root_node();
    assert!(root.child(0).is_none());
    assert!(root.child(99).is_none());
}

#[test]
fn deep_tree_construction() {
    let leaf = Tree::new_for_testing(4, 0, 1, vec![]);
    let mid = Tree::new_for_testing(3, 0, 1, vec![leaf]);
    let top = Tree::new_for_testing(2, 0, 1, vec![mid]);
    let root = Tree::new_for_testing(1, 0, 1, vec![top]);
    assert_eq!(root.root_node().child_count(), 1);
    let c = root.root_node().child(0).unwrap();
    assert_eq!(c.child_count(), 1);
}

#[test]
fn wide_tree_construction() {
    let children: Vec<Tree> = (0..10)
        .map(|i| Tree::new_for_testing(2, i * 10, (i + 1) * 10, vec![]))
        .collect();
    let t = Tree::new_for_testing(1, 0, 100, children);
    assert_eq!(t.root_node().child_count(), 10);
}

#[test]
fn zero_width_node() {
    let t = Tree::new_for_testing(1, 5, 5, vec![]);
    let root = t.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 5);
}

#[test]
fn large_byte_offsets() {
    let t = Tree::new_for_testing(1, 1_000_000, 2_000_000, vec![]);
    let root = t.root_node();
    assert_eq!(root.start_byte(), 1_000_000);
    assert_eq!(root.end_byte(), 2_000_000);
}

// =====================================================================
// TreeCursor tests
// =====================================================================

#[test]
fn cursor_starts_at_root() {
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    let cursor = TreeCursor::new(&t);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 10);
}

#[test]
fn cursor_goto_first_child() {
    let c = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![c]);
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 5);
}

#[test]
fn cursor_goto_first_child_leaf() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_goto_next_sibling() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let t = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().end_byte(), 3);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().start_byte(), 3);
    assert_eq!(cursor.node().end_byte(), 6);
}

#[test]
fn cursor_goto_next_sibling_none() {
    let c = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![c]);
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent() {
    let c = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![c]);
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_parent());
    // Back at root
    assert_eq!(cursor.node().child_count(), 1);
}

#[test]
fn cursor_goto_parent_at_root() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_depth_tracking() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 1, vec![leaf]);
    let t = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let mut cursor = TreeCursor::new(&t);
    assert_eq!(cursor.depth(), 0);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_full_traversal() {
    // Build tree:
    //       1
    //      / \
    //     2   3
    //    / \
    //   4   5
    let c4 = Tree::new_for_testing(4, 0, 1, vec![]);
    let c5 = Tree::new_for_testing(5, 1, 2, vec![]);
    let c2 = Tree::new_for_testing(2, 0, 2, vec![c4, c5]);
    let c3 = Tree::new_for_testing(3, 2, 3, vec![]);
    let t = Tree::new_for_testing(1, 0, 3, vec![c2, c3]);

    let mut cursor = TreeCursor::new(&t);
    let mut visited = vec![];
    loop {
        visited.push(cursor.node().start_byte());
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                // Traversal complete
                assert!(visited.len() >= 5, "Should visit all nodes");
                return;
            }
        }
    }
}

// =====================================================================
// Node property tests
// =====================================================================

#[test]
fn node_child_count_leaf() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(t.root_node().child_count(), 0);
}

#[test]
fn node_child_count_many() {
    let children: Vec<Tree> = (0..5)
        .map(|i| Tree::new_for_testing(2, i, i + 1, vec![]))
        .collect();
    let t = Tree::new_for_testing(1, 0, 5, children);
    assert_eq!(t.root_node().child_count(), 5);
}

// =====================================================================
// Clone / deep copy tests
// =====================================================================

#[test]
fn tree_clone_independence() {
    let c = Tree::new_for_testing(2, 0, 5, vec![]);
    let t1 = Tree::new_for_testing(1, 0, 5, vec![c]);
    let t2 = t1.clone();
    // Both should have same structure
    assert_eq!(t1.root_node().child_count(), t2.root_node().child_count());
    assert_eq!(t1.root_node().start_byte(), t2.root_node().start_byte());
}

// =====================================================================
// Stress tests
// =====================================================================

#[test]
fn very_deep_tree() {
    let mut t = Tree::new_for_testing(100, 0, 1, vec![]);
    for i in (1..100).rev() {
        t = Tree::new_for_testing(i, 0, 1, vec![t]);
    }
    let mut cursor = TreeCursor::new(&t);
    let mut depth = 0;
    while cursor.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, 99);
}

#[test]
fn very_wide_tree() {
    let children: Vec<Tree> = (0..100)
        .map(|i| Tree::new_for_testing(2, i, i + 1, vec![]))
        .collect();
    let t = Tree::new_for_testing(1, 0, 100, children);
    assert_eq!(t.root_node().child_count(), 100);

    // Cursor can traverse all siblings
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    let mut sibling_count = 1;
    while cursor.goto_next_sibling() {
        sibling_count += 1;
    }
    assert_eq!(sibling_count, 100);
}

#[test]
fn balanced_binary_tree() {
    fn build_balanced(depth: usize, sym: u32, start: usize) -> (Tree, usize) {
        if depth == 0 {
            return (
                Tree::new_for_testing(sym, start, start + 1, vec![]),
                start + 1,
            );
        }
        let (left, mid) = build_balanced(depth - 1, sym + 1, start);
        let (right, end) = build_balanced(depth - 1, sym + 1, mid);
        (
            Tree::new_for_testing(sym, start, end, vec![left, right]),
            end,
        )
    }

    let (t, total_end) = build_balanced(5, 1, 0);
    assert_eq!(total_end, 32); // 2^5 = 32 leaves
    assert_eq!(t.root_node().child_count(), 2);
}

// =====================================================================
// Multiple cursor instances
// =====================================================================

#[test]
fn two_cursors_independent() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let t = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);

    let mut cursor_a = TreeCursor::new(&t);
    let mut cursor_b = TreeCursor::new(&t);

    cursor_a.goto_first_child();
    assert_eq!(cursor_a.node().end_byte(), 3);
    // cursor_b is still at root
    assert_eq!(cursor_b.node().child_count(), 2);

    cursor_b.goto_first_child();
    cursor_b.goto_next_sibling();
    assert_eq!(cursor_b.node().start_byte(), 3);
    // cursor_a unchanged
    assert_eq!(cursor_a.node().end_byte(), 3);
}

// =====================================================================
// Edge cases
// =====================================================================

#[test]
fn single_node_tree() {
    let t = Tree::new_for_testing(42, 100, 200, vec![]);
    let root = t.root_node();
    assert_eq!(root.child_count(), 0);
    assert!(root.child(0).is_none());
}

#[test]
fn tree_with_zero_symbol() {
    let t = Tree::new_for_testing(0, 0, 1, vec![]);
    let root = t.root_node();
    assert_eq!(root.start_byte(), 0);
}

#[test]
fn tree_with_max_symbol() {
    let t = Tree::new_for_testing(u32::MAX, 0, 1, vec![]);
    let root = t.root_node();
    assert_eq!(root.end_byte(), 1);
}
