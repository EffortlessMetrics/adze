//! Property-based + unit tests for TreeArena (deep v2).
//!
//! 60+ tests covering capacity properties, allocation invariants,
//! handle semantics, node construction, metrics, reset/clear,
//! multi-arena independence, and stress scenarios.

use adze::arena_allocator::{ArenaMetrics, NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::HashSet;

// ═══════════════════════════════════════════════════════════════════════════
// 1. Arena capacity properties (proptest)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Any capacity > 0 produces a valid arena.
    #[test]
    fn prop_any_positive_capacity_is_valid(cap in 1..=50_000usize) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
        prop_assert!(arena.capacity() >= cap);
    }

    /// Capacity is always >= initial.
    #[test]
    fn prop_capacity_ge_initial(cap in 1..=10_000usize) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.capacity() >= cap);
    }

    /// Memory usage is positive for any valid arena.
    #[test]
    fn prop_memory_usage_positive(cap in 1..=10_000usize) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.memory_usage() > 0);
    }

    /// Freshly created arena starts with exactly 1 chunk.
    #[test]
    fn prop_initial_chunks_eq_one(cap in 1..=10_000usize) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert_eq!(arena.num_chunks(), 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Arena with minimum capacity (1)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_min_capacity_arena_is_empty() {
    let arena = TreeArena::with_capacity(1);
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn unit_min_capacity_single_alloc() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn unit_min_capacity_triggers_growth() {
    let mut arena = TreeArena::with_capacity(1);
    let _h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
    assert!(arena.num_chunks() >= 2);
    assert_eq!(arena.get(h2).value(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Arena with large capacity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_large_capacity_no_growth() {
    let mut arena = TreeArena::with_capacity(10_000);
    for i in 0..10_000 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 10_000);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn unit_large_capacity_memory() {
    let arena = TreeArena::with_capacity(10_000);
    assert!(arena.memory_usage() > 0);
    assert!(arena.capacity() >= 10_000);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Allocation invariants (proptest)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Len grows by 1 per allocation.
    #[test]
    fn prop_len_grows_monotonically(n in 1..500usize) {
        let mut arena = TreeArena::with_capacity(16);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            prop_assert_eq!(arena.len(), i + 1);
        }
    }

    /// Every handle returned by alloc is retrievable.
    #[test]
    fn prop_handles_always_valid(n in 1..300usize) {
        let mut arena = TreeArena::with_capacity(8);
        let mut handles = Vec::new();
        for i in 0..n {
            handles.push(arena.alloc(TreeNode::leaf(i as i32)));
        }
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    /// All handles from a batch are unique.
    #[test]
    fn prop_handles_unique(n in 1..200usize) {
        let mut arena = TreeArena::with_capacity(4);
        let mut set = HashSet::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(set.insert(h), "duplicate handle at index {}", i);
        }
    }

    /// capacity() >= len() always.
    #[test]
    fn prop_capacity_ge_len(n in 1..500usize) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    /// Arena is not empty after at least one allocation.
    #[test]
    fn prop_not_empty_after_alloc(n in 1..100usize) {
        let mut arena = TreeArena::with_capacity(8);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(!arena.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. NodeHandle: Copy, Clone, Debug, Eq, Hash
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_handle_copy_semantics() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(7));
    let h_copy = h; // Copy
    assert_eq!(arena.get(h).value(), arena.get(h_copy).value());
}

#[test]
fn unit_handle_clone_semantics() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(99));
    #[allow(clippy::clone_on_copy)]
    let h_clone = h.clone();
    assert_eq!(h, h_clone);
    assert_eq!(arena.get(h_clone).value(), 99);
}

#[test]
fn unit_handle_debug_format() {
    let h = NodeHandle::new(0, 0);
    let dbg = format!("{:?}", h);
    assert!(dbg.contains("NodeHandle"));
}

#[test]
fn unit_handle_eq_and_hash() {
    let a = NodeHandle::new(1, 2);
    let b = NodeHandle::new(1, 2);
    let c = NodeHandle::new(1, 3);
    assert_eq!(a, b);
    assert_ne!(a, c);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&b));
    assert!(!set.contains(&c));
}

proptest! {
    /// Copied handle always yields same node value.
    #[test]
    fn prop_handle_copy_preserves_value(val in -10_000i32..10_000) {
        let mut arena = TreeArena::with_capacity(4);
        let h = arena.alloc(TreeNode::leaf(val));
        let h2 = h;
        prop_assert_eq!(arena.get(h).value(), arena.get(h2).value());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. TreeNode construction & queries
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Leaf node stores its value.
    #[test]
    fn prop_leaf_stores_value(v in any::<i32>()) {
        let node = TreeNode::leaf(v);
        prop_assert_eq!(node.value(), v);
        prop_assert!(node.is_leaf());
        prop_assert!(!node.is_branch());
        prop_assert!(node.children().is_empty());
    }

    /// Branch with symbol stores symbol and children.
    #[test]
    fn prop_branch_stores_symbol(sym in any::<i32>()) {
        let node = TreeNode::branch_with_symbol(sym, vec![]);
        prop_assert_eq!(node.symbol(), sym);
        prop_assert!(node.is_branch());
        prop_assert!(!node.is_leaf());
    }
}

#[test]
fn unit_branch_default_symbol_is_zero() {
    let node = TreeNode::branch(vec![]);
    assert_eq!(node.symbol(), 0);
    assert!(node.is_branch());
}

#[test]
fn unit_branch_children_are_preserved() {
    let mut arena = TreeArena::with_capacity(8);
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    let parent_ref = arena.get(parent);
    let children = parent_ref.children();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], c1);
    assert_eq!(children[1], c2);
}

#[test]
fn unit_leaf_value_alias() {
    let node = TreeNode::leaf(55);
    assert_eq!(node.value(), node.symbol());
}

#[test]
fn unit_tree_node_clone() {
    let node = TreeNode::leaf(10);
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

#[test]
fn unit_tree_node_debug() {
    let node = TreeNode::leaf(7);
    let dbg = format!("{:?}", node);
    assert!(!dbg.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Mutable access (set_value)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// set_value changes the stored value.
    #[test]
    fn prop_set_value_roundtrip(orig in any::<i32>(), new_val in any::<i32>()) {
        let mut arena = TreeArena::with_capacity(4);
        let h = arena.alloc(TreeNode::leaf(orig));
        arena.get_mut(h).set_value(new_val);
        prop_assert_eq!(arena.get(h).value(), new_val);
    }
}

#[test]
fn unit_set_value_on_branch_is_noop() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::branch(vec![]));
    let original = arena.get(h).value();
    arena.get_mut(h).set_value(999);
    assert_eq!(arena.get(h).value(), original);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Reset / Clear semantics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_reset_empties_arena() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(!arena.is_empty());
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn unit_reset_preserves_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn unit_clear_shrinks_to_one_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
    assert!(arena.is_empty());
}

#[test]
fn unit_alloc_after_reset() {
    let mut arena = TreeArena::with_capacity(4);
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
}

#[test]
fn unit_alloc_after_clear() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(77));
    assert_eq!(arena.get(h).value(), 77);
    assert_eq!(arena.len(), 1);
}

proptest! {
    /// After reset, arena is always empty with len 0.
    #[test]
    fn prop_reset_always_empty(n in 1..300usize) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    /// After clear, num_chunks is 1.
    #[test]
    fn prop_clear_one_chunk(n in 1..300usize) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
        prop_assert!(arena.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Metrics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_metrics_empty_arena() {
    let arena = TreeArena::with_capacity(16);
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert!(m.capacity() >= 16);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn unit_metrics_after_allocs() {
    let mut arena = TreeArena::with_capacity(8);
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.len(), 5);
    assert!(!m.is_empty());
}

#[test]
fn unit_metrics_copy_clone() {
    let arena = TreeArena::with_capacity(4);
    let m = arena.metrics();
    let m2 = m;
    #[allow(clippy::clone_on_copy)]
    let m3 = m.clone();
    assert_eq!(m, m2);
    assert_eq!(m, m3);
}

#[test]
fn unit_metrics_debug() {
    let arena = TreeArena::with_capacity(4);
    let dbg = format!("{:?}", arena.metrics());
    assert!(dbg.contains("ArenaMetrics"));
}

proptest! {
    /// Metrics len matches arena len.
    #[test]
    fn prop_metrics_len_matches(n in 0..200usize) {
        let mut arena = TreeArena::with_capacity(8);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), arena.len());
        prop_assert_eq!(m.capacity(), arena.capacity());
        prop_assert_eq!(m.num_chunks(), arena.num_chunks());
        prop_assert_eq!(m.memory_usage(), arena.memory_usage());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Multiple independent arenas
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_two_arenas_independent() {
    let mut a1 = TreeArena::with_capacity(4);
    let mut a2 = TreeArena::with_capacity(4);
    let h1 = a1.alloc(TreeNode::leaf(10));
    let h2 = a2.alloc(TreeNode::leaf(20));
    assert_eq!(a1.get(h1).value(), 10);
    assert_eq!(a2.get(h2).value(), 20);
    assert_eq!(a1.len(), 1);
    assert_eq!(a2.len(), 1);
}

#[test]
fn unit_reset_one_arena_not_other() {
    let mut a1 = TreeArena::with_capacity(4);
    let mut a2 = TreeArena::with_capacity(4);
    let _h1 = a1.alloc(TreeNode::leaf(1));
    let h2 = a2.alloc(TreeNode::leaf(2));
    a1.reset();
    assert!(a1.is_empty());
    assert!(!a2.is_empty());
    assert_eq!(a2.get(h2).value(), 2);
}

proptest! {
    /// Two arenas with different capacities remain independent.
    #[test]
    fn prop_independent_arenas(
        cap1 in 1..500usize,
        cap2 in 1..500usize,
        n1 in 0..50usize,
        n2 in 0..50usize,
    ) {
        let mut a1 = TreeArena::with_capacity(cap1);
        let mut a2 = TreeArena::with_capacity(cap2);
        for i in 0..n1 {
            a1.alloc(TreeNode::leaf(i as i32));
        }
        for i in 0..n2 {
            a2.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(a1.len(), n1);
        prop_assert_eq!(a2.len(), n2);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Chunk growth
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Exceeding initial capacity triggers additional chunks.
    #[test]
    fn prop_growth_adds_chunks(cap in 1..64usize, extra in 1..200usize) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..(cap + extra) {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.num_chunks() >= 2);
        prop_assert_eq!(arena.len(), cap + extra);
    }

    /// Capacity never shrinks during allocation.
    #[test]
    fn prop_capacity_monotonic(n in 1..300usize) {
        let mut arena = TreeArena::with_capacity(4);
        let mut prev_cap = arena.capacity();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cur_cap = arena.capacity();
            prop_assert!(cur_cap >= prev_cap);
            prev_cap = cur_cap;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Deep tree construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_deep_linear_tree() {
    let mut arena = TreeArena::with_capacity(64);
    let mut prev = arena.alloc(TreeNode::leaf(0));
    for i in 1..50 {
        prev = arena.alloc(TreeNode::branch_with_symbol(i, vec![prev]));
    }
    assert_eq!(arena.len(), 50);
    assert_eq!(arena.get(prev).symbol(), 49);
    assert_eq!(arena.get(prev).children().len(), 1);
}

#[test]
fn unit_wide_tree() {
    let mut arena = TreeArena::with_capacity(64);
    let leaves: Vec<_> = (0..20).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(leaves.clone()));
    assert_eq!(arena.get(root).children().len(), 20);
    for (idx, &child) in arena.get(root).children().iter().enumerate() {
        // Need to drop the ref first because of borrow rules —
        // just compare via handle stored in leaves.
        assert_eq!(child, leaves[idx]);
    }
}

proptest! {
    /// Build tree of random depth; all handles remain valid.
    #[test]
    fn prop_random_depth_tree(depth in 1..80usize) {
        let mut arena = TreeArena::with_capacity(8);
        let mut cur = arena.alloc(TreeNode::leaf(0));
        for d in 1..depth {
            cur = arena.alloc(TreeNode::branch_with_symbol(d as i32, vec![cur]));
        }
        prop_assert_eq!(arena.len(), depth);
        prop_assert_eq!(arena.get(cur).symbol(), (depth - 1) as i32);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Stress / large-scale
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_stress_many_allocs() {
    let mut arena = TreeArena::with_capacity(16);
    let mut handles = Vec::with_capacity(5000);
    for i in 0..5000 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert_eq!(arena.len(), 5000);
    // spot-check
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[2500]).value(), 2500);
    assert_eq!(arena.get(handles[4999]).value(), 4999);
}

#[test]
fn unit_stress_reset_reuse_cycles() {
    let mut arena = TreeArena::with_capacity(8);
    for _cycle in 0..10 {
        for i in 0..100 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn unit_stress_clear_reuse_cycles() {
    let mut arena = TreeArena::with_capacity(8);
    for _cycle in 0..10 {
        for i in 0..100 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.clear();
        assert!(arena.is_empty());
        assert_eq!(arena.num_chunks(), 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Default trait
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_default_arena() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn unit_default_and_new_equivalent() {
    let a = TreeArena::new();
    let b = TreeArena::default();
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.len(), b.len());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. TreeNodeRef / Deref
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_node_ref_deref_leaf() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(5));
    let r = arena.get(h);
    // Deref lets us call TreeNode methods directly
    assert!(r.is_leaf());
    assert_eq!(r.symbol(), 5);
    assert_eq!(r.value(), 5);
    assert!(r.children().is_empty());
}

#[test]
fn unit_node_ref_deref_branch() {
    let mut arena = TreeArena::with_capacity(4);
    let c = arena.alloc(TreeNode::leaf(1));
    let h = arena.alloc(TreeNode::branch_with_symbol(42, vec![c]));
    let r = arena.get(h);
    assert!(r.is_branch());
    assert_eq!(r.symbol(), 42);
    assert_eq!(r.children().len(), 1);
}

#[test]
fn unit_node_ref_get_ref() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(3));
    let r = arena.get(h);
    let node: &TreeNode = r.get_ref();
    assert_eq!(node.value(), 3);
}

#[test]
fn unit_node_ref_as_ref_alias() {
    let mut arena = TreeArena::with_capacity(4);
    let h = arena.alloc(TreeNode::leaf(8));
    let r = arena.get(h);
    assert_eq!(r.get_ref().value(), r.as_ref().value());
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Arena debug formatting
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_arena_debug() {
    let arena = TreeArena::with_capacity(4);
    let dbg = format!("{:?}", arena);
    assert!(dbg.contains("TreeArena"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Mixed allocation patterns (proptest)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Interleaved leaves and branches preserve count.
    #[test]
    fn prop_mixed_alloc_count(n in 1..100usize) {
        let mut arena = TreeArena::with_capacity(8);
        let mut count = 0usize;
        for i in 0..n {
            if i % 3 == 0 {
                arena.alloc(TreeNode::branch(vec![]));
            } else {
                arena.alloc(TreeNode::leaf(i as i32));
            }
            count += 1;
        }
        prop_assert_eq!(arena.len(), count);
    }

    /// Branches referencing earlier handles are retrievable.
    #[test]
    fn prop_back_references(n in 2..80usize) {
        let mut arena = TreeArena::with_capacity(8);
        let first = arena.alloc(TreeNode::leaf(0));
        for i in 1..n {
            arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![first]));
        }
        // first handle still valid
        prop_assert_eq!(arena.get(first).value(), 0);
        prop_assert_eq!(arena.len(), n);
    }

    /// Random symbol values stored and retrieved correctly.
    #[test]
    fn prop_random_symbols(vals in prop::collection::vec(any::<i32>(), 1..100)) {
        let mut arena = TreeArena::with_capacity(8);
        let handles: Vec<_> = vals.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        for (i, &h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(h).value(), vals[i]);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unit_leaf_extreme_values() {
    let mut arena = TreeArena::with_capacity(4);
    let h_min = arena.alloc(TreeNode::leaf(i32::MIN));
    let h_max = arena.alloc(TreeNode::leaf(i32::MAX));
    let h_zero = arena.alloc(TreeNode::leaf(0));
    assert_eq!(arena.get(h_min).value(), i32::MIN);
    assert_eq!(arena.get(h_max).value(), i32::MAX);
    assert_eq!(arena.get(h_zero).value(), 0);
}

#[test]
fn unit_branch_with_no_children() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(node.children().is_empty());
}

#[test]
fn unit_branch_with_symbol_negative() {
    let node = TreeNode::branch_with_symbol(-1, vec![]);
    assert_eq!(node.symbol(), -1);
}

#[test]
fn unit_reset_on_empty_arena() {
    let mut arena = TreeArena::with_capacity(4);
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn unit_clear_on_empty_arena() {
    let mut arena = TreeArena::with_capacity(4);
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn unit_double_reset() {
    let mut arena = TreeArena::with_capacity(4);
    arena.alloc(TreeNode::leaf(1));
    arena.reset();
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn unit_double_clear() {
    let mut arena = TreeArena::with_capacity(4);
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    arena.clear();
    assert!(arena.is_empty());
    assert_eq!(arena.num_chunks(), 1);
}
