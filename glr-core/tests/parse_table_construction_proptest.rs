#![allow(clippy::needless_range_loop)]
//! Property-based tests for `ParseTable` **construction** paths and invariants.
//!
//! These tests focus on building `ParseTable` values from parts, verifying
//! dimensional consistency, symbol-mapping completeness, action-variant
//! handling, cloning fidelity, and transformation round-trips.
//!
//! Run with: `cargo test -p adze-glr-core --test parse_table_construction_proptest`

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

type ActionCell = Vec<Action>;
const SENTINEL: StateId = StateId(u16::MAX);

/// Generate a leaf `Action` (no `Fork`).
fn leaf_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..50u16).prop_map(|s| Action::Shift(StateId(s))),
        (0..50u16).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Generate a `Fork` action containing 2–4 leaf actions.
fn fork_action() -> impl Strategy<Value = Action> {
    prop::collection::vec(leaf_action(), 2..=4).prop_map(Action::Fork)
}

/// Generate any `Action` including `Fork`.
fn any_action() -> impl Strategy<Value = Action> {
    prop_oneof![9 => leaf_action(), 1 => fork_action()]
}

/// Generate an `ActionCell` with 0–4 actions.
fn arb_action_cell() -> impl Strategy<Value = ActionCell> {
    prop::collection::vec(any_action(), 0..=4)
}

fn arb_metadata(id: u16) -> impl Strategy<Value = SymbolMetadata> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(move |(vis, named, sup, term, extra, fragile)| SymbolMetadata {
            name: format!("sym_{id}"),
            is_visible: vis,
            is_named: named,
            is_supertype: sup,
            is_terminal: term,
            is_extra: extra,
            is_fragile: fragile,
            symbol_id: SymbolId(id),
        })
}

/// Construct a well-formed `ParseTable` from dimensions and random contents.
fn make_table(
    num_states: usize,
    num_terminals: usize,
    num_nonterminals: usize,
    action_table: Vec<Vec<ActionCell>>,
    goto_table: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    metadata: Vec<SymbolMetadata>,
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
        grammar: Grammar::new("construction_proptest".to_string()),
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

/// Strategy that yields a consistent `ParseTable`.
fn arb_table() -> impl Strategy<Value = ParseTable> {
    (1usize..=6, 1usize..=4, 1usize..=8).prop_flat_map(|(nt, nnt, ns)| {
        let sc = nt + nnt;
        let actions = prop::collection::vec(
            prop::collection::vec(arb_action_cell(), sc..=sc),
            ns..=ns,
        );
        let gotos = prop::collection::vec(
            prop::collection::vec(
                prop_oneof![Just(SENTINEL), (0..ns as u16).prop_map(StateId)],
                nnt..=nnt,
            ),
            ns..=ns,
        );
        let rules = prop::collection::vec(
            ((nt as u16..sc as u16).prop_map(SymbolId), 0u16..=5)
                .prop_map(|(lhs, rhs_len)| ParseRule { lhs, rhs_len }),
            0..=6,
        );
        let meta = (0..sc as u16).map(arb_metadata).collect::<Vec<_>>();
        (Just(ns), Just(nt), Just(nnt), actions, gotos, rules, meta)
            .prop_map(|(ns, nt, nnt, a, g, r, m)| make_table(ns, nt, nnt, a, g, r, m))
    })
}

// ===========================================================================
// Property tests — construction invariants
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // -----------------------------------------------------------------------
    // 1. Constructed table has correct state_count
    // -----------------------------------------------------------------------
    #[test]
    fn action_table_row_count_equals_state_count(pt in arb_table()) {
        prop_assert_eq!(pt.action_table.len(), pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 2. Every action_table row width equals symbol_count
    // -----------------------------------------------------------------------
    #[test]
    fn action_table_column_width_uniform(pt in arb_table()) {
        for (i, row) in pt.action_table.iter().enumerate() {
            prop_assert_eq!(
                row.len(), pt.symbol_count,
                "row {} width {} != symbol_count {}", i, row.len(), pt.symbol_count
            );
        }
    }

    // -----------------------------------------------------------------------
    // 3. goto_table row count equals state_count
    // -----------------------------------------------------------------------
    #[test]
    fn goto_table_row_count_equals_state_count(pt in arb_table()) {
        prop_assert_eq!(pt.goto_table.len(), pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 4. goto_table column widths are uniform across rows
    // -----------------------------------------------------------------------
    #[test]
    fn goto_table_column_width_uniform(pt in arb_table()) {
        if let Some(first) = pt.goto_table.first() {
            let w = first.len();
            for (i, row) in pt.goto_table.iter().enumerate() {
                prop_assert_eq!(row.len(), w, "goto row {} width mismatch", i);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 5. symbol_to_index has exactly symbol_count entries
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_to_index_size(pt in arb_table()) {
        prop_assert_eq!(pt.symbol_to_index.len(), pt.symbol_count);
    }

    // -----------------------------------------------------------------------
    // 6. index_to_symbol length equals symbol_count
    // -----------------------------------------------------------------------
    #[test]
    fn index_to_symbol_size(pt in arb_table()) {
        prop_assert_eq!(pt.index_to_symbol.len(), pt.symbol_count);
    }

    // -----------------------------------------------------------------------
    // 7. symbol_to_index ↔ index_to_symbol bijectivity
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_index_bijection(pt in arb_table()) {
        for (&sym, &idx) in &pt.symbol_to_index {
            prop_assert!(idx < pt.index_to_symbol.len());
            prop_assert_eq!(pt.index_to_symbol[idx], sym);
        }
        for (idx, &sym) in pt.index_to_symbol.iter().enumerate() {
            prop_assert_eq!(*pt.symbol_to_index.get(&sym).unwrap(), idx);
        }
    }

    // -----------------------------------------------------------------------
    // 8. nonterminal_to_index keys lie in [token_count, symbol_count)
    // -----------------------------------------------------------------------
    #[test]
    fn nonterminal_indices_in_nonterminal_range(pt in arb_table()) {
        for &sym in pt.nonterminal_to_index.keys() {
            prop_assert!(
                (sym.0 as usize) >= pt.token_count,
                "nonterminal {:?} below token_count {}", sym, pt.token_count
            );
            prop_assert!(
                (sym.0 as usize) < pt.symbol_count,
                "nonterminal {:?} >= symbol_count {}", sym, pt.symbol_count
            );
        }
    }

    // -----------------------------------------------------------------------
    // 9. nonterminal_to_index column values are dense [0..nnt)
    // -----------------------------------------------------------------------
    #[test]
    fn nonterminal_column_values_dense(pt in arb_table()) {
        let n = pt.nonterminal_to_index.len();
        let mut cols: Vec<usize> = pt.nonterminal_to_index.values().copied().collect();
        cols.sort();
        cols.dedup();
        prop_assert_eq!(cols.len(), n);
        if n > 0 {
            prop_assert_eq!(*cols.last().unwrap(), n - 1);
        }
    }

    // -----------------------------------------------------------------------
    // 10. Clone preserves action_table cell-by-cell
    // -----------------------------------------------------------------------
    #[test]
    fn clone_preserves_action_cells(pt in arb_table()) {
        let c = pt.clone();
        for s in 0..pt.state_count {
            for col in 0..pt.symbol_count {
                prop_assert_eq!(&c.action_table[s][col], &pt.action_table[s][col]);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 11. Clone preserves goto_table cell-by-cell
    // -----------------------------------------------------------------------
    #[test]
    fn clone_preserves_goto_cells(pt in arb_table()) {
        let c = pt.clone();
        for s in 0..pt.state_count {
            for col in 0..pt.goto_table[s].len() {
                prop_assert_eq!(c.goto_table[s][col], pt.goto_table[s][col]);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 12. Shift action retains state id through clone
    // -----------------------------------------------------------------------
    #[test]
    fn shift_action_clone_fidelity(state in 0u16..500) {
        let a = Action::Shift(StateId(state));
        let b = a.clone();
        prop_assert_eq!(a, b);
    }

    // -----------------------------------------------------------------------
    // 13. Reduce action retains rule id through clone
    // -----------------------------------------------------------------------
    #[test]
    fn reduce_action_clone_fidelity(rule in 0u16..500) {
        let a = Action::Reduce(RuleId(rule));
        let b = a.clone();
        prop_assert_eq!(a, b);
    }

    // -----------------------------------------------------------------------
    // 14. Fork action preserves inner actions
    // -----------------------------------------------------------------------
    #[test]
    fn fork_preserves_inner(inner in prop::collection::vec(leaf_action(), 2..=6)) {
        let f = Action::Fork(inner.clone());
        if let Action::Fork(ref v) = f {
            prop_assert_eq!(v.len(), inner.len());
            for (a, b) in v.iter().zip(inner.iter()) {
                prop_assert_eq!(a, b);
            }
        } else {
            prop_assert!(false, "expected Fork variant");
        }
    }

    // -----------------------------------------------------------------------
    // 15. Action Debug output is non-empty for every variant
    // -----------------------------------------------------------------------
    #[test]
    fn action_debug_non_empty(action in any_action()) {
        let dbg = format!("{:?}", action);
        prop_assert!(!dbg.is_empty());
    }

    // -----------------------------------------------------------------------
    // 16. Default table symbol_count is zero
    // -----------------------------------------------------------------------
    #[test]
    fn default_table_zero_symbols(_dummy in 0u8..1) {
        let pt = ParseTable::default();
        prop_assert_eq!(pt.symbol_count, 0);
        prop_assert_eq!(pt.state_count, 0);
        prop_assert!(pt.symbol_to_index.is_empty());
        prop_assert!(pt.nonterminal_to_index.is_empty());
        prop_assert!(pt.index_to_symbol.is_empty());
    }

    // -----------------------------------------------------------------------
    // 17. eof_symbol is always SymbolId(0) after construction
    // -----------------------------------------------------------------------
    #[test]
    fn eof_symbol_is_zero(pt in arb_table()) {
        prop_assert_eq!(pt.eof_symbol, SymbolId(0));
    }

    // -----------------------------------------------------------------------
    // 18. eof_symbol is always in symbol_to_index
    // -----------------------------------------------------------------------
    #[test]
    fn eof_in_symbol_to_index(pt in arb_table()) {
        prop_assert!(pt.symbol_to_index.contains_key(&pt.eof_symbol));
    }

    // -----------------------------------------------------------------------
    // 19. start_symbol ≥ token_count (it is a nonterminal)
    // -----------------------------------------------------------------------
    #[test]
    fn start_symbol_is_nonterminal(pt in arb_table()) {
        prop_assert!(
            (pt.start_symbol.0 as usize) >= pt.token_count,
            "start {:?} < token_count {}", pt.start_symbol, pt.token_count
        );
    }

    // -----------------------------------------------------------------------
    // 20. initial_state within [0, state_count)
    // -----------------------------------------------------------------------
    #[test]
    fn initial_state_within_bounds(pt in arb_table()) {
        prop_assert!((pt.initial_state.0 as usize) < pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 21. terminal_boundary equals token_count when external_token_count is 0
    // -----------------------------------------------------------------------
    #[test]
    fn terminal_boundary_no_externals(pt in arb_table()) {
        prop_assert_eq!(pt.external_token_count, 0);
        prop_assert_eq!(pt.terminal_boundary(), pt.token_count);
    }

    // -----------------------------------------------------------------------
    // 22. dynamic_prec and rule_assoc lengths match rules
    // -----------------------------------------------------------------------
    #[test]
    fn per_rule_vectors_match_rules_len(pt in arb_table()) {
        prop_assert_eq!(pt.dynamic_prec_by_rule.len(), pt.rules.len());
        prop_assert_eq!(pt.rule_assoc_by_rule.len(), pt.rules.len());
    }

    // -----------------------------------------------------------------------
    // 23. lex_modes length matches state_count
    // -----------------------------------------------------------------------
    #[test]
    fn lex_modes_match_states(pt in arb_table()) {
        prop_assert_eq!(pt.lex_modes.len(), pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 24. rule() accessor round-trips for every rule
    // -----------------------------------------------------------------------
    #[test]
    fn rule_accessor_roundtrip(pt in arb_table()) {
        for (i, r) in pt.rules.iter().enumerate() {
            let (lhs, rhs_len) = pt.rule(RuleId(i as u16));
            prop_assert_eq!(lhs, r.lhs);
            prop_assert_eq!(rhs_len, r.rhs_len);
        }
    }

    // -----------------------------------------------------------------------
    // 25. actions() for mapped symbols returns the exact cell
    // -----------------------------------------------------------------------
    #[test]
    fn actions_returns_exact_cell(pt in arb_table()) {
        for s in 0..pt.state_count {
            for (&sym, &col) in &pt.symbol_to_index {
                let got = pt.actions(StateId(s as u16), sym);
                prop_assert_eq!(got, pt.action_table[s][col].as_slice());
            }
        }
    }

    // -----------------------------------------------------------------------
    // 26. goto() None iff sentinel or missing
    // -----------------------------------------------------------------------
    #[test]
    fn goto_none_iff_sentinel(pt in arb_table()) {
        for s in 0..pt.state_count {
            for (&nt, &col) in &pt.nonterminal_to_index {
                let val = pt.goto_table[s][col];
                let result = pt.goto(StateId(s as u16), nt);
                if val == SENTINEL {
                    prop_assert_eq!(result, None);
                } else {
                    prop_assert_eq!(result, Some(val));
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 27. normalize_eof_to_zero is idempotent
    // -----------------------------------------------------------------------
    #[test]
    fn normalize_eof_idempotent(pt in arb_table()) {
        let once = pt.clone().normalize_eof_to_zero();
        let twice = once.clone().normalize_eof_to_zero();
        prop_assert_eq!(once.eof_symbol, twice.eof_symbol);
        prop_assert_eq!(once.action_table, twice.action_table);
        prop_assert_eq!(once.symbol_to_index, twice.symbol_to_index);
    }

    // -----------------------------------------------------------------------
    // 28. with_detected_goto_indexing is idempotent
    // -----------------------------------------------------------------------
    #[test]
    fn detect_goto_indexing_idempotent(pt in arb_table()) {
        let a = pt.clone().with_detected_goto_indexing();
        let b = a.clone().with_detected_goto_indexing();
        prop_assert_eq!(a.goto_indexing, b.goto_indexing);
    }

    // -----------------------------------------------------------------------
    // 29. valid_symbols length equals terminal_boundary for all states
    // -----------------------------------------------------------------------
    #[test]
    fn valid_symbols_len(pt in arb_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            prop_assert_eq!(vs.len(), pt.terminal_boundary());
        }
    }

    // -----------------------------------------------------------------------
    // 30. valid_symbols and valid_symbols_mask agree
    // -----------------------------------------------------------------------
    #[test]
    fn valid_symbols_and_mask_agree(pt in arb_table()) {
        for s in 0..pt.state_count {
            let a = pt.valid_symbols(StateId(s as u16));
            let b = pt.valid_symbols_mask(StateId(s as u16));
            prop_assert_eq!(a, b);
        }
    }

    // -----------------------------------------------------------------------
    // 31. Construction with empty rules yields empty dynamic_prec/rule_assoc
    // -----------------------------------------------------------------------
    #[test]
    fn empty_rules_empty_prec_assoc(
        nt in 1usize..=4, nnt in 1usize..=3, ns in 1usize..=4,
    ) {
        let sc = nt + nnt;
        let meta: Vec<SymbolMetadata> = (0..sc as u16)
            .map(|i| SymbolMetadata {
                name: format!("s{i}"), is_visible: false, is_named: false,
                is_supertype: false, is_terminal: (i as usize) < nt,
                is_extra: false, is_fragile: false, symbol_id: SymbolId(i),
            })
            .collect();
        let pt = make_table(
            ns, nt, nnt,
            vec![vec![vec![]; sc]; ns],
            vec![vec![SENTINEL; nnt]; ns],
            vec![],
            meta,
        );
        prop_assert!(pt.dynamic_prec_by_rule.is_empty());
        prop_assert!(pt.rule_assoc_by_rule.is_empty());
    }

    // -----------------------------------------------------------------------
    // 32. symbol_metadata length equals symbol_count
    // -----------------------------------------------------------------------
    #[test]
    fn metadata_length_matches_symbol_count(pt in arb_table()) {
        prop_assert_eq!(pt.symbol_metadata.len(), pt.symbol_count);
    }

    // -----------------------------------------------------------------------
    // 33. All symbol_metadata ids form [0..symbol_count)
    // -----------------------------------------------------------------------
    #[test]
    fn metadata_ids_contiguous(pt in arb_table()) {
        for (i, m) in pt.symbol_metadata.iter().enumerate() {
            prop_assert_eq!(m.symbol_id, SymbolId(i as u16));
        }
    }

    // -----------------------------------------------------------------------
    // 34. field_map is empty by default construction
    // -----------------------------------------------------------------------
    #[test]
    fn field_map_empty_default(pt in arb_table()) {
        prop_assert!(pt.field_map.is_empty());
    }

    // -----------------------------------------------------------------------
    // 35. external_scanner_states is empty when external_token_count is 0
    // -----------------------------------------------------------------------
    #[test]
    fn no_external_scanner_states_when_no_externals(pt in arb_table()) {
        prop_assert_eq!(pt.external_token_count, 0);
        prop_assert!(pt.external_scanner_states.is_empty());
    }
}
