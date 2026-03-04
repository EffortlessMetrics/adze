#![allow(clippy::needless_range_loop)]
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

// ---------------------------------------------------------------------------
// 29. Single-action cell is deterministic (exactly one action)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn single_action_cell_is_deterministic(action in leaf_action()) {
        let cell: Vec<Action> = vec![action.clone()];
        prop_assert_eq!(cell.len(), 1);
        prop_assert_eq!(&cell[0], &action);
        // A single-action cell has no conflicts
        let has_shift = matches!(&cell[0], Action::Shift(_));
        let has_reduce = matches!(&cell[0], Action::Reduce(_));
        prop_assert!(!(has_shift && has_reduce));
    }
}

// ---------------------------------------------------------------------------
// 30. Multi-action cell represents GLR conflict
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn multi_action_cell_has_conflict(
        s in 0u16..1000,
        r in 0u16..1000,
    ) {
        // A shift-reduce conflict cell
        let cell: Vec<Action> = vec![
            Action::Shift(StateId(s)),
            Action::Reduce(RuleId(r)),
        ];
        prop_assert!(cell.len() > 1, "GLR conflict cell must have multiple actions");
        let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
        let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
        prop_assert!(has_shift && has_reduce, "shift-reduce conflict expected");
    }
}

// ---------------------------------------------------------------------------
// 31. Empty action cell signals error state
// ---------------------------------------------------------------------------

#[test]
fn empty_action_cell_signals_error() {
    let cell: Vec<Action> = vec![];
    assert!(cell.is_empty());
    assert!(!cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(!cell.iter().any(|a| matches!(a, Action::Reduce(_))));
    assert!(!cell.iter().any(|a| matches!(a, Action::Accept)));
}

// ---------------------------------------------------------------------------
// 32. Action ordering stability: sorting twice yields same result
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn action_ordering_stable(cell in arb_action_cell()) {
        let mut sorted1 = cell.clone();
        sorted1.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));

        let mut sorted2 = sorted1.clone();
        sorted2.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));

        prop_assert_eq!(sorted1, sorted2, "sorting must be stable");
    }
}

// ---------------------------------------------------------------------------
// 33. Cell merge via extend preserves all actions
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_merge_preserves_all(
        cell_a in arb_action_cell(),
        cell_b in arb_action_cell(),
    ) {
        let mut merged = cell_a.clone();
        merged.extend(cell_b.clone());
        prop_assert_eq!(merged.len(), cell_a.len() + cell_b.len());
        for action in &cell_a {
            prop_assert!(merged.contains(action));
        }
        for action in &cell_b {
            prop_assert!(merged.contains(action));
        }
    }
}

// ---------------------------------------------------------------------------
// 34. Cell merge then dedup has no duplicates
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_merge_dedup_unique(
        cell_a in arb_action_cell(),
        cell_b in arb_action_cell(),
    ) {
        let mut merged = cell_a;
        merged.extend(cell_b);
        merged.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        merged.dedup();
        let set: HashSet<_> = merged.iter().cloned().collect();
        prop_assert_eq!(merged.len(), set.len(), "deduped cell must have unique elements");
    }
}

// ---------------------------------------------------------------------------
// 35. Accept action is unique singleton in cell
// ---------------------------------------------------------------------------

#[test]
fn accept_action_handling() {
    let cell: Vec<Action> = vec![Action::Accept];
    assert_eq!(cell.len(), 1);
    assert!(matches!(&cell[0], Action::Accept));

    // Accept with shift is a GLR conflict
    let conflict_cell: Vec<Action> = vec![Action::Accept, Action::Shift(StateId(0))];
    assert_eq!(conflict_cell.len(), 2);

    // Dedup does not collapse Accept with other variants
    let mut mixed = vec![Action::Accept, Action::Error, Action::Accept];
    mixed.dedup();
    // Adjacent Accept duplicates are removed
    assert!(mixed.contains(&Action::Accept));
    assert!(mixed.contains(&Action::Error));
}

// ---------------------------------------------------------------------------
// 36. Error action does not equal any other variant
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_action_ne_others(s in 0u16..1000, r in 0u16..1000) {
        let error = Action::Error;
        prop_assert_ne!(error.clone(), Action::Shift(StateId(s)));
        prop_assert_ne!(error.clone(), Action::Reduce(RuleId(r)));
        prop_assert_ne!(error.clone(), Action::Accept);
        prop_assert_ne!(error.clone(), Action::Recover);
        prop_assert_ne!(error, Action::Fork(vec![]));
    }
}

// ---------------------------------------------------------------------------
// 37. Reduce-reduce conflict cell has multiple reduces
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn reduce_reduce_conflict(r1 in 0u16..1000, r2 in 0u16..1000) {
        prop_assume!(r1 != r2);
        let cell: Vec<Action> = vec![
            Action::Reduce(RuleId(r1)),
            Action::Reduce(RuleId(r2)),
        ];
        let reduce_count = cell.iter().filter(|a| matches!(a, Action::Reduce(_))).count();
        prop_assert_eq!(reduce_count, 2);
    }
}

// ===========================================================================
// Additional tests — added by agent-317
// ===========================================================================

// ---------------------------------------------------------------------------
// 38. Shift with boundary state values
// ---------------------------------------------------------------------------

#[test]
fn shift_boundary_states() {
    let min_shift = Action::Shift(StateId(0));
    let max_shift = Action::Shift(StateId(u16::MAX));
    assert_ne!(min_shift, max_shift);
    assert_eq!(min_shift, Action::Shift(StateId(0)));
    assert_eq!(max_shift, Action::Shift(StateId(u16::MAX)));
}

// ---------------------------------------------------------------------------
// 39. Reduce with boundary rule values
// ---------------------------------------------------------------------------

#[test]
fn reduce_boundary_rules() {
    let min_reduce = Action::Reduce(RuleId(0));
    let max_reduce = Action::Reduce(RuleId(u16::MAX));
    assert_ne!(min_reduce, max_reduce);
    assert_eq!(min_reduce, Action::Reduce(RuleId(0)));
    assert_eq!(max_reduce, Action::Reduce(RuleId(u16::MAX)));
}

// ---------------------------------------------------------------------------
// 40. Debug output for Shift contains the state number
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_shift_contains_state_number(s in 0u16..10000) {
        let action = Action::Shift(StateId(s));
        let dbg = format!("{:?}", action);
        prop_assert!(dbg.contains(&s.to_string()), "Debug {:?} should contain state {}", dbg, s);
    }
}

// ---------------------------------------------------------------------------
// 41. Debug output for Reduce contains the rule number
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_reduce_contains_rule_number(r in 0u16..10000) {
        let action = Action::Reduce(RuleId(r));
        let dbg = format!("{:?}", action);
        prop_assert!(dbg.contains(&r.to_string()), "Debug {:?} should contain rule {}", dbg, r);
    }
}

// ---------------------------------------------------------------------------
// 42. Fork with single child is distinct from the child itself
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fork_single_child_ne_child(action in leaf_action()) {
        let fork = Action::Fork(vec![action.clone()]);
        prop_assert_ne!(fork, action, "Fork([x]) must not equal x");
    }
}

// ---------------------------------------------------------------------------
// 43. ActionCell retain filters correctly
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_retain_shifts_only(cell in arb_action_cell()) {
        let mut shifts_only = cell.clone();
        shifts_only.retain(|a| matches!(a, Action::Shift(_)));
        for a in &shifts_only {
            prop_assert!(matches!(a, Action::Shift(_)));
        }
        prop_assert!(shifts_only.len() <= cell.len());
    }
}

// ---------------------------------------------------------------------------
// 44. ActionCell retain reduces correctly
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_retain_reduces_only(cell in arb_action_cell()) {
        let mut reduces_only = cell.clone();
        reduces_only.retain(|a| matches!(a, Action::Reduce(_)));
        for a in &reduces_only {
            prop_assert!(matches!(a, Action::Reduce(_)));
        }
        prop_assert!(reduces_only.len() <= cell.len());
    }
}

// ---------------------------------------------------------------------------
// 45. ActionCell split: shifts + reduces + others = original length
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_partition_complete(cell in arb_action_cell()) {
        let shifts = cell.iter().filter(|a| matches!(a, Action::Shift(_))).count();
        let reduces = cell.iter().filter(|a| matches!(a, Action::Reduce(_))).count();
        let accepts = cell.iter().filter(|a| matches!(a, Action::Accept)).count();
        let errors = cell.iter().filter(|a| matches!(a, Action::Error)).count();
        let recovers = cell.iter().filter(|a| matches!(a, Action::Recover)).count();
        let forks = cell.iter().filter(|a| matches!(a, Action::Fork(_))).count();
        prop_assert_eq!(shifts + reduces + accepts + errors + recovers + forks, cell.len());
    }
}

// ---------------------------------------------------------------------------
// 46. Serde JSON: Shift roundtrip preserves state value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_json_shift_preserves_state(s in 0u16..=u16::MAX) {
        let action = Action::Shift(StateId(s));
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(action, decoded);
    }
}

// ---------------------------------------------------------------------------
// 47. Serde JSON: Reduce roundtrip preserves rule value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_json_reduce_preserves_rule(r in 0u16..=u16::MAX) {
        let action = Action::Reduce(RuleId(r));
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(action, decoded);
    }
}

// ---------------------------------------------------------------------------
// 48. ActionCell iter position matches index
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_iter_position_consistency(cell in arb_action_cell()) {
        for (i, action) in cell.iter().enumerate() {
            prop_assert_eq!(action, &cell[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 49. ActionCell reverse then reverse is identity
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_reverse_reverse_identity(cell in arb_action_cell()) {
        let mut reversed = cell.clone();
        reversed.reverse();
        reversed.reverse();
        prop_assert_eq!(cell, reversed);
    }
}

// ---------------------------------------------------------------------------
// 50. ActionCell drain empties the cell
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_drain_empties(cell in arb_action_cell()) {
        let original_len = cell.len();
        let mut cell = cell;
        let drained: Vec<Action> = cell.drain(..).collect();
        prop_assert!(cell.is_empty());
        prop_assert_eq!(drained.len(), original_len);
    }
}

// ---------------------------------------------------------------------------
// 51. Shift(0) is not Error (distinct semantics)
// ---------------------------------------------------------------------------

#[test]
fn shift_zero_ne_error() {
    assert_ne!(Action::Shift(StateId(0)), Action::Error);
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
    assert_ne!(Action::Shift(StateId(0)), Action::Recover);
}

// ---------------------------------------------------------------------------
// 52. Reduce(0) is not Error (distinct semantics)
// ---------------------------------------------------------------------------

#[test]
fn reduce_zero_ne_error() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Error);
    assert_ne!(Action::Reduce(RuleId(0)), Action::Accept);
    assert_ne!(Action::Reduce(RuleId(0)), Action::Recover);
}

// ---------------------------------------------------------------------------
// 53. Fork of all same actions has one unique element
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fork_all_same_has_one_unique(action in leaf_action(), n in 2usize..=6) {
        let children: Vec<Action> = vec![action.clone(); n];
        let fork = Action::Fork(children);
        if let Action::Fork(inner) = &fork {
            let unique: HashSet<_> = inner.iter().collect();
            prop_assert_eq!(unique.len(), 1);
        }
    }
}

// ---------------------------------------------------------------------------
// 54. ActionCell determinism: single Shift means no Reduce
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn deterministic_shift_cell(s in 0u16..=u16::MAX) {
        let cell: Vec<Action> = vec![Action::Shift(StateId(s))];
        let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
        prop_assert!(!has_reduce, "Deterministic shift cell must have no reduces");
    }
}

// ---------------------------------------------------------------------------
// 55. ActionCell determinism: single Reduce means no Shift
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn deterministic_reduce_cell(r in 0u16..=u16::MAX) {
        let cell: Vec<Action> = vec![Action::Reduce(RuleId(r))];
        let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
        prop_assert!(!has_shift, "Deterministic reduce cell must have no shifts");
    }
}

// ---------------------------------------------------------------------------
// 56. ActionCell pop removes last element
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_pop_removes_last(cell in prop::collection::vec(arb_action(), 1..=8)) {
        let mut cell = cell;
        let last = cell.last().cloned().unwrap();
        let popped = cell.pop().unwrap();
        prop_assert_eq!(popped, last);
    }
}

// ---------------------------------------------------------------------------
// 57. ActionCell truncate reduces length
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_truncate_reduces_len(cell in prop::collection::vec(arb_action(), 2..=10), cut in 0usize..=1) {
        let original_len = cell.len();
        let mut cell = cell;
        cell.truncate(cut);
        prop_assert!(cell.len() <= cut);
        prop_assert!(cell.len() <= original_len);
    }
}

// ---------------------------------------------------------------------------
// 58. Recover action clones correctly
// ---------------------------------------------------------------------------

#[test]
fn recover_clone_eq() {
    let r = Action::Recover;
    let cloned = r.clone();
    assert_eq!(r, cloned);
    assert_eq!(format!("{:?}", r), format!("{:?}", cloned));
}

// ---------------------------------------------------------------------------
// 59. Fork nesting: Fork(Fork(x)) has depth 2
// ---------------------------------------------------------------------------

#[test]
fn fork_nesting_depth_two() {
    let inner = Action::Fork(vec![Action::Accept]);
    let outer = Action::Fork(vec![inner.clone()]);
    assert_ne!(inner, outer);

    fn depth(a: &Action) -> usize {
        match a {
            Action::Fork(ch) => 1 + ch.iter().map(|c| depth(c)).max().unwrap_or(0),
            _ => 0,
        }
    }
    assert_eq!(depth(&outer), 2);
}

// ---------------------------------------------------------------------------
// 60. Hash determinism: same action hashes identically across calls
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hash_deterministic(action in arb_action()) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut h1 = DefaultHasher::new();
        action.hash(&mut h1);
        let hash1 = h1.finish();

        let mut h2 = DefaultHasher::new();
        action.hash(&mut h2);
        let hash2 = h2.finish();

        prop_assert_eq!(hash1, hash2, "Same action must hash identically");
    }
}

// ---------------------------------------------------------------------------
// 61. ActionCell from_iter roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_from_iter_roundtrip(cell in arb_action_cell()) {
        let collected: Vec<Action> = cell.iter().cloned().collect();
        prop_assert_eq!(&cell, &collected);
    }
}

// ---------------------------------------------------------------------------
// 62. ActionCell split_at preserves total length
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_split_preserves_total(cell in prop::collection::vec(arb_action(), 1..=8)) {
        let mid = cell.len() / 2;
        let (left, right) = cell.split_at(mid);
        prop_assert_eq!(left.len() + right.len(), cell.len());
        // Recombine
        let mut recombined = left.to_vec();
        recombined.extend_from_slice(right);
        prop_assert_eq!(cell, recombined);
    }
}

// ---------------------------------------------------------------------------
// 63. Three-way shift-reduce-accept conflict cell
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn three_way_conflict_cell(s in 0u16..1000, r in 0u16..1000) {
        let cell: Vec<Action> = vec![
            Action::Shift(StateId(s)),
            Action::Reduce(RuleId(r)),
            Action::Accept,
        ];
        prop_assert_eq!(cell.len(), 3);
        prop_assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
        prop_assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
        prop_assert!(cell.iter().any(|a| matches!(a, Action::Accept)));
    }
}

// ---------------------------------------------------------------------------
// 64. ActionCell windows produces overlapping pairs
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_windows_pairs(cell in prop::collection::vec(arb_action(), 2..=8)) {
        let pairs: Vec<_> = cell.windows(2).collect();
        prop_assert_eq!(pairs.len(), cell.len() - 1);
        for (i, window) in pairs.iter().enumerate() {
            prop_assert_eq!(&window[0], &cell[i]);
            prop_assert_eq!(&window[1], &cell[i + 1]);
        }
    }
}

// ---------------------------------------------------------------------------
// 65. ActionCell into_iter then collect roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_into_iter_collect_roundtrip(cell in arb_action_cell()) {
        let original = cell.clone();
        let collected: Vec<Action> = cell.into_iter().collect();
        prop_assert_eq!(original, collected);
    }
}

// ---------------------------------------------------------------------------
// 66. Fork equality is structural
// ---------------------------------------------------------------------------

#[test]
fn fork_structural_equality() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let c = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(3))]);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ---------------------------------------------------------------------------
// 67. All six variants are mutually distinguishable
// ---------------------------------------------------------------------------

#[test]
fn all_variants_distinguishable() {
    let variants: Vec<Action> = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];
    for i in 0..variants.len() {
        for j in 0..variants.len() {
            if i == j {
                assert_eq!(variants[i], variants[j]);
            } else {
                assert_ne!(
                    variants[i], variants[j],
                    "variant {} and {} should differ",
                    i, j
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 68. ActionCell with_capacity does not affect content
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_with_capacity_content(actions in prop::collection::vec(leaf_action(), 0..=6)) {
        let mut cell: Vec<Action> = Vec::with_capacity(100);
        for a in &actions {
            cell.push(a.clone());
        }
        prop_assert_eq!(cell, actions);
    }
}

// ---------------------------------------------------------------------------
// 69. Serde JSON: Error variant serializes/deserializes distinctly
// ---------------------------------------------------------------------------

#[test]
fn serde_json_error_distinct() {
    let error_json = serde_json::to_string(&Action::Error).unwrap();
    let accept_json = serde_json::to_string(&Action::Accept).unwrap();
    let recover_json = serde_json::to_string(&Action::Recover).unwrap();
    assert_ne!(error_json, accept_json);
    assert_ne!(error_json, recover_json);
    assert_ne!(accept_json, recover_json);

    let error: Action = serde_json::from_str(&error_json).unwrap();
    assert_eq!(error, Action::Error);
}

// ---------------------------------------------------------------------------
// 70. ActionCell filter_map extracts shift states
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_filter_map_shift_states(cell in arb_action_cell()) {
        let shift_states: Vec<StateId> = cell
            .iter()
            .filter_map(|a| match a {
                Action::Shift(s) => Some(*s),
                _ => None,
            })
            .collect();
        let shift_count = cell.iter().filter(|a| matches!(a, Action::Shift(_))).count();
        prop_assert_eq!(shift_states.len(), shift_count);
    }
}

// ---------------------------------------------------------------------------
// 71. ActionCell filter_map extracts reduce rule ids
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_filter_map_reduce_rules(cell in arb_action_cell()) {
        let rule_ids: Vec<RuleId> = cell
            .iter()
            .filter_map(|a| match a {
                Action::Reduce(r) => Some(*r),
                _ => None,
            })
            .collect();
        let reduce_count = cell.iter().filter(|a| matches!(a, Action::Reduce(_))).count();
        prop_assert_eq!(rule_ids.len(), reduce_count);
    }
}

// ---------------------------------------------------------------------------
// 72. ActionCell is deterministic iff it has at most one action
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cell_determinism_check(cell in arb_action_cell()) {
        let is_deterministic = cell.len() <= 1;
        if is_deterministic {
            prop_assert!(cell.len() <= 1);
        } else {
            prop_assert!(cell.len() > 1, "Non-deterministic cell must have >1 actions");
        }
    }
}
