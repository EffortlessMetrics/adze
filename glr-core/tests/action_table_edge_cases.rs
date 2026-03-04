#![cfg(feature = "test-api")]

//! Edge-case tests for GLR action table operations.
//!
//! Covers multi-action cells, fork creation, boundary conditions,
//! GOTO remapping, conflict classification, and table validation.

use adze_glr_core::conflict_inspection::{
    ConflictType, classify_conflict, count_conflicts, find_conflicts_for_symbol,
    get_state_conflicts, state_has_conflicts,
};
use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseTable, build_lr1_automaton,
    sanity_check_tables,
};
use adze_ir::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Construct a ParseTable by hand with given dimensions and content.
fn hand_built_table(
    action_table: Vec<Vec<Vec<Action>>>,
    goto_table: Vec<Vec<StateId>>,
    symbol_to_index: BTreeMap<SymbolId, usize>,
    nonterminal_to_index: BTreeMap<SymbolId, usize>,
    eof_symbol: SymbolId,
    start_symbol: SymbolId,
) -> ParseTable {
    let state_count = action_table.len();
    let symbol_count = if state_count > 0 {
        action_table[0].len()
    } else {
        0
    };
    let mut index_to_symbol = vec![SymbolId(u16::MAX); symbol_to_index.len()];
    for (sym, &idx) in &symbol_to_index {
        if idx < index_to_symbol.len() {
            index_to_symbol[idx] = *sym;
        }
    }
    ParseTable {
        action_table,
        goto_table,
        symbol_metadata: vec![],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index,
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol,
        grammar: Grammar::new("test".into()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Build grammar: S → a
fn grammar_s_to_a() -> Grammar {
    let mut g = Grammar::new("s_to_a".into());
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

/// Build grammar with shift-reduce conflict: E → a | E '+' E
fn grammar_shift_reduce() -> Grammar {
    let mut g = Grammar::new("sr".into());
    let a = SymbolId(1);
    let plus = SymbolId(2);
    let e = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
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
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(plus),
                    Symbol::NonTerminal(e),
                ],
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
// 1. Fork action containing nested forks
// ===========================================================================

#[test]
fn fork_with_nested_fork() {
    let inner = Action::Fork(vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))]);
    let outer = Action::Fork(vec![inner.clone(), Action::Shift(StateId(3))]);
    if let Action::Fork(actions) = &outer {
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], inner);
        assert_eq!(actions[1], Action::Shift(StateId(3)));
    } else {
        panic!("expected Fork");
    }
}

// ===========================================================================
// 2. Fork with empty action list
// ===========================================================================

#[test]
fn fork_with_empty_actions() {
    let fork = Action::Fork(vec![]);
    if let Action::Fork(actions) = &fork {
        assert!(actions.is_empty());
    } else {
        panic!("expected Fork");
    }
}

// ===========================================================================
// 3. Fork with single action (degenerate)
// ===========================================================================

#[test]
fn fork_with_single_action() {
    let fork = Action::Fork(vec![Action::Accept]);
    if let Action::Fork(actions) = &fork {
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Accept);
    } else {
        panic!("expected Fork");
    }
}

// ===========================================================================
// 4. Cell with all action variants simultaneously
// ===========================================================================

#[test]
fn cell_with_all_action_variants() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let cell_actions = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
    ];
    let pt = hand_built_table(
        vec![vec![cell_actions.clone()]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell.len(), 5);
    assert!(cell.contains(&Action::Shift(StateId(1))));
    assert!(cell.contains(&Action::Reduce(RuleId(0))));
    assert!(cell.contains(&Action::Accept));
    assert!(cell.contains(&Action::Error));
    assert!(cell.contains(&Action::Recover));
}

// ===========================================================================
// 5. Multi-action cell with many shifts (GLR fork scenario)
// ===========================================================================

#[test]
fn cell_with_multiple_shifts_to_different_states() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![
            Action::Shift(StateId(2)),
            Action::Shift(StateId(5)),
            Action::Shift(StateId(9)),
        ]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell.len(), 3);
    // Each shift targets a distinct state
    let targets: Vec<StateId> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert_eq!(targets, vec![StateId(2), StateId(5), StateId(9)]);
}

// ===========================================================================
// 6. Multi-action cell with many reduces
// ===========================================================================

#[test]
fn cell_with_three_reduce_actions() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![
            Action::Reduce(RuleId(0)),
            Action::Reduce(RuleId(3)),
            Action::Reduce(RuleId(7)),
        ]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell.len(), 3);
    let rules: Vec<RuleId> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(rules.len(), 3);
}

// ===========================================================================
// 7. GOTO remap to DirectSymbolId and back
// ===========================================================================

#[test]
fn goto_remap_roundtrip() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let s_sym = SymbolId(10);

    // Original value via NonterminalMap
    let orig = table.goto(table.initial_state, s_sym);
    assert!(orig.is_some(), "should have goto for start symbol");

    // Remap to DirectSymbolId
    let table_direct = table.clone().remap_goto_to_direct_symbol_id();
    assert_eq!(table_direct.goto_indexing, GotoIndexing::DirectSymbolId);

    // Remap back to NonterminalMap
    let table_back = table_direct.remap_goto_to_nonterminal_map();
    assert_eq!(table_back.goto_indexing, GotoIndexing::NonterminalMap);
    let restored = table_back.goto(table_back.initial_state, s_sym);
    assert_eq!(orig, restored, "roundtrip must preserve goto values");
}

// ===========================================================================
// 8. remap_goto_to_direct_symbol_id is idempotent
// ===========================================================================

#[test]
fn goto_remap_direct_idempotent() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let once = table.clone().remap_goto_to_direct_symbol_id();
    let twice = once.clone().remap_goto_to_direct_symbol_id();
    assert_eq!(once.goto_table, twice.goto_table);
}

// ===========================================================================
// 9. remap_goto_to_nonterminal_map is idempotent
// ===========================================================================

#[test]
fn goto_remap_ntmap_idempotent() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // Already NonterminalMap, should be no-op
    let same = table.clone().remap_goto_to_nonterminal_map();
    assert_eq!(table.goto_table, same.goto_table);
}

// ===========================================================================
// 10. State 0 with only Accept on EOF (minimal accept table)
// ===========================================================================

#[test]
fn minimal_accept_only_table() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(0), 0); // EOF
    let pt = hand_built_table(
        vec![vec![vec![Action::Accept]]],
        vec![vec![]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(0),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(0));
    assert_eq!(cell, &[Action::Accept]);
    // Any other symbol returns empty
    assert!(pt.actions(StateId(0), SymbolId(1)).is_empty());
}

// ===========================================================================
// 11. Large state ID boundary (u16::MAX - 1)
// ===========================================================================

#[test]
fn shift_to_max_state_id() {
    let target = StateId(u16::MAX - 1);
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Shift(target)]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell, &[Action::Shift(target)]);
}

// ===========================================================================
// 12. Large RuleId boundary
// ===========================================================================

#[test]
fn reduce_with_max_rule_id() {
    let rule = RuleId(u16::MAX - 1);
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Reduce(rule)]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell, &[Action::Reduce(rule)]);
}

// ===========================================================================
// 13. GOTO returns None for u16::MAX sentinel
// ===========================================================================

#[test]
fn goto_sentinel_returns_none() {
    let mut nt_idx = BTreeMap::new();
    nt_idx.insert(SymbolId(10), 0);
    let pt = hand_built_table(
        vec![vec![]],
        vec![vec![StateId(u16::MAX)]], // sentinel = no edge
        BTreeMap::new(),
        nt_idx,
        SymbolId(0),
        SymbolId(10),
    );
    // u16::MAX is the "no edge" sentinel; goto should return None
    assert!(pt.goto(StateId(0), SymbolId(10)).is_none());
}

// ===========================================================================
// 14. classify_conflict edge cases
// ===========================================================================

#[test]
fn classify_empty_actions() {
    // No shifts, no reduces → Mixed
    let ty = classify_conflict(&[]);
    assert_eq!(ty, ConflictType::Mixed);
}

#[test]
fn classify_only_accept_and_error() {
    // Accept + Error: no shift, no reduce → Mixed
    let ty = classify_conflict(&[Action::Accept, Action::Error]);
    assert_eq!(ty, ConflictType::Mixed);
}

#[test]
fn classify_shift_reduce() {
    let ty = classify_conflict(&[Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    assert_eq!(ty, ConflictType::ShiftReduce);
}

#[test]
fn classify_reduce_reduce() {
    let ty = classify_conflict(&[Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))]);
    assert_eq!(ty, ConflictType::ReduceReduce);
}

#[test]
fn classify_fork_with_shift_reduce_inside() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    let ty = classify_conflict(&[fork]);
    assert_eq!(ty, ConflictType::ShiftReduce);
}

// ===========================================================================
// 15. state_has_conflicts with out-of-bounds state
// ===========================================================================

#[test]
fn state_has_conflicts_oob_returns_false() {
    let pt = ParseTable::default();
    assert!(!state_has_conflicts(&pt, StateId(100)));
    assert!(!state_has_conflicts(&pt, StateId(u16::MAX)));
}

// ===========================================================================
// 16. conflict inspection on conflict-free table
// ===========================================================================

#[test]
fn count_conflicts_on_simple_grammar() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    // S → a is LR(1), no conflicts expected
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
    assert!(summary.states_with_conflicts.is_empty());
}

// ===========================================================================
// 17. conflict inspection on ambiguous grammar
// ===========================================================================

#[test]
fn count_conflicts_on_shift_reduce_grammar() {
    let g = grammar_shift_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    // E → a | E '+' E has a shift/reduce conflict on '+'
    let total = summary.shift_reduce + summary.reduce_reduce;
    assert!(total > 0, "ambiguous grammar must have conflicts");
}

// ===========================================================================
// 18. get_state_conflicts for non-conflicting state
// ===========================================================================

#[test]
fn get_state_conflicts_returns_empty_for_clean_state() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let conflicts = get_state_conflicts(&table, table.initial_state);
    assert!(conflicts.is_empty());
}

// ===========================================================================
// 19. find_conflicts_for_symbol on non-conflicting symbol
// ===========================================================================

#[test]
fn find_conflicts_for_symbol_returns_empty_for_nonexistent() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let conflicts = find_conflicts_for_symbol(&table, SymbolId(999));
    assert!(conflicts.is_empty());
}

// ===========================================================================
// 20. valid_symbols on a constructed table
// ===========================================================================

#[test]
fn valid_symbols_marks_terminals_with_actions() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let mask = table.valid_symbols(table.initial_state);
    // At least one terminal should be valid in the initial state
    assert!(
        mask.iter().any(|&v| v),
        "initial state must have valid symbols"
    );
}

#[test]
fn valid_symbols_for_oob_state_is_all_false() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let mask = table.valid_symbols(StateId(u16::MAX));
    assert!(mask.iter().all(|&v| !v));
}

// ===========================================================================
// 21. terminal_boundary and is_terminal consistency
// ===========================================================================

#[test]
fn terminal_boundary_matches_token_plus_external() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert_eq!(
        table.terminal_boundary(),
        table.token_count + table.external_token_count
    );
}

#[test]
fn is_terminal_true_for_tokens_false_for_nonterminals() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // token 'a' = SymbolId(1) should be terminal
    assert!(table.is_terminal(SymbolId(1)));
    // nonterminal S = SymbolId(10) should not be terminal (assuming boundary < 10)
    if table.terminal_boundary() <= 10 {
        assert!(!table.is_terminal(SymbolId(10)));
    }
}

// ===========================================================================
// 22. normalize_eof_to_zero is idempotent
// ===========================================================================

#[test]
fn normalize_eof_to_zero_idempotent() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // Already SymbolId(0) after build_lr1_automaton
    let once = table.clone().normalize_eof_to_zero();
    let twice = once.clone().normalize_eof_to_zero();
    assert_eq!(once.eof_symbol, twice.eof_symbol);
    assert_eq!(once.action_table, twice.action_table);
}

// ===========================================================================
// 23. lex_mode for out-of-bounds state
// ===========================================================================

#[test]
fn lex_mode_oob_returns_zero() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let mode = table.lex_mode(StateId(u16::MAX));
    assert_eq!(mode.lex_state, 0);
    assert_eq!(mode.external_lex_state, 0);
}

// ===========================================================================
// 24. is_extra returns false for non-extra symbols
// ===========================================================================

#[test]
fn is_extra_false_for_normal_tokens() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // grammar_s_to_a has no extras defined
    assert!(!table.is_extra(SymbolId(1)));
    assert!(!table.is_extra(SymbolId(10)));
}

// ===========================================================================
// 25. Action Debug/Clone/Hash impls
// ===========================================================================

#[test]
fn action_debug_format() {
    let shift = Action::Shift(StateId(42));
    let dbg = format!("{:?}", shift);
    assert!(dbg.contains("Shift"), "Debug should contain 'Shift'");
    assert!(dbg.contains("42"), "Debug should contain state id");
}

#[test]
fn action_clone_is_independent() {
    let original = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn action_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(1)));
    set.insert(Action::Shift(StateId(1))); // duplicate
    set.insert(Action::Reduce(RuleId(0)));
    assert_eq!(set.len(), 2, "Hash must deduplicate equal actions");
}

// ===========================================================================
// 26. Table with zero-width rows (no symbols)
// ===========================================================================

#[test]
fn table_with_no_symbols() {
    let pt = hand_built_table(
        vec![vec![]], // 1 state, 0 symbol columns
        vec![vec![]],
        BTreeMap::new(),
        BTreeMap::new(),
        SymbolId(0),
        SymbolId(10),
    );
    assert_eq!(pt.state_count, 1);
    assert_eq!(pt.symbol_count, 0);
    // Querying any symbol returns empty
    assert!(pt.actions(StateId(0), SymbolId(0)).is_empty());
}

// ===========================================================================
// 27. sanity_check_tables on a valid automaton-built table
// ===========================================================================

#[test]
fn sanity_check_shift_reduce_grammar() {
    let g = grammar_shift_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    sanity_check_tables(&table).expect("sanity check must pass for E→a|E+E");
}

// ===========================================================================
// 28. Action inequality across variants
// ===========================================================================

#[test]
fn action_cross_variant_inequality() {
    let variants: Vec<Action> = vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
        Action::Fork(vec![]),
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "different variants must not be equal");
            }
        }
    }
}

// ===========================================================================
// 29. valid_symbols_mask matches valid_symbols
// ===========================================================================

#[test]
fn valid_symbols_mask_equals_valid_symbols() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    for s in 0..table.state_count {
        let v1 = table.valid_symbols(StateId(s as u16));
        let v2 = table.valid_symbols_mask(StateId(s as u16));
        assert_eq!(
            v1, v2,
            "valid_symbols and valid_symbols_mask must agree for state {s}"
        );
    }
}

// ===========================================================================
// 30. detect_goto_indexing picks NonterminalMap for automaton tables
// ===========================================================================

#[test]
fn detect_goto_indexing_on_automaton_table() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let mut table = build_lr1_automaton(&g, &ff).unwrap();
    table.detect_goto_indexing();
    // Automaton-built tables use NonterminalMap
    assert_eq!(table.goto_indexing, GotoIndexing::NonterminalMap);
}

// ===========================================================================
// 31. Multiple symbols mapped to different columns
// ===========================================================================

#[test]
fn actions_with_sparse_symbol_mapping() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(100), 0);
    sym_idx.insert(SymbolId(200), 1);
    sym_idx.insert(SymbolId(300), 2);
    let pt = hand_built_table(
        vec![vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Reduce(RuleId(0))],
            vec![Action::Accept],
        ]],
        vec![vec![StateId(0); 3]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    assert_eq!(
        pt.actions(StateId(0), SymbolId(100)),
        &[Action::Shift(StateId(1))]
    );
    assert_eq!(
        pt.actions(StateId(0), SymbolId(200)),
        &[Action::Reduce(RuleId(0))]
    );
    assert_eq!(pt.actions(StateId(0), SymbolId(300)), &[Action::Accept]);
    // Unmapped symbol still returns empty
    assert!(pt.actions(StateId(0), SymbolId(150)).is_empty());
}
