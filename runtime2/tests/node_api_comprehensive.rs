#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the Node API in adze-runtime (runtime2).
//!
//! Test areas:
//!  1. Tree construction patterns
//!  2. Tree root_kind access
//!  3. Tree clone behavior
//!  4. TreeCursor depth tracking
//!  5. TreeCursor child navigation
//!  6. TreeCursor sibling navigation
//!  7. TreeCursor parent navigation
//!  8. TreeCursor reset
//!  9. Deep tree traversal
//! 10. Wide tree traversal (many children)

use adze_runtime::tree::{Tree, TreeCursor};

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
// 1. Tree construction patterns
// ===========================================================================

#[test]
fn stub_tree_root_kind_is_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn stub_tree_byte_range_empty() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn stub_tree_has_no_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn leaf_construction() {
    let tree = leaf(7, 10, 20);
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 7);
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 20);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn branch_construction_with_children() {
    let tree = branch(
        0,
        0,
        15,
        vec![leaf(1, 0, 5), leaf(2, 5, 10), leaf(3, 10, 15)],
    );
    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    assert_eq!(root.child(0).unwrap().kind_id(), 1);
    assert_eq!(root.child(1).unwrap().kind_id(), 2);
    assert_eq!(root.child(2).unwrap().kind_id(), 3);
}

#[test]
fn nested_construction_preserves_grandchildren() {
    let grandchild = leaf(3, 0, 2);
    let child = branch(2, 0, 2, vec![grandchild]);
    let tree = branch(1, 0, 2, vec![child]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let c = root.child(0).unwrap();
    assert_eq!(c.kind_id(), 2);
    assert_eq!(c.child_count(), 1);
    let gc = c.child(0).unwrap();
    assert_eq!(gc.kind_id(), 3);
    assert_eq!(gc.child_count(), 0);
}

#[test]
fn new_for_testing_flattens_children() {
    let inner = branch(2, 5, 10, vec![leaf(3, 5, 7), leaf(4, 7, 10)]);
    let tree = Tree::new_for_testing(1, 0, 15, vec![leaf(5, 0, 5), inner]);

    let root = tree.root_node();
    assert_eq!(root.kind_id(), 1);
    assert_eq!(root.child_count(), 2);
    let c1 = root.child(1).unwrap();
    assert_eq!(c1.kind_id(), 2);
    assert_eq!(c1.child_count(), 2);
    assert_eq!(c1.child(0).unwrap().kind_id(), 3);
    assert_eq!(c1.child(1).unwrap().kind_id(), 4);
}

#[test]
fn stub_equivalent_to_empty_for_testing() {
    let stub = Tree::new_stub();
    let manual = Tree::new_for_testing(0, 0, 0, vec![]);
    assert_eq!(stub.root_node().kind_id(), manual.root_node().kind_id());
    assert_eq!(
        stub.root_node().start_byte(),
        manual.root_node().start_byte()
    );
    assert_eq!(stub.root_node().end_byte(), manual.root_node().end_byte());
    assert_eq!(
        stub.root_node().child_count(),
        manual.root_node().child_count()
    );
}

#[test]
fn zero_width_node() {
    let tree = leaf(1, 7, 7);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), root.end_byte());
    assert!(root.byte_range().is_empty());
}

// ===========================================================================
// 2. Tree root_kind access
// ===========================================================================

#[test]
fn root_kind_stub() {
    assert_eq!(Tree::new_stub().root_kind(), 0);
}

#[test]
fn root_kind_custom_symbol() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn root_kind_large_symbol() {
    let tree = Tree::new_for_testing(65535, 0, 1, vec![]);
    assert_eq!(tree.root_kind(), 65535);
}

#[test]
fn root_kind_matches_root_node_kind_id() {
    let tree = Tree::new_for_testing(99, 0, 5, vec![]);
    assert_eq!(tree.root_kind(), tree.root_node().kind_id() as u32);
}

#[test]
fn root_kind_unchanged_after_adding_children() {
    let tree = branch(50, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    assert_eq!(tree.root_kind(), 50);
}

// ===========================================================================
// 3. Tree clone behavior
// ===========================================================================

#[test]
fn clone_preserves_root_kind() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
}

#[test]
fn clone_preserves_children() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let cloned = tree.clone();
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
    assert_eq!(
        tree.root_node().child(0).unwrap().kind_id(),
        cloned.root_node().child(0).unwrap().kind_id()
    );
}

#[test]
fn clone_preserves_byte_ranges() {
    let tree = leaf(1, 10, 20);
    let cloned = tree.clone();
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
}

#[test]
fn clone_is_independent() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let _cloned = tree.clone();
    // Both exist independently without compiler errors
    assert_eq!(tree.root_kind(), 0);
    assert_eq!(_cloned.root_kind(), 0);
}

#[test]
fn clone_deep_tree() {
    let mut current = leaf(10, 0, 1);
    for sym in (0..5).rev() {
        current = branch(sym, 0, 1, vec![current]);
    }
    let cloned = current.clone();

    // Walk both trees and verify structure matches
    let mut orig_node = current.root_node();
    let mut clone_node = cloned.root_node();
    for _ in 0..5 {
        assert_eq!(orig_node.kind_id(), clone_node.kind_id());
        orig_node = orig_node.child(0).unwrap();
        clone_node = clone_node.child(0).unwrap();
    }
    assert_eq!(orig_node.kind_id(), clone_node.kind_id());
}

// ===========================================================================
// 4. TreeCursor depth tracking
// ===========================================================================

#[test]
fn cursor_depth_at_root_is_zero() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_depth_increments_on_child() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_depth_decrements_on_parent() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_parent();
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_depth_unchanged_on_sibling() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_depth_tracks_deep_traversal() {
    let tree = branch(
        0,
        0,
        1,
        vec![branch(1, 0, 1, vec![branch(2, 0, 1, vec![leaf(3, 0, 1)])])],
    );
    let mut cursor = TreeCursor::new(&tree);
    for expected_depth in 0..=3 {
        assert_eq!(cursor.depth(), expected_depth);
        if expected_depth < 3 {
            assert!(cursor.goto_first_child());
        }
    }
}

// ===========================================================================
// 5. TreeCursor child navigation
// ===========================================================================

#[test]
fn cursor_goto_first_child_on_leaf_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child_on_stub_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_goto_first_child_moves_to_first() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
}

#[test]
fn cursor_first_child_then_first_child() {
    let tree = branch(0, 0, 5, vec![branch(1, 0, 5, vec![leaf(2, 0, 5)])]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(!cursor.goto_first_child()); // leaf
}

// ===========================================================================
// 6. TreeCursor sibling navigation
// ===========================================================================

#[test]
fn cursor_goto_next_sibling_at_root_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_next_sibling_advances() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn cursor_goto_next_sibling_at_last_returns_false() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

#[test]
fn cursor_sibling_traversal_all_children() {
    let tree = branch(
        0,
        0,
        20,
        vec![
            leaf(10, 0, 5),
            leaf(20, 5, 10),
            leaf(30, 10, 15),
            leaf(40, 15, 20),
        ],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();

    let mut symbols = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        symbols.push(cursor.node().kind_id());
    }
    assert_eq!(symbols, vec![10, 20, 30, 40]);
}

#[test]
fn cursor_sibling_does_not_change_depth() {
    let tree = branch(
        0,
        0,
        15,
        vec![leaf(1, 0, 5), leaf(2, 5, 10), leaf(3, 10, 15)],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    let d = cursor.depth();
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), d);
    cursor.goto_next_sibling();
    assert_eq!(cursor.depth(), d);
}

// ===========================================================================
// 7. TreeCursor parent navigation
// ===========================================================================

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_goto_parent_returns_to_root() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_parent_from_deep() {
    let tree = branch(0, 0, 1, vec![branch(1, 0, 1, vec![leaf(2, 0, 1)])]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);

    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);

    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);

    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_parent_after_sibling_returns_to_correct_parent() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
}

// ===========================================================================
// 8. TreeCursor reset
// ===========================================================================

#[test]
fn cursor_reset_returns_to_root() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_reset_from_deep_position() {
    let tree = branch(
        0,
        0,
        1,
        vec![branch(1, 0, 1, vec![branch(2, 0, 1, vec![leaf(3, 0, 1)])])],
    );
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 3);
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn cursor_reset_to_different_tree() {
    let tree1 = branch(10, 0, 5, vec![leaf(11, 0, 5)]);
    let tree2 = branch(20, 0, 3, vec![leaf(21, 0, 3)]);
    let mut cursor = TreeCursor::new(&tree1);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 11);

    cursor.reset(&tree2);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 20);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 21);
}

#[test]
fn cursor_reset_allows_fresh_traversal() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);

    // First traversal
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);

    // Reset and traverse again
    cursor.reset(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);
}

// ===========================================================================
// 9. Deep tree traversal
// ===========================================================================

#[test]
fn deep_tree_cursor_walk_down() {
    let mut current = leaf(10, 0, 1);
    for sym in (0..10).rev() {
        current = branch(sym, 0, 1, vec![current]);
    }
    let mut cursor = TreeCursor::new(&current);
    for expected_sym in 0..=10 {
        assert_eq!(cursor.node().kind_id(), expected_sym as u16);
        if expected_sym < 10 {
            assert!(cursor.goto_first_child());
        }
    }
    assert_eq!(cursor.depth(), 10);
}

#[test]
fn deep_tree_cursor_walk_up() {
    let mut current = leaf(5, 0, 1);
    for sym in (0..5).rev() {
        current = branch(sym, 0, 1, vec![current]);
    }
    let mut cursor = TreeCursor::new(&current);
    // Walk down
    while cursor.goto_first_child() {}
    assert_eq!(cursor.depth(), 5);
    // Walk up
    while cursor.goto_parent() {}
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

#[test]
fn deep_tree_node_api_walk() {
    let mut current = leaf(10, 0, 1);
    for sym in (0..10).rev() {
        current = branch(sym, 0, 1, vec![current]);
    }
    let mut node = current.root_node();
    for expected_sym in 0..=10 {
        assert_eq!(node.kind_id(), expected_sym as u16);
        if expected_sym < 10 {
            assert_eq!(node.child_count(), 1);
            node = node.child(0).unwrap();
        } else {
            assert_eq!(node.child_count(), 0);
        }
    }
}

#[test]
fn deep_tree_mixed_depth_siblings() {
    // root has a deep child and a shallow leaf
    let deep = branch(1, 0, 3, vec![branch(2, 0, 2, vec![leaf(3, 0, 1)])]);
    let shallow = leaf(4, 3, 5);
    let tree = branch(0, 0, 5, vec![deep, shallow]);

    let mut cursor = TreeCursor::new(&tree);
    // Go into deep branch
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 1);
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind_id(), 2);
    assert_eq!(cursor.depth(), 2);

    // Go back up and to shallow sibling
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 4);
    assert_eq!(cursor.depth(), 1);
    assert!(!cursor.goto_first_child()); // leaf
}

// ===========================================================================
// 10. Wide tree traversal (many children)
// ===========================================================================

#[test]
fn wide_tree_50_children_cursor() {
    let children: Vec<Tree> = (0..50)
        .map(|i| leaf(i + 1, i as usize, (i + 1) as usize))
        .collect();
    let tree = branch(0, 0, 50, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();

    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 50);
}

#[test]
fn wide_tree_100_children_node_api() {
    let children: Vec<Tree> = (0..100).map(|i| leaf((i % 256) as u32, i, i + 1)).collect();
    let tree = branch(0, 0, 100, children);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 100);
    for i in 0..100 {
        let child = root.child(i).unwrap();
        assert_eq!(child.start_byte(), i);
        assert_eq!(child.end_byte(), i + 1);
    }
}

#[test]
fn wide_tree_cursor_symbols_in_order() {
    let children: Vec<Tree> = (1..=20).map(|i| leaf(i as u32, 0, 1)).collect();
    let tree = branch(0, 0, 1, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();

    let mut symbols = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        symbols.push(cursor.node().kind_id());
    }
    let expected: Vec<u16> = (1..=20).collect();
    assert_eq!(symbols, expected);
}

#[test]
fn wide_tree_depth_stays_one() {
    let children: Vec<Tree> = (0..30).map(|i| leaf(i, 0, 1)).collect();
    let tree = branch(0, 0, 1, children);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    while cursor.goto_next_sibling() {
        assert_eq!(cursor.depth(), 1);
    }
}

#[test]
fn wide_and_deep_tree() {
    // root -> 3 children, each with 3 grandchildren
    let mut children = vec![];
    for i in 0..3u32 {
        let grandchildren: Vec<Tree> = (0..3u32).map(|j| leaf(i * 10 + j + 1, 0, 1)).collect();
        children.push(branch(i + 1, 0, 3, grandchildren));
    }
    let tree = branch(0, 0, 3, children);

    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().kind_id(), 1);

    // Traverse all grandchildren of first child
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 2);
    let mut gc_syms = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        gc_syms.push(cursor.node().kind_id());
    }
    assert_eq!(gc_syms, vec![1, 2, 3]);

    // Go back up to first child, then to second child
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind_id(), 2);

    // Traverse grandchildren of second child
    cursor.goto_first_child();
    let mut gc_syms2 = vec![cursor.node().kind_id()];
    while cursor.goto_next_sibling() {
        gc_syms2.push(cursor.node().kind_id());
    }
    assert_eq!(gc_syms2, vec![11, 12, 13]);
}

// ===========================================================================
// Additional Node API tests
// ===========================================================================

#[test]
fn kind_returns_unknown_without_language() {
    let tree = leaf(42, 0, 5);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn kind_id_truncates_large_symbol_to_u16() {
    let tree = leaf(300, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 300u16);
}

#[test]
fn child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(usize::MAX).is_none());
}

#[test]
fn named_child_count_equals_child_count() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn child_by_field_name_always_none() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    assert!(tree.root_node().child_by_field_name("left").is_none());
    assert!(tree.root_node().child_by_field_name("").is_none());
}

#[test]
fn parent_always_none() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    assert!(tree.root_node().parent().is_none());
    assert!(tree.root_node().child(0).unwrap().parent().is_none());
}

#[test]
fn next_prev_sibling_always_none() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let c0 = tree.root_node().child(0).unwrap();
    assert!(c0.next_sibling().is_none());
    assert!(c0.prev_sibling().is_none());
    assert!(c0.next_named_sibling().is_none());
    assert!(c0.prev_named_sibling().is_none());
}

#[test]
fn is_named_always_true() {
    assert!(leaf(1, 0, 5).root_node().is_named());
}

#[test]
fn is_missing_always_false() {
    assert!(!leaf(1, 0, 5).root_node().is_missing());
}

#[test]
fn is_error_always_false() {
    assert!(!leaf(1, 0, 5).root_node().is_error());
}

#[test]
fn node_is_copy() {
    let tree = leaf(5, 0, 10);
    let a = tree.root_node();
    let b = a; // Copy
    assert_eq!(a.kind_id(), b.kind_id());
    assert_eq!(a.start_byte(), b.start_byte());
}

#[test]
fn utf8_text_extracts_slice() {
    let source = b"hello world";
    let tree = branch(0, 0, 11, vec![leaf(1, 0, 5), leaf(2, 6, 11)]);
    assert_eq!(
        tree.root_node()
            .child(0)
            .unwrap()
            .utf8_text(source)
            .unwrap(),
        "hello"
    );
    assert_eq!(
        tree.root_node()
            .child(1)
            .unwrap()
            .utf8_text(source)
            .unwrap(),
        "world"
    );
}

#[test]
fn utf8_text_empty_range() {
    let source = b"abc";
    let tree = leaf(1, 2, 2);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "");
}

#[test]
fn utf8_text_invalid_utf8() {
    let source: &[u8] = &[0xff, 0xfe, 0xfd];
    let tree = leaf(1, 0, 3);
    assert!(tree.root_node().utf8_text(source).is_err());
}

#[test]
fn utf8_text_multibyte() {
    let source = "café".as_bytes();
    let tree = leaf(1, 0, source.len());
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "café");
}

#[test]
fn debug_format_contains_range() {
    let tree = leaf(5, 10, 20);
    let dbg = format!("{:?}", tree.root_node());
    assert!(dbg.contains("10..20"));
}

#[test]
fn tree_debug_contains_tree() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let dbg = format!("{tree:?}");
    assert!(dbg.contains("Tree"));
}

#[test]
fn testing_tree_has_no_language() {
    assert!(Tree::new_for_testing(1, 0, 5, vec![]).language().is_none());
}

#[test]
fn testing_tree_has_no_source() {
    assert!(
        Tree::new_for_testing(1, 0, 5, vec![])
            .source_bytes()
            .is_none()
    );
}

#[test]
fn point_display_one_indexed() {
    let p = adze_runtime::Point::new(2, 5);
    assert_eq!(format!("{p}"), "3:6");
}

#[test]
fn point_ordering() {
    let a = adze_runtime::Point::new(0, 0);
    let b = adze_runtime::Point::new(0, 5);
    let c = adze_runtime::Point::new(1, 0);
    assert!(a < b);
    assert!(b < c);
}

#[test]
fn point_equality() {
    let p1 = adze_runtime::Point::new(1, 2);
    let p2 = adze_runtime::Point::new(1, 2);
    assert_eq!(p1, p2);
    assert_ne!(p1, adze_runtime::Point::new(1, 3));
}

#[test]
fn children_with_duplicate_symbols() {
    let tree = branch(0, 0, 9, vec![leaf(5, 0, 3), leaf(5, 3, 6), leaf(5, 6, 9)]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    for i in 0..3 {
        assert_eq!(root.child(i).unwrap().kind_id(), 5);
    }
    // Distinguished by byte range
    assert_eq!(root.child(0).unwrap().start_byte(), 0);
    assert_eq!(root.child(1).unwrap().start_byte(), 3);
    assert_eq!(root.child(2).unwrap().start_byte(), 6);
}

#[test]
fn large_byte_range() {
    let big = usize::MAX / 2;
    let tree = leaf(1, big, big + 100);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), big);
    assert_eq!(root.end_byte(), big + 100);
}

#[test]
fn byte_range_method_consistency() {
    let tree = leaf(5, 100, 200);
    let node = tree.root_node();
    assert_eq!(node.byte_range().start, node.start_byte());
    assert_eq!(node.byte_range().end, node.end_byte());
    assert_eq!(node.byte_range().len(), node.end_byte() - node.start_byte());
}

#[test]
fn cursor_full_dfs_traversal() {
    // Build: root(0) -> [child(1) -> [gc(3)], child(2)]
    let tree = branch(
        0,
        0,
        10,
        vec![branch(1, 0, 5, vec![leaf(3, 0, 5)]), leaf(2, 5, 10)],
    );
    let mut cursor = TreeCursor::new(&tree);
    let mut visited = vec![];

    // DFS traversal
    loop {
        visited.push(cursor.node().kind_id());
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                // done
                assert_eq!(visited, vec![0, 1, 3, 2]);
                return;
            }
        }
    }
}

#[test]
fn cursor_node_byte_ranges() {
    let tree = branch(0, 0, 20, vec![leaf(1, 0, 8), leaf(2, 8, 20)]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 20);

    cursor.goto_first_child();
    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 8);

    cursor.goto_next_sibling();
    assert_eq!(cursor.node().start_byte(), 8);
    assert_eq!(cursor.node().end_byte(), 20);
}
