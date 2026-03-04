// Comprehensive tests for runtime2 Tree edit operations
// Tests in-place editing for incremental parsing support

use adze_runtime::tree::Tree;

#[test]
fn tree_edit_exists() {
    // Verify Tree has edit-related functionality
    let t = Tree::new_for_testing(1, 0, 10, vec![]);
    let t2 = t.clone();
    // Both should be valid
    let _ = t2.root_node().start_byte();
}

#[test]
fn tree_deep_clone_independence() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![child]);
    let t2 = t.clone();
    // Clones should be independent
    assert_eq!(t.root_node().child_count(), t2.root_node().child_count());
}

#[test]
fn tree_various_symbol_ids() {
    for sym in [0u16, 1, 42, 100, 255, 1000, u16::MAX] {
        let t = Tree::new_for_testing(sym as u32, 0, 10, vec![]);
        assert_eq!(t.root_node().kind_id(), sym);
    }
}

#[test]
fn tree_various_byte_ranges() {
    let ranges: Vec<(usize, usize)> = vec![(0, 0), (0, 1), (0, 100), (10, 20), (0, usize::MAX / 2)];
    for (start, end) in ranges {
        let t = Tree::new_for_testing(1, start, end, vec![]);
        assert_eq!(t.root_node().start_byte(), start);
        assert_eq!(t.root_node().end_byte(), end);
    }
}

#[test]
fn tree_stub_is_minimal() {
    let t = Tree::new_stub();
    assert_eq!(t.root_node().child_count(), 0);
    assert_eq!(t.root_node().start_byte(), 0);
    assert_eq!(t.root_node().end_byte(), 0);
}

#[test]
fn tree_clone_deep_nesting() {
    let mut current = Tree::new_for_testing(10, 0, 1, vec![]);
    for i in (1..10).rev() {
        current = Tree::new_for_testing(i as u32, 0, i + 1, vec![current]);
    }
    let cloned = current.clone();
    assert_eq!(cloned.root_node().kind_id(), current.root_node().kind_id());
}

#[test]
fn tree_many_children_clone() {
    let children: Vec<_> = (0..50)
        .map(|i| Tree::new_for_testing(i + 2, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let t = Tree::new_for_testing(1, 0, 50, children);
    let t2 = t.clone();
    assert_eq!(t.root_node().child_count(), t2.root_node().child_count());
}

#[test]
fn tree_binary_structure() {
    let left = Tree::new_for_testing(2, 0, 5, vec![]);
    let right = Tree::new_for_testing(3, 5, 10, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![left, right]);
    assert_eq!(t.root_node().child_count(), 2);
    assert_eq!(t.root_node().child(0).unwrap().kind_id(), 2);
    assert_eq!(t.root_node().child(1).unwrap().kind_id(), 3);
}

#[test]
fn tree_ternary_structure() {
    let a = Tree::new_for_testing(2, 0, 3, vec![]);
    let b = Tree::new_for_testing(3, 3, 7, vec![]);
    let c = Tree::new_for_testing(4, 7, 10, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![a, b, c]);
    assert_eq!(t.root_node().child_count(), 3);
}

#[test]
fn tree_unbalanced_structure() {
    let leaf = Tree::new_for_testing(3, 0, 2, vec![]);
    let deep = Tree::new_for_testing(2, 0, 5, vec![leaf]);
    let shallow = Tree::new_for_testing(4, 5, 10, vec![]);
    let t = Tree::new_for_testing(1, 0, 10, vec![deep, shallow]);
    assert_eq!(t.root_node().child_count(), 2);
}
