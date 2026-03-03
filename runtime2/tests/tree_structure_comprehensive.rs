#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for tree structure and node relationships in adze-runtime.

use adze_runtime::tree::{Tree, TreeCursor};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

// ---------------------------------------------------------------------------
// Leaf / no-children tests
// ---------------------------------------------------------------------------

#[test]
fn leaf_node_has_zero_children() {
    let tree = leaf(1, 0, 5);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn leaf_node_child_returns_none() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().child(0).is_none());
}

#[test]
fn leaf_node_byte_range() {
    let tree = leaf(42, 10, 20);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 20);
    assert_eq!(root.byte_range(), 10..20);
}

#[test]
fn leaf_node_kind_id() {
    let tree = leaf(7, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 7);
}

#[test]
fn leaf_cursor_cannot_descend() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

// ---------------------------------------------------------------------------
// One-child tests
// ---------------------------------------------------------------------------

#[test]
fn one_child_count() {
    let child = leaf(2, 0, 3);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    assert_eq!(tree.root_node().child_count(), 1);
}

#[test]
fn one_child_accessible_by_index() {
    let child = leaf(2, 1, 4);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let c = tree.root_node().child(0).unwrap();
    assert_eq!(c.kind_id(), 2);
    assert_eq!(c.start_byte(), 1);
    assert_eq!(c.end_byte(), 4);
}

#[test]
fn one_child_out_of_bounds_returns_none() {
    let child = leaf(2, 0, 3);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    assert!(tree.root_node().child(1).is_none());
}

#[test]
fn one_child_cursor_traversal() {
    let child = leaf(2, 0, 3);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(!cursor.goto_next_sibling());
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
}

// ---------------------------------------------------------------------------
// Many-children tests
// ---------------------------------------------------------------------------

#[test]
fn many_children_count() {
    let children: Vec<Tree> = (0..5)
        .map(|i| leaf(10 + i, (i * 2) as usize, (i * 2 + 2) as usize))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 10, children);
    assert_eq!(tree.root_node().child_count(), 5);
}

#[test]
fn many_children_sequential_access() {
    let children: Vec<Tree> = (0..5)
        .map(|i| leaf(10 + i, (i * 2) as usize, (i * 2 + 2) as usize))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 10, children);
    let root = tree.root_node();
    for i in 0..5 {
        let c = root.child(i).unwrap();
        assert_eq!(c.kind_id(), 10 + i as u16);
    }
}

#[test]
fn many_children_cursor_siblings() {
    let children: Vec<Tree> = (0..4)
        .map(|i| leaf(20 + i, (i * 3) as usize, (i * 3 + 3) as usize))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 12, children);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    let mut visited = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        visited.push(cursor.node().kind_id());
    }
    assert_eq!(visited, vec![20, 21, 22, 23]);
}

// ---------------------------------------------------------------------------
// Deep tree tests (10+ levels)
// ---------------------------------------------------------------------------

fn build_deep_tree(depth: u32) -> Tree {
    let mut current = leaf(depth, 0, 1);
    for d in (0..depth).rev() {
        current = Tree::new_for_testing(d, 0, 1, vec![current]);
    }
    current
}

#[test]
fn deep_tree_depth_via_cursor() {
    let tree = build_deep_tree(12);
    let mut cursor = TreeCursor::new(&tree);
    let mut max_depth = 0;
    while cursor.goto_first_child() {
        max_depth += 1;
    }
    assert_eq!(max_depth, 12);
}

#[test]
fn deep_tree_leaf_symbol() {
    let tree = build_deep_tree(10);
    let mut cursor = TreeCursor::new(&tree);
    while cursor.goto_first_child() {}
    assert_eq!(cursor.node().kind_id(), 10);
}

#[test]
fn deep_tree_cursor_round_trip() {
    let tree = build_deep_tree(15);
    let mut cursor = TreeCursor::new(&tree);
    let mut depth = 0;
    while cursor.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, 15);
    // Walk all the way back up
    for _ in 0..depth {
        assert!(cursor.goto_parent());
    }
    assert_eq!(cursor.node().kind_id(), 0);
    assert!(!cursor.goto_parent());
}

#[test]
fn deep_tree_cursor_depth_method() {
    let tree = build_deep_tree(10);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    for expected in 1..=10 {
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.depth(), expected);
    }
}

// ---------------------------------------------------------------------------
// Wide tree tests (many siblings)
// ---------------------------------------------------------------------------

#[test]
fn wide_tree_hundred_children() {
    let children: Vec<Tree> = (0..100).map(|i| leaf(i + 1, i as usize, (i + 1) as usize)).collect();
    let tree = Tree::new_for_testing(0, 0, 100, children);
    assert_eq!(tree.root_node().child_count(), 100);
}

#[test]
fn wide_tree_cursor_visits_all_siblings() {
    let n = 50;
    let children: Vec<Tree> = (0..n).map(|i| leaf(i + 1, i as usize, (i + 1) as usize)).collect();
    let tree = Tree::new_for_testing(0, 0, n as usize, children);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, n);
}

#[test]
fn wide_tree_last_child_accessible() {
    let n = 80u32;
    let children: Vec<Tree> = (0..n).map(|i| leaf(i + 1, i as usize, (i + 1) as usize)).collect();
    let tree = Tree::new_for_testing(0, 0, n as usize, children);
    let last = tree.root_node().child((n - 1) as usize).unwrap();
    assert_eq!(last.kind_id(), n as u16);
}

// ---------------------------------------------------------------------------
// Parent relationship tests
// ---------------------------------------------------------------------------

#[test]
fn parent_returns_none_without_links() {
    let child = leaf(2, 0, 3);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    // Node::parent() returns None because parent links are not stored
    assert!(tree.root_node().parent().is_none());
    assert!(tree.root_node().child(0).unwrap().parent().is_none());
}

#[test]
fn cursor_goto_parent_works() {
    let grandchild = leaf(3, 0, 1);
    let child = Tree::new_for_testing(2, 0, 2, vec![grandchild]);
    let tree = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
}

// ---------------------------------------------------------------------------
// Sibling relationship tests
// ---------------------------------------------------------------------------

#[test]
fn next_sibling_returns_none_without_links() {
    let c1 = leaf(2, 0, 2);
    let c2 = leaf(3, 2, 4);
    let tree = Tree::new_for_testing(1, 0, 4, vec![c1, c2]);
    // Node::next_sibling() returns None because sibling links are not stored
    let first = tree.root_node().child(0).unwrap();
    assert!(first.next_sibling().is_none());
}

#[test]
fn prev_sibling_returns_none_without_links() {
    let c1 = leaf(2, 0, 2);
    let c2 = leaf(3, 2, 4);
    let tree = Tree::new_for_testing(1, 0, 4, vec![c1, c2]);
    let second = tree.root_node().child(1).unwrap();
    assert!(second.prev_sibling().is_none());
}

#[test]
fn cursor_sibling_navigation() {
    let children: Vec<Tree> = (0..3).map(|i| leaf(10 + i, (i * 2) as usize, (i * 2 + 2) as usize)).collect();
    let tree = Tree::new_for_testing(1, 0, 6, children);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 10);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 11);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 12);
    assert!(!cursor.goto_next_sibling());
}

// ---------------------------------------------------------------------------
// Byte range tests
// ---------------------------------------------------------------------------

#[test]
fn byte_range_covers_children() {
    let c1 = leaf(2, 5, 10);
    let c2 = leaf(3, 10, 20);
    let tree = Tree::new_for_testing(1, 5, 20, vec![c1, c2]);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 5..20);
}

#[test]
fn child_byte_ranges_non_overlapping() {
    let c1 = leaf(2, 0, 5);
    let c2 = leaf(3, 5, 10);
    let c3 = leaf(4, 10, 15);
    let tree = Tree::new_for_testing(1, 0, 15, vec![c1, c2, c3]);
    let root = tree.root_node();
    for i in 0..root.child_count() - 1 {
        let current = root.child(i).unwrap();
        let next = root.child(i + 1).unwrap();
        assert!(current.end_byte() <= next.start_byte());
    }
}

#[test]
fn zero_width_byte_range() {
    let tree = leaf(1, 5, 5);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.byte_range(), 5..5);
}

// ---------------------------------------------------------------------------
// Root node properties
// ---------------------------------------------------------------------------

#[test]
fn root_kind_matches_symbol() {
    let tree = Tree::new_for_testing(99, 0, 50, vec![]);
    assert_eq!(tree.root_kind(), 99);
    assert_eq!(tree.root_node().kind_id(), 99);
}

#[test]
fn root_is_named() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().is_named());
}

#[test]
fn root_is_not_error() {
    let tree = leaf(1, 0, 5);
    assert!(!tree.root_node().is_error());
}

#[test]
fn root_is_not_missing() {
    let tree = leaf(1, 0, 5);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn root_no_language_returns_unknown_kind() {
    let tree = leaf(1, 0, 5);
    assert!(tree.language().is_none());
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn root_has_no_parent_via_cursor() {
    let tree = leaf(1, 0, 5);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn tree_debug_format_non_empty() {
    let child = leaf(2, 0, 3);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let debug = format!("{:?}", tree);
    assert!(!debug.is_empty());
    assert!(debug.contains("Tree"));
}

#[test]
fn cursor_reset_returns_to_root() {
    let child = leaf(2, 0, 3);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 2);
    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn clone_preserves_structure() {
    let c1 = leaf(2, 0, 3);
    let c2 = leaf(3, 3, 6);
    let tree = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_kind(), tree.root_kind());
    assert_eq!(cloned.root_node().child_count(), 2);
    assert_eq!(cloned.root_node().child(0).unwrap().kind_id(), 2);
    assert_eq!(cloned.root_node().child(1).unwrap().kind_id(), 3);
}

#[test]
fn named_child_count_equals_child_count() {
    let children: Vec<Tree> = (0..4).map(|i| leaf(i + 1, 0, 1)).collect();
    let tree = Tree::new_for_testing(0, 0, 1, children);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn named_child_same_as_child() {
    let c = leaf(5, 0, 3);
    let tree = Tree::new_for_testing(1, 0, 5, vec![c]);
    let root = tree.root_node();
    let via_child = root.child(0).unwrap();
    let via_named = root.named_child(0).unwrap();
    assert_eq!(via_child.kind_id(), via_named.kind_id());
    assert_eq!(via_child.byte_range(), via_named.byte_range());
}
