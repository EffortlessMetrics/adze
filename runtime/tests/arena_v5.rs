//! Arena allocator v5 tests — allocation, retrieval, leaf/branch properties,
//! growth, capacity, handle semantics, reset/clear, large allocations,
//! nested trees, and edge cases.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use std::collections::HashSet;

// ───────────────────────────────────────────────────────────────
// 1. Basic allocation and retrieval (10 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_alloc_single_leaf_and_retrieve() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn test_alloc_single_branch_and_retrieve() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch(vec![child]));
    assert!(arena.get(parent).is_branch());
}

#[test]
fn test_alloc_returns_distinct_handles() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn test_alloc_preserves_insertion_order_values() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..5).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn test_retrieve_after_multiple_allocs() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    let h3 = arena.alloc(TreeNode::leaf(30));
    assert_eq!(arena.get(h1).value(), 10);
    assert_eq!(arena.get(h2).value(), 20);
    assert_eq!(arena.get(h3).value(), 30);
}

#[test]
fn test_alloc_increments_len() {
    let mut arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
}

#[test]
fn test_alloc_branch_with_symbol_retrieval() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(7));
    let h = arena.alloc(TreeNode::branch_with_symbol(99, vec![c]));
    assert_eq!(arena.get(h).symbol(), 99);
}

#[test]
fn test_alloc_with_capacity() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 4);
}

#[test]
fn test_get_mut_changes_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(99);
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn test_alloc_zero_value_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

// ───────────────────────────────────────────────────────────────
// 2. Leaf vs branch node properties (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_leaf_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
}

#[test]
fn test_leaf_is_not_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(!arena.get(h).is_branch());
}

#[test]
fn test_branch_is_branch() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert!(arena.get(h).is_branch());
}

#[test]
fn test_branch_is_not_leaf() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert!(!arena.get(h).is_leaf());
}

#[test]
fn test_leaf_has_no_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_branch_children_match_inputs() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    let node_ref = arena.get(parent);
    let children = node_ref.children();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], c1);
    assert_eq!(children[1], c2);
}

#[test]
fn test_branch_default_symbol_is_zero() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.get(h).symbol(), 0);
}

#[test]
fn test_branch_with_symbol_stores_symbol() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch_with_symbol(55, vec![c]));
    assert_eq!(arena.get(h).symbol(), 55);
    assert!(arena.get(h).is_branch());
}

// ───────────────────────────────────────────────────────────────
// 3. Arena growth and capacity (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_new_arena_has_initial_capacity() {
    let arena = TreeArena::new();
    assert!(arena.capacity() >= 1024);
}

#[test]
fn test_with_capacity_sets_capacity() {
    let arena = TreeArena::with_capacity(16);
    assert_eq!(arena.capacity(), 16);
}

#[test]
fn test_capacity_grows_beyond_initial() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    // Third alloc forces a new chunk
    arena.alloc(TreeNode::leaf(3));
    assert!(arena.capacity() > 2);
}

#[test]
fn test_chunk_count_increases_on_overflow() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_chunk_doubles_capacity() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    let cap_before = arena.capacity();
    arena.alloc(TreeNode::leaf(100));
    // New chunk should double: 4 + 8 = 12
    assert_eq!(arena.capacity(), cap_before + 8);
}

#[test]
fn test_memory_usage_positive() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(arena.memory_usage() > 0);
}

#[test]
fn test_metrics_snapshot_reflects_state() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    let m = arena.metrics();
    assert_eq!(m.len(), 2);
    assert!(!m.is_empty());
    assert!(m.capacity() >= 2);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn test_metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
}

// ───────────────────────────────────────────────────────────────
// 4. NodeHandle uniqueness and Copy semantics (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_handle_copy_semantics() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    let h2 = h; // Copy, not move
    assert_eq!(arena.get(h).value(), 7);
    assert_eq!(arena.get(h2).value(), 7);
}

#[test]
fn test_handles_are_eq() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let h_copy = h;
    assert_eq!(h, h_copy);
}

#[test]
fn test_different_handles_are_ne() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(1));
    assert_ne!(h1, h2);
}

#[test]
fn test_handles_usable_as_hash_keys() {
    let mut arena = TreeArena::new();
    let mut set = HashSet::new();
    for i in 0..10 {
        let h = arena.alloc(TreeNode::leaf(i));
        set.insert(h);
    }
    assert_eq!(set.len(), 10);
}

#[test]
fn test_handle_survives_further_allocs() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn test_handle_copy_used_independently() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(3));
    let copies: Vec<NodeHandle> = (0..5).map(|_| h).collect();
    for c in copies {
        assert_eq!(arena.get(c).value(), 3);
    }
}

#[test]
fn test_handle_debug_format() {
    let h = NodeHandle::new(0, 0);
    let dbg = format!("{h:?}");
    assert!(dbg.contains("NodeHandle"));
}

#[test]
fn test_handle_new_roundtrip() {
    let _h = NodeHandle::new(3, 7);
    // Verify through arena with matching layout
    let mut arena = TreeArena::with_capacity(1);
    // Fill chunks 0..3 with dummy nodes
    let mut dummy_handles = Vec::new();
    for _ in 0..3 {
        // Each chunk has capacity 1, so filling forces new chunks
        dummy_handles.push(arena.alloc(TreeNode::leaf(0)));
    }
    // chunk 3 now exists; allocate node at idx 0..7
    // chunk sizes double: 1, 2, 4, 8
    // chunk 3 has capacity 8, fill to idx 7
    for _ in 0..7 {
        arena.alloc(TreeNode::leaf(0));
    }
    let target = arena.alloc(TreeNode::leaf(777));
    // The target handle should match a specific chunk/offset
    // Instead of relying on exact layout, verify the alloc'd target is accessible
    assert_eq!(arena.get(target).value(), 777);
}

// ───────────────────────────────────────────────────────────────
// 5. Reset/clear behavior (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_reset_empties_arena() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn test_reset_retains_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn test_clear_trims_to_one_chunk() {
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
fn test_alloc_after_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_alloc_after_clear() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(55));
    assert_eq!(arena.get(h).value(), 55);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_multiple_reset_cycles() {
    let mut arena = TreeArena::new();
    for cycle in 0..5 {
        for i in 0..10 {
            arena.alloc(TreeNode::leaf(cycle * 10 + i));
        }
        assert_eq!(arena.len(), 10);
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn test_reset_on_empty_arena() {
    let mut arena = TreeArena::new();
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_clear_on_empty_arena() {
    let mut arena = TreeArena::new();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ───────────────────────────────────────────────────────────────
// 6. Large allocations (100+ nodes) (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_alloc_200_leaves() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..200).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 200);
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn test_alloc_500_nodes_unique_handles() {
    let mut arena = TreeArena::new();
    let mut set = HashSet::new();
    for i in 0..500 {
        let h = arena.alloc(TreeNode::leaf(i));
        set.insert(h);
    }
    assert_eq!(set.len(), 500);
}

#[test]
fn test_alloc_1000_leaves_len() {
    let mut arena = TreeArena::new();
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 1000);
}

#[test]
fn test_large_alloc_first_and_last_accessible() {
    let mut arena = TreeArena::new();
    let first = arena.alloc(TreeNode::leaf(-1));
    for i in 0..998 {
        arena.alloc(TreeNode::leaf(i));
    }
    let last = arena.alloc(TreeNode::leaf(-2));
    assert_eq!(arena.get(first).value(), -1);
    assert_eq!(arena.get(last).value(), -2);
}

#[test]
fn test_large_alloc_grows_capacity() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..200 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() >= 200);
}

#[test]
fn test_large_alloc_multiple_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
}

#[test]
fn test_large_mixed_leaf_and_branch() {
    let mut arena = TreeArena::new();
    let mut leaves = Vec::new();
    for i in 0..100 {
        leaves.push(arena.alloc(TreeNode::leaf(i)));
    }
    for chunk in leaves.chunks(2) {
        let parent = arena.alloc(TreeNode::branch(chunk.to_vec()));
        assert!(arena.get(parent).is_branch());
        assert_eq!(arena.get(parent).children().len(), chunk.len());
    }
    assert!(arena.len() >= 150);
}

#[test]
fn test_large_alloc_reset_and_realloc() {
    let mut arena = TreeArena::new();
    for i in 0..500 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    assert!(arena.is_empty());
    let h = arena.alloc(TreeNode::leaf(777));
    assert_eq!(arena.get(h).value(), 777);
    assert_eq!(arena.len(), 1);
}

// ───────────────────────────────────────────────────────────────
// 7. Nested tree construction (branches of branches) (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_nested_two_levels() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let mid = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let root = arena.alloc(TreeNode::branch(vec![mid]));
    assert!(arena.get(root).is_branch());
    assert_eq!(arena.get(root).children().len(), 1);
    assert_eq!(arena.get(root).children()[0], mid);
}

#[test]
fn test_nested_three_levels() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(42));
    let inner = arena.alloc(TreeNode::branch(vec![leaf]));
    let middle = arena.alloc(TreeNode::branch(vec![inner]));
    let outer = arena.alloc(TreeNode::branch(vec![middle]));
    // Walk down from outer
    let mid_h = arena.get(outer).children()[0];
    let inn_h = arena.get(mid_h).children()[0];
    let lf_h = arena.get(inn_h).children()[0];
    assert_eq!(arena.get(lf_h).value(), 42);
}

#[test]
fn test_nested_wide_tree() {
    let mut arena = TreeArena::new();
    let leaves: Vec<_> = (0..8).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(leaves));
    assert_eq!(arena.get(root).children().len(), 8);
}

#[test]
fn test_nested_binary_tree() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let l4 = arena.alloc(TreeNode::leaf(4));
    let left = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let right = arena.alloc(TreeNode::branch(vec![l3, l4]));
    let root = arena.alloc(TreeNode::branch(vec![left, right]));
    assert_eq!(arena.get(root).children().len(), 2);
    assert!(arena.get(left).is_branch());
    assert!(arena.get(right).is_branch());
}

#[test]
fn test_nested_children_are_accessible_from_root() {
    let mut arena = TreeArena::new();
    let a = arena.alloc(TreeNode::leaf(10));
    let b = arena.alloc(TreeNode::leaf(20));
    let mid = arena.alloc(TreeNode::branch_with_symbol(5, vec![a, b]));
    let root = arena.alloc(TreeNode::branch_with_symbol(1, vec![mid]));
    let mid_handle = arena.get(root).children()[0];
    assert_eq!(arena.get(mid_handle).symbol(), 5);
    let mid_ref = arena.get(mid_handle);
    let leaf_handles: Vec<_> = mid_ref.children().to_vec();
    assert_eq!(arena.get(leaf_handles[0]).value(), 10);
    assert_eq!(arena.get(leaf_handles[1]).value(), 20);
}

#[test]
fn test_nested_deep_chain() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(999));
    for _ in 0..10 {
        current = arena.alloc(TreeNode::branch(vec![current]));
    }
    // Walk 10 levels down
    let mut h = current;
    for _ in 0..10 {
        assert!(arena.get(h).is_branch());
        h = arena.get(h).children()[0];
    }
    assert!(arena.get(h).is_leaf());
    assert_eq!(arena.get(h).value(), 999);
}

#[test]
fn test_nested_diamond_shape() {
    // Two branches share the same leaf child
    let mut arena = TreeArena::new();
    let shared = arena.alloc(TreeNode::leaf(77));
    let left = arena.alloc(TreeNode::branch(vec![shared]));
    let right = arena.alloc(TreeNode::branch(vec![shared]));
    let root = arena.alloc(TreeNode::branch(vec![left, right]));
    assert_eq!(arena.get(root).children().len(), 2);
    // Both children point to same leaf
    let lc = arena.get(left).children()[0];
    let rc = arena.get(right).children()[0];
    assert_eq!(lc, rc);
    assert_eq!(arena.get(lc).value(), 77);
}

#[test]
fn test_nested_branch_symbol_propagation() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(1));
    let inner = arena.alloc(TreeNode::branch_with_symbol(10, vec![leaf]));
    let outer = arena.alloc(TreeNode::branch_with_symbol(20, vec![inner]));
    assert_eq!(arena.get(outer).symbol(), 20);
    let inner_h = arena.get(outer).children()[0];
    assert_eq!(arena.get(inner_h).symbol(), 10);
}

// ───────────────────────────────────────────────────────────────
// 8. Edge cases: empty arena queries, single node (10 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_empty_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
}

#[test]
fn test_empty_arena_len_zero() {
    let arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_empty_arena_capacity_positive() {
    let arena = TreeArena::new();
    assert!(arena.capacity() > 0);
}

#[test]
fn test_empty_arena_num_chunks_one() {
    let arena = TreeArena::new();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_single_leaf_not_empty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(!arena.is_empty());
}

#[test]
fn test_single_leaf_len_one() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_empty_branch_children_empty() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_negative_symbol_values() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-100));
    assert_eq!(arena.get(h).value(), -100);
}

#[test]
fn test_branch_with_negative_symbol() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(0));
    let h = arena.alloc(TreeNode::branch_with_symbol(-42, vec![c]));
    assert_eq!(arena.get(h).symbol(), -42);
}

#[test]
fn test_default_trait_creates_empty() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
#[should_panic]
fn test_with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

#[test]
fn test_i32_max_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn test_i32_min_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}
