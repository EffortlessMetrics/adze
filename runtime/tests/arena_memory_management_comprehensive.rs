//! Comprehensive tests for TreeArena memory management edge cases.
//!
//! Covers: construction, allocation, access, mutation, handles,
//! TreeNode variants, large arenas, reset/clear, metrics, and mixed usage.

use adze::arena_allocator::{ArenaMetrics, NodeHandle, TreeArena, TreeNode};

// ============================================================================
// 1. TreeArena::new() construction
// ============================================================================

#[test]
fn new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn new_arena_has_one_chunk() {
    let arena = TreeArena::new();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn new_arena_has_default_capacity() {
    let arena = TreeArena::new();
    assert_eq!(arena.capacity(), 1024);
}

#[test]
fn new_arena_memory_usage_is_positive() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn new_arena_default_matches_new() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

// ============================================================================
// 2. TreeArena::with_capacity(n)
// ============================================================================

#[test]
fn with_capacity_one() {
    let arena = TreeArena::with_capacity(1);
    assert_eq!(arena.capacity(), 1);
    assert!(arena.is_empty());
}

#[test]
fn with_capacity_small() {
    let arena = TreeArena::with_capacity(4);
    assert_eq!(arena.capacity(), 4);
}

#[test]
fn with_capacity_large() {
    let arena = TreeArena::with_capacity(100_000);
    assert_eq!(arena.capacity(), 100_000);
}

#[test]
fn with_capacity_exact_power_of_two() {
    let arena = TreeArena::with_capacity(512);
    assert_eq!(arena.capacity(), 512);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn with_capacity_odd_number() {
    let arena = TreeArena::with_capacity(7);
    assert_eq!(arena.capacity(), 7);
}

// ============================================================================
// 3. alloc and get roundtrip
// ============================================================================

#[test]
fn alloc_leaf_roundtrip() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn alloc_leaf_zero_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn alloc_leaf_negative_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-1));
    assert_eq!(arena.get(h).value(), -1);
}

#[test]
fn alloc_leaf_max_i32() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn alloc_leaf_min_i32() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

#[test]
fn alloc_increments_len() {
    let mut arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
}

#[test]
fn alloc_not_empty_after_first() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(0));
    assert!(!arena.is_empty());
}

// ============================================================================
// 4. alloc multiple nodes — ordering & independence
// ============================================================================

#[test]
fn multiple_alloc_independent_values() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn multiple_alloc_unique_handles() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h3 = arena.alloc(TreeNode::leaf(3));
    assert_ne!(h1, h2);
    assert_ne!(h2, h3);
    assert_ne!(h1, h3);
}

#[test]
fn alloc_same_value_gives_different_handles() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(42));
    let h2 = arena.alloc(TreeNode::leaf(42));
    assert_ne!(h1, h2);
    assert_eq!(arena.get(h1).value(), arena.get(h2).value());
}

#[test]
fn alloc_across_chunk_boundary() {
    let mut arena = TreeArena::with_capacity(2);
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    let h3 = arena.alloc(TreeNode::leaf(30));
    assert_eq!(arena.num_chunks(), 2);
    assert_eq!(arena.get(h1).value(), 10);
    assert_eq!(arena.get(h2).value(), 20);
    assert_eq!(arena.get(h3).value(), 30);
}

#[test]
fn alloc_fills_first_chunk_exactly() {
    let mut arena = TreeArena::with_capacity(3);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.num_chunks(), 1);
    assert_eq!(arena.len(), 3);
}

// ============================================================================
// 5. get_mut modification
// ============================================================================

#[test]
fn get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn get_mut_set_value_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    arena.get_mut(h).set_value(0);
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn get_mut_does_not_affect_other_nodes() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    arena.get_mut(h1).set_value(100);
    assert_eq!(arena.get(h1).value(), 100);
    assert_eq!(arena.get(h2).value(), 2);
}

#[test]
fn get_mut_negative_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(-999);
    assert_eq!(arena.get(h).value(), -999);
}

#[test]
fn get_mut_multiple_times() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    for i in 1..=5 {
        arena.get_mut(h).set_value(i);
        assert_eq!(arena.get(h).value(), i);
    }
}

#[test]
fn get_mut_branch_set_value_is_noop() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch_with_symbol(10, vec![child]));
    // set_value only works on Leaf nodes; should be a no-op on Branch
    arena.get_mut(parent).set_value(999);
    assert_eq!(arena.get(parent).value(), 10);
}

// ============================================================================
// 6. NodeHandle properties (Copy, Clone, Debug, PartialEq, Eq, Hash)
// ============================================================================

#[test]
fn node_handle_is_copy() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let h2 = h; // Copy
    assert_eq!(arena.get(h).value(), arena.get(h2).value());
}

#[test]
fn node_handle_clone() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    let h2 = h.clone();
    assert_eq!(h, h2);
}

#[test]
fn node_handle_debug_format() {
    let h = NodeHandle::new(0, 0);
    let debug_str = format!("{:?}", h);
    assert!(!debug_str.is_empty());
}

#[test]
fn node_handle_eq_same() {
    let h1 = NodeHandle::new(1, 2);
    let h2 = NodeHandle::new(1, 2);
    assert_eq!(h1, h2);
}

#[test]
fn node_handle_ne_different_chunk() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(1, 0);
    assert_ne!(h1, h2);
}

#[test]
fn node_handle_ne_different_node() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    assert_ne!(h1, h2);
}

#[test]
fn node_handle_hash_consistent() {
    use std::collections::HashSet;
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let mut set = HashSet::new();
    set.insert(h1);
    set.insert(h2);
    assert_eq!(set.len(), 2);
    assert!(set.contains(&h1));
    assert!(set.contains(&h2));
}

#[test]
fn node_handle_in_vec() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..5).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(handles.len(), 5);
}

// ============================================================================
// 7. TreeNode construction
// ============================================================================

#[test]
fn tree_node_leaf_is_leaf() {
    let n = TreeNode::leaf(1);
    assert!(n.is_leaf());
    assert!(!n.is_branch());
}

#[test]
fn tree_node_leaf_value() {
    let n = TreeNode::leaf(42);
    assert_eq!(n.value(), 42);
    assert_eq!(n.symbol(), 42);
}

#[test]
fn tree_node_leaf_children_empty() {
    let n = TreeNode::leaf(1);
    assert!(n.children().is_empty());
}

#[test]
fn tree_node_branch_is_branch() {
    let n = TreeNode::branch(vec![]);
    assert!(n.is_branch());
    assert!(!n.is_leaf());
}

#[test]
fn tree_node_branch_default_symbol() {
    let n = TreeNode::branch(vec![]);
    assert_eq!(n.symbol(), 0);
}

#[test]
fn tree_node_branch_with_symbol() {
    let n = TreeNode::branch_with_symbol(55, vec![]);
    assert_eq!(n.symbol(), 55);
    assert_eq!(n.value(), 55);
}

#[test]
fn tree_node_clone() {
    let n = TreeNode::leaf(99);
    let n2 = n.clone();
    assert_eq!(n, n2);
}

#[test]
fn tree_node_debug_format() {
    let n = TreeNode::leaf(3);
    let debug_str = format!("{:?}", n);
    assert!(!debug_str.is_empty());
}

#[test]
fn tree_node_partial_eq_same() {
    let a = TreeNode::leaf(5);
    let b = TreeNode::leaf(5);
    assert_eq!(a, b);
}

#[test]
fn tree_node_partial_eq_different() {
    let a = TreeNode::leaf(1);
    let b = TreeNode::leaf(2);
    assert_ne!(a, b);
}

// ============================================================================
// 8. TreeNode with children
// ============================================================================

#[test]
fn branch_with_one_child() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch(vec![child]));
    assert_eq!(arena.get(parent).children().len(), 1);
    assert_eq!(arena.get(parent).children()[0], child);
}

#[test]
fn branch_with_multiple_children() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let c3 = arena.alloc(TreeNode::leaf(30));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2, c3]));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 3);
    let c0 = children[0];
    let c1 = children[1];
    let c2 = children[2];
    drop(parent_ref);
    assert_eq!(arena.get(c0).value(), 10);
    assert_eq!(arena.get(c1).value(), 20);
    assert_eq!(arena.get(c2).value(), 30);
}

#[test]
fn branch_empty_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).children().is_empty());
    assert!(arena.get(h).is_branch());
}

#[test]
fn nested_branches() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(1));
    let inner = arena.alloc(TreeNode::branch(vec![leaf]));
    let outer = arena.alloc(TreeNode::branch(vec![inner]));
    let outer_ref = arena.get(outer);
    let outer_children = outer_ref.children();
    assert_eq!(outer_children.len(), 1);
    let inner_h = outer_children[0];
    drop(outer_ref);
    let inner_ref = arena.get(inner_h);
    let inner_children = inner_ref.children();
    assert_eq!(inner_children.len(), 1);
    let leaf_h = inner_children[0];
    drop(inner_ref);
    assert_eq!(arena.get(leaf_h).value(), 1);
}

#[test]
fn deeply_nested_tree() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..=10 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    // Walk down the tree
    let mut node_handle = current;
    for i in (1..=10).rev() {
        assert_eq!(arena.get(node_handle).symbol(), i);
        node_handle = arena.get(node_handle).children()[0];
    }
    assert_eq!(arena.get(node_handle).value(), 0);
    assert!(arena.get(node_handle).is_leaf());
}

#[test]
fn wide_branch_many_children() {
    let mut arena = TreeArena::new();
    let children: Vec<_> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(children.clone()));
    let parent_ref = arena.get(parent);
    let stored_children: Vec<NodeHandle> = parent_ref.children().to_vec();
    drop(parent_ref);
    assert_eq!(stored_children.len(), 50);
    for (i, ch) in stored_children.iter().enumerate() {
        assert_eq!(arena.get(*ch).value(), i as i32);
    }
}

// ============================================================================
// 9. Large arena (hundreds of nodes)
// ============================================================================

#[test]
fn large_arena_500_nodes() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..500).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 500);
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn large_arena_triggers_chunk_growth() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    assert_eq!(arena.len(), 100);
}

#[test]
fn large_arena_capacity_grows() {
    let mut arena = TreeArena::with_capacity(4);
    let initial_cap = arena.capacity();
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() > initial_cap);
}

#[test]
fn large_arena_memory_usage_grows() {
    let mut arena = TreeArena::with_capacity(4);
    let initial_mem = arena.memory_usage();
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.memory_usage() > initial_mem);
}

// ============================================================================
// 10. Arena with mixed node types
// ============================================================================

#[test]
fn mixed_leaves_and_branches() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let b2 = arena.alloc(TreeNode::branch_with_symbol(100, vec![b1, l3]));

    assert!(arena.get(l1).is_leaf());
    assert!(arena.get(b1).is_branch());
    assert!(arena.get(b2).is_branch());
    assert_eq!(arena.get(b2).symbol(), 100);
    assert_eq!(arena.get(b2).children().len(), 2);
}

#[test]
fn mixed_interleaved_alloc() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();
    for i in 0..20 {
        if i % 3 == 0 {
            handles.push(arena.alloc(TreeNode::branch(vec![])));
        } else {
            handles.push(arena.alloc(TreeNode::leaf(i)));
        }
    }
    for (i, h) in handles.iter().enumerate() {
        if i % 3 == 0 {
            assert!(arena.get(*h).is_branch());
        } else {
            assert!(arena.get(*h).is_leaf());
            assert_eq!(arena.get(*h).value(), i as i32);
        }
    }
}

// ============================================================================
// 11. Reset and clear
// ============================================================================

#[test]
fn reset_makes_empty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn reset_preserves_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3)); // triggers second chunk
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn reset_allows_realloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
}

#[test]
fn clear_frees_extra_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3)); // second chunk
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert!(arena.is_empty());
}

#[test]
fn clear_allows_realloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(77));
    assert_eq!(arena.get(h).value(), 77);
}

#[test]
fn double_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    arena.reset();
    assert!(arena.is_empty());
}

// ============================================================================
// 12. Metrics
// ============================================================================

#[test]
fn metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.num_chunks(), 1);
    assert!(m.capacity() > 0);
    assert!(m.memory_usage() > 0);
}

#[test]
fn metrics_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let m = arena.metrics();
    assert_eq!(m.len(), 1);
    assert!(!m.is_empty());
}

#[test]
fn metrics_after_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
}

#[test]
fn metrics_clone_and_eq() {
    let arena = TreeArena::new();
    let m1 = arena.metrics();
    let m2 = m1;
    assert_eq!(m1, m2);
}

#[test]
fn metrics_debug_format() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    let s = format!("{:?}", m);
    assert!(!s.is_empty());
}

// ============================================================================
// 13. TreeNodeRef / TreeNodeRefMut API via Deref
// ============================================================================

#[test]
fn tree_node_ref_deref_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let node_ref = arena.get(h);
    // Deref: call TreeNode methods through the ref
    assert_eq!(node_ref.value(), 5);
    assert!(node_ref.is_leaf());
}

#[test]
fn tree_node_ref_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(42, vec![]));
    let node_ref = arena.get(h);
    assert_eq!(node_ref.symbol(), 42);
}

#[test]
fn tree_node_ref_children_via_deref() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let p = arena.alloc(TreeNode::branch(vec![c]));
    let node_ref = arena.get(p);
    assert_eq!(node_ref.children().len(), 1);
}

#[test]
fn tree_node_ref_as_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(3));
    let node_ref = arena.get(h);
    let raw: &TreeNode = node_ref.as_ref();
    assert_eq!(raw.value(), 3);
}

#[test]
fn tree_node_ref_get_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(8));
    let node_ref = arena.get(h);
    let raw: &TreeNode = node_ref.get_ref();
    assert_eq!(raw.value(), 8);
}

// ============================================================================
// 14. Edge cases and stress
// ============================================================================

#[test]
fn with_capacity_one_multiple_allocs() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h3 = arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.get(h3).value(), 3);
}

#[test]
fn branch_with_symbol_negative() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(-100, vec![]));
    assert_eq!(arena.get(h).symbol(), -100);
}

#[test]
fn alloc_after_chunk_growth_preserves_earlier() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(111));
    // This forces a new chunk
    let h2 = arena.alloc(TreeNode::leaf(222));
    // Verify first-chunk handle still works
    assert_eq!(arena.get(h1).value(), 111);
    assert_eq!(arena.get(h2).value(), 222);
}

#[test]
fn stress_alloc_1000_nodes() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 1000);
    // Spot-check
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[500]).value(), 500);
    assert_eq!(arena.get(handles[999]).value(), 999);
}

#[test]
fn build_binary_tree() {
    let mut arena = TreeArena::new();
    // Build a complete binary tree of depth 4 (15 nodes)
    fn build(arena: &mut TreeArena, depth: i32) -> NodeHandle {
        if depth == 0 {
            arena.alloc(TreeNode::leaf(0))
        } else {
            let left = build(arena, depth - 1);
            let right = build(arena, depth - 1);
            arena.alloc(TreeNode::branch_with_symbol(depth, vec![left, right]))
        }
    }
    let root = build(&mut arena, 4);
    assert_eq!(arena.get(root).symbol(), 4);
    assert_eq!(arena.get(root).children().len(), 2);
    // 2^5 - 1 = 31 nodes total
    assert_eq!(arena.len(), 31);
}

#[test]
fn reset_then_build_new_tree() {
    let mut arena = TreeArena::new();
    let _ = arena.alloc(TreeNode::leaf(1));
    let _ = arena.alloc(TreeNode::leaf(2));
    arena.reset();

    let a = arena.alloc(TreeNode::leaf(10));
    let b = arena.alloc(TreeNode::leaf(20));
    let root = arena.alloc(TreeNode::branch(vec![a, b]));
    assert_eq!(arena.len(), 3);
    assert_eq!(arena.get(root).children().len(), 2);
    assert_eq!(arena.get(a).value(), 10);
    assert_eq!(arena.get(b).value(), 20);
}

#[test]
fn node_handle_used_as_hashmap_key() {
    use std::collections::HashMap;
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let mut map = HashMap::new();
    map.insert(h1, "first");
    map.insert(h2, "second");
    assert_eq!(map[&h1], "first");
    assert_eq!(map[&h2], "second");
}

#[test]
fn chunk_growth_is_exponential() {
    let mut arena = TreeArena::with_capacity(2);
    // Fill first chunk (cap 2)
    arena.alloc(TreeNode::leaf(0));
    arena.alloc(TreeNode::leaf(0));
    // Fill second chunk (cap 4)
    for _ in 0..4 {
        arena.alloc(TreeNode::leaf(0));
    }
    // Fill third chunk (cap 8)
    for _ in 0..8 {
        arena.alloc(TreeNode::leaf(0));
    }
    assert!(arena.num_chunks() >= 3);
    // Total capacity should be at least 2 + 4 + 8 = 14
    assert!(arena.capacity() >= 14);
}

#[test]
fn leaf_and_branch_equality() {
    let l1 = TreeNode::leaf(5);
    let l2 = TreeNode::leaf(5);
    let b1 = TreeNode::branch(vec![]);
    let b2 = TreeNode::branch(vec![]);
    assert_eq!(l1, l2);
    assert_eq!(b1, b2);
    assert_ne!(l1, b1);
}

#[test]
fn get_mut_then_get_consistent() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    {
        let mut mut_ref = arena.get_mut(h);
        mut_ref.set_value(77);
    }
    let val = arena.get(h).value();
    assert_eq!(val, 77);
}
