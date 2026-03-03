#![allow(
    clippy::needless_range_loop,
    clippy::vec_init_then_push,
    clippy::useless_vec
)]

use adze_glr_core::{Action, ActionCell, RuleId, StateId};
use std::collections::{HashMap, HashSet};

// ── 1. Action construction (all variants) ──────────────────────────────────

#[test]
fn construct_shift() {
    let a = Action::Shift(StateId(0));
    assert_eq!(a, Action::Shift(StateId(0)));
}

#[test]
fn construct_shift_large_state() {
    let a = Action::Shift(StateId(u16::MAX));
    assert_eq!(a, Action::Shift(StateId(u16::MAX)));
}

#[test]
fn construct_reduce() {
    let a = Action::Reduce(RuleId(3));
    assert_eq!(a, Action::Reduce(RuleId(3)));
}

#[test]
fn construct_reduce_zero() {
    let a = Action::Reduce(RuleId(0));
    assert_eq!(a, Action::Reduce(RuleId(0)));
}

#[test]
fn construct_accept() {
    let a = Action::Accept;
    assert_eq!(a, Action::Accept);
}

#[test]
fn construct_error() {
    let a = Action::Error;
    assert_eq!(a, Action::Error);
}

#[test]
fn construct_recover() {
    let a = Action::Recover;
    assert_eq!(a, Action::Recover);
}

#[test]
fn construct_fork_two_actions() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    match &a {
        Action::Fork(inner) => assert_eq!(inner.len(), 2),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn construct_fork_empty() {
    let a = Action::Fork(vec![]);
    assert_eq!(a, Action::Fork(vec![]));
}

// ── 2. Action equality and comparison ──────────────────────────────────────

#[test]
fn equality_same_shift() {
    assert_eq!(Action::Shift(StateId(5)), Action::Shift(StateId(5)));
}

#[test]
fn inequality_different_shift_states() {
    assert_ne!(Action::Shift(StateId(1)), Action::Shift(StateId(2)));
}

#[test]
fn inequality_different_variants() {
    assert_ne!(Action::Accept, Action::Error);
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
    assert_ne!(Action::Error, Action::Recover);
}

#[test]
fn equality_accept_is_reflexive() {
    let a = Action::Accept;
    let b = Action::Accept;
    assert_eq!(a, b);
}

#[test]
fn equality_fork_order_matters() {
    let f1 = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let f2 = Action::Fork(vec![Action::Reduce(RuleId(2)), Action::Shift(StateId(1))]);
    // Vec equality is order-dependent
    assert_ne!(f1, f2);
}

#[test]
fn equality_fork_same_order() {
    let f1 = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let f2 = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    assert_eq!(f1, f2);
}

// ── 3. ActionCell construction and manipulation ────────────────────────────

#[test]
fn action_cell_empty() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
}

#[test]
fn action_cell_single_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(0))];
    assert_eq!(cell.len(), 1);
}

#[test]
fn action_cell_push() {
    let cell: ActionCell = vec![Action::Accept, Action::Error];
    assert_eq!(cell.len(), 2);
}

#[test]
fn action_cell_contains() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(3)),
        Action::Reduce(RuleId(1)),
        Action::Accept,
    ];
    assert!(cell.contains(&Action::Accept));
    assert!(!cell.contains(&Action::Error));
}

#[test]
fn action_cell_iter_filter() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(5)),
        Action::Accept,
    ];
    let shifts: Vec<_> = cell
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .collect();
    assert_eq!(shifts.len(), 2);
}

#[test]
fn action_cell_retain() {
    let mut cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Error,
        Action::Reduce(RuleId(2)),
        Action::Error,
    ];
    cell.retain(|a| !matches!(a, Action::Error));
    assert_eq!(cell.len(), 2);
    assert!(!cell.contains(&Action::Error));
}

// ── 4. Multi-action cells (GLR behavior) ───────────────────────────────────

#[test]
fn multi_action_shift_reduce_conflict() {
    let cell: ActionCell = vec![Action::Shift(StateId(4)), Action::Reduce(RuleId(1))];
    let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
    let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
    assert!(
        has_shift && has_reduce,
        "cell represents shift/reduce conflict"
    );
}

#[test]
fn multi_action_reduce_reduce_conflict() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    let reduces: Vec<_> = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .collect();
    assert_eq!(reduces.len(), 2);
}

#[test]
fn multi_action_cell_dedup() {
    let mut cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ];
    cell.dedup();
    assert_eq!(cell.len(), 2);
}

#[test]
fn fork_inside_action_cell() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(3))]);
    let cell: ActionCell = vec![fork.clone()];
    assert_eq!(cell.len(), 1);
    assert_eq!(cell[0], fork);
}

// ── 5. Action type checking / classification ───────────────────────────────

#[test]
fn classify_with_matches_macro() {
    let actions = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];
    assert!(matches!(actions[0], Action::Shift(_)));
    assert!(matches!(actions[1], Action::Reduce(_)));
    assert!(matches!(actions[2], Action::Accept));
    assert!(matches!(actions[3], Action::Error));
    assert!(matches!(actions[4], Action::Recover));
    assert!(matches!(actions[5], Action::Fork(_)));
}

#[test]
fn extract_shift_state() {
    let a = Action::Shift(StateId(42));
    if let Action::Shift(s) = a {
        assert_eq!(s, StateId(42));
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn extract_reduce_rule() {
    let a = Action::Reduce(RuleId(7));
    if let Action::Reduce(r) = a {
        assert_eq!(r, RuleId(7));
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn extract_fork_contents() {
    let inner = vec![Action::Accept, Action::Recover];
    let a = Action::Fork(inner.clone());
    if let Action::Fork(v) = a {
        assert_eq!(v, inner);
    } else {
        panic!("expected Fork");
    }
}

// ── 6. Clone / Debug / Hash traits ─────────────────────────────────────────

#[test]
fn clone_preserves_equality() {
    let actions = vec![
        Action::Shift(StateId(10)),
        Action::Reduce(RuleId(5)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![Action::Accept]),
    ];
    for a in &actions {
        assert_eq!(a, &a.clone());
    }
}

#[test]
fn debug_output_contains_variant_name() {
    let shift_dbg = format!("{:?}", Action::Shift(StateId(1)));
    assert!(shift_dbg.contains("Shift"));

    let reduce_dbg = format!("{:?}", Action::Reduce(RuleId(2)));
    assert!(reduce_dbg.contains("Reduce"));

    let accept_dbg = format!("{:?}", Action::Accept);
    assert!(accept_dbg.contains("Accept"));

    let error_dbg = format!("{:?}", Action::Error);
    assert!(error_dbg.contains("Error"));

    let recover_dbg = format!("{:?}", Action::Recover);
    assert!(recover_dbg.contains("Recover"));

    let fork_dbg = format!("{:?}", Action::Fork(vec![]));
    assert!(fork_dbg.contains("Fork"));
}

#[test]
fn hash_equal_actions_same_bucket() {
    let mut set = HashSet::new();
    set.insert(Action::Accept);
    set.insert(Action::Accept);
    assert_eq!(set.len(), 1);
}

#[test]
fn hash_different_actions_different_buckets() {
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(0)));
    set.insert(Action::Reduce(RuleId(0)));
    set.insert(Action::Accept);
    set.insert(Action::Error);
    set.insert(Action::Recover);
    assert_eq!(set.len(), 5);
}

#[test]
fn hashmap_keyed_by_action() {
    let mut map = HashMap::new();
    map.insert(Action::Shift(StateId(1)), "go to state 1");
    map.insert(Action::Accept, "done");
    assert_eq!(map.get(&Action::Shift(StateId(1))), Some(&"go to state 1"));
    assert_eq!(map.get(&Action::Accept), Some(&"done"));
    assert_eq!(map.get(&Action::Error), None);
}

// ── 7. Collection behavior ─────────────────────────────────────────────────

#[test]
fn sort_action_cells_deterministic() {
    let mut cell: ActionCell = vec![
        Action::Reduce(RuleId(2)),
        Action::Shift(StateId(1)),
        Action::Accept,
    ];
    // Sort by discriminant ordering so results are deterministic
    cell.sort_by_key(|a| match a {
        Action::Shift(s) => (0u8, s.0, 0u16),
        Action::Reduce(r) => (1, r.0, 0),
        Action::Accept => (2, 0, 0),
        Action::Error => (3, 0, 0),
        Action::Recover => (4, 0, 0),
        Action::Fork(_) => (5, 0, 0),
        _ => (255, 0, 0),
    });
    assert!(matches!(cell[0], Action::Shift(_)));
    assert!(matches!(cell[1], Action::Reduce(_)));
    assert!(matches!(cell[2], Action::Accept));
}

#[test]
fn vec_of_action_cells_as_table_row() {
    let row: Vec<ActionCell> = vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(0))],
        vec![],
        vec![Action::Accept],
    ];
    assert_eq!(row.len(), 4);
    assert!(row[2].is_empty());
}

#[test]
fn nested_fork_in_collection() {
    let nested = Action::Fork(vec![
        Action::Fork(vec![Action::Shift(StateId(1))]),
        Action::Reduce(RuleId(0)),
    ]);
    let cell: ActionCell = vec![nested.clone()];
    if let Action::Fork(outer) = &cell[0] {
        assert!(matches!(outer[0], Action::Fork(_)));
        assert!(matches!(outer[1], Action::Reduce(_)));
    } else {
        panic!("expected Fork");
    }
}

// ── 8. Edge cases ──────────────────────────────────────────────────────────

#[test]
fn action_cell_all_same_variant() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(0)),
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
    ];
    assert!(cell.iter().all(|a| matches!(a, Action::Shift(_))));
}

#[test]
fn fork_with_single_action_is_valid() {
    let a = Action::Fork(vec![Action::Accept]);
    if let Action::Fork(v) = &a {
        assert_eq!(v.len(), 1);
    }
}

#[test]
fn large_action_cell() {
    let cell: ActionCell = (0..1000).map(|i| Action::Shift(StateId(i))).collect();
    assert_eq!(cell.len(), 1000);
    assert_eq!(cell[999], Action::Shift(StateId(999)));
}

#[test]
fn clone_fork_is_deep() {
    let original = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
    // Mutating the clone's inner vec (via reconstruction) doesn't affect original
    if let Action::Fork(mut v) = cloned {
        v.push(Action::Accept);
        assert_eq!(v.len(), 3);
    }
    // Original still has 2
    if let Action::Fork(v) = &original {
        assert_eq!(v.len(), 2);
    }
}

// ── 9. Serialization (serde) ───────────────────────────────────────────────

#[test]
fn serde_json_roundtrip_shift() {
    let a = Action::Shift(StateId(42));
    let json = serde_json::to_string(&a).unwrap();
    let back: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, back);
}

#[test]
fn serde_json_roundtrip_all_variants() {
    let actions = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(99)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]),
    ];
    let json = serde_json::to_string(&actions).unwrap();
    let back: Vec<Action> = serde_json::from_str(&json).unwrap();
    assert_eq!(actions, back);
}

#[test]
fn serde_json_roundtrip_action_cell() {
    let cell: ActionCell = vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(1))];
    let json = serde_json::to_string(&cell).unwrap();
    let back: ActionCell = serde_json::from_str(&json).unwrap();
    assert_eq!(cell, back);
}

// ── 10. Pattern matching ───────────────────────────────────────────────────

#[test]
fn match_exhaustive_classification() {
    let actions = vec![
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
        assert_ne!(label, "unknown", "all known variants should be classified");
    }
}

#[test]
fn match_guard_on_shift_state() {
    let a = Action::Shift(StateId(10));
    let is_high_state = matches!(a, Action::Shift(s) if s.0 > 5);
    assert!(is_high_state);
    let is_low_state = matches!(a, Action::Shift(s) if s.0 <= 5);
    assert!(!is_low_state);
}

#[test]
fn match_guard_on_reduce_rule() {
    let a = Action::Reduce(RuleId(0));
    let is_start_rule = matches!(a, Action::Reduce(r) if r.0 == 0);
    assert!(is_start_rule);
}

#[test]
fn partition_cell_by_variant() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(2)),
        Action::Accept,
        Action::Reduce(RuleId(3)),
        Action::Error,
    ];
    let (shifts, rest): (Vec<_>, Vec<_>) = cell
        .into_iter()
        .partition(|a| matches!(a, Action::Shift(_)));
    assert_eq!(shifts.len(), 2);
    assert_eq!(rest.len(), 4);
}

#[test]
fn find_first_accept_in_cell() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(0)),
        Action::Error,
        Action::Accept,
        Action::Reduce(RuleId(1)),
    ];
    let found = cell.iter().find(|a| matches!(a, Action::Accept));
    assert!(found.is_some());
    assert_eq!(found.unwrap(), &Action::Accept);
}
