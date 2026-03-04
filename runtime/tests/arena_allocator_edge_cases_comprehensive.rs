//! Comprehensive edge-case tests for TreeArena API.
//!
//! 60+ tests covering capacity, allocation, retrieval, handles,
//! growth, reset/clear, metrics, debug formatting, and multi-arena usage.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use std::collections::HashSet;

// ============================================================================
// 1. TreeArena::with_capacity – various sizes
// ============================================================================

#[test]
fn with_capacity_1() {
    let arena = TreeArena::with_capacity(1);
    assert_eq!(arena.capacity(), 1);
    assert!(arena.is_empty());
}

#[test]
fn with_capacity_10() {
    let arena = TreeArena::with_capacity(10);
    assert_eq!(arena.capacity(), 10);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn with_capacity_100() {
    let arena = TreeArena::with_capacity(100);
    assert_eq!(arena.capacity(), 100);
    assert_eq!(arena.len(), 0);
}

#[test]
fn with_capacity_1000() {
    let arena = TreeArena::with_capacity(1000);
    assert_eq!(arena.capacity(), 1000);
    assert!(arena.memory_usage() > 0);
}

#[test]
fn default_arena_has_default_capacity() {
    let arena = TreeArena::new();
    // Default chunk size is 1024
    assert_eq!(arena.capacity(), 1024);
}

#[test]
fn default_trait_matches_new() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.len(), b.len());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

// ============================================================================
// 2. with_capacity(0) panics
// ============================================================================

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

#[test]
fn with_capacity_zero_catch_unwind() {
    let result = std::panic::catch_unwind(|| TreeArena::with_capacity(0));
    assert!(result.is_err());
}

// ============================================================================
// 3. Node allocation and retrieval
// ============================================================================

#[test]
fn alloc_leaf_and_retrieve_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn alloc_leaf_negative_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-1));
    assert_eq!(arena.get(h).value(), -1);
}

#[test]
fn alloc_leaf_zero_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn alloc_leaf_i32_max() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn alloc_leaf_i32_min() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

#[test]
fn alloc_branch_no_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn alloc_branch_with_children() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    assert_eq!(arena.get(parent).children().len(), 2);
    assert_eq!(arena.get(parent).children()[0], c1);
    assert_eq!(arena.get(parent).children()[1], c2);
}

#[test]
fn alloc_branch_with_symbol() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(10));
    let b = arena.alloc(TreeNode::branch_with_symbol(99, vec![c]));
    assert_eq!(arena.get(b).symbol(), 99);
    assert_eq!(arena.get(b).value(), 99);
    assert!(arena.get(b).is_branch());
}

#[test]
fn leaf_is_leaf_not_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let node_ref = arena.get(h);
    assert!(node_ref.is_leaf());
    assert!(!node_ref.is_branch());
}

#[test]
fn branch_is_branch_not_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    let node_ref = arena.get(h);
    assert!(node_ref.is_branch());
    assert!(!node_ref.is_leaf());
}

#[test]
fn leaf_children_empty() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn symbol_and_value_agree_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(77));
    let r = arena.get(h);
    assert_eq!(r.symbol(), r.value());
}

#[test]
fn symbol_and_value_agree_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(88, vec![]));
    let r = arena.get(h);
    assert_eq!(r.symbol(), r.value());
}

// ============================================================================
// 4. NodeHandle indexing / identity
// ============================================================================

#[test]
fn node_handle_new_roundtrip() {
    let h = NodeHandle::new(3, 7);
    // We can't read chunk_idx/node_idx directly (private), but equality works.
    assert_eq!(h, NodeHandle::new(3, 7));
}

#[test]
fn distinct_handles_from_distinct_allocs() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn same_handle_retrieves_same_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), arena.get(h).value());
}

// ============================================================================
// 5. NodeHandle equality and ordering
// ============================================================================

#[test]
fn node_handle_equality() {
    let a = NodeHandle::new(0, 0);
    let b = NodeHandle::new(0, 0);
    assert_eq!(a, b);
}

#[test]
fn node_handle_inequality_chunk() {
    let a = NodeHandle::new(0, 0);
    let b = NodeHandle::new(1, 0);
    assert_ne!(a, b);
}

#[test]
fn node_handle_inequality_node() {
    let a = NodeHandle::new(0, 0);
    let b = NodeHandle::new(0, 1);
    assert_ne!(a, b);
}

#[test]
fn node_handle_copy_semantics() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = h1; // Copy
    assert_eq!(h1, h2); // h1 still usable
}

#[test]
fn node_handle_clone_semantics() {
    let h1 = NodeHandle::new(0, 5);
    let h2 = h1;
    assert_eq!(h1, h2);
}

#[test]
fn node_handle_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let a = NodeHandle::new(1, 2);
    let b = NodeHandle::new(1, 2);

    let mut ha = DefaultHasher::new();
    a.hash(&mut ha);
    let mut hb = DefaultHasher::new();
    b.hash(&mut hb);
    assert_eq!(ha.finish(), hb.finish());
}

#[test]
fn node_handle_hash_set_dedup() {
    let mut set = HashSet::new();
    let h = NodeHandle::new(0, 0);
    set.insert(h);
    set.insert(h);
    assert_eq!(set.len(), 1);
}

#[test]
fn node_handle_hash_set_distinct() {
    let mut set = HashSet::new();
    set.insert(NodeHandle::new(0, 0));
    set.insert(NodeHandle::new(0, 1));
    set.insert(NodeHandle::new(1, 0));
    assert_eq!(set.len(), 3);
}

// ============================================================================
// 6. Multiple allocations up to and beyond capacity
// ============================================================================

#[test]
fn fill_exactly_to_capacity() {
    let mut arena = TreeArena::with_capacity(5);
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 5);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn one_past_capacity_triggers_new_chunk() {
    let mut arena = TreeArena::with_capacity(5);
    for i in 0..6 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 6);
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn capacity_1_grows_correctly() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.num_chunks(), 1);
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 2);

    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
}

#[test]
fn many_allocs_all_retrievable() {
    let mut arena = TreeArena::with_capacity(3);
    let handles: Vec<_> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

// ============================================================================
// 7. Arena growth beyond initial capacity (exponential growth)
// ============================================================================

#[test]
fn exponential_growth_second_chunk_double() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    // First chunk = 4, second = 8 => total capacity >= 12
    assert!(arena.capacity() >= 12);
}

#[test]
fn growth_capped_at_max_chunk_size() {
    // Start large enough that doubling would exceed max
    let mut arena = TreeArena::with_capacity(40000);
    for i in 0..40001 {
        arena.alloc(TreeNode::leaf(i));
    }
    // Second chunk should be min(80000, 65536) = 65536
    assert!(arena.capacity() <= 40000 + 65536 + 1);
}

#[test]
fn many_chunks_all_accessible() {
    let mut arena = TreeArena::with_capacity(2);
    let handles: Vec<_> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert!(arena.num_chunks() > 1);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

// ============================================================================
// 8. Arena Debug format
// ============================================================================

#[test]
fn arena_debug_format_nonempty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let dbg = format!("{:?}", arena);
    assert!(dbg.contains("TreeArena"));
}

#[test]
fn arena_debug_format_empty() {
    let arena = TreeArena::new();
    let dbg = format!("{:?}", arena);
    assert!(!dbg.is_empty());
}

#[test]
fn node_handle_debug_format() {
    let h = NodeHandle::new(2, 5);
    let dbg = format!("{:?}", h);
    assert!(dbg.contains("NodeHandle"));
}

#[test]
fn tree_node_debug_format_leaf() {
    let node = TreeNode::leaf(42);
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("TreeNode"));
}

#[test]
fn tree_node_debug_format_branch() {
    let node = TreeNode::branch(vec![]);
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("TreeNode"));
}

// ============================================================================
// 9. TreeNode fields and construction
// ============================================================================

#[test]
fn tree_node_leaf_value() {
    let n = TreeNode::leaf(99);
    assert_eq!(n.value(), 99);
    assert_eq!(n.symbol(), 99);
    assert!(n.is_leaf());
}

#[test]
fn tree_node_branch_default_symbol_zero() {
    let n = TreeNode::branch(vec![]);
    assert_eq!(n.symbol(), 0);
}

#[test]
fn tree_node_branch_with_symbol_custom() {
    let n = TreeNode::branch_with_symbol(55, vec![]);
    assert_eq!(n.symbol(), 55);
}

#[test]
fn tree_node_clone() {
    let original = TreeNode::leaf(7);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn tree_node_partial_eq_leaf() {
    assert_eq!(TreeNode::leaf(1), TreeNode::leaf(1));
    assert_ne!(TreeNode::leaf(1), TreeNode::leaf(2));
}

#[test]
fn tree_node_partial_eq_branch() {
    let a = TreeNode::branch(vec![]);
    let b = TreeNode::branch(vec![]);
    assert_eq!(a, b);
}

#[test]
fn tree_node_partial_eq_branch_different_children() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    assert_ne!(TreeNode::branch(vec![h1]), TreeNode::branch(vec![h2]));
}

#[test]
fn tree_node_leaf_ne_branch() {
    let leaf = TreeNode::leaf(0);
    let branch = TreeNode::branch(vec![]);
    assert_ne!(leaf, branch);
}

// ============================================================================
// 10. Multiple arenas simultaneously
// ============================================================================

#[test]
fn two_arenas_independent() {
    let mut a1 = TreeArena::new();
    let mut a2 = TreeArena::new();

    let h1 = a1.alloc(TreeNode::leaf(100));
    let h2 = a2.alloc(TreeNode::leaf(200));

    assert_eq!(a1.get(h1).value(), 100);
    assert_eq!(a2.get(h2).value(), 200);
    assert_eq!(a1.len(), 1);
    assert_eq!(a2.len(), 1);
}

#[test]
fn three_arenas_independent_lifetimes() {
    let mut a = TreeArena::with_capacity(2);
    let mut b = TreeArena::with_capacity(5);
    let mut c = TreeArena::with_capacity(10);

    let ha = a.alloc(TreeNode::leaf(1));
    let hb = b.alloc(TreeNode::leaf(2));
    let hc = c.alloc(TreeNode::leaf(3));

    assert_eq!(a.get(ha).value(), 1);
    assert_eq!(b.get(hb).value(), 2);
    assert_eq!(c.get(hc).value(), 3);
}

#[test]
fn reset_one_arena_does_not_affect_other() {
    let mut a1 = TreeArena::new();
    let mut a2 = TreeArena::new();

    let _h1 = a1.alloc(TreeNode::leaf(1));
    let h2 = a2.alloc(TreeNode::leaf(2));

    a1.reset();

    assert!(a1.is_empty());
    assert_eq!(a2.len(), 1);
    assert_eq!(a2.get(h2).value(), 2);
}

// ============================================================================
// 11. Arena with single node
// ============================================================================

#[test]
fn single_leaf_len_is_one() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());
}

#[test]
fn single_branch_len_is_one() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::branch(vec![]));
    assert_eq!(arena.len(), 1);
}

#[test]
fn single_node_reset_and_realloc() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(10));
    assert_eq!(arena.get(h1).value(), 10);

    arena.reset();
    assert!(arena.is_empty());

    let h2 = arena.alloc(TreeNode::leaf(20));
    assert_eq!(arena.get(h2).value(), 20);
    assert_eq!(arena.len(), 1);
}

// ============================================================================
// 12. Mutable access edge cases
// ============================================================================

#[test]
fn mutate_leaf_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).value(), 1);
    arena.get_mut(h).set_value(999);
    assert_eq!(arena.get(h).value(), 999);
}

#[test]
fn mutate_does_not_affect_other_nodes() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));

    arena.get_mut(h1).set_value(99);
    assert_eq!(arena.get(h1).value(), 99);
    assert_eq!(arena.get(h2).value(), 20);
}

// ============================================================================
// 13. Reset and clear behaviour
// ============================================================================

#[test]
fn reset_preserves_capacity() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let cap_before = arena.capacity();
    arena.reset();
    assert_eq!(arena.capacity(), cap_before);
}

#[test]
fn clear_reduces_to_one_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert!(arena.is_empty());
}

#[test]
fn clear_then_alloc_works() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
}

#[test]
fn double_reset_is_idempotent() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn double_clear_is_idempotent() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn reset_empty_arena() {
    let mut arena = TreeArena::new();
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn clear_empty_arena() {
    let mut arena = TreeArena::new();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

// ============================================================================
// 14. Metrics API
// ============================================================================

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
fn metrics_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    let m = arena.metrics();
    assert_eq!(m.len(), 2);
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
    let dbg = format!("{:?}", m);
    assert!(dbg.contains("ArenaMetrics"));
}

#[test]
fn memory_usage_grows_with_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    let mem_before = arena.memory_usage();
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.memory_usage() > mem_before);
}

// ============================================================================
// 15. Deep tree construction
// ============================================================================

#[test]
fn deep_tree_chain() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..50 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.get(current).symbol(), 49);
    assert_eq!(arena.get(current).children().len(), 1);
}

#[test]
fn wide_tree() {
    let mut arena = TreeArena::new();
    let children: Vec<_> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch_with_symbol(999, children.clone()));
    assert_eq!(arena.get(root).children().len(), 100);
    assert_eq!(arena.get(arena.get(root).children()[50]).value(), 50);
}

// ============================================================================
// 16. TreeNodeRef via Deref
// ============================================================================

#[test]
fn tree_node_ref_deref_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    // Deref to TreeNode methods
    let r = arena.get(h);
    assert!(r.is_leaf());
    assert_eq!(r.symbol(), 5);
}

#[test]
fn tree_node_ref_get_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    let r = arena.get(h);
    let inner: &TreeNode = r.get_ref();
    assert_eq!(inner.value(), 7);
}

#[test]
fn tree_node_ref_as_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(8));
    let r = arena.get(h);
    let inner: &TreeNode = r.as_ref();
    assert_eq!(inner.value(), 8);
}

// ============================================================================
// 17. Stress / boundary tests
// ============================================================================

#[test]
fn alloc_1000_nodes_sequential() {
    let mut arena = TreeArena::with_capacity(10);
    let handles: Vec<_> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 1000);
    assert_eq!(arena.get(handles[999]).value(), 999);
}

#[test]
fn alloc_and_reset_cycle() {
    let mut arena = TreeArena::with_capacity(8);
    for cycle in 0..5 {
        for i in 0..20 {
            arena.alloc(TreeNode::leaf(cycle * 100 + i));
        }
        assert_eq!(arena.len(), 20);
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn interleave_leaf_and_branch() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let b1 = arena.alloc(TreeNode::branch(vec![l1]));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b2 = arena.alloc(TreeNode::branch(vec![l2, b1]));

    assert!(arena.get(b2).is_branch());
    assert_eq!(arena.get(b2).children().len(), 2);
    assert!(arena.get(l1).is_leaf());
}

// ============================================================================
// 18. is_empty / len consistency
// ============================================================================

#[test]
fn is_empty_true_when_new() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn is_empty_false_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(0));
    assert!(!arena.is_empty());
    assert_eq!(arena.len(), 1);
}

#[test]
fn len_increments_correctly() {
    let mut arena = TreeArena::with_capacity(3);
    for expected in 1..=10 {
        arena.alloc(TreeNode::leaf(0));
        assert_eq!(arena.len(), expected);
    }
}

// ============================================================================
// 19. Handle from arena used after more allocations
// ============================================================================

#[test]
fn early_handle_valid_after_growth() {
    let mut arena = TreeArena::with_capacity(2);
    let first = arena.alloc(TreeNode::leaf(42));
    for i in 1..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    // First handle still valid after many chunk growths
    assert_eq!(arena.get(first).value(), 42);
}

#[test]
#[should_panic(expected = "Invalid node handle")]
#[cfg(debug_assertions)]
fn invalid_handle_chunk_out_of_range() {
    let arena = TreeArena::new();
    let bad = NodeHandle::new(100, 0);
    let _node = arena.get(bad);
}

#[test]
#[should_panic(expected = "Invalid node handle")]
#[cfg(debug_assertions)]
fn invalid_handle_node_out_of_range() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let bad = NodeHandle::new(0, 999);
    let _node = arena.get(bad);
}
