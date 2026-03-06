//! Comprehensive tests for the `Action` enum — v4 edition.
//!
//! Covers construction, equality, hashing, serialization, pattern matching,
//! and edge cases for all six variants: Shift, Reduce, Accept, Error, Recover, Fork.

use adze_glr_core::{Action, RuleId, StateId};
use std::collections::HashSet;

// ── 1. Shift construction and properties (8 tests) ─────────────────────────

#[test]
fn shift_with_zero_state() {
    let a = Action::Shift(StateId(0));
    assert_eq!(a, Action::Shift(StateId(0)));
}

#[test]
fn shift_with_nonzero_state() {
    let a = Action::Shift(StateId(42));
    assert_eq!(a, Action::Shift(StateId(42)));
}

#[test]
fn shift_with_max_state() {
    let a = Action::Shift(StateId(u16::MAX));
    assert_eq!(a, Action::Shift(StateId(u16::MAX)));
}

#[test]
fn shift_inner_state_id_accessible() {
    if let Action::Shift(sid) = Action::Shift(StateId(100)) {
        assert_eq!(sid.0, 100);
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn shift_with_one() {
    assert_eq!(Action::Shift(StateId(1)), Action::Shift(StateId(1)));
}

#[test]
fn shift_boundary_255() {
    assert_eq!(Action::Shift(StateId(255)), Action::Shift(StateId(255)));
}

#[test]
fn shift_boundary_256() {
    assert_eq!(Action::Shift(StateId(256)), Action::Shift(StateId(256)));
}

#[test]
fn shift_debug_contains_state() {
    let dbg = format!("{:?}", Action::Shift(StateId(77)));
    assert!(dbg.contains("Shift"));
    assert!(dbg.contains("77"));
}

// ── 2. Reduce construction and properties (8 tests) ────────────────────────

#[test]
fn reduce_with_zero_rule() {
    let a = Action::Reduce(RuleId(0));
    assert_eq!(a, Action::Reduce(RuleId(0)));
}

#[test]
fn reduce_with_nonzero_rule() {
    let a = Action::Reduce(RuleId(7));
    assert_eq!(a, Action::Reduce(RuleId(7)));
}

#[test]
fn reduce_with_max_rule() {
    let a = Action::Reduce(RuleId(u16::MAX));
    assert_eq!(a, Action::Reduce(RuleId(u16::MAX)));
}

#[test]
fn reduce_inner_rule_id_accessible() {
    if let Action::Reduce(rid) = Action::Reduce(RuleId(55)) {
        assert_eq!(rid.0, 55);
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn reduce_with_one() {
    assert_eq!(Action::Reduce(RuleId(1)), Action::Reduce(RuleId(1)));
}

#[test]
fn reduce_large_rule() {
    assert_eq!(Action::Reduce(RuleId(1000)), Action::Reduce(RuleId(1000)));
}

#[test]
fn reduce_boundary_255() {
    assert_eq!(Action::Reduce(RuleId(255)), Action::Reduce(RuleId(255)));
}

#[test]
fn reduce_debug_contains_rule() {
    let dbg = format!("{:?}", Action::Reduce(RuleId(33)));
    assert!(dbg.contains("Reduce"));
    assert!(dbg.contains("33"));
}

// ── 3. Accept, Error, and Recover (5 tests) ────────────────────────────────

#[test]
fn accept_construction() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn error_construction() {
    assert_eq!(Action::Error, Action::Error);
}

#[test]
fn recover_construction() {
    assert_eq!(Action::Recover, Action::Recover);
}

#[test]
fn accept_debug_string() {
    assert_eq!(format!("{:?}", Action::Accept), "Accept");
}

#[test]
fn error_debug_string() {
    assert_eq!(format!("{:?}", Action::Error), "Error");
}

// ── 4. Action variant predicates via matches! (8 tests) ────────────────────

#[test]
fn shift_matches_shift() {
    assert!(matches!(Action::Shift(StateId(0)), Action::Shift(_)));
}

#[test]
fn shift_does_not_match_reduce() {
    assert!(!matches!(Action::Shift(StateId(0)), Action::Reduce(_)));
}

#[test]
fn reduce_matches_reduce() {
    assert!(matches!(Action::Reduce(RuleId(5)), Action::Reduce(_)));
}

#[test]
fn reduce_does_not_match_shift() {
    assert!(!matches!(Action::Reduce(RuleId(5)), Action::Shift(_)));
}

#[test]
fn accept_matches_accept() {
    assert!(matches!(Action::Accept, Action::Accept));
}

#[test]
fn error_matches_error() {
    assert!(matches!(Action::Error, Action::Error));
}

#[test]
fn recover_matches_recover() {
    assert!(matches!(Action::Recover, Action::Recover));
}

#[test]
fn fork_matches_fork() {
    assert!(matches!(Action::Fork(vec![]), Action::Fork(_)));
}

// ── 5. Action equality and hashing (5 tests) ───────────────────────────────

#[test]
fn eq_same_shift() {
    assert_eq!(Action::Shift(StateId(10)), Action::Shift(StateId(10)));
}

#[test]
fn ne_different_shift_states() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
}

#[test]
fn hash_set_deduplicates_identical_actions() {
    let mut set = HashSet::new();
    set.insert(Action::Accept);
    set.insert(Action::Accept);
    set.insert(Action::Error);
    assert_eq!(set.len(), 2);
}

#[test]
fn hash_set_distinguishes_shifts() {
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(0)));
    set.insert(Action::Shift(StateId(1)));
    set.insert(Action::Shift(StateId(0)));
    assert_eq!(set.len(), 2);
}

#[test]
fn hash_set_distinguishes_reduces() {
    let mut set = HashSet::new();
    set.insert(Action::Reduce(RuleId(0)));
    set.insert(Action::Reduce(RuleId(1)));
    set.insert(Action::Reduce(RuleId(0)));
    assert_eq!(set.len(), 2);
}

// ── 6. Action serialization with serde_json (5 tests) ──────────────────────

#[test]
fn serde_roundtrip_shift() {
    let a = Action::Shift(StateId(42));
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_reduce() {
    let a = Action::Reduce(RuleId(99));
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_accept() {
    let a = Action::Accept;
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_error() {
    let a = Action::Error;
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_fork() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ]);
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

// ── 7. Action pattern matching (8 tests) ───────────────────────────────────

#[test]
fn match_shift_extracts_state() {
    let a = Action::Shift(StateId(99));
    let state = match a {
        Action::Shift(s) => s.0,
        _ => panic!("expected Shift"),
    };
    assert_eq!(state, 99);
}

#[test]
fn match_reduce_extracts_rule() {
    let a = Action::Reduce(RuleId(44));
    let rule = match a {
        Action::Reduce(r) => r.0,
        _ => panic!("expected Reduce"),
    };
    assert_eq!(rule, 44);
}

#[test]
fn match_all_variants_exhaustive() {
    let actions = [
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];
    for a in &actions {
        let label = match a {
            Action::Shift(_) => "shift",
            Action::Reduce(_) => "reduce",
            Action::Accept => "accept",
            Action::Error => "error",
            Action::Recover => "recover",
            Action::Fork(_) => "fork",
            _ => "unknown",
        };
        assert_ne!(label, "unknown");
    }
}

#[test]
fn match_fork_extracts_inner_vec() {
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    if let Action::Fork(ref v) = a {
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], Action::Accept);
        assert_eq!(v[1], Action::Error);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn match_shift_with_specific_value() {
    let a = Action::Shift(StateId(7));
    assert!(matches!(a, Action::Shift(StateId(7))));
    assert!(!matches!(a, Action::Shift(StateId(8))));
}

#[test]
fn match_reduce_with_specific_value() {
    let a = Action::Reduce(RuleId(3));
    assert!(matches!(a, Action::Reduce(RuleId(3))));
    assert!(!matches!(a, Action::Reduce(RuleId(4))));
}

#[test]
fn match_fork_with_guard() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))]);
    assert!(matches!(a, Action::Fork(ref v) if v.len() == 2));
}

#[test]
fn match_nested_fork() {
    let inner = Action::Fork(vec![Action::Accept]);
    let outer = Action::Fork(vec![inner, Action::Error]);
    if let Action::Fork(ref v) = outer {
        assert!(matches!(v[0], Action::Fork(_)));
        assert!(matches!(v[1], Action::Error));
    } else {
        panic!("expected Fork");
    }
}

// ── 8. Edge cases (8 tests) ────────────────────────────────────────────────

#[test]
fn max_state_id_shift() {
    let a = Action::Shift(StateId(u16::MAX));
    if let Action::Shift(s) = a {
        assert_eq!(s.0, u16::MAX);
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn max_rule_id_reduce() {
    let a = Action::Reduce(RuleId(u16::MAX));
    if let Action::Reduce(r) = a {
        assert_eq!(r.0, u16::MAX);
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn zero_ids_shift_and_reduce() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn all_unit_variants_distinct() {
    let variants: [Action; 3] = [Action::Accept, Action::Error, Action::Recover];
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(variants[i], variants[j]);
        }
    }
}

#[test]
fn fork_empty_vs_nonempty() {
    assert_ne!(Action::Fork(vec![]), Action::Fork(vec![Action::Accept]));
}

#[test]
fn fork_preserves_insertion_order() {
    let a = Action::Fork(vec![
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(1)),
        Action::Accept,
    ]);
    if let Action::Fork(ref v) = a {
        assert_eq!(v[0], Action::Reduce(RuleId(0)));
        assert_eq!(v[1], Action::Shift(StateId(1)));
        assert_eq!(v[2], Action::Accept);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn clone_fork_is_independent() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let mut b = a.clone();
    if let Action::Fork(ref mut v) = b {
        v.push(Action::Accept);
    }
    // Original unchanged
    if let Action::Fork(ref v) = a {
        assert_eq!(v.len(), 1);
    }
}

#[test]
fn large_fork_many_actions() {
    let actions: Vec<Action> = (0..100).map(|i| Action::Shift(StateId(i))).collect();
    let fork = Action::Fork(actions);
    if let Action::Fork(ref v) = fork {
        assert_eq!(v.len(), 100);
        assert_eq!(v[0], Action::Shift(StateId(0)));
        assert_eq!(v[99], Action::Shift(StateId(99)));
    } else {
        panic!("expected Fork");
    }
}

// ── 9. Additional coverage — Clone, Hash, cross-variant (8+ tests) ────────

#[test]
fn clone_shift_equals_original() {
    let a = Action::Shift(StateId(10));
    assert_eq!(a.clone(), a);
}

#[test]
fn clone_reduce_equals_original() {
    let a = Action::Reduce(RuleId(5));
    assert_eq!(a.clone(), a);
}

#[test]
fn clone_accept_equals_original() {
    assert_eq!(Action::Accept.clone(), Action::Accept);
}

#[test]
fn clone_error_equals_original() {
    assert_eq!(Action::Error.clone(), Action::Error);
}

#[test]
fn clone_recover_equals_original() {
    assert_eq!(Action::Recover.clone(), Action::Recover);
}

#[test]
fn hash_set_all_six_variant_kinds() {
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(0)));
    set.insert(Action::Reduce(RuleId(0)));
    set.insert(Action::Accept);
    set.insert(Action::Error);
    set.insert(Action::Recover);
    set.insert(Action::Fork(vec![]));
    assert_eq!(set.len(), 6);
}

#[test]
fn ne_shift_vs_accept() {
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

#[test]
fn ne_reduce_vs_error() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Error);
}

#[test]
fn ne_accept_vs_recover() {
    assert_ne!(Action::Accept, Action::Recover);
}

#[test]
fn ne_shift_vs_fork() {
    assert_ne!(
        Action::Shift(StateId(0)),
        Action::Fork(vec![Action::Shift(StateId(0))])
    );
}

#[test]
fn serde_roundtrip_recover() {
    let a = Action::Recover;
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_nested_fork() {
    let inner = Action::Fork(vec![Action::Accept]);
    let outer = Action::Fork(vec![inner, Action::Recover]);
    let json = serde_json::to_string(&outer).unwrap();
    let back: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(outer, back);
}

#[test]
fn serde_roundtrip_max_ids() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(u16::MAX)),
        Action::Reduce(RuleId(u16::MAX)),
    ]);
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn actions_collected_into_vec() {
    let actions: Vec<Action> = [
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(1)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ]
    .to_vec();
    assert_eq!(actions.len(), 6);
}

#[test]
fn recover_debug_string() {
    assert_eq!(format!("{:?}", Action::Recover), "Recover");
}

#[test]
fn fork_debug_contains_inner() {
    let dbg = format!("{:?}", Action::Fork(vec![Action::Accept]));
    assert!(dbg.contains("Fork"));
    assert!(dbg.contains("Accept"));
}
