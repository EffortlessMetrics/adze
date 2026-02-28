// Property-based tests for adze-ir types and grammar operations.
//
// These tests use proptest to generate random inputs and verify invariants.

use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use proptest::prelude::*;

// --- Strategies ---

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..500).prop_map(SymbolId)
}

fn arb_token_pattern() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(|s| TokenPattern::String(s)),
        "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(|s| TokenPattern::Regex(s)),
    ]
}

fn arb_leaf_symbol() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        arb_symbol_id().prop_map(Symbol::Terminal),
        arb_symbol_id().prop_map(Symbol::NonTerminal),
        Just(Symbol::Epsilon),
    ]
}

fn arb_symbol() -> impl Strategy<Value = Symbol> {
    arb_leaf_symbol().prop_recursive(3, 8, 4, |inner| {
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

fn arb_rule(lhs: SymbolId) -> impl Strategy<Value = Rule> {
    (
        prop::collection::vec(arb_leaf_symbol(), 1..5),
        prop::option::of(arb_precedence_kind()),
        prop::option::of(prop_oneof![
            Just(Associativity::Left),
            Just(Associativity::Right),
            Just(Associativity::None),
        ]),
    )
        .prop_map(move |(rhs, precedence, associativity)| Rule {
            lhs,
            rhs,
            precedence,
            associativity,
            fields: vec![],
            production_id: ProductionId(0),
        })
}

/// Build a small grammar with the given number of rules.
fn arb_grammar(max_rules: usize) -> impl Strategy<Value = Grammar> {
    prop::collection::vec(
        (arb_symbol_id(), "[a-z]{3,8}".prop_map(String::from)),
        1..=max_rules,
    )
    .prop_flat_map(|name_pairs| {
        let lhs_ids: Vec<SymbolId> = name_pairs.iter().map(|(id, _)| *id).collect();
        let names = name_pairs;

        let rule_strats: Vec<_> = lhs_ids.iter().map(|id| arb_rule(*id)).collect();

        (Just(names), rule_strats)
    })
    .prop_map(|(names, rules)| {
        let mut grammar = Grammar::default();
        grammar.name = "test_grammar".to_string();
        for ((sym_id, name), rule) in names.into_iter().zip(rules.into_iter()) {
            grammar.rule_names.insert(sym_id, name);
            grammar.add_rule(rule);
        }
        grammar
    })
}

// --- Property tests ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn symbol_id_roundtrip(id in 0u16..=u16::MAX) {
        let sym = SymbolId(id);
        prop_assert_eq!(sym.0, id);
        prop_assert_eq!(format!("{sym}"), format!("Symbol({id})"));
    }

    #[test]
    fn symbol_eq_is_reflexive(sym in arb_symbol()) {
        prop_assert_eq!(&sym, &sym);
    }

    #[test]
    fn symbol_clone_is_equal(sym in arb_symbol()) {
        let cloned = sym.clone();
        prop_assert_eq!(&sym, &cloned);
    }

    #[test]
    fn token_pattern_preserves_content(s in "[a-zA-Z0-9_]{1,50}") {
        let string_tok = TokenPattern::String(s.clone());
        let regex_tok = TokenPattern::Regex(s.clone());

        match &string_tok {
            TokenPattern::String(v) => prop_assert_eq!(v, &s),
            _ => prop_assert!(false, "wrong variant"),
        }
        match &regex_tok {
            TokenPattern::Regex(v) => prop_assert_eq!(v, &s),
            _ => prop_assert!(false, "wrong variant"),
        }
    }

    #[test]
    fn grammar_add_rule_increases_count(
        lhs in arb_symbol_id(),
        rhs_len in 1usize..5,
    ) {
        let mut grammar = Grammar::default();
        let rhs = vec![Symbol::Terminal(SymbolId(0)); rhs_len];
        let rule = Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);
        prop_assert!(grammar.rules.contains_key(&lhs));
        prop_assert_eq!(grammar.rules[&lhs].len(), 1);
    }

    #[test]
    fn grammar_serde_roundtrip(grammar in arb_grammar(5)) {
        let json = serde_json::to_string(&grammar).unwrap();
        let deserialized: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar.name, deserialized.name);
        prop_assert_eq!(grammar.rules.len(), deserialized.rules.len());
    }

    #[test]
    fn normalize_removes_complex_symbols(
        inner_id in arb_symbol_id(),
    ) {
        let mut grammar = Grammar::default();
        grammar.name = "test".to_string();
        let lhs = SymbolId(1);

        // Create a rule with an Optional symbol
        let rule = Rule {
            lhs,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(inner_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);
        grammar.normalize();

        // After normalization, no rule should contain Optional/Repeat/Choice/Sequence
        for rules in grammar.rules.values() {
            for rule in rules {
                for sym in &rule.rhs {
                    prop_assert!(!is_complex(sym), "Found complex symbol after normalize: {sym:?}");
                }
            }
        }
    }

    #[test]
    fn normalize_is_idempotent(grammar in arb_grammar(3)) {
        let mut g1 = grammar.clone();
        g1.normalize();

        let mut g2 = g1.clone();
        g2.normalize();

        // After double-normalizing, the grammar should be structurally equivalent
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
        for (lhs, rules1) in &g1.rules {
            let rules2 = g2.rules.get(lhs).unwrap();
            prop_assert_eq!(rules1.len(), rules2.len());
        }
    }

    #[test]
    fn normalize_repeat_creates_epsilon_alternative(
        inner_id in arb_symbol_id(),
    ) {
        let mut grammar = Grammar::default();
        grammar.name = "test".to_string();
        let lhs = SymbolId(1);

        let rule = Rule {
            lhs,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(inner_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);
        grammar.normalize();

        // The auxiliary rule for Repeat should include an epsilon alternative
        let has_epsilon = grammar.rules.values().any(|rules| {
            rules.iter().any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)))
        });
        prop_assert!(has_epsilon, "Repeat normalization should produce epsilon alternative");
    }

    #[test]
    fn check_empty_terminals_rejects_empty(name in "[a-z]{1,10}") {
        let mut grammar = Grammar::default();
        grammar.tokens.insert(
            SymbolId(0),
            Token {
                name: name.clone(),
                pattern: TokenPattern::String(String::new()),
                fragile: false,
            },
        );
        let result = grammar.check_empty_terminals();
        prop_assert!(result.is_err());
    }

    #[test]
    fn check_empty_terminals_accepts_nonempty(
        name in "[a-z]{1,10}",
        pattern in "[a-z]{1,20}",
    ) {
        let mut grammar = Grammar::default();
        grammar.tokens.insert(
            SymbolId(0),
            Token {
                name,
                pattern: TokenPattern::String(pattern),
                fragile: false,
            },
        );
        let result = grammar.check_empty_terminals();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn precedence_kind_preserves_value(val in -1000i16..1000) {
        let s = PrecedenceKind::Static(val);
        let d = PrecedenceKind::Dynamic(val);
        match s {
            PrecedenceKind::Static(v) => prop_assert_eq!(v, val),
            _ => prop_assert!(false),
        }
        match d {
            PrecedenceKind::Dynamic(v) => prop_assert_eq!(v, val),
            _ => prop_assert!(false),
        }
    }
}

/// Check if a symbol is "complex" (needs normalization).
fn is_complex(sym: &Symbol) -> bool {
    matches!(
        sym,
        Symbol::Optional(_)
            | Symbol::Repeat(_)
            | Symbol::RepeatOne(_)
            | Symbol::Choice(_)
            | Symbol::Sequence(_)
    )
}
