#![allow(clippy::needless_range_loop)]
//! Property-based tests for conflict resolution in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test conflict_resolution_proptest`

use adze_glr_core::advanced_conflict::{
    ConflictAnalyzer, ConflictStats, PrecedenceDecision, PrecedenceResolver,
};
use adze_glr_core::conflict_inspection::{
    ConflictSummary, ConflictType, classify_conflict, count_conflicts,
};
use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::{
    Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, StateId,
    Symbol, SymbolId, Token, TokenPattern,
};
use proptest::prelude::*;
use std::collections::{BTreeMap, HashSet};

// ============================================================================
// Strategies
// ============================================================================

/// Generate a leaf Action (no Fork).
fn leaf_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..500u16).prop_map(|s| Action::Shift(StateId(s))),
        (0..500u16).prop_map(|r| Action::Reduce(RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Generate a valid Associativity value.
fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

/// Generate a precedence level in [1..20].
fn arb_prec_level() -> impl Strategy<Value = i16> {
    1i16..=20
}

// ============================================================================
// Helpers
// ============================================================================

/// Build a simple expression grammar: E → E op E | num
/// with configurable precedence/associativity on the binary rule.
fn expr_grammar_one_op(prec: Option<PrecedenceKind>, assoc: Option<Associativity>) -> Grammar {
    let mut g = Grammar::new("expr_one_op".to_string());
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

/// Build a two-operator expression grammar: E → E + E | E * E | num
fn expr_grammar_two_ops(
    plus_prec: Option<PrecedenceKind>,
    plus_assoc: Option<Associativity>,
    times_prec: Option<PrecedenceKind>,
    times_assoc: Option<Associativity>,
) -> Grammar {
    let mut g = Grammar::new("expr_two_ops".to_string());
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

/// Build a reduce/reduce grammar: S → A | B; A → a; B → a
fn rr_grammar() -> Grammar {
    let mut g = Grammar::new("rr_conflict".to_string());
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
    g
}

/// Count cells with multiple actions in a parse table.
fn count_fork_cells(table: &ParseTable) -> usize {
    let mut n = 0;
    for state in 0..table.state_count {
        for sym in 0..table.action_table[state].len() {
            if table.action_table[state][sym].len() > 1 {
                n += 1;
            }
        }
    }
    n
}

/// Check if there is a cell containing both Shift and Reduce.
fn has_shift_reduce(table: &ParseTable) -> bool {
    for state in &table.action_table {
        for cell in state {
            if cell.len() > 1
                && cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
            {
                return true;
            }
        }
    }
    false
}

/// Create a minimal ParseTable for unit-level conflict testing.
fn make_test_table(action_table: Vec<Vec<Vec<Action>>>) -> ParseTable {
    let state_count = action_table.len();
    ParseTable {
        action_table,
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count,
        symbol_count: 1,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
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

// ============================================================================
// Helper: Build ambiguous grammar E → a | E E
// ============================================================================
fn ambiguous_concat_grammar() -> Grammar {
    let mut g = Grammar::new("ambig_concat".to_string());
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
    g
}

/// Helper: Check if any cell contains only Reduce actions with count > 1.
fn has_reduce_reduce(table: &ParseTable) -> bool {
    for state in &table.action_table {
        for cell in state {
            if cell.len() > 1 && cell.iter().all(|a| matches!(a, Action::Reduce(_))) {
                return true;
            }
        }
    }
    false
}

/// Helper: Collect all ConflictType values from a ConflictSummary.
fn conflict_types_in_summary(summary: &ConflictSummary) -> Vec<ConflictType> {
    let mut types: Vec<ConflictType> = summary
        .conflict_details
        .iter()
        .map(|d| d.conflict_type)
        .collect();
    types.dedup();
    types
}

// ============================================================================
// 1. classify_conflict: Shift + Reduce → ShiftReduce
// ============================================================================
proptest! {
    #[test]
    fn classify_shift_reduce_always(s in 0..500u16, r in 0..500u16) {
        let actions = vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))];
        prop_assert_eq!(classify_conflict(&actions), ConflictType::ShiftReduce);
    }
}

// ============================================================================
// 2. classify_conflict: multiple Reduce → ReduceReduce
// ============================================================================
proptest! {
    #[test]
    fn classify_reduce_reduce_always(r1 in 0..500u16, r2 in 0..500u16) {
        prop_assume!(r1 != r2);
        let actions = vec![Action::Reduce(RuleId(r1)), Action::Reduce(RuleId(r2))];
        prop_assert_eq!(classify_conflict(&actions), ConflictType::ReduceReduce);
    }
}

// ============================================================================
// 3. classify_conflict: multiple Shift only → Mixed
// ============================================================================
proptest! {
    #[test]
    fn classify_shift_shift_is_mixed(s1 in 0..500u16, s2 in 0..500u16) {
        prop_assume!(s1 != s2);
        let actions = vec![Action::Shift(StateId(s1)), Action::Shift(StateId(s2))];
        prop_assert_eq!(classify_conflict(&actions), ConflictType::Mixed);
    }
}

// ============================================================================
// 4. classify_conflict: Fork wrapping S/R → ShiftReduce
// ============================================================================
proptest! {
    #[test]
    fn classify_fork_sr(s in 0..500u16, r in 0..500u16) {
        let actions = vec![Action::Fork(vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))])];
        prop_assert_eq!(classify_conflict(&actions), ConflictType::ShiftReduce);
    }
}

// ============================================================================
// 5. classify_conflict: Fork wrapping R/R → ReduceReduce
// ============================================================================
proptest! {
    #[test]
    fn classify_fork_rr(r1 in 0..500u16, r2 in 0..500u16) {
        prop_assume!(r1 != r2);
        let actions = vec![Action::Fork(vec![Action::Reduce(RuleId(r1)), Action::Reduce(RuleId(r2))])];
        prop_assert_eq!(classify_conflict(&actions), ConflictType::ReduceReduce);
    }
}

// ============================================================================
// 6. PrecedenceResolver: higher shift prec → PreferShift
// ============================================================================
proptest! {
    #[test]
    fn prec_higher_shift_prefers_shift(shift_level in 2i16..=20, reduce_level in 1i16..=19) {
        prop_assume!(shift_level > reduce_level);
        let mut grammar = Grammar::new("test".to_string());
        let shift_sym = SymbolId(1);
        let reduce_sym = SymbolId(10);

        grammar.precedences.push(Precedence {
            level: shift_level,
            associativity: Associativity::Left,
            symbols: vec![shift_sym],
        });
        grammar.rules.insert(reduce_sym, vec![Rule {
            lhs: reduce_sym, rhs: vec![Symbol::Terminal(shift_sym)],
            precedence: Some(PrecedenceKind::Static(reduce_level)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(0), fields: vec![],
        }]);

        let resolver = PrecedenceResolver::new(&grammar);
        let decision = resolver.can_resolve_shift_reduce(shift_sym, reduce_sym);
        prop_assert_eq!(decision, Some(PrecedenceDecision::PreferShift));
    }
}

// ============================================================================
// 7. PrecedenceResolver: higher reduce prec → PreferReduce
// ============================================================================
proptest! {
    #[test]
    fn prec_higher_reduce_prefers_reduce(shift_level in 1i16..=19, reduce_level in 2i16..=20) {
        prop_assume!(reduce_level > shift_level);
        let mut grammar = Grammar::new("test".to_string());
        let shift_sym = SymbolId(1);
        let reduce_sym = SymbolId(10);

        grammar.precedences.push(Precedence {
            level: shift_level,
            associativity: Associativity::Left,
            symbols: vec![shift_sym],
        });
        grammar.rules.insert(reduce_sym, vec![Rule {
            lhs: reduce_sym, rhs: vec![Symbol::Terminal(shift_sym)],
            precedence: Some(PrecedenceKind::Static(reduce_level)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(0), fields: vec![],
        }]);

        let resolver = PrecedenceResolver::new(&grammar);
        let decision = resolver.can_resolve_shift_reduce(shift_sym, reduce_sym);
        prop_assert_eq!(decision, Some(PrecedenceDecision::PreferReduce));
    }
}

// ============================================================================
// 8. PrecedenceResolver: same prec + left-assoc → PreferReduce
// ============================================================================
proptest! {
    #[test]
    fn prec_same_level_left_assoc_prefers_reduce(level in arb_prec_level()) {
        let mut grammar = Grammar::new("test".to_string());
        let shift_sym = SymbolId(1);
        let reduce_sym = SymbolId(10);

        grammar.precedences.push(Precedence {
            level, associativity: Associativity::Left, symbols: vec![shift_sym],
        });
        grammar.rules.insert(reduce_sym, vec![Rule {
            lhs: reduce_sym, rhs: vec![Symbol::Terminal(shift_sym)],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(0), fields: vec![],
        }]);

        let resolver = PrecedenceResolver::new(&grammar);
        prop_assert_eq!(
            resolver.can_resolve_shift_reduce(shift_sym, reduce_sym),
            Some(PrecedenceDecision::PreferReduce)
        );
    }
}

// ============================================================================
// 9. PrecedenceResolver: same prec + right-assoc → PreferShift
// ============================================================================
proptest! {
    #[test]
    fn prec_same_level_right_assoc_prefers_shift(level in arb_prec_level()) {
        let mut grammar = Grammar::new("test".to_string());
        let shift_sym = SymbolId(1);
        let reduce_sym = SymbolId(10);

        grammar.precedences.push(Precedence {
            level, associativity: Associativity::Right, symbols: vec![shift_sym],
        });
        grammar.rules.insert(reduce_sym, vec![Rule {
            lhs: reduce_sym, rhs: vec![Symbol::Terminal(shift_sym)],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: Some(Associativity::Right),
            production_id: ProductionId(0), fields: vec![],
        }]);

        let resolver = PrecedenceResolver::new(&grammar);
        prop_assert_eq!(
            resolver.can_resolve_shift_reduce(shift_sym, reduce_sym),
            Some(PrecedenceDecision::PreferShift)
        );
    }
}

// ============================================================================
// 10. PrecedenceResolver: same prec + non-assoc → Error
// ============================================================================
proptest! {
    #[test]
    fn prec_same_level_none_assoc_is_error(level in arb_prec_level()) {
        let mut grammar = Grammar::new("test".to_string());
        let shift_sym = SymbolId(1);
        let reduce_sym = SymbolId(10);

        grammar.precedences.push(Precedence {
            level, associativity: Associativity::None, symbols: vec![shift_sym],
        });
        grammar.rules.insert(reduce_sym, vec![Rule {
            lhs: reduce_sym, rhs: vec![Symbol::Terminal(shift_sym)],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: Some(Associativity::None),
            production_id: ProductionId(0), fields: vec![],
        }]);

        let resolver = PrecedenceResolver::new(&grammar);
        prop_assert_eq!(
            resolver.can_resolve_shift_reduce(shift_sym, reduce_sym),
            Some(PrecedenceDecision::Error)
        );
    }
}

// ============================================================================
// 11. PrecedenceResolver: unknown symbols → None
// ============================================================================
proptest! {
    #[test]
    fn prec_unknown_symbols_returns_none(s in 100u16..200, r in 200u16..300) {
        let grammar = Grammar::new("empty".to_string());
        let resolver = PrecedenceResolver::new(&grammar);
        prop_assert_eq!(
            resolver.can_resolve_shift_reduce(SymbolId(s), SymbolId(r)),
            None
        );
    }
}

// ============================================================================
// 12. PrecedenceDecision: equality is reflexive
// ============================================================================
proptest! {
    #[test]
    fn prec_decision_eq_reflexive(idx in 0usize..3) {
        let decisions = [PrecedenceDecision::PreferShift, PrecedenceDecision::PreferReduce, PrecedenceDecision::Error];
        let d = &decisions[idx];
        prop_assert_eq!(d, d);
    }
}

// ============================================================================
// 13. ConflictAnalyzer: fresh analyzer has zero stats
// ============================================================================
proptest! {
    #[test]
    fn analyzer_fresh_zero_stats(_dummy in 0u8..1) {
        let analyzer = ConflictAnalyzer::new();
        let stats = analyzer.get_stats();
        prop_assert_eq!(stats.shift_reduce_conflicts, 0);
        prop_assert_eq!(stats.reduce_reduce_conflicts, 0);
        prop_assert_eq!(stats.precedence_resolved, 0);
        prop_assert_eq!(stats.associativity_resolved, 0);
        prop_assert_eq!(stats.explicit_glr, 0);
        prop_assert_eq!(stats.default_resolved, 0);
    }
}

// ============================================================================
// 14. ConflictStats: Default is all zeros
// ============================================================================
proptest! {
    #[test]
    fn conflict_stats_default_is_zero(_dummy in 0u8..1) {
        let stats = ConflictStats::default();
        let total = stats.shift_reduce_conflicts
            + stats.reduce_reduce_conflicts
            + stats.precedence_resolved
            + stats.associativity_resolved
            + stats.explicit_glr
            + stats.default_resolved;
        prop_assert_eq!(total, 0);
    }
}

// ============================================================================
// 15. count_conflicts: empty table → zero conflicts
// ============================================================================
proptest! {
    #[test]
    fn count_conflicts_empty_table(n_states in 1usize..5) {
        // Table with n_states but every cell has at most 1 action
        let action_table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|_| vec![vec![Action::Shift(StateId(0))]])
            .collect();
        let table = make_test_table(action_table);
        let summary = count_conflicts(&table);
        prop_assert_eq!(summary.shift_reduce, 0);
        prop_assert_eq!(summary.reduce_reduce, 0);
        prop_assert!(summary.states_with_conflicts.is_empty());
    }
}

// ============================================================================
// 16. count_conflicts: single S/R cell → exactly 1 S/R conflict
// ============================================================================
proptest! {
    #[test]
    fn count_conflicts_single_sr(s in 0..500u16, r in 0..500u16) {
        let table = make_test_table(vec![vec![
            vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))],
        ]]);
        let summary = count_conflicts(&table);
        prop_assert_eq!(summary.shift_reduce, 1);
        prop_assert_eq!(summary.reduce_reduce, 0);
        prop_assert_eq!(summary.states_with_conflicts.len(), 1);
    }
}

// ============================================================================
// 17. count_conflicts: single R/R cell → exactly 1 R/R conflict
// ============================================================================
proptest! {
    #[test]
    fn count_conflicts_single_rr(r1 in 0..500u16, r2 in 0..500u16) {
        prop_assume!(r1 != r2);
        let table = make_test_table(vec![vec![
            vec![Action::Reduce(RuleId(r1)), Action::Reduce(RuleId(r2))],
        ]]);
        let summary = count_conflicts(&table);
        prop_assert_eq!(summary.shift_reduce, 0);
        prop_assert_eq!(summary.reduce_reduce, 1);
        prop_assert_eq!(summary.states_with_conflicts.len(), 1);
    }
}

// ============================================================================
// 18. count_conflicts: states_with_conflicts has unique entries
// ============================================================================
proptest! {
    #[test]
    fn conflict_states_are_unique(n in 1usize..4) {
        // Build a table where every state has a S/R conflict
        let action_table: Vec<Vec<Vec<Action>>> = (0..n)
            .map(|_| vec![vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))]])
            .collect();
        let table = make_test_table(action_table);
        let summary = count_conflicts(&table);
        let unique: HashSet<_> = summary.states_with_conflicts.iter().copied().collect();
        prop_assert_eq!(unique.len(), summary.states_with_conflicts.len());
    }
}

// ============================================================================
// 19. count_conflicts: conflict_details len matches total S/R + R/R + Mixed
// ============================================================================
proptest! {
    #[test]
    fn conflict_details_len_matches_totals(
        has_sr in proptest::bool::ANY,
        has_rr in proptest::bool::ANY,
    ) {
        let mut cells = Vec::new();
        if has_sr {
            cells.push(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
        }
        if has_rr {
            cells.push(vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))]);
        }
        if cells.is_empty() {
            cells.push(vec![Action::Shift(StateId(0))]);
        }
        let table = make_test_table(vec![cells]);
        let summary = count_conflicts(&table);
        // Each multi-action cell produces exactly one ConflictDetail
        let expected_details = if has_sr as usize + has_rr as usize > 0 {
            has_sr as usize + has_rr as usize
        } else {
            0
        };
        prop_assert_eq!(summary.conflict_details.len(), expected_details);
    }
}

// ============================================================================
// 20. Multiple conflicts on same state: counts accumulate
// ============================================================================
proptest! {
    #[test]
    fn multiple_conflicts_same_state_accumulate(n_conflicts in 1usize..5) {
        let cells: Vec<Vec<Action>> = (0..n_conflicts)
            .map(|i| vec![Action::Shift(StateId(i as u16)), Action::Reduce(RuleId(i as u16))])
            .collect();
        let table = make_test_table(vec![cells]);
        let summary = count_conflicts(&table);
        prop_assert_eq!(summary.shift_reduce, n_conflicts);
        // All conflicts in one state
        prop_assert_eq!(summary.states_with_conflicts.len(), 1);
    }
}

// ============================================================================
// 21. Conflict count stability: building the same grammar twice yields same count
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn conflict_count_stable_across_builds(level in 1i16..=5) {
        let g1 = expr_grammar_one_op(Some(PrecedenceKind::Static(level)), Some(Associativity::Left));
        let ff1 = FirstFollowSets::compute(&g1).unwrap();
        let t1 = build_lr1_automaton(&g1, &ff1).unwrap();
        let s1 = count_conflicts(&t1);

        let g2 = expr_grammar_one_op(Some(PrecedenceKind::Static(level)), Some(Associativity::Left));
        let ff2 = FirstFollowSets::compute(&g2).unwrap();
        let t2 = build_lr1_automaton(&g2, &ff2).unwrap();
        let s2 = count_conflicts(&t2);

        prop_assert_eq!(s1.shift_reduce, s2.shift_reduce);
        prop_assert_eq!(s1.reduce_reduce, s2.reduce_reduce);
        prop_assert_eq!(s1.states_with_conflicts.len(), s2.states_with_conflicts.len());
    }
}

// ============================================================================
// 22. Left-assoc: no S/R conflicts remain for any precedence level
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn left_assoc_resolves_all_sr(level in 1i16..=10) {
        let g = expr_grammar_one_op(Some(PrecedenceKind::Static(level)), Some(Associativity::Left));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        prop_assert!(!has_shift_reduce(&table), "Left-assoc should resolve all S/R");
    }
}

// ============================================================================
// 23. Right-assoc: no S/R conflicts remain for any precedence level
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn right_assoc_resolves_all_sr(level in 1i16..=10) {
        let g = expr_grammar_one_op(Some(PrecedenceKind::Static(level)), Some(Associativity::Right));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        prop_assert!(!has_shift_reduce(&table), "Right-assoc should resolve all S/R");
    }
}

// ============================================================================
// 24. No precedence: S/R conflict always present for E → E op E grammar
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]
    #[test]
    fn no_prec_always_has_sr(_dummy in 0u8..1) {
        let g = expr_grammar_one_op(None, None);
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        prop_assert!(has_shift_reduce(&table), "Grammar without prec should have S/R conflict");
    }
}

// ============================================================================
// 25. Non-assoc: conflict preserved (fork cells remain)
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn non_assoc_preserves_conflict(level in 1i16..=10) {
        let g = expr_grammar_one_op(Some(PrecedenceKind::Static(level)), Some(Associativity::None));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        let forks = count_fork_cells(&table);
        prop_assert!(forks > 0, "Non-assoc should preserve fork cells (found {})", forks);
    }
}

// ============================================================================
// 26. Two-op grammar: higher prec resolves cross-operator conflicts
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn two_ops_higher_prec_resolves(low in 1i16..=5, high in 6i16..=10) {
        let g = expr_grammar_two_ops(
            Some(PrecedenceKind::Static(low)),  Some(Associativity::Left),
            Some(PrecedenceKind::Static(high)), Some(Associativity::Left),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        let summary = count_conflicts(&table);
        prop_assert_eq!(summary.shift_reduce, 0, "Cross-op S/R should be resolved");
    }
}

// ============================================================================
// 27. R/R grammar always builds successfully
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]
    #[test]
    fn rr_grammar_builds_ok(_dummy in 0u8..1) {
        let g = rr_grammar();
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        prop_assert!(table.state_count > 0, "R/R grammar should produce a valid table");
    }
}

// ============================================================================
// 28. ConflictSummary Display contains S/R and R/R counts
// ============================================================================
proptest! {
    #[test]
    fn summary_display_contains_counts(sr in 0usize..5, rr in 0usize..5) {
        let mut cells: Vec<Vec<Action>> = Vec::new();
        for i in 0..sr {
            cells.push(vec![Action::Shift(StateId(i as u16)), Action::Reduce(RuleId(i as u16))]);
        }
        for i in 0..rr {
            cells.push(vec![Action::Reduce(RuleId(100 + i as u16)), Action::Reduce(RuleId(200 + i as u16))]);
        }
        if cells.is_empty() {
            cells.push(vec![Action::Shift(StateId(0))]);
        }
        let table = make_test_table(vec![cells]);
        let summary = count_conflicts(&table);
        let display = format!("{}", summary);
        let sr_str = format!("Shift/Reduce conflicts: {}", summary.shift_reduce);
        let rr_str = format!("Reduce/Reduce conflicts: {}", summary.reduce_reduce);
        prop_assert!(display.contains(&sr_str), "Display missing S/R count");
        prop_assert!(display.contains(&rr_str), "Display missing R/R count");
    }
}

// ============================================================================
// 29. classify_conflict: Accept/Error/Recover alone → Mixed
// ============================================================================
proptest! {
    #[test]
    fn classify_non_shift_reduce_is_mixed(idx in 0usize..3) {
        let sentinel = [Action::Accept, Action::Error, Action::Recover];
        let actions = vec![sentinel[idx].clone(), sentinel[(idx + 1) % 3].clone()];
        prop_assert_eq!(classify_conflict(&actions), ConflictType::Mixed);
    }
}

// ============================================================================
// 30. PrecedenceResolver: associativity determines tie-break direction
// ============================================================================
proptest! {
    #[test]
    fn assoc_determines_tiebreak(level in arb_prec_level(), assoc in arb_associativity()) {
        let mut grammar = Grammar::new("test".to_string());
        let shift_sym = SymbolId(1);
        let reduce_sym = SymbolId(10);

        grammar.precedences.push(Precedence {
            level, associativity: assoc, symbols: vec![shift_sym],
        });
        grammar.rules.insert(reduce_sym, vec![Rule {
            lhs: reduce_sym, rhs: vec![Symbol::Terminal(shift_sym)],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: Some(assoc),
            production_id: ProductionId(0), fields: vec![],
        }]);

        let resolver = PrecedenceResolver::new(&grammar);
        let decision = resolver.can_resolve_shift_reduce(shift_sym, reduce_sym);
        let expected = match assoc {
            Associativity::Left => Some(PrecedenceDecision::PreferReduce),
            Associativity::Right => Some(PrecedenceDecision::PreferShift),
            Associativity::None => Some(PrecedenceDecision::Error),
        };
        prop_assert_eq!(decision, expected);
    }
}

// ============================================================================
// 31. Two-op grammar without prec: fork cells ≥ 2
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]
    #[test]
    fn two_ops_no_prec_multiple_forks(_dummy in 0u8..1) {
        let g = expr_grammar_two_ops(None, None, None, None);
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        let forks = count_fork_cells(&table);
        prop_assert!(forks >= 2, "Two-op no-prec grammar should have ≥2 fork cells, got {}", forks);
    }
}

// ============================================================================
// 32. State count is always positive after successful build
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn table_state_count_positive(
        prec in prop::option::of(arb_prec_level().prop_map(PrecedenceKind::Static)),
        assoc in prop::option::of(arb_associativity()),
    ) {
        let g = expr_grammar_one_op(prec, assoc);
        let ff = FirstFollowSets::compute(&g).unwrap();
        let table = build_lr1_automaton(&g, &ff).unwrap();
        prop_assert!(table.state_count > 0);
        prop_assert_eq!(table.state_count, table.action_table.len());
    }
}

// ============================================================================
// 33. classify_conflict: single action is never ShiftReduce
// ============================================================================
proptest! {
    #[test]
    fn classify_single_action_never_shift_reduce(action in leaf_action()) {
        let ty = classify_conflict(&[action]);
        // A single action cannot have both Shift and Reduce simultaneously
        prop_assert!(ty != ConflictType::ShiftReduce,
            "Single action should never classify as ShiftReduce");
    }
}

// ============================================================================
// 34. Precedence resolution is deterministic (same input → same output)
// ============================================================================
proptest! {
    #[test]
    fn prec_resolution_deterministic(
        level_shift in arb_prec_level(),
        level_reduce in arb_prec_level(),
        assoc in arb_associativity(),
    ) {
        let mut grammar = Grammar::new("test".to_string());
        let shift_sym = SymbolId(1);
        let reduce_sym = SymbolId(10);

        grammar.precedences.push(Precedence {
            level: level_shift, associativity: assoc, symbols: vec![shift_sym],
        });
        grammar.rules.insert(reduce_sym, vec![Rule {
            lhs: reduce_sym, rhs: vec![Symbol::Terminal(shift_sym)],
            precedence: Some(PrecedenceKind::Static(level_reduce)),
            associativity: Some(assoc),
            production_id: ProductionId(0), fields: vec![],
        }]);

        let r1 = PrecedenceResolver::new(&grammar);
        let r2 = PrecedenceResolver::new(&grammar);
        let d1 = r1.can_resolve_shift_reduce(shift_sym, reduce_sym);
        let d2 = r2.can_resolve_shift_reduce(shift_sym, reduce_sym);
        prop_assert_eq!(d1, d2, "Same grammar should yield same decision");
    }
}

// ============================================================================
// 35. ConflictSummary: states_with_conflicts ⊆ [0..state_count)
// ============================================================================
proptest! {
    #[test]
    fn conflict_states_within_bounds(n_states in 1usize..6) {
        let action_table: Vec<Vec<Vec<Action>>> = (0..n_states)
            .map(|i| {
                if i % 2 == 0 {
                    vec![vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))]]
                } else {
                    vec![vec![Action::Shift(StateId(0))]]
                }
            })
            .collect();
        let table = make_test_table(action_table);
        let summary = count_conflicts(&table);
        for state in &summary.states_with_conflicts {
            prop_assert!((state.0 as usize) < n_states,
                "State {} out of bounds (state_count={})", state.0, n_states);
        }
    }
}

// ============================================================================
// 36. state_has_conflicts returns false for out-of-bounds state
// ============================================================================
proptest! {
    #[test]
    fn state_has_conflicts_oob_returns_false(extra in 1u16..100) {
        use adze_glr_core::conflict_inspection::state_has_conflicts;
        let table = make_test_table(vec![vec![vec![Action::Shift(StateId(0))]]]);
        let oob = StateId(table.state_count as u16 + extra);
        prop_assert!(!state_has_conflicts(&table, oob));
    }
}

// ============================================================================
// 37. state_has_conflicts true iff cell.len() > 1
// ============================================================================
proptest! {
    #[test]
    fn state_has_conflicts_iff_multi_action(s in 0..500u16, r in 0..500u16) {
        use adze_glr_core::conflict_inspection::state_has_conflicts;
        let table = make_test_table(vec![
            vec![vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))]],
            vec![vec![Action::Shift(StateId(s))]],
        ]);
        prop_assert!(state_has_conflicts(&table, StateId(0)));
        prop_assert!(!state_has_conflicts(&table, StateId(1)));
    }
}

// ============================================================================
// 38. get_state_conflicts returns only conflicts for the queried state
// ============================================================================
proptest! {
    #[test]
    fn get_state_conflicts_filters_by_state(s in 0..100u16, r in 0..100u16) {
        use adze_glr_core::conflict_inspection::get_state_conflicts;
        let table = make_test_table(vec![
            vec![vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))]],
            vec![vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))]],
            vec![vec![Action::Shift(StateId(s))]],
        ]);
        let c0 = get_state_conflicts(&table, StateId(0));
        let c1 = get_state_conflicts(&table, StateId(1));
        let c2 = get_state_conflicts(&table, StateId(2));
        prop_assert_eq!(c0.len(), 1);
        prop_assert_eq!(c1.len(), 1);
        prop_assert_eq!(c2.len(), 0);
    }
}

// ============================================================================
// 39. find_conflicts_for_symbol filters correctly by symbol
// ============================================================================
#[test]
fn find_conflicts_for_symbol_filters_correctly() {
    use adze_glr_core::conflict_inspection::find_conflicts_for_symbol;
    let mut table = make_test_table(vec![vec![
        vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))],
        vec![Action::Shift(StateId(1))],
    ]]);
    // Set up index_to_symbol so the first column maps to SymbolId(5) and second to SymbolId(6)
    table.index_to_symbol = vec![SymbolId(5), SymbolId(6)];
    let c5 = find_conflicts_for_symbol(&table, SymbolId(5));
    let c6 = find_conflicts_for_symbol(&table, SymbolId(6));
    assert_eq!(c5.len(), 1);
    assert_eq!(c6.len(), 0);
}

// ============================================================================
// 40. classify_conflict: empty actions → Mixed (no shift, no reduce)
// ============================================================================
#[test]
fn classify_empty_actions_is_mixed() {
    let ct = classify_conflict(&[]);
    assert_eq!(ct, ConflictType::Mixed);
}

// ============================================================================
// 41. classify_conflict: Accept + Error → Mixed
// ============================================================================
#[test]
fn classify_accept_error_is_mixed() {
    let ct = classify_conflict(&[Action::Accept, Action::Error]);
    assert_eq!(ct, ConflictType::Mixed);
}

// ============================================================================
// 42. classify_conflict: Recover + Shift → Mixed (has_shift but no reduce)
// ============================================================================
#[test]
fn classify_recover_shift_is_mixed() {
    let ct = classify_conflict(&[Action::Recover, Action::Shift(StateId(1))]);
    assert_eq!(ct, ConflictType::Mixed);
}

// ============================================================================
// 43. classify_conflict: nested Fork([Fork([Shift, Reduce])]) → ShiftReduce
// ============================================================================
#[test]
fn classify_nested_fork_sr() {
    let inner = Action::Fork(vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))]);
    let outer = Action::Fork(vec![inner]);
    let ct = classify_conflict(&[outer]);
    assert_eq!(ct, ConflictType::ShiftReduce);
}

// ============================================================================
// 44. classify_conflict: Fork([Reduce, Reduce]) → ReduceReduce
// ============================================================================
#[test]
fn classify_fork_reduce_reduce() {
    let ct = classify_conflict(&[Action::Fork(vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ])]);
    assert_eq!(ct, ConflictType::ReduceReduce);
}

// ============================================================================
// 45. ConflictDetail actions length matches cell size
// ============================================================================
proptest! {
    #[test]
    fn conflict_detail_actions_length(n_actions in 2usize..6) {
        let actions: Vec<Action> = (0..n_actions)
            .map(|i| if i % 2 == 0 {
                Action::Shift(StateId(i as u16))
            } else {
                Action::Reduce(RuleId(i as u16))
            })
            .collect();
        let table = make_test_table(vec![vec![actions.clone()]]);
        let summary = count_conflicts(&table);
        prop_assert_eq!(summary.conflict_details.len(), 1);
        prop_assert_eq!(summary.conflict_details[0].actions.len(), n_actions);
    }
}

// ============================================================================
// 46. ConflictSummary Display includes shift_reduce and reduce_reduce counts
// ============================================================================
proptest! {
    #[test]
    fn summary_display_includes_both_counts(sr in 0usize..4, rr in 0usize..4) {
        let mut cells = Vec::new();
        for i in 0..sr {
            cells.push(vec![Action::Shift(StateId(i as u16)), Action::Reduce(RuleId(i as u16))]);
        }
        for i in 0..rr {
            cells.push(vec![Action::Reduce(RuleId(100 + i as u16)), Action::Reduce(RuleId(200 + i as u16))]);
        }
        if cells.is_empty() {
            cells.push(vec![Action::Shift(StateId(0))]);
        }
        let table = make_test_table(vec![cells]);
        let summary = count_conflicts(&table);
        let display = format!("{}", summary);
        prop_assert!(display.contains("Shift/Reduce conflicts:"));
        prop_assert!(display.contains("Reduce/Reduce conflicts:"));
        prop_assert!(display.contains("States with conflicts:"));
    }
}

// ============================================================================
// 47. ConflictDetail Display includes state and symbol info
// ============================================================================
#[test]
fn conflict_detail_display_format() {
    use adze_glr_core::conflict_inspection::ConflictDetail;
    let detail = ConflictDetail {
        state: StateId(7),
        symbol: SymbolId(3),
        symbol_name: "plus".into(),
        conflict_type: ConflictType::ShiftReduce,
        actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))],
        priorities: vec![0, 0],
    };
    let s = format!("{}", detail);
    assert!(s.contains("State 7"));
    assert!(s.contains("plus"));
    assert!(s.contains("ShiftReduce"));
}

// ============================================================================
// 48. PrecedenceResolver: no token/symbol prec → always None
// ============================================================================
#[test]
fn prec_resolver_empty_grammar_returns_none() {
    let g = Grammar::new("empty".to_string());
    let resolver = PrecedenceResolver::new(&g);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(1), SymbolId(2)),
        None
    );
}

// ============================================================================
// 49. PrecedenceResolver: only token prec set, no symbol prec → None
// ============================================================================
#[test]
fn prec_resolver_only_token_prec_returns_none() {
    let mut g = Grammar::new("token_only".to_string());
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    let resolver = PrecedenceResolver::new(&g);
    // Shift symbol has prec, but reduce symbol has no rule prec → None
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(1), SymbolId(99)),
        None
    );
}

// ============================================================================
// 50. PrecedenceResolver: only symbol prec set, no token prec → None
// ============================================================================
#[test]
fn prec_resolver_only_symbol_prec_returns_none() {
    let mut g = Grammar::new("symbol_only".to_string());
    g.rules.insert(
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
    let resolver = PrecedenceResolver::new(&g);
    // Shift symbol has no token prec → None
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(1), SymbolId(10)),
        None
    );
}

// ============================================================================
// 51. ConflictAnalyzer: analyze_table on conflict-free table → zero stats
// ============================================================================
#[test]
fn analyzer_conflict_free_table_zero_stats() {
    let table = make_test_table(vec![vec![vec![Action::Shift(StateId(0))]]]);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
    assert_eq!(stats.associativity_resolved, 0);
    assert_eq!(stats.explicit_glr, 0);
    assert_eq!(stats.default_resolved, 0);
}

// ============================================================================
// 52. ConflictStats: Clone produces identical copy
// ============================================================================
#[test]
fn conflict_stats_clone_eq() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 3,
        reduce_reduce_conflicts: 1,
        precedence_resolved: 2,
        associativity_resolved: 4,
        explicit_glr: 5,
        default_resolved: 0,
    };
    let cloned = stats.clone();
    assert_eq!(cloned.shift_reduce_conflicts, 3);
    assert_eq!(cloned.reduce_reduce_conflicts, 1);
    assert_eq!(cloned.precedence_resolved, 2);
    assert_eq!(cloned.associativity_resolved, 4);
    assert_eq!(cloned.explicit_glr, 5);
    assert_eq!(cloned.default_resolved, 0);
}

// ============================================================================
// 53. ConflictResolver::detect_conflicts on ambiguous E → a | E E grammar
// ============================================================================
#[test]
fn detect_conflicts_ambig_concat_grammar() {
    let g = ambiguous_concat_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = adze_glr_core::ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = adze_glr_core::ConflictResolver::detect_conflicts(&collection, &g, &ff);
    assert!(
        !resolver.conflicts.is_empty(),
        "E → a | E E should produce conflicts"
    );
}

// ============================================================================
// 54. ConflictResolver: all detected conflicts have valid ConflictType
// ============================================================================
#[test]
fn detect_conflicts_all_have_valid_type() {
    let g = ambiguous_concat_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = adze_glr_core::ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = adze_glr_core::ConflictResolver::detect_conflicts(&collection, &g, &ff);
    for c in &resolver.conflicts {
        // ConflictType is either ShiftReduce or ReduceReduce (the lib.rs enum)
        match c.conflict_type {
            adze_glr_core::ConflictType::ShiftReduce => {}
            adze_glr_core::ConflictType::ReduceReduce => {}
        }
    }
}

// ============================================================================
// 55. ConflictResolver: each conflict has ≥2 actions
// ============================================================================
#[test]
fn detect_conflicts_each_has_multiple_actions() {
    let g = ambiguous_concat_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = adze_glr_core::ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = adze_glr_core::ConflictResolver::detect_conflicts(&collection, &g, &ff);
    for c in &resolver.conflicts {
        assert!(
            c.actions.len() >= 2,
            "Conflict at state {:?} should have ≥2 actions, got {}",
            c.state,
            c.actions.len()
        );
    }
}

// ============================================================================
// 56. build_lr1_automaton on ambiguous grammar preserves fork cells
// ============================================================================
#[test]
fn build_automaton_ambig_has_fork_cells() {
    let g = ambiguous_concat_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).expect("build_lr1_automaton failed");
    let forks = count_fork_cells(&table);
    assert!(
        forks > 0,
        "Ambiguous grammar should have fork cells in the parse table"
    );
}

// ============================================================================
// 57. rr_grammar produces reduce/reduce conflicts
// ============================================================================
#[test]
fn rr_grammar_has_reduce_reduce_conflicts() {
    let g = rr_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = adze_glr_core::ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = adze_glr_core::ConflictResolver::detect_conflicts(&collection, &g, &ff);
    let has_rr = resolver
        .conflicts
        .iter()
        .any(|c| matches!(c.conflict_type, adze_glr_core::ConflictType::ReduceReduce));
    assert!(
        has_rr,
        "S → A | B; A → a; B → a should produce reduce/reduce conflict"
    );
}

// ============================================================================
// 58. classify_conflict with 3 Reduce actions → ReduceReduce
// ============================================================================
#[test]
fn classify_three_reduces_is_rr() {
    let ct = classify_conflict(&[
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ]);
    assert_eq!(ct, ConflictType::ReduceReduce);
}

// ============================================================================
// 59. classify_conflict: Shift + Reduce + Reduce → ShiftReduce
// ============================================================================
#[test]
fn classify_shift_reduce_reduce_is_sr() {
    let ct = classify_conflict(&[
        Action::Shift(StateId(0)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ]);
    assert_eq!(ct, ConflictType::ShiftReduce);
}

// ============================================================================
// 60. count_conflicts: no-conflict table → empty details
// ============================================================================
#[test]
fn count_conflicts_no_conflict_empty_details() {
    let table = make_test_table(vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Reduce(RuleId(0))]],
    ]);
    let summary = count_conflicts(&table);
    assert!(summary.conflict_details.is_empty());
    assert!(summary.states_with_conflicts.is_empty());
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}
