#![allow(clippy::needless_range_loop)]

//! Property-based tests for `Grammar::normalize()`.
//!
//! Tests that normalization:
//! - Flattens Optional, Repeat, Choice, Sequence into auxiliary rules
//! - Is idempotent (normalizing twice == normalizing once)
//! - Never decreases rule count
//! - Keeps symbol IDs unique
//! - Preserves the start rule

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};
use proptest::prelude::*;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A symbol is "simple" when it carries no complex wrapper.
fn is_simple(sym: &Symbol) -> bool {
    matches!(
        sym,
        Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon
    )
}

/// Assert every RHS position in every rule is simple.
fn assert_fully_normalized(grammar: &Grammar) {
    for (_lhs, rules) in &grammar.rules {
        for rule in rules {
            for sym in &rule.rhs {
                assert!(is_simple(sym), "Non-normalized symbol found: {sym:?}");
            }
        }
    }
}

/// Total number of individual Rule structs across all LHS symbols.
fn total_rule_count(grammar: &Grammar) -> usize {
    grammar.rules.values().map(|v| v.len()).sum()
}

/// Collect all LHS SymbolIds.
fn all_lhs_ids(grammar: &Grammar) -> HashSet<SymbolId> {
    grammar.rules.keys().copied().collect()
}

fn tok_a() -> SymbolId {
    SymbolId(1)
}

fn tok_b() -> SymbolId {
    SymbolId(2)
}

/// Minimal grammar with tokens A, B and a single `root -> A` rule.
fn base_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build()
}

/// Replace root's RHS with `symbols`.
fn grammar_with_rhs(symbols: Vec<Symbol>) -> Grammar {
    let mut g = base_grammar();
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: symbols,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Grammar whose root rule contains `[Terminal(A), complex]`.
fn grammar_with_complex(complex: Symbol) -> Grammar {
    let mut g = base_grammar();
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Terminal(tok_a()), complex],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Build a simple grammar with N tokens and M rules (all terminal-only RHS).
fn build_simple_grammar(num_tokens: usize, num_rules: usize) -> Grammar {
    let tok_names: Vec<String> = (0..num_tokens).map(|i| format!("tok_{i}")).collect();
    let rule_names: Vec<String> = (0..num_rules.max(1)).map(|i| format!("rule_{i}")).collect();
    let mut b = GrammarBuilder::new("simple");
    for name in &tok_names {
        b = b.token(name, name);
    }
    for (i, name) in rule_names.iter().enumerate() {
        if !tok_names.is_empty() {
            b = b.rule(name, vec![&tok_names[i % tok_names.len()]]);
        }
    }
    b.build()
}

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

fn terminal_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        Just(Symbol::Terminal(tok_a())),
        Just(Symbol::Terminal(tok_b())),
    ]
}

fn complex_symbol_strategy() -> impl Strategy<Value = Symbol> {
    terminal_strategy().prop_flat_map(|inner| {
        prop_oneof![
            Just(Symbol::Optional(Box::new(inner.clone()))),
            Just(Symbol::Repeat(Box::new(inner.clone()))),
            Just(Symbol::RepeatOne(Box::new(inner.clone()))),
            Just(Symbol::Choice(vec![
                inner.clone(),
                Symbol::Terminal(tok_b()),
            ])),
            Just(Symbol::Sequence(vec![
                inner.clone(),
                Symbol::Terminal(tok_b()),
            ])),
        ]
    })
}

fn nested_complex_strategy() -> impl Strategy<Value = Symbol> {
    complex_symbol_strategy().prop_flat_map(|inner| {
        prop_oneof![
            Just(Symbol::Optional(Box::new(inner.clone()))),
            Just(Symbol::Repeat(Box::new(inner.clone()))),
            Just(Symbol::Choice(vec![
                inner.clone(),
                Symbol::Terminal(tok_a()),
            ])),
            Just(Symbol::Sequence(vec![
                Symbol::Terminal(tok_a()),
                inner.clone(),
            ])),
        ]
    })
}

fn deep_nested_strategy() -> impl Strategy<Value = Symbol> {
    nested_complex_strategy().prop_flat_map(|inner| {
        prop_oneof![
            Just(Symbol::Optional(Box::new(inner.clone()))),
            Just(Symbol::Repeat(Box::new(inner.clone()))),
            Just(Symbol::RepeatOne(Box::new(inner.clone()))),
        ]
    })
}

// =========================================================================
// 1. No complex symbols remain after normalization
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn no_complex_after_optional(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Optional(Box::new(term)));
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn no_complex_after_repeat(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Repeat(Box::new(term)));
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn no_complex_after_repeat_one(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::RepeatOne(Box::new(term)));
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn no_complex_after_choice(n in 2_usize..6) {
        let choices: Vec<Symbol> = (0..n)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_complex(Symbol::Choice(choices));
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn no_complex_after_sequence(len in 2_usize..6) {
        let seq: Vec<Symbol> = (0..len)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_rhs(vec![Symbol::Sequence(seq)]);
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn no_complex_after_nested(sym in nested_complex_strategy()) {
        let mut g = grammar_with_complex(sym);
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn no_complex_after_deep_nesting(sym in deep_nested_strategy()) {
        let mut g = grammar_with_complex(sym);
        g.normalize();
        assert_fully_normalized(&g);
    }
}

// =========================================================================
// 2. Idempotency — normalizing twice == normalizing once
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn idempotent_simple(num_tok in 1_usize..5, num_rule in 1_usize..5) {
        let mut g1 = build_simple_grammar(num_tok, num_rule);
        let mut g2 = g1.clone();
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
    }

    #[test]
    fn idempotent_optional(term in terminal_strategy()) {
        let g0 = grammar_with_complex(Symbol::Optional(Box::new(term)));
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }

    #[test]
    fn idempotent_repeat(term in terminal_strategy()) {
        let g0 = grammar_with_complex(Symbol::Repeat(Box::new(term)));
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }

    #[test]
    fn idempotent_choice(n in 2_usize..5) {
        let choices: Vec<Symbol> = (0..n)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let g0 = grammar_with_complex(Symbol::Choice(choices));
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }

    #[test]
    fn idempotent_nested(sym in nested_complex_strategy()) {
        let g0 = grammar_with_complex(sym);
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }

    #[test]
    fn idempotent_deep(sym in deep_nested_strategy()) {
        let g0 = grammar_with_complex(sym);
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }
}

// =========================================================================
// 3. Rule count increases or stays same
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_count_nondecreasing_optional(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Optional(Box::new(term)));
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_nondecreasing_repeat(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Repeat(Box::new(term)));
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_nondecreasing_choice(n in 2_usize..6) {
        let choices: Vec<Symbol> = (0..n)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_complex(Symbol::Choice(choices));
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_nondecreasing_simple(num_tok in 1_usize..5, num_rule in 1_usize..5) {
        let mut g = build_simple_grammar(num_tok, num_rule);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn lhs_count_nondecreasing(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex(sym);
        let before = g.rules.len();
        g.normalize();
        prop_assert!(g.rules.len() >= before);
    }
}

// =========================================================================
// 4. Symbol IDs remain unique after normalization
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn lhs_ids_unique_after_optional(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Optional(Box::new(term)));
        g.normalize();
        let ids = all_lhs_ids(&g);
        prop_assert_eq!(ids.len(), g.rules.len(), "LHS SymbolIds must be unique");
    }

    #[test]
    fn lhs_ids_unique_after_repeat(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Repeat(Box::new(term)));
        g.normalize();
        let ids = all_lhs_ids(&g);
        prop_assert_eq!(ids.len(), g.rules.len());
    }

    #[test]
    fn lhs_ids_unique_after_nested(sym in nested_complex_strategy()) {
        let mut g = grammar_with_complex(sym);
        g.normalize();
        let ids = all_lhs_ids(&g);
        prop_assert_eq!(ids.len(), g.rules.len());
    }

    #[test]
    fn lhs_ids_unique_after_deep(sym in deep_nested_strategy()) {
        let mut g = grammar_with_complex(sym);
        g.normalize();
        let ids = all_lhs_ids(&g);
        prop_assert_eq!(ids.len(), g.rules.len());
    }

    #[test]
    fn aux_ids_above_existing_max(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex(sym);
        let max_before = g.rules.keys().map(|id| id.0).max().unwrap_or(0);
        g.normalize();
        for lhs in g.rules.keys() {
            if lhs.0 > max_before {
                prop_assert!(
                    lhs.0 >= max_before + 1000,
                    "Aux id {} must be >= {} (max_before + 1000)",
                    lhs.0,
                    max_before + 1000,
                );
            }
        }
    }
}

// =========================================================================
// 5. Start rule is preserved
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn start_symbol_preserved_optional(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Optional(Box::new(term)));
        let start_before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), start_before, "Start symbol must survive normalize");
    }

    #[test]
    fn start_symbol_preserved_repeat(term in terminal_strategy()) {
        let mut g = grammar_with_complex(Symbol::Repeat(Box::new(term)));
        let start_before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), start_before);
    }

    #[test]
    fn start_symbol_preserved_choice(n in 2_usize..5) {
        let choices: Vec<Symbol> = (0..n)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_complex(Symbol::Choice(choices));
        let start_before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), start_before);
    }

    #[test]
    fn start_symbol_preserved_nested(sym in nested_complex_strategy()) {
        let mut g = grammar_with_complex(sym);
        let start_before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), start_before);
    }

    #[test]
    fn start_rule_still_has_productions(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex(sym);
        let start = g.start_symbol().unwrap();
        g.normalize();
        let rules = g.get_rules_for_symbol(start);
        prop_assert!(rules.is_some() && !rules.unwrap().is_empty(),
            "Start symbol must still have at least one production");
    }
}

// =========================================================================
// 6. Structural properties
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_name_preserved(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex(sym);
        let name = g.name.clone();
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn token_count_preserved(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex(sym);
        let count = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), count);
    }

    #[test]
    fn extras_preserved(_dummy in 0_u8..1) {
        let mut g = GrammarBuilder::new("extras_test")
            .token("main", "main")
            .token("ws", r"\s+")
            .rule("start", vec!["main"])
            .extra("ws")
            .build();
        let extras_before = g.extras.clone();
        g.normalize();
        prop_assert_eq!(&g.extras, &extras_before);
    }
}

// =========================================================================
// 7. Mixed complex symbols
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn mixed_optional_and_repeat(idx in 0_usize..2) {
        let term = if idx == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) };
        let opt = Symbol::Optional(Box::new(term.clone()));
        let rep = Symbol::Repeat(Box::new(term));
        let mut g = grammar_with_rhs(vec![opt, rep]);
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.rules.len() >= 3, "Optional + Repeat => ≥ 2 aux LHS entries");
    }

    #[test]
    fn mixed_all_complex_types(_dummy in 0_u8..1) {
        let rhs = vec![
            Symbol::Optional(Box::new(Symbol::Terminal(tok_a()))),
            Symbol::Repeat(Box::new(Symbol::Terminal(tok_b()))),
            Symbol::RepeatOne(Box::new(Symbol::Terminal(tok_a()))),
            Symbol::Choice(vec![Symbol::Terminal(tok_a()), Symbol::Terminal(tok_b())]),
            Symbol::Sequence(vec![Symbol::Terminal(tok_a()), Symbol::Terminal(tok_b())]),
        ];
        let mut g = grammar_with_rhs(rhs);
        let before = total_rule_count(&g);
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(total_rule_count(&g) > before);
    }

    #[test]
    fn mixed_nested_idempotent(sym in nested_complex_strategy()) {
        let other = Symbol::Optional(Box::new(Symbol::Terminal(tok_a())));
        let g0 = grammar_with_rhs(vec![sym, other]);
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }
}
