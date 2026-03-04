//! Comprehensive tests for helpers and utility modules.
//!
//! Focuses on `collect_token_indices` and `eof_accepts_or_reduces` with
//! edge cases, many-token scenarios, external tokens, and action-table
//! configurations not covered by the existing unit tests.

use std::collections::BTreeMap;

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId, Token, TokenPattern};
use adze_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable for integration tests.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
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
    for (symbol_id, index) in &symbol_to_index {
        index_to_symbol[*index] = *symbol_id;
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
        grammar: Grammar::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

fn make_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(pattern.to_string()),
        fragile: false,
    }
}

// ---------------------------------------------------------------------------
// collect_token_indices tests
// ---------------------------------------------------------------------------

/// Many tokens should all appear sorted and deduplicated.
#[test]
fn collect_token_indices_many_tokens() {
    let mut grammar = Grammar::default();
    let table = make_empty_table(1, 20, 1, 0);

    // Insert 20 terminals into the grammar (SymbolIds 1..=20)
    for i in 1..=20u16 {
        grammar.tokens.insert(
            SymbolId(i),
            make_token(&format!("t{i}"), &format!("tok{i}")),
        );
    }

    let indices = collect_token_indices(&grammar, &table);

    // All 20 tokens + EOF must be present
    assert_eq!(indices.len(), 21);
    // Must be sorted
    assert!(indices.windows(2).all(|w| w[0] < w[1]));
    // EOF column must be included
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    assert!(indices.contains(&eof_col));
}

/// Tokens not present in symbol_to_index are silently skipped.
#[test]
fn collect_token_indices_missing_tokens_skipped() {
    let mut grammar = Grammar::default();
    let table = make_empty_table(1, 2, 0, 0);

    // Add a token whose SymbolId is NOT in the table's symbol_to_index
    grammar
        .tokens
        .insert(SymbolId(999), make_token("phantom", "???"));
    // Also add one that IS in the map
    grammar.tokens.insert(SymbolId(1), make_token("real", "a"));

    let indices = collect_token_indices(&grammar, &table);

    // Only EOF + the one real token should appear
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    assert!(indices.contains(&eof_col));
    assert!(indices.contains(&1)); // SymbolId(1) → column 1
    assert!(!indices.contains(&999));
    assert_eq!(indices.len(), 2);
}

/// When the grammar has tokens that map to the same column as EOF, dedup
/// should collapse them into one entry.
#[test]
fn collect_token_indices_dedup_with_eof_overlap() {
    let mut grammar = Grammar::default();
    let mut table = make_empty_table(1, 2, 0, 0);

    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    // Force a grammar token to share the EOF column
    let overlapping = SymbolId(100);
    table.symbol_to_index.insert(overlapping, eof_col);
    grammar
        .tokens
        .insert(overlapping, make_token("overlaps_eof", "x"));

    let indices = collect_token_indices(&grammar, &table);

    // The EOF column should appear exactly once despite two mappings
    assert_eq!(indices.iter().filter(|&&c| c == eof_col).count(), 1);
}

/// External tokens shift EOF position; verify collect_token_indices still
/// finds the correct EOF column.
#[test]
fn collect_token_indices_with_external_tokens() {
    let mut grammar = Grammar::default();
    let table = make_empty_table(1, 3, 1, 2);

    // 3 terminals + 2 externals → EOF at column 1+3+2 = 6
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    assert_eq!(eof_col, 6);

    // Register the 3 real terminals in the grammar
    for i in 1..=3u16 {
        grammar
            .tokens
            .insert(SymbolId(i), make_token(&format!("t{i}"), &format!("{i}")));
    }

    let indices = collect_token_indices(&grammar, &table);
    assert!(indices.contains(&eof_col));
    // 3 tokens + EOF = 4 entries
    assert_eq!(indices.len(), 4);
    assert!(indices.windows(2).all(|w| w[0] < w[1]));
}

// ---------------------------------------------------------------------------
// eof_accepts_or_reduces tests
// ---------------------------------------------------------------------------

/// Accept action on EOF in state 0 should return true.
#[test]
fn eof_accepts_or_reduces_accept_on_eof() {
    let mut table = make_empty_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    table.action_table[0][eof_col] = vec![Action::Accept];

    assert!(eof_accepts_or_reduces(&table));
}

/// Reduce action on EOF in state 0 should return true.
#[test]
fn eof_accepts_or_reduces_reduce_on_eof() {
    let mut table = make_empty_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    table.action_table[0][eof_col] = vec![Action::Reduce(RuleId(0))];

    assert!(eof_accepts_or_reduces(&table));
}

/// Shift on EOF in state 0 should return false.
#[test]
fn eof_accepts_or_reduces_shift_on_eof() {
    let mut table = make_empty_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    table.action_table[0][eof_col] = vec![Action::Shift(StateId(1))];

    assert!(!eof_accepts_or_reduces(&table));
}

/// Empty action cell on EOF in state 0 should return false.
#[test]
fn eof_accepts_or_reduces_empty_cell() {
    let table = make_empty_table(1, 1, 0, 0);
    // Default cells are empty vecs
    assert!(!eof_accepts_or_reduces(&table));
}

/// When EOF symbol is missing from symbol_to_index, should return false.
#[test]
fn eof_accepts_or_reduces_missing_eof_symbol() {
    let mut table = make_empty_table(1, 1, 0, 0);
    table.symbol_to_index.remove(&table.eof_symbol);

    assert!(!eof_accepts_or_reduces(&table));
}

/// Multi-action cell containing both Shift and Accept should still detect
/// the Accept and return true.
#[test]
fn eof_accepts_or_reduces_multi_action_cell() {
    let mut table = make_empty_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    table.action_table[0][eof_col] = vec![Action::Shift(StateId(1)), Action::Error, Action::Accept];

    assert!(eof_accepts_or_reduces(&table));
}

/// Only state 0 matters; Accept on EOF in state 1 should not affect result.
#[test]
fn eof_accepts_or_reduces_ignores_non_initial_states() {
    let mut table = make_empty_table(3, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    // State 0: empty (no accept/reduce)
    // State 1: Accept on EOF
    table.action_table[1][eof_col] = vec![Action::Accept];
    // State 2: Reduce on EOF
    table.action_table[2][eof_col] = vec![Action::Reduce(RuleId(0))];

    assert!(!eof_accepts_or_reduces(&table));
}

/// Empty action table should return false.
#[test]
fn eof_accepts_or_reduces_empty_action_table() {
    let mut table = make_empty_table(1, 1, 0, 0);
    table.action_table.clear();

    assert!(!eof_accepts_or_reduces(&table));
}
