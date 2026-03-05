//! Comprehensive tests for TreeNode construction, tree building, and arena operations.
//!
//! Organized into 8 sections with 55+ tests covering:
//! 1. TreeNode construction and properties
//! 2. TreeNode symbol and value semantics
//! 3. TreeNode children
//! 4. TreeNode leaf/branch classification
//! 5. Tree building with arena
//! 6. Tree mutation via get_mut
//! 7. Multi-level tree construction
//! 8. Edge cases

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ══════════════════════════════════════════════════
// 1. TreeNode construction and properties (8 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_leaf_creates_leaf_node() {
    let node = TreeNode::leaf(42);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
}

#[test]
fn test_leaf_stores_value() {
    let node = TreeNode::leaf(99);
    assert_eq!(node.value(), 99);
}

#[test]
fn test_leaf_symbol_equals_value() {
    let node = TreeNode::leaf(7);
    assert_eq!(node.symbol(), node.value());
    assert_eq!(node.symbol(), 7);
}

#[test]
fn test_leaf_negative_value() {
    let node = TreeNode::leaf(-1);
    assert_eq!(node.value(), -1);
}

#[test]
fn test_leaf_zero_value() {
    let node = TreeNode::leaf(0);
    assert_eq!(node.value(), 0);
    assert!(node.is_leaf());
}

#[test]
fn test_leaf_max_value() {
    let node = TreeNode::leaf(i32::MAX);
    assert_eq!(node.value(), i32::MAX);
}

#[test]
fn test_leaf_min_value() {
    let node = TreeNode::leaf(i32::MIN);
    assert_eq!(node.value(), i32::MIN);
}

#[test]
fn test_leaf_clone_preserves_value() {
    let node = TreeNode::leaf(55);
    let cloned = node.clone();
    assert_eq!(cloned.value(), 55);
    assert_eq!(node, cloned);
}

// ══════════════════════════════════════════════════
// 2. TreeNode symbol and value semantics (8 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_branch_default_symbol_is_zero() {
    let branch = TreeNode::branch(vec![]);
    assert_eq!(branch.symbol(), 0);
}

#[test]
fn test_branch_with_symbol_stores_symbol() {
    let branch = TreeNode::branch_with_symbol(10, vec![]);
    assert_eq!(branch.symbol(), 10);
    assert_eq!(branch.value(), 10);
}

#[test]
fn test_branch_with_symbol_negative() {
    let branch = TreeNode::branch_with_symbol(-5, vec![]);
    assert_eq!(branch.symbol(), -5);
}

#[test]
fn test_branch_value_equals_symbol() {
    let branch = TreeNode::branch_with_symbol(42, vec![]);
    assert_eq!(branch.value(), branch.symbol());
}

#[test]
fn test_distinct_leaves_have_distinct_symbols() {
    let a = TreeNode::leaf(1);
    let b = TreeNode::leaf(2);
    assert_ne!(a.symbol(), b.symbol());
}

#[test]
fn test_same_value_leaves_are_equal() {
    let a = TreeNode::leaf(100);
    let b = TreeNode::leaf(100);
    assert_eq!(a, b);
}

#[test]
fn test_different_value_leaves_are_not_equal() {
    let a = TreeNode::leaf(1);
    let b = TreeNode::leaf(2);
    assert_ne!(a, b);
}

#[test]
fn test_leaf_and_branch_with_same_symbol_differ() {
    let leaf = TreeNode::leaf(0);
    let branch = TreeNode::branch(vec![]);
    // Both have symbol 0, but they are different kinds
    assert_ne!(leaf, branch);
}

// ══════════════════════════════════════════════════
// 3. TreeNode children (8 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_leaf_has_no_children() {
    let node = TreeNode::leaf(1);
    assert!(node.children().is_empty());
}

#[test]
fn test_branch_with_no_children() {
    let branch = TreeNode::branch(vec![]);
    assert!(branch.children().is_empty());
}

#[test]
fn test_branch_with_one_child() {
    let handle = NodeHandle::new(0, 0);
    let branch = TreeNode::branch(vec![handle]);
    assert_eq!(branch.children().len(), 1);
    assert_eq!(branch.children()[0], handle);
}

#[test]
fn test_branch_with_multiple_children() {
    let h0 = NodeHandle::new(0, 0);
    let h1 = NodeHandle::new(0, 1);
    let h2 = NodeHandle::new(0, 2);
    let branch = TreeNode::branch(vec![h0, h1, h2]);
    assert_eq!(branch.children().len(), 3);
}

#[test]
fn test_branch_children_preserve_order() {
    let h0 = NodeHandle::new(0, 0);
    let h1 = NodeHandle::new(0, 1);
    let h2 = NodeHandle::new(0, 2);
    let branch = TreeNode::branch(vec![h0, h1, h2]);
    assert_eq!(branch.children()[0], h0);
    assert_eq!(branch.children()[1], h1);
    assert_eq!(branch.children()[2], h2);
}

#[test]
fn test_branch_with_symbol_has_children() {
    let h0 = NodeHandle::new(0, 0);
    let branch = TreeNode::branch_with_symbol(5, vec![h0]);
    assert_eq!(branch.children().len(), 1);
    assert_eq!(branch.symbol(), 5);
}

#[test]
fn test_children_from_arena_allocated_nodes() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));

    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 2);
    assert_eq!(arena.get(children[0]).value(), 10);
    assert_eq!(arena.get(children[1]).value(), 20);
}

#[test]
fn test_branch_clone_preserves_children() {
    let h0 = NodeHandle::new(0, 0);
    let h1 = NodeHandle::new(0, 1);
    let branch = TreeNode::branch(vec![h0, h1]);
    let cloned = branch.clone();
    assert_eq!(cloned.children(), branch.children());
}

// ══════════════════════════════════════════════════
// 4. TreeNode leaf/branch classification (5 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_leaf_is_leaf_not_branch() {
    let node = TreeNode::leaf(1);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
}

#[test]
fn test_branch_is_branch_not_leaf() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
}

#[test]
fn test_branch_with_symbol_is_branch() {
    let node = TreeNode::branch_with_symbol(5, vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
}

#[test]
fn test_arena_ref_leaf_detection() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(1));
    let node_ref = arena.get(handle);
    assert!(node_ref.is_leaf());
    assert!(!node_ref.is_branch());
}

#[test]
fn test_arena_ref_branch_detection() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(1));
    let handle = arena.alloc(TreeNode::branch(vec![child]));
    let node_ref = arena.get(handle);
    assert!(node_ref.is_branch());
    assert!(!node_ref.is_leaf());
}

// ══════════════════════════════════════════════════
// 5. Tree building with arena (8 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_arena_new_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_arena_alloc_increments_len() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());

    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
}

#[test]
fn test_arena_get_returns_correct_node() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(100));
    let h2 = arena.alloc(TreeNode::leaf(200));

    assert_eq!(arena.get(h1).value(), 100);
    assert_eq!(arena.get(h2).value(), 200);
}

#[test]
fn test_arena_handles_are_unique() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn test_arena_with_capacity() {
    let arena = TreeArena::with_capacity(8);
    assert_eq!(arena.capacity(), 8);
    assert_eq!(arena.num_chunks(), 1);
    assert!(arena.is_empty());
}

#[test]
fn test_arena_chunk_growth_on_overflow() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 1);

    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.num_chunks(), 2);
    assert_eq!(arena.len(), 3);
}

#[test]
fn test_arena_default_equals_new() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

#[test]
fn test_arena_memory_usage_positive() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(arena.memory_usage() > 0);
}

// ══════════════════════════════════════════════════
// 6. Tree mutation via get_mut (5 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_get_mut_set_value_on_leaf() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(handle).value(), 1);

    arena.get_mut(handle).set_value(99);
    assert_eq!(arena.get(handle).value(), 99);
}

#[test]
fn test_get_mut_set_value_multiple_times() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(0));

    for i in 1..=5 {
        arena.get_mut(handle).set_value(i);
        assert_eq!(arena.get(handle).value(), i);
    }
}

#[test]
fn test_get_mut_one_node_does_not_affect_another() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));

    arena.get_mut(h1).set_value(99);
    assert_eq!(arena.get(h1).value(), 99);
    assert_eq!(arena.get(h2).value(), 20);
}

#[test]
fn test_get_mut_deref_reads_symbol() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(42));
    let node_mut = arena.get_mut(handle);
    // DerefMut -> Deref -> TreeNode, so we can call symbol()
    assert_eq!(node_mut.symbol(), 42);
}

#[test]
fn test_get_mut_deref_reads_is_leaf() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(1));
    let node_mut = arena.get_mut(handle);
    assert!(node_mut.is_leaf());
    assert!(!node_mut.is_branch());
}

// ══════════════════════════════════════════════════
// 7. Multi-level tree construction (5 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_two_level_tree() {
    let mut arena = TreeArena::new();
    let leaf1 = arena.alloc(TreeNode::leaf(1));
    let leaf2 = arena.alloc(TreeNode::leaf(2));
    let root = arena.alloc(TreeNode::branch(vec![leaf1, leaf2]));

    assert!(arena.get(root).is_branch());
    assert_eq!(arena.get(root).children().len(), 2);
    assert!(arena.get(leaf1).is_leaf());
    assert!(arena.get(leaf2).is_leaf());
}

#[test]
fn test_three_level_tree() {
    let mut arena = TreeArena::new();

    // Level 3 (leaves)
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let l4 = arena.alloc(TreeNode::leaf(4));

    // Level 2 (branches)
    let left = arena.alloc(TreeNode::branch_with_symbol(10, vec![l1, l2]));
    let right = arena.alloc(TreeNode::branch_with_symbol(20, vec![l3, l4]));

    // Level 1 (root)
    let root = arena.alloc(TreeNode::branch_with_symbol(100, vec![left, right]));

    assert_eq!(arena.get(root).symbol(), 100);
    let root_ref = arena.get(root);
    let root_children = root_ref.children();
    assert_eq!(root_children.len(), 2);

    let left_handle = root_children[0];
    let right_handle = root_children[1];

    let left_ref = arena.get(left_handle);
    let left_children = left_ref.children();
    assert_eq!(left_children.len(), 2);
    assert_eq!(arena.get(left_children[0]).value(), 1);
    assert_eq!(arena.get(left_children[1]).value(), 2);

    let right_ref = arena.get(right_handle);
    let right_children = right_ref.children();
    assert_eq!(right_children.len(), 2);
    assert_eq!(arena.get(right_children[0]).value(), 3);
    assert_eq!(arena.get(right_children[1]).value(), 4);
}

#[test]
fn test_unbalanced_tree() {
    let mut arena = TreeArena::new();

    let deep = arena.alloc(TreeNode::leaf(42));
    let mid = arena.alloc(TreeNode::branch(vec![deep]));
    let shallow = arena.alloc(TreeNode::leaf(7));
    let root = arena.alloc(TreeNode::branch(vec![mid, shallow]));

    // root has 2 children: a branch and a leaf
    let root_ref = arena.get(root);
    let root_children = root_ref.children();
    let mid_handle = root_children[0];
    let shallow_handle = root_children[1];

    assert!(arena.get(mid_handle).is_branch());
    assert!(arena.get(shallow_handle).is_leaf());
    assert_eq!(arena.get(shallow_handle).value(), 7);

    // mid has 1 child: the deep leaf
    let mid_ref = arena.get(mid_handle);
    let mid_children = mid_ref.children();
    assert_eq!(mid_children.len(), 1);
    assert_eq!(arena.get(mid_children[0]).value(), 42);
}

#[test]
fn test_wide_tree_many_children() {
    let mut arena = TreeArena::new();
    let mut child_handles = Vec::new();
    for i in 0..20 {
        child_handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    let root = arena.alloc(TreeNode::branch(child_handles));

    let root_ref = arena.get(root);
    let children = root_ref.children();
    assert_eq!(children.len(), 20);
    for (i, &child) in children.iter().enumerate() {
        assert_eq!(arena.get(child).value(), i as i32);
    }
}

#[test]
fn test_deep_chain_tree() {
    let mut arena = TreeArena::new();

    // Build a linear chain: root -> ... -> leaf
    let leaf = arena.alloc(TreeNode::leaf(999));
    let mut current = leaf;
    for sym in (1..=10).rev() {
        current = arena.alloc(TreeNode::branch_with_symbol(sym, vec![current]));
    }

    // Walk from root to leaf
    let mut node_handle = current;
    for expected_sym in 1..=10 {
        let node_ref = arena.get(node_handle);
        assert!(node_ref.is_branch());
        assert_eq!(node_ref.symbol(), expected_sym);
        assert_eq!(node_ref.children().len(), 1);
        node_handle = node_ref.children()[0];
    }
    // Final node is the leaf
    assert!(arena.get(node_handle).is_leaf());
    assert_eq!(arena.get(node_handle).value(), 999);
}

// ══════════════════════════════════════════════════
// 8. Edge cases (8 tests)
// ══════════════════════════════════════════════════

#[test]
fn test_arena_reset_then_reuse() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h1).value(), 1);

    arena.reset();
    assert!(arena.is_empty());

    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_arena_clear_frees_excess_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    assert!(chunks_before > 1);

    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert!(arena.is_empty());
}

#[test]
fn test_arena_metrics_snapshot() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));

    let metrics = arena.metrics();
    assert_eq!(metrics.len(), 2);
    assert!(!metrics.is_empty());
    assert!(metrics.capacity() >= 2);
    assert!(metrics.num_chunks() >= 1);
    assert!(metrics.memory_usage() > 0);
}

#[test]
fn test_arena_metrics_empty() {
    let arena = TreeArena::new();
    let metrics = arena.metrics();
    assert_eq!(metrics.len(), 0);
    assert!(metrics.is_empty());
}

#[test]
fn test_node_handle_equality() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    let h3 = NodeHandle::new(0, 1);
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[test]
fn test_node_handle_is_copy() {
    let h1 = NodeHandle::new(0, 5);
    let h2 = h1; // Copy
    assert_eq!(h1, h2); // h1 still usable
}

#[test]
fn test_node_handle_hash_consistent() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let h1 = NodeHandle::new(1, 2);
    let h2 = NodeHandle::new(1, 2);
    set.insert(h1);
    assert!(set.contains(&h2));
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn test_arena_zero_capacity_panics() {
    let _arena = TreeArena::with_capacity(0);
}

// ══════════════════════════════════════════════════
// Additional edge case / integration tests
// ══════════════════════════════════════════════════

#[test]
fn test_arena_many_allocations_across_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    let mut handles = Vec::new();
    for i in 0..100 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert_eq!(arena.len(), 100);
    assert!(arena.num_chunks() > 1);

    // Verify all nodes are retrievable
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn test_branch_with_symbol_zero_children() {
    let branch = TreeNode::branch_with_symbol(77, vec![]);
    assert!(branch.is_branch());
    assert!(branch.children().is_empty());
    assert_eq!(branch.symbol(), 77);
}

#[test]
fn test_arena_reset_preserves_capacity() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();

    arena.reset();
    // reset keeps chunks, just clears data
    assert_eq!(arena.num_chunks(), chunks_before);
    assert!(arena.is_empty());
}

#[test]
fn test_tree_node_debug_format() {
    let node = TreeNode::leaf(42);
    let debug_str = format!("{node:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn test_node_handle_debug_format() {
    let handle = NodeHandle::new(1, 2);
    let debug_str = format!("{handle:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn test_arena_ref_value_and_symbol() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(55));
    let node_ref = arena.get(handle);
    assert_eq!(node_ref.value(), 55);
    assert_eq!(node_ref.symbol(), 55);
}

#[test]
fn test_arena_ref_get_ref() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(10));
    let node_ref = arena.get(handle);
    let inner: &TreeNode = node_ref.get_ref();
    assert_eq!(inner.value(), 10);
}

#[test]
fn test_set_value_on_branch_is_noop() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::branch_with_symbol(5, vec![]));
    // set_value only changes Leaf variant; on Branch it's a no-op
    arena.get_mut(handle).set_value(99);
    assert_eq!(arena.get(handle).symbol(), 5);
}
