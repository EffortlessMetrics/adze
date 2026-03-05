#![allow(clippy::needless_range_loop)]
//! Property-based tests for adze-tablegen compression properties (v3).
//!
//! 50+ tests covering:
//! 1. Compression preserves actions proptest (8 tests)
//! 2. Compression preserves gotos proptest (5 tests)
//! 3. Compressed size ≤ original proptest (5 tests)
//! 4. Deterministic compression proptest (5 tests)
//! 5. Regular compression tests (8 tests)
//! 6. RLE encoding properties (5 tests)
//! 7. Grammar topology compression (8 tests)
//! 8. Edge cases (6 tests)

use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseRule, ParseTable, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use adze_tablegen::compress::{CompressedGotoEntry, CompressedParseTable, TableCompressor};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable from raw action/goto data.
fn make_parse_table(
    mut actions: Vec<Vec<Vec<Action>>>,
    mut gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start_symbol: SymbolId,
    eof_symbol: SymbolId,
) -> ParseTable {
    let state_count = actions.len().max(1);
    let sym_from_act = actions.first().map(|r| r.len()).unwrap_or(0);
    let sym_from_goto = gotos.first().map(|r| r.len()).unwrap_or(0);
    let min_needed = (start_symbol.0 as usize + 1).max(eof_symbol.0 as usize + 1);
    let symbol_count = sym_from_act.max(sym_from_goto).max(min_needed).max(1);

    if actions.is_empty() {
        actions = vec![vec![vec![]; symbol_count]];
    } else {
        for row in &mut actions {
            if row.len() < symbol_count {
                row.resize_with(symbol_count, Vec::new);
            }
        }
    }
    if gotos.len() < state_count {
        gotos.resize_with(state_count, || vec![INVALID; symbol_count]);
    }
    for row in &mut gotos {
        if row.len() < symbol_count {
            row.resize(symbol_count, INVALID);
        }
    }

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index = BTreeMap::new();
    for col in 0..symbol_count {
        if gotos.iter().any(|r| r[col] != INVALID) {
            nonterminal_to_index.insert(SymbolId(col as u16), col);
        }
    }
    nonterminal_to_index
        .entry(start_symbol)
        .or_insert(start_symbol.0 as usize);

    let eof_idx = eof_symbol.0 as usize;
    let token_count = eof_idx;
    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        state_count
    ];
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (&sid, &idx) in &symbol_to_index {
        index_to_symbol[idx] = sid;
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules,
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count,
        external_token_count: 0,
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
        grammar: Grammar::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build an empty table with given dimensions.
fn make_empty_table(states: usize, terms: usize, nonterms: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms;
    let nonterms_eff = nonterms.max(1);
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];
    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);

    make_parse_table(actions, gotos, vec![], start_symbol, eof_symbol)
}

/// Build a grammar with N tokens and a simple rule.
fn grammar_with_n_tokens(n: usize) -> Grammar {
    let count = n.max(1);
    let mut builder = GrammarBuilder::new("proptest_v3");
    for i in 0..count {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    builder.build()
}

/// Try to build a parse table, returning None on failure.
fn try_build_table(grammar: &Grammar) -> Option<ParseTable> {
    let ff = FirstFollowSets::compute(grammar).ok()?;
    build_lr1_automaton(grammar, &ff).ok()
}

/// Build parse table (panics on failure).
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("LR(1) automaton")
}

// ═══════════════════════════════════════════════════════════════════════════
// Strategies
// ═══════════════════════════════════════════════════════════════════════════

fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

fn flat_action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

fn action_cell_strategy() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(action_strategy(), 0..=3)
}

fn action_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Vec<Action>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(action_cell_strategy(), symbols..=symbols),
            states..=states,
        )
    })
}

fn flat_action_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Action>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(flat_action_strategy(), symbols..=symbols),
            states..=states,
        )
    })
}

fn goto_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Option<StateId>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        let cell = prop_oneof![
            3 => Just(None),
            1 => (0u16..20).prop_map(|s| Some(StateId(s))),
        ];
        prop::collection::vec(
            prop::collection::vec(cell, symbols..=symbols),
            states..=states,
        )
    })
}

/// Strategy for number of tokens in a grammar (2-4).
fn token_count_strategy() -> impl Strategy<Value = usize> {
    2usize..=4
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Compression preserves actions proptest (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Decompressing each cell recovers the first action (or Error for empty).
    #[test]
    fn action_roundtrip_first_action(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                let got = decompress_action(&compressed, state, sym);
                prop_assert_eq!(got, expected, "state={}, sym={}", state, sym);
            }
        }
    }

    /// All states in the original table are preserved in the compressed version.
    #[test]
    fn action_state_count_preserved(table in action_table_strategy(10, 6)) {
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.state_to_row.len(), table.len());
    }

    /// The unique_rows mapping is consistent with the original data.
    #[test]
    fn action_unique_rows_match_originals(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        for (state, &row_idx) in compressed.state_to_row.iter().enumerate() {
            prop_assert!(row_idx < compressed.unique_rows.len());
            prop_assert_eq!(&compressed.unique_rows[row_idx], &table[state]);
        }
    }

    /// Shift actions survive roundtrip through compression.
    #[test]
    fn action_shifts_preserved(
        n_states in 1usize..=6,
        n_symbols in 1usize..=6,
        target in 1u16..50,
    ) {
        let mut table = vec![vec![vec![Action::Error]; n_symbols]; n_states];
        table[0][0] = vec![Action::Shift(StateId(target))];
        let compressed = compress_action_table(&table);
        let got = decompress_action(&compressed, 0, 0);
        prop_assert_eq!(got, Action::Shift(StateId(target)));
    }

    /// Reduce actions survive roundtrip through compression.
    #[test]
    fn action_reduces_preserved(
        n_states in 1usize..=6,
        n_symbols in 1usize..=6,
        rule in 0u16..50,
    ) {
        let mut table = vec![vec![vec![Action::Error]; n_symbols]; n_states];
        table[0][0] = vec![Action::Reduce(RuleId(rule))];
        let compressed = compress_action_table(&table);
        let got = decompress_action(&compressed, 0, 0);
        prop_assert_eq!(got, Action::Reduce(RuleId(rule)));
    }

    /// Accept actions survive roundtrip.
    #[test]
    fn action_accept_preserved(n_states in 1usize..=6, n_symbols in 1usize..=6) {
        let mut table = vec![vec![vec![Action::Error]; n_symbols]; n_states];
        table[0][0] = vec![Action::Accept];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
    }

    /// Empty cells decompress as Error.
    #[test]
    fn action_empty_cells_become_error(table in action_table_strategy(6, 6)) {
        let compressed = compress_action_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, cell) in row.iter().enumerate() {
                if cell.is_empty() {
                    prop_assert_eq!(
                        decompress_action(&compressed, state, sym),
                        Action::Error,
                        "Empty cell at state={}, sym={} should be Error", state, sym
                    );
                }
            }
        }
    }

    /// GLR cells with multiple actions: first action is preserved.
    #[test]
    fn action_glr_multi_first_preserved(
        n_states in 2usize..=5,
        n_symbols in 2usize..=5,
        shift_target in 1u16..50,
        reduce_rule in 0u16..20,
    ) {
        let mut table = vec![vec![vec![Action::Error]; n_symbols]; n_states];
        table[0][0] = vec![
            Action::Shift(StateId(shift_target)),
            Action::Reduce(RuleId(reduce_rule)),
        ];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(
            decompress_action(&compressed, 0, 0),
            Action::Shift(StateId(shift_target))
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Compression preserves gotos proptest (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Every goto cell survives roundtrip.
    #[test]
    fn goto_roundtrip_all_cells(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, &expected) in row.iter().enumerate() {
                let got = decompress_goto(&compressed, state, sym);
                prop_assert_eq!(got, expected, "state={}, sym={}", state, sym);
            }
        }
    }

    /// Specific non-None goto entries are preserved.
    #[test]
    fn goto_non_none_preserved(
        n_states in 2usize..=6,
        n_symbols in 2usize..=6,
        target in 0u16..20,
    ) {
        let mut table: Vec<Vec<Option<StateId>>> = vec![vec![None; n_symbols]; n_states];
        table[0][0] = Some(StateId(target));
        let compressed = compress_goto_table(&table);
        prop_assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(target)));
    }

    /// None entries decompress as None.
    #[test]
    fn goto_none_entries_preserved(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (sym, entry) in row.iter().enumerate() {
                if entry.is_none() {
                    prop_assert_eq!(
                        decompress_goto(&compressed, state, sym),
                        None,
                        "None entry at ({},{}) must stay None", state, sym
                    );
                }
            }
        }
    }

    /// Sparse table: entry count matches non-None cells.
    #[test]
    fn goto_sparse_entry_count(table in goto_table_strategy(8, 8)) {
        let compressed = compress_goto_table(&table);
        let non_none = table.iter().flat_map(|r| r.iter()).filter(|e| e.is_some()).count();
        prop_assert_eq!(
            compressed.entries.len(),
            non_none,
            "Entry count must match non-None cells"
        );
    }

    /// All-None table compresses to zero entries.
    #[test]
    fn goto_all_none_is_empty(n_states in 1usize..=8, n_symbols in 1usize..=8) {
        let table: Vec<Vec<Option<StateId>>> = vec![vec![None; n_symbols]; n_states];
        let compressed = compress_goto_table(&table);
        prop_assert!(compressed.entries.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Compressed size ≤ original proptest (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Row deduplication never produces more unique rows than original states.
    #[test]
    fn size_action_dedup_never_inflates(table in action_table_strategy(12, 8)) {
        let compressed = compress_action_table(&table);
        prop_assert!(compressed.unique_rows.len() <= table.len());
    }

    /// Sparse goto never has more entries than total cells.
    #[test]
    fn size_goto_sparse_never_inflates(table in goto_table_strategy(12, 8)) {
        let compressed = compress_goto_table(&table);
        let cols = if table.is_empty() { 0 } else { table[0].len() };
        let total = table.len() * cols;
        prop_assert!(compressed.entries.len() <= total);
    }

    /// Uniform action tables (all rows identical) compress to 1 unique row.
    #[test]
    fn size_uniform_action_single_row(
        row in prop::collection::vec(action_cell_strategy(), 1..=6),
        n_copies in 2usize..=8,
    ) {
        let table: Vec<Vec<Vec<Action>>> = vec![row; n_copies];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.unique_rows.len(), 1);
    }

    /// All-None goto is maximally compressed (zero entries).
    #[test]
    fn size_all_none_goto_zero(n_states in 1usize..=10, n_symbols in 1usize..=10) {
        let table: Vec<Vec<Option<StateId>>> = vec![vec![None; n_symbols]; n_states];
        let compressed = compress_goto_table(&table);
        prop_assert!(compressed.entries.is_empty());
    }

    /// CompressedParseTable preserves state/symbol counts from ParseTable.
    #[test]
    fn size_compressed_parse_table_dimensions(
        states in 1usize..=8,
        terms in 1usize..=4,
        nonterms in 1usize..=3,
    ) {
        let pt = make_empty_table(states, terms, nonterms);
        let cpt = CompressedParseTable::from_parse_table(&pt);
        prop_assert_eq!(cpt.state_count(), pt.state_count);
        prop_assert_eq!(cpt.symbol_count(), pt.symbol_count);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Deterministic compression proptest (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Compressing the same action table twice yields identical unique_rows.
    #[test]
    fn determinism_action_unique_rows(table in action_table_strategy(8, 6)) {
        let c1 = compress_action_table(&table);
        let c2 = compress_action_table(&table);
        prop_assert_eq!(c1.unique_rows, c2.unique_rows);
    }

    /// Compressing the same action table twice yields identical state_to_row.
    #[test]
    fn determinism_action_state_mapping(table in action_table_strategy(8, 6)) {
        let c1 = compress_action_table(&table);
        let c2 = compress_action_table(&table);
        prop_assert_eq!(c1.state_to_row, c2.state_to_row);
    }

    /// Compressing the same goto table twice yields identical entries.
    #[test]
    fn determinism_goto_entries(table in goto_table_strategy(8, 6)) {
        let c1 = compress_goto_table(&table);
        let c2 = compress_goto_table(&table);
        prop_assert_eq!(c1.entries, c2.entries);
    }

    /// BitPackedActionTable decompress is deterministic across two packs.
    #[test]
    fn determinism_bitpacked_decompress(table in flat_action_table_strategy(4, 4)) {
        let p1 = BitPackedActionTable::from_table(&table);
        let p2 = BitPackedActionTable::from_table(&table);
        for (s, row) in table.iter().enumerate() {
            for sym in 0..row.len() {
                prop_assert_eq!(p1.decompress(s, sym), p2.decompress(s, sym));
            }
        }
    }

    /// GrammarBuilder produces the same grammar twice (determinism).
    #[test]
    fn determinism_grammar_builder(n_tokens in 1usize..=4) {
        let g1 = grammar_with_n_tokens(n_tokens);
        let g2 = grammar_with_n_tokens(n_tokens);
        // Same token count and rule count
        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Regular compression tests (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regular_row_dedup_with_two_identical_rows() {
    let row = vec![vec![Action::Shift(StateId(1))], vec![Action::Error]];
    let table = vec![row.clone(), row];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(compressed.state_to_row, [0, 0]);
}

#[test]
fn regular_row_dedup_with_distinct_rows() {
    let table = vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Reduce(RuleId(0))]],
    ];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 2);
}

#[test]
fn regular_sparse_goto_single_entry() {
    let table = vec![vec![None, Some(StateId(5)), None]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 0, 1), Some(StateId(5)));
}

#[test]
fn regular_sparse_goto_all_populated() {
    let table = vec![vec![Some(StateId(1)), Some(StateId(2)), Some(StateId(3))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 3);
}

#[test]
fn regular_empty_action_table() {
    let table: Vec<Vec<Vec<Action>>> = vec![vec![vec![]]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
}

#[test]
fn regular_accept_in_action_table() {
    let table = vec![vec![vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
}

#[test]
fn regular_compressor_new_has_defaults() {
    let compressor = TableCompressor::new();
    // TableCompressor::new() should not panic and produce a valid compressor.
    let table: Vec<Vec<Vec<Action>>> = vec![vec![vec![Action::Error]; 3]; 2];
    let symbol_to_index: BTreeMap<SymbolId, usize> =
        (0..3).map(|i| (SymbolId(i as u16), i)).collect();
    let result = compressor.compress_action_table_small(&table, &symbol_to_index);
    assert!(result.is_ok());
}

#[test]
fn regular_bitpacked_all_errors() {
    let table: Vec<Vec<Action>> = vec![vec![Action::Error; 4]; 3];
    let packed = BitPackedActionTable::from_table(&table);
    for s in 0..3 {
        for sym in 0..4 {
            assert_eq!(packed.decompress(s, sym), Action::Error);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. RLE encoding properties (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rle_single_entry_is_single_variant() {
    let entry = CompressedGotoEntry::Single(42);
    match entry {
        CompressedGotoEntry::Single(v) => assert_eq!(v, 42),
        CompressedGotoEntry::RunLength { .. } => panic!("Expected Single"),
    }
}

#[test]
fn rle_run_length_stores_count() {
    let entry = CompressedGotoEntry::RunLength { state: 7, count: 5 };
    match entry {
        CompressedGotoEntry::RunLength { state, count } => {
            assert_eq!(state, 7);
            assert_eq!(count, 5);
        }
        CompressedGotoEntry::Single(_) => panic!("Expected RunLength"),
    }
}

#[test]
fn rle_goto_table_compresses_sparse_data() {
    // The compress_goto_table in the compression module uses HashMap-based
    // sparse representation; verify sparse property.
    let table = vec![
        vec![None, None, None, Some(StateId(1)), None],
        vec![None, None, None, None, None],
        vec![None, Some(StateId(2)), None, None, None],
    ];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 2);
}

#[test]
fn rle_goto_table_with_dense_row() {
    let table = vec![vec![
        Some(StateId(0)),
        Some(StateId(1)),
        Some(StateId(2)),
        Some(StateId(3)),
    ]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 4);
    for sym in 0..4 {
        assert_eq!(
            decompress_goto(&compressed, 0, sym),
            Some(StateId(sym as u16))
        );
    }
}

#[test]
fn rle_compressed_goto_entry_debug() {
    let single = CompressedGotoEntry::Single(10);
    let run = CompressedGotoEntry::RunLength { state: 3, count: 2 };
    // Debug formatting should not panic
    let _ = format!("{single:?}");
    let _ = format!("{run:?}");
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Grammar topology compression (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn topology_single_token_grammar() {
    let grammar = grammar_with_n_tokens(1);
    if let Some(pt) = try_build_table(&grammar) {
        let compressed = compress_action_table(&pt.action_table);
        assert!(compressed.unique_rows.len() <= pt.state_count);
    }
}

#[test]
fn topology_two_token_grammar() {
    let grammar = GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a", "b"])
        .start("root")
        .build();
    let pt = build_table(&grammar);
    let compressed = compress_action_table(&pt.action_table);
    // Dedup should not inflate
    assert!(compressed.unique_rows.len() <= pt.state_count);
}

#[test]
fn topology_alternative_rules() {
    let grammar = GrammarBuilder::new("alt_rules")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["x"])
        .rule("root", vec!["y"])
        .start("root")
        .build();
    if let Some(pt) = try_build_table(&grammar) {
        let compressed = compress_action_table(&pt.action_table);
        assert!(compressed.unique_rows.len() <= pt.state_count);
    }
}

#[test]
fn topology_chain_grammar_compresses() {
    let grammar = GrammarBuilder::new("chain")
        .token("tok", "t")
        .rule("root", vec!["mid"])
        .rule("mid", vec!["tok"])
        .start("root")
        .build();
    if let Some(pt) = try_build_table(&grammar) {
        let cpt = CompressedParseTable::from_parse_table(&pt);
        assert_eq!(cpt.state_count(), pt.state_count);
    }
}

#[test]
fn topology_three_token_sequence() {
    let grammar = GrammarBuilder::new("seq3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("root", vec!["a", "b", "c"])
        .start("root")
        .build();
    let pt = build_table(&grammar);
    let goto_sparse: Vec<Vec<Option<StateId>>> = pt
        .goto_table
        .iter()
        .map(|row| {
            row.iter()
                .map(|&s| if s == INVALID { None } else { Some(s) })
                .collect()
        })
        .collect();
    let compressed = compress_goto_table(&goto_sparse);
    // Sparse representation should have fewer entries than total cells
    let total_cells = goto_sparse.len() * goto_sparse.first().map(|r| r.len()).unwrap_or(0);
    assert!(compressed.entries.len() <= total_cells);
}

#[test]
fn topology_grammar_table_state_count_consistent() {
    let grammar = grammar_with_n_tokens(3);
    if let Some(pt) = try_build_table(&grammar) {
        assert_eq!(pt.action_table.len(), pt.state_count);
        assert_eq!(pt.goto_table.len(), pt.state_count);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Random small grammars compress without panic.
    #[test]
    fn topology_random_grammar_compresses(n_tokens in token_count_strategy()) {
        let grammar = grammar_with_n_tokens(n_tokens);
        if let Some(pt) = try_build_table(&grammar) {
            let compressed = compress_action_table(&pt.action_table);
            prop_assert!(compressed.unique_rows.len() <= pt.state_count);
        }
    }

    /// CompressedParseTable from grammar-built table preserves dimensions.
    #[test]
    fn topology_compressed_parse_table_from_grammar(n_tokens in token_count_strategy()) {
        let grammar = grammar_with_n_tokens(n_tokens);
        if let Some(pt) = try_build_table(&grammar) {
            let cpt = CompressedParseTable::from_parse_table(&pt);
            prop_assert_eq!(cpt.state_count(), pt.state_count);
            prop_assert_eq!(cpt.symbol_count(), pt.symbol_count);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Edge cases (6 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn edge_single_cell_action_table() {
    let table = vec![vec![vec![Action::Shift(StateId(1))]]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1))
    );
}

#[test]
fn edge_single_cell_goto_table() {
    let table = vec![vec![Some(StateId(42))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 1);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(42)));
}

#[test]
fn edge_large_state_ids_in_goto() {
    let large = StateId(u16::MAX - 1);
    let table = vec![vec![Some(large)]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(large));
}

#[test]
fn edge_many_duplicate_rows() {
    let row = vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(0))],
        vec![Action::Error],
    ];
    let table: Vec<Vec<Vec<Action>>> = vec![row; 20];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    for &idx in &compressed.state_to_row {
        assert_eq!(idx, 0);
    }
}

#[test]
fn edge_all_distinct_rows() {
    let table: Vec<Vec<Vec<Action>>> = (0..5)
        .map(|i| vec![vec![Action::Shift(StateId(i as u16 + 1))]])
        .collect();
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 5);
}

#[test]
fn edge_compressed_parse_table_for_testing() {
    let cpt = CompressedParseTable::new_for_testing(10, 5);
    assert_eq!(cpt.symbol_count(), 10);
    assert_eq!(cpt.state_count(), 5);
}
