//! Comprehensive tests for TreeNode struct and operations (v9)
//!
//! 80+ tests covering TreeNode construction, field access, trait impls,
//! arena allocation, NodeHandle semantics, and edge cases.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use std::collections::HashSet;

// ============================================================================
// 1–3. TreeNode::leaf construction basics
// ============================================================================

#[test]
fn leaf_creates_a_leaf_node() {
    let node = TreeNode::leaf(1);
    assert!(node.is_leaf());
}

#[test]
fn leaf_node_has_zero_children() {
    let node = TreeNode::leaf(42);
    assert!(node.children().is_empty());
}

#[test]
fn leaf_node_is_not_a_branch() {
    let node = TreeNode::leaf(7);
    assert!(!node.is_branch());
}

// ============================================================================
// 4. TreeNode field access
// ============================================================================

#[test]
fn leaf_symbol_matches_constructor_arg() {
    let node = TreeNode::leaf(99);
    assert_eq!(node.symbol(), 99);
}

#[test]
fn leaf_value_matches_symbol() {
    let node = TreeNode::leaf(55);
    assert_eq!(node.value(), node.symbol());
}

#[test]
fn branch_symbol_defaults_to_zero() {
    let node = TreeNode::branch(vec![]);
    assert_eq!(node.symbol(), 0);
}

#[test]
fn branch_with_symbol_stores_symbol() {
    let node = TreeNode::branch_with_symbol(123, vec![]);
    assert_eq!(node.symbol(), 123);
}

// ============================================================================
// 5. TreeNode Clone
// ============================================================================

#[test]
fn tree_node_clone_leaf() {
    let node = TreeNode::leaf(10);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

#[test]
fn tree_node_clone_branch() {
    let handle = NodeHandle::new(0, 0);
    let node = TreeNode::branch(vec![handle]);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

#[test]
fn cloned_leaf_retains_symbol() {
    let node = TreeNode::leaf(777);
    let cloned = node.clone();
    assert_eq!(cloned.symbol(), 777);
}

// ============================================================================
// 6. TreeNode Debug
// ============================================================================

#[test]
fn tree_node_debug_is_non_empty() {
    let node = TreeNode::leaf(1);
    let dbg = format!("{node:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn branch_debug_is_non_empty() {
    let node = TreeNode::branch(vec![]);
    let dbg = format!("{node:?}");
    assert!(!dbg.is_empty());
}

// ============================================================================
// 7–9. TreeNode PartialEq
// ============================================================================

#[test]
fn identical_leaves_are_equal() {
    let a = TreeNode::leaf(42);
    let b = TreeNode::leaf(42);
    assert_eq!(a, b);
}

#[test]
fn different_symbol_leaves_not_equal() {
    let a = TreeNode::leaf(1);
    let b = TreeNode::leaf(2);
    assert_ne!(a, b);
}

#[test]
fn leaf_and_branch_not_equal() {
    let leaf = TreeNode::leaf(0);
    let branch = TreeNode::branch(vec![]);
    assert_ne!(leaf, branch);
}

#[test]
fn branches_same_symbol_same_children_equal() {
    let h = NodeHandle::new(0, 0);
    let a = TreeNode::branch_with_symbol(5, vec![h]);
    let b = TreeNode::branch_with_symbol(5, vec![h]);
    assert_eq!(a, b);
}

#[test]
fn branches_different_symbols_not_equal() {
    let a = TreeNode::branch_with_symbol(1, vec![]);
    let b = TreeNode::branch_with_symbol(2, vec![]);
    assert_ne!(a, b);
}

// ============================================================================
// 10. TreeNode with various kind values
// ============================================================================

#[test]
fn leaf_symbol_zero() {
    assert_eq!(TreeNode::leaf(0).symbol(), 0);
}

#[test]
fn leaf_symbol_negative() {
    assert_eq!(TreeNode::leaf(-1).symbol(), -1);
}

#[test]
fn leaf_symbol_max_i32() {
    assert_eq!(TreeNode::leaf(i32::MAX).symbol(), i32::MAX);
}

#[test]
fn leaf_symbol_min_i32() {
    assert_eq!(TreeNode::leaf(i32::MIN).symbol(), i32::MIN);
}

#[test]
fn branch_symbol_negative() {
    let node = TreeNode::branch_with_symbol(-42, vec![]);
    assert_eq!(node.symbol(), -42);
}

// ============================================================================
// 11. TreeNode children and branch structure
// ============================================================================

#[test]
fn branch_children_match_constructor() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let node = TreeNode::branch(vec![h1, h2]);
    assert_eq!(node.children().len(), 2);
    assert_eq!(node.children()[0], h1);
    assert_eq!(node.children()[1], h2);
}

#[test]
fn branch_with_empty_children() {
    let node = TreeNode::branch(vec![]);
    assert!(node.children().is_empty());
    assert!(node.is_branch());
}

#[test]
fn leaf_children_returns_empty_slice() {
    let node = TreeNode::leaf(5);
    let children: &[NodeHandle] = node.children();
    assert!(children.is_empty());
}

// ============================================================================
// 12–13. TreeNode in arena → retrievable
// ============================================================================

#[test]
fn arena_alloc_leaf_then_get() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(handle).value(), 42);
}

#[test]
fn arena_alloc_branch_then_get() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(10));
    let parent = arena.alloc(TreeNode::branch(vec![child]));
    assert!(arena.get(parent).is_branch());
    assert_eq!(arena.get(parent).children().len(), 1);
}

#[test]
fn multiple_nodes_all_retrievable() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();

    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn arena_preserves_branch_symbol() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::branch_with_symbol(999, vec![]));
    assert_eq!(arena.get(handle).symbol(), 999);
}

// ============================================================================
// 14. NodeHandle equality
// ============================================================================

#[test]
fn same_handle_values_are_equal() {
    let h1 = NodeHandle::new(0, 5);
    let h2 = NodeHandle::new(0, 5);
    assert_eq!(h1, h2);
}

#[test]
fn different_chunk_handles_not_equal() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(1, 0);
    assert_ne!(h1, h2);
}

#[test]
fn different_node_handles_not_equal() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    assert_ne!(h1, h2);
}

// ============================================================================
// 15. NodeHandle Copy
// ============================================================================

#[test]
fn node_handle_is_copy() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = h1; // Copy, not move
    assert_eq!(h1, h2);
}

#[test]
fn node_handle_copy_through_function() {
    fn take_handle(h: NodeHandle) -> NodeHandle {
        h
    }
    let original = NodeHandle::new(1, 2);
    let returned = take_handle(original);
    // original still usable because NodeHandle is Copy
    assert_eq!(original, returned);
}

// ============================================================================
// 16. NodeHandle Hash (use in HashSet)
// ============================================================================

#[test]
fn node_handle_in_hashset() {
    let mut set = HashSet::new();
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    set.insert(h1);
    set.insert(h2);
    assert_eq!(set.len(), 2);
    assert!(set.contains(&h1));
    assert!(set.contains(&h2));
}

#[test]
fn duplicate_handle_not_double_inserted() {
    let mut set = HashSet::new();
    let h = NodeHandle::new(3, 7);
    set.insert(h);
    set.insert(h);
    assert_eq!(set.len(), 1);
}

#[test]
fn hashset_with_many_handles() {
    let mut set = HashSet::new();
    for i in 0..50 {
        set.insert(NodeHandle::new(0, i));
    }
    assert_eq!(set.len(), 50);
}

// ============================================================================
// 17. Arena alloc then get → same node
// ============================================================================

#[test]
fn alloc_get_roundtrip_leaf() {
    let mut arena = TreeArena::new();
    let node = TreeNode::leaf(314);
    let handle = arena.alloc(node.clone());
    let retrieved = arena.get(handle);
    assert_eq!(retrieved.symbol(), 314);
    assert!(retrieved.is_leaf());
}

#[test]
fn alloc_get_roundtrip_branch() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let node = TreeNode::branch_with_symbol(50, vec![c1, c2]);
    let handle = arena.alloc(node);
    let retrieved = arena.get(handle);
    assert_eq!(retrieved.symbol(), 50);
    assert_eq!(retrieved.children().len(), 2);
}

// ============================================================================
// 18. Arena with 100 nodes
// ============================================================================

#[test]
fn arena_hundred_leaves() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 100);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn arena_hundred_branches() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();
    for i in 0..100 {
        let h = arena.alloc(TreeNode::branch_with_symbol(i, vec![]));
        handles.push(h);
    }
    assert_eq!(arena.len(), 100);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).symbol(), i as i32);
    }
}

// ============================================================================
// 19. Arena clear → len is zero
// ============================================================================

#[test]
fn arena_clear_resets_len() {
    let mut arena = TreeArena::new();
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 10);
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_reset_resets_len() {
    let mut arena = TreeArena::new();
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn arena_clear_allows_reallocation() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let handle = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(handle).value(), 2);
    assert_eq!(arena.len(), 1);
}

// ============================================================================
// 20. Various TreeNode field combinations
// ============================================================================

#[test]
fn leaf_with_symbol_one() {
    let node = TreeNode::leaf(1);
    assert_eq!(node.symbol(), 1);
    assert!(node.is_leaf());
    assert!(node.children().is_empty());
}

#[test]
fn branch_with_one_child() {
    let h = NodeHandle::new(0, 0);
    let node = TreeNode::branch_with_symbol(10, vec![h]);
    assert_eq!(node.symbol(), 10);
    assert!(node.is_branch());
    assert_eq!(node.children().len(), 1);
}

#[test]
fn branch_with_many_children() {
    let children: Vec<NodeHandle> = (0..20).map(|i| NodeHandle::new(0, i)).collect();
    let node = TreeNode::branch_with_symbol(5, children);
    assert_eq!(node.children().len(), 20);
}

// ============================================================================
// 21–30. Additional TreeNode trait and semantic tests
// ============================================================================

#[test]
fn tree_node_debug_contains_leaf_info() {
    let node = TreeNode::leaf(42);
    let dbg = format!("{node:?}");
    assert!(dbg.contains("Leaf"));
}

#[test]
fn tree_node_debug_contains_branch_info() {
    let node = TreeNode::branch(vec![]);
    let dbg = format!("{node:?}");
    assert!(dbg.contains("Branch"));
}

#[test]
fn branch_default_value_is_zero() {
    let node = TreeNode::branch(vec![]);
    assert_eq!(node.value(), 0);
}

#[test]
fn branch_with_symbol_value_matches_symbol() {
    let node = TreeNode::branch_with_symbol(88, vec![]);
    assert_eq!(node.value(), node.symbol());
}

#[test]
fn leaf_clone_is_independent() {
    let node = TreeNode::leaf(5);
    let mut cloned = node.clone();
    // Modify cloned through arena mutation to confirm independence
    let mut arena = TreeArena::new();
    let h = arena.alloc(cloned);
    arena.get_mut(h).set_value(99);
    // Original unchanged (we verify via symbol)
    assert_eq!(node.symbol(), 5);
    cloned = TreeNode::leaf(99);
    assert_ne!(node, cloned);
}

#[test]
fn arena_new_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_with_capacity_is_empty() {
    let arena = TreeArena::with_capacity(64);
    assert!(arena.is_empty());
}

#[test]
fn arena_with_capacity_has_capacity() {
    let arena = TreeArena::with_capacity(64);
    assert_eq!(arena.capacity(), 64);
}

#[test]
fn arena_default_is_empty() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
}

#[test]
fn arena_len_increments() {
    let mut arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
}

// ============================================================================
// 31–40. Arena mutation and get_mut
// ============================================================================

#[test]
fn get_mut_set_value_on_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
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
fn get_ref_returns_node_reference() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(55));
    let node_ref = arena.get(h);
    let inner: &TreeNode = node_ref.get_ref();
    assert_eq!(inner.symbol(), 55);
}

#[test]
fn get_as_ref_returns_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(33));
    let node_ref = arena.get(h);
    let inner: &TreeNode = node_ref.as_ref();
    assert_eq!(inner.symbol(), 33);
}

#[test]
fn deref_on_tree_node_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    let node_ref = arena.get(h);
    // Deref allows calling TreeNode methods directly
    assert_eq!(node_ref.symbol(), 7);
    assert!(node_ref.is_leaf());
}

// ============================================================================
// 41–50. Arena memory and capacity
// ============================================================================

#[test]
fn arena_memory_usage_positive() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn arena_num_chunks_starts_at_one() {
    let arena = TreeArena::new();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn arena_capacity_at_least_initial() {
    let arena = TreeArena::with_capacity(256);
    assert!(arena.capacity() >= 256);
}

#[test]
fn arena_metrics_snapshot() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let metrics = arena.metrics();
    assert_eq!(metrics.len(), 1);
}

#[test]
fn arena_reset_keeps_capacity() {
    let mut arena = TreeArena::with_capacity(128);
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    let cap_before = arena.capacity();
    arena.reset();
    assert_eq!(arena.capacity(), cap_before);
    assert!(arena.is_empty());
}

#[test]
fn arena_clear_reduces_to_one_chunk() {
    let mut arena = TreeArena::with_capacity(4);
    // Force multiple chunks
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

// ============================================================================
// 51–60. Complex tree structures
// ============================================================================

#[test]
fn nested_branch_structure() {
    let mut arena = TreeArena::new();
    let leaf1 = arena.alloc(TreeNode::leaf(1));
    let leaf2 = arena.alloc(TreeNode::leaf(2));
    let inner = arena.alloc(TreeNode::branch(vec![leaf1, leaf2]));
    let root = arena.alloc(TreeNode::branch_with_symbol(100, vec![inner]));

    assert_eq!(arena.get(root).symbol(), 100);
    assert_eq!(arena.get(root).children().len(), 1);

    let inner_handle = arena.get(root).children()[0];
    assert!(arena.get(inner_handle).is_branch());
    assert_eq!(arena.get(inner_handle).children().len(), 2);
}

#[test]
fn wide_branch() {
    let mut arena = TreeArena::new();
    let leaves: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(leaves));
    assert_eq!(arena.get(root).children().len(), 50);
}

#[test]
fn deep_chain() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..=20 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.get(current).symbol(), 20);
}

#[test]
fn branch_children_reference_valid_handles() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));

    for &child_handle in arena.get(parent).children() {
        // Each child handle should be dereferenceable
        let _ = arena.get(child_handle).value();
    }
}

#[test]
fn sibling_branches_share_child() {
    let mut arena = TreeArena::new();
    let shared = arena.alloc(TreeNode::leaf(42));
    let b1 = arena.alloc(TreeNode::branch(vec![shared]));
    let b2 = arena.alloc(TreeNode::branch(vec![shared]));

    assert_eq!(arena.get(b1).children()[0], arena.get(b2).children()[0]);
}

// ============================================================================
// 61–70. Edge cases and boundary tests
// ============================================================================

#[test]
fn alloc_single_then_clear_then_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h).value(), 2);
}

#[test]
fn repeated_clear_is_safe() {
    let mut arena = TreeArena::new();
    arena.clear();
    arena.clear();
    assert!(arena.is_empty());
}

#[test]
fn repeated_reset_is_safe() {
    let mut arena = TreeArena::new();
    arena.reset();
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn alloc_after_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn leaf_symbol_large_positive() {
    let node = TreeNode::leaf(1_000_000);
    assert_eq!(node.symbol(), 1_000_000);
}

#[test]
fn leaf_symbol_large_negative() {
    let node = TreeNode::leaf(-1_000_000);
    assert_eq!(node.symbol(), -1_000_000);
}

#[test]
fn branch_with_symbol_max() {
    let node = TreeNode::branch_with_symbol(i32::MAX, vec![]);
    assert_eq!(node.symbol(), i32::MAX);
}

#[test]
fn branch_with_symbol_min() {
    let node = TreeNode::branch_with_symbol(i32::MIN, vec![]);
    assert_eq!(node.symbol(), i32::MIN);
}

#[test]
fn node_handle_new_zero_zero() {
    let h = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    assert_eq!(h, h2);
}

#[test]
fn node_handle_new_large_indices() {
    let h = NodeHandle::new(u32::MAX, u32::MAX);
    let h2 = NodeHandle::new(u32::MAX, u32::MAX);
    assert_eq!(h, h2);
}

// ============================================================================
// 71–80. Arena stress and mixed operations
// ============================================================================

#[test]
fn arena_interleaved_leaf_and_branch() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let b1 = arena.alloc(TreeNode::branch(vec![l1]));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b2 = arena.alloc(TreeNode::branch(vec![l2, b1]));

    assert!(arena.get(l1).is_leaf());
    assert!(arena.get(b1).is_branch());
    assert!(arena.get(l2).is_leaf());
    assert!(arena.get(b2).is_branch());
    assert_eq!(arena.get(b2).children().len(), 2);
}

#[test]
fn arena_five_hundred_nodes() {
    let mut arena = TreeArena::new();
    for i in 0..500 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 500);
}

#[test]
fn arena_alloc_clear_cycle() {
    let mut arena = TreeArena::new();
    for _ in 0..5 {
        for i in 0..10 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.clear();
        assert!(arena.is_empty());
    }
}

#[test]
fn arena_alloc_reset_cycle() {
    let mut arena = TreeArena::new();
    for _ in 0..5 {
        for i in 0..10 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn collect_handles_in_hashset() {
    let mut arena = TreeArena::new();
    let mut set = HashSet::new();
    for i in 0..25 {
        let h = arena.alloc(TreeNode::leaf(i));
        set.insert(h);
    }
    assert_eq!(set.len(), 25);
}

#[test]
fn node_handle_debug_is_non_empty() {
    let h = NodeHandle::new(1, 2);
    let dbg = format!("{h:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn arena_debug_is_non_empty() {
    let arena = TreeArena::new();
    let dbg = format!("{arena:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn get_ref_and_as_ref_agree() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(77));
    let node_ref = arena.get(h);
    assert_eq!(node_ref.get_ref().symbol(), node_ref.as_ref().symbol());
}

#[test]
fn branch_is_not_leaf() {
    let node = TreeNode::branch(vec![]);
    assert!(!node.is_leaf());
}

#[test]
fn branch_with_symbol_is_not_leaf() {
    let node = TreeNode::branch_with_symbol(5, vec![]);
    assert!(!node.is_leaf());
}

// ============================================================================
// 81–85. Additional coverage
// ============================================================================

#[test]
fn arena_first_handle_retrieval() {
    let mut arena = TreeArena::new();
    let first = arena.alloc(TreeNode::leaf(0));
    for i in 1..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    // First handle still valid after many allocations
    assert_eq!(arena.get(first).value(), 0);
}

#[test]
fn arena_last_handle_retrieval() {
    let mut arena = TreeArena::new();
    let mut last = NodeHandle::new(0, 0);
    for i in 0..50 {
        last = arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.get(last).value(), 49);
}

#[test]
fn branch_children_slice_len() {
    let handles: Vec<NodeHandle> = (0..5).map(|i| NodeHandle::new(0, i)).collect();
    let node = TreeNode::branch(handles);
    assert_eq!(node.children().len(), 5);
}

#[test]
fn clone_branch_preserves_children_count() {
    let handles: Vec<NodeHandle> = (0..3).map(|i| NodeHandle::new(0, i)).collect();
    let node = TreeNode::branch_with_symbol(7, handles);
    let cloned = node.clone();
    assert_eq!(cloned.children().len(), 3);
    assert_eq!(cloned.symbol(), 7);
}

#[test]
fn arena_with_capacity_one() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.len(), 2);
}
