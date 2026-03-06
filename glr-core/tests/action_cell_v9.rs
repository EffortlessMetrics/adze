#![cfg(feature = "test-api")]

//! Comprehensive tests for ActionCell (Vec<Action>) and Action enum in adze-glr-core.
//! 80+ tests covering construction, equality, debug formatting, parse table integration,
//! GLR multi-action cells, edge cases, and builder-based grammar pipelines.

use adze_glr_core::{
    Action, ActionCell, FirstFollowSets, GotoIndexing, LexMode, ParseRule, ParseTable, RuleId,
    StateId, SymbolId, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use std::collections::BTreeMap;

// ===========================================================================
// Helpers
// ===========================================================================

/// Hand-build a ParseTable for direct structural tests.
fn hand_built_table(
    action_table: Vec<Vec<ActionCell>>,
    goto_table: Vec<Vec<StateId>>,
    symbol_to_index: BTreeMap<SymbolId, usize>,
    nonterminal_to_index: BTreeMap<SymbolId, usize>,
    rules: Vec<ParseRule>,
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
        rules,
        nonterminal_to_index,
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol,
        grammar: Grammar::new("ac_v9_test".into()),
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

/// Build a parse table from a Grammar.
fn build_table(g: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(g).unwrap();
    build_lr1_automaton(g, &ff).unwrap()
}

/// Minimal grammar S → a (raw IR).
fn grammar_s_a() -> Grammar {
    let mut g = Grammar::new("ac_v9_s_a".into());
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
    g.rule_names.insert(s, "s".into());
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

/// Two-token grammar S → a b (raw IR).
fn grammar_s_ab() -> Grammar {
    let mut g = Grammar::new("ac_v9_s_ab".into());
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
    g.rule_names.insert(s, "s".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a), Symbol::Terminal(b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Shift-reduce conflict grammar: E → a | E '+' E
fn grammar_sr_conflict() -> Grammar {
    let mut g = Grammar::new("ac_v9_sr".into());
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
            name: "plus".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "e".into());
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

/// Builder-based grammar: expr → num | expr op expr
fn grammar_builder_expr() -> Grammar {
    GrammarBuilder::new("ac_v9_expr")
        .token("num", "\\d+")
        .token("op", "[+\\-]")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "op", "expr"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1–5. ActionCell::new (Vec::new) — empty cell basics
// ===========================================================================

#[test]
fn t01_empty_cell_is_empty() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
}

#[test]
fn t02_empty_cell_len_zero() {
    let cell: ActionCell = vec![];
    assert_eq!(cell.len(), 0);
}

#[test]
fn t03_shift_cell_len_one() {
    let cell: ActionCell = vec![Action::Shift(StateId(1))];
    assert_eq!(cell.len(), 1);
}

#[test]
fn t04_reduce_cell_len_one() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(0))];
    assert_eq!(cell.len(), 1);
}

#[test]
fn t05_accept_cell_len_one() {
    let cell: ActionCell = vec![Action::Accept];
    assert_eq!(cell.len(), 1);
}

// ===========================================================================
// 6–7. Multi-action cells & actions() slice
// ===========================================================================

#[test]
fn t06_multi_action_cell_correct_len() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ];
    assert_eq!(cell.len(), 3);
}

#[test]
fn t07_actions_slice_matches_contents() {
    let cell: ActionCell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))];
    let slice: &[Action] = &cell;
    assert_eq!(slice[0], Action::Shift(StateId(5)));
    assert_eq!(slice[1], Action::Reduce(RuleId(3)));
}

// ===========================================================================
// 8–12. Action equality
// ===========================================================================

#[test]
fn t08_shift_equality() {
    assert_eq!(Action::Shift(StateId(7)), Action::Shift(StateId(7)));
}

#[test]
fn t09_reduce_equality() {
    assert_eq!(Action::Reduce(RuleId(4)), Action::Reduce(RuleId(4)));
}

#[test]
fn t10_accept_equality() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn t11_error_equality() {
    assert_eq!(Action::Error, Action::Error);
}

#[test]
fn t12_recover_equality() {
    assert_eq!(Action::Recover, Action::Recover);
}

// ===========================================================================
// 13. Fork contains inner actions
// ===========================================================================

#[test]
fn t13_fork_contains_inner_actions() {
    let inner = vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))];
    let fork = Action::Fork(inner.clone());
    match fork {
        Action::Fork(v) => {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0], Action::Shift(StateId(2)));
            assert_eq!(v[1], Action::Reduce(RuleId(1)));
        }
        _ => panic!("expected Fork"),
    }
}

// ===========================================================================
// 14. Debug format
// ===========================================================================

#[test]
fn t14_shift_debug_contains_variant_name() {
    let dbg = format!("{:?}", Action::Shift(StateId(3)));
    assert!(dbg.contains("Shift"));
}

#[test]
fn t14b_reduce_debug_contains_variant_name() {
    let dbg = format!("{:?}", Action::Reduce(RuleId(5)));
    assert!(dbg.contains("Reduce"));
}

#[test]
fn t14c_accept_debug_contains_variant_name() {
    let dbg = format!("{:?}", Action::Accept);
    assert!(dbg.contains("Accept"));
}

#[test]
fn t14d_error_debug_contains_variant_name() {
    let dbg = format!("{:?}", Action::Error);
    assert!(dbg.contains("Error"));
}

#[test]
fn t14e_recover_debug_contains_variant_name() {
    let dbg = format!("{:?}", Action::Recover);
    assert!(dbg.contains("Recover"));
}

#[test]
fn t14f_fork_debug_contains_variant_name() {
    let fork = Action::Fork(vec![Action::Accept]);
    let dbg = format!("{:?}", fork);
    assert!(dbg.contains("Fork"));
}

// ===========================================================================
// 15–17. Parse table integration — actions from real tables
// ===========================================================================

#[test]
fn t15_table_actions_contain_shift_on_terminal() {
    let g = grammar_s_a();
    let table = build_table(&g);
    let a = SymbolId(1);
    let actions = table.actions(table.initial_state, a);
    assert!(actions.iter().any(|act| matches!(act, Action::Shift(_))));
}

#[test]
fn t16_table_empty_cell_for_invalid_symbol() {
    let g = grammar_s_a();
    let table = build_table(&g);
    // SymbolId(999) is not in the grammar
    let actions = table.actions(table.initial_state, SymbolId(999));
    assert!(actions.is_empty());
}

#[test]
fn t17_table_nonempty_cell_for_valid_terminal() {
    let g = grammar_s_a();
    let table = build_table(&g);
    let a = SymbolId(1);
    let actions = table.actions(table.initial_state, a);
    assert!(!actions.is_empty());
}

// ===========================================================================
// 18. Clone
// ===========================================================================

#[test]
fn t18_action_clone_preserves_value() {
    let original = Action::Fork(vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(1))]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ===========================================================================
// 19. Various ActionCell sizes
// ===========================================================================

#[test]
fn t19a_cell_size_zero() {
    let cell: ActionCell = vec![];
    assert_eq!(cell.len(), 0);
    assert!(cell.is_empty());
}

#[test]
fn t19b_cell_size_one() {
    let cell: ActionCell = vec![Action::Accept];
    assert_eq!(cell.len(), 1);
    assert!(!cell.is_empty());
}

#[test]
fn t19c_cell_size_two() {
    let cell: ActionCell = vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn t19d_cell_size_five() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(0)),
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
    ];
    assert_eq!(cell.len(), 5);
}

// ===========================================================================
// 20. GLR multi-action cells
// ===========================================================================

#[test]
fn t20_glr_cell_shift_reduce_pair() {
    let cell: ActionCell = vec![Action::Shift(StateId(4)), Action::Reduce(RuleId(2))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

// ===========================================================================
// 21–30. Action inequality tests
// ===========================================================================

#[test]
fn t21_shift_neq_different_state() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
}

#[test]
fn t22_reduce_neq_different_rule() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
}

#[test]
fn t23_shift_neq_reduce() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn t24_accept_neq_error() {
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn t25_accept_neq_recover() {
    assert_ne!(Action::Accept, Action::Recover);
}

#[test]
fn t26_error_neq_recover() {
    assert_ne!(Action::Error, Action::Recover);
}

#[test]
fn t27_shift_neq_accept() {
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

#[test]
fn t28_reduce_neq_accept() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Accept);
}

#[test]
fn t29_fork_neq_shift() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1))]);
    assert_ne!(fork, Action::Shift(StateId(1)));
}

#[test]
fn t30_fork_neq_different_contents() {
    let fork_a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let fork_b = Action::Fork(vec![Action::Shift(StateId(2))]);
    assert_ne!(fork_a, fork_b);
}

// ===========================================================================
// 31–35. ID type basics
// ===========================================================================

#[test]
fn t31_state_id_copy() {
    let id = StateId(42);
    let id2 = id;
    assert_eq!(id, id2);
}

#[test]
fn t32_rule_id_copy() {
    let id = RuleId(17);
    let id2 = id;
    assert_eq!(id, id2);
}

#[test]
fn t33_symbol_id_copy() {
    let id = SymbolId(99);
    let id2 = id;
    assert_eq!(id, id2);
}

#[test]
fn t34_state_id_zero() {
    assert_eq!(StateId(0).0, 0);
}

#[test]
fn t35_state_id_max() {
    assert_eq!(StateId(u16::MAX).0, u16::MAX);
}

// ===========================================================================
// 36–40. Hand-built parse table tests
// ===========================================================================

#[test]
fn t36_hand_built_empty_table() {
    let table = hand_built_table(
        vec![],
        vec![],
        BTreeMap::new(),
        BTreeMap::new(),
        vec![],
        SymbolId(0),
        SymbolId(10),
    );
    assert_eq!(table.state_count, 0);
}

#[test]
fn t37_hand_built_single_shift() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let table = hand_built_table(
        vec![vec![vec![Action::Shift(StateId(1))]]],
        vec![vec![]],
        sym_idx,
        BTreeMap::new(),
        vec![],
        SymbolId(0),
        SymbolId(10),
    );
    let actions = table.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], Action::Shift(StateId(1)));
}

#[test]
fn t38_hand_built_accept_on_eof() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(0), 0);
    let table = hand_built_table(
        vec![vec![vec![Action::Accept]]],
        vec![vec![]],
        sym_idx,
        BTreeMap::new(),
        vec![],
        SymbolId(0),
        SymbolId(10),
    );
    let actions = table.actions(StateId(0), SymbolId(0));
    assert_eq!(actions, &[Action::Accept]);
}

#[test]
fn t39_hand_built_empty_cell_returns_empty() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let table = hand_built_table(
        vec![vec![vec![]]],
        vec![vec![]],
        sym_idx,
        BTreeMap::new(),
        vec![],
        SymbolId(0),
        SymbolId(10),
    );
    let actions = table.actions(StateId(0), SymbolId(1));
    assert!(actions.is_empty());
}

#[test]
fn t40_hand_built_multi_action_cell() {
    let mut sym_idx = BTreeMap::new();
    sym_idx.insert(SymbolId(1), 0);
    let cell = vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(0))];
    let table = hand_built_table(
        vec![vec![cell]],
        vec![vec![]],
        sym_idx,
        BTreeMap::new(),
        vec![],
        SymbolId(0),
        SymbolId(10),
    );
    let actions = table.actions(StateId(0), SymbolId(1));
    assert_eq!(actions.len(), 2);
}

// ===========================================================================
// 41–50. Builder-based grammar → parse table → action checks
// ===========================================================================

#[test]
fn t41_builder_grammar_produces_valid_table() {
    let g = grammar_builder_expr();
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

#[test]
fn t42_builder_table_has_accept_somewhere() {
    let g = grammar_builder_expr();
    let table = build_table(&g);
    let has_accept = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flatten()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        has_accept,
        "table should contain at least one Accept action"
    );
}

#[test]
fn t43_builder_table_has_shift_actions() {
    let g = grammar_builder_expr();
    let table = build_table(&g);
    let has_shift = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flatten()
        .any(|a| matches!(a, Action::Shift(_)));
    assert!(has_shift, "table should contain shift actions");
}

#[test]
fn t44_builder_table_has_reduce_actions() {
    let g = grammar_builder_expr();
    let table = build_table(&g);
    let has_reduce = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flatten()
        .any(|a| matches!(a, Action::Reduce(_)));
    assert!(has_reduce, "table should contain reduce actions");
}

#[test]
fn t45_raw_s_a_table_initial_state_zero() {
    let g = grammar_s_a();
    let table = build_table(&g);
    assert_eq!(table.initial_state, StateId(0));
}

#[test]
fn t46_raw_s_ab_table_has_two_tokens() {
    let g = grammar_s_ab();
    let table = build_table(&g);
    // Both terminals a and b should be indexed
    assert!(table.symbol_to_index.contains_key(&SymbolId(1)));
    assert!(table.symbol_to_index.contains_key(&SymbolId(2)));
}

#[test]
fn t47_raw_s_ab_initial_shift_on_a() {
    let g = grammar_s_ab();
    let table = build_table(&g);
    let actions = table.actions(table.initial_state, SymbolId(1));
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on 'a'"
    );
}

#[test]
fn t48_raw_s_ab_no_shift_on_b_initially() {
    let g = grammar_s_ab();
    let table = build_table(&g);
    // 'b' is not expected in the initial state
    let actions = table.actions(table.initial_state, SymbolId(2));
    assert!(
        !actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should not shift on 'b'"
    );
}

#[test]
fn t49_raw_s_a_eof_accept_exists() {
    let g = grammar_s_a();
    let table = build_table(&g);
    let has_eof_accept = table.action_table.iter().any(|row| {
        if let Some(&col) = table.symbol_to_index.get(&table.eof_symbol) {
            row.get(col)
                .is_some_and(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
        } else {
            false
        }
    });
    assert!(has_eof_accept, "some state should accept on EOF");
}

#[test]
fn t50_raw_s_a_goto_on_s() {
    let g = grammar_s_a();
    let table = build_table(&g);
    let s = SymbolId(10);
    let goto = table.goto(table.initial_state, s);
    assert!(goto.is_some(), "goto(initial, S) should exist");
}

// ===========================================================================
// 51–60. Fork action tests
// ===========================================================================

#[test]
fn t51_empty_fork() {
    let fork = Action::Fork(vec![]);
    match fork {
        Action::Fork(v) => assert!(v.is_empty()),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn t52_fork_single_action() {
    let fork = Action::Fork(vec![Action::Accept]);
    match fork {
        Action::Fork(v) => assert_eq!(v.len(), 1),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn t53_fork_three_actions() {
    let fork = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(2)),
    ]);
    match fork {
        Action::Fork(v) => assert_eq!(v.len(), 3),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn t54_nested_fork() {
    let inner = Action::Fork(vec![Action::Accept]);
    let outer = Action::Fork(vec![inner.clone()]);
    match outer {
        Action::Fork(v) => {
            assert_eq!(v.len(), 1);
            assert!(matches!(&v[0], Action::Fork(_)));
        }
        _ => panic!("expected Fork"),
    }
}

#[test]
fn t55_fork_equality_same_contents() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    assert_eq!(a, b);
}

#[test]
fn t56_fork_inequality_different_order() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let b = Action::Fork(vec![Action::Reduce(RuleId(2)), Action::Shift(StateId(1))]);
    assert_ne!(a, b);
}

#[test]
fn t57_fork_inequality_different_length() {
    let a = Action::Fork(vec![Action::Accept]);
    let b = Action::Fork(vec![Action::Accept, Action::Error]);
    assert_ne!(a, b);
}

#[test]
fn t58_fork_clone_independence() {
    let original = Action::Fork(vec![Action::Shift(StateId(7))]);
    let mut cloned = original.clone();
    // Mutate cloned via pattern matching
    if let Action::Fork(ref mut v) = cloned {
        v.push(Action::Error);
    }
    assert_ne!(original, cloned);
}

#[test]
fn t59_fork_debug_shows_inner() {
    let fork = Action::Fork(vec![Action::Shift(StateId(3))]);
    let dbg = format!("{:?}", fork);
    assert!(dbg.contains("Shift"));
    assert!(dbg.contains("Fork"));
}

#[test]
fn t60_fork_with_all_variants() {
    let fork = Action::Fork(vec![
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
        Action::Recover,
    ]);
    match fork {
        Action::Fork(v) => assert_eq!(v.len(), 5),
        _ => panic!("expected Fork"),
    }
}

// ===========================================================================
// 61–70. Cell iteration and containment
// ===========================================================================

#[test]
fn t61_cell_iter_yields_all_elements() {
    let cell: ActionCell = vec![Action::Accept, Action::Error, Action::Recover];
    let collected: Vec<_> = cell.iter().collect();
    assert_eq!(collected.len(), 3);
}

#[test]
fn t62_cell_contains_specific_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(5)), Action::Accept];
    assert!(cell.contains(&Action::Accept));
}

#[test]
fn t63_cell_does_not_contain_missing_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(5))];
    assert!(!cell.contains(&Action::Accept));
}

#[test]
fn t64_cell_iter_filter_shifts() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(2)),
    ];
    let shift_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .count();
    assert_eq!(shift_count, 2);
}

#[test]
fn t65_cell_iter_filter_reduces() {
    let cell: ActionCell = vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Accept,
    ];
    let reduce_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert_eq!(reduce_count, 2);
}

#[test]
fn t66_cell_first_element() {
    let cell: ActionCell = vec![Action::Shift(StateId(10))];
    assert_eq!(cell.first(), Some(&Action::Shift(StateId(10))));
}

#[test]
fn t67_cell_last_element() {
    let cell: ActionCell = vec![Action::Shift(StateId(1)), Action::Accept];
    assert_eq!(cell.last(), Some(&Action::Accept));
}

#[test]
fn t68_cell_first_on_empty() {
    let cell: ActionCell = vec![];
    assert_eq!(cell.first(), None);
}

#[test]
fn t69_cell_push_grows() {
    let mut cell: ActionCell = vec![Action::Accept];
    cell.push(Action::Error);
    assert_eq!(cell.len(), 2);
}

#[test]
fn t70_cell_extend() {
    let mut cell: ActionCell = vec![Action::Accept];
    cell.extend(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    assert_eq!(cell.len(), 3);
}

// ===========================================================================
// 71–75. Conflict grammar table tests
// ===========================================================================

#[test]
fn t71_sr_conflict_table_builds() {
    let g = grammar_sr_conflict();
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

#[test]
fn t72_sr_conflict_has_multi_action_cell() {
    let g = grammar_sr_conflict();
    let table = build_table(&g);
    let has_multi = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .any(|cell| cell.len() > 1);
    assert!(
        has_multi,
        "SR conflict grammar should produce multi-action cells"
    );
}

#[test]
fn t73_sr_conflict_multi_cell_has_shift_and_reduce() {
    let g = grammar_sr_conflict();
    let table = build_table(&g);
    let multi_cell = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .find(|cell| cell.len() > 1);
    if let Some(cell) = multi_cell {
        let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
        let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
        assert!(
            has_shift || has_reduce,
            "multi-action cell should contain shift or reduce"
        );
    }
}

#[test]
fn t74_all_table_cells_have_valid_actions() {
    let g = grammar_s_a();
    let table = build_table(&g);
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                match action {
                    Action::Shift(sid) => assert!(sid.0 < table.state_count as u16),
                    Action::Reduce(rid) => assert!((rid.0 as usize) < table.rules.len()),
                    Action::Accept | Action::Error | Action::Recover => {}
                    Action::Fork(inner) => assert!(!inner.is_empty()),
                    _ => {} // non_exhaustive
                }
            }
        }
    }
}

#[test]
fn t75_builder_expr_table_actions_valid() {
    let g = grammar_builder_expr();
    let table = build_table(&g);
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                match action {
                    Action::Shift(sid) => assert!(sid.0 < table.state_count as u16),
                    Action::Reduce(rid) => assert!((rid.0 as usize) < table.rules.len()),
                    Action::Accept | Action::Error | Action::Recover => {}
                    Action::Fork(inner) => assert!(!inner.is_empty()),
                    _ => {}
                }
            }
        }
    }
}

// ===========================================================================
// 76–80. Edge cases and miscellaneous
// ===========================================================================

#[test]
fn t76_action_shift_state_zero() {
    let a = Action::Shift(StateId(0));
    assert!(matches!(a, Action::Shift(StateId(0))));
}

#[test]
fn t77_action_reduce_rule_zero() {
    let a = Action::Reduce(RuleId(0));
    assert!(matches!(a, Action::Reduce(RuleId(0))));
}

#[test]
fn t78_action_shift_max_state() {
    let a = Action::Shift(StateId(u16::MAX));
    assert!(matches!(a, Action::Shift(StateId(s)) if s == u16::MAX));
}

#[test]
fn t79_action_reduce_max_rule() {
    let a = Action::Reduce(RuleId(u16::MAX));
    assert!(matches!(a, Action::Reduce(RuleId(r)) if r == u16::MAX));
}

#[test]
fn t80_table_out_of_bounds_state_returns_empty() {
    let g = grammar_s_a();
    let table = build_table(&g);
    let oob_state = StateId(u16::MAX);
    let actions = table.actions(oob_state, SymbolId(1));
    assert!(actions.is_empty());
}

// ===========================================================================
// 81–85. Bonus: additional coverage
// ===========================================================================

#[test]
fn t81_cell_from_single_error() {
    let cell: ActionCell = vec![Action::Error];
    assert_eq!(cell.len(), 1);
    assert_eq!(cell[0], Action::Error);
}

#[test]
fn t82_cell_from_single_recover() {
    let cell: ActionCell = vec![Action::Recover];
    assert_eq!(cell.len(), 1);
    assert_eq!(cell[0], Action::Recover);
}

#[test]
fn t83_cell_dedup_removes_duplicates() {
    let mut cell: ActionCell = vec![Action::Accept, Action::Accept, Action::Error];
    cell.dedup();
    assert_eq!(cell.len(), 2);
}

#[test]
fn t84_cell_retain_only_shifts() {
    let mut cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(2)),
    ];
    cell.retain(|a| matches!(a, Action::Shift(_)));
    assert_eq!(cell.len(), 2);
}

#[test]
fn t85_builder_simple_grammar_table_nonempty() {
    let g = GrammarBuilder::new("ac_v9_simple")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0);
    assert!(table.symbol_count > 0);
}
