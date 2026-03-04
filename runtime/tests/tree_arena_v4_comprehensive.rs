//! Comprehensive tests for TreeArena, NodeHandle, and TreeNode.

use std::collections::{HashMap, HashSet};

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ──────────────────────────────────────────────
// TreeArena::with_capacity
// ──────────────────────────────────────────────

#[test]
fn with_capacity_one() {
    let arena = TreeArena::with_capacity(1);
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    assert_eq!(arena.capacity(), 1);
}

#[test]
fn with_capacity_small() {
    let arena = TreeArena::with_capacity(4);
    assert_eq!(arena.capacity(), 4);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn with_capacity_large() {
    let arena = TreeArena::with_capacity(10_000);
    assert_eq!(arena.capacity(), 10_000);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

// ──────────────────────────────────────────────
// TreeArena::alloc / get / get_mut
// ──────────────────────────────────────────────

#[test]
fn alloc_single_leaf() {
    let mut arena = TreeArena::with_capacity(8);
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn alloc_multiple_leaves() {
    let mut arena = TreeArena::with_capacity(8);
    let handles: Vec<_> = (0..5).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
    assert_eq!(arena.len(), 5);
}

#[test]
fn alloc_branch_with_children() {
    let mut arena = TreeArena::with_capacity(8);
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));

    assert!(arena.get(parent).is_branch());
    assert_eq!(arena.get(parent).children().len(), 2);
    assert_eq!(arena.get(parent).children()[0], c1);
    assert_eq!(arena.get(parent).children()[1], c2);
}

#[test]
fn get_returns_correct_values_after_many_allocs() {
    let mut arena = TreeArena::with_capacity(4);
    let handles: Vec<_> = (0..20)
        .map(|i| arena.alloc(TreeNode::leaf(i * 3)))
        .collect();
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), (i as i32) * 3);
    }
}

#[test]
fn get_mut_set_value() {
    let mut arena = TreeArena::with_capacity(8);
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).value(), 1);

    arena.get_mut(h).set_value(42);
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn get_mut_does_not_affect_other_nodes() {
    let mut arena = TreeArena::with_capacity(8);
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));

    arena.get_mut(h1).set_value(99);

    assert_eq!(arena.get(h1).value(), 99);
    assert_eq!(arena.get(h2).value(), 20);
}

// ──────────────────────────────────────────────
// NodeHandle: Display, Debug, Clone, Copy, PartialEq, Eq, Hash
// ──────────────────────────────────────────────

#[test]
fn node_handle_debug() {
    let h = NodeHandle::new(0, 5);
    let dbg = format!("{h:?}");
    assert!(dbg.contains("NodeHandle"));
}

#[test]
fn node_handle_clone() {
    let h = NodeHandle::new(1, 2);
    let h2 = h;
    assert_eq!(h, h2);
}

#[test]
fn node_handle_copy() {
    let h = NodeHandle::new(3, 4);
    let h2 = h; // Copy
    let _h3 = h; // still usable after copy
    assert_eq!(h, h2);
}

#[test]
fn node_handle_eq() {
    let a = NodeHandle::new(0, 0);
    let b = NodeHandle::new(0, 0);
    let c = NodeHandle::new(0, 1);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn node_handle_hash_consistent_with_eq() {
    let mut set = HashSet::new();
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    set.insert(h1);
    set.insert(h2);
    assert_eq!(set.len(), 1, "equal handles must hash identically");
}

#[test]
fn node_handle_hash_in_hashmap() {
    let mut map = HashMap::new();
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(1, 3);
    map.insert(h1, "first");
    map.insert(h2, "second");
    assert_eq!(map[&h1], "first");
    assert_eq!(map[&h2], "second");
}

#[test]
fn node_handle_distinct_handles_differ() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let h3 = NodeHandle::new(1, 0);
    assert_ne!(h1, h2);
    assert_ne!(h1, h3);
    assert_ne!(h2, h3);
}

// ──────────────────────────────────────────────
// TreeNode: creation and field access
// ──────────────────────────────────────────────

#[test]
fn tree_node_leaf_value() {
    let node = TreeNode::leaf(7);
    assert_eq!(node.value(), 7);
    assert_eq!(node.symbol(), 7);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
}

#[test]
fn tree_node_leaf_negative_value() {
    let node = TreeNode::leaf(-42);
    assert_eq!(node.value(), -42);
}

#[test]
fn tree_node_leaf_zero() {
    let node = TreeNode::leaf(0);
    assert_eq!(node.value(), 0);
}

#[test]
fn tree_node_branch_empty_children() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
    assert!(node.children().is_empty());
    assert_eq!(node.symbol(), 0); // default symbol for branch
}

#[test]
fn tree_node_branch_with_symbol() {
    let h = NodeHandle::new(0, 0);
    let node = TreeNode::branch_with_symbol(55, vec![h]);
    assert_eq!(node.symbol(), 55);
    assert_eq!(node.value(), 55);
    assert_eq!(node.children().len(), 1);
}

#[test]
fn tree_node_leaf_has_no_children() {
    let node = TreeNode::leaf(1);
    assert!(node.children().is_empty());
}

#[test]
fn tree_node_clone() {
    let node = TreeNode::leaf(10);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

#[test]
fn tree_node_debug() {
    let node = TreeNode::leaf(5);
    let dbg = format!("{node:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn tree_node_partial_eq() {
    let a = TreeNode::leaf(1);
    let b = TreeNode::leaf(1);
    let c = TreeNode::leaf(2);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ──────────────────────────────────────────────
// TreeNodeRef / TreeNodeRefMut through arena
// ──────────────────────────────────────────────

#[test]
fn tree_node_ref_is_leaf() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn tree_node_ref_is_branch() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(!arena.get(h).is_leaf());
}

#[test]
fn tree_node_ref_symbol() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(77));
    assert_eq!(arena.get(h).symbol(), 77);
}

#[test]
fn tree_node_ref_children_for_branch() {
    let mut arena = TreeArena::with_capacity(8);
    let c = arena.alloc(TreeNode::leaf(1));
    let p = arena.alloc(TreeNode::branch(vec![c]));
    let p_ref = arena.get(p);
    let children = p_ref.children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], c);
}

#[test]
fn tree_node_ref_deref() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(3));
    // Deref gives us access to TreeNode methods directly
    let node_ref = arena.get(h);
    assert_eq!(node_ref.value(), 3);
    assert!(node_ref.is_leaf());
}

// ──────────────────────────────────────────────
// Arena growth: alloc many nodes, verify accessible
// ──────────────────────────────────────────────

#[test]
fn arena_growth_triggers_new_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    assert_eq!(arena.len(), 10);
}

#[test]
fn arena_growth_all_handles_valid_after_growth() {
    let mut arena = TreeArena::with_capacity(2);
    let handles: Vec<_> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn arena_large_1000_nodes() {
    let mut arena = TreeArena::with_capacity(16);
    let handles: Vec<_> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 1000);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn arena_large_5000_nodes() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..5000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 5000);
    // spot check first, middle, last
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[2500]).value(), 2500);
    assert_eq!(arena.get(handles[4999]).value(), 4999);
}

// ──────────────────────────────────────────────
// Arena with different node types/data
// ──────────────────────────────────────────────

#[test]
fn arena_mixed_leaf_and_branch() {
    let mut arena = TreeArena::with_capacity(8);
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));

    assert!(arena.get(l1).is_leaf());
    assert!(arena.get(b).is_branch());
    assert!(arena.get(l3).is_leaf());
    assert_eq!(arena.len(), 4);
}

#[test]
fn arena_nested_branches() {
    let mut arena = TreeArena::with_capacity(16);
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let inner = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let outer = arena.alloc(TreeNode::branch_with_symbol(100, vec![inner, l3]));

    assert_eq!(arena.get(outer).symbol(), 100);
    assert_eq!(arena.get(outer).children().len(), 2);

    let inner_ref = arena.get(inner);
    assert_eq!(inner_ref.children().len(), 2);
}

#[test]
fn arena_branch_with_symbol_values() {
    let mut arena = TreeArena::with_capacity(8);
    let c = arena.alloc(TreeNode::leaf(1));
    let b = arena.alloc(TreeNode::branch_with_symbol(42, vec![c]));
    assert_eq!(arena.get(b).symbol(), 42);
    assert_eq!(arena.get(b).value(), 42);
}

#[test]
fn arena_leaf_min_max_values() {
    let mut arena = TreeArena::with_capacity(4);
    let h_min = arena.alloc(TreeNode::leaf(i32::MIN));
    let h_max = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h_min).value(), i32::MIN);
    assert_eq!(arena.get(h_max).value(), i32::MAX);
}

// ──────────────────────────────────────────────
// Reset and clear
// ──────────────────────────────────────────────

#[test]
fn reset_preserves_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn clear_frees_excess_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert_eq!(arena.len(), 0);
}

#[test]
fn reuse_after_reset() {
    let mut arena = TreeArena::with_capacity(4);
    let _h = arena.alloc(TreeNode::leaf(1));
    arena.reset();

    let h2 = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h2).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn reuse_after_clear() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(7));
    assert_eq!(arena.get(h).value(), 7);
    assert_eq!(arena.len(), 1);
}

// ──────────────────────────────────────────────
// Metrics
// ──────────────────────────────────────────────

#[test]
fn metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert!(m.capacity() > 0);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn metrics_after_allocs() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.len(), 5);
    assert!(!m.is_empty());
}

#[test]
fn metrics_after_reset() {
    let mut arena = TreeArena::with_capacity(4);
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
}

// ──────────────────────────────────────────────
// Edge cases
// ──────────────────────────────────────────────

#[test]
fn single_node_arena() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.capacity(), 1);
}

#[test]
fn single_node_arena_then_grow() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 2);
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
}

#[test]
fn default_arena() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
    assert!(arena.capacity() >= 1024);
}

#[test]
fn capacity_grows_on_demand() {
    let mut arena = TreeArena::with_capacity(2);
    let initial_cap = arena.capacity();
    for _ in 0..3 {
        arena.alloc(TreeNode::leaf(0));
    }
    assert!(arena.capacity() > initial_cap);
}

#[test]
fn handles_from_different_chunks_are_unique() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn memory_usage_increases_with_capacity() {
    let small = TreeArena::with_capacity(10);
    let large = TreeArena::with_capacity(10_000);
    assert!(large.memory_usage() > small.memory_usage());
}

#[test]
fn arena_debug_format() {
    let arena = TreeArena::with_capacity(4);
    let dbg = format!("{arena:?}");
    assert!(dbg.contains("TreeArena"));
}

#[test]
fn node_handle_in_hashset() {
    let mut arena = TreeArena::with_capacity(8);
    let handles: Vec<_> = (0..5).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let set: HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(set.len(), 5);
    for h in &handles {
        assert!(set.contains(h));
    }
}

#[test]
fn branch_children_resolve_to_correct_leaves() {
    let mut arena = TreeArena::with_capacity(16);
    let leaves: Vec<_> = (0..4)
        .map(|i| arena.alloc(TreeNode::leaf(i * 10)))
        .collect();
    let parent = arena.alloc(TreeNode::branch(leaves.clone()));

    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    for (i, &child_handle) in children.iter().enumerate() {
        assert_eq!(arena.get(child_handle).value(), (i as i32) * 10);
    }
}

#[test]
fn arena_max_capacity_value() {
    // Use a very large but reasonable capacity
    let arena = TreeArena::with_capacity(100_000);
    assert_eq!(arena.capacity(), 100_000);
    assert_eq!(arena.num_chunks(), 1);
}
