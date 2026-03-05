#![cfg(feature = "test-api")]

//! Comprehensive tests for Action enum and ActionCell behaviour in adze-glr-core.
//!
//! Groups:
//!   1. Action variant construction (8)
//!   2. Action equality (8)
//!   3. Action Debug/Clone (8)
//!   4. Actions from parse table (7)
//!   5. Multi-action cells / GLR (8)
//!   6. Action pattern matching (8)
//!   7. Edge cases (8)

use adze_glr_core::{Action, ActionCell, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};

// ===========================================================================
// Helpers
// ===========================================================================

/// Minimal grammar: S → a
fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

/// Ambiguous grammar: E → a | E a  (shift/reduce conflict on `a`)
fn ambiguous_grammar() -> Grammar {
    GrammarBuilder::new("ambig")
        .token("a", "a")
        .rule("E", vec!["a"])
        .rule("E", vec!["E", "a"])
        .start("E")
        .build()
}

/// Build parse table from a grammar (handles normalization).
fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

// ===========================================================================
// 1. Action variant construction (8 tests)
// ===========================================================================

#[test]
fn construct_shift_zero() {
    let a = Action::Shift(StateId(0));
    assert!(matches!(a, Action::Shift(StateId(0))));
}

#[test]
fn construct_shift_nonzero() {
    let a = Action::Shift(StateId(42));
    assert!(matches!(a, Action::Shift(StateId(42))));
}

#[test]
fn construct_shift_max_u16() {
    let a = Action::Shift(StateId(u16::MAX));
    assert!(matches!(a, Action::Shift(StateId(u16::MAX))));
}

#[test]
fn construct_reduce_zero() {
    let a = Action::Reduce(RuleId(0));
    assert!(matches!(a, Action::Reduce(RuleId(0))));
}

#[test]
fn construct_reduce_large() {
    let a = Action::Reduce(RuleId(9999));
    assert!(matches!(a, Action::Reduce(RuleId(9999))));
}

#[test]
fn construct_accept() {
    let a = Action::Accept;
    assert!(matches!(a, Action::Accept));
}

#[test]
fn construct_error() {
    let a = Action::Error;
    assert!(matches!(a, Action::Error));
}

#[test]
fn construct_fork_with_inner_actions() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    assert!(matches!(a, Action::Fork(_)));
}

// ===========================================================================
// 2. Action equality (8 tests)
// ===========================================================================

#[test]
fn eq_shift_same_state() {
    assert_eq!(Action::Shift(StateId(5)), Action::Shift(StateId(5)));
}

#[test]
fn ne_shift_different_state() {
    assert_ne!(Action::Shift(StateId(5)), Action::Shift(StateId(6)));
}

#[test]
fn eq_reduce_same_rule() {
    assert_eq!(Action::Reduce(RuleId(3)), Action::Reduce(RuleId(3)));
}

#[test]
fn ne_reduce_different_rule() {
    assert_ne!(Action::Reduce(RuleId(3)), Action::Reduce(RuleId(4)));
}

#[test]
fn eq_accept_accept() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn ne_shift_vs_reduce() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn ne_accept_vs_error() {
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn ne_shift_vs_accept() {
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

// ===========================================================================
// 3. Action Debug / Clone (8 tests)
// ===========================================================================

#[test]
fn debug_shift_contains_state() {
    let dbg = format!("{:?}", Action::Shift(StateId(7)));
    assert!(dbg.contains("Shift"), "expected Shift in: {dbg}");
    assert!(dbg.contains("7"), "expected 7 in: {dbg}");
}

#[test]
fn debug_reduce_contains_rule() {
    let dbg = format!("{:?}", Action::Reduce(RuleId(11)));
    assert!(dbg.contains("Reduce"), "expected Reduce in: {dbg}");
    assert!(dbg.contains("11"), "expected 11 in: {dbg}");
}

#[test]
fn debug_accept_format() {
    let dbg = format!("{:?}", Action::Accept);
    assert!(dbg.contains("Accept"), "expected Accept in: {dbg}");
}

#[test]
fn debug_error_format() {
    let dbg = format!("{:?}", Action::Error);
    assert!(dbg.contains("Error"), "expected Error in: {dbg}");
}

#[test]
fn debug_fork_format() {
    let dbg = format!("{:?}", Action::Fork(vec![Action::Accept]));
    assert!(dbg.contains("Fork"), "expected Fork in: {dbg}");
}

#[test]
fn clone_shift_equals_original() {
    let orig = Action::Shift(StateId(99));
    let cloned = orig.clone();
    assert_eq!(orig, cloned);
}

#[test]
fn clone_reduce_equals_original() {
    let orig = Action::Reduce(RuleId(42));
    let cloned = orig.clone();
    assert_eq!(orig, cloned);
}

#[test]
fn clone_fork_equals_original() {
    let orig = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let cloned = orig.clone();
    assert_eq!(orig, cloned);
}

// ===========================================================================
// 4. Actions from parse table (7 tests)
// ===========================================================================

#[test]
fn table_has_nonzero_states() {
    let table = build_table(&simple_grammar());
    assert!(table.state_count > 0);
}

#[test]
fn table_initial_state_has_actions() {
    let table = build_table(&simple_grammar());
    let mut found = false;
    for &sym in table.symbol_to_index.keys() {
        if !table.actions(table.initial_state, sym).is_empty() {
            found = true;
            break;
        }
    }
    assert!(found, "initial state should have at least one action");
}

#[test]
fn table_contains_shift_action() {
    let table = build_table(&simple_grammar());
    let has_shift = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    });
    assert!(has_shift, "simple grammar table should contain a Shift");
}

#[test]
fn table_contains_reduce_action() {
    let table = build_table(&simple_grammar());
    let has_reduce = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    });
    assert!(has_reduce, "simple grammar table should contain a Reduce");
}

#[test]
fn table_contains_accept_action() {
    let table = build_table(&simple_grammar());
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "simple grammar table should contain Accept");
}

#[test]
fn table_unknown_symbol_returns_empty() {
    let table = build_table(&simple_grammar());
    let actions = table.actions(table.initial_state, SymbolId(60000));
    assert!(actions.is_empty());
}

#[test]
fn table_out_of_range_state_returns_empty() {
    let table = build_table(&simple_grammar());
    let out_of_range = StateId(u16::MAX);
    // Pick any symbol that exists
    if let Some(&sym) = table.symbol_to_index.keys().next() {
        assert!(table.actions(out_of_range, sym).is_empty());
    }
}

// ===========================================================================
// 5. Multi-action cells / GLR (8 tests)
// ===========================================================================

#[test]
fn action_cell_type_is_vec() {
    let cell: ActionCell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn action_cell_empty() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
}

#[test]
fn action_cell_single_shift() {
    let cell: ActionCell = vec![Action::Shift(StateId(3))];
    assert_eq!(cell.len(), 1);
    assert!(matches!(cell[0], Action::Shift(StateId(3))));
}

#[test]
fn action_cell_shift_reduce_conflict() {
    let cell: ActionCell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(2))];
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn action_cell_reduce_reduce_conflict() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    let reduce_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert_eq!(reduce_count, 2);
}

#[test]
fn ambiguous_grammar_produces_multi_action_or_fork() {
    // E → a | E a creates a shift/reduce conflict.
    // The LR(1) builder may resolve it deterministically (choosing shift),
    // but the table should still be valid and contain the core action types.
    let table = build_table(&ambiguous_grammar());
    assert!(table.state_count > 0);

    let mut has_shift = false;
    let mut has_reduce = false;
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                match action {
                    Action::Shift(_) => has_shift = true,
                    Action::Reduce(_) => has_reduce = true,
                    Action::Fork(_) => {
                        has_shift = true;
                        has_reduce = true;
                    }
                    _ => {}
                }
            }
        }
    }
    assert!(has_shift, "ambiguous grammar table should contain Shift");
    assert!(has_reduce, "ambiguous grammar table should contain Reduce");
}

#[test]
fn action_cell_preserves_insertion_order() {
    let cell: ActionCell = vec![
        Action::Reduce(RuleId(10)),
        Action::Shift(StateId(20)),
        Action::Accept,
    ];
    assert!(matches!(cell[0], Action::Reduce(RuleId(10))));
    assert!(matches!(cell[1], Action::Shift(StateId(20))));
    assert!(matches!(cell[2], Action::Accept));
}

#[test]
fn fork_action_contains_multiple_inner_actions() {
    let fork = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ]);
    if let Action::Fork(inner) = &fork {
        assert_eq!(inner.len(), 3);
        assert!(inner.iter().any(|a| matches!(a, Action::Shift(_))));
        assert_eq!(
            inner
                .iter()
                .filter(|a| matches!(a, Action::Reduce(_)))
                .count(),
            2,
        );
    } else {
        panic!("expected Fork variant");
    }
}

// ===========================================================================
// 6. Action pattern matching (8 tests)
// ===========================================================================

#[test]
fn match_shift_extracts_state_id() {
    let a = Action::Shift(StateId(77));
    match a {
        Action::Shift(sid) => assert_eq!(sid, StateId(77)),
        _ => panic!("expected Shift"),
    }
}

#[test]
fn match_reduce_extracts_rule_id() {
    let a = Action::Reduce(RuleId(33));
    match a {
        Action::Reduce(rid) => assert_eq!(rid, RuleId(33)),
        _ => panic!("expected Reduce"),
    }
}

#[test]
fn match_accept_has_no_payload() {
    let a = Action::Accept;
    assert!(matches!(a, Action::Accept));
}

#[test]
fn match_error_has_no_payload() {
    let a = Action::Error;
    assert!(matches!(a, Action::Error));
}

#[test]
fn match_fork_extracts_inner_vec() {
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    match a {
        Action::Fork(ref inner) => assert_eq!(inner.len(), 2),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn is_shift_predicate() {
    let actions = [
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
    ];
    let shifts: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .collect();
    assert_eq!(shifts.len(), 1);
}

#[test]
fn is_reduce_predicate() {
    let actions = [
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Shift(StateId(2)),
    ];
    let reduces: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .collect();
    assert_eq!(reduces.len(), 2);
}

#[test]
fn match_all_variants_exhaustive() {
    let variants = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];
    for v in &variants {
        let label = match v {
            Action::Shift(_) => "shift",
            Action::Reduce(_) => "reduce",
            Action::Accept => "accept",
            Action::Error => "error",
            Action::Recover => "recover",
            Action::Fork(_) => "fork",
            _ => "unknown",
        };
        assert_ne!(label, "unknown", "unhandled variant: {v:?}");
    }
}

// ===========================================================================
// 7. Edge cases (8 tests)
// ===========================================================================

#[test]
fn shift_state_zero_is_valid() {
    let a = Action::Shift(StateId(0));
    if let Action::Shift(sid) = a {
        assert_eq!(sid.0, 0);
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn shift_state_max_is_valid() {
    let a = Action::Shift(StateId(u16::MAX));
    if let Action::Shift(sid) = a {
        assert_eq!(sid.0, u16::MAX);
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn reduce_rule_zero_is_valid() {
    let a = Action::Reduce(RuleId(0));
    if let Action::Reduce(rid) = a {
        assert_eq!(rid.0, 0);
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn reduce_rule_max_is_valid() {
    let a = Action::Reduce(RuleId(u16::MAX));
    if let Action::Reduce(rid) = a {
        assert_eq!(rid.0, u16::MAX);
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn fork_empty_inner_is_representable() {
    let a = Action::Fork(vec![]);
    if let Action::Fork(ref inner) = a {
        assert!(inner.is_empty());
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_nested_is_representable() {
    let a = Action::Fork(vec![Action::Fork(vec![Action::Accept])]);
    if let Action::Fork(outer) = &a {
        assert_eq!(outer.len(), 1);
        assert!(matches!(outer[0], Action::Fork(_)));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn recover_variant_exists() {
    let a = Action::Recover;
    assert!(matches!(a, Action::Recover));
    assert_eq!(format!("{a:?}"), "Recover");
}

#[test]
fn builder_grammar_produces_valid_table() {
    let grammar = GrammarBuilder::new("tiny")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count > 0);

    // Must have Shift, Reduce, and Accept somewhere
    let mut has_shift = false;
    let mut has_reduce = false;
    let mut has_accept = false;
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                match action {
                    Action::Shift(_) => has_shift = true,
                    Action::Reduce(_) => has_reduce = true,
                    Action::Accept => has_accept = true,
                    _ => {}
                }
            }
        }
    }
    assert!(has_shift, "expected Shift in builder grammar table");
    assert!(has_reduce, "expected Reduce in builder grammar table");
    assert!(has_accept, "expected Accept in builder grammar table");
}
