//! Property-based tests (v4) for TreeArena.
//!
//! 55 tests covering allocation invariants, handle uniqueness, node preservation,
//! capacity bounds, reset/clear semantics, metrics consistency, and edge cases.

use adze::arena_allocator::{ArenaMetrics, NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Helpers
// ============================================================================

fn arb_symbol() -> impl Strategy<Value = i32> {
    prop::num::i32::ANY
}

fn arb_capacity() -> impl Strategy<Value = usize> {
    1usize..=512
}

fn arb_alloc_count() -> impl Strategy<Value = usize> {
    1usize..=200
}

// ============================================================================
// 1. Property: arena len equals number of allocs
// ============================================================================

proptest! {
    #[test]
    fn prop_len_equals_alloc_count(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn prop_len_equals_alloc_count_with_capacity(cap in arb_capacity(), n in arb_alloc_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn prop_len_zero_after_reset(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn prop_len_zero_after_clear(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn prop_len_increments_one_at_a_time(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            prop_assert_eq!(arena.len(), i);
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.len(), n);
    }
}

// ============================================================================
// 2. Property: allocated nodes are retrievable
// ============================================================================

proptest! {
    #[test]
    fn prop_alloc_then_get_leaf(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn prop_all_nodes_retrievable(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn prop_branch_children_retrievable(n in 1usize..=50) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        prop_assert_eq!(arena.get(parent).children().len(), n);
        for (i, ch) in arena.get(parent).children().iter().enumerate() {
            prop_assert_eq!(arena.get(*ch).value(), i as i32);
        }
    }

    #[test]
    fn prop_get_mut_then_get(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        arena.get_mut(h).set_value(sym.wrapping_add(1));
        prop_assert_eq!(arena.get(h).value(), sym.wrapping_add(1));
    }

    #[test]
    fn prop_nodes_survive_subsequent_allocs(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let first = arena.alloc(TreeNode::leaf(-1));
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.get(first).value(), -1);
    }
}

// ============================================================================
// 3. Property: arena is never empty after alloc
// ============================================================================

proptest! {
    #[test]
    fn prop_not_empty_after_alloc(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(!arena.is_empty());
        }
    }

    #[test]
    fn prop_empty_before_first_alloc(cap in arb_capacity()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn prop_empty_after_reset(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn prop_empty_after_clear(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn prop_is_empty_iff_len_zero(n in 0usize..=100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.is_empty(), arena.len() == 0);
    }
}

// ============================================================================
// 4. Property: capacity >= len
// ============================================================================

proptest! {
    #[test]
    fn prop_capacity_ge_len(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(arena.capacity() >= arena.len());
        }
    }

    #[test]
    fn prop_capacity_ge_len_custom_cap(cap in arb_capacity(), n in arb_alloc_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    #[test]
    fn prop_initial_capacity_matches(cap in arb_capacity()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert_eq!(arena.capacity(), cap);
    }

    #[test]
    fn prop_capacity_never_shrinks_on_alloc(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let mut prev_cap = arena.capacity();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cur = arena.capacity();
            prop_assert!(cur >= prev_cap);
            prev_cap = cur;
        }
    }

    #[test]
    fn prop_metrics_capacity_matches_method(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.capacity(), arena.capacity());
        prop_assert_eq!(m.len(), arena.len());
    }
}

// ============================================================================
// 5. Property: alloc returns unique handles
// ============================================================================

proptest! {
    #[test]
    fn prop_handles_unique(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let mut seen = HashSet::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(seen.insert(h), "duplicate handle at index {}", i);
        }
    }

    #[test]
    fn prop_handles_unique_across_chunks(n in 1usize..=200) {
        let mut arena = TreeArena::with_capacity(4);
        let mut seen = HashSet::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(seen.insert(h));
        }
        prop_assert_eq!(seen.len(), n);
    }

    #[test]
    fn prop_handles_deterministic(n in arb_alloc_count()) {
        let mut a1 = TreeArena::new();
        let mut a2 = TreeArena::new();
        for i in 0..n {
            let h1 = a1.alloc(TreeNode::leaf(i as i32));
            let h2 = a2.alloc(TreeNode::leaf(i as i32));
            prop_assert_eq!(h1, h2);
        }
    }

    #[test]
    fn prop_handle_is_copy(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        let h2 = h;
        prop_assert_eq!(arena.get(h).value(), arena.get(h2).value());
    }
}

// ============================================================================
// 6. Property: node kind is preserved
// ============================================================================

proptest! {
    #[test]
    fn prop_leaf_preserved(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        let node = arena.get(h);
        prop_assert!(node.is_leaf());
        prop_assert!(!node.is_branch());
        prop_assert_eq!(node.value(), sym);
        prop_assert_eq!(node.symbol(), sym);
    }

    #[test]
    fn prop_branch_preserved(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        let node = arena.get(h);
        prop_assert!(node.is_branch());
        prop_assert!(!node.is_leaf());
        prop_assert_eq!(node.symbol(), sym);
    }

    #[test]
    fn prop_leaf_children_always_empty(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).children().is_empty());
    }

    #[test]
    fn prop_branch_default_symbol_zero(n in 0usize..=20) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let h = arena.alloc(TreeNode::branch(children));
        prop_assert_eq!(arena.get(h).symbol(), 0);
    }

    #[test]
    fn prop_symbol_equals_value(sym in arb_symbol()) {
        let n = TreeNode::leaf(sym);
        prop_assert_eq!(n.symbol(), n.value());
    }

    #[test]
    fn prop_branch_children_count(n in 0usize..=50) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let h = arena.alloc(TreeNode::branch(children));
        prop_assert_eq!(arena.get(h).children().len(), n);
    }
}

// ============================================================================
// 7. Property: node byte range / symbol values preserved across operations
// ============================================================================

proptest! {
    #[test]
    fn prop_many_leaves_values_preserved(vals in prop::collection::vec(arb_symbol(), 1..100)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    #[test]
    fn prop_set_value_leaf(original in arb_symbol(), replacement in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(original));
        arena.get_mut(h).set_value(replacement);
        prop_assert_eq!(arena.get(h).value(), replacement);
    }

    #[test]
    fn prop_clone_node_equality(sym in arb_symbol()) {
        let n = TreeNode::leaf(sym);
        let c = n.clone();
        prop_assert_eq!(n, c);
    }

    #[test]
    fn prop_branch_with_symbol_preserved(sym in arb_symbol(), n_children in 0usize..=10) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n_children).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, children.clone()));
        let node = arena.get(h);
        prop_assert_eq!(node.symbol(), sym);
        prop_assert_eq!(node.children().len(), n_children);
        for (i, ch) in node.children().iter().enumerate() {
            prop_assert_eq!(arena.get(*ch).value(), i as i32);
        }
    }
}

// ============================================================================
// 8. Metrics consistency
// ============================================================================

proptest! {
    #[test]
    fn prop_metrics_len_matches(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.metrics().len(), arena.len());
    }

    #[test]
    fn prop_metrics_is_empty_matches(n in 0usize..=50) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.metrics().is_empty(), arena.is_empty());
    }

    #[test]
    fn prop_metrics_num_chunks(cap in 1usize..=8, n in 1usize..=100) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.metrics().num_chunks(), arena.num_chunks());
        prop_assert!(arena.num_chunks() >= 1);
    }

    #[test]
    fn prop_memory_usage_positive_when_nonempty(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.memory_usage() > 0);
        prop_assert_eq!(arena.metrics().memory_usage(), arena.memory_usage());
    }
}

// ============================================================================
// 9. Reset / clear properties
// ============================================================================

proptest! {
    #[test]
    fn prop_alloc_after_reset(n in arb_alloc_count(), sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
        prop_assert_eq!(arena.len(), 1);
    }

    #[test]
    fn prop_alloc_after_clear(n in arb_alloc_count(), sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
        prop_assert_eq!(arena.len(), 1);
    }

    #[test]
    fn prop_reset_preserves_capacity(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let cap_before = arena.capacity();
        arena.reset();
        prop_assert_eq!(arena.capacity(), cap_before);
    }

    #[test]
    fn prop_clear_reduces_to_one_chunk(n in arb_alloc_count()) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
    }
}

// ============================================================================
// 10. Edge cases — unit tests
// ============================================================================

#[test]
fn edge_single_alloc() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn edge_capacity_one() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.len(), 2);
    assert!(arena.num_chunks() >= 2);
}

#[test]
#[should_panic]
fn edge_with_capacity_zero_panics() {
    TreeArena::with_capacity(0);
}

#[test]
fn edge_default_is_new() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.len(), b.len());
}

#[test]
fn edge_i32_min_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

#[test]
fn edge_i32_max_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn edge_empty_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
    assert_eq!(arena.get(h).symbol(), 0);
}

#[test]
fn edge_nested_branch() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(99));
    let mid = arena.alloc(TreeNode::branch(vec![leaf]));
    let root = arena.alloc(TreeNode::branch(vec![mid]));
    assert_eq!(arena.get(root).children().len(), 1);
    let mid_h = arena.get(root).children()[0];
    assert_eq!(arena.get(mid_h).children().len(), 1);
    let leaf_h = arena.get(mid_h).children()[0];
    assert_eq!(arena.get(leaf_h).value(), 99);
}

#[test]
fn edge_multiple_reset_cycles() {
    let mut arena = TreeArena::new();
    for _cycle in 0..5 {
        for i in 0..10 {
            arena.alloc(TreeNode::leaf(i));
        }
        assert_eq!(arena.len(), 10);
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn edge_chunk_growth_small_capacity() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 20);
    assert!(arena.num_chunks() > 1);
}

#[test]
fn edge_handle_equality() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(h1, h1);
    assert_ne!(h1, h2);
}

#[test]
fn edge_handle_hash_usable() {
    let mut arena = TreeArena::new();
    let mut set = HashSet::new();
    let h = arena.alloc(TreeNode::leaf(0));
    set.insert(h);
    assert!(set.contains(&h));
}

#[test]
fn edge_metrics_fresh_arena() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert!(m.capacity() > 0);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn edge_set_value_preserves_leaf_kind() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert!(arena.get(h).is_leaf());
    assert_eq!(arena.get(h).value(), 20);
}
