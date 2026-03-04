//! Comprehensive tests for the `Action` enum and `ParseRule` struct.

use adze_glr_core::{Action, ParseRule, RuleId, StateId, SymbolId};

// ── 1. Action::Shift construction ──────────────────────────────────────────

#[test]
fn shift_zero_state() {
    let a = Action::Shift(StateId(0));
    assert_eq!(a, Action::Shift(StateId(0)));
}

#[test]
fn shift_nonzero_state() {
    let a = Action::Shift(StateId(42));
    assert_eq!(a, Action::Shift(StateId(42)));
}

#[test]
fn shift_max_state() {
    let a = Action::Shift(StateId(u16::MAX));
    assert_eq!(a, Action::Shift(StateId(u16::MAX)));
}

#[test]
fn shift_one() {
    let a = Action::Shift(StateId(1));
    assert_eq!(a, Action::Shift(StateId(1)));
}

#[test]
fn shift_boundary_255() {
    let a = Action::Shift(StateId(255));
    assert_eq!(a, Action::Shift(StateId(255)));
}

#[test]
fn shift_boundary_256() {
    let a = Action::Shift(StateId(256));
    assert_eq!(a, Action::Shift(StateId(256)));
}

// ── 2. Action::Reduce construction ─────────────────────────────────────────

#[test]
fn reduce_zero_rule() {
    let a = Action::Reduce(RuleId(0));
    assert_eq!(a, Action::Reduce(RuleId(0)));
}

#[test]
fn reduce_nonzero_rule() {
    let a = Action::Reduce(RuleId(7));
    assert_eq!(a, Action::Reduce(RuleId(7)));
}

#[test]
fn reduce_max_rule() {
    let a = Action::Reduce(RuleId(u16::MAX));
    assert_eq!(a, Action::Reduce(RuleId(u16::MAX)));
}

#[test]
fn reduce_large_rule() {
    let a = Action::Reduce(RuleId(1000));
    assert_eq!(a, Action::Reduce(RuleId(1000)));
}

// ── 3. Action::Accept construction ─────────────────────────────────────────

#[test]
fn accept_construction() {
    let a = Action::Accept;
    assert_eq!(a, Action::Accept);
}

#[test]
fn accept_is_accept() {
    assert!(matches!(Action::Accept, Action::Accept));
}

// ── 4. Action::Error construction ──────────────────────────────────────────

#[test]
fn error_construction() {
    let a = Action::Error;
    assert_eq!(a, Action::Error);
}

#[test]
fn error_is_error() {
    assert!(matches!(Action::Error, Action::Error));
}

// ── 5. Action::Fork construction ───────────────────────────────────────────

#[test]
fn fork_empty() {
    let a = Action::Fork(vec![]);
    assert_eq!(a, Action::Fork(vec![]));
}

#[test]
fn fork_single_shift() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    assert_eq!(a, Action::Fork(vec![Action::Shift(StateId(1))]));
}

#[test]
fn fork_single_reduce() {
    let a = Action::Fork(vec![Action::Reduce(RuleId(0))]);
    assert_eq!(a, Action::Fork(vec![Action::Reduce(RuleId(0))]));
}

#[test]
fn fork_shift_reduce() {
    let a = Action::Fork(vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))]);
    assert!(matches!(a, Action::Fork(ref v) if v.len() == 2));
}

#[test]
fn fork_multiple_shifts() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
        Action::Shift(StateId(3)),
    ]);
    if let Action::Fork(ref v) = a {
        assert_eq!(v.len(), 3);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_nested() {
    let inner = Action::Fork(vec![Action::Shift(StateId(0))]);
    let outer = Action::Fork(vec![inner.clone(), Action::Accept]);
    if let Action::Fork(ref v) = outer {
        assert_eq!(v.len(), 2);
        assert!(matches!(v[0], Action::Fork(_)));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_with_accept_and_error() {
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    assert_eq!(a, Action::Fork(vec![Action::Accept, Action::Error]));
}

#[test]
fn fork_with_recover() {
    let a = Action::Fork(vec![Action::Recover, Action::Shift(StateId(0))]);
    if let Action::Fork(ref v) = a {
        assert_eq!(v[0], Action::Recover);
    } else {
        panic!("expected Fork");
    }
}

// ── 6. Action Clone behavior ───────────────────────────────────────────────

#[test]
fn clone_shift() {
    let a = Action::Shift(StateId(10));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_reduce() {
    let a = Action::Reduce(RuleId(5));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_accept() {
    let a = Action::Accept;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_error() {
    let a = Action::Error;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_fork() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_fork_independence() {
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
fn clone_recover() {
    let a = Action::Recover;
    assert_eq!(a.clone(), Action::Recover);
}

// ── 7. Action Debug output ─────────────────────────────────────────────────

#[test]
fn debug_shift() {
    let s = format!("{:?}", Action::Shift(StateId(7)));
    assert!(s.contains("Shift"));
    assert!(s.contains("7"));
}

#[test]
fn debug_reduce() {
    let s = format!("{:?}", Action::Reduce(RuleId(3)));
    assert!(s.contains("Reduce"));
    assert!(s.contains("3"));
}

#[test]
fn debug_accept() {
    let s = format!("{:?}", Action::Accept);
    assert_eq!(s, "Accept");
}

#[test]
fn debug_error() {
    let s = format!("{:?}", Action::Error);
    assert_eq!(s, "Error");
}

#[test]
fn debug_recover() {
    let s = format!("{:?}", Action::Recover);
    assert_eq!(s, "Recover");
}

#[test]
fn debug_fork() {
    let s = format!("{:?}", Action::Fork(vec![Action::Accept]));
    assert!(s.contains("Fork"));
    assert!(s.contains("Accept"));
}

#[test]
fn debug_fork_empty() {
    let s = format!("{:?}", Action::Fork(vec![]));
    assert!(s.contains("Fork"));
}

// ── 8. Action PartialEq ───────────────────────────────────────────────────

#[test]
fn eq_shift_same() {
    assert_eq!(Action::Shift(StateId(0)), Action::Shift(StateId(0)));
}

#[test]
fn ne_shift_different_state() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
}

#[test]
fn ne_shift_vs_reduce() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn ne_shift_vs_accept() {
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

#[test]
fn ne_shift_vs_error() {
    assert_ne!(Action::Shift(StateId(0)), Action::Error);
}

#[test]
fn ne_accept_vs_error() {
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn eq_reduce_same() {
    assert_eq!(Action::Reduce(RuleId(99)), Action::Reduce(RuleId(99)));
}

#[test]
fn ne_reduce_different_rule() {
    assert_ne!(Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2)));
}

#[test]
fn eq_fork_same_contents() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]);
    let b = Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]);
    assert_eq!(a, b);
}

#[test]
fn ne_fork_different_order() {
    let a = Action::Fork(vec![Action::Accept, Action::Shift(StateId(1))]);
    let b = Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]);
    assert_ne!(a, b);
}

#[test]
fn ne_fork_different_length() {
    let a = Action::Fork(vec![Action::Accept]);
    let b = Action::Fork(vec![Action::Accept, Action::Error]);
    assert_ne!(a, b);
}

#[test]
fn ne_fork_vs_accept() {
    assert_ne!(Action::Fork(vec![Action::Accept]), Action::Accept);
}

#[test]
fn eq_accept_accept() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn eq_error_error() {
    assert_eq!(Action::Error, Action::Error);
}

#[test]
fn ne_recover_vs_error() {
    assert_ne!(Action::Recover, Action::Error);
}

#[test]
fn eq_recover_recover() {
    assert_eq!(Action::Recover, Action::Recover);
}

// ── 9. ParseRule construction ──────────────────────────────────────────────

#[test]
fn parse_rule_basic() {
    let r = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 1,
    };
    assert_eq!(r.lhs, SymbolId(0));
    assert_eq!(r.rhs_len, 1);
}

#[test]
fn parse_rule_zero_rhs() {
    let r = ParseRule {
        lhs: SymbolId(5),
        rhs_len: 0,
    };
    assert_eq!(r.rhs_len, 0);
}

#[test]
fn parse_rule_large_lhs() {
    let r = ParseRule {
        lhs: SymbolId(u16::MAX),
        rhs_len: 3,
    };
    assert_eq!(r.lhs, SymbolId(u16::MAX));
}

#[test]
fn parse_rule_max_rhs() {
    let r = ParseRule {
        lhs: SymbolId(1),
        rhs_len: u16::MAX,
    };
    assert_eq!(r.rhs_len, u16::MAX);
}

#[test]
fn parse_rule_clone() {
    let r = ParseRule {
        lhs: SymbolId(10),
        rhs_len: 4,
    };
    let r2 = r.clone();
    assert_eq!(r.lhs, r2.lhs);
    assert_eq!(r.rhs_len, r2.rhs_len);
}

#[test]
fn parse_rule_debug() {
    let r = ParseRule {
        lhs: SymbolId(2),
        rhs_len: 3,
    };
    let s = format!("{:?}", r);
    assert!(s.contains("ParseRule"));
    assert!(s.contains("lhs"));
    assert!(s.contains("rhs_len"));
}

// ── 10. ParseRule field access ─────────────────────────────────────────────

#[test]
fn parse_rule_lhs_field() {
    let r = ParseRule {
        lhs: SymbolId(77),
        rhs_len: 2,
    };
    assert_eq!(r.lhs.0, 77);
}

#[test]
fn parse_rule_rhs_len_field() {
    let r = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 12,
    };
    assert_eq!(r.rhs_len, 12);
}

#[test]
fn parse_rule_different_instances_independent() {
    let r1 = ParseRule {
        lhs: SymbolId(1),
        rhs_len: 2,
    };
    let r2 = ParseRule {
        lhs: SymbolId(3),
        rhs_len: 4,
    };
    assert_ne!(r1.lhs, r2.lhs);
    assert_ne!(r1.rhs_len, r2.rhs_len);
}

#[test]
fn parse_rule_many_rules() {
    let rules: Vec<ParseRule> = (0..100)
        .map(|i| ParseRule {
            lhs: SymbolId(i),
            rhs_len: i % 5,
        })
        .collect();
    assert_eq!(rules.len(), 100);
    assert_eq!(rules[50].lhs, SymbolId(50));
    assert_eq!(rules[50].rhs_len, 0);
}

#[test]
fn parse_rule_lhs_zero() {
    let r = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 0,
    };
    assert_eq!(r.lhs.0, 0);
    assert_eq!(r.rhs_len, 0);
}

// ── Additional coverage ────────────────────────────────────────────────────

#[test]
fn action_in_vec() {
    let actions: Vec<Action> = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(1)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];
    assert_eq!(actions.len(), 6);
}

#[test]
fn action_in_hash_set() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Accept);
    set.insert(Action::Error);
    set.insert(Action::Accept); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn action_hash_shift_distinct() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(0)));
    set.insert(Action::Shift(StateId(1)));
    assert_eq!(set.len(), 2);
}

#[test]
fn action_hash_reduce_distinct() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Reduce(RuleId(0)));
    set.insert(Action::Reduce(RuleId(1)));
    set.insert(Action::Reduce(RuleId(0))); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn matches_shift_pattern() {
    let a = Action::Shift(StateId(99));
    assert!(matches!(a, Action::Shift(StateId(99))));
    assert!(!matches!(a, Action::Shift(StateId(100))));
}

#[test]
fn matches_reduce_pattern() {
    let a = Action::Reduce(RuleId(5));
    assert!(matches!(a, Action::Reduce(RuleId(5))));
    assert!(!matches!(a, Action::Reduce(RuleId(6))));
}

#[test]
fn fork_preserves_order() {
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
fn state_id_inner_access() {
    let s = StateId(123);
    assert_eq!(s.0, 123);
}

#[test]
fn rule_id_inner_access() {
    let r = RuleId(456);
    assert_eq!(r.0, 456);
}

#[test]
fn symbol_id_inner_access() {
    let s = SymbolId(789);
    assert_eq!(s.0, 789);
}
