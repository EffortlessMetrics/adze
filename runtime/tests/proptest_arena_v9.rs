//! Property-based tests for `TreeArena` allocator.
//!
//! Tests cover allocation, handle validity, value preservation,
//! child relationships, clear/reset, and edge cases.

use adze::arena_allocator::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Strategy for symbol values used in leaf/branch nodes.
fn symbol_strategy() -> impl Strategy<Value = i32> {
    prop_oneof![
        Just(0),
        Just(1),
        Just(-1),
        Just(i32::MAX),
        Just(i32::MIN),
        -1000..1000i32,
    ]
}

/// Strategy for a small collection size (1..=64).
fn small_count() -> impl Strategy<Value = usize> {
    1..=64usize
}

/// Strategy for a tiny collection size (1..=8).
fn tiny_count() -> impl Strategy<Value = usize> {
    1..=8usize
}

// ===========================================================================
// 1. Arena size after N allocs (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn pt_size_after_n_leaf_allocs(n in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(1));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn pt_size_after_n_branch_allocs(n in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::branch(vec![]));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn pt_size_after_mixed_allocs(
        leaves in small_count(),
        branches in small_count(),
    ) {
        let mut arena = TreeArena::new();
        for _ in 0..leaves {
            arena.alloc(TreeNode::leaf(0));
        }
        for _ in 0..branches {
            arena.alloc(TreeNode::branch(vec![]));
        }
        prop_assert_eq!(arena.len(), leaves + branches);
    }

    #[test]
    fn pt_is_empty_false_after_alloc(n in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert!(!arena.is_empty());
    }

    #[test]
    fn pt_size_with_small_capacity(
        cap in 1..=4usize,
        n in 1..=32usize,
    ) {
        let mut arena = TreeArena::with_capacity(cap);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.len(), n);
    }
}

// ===========================================================================
// 2. All handles valid (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn pt_leaf_handles_valid(n in small_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        for h in &handles {
            // Access must not panic
            let _ = arena.get(*h);
        }
    }

    #[test]
    fn pt_branch_handles_valid(n in small_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|_| arena.alloc(TreeNode::branch(vec![])))
            .collect();
        for h in &handles {
            let _ = arena.get(*h);
        }
    }

    #[test]
    fn pt_handles_unique(n in 2..=64usize) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        for i in 0..handles.len() {
            for j in (i + 1)..handles.len() {
                prop_assert_ne!(handles[i], handles[j]);
            }
        }
    }

    #[test]
    fn pt_handles_valid_across_chunks(n in 1..=128usize) {
        let mut arena = TreeArena::with_capacity(4);
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn pt_get_mut_handles_valid(n in small_count()) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        for h in &handles {
            let _ = arena.get_mut(*h);
        }
    }
}

// ===========================================================================
// 3. Value preservation (symbol) (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn pt_leaf_value_preserved(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
    }

    #[test]
    fn pt_branch_symbol_preserved(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).symbol(), sym);
    }

    #[test]
    fn pt_many_values_preserved(values in prop::collection::vec(symbol_strategy(), 1..=64)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = values.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        for (h, &expected) in handles.iter().zip(values.iter()) {
            prop_assert_eq!(arena.get(*h).value(), expected);
        }
    }

    #[test]
    fn pt_value_stable_after_more_allocs(sym in symbol_strategy(), extra in 1..=32usize) {
        let mut arena = TreeArena::new();
        let first = arena.alloc(TreeNode::leaf(sym));
        for _ in 0..extra {
            arena.alloc(TreeNode::leaf(0));
        }
        prop_assert_eq!(arena.get(first).value(), sym);
    }

    #[test]
    fn pt_set_value_via_get_mut(original in symbol_strategy(), updated in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(original));
        arena.get_mut(h).set_value(updated);
        prop_assert_eq!(arena.get(h).value(), updated);
    }
}

// ===========================================================================
// 4. Node type preservation (leaf vs branch) (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn pt_leaf_stays_leaf(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).is_leaf());
        prop_assert!(!arena.get(h).is_branch());
    }

    #[test]
    fn pt_branch_stays_branch(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).is_branch());
        prop_assert!(!arena.get(h).is_leaf());
    }

    #[test]
    fn pt_mixed_types_preserved(n in tiny_count()) {
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
    fn pt_leaf_children_empty(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).children().is_empty());
    }

    #[test]
    fn pt_type_stable_across_chunks(n in 1..=128usize) {
        let mut arena = TreeArena::with_capacity(4);
        let handles: Vec<_> = (0..n)
            .map(|i| {
                if i % 3 == 0 {
                    (arena.alloc(TreeNode::branch(vec![])), true)
                } else {
                    (arena.alloc(TreeNode::leaf(i as i32)), false)
                }
            })
            .collect();
        for (h, is_branch) in &handles {
            prop_assert_eq!(arena.get(*h).is_branch(), *is_branch);
        }
    }
}

// ===========================================================================
// 5. Child count / children (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn pt_branch_child_count(n_children in 0..=8usize) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n_children)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        prop_assert_eq!(arena.get(parent).children().len(), n_children);
    }

    #[test]
    fn pt_children_handles_match(n_children in 1..=8usize) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..n_children)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let parent = arena.alloc(TreeNode::branch(children.clone()));
        let parent_ref = arena.get(parent);
        let stored = parent_ref.children();
        prop_assert_eq!(stored, &children[..]);
    }

    #[test]
    fn pt_children_values_accessible(values in prop::collection::vec(symbol_strategy(), 1..=8)) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = values.iter()
            .map(|&v| arena.alloc(TreeNode::leaf(v)))
            .collect();
        let parent = arena.alloc(TreeNode::branch(children));
        let child_handles = arena.get(parent).children().to_vec();
        for (h, &expected) in child_handles.iter().zip(values.iter()) {
            prop_assert_eq!(arena.get(*h).value(), expected);
        }
    }

    #[test]
    fn pt_nested_branches(depth in 1..=6usize) {
        let mut arena = TreeArena::new();
        let mut current = arena.alloc(TreeNode::leaf(0));
        for d in 1..=depth {
            current = arena.alloc(TreeNode::branch_with_symbol(d as i32, vec![current]));
        }
        // Walk back down
        let mut node_handle = current;
        for d in (1..=depth).rev() {
            let node_ref = arena.get(node_handle);
            prop_assert_eq!(node_ref.symbol(), d as i32);
            prop_assert_eq!(node_ref.children().len(), 1);
            node_handle = node_ref.children()[0];
        }
        prop_assert_eq!(arena.get(node_handle).value(), 0);
    }

    #[test]
    fn pt_empty_branch_no_children(sym in symbol_strategy()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert!(arena.get(h).children().is_empty());
    }
}

// ===========================================================================
// 6. Arena clear / reset (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn pt_reset_makes_empty(n in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn pt_clear_makes_empty(n in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn pt_alloc_after_reset(n in small_count(), m in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.reset();
        let handles: Vec<_> = (0..m)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        prop_assert_eq!(arena.len(), m);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn pt_alloc_after_clear(n in small_count(), m in small_count()) {
        let mut arena = TreeArena::new();
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        for _ in 0..m {
            arena.alloc(TreeNode::leaf(1));
        }
        prop_assert_eq!(arena.len(), m);
    }

    #[test]
    fn pt_clear_single_chunk(n in small_count()) {
        let mut arena = TreeArena::with_capacity(2);
        for _ in 0..n {
            arena.alloc(TreeNode::leaf(0));
        }
        arena.clear();
        prop_assert_eq!(arena.num_chunks(), 1);
    }
}

// ===========================================================================
// 7. Regular arena tests (10 tests)
// ===========================================================================

#[test]
fn test_new_arena_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn test_single_leaf_alloc() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn test_single_branch_alloc() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(arena.get(h).children().is_empty());
}

#[test]
fn test_branch_with_children() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    assert_eq!(arena.get(parent).children().len(), 2);
    assert_eq!(arena.get(parent).children()[0], c1);
    assert_eq!(arena.get(parent).children()[1], c2);
}

#[test]
fn test_branch_with_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(99, vec![]));
    assert_eq!(arena.get(h).symbol(), 99);
    assert!(arena.get(h).is_branch());
}

#[test]
fn test_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn test_reset_preserves_capacity() {
    let mut arena = TreeArena::with_capacity(4);
    for _ in 0..10 {
        arena.alloc(TreeNode::leaf(0));
    }
    let chunks_before = arena.num_chunks();
    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
    assert!(arena.is_empty());
}

#[test]
fn test_clear_shrinks_to_one_chunk() {
    let mut arena = TreeArena::with_capacity(2);
    for _ in 0..20 {
        arena.alloc(TreeNode::leaf(0));
    }
    assert!(arena.num_chunks() > 1);
    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn test_capacity_grows_with_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    let cap_initial = arena.capacity();
    // Fill first chunk
    arena.alloc(TreeNode::leaf(0));
    arena.alloc(TreeNode::leaf(0));
    // Trigger second chunk
    arena.alloc(TreeNode::leaf(0));
    assert!(arena.capacity() > cap_initial);
}

#[test]
fn test_metrics_snapshot() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    let m = arena.metrics();
    assert_eq!(m.len(), 2);
    assert!(!m.is_empty());
    assert!(m.capacity() >= 2);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

// ===========================================================================
// 8. Edge cases (10 tests)
// ===========================================================================

#[test]
fn test_default_arena() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
}

#[test]
fn test_with_capacity_one() {
    let mut arena = TreeArena::with_capacity(1);
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.len(), 2);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn test_with_capacity_zero_panics() {
    let _ = TreeArena::with_capacity(0);
}

#[test]
fn test_leaf_symbol_i32_min() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MIN));
    assert_eq!(arena.get(h).value(), i32::MIN);
}

#[test]
fn test_leaf_symbol_i32_max() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(i32::MAX));
    assert_eq!(arena.get(h).value(), i32::MAX);
}

#[test]
fn test_deep_nesting() {
    let mut arena = TreeArena::new();
    let mut current = arena.alloc(TreeNode::leaf(0));
    for _ in 0..100 {
        current = arena.alloc(TreeNode::branch(vec![current]));
    }
    assert_eq!(arena.len(), 101);
    assert!(arena.get(current).is_branch());
}

#[test]
fn test_wide_branch() {
    let mut arena = TreeArena::new();
    let children: Vec<_> = (0..256).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let parent = arena.alloc(TreeNode::branch(children));
    assert_eq!(arena.get(parent).children().len(), 256);
}

#[test]
fn test_multiple_resets() {
    let mut arena = TreeArena::new();
    for _ in 0..5 {
        for j in 0..10 {
            arena.alloc(TreeNode::leaf(j));
        }
        assert_eq!(arena.len(), 10);
        arena.reset();
        assert!(arena.is_empty());
    }
}

#[test]
fn test_multiple_clears() {
    let mut arena = TreeArena::new();
    for _ in 0..5 {
        for j in 0..10 {
            arena.alloc(TreeNode::leaf(j));
        }
        arena.clear();
        assert!(arena.is_empty());
        assert_eq!(arena.num_chunks(), 1);
    }
}

#[test]
fn test_node_handle_equality() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 0);
    let h3 = NodeHandle::new(0, 1);
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}
