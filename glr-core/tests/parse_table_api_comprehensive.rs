//! Comprehensive tests for the ParseTable API and related types in adze-glr-core.

use adze_glr_core::{
    Action, ActionCell, ConflictType, GotoIndexing, LexMode, ParseRule, ParseTable, SymbolMetadata,
};
use adze_ir::{RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal ParseTable with sensible defaults for testing.
fn make_table(num_states: usize, num_terminal_cols: usize, num_goto_cols: usize) -> ParseTable {
    ParseTable {
        state_count: num_states,
        action_table: vec![vec![vec![]; num_terminal_cols]; num_states],
        goto_table: vec![vec![StateId(u16::MAX); num_goto_cols]; num_states],
        ..Default::default()
    }
}

/// Build symbol_to_index / index_to_symbol pair for terminal IDs.
fn build_terminal_mapping(ids: &[u16]) -> (BTreeMap<SymbolId, usize>, Vec<SymbolId>) {
    let mut s2i = BTreeMap::new();
    let mut i2s = Vec::new();
    for (col, &id) in ids.iter().enumerate() {
        s2i.insert(SymbolId(id), col);
        i2s.push(SymbolId(id));
    }
    (s2i, i2s)
}

// ===========================================================================
// 1. Action enum — variant construction
// ===========================================================================

#[test]
fn action_shift_holds_state() {
    let a = Action::Shift(StateId(7));
    assert!(matches!(a, Action::Shift(StateId(7))));
}

#[test]
fn action_reduce_holds_rule() {
    let a = Action::Reduce(RuleId(3));
    assert!(matches!(a, Action::Reduce(RuleId(3))));
}

#[test]
fn action_accept_variant() {
    let a = Action::Accept;
    assert!(matches!(a, Action::Accept));
}

#[test]
fn action_error_variant() {
    let a = Action::Error;
    assert!(matches!(a, Action::Error));
}

#[test]
fn action_recover_variant() {
    let a = Action::Recover;
    assert!(matches!(a, Action::Recover));
}

#[test]
fn action_fork_holds_inner_actions() {
    let inner = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    let a = Action::Fork(inner.clone());
    match a {
        Action::Fork(v) => assert_eq!(v.len(), 2),
        _ => panic!("expected Fork"),
    }
}

// ===========================================================================
// 2. Action — Clone, PartialEq, Debug
// ===========================================================================

#[test]
fn action_clone_equals_original() {
    let actions = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![Action::Accept]),
    ];
    for a in &actions {
        assert_eq!(a, &a.clone());
    }
}

#[test]
fn action_debug_contains_variant_name() {
    assert!(format!("{:?}", Action::Shift(StateId(1))).contains("Shift"));
    assert!(format!("{:?}", Action::Reduce(RuleId(2))).contains("Reduce"));
    assert!(format!("{:?}", Action::Accept).contains("Accept"));
    assert!(format!("{:?}", Action::Error).contains("Error"));
    assert!(format!("{:?}", Action::Recover).contains("Recover"));
    assert!(format!("{:?}", Action::Fork(vec![])).contains("Fork"));
}

#[test]
fn action_partial_eq_different_variants() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
    assert_ne!(Action::Accept, Action::Error);
    assert_ne!(Action::Recover, Action::Accept);
}

#[test]
fn action_partial_eq_same_variant_different_payload() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
}

#[test]
fn action_eq_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(5)));
    set.insert(Action::Shift(StateId(5))); // duplicate
    assert_eq!(set.len(), 1);
    set.insert(Action::Shift(StateId(6)));
    assert_eq!(set.len(), 2);
}

// ===========================================================================
// 3. ParseTable — default / empty construction
// ===========================================================================

#[test]
fn default_parse_table_is_empty() {
    let pt = ParseTable::default();
    assert_eq!(pt.state_count, 0);
    assert!(pt.action_table.is_empty());
    assert!(pt.goto_table.is_empty());
    assert!(pt.symbol_to_index.is_empty());
    assert!(pt.index_to_symbol.is_empty());
    assert!(pt.rules.is_empty());
}

#[test]
fn default_parse_table_eof_symbol() {
    let pt = ParseTable::default();
    assert_eq!(pt.eof_symbol, SymbolId(0));
    assert_eq!(pt.eof(), SymbolId(0));
}

#[test]
fn default_parse_table_start_symbol() {
    let pt = ParseTable::default();
    assert_eq!(pt.start_symbol, SymbolId(0));
    assert_eq!(pt.start_symbol(), SymbolId(0));
}

#[test]
fn default_parse_table_initial_state() {
    let pt = ParseTable::default();
    assert_eq!(pt.initial_state, StateId(0));
}

#[test]
fn default_parse_table_goto_indexing() {
    let pt = ParseTable::default();
    assert_eq!(pt.goto_indexing, GotoIndexing::NonterminalMap);
}

// ===========================================================================
// 4. ParseTable — single state
// ===========================================================================

#[test]
fn single_state_action_table_dimensions() {
    let pt = make_table(1, 3, 2);
    assert_eq!(pt.action_table.len(), 1);
    assert_eq!(pt.action_table[0].len(), 3);
}

#[test]
fn single_state_goto_table_dimensions() {
    let pt = make_table(1, 3, 2);
    assert_eq!(pt.goto_table.len(), 1);
    assert_eq!(pt.goto_table[0].len(), 2);
}

// ===========================================================================
// 5. ParseTable — multiple states
// ===========================================================================

#[test]
fn multiple_states_action_table_rows() {
    let pt = make_table(5, 4, 3);
    assert_eq!(pt.action_table.len(), 5);
    for row in &pt.action_table {
        assert_eq!(row.len(), 4);
    }
}

#[test]
fn multiple_states_goto_table_rows() {
    let pt = make_table(5, 4, 3);
    assert_eq!(pt.goto_table.len(), 5);
    for row in &pt.goto_table {
        assert_eq!(row.len(), 3);
    }
}

// ===========================================================================
// 6. ParseTable::actions() — action lookups
// ===========================================================================

#[test]
fn actions_returns_empty_for_unknown_symbol() {
    let pt = make_table(2, 3, 1);
    // No symbol_to_index mapping, so any lookup should return empty.
    let actions = pt.actions(StateId(0), SymbolId(42));
    assert!(actions.is_empty());
}

#[test]
fn actions_returns_empty_for_out_of_range_state() {
    let mut pt = make_table(2, 3, 1);
    let (s2i, i2s) = build_terminal_mapping(&[0, 1, 2]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;

    let actions = pt.actions(StateId(99), SymbolId(0));
    assert!(actions.is_empty());
}

#[test]
fn actions_returns_inserted_shift() {
    let mut pt = make_table(1, 2, 0);
    let (s2i, i2s) = build_terminal_mapping(&[10, 11]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;
    pt.action_table[0][0].push(Action::Shift(StateId(1)));

    let actions = pt.actions(StateId(0), SymbolId(10));
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], Action::Shift(StateId(1)));
}

#[test]
fn actions_returns_inserted_reduce() {
    let mut pt = make_table(1, 2, 0);
    let (s2i, i2s) = build_terminal_mapping(&[10, 11]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;
    pt.action_table[0][1].push(Action::Reduce(RuleId(0)));

    let actions = pt.actions(StateId(0), SymbolId(11));
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], Action::Reduce(RuleId(0)));
}

#[test]
fn actions_returns_accept() {
    let mut pt = make_table(1, 1, 0);
    let (s2i, i2s) = build_terminal_mapping(&[0]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;
    pt.action_table[0][0].push(Action::Accept);

    let actions = pt.actions(StateId(0), SymbolId(0));
    assert_eq!(actions, &[Action::Accept]);
}

#[test]
fn actions_multiple_in_cell() {
    let mut pt = make_table(1, 1, 0);
    let (s2i, i2s) = build_terminal_mapping(&[5]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;

    pt.action_table[0][0].push(Action::Shift(StateId(2)));
    pt.action_table[0][0].push(Action::Reduce(RuleId(1)));

    let actions = pt.actions(StateId(0), SymbolId(5));
    assert_eq!(actions.len(), 2);
}

// ===========================================================================
// 7. ParseTable::goto() — goto lookups
// ===========================================================================

#[test]
fn goto_returns_none_for_unmapped_nonterminal() {
    let pt = make_table(1, 0, 2);
    assert!(pt.goto(StateId(0), SymbolId(99)).is_none());
}

#[test]
fn goto_returns_none_for_sentinel_value() {
    let mut pt = make_table(1, 0, 2);
    pt.nonterminal_to_index.insert(SymbolId(10), 0);
    // The helper initializes goto cells to u16::MAX (sentinel).
    assert!(pt.goto(StateId(0), SymbolId(10)).is_none());
}

#[test]
fn goto_returns_valid_target_state() {
    let mut pt = make_table(2, 0, 2);
    pt.nonterminal_to_index.insert(SymbolId(10), 0);
    pt.goto_table[0][0] = StateId(1);

    assert_eq!(pt.goto(StateId(0), SymbolId(10)), Some(StateId(1)));
}

#[test]
fn goto_returns_none_for_out_of_range_state() {
    let mut pt = make_table(1, 0, 1);
    pt.nonterminal_to_index.insert(SymbolId(10), 0);
    assert!(pt.goto(StateId(99), SymbolId(10)).is_none());
}

#[test]
fn goto_zero_is_valid_target() {
    let mut pt = make_table(2, 0, 1);
    pt.nonterminal_to_index.insert(SymbolId(10), 0);
    pt.goto_table[1][0] = StateId(0);
    // StateId(0) != u16::MAX so it should be returned.
    assert_eq!(pt.goto(StateId(1), SymbolId(10)), Some(StateId(0)));
}

// ===========================================================================
// 8. Symbol-to-index / index-to-symbol consistency
// ===========================================================================

#[test]
fn symbol_index_roundtrip() {
    let ids = &[0u16, 5, 10, 20];
    let (s2i, i2s) = build_terminal_mapping(ids);

    for (&sym, &col) in &s2i {
        assert_eq!(i2s[col], sym, "index_to_symbol should map back to original");
    }
}

#[test]
fn index_to_symbol_length_matches_column_count() {
    let ids = &[1, 2, 3, 4, 5];
    let (s2i, i2s) = build_terminal_mapping(ids);
    assert_eq!(s2i.len(), i2s.len());
}

#[test]
fn symbol_to_index_is_injective() {
    let ids = &[0, 1, 2, 3, 4];
    let (s2i, _) = build_terminal_mapping(ids);
    let mut seen_cols = std::collections::HashSet::new();
    for &col in s2i.values() {
        assert!(seen_cols.insert(col), "column indices must be unique");
    }
}

// ===========================================================================
// 9. Edge cases — zero states, empty cells, max IDs
// ===========================================================================

#[test]
fn zero_states_actions_returns_empty() {
    let mut pt = make_table(0, 0, 0);
    let (s2i, i2s) = build_terminal_mapping(&[0]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;
    assert!(pt.actions(StateId(0), SymbolId(0)).is_empty());
}

#[test]
fn zero_states_goto_returns_none() {
    let mut pt = make_table(0, 0, 0);
    pt.nonterminal_to_index.insert(SymbolId(10), 0);
    assert!(pt.goto(StateId(0), SymbolId(10)).is_none());
}

#[test]
fn empty_action_cell_is_empty_vec() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
    assert_eq!(cell.len(), 0);
}

#[test]
fn max_symbol_id_in_mapping() {
    let max_id = u16::MAX;
    let (s2i, i2s) = build_terminal_mapping(&[max_id]);
    assert_eq!(s2i[&SymbolId(max_id)], 0);
    assert_eq!(i2s[0], SymbolId(max_id));
}

#[test]
fn max_state_id_shift() {
    let a = Action::Shift(StateId(u16::MAX));
    match a {
        Action::Shift(s) => assert_eq!(s.0, u16::MAX),
        _ => panic!("expected Shift"),
    }
}

#[test]
fn max_rule_id_reduce() {
    let a = Action::Reduce(RuleId(u16::MAX));
    match a {
        Action::Reduce(r) => assert_eq!(r.0, u16::MAX),
        _ => panic!("expected Reduce"),
    }
}

// ===========================================================================
// 10. Fork action variant
// ===========================================================================

#[test]
fn fork_with_empty_inner() {
    let a = Action::Fork(vec![]);
    match a {
        Action::Fork(v) => assert!(v.is_empty()),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn fork_nested_fork() {
    let inner = Action::Fork(vec![Action::Shift(StateId(1))]);
    let outer = Action::Fork(vec![inner.clone()]);
    match outer {
        Action::Fork(v) => {
            assert_eq!(v.len(), 1);
            assert_eq!(v[0], inner);
        }
        _ => panic!("expected Fork"),
    }
}

#[test]
fn fork_equality() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    let b = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    assert_eq!(a, b);
}

#[test]
fn fork_inequality_different_order() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    let b = Action::Fork(vec![Action::Reduce(RuleId(0)), Action::Shift(StateId(1))]);
    assert_ne!(a, b);
}

// ===========================================================================
// 11. GLR conflict scenarios
// ===========================================================================

#[test]
fn shift_reduce_conflict_in_single_cell() {
    let mut pt = make_table(1, 1, 0);
    let (s2i, i2s) = build_terminal_mapping(&[1]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;

    pt.action_table[0][0].push(Action::Shift(StateId(2)));
    pt.action_table[0][0].push(Action::Reduce(RuleId(0)));

    let actions = pt.actions(StateId(0), SymbolId(1));
    assert!(actions.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(actions.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn reduce_reduce_conflict_in_single_cell() {
    let mut pt = make_table(1, 1, 0);
    let (s2i, i2s) = build_terminal_mapping(&[1]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;

    pt.action_table[0][0].push(Action::Reduce(RuleId(0)));
    pt.action_table[0][0].push(Action::Reduce(RuleId(1)));

    let actions = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 2);
    assert!(actions.iter().all(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn three_way_conflict() {
    let mut pt = make_table(1, 1, 0);
    let (s2i, i2s) = build_terminal_mapping(&[1]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;

    pt.action_table[0][0].push(Action::Shift(StateId(1)));
    pt.action_table[0][0].push(Action::Reduce(RuleId(0)));
    pt.action_table[0][0].push(Action::Reduce(RuleId(1)));

    let actions = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 3);
}

// ===========================================================================
// 12. Table invariants — dimensions, consistency
// ===========================================================================

#[test]
fn action_table_row_widths_are_uniform() {
    let pt = make_table(4, 6, 3);
    let expected_width = pt.action_table[0].len();
    for row in &pt.action_table {
        assert_eq!(row.len(), expected_width);
    }
}

#[test]
fn goto_table_row_widths_are_uniform() {
    let pt = make_table(4, 6, 3);
    let expected_width = pt.goto_table[0].len();
    for row in &pt.goto_table {
        assert_eq!(row.len(), expected_width);
    }
}

#[test]
fn state_count_matches_table_rows() {
    let pt = make_table(3, 2, 2);
    assert_eq!(pt.state_count, pt.action_table.len());
    assert_eq!(pt.state_count, pt.goto_table.len());
}

// ===========================================================================
// 13. ParseTable convenience methods
// ===========================================================================

#[test]
fn terminal_boundary_default() {
    let pt = ParseTable::default();
    assert_eq!(pt.terminal_boundary(), 0);
}

#[test]
fn terminal_boundary_with_tokens() {
    let pt = ParseTable {
        token_count: 5,
        external_token_count: 3,
        ..Default::default()
    };
    assert_eq!(pt.terminal_boundary(), 8);
}

#[test]
fn is_terminal_below_boundary() {
    let pt = ParseTable {
        token_count: 3,
        ..Default::default()
    };
    assert!(pt.is_terminal(SymbolId(0)));
    assert!(pt.is_terminal(SymbolId(2)));
    assert!(!pt.is_terminal(SymbolId(3)));
}

#[test]
fn is_extra_checks_extras_list() {
    let mut pt = ParseTable::default();
    pt.extras.push(SymbolId(5));
    assert!(pt.is_extra(SymbolId(5)));
    assert!(!pt.is_extra(SymbolId(6)));
}

#[test]
fn error_symbol_is_zero() {
    let pt = ParseTable::default();
    assert_eq!(pt.error_symbol(), SymbolId(0));
}

#[test]
fn rule_accessor() {
    let mut pt = ParseTable::default();
    pt.rules.push(ParseRule {
        lhs: SymbolId(10),
        rhs_len: 3,
    });
    let (lhs, rhs_len) = pt.rule(RuleId(0));
    assert_eq!(lhs, SymbolId(10));
    assert_eq!(rhs_len, 3);
}

#[test]
fn grammar_accessor_returns_reference() {
    let pt = ParseTable::default();
    assert_eq!(pt.grammar().name, "default");
}

#[test]
fn lex_mode_default_for_missing_state() {
    let pt = ParseTable::default();
    let mode = pt.lex_mode(StateId(99));
    assert_eq!(mode.lex_state, 0);
    assert_eq!(mode.external_lex_state, 0);
}

#[test]
fn lex_mode_returns_configured_value() {
    let mut pt = ParseTable::default();
    pt.lex_modes.push(LexMode {
        lex_state: 42,
        external_lex_state: 7,
    });
    let mode = pt.lex_mode(StateId(0));
    assert_eq!(mode.lex_state, 42);
    assert_eq!(mode.external_lex_state, 7);
}

#[test]
fn valid_symbols_empty_for_default_table() {
    let pt = ParseTable::default();
    let mask = pt.valid_symbols(StateId(0));
    assert!(mask.is_empty());
}

#[test]
fn valid_symbols_reflects_nonempty_cells() {
    let mut pt = make_table(1, 3, 0);
    pt.token_count = 3;
    pt.action_table[0][1].push(Action::Shift(StateId(0)));

    let mask = pt.valid_symbols(StateId(0));
    assert_eq!(mask.len(), 3);
    assert!(!mask[0]);
    assert!(mask[1]);
    assert!(!mask[2]);
}

// ===========================================================================
// 14. ParseTable clone
// ===========================================================================

#[test]
fn parse_table_clone_is_independent() {
    let mut pt = make_table(1, 2, 1);
    let (s2i, i2s) = build_terminal_mapping(&[0, 1]);
    pt.symbol_to_index = s2i;
    pt.index_to_symbol = i2s;
    pt.action_table[0][0].push(Action::Accept);

    let pt2 = pt.clone();
    assert_eq!(pt2.actions(StateId(0), SymbolId(0)), &[Action::Accept]);
    assert_eq!(pt2.state_count, pt.state_count);
}

// ===========================================================================
// 15. ParseTable Debug
// ===========================================================================

#[test]
fn parse_table_debug_does_not_panic() {
    let pt = ParseTable::default();
    let dbg = format!("{:?}", pt);
    assert!(!dbg.is_empty());
}

// ===========================================================================
// 16. SymbolMetadata construction
// ===========================================================================

#[test]
fn symbol_metadata_fields() {
    let sm = SymbolMetadata {
        name: "identifier".into(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(1),
    };
    assert_eq!(sm.name, "identifier");
    assert!(sm.is_visible);
    assert!(sm.is_named);
    assert!(sm.is_terminal);
    assert_eq!(sm.symbol_id, SymbolId(1));
}

// ===========================================================================
// 17. ParseRule construction
// ===========================================================================

#[test]
fn parse_rule_fields() {
    let r = ParseRule {
        lhs: SymbolId(20),
        rhs_len: 5,
    };
    assert_eq!(r.lhs, SymbolId(20));
    assert_eq!(r.rhs_len, 5);
}

// ===========================================================================
// 18. GotoIndexing variants
// ===========================================================================

#[test]
fn goto_indexing_equality() {
    assert_eq!(GotoIndexing::NonterminalMap, GotoIndexing::NonterminalMap);
    assert_eq!(GotoIndexing::DirectSymbolId, GotoIndexing::DirectSymbolId);
    assert_ne!(GotoIndexing::NonterminalMap, GotoIndexing::DirectSymbolId);
}

// ===========================================================================
// 19. Conflict type
// ===========================================================================

#[test]
fn conflict_type_equality() {
    assert_eq!(ConflictType::ShiftReduce, ConflictType::ShiftReduce);
    assert_eq!(ConflictType::ReduceReduce, ConflictType::ReduceReduce);
    assert_ne!(ConflictType::ShiftReduce, ConflictType::ReduceReduce);
}
