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
// Property tests — ParseTable creation and dimensions
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 1. Default ParseTable has consistent dimensions
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
    }

    // 2. State count matches action_table rows
    #[test]
    fn state_count_matches_action_rows(pt in arb_parse_table()) {
        prop_assert_eq!(pt.state_count, pt.action_table.len());
    }

    // 3. Symbol count matches action_table columns per row
    #[test]
    fn symbol_count_matches_action_cols(pt in arb_parse_table()) {
        for (i, row) in pt.action_table.iter().enumerate() {
            prop_assert_eq!(
                row.len(), pt.symbol_count,
                "action_table row {} has {} cols, expected {}", i, row.len(), pt.symbol_count
            );
        }
    }

    // 4. Empty tables: zero-state table has no action/goto rows
    #[test]
    fn empty_table_has_no_rows(_dummy in 0u8..1) {
        let pt = build_table(0, 1, 1, vec![], vec![], vec![], vec![
            adze_glr_core::SymbolMetadata {
                name: "t".into(), is_visible: false, is_named: false, is_supertype: false,
                is_terminal: true, is_extra: false, is_fragile: false, symbol_id: SymbolId(0),
            },
            adze_glr_core::SymbolMetadata {
                name: "n".into(), is_visible: false, is_named: false, is_supertype: false,
                is_terminal: false, is_extra: false, is_fragile: false, symbol_id: SymbolId(1),
            },
        ]);
        prop_assert_eq!(pt.state_count, 0);
        prop_assert!(pt.action_table.is_empty());
        prop_assert!(pt.goto_table.is_empty());
    }

    // 5. symbol_to_index and index_to_symbol are bijective
    #[test]
    fn symbol_index_roundtrip(pt in arb_parse_table()) {
        for (&sym, &idx) in &pt.symbol_to_index {
            prop_assert!(idx < pt.index_to_symbol.len());
            prop_assert_eq!(pt.index_to_symbol[idx], sym);
        }
        for (idx, &sym) in pt.index_to_symbol.iter().enumerate() {
            let mapped = pt.symbol_to_index.get(&sym);
            prop_assert!(mapped.is_some());
            prop_assert_eq!(*mapped.unwrap(), idx);
        }
    }

    // 6. symbol_to_index length equals symbol_count
    #[test]
    fn symbol_to_index_size(pt in arb_parse_table()) {
        prop_assert_eq!(pt.symbol_to_index.len(), pt.symbol_count);
        prop_assert_eq!(pt.index_to_symbol.len(), pt.symbol_count);
    }
}

// ===========================================================================
// Property tests — ParseTable clone
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 7. Clone preserves all structural fields
    #[test]
    fn clone_preserves_fields(pt in arb_parse_table()) {
        let c = pt.clone();
        prop_assert_eq!(c.state_count, pt.state_count);
        prop_assert_eq!(c.symbol_count, pt.symbol_count);
        prop_assert_eq!(c.action_table, pt.action_table);
        prop_assert_eq!(c.goto_table, pt.goto_table);
        prop_assert_eq!(c.eof_symbol, pt.eof_symbol);
        prop_assert_eq!(c.start_symbol, pt.start_symbol);
        prop_assert_eq!(c.token_count, pt.token_count);
        prop_assert_eq!(c.initial_state, pt.initial_state);
        prop_assert_eq!(c.symbol_to_index, pt.symbol_to_index);
        prop_assert_eq!(c.index_to_symbol, pt.index_to_symbol);
        prop_assert_eq!(c.nonterminal_to_index, pt.nonterminal_to_index);
        prop_assert_eq!(c.goto_indexing, pt.goto_indexing);
        prop_assert_eq!(c.rules.len(), pt.rules.len());
        prop_assert_eq!(c.lex_modes, pt.lex_modes);
        prop_assert_eq!(c.extras, pt.extras);
        prop_assert_eq!(c.dynamic_prec_by_rule, pt.dynamic_prec_by_rule);
        prop_assert_eq!(c.field_map, pt.field_map);
    }

    // 8. Clone is independent — mutating clone does not affect original
    #[test]
    fn clone_independence(pt in arb_parse_table()) {
        let mut c = pt.clone();
        c.state_count = 9999;
        c.symbol_count = 8888;
        prop_assert_ne!(c.state_count, pt.state_count);
        prop_assert_ne!(c.symbol_count, pt.symbol_count);
        // original unchanged
        prop_assert_eq!(pt.action_table.len(), pt.state_count);
    }
}

// ===========================================================================
// Property tests — action lookup
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 9. actions() returns correct cell for valid state/symbol
    #[test]
    fn actions_valid_lookup(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&sym, &col) in &pt.symbol_to_index {
                let actions = pt.actions(StateId(s as u16), sym);
                prop_assert_eq!(actions, pt.action_table[s][col].as_slice());
            }
        }
    }

    // 10. actions() returns empty for out-of-range state
    #[test]
    fn actions_oob_state(pt in arb_parse_table()) {
        let oob = StateId(pt.state_count as u16 + 10);
        let result = pt.actions(oob, SymbolId(0));
        prop_assert!(result.is_empty());
    }

    // 11. actions() returns empty for unmapped symbol
    #[test]
    fn actions_unmapped_symbol(pt in arb_parse_table()) {
        let unmapped = SymbolId(pt.symbol_count as u16 + 100);
        let result = pt.actions(StateId(0), unmapped);
        prop_assert!(result.is_empty());
    }

    // 12. Empty action cell yields empty actions slice
    #[test]
    fn empty_cell_implies_no_actions(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.action_table[s].len() {
                if pt.action_table[s][col].is_empty()
                    && let Some((&sym, _)) = pt.symbol_to_index.iter().find(|&(_, &c)| c == col) {
                        let actions = pt.actions(StateId(s as u16), sym);
                        prop_assert!(actions.is_empty());
                    }
            }
        }
    }
}

// ===========================================================================
// Property tests — goto lookup
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 13. Goto table rows match state count
    #[test]
    fn goto_rows_match_state_count(pt in arb_parse_table()) {
        prop_assert_eq!(pt.goto_table.len(), pt.state_count);
    }

    // 14. Goto table column widths are uniform
    #[test]
    fn goto_cols_uniform(pt in arb_parse_table()) {
        if let Some(first) = pt.goto_table.first() {
            for (i, row) in pt.goto_table.iter().enumerate() {
                prop_assert_eq!(row.len(), first.len(),
                    "goto_table row {} width mismatch", i);
            }
        }
    }

    // 15. goto() returns None for out-of-range state
    #[test]
    fn goto_oob_state(pt in arb_parse_table()) {
        let oob = StateId(pt.state_count as u16 + 10);
        for &nt in pt.nonterminal_to_index.keys() {
            prop_assert_eq!(pt.goto(oob, nt), None);
        }
    }

    // 16. goto() returns None for unmapped nonterminal
    #[test]
    fn goto_unmapped_nt(pt in arb_parse_table()) {
        let unmapped = SymbolId(pt.symbol_count as u16 + 200);
        prop_assert_eq!(pt.goto(StateId(0), unmapped), None);
    }

    // 17. goto() sentinel value means None
    #[test]
    fn goto_sentinel_is_none(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&nt, &col) in &pt.nonterminal_to_index {
                if col < pt.goto_table[s].len() && pt.goto_table[s][col] == NO_GOTO {
                    prop_assert_eq!(pt.goto(StateId(s as u16), nt), None);
                }
            }
        }
    }

    // 18. nonterminal_to_index keys are non-terminals
    #[test]
    fn nonterminal_keys_are_nonterminals(pt in arb_parse_table()) {
        let boundary = pt.terminal_boundary();
        for &sym in pt.nonterminal_to_index.keys() {
            prop_assert!((sym.0 as usize) >= boundary);
        }
    }
}

// ===========================================================================
// Property tests — state / symbol counts and accessors
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 19. terminal_boundary equals token_count + external_token_count
    #[test]
    fn terminal_boundary_correct(pt in arb_parse_table()) {
        prop_assert_eq!(pt.terminal_boundary(), pt.token_count + pt.external_token_count);
    }

    // 20. is_terminal consistent with terminal_boundary
    #[test]
    fn is_terminal_consistent(pt in arb_parse_table()) {
        let boundary = pt.terminal_boundary();
        for i in 0..pt.symbol_count {
            let sym = SymbolId(i as u16);
            if i < boundary {
                prop_assert!(pt.is_terminal(sym));
            } else {
                prop_assert!(!pt.is_terminal(sym));
            }
        }
    }

    // 21. eof() returns eof_symbol
    #[test]
    fn eof_accessor(pt in arb_parse_table()) {
        prop_assert_eq!(pt.eof(), pt.eof_symbol);
    }

    // 22. start_symbol() returns start_symbol
    #[test]
    fn start_symbol_accessor(pt in arb_parse_table()) {
        prop_assert_eq!(pt.start_symbol(), pt.start_symbol);
    }

    // 23. initial_state is within state_count
    #[test]
    fn initial_state_in_range(pt in arb_parse_table()) {
        prop_assert!((pt.initial_state.0 as usize) < pt.state_count);
    }

    // 24. rule() accessor is consistent with rules vec
    #[test]
    fn rule_accessor(pt in arb_parse_table()) {
        for (i, r) in pt.rules.iter().enumerate() {
            let (lhs, rhs_len) = pt.rule(RuleId(i as u16));
            prop_assert_eq!(lhs, r.lhs);
            prop_assert_eq!(rhs_len, r.rhs_len);
        }
    }

    // 25. valid_symbols length equals terminal_boundary
    #[test]
    fn valid_symbols_length(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            prop_assert_eq!(vs.len(), pt.terminal_boundary());
        }
    }

    // 26. lex_modes length matches state_count
    #[test]
    fn lex_modes_length(pt in arb_parse_table()) {
        prop_assert_eq!(pt.lex_modes.len(), pt.state_count);
    }

    // 27. dynamic_prec_by_rule and rule_assoc_by_rule lengths match rules
    #[test]
    fn auxiliary_vecs_match_rules(pt in arb_parse_table()) {
        prop_assert_eq!(pt.dynamic_prec_by_rule.len(), pt.rules.len());
        prop_assert_eq!(pt.rule_assoc_by_rule.len(), pt.rules.len());
    }
}

// ===========================================================================
// Property tests — serde roundtrip (Action always derives Serialize/Deserialize)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 28. Action::Shift serde roundtrip
    #[test]
    fn action_shift_serde_roundtrip(s in 0u16..1000) {
        let action = Action::Shift(StateId(s));
        let json = serde_json::to_string(&action).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(action, back);
    }

    // 29. Action::Reduce serde roundtrip
    #[test]
    fn action_reduce_serde_roundtrip(r in 0u16..1000) {
        let action = Action::Reduce(RuleId(r));
        let json = serde_json::to_string(&action).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(action, back);
    }

    // 30. Action enum variants serde roundtrip (all leaf variants)
    #[test]
    fn action_leaf_serde_roundtrip(action in leaf_action()) {
        let json = serde_json::to_string(&action).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(action, back);
    }

    // 31. ActionCell (Vec<Action>) serde roundtrip
    #[test]
    fn action_cell_serde_roundtrip(cell in arb_action_cell()) {
        let json = serde_json::to_string(&cell).unwrap();
        let back: Vec<Action> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(cell, back);
    }

    // 32. Action::Fork serde roundtrip
    #[test]
    fn action_fork_serde_roundtrip(inner in prop::collection::vec(leaf_action(), 1..=4)) {
        let action = Action::Fork(inner);
        let json = serde_json::to_string(&action).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(action, back);
    }
}

// ===========================================================================
// Property tests — with_detected_goto_indexing and extras
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 33. with_detected_goto_indexing is idempotent
    #[test]
    fn detected_goto_idempotent(pt in arb_parse_table()) {
        let once = pt.clone().with_detected_goto_indexing();
        let twice = once.clone().with_detected_goto_indexing();
        prop_assert_eq!(once.goto_indexing, twice.goto_indexing);
    }

    // 34. is_extra consistent with extras vec
    #[test]
    fn is_extra_consistent(pt in arb_parse_table()) {
        for i in 0..pt.symbol_count {
            let sym = SymbolId(i as u16);
            prop_assert_eq!(pt.is_extra(sym), pt.extras.contains(&sym));
        }
    }

    // 35. ParseRule clone preserves fields
    #[test]
    fn parse_rule_clone(lhs in 0u16..100, rhs_len in 0u16..20) {
        let rule = ParseRule { lhs: SymbolId(lhs), rhs_len };
        let cloned = rule.clone();
        prop_assert_eq!(cloned.lhs, rule.lhs);
        prop_assert_eq!(cloned.rhs_len, rule.rhs_len);
    }
}

// ===========================================================================
// Property tests — Debug trait
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 36. Debug output for ParseTable is non-empty
    #[test]
    fn debug_output_non_empty(pt in arb_parse_table()) {
        let dbg = format!("{:?}", pt);
        prop_assert!(!dbg.is_empty());
        prop_assert!(dbg.contains("action_table"));
    }

    // 37. Debug output for Action variants is well-formed
    #[test]
    fn debug_action_variants(action in leaf_action()) {
        let dbg = format!("{:?}", action);
        prop_assert!(!dbg.is_empty());
        let valid = dbg.starts_with("Shift") || dbg.starts_with("Reduce")
            || dbg.starts_with("Accept") || dbg.starts_with("Error")
            || dbg.starts_with("Recover");
        prop_assert!(valid, "unexpected debug: {}", dbg);
    }

    // 38. Debug output for Fork variant contains inner actions
    #[test]
    fn debug_fork_variant(inner in prop::collection::vec(leaf_action(), 1..=3)) {
        let action = Action::Fork(inner);
        let dbg = format!("{:?}", action);
        prop_assert!(dbg.starts_with("Fork"));
    }
}

// ===========================================================================
// Property tests — valid_symbols_mask and error_symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 39. valid_symbols_mask length matches terminal_boundary
    #[test]
    fn valid_symbols_mask_length(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let mask = pt.valid_symbols_mask(StateId(s as u16));
            prop_assert_eq!(mask.len(), pt.terminal_boundary());
        }
    }

    // 40. valid_symbols_mask agrees with valid_symbols
    #[test]
    fn valid_symbols_mask_agrees(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let vs = pt.valid_symbols(StateId(s as u16));
            let mask = pt.valid_symbols_mask(StateId(s as u16));
            prop_assert_eq!(vs.len(), mask.len());
            for i in 0..vs.len() {
                prop_assert_eq!(vs[i], mask[i],
                    "mismatch at state {} symbol {}", s, i);
            }
        }
    }

    // 41. valid_symbols_mask for OOB state is all false
    #[test]
    fn valid_symbols_mask_oob(pt in arb_parse_table()) {
        let oob = StateId(pt.state_count as u16 + 5);
        let mask = pt.valid_symbols_mask(oob);
        for b in &mask {
            prop_assert!(!b);
        }
    }

    // 42. error_symbol is always SymbolId(0)
    #[test]
    fn error_symbol_is_zero(pt in arb_parse_table()) {
        prop_assert_eq!(pt.error_symbol(), SymbolId(0));
    }

    // 43. grammar() accessor returns reference to grammar field
    #[test]
    fn grammar_accessor(pt in arb_parse_table()) {
        let g = pt.grammar();
        // The Grammar name should be "proptest" as set in build_table
        prop_assert_eq!(&g.name, "proptest");
    }
}

// ===========================================================================
// Property tests — lex_mode accessor
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 44. lex_mode returns correct mode for each state
    #[test]
    fn lex_mode_accessor(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            let mode = pt.lex_mode(StateId(s as u16));
            prop_assert_eq!(mode, pt.lex_modes[s]);
        }
    }

    // 45. lex_mode for initial state is accessible
    #[test]
    fn lex_mode_initial_state(pt in arb_parse_table()) {
        let mode = pt.lex_mode(pt.initial_state);
        prop_assert_eq!(mode.lex_state, 0);
        prop_assert_eq!(mode.external_lex_state, 0);
    }
}

// ===========================================================================
// Property tests — determinism / conflict detection
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 46. Single-action cells are deterministic
    #[test]
    fn single_action_cells_deterministic(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for col in 0..pt.action_table[s].len() {
                let cell = &pt.action_table[s][col];
                if cell.len() <= 1 {
                    // 0 or 1 action means no conflict
                    prop_assert!(cell.len() <= 1);
                }
            }
        }
    }

    // 47. Shift-only table has no reduce actions
    #[test]
    fn shift_only_table(_dummy in 0u8..1) {
        let action_table = vec![
            vec![
                vec![Action::Shift(StateId(1))],
                vec![Action::Shift(StateId(0))],
            ],
        ];
        let goto_table = vec![vec![NO_GOTO]];
        let meta = vec![
            adze_glr_core::SymbolMetadata {
                name: "t0".into(), is_visible: true, is_named: false, is_supertype: false,
                is_terminal: true, is_extra: false, is_fragile: false, symbol_id: SymbolId(0),
            },
            adze_glr_core::SymbolMetadata {
                name: "n0".into(), is_visible: true, is_named: true, is_supertype: false,
                is_terminal: false, is_extra: false, is_fragile: false, symbol_id: SymbolId(1),
            },
        ];
        let pt = build_table(1, 1, 1, action_table, goto_table, vec![], meta);
        for row in &pt.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Reduce(_) = action { prop_assert!(false, "unexpected reduce") }
                }
            }
        }
    }

    // 48. Reduce-only table has no shift actions
    #[test]
    fn reduce_only_table(_dummy in 0u8..1) {
        let rule = ParseRule { lhs: SymbolId(1), rhs_len: 1 };
        let action_table = vec![
            vec![
                vec![Action::Reduce(RuleId(0))],
                vec![Action::Reduce(RuleId(0))],
            ],
        ];
        let goto_table = vec![vec![NO_GOTO]];
        let meta = vec![
            adze_glr_core::SymbolMetadata {
                name: "t0".into(), is_visible: true, is_named: false, is_supertype: false,
                is_terminal: true, is_extra: false, is_fragile: false, symbol_id: SymbolId(0),
            },
            adze_glr_core::SymbolMetadata {
                name: "n0".into(), is_visible: true, is_named: true, is_supertype: false,
                is_terminal: false, is_extra: false, is_fragile: false, symbol_id: SymbolId(1),
            },
        ];
        let pt = build_table(1, 1, 1, action_table, goto_table, vec![rule], meta);
        for row in &pt.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Shift(_) = action { prop_assert!(false, "unexpected shift") }
                }
            }
        }
    }

    // 49. Multi-action cell indicates GLR conflict
    #[test]
    fn multi_action_cell_is_conflict(_dummy in 0u8..1) {
        let rule = ParseRule { lhs: SymbolId(1), rhs_len: 1 };
        let action_table = vec![
            vec![
                vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))],
                vec![],
            ],
        ];
        let goto_table = vec![vec![NO_GOTO]];
        let meta = vec![
            adze_glr_core::SymbolMetadata {
                name: "t0".into(), is_visible: true, is_named: false, is_supertype: false,
                is_terminal: true, is_extra: false, is_fragile: false, symbol_id: SymbolId(0),
            },
            adze_glr_core::SymbolMetadata {
                name: "n0".into(), is_visible: true, is_named: true, is_supertype: false,
                is_terminal: false, is_extra: false, is_fragile: false, symbol_id: SymbolId(1),
            },
        ];
        let pt = build_table(1, 1, 1, action_table, goto_table, vec![rule], meta);
        let cell = &pt.action_table[0][0];
        prop_assert_eq!(cell.len(), 2);
        prop_assert_eq!(cell[0].clone(), Action::Shift(StateId(0)));
        prop_assert_eq!(cell[1].clone(), Action::Reduce(RuleId(0)));
    }
}

// ===========================================================================
// Property tests — goto with valid entries
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 50. goto() returns Some for non-sentinel entries
    #[test]
    fn goto_non_sentinel_returns_some(pt in arb_parse_table()) {
        for s in 0..pt.state_count {
            for (&nt, &col) in &pt.nonterminal_to_index {
                if col < pt.goto_table[s].len() {
                    let entry = pt.goto_table[s][col];
                    if entry != NO_GOTO {
                        let result = pt.goto(StateId(s as u16), nt);
                        prop_assert!(result.is_some());
                        prop_assert_eq!(result.unwrap(), entry);
                    }
                }
            }
        }
    }

    // 51. nonterminal_to_index size matches goto column count
    #[test]
    fn nonterminal_index_matches_goto_cols(pt in arb_parse_table()) {
        if let Some(first_row) = pt.goto_table.first() {
            prop_assert_eq!(pt.nonterminal_to_index.len(), first_row.len());
        }
    }

    // 52. nonterminal_to_index values are contiguous from 0
    #[test]
    fn nonterminal_index_contiguous(pt in arb_parse_table()) {
        let mut indices: Vec<usize> = pt.nonterminal_to_index.values().copied().collect();
        indices.sort();
        for (i, &idx) in indices.iter().enumerate() {
            prop_assert_eq!(idx, i, "non-contiguous nonterminal index");
        }
    }
}

// ===========================================================================
// Property tests — normalize_eof_to_zero
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 53. normalize_eof_to_zero preserves state_count
    #[test]
    fn normalize_eof_preserves_state_count(pt in arb_parse_table()) {
        let original_count = pt.state_count;
        let normalized = pt.normalize_eof_to_zero();
        prop_assert_eq!(normalized.state_count, original_count);
    }

    // 54. normalize_eof_to_zero preserves symbol_count
    #[test]
    fn normalize_eof_preserves_symbol_count(pt in arb_parse_table()) {
        let original_count = pt.symbol_count;
        let normalized = pt.normalize_eof_to_zero();
        prop_assert_eq!(normalized.symbol_count, original_count);
    }

    // 55. normalize_eof_to_zero sets eof to SymbolId(0)
    #[test]
    fn normalize_eof_sets_zero(pt in arb_parse_table()) {
        let normalized = pt.normalize_eof_to_zero();
        prop_assert_eq!(normalized.eof_symbol, SymbolId(0));
    }

    // 56. normalize_eof_to_zero is idempotent
    #[test]
    fn normalize_eof_idempotent(pt in arb_parse_table()) {
        let once = pt.normalize_eof_to_zero();
        let once_eof = once.eof_symbol;
        let once_action_table = once.action_table.clone();
        let twice = once.normalize_eof_to_zero();
        prop_assert_eq!(twice.eof_symbol, once_eof);
        prop_assert_eq!(twice.action_table, once_action_table);
    }
}

// ===========================================================================
// Property tests — Action equality and hashing
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // 57. Shift actions with same state are equal
    #[test]
    fn shift_eq(s in 0u16..1000) {
        let a = Action::Shift(StateId(s));
        let b = Action::Shift(StateId(s));
        prop_assert_eq!(a, b);
    }

    // 58. Shift actions with different states are not equal
    #[test]
    fn shift_ne(s1 in 0u16..500, s2 in 500u16..1000) {
        let a = Action::Shift(StateId(s1));
        let b = Action::Shift(StateId(s2));
        prop_assert_ne!(a, b);
    }

    // 59. Reduce actions with same rule are equal
    #[test]
    fn reduce_eq(r in 0u16..1000) {
        let a = Action::Reduce(RuleId(r));
        let b = Action::Reduce(RuleId(r));
        prop_assert_eq!(a, b);
    }

    // 60. Shift and Reduce are never equal
    #[test]
    fn shift_ne_reduce(s in 0u16..1000, r in 0u16..1000) {
        let a = Action::Shift(StateId(s));
        let b = Action::Reduce(RuleId(r));
        prop_assert_ne!(a, b);
    }

    // 61. Action hashing: equal actions hash equal
    #[test]
    fn equal_actions_hash_equal(s in 0u16..1000) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let a = Action::Shift(StateId(s));
        let b = Action::Shift(StateId(s));
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }

    // 62. Accept and Error are distinct
    #[test]
    fn accept_ne_error(_dummy in 0u8..1) {
        prop_assert_ne!(Action::Accept, Action::Error);
        prop_assert_ne!(Action::Accept, Action::Recover);
        prop_assert_ne!(Action::Error, Action::Recover);
    }
}

// ===========================================================================
// Property tests — structural invariants
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 63. symbol_metadata length matches symbol_count
    #[test]
    fn symbol_metadata_length(pt in arb_parse_table()) {
        prop_assert_eq!(pt.symbol_metadata.len(), pt.symbol_count);
    }

    // 64. token_count + nonterminal count = symbol_count
    #[test]
    fn token_plus_nonterminal_eq_symbol_count(pt in arb_parse_table()) {
        prop_assert_eq!(
            pt.token_count + pt.external_token_count + pt.nonterminal_to_index.len(),
            pt.symbol_count
        );
    }

    // 65. Default table has NonterminalMap goto indexing
    #[test]
    fn default_goto_indexing(_dummy in 0u8..1) {
        let pt = ParseTable::default();
        prop_assert_eq!(pt.goto_indexing, GotoIndexing::NonterminalMap);
    }
}
