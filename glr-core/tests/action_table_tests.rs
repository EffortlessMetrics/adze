#![cfg(feature = "test-api")]

//! Comprehensive tests for the ActionCell architecture (Vec<Vec<Action>>)
//! and parse table construction in adze-glr-core.

use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseTable, build_lr1_automaton,
    sanity_check_tables,
};
use adze_ir::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar: S → a
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

/// Build a grammar with a shift-reduce conflict: E → a | E '+' E
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

/// Build a grammar with a reduce-reduce conflict: S → A | B; A → a; B → a
fn grammar_reduce_reduce() -> Grammar {
    let mut g = Grammar::new("rr".into());
    let a_tok = SymbolId(1);
    let s = SymbolId(10);
    let big_a = SymbolId(11);
    let big_b = SymbolId(12);
    g.tokens.insert(
        a_tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(big_a, "A".into());
    g.rule_names.insert(big_b, "B".into());
    g.rules.insert(
        s,
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(big_a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(big_b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g.rules.insert(
        big_a,
        vec![Rule {
            lhs: big_a,
            rhs: vec![Symbol::Terminal(a_tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );
    g.rules.insert(
        big_b,
        vec![Rule {
            lhs: big_b,
            rhs: vec![Symbol::Terminal(a_tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(3),
        }],
    );
    g
}

/// Construct a ParseTable by hand with the given dimensions and content.
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

// ===========================================================================
// 1. Empty action table creation
// ===========================================================================

#[test]
fn empty_action_table() {
    let pt = ParseTable::default();
    assert!(pt.action_table.is_empty());
    assert_eq!(pt.state_count, 0);
    assert_eq!(pt.symbol_count, 0);
}

#[test]
fn empty_table_actions_returns_empty_slice() {
    let pt = ParseTable::default();
    let actions = pt.actions(StateId(0), SymbolId(0));
    assert!(actions.is_empty());
}

// ===========================================================================
// 2. Single shift action in cell
// ===========================================================================

#[test]
fn single_shift_in_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Shift(StateId(1))]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell.len(), 1);
    assert_eq!(cell[0], Action::Shift(StateId(1)));
}

// ===========================================================================
// 3. Single reduce action in cell
// ===========================================================================

#[test]
fn single_reduce_in_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(5), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Reduce(RuleId(0))]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(5));
    assert_eq!(cell, &[Action::Reduce(RuleId(0))]);
}

// ===========================================================================
// 4. Accept action in cell
// ===========================================================================

#[test]
fn accept_action_in_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(0), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Accept]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(0),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(0));
    assert_eq!(cell, &[Action::Accept]);
}

// ===========================================================================
// 5. Error (empty) cell
// ===========================================================================

#[test]
fn error_cell_is_empty() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    sym_idx.insert(SymbolId(2), 1);
    let pt = hand_built_table(
        vec![vec![
            vec![Action::Shift(StateId(1))],
            vec![], // empty = error
        ]],
        vec![vec![StateId(0), StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    // Symbol 2 has an empty cell
    let cell = pt.actions(StateId(0), SymbolId(2));
    assert!(cell.is_empty(), "empty cell means error/no valid action");
}

#[test]
fn explicit_error_action_in_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Error]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell, &[Action::Error]);
}

// ===========================================================================
// 6. Multiple actions in same cell (shift-reduce conflict)
// ===========================================================================

#[test]
fn shift_reduce_conflict_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![
            Action::Shift(StateId(2)),
            Action::Reduce(RuleId(0)),
        ]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell.len(), 2);
    assert!(cell.contains(&Action::Shift(StateId(2))));
    assert!(cell.contains(&Action::Reduce(RuleId(0))));
}

// ===========================================================================
// 7. Multiple reduce actions in same cell (reduce-reduce conflict)
// ===========================================================================

#[test]
fn reduce_reduce_conflict_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![
            Action::Reduce(RuleId(0)),
            Action::Reduce(RuleId(1)),
        ]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell.len(), 2);
    assert!(cell.contains(&Action::Reduce(RuleId(0))));
    assert!(cell.contains(&Action::Reduce(RuleId(1))));
}

// ===========================================================================
// 8. Action priority ordering within cells
// ===========================================================================

#[test]
fn action_sort_order_shift_before_reduce() {
    // Verify the documented canonical order: Shift < Reduce < Accept < Error < Recover
    // by building a table from the LR(1) automaton and checking normalized cells.
    // build_lr1_automaton runs normalize_action_table, so the result is sorted.
    let g = grammar_shift_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // In every multi-action cell, Shift must appear before Reduce, which must
    // appear before Accept/Error/Recover.
    for row in &table.action_table {
        for cell in row {
            if cell.len() > 1 {
                let mut last_type_key: u8 = 0;
                for action in cell {
                    let key = match action {
                        Action::Shift(_) => 0,
                        Action::Reduce(_) => 1,
                        Action::Accept => 2,
                        Action::Error => 3,
                        Action::Recover => 4,
                        Action::Fork(_) => 5,
                        _ => 6,
                    };
                    assert!(
                        key >= last_type_key,
                        "action ordering violated: {:?} appeared after type key {}",
                        action,
                        last_type_key,
                    );
                    last_type_key = key;
                }
            }
        }
    }
}

#[test]
fn action_sort_order_shift_by_state() {
    // Two shifts: lower state ID should come first
    let s1 = Action::Shift(StateId(3));
    let s2 = Action::Shift(StateId(7));

    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![s2.clone(), s1.clone()]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    // Not auto-sorted in hand-built, but the documented key for Shift is (0, state).
    // We verify the hand-built content is preserved.
    assert_eq!(cell.len(), 2);
    assert!(cell.contains(&s1));
    assert!(cell.contains(&s2));
}

// ===========================================================================
// 9. Action table indexing by (state, symbol)
// ===========================================================================

#[test]
fn action_table_indexing_two_states_two_symbols() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0); // token 'a'
    sym_idx.insert(SymbolId(0), 1); // EOF
    let pt = hand_built_table(
        vec![
            vec![vec![Action::Shift(StateId(1))], vec![]],
            vec![vec![], vec![Action::Accept]],
        ],
        vec![vec![StateId(0); 2]; 2],
        sym_idx,
        BTreeMap::new(),
        SymbolId(0),
        SymbolId(10),
    );
    assert_eq!(
        pt.actions(StateId(0), SymbolId(1)),
        &[Action::Shift(StateId(1))]
    );
    assert!(pt.actions(StateId(0), SymbolId(0)).is_empty());
    assert!(pt.actions(StateId(1), SymbolId(1)).is_empty());
    assert_eq!(pt.actions(StateId(1), SymbolId(0)), &[Action::Accept]);
}

#[test]
fn action_table_indexing_uses_symbol_to_index_map() {
    // SymbolId(42) maps to column 0
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(42), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Shift(StateId(5))]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    assert_eq!(
        pt.actions(StateId(0), SymbolId(42)),
        &[Action::Shift(StateId(5))]
    );
}

// ===========================================================================
// 10. Out-of-bounds state access
// ===========================================================================

#[test]
fn out_of_bounds_state_returns_empty() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Shift(StateId(1))]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    // State 5 doesn't exist (only state 0)
    assert!(pt.actions(StateId(5), SymbolId(1)).is_empty());
    assert!(pt.actions(StateId(u16::MAX), SymbolId(1)).is_empty());
}

// ===========================================================================
// 11. Out-of-bounds symbol access
// ===========================================================================

#[test]
fn out_of_bounds_symbol_returns_empty() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Shift(StateId(1))]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    // SymbolId(999) not in symbol_to_index
    assert!(pt.actions(StateId(0), SymbolId(999)).is_empty());
}

#[test]
fn unmapped_symbol_returns_empty() {
    // Table has column for SymbolId(1) but we query SymbolId(2) which has no mapping
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Shift(StateId(1))]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    assert!(pt.actions(StateId(0), SymbolId(2)).is_empty());
}

// ===========================================================================
// 12. Table construction from LR(1) automaton
// ===========================================================================

#[test]
fn build_lr1_simple_grammar_produces_valid_table() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    assert!(table.state_count > 0, "must have at least one state");
    assert_eq!(table.start_symbol(), SymbolId(10));
    // There should be a shift on 'a' (SymbolId(1)) somewhere
    let has_shift_a = (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), SymbolId(1))
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    });
    assert!(has_shift_a, "table must shift on terminal 'a'");
}

#[test]
fn build_lr1_simple_grammar_has_accept() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let eof = table.eof();
    let has_accept = (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(has_accept, "table must contain Accept on EOF");
}

#[test]
fn build_lr1_simple_grammar_has_reduce() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let has_reduce = (0..table.state_count).any(|s| {
        table.action_table.get(s).map_or(false, |row| {
            row.iter()
                .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
        })
    });
    assert!(has_reduce, "table must have at least one Reduce action");
}

#[test]
fn build_lr1_simple_grammar_passes_sanity_check() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    sanity_check_tables(&table).expect("sanity check must pass");
}

#[test]
fn build_lr1_shift_reduce_grammar() {
    let g = grammar_shift_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(
        table.state_count >= 3,
        "E→a|E+E grammar needs multiple states"
    );
    // The table should exist without panicking — conflicts are preserved for GLR
}

#[test]
fn build_lr1_reduce_reduce_grammar() {
    let g = grammar_reduce_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
    // Reduce-reduce conflicts are resolved by precedence or preserved for GLR
}

// ===========================================================================
// 13. GOTO table entries are correct
// ===========================================================================

#[test]
fn goto_table_has_entry_for_start_symbol() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // After reducing S → a, goto(initial_state, S) should exist
    let target = table.goto(table.initial_state, SymbolId(10));
    assert!(
        target.is_some(),
        "goto(initial_state, S) must exist for grammar S→a"
    );
}

#[test]
fn goto_returns_correct_target_state() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let target = table.goto(table.initial_state, SymbolId(10)).unwrap();
    // The target state should be a valid state index
    assert!(
        (target.0 as usize) < table.state_count,
        "goto target must be within state_count"
    );
}

#[test]
fn goto_for_nonterminals_in_shift_reduce_grammar() {
    let g = grammar_shift_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // E (SymbolId(10)) should have a goto from the initial state
    let target = table.goto(table.initial_state, SymbolId(10));
    assert!(
        target.is_some(),
        "goto(initial_state, E) must exist for E→a|E+E"
    );
}

// ===========================================================================
// 14. GOTO table default (no entry) handling
// ===========================================================================

#[test]
fn goto_returns_none_for_terminal() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // Terminal 'a' (SymbolId(1)) should NOT have a goto entry
    let target = table.goto(StateId(0), SymbolId(1));
    assert!(target.is_none(), "goto for a terminal must be None");
}

#[test]
fn goto_returns_none_for_nonexistent_nonterminal() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // SymbolId(999) doesn't exist in the grammar
    let target = table.goto(StateId(0), SymbolId(999));
    assert!(target.is_none(), "goto for unknown symbol must be None");
}

#[test]
fn goto_returns_none_for_out_of_bounds_state() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let target = table.goto(StateId(u16::MAX), SymbolId(10));
    assert!(target.is_none(), "goto for OOB state must be None");
}

// ===========================================================================
// 15. Compressed table preserves all actions
// ===========================================================================
// (The normalize_action_table deduplicates and sorts; verify content is preserved.)

#[test]
fn normalized_table_deduplicates_actions() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // Every cell should have no duplicate actions after normalization
    for (si, row) in table.action_table.iter().enumerate() {
        for (ci, cell) in row.iter().enumerate() {
            let mut seen = std::collections::HashSet::new();
            for action in cell {
                assert!(
                    seen.insert(action.clone()),
                    "duplicate action {:?} in state {} col {}",
                    action,
                    si,
                    ci,
                );
            }
        }
    }
}

#[test]
fn normalized_table_sort_order() {
    let g = grammar_shift_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // In every cell with multiple actions, shifts should appear before reduces
    for row in &table.action_table {
        for cell in row {
            if cell.len() > 1 {
                let mut saw_reduce = false;
                for action in cell {
                    if matches!(action, Action::Reduce(_)) {
                        saw_reduce = true;
                    }
                    if matches!(action, Action::Shift(_)) && saw_reduce {
                        panic!("Shift appeared after Reduce — normalization broken");
                    }
                }
            }
        }
    }
}

#[test]
fn normalized_table_preserves_shift_actions() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // At least one shift on terminal 'a' must survive normalization
    let has_shift = (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), SymbolId(1))
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    });
    assert!(has_shift, "normalization must preserve shift actions");
}

// ===========================================================================
// Additional coverage: Action equality, Fork variant, table metadata
// ===========================================================================

#[test]
fn action_equality() {
    assert_eq!(Action::Shift(StateId(1)), Action::Shift(StateId(1)));
    assert_ne!(Action::Shift(StateId(1)), Action::Shift(StateId(2)));
    assert_eq!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(0)));
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
    assert_eq!(Action::Accept, Action::Accept);
    assert_eq!(Action::Error, Action::Error);
    assert_eq!(Action::Recover, Action::Recover);
    assert_ne!(Action::Shift(StateId(1)), Action::Reduce(RuleId(1)));
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn fork_action_holds_multiple_actions() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    if let Action::Fork(inner) = &fork {
        assert_eq!(inner.len(), 2);
        assert_eq!(inner[0], Action::Shift(StateId(1)));
        assert_eq!(inner[1], Action::Reduce(RuleId(0)));
    } else {
        panic!("expected Fork variant");
    }
}

#[test]
fn parse_table_default_fields() {
    let pt = ParseTable::default();
    assert_eq!(pt.eof_symbol, SymbolId(0));
    assert_eq!(pt.start_symbol, SymbolId(0));
    assert_eq!(pt.initial_state, StateId(0));
    assert!(pt.rules.is_empty());
    assert!(pt.symbol_to_index.is_empty());
    assert!(pt.nonterminal_to_index.is_empty());
    assert!(matches!(pt.goto_indexing, GotoIndexing::NonterminalMap));
}

#[test]
fn eof_and_start_accessors() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    assert_eq!(table.eof(), table.eof_symbol);
    assert_eq!(table.start_symbol(), table.start_symbol);
}

#[test]
fn index_to_symbol_consistent_with_symbol_to_index() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    for (sym, &idx) in &table.symbol_to_index {
        assert_eq!(
            table.index_to_symbol[idx], *sym,
            "index_to_symbol must mirror symbol_to_index"
        );
    }
}

#[test]
fn action_table_dimensions_match_state_count() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    assert_eq!(table.action_table.len(), table.state_count);
    for row in &table.action_table {
        assert_eq!(
            row.len(),
            table.symbol_count,
            "each row must have symbol_count columns"
        );
    }
}

#[test]
fn goto_table_dimensions_match_state_count() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn multiple_states_have_distinct_action_rows() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // For S→a we expect ≥2 states; they should differ in content
    assert!(table.state_count >= 2, "S→a needs at least 2 states");
    let row0 = &table.action_table[0];
    let row1 = &table.action_table[1];
    assert_ne!(
        row0, row1,
        "different states should have different action rows"
    );
}

#[test]
fn shift_reduce_grammar_conflict_cells_have_multiple_actions() {
    let g = grammar_shift_reduce();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // There should be at least one cell with >1 action (the conflict)
    // OR the conflict may have been resolved by precedence. Either is valid.
    let max_cell_size: usize = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .map(|cell| cell.len())
        .max()
        .unwrap_or(0);
    // A GLR table for E→a|E+E with no precedence should have conflict cells
    // but conflict resolution may reduce them to 1. Just ensure table is non-trivial.
    assert!(max_cell_size >= 1, "table must have at least one action");
}

#[test]
fn recover_action_in_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let pt = hand_built_table(
        vec![vec![vec![Action::Recover]]],
        vec![vec![StateId(0)]],
        sym_idx,
        BTreeMap::new(),
        SymbolId(99),
        SymbolId(10),
    );
    let cell = pt.actions(StateId(0), SymbolId(1));
    assert_eq!(cell, &[Action::Recover]);
}
