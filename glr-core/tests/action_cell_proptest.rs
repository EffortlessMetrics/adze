//! Property-based tests for `Action`, `ActionCell`, and parse table
//! action/goto operations.
//!
//! Run with: `cargo test -p adze-glr-core --test action_cell_proptest`

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use proptest::prelude::*;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a leaf `Action` (no `Fork`).
fn leaf_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..=u16::MAX).prop_map(|s| Action::Shift(StateId(s))),
        (0..=u16::MAX).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Generate an `Action` that may contain nested `Fork` (depth ≤ 2).
fn arb_action() -> impl Strategy<Value = Action> {
    leaf_action().prop_recursive(2, 16, 4, |inner| {
        prop::collection::vec(inner, 1..=6).prop_map(Action::Fork)
    })
}

/// Generate an `ActionCell` (a `Vec<Action>`).
fn arb_action_cell() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(arb_action(), 0..=8)
}

// ---------------------------------------------------------------------------
// 1. Clone roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_roundtrip(action in arb_action()) {
        let cloned = action.clone();
        prop_assert_eq!(&action, &cloned);
    }
}

// ---------------------------------------------------------------------------
// 2. Equality reflexivity
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn eq_reflexive(action in arb_action()) {
        prop_assert_eq!(&action, &action);
    }
}

// ---------------------------------------------------------------------------
// 3. Equality symmetry
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn eq_symmetric(a in arb_action(), b in arb_action()) {
        prop_assert_eq!(a == b, b == a);
    }
}

// ---------------------------------------------------------------------------
// 4. Hash consistency with Eq
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hash_consistent_with_eq(a in arb_action(), b in arb_action()) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        if a == b {
            let mut h1 = DefaultHasher::new();
            let mut h2 = DefaultHasher::new();
            a.hash(&mut h1);
            b.hash(&mut h2);
            prop_assert_eq!(h1.finish(), h2.finish());
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Debug formatting is non-empty
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_format_non_empty(action in arb_action()) {
        let dbg = format!("{:?}", action);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 6. Debug contains variant name
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_variant(action in leaf_action()) {
        let dbg = format!("{:?}", action);
        let expected = match &action {
            Action::Shift(_) => "Shift",
            Action::Reduce(_) => "Reduce",
            Action::Accept => "Accept",
            Action::Error => "Error",
            Action::Recover => "Recover",
            Action::Fork(_) => "Fork",
            _ => "Unknown",
        };
        prop_assert!(dbg.contains(expected), "Debug {:?} missing variant {}", dbg, expected);
    }
}

// ---------------------------------------------------------------------------
// 7. Shift state roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn shift_state_roundtrip(s in 0..=u16::MAX) {
        let action = Action::Shift(StateId(s));
        match action {
            Action::Shift(StateId(v)) => prop_assert_eq!(v, s),
            _ => prop_assert!(false, "Expected Shift"),
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Reduce rule roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn reduce_rule_roundtrip(r in 0..=u16::MAX) {
        let action = Action::Reduce(RuleId(r));
        match action {
            Action::Reduce(RuleId(v)) => prop_assert_eq!(v, r),
            _ => prop_assert!(false, "Expected Reduce"),
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Fork preserves children
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fork_preserves_children(actions in prop::collection::vec(leaf_action(), 1..=8)) {
        let fork = Action::Fork(actions.clone());
        match fork {
            Action::Fork(inner) => prop_assert_eq!(inner, actions),
            _ => prop_assert!(false, "Expected Fork"),
        }
    }
}

// ---------------------------------------------------------------------------
// 10. Fork children length
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fork_children_len(actions in prop::collection::vec(leaf_action(), 0..=10)) {
        let fork = Action::Fork(actions.clone());
        if let Action::Fork(inner) = fork {
            prop_assert_eq!(inner.len(), actions.len());
        }
    }
}

// ---------------------------------------------------------------------------
// 11. ActionCell dedup idempotence
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_dedup_idempotent(cell in arb_action_cell()) {
        let mut once = cell.clone();
        once.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        once.dedup();

        let mut twice = once.clone();
        twice.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        twice.dedup();

        prop_assert_eq!(once, twice);
    }
}

// ---------------------------------------------------------------------------
// 12. ActionCell dedup never increases length
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_dedup_shrinks_or_same(cell in arb_action_cell()) {
        let original_len = cell.len();
        let mut deduped = cell;
        deduped.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        deduped.dedup();
        prop_assert!(deduped.len() <= original_len);
    }
}

// ---------------------------------------------------------------------------
// 13. ActionCell dedup preserves unique elements
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_dedup_preserves_uniques(cell in arb_action_cell()) {
        let unique_set: HashSet<_> = cell.iter().cloned().collect();
        let mut deduped = cell;
        deduped.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        deduped.dedup();
        let deduped_set: HashSet<_> = deduped.into_iter().collect();
        prop_assert_eq!(unique_set, deduped_set);
    }
}

// ---------------------------------------------------------------------------
// 14. Double-clone identity
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn double_clone_identity(cell in arb_action_cell()) {
        let c1 = cell.clone();
        let c2 = c1.clone();
        prop_assert_eq!(c1, c2);
    }
}

// ---------------------------------------------------------------------------
// 15. Fork flattening: inner Fork contents are accessible
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fork_flatten_collects_all_leaves(
        a in leaf_action(),
        b in leaf_action(),
        c in leaf_action(),
    ) {
        let inner_fork = Action::Fork(vec![b.clone(), c.clone()]);
        let outer_fork = Action::Fork(vec![a.clone(), inner_fork]);

        fn collect_leaves(action: &Action) -> Vec<Action> {
            match action {
                Action::Fork(children) => children.iter().flat_map(collect_leaves).collect(),
                other => vec![other.clone()],
            }
        }

        let leaves = collect_leaves(&outer_fork);
        prop_assert!(leaves.contains(&a));
        prop_assert!(leaves.contains(&b));
        prop_assert!(leaves.contains(&c));
        prop_assert_eq!(leaves.len(), 3);
    }
}

// ---------------------------------------------------------------------------
// 16. Leaf action flatten is identity
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn leaf_flatten_identity(action in leaf_action()) {
        fn collect_leaves(action: &Action) -> Vec<Action> {
            match action {
                Action::Fork(children) => children.iter().flat_map(collect_leaves).collect(),
                other => vec![other.clone()],
            }
        }
        let leaves = collect_leaves(&action);
        prop_assert_eq!(leaves.len(), 1);
        prop_assert_eq!(&leaves[0], &action);
    }
}

// ---------------------------------------------------------------------------
// 17. Different variants are not equal
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn shift_ne_reduce(s in 0u16..1000, r in 0u16..1000) {
        let shift = Action::Shift(StateId(s));
        let reduce = Action::Reduce(RuleId(r));
        prop_assert_ne!(shift, reduce);
    }
}

// ---------------------------------------------------------------------------
// 18. Accept, Error, Recover are distinct singletons
// ---------------------------------------------------------------------------

#[test]
fn singleton_variants_distinct() {
    let singletons = [Action::Accept, Action::Error, Action::Recover];
    for i in 0..singletons.len() {
        for j in 0..singletons.len() {
            if i == j {
                assert_eq!(singletons[i], singletons[j]);
            } else {
                assert_ne!(singletons[i], singletons[j]);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 19. Shift ordering by state id
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn shift_different_states_ne(s1 in 0u16..10000, s2 in 0u16..10000) {
        let a = Action::Shift(StateId(s1));
        let b = Action::Shift(StateId(s2));
        prop_assert_eq!(a == b, s1 == s2);
    }
}

// ---------------------------------------------------------------------------
// 20. Reduce ordering by rule id
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn reduce_different_rules_ne(r1 in 0u16..10000, r2 in 0u16..10000) {
        let a = Action::Reduce(RuleId(r1));
        let b = Action::Reduce(RuleId(r2));
        prop_assert_eq!(a == b, r1 == r2);
    }
}

// ---------------------------------------------------------------------------
// 21. Fork equality depends on children order
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fork_eq_order_sensitive(a in leaf_action(), b in leaf_action()) {
        let f1 = Action::Fork(vec![a.clone(), b.clone()]);
        let f2 = Action::Fork(vec![b.clone(), a.clone()]);
        // Equal only if both children are the same action
        if a == b {
            prop_assert_eq!(f1, f2);
        } else {
            prop_assert_ne!(f1, f2);
        }
    }
}

// ---------------------------------------------------------------------------
// 22. Empty fork is valid
// ---------------------------------------------------------------------------

#[test]
fn empty_fork_is_valid() {
    let fork = Action::Fork(vec![]);
    let cloned = fork.clone();
    assert_eq!(fork, cloned);
    if let Action::Fork(inner) = &fork {
        assert!(inner.is_empty());
    } else {
        panic!("Expected Fork");
    }
}

// ---------------------------------------------------------------------------
// 23. ActionCell with duplicates: dedup removes exact duplicates
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_with_explicit_duplicates(action in leaf_action()) {
        let mut cell = vec![action.clone(), action.clone(), action.clone()];
        cell.dedup();
        prop_assert_eq!(cell.len(), 1);
        prop_assert_eq!(&cell[0], &action);
    }
}

// ---------------------------------------------------------------------------
// 24. Serde JSON roundtrip for Action
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_json_roundtrip(action in arb_action()) {
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&action, &deserialized);
    }
}

// ---------------------------------------------------------------------------
// 25. Serde JSON roundtrip for ActionCell
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_json_roundtrip_cell(cell in arb_action_cell()) {
        let json = serde_json::to_string(&cell).unwrap();
        let deserialized: Vec<Action> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cell, &deserialized);
    }
}

// ---------------------------------------------------------------------------
// 26. HashSet insertion de-duplicates actions
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hashset_deduplicates(actions in prop::collection::vec(arb_action(), 0..=12)) {
        let set: HashSet<_> = actions.iter().cloned().collect();
        // Every action in the set must appear in the original
        for a in &set {
            prop_assert!(actions.contains(a));
        }
        // Set size ≤ original
        prop_assert!(set.len() <= actions.len());
    }
}

// ---------------------------------------------------------------------------
// 27. ActionCell contains check after push
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_push_then_contains(cell in arb_action_cell(), new in arb_action()) {
        let mut cell = cell;
        cell.push(new.clone());
        prop_assert!(cell.contains(&new));
    }
}

// ---------------------------------------------------------------------------
// 28. Fork nested depth invariant
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn nested_fork_depth_bounded(action in arb_action()) {
        fn max_depth(action: &Action) -> usize {
            match action {
                Action::Fork(children) => {
                    1 + children.iter().map(max_depth).max().unwrap_or(0)
                }
                _ => 0,
            }
        }
        // Our strategy caps recursion at depth 2
        prop_assert!(max_depth(&action) <= 3);
    }
}
