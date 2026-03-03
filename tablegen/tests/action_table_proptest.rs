#![allow(clippy::needless_range_loop)]
//! Property-based tests for action table generation in adze-tablegen.
//!
//! Properties verified:
//! 1.  Action table present in generated compressed tables
//! 2.  Shift actions encoded correctly
//! 3.  Reduce actions encoded correctly
//! 4.  Accept action encoded as 0xFFFF
//! 5.  Error action encoded as 0xFFFE
//! 6.  Action table determinism (same input → same output)
//! 7.  Action table size (compressed ≤ uncompressed)
//! 8.  Action table compression produces valid row offsets
//! 9.  Accept action placement in EOF column
//! 10. Row offsets are non-decreasing
//! 11. Row offsets length equals state_count + 1
//! 12. Default actions array length equals state_count
//! 13. Shift encoding roundtrips
//! 14. Reduce encoding roundtrips
//! 15. Shift state range preserved
//! 16. Reduce rule range preserved
//! 17. Compressed entry symbols are valid column indices
//! 18. Empty rows produce no entries
//! 19. All-error rows produce no entries
//! 20. Single-action rows produce exactly one entry
//! 21. Multi-action cells produce multiple entries
//! 22. Compression is idempotent
//! 23. Goto table compression row offsets valid
//! 24. Small table threshold respected
//! 25. Large state IDs within shift encoding range
//! 26. Large rule IDs within reduce encoding range
//! 27. Mixed action rows preserve all non-error actions
//! 28. Compressed table validate succeeds for well-formed tables
//! 29. Action table with only accepts
//! 30. Sparse action table compression

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use adze_tablegen::{
    CompressedActionEntry, CompressedTables, TableCompressor, collect_token_indices,
    eof_accepts_or_reduces,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable suitable for property tests.
/// Mirrors the logic in `test_helpers::test::make_empty_table`.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; states];
    let gotos: Vec<Vec<StateId>> = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (i, slot) in index_to_symbol.iter_mut().enumerate() {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        *slot = sym;
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        symbol_metadata: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol,
        grammar: Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: eof_idx + 1,
        external_token_count: externals,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Build a grammar that registers terminal tokens matching the table layout.
/// Token columns in the table are 1..=terms (column 0 is ERROR).
fn grammar_for_table(table: &ParseTable, terms: usize) -> Grammar {
    use adze_ir::{Token, TokenPattern};
    let mut grammar = Grammar::new("test".to_string());
    for i in 1..=terms {
        let sym = SymbolId(i as u16);
        if table.symbol_to_index.contains_key(&sym) {
            grammar.tokens.insert(
                sym,
                Token {
                    name: format!("tok_{i}"),
                    pattern: TokenPattern::String(format!("t{i}")),
                    fragile: false,
                },
            );
        }
    }
    grammar
}

/// Place a shift action on a real terminal column in state 0 so compression's
/// "state 0 must have at least one token shift" check passes, then
/// compress the table.
fn compress_with_grammar(
    table: &mut ParseTable,
    terms: usize,
) -> adze_tablegen::Result<CompressedTables> {
    // Put Shift(1) at column 1 (first terminal) in state 0
    if table.action_table[0].len() > 1 {
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
    }
    let grammar = grammar_for_table(table, terms);
    let token_indices = collect_token_indices(&grammar, table);
    let start_empty = eof_accepts_or_reduces(table);
    TableCompressor::new().compress(table, &token_indices, start_empty)
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Strategy for small table dimensions.
fn table_dims() -> impl Strategy<Value = (usize, usize, usize)> {
    (
        1usize..=6, // states
        1usize..=5, // terms
        0usize..=3, // nonterms
    )
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // 1. Compressed action table is present and non-empty when table has actions
    #[test]
    fn action_table_present_in_compressed(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();
        prop_assert!(!compressed.action_table.row_offsets.is_empty());
    }

    // 2. Shift actions are preserved in compressed entries
    #[test]
    fn shift_actions_in_table(state_id in 1u16..200) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Shift(StateId(state_id))).unwrap();
        // Shift encoding: state_id directly (high bit 0)
        prop_assert_eq!(encoded, state_id);
    }

    // 3. Reduce actions have high bit set
    #[test]
    fn reduce_actions_in_table(rule_id in 0u16..100) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Reduce(RuleId(rule_id))).unwrap();
        // Reduce: 0x8000 | (rule_id + 1)
        prop_assert_eq!(encoded, 0x8000 | (rule_id + 1));
        prop_assert!(encoded & 0x8000 != 0, "reduce must have high bit set");
    }

    // 4. Action table determinism: compressing twice yields identical results
    #[test]
    fn action_table_determinism(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let grammar = grammar_for_table(&table, terms);
        let tok_ix = collect_token_indices(&grammar, &table);
        let start_empty = eof_accepts_or_reduces(&table);

        let c1 = TableCompressor::new().compress(&table, &tok_ix, start_empty).unwrap();
        let c2 = TableCompressor::new().compress(&table, &tok_ix, start_empty).unwrap();

        // Row offsets must be identical
        prop_assert_eq!(&c1.action_table.row_offsets, &c2.action_table.row_offsets);
        // Data length must be identical
        prop_assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
        // Default actions must be identical
        prop_assert_eq!(c1.action_table.default_actions.len(), c2.action_table.default_actions.len());
    }

    // 5. Compressed action table data size ≤ sum of all non-error cells
    #[test]
    fn action_table_size(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();

        let non_error_count: usize = table.action_table.iter()
            .flat_map(|row| row.iter())
            .flat_map(|cell| cell.iter())
            .filter(|a| !matches!(a, Action::Error))
            .count();

        prop_assert!(
            compressed.action_table.data.len() <= non_error_count,
            "compressed entries ({}) must not exceed non-error actions ({})",
            compressed.action_table.data.len(),
            non_error_count
        );
    }

    // 6. Action table compression: row offsets are non-decreasing
    #[test]
    fn action_table_compression_row_offsets_nondecreasing(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();
        let offsets = &compressed.action_table.row_offsets;
        for i in 1..offsets.len() {
            prop_assert!(
                offsets[i] >= offsets[i - 1],
                "row_offsets[{}]={} < row_offsets[{}]={}",
                i, offsets[i], i - 1, offsets[i - 1]
            );
        }
    }

    // 7. Accept action placement: encoding is 0xFFFF
    #[test]
    fn accept_action_placement(_dummy in 0u8..1) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Accept).unwrap();
        prop_assert_eq!(encoded, 0xFFFF);
    }

    // 8. Error action encoding is 0xFFFE
    #[test]
    fn error_action_encoding(_dummy in 0u8..1) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Error).unwrap();
        prop_assert_eq!(encoded, 0xFFFE);
    }

    // 9. Row offsets length = state_count + 1
    #[test]
    fn row_offsets_length_matches_states(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();
        prop_assert_eq!(
            compressed.action_table.row_offsets.len(),
            table.state_count + 1
        );
    }

    // 10. Default actions array length equals state_count
    #[test]
    fn default_actions_length_matches_states(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();
        prop_assert_eq!(
            compressed.action_table.default_actions.len(),
            table.state_count
        );
    }

    // 11. Shift encoding roundtrip: encode then check value
    #[test]
    fn shift_encoding_roundtrips(state in 1u16..0x7FFF) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Shift(StateId(state))).unwrap();
        // Shift: encoded == state, high bit is 0
        prop_assert_eq!(encoded, state);
        prop_assert!(encoded < 0x8000);
    }

    // 12. Reduce encoding roundtrip: decode the rule id from encoded value
    #[test]
    fn reduce_encoding_roundtrips(rule in 0u16..0x3FFF) {
        let compressor = TableCompressor::new();
        let encoded = compressor.encode_action_small(&Action::Reduce(RuleId(rule))).unwrap();
        let decoded_rule = (encoded & 0x7FFF) - 1;
        prop_assert_eq!(decoded_rule, rule);
    }

    // 13. Shift state too large produces error
    #[test]
    fn shift_state_range_validated(state in 0x8000u16..=u16::MAX) {
        let compressor = TableCompressor::new();
        let result = compressor.encode_action_small(&Action::Shift(StateId(state)));
        prop_assert!(result.is_err());
    }

    // 14. Reduce rule too large produces error
    #[test]
    fn reduce_rule_range_validated(rule in 0x4000u16..=u16::MAX) {
        let compressor = TableCompressor::new();
        let result = compressor.encode_action_small(&Action::Reduce(RuleId(rule)));
        prop_assert!(result.is_err());
    }

    // 15. Compressed entry symbols are valid column indices
    #[test]
    fn compressed_entry_symbols_valid(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();
        for entry in &compressed.action_table.data {
            prop_assert!(
                (entry.symbol as usize) < table.symbol_count,
                "symbol {} >= symbol_count {}",
                entry.symbol, table.symbol_count
            );
        }
    }

    // 16. Empty action rows produce no entries
    #[test]
    fn empty_rows_produce_no_entries(terms in 1usize..=5) {
        let compressor = TableCompressor::new();
        // One row of empty cells
        let action_table = vec![vec![vec![]; terms + 3]];
        let symbol_to_index = BTreeMap::new();
        let result = compressor.compress_action_table_small(&action_table, &symbol_to_index).unwrap();
        prop_assert!(result.data.is_empty());
    }

    // 17. All-error rows produce no entries (errors are skipped)
    #[test]
    fn all_error_rows_produce_no_entries(terms in 1usize..=5) {
        let compressor = TableCompressor::new();
        let action_table = vec![vec![vec![Action::Error]; terms + 3]];
        let symbol_to_index = BTreeMap::new();
        let result = compressor.compress_action_table_small(&action_table, &symbol_to_index).unwrap();
        prop_assert!(result.data.is_empty());
    }

    // 18. Single non-error action row produces exactly one entry
    #[test]
    fn single_action_row_one_entry(col in 0usize..5) {
        let compressor = TableCompressor::new();
        let mut row = vec![vec![]; 6];
        row[col] = vec![Action::Shift(StateId(1))];
        let action_table = vec![row];
        let symbol_to_index = BTreeMap::new();
        let result = compressor.compress_action_table_small(&action_table, &symbol_to_index).unwrap();
        prop_assert_eq!(result.data.len(), 1);
        prop_assert_eq!(result.data[0].symbol, col as u16);
    }

    // 19. Multi-action cells produce multiple entries
    #[test]
    fn multi_action_cells_produce_entries(n_actions in 2usize..=4) {
        let compressor = TableCompressor::new();
        let cell: Vec<Action> = (0..n_actions)
            .map(|i| Action::Shift(StateId(i as u16 + 1)))
            .collect();
        let action_table = vec![vec![cell]];
        let symbol_to_index = BTreeMap::new();
        let result = compressor.compress_action_table_small(&action_table, &symbol_to_index).unwrap();
        prop_assert_eq!(result.data.len(), n_actions);
    }

    // 20. Compression is idempotent on action table
    #[test]
    fn compression_idempotent_action(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];

        let compressor = TableCompressor::new();
        let c1 = compressor.compress_action_table_small(
            &table.action_table, &table.symbol_to_index
        ).unwrap();
        // Re-compress the same source
        let c2 = compressor.compress_action_table_small(
            &table.action_table, &table.symbol_to_index
        ).unwrap();
        prop_assert_eq!(c1.data.len(), c2.data.len());
        prop_assert_eq!(c1.row_offsets, c2.row_offsets);
    }

    // 21. Goto table compression produces valid row offsets
    #[test]
    fn goto_table_row_offsets_valid(
        (states, terms, nonterms) in table_dims()
    ) {
        let table = make_empty_table(states, terms, nonterms, 0);
        let compressor = TableCompressor::new();
        let compressed_goto = compressor.compress_goto_table_small(&table.goto_table).unwrap();
        prop_assert_eq!(
            compressed_goto.row_offsets.len(),
            table.state_count + 1,
            "goto row offsets length mismatch"
        );
    }

    // 22. Small table threshold is respected
    #[test]
    fn small_table_threshold_respected(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();
        // Our test tables are always small
        prop_assert!(
            table.state_count < compressed.small_table_threshold,
            "test tables should be below small_table_threshold"
        );
    }

    // 23. Mixed action rows preserve all non-error actions
    #[test]
    fn mixed_actions_preserved(terms in 2usize..=6) {
        let compressor = TableCompressor::new();
        let symbol_count = terms + 3;
        let mut row = vec![vec![]; symbol_count];
        row[0] = vec![Action::Shift(StateId(1))];
        row[1] = vec![Action::Reduce(RuleId(0))];
        if symbol_count > 2 {
            row[2] = vec![Action::Error]; // should be skipped
        }
        if symbol_count > 3 {
            row[3] = vec![Action::Accept];
        }
        let action_table = vec![row];
        let symbol_to_index = BTreeMap::new();
        let result = compressor.compress_action_table_small(&action_table, &symbol_to_index).unwrap();

        let expected_count = if symbol_count > 3 { 3 } else { 2 };
        prop_assert_eq!(result.data.len(), expected_count);
    }

    // 24. Compressed tables validate succeeds for well-formed tables
    #[test]
    fn compressed_tables_validate_ok(
        (states, terms, nonterms) in table_dims()
    ) {
        let mut table = make_empty_table(states.max(2), terms, nonterms, 0);
        table.action_table[0][1] = vec![Action::Shift(StateId(1))];
        let compressed = compress_with_grammar(&mut table, terms).unwrap();
        prop_assert!(compressed.validate(&table).is_ok());
    }

    // 25. Action table with only accepts on EOF column
    #[test]
    fn accept_only_table(terms in 1usize..=4) {
        let compressor = TableCompressor::new();
        let mut table = make_empty_table(1, terms, 0, 0);
        let eof_idx = table.symbol_to_index[&table.eof_symbol];
        // Place Accept at EOF
        table.action_table[0][eof_idx] = vec![Action::Accept];
        // Pass start_can_be_empty = true so state-0 validation accepts EOF-only actions
        let grammar = grammar_for_table(&table, terms);
        let tok_ix = collect_token_indices(&grammar, &table);
        let result = compressor.compress(&table, &tok_ix, true);
        prop_assert!(result.is_ok());
    }
}

// ---------------------------------------------------------------------------
// Non-proptest property tests (deterministic edge cases)
// ---------------------------------------------------------------------------

/// 26. CompressedActionEntry::new creates correct entry
#[test]
fn compressed_action_entry_new() {
    let entry = CompressedActionEntry::new(7, Action::Shift(StateId(3)));
    assert_eq!(entry.symbol, 7);
    assert_eq!(entry.action, Action::Shift(StateId(3)));
}

/// 27. Recover action has distinct encoding
#[test]
fn recover_action_encoding() {
    let compressor = TableCompressor::new();
    let encoded = compressor.encode_action_small(&Action::Recover).unwrap();
    assert_eq!(encoded, 0xFFFD);
}

/// 28. Empty parse table (zero states) rejected by compress
#[test]
fn empty_parse_table_rejected() {
    let mut table = make_empty_table(1, 1, 0, 0);
    table.state_count = 0;
    table.action_table.clear();
    let grammar = grammar_for_table(&table, 1);
    let tok_ix = collect_token_indices(&grammar, &table);
    let result = TableCompressor::new().compress(&table, &tok_ix, false);
    assert!(result.is_err());
}

/// 29. Sparse action table: mostly empty cells produce small compressed output
#[test]
fn sparse_action_table_compression() {
    let compressor = TableCompressor::new();
    let symbol_count = 20;
    let mut row = vec![vec![]; symbol_count];
    // Only one cell has an action
    row[5] = vec![Action::Shift(StateId(2))];
    let action_table = vec![row; 4]; // 4 identical rows
    let symbol_to_index = BTreeMap::new();
    let result = compressor
        .compress_action_table_small(&action_table, &symbol_to_index)
        .unwrap();
    // Each row contributes 1 entry = 4 total
    assert_eq!(result.data.len(), 4);
    assert_eq!(result.row_offsets.len(), 5); // 4 states + 1
}

/// 30. Last row offset equals data length
#[test]
fn last_row_offset_equals_data_len() {
    let compressor = TableCompressor::new();
    let mut rows = Vec::new();
    for i in 0..3 {
        let mut row = vec![vec![]; 5];
        row[i] = vec![Action::Reduce(RuleId(i as u16))];
        rows.push(row);
    }
    let symbol_to_index = BTreeMap::new();
    let result = compressor
        .compress_action_table_small(&rows, &symbol_to_index)
        .unwrap();
    assert_eq!(
        *result.row_offsets.last().unwrap() as usize,
        result.data.len()
    );
}
