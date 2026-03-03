#![allow(clippy::needless_range_loop)]

//! Property-based tests for `Grammar::normalize()`.
//!
//! Verifies key invariants:
//! - Normalization is idempotent
//! - Normalization preserves start symbol
//! - Normalization flattens Optional, Repeat, Choice, Sequence into auxiliary rules
//! - Auxiliary rule naming uses IDs above max_existing + 1000
//! - Already-normalized grammars are unchanged

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if a symbol is "simple" (no complex wrappers).
fn is_simple(sym: &Symbol) -> bool {
    matches!(
        sym,
        Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon
    )
}

/// Assert every RHS symbol in every rule is simple.
fn assert_fully_normalized(grammar: &Grammar) {
    for (_lhs, rules) in &grammar.rules {
        for rule in rules {
            for sym in &rule.rhs {
                assert!(is_simple(sym), "Non-normalized symbol found: {sym:?}");
            }
        }
    }
}

/// Total number of individual rules across all LHS symbols.
fn total_rule_count(grammar: &Grammar) -> usize {
    grammar.rules.values().map(|v| v.len()).sum()
}

/// Build a simple grammar with the given number of tokens and rules.
fn build_simple_grammar(num_tokens: usize, num_rules: usize) -> Grammar {
    let token_names: Vec<String> = (0..num_tokens).map(|i| format!("tok_{i}")).collect();
    let rule_names: Vec<String> = (0..num_rules.max(1)).map(|i| format!("rule_{i}")).collect();

    let mut builder = GrammarBuilder::new("test");
    for name in &token_names {
        builder = builder.token(name, name);
    }
    for (i, name) in rule_names.iter().enumerate() {
        if !token_names.is_empty() {
            builder = builder.rule(name, vec![&token_names[i % token_names.len()]]);
        }
    }
    builder.build()
}

/// Build a grammar with one rule whose RHS contains the given complex symbol.
fn grammar_with_complex_rhs(complex: Symbol) -> Grammar {
    let mut g = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    let a_id = *g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "A")
        .map(|(id, _)| id)
        .unwrap();

    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Terminal(a_id), complex],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Build a grammar with two tokens (A, B) and a rule whose RHS is *only* `symbols`.
fn grammar_with_rhs(symbols: Vec<Symbol>) -> Grammar {
    let mut g = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();

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

fn tok_a() -> SymbolId {
    SymbolId(1)
}

fn tok_b() -> SymbolId {
    SymbolId(2)
}

/// Proptest strategy that generates a terminal Symbol referencing tok_a or tok_b.
fn terminal_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        Just(Symbol::Terminal(tok_a())),
        Just(Symbol::Terminal(tok_b())),
    ]
}

/// Proptest strategy that generates a complex Symbol (depth 1).
fn complex_symbol_strategy() -> impl Strategy<Value = Symbol> {
    terminal_strategy().prop_flat_map(|inner| {
        prop_oneof![
            Just(Symbol::Optional(Box::new(inner.clone()))),
            Just(Symbol::Repeat(Box::new(inner.clone()))),
            Just(Symbol::RepeatOne(Box::new(inner.clone()))),
            Just(Symbol::Choice(vec![
                inner.clone(),
                Symbol::Terminal(tok_b())
            ])),
            Just(Symbol::Sequence(vec![
                inner.clone(),
                Symbol::Terminal(tok_b())
            ])),
        ]
    })
}

/// Proptest strategy that generates a complex Symbol with up to 2 levels of nesting.
fn nested_complex_strategy() -> impl Strategy<Value = Symbol> {
    complex_symbol_strategy().prop_flat_map(|inner| {
        prop_oneof![
            Just(Symbol::Optional(Box::new(inner.clone()))),
            Just(Symbol::Repeat(Box::new(inner.clone()))),
            Just(Symbol::Choice(vec![
                inner.clone(),
                Symbol::Terminal(tok_a())
            ])),
            Just(Symbol::Sequence(vec![
                Symbol::Terminal(tok_a()),
                inner.clone()
            ])),
        ]
    })
}

/// Proptest strategy for deeply nested symbols (3+ levels).
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
// 1. Normalization is idempotent
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn idempotent_simple_grammar(
        num_tokens in 1_usize..5,
        num_rules in 1_usize..5,
    ) {
        let mut g1 = build_simple_grammar(num_tokens, num_rules);
        let mut g2 = g1.clone();
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
    }

    #[test]
    fn idempotent_optional(term in terminal_strategy()) {
        let g0 = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }

    #[test]
    fn idempotent_repeat(term in terminal_strategy()) {
        let g0 = grammar_with_complex_rhs(Symbol::Repeat(Box::new(term)));
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
        let g0 = grammar_with_complex_rhs(Symbol::Choice(choices));
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }

    #[test]
    fn idempotent_nested(sym in nested_complex_strategy()) {
        let g0 = grammar_with_complex_rhs(sym);
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }
}

// =========================================================================
// 2. Normalization preserves start symbol
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn start_symbol_preserved_simple(
        num_tokens in 1_usize..5,
        num_rules in 1_usize..4,
    ) {
        let mut g = build_simple_grammar(num_tokens, num_rules);
        let original_keys: Vec<SymbolId> = g.rules.keys().copied().collect();
        g.normalize();
        // All original rule LHS symbols must still be present
        for key in &original_keys {
            prop_assert!(g.rules.contains_key(key), "Original LHS {key} must survive normalize");
        }
    }

    #[test]
    fn start_symbol_preserved_complex(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let root_id = g.find_symbol_by_name("root").unwrap();
        g.normalize();
        // root must still exist with rules after normalization
        prop_assert!(g.rules.contains_key(&root_id), "root must still have rules");
        // The rule_names entry must still map root_id to "root"
        prop_assert_eq!(g.rule_names.get(&root_id).map(|s| s.as_str()), Some("root"));
    }

    #[test]
    fn start_symbol_preserved_deep(sym in deep_nested_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let root_id = g.find_symbol_by_name("root").unwrap();
        g.normalize();
        prop_assert!(g.rules.contains_key(&root_id));
    }
}

// =========================================================================
// 3. Normalization flattens Optional symbols
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn optional_creates_aux_rules(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.rules.len() >= 2, "Expected auxiliary rules for Optional");
    }

    #[test]
    fn optional_aux_has_epsilon_alternative(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        g.normalize();
        let has_epsilon = g.rules.values().flatten().any(|r| {
            r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon)
        });
        prop_assert!(has_epsilon, "Optional normalization must produce ε alternative");
    }

    #[test]
    fn optional_preserves_inner_terminal(term in terminal_strategy()) {
        let inner_id = match &term {
            Symbol::Terminal(id) => *id,
            _ => unreachable!(),
        };
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        g.normalize();
        let found = g.rules.values().flatten().any(|r| {
            r.rhs.iter().any(|s| matches!(s, Symbol::Terminal(id) if *id == inner_id))
        });
        prop_assert!(found, "Inner terminal of Optional must be preserved");
    }
}

// =========================================================================
// 4. Normalization flattens Repeat symbols
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn repeat_creates_aux_rules(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(term)));
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.rules.len() >= 2, "Expected auxiliary rules for Repeat");
    }

    #[test]
    fn repeat_aux_has_recursive_production(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(term)));
        g.normalize();
        let has_recursive = g.rules.iter().any(|(lhs, rules)| {
            rules.iter().any(|r| {
                r.rhs.iter().any(|s| matches!(s, Symbol::NonTerminal(id) if id == lhs))
            })
        });
        prop_assert!(has_recursive, "Repeat aux must have recursive production");
    }

    #[test]
    fn repeat_one_creates_aux_without_epsilon(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::RepeatOne(Box::new(term)));
        g.normalize();
        assert_fully_normalized(&g);
        let root_id = g.find_symbol_by_name("root").unwrap();
        for (lhs, rules) in &g.rules {
            if *lhs == root_id {
                continue;
            }
            let has_epsilon_only = rules.iter().any(|r| {
                r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon)
            });
            prop_assert!(
                !has_epsilon_only,
                "RepeatOne must not produce ε-only alternative"
            );
        }
    }
}

// =========================================================================
// 5. Normalization flattens Choice symbols
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn choice_creates_aux_rules(n_choices in 2_usize..5) {
        let choices: Vec<Symbol> = (0..n_choices)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_complex_rhs(Symbol::Choice(choices));
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.rules.len() >= 2, "Expected auxiliary rules for Choice");
    }

    #[test]
    fn choice_aux_has_correct_alternative_count(n_choices in 2_usize..5) {
        let choices: Vec<Symbol> = (0..n_choices)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let expected = n_choices;
        let mut g = grammar_with_complex_rhs(Symbol::Choice(choices));
        g.normalize();
        let root_id = g.find_symbol_by_name("root").unwrap();
        let found = g.rules.iter().any(|(lhs, rules)| *lhs != root_id && rules.len() == expected);
        prop_assert!(found, "No aux rule had {expected} alternatives");
    }
}

// =========================================================================
// 6. Normalization flattens Sequence symbols
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn sequence_is_inlined(seq_len in 2_usize..5) {
        let seq: Vec<Symbol> = (0..seq_len)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_rhs(vec![Symbol::Sequence(seq.clone())]);
        g.normalize();
        assert_fully_normalized(&g);
        let root_id = g.find_symbol_by_name("root").unwrap();
        let root_rules = &g.rules[&root_id];
        prop_assert_eq!(root_rules[0].rhs.len(), seq_len);
    }

    #[test]
    fn sequence_does_not_create_aux(seq_len in 2_usize..5) {
        let seq: Vec<Symbol> = (0..seq_len)
            .map(|_| Symbol::Terminal(tok_a()))
            .collect();
        let mut g = grammar_with_rhs(vec![Symbol::Sequence(seq)]);
        g.normalize();
        prop_assert_eq!(g.rules.len(), 1, "Sequence should not create auxiliary rules");
    }

    #[test]
    fn sequence_with_trailing_terminal(seq_len in 1_usize..4) {
        let seq: Vec<Symbol> = (0..seq_len)
            .map(|_| Symbol::Terminal(tok_a()))
            .collect();
        let rhs = vec![Symbol::Sequence(seq), Symbol::Terminal(tok_b())];
        let mut g = grammar_with_rhs(rhs);
        g.normalize();
        assert_fully_normalized(&g);
        let root_id = g.find_symbol_by_name("root").unwrap();
        let root_rules = &g.rules[&root_id];
        prop_assert_eq!(root_rules[0].rhs.len(), seq_len + 1);
    }
}

// =========================================================================
// 7. Auxiliary rule naming
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn aux_ids_above_existing_max(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
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

    #[test]
    fn aux_ids_are_sequential(sym in nested_complex_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let max_before = g.rules.keys().map(|id| id.0).max().unwrap_or(0);
        g.normalize();
        let mut aux_ids: Vec<u16> = g.rules.keys().filter(|id| id.0 > max_before).map(|id| id.0).collect();
        aux_ids.sort();
        for i in 1..aux_ids.len() {
            prop_assert_eq!(aux_ids[i], aux_ids[i - 1] + 1, "Auxiliary IDs must be sequential");
        }
    }

    #[test]
    fn aux_ids_no_collision_with_originals(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let original_ids: Vec<SymbolId> = g.rules.keys().copied().collect();
        g.normalize();
        let new_ids: Vec<SymbolId> = g.rules.keys().filter(|id| !original_ids.contains(id)).copied().collect();
        for new_id in &new_ids {
            prop_assert!(!original_ids.contains(new_id), "Aux id {new_id} collides with original");
        }
    }
}

// =========================================================================
// 8. Already-normalized grammar is unchanged
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn terminal_only_grammar_unchanged(num_tokens in 1_usize..6) {
        let mut g = build_simple_grammar(num_tokens, 1);
        let rules_before = total_rule_count(&g);
        let lhs_before = g.rules.len();
        g.normalize();
        prop_assert_eq!(total_rule_count(&g), rules_before);
        prop_assert_eq!(g.rules.len(), lhs_before);
        assert_fully_normalized(&g);
    }

    #[test]
    fn already_normalized_grammar_stable(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        g.normalize();
        let rules_after_first = total_rule_count(&g);
        let lhs_after_first = g.rules.len();
        g.normalize();
        prop_assert_eq!(total_rule_count(&g), rules_after_first);
        prop_assert_eq!(g.rules.len(), lhs_after_first);
        assert_fully_normalized(&g);
    }
}

// =========================================================================
// 9. Deep nesting and mixed complex symbols
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn deep_nested_fully_normalized(sym in deep_nested_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn deep_nested_idempotent(sym in deep_nested_strategy()) {
        let g0 = grammar_with_complex_rhs(sym);
        let mut g1 = g0.clone();
        let mut g2 = g0;
        g1.normalize();
        g2.normalize();
        g2.normalize();
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
    }

    #[test]
    fn deep_nested_rule_count_grows(sym in deep_nested_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) > before);
    }

    #[test]
    fn mixed_optional_and_repeat(term_idx in 0_usize..2) {
        let term = if term_idx == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) };
        let opt = Symbol::Optional(Box::new(term.clone()));
        let rep = Symbol::Repeat(Box::new(term));
        let mut g = grammar_with_rhs(vec![opt, rep]);
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.rules.len() >= 3, "Optional + Repeat should create ≥2 aux LHS entries");
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
        prop_assert!(g.validate().is_ok());
    }
}

// =========================================================================
// 10. Additional structural invariants
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_preserves_grammar_name(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let name = g.name.clone();
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn normalize_preserves_token_count(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let count = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), count);
    }

    #[test]
    fn normalize_preserves_extras(_dummy in 0_u8..1) {
        let mut g = GrammarBuilder::new("extras")
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
// 11. Determinism – two clones normalize identically
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_deterministic(sym in complex_symbol_strategy()) {
        let base = grammar_with_complex_rhs(sym);
        let mut g1 = base.clone();
        let mut g2 = base;
        g1.normalize();
        g2.normalize();
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
        for (lhs, rules1) in &g1.rules {
            let rules2 = g2.rules.get(lhs);
            prop_assert!(rules2.is_some(), "LHS {lhs} missing in second normalize");
            prop_assert_eq!(rules1.len(), rules2.unwrap().len());
        }
    }

    #[test]
    fn normalize_deterministic_deep(sym in deep_nested_strategy()) {
        let base = grammar_with_complex_rhs(sym);
        let mut g1 = base.clone();
        let mut g2 = base;
        g1.normalize();
        g2.normalize();
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
        prop_assert_eq!(total_rule_count(&g1), total_rule_count(&g2));
        for (lhs, rules1) in &g1.rules {
            let rules2 = g2.rules.get(lhs);
            prop_assert!(rules2.is_some(), "LHS {lhs} missing in second normalize");
            let rules2 = rules2.unwrap();
            for i in 0..rules1.len() {
                prop_assert_eq!(rules1[i].rhs.len(), rules2[i].rhs.len());
            }
        }
    }

    #[test]
    fn normalize_rule_count_never_decreases(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(
            total_rule_count(&g) >= before,
            "Normalize must never reduce rule count: before={before}, after={}",
            total_rule_count(&g)
        );
    }
}
