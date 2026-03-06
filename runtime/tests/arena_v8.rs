//! Comprehensive tests for TreeArena (v8)
//!
//! Covers: construction, allocation, access, tree structures, clear/reset,
//! metrics, chunk growth, handle semantics, and mixed workloads.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ---------------------------------------------------------------------------
// 1. New arena is empty
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 2. Alloc one leaf, len == 1
// ---------------------------------------------------------------------------

#[test]
fn test_alloc_one_leaf_len() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_alloc_one_leaf_not_empty() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(0));
    assert!(!arena.is_empty());
}

// ---------------------------------------------------------------------------
// 3. Alloc many leaves (10, 100)
// ---------------------------------------------------------------------------

#[test]
fn test_alloc_ten_leaves() {
    let mut arena = TreeArena::new();
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 10);
}

#[test]
fn test_alloc_one_hundred_leaves() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 100);
}

// ---------------------------------------------------------------------------
// 4. Get returns correct value after alloc
// ---------------------------------------------------------------------------

#[test]
fn test_get_returns_correct_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn test_get_returns_correct_value_negative() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-7));
    assert_eq!(arena.get(h).value(), -7);
}

#[test]
fn test_get_returns_correct_value_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn test_get_returns_correct_value_max() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn test_get_returns_correct_value_min() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

// ---------------------------------------------------------------------------
// 5. Leaf node is_leaf == true, child_count == 0
// ---------------------------------------------------------------------------

#[test]
fn test_leaf_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
}

#[test]
fn test_leaf_children_len_is_zero() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).children().len(), 0);
}

#[test]
fn test_leaf_children_is_empty() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_leaf_is_not_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    assert!(!arena.get(h).is_branch());
}

// ---------------------------------------------------------------------------
// 6. Node with children: is_leaf == false, correct child_count
// ---------------------------------------------------------------------------

#[test]
fn test_branch_is_not_leaf() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch(vec![c]));
    assert!(!arena.get(parent).is_leaf());
}

#[test]
fn test_branch_is_branch() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch(vec![c]));
    assert!(arena.get(parent).is_branch());
}

#[test]
fn test_branch_children_len_one() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.get(parent).children().len(), 1);
}

#[test]
fn test_branch_children_len_three() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let c3 = arena.alloc(TreeNode::leaf(3));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2, c3]));
    assert_eq!(arena.get(parent).children().len(), 3);
}

// ---------------------------------------------------------------------------
// 7. Children are accessible via .children()
// ---------------------------------------------------------------------------

#[test]
fn test_children_handles_match() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children[0], c1);
    assert_eq!(children[1], c2);
}

#[test]
fn test_children_values_via_handles() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    let parent_ref = arena.get(parent);
    let ch0 = parent_ref.children()[0];
    let ch1 = parent_ref.children()[1];
    assert_eq!(arena.get(ch0).value(), 10);
    assert_eq!(arena.get(ch1).value(), 20);
}

#[test]
fn test_children_iteration() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..5)
        .map(|i| arena.alloc(TreeNode::leaf(i * 10)))
        .collect();
    let parent = arena.alloc(TreeNode::branch(handles.clone()));
    let parent_ref = arena.get(parent);
    let children: Vec<NodeHandle> = parent_ref.children().to_vec();
    for (idx, ch) in children.iter().enumerate() {
        assert_eq!(arena.get(*ch).value(), idx as i32 * 10);
    }
}

// ---------------------------------------------------------------------------
// 8. with_capacity creates arena (doesn't panic)
// ---------------------------------------------------------------------------

#[test]
fn test_with_capacity_one() {
    let arena = TreeArena::with_capacity(1);
    assert!(arena.is_empty());
}

#[test]
fn test_with_capacity_large() {
    let arena = TreeArena::with_capacity(100_000);
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_with_capacity_alloc_works() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(7));
    assert_eq!(arena.get(h).value(), 7);
    assert_eq!(arena.len(), 1);
}

#[test]
#[should_panic]
fn test_with_capacity_zero_panics() {
    TreeArena::with_capacity(0);
}

// ---------------------------------------------------------------------------
// 9. Arena clear makes it empty
// ---------------------------------------------------------------------------

#[test]
fn test_clear_makes_empty() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_clear_retains_one_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

// ---------------------------------------------------------------------------
// 10. Arena after clear, alloc works again
// ---------------------------------------------------------------------------

#[test]
fn test_alloc_after_clear() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 2);
}

#[test]
fn test_alloc_many_after_clear() {
    let mut arena = TreeArena::new();
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let mut handles = Vec::new();
    for i in 0..50 {
        handles.push(arena.alloc(TreeNode::leaf(i + 1000)));
    }
    assert_eq!(arena.len(), 50);
    for (idx, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), idx as i32 + 1000);
    }
}

// ---------------------------------------------------------------------------
// 11. Multiple TreeNode::leaf values
// ---------------------------------------------------------------------------

#[test]
fn test_leaf_value_positive() {
    let node = TreeNode::leaf(255);
    assert_eq!(node.value(), 255);
}

#[test]
fn test_leaf_value_negative() {
    let node = TreeNode::leaf(-1);
    assert_eq!(node.value(), -1);
}

#[test]
fn test_leaf_value_zero() {
    let node = TreeNode::leaf(0);
    assert_eq!(node.value(), 0);
}

#[test]
fn test_leaf_various_values() {
    for val in [1, 10, 100, 1000, -42, i32::MAX, i32::MIN] {
        let node = TreeNode::leaf(val);
        assert_eq!(node.value(), val);
    }
}

// ---------------------------------------------------------------------------
// 12. Branch with various child counts (0, 1, 5, 20)
// ---------------------------------------------------------------------------

#[test]
fn test_branch_zero_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_branch_one_child() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.get(h).children().len(), 1);
}

#[test]
fn test_branch_five_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..5).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let h = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(h).children().len(), 5);
}

#[test]
fn test_branch_twenty_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..20).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let h = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(h).children().len(), 20);
}

#[test]
fn test_branch_with_symbol_preserves_value() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch_with_symbol(77, vec![c]));
    assert_eq!(arena.get(h).value(), 77);
    assert!(arena.get(h).is_branch());
}

// ---------------------------------------------------------------------------
// 13. Nested tree structures (children of children)
// ---------------------------------------------------------------------------

#[test]
fn test_nested_two_levels() {
    let mut arena = TreeArena::new();
    let leaf1 = arena.alloc(TreeNode::leaf(1));
    let leaf2 = arena.alloc(TreeNode::leaf(2));
    let inner = arena.alloc(TreeNode::branch(vec![leaf1, leaf2]));
    let root = arena.alloc(TreeNode::branch(vec![inner]));

    let root_ref = arena.get(root);
    let root_children: Vec<NodeHandle> = root_ref.children().to_vec();
    assert_eq!(root_children.len(), 1);
    let inner_ref = arena.get(root_children[0]);
    let inner_children: Vec<NodeHandle> = inner_ref.children().to_vec();
    assert_eq!(inner_children.len(), 2);
    assert_eq!(arena.get(inner_children[0]).value(), 1);
    assert_eq!(arena.get(inner_children[1]).value(), 2);
}

#[test]
fn test_nested_three_levels() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(99));
    let mid = arena.alloc(TreeNode::branch(vec![leaf]));
    let top = arena.alloc(TreeNode::branch(vec![mid]));

    let mid_h = arena.get(top).children()[0];
    let leaf_h = arena.get(mid_h).children()[0];
    assert_eq!(arena.get(leaf_h).value(), 99);
    assert!(arena.get(leaf_h).is_leaf());
}

#[test]
fn test_wide_and_deep_tree() {
    let mut arena = TreeArena::new();
    // Build 3 subtrees, each with 3 leaves
    let mut subtrees = Vec::new();
    for base in [0, 100, 200] {
        let leaves: Vec<NodeHandle> = (0..3)
            .map(|i| arena.alloc(TreeNode::leaf(base + i)))
            .collect();
        subtrees.push(arena.alloc(TreeNode::branch(leaves)));
    }
    let root = arena.alloc(TreeNode::branch(subtrees));

    assert_eq!(arena.get(root).children().len(), 3);
    // Total nodes: 9 leaves + 3 branches + 1 root = 13
    assert_eq!(arena.len(), 13);
}

// ---------------------------------------------------------------------------
// 14. Arena with 500+ nodes
// ---------------------------------------------------------------------------

#[test]
fn test_arena_five_hundred_nodes() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::with_capacity(500);
    for i in 0..500 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert_eq!(arena.len(), 500);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn test_arena_one_thousand_nodes() {
    let mut arena = TreeArena::new();
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 1000);
}

#[test]
fn test_arena_two_thousand_nodes() {
    let mut arena = TreeArena::new();
    for i in 0..2000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 2000);
}

// ---------------------------------------------------------------------------
// 15. num_chunks for various sizes
// ---------------------------------------------------------------------------

#[test]
fn test_num_chunks_default_stays_one_under_capacity() {
    let mut arena = TreeArena::new();
    // DEFAULT_CHUNK_SIZE is 1024, allocating 1000 should stay in 1 chunk
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_num_chunks_grows_past_capacity() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn test_num_chunks_small_capacity_many_allocs() {
    let mut arena = TreeArena::with_capacity(2);
    // 2 + 4 + 8 = 14 capacity across 3 chunks
    for i in 0..14 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 3);
}

#[test]
fn test_num_chunks_exact_boundary() {
    let mut arena = TreeArena::with_capacity(3);
    for i in 0..3 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 1);
    // One more triggers new chunk
    arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.num_chunks(), 2);
}

// ---------------------------------------------------------------------------
// 16. NodeHandle is Copy (can be used in multiple places)
// ---------------------------------------------------------------------------

#[test]
fn test_node_handle_is_copy() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    // Use handle multiple times without moving
    let a = h;
    let b = h;
    assert_eq!(arena.get(a).value(), 42);
    assert_eq!(arena.get(b).value(), 42);
    assert_eq!(a, b);
}

#[test]
fn test_node_handle_in_multiple_branches() {
    let mut arena = TreeArena::new();
    let shared = arena.alloc(TreeNode::leaf(7));
    // Same handle used as child of two different parents
    let p1 = arena.alloc(TreeNode::branch(vec![shared]));
    let p2 = arena.alloc(TreeNode::branch(vec![shared]));
    assert_eq!(arena.get(p1).children()[0], shared);
    assert_eq!(arena.get(p2).children()[0], shared);
}

#[test]
fn test_node_handle_stored_in_vec() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let v = vec![h, h, h];
    for &stored in &v {
        assert_eq!(arena.get(stored).value(), 5);
    }
}

#[test]
fn test_node_handle_equality() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
    let h1_copy = h1;
    assert_eq!(h1, h1_copy);
}

// ---------------------------------------------------------------------------
// 17. Arena preserves all nodes (no corruption after many allocs)
// ---------------------------------------------------------------------------

#[test]
fn test_no_corruption_after_many_allocs() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..200).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    // Verify every node still holds its original value
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn test_no_corruption_across_chunk_boundaries() {
    let mut arena = TreeArena::with_capacity(8);
    let handles: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert!(arena.num_chunks() > 1);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn test_branch_children_stable_after_more_allocs() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    // Allocate more nodes after creating the branch
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i + 1000));
    }
    // Original branch children still correct
    let parent_ref = arena.get(parent);
    let ch0 = parent_ref.children()[0];
    let ch1 = parent_ref.children()[1];
    assert_eq!(arena.get(ch0).value(), 10);
    assert_eq!(arena.get(ch1).value(), 20);
}

// ---------------------------------------------------------------------------
// 18. Sequential alloc + get roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_roundtrip_sequential_leaves() {
    let mut arena = TreeArena::new();
    for val in [0, 1, -1, 42, 1000, -999, i32::MAX] {
        let h = arena.alloc(TreeNode::leaf(val));
        assert_eq!(arena.get(h).value(), val);
    }
}

#[test]
fn test_roundtrip_sequential_branches() {
    let mut arena = TreeArena::new();
    for count in [0, 1, 3, 10] {
        let children: Vec<NodeHandle> =
            (0..count).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
        let parent = arena.alloc(TreeNode::branch(children));
        assert_eq!(arena.get(parent).children().len(), count as usize);
    }
}

#[test]
fn test_roundtrip_branch_with_symbol() {
    let mut arena = TreeArena::new();
    for symbol in [0, 1, 42, -1, 999] {
        let c = arena.alloc(TreeNode::leaf(0));
        let h = arena.alloc(TreeNode::branch_with_symbol(symbol, vec![c]));
        assert_eq!(arena.get(h).value(), symbol);
    }
}

// ---------------------------------------------------------------------------
// 19. Arena with mixed leaf and branch nodes
// ---------------------------------------------------------------------------

#[test]
fn test_mixed_leaf_and_branch_nodes() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let b2 = arena.alloc(TreeNode::branch(vec![l2, l3]));

    assert!(arena.get(l1).is_leaf());
    assert!(arena.get(l2).is_leaf());
    assert!(arena.get(l3).is_leaf());
    assert!(arena.get(b1).is_branch());
    assert!(arena.get(b2).is_branch());
    assert_eq!(arena.len(), 5);
}

#[test]
fn test_mixed_interleaved_allocs() {
    let mut arena = TreeArena::new();
    let mut leaves = Vec::new();
    let mut branches = Vec::new();
    for i in 0..20 {
        if i % 3 == 0 && !leaves.is_empty() {
            let children = leaves.clone();
            branches.push(arena.alloc(TreeNode::branch(children)));
            leaves.clear();
        } else {
            leaves.push(arena.alloc(TreeNode::leaf(i)));
        }
    }
    // All allocated nodes should be accessible
    for &h in &branches {
        assert!(arena.get(h).is_branch());
    }
    for &h in &leaves {
        assert!(arena.get(h).is_leaf());
    }
}

// ---------------------------------------------------------------------------
// 20. Building a simple tree structure (root -> children -> leaves)
// ---------------------------------------------------------------------------

#[test]
fn test_simple_expression_tree() {
    // Represent: (1 + 2) * 3
    let mut arena = TreeArena::new();
    let one = arena.alloc(TreeNode::leaf(1));
    let two = arena.alloc(TreeNode::leaf(2));
    let add = arena.alloc(TreeNode::branch_with_symbol(100, vec![one, two]));
    let three = arena.alloc(TreeNode::leaf(3));
    let mul = arena.alloc(TreeNode::branch_with_symbol(101, vec![add, three]));

    assert_eq!(arena.get(mul).value(), 101);
    let mul_ref = arena.get(mul);
    let mul_kids: Vec<NodeHandle> = mul_ref.children().to_vec();
    assert_eq!(mul_kids.len(), 2);
    assert_eq!(arena.get(mul_kids[0]).value(), 100);
    assert_eq!(arena.get(mul_kids[1]).value(), 3);
    let add_ref = arena.get(mul_kids[0]);
    let add_kids: Vec<NodeHandle> = add_ref.children().to_vec();
    assert_eq!(add_kids.len(), 2);
    assert_eq!(arena.get(add_kids[0]).value(), 1);
    assert_eq!(arena.get(add_kids[1]).value(), 2);
}

#[test]
fn test_flat_sibling_tree() {
    // root with 10 leaf children
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(root).children().len(), 10);
    for (i, &ch) in arena.get(root).children().iter().enumerate() {
        assert_eq!(arena.get(ch).value(), i as i32);
    }
}

#[test]
fn test_binary_tree_structure() {
    // Build a complete binary tree of depth 3 (7 nodes)
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let l4 = arena.alloc(TreeNode::leaf(4));
    let b1 = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let b2 = arena.alloc(TreeNode::branch(vec![l3, l4]));
    let root = arena.alloc(TreeNode::branch(vec![b1, b2]));

    assert_eq!(arena.len(), 7);
    assert!(arena.get(root).is_branch());
    assert_eq!(arena.get(root).children().len(), 2);
    for &child_h in arena.get(root).children() {
        assert!(arena.get(child_h).is_branch());
        assert_eq!(arena.get(child_h).children().len(), 2);
        for &leaf_h in arena.get(child_h).children() {
            assert!(arena.get(leaf_h).is_leaf());
        }
    }
}

// ---------------------------------------------------------------------------
// Additional: reset vs clear, metrics, capacity, edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_reset_retains_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert!(arena.is_empty());
    // reset keeps all chunks
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn test_alloc_after_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h).value(), 2);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_capacity_at_least_len() {
    let mut arena = TreeArena::new();
    for i in 0..50 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() >= arena.len());
}

#[test]
fn test_capacity_grows_with_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    let initial_cap = arena.capacity();
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() > initial_cap);
}

#[test]
fn test_memory_usage_positive() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(arena.memory_usage() > 0);
}

#[test]
fn test_metrics_snapshot() {
    let mut arena = TreeArena::new();
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.len(), 10);
    assert!(!m.is_empty());
    assert!(m.capacity() >= 10);
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

#[test]
fn test_default_creates_empty_arena() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_multiple_clear_cycles() {
    let mut arena = TreeArena::new();
    for cycle in 0..5 {
        for i in 0..20 {
            arena.alloc(TreeNode::leaf(cycle * 100 + i));
        }
        arena.clear();
        assert!(arena.is_empty());
    }
}

#[test]
fn test_multiple_reset_cycles() {
    let mut arena = TreeArena::new();
    for cycle in 0..5 {
        let h = arena.alloc(TreeNode::leaf(cycle));
        assert_eq!(arena.get(h).value(), cycle);
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn test_branch_default_symbol_is_zero() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn test_leaf_symbol_equals_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).symbol(), 42);
    assert_eq!(arena.get(h).value(), arena.get(h).symbol());
}

#[test]
fn test_deeply_nested_chain() {
    // Linear chain of 50 nodes deep
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..50 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    // Walk from root to leaf
    let mut node = current;
    for expected in (0..50).rev() {
        assert_eq!(arena.get(node).value(), expected);
        let node_ref = arena.get(node);
        let kids = node_ref.children();
        if !kids.is_empty() {
            node = kids[0];
        }
    }
}

#[test]
fn test_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    assert_eq!(arena.get(h).value(), 10);
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn test_node_handle_new() {
    // NodeHandle::new is public for testing
    let h = NodeHandle::new(0, 0);
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(5));
    assert_eq!(arena.get(h).value(), 5);
}

#[test]
fn test_tree_node_is_leaf_standalone() {
    let leaf = TreeNode::leaf(42);
    assert!(leaf.is_leaf());
    assert!(!leaf.is_branch());
}

#[test]
fn test_tree_node_is_branch_standalone() {
    let branch = TreeNode::branch(vec![]);
    assert!(branch.is_branch());
    assert!(!branch.is_leaf());
}

#[test]
fn test_tree_node_children_standalone() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let branch = TreeNode::branch(vec![h1, h2]);
    assert_eq!(branch.children().len(), 2);
    assert_eq!(branch.children()[0], h1);
}

#[test]
fn test_tree_node_leaf_children_empty() {
    let leaf = TreeNode::leaf(0);
    assert!(leaf.children().is_empty());
}

#[test]
fn test_large_branch_fifty_children() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let h = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(h).children().len(), 50);
}

#[test]
fn test_arena_len_after_mixed_allocs() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    let c = arena.alloc(TreeNode::leaf(3));
    arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.len(), 4);
}

#[test]
fn test_alloc_stress_and_verify() {
    let mut arena = TreeArena::with_capacity(16);
    let mut handles = Vec::new();
    for i in 0..600 {
        let h = if i % 5 == 0 && !handles.is_empty() {
            let last = *handles.last().unwrap();
            arena.alloc(TreeNode::branch_with_symbol(i, vec![last]))
        } else {
            arena.alloc(TreeNode::leaf(i))
        };
        handles.push(h);
    }
    assert_eq!(arena.len(), 600);
    // Spot check a sample of handles
    for &idx in &[0, 50, 100, 200, 400, 599] {
        assert_eq!(arena.get(handles[idx]).value(), idx as i32);
    }
}
