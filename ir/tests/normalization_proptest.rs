#![allow(clippy::needless_range_loop)]

//! Property-based tests for `Grammar::normalize()`.
//!
//! Verifies key invariants:
//! - Normalization flattens Optional, Repeat, Choice, Sequence into auxiliary rules
//! - Normalization is idempotent
//! - Normalized grammars pass validation
//! - Terminal symbols are preserved
//! - Rule count never decreases
//! - Deep nesting and mixed complex symbols are handled

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

    // Replace the placeholder rule with one containing the complex symbol.
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

/// Return the SymbolId for token A (always SymbolId(1) in our helpers).
fn tok_a() -> SymbolId {
    SymbolId(1)
}

/// Return the SymbolId for token B (always SymbolId(2) in our helpers).
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
// 1. Normalization preserves terminal symbols
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn terminals_preserved_after_normalize(
        num_tokens in 1_usize..6,
        num_rules in 1_usize..4,
    ) {
        let mut g = build_simple_grammar(num_tokens, num_rules);
        let tokens_before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), tokens_before, "tokens must not change");
    }

    #[test]
    fn terminal_only_grammar_unchanged(
        num_tokens in 1_usize..6,
    ) {
        let mut g = build_simple_grammar(num_tokens, 1);
        let rules_before = total_rule_count(&g);
        g.normalize();
        prop_assert_eq!(total_rule_count(&g), rules_before);
        assert_fully_normalized(&g);
    }

    #[test]
    fn terminal_symbols_still_reachable(
        num_tokens in 1_usize..4,
    ) {
        // Use num_tokens rules so every token is referenced at least once
        let mut g = build_simple_grammar(num_tokens, num_tokens);
        g.normalize();
        // Every original token id should still be referenced somewhere
        for tok_id in g.tokens.keys() {
            let found = g.rules.values().flatten().any(|r| {
                r.rhs.iter().any(|s| matches!(s, Symbol::Terminal(id) if id == tok_id))
            });
            prop_assert!(found, "Token {tok_id} not found in any rule after normalize");
        }
    }
}

// =========================================================================
// 2. Normalization flattens Optional into auxiliary rules
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn optional_creates_aux_rules(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        g.normalize();
        assert_fully_normalized(&g);
        // Original 1 LHS + at least 1 auxiliary LHS
        prop_assert!(g.rules.len() >= 2, "Expected auxiliary rules for Optional");
    }

    #[test]
    fn optional_aux_has_epsilon_alternative(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        g.normalize();
        // At least one auxiliary rule should have an epsilon alternative
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
        // The inner terminal must appear in some aux rule's RHS
        let found = g.rules.values().flatten().any(|r| {
            r.rhs.iter().any(|s| matches!(s, Symbol::Terminal(id) if *id == inner_id))
        });
        prop_assert!(found, "Inner terminal of Optional must be preserved");
    }
}

// =========================================================================
// 3. Normalization flattens Repeat into auxiliary rules
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
    fn repeat_aux_has_epsilon_alternative(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(term)));
        g.normalize();
        let has_epsilon = g.rules.values().flatten().any(|r| {
            r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon)
        });
        prop_assert!(has_epsilon, "Repeat normalization must produce ε alternative");
    }

    #[test]
    fn repeat_aux_has_recursive_production(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(term)));
        g.normalize();
        // The aux rule for Repeat should have a self-recursive production: aux -> aux inner
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
        // RepeatOne aux should NOT have an epsilon-only alternative for the aux LHS
        // (it has `aux -> inner` instead of `aux -> ε`)
        let root_id = g.find_symbol_by_name("root").unwrap();
        for (lhs, rules) in &g.rules {
            if *lhs == root_id {
                continue;
            }
            // Aux rules: should not have an epsilon-only production
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
// 4. Normalization flattens Choice into auxiliary rules
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn choice_creates_aux_rules(
        n_choices in 2_usize..5,
    ) {
        let choices: Vec<Symbol> = (0..n_choices)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_complex_rhs(Symbol::Choice(choices));
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.rules.len() >= 2, "Expected auxiliary rules for Choice");
    }

    #[test]
    fn choice_aux_has_correct_alternative_count(
        n_choices in 2_usize..5,
    ) {
        let choices: Vec<Symbol> = (0..n_choices)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let expected = n_choices;
        let mut g = grammar_with_complex_rhs(Symbol::Choice(choices));
        g.normalize();
        // The aux LHS should have exactly n_choices alternative productions
        let root_id = g.find_symbol_by_name("root").unwrap();
        for (lhs, rules) in &g.rules {
            if *lhs == root_id {
                continue;
            }
            // This aux should be the Choice aux with the expected number of alternatives
            if rules.len() == expected {
                return Ok(());
            }
        }
        prop_assert!(false, "No aux rule had {expected} alternatives");
    }

    #[test]
    fn choice_with_epsilon_alternative(
        n_non_eps in 1_usize..4,
    ) {
        let mut choices: Vec<Symbol> = (0..n_non_eps)
            .map(|_| Symbol::Terminal(tok_a()))
            .collect();
        choices.push(Symbol::Epsilon);
        let mut g = grammar_with_complex_rhs(Symbol::Choice(choices));
        g.normalize();
        assert_fully_normalized(&g);
    }
}

// =========================================================================
// 5. Normalization flattens Sequence into the current rule
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn sequence_is_inlined(
        seq_len in 2_usize..5,
    ) {
        let seq: Vec<Symbol> = (0..seq_len)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_rhs(vec![Symbol::Sequence(seq.clone())]);
        g.normalize();
        assert_fully_normalized(&g);
        // Sequence gets inlined, so the root rule's RHS should contain the flattened symbols
        let root_id = g.find_symbol_by_name("root").unwrap();
        let root_rules = &g.rules[&root_id];
        prop_assert_eq!(root_rules[0].rhs.len(), seq_len);
    }

    #[test]
    fn sequence_does_not_create_aux(
        seq_len in 2_usize..5,
    ) {
        let seq: Vec<Symbol> = (0..seq_len)
            .map(|_| Symbol::Terminal(tok_a()))
            .collect();
        let mut g = grammar_with_rhs(vec![Symbol::Sequence(seq)]);
        g.normalize();
        // Sequence inlining should not create new LHS symbols
        prop_assert_eq!(g.rules.len(), 1, "Sequence should not create auxiliary rules");
    }

    #[test]
    fn sequence_with_trailing_terminal(
        seq_len in 1_usize..4,
    ) {
        let seq: Vec<Symbol> = (0..seq_len)
            .map(|_| Symbol::Terminal(tok_a()))
            .collect();
        // Add a plain terminal after the sequence
        let rhs = vec![Symbol::Sequence(seq), Symbol::Terminal(tok_b())];
        let mut g = grammar_with_rhs(rhs);
        g.normalize();
        assert_fully_normalized(&g);
        let root_id = g.find_symbol_by_name("root").unwrap();
        let root_rules = &g.rules[&root_id];
        // seq_len items from sequence + 1 trailing terminal
        prop_assert_eq!(root_rules[0].rhs.len(), seq_len + 1);
    }
}

// =========================================================================
// 6. Normalization is idempotent
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
// 7. Normalized grammar passes validation
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalized_simple_grammar_validates(
        num_tokens in 1_usize..5,
        num_rules in 1_usize..4,
    ) {
        let mut g = build_simple_grammar(num_tokens, num_rules);
        g.normalize();
        // validate() checks field ordering and symbol resolution
        prop_assert!(g.validate().is_ok(), "Normalized grammar must validate");
    }

    #[test]
    fn normalized_optional_validates(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn normalized_repeat_validates(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(term)));
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn normalized_choice_validates(n in 2_usize..5) {
        let choices: Vec<Symbol> = (0..n)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_complex_rhs(Symbol::Choice(choices));
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }
}

// =========================================================================
// 8. Rule count after normalization >= original
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_count_nondecreasing_optional(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(term)));
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before,
            "Rule count must not decrease: before={before}, after={}", total_rule_count(&g));
    }

    #[test]
    fn rule_count_nondecreasing_repeat(term in terminal_strategy()) {
        let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(term)));
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_nondecreasing_choice(n in 2_usize..5) {
        let choices: Vec<Symbol> = (0..n)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let mut g = grammar_with_complex_rhs(Symbol::Choice(choices));
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn lhs_count_nondecreasing_simple(
        num_tokens in 1_usize..5,
        num_rules in 1_usize..4,
    ) {
        let mut g = build_simple_grammar(num_tokens, num_rules);
        let before = g.rules.len();
        g.normalize();
        prop_assert!(g.rules.len() >= before);
    }
}

// =========================================================================
// 9. Deep nesting normalization
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
    fn deep_nested_validates(sym in deep_nested_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn deep_nested_rule_count_grows(sym in deep_nested_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let before = total_rule_count(&g);
        g.normalize();
        // Deep nesting always produces auxiliary rules
        prop_assert!(total_rule_count(&g) > before);
    }
}

// =========================================================================
// 10. Mixed complex symbols normalization
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

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
    fn mixed_choice_and_sequence(n_choices in 2_usize..4, seq_len in 2_usize..4) {
        let choices: Vec<Symbol> = (0..n_choices)
            .map(|i| if i % 2 == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) })
            .collect();
        let seq: Vec<Symbol> = (0..seq_len)
            .map(|_| Symbol::Terminal(tok_a()))
            .collect();
        let mut g = grammar_with_rhs(vec![Symbol::Choice(choices), Symbol::Sequence(seq)]);
        g.normalize();
        assert_fully_normalized(&g);
    }

    #[test]
    fn mixed_repeat_one_and_optional(term_idx in 0_usize..2) {
        let term = if term_idx == 0 { Symbol::Terminal(tok_a()) } else { Symbol::Terminal(tok_b()) };
        let rep1 = Symbol::RepeatOne(Box::new(term.clone()));
        let opt = Symbol::Optional(Box::new(term));
        let mut g = grammar_with_rhs(vec![rep1, opt]);
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.rules.len() >= 3);
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

    #[test]
    fn mixed_validates_after_normalize(sym in nested_complex_strategy()) {
        let other = Symbol::Repeat(Box::new(Symbol::Terminal(tok_b())));
        let mut g = grammar_with_rhs(vec![sym, other]);
        g.normalize();
        assert_fully_normalized(&g);
        prop_assert!(g.validate().is_ok());
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

// =========================================================================
// Additional structural properties
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn aux_ids_above_existing_max(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let max_before = g.rules.keys().map(|id| id.0).max().unwrap_or(0);
        g.normalize();
        // Auxiliary symbol IDs start at max_existing + 1000
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
    fn normalize_preserves_grammar_name(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let name = g.name.clone();
        g.normalize();
        prop_assert_eq!(&g.name, &name);
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

    #[test]
    fn normalize_preserves_token_count(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs(sym);
        let count = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), count);
    }
}
