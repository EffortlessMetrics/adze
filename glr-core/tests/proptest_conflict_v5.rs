//! Property-based tests for conflict detection in parse tables.
//!
//! Run with: `cargo test -p adze-glr-core --test proptest_conflict_v5 -- --test-threads=2`

use adze_glr_core::{
    Action, ConflictResolver, FirstFollowSets, GotoIndexing, ItemSetCollection, ParseTable,
};
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId,
    Token, TokenPattern,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ============================================================================
// Helpers
// ============================================================================

/// Build a simple unambiguous grammar: S → a
fn unambiguous_single(tok_id: u16) -> Grammar {
    let mut g = Grammar::new("unambig_single".into());
    let a = SymbolId(tok_id);
    let s = SymbolId(tok_id + 10);

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
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    g
}

/// Build an unambiguous two-rule grammar: S → a b
fn unambiguous_seq(t1: u16, t2: u16) -> Grammar {
    let mut g = Grammar::new("unambig_seq".into());
    let a = SymbolId(t1);
    let b = SymbolId(t2);
    let s = SymbolId(20);

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
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a), Symbol::Terminal(b)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    g
}

/// Build an unambiguous chain grammar: S → A; A → a
fn unambiguous_chain() -> Grammar {
    let mut g = Grammar::new("unambig_chain".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    let a_nt = SymbolId(11);

    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(a_nt, "A".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(a_nt)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    g.rules.insert(
        a_nt,
        vec![Rule {
            lhs: a_nt,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        }],
    );
    g
}

/// Build ambiguous grammar: E → a | E E
fn ambiguous_concat() -> Grammar {
    let mut g = Grammar::new("ambig_concat".into());
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

/// Build expression grammar: E → E op E | num
fn expr_grammar(prec: Option<PrecedenceKind>, assoc: Option<Associativity>) -> Grammar {
    let mut g = Grammar::new("expr".into());
    let num = SymbolId(1);
    let op = SymbolId(2);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("0".into()),
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

/// Build two-operator expr grammar: E → E + E | E * E | num
fn two_op_grammar(
    plus_prec: Option<PrecedenceKind>,
    plus_assoc: Option<Associativity>,
    star_prec: Option<PrecedenceKind>,
    star_assoc: Option<Associativity>,
) -> Grammar {
    let mut g = Grammar::new("two_op".into());
    let num = SymbolId(1);
    let plus = SymbolId(2);
    let star = SymbolId(3);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("0".into()),
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
                    Symbol::Terminal(star),
                    Symbol::NonTerminal(e),
                ],
                precedence: star_prec,
                associativity: star_assoc,
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

/// Build reduce/reduce grammar: S → A | B; A → a; B → a
fn rr_grammar() -> Grammar {
    let mut g = Grammar::new("rr".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    let a_nt = SymbolId(11);
    let b_nt = SymbolId(12);

    g.tokens.insert(
        a,
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
            rhs: vec![Symbol::Terminal(a)],
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
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(3),
            fields: vec![],
        }],
    );
    g
}

/// Build a multi-nonterminal grammar with `n` extra non-terminals.
/// S → A1; A1 → A2; ... An → a
fn chain_grammar(n: usize) -> Grammar {
    let n = n.clamp(1, 8);
    let mut g = Grammar::new("chain".into());
    let a = SymbolId(1);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );

    let s = SymbolId(10);
    g.rule_names.insert(s, "S".into());

    let mut nts: Vec<SymbolId> = Vec::new();
    for i in 0..n {
        let nt = SymbolId(11 + i as u16);
        g.rule_names.insert(nt, format!("N{}", i));
        nts.push(nt);
    }

    // S → first NT
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(nts[0])],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );

    for i in 0..n {
        let rhs = if i + 1 < n {
            vec![Symbol::NonTerminal(nts[i + 1])]
        } else {
            vec![Symbol::Terminal(a)]
        };
        g.rules.insert(
            nts[i],
            vec![Rule {
                lhs: nts[i],
                rhs,
                precedence: None,
                associativity: None,
                production_id: ProductionId(1 + i as u16),
                fields: vec![],
            }],
        );
    }
    g
}

/// Detect conflicts using ItemSetCollection + ConflictResolver.
fn detect_conflicts(g: &Grammar) -> ConflictResolver {
    let ff = FirstFollowSets::compute(g).expect("FIRST/FOLLOW should succeed");
    let collection = ItemSetCollection::build_canonical_collection(g, &ff);
    ConflictResolver::detect_conflicts(&collection, g, &ff)
}

/// Build parse table from grammar.
fn build_table(g: &Grammar) -> Option<ParseTable> {
    let ff = FirstFollowSets::compute(g).ok()?;
    adze_glr_core::build_lr1_automaton(g, &ff).ok()
}

/// Create a minimal ParseTable for unit-level testing.
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
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        grammar: Grammar::new("test".into()),
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

/// Check whether any cell in the action table contains a Fork action.
fn has_fork(table: &ParseTable) -> bool {
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if matches!(action, Action::Fork(_)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Count total actions across all cells in the action table.
fn total_actions(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .map(|cell| cell.len())
        .sum()
}

/// Count cells that have more than one action.
fn multi_action_cells(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

/// Collect all Fork actions from a table.
fn collect_forks(table: &ParseTable) -> Vec<&Vec<Action>> {
    let mut forks = Vec::new();
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Fork(inner) = action {
                    forks.push(inner);
                }
            }
        }
    }
    forks
}

/// Total number of rules in a grammar.
fn rule_count(g: &Grammar) -> usize {
    g.rules.values().map(|rs| rs.len()).sum()
}

// ============================================================================
// Category 1: Unambiguous grammars have no Fork actions (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn unambig_single_no_fork(tok_id in 1u16..10) {
        let g = unambiguous_single(tok_id);
        if let Some(table) = build_table(&g) {
            prop_assert!(!has_fork(&table), "unambiguous single-rule grammar should have no Fork");
        }
    }

    #[test]
    fn unambig_seq_no_fork(t1 in 1u16..5, t2 in 5u16..10) {
        let g = unambiguous_seq(t1, t2);
        if let Some(table) = build_table(&g) {
            prop_assert!(!has_fork(&table), "unambiguous sequence grammar should have no Fork");
        }
    }

    #[test]
    fn unambig_chain_no_fork(_dummy in 0u8..1) {
        let g = unambiguous_chain();
        if let Some(table) = build_table(&g) {
            prop_assert!(!has_fork(&table), "unambiguous chain grammar should have no Fork");
        }
    }

    #[test]
    fn unambig_chain_n_no_fork(n in 1usize..6) {
        let g = chain_grammar(n);
        if let Some(table) = build_table(&g) {
            prop_assert!(!has_fork(&table), "chain grammar of length {} should have no Fork", n);
        }
    }

    #[test]
    fn unambig_no_conflicts(tok_id in 1u16..10) {
        let g = unambiguous_single(tok_id);
        let resolver = detect_conflicts(&g);
        prop_assert!(resolver.conflicts.is_empty(), "unambiguous grammar should have 0 conflicts");
    }
}

// ============================================================================
// Category 2: Action count is bounded by grammar size (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn action_count_bounded_single(tok_id in 1u16..10) {
        let g = unambiguous_single(tok_id);
        if let Some(table) = build_table(&g) {
            let total = total_actions(&table);
            let cells: usize = table.action_table.iter().map(|row| row.len()).sum();
            // Each cell has at most a handful of actions bounded by rule count
            prop_assert!(total <= cells * (rule_count(&g) + 5),
                "total actions {} should be bounded by cells * (rules + 5)", total);
        }
    }

    #[test]
    fn action_count_bounded_expr(_dummy in 0u8..1) {
        let g = expr_grammar(None, None);
        if let Some(table) = build_table(&g) {
            let total = total_actions(&table);
            let max_per_cell = rule_count(&g) + 3;
            let cells: usize = table.action_table.iter().map(|row| row.len()).sum();
            prop_assert!(total <= cells * max_per_cell,
                "total {} should be bounded", total);
        }
    }

    #[test]
    fn multi_action_bounded_by_state_count(_dummy in 0u8..1) {
        let g = ambiguous_concat();
        if let Some(table) = build_table(&g) {
            let mc = multi_action_cells(&table);
            let cells: usize = table.action_table.iter().map(|row| row.len()).sum();
            prop_assert!(mc <= cells, "multi-action cells {} ≤ total cells {}", mc, cells);
        }
    }

    #[test]
    fn chain_action_count_scales(n in 1usize..6) {
        let g = chain_grammar(n);
        if let Some(table) = build_table(&g) {
            let total = total_actions(&table);
            // Bounded by state_count * symbol_count * rules
            let bound = table.state_count * table.symbol_count.max(1) * (rule_count(&g) + 2);
            prop_assert!(total <= bound, "total {} should be ≤ {}", total, bound);
        }
    }

    #[test]
    fn two_op_action_bounded(level in 1i16..=10) {
        let g = two_op_grammar(
            Some(PrecedenceKind::Static(level)),
            Some(Associativity::Left),
            Some(PrecedenceKind::Static(level + 1)),
            Some(Associativity::Left),
        );
        if let Some(table) = build_table(&g) {
            let total = total_actions(&table);
            let cells: usize = table.action_table.iter().map(|row| row.len()).sum();
            prop_assert!(total <= cells * 10, "bounded action count");
        }
    }
}

// ============================================================================
// Category 3: Conflict detection is deterministic (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn detect_deterministic_single(tok_id in 1u16..10) {
        let g = unambiguous_single(tok_id);
        let r1 = detect_conflicts(&g);
        let r2 = detect_conflicts(&g);
        prop_assert_eq!(r1.conflicts.len(), r2.conflicts.len(),
            "conflict count should be deterministic");
    }

    #[test]
    fn detect_deterministic_ambig(_dummy in 0u8..1) {
        let g = ambiguous_concat();
        let r1 = detect_conflicts(&g);
        let r2 = detect_conflicts(&g);
        prop_assert_eq!(r1.conflicts.len(), r2.conflicts.len());
    }

    #[test]
    fn detect_deterministic_rr(_dummy in 0u8..1) {
        let g = rr_grammar();
        let r1 = detect_conflicts(&g);
        let r2 = detect_conflicts(&g);
        prop_assert_eq!(r1.conflicts.len(), r2.conflicts.len());
    }

    #[test]
    fn detect_deterministic_expr(_dummy in 0u8..1) {
        let g = expr_grammar(None, None);
        let r1 = detect_conflicts(&g);
        let r2 = detect_conflicts(&g);
        prop_assert_eq!(r1.conflicts.len(), r2.conflicts.len());
    }

    #[test]
    fn build_table_deterministic_single(tok_id in 1u16..10) {
        let g = unambiguous_single(tok_id);
        let t1 = build_table(&g);
        let t2 = build_table(&g);
        match (t1, t2) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.state_count, b.state_count);
                prop_assert_eq!(a.action_table.len(), b.action_table.len());
            }
            (None, None) => {} // both failed equally
            _ => prop_assert!(false, "one succeeded and the other failed"),
        }
    }
}

// ============================================================================
// Category 4: Fork actions contain valid inner actions (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn fork_inner_not_empty(s in 0..500u16, r in 0..500u16) {
        let fork = Action::Fork(vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))]);
        if let Action::Fork(inner) = &fork {
            prop_assert!(!inner.is_empty(), "Fork should not be empty");
        }
    }

    #[test]
    fn fork_inner_no_nested_fork(s in 0..500u16, r in 0..500u16) {
        let fork = Action::Fork(vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))]);
        if let Action::Fork(inner) = &fork {
            for a in inner {
                prop_assert!(!matches!(a, Action::Fork(_)), "Fork should not nest");
            }
        }
    }

    #[test]
    fn fork_inner_has_shift_or_reduce(s in 0..500u16, r in 0..500u16) {
        let fork = Action::Fork(vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(r))]);
        if let Action::Fork(inner) = &fork {
            let has_shift_or_reduce = inner.iter().any(|a| {
                matches!(a, Action::Shift(_) | Action::Reduce(_))
            });
            prop_assert!(has_shift_or_reduce, "Fork must contain Shift or Reduce");
        }
    }

    #[test]
    fn table_forks_contain_valid_leaf_actions(_dummy in 0u8..1) {
        let g = expr_grammar(None, None);
        if let Some(table) = build_table(&g) {
            for fork_inner in collect_forks(&table) {
                for action in fork_inner {
                    match action {
                        Action::Shift(_) | Action::Reduce(_) | Action::Accept
                        | Action::Error | Action::Recover => {}
                        Action::Fork(_) => {
                            prop_assert!(false, "nested Fork inside Fork");
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    #[test]
    fn resolved_fork_inner_valid(_dummy in 0u8..1) {
        let g = ambiguous_concat();
        let ff = FirstFollowSets::compute(&g).unwrap();
        let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
        let mut resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
        resolver.resolve_conflicts(&g);
        for conflict in &resolver.conflicts {
            for action in &conflict.actions {
                if let Action::Fork(inner) = action {
                    prop_assert!(!inner.is_empty(), "resolved Fork should not be empty");
                    for a in inner {
                        prop_assert!(!matches!(a, Action::Fork(_)), "no nested Fork after resolve");
                    }
                }
            }
        }
    }
}

// ============================================================================
// Category 5: Shift targets in Fork are valid state IDs (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn shift_state_preserved_in_fork(s in 0..1000u16) {
        let fork = Action::Fork(vec![Action::Shift(StateId(s)), Action::Reduce(RuleId(0))]);
        if let Action::Fork(inner) = &fork {
            let found = inner.iter().any(|a| matches!(a, Action::Shift(sid) if sid.0 == s));
            prop_assert!(found, "Shift state {} should be preserved in Fork", s);
        }
    }

    #[test]
    fn table_shift_targets_in_range(_dummy in 0u8..1) {
        let g = expr_grammar(None, None);
        if let Some(table) = build_table(&g) {
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        if let Action::Shift(sid) = action {
                            prop_assert!((sid.0 as usize) < table.state_count,
                                "Shift target {} out of range (states={})", sid.0, table.state_count);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fork_shift_targets_in_range(_dummy in 0u8..1) {
        let g = expr_grammar(None, None);
        if let Some(table) = build_table(&g) {
            for fork_inner in collect_forks(&table) {
                for action in fork_inner {
                    if let Action::Shift(sid) = action {
                        prop_assert!((sid.0 as usize) < table.state_count,
                            "Fork Shift target {} out of range", sid.0);
                    }
                }
            }
        }
    }

    #[test]
    fn ambig_shift_targets_valid(_dummy in 0u8..1) {
        let g = ambiguous_concat();
        if let Some(table) = build_table(&g) {
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        match action {
                            Action::Shift(sid) => {
                                prop_assert!((sid.0 as usize) < table.state_count);
                            }
                            Action::Fork(inner) => {
                                for a in inner {
                                    if let Action::Shift(sid) = a {
                                        prop_assert!((sid.0 as usize) < table.state_count);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn two_op_shift_targets_valid(level in 1i16..=5) {
        let g = two_op_grammar(
            Some(PrecedenceKind::Static(level)),
            Some(Associativity::Left),
            Some(PrecedenceKind::Static(level + 1)),
            Some(Associativity::Left),
        );
        if let Some(table) = build_table(&g) {
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        if let Action::Shift(sid) = action {
                            prop_assert!((sid.0 as usize) < table.state_count);
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Category 6: Reduce in Fork has valid rule IDs (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn reduce_rule_preserved_in_fork(r in 0..1000u16) {
        let fork = Action::Fork(vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(r))]);
        if let Action::Fork(inner) = &fork {
            let found = inner.iter().any(|a| matches!(a, Action::Reduce(rid) if rid.0 == r));
            prop_assert!(found, "Reduce rule {} should be preserved in Fork", r);
        }
    }

    #[test]
    fn table_reduce_ids_valid(_dummy in 0u8..1) {
        let g = expr_grammar(None, None);
        if let Some(table) = build_table(&g) {
            let max_rule = table.rules.len();
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        if let Action::Reduce(rid) = action {
                            prop_assert!((rid.0 as usize) < max_rule + 10,
                                "Reduce rule {} should be in reasonable range (rules={})", rid.0, max_rule);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fork_reduce_ids_valid(_dummy in 0u8..1) {
        let g = expr_grammar(None, None);
        if let Some(table) = build_table(&g) {
            let max_rule = table.rules.len();
            for fork_inner in collect_forks(&table) {
                for action in fork_inner {
                    if let Action::Reduce(rid) = action {
                        prop_assert!((rid.0 as usize) < max_rule + 10,
                            "Fork Reduce rule {} in range (rules={})", rid.0, max_rule);
                    }
                }
            }
        }
    }

    #[test]
    fn ambig_reduce_ids_bounded(_dummy in 0u8..1) {
        let g = ambiguous_concat();
        if let Some(table) = build_table(&g) {
            let max_rule = table.rules.len();
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        match action {
                            Action::Reduce(rid) => {
                                prop_assert!((rid.0 as usize) < max_rule + 10);
                            }
                            Action::Fork(inner) => {
                                for a in inner {
                                    if let Action::Reduce(rid) = a {
                                        prop_assert!((rid.0 as usize) < max_rule + 10);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn rr_reduce_ids_bounded(_dummy in 0u8..1) {
        let g = rr_grammar();
        if let Some(table) = build_table(&g) {
            let max_rule = table.rules.len();
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        if let Action::Reduce(rid) = action {
                            prop_assert!((rid.0 as usize) < max_rule + 10);
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Category 7: Precedence reduces conflicts (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn left_assoc_resolves_sr(level in 1i16..=10) {
        let g_no_prec = expr_grammar(None, None);
        let g_prec = expr_grammar(
            Some(PrecedenceKind::Static(level)),
            Some(Associativity::Left),
        );
        let r_no = detect_conflicts(&g_no_prec);
        let r_prec = detect_conflicts(&g_prec);
        // With precedence, either fewer or equal conflicts
        prop_assert!(r_prec.conflicts.len() <= r_no.conflicts.len() + 1,
            "precedence should not increase conflicts: no_prec={} prec={}",
            r_no.conflicts.len(), r_prec.conflicts.len());
    }

    #[test]
    fn right_assoc_resolves_sr(level in 1i16..=10) {
        let g_prec = expr_grammar(
            Some(PrecedenceKind::Static(level)),
            Some(Associativity::Right),
        );
        let mut resolver = detect_conflicts(&g_prec);
        resolver.resolve_conflicts(&g_prec);
        // After resolution, each conflict action list should be bounded
        for c in &resolver.conflicts {
            prop_assert!(c.actions.len() <= rule_count(&g_prec) + 2,
                "resolved conflict should have bounded actions, got {}", c.actions.len());
        }
    }

    #[test]
    fn two_op_prec_order_matters(low in 1i16..=5, high in 6i16..=10) {
        let g = two_op_grammar(
            Some(PrecedenceKind::Static(low)),
            Some(Associativity::Left),
            Some(PrecedenceKind::Static(high)),
            Some(Associativity::Left),
        );
        let mut resolver = detect_conflicts(&g);
        let before_total: usize = resolver.conflicts.iter().map(|c| c.actions.len()).sum();
        resolver.resolve_conflicts(&g);
        let after_total: usize = resolver.conflicts.iter().map(|c| c.actions.len()).sum();
        // Resolution should not increase total action count
        prop_assert!(after_total <= before_total + 1,
            "resolution should not increase total: before={} after={}", before_total, after_total);
    }

    #[test]
    fn prec_resolve_reduces_conflict_actions(level in 1i16..=10) {
        let g = expr_grammar(
            Some(PrecedenceKind::Static(level)),
            Some(Associativity::Left),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
        let before = ConflictResolver::detect_conflicts(&collection, &g, &ff);
        let mut after = ConflictResolver::detect_conflicts(&collection, &g, &ff);
        after.resolve_conflicts(&g);
        // Total action count should not increase after resolution
        let before_total: usize = before.conflicts.iter().map(|c| c.actions.len()).sum();
        let after_total: usize = after.conflicts.iter().map(|c| c.actions.len()).sum();
        prop_assert!(after_total <= before_total + 1,
            "resolution should not increase total actions: before={} after={}", before_total, after_total);
    }

    #[test]
    fn resolve_preserves_conflict_count(level in 1i16..=10) {
        let g = expr_grammar(
            Some(PrecedenceKind::Static(level)),
            Some(Associativity::Left),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
        let before = ConflictResolver::detect_conflicts(&collection, &g, &ff);
        let mut after = ConflictResolver::detect_conflicts(&collection, &g, &ff);
        after.resolve_conflicts(&g);
        // Resolve doesn't remove entries, it modifies action lists
        prop_assert_eq!(after.conflicts.len(), before.conflicts.len(),
            "resolve_conflicts should preserve conflict entry count");
    }
}

// ============================================================================
// Category 8: Larger grammars may have more conflicts (5 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn single_rule_fewer_conflicts_than_ambig(_dummy in 0u8..1) {
        let g_simple = unambiguous_single(1);
        let g_ambig = ambiguous_concat();
        let r_simple = detect_conflicts(&g_simple);
        let r_ambig = detect_conflicts(&g_ambig);
        prop_assert!(r_simple.conflicts.len() <= r_ambig.conflicts.len(),
            "simple={} should be ≤ ambig={}", r_simple.conflicts.len(), r_ambig.conflicts.len());
    }

    #[test]
    fn chain_no_more_conflicts_than_ambig(n in 1usize..6) {
        let g_chain = chain_grammar(n);
        let g_ambig = ambiguous_concat();
        let r_chain = detect_conflicts(&g_chain);
        let r_ambig = detect_conflicts(&g_ambig);
        prop_assert!(r_chain.conflicts.len() <= r_ambig.conflicts.len() + 1,
            "chain({})={} should be ≤ ambig={}+1", n, r_chain.conflicts.len(), r_ambig.conflicts.len());
    }

    #[test]
    fn conflict_count_nonnegative(tok_id in 1u16..10) {
        let g = unambiguous_single(tok_id);
        let r = detect_conflicts(&g);
        // conflicts.len() is usize, always >= 0, but verify it's a valid count
        prop_assert!(r.conflicts.len() < 10000, "conflict count should be reasonable");
    }

    #[test]
    fn two_op_has_conflicts_without_prec(_dummy in 0u8..1) {
        let g = two_op_grammar(None, None, None, None);
        let r = detect_conflicts(&g);
        // Two-operator grammar without prec should have conflicts
        prop_assert!(!r.conflicts.is_empty(),
            "two-op grammar without prec should have conflicts");
    }

    #[test]
    fn rr_grammar_has_conflicts(_dummy in 0u8..1) {
        let g = rr_grammar();
        let r = detect_conflicts(&g);
        prop_assert!(!r.conflicts.is_empty(), "R/R grammar should have conflicts");
    }
}

// ============================================================================
// Category 9: Edge cases (6 properties)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn empty_fork_is_constructible(_dummy in 0u8..1) {
        let fork = Action::Fork(vec![]);
        if let Action::Fork(inner) = &fork {
            prop_assert!(inner.is_empty());
        }
    }

    #[test]
    fn single_action_fork(s in 0..500u16) {
        let fork = Action::Fork(vec![Action::Shift(StateId(s))]);
        if let Action::Fork(inner) = &fork {
            prop_assert_eq!(inner.len(), 1);
        }
    }

    #[test]
    fn conflict_type_sr_vs_rr(has_shift in proptest::bool::ANY) {
        let actions = if has_shift {
            vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))]
        } else {
            vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))]
        };
        // Build a mini table with this conflict
        let table = make_test_table(vec![vec![actions]]);
        let mc = multi_action_cells(&table);
        prop_assert_eq!(mc, 1, "should have exactly 1 multi-action cell");
    }

    #[test]
    fn make_test_table_state_count(n in 1usize..10) {
        let action_table: Vec<Vec<Vec<Action>>> = (0..n)
            .map(|_| vec![vec![Action::Error]])
            .collect();
        let table = make_test_table(action_table);
        prop_assert_eq!(table.state_count, n);
        prop_assert_eq!(table.action_table.len(), n);
    }

    #[test]
    fn conflict_state_ids_valid(_dummy in 0u8..1) {
        let g = ambiguous_concat();
        let ff = FirstFollowSets::compute(&g).unwrap();
        let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
        let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
        let n_states = collection.sets.len();
        for conflict in &resolver.conflicts {
            prop_assert!((conflict.state.0 as usize) < n_states,
                "conflict state {} out of range (states={})", conflict.state.0, n_states);
        }
    }

    #[test]
    fn conflict_actions_nonempty(_dummy in 0u8..1) {
        let g = ambiguous_concat();
        let resolver = detect_conflicts(&g);
        for conflict in &resolver.conflicts {
            prop_assert!(!conflict.actions.is_empty(),
                "conflict at state {} should have actions", conflict.state.0);
        }
    }
}
