//! Comprehensive v2 tests for TreeCursor navigation.

use adze_runtime::tree::{Tree, TreeCursor};

// ── Cursor creation ──

#[test]
fn cursor_from_stub_v2() {
    let t = Tree::new_stub();
    let c = TreeCursor::new(&t);
    assert_eq!(c.depth(), 0);
}

#[test]
fn cursor_from_testing_tree_v2() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let c = TreeCursor::new(&t);
    assert_eq!(c.depth(), 0);
}

// ── Single node navigation ──

#[test]
fn single_no_first_child_v2() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut c = TreeCursor::new(&t);
    assert!(!c.goto_first_child());
}

#[test]
fn single_no_next_sibling_v2() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut c = TreeCursor::new(&t);
    assert!(!c.goto_next_sibling());
}

#[test]
fn single_no_parent_v2() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut c = TreeCursor::new(&t);
    assert!(!c.goto_parent());
}

// ── Two-level ──

#[test]
fn goto_first_child_v2() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let parent = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut c = TreeCursor::new(&parent);
    assert!(c.goto_first_child());
}

#[test]
fn depth_increases_on_child_v2() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let parent = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    assert_eq!(c.depth(), 1);
}

#[test]
fn goto_parent_after_child_v2() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let parent = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    assert!(c.goto_parent());
    assert_eq!(c.depth(), 0);
}

// ── Sibling navigation ──

#[test]
fn sibling_nav_v2() {
    let c1 = Tree::new_for_testing(2, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(3, 2, 5, vec![]);
    let parent = Tree::new_for_testing(1, 0, 5, vec![c1, c2]);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    assert!(c.goto_next_sibling());
}

#[test]
fn no_more_siblings_v2() {
    let c1 = Tree::new_for_testing(2, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(3, 2, 5, vec![]);
    let parent = Tree::new_for_testing(1, 0, 5, vec![c1, c2]);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    c.goto_next_sibling();
    assert!(!c.goto_next_sibling());
}

#[test]
fn three_siblings_v2() {
    let c1 = Tree::new_for_testing(2, 0, 1, vec![]);
    let c2 = Tree::new_for_testing(3, 1, 2, vec![]);
    let c3 = Tree::new_for_testing(4, 2, 3, vec![]);
    let parent = Tree::new_for_testing(1, 0, 3, vec![c1, c2, c3]);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    assert!(c.goto_next_sibling());
    assert!(c.goto_next_sibling());
    assert!(!c.goto_next_sibling());
}

// ── Deep traversal ──

#[test]
fn depth_three_v2() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 1, vec![leaf]);
    let root = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let mut c = TreeCursor::new(&root);
    c.goto_first_child();
    c.goto_first_child();
    assert_eq!(c.depth(), 2);
}

#[test]
fn return_from_depth_v2() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 1, vec![leaf]);
    let root = Tree::new_for_testing(1, 0, 1, vec![mid]);
    let mut c = TreeCursor::new(&root);
    c.goto_first_child();
    c.goto_first_child();
    c.goto_parent();
    c.goto_parent();
    assert_eq!(c.depth(), 0);
}

// ── Reset ──

#[test]
fn reset_to_root_v2() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let parent = Tree::new_for_testing(1, 0, 3, vec![child]);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    c.reset(&parent);
    assert_eq!(c.depth(), 0);
}

// ── Count children traversal ──

#[test]
fn five_children_v2() {
    let children: Vec<Tree> = (0..5)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let parent = Tree::new_for_testing(1, 0, 5, children);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    let mut count = 1;
    while c.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 5);
}

#[test]
fn ten_children_v2() {
    let children: Vec<Tree> = (0..10)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let parent = Tree::new_for_testing(1, 0, 10, children);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    let mut count = 1;
    while c.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 10);
}

#[test]
fn twenty_children_v2() {
    let children: Vec<Tree> = (0..20)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let parent = Tree::new_for_testing(1, 0, 20, children);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    let mut count = 1;
    while c.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 20);
}

// ── Edge cases ──

#[test]
fn stub_no_navigation_v2() {
    let t = Tree::new_stub();
    let mut c = TreeCursor::new(&t);
    assert!(!c.goto_first_child());
    assert!(!c.goto_next_sibling());
    assert!(!c.goto_parent());
}

#[test]
fn repeated_parent_at_root_v2() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let mut c = TreeCursor::new(&t);
    for _ in 0..5 {
        assert!(!c.goto_parent());
    }
    assert_eq!(c.depth(), 0);
}

#[test]
fn leaf_no_children_v2() {
    let leaf = Tree::new_for_testing(2, 0, 1, vec![]);
    let parent = Tree::new_for_testing(1, 0, 1, vec![leaf]);
    let mut c = TreeCursor::new(&parent);
    c.goto_first_child();
    assert!(!c.goto_first_child());
}

// ── Deep tree (depth 5) ──

#[test]
fn depth_five_v2() {
    let mut t = Tree::new_for_testing(5, 0, 1, vec![]);
    for i in (1..5).rev() {
        t = Tree::new_for_testing(i, 0, 1, vec![t]);
    }
    let mut c = TreeCursor::new(&t);
    let mut depth = 0;
    while c.goto_first_child() {
        depth += 1;
    }
    assert_eq!(depth, 4);
}
