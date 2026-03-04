// Comprehensive tests for runtime2 Tree API
// Tests Tree creation, editing, cloning, and cursor operations

use adze_runtime::tree::{Tree, TreeCursor};

#[test]
fn tree_stub_has_zero_root_byte_range() {
    let t = Tree::new_stub();
    assert_eq!(t.root_node().start_byte(), 0);
    assert_eq!(t.root_node().end_byte(), 0);
}

#[test]
fn tree_for_testing_basic() {
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    assert_eq!(t.root_node().start_byte(), 0);
    assert_eq!(t.root_node().end_byte(), 10);
}

#[test]
fn tree_for_testing_with_children() {
    let c1 = Tree::new_for_testing(2, 0, 5, vec![]);
    let c2 = Tree::new_for_testing(3, 5, 10, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![c1, c2]);
    assert_eq!(t.root_node().child_count(), 2);
}

#[test]
fn tree_for_testing_nested() {
    let leaf = Tree::new_for_testing(4, 0, 2, vec![]);
    let mid = Tree::new_for_testing(3, 0, 5, vec![leaf]);
    let root = Tree::new_for_testing(1, 0, 10, vec![mid]);
    assert_eq!(root.root_node().child_count(), 1);
}

#[test]
fn tree_root_node_kind_id() {
    let t = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(t.root_node().kind_id(), 42);
}

#[test]
fn tree_stub_debug() {
    let t = Tree::new_stub();
    let dbg = format!("{:?}", t);
    assert!(!dbg.is_empty());
}

#[test]
fn tree_clone_preserves_structure() {
    let c = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![c]);
    let t2 = t.clone();
    assert_eq!(t2.root_node().child_count(), t.root_node().child_count());
}

#[test]
fn cursor_new_from_tree() {
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    let _c = TreeCursor::new(&t);
}

#[test]
fn cursor_goto_first_child_with_children() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![child]);
    let mut c = TreeCursor::new(&t);
    assert!(c.goto_first_child());
}

#[test]
fn cursor_goto_first_child_no_children() {
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    let mut c = TreeCursor::new(&t);
    assert!(!c.goto_first_child());
}

#[test]
fn cursor_goto_next_sibling() {
    let c1 = Tree::new_for_testing(2, 0, 5, vec![]);
    let c2 = Tree::new_for_testing(3, 5, 10, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![c1, c2]);
    let mut cur = TreeCursor::new(&t);
    assert!(cur.goto_first_child());
    assert!(cur.goto_next_sibling());
}

#[test]
fn cursor_goto_parent() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![child]);
    let mut c = TreeCursor::new(&t);
    c.goto_first_child();
    assert!(c.goto_parent());
}

#[test]
fn cursor_depth_at_root() {
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    let c = TreeCursor::new(&t);
    assert_eq!(c.depth(), 0);
}

#[test]
fn cursor_depth_at_child() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![child]);
    let mut c = TreeCursor::new(&t);
    c.goto_first_child();
    assert_eq!(c.depth(), 1);
}

#[test]
fn cursor_traversal_all_children() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let c3 = Tree::new_for_testing(4, 6, 10, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![c1, c2, c3]);
    let mut cur = TreeCursor::new(&t);
    assert!(cur.goto_first_child());
    let mut count = 1;
    while cur.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 3);
}
