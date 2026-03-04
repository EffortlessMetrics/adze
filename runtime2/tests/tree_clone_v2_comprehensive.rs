//! Comprehensive tests for Tree::clone() behavior.
//!
//! Validates structural preservation, deep independence, various tree shapes,
//! byte range correctness, and sequential clone stability.

use adze_runtime::tree::Tree;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn leaf(sym: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(sym, start, end, vec![])
}

fn node(sym: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(sym, start, end, children)
}

/// Count every node reachable from `root_node()` via depth-first traversal.
fn count_nodes(tree: &Tree) -> usize {
    fn count_via_node(n: adze_runtime::node::Node<'_>) -> usize {
        let mut total = 1;
        for i in 0..n.child_count() {
            if let Some(child) = n.child(i) {
                total += count_via_node(child);
            }
        }
        total
    }
    count_via_node(tree.root_node())
}

/// Collect `(kind_id, start_byte, end_byte)` tuples in pre-order.
fn snapshot(tree: &Tree) -> Vec<(u16, usize, usize)> {
    fn walk(n: adze_runtime::node::Node<'_>, out: &mut Vec<(u16, usize, usize)>) {
        out.push((n.kind_id(), n.start_byte(), n.end_byte()));
        for i in 0..n.child_count() {
            if let Some(child) = n.child(i) {
                walk(child, out);
            }
        }
    }
    let mut v = Vec::new();
    walk(tree.root_node(), &mut v);
    v
}

/// Build a left-spine tree of given depth (each node has one child).
fn left_spine(depth: usize) -> Tree {
    let mut current = leaf(depth as u32, 0, 1);
    for d in (0..depth).rev() {
        current = node(d as u32, 0, (depth - d + 1) as usize, vec![current]);
    }
    current
}

/// Build a wide tree: root with `width` leaf children.
fn wide_tree(width: usize) -> Tree {
    let children: Vec<Tree> = (0..width)
        .map(|i| leaf((i + 1) as u32, i * 2, i * 2 + 2))
        .collect();
    node(0, 0, width * 2, children)
}

/// Build a balanced binary tree of given depth.
fn balanced_binary(depth: usize, sym_base: u32, start: usize) -> Tree {
    if depth == 0 {
        return leaf(sym_base, start, start + 1);
    }
    let left = balanced_binary(depth - 1, sym_base * 2, start);
    let left_end = left.root_node().end_byte();
    let right = balanced_binary(depth - 1, sym_base * 2 + 1, left_end);
    let right_end = right.root_node().end_byte();
    node(sym_base, start, right_end, vec![left, right])
}

// ── 1. Empty / stub trees ────────────────────────────────────────────────────

#[test]
fn clone_stub_tree() {
    let t = Tree::new_stub();
    let c = t.clone();
    assert_eq!(c.root_node().kind_id(), 0);
    assert_eq!(c.root_node().start_byte(), 0);
    assert_eq!(c.root_node().end_byte(), 0);
    assert_eq!(c.root_node().child_count(), 0);
}

#[test]
fn clone_stub_tree_snapshot_matches() {
    let t = Tree::new_stub();
    assert_eq!(snapshot(&t), snapshot(&t.clone()));
}

// ── 2. Single leaf ───────────────────────────────────────────────────────────

#[test]
fn clone_single_leaf_preserves_symbol() {
    let t = leaf(42, 5, 10);
    let c = t.clone();
    assert_eq!(c.root_node().kind_id(), 42);
}

#[test]
fn clone_single_leaf_preserves_byte_range() {
    let t = leaf(1, 100, 200);
    let c = t.clone();
    assert_eq!(c.root_node().start_byte(), 100);
    assert_eq!(c.root_node().end_byte(), 200);
}

#[test]
fn clone_single_leaf_child_count_zero() {
    let t = leaf(7, 0, 5);
    assert_eq!(t.clone().root_node().child_count(), 0);
}

// ── 3. Binary tree ──────────────────────────────────────────────────────────

#[test]
fn clone_binary_tree_preserves_structure() {
    let t = node(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let c = t.clone();
    assert_eq!(c.root_node().child_count(), 2);
    assert_eq!(c.root_node().child(0).unwrap().kind_id(), 1);
    assert_eq!(c.root_node().child(1).unwrap().kind_id(), 2);
}

#[test]
fn clone_binary_tree_preserves_byte_ranges() {
    let t = node(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let c = t.clone();
    let left = c.root_node().child(0).unwrap();
    let right = c.root_node().child(1).unwrap();
    assert_eq!(left.start_byte(), 0);
    assert_eq!(left.end_byte(), 5);
    assert_eq!(right.start_byte(), 5);
    assert_eq!(right.end_byte(), 10);
}

#[test]
fn clone_binary_tree_snapshot_matches() {
    let t = node(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    assert_eq!(snapshot(&t), snapshot(&t.clone()));
}

// ── 4. Deep independence ────────────────────────────────────────────────────

#[test]
fn clone_independence_root_kind() {
    let t = leaf(10, 0, 5);
    let c = t.clone();
    // After cloning, original is unchanged regardless of clone existence.
    assert_eq!(t.root_node().kind_id(), 10);
    assert_eq!(c.root_node().kind_id(), 10);
}

#[test]
fn clone_independence_child_byte_ranges() {
    let t = node(0, 0, 20, vec![leaf(1, 0, 10), leaf(2, 10, 20)]);
    let _c = t.clone();
    // Original tree child ranges intact.
    assert_eq!(t.root_node().child(0).unwrap().start_byte(), 0);
    assert_eq!(t.root_node().child(1).unwrap().end_byte(), 20);
}

#[test]
fn clone_independence_node_count() {
    let t = node(0, 0, 6, vec![leaf(1, 0, 2), leaf(2, 2, 4), leaf(3, 4, 6)]);
    let c = t.clone();
    assert_eq!(count_nodes(&t), count_nodes(&c));
}

#[test]
fn clone_drop_does_not_affect_original() {
    let t = node(0, 0, 10, vec![leaf(1, 0, 5)]);
    {
        let _c = t.clone();
        // clone drops here
    }
    assert_eq!(t.root_node().kind_id(), 0);
    assert_eq!(t.root_node().child(0).unwrap().kind_id(), 1);
}

#[test]
fn clone_both_live_simultaneously() {
    let t = node(0, 0, 8, vec![leaf(1, 0, 4), leaf(2, 4, 8)]);
    let c1 = t.clone();
    let c2 = t.clone();
    assert_eq!(snapshot(&t), snapshot(&c1));
    assert_eq!(snapshot(&t), snapshot(&c2));
}

// ── 5. Wide trees ───────────────────────────────────────────────────────────

#[test]
fn clone_wide_10_children() {
    let t = wide_tree(10);
    let c = t.clone();
    assert_eq!(c.root_node().child_count(), 10);
    assert_eq!(snapshot(&t), snapshot(&c));
}

#[test]
fn clone_wide_50_children() {
    let t = wide_tree(50);
    let c = t.clone();
    assert_eq!(c.root_node().child_count(), 50);
    assert_eq!(count_nodes(&t), count_nodes(&c));
}

#[test]
fn clone_wide_100_children_last_child() {
    let t = wide_tree(100);
    let c = t.clone();
    let last = c.root_node().child(99).unwrap();
    assert_eq!(last.kind_id(), 100);
    assert_eq!(last.start_byte(), 198);
    assert_eq!(last.end_byte(), 200);
}

// ── 6. Deep trees ───────────────────────────────────────────────────────────

#[test]
fn clone_deep_10() {
    let t = left_spine(10);
    let c = t.clone();
    assert_eq!(snapshot(&t), snapshot(&c));
}

#[test]
fn clone_deep_50() {
    let t = left_spine(50);
    let c = t.clone();
    assert_eq!(count_nodes(&t), 51);
    assert_eq!(count_nodes(&c), 51);
}

#[test]
fn clone_deep_100() {
    let t = left_spine(100);
    let c = t.clone();
    assert_eq!(snapshot(&t), snapshot(&c));
}

#[test]
fn clone_deep_200_preserves_leaf() {
    let t = left_spine(200);
    let c = t.clone();
    // Walk down to the leaf.
    let mut n = c.root_node();
    for _ in 0..200 {
        n = n.child(0).unwrap();
    }
    assert_eq!(n.kind_id(), 200);
    assert_eq!(n.child_count(), 0);
}

// ── 7. Balanced binary trees ────────────────────────────────────────────────

#[test]
fn clone_balanced_depth_3() {
    let t = balanced_binary(3, 1, 0);
    let c = t.clone();
    assert_eq!(count_nodes(&t), count_nodes(&c));
    assert_eq!(snapshot(&t), snapshot(&c));
}

#[test]
fn clone_balanced_depth_5() {
    let t = balanced_binary(5, 1, 0);
    let c = t.clone();
    // 2^6 - 1 = 63 nodes
    assert_eq!(count_nodes(&c), 63);
    assert_eq!(snapshot(&t), snapshot(&c));
}

#[test]
fn clone_balanced_depth_8_node_count() {
    let t = balanced_binary(8, 1, 0);
    let c = t.clone();
    assert_eq!(count_nodes(&c), 511);
}

// ── 8. Sequential clones ────────────────────────────────────────────────────

#[test]
fn sequential_clone_5_times() {
    let t = node(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let orig_snap = snapshot(&t);
    for _ in 0..5 {
        let c = t.clone();
        assert_eq!(snapshot(&c), orig_snap);
    }
}

#[test]
fn sequential_clone_chain() {
    // c1 = clone(t), c2 = clone(c1), …
    let t = node(0, 0, 10, vec![leaf(1, 0, 3), leaf(2, 3, 7), leaf(3, 7, 10)]);
    let orig_snap = snapshot(&t);
    let mut current = t.clone();
    for _ in 0..10 {
        current = current.clone();
        assert_eq!(snapshot(&current), orig_snap);
    }
}

#[test]
fn sequential_clone_all_independent() {
    let t = node(0, 0, 6, vec![leaf(1, 0, 3), leaf(2, 3, 6)]);
    let clones: Vec<Tree> = (0..20).map(|_| t.clone()).collect();
    let orig_snap = snapshot(&t);
    for c in &clones {
        assert_eq!(snapshot(c), orig_snap);
    }
}

// ── 9. Preserve child_count at every level ──────────────────────────────────

#[test]
fn clone_preserves_child_count_at_every_level() {
    let t = node(
        0,
        0,
        12,
        vec![
            node(1, 0, 6, vec![leaf(3, 0, 3), leaf(4, 3, 6)]),
            node(2, 6, 12, vec![leaf(5, 6, 9), leaf(6, 9, 12)]),
        ],
    );
    let c = t.clone();
    assert_eq!(c.root_node().child_count(), 2);
    assert_eq!(c.root_node().child(0).unwrap().child_count(), 2);
    assert_eq!(c.root_node().child(1).unwrap().child_count(), 2);
    assert_eq!(
        c.root_node()
            .child(0)
            .unwrap()
            .child(0)
            .unwrap()
            .child_count(),
        0
    );
}

// ── 10. Byte range edge cases ───────────────────────────────────────────────

#[test]
fn clone_preserves_zero_width_range() {
    let t = leaf(1, 5, 5);
    let c = t.clone();
    assert_eq!(c.root_node().start_byte(), 5);
    assert_eq!(c.root_node().end_byte(), 5);
}

#[test]
fn clone_preserves_large_byte_range() {
    let t = leaf(1, 0, usize::MAX / 2);
    let c = t.clone();
    assert_eq!(c.root_node().end_byte(), usize::MAX / 2);
}

#[test]
fn clone_preserves_nonzero_start() {
    let t = node(0, 50, 100, vec![leaf(1, 50, 75), leaf(2, 75, 100)]);
    let c = t.clone();
    assert_eq!(c.root_node().start_byte(), 50);
    assert_eq!(c.root_node().child(0).unwrap().start_byte(), 50);
    assert_eq!(c.root_node().child(1).unwrap().start_byte(), 75);
}

// ── 11. Symbol id preservation ──────────────────────────────────────────────

#[test]
fn clone_preserves_symbol_ids_u16_max() {
    // kind_id() returns u16, so test with u16::MAX-range value.
    let t = leaf(u16::MAX as u32, 0, 1);
    let c = t.clone();
    assert_eq!(c.root_node().kind_id(), u16::MAX);
}

#[test]
fn clone_preserves_distinct_symbol_ids_in_children() {
    let children: Vec<Tree> = (100u32..110).map(|s| leaf(s, 0, 1)).collect();
    let t = node(0, 0, 1, children);
    let c = t.clone();
    for i in 0..10 {
        assert_eq!(c.root_node().child(i).unwrap().kind_id(), (100 + i) as u16,);
    }
}

#[test]
fn clone_preserves_root_kind() {
    let t = node(999, 0, 5, vec![leaf(1, 0, 5)]);
    let c = t.clone();
    assert_eq!(c.root_kind(), 999);
}

// ── 12. Heterogeneous shapes ────────────────────────────────────────────────

#[test]
fn clone_unbalanced_tree() {
    // Left-heavy: root -> [node(1->[2,3]), leaf(4)]
    let t = node(
        0,
        0,
        12,
        vec![
            node(1, 0, 9, vec![leaf(2, 0, 4), leaf(3, 4, 9)]),
            leaf(4, 9, 12),
        ],
    );
    let c = t.clone();
    assert_eq!(snapshot(&t), snapshot(&c));
}

#[test]
fn clone_right_heavy_tree() {
    let t = node(
        0,
        0,
        20,
        vec![
            leaf(1, 0, 2),
            node(
                2,
                2,
                20,
                vec![
                    leaf(3, 2, 4),
                    node(4, 4, 20, vec![leaf(5, 4, 10), leaf(6, 10, 20)]),
                ],
            ),
        ],
    );
    let c = t.clone();
    assert_eq!(snapshot(&t), snapshot(&c));
    assert_eq!(count_nodes(&c), 7);
}

#[test]
fn clone_mixed_arity_tree() {
    let t = node(
        0,
        0,
        30,
        vec![
            leaf(1, 0, 5),
            node(2, 5, 15, vec![leaf(3, 5, 10), leaf(4, 10, 15)]),
            leaf(5, 15, 20),
            node(
                6,
                20,
                30,
                vec![leaf(7, 20, 23), leaf(8, 23, 26), leaf(9, 26, 30)],
            ),
        ],
    );
    let c = t.clone();
    assert_eq!(snapshot(&t), snapshot(&c));
}

// ── 13. root_node() equivalence ─────────────────────────────────────────────

#[test]
fn clone_root_node_kind_id_matches() {
    let t = node(55, 0, 10, vec![]);
    let c = t.clone();
    assert_eq!(t.root_node().kind_id(), c.root_node().kind_id());
}

#[test]
fn clone_root_node_byte_range_matches() {
    let t = node(0, 42, 99, vec![]);
    let c = t.clone();
    assert_eq!(t.root_node().byte_range(), c.root_node().byte_range());
}

#[test]
fn clone_root_node_child_count_matches() {
    let t = node(0, 0, 6, vec![leaf(1, 0, 2), leaf(2, 2, 4), leaf(3, 4, 6)]);
    let c = t.clone();
    assert_eq!(t.root_node().child_count(), c.root_node().child_count());
}

// ── 14. Multiple levels of children ─────────────────────────────────────────

#[test]
fn clone_3_levels_deep() {
    let t = node(
        0,
        0,
        8,
        vec![node(
            1,
            0,
            8,
            vec![node(2, 0, 8, vec![leaf(3, 0, 4), leaf(4, 4, 8)])],
        )],
    );
    let c = t.clone();
    assert_eq!(count_nodes(&c), 5);
    assert_eq!(snapshot(&t), snapshot(&c));
}

#[test]
fn clone_4_levels_deep_preserves_leaf_bytes() {
    let t = node(
        0,
        0,
        16,
        vec![node(
            1,
            0,
            16,
            vec![node(2, 0, 16, vec![node(3, 0, 16, vec![leaf(4, 7, 13)])])],
        )],
    );
    let c = t.clone();
    let deep = c
        .root_node()
        .child(0)
        .unwrap()
        .child(0)
        .unwrap()
        .child(0)
        .unwrap()
        .child(0)
        .unwrap();
    assert_eq!(deep.kind_id(), 4);
    assert_eq!(deep.start_byte(), 7);
    assert_eq!(deep.end_byte(), 13);
}

// ── 15. node_count structural check ─────────────────────────────────────────

#[test]
fn clone_node_count_single() {
    assert_eq!(count_nodes(&leaf(0, 0, 1).clone()), 1);
}

#[test]
fn clone_node_count_two() {
    let t = node(0, 0, 2, vec![leaf(1, 0, 2)]);
    assert_eq!(count_nodes(&t.clone()), 2);
}

#[test]
fn clone_node_count_wide_25() {
    let t = wide_tree(25);
    assert_eq!(count_nodes(&t.clone()), 26); // 1 root + 25 children
}

#[test]
fn clone_node_count_balanced_4() {
    let t = balanced_binary(4, 1, 0);
    // 2^5 - 1 = 31
    assert_eq!(count_nodes(&t.clone()), 31);
}

// ── 16. Snapshot idempotence ────────────────────────────────────────────────

#[test]
fn snapshot_clone_idempotent_leaf() {
    let t = leaf(7, 3, 9);
    assert_eq!(snapshot(&t), snapshot(&t.clone().clone()));
}

#[test]
fn snapshot_clone_idempotent_tree() {
    let t = node(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    assert_eq!(snapshot(&t), snapshot(&t.clone().clone().clone()));
}

// ── 17. language() after clone ──────────────────────────────────────────────

#[test]
fn clone_preserves_no_language() {
    let t = leaf(1, 0, 1);
    assert!(t.clone().language().is_none());
}

// ── 18. source_bytes after clone ────────────────────────────────────────────

#[test]
fn clone_stub_has_no_source() {
    let t = Tree::new_stub();
    assert!(t.clone().source_bytes().is_none());
}

#[test]
fn clone_testing_tree_has_no_source() {
    let t = leaf(1, 0, 5);
    assert!(t.clone().source_bytes().is_none());
}

// ── 19. Stress: many clones don't corrupt ───────────────────────────────────

#[test]
fn stress_100_clones_identical() {
    let t = node(
        0,
        0,
        20,
        vec![
            node(1, 0, 10, vec![leaf(3, 0, 5), leaf(4, 5, 10)]),
            node(2, 10, 20, vec![leaf(5, 10, 15), leaf(6, 15, 20)]),
        ],
    );
    let orig = snapshot(&t);
    for _ in 0..100 {
        assert_eq!(snapshot(&t.clone()), orig);
    }
}

#[test]
fn stress_clone_chain_50() {
    let t = left_spine(20);
    let orig = snapshot(&t);
    let mut cur = t.clone();
    for _ in 0..50 {
        cur = cur.clone();
    }
    assert_eq!(snapshot(&cur), orig);
}

// ── 20. Edge: root with one child ───────────────────────────────────────────

#[test]
fn clone_root_with_single_child() {
    let t = node(0, 0, 5, vec![leaf(1, 0, 5)]);
    let c = t.clone();
    assert_eq!(c.root_node().child_count(), 1);
    assert_eq!(c.root_node().child(0).unwrap().kind_id(), 1);
}

#[test]
fn clone_root_with_single_nested_child() {
    let t = node(0, 0, 5, vec![node(1, 0, 5, vec![leaf(2, 0, 5)])]);
    let c = t.clone();
    let grandchild = c.root_node().child(0).unwrap().child(0).unwrap();
    assert_eq!(grandchild.kind_id(), 2);
    assert_eq!(grandchild.start_byte(), 0);
    assert_eq!(grandchild.end_byte(), 5);
}

// ── 21. Verify is_error / is_missing / is_named from clone ──────────────────

#[test]
fn clone_leaf_is_not_error() {
    let t = leaf(1, 0, 5);
    assert!(!t.clone().root_node().is_error());
}

#[test]
fn clone_leaf_is_not_missing() {
    let t = leaf(1, 0, 5);
    assert!(!t.clone().root_node().is_missing());
}

// ── 22. Clone of clone of deep tree ─────────────────────────────────────────

#[test]
fn double_clone_deep_tree() {
    let t = left_spine(30);
    let c1 = t.clone();
    let c2 = c1.clone();
    assert_eq!(snapshot(&t), snapshot(&c2));
}

#[test]
fn triple_clone_balanced_tree() {
    let t = balanced_binary(4, 1, 0);
    let c = t.clone().clone().clone();
    assert_eq!(count_nodes(&c), 31);
    assert_eq!(snapshot(&t), snapshot(&c));
}

// ── 23. Clone preserves byte_range() method ─────────────────────────────────

#[test]
fn clone_byte_range_method() {
    let t = leaf(1, 10, 20);
    let c = t.clone();
    assert_eq!(c.root_node().byte_range(), 10..20);
}

#[test]
fn clone_child_byte_range_method() {
    let t = node(0, 0, 10, vec![leaf(1, 3, 7)]);
    let c = t.clone();
    assert_eq!(c.root_node().child(0).unwrap().byte_range(), 3..7);
}
