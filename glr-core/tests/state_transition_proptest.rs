#![allow(clippy::needless_range_loop)]
//! Property-based tests for state transitions in adze-glr-core.
//!
//! Tests that ACTION and GOTO table lookups behave correctly under random
//! state/symbol inputs: valid transitions land in valid states, invalid
//! lookups return empty/None, and determinism is preserved.
//!
//! Run with: `cargo test -p adze-glr-core --test state_transition_proptest`

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Constants & helpers
// ---------------------------------------------------------------------------

type ActionCell = Vec<Action>;

/// Sentinel for "no GOTO transition".
const NO_GOTO: StateId = StateId(65535);

/// Generate a leaf `Action` (no nested `Fork`).
fn leaf_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..64u16).prop_map(|s| Action::Shift(StateId(s))),
        (0..32u16).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Generate an `ActionCell` with 0–4 actions.
fn arb_action_cell() -> impl Strategy<Value = ActionCell> {
    prop::collection::vec(leaf_action(), 0..=4)
}

/// Generate a `SymbolMetadata`.
fn arb_symbol_metadata(id: u16) -> impl Strategy<Value = adze_glr_core::SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(move |(vis, named, term)| {
        adze_glr_core::SymbolMetadata {
            name: format!("sym_{id}"),
            is_visible: vis,
            is_named: named,
            is_supertype: false,
            is_terminal: term,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(id),
        }
    })
}

/// Build a well-formed `ParseTable` from generated dimensions and content.
fn build_table(
    num_states: usize,
    num_terminals: usize,
    num_nonterminals: usize,
    action_table: Vec<Vec<ActionCell>>,
    goto_table: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    metadata: Vec<adze_glr_core::SymbolMetadata>,
) -> ParseTable {
    let symbol_count = num_terminals + num_nonterminals;

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for i in num_terminals..symbol_count {
        nonterminal_to_index.insert(SymbolId(i as u16), i - num_terminals);
    }

    ParseTable {
        action_table,
        goto_table,
        rules: rules.clone(),
        state_count: num_states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(num_terminals as u16),
        grammar: Grammar::new("proptest_st".to_string()),
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
    (1usize..=6, 1usize..=4, 1usize..=8)
        .prop_flat_map(|(num_t, num_nt, num_s)| {
            let sym_count = num_t + num_nt;
            let actions = prop::collection::vec(
                prop::collection::vec(arb_action_cell(), sym_count..=sym_count),
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
                    (num_t as u16..sym_count as u16).prop_map(SymbolId),
                    0u16..=4,
                )
                    .prop_map(|(lhs, rhs_len)| ParseRule { lhs, rhs_len }),
                0..=6,
            );
            let metadata = (0..sym_count as u16)
                .map(arb_symbol_metadata)
                .collect::<Vec<_>>();
            (
                Just(num_s),
                Just(num_t),
                Just(num_nt),
                actions,
                gotos,
                rules,
                metadata,
            )
        })
        .prop_map(|(ns, nt, nnt, a, g, r, m)| build_table(ns, nt, nnt, a, g, r, m))
}

// ===========================================================================
// 1–10: Core transition validity
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 1. Shift targets reference states within bounds
    #[test]
    fn shift_targets_within_state_count(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.action_table[s].len() {
                for action in &pt.action_table[s][col] {
                    if let Action::Shift(target) = action {
                        prop_assert!(
                            (target.0 as usize) < 64,
                            "shift target {:?} generated out of strategy range", target
                        );
                    }
                }
            }
        }
    }

    // 2. Reduce rule IDs are within strategy range
    #[test]
    fn reduce_rule_ids_bounded(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.action_table[s].len() {
                for action in &pt.action_table[s][col] {
                    if let Action::Reduce(rid) = action {
                        prop_assert!(
                            rid.0 < 32,
                            "reduce rule {:?} out of strategy range", rid
                        );
                    }
                }
            }
        }
    }

    // 3. GOTO targets that are not NO_GOTO are within state_count
    #[test]
    fn goto_targets_valid_or_sentinel(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.goto_table[s].len() {
                let target = pt.goto_table[s][col];
                prop_assert!(
                    target == NO_GOTO || (target.0 as usize) < pt.state_count,
                    "goto[{}][{}] = {:?} is neither sentinel nor valid state (count={})",
                    s, col, target, pt.state_count
                );
            }
        }
    }

    // 4. actions() on an out-of-range state returns empty
    #[test]
    fn actions_oob_state_returns_empty(pt in arb_parse_table(), extra in 0u16..100) {
        let oob = StateId(pt.state_count as u16 + extra);
        for &sym in pt.symbol_to_index.keys() {
            let actions = pt.actions(oob, sym);
            prop_assert!(actions.is_empty(), "OOB state {:?} should yield empty actions", oob);
        }
    }

    // 5. actions() on an unmapped symbol returns empty
    #[test]
    fn actions_unmapped_symbol_returns_empty(pt in arb_parse_table()) {
        let unmapped = SymbolId(pt.symbol_count as u16 + 100);
        for s in 0..pt.state_count {
            let actions = pt.actions(StateId(s as u16), unmapped);
            prop_assert!(actions.is_empty(), "unmapped symbol should yield empty actions");
        }
    }

    // 6. goto() on an out-of-range state returns None
    #[test]
    fn goto_oob_state_returns_none(pt in arb_parse_table(), extra in 0u16..100) {
        let oob = StateId(pt.state_count as u16 + extra);
        for &nt in pt.nonterminal_to_index.keys() {
            prop_assert!(pt.goto(oob, nt).is_none(), "OOB state {:?} goto should be None", oob);
        }
    }

    // 7. goto() on an unmapped nonterminal returns None
    #[test]
    fn goto_unmapped_nonterminal_returns_none(pt in arb_parse_table()) {
        let unmapped = SymbolId(pt.symbol_count as u16 + 200);
        for s in 0..pt.state_count {
            prop_assert!(pt.goto(StateId(s as u16), unmapped).is_none());
        }
    }

    // 8. Determinism: actions() called twice yields identical results
    #[test]
    fn actions_deterministic(pt in arb_parse_table(), s_idx in 0u16..8, sym_id in 0u16..10) {
        let state = StateId(s_idx);
        let sym = SymbolId(sym_id);
        let a1 = pt.actions(state, sym);
        let a2 = pt.actions(state, sym);
        prop_assert_eq!(a1, a2, "actions() must be deterministic");
    }

    // 9. Determinism: goto() called twice yields identical results
    #[test]
    fn goto_deterministic(pt in arb_parse_table(), s_idx in 0u16..8, sym_id in 0u16..10) {
        let state = StateId(s_idx);
        let nt = SymbolId(sym_id);
        let g1 = pt.goto(state, nt);
        let g2 = pt.goto(state, nt);
        prop_assert_eq!(g1, g2, "goto() must be deterministic");
    }

    // 10. state_count matches action_table row count
    #[test]
    fn state_count_matches_action_rows(pt in arb_parse_table()) {
        prop_assert_eq!(pt.state_count, pt.action_table.len());
    }
}

// ===========================================================================
// 11–20: Consistency and symmetry
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 11. state_count matches goto_table row count
    #[test]
    fn state_count_matches_goto_rows(pt in arb_parse_table()) {
        prop_assert_eq!(pt.state_count, pt.goto_table.len());
    }

    // 12. symbol_to_index and index_to_symbol are inverses
    #[test]
    fn symbol_index_roundtrip(pt in arb_parse_table()) {
        for (&sym, &idx) in &pt.symbol_to_index {
            if idx < pt.index_to_symbol.len() {
                prop_assert_eq!(
                    pt.index_to_symbol[idx], sym,
                    "index_to_symbol[{}] should be {:?}", idx, sym
                );
            }
        }
    }

    // 13. All action table rows have the same width (symbol_count)
    #[test]
    fn action_table_rows_uniform_width(pt in arb_parse_table()) {
        for (s, row) in pt.action_table.iter().enumerate() {
            prop_assert_eq!(
                row.len(), pt.symbol_count,
                "action_table[{}].len()={} != symbol_count={}", s, row.len(), pt.symbol_count
            );
        }
    }

    // 14. All goto table rows have the same width (num nonterminals)
    #[test]
    fn goto_table_rows_uniform_width(pt in arb_parse_table()) {
        let expected = pt.nonterminal_to_index.len();
        for (s, row) in pt.goto_table.iter().enumerate() {
            prop_assert_eq!(
                row.len(), expected,
                "goto_table[{}].len()={} != expected {}", s, row.len(), expected
            );
        }
    }

    // 15. Action cells contain only leaf actions (no nested Fork in our generation)
    #[test]
    fn no_nested_forks_in_cells(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.action_table[s].len() {
                for action in &pt.action_table[s][col] {
                    prop_assert!(
                        !matches!(action, Action::Fork(_)),
                        "leaf_action strategy should never produce Fork"
                    );
                }
            }
        }
    }

    // 16. valid_symbols() length equals terminal_boundary
    #[test]
    fn valid_symbols_length(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            prop_assert_eq!(vs.len(), pt.terminal_boundary());
        }
    }

    // 17. valid_symbols() true iff action cell is non-empty for that column
    #[test]
    fn valid_symbols_matches_nonempty_cells(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            let boundary = pt.terminal_boundary();
            for t in 0..boundary.min(pt.action_table[s].len()) {
                let cell_nonempty = !pt.action_table[s][t].is_empty();
                prop_assert_eq!(
                    vs[t], cell_nonempty,
                    "valid_symbols mismatch at state {} col {}", s, t
                );
            }
        }
    }

    // 18. goto() returns None for sentinel entries
    #[test]
    fn goto_sentinel_yields_none(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&nt, &col) in &pt.nonterminal_to_index {
                if col < pt.goto_table[s].len() && pt.goto_table[s][col] == NO_GOTO {
                    prop_assert!(
                        pt.goto(StateId(s as u16), nt).is_none(),
                        "sentinel in goto[{}][{}] should yield None", s, col
                    );
                }
            }
        }
    }

    // 19. goto() returns Some for non-sentinel entries
    #[test]
    fn goto_non_sentinel_yields_some(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&nt, &col) in &pt.nonterminal_to_index {
                if col < pt.goto_table[s].len() && pt.goto_table[s][col] != NO_GOTO {
                    let result = pt.goto(StateId(s as u16), nt);
                    prop_assert!(
                        result.is_some(),
                        "non-sentinel goto[{}][{}] should yield Some", s, col
                    );
                    prop_assert_eq!(result.unwrap(), pt.goto_table[s][col]);
                }
            }
        }
    }

    // 20. actions() via symbol_to_index matches direct table access
    #[test]
    fn actions_matches_direct_table_access(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&sym, &col) in &pt.symbol_to_index {
                let via_api = pt.actions(StateId(s as u16), sym);
                let direct = &pt.action_table[s][col];
                prop_assert_eq!(
                    via_api, direct.as_slice(),
                    "actions API vs direct table mismatch at state {} sym {:?}", s, sym
                );
            }
        }
    }
}

// ===========================================================================
// 21–30: Bounds, cloning, and structural invariants
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 21. Cloned table has identical transition results
    #[test]
    fn cloned_table_identical_transitions(pt in arb_parse_table(), s_idx in 0u16..8, sym_id in 0u16..10) {
        let cloned = pt.clone();
        let state = StateId(s_idx);
        let sym = SymbolId(sym_id);
        prop_assert_eq!(pt.actions(state, sym), cloned.actions(state, sym));
        prop_assert_eq!(pt.goto(state, sym), cloned.goto(state, sym));
    }

    // 22. initial_state is within state_count
    #[test]
    fn initial_state_within_bounds(pt in arb_parse_table()) {
        prop_assert!((pt.initial_state.0 as usize) < pt.state_count);
    }

    // 23. eof_symbol is mapped in symbol_to_index
    #[test]
    fn eof_in_symbol_to_index(pt in arb_parse_table()) {
        prop_assert!(
            pt.symbol_to_index.contains_key(&pt.eof_symbol),
            "eof {:?} missing from symbol_to_index", pt.eof_symbol
        );
    }

    // 24. start_symbol is mapped in nonterminal_to_index
    #[test]
    fn start_in_nonterminal_to_index(pt in arb_parse_table()) {
        prop_assert!(
            pt.nonterminal_to_index.contains_key(&pt.start_symbol),
            "start {:?} missing from nonterminal_to_index", pt.start_symbol
        );
    }

    // 25. lex_modes length equals state_count
    #[test]
    fn lex_modes_match_state_count(pt in arb_parse_table()) {
        prop_assert_eq!(pt.lex_modes.len(), pt.state_count);
    }

    // 26. terminal_boundary equals token_count (no external tokens in our tables)
    #[test]
    fn terminal_boundary_is_token_count(pt in arb_parse_table()) {
        prop_assert_eq!(pt.terminal_boundary(), pt.token_count + pt.external_token_count);
    }

    // 27. is_terminal true for ids below terminal_boundary
    #[test]
    fn is_terminal_below_boundary(pt in arb_parse_table(), id in 0u16..20) {
        let sym = SymbolId(id);
        let expected = (id as usize) < pt.terminal_boundary();
        prop_assert_eq!(pt.is_terminal(sym), expected);
    }

    // 28. nonterminal_to_index keys are at or above terminal boundary
    #[test]
    fn nonterminal_keys_above_terminal_boundary(pt in arb_parse_table()) {
        let boundary = pt.terminal_boundary();
        for &sym in pt.nonterminal_to_index.keys() {
            prop_assert!(
                (sym.0 as usize) >= boundary,
                "nonterminal {:?} below terminal boundary {}", sym, boundary
            );
        }
    }

    // 29. action_table cell count equals state_count × symbol_count
    #[test]
    fn total_action_cells(pt in arb_parse_table()) {
        let total: usize = pt.action_table.iter().map(|r| r.len()).sum();
        prop_assert_eq!(total, pt.state_count * pt.symbol_count);
    }

    // 30. symbol_to_index values are unique
    #[test]
    fn symbol_to_index_values_unique(pt in arb_parse_table()) {
        let vals: Vec<usize> = pt.symbol_to_index.values().copied().collect();
        let mut sorted = vals.clone();
        sorted.sort();
        sorted.dedup();
        prop_assert_eq!(vals.len(), sorted.len(), "symbol_to_index has duplicate column indices");
    }
}

// ===========================================================================
// 31–35: Edge cases and advanced properties
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 31. valid_symbols_mask equals valid_symbols
    #[test]
    fn valid_symbols_mask_matches(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let sid = StateId(s as u16);
            let vs = pt.valid_symbols(sid);
            let mask = pt.valid_symbols_mask(sid);
            prop_assert_eq!(vs, mask, "valid_symbols vs valid_symbols_mask mismatch at state {}", s);
        }
    }

    // 32. lex_mode for OOB state returns default LexMode
    #[test]
    fn lex_mode_oob_returns_default(pt in arb_parse_table(), extra in 1u16..50) {
        let oob = StateId(pt.state_count as u16 + extra);
        let mode = pt.lex_mode(oob);
        prop_assert_eq!(mode.lex_state, 0);
        prop_assert_eq!(mode.external_lex_state, 0);
    }

    // 33. lex_mode for valid state returns stored value
    #[test]
    fn lex_mode_valid_state(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let mode = pt.lex_mode(StateId(s as u16));
            prop_assert_eq!(mode, pt.lex_modes[s]);
        }
    }

    // 34. with_detected_goto_indexing is idempotent
    #[test]
    fn goto_detection_idempotent(pt in arb_parse_table()) {
        let once = pt.clone().with_detected_goto_indexing();
        let twice = once.clone().with_detected_goto_indexing();
        prop_assert_eq!(once.goto_indexing, twice.goto_indexing);
    }

    // 35. A table with a single Accept action has it reachable via actions()
    #[test]
    fn single_accept_reachable(
        num_t in 1usize..=4,
        num_nt in 1usize..=3,
        target_state in 0usize..4,
        target_col in 0usize..4,
    ) {
        let num_s = (target_state + 1).max(1);
        let sym_count = num_t + num_nt;
        let col = target_col % sym_count;

        let mut action_table = vec![vec![vec![]; sym_count]; num_s];
        action_table[target_state % num_s][col] = vec![Action::Accept];

        let goto_table = vec![vec![NO_GOTO; num_nt]; num_s];
        let metadata = (0..sym_count as u16)
            .map(|i| adze_glr_core::SymbolMetadata {
                name: format!("s{i}"),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: (i as usize) < num_t,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(i),
            })
            .collect();

        let pt = build_table(num_s, num_t, num_nt, action_table, goto_table, vec![], metadata);
        let state = StateId((target_state % num_s) as u16);
        let sym = SymbolId(col as u16);
        let actions = pt.actions(state, sym);
        prop_assert!(
            actions.iter().any(|a| matches!(a, Action::Accept)),
            "Accept should be reachable at state {:?} sym {:?}", state, sym
        );
    }
}
