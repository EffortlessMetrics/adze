//! Comprehensive property tests for ActionCell and Action enum.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test action_cell_properties_comprehensive --features test-api

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};

// ============================================================================
// 1. Action::Shift construction and Debug
// ============================================================================

#[test]
fn shift_zero_state() {
    let a = Action::Shift(StateId(0));
    assert!(matches!(a, Action::Shift(StateId(0))));
}

#[test]
fn shift_max_state() {
    let a = Action::Shift(StateId(u16::MAX));
    assert!(matches!(a, Action::Shift(StateId(65535))));
}

#[test]
fn shift_debug_contains_state_id() {
    let a = Action::Shift(StateId(42));
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Shift"), "debug should contain Shift: {dbg}");
    assert!(
        dbg.contains("42"),
        "debug should contain state value: {dbg}"
    );
}

#[test]
fn shift_debug_format_roundtrip_readable() {
    let a = Action::Shift(StateId(7));
    let dbg = format!("{a:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn shift_different_states_not_equal() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
}

// ============================================================================
// 2. Action::Reduce construction and Debug
// ============================================================================

#[test]
fn reduce_zero_rule() {
    let a = Action::Reduce(RuleId(0));
    assert!(matches!(a, Action::Reduce(RuleId(0))));
}

#[test]
fn reduce_max_rule() {
    let a = Action::Reduce(RuleId(u16::MAX));
    assert!(matches!(a, Action::Reduce(RuleId(65535))));
}

#[test]
fn reduce_debug_contains_rule_id() {
    let a = Action::Reduce(RuleId(99));
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Reduce"), "debug should contain Reduce: {dbg}");
    assert!(dbg.contains("99"), "debug should contain rule value: {dbg}");
}

#[test]
fn reduce_different_rules_not_equal() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
}

#[test]
fn reduce_same_rule_equal() {
    assert_eq!(Action::Reduce(RuleId(10)), Action::Reduce(RuleId(10)));
}

// ============================================================================
// 3. Action::Accept and Action::Error
// ============================================================================

#[test]
fn accept_is_accept() {
    assert!(matches!(Action::Accept, Action::Accept));
}

#[test]
fn error_is_error() {
    assert!(matches!(Action::Error, Action::Error));
}

#[test]
fn accept_not_error() {
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn accept_debug_output() {
    let dbg = format!("{:?}", Action::Accept);
    assert_eq!(dbg, "Accept");
}

#[test]
fn error_debug_output() {
    let dbg = format!("{:?}", Action::Error);
    assert_eq!(dbg, "Error");
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
fn accept_not_shift() {
    assert_ne!(Action::Accept, Action::Shift(StateId(0)));
}

#[test]
fn error_not_reduce() {
    assert_ne!(Action::Error, Action::Reduce(RuleId(0)));
}

#[test]
fn recover_is_recover() {
    assert!(matches!(Action::Recover, Action::Recover));
}

#[test]
fn recover_not_error() {
    assert_ne!(Action::Recover, Action::Error);
}

// ============================================================================
// 4. Action::Fork with multiple actions
// ============================================================================

#[test]
fn fork_with_shift_and_reduce() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    if let Action::Fork(ref actions) = a {
        assert_eq!(actions.len(), 2);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_empty_vec() {
    let a = Action::Fork(vec![]);
    if let Action::Fork(ref actions) = a {
        assert!(actions.is_empty());
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_single_action() {
    let a = Action::Fork(vec![Action::Accept]);
    if let Action::Fork(ref actions) = a {
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Accept);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_three_actions() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(10)),
        Action::Reduce(RuleId(20)),
        Action::Accept,
    ]);
    if let Action::Fork(ref actions) = a {
        assert_eq!(actions.len(), 3);
        assert!(matches!(actions[2], Action::Accept));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_nested_fork() {
    let inner = Action::Fork(vec![Action::Shift(StateId(1))]);
    let outer = Action::Fork(vec![inner.clone(), Action::Reduce(RuleId(0))]);
    if let Action::Fork(ref actions) = outer {
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], Action::Fork(_)));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_debug_contains_inner_actions() {
    let a = Action::Fork(vec![Action::Shift(StateId(5)), Action::Accept]);
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Fork"));
    assert!(dbg.contains("Shift"));
    assert!(dbg.contains("Accept"));
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
    let a = Action::Fork(vec![Action::Accept]);
    let b = Action::Fork(vec![Action::Accept, Action::Error]);
    assert_ne!(a, b);
}

// ============================================================================
// 5. Action clone
// ============================================================================

#[test]
fn clone_shift() {
    let a = Action::Shift(StateId(3));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_reduce() {
    let a = Action::Reduce(RuleId(7));
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
fn clone_fork_is_independent() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let mut b = a.clone();
    if let Action::Fork(ref mut actions) = b {
        actions.push(Action::Accept);
    }
    // a should be unchanged
    if let Action::Fork(ref actions) = a {
        assert_eq!(actions.len(), 1);
    }
}

// ============================================================================
// 6. Action in Vec (ActionCell)
// ============================================================================

type ActionCell = Vec<Action>;

#[test]
fn action_cell_empty() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
}

#[test]
fn action_cell_single_shift() {
    let cell: ActionCell = vec![Action::Shift(StateId(0))];
    assert_eq!(cell.len(), 1);
}

#[test]
fn action_cell_multiple_actions() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ];
    assert_eq!(cell.len(), 3);
}

#[test]
fn action_cell_push_and_pop() {
    let mut cell: ActionCell = vec![Action::Accept];
    cell.push(Action::Error);
    assert_eq!(cell.len(), 2);
    let popped = cell.pop().unwrap();
    assert_eq!(popped, Action::Error);
}

#[test]
fn action_cell_contains() {
    let cell: ActionCell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))];
    assert!(cell.contains(&Action::Shift(StateId(5))));
    assert!(!cell.contains(&Action::Accept));
}

#[test]
fn action_cell_iter_count() {
    let cell: ActionCell = vec![Action::Accept, Action::Error, Action::Recover];
    assert_eq!(cell.len(), 3);
}

#[test]
fn action_cell_clone() {
    let cell: ActionCell = vec![Action::Shift(StateId(1)), Action::Accept];
    let cloned = cell.clone();
    assert_eq!(cell, cloned);
}

#[test]
fn action_cell_retain_shifts() {
    let mut cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(2)),
    ];
    cell.retain(|a| matches!(a, Action::Shift(_)));
    assert_eq!(cell.len(), 2);
}

// ============================================================================
// 7. ActionCell from parse table (construction patterns)
// ============================================================================

#[test]
fn action_cell_2d_table_layout() {
    let table: Vec<Vec<ActionCell>> = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Error]],
        vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]],
    ];
    assert_eq!(table.len(), 2);
    assert_eq!(table[0].len(), 2);
    assert_eq!(table[1][1], vec![Action::Accept]);
}

#[test]
fn action_cell_table_lookup() {
    let table: Vec<Vec<ActionCell>> = vec![vec![vec![Action::Shift(StateId(1))], vec![]]];
    let cell = &table[0][0];
    assert!(matches!(cell.first(), Some(Action::Shift(StateId(1)))));
}

#[test]
fn action_cell_table_empty_cells_for_errors() {
    let table: Vec<Vec<ActionCell>> = vec![vec![vec![], vec![], vec![]]];
    for cell in &table[0] {
        assert!(cell.is_empty());
    }
}

// ============================================================================
// 8. Multiple actions in same cell (GLR conflicts)
// ============================================================================

#[test]
fn shift_reduce_conflict_cell() {
    let cell: ActionCell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn reduce_reduce_conflict_cell() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    assert_eq!(cell.len(), 2);
    let reduce_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert_eq!(reduce_count, 2);
}

#[test]
fn triple_conflict_cell() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Reduce(RuleId(3)),
    ];
    assert_eq!(cell.len(), 3);
}

#[test]
fn conflict_cell_first_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(10)), Action::Reduce(RuleId(5))];
    assert_eq!(cell[0], Action::Shift(StateId(10)));
}

// ============================================================================
// 9. Action pattern matching
// ============================================================================

#[test]
fn match_shift_extracts_state() {
    let a = Action::Shift(StateId(42));
    let state = match a {
        Action::Shift(s) => s,
        _ => panic!("expected Shift"),
    };
    assert_eq!(state, StateId(42));
}

#[test]
fn match_reduce_extracts_rule() {
    let a = Action::Reduce(RuleId(7));
    let rule = match a {
        Action::Reduce(r) => r,
        _ => panic!("expected Reduce"),
    };
    assert_eq!(rule, RuleId(7));
}

#[test]
fn match_fork_extracts_vec() {
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    let actions = match a {
        Action::Fork(v) => v,
        _ => panic!("expected Fork"),
    };
    assert_eq!(actions.len(), 2);
}

#[test]
fn match_all_variants_exhaustive() {
    let variants = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];
    for v in &variants {
        let label = match v {
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
fn if_let_shift_guard() {
    let a = Action::Shift(StateId(100));
    if let Action::Shift(s) = a {
        assert_eq!(s.0, 100);
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn matches_macro_with_guard() {
    let a = Action::Shift(StateId(50));
    assert!(matches!(a, Action::Shift(s) if s.0 >= 50));
    assert!(!matches!(a, Action::Shift(s) if s.0 > 50));
}

// ============================================================================
// 10. Action equality / comparison properties
// ============================================================================

#[test]
fn equality_is_reflexive() {
    let actions = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![Action::Accept]),
    ];
    for a in &actions {
        assert_eq!(a, a);
    }
}

#[test]
fn equality_is_symmetric() {
    let a = Action::Shift(StateId(5));
    let b = Action::Shift(StateId(5));
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn equality_is_transitive() {
    let a = Action::Reduce(RuleId(3));
    let b = Action::Reduce(RuleId(3));
    let c = Action::Reduce(RuleId(3));
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

#[test]
fn inequality_cross_variant_shift_vs_accept() {
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

#[test]
fn inequality_cross_variant_reduce_vs_error() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Error);
}

#[test]
fn inequality_cross_variant_fork_vs_recover() {
    assert_ne!(Action::Fork(vec![]), Action::Recover);
}

// ============================================================================
// 11. Hash consistency (equal values must have equal hashes)
// ============================================================================

#[test]
fn hash_consistent_with_eq_shift() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let a = Action::Shift(StateId(10));
    let b = Action::Shift(StateId(10));
    let mut ha = DefaultHasher::new();
    let mut hb = DefaultHasher::new();
    a.hash(&mut ha);
    b.hash(&mut hb);
    assert_eq!(ha.finish(), hb.finish());
}

#[test]
fn hash_consistent_with_eq_fork() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    let b = Action::Fork(vec![Action::Accept, Action::Error]);
    let mut ha = DefaultHasher::new();
    let mut hb = DefaultHasher::new();
    a.hash(&mut ha);
    b.hash(&mut hb);
    assert_eq!(ha.finish(), hb.finish());
}

#[test]
fn hash_in_hashset() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(1)));
    set.insert(Action::Shift(StateId(1))); // duplicate
    set.insert(Action::Accept);
    assert_eq!(set.len(), 2);
}

// ============================================================================
// 12. Additional edge cases and property tests
// ============================================================================

#[test]
fn action_cell_dedup() {
    let mut cell: ActionCell = vec![Action::Accept, Action::Accept, Action::Error];
    cell.dedup();
    assert_eq!(cell.len(), 2);
}

#[test]
fn action_cell_sort_by_discriminant() {
    let mut cell: ActionCell = vec![Action::Accept, Action::Shift(StateId(0)), Action::Error];
    cell.sort_by_key(|a| format!("{:?}", std::mem::discriminant(a)));
    // Just verify it doesn't panic and preserves length
    assert_eq!(cell.len(), 3);
}

#[test]
fn fork_with_all_variant_types() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
        Action::Error,
        Action::Recover,
    ]);
    if let Action::Fork(ref actions) = a {
        assert_eq!(actions.len(), 5);
    }
}

#[test]
fn action_cell_filter_map_shifts_to_states() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(3)),
    ];
    let states: Vec<StateId> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert_eq!(states, vec![StateId(1), StateId(3)]);
}

#[test]
fn action_cell_filter_map_reduces_to_rules() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(5)),
        Action::Reduce(RuleId(10)),
    ];
    let rules: Vec<RuleId> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(rules, vec![RuleId(5), RuleId(10)]);
}

#[test]
fn action_cell_partition_by_type() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ];
    let (shifts, rest): (ActionCell, ActionCell) = cell
        .into_iter()
        .partition(|a| matches!(a, Action::Shift(_)));
    assert_eq!(shifts.len(), 1);
    assert_eq!(rest.len(), 2);
}

#[test]
fn state_id_inner_value_accessible() {
    let s = StateId(123);
    assert_eq!(s.0, 123);
}

#[test]
fn rule_id_inner_value_accessible() {
    let r = RuleId(456);
    assert_eq!(r.0, 456);
}

#[test]
fn shift_with_boundary_state_one() {
    let a = Action::Shift(StateId(1));
    assert!(matches!(a, Action::Shift(StateId(1))));
}
