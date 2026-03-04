#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for Node traversal in adze-runtime.
//!
//! Covers child access, named child filtering, out-of-bounds access,
//! deep traversal, sibling navigation, and mixed named/unnamed children.

use adze_runtime::Tree;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Leaf node helper.
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Build a simple tree:
///   root(0) [0..20]
///     ├── a(1) [0..5]
///     ├── b(2) [5..10]
///     └── c(3) [10..20]
fn simple_tree() -> Tree {
    Tree::new_for_testing(
        0,
        0,
        20,
        vec![leaf(1, 0, 5), leaf(2, 5, 10), leaf(3, 10, 20)],
    )
}

/// Build a deeper tree:
///   root(0) [0..30]
///     ├── a(1) [0..15]
///     │   ├── d(4) [0..7]
///     │   └── e(5) [7..15]
///     └── b(2) [15..30]
///         └── f(6) [15..30]
fn deep_tree() -> Tree {
    let a = Tree::new_for_testing(1, 0, 15, vec![leaf(4, 0, 7), leaf(5, 7, 15)]);
    let b = Tree::new_for_testing(2, 15, 30, vec![leaf(6, 15, 30)]);
    Tree::new_for_testing(0, 0, 30, vec![a, b])
}

// ---------------------------------------------------------------------------
// child(0) on root
// ---------------------------------------------------------------------------

#[test]
fn child_zero_returns_first_child() {
    let tree = simple_tree();
    let root = tree.root_node();
    let first = root.child(0).expect("child(0) should exist");
    assert_eq!(first.kind_id(), 1);
    assert_eq!(first.start_byte(), 0);
    assert_eq!(first.end_byte(), 5);
}

#[test]
fn child_zero_on_leaf_returns_none() {
    let tree = leaf(42, 0, 10);
    let root = tree.root_node();
    assert!(root.child(0).is_none());
}

#[test]
fn child_zero_on_stub_returns_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.child(0).is_none());
}

// ---------------------------------------------------------------------------
// child_count matches children
// ---------------------------------------------------------------------------

#[test]
fn child_count_matches_actual_children() {
    let tree = simple_tree();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    for i in 0..root.child_count() {
        assert!(root.child(i).is_some(), "child({i}) should exist");
    }
}

#[test]
fn child_count_zero_for_leaf() {
    let tree = leaf(1, 0, 5);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn child_count_one() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    assert_eq!(tree.root_node().child_count(), 1);
}

#[test]
fn child_count_many() {
    let children: Vec<Tree> = (0..10)
        .map(|i| leaf(i + 1, i as usize, (i + 1) as usize))
        .collect();
    let tree = Tree::new_for_testing(0, 0, 10, children);
    assert_eq!(tree.root_node().child_count(), 10);
}

// ---------------------------------------------------------------------------
// named_child filtering (Phase 1: same as child)
// ---------------------------------------------------------------------------

#[test]
fn named_child_count_equals_child_count() {
    let tree = simple_tree();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn named_child_returns_same_as_child() {
    let tree = simple_tree();
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let c = root.child(i).unwrap();
        let nc = root.named_child(i).unwrap();
        assert_eq!(c.kind_id(), nc.kind_id());
        assert_eq!(c.byte_range(), nc.byte_range());
    }
}

#[test]
fn named_child_out_of_bounds_returns_none() {
    let tree = simple_tree();
    assert!(tree.root_node().named_child(100).is_none());
}

#[test]
fn named_child_zero_for_leaf() {
    let tree = leaf(7, 0, 3);
    assert_eq!(tree.root_node().named_child_count(), 0);
    assert!(tree.root_node().named_child(0).is_none());
}

// ---------------------------------------------------------------------------
// Out-of-bounds child access
// ---------------------------------------------------------------------------

#[test]
fn child_out_of_bounds_returns_none() {
    let tree = simple_tree();
    let root = tree.root_node();
    assert!(root.child(3).is_none());
    assert!(root.child(100).is_none());
    assert!(root.child(usize::MAX).is_none());
}

#[test]
fn child_out_of_bounds_on_empty_tree() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(1).is_none());
}

#[test]
fn child_boundary_index() {
    let tree = simple_tree();
    let root = tree.root_node();
    // Last valid index
    assert!(root.child(2).is_some());
    // One past last
    assert!(root.child(3).is_none());
}

// ---------------------------------------------------------------------------
// Deep traversal (child of child)
// ---------------------------------------------------------------------------

#[test]
fn deep_traversal_two_levels() {
    let tree = deep_tree();
    let root = tree.root_node();
    let a = root.child(0).expect("first child");
    assert_eq!(a.kind_id(), 1);
    let d = a.child(0).expect("grandchild d");
    assert_eq!(d.kind_id(), 4);
    assert_eq!(d.start_byte(), 0);
    assert_eq!(d.end_byte(), 7);
}

#[test]
fn deep_traversal_second_branch() {
    let tree = deep_tree();
    let root = tree.root_node();
    let b = root.child(1).expect("second child");
    let f = b.child(0).expect("grandchild f");
    assert_eq!(f.kind_id(), 6);
    assert_eq!(f.byte_range(), 15..30);
}

#[test]
fn deep_traversal_grandchild_is_leaf() {
    let tree = deep_tree();
    let d = tree.root_node().child(0).unwrap().child(0).unwrap();
    assert_eq!(d.child_count(), 0);
    assert!(d.child(0).is_none());
}

#[test]
fn three_level_deep_traversal() {
    let inner = Tree::new_for_testing(3, 2, 4, vec![leaf(4, 2, 4)]);
    let mid = Tree::new_for_testing(2, 0, 6, vec![inner]);
    let tree = Tree::new_for_testing(1, 0, 6, vec![mid]);

    let root = tree.root_node();
    let level1 = root.child(0).expect("level 1");
    assert_eq!(level1.kind_id(), 2);
    let level2 = level1.child(0).expect("level 2");
    assert_eq!(level2.kind_id(), 3);
    let level3 = level2.child(0).expect("level 3");
    assert_eq!(level3.kind_id(), 4);
    assert!(level3.child(0).is_none());
}

// ---------------------------------------------------------------------------
// Sibling navigation (next_sibling / prev_sibling)
// ---------------------------------------------------------------------------

#[test]
fn next_sibling_returns_none() {
    let tree = simple_tree();
    let first = tree.root_node().child(0).unwrap();
    // Sibling links not stored — always None
    assert!(first.next_sibling().is_none());
}

#[test]
fn prev_sibling_returns_none() {
    let tree = simple_tree();
    let last = tree.root_node().child(2).unwrap();
    assert!(last.prev_sibling().is_none());
}

#[test]
fn next_named_sibling_returns_none() {
    let tree = simple_tree();
    let first = tree.root_node().child(0).unwrap();
    assert!(first.next_named_sibling().is_none());
}

#[test]
fn prev_named_sibling_returns_none() {
    let tree = simple_tree();
    let last = tree.root_node().child(2).unwrap();
    assert!(last.prev_named_sibling().is_none());
}

#[test]
fn root_sibling_returns_none() {
    let tree = simple_tree();
    let root = tree.root_node();
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
}

// ---------------------------------------------------------------------------
// Mixed named/unnamed children (Phase 1: all named)
// ---------------------------------------------------------------------------

#[test]
fn all_children_are_named_in_phase1() {
    let tree = simple_tree();
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let c = root.child(i).unwrap();
        assert!(c.is_named(), "child({i}) should be named in Phase 1");
    }
}

#[test]
fn is_named_true_for_leaf() {
    let tree = leaf(99, 0, 1);
    assert!(tree.root_node().is_named());
}

#[test]
fn is_named_true_for_deep_node() {
    let tree = deep_tree();
    let grandchild = tree.root_node().child(0).unwrap().child(1).unwrap();
    assert!(grandchild.is_named());
}

// ---------------------------------------------------------------------------
// Additional traversal and metadata tests
// ---------------------------------------------------------------------------

#[test]
fn kind_id_propagates_through_children() {
    let tree = simple_tree();
    let root = tree.root_node();
    let expected_ids: Vec<u16> = vec![1, 2, 3];
    for i in 0..root.child_count() {
        assert_eq!(root.child(i).unwrap().kind_id(), expected_ids[i]);
    }
}

#[test]
fn byte_ranges_preserved_in_children() {
    let tree = simple_tree();
    let root = tree.root_node();
    let expected_ranges = [0..5, 5..10, 10..20];
    for i in 0..root.child_count() {
        assert_eq!(root.child(i).unwrap().byte_range(), expected_ranges[i]);
    }
}

#[test]
fn root_byte_range_spans_all_children() {
    let tree = simple_tree();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 20);
}

#[test]
fn parent_always_none() {
    let tree = deep_tree();
    assert!(tree.root_node().parent().is_none());
    assert!(tree.root_node().child(0).unwrap().parent().is_none());
}

#[test]
fn child_by_field_name_always_none() {
    let tree = simple_tree();
    let root = tree.root_node();
    assert!(root.child_by_field_name("body").is_none());
    assert!(root.child_by_field_name("").is_none());
}

#[test]
fn iterate_all_children_with_index() {
    let tree = simple_tree();
    let root = tree.root_node();
    let mut collected = Vec::new();
    for i in 0..root.child_count() {
        collected.push(root.child(i).unwrap().kind_id());
    }
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn deep_tree_child_counts() {
    let tree = deep_tree();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().child_count(), 2);
    assert_eq!(root.child(1).unwrap().child_count(), 1);
    assert_eq!(root.child(0).unwrap().child(0).unwrap().child_count(), 0);
}

#[test]
fn single_child_tree() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![leaf(1, 0, 10)]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    assert!(root.child(0).is_some());
    assert!(root.child(1).is_none());
    assert_eq!(root.child(0).unwrap().child_count(), 0);
}
