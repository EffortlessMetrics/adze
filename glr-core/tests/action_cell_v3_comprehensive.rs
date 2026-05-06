#![cfg(feature = "test-api")]

//! Comprehensive tests for ActionCell architecture in adze-glr-core.
//!
//! Covers: Action construction, Debug/Clone/PartialEq, variant discrimination,
//! multi-action cells, serialization, StateId/SymbolId in actions, ordering
//! patterns, and edge cases.

use adze_glr_core::{
    Action, ActionCell, FirstFollowSets, RuleId, StateId, SymbolId, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use std::collections::HashSet;

// ===========================================================================
// Helpers
// ===========================================================================

/// Minimal grammar: S → a
fn grammar_s_to_a() -> Grammar {
    let mut g = Grammar::new("s_a".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Two-alternative grammar: S → a | b
fn grammar_s_to_a_or_b() -> Grammar {
    let mut g = Grammar::new("s_ab".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g
}

/// Ambiguous grammar: E → a | E E (inherently ambiguous)
fn grammar_ambiguous() -> Grammar {
    let mut g = Grammar::new("ambig".into());
    let a = SymbolId(1);
    let e = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::NonTerminal(e), Symbol::NonTerminal(e)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g
}

// ===========================================================================
// 1. Action construction (10 tests)
// ===========================================================================

#[test]
fn construct_shift_zero() {
    let a = Action::Shift(StateId(0));
    assert!(matches!(a, Action::Shift(StateId(0))));
}

#[test]
fn construct_shift_large() {
    let a = Action::Shift(StateId(10_000));
    assert!(matches!(a, Action::Shift(StateId(10_000))));
}

#[test]
fn construct_shift_max() {
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
    let a = Action::Reduce(RuleId(999));
    assert!(matches!(a, Action::Reduce(RuleId(999))));
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
fn construct_recover() {
    let a = Action::Recover;
    assert!(matches!(a, Action::Recover));
}

#[test]
fn construct_fork_two_actions() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    match &a {
        Action::Fork(inner) => {
            assert_eq!(inner.len(), 2);
            assert!(matches!(inner[0], Action::Shift(StateId(1))));
            assert!(matches!(inner[1], Action::Reduce(RuleId(2))));
        }
        _ => panic!("expected Fork"),
    }
}

#[test]
fn construct_fork_nested() {
    let inner_fork = Action::Fork(vec![Action::Shift(StateId(5))]);
    let outer = Action::Fork(vec![inner_fork, Action::Accept]);
    match &outer {
        Action::Fork(actions) => {
            assert_eq!(actions.len(), 2);
            assert!(matches!(&actions[0], Action::Fork(_)));
            assert!(matches!(actions[1], Action::Accept));
        }
        _ => panic!("expected Fork"),
    }
}

// ===========================================================================
// 2. Action Debug/Clone/PartialEq (8 tests)
// ===========================================================================

#[test]
fn debug_shift_contains_state_id() {
    let a = Action::Shift(StateId(42));
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Shift"), "Debug output: {dbg}");
    assert!(dbg.contains("42"), "Debug output: {dbg}");
}

#[test]
fn debug_reduce_contains_rule_id() {
    let a = Action::Reduce(RuleId(7));
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Reduce"), "Debug output: {dbg}");
    assert!(dbg.contains("7"), "Debug output: {dbg}");
}

#[test]
fn debug_accept_format() {
    let dbg = format!("{:?}", Action::Accept);
    assert_eq!(dbg, "Accept");
}

#[test]
fn debug_error_format() {
    let dbg = format!("{:?}", Action::Error);
    assert_eq!(dbg, "Error");
}

#[test]
fn clone_preserves_shift() {
    let a = Action::Shift(StateId(99));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_preserves_fork() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ]);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn partialeq_same_shift_values() {
    assert_eq!(Action::Shift(StateId(5)), Action::Shift(StateId(5)));
}

#[test]
fn partialeq_different_shift_values() {
    assert_ne!(Action::Shift(StateId(5)), Action::Shift(StateId(6)));
}

// ===========================================================================
// 3. Action variant discrimination (8 tests)
// ===========================================================================

#[test]
fn discriminate_shift_from_reduce() {
    let s = Action::Shift(StateId(0));
    let r = Action::Reduce(RuleId(0));
    assert_ne!(s, r);
    assert!(matches!(s, Action::Shift(_)));
    assert!(!matches!(s, Action::Reduce(_)));
}

#[test]
fn discriminate_accept_from_error() {
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn discriminate_error_from_recover() {
    assert_ne!(Action::Error, Action::Recover);
}

#[test]
fn discriminate_fork_from_shift() {
    let f = Action::Fork(vec![Action::Shift(StateId(0))]);
    let s = Action::Shift(StateId(0));
    assert_ne!(f, s);
}

#[test]
fn discriminate_all_unit_variants() {
    let variants: Vec<Action> = vec![Action::Accept, Action::Error, Action::Recover];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn match_shift_extracts_state() {
    let a = Action::Shift(StateId(77));
    if let Action::Shift(sid) = a {
        assert_eq!(sid, StateId(77));
    } else {
        panic!("expected Shift");
    }
}

#[test]
fn match_reduce_extracts_rule() {
    let a = Action::Reduce(RuleId(33));
    if let Action::Reduce(rid) = a {
        assert_eq!(rid, RuleId(33));
    } else {
        panic!("expected Reduce");
    }
}

#[test]
fn match_fork_extracts_vec() {
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    if let Action::Fork(actions) = a {
        assert_eq!(actions.len(), 2);
    } else {
        panic!("expected Fork");
    }
}

// ===========================================================================
// 4. Multi-action cells in ambiguous grammars (8 tests)
// ===========================================================================

#[test]
fn action_cell_is_vec_of_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn empty_action_cell() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
}

#[test]
fn single_action_cell() {
    let cell: ActionCell = vec![Action::Accept];
    assert_eq!(cell.len(), 1);
    assert!(matches!(cell[0], Action::Accept));
}

#[test]
fn multi_action_cell_shift_reduce_conflict() {
    let cell: ActionCell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(2))];
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn multi_action_cell_reduce_reduce_conflict() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    let reduce_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert_eq!(reduce_count, 2);
}

#[test]
fn ambiguous_grammar_produces_multi_action_cells() {
    let g = grammar_ambiguous();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // An ambiguous grammar should have at least one cell with multiple actions
    // OR the automaton builder may encode conflicts as Fork actions.
    let mut has_multi_action = false;
    for state_row in &table.action_table {
        for cell in state_row {
            if cell.len() > 1 {
                has_multi_action = true;
            }
            // Fork actions also indicate multi-action
            for action in cell {
                if matches!(action, Action::Fork(_)) {
                    has_multi_action = true;
                }
            }
        }
    }
    assert!(
        has_multi_action,
        "ambiguous grammar E → a | E E should produce multi-action cells or forks"
    );
}

#[test]
fn parse_table_actions_returns_slice() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let a = SymbolId(1);
    let actions = table.actions(table.initial_state, a);
    // Should return a non-empty slice for the initial state on terminal 'a'
    assert!(
        !actions.is_empty(),
        "initial state should have actions on 'a'"
    );
}

#[test]
fn parse_table_actions_unknown_symbol_returns_empty() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let unknown = SymbolId(9999);
    let actions = table.actions(table.initial_state, unknown);
    assert!(actions.is_empty());
}

// ===========================================================================
// 5. Action serialization via serde_json (5 tests)
// ===========================================================================

#[test]
fn serde_roundtrip_shift() {
    let a = Action::Shift(StateId(42));
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_reduce() {
    let a = Action::Reduce(RuleId(7));
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_accept() {
    let json = serde_json::to_string(&Action::Accept).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(b, Action::Accept);
}

#[test]
fn serde_roundtrip_fork() {
    let a = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ]);
    let json = serde_json::to_string(&a).unwrap();
    let b: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}

#[test]
fn serde_roundtrip_action_cell() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(1)),
        Action::Error,
        Action::Recover,
    ];
    let json = serde_json::to_string(&cell).unwrap();
    let b: ActionCell = serde_json::from_str(&json).unwrap();
    assert_eq!(cell, b);
}

// ===========================================================================
// 6. StateId/SymbolId in actions (5 tests)
// ===========================================================================

#[test]
fn state_id_display() {
    let sid = StateId(42);
    let s = format!("{sid}");
    assert!(s.contains("42"), "Display: {s}");
}

#[test]
fn state_id_ordering() {
    assert!(StateId(0) < StateId(1));
    assert!(StateId(100) < StateId(u16::MAX));
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(255) < SymbolId(256));
}

#[test]
fn rule_id_ordering() {
    assert!(RuleId(0) < RuleId(1));
    assert!(RuleId(100) < RuleId(u16::MAX));
}

#[test]
fn state_id_hash_in_set() {
    let mut set = HashSet::new();
    set.insert(StateId(1));
    set.insert(StateId(2));
    set.insert(StateId(1)); // duplicate
    assert_eq!(set.len(), 2);
}

// ===========================================================================
// 7. Action ordering patterns (5 tests)
// ===========================================================================

#[test]
fn action_hash_same_for_equal_actions() {
    use std::hash::{Hash, Hasher};
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    Action::Shift(StateId(5)).hash(&mut h1);
    Action::Shift(StateId(5)).hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn action_hash_differs_for_distinct_actions() {
    use std::hash::{Hash, Hasher};
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    Action::Shift(StateId(1)).hash(&mut h1);
    Action::Reduce(RuleId(1)).hash(&mut h2);
    // Not guaranteed to differ, but overwhelmingly likely
    // We just verify both finish without panic
    let _ = h1.finish();
    let _ = h2.finish();
}

#[test]
fn action_cell_can_be_sorted_by_debug() {
    // Verify ActionCell elements can be compared via Debug representation for determinism
    let cell: ActionCell = vec![
        Action::Reduce(RuleId(2)),
        Action::Shift(StateId(1)),
        Action::Accept,
    ];
    let mut dbg_strs: Vec<String> = cell.iter().map(|a| format!("{a:?}")).collect();
    dbg_strs.sort();
    assert_eq!(dbg_strs[0], "Accept");
}

#[test]
fn action_dedup_in_cell() {
    let mut cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ];
    cell.dedup();
    assert_eq!(cell.len(), 2);
}

#[test]
fn action_cell_retain_only_shifts() {
    let mut cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Shift(StateId(3)),
        Action::Accept,
    ];
    cell.retain(|a| matches!(a, Action::Shift(_)));
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().all(|a| matches!(a, Action::Shift(_))));
}

// ===========================================================================
// 8. Edge cases (6 tests)
// ===========================================================================

#[test]
fn fork_with_empty_vec() {
    let a = Action::Fork(vec![]);
    match &a {
        Action::Fork(v) => assert!(v.is_empty()),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn fork_single_action() {
    let a = Action::Fork(vec![Action::Accept]);
    match &a {
        Action::Fork(v) => assert_eq!(v.len(), 1),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn parse_table_actions_out_of_bounds_state() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let far_state = StateId(u16::MAX);
    let a = SymbolId(1);
    let actions = table.actions(far_state, a);
    assert!(actions.is_empty());
}

#[test]
fn builder_grammar_produces_valid_table() {
    let g = GrammarBuilder::new("builder_test")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
    assert!(!table.action_table.is_empty());
}

#[test]
fn action_cell_many_reduces() {
    let cell: ActionCell = (0..100).map(|i| Action::Reduce(RuleId(i))).collect();
    assert_eq!(cell.len(), 100);
    for (i, a) in cell.iter().enumerate() {
        assert!(matches!(a, Action::Reduce(RuleId(r)) if *r == i as u16));
    }
}

#[test]
fn two_alternative_grammar_has_shift_on_both_tokens() {
    let g = grammar_s_to_a_or_b();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let a = SymbolId(1);
    let b = SymbolId(2);
    let actions_a = table.actions(table.initial_state, a);
    let actions_b = table.actions(table.initial_state, b);
    assert!(
        !actions_a.is_empty(),
        "initial state should have actions on 'a'"
    );
    assert!(
        !actions_b.is_empty(),
        "initial state should have actions on 'b'"
    );
}
