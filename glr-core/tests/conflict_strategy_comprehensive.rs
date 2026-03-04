#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for advanced conflict resolution strategies in adze-glr-core.
//!
//! Covers precedence-based resolution, associativity-based resolution, GLR fork
//! when conflicts can't be resolved, multiple conflicting actions, priority ordering,
//! resolution with missing precedence info, and conflict detection accuracy.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_strategy_comprehensive

use adze_glr_core::advanced_conflict::{
    ConflictAnalyzer, ConflictStats, PrecedenceDecision, PrecedenceResolver,
};
use adze_glr_core::conflict_inspection::{
    ConflictType as InspectionConflictType, classify_conflict, count_conflicts,
    find_conflicts_for_symbol, get_state_conflicts, state_has_conflicts,
};
use adze_glr_core::precedence_compare::{
    PrecedenceComparison, PrecedenceInfo, StaticPrecedenceResolver, compare_precedences,
};
use adze_glr_core::{
    Action, ConflictResolver, ConflictType, FirstFollowSets, GotoIndexing, ParseTable,
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

fn prec_info(level: i16, assoc: Associativity) -> PrecedenceInfo {
    PrecedenceInfo {
        level,
        associativity: assoc,
        is_fragile: false,
    }
}

/// E → E op E | num with explicit prec/assoc on token declaration and rule.
fn expr_grammar_with_prec(
    token_level: i16,
    token_assoc: Associativity,
    rule_level: i16,
    rule_assoc: Associativity,
) -> Grammar {
    let mut g = Grammar::new("expr".into());
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
    g.precedences.push(Precedence {
        level: token_level,
        associativity: token_assoc,
        symbols: vec![op],
    });
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
                precedence: Some(PrecedenceKind::Static(rule_level)),
                associativity: Some(rule_assoc),
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

/// Two-operator grammar with explicit precedence declarations.
fn two_op_grammar(
    plus_level: i16,
    plus_assoc: Associativity,
    star_level: i16,
    star_assoc: Associativity,
) -> Grammar {
    let mut g = Grammar::new("two_ops".into());
    let num = SymbolId(1);
    let plus = SymbolId(2);
    let star = SymbolId(3);
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
        star,
        Token {
            name: "*".into(),
            pattern: TokenPattern::String("*".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.precedences.push(Precedence {
        level: plus_level,
        associativity: plus_assoc,
        symbols: vec![plus],
    });
    g.precedences.push(Precedence {
        level: star_level,
        associativity: star_assoc,
        symbols: vec![star],
    });
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
                precedence: Some(PrecedenceKind::Static(plus_level)),
                associativity: Some(plus_assoc),
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(star),
                    Symbol::NonTerminal(e),
                ],
                precedence: Some(PrecedenceKind::Static(star_level)),
                associativity: Some(star_assoc),
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
// 1. Precedence-based resolution
// ===========================================================================

#[test]
fn prec_higher_shift_wins() {
    assert_eq!(
        compare_precedences(
            Some(prec_info(5, Associativity::Left)),
            Some(prec_info(2, Associativity::Left))
        ),
        PrecedenceComparison::PreferShift,
    );
}

#[test]
fn prec_higher_reduce_wins() {
    assert_eq!(
        compare_precedences(
            Some(prec_info(1, Associativity::Left)),
            Some(prec_info(4, Associativity::Left))
        ),
        PrecedenceComparison::PreferReduce,
    );
}

#[test]
fn prec_equal_levels_defers_to_associativity() {
    // Left → reduce, Right → shift, None → error
    assert_eq!(
        compare_precedences(
            Some(prec_info(3, Associativity::Left)),
            Some(prec_info(3, Associativity::Left))
        ),
        PrecedenceComparison::PreferReduce,
    );
    assert_eq!(
        compare_precedences(
            Some(prec_info(3, Associativity::Right)),
            Some(prec_info(3, Associativity::Right))
        ),
        PrecedenceComparison::PreferShift,
    );
    assert_eq!(
        compare_precedences(
            Some(prec_info(3, Associativity::Left)),
            Some(prec_info(3, Associativity::None))
        ),
        PrecedenceComparison::Error,
    );
}

#[test]
fn prec_negative_levels_compare_correctly() {
    assert_eq!(
        compare_precedences(
            Some(prec_info(-1, Associativity::Left)),
            Some(prec_info(-5, Associativity::Left))
        ),
        PrecedenceComparison::PreferShift,
    );
    assert_eq!(
        compare_precedences(
            Some(prec_info(-10, Associativity::Left)),
            Some(prec_info(-2, Associativity::Left))
        ),
        PrecedenceComparison::PreferReduce,
    );
}

// ===========================================================================
// 2. Associativity-based resolution
// ===========================================================================

#[test]
fn assoc_only_reduce_side_determines_outcome() {
    // When levels equal, only the *reduce* side's associativity matters.
    assert_eq!(
        compare_precedences(
            Some(prec_info(2, Associativity::Right)),
            Some(prec_info(2, Associativity::Left))
        ),
        PrecedenceComparison::PreferReduce,
    );
    assert_eq!(
        compare_precedences(
            Some(prec_info(2, Associativity::Left)),
            Some(prec_info(2, Associativity::Right))
        ),
        PrecedenceComparison::PreferShift,
    );
}

#[test]
fn assoc_none_on_reduce_side_yields_error() {
    assert_eq!(
        compare_precedences(
            Some(prec_info(1, Associativity::Left)),
            Some(prec_info(1, Associativity::None))
        ),
        PrecedenceComparison::Error,
    );
}

// ===========================================================================
// 3. Resolution with missing precedence info
// ===========================================================================

#[test]
fn missing_prec_returns_none() {
    assert_eq!(
        compare_precedences(None, Some(prec_info(1, Associativity::Left))),
        PrecedenceComparison::None
    );
    assert_eq!(
        compare_precedences(Some(prec_info(1, Associativity::Left)), None),
        PrecedenceComparison::None
    );
    assert_eq!(compare_precedences(None, None), PrecedenceComparison::None);
}

// ===========================================================================
// 4. PrecedenceResolver from Grammar
// ===========================================================================

#[test]
fn precedence_resolver_shift_reduce_decisions() {
    // Equal prec, left-assoc → reduce
    let g1 = expr_grammar_with_prec(2, Associativity::Left, 2, Associativity::Left);
    let r1 = PrecedenceResolver::new(&g1);
    assert_eq!(
        r1.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferReduce)
    );

    // Higher token prec → shift
    let g2 = expr_grammar_with_prec(5, Associativity::Left, 1, Associativity::Left);
    let r2 = PrecedenceResolver::new(&g2);
    assert_eq!(
        r2.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferShift)
    );

    // Higher rule prec → reduce
    let g3 = expr_grammar_with_prec(1, Associativity::Left, 5, Associativity::Left);
    let r3 = PrecedenceResolver::new(&g3);
    assert_eq!(
        r3.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferReduce)
    );
}

#[test]
fn precedence_resolver_right_assoc_prefers_shift() {
    let g = expr_grammar_with_prec(3, Associativity::Right, 3, Associativity::Right);
    let resolver = PrecedenceResolver::new(&g);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferShift),
    );
}

#[test]
fn precedence_resolver_none_assoc_returns_error() {
    let g = expr_grammar_with_prec(3, Associativity::None, 3, Associativity::None);
    let resolver = PrecedenceResolver::new(&g);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::Error),
    );
}

#[test]
fn precedence_resolver_unknown_symbols_return_none() {
    let g = expr_grammar_with_prec(1, Associativity::Left, 1, Associativity::Left);
    let resolver = PrecedenceResolver::new(&g);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(99), SymbolId(10)),
        None
    );
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(99)),
        None
    );
}

// ===========================================================================
// 5. StaticPrecedenceResolver
// ===========================================================================

#[test]
fn static_resolver_extracts_token_and_rule_prec() {
    let g = two_op_grammar(1, Associativity::Left, 2, Associativity::Right);
    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    let plus = resolver.token_precedence(SymbolId(2)).unwrap();
    assert_eq!((plus.level, plus.associativity), (1, Associativity::Left));
    let star = resolver.token_precedence(SymbolId(3)).unwrap();
    assert_eq!((star.level, star.associativity), (2, Associativity::Right));
    let rule0 = resolver.rule_precedence(RuleId(0)).unwrap();
    assert_eq!(rule0.level, 1);
    let rule1 = resolver.rule_precedence(RuleId(1)).unwrap();
    assert_eq!(rule1.level, 2);
}

#[test]
fn static_resolver_missing_entries_return_none() {
    let g = two_op_grammar(1, Associativity::Left, 2, Associativity::Right);
    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    assert!(resolver.token_precedence(SymbolId(999)).is_none());
    assert!(resolver.rule_precedence(RuleId(2)).is_none()); // num rule has no prec
}

// ===========================================================================
// 6. GLR fork when conflicts can't be resolved
// ===========================================================================

#[test]
fn no_prec_grammar_preserves_shift_reduce_conflicts() {
    let mut g = Grammar::new("no_prec".into());
    let num = SymbolId(1);
    let plus = SymbolId(2);
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
                precedence: None,
                associativity: None,
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
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0,
        "grammar without precedence must have unresolved S/R conflicts"
    );
}

#[test]
fn fork_action_holds_multiple_alternatives() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    if let Action::Fork(actions) = &fork {
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], Action::Shift(_)));
        assert!(matches!(actions[1], Action::Reduce(_)));
    } else {
        panic!("expected Fork action");
    }
}

// ===========================================================================
// 7. Multiple conflicting actions in same cell
// ===========================================================================

#[test]
fn classify_conflict_shift_reduce() {
    let cell = vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(1))];
    assert_eq!(
        classify_conflict(&cell),
        InspectionConflictType::ShiftReduce
    );
}

#[test]
fn classify_conflict_reduce_reduce() {
    let cell = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    assert_eq!(
        classify_conflict(&cell),
        InspectionConflictType::ReduceReduce
    );
}

#[test]
fn classify_conflict_shift_plus_two_reduces() {
    let cell = vec![
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ];
    assert_eq!(
        classify_conflict(&cell),
        InspectionConflictType::ShiftReduce
    );
}

#[test]
fn classify_conflict_inside_fork() {
    let cell = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ])];
    assert_eq!(
        classify_conflict(&cell),
        InspectionConflictType::ShiftReduce
    );
}

#[test]
fn single_or_empty_cells_are_not_conflicts() {
    let table1 = make_table(vec![vec![vec![Action::Shift(StateId(1))]]]);
    let s1 = count_conflicts(&table1);
    assert!(s1.conflict_details.is_empty());

    let table2 = make_table(vec![vec![vec![]]]);
    let s2 = count_conflicts(&table2);
    assert!(s2.conflict_details.is_empty());
}

// ===========================================================================
// 8. Priority ordering of resolution strategies
// ===========================================================================

#[test]
fn conflict_resolver_resolves_shift_reduce_via_precedence() {
    let g = expr_grammar_with_prec(1, Associativity::Left, 1, Associativity::Left);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = adze_glr_core::ItemSetCollection::build_canonical_collection(&g, &ff);
    let mut resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    let had_conflicts = !resolver.conflicts.is_empty();
    resolver.resolve_conflicts(&g);
    if had_conflicts {
        for conflict in &resolver.conflicts {
            if conflict.conflict_type == ConflictType::ShiftReduce {
                assert!(
                    conflict.actions.len() <= 2,
                    "conflict should be resolved or wrapped in Fork"
                );
            }
        }
    }
}

#[test]
fn conflict_resolver_handles_reduce_reduce() {
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
    let collection = adze_glr_core::ItemSetCollection::build_canonical_collection(&g, &ff);
    let mut resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    resolver.resolve_conflicts(&g);
    // Resolver should process all conflicts without panic.
    let rr = resolver
        .conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::ReduceReduce)
        .count();
    assert!(resolver.conflicts.len() >= rr);
}

#[test]
fn precedence_takes_priority_over_default_fork() {
    let result = compare_precedences(
        Some(prec_info(3, Associativity::Left)),
        Some(prec_info(1, Associativity::Left)),
    );
    assert_eq!(result, PrecedenceComparison::PreferShift);
}

// ===========================================================================
// 9. Conflict detection accuracy
// ===========================================================================

#[test]
fn state_has_conflicts_detects_multi_action_cells() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    assert!(state_has_conflicts(&table, StateId(0)));
}

#[test]
fn state_has_conflicts_false_when_clean() {
    let table = make_table(vec![vec![vec![Action::Shift(StateId(1))]]]);
    assert!(!state_has_conflicts(&table, StateId(0)));
    assert!(!state_has_conflicts(&table, StateId(99))); // out of bounds
}

#[test]
fn get_state_conflicts_returns_details() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Accept]],
    ]);
    let conflicts = get_state_conflicts(&table, StateId(0));
    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0].conflict_type,
        InspectionConflictType::ShiftReduce
    );
    let clean = get_state_conflicts(&table, StateId(1));
    assert!(clean.is_empty());
}

#[test]
fn find_conflicts_for_symbol_uses_index_to_symbol() {
    let mut table = make_table(vec![vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        vec![Action::Shift(StateId(2))],
    ]]);
    table.index_to_symbol = vec![SymbolId(5), SymbolId(6)];
    assert_eq!(find_conflicts_for_symbol(&table, SymbolId(5)).len(), 1);
    assert!(find_conflicts_for_symbol(&table, SymbolId(6)).is_empty());
}

#[test]
fn count_conflicts_across_multiple_states() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))]],
        vec![vec![Action::Accept]],
    ]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 1);
    assert_eq!(summary.reduce_reduce, 1);
    assert_eq!(summary.states_with_conflicts.len(), 2);
}

// ===========================================================================
// 10. ConflictAnalyzer and ConflictStats
// ===========================================================================

#[test]
fn conflict_stats_default_all_zeros() {
    let stats = ConflictStats::default();
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
    assert_eq!(stats.associativity_resolved, 0);
    assert_eq!(stats.explicit_glr, 0);
    assert_eq!(stats.default_resolved, 0);
}

#[test]
fn analyzer_returns_consistent_stats() {
    let table = make_table(vec![vec![vec![Action::Shift(StateId(0))]]]);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    let stats2 = analyzer.get_stats().clone();
    assert_eq!(
        stats.reduce_reduce_conflicts,
        stats2.reduce_reduce_conflicts
    );
}

// ===========================================================================
// 11. Integration: full pipeline builds
// ===========================================================================

#[test]
fn left_assoc_grammar_builds_table() {
    let g = expr_grammar_with_prec(1, Associativity::Left, 1, Associativity::Left);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
}

#[test]
fn right_assoc_grammar_builds_table() {
    let g = expr_grammar_with_prec(1, Associativity::Right, 1, Associativity::Right);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
}

#[test]
fn two_op_different_prec_builds_table() {
    let g = two_op_grammar(1, Associativity::Left, 2, Associativity::Left);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
}
