// Comprehensive tests for runtime2 Tree node API
// Tests node-level operations on Tree structures

use adze_runtime::tree::Tree;

#[test]
fn root_node_kind_id() {
    let t = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(t.root_node().kind_id(), 42);
}

#[test]
fn root_node_start_byte() {
    let t = Tree::new_for_testing(1, 5, 15, vec![]);
    assert_eq!(t.root_node().start_byte(), 5);
}

#[test]
fn root_node_end_byte() {
    let t = Tree::new_for_testing(1, 5, 15, vec![]);
    assert_eq!(t.root_node().end_byte(), 15);
}

#[test]
fn root_node_child_count_no_children() {
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    assert_eq!(t.root_node().child_count(), 0);
}

#[test]
fn root_node_child_count_with_children() {
    let c1 = Tree::new_for_testing(2, 0, 5, vec![]);
    let c2 = Tree::new_for_testing(3, 5, 10, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![c1, c2]);
    assert_eq!(t.root_node().child_count(), 2);
}

#[test]
fn child_node_kind_id() {
    let child = Tree::new_for_testing(99, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![child]);
    let root = t.root_node();
    let c = root.child(0).unwrap();
    assert_eq!(c.kind_id(), 99);
}

#[test]
fn child_node_byte_range() {
    let child = Tree::new_for_testing(2, 3, 7, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![child]);
    let c = t.root_node().child(0).unwrap();
    assert_eq!(c.start_byte(), 3);
    assert_eq!(c.end_byte(), 7);
}

#[test]
fn child_out_of_bounds_returns_none() {
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    assert!(t.root_node().child(0).is_none());
}

#[test]
fn nested_children_accessible() {
    let leaf = Tree::new_for_testing(3, 0, 2, vec![]);
    let mid = Tree::new_for_testing(2, 0, 5, vec![leaf]);
    let root = Tree::new_for_testing(1, 0, 10, vec![mid]);
    let t = root;
    let mid_node = t.root_node().child(0).unwrap();
    let leaf_node = mid_node.child(0).unwrap();
    assert_eq!(leaf_node.kind_id(), 3);
}

#[test]
fn tree_with_many_children() {
    let children: Vec<_> = (0..10)
        .map(|i| Tree::new_for_testing(i + 2, i as usize * 5, (i as usize + 1) * 5, vec![]))
        .collect();
    let t = Tree::new_for_testing(1, 0, 50, children);
    assert_eq!(t.root_node().child_count(), 10);
}

#[test]
fn stub_tree_child_count() {
    let t = Tree::new_stub();
    assert_eq!(t.root_node().child_count(), 0);
}

#[test]
fn deep_nesting_5_levels() {
    let l4 = Tree::new_for_testing(5, 0, 1, vec![]);
    let l3 = Tree::new_for_testing(4, 0, 2, vec![l4]);
    let l2 = Tree::new_for_testing(3, 0, 3, vec![l3]);
    let l1 = Tree::new_for_testing(2, 0, 4, vec![l2]);
    let root = Tree::new_for_testing(1, 0, 5, vec![l1]);
    assert_eq!(root.root_node().child_count(), 1);
    let c1 = root.root_node().child(0).unwrap();
    assert_eq!(c1.child_count(), 1);
}
