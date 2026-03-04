//! Property-based tests (v3) for TreeArena.
//!
//! 55 tests covering allocation, handle uniqueness, mutation persistence,
//! metrics consistency, reset/clear semantics, chunk growth, and stress.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Helpers
// ============================================================================

fn arb_capacity() -> impl Strategy<Value = usize> {
    1usize..=512
}

fn arb_symbol() -> impl Strategy<Value = i32> {
    prop::num::i32::ANY
}

fn arb_node_count() -> impl Strategy<Value = usize> {
    1usize..=200
}

// ============================================================================
// 1. Allocation always succeeds for non-zero arenas (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn pt01_alloc_leaf_succeeds(cap in arb_capacity(), val in arb_symbol()) {
        let mut arena = TreeArena::with_capacity(cap);
        let _h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.len(), 1);
    }

    #[test]
    fn pt02_alloc_branch_succeeds(cap in arb_capacity(), sym in arb_symbol()) {
        let mut arena = TreeArena::with_capacity(cap);
        let _h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.len(), 1);
    }

    #[test]
    fn pt03_alloc_n_nodes_succeeds(cap in arb_capacity(), n in arb_node_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }
}

// ============================================================================
// 2. Get after alloc returns same node data (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn pt04_get_leaf_value_roundtrip(val in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
    }

    #[test]
    fn pt05_get_branch_symbol_roundtrip(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    #[test]
    fn pt06_get_leaf_is_leaf(val in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }

    #[test]
    fn pt07_get_branch_is_branch(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }

    #[test]
    fn pt08_get_preserves_children(n in 1usize..=20) {
        let mut arena = TreeArena::new();
        let children: Vec<NodeHandle> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        let parent_ref = arena.get(parent);
        prop_assert_eq!(parent_ref.children(), &children[..]);
    }

    #[test]
    fn pt09_all_values_retrievable(vals in prop::collection::vec(arb_symbol(), 1..50)) {
        let mut arena = TreeArena::new();
        let handles: Vec<NodeHandle> = vals.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }
}

// ============================================================================
// 3. Multiple allocs produce unique handles (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn pt10_handles_unique_small(n in 2usize..=50) {
        let mut arena = TreeArena::new();
        let handles: Vec<NodeHandle> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let set: HashSet<NodeHandle> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn pt11_handles_unique_across_chunks(cap in 1usize..=4, n in 5usize..=30) {
        let mut arena = TreeArena::with_capacity(cap);
        let handles: Vec<NodeHandle> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let set: HashSet<NodeHandle> = handles.iter().copied().collect();
        prop_assert_eq!(set.len(), n);
    }

    #[test]
    fn pt12_handles_not_equal_different_allocs(a in arb_symbol(), b in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(a));
        let h2 = arena.alloc(TreeNode::leaf(b));
        prop_assert_ne!(h1, h2);
    }

    #[test]
    fn pt13_handle_copy_semantics(val in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        let h_copy = h;
        prop_assert_eq!(h, h_copy);
        prop_assert_eq!(arena.get(h).value(), arena.get(h_copy).value());
    }
}

// ============================================================================
// 4. get_mut modifications persist (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn pt14_set_value_persists(orig in arb_symbol(), new_val in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(orig));
        arena.get_mut(h).set_value(new_val);
        prop_assert_eq!(arena.get(h).value(), new_val);
    }

    #[test]
    fn pt15_mutation_does_not_affect_other_nodes(a in arb_symbol(), b in arb_symbol(), new_val in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h1 = arena.alloc(TreeNode::leaf(a));
        let h2 = arena.alloc(TreeNode::leaf(b));
        arena.get_mut(h1).set_value(new_val);
        prop_assert_eq!(arena.get(h1).value(), new_val);
        prop_assert_eq!(arena.get(h2).value(), b);
    }

    #[test]
    fn pt16_multiple_mutations_last_wins(val in arb_symbol(), updates in prop::collection::vec(arb_symbol(), 2..10)) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(val));
        for &u in &updates {
            arena.get_mut(h).set_value(u);
        }
        prop_assert_eq!(arena.get(h).value(), *updates.last().unwrap());
    }

    #[test]
    fn pt17_mutate_across_chunks(n in 5usize..=30, new_val in arb_symbol()) {
        let mut arena = TreeArena::with_capacity(2);
        let handles: Vec<NodeHandle> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        // Mutate last handle (likely in a later chunk)
        let last = *handles.last().unwrap();
        arena.get_mut(last).set_value(new_val);
        prop_assert_eq!(arena.get(last).value(), new_val);
        // First handle unaffected
        prop_assert_eq!(arena.get(handles[0]).value(), 0);
    }
}

// ============================================================================
// 5. Large arena stress tests (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn pt18_stress_1000_nodes(cap in 1usize..=16) {
        let mut arena = TreeArena::with_capacity(cap);
        let mut handles = Vec::with_capacity(1000);
        for i in 0..1000 {
            handles.push(arena.alloc(TreeNode::leaf(i)));
        }
        prop_assert_eq!(arena.len(), 1000);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn pt19_stress_deep_tree(depth in 10usize..=100) {
        let mut arena = TreeArena::new();
        let mut current = arena.alloc(TreeNode::leaf(0));
        for i in 1..=depth {
            current = arena.alloc(TreeNode::branch_with_symbol(i as i32, vec![current]));
        }
        prop_assert!(arena.get(current).is_branch());
        prop_assert_eq!(arena.get(current).symbol(), depth as i32);
        prop_assert_eq!(arena.len(), depth + 1);
    }

    #[test]
    fn pt20_stress_wide_tree(width in 10usize..=200) {
        let mut arena = TreeArena::new();
        let children: Vec<NodeHandle> = (0..width)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let root = arena.alloc(TreeNode::branch(children.clone()));
        prop_assert_eq!(arena.get(root).children().len(), width);
        prop_assert_eq!(arena.len(), width + 1);
    }
}

// ============================================================================
// 6. Metrics consistency (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn pt21_capacity_ge_len(cap in arb_capacity(), n in arb_node_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    #[test]
    fn pt22_num_chunks_ge_one(cap in arb_capacity(), n in 0usize..=100) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.num_chunks() >= 1);
    }

    #[test]
    fn pt23_memory_usage_positive_after_alloc(cap in arb_capacity()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.memory_usage() > 0);
    }

    #[test]
    fn pt24_metrics_snapshot_matches_direct(cap in arb_capacity(), n in arb_node_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), arena.len());
        prop_assert_eq!(m.capacity(), arena.capacity());
        prop_assert_eq!(m.num_chunks(), arena.num_chunks());
        prop_assert_eq!(m.memory_usage(), arena.memory_usage());
    }

    #[test]
    fn pt25_is_empty_iff_len_zero(cap in arb_capacity(), n in 0usize..=10) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.is_empty(), arena.len() == 0);
    }
}

// ============================================================================
// 7. Reset / clear semantics (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn pt26_reset_empties_arena(n in arb_node_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn pt27_clear_empties_arena(n in arb_node_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn pt28_clear_leaves_one_chunk(cap in arb_capacity(), n in arb_node_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    #[test]
    fn pt29_reset_preserves_chunks(cap in 1usize..=4, n in 10usize..=50) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let chunks_before = arena.num_chunks();
        arena.reset();
        prop_assert_eq!(arena.num_chunks(), chunks_before);
    }

    #[test]
    fn pt30_alloc_after_reset_works(n in arb_node_count(), val in arb_symbol()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
        prop_assert_eq!(arena.len(), 1);
    }

    #[test]
    fn pt31_alloc_after_clear_works(n in arb_node_count(), val in arb_symbol()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
        prop_assert_eq!(arena.len(), 1);
    }
}

// ============================================================================
// 8. Chunk growth invariants (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn pt32_chunks_grow_when_full(cap in 1usize..=8) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..cap {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.num_chunks(), 1);
        arena.alloc(TreeNode::leaf(999));
        prop_assert_eq!(arena.num_chunks(), 2);
    }

    #[test]
    fn pt33_capacity_monotonically_increases(cap in 1usize..=8, n in 1usize..=100) {
        let mut arena = TreeArena::with_capacity(cap);
        let mut prev_cap = arena.capacity();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cur_cap = arena.capacity();
            prop_assert!(cur_cap >= prev_cap);
            prev_cap = cur_cap;
        }
    }
}

// ============================================================================
// 9. NodeHandle properties (unit tests)
// ============================================================================

#[test]
fn ut01_handle_new_roundtrip() {
    let h = NodeHandle::new(3, 7);
    // Verify via arena that the indices matter
    assert_eq!(h, NodeHandle::new(3, 7));
}

#[test]
fn ut02_handle_equality() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    assert_eq!(h1, h2);
}

#[test]
fn ut03_handle_inequality_chunk() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(1, 0);
    assert_ne!(h1, h2);
}

#[test]
fn ut04_handle_inequality_node() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    assert_ne!(h1, h2);
}

#[test]
fn ut05_handle_copy() {
    let h = NodeHandle::new(2, 5);
    let h2 = h;
    assert_eq!(h, h2);
}

#[test]
fn ut06_handle_clone() {
    let h = NodeHandle::new(2, 5);
    #[allow(clippy::clone_on_copy)]
    let h2 = h.clone();
    assert_eq!(h, h2);
}

#[test]
fn ut07_handle_debug_format() {
    let h = NodeHandle::new(1, 2);
    let dbg = format!("{:?}", h);
    assert!(dbg.contains("NodeHandle"));
}

#[test]
fn ut08_handle_hash_consistent() {
    use std::collections::HashMap;
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    let mut map = HashMap::new();
    map.insert(h, 100);
    assert_eq!(map[&h], 100);
}

// ============================================================================
// 10. TreeNode construction (unit tests)
// ============================================================================

#[test]
fn ut09_leaf_zero() {
    let n = TreeNode::leaf(0);
    assert!(n.is_leaf());
    assert_eq!(n.value(), 0);
    assert_eq!(n.symbol(), 0);
    assert!(n.children().is_empty());
}

#[test]
fn ut10_leaf_negative() {
    let n = TreeNode::leaf(-1);
    assert_eq!(n.value(), -1);
}

#[test]
fn ut11_branch_empty_children() {
    let n = TreeNode::branch(vec![]);
    assert!(n.is_branch());
    assert!(n.children().is_empty());
    assert_eq!(n.symbol(), 0);
}

#[test]
fn ut12_branch_with_symbol_custom() {
    let n = TreeNode::branch_with_symbol(42, vec![]);
    assert!(n.is_branch());
    assert_eq!(n.symbol(), 42);
}

#[test]
fn ut13_node_clone_eq() {
    let n = TreeNode::leaf(99);
    let c = n.clone();
    assert_eq!(n, c);
}

#[test]
fn ut14_branch_children_stored() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let n = TreeNode::branch(vec![h1, h2]);
    assert_eq!(n.children(), &[h1, h2]);
}

// ============================================================================
// 11. Arena construction edge cases (unit tests)
// ============================================================================

#[test]
fn ut15_new_default_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn ut16_default_trait() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn ut17_zero_capacity_panics() {
    let _arena = TreeArena::with_capacity(0);
}

#[test]
fn ut18_capacity_one() {
    let mut arena = TreeArena::with_capacity(1);
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h).value(), 1);
    // Second alloc forces new chunk
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.num_chunks(), 2);
}

// ============================================================================
// 12. TreeNodeRef / TreeNodeRefMut (unit tests)
// ============================================================================

#[test]
fn ut19_ref_deref_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(77));
    let node_ref = arena.get(h);
    // Deref gives access to TreeNode methods
    assert_eq!(node_ref.value(), 77);
    assert!(node_ref.is_leaf());
}

#[test]
fn ut20_ref_children_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn ut21_ref_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn ut22_set_value_on_branch_is_noop() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(5, vec![]));
    arena.get_mut(h).set_value(99);
    // Branch symbol unchanged; set_value only mutates Leaf variant
    assert_eq!(arena.get(h).symbol(), 5);
}

// ============================================================================
// 13. Metrics (unit tests)
// ============================================================================

#[test]
fn ut23_empty_metrics() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert!(m.capacity() > 0);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn ut24_metrics_after_allocs() {
    let mut arena = TreeArena::new();
    for i in 0..5 {
        arena.alloc(TreeNode::leaf(i));
    }
    let m = arena.metrics();
    assert_eq!(m.len(), 5);
    assert!(!m.is_empty());
}

// ============================================================================
// 14. Mixed proptest: combining properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn pt34_alloc_get_mutate_get_cycle(
        vals in prop::collection::vec(arb_symbol(), 2..20),
        target_idx in 0usize..20,
        new_val in arb_symbol(),
    ) {
        let n = vals.len();
        let target_idx = target_idx % n;

        let mut arena = TreeArena::new();
        let handles: Vec<NodeHandle> = vals.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();

        // Verify original values
        for (i, &h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(h).value(), vals[i]);
        }

        // Mutate one
        arena.get_mut(handles[target_idx]).set_value(new_val);
        prop_assert_eq!(arena.get(handles[target_idx]).value(), new_val);

        // Others unchanged
        for (i, &h) in handles.iter().enumerate() {
            if i != target_idx {
                prop_assert_eq!(arena.get(h).value(), vals[i]);
            }
        }
    }

    #[test]
    fn pt35_reset_then_realloc_same_count(n in 1usize..=50) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();

        let mut handles = Vec::new();
        for i in 0..n {
            handles.push(arena.alloc(TreeNode::leaf((i as i32) + 1000)));
        }
        prop_assert_eq!(arena.len(), n);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), (i as i32) + 1000);
        }
    }

    #[test]
    fn pt36_branch_children_integrity(n in 1usize..=30) {
        let mut arena = TreeArena::new();
        let leaves: Vec<NodeHandle> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let root = arena.alloc(TreeNode::branch(leaves.clone()));

        let root_ref = arena.get(root);
        let retrieved = root_ref.children().to_vec();
        prop_assert_eq!(retrieved.len(), n);
        for (i, &child_h) in retrieved.iter().enumerate() {
            prop_assert_eq!(child_h, leaves[i]);
            prop_assert_eq!(arena.get(child_h).value(), i as i32);
        }
    }

    #[test]
    fn pt37_metrics_len_equals_node_count(cap in arb_capacity(), n in arb_node_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.metrics().len(), n);
    }

    #[test]
    fn pt38_double_reset_is_idempotent(n in arb_node_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        arena.reset();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn pt39_double_clear_is_idempotent(n in arb_node_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        arena.clear();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.num_chunks(), 1);
    }

    #[test]
    fn pt40_leaf_symbol_equals_value_prop(val in arb_symbol()) {
        let n = TreeNode::leaf(val);
        prop_assert_eq!(n.symbol(), n.value());
    }

    #[test]
    fn pt41_branch_default_symbol_zero_prop(n in 0usize..=10) {
        let children: Vec<NodeHandle> = (0..n)
            .map(|i| NodeHandle::new(0, i as u32))
            .collect();
        let node = TreeNode::branch(children);
        prop_assert_eq!(node.symbol(), 0);
    }

    #[test]
    fn pt42_node_clone_preserves_kind(val in arb_symbol()) {
        let leaf = TreeNode::leaf(val);
        let cloned = leaf.clone();
        prop_assert!(cloned.is_leaf());
        prop_assert_eq!(leaf, cloned);
    }
}

// ============================================================================
// 15. Additional stress and edge-case unit tests
// ============================================================================

#[test]
fn ut25_alloc_i32_min_max() {
    let mut arena = TreeArena::new();
    let h_min = arena.alloc(TreeNode::leaf(i32::MIN));
    let h_max = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h_min).value(), i32::MIN);
    assert_eq!(arena.get(h_max).value(), i32::MAX);
}

#[test]
fn ut26_many_chunks_all_accessible() {
    let mut arena = TreeArena::with_capacity(1);
    let mut handles = Vec::new();
    for i in 0..20 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }
    assert!(arena.num_chunks() > 1);
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn ut27_nested_branches() {
    let mut arena = TreeArena::new();
    let l1 = arena.alloc(TreeNode::leaf(1));
    let l2 = arena.alloc(TreeNode::leaf(2));
    let b1 = arena.alloc(TreeNode::branch(vec![l1, l2]));
    let l3 = arena.alloc(TreeNode::leaf(3));
    let root = arena.alloc(TreeNode::branch_with_symbol(99, vec![b1, l3]));

    assert_eq!(arena.get(root).symbol(), 99);
    let root_ref = arena.get(root);
    let root_children = root_ref.children().to_vec();
    assert_eq!(root_children.len(), 2);
    assert_eq!(arena.get(root_children[0]).children().len(), 2);
    assert!(arena.get(root_children[1]).is_leaf());
}

#[test]
fn ut28_clear_then_many_allocs() {
    let mut arena = TreeArena::with_capacity(2);
    for i in 0..10 {
        arena.alloc(TreeNode::leaf(i));
    }
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);

    for i in 0..20 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.len(), 20);
}
