//! Property-based tests (v5) for TreeArena.
//!
//! 55 property tests covering capacity ranges, allocation patterns, handle
//! validity, kind preservation, uniqueness, growth, interleaved ops, large
//! allocations, sequential validity, and invariants under random operations.

use adze::arena_allocator::{TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Helpers
// ============================================================================

fn arb_symbol() -> impl Strategy<Value = i32> {
    prop::num::i32::ANY
}

fn arb_capacity() -> impl Strategy<Value = usize> {
    1usize..=1000
}

fn arb_alloc_count() -> impl Strategy<Value = usize> {
    1usize..=300
}

fn arb_small_count() -> impl Strategy<Value = usize> {
    1usize..=50
}

// ============================================================================
// 1. Random capacity always works (1..1000)
// ============================================================================

proptest! {
    #[test]
    fn v5_capacity_range_valid(cap in 1usize..=1000) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert_eq!(arena.capacity(), cap);
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn v5_capacity_alloc_one(cap in arb_capacity()) {
        let mut arena = TreeArena::with_capacity(cap);
        let h = arena.alloc(TreeNode::leaf(7));
        prop_assert_eq!(arena.len(), 1);
        prop_assert_eq!(arena.get(h).value(), 7);
    }

    #[test]
    fn v5_capacity_fill_exact(cap in 1usize..=128) {
        let mut arena = TreeArena::with_capacity(cap);
        let mut handles = Vec::with_capacity(cap);
        for i in 0..cap {
            handles.push(arena.alloc(TreeNode::leaf(i as i32)));
        }
        prop_assert_eq!(arena.len(), cap);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_capacity_overflow_by_one(cap in 1usize..=128) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..=cap {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), cap + 1);
        prop_assert!(arena.capacity() > cap);
    }

    #[test]
    fn v5_capacity_memory_positive(cap in arb_capacity()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.memory_usage() > 0);
    }
}

// ============================================================================
// 2. Random number of allocations
// ============================================================================

proptest! {
    #[test]
    fn v5_random_alloc_count_leaves(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_random_alloc_count_branches(n in 1usize..=100) {
        let mut arena = TreeArena::new();
        let mut prev_handles = Vec::new();
        for i in 0..n {
            let children = prev_handles.clone();
            let h = arena.alloc(TreeNode::branch_with_symbol(i as i32, children));
            prev_handles.push(h);
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn v5_random_mixed_leaf_branch(ops in prop::collection::vec(prop::bool::ANY, 1..200)) {
        let mut arena = TreeArena::new();
        let mut leaf_handles = Vec::new();
        for is_leaf in &ops {
            if *is_leaf || leaf_handles.is_empty() {
                leaf_handles.push(arena.alloc(TreeNode::leaf(0)));
            } else {
                let child = *leaf_handles.last().unwrap();
                arena.alloc(TreeNode::branch(vec![child]));
            }
        }
        prop_assert_eq!(arena.len(), ops.len());
    }

    #[test]
    fn v5_alloc_count_with_varied_capacity(cap in 1usize..=64, n in 1usize..=200) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
        prop_assert!(arena.capacity() >= n);
    }

    #[test]
    fn v5_alloc_preserves_count_after_get(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        // Access all handles — should not change len
        for h in &handles {
            let _ = arena.get(*h).value();
        }
        prop_assert_eq!(arena.len(), n);
    }
}

// ============================================================================
// 3. All allocated handles are valid
// ============================================================================

proptest! {
    #[test]
    fn v5_all_handles_valid_leaves(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        for h in &handles {
            let node = arena.get(*h);
            prop_assert!(node.is_leaf());
        }
    }

    #[test]
    fn v5_all_handles_valid_branches(n in arb_small_count()) {
        let mut arena = TreeArena::new();
        let leaf = arena.alloc(TreeNode::leaf(0));
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![leaf])))
            .collect();
        for h in &handles {
            let node = arena.get(*h);
            prop_assert!(node.is_branch());
            prop_assert_eq!(node.children().len(), 1);
        }
    }

    #[test]
    fn v5_handle_valid_after_many_allocs(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let first = arena.alloc(TreeNode::leaf(-999));
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.get(first).value(), -999);
    }

    #[test]
    fn v5_handles_valid_across_chunk_boundary(cap in 1usize..=8, n in 10usize..=100) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_get_mut_does_not_invalidate_other_handles(n in 2usize..=100) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        // Mutate the first handle
        arena.get_mut(handles[0]).set_value(9999);
        // All other handles remain valid
        for (i, h) in handles.iter().enumerate().skip(1) {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
        prop_assert_eq!(arena.get(handles[0]).value(), 9999);
    }
}

// ============================================================================
// 4. Kind values preserved after allocation
// ============================================================================

proptest! {
    #[test]
    fn v5_leaf_value_roundtrip(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    #[test]
    fn v5_branch_symbol_roundtrip(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    #[test]
    fn v5_many_distinct_values(vals in prop::collection::vec(arb_symbol(), 1..200)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (&v, &h) in vals.iter().zip(handles.iter()) {
            prop_assert_eq!(arena.get(h).value(), v);
        }
    }

    #[test]
    fn v5_extreme_symbol_values(sym in prop::sample::select(vec![
        i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX
    ])) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn v5_set_value_preserves_kind(original in arb_symbol(), replacement in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(original));
        prop_assert!(arena.get(h).is_leaf());
        arena.get_mut(h).set_value(replacement);
        prop_assert!(arena.get(h).is_leaf());
        prop_assert_eq!(arena.get(h).value(), replacement);
    }
}

// ============================================================================
// 5. Handles are unique
// ============================================================================

proptest! {
    #[test]
    fn v5_handles_unique_set(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn v5_handles_unique_small_capacity(n in 1usize..=150) {
        let mut arena = TreeArena::with_capacity(2);
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn v5_consecutive_handles_differ(n in 2usize..=200) {
        let mut arena = TreeArena::new();
        let mut prev = arena.alloc(TreeNode::leaf(0));
        for i in 1..n {
            let cur = arena.alloc(TreeNode::leaf(i as i32));
            prop_assert_ne!(prev, cur);
            prev = cur;
        }
    }

    #[test]
    fn v5_handles_hashmap_usable(n in arb_small_count()) {
        let mut arena = TreeArena::new();
        let mut map = HashMap::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            map.insert(h, i);
        }
        prop_assert_eq!(map.len(), n);
    }

    #[test]
    fn v5_handle_copy_semantics(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(sym));
        let h2 = h1; // Copy
        let h3 = h1; // Copy again
        prop_assert_eq!(h1, h2);
        prop_assert_eq!(h2, h3);
        prop_assert_eq!(arena.get(h1).value(), arena.get(h3).value());
    }
}

// ============================================================================
// 6. Arena grows on demand
// ============================================================================

proptest! {
    #[test]
    fn v5_grows_past_initial_capacity(cap in 1usize..=32, extra in 1usize..=100) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..(cap + extra) {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), cap + extra);
        prop_assert!(arena.capacity() >= cap + extra);
    }

    #[test]
    fn v5_chunk_count_increases(cap in 1usize..=4, n in 20usize..=100) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.num_chunks() > 1);
    }

    #[test]
    fn v5_capacity_monotonic(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let mut caps = Vec::with_capacity(n);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            caps.push(arena.capacity());
        }
        for window in caps.windows(2) {
            prop_assert!(window[1] >= window[0]);
        }
    }

    #[test]
    fn v5_num_chunks_monotonic(n in arb_alloc_count()) {
        let mut arena = TreeArena::with_capacity(4);
        let mut chunks = Vec::with_capacity(n);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            chunks.push(arena.num_chunks());
        }
        for window in chunks.windows(2) {
            prop_assert!(window[1] >= window[0]);
        }
    }

    #[test]
    fn v5_memory_grows_with_allocs(n in 2usize..=200) {
        let mut arena = TreeArena::new();
        arena.alloc(TreeNode::leaf(0));
        let mem_after_one = arena.memory_usage();
        for i in 1..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.memory_usage() >= mem_after_one);
    }
}

// ============================================================================
// 7. Random alloc/get interleaved patterns
// ============================================================================

proptest! {
    #[test]
    fn v5_interleaved_alloc_get(ops in prop::collection::vec(0u8..10, 1..200)) {
        let mut arena = TreeArena::new();
        let mut handles = Vec::new();
        for op in &ops {
            if *op < 7 || handles.is_empty() {
                // Allocate
                let h = arena.alloc(TreeNode::leaf(handles.len() as i32));
                handles.push(h);
            } else {
                // Read a random existing handle
                let idx = (*op as usize) % handles.len();
                let node = arena.get(handles[idx]);
                prop_assert!(node.is_leaf());
            }
        }
        prop_assert_eq!(arena.len(), handles.len());
    }

    #[test]
    fn v5_alloc_get_mut_interleaved(n in arb_small_count()) {
        let mut arena = TreeArena::new();
        let mut handles = Vec::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            handles.push(h);
            // Immediately mutate previous if exists
            if i > 0 {
                arena.get_mut(handles[i - 1]).set_value(-(i as i32));
            }
        }
        // Verify mutations
        for (i, h) in handles.iter().enumerate() {
            if i < n - 1 {
                prop_assert_eq!(arena.get(*h).value(), -((i + 1) as i32));
            } else {
                prop_assert_eq!(arena.get(*h).value(), i as i32);
            }
        }
    }

    #[test]
    fn v5_alloc_branch_then_read_children(depth in 1usize..=20) {
        let mut arena = TreeArena::new();
        let mut current = arena.alloc(TreeNode::leaf(0));
        for i in 1..=depth {
            current = arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![current]));
        }
        // Walk back down
        let mut node_h = current;
        for i in (1..=depth).rev() {
            let node = arena.get(node_h);
            prop_assert!(node.is_branch());
            prop_assert_eq!(node.symbol(), i as i32);
            prop_assert_eq!(node.children().len(), 1);
            node_h = node.children()[0];
        }
        prop_assert!(arena.get(node_h).is_leaf());
        prop_assert_eq!(arena.get(node_h).value(), 0);
    }

    #[test]
    fn v5_wide_branch_random_width(width in 1usize..=100) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..width)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        let node = arena.get(parent);
        prop_assert_eq!(node.children().len(), width);
        for (i, ch) in node.children().iter().enumerate() {
            prop_assert_eq!(arena.get(*ch).value(), i as i32);
        }
    }
}

// ============================================================================
// 8. Large numbers of allocations
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn v5_large_alloc_1000(n in 800usize..=1200) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        prop_assert_eq!(arena.len(), n);
        // Spot-check first, middle, last
        prop_assert_eq!(arena.get(handles[0]).value(), 0);
        prop_assert_eq!(arena.get(handles[n / 2]).value(), (n / 2) as i32);
        prop_assert_eq!(arena.get(handles[n - 1]).value(), (n - 1) as i32);
    }

    #[test]
    fn v5_large_alloc_all_unique(n in 500usize..=1000) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn v5_large_alloc_all_retrievable(n in 500usize..=1000) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_large_alloc_metrics_consistent(n in 500usize..=1000) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), n);
        prop_assert!(!m.is_empty());
        prop_assert!(m.capacity() >= n);
        prop_assert!(m.num_chunks() >= 1);
        prop_assert!(m.memory_usage() > 0);
    }
}

// ============================================================================
// 9. Sequential handle validity
// ============================================================================

proptest! {
    #[test]
    fn v5_sequential_allocs_order_preserved(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        // Reading in forward order
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_sequential_allocs_reverse_read(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        // Reading in reverse order
        for (i, h) in handles.iter().enumerate().rev() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_first_handle_always_valid(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let first = arena.alloc(TreeNode::leaf(42));
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.get(first).value(), 42);
    }

    #[test]
    fn v5_last_handle_always_valid(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n.saturating_sub(1) {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let last = arena.alloc(TreeNode::leaf(-1));
        prop_assert_eq!(arena.get(last).value(), -1);
    }

    #[test]
    fn v5_random_access_order(
        n in 10usize..=200,
        indices in prop::collection::vec(0usize..10, 1..50),
    ) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        for idx in &indices {
            let actual_idx = *idx % n;
            prop_assert_eq!(arena.get(handles[actual_idx]).value(), actual_idx as i32);
        }
    }
}

// ============================================================================
// 10. Arena invariants under random operations
// ============================================================================

proptest! {
    #[test]
    fn v5_invariant_len_le_capacity(cap in 1usize..=64, n in arb_alloc_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(arena.len() <= arena.capacity());
        }
    }

    #[test]
    fn v5_invariant_not_empty_iff_len_gt_zero(n in 0usize..=100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(!arena.is_empty(), !arena.is_empty());
    }

    #[test]
    fn v5_invariant_metrics_agree(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), arena.len());
        prop_assert_eq!(m.capacity(), arena.capacity());
        prop_assert_eq!(m.num_chunks(), arena.num_chunks());
        prop_assert_eq!(m.memory_usage(), arena.memory_usage());
        prop_assert_eq!(m.is_empty(), arena.is_empty());
    }

    #[test]
    fn v5_invariant_reset_then_reuse(
        n1 in arb_small_count(),
        n2 in arb_small_count(),
    ) {
        let mut arena = TreeArena::new();
        for i in 0..n1 {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        let handles: Vec<_> = (0..n2)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        prop_assert_eq!(arena.len(), n2);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_invariant_clear_then_reuse(
        n1 in arb_small_count(),
        n2 in arb_small_count(),
    ) {
        let mut arena = TreeArena::new();
        for i in 0..n1 {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
        let handles: Vec<_> = (0..n2)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        prop_assert_eq!(arena.len(), n2);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn v5_invariant_multiple_resets(cycles in 1usize..=5, per_cycle in arb_small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..cycles {
            for i in 0..per_cycle {
                arena.alloc(TreeNode::leaf(i as i32));
            }
            prop_assert_eq!(arena.len(), per_cycle);
            arena.reset();
            prop_assert!(arena.is_empty());
        }
    }

    #[test]
    fn v5_invariant_deterministic_across_arenas(n in arb_alloc_count()) {
        let mut a = TreeArena::new();
        let mut b = TreeArena::new();
        for i in 0..n {
            let ha = a.alloc(TreeNode::leaf(i as i32));
            let hb = b.alloc(TreeNode::leaf(i as i32));
            prop_assert_eq!(ha, hb);
            prop_assert_eq!(a.get(ha).value(), b.get(hb).value());
        }
    }

    #[test]
    fn v5_invariant_leaf_is_not_branch(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }

    #[test]
    fn v5_invariant_branch_is_not_leaf(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }

    #[test]
    fn v5_invariant_children_empty_for_leaf(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).children().is_empty());
    }
}
