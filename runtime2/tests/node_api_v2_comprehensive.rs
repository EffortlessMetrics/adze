//! Comprehensive tests for the Node API v2 in adze-runtime (runtime2).
//!
//! Test areas:
//!  1. Node basic properties from Tree::new_for_testing (10 tests)
//!  2. Child access patterns (8 tests)
//!  3. Named vs unnamed children (8 tests)
//!  4. Sibling navigation (8 tests)
//!  5. Node byte ranges (8 tests)
//!  6. Kind and kind_id consistency (5 tests)
//!  7. Error nodes (5 tests)
//!  8. Deep and wide tree node access (8 tests)
//!  9. Edge cases: leaf nodes, root properties (5 tests)

use adze_runtime::Tree;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

fn branch(symbol: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(symbol, start, end, children)
}

// ===========================================================================
// 1. Node basic properties from Tree::new_for_testing (10 tests)
// ===========================================================================

#[test]
fn basic_root_kind_id() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_node().kind_id(), 42);
}

#[test]
fn basic_root_start_byte() {
    let tree = Tree::new_for_testing(1, 5, 15, vec![]);
    assert_eq!(tree.root_node().start_byte(), 5);
}

#[test]
fn basic_root_end_byte() {
    let tree = Tree::new_for_testing(1, 5, 15, vec![]);
    assert_eq!(tree.root_node().end_byte(), 15);
}

#[test]
fn basic_root_child_count_empty() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn basic_root_child_count_with_children() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn basic_root_is_named() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    assert!(tree.root_node().is_named());
}

#[test]
fn basic_root_kind_without_language() {
    let tree = Tree::new_for_testing(99, 0, 5, vec![]);
    // Without a language set, kind() returns "unknown"
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn basic_stub_tree_kind_id() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn basic_stub_tree_start_byte() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().start_byte(), 0);
}

#[test]
fn basic_stub_tree_end_byte() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().end_byte(), 0);
}

// ===========================================================================
// 2. Child access patterns (8 tests)
// ===========================================================================

#[test]
fn child_access_first_child() {
    let tree = branch(
        1,
        0,
        30,
        vec![leaf(10, 0, 10), leaf(11, 10, 20), leaf(12, 20, 30)],
    );
    let first = tree.root_node().child(0).unwrap();
    assert_eq!(first.kind_id(), 10);
}

#[test]
fn child_access_last_child() {
    let tree = branch(
        1,
        0,
        30,
        vec![leaf(10, 0, 10), leaf(11, 10, 20), leaf(12, 20, 30)],
    );
    let last = tree.root_node().child(2).unwrap();
    assert_eq!(last.kind_id(), 12);
}

#[test]
fn child_access_middle_child() {
    let tree = branch(
        1,
        0,
        30,
        vec![leaf(10, 0, 10), leaf(11, 10, 20), leaf(12, 20, 30)],
    );
    let mid = tree.root_node().child(1).unwrap();
    assert_eq!(mid.kind_id(), 11);
}

#[test]
fn child_access_out_of_bounds_returns_none() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    assert!(tree.root_node().child(1).is_none());
}

#[test]
fn child_access_far_out_of_bounds() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    assert!(tree.root_node().child(100).is_none());
}

#[test]
fn child_access_on_leaf_returns_none() {
    let tree = leaf(1, 0, 10);
    assert!(tree.root_node().child(0).is_none());
}

#[test]
fn child_access_preserves_byte_ranges() {
    let tree = branch(1, 0, 20, vec![leaf(2, 3, 7), leaf(3, 10, 18)]);
    let c0 = tree.root_node().child(0).unwrap();
    let c1 = tree.root_node().child(1).unwrap();
    assert_eq!(c0.start_byte(), 3);
    assert_eq!(c0.end_byte(), 7);
    assert_eq!(c1.start_byte(), 10);
    assert_eq!(c1.end_byte(), 18);
}

#[test]
fn child_access_nested_grandchild() {
    let grandchild = leaf(100, 2, 4);
    let child = branch(50, 0, 10, vec![grandchild]);
    let tree = branch(1, 0, 20, vec![child]);

    let gc = tree.root_node().child(0).unwrap().child(0).unwrap();
    assert_eq!(gc.kind_id(), 100);
    assert_eq!(gc.start_byte(), 2);
    assert_eq!(gc.end_byte(), 4);
}

// ===========================================================================
// 3. Named vs unnamed children (8 tests)
// ===========================================================================

#[test]
fn named_child_count_equals_child_count_phase1() {
    let tree = branch(
        1,
        0,
        30,
        vec![leaf(2, 0, 10), leaf(3, 10, 20), leaf(4, 20, 30)],
    );
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn named_child_same_as_child_phase1() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10), leaf(3, 10, 20)]);
    let root = tree.root_node();
    let c = root.child(0).unwrap();
    let nc = root.named_child(0).unwrap();
    assert_eq!(c.kind_id(), nc.kind_id());
    assert_eq!(c.start_byte(), nc.start_byte());
}

#[test]
fn named_child_out_of_bounds_returns_none() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    assert!(tree.root_node().named_child(1).is_none());
}

#[test]
fn named_child_on_leaf_returns_none() {
    let tree = leaf(1, 0, 10);
    assert!(tree.root_node().named_child(0).is_none());
}

#[test]
fn named_child_count_zero_for_leaf() {
    let tree = leaf(1, 0, 10);
    assert_eq!(tree.root_node().named_child_count(), 0);
}

#[test]
fn named_child_count_stub() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().named_child_count(), 0);
}

#[test]
fn named_child_all_children_are_named_phase1() {
    let tree = branch(
        1,
        0,
        30,
        vec![leaf(2, 0, 10), leaf(3, 10, 20), leaf(4, 20, 30)],
    );
    let root = tree.root_node();
    for i in 0..root.named_child_count() {
        assert!(root.named_child(i).unwrap().is_named());
    }
}

#[test]
fn named_child_index_matches_child_index_phase1() {
    let tree = branch(1, 0, 20, vec![leaf(10, 0, 10), leaf(20, 10, 20)]);
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let c = root.child(i).unwrap();
        let nc = root.named_child(i).unwrap();
        assert_eq!(c.kind_id(), nc.kind_id());
    }
}

// ===========================================================================
// 4. Sibling navigation (8 tests)
// ===========================================================================

#[test]
fn next_sibling_returns_none_no_parent_links() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10), leaf(3, 10, 20)]);
    let first = tree.root_node().child(0).unwrap();
    assert!(first.next_sibling().is_none());
}

#[test]
fn prev_sibling_returns_none_no_parent_links() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10), leaf(3, 10, 20)]);
    let second = tree.root_node().child(1).unwrap();
    assert!(second.prev_sibling().is_none());
}

#[test]
fn next_named_sibling_returns_none() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10), leaf(3, 10, 20)]);
    let first = tree.root_node().child(0).unwrap();
    assert!(first.next_named_sibling().is_none());
}

#[test]
fn prev_named_sibling_returns_none() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10), leaf(3, 10, 20)]);
    let second = tree.root_node().child(1).unwrap();
    assert!(second.prev_named_sibling().is_none());
}

#[test]
fn root_next_sibling_returns_none() {
    let tree = leaf(1, 0, 10);
    assert!(tree.root_node().next_sibling().is_none());
}

#[test]
fn root_prev_sibling_returns_none() {
    let tree = leaf(1, 0, 10);
    assert!(tree.root_node().prev_sibling().is_none());
}

#[test]
fn stub_tree_next_sibling_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().next_sibling().is_none());
}

#[test]
fn stub_tree_prev_sibling_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().prev_sibling().is_none());
}

// ===========================================================================
// 5. Node byte ranges (8 tests)
// ===========================================================================

#[test]
fn byte_range_leaf_node() {
    let tree = leaf(1, 10, 25);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 10..25);
}

#[test]
fn byte_range_root_spans_children() {
    let tree = branch(1, 0, 50, vec![leaf(2, 0, 20), leaf(3, 20, 50)]);
    assert_eq!(tree.root_node().byte_range(), 0..50);
}

#[test]
fn byte_range_child_subset_of_parent() {
    let tree = branch(1, 0, 100, vec![leaf(2, 10, 40), leaf(3, 50, 90)]);
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    assert!(c0.start_byte() >= root.start_byte());
    assert!(c0.end_byte() <= root.end_byte());
}

#[test]
fn byte_range_zero_width() {
    let tree = leaf(1, 5, 5);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), root.end_byte());
    assert_eq!(root.byte_range(), 5..5);
}

#[test]
fn byte_range_large_offsets() {
    let tree = leaf(1, 1_000_000, 2_000_000);
    assert_eq!(tree.root_node().start_byte(), 1_000_000);
    assert_eq!(tree.root_node().end_byte(), 2_000_000);
}

#[test]
fn byte_range_stub() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().byte_range(), 0..0);
}

#[test]
fn byte_range_consistency_start_le_end() {
    let tree = branch(1, 5, 50, vec![leaf(2, 5, 25), leaf(3, 25, 50)]);
    let root = tree.root_node();
    assert!(root.start_byte() <= root.end_byte());
    for i in 0..root.child_count() {
        let c = root.child(i).unwrap();
        assert!(c.start_byte() <= c.end_byte());
    }
}

#[test]
fn byte_range_adjacent_children() {
    let tree = branch(
        1,
        0,
        30,
        vec![leaf(2, 0, 10), leaf(3, 10, 20), leaf(4, 20, 30)],
    );
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    let c1 = root.child(1).unwrap();
    let c2 = root.child(2).unwrap();
    assert_eq!(c0.end_byte(), c1.start_byte());
    assert_eq!(c1.end_byte(), c2.start_byte());
}

// ===========================================================================
// 6. Kind and kind_id consistency (5 tests)
// ===========================================================================

#[test]
fn kind_id_matches_symbol() {
    let tree = Tree::new_for_testing(255, 0, 10, vec![]);
    assert_eq!(tree.root_node().kind_id(), 255);
}

#[test]
fn kind_id_zero() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![]);
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn kind_id_max_u16() {
    let tree = Tree::new_for_testing(u16::MAX as u32, 0, 10, vec![]);
    assert_eq!(tree.root_node().kind_id(), u16::MAX);
}

#[test]
fn kind_id_child_preserves_symbol() {
    let tree = branch(1, 0, 20, vec![leaf(77, 0, 10), leaf(88, 10, 20)]);
    assert_eq!(tree.root_node().child(0).unwrap().kind_id(), 77);
    assert_eq!(tree.root_node().child(1).unwrap().kind_id(), 88);
}

#[test]
fn kind_returns_unknown_without_language() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    assert_eq!(tree.root_node().kind(), "unknown");
    assert_eq!(tree.root_node().child(0).unwrap().kind(), "unknown");
}

// ===========================================================================
// 7. Error nodes (5 tests)
// ===========================================================================

#[test]
fn is_error_returns_false_phase1() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    assert!(!tree.root_node().is_error());
}

#[test]
fn is_error_false_for_child() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10)]);
    assert!(!tree.root_node().child(0).unwrap().is_error());
}

#[test]
fn is_missing_returns_false_phase1() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn is_error_false_stub() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_error());
}

#[test]
fn is_missing_false_stub() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_missing());
}

// ===========================================================================
// 8. Deep and wide tree node access (8 tests)
// ===========================================================================

#[test]
fn deep_tree_four_levels() {
    let l3 = leaf(4, 3, 4);
    let l2 = branch(3, 2, 5, vec![l3]);
    let l1 = branch(2, 1, 6, vec![l2]);
    let tree = branch(1, 0, 7, vec![l1]);

    let root = tree.root_node();
    let depth1 = root.child(0).unwrap();
    let depth2 = depth1.child(0).unwrap();
    let depth3 = depth2.child(0).unwrap();

    assert_eq!(root.kind_id(), 1);
    assert_eq!(depth1.kind_id(), 2);
    assert_eq!(depth2.kind_id(), 3);
    assert_eq!(depth3.kind_id(), 4);
}

#[test]
fn deep_tree_leaf_has_no_children() {
    let l3 = leaf(4, 3, 4);
    let l2 = branch(3, 2, 5, vec![l3]);
    let l1 = branch(2, 1, 6, vec![l2]);
    let tree = branch(1, 0, 7, vec![l1]);

    let deepest = tree
        .root_node()
        .child(0)
        .unwrap()
        .child(0)
        .unwrap()
        .child(0)
        .unwrap();
    assert_eq!(deepest.child_count(), 0);
    assert!(deepest.child(0).is_none());
}

#[test]
fn wide_tree_ten_children() {
    let children: Vec<Tree> = (0..10)
        .map(|i| leaf(i + 10, i as usize * 5, (i as usize + 1) * 5))
        .collect();
    let tree = branch(1, 0, 50, children);

    assert_eq!(tree.root_node().child_count(), 10);
    for i in 0..10 {
        let c = tree.root_node().child(i).unwrap();
        assert_eq!(c.kind_id(), (i as u16) + 10);
    }
}

#[test]
fn wide_tree_last_child_kind() {
    let children: Vec<Tree> = (0..5)
        .map(|i| leaf(100 + i, i as usize * 2, (i as usize + 1) * 2))
        .collect();
    let tree = branch(1, 0, 10, children);
    let last = tree.root_node().child(4).unwrap();
    assert_eq!(last.kind_id(), 104);
}

#[test]
fn deep_wide_mixed() {
    let gc0 = leaf(30, 0, 2);
    let gc1 = leaf(31, 2, 4);
    let c0 = branch(20, 0, 4, vec![gc0, gc1]);
    let c1 = leaf(21, 4, 8);
    let c2 = branch(22, 8, 14, vec![leaf(32, 8, 11), leaf(33, 11, 14)]);
    let tree = branch(1, 0, 14, vec![c0, c1, c2]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    assert_eq!(root.child(0).unwrap().child_count(), 2);
    assert_eq!(root.child(1).unwrap().child_count(), 0);
    assert_eq!(root.child(2).unwrap().child_count(), 2);
    assert_eq!(root.child(2).unwrap().child(1).unwrap().kind_id(), 33);
}

#[test]
fn deep_tree_six_levels() {
    let mut current = leaf(60, 5, 6);
    for depth in (1..6).rev() {
        current = branch(depth as u32, depth, 7 + depth, vec![current]);
    }

    let root = current.root_node();
    let mut node = root;
    for expected_kind in 1u16..=5 {
        assert_eq!(node.kind_id(), expected_kind);
        node = node.child(0).unwrap();
    }
    assert_eq!(node.kind_id(), 60);
    assert_eq!(node.child_count(), 0);
}

#[test]
fn wide_tree_out_of_bounds() {
    let children: Vec<Tree> = (0..3)
        .map(|i| leaf(i + 10, i as usize * 4, (i as usize + 1) * 4))
        .collect();
    let tree = branch(1, 0, 12, children);
    assert!(tree.root_node().child(3).is_none());
    assert!(tree.root_node().child(usize::MAX).is_none());
}

#[test]
fn deep_tree_byte_ranges_nest() {
    let gc = leaf(3, 5, 8);
    let child = branch(2, 3, 10, vec![gc]);
    let tree = branch(1, 0, 15, vec![child]);

    let root = tree.root_node();
    let c = root.child(0).unwrap();
    let g = c.child(0).unwrap();

    assert!(c.start_byte() >= root.start_byte());
    assert!(c.end_byte() <= root.end_byte());
    assert!(g.start_byte() >= c.start_byte());
    assert!(g.end_byte() <= c.end_byte());
}

// ===========================================================================
// 9. Edge cases: leaf nodes, root properties (5 tests)
// ===========================================================================

#[test]
fn leaf_node_child_count_is_zero() {
    let tree = leaf(1, 0, 10);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn root_parent_returns_none() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10)]);
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn child_parent_returns_none_no_parent_links() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10)]);
    let child = tree.root_node().child(0).unwrap();
    assert!(child.parent().is_none());
}

#[test]
fn child_by_field_name_returns_none() {
    let tree = branch(1, 0, 20, vec![leaf(2, 0, 10)]);
    assert!(tree.root_node().child_by_field_name("name").is_none());
}

#[test]
fn node_debug_format_contains_kind_and_range() {
    let tree = Tree::new_for_testing(7, 10, 30, vec![]);
    let dbg = format!("{:?}", tree.root_node());
    assert!(dbg.contains("Node"));
    assert!(dbg.contains("10..30"));
}
