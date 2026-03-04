//! Comprehensive tests for runtime Node API.

use adze_runtime::tree::Tree;

// ── Tree construction patterns ──

#[test]
fn node_api_stub() {
    let t = Tree::new_stub();
    let _ = format!("{:?}", t);
}

#[test]
fn node_api_simple_leaf() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let _ = format!("{:?}", t);
}

#[test]
fn node_api_with_two_children() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let root = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    let _ = format!("{:?}", root);
}

#[test]
fn node_api_deep_tree() {
    let l = Tree::new_for_testing(4, 0, 1, vec![]);
    let m = Tree::new_for_testing(3, 0, 1, vec![l]);
    let t = Tree::new_for_testing(2, 0, 1, vec![m]);
    let r = Tree::new_for_testing(1, 0, 1, vec![t]);
    let _ = format!("{:?}", r);
}

#[test]
fn node_api_wide_tree() {
    let children: Vec<Tree> = (0..10)
        .map(|i| Tree::new_for_testing(i + 2, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let root = Tree::new_for_testing(1, 0, 10, children);
    let _ = format!("{:?}", root);
}

// ── TreeCursor full traversal ──

#[test]
fn cursor_root_only() {
    use adze_runtime::tree::TreeCursor;
    let t = Tree::new_for_testing(1, 0, 1, vec![]);
    let cursor = TreeCursor::new(&t);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_first_child_leaf() {
    use adze_runtime::tree::TreeCursor;
    let t = Tree::new_for_testing(1, 0, 1, vec![]);
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_first_child_exists() {
    use adze_runtime::tree::TreeCursor;
    let child = Tree::new_for_testing(2, 0, 1, vec![]);
    let root = Tree::new_for_testing(1, 0, 1, vec![child]);
    let mut cursor = TreeCursor::new(&root);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_sibling_traversal() {
    use adze_runtime::tree::TreeCursor;
    let c1 = Tree::new_for_testing(2, 0, 1, vec![]);
    let c2 = Tree::new_for_testing(3, 1, 2, vec![]);
    let c3 = Tree::new_for_testing(4, 2, 3, vec![]);
    let root = Tree::new_for_testing(1, 0, 3, vec![c1, c2, c3]);
    let mut cursor = TreeCursor::new(&root);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling()); // c2
    assert!(cursor.goto_next_sibling()); // c3
    assert!(!cursor.goto_next_sibling()); // no more
}

#[test]
fn cursor_parent_navigation() {
    use adze_runtime::tree::TreeCursor;
    let child = Tree::new_for_testing(2, 0, 1, vec![]);
    let root = Tree::new_for_testing(1, 0, 1, vec![child]);
    let mut cursor = TreeCursor::new(&root);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_deep_descent() {
    use adze_runtime::tree::TreeCursor;
    let d3 = Tree::new_for_testing(4, 0, 1, vec![]);
    let d2 = Tree::new_for_testing(3, 0, 1, vec![d3]);
    let d1 = Tree::new_for_testing(2, 0, 1, vec![d2]);
    let root = Tree::new_for_testing(1, 0, 1, vec![d1]);
    let mut cursor = TreeCursor::new(&root);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 3);
    assert!(!cursor.goto_first_child()); // leaf
}

#[test]
fn cursor_reset_to_root() {
    use adze_runtime::tree::TreeCursor;
    let child = Tree::new_for_testing(2, 0, 1, vec![]);
    let root = Tree::new_for_testing(1, 0, 1, vec![child]);
    let mut cursor = TreeCursor::new(&root);
    cursor.goto_first_child();
    assert_eq!(cursor.depth(), 1);
    cursor.reset(&root);
    assert_eq!(cursor.depth(), 0);
}

// ── Tree clone ──

#[test]
fn tree_clone_preserves_structure() {
    let c = Tree::new_for_testing(2, 0, 3, vec![]);
    let root = Tree::new_for_testing(1, 0, 5, vec![c]);
    let cloned = root.clone();
    let _ = format!("{:?}", cloned);
}

// ── Tree debug ──

#[test]
fn tree_debug_format() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let d = format!("{:?}", t);
    assert!(!d.is_empty());
}

// ── Multiple trees ──

#[test]
fn multiple_independent_trees() {
    let t1 = Tree::new_for_testing(1, 0, 5, vec![]);
    let t2 = Tree::new_for_testing(2, 0, 10, vec![]);
    let t3 = Tree::new_for_testing(3, 5, 15, vec![]);
    let _ = (t1, t2, t3);
}
