//! Comprehensive tests for the GLRTreeCursor API.
//!
//! Tests cursor navigation (goto_first_child, goto_next_sibling, goto_parent),
//! node introspection (kind, byte ranges), DFS traversal, reset, independence
//! of multiple cursors, deep/wide trees, and field name resolution.

use adze::adze_ir as ir;
use adze::glr_tree_bridge::{GLRTree, GLRTreeCursor};
use adze::subtree::{ChildEdge, Subtree, SubtreeNode};

use ir::{FieldId, Grammar, SymbolId};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a leaf subtree (no children).
fn leaf(sym: u16, start: usize, end: usize) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: start..end,
        },
        vec![],
    ))
}

/// Build an internal subtree whose children carry no field info.
fn branch(sym: u16, start: usize, end: usize, children: Vec<Arc<Subtree>>) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: start..end,
        },
        children,
    ))
}

/// Build an internal subtree with field-annotated children.
fn branch_with_fields(
    sym: u16,
    start: usize,
    end: usize,
    children: Vec<ChildEdge>,
) -> Arc<Subtree> {
    Arc::new(Subtree::new_with_fields(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: start..end,
        },
        children,
    ))
}

/// Create a minimal grammar that maps symbol IDs to names.
fn grammar_with(names: &[(u16, &str)]) -> Grammar {
    let mut g = Grammar::new("test".to_string());
    for &(id, name) in names {
        g.rule_names.insert(SymbolId(id), name.to_string());
    }
    g
}

/// Build a `GLRTree` from a root subtree, source, and grammar.
fn make_tree(root: Arc<Subtree>, source: &[u8], grammar: Grammar) -> GLRTree {
    GLRTree::new(root, source.to_vec(), grammar)
}

/// Collect all node symbols via DFS using the cursor, returning symbols
/// in pre-order.
fn dfs_symbols(cursor: &mut GLRTreeCursor<'_>) -> Vec<u16> {
    let mut symbols = Vec::new();
    dfs_collect(cursor, &mut symbols);
    symbols
}

fn dfs_collect(cursor: &mut GLRTreeCursor<'_>, out: &mut Vec<u16>) {
    out.push(cursor.node().symbol());
    if cursor.goto_first_child() {
        loop {
            dfs_collect(cursor, out);
            if !cursor.goto_next_sibling() {
                // goto_next_sibling pops the current node on failure,
                // leaving the cursor at the parent already.
                break;
            }
        }
    }
}

/// Collect pre-order symbols via recursive node.children() iterator.
fn node_dfs_symbols(node: &adze::glr_tree_bridge::GLRNode<'_>) -> Vec<u16> {
    let mut out = vec![node.symbol()];
    for child in node.children() {
        out.extend(node_dfs_symbols(&child));
    }
    out
}

// ---------------------------------------------------------------------------
// Simple tree used by many tests:
//
//       root(1) [0..20]
//      /          \
//  left(2) [0..10]   right(3) [10..20]
//    |
// deep(4) [0..5]
// ---------------------------------------------------------------------------

fn simple_tree() -> GLRTree {
    let deep = leaf(4, 0, 5);
    let left = branch(2, 0, 10, vec![deep]);
    let right = leaf(3, 10, 20);
    let root = branch(1, 0, 20, vec![left, right]);
    let grammar = grammar_with(&[(1, "root"), (2, "left"), (3, "right"), (4, "deep")]);
    make_tree(root, b"01234567890123456789", grammar)
}

// ===========================================================================
// 1. Cursor starts at root node
// ===========================================================================

#[test]
fn cursor_starts_at_root() {
    let tree = simple_tree();
    let cursor = tree.root_node().walk();
    assert_eq!(cursor.node().kind(), "root");
    assert_eq!(cursor.node().symbol(), 1);
}

// ===========================================================================
// 2. goto_first_child moves to first child
// ===========================================================================

#[test]
fn goto_first_child_moves_to_first_child() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "left");
}

// ===========================================================================
// 3. goto_next_sibling moves to next sibling
// ===========================================================================

#[test]
fn goto_next_sibling_moves_to_next_sibling() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "right");
}

// ===========================================================================
// 4. goto_parent moves to parent
// ===========================================================================

#[test]
fn goto_parent_moves_to_parent() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "root");
}

// ===========================================================================
// 5. goto_first_child returns false at leaf
// ===========================================================================

#[test]
fn goto_first_child_returns_false_at_leaf() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    // Navigate to "deep" leaf: root -> left -> deep
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "deep");
    assert!(!cursor.goto_first_child());
}

// ===========================================================================
// 6. goto_next_sibling returns false at last sibling
// ===========================================================================

#[test]
fn goto_next_sibling_returns_false_at_last_sibling() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child()); // left
    assert!(cursor.goto_next_sibling()); // right
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

// ===========================================================================
// 7. goto_parent returns false at root
// ===========================================================================

#[test]
fn goto_parent_returns_false_at_root() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "root");
}

// ===========================================================================
// 8. Cursor correctly reports node kind at each position
// ===========================================================================

#[test]
fn cursor_reports_correct_kind_at_each_position() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.node().kind(), "root");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "left");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "deep");

    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "left");

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "right");
}

// ===========================================================================
// 9. Cursor correctly reports byte range at each position
// ===========================================================================

#[test]
fn cursor_reports_correct_byte_range() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.node().start_byte(), 0);
    assert_eq!(cursor.node().end_byte(), 20);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().byte_range(), 0..10);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().byte_range(), 0..5);

    assert!(cursor.goto_parent());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().byte_range(), 10..20);
}

// ===========================================================================
// 10. Full DFS traversal using cursor matches node iteration
// ===========================================================================

#[test]
fn dfs_traversal_matches_node_iteration() {
    let tree = simple_tree();
    let root = tree.root_node();

    let mut cursor = root.walk();
    let cursor_order = dfs_symbols(&mut cursor);
    let iter_order = node_dfs_symbols(&root);

    assert_eq!(cursor_order, iter_order);
    // Also assert the exact expected pre-order
    assert_eq!(cursor_order, vec![1, 2, 4, 3]);
}

// ===========================================================================
// 11. Cursor reset puts cursor back at root
// ===========================================================================

#[test]
fn reset_puts_cursor_back_at_root() {
    let tree = simple_tree();
    let root = tree.root_node();
    let mut cursor = root.walk();

    // Navigate deep
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "deep");

    // Reset
    cursor.reset(root.clone());
    assert_eq!(cursor.node().kind(), "root");
    assert!(!cursor.goto_parent()); // at root again
}

// ===========================================================================
// 12. Multiple cursors on same tree are independent
// ===========================================================================

#[test]
fn multiple_cursors_are_independent() {
    let tree = simple_tree();
    let root = tree.root_node();

    let mut c1 = root.walk();
    let mut c2 = root.walk();

    // Move c1 to left child
    assert!(c1.goto_first_child());
    assert_eq!(c1.node().kind(), "left");

    // c2 should still be at root
    assert_eq!(c2.node().kind(), "root");

    // Move c2 to right child
    assert!(c2.goto_first_child());
    assert!(c2.goto_next_sibling());
    assert_eq!(c2.node().kind(), "right");

    // c1 unchanged
    assert_eq!(c1.node().kind(), "left");
}

// ===========================================================================
// 13. Cursor on deeply nested tree (100+ levels)
// ===========================================================================

#[test]
fn cursor_on_deeply_nested_tree() {
    let depth = 150;
    let mut names: Vec<(u16, &str)> = Vec::new();
    // Build a chain: node_0 -> node_1 -> ... -> node_depth (leaf)
    // We'll reuse a static name via grammar; symbols are 1..=depth+1
    let mut current = leaf(depth as u16 + 1, 0, 1);
    for i in (1..=depth as u16).rev() {
        current = branch(i, 0, 1, vec![current]);
    }
    // Grammar: just map 1 -> "n" for simplicity
    for i in 1..=depth as u16 + 1 {
        names.push((i, "n"));
    }
    let grammar = grammar_with(&names);
    let tree = make_tree(current, b"x", grammar);

    let mut cursor = tree.root_node().walk();

    // Descend all the way to the leaf
    let mut levels_down = 0;
    while cursor.goto_first_child() {
        levels_down += 1;
    }
    assert_eq!(levels_down, depth);
    assert_eq!(cursor.node().symbol(), depth as u16 + 1);

    // Ascend all the way back to root
    let mut levels_up = 0;
    while cursor.goto_parent() {
        levels_up += 1;
    }
    assert_eq!(levels_up, depth);
    assert_eq!(cursor.node().symbol(), 1);
}

// ===========================================================================
// 14. Cursor on wide tree (100+ children)
// ===========================================================================

#[test]
fn cursor_on_wide_tree() {
    let width = 150;
    let children: Vec<Arc<Subtree>> = (0..width).map(|i| leaf(i as u16 + 10, i, i + 1)).collect();
    let root = branch(1, 0, width, children);
    let mut names = vec![(1u16, "root")];
    for i in 0..width {
        names.push((i as u16 + 10, "child"));
    }
    let grammar = grammar_with(&names);
    let tree = make_tree(root, &vec![b'x'; width], grammar);

    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child());

    let mut count = 1; // first child
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, width);

    // After the last goto_next_sibling fails the cursor is already at root
    // (the implementation pops the current node on failure).
    assert_eq!(cursor.node().kind(), "root");
}

// ===========================================================================
// 15. Cursor field_name returns None (not implemented on cursor)
//     but child_by_field_name works on nodes with field info
// ===========================================================================

#[test]
fn cursor_field_name_returns_none() {
    let left = leaf(2, 0, 5);
    let right = leaf(3, 5, 10);
    let root = branch_with_fields(
        1,
        0,
        10,
        vec![ChildEdge::new(left, 0), ChildEdge::new(right, 1)],
    );
    let mut grammar = grammar_with(&[(1, "root"), (2, "left_val"), (3, "right_val")]);
    grammar.fields.insert(FieldId(0), "lhs".to_string());
    grammar.fields.insert(FieldId(1), "rhs".to_string());
    let tree = make_tree(root, b"0123456789", grammar);

    let mut cursor = tree.root_node().walk();
    // Cursor-level field_name is not implemented
    assert!(cursor.field_name().is_none());
    assert!(cursor.goto_first_child());
    assert!(cursor.field_name().is_none());
}

#[test]
fn node_child_by_field_name_resolves_fields() {
    let left = leaf(2, 0, 5);
    let right = leaf(3, 5, 10);
    let root = branch_with_fields(
        1,
        0,
        10,
        vec![ChildEdge::new(left, 0), ChildEdge::new(right, 1)],
    );
    let mut grammar = grammar_with(&[(1, "root"), (2, "left_val"), (3, "right_val")]);
    grammar.fields.insert(FieldId(0), "lhs".to_string());
    grammar.fields.insert(FieldId(1), "rhs".to_string());
    let tree = make_tree(root, b"0123456789", grammar);

    let root_node = tree.root_node();
    let lhs = root_node.child_by_field_name("lhs").unwrap();
    assert_eq!(lhs.kind(), "left_val");
    let rhs = root_node.child_by_field_name("rhs").unwrap();
    assert_eq!(rhs.kind(), "right_val");
    assert!(root_node.child_by_field_name("nonexistent").is_none());
}

// ===========================================================================
// Additional tests to reach 25+
// ===========================================================================

// 16. goto_parent after goto_next_sibling returns to parent
#[test]
fn goto_parent_after_sibling_returns_to_parent() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child()); // left
    assert!(cursor.goto_next_sibling()); // right
    assert!(cursor.goto_parent()); // root
    assert_eq!(cursor.node().kind(), "root");
}

// 17. Repeated goto_parent at root stays at root
#[test]
fn repeated_goto_parent_at_root_stays_at_root() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(!cursor.goto_parent());
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "root");
}

// 18. goto_next_sibling at root returns false
#[test]
fn goto_next_sibling_at_root_returns_false() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    assert!(!cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "root");
}

// 19. Cursor on single-node tree (root with no children)
#[test]
fn cursor_on_single_node_tree() {
    let root = leaf(1, 0, 5);
    let grammar = grammar_with(&[(1, "lonely")]);
    let tree = make_tree(root, b"hello", grammar);

    let mut cursor = tree.root_node().walk();
    assert_eq!(cursor.node().kind(), "lonely");
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
}

// 20. Round-trip: descend then ascend returns to same node
#[test]
fn round_trip_descend_ascend() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    let root_id = cursor.node().id();

    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child()); // deep
    assert!(cursor.goto_parent());
    assert!(cursor.goto_parent());

    assert_eq!(cursor.node().id(), root_id);
}

// 21. Cursor node() returns correct symbol after complex navigation
#[test]
fn complex_navigation_sequence() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();

    // root -> left -> deep -> (back to left) -> right
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().symbol(), 2); // left
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().symbol(), 4); // deep
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().symbol(), 2); // left
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().symbol(), 3); // right
    assert!(!cursor.goto_first_child()); // right is a leaf
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().symbol(), 1); // root
}

// 22. DFS on a balanced binary tree
#[test]
fn dfs_balanced_binary_tree() {
    //       1
    //      / \
    //     2   3
    //    / \ / \
    //   4  5 6  7
    let n4 = leaf(4, 0, 1);
    let n5 = leaf(5, 1, 2);
    let n6 = leaf(6, 2, 3);
    let n7 = leaf(7, 3, 4);
    let n2 = branch(2, 0, 2, vec![n4, n5]);
    let n3 = branch(3, 2, 4, vec![n6, n7]);
    let root = branch(1, 0, 4, vec![n2, n3]);

    let grammar = grammar_with(&[
        (1, "root"),
        (2, "a"),
        (3, "b"),
        (4, "c"),
        (5, "d"),
        (6, "e"),
        (7, "f"),
    ]);
    let tree = make_tree(root, b"abcd", grammar);

    let mut cursor = tree.root_node().walk();
    let symbols = dfs_symbols(&mut cursor);
    assert_eq!(symbols, vec![1, 2, 4, 5, 3, 6, 7]);
}

// 23. reset to a non-root node
#[test]
fn reset_to_non_root_node() {
    let tree = simple_tree();
    let root = tree.root_node();
    let left_child = root.child(0).unwrap();

    let mut cursor = root.walk();
    cursor.reset(left_child.clone());
    assert_eq!(cursor.node().kind(), "left");

    // After reset, parent should return false (left is now the cursor root)
    assert!(!cursor.goto_parent());

    // Can still descend
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "deep");
}

// 24. Cursor child_count visible through node at each level
#[test]
fn node_child_count_at_each_level() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.node().child_count(), 2); // root has 2 children

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().child_count(), 1); // left has 1 child

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().child_count(), 0); // deep is a leaf

    assert!(cursor.goto_parent());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().child_count(), 0); // right is a leaf
}

// 25. Cursor traversal on tree with error nodes
#[test]
fn cursor_with_error_nodes() {
    let good = leaf(2, 0, 3);
    let bad = Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(3),
            is_error: true,
            byte_range: 3..6,
        },
        vec![],
    ));
    let root = branch(1, 0, 6, vec![good, bad]);
    let grammar = grammar_with(&[(1, "root"), (2, "ok"), (3, "ERROR")]);
    let tree = make_tree(root, b"abcdef", grammar);

    let mut cursor = tree.root_node().walk();
    assert!(cursor.node().has_error());

    assert!(cursor.goto_first_child());
    assert!(!cursor.node().is_error());

    assert!(cursor.goto_next_sibling());
    assert!(cursor.node().is_error());
    assert_eq!(cursor.node().byte_range(), 3..6);
}

// 26. utf8_text through cursor node
#[test]
fn cursor_node_utf8_text() {
    let child = leaf(2, 6, 11);
    let root = branch(1, 0, 11, vec![child]);
    let grammar = grammar_with(&[(1, "root"), (2, "word")]);
    let source = b"hello world";
    let tree = make_tree(root, source, grammar);

    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().utf8_text(tree.text()).unwrap(), "world");
}

// 27. Iterate all siblings and verify order
#[test]
fn iterate_all_siblings_in_order() {
    let c1 = leaf(10, 0, 1);
    let c2 = leaf(20, 1, 2);
    let c3 = leaf(30, 2, 3);
    let c4 = leaf(40, 3, 4);
    let root = branch(1, 0, 4, vec![c1, c2, c3, c4]);
    let grammar = grammar_with(&[(1, "root"), (10, "a"), (20, "b"), (30, "c"), (40, "d")]);
    let tree = make_tree(root, b"abcd", grammar);

    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child());

    let mut kinds = vec![cursor.node().kind().to_string()];
    while cursor.goto_next_sibling() {
        kinds.push(cursor.node().kind().to_string());
    }
    assert_eq!(kinds, vec!["a", "b", "c", "d"]);
}

// 28. Cursor walk created from non-root node
#[test]
fn walk_from_non_root_node() {
    let tree = simple_tree();
    let left = tree.root_node().child(0).unwrap();
    let mut cursor = left.walk();

    assert_eq!(cursor.node().kind(), "left");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "deep");
    // goto_parent succeeds (back to left), then fails at cursor root
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "left");
    assert!(!cursor.goto_parent());
}

#[test]
fn walk_from_non_root_node_parent_boundary() {
    let tree = simple_tree();
    let left = tree.root_node().child(0).unwrap();
    let mut cursor = left.walk();

    assert_eq!(cursor.node().kind(), "left");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "deep");

    // Can go back to the cursor root (left)
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "left");

    // Cannot go above the cursor root
    assert!(!cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "left");
}

// 29. DFS traversal count matches total node count
#[test]
fn dfs_count_matches_total_nodes() {
    let tree = simple_tree();
    let mut cursor = tree.root_node().walk();
    let symbols = dfs_symbols(&mut cursor);
    // root(1), left(2), deep(4), right(3) = 4 nodes
    assert_eq!(symbols.len(), 4);
}

// 30. Cursor on tree where root has exactly one child
#[test]
fn cursor_root_with_single_child() {
    let child = leaf(2, 0, 5);
    let root = branch(1, 0, 5, vec![child]);
    let grammar = grammar_with(&[(1, "wrapper"), (2, "inner")]);
    let tree = make_tree(root, b"hello", grammar);

    let mut cursor = tree.root_node().walk();
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "inner");
    // goto_next_sibling fails and pops inner, leaving cursor at root
    assert!(!cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "wrapper");
}
