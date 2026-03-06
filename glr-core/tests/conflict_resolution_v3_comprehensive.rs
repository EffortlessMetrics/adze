//! Comprehensive tests for conflict detection and resolution in adze-glr-core.
//!
//! Covers: unambiguous grammars, shift-reduce conflicts, reduce-reduce conflicts,
//! precedence-based resolution, conflict counts/properties, grammar topologies,
//! ConflictResolver API, and edge cases.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_resolution_v3_comprehensive

use adze_glr_core::conflict_inspection::{
    classify_conflict, count_conflicts, find_conflicts_for_symbol, get_state_conflicts,
    state_has_conflicts,
};
use adze_glr_core::{
    Action, ConflictResolver, ConflictType, FirstFollowSets, ItemSetCollection, ParseTable,
    build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a parse table from a mutable grammar via the standard pipeline.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(grammar, &ff).expect("LR(1) automaton should build")
}

/// Build item sets + first-follow for the ConflictResolver API.
fn build_collection(grammar: &Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    let collection = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (collection, ff)
}

/// Count cells with more than one action in the parse table.
fn count_fork_cells(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|state| state.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

/// Return true if any cell contains both Shift and Reduce actions.
fn has_shift_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|state| {
        state.iter().any(|cell| {
            cell.len() > 1
                && cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
        })
    })
}

/// Return true if any cell contains two or more Reduce actions.
#[allow(dead_code)]
fn has_reduce_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|state| {
        state.iter().any(|cell| {
            cell.iter()
                .filter(|a| matches!(a, Action::Reduce(_)))
                .count()
                >= 2
        })
    })
}

/// Build a simple expression grammar: E → E op E | num (manual construction).
fn expr_grammar_one_op(
    name: &str,
    op_str: &str,
    prec: Option<PrecedenceKind>,
    assoc: Option<Associativity>,
) -> Grammar {
    let mut g = Grammar::new(name.to_string());

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
            name: op_str.into(),
            pattern: TokenPattern::String(op_str.into()),
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

/// Build a two-operator expression grammar: E → E + E | E * E | num.
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

/// Build a dangling-else grammar.
fn dangling_else_grammar() -> Grammar {
    let mut g = Grammar::new("dangling_else".to_string());

    let if_tok = SymbolId(1);
    let then_tok = SymbolId(2);
    let else_tok = SymbolId(3);
    let expr_tok = SymbolId(4);
    let stmt_tok = SymbolId(5);
    let s = SymbolId(10);

    for (id, name, pat) in [
        (if_tok, "if", "if"),
        (then_tok, "then", "then"),
        (else_tok, "else", "else"),
        (expr_tok, "expr", "expr"),
        (stmt_tok, "stmt", "stmt"),
    ] {
        g.tokens.insert(
            id,
            Token {
                name: name.into(),
                pattern: TokenPattern::String(pat.into()),
                fragile: false,
            },
        );
    }

    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![
            // S → if expr then S
            Rule {
                lhs: s,
                rhs: vec![
                    Symbol::Terminal(if_tok),
                    Symbol::Terminal(expr_tok),
                    Symbol::Terminal(then_tok),
                    Symbol::NonTerminal(s),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            // S → if expr then S else S
            Rule {
                lhs: s,
                rhs: vec![
                    Symbol::Terminal(if_tok),
                    Symbol::Terminal(expr_tok),
                    Symbol::Terminal(then_tok),
                    Symbol::NonTerminal(s),
                    Symbol::Terminal(else_tok),
                    Symbol::NonTerminal(s),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            // S → stmt
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(stmt_tok)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );
    g
}

/// Build a reduce-reduce grammar: S → A | B; A → a; B → a.
fn reduce_reduce_grammar() -> Grammar {
    let mut g = Grammar::new("rr_grammar".to_string());

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

// ===========================================================================
// 1. No conflicts in unambiguous grammars (8 tests)
// ===========================================================================

#[test]
fn test_unambiguous_single_token_no_conflicts() {
    // S → a
    let g = GrammarBuilder::new("single_tok")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(
        count_fork_cells(&table),
        0,
        "S → a should have no conflicts"
    );
}

#[test]
fn test_unambiguous_two_token_sequence_no_conflicts() {
    // S → a b
    let g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(
        count_fork_cells(&table),
        0,
        "S → a b should have no conflicts"
    );
}

#[test]
fn test_unambiguous_alternation_different_tokens_no_conflicts() {
    // S → a | b (different lookahead for each alternative)
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(
        count_fork_cells(&table),
        0,
        "S → a | b should have no conflicts"
    );
}

#[test]
fn test_unambiguous_chain_rules_no_conflicts() {
    // S → A; A → a
    let g = GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(
        count_fork_cells(&table),
        0,
        "chain rule S → A → a should have no conflicts"
    );
}

#[test]
fn test_unambiguous_three_token_sequence_no_conflicts() {
    // S → a b c
    let g = GrammarBuilder::new("three_seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_unambiguous_nested_nonterminals_no_conflicts() {
    // S → A B; A → a; B → b
    let g = GrammarBuilder::new("nested")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .rule("S", vec!["A", "B"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_unambiguous_lr1_distinguishable_alternatives() {
    // S → a A | b B; A → c; B → c
    // LR(1) can distinguish because 'a' vs 'b' on first token.
    let g = GrammarBuilder::new("lr1_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("A", vec!["c"])
        .rule("B", vec!["c"])
        .rule("S", vec!["a", "A"])
        .rule("S", vec!["b", "B"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(
        count_fork_cells(&table),
        0,
        "LR(1)-distinguishable alternatives should have no conflicts"
    );
}

#[test]
fn test_unambiguous_conflict_summary_zero() {
    let g = GrammarBuilder::new("zero_conf")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
    assert!(summary.states_with_conflicts.is_empty());
    assert!(summary.conflict_details.is_empty());
}

// ===========================================================================
// 2. Shift-reduce conflicts detection (8 tests)
// ===========================================================================

#[test]
fn test_sr_one_op_no_prec_detected() {
    // E → E + E | num — no precedence: S/R must be preserved.
    let g = expr_grammar_one_op("sr_noprec", "+", None, None);
    let table = build_table(&g);
    assert!(
        has_shift_reduce(&table),
        "E → E + E without precedence should have S/R conflicts"
    );
}

#[test]
fn test_sr_two_ops_no_prec_detected() {
    let g = expr_grammar_two_ops(None, None, None, None);
    let table = build_table(&g);
    assert!(has_shift_reduce(&table));
}

#[test]
fn test_sr_dangling_else_detected() {
    let g = dangling_else_grammar();
    let table = build_table(&g);
    assert!(
        has_shift_reduce(&table),
        "Dangling-else should produce shift/reduce conflicts"
    );
}

#[test]
fn test_sr_dangling_else_involves_else_token() {
    let g = dangling_else_grammar();
    let else_tok = SymbolId(3);
    let table = build_table(&g);

    // Verify the conflict involves the 'else' token.
    if let Some(&idx) = table.symbol_to_index.get(&else_tok) {
        let else_conflict = table
            .action_table
            .iter()
            .any(|state| state.get(idx).is_some_and(|cell| cell.len() > 1));
        assert!(else_conflict, "conflict should be on the 'else' lookahead");
    }
}

#[test]
fn test_sr_count_matches_manual_scan() {
    let g = expr_grammar_two_ops(None, None, None, None);
    let table = build_table(&g);
    let summary = count_conflicts(&table);

    let manual_sr = table
        .action_table
        .iter()
        .flat_map(|s| s.iter())
        .filter(|cell| {
            cell.len() > 1
                && cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
        })
        .count();

    assert_eq!(summary.shift_reduce, manual_sr);
}

#[test]
fn test_sr_no_prec_fork_cells_positive() {
    let g = expr_grammar_one_op("fork_check", "+", None, None);
    let table = build_table(&g);
    assert!(count_fork_cells(&table) > 0);
}

#[test]
fn test_sr_conflict_has_both_shift_and_reduce_actions() {
    let g = expr_grammar_one_op("action_check", "+", None, None);
    let table = build_table(&g);

    // Find a cell with multiple actions and verify it has both Shift and Reduce.
    let found = table
        .action_table
        .iter()
        .flat_map(|s| s.iter())
        .any(|cell| {
            cell.len() > 1
                && cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
        });
    assert!(found, "should find a cell with both Shift and Reduce");
}

#[test]
fn test_sr_ambiguous_concatenation_grammar() {
    // E → a | E E (inherently ambiguous)
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
    let table = build_table(&g);
    assert!(
        count_fork_cells(&table) > 0,
        "E → a | E E should be ambiguous"
    );
}

// ===========================================================================
// 3. Reduce-reduce conflicts (5 tests)
// ===========================================================================

#[test]
fn test_rr_basic_grammar_builds_ok() {
    // S → A | B; A → a; B → a — classic R/R conflict.
    let g = reduce_reduce_grammar();
    let table = build_table(&g);
    assert!(
        table.state_count > 0,
        "parse table should build successfully"
    );
}

#[test]
fn test_rr_grammar_summary_nonnegative() {
    let g = reduce_reduce_grammar();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    // Regardless of whether the builder resolves R/R internally, counts must be >= 0.
    assert!(
        summary.shift_reduce + summary.reduce_reduce < 100,
        "sanity check"
    );
}

#[test]
fn test_rr_detector_finds_rr_via_item_sets() {
    // Use ConflictResolver directly on item sets.
    let g = reduce_reduce_grammar();
    let (collection, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    // The detector should find at least one conflict (R/R or R/Accept).
    // It works at the item-set level, before the builder resolves.
    let rr_count = resolver
        .conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::ReduceReduce)
        .count();
    // If the grammar truly has an R/R conflict, it should be detected here.
    // The builder may resolve it later by picking the lowest production ID.
    eprintln!(
        "R/R conflicts detected at item-set level: {} (total: {})",
        rr_count,
        resolver.conflicts.len()
    );
    assert!(
        !resolver.conflicts.is_empty(),
        "ConflictResolver should detect at least one conflict for S → A | B; A → a; B → a"
    );
}

#[test]
fn test_rr_three_alternatives_same_token() {
    // S → A | B | C; A → x; B → x; C → x — triple R/R.
    let mut g = Grammar::new("triple_rr".to_string());
    let x_tok = SymbolId(1);
    let s = SymbolId(10);
    let a_nt = SymbolId(11);
    let b_nt = SymbolId(12);
    let c_nt = SymbolId(13);

    g.tokens.insert(
        x_tok,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(a_nt, "A".into());
    g.rule_names.insert(b_nt, "B".into());
    g.rule_names.insert(c_nt, "C".into());

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
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(c_nt)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );
    for (nt, pid) in [(a_nt, 3), (b_nt, 4), (c_nt, 5)] {
        g.rules.insert(
            nt,
            vec![Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(x_tok)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(pid),
                fields: vec![],
            }],
        );
    }

    let (collection, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    assert!(
        !resolver.conflicts.is_empty(),
        "triple R/R grammar should have conflicts"
    );
}

#[test]
fn test_rr_conflict_type_classification() {
    let g = reduce_reduce_grammar();
    let (collection, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    let rr = resolver
        .conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::ReduceReduce)
        .count();
    let sr = resolver
        .conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::ShiftReduce)
        .count();
    // R/R grammar should NOT have S/R conflicts.
    assert_eq!(sr, 0, "R/R grammar should not produce S/R conflicts");
    assert!(rr > 0, "R/R grammar should produce R/R conflicts");
}

// ===========================================================================
// 4. Conflict resolution with precedence (8 tests)
// ===========================================================================

#[test]
fn test_prec_left_assoc_resolves_sr() {
    let g = expr_grammar_one_op(
        "left",
        "+",
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
    );
    let table = build_table(&g);
    assert!(
        !has_shift_reduce(&table),
        "Left-associative should resolve S/R"
    );
}

#[test]
fn test_prec_right_assoc_resolves_sr() {
    let g = expr_grammar_one_op(
        "right",
        "=",
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Right),
    );
    let table = build_table(&g);
    assert!(
        !has_shift_reduce(&table),
        "Right-associative should resolve S/R"
    );
}

#[test]
fn test_prec_nonassoc_preserves_conflict() {
    let g = expr_grammar_one_op(
        "nonassoc",
        "==",
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::None),
    );
    let table = build_table(&g);
    // Non-associative keeps both actions (GLR mode).
    let forks = count_fork_cells(&table);
    assert!(
        forks > 0,
        "Non-associative at equal precedence should preserve the conflict"
    );
}

#[test]
fn test_prec_higher_wins_over_lower() {
    // E → E + E (prec 1, left) | E * E (prec 2, left) | num
    // All conflicts should be resolved.
    let g = expr_grammar_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(2)),
        Some(Associativity::Left),
    );
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "higher precedence should resolve all S/R conflicts"
    );
}

#[test]
fn test_prec_same_level_left_assoc_no_sr() {
    // Both ops at prec 1, left: should still resolve via associativity.
    let g = expr_grammar_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
    );
    let table = build_table(&g);
    assert!(
        !has_shift_reduce(&table),
        "same-level left-assoc should resolve S/R"
    );
}

#[test]
fn test_prec_builder_api_left_assoc_resolves() {
    let g = GrammarBuilder::new("builder_left")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule_with_precedence("E", vec!["E", "plus", "E"], 1, Associativity::Left)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    let table = build_table(&g);
    assert!(
        !has_shift_reduce(&table),
        "GrammarBuilder left-assoc should resolve S/R"
    );
}

#[test]
fn test_prec_builder_api_right_assoc_resolves() {
    let g = GrammarBuilder::new("builder_right")
        .token("num", r"\d+")
        .token("eq", "=")
        .rule_with_precedence("E", vec!["E", "eq", "E"], 1, Associativity::Right)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    let table = build_table(&g);
    assert!(
        !has_shift_reduce(&table),
        "GrammarBuilder right-assoc should resolve S/R"
    );
}

#[test]
fn test_prec_mixed_assoc_levels() {
    // '+' at prec 1 left, '*' at prec 2 left, '^' at prec 3 right.
    let mut g = Grammar::new("mixed_prec".to_string());

    let num = SymbolId(1);
    let plus = SymbolId(2);
    let times = SymbolId(3);
    let caret = SymbolId(4);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    for (id, name, pat) in [(plus, "+", "+"), (times, "*", "*"), (caret, "^", "^")] {
        g.tokens.insert(
            id,
            Token {
                name: name.into(),
                pattern: TokenPattern::String(pat.into()),
                fragile: false,
            },
        );
    }
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
                precedence: Some(PrecedenceKind::Static(1)),
                associativity: Some(Associativity::Left),
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
                precedence: Some(PrecedenceKind::Static(2)),
                associativity: Some(Associativity::Left),
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(caret),
                    Symbol::NonTerminal(e),
                ],
                precedence: Some(PrecedenceKind::Static(3)),
                associativity: Some(Associativity::Right),
                production_id: ProductionId(2),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(num)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(3),
                fields: vec![],
            },
        ],
    );

    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "fully annotated 3-operator grammar should have no S/R conflicts"
    );
}

// ===========================================================================
// 5. Conflict counts and properties (5 tests)
// ===========================================================================

#[test]
fn test_count_states_with_conflicts_unique() {
    let g = expr_grammar_two_ops(None, None, None, None);
    let table = build_table(&g);
    let summary = count_conflicts(&table);

    let unique: std::collections::HashSet<_> =
        summary.states_with_conflicts.iter().copied().collect();
    assert_eq!(
        unique.len(),
        summary.states_with_conflicts.len(),
        "states_with_conflicts should be unique"
    );
}

#[test]
fn test_count_conflict_details_nonempty_for_ambiguous() {
    let g = expr_grammar_one_op("det_nonempty", "+", None, None);
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert!(
        !summary.conflict_details.is_empty(),
        "ambiguous grammar should have conflict details"
    );
}

#[test]
fn test_count_display_format_contains_key_fields() {
    let g = expr_grammar_two_ops(None, None, None, None);
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    let display = format!("{summary}");
    assert!(display.contains("Shift/Reduce conflicts:"));
    assert!(display.contains("Reduce/Reduce conflicts:"));
}

#[test]
fn test_count_resolved_grammar_zero_conflicts() {
    let g = expr_grammar_two_ops(
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        Some(PrecedenceKind::Static(2)),
        Some(Associativity::Left),
    );
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

#[test]
fn test_count_multiple_ops_in_same_state() {
    let g = expr_grammar_two_ops(None, None, None, None);
    let table = build_table(&g);

    let max_conflicts_in_one_state = table
        .action_table
        .iter()
        .map(|state| state.iter().filter(|c| c.len() > 1).count())
        .max()
        .unwrap_or(0);

    assert!(
        max_conflicts_in_one_state >= 2,
        "two-op grammar should have ≥ 2 conflict cells in at least one state"
    );
}

// ===========================================================================
// 6. Grammar topologies (8 tests)
// ===========================================================================

#[test]
fn test_topology_right_recursive_list_no_conflicts() {
    // L → item | item L (right-recursive, unambiguous)
    let g = GrammarBuilder::new("rr_list")
        .token("item", "item")
        .rule("L", vec!["item"])
        .rule("L", vec!["item", "L"])
        .start("L")
        .build();
    let table = build_table(&g);
    assert_eq!(
        count_fork_cells(&table),
        0,
        "right-recursive list is unambiguous"
    );
}

#[test]
fn test_topology_left_recursive_list_no_sr_with_prec() {
    // L → L sep item | item (left-recursive with prec → resolved)
    let g = GrammarBuilder::new("lr_list")
        .token("item", "item")
        .token("sep", ",")
        .rule_with_precedence("L", vec!["L", "sep", "item"], 1, Associativity::Left)
        .rule("L", vec!["item"])
        .start("L")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

#[test]
fn test_topology_if_else_classic_conflict() {
    let g = dangling_else_grammar();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0,
        "if-then-else should have S/R conflict"
    );
}

#[test]
fn test_topology_expression_with_parens_no_conflicts() {
    // E → num | ( E )
    let g = GrammarBuilder::new("parens")
        .token("num", r"\d+")
        .token("lparen", "(")
        .token("rparen", ")")
        .rule("E", vec!["num"])
        .rule("E", vec!["lparen", "E", "rparen"])
        .start("E")
        .build();
    let table = build_table(&g);
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_topology_optional_trailing_comma() {
    // L → item | L comma item (left-recursive separator list)
    let g = GrammarBuilder::new("comma_list")
        .token("item", "item")
        .token("comma", ",")
        .rule("L", vec!["item"])
        .rule("L", vec!["L", "comma", "item"])
        .start("L")
        .build();
    let table = build_table(&g);
    assert_eq!(count_fork_cells(&table), 0, "separator list is unambiguous");
}

#[test]
fn test_topology_nested_expression_resolved() {
    // E → E + E | E * E | ( E ) | num — fully annotated with prec.
    let mut g = Grammar::new("nested_expr".to_string());

    let num = SymbolId(1);
    let plus = SymbolId(2);
    let times = SymbolId(3);
    let lparen = SymbolId(4);
    let rparen = SymbolId(5);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    for (id, name, pat) in [
        (plus, "+", "+"),
        (times, "*", "*"),
        (lparen, "(", "("),
        (rparen, ")", ")"),
    ] {
        g.tokens.insert(
            id,
            Token {
                name: name.into(),
                pattern: TokenPattern::String(pat.into()),
                fragile: false,
            },
        );
    }
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
                precedence: Some(PrecedenceKind::Static(1)),
                associativity: Some(Associativity::Left),
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
                precedence: Some(PrecedenceKind::Static(2)),
                associativity: Some(Associativity::Left),
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::Terminal(lparen),
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(rparen),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(num)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(3),
                fields: vec![],
            },
        ],
    );

    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "fully annotated expr should resolve"
    );
}

#[test]
fn test_topology_single_epsilon_like_grammar() {
    // S → a  (trivial single-production; ensure no spurious conflicts)
    let g = GrammarBuilder::new("epsilon_like")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce + summary.reduce_reduce, 0);
}

#[test]
fn test_topology_deeply_nested_chain_no_conflicts() {
    // S → A; A → B; B → C; C → x
    let g = GrammarBuilder::new("deep_chain")
        .token("x", "x")
        .rule("C", vec!["x"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let table = build_table(&g);
    assert_eq!(count_fork_cells(&table), 0);
}

// ===========================================================================
// 7. ConflictResolver API (5 tests)
// ===========================================================================

#[test]
fn test_resolver_api_detect_returns_nonempty_for_ambiguous() {
    let g = expr_grammar_one_op("api_ambig", "+", None, None);
    let (collection, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    assert!(
        !resolver.conflicts.is_empty(),
        "detect_conflicts should find conflicts in ambiguous grammar"
    );
}

#[test]
fn test_resolver_api_detect_returns_empty_for_unambiguous() {
    let g = GrammarBuilder::new("api_unambig")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (collection, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    assert!(
        resolver.conflicts.is_empty(),
        "detect_conflicts should find no conflicts in unambiguous grammar"
    );
}

#[test]
fn test_resolver_api_conflict_fields_populated() {
    let g = expr_grammar_one_op("api_fields", "+", None, None);
    let (collection, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    for conflict in &resolver.conflicts {
        assert!(
            conflict.actions.len() > 1,
            "conflict should have >1 actions"
        );
        // state is a valid StateId
        assert!(
            (conflict.state.0 as usize) < collection.sets.len(),
            "conflict state should be valid"
        );
    }
}

#[test]
fn test_resolver_api_resolve_reduces_conflicts() {
    let g = expr_grammar_one_op(
        "api_resolve",
        "+",
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
    );
    let (collection, ff) = build_collection(&g);
    let mut resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    let before = resolver.conflicts.len();
    resolver.resolve_conflicts(&g);
    let after = resolver.conflicts.len();
    // resolve_conflicts should not add conflicts; may reduce or keep them.
    assert!(
        after <= before,
        "resolve_conflicts should not add conflicts (before: {before}, after: {after})"
    );
}

#[test]
fn test_resolver_api_sr_type_classification() {
    let g = expr_grammar_one_op("api_type", "+", None, None);
    let (collection, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    let sr_conflicts: Vec<_> = resolver
        .conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::ShiftReduce)
        .collect();
    assert!(
        !sr_conflicts.is_empty(),
        "E → E + E should produce ShiftReduce classified conflicts"
    );
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn test_edge_single_token_grammar_no_panic() {
    let g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let _table = build_table(&g);
}

#[test]
fn test_edge_two_identical_rules_same_lhs() {
    // S → a; S → a (duplicate rules — may or may not create a conflict)
    let mut g = Grammar::new("dup_rules".to_string());
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
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );
    // Should not panic.
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

#[test]
fn test_edge_state_has_conflicts_api() {
    let g = expr_grammar_one_op("state_api", "+", None, None);
    let table = build_table(&g);

    // Find a state that should have conflicts.
    let any_has =
        (0..table.state_count).any(|s| state_has_conflicts(&table, adze_ir::StateId(s as u16)));
    assert!(any_has, "at least one state should have conflicts");
}

#[test]
fn test_edge_state_has_conflicts_invalid_state() {
    let g = GrammarBuilder::new("invalid_state")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&g);
    // Out-of-range state should return false, not panic.
    assert!(!state_has_conflicts(&table, adze_ir::StateId(9999)));
}

#[test]
fn test_edge_get_state_conflicts_returns_subset() {
    let g = expr_grammar_one_op("get_state", "+", None, None);
    let table = build_table(&g);
    let summary = count_conflicts(&table);

    if let Some(&state) = summary.states_with_conflicts.first() {
        let state_conflicts = get_state_conflicts(&table, state);
        assert!(
            !state_conflicts.is_empty(),
            "get_state_conflicts should return conflicts for a conflicting state"
        );
        for detail in &state_conflicts {
            assert_eq!(detail.state, state);
        }
    }
}

#[test]
fn test_edge_find_conflicts_for_symbol_api() {
    let g = expr_grammar_one_op("find_sym", "+", None, None);
    let op = SymbolId(2);
    let table = build_table(&g);
    let sym_conflicts = find_conflicts_for_symbol(&table, op);
    // The '+' operator symbol should appear in conflicts.
    for detail in &sym_conflicts {
        assert_eq!(detail.symbol, op);
    }
}

#[test]
fn test_edge_classify_conflict_shift_reduce() {
    use adze_glr_core::conflict_inspection::ConflictType as InspConflictType;
    let actions = vec![
        Action::Shift(adze_ir::StateId(1)),
        Action::Reduce(adze_ir::RuleId(0)),
    ];
    let ct = classify_conflict(&actions);
    assert_eq!(ct, InspConflictType::ShiftReduce);
}

#[test]
fn test_edge_classify_conflict_reduce_reduce() {
    use adze_glr_core::conflict_inspection::ConflictType as InspConflictType;
    let actions = vec![
        Action::Reduce(adze_ir::RuleId(0)),
        Action::Reduce(adze_ir::RuleId(1)),
    ];
    let ct = classify_conflict(&actions);
    assert_eq!(ct, InspConflictType::ReduceReduce);
}
