#![cfg(feature = "test-api")]

use adze_glr_core::StateId;
use adze_glr_core::SymbolId;
use adze_glr_core::gss::GraphStructuredStack;
use adze_glr_core::gss_arena::{ArenaGSS, ArenaGSSManager};
use adze_glr_core::stack::StackNode;
use typed_arena::Arena;

// ---------------------------------------------------------------------------
// GSS (Rc-based GraphStructuredStack) tests
// ---------------------------------------------------------------------------

mod gss_tests {
    use super::*;

    #[test]
    fn new_creates_single_head_at_initial_state() {
        let gss = GraphStructuredStack::new(StateId(0));
        assert_eq!(gss.active_heads.len(), 1);
        assert_eq!(gss.top_state(0), StateId(0));
        assert_eq!(gss.get_stats().total_nodes_created, 1);
    }

    #[test]
    fn push_updates_top_state() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(5), Some(SymbolId(42)));
        assert_eq!(gss.top_state(0), StateId(5));
        assert_eq!(gss.get_stats().total_nodes_created, 2);
    }

    #[test]
    fn pop_returns_states_in_order() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(1), None);
        gss.push(0, StateId(2), None);
        gss.push(0, StateId(3), None);

        let popped = gss.pop(0, 2).unwrap();
        // pop returns the popped states from bottom to top
        assert_eq!(popped, vec![StateId(2), StateId(3)]);
        assert_eq!(gss.top_state(0), StateId(1));
    }

    #[test]
    fn pop_too_many_returns_none() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(1), None);
        // Try to pop 5 states when only 2 exist (initial + 1 pushed)
        assert!(gss.pop(0, 5).is_none());
    }

    #[test]
    fn fork_creates_independent_head() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(1), None);

        let fork_idx = gss.fork_head(0);
        assert_eq!(gss.active_heads.len(), 2);
        assert_eq!(gss.top_state(0), gss.top_state(fork_idx));

        // Diverge the two heads
        gss.push(0, StateId(10), None);
        gss.push(fork_idx, StateId(20), None);
        assert_eq!(gss.top_state(0), StateId(10));
        assert_eq!(gss.top_state(fork_idx), StateId(20));
    }

    #[test]
    fn merge_deduplicates_same_state_same_parent() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(1), None);

        // Fork and push same state on both heads
        let fork_idx = gss.fork_head(0);
        gss.push(0, StateId(2), None);
        gss.push(fork_idx, StateId(2), None);

        assert!(gss.can_merge(0, fork_idx));
        gss.deduplicate();
        assert_eq!(gss.active_heads.len(), 1);
        assert_eq!(gss.get_stats().total_merges, 1);
    }

    #[test]
    fn cannot_merge_different_states() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(1), None);

        let fork_idx = gss.fork_head(0);
        gss.push(0, StateId(2), None);
        gss.push(fork_idx, StateId(3), None);

        assert!(!gss.can_merge(0, fork_idx));
    }

    #[test]
    fn mark_completed_moves_head() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(1), None);

        // Fork so we still have an active head after marking one completed
        let _fork = gss.fork_head(0);
        assert_eq!(gss.active_heads.len(), 2);

        gss.mark_completed(0);
        assert_eq!(gss.active_heads.len(), 1);
        assert_eq!(gss.completed_heads.len(), 1);
    }

    #[test]
    fn get_states_returns_root_to_head_path() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.push(0, StateId(1), None);
        gss.push(0, StateId(2), None);

        let states = gss.active_heads[0].get_states();
        assert_eq!(states, vec![StateId(0), StateId(1), StateId(2)]);
    }

    #[test]
    fn stats_track_forks_and_max_heads() {
        let mut gss = GraphStructuredStack::new(StateId(0));
        gss.fork_head(0);
        gss.fork_head(0);
        gss.fork_head(0);

        assert_eq!(gss.get_stats().total_forks, 3);
        assert_eq!(gss.get_stats().max_active_heads, 4);
    }
}

// ---------------------------------------------------------------------------
// ArenaGSS tests
// ---------------------------------------------------------------------------

mod arena_gss_tests {
    use super::*;

    #[test]
    fn arena_new_creates_single_head() {
        let arena = Arena::new();
        let gss = ArenaGSS::new(&arena, StateId(0));
        assert_eq!(gss.active_heads.len(), 1);
        assert_eq!(gss.top_state(0), StateId(0));
    }

    #[test]
    fn arena_push_pop_roundtrip() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        gss.push(0, StateId(1), Some(SymbolId(10)));
        gss.push(0, StateId(2), None);
        gss.push(0, StateId(3), Some(SymbolId(30)));

        assert_eq!(gss.top_state(0), StateId(3));

        let popped = gss.pop(0, 2).unwrap();
        assert_eq!(popped, vec![StateId(2), StateId(3)]);
        assert_eq!(gss.top_state(0), StateId(1));
    }

    #[test]
    fn arena_fork_shares_parent_pointers() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        gss.push(0, StateId(1), None);
        gss.push(0, StateId(2), None);

        let f1 = gss.fork_head(0);
        let f2 = gss.fork_head(0);

        // All three heads share the same parent pointer
        assert!(gss.active_heads[0].shares_prefix_with(gss.active_heads[f1]));
        assert!(gss.active_heads[0].shares_prefix_with(gss.active_heads[f2]));
        assert!(std::ptr::eq(
            gss.active_heads[0].parent.unwrap(),
            gss.active_heads[f1].parent.unwrap()
        ));
    }

    #[test]
    fn arena_deduplicate_merges_compatible_heads() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        gss.push(0, StateId(1), None);
        let fork = gss.fork_head(0);
        gss.push(0, StateId(5), None);
        gss.push(fork, StateId(5), None);

        assert!(gss.can_merge(0, fork));
        gss.deduplicate();
        assert_eq!(gss.active_heads.len(), 1);
        assert_eq!(gss.get_stats().total_merges, 1);
    }

    #[test]
    fn arena_manager_creates_sessions() {
        let manager = ArenaGSSManager::new();
        {
            let mut session = manager.new_session(StateId(0));
            session.push(0, StateId(1), None);
            session.push(0, StateId(2), None);
            assert_eq!(session.top_state(0), StateId(2));
            assert_eq!(session.get_stats().total_nodes_created, 3);
        }
    }

    #[test]
    fn arena_manager_default_impl() {
        let manager = ArenaGSSManager::default();
        let session = manager.new_session(StateId(42));
        assert_eq!(session.top_state(0), StateId(42));
    }

    #[test]
    fn arena_get_states_traces_full_path() {
        let arena = Arena::new();
        let mut gss = ArenaGSS::new(&arena, StateId(0));
        gss.push(0, StateId(10), None);
        gss.push(0, StateId(20), None);
        gss.push(0, StateId(30), None);

        let states = gss.active_heads[0].get_states();
        assert_eq!(
            states,
            vec![StateId(0), StateId(10), StateId(20), StateId(30)]
        );
    }
}

// ---------------------------------------------------------------------------
// Persistent StackNode tests
// ---------------------------------------------------------------------------

mod stack_tests {
    use super::*;

    #[test]
    fn push_pop_basic() {
        let mut stack = StackNode::new();
        stack.push(1, None);
        stack.push(2, Some(100));
        stack.push(3, None);

        assert_eq!(stack.depth(), 3);
        assert_eq!(stack.pop(), Some((3, None)));
        assert_eq!(stack.pop(), Some((2, Some(100))));
        assert_eq!(stack.pop(), Some((1, None)));
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
    }

    #[test]
    fn fork_shares_structure() {
        let mut s1 = StackNode::with_state(1);
        s1.push(2, None);
        s1.push(3, None);

        let s2 = s1.fork();
        assert_eq!(s1.to_vec(), vec![1, 2, 3]);
        assert_eq!(s2.to_vec(), vec![1, 2, 3]);

        // Mutating the original doesn't affect the fork
        s1.push(4, None);
        assert_eq!(s1.to_vec(), vec![1, 2, 3, 4]);
        assert_eq!(s2.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn spill_to_tail_and_recover() {
        let mut stack = StackNode::new();
        // Push enough entries to trigger at least one spill
        for i in 1..=20 {
            stack.push(i, None);
        }
        assert_eq!(stack.depth(), 20);
        assert_eq!(stack.top(), Some(20));

        // Pop everything back in reverse order
        for i in (1..=20).rev() {
            assert_eq!(stack.pop(), Some((i, None)));
        }
        assert!(stack.is_empty());
    }

    #[test]
    fn top_returns_state_not_symbol() {
        let mut stack = StackNode::new();
        stack.push(7, Some(11));
        assert_eq!(stack.top(), Some(7));

        stack.push(9, None);
        assert_eq!(stack.top(), Some(9));
    }

    #[test]
    fn can_merge_with_matching_stacks() {
        let mut s1 = StackNode::with_state(1);
        s1.push(2, None);
        s1.push(3, None);

        let s2 = s1.fork();
        assert!(s1.can_merge_with(&s2));
    }

    #[test]
    fn cannot_merge_different_depth() {
        let mut s1 = StackNode::with_state(1);
        s1.push(2, None);

        let mut s2 = StackNode::with_state(1);
        s2.push(2, None);
        s2.push(3, None);

        assert!(!s1.can_merge_with(&s2));
    }

    #[test]
    fn to_vec_preserves_order() {
        let mut stack = StackNode::with_state(10);
        stack.push(20, None);
        stack.push(30, Some(5));
        stack.push(40, None);

        // to_vec returns states only (no symbols), root to top
        assert_eq!(stack.to_vec(), vec![10, 20, 30, 40]);
    }

    #[test]
    fn empty_stack_properties() {
        let stack = StackNode::new();
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);
        assert_eq!(stack.top(), None);
        assert_eq!(stack.to_vec(), Vec::<u16>::new());
    }

    #[test]
    fn from_raw_constructs_valid_node() {
        // head must be pairs: [state, sym, state, sym]
        let node = StackNode::from_raw(5, vec![10, u16::MAX], None);
        assert_eq!(node.top(), Some(10));
        assert_eq!(node.state, 5);
    }

    #[test]
    fn glr_stack_trait_on_stack_node() {
        use adze_glr_core::stack::test_helpers::GlrStack;

        let mut stack = StackNode::new();
        GlrStack::push(&mut stack, 1);
        GlrStack::push(&mut stack, 2);
        GlrStack::push(&mut stack, 3);

        assert_eq!(GlrStack::len(&stack), 3);
        assert_eq!(GlrStack::peek(&stack), Some(3));
        assert_eq!(GlrStack::pop(&mut stack), Some(3));
        assert_eq!(GlrStack::len(&stack), 2);
        assert!(!GlrStack::is_empty(&stack));
    }

    #[test]
    fn glr_stack_trait_on_vec() {
        use adze_glr_core::stack::test_helpers::GlrStack;

        let mut v: Vec<u16> = Vec::new();
        GlrStack::push(&mut v, 10);
        GlrStack::push(&mut v, 20);

        assert_eq!(GlrStack::peek(&v), Some(20));
        assert_eq!(GlrStack::pop(&mut v), Some(20));
        assert_eq!(GlrStack::len(&v), 1);
    }

    #[test]
    fn fork_after_spill_is_independent() {
        let mut s1 = StackNode::new();
        // Push enough to trigger spill
        for i in 1..=10 {
            s1.push(i, None);
        }

        let mut s2 = s1.fork();

        // Push more on original
        s1.push(99, None);
        // Push different value on fork
        s2.push(88, None);

        assert_eq!(s1.top(), Some(99));
        assert_eq!(s2.top(), Some(88));
        assert_eq!(s1.depth(), 11);
        assert_eq!(s2.depth(), 11);
    }

    #[test]
    fn last_is_alias_for_top() {
        let mut stack = StackNode::with_state(5);
        stack.push(10, None);
        assert_eq!(stack.last(), stack.top());
        assert_eq!(stack.last(), Some(10));
    }
}
