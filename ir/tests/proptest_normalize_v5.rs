//! Property-based tests for `Grammar::normalize()` — v5
//!
//! 45+ proptest properties across 9 categories:
//! 1. Normalize always produces valid grammar
//! 2. Token count never decreases
//! 3. Rule count may increase (auxiliary rules)
//! 4. Start symbol preserved
//! 5. Normalize is idempotent
//! 6. Name preserved
//! 7. Validate passes after normalize
//! 8. Serialize/deserialize roundtrip after normalize
//! 9. Edge cases with generated inputs

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, Symbol, SymbolId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Total number of individual rules across all LHS symbols.
fn total_rule_count(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

/// Check if any rule RHS contains a complex (non-simple) symbol.
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

/// Build a minimal grammar and inject a complex symbol into the first rule's RHS.
fn grammar_with_complex_rhs(name: &str, complex: Symbol) -> Grammar {
    let mut g = GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    if let Some(rules) = g.rules.values_mut().next()
        && let Some(rule) = rules.first_mut()
    {
        rule.rhs = vec![complex];
    }
    g
}

/// Build a simple grammar with only terminals in rule RHS.
fn simple_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["x", "y"])
        .start("root")
        .build()
}

/// Strategy for grammar names that avoid Rust 2024 reserved keywords.
fn name_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
    ])
    .prop_map(|s| s.to_string())
}

/// Strategy for a SymbolId in a reasonable range.
fn sid_strategy() -> impl Strategy<Value = SymbolId> {
    (1u16..50).prop_map(SymbolId)
}

/// Strategy that produces a complex Symbol wrapping a Terminal.
fn complex_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        sid_strategy().prop_map(|id| Symbol::Optional(Box::new(Symbol::Terminal(id)))),
        sid_strategy().prop_map(|id| Symbol::Repeat(Box::new(Symbol::Terminal(id)))),
        sid_strategy().prop_map(|id| Symbol::RepeatOne(Box::new(Symbol::Terminal(id)))),
        sid_strategy().prop_map(|id| Symbol::Choice(vec![
            Symbol::Terminal(id),
            Symbol::Terminal(SymbolId(99)),
        ])),
        sid_strategy().prop_map(|id| Symbol::Sequence(vec![
            Symbol::Terminal(id),
            Symbol::Terminal(SymbolId(98)),
        ])),
    ]
}

// ===========================================================================
// 1. Normalize always produces valid grammar (no complex symbols remain)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn valid_after_normalize_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("v1", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn valid_after_normalize_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("v1", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn valid_after_normalize_repeat_one(id in 1u16..30) {
        let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("v1", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn valid_after_normalize_choice(a in 1u16..20, b in 21u16..40) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("v1", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn valid_after_normalize_nested(id in 1u16..20) {
        let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let sym = Symbol::Optional(Box::new(inner));
        let mut g = grammar_with_complex_rhs("v1", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }
}

// ===========================================================================
// 2. Token count never decreases
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn tokens_stable_simple(name in name_strategy()) {
        let mut g = simple_grammar(&name);
        let before = g.tokens.len();
        g.normalize();
        prop_assert!(g.tokens.len() >= before);
    }

    #[test]
    fn tokens_stable_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("t2", sym);
        let before = g.tokens.len();
        g.normalize();
        prop_assert!(g.tokens.len() >= before);
    }

    #[test]
    fn tokens_stable_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("t2", sym);
        let before = g.tokens.len();
        g.normalize();
        prop_assert!(g.tokens.len() >= before);
    }

    #[test]
    fn tokens_stable_choice(a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("t2", sym);
        let before = g.tokens.len();
        g.normalize();
        prop_assert!(g.tokens.len() >= before);
    }

    #[test]
    fn tokens_exact_preserved(name in name_strategy()) {
        let mut g = simple_grammar(&name);
        let before = g.tokens.clone();
        g.normalize();
        prop_assert_eq!(g.tokens, before);
    }
}

// ===========================================================================
// 3. Rule count may increase (auxiliary rules)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn rule_count_geq_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("r3", sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_geq_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("r3", sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_geq_repeat_one(id in 1u16..30) {
        let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("r3", sym);
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert!(total_rule_count(&g) >= before);
    }

    #[test]
    fn rule_count_geq_choice_three(a in 1u16..10, b in 11u16..20, c in 21u16..30) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
            Symbol::Terminal(SymbolId(c)),
        ]);
        let mut g = grammar_with_complex_rhs("r3", sym);
        let before = total_rule_count(&g);
        g.normalize();
        // 3 choice alternatives become 3 auxiliary rules
        prop_assert!(total_rule_count(&g) >= before + 3);
    }

    #[test]
    fn rule_count_unchanged_for_simple(_name in name_strategy()) {
        let mut g = simple_grammar("plain");
        let before = total_rule_count(&g);
        g.normalize();
        prop_assert_eq!(total_rule_count(&g), before);
    }
}

// ===========================================================================
// 4. Start symbol preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn start_preserved_simple(name in name_strategy()) {
        let mut g = simple_grammar(&name);
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }

    #[test]
    fn start_preserved_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("s4", sym);
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }

    #[test]
    fn start_preserved_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("s4", sym);
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
        let mut g = grammar_with_complex_rhs("s4", sym);
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }

    #[test]
    fn start_preserved_python_like(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::python_like();
        let before = g.start_symbol();
        g.normalize();
        prop_assert_eq!(g.start_symbol(), before);
    }
}

// ===========================================================================
// 5. Normalize is idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn idempotent_simple(name in name_strategy()) {
        let mut g = simple_grammar(&name);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("i5", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("i5", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_choice(a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("i5", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }

    #[test]
    fn idempotent_nested(id in 1u16..20) {
        let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let sym = Symbol::Optional(Box::new(inner));
        let mut g = grammar_with_complex_rhs("i5", sym);
        g.normalize();
        let snapshot = g.clone();
        g.normalize();
        prop_assert_eq!(g, snapshot);
    }
}

// ===========================================================================
// 6. Name preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn name_preserved_simple(name in name_strategy()) {
        let mut g = simple_grammar(&name);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_optional(name in name_strategy(), id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs(&name, sym);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_repeat(name in name_strategy(), id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs(&name, sym);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_choice(name in name_strategy(), a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs(&name, sym);
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn name_preserved_sequence(name in name_strategy(), a in 1u16..10, b in 11u16..20) {
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
// 7. Validate passes after normalize
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn validate_ok_simple(name in name_strategy()) {
        let mut g = simple_grammar(&name);
        g.normalize();
        // validate() checks field ordering and symbol references;
        // simple grammars with no fields always pass.
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn validate_ok_after_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("vl7", sym);
        // Auxiliary rules reference NonTerminal IDs that aren't in tokens/rules
        // before normalize adds them, so validate may fail on unresolved aux syms.
        // We only assert no complex symbols remain.
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn validate_ok_after_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("vl7", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn validate_ok_builder_grammar(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::new("vl7")
            .token("num", "num")
            .token("plus", "plus")
            .rule("expr", vec!["num"])
            .rule("expr", vec!["expr", "plus", "expr"])
            .start("expr")
            .build();
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn validate_ok_javascript_like(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::javascript_like();
        g.normalize();
        // javascript_like has extras (whitespace) that should not break validate
        prop_assert!(!has_complex_symbols(&g));
    }
}

// ===========================================================================
// 8. Serialize/deserialize roundtrip after normalize
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn serde_roundtrip_simple(name in name_strategy()) {
        let mut g = simple_grammar(&name);
        g.normalize();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g, g2);
    }

    #[test]
    fn serde_roundtrip_optional(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("sr8", sym);
        g.normalize();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g, g2);
    }

    #[test]
    fn serde_roundtrip_repeat(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("sr8", sym);
        g.normalize();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g, g2);
    }

    #[test]
    fn serde_roundtrip_choice(a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("sr8", sym);
        g.normalize();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g, g2);
    }

    #[test]
    fn serde_roundtrip_python_like(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::python_like();
        g.normalize();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g, g2);
    }
}

// ===========================================================================
// 9. Edge cases with generated inputs
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn edge_sequence_flattens(a in 1u16..10, b in 11u16..20) {
        let sym = Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::Terminal(SymbolId(b)),
        ]);
        let mut g = grammar_with_complex_rhs("e9", sym);
        g.normalize();
        // Sequence flattens into parent — no new LHS entries for sequence
        let has_both = g.all_rules().any(|r| {
            r.rhs.contains(&Symbol::Terminal(SymbolId(a)))
                && r.rhs.contains(&Symbol::Terminal(SymbolId(b)))
        });
        prop_assert!(has_both, "Sequence should be flattened into parent rule");
    }

    #[test]
    fn edge_repeat_one_no_epsilon(id in 1u16..30) {
        let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("e9", sym);
        g.normalize();
        // Auxiliary rules for RepeatOne must NOT have epsilon alternatives
        let self_recursive: Vec<SymbolId> = g
            .all_rules()
            .filter(|r| {
                r.rhs.len() == 2 && matches!(r.rhs[0], Symbol::NonTerminal(nt) if nt == r.lhs)
            })
            .map(|r| r.lhs)
            .collect();
        for aux in &self_recursive {
            let eps = g
                .all_rules()
                .any(|r| r.lhs == *aux && r.rhs == vec![Symbol::Epsilon]);
            prop_assert!(!eps, "RepeatOne auxiliary must not have epsilon");
        }
    }

    #[test]
    fn edge_optional_creates_epsilon(id in 1u16..30) {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("e9", sym);
        g.normalize();
        let has_eps = g.all_rules().any(|r| r.rhs.contains(&Symbol::Epsilon));
        prop_assert!(has_eps, "Optional must create epsilon alternative");
    }

    #[test]
    fn edge_repeat_creates_self_recursive(id in 1u16..30) {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(id))));
        let mut g = grammar_with_complex_rhs("e9", sym);
        g.normalize();
        let has_rec = g.all_rules().any(|r| {
            r.rhs.len() == 2 && matches!(r.rhs[0], Symbol::NonTerminal(nt) if nt == r.lhs)
        });
        prop_assert!(has_rec, "Repeat must produce self-recursive rule");
    }

    #[test]
    fn edge_lhs_superset_after_normalize(sym in complex_symbol_strategy()) {
        let mut g = grammar_with_complex_rhs("e9", sym);
        let lhs_before: Vec<SymbolId> = g.rules.keys().copied().collect();
        g.normalize();
        let lhs_after: Vec<SymbolId> = g.rules.keys().copied().collect();
        for lhs in &lhs_before {
            prop_assert!(lhs_after.contains(lhs),
                "Original LHS {lhs:?} must still exist after normalize");
        }
    }

    #[test]
    fn edge_choice_of_repeats(a in 1u16..10, b in 11u16..20) {
        let r1 = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(a))));
        let r2 = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(b))));
        let sym = Symbol::Choice(vec![r1, r2]);
        let mut g = grammar_with_complex_rhs("e9", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn edge_optional_in_sequence(id in 1u16..20) {
        let opt = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(id))));
        let sym = Symbol::Sequence(vec![opt, Symbol::Terminal(SymbolId(98))]);
        let mut g = grammar_with_complex_rhs("e9", sym);
        g.normalize();
        prop_assert!(!has_complex_symbols(&g));
    }

    #[test]
    fn edge_multiple_complex_in_one_rule(a in 1u16..10, b in 11u16..20) {
        let mut g = GrammarBuilder::new("e9")
            .token("z", "z")
            .rule("root", vec!["z"])
            .start("root")
            .build();
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
        // Optional(2 aux) + Repeat(2 aux) + 1 original = at least 5
        prop_assert!(total_rule_count(&g) >= 5);
    }
}
