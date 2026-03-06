//! Comprehensive tests for the `TreeArena` arena allocator.
//!
//! Covers: creation, allocation, handles, node construction, get/get_mut,
//! parent-child tree building, clear/reset/capacity, multiple trees,
//! and edge cases (many nodes, deep trees, wide trees, empty arena).

use adze::arena_allocator::{ArenaMetrics, NodeHandle, TreeArena, TreeNode};

// ──────────────────────────────────────────────────────────────────────
// 1. Arena creation and basic alloc (8 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
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
fn test_with_capacity_creates_arena() {
    let arena = TreeArena::with_capacity(16);
    assert!(arena.is_empty());
    assert_eq!(arena.capacity(), 16);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn test_with_capacity_zero_panics() {
    let _ = TreeArena::with_capacity(0);
}

#[test]
fn test_alloc_single_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn test_alloc_increments_len() {
    let mut arena = TreeArena::new();
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 5);
}

#[test]
fn test_default_trait_creates_arena() {
    let arena: TreeArena = Default::default();
    assert!(arena.is_empty());
    assert_eq!(arena.capacity(), 1024);
}

// ──────────────────────────────────────────────────────────────────────
// 2. NodeHandle properties (5 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_node_handle_is_copy() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    // Use h twice without cloning — proves Copy
    let _ = arena.get(h);
    let _ = arena.get(h);
}

#[test]
fn test_node_handle_equality() {
    let h1 = NodeHandle::new(0, 5);
    let h2 = NodeHandle::new(0, 5);
    assert_eq!(h1, h2);
}

#[test]
fn test_node_handle_inequality() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    assert_ne!(h1, h2);
}

#[test]
fn test_node_handle_debug_format() {
    let h = NodeHandle::new(1, 2);
    let dbg = format!("{h:?}");
    assert!(dbg.contains("NodeHandle"));
}

#[test]
fn test_node_handle_hash_usable_in_set() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    set.insert(h1);
    set.insert(h2);
    set.insert(h1); // duplicate
    assert_eq!(set.len(), 2);
}

// ──────────────────────────────────────────────────────────────────────
// 3. TreeNode construction (8 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_leaf_node_value() {
    let node = TreeNode::leaf(99);
    assert_eq!(node.value(), 99);
    assert_eq!(node.symbol(), 99);
}

#[test]
fn test_leaf_is_leaf() {
    let node = TreeNode::leaf(1);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
}

#[test]
fn test_leaf_children_empty() {
    let node = TreeNode::leaf(1);
    assert!(node.children().is_empty());
}

#[test]
fn test_branch_node_no_children() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
    assert!(node.children().is_empty());
}

#[test]
fn test_branch_node_with_children() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let node = TreeNode::branch(vec![h1, h2]);
    assert_eq!(node.children().len(), 2);
    assert_eq!(node.children()[0], h1);
    assert_eq!(node.children()[1], h2);
}

#[test]
fn test_branch_default_symbol_is_zero() {
    let node = TreeNode::branch(vec![]);
    assert_eq!(node.symbol(), 0);
}

#[test]
fn test_branch_with_symbol() {
    let node = TreeNode::branch_with_symbol(42, vec![]);
    assert_eq!(node.symbol(), 42);
    assert_eq!(node.value(), 42);
    assert!(node.is_branch());
}

#[test]
fn test_tree_node_clone() {
    let node = TreeNode::leaf(7);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

// ──────────────────────────────────────────────────────────────────────
// 4. Arena get/get_mut (8 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_get_returns_correct_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    assert_eq!(arena.get(h).value(), 10);
}

#[test]
fn test_get_ref_deref_to_tree_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let node_ref = arena.get(h);
    // Deref allows calling TreeNode methods directly
    assert!(node_ref.is_leaf());
    assert_eq!(node_ref.symbol(), 5);
}

#[test]
fn test_get_ref_is_leaf_and_is_branch() {
    let mut arena = TreeArena::new();
    let leaf_h = arena.alloc(TreeNode::leaf(1));
    let branch_h = arena.alloc(TreeNode::branch(vec![leaf_h]));

    assert!(arena.get(leaf_h).is_leaf());
    assert!(!arena.get(leaf_h).is_branch());
    assert!(arena.get(branch_h).is_branch());
    assert!(!arena.get(branch_h).is_leaf());
}

#[test]
fn test_get_ref_children() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let p = arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.get(p).children(), &[c]);
}

#[test]
fn test_get_ref_get_ref_method() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(3));
    let node: &TreeNode = arena.get(h).get_ref();
    assert_eq!(node.value(), 3);
}

#[test]
fn test_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    arena.get_mut(h).set_value(99);
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn test_get_mut_deref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    // DerefMut should let us read via Deref
    let r = arena.get_mut(h);
    assert_eq!(r.value(), 7);
}

#[test]
fn test_get_multiple_handles_stable() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    let h2 = arena.alloc(TreeNode::leaf(20));
    let h3 = arena.alloc(TreeNode::leaf(30));

    // Values remain stable after multiple allocs
    assert_eq!(arena.get(h1).value(), 10);
    assert_eq!(arena.get(h2).value(), 20);
    assert_eq!(arena.get(h3).value(), 30);
}

// ──────────────────────────────────────────────────────────────────────
// 5. Tree building — parent-child relationships (8 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_simple_parent_child() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch(vec![child]));

    assert_eq!(arena.get(parent).children().len(), 1);
    assert_eq!(arena.get(parent).children()[0], child);
}

#[test]
fn test_parent_with_two_children() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));

    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 2);
    let c0 = children[0];
    let c1 = children[1];
    assert_eq!(arena.get(c0).value(), 1);
    assert_eq!(arena.get(c1).value(), 2);
}

#[test]
fn test_three_level_tree() {
    let mut arena = TreeArena::new();
    let grandchild = arena.alloc(TreeNode::leaf(100));
    let child = arena.alloc(TreeNode::branch(vec![grandchild]));
    let root = arena.alloc(TreeNode::branch(vec![child]));

    let root_ref = arena.get(root);
    let root_children = root_ref.children();
    assert_eq!(root_children.len(), 1);
    let child_h = root_children[0];
    let child_ref = arena.get(child_h);
    let child_children = child_ref.children();
    assert_eq!(child_children.len(), 1);
    let gc_h = child_children[0];
    assert_eq!(arena.get(gc_h).value(), 100);
}

#[test]
fn test_branch_with_symbol_and_children() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch_with_symbol(50, vec![c]));

    assert_eq!(arena.get(parent).symbol(), 50);
    assert_eq!(arena.get(parent).children().len(), 1);
}

#[test]
fn test_sibling_nodes_independent() {
    let mut arena = TreeArena::new();
    let s1 = arena.alloc(TreeNode::leaf(10));
    let s2 = arena.alloc(TreeNode::leaf(20));
    let _parent = arena.alloc(TreeNode::branch(vec![s1, s2]));

    // Siblings are independent nodes
    assert_ne!(s1, s2);
    assert_eq!(arena.get(s1).value(), 10);
    assert_eq!(arena.get(s2).value(), 20);
}

#[test]
fn test_shared_child_in_multiple_parents() {
    // NodeHandle is Copy, so same child can appear in multiple parents
    let mut arena = TreeArena::new();
    let shared = arena.alloc(TreeNode::leaf(42));
    let p1 = arena.alloc(TreeNode::branch(vec![shared]));
    let p2 = arena.alloc(TreeNode::branch(vec![shared]));

    assert_eq!(arena.get(p1).children()[0], arena.get(p2).children()[0]);
}

#[test]
fn test_leaf_has_no_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_nested_branch_only_tree() {
    let mut arena = TreeArena::new();
    let inner = arena.alloc(TreeNode::branch(vec![]));
    let middle = arena.alloc(TreeNode::branch(vec![inner]));
    let outer = arena.alloc(TreeNode::branch(vec![middle]));

    assert!(arena.get(outer).is_branch());
    assert!(arena.get(arena.get(outer).children()[0]).is_branch());
    let inner_h = arena.get(arena.get(outer).children()[0]).children()[0];
    assert!(arena.get(inner_h).is_branch());
    assert!(arena.get(inner_h).children().is_empty());
}

// ──────────────────────────────────────────────────────────────────────
// 6. Arena clear, reset, and capacity (5 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_reset_keeps_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..6 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    assert!(chunks_before > 1);

    arena.reset();
    assert!(arena.is_empty());
    // reset retains chunks
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn test_clear_shrinks_to_one_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..6 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);

    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_capacity_grows_with_allocations() {
    let mut arena = TreeArena::with_capacity(2);
    let initial_cap = arena.capacity();

    // Fill beyond first chunk
    for i in 0..3 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() > initial_cap);
}

#[test]
fn test_alloc_after_reset_reuses_arena() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.reset();

    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn test_alloc_after_clear_reuses_arena() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..6 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();

    let h = arena.alloc(TreeNode::leaf(77));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 77);
}

// ──────────────────────────────────────────────────────────────────────
// 7. Multiple trees in one arena (5 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_two_independent_trees() {
    let mut arena = TreeArena::new();

    // Tree 1
    let t1_c = arena.alloc(TreeNode::leaf(1));
    let t1_root = arena.alloc(TreeNode::branch(vec![t1_c]));

    // Tree 2
    let t2_c = arena.alloc(TreeNode::leaf(2));
    let t2_root = arena.alloc(TreeNode::branch(vec![t2_c]));

    assert_eq!(arena.get(t1_root).children().len(), 1);
    assert_eq!(arena.get(t2_root).children().len(), 1);
    assert_ne!(t1_root, t2_root);
}

#[test]
fn test_multiple_roots_same_arena() {
    let mut arena = TreeArena::new();
    let r1 = arena.alloc(TreeNode::branch(vec![]));
    let r2 = arena.alloc(TreeNode::branch(vec![]));
    let r3 = arena.alloc(TreeNode::branch(vec![]));

    assert_eq!(arena.len(), 3);
    assert_ne!(r1, r2);
    assert_ne!(r2, r3);
}

#[test]
fn test_trees_share_arena_len() {
    let mut arena = TreeArena::new();

    // Tree 1: root + 2 leaves = 3
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let _r1 = arena.alloc(TreeNode::branch(vec![l1, l2]));

    // Tree 2: root + 1 leaf = 2
    let l3 = arena.alloc(TreeNode::leaf(3));
    let _r2 = arena.alloc(TreeNode::branch(vec![l3]));

    assert_eq!(arena.len(), 5);
}

#[test]
fn test_tree_handles_valid_after_second_tree() {
    let mut arena = TreeArena::new();
    let h_first = arena.alloc(TreeNode::leaf(111));

    // Build another tree
    let c = arena.alloc(TreeNode::leaf(222));
    let _root = arena.alloc(TreeNode::branch(vec![c]));

    // First handle still valid
    assert_eq!(arena.get(h_first).value(), 111);
}

#[test]
fn test_cross_tree_child_sharing() {
    let mut arena = TreeArena::new();
    let shared_leaf = arena.alloc(TreeNode::leaf(42));

    let tree_a = arena.alloc(TreeNode::branch(vec![shared_leaf]));
    let tree_b = arena.alloc(TreeNode::branch(vec![shared_leaf]));

    assert_eq!(
        arena.get(arena.get(tree_a).children()[0]).value(),
        arena.get(arena.get(tree_b).children()[0]).value()
    );
}

// ──────────────────────────────────────────────────────────────────────
// 8. Edge cases (8 tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn test_many_nodes_forces_chunk_growth() {
    let mut arena = TreeArena::with_capacity(4);
    let mut handles = Vec::new();
    for i in 0..100 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert_eq!(arena.len(), 100);
    assert!(arena.num_chunks() > 1);

    // All handles still valid
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn test_deep_tree() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..200 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }

    // Walk down the spine
    let mut node_h = current;
    for i in (1..200).rev() {
        assert_eq!(arena.get(node_h).symbol(), i);
        let node_ref = arena.get(node_h);
        let children = node_ref.children();
        assert_eq!(children.len(), 1);
        node_h = children[0];
    }
    // Bottom leaf
    assert_eq!(arena.get(node_h).value(), 0);
    assert!(arena.get(node_h).is_leaf());
}

#[test]
fn test_wide_tree() {
    let mut arena = TreeArena::new();
    let mut children = Vec::new();
    for i in 0..500 {
        children.push(arena.alloc(TreeNode::leaf(i)));
    }
    let root = arena.alloc(TreeNode::branch(children));

    assert_eq!(arena.get(root).children().len(), 500);
    assert_eq!(arena.len(), 501); // 500 leaves + 1 root
}

#[test]
fn test_empty_arena_len_zero() {
    let arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn test_negative_symbol_values() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-1));
    assert_eq!(arena.get(h).value(), -1);

    let h2 = arena.alloc(TreeNode::branch_with_symbol(-100, vec![]));
    assert_eq!(arena.get(h2).symbol(), -100);
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
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));

    let m: ArenaMetrics = arena.metrics();
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
    assert_eq!(m.num_chunks(), 1);
}
