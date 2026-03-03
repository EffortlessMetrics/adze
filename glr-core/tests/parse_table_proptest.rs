#![allow(clippy::needless_range_loop)]
//! Property-based tests for `ParseTable` construction and invariants.
//!
//! Run with: `cargo test -p adze-glr-core --test parse_table_proptest`

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

type ActionCell = Vec<Action>;

const NO_GOTO: StateId = StateId(65535);

/// Generate a leaf `Action` (no `Fork`).
fn leaf_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..100u16).prop_map(|s| Action::Shift(StateId(s))),
        (0..100u16).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Generate an `ActionCell`.
fn arb_action_cell() -> impl Strategy<Value = ActionCell> {
    prop::collection::vec(leaf_action(), 0..=4)
}

/// Generate a `SymbolMetadata`.
fn arb_symbol_metadata(id: u16) -> impl Strategy<Value = adze_glr_core::SymbolMetadata> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(move |(vis, named, sup, term, extra, fragile)| {
            adze_glr_core::SymbolMetadata {
                name: format!("sym_{id}"),
                is_visible: vis,
                is_named: named,
                is_supertype: sup,
                is_terminal: term,
                is_extra: extra,
                is_fragile: fragile,
                symbol_id: SymbolId(id),
            }
        })
}

/// Build a well-formed `ParseTable` with the given dimensions.
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
        grammar: Grammar::new("proptest".to_string()),
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

/// Strategy that generates a consistent `ParseTable` with random dimensions.
fn arb_parse_table() -> impl Strategy<Value = ParseTable> {
    // num_terminals 1..=6, num_nonterminals 1..=4, num_states 1..=8
    (1usize..=6, 1usize..=4, 1usize..=8)
        .prop_flat_map(|(num_terminals, num_nonterminals, num_states)| {
            let sym_count = num_terminals + num_nonterminals;
            let actions = prop::collection::vec(
                prop::collection::vec(arb_action_cell(), sym_count..=sym_count),
                num_states..=num_states,
            );
            let gotos = prop::collection::vec(
                prop::collection::vec(
                    prop_oneof![Just(NO_GOTO), (0..num_states as u16).prop_map(StateId),],
                    num_nonterminals..=num_nonterminals,
                ),
                num_states..=num_states,
            );
            let rules = prop::collection::vec(
                (
                    (num_terminals as u16..sym_count as u16).prop_map(SymbolId),
                    0u16..=5,
                )
                    .prop_map(|(lhs, rhs_len)| ParseRule { lhs, rhs_len }),
                0..=6,
            );
            let metadata = (0..sym_count as u16)
                .map(arb_symbol_metadata)
                .collect::<Vec<_>>();
            (
                Just(num_states),
                Just(num_terminals),
                Just(num_nonterminals),
                actions,
                gotos,
                rules,
                metadata,
            )
        })
        .prop_map(|(ns, nt, nnt, actions, gotos, rules, metadata)| {
            build_table(ns, nt, nnt, actions, gotos, rules, metadata)
        })
}

// ===========================================================================
// Property tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // -----------------------------------------------------------------------
    // 1. Default ParseTable has consistent dimensions
    // -----------------------------------------------------------------------
    #[test]
    fn default_table_dimensions(_dummy in 0u8..1) {
        let pt = ParseTable::default();
        prop_assert_eq!(pt.state_count, 0);
        prop_assert_eq!(pt.symbol_count, 0);
        prop_assert!(pt.action_table.is_empty());
        prop_assert!(pt.goto_table.is_empty());
        prop_assert!(pt.symbol_metadata.is_empty());
        prop_assert!(pt.rules.is_empty());
        prop_assert!(pt.symbol_to_index.is_empty());
        prop_assert!(pt.index_to_symbol.is_empty());
        prop_assert!(pt.nonterminal_to_index.is_empty());
        prop_assert!(pt.lex_modes.is_empty());
        prop_assert!(pt.extras.is_empty());
    }

    // -----------------------------------------------------------------------
    // 2. State count matches action_table rows
    // -----------------------------------------------------------------------
    #[test]
    fn state_count_matches_action_rows(pt in arb_parse_table()) {
        prop_assert_eq!(pt.state_count, pt.action_table.len());
    }

    // -----------------------------------------------------------------------
    // 3. Symbol count matches action_table columns
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_count_matches_action_cols(pt in arb_parse_table()) {
        for (i, row) in pt.action_table.iter().enumerate() {
            prop_assert_eq!(
                row.len(),
                pt.symbol_count,
                "action_table row {} has {} cols, expected {}",
                i, row.len(), pt.symbol_count
            );
        }
    }

    // -----------------------------------------------------------------------
    // 4. ParseTable clone preserves all fields
    // -----------------------------------------------------------------------
    #[test]
    fn clone_preserves_fields(pt in arb_parse_table()) {
        let cloned = pt.clone();
        prop_assert_eq!(cloned.state_count, pt.state_count);
        prop_assert_eq!(cloned.symbol_count, pt.symbol_count);
        prop_assert_eq!(cloned.action_table, pt.action_table);
        prop_assert_eq!(cloned.goto_table, pt.goto_table);
        prop_assert_eq!(cloned.eof_symbol, pt.eof_symbol);
        prop_assert_eq!(cloned.start_symbol, pt.start_symbol);
        prop_assert_eq!(cloned.token_count, pt.token_count);
        prop_assert_eq!(cloned.external_token_count, pt.external_token_count);
        prop_assert_eq!(cloned.initial_state, pt.initial_state);
        prop_assert_eq!(cloned.symbol_to_index, pt.symbol_to_index);
        prop_assert_eq!(cloned.index_to_symbol, pt.index_to_symbol);
        prop_assert_eq!(cloned.nonterminal_to_index, pt.nonterminal_to_index);
        prop_assert_eq!(cloned.goto_indexing, pt.goto_indexing);
        prop_assert_eq!(cloned.rules.len(), pt.rules.len());
        prop_assert_eq!(cloned.lex_modes, pt.lex_modes);
        prop_assert_eq!(cloned.extras, pt.extras);
        prop_assert_eq!(cloned.dynamic_prec_by_rule, pt.dynamic_prec_by_rule);
        prop_assert_eq!(cloned.rule_assoc_by_rule, pt.rule_assoc_by_rule);
        prop_assert_eq!(cloned.field_names, pt.field_names);
        prop_assert_eq!(cloned.field_map, pt.field_map);
    }

    // -----------------------------------------------------------------------
    // 5. symbol_to_index and index_to_symbol are consistent
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_index_roundtrip(pt in arb_parse_table()) {
        for (&sym, &idx) in &pt.symbol_to_index {
            prop_assert!(idx < pt.index_to_symbol.len(),
                "index {} out of range for index_to_symbol (len {})", idx, pt.index_to_symbol.len());
            prop_assert_eq!(pt.index_to_symbol[idx], sym,
                "index_to_symbol[{}] = {:?}, expected {:?}", idx, pt.index_to_symbol[idx], sym);
        }
        for (idx, &sym) in pt.index_to_symbol.iter().enumerate() {
            let mapped = pt.symbol_to_index.get(&sym);
            prop_assert!(mapped.is_some(), "symbol {:?} not in symbol_to_index", sym);
            prop_assert_eq!(*mapped.unwrap(), idx);
        }
    }

    // -----------------------------------------------------------------------
    // 6. Goto table rows match state count
    // -----------------------------------------------------------------------
    #[test]
    fn goto_rows_match_state_count(pt in arb_parse_table()) {
        prop_assert_eq!(pt.goto_table.len(), pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 7. Goto table column widths are uniform
    // -----------------------------------------------------------------------
    #[test]
    fn goto_cols_uniform(pt in arb_parse_table()) {
        if let Some(first_row) = pt.goto_table.first() {
            let expected_cols = first_row.len();
            for (i, row) in pt.goto_table.iter().enumerate() {
                prop_assert_eq!(row.len(), expected_cols,
                    "goto_table row {} has {} cols, expected {}", i, row.len(), expected_cols);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 8. terminal_boundary equals token_count + external_token_count
    // -----------------------------------------------------------------------
    #[test]
    fn terminal_boundary_correct(pt in arb_parse_table()) {
        prop_assert_eq!(pt.terminal_boundary(), pt.token_count + pt.external_token_count);
    }

    // -----------------------------------------------------------------------
    // 9. is_terminal consistent with terminal_boundary
    // -----------------------------------------------------------------------
    #[test]
    fn is_terminal_consistent(pt in arb_parse_table()) {
        let boundary = pt.terminal_boundary();
        for i in 0..pt.symbol_count {
            let sym = SymbolId(i as u16);
            if (i) < boundary {
                prop_assert!(pt.is_terminal(sym), "symbol {} should be terminal", i);
            } else {
                prop_assert!(!pt.is_terminal(sym), "symbol {} should not be terminal", i);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 10. eof() returns eof_symbol
    // -----------------------------------------------------------------------
    #[test]
    fn eof_accessor(pt in arb_parse_table()) {
        prop_assert_eq!(pt.eof(), pt.eof_symbol);
    }

    // -----------------------------------------------------------------------
    // 11. start_symbol() returns start_symbol
    // -----------------------------------------------------------------------
    #[test]
    fn start_symbol_accessor(pt in arb_parse_table()) {
        prop_assert_eq!(pt.start_symbol(), pt.start_symbol);
    }

    // -----------------------------------------------------------------------
    // 12. lex_modes length matches state_count
    // -----------------------------------------------------------------------
    #[test]
    fn lex_modes_length(pt in arb_parse_table()) {
        prop_assert_eq!(pt.lex_modes.len(), pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 13. lex_mode accessor returns default for out-of-range state
    // -----------------------------------------------------------------------
    #[test]
    fn lex_mode_out_of_range(pt in arb_parse_table()) {
        let oob = StateId(pt.state_count as u16 + 1);
        let mode = pt.lex_mode(oob);
        prop_assert_eq!(mode.lex_state, 0);
        prop_assert_eq!(mode.external_lex_state, 0);
    }

    // -----------------------------------------------------------------------
    // 14. lex_mode accessor returns stored value for valid state
    // -----------------------------------------------------------------------
    #[test]
    fn lex_mode_in_range(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let mode = pt.lex_mode(StateId(s as u16));
            prop_assert_eq!(mode, pt.lex_modes[s]);
        }
    }

    // -----------------------------------------------------------------------
    // 15. actions() returns empty for out-of-range state
    // -----------------------------------------------------------------------
    #[test]
    fn actions_oob_state(pt in arb_parse_table()) {
        let oob = StateId(pt.state_count as u16 + 10);
        let result = pt.actions(oob, SymbolId(0));
        prop_assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // 16. actions() returns empty for unmapped symbol
    // -----------------------------------------------------------------------
    #[test]
    fn actions_unmapped_symbol(pt in arb_parse_table()) {
        let unmapped = SymbolId(pt.symbol_count as u16 + 100);
        let result = pt.actions(StateId(0), unmapped);
        prop_assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // 17. actions() returns correct cell for valid state/symbol
    // -----------------------------------------------------------------------
    #[test]
    fn actions_valid_lookup(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&sym, &col) in &pt.symbol_to_index {
                let actions = pt.actions(StateId(s as u16), sym);
                prop_assert_eq!(
                    actions,
                    pt.action_table[s][col].as_slice(),
                    "actions({}, {:?}) mismatch", s, sym
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // 18. goto() returns None for out-of-range state
    // -----------------------------------------------------------------------
    #[test]
    fn goto_oob_state(pt in arb_parse_table()) {
        let oob = StateId(pt.state_count as u16 + 10);
        for &nt in pt.nonterminal_to_index.keys() {
            prop_assert_eq!(pt.goto(oob, nt), None);
        }
    }

    // -----------------------------------------------------------------------
    // 19. goto() returns None for unmapped nonterminal
    // -----------------------------------------------------------------------
    #[test]
    fn goto_unmapped_nt(pt in arb_parse_table()) {
        let unmapped = SymbolId(pt.symbol_count as u16 + 200);
        prop_assert_eq!(pt.goto(StateId(0), unmapped), None);
    }

    // -----------------------------------------------------------------------
    // 20. goto() sentinel value means None
    // -----------------------------------------------------------------------
    #[test]
    fn goto_sentinel_is_none(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&nt, &col) in &pt.nonterminal_to_index {
                if col < pt.goto_table[s].len() && pt.goto_table[s][col] == NO_GOTO {
                    prop_assert_eq!(pt.goto(StateId(s as u16), nt), None,
                        "goto({}, {:?}) should be None for sentinel", s, nt);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 21. rule() accessor is consistent with rules vec
    // -----------------------------------------------------------------------
    #[test]
    fn rule_accessor(pt in arb_parse_table()) {
        for (i, r) in pt.rules.iter().enumerate() {
            let (lhs, rhs_len) = pt.rule(RuleId(i as u16));
            prop_assert_eq!(lhs, r.lhs);
            prop_assert_eq!(rhs_len, r.rhs_len);
        }
    }

    // -----------------------------------------------------------------------
    // 22. is_extra checks extras list
    // -----------------------------------------------------------------------
    #[test]
    fn is_extra_consistent(pt in arb_parse_table()) {
        for i in 0..pt.symbol_count {
            let sym = SymbolId(i as u16);
            prop_assert_eq!(pt.is_extra(sym), pt.extras.contains(&sym));
        }
    }

    // -----------------------------------------------------------------------
    // 23. valid_symbols length equals terminal_boundary
    // -----------------------------------------------------------------------
    #[test]
    fn valid_symbols_length(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            prop_assert_eq!(vs.len(), pt.terminal_boundary());
        }
    }

    // -----------------------------------------------------------------------
    // 24. valid_symbols_mask matches valid_symbols
    // -----------------------------------------------------------------------
    #[test]
    fn valid_symbols_mask_parity(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            let vm = pt.valid_symbols_mask(StateId(s as u16));
            prop_assert_eq!(vs, vm,
                "valid_symbols and valid_symbols_mask disagree for state {}", s);
        }
    }

    // -----------------------------------------------------------------------
    // 25. dynamic_prec_by_rule length matches rules
    // -----------------------------------------------------------------------
    #[test]
    fn dynamic_prec_length(pt in arb_parse_table()) {
        prop_assert_eq!(pt.dynamic_prec_by_rule.len(), pt.rules.len());
    }

    // -----------------------------------------------------------------------
    // 26. rule_assoc_by_rule length matches rules
    // -----------------------------------------------------------------------
    #[test]
    fn rule_assoc_length(pt in arb_parse_table()) {
        prop_assert_eq!(pt.rule_assoc_by_rule.len(), pt.rules.len());
    }

    // -----------------------------------------------------------------------
    // 27. symbol_metadata symbol_id matches index
    // -----------------------------------------------------------------------
    #[test]
    fn metadata_symbol_id_matches_index(pt in arb_parse_table()) {
        for (i, meta) in pt.symbol_metadata.iter().enumerate() {
            prop_assert_eq!(meta.symbol_id, SymbolId(i as u16),
                "symbol_metadata[{}].symbol_id = {:?}", i, meta.symbol_id);
        }
    }

    // -----------------------------------------------------------------------
    // 28. symbol_metadata name is non-empty
    // -----------------------------------------------------------------------
    #[test]
    fn metadata_name_non_empty(pt in arb_parse_table()) {
        for (i, meta) in pt.symbol_metadata.iter().enumerate() {
            prop_assert!(!meta.name.is_empty(),
                "symbol_metadata[{}] has empty name", i);
        }
    }

    // -----------------------------------------------------------------------
    // 29. nonterminal_to_index keys are non-terminals
    // -----------------------------------------------------------------------
    #[test]
    fn nonterminal_keys_are_nonterminals(pt in arb_parse_table()) {
        let boundary = pt.terminal_boundary();
        for &sym in pt.nonterminal_to_index.keys() {
            prop_assert!(
                (sym.0 as usize) >= boundary,
                "nonterminal_to_index contains terminal symbol {:?} (boundary={})",
                sym, boundary
            );
        }
    }

    // -----------------------------------------------------------------------
    // 30. initial_state is within state_count
    // -----------------------------------------------------------------------
    #[test]
    fn initial_state_in_range(pt in arb_parse_table()) {
        prop_assert!((pt.initial_state.0 as usize) < pt.state_count,
            "initial_state {:?} >= state_count {}", pt.initial_state, pt.state_count);
    }
}

// ===========================================================================
// SymbolMetadata-focused property tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    // -----------------------------------------------------------------------
    // 31. SymbolMetadata clone roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_metadata_clone(meta in arb_symbol_metadata(42)) {
        let cloned = meta.clone();
        prop_assert_eq!(cloned.name, meta.name);
        prop_assert_eq!(cloned.is_visible, meta.is_visible);
        prop_assert_eq!(cloned.is_named, meta.is_named);
        prop_assert_eq!(cloned.is_supertype, meta.is_supertype);
        prop_assert_eq!(cloned.is_terminal, meta.is_terminal);
        prop_assert_eq!(cloned.is_extra, meta.is_extra);
        prop_assert_eq!(cloned.is_fragile, meta.is_fragile);
        prop_assert_eq!(cloned.symbol_id, meta.symbol_id);
    }

    // -----------------------------------------------------------------------
    // 32. SymbolMetadata debug representation contains name
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_metadata_debug_contains_name(meta in arb_symbol_metadata(7)) {
        let debug = format!("{:?}", meta);
        prop_assert!(debug.contains("sym_7"), "Debug output should contain name");
    }

    // -----------------------------------------------------------------------
    // 33. SymbolMetadata boolean fields are independent
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_metadata_fields_independent(
        v in any::<bool>(), n in any::<bool>(), s in any::<bool>(),
        t in any::<bool>(), e in any::<bool>(), f in any::<bool>(),
    ) {
        let meta = adze_glr_core::SymbolMetadata {
            name: "test".into(),
            is_visible: v, is_named: n, is_supertype: s,
            is_terminal: t, is_extra: e, is_fragile: f,
            symbol_id: SymbolId(0),
        };
        prop_assert_eq!(meta.is_visible, v);
        prop_assert_eq!(meta.is_named, n);
        prop_assert_eq!(meta.is_supertype, s);
        prop_assert_eq!(meta.is_terminal, t);
        prop_assert_eq!(meta.is_extra, e);
        prop_assert_eq!(meta.is_fragile, f);
    }
}

// ===========================================================================
// Action cell invariant tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // -----------------------------------------------------------------------
    // 34. Action cell is a valid Vec (can be cloned and compared)
    // -----------------------------------------------------------------------
    #[test]
    fn action_cell_clone_eq(cell in arb_action_cell()) {
        let cloned = cell.clone();
        prop_assert_eq!(&cell, &cloned);
    }

    // -----------------------------------------------------------------------
    // 35. Empty action cell means error/no-action
    // -----------------------------------------------------------------------
    #[test]
    fn empty_cell_implies_no_actions(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.action_table[s].len() {
                if pt.action_table[s][col].is_empty() {
                    // Find the corresponding symbol for this column
                    if let Some((&sym, _)) = pt.symbol_to_index.iter().find(|&(_, &c)| c == col) {
                        let actions = pt.actions(StateId(s as u16), sym);
                        prop_assert!(actions.is_empty(),
                            "empty cell at ({}, {}) should yield empty actions", s, col);
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 36. ParseTable with extras: is_extra reflects membership
    // -----------------------------------------------------------------------
    #[test]
    fn extras_membership(
        num_terminals in 2usize..=5,
        extra_idx in 0usize..2,
    ) {
        let num_nt = 1;
        let sym_count = num_terminals + num_nt;
        let actual_extra = extra_idx.min(num_terminals - 1);
        let extra_sym = SymbolId(actual_extra as u16);

        let mut pt = build_table(
            1, num_terminals, num_nt,
            vec![vec![vec![]; sym_count]],
            vec![vec![NO_GOTO; num_nt]],
            vec![],
            (0..sym_count as u16).map(|i| adze_glr_core::SymbolMetadata {
                name: format!("s{i}"),
                is_visible: false, is_named: false, is_supertype: false,
                is_terminal: (i as usize) < num_terminals,
                is_extra: false, is_fragile: false,
                symbol_id: SymbolId(i),
            }).collect(),
        );
        pt.extras.push(extra_sym);

        prop_assert!(pt.is_extra(extra_sym));
        // A symbol not in extras should return false
        let non_extra = SymbolId((actual_extra as u16 + 1).min(num_terminals as u16 - 1));
        if non_extra != extra_sym {
            prop_assert!(!pt.is_extra(non_extra));
        }
    }
}

// ===========================================================================
// GotoIndexing property tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // -----------------------------------------------------------------------
    // 37. GotoIndexing clone/eq
    // -----------------------------------------------------------------------
    #[test]
    fn goto_indexing_clone_eq(use_direct in any::<bool>()) {
        let gi = if use_direct { GotoIndexing::DirectSymbolId } else { GotoIndexing::NonterminalMap };
        let cloned = gi;
        prop_assert_eq!(gi, cloned);
    }

    // -----------------------------------------------------------------------
    // 38. LexMode clone/eq
    // -----------------------------------------------------------------------
    #[test]
    fn lex_mode_clone_eq(ls in 0u16..100, els in 0u16..100) {
        let mode = LexMode { lex_state: ls, external_lex_state: els };
        let cloned = mode;
        prop_assert_eq!(mode, cloned);
    }

    // -----------------------------------------------------------------------
    // 39. ParseRule clone preserves fields
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_clone(lhs in 0u16..100, rhs_len in 0u16..20) {
        let rule = ParseRule { lhs: SymbolId(lhs), rhs_len };
        let cloned = rule.clone();
        prop_assert_eq!(cloned.lhs, rule.lhs);
        prop_assert_eq!(cloned.rhs_len, rule.rhs_len);
    }

    // -----------------------------------------------------------------------
    // 40. with_detected_goto_indexing is idempotent
    // -----------------------------------------------------------------------
    #[test]
    fn detected_goto_idempotent(pt in arb_parse_table()) {
        let once = pt.clone().with_detected_goto_indexing();
        let twice = once.clone().with_detected_goto_indexing();
        prop_assert_eq!(once.goto_indexing, twice.goto_indexing);
    }
}
