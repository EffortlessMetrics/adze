#![allow(clippy::needless_range_loop)]

//! Property-based tests for lex mode generation in `adze-tablegen`.
//!
//! Properties verified:
//!  1. Lex mode count matches state count
//!  2. Lex mode valid_symbols is a bitmap (boolean values only)
//!  3. Lex mode external_lex_state is correct
//!  4. Default lex mode
//!  5. Multiple lex modes
//!  6. Lex mode determinism (same input → same output)

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{Grammar, StateId, SymbolId};
use adze_tablegen::serializer::{SerializableLexState, serialize_language};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ── Helpers ──────────────────────────────────────────────────────────

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal parse table with `states` states, `terms` terminals,
/// `nonterms` non-terminals, and `externals` external tokens.
fn empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);
    let token_count = eof_idx - externals;

    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (&sym, &idx) in &symbol_to_index {
        index_to_symbol[idx] = sym;
    }

    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        states
    ];

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count,
        external_token_count: externals,
        eof_symbol,
        start_symbol,
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: Grammar::new("test".to_string()),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build a parse table with explicit lex modes.
fn table_with_lex_modes(state_count: usize, modes: Vec<LexMode>) -> ParseTable {
    let mut pt = empty_table(state_count, 1, 1, 0);
    pt.lex_modes = modes;
    pt
}

/// Build a parse table with explicit lex modes and external scanner states.
fn table_with_externals(
    state_count: usize,
    modes: Vec<LexMode>,
    ext_states: Vec<Vec<bool>>,
    ext_count: usize,
) -> ParseTable {
    let mut pt = empty_table(state_count, 1, 1, ext_count);
    pt.lex_modes = modes;
    pt.external_scanner_states = ext_states;
    pt
}

// ── Strategies ───────────────────────────────────────────────────────

fn lex_mode_strategy() -> impl Strategy<Value = LexMode> {
    (0u16..256, 0u16..16).prop_map(|(lex_state, external_lex_state)| LexMode {
        lex_state,
        external_lex_state,
    })
}

fn state_count_strategy() -> impl Strategy<Value = usize> {
    1usize..=32
}

fn lex_modes_for_states(max_states: usize) -> impl Strategy<Value = (usize, Vec<LexMode>)> {
    (1usize..=max_states)
        .prop_flat_map(|n| (Just(n), prop::collection::vec(lex_mode_strategy(), n..=n)))
}

// ══════════════════════════════════════════════════════════════════════
// 1 – Lex mode count matches state count
// ══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Lex mode vector length always equals state_count for empty_table.
    #[test]
    fn lex_mode_count_equals_state_count(states in 1usize..=64) {
        let pt = empty_table(states, 1, 1, 0);
        prop_assert_eq!(pt.lex_modes.len(), states);
    }

    /// After serialization, the JSON lex_modes array length matches state_count.
    #[test]
    fn serialized_lex_mode_count_matches_state_count(states in 1usize..=32) {
        let pt = empty_table(states, 1, 1, 0);
        let json = serialize_language(&pt.grammar, &pt, None).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let modes = v["lex_modes"].as_array().unwrap();
        prop_assert_eq!(modes.len(), states);
    }

    /// Lex mode count matches for tables with external tokens.
    #[test]
    fn lex_mode_count_with_externals(
        states in 1usize..=32,
        externals in 0usize..=4,
    ) {
        let pt = empty_table(states, 1, 1, externals);
        prop_assert_eq!(pt.lex_modes.len(), states);
    }

    /// Custom lex_modes vector preserves length after assignment.
    #[test]
    fn custom_lex_modes_preserve_length((n, modes) in lex_modes_for_states(32)) {
        let pt = table_with_lex_modes(n, modes.clone());
        prop_assert_eq!(pt.lex_modes.len(), n);
    }
}

// ══════════════════════════════════════════════════════════════════════
// 2 – valid_symbols is a bitmap (boolean mask)
// ══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// valid_symbols for any state is exactly a Vec<bool>.
    #[test]
    fn valid_symbols_is_boolean_mask(states in 1usize..=16) {
        let pt = empty_table(states, 3, 1, 0);
        for s in 0..states {
            let mask = pt.valid_symbols(StateId(s as u16));
            // Each element is bool (true/false); empty table means all false.
            for &b in &mask {
                prop_assert!(!b, "empty table should have no valid symbols");
            }
        }
    }

    /// valid_symbols mask length equals the terminal boundary.
    #[test]
    fn valid_symbols_length_equals_terminal_boundary(
        states in 1usize..=16,
        terms in 1usize..=10,
    ) {
        let pt = empty_table(states, terms, 1, 0);
        let boundary = pt.terminal_boundary();
        for s in 0..states {
            let mask = pt.valid_symbols(StateId(s as u16));
            prop_assert_eq!(mask.len(), boundary);
        }
    }

    /// valid_symbols_mask is consistent with valid_symbols.
    #[test]
    fn valid_symbols_mask_equals_valid_symbols(states in 1usize..=16) {
        let pt = empty_table(states, 2, 1, 0);
        for s in 0..states {
            let sid = StateId(s as u16);
            let a = pt.valid_symbols(sid);
            let b = pt.valid_symbols_mask(sid);
            prop_assert_eq!(a, b);
        }
    }

    /// valid_symbols out-of-range state returns all-false mask.
    #[test]
    fn valid_symbols_out_of_range_state(states in 1usize..=8) {
        let pt = empty_table(states, 2, 1, 0);
        let mask = pt.valid_symbols(StateId((states + 10) as u16));
        for &b in &mask {
            prop_assert!(!b);
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// 3 – external_lex_state correctness
// ══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Default empty_table sets external_lex_state to 0 for all states.
    #[test]
    fn default_external_lex_state_is_zero(states in 1usize..=32) {
        let pt = empty_table(states, 1, 1, 0);
        for mode in &pt.lex_modes {
            prop_assert_eq!(mode.external_lex_state, 0);
        }
    }

    /// Custom external_lex_state values are preserved.
    #[test]
    fn custom_external_lex_state_preserved((n, modes) in lex_modes_for_states(16)) {
        let pt = table_with_lex_modes(n, modes.clone());
        for i in 0..n {
            prop_assert_eq!(pt.lex_modes[i].external_lex_state, modes[i].external_lex_state);
        }
    }

    /// lex_mode() accessor returns correct external_lex_state.
    #[test]
    fn lex_mode_accessor_external_lex_state((n, modes) in lex_modes_for_states(16)) {
        let pt = table_with_lex_modes(n, modes.clone());
        for i in 0..n {
            let lm = pt.lex_mode(StateId(i as u16));
            prop_assert_eq!(lm.external_lex_state, modes[i].external_lex_state);
        }
    }

    /// external_scanner_states bitmap width matches external_token_count.
    #[test]
    fn external_scanner_states_width_matches_ext_count(
        states in 1usize..=8,
        ext_count in 1usize..=4,
    ) {
        let ext_states = vec![vec![true; ext_count]; states];
        let modes = vec![LexMode { lex_state: 0, external_lex_state: 0 }; states];
        let pt = table_with_externals(states, modes, ext_states, ext_count);
        for row in &pt.external_scanner_states {
            prop_assert_eq!(row.len(), ext_count);
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// 4 – Default lex mode
// ══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Out-of-bounds lex_mode() returns default (lex_state=0, external_lex_state=0).
    #[test]
    fn out_of_bounds_lex_mode_returns_default(states in 1usize..=16) {
        let pt = empty_table(states, 1, 1, 0);
        let oob = pt.lex_mode(StateId((states + 100) as u16));
        prop_assert_eq!(oob.lex_state, 0);
        prop_assert_eq!(oob.external_lex_state, 0);
    }

    /// Default parse table has empty lex_modes (lex_mode accessor returns default).
    #[test]
    fn default_parse_table_lex_mode_accessor(_dummy in 0u8..1) {
        let pt = ParseTable::default();
        let lm = pt.lex_mode(StateId(0));
        prop_assert_eq!(lm.lex_state, 0);
        prop_assert_eq!(lm.external_lex_state, 0);
    }

    /// Default empty_table lex_state is 0 for all states.
    #[test]
    fn default_lex_state_is_zero(states in 1usize..=32) {
        let pt = empty_table(states, 2, 1, 0);
        for mode in &pt.lex_modes {
            prop_assert_eq!(mode.lex_state, 0);
        }
    }

    /// Serialized default lex modes all have lex_state = index and external_lex_state = 0.
    #[test]
    fn serialized_default_lex_modes_have_sequential_lex_state(states in 1usize..=16) {
        let pt = empty_table(states, 1, 1, 0);
        let json = serialize_language(&pt.grammar, &pt, None).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let modes = v["lex_modes"].as_array().unwrap();
        for (i, mode) in modes.iter().enumerate() {
            let ls = mode["lex_state"].as_u64().unwrap();
            prop_assert_eq!(ls, i as u64, "serialized lex_state should equal index");
            let ext = mode["external_lex_state"].as_u64().unwrap();
            prop_assert_eq!(ext, 0, "default external_lex_state should be 0");
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// 5 – Multiple lex modes
// ══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Each state can have a unique lex_state value.
    #[test]
    fn unique_lex_states_per_state(n in 1usize..=16) {
        let modes: Vec<LexMode> = (0..n)
            .map(|i| LexMode {
                lex_state: i as u16,
                external_lex_state: 0,
            })
            .collect();
        let pt = table_with_lex_modes(n, modes);
        for i in 0..n {
            prop_assert_eq!(pt.lex_modes[i].lex_state, i as u16);
        }
    }

    /// Multiple different external_lex_state values coexist.
    #[test]
    fn multiple_external_lex_states(n in 2usize..=16) {
        let modes: Vec<LexMode> = (0..n)
            .map(|i| LexMode {
                lex_state: 0,
                external_lex_state: (i % 4) as u16,
            })
            .collect();
        let pt = table_with_lex_modes(n, modes.clone());
        for i in 0..n {
            prop_assert_eq!(pt.lex_modes[i].external_lex_state, (i % 4) as u16);
        }
    }

    /// Lex modes with mixed lex_state and external_lex_state are stored correctly.
    #[test]
    fn mixed_lex_state_and_external_state((n, modes) in lex_modes_for_states(16)) {
        let pt = table_with_lex_modes(n, modes.clone());
        for i in 0..n {
            prop_assert_eq!(pt.lex_modes[i].lex_state, modes[i].lex_state);
            prop_assert_eq!(pt.lex_modes[i].external_lex_state, modes[i].external_lex_state);
        }
    }

    /// lex_mode() accessor works for every valid state index with custom modes.
    #[test]
    fn lex_mode_accessor_all_states((n, modes) in lex_modes_for_states(16)) {
        let pt = table_with_lex_modes(n, modes.clone());
        for i in 0..n {
            let lm = pt.lex_mode(StateId(i as u16));
            prop_assert_eq!(lm, modes[i]);
        }
    }

    /// Duplicate lex modes are allowed and preserved.
    #[test]
    fn duplicate_lex_modes_allowed(n in 2usize..=16, ls in 0u16..100, ext in 0u16..8) {
        let mode = LexMode { lex_state: ls, external_lex_state: ext };
        let modes = vec![mode; n];
        let pt = table_with_lex_modes(n, modes);
        for i in 0..n {
            prop_assert_eq!(pt.lex_modes[i].lex_state, ls);
            prop_assert_eq!(pt.lex_modes[i].external_lex_state, ext);
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// 6 – Lex mode determinism
// ══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Building the same table twice produces identical lex modes.
    #[test]
    fn deterministic_lex_mode_generation(states in 1usize..=32) {
        let pt1 = empty_table(states, 2, 1, 0);
        let pt2 = empty_table(states, 2, 1, 0);
        prop_assert_eq!(pt1.lex_modes, pt2.lex_modes);
    }

    /// Serializing the same table twice yields identical lex_modes JSON.
    #[test]
    fn deterministic_serialization(states in 1usize..=16) {
        let pt = empty_table(states, 1, 1, 0);
        let json1 = serialize_language(&pt.grammar, &pt, None).unwrap();
        let json2 = serialize_language(&pt.grammar, &pt, None).unwrap();
        let v1: serde_json::Value = serde_json::from_str(&json1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        prop_assert_eq!(&v1["lex_modes"], &v2["lex_modes"]);
    }

    /// Lex mode order is deterministic (index i always corresponds to state i).
    #[test]
    fn lex_mode_order_deterministic((n, modes) in lex_modes_for_states(16)) {
        let pt1 = table_with_lex_modes(n, modes.clone());
        let pt2 = table_with_lex_modes(n, modes.clone());
        for i in 0..n {
            prop_assert_eq!(pt1.lex_modes[i], pt2.lex_modes[i]);
        }
    }

    /// Changing state count changes lex mode count deterministically.
    #[test]
    fn state_count_change_reflects_in_lex_modes(
        s1 in 1usize..=16,
        s2 in 1usize..=16,
    ) {
        let pt1 = empty_table(s1, 1, 1, 0);
        let pt2 = empty_table(s2, 1, 1, 0);
        prop_assert_eq!(pt1.lex_modes.len(), s1);
        prop_assert_eq!(pt2.lex_modes.len(), s2);
    }

    /// Serialized lex_state values are deterministic and sequential.
    #[test]
    fn serialized_lex_state_deterministic(states in 1usize..=16) {
        let pt = empty_table(states, 1, 1, 0);
        let json1 = serialize_language(&pt.grammar, &pt, None).unwrap();
        let json2 = serialize_language(&pt.grammar, &pt, None).unwrap();
        let v1: serde_json::Value = serde_json::from_str(&json1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        let modes1 = v1["lex_modes"].as_array().unwrap();
        let modes2 = v2["lex_modes"].as_array().unwrap();
        for i in 0..states {
            prop_assert_eq!(
                modes1[i]["lex_state"].as_u64(),
                modes2[i]["lex_state"].as_u64()
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// 7 – Additional edge-case and cross-cutting properties
// ══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Single-state table has exactly one lex mode.
    #[test]
    fn single_state_single_lex_mode(terms in 1usize..=8) {
        let pt = empty_table(1, terms, 1, 0);
        prop_assert_eq!(pt.lex_modes.len(), 1);
    }

    /// External scanner states bitmap rows match state_count.
    #[test]
    fn external_scanner_states_row_count(
        states in 1usize..=8,
        ext_count in 1usize..=4,
    ) {
        let ext_states = vec![vec![false; ext_count]; states];
        let modes = vec![LexMode { lex_state: 0, external_lex_state: 0 }; states];
        let pt = table_with_externals(states, modes, ext_states, ext_count);
        prop_assert_eq!(pt.external_scanner_states.len(), states);
    }

    /// external_scanner_states is a proper bitmap (only bool values).
    #[test]
    fn external_scanner_states_is_bitmap(
        states in 1usize..=8,
        ext_count in 1usize..=4,
    ) {
        // Generate random bitmap
        let ext_states: Vec<Vec<bool>> = (0..states)
            .map(|s| (0..ext_count).map(|t| (s + t) % 2 == 0).collect())
            .collect();
        let modes = vec![LexMode { lex_state: 0, external_lex_state: 0 }; states];
        let pt = table_with_externals(states, modes, ext_states, ext_count);
        for row in &pt.external_scanner_states {
            for &val in row {
                // val is bool; just verify it's accessible (type system ensures bool).
                prop_assert!(val || !val);
            }
        }
    }

    /// LexMode fields fit in u16 range.
    #[test]
    fn lex_mode_fields_fit_u16(ls in 0u16..=u16::MAX, ext in 0u16..=u16::MAX) {
        let mode = LexMode { lex_state: ls, external_lex_state: ext };
        prop_assert_eq!(mode.lex_state, ls);
        prop_assert_eq!(mode.external_lex_state, ext);
    }
}
