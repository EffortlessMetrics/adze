//! Comprehensive stress tests for TreeArena and related types.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use std::collections::HashSet;

// ============================================================
// 1. Arena construction and initial state (8 tests)
// ============================================================

#[test]
fn test_new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
}

#[test]
fn test_new_arena_len_zero() {
    let arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_new_arena_has_one_chunk() {
    let arena = TreeArena::new();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_new_arena_default_capacity() {
    let arena = TreeArena::new();
    assert_eq!(arena.capacity(), 1024);
}

#[test]
fn test_with_capacity_sets_capacity() {
    let arena = TreeArena::with_capacity(16);
    assert_eq!(arena.capacity(), 16);
}

#[test]
fn test_with_capacity_one() {
    let arena = TreeArena::with_capacity(1);
    assert_eq!(arena.capacity(), 1);
    assert!(arena.is_empty());
}

#[test]
fn test_default_is_same_as_new() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

#[test]
fn test_new_arena_memory_usage_positive() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

// ============================================================
// 2. Allocation and retrieval (10 tests)
// ============================================================

#[test]
fn test_alloc_leaf_returns_handle() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).value(), 1);
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
fn test_alloc_leaf_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(99));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn test_alloc_branch_is_branch() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert!(arena.get(h).is_branch());
    assert!(!arena.get(h).is_leaf());
}

#[test]
fn test_alloc_branch_children() {
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
fn test_alloc_branch_with_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(42, vec![]));
    assert_eq!(arena.get(h).symbol(), 42);
}

#[test]
fn test_alloc_branch_no_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_retrieve_preserves_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-100));
    assert_eq!(arena.get(h).value(), -100);
}

#[test]
fn test_retrieve_via_deref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    // TreeNodeRef implements Deref<Target=TreeNode>
    let node_ref = arena.get(h);
    assert_eq!(node_ref.symbol(), 7);
    assert!(node_ref.is_leaf());
}

#[test]
fn test_alloc_not_empty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(0));
    assert!(!arena.is_empty());
}

// ============================================================
// 3. Multiple allocations and handle uniqueness (8 tests)
// ============================================================

#[test]
fn test_handles_are_unique() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn test_many_handles_unique() {
    let mut arena = TreeArena::with_capacity(4);
    let handles: Vec<NodeHandle> = (0..20).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let set: HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(set.len(), 20);
}

#[test]
fn test_interleaved_leaf_branch_retrieval() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::branch(vec![h1]));
    let h3 = arena.alloc(TreeNode::leaf(30));
    assert_eq!(arena.get(h1).value(), 10);
    assert!(arena.get(h2).is_branch());
    assert_eq!(arena.get(h3).value(), 30);
}

#[test]
fn test_all_values_retrievable() {
    let mut arena = TreeArena::with_capacity(8);
    let handles: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn test_handles_stable_across_growth() {
    let mut arena = TreeArena::with_capacity(2);
    let h1 = arena.alloc(TreeNode::leaf(100));
    let h2 = arena.alloc(TreeNode::leaf(200));
    // This triggers a new chunk
    let h3 = arena.alloc(TreeNode::leaf(300));
    assert_eq!(arena.get(h1).value(), 100);
    assert_eq!(arena.get(h2).value(), 200);
    assert_eq!(arena.get(h3).value(), 300);
}

#[test]
fn test_first_and_last_handle_valid() {
    let mut arena = TreeArena::with_capacity(4);
    let first = arena.alloc(TreeNode::leaf(0));
    for i in 1..99 {
        arena.alloc(TreeNode::leaf(i));
    }
    let last = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(first).value(), 0);
    assert_eq!(arena.get(last).value(), 99);
}

#[test]
fn test_len_matches_allocation_count() {
    let mut arena = TreeArena::with_capacity(3);
    for i in 0..25 {
        arena.alloc(TreeNode::leaf(i));
        assert_eq!(arena.len(), (i + 1) as usize);
    }
}

#[test]
fn test_capacity_grows_with_allocations() {
    let mut arena = TreeArena::with_capacity(2);
    let initial_cap = arena.capacity();
    // Fill first chunk and trigger growth
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3));
    assert!(arena.capacity() > initial_cap);
}

// ============================================================
// 4. Mutation through get_mut (5 tests)
// ============================================================

#[test]
fn test_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(99);
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn test_get_mut_does_not_affect_others() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    arena.get_mut(h1).set_value(999);
    assert_eq!(arena.get(h1).value(), 999);
    assert_eq!(arena.get(h2).value(), 20);
}

#[test]
fn test_get_mut_multiple_times() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    for v in 1..=10 {
        arena.get_mut(h).set_value(v);
    }
    assert_eq!(arena.get(h).value(), 10);
}

#[test]
fn test_get_mut_deref_access() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    // TreeNodeRefMut implements Deref<Target=TreeNode>
    let node_ref = arena.get_mut(h);
    assert_eq!(node_ref.symbol(), 5);
}

#[test]
fn test_get_mut_deref_mut_access() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    {
        let mut node_ref = arena.get_mut(h);
        // DerefMut allows direct access to TreeNode
        node_ref.set_value(77);
    }
    assert_eq!(arena.get(h).value(), 77);
}

// ============================================================
// 5. Reset and clear behavior (8 tests)
// ============================================================

#[test]
fn test_reset_empties_arena() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_reset_preserves_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn test_reset_allows_reallocation() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_clear_empties_arena() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_clear_reduces_to_one_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_clear_allows_reallocation() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(77));
    assert_eq!(arena.get(h).value(), 77);
}

#[test]
fn test_double_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn test_reset_then_clear() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ============================================================
// 6. Growth patterns under stress (8 tests)
// ============================================================

#[test]
fn test_growth_adds_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
}

#[test]
fn test_exponential_chunk_growth() {
    // Start with cap=2. After filling, next chunk should be 4 (2*2).
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.capacity(), 2);
    // Trigger new chunk
    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.capacity(), 2 + 4); // original 2 + doubled 4
}

#[test]
fn test_capacity_never_shrinks_on_alloc() {
    let mut arena = TreeArena::with_capacity(4);
    let mut prev_cap = arena.capacity();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
        let cap = arena.capacity();
        assert!(cap >= prev_cap);
        prev_cap = cap;
    }
}

#[test]
fn test_stress_1000_allocations() {
    let mut arena = TreeArena::with_capacity(8);
    let handles: Vec<NodeHandle> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 1000);
    // Spot-check
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[500]).value(), 500);
    assert_eq!(arena.get(handles[999]).value(), 999);
}

#[test]
fn test_stress_alloc_reset_realloc() {
    let mut arena = TreeArena::with_capacity(4);
    for cycle in 0..5 {
        for i in 0..100 {
            arena.alloc(TreeNode::leaf(cycle * 100 + i));
        }
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn test_memory_usage_scales_with_capacity() {
    let small = TreeArena::with_capacity(16);
    let large = TreeArena::with_capacity(1024);
    assert!(large.memory_usage() > small.memory_usage());
}

#[test]
fn test_many_branches_deep_tree() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..100 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.len(), 100);
    assert_eq!(arena.get(current).symbol(), 99);
    assert_eq!(arena.get(current).children().len(), 1);
}

#[test]
fn test_wide_branch_node() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(children.clone()));
    assert_eq!(arena.get(root).children().len(), 50);
    for (i, &c) in arena.get(root).children().iter().enumerate() {
        assert_eq!(arena.get(c).value(), i as i32);
    }
}

// ============================================================
// 7. NodeHandle trait properties (5 tests)
// ============================================================

#[test]
fn test_node_handle_copy() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let h_copy = h; // Copy
    assert_eq!(arena.get(h).value(), 1);
    assert_eq!(arena.get(h_copy).value(), 1);
}

#[test]
fn test_node_handle_clone() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    #[allow(clippy::clone_on_copy)]
    let h_clone = h.clone();
    assert_eq!(h, h_clone);
}

#[test]
fn test_node_handle_eq() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    let h3 = NodeHandle::new(0, 1);
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[test]
fn test_node_handle_hash_consistent() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let h1 = NodeHandle::new(1, 2);
    let h2 = NodeHandle::new(1, 2);

    let hash = |h: &NodeHandle| {
        let mut hasher = DefaultHasher::new();
        h.hash(&mut hasher);
        hasher.finish()
    };

    assert_eq!(hash(&h1), hash(&h2));
}

#[test]
fn test_node_handle_debug() {
    let h = NodeHandle::new(3, 7);
    let debug_str = format!("{:?}", h);
    assert!(debug_str.contains("NodeHandle"));
}

// ============================================================
// 8. TreeNode construction and properties (5 tests)
// ============================================================

#[test]
fn test_tree_node_leaf_value() {
    let node = TreeNode::leaf(42);
    assert_eq!(node.value(), 42);
    assert_eq!(node.symbol(), 42);
}

#[test]
fn test_tree_node_leaf_has_no_children() {
    let node = TreeNode::leaf(1);
    assert!(node.children().is_empty());
}

#[test]
fn test_tree_node_branch_default_symbol() {
    let node = TreeNode::branch(vec![]);
    assert_eq!(node.symbol(), 0);
}

#[test]
fn test_tree_node_branch_with_symbol_value() {
    let node = TreeNode::branch_with_symbol(55, vec![]);
    assert_eq!(node.symbol(), 55);
    assert_eq!(node.value(), 55);
}

#[test]
fn test_tree_node_clone_eq() {
    let node = TreeNode::leaf(123);
    #[allow(clippy::clone_on_copy)]
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

// ============================================================
// 9. Edge cases (3+ tests)
// ============================================================

#[test]
fn test_alloc_negative_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

#[test]
fn test_alloc_zero_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn test_stress_5000_allocs_all_valid() {
    let mut arena = TreeArena::with_capacity(16);
    let handles: Vec<NodeHandle> = (0..5000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 5000);
    // Verify first, middle, and last
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[2500]).value(), 2500);
    assert_eq!(arena.get(handles[4999]).value(), 4999);
}

// ============================================================
// 10. Metrics (bonus — 3 tests)
// ============================================================

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
fn test_metrics_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let m = arena.metrics();
    assert_eq!(m.len(), 1);
    assert!(!m.is_empty());
}

#[test]
fn test_metrics_after_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
}
