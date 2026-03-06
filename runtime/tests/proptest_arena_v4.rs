//! Property-based tests (v4) for TreeArena and error recovery types.
//!
//! 44 proptest properties covering allocation stability, handle uniqueness,
//! arena capacity, node retrieval consistency, large allocation sequences,
//! and ErrorRecoveryConfig / ErrorNode invariants.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::arena_allocator::{TreeArena, TreeNode};
use adze::error_recovery::{ErrorNode, ErrorRecoveryConfig, RecoveryStrategy};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;
use proptest::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Strategies
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

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..=1000).prop_map(SymbolId)
}

fn arb_recovery_strategy() -> impl Strategy<Value = RecoveryStrategy> {
    prop_oneof![
        Just(RecoveryStrategy::PanicMode),
        Just(RecoveryStrategy::TokenInsertion),
        Just(RecoveryStrategy::TokenDeletion),
        Just(RecoveryStrategy::TokenSubstitution),
        Just(RecoveryStrategy::PhraseLevel),
        Just(RecoveryStrategy::ScopeRecovery),
    ]
}

// ============================================================================
// 1. Allocation stability — nodes survive subsequent allocs
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn prop_first_node_stable_after_many_allocs(n in arb_alloc_count(), sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let first = arena.alloc(TreeNode::leaf(sym));
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.get(first).value(), sym);
    }

    #[test]
    fn prop_all_leaves_stable(vals in prop::collection::vec(arb_symbol(), 1..150)) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = vals.iter().map(|&v| arena.alloc(TreeNode::leaf(v))).collect();
        for (h, &v) in handles.iter().zip(vals.iter()) {
            prop_assert_eq!(arena.get(*h).value(), v);
        }
    }

    #[test]
    fn prop_branch_children_stable_after_growth(n in 1usize..=40) {
        let mut arena = TreeArena::with_capacity(4);
        let children: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let parent = arena.alloc(TreeNode::branch(children));
        // Force chunk growth
        for i in 0..100 {
            arena.alloc(TreeNode::leaf(i));
        }
        prop_assert_eq!(arena.get(parent).children().len(), n);
        for (i, ch) in arena.get(parent).children().iter().enumerate() {
            prop_assert_eq!(arena.get(*ch).value(), i as i32);
        }
    }

    #[test]
    fn prop_mutation_persists_across_allocs(sym in arb_symbol(), extra in 1usize..=100) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(0));
        arena.get_mut(h).set_value(sym);
        for i in 0..extra {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.get(h).value(), sym);
    }
}

// ============================================================================
// 2. Handle uniqueness
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn prop_handles_unique(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let mut seen = HashSet::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(seen.insert(h), "duplicate handle at alloc {}", i);
        }
    }

    #[test]
    fn prop_handles_unique_small_capacity(n in 1usize..=200) {
        let mut arena = TreeArena::with_capacity(2);
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
    fn prop_handle_copy_semantics(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        let h2 = h; // Copy, not clone
        prop_assert_eq!(arena.get(h).value(), arena.get(h2).value());
        prop_assert_eq!(h, h2);
    }
}

// ============================================================================
// 3. Arena capacity invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn prop_capacity_ge_len(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(arena.capacity() >= arena.len());
        }
    }

    #[test]
    fn prop_capacity_ge_len_custom(cap in arb_capacity(), n in arb_alloc_count()) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }

    #[test]
    fn prop_initial_capacity_exact(cap in arb_capacity()) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert_eq!(arena.capacity(), cap);
    }

    #[test]
    fn prop_capacity_monotonic_on_alloc(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        let mut prev = arena.capacity();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
            let cur = arena.capacity();
            prop_assert!(cur >= prev, "capacity shrank from {} to {}", prev, cur);
            prev = cur;
        }
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
}

// ============================================================================
// 4. Node retrieval consistency
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn prop_leaf_kind_preserved(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        let node = arena.get(h);
        prop_assert!(node.is_leaf());
        prop_assert!(!node.is_branch());
        prop_assert_eq!(node.value(), sym);
    }

    #[test]
    fn prop_branch_kind_preserved(sym in arb_symbol(), n in 0usize..=20) {
        let mut arena = TreeArena::new();
        let kids: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, kids));
        let node = arena.get(h);
        prop_assert!(node.is_branch());
        prop_assert!(!node.is_leaf());
        prop_assert_eq!(node.symbol(), sym);
        prop_assert_eq!(node.children().len(), n);
    }

    #[test]
    fn prop_leaf_children_empty(sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert!(arena.get(h).children().is_empty());
    }

    #[test]
    fn prop_branch_default_symbol_zero(n in 0usize..=30) {
        let mut arena = TreeArena::new();
        let kids: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        let h = arena.alloc(TreeNode::branch(kids));
        prop_assert_eq!(arena.get(h).symbol(), 0);
    }

    #[test]
    fn prop_get_mut_set_value_roundtrip(orig in arb_symbol(), replacement in arb_symbol()) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(orig));
        arena.get_mut(h).set_value(replacement);
        prop_assert_eq!(arena.get(h).value(), replacement);
        prop_assert!(arena.get(h).is_leaf());
    }
}

// ============================================================================
// 5. Large allocation sequences
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_large_alloc_all_retrievable(n in 500usize..=2000) {
        let mut arena = TreeArena::new();
        let handles: Vec<_> = (0..n).map(|i| arena.alloc(TreeNode::leaf(i as i32))).collect();
        prop_assert_eq!(arena.len(), n);
        for (i, h) in handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }

    #[test]
    fn prop_large_alloc_forces_chunk_growth(n in 2000usize..=5000) {
        let mut arena = TreeArena::with_capacity(8);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.num_chunks() > 1);
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn prop_large_alloc_handles_unique(n in 500usize..=2000) {
        let mut arena = TreeArena::new();
        let mut seen = HashSet::new();
        for i in 0..n {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            prop_assert!(seen.insert(h));
        }
        prop_assert_eq!(seen.len(), n);
    }

    #[test]
    fn prop_large_mixed_leaf_branch(count in 200usize..=800) {
        let mut arena = TreeArena::new();
        let mut leaf_handles = Vec::new();
        for i in 0..count {
            let h = arena.alloc(TreeNode::leaf(i as i32));
            leaf_handles.push(h);
        }
        // Create branches referencing earlier leaves
        for chunk in leaf_handles.chunks(4) {
            arena.alloc(TreeNode::branch(chunk.to_vec()));
        }
        // Verify original leaves untouched
        for (i, h) in leaf_handles.iter().enumerate() {
            prop_assert_eq!(arena.get(*h).value(), i as i32);
        }
    }
}

// ============================================================================
// 6. Len / is_empty / metrics consistency
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn prop_len_equals_alloc_count(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn prop_is_empty_iff_len_zero(n in 0usize..=100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.is_empty(), n == 0);
    }

    #[test]
    fn prop_metrics_agree_with_accessors(n in arb_alloc_count()) {
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
    fn prop_memory_usage_positive_after_alloc(n in arb_alloc_count()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.memory_usage() > 0);
    }

    #[test]
    fn prop_num_chunks_at_least_one(cap in arb_capacity(), n in 0usize..=50) {
        let mut arena = TreeArena::with_capacity(cap);
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.num_chunks() >= 1);
    }
}

// ============================================================================
// 7. Reset / clear semantics
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn prop_reset_then_alloc(n in arb_alloc_count(), sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
        prop_assert_eq!(arena.len(), 1);
    }

    #[test]
    fn prop_clear_then_alloc(n in arb_alloc_count(), sym in arb_symbol()) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.clear();
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.num_chunks(), 1);
        let h = arena.alloc(TreeNode::leaf(sym));
        prop_assert_eq!(arena.get(h).value(), sym);
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
// 8. ErrorRecoveryConfig properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn prop_default_config_allows_delete_non_sync(token in arb_symbol_id()) {
        let cfg = ErrorRecoveryConfig::default();
        // Default has no sync tokens, so any token is deletable
        prop_assert!(cfg.can_delete_token(token));
    }

    #[test]
    fn prop_default_config_allows_replace_non_sync(token in arb_symbol_id()) {
        let cfg = ErrorRecoveryConfig::default();
        prop_assert!(cfg.can_replace_token(token));
    }

    #[test]
    fn prop_sync_token_blocks_replace(token in arb_symbol_id()) {
        let mut cfg = ErrorRecoveryConfig::default();
        cfg.sync_tokens.push(token);
        prop_assert!(!cfg.can_replace_token(token));
    }

    #[test]
    fn prop_deletable_set_allows_delete_even_if_sync(token in arb_symbol_id()) {
        let mut cfg = ErrorRecoveryConfig::default();
        cfg.sync_tokens.push(token);
        cfg.deletable_tokens.insert(token.0);
        // Explicitly in deletable_tokens → can_delete_token returns true
        prop_assert!(cfg.can_delete_token(token));
    }

    #[test]
    fn prop_default_config_field_values(_dummy in Just(())) {
        let cfg = ErrorRecoveryConfig::default();
        prop_assert_eq!(cfg.max_panic_skip, 50);
        prop_assert_eq!(cfg.max_token_deletions, 3);
        prop_assert_eq!(cfg.max_token_insertions, 2);
        prop_assert_eq!(cfg.max_consecutive_errors, 10);
        prop_assert!(cfg.enable_phrase_recovery);
        prop_assert!(cfg.enable_scope_recovery);
        prop_assert!(!cfg.enable_indentation_recovery);
        prop_assert!(cfg.sync_tokens.is_empty());
        prop_assert!(cfg.insert_candidates.is_empty());
        prop_assert!(cfg.scope_delimiters.is_empty());
    }

    #[test]
    fn prop_config_sync_tokens_roundtrip(
        tokens in prop::collection::vec(arb_symbol_id(), 0..20),
    ) {
        let mut cfg = ErrorRecoveryConfig::default();
        for &t in &tokens {
            cfg.sync_tokens.push(t);
        }
        prop_assert_eq!(cfg.sync_tokens.len(), tokens.len());
        for (a, b) in cfg.sync_tokens.iter().zip(tokens.iter()) {
            prop_assert_eq!(a, b);
        }
    }
}

// ============================================================================
// 9. ErrorNode properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn prop_error_node_byte_range_valid(
        start in 0usize..10_000,
        len in 0usize..1_000,
        strategy in arb_recovery_strategy(),
    ) {
        let end = start + len;
        let node = ErrorNode {
            start_byte: start,
            end_byte: end,
            start_position: (0, start),
            end_position: (0, end),
            expected: vec![],
            actual: None,
            recovery: strategy,
            skipped_tokens: vec![],
        };
        prop_assert!(node.start_byte <= node.end_byte);
    }

    #[test]
    fn prop_error_node_preserves_expected(
        expected in prop::collection::vec(0u16..500, 0..20),
        strategy in arb_recovery_strategy(),
    ) {
        let node = ErrorNode {
            start_byte: 0,
            end_byte: 10,
            start_position: (0, 0),
            end_position: (0, 10),
            expected: expected.clone(),
            actual: Some(999),
            recovery: strategy,
            skipped_tokens: vec![],
        };
        prop_assert_eq!(node.expected, expected);
    }

    #[test]
    fn prop_error_node_clone_equality(
        start in 0usize..5_000,
        end_offset in 0usize..500,
        actual in prop::option::of(0u16..1000),
    ) {
        let node = ErrorNode {
            start_byte: start,
            end_byte: start + end_offset,
            start_position: (0, 0),
            end_position: (0, 0),
            expected: vec![1, 2, 3],
            actual,
            recovery: RecoveryStrategy::PanicMode,
            skipped_tokens: vec![42],
        };
        let cloned = node.clone();
        prop_assert_eq!(cloned.start_byte, node.start_byte);
        prop_assert_eq!(cloned.end_byte, node.end_byte);
        prop_assert_eq!(cloned.expected, node.expected);
        prop_assert_eq!(cloned.actual, node.actual);
        prop_assert_eq!(cloned.skipped_tokens, node.skipped_tokens);
    }

    #[test]
    fn prop_error_node_skipped_tokens_preserved(
        skipped in prop::collection::vec(0u16..500, 0..30),
    ) {
        let node = ErrorNode {
            start_byte: 0,
            end_byte: 0,
            start_position: (0, 0),
            end_position: (0, 0),
            expected: vec![],
            actual: None,
            recovery: RecoveryStrategy::TokenDeletion,
            skipped_tokens: skipped.clone(),
        };
        prop_assert_eq!(node.skipped_tokens.len(), skipped.len());
        prop_assert_eq!(node.skipped_tokens, skipped);
    }
}

// ============================================================================
// 10. Cross-cutting: arena + error config interactions
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_scope_delimiters_roundtrip(
        pairs in prop::collection::vec((0u16..200, 0u16..200), 0..10),
    ) {
        let cfg = ErrorRecoveryConfig {
            scope_delimiters: pairs.clone(),
            ..ErrorRecoveryConfig::default()
        };
        prop_assert_eq!(cfg.scope_delimiters.len(), pairs.len());
        for (i, (open, close)) in cfg.scope_delimiters.iter().enumerate() {
            prop_assert_eq!(*open, pairs[i].0);
            prop_assert_eq!(*close, pairs[i].1);
        }
    }

    #[test]
    fn prop_recovery_strategy_copy_semantics(strategy in arb_recovery_strategy()) {
        let s2 = strategy; // Copy
        prop_assert_eq!(strategy, s2);
    }
}
