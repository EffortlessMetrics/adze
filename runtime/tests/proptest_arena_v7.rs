//! Property-based tests for TreeArena v7 — 55+ proptest cases covering
//! allocation roundtrips, uniqueness, length consistency, NodeHandle
//! properties, mutation, growth under stress, parent-child relationships,
//! and edge cases.

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use adze::arena_allocator::{TreeArena, TreeNode};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn cap_strategy() -> impl Strategy<Value = usize> {
    1_usize..4096
}

fn symbol_strategy() -> impl Strategy<Value = i32> {
    prop::num::i32::ANY
}

fn small_count() -> impl Strategy<Value = usize> {
    1_usize..128
}

fn tiny_count() -> impl Strategy<Value = usize> {
    1_usize..32
}

// ===========================================================================
// 1. Alloc/get roundtrip (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 1a. Single leaf alloc+get roundtrips the value.
    #[test]
    fn roundtrip_single_leaf(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
    }

    /// 1b. Leaf roundtrip with custom capacity.
    #[test]
    fn roundtrip_leaf_custom_cap(cap in cap_strategy(), val in symbol_strategy()) {
        let mut arena = TreeArena::with_capacity(cap);
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
        prop_assert!(arena.get(h).is_leaf());
    }

    /// 1c. Branch roundtrip preserves is_branch.
    #[test]
    fn roundtrip_branch_empty_children(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    /// 1d. Branch with children roundtrip.
    #[test]
    fn roundtrip_branch_with_children(sym in symbol_strategy(), n in tiny_count()) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let parent = arena.alloc(TreeNode::branch_with_symbol(sym, children.clone()));
        let node_ref = arena.get(parent);
        prop_assert_eq!(node_ref.children(), &children[..]);
    }

    /// 1e. Bulk leaf alloc preserves every value.
    #[test]
    fn roundtrip_bulk_leaves(vals in prop::collection::vec(symbol_strategy(), 1..200)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    /// 1f. get().is_leaf() matches what we allocated.
    #[test]
    fn roundtrip_leaf_flag(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }

    /// 1g. get().is_branch() matches what we allocated.
    #[test]
    fn roundtrip_branch_flag(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(val, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }

    /// 1h. Leaf children() always returns empty slice.
    #[test]
    fn roundtrip_leaf_no_children(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert!(arena.get(h).children().is_empty());
    }
}

// ===========================================================================
// 2. Multiple allocations maintain uniqueness (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 2a. All handles from bulk alloc are distinct.
    #[test]
    fn unique_handles_bulk(n in small_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), handles.len());
    }

    /// 2b. No two sequential allocs return the same handle.
    #[test]
    fn unique_handles_sequential(n in 2_usize..64) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(0));
        for _ in 1..n {
            let h2 = arena.alloc(TreeNode::leaf(0));
            prop_assert_ne!(h1, h2);
        }
    }

    /// 2c. Handles differ even with identical values.
    #[test]
    fn unique_handles_same_value(val in symbol_strategy(), n in 2_usize..64) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|_| arena.alloc(TreeNode::leaf(val))).collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    /// 2d. Handles differ across chunk boundaries.
    #[test]
    fn unique_handles_cross_chunk(cap in 1_usize..16, n in 20_usize..100) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    /// 2e. Each handle resolves to the correct value.
    #[test]
    fn unique_values_preserved(vals in prop::collection::vec(symbol_strategy(), 2..64)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), vals[i]);
        }
    }

    /// 2f. Branch and leaf handles are distinct.
    #[test]
    fn unique_handles_mixed_types(n in tiny_count()) {
        let mut arena = TreeArena::new();
        let mut handles = Vec::new();
        for i in 0..n {
            if i % 2 == 0 {
                handles.push(arena.alloc(TreeNode::leaf(i as i32)));
            } else {
                handles.push(arena.alloc(TreeNode::branch(vec![])));
            }
        }
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    /// 2g. Hash values of distinct handles are mostly distinct.
    #[test]
    fn unique_handle_hashes(n in 2_usize..64) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let hashes: HashSet<_> = handles.iter().map(|h| {
            let mut hasher = DefaultHasher::new();
            h.hash(&mut hasher);
            hasher.finish()
        }).collect();
        // Hash collisions possible but extremely unlikely for small n
        prop_assert!(hashes.len() > n / 2);
    }

    /// 2h. Handles from different arenas may collide but each resolves correctly in its own.
    #[test]
    fn unique_per_arena_values(val_a in symbol_strategy(), val_b in symbol_strategy()) {
        let mut arena_a = TreeArena::new();
        let mut arena_b = TreeArena::new();
        let ha = arena_a.alloc(TreeNode::leaf(val_a));
        let hb = arena_b.alloc(TreeNode::leaf(val_b));
        prop_assert_eq!(arena_a.get(ha).value(), val_a);
        prop_assert_eq!(arena_b.get(hb).value(), val_b);
    }
}

// ===========================================================================
// 3. Arena length consistency (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 3a. len() increments by one per alloc.
    #[test]
    fn len_increments(n in small_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            prop_assert_eq!(arena.len(), i);
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    /// 3b. is_empty is true only before first alloc.
    #[test]
    fn is_empty_only_initially(n in 1_usize..64) {
        let mut arena = TreeArena::new();
        prop_assert!(arena.is_empty());
        arena.alloc(TreeNode::leaf(0));
        prop_assert!(!arena.is_empty());
        for _ in 1..n {
            arena.alloc(TreeNode::leaf(0));
            prop_assert!(!arena.is_empty());
        }
    }

    /// 3c. capacity >= len always.
    #[test]
    fn capacity_ge_len(cap in cap_strategy(), n in small_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    /// 3d. metrics().len() agrees with arena.len() after allocations.
    #[test]
    fn metrics_len_agrees(cap in cap_strategy(), n in small_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.metrics().len(), arena.len());
        prop_assert_eq!(arena.metrics().capacity(), arena.capacity());
    }

    /// 3e. reset brings len back to zero.
    #[test]
    fn reset_zeroes_len(n in small_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert_eq!(arena.len(), 0);
        prop_assert!(arena.is_empty());
    }
}

// ===========================================================================
// 4. NodeHandle properties (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 4a. NodeHandle is Copy — use after copy is valid.
    #[test]
    fn handle_is_copy(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(val));
        let h2 = h1; // copy
        prop_assert_eq!(arena.get(h1).value(), arena.get(h2).value());
    }

    /// 4b. NodeHandle Clone produces equal handle.
    #[test]
    fn handle_clone_eq(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(val));
        #[allow(clippy::clone_on_copy)]
        let h2 = h1.clone();
        prop_assert_eq!(h1, h2);
    }

    /// 4c. NodeHandle PartialEq is reflexive.
    #[test]
    fn handle_eq_reflexive(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(h, h);
    }

    /// 4d. NodeHandle Debug format doesn't panic.
    #[test]
    fn handle_debug_no_panic(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        let _ = format!("{:?}", h);
    }

    /// 4e. NodeHandle Hash is deterministic — same handle gives same hash.
    #[test]
    fn handle_hash_deterministic(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        let hash1 = {
            let mut hasher = DefaultHasher::new();
            h.hash(&mut hasher);
            hasher.finish()
        };
        let hash2 = {
            let mut hasher = DefaultHasher::new();
            h.hash(&mut hasher);
            hasher.finish()
        };
        prop_assert_eq!(hash1, hash2);
    }
}

// ===========================================================================
// 5. TreeNode modification through get_mut (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 5a. set_value changes the leaf's value.
    #[test]
    fn mut_set_value(old in symbol_strategy(), new in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(old));
        arena.get_mut(h).set_value(new);
        prop_assert_eq!(arena.get(h).value(), new);
    }

    /// 5b. Mutating one leaf doesn't affect another.
    #[test]
    fn mut_isolation(v1 in symbol_strategy(), v2 in symbol_strategy(), new_v in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(v1));
        let h2 = arena.alloc(TreeNode::leaf(v2));
        arena.get_mut(h1).set_value(new_v);
        prop_assert_eq!(arena.get(h1).value(), new_v);
        prop_assert_eq!(arena.get(h2).value(), v2);
    }

    /// 5c. Double mutation — second write wins.
    #[test]
    fn mut_double_write(a in symbol_strategy(), b in symbol_strategy(), c in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(a));
        arena.get_mut(h).set_value(b);
        arena.get_mut(h).set_value(c);
        prop_assert_eq!(arena.get(h).value(), c);
    }

    /// 5d. Mutating first node in a large arena works.
    #[test]
    fn mut_first_node(new_val in symbol_strategy(), n in 2_usize..128) {
        let mut arena = TreeArena::new();
        let first = arena.alloc(TreeNode::leaf(0));
        for i in 1..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.get_mut(first).set_value(new_val);
        prop_assert_eq!(arena.get(first).value(), new_val);
    }

    /// 5e. Mutating last node in a large arena works.
    #[test]
    fn mut_last_node(new_val in symbol_strategy(), n in 2_usize..128) {
        let mut arena = TreeArena::new();
        let mut last = arena.alloc(TreeNode::leaf(0));
        for i in 1..n {
            last = arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.get_mut(last).set_value(new_val);
        prop_assert_eq!(arena.get(last).value(), new_val);
    }

    /// 5f. set_value on branch is a no-op (branches have no mutable leaf symbol).
    #[test]
    fn mut_branch_noop(sym in symbol_strategy(), new_val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        arena.get_mut(h).set_value(new_val);
        // set_value only affects Leaf variant — branch symbol unchanged
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    /// 5g. Mutation across chunk boundary.
    #[test]
    fn mut_across_chunks(new_val in symbol_strategy()) {
        let mut arena = TreeArena::with_capacity(2);
        let _h1 = arena.alloc(TreeNode::leaf(1));
        let _h2 = arena.alloc(TreeNode::leaf(2));
        // Third alloc triggers new chunk
        let h3 = arena.alloc(TreeNode::leaf(3));
        arena.get_mut(h3).set_value(new_val);
        prop_assert_eq!(arena.get(h3).value(), new_val);
    }

    /// 5h. Bulk mutation preserves all updated values.
    #[test]
    fn mut_bulk_update(
        vals in prop::collection::vec(symbol_strategy(), 1..64),
        offset in 1_i32..1000,
    ) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for &h in &handles {
            let old = arena.get(h).value();
            arena.get_mut(h).set_value(old.wrapping_add(offset));
        }
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v.wrapping_add(offset));
        }
    }
}

// ===========================================================================
// 6. Arena growth under stress (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 6a. Arena grows chunks when cap is tiny.
    #[test]
    fn growth_tiny_cap(cap in 1_usize..8, n in 16_usize..128) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.num_chunks() > 1);
        prop_assert_eq!(arena.len(), n);
    }

    /// 6b. capacity grows to accommodate all allocations.
    #[test]
    fn growth_capacity_sufficient(cap in 1_usize..16, n in 16_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= n);
    }

    /// 6c. memory_usage never decreases after allocations.
    #[test]
    fn growth_memory_monotonic(n in tiny_count()) {
        let mut arena = TreeArena::new();
        let mut prev = arena.memory_usage();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let current = arena.memory_usage();
            prop_assert!(current >= prev);
            prev = current;
        }
    }

    /// 6d. All values survive growth across many chunks.
    #[test]
    fn growth_values_survive(cap in 1_usize..4, n in 32_usize..128) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    /// 6e. num_chunks never decreases during allocation.
    #[test]
    fn growth_chunks_monotonic(cap in 1_usize..8, n in small_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        let mut prev_chunks = arena.num_chunks();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cur = arena.num_chunks();
            prop_assert!(cur >= prev_chunks);
            prev_chunks = cur;
        }
    }

    /// 6f. Reset preserves chunk count (memory is retained).
    #[test]
    fn growth_reset_keeps_chunks(cap in 1_usize..4, n in 16_usize..64) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let chunks_before = arena.num_chunks();
        arena.reset();
        prop_assert_eq!(arena.num_chunks(), chunks_before);
    }

    /// 6g. clear() reduces chunks to 1.
    #[test]
    fn growth_clear_reduces_chunks(cap in 1_usize..4, n in 16_usize..64) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
        prop_assert_eq!(arena.len(), 0);
    }

    /// 6h. Arena usable after reset — re-allocate same count.
    #[test]
    fn growth_reuse_after_reset(n in small_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf((i as i32) + 1000))).collect();
        prop_assert_eq!(arena.len(), n);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), (i as i32) + 1000);
        }
    }
}

// ===========================================================================
// 7. Parent-child relationships (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 7a. Branch children() returns the same handles we gave it.
    #[test]
    fn parent_child_roundtrip(n in 1_usize..16) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        let node_ref = arena.get(parent);
        prop_assert_eq!(node_ref.children(), &children[..]);
    }

    /// 7b. Child values accessible through parent's children list.
    #[test]
    fn parent_child_values(vals in prop::collection::vec(symbol_strategy(), 1..16)) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        for (i, &ch) in arena.get(parent).children().iter().enumerate() {
            prop_assert_eq!(arena.get(ch).value(), vals[i]);
        }
    }

    /// 7c. Nested branches — grandparent -> parent -> leaf.
    #[test]
    fn nested_branches(val in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let leaf = arena.alloc(TreeNode::leaf(val));
        let parent = arena.alloc(TreeNode::branch(vec![leaf]));
        let grandparent = arena.alloc(TreeNode::branch(vec![parent]));

        let gp_children = arena.get(grandparent).children().to_vec();
        prop_assert_eq!(gp_children.len(), 1);
        let p_children = arena.get(gp_children[0]).children().to_vec();
        prop_assert_eq!(p_children.len(), 1);
        prop_assert_eq!(arena.get(p_children[0]).value(), val);
    }

    /// 7d. Branch with no children has empty children().
    #[test]
    fn branch_no_children(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).children().is_empty());
    }

    /// 7e. branch_with_symbol preserves symbol and children.
    #[test]
    fn branch_with_symbol_roundtrip(sym in symbol_strategy(), n in 0_usize..8) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, children.clone()));
        let node_ref = arena.get(h);
        prop_assert_eq!(node_ref.symbol(), sym);
        prop_assert_eq!(node_ref.children(), &children[..]);
    }

    /// 7f. Deep tree (linear chain) — all values accessible.
    #[test]
    fn deep_linear_chain(depth in 1_usize..32) {
        let mut arena = TreeArena::new();
        let leaf = arena.alloc(TreeNode::leaf(42));
        let mut current = leaf;
        for _ in 0..depth {
            current = arena.alloc(TreeNode::branch(vec![current]));
        }
        // Walk down
        let mut node = current;
        for _ in 0..depth {
            let ch = arena.get(node).children().to_vec();
            prop_assert_eq!(ch.len(), 1);
            node = ch[0];
        }
        prop_assert_eq!(arena.get(node).value(), 42);
    }

    /// 7g. Wide tree — one parent with many children.
    #[test]
    fn wide_tree(n in 1_usize..64) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let root = arena.alloc(TreeNode::branch(children.clone()));
        prop_assert_eq!(arena.get(root).children().len(), n);
        for (i, &ch) in arena.get(root).children().iter().enumerate() {
            prop_assert_eq!(arena.get(ch).value(), i as i32);
        }
    }

    /// 7h. Multiple sibling branches share child handles correctly.
    #[test]
    fn sibling_branches(n in 1_usize..8) {
        let mut arena = TreeArena::new();
        let shared_leaf = arena.alloc(TreeNode::leaf(99));
        let branches: Vec<_> = (0..n)
            .map(|_| arena.alloc(TreeNode::branch(vec![shared_leaf])))
            .collect();
        for &b in &branches {
            let node_ref = arena.get(b);
            let ch = node_ref.children();
            prop_assert_eq!(ch.len(), 1);
            prop_assert_eq!(arena.get(ch[0]).value(), 99);
        }
    }
}

// ===========================================================================
// 8. Edge cases (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 8a. with_capacity(1) works and grows.
    #[test]
    fn edge_cap_one(n in 1_usize..64) {
        let mut arena = TreeArena::with_capacity(1);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    /// 8b. Extreme symbol values (i32 boundary).
    #[test]
    fn edge_extreme_symbols(val in prop::num::i32::ANY) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
    }

    /// 8c. Alloc after reset produces valid handles.
    #[test]
    fn edge_alloc_after_reset(n in 1_usize..32) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        let h = arena.alloc(TreeNode::leaf(777));
        prop_assert_eq!(arena.get(h).value(), 777);
        prop_assert_eq!(arena.len(), 1);
    }

    /// 8d. Alloc after clear produces valid handles.
    #[test]
    fn edge_alloc_after_clear(n in 1_usize..32) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        let h = arena.alloc(TreeNode::leaf(888));
        prop_assert_eq!(arena.get(h).value(), 888);
        prop_assert_eq!(arena.len(), 1);
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    /// 8e. Multiple reset-alloc cycles.
    #[test]
    fn edge_repeated_reset_cycles(cycles in 1_usize..8, n in 1_usize..32) {
        let mut arena = TreeArena::new();
        for _ in 0..cycles {
            let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
            prop_assert_eq!(arena.len(), n);
            for (i, h) in handles.iter().enumerate() {
                prop_assert_eq!(arena.get(*h).value(), i as i32);
            }
            arena.reset();
            prop_assert_eq!(arena.len(), 0);
        }
    }
}
