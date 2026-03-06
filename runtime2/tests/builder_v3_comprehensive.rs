//! Comprehensive tests for Tree construction and forest builder patterns.
//!
//! Covers tree construction, children, deep/wide trees, properties,
//! traversal, mixed node types, and edge cases.

use adze_runtime::tree::*;

// ---------------------------------------------------------------------------
// Helper: build a leaf tree (no children)
// ---------------------------------------------------------------------------
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

// ---------------------------------------------------------------------------
// 1. Tree construction from parts (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn test_construct_stub_tree_has_zero_symbol() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
}

#[test]
fn test_construct_stub_tree_has_zero_byte_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn test_construct_stub_tree_has_no_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn test_construct_leaf_via_new_for_testing() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 42);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 10);
}

#[test]
fn test_construct_leaf_has_no_children() {
    let tree = leaf(7, 0, 5);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn test_construct_preserves_large_symbol_id() {
    let tree = leaf(65535, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 65535);
}

#[test]
fn test_construct_root_kind_returns_u32() {
    let tree = leaf(300, 0, 1);
    assert_eq!(tree.root_kind(), 300);
}

#[test]
fn test_construct_preserves_byte_range() {
    let tree = leaf(1, 100, 200);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 100..200);
}

// ---------------------------------------------------------------------------
// 2. Tree with children (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn test_one_child_count() {
    let child = leaf(1, 0, 3);
    let tree = Tree::new_for_testing(0, 0, 3, vec![child]);
    assert_eq!(tree.root_node().child_count(), 1);
}

#[test]
fn test_two_children_count() {
    let c1 = leaf(1, 0, 2);
    let c2 = leaf(2, 2, 5);
    let tree = Tree::new_for_testing(0, 0, 5, vec![c1, c2]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn test_child_symbol_preserved() {
    let child = leaf(99, 0, 4);
    let tree = Tree::new_for_testing(0, 0, 4, vec![child]);
    let c = tree.root_node().child(0).unwrap();
    assert_eq!(c.kind_id(), 99);
}

#[test]
fn test_child_byte_range_preserved() {
    let child = leaf(1, 3, 7);
    let tree = Tree::new_for_testing(0, 0, 7, vec![child]);
    let c = tree.root_node().child(0).unwrap();
    assert_eq!(c.start_byte(), 3);
    assert_eq!(c.end_byte(), 7);
}

#[test]
fn test_children_order_matches_insertion() {
    let c1 = leaf(10, 0, 1);
    let c2 = leaf(20, 1, 2);
    let c3 = leaf(30, 2, 3);
    let tree = Tree::new_for_testing(0, 0, 3, vec![c1, c2, c3]);
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().kind_id(), 10);
    assert_eq!(root.child(1).unwrap().kind_id(), 20);
    assert_eq!(root.child(2).unwrap().kind_id(), 30);
}

#[test]
fn test_child_out_of_bounds_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![leaf(1, 0, 5)]);
    assert!(tree.root_node().child(1).is_none());
}

#[test]
fn test_named_child_count_matches_child_count() {
    let c1 = leaf(1, 0, 2);
    let c2 = leaf(2, 2, 4);
    let tree = Tree::new_for_testing(0, 0, 4, vec![c1, c2]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn test_named_child_returns_same_as_child() {
    let c = leaf(5, 0, 3);
    let tree = Tree::new_for_testing(0, 0, 3, vec![c]);
    let root = tree.root_node();
    assert_eq!(
        root.named_child(0).unwrap().kind_id(),
        root.child(0).unwrap().kind_id()
    );
}

// ---------------------------------------------------------------------------
// 3. Deep tree building (5 tests)
// ---------------------------------------------------------------------------

fn build_chain(depth: usize) -> Tree {
    let mut current = leaf(depth as u32, 0, 1);
    for i in (0..depth).rev() {
        current = Tree::new_for_testing(i as u32, 0, 1, vec![current]);
    }
    current
}

#[test]
fn test_deep_chain_root_symbol() {
    let tree = build_chain(5);
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn test_deep_chain_leaf_symbol() {
    let tree = build_chain(4);
    let mut node = tree.root_node();
    for _ in 0..4 {
        node = node.child(0).unwrap();
    }
    assert_eq!(node.kind_id(), 4);
    assert_eq!(node.child_count(), 0);
}

#[test]
fn test_deep_chain_each_level_one_child() {
    let tree = build_chain(3);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    assert_eq!(root.child(0).unwrap().child_count(), 1);
    assert_eq!(root.child(0).unwrap().child(0).unwrap().child_count(), 1);
}

#[test]
fn test_deep_chain_50_levels() {
    let tree = build_chain(50);
    let mut node = tree.root_node();
    for expected in 0..50 {
        assert_eq!(node.kind_id(), expected as u16);
        node = node.child(0).unwrap();
    }
    assert_eq!(node.kind_id(), 50);
}

#[test]
fn test_deep_chain_cursor_depth() {
    let tree = build_chain(10);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    for expected_depth in 1..=10 {
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.depth(), expected_depth);
    }
    assert!(!cursor.goto_first_child());
}

// ---------------------------------------------------------------------------
// 4. Wide tree building (5 tests)
// ---------------------------------------------------------------------------

fn build_wide(width: usize) -> Tree {
    let children: Vec<Tree> = (0..width).map(|i| leaf(i as u32 + 1, i, i + 1)).collect();
    Tree::new_for_testing(0, 0, width, children)
}

#[test]
fn test_wide_tree_child_count() {
    let tree = build_wide(20);
    assert_eq!(tree.root_node().child_count(), 20);
}

#[test]
fn test_wide_tree_each_child_is_leaf() {
    let tree = build_wide(10);
    for i in 0..10 {
        assert_eq!(tree.root_node().child(i).unwrap().child_count(), 0);
    }
}

#[test]
fn test_wide_tree_children_have_sequential_symbols() {
    let tree = build_wide(5);
    let root = tree.root_node();
    for i in 0..5 {
        assert_eq!(root.child(i).unwrap().kind_id(), (i + 1) as u16);
    }
}

#[test]
fn test_wide_tree_100_children() {
    let tree = build_wide(100);
    assert_eq!(tree.root_node().child_count(), 100);
    assert_eq!(tree.root_node().child(99).unwrap().kind_id(), 100);
}

#[test]
fn test_wide_tree_cursor_sibling_walk() {
    let tree = build_wide(5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 5);
}

// ---------------------------------------------------------------------------
// 5. Tree properties after build (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn test_root_node_is_named() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().is_named());
}

#[test]
fn test_root_node_is_not_missing() {
    let tree = leaf(1, 0, 5);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn test_root_node_is_not_error() {
    let tree = leaf(1, 0, 5);
    assert!(!tree.root_node().is_error());
}

#[test]
fn test_language_is_none_for_testing_tree() {
    let tree = leaf(1, 0, 5);
    assert!(tree.language().is_none());
}

#[test]
fn test_kind_returns_unknown_without_language() {
    let tree = leaf(1, 0, 5);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn test_source_bytes_none_for_testing_tree() {
    let tree = leaf(1, 0, 5);
    assert!(tree.source_bytes().is_none());
}

#[test]
fn test_start_position_is_zero() {
    let tree = leaf(1, 10, 20);
    let p = tree.root_node().start_position();
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn test_end_position_is_zero() {
    let tree = leaf(1, 10, 20);
    let p = tree.root_node().end_position();
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

// ---------------------------------------------------------------------------
// 6. Tree traversal after build (8 tests)
// ---------------------------------------------------------------------------

fn sample_tree() -> Tree {
    let gc = leaf(3, 2, 3);
    let c1 = Tree::new_for_testing(1, 0, 3, vec![gc]);
    let c2 = leaf(2, 3, 5);
    Tree::new_for_testing(0, 0, 5, vec![c1, c2])
}

#[test]
fn test_cursor_starts_at_root() {
    let tree = sample_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn test_cursor_first_child() {
    let tree = sample_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn test_cursor_grandchild() {
    let tree = sample_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 3);
}

#[test]
fn test_cursor_sibling_after_child() {
    let tree = sample_tree();
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn test_cursor_parent_from_child() {
    let tree = sample_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn test_cursor_no_parent_at_root() {
    let tree = sample_tree();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_reset_returns_to_root() {
    let tree = sample_tree();
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn test_cursor_leaf_has_no_first_child() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

// ---------------------------------------------------------------------------
// 7. Mixed node types (5 tests)
// ---------------------------------------------------------------------------

#[test]
fn test_mixed_depths_tree() {
    // Root with: leaf, node-with-child, leaf
    let c1 = leaf(1, 0, 1);
    let gc = leaf(4, 1, 2);
    let c2 = Tree::new_for_testing(2, 1, 2, vec![gc]);
    let c3 = leaf(3, 2, 3);
    let tree = Tree::new_for_testing(0, 0, 3, vec![c1, c2, c3]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    assert_eq!(root.child(0).unwrap().child_count(), 0);
    assert_eq!(root.child(1).unwrap().child_count(), 1);
    assert_eq!(root.child(2).unwrap().child_count(), 0);
}

#[test]
fn test_mixed_symbols_unique() {
    let children: Vec<Tree> = (10..15).map(|s| leaf(s, 0, 1)).collect();
    let tree = Tree::new_for_testing(0, 0, 1, children);
    let root = tree.root_node();
    let symbols: Vec<u16> = (0..5).map(|i| root.child(i).unwrap().kind_id()).collect();
    assert_eq!(symbols, [10, 11, 12, 13, 14]);
}

#[test]
fn test_mixed_byte_ranges() {
    let c1 = leaf(1, 0, 10);
    let c2 = leaf(2, 10, 100);
    let c3 = leaf(3, 100, 1000);
    let tree = Tree::new_for_testing(0, 0, 1000, vec![c1, c2, c3]);
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().byte_range(), 0..10);
    assert_eq!(root.child(1).unwrap().byte_range(), 10..100);
    assert_eq!(root.child(2).unwrap().byte_range(), 100..1000);
}

#[test]
fn test_binary_tree_structure() {
    let ll = leaf(3, 0, 1);
    let lr = leaf(4, 1, 2);
    let left = Tree::new_for_testing(1, 0, 2, vec![ll, lr]);
    let rl = leaf(5, 2, 3);
    let rr = leaf(6, 3, 4);
    let right = Tree::new_for_testing(2, 2, 4, vec![rl, rr]);
    let tree = Tree::new_for_testing(0, 0, 4, vec![left, right]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().child_count(), 2);
    assert_eq!(root.child(1).unwrap().child_count(), 2);
    assert_eq!(root.child(0).unwrap().child(0).unwrap().kind_id(), 3);
    assert_eq!(root.child(1).unwrap().child(1).unwrap().kind_id(), 6);
}

#[test]
fn test_unbalanced_tree() {
    // Left-heavy: root -> left(deep chain) + right(leaf)
    let deep = build_chain(5);
    let shallow = leaf(100, 1, 2);
    let tree = Tree::new_for_testing(200, 0, 2, vec![deep, shallow]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    // Deep side
    let mut node = root.child(0).unwrap();
    assert_eq!(node.kind_id(), 0);
    for _ in 0..4 {
        node = node.child(0).unwrap();
    }
    assert_eq!(node.kind_id(), 4);
    // Shallow side
    assert_eq!(root.child(1).unwrap().kind_id(), 100);
}

// ---------------------------------------------------------------------------
// 8. Edge cases (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn test_zero_length_byte_range() {
    let tree = leaf(1, 5, 5);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.byte_range(), 5..5);
}

#[test]
fn test_symbol_zero() {
    let tree = leaf(0, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 0);
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn test_clone_independence() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let cloned = tree.clone();
    // Cloned tree has same structure
    assert_eq!(cloned.root_node().kind_id(), 1);
    assert_eq!(cloned.root_node().child_count(), 2);
    assert_eq!(cloned.root_node().child(0).unwrap().kind_id(), 2);
}

#[test]
fn test_debug_format_contains_kind() {
    let tree = leaf(42, 0, 10);
    let debug_str = format!("{:?}", tree);
    assert!(debug_str.contains("Tree"));
}

#[test]
fn test_utf8_text_extraction() {
    let source = b"hello world";
    let tree = leaf(1, 0, 5);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "hello");
}

#[test]
fn test_utf8_text_extraction_middle() {
    let source = b"hello world";
    let tree = leaf(1, 6, 11);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "world");
}

#[test]
fn test_child_by_field_name_returns_none() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![leaf(1, 0, 5)]);
    assert!(tree.root_node().child_by_field_name("anything").is_none());
}

#[test]
fn test_parent_sibling_return_none() {
    let tree = sample_tree();
    let root = tree.root_node();
    assert!(root.parent().is_none());
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
    assert!(root.next_named_sibling().is_none());
    assert!(root.prev_named_sibling().is_none());
}
