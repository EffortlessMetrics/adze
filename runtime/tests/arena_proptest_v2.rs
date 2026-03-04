//! Property-based tests (v2) for TreeArena, NodeHandle, TreeNode, and ArenaMetrics.
//!
//! 60+ tests covering allocation invariants, handle validity, node construction,
//! metrics accuracy, reset/clear semantics, and edge cases.

use adze::arena_allocator::{ArenaMetrics, NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::HashSet;

// ============================================================================
// 1. TreeNode construction — leaf
// ============================================================================

#[test]
fn leaf_value_i32_min() {
    let n = TreeNode::leaf(i32::MIN);
    assert!(n.is_leaf());
    assert_eq!(n.value(), i32::MIN);
}

#[test]
fn leaf_value_i32_max() {
    let n = TreeNode::leaf(i32::MAX);
    assert!(n.is_leaf());
    assert_eq!(n.value(), i32::MAX);
}

#[test]
fn leaf_children_always_empty() {
    let n = TreeNode::leaf(7);
    assert!(n.children().is_empty());
}

#[test]
fn leaf_symbol_equals_value() {
    let n = TreeNode::leaf(-42);
    assert_eq!(n.symbol(), n.value());
}

#[test]
fn leaf_is_not_branch() {
    let n = TreeNode::leaf(0);
    assert!(!n.is_branch());
}

#[test]
fn leaf_clone_preserves_value() {
    let n = TreeNode::leaf(123);
    let c = n.clone();
    assert_eq!(n, c);
    assert_eq!(c.value(), 123);
}

// ============================================================================
// 2. TreeNode construction — branch
// ============================================================================

#[test]
fn branch_default_symbol_is_zero() {
    let n = TreeNode::branch(vec![]);
    assert_eq!(n.symbol(), 0);
}

#[test]
fn branch_with_symbol_preserves_symbol() {
    let n = TreeNode::branch_with_symbol(-99, vec![]);
    assert_eq!(n.symbol(), -99);
    assert_eq!(n.value(), -99);
}

#[test]
fn branch_is_not_leaf() {
    let n = TreeNode::branch(vec![]);
    assert!(!n.is_leaf());
}

#[test]
fn branch_children_stored_in_order() {
    let h0 = NodeHandle::new(0, 0);
    let h1 = NodeHandle::new(0, 1);
    let h2 = NodeHandle::new(0, 2);
    let n = TreeNode::branch(vec![h0, h1, h2]);
    assert_eq!(n.children(), &[h0, h1, h2]);
}

#[test]
fn branch_clone_preserves_children() {
    let h = NodeHandle::new(1, 2);
    let n = TreeNode::branch_with_symbol(5, vec![h]);
    let c = n.clone();
    assert_eq!(c.children().len(), 1);
    assert_eq!(c.symbol(), 5);
}

#[test]
fn branch_partial_eq() {
    let n1 = TreeNode::branch_with_symbol(3, vec![]);
    let n2 = TreeNode::branch_with_symbol(3, vec![]);
    assert_eq!(n1, n2);
}

#[test]
fn branch_not_equal_different_symbol() {
    let n1 = TreeNode::branch_with_symbol(1, vec![]);
    let n2 = TreeNode::branch_with_symbol(2, vec![]);
    assert_ne!(n1, n2);
}

// ============================================================================
// 3. NodeHandle traits
// ============================================================================

#[test]
fn handle_copy_semantics() {
    let h = NodeHandle::new(5, 10);
    let h2 = h; // Copy
    assert_eq!(h, h2);
}

#[test]
fn handle_hash_consistency() {
    use std::hash::{Hash, Hasher};
    let h1 = NodeHandle::new(1, 2);
    let h2 = NodeHandle::new(1, 2);
    let mut s1 = std::collections::hash_map::DefaultHasher::new();
    let mut s2 = std::collections::hash_map::DefaultHasher::new();
    h1.hash(&mut s1);
    h2.hash(&mut s2);
    assert_eq!(s1.finish(), s2.finish());
}

#[test]
fn handle_in_hashset() {
    let mut set = HashSet::new();
    set.insert(NodeHandle::new(0, 0));
    set.insert(NodeHandle::new(0, 0)); // duplicate
    set.insert(NodeHandle::new(0, 1));
    assert_eq!(set.len(), 2);
}

#[test]
fn handle_debug_contains_type_name() {
    let h = NodeHandle::new(3, 7);
    let dbg = format!("{:?}", h);
    assert!(dbg.contains("NodeHandle"));
}

// ============================================================================
// 4. Arena — basic lifecycle
// ============================================================================

#[test]
fn new_arena_has_one_chunk() {
    let a = TreeArena::new();
    assert_eq!(a.num_chunks(), 1);
}

#[test]
fn new_arena_len_zero() {
    let a = TreeArena::new();
    assert_eq!(a.len(), 0);
    assert!(a.is_empty());
}

#[test]
fn with_capacity_sets_minimum() {
    let a = TreeArena::with_capacity(256);
    assert!(a.capacity() >= 256);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn with_capacity_zero_panics() {
    let _ = TreeArena::with_capacity(0);
}

#[test]
fn default_equals_new() {
    let a = TreeArena::default();
    let b = TreeArena::new();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.capacity(), b.capacity());
    assert_eq!(a.num_chunks(), b.num_chunks());
}

// ============================================================================
// 5. Arena — allocation and retrieval
// ============================================================================

#[test]
fn alloc_returns_unique_handles() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_ne!(h1, h2);
}

#[test]
fn alloc_leaf_retrievable() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

#[test]
fn alloc_branch_children_traversable() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(10));
    let c2 = arena.alloc(TreeNode::leaf(20));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));

    let pref = arena.get(parent);
    assert_eq!(pref.children().len(), 2);
    // Follow child handles
    assert_eq!(arena.get(pref.children()[0]).value(), 10);
    assert_eq!(arena.get(pref.children()[1]).value(), 20);
}

#[test]
fn alloc_increments_len() {
    let mut arena = TreeArena::new();
    for i in 1..=5 {
        arena.alloc(TreeNode::leaf(i));
        assert_eq!(arena.len(), i as usize);
    }
}

#[test]
fn get_ref_and_as_ref_equivalent() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(77));
    let r = arena.get(h);
    assert_eq!(r.get_ref().value(), r.as_ref().value());
}

// ============================================================================
// 6. Arena — mutable access
// ============================================================================

#[test]
fn set_value_round_trip() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(0));
    arena.get_mut(h).set_value(999);
    assert_eq!(arena.get(h).value(), 999);
}

#[test]
fn set_value_on_branch_is_noop() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(5, vec![]));
    arena.get_mut(h).set_value(100);
    // set_value only affects Leaf nodes; branch symbol unchanged
    assert_eq!(arena.get(h).symbol(), 5);
}

#[test]
fn get_mut_deref_reads_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    let m = arena.get_mut(h);
    assert_eq!(m.value(), 42); // Deref to TreeNode
}

// ============================================================================
// 7. Arena — reset
// ============================================================================

#[test]
fn reset_preserves_capacity() {
    let mut arena = TreeArena::with_capacity(16);
    for i in 0..16 {
        arena.alloc(TreeNode::leaf(i));
    }
    let cap = arena.capacity();
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert_eq!(arena.capacity(), cap);
}

#[test]
fn reset_allows_reallocation() {
    let mut arena = TreeArena::new();
    let _h1 = arena.alloc(TreeNode::leaf(1));
    arena.reset();
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.len(), 1);
}

#[test]
fn multiple_resets() {
    let mut arena = TreeArena::new();
    for _ in 0..5 {
        for i in 0..10 {
            arena.alloc(TreeNode::leaf(i));
        }
        assert_eq!(arena.len(), 10);
        arena.reset();
        assert!(arena.is_empty());
    }
}

// ============================================================================
// 8. Arena — clear
// ============================================================================

#[test]
fn clear_reduces_to_one_chunk() {
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
fn clear_then_alloc_works() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
    assert_eq!(arena.len(), 1);
}

// ============================================================================
// 9. Arena — chunk growth
// ============================================================================

#[test]
fn chunk_doubles_on_overflow() {
    let mut arena = TreeArena::with_capacity(4);
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(4));
    assert_eq!(arena.num_chunks(), 2);
    // Second chunk capacity should be 2x first (8)
    assert!(arena.capacity() >= 4 + 8);
}

#[test]
fn many_chunks_all_handles_valid() {
    let mut arena = TreeArena::with_capacity(1);
    let mut handles = Vec::new();
    for i in 0..50 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert!(arena.num_chunks() > 1);
    for (i, &h) in handles.iter().enumerate() {
        assert_eq!(arena.get(h).value(), i as i32);
    }
}

// ============================================================================
// 10. ArenaMetrics
// ============================================================================

#[test]
fn metrics_on_empty_arena() {
    let a = TreeArena::new();
    let m = a.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert!(m.capacity() > 0);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn metrics_len_matches_arena_len() {
    let mut a = TreeArena::new();
    for i in 0..25 {
        a.alloc(TreeNode::leaf(i));
    }
    assert_eq!(a.metrics().len(), a.len());
}

#[test]
fn metrics_capacity_matches_arena_capacity() {
    let mut a = TreeArena::with_capacity(8);
    for i in 0..20 {
        a.alloc(TreeNode::leaf(i));
    }
    assert_eq!(a.metrics().capacity(), a.capacity());
}

#[test]
fn metrics_num_chunks_matches() {
    let mut a = TreeArena::with_capacity(3);
    for i in 0..30 {
        a.alloc(TreeNode::leaf(i));
    }
    assert_eq!(a.metrics().num_chunks(), a.num_chunks());
}

#[test]
fn metrics_memory_usage_matches() {
    let mut a = TreeArena::new();
    for i in 0..10 {
        a.alloc(TreeNode::leaf(i));
    }
    assert_eq!(a.metrics().memory_usage(), a.memory_usage());
}

#[test]
fn metrics_copy_clone() {
    let a = TreeArena::new();
    let m = a.metrics();
    let m2 = m; // Copy
    let m3 = m.clone();
    assert_eq!(m, m2);
    assert_eq!(m, m3);
}

#[test]
fn metrics_debug_format() {
    let a = TreeArena::new();
    let dbg = format!("{:?}", a.metrics());
    assert!(dbg.contains("ArenaMetrics"));
}

#[test]
fn metrics_after_reset() {
    let mut a = TreeArena::new();
    a.alloc(TreeNode::leaf(1));
    a.reset();
    let m = a.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert!(m.capacity() > 0);
}

// ============================================================================
// 11. Edge cases
// ============================================================================

#[test]
fn single_node_arena() {
    let mut a = TreeArena::new();
    let h = a.alloc(TreeNode::leaf(42));
    assert_eq!(a.len(), 1);
    assert!(!a.is_empty());
    assert_eq!(a.get(h).value(), 42);
}

#[test]
fn empty_branch_as_root() {
    let mut a = TreeArena::new();
    let h = a.alloc(TreeNode::branch(vec![]));
    assert!(a.get(h).is_branch());
    assert_eq!(a.get(h).children().len(), 0);
}

#[test]
fn branch_referencing_itself_structurally() {
    // A handle can be constructed that would refer to the node itself.
    // This is structurally valid (no runtime cycle detection).
    let mut a = TreeArena::new();
    let placeholder = a.alloc(TreeNode::leaf(0));
    // Create a branch that holds `placeholder` as a child
    let _root = a.alloc(TreeNode::branch(vec![placeholder]));
    assert_eq!(a.len(), 2);
}

#[test]
fn capacity_one_forces_many_chunks() {
    let mut a = TreeArena::with_capacity(1);
    for i in 0..10 {
        a.alloc(TreeNode::leaf(i));
    }
    assert!(a.num_chunks() >= 2);
    assert_eq!(a.len(), 10);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Invalid node handle")]
fn invalid_handle_chunk_oob() {
    let a = TreeArena::new();
    let bad = NodeHandle::new(100, 0);
    let _ = a.get(bad);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Invalid node handle")]
fn invalid_handle_node_oob() {
    let mut a = TreeArena::new();
    a.alloc(TreeNode::leaf(1));
    let bad = NodeHandle::new(0, 999);
    let _ = a.get(bad);
}

#[test]
fn tree_node_ref_is_leaf_and_is_branch() {
    let mut a = TreeArena::new();
    let lh = a.alloc(TreeNode::leaf(1));
    let bh = a.alloc(TreeNode::branch(vec![lh]));
    let lr = a.get(lh);
    let br = a.get(bh);
    assert!(lr.is_leaf());
    assert!(!lr.is_branch());
    assert!(br.is_branch());
    assert!(!br.is_leaf());
}

// ============================================================================
// 12. Property tests — TreeNode invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn prop_leaf_roundtrip(v in any::<i32>()) {
        let n = TreeNode::leaf(v);
        prop_assert!(n.is_leaf());
        prop_assert!(!n.is_branch());
        prop_assert_eq!(n.value(), v);
        prop_assert_eq!(n.symbol(), v);
        prop_assert!(n.children().is_empty());
    }

    #[test]
    fn prop_branch_symbol_roundtrip(s in any::<i32>()) {
        let n = TreeNode::branch_with_symbol(s, vec![]);
        prop_assert!(n.is_branch());
        prop_assert_eq!(n.symbol(), s);
        prop_assert_eq!(n.value(), s);
    }

    #[test]
    fn prop_leaf_clone_eq(v in any::<i32>()) {
        let n = TreeNode::leaf(v);
        prop_assert_eq!(n.clone(), n);
    }

    #[test]
    fn prop_branch_children_count(cnt in 0usize..20) {
        let handles: Vec<_> = (0..cnt).map(|i| NodeHandle::new(0, i as u32)).collect();
        let n = TreeNode::branch(handles.clone());
        prop_assert_eq!(n.children().len(), cnt);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(&n.children()[i], h);
        }
    }
}

// ============================================================================
// 13. Property tests — Arena allocation invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_alloc_count_equals_len(n in 1usize..200) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
        prop_assert!(!arena.is_empty());
    }

    #[test]
    fn prop_all_handles_unique(n in 2usize..100) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let set: HashSet<_> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn prop_all_values_retrievable(values in prop::collection::vec(any::<i32>(), 1..150)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = values.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        for (h, &v) in handles.iter().zip(values.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    #[test]
    fn prop_capacity_ge_len(n in 1usize..300) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    #[test]
    fn prop_capacity_monotonically_grows(n in 1usize..200) {
        let mut arena = TreeArena::new();
        let mut prev_cap = arena.capacity();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cap = arena.capacity();
            prop_assert!(cap >= prev_cap);
            prev_cap = cap;
        }
    }
}

// ============================================================================
// 14. Property tests — reset/clear semantics
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_reset_zeroes_len(n in 1usize..100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert_eq!(arena.len(), 0);
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn prop_reset_preserves_capacity(n in 1usize..100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let cap = arena.capacity();
        arena.reset();
        prop_assert_eq!(arena.capacity(), cap);
    }

    #[test]
    fn prop_clear_leaves_one_chunk(n in 1usize..100) {
        let mut arena = TreeArena::with_capacity(2);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn prop_reset_then_realloc_same_count(n in 1usize..80) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let cap_before = arena.capacity();
        arena.reset();

        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32 + 1000)))
            .collect();
        // Capacity should not grow when reallocating same count
        prop_assert_eq!(arena.capacity(), cap_before);
        for (i, &h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(h).value(), i as i32 + 1000);
        }
    }
}

// ============================================================================
// 15. Property tests — metrics consistency
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_metrics_len_consistent(n in 0usize..100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), arena.len());
        prop_assert_eq!(m.len(), n);
    }

    #[test]
    fn prop_metrics_capacity_consistent(n in 0usize..100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.capacity(), arena.capacity());
        prop_assert!(m.capacity() >= m.len());
    }

    #[test]
    fn prop_metrics_num_chunks_consistent(n in 0usize..100) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.num_chunks(), arena.num_chunks());
        prop_assert!(m.num_chunks() >= 1);
    }

    #[test]
    fn prop_metrics_memory_positive(n in 0usize..50) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.metrics().memory_usage() > 0);
    }

    #[test]
    fn prop_metrics_is_empty_iff_len_zero(n in 0usize..50) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.is_empty(), m.len() == 0);
    }
}

// ============================================================================
// 16. Property tests — with_capacity
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_with_capacity_ge_requested(cap in 1usize..500) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.capacity() >= cap);
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    #[test]
    fn prop_with_capacity_fill_exact(cap in 1usize..50) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..cap {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        // Exactly at capacity — still one chunk
        prop_assert_eq!(arena.num_chunks(), 1);
        prop_assert_eq!(arena.len(), cap);
    }

    #[test]
    fn prop_with_capacity_overflow_adds_chunk(cap in 1usize..50) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..=cap {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.num_chunks(), 2);
    }
}

// ============================================================================
// 17. Property tests — tree structure
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_wide_tree_children_match(width in 0usize..40) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..width)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let root = arena.alloc(TreeNode::branch(children.clone()));
        let r = arena.get(root);
        prop_assert_eq!(r.children().len(), width);
        for (i, &ch) in r.children().iter().enumerate() {
            prop_assert_eq!(arena.get(ch).value(), i as i32);
        }
    }

    #[test]
    fn prop_deep_chain(depth in 1usize..30) {
        let mut arena = TreeArena::new();
        let mut current = arena.alloc(TreeNode::leaf(0));
        for i in 1..depth {
            current = arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![current]));
        }
        prop_assert_eq!(arena.len(), depth);
        let root = arena.get(current);
        if depth > 1 {
            prop_assert!(root.is_branch());
            prop_assert_eq!(root.children().len(), 1);
        }
    }

    #[test]
    fn prop_set_value_roundtrip(original in any::<i32>(), updated in any::<i32>()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(original));
        prop_assert_eq!(arena.get(h).value(), original);
        arena.get_mut(h).set_value(updated);
        prop_assert_eq!(arena.get(h).value(), updated);
    }
}

// ============================================================================
// 18. Property tests — NodeHandle
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_handle_eq_reflexive(c in 0u32..100, n in 0u32..100) {
        let h = NodeHandle::new(c, n);
        prop_assert_eq!(h, h);
    }

    #[test]
    fn prop_handle_eq_symmetric(c in 0u32..100, n in 0u32..100) {
        let h1 = NodeHandle::new(c, n);
        let h2 = NodeHandle::new(c, n);
        prop_assert_eq!(h1, h2);
        prop_assert_eq!(h2, h1);
    }

    #[test]
    fn prop_handle_ne_different_chunk(c1 in 0u32..50, c2 in 50u32..100, n in 0u32..100) {
        let h1 = NodeHandle::new(c1, n);
        let h2 = NodeHandle::new(c2, n);
        prop_assert_ne!(h1, h2);
    }

    #[test]
    fn prop_handle_ne_different_node(c in 0u32..100, n1 in 0u32..50, n2 in 50u32..100) {
        let h1 = NodeHandle::new(c, n1);
        let h2 = NodeHandle::new(c, n2);
        prop_assert_ne!(h1, h2);
    }
}

// ============================================================================
// 19. Stress / combined scenarios
// ============================================================================

#[test]
fn stress_alloc_reset_cycles() {
    let mut arena = TreeArena::with_capacity(8);
    for cycle in 0..10 {
        let base = cycle * 100;
        let handles: Vec<_> = (0..50)
            .map(|i| arena.alloc(TreeNode::leaf(base + i)))
            .collect();
        for (i, &h) in handles.iter().enumerate() {
            assert_eq!(arena.get(h).value(), base + i as i32);
        }
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn stress_mixed_leaf_branch() {
    let mut arena = TreeArena::new();
    let mut all_leaves = Vec::new();
    for i in 0..100 {
        let h = arena.alloc(TreeNode::leaf(i));
        all_leaves.push(h);
    }
    // Create branches grouping every 10 leaves
    let mut roots = Vec::new();
    for chunk in all_leaves.chunks(10) {
        let root = arena.alloc(TreeNode::branch_with_symbol(
            roots.len() as i32,
            chunk.to_vec(),
        ));
        roots.push(root);
    }
    assert_eq!(roots.len(), 10);
    for (i, &r) in roots.iter().enumerate() {
        let node = arena.get(r);
        assert!(node.is_branch());
        assert_eq!(node.children().len(), 10);
        assert_eq!(node.symbol(), i as i32);
    }
}

#[test]
fn stress_clear_and_rebuild() {
    let mut arena = TreeArena::with_capacity(4);
    for _ in 0..5 {
        for i in 0..30 {
            arena.alloc(TreeNode::leaf(i));
        }
        arena.clear();
        assert_eq!(arena.num_chunks(), 1);
        assert!(arena.is_empty());
    }
    // Final allocation after clears
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn memory_usage_grows_with_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    let m0 = arena.memory_usage();
    // Fill first chunk
    arena.alloc(TreeNode::leaf(0));
    arena.alloc(TreeNode::leaf(1));
    // Trigger second chunk
    arena.alloc(TreeNode::leaf(2));
    let m1 = arena.memory_usage();
    assert!(m1 > m0);
}

#[test]
fn arena_debug_format() {
    let a = TreeArena::new();
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("TreeArena"));
}
