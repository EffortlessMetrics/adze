//! Property-based tests (v5) for `TreeArena`.
//!
//! 46 proptest properties organized into 9 categories:
//! 1. Arena len equals number of allocations (5)
//! 2. All allocated handles are retrievable (5)
//! 3. Node properties preserved after allocation (5)
//! 4. Arena is_empty iff len == 0 (5)
//! 5. Capacity >= len always (5)
//! 6. Reset makes arena empty (5)
//! 7. NodeHandle equality and uniqueness (5)
//! 8. Large allocation sequences (5)
//! 9. Edge cases (6)

use adze::arena_allocator::{TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Strategies
// ============================================================================

fn symbol_val() -> impl Strategy<Value = i32> {
    prop::num::i32::ANY
}

fn alloc_count() -> impl Strategy<Value = usize> {
    1_usize..300
}

fn small_count() -> impl Strategy<Value = usize> {
    1_usize..50
}

// ============================================================================
// 1. Arena len equals number of allocations (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_len_equals_leaf_count(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_len_equals_branch_count(n in 1_usize..100) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::branch(vec![]));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_len_equals_mixed_count(
        leaves in 0_usize..150,
        branches in 0_usize..150,
    ) {
        prop_assume!(leaves + branches > 0);
        let mut arena = TreeArena::new();
        for i in 0..leaves {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        for _ in 0..branches {
            arena.alloc(TreeNode::branch(vec![]));
        }
        prop_assert_eq!(arena.len(), leaves + branches);
    }

    #[test]
    fn v5_len_increments_by_one(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for expected in 1..=n {
            arena.alloc(TreeNode::leaf(0));
            prop_assert_eq!(arena.len(), expected);
        }
    }

    #[test]
    fn v5_len_correct_across_chunks(cap in 1_usize..8, n in 10_usize..100) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.len(), n);
    }
}

// ============================================================================
// 2. All allocated handles are retrievable (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_leaf_handles_retrievable(vals in prop::collection::vec(symbol_val(), 1..200)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    #[test]
    fn v5_branch_handles_retrievable(n in 1_usize..100) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![])))
            .collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).symbol(), i as i32);
        }
    }

    #[test]
    fn v5_handles_survive_chunk_growth(cap in 1_usize..8, n in 20_usize..100) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_handles_random_access_order(
        vals in prop::collection::vec(symbol_val(), 2..200),
        seed in prop::num::u64::ANY,
    ) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        let n = handles.len();
        for step in 0..n {
            let idx = (seed as usize).wrapping_add(step.wrapping_mul(7)) % n;
            prop_assert_eq!(arena.get(handles[idx]).value(), vals[idx]);
        }
    }

    #[test]
    fn v5_handles_valid_after_interleaved_alloc(n in 1_usize..100) {
        let mut arena = TreeArena::new();
        let mut handles = Vec::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            handles.push(h);
            prop_assert_eq!(arena.get(h).value(), i as i32);
        }
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }
}

// ============================================================================
// 3. Node properties preserved after allocation (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_leaf_value_preserved(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    #[test]
    fn v5_branch_symbol_preserved(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    #[test]
    fn v5_leaf_kind_flags(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }

    #[test]
    fn v5_branch_kind_flags(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }

    #[test]
    fn v5_branch_children_preserved(n in 1_usize..20) {
        let mut arena = TreeArena::new();
        let child_handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let expected = child_handles.to_vec();
        let parent = arena.alloc(TreeNode::branch(child_handles));
        let node_ref = arena.get(parent);
        let read = node_ref.children();
        prop_assert_eq!(read.len(), n);
        for (i, &ch) in read.iter().enumerate() {
            prop_assert_eq!(ch, expected[i]);
        }
    }
}

// ============================================================================
// 4. Arena is_empty iff len == 0 (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_new_arena_is_empty(cap in 1_usize..1000) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_after_alloc_not_empty(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        arena.alloc(TreeNode::leaf(sym));
        prop_assert!(!arena.is_empty());
    }

    #[test]
    fn v5_is_empty_iff_len_zero(n in 0_usize..100) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.is_empty(), n == 0);
    }

    #[test]
    fn v5_after_reset_is_empty(n in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_after_clear_is_empty(n in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }
}

// ============================================================================
// 5. Capacity >= len always (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_capacity_ge_len_after_allocs(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    #[test]
    fn v5_capacity_ge_len_small_cap(cap in 1_usize..8, n in 1_usize..100) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    #[test]
    fn v5_capacity_ge_len_every_step(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
            prop_assert!(arena.capacity() >= arena.len());
        }
    }

    #[test]
    fn v5_metrics_capacity_ge_metrics_len(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        let m = arena.metrics();
        prop_assert!(m.capacity() >= m.len());
    }

    #[test]
    fn v5_capacity_ge_len_after_reset_realloc(
        first in 1_usize..100,
        second in 1_usize..100,
    ) {
        let mut arena = TreeArena::new();
        for _ in 0..first {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        for _ in 0..second {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }
}

// ============================================================================
// 6. Reset makes arena empty (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_reset_len_zero(n in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_reset_is_empty(n in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn v5_clear_len_zero(n in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_multiple_resets(cycles in 1_usize..5, per_cycle in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..cycles {
            for _ in 0..per_cycle {
                arena.alloc(TreeNode::leaf(0));
            }
            arena.reset();
            prop_assert_eq!(arena.len(), 0);
            prop_assert!(arena.is_empty());
        }
    }

    #[test]
    fn v5_reset_then_reuse(
        first in alloc_count(),
        second_vals in prop::collection::vec(symbol_val(), 1..100),
    ) {
        let mut arena = TreeArena::new();
        for _ in 0..first {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        let handles: Vec<_> = second_vals
            .iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        prop_assert_eq!(arena.len(), second_vals.len());
        for (h, &v) in handles.iter().zip(second_vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }
}

// ============================================================================
// 7. NodeHandle equality and uniqueness (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_all_handles_distinct(n in alloc_count()) {
        let mut arena = TreeArena::new();
        let mut set = HashSet::new();
        for _ in 0..n {
            let h = arena.alloc(TreeNode::leaf(0));
            prop_assert!(set.insert(h));
        }
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn v5_handles_usable_as_map_keys(n in small_count()) {
        let mut arena = TreeArena::new();
        let mut map = HashMap::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            map.insert(h, i);
        }
        prop_assert_eq!(map.len(), n);
    }

    #[test]
    fn v5_handle_copy_semantics(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(sym));
        let h2 = h1; // Copy, not clone
        prop_assert_eq!(h1, h2);
        prop_assert_eq!(arena.get(h1).value(), arena.get(h2).value());
    }

    #[test]
    fn v5_consecutive_handles_differ(n in 2_usize..200) {
        let mut arena = TreeArena::new();
        let mut prev = arena.alloc(TreeNode::leaf(0));
        for _ in 1..n {
            let cur = arena.alloc(TreeNode::leaf(0));
            prop_assert_ne!(prev, cur);
            prev = cur;
        }
    }

    #[test]
    fn v5_handle_set_size_equals_n(n in alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|_| arena.alloc(TreeNode::leaf(0)))
            .collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }
}

// ============================================================================
// 8. Large allocation sequences (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn v5_large_alloc_len(n in 800_usize..1200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_large_alloc_all_unique(n in 500_usize..1000) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|_| arena.alloc(TreeNode::leaf(0)))
            .collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn v5_large_alloc_all_retrievable(n in 500_usize..1000) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_large_alloc_metrics_consistent(n in 500_usize..1000) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), n);
        prop_assert!(m.capacity() >= n);
        prop_assert!(m.num_chunks() >= 1);
        prop_assert!(m.memory_usage() > 0);
    }

    #[test]
    fn v5_large_alloc_capacity_ge_len(n in 500_usize..1000) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
            prop_assert!(arena.capacity() >= arena.len());
        }
    }
}

// ============================================================================
// 9. Edge cases (6 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_edge_capacity_one(n in 1_usize..50) {
        let mut arena = TreeArena::with_capacity(1);
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        prop_assert_eq!(arena.len(), n);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_edge_extreme_symbols(sym in prop::sample::select(
        vec![i32::MIN, i32::MAX, 0, 1, -1],
    )) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn v5_edge_deep_chain(depth in 1_usize..30) {
        let mut arena = TreeArena::new();
        let mut current = arena.alloc(TreeNode::leaf(0));
        for d in 1..depth {
            current = arena.alloc(TreeNode::branch_with_symbol(d as i32, vec![current]));
        }
        prop_assert_eq!(arena.len(), depth);
        prop_assert_eq!(arena.get(current).symbol(), (depth - 1) as i32);
    }

    #[test]
    fn v5_edge_wide_branch(width in 1_usize..200) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..width)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let parent = arena.alloc(TreeNode::branch(children));
        let node_ref = arena.get(parent);
        let read = node_ref.children();
        prop_assert_eq!(read.len(), width);
    }

    #[test]
    fn v5_edge_set_value_preserves_kind(
        original in symbol_val(),
        replacement in symbol_val(),
    ) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(original));
        arena.get_mut(h).set_value(replacement);
        prop_assert!(arena.get(h).is_leaf());
        prop_assert_eq!(arena.get(h).value(), replacement);
    }

    #[test]
    fn v5_edge_leaf_children_empty(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).children().is_empty());
    }
}
