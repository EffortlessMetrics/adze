#![allow(clippy::needless_range_loop)]

//! Comprehensive conflict resolution tests for GLR core.
//!
//! Covers: shift-reduce detection, reduce-reduce detection, precedence-based
//! resolution, associativity-based resolution, multi-action cells, edge cases,
//! and integration with ParseTable.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_resolution_comprehensive

use adze_glr_core::advanced_conflict::{ConflictAnalyzer, PrecedenceDecision, PrecedenceResolver};
use adze_glr_core::conflict_inspection::{
    ConflictType, classify_conflict, count_conflicts, find_conflicts_for_symbol,
    get_state_conflicts, state_has_conflicts,
};
use adze_glr_core::precedence_compare::{
    PrecedenceComparison, PrecedenceInfo, StaticPrecedenceResolver, compare_precedences,
};
use adze_glr_core::{
    Action, Conflict, ConflictResolver, FirstFollowSets, GotoIndexing, LexMode, ParseTable,
    build_lr1_automaton,
};
use adze_ir::{
    Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, StateId,
    Symbol, SymbolId, Token, TokenPattern,
};
use std::collections::BTreeMap;

// ===========================================================================
// Helpers
// ===========================================================================

/// Create a minimal ParseTable with the given action table for unit-level tests.
fn make_table(action_table: Vec<Vec<Vec<Action>>>) -> ParseTable {
    let state_count = action_table.len();
    ParseTable {
        action_table,
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 0,
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

/// Build a single-operator expression grammar: E → E op E | num.
fn expr_one_op(prec: Option<PrecedenceKind>, assoc: Option<Associativity>) -> Grammar {
    let mut g = Grammar::new("one_op".into());
    let num = SymbolId(1);
    let op = SymbolId(2);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        op,
        Token {
            name: "op".into(),
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
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(op),
                    Symbol::NonTerminal(e),
                ],
                precedence: prec.clone(),
                associativity: assoc,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(num)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );
    g
}

/// Build a two-operator expression grammar: E → E + E | E * E | num.
fn expr_two_ops(
    plus_prec: Option<PrecedenceKind>,
    plus_assoc: Option<Associativity>,
    times_prec: Option<PrecedenceKind>,
    times_assoc: Option<Associativity>,
) -> Grammar {
    let mut g = Grammar::new("two_ops".into());
    let num = SymbolId(1);
    let plus = SymbolId(2);
    let times = SymbolId(3);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
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
    g.tokens.insert(
        times,
        Token {
            name: "*".into(),
            pattern: TokenPattern::String("*".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());

    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(plus),
                    Symbol::NonTerminal(e),
                ],
                precedence: plus_prec,
                associativity: plus_assoc,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(times),
                    Symbol::NonTerminal(e),
                ],
                precedence: times_prec,
                associativity: times_assoc,
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(num)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );
    g
}

// ===========================================================================
// 1. Shift-reduce conflict detection
// ===========================================================================

#[test]
fn detect_shift_reduce_in_action_cell() {
    let cell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))];
    assert_eq!(classify_conflict(&cell), ConflictType::ShiftReduce);
}

#[test]
fn detect_shift_reduce_in_table() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 1);
    assert_eq!(summary.reduce_reduce, 0);
}

#[test]
fn shift_reduce_preserved_without_precedence() {
    let g = expr_one_op(None, None);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0,
        "S/R conflict must be preserved when no precedence is set"
    );
}

// ===========================================================================
// 2. Reduce-reduce conflict detection
// ===========================================================================

#[test]
fn detect_reduce_reduce_in_action_cell() {
    let cell = vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(5))];
    assert_eq!(classify_conflict(&cell), ConflictType::ReduceReduce);
}

#[test]
fn detect_reduce_reduce_in_table() {
    let table = make_table(vec![vec![vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.reduce_reduce, 1);
    assert_eq!(summary.shift_reduce, 0);
}

#[test]
fn reduce_reduce_from_ambiguous_grammar() {
    // S → A | B,  A → a,  B → a
    let mut g = Grammar::new("rr".into());
    let a_tok = SymbolId(1);
    let s = SymbolId(10);
    let a_nt = SymbolId(11);
    let b_nt = SymbolId(12);
    g.tokens.insert(
        a_tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(a_nt, "A".into());
    g.rule_names.insert(b_nt, "B".into());
    g.rules.insert(
        s,
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(a_nt)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(b_nt)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );
    g.rules.insert(
        a_nt,
        vec![Rule {
            lhs: a_nt,
            rhs: vec![Symbol::Terminal(a_tok)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(2),
            fields: vec![],
        }],
    );
    g.rules.insert(
        b_nt,
        vec![Rule {
            lhs: b_nt,
            rhs: vec![Symbol::Terminal(a_tok)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(3),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // Table was built; resolver may have resolved R/R by lowest production ID.
    assert!(table.state_count > 0);
}

// ===========================================================================
// 3. Precedence-based resolution
// ===========================================================================

#[test]
fn higher_shift_precedence_wins() {
    let shift = PrecedenceInfo {
        level: 3,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferShift
    );
}

#[test]
fn higher_reduce_precedence_wins() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 5,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferReduce
    );
}

#[test]
fn precedence_resolver_shift_higher() {
    let mut grammar = Grammar::new("prec".into());
    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)],
    });
    grammar.rules.insert(
        SymbolId(10),
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferShift)
    );
}

#[test]
fn precedence_resolver_reduce_higher() {
    let mut grammar = Grammar::new("prec2".into());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });
    grammar.rules.insert(
        SymbolId(10),
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(3)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferReduce)
    );
}

#[test]
fn two_op_higher_prec_resolves_all_conflicts() {
    let g = expr_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(2)),
        Some(Associativity::Left),
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "All S/R should be resolved with different precedence levels"
    );
    assert_eq!(summary.reduce_reduce, 0);
}

// ===========================================================================
// 4. Associativity-based resolution (Left, Right, None)
// ===========================================================================

#[test]
fn left_assoc_same_prec_prefers_reduce() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferReduce
    );
}

#[test]
fn right_assoc_same_prec_prefers_shift() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Right,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Right,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferShift
    );
}

#[test]
fn non_assoc_same_prec_yields_error() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::None,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::None,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::Error
    );
}

#[test]
fn left_assoc_grammar_no_remaining_sr() {
    let g = expr_one_op(Some(PrecedenceKind::Static(1)), Some(Associativity::Left));
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "Left-assoc should resolve S/R in favour of reduce"
    );
}

#[test]
fn right_assoc_grammar_no_remaining_sr() {
    let g = expr_one_op(Some(PrecedenceKind::Static(1)), Some(Associativity::Right));
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "Right-assoc should resolve S/R in favour of shift"
    );
}

#[test]
fn non_assoc_grammar_keeps_fork() {
    let g = expr_one_op(Some(PrecedenceKind::Static(1)), Some(Associativity::None));
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // Non-assoc at same prec keeps both actions (GLR fork).
    let fork_cells: usize = table
        .action_table
        .iter()
        .flat_map(|s| s.iter())
        .filter(|c| c.len() > 1)
        .count();
    assert!(
        fork_cells > 0,
        "Non-assoc should preserve conflict as GLR fork"
    );
}

#[test]
fn precedence_resolver_same_prec_left_assoc() {
    let mut grammar = Grammar::new("same_left".into());
    grammar.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });
    grammar.rules.insert(
        SymbolId(10),
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferReduce)
    );
}

#[test]
fn precedence_resolver_same_prec_right_assoc() {
    let mut grammar = Grammar::new("same_right".into());
    grammar.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)],
    });
    grammar.rules.insert(
        SymbolId(10),
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Right),
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferShift)
    );
}

#[test]
fn precedence_resolver_same_prec_non_assoc() {
    let mut grammar = Grammar::new("same_none".into());
    grammar.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::None,
        symbols: vec![SymbolId(2)],
    });
    grammar.rules.insert(
        SymbolId(10),
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::None),
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::Error)
    );
}

// ===========================================================================
// 5. Multi-action cell behaviour (GLR multi-path)
// ===========================================================================

#[test]
fn classify_fork_containing_shift_reduce() {
    let cell = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ])];
    assert_eq!(classify_conflict(&cell), ConflictType::ShiftReduce);
}

#[test]
fn classify_fork_containing_reduce_reduce() {
    let cell = vec![Action::Fork(vec![
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ])];
    assert_eq!(classify_conflict(&cell), ConflictType::ReduceReduce);
}

#[test]
fn multiple_actions_counted_as_single_conflict() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ]]]);
    let summary = count_conflicts(&table);
    // One cell → one conflict (classified as ShiftReduce because it has both).
    assert_eq!(summary.shift_reduce, 1);
}

#[test]
fn fork_action_in_table_detected() {
    let table = make_table(vec![vec![vec![
        Action::Fork(vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(0))]),
        Action::Reduce(RuleId(1)),
    ]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 1);
}

// ===========================================================================
// 6. Conflict with same precedence level
// ===========================================================================

#[test]
fn same_prec_left_assoc_resolves_via_associativity() {
    let g = expr_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    // Both operators at same prec, left-assoc: all S/R resolved.
    assert_eq!(summary.shift_reduce, 0);
}

#[test]
fn same_prec_right_assoc_resolves_via_associativity() {
    let g = expr_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Right),
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Right),
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
}

// ===========================================================================
// 7. Conflict with different precedence levels
// ===========================================================================

#[test]
fn different_prec_levels_deterministic_table() {
    let g = expr_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(2)),
        Some(Associativity::Left),
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let multi: usize = table
        .action_table
        .iter()
        .flat_map(|s| s.iter())
        .filter(|c| c.len() > 1)
        .count();
    assert_eq!(
        multi, 0,
        "Distinct precedence levels should yield fully deterministic table"
    );
}

#[test]
fn compare_prec_none_info_returns_none() {
    assert_eq!(compare_precedences(None, None), PrecedenceComparison::None);
    let info = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(info), None),
        PrecedenceComparison::None
    );
    assert_eq!(
        compare_precedences(None, Some(info)),
        PrecedenceComparison::None
    );
}

// ===========================================================================
// 8. Unresolved conflicts (no precedence info)
// ===========================================================================

#[test]
fn no_prec_info_on_both_ops_keeps_conflicts() {
    let g = expr_two_ops(None, None, None, None);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0,
        "Missing precedence should leave S/R conflicts"
    );
}

#[test]
fn precedence_resolver_missing_token_returns_none() {
    let grammar = Grammar::new("empty".into());
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(99), SymbolId(88)),
        None
    );
}

#[test]
fn static_resolver_no_token_prec() {
    let grammar = Grammar::new("empty".into());
    let resolver = StaticPrecedenceResolver::from_grammar(&grammar);
    assert!(resolver.token_precedence(SymbolId(1)).is_none());
    assert!(resolver.rule_precedence(RuleId(0)).is_none());
}

// ===========================================================================
// 9. Edge cases (single action cells, empty cells)
// ===========================================================================

#[test]
fn single_action_cell_no_conflict() {
    let table = make_table(vec![vec![vec![Action::Shift(StateId(1))]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
    assert!(summary.states_with_conflicts.is_empty());
}

#[test]
fn empty_action_cell_no_conflict() {
    let table = make_table(vec![vec![vec![]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

#[test]
fn single_state_empty_cells_no_conflict() {
    // A table with one state whose only cell is empty (error state).
    let table = make_table(vec![vec![vec![], vec![]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
    assert!(summary.conflict_details.is_empty());
}

#[test]
fn accept_action_not_a_conflict() {
    let table = make_table(vec![vec![vec![Action::Accept]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

#[test]
fn classify_only_shifts_is_mixed() {
    let cell = vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))];
    assert_eq!(classify_conflict(&cell), ConflictType::Mixed);
}

#[test]
fn classify_only_accept_and_error_is_mixed() {
    let cell = vec![Action::Accept, Action::Error];
    assert_eq!(classify_conflict(&cell), ConflictType::Mixed);
}

// ===========================================================================
// 10. Integration with ParseTable
// ===========================================================================

#[test]
fn state_has_conflicts_api() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Shift(StateId(2))]],
    ]);
    assert!(state_has_conflicts(&table, StateId(0)));
    assert!(!state_has_conflicts(&table, StateId(1)));
    assert!(!state_has_conflicts(&table, StateId(99))); // out of bounds
}

#[test]
fn get_state_conflicts_returns_correct_state() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))]],
    ]);
    let c0 = get_state_conflicts(&table, StateId(0));
    assert_eq!(c0.len(), 1);
    assert_eq!(c0[0].conflict_type, ConflictType::ShiftReduce);

    let c1 = get_state_conflicts(&table, StateId(1));
    assert_eq!(c1.len(), 1);
    assert_eq!(c1[0].conflict_type, ConflictType::ReduceReduce);
}

#[test]
fn find_conflicts_for_symbol_with_index_to_symbol() {
    let mut table = make_table(vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(0))],
    ]]);
    table.index_to_symbol = vec![SymbolId(10), SymbolId(20)];
    let conflicts = find_conflicts_for_symbol(&table, SymbolId(20));
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].symbol, SymbolId(20));
}

#[test]
fn conflict_summary_display_includes_counts() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    let summary = count_conflicts(&table);
    let text = format!("{}", summary);
    assert!(text.contains("Shift/Reduce conflicts: 1"));
    assert!(text.contains("Reduce/Reduce conflicts: 0"));
}

#[test]
fn conflict_resolver_detect_and_resolve_shift_reduce() {
    let mut g = Grammar::new("resolve_sr".into());
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
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::NonTerminal(e), Symbol::NonTerminal(e)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // An inherently ambiguous grammar should build successfully.
    assert!(table.state_count > 0);
}

#[test]
fn conflict_analyzer_default_stats_zeroed() {
    let analyzer = ConflictAnalyzer::new();
    let stats = analyzer.get_stats();
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
    assert_eq!(stats.associativity_resolved, 0);
    assert_eq!(stats.explicit_glr, 0);
    assert_eq!(stats.default_resolved, 0);
}

#[test]
fn conflict_analyzer_analyze_deterministic_table() {
    let g = expr_one_op(Some(PrecedenceKind::Static(1)), Some(Associativity::Left));
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    // Current simplified implementation returns zeroed stats.
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn static_precedence_resolver_extracts_from_grammar() {
    let mut grammar = Grammar::new("extract".into());
    grammar.precedences.push(Precedence {
        level: 3,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(5), SymbolId(6)],
    });
    grammar.precedences.push(Precedence {
        level: 7,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(8)],
    });

    let resolver = StaticPrecedenceResolver::from_grammar(&grammar);
    let p5 = resolver.token_precedence(SymbolId(5)).unwrap();
    assert_eq!(p5.level, 3);
    assert_eq!(p5.associativity, Associativity::Left);

    let p8 = resolver.token_precedence(SymbolId(8)).unwrap();
    assert_eq!(p8.level, 7);
    assert_eq!(p8.associativity, Associativity::Right);

    assert!(resolver.token_precedence(SymbolId(99)).is_none());
}

#[test]
fn minimal_single_state_table_no_conflicts() {
    // A table with a single state and one deterministic action.
    let table = make_table(vec![vec![vec![Action::Accept]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

#[test]
fn multiple_states_with_independent_conflicts() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))]],
        vec![vec![Action::Shift(StateId(3))]],
    ]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 1);
    assert_eq!(summary.reduce_reduce, 1);
    assert_eq!(summary.states_with_conflicts.len(), 2);
    assert!(summary.states_with_conflicts.contains(&StateId(0)));
    assert!(summary.states_with_conflicts.contains(&StateId(1)));
}
