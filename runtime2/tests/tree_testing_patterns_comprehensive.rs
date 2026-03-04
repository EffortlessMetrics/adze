//! Comprehensive tests for Tree::new_for_testing patterns and behaviors.

use adze_runtime::tree::Tree;

#[test]
fn testing_leaf_node() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_two_children() {
    let c1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(3, 3, 6, vec![]);
    let root = Tree::new_for_testing(1, 0, 6, vec![c1, c2]);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_three_children() {
    let children: Vec<Tree> = (0..3)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let root = Tree::new_for_testing(1, 0, 3, children);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_depth_one() {
    let leaf = Tree::new_for_testing(2, 0, 1, vec![]);
    let root = Tree::new_for_testing(1, 0, 1, vec![leaf]);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_depth_three() {
    let l3 = Tree::new_for_testing(4, 0, 1, vec![]);
    let l2 = Tree::new_for_testing(3, 0, 1, vec![l3]);
    let l1 = Tree::new_for_testing(2, 0, 1, vec![l2]);
    let root = Tree::new_for_testing(1, 0, 1, vec![l1]);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_depth_five() {
    let mut t = Tree::new_for_testing(6, 0, 1, vec![]);
    for kind in (1..6).rev() {
        t = Tree::new_for_testing(kind, 0, 1, vec![t]);
    }
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_wide_ten() {
    let children: Vec<Tree> = (0..10)
        .map(|i| Tree::new_for_testing(i + 100, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let root = Tree::new_for_testing(1, 0, 10, children);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_wide_twenty() {
    let children: Vec<Tree> = (0..20)
        .map(|i| Tree::new_for_testing(i + 100, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let root = Tree::new_for_testing(1, 0, 20, children);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_kind_zero() {
    let t = Tree::new_for_testing(0, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 0);
}

#[test]
fn testing_kind_one() {
    let t = Tree::new_for_testing(1, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_kind_large() {
    let t = Tree::new_for_testing(50000, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 50000);
}

#[test]
fn testing_kind_max() {
    let t = Tree::new_for_testing(u32::MAX, 0, 1, vec![]);
    assert_eq!(t.root_kind(), u32::MAX);
}

#[test]
fn testing_clone_leaf() {
    let t = Tree::new_for_testing(42, 0, 10, vec![]);
    let c = t.clone();
    assert_eq!(c.root_kind(), 42);
}

#[test]
fn testing_clone_with_children() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![child]);
    let c = t.clone();
    assert_eq!(c.root_kind(), 1);
}

#[test]
fn testing_empty_byte_range() {
    let t = Tree::new_for_testing(1, 0, 0, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_large_byte_range() {
    let t = Tree::new_for_testing(1, 0, 1_000_000, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_nonzero_start() {
    let t = Tree::new_for_testing(1, 500, 1000, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_multiple_independent() {
    let t1 = Tree::new_for_testing(1, 0, 5, vec![]);
    let t2 = Tree::new_for_testing(2, 0, 10, vec![]);
    let t3 = Tree::new_for_testing(3, 5, 15, vec![]);
    assert_eq!(t1.root_kind(), 1);
    assert_eq!(t2.root_kind(), 2);
    assert_eq!(t3.root_kind(), 3);
}

#[test]
fn testing_balanced_binary() {
    let ll = Tree::new_for_testing(4, 0, 1, vec![]);
    let lr = Tree::new_for_testing(5, 1, 2, vec![]);
    let rl = Tree::new_for_testing(6, 2, 3, vec![]);
    let rr = Tree::new_for_testing(7, 3, 4, vec![]);
    let left = Tree::new_for_testing(2, 0, 2, vec![ll, lr]);
    let right = Tree::new_for_testing(3, 2, 4, vec![rl, rr]);
    let root = Tree::new_for_testing(1, 0, 4, vec![left, right]);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_left_skew() {
    let d = Tree::new_for_testing(5, 0, 1, vec![]);
    let c = Tree::new_for_testing(4, 0, 1, vec![d]);
    let b = Tree::new_for_testing(3, 0, 1, vec![c]);
    let a = Tree::new_for_testing(2, 0, 1, vec![b]);
    let root = Tree::new_for_testing(1, 0, 1, vec![a]);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_right_skew() {
    let d = Tree::new_for_testing(5, 0, 1, vec![]);
    let c = Tree::new_for_testing(4, 0, 1, vec![d]);
    let root = Tree::new_for_testing(1, 0, 1, vec![c]);
    assert_eq!(root.root_kind(), 1);
}
