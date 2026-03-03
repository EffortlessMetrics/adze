#![allow(clippy::needless_range_loop)]
//! Property-based tests for state management in adze-glr-core.
//!
//! Tests StateId creation, comparison, ordering, hashing, and how states
//! appear in parse tables (state counts, transitions, initial state, reachability).
//!
//! Run with: `cargo test -p adze-glr-core --test state_proptest`

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

type ActionCell = Vec<Action>;

const NO_GOTO: StateId = StateId(u16::MAX);

fn leaf_action(max_state: u16) -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..max_state).prop_map(|s| Action::Shift(StateId(s))),
        (0..16u16).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

fn arb_action_cell(max_state: u16) -> impl Strategy<Value = ActionCell> {
    prop::collection::vec(leaf_action(max_state), 0..=3)
}

/// Build a well-formed `ParseTable` with given dimensions.
fn build_table(
    num_states: usize,
    num_terminals: usize,
    num_nonterminals: usize,
    action_table: Vec<Vec<ActionCell>>,
    goto_table: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
) -> ParseTable {
    let sym_count = num_terminals + num_nonterminals;

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..sym_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for i in num_terminals..sym_count {
        nonterminal_to_index.insert(SymbolId(i as u16), i - num_terminals);
    }

    let metadata = (0..sym_count as u16)
        .map(|i| adze_glr_core::SymbolMetadata {
            name: format!("s{i}"),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: (i as usize) < num_terminals,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(i),
        })
        .collect();

    ParseTable {
        action_table,
        goto_table,
        rules: rules.clone(),
        state_count: num_states,
        symbol_count: sym_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(num_terminals as u16),
        grammar: Grammar::new("state_proptest".to_string()),
        symbol_metadata: metadata,
        initial_state: StateId(0),
        token_count: num_terminals,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            num_states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; rules.len()],
        rule_assoc_by_rule: vec![0; rules.len()],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
    }
}

/// Strategy that generates a consistent `ParseTable`.
fn arb_parse_table() -> impl Strategy<Value = ParseTable> {
    (1usize..=5, 1usize..=4, 1usize..=6)
        .prop_flat_map(|(num_t, num_nt, num_s)| {
            let sym_count = num_t + num_nt;
            let actions = prop::collection::vec(
                prop::collection::vec(arb_action_cell(num_s as u16), sym_count..=sym_count),
                num_s..=num_s,
            );
            let gotos = prop::collection::vec(
                prop::collection::vec(
                    prop_oneof![Just(NO_GOTO), (0..num_s as u16).prop_map(StateId)],
                    num_nt..=num_nt,
                ),
                num_s..=num_s,
            );
            let rules = prop::collection::vec(
                (
                    (num_t as u16..(num_t + num_nt) as u16).prop_map(SymbolId),
                    0u16..=4,
                )
                    .prop_map(|(lhs, rhs_len)| ParseRule { lhs, rhs_len }),
                0..=4,
            );
            (
                Just(num_s),
                Just(num_t),
                Just(num_nt),
                actions,
                gotos,
                rules,
            )
        })
        .prop_map(|(ns, nt, nnt, a, g, r)| build_table(ns, nt, nnt, a, g, r))
}

// ===========================================================================
// 1–8: StateId creation, comparison, ordering, hashing, clone, debug
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// 1. StateId round-trips through its inner value.
    #[test]
    fn state_id_roundtrip(val in any::<u16>()) {
        let id = StateId(val);
        prop_assert_eq!(id.0, val);
    }

    /// 2. StateId equality is reflexive.
    #[test]
    fn state_id_eq_reflexive(val in any::<u16>()) {
        let id = StateId(val);
        prop_assert_eq!(id, id);
    }

    /// 3. StateId equality matches when inner values match.
    #[test]
    fn state_id_eq_iff_same_inner(a in any::<u16>(), b in any::<u16>()) {
        let sa = StateId(a);
        let sb = StateId(b);
        prop_assert_eq!(sa == sb, a == b);
    }

    /// 4. StateId ordering is consistent with u16 ordering.
    #[test]
    fn state_id_ord_consistent(a in any::<u16>(), b in any::<u16>()) {
        let sa = StateId(a);
        let sb = StateId(b);
        prop_assert_eq!(sa.cmp(&sb), a.cmp(&b));
    }

    /// 5. StateId partial ordering agrees with total ordering.
    #[test]
    fn state_id_partial_ord_agrees(a in any::<u16>(), b in any::<u16>()) {
        let sa = StateId(a);
        let sb = StateId(b);
        prop_assert_eq!(sa.partial_cmp(&sb), Some(sa.cmp(&sb)));
    }

    /// 6. StateId hashing: equal values produce equal hashes.
    #[test]
    fn state_id_hash_eq(val in any::<u16>()) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let s1 = StateId(val);
        let s2 = StateId(val);
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        s1.hash(&mut h1);
        s2.hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }

    /// 7. StateId clone produces an equal value.
    #[test]
    fn state_id_clone(val in any::<u16>()) {
        let id = StateId(val);
        #[allow(clippy::clone_on_copy)]
        let cloned = id.clone();
        prop_assert_eq!(id, cloned);
    }

    /// 8. StateId debug format contains the inner value.
    #[test]
    fn state_id_debug_contains_value(val in 0u16..10000) {
        let id = StateId(val);
        let dbg = format!("{:?}", id);
        prop_assert!(
            dbg.contains(&val.to_string()),
            "Debug output {:?} should contain {}", dbg, val
        );
    }
}

// ===========================================================================
// 9–16: StateId in collections and ordering properties
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// 9. StateId can be used as HashMap key; lookup works.
    #[test]
    fn state_id_hashmap_key(vals in prop::collection::vec(any::<u16>(), 1..20)) {
        let mut map = HashMap::new();
        for &v in &vals {
            map.insert(StateId(v), v);
        }
        for &v in &vals {
            prop_assert_eq!(map.get(&StateId(v)), Some(&v));
        }
    }

    /// 10. StateId can be inserted into HashSet; deduplication works.
    #[test]
    fn state_id_hashset_dedup(vals in prop::collection::vec(0u16..50, 1..30)) {
        let set: HashSet<StateId> = vals.iter().map(|&v| StateId(v)).collect();
        let unique: HashSet<u16> = vals.iter().copied().collect();
        prop_assert_eq!(set.len(), unique.len());
    }

    /// 11. StateId in BTreeSet maintains sorted order.
    #[test]
    fn state_id_btreeset_sorted(vals in prop::collection::vec(any::<u16>(), 1..20)) {
        let set: BTreeSet<StateId> = vals.iter().map(|&v| StateId(v)).collect();
        let collected: Vec<StateId> = set.iter().copied().collect();
        for i in 1..collected.len() {
            prop_assert!(collected[i - 1] < collected[i]);
        }
    }

    /// 12. Sorting a vec of StateIds produces the same order as sorting their inner values.
    #[test]
    fn state_id_sort_matches_u16(vals in prop::collection::vec(any::<u16>(), 0..30)) {
        let mut state_ids: Vec<StateId> = vals.iter().map(|&v| StateId(v)).collect();
        let mut raw: Vec<u16> = vals.clone();
        state_ids.sort();
        raw.sort();
        for i in 0..state_ids.len() {
            prop_assert_eq!(state_ids[i].0, raw[i]);
        }
    }

    /// 13. StateId(0) is the minimum possible StateId.
    #[test]
    fn state_id_zero_is_minimum(val in 1u16..=u16::MAX) {
        prop_assert!(StateId(0) < StateId(val));
    }

    /// 14. StateId display format contains the inner value.
    #[test]
    fn state_id_display(val in 0u16..10000) {
        let id = StateId(val);
        let disp = format!("{}", id);
        prop_assert!(
            disp.contains(&val.to_string()),
            "Display {:?} should contain {}", disp, val
        );
    }

    /// 15. StateId copy semantics: original unchanged after copy.
    #[test]
    fn state_id_copy_semantics(val in any::<u16>()) {
        let original = StateId(val);
        let copied = original; // Copy
        prop_assert_eq!(original, copied);
        prop_assert_eq!(original.0, val);
    }

    /// 16. Transitivity: if a < b and b < c then a < c.
    #[test]
    fn state_id_ord_transitive(a in 0u16..100, b in 100u16..200, c in 200u16..300) {
        let sa = StateId(a);
        let sb = StateId(b);
        let sc = StateId(c);
        prop_assert!(sa < sb);
        prop_assert!(sb < sc);
        prop_assert!(sa < sc);
    }
}

// ===========================================================================
// 17–25: State management in parse tables
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// 17. state_count in parse table matches action_table row count.
    #[test]
    fn table_state_count_matches_rows(pt in arb_parse_table()) {
        prop_assert_eq!(pt.state_count, pt.action_table.len());
        prop_assert_eq!(pt.state_count, pt.goto_table.len());
    }

    /// 18. initial_state is always StateId(0) in our constructed tables.
    #[test]
    fn initial_state_is_zero(pt in arb_parse_table()) {
        prop_assert_eq!(pt.initial_state, StateId(0));
    }

    /// 19. initial_state is within valid state range.
    #[test]
    fn initial_state_within_bounds(pt in arb_parse_table()) {
        prop_assert!((pt.initial_state.0 as usize) < pt.state_count);
    }

    /// 20. Action table lookups at initial state return a valid slice (not panic).
    #[test]
    fn initial_state_actions_safe(pt in arb_parse_table()) {
        for (&sym, _) in &pt.symbol_to_index {
            let _actions = pt.actions(pt.initial_state, sym);
            // no panic is the assertion
        }
    }

    /// 21. Shift actions reference valid state indices.
    #[test]
    fn shift_targets_within_bounds(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.action_table[s].len() {
                for action in &pt.action_table[s][col] {
                    if let Action::Shift(target) = action {
                        prop_assert!(
                            (target.0 as usize) < pt.state_count,
                            "shift target {:?} >= state_count {}", target, pt.state_count
                        );
                    }
                }
            }
        }
    }

    /// 22. States reachable from initial via shift are within bounds.
    #[test]
    fn reachable_states_within_bounds(pt in arb_parse_table()) {
        let mut visited = HashSet::new();
        let mut worklist = vec![pt.initial_state];
        while let Some(state) = worklist.pop() {
            if !visited.insert(state) {
                continue;
            }
            prop_assert!((state.0 as usize) < pt.state_count);
            let s = state.0 as usize;
            // Collect shift targets
            for col in 0..pt.action_table[s].len() {
                for action in &pt.action_table[s][col] {
                    if let Action::Shift(target) = action {
                        worklist.push(*target);
                    }
                }
            }
            // Collect goto targets
            for col in 0..pt.goto_table[s].len() {
                let target = pt.goto_table[s][col];
                if target != NO_GOTO {
                    worklist.push(target);
                }
            }
        }
        // All visited states are within bounds (checked in loop)
    }

    /// 23. goto() returns None for the initial state with unknown nonterminals.
    #[test]
    fn goto_initial_unknown_nt_none(pt in arb_parse_table()) {
        let unknown = SymbolId(pt.symbol_count as u16 + 50);
        prop_assert!(pt.goto(pt.initial_state, unknown).is_none());
    }

    /// 24. Cloned parse table preserves state_count and initial_state.
    #[test]
    fn clone_preserves_state_info(pt in arb_parse_table()) {
        let cloned = pt.clone();
        prop_assert_eq!(pt.state_count, cloned.state_count);
        prop_assert_eq!(pt.initial_state, cloned.initial_state);
        prop_assert_eq!(pt.eof_symbol, cloned.eof_symbol);
    }

    /// 25. All states have action rows of uniform width (symbol_count).
    #[test]
    fn all_states_uniform_action_width(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            prop_assert_eq!(
                pt.action_table[s].len(),
                pt.symbol_count,
                "state {} action row width {} != symbol_count {}",
                s,
                pt.action_table[s].len(),
                pt.symbol_count
            );
        }
    }
}

// ===========================================================================
// 26–30: Additional state properties
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// 26. Every valid state has a lex_mode entry.
    #[test]
    fn every_state_has_lex_mode(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let _mode = pt.lex_mode(StateId(s as u16));
            // no panic is the assertion
        }
    }

    /// 27. valid_symbols for each state has correct length.
    #[test]
    fn valid_symbols_per_state(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            prop_assert_eq!(vs.len(), pt.terminal_boundary());
        }
    }

    /// 28. Out-of-bound state actions always return empty.
    #[test]
    fn oob_state_actions_empty(pt in arb_parse_table(), offset in 1u16..100) {
        let oob = StateId(pt.state_count as u16 + offset);
        for (&sym, _) in &pt.symbol_to_index {
            prop_assert!(pt.actions(oob, sym).is_empty());
        }
    }

    /// 29. Out-of-bound state goto always returns None.
    #[test]
    fn oob_state_goto_none(pt in arb_parse_table(), offset in 1u16..100) {
        let oob = StateId(pt.state_count as u16 + offset);
        for &nt in pt.nonterminal_to_index.keys() {
            prop_assert!(pt.goto(oob, nt).is_none());
        }
    }

    /// 30. Each state index [0..state_count) can be queried without panic.
    #[test]
    fn all_states_queryable(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let sid = StateId(s as u16);
            // actions for every mapped terminal
            for (&sym, _) in &pt.symbol_to_index {
                let _ = pt.actions(sid, sym);
            }
            // goto for every mapped nonterminal
            for &nt in pt.nonterminal_to_index.keys() {
                let _ = pt.goto(sid, nt);
            }
        }
    }
}
