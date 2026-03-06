//! Stress tests for TreeArena (v8)
//!
//! 80+ tests covering allocation counts, chunk growth, node retrieval,
//! tree structures (deep, wide, mixed), clear/reset cycles, and edge cases.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

/// Default chunk size used by `TreeArena::new()`.
const DEFAULT_CHUNK_SIZE: usize = 1024;

// ===========================================================================
// 1–3. Basic allocation counts
// ===========================================================================

#[test]
fn test_alloc_one_node_len_is_one() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_alloc_100_nodes_len() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 100);
}

#[test]
fn test_alloc_1000_nodes_len() {
    let mut arena = TreeArena::new();
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 1000);
}

// ===========================================================================
// 4–6. Chunk growth
// ===========================================================================

#[test]
fn test_alloc_default_chunk_size_one_chunk() {
    let mut arena = TreeArena::new();
    for i in 0..DEFAULT_CHUNK_SIZE as i32 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_alloc_default_chunk_size_plus_one_two_chunks() {
    let mut arena = TreeArena::new();
    for i in 0..=DEFAULT_CHUNK_SIZE as i32 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_alloc_5000_nodes_correct_chunk_count() {
    let mut arena = TreeArena::new();
    for i in 0..5000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 5000);
    // Chunk sizes: 1024, 2048 → total 3072 after 2 chunks.
    // Need a third chunk for nodes 3073..5000.
    assert!(arena.num_chunks() >= 3);
}

// ===========================================================================
// 7–8. get() returns correct data
// ===========================================================================

#[test]
fn test_get_returns_correct_leaf_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn test_alloc_then_get_1000_nodes_all_correct() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

// ===========================================================================
// 9. with_capacity
// ===========================================================================

#[test]
fn test_with_capacity_2048_one_chunk() {
    let arena = TreeArena::with_capacity(2048);
    assert_eq!(arena.num_chunks(), 1);
    assert_eq!(arena.capacity(), 2048);
}

// ===========================================================================
// 10–11. clear() behaviour
// ===========================================================================

#[test]
fn test_clear_resets_len_to_zero() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn test_after_clear_re_allocate_works() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 99);
}

// ===========================================================================
// 12. Parent-child relationships
// ===========================================================================

#[test]
fn test_branch_children_relationship() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], c1);
    assert_eq!(children[1], c2);
}

// ===========================================================================
// 13. Multiple clears and re-fills
// ===========================================================================

#[test]
fn test_multiple_clear_refill_cycles() {
    let mut arena = TreeArena::new();
    for cycle in 0..5 {
        for i in 0..200 {
            arena.alloc(TreeNode::leaf(cycle * 1000 + i));
        }
        assert_eq!(arena.len(), 200);
        arena.clear();
        assert!(arena.is_empty());
    }
}

// ===========================================================================
// 14. Large symbol values
// ===========================================================================

#[test]
fn test_large_symbol_values() {
    let mut arena = TreeArena::new();
    let h0 = arena.alloc(TreeNode::leaf(0));
    let h_max = arena.alloc(TreeNode::leaf(i32::MAX));
    let h_min = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h0).value(), 0);
    assert_eq!(arena.get(h_max).value(), i32::MAX);
    assert_eq!(arena.get(h_min).value(), i32::MIN);
}

// ===========================================================================
// 15. Various symbol_id values
// ===========================================================================

#[test]
fn test_symbol_value_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).symbol(), 0);
}

#[test]
fn test_symbol_value_one() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).symbol(), 1);
}

#[test]
fn test_symbol_value_100() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(100));
    assert_eq!(arena.get(h).symbol(), 100);
}

#[test]
fn test_symbol_value_65535() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(65535));
    assert_eq!(arena.get(h).symbol(), 65535);
}

#[test]
fn test_branch_with_symbol_value() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch_with_symbol(999, vec![c]));
    assert_eq!(arena.get(h).symbol(), 999);
}

// ===========================================================================
// 16. Nodes with multiple children
// ===========================================================================

#[test]
fn test_branch_with_four_children() {
    let mut arena = TreeArena::new();
    let kids: Vec<NodeHandle> = (0..4).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(kids.clone()));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 4);
    for (i, &ch) in children.iter().enumerate() {
        assert_eq!(arena.get(ch).value(), i as i32);
    }
}

#[test]
fn test_branch_with_ten_children() {
    let mut arena = TreeArena::new();
    let kids: Vec<NodeHandle> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(kids.clone()));
    assert_eq!(arena.get(parent).children().len(), 10);
}

// ===========================================================================
// 17. Deep tree (depth 50+)
// ===========================================================================

#[test]
fn test_deep_tree_depth_50() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for depth in 1..=50 {
        current = arena.alloc(TreeNode::branch_with_symbol(depth, vec![current]));
    }
    // Walk back from root
    let mut node_ref = arena.get(current);
    for expected in (1..=50).rev() {
        assert_eq!(node_ref.symbol(), expected);
        let kids = node_ref.children();
        assert_eq!(kids.len(), 1);
        node_ref = arena.get(kids[0]);
    }
    assert_eq!(node_ref.value(), 0);
    assert!(node_ref.is_leaf());
}

#[test]
fn test_deep_tree_depth_100() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(-1));
    for depth in 1..=100 {
        current = arena.alloc(TreeNode::branch_with_symbol(depth, vec![current]));
    }
    assert_eq!(arena.len(), 101);
    assert_eq!(arena.get(current).symbol(), 100);
}

// ===========================================================================
// 18. Wide tree (100+ children)
// ===========================================================================

#[test]
fn test_wide_tree_100_children() {
    let mut arena = TreeArena::new();
    let kids: Vec<NodeHandle> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(kids));
    let root_ref = arena.get(root);
    let children = root_ref.children();
    assert_eq!(children.len(), 100);
    for (i, &ch) in children.iter().enumerate() {
        assert_eq!(arena.get(ch).value(), i as i32);
    }
}

#[test]
fn test_wide_tree_500_children() {
    let mut arena = TreeArena::new();
    let kids: Vec<NodeHandle> = (0..500).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(kids));
    assert_eq!(arena.get(root).children().len(), 500);
    assert_eq!(arena.len(), 501);
}

// ===========================================================================
// 19. Mixed tree shapes
// ===========================================================================

#[test]
fn test_mixed_tree_shape() {
    let mut arena = TreeArena::new();
    // Two deep branches and one wide branch under the same root
    let deep_leaf = arena.alloc(TreeNode::leaf(1));
    let deep_mid = arena.alloc(TreeNode::branch(vec![deep_leaf]));
    let deep_top = arena.alloc(TreeNode::branch(vec![deep_mid]));

    let wide_kids: Vec<NodeHandle> = (10..20).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let wide_top = arena.alloc(TreeNode::branch(wide_kids));

    let single = arena.alloc(TreeNode::leaf(99));
    let root = arena.alloc(TreeNode::branch(vec![deep_top, wide_top, single]));

    let root_ref = arena.get(root);
    let root_children = root_ref.children();
    assert_eq!(root_children.len(), 3);
    assert!(arena.get(root_children[0]).is_branch());
    assert!(arena.get(root_children[1]).is_branch());
    assert!(arena.get(root_children[2]).is_leaf());
}

// ===========================================================================
// 20. num_chunks scales correctly
// ===========================================================================

#[test]
fn test_num_chunks_grows_with_allocations() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..8 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 1);

    arena.alloc(TreeNode::leaf(8));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_small_capacity_many_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 100);
    assert!(arena.num_chunks() >= 2);
}

// ===========================================================================
// 21–30. reset() tests
// ===========================================================================

#[test]
fn test_reset_clears_len() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn test_reset_retains_chunks() {
    let mut arena = TreeArena::new();
    // Force multiple chunks
    for i in 0..(DEFAULT_CHUNK_SIZE as i32 + 100) {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn test_clear_frees_excess_chunks() {
    let mut arena = TreeArena::new();
    for i in 0..(DEFAULT_CHUNK_SIZE as i32 + 100) {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() >= 2);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_reset_then_alloc_works() {
    let mut arena = TreeArena::new();
    for i in 0..500 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(777));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 777);
}

#[test]
fn test_multiple_reset_cycles() {
    let mut arena = TreeArena::new();
    for _cycle in 0..10 {
        for i in 0..100 {
            arena.alloc(TreeNode::leaf(i));
        }
        assert_eq!(arena.len(), 100);
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn test_reset_after_single_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn test_reset_empty_arena_is_noop() {
    let mut arena = TreeArena::new();
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_clear_empty_arena_is_noop() {
    let mut arena = TreeArena::new();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_double_reset() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn test_double_clear() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 31–40. Leaf / branch property checks
// ===========================================================================

#[test]
fn test_leaf_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    assert!(arena.get(h).is_leaf());
}

#[test]
fn test_leaf_is_not_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
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
    let h = arena.alloc(TreeNode::leaf(7));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_branch_no_children_is_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_branch_with_symbol_stores_symbol() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch_with_symbol(42, vec![c]));
    assert_eq!(arena.get(h).symbol(), 42);
    assert!(arena.get(h).is_branch());
}

#[test]
fn test_branch_default_symbol_is_zero() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.get(h).symbol(), 0);
}

#[test]
fn test_value_and_symbol_agree() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(123));
    assert_eq!(arena.get(h).value(), arena.get(h).symbol());
}

#[test]
fn test_branch_value_and_symbol_agree() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch_with_symbol(55, vec![c]));
    assert_eq!(arena.get(h).value(), arena.get(h).symbol());
}

// ===========================================================================
// 41–50. Capacity and metrics
// ===========================================================================

#[test]
fn test_new_arena_capacity_is_default_chunk_size() {
    let arena = TreeArena::new();
    assert_eq!(arena.capacity(), DEFAULT_CHUNK_SIZE);
}

#[test]
fn test_with_capacity_sets_requested_capacity() {
    let arena = TreeArena::with_capacity(512);
    assert_eq!(arena.capacity(), 512);
}

#[test]
fn test_capacity_grows_after_overflow() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..9 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() > 8);
}

#[test]
fn test_memory_usage_positive() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn test_memory_usage_grows_with_capacity() {
    let small = TreeArena::with_capacity(16);
    let large = TreeArena::with_capacity(4096);
    assert!(large.memory_usage() > small.memory_usage());
}

#[test]
fn test_metrics_len() {
    let mut arena = TreeArena::new();
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.metrics().len(), 10);
}

#[test]
fn test_metrics_is_empty_true() {
    let arena = TreeArena::new();
    assert!(arena.metrics().is_empty());
}

#[test]
fn test_metrics_is_empty_false() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(!arena.metrics().is_empty());
}

#[test]
fn test_metrics_num_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.metrics().num_chunks(), 2);
}

#[test]
fn test_default_arena_equals_new() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

// ===========================================================================
// 51–60. Handle identity and stability
// ===========================================================================

#[test]
fn test_different_allocs_produce_different_handles() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn test_handles_stable_after_further_allocs() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    for i in 2..500 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.get(h1).value(), 1);
}

#[test]
fn test_handles_stable_across_chunk_boundary() {
    let mut arena = TreeArena::with_capacity(8);
    let first = arena.alloc(TreeNode::leaf(0));
    // Fill first chunk and spill into second
    for i in 1..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.get(first).value(), 0);
}

#[test]
fn test_handle_copy_semantics() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    let h2 = h; // Copy, not move
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.get(h2).value(), 42);
}

#[test]
fn test_handle_equality() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let copy = h;
    assert_eq!(h, copy);
}

#[test]
fn test_1000_unique_handles() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let unique: std::collections::HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(unique.len(), 1000);
}

#[test]
fn test_handle_debug_format() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let dbg = format!("{h:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn test_handle_hash_works() {
    use std::collections::HashMap;
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    let mut map = HashMap::new();
    map.insert(h1, "ten");
    map.insert(h2, "twenty");
    assert_eq!(map[&h1], "ten");
    assert_eq!(map[&h2], "twenty");
}

#[test]
fn test_get_after_many_cross_chunk_allocs() {
    let mut arena = TreeArena::with_capacity(16);
    let handles: Vec<NodeHandle> = (0..200).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn test_interleaved_leaf_branch_retrieval() {
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

// ===========================================================================
// 61–70. Stress: large allocations
// ===========================================================================

#[test]
fn test_alloc_2048_nodes() {
    let mut arena = TreeArena::new();
    for i in 0..2048 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 2048);
}

#[test]
fn test_alloc_4096_nodes() {
    let mut arena = TreeArena::new();
    for i in 0..4096 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 4096);
}

#[test]
fn test_alloc_10000_nodes() {
    let mut arena = TreeArena::new();
    for i in 0..10000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 10000);
}

#[test]
fn test_alloc_10000_and_verify_last() {
    let mut arena = TreeArena::new();
    let mut last = arena.alloc(TreeNode::leaf(0));
    for i in 1..10000 {
        last = arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.get(last).value(), 9999);
}

#[test]
fn test_alloc_10000_and_verify_first() {
    let mut arena = TreeArena::new();
    let first = arena.alloc(TreeNode::leaf(-1));
    for i in 1..10000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.get(first).value(), -1);
}

#[test]
fn test_stress_branch_with_1000_children() {
    let mut arena = TreeArena::new();
    let kids: Vec<NodeHandle> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(kids));
    assert_eq!(arena.get(root).children().len(), 1000);
    assert_eq!(arena.len(), 1001);
}

#[test]
fn test_alternating_leaf_branch_1000() {
    let mut arena = TreeArena::new();
    let mut prev = arena.alloc(TreeNode::leaf(0));
    for i in 1..1000 {
        if i % 2 == 0 {
            prev = arena.alloc(TreeNode::leaf(i));
        } else {
            prev = arena.alloc(TreeNode::branch(vec![prev]));
        }
    }
    assert_eq!(arena.len(), 1000);
    // Verify last node is accessible
    let _ = arena.get(prev);
}

#[test]
fn test_deep_tree_depth_200() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for depth in 1..=200 {
        current = arena.alloc(TreeNode::branch_with_symbol(depth, vec![current]));
    }
    assert_eq!(arena.len(), 201);
    assert_eq!(arena.get(current).symbol(), 200);
}

#[test]
fn test_wide_then_deep_combined() {
    let mut arena = TreeArena::new();
    // Wide: 50 leaves under one branch
    let wide_kids: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let wide = arena.alloc(TreeNode::branch(wide_kids));

    // Deep: chain of 50 branches
    let mut chain = arena.alloc(TreeNode::leaf(100));
    for i in 1..=50 {
        chain = arena.alloc(TreeNode::branch_with_symbol(100 + i, vec![chain]));
    }

    let root = arena.alloc(TreeNode::branch(vec![wide, chain]));
    assert_eq!(arena.get(root).children().len(), 2);
    // 50 wide leaves + 1 wide branch + 1 deep leaf + 50 deep branches + 1 root
    assert_eq!(arena.len(), 103);
}

#[test]
fn test_clear_after_large_stress() {
    let mut arena = TreeArena::new();
    for i in 0..5000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(!arena.is_empty());
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 71–80. Edge cases and misc
// ===========================================================================

#[test]
fn test_with_capacity_1() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 1);
}

#[test]
fn test_with_capacity_1_overflow() {
    let mut arena = TreeArena::with_capacity(1);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_branch_with_empty_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).children().is_empty());
    assert!(arena.get(h).is_branch());
}

#[test]
fn test_negative_symbol_values() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-1));
    assert_eq!(arena.get(h).value(), -1);
}

#[test]
fn test_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn test_capacity_never_shrinks_on_alloc() {
    let mut arena = TreeArena::new();
    let mut prev_cap = arena.capacity();
    for i in 0..3000 {
        arena.alloc(TreeNode::leaf(i));
        let cap = arena.capacity();
        assert!(cap >= prev_cap);
        prev_cap = cap;
    }
}

#[test]
fn test_new_arena_not_empty_after_one_alloc() {
    let mut arena = TreeArena::new();
    assert!(arena.is_empty());
    arena.alloc(TreeNode::leaf(0));
    assert!(!arena.is_empty());
}

#[test]
fn test_reset_then_build_tree() {
    let mut arena = TreeArena::new();
    // First parse
    let l = arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::branch(vec![l]));
    arena.reset();

    // Second parse — completely new tree
    let a = arena.alloc(TreeNode::leaf(10));
    let b = arena.alloc(TreeNode::leaf(20));
    let root = arena.alloc(TreeNode::branch(vec![a, b]));
    assert_eq!(arena.len(), 3);
    assert_eq!(arena.get(root).children().len(), 2);
    assert_eq!(arena.get(a).value(), 10);
    assert_eq!(arena.get(b).value(), 20);
}

#[test]
fn test_many_small_branches() {
    let mut arena = TreeArena::new();
    let mut branches = Vec::new();
    for i in 0..200 {
        let leaf = arena.alloc(TreeNode::leaf(i));
        let branch = arena.alloc(TreeNode::branch(vec![leaf]));
        branches.push(branch);
    }
    assert_eq!(arena.len(), 400);
    for (i, &b) in branches.iter().enumerate() {
        let branch_ref = arena.get(b);
        let kids = branch_ref.children();
        assert_eq!(kids.len(), 1);
        let child_handle = kids[0];
        assert_eq!(arena.get(child_handle).value(), i as i32);
    }
}

#[test]
fn test_clear_reset_interleaved() {
    let mut arena = TreeArena::new();
    for i in 0..2000 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    assert!(arena.is_empty());
    assert!(arena.num_chunks() >= 2);

    for i in 0..500 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);

    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

// ===========================================================================
// 81–85. Additional coverage
// ===========================================================================

#[test]
fn test_chunk_count_with_capacity_2() {
    let mut arena = TreeArena::with_capacity(2);
    // cap 2, then 4, then 8, …
    for i in 0..7 {
        arena.alloc(TreeNode::leaf(i));
    }
    // 2 + 4 = 6, so 7 nodes need 3 chunks
    assert_eq!(arena.num_chunks(), 3);
}

#[test]
fn test_branch_children_point_to_correct_leaves() {
    let mut arena = TreeArena::new();
    let a = arena.alloc(TreeNode::leaf(111));
    let b = arena.alloc(TreeNode::leaf(222));
    let c = arena.alloc(TreeNode::leaf(333));
    let root = arena.alloc(TreeNode::branch(vec![a, b, c]));
    let root_ref = arena.get(root);
    let kids = root_ref.children();
    let k0 = kids[0];
    let k1 = kids[1];
    let k2 = kids[2];
    assert_eq!(arena.get(k0).value(), 111);
    assert_eq!(arena.get(k1).value(), 222);
    assert_eq!(arena.get(k2).value(), 333);
}

#[test]
fn test_tree_node_clone() {
    let node = TreeNode::leaf(42);
    let node2 = node.clone();
    assert_eq!(node, node2);
}

#[test]
fn test_tree_node_debug() {
    let node = TreeNode::leaf(7);
    let dbg = format!("{node:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn test_arena_metrics_after_stress() {
    let mut arena = TreeArena::new();
    for i in 0..3000 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.len(), 3000);
    assert!(m.num_chunks() >= 2);
    assert!(!m.is_empty());
}
