//! Arena allocator v6 tests — 64 tests across 8 categories:
//! alloc, get, handle, clear, capacity, tree, walk, edge.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use std::collections::HashSet;

// ───────────────────────────────────────────────────────────────
// Helpers
// ───────────────────────────────────────────────────────────────

/// Build a linear chain: leaf ← branch ← branch ← … (depth nodes).
#[allow(dead_code)]
fn build_chain(arena: &mut TreeArena, depth: usize) -> NodeHandle {
    let mut cur = arena.alloc(TreeNode::leaf(0));
    for i in 1..depth {
        cur = arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![cur]));
    }
    cur
}

/// Collect all handles reachable from `root` via DFS.
#[allow(dead_code)]
fn collect_handles(arena: &TreeArena, root: NodeHandle) -> Vec<NodeHandle> {
    let mut stack = vec![root];
    let mut out = Vec::new();
    while let Some(h) = stack.pop() {
        out.push(h);
        let node = arena.get(h);
        for &child in node.children().iter().rev() {
            stack.push(child);
        }
    }
    out
}

/// Count descendants (including root) reachable from `root`.
#[allow(dead_code)]
fn count_nodes(arena: &TreeArena, root: NodeHandle) -> usize {
    collect_handles(arena, root).len()
}

// ───────────────────────────────────────────────────────────────
// 1. arena_alloc_* — allocation operations (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_alloc_single_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn arena_alloc_multiple_leaves() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 10);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn arena_alloc_branch_with_children() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    assert!(arena.get(parent).is_branch());
    assert_eq!(arena.get(parent).children().len(), 2);
}

#[test]
fn arena_alloc_branch_with_symbol() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(7));
    let b = arena.alloc(TreeNode::branch_with_symbol(42, vec![c]));
    assert_eq!(arena.get(b).symbol(), 42);
}

#[test]
fn arena_alloc_empty_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn arena_alloc_distinct_handles() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(0));
    let h2 = arena.alloc(TreeNode::leaf(0));
    assert_ne!(h1, h2);
}

#[test]
fn arena_alloc_preserves_order() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (100..108).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (idx, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), 100 + idx as i32);
    }
}

#[test]
fn arena_alloc_after_reset_reuses_space() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i + 100));
    }
    // After reset, arena reuses existing chunks — no new chunks needed.
    assert_eq!(arena.num_chunks(), chunks_before);
}

// ───────────────────────────────────────────────────────────────
// 2. arena_get_* — get / get_mut operations (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_get_returns_correct_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(55));
    assert_eq!(arena.get(h).value(), 55);
}

#[test]
fn arena_get_ref_deref_to_tree_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(3));
    let node_ref = arena.get(h);
    // Deref should provide TreeNode methods directly
    assert!(node_ref.is_leaf());
    assert!(!node_ref.is_branch());
}

#[test]
fn arena_get_leaf_has_no_children() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn arena_get_branch_has_children() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let b = arena.alloc(TreeNode::branch(vec![c]));
    assert!(!arena.get(b).children().is_empty());
    assert_eq!(arena.get(b).children()[0], c);
}

#[test]
fn arena_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn arena_get_mut_does_not_affect_others() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    arena.get_mut(h1).set_value(99);
    assert_eq!(arena.get(h1).value(), 99);
    assert_eq!(arena.get(h2).value(), 2);
}

#[test]
fn arena_get_as_ref_returns_tree_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    let node_ref = arena.get(h);
    let inner: &TreeNode = node_ref.as_ref();
    assert_eq!(inner.value(), 7);
}

#[test]
fn arena_get_symbol_matches_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).symbol(), arena.get(h).value());
}

// ───────────────────────────────────────────────────────────────
// 3. arena_handle_* — handle validity / equality (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_handle_copy_semantics() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    let h_copy = h; // Copy, not move
    assert_eq!(arena.get(h).value(), arena.get(h_copy).value());
}

#[test]
fn arena_handle_equality() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(1)); // same value, different handle
    assert_ne!(h1, h2);
    assert_eq!(h1, h1);
}

#[test]
fn arena_handle_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));

    let mut hasher1 = DefaultHasher::new();
    h.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    h.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}

#[test]
fn arena_handle_in_hashset() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..5).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let set: HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(set.len(), 5);
    for &h in &handles {
        assert!(set.contains(&h));
    }
}

#[test]
fn arena_handle_debug_format() {
    let h = NodeHandle::new(0, 3);
    let dbg = format!("{:?}", h);
    assert!(!dbg.is_empty());
}

#[test]
fn arena_handle_clone_equals_original() {
    let h = NodeHandle::new(1, 5);
    let h2 = h.clone();
    assert_eq!(h, h2);
}

#[test]
fn arena_handle_new_manual() {
    let h = NodeHandle::new(0, 0);
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(77));
    assert_eq!(arena.get(h).value(), 77);
}

#[test]
fn arena_handle_used_as_child() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(5));
    let branch = arena.alloc(TreeNode::branch(vec![leaf]));
    assert_eq!(arena.get(branch).children()[0], leaf);
}

// ───────────────────────────────────────────────────────────────
// 4. arena_clear_* — clear / reset operations (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_clear_empties_arena() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_clear_retains_single_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    // Force multiple chunks
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn arena_clear_allows_reallocation() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h).value(), 2);
    assert_eq!(arena.len(), 1);
}

#[test]
fn arena_reset_preserves_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
    assert!(arena.is_empty());
}

#[test]
fn arena_reset_allows_reallocation() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(10));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(20));
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn arena_clear_then_alloc_len_correct() {
    let mut arena = TreeArena::new();
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.len(), 1);
}

#[test]
fn arena_clear_repeated() {
    let mut arena = TreeArena::new();
    for _ in 0..3 {
        for i in 0..5 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.clear();
    }
    assert!(arena.is_empty());
}

#[test]
fn arena_reset_repeated() {
    let mut arena = TreeArena::with_capacity(4);
    for _ in 0..3 {
        for i in 0..4 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.reset();
    }
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

// ───────────────────────────────────────────────────────────────
// 5. arena_capacity_* — capacity management (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_capacity_initial() {
    let arena = TreeArena::new();
    assert!(arena.capacity() >= 1024);
}

#[test]
fn arena_capacity_with_capacity() {
    let arena = TreeArena::with_capacity(16);
    assert_eq!(arena.capacity(), 16);
}

#[test]
fn arena_capacity_grows_on_overflow() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    let cap_before = arena.capacity();
    arena.alloc(TreeNode::leaf(3)); // triggers new chunk
    assert!(arena.capacity() > cap_before);
}

#[test]
fn arena_capacity_num_chunks_grows() {
    let mut arena = TreeArena::with_capacity(1);
    arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn arena_capacity_memory_usage_positive() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn arena_capacity_memory_grows_with_alloc() {
    let mut arena = TreeArena::with_capacity(2);
    let mem_before = arena.memory_usage();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3)); // new chunk
    assert!(arena.memory_usage() > mem_before);
}

#[test]
fn arena_capacity_metrics_snapshot() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let m = arena.metrics();
    assert_eq!(m.len(), 1);
    assert!(!m.is_empty());
    assert!(m.capacity() >= 1024);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn arena_capacity_default_matches_new() {
    let a1 = TreeArena::new();
    let a2 = TreeArena::default();
    assert_eq!(a1.capacity(), a2.capacity());
    assert_eq!(a1.num_chunks(), a2.num_chunks());
}

// ───────────────────────────────────────────────────────────────
// 6. arena_tree_* — tree building in arena (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_tree_single_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn arena_tree_parent_child_link() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(1));
    let parent = arena.alloc(TreeNode::branch(vec![child]));
    assert_eq!(arena.get(parent).children()[0], child);
}

#[test]
fn arena_tree_multi_child_branch() {
    let mut arena = TreeArena::new();
    let children: Vec<_> = (0..4).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(children.clone()));
    assert_eq!(arena.get(parent).children().len(), 4);
    for (i, &ch) in arena.get(parent).children().iter().enumerate() {
        assert_eq!(arena.get(ch).value(), i as i32);
    }
}

#[test]
fn arena_tree_nested_branches() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let mid = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let root = arena.alloc(TreeNode::branch(vec![mid]));
    assert!(arena.get(root).is_branch());
    assert!(arena.get(mid).is_branch());
    assert!(arena.get(l1).is_leaf());
}

#[test]
fn arena_tree_deep_nesting() {
    let mut arena = TreeArena::new();
    let root = build_chain(&mut arena, 20);
    assert_eq!(count_nodes(&arena, root), 20);
}

#[test]
fn arena_tree_wide_branch() {
    let mut arena = TreeArena::new();
    let leaves: Vec<_> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(leaves));
    assert_eq!(arena.get(root).children().len(), 100);
}

#[test]
fn arena_tree_branch_symbol_propagates() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(0));
    let b = arena.alloc(TreeNode::branch_with_symbol(77, vec![c]));
    assert_eq!(arena.get(b).symbol(), 77);
    assert_eq!(arena.get(b).value(), 77);
}

#[test]
fn arena_tree_mixed_leaves_and_branches() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let root = arena.alloc(TreeNode::branch(vec![b1, l2, l3]));
    assert_eq!(arena.get(root).children().len(), 3);
    assert!(arena.get(b1).is_branch());
    assert!(arena.get(l2).is_leaf());
}

// ───────────────────────────────────────────────────────────────
// 7. arena_walk_* — walking arena-based trees (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_walk_leaf_only() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let nodes = collect_handles(&arena, h);
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0], h);
}

#[test]
fn arena_walk_parent_child() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(10));
    let parent = arena.alloc(TreeNode::branch(vec![child]));
    let nodes = collect_handles(&arena, parent);
    assert_eq!(nodes.len(), 2);
}

#[test]
fn arena_walk_chain_depth() {
    let mut arena = TreeArena::new();
    let root = build_chain(&mut arena, 10);
    let nodes = collect_handles(&arena, root);
    assert_eq!(nodes.len(), 10);
}

#[test]
fn arena_walk_wide_tree() {
    let mut arena = TreeArena::new();
    let leaves: Vec<_> = (0..8).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(leaves));
    let nodes = collect_handles(&arena, root);
    assert_eq!(nodes.len(), 9); // 1 root + 8 leaves
}

#[test]
fn arena_walk_collects_all_values() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(10));
    let l2 = arena.alloc(TreeNode::leaf(20));
    let root = arena.alloc(TreeNode::branch_with_symbol(0, vec![l1, l2]));
    let handles = collect_handles(&arena, root);
    let values: Vec<i32> = handles.iter().map(|&h| arena.get(h).value()).collect();
    assert!(values.contains(&10));
    assert!(values.contains(&20));
    assert!(values.contains(&0));
}

#[test]
fn arena_walk_dfs_visits_root_first() {
    let mut arena = TreeArena::new();
    let child = arena.alloc(TreeNode::leaf(2));
    let root = arena.alloc(TreeNode::branch_with_symbol(1, vec![child]));
    let handles = collect_handles(&arena, root);
    assert_eq!(arena.get(handles[0]).value(), 1); // root first
}

#[test]
fn arena_walk_two_level_tree() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1]));
    let b2 = arena.alloc(TreeNode::branch(vec![l2]));
    let root = arena.alloc(TreeNode::branch(vec![b1, b2]));
    assert_eq!(count_nodes(&arena, root), 5);
}

#[test]
fn arena_walk_handles_are_unique() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let root = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let handles = collect_handles(&arena, root);
    let set: HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(set.len(), handles.len());
}

// ───────────────────────────────────────────────────────────────
// 8. arena_edge_* — edge cases (8 tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn arena_edge_empty_arena() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_edge_with_capacity_one() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).value(), 1);
    // Triggers new chunk
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
#[should_panic]
fn arena_edge_zero_capacity_panics() {
    let _ = TreeArena::with_capacity(0);
}

#[test]
fn arena_edge_large_allocation() {
    let mut arena = TreeArena::with_capacity(4);
    let mut handles = Vec::new();
    for i in 0..1000 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert_eq!(arena.len(), 1000);
    // Spot-check first, middle, last
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[499]).value(), 499);
    assert_eq!(arena.get(handles[999]).value(), 999);
}

#[test]
fn arena_edge_negative_symbol_values() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(-1));
    assert_eq!(arena.get(h).value(), -1);
    let h2 = arena.alloc(TreeNode::branch_with_symbol(-100, vec![h]));
    assert_eq!(arena.get(h2).symbol(), -100);
}

#[test]
fn arena_edge_zero_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h).value(), 0);
}

#[test]
fn arena_edge_max_i32_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn arena_edge_min_i32_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}
