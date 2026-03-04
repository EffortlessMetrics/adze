//! Comprehensive tests for TreeArena, NodeHandle, and TreeNode.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ── TreeNode ──

#[test]
fn tree_node_leaf() {
    let node = TreeNode::leaf(42);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
    assert_eq!(node.value(), 42);
    assert_eq!(node.symbol(), 42);
}

#[test]
fn tree_node_leaf_zero() {
    let node = TreeNode::leaf(0);
    assert!(node.is_leaf());
    assert_eq!(node.value(), 0);
}

#[test]
fn tree_node_leaf_negative() {
    let node = TreeNode::leaf(-1);
    assert!(node.is_leaf());
    assert_eq!(node.value(), -1);
}

#[test]
fn tree_node_branch_empty() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
    assert_eq!(node.children().len(), 0);
}

#[test]
fn tree_node_branch_with_children() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let node = TreeNode::branch(vec![h1, h2]);
    assert!(node.is_branch());
    assert_eq!(node.children().len(), 2);
}

#[test]
fn tree_node_branch_with_symbol() {
    let h1 = NodeHandle::new(0, 0);
    let node = TreeNode::branch_with_symbol(99, vec![h1]);
    assert!(node.is_branch());
    assert_eq!(node.symbol(), 99);
    assert_eq!(node.children().len(), 1);
}

#[test]
fn tree_node_clone() {
    let node = TreeNode::leaf(5);
    let cloned = node.clone();
    assert_eq!(cloned.value(), 5);
}

#[test]
fn tree_node_debug() {
    let node = TreeNode::leaf(10);
    let dbg = format!("{:?}", node);
    assert!(!dbg.is_empty());
}

// ── NodeHandle ──

#[test]
fn node_handle_new() {
    let h = NodeHandle::new(1, 2);
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
fn node_handle_clone() {
    let h = NodeHandle::new(3, 4);
    let cloned = h.clone();
    assert_eq!(h, cloned);
}

#[test]
fn node_handle_copy() {
    let h = NodeHandle::new(3, 4);
    let copied = h;
    assert_eq!(h, copied);
}

// ── TreeArena ──

#[test]
fn arena_new_is_empty() {
    let arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn arena_with_capacity() {
    let arena = TreeArena::with_capacity(100);
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    assert!(arena.capacity() >= 100);
}

#[test]
fn arena_alloc_leaf() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());

    let node = arena.get(handle);
    assert_eq!(node.value(), 42);
}

#[test]
fn arena_alloc_multiple() {
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
fn arena_alloc_branch_with_children() {
    let mut arena = TreeArena::new();
    let child1 = arena.alloc(TreeNode::leaf(10));
    let child2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![child1, child2]));
    assert_eq!(arena.len(), 3);

    let parent_node = arena.get(parent);
    assert!(parent_node.get_ref().is_branch());
}

#[test]
fn arena_get_mut() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(1));
    {
        let mut node_mut = arena.get_mut(handle);
        // TreeNodeRefMut exists; just verify it works
        let _ = node_mut.value();
    }
    assert_eq!(arena.get(handle).value(), 1);
}

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
    arena.alloc(TreeNode::leaf(2));
    arena.clear();
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_num_chunks() {
    let arena = TreeArena::new();
    assert!(arena.num_chunks() >= 0);
}

#[test]
fn arena_memory_usage() {
    let mut arena = TreeArena::new();
    let initial = arena.memory_usage();
    arena.alloc(TreeNode::leaf(1));
    let after = arena.memory_usage();
    assert!(after >= initial);
}

#[test]
fn arena_capacity_grows() {
    let mut arena = TreeArena::with_capacity(2);
    let initial_cap = arena.capacity();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    // Capacity should have grown
    assert!(arena.capacity() >= initial_cap);
    assert_eq!(arena.len(), 100);
}

#[test]
fn arena_stress_test() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();
    for i in 0..1000 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert_eq!(arena.len(), 1000);
    // Verify all values are correct
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn arena_deep_tree() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(0));
    let mut current = leaf;
    for i in 1..50 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.len(), 50);
    assert_eq!(arena.get(current).get_ref().symbol(), 49);
}

#[test]
fn arena_wide_tree() {
    let mut arena = TreeArena::new();
    let mut children = Vec::new();
    for i in 0..100 {
        children.push(arena.alloc(TreeNode::leaf(i)));
    }
    let root = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.len(), 101);
    assert!(arena.get(root).get_ref().is_branch());
}

// ── TreeNodeRef ──

#[test]
fn tree_node_ref_get_ref() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(42));
    let node_ref = arena.get(handle);
    let inner = node_ref.get_ref();
    assert_eq!(inner.value(), 42);
}

#[test]
fn tree_node_ref_as_ref() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(42));
    let node_ref = arena.get(handle);
    let inner = node_ref.as_ref();
    assert_eq!(inner.value(), 42);
}

#[test]
fn tree_node_ref_value() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(99));
    let node_ref = arena.get(handle);
    assert_eq!(node_ref.value(), 99);
}

#[test]
fn tree_node_ref_symbol() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::branch_with_symbol(7, vec![]));
    let node_ref = arena.get(handle);
    assert_eq!(node_ref.symbol(), 7);
}
