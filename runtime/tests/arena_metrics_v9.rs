//! Comprehensive tests for TreeArena metrics and capacity (v9)
//!
//! Covers: construction, allocation counts, chunk growth, clear/reset,
//! with_capacity variants, monotonicity invariants, metrics snapshots,
//! and capacity/memory_usage accessors.

use adze::arena_allocator::{TreeArena, TreeNode};

/// Mirror of the crate-private `DEFAULT_CHUNK_SIZE` constant.
const DEFAULT_CHUNK_SIZE: usize = 1024;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn alloc_n(arena: &mut TreeArena, n: usize) {
    for i in 0..n {
        arena.alloc(TreeNode::leaf(i as i32));
    }
}

// ===========================================================================
// 1–6  Default / new arena properties
// ===========================================================================

#[test]
fn test_default_arena_len_is_zero() {
    let arena = TreeArena::default();
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_default_arena_is_empty() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
}

#[test]
fn test_default_arena_num_chunks_is_one() {
    let arena = TreeArena::default();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_new_arena_len_is_zero() {
    let arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
}

#[test]
fn test_new_arena_num_chunks_is_one() {
    let arena = TreeArena::new();
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 7–9  Single allocation
// ===========================================================================

#[test]
fn test_one_alloc_len_is_one() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_one_alloc_not_empty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(!arena.is_empty());
}

#[test]
fn test_one_alloc_still_one_chunk() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 10–12  100 allocations
// ===========================================================================

#[test]
fn test_hundred_allocs_len() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    assert_eq!(arena.len(), 100);
}

#[test]
fn test_hundred_allocs_not_empty() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    assert!(!arena.is_empty());
}

#[test]
fn test_hundred_allocs_single_chunk() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 13–15  Exactly DEFAULT_CHUNK_SIZE allocs → 1 chunk
// ===========================================================================

#[test]
fn test_full_first_chunk_len() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.len(), DEFAULT_CHUNK_SIZE);
}

#[test]
fn test_full_first_chunk_one_chunk() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_full_first_chunk_not_empty() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE);
    assert!(!arena.is_empty());
}

// ===========================================================================
// 16–18  DEFAULT_CHUNK_SIZE + 1 allocs → 2 chunks
// ===========================================================================

#[test]
fn test_overflow_first_chunk_two_chunks() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_overflow_first_chunk_len() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.len(), DEFAULT_CHUNK_SIZE + 1);
}

#[test]
fn test_overflow_first_chunk_not_empty() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert!(!arena.is_empty());
}

// ===========================================================================
// 19–20  2 × DEFAULT_CHUNK_SIZE allocs → still 2 chunks
//        (second chunk has capacity 2 × DEFAULT_CHUNK_SIZE)
// ===========================================================================

#[test]
fn test_double_chunk_size_still_two_chunks() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 2 * DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_double_chunk_size_len() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 2 * DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.len(), 2 * DEFAULT_CHUNK_SIZE);
}

// ===========================================================================
// 21–22  3 × DEFAULT_CHUNK_SIZE (1024 + 2048) → 2 chunks; +1 → 3 chunks
// ===========================================================================

#[test]
fn test_three_times_chunk_still_two_chunks() {
    let mut arena = TreeArena::new();
    // chunk0 capacity 1024, chunk1 capacity 2048 → total 3072
    alloc_n(&mut arena, 3 * DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_three_times_chunk_plus_one_triggers_third() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 3 * DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.num_chunks(), 3);
}

// ===========================================================================
// 23  with_capacity(0) → panics
// ===========================================================================

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn test_with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

// ===========================================================================
// 24–28  with_capacity(512)
// ===========================================================================

#[test]
fn test_with_capacity_512_is_empty() {
    let arena = TreeArena::with_capacity(512);
    assert!(arena.is_empty());
}

#[test]
fn test_with_capacity_512_len_zero() {
    let arena = TreeArena::with_capacity(512);
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_with_capacity_512_one_chunk() {
    let arena = TreeArena::with_capacity(512);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_with_capacity_512_alloc_works() {
    let mut arena = TreeArena::with_capacity(512);
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_with_capacity_512_chunk_boundary() {
    let mut arena = TreeArena::with_capacity(512);
    alloc_n(&mut arena, 512);
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.num_chunks(), 2);
}

// ===========================================================================
// 29–31  with_capacity(2048)
// ===========================================================================

#[test]
fn test_with_capacity_2048_is_empty() {
    let arena = TreeArena::with_capacity(2048);
    assert!(arena.is_empty());
}

#[test]
fn test_with_capacity_2048_fills_one_chunk() {
    let mut arena = TreeArena::with_capacity(2048);
    alloc_n(&mut arena, 2048);
    assert_eq!(arena.len(), 2048);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_with_capacity_2048_overflow() {
    let mut arena = TreeArena::with_capacity(2048);
    alloc_n(&mut arena, 2049);
    assert_eq!(arena.num_chunks(), 2);
}

// ===========================================================================
// 32–34  with_capacity(1)
// ===========================================================================

#[test]
fn test_with_capacity_1_works() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_with_capacity_1_second_alloc_new_chunk() {
    let mut arena = TreeArena::with_capacity(1);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 2);
    assert_eq!(arena.len(), 2);
}

#[test]
fn test_with_capacity_1_many_allocs() {
    let mut arena = TreeArena::with_capacity(1);
    alloc_n(&mut arena, 50);
    assert_eq!(arena.len(), 50);
}

// ===========================================================================
// 35–38  clear() basics
// ===========================================================================

#[test]
fn test_clear_resets_len_to_zero() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 50);
    arena.clear();
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_clear_makes_arena_empty() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 50);
    arena.clear();
    assert!(arena.is_empty());
}

#[test]
fn test_clear_on_empty_arena() {
    let mut arena = TreeArena::new();
    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn test_clear_on_single_element() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

// ===========================================================================
// 39–41  clear() reduces chunks to 1
// ===========================================================================

#[test]
fn test_clear_reduces_two_chunks_to_one() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.num_chunks(), 2);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_clear_reduces_many_chunks_to_one() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 3 * DEFAULT_CHUNK_SIZE + 1);
    assert!(arena.num_chunks() >= 3);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_clear_single_chunk_stays_one() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 500);
    assert_eq!(arena.num_chunks(), 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 42–44  Alloc after clear
// ===========================================================================

#[test]
fn test_alloc_after_clear_works() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 10);
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_alloc_many_after_clear() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 100);
    arena.clear();
    alloc_n(&mut arena, 500);
    assert_eq!(arena.len(), 500);
}

#[test]
fn test_alloc_after_clear_values_correct() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    arena.clear();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    assert_eq!(arena.get(h1).value(), 10);
    assert_eq!(arena.get(h2).value(), 20);
}

// ===========================================================================
// 45–47  Multiple clear cycles
// ===========================================================================

#[test]
fn test_multiple_clear_cycles_len() {
    let mut arena = TreeArena::new();
    for cycle in 0..5 {
        alloc_n(&mut arena, 200);
        assert_eq!(arena.len(), 200, "cycle {cycle} len before clear");
        arena.clear();
        assert_eq!(arena.len(), 0, "cycle {cycle} len after clear");
    }
}

#[test]
fn test_multiple_clear_cycles_is_empty() {
    let mut arena = TreeArena::new();
    for _ in 0..5 {
        alloc_n(&mut arena, 50);
        assert!(!arena.is_empty());
        arena.clear();
        assert!(arena.is_empty());
    }
}

#[test]
fn test_clear_cycle_with_growing_sizes() {
    let mut arena = TreeArena::new();
    for size in [10, 100, 1000, 2000, 100] {
        alloc_n(&mut arena, size);
        assert_eq!(arena.len(), size);
        arena.clear();
        assert_eq!(arena.len(), 0);
    }
}

// ===========================================================================
// 48–51  reset() behaviour (keeps chunks, clears data)
// ===========================================================================

#[test]
fn test_reset_len_zero() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    arena.reset();
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_reset_is_empty() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn test_reset_preserves_chunks() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn test_alloc_after_reset_works() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(77));
    assert_eq!(arena.get(h).value(), 77);
    assert_eq!(arena.len(), 1);
}

// ===========================================================================
// 52–54  len grows monotonically (without clear/reset)
// ===========================================================================

#[test]
fn test_len_strictly_increases() {
    let mut arena = TreeArena::new();
    let mut prev = 0;
    for i in 0..500 {
        arena.alloc(TreeNode::leaf(i));
        let cur = arena.len();
        assert!(cur > prev, "len must strictly grow: {prev} -> {cur}");
        prev = cur;
    }
}

#[test]
fn test_len_increments_by_one() {
    let mut arena = TreeArena::new();
    for i in 0..200 {
        assert_eq!(arena.len(), i);
        arena.alloc(TreeNode::leaf(i as i32));
    }
    assert_eq!(arena.len(), 200);
}

#[test]
fn test_len_across_chunk_boundary() {
    let mut arena = TreeArena::new();
    for i in 0..(DEFAULT_CHUNK_SIZE + 10) {
        assert_eq!(arena.len(), i);
        arena.alloc(TreeNode::leaf(0));
    }
}

// ===========================================================================
// 55–57  num_chunks grows monotonically (without clear/reset)
// ===========================================================================

#[test]
fn test_num_chunks_nondecreasing() {
    let mut arena = TreeArena::new();
    let mut prev = arena.num_chunks();
    for i in 0..5000 {
        arena.alloc(TreeNode::leaf(i));
        let cur = arena.num_chunks();
        assert!(cur >= prev, "chunks must not decrease: {prev} -> {cur}");
        prev = cur;
    }
}

#[test]
fn test_num_chunks_steps_at_boundary() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_num_chunks_second_boundary() {
    let mut arena = TreeArena::new();
    // chunk0: 1024, chunk1: 2048 → fills at 3072
    alloc_n(&mut arena, 3 * DEFAULT_CHUNK_SIZE);
    assert_eq!(arena.num_chunks(), 2);
    arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.num_chunks(), 3);
}

// ===========================================================================
// 58–61  is_empty ↔ len == 0
// ===========================================================================

#[test]
fn test_empty_arena_consistency() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_nonempty_arena_consistency() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(!arena.is_empty());
    assert_ne!(arena.len(), 0);
}

#[test]
fn test_cleared_arena_consistency() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_is_empty_matches_len_through_allocs() {
    let mut arena = TreeArena::new();
    assert!(arena.is_empty());
    for i in 1..=50 {
        arena.alloc(TreeNode::leaf(i));
        assert!(!arena.is_empty());
        assert_eq!(arena.len(), i as usize);
    }
}

// ===========================================================================
// 62–65  Capacity doesn't affect correctness
// ===========================================================================

#[test]
fn test_various_capacities_same_len() {
    for cap in [1, 2, 10, 100, 512, 1024, 2048] {
        let mut arena = TreeArena::with_capacity(cap);
        alloc_n(&mut arena, 50);
        assert_eq!(arena.len(), 50, "capacity {cap}");
    }
}

#[test]
fn test_various_capacities_values_preserved() {
    for cap in [1, 64, 1024] {
        let mut arena = TreeArena::with_capacity(cap);
        let h1 = arena.alloc(TreeNode::leaf(11));
        let h2 = arena.alloc(TreeNode::leaf(22));
        assert_eq!(arena.get(h1).value(), 11, "capacity {cap}");
        assert_eq!(arena.get(h2).value(), 22, "capacity {cap}");
    }
}

#[test]
fn test_large_capacity_single_chunk() {
    let mut arena = TreeArena::with_capacity(10_000);
    alloc_n(&mut arena, 9_999);
    assert_eq!(arena.num_chunks(), 1);
    assert_eq!(arena.len(), 9_999);
}

#[test]
fn test_small_capacity_many_chunks() {
    let mut arena = TreeArena::with_capacity(1);
    alloc_n(&mut arena, 100);
    assert_eq!(arena.len(), 100);
    assert!(arena.num_chunks() > 1);
}

// ===========================================================================
// 66–70  metrics() snapshot
// ===========================================================================

#[test]
fn test_metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.num_chunks(), 1);
}

#[test]
fn test_metrics_after_allocs() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 10);
    let m = arena.metrics();
    assert_eq!(m.len(), 10);
    assert!(!m.is_empty());
}

#[test]
fn test_metrics_len_matches_arena_len() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 200);
    let m = arena.metrics();
    assert_eq!(m.len(), arena.len());
}

#[test]
fn test_metrics_num_chunks_matches() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    let m = arena.metrics();
    assert_eq!(m.num_chunks(), arena.num_chunks());
}

#[test]
fn test_metrics_after_clear() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 500);
    arena.clear();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.num_chunks(), 1);
}

// ===========================================================================
// 71–73  capacity() accessor
// ===========================================================================

#[test]
fn test_capacity_default_arena() {
    let arena = TreeArena::new();
    assert_eq!(arena.capacity(), DEFAULT_CHUNK_SIZE);
}

#[test]
fn test_capacity_with_custom() {
    let arena = TreeArena::with_capacity(512);
    assert_eq!(arena.capacity(), 512);
}

#[test]
fn test_capacity_grows_after_chunk_overflow() {
    let mut arena = TreeArena::new();
    let cap_before = arena.capacity();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert!(arena.capacity() > cap_before);
}

// ===========================================================================
// 74–76  memory_usage()
// ===========================================================================

#[test]
fn test_memory_usage_positive_for_new_arena() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn test_memory_usage_grows_with_chunks() {
    let mut arena = TreeArena::new();
    let usage_before = arena.memory_usage();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert!(arena.memory_usage() > usage_before);
}

#[test]
fn test_memory_usage_metrics_matches() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    assert_eq!(arena.memory_usage(), arena.metrics().memory_usage());
}

// ===========================================================================
// 77–79  get() correctness across chunk boundaries
// ===========================================================================

#[test]
fn test_get_first_and_last_in_chunk() {
    let mut arena = TreeArena::new();
    let first = arena.alloc(TreeNode::leaf(1));
    for i in 2..DEFAULT_CHUNK_SIZE as i32 {
        arena.alloc(TreeNode::leaf(i));
    }
    let last = arena.alloc(TreeNode::leaf(DEFAULT_CHUNK_SIZE as i32));
    assert_eq!(arena.get(first).value(), 1);
    assert_eq!(arena.get(last).value(), DEFAULT_CHUNK_SIZE as i32);
}

#[test]
fn test_get_across_chunk_boundary() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();
    for i in 0..(DEFAULT_CHUNK_SIZE + 5) as i32 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    // Verify first, boundary, and post-boundary handles
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(
        arena.get(handles[DEFAULT_CHUNK_SIZE - 1]).value(),
        (DEFAULT_CHUNK_SIZE - 1) as i32
    );
    assert_eq!(
        arena.get(handles[DEFAULT_CHUNK_SIZE]).value(),
        DEFAULT_CHUNK_SIZE as i32
    );
    assert_eq!(
        arena.get(handles[DEFAULT_CHUNK_SIZE + 4]).value(),
        (DEFAULT_CHUNK_SIZE + 4) as i32
    );
}

#[test]
fn test_get_after_many_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    let mut handles = Vec::new();
    for i in 0..50 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

// ===========================================================================
// 80–82  Branch nodes and mixed types
// ===========================================================================

#[test]
fn test_branch_node_len_counts() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::branch(vec![c1, c2]));
    assert_eq!(arena.len(), 3);
}

#[test]
fn test_mixed_leaf_branch_get() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(42));
    let branch = arena.alloc(TreeNode::branch(vec![leaf]));
    assert!(arena.get(leaf).is_leaf());
    assert!(arena.get(branch).is_branch());
}

#[test]
fn test_branch_children_accessible() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(7));
    let parent = arena.alloc(TreeNode::branch(vec![c]));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 1);
    let child_handle = children[0];
    let child_value = arena.get(child_handle).value();
    assert_eq!(child_value, 7);
}

// ===========================================================================
// 83–85  Reset vs clear difference
// ===========================================================================

#[test]
fn test_reset_keeps_all_chunks() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.num_chunks(), 2);
    arena.reset();
    assert_eq!(arena.num_chunks(), 2);
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_clear_drops_extra_chunks() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 1);
    assert_eq!(arena.num_chunks(), 2);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_reset_then_realloc_works() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 10);
    arena.reset();
    alloc_n(&mut arena, DEFAULT_CHUNK_SIZE + 10);
    assert_eq!(arena.len(), DEFAULT_CHUNK_SIZE + 10);
}

// ===========================================================================
// 86  Capacity metrics snapshot
// ===========================================================================

#[test]
fn test_metrics_capacity_matches_arena() {
    let mut arena = TreeArena::with_capacity(256);
    alloc_n(&mut arena, 300);
    let m = arena.metrics();
    assert_eq!(m.capacity(), arena.capacity());
}

// ===========================================================================
// 87  Double-clear is idempotent
// ===========================================================================

#[test]
fn test_double_clear_idempotent() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    arena.clear();
    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ===========================================================================
// 88  Double-reset is idempotent
// ===========================================================================

#[test]
fn test_double_reset_idempotent() {
    let mut arena = TreeArena::new();
    alloc_n(&mut arena, 100);
    arena.reset();
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

// ===========================================================================
// 89  Capacity at least as large as len
// ===========================================================================

#[test]
fn test_capacity_gte_len() {
    let mut arena = TreeArena::new();
    for i in 0..2000 {
        arena.alloc(TreeNode::leaf(i));
        assert!(
            arena.capacity() >= arena.len(),
            "capacity {} < len {}",
            arena.capacity(),
            arena.len()
        );
    }
}

// ===========================================================================
// 90  metrics().is_empty matches arena.is_empty
// ===========================================================================

#[test]
fn test_metrics_is_empty_matches() {
    let mut arena = TreeArena::new();
    assert_eq!(arena.metrics().is_empty(), arena.is_empty());
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.metrics().is_empty(), arena.is_empty());
    arena.clear();
    assert_eq!(arena.metrics().is_empty(), arena.is_empty());
}
