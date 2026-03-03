#![cfg(feature = "test-api")]

use std::collections::BTreeMap;

use adze_glr_core::test_helpers::test::{
    actions_for, goto_for, has_accept_on_eof, reduce_rules, shift_destinations,
};
use adze_glr_core::{Action, GotoIndexing, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};

// ---------------------------------------------------------------------------
// Helpers to build minimal ParseTable fixtures
// ---------------------------------------------------------------------------

/// Build a minimal ParseTable with the given terminal symbols, one state row,
/// and NonterminalMap goto indexing.
fn minimal_table(terminals: &[SymbolId], actions: Vec<Vec<Action>>, eof: SymbolId) -> ParseTable {
    let symbol_to_index: BTreeMap<SymbolId, usize> =
        terminals.iter().enumerate().map(|(i, s)| (*s, i)).collect();
    let index_to_symbol: Vec<SymbolId> = terminals.to_vec();
    let state_count = actions.len();
    let symbol_count = terminals.len();

    ParseTable {
        action_table: actions
            .into_iter()
            .map(|row| row.into_iter().map(|a| vec![a]).collect())
            .collect(),
        goto_table: vec![vec![]; state_count],
        symbol_metadata: vec![],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: eof,
        start_symbol: SymbolId(0),
        grammar: Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: symbol_count,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Build a table with goto entries using NonterminalMap indexing.
fn table_with_goto(
    terminals: &[SymbolId],
    actions: Vec<Vec<Action>>,
    nonterminals: &[SymbolId],
    goto_rows: Vec<Vec<StateId>>,
    eof: SymbolId,
) -> ParseTable {
    let mut table = minimal_table(terminals, actions, eof);
    let nt_map: BTreeMap<SymbolId, usize> = nonterminals
        .iter()
        .enumerate()
        .map(|(i, s)| (*s, i))
        .collect();
    table.nonterminal_to_index = nt_map;
    table.goto_table = goto_rows;
    table.goto_indexing = GotoIndexing::NonterminalMap;
    table
}

/// Build a table with goto entries using DirectSymbolId indexing.
fn table_with_direct_goto(
    terminals: &[SymbolId],
    actions: Vec<Vec<Action>>,
    goto_rows: Vec<Vec<StateId>>,
    eof: SymbolId,
) -> ParseTable {
    let mut table = minimal_table(terminals, actions, eof);
    let max_symbol = goto_rows.iter().map(|row| row.len()).max().unwrap_or(0);
    table.nonterminal_to_index = (0..max_symbol)
        .map(|symbol| (SymbolId(symbol as u16), symbol))
        .collect();
    table.goto_table = goto_rows;
    table.remap_goto_to_direct_symbol_id()
}

// ===== 1. actions_for — basic retrieval =====================================

#[test]
fn actions_for_returns_shift() {
    let sym_a = SymbolId(1);
    let eof = SymbolId(0);
    let table = minimal_table(
        &[eof, sym_a],
        vec![vec![Action::Error, Action::Shift(StateId(2))]],
        eof,
    );
    let acts = actions_for(&table, 0, sym_a);
    assert_eq!(acts.len(), 1);
    assert!(matches!(acts[0], Action::Shift(StateId(2))));
}

#[test]
fn actions_for_returns_reduce() {
    let sym_a = SymbolId(1);
    let eof = SymbolId(0);
    let table = minimal_table(
        &[eof, sym_a],
        vec![vec![Action::Error, Action::Reduce(RuleId(3))]],
        eof,
    );
    let acts = actions_for(&table, 0, sym_a);
    assert_eq!(acts.len(), 1);
    assert!(matches!(acts[0], Action::Reduce(RuleId(3))));
}

#[test]
fn actions_for_returns_accept() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Accept]], eof);
    let acts = actions_for(&table, 0, eof);
    assert!(matches!(acts[0], Action::Accept));
}

#[test]
fn actions_for_returns_error() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Error]], eof);
    let acts = actions_for(&table, 0, eof);
    assert!(matches!(acts[0], Action::Error));
}

#[test]
fn actions_for_correct_state_row() {
    let eof = SymbolId(0);
    let sym_a = SymbolId(1);
    let table = minimal_table(
        &[eof, sym_a],
        vec![
            vec![Action::Error, Action::Shift(StateId(1))],
            vec![Action::Accept, Action::Reduce(RuleId(0))],
        ],
        eof,
    );
    // State 0
    let acts0 = actions_for(&table, 0, sym_a);
    assert!(matches!(acts0[0], Action::Shift(StateId(1))));
    // State 1
    let acts1 = actions_for(&table, 1, sym_a);
    assert!(matches!(acts1[0], Action::Reduce(RuleId(0))));
}

#[test]
#[should_panic(expected = "Symbol")]
fn actions_for_panics_on_unknown_symbol() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Error]], eof);
    let _ = actions_for(&table, 0, SymbolId(99));
}

// ===== 2. has_accept_on_eof ================================================

#[test]
fn has_accept_on_eof_true_when_accept_present() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Accept]], eof);
    assert!(has_accept_on_eof(&table, 0));
}

#[test]
fn has_accept_on_eof_false_when_only_error() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Error]], eof);
    assert!(!has_accept_on_eof(&table, 0));
}

#[test]
fn has_accept_on_eof_false_when_shift() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Shift(StateId(1))]], eof);
    assert!(!has_accept_on_eof(&table, 0));
}

#[test]
fn has_accept_on_eof_false_when_reduce() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Reduce(RuleId(0))]], eof);
    assert!(!has_accept_on_eof(&table, 0));
}

// ===== 3. shift_destinations ===============================================

#[test]
fn shift_destinations_returns_single_shift() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    let table = minimal_table(
        &[eof, sym],
        vec![vec![Action::Error, Action::Shift(StateId(5))]],
        eof,
    );
    let dests = shift_destinations(&table, 0, sym);
    assert_eq!(dests, vec![StateId(5)]);
}

#[test]
fn shift_destinations_empty_when_no_shifts() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    let table = minimal_table(
        &[eof, sym],
        vec![vec![Action::Error, Action::Reduce(RuleId(0))]],
        eof,
    );
    let dests = shift_destinations(&table, 0, sym);
    assert!(dests.is_empty());
}

#[test]
fn shift_destinations_ignores_non_shift_actions() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Accept]], eof);
    let dests = shift_destinations(&table, 0, eof);
    assert!(dests.is_empty());
}

// ===== 4. reduce_rules =====================================================

#[test]
fn reduce_rules_returns_single_rule() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    let table = minimal_table(
        &[eof, sym],
        vec![vec![Action::Error, Action::Reduce(RuleId(7))]],
        eof,
    );
    let rules = reduce_rules(&table, 0, sym);
    assert_eq!(rules, vec![RuleId(7)]);
}

#[test]
fn reduce_rules_empty_when_no_reduces() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    let table = minimal_table(
        &[eof, sym],
        vec![vec![Action::Error, Action::Shift(StateId(1))]],
        eof,
    );
    let rules = reduce_rules(&table, 0, sym);
    assert!(rules.is_empty());
}

#[test]
fn reduce_rules_ignores_accept_and_error() {
    let eof = SymbolId(0);
    let table = minimal_table(&[eof], vec![vec![Action::Accept]], eof);
    let rules = reduce_rules(&table, 0, eof);
    assert!(rules.is_empty());
}

// ===== 5. goto_for — NonterminalMap indexing ================================

#[test]
fn goto_for_returns_target_state() {
    let eof = SymbolId(0);
    let nt = SymbolId(10);
    let table = table_with_goto(
        &[eof],
        vec![vec![Action::Error]],
        &[nt],
        vec![vec![StateId(3)]],
        eof,
    );
    assert_eq!(goto_for(&table, 0, nt), Some(StateId(3)));
}

#[test]
fn goto_for_returns_none_for_zero_state() {
    let eof = SymbolId(0);
    let nt = SymbolId(10);
    let table = table_with_goto(
        &[eof],
        vec![vec![Action::Error]],
        &[nt],
        vec![vec![StateId(0)]],
        eof,
    );
    // StateId(0) is filtered out as "no goto"
    assert_eq!(goto_for(&table, 0, nt), None);
}

#[test]
fn goto_for_returns_none_for_unknown_nonterminal() {
    let eof = SymbolId(0);
    let nt = SymbolId(10);
    let table = table_with_goto(
        &[eof],
        vec![vec![Action::Error]],
        &[nt],
        vec![vec![StateId(3)]],
        eof,
    );
    assert_eq!(goto_for(&table, 0, SymbolId(99)), None);
}

// ===== 6. goto_for — DirectSymbolId indexing ================================

#[test]
fn goto_for_direct_returns_target_state() {
    let eof = SymbolId(0);
    let nt = SymbolId(2);
    // Need goto row wide enough for index 2
    let table = table_with_direct_goto(
        &[eof],
        vec![vec![Action::Error]],
        vec![vec![StateId(0), StateId(0), StateId(7)]],
        eof,
    );
    assert_eq!(goto_for(&table, 0, nt), Some(StateId(7)));
}

#[test]
fn goto_for_direct_returns_none_for_zero() {
    let eof = SymbolId(0);
    let nt = SymbolId(1);
    let table = table_with_direct_goto(
        &[eof],
        vec![vec![Action::Error]],
        vec![vec![StateId(0), StateId(0)]],
        eof,
    );
    assert_eq!(goto_for(&table, 0, nt), None);
}

#[test]
fn goto_for_direct_returns_none_for_out_of_bounds() {
    let eof = SymbolId(0);
    let table = table_with_direct_goto(
        &[eof],
        vec![vec![Action::Error]],
        vec![vec![StateId(0)]],
        eof,
    );
    // SymbolId(5) is out of bounds for a row of length 1
    assert_eq!(goto_for(&table, 0, SymbolId(5)), None);
}

// ===== 7. Multi-action cells (GLR scenarios) ===============================

#[test]
fn shift_destinations_with_multiple_shifts() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    // Manually build a multi-action cell
    let mut table = minimal_table(&[eof, sym], vec![vec![Action::Error, Action::Error]], eof);
    table.action_table[0][1] = vec![Action::Shift(StateId(2)), Action::Shift(StateId(3))];
    let dests = shift_destinations(&table, 0, sym);
    assert_eq!(dests, vec![StateId(2), StateId(3)]);
}

#[test]
fn reduce_rules_with_multiple_reduces() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    let mut table = minimal_table(&[eof, sym], vec![vec![Action::Error, Action::Error]], eof);
    table.action_table[0][1] = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    let rules = reduce_rules(&table, 0, sym);
    assert_eq!(rules, vec![RuleId(1), RuleId(2)]);
}

#[test]
fn shift_reduce_conflict_cell() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    let mut table = minimal_table(&[eof, sym], vec![vec![Action::Error, Action::Error]], eof);
    table.action_table[0][1] = vec![Action::Shift(StateId(4)), Action::Reduce(RuleId(5))];
    let dests = shift_destinations(&table, 0, sym);
    assert_eq!(dests, vec![StateId(4)]);
    let rules = reduce_rules(&table, 0, sym);
    assert_eq!(rules, vec![RuleId(5)]);
}

// ===== 8. Multi-state tables ===============================================

#[test]
fn multi_state_actions_for_each_state() {
    let eof = SymbolId(0);
    let sym = SymbolId(1);
    let table = minimal_table(
        &[eof, sym],
        vec![
            vec![Action::Error, Action::Shift(StateId(1))],
            vec![Action::Error, Action::Shift(StateId(2))],
            vec![Action::Accept, Action::Reduce(RuleId(0))],
        ],
        eof,
    );
    assert!(matches!(
        actions_for(&table, 0, sym)[0],
        Action::Shift(StateId(1))
    ));
    assert!(matches!(
        actions_for(&table, 1, sym)[0],
        Action::Shift(StateId(2))
    ));
    assert!(matches!(
        actions_for(&table, 2, sym)[0],
        Action::Reduce(RuleId(0))
    ));
    assert!(has_accept_on_eof(&table, 2));
    assert!(!has_accept_on_eof(&table, 0));
}

// ===== 9. Goto with multiple nonterminals ==================================

#[test]
fn goto_for_multiple_nonterminals() {
    let eof = SymbolId(0);
    let nt_a = SymbolId(10);
    let nt_b = SymbolId(11);
    let table = table_with_goto(
        &[eof],
        vec![vec![Action::Error]],
        &[nt_a, nt_b],
        vec![vec![StateId(2), StateId(5)]],
        eof,
    );
    assert_eq!(goto_for(&table, 0, nt_a), Some(StateId(2)));
    assert_eq!(goto_for(&table, 0, nt_b), Some(StateId(5)));
}

// ===== 10. Edge case: empty action cell ====================================

#[test]
fn actions_for_empty_cell() {
    let eof = SymbolId(0);
    let mut table = minimal_table(&[eof], vec![vec![Action::Error]], eof);
    // Replace with empty cell
    table.action_table[0][0] = vec![];
    let acts = actions_for(&table, 0, eof);
    assert!(acts.is_empty());
    assert!(!has_accept_on_eof(&table, 0));
    assert!(shift_destinations(&table, 0, eof).is_empty());
    assert!(reduce_rules(&table, 0, eof).is_empty());
}

// ===== 11. Non-zero EOF symbol =============================================

#[test]
fn has_accept_on_eof_with_nonzero_eof_symbol() {
    let eof = SymbolId(42);
    let sym = SymbolId(1);
    let table = minimal_table(&[sym, eof], vec![vec![Action::Error, Action::Accept]], eof);
    assert!(has_accept_on_eof(&table, 0));
}

// ===== 12. ParseTable default ===============================================

#[test]
fn parse_table_default_has_empty_tables() {
    let table = ParseTable::default();
    assert_eq!(table.state_count, 0);
    assert_eq!(table.symbol_count, 0);
    assert!(table.action_table.is_empty());
    assert!(table.goto_table.is_empty());
    assert!(table.rules.is_empty());
    assert!(table.symbol_to_index.is_empty());
    assert!(table.nonterminal_to_index.is_empty());
}
