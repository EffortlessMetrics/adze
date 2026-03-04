//! Comprehensive tests for Tree and TreeNode construction and manipulation.

use adze_runtime::node::Point;
use adze_runtime::tree::{Tree, TreeCursor};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Leaf node: no children.
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Branch node with children.
fn branch(symbol: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(symbol, start, end, children)
}

/// Build a simple `1 + 2` style tree:
///   expr(0..5)
///     number(0..1)
///     plus(1..2)
///     number(2..5)
fn simple_expr_tree() -> Tree {
    branch(0, 0, 5, vec![leaf(1, 0, 1), leaf(2, 1, 2), leaf(1, 2, 5)])
}

// ===================================================================
// 1. Tree construction from root node
// ===================================================================

#[test]
fn new_stub_has_zero_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn new_stub_byte_ranges_are_zero() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn new_stub_root_kind_is_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn new_for_testing_preserves_symbol() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn new_for_testing_preserves_byte_range() {
    let tree = Tree::new_for_testing(0, 5, 20, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 20);
}

#[test]
fn new_for_testing_with_children() {
    let tree = simple_expr_tree();
    assert_eq!(tree.root_node().child_count(), 3);
}

#[test]
fn language_defaults_to_none() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn source_bytes_defaults_to_none() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

// ===================================================================
// 2. TreeNode types — leaf vs branch
// ===================================================================

#[test]
fn leaf_node_has_zero_children() {
    let tree = leaf(1, 0, 3);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn branch_node_has_expected_children() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn nested_branch_preserves_depth() {
    let inner = branch(2, 2, 4, vec![leaf(3, 2, 3), leaf(4, 3, 4)]);
    let tree = branch(0, 0, 6, vec![leaf(1, 0, 2), inner]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    let second = root.child(1).unwrap();
    assert_eq!(second.child_count(), 2);
}

// ===================================================================
// 3. Node accessors
// ===================================================================

#[test]
fn kind_without_language_returns_unknown() {
    let tree = leaf(5, 0, 3);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn kind_without_language_is_always_unknown() {
    // Without a language, every symbol resolves to "unknown"
    for sym in [0, 1, 99, 1000] {
        let tree = leaf(sym, 0, 1);
        assert_eq!(tree.root_node().kind(), "unknown");
    }
}

#[test]
fn kind_id_returns_symbol_as_u16() {
    let tree = Tree::new_for_testing(300, 0, 1, vec![]);
    assert_eq!(tree.root_node().kind_id(), 300u16);
}

#[test]
fn byte_range_matches_start_end() {
    let tree = leaf(0, 10, 25);
    assert_eq!(tree.root_node().byte_range(), 10..25);
}

#[test]
fn start_position_returns_zero_point() {
    let tree = leaf(0, 0, 1);
    assert_eq!(tree.root_node().start_position(), Point::new(0, 0));
}

#[test]
fn end_position_returns_zero_point() {
    let tree = leaf(0, 0, 1);
    assert_eq!(tree.root_node().end_position(), Point::new(0, 0));
}

#[test]
fn is_named_always_true() {
    let tree = leaf(0, 0, 1);
    assert!(tree.root_node().is_named());
}

#[test]
fn is_error_always_false() {
    let tree = leaf(0, 0, 1);
    assert!(!tree.root_node().is_error());
}

#[test]
fn is_missing_always_false() {
    let tree = leaf(0, 0, 1);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn named_child_count_equals_child_count() {
    let tree = simple_expr_tree();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

// ===================================================================
// 4. Parent / child relationships
// ===================================================================

#[test]
fn child_returns_correct_nodes() {
    let tree = simple_expr_tree();
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    assert_eq!(c0.kind_id(), 1);
    assert_eq!(c0.start_byte(), 0);
    assert_eq!(c0.end_byte(), 1);
}

#[test]
fn child_out_of_bounds_returns_none() {
    let tree = leaf(0, 0, 1);
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(999).is_none());
}

#[test]
fn named_child_matches_child() {
    let tree = simple_expr_tree();
    let root = tree.root_node();
    let c = root.child(1).unwrap();
    let nc = root.named_child(1).unwrap();
    assert_eq!(c.kind_id(), nc.kind_id());
    assert_eq!(c.byte_range(), nc.byte_range());
}

#[test]
fn parent_returns_none() {
    let tree = simple_expr_tree();
    assert!(tree.root_node().parent().is_none());
    assert!(tree.root_node().child(0).unwrap().parent().is_none());
}

// ===================================================================
// 5. Sibling navigation
// ===================================================================

#[test]
fn next_sibling_returns_none() {
    let tree = simple_expr_tree();
    assert!(tree.root_node().child(0).unwrap().next_sibling().is_none());
}

#[test]
fn prev_sibling_returns_none() {
    let tree = simple_expr_tree();
    assert!(tree.root_node().child(1).unwrap().prev_sibling().is_none());
}

#[test]
fn next_named_sibling_returns_none() {
    let tree = simple_expr_tree();
    assert!(
        tree.root_node()
            .child(0)
            .unwrap()
            .next_named_sibling()
            .is_none()
    );
}

#[test]
fn prev_named_sibling_returns_none() {
    let tree = simple_expr_tree();
    assert!(
        tree.root_node()
            .child(2)
            .unwrap()
            .prev_named_sibling()
            .is_none()
    );
}

// ===================================================================
// 6. Named vs anonymous nodes (Phase 1: all named)
// ===================================================================

#[test]
fn all_nodes_are_named_in_phase1() {
    let tree = simple_expr_tree();
    let root = tree.root_node();
    assert!(root.is_named());
    for i in 0..root.child_count() {
        assert!(root.child(i).unwrap().is_named());
    }
}

// ===================================================================
// 7. Error nodes and missing nodes (always false)
// ===================================================================

#[test]
fn root_is_not_error() {
    assert!(!simple_expr_tree().root_node().is_error());
}

#[test]
fn children_are_not_error() {
    let tree = simple_expr_tree();
    for i in 0..tree.root_node().child_count() {
        assert!(!tree.root_node().child(i).unwrap().is_error());
    }
}

#[test]
fn root_is_not_missing() {
    assert!(!simple_expr_tree().root_node().is_missing());
}

#[test]
fn children_are_not_missing() {
    let tree = simple_expr_tree();
    for i in 0..tree.root_node().child_count() {
        assert!(!tree.root_node().child(i).unwrap().is_missing());
    }
}

// ===================================================================
// 8. Tree walking / cursor usage
// ===================================================================

#[test]
fn cursor_starts_at_root() {
    let tree = simple_expr_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_goto_first_child_on_leaf_returns_false() {
    let tree = leaf(5, 0, 3);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_next_sibling() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_full_dfs_traversal() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    let mut visited = vec![cursor.node().kind_id()];
    if cursor.goto_first_child() {
        visited.push(cursor.node().kind_id());
        while cursor.goto_next_sibling() {
            visited.push(cursor.node().kind_id());
        }
    }
    assert_eq!(visited, vec![0, 1, 2, 1]);
}

#[test]
fn cursor_reset() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_depth_increases_with_nesting() {
    let inner = branch(2, 2, 4, vec![leaf(3, 2, 3)]);
    let tree = branch(0, 0, 5, vec![inner]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
}

#[test]
fn cursor_node_byte_ranges_correct() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().byte_range(), 0..5);
    cursor.goto_first_child();
    assert_eq!(cursor.node().byte_range(), 0..1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().byte_range(), 1..2);
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().byte_range(), 2..5);
}

#[test]
fn cursor_traversal_roundtrip() {
    let tree = simple_expr_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_parent();
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.node().start_byte(), 0);
}

// ===================================================================
// 9. Node field access
// ===================================================================

#[test]
fn child_by_field_name_returns_none() {
    let tree = simple_expr_tree();
    assert!(tree.root_node().child_by_field_name("left").is_none());
    assert!(tree.root_node().child_by_field_name("operator").is_none());
    assert!(tree.root_node().child_by_field_name("right").is_none());
}

#[test]
fn child_by_field_name_on_leaf_returns_none() {
    let tree = leaf(1, 0, 1);
    assert!(tree.root_node().child_by_field_name("value").is_none());
}

// ===================================================================
// 10. Deep trees (many levels)
// ===================================================================

#[test]
fn deep_tree_cursor_reaches_bottom() {
    const DEPTH: u32 = 50;
    let mut current = leaf(DEPTH, 0, 1);
    for i in (0..DEPTH).rev() {
        current = branch(i, 0, 1, vec![current]);
    }
    let tree = current;
    let mut cursor = TreeCursor::new(&tree);
    let mut depth = 0u32;
    while cursor.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, DEPTH);
    assert_eq!(cursor.node().kind_id(), DEPTH as u16);
}

#[test]
fn deep_tree_root_has_one_child_per_level() {
    let inner3 = leaf(3, 3, 4);
    let inner2 = branch(2, 2, 4, vec![inner3]);
    let inner1 = branch(1, 1, 4, vec![inner2]);
    let tree = branch(0, 0, 4, vec![inner1]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let c1 = root.child(0).unwrap();
    assert_eq!(c1.child_count(), 1);
    let c2 = c1.child(0).unwrap();
    assert_eq!(c2.child_count(), 1);
    let c3 = c2.child(0).unwrap();
    assert_eq!(c3.child_count(), 0);
}

#[test]
fn deep_tree_cursor_parent_returns_to_root() {
    let inner = branch(2, 0, 1, vec![leaf(3, 0, 1)]);
    let tree = branch(0, 0, 1, vec![branch(1, 0, 1, vec![inner])]);

    let mut cursor = TreeCursor::new(&tree);
    // Go all the way down
    while cursor.goto_first_child() {}
    assert_eq!(cursor.depth(), 3);
    // Come all the way back up
    while cursor.goto_parent() {}
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

// ===================================================================
// 11. Wide trees (many children)
// ===================================================================

#[test]
fn wide_tree_100_children() {
    let children: Vec<Tree> = (0..100).map(|i| leaf(1, i, i + 1)).collect();
    let tree = branch(0, 0, 100, children);
    assert_eq!(tree.root_node().child_count(), 100);
}

#[test]
fn wide_tree_cursor_visits_all_siblings() {
    let n = 50;
    let children: Vec<Tree> = (0..n).map(|i| leaf(i as u32 + 1, i, i + 1)).collect();
    let tree = branch(0, 0, n, children);

    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, n);
}

#[test]
fn wide_tree_child_access_by_index() {
    let children: Vec<Tree> = (0..20).map(|i| leaf(i as u32, i * 2, i * 2 + 2)).collect();
    let tree = branch(0, 0, 40, children);
    let root = tree.root_node();

    for i in 0..20 {
        let child = root.child(i).unwrap();
        assert_eq!(child.kind_id(), i as u16);
        assert_eq!(child.start_byte(), i * 2);
        assert_eq!(child.end_byte(), i * 2 + 2);
    }
    assert!(root.child(20).is_none());
}

#[test]
fn wide_tree_last_child_correct() {
    let children: Vec<Tree> = (0..10).map(|i| leaf(i as u32 + 1, i, i + 1)).collect();
    let tree = branch(0, 0, 10, children);
    let root = tree.root_node();
    let last = root.child(9).unwrap();
    assert_eq!(last.kind_id(), 10);
    assert_eq!(last.start_byte(), 9);
    assert_eq!(last.end_byte(), 10);
}

// ===================================================================
// 12. Single-node trees
// ===================================================================

#[test]
fn single_node_tree_root_is_leaf() {
    let tree = leaf(42, 0, 10);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
    assert_eq!(root.kind_id(), 42);
    assert_eq!(root.byte_range(), 0..10);
}

#[test]
fn single_node_cursor_cannot_descend() {
    let tree = leaf(0, 0, 1);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
}

#[test]
fn single_node_cursor_depth_is_zero() {
    let tree = leaf(0, 0, 1);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

// ===================================================================
// 13. Clone and equality behavior
// ===================================================================

#[test]
fn tree_clone_is_independent() {
    let tree = simple_expr_tree();
    let cloned = tree.clone();
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    assert_eq!(
        tree.root_node().byte_range(),
        cloned.root_node().byte_range()
    );
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
}

#[test]
fn cloned_tree_children_match_original() {
    let tree = simple_expr_tree();
    let cloned = tree.clone();
    for i in 0..tree.root_node().child_count() {
        let orig = tree.root_node().child(i).unwrap();
        let copy = cloned.root_node().child(i).unwrap();
        assert_eq!(orig.kind_id(), copy.kind_id());
        assert_eq!(orig.byte_range(), copy.byte_range());
    }
}

#[test]
fn node_is_copy() {
    let tree = leaf(5, 0, 3);
    let n1 = tree.root_node();
    let n2 = n1; // Copy
    assert_eq!(n1.kind_id(), n2.kind_id());
    assert_eq!(n1.byte_range(), n2.byte_range());
}

#[test]
fn tree_debug_format() {
    let tree = leaf(0, 0, 5);
    let debug = format!("{:?}", tree);
    assert!(debug.contains("Tree"));
    assert!(debug.contains("Node"));
}

#[test]
fn node_debug_format() {
    let tree = leaf(0, 0, 5);
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("Node"));
    assert!(debug.contains("range"));
}

// ===================================================================
// 14. Point type
// ===================================================================

#[test]
fn point_new_const() {
    const P: Point = Point::new(3, 7);
    assert_eq!(P.row, 3);
    assert_eq!(P.column, 7);
}

#[test]
fn point_clone_and_copy() {
    let p1 = Point::new(1, 2);
    let p2 = p1; // Copy
    let p3 = p1.clone();
    assert_eq!(p1, p2);
    assert_eq!(p1, p3);
}

#[test]
fn point_equality() {
    assert_eq!(Point::new(0, 0), Point::new(0, 0));
    assert_ne!(Point::new(0, 0), Point::new(0, 1));
    assert_ne!(Point::new(1, 0), Point::new(0, 0));
}

#[test]
fn point_ordering() {
    assert!(Point::new(0, 0) < Point::new(0, 1));
    assert!(Point::new(0, 1) < Point::new(1, 0));
    assert!(Point::new(1, 0) < Point::new(1, 1));
}

#[test]
fn point_display_format() {
    let p = Point::new(2, 5);
    // Display adds 1 to both row and column for 1-based display
    assert_eq!(format!("{}", p), "3:6");
}

#[test]
fn point_debug_format() {
    let p = Point::new(0, 0);
    let debug = format!("{:?}", p);
    assert!(debug.contains("Point"));
}

// ===================================================================
// 15. utf8_text
// ===================================================================

#[test]
fn utf8_text_extracts_slice() {
    let source = b"hello world";
    let tree = leaf(0, 6, 11);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "world");
}

#[test]
fn utf8_text_full_range() {
    let source = b"abc";
    let tree = leaf(0, 0, 3);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "abc");
}

#[test]
fn utf8_text_empty_range() {
    let source = b"abc";
    let tree = leaf(0, 1, 1);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "");
}

#[test]
fn utf8_text_invalid_utf8() {
    let source: &[u8] = &[0xff, 0xfe, 0xfd];
    let tree = leaf(0, 0, 3);
    assert!(tree.root_node().utf8_text(source).is_err());
}

// ===================================================================
// 16. Language integration
// ===================================================================

#[test]
fn children_kind_resolves_without_language() {
    let tree = branch(2, 0, 5, vec![leaf(4, 0, 2), leaf(1, 2, 5)]);
    // Without language, all nodes resolve to "unknown"
    assert_eq!(tree.root_node().kind(), "unknown");
    assert_eq!(tree.root_node().child(0).unwrap().kind(), "unknown");
    assert_eq!(tree.root_node().child(1).unwrap().kind(), "unknown");
}

#[test]
fn language_is_none_for_testing_trees() {
    let tree = Tree::new_for_testing(0, 0, 1, vec![]);
    assert!(tree.language().is_none());
}

// ===================================================================
// 17. Mixed deep + wide
// ===================================================================

#[test]
fn mixed_deep_wide_tree() {
    // Root has 3 children; second child has 2 grandchildren
    let gc1 = leaf(10, 3, 5);
    let gc2 = leaf(11, 5, 7);
    let c1 = leaf(1, 0, 3);
    let c2 = branch(2, 3, 7, vec![gc1, gc2]);
    let c3 = leaf(3, 7, 10);
    let tree = branch(0, 0, 10, vec![c1, c2, c3]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    let mid = root.child(1).unwrap();
    assert_eq!(mid.child_count(), 2);
    assert_eq!(mid.child(0).unwrap().kind_id(), 10);
    assert_eq!(mid.child(1).unwrap().kind_id(), 11);
}

#[test]
fn cursor_navigates_mixed_tree() {
    let gc = leaf(10, 0, 1);
    let c1 = branch(1, 0, 1, vec![gc]);
    let c2 = leaf(2, 1, 2);
    let tree = branch(0, 0, 2, vec![c1, c2]);

    let mut cursor = TreeCursor::new(&tree);
    // Go into first child then into grandchild
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 10);
    assert_eq!(cursor.depth(), 2);
    // Can't go deeper
    assert!(!cursor.goto_first_child());
    // Back to parent, then sibling
    assert!(cursor.goto_parent());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
    assert_eq!(cursor.depth(), 1);
}

// ===================================================================
// 18. Edge cases
// ===================================================================

#[test]
fn zero_length_byte_range() {
    let tree = leaf(0, 5, 5);
    assert_eq!(tree.root_node().start_byte(), 5);
    assert_eq!(tree.root_node().end_byte(), 5);
    assert_eq!(tree.root_node().byte_range(), 5..5);
}

#[test]
fn large_symbol_id() {
    let tree = Tree::new_for_testing(u32::MAX, 0, 1, vec![]);
    assert_eq!(tree.root_kind(), u32::MAX);
    // kind_id truncates to u16
    assert_eq!(tree.root_node().kind_id(), u16::MAX);
}

#[test]
fn large_byte_ranges() {
    let big = usize::MAX / 2;
    let tree = leaf(0, big, big + 100);
    assert_eq!(tree.root_node().start_byte(), big);
    assert_eq!(tree.root_node().end_byte(), big + 100);
}

#[test]
fn cursor_reset_to_different_tree() {
    let tree1 = leaf(1, 0, 1);
    let tree2 = leaf(2, 0, 1);
    let mut cursor = TreeCursor::new(&tree1);
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.reset(&tree2);
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn cursor_sibling_at_root_returns_false() {
    let tree = simple_expr_tree();
    let cursor = TreeCursor::new(&tree);
    let mut c = cursor;
    assert!(!c.goto_next_sibling());
}

#[test]
fn tree_with_empty_children_vec() {
    let tree = Tree::new_for_testing(0, 0, 10, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
    assert!(tree.root_node().child(0).is_none());
}
