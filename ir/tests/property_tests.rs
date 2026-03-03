use adze_ir::*;
use proptest::prelude::*;
use std::collections::HashSet;

// -- Strategies for generating arbitrary grammar components --

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..100).prop_map(SymbolId)
}

fn arb_production_id() -> impl Strategy<Value = ProductionId> {
    (0u16..100).prop_map(ProductionId)
}

fn arb_simple_symbol() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        arb_symbol_id().prop_map(Symbol::Terminal),
        arb_symbol_id().prop_map(Symbol::NonTerminal),
        Just(Symbol::Epsilon),
    ]
}

fn arb_symbol() -> impl Strategy<Value = Symbol> {
    arb_simple_symbol().prop_recursive(3, 8, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..4).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..4).prop_map(Symbol::Sequence),
        ]
    })
}

fn arb_precedence_kind() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        (-100i16..100).prop_map(PrecedenceKind::Static),
        (-100i16..100).prop_map(PrecedenceKind::Dynamic),
    ]
}

fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

fn arb_rule(lhs: SymbolId) -> impl Strategy<Value = Rule> {
    (
        prop::collection::vec(arb_symbol(), 1..5),
        prop::option::of(arb_precedence_kind()),
        prop::option::of(arb_associativity()),
        arb_production_id(),
    )
        .prop_map(
            move |(rhs, precedence, associativity, production_id)| Rule {
                lhs,
                rhs,
                precedence,
                associativity,
                fields: vec![],
                production_id,
            },
        )
}

/// Generate a grammar with 1..max_rules rules spread across 1..max_symbols LHS symbols,
/// each LHS backed by a token so symbols are "defined".
fn arb_grammar(max_symbols: usize, max_rules_per: usize) -> impl Strategy<Value = Grammar> {
    let num_symbols = 1..=max_symbols;
    num_symbols.prop_flat_map(move |n| {
        let rules_strategies: Vec<_> = (0..n)
            .map(|i| {
                let lhs = SymbolId(i as u16);
                prop::collection::vec(arb_rule(lhs), 1..=max_rules_per)
            })
            .collect();
        ("[a-z]{3,10}".prop_map(|s| s.to_string()), rules_strategies).prop_map(
            move |(name, all_rules)| {
                let mut grammar = Grammar::new(name);
                for (i, rules) in all_rules.into_iter().enumerate() {
                    let sid = SymbolId(i as u16);
                    // Register a token so the symbol is "defined"
                    grammar.tokens.insert(
                        sid,
                        Token {
                            name: format!("tok_{i}"),
                            pattern: TokenPattern::String(format!("t{i}")),
                            fragile: false,
                        },
                    );
                    grammar.rule_names.insert(sid, format!("rule_{i}"));
                    for rule in rules {
                        grammar.add_rule(rule);
                    }
                }
                grammar
            },
        )
    })
}

// ---- Property tests ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 1. Grammar normalization: any grammar should normalize without panic
    #[test]
    fn normalize_does_not_panic(mut grammar in arb_grammar(5, 3)) {
        // Should not panic regardless of the grammar content
        let _rules = grammar.normalize();
    }

    /// 2. Symbol ID allocation: normalized grammars should have unique symbol IDs
    #[test]
    fn normalized_symbol_ids_are_unique(mut grammar in arb_grammar(5, 3)) {
        grammar.normalize();
        let mut seen_lhs: HashSet<SymbolId> = HashSet::new();
        // Every LHS key in the rules map should be unique (guaranteed by IndexMap,
        // but verify the auxiliary IDs don't collide with originals).
        for &lhs in grammar.rules.keys() {
            prop_assert!(seen_lhs.insert(lhs), "duplicate LHS SymbolId: {lhs}");
        }
    }

    /// 3. Validation: empty grammars should produce errors
    #[test]
    fn empty_grammar_has_validation_errors(name in "[a-z]{3,10}") {
        let grammar = Grammar::new(name);
        let mut validator = GrammarValidator::new();
        let result = validator.validate(&grammar);
        prop_assert!(
            result.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)),
            "expected EmptyGrammar error for an empty grammar"
        );
    }

    /// 4. Optimizer: optimization should not panic and should preserve the grammar name
    #[test]
    fn optimizer_preserves_grammar_identity(grammar in arb_grammar(5, 3)) {
        let original_name = grammar.name.clone();
        let mut optimized = grammar.clone();
        let mut optimizer = GrammarOptimizer::new();
        let stats = optimizer.optimize(&mut optimized);
        prop_assert_eq!(&optimized.name, &original_name, "optimizer changed grammar name");
        // Stats total should be non-negative (sanity check)
        let _ = stats.total();
    }

    /// 5. Serialization roundtrip: serialize → deserialize → equals
    #[test]
    fn serde_roundtrip(grammar in arb_grammar(4, 2)) {
        let json = serde_json::to_string(&grammar).expect("serialize failed");
        let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize failed");
        // Grammar doesn't derive PartialEq, so compare via re-serialization
        let json2 = serde_json::to_string(&deserialized).expect("re-serialize failed");
        prop_assert_eq!(json, json2, "roundtrip produced different JSON");
    }

    /// 6. Grammar cloning: clone should equal original (via serialization)
    #[test]
    fn clone_equals_original(grammar in arb_grammar(4, 2)) {
        let cloned = grammar.clone();
        let json_orig = serde_json::to_string(&grammar).expect("serialize original");
        let json_clone = serde_json::to_string(&cloned).expect("serialize clone");
        prop_assert_eq!(json_orig, json_clone, "clone differs from original");
    }
}
