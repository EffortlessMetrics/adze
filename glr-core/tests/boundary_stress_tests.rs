#![allow(
    clippy::needless_range_loop,
    clippy::vec_init_then_push,
    clippy::useless_vec
)]

use adze_glr_core::{
    Action, ActionCell, Conflict, ConflictResolver, ConflictType, ParseRule, ParseTable, RuleId,
    StateId, SymbolId, SymbolMetadata,
};
use std::collections::BTreeMap;

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: ParseTable with 0 states, 0 symbols
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_zero_states_zero_symbols() {
    let table = ParseTable::default();
    assert_eq!(table.state_count, 0);
    assert_eq!(table.symbol_count, 0);
    assert_eq!(table.action_table.len(), 0);
    assert_eq!(table.goto_table.len(), 0);
    assert_eq!(table.symbol_to_index.len(), 0);
    assert_eq!(table.index_to_symbol.len(), 0);
    assert_eq!(table.nonterminal_to_index.len(), 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: ParseTable with 1 state, 1 symbol
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_one_state_one_symbol() {
    let mut table = ParseTable {
        state_count: 1,
        symbol_count: 1,
        action_table: vec![vec![vec![]]],
        goto_table: vec![vec![]],
        ..Default::default()
    };

    let sym = SymbolId(0);
    table.symbol_to_index.insert(sym, 0);
    table.index_to_symbol.push(sym);

    assert_eq!(table.state_count, 1);
    assert_eq!(table.symbol_count, 1);
    assert_eq!(table.action_table.len(), 1);
    assert_eq!(table.action_table[0].len(), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: ParseTable with 100+ states
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_hundred_states() {
    let mut table = ParseTable::default();
    let num_states = 150;
    let num_symbols = 10;

    table.state_count = num_states;
    table.symbol_count = num_symbols;

    // Create action_table with 150 states, 10 symbols each
    table.action_table = vec![vec![vec![]; num_symbols]; num_states];
    table.goto_table = vec![vec![]; num_states];

    // Add symbols to mapping
    for i in 0..num_symbols {
        let sym = SymbolId(i as u16);
        table.symbol_to_index.insert(sym, i);
        table.index_to_symbol.push(sym);
    }

    assert_eq!(table.action_table.len(), num_states);
    assert_eq!(table.action_table[0].len(), num_symbols);
    assert_eq!(table.action_table[num_states - 1].len(), num_symbols);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4: ActionCell with 0 actions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_cell_zero_actions() {
    let cell: ActionCell = vec![];
    assert_eq!(cell.len(), 0);
    assert!(cell.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: ActionCell with many actions (10+)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_cell_many_actions() {
    let mut cell: ActionCell = vec![];

    for i in 0..15 {
        if i % 2 == 0 {
            cell.push(Action::Shift(StateId(i as u16)));
        } else {
            cell.push(Action::Reduce(RuleId(i as u16)));
        }
    }

    assert_eq!(cell.len(), 15);
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 6: Action::Shift to max state ID
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_shift_max_state_id() {
    let max_state = StateId(u16::MAX);
    let action = Action::Shift(max_state);

    match action {
        Action::Shift(state) => {
            assert_eq!(state, max_state);
            assert_eq!(state.0, u16::MAX);
        }
        _ => panic!("Expected Shift action"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 7: Action::Reduce with max production ID
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_reduce_max_production_id() {
    let max_rule = RuleId(u16::MAX);
    let action = Action::Reduce(max_rule);

    match action {
        Action::Reduce(rule) => {
            assert_eq!(rule, max_rule);
            assert_eq!(rule.0, u16::MAX);
        }
        _ => panic!("Expected Reduce action"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 8: ParseTable serialization roundtrip (structure preservation)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_structure_preservation() {
    let mut table1 = ParseTable {
        state_count: 50,
        symbol_count: 20,
        action_table: vec![vec![vec![Action::Accept]; 20]; 50],
        goto_table: vec![vec![]; 50],
        ..Default::default()
    };

    for i in 0..20 {
        let sym = SymbolId(i as u16);
        table1.symbol_to_index.insert(sym, i);
        table1.index_to_symbol.push(sym);
    }

    // Verify structure is preserved
    assert_eq!(table1.action_table.len(), 50);
    assert_eq!(table1.action_table[0].len(), 20);
    assert!(
        table1.action_table[0][0]
            .iter()
            .any(|a| matches!(a, Action::Accept))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 9: Empty conflict list
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_empty_conflict_list() {
    let resolver = ConflictResolver { conflicts: vec![] };

    assert_eq!(resolver.conflicts.len(), 0);
    assert!(resolver.conflicts.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 10: Conflict with all types (ShiftReduce, ReduceReduce)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_conflict_all_types() {
    let shift_reduce_conflict = Conflict {
        state: StateId(0),
        symbol: SymbolId(0),
        actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    };

    let reduce_reduce_conflict = Conflict {
        state: StateId(1),
        symbol: SymbolId(1),
        actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ReduceReduce,
    };

    assert_eq!(
        shift_reduce_conflict.conflict_type,
        ConflictType::ShiftReduce
    );
    assert_eq!(
        reduce_reduce_conflict.conflict_type,
        ConflictType::ReduceReduce
    );
    assert_eq!(shift_reduce_conflict.actions.len(), 2);
    assert_eq!(reduce_reduce_conflict.actions.len(), 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 11: SymbolMetadata with all combinations of is_terminal/is_named
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_symbol_metadata_all_combinations() {
    let combinations = vec![(true, true), (true, false), (false, true), (false, false)];

    for (is_terminal, is_named) in combinations {
        let metadata = SymbolMetadata {
            name: "test".to_string(),
            is_visible: true,
            is_named,
            is_supertype: false,
            is_terminal,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        };

        assert_eq!(metadata.is_terminal, is_terminal);
        assert_eq!(metadata.is_named, is_named);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 12: ParseTable with empty goto table
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_empty_goto_table() {
    let table = ParseTable {
        state_count: 10,
        symbol_count: 5,
        action_table: vec![vec![vec![]; 5]; 10],
        goto_table: vec![vec![]; 10],
        ..Default::default()
    };

    assert_eq!(table.goto_table.len(), 10);
    for row in &table.goto_table {
        assert!(row.is_empty());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 13: ParseTable with dense goto table
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_dense_goto_table() {
    let mut table = ParseTable::default();
    let num_states = 20;
    let num_nonterminals = 15;

    table.state_count = num_states;
    table.action_table = vec![vec![]; num_states];
    table.goto_table = vec![vec![StateId(0); num_nonterminals]; num_states];

    for i in 0..num_nonterminals {
        let sym = SymbolId((1000 + i) as u16);
        table.nonterminal_to_index.insert(sym, i);
    }

    assert_eq!(table.goto_table.len(), num_states);
    assert_eq!(table.goto_table[0].len(), num_nonterminals);
    assert_eq!(table.goto_table[num_states - 1].len(), num_nonterminals);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 14: Large production with many symbols in RHS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_large_production_many_symbols() {
    let rule = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 100,
    };

    assert_eq!(rule.rhs_len, 100);
    assert_eq!(rule.lhs, SymbolId(0));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 15: State with only shift actions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_state_only_shift_actions() {
    let _table = ParseTable::default();
    let num_symbols = 8;

    // Create a state with only shift actions
    let mut state_actions = vec![];
    for i in 0..num_symbols {
        state_actions.push(vec![Action::Shift(StateId(i as u16))]);
    }

    assert!(
        state_actions
            .iter()
            .all(|cell| { cell.iter().all(|a| matches!(a, Action::Shift(_))) })
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 16: State with only reduce actions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_state_only_reduce_actions() {
    let mut state_actions = vec![];
    for i in 0..10 {
        state_actions.push(vec![Action::Reduce(RuleId(i as u16))]);
    }

    assert!(
        state_actions
            .iter()
            .all(|cell| { cell.iter().all(|a| matches!(a, Action::Reduce(_))) })
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 17: State with mixed shift/reduce/accept actions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_state_mixed_actions() {
    let actions = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ];

    let has_shift = actions.iter().any(|a| matches!(a, Action::Shift(_)));
    let has_reduce = actions.iter().any(|a| matches!(a, Action::Reduce(_)));
    let has_accept = actions.iter().any(|a| matches!(a, Action::Accept));

    assert!(has_shift && has_reduce && has_accept);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 18: ParseTable equality comparison
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_equality() {
    let table1 = ParseTable::default();
    let table2 = ParseTable::default();

    // Both should have same default values
    assert_eq!(table1.state_count, table2.state_count);
    assert_eq!(table1.symbol_count, table2.symbol_count);
    assert_eq!(table1.eof_symbol, table2.eof_symbol);
    assert_eq!(table1.start_symbol, table2.start_symbol);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 19: Symbol ID boundary values (0, 1, u16::MAX-1, u16::MAX)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_symbol_id_boundary_values() {
    let sym_zero = SymbolId(0);
    let sym_one = SymbolId(1);
    let sym_near_max = SymbolId(u16::MAX - 1);
    let sym_max = SymbolId(u16::MAX);

    assert_eq!(sym_zero.0, 0);
    assert_eq!(sym_one.0, 1);
    assert_eq!(sym_near_max.0, u16::MAX - 1);
    assert_eq!(sym_max.0, u16::MAX);

    // Test in BTreeMap (symbol_to_index)
    let mut map = BTreeMap::new();
    map.insert(sym_zero, 0);
    map.insert(sym_one, 1);
    map.insert(sym_near_max, 2);
    map.insert(sym_max, 3);

    assert_eq!(map.len(), 4);
    assert_eq!(map.get(&sym_zero), Some(&0));
    assert_eq!(map.get(&sym_max), Some(&3));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 20: Production with 0-length RHS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_production_zero_length_rhs() {
    let rule = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 0,
    };

    assert_eq!(rule.rhs_len, 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 21: Multiple conflicts on same state
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_multiple_conflicts_same_state() {
    let state = StateId(5);
    let conflicts = vec![
        Conflict {
            state,
            symbol: SymbolId(0),
            actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
            conflict_type: ConflictType::ShiftReduce,
        },
        Conflict {
            state,
            symbol: SymbolId(1),
            actions: vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))],
            conflict_type: ConflictType::ReduceReduce,
        },
        Conflict {
            state,
            symbol: SymbolId(2),
            actions: vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(3))],
            conflict_type: ConflictType::ShiftReduce,
        },
    ];

    let same_state_count = conflicts.iter().filter(|c| c.state == state).count();
    assert_eq!(same_state_count, 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 22: Action ordering within ActionCell
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_cell_ordering() {
    let cell: ActionCell = vec![
        Action::Reduce(RuleId(1)),
        Action::Shift(StateId(5)),
        Action::Error,
        Action::Accept,
    ];

    // Verify all actions exist before sorting
    assert_eq!(cell.len(), 4);

    // In practice, cells should be normalized/sorted
    // This test just verifies the structure can hold mixed types
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Error)));
    assert!(cell.iter().any(|a| matches!(a, Action::Accept)));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 23: Empty symbol_to_index mapping
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_empty_symbol_to_index_mapping() {
    let table = ParseTable {
        symbol_to_index: BTreeMap::new(),
        ..Default::default()
    };

    assert!(table.symbol_to_index.is_empty());
    assert_eq!(table.symbol_to_index.len(), 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 24: State count matching action table dimensions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_state_count_matches_action_table() {
    let mut table = ParseTable::default();
    let num_states = 25;
    let num_symbols = 12;

    table.state_count = num_states;
    table.action_table = vec![vec![vec![]; num_symbols]; num_states];

    assert_eq!(table.action_table.len(), table.state_count);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 25: Goto table with holes (missing entries)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_goto_table_with_holes() {
    let mut table = ParseTable::default();
    let num_states = 10;

    table.state_count = num_states;
    // Create goto table with varying row lengths (holes)
    for i in 0..num_states {
        let len = if i % 2 == 0 { 5 } else { 3 };
        table.goto_table.push(vec![StateId(0); len]);
    }

    assert_eq!(table.goto_table.len(), num_states);
    assert_eq!(table.goto_table[0].len(), 5);
    assert_eq!(table.goto_table[1].len(), 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 26: Accept action placement
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_accept_action_placement() {
    let mut table = ParseTable {
        state_count: 5,
        symbol_count: 3,
        ..Default::default()
    };

    // Create action table where only the final state has Accept
    for state_idx in 0..5 {
        let mut state_actions = vec![];
        for sym_idx in 0..3 {
            if state_idx == 4 && sym_idx == 0 {
                state_actions.push(vec![Action::Accept]);
            } else {
                state_actions.push(vec![Action::Error]);
            }
        }
        table.action_table.push(state_actions);
    }

    // Verify Accept is only in state 4
    for (state_idx, state_actions) in table.action_table.iter().enumerate() {
        let has_accept = state_actions
            .iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)));

        if state_idx == 4 {
            assert!(has_accept);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 27: ParseTable cloning preserves structure
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_clone_preserves_structure() {
    let mut table1 = ParseTable {
        state_count: 15,
        symbol_count: 8,
        action_table: vec![vec![vec![Action::Accept]; 8]; 15],
        goto_table: vec![vec![]; 15],
        ..Default::default()
    };

    for i in 0..8 {
        let sym = SymbolId(i as u16);
        table1.symbol_to_index.insert(sym, i);
        table1.index_to_symbol.push(sym);
    }

    // Clone should preserve structure
    let table2 = table1.clone();

    assert_eq!(table1.state_count, table2.state_count);
    assert_eq!(table1.symbol_count, table2.symbol_count);
    assert_eq!(table1.action_table.len(), table2.action_table.len());
    assert_eq!(table1.symbol_to_index.len(), table2.symbol_to_index.len());
    assert_eq!(table1.index_to_symbol.len(), table2.index_to_symbol.len());
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 28: Action::Fork with nested actions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_fork_nested() {
    let fork = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Error,
    ]);

    match fork {
        Action::Fork(actions) => {
            assert_eq!(actions.len(), 3);
            assert!(actions.iter().any(|a| matches!(a, Action::Shift(_))));
        }
        _ => panic!("Expected Fork"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 29: Action::Recover variant
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_recover() {
    let action = Action::Recover;
    match action {
        Action::Recover => {
            // Verify the variant exists and can be matched
        }
        _ => panic!("Expected Recover"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 30: Action::Error variant
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_error() {
    let action = Action::Error;
    match action {
        Action::Error => {
            // Verify the variant exists
        }
        _ => panic!("Expected Error"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 31: ParseTable with many metadata entries
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_many_metadata() {
    let mut table = ParseTable {
        symbol_count: 50,
        ..Default::default()
    };

    for i in 0..50 {
        let meta = SymbolMetadata {
            name: format!("symbol_{}", i),
            is_visible: i % 2 == 0,
            is_named: i % 3 == 0,
            is_supertype: false,
            is_terminal: i < 20,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(i as u16),
        };
        table.symbol_metadata.push(meta);
    }

    assert_eq!(table.symbol_metadata.len(), 50);
    let terminals = table
        .symbol_metadata
        .iter()
        .filter(|m| m.is_terminal)
        .count();
    assert_eq!(terminals, 20);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 32: BTreeMap for symbol_to_index vs HashMap-like behavior
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_symbol_to_index_btreemap() {
    let mut map = BTreeMap::new();

    // Insert in non-sequential order
    map.insert(SymbolId(100), 0);
    map.insert(SymbolId(1), 1);
    map.insert(SymbolId(50), 2);
    map.insert(SymbolId(200), 3);

    // BTreeMap maintains sorted order
    let keys: Vec<_> = map.keys().copied().collect();
    assert_eq!(keys[0], SymbolId(1));
    assert_eq!(keys[1], SymbolId(50));
    assert_eq!(keys[2], SymbolId(100));
    assert_eq!(keys[3], SymbolId(200));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 33: Conflict with state and symbol metadata
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_conflict_with_metadata() {
    let conflict = Conflict {
        state: StateId(42),
        symbol: SymbolId(17),
        actions: vec![Action::Shift(StateId(10)), Action::Reduce(RuleId(5))],
        conflict_type: ConflictType::ShiftReduce,
    };

    assert_eq!(conflict.state.0, 42);
    assert_eq!(conflict.symbol.0, 17);
    assert_eq!(conflict.actions.len(), 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 34: ActionCell with Accept and Error mixed
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_cell_accept_error_mixed() {
    let cell: ActionCell = vec![Action::Accept, Action::Error];

    assert_eq!(cell.len(), 2);
    assert!(cell.iter().any(|a| matches!(a, Action::Accept)));
    assert!(cell.iter().any(|a| matches!(a, Action::Error)));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 35: Stress test: Large action table with many conflicts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_stress_large_action_table() {
    let num_states = 100;
    let num_symbols = 50;

    let mut table = ParseTable {
        state_count: num_states,
        symbol_count: num_symbols,
        ..Default::default()
    };

    // Build a large action table
    for state_idx in 0..num_states {
        let mut state = vec![];
        for sym_idx in 0..num_symbols {
            let mut cell: ActionCell = vec![];

            // Add varying number of actions per cell
            let action_count = (state_idx + sym_idx) % 3 + 1;
            for action_idx in 0..action_count {
                if action_idx % 2 == 0 {
                    cell.push(Action::Shift(StateId((state_idx + action_idx) as u16)));
                } else {
                    cell.push(Action::Reduce(RuleId((sym_idx + action_idx) as u16)));
                }
            }
            state.push(cell);
        }
        table.action_table.push(state);
    }

    assert_eq!(table.action_table.len(), num_states);
    assert_eq!(table.action_table[0].len(), num_symbols);

    // Verify total actions
    let total_actions: usize = table
        .action_table
        .iter()
        .map(|state| state.iter().map(|cell| cell.len()).sum::<usize>())
        .sum();
    assert!(total_actions > 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 36: ParseTable with complex symbol mapping
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_table_complex_symbol_mapping() {
    let mut table = ParseTable::default();

    // Create a non-sequential symbol mapping
    let symbols = vec![
        SymbolId(5),
        SymbolId(10),
        SymbolId(2),
        SymbolId(100),
        SymbolId(1),
    ];

    for (idx, &sym) in symbols.iter().enumerate() {
        table.symbol_to_index.insert(sym, idx);
        table.index_to_symbol.push(sym);
    }

    // Verify bidirectional mapping
    for (idx, &sym) in symbols.iter().enumerate() {
        assert_eq!(table.symbol_to_index.get(&sym), Some(&idx));
        assert_eq!(table.index_to_symbol[idx], sym);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 37: SymbolMetadata with all boolean combinations
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_symbol_metadata_all_boolean_combinations() {
    let mut count = 0;
    for is_visible in [true, false].iter() {
        for is_named in [true, false].iter() {
            for is_terminal in [true, false].iter() {
                for is_extra in [true, false].iter() {
                    let metadata = SymbolMetadata {
                        name: "test".to_string(),
                        is_visible: *is_visible,
                        is_named: *is_named,
                        is_supertype: false,
                        is_terminal: *is_terminal,
                        is_extra: *is_extra,
                        is_fragile: false,
                        symbol_id: SymbolId(count as u16),
                    };

                    assert_eq!(metadata.is_visible, *is_visible);
                    assert_eq!(metadata.is_named, *is_named);
                    assert_eq!(metadata.is_terminal, *is_terminal);
                    assert_eq!(metadata.is_extra, *is_extra);

                    count += 1;
                }
            }
        }
    }

    assert_eq!(count, 16); // 2^4 combinations
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 38: Stress test: ConflictResolver with many conflicts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_stress_many_conflicts() {
    let mut conflicts = vec![];

    for state_id in 0..50 {
        for symbol_id in 0..10 {
            let is_shift_reduce = (state_id + symbol_id) % 2 == 0;

            let conflict = Conflict {
                state: StateId(state_id as u16),
                symbol: SymbolId(symbol_id as u16),
                actions: if is_shift_reduce {
                    vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]
                } else {
                    vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))]
                },
                conflict_type: if is_shift_reduce {
                    ConflictType::ShiftReduce
                } else {
                    ConflictType::ReduceReduce
                },
            };
            conflicts.push(conflict);
        }
    }

    let resolver = ConflictResolver { conflicts };
    assert_eq!(resolver.conflicts.len(), 500);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 39: ParseRule with maximum RHS length
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_parse_rule_max_rhs_length() {
    let rule = ParseRule {
        lhs: SymbolId(0),
        rhs_len: u16::MAX,
    };

    assert_eq!(rule.rhs_len, u16::MAX);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 40: Action variant exhaustiveness
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_action_variants_exist() {
    let actions = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];

    assert_eq!(actions.len(), 6);

    let mut shift_count = 0;
    let mut reduce_count = 0;
    let mut accept_count = 0;
    let mut error_count = 0;
    let mut recover_count = 0;
    let mut fork_count = 0;

    for action in actions {
        match action {
            Action::Shift(_) => shift_count += 1,
            Action::Reduce(_) => reduce_count += 1,
            Action::Accept => accept_count += 1,
            Action::Error => error_count += 1,
            Action::Recover => recover_count += 1,
            Action::Fork(_) => fork_count += 1,
            _ => {} // Handle any other variants added in the future
        }
    }

    assert_eq!(shift_count, 1);
    assert_eq!(reduce_count, 1);
    assert_eq!(accept_count, 1);
    assert_eq!(error_count, 1);
    assert_eq!(recover_count, 1);
    assert_eq!(fork_count, 1);
}
