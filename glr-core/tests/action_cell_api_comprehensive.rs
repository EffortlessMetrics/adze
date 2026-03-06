//! Comprehensive tests for ActionCell, Action, and related ID types in adze-glr-core.

use adze_glr_core::*;
use std::collections::BTreeMap;

// ============================================================================
// 1. Action enum variants — construction, matching, equality
// ============================================================================

#[test]
fn action_shift_construction() {
    let a = Action::Shift(StateId(5));
    assert!(matches!(a, Action::Shift(StateId(5))));
}

#[test]
fn action_reduce_construction() {
    let a = Action::Reduce(RuleId(3));
    assert!(matches!(a, Action::Reduce(RuleId(3))));
}

#[test]
fn action_accept_construction() {
    let a = Action::Accept;
    assert!(matches!(a, Action::Accept));
}

#[test]
fn action_error_construction() {
    let a = Action::Error;
    assert!(matches!(a, Action::Error));
}

#[test]
fn action_recover_construction() {
    let a = Action::Recover;
    assert!(matches!(a, Action::Recover));
}

#[test]
fn action_fork_construction() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    match &a {
        Action::Fork(actions) => {
            assert_eq!(actions.len(), 2);
            assert!(matches!(actions[0], Action::Shift(StateId(1))));
            assert!(matches!(actions[1], Action::Reduce(RuleId(2))));
        }
        _ => panic!("expected Fork"),
    }
}

#[test]
fn action_equality_same_variant() {
    assert_eq!(Action::Shift(StateId(1)), Action::Shift(StateId(1)));
    assert_eq!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(0)));
    assert_eq!(Action::Accept, Action::Accept);
    assert_eq!(Action::Error, Action::Error);
    assert_eq!(Action::Recover, Action::Recover);
}

#[test]
fn action_inequality_different_variant() {
    assert_ne!(Action::Shift(StateId(1)), Action::Reduce(RuleId(1)));
    assert_ne!(Action::Accept, Action::Error);
    assert_ne!(Action::Error, Action::Recover);
}

#[test]
fn action_inequality_same_variant_different_value() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
}

#[test]
fn action_fork_equality() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let b = Action::Fork(vec![Action::Shift(StateId(1))]);
    assert_eq!(a, b);
}

#[test]
fn action_fork_inequality_different_contents() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let b = Action::Fork(vec![Action::Shift(StateId(2))]);
    assert_ne!(a, b);
}

#[test]
fn action_fork_empty() {
    let a = Action::Fork(vec![]);
    assert!(matches!(a, Action::Fork(ref v) if v.is_empty()));
}

// ============================================================================
// 2. StateId / SymbolId / RuleId — construction, comparison, ordering
// ============================================================================

#[test]
fn state_id_construction_and_equality() {
    let a = StateId(0);
    let b = StateId(0);
    let c = StateId(42);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn symbol_id_construction_and_equality() {
    let a = SymbolId(10);
    let b = SymbolId(10);
    let c = SymbolId(20);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn rule_id_construction_and_equality() {
    let a = RuleId(0);
    let b = RuleId(0);
    let c = RuleId(1);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn id_ordering() {
    assert!(StateId(0) < StateId(1));
    assert!(SymbolId(5) < SymbolId(10));
    assert!(RuleId(3) > RuleId(2));
}

#[test]
fn id_inner_value_access() {
    assert_eq!(StateId(99).0, 99);
    assert_eq!(SymbolId(255).0, 255);
    assert_eq!(RuleId(u16::MAX).0, u16::MAX);
}

#[test]
fn id_copy_semantics() {
    let s = StateId(7);
    let s2 = s;
    assert_eq!(s, s2); // both usable after copy
}

#[test]
fn id_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    assert_eq!(set.len(), 2);
}

// ============================================================================
// 3. ActionCell manipulation — push, contains, multi-action
// ============================================================================

#[test]
fn action_cell_empty() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
    assert_eq!(cell.len(), 0);
}

#[test]
fn action_cell_single_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(1))];
    assert_eq!(cell.len(), 1);
    assert!(cell.contains(&Action::Shift(StateId(1))));
}

#[test]
fn action_cell_multiple_actions() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
    ];
    assert_eq!(cell.len(), 3);
    assert!(cell.contains(&Action::Accept));
}

#[test]
fn action_cell_push() {
    let mut cell: ActionCell = vec![Action::Error];
    cell.push(Action::Recover);
    assert_eq!(cell.len(), 2);
    assert!(cell.contains(&Action::Recover));
}

#[test]
fn action_cell_contains_fork() {
    let fork = Action::Fork(vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(1))]);
    let cell: ActionCell = vec![fork.clone()];
    assert!(cell.contains(&fork));
}

#[test]
fn action_cell_iter() {
    let cell: ActionCell = vec![Action::Shift(StateId(0)), Action::Shift(StateId(1))];
    let shifts: Vec<_> = cell
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .collect();
    assert_eq!(shifts.len(), 2);
}

#[test]
fn action_cell_dedup_via_sort() {
    // ActionCell is just Vec<Action>; demonstrate dedup pattern
    let mut cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(1)),
        Action::Error,
    ];
    cell.dedup();
    assert_eq!(cell.len(), 2);
}

#[test]
fn action_cell_glr_shift_reduce_conflict() {
    // A GLR cell can hold both Shift and Reduce simultaneously
    let cell: ActionCell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))];
    let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
    let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
    assert!(has_shift && has_reduce);
}

// ============================================================================
// 4. ParseTable construction and querying
// ============================================================================

/// Helper: build a minimal ParseTable with one state, one terminal, one rule.
fn make_minimal_table() -> ParseTable {
    let eof = SymbolId(0);
    let terminal_a = SymbolId(1);
    let start_nt = SymbolId(10);

    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(eof, 0);
    symbol_to_index.insert(terminal_a, 1);

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_nt, 0);

    // State 0: Shift(1) on terminal_a, Accept on EOF
    // State 1: Reduce(0) on EOF
    let action_table = vec![
        vec![vec![Action::Accept], vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Reduce(RuleId(0))], vec![]],
    ];

    let goto_table = vec![
        vec![StateId(u16::MAX)], // state 0
        vec![StateId(u16::MAX)], // state 1
    ];

    ParseTable {
        action_table,
        goto_table,
        state_count: 2,
        symbol_count: 3,
        symbol_to_index,
        index_to_symbol: vec![eof, terminal_a],
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start_nt,
        token_count: 2,
        rules: vec![ParseRule {
            lhs: start_nt,
            rhs_len: 1,
        }],
        ..ParseTable::default()
    }
}

#[test]
fn parse_table_default_has_zero_states() {
    let pt = ParseTable::default();
    assert_eq!(pt.state_count, 0);
    assert_eq!(pt.symbol_count, 0);
    assert!(pt.action_table.is_empty());
    assert!(pt.goto_table.is_empty());
}

#[test]
fn parse_table_actions_returns_slice() {
    let pt = make_minimal_table();
    let actions = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 1);
    assert!(matches!(actions[0], Action::Shift(StateId(1))));
}

#[test]
fn parse_table_actions_returns_empty_for_unknown_symbol() {
    let pt = make_minimal_table();
    let actions = pt.actions(StateId(0), SymbolId(999));
    assert!(actions.is_empty());
}

#[test]
fn parse_table_actions_returns_empty_for_out_of_range_state() {
    let pt = make_minimal_table();
    let actions = pt.actions(StateId(999), SymbolId(1));
    assert!(actions.is_empty());
}

#[test]
fn parse_table_eof_and_start_symbol() {
    let pt = make_minimal_table();
    assert_eq!(pt.eof(), SymbolId(0));
    assert_eq!(pt.start_symbol(), SymbolId(10));
}

#[test]
fn parse_table_rule_lookup() {
    let pt = make_minimal_table();
    let (lhs, rhs_len) = pt.rule(RuleId(0));
    assert_eq!(lhs, SymbolId(10));
    assert_eq!(rhs_len, 1);
}

#[test]
fn parse_table_goto_no_edge_returns_none() {
    let pt = make_minimal_table();
    // goto table filled with u16::MAX sentinel → None
    assert_eq!(pt.goto(StateId(0), SymbolId(10)), None);
}

#[test]
fn parse_table_goto_valid_edge() {
    let mut pt = make_minimal_table();
    // Manually set a valid goto edge: state 0, nonterminal 10 → state 1
    pt.goto_table[0][0] = StateId(1);
    assert_eq!(pt.goto(StateId(0), SymbolId(10)), Some(StateId(1)));
}

#[test]
fn parse_table_goto_unknown_nonterminal() {
    let pt = make_minimal_table();
    assert_eq!(pt.goto(StateId(0), SymbolId(999)), None);
}

#[test]
fn parse_table_error_symbol() {
    let pt = make_minimal_table();
    assert_eq!(pt.error_symbol(), SymbolId(0));
}

// ============================================================================
// 5. Fork action with nested and multiple sub-actions
// ============================================================================

#[test]
fn fork_with_three_alternatives() {
    let fork = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ]);
    if let Action::Fork(ref inner) = fork {
        assert_eq!(inner.len(), 3);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_nested_inside_fork() {
    let inner_fork = Action::Fork(vec![Action::Shift(StateId(2)), Action::Accept]);
    let outer = Action::Fork(vec![Action::Shift(StateId(0)), inner_fork.clone()]);
    if let Action::Fork(ref actions) = outer {
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[1], inner_fork);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_in_action_cell() {
    let cell: ActionCell = vec![
        Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]),
        Action::Error,
    ];
    assert_eq!(cell.len(), 2);
    let forks: Vec<_> = cell
        .iter()
        .filter(|a| matches!(a, Action::Fork(_)))
        .collect();
    assert_eq!(forks.len(), 1);
}

// ============================================================================
// 6. Serialization traits — Debug, Clone
// ============================================================================

#[test]
fn action_debug_format() {
    let a = Action::Shift(StateId(7));
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("Shift"));
    assert!(dbg.contains("7"));
}

#[test]
fn action_clone() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn state_id_debug_format() {
    let s = StateId(42);
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("42"));
}

#[test]
fn symbol_id_debug_format() {
    let s = SymbolId(255);
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("255"));
}

#[test]
fn parse_table_clone() {
    let pt = make_minimal_table();
    let pt2 = pt.clone();
    assert_eq!(pt2.state_count, pt.state_count);
    assert_eq!(pt2.eof(), pt.eof());
    assert_eq!(pt2.start_symbol(), pt.start_symbol());
}

#[test]
fn parse_table_debug_does_not_panic() {
    let pt = make_minimal_table();
    let dbg = format!("{:?}", pt);
    assert!(!dbg.is_empty());
}

#[test]
fn parse_rule_debug_and_clone() {
    let r = ParseRule {
        lhs: SymbolId(5),
        rhs_len: 3,
    };
    let r2 = r.clone();
    assert_eq!(r2.lhs, SymbolId(5));
    assert_eq!(r2.rhs_len, 3);
    let dbg = format!("{:?}", r);
    assert!(dbg.contains("5"));
}

#[test]
fn action_cell_clone_independence() {
    let mut cell: ActionCell = vec![Action::Shift(StateId(1))];
    let cell2 = cell.clone();
    cell.push(Action::Error);
    // clone is independent
    assert_eq!(cell.len(), 2);
    assert_eq!(cell2.len(), 1);
}

// ============================================================================
// 7. Edge cases
// ============================================================================

#[test]
fn action_with_max_state_id() {
    let a = Action::Shift(StateId(u16::MAX));
    assert!(matches!(a, Action::Shift(StateId(65535))));
}

#[test]
fn action_with_zero_ids() {
    let a = Action::Shift(StateId(0));
    let b = Action::Reduce(RuleId(0));
    assert!(matches!(a, Action::Shift(StateId(0))));
    assert!(matches!(b, Action::Reduce(RuleId(0))));
}
