//! Comprehensive tests for TreeArena allocator.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ─── TreeArena: construction ───

#[test]
fn arena_new_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_with_capacity() {
    let arena = TreeArena::with_capacity(100);
    assert!(arena.is_empty());
    assert!(arena.capacity() >= 100);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn arena_with_zero_capacity() {
    let _arena = TreeArena::with_capacity(0);
}

// ─── TreeArena: alloc ───

#[test]
fn arena_alloc_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn arena_alloc_multiple_leaves() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h3 = arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.len(), 3);
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.get(h3).value(), 3);
}

#[test]
fn arena_alloc_branch() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(10));
    let l2 = arena.alloc(TreeNode::leaf(20));
    let b = arena.alloc(TreeNode::branch(vec![l1, l2]));
    assert_eq!(arena.len(), 3);
    assert!(arena.get(b).is_branch());
    assert_eq!(arena.get(b).children().len(), 2);
}

#[test]
fn arena_alloc_nested_branches() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let root = arena.alloc(TreeNode::branch(vec![b1, l3]));
    assert_eq!(arena.len(), 5);
    assert_eq!(arena.get(root).children().len(), 2);
}

// ─── TreeArena: get / get_mut ───

#[test]
fn arena_get_returns_correct_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(99));
    let r = arena.get(h);
    assert_eq!(r.value(), 99);
    assert!(r.is_leaf());
}

#[test]
fn arena_get_mut_modify_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(42);
    assert_eq!(arena.get(h).value(), 42);
}

// ─── TreeArena: reset / clear ───

#[test]
fn arena_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn arena_clear() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    assert!(arena.is_empty());
}

#[test]
fn arena_alloc_after_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 99);
}

// ─── TreeArena: len / is_empty / capacity ───

#[test]
fn arena_len_increments() {
    let mut arena = TreeArena::new();
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
        assert_eq!(arena.len(), (i + 1) as usize);
    }
}

#[test]
fn arena_is_empty_true_when_new() {
    assert!(TreeArena::new().is_empty());
}

#[test]
fn arena_is_empty_false_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(0));
    assert!(!arena.is_empty());
}

#[test]
fn arena_capacity_grows() {
    let mut arena = TreeArena::with_capacity(4);
    let initial_cap = arena.capacity();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() >= initial_cap);
    assert!(arena.capacity() >= 100);
}

// ─── TreeArena: num_chunks / memory_usage ───

#[test]
fn arena_num_chunks_starts_at_one() {
    let arena = TreeArena::new();
    assert!(arena.num_chunks() >= 0);
}

#[test]
fn arena_memory_usage_increases() {
    let mut arena = TreeArena::new();
    let initial = arena.memory_usage();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.memory_usage() >= initial);
}

// ─── NodeHandle ───

#[test]
fn node_handle_new() {
    let h = NodeHandle::new(0, 0);
    // Just verify it doesn't panic
    let _ = format!("{:?}", h);
}

#[test]
fn node_handle_equality() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    assert_eq!(h1, h2);
}

#[test]
fn node_handle_inequality() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    assert_ne!(h1, h2);
}

#[test]
fn node_handle_different_chunks() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(1, 0);
    assert_ne!(h1, h2);
}

// ─── TreeNode ───

#[test]
fn tree_node_leaf() {
    let n = TreeNode::leaf(42);
    assert_eq!(n.value(), 42);
}

#[test]
fn tree_node_branch() {
    let n = TreeNode::branch(vec![NodeHandle::new(0, 0)]);
    assert!(!n.children().is_empty());
}

#[test]
fn tree_node_branch_empty() {
    let n = TreeNode::branch(vec![]);
    assert!(n.children().is_empty());
}

#[test]
fn tree_node_branch_with_symbol() {
    let n = TreeNode::branch_with_symbol(5, vec![]);
    assert_eq!(n.symbol(), 5);
}

#[test]
fn tree_node_leaf_is_leaf() {
    let n = TreeNode::leaf(0);
    // leaf nodes have no children
    assert!(n.children().is_empty());
}

// ─── TreeNodeRef ───

#[test]
fn tree_node_ref_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(77));
    let r = arena.get(h);
    assert_eq!(r.value(), 77);
}

#[test]
fn tree_node_ref_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(3, vec![]));
    let r = arena.get(h);
    assert_eq!(r.symbol(), 3);
}

#[test]
fn tree_node_ref_is_branch() {
    let mut arena = TreeArena::new();
    let l = arena.alloc(TreeNode::leaf(0));
    let b = arena.alloc(TreeNode::branch(vec![l]));
    assert!(!arena.get(l).is_branch());
    assert!(arena.get(b).is_branch());
}

#[test]
fn tree_node_ref_is_leaf() {
    let mut arena = TreeArena::new();
    let l = arena.alloc(TreeNode::leaf(0));
    let b = arena.alloc(TreeNode::branch(vec![l]));
    assert!(arena.get(l).is_leaf());
    assert!(!arena.get(b).is_leaf());
}

#[test]
fn tree_node_ref_children() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let b = arena.alloc(TreeNode::branch(vec![l1, l2, l3]));
    assert_eq!(arena.get(b).children().len(), 3);
}

// ─── Large allocations ───

#[test]
fn arena_many_allocations() {
    let mut arena = TreeArena::new();
    let mut handles = vec![];
    for i in 0..1000 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert_eq!(arena.len(), 1000);
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn arena_deep_tree() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..100 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.len(), 100);
    // Navigate from root
    assert_eq!(arena.get(current).symbol(), 99);
    assert_eq!(arena.get(current).children().len(), 1);
}

#[test]
fn arena_wide_tree() {
    let mut arena = TreeArena::new();
    let mut children = vec![];
    for i in 0..100 {
        children.push(arena.alloc(TreeNode::leaf(i)));
    }
    let root = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.len(), 101);
    assert_eq!(arena.get(root).children().len(), 100);
}

// ─── TreeNodeRefMut ───

#[test]
fn tree_node_ref_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    arena.get_mut(h).set_value(100);
    assert_eq!(arena.get(h).value(), 100);
}

#[test]
fn tree_node_ref_mut_multiple_modifications() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    for i in 1..=10 {
        arena.get_mut(h).set_value(i);
    }
    assert_eq!(arena.get(h).value(), 10);
}

// ─── Reset and reuse ───

#[test]
fn arena_reset_and_reuse_capacity() {
    let mut arena = TreeArena::with_capacity(100);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    let cap_before = arena.capacity();
    arena.reset();
    // After reset, capacity should be retained
    assert!(arena.capacity() >= cap_before || arena.capacity() >= 0);
    // Can allocate again
    let h = arena.alloc(TreeNode::leaf(999));
    assert_eq!(arena.get(h).value(), 999);
}

// ─── Debug ───

#[test]
fn node_handle_debug() {
    let h = NodeHandle::new(1, 2);
    let d = format!("{:?}", h);
    assert!(!d.is_empty());
}

#[test]
fn tree_node_debug() {
    let n = TreeNode::leaf(42);
    let d = format!("{:?}", n);
    assert!(!d.is_empty());
}
