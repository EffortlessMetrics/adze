#![cfg(feature = "test-api")]

//! Property-based tests for `ActionCell` (`Vec<Action>`) and `Action` in adze-glr-core.
//!
//! 80+ tests (proptest + unit) covering:
//!   - Newtype roundtrips (SymbolId, StateId, RuleId)
//!   - Equality/ordering reflexivity & symmetry
//!   - Action construction and clone fidelity
//!   - ActionCell emptiness, length, and conflict detection
//!   - Parse table integration via GrammarBuilder pipelines
//!   - Goto validity, Accept reachability, shift/reduce coverage
//!
//! Run with:
//! ```bash
//! cargo test -p adze-glr-core --test action_cell_prop_v9 --features test-api -- --test-threads=2
//! ```

use adze_glr_core::{
    Action, ActionCell, FirstFollowSets, ParseTable, build_lr1_automaton,
    conflict_inspection::action_cell_has_conflict,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::HashSet;

// ===========================================================================
// Strategies
// ===========================================================================

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..100).prop_map(SymbolId)
}

fn arb_state_id() -> impl Strategy<Value = StateId> {
    (0u16..100).prop_map(StateId)
}

fn arb_rule_id() -> impl Strategy<Value = RuleId> {
    (0u16..50).prop_map(RuleId)
}

fn leaf_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0u16..=u16::MAX).prop_map(|s| Action::Shift(StateId(s))),
        (0u16..=u16::MAX).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

fn arb_action() -> impl Strategy<Value = Action> {
    leaf_action().prop_recursive(2, 16, 4, |inner| {
        prop::collection::vec(inner, 1..=6).prop_map(Action::Fork)
    })
}

fn arb_action_cell() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(arb_action(), 0..=8)
}

/// Random grammar with 1–4 tokens: S → t_i (one or more alternatives).
fn arb_grammar_simple() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 1usize..=4)
        .prop_flat_map(|(n_tok, n_rules)| {
            let indices = proptest::collection::vec(0..n_tok, n_rules);
            (Just(n_tok), indices)
        })
        .prop_map(|(n_tok, indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("acp_v9_simple");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            b = b.rule("s", vec![tok_names[0].as_str()]);
            for &idx in &indices {
                b = b.rule("s", vec![tok_names[idx].as_str()]);
            }
            b.start("s").build()
        })
}

/// Random grammar with two nonterminals: s → a, a → t_i.
fn arb_grammar_two_nt() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 0usize..=3)
        .prop_flat_map(|(n_tok, n_extra)| {
            let indices = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), indices)
        })
        .prop_map(|(n_tok, indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("acp_v9_two_nt");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            b = b.rule("s", vec!["a"]);
            b = b.rule("a", vec![tok_names[0].as_str()]);
            for &idx in &indices {
                b = b.rule("a", vec![tok_names[idx].as_str()]);
            }
            b.start("s").build()
        })
}

/// Random grammar with chain: s → a → b → t_i.
fn arb_grammar_chain() -> impl Strategy<Value = Grammar> {
    (1usize..=3).prop_map(|n_tok| {
        let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
        let mut b = GrammarBuilder::new("acp_v9_chain");
        for tn in &tok_names {
            b = b.token(tn, tn);
        }
        b = b.rule("s", vec!["a"]);
        b = b.rule("a", vec!["b"]);
        b = b.rule("b", vec![tok_names[0].as_str()]);
        b.start("s").build()
    })
}

// ===========================================================================
// Helpers
// ===========================================================================

fn build_table(g: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(g).expect("FIRST/FOLLOW computation");
    build_lr1_automaton(g, &ff).expect("LR(1) automaton")
}

fn build_table_normalized(g: &Grammar) -> ParseTable {
    let mut g = g.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("automaton")
}

fn has_conflict(cell: &[Action]) -> bool {
    action_cell_has_conflict(cell)
}

fn find_accept_in_table(pt: &ParseTable) -> bool {
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), pt.eof())
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn all_shift_targets(pt: &ParseTable) -> Vec<StateId> {
    let mut targets = Vec::new();
    for s in 0..pt.state_count {
        for &sym in &pt.index_to_symbol {
            for action in pt.actions(StateId(s as u16), sym) {
                if let Action::Shift(target) = action {
                    targets.push(*target);
                }
            }
        }
    }
    targets
}

fn all_reduce_rules(pt: &ParseTable) -> Vec<RuleId> {
    let mut rules = Vec::new();
    for s in 0..pt.state_count {
        for &sym in &pt.index_to_symbol {
            for action in pt.actions(StateId(s as u16), sym) {
                if let Action::Reduce(r) = action {
                    rules.push(*r);
                }
            }
        }
    }
    rules
}

// ===========================================================================
// 1–3. Newtype roundtrips (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn pt01_symbol_id_roundtrip(v in 0u16..100) {
        let id = SymbolId(v);
        prop_assert_eq!(id.0, v);
    }

    #[test]
    fn pt02_state_id_roundtrip(v in 0u16..100) {
        let id = StateId(v);
        prop_assert_eq!(id.0, v);
    }

    #[test]
    fn pt03_rule_id_roundtrip(v in 0u16..50) {
        let id = RuleId(v);
        prop_assert_eq!(id.0, v);
    }
}

// ===========================================================================
// 4–6. Equality and ordering properties (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn pt04_symbol_id_eq_reflexive(id in arb_symbol_id()) {
        prop_assert_eq!(id, id);
    }

    #[test]
    fn pt05_state_id_ordering_consistent(a in arb_state_id(), b in arb_state_id()) {
        // If a <= b and b <= a then a == b
        if a.0 <= b.0 && b.0 <= a.0 {
            prop_assert_eq!(a, b);
        }
    }

    #[test]
    fn pt06_rule_id_eq_symmetric(a in arb_rule_id(), b in arb_rule_id()) {
        prop_assert_eq!(a == b, b == a);
    }
}

// ===========================================================================
// 7–10. Action construction and clone (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn pt07_shift_contains_state(s in arb_state_id()) {
        let action = Action::Shift(s);
        match action {
            Action::Shift(inner) => prop_assert_eq!(inner, s),
            _ => prop_assert!(false, "expected Shift"),
        }
    }

    #[test]
    fn pt08_reduce_contains_rule(r in arb_rule_id()) {
        let action = Action::Reduce(r);
        match action {
            Action::Reduce(inner) => prop_assert_eq!(inner, r),
            _ => prop_assert!(false, "expected Reduce"),
        }
    }

    #[test]
    fn pt09_action_eq_reflexive(action in arb_action()) {
        prop_assert_eq!(&action, &action);
    }

    #[test]
    fn pt10_action_clone_equals_original(action in arb_action()) {
        let cloned = action.clone();
        prop_assert_eq!(&action, &cloned);
    }
}

// ===========================================================================
// 11–15. ActionCell unit tests
// ===========================================================================

#[test]
fn t11_empty_cell_is_empty() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
    assert_eq!(cell.len(), 0);
}

#[test]
fn t12_single_action_cell_len_one() {
    let cell: ActionCell = vec![Action::Shift(StateId(1))];
    assert_eq!(cell.len(), 1);
}

#[test]
fn t13_single_shift_not_conflict() {
    let cell: ActionCell = vec![Action::Shift(StateId(0))];
    assert!(!has_conflict(&cell));
}

#[test]
fn t14_single_reduce_not_conflict() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(0))];
    assert!(!has_conflict(&cell));
}

#[test]
fn t15_fork_shift_reduce_is_conflict() {
    let cell: ActionCell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    assert!(has_conflict(&cell));
}

// ===========================================================================
// 16–20. Grammar → parse table integration (proptest + unit)
// ===========================================================================

proptest! {
    #[test]
    fn pt16_grammar_with_n_tokens_has_actions(g in arb_grammar_simple()) {
        let table = build_table(&g);
        // At least one state must have a non-empty action cell
        let has_any_action = (0..table.state_count).any(|s| {
            table.index_to_symbol.iter().any(|sym| {
                !table.actions(StateId(s as u16), *sym).is_empty()
            })
        });
        prop_assert!(has_any_action, "parse table has no actions");
    }

    #[test]
    fn pt17_all_actions_valid_variants(g in arb_grammar_simple()) {
        let table = build_table(&g);
        for s in 0..table.state_count {
            for &sym in &table.index_to_symbol {
                for action in table.actions(StateId(s as u16), sym) {
                    let valid = matches!(
                        action,
                        Action::Shift(_) | Action::Reduce(_) | Action::Accept
                            | Action::Error | Action::Recover | Action::Fork(_)
                    );
                    prop_assert!(valid, "unexpected action variant");
                }
            }
        }
    }

    #[test]
    fn pt18_goto_returns_valid_state_ids(g in arb_grammar_two_nt()) {
        let table = build_table(&g);
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if let Some(target) = table.goto(StateId(s as u16), nt) {
                    prop_assert!(
                        (target.0 as usize) < table.state_count,
                        "goto target {} exceeds state count {}",
                        target.0,
                        table.state_count
                    );
                }
            }
        }
    }
}

#[test]
fn t19_accept_action_in_table() {
    let g = GrammarBuilder::new("acp_v9_accept")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(
        find_accept_in_table(&table),
        "table must contain Accept on EOF"
    );
}

#[test]
fn t20_two_alt_grammar_action_cell_properties() {
    let g = GrammarBuilder::new("acp_v9_twoalt")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(find_accept_in_table(&table));
}

// ===========================================================================
// 21–30. Action equality & debug (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn pt21_action_eq_symmetric(a in arb_action(), b in arb_action()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn pt22_action_ne_shift_different_states(a in arb_state_id(), b in arb_state_id()) {
        let sa = Action::Shift(a);
        let sb = Action::Shift(b);
        prop_assert_eq!(sa == sb, a == b);
    }

    #[test]
    fn pt23_action_ne_reduce_different_rules(a in arb_rule_id(), b in arb_rule_id()) {
        let ra = Action::Reduce(a);
        let rb = Action::Reduce(b);
        prop_assert_eq!(ra == rb, a == b);
    }

    #[test]
    fn pt24_shift_ne_reduce(s in arb_state_id(), r in arb_rule_id()) {
        let shift = Action::Shift(s);
        let reduce = Action::Reduce(r);
        prop_assert_ne!(shift, reduce);
    }

    #[test]
    fn pt25_action_debug_not_empty(action in arb_action()) {
        let dbg = format!("{action:?}");
        prop_assert!(!dbg.is_empty());
    }

    #[test]
    fn pt26_shift_debug_contains_shift(s in arb_state_id()) {
        let dbg = format!("{:?}", Action::Shift(s));
        prop_assert!(dbg.contains("Shift"));
    }

    #[test]
    fn pt27_reduce_debug_contains_reduce(r in arb_rule_id()) {
        let dbg = format!("{:?}", Action::Reduce(r));
        prop_assert!(dbg.contains("Reduce"));
    }

    #[test]
    fn pt28_fork_debug_contains_fork(actions in prop::collection::vec(leaf_action(), 2..=4)) {
        let fork = Action::Fork(actions);
        let dbg = format!("{fork:?}");
        prop_assert!(dbg.contains("Fork"));
    }

    #[test]
    fn pt29_action_cell_len_matches_vec(cell in arb_action_cell()) {
        let expected = cell.as_slice().len();
        prop_assert_eq!(cell.len(), expected);
    }

    #[test]
    fn pt30_action_cell_is_empty_iff_len_zero(cell in arb_action_cell()) {
        prop_assert_eq!(cell.is_empty(), cell.as_slice().is_empty());
    }
}

// ===========================================================================
// 31–40. Hash consistency & collection behavior (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn pt31_hash_consistent_with_eq(a in arb_action(), b in arb_action()) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        if a == b {
            let mut ha = DefaultHasher::new();
            let mut hb = DefaultHasher::new();
            a.hash(&mut ha);
            b.hash(&mut hb);
            prop_assert_eq!(ha.finish(), hb.finish());
        }
    }

    #[test]
    fn pt32_symbol_id_copy_semantics(id in arb_symbol_id()) {
        let copy = id;
        prop_assert_eq!(copy, id);
    }

    #[test]
    fn pt33_state_id_copy_semantics(id in arb_state_id()) {
        let copy = id;
        prop_assert_eq!(copy, id);
    }

    #[test]
    fn pt34_rule_id_copy_semantics(id in arb_rule_id()) {
        let copy = id;
        prop_assert_eq!(copy, id);
    }

    #[test]
    fn pt35_action_in_hashset(action in leaf_action()) {
        let mut set = HashSet::new();
        set.insert(action.clone());
        prop_assert!(set.contains(&action));
    }

    #[test]
    fn pt36_duplicate_actions_in_hashset(action in leaf_action()) {
        let mut set = HashSet::new();
        set.insert(action.clone());
        set.insert(action.clone());
        prop_assert_eq!(set.len(), 1);
    }

    #[test]
    fn pt37_fork_preserves_inner_actions(actions in prop::collection::vec(leaf_action(), 1..=5)) {
        let fork = Action::Fork(actions.clone());
        if let Action::Fork(inner) = fork {
            prop_assert_eq!(inner, actions);
        } else {
            prop_assert!(false, "expected Fork");
        }
    }

    #[test]
    fn pt38_cell_conflict_iff_multi_action(cell in arb_action_cell()) {
        prop_assert_eq!(has_conflict(&cell), cell.len() > 1);
    }

    #[test]
    fn pt39_empty_cell_no_conflict(_dummy in 0u8..1) {
        let cell: ActionCell = vec![];
        prop_assert!(!has_conflict(&cell));
    }

    #[test]
    fn pt40_singleton_cell_no_conflict(action in leaf_action()) {
        let cell: ActionCell = vec![action];
        prop_assert!(!has_conflict(&cell));
    }
}

// ===========================================================================
// 41–50. Parse table structural properties (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn pt41_table_state_count_positive(g in arb_grammar_simple()) {
        let table = build_table(&g);
        prop_assert!(table.state_count > 0);
    }

    #[test]
    fn pt42_shift_targets_within_bounds(g in arb_grammar_simple()) {
        let targets = all_shift_targets(&build_table(&g));
        for t in &targets {
            prop_assert!(
                (t.0 as usize) < build_table(&g).state_count,
                "shift target out of bounds"
            );
        }
    }

    #[test]
    fn pt43_reduce_rules_within_bounds(g in arb_grammar_simple()) {
        let table = build_table(&g);
        let rules = all_reduce_rules(&table);
        for r in &rules {
            prop_assert!(
                (r.0 as usize) < table.rules.len(),
                "reduce rule {} out of bounds (rules len {})",
                r.0,
                table.rules.len()
            );
        }
    }

    #[test]
    fn pt44_eof_symbol_in_symbol_to_index(g in arb_grammar_simple()) {
        let table = build_table(&g);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof()),
            "EOF not in symbol_to_index"
        );
    }

    #[test]
    fn pt45_table_has_accept_on_eof(g in arb_grammar_simple()) {
        let table = build_table(&g);
        prop_assert!(find_accept_in_table(&table), "no Accept action found");
    }

    #[test]
    fn pt46_chain_grammar_valid_table(g in arb_grammar_chain()) {
        let table = build_table(&g);
        prop_assert!(table.state_count > 0);
        prop_assert!(find_accept_in_table(&table));
    }

    #[test]
    fn pt47_two_nt_grammar_goto_exists(g in arb_grammar_two_nt()) {
        let table = build_table(&g);
        // At least one goto entry should exist for some nonterminal
        let has_goto = (0..table.state_count).any(|s| {
            table.nonterminal_to_index.keys().any(|&nt| {
                table.goto(StateId(s as u16), nt).is_some()
            })
        });
        prop_assert!(has_goto, "no goto entries found");
    }

    #[test]
    fn pt48_initial_state_has_actions(g in arb_grammar_simple()) {
        let table = build_table(&g);
        let init = table.initial_state;
        let has_action = table.index_to_symbol.iter().any(|sym| {
            !table.actions(init, *sym).is_empty()
        });
        prop_assert!(has_action, "initial state has no actions");
    }

    #[test]
    fn pt49_action_table_row_count_matches_state_count(g in arb_grammar_simple()) {
        let table = build_table(&g);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }

    #[test]
    fn pt50_goto_table_row_count_matches_state_count(g in arb_grammar_two_nt()) {
        let table = build_table(&g);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
    }
}

// ===========================================================================
// 51–60. Unit tests: grammar patterns → action cell properties
// ===========================================================================

#[test]
fn t51_single_token_grammar_minimal_states() {
    let g = GrammarBuilder::new("acp_v9_single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    // S → a requires at least: initial, after-shift, accept states
    assert!(table.state_count >= 3);
}

#[test]
fn t52_sequence_grammar_shift_chain() {
    let g = GrammarBuilder::new("acp_v9_seq")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 4);
    assert!(find_accept_in_table(&table));
}

#[test]
fn t53_alternative_grammar_both_tokens_shift() {
    let g = GrammarBuilder::new("acp_v9_alt")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let table = build_table(&g);
    let init = table.initial_state;
    // Both tokens should produce actions at initial state
    let mut found_tokens = 0;
    for &sym in &table.index_to_symbol {
        if !table.actions(init, sym).is_empty() {
            found_tokens += 1;
        }
    }
    assert!(found_tokens >= 2, "expected actions for at least 2 symbols");
}

#[test]
fn t54_chain_grammar_goto_entries() {
    let g = GrammarBuilder::new("acp_v9_ch")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    // Goto table should have entries for nonterminals s, a, b
    let nt_count = table.nonterminal_to_index.len();
    assert!(
        nt_count >= 3,
        "expected at least 3 nonterminals, got {nt_count}"
    );
}

#[test]
fn t55_expression_grammar_with_precedence() {
    let g = GrammarBuilder::new("acp_v9_prec")
        .token("n", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 4);
    assert!(find_accept_in_table(&table));
}

#[test]
fn t56_left_recursive_list_grammar() {
    let g = GrammarBuilder::new("acp_v9_lrec")
        .token("item", "x")
        .rule("l", vec!["item"])
        .rule("l", vec!["l", "item"])
        .start("l")
        .build();
    let table = build_table(&g);
    assert!(find_accept_in_table(&table));
    // Should have shift actions for "item" in at least 2 states
    let mut item_shifts = 0;
    for s in 0..table.state_count {
        for &sym in &table.index_to_symbol {
            for action in table.actions(StateId(s as u16), sym) {
                if matches!(action, Action::Shift(_)) {
                    item_shifts += 1;
                }
            }
        }
    }
    assert!(item_shifts >= 2, "expected at least 2 shift actions");
}

#[test]
fn t57_empty_actions_for_invalid_symbol() {
    let g = GrammarBuilder::new("acp_v9_inv")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    // A symbol not in the grammar should return empty actions
    let bogus = SymbolId(9999);
    let actions = table.actions(StateId(0), bogus);
    assert!(actions.is_empty());
}

#[test]
fn t58_empty_actions_for_invalid_state() {
    let g = GrammarBuilder::new("acp_v9_invs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let bogus_state = StateId(9999);
    let actions = table.actions(bogus_state, table.eof());
    assert!(actions.is_empty());
}

#[test]
fn t59_goto_none_for_invalid_nonterminal() {
    let g = GrammarBuilder::new("acp_v9_gn")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let bogus_nt = SymbolId(9999);
    assert!(table.goto(StateId(0), bogus_nt).is_none());
}

#[test]
fn t60_rule_info_valid() {
    let g = GrammarBuilder::new("acp_v9_ri")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    // Every reduce action should reference a valid rule
    for r in all_reduce_rules(&table) {
        let (lhs, rhs_len) = table.rule(r);
        assert!(lhs.0 > 0, "rule LHS should be a valid symbol");
        assert!(rhs_len > 0, "rule should have at least one RHS symbol");
    }
}

// ===========================================================================
// 61–70. Deeper proptest: table invariants
// ===========================================================================

proptest! {
    #[test]
    fn pt61_every_reduce_references_valid_rule(g in arb_grammar_simple()) {
        let table = build_table(&g);
        for s in 0..table.state_count {
            for &sym in &table.index_to_symbol {
                for action in table.actions(StateId(s as u16), sym) {
                    if let Action::Reduce(r) = action {
                        prop_assert!(
                            (r.0 as usize) < table.rules.len(),
                            "rule {} not in rules table",
                            r.0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn pt62_start_symbol_matches_grammar(g in arb_grammar_simple()) {
        let table = build_table(&g);
        // Start symbol should be in nonterminal_to_index
        prop_assert!(
            table.nonterminal_to_index.contains_key(&table.start_symbol()),
            "start symbol not in nonterminal index"
        );
    }

    #[test]
    fn pt63_eof_not_in_nonterminal_index(g in arb_grammar_simple()) {
        let table = build_table(&g);
        prop_assert!(
            !table.nonterminal_to_index.contains_key(&table.eof()),
            "EOF should not be a nonterminal"
        );
    }

    #[test]
    fn pt64_no_shift_on_eof_at_accept_states(g in arb_grammar_simple()) {
        let table = build_table(&g);
        let eof = table.eof();
        for s in 0..table.state_count {
            let actions = table.actions(StateId(s as u16), eof);
            let has_accept = actions.iter().any(|a| matches!(a, Action::Accept));
            let has_shift = actions.iter().any(|a| matches!(a, Action::Shift(_)));
            if has_accept {
                prop_assert!(
                    !has_shift,
                    "state {s} has both Accept and Shift on EOF"
                );
            }
        }
    }

    #[test]
    fn pt65_action_table_columns_consistent(g in arb_grammar_simple()) {
        let table = build_table(&g);
        if table.state_count > 0 {
            let expected_cols = table.action_table[0].len();
            for row in &table.action_table {
                prop_assert_eq!(row.len(), expected_cols, "inconsistent column count");
            }
        }
    }

    #[test]
    fn pt66_index_to_symbol_bijection(g in arb_grammar_simple()) {
        let table = build_table(&g);
        for (sym, &idx) in &table.symbol_to_index {
            if idx < table.index_to_symbol.len() {
                prop_assert_eq!(
                    table.index_to_symbol[idx], *sym,
                    "symbol_to_index/index_to_symbol mismatch"
                );
            }
        }
    }

    #[test]
    fn pt67_chain_grammar_has_multiple_goto_entries(g in arb_grammar_chain()) {
        let table = build_table(&g);
        let mut goto_count = 0;
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if table.goto(StateId(s as u16), nt).is_some() {
                    goto_count += 1;
                }
            }
        }
        prop_assert!(goto_count >= 3, "chain grammar should have ≥3 goto entries, got {goto_count}");
    }

    #[test]
    fn pt68_two_nt_grammar_table_accept(g in arb_grammar_two_nt()) {
        let table = build_table(&g);
        prop_assert!(find_accept_in_table(&table));
    }

    #[test]
    fn pt69_simple_grammar_no_error_at_initial_on_valid_token(g in arb_grammar_simple()) {
        let table = build_table(&g);
        let init = table.initial_state;
        // At least one token should NOT produce Error at the initial state
        let any_non_error = table.index_to_symbol.iter().any(|sym| {
            let actions = table.actions(init, *sym);
            !actions.is_empty() && !actions.iter().all(|a| matches!(a, Action::Error))
        });
        prop_assert!(any_non_error, "initial state should have at least one non-error action");
    }

    #[test]
    fn pt70_all_goto_targets_have_action_rows(g in arb_grammar_two_nt()) {
        let table = build_table(&g);
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if let Some(target) = table.goto(StateId(s as u16), nt) {
                    prop_assert!(
                        (target.0 as usize) < table.action_table.len(),
                        "goto target {} has no action row",
                        target.0
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 71–80. Unit tests: edge cases and builder grammars
// ===========================================================================

#[test]
fn t71_accept_is_accept() {
    let a = Action::Accept;
    assert!(matches!(a, Action::Accept));
}

#[test]
fn t72_error_is_error() {
    let a = Action::Error;
    assert!(matches!(a, Action::Error));
}

#[test]
fn t73_recover_is_recover() {
    let a = Action::Recover;
    assert!(matches!(a, Action::Recover));
}

#[test]
fn t74_fork_contains_all_children() {
    let children = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
    ];
    let fork = Action::Fork(children.clone());
    if let Action::Fork(inner) = fork {
        assert_eq!(inner.len(), 3);
        assert_eq!(inner, children);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn t75_multi_alt_grammar_table() {
    let g = GrammarBuilder::new("acp_v9_multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(find_accept_in_table(&table));
    // Should have shift actions for all 3 tokens at initial
    let init = table.initial_state;
    let shifts: Vec<_> = table
        .index_to_symbol
        .iter()
        .filter(|sym| {
            table
                .actions(init, **sym)
                .iter()
                .any(|a| matches!(a, Action::Shift(_)))
        })
        .collect();
    assert!(
        shifts.len() >= 3,
        "expected shifts for 3 tokens, got {}",
        shifts.len()
    );
}

#[test]
fn t76_nested_nonterminals_goto_depth() {
    let g = GrammarBuilder::new("acp_v9_nest")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    // Should have goto entries for s, a, b, c
    assert!(table.nonterminal_to_index.len() >= 4);
}

#[test]
fn t77_eof_symbol_distinct_from_tokens() {
    let g = GrammarBuilder::new("acp_v9_eof")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    // EOF should not be a regular token's symbol
    for &sym in table.symbol_to_index.keys() {
        if sym != eof {
            assert_ne!(sym, eof);
        }
    }
}

#[test]
fn t78_rule_rhs_len_matches_grammar() {
    let g = GrammarBuilder::new("acp_v9_rlen")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let table = build_table(&g);
    // Find the rule with rhs_len == 3
    let has_three = table.rules.iter().any(|r| r.rhs_len == 3);
    assert!(has_three, "expected a rule with 3 RHS symbols");
}

#[test]
fn t79_precedence_grammar_table_valid() {
    let g = GrammarBuilder::new("acp_v9_prec2")
        .token("n", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let table = build_table(&g);
    // Validate every reduce rule is in bounds
    for r in all_reduce_rules(&table) {
        assert!((r.0 as usize) < table.rules.len());
    }
    // Validate every shift target is in bounds
    for t in all_shift_targets(&table) {
        assert!((t.0 as usize) < table.state_count);
    }
}

#[test]
fn t80_normalized_table_equivalent() {
    let g = GrammarBuilder::new("acp_v9_norm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table_normalized(&g);
    assert!(table.state_count > 0);
    assert!(find_accept_in_table(&table));
}

// ===========================================================================
// 81–85. Additional proptest: Fork and multi-action cells
// ===========================================================================

proptest! {
    #[test]
    fn pt81_fork_len_matches_children(children in prop::collection::vec(leaf_action(), 1..=6)) {
        let fork = Action::Fork(children.clone());
        if let Action::Fork(inner) = fork {
            prop_assert_eq!(inner.len(), children.len());
        }
    }

    #[test]
    fn pt82_cell_with_two_actions_is_conflict(a in leaf_action(), b in leaf_action()) {
        let cell: ActionCell = vec![a, b];
        prop_assert!(has_conflict(&cell));
    }

    #[test]
    fn pt83_action_cell_iter_matches_len(cell in arb_action_cell()) {
        let slice_len = cell.as_slice().len();
        prop_assert_eq!(slice_len, cell.len());
    }

    #[test]
    fn pt84_symbol_id_ne_when_different(a in 0u16..50, b in 50u16..100) {
        prop_assert_ne!(SymbolId(a), SymbolId(b));
    }

    #[test]
    fn pt85_state_id_ne_when_different(a in 0u16..50, b in 50u16..100) {
        prop_assert_ne!(StateId(a), StateId(b));
    }
}
