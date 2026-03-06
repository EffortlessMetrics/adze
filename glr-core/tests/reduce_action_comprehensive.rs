#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for reduce actions and production handling in adze-glr-core.

use adze_glr_core::{
    Action, ActionCell, Conflict, ConflictResolver, ConflictType, GotoIndexing, ParseRule,
    ParseTable,
};
use adze_ir::{RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helper: build a minimal ParseTable with the given action_table and rules
// ---------------------------------------------------------------------------
fn minimal_table(
    action_table: Vec<Vec<ActionCell>>,
    rules: Vec<ParseRule>,
    symbol_to_index: BTreeMap<SymbolId, usize>,
) -> ParseTable {
    let state_count = action_table.len();
    let cols = action_table.first().map_or(0, |r| r.len());
    ParseTable {
        action_table,
        goto_table: vec![vec![StateId(u16::MAX); 1]; state_count],
        symbol_metadata: vec![],
        state_count,
        symbol_count: cols,
        symbol_to_index,
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules,
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(100),
        grammar: Default::default(),
        initial_state: StateId(0),
        token_count: cols,
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

// ── 1. Reduce action construction and field access ─────────────────────────

#[test]
fn reduce_construction_basic() {
    let a = Action::Reduce(RuleId(0));
    assert_eq!(a, Action::Reduce(RuleId(0)));
}

#[test]
fn reduce_construction_various_rule_ids() {
    for id in [0, 1, 5, 42, 255, 1000, u16::MAX] {
        let a = Action::Reduce(RuleId(id));
        if let Action::Reduce(rid) = a {
            assert_eq!(rid.0, id);
        } else {
            panic!("expected Reduce");
        }
    }
}

#[test]
fn reduce_extracts_rule_id() {
    let a = Action::Reduce(RuleId(7));
    match a {
        Action::Reduce(rid) => assert_eq!(rid, RuleId(7)),
        _ => panic!("expected Reduce"),
    }
}

#[test]
fn reduce_rule_id_zero() {
    let a = Action::Reduce(RuleId(0));
    match a {
        Action::Reduce(rid) => assert_eq!(rid.0, 0),
        _ => panic!("expected Reduce"),
    }
}

#[test]
fn reduce_rule_id_max() {
    let a = Action::Reduce(RuleId(u16::MAX));
    match a {
        Action::Reduce(rid) => assert_eq!(rid.0, u16::MAX),
        _ => panic!("expected Reduce"),
    }
}

// ── 2. Multiple reduces in same cell (reduce/reduce conflict) ──────────────

#[test]
fn two_reduces_in_single_cell() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().all(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn three_reduces_in_single_cell() {
    let cell: ActionCell = vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ];
    assert_eq!(cell.len(), 3);
    let rule_ids: Vec<u16> = cell
        .iter()
        .map(|a| match a {
            Action::Reduce(rid) => rid.0,
            _ => panic!("expected Reduce"),
        })
        .collect();
    assert_eq!(rule_ids, vec![0, 1, 2]);
}

#[test]
fn duplicate_reduces_preserved_in_cell() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(5)), Action::Reduce(RuleId(5))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn reduce_reduce_conflict_type() {
    let conflict = Conflict {
        state: StateId(0),
        symbol: SymbolId(1),
        actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ReduceReduce,
    };
    assert_eq!(conflict.conflict_type, ConflictType::ReduceReduce);
    assert_eq!(conflict.actions.len(), 2);
}

// ── 3. Reduce + Shift in same cell (shift/reduce conflict) ─────────────────

#[test]
fn shift_reduce_in_single_cell() {
    let cell: ActionCell = vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn shift_reduce_conflict_type() {
    let conflict = Conflict {
        state: StateId(2),
        symbol: SymbolId(3),
        actions: vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ShiftReduce,
    };
    assert_eq!(conflict.conflict_type, ConflictType::ShiftReduce);
}

#[test]
fn shift_and_multiple_reduces() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(2)),
    ];
    let shift_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .count();
    let reduce_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert_eq!(shift_count, 1);
    assert_eq!(reduce_count, 2);
}

#[test]
fn shift_reduce_conflict_order_independent() {
    let cell_sr: ActionCell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let cell_rs: ActionCell = vec![Action::Reduce(RuleId(0)), Action::Shift(StateId(1))];
    // Both should contain the same set of actions
    assert!(cell_sr.contains(&Action::Shift(StateId(1))));
    assert!(cell_sr.contains(&Action::Reduce(RuleId(0))));
    assert!(cell_rs.contains(&Action::Shift(StateId(1))));
    assert!(cell_rs.contains(&Action::Reduce(RuleId(0))));
}

// ── 4. Production ID tracking via ParseRule ────────────────────────────────

#[test]
fn parse_rule_lhs_and_rhs_len() {
    let rule = ParseRule {
        lhs: SymbolId(10),
        rhs_len: 3,
    };
    assert_eq!(rule.lhs, SymbolId(10));
    assert_eq!(rule.rhs_len, 3);
}

#[test]
fn parse_rule_multiple_productions() {
    let rules = [
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 1,
        }, // rule 0: S -> a
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 3,
        }, // rule 1: S -> a b c
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 2,
        }, // rule 2: E -> a b
    ];
    assert_eq!(rules.len(), 3);
    assert_eq!(rules[0].rhs_len, 1);
    assert_eq!(rules[1].rhs_len, 3);
    assert_eq!(rules[2].lhs, SymbolId(11));
}

#[test]
fn table_rule_lookup_by_rule_id() {
    let rules = vec![
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 2,
        },
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 4,
        },
    ];
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(0), 0);
    let table = minimal_table(vec![vec![vec![]]], rules, sym_idx);

    let (lhs, rhs_len) = table.rule(RuleId(0));
    assert_eq!(lhs, SymbolId(10));
    assert_eq!(rhs_len, 2);

    let (lhs, rhs_len) = table.rule(RuleId(1));
    assert_eq!(lhs, SymbolId(11));
    assert_eq!(rhs_len, 4);
}

#[test]
fn reduce_action_references_correct_rule() {
    let rules = [
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 1,
        },
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 3,
        },
    ];
    let action = Action::Reduce(RuleId(1));
    if let Action::Reduce(rid) = action {
        assert_eq!(rules[rid.0 as usize].lhs, SymbolId(11));
        assert_eq!(rules[rid.0 as usize].rhs_len, 3);
    }
}

// ── 5. Symbol count (pop count) handling ───────────────────────────────────

#[test]
fn symbol_count_zero_for_epsilon() {
    let rule = ParseRule {
        lhs: SymbolId(10),
        rhs_len: 0,
    };
    assert_eq!(rule.rhs_len, 0);
}

#[test]
fn symbol_count_one() {
    let rule = ParseRule {
        lhs: SymbolId(10),
        rhs_len: 1,
    };
    assert_eq!(rule.rhs_len, 1);
}

#[test]
fn symbol_count_large() {
    let rule = ParseRule {
        lhs: SymbolId(10),
        rhs_len: 100,
    };
    assert_eq!(rule.rhs_len, 100);
}

#[test]
fn symbol_count_max() {
    let rule = ParseRule {
        lhs: SymbolId(10),
        rhs_len: u16::MAX,
    };
    assert_eq!(rule.rhs_len, u16::MAX);
}

#[test]
fn various_rhs_lengths_in_table() {
    let rules: Vec<ParseRule> = (0..10)
        .map(|i| ParseRule {
            lhs: SymbolId(10),
            rhs_len: i,
        })
        .collect();
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(0), 0);
    let table = minimal_table(vec![vec![vec![]]], rules, sym_idx);
    for i in 0..10 {
        let (_lhs, rhs_len) = table.rule(RuleId(i));
        assert_eq!(rhs_len, i);
    }
}

// ── 6. Reduce with 0 symbols (epsilon productions) ────────────────────────

#[test]
fn epsilon_production_in_table() {
    let rules = vec![ParseRule {
        lhs: SymbolId(10),
        rhs_len: 0,
    }];
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let table = minimal_table(vec![vec![vec![Action::Reduce(RuleId(0))]]], rules, sym_idx);
    let actions = table.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], Action::Reduce(RuleId(0)));
    let (_lhs, rhs_len) = table.rule(RuleId(0));
    assert_eq!(rhs_len, 0, "epsilon production should pop 0 symbols");
}

#[test]
fn epsilon_and_nonempty_productions_coexist() {
    let rules = vec![
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 0,
        }, // epsilon
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 2,
        }, // non-epsilon
    ];
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let cell: ActionCell = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    let table = minimal_table(vec![vec![cell]], rules, sym_idx);

    let actions = table.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 2);
    let (_, len0) = table.rule(RuleId(0));
    let (_, len1) = table.rule(RuleId(1));
    assert_eq!(len0, 0);
    assert_eq!(len1, 2);
}

#[test]
fn multiple_epsilon_reduces_in_cell() {
    let rules = [
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 0,
        },
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 0,
        },
    ];
    let cell: ActionCell = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
    for a in &cell {
        if let Action::Reduce(rid) = a {
            assert_eq!(rules[rid.0 as usize].rhs_len, 0);
        }
    }
}

// ── 7. Action equality and Debug ───────────────────────────────────────────

#[test]
fn reduce_equality_same_rule() {
    assert_eq!(Action::Reduce(RuleId(5)), Action::Reduce(RuleId(5)));
}

#[test]
fn reduce_inequality_different_rule() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
}

#[test]
fn reduce_not_equal_to_shift() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Shift(StateId(0)));
}

#[test]
fn reduce_not_equal_to_accept() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Accept);
}

#[test]
fn reduce_not_equal_to_error() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Error);
}

#[test]
fn reduce_debug_format() {
    let a = Action::Reduce(RuleId(42));
    let dbg = format!("{a:?}");
    assert!(
        dbg.contains("Reduce"),
        "Debug should contain 'Reduce': {dbg}"
    );
    assert!(dbg.contains("42"), "Debug should contain rule id 42: {dbg}");
}

#[test]
fn reduce_clone_equals_original() {
    let a = Action::Reduce(RuleId(9));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn reduce_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Reduce(RuleId(3)));
    set.insert(Action::Reduce(RuleId(3)));
    assert_eq!(set.len(), 1, "identical reduces should hash equally");

    set.insert(Action::Reduce(RuleId(4)));
    assert_eq!(set.len(), 2);
}

// ── 8. Table lookup for reduce actions ─────────────────────────────────────

#[test]
fn table_lookup_reduce_in_state_0() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let table = minimal_table(
        vec![vec![vec![Action::Reduce(RuleId(0))]]],
        vec![ParseRule {
            lhs: SymbolId(10),
            rhs_len: 1,
        }],
        sym_idx,
    );
    let actions = table.actions(StateId(0), SymbolId(1));
    assert_eq!(actions, &[Action::Reduce(RuleId(0))]);
}

#[test]
fn table_lookup_empty_cell_returns_empty() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let table = minimal_table(vec![vec![vec![]]], vec![], sym_idx);
    let actions = table.actions(StateId(0), SymbolId(1));
    assert!(actions.is_empty());
}

#[test]
fn table_lookup_unknown_symbol_returns_empty() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let table = minimal_table(
        vec![vec![vec![Action::Reduce(RuleId(0))]]],
        vec![ParseRule {
            lhs: SymbolId(10),
            rhs_len: 1,
        }],
        sym_idx,
    );
    // SymbolId(99) not in symbol_to_index
    let actions = table.actions(StateId(0), SymbolId(99));
    assert!(actions.is_empty());
}

#[test]
fn table_lookup_out_of_bounds_state_returns_empty() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let table = minimal_table(
        vec![vec![vec![Action::Reduce(RuleId(0))]]],
        vec![ParseRule {
            lhs: SymbolId(10),
            rhs_len: 1,
        }],
        sym_idx,
    );
    let actions = table.actions(StateId(999), SymbolId(1));
    assert!(actions.is_empty());
}

#[test]
fn table_with_reduce_across_multiple_states() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    sym_idx.insert(SymbolId(2), 1);
    let table = minimal_table(
        vec![
            // state 0: reduce by rule 0 on sym 1, shift on sym 2
            vec![
                vec![Action::Reduce(RuleId(0))],
                vec![Action::Shift(StateId(1))],
            ],
            // state 1: reduce by rule 1 on sym 2
            vec![vec![], vec![Action::Reduce(RuleId(1))]],
        ],
        vec![
            ParseRule {
                lhs: SymbolId(10),
                rhs_len: 1,
            },
            ParseRule {
                lhs: SymbolId(11),
                rhs_len: 2,
            },
        ],
        sym_idx,
    );

    let a0 = table.actions(StateId(0), SymbolId(1));
    assert_eq!(a0, &[Action::Reduce(RuleId(0))]);

    let a1 = table.actions(StateId(1), SymbolId(2));
    assert_eq!(a1, &[Action::Reduce(RuleId(1))]);

    // state 1, sym 1 should be empty
    let a_empty = table.actions(StateId(1), SymbolId(1));
    assert!(a_empty.is_empty());
}

#[test]
fn table_glr_cell_with_shift_and_reduce() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let cell: ActionCell = vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(0))];
    let table = minimal_table(
        vec![vec![cell]],
        vec![ParseRule {
            lhs: SymbolId(10),
            rhs_len: 1,
        }],
        sym_idx,
    );
    let actions = table.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0], Action::Shift(StateId(2)));
    assert_eq!(actions[1], Action::Reduce(RuleId(0)));
}

#[test]
fn table_glr_cell_with_multiple_reduces() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let cell: ActionCell = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    let table = minimal_table(
        vec![vec![cell]],
        vec![
            ParseRule {
                lhs: SymbolId(10),
                rhs_len: 1,
            },
            ParseRule {
                lhs: SymbolId(11),
                rhs_len: 3,
            },
        ],
        sym_idx,
    );
    let actions = table.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 2);

    // Both rule lookups should succeed
    let (lhs0, len0) = table.rule(RuleId(0));
    assert_eq!(lhs0, SymbolId(10));
    assert_eq!(len0, 1);
    let (lhs1, len1) = table.rule(RuleId(1));
    assert_eq!(lhs1, SymbolId(11));
    assert_eq!(len1, 3);
}

#[test]
fn fork_wrapping_reduces() {
    let fork = Action::Fork(vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))]);
    if let Action::Fork(inner) = &fork {
        assert_eq!(inner.len(), 2);
        assert!(inner.iter().all(|a| matches!(a, Action::Reduce(_))));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn fork_wrapping_shift_and_reduce() {
    let fork = Action::Fork(vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))]);
    if let Action::Fork(inner) = &fork {
        assert_eq!(inner.len(), 2);
        assert!(matches!(inner[0], Action::Shift(StateId(5))));
        assert!(matches!(inner[1], Action::Reduce(RuleId(3))));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn conflict_type_equality() {
    assert_eq!(ConflictType::ReduceReduce, ConflictType::ReduceReduce);
    assert_eq!(ConflictType::ShiftReduce, ConflictType::ShiftReduce);
    assert_ne!(ConflictType::ShiftReduce, ConflictType::ReduceReduce);
}

#[test]
fn conflict_resolver_empty() {
    let resolver = ConflictResolver { conflicts: vec![] };
    assert!(resolver.conflicts.is_empty());
}
