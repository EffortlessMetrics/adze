//! Comprehensive v3 tests for TreeArena, NodeHandle, TreeNode, and related types.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ===== Section 1: TreeArena construction =====

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
fn new_arena_default_capacity() {
    let arena = TreeArena::new();
    assert_eq!(arena.capacity(), 1024);
}

#[test]
fn with_capacity_small() {
    let arena = TreeArena::with_capacity(1);
    assert_eq!(arena.capacity(), 1);
    assert!(arena.is_empty());
}

#[test]
fn with_capacity_large() {
    let arena = TreeArena::with_capacity(100_000);
    assert_eq!(arena.capacity(), 100_000);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

#[test]
fn default_equals_new() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.len(), b.len());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

// ===== Section 2: Single allocation =====

#[test]
fn alloc_leaf_returns_handle() {
    let mut arena = TreeArena::new();
    let _h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
}

#[test]
fn alloc_leaf_value_preserved() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn alloc_leaf_symbol_preserved() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    assert_eq!(arena.get(h).symbol(), 7);
}

#[test]
fn alloc_leaf_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn alloc_leaf_no_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn alloc_negative_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-42));
    assert_eq!(arena.get(h).value(), -42);
}

#[test]
fn alloc_zero_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn alloc_i32_max() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn alloc_i32_min() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

// ===== Section 3: Branch nodes =====

#[test]
fn alloc_branch_empty_children() {
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
    assert_eq!(arena.get(parent).children(), &[c1, c2]);
}

#[test]
fn branch_default_symbol_is_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert_eq!(arena.get(h).symbol(), 0);
}

#[test]
fn branch_with_symbol_preserves_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(55, vec![]));
    assert_eq!(arena.get(h).symbol(), 55);
}

#[test]
fn branch_with_symbol_and_children() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(10));
    let h = arena.alloc(TreeNode::branch_with_symbol(77, vec![c]));
    assert_eq!(arena.get(h).symbol(), 77);
    assert_eq!(arena.get(h).children(), &[c]);
}

#[test]
fn branch_is_not_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(!arena.get(h).is_leaf());
}

// ===== Section 4: Multiple allocations =====

#[test]
fn multiple_leaves_all_accessible() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..20).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn handles_are_distinct() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn len_tracks_allocations() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
        assert_eq!(arena.len(), (i + 1) as usize);
    }
}

#[test]
fn is_empty_false_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(0));
    assert!(!arena.is_empty());
}

// ===== Section 5: Arena growth =====

#[test]
fn arena_grows_when_full() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 1);

    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn capacity_doubles_on_growth() {
    let mut arena = TreeArena::with_capacity(4);
    // Fill first chunk of capacity 4
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    // Trigger second chunk (capacity 8)
    arena.alloc(TreeNode::leaf(100));
    assert_eq!(arena.capacity(), 4 + 8);
}

#[test]
fn multiple_chunk_growth() {
    let mut arena = TreeArena::with_capacity(1);
    // chunk 0: cap 1, chunk 1: cap 2, chunk 2: cap 4
    arena.alloc(TreeNode::leaf(0)); // fills chunk 0
    arena.alloc(TreeNode::leaf(1)); // new chunk 1
    arena.alloc(TreeNode::leaf(2)); // fills chunk 1
    arena.alloc(TreeNode::leaf(3)); // new chunk 2
    assert_eq!(arena.num_chunks(), 3);
}

#[test]
fn handles_valid_across_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    let h3 = arena.alloc(TreeNode::leaf(30)); // triggers new chunk

    assert_eq!(arena.get(h1).value(), 10);
    assert_eq!(arena.get(h2).value(), 20);
    assert_eq!(arena.get(h3).value(), 30);
}

#[test]
fn chunk_growth_capped_at_max() {
    // MAX_CHUNK_SIZE is 65536; start at 32768 so next would be 65536
    let mut arena = TreeArena::with_capacity(32768);
    for i in 0..32768 {
        arena.alloc(TreeNode::leaf(i));
    }
    // Trigger second chunk
    arena.alloc(TreeNode::leaf(0));
    // Second chunk should be min(32768*2, 65536) = 65536
    assert_eq!(arena.capacity(), 32768 + 65536);
}

// ===== Section 6: get / get_mut =====

#[test]
fn get_returns_correct_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    let node_ref = arena.get(h);
    assert_eq!(node_ref.value(), 42);
    assert!(node_ref.is_leaf());
}

#[test]
fn get_ref_returns_treenode_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let node_ref = arena.get(h);
    let inner: &TreeNode = node_ref.get_ref();
    assert_eq!(inner.value(), 5);
}

#[test]
fn get_as_ref_returns_treenode_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let node_ref = arena.get(h);
    let inner: &TreeNode = node_ref.as_ref();
    assert_eq!(inner.value(), 5);
}

#[test]
fn get_deref_to_tree_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let node_ref = arena.get(h);
    // Deref allows calling TreeNode methods directly
    assert_eq!(node_ref.symbol(), 5);
    assert!(node_ref.is_leaf());
}

#[test]
fn get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(999);
    assert_eq!(arena.get(h).value(), 999);
}

#[test]
fn get_mut_deref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    {
        let node_mut = arena.get_mut(h);
        assert_eq!(node_mut.value(), 10);
    }
}

#[test]
fn get_mut_deref_mut() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn set_value_only_affects_target() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));

    arena.get_mut(h1).set_value(100);

    assert_eq!(arena.get(h1).value(), 100);
    assert_eq!(arena.get(h2).value(), 2);
}

// ===== Section 7: Node data preservation =====

#[test]
fn leaf_clone_preserves_value() {
    let node = TreeNode::leaf(42);
    let cloned = node.clone();
    assert_eq!(cloned.value(), 42);
}

#[test]
fn branch_clone_preserves_children() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let node = TreeNode::branch(vec![h1, h2]);
    let cloned = node.clone();
    assert_eq!(cloned.children(), &[h1, h2]);
}

#[test]
fn leaf_equality() {
    assert_eq!(TreeNode::leaf(42), TreeNode::leaf(42));
    assert_ne!(TreeNode::leaf(1), TreeNode::leaf(2));
}

#[test]
fn branch_equality() {
    let h = NodeHandle::new(0, 0);
    assert_eq!(TreeNode::branch(vec![h]), TreeNode::branch(vec![h]));
}

#[test]
fn leaf_debug_format() {
    let node = TreeNode::leaf(42);
    let debug_str = format!("{:?}", node);
    assert!(debug_str.contains("42"));
}

#[test]
fn node_handle_copy() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = h1; // Copy
    assert_eq!(h1, h2);
}

#[test]
fn node_handle_clone() {
    let h1 = NodeHandle::new(1, 2);
    let h2 = h1;
    assert_eq!(h1, h2);
}

#[test]
fn node_handle_eq() {
    assert_eq!(NodeHandle::new(0, 0), NodeHandle::new(0, 0));
    assert_ne!(NodeHandle::new(0, 0), NodeHandle::new(0, 1));
    assert_ne!(NodeHandle::new(0, 0), NodeHandle::new(1, 0));
}

#[test]
fn node_handle_hash_consistent() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(NodeHandle::new(0, 0));
    set.insert(NodeHandle::new(0, 0)); // duplicate
    assert_eq!(set.len(), 1);
}

#[test]
fn node_handle_hash_distinct() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(NodeHandle::new(0, 0));
    set.insert(NodeHandle::new(0, 1));
    set.insert(NodeHandle::new(1, 0));
    assert_eq!(set.len(), 3);
}

// ===== Section 8: Reset and clear =====

#[test]
fn reset_makes_empty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn reset_preserves_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn reset_allows_reallocation() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn clear_makes_empty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn clear_keeps_one_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn clear_allows_reallocation() {
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

#[test]
fn reset_then_clear() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert!(arena.is_empty());
}

// ===== Section 9: Metrics =====

#[test]
fn metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.num_chunks(), 1);
    assert_eq!(m.capacity(), 1024);
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
fn metrics_eq() {
    let arena = TreeArena::new();
    let m1 = arena.metrics();
    let m2 = arena.metrics();
    assert_eq!(m1, m2);
}

#[test]
fn metrics_clone() {
    let arena = TreeArena::new();
    let m1 = arena.metrics();
    let m2 = m1;
    assert_eq!(m1, m2);
}

#[test]
fn metrics_debug() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    let s = format!("{:?}", m);
    assert!(s.contains("ArenaMetrics"));
}

#[test]
fn memory_usage_positive_for_nonempty_capacity() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn memory_usage_grows_with_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    let usage_before = arena.memory_usage();
    for i in 0..3 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.memory_usage() > usage_before);
}

// ===== Section 10: Stress tests =====

#[test]
fn stress_1000_leaves() {
    let mut arena = TreeArena::with_capacity(16);
    let handles: Vec<_> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 1000);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn stress_deep_tree() {
    let mut arena = TreeArena::new();
    let mut prev = arena.alloc(TreeNode::leaf(0));
    for i in 1..200 {
        prev = arena.alloc(TreeNode::branch_with_symbol(i, vec![prev]));
    }
    // Walk from root to leaf
    let mut current = prev;
    for i in (1..200).rev() {
        let node = arena.get(current);
        assert_eq!(node.symbol(), i);
        assert!(node.is_branch());
        current = node.children()[0];
    }
    assert_eq!(arena.get(current).value(), 0);
    assert!(arena.get(current).is_leaf());
}

#[test]
fn stress_wide_tree() {
    let mut arena = TreeArena::new();
    let children: Vec<_> = (0..500).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(children.clone()));
    assert_eq!(arena.get(root).children().len(), 500);
    for (i, &c) in children.iter().enumerate() {
        assert_eq!(arena.get(c).value(), i as i32);
    }
}

#[test]
fn stress_alloc_reset_repeat() {
    let mut arena = TreeArena::with_capacity(8);
    for round in 0..50 {
        for j in 0..20 {
            arena.alloc(TreeNode::leaf(round * 20 + j));
        }
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn stress_alloc_clear_repeat() {
    let mut arena = TreeArena::with_capacity(8);
    for _ in 0..50 {
        for j in 0..20 {
            arena.alloc(TreeNode::leaf(j));
        }
        arena.clear();
        assert!(arena.is_empty());
        assert_eq!(arena.num_chunks(), 1);
    }
}

// ===== Section 11: Edge cases =====

#[test]
fn capacity_one_multiple_allocs() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    let h3 = arena.alloc(TreeNode::leaf(30));
    assert_eq!(arena.get(h1).value(), 10);
    assert_eq!(arena.get(h2).value(), 20);
    assert_eq!(arena.get(h3).value(), 30);
}

#[test]
fn branch_referencing_itself_handle() {
    // A branch can hold handles that were allocated before it
    let mut arena = TreeArena::new();
    let a = arena.alloc(TreeNode::leaf(1));
    let b = arena.alloc(TreeNode::branch(vec![a, a, a]));
    assert_eq!(arena.get(b).children().len(), 3);
    assert_eq!(arena.get(b).children()[0], a);
}

#[test]
fn branch_with_symbol_negative() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(-1, vec![]));
    assert_eq!(arena.get(h).symbol(), -1);
}

#[test]
fn leaf_children_always_empty() {
    let node = TreeNode::leaf(42);
    assert!(node.children().is_empty());
}

#[test]
fn branch_value_equals_symbol() {
    let node = TreeNode::branch_with_symbol(10, vec![]);
    assert_eq!(node.value(), node.symbol());
}

#[test]
fn leaf_value_equals_symbol() {
    let node = TreeNode::leaf(10);
    assert_eq!(node.value(), node.symbol());
}

#[test]
fn arena_debug_does_not_panic() {
    let arena = TreeArena::new();
    let _ = format!("{:?}", arena);
}

#[test]
fn node_handle_debug_does_not_panic() {
    let h = NodeHandle::new(0, 0);
    let _ = format!("{:?}", h);
}

#[test]
fn set_value_on_branch_is_noop() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(5, vec![]));
    arena.get_mut(h).set_value(999);
    // set_value only changes Leaf nodes, so branch symbol stays
    assert_eq!(arena.get(h).symbol(), 5);
}

#[test]
fn treenode_ref_children_on_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let node_ref = arena.get(h);
    assert!(node_ref.children().is_empty());
}

#[test]
fn treenode_ref_is_branch_on_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    let node_ref = arena.get(h);
    assert!(node_ref.is_branch());
    assert!(!node_ref.is_leaf());
}

#[test]
fn treenode_ref_is_leaf_on_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let node_ref = arena.get(h);
    assert!(node_ref.is_leaf());
    assert!(!node_ref.is_branch());
}

#[test]
fn reset_empty_arena_is_noop() {
    let mut arena = TreeArena::new();
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn clear_empty_arena_is_noop() {
    let mut arena = TreeArena::new();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn capacity_monotonically_grows() {
    let mut arena = TreeArena::with_capacity(4);
    let mut prev_cap = arena.capacity();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
        let cap = arena.capacity();
        assert!(cap >= prev_cap);
        prev_cap = cap;
    }
}

#[test]
fn num_chunks_monotonically_grows() {
    let mut arena = TreeArena::with_capacity(4);
    let mut prev = arena.num_chunks();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
        let cur = arena.num_chunks();
        assert!(cur >= prev);
        prev = cur;
    }
}

#[test]
fn many_branches_with_shared_children() {
    let mut arena = TreeArena::new();
    let shared = arena.alloc(TreeNode::leaf(42));
    let parents: Vec<_> = (0..100)
        .map(|i| arena.alloc(TreeNode::branch_with_symbol(i, vec![shared])))
        .collect();
    for (i, &p) in parents.iter().enumerate() {
        assert_eq!(arena.get(p).symbol(), i as i32);
        assert_eq!(arena.get(p).children()[0], shared);
    }
    assert_eq!(arena.get(shared).value(), 42);
}

#[test]
fn interleaved_leaf_and_branch() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();
    for i in 0..20 {
        if i % 2 == 0 {
            handles.push(arena.alloc(TreeNode::leaf(i)));
        } else {
            let prev = handles[handles.len() - 1];
            handles.push(arena.alloc(TreeNode::branch(vec![prev])));
        }
    }
    for (i, &h) in handles.iter().enumerate() {
        if i % 2 == 0 {
            assert!(arena.get(h).is_leaf());
        } else {
            assert!(arena.get(h).is_branch());
        }
    }
}

#[test]
fn get_mut_then_get_consistent() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(42);
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.get(h).symbol(), 42);
}

#[test]
fn metrics_capacity_matches_arena_capacity() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.capacity(), arena.capacity());
    assert_eq!(m.len(), arena.len());
    assert_eq!(m.num_chunks(), arena.num_chunks());
    assert_eq!(m.memory_usage(), arena.memory_usage());
}

#[test]
fn alloc_after_partial_fill_and_reset() {
    let mut arena = TreeArena::with_capacity(10);
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(100));
    assert_eq!(arena.get(h).value(), 100);
    assert_eq!(arena.len(), 1);
}
