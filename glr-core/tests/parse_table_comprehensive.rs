#![allow(
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::vec_init_then_push,
    clippy::useless_vec
)]

//! Comprehensive tests for ParseTable in adze-glr-core.
//!
//! This test suite covers:
//! - ParseTable::default() initialization
//! - Action table and GOTO table construction
//! - Symbol mapping consistency (symbol_to_index, index_to_symbol)
//! - Action enum variants and their properties
//! - ActionCell (Vec<Action>) for GLR fork support
//! - ParseRule creation and manipulation
//! - SymbolMetadata creation and properties
//! - Large tables with many states
//! - All action types in a single table
//! - Fork actions with nested actions
//! - Empty table invariants

use adze_glr_core::{
    Action, ActionCell, FirstFollowSets, GotoIndexing, Grammar, ParseRule, ParseTable, RuleId,
    StateId, SymbolId, SymbolMetadata,
};
use std::collections::BTreeMap;

// =============================================================================
// Helpers
// =============================================================================

/// Create a minimal ParseTable with default values
fn create_minimal_table() -> ParseTable {
    ParseTable::default()
}

/// Create a ParseTable with custom state and symbol counts
fn create_sized_table(state_count: usize, symbol_count: usize) -> ParseTable {
    let mut table = ParseTable::default();
    table.state_count = state_count;
    table.symbol_count = symbol_count;
    table
}

/// Create a SymbolMetadata for testing
fn create_symbol_meta(name: &str, id: u16, is_terminal: bool) -> SymbolMetadata {
    SymbolMetadata {
        name: name.to_string(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(id),
    }
}

/// Create a ParseRule for testing
fn create_rule(lhs: u16, rhs_len: u16) -> ParseRule {
    ParseRule {
        lhs: SymbolId(lhs),
        rhs_len,
    }
}

// =============================================================================
// 1. ParseTable::default() field values
// =============================================================================

#[test]
fn test_default_table_empty_action_table() {
    let table = create_minimal_table();
    assert!(table.action_table.is_empty());
}

#[test]
fn test_default_table_empty_goto_table() {
    let table = create_minimal_table();
    assert!(table.goto_table.is_empty());
}

#[test]
fn test_default_table_empty_symbol_metadata() {
    let table = create_minimal_table();
    assert!(table.symbol_metadata.is_empty());
}

#[test]
fn test_default_table_zero_state_count() {
    let table = create_minimal_table();
    assert_eq!(table.state_count, 0);
}

#[test]
fn test_default_table_zero_symbol_count() {
    let table = create_minimal_table();
    assert_eq!(table.symbol_count, 0);
}

#[test]
fn test_default_table_empty_symbol_to_index() {
    let table = create_minimal_table();
    assert!(table.symbol_to_index.is_empty());
}

#[test]
fn test_default_table_empty_index_to_symbol() {
    let table = create_minimal_table();
    assert!(table.index_to_symbol.is_empty());
}

#[test]
fn test_default_table_empty_rules() {
    let table = create_minimal_table();
    assert!(table.rules.is_empty());
}

#[test]
fn test_default_table_zero_initial_state() {
    let table = create_minimal_table();
    assert_eq!(table.initial_state, StateId(0));
}

#[test]
fn test_default_table_zero_eof_symbol() {
    let table = create_minimal_table();
    assert_eq!(table.eof_symbol, SymbolId(0));
}

#[test]
fn test_default_table_zero_start_symbol() {
    let table = create_minimal_table();
    assert_eq!(table.start_symbol, SymbolId(0));
}

#[test]
fn test_default_table_goto_indexing() {
    let table = create_minimal_table();
    assert_eq!(table.goto_indexing, GotoIndexing::NonterminalMap);
}

#[test]
fn test_default_table_zero_token_count() {
    let table = create_minimal_table();
    assert_eq!(table.token_count, 0);
}

#[test]
fn test_default_table_zero_external_token_count() {
    let table = create_minimal_table();
    assert_eq!(table.external_token_count, 0);
}

#[test]
fn test_default_table_empty_extras() {
    let table = create_minimal_table();
    assert!(table.extras.is_empty());
}

#[test]
fn test_default_table_empty_lex_modes() {
    let table = create_minimal_table();
    assert!(table.lex_modes.is_empty());
}

// =============================================================================
// 2. Adding actions to action_table
// =============================================================================

#[test]
fn test_action_table_add_single_action() {
    let mut table = create_minimal_table();
    // action_table is Vec<Vec<ActionCell>>, so we need to add a state row
    table
        .action_table
        .push(vec![vec![Action::Shift(StateId(1))]]);
    assert_eq!(table.action_table[0].len(), 1);
    assert_eq!(table.action_table[0][0][0], Action::Shift(StateId(1)));
}

#[test]
fn test_action_table_multiple_rows() {
    let mut table = create_minimal_table();
    table
        .action_table
        .push(vec![vec![Action::Shift(StateId(1))]]);
    table
        .action_table
        .push(vec![vec![Action::Reduce(RuleId(0))]]);
    table.action_table.push(vec![vec![Action::Accept]]);
    assert_eq!(table.action_table.len(), 3);
}

#[test]
fn test_action_table_multiple_columns() {
    let mut table = create_minimal_table();
    // ActionCell is the inner Vec<Action>
    let mut action_cell = vec![];
    action_cell.push(Action::Shift(StateId(1)));
    action_cell.push(Action::Reduce(RuleId(2)));
    action_cell.push(Action::Accept);
    table.action_table.push(vec![action_cell]);
    assert_eq!(table.action_table[0].len(), 1);
    assert_eq!(table.action_table[0][0].len(), 3);
}

#[test]
fn test_action_table_modify_existing_action() {
    let mut table = create_minimal_table();
    table.action_table.push(vec![vec![Action::Error]]);
    table.action_table[0][0][0] = Action::Shift(StateId(5));
    assert_eq!(table.action_table[0][0][0], Action::Shift(StateId(5)));
}

#[test]
fn test_action_table_large_state_count() {
    let mut table = create_sized_table(100, 50);
    for _ in 0..100 {
        table.action_table.push(vec![vec![Action::Error]]);
    }
    assert_eq!(table.action_table.len(), 100);
}

// =============================================================================
// 3. Adding entries to goto_table
// =============================================================================

#[test]
fn test_goto_table_add_single_entry() {
    let mut table = create_minimal_table();
    table.goto_table.push(vec![]);
    table.goto_table[0].push(StateId(2));
    assert_eq!(table.goto_table[0].len(), 1);
    assert_eq!(table.goto_table[0][0], StateId(2));
}

#[test]
fn test_goto_table_multiple_rows() {
    let mut table = create_minimal_table();
    table.goto_table.push(vec![StateId(1)]);
    table.goto_table.push(vec![StateId(2)]);
    table.goto_table.push(vec![StateId(3)]);
    assert_eq!(table.goto_table.len(), 3);
}

#[test]
fn test_goto_table_multiple_columns() {
    let mut table = create_minimal_table();
    table
        .goto_table
        .push(vec![StateId(1), StateId(2), StateId(3)]);
    assert_eq!(table.goto_table[0].len(), 3);
}

#[test]
fn test_goto_table_large_state_ids() {
    let mut table = create_minimal_table();
    table.goto_table.push(vec![StateId(u16::MAX - 10)]);
    table.goto_table.push(vec![StateId(u16::MAX)]);
    assert_eq!(table.goto_table[1][0], StateId(u16::MAX));
}

// =============================================================================
// 4. symbol_to_index consistency
// =============================================================================

#[test]
fn test_symbol_to_index_insert_single() {
    let mut table = create_minimal_table();
    table.symbol_to_index.insert(SymbolId(42), 0);
    assert_eq!(table.symbol_to_index.get(&SymbolId(42)), Some(&0));
}

#[test]
fn test_symbol_to_index_multiple_entries() {
    let mut table = create_minimal_table();
    table.symbol_to_index.insert(SymbolId(1), 0);
    table.symbol_to_index.insert(SymbolId(2), 1);
    table.symbol_to_index.insert(SymbolId(3), 2);
    assert_eq!(table.symbol_to_index.len(), 3);
}

#[test]
fn test_index_to_symbol_mapping() {
    let mut table = create_minimal_table();
    table.index_to_symbol.push(SymbolId(42));
    table.index_to_symbol.push(SymbolId(43));
    assert_eq!(table.index_to_symbol[0], SymbolId(42));
    assert_eq!(table.index_to_symbol[1], SymbolId(43));
}

#[test]
fn test_symbol_to_index_and_index_to_symbol_consistency() {
    let mut table = create_minimal_table();
    let symbols = vec![SymbolId(10), SymbolId(20), SymbolId(30)];
    for (idx, &sym) in symbols.iter().enumerate() {
        table.symbol_to_index.insert(sym, idx);
        table.index_to_symbol.push(sym);
    }
    // Verify consistency
    for (idx, &sym) in symbols.iter().enumerate() {
        assert_eq!(table.symbol_to_index[&sym], idx);
        assert_eq!(table.index_to_symbol[idx], sym);
    }
}

#[test]
fn test_symbol_to_index_ordered_map() {
    let mut table = create_minimal_table();
    table.symbol_to_index.insert(SymbolId(100), 0);
    table.symbol_to_index.insert(SymbolId(10), 1);
    table.symbol_to_index.insert(SymbolId(50), 2);
    let keys: Vec<_> = table.symbol_to_index.keys().copied().collect();
    // BTreeMap maintains sorted order
    assert_eq!(keys[0], SymbolId(10));
    assert_eq!(keys[1], SymbolId(50));
    assert_eq!(keys[2], SymbolId(100));
}

// =============================================================================
// 5. Action enum variant properties (Clone, PartialEq, Debug)
// =============================================================================

#[test]
fn test_action_shift_clone() {
    let a = Action::Shift(StateId(5));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_action_reduce_clone() {
    let a = Action::Reduce(RuleId(3));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_action_accept_clone() {
    let a = Action::Accept;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_action_fork_clone() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_action_debug_shift() {
    let a = Action::Shift(StateId(5));
    let debug_str = format!("{:?}", a);
    assert!(debug_str.contains("Shift"));
}

#[test]
fn test_action_debug_reduce() {
    let a = Action::Reduce(RuleId(3));
    let debug_str = format!("{:?}", a);
    assert!(debug_str.contains("Reduce"));
}

#[test]
fn test_action_debug_accept() {
    let a = Action::Accept;
    let debug_str = format!("{:?}", a);
    assert!(debug_str.contains("Accept"));
}

#[test]
fn test_action_debug_error() {
    let a = Action::Error;
    let debug_str = format!("{:?}", a);
    assert!(debug_str.contains("Error"));
}

#[test]
fn test_action_debug_recover() {
    let a = Action::Recover;
    let debug_str = format!("{:?}", a);
    assert!(debug_str.contains("Recover"));
}

#[test]
fn test_action_debug_fork() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let debug_str = format!("{:?}", a);
    assert!(debug_str.contains("Fork"));
}

#[test]
fn test_action_partialeq_shift() {
    assert_eq!(Action::Shift(StateId(1)), Action::Shift(StateId(1)));
    assert_ne!(Action::Shift(StateId(1)), Action::Shift(StateId(2)));
}

#[test]
fn test_action_partialeq_reduce() {
    assert_eq!(Action::Reduce(RuleId(1)), Action::Reduce(RuleId(1)));
    assert_ne!(Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2)));
}

#[test]
fn test_action_partialeq_accept() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn test_action_partialeq_error() {
    assert_eq!(Action::Error, Action::Error);
}

#[test]
fn test_action_partialeq_recover() {
    assert_eq!(Action::Recover, Action::Recover);
}

#[test]
fn test_action_partialeq_fork() {
    let fork1 = Action::Fork(vec![Action::Shift(StateId(1))]);
    let fork2 = Action::Fork(vec![Action::Shift(StateId(1))]);
    let fork3 = Action::Fork(vec![Action::Shift(StateId(2))]);
    assert_eq!(fork1, fork2);
    assert_ne!(fork1, fork3);
}

#[test]
fn test_action_cross_variant_inequality() {
    assert_ne!(Action::Accept, Action::Error);
    assert_ne!(Action::Shift(StateId(1)), Action::Reduce(RuleId(1)));
    assert_ne!(Action::Accept, Action::Recover);
    assert_ne!(Action::Shift(StateId(1)), Action::Accept);
}

// =============================================================================
// 6. ActionCell with multiple actions (GLR fork)
// =============================================================================

#[test]
fn test_action_cell_empty() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
}

#[test]
fn test_action_cell_single_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(1))];
    assert_eq!(cell.len(), 1);
    assert_eq!(cell[0], Action::Shift(StateId(1)));
}

#[test]
fn test_action_cell_multiple_actions() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ];
    assert_eq!(cell.len(), 3);
    assert_eq!(cell[0], Action::Shift(StateId(1)));
    assert_eq!(cell[1], Action::Reduce(RuleId(2)));
    assert_eq!(cell[2], Action::Accept);
}

#[test]
fn test_action_cell_shift_reduce_conflict() {
    let cell: ActionCell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    assert_eq!(cell.len(), 2);
    // This represents a shift-reduce conflict in GLR parsing
}

#[test]
fn test_action_cell_reduce_reduce_conflict() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    assert_eq!(cell.len(), 2);
    // This represents a reduce-reduce conflict in GLR parsing
}

#[test]
fn test_action_cell_with_fork() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let cell: ActionCell = vec![fork];
    assert_eq!(cell.len(), 1);
    match &cell[0] {
        Action::Fork(inner) => assert_eq!(inner.len(), 2),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn test_action_cell_with_error_and_shift() {
    let cell: ActionCell = vec![Action::Error, Action::Shift(StateId(1))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn test_action_cell_iteration() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ];
    let mut count = 0;
    for _action in cell.iter() {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn test_action_cell_clone() {
    let cell: ActionCell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    let cell2 = cell.clone();
    assert_eq!(cell, cell2);
}

// =============================================================================
// 7. ParseRule creation and properties
// =============================================================================

#[test]
fn test_parse_rule_basic() {
    let rule = create_rule(10, 2);
    assert_eq!(rule.lhs, SymbolId(10));
    assert_eq!(rule.rhs_len, 2);
}

#[test]
fn test_parse_rule_zero_rhs() {
    let rule = create_rule(10, 0);
    assert_eq!(rule.rhs_len, 0);
}

#[test]
fn test_parse_rule_large_rhs() {
    let rule = create_rule(10, u16::MAX);
    assert_eq!(rule.rhs_len, u16::MAX);
}

#[test]
fn test_parse_rule_multiple_rules() {
    let mut rules = vec![];
    rules.push(create_rule(10, 1));
    rules.push(create_rule(11, 2));
    rules.push(create_rule(12, 3));
    assert_eq!(rules.len(), 3);
}

#[test]
fn test_parse_rule_clone() {
    let rule = create_rule(10, 2);
    let rule2 = rule.clone();
    assert_eq!(rule.lhs, rule2.lhs);
    assert_eq!(rule.rhs_len, rule2.rhs_len);
}

#[test]
fn test_parse_rule_debug() {
    let rule = create_rule(10, 2);
    let debug_str = format!("{:?}", rule);
    assert!(debug_str.contains("lhs"));
    assert!(debug_str.contains("rhs_len"));
}

#[test]
fn test_parse_rule_in_table() {
    let mut table = create_minimal_table();
    table.rules.push(create_rule(10, 1));
    table.rules.push(create_rule(11, 2));
    assert_eq!(table.rules.len(), 2);
}

// =============================================================================
// 8. SymbolMetadata creation
// =============================================================================

#[test]
fn test_symbol_metadata_terminal() {
    let meta = create_symbol_meta("token", 1, true);
    assert_eq!(meta.name, "token");
    assert!(meta.is_terminal);
    assert_eq!(meta.symbol_id, SymbolId(1));
}

#[test]
fn test_symbol_metadata_nonterminal() {
    let meta = create_symbol_meta("expr", 10, false);
    assert_eq!(meta.name, "expr");
    assert!(!meta.is_terminal);
    assert_eq!(meta.symbol_id, SymbolId(10));
}

#[test]
fn test_symbol_metadata_visibility() {
    let mut meta = create_symbol_meta("token", 1, true);
    assert!(meta.is_visible);
    meta.is_visible = false;
    assert!(!meta.is_visible);
}

#[test]
fn test_symbol_metadata_named() {
    let mut meta = create_symbol_meta("token", 1, true);
    assert!(meta.is_named);
    meta.is_named = false;
    assert!(!meta.is_named);
}

#[test]
fn test_symbol_metadata_supertype() {
    let mut meta = create_symbol_meta("_expression", 10, false);
    assert!(!meta.is_supertype);
    meta.is_supertype = true;
    assert!(meta.is_supertype);
}

#[test]
fn test_symbol_metadata_extra() {
    let mut meta = create_symbol_meta("whitespace", 2, true);
    assert!(!meta.is_extra);
    meta.is_extra = true;
    assert!(meta.is_extra);
}

#[test]
fn test_symbol_metadata_fragile() {
    let mut meta = create_symbol_meta("token", 1, true);
    assert!(!meta.is_fragile);
    meta.is_fragile = true;
    assert!(meta.is_fragile);
}

#[test]
fn test_symbol_metadata_clone() {
    let meta = create_symbol_meta("token", 1, true);
    let meta2 = meta.clone();
    assert_eq!(meta.name, meta2.name);
    assert_eq!(meta.symbol_id, meta2.symbol_id);
}

#[test]
fn test_symbol_metadata_in_table() {
    let mut table = create_minimal_table();
    table.symbol_metadata.push(create_symbol_meta("a", 1, true));
    table
        .symbol_metadata
        .push(create_symbol_meta("expr", 10, false));
    assert_eq!(table.symbol_metadata.len(), 2);
}

// =============================================================================
// 9. Large tables (many states)
// =============================================================================

#[test]
fn test_large_action_table_1000_states() {
    let mut table = create_sized_table(1000, 50);
    for _ in 0..1000 {
        table.action_table.push(vec![vec![Action::Error]]);
    }
    assert_eq!(table.action_table.len(), 1000);
}

#[test]
fn test_large_goto_table_500_states() {
    let mut table = create_sized_table(500, 100);
    for i in 0..500 {
        table.goto_table.push(vec![StateId(i as u16)]);
    }
    assert_eq!(table.goto_table.len(), 500);
}

#[test]
fn test_large_symbol_mapping() {
    let mut table = create_minimal_table();
    for i in 0..200 {
        table.symbol_to_index.insert(SymbolId(i), i as usize);
        table.index_to_symbol.push(SymbolId(i));
    }
    assert_eq!(table.symbol_to_index.len(), 200);
    assert_eq!(table.index_to_symbol.len(), 200);
}

#[test]
fn test_large_symbol_metadata_collection() {
    let mut table = create_minimal_table();
    for i in 0..100 {
        let is_terminal = i % 2 == 0;
        table
            .symbol_metadata
            .push(create_symbol_meta(&format!("sym{}", i), i, is_terminal));
    }
    assert_eq!(table.symbol_metadata.len(), 100);
}

#[test]
fn test_large_rule_collection() {
    let mut table = create_minimal_table();
    for i in 0..50 {
        table.rules.push(create_rule(10 + i, i as u16));
    }
    assert_eq!(table.rules.len(), 50);
}

// =============================================================================
// 10. Table with all action types
// =============================================================================

#[test]
fn test_table_all_action_types() {
    let mut table = create_minimal_table();
    let mut cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
        Action::Error,
        Action::Recover,
    ];
    table.action_table.push(vec![cell]);
    assert_eq!(table.action_table[0].len(), 1);
    assert_eq!(table.action_table[0][0].len(), 5);
    assert_eq!(table.action_table[0][0][0], Action::Shift(StateId(1)));
    assert_eq!(table.action_table[0][0][1], Action::Reduce(RuleId(2)));
    assert_eq!(table.action_table[0][0][2], Action::Accept);
    assert_eq!(table.action_table[0][0][3], Action::Error);
    assert_eq!(table.action_table[0][0][4], Action::Recover);
}

#[test]
fn test_table_multiple_rows_different_actions() {
    let mut table = create_minimal_table();
    table
        .action_table
        .push(vec![vec![Action::Shift(StateId(1))]]);
    table
        .action_table
        .push(vec![vec![Action::Reduce(RuleId(2))]]);
    table.action_table.push(vec![vec![Action::Accept]]);
    table.action_table.push(vec![vec![Action::Error]]);
    table.action_table.push(vec![vec![Action::Recover]]);
    assert_eq!(table.action_table.len(), 5);
}

// =============================================================================
// 11. Fork action with nested actions
// =============================================================================

#[test]
fn test_fork_single_action() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1))]);
    match fork {
        Action::Fork(inner) => assert_eq!(inner.len(), 1),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn test_fork_two_actions() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    match fork {
        Action::Fork(inner) => {
            assert_eq!(inner.len(), 2);
            assert_eq!(inner[0], Action::Shift(StateId(1)));
            assert_eq!(inner[1], Action::Reduce(RuleId(2)));
        }
        _ => panic!("expected Fork"),
    }
}

#[test]
fn test_fork_multiple_reduces() {
    let fork = Action::Fork(vec![
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
        Action::Reduce(RuleId(3)),
    ]);
    match fork {
        Action::Fork(inner) => assert_eq!(inner.len(), 3),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn test_fork_in_action_cell() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let cell: ActionCell = vec![fork];
    assert_eq!(cell.len(), 1);
}

#[test]
fn test_fork_clone_preserves_inner_actions() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let fork2 = fork.clone();
    assert_eq!(fork, fork2);
}

// =============================================================================
// 12. Empty table invariants
// =============================================================================

#[test]
fn test_empty_table_has_all_empty_collections() {
    let table = create_minimal_table();
    assert!(table.action_table.is_empty());
    assert!(table.goto_table.is_empty());
    assert!(table.symbol_metadata.is_empty());
    assert!(table.symbol_to_index.is_empty());
    assert!(table.index_to_symbol.is_empty());
    assert!(table.rules.is_empty());
}

#[test]
fn test_empty_table_counts_are_zero() {
    let table = create_minimal_table();
    assert_eq!(table.state_count, 0);
    assert_eq!(table.symbol_count, 0);
    assert_eq!(table.token_count, 0);
    assert_eq!(table.external_token_count, 0);
}

#[test]
fn test_empty_table_default_symbols() {
    let table = create_minimal_table();
    assert_eq!(table.eof_symbol, SymbolId(0));
    assert_eq!(table.start_symbol, SymbolId(0));
}

#[test]
fn test_empty_table_initial_state() {
    let table = create_minimal_table();
    assert_eq!(table.initial_state, StateId(0));
}

#[test]
fn test_empty_table_nonterminal_map_indexing() {
    let table = create_minimal_table();
    assert_eq!(table.goto_indexing, GotoIndexing::NonterminalMap);
}

#[test]
fn test_empty_table_grammar_is_default() {
    let table = create_minimal_table();
    assert_eq!(table.grammar.name, "default");
}

// =============================================================================
// 13. Additional coverage tests
// =============================================================================

#[test]
fn test_action_cell_push_and_pop() {
    let mut cell: ActionCell = vec![Action::Shift(StateId(1))];
    cell.push(Action::Reduce(RuleId(2)));
    assert_eq!(cell.len(), 2);
    let popped = cell.pop();
    assert_eq!(popped, Some(Action::Reduce(RuleId(2))));
    assert_eq!(cell.len(), 1);
}

#[test]
fn test_parse_table_field_independence() {
    let mut table1 = create_minimal_table();
    let mut table2 = create_minimal_table();
    table1
        .action_table
        .push(vec![vec![Action::Shift(StateId(1))]]);
    assert!(table2.action_table.is_empty());
}

#[test]
fn test_symbol_metadata_all_flags_combination() {
    let mut meta = create_symbol_meta("test", 1, true);
    meta.is_visible = false;
    meta.is_named = false;
    meta.is_supertype = true;
    meta.is_extra = true;
    meta.is_fragile = true;

    assert!(!meta.is_visible);
    assert!(!meta.is_named);
    assert!(meta.is_supertype);
    assert!(meta.is_extra);
    assert!(meta.is_fragile);
}

#[test]
fn test_multiple_tables_independent() {
    let mut t1 = create_minimal_table();
    let mut t2 = create_minimal_table();
    let mut t3 = create_minimal_table();

    t1.state_count = 10;
    t2.state_count = 20;
    t3.state_count = 30;

    assert_eq!(t1.state_count, 10);
    assert_eq!(t2.state_count, 20);
    assert_eq!(t3.state_count, 30);
}

#[test]
fn test_action_shift_with_max_state() {
    let action = Action::Shift(StateId(u16::MAX));
    assert_eq!(action, Action::Shift(StateId(u16::MAX)));
}

#[test]
fn test_action_reduce_with_max_rule() {
    let action = Action::Reduce(RuleId(u16::MAX));
    assert_eq!(action, Action::Reduce(RuleId(u16::MAX)));
}

#[test]
fn test_symbol_id_with_max_value() {
    let sym = SymbolId(u16::MAX);
    let meta = SymbolMetadata {
        name: "max_sym".to_string(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: sym,
    };
    assert_eq!(meta.symbol_id, SymbolId(u16::MAX));
}

#[test]
fn test_parse_rule_equality() {
    let r1 = create_rule(10, 2);
    let r2 = create_rule(10, 2);
    // ParseRule implements Clone, not Eq, but we can compare fields
    assert_eq!(r1.lhs, r2.lhs);
    assert_eq!(r1.rhs_len, r2.rhs_len);
}
