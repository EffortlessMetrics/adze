//! Tests for Action enum encoding: construction, field access, display,
//! comparison, classification, and edge cases.

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};

// ===========================================================================
// 1. Shift creation and field access (8 tests)
// ===========================================================================

#[test]
fn shift_zero_state_id() {
    let a = Action::Shift(StateId(0));
    match a {
        Action::Shift(s) => assert_eq!(s.0, 0),
        _ => panic!("expected Shift"),
    }
}

#[test]
fn shift_one_state_id() {
    let a = Action::Shift(StateId(1));
    match a {
        Action::Shift(s) => assert_eq!(s, StateId(1)),
        _ => panic!("expected Shift"),
    }
}

#[test]
fn shift_max_u16_state_id() {
    let a = Action::Shift(StateId(u16::MAX));
    match a {
        Action::Shift(s) => assert_eq!(s.0, u16::MAX),
        _ => panic!("expected Shift"),
    }
}

#[test]
fn shift_inner_value_accessible() {
    let a = Action::Shift(StateId(42));
    if let Action::Shift(s) = a {
        assert_eq!(s.0, 42);
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn shift_copies_state_id() {
    let sid = StateId(99);
    let a = Action::Shift(sid);
    // sid is Copy — use it again without .clone()
    assert_eq!(sid, StateId(99));
    assert_eq!(a, Action::Shift(StateId(99)));
}

#[test]
fn shift_preserves_state_id_across_binding() {
    let a = Action::Shift(StateId(500));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn shift_two_different_states_not_equal() {
    let a = Action::Shift(StateId(1));
    let b = Action::Shift(StateId(2));
    assert_ne!(a, b);
}

#[test]
fn shift_same_state_equal() {
    let a = Action::Shift(StateId(7));
    let b = Action::Shift(StateId(7));
    assert_eq!(a, b);
}

// ===========================================================================
// 2. Reduce creation and field access (8 tests)
// ===========================================================================

#[test]
fn reduce_zero_rule_id() {
    let a = Action::Reduce(RuleId(0));
    match a {
        Action::Reduce(r) => assert_eq!(r.0, 0),
        _ => panic!("expected Reduce"),
    }
}

#[test]
fn reduce_nonzero_rule_id() {
    let a = Action::Reduce(RuleId(5));
    match a {
        Action::Reduce(r) => assert_eq!(r, RuleId(5)),
        _ => panic!("expected Reduce"),
    }
}

#[test]
fn reduce_max_u16_rule_id() {
    let a = Action::Reduce(RuleId(u16::MAX));
    match a {
        Action::Reduce(r) => assert_eq!(r.0, u16::MAX),
        _ => panic!("expected Reduce"),
    }
}

#[test]
fn reduce_inner_value_accessible() {
    let a = Action::Reduce(RuleId(33));
    if let Action::Reduce(r) = a {
        assert_eq!(r.0, 33);
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn reduce_copies_rule_id() {
    let rid = RuleId(10);
    let a = Action::Reduce(rid);
    // rid is Copy — use it again without .clone()
    assert_eq!(rid, RuleId(10));
    assert_eq!(a, Action::Reduce(RuleId(10)));
}

#[test]
fn reduce_preserves_rule_id_across_clone() {
    let a = Action::Reduce(RuleId(200));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn reduce_two_different_rules_not_equal() {
    let a = Action::Reduce(RuleId(1));
    let b = Action::Reduce(RuleId(2));
    assert_ne!(a, b);
}

#[test]
fn reduce_same_rule_equal() {
    let a = Action::Reduce(RuleId(15));
    let b = Action::Reduce(RuleId(15));
    assert_eq!(a, b);
}

// ===========================================================================
// 3. Accept creation (8 tests)
// ===========================================================================

#[test]
fn accept_creation() {
    let a = Action::Accept;
    assert_eq!(a, Action::Accept);
}

#[test]
fn accept_matches_itself() {
    assert!(matches!(Action::Accept, Action::Accept));
}

#[test]
fn accept_not_shift() {
    assert!(!matches!(Action::Accept, Action::Shift(_)));
}

#[test]
fn accept_not_reduce() {
    assert!(!matches!(Action::Accept, Action::Reduce(_)));
}

#[test]
fn accept_not_fork() {
    assert!(!matches!(Action::Accept, Action::Fork(_)));
}

#[test]
fn accept_not_error() {
    assert!(!matches!(Action::Accept, Action::Error));
}

#[test]
fn accept_clone_equals_original() {
    let a = Action::Accept;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn accept_not_recover() {
    assert!(!matches!(Action::Accept, Action::Recover));
}

// ===========================================================================
// 4. Fork creation with multiple actions (8 tests)
// ===========================================================================

#[test]
fn fork_with_shift_and_reduce() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner.len(), 2);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_inner_actions_accessible() {
    let a = Action::Fork(vec![Action::Shift(StateId(10)), Action::Accept]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner[0], Action::Shift(StateId(10)));
        assert_eq!(inner[1], Action::Accept);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_with_three_actions() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner.len(), 3);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_with_duplicate_shifts() {
    let a = Action::Fork(vec![Action::Shift(StateId(5)), Action::Shift(StateId(5))]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner[0], inner[1]);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_single_action() {
    let a = Action::Fork(vec![Action::Accept]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner.len(), 1);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_equality_same_contents() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    assert_eq!(a, b);
}

#[test]
fn fork_inequality_different_order() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = Action::Fork(vec![Action::Reduce(RuleId(2)), Action::Shift(StateId(1))]);
    assert_ne!(a, b);
}

#[test]
fn fork_inequality_different_length() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let b = Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]);
    assert_ne!(a, b);
}

// ===========================================================================
// 5. Action equality and comparison (8 tests)
// ===========================================================================

#[test]
fn shift_not_equal_to_reduce() {
    let a = Action::Shift(StateId(1));
    let b = Action::Reduce(RuleId(1));
    assert_ne!(a, b);
}

#[test]
fn shift_not_equal_to_accept() {
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

#[test]
fn reduce_not_equal_to_accept() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Accept);
}

#[test]
fn error_not_equal_to_recover() {
    assert_ne!(Action::Error, Action::Recover);
}

#[test]
fn accept_equals_accept() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn error_equals_error() {
    assert_eq!(Action::Error, Action::Error);
}

#[test]
fn recover_equals_recover() {
    assert_eq!(Action::Recover, Action::Recover);
}

#[test]
fn fork_not_equal_to_shift() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1))]);
    let shift = Action::Shift(StateId(1));
    assert_ne!(fork, shift);
}

// ===========================================================================
// 6. Action Debug/Display formatting (8 tests)
// ===========================================================================

#[test]
fn debug_shift_contains_state_value() {
    let a = Action::Shift(StateId(42));
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Shift"), "expected 'Shift' in: {dbg}");
    assert!(dbg.contains("42"), "expected '42' in: {dbg}");
}

#[test]
fn debug_reduce_contains_rule_value() {
    let a = Action::Reduce(RuleId(7));
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Reduce"), "expected 'Reduce' in: {dbg}");
    assert!(dbg.contains("7"), "expected '7' in: {dbg}");
}

#[test]
fn debug_accept_format() {
    let dbg = format!("{:?}", Action::Accept);
    assert!(dbg.contains("Accept"), "expected 'Accept' in: {dbg}");
}

#[test]
fn debug_error_format() {
    let dbg = format!("{:?}", Action::Error);
    assert!(dbg.contains("Error"), "expected 'Error' in: {dbg}");
}

#[test]
fn debug_recover_format() {
    let dbg = format!("{:?}", Action::Recover);
    assert!(dbg.contains("Recover"), "expected 'Recover' in: {dbg}");
}

#[test]
fn debug_fork_contains_inner_actions() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]);
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Fork"), "expected 'Fork' in: {dbg}");
    assert!(dbg.contains("Shift"), "expected 'Shift' in: {dbg}");
    assert!(dbg.contains("Accept"), "expected 'Accept' in: {dbg}");
}

#[test]
fn debug_shift_max_state() {
    let a = Action::Shift(StateId(u16::MAX));
    let dbg = format!("{a:?}");
    assert!(
        dbg.contains(&u16::MAX.to_string()),
        "expected '{max}' in: {dbg}",
        max = u16::MAX
    );
}

#[test]
fn debug_reduce_max_rule() {
    let a = Action::Reduce(RuleId(u16::MAX));
    let dbg = format!("{a:?}");
    assert!(
        dbg.contains(&u16::MAX.to_string()),
        "expected '{max}' in: {dbg}",
        max = u16::MAX
    );
}

// ===========================================================================
// 7. Action classification (is_shift, is_reduce, is_accept, is_fork) (8 tests)
// ===========================================================================

#[test]
fn classify_shift() {
    let a = Action::Shift(StateId(1));
    assert!(matches!(a, Action::Shift(_)));
    assert!(!matches!(a, Action::Reduce(_)));
    assert!(!matches!(a, Action::Accept));
    assert!(!matches!(a, Action::Fork(_)));
}

#[test]
fn classify_reduce() {
    let a = Action::Reduce(RuleId(1));
    assert!(matches!(a, Action::Reduce(_)));
    assert!(!matches!(a, Action::Shift(_)));
    assert!(!matches!(a, Action::Accept));
    assert!(!matches!(a, Action::Fork(_)));
}

#[test]
fn classify_accept() {
    let a = Action::Accept;
    assert!(matches!(a, Action::Accept));
    assert!(!matches!(a, Action::Shift(_)));
    assert!(!matches!(a, Action::Reduce(_)));
    assert!(!matches!(a, Action::Fork(_)));
}

#[test]
fn classify_fork() {
    let a = Action::Fork(vec![Action::Accept]);
    assert!(matches!(a, Action::Fork(_)));
    assert!(!matches!(a, Action::Shift(_)));
    assert!(!matches!(a, Action::Reduce(_)));
    assert!(!matches!(a, Action::Accept));
}

#[test]
fn classify_error() {
    let a = Action::Error;
    assert!(matches!(a, Action::Error));
    assert!(!matches!(a, Action::Accept));
    assert!(!matches!(a, Action::Shift(_)));
}

#[test]
fn classify_recover() {
    let a = Action::Recover;
    assert!(matches!(a, Action::Recover));
    assert!(!matches!(a, Action::Error));
    assert!(!matches!(a, Action::Accept));
}

#[test]
fn classify_all_variants_in_vec() {
    let actions: Vec<Action> = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![Action::Accept]),
    ];
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, Action::Shift(_)))
            .count(),
        1
    );
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, Action::Reduce(_)))
            .count(),
        1
    );
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, Action::Accept))
            .count(),
        1
    );
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, Action::Error))
            .count(),
        1
    );
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, Action::Recover))
            .count(),
        1
    );
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, Action::Fork(_)))
            .count(),
        1
    );
}

#[test]
fn classify_fork_inner_actions() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(3)),
        Action::Reduce(RuleId(4)),
        Action::Accept,
    ]);
    if let Action::Fork(ref inner) = a {
        assert!(matches!(inner[0], Action::Shift(_)));
        assert!(matches!(inner[1], Action::Reduce(_)));
        assert!(matches!(inner[2], Action::Accept));
    } else {
        panic!("expected Fork");
    }
}

// ===========================================================================
// 8. Edge cases: empty fork, nested fork, large state IDs (8 tests)
// ===========================================================================

#[test]
fn empty_fork() {
    let a = Action::Fork(vec![]);
    if let Action::Fork(ref inner) = a {
        assert!(inner.is_empty());
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn nested_fork() {
    let inner_fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let outer = Action::Fork(vec![inner_fork, Action::Accept]);
    if let Action::Fork(ref outer_inner) = outer {
        assert_eq!(outer_inner.len(), 2);
        assert!(matches!(outer_inner[0], Action::Fork(_)));
        assert!(matches!(outer_inner[1], Action::Accept));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn deeply_nested_fork() {
    let level3 = Action::Fork(vec![Action::Accept]);
    let level2 = Action::Fork(vec![level3]);
    let level1 = Action::Fork(vec![level2]);
    // Verify we can reach the innermost Accept
    if let Action::Fork(ref l1) = level1
        && let Action::Fork(ref l2) = l1[0]
        && let Action::Fork(ref l3) = l2[0]
    {
        assert!(matches!(l3[0], Action::Accept));
        return;
    }
    panic!("failed to traverse nested forks");
}

#[test]
fn large_state_id_boundary() {
    let a = Action::Shift(StateId(u16::MAX - 1));
    if let Action::Shift(s) = a {
        assert_eq!(s.0, u16::MAX - 1);
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn large_rule_id_boundary() {
    let a = Action::Reduce(RuleId(u16::MAX - 1));
    if let Action::Reduce(r) = a {
        assert_eq!(r.0, u16::MAX - 1);
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn fork_with_many_actions() {
    let actions: Vec<Action> = (0..100).map(|i| Action::Shift(StateId(i))).collect();
    let a = Action::Fork(actions);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner.len(), 100);
        assert_eq!(inner[0], Action::Shift(StateId(0)));
        assert_eq!(inner[99], Action::Shift(StateId(99)));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn hash_consistency_for_equal_actions() {
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn compute_hash(action: &Action) -> u64 {
        let mut hasher = DefaultHasher::new();
        action.hash(&mut hasher);
        hasher.finish()
    }

    let a = Action::Shift(StateId(42));
    let b = Action::Shift(StateId(42));
    assert_eq!(compute_hash(&a), compute_hash(&b));
}

#[test]
fn action_cell_with_mixed_variants() {
    let cell: Vec<Action> = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(1)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![Action::Shift(StateId(2))]),
    ];
    assert_eq!(cell.len(), 6);
    assert!(matches!(cell[0], Action::Shift(_)));
    assert!(matches!(cell[5], Action::Fork(_)));
}
