#![allow(clippy::needless_range_loop, unused_imports, unused_variables)]
//! Property-based tests for the arena-allocated GSS implementation.
//!
//! Run with: cargo test -p adze-glr-core --test gss_arena_proptest

use adze_glr_core::gss_arena::{ArenaGSS, ArenaGSSManager, ArenaGSSStats};
use adze_ir::{StateId, SymbolId};
use proptest::prelude::*;
use typed_arena::Arena;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn state_id_strategy() -> impl Strategy<Value = StateId> {
    (0u16..1000).prop_map(StateId)
}

fn symbol_id_strategy() -> impl Strategy<Value = SymbolId> {
    (1u16..500).prop_map(SymbolId)
}

fn opt_symbol_strategy() -> impl Strategy<Value = Option<SymbolId>> {
    prop_oneof![Just(None), (1u16..500).prop_map(|s| Some(SymbolId(s)))]
}

// ---------------------------------------------------------------------------
// 1. Arena allocation and deallocation
// ---------------------------------------------------------------------------

proptest! {
    /// Creating a GSS always yields exactly one active head with the given state.
    #[test]
    fn initial_arena_has_one_head(init in state_id_strategy()) {
        let arena = Arena::new();
        let gss = ArenaGSS::new(&arena, init);

        prop_assert_eq!(gss.active_heads.len(), 1);
        prop_assert_eq!(gss.top_state(0), init);
    }

    /// ArenaGSSManager sessions start with exactly one head.
    #[test]
    fn manager_session_has_one_head(init in state_id_strategy()) {
        let mgr = ArenaGSSManager::new();
        let gss = mgr.new_session(init);

        prop_assert_eq!(gss.active_heads.len(), 1);
        prop_assert_eq!(gss.top_state(0), init);
    }

    /// Clearing the manager and creating a new session works.
    #[test]
    fn manager_clear_and_reuse(s1 in state_id_strategy(), s2 in state_id_strategy()) {
        let mut mgr = ArenaGSSManager::new();
        {
            let mut gss = mgr.new_session(s1);
            gss.push(0, s2, None);
        }
        mgr.clear();
        let gss2 = mgr.new_session(s2);
        prop_assert_eq!(gss2.top_state(0), s2);
    }
}

// ---------------------------------------------------------------------------
// 2. Node creation and linking
// ---------------------------------------------------------------------------

proptest! {
    /// Pushing updates the top state to the new value.
    #[test]
    fn push_updates_top_state(
        init in state_id_strategy(),
        new_state in state_id_strategy(),
        sym in opt_symbol_strategy(),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        gss.push(0, new_state, sym);

        prop_assert_eq!(gss.top_state(0), new_state);
    }

    /// Pushing N times increases depth to N.
    #[test]
    fn push_increases_depth(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 1..20),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        prop_assert_eq!(gss.active_heads[0].depth, pushes.len());
    }

    /// get_states returns the full path from root to current head.
    #[test]
    fn get_states_returns_full_path(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 0..15),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let states = gss.active_heads[0].get_states();
        prop_assert_eq!(states.len(), pushes.len() + 1);
        prop_assert_eq!(states[0], init);
        for i in 0..pushes.len() {
            prop_assert_eq!(states[i + 1], pushes[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Stack forking and merging
// ---------------------------------------------------------------------------

proptest! {
    /// Forking a head duplicates it; both point to the same node.
    #[test]
    fn fork_duplicates_head(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 0..10),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let fork_idx = gss.fork_head(0);
        prop_assert_eq!(gss.active_heads.len(), 2);
        prop_assert_eq!(gss.top_state(0), gss.top_state(fork_idx));
        prop_assert!(std::ptr::eq(gss.active_heads[0], gss.active_heads[fork_idx]));
    }

    /// After forking, pushing on one head doesn't affect the other.
    #[test]
    fn fork_independence(
        init in state_id_strategy(),
        s1 in state_id_strategy(),
        s2 in state_id_strategy(),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        let fork_idx = gss.fork_head(0);
        gss.push(0, s1, None);
        gss.push(fork_idx, s2, None);

        prop_assert_eq!(gss.top_state(0), s1);
        prop_assert_eq!(gss.top_state(fork_idx), s2);
    }

    /// Multiple forks all share the same parent prefix.
    #[test]
    fn multi_fork_shared_prefix(
        init in state_id_strategy(),
        n_forks in 2usize..8,
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        gss.push(0, StateId(999), None);

        let mut fork_indices = vec![0usize];
        for _ in 1..n_forks {
            fork_indices.push(gss.fork_head(0));
        }

        for i in 1..fork_indices.len() {
            prop_assert!(
                gss.active_heads[fork_indices[0]]
                    .shares_prefix_with(gss.active_heads[fork_indices[i]])
            );
        }
    }

    /// Merging two heads with the same state and parent reduces head count.
    #[test]
    fn merge_reduces_heads(init in state_id_strategy()) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        gss.push(0, StateId(42), None);

        let fork_idx = gss.fork_head(0);
        prop_assert_eq!(gss.active_heads.len(), 2);
        prop_assert!(gss.can_merge(0, fork_idx));

        gss.merge_heads(0, fork_idx);
        prop_assert_eq!(gss.active_heads.len(), 1);
    }

    /// can_merge is false for same index.
    #[test]
    fn cannot_merge_self(init in state_id_strategy()) {
        let arena = Arena::new();
        let gss = ArenaGSS::new(&arena, init);
        prop_assert!(!gss.can_merge(0, 0));
    }

    /// Diverged heads (different states) cannot be merged.
    #[test]
    fn diverged_heads_not_mergeable(
        init in state_id_strategy(),
        s1 in state_id_strategy(),
        s2 in state_id_strategy(),
    ) {
        prop_assume!(s1 != s2);
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        let fork_idx = gss.fork_head(0);
        gss.push(0, s1, None);
        gss.push(fork_idx, s2, None);

        prop_assert!(!gss.can_merge(0, fork_idx));
    }
}

// ---------------------------------------------------------------------------
// 4. Stats tracking consistency
// ---------------------------------------------------------------------------

proptest! {
    /// Initial stats: 1 node, max 1 head, 0 forks, 0 merges.
    #[test]
    fn initial_stats(init in state_id_strategy()) {
        let arena = Arena::new();
        let gss = ArenaGSS::new(&arena, init);
        let stats = gss.get_stats();

        prop_assert_eq!(stats.total_nodes_created, 1);
        prop_assert_eq!(stats.max_active_heads, 1);
        prop_assert_eq!(stats.total_forks, 0);
        prop_assert_eq!(stats.total_merges, 0);
    }

    /// Each push increments total_nodes_created by 1.
    #[test]
    fn push_increments_nodes_created(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 1..30),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        prop_assert_eq!(gss.get_stats().total_nodes_created, 1 + pushes.len());
    }

    /// Each fork increments total_forks by 1.
    #[test]
    fn fork_increments_total_forks(
        init in state_id_strategy(),
        n_forks in 1usize..10,
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for _ in 0..n_forks {
            gss.fork_head(0);
        }

        prop_assert_eq!(gss.get_stats().total_forks, n_forks);
    }

    /// max_active_heads is always >= current active head count.
    #[test]
    fn max_heads_tracks_peak(
        init in state_id_strategy(),
        n_forks in 1usize..10,
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for _ in 0..n_forks {
            gss.fork_head(0);
        }

        let expected_heads = 1 + n_forks;
        prop_assert!(gss.get_stats().max_active_heads >= expected_heads);
        prop_assert_eq!(gss.active_heads.len(), expected_heads);
    }

    /// Merge increments total_merges counter.
    #[test]
    fn merge_increments_counter(init in state_id_strategy()) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        let fork_idx = gss.fork_head(0);
        gss.merge_heads(0, fork_idx);

        prop_assert_eq!(gss.get_stats().total_merges, 1);
    }
}

// ---------------------------------------------------------------------------
// 5. Memory usage properties
// ---------------------------------------------------------------------------

proptest! {
    /// ArenaGSSStats default is all zeroes.
    #[test]
    fn stats_default_is_zero(_dummy in 0..1u8) {
        let stats = ArenaGSSStats::default();
        prop_assert_eq!(stats.total_nodes_created, 0);
        prop_assert_eq!(stats.max_active_heads, 0);
        prop_assert_eq!(stats.total_forks, 0);
        prop_assert_eq!(stats.total_merges, 0);
        prop_assert_eq!(stats.arena_bytes_allocated, 0);
    }

    /// Forking does NOT allocate new arena nodes (node count stays same).
    #[test]
    fn fork_shares_memory(
        init in state_id_strategy(),
        n_forks in 1usize..10,
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        let nodes_before = gss.get_stats().total_nodes_created;

        for _ in 0..n_forks {
            gss.fork_head(0);
        }

        // Forking reuses the existing node, no new allocation.
        prop_assert_eq!(gss.get_stats().total_nodes_created, nodes_before);
    }
}

// ---------------------------------------------------------------------------
// 6. Edge cases
// ---------------------------------------------------------------------------

proptest! {
    /// Pop zero elements is a no-op.
    #[test]
    fn pop_zero_is_noop(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 1..5),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let top_before = gss.top_state(0);
        let result = gss.pop(0, 0);
        prop_assert!(result.is_some());
        prop_assert_eq!(gss.top_state(0), top_before);
    }

    /// Popping more than depth returns None.
    #[test]
    fn pop_too_many_returns_none(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 0..5),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let depth = gss.active_heads[0].depth;
        // Popping depth+2 should exceed the stack (depth+1 nodes total).
        let result = gss.pop(0, depth + 2);
        prop_assert!(result.is_none());
    }

    /// Single-node arena: initial node has depth 0 and no parent.
    #[test]
    fn single_node_properties(init in state_id_strategy()) {
        let arena = Arena::new();
        let gss = ArenaGSS::new(&arena, init);

        let head = gss.active_heads[0];
        prop_assert_eq!(head.depth, 0);
        prop_assert!(head.parent.is_none());
        prop_assert!(head.symbol.is_none());
        prop_assert_eq!(head.state, init);
    }
}

// ---------------------------------------------------------------------------
// 7. Concurrent-like access patterns (sequential multi-session)
// ---------------------------------------------------------------------------

proptest! {
    /// Multiple sequential sessions from the same manager are independent.
    #[test]
    fn sequential_sessions_independent(
        s1 in state_id_strategy(),
        s2 in state_id_strategy(),
        pushes1 in prop::collection::vec(state_id_strategy(), 1..5),
        _pushes2 in prop::collection::vec(state_id_strategy(), 1..5),
    ) {
        let mgr = ArenaGSSManager::new();

        let stats1 = {
            let mut gss = mgr.new_session(s1);
            for s in &pushes1 {
                gss.push(0, *s, None);
            }
            gss.get_stats().total_nodes_created
        };

        // Second session in the same arena accumulates nodes, but its own
        // view starts fresh.
        let gss2 = mgr.new_session(s2);
        prop_assert_eq!(gss2.active_heads.len(), 1);
        prop_assert_eq!(gss2.top_state(0), s2);
        // The first session created 1+pushes1.len() nodes.
        prop_assert_eq!(stats1, 1 + pushes1.len());
    }
}

// ---------------------------------------------------------------------------
// 8. Node traversal
// ---------------------------------------------------------------------------

proptest! {
    /// Pop returns states in bottom-to-top order for the popped segment.
    #[test]
    fn pop_returns_correct_states(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 2..10),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let pop_count = pushes.len() / 2;
        let popped = gss.pop(0, pop_count).unwrap();
        prop_assert_eq!(popped.len(), pop_count);

        // Popped states should be the last `pop_count` pushes in order.
        let expected: Vec<StateId> = pushes[pushes.len() - pop_count..].to_vec();
        prop_assert_eq!(popped, expected);
    }

    /// After pop, top state matches the state just below the popped segment.
    #[test]
    fn pop_restores_correct_top(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 2..10),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let pop_count = pushes.len() / 2;
        gss.pop(0, pop_count);

        let remaining = pushes.len() - pop_count;
        let expected_top = if remaining > 0 {
            pushes[remaining - 1]
        } else {
            init
        };
        prop_assert_eq!(gss.top_state(0), expected_top);
    }

    /// shares_prefix_with is symmetric.
    #[test]
    fn shares_prefix_symmetric(
        init in state_id_strategy(),
        s1 in state_id_strategy(),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        gss.push(0, s1, None);

        let fork_idx = gss.fork_head(0);
        let a = gss.active_heads[0];
        let b = gss.active_heads[fork_idx];

        prop_assert_eq!(a.shares_prefix_with(b), b.shares_prefix_with(a));
    }
}

// ---------------------------------------------------------------------------
// 9. Large-scale arena operations
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Pushing many states doesn't corrupt the stack path.
    #[test]
    fn large_stack_integrity(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 50..200),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let states = gss.active_heads[0].get_states();
        prop_assert_eq!(states.len(), pushes.len() + 1);
        prop_assert_eq!(states[0], init);
        for i in 0..pushes.len() {
            prop_assert_eq!(states[i + 1], pushes[i]);
        }
    }

    /// Many forks followed by deduplicate merges identical heads.
    #[test]
    fn large_fork_deduplicate(
        init in state_id_strategy(),
        n_forks in 10usize..50,
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for _ in 0..n_forks {
            gss.fork_head(0);
        }

        prop_assert_eq!(gss.active_heads.len(), 1 + n_forks);

        gss.deduplicate();
        prop_assert_eq!(gss.active_heads.len(), 1);
        prop_assert!(gss.get_stats().total_merges > 0);
    }

    /// Fork-push-merge cycle preserves stats consistency.
    #[test]
    fn fork_push_merge_cycle(
        init in state_id_strategy(),
        rounds in 3usize..15,
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        let mut total_forks = 0usize;

        for round in 0..rounds {
            let fork_idx = gss.fork_head(0);
            total_forks += 1;
            // Diverge the fork so it can't be merged
            gss.push(fork_idx, StateId(round as u16 + 500), None);
        }

        prop_assert_eq!(gss.get_stats().total_forks, total_forks);
        // 1 initial node + `rounds` pushes on forked heads
        prop_assert_eq!(gss.get_stats().total_nodes_created, 1 + rounds);
    }
}

// ---------------------------------------------------------------------------
// 10. Arena reset / clear
// ---------------------------------------------------------------------------

proptest! {
    /// After manager.clear(), a new session is fully fresh.
    #[test]
    fn clear_resets_everything(
        s1 in state_id_strategy(),
        s2 in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 1..10),
    ) {
        let mut mgr = ArenaGSSManager::new();

        {
            let mut gss = mgr.new_session(s1);
            for s in &pushes {
                gss.push(0, *s, None);
            }
            gss.fork_head(0);
        }

        mgr.clear();

        let gss2 = mgr.new_session(s2);
        prop_assert_eq!(gss2.active_heads.len(), 1);
        prop_assert_eq!(gss2.top_state(0), s2);
        prop_assert_eq!(gss2.get_stats().total_nodes_created, 1);
        prop_assert_eq!(gss2.get_stats().total_forks, 0);
        prop_assert_eq!(gss2.get_stats().total_merges, 0);
    }

    /// Default manager can be created and used immediately.
    #[test]
    fn default_manager(init in state_id_strategy()) {
        let mgr = ArenaGSSManager::default();
        let gss = mgr.new_session(init);
        prop_assert_eq!(gss.top_state(0), init);
    }
}

// ---------------------------------------------------------------------------
// Additional property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Deduplicate is idempotent: calling it twice yields the same result.
    #[test]
    fn deduplicate_idempotent(
        init in state_id_strategy(),
        n_forks in 2usize..10,
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for _ in 0..n_forks {
            gss.fork_head(0);
        }

        gss.deduplicate();
        let heads_after_first = gss.active_heads.len();
        let merges_after_first = gss.get_stats().total_merges;

        gss.deduplicate();
        prop_assert_eq!(gss.active_heads.len(), heads_after_first);
        prop_assert_eq!(gss.get_stats().total_merges, merges_after_first);
    }

    /// Pop then push restores depth correctly.
    #[test]
    fn pop_push_depth_consistency(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 3..10),
        new_state in state_id_strategy(),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        let pop_count = 2;
        gss.pop(0, pop_count);
        let depth_after_pop = gss.active_heads[0].depth;

        gss.push(0, new_state, None);
        prop_assert_eq!(gss.active_heads[0].depth, depth_after_pop + 1);
    }

    /// completed_heads starts empty and is independent of active_heads.
    #[test]
    fn completed_heads_initially_empty(
        init in state_id_strategy(),
        pushes in prop::collection::vec(state_id_strategy(), 0..5),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        for s in &pushes {
            gss.push(0, *s, None);
        }

        prop_assert!(gss.completed_heads.is_empty());
    }

    /// Forked and diverged heads have different get_states paths.
    #[test]
    fn diverged_paths_differ(
        init in state_id_strategy(),
        s1 in state_id_strategy(),
        s2 in state_id_strategy(),
    ) {
        prop_assume!(s1 != s2);
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);

        let fork_idx = gss.fork_head(0);
        gss.push(0, s1, None);
        gss.push(fork_idx, s2, None);

        let path0 = gss.active_heads[0].get_states();
        let path1 = gss.active_heads[fork_idx].get_states();

        // Both share the root but differ in the last element.
        prop_assert_eq!(path0[0], init);
        prop_assert_eq!(path1[0], init);
        prop_assert_ne!(path0.last(), path1.last());
    }

    /// Push with symbol preserves the symbol in the node.
    #[test]
    fn push_preserves_symbol(
        init in state_id_strategy(),
        new_state in state_id_strategy(),
        sym in symbol_id_strategy(),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        gss.push(0, new_state, Some(sym));

        prop_assert_eq!(gss.active_heads[0].symbol, Some(sym));
    }

    /// Push with None symbol stores None.
    #[test]
    fn push_none_symbol(
        init in state_id_strategy(),
        new_state in state_id_strategy(),
    ) {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, init);
        gss.push(0, new_state, None);

        prop_assert_eq!(gss.active_heads[0].symbol, None);
    }
}
