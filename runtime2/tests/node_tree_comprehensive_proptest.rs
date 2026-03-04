// Comprehensive property tests for runtime2 Node and Tree API
use adze_runtime::tree::TreeCursor;
use adze_runtime::{InputEdit, Tree};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Tree construction
// ---------------------------------------------------------------------------

#[test]
fn tree_stub() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_for_testing_leaf() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_for_testing_with_children() {
    let child1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let child2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let tree = Tree::new_for_testing(1, 0, 6, vec![child1, child2]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
}

// ---------------------------------------------------------------------------
// Node byte range
// ---------------------------------------------------------------------------

#[test]
fn node_byte_range() {
    let tree = Tree::new_for_testing(1, 10, 20, vec![]);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 10..20);
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 20);
}

// ---------------------------------------------------------------------------
// Node kind
// ---------------------------------------------------------------------------

#[test]
fn node_kind_id() {
    let tree = Tree::new_for_testing(42, 0, 5, vec![]);
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 42);
}

// ---------------------------------------------------------------------------
// Node children access
// ---------------------------------------------------------------------------

#[test]
fn node_child_by_index() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 3, vec![child]);
    let root = tree.root_node();
    let c = root.child(0);
    assert!(c.is_some());
    assert_eq!(c.unwrap().kind_id(), 2);
}

#[test]
fn node_child_out_of_bounds() {
    let tree = Tree::new_for_testing(1, 0, 3, vec![]);
    let root = tree.root_node();
    assert!(root.child(0).is_none());
    assert!(root.child(100).is_none());
}

// ---------------------------------------------------------------------------
// Node predicates
// ---------------------------------------------------------------------------

#[test]
fn node_is_error_default_false() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = tree.root_node();
    assert!(!root.is_error());
}

#[test]
fn node_is_missing_default_false() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = tree.root_node();
    assert!(!root.is_missing());
}

// ---------------------------------------------------------------------------
// TreeCursor
// ---------------------------------------------------------------------------

#[test]
fn tree_cursor_basic() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(!cursor.goto_first_child()); // leaf
}

#[test]
fn tree_cursor_sibling() {
    let c1 = Tree::new_for_testing(2, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(3, 2, 4, vec![]);
    let tree = Tree::new_for_testing(1, 0, 4, vec![c1, c2]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

#[test]
fn tree_cursor_goto_parent() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(!cursor.goto_parent()); // at root
}

#[test]
fn tree_cursor_depth() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 1, vec![leaf]);
    let tree = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn tree_cursor_reset() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 1);
}

// ---------------------------------------------------------------------------
// Tree root_kind
// ---------------------------------------------------------------------------

#[test]
fn tree_root_kind() {
    let tree = Tree::new_for_testing(42, 0, 5, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

// ---------------------------------------------------------------------------
// Deep trees
// ---------------------------------------------------------------------------

#[test]
fn deep_tree_cursor_traversal() {
    // Build a 10-level deep tree
    let mut tree = Tree::new_for_testing(10, 0, 1, vec![]);
    for i in (0..10).rev() {
        tree = Tree::new_for_testing(i as u32, 0, 1, vec![tree]);
    }
    let mut cursor = TreeCursor::new(&tree);
    for expected_depth in 0..=10 {
        assert_eq!(cursor.depth(), expected_depth);
        if expected_depth < 10 {
            assert!(cursor.goto_first_child());
        }
    }
}

// ---------------------------------------------------------------------------
// Wide trees
// ---------------------------------------------------------------------------

#[test]
fn wide_tree_children() {
    let children: Vec<_> = (0..20)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 20, children);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 20);
    for i in 0..20 {
        let child = root.child(i).unwrap();
        assert_eq!(child.kind_id(), (i + 10) as u16);
    }
}

// ---------------------------------------------------------------------------
// Sibling navigation via cursor
// ---------------------------------------------------------------------------

#[test]
fn cursor_sibling_navigation() {
    let c1 = Tree::new_for_testing(2, 0, 1, vec![]);
    let c2 = Tree::new_for_testing(3, 1, 2, vec![]);
    let c3 = Tree::new_for_testing(4, 2, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 3, vec![c1, c2, c3]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 4);
    assert!(!cursor.goto_next_sibling());
}

// ---------------------------------------------------------------------------
// InputEdit struct
// ---------------------------------------------------------------------------

#[test]
fn input_edit_creation() {
    use adze_runtime::node::Point;
    let edit = InputEdit {
        start_byte: 0,
        old_end_byte: 5,
        new_end_byte: 10,
        start_position: Point::new(0, 0),
        old_end_position: Point::new(0, 5),
        new_end_position: Point::new(0, 10),
    };
    assert_eq!(edit.start_byte, 0);
    assert_eq!(edit.old_end_byte, 5);
    assert_eq!(edit.new_end_byte, 10);
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn tree_byte_range_consistent(start in 0usize..1000, len in 0usize..1000) {
        let end = start + len;
        let tree = Tree::new_for_testing(1, start, end, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.start_byte(), start);
        prop_assert_eq!(root.end_byte(), end);
        prop_assert_eq!(root.byte_range(), start..end);
    }

    #[test]
    fn tree_kind_id_preserved(kind in 0u32..1000) {
        let tree = Tree::new_for_testing(kind, 0, 10, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.kind_id(), kind as u16);
    }

    #[test]
    fn child_count_matches(n in 0usize..10) {
        let children: Vec<_> = (0..n)
            .map(|i| Tree::new_for_testing((i + 2) as u32, i, i + 1, vec![]))
            .collect();
        let tree = Tree::new_for_testing(1, 0, n, children);
        let root = tree.root_node();
        prop_assert_eq!(root.child_count(), n);
    }

    #[test]
    fn cursor_depth_matches_level(depth in 1usize..8) {
        use adze_runtime::tree::TreeCursor;
        // Build a tree of given depth
        let mut t = Tree::new_for_testing(depth as u32, 0, 1, vec![]);
        for i in (0..depth).rev() {
            t = Tree::new_for_testing(i as u32, 0, 1, vec![t]);
        }
        let mut cursor = TreeCursor::new(&t);
        for d in 0..depth {
            prop_assert_eq!(cursor.depth(), d);
            prop_assert!(cursor.goto_first_child());
        }
        prop_assert_eq!(cursor.depth(), depth);
    }
}

// ---------------------------------------------------------------------------
// Point struct
// ---------------------------------------------------------------------------

#[test]
fn point_new() {
    use adze_runtime::node::Point;
    let p = Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_zero() {
    use adze_runtime::node::Point;
    let p = Point::new(0, 0);
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}
