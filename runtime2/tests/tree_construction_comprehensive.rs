//! Comprehensive tests for runtime2 Tree construction and properties.

use adze_runtime::tree::Tree;

// ── Tree::new_stub ──

#[test]
fn tree_new_stub_exists() {
    let t = Tree::new_stub();
    let _ = format!("{:?}", t);
}

#[test]
fn tree_new_stub_clone() {
    let t = Tree::new_stub();
    let c = t.clone();
    let _ = format!("{:?}", c);
}

// ── Tree::new_for_testing ──

#[test]
fn tree_new_for_testing_basic() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let _ = format!("{:?}", t);
}

#[test]
fn tree_new_for_testing_with_children() {
    let c1 = Tree::new_for_testing(2, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(3, 2, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![c1, c2]);
    let _ = format!("{:?}", t);
}

#[test]
fn tree_new_for_testing_nested() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 1, vec![leaf]);
    let root = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let _ = format!("{:?}", root);
}

#[test]
fn tree_new_for_testing_zero_range() {
    let t = Tree::new_for_testing(1, 0, 0, vec![]);
    let _ = format!("{:?}", t);
}

#[test]
fn tree_new_for_testing_large_range() {
    let t = Tree::new_for_testing(1, 0, 1_000_000, vec![]);
    let _ = format!("{:?}", t);
}

#[test]
fn tree_new_for_testing_symbol_zero() {
    let t = Tree::new_for_testing(0, 0, 1, vec![]);
    let _ = format!("{:?}", t);
}

#[test]
fn tree_new_for_testing_high_symbol() {
    let t = Tree::new_for_testing(65535, 0, 1, vec![]);
    let _ = format!("{:?}", t);
}

// ── Tree clone ──

#[test]
fn tree_clone_simple() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let c = t.clone();
    let _ = format!("{:?}", c);
}

#[test]
fn tree_clone_with_children() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![child]);
    let c = t.clone();
    let _ = format!("{:?}", c);
}

// ── Tree debug ──

#[test]
fn tree_debug_stub() {
    let t = Tree::new_stub();
    let d = format!("{:?}", t);
    assert!(!d.is_empty());
}

#[test]
fn tree_debug_testing() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let d = format!("{:?}", t);
    assert!(!d.is_empty());
}

// ── TreeCursor ──

#[test]
fn tree_cursor_new() {
    use adze_runtime::tree::TreeCursor;
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let _cursor = TreeCursor::new(&t);
}

#[test]
fn tree_cursor_depth_at_root() {
    use adze_runtime::tree::TreeCursor;
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let cursor = TreeCursor::new(&t);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn tree_cursor_goto_first_child_no_children() {
    use adze_runtime::tree::TreeCursor;
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_first_child());
}

#[test]
fn tree_cursor_goto_first_child_with_child() {
    use adze_runtime::tree::TreeCursor;
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn tree_cursor_goto_next_sibling() {
    use adze_runtime::tree::TreeCursor;
    let c1 = Tree::new_for_testing(2, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(3, 2, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![c1, c2]);
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling());
}

#[test]
fn tree_cursor_goto_parent() {
    use adze_runtime::tree::TreeCursor;
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn tree_cursor_goto_parent_at_root() {
    use adze_runtime::tree::TreeCursor;
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_parent());
}

#[test]
fn tree_cursor_reset() {
    use adze_runtime::tree::TreeCursor;
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![child]);
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    cursor.reset(&t);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn tree_cursor_deep_traversal() {
    use adze_runtime::tree::TreeCursor;
    let leaf = Tree::new_for_testing(4, 0, 1, vec![]);
    let mid = Tree::new_for_testing(3, 0, 1, vec![leaf]);
    let top = Tree::new_for_testing(2, 0, 1, vec![mid]);
    let root = Tree::new_for_testing(1, 0, 1, vec![top]);
    let mut cursor = TreeCursor::new(&root);
    assert!(cursor.goto_first_child()); // depth 1
    assert!(cursor.goto_first_child()); // depth 2
    assert!(cursor.goto_first_child()); // depth 3
    assert_eq!(cursor.depth(), 3);
}

#[test]
fn tree_cursor_sibling_traversal() {
    use adze_runtime::tree::TreeCursor;
    let c1 = Tree::new_for_testing(2, 0, 1, vec![]);
    let c2 = Tree::new_for_testing(3, 1, 2, vec![]);
    let c3 = Tree::new_for_testing(4, 2, 3, vec![]);
    let root = Tree::new_for_testing(1, 0, 3, vec![c1, c2, c3]);
    let mut cursor = TreeCursor::new(&root);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling());
    assert!(cursor.goto_next_sibling());
    assert!(!cursor.goto_next_sibling()); // no more siblings
}

// ── Many children ──

#[test]
fn tree_many_children() {
    let children: Vec<Tree> = (0..100)
        .map(|i| Tree::new_for_testing(i + 2, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let root = Tree::new_for_testing(1, 0, 100, children);
    let _ = format!("{:?}", root);
}

// ── Tree with mixed children ──

#[test]
fn tree_mixed_depth() {
    let deep_leaf = Tree::new_for_testing(5, 0, 1, vec![]);
    let deep_mid = Tree::new_for_testing(4, 0, 1, vec![deep_leaf]);
    let deep_top = Tree::new_for_testing(3, 0, 1, vec![deep_mid]);
    let shallow = Tree::new_for_testing(2, 1, 2, vec![]);
    let root = Tree::new_for_testing(1, 0, 2, vec![deep_top, shallow]);
    let _ = format!("{:?}", root);
}
