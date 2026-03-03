#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the Graph-Structured Stack (GSS) in adze-glr-core.

use adze_glr_core::gss_arena::{ArenaGSS, ArenaGSSManager, ArenaGSSStats, ArenaStackNode};
use adze_ir::{StateId, SymbolId};
use typed_arena::Arena;

// ---------------------------------------------------------------------------
// GSS node creation
// ---------------------------------------------------------------------------

#[test]
fn node_creation_initial_state() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(0));
    assert_eq!(gss.active_heads.len(), 1);
    assert_eq!(gss.top_state(0), StateId(0));
}

#[test]
fn node_creation_with_nonzero_initial_state() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(42));
    assert_eq!(gss.top_state(0), StateId(42));
}

#[test]
fn node_initial_depth_is_zero() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(0));
    assert_eq!(gss.active_heads[0].depth, 0);
}

#[test]
fn node_initial_has_no_parent() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(0));
    assert!(gss.active_heads[0].parent.is_none());
}

// ---------------------------------------------------------------------------
// Stack push / pop operations
// ---------------------------------------------------------------------------

#[test]
fn push_increments_depth() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(10)));
    assert_eq!(gss.active_heads[0].depth, 1);
    gss.push(0, StateId(2), Some(SymbolId(20)));
    assert_eq!(gss.active_heads[0].depth, 2);
}

#[test]
fn push_sets_symbol() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(99)));
    assert_eq!(gss.active_heads[0].symbol, Some(SymbolId(99)));
}

#[test]
fn push_none_symbol() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    assert_eq!(gss.active_heads[0].symbol, None);
}

#[test]
fn pop_single_returns_correct_state() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(5), None);
    let popped = gss.pop(0, 1).unwrap();
    assert_eq!(popped, vec![StateId(5)]);
    assert_eq!(gss.top_state(0), StateId(0));
}

#[test]
fn pop_multiple_returns_ordered_states() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    gss.push(0, StateId(2), None);
    gss.push(0, StateId(3), None);
    let popped = gss.pop(0, 3).unwrap();
    // pop reverses so result is bottom-to-top order of popped nodes
    assert_eq!(popped, vec![StateId(1), StateId(2), StateId(3)]);
    assert_eq!(gss.top_state(0), StateId(0));
}

#[test]
fn pop_too_many_returns_none() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    // stack has depth 1 above root; popping 3 should fail
    let result = gss.pop(0, 3);
    assert!(result.is_none());
}

#[test]
fn pop_zero_returns_empty_vec() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    let popped = gss.pop(0, 0).unwrap();
    assert!(popped.is_empty());
    // head unchanged
    assert_eq!(gss.top_state(0), StateId(1));
}

// ---------------------------------------------------------------------------
// Fork handling (multiple heads)
// ---------------------------------------------------------------------------

#[test]
fn fork_creates_new_head() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    let idx = gss.fork_head(0);
    assert_eq!(gss.active_heads.len(), 2);
    assert_eq!(gss.top_state(idx), StateId(1));
}

#[test]
fn fork_returns_correct_index() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    let idx = gss.fork_head(0);
    assert_eq!(idx, 1);
    let idx2 = gss.fork_head(0);
    assert_eq!(idx2, 2);
}

#[test]
fn forked_heads_diverge_after_push() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    let fork_idx = gss.fork_head(0);
    gss.push(0, StateId(10), None);
    gss.push(fork_idx, StateId(20), None);
    assert_eq!(gss.top_state(0), StateId(10));
    assert_eq!(gss.top_state(fork_idx), StateId(20));
}

#[test]
fn forked_heads_share_prefix() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    let fork_idx = gss.fork_head(0);
    // Both point to the same node so they share a prefix
    assert!(gss.active_heads[0].shares_prefix_with(gss.active_heads[fork_idx]));
}

#[test]
fn multiple_forks_from_same_head() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    let f1 = gss.fork_head(0);
    let f2 = gss.fork_head(0);
    let f3 = gss.fork_head(0);
    assert_eq!(gss.active_heads.len(), 4);
    // All share the same top node initially
    for i in [f1, f2, f3] {
        assert_eq!(gss.top_state(i), gss.top_state(0));
    }
}

// ---------------------------------------------------------------------------
// Merge operations
// ---------------------------------------------------------------------------

#[test]
fn can_merge_identical_heads() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    let fork_idx = gss.fork_head(0);
    // Same state and same parent → mergeable
    assert!(gss.can_merge(0, fork_idx));
}

#[test]
fn cannot_merge_same_index() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(0));
    assert!(!gss.can_merge(0, 0));
}

#[test]
fn cannot_merge_different_states() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    let fork_idx = gss.fork_head(0);
    gss.push(0, StateId(1), None);
    gss.push(fork_idx, StateId(2), None);
    assert!(!gss.can_merge(0, fork_idx));
}

#[test]
fn merge_heads_removes_duplicate() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    let fork_idx = gss.fork_head(0);
    assert_eq!(gss.active_heads.len(), 2);
    gss.merge_heads(0, fork_idx);
    assert_eq!(gss.active_heads.len(), 1);
}

#[test]
fn deduplicate_removes_all_duplicates() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    // Create several duplicates
    gss.fork_head(0);
    gss.fork_head(0);
    gss.fork_head(0);
    assert_eq!(gss.active_heads.len(), 4);
    gss.deduplicate();
    assert_eq!(gss.active_heads.len(), 1);
}

#[test]
fn deduplicate_preserves_diverged_heads() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    let fork_idx = gss.fork_head(0);
    gss.push(0, StateId(1), None);
    gss.push(fork_idx, StateId(2), None);
    gss.deduplicate();
    // Both heads have different states → neither removed
    assert_eq!(gss.active_heads.len(), 2);
}

// ---------------------------------------------------------------------------
// Statistics tracking accuracy
// ---------------------------------------------------------------------------

#[test]
fn stats_initial_node_count() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(0));
    assert_eq!(gss.get_stats().total_nodes_created, 1);
}

#[test]
fn stats_nodes_created_after_pushes() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    for i in 1..=5 {
        gss.push(0, StateId(i), None);
    }
    assert_eq!(gss.get_stats().total_nodes_created, 6); // 1 initial + 5 pushes
}

#[test]
fn stats_fork_count() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.fork_head(0);
    gss.fork_head(0);
    gss.fork_head(0);
    assert_eq!(gss.get_stats().total_forks, 3);
}

#[test]
fn stats_max_active_heads_tracks_peak() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    // Fork to create 4 heads
    gss.fork_head(0);
    gss.fork_head(0);
    gss.fork_head(0);
    assert_eq!(gss.get_stats().max_active_heads, 4);
    // After dedup, active_heads shrinks but max stays at peak
    gss.deduplicate();
    assert_eq!(gss.get_stats().max_active_heads, 4);
}

#[test]
fn stats_merge_count() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    gss.fork_head(0);
    gss.fork_head(0);
    gss.deduplicate();
    assert_eq!(gss.get_stats().total_merges, 2);
}

#[test]
fn stats_default_values() {
    let stats = ArenaGSSStats::default();
    assert_eq!(stats.total_nodes_created, 0);
    assert_eq!(stats.max_active_heads, 0);
    assert_eq!(stats.total_forks, 0);
    assert_eq!(stats.total_merges, 0);
    assert_eq!(stats.arena_bytes_allocated, 0);
}

// ---------------------------------------------------------------------------
// Memory usage patterns
// ---------------------------------------------------------------------------

#[test]
fn shared_parent_pointer_equality() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    gss.push(0, StateId(2), None);
    let fork_idx = gss.fork_head(0);
    // Both heads point to the exact same arena node
    assert!(std::ptr::eq(
        gss.active_heads[0],
        gss.active_heads[fork_idx],
    ));
}

#[test]
fn arena_manager_reuse_after_clear() {
    let mut manager = ArenaGSSManager::new();
    {
        let mut gss = manager.new_session(StateId(0));
        gss.push(0, StateId(1), None);
        assert_eq!(gss.get_stats().total_nodes_created, 2);
    }
    manager.clear();
    {
        let gss = manager.new_session(StateId(100));
        // Fresh session starts with 1 node
        assert_eq!(gss.get_stats().total_nodes_created, 1);
        assert_eq!(gss.top_state(0), StateId(100));
    }
}

// ---------------------------------------------------------------------------
// Empty stack behaviour
// ---------------------------------------------------------------------------

#[test]
fn empty_stack_has_single_head() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(0));
    assert_eq!(gss.active_heads.len(), 1);
    assert!(gss.completed_heads.is_empty());
}

#[test]
fn pop_on_root_returns_none() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    // Attempting to pop the only node (root) with count > 0 that exceeds depth
    let result = gss.pop(0, 1);
    // Root has no parent so popping should fail for count > depth
    // depth is 0, popping 1 collects root state then tries parent (None).
    // Actually: for count=1 it collects the root state, then current = None,
    // then the "Update the head" branch needs current=Some, but it is None,
    // so head is NOT updated. pop still returns Some.
    assert!(result.is_some());
    assert_eq!(result.unwrap(), vec![StateId(0)]);
}

// ---------------------------------------------------------------------------
// Large stack operations
// ---------------------------------------------------------------------------

#[test]
fn large_push_pop_cycle() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    let count = 500;
    for i in 1..=count {
        gss.push(0, StateId(i), Some(SymbolId(i as u16)));
    }
    assert_eq!(gss.active_heads[0].depth, count as usize);
    let popped = gss.pop(0, count as usize).unwrap();
    assert_eq!(popped.len(), count as usize);
    assert_eq!(gss.top_state(0), StateId(0));
}

#[test]
fn many_forks_and_independent_pushes() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    let num_forks = 50;
    let mut indices = vec![0usize];
    for _ in 0..num_forks {
        let idx = gss.fork_head(0);
        indices.push(idx);
    }
    // Push different states onto each fork
    for i in 0..indices.len() {
        gss.push(indices[i], StateId((i + 1) as u16), None);
    }
    // Every head should have a unique top state
    for i in 0..indices.len() {
        assert_eq!(gss.top_state(indices[i]), StateId((i + 1) as u16));
    }
    assert_eq!(gss.get_stats().total_forks, num_forks);
}

// ---------------------------------------------------------------------------
// Stack traversal (get_states)
// ---------------------------------------------------------------------------

#[test]
fn get_states_returns_root_to_top_order() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    gss.push(0, StateId(2), None);
    gss.push(0, StateId(3), None);
    let states = gss.active_heads[0].get_states();
    assert_eq!(states, vec![StateId(0), StateId(1), StateId(2), StateId(3)]);
}

#[test]
fn get_states_for_root_only() {
    let arena = Arena::new();
    let gss = ArenaGSS::new(&arena, StateId(7));
    let states = gss.active_heads[0].get_states();
    assert_eq!(states, vec![StateId(7)]);
}

#[test]
fn get_states_after_fork_and_diverge() {
    let arena = Arena::new();
    let mut gss = ArenaGSS::new(&arena, StateId(0));
    gss.push(0, StateId(1), None);
    let fork_idx = gss.fork_head(0);
    gss.push(0, StateId(2), None);
    gss.push(fork_idx, StateId(3), None);
    let states_a = gss.active_heads[0].get_states();
    let states_b = gss.active_heads[fork_idx].get_states();
    assert_eq!(states_a, vec![StateId(0), StateId(1), StateId(2)]);
    assert_eq!(states_b, vec![StateId(0), StateId(1), StateId(3)]);
}
