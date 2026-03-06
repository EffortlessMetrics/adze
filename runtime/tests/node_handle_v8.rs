//! Comprehensive tests for NodeHandle and TreeNode operations (v8)
//!
//! 80+ tests covering allocation, handle semantics, node kinds,
//! parent-child relationships, arena lifecycle, and edge cases.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ============================================================================
// 1. Alloc returns a NodeHandle
// ============================================================================

#[test]
fn alloc_leaf_returns_handle() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(handle).value(), 1);
}

#[test]
fn alloc_branch_returns_handle() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(10));
    let handle = arena.alloc(TreeNode::branch(vec![child]));
    assert!(arena.get(handle).is_branch());
}

#[test]
fn alloc_branch_with_symbol_returns_handle() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::branch_with_symbol(77, vec![]));
    assert_eq!(arena.get(handle).symbol(), 77);
}

// ============================================================================
// 2. NodeHandle is Copy
// ============================================================================

#[test]
fn node_handle_is_copy() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(42));
    let h2 = h1; // Copy
    assert_eq!(arena.get(h1).value(), 42);
    assert_eq!(arena.get(h2).value(), 42);
}

#[test]
fn node_handle_copy_after_use() {
    let mut arena = TreeArena::new();
    let original = arena.alloc(TreeNode::leaf(99));
    // Use original, then copy — both still valid.
    assert_eq!(arena.get(original).value(), 99);
    let copied = original;
    assert_eq!(arena.get(copied).value(), 99);
}

#[test]
fn node_handle_equality() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let copy = h;
    assert_eq!(h, copy);
}

#[test]
fn node_handle_inequality() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

// ============================================================================
// 3. get(handle) returns correct TreeNode
// ============================================================================

#[test]
fn get_returns_correct_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(55));
    let node_ref = arena.get(h);
    assert!(node_ref.is_leaf());
    assert_eq!(node_ref.value(), 55);
}

#[test]
fn get_returns_correct_branch() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    let node_ref = arena.get(h);
    assert!(node_ref.is_branch());
    assert_eq!(node_ref.children().len(), 1);
}

#[test]
fn get_deref_provides_tree_node_methods() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    // TreeNodeRef derefs to TreeNode
    let node_ref = arena.get(h);
    assert_eq!(node_ref.symbol(), 7);
    assert!(node_ref.is_leaf());
}

// ============================================================================
// 4–6. TreeNode symbol/value matches input
// ============================================================================

#[test]
fn leaf_symbol_matches_input() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(123));
    assert_eq!(arena.get(h).symbol(), 123);
}

#[test]
fn leaf_value_matches_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), arena.get(h).symbol());
}

#[test]
fn branch_default_symbol_is_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert_eq!(arena.get(h).symbol(), 0);
}

#[test]
fn branch_with_symbol_matches_input() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(500, vec![]));
    assert_eq!(arena.get(h).symbol(), 500);
}

#[test]
fn negative_symbol_preserved() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-1));
    assert_eq!(arena.get(h).symbol(), -1);
}

#[test]
fn zero_symbol_preserved() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).symbol(), 0);
}

#[test]
fn max_i32_symbol_preserved() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).symbol(), i32::MAX);
}

#[test]
fn min_i32_symbol_preserved() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).symbol(), i32::MIN);
}

// ============================================================================
// 7–9. Children and leaf/branch distinctions
// ============================================================================

#[test]
fn leaf_has_zero_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn leaf_is_leaf_not_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn branch_is_branch_not_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(!arena.get(h).is_leaf());
}

#[test]
fn branch_with_no_children_is_still_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn branch_children_count_matches() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let c3 = arena.alloc(TreeNode::leaf(3));
    let h = arena.alloc(TreeNode::branch(vec![c1, c2, c3]));
    assert_eq!(arena.get(h).children().len(), 3);
}

// ============================================================================
// 10. Multiple allocs → different handles
// ============================================================================

#[test]
fn sequential_allocs_produce_distinct_handles() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h3 = arena.alloc(TreeNode::leaf(3));
    assert_ne!(h1, h2);
    assert_ne!(h2, h3);
    assert_ne!(h1, h3);
}

#[test]
fn hundred_allocs_all_distinct() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();

    for (i, &hi) in handles.iter().enumerate() {
        for &hj in &handles[i + 1..] {
            assert_ne!(hi, hj);
        }
    }
}

// ============================================================================
// 11. Handle stability: alloc many, get all correct
// ============================================================================

#[test]
fn handle_stability_across_1000_allocs() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();

    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn handle_stability_across_chunk_boundaries() {
    let mut arena = TreeArena::with_capacity(4);
    let handles: Vec<NodeHandle> = (0..20)
        .map(|i| arena.alloc(TreeNode::leaf(i * 10)))
        .collect();
    assert!(arena.num_chunks() > 1);

    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), (i as i32) * 10);
    }
}

#[test]
fn early_handles_valid_after_many_later_allocs() {
    let mut arena = TreeArena::new();
    let first = arena.alloc(TreeNode::leaf(999));

    for i in 0..5000 {
        arena.alloc(TreeNode::leaf(i));
    }

    assert_eq!(arena.get(first).value(), 999);
}

// ============================================================================
// 12. Parent-child relationship setup
// ============================================================================

#[test]
fn branch_children_resolve_to_correct_leaves() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));

    let (ch0, ch1) = {
        let r = arena.get(parent);
        (r.children()[0], r.children()[1])
    };
    assert_eq!(arena.get(ch0).value(), 10);
    assert_eq!(arena.get(ch1).value(), 20);
}

#[test]
fn nested_branches() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(1));
    let inner = arena.alloc(TreeNode::branch(vec![leaf]));
    let outer = arena.alloc(TreeNode::branch(vec![inner]));

    let inner_h = {
        let r = arena.get(outer);
        assert_eq!(r.children().len(), 1);
        r.children()[0]
    };

    let leaf_h = {
        let r = arena.get(inner_h);
        assert_eq!(r.children().len(), 1);
        r.children()[0]
    };

    assert_eq!(arena.get(leaf_h).value(), 1);
}

#[test]
fn deep_nesting_five_levels() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(42));
    for _ in 0..5 {
        current = arena.alloc(TreeNode::branch(vec![current]));
    }

    // Walk down 5 levels to the leaf
    let mut node = current;
    for _ in 0..5 {
        let r = arena.get(node);
        assert_eq!(r.children().len(), 1);
        node = r.children()[0];
    }
    assert_eq!(arena.get(node).value(), 42);
    assert!(arena.get(node).is_leaf());
}

// ============================================================================
// 13. Child list management
// ============================================================================

#[test]
fn children_slice_is_ordered() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let c3 = arena.alloc(TreeNode::leaf(30));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2, c3]));

    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children[0], c1);
    assert_eq!(children[1], c2);
    assert_eq!(children[2], c3);
}

#[test]
fn shared_child_in_multiple_parents() {
    let mut arena = TreeArena::new();
    let shared = arena.alloc(TreeNode::leaf(99));
    let p1 = arena.alloc(TreeNode::branch(vec![shared]));
    let p2 = arena.alloc(TreeNode::branch(vec![shared]));

    assert_eq!(arena.get(p1).children()[0], shared);
    assert_eq!(arena.get(p2).children()[0], shared);
    assert_eq!(arena.get(shared).value(), 99);
}

// ============================================================================
// 14. TreeNode with 0 children → leaf
// ============================================================================

#[test]
fn leaf_node_is_identified_as_leaf() {
    let node = TreeNode::leaf(5);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
}

#[test]
fn leaf_children_returns_empty_slice() {
    let node = TreeNode::leaf(5);
    assert!(node.children().is_empty());
}

// ============================================================================
// 15–16. Branch with 4 children and 5+ children
// ============================================================================

#[test]
fn branch_with_four_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..4).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(children.clone()));
    assert_eq!(arena.get(parent).children().len(), 4);

    for (i, &child) in arena.get(parent).children().iter().enumerate() {
        assert_eq!(arena.get(child).value(), i as i32);
    }
}

#[test]
fn branch_with_five_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..5)
        .map(|i| arena.alloc(TreeNode::leaf(i * 100)))
        .collect();
    let parent = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(parent).children().len(), 5);
}

#[test]
fn branch_with_ten_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(children));

    let child_handles: Vec<NodeHandle> = {
        let r = arena.get(parent);
        r.children().to_vec()
    };

    assert_eq!(child_handles.len(), 10);
    for (i, child) in child_handles.iter().enumerate() {
        assert_eq!(arena.get(*child).value(), i as i32);
    }
}

#[test]
fn branch_with_fifty_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(parent).children().len(), 50);
}

// ============================================================================
// 17–18. Symbol ranges and edge values
// ============================================================================

#[test]
fn symbol_range_small_positive() {
    let mut arena = TreeArena::new();
    for v in [0, 1, 100, 255, 1000] {
        let h = arena.alloc(TreeNode::leaf(v));
        assert_eq!(arena.get(h).symbol(), v);
    }
}

#[test]
fn symbol_range_large_positive() {
    let mut arena = TreeArena::new();
    for v in [65535, 100_000, i32::MAX] {
        let h = arena.alloc(TreeNode::leaf(v));
        assert_eq!(arena.get(h).symbol(), v);
    }
}

#[test]
fn symbol_range_negative() {
    let mut arena = TreeArena::new();
    for v in [-1, -100, -65535, i32::MIN] {
        let h = arena.alloc(TreeNode::leaf(v));
        assert_eq!(arena.get(h).symbol(), v);
    }
}

#[test]
fn branch_symbol_range() {
    let mut arena = TreeArena::new();
    for v in [0, 1, 100, 65535, -1, i32::MAX, i32::MIN] {
        let h = arena.alloc(TreeNode::branch_with_symbol(v, vec![]));
        assert_eq!(arena.get(h).symbol(), v);
    }
}

// ============================================================================
// 19. Large allocations don't corrupt earlier handles
// ============================================================================

#[test]
fn large_alloc_preserves_earlier_handles() {
    let mut arena = TreeArena::new();
    let sentinel = arena.alloc(TreeNode::leaf(777));

    for i in 0..10_000 {
        arena.alloc(TreeNode::leaf(i));
    }

    assert_eq!(arena.get(sentinel).value(), 777);
}

#[test]
fn interleaved_leaf_and_branch_allocs_stable() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();

    for i in 0..500 {
        let leaf = arena.alloc(TreeNode::leaf(i));
        let branch = arena.alloc(TreeNode::branch(vec![leaf]));
        handles.push((leaf, branch, i));
    }

    for (leaf_h, branch_h, expected) in &handles {
        assert_eq!(arena.get(*leaf_h).value(), *expected);
        assert!(arena.get(*branch_h).is_branch());
        assert_eq!(arena.get(*branch_h).children()[0], *leaf_h);
    }
}

// ============================================================================
// 20. Arena len / is_empty
// ============================================================================

#[test]
fn new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_len_increments() {
    let mut arena = TreeArena::new();
    for i in 1..=10 {
        arena.alloc(TreeNode::leaf(i));
        assert_eq!(arena.len(), i as usize);
    }
}

#[test]
fn arena_not_empty_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(!arena.is_empty());
}

// ============================================================================
// Arena reset and clear
// ============================================================================

#[test]
fn reset_makes_arena_empty() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn reset_retains_capacity() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    let cap_before = arena.capacity();
    arena.reset();
    assert_eq!(arena.capacity(), cap_before);
}

#[test]
fn clear_reduces_to_one_chunk() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert!(arena.is_empty());
}

#[test]
fn clear_then_realloc_works() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();

    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
}

#[test]
fn reset_then_realloc_works() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();

    let h = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h).value(), 2);
}

#[test]
fn multiple_reset_cycles() {
    let mut arena = TreeArena::new();

    for cycle in 0..5 {
        for i in 0..100 {
            arena.alloc(TreeNode::leaf(cycle * 1000 + i));
        }
        assert_eq!(arena.len(), 100);
        arena.reset();
        assert!(arena.is_empty());
    }
}

// ============================================================================
// Arena with_capacity
// ============================================================================

#[test]
fn with_capacity_creates_single_chunk() {
    let arena = TreeArena::with_capacity(256);
    assert_eq!(arena.num_chunks(), 1);
    assert_eq!(arena.capacity(), 256);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

#[test]
fn with_capacity_one_works() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

// ============================================================================
// Chunk growth behavior
// ============================================================================

#[test]
fn small_capacity_triggers_growth() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 1);

    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn chunk_count_grows_logarithmically() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    // Exponential chunk sizes: 4, 8, 16, 32, 64, 128, 256, 512 = 1020
    assert!(arena.num_chunks() <= 10);
}

#[test]
fn capacity_grows_with_allocs() {
    let mut arena = TreeArena::with_capacity(4);
    let initial_cap = arena.capacity();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() > initial_cap);
}

// ============================================================================
// Mutable access (get_mut)
// ============================================================================

#[test]
fn get_mut_set_value_on_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    {
        let mut node = arena.get_mut(h);
        node.set_value(20);
    }
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn get_mut_does_not_affect_other_nodes() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    {
        let mut node = arena.get_mut(h1);
        node.set_value(999);
    }
    assert_eq!(arena.get(h1).value(), 999);
    assert_eq!(arena.get(h2).value(), 20);
}

#[test]
fn get_mut_then_get_consistent() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    {
        let mut node = arena.get_mut(h);
        node.set_value(50);
    }
    let v = arena.get(h).value();
    assert_eq!(v, 50);
}

// ============================================================================
// TreeNode standalone construction
// ============================================================================

#[test]
fn tree_node_leaf_constructor() {
    let node = TreeNode::leaf(42);
    assert!(node.is_leaf());
    assert_eq!(node.value(), 42);
    assert_eq!(node.symbol(), 42);
}

#[test]
fn tree_node_branch_constructor() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert_eq!(node.symbol(), 0);
}

#[test]
fn tree_node_branch_with_symbol_constructor() {
    let node = TreeNode::branch_with_symbol(55, vec![]);
    assert!(node.is_branch());
    assert_eq!(node.symbol(), 55);
    assert_eq!(node.value(), 55);
}

#[test]
fn tree_node_clone_equality() {
    let node = TreeNode::leaf(42);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

#[test]
fn tree_node_debug_format() {
    let node = TreeNode::leaf(1);
    let dbg = format!("{:?}", node);
    assert!(!dbg.is_empty());
}

// ============================================================================
// NodeHandle construction and traits
// ============================================================================

#[test]
fn node_handle_new_constructor() {
    let h = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    assert_eq!(h, h2);
}

#[test]
fn node_handle_different_indices_not_equal() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    assert_ne!(h1, h2);
}

#[test]
fn node_handle_debug_format() {
    let h = NodeHandle::new(1, 2);
    let dbg = format!("{:?}", h);
    assert!(!dbg.is_empty());
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

// ============================================================================
// Invalid handle (debug assertions)
// ============================================================================

#[test]
#[should_panic(expected = "Invalid node handle")]
#[cfg(debug_assertions)]
fn get_with_invalid_handle_panics() {
    let arena = TreeArena::new();
    let bad = NodeHandle::new(999, 999);
    let _node = arena.get(bad);
}

#[test]
#[should_panic(expected = "Invalid node handle")]
#[cfg(debug_assertions)]
fn get_mut_with_invalid_handle_panics() {
    let mut arena = TreeArena::new();
    let bad = NodeHandle::new(0, 999);
    let _node = arena.get_mut(bad);
}

// ============================================================================
// Memory and metrics
// ============================================================================

#[test]
fn memory_usage_positive_initially() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn memory_usage_grows_with_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    let initial = arena.memory_usage();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.memory_usage() > initial);
}

#[test]
fn metrics_snapshot_reflects_state() {
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
fn metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
}

// ============================================================================
// Default trait
// ============================================================================

#[test]
fn arena_default_is_new() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ============================================================================
// TreeNodeRef API
// ============================================================================

#[test]
fn tree_node_ref_value_and_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(33));
    let r = arena.get(h);
    assert_eq!(r.value(), 33);
    assert_eq!(r.symbol(), 33);
}

#[test]
fn tree_node_ref_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn tree_node_ref_is_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(!arena.get(h).is_leaf());
}

#[test]
fn tree_node_ref_children() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    let r = arena.get(h);
    assert_eq!(r.children().len(), 1);
    assert_eq!(r.children()[0], c);
}

#[test]
fn tree_node_ref_get_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    let r = arena.get(h);
    let inner = r.get_ref();
    assert_eq!(inner.value(), 7);
}

// ============================================================================
// Complex tree structures
// ============================================================================

#[test]
fn wide_tree_many_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch_with_symbol(1, children));

    let root_children: Vec<NodeHandle> = {
        let r = arena.get(root);
        r.children().to_vec()
    };

    assert_eq!(root_children.len(), 100);
    assert_eq!(arena.get(root_children[0]).value(), 0);
    assert_eq!(arena.get(root_children[99]).value(), 99);
}

#[test]
fn binary_tree_structure() {
    let mut arena = TreeArena::new();

    //       root(0)
    //      /       \
    //    b1(1)     b2(2)
    //   /   \     /   \
    //  l1   l2   l3   l4
    let l1 = arena.alloc(TreeNode::leaf(10));
    let l2 = arena.alloc(TreeNode::leaf(20));
    let l3 = arena.alloc(TreeNode::leaf(30));
    let l4 = arena.alloc(TreeNode::leaf(40));
    let b1 = arena.alloc(TreeNode::branch_with_symbol(1, vec![l1, l2]));
    let b2 = arena.alloc(TreeNode::branch_with_symbol(2, vec![l3, l4]));
    let root = arena.alloc(TreeNode::branch_with_symbol(0, vec![b1, b2]));

    assert_eq!(arena.get(root).children().len(), 2);
    assert_eq!(arena.get(b1).children().len(), 2);
    assert_eq!(arena.get(b2).children().len(), 2);
    assert!(arena.get(l1).is_leaf());
    assert_eq!(arena.get(l3).value(), 30);
}

#[test]
fn linear_chain_100_deep() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..100 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }

    assert_eq!(arena.get(current).symbol(), 99);
    assert_eq!(arena.len(), 100);
}

// ============================================================================
// Stress and boundary tests
// ============================================================================

#[test]
fn alloc_10000_all_retrievable() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..10_000)
        .map(|i| arena.alloc(TreeNode::leaf(i)))
        .collect();

    assert_eq!(arena.len(), 10_000);
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[4999]).value(), 4999);
    assert_eq!(arena.get(handles[9999]).value(), 9999);
}

#[test]
fn arena_capacity_one_grows_correctly() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.num_chunks(), 1);

    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 2);

    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
}

#[test]
fn reset_and_realloc_with_growth() {
    let mut arena = TreeArena::with_capacity(4);

    // Small session
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();

    // Larger session forces growth
    let handles: Vec<NodeHandle> = (0..20)
        .map(|i| arena.alloc(TreeNode::leaf(i * 2)))
        .collect();

    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), (i as i32) * 2);
    }
}

#[test]
fn mixed_leaf_and_branch_stability() {
    let mut arena = TreeArena::new();
    let mut all_handles = Vec::new();

    for i in 0..200 {
        if i % 3 == 0 {
            let h = arena.alloc(TreeNode::leaf(i));
            all_handles.push((h, i, true));
        } else {
            let h = arena.alloc(TreeNode::branch_with_symbol(i, vec![]));
            all_handles.push((h, i, false));
        }
    }

    for (h, expected_sym, is_leaf) in &all_handles {
        assert_eq!(arena.get(*h).symbol(), *expected_sym);
        if *is_leaf {
            assert!(arena.get(*h).is_leaf());
        } else {
            assert!(arena.get(*h).is_branch());
        }
    }
}
