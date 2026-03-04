//! Property-based tests for TreeArena, NodeHandle, and TreeNode.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use adze::arena_allocator::{ArenaMetrics, NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn cap_strategy() -> impl Strategy<Value = usize> {
    1_usize..10_000
}

fn symbol_strategy() -> impl Strategy<Value = i32> {
    prop::num::i32::ANY
}

fn small_count() -> impl Strategy<Value = usize> {
    0_usize..256
}

fn child_count() -> impl Strategy<Value = usize> {
    0_usize..16
}

// ---------------------------------------------------------------------------
// 1-10: TreeArena construction & capacity
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 1. with_capacity always produces an empty arena.
    #[test]
    fn arena_with_capacity_is_empty(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert_eq!(arena.len(), 0);
        prop_assert!(arena.is_empty());
    }

    /// 2. Initial capacity is at least what was requested.
    #[test]
    fn arena_capacity_ge_requested(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.capacity() >= cap);
    }

    /// 3. Fresh arena always has exactly 1 chunk.
    #[test]
    fn arena_starts_with_one_chunk(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    /// 4. Metrics agree with direct accessors on fresh arena.
    #[test]
    fn arena_fresh_metrics_consistent(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        let m = arena.metrics();
        prop_assert_eq!(m.len(), arena.len());
        prop_assert_eq!(m.capacity(), arena.capacity());
        prop_assert_eq!(m.num_chunks(), arena.num_chunks());
        prop_assert_eq!(m.memory_usage(), arena.memory_usage());
    }

    /// 5. memory_usage equals capacity * size_of::<TreeNode>().
    #[test]
    fn arena_memory_usage_formula(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        let node_size = std::mem::size_of::<TreeNode>();
        prop_assert_eq!(arena.memory_usage(), arena.capacity() * node_size);
    }

    /// 6. Default arena is equivalent to with_capacity(1024).
    #[test]
    fn arena_default_matches_new(_dummy in 0..1u8) {
        let a = TreeArena::new();
        let b = TreeArena::default();
        prop_assert_eq!(a.capacity(), b.capacity());
        prop_assert_eq!(a.len(), b.len());
    }

    /// 7. is_empty iff len == 0 after random allocations.
    #[test]
    fn arena_is_empty_iff_len_zero(cap in cap_strategy(), n in small_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.is_empty(), arena.len() == 0);
    }

    /// 8. Metrics snapshot is Copy + Clone + PartialEq.
    #[test]
    fn metrics_is_copy_clone(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        let m1 = arena.metrics();
        let m2 = m1;          // Copy
        let m3 = m1.clone();  // Clone
        prop_assert_eq!(m1, m2);
        prop_assert_eq!(m1, m3);
    }

    /// 9. Metrics Debug impl doesn't panic.
    #[test]
    fn metrics_debug_no_panic(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        let m = arena.metrics();
        let _ = format!("{:?}", m);
    }

    /// 10. Arena Debug impl doesn't panic.
    #[test]
    fn arena_debug_no_panic(cap in cap_strategy()) {
        let arena = TreeArena::with_capacity(cap);
        let _ = format!("{:?}", arena);
    }
}

// ---------------------------------------------------------------------------
// 11-20: Allocation & retrieval
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 11. Single alloc+get round-trips the value.
    #[test]
    fn alloc_get_roundtrip(cap in cap_strategy(), val in symbol_strategy()) {
        let mut arena = TreeArena::with_capacity(cap);
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
    }

    /// 12. Bulk alloc preserves all values.
    #[test]
    fn bulk_alloc_preserves_values(
        cap in cap_strategy(),
        vals in prop::collection::vec(symbol_strategy(), 0..128),
    ) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    /// 13. len increments by 1 per allocation.
    #[test]
    fn len_increments(cap in cap_strategy(), n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            prop_assert_eq!(arena.len(), i + 1);
        }
    }

    /// 14. get_mut can change a leaf value.
    #[test]
    fn get_mut_changes_value(cap in cap_strategy(), v1 in symbol_strategy(), v2 in symbol_strategy()) {
        let mut arena = TreeArena::with_capacity(cap);
        let h = arena.alloc(TreeNode::leaf(v1));
        arena.get_mut(h).set_value(v2);
        prop_assert_eq!(arena.get(h).value(), v2);
    }

    /// 15. get_mut on one handle doesn't affect others.
    #[test]
    fn get_mut_isolated(cap in cap_strategy(), a in symbol_strategy(), b in symbol_strategy(), c in symbol_strategy()) {
        let mut arena = TreeArena::with_capacity(cap);
        let h1 = arena.alloc(TreeNode::leaf(a));
        let h2 = arena.alloc(TreeNode::leaf(b));
        arena.get_mut(h1).set_value(c);
        prop_assert_eq!(arena.get(h1).value(), c);
        prop_assert_eq!(arena.get(h2).value(), b);
    }

    /// 16. All handles from one arena are unique.
    #[test]
    fn handles_are_unique(cap in cap_strategy(), n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), handles.len());
    }

    /// 17. Branch alloc preserves children.
    #[test]
    fn branch_children_preserved(cap in cap_strategy(), nchildren in child_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        let children: Vec<_> = (0..nchildren).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        let node_ref = arena.get(parent);
        prop_assert_eq!(node_ref.children(), &children[..]);
    }

    /// 18. branch_with_symbol preserves symbol.
    #[test]
    fn branch_with_symbol_preserved(cap in cap_strategy(), sym in symbol_strategy(), nch in child_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        let children: Vec<_> = (0..nch).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, children));
        prop_assert_eq!(arena.get(h).symbol(), sym);
        prop_assert!(arena.get(h).is_branch());
    }

    /// 19. Leaf nodes report is_leaf=true, is_branch=false.
    #[test]
    fn leaf_kind_flags(cap in cap_strategy(), val in symbol_strategy()) {
        let mut arena = TreeArena::with_capacity(cap);
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }

    /// 20. Branch nodes report is_branch=true, is_leaf=false.
    #[test]
    fn branch_kind_flags(cap in cap_strategy()) {
        let mut arena = TreeArena::with_capacity(cap);
        let c = arena.alloc(TreeNode::leaf(0));
        let h = arena.alloc(TreeNode::branch(vec![c]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }
}

// ---------------------------------------------------------------------------
// 21-30: Reset and clear invariants
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 21. reset sets len to 0.
    #[test]
    fn reset_zeroes_len(cap in cap_strategy(), n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        arena.reset();
        prop_assert_eq!(arena.len(), 0);
        prop_assert!(arena.is_empty());
    }

    /// 22. reset preserves capacity.
    #[test]
    fn reset_preserves_capacity(cap in cap_strategy(), n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        let cap_before = arena.capacity();
        arena.reset();
        prop_assert_eq!(arena.capacity(), cap_before);
    }

    /// 23. reset preserves num_chunks.
    #[test]
    fn reset_preserves_chunks(cap in cap_strategy(), n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        let chunks_before = arena.num_chunks();
        arena.reset();
        prop_assert_eq!(arena.num_chunks(), chunks_before);
    }

    /// 24. clear sets len to 0.
    #[test]
    fn clear_zeroes_len(cap in cap_strategy(), n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        arena.clear();
        prop_assert_eq!(arena.len(), 0);
        prop_assert!(arena.is_empty());
    }

    /// 25. clear reduces num_chunks to 1.
    #[test]
    fn clear_reduces_to_one_chunk(cap in 1_usize..16, n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    /// 26. After reset, arena can be re-used identically.
    #[test]
    fn reuse_after_reset(cap in cap_strategy(), n in 1_usize..64) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        arena.reset();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32 + 100))).collect();
        for (idx, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), idx as i32 + 100);
        }
    }

    /// 27. After clear, arena can be re-used identically.
    #[test]
    fn reuse_after_clear(cap in cap_strategy(), n in 1_usize..64) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        arena.clear();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32 + 200))).collect();
        for (idx, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), idx as i32 + 200);
        }
    }

    /// 28. Multiple resets don't corrupt arena.
    #[test]
    fn multiple_resets(cap in cap_strategy(), rounds in 1_usize..8, n in 1_usize..32) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..rounds {
            for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
            arena.reset();
            prop_assert_eq!(arena.len(), 0);
        }
    }

    /// 29. Multiple clears don't corrupt arena.
    #[test]
    fn multiple_clears(cap in cap_strategy(), rounds in 1_usize..8, n in 1_usize..32) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..rounds {
            for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
            arena.clear();
            prop_assert_eq!(arena.len(), 0);
            prop_assert_eq!(arena.num_chunks(), 1);
        }
    }

    /// 30. Metrics after reset reflect empty state but retained capacity.
    #[test]
    fn metrics_after_reset(cap in cap_strategy(), n in 1_usize..128) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        let cap_before = arena.capacity();
        arena.reset();
        let m = arena.metrics();
        prop_assert_eq!(m.len(), 0);
        prop_assert!(m.is_empty());
        prop_assert_eq!(m.capacity(), cap_before);
    }
}

// ---------------------------------------------------------------------------
// 31-40: Arena growth patterns
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 31. Filling exactly cap nodes stays in one chunk.
    #[test]
    fn filling_cap_one_chunk(cap in 1_usize..512) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..cap { arena.alloc(TreeNode::leaf(i as i32)); }
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    /// 32. cap+1 nodes creates exactly 2 chunks.
    #[test]
    fn cap_plus_one_two_chunks(cap in 1_usize..512) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..=cap { arena.alloc(TreeNode::leaf(i as i32)); }
        prop_assert_eq!(arena.num_chunks(), 2);
    }

    /// 33. Capacity never decreases after allocation.
    #[test]
    fn capacity_monotonic(cap in cap_strategy(), n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        let mut prev = arena.capacity();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cur = arena.capacity();
            prop_assert!(cur >= prev);
            prev = cur;
        }
    }

    /// 34. num_chunks never decreases during allocation.
    #[test]
    fn chunks_monotonic(cap in 1_usize..32, n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        let mut prev = arena.num_chunks();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cur = arena.num_chunks();
            prop_assert!(cur >= prev);
            prev = cur;
        }
    }

    /// 35. capacity >= len always.
    #[test]
    fn capacity_ge_len(cap in cap_strategy(), n in small_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        prop_assert!(arena.capacity() >= arena.len());
    }

    /// 36. memory_usage >= len * size_of TreeNode.
    #[test]
    fn memory_ge_len_times_node(cap in cap_strategy(), n in small_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        let node_sz = std::mem::size_of::<TreeNode>();
        prop_assert!(arena.memory_usage() >= arena.len() * node_sz);
    }

    /// 37. Metrics consistency after growth.
    #[test]
    fn metrics_consistent_after_growth(cap in 1_usize..32, n in 1_usize..256) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n { arena.alloc(TreeNode::leaf(i as i32)); }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), arena.len());
        prop_assert_eq!(m.capacity(), arena.capacity());
        prop_assert_eq!(m.num_chunks(), arena.num_chunks());
        prop_assert_eq!(m.memory_usage(), arena.memory_usage());
    }

    /// 38. Two arenas with same operations have same len.
    #[test]
    fn deterministic_len(cap in cap_strategy(), n in small_count()) {
        let mut a = TreeArena::with_capacity(cap);
        let mut b = TreeArena::with_capacity(cap);
        for i in 0..n {
            a.alloc(TreeNode::leaf(i as i32));
            b.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(a.len(), b.len());
        prop_assert_eq!(a.capacity(), b.capacity());
        prop_assert_eq!(a.num_chunks(), b.num_chunks());
    }

    /// 39. Chunk growth is exponential (capacity at least doubles).
    #[test]
    fn growth_at_least_doubles(cap in 1_usize..128) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..cap { arena.alloc(TreeNode::leaf(i as i32)); }
        let cap1 = arena.capacity();
        // trigger second chunk
        arena.alloc(TreeNode::leaf(0));
        let cap2 = arena.capacity();
        prop_assert!(cap2 >= cap1 + cap, "Expected cap2({}) >= cap1({}) + initial({})", cap2, cap1, cap);
    }

    /// 40. Arena handles all survive growth.
    #[test]
    fn handles_survive_growth(cap in 1_usize..32, n in 1_usize..128) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<_> = (0..n).map(|i| {
            arena.alloc(TreeNode::leaf(i as i32))
        }).collect();
        // Verify all handles still work after potential growth
        for (idx, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), idx as i32);
        }
    }
}

// ---------------------------------------------------------------------------
// 41-50: NodeHandle properties & TreeNode construction
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 41. NodeHandle is Copy: original and copy are equal.
    #[test]
    fn handle_copy(a in 0_u32..1000, b in 0_u32..1000) {
        let h = NodeHandle::new(a, b);
        let h2 = h;   // Copy
        prop_assert_eq!(h, h2);
    }

    /// 42. NodeHandle Clone equals original.
    #[test]
    fn handle_clone(a in 0_u32..1000, b in 0_u32..1000) {
        let h = NodeHandle::new(a, b);
        #[allow(clippy::clone_on_copy)]
        let h2 = h.clone();
        prop_assert_eq!(h, h2);
    }

    /// 43. Equal NodeHandles have equal hashes.
    #[test]
    fn handle_eq_implies_hash_eq(a in 0_u32..1000, b in 0_u32..1000) {
        let h1 = NodeHandle::new(a, b);
        let h2 = NodeHandle::new(a, b);
        prop_assert_eq!(h1, h2);
        prop_assert_eq!(hash_of(&h1), hash_of(&h2));
    }

    /// 44. Different indices produce different handles.
    #[test]
    fn handle_ne_different_indices(a in 0_u32..1000, b in 0_u32..1000, c in 0_u32..1000, d in 0_u32..1000) {
        prop_assume!(a != c || b != d);
        let h1 = NodeHandle::new(a, b);
        let h2 = NodeHandle::new(c, d);
        prop_assert_ne!(h1, h2);
    }

    /// 45. NodeHandle Debug contains both indices.
    #[test]
    fn handle_debug_format(a in 0_u32..100, b in 0_u32..100) {
        let h = NodeHandle::new(a, b);
        let dbg = format!("{:?}", h);
        prop_assert!(dbg.contains(&a.to_string()));
        prop_assert!(dbg.contains(&b.to_string()));
    }

    /// 46. NodeHandle works in HashSet.
    #[test]
    fn handle_hashset(vals in prop::collection::vec((0_u32..100, 0_u32..100), 1..64)) {
        let mut set = HashSet::new();
        for &(a, b) in &vals {
            set.insert(NodeHandle::new(a, b));
        }
        // Set deduplicates identical pairs
        let unique: HashSet<_> = vals.iter().copied().collect();
        prop_assert_eq!(set.len(), unique.len());
    }

    /// 47. TreeNode::leaf round-trips value/symbol.
    #[test]
    fn treenode_leaf_value(val in symbol_strategy()) {
        let n = TreeNode::leaf(val);
        prop_assert_eq!(n.value(), val);
        prop_assert_eq!(n.symbol(), val);
        prop_assert!(n.is_leaf());
        prop_assert!(!n.is_branch());
        prop_assert!(n.children().is_empty());
    }

    /// 48. TreeNode::branch has symbol 0 and given children.
    #[test]
    fn treenode_branch_default_symbol(nch in child_count()) {
        let children: Vec<_> = (0..nch).map(|i| NodeHandle::new(0, i as u32)).collect();
        let n = TreeNode::branch(children.clone());
        prop_assert_eq!(n.symbol(), 0);
        prop_assert!(n.is_branch());
        prop_assert!(!n.is_leaf());
        prop_assert_eq!(n.children(), &children[..]);
    }

    /// 49. TreeNode Clone produces equal node.
    #[test]
    fn treenode_clone_eq(val in symbol_strategy()) {
        let n = TreeNode::leaf(val);
        let n2 = n.clone();
        prop_assert_eq!(n, n2);
    }

    /// 50. TreeNode Debug doesn't panic on any value.
    #[test]
    fn treenode_debug_no_panic(val in symbol_strategy()) {
        let leaf = TreeNode::leaf(val);
        let _ = format!("{:?}", leaf);
        let branch = TreeNode::branch_with_symbol(val, vec![NodeHandle::new(0, 0)]);
        let _ = format!("{:?}", branch);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}
