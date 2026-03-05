//! Stress and property-based tests for TreeArena (v9)
//!
//! 80+ tests covering allocation counts, chunk growth, node retrieval,
//! property-based roundtrips, tree structures, clear/reset cycles,
//! metrics, and edge cases.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::HashSet;

/// Mirror of the private `DEFAULT_CHUNK_SIZE` in `arena_allocator.rs`.
const DEFAULT_CHUNK_SIZE: usize = 1024;

// ── helpers ─────────────────────────────────────────────────────────────

// ===========================================================================
// 1–3. Basic allocation counts
// ===========================================================================

#[test]
fn test_alloc_single_node_len_is_one() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_alloc_100_nodes_len_is_100() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 100);
}

#[test]
fn test_alloc_10000_nodes_stress() {
    let mut arena = TreeArena::new();
    for i in 0..10_000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 10_000);
}

// ===========================================================================
// 4. get() returns what was allocated
// ===========================================================================

#[test]
fn test_get_returns_allocated_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

// ===========================================================================
// 5–9. proptest: alloc then get preserves fields
// ===========================================================================

proptest! {
    #[test]
    fn pt_alloc_get_preserves_symbol(sym in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    #[test]
    fn pt_alloc_get_preserves_value(sym in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn pt_alloc_get_preserves_is_leaf(sym in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).is_leaf());
    }

    #[test]
    fn pt_alloc_get_preserves_is_branch(sym in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
    }

    #[test]
    fn pt_alloc_get_branch_preserves_symbol(sym in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }
}

// ===========================================================================
// 10. NodeHandle equality
// ===========================================================================

#[test]
fn test_node_handle_equality() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h1_copy = h1;
    assert_eq!(h1, h1_copy);
    assert_ne!(h1, h2);
}

// ===========================================================================
// 11. NodeHandle in HashSet
// ===========================================================================

#[test]
fn test_node_handle_in_hashset() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let set: HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(set.len(), 50);
    for h in &handles {
        assert!(set.contains(h));
    }
}

// ===========================================================================
// 12. Clear removes all → len 0
// ===========================================================================

#[test]
fn test_clear_removes_all() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

// ===========================================================================
// 13. Clear then alloc works
// ===========================================================================

#[test]
fn test_clear_then_alloc() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(999));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 999);
}

// ===========================================================================
// 14–15. with_capacity edge cases
// ===========================================================================

#[test]
fn test_with_capacity_1_works() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(7));
    assert_eq!(arena.get(h).value(), 7);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_with_capacity_10000_works() {
    let arena = TreeArena::with_capacity(10_000);
    assert_eq!(arena.capacity(), 10_000);
    assert!(arena.is_empty());
}

// ===========================================================================
// 16. After many allocs, num_chunks grows
// ===========================================================================

#[test]
fn test_many_allocs_grow_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
}

// ===========================================================================
// 17. DEFAULT_CHUNK_SIZE is 1024
// ===========================================================================

#[test]
fn test_default_chunk_size_is_1024() {
    assert_eq!(DEFAULT_CHUNK_SIZE, 1024);
    let arena = TreeArena::new();
    assert_eq!(arena.capacity(), 1024);
}

// ===========================================================================
// 18. Alloc exactly DEFAULT_CHUNK_SIZE → 1 chunk
// ===========================================================================

#[test]
fn test_alloc_exactly_default_chunk_size_one_chunk() {
    let mut arena = TreeArena::new();
    for i in 0..DEFAULT_CHUNK_SIZE as i32 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 19. Alloc DEFAULT_CHUNK_SIZE + 1 → 2 chunks
// ===========================================================================

#[test]
fn test_alloc_default_chunk_size_plus_one_two_chunks() {
    let mut arena = TreeArena::new();
    for i in 0..=DEFAULT_CHUNK_SIZE as i32 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.num_chunks(), 2);
}

// ===========================================================================
// 20. Multiple clear/alloc cycles
// ===========================================================================

#[test]
fn test_multiple_clear_alloc_cycles() {
    let mut arena = TreeArena::new();
    for cycle in 0..10 {
        for i in 0..200 {
            arena.alloc(TreeNode::leaf(cycle * 1000 + i));
        }
        assert_eq!(arena.len(), 200);
        arena.clear();
        assert!(arena.is_empty());
    }
}

// ===========================================================================
// 21. Reset retains chunks, clear drops excess
// ===========================================================================

#[test]
fn test_reset_retains_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    assert!(chunks_before > 1);
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn test_clear_drops_excess_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 23. Default trait
// ===========================================================================

#[test]
fn test_default_creates_arena() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 24–25. Leaf vs Branch discrimination
// ===========================================================================

#[test]
fn test_leaf_is_not_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn test_branch_is_not_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(!arena.get(h).is_leaf());
}

// ===========================================================================
// 26. Leaf children is empty
// ===========================================================================

#[test]
fn test_leaf_children_empty() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    assert!(arena.get(h).children().is_empty());
}

// ===========================================================================
// 27–28. Branch with children
// ===========================================================================

#[test]
fn test_branch_preserves_children() {
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

#[test]
fn test_branch_with_symbol_preserves_children_and_symbol() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch_with_symbol(99, vec![c]));
    assert_eq!(arena.get(parent).symbol(), 99);
    assert_eq!(arena.get(parent).children().len(), 1);
    assert_eq!(arena.get(parent).children()[0], c);
}

// ===========================================================================
// 29. Deep tree
// ===========================================================================

#[test]
fn test_deep_tree_1000_levels() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..1000 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.get(current).symbol(), 999);
    assert_eq!(arena.len(), 1000);
}

// ===========================================================================
// 30. Wide tree
// ===========================================================================

#[test]
fn test_wide_tree_500_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..500).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(children.clone()));
    assert_eq!(arena.get(root).children().len(), 500);
    assert_eq!(arena.len(), 501);
}

// ===========================================================================
// 31. Mixed tree
// ===========================================================================

#[test]
fn test_mixed_tree_leaves_and_branches() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let root = arena.alloc(TreeNode::branch(vec![b1, l3]));
    assert_eq!(arena.len(), 5);
    assert_eq!(arena.get(root).children().len(), 2);
    assert!(arena.get(b1).is_branch());
    assert!(arena.get(l3).is_leaf());
}

// ===========================================================================
// 32. Symbol boundary values
// ===========================================================================

#[test]
fn test_symbol_i32_max() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn test_symbol_i32_min() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

#[test]
fn test_symbol_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn test_symbol_negative_one() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-1));
    assert_eq!(arena.get(h).value(), -1);
}

// ===========================================================================
// 36. Capacity after with_capacity
// ===========================================================================

#[test]
fn test_with_capacity_reports_correct_capacity() {
    let arena = TreeArena::with_capacity(512);
    assert_eq!(arena.capacity(), 512);
}

// ===========================================================================
// 37. Memory usage is non-zero after new
// ===========================================================================

#[test]
fn test_memory_usage_nonzero() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

// ===========================================================================
// 38. Metrics snapshot
// ===========================================================================

#[test]
fn test_metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.num_chunks(), 1);
    assert!(m.capacity() > 0);
    assert!(m.memory_usage() > 0);
}

#[test]
fn test_metrics_after_allocs() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.len(), 50);
    assert!(!m.is_empty());
}

// ===========================================================================
// 40. get_mut
// ===========================================================================

#[test]
fn test_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(42);
    assert_eq!(arena.get(h).value(), 42);
}

// ===========================================================================
// 41–42. Stress: alloc across multiple chunks, verify all
// ===========================================================================

#[test]
fn test_stress_alloc_5000_verify_all() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..5000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn test_stress_alloc_10000_verify_all() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..10_000)
        .map(|i| arena.alloc(TreeNode::leaf(i)))
        .collect();
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

// ===========================================================================
// 43. Small capacity forces many chunks
// ===========================================================================

#[test]
fn test_small_capacity_many_chunks() {
    let mut arena = TreeArena::with_capacity(1);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    assert_eq!(arena.len(), 100);
}

// ===========================================================================
// 44. Reset then refill preserves correctness
// ===========================================================================

#[test]
fn test_reset_then_refill() {
    let mut arena = TreeArena::new();
    for i in 0..500 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    let handles: Vec<NodeHandle> = (0..500)
        .map(|i| arena.alloc(TreeNode::leaf(i + 1000)))
        .collect();
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32 + 1000);
    }
}

// ===========================================================================
// 45. Multiple resets
// ===========================================================================

#[test]
fn test_multiple_resets() {
    let mut arena = TreeArena::new();
    for _ in 0..5 {
        for i in 0..200 {
            arena.alloc(TreeNode::leaf(i));
        }
        assert_eq!(arena.len(), 200);
        arena.reset();
        assert!(arena.is_empty());
    }
}

// ===========================================================================
// 46. Capacity grows after chunk allocation
// ===========================================================================

#[test]
fn test_capacity_grows_with_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    let initial_cap = arena.capacity();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() > initial_cap);
}

// ===========================================================================
// 47. Interleaved leaves and branches
// ===========================================================================

#[test]
fn test_interleaved_leaves_and_branches() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();
    for i in 0..100 {
        if i % 2 == 0 {
            handles.push(arena.alloc(TreeNode::leaf(i)));
        } else {
            let prev = handles[handles.len() - 1];
            handles.push(arena.alloc(TreeNode::branch(vec![prev])));
        }
    }
    assert_eq!(arena.len(), 100);
    for (i, h) in handles.iter().enumerate() {
        if i % 2 == 0 {
            assert!(arena.get(*h).is_leaf());
        } else {
            assert!(arena.get(*h).is_branch());
        }
    }
}

// ===========================================================================
// 48. Empty branch
// ===========================================================================

#[test]
fn test_empty_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
}

// ===========================================================================
// 49. branch_with_symbol default symbol
// ===========================================================================

#[test]
fn test_branch_default_symbol_is_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert_eq!(arena.get(h).symbol(), 0);
}

// ===========================================================================
// 50. Handles are distinct
// ===========================================================================

#[test]
fn test_all_handles_distinct() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..200).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let set: HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(set.len(), 200);
}

// ===========================================================================
// 51. NodeHandle copy semantics
// ===========================================================================

#[test]
fn test_node_handle_copy() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(5));
    let h2 = h1; // Copy
    assert_eq!(arena.get(h1).value(), arena.get(h2).value());
}

// ===========================================================================
// 52. Stress: deep then wide
// ===========================================================================

#[test]
fn test_deep_then_wide_tree() {
    let mut arena = TreeArena::new();
    // Deep chain of 100
    let mut chain = arena.alloc(TreeNode::leaf(0));
    for i in 1..100 {
        chain = arena.alloc(TreeNode::branch_with_symbol(i, vec![chain]));
    }
    // Wide: 100 leaves under one root
    let leaves: Vec<NodeHandle> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(leaves));
    assert_eq!(arena.len(), 201);
    assert!(arena.get(root).is_branch());
    assert_eq!(arena.get(root).children().len(), 100);
}

// ===========================================================================
// 53. Stress: alloc 50_000 nodes
// ===========================================================================

#[test]
fn test_alloc_50000_nodes() {
    let mut arena = TreeArena::new();
    for i in 0..50_000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 50_000);
}

// ===========================================================================
// 54–55. proptest: roundtrip N leaves
// ===========================================================================

proptest! {
    #[test]
    fn pt_roundtrip_n_leaves(count in 1usize..500) {
        let mut arena = TreeArena::new();
        let handles: Vec<NodeHandle> =
            (0..count as i32).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
        prop_assert_eq!(arena.len(), count);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn pt_roundtrip_branch_symbol(sym in any::<i32>(), n_children in 0usize..10) {
        let mut arena = TreeArena::new();
        let children: Vec<NodeHandle> =
            (0..n_children as i32).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
        let parent = arena.alloc(TreeNode::branch_with_symbol(sym, children.clone()));
        prop_assert_eq!(arena.get(parent).symbol(), sym);
        prop_assert_eq!(arena.get(parent).children().len(), n_children);
    }
}

// ===========================================================================
// 56. Capacity 1: forces immediate chunk growth
// ===========================================================================

#[test]
fn test_capacity_1_three_allocs() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h3 = arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.len(), 3);
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.get(h3).value(), 3);
}

// ===========================================================================
// 57. Exponential chunk growth
// ===========================================================================

#[test]
fn test_exponential_chunk_growth() {
    // Start with capacity 4; chunks should grow 4, 8, 16, ...
    let mut arena = TreeArena::with_capacity(4);
    // Fill first chunk (4 nodes)
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 1);
    // One more triggers new chunk
    arena.alloc(TreeNode::leaf(100));
    assert_eq!(arena.num_chunks(), 2);
    // Fill the second chunk (capacity 8, already has 1)
    for i in 0..7 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 2);
    // One more triggers third chunk
    arena.alloc(TreeNode::leaf(200));
    assert_eq!(arena.num_chunks(), 3);
}

// ===========================================================================
// 58. Clear preserves initial chunk capacity
// ===========================================================================

#[test]
fn test_clear_preserves_first_chunk_capacity() {
    let mut arena = TreeArena::with_capacity(512);
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    // After clear, only one chunk retained
    assert_eq!(arena.num_chunks(), 1);
    assert_eq!(arena.capacity(), 512);
}

// ===========================================================================
// 59. Many handles remain valid across chunk boundaries
// ===========================================================================

#[test]
fn test_handles_valid_across_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    // These will be in chunk 2
    let h3 = arena.alloc(TreeNode::leaf(30));
    let h4 = arena.alloc(TreeNode::leaf(40));
    assert!(arena.num_chunks() >= 2);
    assert_eq!(arena.get(h1).value(), 10);
    assert_eq!(arena.get(h2).value(), 20);
    assert_eq!(arena.get(h3).value(), 30);
    assert_eq!(arena.get(h4).value(), 40);
}

// ===========================================================================
// 60. Branch children point to correct nodes
// ===========================================================================

#[test]
fn test_branch_children_point_to_correct_nodes() {
    let mut arena = TreeArena::new();
    let leaves: Vec<NodeHandle> = (0..10)
        .map(|i| arena.alloc(TreeNode::leaf(i * 10)))
        .collect();
    let parent = arena.alloc(TreeNode::branch(leaves.clone()));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    for (i, child_handle) in children.iter().enumerate() {
        assert_eq!(arena.get(*child_handle).value(), i as i32 * 10);
    }
}

// ===========================================================================
// 61. Nested branches two levels
// ===========================================================================

#[test]
fn test_nested_branches_two_levels() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let l4 = arena.alloc(TreeNode::leaf(4));
    let b2 = arena.alloc(TreeNode::branch(vec![l3, l4]));
    let root = arena.alloc(TreeNode::branch(vec![b1, b2]));
    assert_eq!(arena.get(root).children().len(), 2);
    assert!(arena.get(arena.get(root).children()[0]).is_branch());
    assert!(arena.get(arena.get(root).children()[1]).is_branch());
}

// ===========================================================================
// 62. Arena is_empty after creation
// ===========================================================================

#[test]
fn test_new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
}

// ===========================================================================
// 63. Arena not empty after alloc
// ===========================================================================

#[test]
fn test_arena_not_empty_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(!arena.is_empty());
}

// ===========================================================================
// 64. with_capacity capacity matches
// ===========================================================================

#[test]
fn test_with_capacity_exact_capacity() {
    for cap in [1, 2, 16, 128, 1024, 4096] {
        let arena = TreeArena::with_capacity(cap);
        assert_eq!(arena.capacity(), cap);
    }
}

// ===========================================================================
// 65. proptest: capacity allocation roundtrip
// ===========================================================================

proptest! {
    #[test]
    fn pt_with_capacity_correct(cap in 1usize..5000) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert_eq!(arena.capacity(), cap);
        prop_assert!(arena.is_empty());
    }
}

// ===========================================================================
// 66. Multiple arena instances independent
// ===========================================================================

#[test]
fn test_multiple_arenas_independent() {
    let mut a1 = TreeArena::new();
    let mut a2 = TreeArena::new();
    let h1 = a1.alloc(TreeNode::leaf(1));
    let h2 = a2.alloc(TreeNode::leaf(2));
    assert_eq!(a1.get(h1).value(), 1);
    assert_eq!(a2.get(h2).value(), 2);
    assert_eq!(a1.len(), 1);
    assert_eq!(a2.len(), 1);
}

// ===========================================================================
// 67. Clear then many allocs
// ===========================================================================

#[test]
fn test_clear_then_many_allocs() {
    let mut arena = TreeArena::new();
    for i in 0..2000 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let handles: Vec<NodeHandle> = (0..3000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 3000);
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[2999]).value(), 2999);
}

// ===========================================================================
// 68. proptest: clear/refill preserves correctness
// ===========================================================================

proptest! {
    #[test]
    fn pt_clear_refill(count in 1usize..200) {
        let mut arena = TreeArena::new();
        for i in 0..count as i32 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
        let handles: Vec<NodeHandle> =
            (0..count as i32).map(|i| arena.alloc(TreeNode::leaf(i + 100))).collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32 + 100);
        }
    }
}

// ===========================================================================
// 69. Stress: alternating clear and grow
// ===========================================================================

#[test]
fn test_alternating_clear_and_grow() {
    let mut arena = TreeArena::new();
    for round in 0..20 {
        let count = (round + 1) * 50;
        for i in 0..count {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        assert_eq!(arena.len(), count);
        arena.clear();
    }
}

// ===========================================================================
// 70. Branch with single child
// ===========================================================================

#[test]
fn test_branch_single_child() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(42));
    let branch = arena.alloc(TreeNode::branch(vec![leaf]));
    assert_eq!(arena.get(branch).children().len(), 1);
    assert_eq!(arena.get(branch).children()[0], leaf);
}

// ===========================================================================
// 71. get_mut on branch does not affect children
// ===========================================================================

#[test]
fn test_get_mut_branch_children_unchanged() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(5));
    let parent = arena.alloc(TreeNode::branch_with_symbol(1, vec![c]));
    // get_mut - we can't change branch symbol easily, but verify children survive
    let _ = arena.get_mut(parent);
    assert_eq!(arena.get(parent).children().len(), 1);
    assert_eq!(arena.get(c).value(), 5);
}

// ===========================================================================
// 72. Stress: 100 branches each with 10 children
// ===========================================================================

#[test]
fn test_stress_100_branches_10_children_each() {
    let mut arena = TreeArena::new();
    for _ in 0..100 {
        let children: Vec<NodeHandle> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
        let parent = arena.alloc(TreeNode::branch(children));
        assert_eq!(arena.get(parent).children().len(), 10);
    }
    assert_eq!(arena.len(), 1100); // 100 * 11
}

// ===========================================================================
// 73. proptest: branches preserve child count
// ===========================================================================

proptest! {
    #[test]
    fn pt_branch_child_count(n in 0usize..20) {
        let mut arena = TreeArena::new();
        let children: Vec<NodeHandle> =
            (0..n as i32).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
        let parent = arena.alloc(TreeNode::branch(children));
        prop_assert_eq!(arena.get(parent).children().len(), n);
    }
}

// ===========================================================================
// 74. Metrics after clear
// ===========================================================================

#[test]
fn test_metrics_after_clear() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.num_chunks(), 1);
}

// ===========================================================================
// 75. Metrics after reset
// ===========================================================================

#[test]
fn test_metrics_after_reset() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    // Reset retains all chunks
    assert_eq!(m.num_chunks(), chunks_before);
}

// ===========================================================================
// 76. proptest: alloc then len
// ===========================================================================

proptest! {
    #[test]
    fn pt_alloc_len(count in 0usize..500) {
        let mut arena = TreeArena::new();
        for i in 0..count as i32 {
            arena.alloc(TreeNode::leaf(i));
        }
        prop_assert_eq!(arena.len(), count);
    }
}

// ===========================================================================
// 77. Stress: rapid alloc/reset cycles
// ===========================================================================

#[test]
fn test_rapid_alloc_reset_100_cycles() {
    let mut arena = TreeArena::new();
    for _ in 0..100 {
        arena.alloc(TreeNode::leaf(1));
        arena.reset();
    }
    assert!(arena.is_empty());
}

// ===========================================================================
// 78. Verify deref on TreeNodeRef
// ===========================================================================

#[test]
fn test_tree_node_ref_deref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(77));
    let node_ref = arena.get(h);
    // Access through Deref
    assert_eq!(node_ref.symbol(), 77);
    assert!(node_ref.is_leaf());
}

// ===========================================================================
// 79. set_value only affects leaf
// ===========================================================================

#[test]
fn test_set_value_on_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(999);
    assert_eq!(arena.get(h).value(), 999);
}

#[test]
fn test_set_value_on_branch_no_effect() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(50, vec![]));
    arena.get_mut(h).set_value(999);
    // set_value only modifies leaves; branch symbol is unchanged
    assert_eq!(arena.get(h).symbol(), 50);
}

// ===========================================================================
// 81. proptest: many branches roundtrip
// ===========================================================================

proptest! {
    #[test]
    fn pt_many_branches_roundtrip(
        data in prop::collection::vec((any::<i32>(), 0usize..5), 1..50)
    ) {
        let mut arena = TreeArena::new();
        let mut branch_handles = Vec::new();
        for (sym, n_children) in &data {
            let children: Vec<NodeHandle> =
                (0..*n_children as i32).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
            let bh = arena.alloc(TreeNode::branch_with_symbol(*sym, children));
            branch_handles.push((*sym, *n_children, bh));
        }
        for (sym, n_children, bh) in &branch_handles {
            prop_assert_eq!(arena.get(*bh).symbol(), *sym);
            prop_assert_eq!(arena.get(*bh).children().len(), *n_children);
        }
    }
}

// ===========================================================================
// 82. proptest: reset preserves chunk count
// ===========================================================================

proptest! {
    #[test]
    fn pt_reset_preserves_chunks(count in 1usize..2000) {
        let mut arena = TreeArena::new();
        for i in 0..count as i32 {
            arena.alloc(TreeNode::leaf(i));
        }
        let chunks = arena.num_chunks();
        arena.reset();
        prop_assert_eq!(arena.num_chunks(), chunks);
    }
}

// ===========================================================================
// 83. Capacity is always >= len
// ===========================================================================

#[test]
fn test_capacity_ge_len() {
    let mut arena = TreeArena::new();
    for i in 0..5000 {
        arena.alloc(TreeNode::leaf(i));
        assert!(arena.capacity() >= arena.len());
    }
}

// ===========================================================================
// 84. proptest: capacity >= len
// ===========================================================================

proptest! {
    #[test]
    fn pt_capacity_ge_len(count in 0usize..1000) {
        let mut arena = TreeArena::new();
        for i in 0..count as i32 {
            arena.alloc(TreeNode::leaf(i));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }
}

// ===========================================================================
// 85. Stress: 20_000 branches
// ===========================================================================

#[test]
fn test_stress_20000_branches() {
    let mut arena = TreeArena::new();
    for i in 0..20_000 {
        arena.alloc(TreeNode::branch_with_symbol(i, vec![]));
    }
    assert_eq!(arena.len(), 20_000);
}

// ===========================================================================
// 86. NodeHandle Debug format
// ===========================================================================

#[test]
fn test_node_handle_debug() {
    let h = NodeHandle::new(0, 0);
    let dbg = format!("{h:?}");
    assert!(!dbg.is_empty());
}

// ===========================================================================
// 87. Arena Debug format
// ===========================================================================

#[test]
fn test_arena_debug() {
    let arena = TreeArena::new();
    let dbg = format!("{arena:?}");
    assert!(!dbg.is_empty());
}

// ===========================================================================
// 88. Stress: fill, clear, fill larger, clear, fill even larger
// ===========================================================================

#[test]
fn test_escalating_fill_clear_cycles() {
    let mut arena = TreeArena::new();
    for size in [100, 500, 2000, 5000] {
        for i in 0..size {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        assert_eq!(arena.len(), size);
        arena.clear();
        assert!(arena.is_empty());
    }
}

// ===========================================================================
// 89. proptest: NodeHandle eq reflexive
// ===========================================================================

proptest! {
    #[test]
    fn pt_handle_eq_reflexive(chunk in 0u32..100, node in 0u32..100) {
        let h = NodeHandle::new(chunk, node);
        prop_assert_eq!(h, h);
    }
}

// ===========================================================================
// 90. proptest: distinct indices → distinct handles
// ===========================================================================

proptest! {
    #[test]
    fn pt_distinct_handles(
        c1 in 0u32..100, n1 in 0u32..100,
        c2 in 0u32..100, n2 in 0u32..100,
    ) {
        let h1 = NodeHandle::new(c1, n1);
        let h2 = NodeHandle::new(c2, n2);
        if c1 == c2 && n1 == n2 {
            prop_assert_eq!(h1, h2);
        } else {
            prop_assert_ne!(h1, h2);
        }
    }
}

// ===========================================================================
// 91. TreeNode clone
// ===========================================================================

#[test]
fn test_tree_node_clone() {
    let node = TreeNode::leaf(42);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

// ===========================================================================
// 92. TreeNode partial eq
// ===========================================================================

#[test]
fn test_tree_node_partial_eq() {
    let a = TreeNode::leaf(1);
    let b = TreeNode::leaf(1);
    let c = TreeNode::leaf(2);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ===========================================================================
// 93. TreeNode branch eq
// ===========================================================================

#[test]
fn test_tree_node_branch_eq() {
    let h = NodeHandle::new(0, 0);
    let a = TreeNode::branch(vec![h]);
    let b = TreeNode::branch(vec![h]);
    assert_eq!(a, b);
}

// ===========================================================================
// 94. TreeNode debug
// ===========================================================================

#[test]
fn test_tree_node_debug() {
    let node = TreeNode::leaf(5);
    let dbg = format!("{node:?}");
    assert!(!dbg.is_empty());
}

// ===========================================================================
// 95. Memory usage increases with allocs
// ===========================================================================

#[test]
fn test_memory_usage_increases_with_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    let mem_before = arena.memory_usage();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.memory_usage() > mem_before);
}

// ===========================================================================
// 96. with_capacity(0) panics
// ===========================================================================

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn test_with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

// ===========================================================================
// 97. proptest: alloc leaf → is_leaf, not is_branch
// ===========================================================================

proptest! {
    #[test]
    fn pt_leaf_is_leaf_not_branch(sym in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }
}

// ===========================================================================
// 98. proptest: alloc branch → is_branch, not is_leaf
// ===========================================================================

proptest! {
    #[test]
    fn pt_branch_is_branch_not_leaf(sym in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }
}

// ===========================================================================
// 99. Stress: build a balanced binary tree of depth 10
// ===========================================================================

#[test]
fn test_balanced_binary_tree_depth_10() {
    let mut arena = TreeArena::new();
    fn build_tree(arena: &mut TreeArena, depth: usize) -> NodeHandle {
        if depth == 0 {
            arena.alloc(TreeNode::leaf(0))
        } else {
            let left = build_tree(arena, depth - 1);
            let right = build_tree(arena, depth - 1);
            arena.alloc(TreeNode::branch(vec![left, right]))
        }
    }
    let root = build_tree(&mut arena, 10);
    // 2^11 - 1 = 2047 nodes
    assert_eq!(arena.len(), 2047);
    assert!(arena.get(root).is_branch());
}

// ===========================================================================
// 100. Stress: verify handles in HashSet across chunk boundaries
// ===========================================================================

#[test]
fn test_handles_hashset_across_chunks() {
    let mut arena = TreeArena::with_capacity(8);
    let mut set = HashSet::new();
    for i in 0..500 {
        let h = arena.alloc(TreeNode::leaf(i));
        assert!(set.insert(h));
    }
    assert_eq!(set.len(), 500);
    assert!(arena.num_chunks() > 1);
}
