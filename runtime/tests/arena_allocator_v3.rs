//! Arena allocator v3 tests — allocation, retrieval, capacity, independence,
//! node construction, handle identity, stress, and edge cases.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use adze::error_recovery::{ErrorNode, ErrorRecoveryConfig, ErrorRecoveryState, RecoveryStrategy};

// ───────────────────────────────────────────────────────────────
// 1. Arena allocation — alloc returns valid handles
// ───────────────────────────────────────────────────────────────

#[test]
fn test_alloc_leaf_returns_handle() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).value(), 1);
}

#[test]
fn test_alloc_branch_returns_handle() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(10));
    let h = arena.alloc(TreeNode::branch(vec![c]));
    assert!(arena.get(h).is_branch());
}

#[test]
fn test_alloc_branch_with_symbol_returns_handle() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(5));
    let h = arena.alloc(TreeNode::branch_with_symbol(99, vec![c]));
    assert_eq!(arena.get(h).symbol(), 99);
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
fn test_alloc_with_capacity() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..8 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 8);
}

#[test]
fn test_alloc_empty_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).children().is_empty());
}

// ───────────────────────────────────────────────────────────────
// 2. Arena retrieval — get returns allocated nodes
// ───────────────────────────────────────────────────────────────

#[test]
fn test_get_leaf_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn test_get_branch_children() {
    let mut arena = TreeArena::new();
    let a = arena.alloc(TreeNode::leaf(1));
    let b = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![a, b]));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], a);
    assert_eq!(children[1], b);
}

#[test]
fn test_get_returns_correct_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(77, vec![]));
    assert_eq!(arena.get(h).symbol(), 77);
}

#[test]
fn test_get_via_deref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(3));
    let node_ref = arena.get(h);
    // TreeNodeRef derefs to TreeNode
    assert!(node_ref.is_leaf());
    assert_eq!(node_ref.symbol(), 3);
}

#[test]
fn test_get_ref_method() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(9));
    let node_ref = arena.get(h);
    let inner: &TreeNode = node_ref.get_ref();
    assert_eq!(inner.value(), 9);
}

#[test]
fn test_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    arena.get_mut(h).set_value(100);
    assert_eq!(arena.get(h).value(), 100);
}

#[test]
fn test_get_multiple_nodes_after_alloc() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..10).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

// ───────────────────────────────────────────────────────────────
// 3. Arena capacity — handles sequential allocation
// ───────────────────────────────────────────────────────────────

#[test]
fn test_capacity_at_creation() {
    let arena = TreeArena::with_capacity(64);
    assert!(arena.capacity() >= 64);
}

#[test]
fn test_default_capacity() {
    let arena = TreeArena::new();
    assert!(arena.capacity() >= 1024);
}

#[test]
fn test_capacity_grows_on_overflow() {
    let mut arena = TreeArena::with_capacity(4);
    let initial_cap = arena.capacity();
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.capacity() > initial_cap);
}

#[test]
fn test_num_chunks_grows() {
    let mut arena = TreeArena::with_capacity(2);
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    // Filling first chunk
    arena.alloc(TreeNode::leaf(3));
    assert!(arena.num_chunks() >= 2);
}

#[test]
fn test_memory_usage_increases() {
    let mut arena = TreeArena::new();
    let usage_before = arena.memory_usage();
    for i in 0..2048 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.memory_usage() >= usage_before);
}

#[test]
fn test_is_empty_on_new_arena() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
}

#[test]
fn test_not_empty_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    assert!(!arena.is_empty());
}

// ───────────────────────────────────────────────────────────────
// 4. Arena independence — separate arenas don't interfere
// ───────────────────────────────────────────────────────────────

#[test]
fn test_two_arenas_independent_alloc() {
    let mut a1 = TreeArena::new();
    let mut a2 = TreeArena::new();
    let h1 = a1.alloc(TreeNode::leaf(111));
    let h2 = a2.alloc(TreeNode::leaf(222));
    assert_eq!(a1.get(h1).value(), 111);
    assert_eq!(a2.get(h2).value(), 222);
}

#[test]
fn test_two_arenas_independent_len() {
    let mut a1 = TreeArena::new();
    let mut a2 = TreeArena::new();
    a1.alloc(TreeNode::leaf(1));
    a1.alloc(TreeNode::leaf(2));
    a2.alloc(TreeNode::leaf(3));
    assert_eq!(a1.len(), 2);
    assert_eq!(a2.len(), 1);
}

#[test]
fn test_reset_one_arena_does_not_affect_other() {
    let mut a1 = TreeArena::new();
    let mut a2 = TreeArena::new();
    a1.alloc(TreeNode::leaf(10));
    let h2 = a2.alloc(TreeNode::leaf(20));
    a1.reset();
    assert!(a1.is_empty());
    assert_eq!(a2.get(h2).value(), 20);
}

#[test]
fn test_clear_one_arena_does_not_affect_other() {
    let mut a1 = TreeArena::new();
    let mut a2 = TreeArena::new();
    a1.alloc(TreeNode::leaf(10));
    let h2 = a2.alloc(TreeNode::leaf(20));
    a1.clear();
    assert!(a1.is_empty());
    assert_eq!(a2.get(h2).value(), 20);
}

#[test]
fn test_independent_arenas_different_capacities() {
    let a1 = TreeArena::with_capacity(8);
    let a2 = TreeArena::with_capacity(256);
    assert!(a1.capacity() < a2.capacity());
}

// ───────────────────────────────────────────────────────────────
// 5. Node construction — TreeNode construction variations
// ───────────────────────────────────────────────────────────────

#[test]
fn test_leaf_is_leaf() {
    let node = TreeNode::leaf(5);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
}

#[test]
fn test_branch_is_branch() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
}

#[test]
fn test_leaf_symbol() {
    let node = TreeNode::leaf(42);
    assert_eq!(node.symbol(), 42);
    assert_eq!(node.value(), 42);
}

#[test]
fn test_branch_default_symbol_is_zero() {
    let node = TreeNode::branch(vec![]);
    assert_eq!(node.symbol(), 0);
}

#[test]
fn test_branch_with_symbol_value() {
    let node = TreeNode::branch_with_symbol(55, vec![]);
    assert_eq!(node.symbol(), 55);
    assert_eq!(node.value(), 55);
}

#[test]
fn test_leaf_children_empty() {
    let node = TreeNode::leaf(1);
    assert!(node.children().is_empty());
}

#[test]
fn test_branch_children_count() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let node = TreeNode::branch(vec![h1, h2]);
    assert_eq!(node.children().len(), 2);
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
fn test_tree_node_clone() {
    let node = TreeNode::leaf(7);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

#[test]
fn test_tree_node_debug() {
    let node = TreeNode::leaf(1);
    let dbg = format!("{node:?}");
    assert!(!dbg.is_empty());
}

// ───────────────────────────────────────────────────────────────
// 6. Handle identity — handles are unique per allocation
// ───────────────────────────────────────────────────────────────

#[test]
fn test_consecutive_handles_differ() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn test_handle_copy_semantics() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = h1; // Copy, not move
    assert_eq!(h1, h2);
    assert_eq!(arena.get(h1).value(), arena.get(h2).value());
}

#[test]
fn test_handle_eq_reflexive() {
    let h = NodeHandle::new(0, 0);
    assert_eq!(h, h);
}

#[test]
fn test_handle_eq_symmetric() {
    let a = NodeHandle::new(1, 2);
    let b = NodeHandle::new(1, 2);
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn test_handle_ne_different_chunk() {
    let a = NodeHandle::new(0, 0);
    let b = NodeHandle::new(1, 0);
    assert_ne!(a, b);
}

#[test]
fn test_handle_ne_different_node() {
    let a = NodeHandle::new(0, 0);
    let b = NodeHandle::new(0, 1);
    assert_ne!(a, b);
}

#[test]
fn test_handle_hash_consistent() {
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

#[test]
fn test_many_handles_all_unique() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..200).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let unique: std::collections::HashSet<NodeHandle> = handles.iter().copied().collect();
    assert_eq!(unique.len(), 200);
}

#[test]
fn test_handle_debug_format() {
    let h = NodeHandle::new(3, 7);
    let dbg = format!("{h:?}");
    assert!(!dbg.is_empty());
}

// ───────────────────────────────────────────────────────────────
// 7. Stress testing — many allocations, large trees
// ───────────────────────────────────────────────────────────────

#[test]
fn test_stress_alloc_10k_leaves() {
    let mut arena = TreeArena::new();
    let handles: Vec<NodeHandle> = (0..10_000)
        .map(|i| arena.alloc(TreeNode::leaf(i)))
        .collect();
    assert_eq!(arena.len(), 10_000);
    // Spot-check first, middle, last
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[5000]).value(), 5000);
    assert_eq!(arena.get(handles[9999]).value(), 9999);
}

#[test]
fn test_stress_deep_tree() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for i in 1..500 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.get(current).symbol(), 499);
    assert!(arena.get(current).is_branch());
}

#[test]
fn test_stress_wide_tree() {
    let mut arena = TreeArena::new();
    let children: Vec<NodeHandle> = (0..1000).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(root).children().len(), 1000);
}

#[test]
fn test_stress_reset_and_realloc() {
    let mut arena = TreeArena::with_capacity(16);
    for _ in 0..5 {
        for i in 0..100 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.reset();
        assert!(arena.is_empty());
    }
    // After final reset, can still allocate
    let h = arena.alloc(TreeNode::leaf(999));
    assert_eq!(arena.get(h).value(), 999);
}

#[test]
fn test_stress_mixed_leaf_and_branch() {
    let mut arena = TreeArena::new();
    let mut handles = Vec::new();
    for i in 0..500 {
        if i % 3 == 0 {
            let leaf = arena.alloc(TreeNode::leaf(i));
            handles.push(leaf);
        } else {
            let children: Vec<NodeHandle> = handles.iter().rev().take(2).copied().collect();
            let branch = arena.alloc(TreeNode::branch_with_symbol(i, children));
            handles.push(branch);
        }
    }
    assert_eq!(arena.len(), 500);
}

#[test]
fn test_stress_alloc_across_chunk_boundaries() {
    let mut arena = TreeArena::with_capacity(4);
    let mut handles = Vec::new();
    for i in 0..20 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert!(arena.num_chunks() > 1);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

#[test]
fn test_stress_metrics_after_many_allocs() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.len(), 100);
    assert!(m.capacity() >= 100);
    assert!(m.num_chunks() > 1);
    assert!(m.memory_usage() > 0);
}

// ───────────────────────────────────────────────────────────────
// 8. Edge cases — empty arena operations
// ───────────────────────────────────────────────────────────────

#[test]
fn test_new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_reset_empty_arena() {
    let mut arena = TreeArena::new();
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn test_clear_empty_arena() {
    let mut arena = TreeArena::new();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_metrics_empty_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
    assert!(m.capacity() > 0);
    assert_eq!(m.num_chunks(), 1);
}

#[test]
fn test_default_trait() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
}

#[test]
#[should_panic]
fn test_with_capacity_zero_panics() {
    let _ = TreeArena::with_capacity(0);
}

#[test]
fn test_reset_then_alloc() {
    let mut arena = TreeArena::new();
    let _ = arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h).value(), 2);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_clear_then_alloc() {
    let mut arena = TreeArena::new();
    let _ = arena.alloc(TreeNode::leaf(1));
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h).value(), 2);
    assert_eq!(arena.len(), 1);
}

#[test]
fn test_clear_drops_excess_chunks() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_arena_metrics_snapshot_is_copy() {
    let arena = TreeArena::new();
    let m1 = arena.metrics();
    let m2 = m1; // Copy
    assert_eq!(m1, m2);
}

// ───────────────────────────────────────────────────────────────
// Additional coverage: error recovery types (smoke tests)
// ───────────────────────────────────────────────────────────────

#[test]
fn test_error_recovery_config_defaults() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
    assert_eq!(cfg.max_token_deletions, 3);
    assert_eq!(cfg.max_token_insertions, 2);
    assert_eq!(cfg.max_consecutive_errors, 10);
    assert!(cfg.enable_phrase_recovery);
    assert!(cfg.enable_scope_recovery);
    assert!(!cfg.enable_indentation_recovery);
}

#[test]
fn test_error_recovery_state_new() {
    let cfg = ErrorRecoveryConfig::default();
    let state = ErrorRecoveryState::new(cfg);
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn test_error_node_construction() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![1, 2],
        actual: Some(3),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![3],
    };
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 5);
    assert_eq!(node.expected.len(), 2);
    assert_eq!(node.actual, Some(3));
}

#[test]
fn test_recovery_strategy_equality() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
    assert_ne!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn test_error_node_clone() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 20,
        start_position: (1, 0),
        end_position: (1, 10),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![],
    };
    let cloned = node.clone();
    assert_eq!(cloned.start_byte, 10);
    assert_eq!(cloned.recovery, RecoveryStrategy::TokenDeletion);
}
