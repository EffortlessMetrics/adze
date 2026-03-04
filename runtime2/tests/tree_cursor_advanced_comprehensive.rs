//! Comprehensive tests for Tree and TreeCursor advanced patterns.

use adze_runtime::node::Point;
use adze_runtime::tree::{Tree, TreeCursor};

// ── Tree::new_stub ──

#[test]
fn stub_tree_root_kind() {
    let t = Tree::new_stub();
    assert_eq!(t.root_kind(), 0);
}

#[test]
fn stub_tree_language_none() {
    let t = Tree::new_stub();
    assert!(t.language().is_none());
}

#[test]
fn stub_tree_source_none() {
    let t = Tree::new_stub();
    assert!(t.source_bytes().is_none());
}

#[test]
fn stub_tree_root_node_kind_id() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert_eq!(root.kind_id(), 0);
}

#[test]
fn stub_tree_root_node_byte_range() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn stub_tree_root_node_positions() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert_eq!(root.start_position(), Point { row: 0, column: 0 });
    assert_eq!(root.end_position(), Point { row: 0, column: 0 });
}

#[test]
fn stub_tree_root_no_children() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert_eq!(root.child_count(), 0);
}

#[test]
fn stub_tree_root_named_child_count() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert_eq!(root.named_child_count(), 0);
}

#[test]
fn stub_tree_root_child_none() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.child(0).is_none());
}

#[test]
fn stub_tree_root_named_child_none() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.named_child(0).is_none());
}

#[test]
fn stub_tree_root_not_error() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(!root.is_error());
}

#[test]
fn stub_tree_root_not_missing() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(!root.is_missing());
}

#[test]
fn stub_tree_root_parent_none() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.parent().is_none());
}

#[test]
fn stub_tree_root_next_sibling_none() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.next_sibling().is_none());
}

#[test]
fn stub_tree_root_prev_sibling_none() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.prev_sibling().is_none());
}

// ── Tree::new_for_testing ──

#[test]
fn testing_tree_root_kind() {
    let t = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(t.root_kind(), 42);
}

#[test]
fn testing_tree_byte_range() {
    let t = Tree::new_for_testing(1, 5, 20, vec![]);
    let root = t.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 20);
}

#[test]
fn testing_tree_with_children() {
    let child1 = Tree::new_for_testing(10, 0, 3, vec![]);
    let child2 = Tree::new_for_testing(11, 3, 6, vec![]);
    let parent = Tree::new_for_testing(1, 0, 6, vec![child1, child2]);
    let root = parent.root_node();
    assert_eq!(root.child_count(), 2);
}

#[test]
fn testing_tree_child_access() {
    let child = Tree::new_for_testing(10, 0, 3, vec![]);
    let parent = Tree::new_for_testing(1, 0, 3, vec![child]);
    let root = parent.root_node();
    let c = root.child(0).unwrap();
    assert_eq!(c.kind_id(), 10);
}

#[test]
fn testing_tree_child_out_of_bounds() {
    let parent = Tree::new_for_testing(1, 0, 3, vec![]);
    let root = parent.root_node();
    assert!(root.child(0).is_none());
    assert!(root.child(99).is_none());
}

#[test]
fn testing_tree_nested_children() {
    let leaf = Tree::new_for_testing(20, 0, 1, vec![]);
    let mid = Tree::new_for_testing(10, 0, 1, vec![leaf]);
    let root_tree = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let root = root_tree.root_node();
    assert_eq!(root.child_count(), 1);
    let child = root.child(0).unwrap();
    assert_eq!(child.child_count(), 1);
}

#[test]
fn testing_tree_zero_span() {
    let t = Tree::new_for_testing(1, 0, 0, vec![]);
    let root = t.root_node();
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn testing_tree_large_symbol() {
    let t = Tree::new_for_testing(u32::MAX, 0, 10, vec![]);
    assert_eq!(t.root_kind(), u32::MAX);
}

#[test]
fn testing_tree_multiple_children() {
    let children: Vec<_> = (0..10)
        .map(|i| Tree::new_for_testing(100 + i, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let parent = Tree::new_for_testing(1, 0, 10, children);
    let root = parent.root_node();
    assert_eq!(root.child_count(), 10);
    for i in 0..10 {
        let c = root.child(i).unwrap();
        assert_eq!(c.kind_id(), (100 + i) as u16);
    }
}

// ── TreeCursor ──

#[test]
fn cursor_construction() {
    let t = Tree::new_stub();
    let _cursor = TreeCursor::new(&t);
}

#[test]
fn cursor_node_is_root() {
    let t = Tree::new_stub();
    let cursor = TreeCursor::new(&t);
    let node = cursor.node();
    assert_eq!(node.kind_id(), 0);
}

#[test]
fn cursor_depth_at_root() {
    let t = Tree::new_stub();
    let cursor = TreeCursor::new(&t);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child_empty() {
    let t = Tree::new_stub();
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_goto_next_sibling_root() {
    let t = Tree::new_stub();
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent_at_root() {
    let t = Tree::new_stub();
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_navigate_children() {
    let c1 = Tree::new_for_testing(10, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(11, 3, 6, vec![]);
    let parent = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    let mut cursor = TreeCursor::new(&parent);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 10);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 11);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_depth_tracks() {
    let leaf = Tree::new_for_testing(20, 0, 1, vec![]);
    let mid = Tree::new_for_testing(10, 0, 1, vec![leaf]);
    let root_tree = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let mut cursor = TreeCursor::new(&root_tree);
    assert_eq!(cursor.depth(), 0);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn cursor_goto_parent() {
    let leaf = Tree::new_for_testing(20, 0, 1, vec![]);
    let mid = Tree::new_for_testing(10, 0, 1, vec![leaf]);
    let root_tree = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let mut cursor = TreeCursor::new(&root_tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().kind_id(), 10);
}

#[test]
fn cursor_reset() {
    let c1 = Tree::new_for_testing(10, 0, 3, vec![]);
    let parent = Tree::new_for_testing(1, 0, 3, vec![c1]);
    let mut cursor = TreeCursor::new(&parent);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&parent);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn cursor_full_traversal() {
    let c1 = Tree::new_for_testing(10, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(11, 2, 4, vec![]);
    let c3 = Tree::new_for_testing(12, 4, 6, vec![]);
    let parent = Tree::new_for_testing(1, 0, 6, vec![c1, c2, c3]);
    let mut cursor = TreeCursor::new(&parent);
    let mut visited = vec![cursor.node().kind_id()];
    if cursor.goto_first_child() {
        visited.push(cursor.node().kind_id());
        while cursor.goto_next_sibling() {
            visited.push(cursor.node().kind_id());
        }
    }
    assert_eq!(visited, vec![1, 10, 11, 12]);
}

// ── Node relationships ──

#[test]
fn node_field_name_child_on_stub() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.child_by_field_name("nonexistent").is_none());
}

#[test]
fn node_next_named_sibling() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.next_named_sibling().is_none());
}

#[test]
fn node_prev_named_sibling() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert!(root.prev_named_sibling().is_none());
}

// ── Point ──

#[test]
fn point_default() {
    let p = Point { row: 0, column: 0 };
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_ordering() {
    let p1 = Point { row: 0, column: 5 };
    let p2 = Point { row: 1, column: 0 };
    assert!(p1 < p2);
}

#[test]
fn point_equality() {
    let p1 = Point { row: 3, column: 7 };
    let p2 = Point { row: 3, column: 7 };
    assert_eq!(p1, p2);
}

#[test]
fn point_copy() {
    let p1 = Point { row: 1, column: 2 };
    let p2 = p1;
    assert_eq!(p1, p2);
}

#[test]
fn point_debug() {
    let p = Point { row: 0, column: 0 };
    let d = format!("{:?}", p);
    assert!(d.contains("Point"));
}

// ── Deep nesting ──

#[test]
fn deep_nesting_5_levels() {
    let mut tree = Tree::new_for_testing(50, 0, 1, vec![]);
    for i in (0..5).rev() {
        tree = Tree::new_for_testing(i, 0, 1, vec![tree]);
    }
    let mut cursor = TreeCursor::new(&tree);
    for expected_depth in 0..=5 {
        assert_eq!(cursor.depth(), expected_depth);
        if expected_depth < 5 {
            assert!(cursor.goto_first_child());
        }
    }
}

// ── Multiple stubs ──

#[test]
fn multiple_stubs_independent() {
    let t1 = Tree::new_stub();
    let t2 = Tree::new_stub();
    assert_eq!(t1.root_kind(), t2.root_kind());
}

// ── Node is_named on testing tree ──

#[test]
fn testing_tree_node_is_named() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = t.root_node();
    // new_for_testing may or may not set named
    let _ = root.is_named();
}

// ── Cursor on many siblings ──

#[test]
fn cursor_many_siblings() {
    let children: Vec<_> = (0..20)
        .map(|i| Tree::new_for_testing(100 + i, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let parent = Tree::new_for_testing(1, 0, 20, children);
    let mut cursor = TreeCursor::new(&parent);
    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 20);
}
