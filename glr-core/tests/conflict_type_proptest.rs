#![allow(clippy::needless_range_loop)]

//! Property-based tests for `ConflictType` in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test conflict_type_proptest`

use adze_glr_core::{Action, Conflict, ConflictResolver, ConflictType};
use adze_ir::{RuleId, StateId, SymbolId};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a random `ConflictType`.
fn arb_conflict_type() -> impl Strategy<Value = ConflictType> {
    prop_oneof![
        Just(ConflictType::ShiftReduce),
        Just(ConflictType::ReduceReduce),
    ]
}

/// Generate a random `StateId`.
fn arb_state_id() -> impl Strategy<Value = StateId> {
    (0..1000u16).prop_map(StateId)
}

/// Generate a random `SymbolId`.
fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0..1000u16).prop_map(SymbolId)
}

/// Generate a leaf Action (no Fork).
fn leaf_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..500u16).prop_map(|s| Action::Shift(StateId(s))),
        (0..500u16).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Generate a non-empty action vec.
fn arb_actions() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(leaf_action(), 1..=8)
}

/// Generate a random `Conflict`.
fn arb_conflict() -> impl Strategy<Value = Conflict> {
    (
        arb_state_id(),
        arb_symbol_id(),
        arb_actions(),
        arb_conflict_type(),
    )
        .prop_map(|(state, symbol, actions, conflict_type)| Conflict {
            state,
            symbol,
            actions,
            conflict_type,
        })
}

/// Generate a `ConflictResolver` with random conflicts.
fn arb_conflict_resolver() -> impl Strategy<Value = ConflictResolver> {
    prop::collection::vec(arb_conflict(), 0..=10)
        .prop_map(|conflicts| ConflictResolver { conflicts })
}

// ===========================================================================
// 1. ConflictType creation — ShiftReduce
// ===========================================================================

proptest! {
    #[test]
    fn shift_reduce_creation(_seed in 0u32..1000) {
        let ct = ConflictType::ShiftReduce;
        prop_assert!(matches!(ct, ConflictType::ShiftReduce));
    }
}

// ===========================================================================
// 2. ConflictType creation — ReduceReduce
// ===========================================================================

proptest! {
    #[test]
    fn reduce_reduce_creation(_seed in 0u32..1000) {
        let ct = ConflictType::ReduceReduce;
        prop_assert!(matches!(ct, ConflictType::ReduceReduce));
    }
}

// ===========================================================================
// 3. ConflictType equality — reflexive
// ===========================================================================

proptest! {
    #[test]
    fn equality_reflexive(ct in arb_conflict_type()) {
        prop_assert_eq!(&ct, &ct);
    }
}

// ===========================================================================
// 4. ConflictType equality — symmetric
// ===========================================================================

proptest! {
    #[test]
    fn equality_symmetric(a in arb_conflict_type(), b in arb_conflict_type()) {
        prop_assert_eq!(a == b, b == a);
    }
}

// ===========================================================================
// 5. ConflictType equality — transitive
// ===========================================================================

proptest! {
    #[test]
    fn equality_transitive(
        a in arb_conflict_type(),
        b in arb_conflict_type(),
        c in arb_conflict_type(),
    ) {
        if a == b && b == c {
            prop_assert_eq!(a, c);
        }
    }
}

// ===========================================================================
// 6. ConflictType equality — same variant equals
// ===========================================================================

proptest! {
    #[test]
    fn same_variant_always_equal(_seed in 0u32..100) {
        prop_assert_eq!(ConflictType::ShiftReduce, ConflictType::ShiftReduce);
        prop_assert_eq!(ConflictType::ReduceReduce, ConflictType::ReduceReduce);
    }
}

// ===========================================================================
// 7. ConflictType equality — different variants not equal
// ===========================================================================

proptest! {
    #[test]
    fn different_variants_not_equal(_seed in 0u32..100) {
        prop_assert_ne!(ConflictType::ShiftReduce, ConflictType::ReduceReduce);
        prop_assert_ne!(ConflictType::ReduceReduce, ConflictType::ShiftReduce);
    }
}

// ===========================================================================
// 8. ConflictType clone — preserves variant
// ===========================================================================

proptest! {
    #[test]
    fn clone_preserves_variant(ct in arb_conflict_type()) {
        let cloned = ct.clone();
        prop_assert_eq!(ct, cloned);
    }
}

// ===========================================================================
// 9. ConflictType clone — independent copy
// ===========================================================================

proptest! {
    #[test]
    fn clone_produces_independent_copy(ct in arb_conflict_type()) {
        let cloned = ct.clone();
        // Mutating original via reassignment should not affect clone
        let _original_moved = ct;
        prop_assert!(
            matches!(cloned, ConflictType::ShiftReduce)
                || matches!(cloned, ConflictType::ReduceReduce)
        );
    }
}

// ===========================================================================
// 10. ConflictType debug — ShiftReduce display
// ===========================================================================

proptest! {
    #[test]
    fn debug_shift_reduce_contains_name(_seed in 0u32..100) {
        let dbg = format!("{:?}", ConflictType::ShiftReduce);
        prop_assert!(dbg.contains("ShiftReduce"));
    }
}

// ===========================================================================
// 11. ConflictType debug — ReduceReduce display
// ===========================================================================

proptest! {
    #[test]
    fn debug_reduce_reduce_contains_name(_seed in 0u32..100) {
        let dbg = format!("{:?}", ConflictType::ReduceReduce);
        prop_assert!(dbg.contains("ReduceReduce"));
    }
}

// ===========================================================================
// 12. ConflictType debug — non-empty output
// ===========================================================================

proptest! {
    #[test]
    fn debug_output_is_non_empty(ct in arb_conflict_type()) {
        let dbg = format!("{:?}", ct);
        prop_assert!(!dbg.is_empty());
    }
}

// ===========================================================================
// 13. ConflictType debug — deterministic
// ===========================================================================

proptest! {
    #[test]
    fn debug_output_is_deterministic(ct in arb_conflict_type()) {
        let d1 = format!("{:?}", ct);
        let d2 = format!("{:?}", ct);
        prop_assert_eq!(d1, d2);
    }
}

// ===========================================================================
// 14. ConflictType variants coverage — exactly two
// ===========================================================================

proptest! {
    #[test]
    fn variant_is_one_of_two(ct in arb_conflict_type()) {
        let is_sr = matches!(ct, ConflictType::ShiftReduce);
        let is_rr = matches!(ct, ConflictType::ReduceReduce);
        prop_assert!(is_sr || is_rr);
        prop_assert!(is_sr != is_rr, "exactly one variant must match");
    }
}

// ===========================================================================
// 15. ConflictType variants coverage — strategy covers both
// ===========================================================================

#[test]
fn strategy_covers_both_variants() {
    let mut seen_sr = false;
    let mut seen_rr = false;
    let mut runner = proptest::test_runner::TestRunner::default();
    let strategy = arb_conflict_type();
    for _ in 0..200 {
        let ct = strategy.new_tree(&mut runner).unwrap().current();
        match ct {
            ConflictType::ShiftReduce => seen_sr = true,
            ConflictType::ReduceReduce => seen_rr = true,
        }
        if seen_sr && seen_rr {
            break;
        }
    }
    assert!(seen_sr, "strategy should produce ShiftReduce");
    assert!(seen_rr, "strategy should produce ReduceReduce");
}

// ===========================================================================
// 16. ConflictType in Conflict struct — roundtrip
// ===========================================================================

proptest! {
    #[test]
    fn conflict_preserves_type(ct in arb_conflict_type()) {
        let conflict = Conflict {
            state: StateId(0),
            symbol: SymbolId(1),
            actions: vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))],
            conflict_type: ct.clone(),
        };
        prop_assert_eq!(conflict.conflict_type, ct);
    }
}

// ===========================================================================
// 17. ConflictType in Conflict struct — clone propagates
// ===========================================================================

proptest! {
    #[test]
    fn conflict_clone_preserves_type(c in arb_conflict()) {
        let cloned = c.clone();
        prop_assert_eq!(c.conflict_type, cloned.conflict_type);
    }
}

// ===========================================================================
// 18. ConflictType in Conflict struct — state and symbol preserved
// ===========================================================================

proptest! {
    #[test]
    fn conflict_preserves_all_fields(
        state in arb_state_id(),
        symbol in arb_symbol_id(),
        ct in arb_conflict_type(),
    ) {
        let conflict = Conflict {
            state,
            symbol,
            actions: vec![Action::Accept],
            conflict_type: ct.clone(),
        };
        prop_assert_eq!(conflict.state, state);
        prop_assert_eq!(conflict.symbol, symbol);
        prop_assert_eq!(conflict.conflict_type, ct);
    }
}

// ===========================================================================
// 19. ConflictType in Conflict — debug includes type
// ===========================================================================

proptest! {
    #[test]
    fn conflict_debug_includes_type(c in arb_conflict()) {
        let dbg = format!("{:?}", c);
        let type_str = format!("{:?}", c.conflict_type);
        prop_assert!(dbg.contains(&type_str));
    }
}

// ===========================================================================
// 20. ConflictType pattern matching — exhaustive
// ===========================================================================

proptest! {
    #[test]
    fn pattern_match_is_exhaustive(ct in arb_conflict_type()) {
        let label = match ct {
            ConflictType::ShiftReduce => "sr",
            ConflictType::ReduceReduce => "rr",
        };
        prop_assert!(!label.is_empty());
    }
}

// ===========================================================================
// 21. ConflictType pattern matching — discriminant stable
// ===========================================================================

proptest! {
    #[test]
    fn pattern_match_consistent_with_eq(a in arb_conflict_type(), b in arb_conflict_type()) {
        let same_branch = match (&a, &b) {
            (ConflictType::ShiftReduce, ConflictType::ShiftReduce) => true,
            (ConflictType::ReduceReduce, ConflictType::ReduceReduce) => true,
            _ => false,
        };
        prop_assert_eq!(a == b, same_branch);
    }
}

// ===========================================================================
// 22. ConflictType pattern matching — mapping preserves info
// ===========================================================================

proptest! {
    #[test]
    fn pattern_match_maps_to_distinct_strings(ct in arb_conflict_type()) {
        let mapped = match ct {
            ConflictType::ShiftReduce => "shift-reduce",
            ConflictType::ReduceReduce => "reduce-reduce",
        };
        prop_assert!(mapped == "shift-reduce" || mapped == "reduce-reduce");
    }
}

// ===========================================================================
// 23. ConflictResolver — conflicts vec is accessible
// ===========================================================================

proptest! {
    #[test]
    fn resolver_conflicts_accessible(resolver in arb_conflict_resolver()) {
        let len = resolver.conflicts.len();
        prop_assert!(len <= 10);
        for c in &resolver.conflicts {
            prop_assert!(
                matches!(c.conflict_type, ConflictType::ShiftReduce | ConflictType::ReduceReduce)
            );
        }
    }
}

// ===========================================================================
// 24. ConflictResolver — clone preserves all conflict types
// ===========================================================================

proptest! {
    #[test]
    fn resolver_clone_preserves_types(resolver in arb_conflict_resolver()) {
        let cloned = resolver.clone();
        prop_assert_eq!(resolver.conflicts.len(), cloned.conflicts.len());
        for i in 0..resolver.conflicts.len() {
            prop_assert_eq!(
                &resolver.conflicts[i].conflict_type,
                &cloned.conflicts[i].conflict_type,
            );
        }
    }
}

// ===========================================================================
// 25. ConflictType — collecting into set deduplicates
// ===========================================================================

proptest! {
    #[test]
    fn collecting_into_set_deduplicates(types in prop::collection::vec(arb_conflict_type(), 1..=20)) {
        let set: HashSet<String> = types.iter().map(|ct| format!("{:?}", ct)).collect();
        prop_assert!(set.len() <= 2);
        prop_assert!(set.len() >= 1);
    }
}

// ===========================================================================
// 26. ConflictType — vec of types preserves ordering
// ===========================================================================

proptest! {
    #[test]
    fn vec_preserves_insertion_order(types in prop::collection::vec(arb_conflict_type(), 0..=20)) {
        let cloned: Vec<ConflictType> = types.clone();
        for i in 0..types.len() {
            prop_assert_eq!(&types[i], &cloned[i]);
        }
    }
}

// ===========================================================================
// 27. ConflictType — multiple conflicts with mixed types
// ===========================================================================

proptest! {
    #[test]
    fn mixed_conflict_types_in_resolver(
        sr_count in 0usize..=5,
        rr_count in 0usize..=5,
    ) {
        let mut conflicts = Vec::new();
        for i in 0..sr_count {
            conflicts.push(Conflict {
                state: StateId(i as u16),
                symbol: SymbolId(0),
                actions: vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))],
                conflict_type: ConflictType::ShiftReduce,
            });
        }
        for i in 0..rr_count {
            conflicts.push(Conflict {
                state: StateId((sr_count + i) as u16),
                symbol: SymbolId(0),
                actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
                conflict_type: ConflictType::ReduceReduce,
            });
        }
        let resolver = ConflictResolver { conflicts };
        let actual_sr = resolver
            .conflicts
            .iter()
            .filter(|c| c.conflict_type == ConflictType::ShiftReduce)
            .count();
        let actual_rr = resolver
            .conflicts
            .iter()
            .filter(|c| c.conflict_type == ConflictType::ReduceReduce)
            .count();
        prop_assert_eq!(actual_sr, sr_count);
        prop_assert_eq!(actual_rr, rr_count);
    }
}

// ===========================================================================
// 28. ConflictType — clone then compare
// ===========================================================================

proptest! {
    #[test]
    fn clone_then_compare(ct in arb_conflict_type()) {
        let c1 = ct.clone();
        let c2 = c1.clone();
        prop_assert_eq!(&c1, &c2);
        prop_assert_eq!(&ct, &c2);
    }
}

// ===========================================================================
// 29. ConflictType — pattern matching returns correct tag
// ===========================================================================

proptest! {
    #[test]
    fn pattern_match_tag_roundtrip(ct in arb_conflict_type()) {
        let tag: u8 = match ct {
            ConflictType::ShiftReduce => 0,
            ConflictType::ReduceReduce => 1,
        };
        let reconstructed = match tag {
            0 => ConflictType::ShiftReduce,
            1 => ConflictType::ReduceReduce,
            _ => unreachable!(),
        };
        prop_assert_eq!(ct, reconstructed);
    }
}

// ===========================================================================
// 30. ConflictType — debug string lengths are bounded
// ===========================================================================

proptest! {
    #[test]
    fn debug_string_length_is_bounded(ct in arb_conflict_type()) {
        let dbg = format!("{:?}", ct);
        // Variant names are short; debug output should be under 50 chars
        prop_assert!(dbg.len() < 50, "debug too long: {}", dbg);
    }
}

// ===========================================================================
// 31. Conflict — actions vec is non-empty from strategy
// ===========================================================================

proptest! {
    #[test]
    fn conflict_actions_non_empty(c in arb_conflict()) {
        prop_assert!(!c.actions.is_empty());
    }
}

// ===========================================================================
// 32. ConflictType — if_let chains
// ===========================================================================

proptest! {
    #[test]
    fn if_let_shift_reduce(ct in arb_conflict_type()) {
        if let ConflictType::ShiftReduce = ct {
            prop_assert_ne!(ct, ConflictType::ReduceReduce);
        } else {
            prop_assert_eq!(ct, ConflictType::ReduceReduce);
        }
    }
}
