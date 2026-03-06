//! Deep comprehensive tests for the Node API in adze-runtime (runtime2).
//!
//! 60+ tests covering: stub properties, testing-tree properties, child navigation,
//! sibling navigation, parent navigation, named child filtering, field name lookup,
//! byte ranges/positions, error/missing flags, and deep tree traversal patterns.

use adze_runtime::node::Point;
use adze_runtime::tree::Tree;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

fn branch(symbol: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(symbol, start, end, children)
}

/// Build a sample tree:
///   root(0, 0..20)
///     ├─ child_a(1, 0..5)
///     ├─ child_b(2, 5..12)
///     │    ├─ grandchild_x(3, 5..8)
///     │    └─ grandchild_y(4, 8..12)
///     └─ child_c(5, 12..20)
fn sample_tree() -> Tree {
    branch(
        0,
        0,
        20,
        vec![
            leaf(1, 0, 5),
            branch(2, 5, 12, vec![leaf(3, 5, 8), leaf(4, 8, 12)]),
            leaf(5, 12, 20),
        ],
    )
}

// ===== 1. Node properties on stub tree =====

#[test]
fn stub_kind_id_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn stub_kind_is_unknown() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn stub_byte_range_empty() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn stub_child_count_zero() {
    assert_eq!(Tree::new_stub().root_node().child_count(), 0);
}

#[test]
fn stub_named_child_count_zero() {
    assert_eq!(Tree::new_stub().root_node().named_child_count(), 0);
}

#[test]
fn stub_is_named() {
    assert!(Tree::new_stub().root_node().is_named());
}

#[test]
fn stub_is_not_missing() {
    assert!(!Tree::new_stub().root_node().is_missing());
}

#[test]
fn stub_is_not_error() {
    assert!(!Tree::new_stub().root_node().is_error());
}

#[test]
fn stub_parent_is_none() {
    assert!(Tree::new_stub().root_node().parent().is_none());
}

#[test]
fn stub_child_out_of_bounds() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.child(0).is_none());
    assert!(root.child(usize::MAX).is_none());
}

// ===== 2. Node properties on testing trees =====

#[test]
fn testing_tree_kind_id() {
    let tree = leaf(42, 0, 10);
    assert_eq!(tree.root_node().kind_id(), 42);
}

#[test]
fn testing_tree_kind_without_language_is_unknown() {
    let tree = leaf(7, 0, 3);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn testing_tree_byte_range() {
    let tree = leaf(1, 10, 25);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 25);
    assert_eq!(root.byte_range(), 10..25);
}

#[test]
fn testing_tree_zero_width_node() {
    let tree = leaf(1, 5, 5);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), root.end_byte());
    assert_eq!(root.byte_range(), 5..5);
}

#[test]
fn testing_tree_child_count_matches() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 3), leaf(2, 3, 7), leaf(3, 7, 10)]);
    assert_eq!(tree.root_node().child_count(), 3);
}

#[test]
fn testing_tree_leaf_has_no_children() {
    let tree = leaf(1, 0, 5);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn testing_tree_root_kind_matches() {
    let tree = branch(99, 0, 100, vec![leaf(1, 0, 50)]);
    assert_eq!(tree.root_kind(), 99);
}

#[test]
fn testing_tree_is_named_always_true() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().is_named());
}

// ===== 3. Child navigation =====

#[test]
fn child_returns_correct_kind_ids() {
    let tree = sample_tree();
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().kind_id(), 1);
    assert_eq!(root.child(1).unwrap().kind_id(), 2);
    assert_eq!(root.child(2).unwrap().kind_id(), 5);
}

#[test]
fn child_returns_none_past_end() {
    let tree = sample_tree();
    assert!(tree.root_node().child(3).is_none());
}

#[test]
fn child_returns_none_for_large_index() {
    let tree = sample_tree();
    assert!(tree.root_node().child(999).is_none());
}

#[test]
fn child_byte_ranges_match() {
    let tree = sample_tree();
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    assert_eq!(c0.start_byte(), 0);
    assert_eq!(c0.end_byte(), 5);
    let c1 = root.child(1).unwrap();
    assert_eq!(c1.start_byte(), 5);
    assert_eq!(c1.end_byte(), 12);
}

#[test]
fn grandchild_navigation() {
    let tree = sample_tree();
    let mid = tree.root_node().child(1).unwrap();
    assert_eq!(mid.child_count(), 2);
    assert_eq!(mid.child(0).unwrap().kind_id(), 3);
    assert_eq!(mid.child(1).unwrap().kind_id(), 4);
}

#[test]
fn grandchild_byte_ranges() {
    let tree = sample_tree();
    let mid = tree.root_node().child(1).unwrap();
    let gx = mid.child(0).unwrap();
    let gy = mid.child(1).unwrap();
    assert_eq!(gx.byte_range(), 5..8);
    assert_eq!(gy.byte_range(), 8..12);
}

#[test]
fn leaf_child_returns_none() {
    let tree = sample_tree();
    let first_leaf = tree.root_node().child(0).unwrap();
    assert!(first_leaf.child(0).is_none());
    assert_eq!(first_leaf.child_count(), 0);
}

#[test]
fn iterate_all_children() {
    let tree = sample_tree();
    let root = tree.root_node();
    let ids: Vec<u16> = (0..root.child_count())
        .map(|i| root.child(i).unwrap().kind_id())
        .collect();
    assert_eq!(ids, vec![1, 2, 5]);
}

// ===== 4. Sibling navigation =====

#[test]
fn next_sibling_returns_none() {
    let tree = sample_tree();
    let root = tree.root_node();
    assert!(root.next_sibling().is_none());
}

#[test]
fn prev_sibling_returns_none() {
    let tree = sample_tree();
    let root = tree.root_node();
    assert!(root.prev_sibling().is_none());
}

#[test]
fn next_sibling_on_child_returns_none() {
    let tree = sample_tree();
    let child = tree.root_node().child(0).unwrap();
    assert!(child.next_sibling().is_none());
}

#[test]
fn prev_sibling_on_child_returns_none() {
    let tree = sample_tree();
    let child = tree.root_node().child(2).unwrap();
    assert!(child.prev_sibling().is_none());
}

// ===== 5. Parent navigation =====

#[test]
fn parent_returns_none_on_root() {
    let tree = sample_tree();
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn parent_returns_none_on_child() {
    let tree = sample_tree();
    let child = tree.root_node().child(0).unwrap();
    assert!(child.parent().is_none());
}

#[test]
fn parent_returns_none_on_grandchild() {
    let tree = sample_tree();
    let grandchild = tree.root_node().child(1).unwrap().child(0).unwrap();
    assert!(grandchild.parent().is_none());
}

// ===== 6. Named child filtering =====

#[test]
fn named_child_count_equals_child_count() {
    let tree = sample_tree();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn named_child_same_as_child() {
    let tree = sample_tree();
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let child = root.child(i).unwrap();
        let named = root.named_child(i).unwrap();
        assert_eq!(child.kind_id(), named.kind_id());
        assert_eq!(child.byte_range(), named.byte_range());
    }
}

#[test]
fn named_child_out_of_bounds() {
    let tree = sample_tree();
    assert!(tree.root_node().named_child(100).is_none());
}

#[test]
fn named_child_on_leaf() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().named_child(0).is_none());
    assert_eq!(tree.root_node().named_child_count(), 0);
}

// ===== 7. Field name child lookup =====

#[test]
fn child_by_field_name_returns_none() {
    let tree = sample_tree();
    assert!(tree.root_node().child_by_field_name("left").is_none());
}

#[test]
fn child_by_field_name_empty_string() {
    let tree = sample_tree();
    assert!(tree.root_node().child_by_field_name("").is_none());
}

#[test]
fn child_by_field_name_on_leaf() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().child_by_field_name("value").is_none());
}

#[test]
fn child_by_field_name_on_stub() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child_by_field_name("body").is_none());
}

// ===== 8. Byte ranges and positions =====

#[test]
fn position_start_is_zero() {
    let tree = leaf(1, 10, 20);
    let p = tree.root_node().start_position();
    assert_eq!(p, Point::new(0, 0));
}

#[test]
fn position_end_is_zero() {
    let tree = leaf(1, 10, 20);
    let p = tree.root_node().end_position();
    assert_eq!(p, Point::new(0, 0));
}

#[test]
fn byte_range_consistency() {
    let tree = leaf(1, 3, 7);
    let root = tree.root_node();
    let range = root.byte_range();
    assert_eq!(range.start, root.start_byte());
    assert_eq!(range.end, root.end_byte());
}

#[test]
fn large_byte_offsets() {
    let big = 1_000_000;
    let tree = leaf(1, big, big + 500);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), big);
    assert_eq!(root.end_byte(), big + 500);
}

#[test]
fn children_byte_ranges_within_parent() {
    let tree = sample_tree();
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let child = root.child(i).unwrap();
        assert!(child.start_byte() >= root.start_byte());
        assert!(child.end_byte() <= root.end_byte());
    }
}

#[test]
fn utf8_text_on_valid_source() {
    let tree = leaf(1, 0, 5);
    let source = b"hello world";
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "hello");
}

#[test]
fn utf8_text_on_child() {
    let tree = branch(0, 0, 11, vec![leaf(1, 0, 5), leaf(2, 6, 11)]);
    let source = b"hello world";
    let c0 = tree.root_node().child(0).unwrap();
    let c1 = tree.root_node().child(1).unwrap();
    assert_eq!(c0.utf8_text(source).unwrap(), "hello");
    assert_eq!(c1.utf8_text(source).unwrap(), "world");
}

#[test]
fn utf8_text_empty_range() {
    let tree = leaf(1, 3, 3);
    let source = b"abcdef";
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "");
}

// ===== 9. Error/missing flags =====

#[test]
fn is_error_false_on_all_nodes() {
    let tree = sample_tree();
    let root = tree.root_node();
    assert!(!root.is_error());
    for i in 0..root.child_count() {
        assert!(!root.child(i).unwrap().is_error());
    }
}

#[test]
fn is_missing_false_on_all_nodes() {
    let tree = sample_tree();
    let root = tree.root_node();
    assert!(!root.is_missing());
    for i in 0..root.child_count() {
        assert!(!root.child(i).unwrap().is_missing());
    }
}

#[test]
fn is_error_false_on_stub() {
    assert!(!Tree::new_stub().root_node().is_error());
}

#[test]
fn is_missing_false_on_stub() {
    assert!(!Tree::new_stub().root_node().is_missing());
}

#[test]
fn error_flags_on_grandchildren() {
    let tree = sample_tree();
    let gc = tree.root_node().child(1).unwrap().child(0).unwrap();
    assert!(!gc.is_error());
    assert!(!gc.is_missing());
}

// ===== 10. Deep tree traversal patterns =====

#[test]
fn three_level_deep_navigation() {
    let deep = branch(
        0,
        0,
        30,
        vec![branch(1, 0, 15, vec![branch(2, 0, 7, vec![leaf(3, 0, 3)])])],
    );
    let level3 = deep
        .root_node()
        .child(0)
        .unwrap()
        .child(0)
        .unwrap()
        .child(0)
        .unwrap();
    assert_eq!(level3.kind_id(), 3);
    assert_eq!(level3.byte_range(), 0..3);
    assert_eq!(level3.child_count(), 0);
}

#[test]
fn wide_tree_all_children_accessible() {
    let n = 10;
    let children: Vec<Tree> = (0..n)
        .map(|i| leaf(i as u32 + 1, i * 5, (i + 1) * 5))
        .collect();
    let tree = branch(0, 0, n * 5, children);
    let root = tree.root_node();
    assert_eq!(root.child_count(), n);
    for i in 0..n {
        let c = root.child(i).unwrap();
        assert_eq!(c.kind_id(), i as u16 + 1);
        assert_eq!(c.start_byte(), i * 5);
        assert_eq!(c.end_byte(), (i + 1) * 5);
    }
}

#[test]
fn single_child_tree() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 10)]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let only = root.child(0).unwrap();
    assert_eq!(only.kind_id(), 1);
    assert_eq!(only.byte_range(), 0..10);
}

#[test]
fn deeply_nested_single_chain() {
    // 0 -> 1 -> 2 -> 3 -> 4 (leaf)
    let tree = branch(
        0,
        0,
        100,
        vec![branch(
            1,
            0,
            80,
            vec![branch(
                2,
                0,
                60,
                vec![branch(3, 0, 40, vec![leaf(4, 0, 20)])],
            )],
        )],
    );
    let mut node = tree.root_node();
    for expected_id in 0..=4u16 {
        assert_eq!(node.kind_id(), expected_id);
        if expected_id < 4 {
            node = node.child(0).unwrap();
        }
    }
    assert_eq!(node.child_count(), 0);
}

#[test]
fn count_all_nodes_recursive() {
    fn count_nodes(tree: &Tree) -> usize {
        fn count(node: adze_runtime::Node<'_>) -> usize {
            let mut total = 1;
            for i in 0..node.child_count() {
                total += count(node.child(i).unwrap());
            }
            total
        }
        count(tree.root_node())
    }
    let tree = sample_tree();
    // root + 3 children + 2 grandchildren = 6
    assert_eq!(count_nodes(&tree), 6);
}

#[test]
fn collect_leaf_byte_ranges() {
    fn collect_leaves(node: adze_runtime::Node<'_>) -> Vec<std::ops::Range<usize>> {
        if node.child_count() == 0 {
            return vec![node.byte_range()];
        }
        let mut ranges = Vec::new();
        for i in 0..node.child_count() {
            ranges.extend(collect_leaves(node.child(i).unwrap()));
        }
        ranges
    }
    let tree = sample_tree();
    let leaves = collect_leaves(tree.root_node());
    assert_eq!(leaves, vec![0..5, 5..8, 8..12, 12..20]);
}

#[test]
fn collect_kind_ids_preorder() {
    fn preorder(node: adze_runtime::Node<'_>, acc: &mut Vec<u16>) {
        acc.push(node.kind_id());
        for i in 0..node.child_count() {
            preorder(node.child(i).unwrap(), acc);
        }
    }
    let tree = sample_tree();
    let mut ids = Vec::new();
    preorder(tree.root_node(), &mut ids);
    assert_eq!(ids, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn node_is_copy() {
    let tree = leaf(1, 0, 5);
    let node = tree.root_node();
    let copy = node;
    assert_eq!(node.kind_id(), copy.kind_id());
    assert_eq!(node.byte_range(), copy.byte_range());
}

#[test]
fn node_debug_format() {
    let tree = leaf(7, 3, 9);
    let dbg = format!("{:?}", tree.root_node());
    assert!(dbg.contains("Node"));
    assert!(dbg.contains("3..9"));
}

#[test]
fn tree_debug_format() {
    let tree = leaf(1, 0, 5);
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"));
}

#[test]
fn next_named_sibling_returns_none() {
    let tree = sample_tree();
    let child = tree.root_node().child(0).unwrap();
    assert!(child.next_named_sibling().is_none());
}

#[test]
fn prev_named_sibling_returns_none() {
    let tree = sample_tree();
    let child = tree.root_node().child(2).unwrap();
    assert!(child.prev_named_sibling().is_none());
}

#[test]
fn point_new_constructor() {
    let p = Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_clone_and_copy() {
    let p = Point::new(1, 2);
    let p2 = p;
    let p3 = p;
    assert_eq!(p, p2);
    assert_eq!(p, p3);
}

#[test]
fn point_ord() {
    let a = Point::new(0, 5);
    let b = Point::new(1, 0);
    let c = Point::new(0, 10);
    assert!(a < b);
    assert!(a < c);
    assert!(b > c);
}

#[test]
fn point_eq() {
    assert_eq!(Point::new(3, 7), Point::new(3, 7));
    assert_ne!(Point::new(3, 7), Point::new(3, 8));
}

#[test]
fn point_display() {
    let p = Point::new(2, 4);
    let s = format!("{p}");
    // Display uses 1-indexed
    assert_eq!(s, "3:5");
}

#[test]
fn tree_clone_independence() {
    let tree = sample_tree();
    let cloned = tree.clone();
    // Both have same structure
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    assert_eq!(
        tree.root_node().byte_range(),
        cloned.root_node().byte_range()
    );
}

#[test]
fn no_language_means_no_language() {
    let tree = leaf(1, 0, 5);
    assert!(tree.language().is_none());
}

#[test]
fn source_bytes_none_for_testing_tree() {
    let tree = leaf(1, 0, 5);
    assert!(tree.source_bytes().is_none());
}

#[test]
fn root_kind_returns_symbol() {
    let tree = branch(42, 0, 100, vec![leaf(1, 0, 50)]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn child_inherits_no_language() {
    let tree = sample_tree();
    let child = tree.root_node().child(0).unwrap();
    // Without language set, kind should be "unknown"
    assert_eq!(child.kind(), "unknown");
}

#[test]
fn mixed_depth_traversal() {
    // root -> [leaf, branch -> [leaf, leaf], leaf]
    let tree = sample_tree();
    let root = tree.root_node();

    // First child is a leaf
    assert_eq!(root.child(0).unwrap().child_count(), 0);
    // Second child is a branch with 2 children
    assert_eq!(root.child(1).unwrap().child_count(), 2);
    // Third child is a leaf
    assert_eq!(root.child(2).unwrap().child_count(), 0);
}

#[test]
fn empty_children_vec() {
    let tree = branch(0, 0, 10, vec![]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
    assert!(root.child(0).is_none());
}

#[test]
fn max_symbol_id() {
    // u16::MAX = 65535
    let tree = leaf(65535, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 65535);
}

#[test]
fn kind_id_truncation_from_u32() {
    // kind_id() returns u16, so large u32 symbols truncate
    let tree = leaf(65536, 0, 1);
    // 65536 as u16 == 0
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn child_of_child_of_child_none_on_leaf() {
    let tree = branch(0, 0, 10, vec![branch(1, 0, 5, vec![leaf(2, 0, 3)])]);
    let deepest = tree.root_node().child(0).unwrap().child(0).unwrap();
    assert_eq!(deepest.kind_id(), 2);
    assert!(deepest.child(0).is_none());
}
