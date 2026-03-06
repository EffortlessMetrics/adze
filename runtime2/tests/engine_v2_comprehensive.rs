//! Comprehensive tests for adze-runtime engine/tree/node/cursor public API.

use adze_runtime::Tree;
use adze_runtime::node::Point;
use adze_runtime::tree::TreeCursor;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a simple tree: root(symbol=1, 0..20) -> [child(2, 0..10), child(4, 10..20)]
fn two_child_tree() -> Tree {
    let c0 = Tree::new_for_testing(2, 0, 10, vec![]);
    let c1 = Tree::new_for_testing(4, 10, 20, vec![]);
    Tree::new_for_testing(1, 0, 20, vec![c0, c1])
}

/// Build a nested tree: root(1,0..30) -> child(2,0..15) -> grandchild(3,5..10)
fn nested_tree() -> Tree {
    let grandchild = Tree::new_for_testing(3, 5, 10, vec![]);
    let child = Tree::new_for_testing(2, 0, 15, vec![grandchild]);
    Tree::new_for_testing(1, 0, 30, vec![child])
}

/// Build a wider tree with three children.
fn three_child_tree() -> Tree {
    let c0 = Tree::new_for_testing(10, 0, 5, vec![]);
    let c1 = Tree::new_for_testing(11, 5, 10, vec![]);
    let c2 = Tree::new_for_testing(12, 10, 15, vec![]);
    Tree::new_for_testing(1, 0, 15, vec![c0, c1, c2])
}

// ===========================================================================
// 1. Tree construction patterns (10 tests)
// ===========================================================================

#[test]
fn test_stub_tree_has_zero_symbol() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn test_stub_tree_has_zero_byte_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn test_stub_tree_has_no_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn test_new_for_testing_sets_symbol() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
    assert_eq!(tree.root_node().kind_id(), 42);
}

#[test]
fn test_new_for_testing_sets_byte_range() {
    let tree = Tree::new_for_testing(1, 5, 25, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 25);
    assert_eq!(root.byte_range(), 5..25);
}

#[test]
fn test_new_for_testing_with_one_child() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let tree = Tree::new_for_testing(1, 0, 10, vec![child]);
    assert_eq!(tree.root_node().child_count(), 1);
    let c = tree.root_node().child(0).unwrap();
    assert_eq!(c.kind_id(), 2);
}

#[test]
fn test_new_for_testing_with_multiple_children() {
    let tree = two_child_tree();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().kind_id(), 2);
    assert_eq!(root.child(1).unwrap().kind_id(), 4);
}

#[test]
fn test_new_for_testing_nested_children_preserved() {
    let tree = nested_tree();
    let root = tree.root_node();
    let child = root.child(0).unwrap();
    assert_eq!(child.kind_id(), 2);
    assert_eq!(child.child_count(), 1);
    let grandchild = child.child(0).unwrap();
    assert_eq!(grandchild.kind_id(), 3);
    assert_eq!(grandchild.byte_range(), 5..10);
}

#[test]
fn test_new_for_testing_empty_children_vec() {
    let tree = Tree::new_for_testing(99, 0, 0, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn test_root_kind_returns_u32_symbol() {
    let tree = Tree::new_for_testing(0xFFFF, 0, 1, vec![]);
    assert_eq!(tree.root_kind(), 0xFFFF);
}

// ===========================================================================
// 2. Node properties (8 tests)
// ===========================================================================

#[test]
fn test_node_kind_without_language_returns_unknown() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn test_node_kind_id_matches_symbol() {
    let tree = Tree::new_for_testing(7, 0, 5, vec![]);
    assert_eq!(tree.root_node().kind_id(), 7u16);
}

#[test]
fn test_node_byte_range_matches_construction() {
    let tree = Tree::new_for_testing(1, 3, 17, vec![]);
    assert_eq!(tree.root_node().byte_range(), 3..17);
}

#[test]
fn test_node_start_byte() {
    let tree = Tree::new_for_testing(1, 10, 20, vec![]);
    assert_eq!(tree.root_node().start_byte(), 10);
}

#[test]
fn test_node_end_byte() {
    let tree = Tree::new_for_testing(1, 10, 20, vec![]);
    assert_eq!(tree.root_node().end_byte(), 20);
}

#[test]
fn test_node_is_named_returns_true() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().is_named());
}

#[test]
fn test_node_is_missing_returns_false() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_missing());
}

#[test]
fn test_node_is_error_returns_false() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_error());
}

// ===========================================================================
// 3. Point / position types (8 tests)
// ===========================================================================

#[test]
fn test_point_new() {
    let p = Point::new(5, 12);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 12);
}

#[test]
fn test_point_fields_direct_construction() {
    let p = Point { row: 3, column: 7 };
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

#[test]
fn test_point_equality() {
    assert_eq!(Point::new(1, 2), Point::new(1, 2));
    assert_ne!(Point::new(1, 2), Point::new(1, 3));
    assert_ne!(Point::new(0, 0), Point::new(1, 0));
}

#[test]
fn test_point_ordering() {
    // Derived Ord compares fields in declaration order: (row, column)
    assert!(Point::new(0, 0) < Point::new(0, 1));
    assert!(Point::new(0, 9) < Point::new(1, 0));
    assert!(Point::new(2, 5) > Point::new(2, 4));
}

#[test]
fn test_point_display_is_one_indexed() {
    let p = Point::new(0, 0);
    assert_eq!(format!("{p}"), "1:1");
    let p2 = Point::new(3, 7);
    assert_eq!(format!("{p2}"), "4:8");
}

#[test]
fn test_point_clone() {
    let p = Point::new(9, 4);
    let p2 = p;
    assert_eq!(p, p2);
}

#[test]
fn test_node_start_position_is_zero_in_phase_1() {
    let tree = Tree::new_for_testing(1, 10, 20, vec![]);
    let pos = tree.root_node().start_position();
    assert_eq!(pos, Point::new(0, 0));
}

#[test]
fn test_node_end_position_is_zero_in_phase_1() {
    let tree = Tree::new_for_testing(1, 10, 20, vec![]);
    let pos = tree.root_node().end_position();
    assert_eq!(pos, Point::new(0, 0));
}

// ===========================================================================
// 4. Tree navigation (8 tests)
// ===========================================================================

#[test]
fn test_child_by_index_returns_correct_node() {
    let tree = three_child_tree();
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().kind_id(), 10);
    assert_eq!(root.child(1).unwrap().kind_id(), 11);
    assert_eq!(root.child(2).unwrap().kind_id(), 12);
}

#[test]
fn test_child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child(0).is_none());
    let tree2 = two_child_tree();
    assert!(tree2.root_node().child(99).is_none());
}

#[test]
fn test_named_child_same_as_child_in_phase_1() {
    let tree = two_child_tree();
    let root = tree.root_node();
    assert_eq!(
        root.named_child(0).map(|n| n.kind_id()),
        root.child(0).map(|n| n.kind_id()),
    );
}

#[test]
fn test_child_by_field_name_returns_none() {
    let tree = two_child_tree();
    assert!(tree.root_node().child_by_field_name("name").is_none());
}

#[test]
fn test_parent_returns_none() {
    let tree = nested_tree();
    let child = tree.root_node().child(0).unwrap();
    assert!(child.parent().is_none());
}

#[test]
fn test_next_sibling_returns_none() {
    let tree = two_child_tree();
    let c0 = tree.root_node().child(0).unwrap();
    assert!(c0.next_sibling().is_none());
}

#[test]
fn test_prev_sibling_returns_none() {
    let tree = two_child_tree();
    let c1 = tree.root_node().child(1).unwrap();
    assert!(c1.prev_sibling().is_none());
}

#[test]
fn test_cursor_full_depth_first_traversal() {
    // root(1) -> [child(10,0..5), child(11,5..10)]
    let tree = Tree::new_for_testing(
        1,
        0,
        10,
        vec![
            Tree::new_for_testing(10, 0, 5, vec![]),
            Tree::new_for_testing(11, 5, 10, vec![]),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);

    // At root
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);

    // First child
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 10);
    assert_eq!(cursor.depth(), 1);

    // Next sibling
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 11);

    // No more siblings
    assert!(!cursor.goto_next_sibling());

    // Back to root
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

// ===========================================================================
// 5. Tree Debug / Clone (5 tests)
// ===========================================================================

#[test]
fn test_tree_debug_contains_node_info() {
    let tree = Tree::new_for_testing(5, 0, 10, vec![]);
    let dbg = format!("{tree:?}");
    assert!(dbg.contains("Tree"), "Debug output: {dbg}");
    assert!(dbg.contains("Node"), "Debug output: {dbg}");
}

#[test]
fn test_tree_clone_is_independent() {
    let tree = two_child_tree();
    let cloned = tree.clone();
    // Both roots should report the same structure.
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
}

#[test]
fn test_node_debug_format() {
    let tree = Tree::new_for_testing(7, 3, 15, vec![]);
    let dbg = format!("{:?}", tree.root_node());
    // Node Debug: "Node { kind: unknown, range: 3..15 }"
    assert!(dbg.contains("Node"), "Debug output: {dbg}");
    assert!(dbg.contains("3..15"), "Debug output: {dbg}");
}

#[test]
fn test_point_debug_format() {
    let p = Point::new(2, 8);
    let dbg = format!("{p:?}");
    assert!(dbg.contains("row: 2"), "Debug output: {dbg}");
    assert!(dbg.contains("column: 8"), "Debug output: {dbg}");
}

#[test]
fn test_tree_clone_preserves_nested_children() {
    let tree = nested_tree();
    let cloned = tree.clone();
    let gc = cloned.root_node().child(0).unwrap().child(0).unwrap();
    assert_eq!(gc.kind_id(), 3);
    assert_eq!(gc.byte_range(), 5..10);
}

// ===========================================================================
// 6. Forest / language / source integration (5 tests)
// ===========================================================================

#[test]
fn test_stub_tree_language_is_none() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn test_stub_tree_source_bytes_is_none() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn test_new_for_testing_language_is_none() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.language().is_none());
}

#[test]
fn test_node_child_count_zero_for_leaf() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn test_node_named_child_count_equals_child_count_phase_1() {
    let tree = three_child_tree();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

// ===========================================================================
// 7. Text extraction / utf8_text (5 tests)
// ===========================================================================

#[test]
fn test_utf8_text_extracts_correct_slice() {
    let source = b"hello world";
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "hello");
}

#[test]
fn test_utf8_text_on_child_node() {
    let source = b"hello world";
    let child = Tree::new_for_testing(2, 6, 11, vec![]);
    let tree = Tree::new_for_testing(1, 0, 11, vec![child]);
    let c = tree.root_node().child(0).unwrap();
    assert_eq!(c.utf8_text(source).unwrap(), "world");
}

#[test]
fn test_utf8_text_empty_range() {
    let source = b"hello";
    let tree = Tree::new_for_testing(1, 3, 3, vec![]);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "");
}

#[test]
fn test_utf8_text_full_source() {
    let source = b"fn main() {}";
    let tree = Tree::new_for_testing(1, 0, source.len(), vec![]);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "fn main() {}");
}

#[test]
fn test_utf8_text_multibyte_utf8() {
    let source = "café".as_bytes(); // é is 2 bytes
    let tree = Tree::new_for_testing(1, 0, source.len(), vec![]);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "café");
}

// ===========================================================================
// 8. Edge cases (6 tests)
// ===========================================================================

#[test]
fn test_deeply_nested_tree() {
    // Build 10 levels deep
    let mut current = Tree::new_for_testing(10, 0, 1, vec![]);
    for sym in (0u32..10).rev() {
        current = Tree::new_for_testing(sym, 0, 1, vec![current]);
    }
    // Walk down to the deepest node via cursor
    let mut cursor = TreeCursor::new(&current);
    for expected_sym in 0u16..=10 {
        assert_eq!(cursor.node().kind_id(), expected_sym);
        if expected_sym < 10 {
            assert!(cursor.goto_first_child());
        }
    }
    assert_eq!(cursor.depth(), 10);
}

#[test]
fn test_tree_with_many_children() {
    let children: Vec<Tree> = (0u32..50)
        .map(|i| Tree::new_for_testing(i + 100, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 50, children);
    assert_eq!(tree.root_node().child_count(), 50);
    // Spot check first and last
    assert_eq!(tree.root_node().child(0).unwrap().kind_id(), 100);
    assert_eq!(tree.root_node().child(49).unwrap().kind_id(), 149);
}

#[test]
fn test_kind_id_u16_range() {
    // kind_id() returns u16 — verify large symbol values within u16 range
    let tree = Tree::new_for_testing(u16::MAX as u32, 0, 1, vec![]);
    assert_eq!(tree.root_node().kind_id(), u16::MAX);
}

#[test]
fn test_zero_length_byte_range() {
    let tree = Tree::new_for_testing(1, 5, 5, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.byte_range(), 5..5);
}

#[test]
fn test_cursor_at_root_depth_is_zero() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_reset_returns_to_root() {
    let tree = nested_tree();
    let mut cursor = TreeCursor::new(&tree);
    // Navigate down
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
    // Reset
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 1);
}

// ===========================================================================
// Bonus: additional coverage (4 tests)
// ===========================================================================

#[test]
fn test_cursor_goto_parent_at_root_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn test_cursor_goto_first_child_on_leaf_returns_false() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn test_cursor_goto_next_sibling_at_root_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn test_next_named_sibling_returns_none() {
    let tree = two_child_tree();
    let c = tree.root_node().child(0).unwrap();
    assert!(c.next_named_sibling().is_none());
    assert!(c.prev_named_sibling().is_none());
}
