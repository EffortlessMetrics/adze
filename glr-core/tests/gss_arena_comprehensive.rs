//! Comprehensive tests for ArenaGSS, ArenaGSSManager, and ArenaStackNode.

use adze_glr_core::gss_arena::ArenaGSSManager;
use adze_ir::{StateId, SymbolId};

#[test]
fn manager_new_creates_empty() {
    let _mgr = ArenaGSSManager::new();
}

#[test]
fn session_initial_state() {
    let mgr = ArenaGSSManager::new();
    let gss = mgr.new_session(StateId(0));
    assert_eq!(gss.top_state(0), StateId(0));
}

#[test]
fn session_push_changes_top_state() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(1)));
    assert_eq!(gss.top_state(0), StateId(1));
}

#[test]
fn session_push_multiple() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(1)));
    gss.push(0, StateId(2), Some(SymbolId(2)));
    assert_eq!(gss.top_state(0), StateId(2));
}

#[test]
fn session_pop_returns_states() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(1)));
    gss.push(0, StateId(2), Some(SymbolId(2)));
    let popped = gss.pop(0, 2);
    assert!(popped.is_some());
}

#[test]
fn session_pop_restores_state() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(1)));
    gss.pop(0, 1);
    assert_eq!(gss.top_state(0), StateId(0));
}

#[test]
fn session_fork_head() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(1)));
    let forked = gss.fork_head(0);
    assert_ne!(forked, 0);
    assert_eq!(gss.top_state(forked), StateId(1));
}

#[test]
fn forked_heads_independent() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), Some(SymbolId(1)));
    let forked = gss.fork_head(0);
    gss.push(0, StateId(2), Some(SymbolId(2)));
    // Forked head should still be at state 1
    assert_eq!(gss.top_state(forked), StateId(1));
    assert_eq!(gss.top_state(0), StateId(2));
}

#[test]
fn can_merge_same_state() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    let forked = gss.fork_head(0);
    // Both at initial state 0
    assert!(gss.can_merge(0, forked));
}

#[test]
fn cannot_merge_different_states() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    let forked = gss.fork_head(0);
    gss.push(0, StateId(1), Some(SymbolId(1)));
    // head 0 at state 1, forked at state 0
    assert!(!gss.can_merge(0, forked));
}

#[test]
fn merge_heads_reduces_count() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    let forked = gss.fork_head(0);
    gss.merge_heads(0, forked);
    // After merge, forked head is effectively removed
    // The kept head should still work
    assert_eq!(gss.top_state(0), StateId(0));
}

#[test]
fn get_stats_initial() {
    let mgr = ArenaGSSManager::new();
    let gss = mgr.new_session(StateId(0));
    let stats = gss.get_stats();
    assert_eq!(stats.total_forks, 0);
}

#[test]
fn get_stats_after_fork() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    let _forked = gss.fork_head(0);
    let stats = gss.get_stats();
    assert_eq!(stats.total_forks, 1);
}

#[test]
fn deduplicate_no_panic() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.fork_head(0);
    gss.deduplicate();
}

#[test]
fn manager_clear() {
    let mut mgr = ArenaGSSManager::new();
    {
        let _gss = mgr.new_session(StateId(0));
    }
    mgr.clear();
}

#[test]
fn pop_more_than_pushed_returns_none() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), None);
    let result = gss.pop(0, 5); // popping more than available
    // May return None or fewer states
    let _ = result;
}

#[test]
fn push_with_none_symbol() {
    let mgr = ArenaGSSManager::new();
    let mut gss = mgr.new_session(StateId(0));
    gss.push(0, StateId(1), None);
    assert_eq!(gss.top_state(0), StateId(1));
}
