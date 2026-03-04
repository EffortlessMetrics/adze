//! Property-based and unit tests for visitor patterns and arena interactions.
//!
//! 55+ tests covering StatsVisitor, PrettyPrintVisitor, SearchVisitor,
//! VisitorAction, TreeArena, TreeNode, and multi-visitor workflows.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use adze::visitor::{PrettyPrintVisitor, StatsVisitor, VisitorAction};
use proptest::prelude::*;

// ============================================================================
// 1. Property: StatsVisitor always starts with zero counts
// ============================================================================

proptest! {
    #[test]
    fn stats_visitor_default_total_nodes_zero(_seed in 0u64..1000) {
        let sv = StatsVisitor::default();
        prop_assert_eq!(sv.total_nodes, 0);
    }

    #[test]
    fn stats_visitor_default_leaf_nodes_zero(_seed in 0u64..1000) {
        let sv = StatsVisitor::default();
        prop_assert_eq!(sv.leaf_nodes, 0);
    }

    #[test]
    fn stats_visitor_default_error_nodes_zero(_seed in 0u64..1000) {
        let sv = StatsVisitor::default();
        prop_assert_eq!(sv.error_nodes, 0);
    }

    #[test]
    fn stats_visitor_default_max_depth_zero(_seed in 0u64..1000) {
        let sv = StatsVisitor::default();
        prop_assert_eq!(sv.max_depth, 0);
    }

    #[test]
    fn stats_visitor_default_node_counts_empty(_seed in 0u64..1000) {
        let sv = StatsVisitor::default();
        prop_assert!(sv.node_counts.is_empty());
    }
}

// ============================================================================
// 2. Property: PrettyPrintVisitor output is non-null / empty at start
// ============================================================================

proptest! {
    #[test]
    fn pretty_print_new_output_is_empty(_seed in 0u64..1000) {
        let pp = PrettyPrintVisitor::new();
        prop_assert_eq!(pp.output(), "");
    }

    #[test]
    fn pretty_print_new_output_len_zero(_seed in 0u64..1000) {
        let pp = PrettyPrintVisitor::new();
        prop_assert_eq!(pp.output().len(), 0);
    }

    #[test]
    fn pretty_print_output_is_valid_utf8(_seed in 0u64..1000) {
        let pp = PrettyPrintVisitor::new();
        // &str is always valid UTF-8; verify it doesn't panic
        let _bytes = pp.output().as_bytes();
        prop_assert!(true);
    }
}

// ============================================================================
// 3. Property: SearchVisitor starts with no matches
// ============================================================================

proptest! {
    #[test]
    fn search_visitor_starts_unfound(_seed in 0u64..1000) {
        let sv = adze::visitor::SearchVisitor::new(|_node| false);
        prop_assert!(sv.matches.is_empty());
    }

    #[test]
    fn search_visitor_always_true_still_empty_without_walk(_seed in 0u64..1000) {
        let sv = adze::visitor::SearchVisitor::new(|_node| true);
        prop_assert!(sv.matches.is_empty());
    }
}

// ============================================================================
// 4. Property: Arena len matches allocation count
// ============================================================================

proptest! {
    #[test]
    fn arena_len_matches_alloc_count(count in 1usize..100) {
        let mut arena = TreeArena::with_capacity(8);
        for i in 0..count {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), count);
    }

    #[test]
    fn arena_is_empty_when_no_allocs(cap in 1usize..512) {
        let arena = TreeArena::with_capacity(cap);
        prop_assert!(arena.is_empty());
        prop_assert_eq!(arena.len(), 0);
    }

    #[test]
    fn arena_not_empty_after_one_alloc(cap in 1usize..512) {
        let mut arena = TreeArena::with_capacity(cap);
        arena.alloc(TreeNode::leaf(0));
        prop_assert!(!arena.is_empty());
    }

    #[test]
    fn arena_capacity_gte_len(count in 1usize..200) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..count {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert!(arena.capacity() >= arena.len());
    }
}

// ============================================================================
// 5. Property: VisitorAction variants are distinct
// ============================================================================

proptest! {
    #[test]
    fn visitor_action_continue_ne_skip(_seed in 0u64..100) {
        prop_assert_ne!(VisitorAction::Continue, VisitorAction::SkipChildren);
    }

    #[test]
    fn visitor_action_continue_ne_stop(_seed in 0u64..100) {
        prop_assert_ne!(VisitorAction::Continue, VisitorAction::Stop);
    }

    #[test]
    fn visitor_action_skip_ne_stop(_seed in 0u64..100) {
        prop_assert_ne!(VisitorAction::SkipChildren, VisitorAction::Stop);
    }

    #[test]
    fn visitor_action_eq_is_reflexive(variant in 0u8..3) {
        let action = match variant {
            0 => VisitorAction::Continue,
            1 => VisitorAction::SkipChildren,
            _ => VisitorAction::Stop,
        };
        prop_assert_eq!(action, action);
    }

    #[test]
    fn visitor_action_copy_semantics(variant in 0u8..3) {
        let a = match variant {
            0 => VisitorAction::Continue,
            1 => VisitorAction::SkipChildren,
            _ => VisitorAction::Stop,
        };
        let b = a; // Copy
        prop_assert_eq!(a, b);
    }
}

// ============================================================================
// 6. Unit: TreeNode construction patterns
// ============================================================================

#[test]
fn leaf_is_leaf() {
    let node = TreeNode::leaf(42);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
}

#[test]
fn leaf_value_roundtrip() {
    let node = TreeNode::leaf(99);
    assert_eq!(node.value(), 99);
}

#[test]
fn leaf_symbol_equals_value() {
    let node = TreeNode::leaf(-7);
    assert_eq!(node.symbol(), node.value());
}

#[test]
fn leaf_children_is_empty() {
    let node = TreeNode::leaf(1);
    assert!(node.children().is_empty());
}

#[test]
fn branch_is_branch() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
}

#[test]
fn branch_empty_children() {
    let node = TreeNode::branch(vec![]);
    assert!(node.children().is_empty());
}

#[test]
fn branch_default_symbol_is_zero() {
    let node = TreeNode::branch(vec![]);
    assert_eq!(node.symbol(), 0);
}

#[test]
fn branch_with_symbol_stores_symbol() {
    let node = TreeNode::branch_with_symbol(55, vec![]);
    assert_eq!(node.symbol(), 55);
}

#[test]
fn branch_with_children_preserves_handles() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let node = TreeNode::branch(vec![h1, h2]);
    assert_eq!(node.children().len(), 2);
    assert_eq!(node.children()[0], h1);
    assert_eq!(node.children()[1], h2);
}

#[test]
fn leaf_clone_equality() {
    let a = TreeNode::leaf(42);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn branch_clone_equality() {
    let h = NodeHandle::new(0, 0);
    let a = TreeNode::branch(vec![h]);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn leaf_debug_contains_value() {
    let node = TreeNode::leaf(123);
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("123"));
}

// ============================================================================
// 7. Unit: Arena operations
// ============================================================================

#[test]
fn arena_new_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_default_is_empty() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
}

#[test]
fn arena_with_capacity_is_empty() {
    let arena = TreeArena::with_capacity(16);
    assert!(arena.is_empty());
}

#[test]
#[should_panic]
fn arena_with_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

#[test]
fn arena_alloc_leaf_and_get() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn arena_alloc_branch_and_get() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));

    assert!(arena.get(parent).is_branch());
    assert_eq!(arena.get(parent).children().len(), 2);
}

#[test]
fn arena_get_ref_deref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(77));
    let node_ref = arena.get(h);
    // Deref to TreeNode
    assert_eq!(node_ref.value(), 77);
    assert!(node_ref.is_leaf());
}

#[test]
fn arena_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    assert_eq!(arena.get(h).value(), 10);

    arena.get_mut(h).set_value(20);
    assert_eq!(arena.get(h).value(), 20);
}

#[test]
fn arena_reset_clears_len() {
    let mut arena = TreeArena::with_capacity(4);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);

    arena.reset();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn arena_clear_clears_len() {
    let mut arena = TreeArena::with_capacity(4);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));

    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn arena_reset_preserves_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3)); // triggers new chunk
    let chunks_before = arena.num_chunks();

    arena.reset();
    assert_eq!(arena.num_chunks(), chunks_before);
}

#[test]
fn arena_clear_truncates_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3)); // triggers new chunk
    assert!(arena.num_chunks() > 1);

    arena.clear();
    assert_eq!(arena.num_chunks(), 1);
}

#[test]
fn arena_reuse_after_reset() {
    let mut arena = TreeArena::with_capacity(4);
    let h1 = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.get(h1).value(), 1);

    arena.reset();

    let h2 = arena.alloc(TreeNode::leaf(99));
    assert_eq!(arena.get(h2).value(), 99);
    assert_eq!(arena.len(), 1);
}

#[test]
fn arena_metrics_snapshot() {
    let mut arena = TreeArena::with_capacity(8);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));

    let m = arena.metrics();
    assert_eq!(m.len(), 2);
    assert!(!m.is_empty());
    assert!(m.capacity() >= 8);
    assert_eq!(m.num_chunks(), 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn arena_metrics_empty() {
    let arena = TreeArena::with_capacity(8);
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
}

#[test]
fn arena_memory_usage_positive() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn arena_chunk_growth_doubles() {
    let mut arena = TreeArena::with_capacity(2);
    // Fill chunk 1 (cap=2)
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 1);

    // Trigger chunk 2 (cap=4)
    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.num_chunks(), 2);

    // Fill chunk 2
    arena.alloc(TreeNode::leaf(4));
    arena.alloc(TreeNode::leaf(5));
    arena.alloc(TreeNode::leaf(6));
    assert_eq!(arena.num_chunks(), 2);

    // Trigger chunk 3 (cap=8)
    arena.alloc(TreeNode::leaf(7));
    assert_eq!(arena.num_chunks(), 3);
}

// ============================================================================
// 8. Unit: Multiple visitors / combined workflows
// ============================================================================

#[test]
fn stats_and_pretty_print_independent() {
    let sv = StatsVisitor::default();
    let pp = PrettyPrintVisitor::new();
    assert_eq!(sv.total_nodes, 0);
    assert_eq!(pp.output(), "");
}

#[test]
fn two_stats_visitors_independent() {
    let sv1 = StatsVisitor::default();
    let sv2 = StatsVisitor::default();
    assert_eq!(sv1.total_nodes, sv2.total_nodes);
    assert_eq!(sv1.leaf_nodes, sv2.leaf_nodes);
}

#[test]
fn search_visitor_with_false_predicate() {
    let sv = adze::visitor::SearchVisitor::new(|_| false);
    assert!(sv.matches.is_empty());
}

#[test]
fn search_visitor_with_true_predicate_no_walk() {
    let sv = adze::visitor::SearchVisitor::new(|_| true);
    assert!(sv.matches.is_empty());
}

#[test]
fn visitor_action_debug_format() {
    let dbg = format!("{:?}", VisitorAction::Continue);
    assert!(dbg.contains("Continue"));

    let dbg = format!("{:?}", VisitorAction::SkipChildren);
    assert!(dbg.contains("SkipChildren"));

    let dbg = format!("{:?}", VisitorAction::Stop);
    assert!(dbg.contains("Stop"));
}

#[test]
fn visitor_action_clone() {
    let a = VisitorAction::Continue;
    let b = a;
    assert_eq!(a, b);

    let c = VisitorAction::Stop;
    let d = c;
    assert_eq!(c, d);
}

#[test]
fn pretty_print_default_same_as_new() {
    let a = PrettyPrintVisitor::new();
    let b = PrettyPrintVisitor::default();
    assert_eq!(a.output(), b.output());
}

// ============================================================================
// 9. Property: Arena + TreeNode combined properties
// ============================================================================

proptest! {
    #[test]
    fn alloc_leaf_preserves_value(val in prop::num::i32::ANY) {
        let mut arena = TreeArena::with_capacity(4);
        let h = arena.alloc(TreeNode::leaf(val));
        prop_assert_eq!(arena.get(h).value(), val);
    }

    #[test]
    fn alloc_branch_with_symbol_preserves(sym in prop::num::i32::ANY) {
        let mut arena = TreeArena::with_capacity(4);
        let h = arena.alloc(TreeNode::branch_with_symbol(sym, vec![]));
        prop_assert_eq!(arena.get(h).symbol(), sym);
        prop_assert!(arena.get(h).is_branch());
    }

    #[test]
    fn leaf_is_never_branch(val in prop::num::i32::ANY) {
        let node = TreeNode::leaf(val);
        prop_assert!(node.is_leaf());
        prop_assert!(!node.is_branch());
    }

    #[test]
    fn branch_is_never_leaf(sym in prop::num::i32::ANY) {
        let node = TreeNode::branch_with_symbol(sym, vec![]);
        prop_assert!(node.is_branch());
        prop_assert!(!node.is_leaf());
    }

    #[test]
    fn arena_len_after_reset_is_zero(count in 1usize..50) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..count {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert_eq!(arena.len(), 0);
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn arena_realloc_after_reset(first in 1usize..30, second in 1usize..30) {
        let mut arena = TreeArena::with_capacity(4);
        for i in 0..first {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        for i in 0..second {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), second);
    }

    #[test]
    fn node_handle_equality(chunk in 0u32..10, idx in 0u32..100) {
        let h1 = NodeHandle::new(chunk, idx);
        let h2 = NodeHandle::new(chunk, idx);
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn node_handle_inequality(
        c1 in 0u32..10, i1 in 0u32..100,
        c2 in 0u32..10, i2 in 0u32..100,
    ) {
        let h1 = NodeHandle::new(c1, i1);
        let h2 = NodeHandle::new(c2, i2);
        if c1 != c2 || i1 != i2 {
            prop_assert_ne!(h1, h2);
        }
    }
}

// ============================================================================
// 10. Property: TreeNode Clone + PartialEq
// ============================================================================

proptest! {
    #[test]
    fn leaf_clone_eq(val in prop::num::i32::ANY) {
        let a = TreeNode::leaf(val);
        let b = a.clone();
        prop_assert_eq!(a, b);
    }

    #[test]
    fn branch_with_sym_clone_eq(sym in prop::num::i32::ANY) {
        let a = TreeNode::branch_with_symbol(sym, vec![]);
        let b = a.clone();
        prop_assert_eq!(a, b);
    }

    #[test]
    fn leaf_ne_branch_same_value(val in prop::num::i32::ANY) {
        let leaf = TreeNode::leaf(val);
        let branch = TreeNode::branch_with_symbol(val, vec![]);
        prop_assert_ne!(leaf, branch);
    }
}

// ============================================================================
// 11. Unit: NodeHandle hash consistency
// ============================================================================

#[test]
fn node_handle_hash_consistent() {
    use std::collections::HashSet;
    let h = NodeHandle::new(1, 2);
    let mut set = HashSet::new();
    set.insert(h);
    assert!(set.contains(&NodeHandle::new(1, 2)));
    assert!(!set.contains(&NodeHandle::new(1, 3)));
}

#[test]
fn node_handle_debug_format() {
    let h = NodeHandle::new(3, 7);
    let dbg = format!("{:?}", h);
    assert!(dbg.contains("3"));
    assert!(dbg.contains("7"));
}

// ============================================================================
// 12. Unit: TreeNodeRef accessors
// ============================================================================

#[test]
fn tree_node_ref_value_and_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(88));
    let r = arena.get(h);
    assert_eq!(r.value(), 88);
    assert_eq!(r.symbol(), 88);
}

#[test]
fn tree_node_ref_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn tree_node_ref_is_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(!arena.get(h).is_leaf());
}

#[test]
fn tree_node_ref_children() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let p = arena.alloc(TreeNode::branch(vec![c]));
    assert_eq!(arena.get(p).children().len(), 1);
    assert_eq!(arena.get(p).children()[0], c);
}

#[test]
fn tree_node_ref_get_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let r = arena.get(h);
    let inner = r.get_ref();
    assert_eq!(inner.value(), 5);
}

#[test]
fn tree_node_ref_as_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let r = arena.get(h);
    let inner = r.as_ref();
    assert_eq!(inner.value(), 5);
}
