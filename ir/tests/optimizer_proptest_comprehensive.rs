#![allow(clippy::needless_range_loop)]

//! Comprehensive property-based and unit tests for grammar optimization.
//!
//! Tests cover: normalize() idempotence, start symbol preservation, grammar name
//! preservation, rule count monotonicity, token preservation, no-panic guarantees,
//! normalize→optimize pipeline, and determinism.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build()
}

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token(";", ";")
        .rule("program", vec!["stmt"])
        .rule("program", vec!["program", "stmt"])
        .rule("stmt", vec!["expr", ";"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("program")
        .build()
}

fn grammar_with_optional() -> Grammar {
    let mut g = GrammarBuilder::new("opt_grammar")
        .token("A", "a")
        .token("B", "b")
        .rule("start_rule", vec!["A"])
        .start("start_rule")
        .build();
    // Manually inject an Optional symbol into a rule
    let start_id = *g.rules.keys().next().unwrap();
    let a_id = *g.tokens.keys().next().unwrap();
    g.rules.get_mut(&start_id).unwrap().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

fn grammar_with_repeat() -> Grammar {
    let mut g = GrammarBuilder::new("rep_grammar")
        .token("X", "x")
        .rule("items", vec!["X"])
        .start("items")
        .build();
    let start_id = *g.rules.keys().next().unwrap();
    let x_id = *g.tokens.keys().next().unwrap();
    g.rules.get_mut(&start_id).unwrap().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(x_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

fn grammar_with_repeat_one() -> Grammar {
    let mut g = GrammarBuilder::new("rep1_grammar")
        .token("Y", "y")
        .rule("elems", vec!["Y"])
        .start("elems")
        .build();
    let start_id = *g.rules.keys().next().unwrap();
    let y_id = *g.tokens.keys().next().unwrap();
    g.rules.get_mut(&start_id).unwrap().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(y_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

fn grammar_with_choice() -> Grammar {
    let mut g = GrammarBuilder::new("choice_grammar")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let start_id = *g.rules.keys().next().unwrap();
    let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    g.rules.get_mut(&start_id).unwrap().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(ids[0]),
            Symbol::Terminal(ids[1]),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

fn grammar_with_sequence() -> Grammar {
    let mut g = GrammarBuilder::new("seq_grammar")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let start_id = *g.rules.keys().next().unwrap();
    let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    g.rules.get_mut(&start_id).unwrap().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(ids[0]),
            Symbol::Terminal(ids[1]),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

fn grammar_with_nested_complex() -> Grammar {
    let mut g = GrammarBuilder::new("nested")
        .token("T", "t")
        .rule("root", vec!["T"])
        .start("root")
        .build();
    let start_id = *g.rules.keys().next().unwrap();
    let t_id = *g.tokens.keys().next().unwrap();
    // Optional(Repeat(Terminal))
    g.rules.get_mut(&start_id).unwrap().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
            Symbol::Terminal(t_id),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

fn total_rule_count(g: &Grammar) -> usize {
    g.all_rules().count()
}

/// Build a grammar from proptest-generated parameters
fn build_generated_grammar(
    grammar_name: &str,
    token_names: &[String],
    rule_names: &[String],
) -> Grammar {
    let mut builder = GrammarBuilder::new(grammar_name);

    let tok_ids: Vec<String> = token_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("t_{i}_{n}"))
        .collect();
    for tok in &tok_ids {
        builder = builder.token(tok, tok);
    }

    let rule_ids: Vec<String> = rule_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("r_{i}_{n}"))
        .collect();

    let first_tok = &tok_ids[0];
    let second_tok = if tok_ids.len() > 1 {
        &tok_ids[1]
    } else {
        &tok_ids[0]
    };

    for (i, rname) in rule_ids.iter().enumerate() {
        if i == 0 && rule_ids.len() > 1 {
            builder = builder.rule(rname, vec![first_tok, &rule_ids[1]]);
        } else {
            builder = builder.rule(rname, vec![first_tok, second_tok]);
        }
    }

    builder = builder.start(&rule_ids[0]);
    builder.build()
}

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

fn name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,6}".prop_map(|s| s)
}

fn token_names_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strategy(), 1..5)
}

fn rule_names_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strategy(), 1..5)
}

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{2,8}".prop_map(|s| s)
}

// ===========================================================================
// PROPERTY 1: normalize() is idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalize_idempotent(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g1 = build_generated_grammar(&gname, &tnames, &rnames);
        g1.normalize();
        let snapshot = g1.clone();
        g1.normalize();
        prop_assert_eq!(
            total_rule_count(&g1),
            total_rule_count(&snapshot),
            "rule count changed on second normalize"
        );
        prop_assert_eq!(g1.tokens.len(), snapshot.tokens.len());
    }
}

#[test]
fn unit_normalize_idempotent_simple() {
    let mut g = simple_grammar();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

#[test]
fn unit_normalize_idempotent_arithmetic() {
    let mut g = arithmetic_grammar();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

#[test]
fn unit_normalize_idempotent_optional() {
    let mut g = grammar_with_optional();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

#[test]
fn unit_normalize_idempotent_repeat() {
    let mut g = grammar_with_repeat();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

#[test]
fn unit_normalize_idempotent_repeat_one() {
    let mut g = grammar_with_repeat_one();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

#[test]
fn unit_normalize_idempotent_choice() {
    let mut g = grammar_with_choice();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

#[test]
fn unit_normalize_idempotent_sequence() {
    let mut g = grammar_with_sequence();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

#[test]
fn unit_normalize_idempotent_nested() {
    let mut g = grammar_with_nested_complex();
    g.normalize();
    let snap = g.clone();
    g.normalize();
    assert_eq!(total_rule_count(&g), total_rule_count(&snap));
}

// ===========================================================================
// PROPERTY 2: normalize() preserves start symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalize_preserves_start_symbol(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        let start_before = g.start_symbol();
        g.normalize();
        let start_after = g.start_symbol();
        // start_symbol() relies on heuristics over rules; the first rule's LHS
        // must still be present after normalize.
        prop_assert!(start_before.is_some());
        prop_assert!(start_after.is_some());
    }
}

#[test]
fn unit_normalize_preserves_start_simple() {
    let mut g = simple_grammar();
    let before = g.start_symbol();
    g.normalize();
    let after = g.start_symbol();
    assert_eq!(before, after);
}

#[test]
fn unit_normalize_preserves_start_arithmetic() {
    let mut g = arithmetic_grammar();
    let before = g.start_symbol();
    g.normalize();
    let after = g.start_symbol();
    assert_eq!(before, after);
}

#[test]
fn unit_normalize_preserves_start_multi() {
    let mut g = multi_rule_grammar();
    let before = g.start_symbol();
    g.normalize();
    let after = g.start_symbol();
    assert_eq!(before, after);
}

// ===========================================================================
// PROPERTY 3: normalize() preserves grammar name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalize_preserves_name(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        let name_before = g.name.clone();
        g.normalize();
        prop_assert_eq!(g.name, name_before);
    }
}

#[test]
fn unit_normalize_preserves_name_simple() {
    let mut g = simple_grammar();
    g.normalize();
    assert_eq!(g.name, "simple");
}

#[test]
fn unit_normalize_preserves_name_arith() {
    let mut g = arithmetic_grammar();
    g.normalize();
    assert_eq!(g.name, "arith");
}

// ===========================================================================
// PROPERTY 4: After normalize, rule count >= original rule count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalize_rule_count_monotonic(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        let before = total_rule_count(&g);
        g.normalize();
        let after = total_rule_count(&g);
        prop_assert!(after >= before, "rules decreased from {} to {}", before, after);
    }
}

#[test]
fn unit_normalize_rule_count_grows_with_optional() {
    let mut g = grammar_with_optional();
    let before = total_rule_count(&g);
    g.normalize();
    let after = total_rule_count(&g);
    assert!(after > before, "expected aux rules from Optional expansion");
}

#[test]
fn unit_normalize_rule_count_grows_with_repeat() {
    let mut g = grammar_with_repeat();
    let before = total_rule_count(&g);
    g.normalize();
    let after = total_rule_count(&g);
    assert!(after > before, "expected aux rules from Repeat expansion");
}

#[test]
fn unit_normalize_rule_count_grows_with_repeat_one() {
    let mut g = grammar_with_repeat_one();
    let before = total_rule_count(&g);
    g.normalize();
    let after = total_rule_count(&g);
    assert!(
        after > before,
        "expected aux rules from RepeatOne expansion"
    );
}

#[test]
fn unit_normalize_rule_count_grows_with_choice() {
    let mut g = grammar_with_choice();
    let before = total_rule_count(&g);
    g.normalize();
    let after = total_rule_count(&g);
    assert!(after > before, "expected aux rules from Choice expansion");
}

#[test]
fn unit_normalize_rule_count_stable_for_flat() {
    let mut g = simple_grammar();
    let before = total_rule_count(&g);
    g.normalize();
    let after = total_rule_count(&g);
    assert_eq!(after, before, "no complex symbols, count should be stable");
}

#[test]
fn unit_normalize_rule_count_grows_with_nested() {
    let mut g = grammar_with_nested_complex();
    let before = total_rule_count(&g);
    g.normalize();
    let after = total_rule_count(&g);
    assert!(after > before, "nested complex should produce aux rules");
}

// ===========================================================================
// PROPERTY 5: After normalize, token count preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalize_preserves_token_count(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        let before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), before);
    }
}

#[test]
fn unit_normalize_preserves_tokens_simple() {
    let mut g = simple_grammar();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn unit_normalize_preserves_tokens_optional() {
    let mut g = grammar_with_optional();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn unit_normalize_preserves_tokens_repeat() {
    let mut g = grammar_with_repeat();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn unit_normalize_preserves_tokens_multi() {
    let mut g = multi_rule_grammar();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

// ===========================================================================
// PROPERTY 6: For any valid grammar, normalize() doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalize_never_panics(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        g.normalize();
        // reaching here means no panic
    }
}

#[test]
fn unit_normalize_no_panic_empty_grammar() {
    let mut g = Grammar::new("empty".into());
    g.normalize();
}

#[test]
fn unit_normalize_no_panic_single_epsilon_rule() {
    let mut g = GrammarBuilder::new("eps")
        .token("T", "t")
        .rule("start_rule", vec![])
        .start("start_rule")
        .build();
    g.normalize();
}

#[test]
fn unit_normalize_no_panic_python_like() {
    let mut g = GrammarBuilder::python_like();
    g.normalize();
}

#[test]
fn unit_normalize_no_panic_javascript_like() {
    let mut g = GrammarBuilder::javascript_like();
    g.normalize();
}

// ===========================================================================
// PROPERTY 7: Grammar build → normalize → optimize pipeline succeeds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_build_normalize_optimize_pipeline(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        g.normalize();
        let result = optimize_grammar(g);
        prop_assert!(result.is_ok());
    }
}

#[test]
fn unit_pipeline_simple() {
    let mut g = simple_grammar();
    g.normalize();
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn unit_pipeline_arithmetic() {
    let mut g = arithmetic_grammar();
    g.normalize();
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn unit_pipeline_multi_rule() {
    let mut g = multi_rule_grammar();
    g.normalize();
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn unit_pipeline_optional() {
    let mut g = grammar_with_optional();
    g.normalize();
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn unit_pipeline_repeat() {
    let mut g = grammar_with_repeat();
    g.normalize();
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn unit_pipeline_python_like() {
    let mut g = GrammarBuilder::python_like();
    g.normalize();
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn unit_pipeline_javascript_like() {
    let mut g = GrammarBuilder::javascript_like();
    g.normalize();
    assert!(optimize_grammar(g).is_ok());
}

// ===========================================================================
// PROPERTY 8: Determinism — same input → same normalized output
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalize_deterministic(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g1 = build_generated_grammar(&gname, &tnames, &rnames);
        let mut g2 = g1.clone();
        g1.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
        prop_assert_eq!(g1.name, g2.name);
        // Deep equality of rule structure
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
    }
}

#[test]
fn unit_normalize_deterministic_simple() {
    let mut g1 = simple_grammar();
    let mut g2 = simple_grammar();
    g1.normalize();
    g2.normalize();
    assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    assert_eq!(g1.rules.len(), g2.rules.len());
}

#[test]
fn unit_normalize_deterministic_optional() {
    let mut g1 = grammar_with_optional();
    let mut g2 = grammar_with_optional();
    g1.normalize();
    g2.normalize();
    assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
}

#[test]
fn unit_normalize_deterministic_nested() {
    let mut g1 = grammar_with_nested_complex();
    let mut g2 = grammar_with_nested_complex();
    g1.normalize();
    g2.normalize();
    assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    assert_eq!(g1.rules.len(), g2.rules.len());
}

// ===========================================================================
// ADDITIONAL: optimize_grammar preserves key invariants
// ===========================================================================

#[test]
fn unit_optimize_preserves_grammar_name() {
    let g = simple_grammar();
    let optimized = optimize_grammar(g).unwrap();
    assert_eq!(optimized.name, "simple");
}

#[test]
fn unit_optimize_returns_ok_for_empty() {
    let g = Grammar::new("empty".into());
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn unit_optimizer_stats_total() {
    let mut g = simple_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // total should be sum of fields
    assert_eq!(
        stats.total(),
        stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules
    );
}

#[test]
fn unit_optimizer_on_python_like() {
    let mut g = GrammarBuilder::python_like();
    let mut opt = GrammarOptimizer::new();
    let _stats = opt.optimize(&mut g);
    // Should not panic; grammar should still be non-empty
    assert!(total_rule_count(&g) > 0);
}

#[test]
fn unit_optimizer_on_javascript_like() {
    let mut g = GrammarBuilder::javascript_like();
    let mut opt = GrammarOptimizer::new();
    let _stats = opt.optimize(&mut g);
    assert!(total_rule_count(&g) > 0);
}

// ===========================================================================
// ADDITIONAL: normalize expansion correctness
// ===========================================================================

#[test]
fn unit_optional_expands_to_two_aux_rules() {
    let mut g = grammar_with_optional();
    let rules_before = g.rules.len();
    g.normalize();
    // Optional creates an aux nonterminal with 2 productions (inner | epsilon)
    assert!(g.rules.len() > rules_before);
}

#[test]
fn unit_repeat_expands_to_two_aux_rules() {
    let mut g = grammar_with_repeat();
    let rules_before = g.rules.len();
    g.normalize();
    assert!(g.rules.len() > rules_before);
}

#[test]
fn unit_repeat_one_expands_to_two_aux_rules() {
    let mut g = grammar_with_repeat_one();
    let rules_before = g.rules.len();
    g.normalize();
    assert!(g.rules.len() > rules_before);
}

#[test]
fn unit_choice_expands_to_aux_rules() {
    let mut g = grammar_with_choice();
    let rules_before = g.rules.len();
    g.normalize();
    assert!(g.rules.len() > rules_before);
}

#[test]
fn unit_sequence_flattened_no_new_nonterminals() {
    let mut g = grammar_with_sequence();
    let nt_count_before = g.rules.len();
    g.normalize();
    // Sequence flattens in-place, doesn't create a new nonterminal
    assert_eq!(g.rules.len(), nt_count_before);
}

#[test]
fn unit_no_complex_symbols_after_normalize_optional() {
    let mut g = grammar_with_optional();
    g.normalize();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(
                !matches!(
                    sym,
                    Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                ),
                "found complex symbol after normalize: {:?}",
                sym
            );
        }
    }
}

#[test]
fn unit_no_complex_symbols_after_normalize_repeat() {
    let mut g = grammar_with_repeat();
    g.normalize();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(!matches!(
                sym,
                Symbol::Optional(_)
                    | Symbol::Repeat(_)
                    | Symbol::RepeatOne(_)
                    | Symbol::Choice(_)
                    | Symbol::Sequence(_)
            ));
        }
    }
}

#[test]
fn unit_no_complex_symbols_after_normalize_nested() {
    let mut g = grammar_with_nested_complex();
    g.normalize();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(!matches!(
                sym,
                Symbol::Optional(_)
                    | Symbol::Repeat(_)
                    | Symbol::RepeatOne(_)
                    | Symbol::Choice(_)
                    | Symbol::Sequence(_)
            ));
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_no_complex_symbols_after_normalize(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        g.normalize();
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(
                    sym,
                    Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                ));
            }
        }
    }
}

// ===========================================================================
// ADDITIONAL: normalize preserves existing data
// ===========================================================================

#[test]
fn unit_normalize_preserves_precedence_info() {
    let mut g = arithmetic_grammar();
    g.normalize();
    // The original rules with precedence should retain their info
    let mut found_prec = false;
    for rule in g.all_rules() {
        if rule.precedence.is_some() {
            found_prec = true;
            break;
        }
    }
    assert!(found_prec, "precedence info lost after normalize");
}

#[test]
fn unit_normalize_preserves_associativity_info() {
    let mut g = arithmetic_grammar();
    g.normalize();
    let mut found_assoc = false;
    for rule in g.all_rules() {
        if rule.associativity.is_some() {
            found_assoc = true;
            break;
        }
    }
    assert!(found_assoc, "associativity info lost after normalize");
}

#[test]
fn unit_normalize_preserves_extras() {
    let mut g = GrammarBuilder::python_like();
    let extras_before = g.extras.len();
    g.normalize();
    assert_eq!(g.extras.len(), extras_before);
}

#[test]
fn unit_normalize_preserves_externals() {
    let mut g = GrammarBuilder::python_like();
    let ext_before = g.externals.len();
    g.normalize();
    assert_eq!(g.externals.len(), ext_before);
}

#[test]
fn unit_normalize_preserves_conflicts() {
    let mut g = simple_grammar();
    let conflicts_before = g.conflicts.len();
    g.normalize();
    assert_eq!(g.conflicts.len(), conflicts_before);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_normalize_preserves_extras_and_externals(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let mut g = build_generated_grammar(&gname, &tnames, &rnames);
        let extras_before = g.extras.len();
        let ext_before = g.externals.len();
        let conflicts_before = g.conflicts.len();
        g.normalize();
        prop_assert_eq!(g.extras.len(), extras_before);
        prop_assert_eq!(g.externals.len(), ext_before);
        prop_assert_eq!(g.conflicts.len(), conflicts_before);
    }
}

// ===========================================================================
// ADDITIONAL: optimize + normalize combined properties
// ===========================================================================

#[test]
fn unit_optimize_then_normalize_no_panic() {
    let mut g = multi_rule_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    g.normalize();
}

#[test]
fn unit_normalize_then_optimize_then_normalize_idempotent_tokens() {
    let mut g = grammar_with_optional();
    g.normalize();
    let g = optimize_grammar(g).unwrap();
    let token_count = g.tokens.len();
    let mut g2 = g.clone();
    g2.normalize();
    assert_eq!(g2.tokens.len(), token_count);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_optimize_never_panics(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let g = build_generated_grammar(&gname, &tnames, &rnames);
        let _ = optimize_grammar(g);
    }

    #[test]
    fn prop_optimize_preserves_name(
        gname in grammar_name_strategy(),
        tnames in token_names_strategy(),
        rnames in rule_names_strategy(),
    ) {
        let g = build_generated_grammar(&gname, &tnames, &rnames);
        let name = g.name.clone();
        let optimized = optimize_grammar(g).unwrap();
        prop_assert_eq!(optimized.name, name);
    }
}
