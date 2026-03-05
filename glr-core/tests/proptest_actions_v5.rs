//! Property-based tests for `Action` enum and parse table action queries (v5).
//!
//! Run with:
//! ```bash
//! cargo test -p adze-glr-core --test proptest_actions_v5 -- --test-threads=2
//! ```

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, StateId, SymbolId};
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

/// Generate a non-empty `Fork` action.
fn arb_fork() -> impl Strategy<Value = Action> {
    prop::collection::vec(leaf_action(), 2..=6).prop_map(Action::Fork)
}

/// Build a simple grammar and return its parse table.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("build_lr1_automaton failed")
}

/// Random valid grammar: 1-4 tokens, 1-4 rules (S -> t_i).
fn arb_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 1usize..=4)
        .prop_flat_map(|(n_tok, n_rules)| {
            let indices = proptest::collection::vec(0..n_tok, n_rules);
            (Just(n_tok), indices)
        })
        .prop_map(|(n_tok, indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("arb");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            b = b.rule("S", vec![tok_names[0].as_str()]);
            for &idx in &indices {
                b = b.rule("S", vec![tok_names[idx].as_str()]);
            }
            b.start("S").build()
        })
}

/// Random grammar with two nonterminals: S -> A, A -> t_i.
fn arb_two_nt_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 0usize..=3)
        .prop_flat_map(|(n_tok, n_extra)| {
            let indices = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), indices)
        })
        .prop_map(|(n_tok, indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("two_nt");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            b = b.rule("S", vec!["A"]);
            b = b.rule("A", vec![tok_names[0].as_str()]);
            for &idx in &indices {
                b = b.rule("A", vec![tok_names[idx].as_str()]);
            }
            b.start("S").build()
        })
}

/// Random grammar with chain: S -> A -> B -> t_i.
fn arb_chain_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=3).prop_map(|n_tok| {
        let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
        let mut b = GrammarBuilder::new("chain");
        for tn in &tok_names {
            b = b.token(tn, tn);
        }
        b = b.rule("S", vec!["A"]);
        b = b.rule("A", vec!["B"]);
        b = b.rule("B", vec![tok_names[0].as_str()]);
        b.start("S").build()
    })
}

/// Collect all Shift targets from a parse table.
fn all_shift_targets(table: &ParseTable) -> Vec<StateId> {
    let mut targets = Vec::new();
    for s in 0..table.state_count {
        for &sym in &table.index_to_symbol {
            for action in table.actions(StateId(s as u16), sym) {
                if let Action::Shift(target) = action {
                    targets.push(*target);
                }
            }
        }
    }
    targets
}

/// Collect all Reduce RuleIds from a parse table.
fn all_reduce_rule_ids(table: &ParseTable) -> Vec<RuleId> {
    let mut ids = Vec::new();
    for s in 0..table.state_count {
        for &sym in &table.index_to_symbol {
            for action in table.actions(StateId(s as u16), sym) {
                if let Action::Reduce(rid) = action {
                    ids.push(*rid);
                }
            }
        }
    }
    ids
}

// ===========================================================================
// CATEGORY 1: Shift action preserves state ID (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn shift_preserves_state_id(sid in 0..=u16::MAX) {
        let action = Action::Shift(StateId(sid));
        match action {
            Action::Shift(s) => prop_assert_eq!(s.0, sid),
            _ => prop_assert!(false, "expected Shift"),
        }
    }

    #[test]
    fn shift_zero_roundtrip(_dummy in 0u8..1) {
        let action = Action::Shift(StateId(0));
        prop_assert_eq!(action, Action::Shift(StateId(0)));
    }

    #[test]
    fn shift_max_roundtrip(_dummy in 0u8..1) {
        let action = Action::Shift(StateId(u16::MAX));
        prop_assert_eq!(action, Action::Shift(StateId(u16::MAX)));
    }

    #[test]
    fn shift_different_ids_differ(a in 0..=u16::MAX, b in 0..=u16::MAX) {
        let sa = Action::Shift(StateId(a));
        let sb = Action::Shift(StateId(b));
        prop_assert_eq!(sa == sb, a == b);
    }

    #[test]
    fn shift_clone_preserves_inner(sid in 0..=u16::MAX) {
        let action = Action::Shift(StateId(sid));
        let cloned = action.clone();
        if let (Action::Shift(a), Action::Shift(b)) = (&action, &cloned) {
            prop_assert_eq!(a.0, b.0);
        } else {
            prop_assert!(false, "clone changed variant");
        }
    }
}

// ===========================================================================
// CATEGORY 2: Reduce action preserves rule_id (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn reduce_preserves_rule_id(rid in 0..=u16::MAX) {
        let action = Action::Reduce(RuleId(rid));
        match action {
            Action::Reduce(r) => prop_assert_eq!(r.0, rid),
            _ => prop_assert!(false, "expected Reduce"),
        }
    }

    #[test]
    fn reduce_zero_roundtrip(_dummy in 0u8..1) {
        let action = Action::Reduce(RuleId(0));
        prop_assert_eq!(action, Action::Reduce(RuleId(0)));
    }

    #[test]
    fn reduce_max_roundtrip(_dummy in 0u8..1) {
        let action = Action::Reduce(RuleId(u16::MAX));
        prop_assert_eq!(action, Action::Reduce(RuleId(u16::MAX)));
    }

    #[test]
    fn reduce_different_ids_differ(a in 0..=u16::MAX, b in 0..=u16::MAX) {
        let ra = Action::Reduce(RuleId(a));
        let rb = Action::Reduce(RuleId(b));
        prop_assert_eq!(ra == rb, a == b);
    }

    #[test]
    fn reduce_clone_preserves_inner(rid in 0..=u16::MAX) {
        let action = Action::Reduce(RuleId(rid));
        let cloned = action.clone();
        if let (Action::Reduce(a), Action::Reduce(b)) = (&action, &cloned) {
            prop_assert_eq!(a.0, b.0);
        } else {
            prop_assert!(false, "clone changed variant");
        }
    }
}

// ===========================================================================
// CATEGORY 3: Fork contains all inner actions (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn fork_preserves_inner_count(actions in prop::collection::vec(leaf_action(), 1..=8)) {
        let expected_len = actions.len();
        let fork = Action::Fork(actions);
        if let Action::Fork(inner) = &fork {
            prop_assert_eq!(inner.len(), expected_len);
        } else {
            prop_assert!(false, "expected Fork");
        }
    }

    #[test]
    fn fork_preserves_inner_elements(actions in prop::collection::vec(leaf_action(), 1..=8)) {
        let expected = actions.clone();
        let fork = Action::Fork(actions);
        if let Action::Fork(inner) = &fork {
            prop_assert_eq!(inner, &expected);
        } else {
            prop_assert!(false, "expected Fork");
        }
    }

    #[test]
    fn fork_order_matters(
        a in leaf_action(),
        b in leaf_action(),
    ) {
        let fork_ab = Action::Fork(vec![a.clone(), b.clone()]);
        let fork_ba = Action::Fork(vec![b.clone(), a.clone()]);
        // Equal iff a == b (same order trivially), or the pair is the same
        if a != b {
            prop_assert_ne!(fork_ab, fork_ba);
        }
    }

    #[test]
    fn fork_clone_preserves_all(actions in prop::collection::vec(leaf_action(), 1..=6)) {
        let fork = Action::Fork(actions);
        let cloned = fork.clone();
        prop_assert_eq!(&fork, &cloned);
    }

    #[test]
    fn fork_nested_preserves_structure(inner in prop::collection::vec(leaf_action(), 2..=4)) {
        let nested = Action::Fork(vec![Action::Fork(inner.clone())]);
        if let Action::Fork(outer) = &nested {
            prop_assert_eq!(outer.len(), 1);
            if let Action::Fork(inner_recovered) = &outer[0] {
                prop_assert_eq!(inner_recovered, &inner);
            } else {
                prop_assert!(false, "expected nested Fork");
            }
        } else {
            prop_assert!(false, "expected Fork");
        }
    }
}

// ===========================================================================
// CATEGORY 4: Action equality and comparison (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn eq_reflexive(action in arb_action()) {
        prop_assert_eq!(&action, &action);
    }

    #[test]
    fn eq_symmetric(a in arb_action(), b in arb_action()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn shift_ne_reduce(sid in 0..=u16::MAX, rid in 0..=u16::MAX) {
        let shift = Action::Shift(StateId(sid));
        let reduce = Action::Reduce(RuleId(rid));
        prop_assert_ne!(shift, reduce);
    }

    #[test]
    fn shift_ne_accept(sid in 0..=u16::MAX) {
        let shift = Action::Shift(StateId(sid));
        prop_assert_ne!(shift, Action::Accept);
    }

    #[test]
    fn reduce_ne_error(rid in 0..=u16::MAX) {
        let reduce = Action::Reduce(RuleId(rid));
        prop_assert_ne!(reduce, Action::Error);
    }
}

// ===========================================================================
// CATEGORY 5: Action Debug output is non-empty (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn debug_shift_nonempty(sid in 0..=u16::MAX) {
        let s = format!("{:?}", Action::Shift(StateId(sid)));
        prop_assert!(!s.is_empty());
    }

    #[test]
    fn debug_reduce_nonempty(rid in 0..=u16::MAX) {
        let s = format!("{:?}", Action::Reduce(RuleId(rid)));
        prop_assert!(!s.is_empty());
    }

    #[test]
    fn debug_accept_nonempty(_dummy in 0u8..1) {
        let s = format!("{:?}", Action::Accept);
        prop_assert!(!s.is_empty());
    }

    #[test]
    fn debug_fork_nonempty(fork in arb_fork()) {
        let s = format!("{:?}", fork);
        prop_assert!(!s.is_empty());
    }

    #[test]
    fn debug_arb_action_nonempty(action in arb_action()) {
        let s = format!("{:?}", action);
        prop_assert!(!s.is_empty());
    }
}

// ===========================================================================
// CATEGORY 6: Parse table actions are valid (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn shift_targets_in_range(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!(
                (target.0 as usize) < table.state_count,
                "shift target {} >= state_count {}", target.0, table.state_count,
            );
        }
    }

    #[test]
    fn reduce_rule_ids_valid(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        for rid in all_reduce_rule_ids(&table) {
            prop_assert!(
                (rid.0 as usize) < table.rules.len(),
                "reduce rule_id {} >= rules.len() {}", rid.0, table.rules.len(),
            );
        }
    }

    #[test]
    fn shift_targets_in_range_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    #[test]
    fn reduce_rule_ids_valid_chain(grammar in arb_chain_grammar()) {
        let table = build_table(&grammar);
        for rid in all_reduce_rule_ids(&table) {
            prop_assert!((rid.0 as usize) < table.rules.len());
        }
    }

    #[test]
    fn table_has_at_least_one_accept(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let eof = table.eof();
        let has_accept = (0..table.state_count).any(|s| {
            table.actions(StateId(s as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        });
        prop_assert!(has_accept, "table should contain at least one Accept action");
    }
}

// ===========================================================================
// CATEGORY 7: Parse table goto entries in range (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn goto_targets_in_range(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if let Some(target) = table.goto(StateId(s as u16), nt) {
                    prop_assert!(
                        (target.0 as usize) < table.state_count,
                        "goto target {} >= state_count {}", target.0, table.state_count,
                    );
                }
            }
        }
    }

    #[test]
    fn goto_targets_in_range_chain(grammar in arb_chain_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if let Some(target) = table.goto(StateId(s as u16), nt) {
                    prop_assert!((target.0 as usize) < table.state_count);
                }
            }
        }
    }

    #[test]
    fn goto_returns_none_for_terminals(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        // Terminals should NOT appear in the nonterminal_to_index map,
        // so goto for a terminal should return None.
        for s in 0..table.state_count {
            for &sym in &table.index_to_symbol {
                if !table.nonterminal_to_index.contains_key(&sym) {
                    prop_assert!(table.goto(StateId(s as u16), sym).is_none());
                }
            }
        }
    }

    #[test]
    fn goto_table_has_rows_for_each_state(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.goto_table.len() >= table.state_count);
    }

    #[test]
    fn goto_nonexistent_symbol_returns_none(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        // SymbolId(u16::MAX) should not be a real nonterminal
        let bogus = SymbolId(u16::MAX);
        for s in 0..table.state_count {
            prop_assert!(table.goto(StateId(s as u16), bogus).is_none());
        }
    }
}

// ===========================================================================
// CATEGORY 8: Parse table determinism (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn actions_query_is_deterministic(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &sym in &table.index_to_symbol {
                let a1 = table.actions(StateId(s as u16), sym);
                let a2 = table.actions(StateId(s as u16), sym);
                prop_assert_eq!(a1, a2);
            }
        }
    }

    #[test]
    fn goto_query_is_deterministic(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                let g1 = table.goto(StateId(s as u16), nt);
                let g2 = table.goto(StateId(s as u16), nt);
                prop_assert_eq!(g1, g2);
            }
        }
    }

    #[test]
    fn rule_lookup_is_deterministic(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        for i in 0..table.rules.len() {
            let r1 = table.rule(RuleId(i as u16));
            let r2 = table.rule(RuleId(i as u16));
            prop_assert_eq!(r1, r2);
        }
    }

    #[test]
    fn eof_symbol_is_stable(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let e1 = table.eof();
        let e2 = table.eof();
        prop_assert_eq!(e1, e2);
    }

    #[test]
    fn initial_state_is_stable(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let s1 = table.initial_state;
        let s2 = table.initial_state;
        prop_assert_eq!(s1, s2);
    }
}

// ===========================================================================
// CATEGORY 9: Edge cases (6 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn hash_consistent_with_eq(a in arb_action(), b in arb_action()) {
        use std::hash::{Hash, Hasher};
        if a == b {
            let mut ha = std::collections::hash_map::DefaultHasher::new();
            let mut hb = std::collections::hash_map::DefaultHasher::new();
            a.hash(&mut ha);
            b.hash(&mut hb);
            prop_assert_eq!(ha.finish(), hb.finish());
        }
    }

    #[test]
    fn action_inserted_in_hashset(action in arb_action()) {
        let mut set = HashSet::new();
        set.insert(action.clone());
        prop_assert!(set.contains(&action));
        prop_assert_eq!(set.len(), 1);
    }

    #[test]
    fn fork_empty_vec_is_valid(_dummy in 0u8..1) {
        // Fork with an empty vec is constructible (no panic).
        let fork = Action::Fork(vec![]);
        if let Action::Fork(inner) = &fork {
            prop_assert!(inner.is_empty());
        } else {
            prop_assert!(false, "expected Fork");
        }
    }

    #[test]
    fn accept_error_recover_are_distinct(_dummy in 0u8..1) {
        prop_assert_ne!(Action::Accept, Action::Error);
        prop_assert_ne!(Action::Accept, Action::Recover);
        prop_assert_ne!(Action::Error, Action::Recover);
    }

    #[test]
    fn actions_for_out_of_range_state_returns_empty(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let bogus_state = StateId(u16::MAX);
        for &sym in &table.index_to_symbol {
            let actions = table.actions(bogus_state, sym);
            prop_assert!(actions.is_empty());
        }
    }

    #[test]
    fn actions_for_unknown_symbol_returns_empty(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let bogus_sym = SymbolId(u16::MAX);
        for s in 0..table.state_count {
            let actions = table.actions(StateId(s as u16), bogus_sym);
            prop_assert!(actions.is_empty());
        }
    }
}
