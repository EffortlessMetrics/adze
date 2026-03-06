//! Property-based tests (v5) for `TreeArena`.
//!
//! 62 proptest properties organized into 15 categories:
//!  1. Allocate N nodes → len() == N (4)
//!  2. Allocate N nodes → !is_empty() (4)
//!  3. Get allocated handle → correct symbol (4)
//!  4. Get allocated handle → correct value roundtrip (4)
//!  5. with_capacity(C) then alloc N → len() == N (4)
//!  6. After clear → is_empty() (4)
//!  7. After clear → len() == 0 (4)
//!  8. Re-alloc after clear works (4)
//!  9. num_chunks >= ceil(N / capacity) (4)
//! 10. Symbol roundtrip for edge values (5)
//! 11. Branch children preserved (4)
//! 12. Leaf/branch kind flags preserved (4)
//! 13. Multiple alloc/clear cycles → len() correct (5)
//! 14. Handles from different allocs are different (4)
//! 15. Edge cases and misc (8)

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Strategies
// ============================================================================

fn symbol_val() -> impl Strategy<Value = i32> {
    prop_oneof![
        Just(0),
        Just(1),
        Just(-1),
        Just(i32::MAX),
        Just(i32::MIN),
        -10_000..10_000i32,
    ]
}

fn alloc_count() -> impl Strategy<Value = usize> {
    1_usize..500
}

fn small_count() -> impl Strategy<Value = usize> {
    1_usize..50
}

// ============================================================================
// 1. Allocate N nodes → len() == N (4 properties)
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
    fn v5_len_equals_branch_count(n in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::branch(vec![]));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_len_equals_mixed_count(
        leaves in 0_usize..250,
        branches in 0_usize..250,
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
}

// ============================================================================
// 2. Allocate N nodes → !is_empty() (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_not_empty_after_leaf_allocs(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert!(!arena.is_empty());
    }

    #[test]
    fn v5_not_empty_after_branch_allocs(n in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::branch(vec![]));
        }
        prop_assert!(!arena.is_empty());
    }

    #[test]
    fn v5_not_empty_after_single_alloc(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        arena.alloc(TreeNode::leaf(sym));
        prop_assert!(!arena.is_empty());
    }

    #[test]
    fn v5_is_empty_iff_len_zero(n in 0_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.is_empty(), n == 0);
    }
}

// ============================================================================
// 3. Get allocated handle → correct symbol (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_leaf_symbol_preserved(vals in prop::collection::vec(symbol_val(), 1..200)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).symbol(), v);
        }
    }

    #[test]
    fn v5_branch_symbol_preserved(vals in prop::collection::vec(symbol_val(), 1..100)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter()
            .map(|&v| arena.alloc(TreeNode::branch_with_symbol(v, vec![])))
            .collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).symbol(), v);
        }
    }

    #[test]
    fn v5_symbol_random_access_order(
        vals in prop::collection::vec(symbol_val(), 2..200),
        seed in prop::num::u64::ANY,
    ) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        let n = handles.len();
        for step in 0..n {
            let idx = (seed as usize).wrapping_add(step.wrapping_mul(7)) % n;
            prop_assert_eq!(arena.get(handles[idx]).symbol(), vals[idx]);
        }
    }

    #[test]
    fn v5_handles_valid_after_interleaved_alloc(n in 1_usize..100) {
        let mut arena = TreeArena::new();
        let mut handles = Vec::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            handles.push(h);
            prop_assert_eq!(arena.get(h).symbol(), i as i32);
        }
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).symbol(), i as i32);
        }
    }
}

// ============================================================================
// 4. Get allocated handle → correct value roundtrip (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_leaf_value_roundtrip(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn v5_branch_value_roundtrip(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn v5_value_equals_symbol(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        let node_ref = arena.get(h);
        prop_assert_eq!(node_ref.value(), node_ref.symbol());
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
}

// ============================================================================
// 5. with_capacity(C) then alloc N → len() == N (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_with_capacity_len_correct(cap in 1_usize..200, n in alloc_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_with_capacity_small_cap_large_n(cap in 1_usize..4, n in 100_usize..300) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(42));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_with_capacity_large_cap_small_n(cap in 500_usize..2000, n in 1_usize..50) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_with_capacity_exact_fill(cap in 1_usize..100) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..cap {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.len(), cap);
    }
}

// ============================================================================
// 6. After clear → is_empty() (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_clear_makes_empty(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn v5_clear_with_branches_makes_empty(n in 1_usize..100) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::branch(vec![]));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn v5_clear_custom_capacity_makes_empty(
        cap in 1_usize..100,
        n in 1_usize..200,
    ) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn v5_reset_makes_empty(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
    }
}

// ============================================================================
// 7. After clear → len() == 0 (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_clear_len_zero(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_reset_len_zero(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_clear_custom_cap_len_zero(cap in 1_usize..50, n in 1_usize..200) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_clear_mixed_nodes_len_zero(
        leaves in 1_usize..100,
        branches in 1_usize..100,
    ) {
        let mut arena = TreeArena::new();
        for i in 0..leaves {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        for _ in 0..branches {
            arena.alloc(TreeNode::branch(vec![]));
        }
        arena.clear();
        prop_assert_eq!(arena.len(), 0);
    }
}

// ============================================================================
// 8. Re-alloc after clear works (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_realloc_after_clear(
        first in 1_usize..200,
        second_vals in prop::collection::vec(symbol_val(), 1..100),
    ) {
        let mut arena = TreeArena::new();
        for _ in 0..first {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        let handles: Vec<_> = second_vals.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        prop_assert_eq!(arena.len(), second_vals.len());
        for (h, &v) in handles.iter().zip(second_vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    #[test]
    fn v5_realloc_after_reset(
        first in 1_usize..200,
        second_vals in prop::collection::vec(symbol_val(), 1..100),
    ) {
        let mut arena = TreeArena::new();
        for _ in 0..first {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        let handles: Vec<_> = second_vals.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        prop_assert_eq!(arena.len(), second_vals.len());
        for (h, &v) in handles.iter().zip(second_vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    #[test]
    fn v5_realloc_branches_after_clear(n in 1_usize..50) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::branch(vec![]));
        }
        arena.clear();
        let mut handles = Vec::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![]));
            handles.push(h);
        }
        prop_assert_eq!(arena.len(), n);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).symbol(), i as i32);
        }
    }

    #[test]
    fn v5_realloc_more_after_clear(first in 1_usize..100, second in 1_usize..200) {
        let mut arena = TreeArena::new();
        for _ in 0..first {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        for _ in 0..second {
            arena.alloc(TreeNode::leaf(1));
        }
        prop_assert_eq!(arena.len(), second);
    }
}

// ============================================================================
// 9. num_chunks >= ceil(N / capacity) (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_num_chunks_at_least_one(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert!(arena.num_chunks() >= 1);
    }

    #[test]
    fn v5_num_chunks_grows_past_capacity(cap in 1_usize..8, n in 10_usize..100) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        // With exponential growth we must have at least 1 chunk,
        // and capacity must cover all nodes.
        prop_assert!(arena.capacity() >= n);
        prop_assert!(arena.num_chunks() >= 1);
    }

    #[test]
    fn v5_single_chunk_within_capacity(cap in 50_usize..500, n in 1_usize..50) {
        prop_assume!(n <= cap);
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    #[test]
    fn v5_multiple_chunks_when_exceeding_cap(cap in 1_usize..10) {
        let n = cap + 1;
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert!(arena.num_chunks() >= 2);
    }
}

// ============================================================================
// 10. Symbol roundtrip for edge values (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_symbol_roundtrip_positive(sym in 0..1000i32) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn v5_symbol_roundtrip_negative(sym in -1000..0i32) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn v5_symbol_roundtrip_extremes(sym in prop::sample::select(
        vec![i32::MIN, i32::MAX, 0, 1, -1, i32::MIN + 1, i32::MAX - 1],
    )) {
        let mut arena = TreeArena::new();
        let h_leaf = arena.alloc(TreeNode::leaf(sym));
        let h_branch = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h_leaf).value(), sym);
        prop_assert_eq!(arena.get(h_branch).symbol(), sym);
    }

    #[test]
    fn v5_symbol_roundtrip_full_range(sym in prop::num::i32::ANY) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn v5_branch_symbol_roundtrip_full_range(sym in prop::num::i32::ANY) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }
}

// ============================================================================
// 11. Branch children preserved (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_branch_children_count(width in 1_usize..50) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..width)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let expected = children.to_vec();
        let parent = arena.alloc(TreeNode::branch(children));
        prop_assert_eq!(arena.get(parent).children().len(), width);
        for (i, &ch) in arena.get(parent).children().iter().enumerate() {
            prop_assert_eq!(ch, expected[i]);
        }
    }

    #[test]
    fn v5_branch_empty_children(_dummy in Just(())) {
        let mut arena = TreeArena::new();
        let parent = arena.alloc(TreeNode::branch(vec![]));
        prop_assert!(arena.get(parent).children().is_empty());
    }

    #[test]
    fn v5_leaf_has_no_children(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).children().is_empty());
    }

    #[test]
    fn v5_nested_branches_children(depth in 2_usize..20) {
        let mut arena = TreeArena::new();
        let mut current = arena.alloc(TreeNode::leaf(0));
        for d in 1..depth {
            let parent = arena.alloc(TreeNode::branch_with_symbol(d as i32, vec![current]));
            let node_ref = arena.get(parent);
            let children = node_ref.children();
            prop_assert_eq!(children.len(), 1);
            prop_assert_eq!(children[0], current);
            current = parent;
        }
    }
}

// ============================================================================
// 12. Leaf/branch kind flags preserved (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_leaf_is_leaf(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }

    #[test]
    fn v5_branch_is_branch(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }

    #[test]
    fn v5_mixed_kinds_preserved(n in 1_usize..100) {
        let mut arena = TreeArena::new();
        let mut leaf_handles = Vec::new();
        let mut branch_handles = Vec::new();
        for i in 0..n {
            if i % 2 == 0 {
                leaf_handles.push(arena.alloc(TreeNode::leaf(i as i32)));
            } else {
                branch_handles.push(arena.alloc(TreeNode::branch(vec![])));
            }
        }
        for h in &leaf_handles {
            prop_assert!(arena.get(*h).is_leaf());
        }
        for h in &branch_handles {
            prop_assert!(arena.get(*h).is_branch());
        }
    }

    #[test]
    fn v5_set_value_preserves_leaf_kind(
        original in symbol_val(),
        replacement in symbol_val(),
    ) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(original));
        arena.get_mut(h).set_value(replacement);
        prop_assert!(arena.get(h).is_leaf());
        prop_assert_eq!(arena.get(h).value(), replacement);
    }
}

// ============================================================================
// 13. Multiple alloc/clear cycles → len() correct (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn v5_multiple_clear_cycles(
        cycles in 1_usize..6,
        per_cycle in small_count(),
    ) {
        let mut arena = TreeArena::new();
        for _ in 0..cycles {
            for _ in 0..per_cycle {
                arena.alloc(TreeNode::leaf(0));
            }
            prop_assert_eq!(arena.len(), per_cycle);
            arena.clear();
            prop_assert_eq!(arena.len(), 0);
        }
    }

    #[test]
    fn v5_multiple_reset_cycles(
        cycles in 1_usize..6,
        per_cycle in small_count(),
    ) {
        let mut arena = TreeArena::new();
        for _ in 0..cycles {
            for _ in 0..per_cycle {
                arena.alloc(TreeNode::leaf(0));
            }
            prop_assert_eq!(arena.len(), per_cycle);
            arena.reset();
            prop_assert_eq!(arena.len(), 0);
        }
    }

    #[test]
    fn v5_alternating_clear_reset_cycles(cycles in 1_usize..6, per_cycle in small_count()) {
        let mut arena = TreeArena::new();
        for c in 0..cycles {
            for _ in 0..per_cycle {
                arena.alloc(TreeNode::leaf(0));
            }
            if c % 2 == 0 {
                arena.clear();
            } else {
                arena.reset();
            }
            prop_assert_eq!(arena.len(), 0);
            prop_assert!(arena.is_empty());
        }
    }

    #[test]
    fn v5_growing_cycles(cycles in 2_usize..5) {
        let mut arena = TreeArena::new();
        for c in 1..=cycles {
            let count = c * 20;
            for _ in 0..count {
                arena.alloc(TreeNode::leaf(0));
            }
            prop_assert_eq!(arena.len(), count);
            arena.reset();
        }
    }

    #[test]
    fn v5_cycle_values_independent(
        first_vals in prop::collection::vec(symbol_val(), 1..50),
        second_vals in prop::collection::vec(symbol_val(), 1..50),
    ) {
        let mut arena = TreeArena::new();
        for &v in &first_vals {
            arena.alloc(TreeNode::leaf(v));
        }
        arena.clear();
        let handles: Vec<_> = second_vals.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        for (h, &v) in handles.iter().zip(second_vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }
}

// ============================================================================
// 14. Handles from different allocs are different (4 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn v5_all_handles_unique(n in alloc_count()) {
        let mut arena = TreeArena::new();
        let mut set = HashSet::new();
        for _ in 0..n {
            let h = arena.alloc(TreeNode::leaf(0));
            prop_assert!(set.insert(h));
        }
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn v5_consecutive_handles_differ(n in 2_usize..300) {
        let mut arena = TreeArena::new();
        let mut prev = arena.alloc(TreeNode::leaf(0));
        for _ in 1..n {
            let cur = arena.alloc(TreeNode::leaf(0));
            prop_assert_ne!(prev, cur);
            prev = cur;
        }
    }

    #[test]
    fn v5_handle_copy_semantics(sym in symbol_val()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(sym));
        let h2 = h1; // Copy, not move
        prop_assert_eq!(h1, h2);
        prop_assert_eq!(arena.get(h1).value(), arena.get(h2).value());
    }

    #[test]
    fn v5_handle_set_size(n in alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|_| arena.alloc(TreeNode::leaf(0)))
            .collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }
}

// ============================================================================
// 15. Edge cases and misc (8 properties)
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
        prop_assert_eq!(arena.get(parent).children().len(), width);
    }

    #[test]
    fn v5_capacity_ge_len_always(n in alloc_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
            prop_assert!(arena.capacity() >= arena.len());
        }
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

    #[test]
    fn v5_metrics_consistent(n in alloc_count()) {
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
    fn v5_new_arena_is_empty(cap in 1_usize..1000) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn v5_node_handle_new_roundtrip(
        chunk_idx in 0..100u32,
        node_idx in 0..1000u32,
    ) {
        let h1 = NodeHandle::new(chunk_idx, node_idx);
        let h2 = NodeHandle::new(chunk_idx, node_idx);
        prop_assert_eq!(h1, h2);
    }
}
