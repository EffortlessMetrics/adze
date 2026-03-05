//! Property-based tests for Grammar::normalize()
//!
//! Tests that normalize() correctly expands Optional, Repeat, RepeatOne,
//! Choice, and Sequence symbols into auxiliary rules.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar with one token and one rule from terminal symbols only.
fn simple_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Build a grammar and manually inject a complex symbol into the first rule's RHS.
fn grammar_with_complex_rhs(name: &str, complex: Symbol) -> Grammar {
    let mut g = GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    // Replace the first rule's RHS with the complex symbol
    if let Some(rules) = g.rules.values_mut().next()
        && let Some(rule) = rules.first_mut()
    {
        rule.rhs = vec![complex];
    }
    g
}

/// Count total number of rules across all LHS symbols.
fn total_rule_count(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum::<usize>()
}

/// Check if any rule RHS contains a complex symbol.
fn has_complex_symbols(g: &Grammar) -> bool {
    g.all_rules().any(|r| {
        r.rhs.iter().any(|s| {
            matches!(
                s,
                Symbol::Optional(_)
                    | Symbol::Repeat(_)
                    | Symbol::RepeatOne(_)
                    | Symbol::Choice(_)
                    | Symbol::Sequence(_)
            )
        })
    })
}

/// Proptest strategy: pick a grammar name that avoids Rust 2024 reserved keywords.
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
    ])
    .prop_map(|s| s.to_string())
}

/// Proptest strategy for a SymbolId in a reasonable range.
fn symbol_id_strategy() -> impl Strategy<Value = SymbolId> {
    (1u16..50).prop_map(SymbolId)
}

/// Strategy that produces a complex Symbol wrapping a Terminal.
fn complex_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        symbol_id_strategy().prop_map(|id| Symbol::Optional(Box::new(Symbol::Terminal(id)))),
        symbol_id_strategy().prop_map(|id| Symbol::Repeat(Box::new(Symbol::Terminal(id)))),
        symbol_id_strategy().prop_map(|id| Symbol::RepeatOne(Box::new(Symbol::Terminal(id)))),
        symbol_id_strategy().prop_map(|id| Symbol::Choice(vec![
            Symbol::Terminal(id),
            Symbol::Terminal(SymbolId(99))
        ])),
        symbol_id_strategy().prop_map(|id| Symbol::Sequence(vec![
            Symbol::Terminal(id),
            Symbol::Terminal(SymbolId(98))
        ])),
    ]
}

// ===========================================================================
// 1. Idempotency — normalize(normalize(g)) == normalize(g)  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn idempotent_simple_grammar(name in grammar_name_strategy()) {
        let mut g1 = simple_grammar(&name);
        g1.normalize();
        let snapshot = g1.clone();
        g1.normalize();
        prop_assert_eq!(g1, snapshot);
    }

    #[test]
    fn idempotent_with_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("opt", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_with_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("rep", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_with_repeat_one(id in 1u16..30) {
        let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("rep1", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_with_choice(a in 1u16..20, b in 21u16..40) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("ch", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_with_sequence(a in 1u16..20, b in 21u16..40) {
        let sym = Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("seq", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_nested_optional_repeat(id in 1u16..30) {
        let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let sym = Symbol::Optional(Box::new(inner));
        let mut g = grammar_with_complex_rhs("nest", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_python_like(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::python_like();
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }
}

// ===========================================================================
// 2. Normalize preserves grammar name  (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn name_preserved_simple(name in grammar_name_strategy()) {
        let mut g = simple_grammar(&name);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_with_optional(name in grammar_name_strategy(), id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs(&name, sym);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_with_repeat(name in grammar_name_strategy(), id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs(&name, sym);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_with_choice(name in grammar_name_strategy(), a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs(&name, sym);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_with_sequence(name in grammar_name_strategy(), a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs(&name, sym);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }
}

// ===========================================================================
// 3. Normalize preserves start symbol  (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn start_preserved_simple(name in grammar_name_strategy()) {
        let mut g = simple_grammar(&name);
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }

    #[test]
    fn start_preserved_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("sp", sym);
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }

    #[test]
    fn start_preserved_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("sp", sym);
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }

    #[test]
    fn start_preserved_choice(a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("sp", sym);
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }

    #[test]
    fn start_preserved_javascript_like(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::javascript_like();
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }
}

// ===========================================================================
// 4. Normalize preserves tokens  (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn tokens_preserved_simple(name in grammar_name_strategy()) {
        let mut g = simple_grammar(&name);
        let before = g.tokens.clone();
        g.normalize();
        prop_assert_eq!(g.tokens, before);
    }

    #[test]
    fn tokens_preserved_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("tok", sym);
        let before = g.tokens.clone();
        g.normalize();
        prop_assert_eq!(g.tokens, before);
    }

    #[test]
    fn tokens_preserved_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("tok", sym);
        let before = g.tokens.clone();
        g.normalize();
        prop_assert_eq!(g.tokens, before);
    }

    #[test]
    fn tokens_preserved_python_like(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::python_like();
        let before = g.tokens.clone();
        g.normalize();
        prop_assert_eq!(g.tokens, before);
    }

    #[test]
    fn tokens_preserved_javascript_like(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::javascript_like();
        let before = g.tokens.clone();
        g.normalize();
        prop_assert_eq!(g.tokens, before);
    }
}

// ===========================================================================
// 5. Normalized grammar has ≥ original rule count  (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn rule_count_geq_with_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("rc", sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before,
            "expected >= {} rules but got {}", before, total_rule_count(&g));
    }

    #[test]
    fn rule_count_geq_with_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("rc", sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_geq_with_repeat_one(id in 1u16..30) {
        let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("rc", sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_geq_with_choice(a in 1u16..10, b in 11u16..20, c in 21u16..30) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
            Symbol::Terminal(SymbolId(c)),
        ]);
        let mut g = grammar_with_complex_rhs("rc", sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_equal_for_no_complex(_name in grammar_name_strategy()) {
        let mut g = GrammarBuilder::new("plain")
            .token("x", "x")
            .token("y", "y")
            .rule("s", vec!["x", "y"])
            .start("s")
            .build();
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert_eq!(total_rule_count(&g), before);
    }
}

// ===========================================================================
// 6. Normalize produces valid grammar (no complex symbols remain)  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn no_complex_after_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn no_complex_after_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn no_complex_after_repeat_one(id in 1u16..30) {
        let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn no_complex_after_choice(a in 1u16..20, b in 21u16..40) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn no_complex_after_sequence(a in 1u16..20, b in 21u16..40) {
        let sym = Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn no_complex_after_nested_repeat_choice(id in 1u16..20) {
        let inner = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(id)),
            Symbol::Epsilon,
        ]);
        let sym = Symbol::Repeat(Box::new(inner));
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn no_complex_after_nested_optional_in_sequence(id in 1u16..20) {
        let opt = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let sym = Symbol::Sequence(vec![opt, Symbol::Terminal(SymbolId(99))]);
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn no_complex_after_choice_of_repeats(a in 1u16..20, b in 21u16..40) {
        let r1 = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(a))));
        let r2 = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(b))));
        let sym = Symbol::Choice(vec![r1, r2]);
        let mut g = grammar_with_complex_rhs("v", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }
}

// ===========================================================================
// 7. Various grammar patterns  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn optional_creates_epsilon_alt(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("pat", sym);
        g.normalize();
        // The auxiliary rule should have an epsilon alternative
        let has_eps = g.all_rules().any(|r| r.rhs.contains(&Symbol::Epsilon));
        prop_assert!(has_eps, "Optional must produce an epsilon alternative");
    }

    #[test]
    fn repeat_creates_self_recursive_rule(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("pat", sym);
        g.normalize();
        // Must have a rule of the form: aux -> aux T
        let has_recursive = g.all_rules().any(|r| {
            r.rhs.len() == 2 && matches!(r.rhs[0], Symbol::NonTerminal(nt) if nt == r.lhs)
        });
        prop_assert!(has_recursive, "Repeat must produce a left-recursive rule");
    }

    #[test]
    fn repeat_one_no_epsilon(id in 1u16..30) {
        let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("pat", sym);
        g.normalize();
        // RepeatOne auxiliary should NOT have an epsilon production (unlike Repeat)
        // It produces: aux -> aux T | T
        let aux_ids: Vec<SymbolId> = g
            .all_rules()
            .filter(|r| {
                r.rhs.len() == 2 && matches!(r.rhs[0], Symbol::NonTerminal(nt) if nt == r.lhs)
            })
            .map(|r| r.lhs)
            .collect();

        for aux_id in &aux_ids {
            let eps_alt = g.all_rules().any(|r| {
                r.lhs == *aux_id && r.rhs.len() == 1 && r.rhs[0] == Symbol::Epsilon
            });
            prop_assert!(!eps_alt, "RepeatOne auxiliary must not have epsilon alternative");
        }
    }

    #[test]
    fn choice_creates_one_rule_per_alt(a in 1u16..10, b in 11u16..20, c in 21u16..30) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
            Symbol::Terminal(SymbolId(c)),
        ]);
        let mut g = grammar_with_complex_rhs("pat", sym);
        let rules_before = total_rule_count(&g);
        g.normalize();
        // Choice with 3 alternatives adds 3 auxiliary rules; original rule remains = +3
        prop_assert!(total_rule_count(&g) >= rules_before + 3);
    }

    #[test]
    fn sequence_flattens_into_parent(a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("pat", sym);
        g.normalize();
        // After normalize, the start rule should have the flattened symbols
        let start_rules: Vec<&Rule> = g.all_rules().collect();
        let has_flat = start_rules
            .iter()
            .any(|r| r.rhs.contains(&Symbol::Terminal(SymbolId(a)))
                    && r.rhs.contains(&Symbol::Terminal(SymbolId(b))));
        prop_assert!(has_flat, "Sequence should flatten into parent rule");
    }

    #[test]
    fn multiple_complex_in_one_rule(a in 1u16..10, b in 11u16..20) {
        let mut g = GrammarBuilder::new("multi")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        // Inject two complex symbols
        if let Some(rules) = g.rules.values_mut().next()
            && let Some(rule) = rules.first_mut()
        {
            rule.rhs = vec![
                Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(a)))),
                Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(b)))),
            ];
        }
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
        // Should have created auxiliary rules for both
        prop_assert!(total_rule_count(&g) >= 5); // 1 orig + 2 for opt + 2 for rep
    }

    #[test]
    fn lhs_symbols_are_superset(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs("sup", sym);
        let lhs_before: Vec<SymbolId> = g.rules.keys().copied().collect();
        g.normalize();
        let lhs_after: Vec<SymbolId> = g.rules.keys().copied().collect();
        for lhs in &lhs_before {
            prop_assert!(lhs_after.contains(lhs),
                "Original LHS {:?} must still be present after normalize", lhs);
        }
    }

    #[test]
    fn precedence_preserved_through_normalize(id in 1u16..20) {
        let mut g = GrammarBuilder::new("prec")
            .token("x", "x")
            .token("+", "+")
            .rule_with_precedence("expr", vec!["expr", "+", "expr"], 5, Associativity::Left)
            .rule("expr", vec!["x"])
            .start("expr")
            .build();
        // Inject an Optional into the first rule
        if let Some(rules) = g.rules.values_mut().next()
            && let Some(rule) = rules.first_mut()
        {
            rule.rhs.push(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id)))));
        }
        g.normalize();
        // The original rule with precedence should still have it
        let has_prec = g.all_rules().any(|r| {
            matches!(r.precedence, Some(PrecedenceKind::Static(5)))
        });
        prop_assert!(has_prec, "Precedence must be preserved");
    }
}

// ===========================================================================
// 8. Edge cases  (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn empty_grammar_normalize(name in grammar_name_strategy()) {
        let mut g = Grammar::new(name.clone());
        g.normalize();
        prop_assert_eq!(&g.name, &name);
        prop_assert_eq!(total_rule_count(&g), 0);
    }

    #[test]
    fn epsilon_only_rule_unchanged(_dummy in 0u8..5) {
        let mut g = GrammarBuilder::new("eps")
            .rule("s", vec![])
            .start("s")
            .build();
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert_eq!(total_rule_count(&g), before);
    }

    #[test]
    fn deeply_nested_complex(depth in 1u8..4) {
        // Build nested Optional(Optional(...(Terminal)))
        let mut sym = Symbol::Terminal(SymbolId(42));
        for _ in 0..depth {
            sym = Symbol::Optional(Box::new(sym));
        }
        let mut g = grammar_with_complex_rhs("deep", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn choice_with_single_alternative(id in 1u16..30) {
        let sym = Symbol::Choice(vec![Symbol::Terminal(SymbolId(id))]);
        let mut g = grammar_with_complex_rhs("c1", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn sequence_with_single_element(id in 1u16..30) {
        let sym = Symbol::Sequence(vec![Symbol::Terminal(SymbolId(id))]);
        let mut g = grammar_with_complex_rhs("s1", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
        // Single-element sequence should flatten — no new LHS symbols beyond the original
    }

    #[test]
    fn normalize_return_value_matches_grammar(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs("rv", sym);
        let returned = g.normalize();
        let all_from_grammar: Vec<Rule> = g.all_rules().cloned().collect();
        prop_assert_eq!(returned.len(), all_from_grammar.len(),
            "normalize() return value must have same rule count as grammar");
    }
}
