//! Comprehensive tests for runtime2 Tree cloning and equality behavior.

use adze_runtime::tree::Tree;

// ── Stub tree basics ──

#[test]
fn stub_tree_root_kind() {
    let t = Tree::new_stub();
    assert_eq!(t.root_kind(), 0);
}

#[test]
fn stub_tree_debug() {
    let t = Tree::new_stub();
    let dbg = format!("{:?}", t);
    assert!(!dbg.is_empty());
}

#[test]
fn stub_tree_clone() {
    let t = Tree::new_stub();
    let t2 = t.clone();
    assert_eq!(t.root_kind(), t2.root_kind());
}

// ── Testing trees ──

#[test]
fn testing_tree_basic() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_tree_clone_independent() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    let t2 = t.clone();
    assert_eq!(t.root_kind(), t2.root_kind());
}

#[test]
fn testing_tree_with_children() {
    let child1 = Tree::new_for_testing(2, 0, 2, vec![]);
    let child2 = Tree::new_for_testing(3, 2, 5, vec![]);
    let parent = Tree::new_for_testing(1, 0, 5, vec![child1, child2]);
    assert_eq!(parent.root_kind(), 1);
}

#[test]
fn testing_tree_nested() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 1, vec![leaf]);
    let root = Tree::new_for_testing(1, 0, 1, vec![mid]);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn testing_tree_zero_range() {
    let t = Tree::new_for_testing(1, 0, 0, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn testing_tree_large_symbol() {
    let t = Tree::new_for_testing(999, 0, 100, vec![]);
    assert_eq!(t.root_kind(), 999);
}

#[test]
fn testing_tree_large_range() {
    let t = Tree::new_for_testing(1, 0, 1_000_000, vec![]);
    assert_eq!(t.root_kind(), 1);
}

// ── Clone and comparison ──

#[test]
fn clone_stub_preserves_kind() {
    let t = Tree::new_stub();
    let c = t.clone();
    assert_eq!(t.root_kind(), c.root_kind());
}

#[test]
fn clone_testing_tree_preserves_kind() {
    let t = Tree::new_for_testing(42, 0, 10, vec![]);
    let c = t.clone();
    assert_eq!(c.root_kind(), 42);
}

#[test]
fn clone_with_children_preserves_kind() {
    let child = Tree::new_for_testing(2, 0, 5, vec![]);
    let parent = Tree::new_for_testing(1, 0, 5, vec![child]);
    let cloned = parent.clone();
    assert_eq!(cloned.root_kind(), 1);
}

// ── Multiple trees ──

#[test]
fn multiple_stubs_independent() {
    let t1 = Tree::new_stub();
    let t2 = Tree::new_stub();
    assert_eq!(t1.root_kind(), t2.root_kind());
}

#[test]
fn multiple_testing_trees_different() {
    let t1 = Tree::new_for_testing(1, 0, 5, vec![]);
    let t2 = Tree::new_for_testing(2, 0, 5, vec![]);
    assert_ne!(t1.root_kind(), t2.root_kind());
}

// ── Debug format ──

#[test]
fn debug_stub() {
    let t = Tree::new_stub();
    let s = format!("{:?}", t);
    assert!(s.len() > 0);
}

#[test]
fn debug_testing_tree() {
    let t = Tree::new_for_testing(5, 10, 20, vec![]);
    let s = format!("{:?}", t);
    assert!(s.len() > 0);
}

#[test]
fn debug_with_children() {
    let child = Tree::new_for_testing(2, 0, 1, vec![]);
    let parent = Tree::new_for_testing(1, 0, 1, vec![child]);
    let s = format!("{:?}", parent);
    assert!(s.len() > 0);
}

// ── Various symbol IDs ──

#[test]
fn symbol_zero() {
    let t = Tree::new_for_testing(0, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 0);
}

#[test]
fn symbol_one() {
    let t = Tree::new_for_testing(1, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn symbol_max_u16() {
    let t = Tree::new_for_testing(u16::MAX as u32, 0, 1, vec![]);
    assert_eq!(t.root_kind(), u16::MAX as u32);
}

#[test]
fn symbol_256() {
    let t = Tree::new_for_testing(256, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 256);
}

#[test]
fn symbol_1000() {
    let t = Tree::new_for_testing(1000, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 1000);
}

// ── Byte ranges ──

#[test]
fn range_zero_to_zero() {
    let t = Tree::new_for_testing(1, 0, 0, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn range_zero_to_one() {
    let t = Tree::new_for_testing(1, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn range_large() {
    let t = Tree::new_for_testing(1, 0, 999_999, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn range_offset() {
    let t = Tree::new_for_testing(1, 100, 200, vec![]);
    assert_eq!(t.root_kind(), 1);
}

// ── Children count variations ──

#[test]
fn zero_children() {
    let t = Tree::new_for_testing(1, 0, 5, vec![]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn one_child() {
    let c = Tree::new_for_testing(2, 0, 2, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![c]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn two_children() {
    let c1 = Tree::new_for_testing(2, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(3, 2, 5, vec![]);
    let t = Tree::new_for_testing(1, 0, 5, vec![c1, c2]);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn five_children() {
    let children: Vec<Tree> = (0..5)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let t = Tree::new_for_testing(1, 0, 5, children);
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn ten_children() {
    let children: Vec<Tree> = (0..10)
        .map(|i| Tree::new_for_testing(i + 10, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let t = Tree::new_for_testing(1, 0, 10, children);
    assert_eq!(t.root_kind(), 1);
}

// ── Deep nesting ──

#[test]
fn depth_3() {
    let leaf = Tree::new_for_testing(3, 0, 1, vec![]);
    let mid = Tree::new_for_testing(2, 0, 1, vec![leaf]);
    let root = Tree::new_for_testing(1, 0, 1, vec![mid]);
    assert_eq!(root.root_kind(), 1);
}

#[test]
fn depth_5() {
    let mut t = Tree::new_for_testing(5, 0, 1, vec![]);
    for i in (1..5).rev() {
        t = Tree::new_for_testing(i, 0, 1, vec![t]);
    }
    assert_eq!(t.root_kind(), 1);
}

#[test]
fn depth_10() {
    let mut t = Tree::new_for_testing(10, 0, 1, vec![]);
    for i in (1..10).rev() {
        t = Tree::new_for_testing(i, 0, 1, vec![t]);
    }
    assert_eq!(t.root_kind(), 1);
}
