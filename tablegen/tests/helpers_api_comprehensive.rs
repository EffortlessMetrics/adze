//! Comprehensive tests for the `helpers` module in `adze-tablegen`.
//!
//! Covers `collect_token_indices` and `eof_accepts_or_reduces` with a wide
//! range of edge cases: empty grammars, large token sets, duplicate columns,
//! external tokens, all Action variants, multi-state tables, and more.

use std::collections::BTreeMap;

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId, Token, TokenPattern};
use adze_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal but valid `ParseTable` for integration tests.
fn make_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = nonterms.max(1);
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

fn tok(name: &str, pat: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(pat.to_string()),
        fragile: false,
    }
}

// =========================================================================
// collect_token_indices
// =========================================================================

/// Empty grammar should still include the EOF column.
#[test]
fn cti_empty_grammar_includes_eof() {
    let grammar = Grammar::default();
    let table = make_table(1, 0, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    let indices = collect_token_indices(&grammar, &table);

    assert_eq!(indices, vec![eof_col]);
}

/// Single token plus EOF gives exactly two entries.
#[test]
fn cti_single_token() {
    let mut grammar = Grammar::default();
    grammar.tokens.insert(SymbolId(1), tok("a", "a"));

    let table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    let indices = collect_token_indices(&grammar, &table);

    assert_eq!(indices.len(), 2);
    assert!(indices.contains(&1)); // SymbolId(1) → column 1
    assert!(indices.contains(&eof_col));
}

/// Result is always sorted in ascending order.
#[test]
fn cti_result_is_sorted() {
    let mut grammar = Grammar::default();
    for i in 1..=5u16 {
        grammar
            .tokens
            .insert(SymbolId(i), tok(&format!("t{i}"), &format!("{i}")));
    }
    let table = make_table(1, 5, 0, 0);

    let indices = collect_token_indices(&grammar, &table);

    assert!(indices.windows(2).all(|w| w[0] < w[1]));
}

/// Duplicate column indices are deduplicated.
#[test]
fn cti_deduplicates_columns() {
    let mut grammar = Grammar::default();
    grammar.tokens.insert(SymbolId(1), tok("a", "a"));
    grammar.tokens.insert(SymbolId(100), tok("alias", "a"));

    let mut table = make_table(1, 1, 0, 0);
    // Force both grammar tokens to map to the same column
    table.symbol_to_index.insert(SymbolId(100), 1);

    let indices = collect_token_indices(&grammar, &table);

    // Column 1 should appear only once, plus EOF
    assert_eq!(indices.iter().filter(|&&c| c == 1).count(), 1);
}

/// Tokens not present in symbol_to_index are silently skipped.
#[test]
fn cti_missing_symbol_skipped() {
    let mut grammar = Grammar::default();
    grammar.tokens.insert(SymbolId(1), tok("real", "r"));
    grammar.tokens.insert(SymbolId(9999), tok("phantom", "???"));

    let table = make_table(1, 1, 0, 0);

    let indices = collect_token_indices(&grammar, &table);

    assert!(!indices.contains(&9999));
    assert!(indices.contains(&1));
}

/// When a grammar token maps to the same column as EOF, only one entry appears.
#[test]
fn cti_eof_overlap_dedup() {
    let mut grammar = Grammar::default();
    let mut table = make_table(1, 2, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    grammar.tokens.insert(SymbolId(200), tok("over", "x"));
    table.symbol_to_index.insert(SymbolId(200), eof_col);

    let indices = collect_token_indices(&grammar, &table);

    assert_eq!(indices.iter().filter(|&&c| c == eof_col).count(), 1);
}

/// Many tokens (20) are all included.
#[test]
fn cti_many_tokens() {
    let mut grammar = Grammar::default();
    let table = make_table(1, 20, 0, 0);

    for i in 1..=20u16 {
        grammar
            .tokens
            .insert(SymbolId(i), tok(&format!("t{i}"), &format!("{i}")));
    }

    let indices = collect_token_indices(&grammar, &table);

    // 20 tokens + 1 EOF = 21
    assert_eq!(indices.len(), 21);
    assert!(indices.windows(2).all(|w| w[0] < w[1]));
}

/// External tokens shift the EOF column.
#[test]
fn cti_with_external_tokens() {
    let mut grammar = Grammar::default();
    let table = make_table(1, 3, 1, 2);

    // EOF should be at 1 + 3 + 2 = 6
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    assert_eq!(eof_col, 6);

    for i in 1..=3u16 {
        grammar
            .tokens
            .insert(SymbolId(i), tok(&format!("t{i}"), &format!("{i}")));
    }

    let indices = collect_token_indices(&grammar, &table);

    assert_eq!(indices.len(), 4); // 3 tokens + EOF
    assert!(indices.contains(&eof_col));
}

/// When EOF itself is missing from symbol_to_index, only grammar tokens appear.
#[test]
fn cti_missing_eof_symbol() {
    let mut grammar = Grammar::default();
    let mut table = make_table(1, 2, 0, 0);

    grammar.tokens.insert(SymbolId(1), tok("a", "a"));
    grammar.tokens.insert(SymbolId(2), tok("b", "b"));

    table.symbol_to_index.remove(&table.eof_symbol);

    let indices = collect_token_indices(&grammar, &table);

    assert_eq!(indices, vec![1, 2]); // no EOF
}

/// Both grammar and table are empty → empty result when EOF is removed.
#[test]
fn cti_completely_empty() {
    let grammar = Grammar::default();
    let mut table = make_table(1, 0, 0, 0);
    table.symbol_to_index.remove(&table.eof_symbol);

    let indices = collect_token_indices(&grammar, &table);

    assert!(indices.is_empty());
}

/// Nonterminal symbols in the grammar's token map are included if they
/// appear in symbol_to_index (function doesn't distinguish term/nonterm).
#[test]
fn cti_does_not_filter_by_symbol_kind() {
    let mut grammar = Grammar::default();
    let table = make_table(1, 5, 2, 0);

    // Insert a token whose SymbolId falls in the nonterminal range
    let nt_id = table.start_symbol; // a nonterminal column
    grammar.tokens.insert(nt_id, tok("fake_nt", "n"));
    // Also a normal terminal
    grammar.tokens.insert(SymbolId(1), tok("t1", "a"));

    let indices = collect_token_indices(&grammar, &table);

    assert!(indices.contains(&(nt_id.0 as usize)));
    assert!(indices.contains(&1));
}

/// Result is idempotent: calling twice yields the same result.
#[test]
fn cti_idempotent() {
    let mut grammar = Grammar::default();
    let table = make_table(1, 3, 0, 0);
    for i in 1..=3u16 {
        grammar
            .tokens
            .insert(SymbolId(i), tok(&format!("t{i}"), &format!("{i}")));
    }

    let a = collect_token_indices(&grammar, &table);
    let b = collect_token_indices(&grammar, &table);

    assert_eq!(a, b);
}

// =========================================================================
// eof_accepts_or_reduces
// =========================================================================

/// Accept on EOF in state 0 → true.
#[test]
fn ear_accept_returns_true() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Accept];

    assert!(eof_accepts_or_reduces(&table));
}

/// Reduce on EOF in state 0 → true.
#[test]
fn ear_reduce_returns_true() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Reduce(RuleId(0))];

    assert!(eof_accepts_or_reduces(&table));
}

/// Shift on EOF in state 0 → false.
#[test]
fn ear_shift_returns_false() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Shift(StateId(1))];

    assert!(!eof_accepts_or_reduces(&table));
}

/// Error action on EOF → false.
#[test]
fn ear_error_returns_false() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Error];

    assert!(!eof_accepts_or_reduces(&table));
}

/// Recover action on EOF → false.
#[test]
fn ear_recover_returns_false() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Recover];

    assert!(!eof_accepts_or_reduces(&table));
}

/// Empty cell (no actions) → false.
#[test]
fn ear_empty_cell_returns_false() {
    let table = make_table(1, 1, 0, 0);
    assert!(!eof_accepts_or_reduces(&table));
}

/// Empty action table → false.
#[test]
fn ear_empty_action_table() {
    let mut table = make_table(1, 1, 0, 0);
    table.action_table.clear();
    assert!(!eof_accepts_or_reduces(&table));
}

/// Missing EOF in symbol_to_index → false.
#[test]
fn ear_missing_eof_returns_false() {
    let mut table = make_table(1, 1, 0, 0);
    table.symbol_to_index.remove(&table.eof_symbol);
    assert!(!eof_accepts_or_reduces(&table));
}

/// Multi-action cell with Accept among other actions → true.
#[test]
fn ear_multi_action_with_accept() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Shift(StateId(1)), Action::Error, Action::Accept];

    assert!(eof_accepts_or_reduces(&table));
}

/// Multi-action cell with Reduce among other actions → true.
#[test]
fn ear_multi_action_with_reduce() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Error, Action::Reduce(RuleId(42))];

    assert!(eof_accepts_or_reduces(&table));
}

/// Multi-action cell with only Shift and Error → false.
#[test]
fn ear_multi_action_no_accept_no_reduce() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Shift(StateId(0)), Action::Error];

    assert!(!eof_accepts_or_reduces(&table));
}

/// Only state 0 is inspected; Accept on later state is irrelevant.
#[test]
fn ear_only_inspects_state_zero() {
    let mut table = make_table(3, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    // State 0: empty
    // State 1: Accept
    table.action_table[1][eof_col] = vec![Action::Accept];
    // State 2: Reduce
    table.action_table[2][eof_col] = vec![Action::Reduce(RuleId(0))];

    assert!(!eof_accepts_or_reduces(&table));
}

/// Accept on a non-EOF column in state 0 does not count.
#[test]
fn ear_accept_on_wrong_column() {
    let mut table = make_table(1, 3, 0, 0);
    // Place Accept on column 1 (a terminal, not EOF)
    table.action_table[0][1] = vec![Action::Accept];

    assert!(!eof_accepts_or_reduces(&table));
}

/// With external tokens, EOF shifts to a higher column; Accept there → true.
#[test]
fn ear_with_external_tokens() {
    let mut table = make_table(1, 2, 0, 3);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    assert_eq!(eof_col, 6); // 1 + 2 + 3

    table.action_table[0][eof_col] = vec![Action::Accept];

    assert!(eof_accepts_or_reduces(&table));
}

/// Fork action wrapping Accept is NOT considered accept (function checks top-level).
#[test]
fn ear_fork_wrapping_accept() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Fork(vec![Action::Accept])];

    // Fork is not Accept or Reduce at the top level
    assert!(!eof_accepts_or_reduces(&table));
}

/// Reduce with different rule IDs all count as true.
#[test]
fn ear_reduce_various_rule_ids() {
    for rule_id in [0u16, 1, 100, u16::MAX] {
        let mut table = make_table(1, 1, 0, 0);
        let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
        table.action_table[0][eof_col] = vec![Action::Reduce(RuleId(rule_id))];

        assert!(
            eof_accepts_or_reduces(&table),
            "Reduce(RuleId({rule_id})) should return true"
        );
    }
}

/// Single-state table with Accept and Reduce simultaneously → true.
#[test]
fn ear_both_accept_and_reduce() {
    let mut table = make_table(1, 1, 0, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    table.action_table[0][eof_col] = vec![Action::Accept, Action::Reduce(RuleId(0))];

    assert!(eof_accepts_or_reduces(&table));
}

/// Large table (50 states); only state 0 matters.
#[test]
fn ear_large_table_only_state_zero() {
    let mut table = make_table(50, 5, 2, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    // Place Accept in every state except 0
    for s in 1..50 {
        table.action_table[s][eof_col] = vec![Action::Accept];
    }

    assert!(!eof_accepts_or_reduces(&table));

    // Now add Accept to state 0 too
    table.action_table[0][eof_col] = vec![Action::Accept];
    assert!(eof_accepts_or_reduces(&table));
}

// =========================================================================
// Interaction / combined tests
// =========================================================================

/// Both helpers work correctly on the same table.
#[test]
fn combined_both_helpers_same_table() {
    let mut grammar = Grammar::default();
    let mut table = make_table(2, 3, 1, 0);
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();

    for i in 1..=3u16 {
        grammar
            .tokens
            .insert(SymbolId(i), tok(&format!("t{i}"), &format!("{i}")));
    }

    // Before adding Accept
    let indices = collect_token_indices(&grammar, &table);
    assert_eq!(indices.len(), 4); // 3 tokens + EOF
    assert!(!eof_accepts_or_reduces(&table));

    // After adding Accept
    table.action_table[0][eof_col] = vec![Action::Accept];
    assert!(eof_accepts_or_reduces(&table));

    // collect_token_indices is unaffected by actions
    let indices2 = collect_token_indices(&grammar, &table);
    assert_eq!(indices, indices2);
}

/// Both helpers agree on a table with no symbols mapped.
#[test]
fn combined_empty_symbol_map() {
    let grammar = Grammar::default();
    let mut table = make_table(1, 0, 0, 0);
    table.symbol_to_index.clear();

    let indices = collect_token_indices(&grammar, &table);
    assert!(indices.is_empty());
    assert!(!eof_accepts_or_reduces(&table));
}
