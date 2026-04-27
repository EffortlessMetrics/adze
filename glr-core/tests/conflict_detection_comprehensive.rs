#![allow(clippy::needless_range_loop, unused_imports, clippy::clone_on_copy)]

//! Comprehensive conflict detection and resolution tests for GLR core.
//!
//! Covers: shift-reduce detection, reduce-reduce detection, precedence-based
//! resolution, associativity-based resolution, GLR fork detection,
//! action cell inspection, ParseTable properties, and edge cases.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_detection_comprehensive

use adze_glr_core::advanced_conflict::{ConflictAnalyzer, PrecedenceDecision, PrecedenceResolver};
use adze_glr_core::conflict_inspection::{
    ConflictType, action_cell_has_conflict, classify_conflict, count_conflicts,
    find_conflicts_for_symbol, get_state_conflicts, state_has_conflicts,
};
use adze_glr_core::precedence_compare::{
    PrecedenceComparison, PrecedenceInfo, StaticPrecedenceResolver, compare_precedences,
};
use adze_glr_core::{
    Action, Conflict, ConflictResolver, FirstFollowSets, GotoIndexing, LexMode, ParseTable,
    build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
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

/// Build a single-operator expression grammar: expr → expr op expr | num.
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
                precedence: prec,
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

/// Build a two-operator expression grammar: expr → expr + expr | expr * expr | num.
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

/// Build a simple grammar with no conflicts: s → a | b
fn simple_no_conflict() -> Grammar {
    let mut g = Grammar::new("no_conflict".into());
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
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(b)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );
    g
}

/// Build dangling-else grammar: s → if e then s | if e then s else s | a
fn dangling_else() -> Grammar {
    let mut g = Grammar::new("dangling_else".into());
    let _if = SymbolId(1);
    let _then = SymbolId(2);
    let _else = SymbolId(3);
    let e = SymbolId(4);
    let a = SymbolId(5);
    let s = SymbolId(10);

    g.tokens.insert(
        _if,
        Token {
            name: "if".into(),
            pattern: TokenPattern::String("if".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        _then,
        Token {
            name: "then".into(),
            pattern: TokenPattern::String("then".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        _else,
        Token {
            name: "else".into(),
            pattern: TokenPattern::String("else".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        e,
        Token {
            name: "e".into(),
            pattern: TokenPattern::String("e".into()),
            fragile: false,
        },
    );
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
        vec![
            Rule {
                lhs: s,
                rhs: vec![
                    Symbol::Terminal(_if),
                    Symbol::Terminal(e),
                    Symbol::Terminal(_then),
                    Symbol::NonTerminal(s),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: s,
                rhs: vec![
                    Symbol::Terminal(_if),
                    Symbol::Terminal(e),
                    Symbol::Terminal(_then),
                    Symbol::NonTerminal(s),
                    Symbol::Terminal(_else),
                    Symbol::NonTerminal(s),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(a)],
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
// §1  Shift-reduce conflict detection
// ===========================================================================

/// A conflict-free grammar (S → a | b) must produce zero conflicts.
#[test]
fn test_01_simple_no_conflict_grammar() {
    let g = simple_no_conflict();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "no S/R conflicts expected");
    assert_eq!(summary.reduce_reduce, 0, "no R/R conflicts expected");
}

/// The classic dangling-else grammar must produce at least one S/R conflict.
#[test]
fn test_02_dangling_else_shift_reduce() {
    let g = dangling_else();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0,
        "Dangling else grammar must have shift/reduce conflict"
    );
}

/// An ambiguous E → E op E | num (no prec/assoc) must produce S/R conflicts.
#[test]
fn test_03_ambiguous_single_op_has_sr_conflict() {
    let g = expr_one_op(None, None);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0,
        "Unresolved S/R conflict should be preserved"
    );
}

/// classify_conflict correctly identifies a Shift+Reduce cell.
#[test]
fn test_04_classify_shift_reduce_cell() {
    let cell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))];
    assert_eq!(classify_conflict(&cell), ConflictType::ShiftReduce);
}

/// The dangling-else conflict specifically appears in the states_with_conflicts list.
#[test]
fn test_05_dangling_else_conflict_states_nonempty() {
    let g = dangling_else();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        !summary.states_with_conflicts.is_empty(),
        "Must report at least one conflicting state"
    );
}

// ===========================================================================
// §2  Reduce-reduce conflict detection
// ===========================================================================

/// classify_conflict correctly identifies a Reduce+Reduce cell.
#[test]
fn test_06_classify_reduce_reduce_cell() {
    let cell = vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(5))];
    assert_eq!(classify_conflict(&cell), ConflictType::ReduceReduce);
}

/// A synthetic table with only R/R conflicts should count them correctly.
#[test]
fn test_07_count_reduce_reduce_only() {
    let table = make_table(vec![vec![vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.reduce_reduce, 1);
    assert_eq!(summary.shift_reduce, 0);
}

/// A synthetic table with multiple Reduce actions demonstrates R/R conflict
/// detection on a real ParseTable object queried via conflict_inspection.
#[test]
fn test_08_synthetic_rr_conflict_in_table() {
    let table = make_table(vec![
        vec![vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))]],
        vec![vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(3))]],
    ]);
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.reduce_reduce, 2,
        "Both states should have R/R conflicts"
    );
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.states_with_conflicts.len(), 2);
}

// ===========================================================================
// §3  Precedence resolution
// ===========================================================================

/// PrecedenceResolver can be constructed from a grammar with precedence info.
#[test]
fn test_09_precedence_resolver_creation() {
    let mut g = expr_one_op(Some(PrecedenceKind::Static(1)), Some(Associativity::Left));
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });
    let resolver = PrecedenceResolver::new(&g);
    let _ = resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10));
}

/// Higher shift-precedence beats lower reduce-precedence → PreferShift.
#[test]
fn test_10_higher_shift_prec_wins() {
    let shift_prec = Some(PrecedenceInfo {
        level: 5,
        associativity: Associativity::Left,
        is_fragile: false,
    });
    let reduce_prec = Some(PrecedenceInfo {
        level: 3,
        associativity: Associativity::Left,
        is_fragile: false,
    });
    assert_eq!(
        compare_precedences(shift_prec, reduce_prec),
        PrecedenceComparison::PreferShift,
    );
}

/// Higher reduce-precedence beats lower shift-precedence → PreferReduce.
#[test]
fn test_11_higher_reduce_prec_wins() {
    let shift_prec = Some(PrecedenceInfo {
        level: 2,
        associativity: Associativity::Left,
        is_fragile: false,
    });
    let reduce_prec = Some(PrecedenceInfo {
        level: 4,
        associativity: Associativity::Left,
        is_fragile: false,
    });
    assert_eq!(
        compare_precedences(shift_prec, reduce_prec),
        PrecedenceComparison::PreferReduce,
    );
}

/// When no precedence info is available, compare_precedences returns None.
#[test]
fn test_12_no_prec_returns_none() {
    assert_eq!(compare_precedences(None, None), PrecedenceComparison::None,);
}

/// StaticPrecedenceResolver extracts per-rule precedence from grammar rules.
#[test]
fn test_13_static_resolver_extracts_rule_prec() {
    let g = expr_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(2)),
        Some(Associativity::Left),
    );
    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    // RuleId(0) = plus rule (prec 1), RuleId(1) = times rule (prec 2)
    let plus_prec = resolver.rule_precedence(RuleId(0));
    let times_prec = resolver.rule_precedence(RuleId(1));
    assert_eq!(plus_prec.map(|p| p.level), Some(1));
    assert_eq!(times_prec.map(|p| p.level), Some(2));
}

// ===========================================================================
// §4  Associativity resolution
// ===========================================================================

/// Left associativity at equal precedence favors reduce.
#[test]
fn test_14_left_assoc_favors_reduce() {
    let mut g = expr_one_op(Some(PrecedenceKind::Static(1)), Some(Associativity::Left));
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });
    let resolver = PrecedenceResolver::new(&g);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferReduce),
    );
}

/// Right associativity at equal precedence favors shift.
#[test]
fn test_15_right_assoc_favors_shift() {
    let mut g = expr_one_op(Some(PrecedenceKind::Static(1)), Some(Associativity::Right));
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)],
    });
    let resolver = PrecedenceResolver::new(&g);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(10)),
        Some(PrecedenceDecision::PreferShift),
    );
}

/// Non-associativity at equal precedence produces Error.
#[test]
fn test_16_none_assoc_produces_error() {
    let shift_prec = Some(PrecedenceInfo {
        level: 3,
        associativity: Associativity::None,
        is_fragile: false,
    });
    let reduce_prec = Some(PrecedenceInfo {
        level: 3,
        associativity: Associativity::None,
        is_fragile: false,
    });
    assert_eq!(
        compare_precedences(shift_prec, reduce_prec),
        PrecedenceComparison::Error,
    );
}

/// Left-assoc compare_precedences at same level.
#[test]
fn test_17_compare_prec_left_assoc_same_level() {
    let info = Some(PrecedenceInfo {
        level: 2,
        associativity: Associativity::Left,
        is_fragile: false,
    });
    assert_eq!(
        compare_precedences(info, info),
        PrecedenceComparison::PreferReduce,
    );
}

/// Right-assoc compare_precedences at same level.
#[test]
fn test_18_compare_prec_right_assoc_same_level() {
    let info = Some(PrecedenceInfo {
        level: 2,
        associativity: Associativity::Right,
        is_fragile: false,
    });
    assert_eq!(
        compare_precedences(info, info),
        PrecedenceComparison::PreferShift,
    );
}

// ===========================================================================
// §5  GLR fork detection
// ===========================================================================

/// classify_conflict unwraps Fork(Shift, Reduce) into ShiftReduce.
#[test]
fn test_19_fork_classified_as_shift_reduce() {
    let cell = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ])];
    assert_eq!(classify_conflict(&cell), ConflictType::ShiftReduce);
}

/// Fork(Reduce, Reduce) is classified as ReduceReduce.
#[test]
fn test_20_fork_classified_as_reduce_reduce() {
    let cell = vec![Action::Fork(vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ])];
    assert_eq!(classify_conflict(&cell), ConflictType::ReduceReduce);
}

/// An ambiguous grammar built via GrammarBuilder should produce conflicted
/// action cells (indicating GLR forking points).
#[test]
fn test_21_grammarbuilder_ambiguous_has_multi_action_cells() {
    let g = GrammarBuilder::new("ambig")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let has_conflicted_cell = table
        .action_table
        .iter()
        .any(|row| row.iter().any(|cell| action_cell_has_conflict(cell)));
    assert!(
        has_conflicted_cell,
        "Ambiguous grammar must have at least one conflicted action cell (GLR fork)"
    );
}

/// An unambiguous grammar built via GrammarBuilder should have no conflicted cells.
#[test]
fn test_22_grammarbuilder_unambiguous_no_multi_action() {
    let g = GrammarBuilder::new("unamb")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let has_conflicted_cell = table
        .action_table
        .iter()
        .any(|row| row.iter().any(|cell| action_cell_has_conflict(cell)));
    assert!(
        !has_conflicted_cell,
        "Unambiguous grammar must have no conflicted action cells"
    );
}

/// A four-operator arithmetic grammar without precedence has many fork points.
#[test]
fn test_23_four_op_grammar_many_conflicts() {
    let mut g = Grammar::new("four_ops".into());
    let num = SymbolId(1);
    let plus = SymbolId(2);
    let minus = SymbolId(3);
    let times = SymbolId(4);
    let div = SymbolId(5);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    for (sid, name) in &[(plus, "+"), (minus, "-"), (times, "*"), (div, "/")] {
        g.tokens.insert(
            *sid,
            Token {
                name: name.to_string(),
                pattern: TokenPattern::String(name.to_string()),
                fragile: false,
            },
        );
    }
    g.rule_names.insert(e, "E".into());

    let mut rules = vec![];
    for (i, op) in &[(0u16, plus), (1u16, minus), (2u16, times), (3u16, div)] {
        rules.push(Rule {
            lhs: e,
            rhs: vec![
                Symbol::NonTerminal(e),
                Symbol::Terminal(*op),
                Symbol::NonTerminal(e),
            ],
            precedence: None,
            associativity: None,
            production_id: ProductionId(*i),
            fields: vec![],
        });
    }
    rules.push(Rule {
        lhs: e,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(4u16),
        fields: vec![],
    });
    g.rules.insert(e, rules);

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.conflict_details.len() > 1,
        "Four-operator grammar should have many conflicts"
    );
}

// ===========================================================================
// §6  ParseTable properties for conflict-free grammars
// ===========================================================================

/// A conflict-free grammar's parse table must have state_count > 0.
#[test]
fn test_24_conflict_free_table_has_states() {
    let g = simple_no_conflict();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
}

/// An optional-rule grammar (opt → a | ε; s → opt) has no conflicts.
#[test]
fn test_25_optional_grammar_no_conflicts() {
    let mut g = Grammar::new("optional".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    let opt = SymbolId(11);

    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(opt, "Opt".into());

    g.rules.insert(
        opt,
        vec![
            Rule {
                lhs: opt,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: opt,
                rhs: vec![],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );

    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(opt)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(2),
            fields: vec![],
        }],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.conflict_details.len(), 0);
}

/// A left-recursive list grammar (list → a | list a) has no conflicts.
#[test]
fn test_26_left_recursive_list_no_conflicts() {
    let mut g = Grammar::new("repeat".into());
    let a = SymbolId(1);
    let list = SymbolId(10);

    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(list, "List".into());

    g.rules.insert(
        list,
        vec![
            Rule {
                lhs: list,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: list,
                rhs: vec![Symbol::NonTerminal(list), Symbol::Terminal(a)],
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
    assert_eq!(summary.conflict_details.len(), 0);
}

/// An epsilon-aware grammar (s → b opt; opt → a | ε) compiles without conflict.
#[test]
fn test_27_epsilon_production_no_conflicts() {
    let mut g = Grammar::new("epsilon".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
    let opt = SymbolId(11);

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
    g.rule_names.insert(opt, "Opt".into());

    g.rules.insert(
        opt,
        vec![
            Rule {
                lhs: opt,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: opt,
                rhs: vec![],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );

    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(b), Symbol::NonTerminal(opt)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(2),
            fields: vec![],
        }],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
}

/// A two-operator grammar with full precedence can still be built and has states.
#[test]
fn test_28_two_op_with_prec_builds_table() {
    let g = expr_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(2)),
        Some(Associativity::Left),
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
}

// ===========================================================================
// §7  Action cell inspection and classification
// ===========================================================================

/// A synthetic table with S/R + R/R in separate states reports both types.
#[test]
fn test_29_mixed_conflict_types_across_states() {
    let table = make_table(vec![
        vec![
            vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
            vec![],
        ],
        vec![
            vec![],
            vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(3))],
        ],
    ]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 1);
    assert_eq!(summary.reduce_reduce, 1);
}

/// ConflictDetail stores the correct state ID.
#[test]
fn test_30_conflict_detail_state_id() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(5)),
        Action::Reduce(RuleId(2)),
    ]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.conflict_details.len(), 1);
    assert_eq!(summary.conflict_details[0].state, StateId(0));
    assert!(matches!(
        summary.conflict_details[0].conflict_type,
        ConflictType::ShiftReduce
    ));
    assert_eq!(summary.conflict_details[0].actions.len(), 2);
}

/// state_has_conflicts returns true only for states with multi-action cells.
#[test]
fn test_31_state_has_conflicts_query() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Shift(StateId(2))]],
    ]);
    assert!(state_has_conflicts(&table, StateId(0)));
    assert!(!state_has_conflicts(&table, StateId(1)));
}

/// state_has_conflicts returns false for out-of-bounds state IDs.
#[test]
fn test_32_state_has_conflicts_oob() {
    let table = make_table(vec![vec![vec![Action::Shift(StateId(1))]]]);
    assert!(!state_has_conflicts(&table, StateId(99)));
}

/// get_state_conflicts returns only conflicts for the requested state.
#[test]
fn test_33_get_state_conflicts_filters_correctly() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(3))]],
    ]);
    let s0 = get_state_conflicts(&table, StateId(0));
    let s1 = get_state_conflicts(&table, StateId(1));
    assert_eq!(s0.len(), 1);
    assert_eq!(s0[0].state, StateId(0));
    assert_eq!(s1.len(), 1);
    assert_eq!(s1[0].state, StateId(1));
}

/// find_conflicts_for_symbol locates conflicts on a specific lookahead.
#[test]
fn test_34_find_conflicts_for_symbol() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    let conflicts = find_conflicts_for_symbol(&table, SymbolId(0));
    assert!(!conflicts.is_empty());
}

/// ConflictAnalyzer can analyze a table and return stats (may be zero for
/// simple synthetic tables that lack full grammar context).
#[test]
fn test_35_conflict_analyzer_runs() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    // ConflictAnalyzer works on full tables; synthetic tables have zero counts.
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    // get_stats returns the same data.
    let cached = analyzer.get_stats();
    assert_eq!(cached.shift_reduce_conflicts, stats.shift_reduce_conflicts);
}

/// Conflict actions stored in details always have ≥ 2 entries.
#[test]
fn test_36_conflict_actions_always_multiple() {
    let table = make_table(vec![vec![
        vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))],
    ]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.conflict_details.len(), 2);
    for detail in &summary.conflict_details {
        assert!(detail.actions.len() >= 2);
    }
}

/// All conflict state IDs must be within [0, state_count).
#[test]
fn test_37_conflict_state_ids_in_range() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
        vec![vec![Action::Reduce(RuleId(2)), Action::Reduce(RuleId(3))]],
    ]);
    let summary = count_conflicts(&table);
    for detail in &summary.conflict_details {
        assert!((detail.state.0 as usize) < table.state_count);
    }
}

/// A single-state table with only empty cells produces zero conflicts.
#[test]
fn test_38_all_empty_cells_no_conflicts() {
    let table = make_table(vec![vec![vec![], vec![]]]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.conflict_details.len(), 0);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

/// A table where every cell has exactly one action is conflict-free.
#[test]
fn test_39_single_action_cells_no_conflicts() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Accept]],
        vec![
            vec![Action::Reduce(RuleId(0))],
            vec![Action::Reduce(RuleId(0))],
        ],
    ]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.conflict_details.len(), 0);
}

/// GrammarBuilder-based grammar with precedence builds without panics.
#[test]
fn test_40_grammarbuilder_with_precedence_builds() {
    let g = GrammarBuilder::new("prec_test")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0, "Table with precedence must build");
}
