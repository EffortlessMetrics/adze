//! Property-based edge-case tests for IR crate types.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::GrammarValidator;
use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol,
    SymbolId, Token, TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn symbol_id_strategy() -> impl Strategy<Value = SymbolId> {
    any::<u16>().prop_map(SymbolId)
}

fn field_id_strategy() -> impl Strategy<Value = FieldId> {
    any::<u16>().prop_map(FieldId)
}

fn production_id_strategy() -> impl Strategy<Value = ProductionId> {
    any::<u16>().prop_map(ProductionId)
}

fn precedence_kind_strategy() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        any::<i16>().prop_map(PrecedenceKind::Static),
        any::<i16>().prop_map(PrecedenceKind::Dynamic),
    ]
}

fn associativity_strategy() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

/// Simple leaf symbols only (no recursive nesting).
fn leaf_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        symbol_id_strategy().prop_map(Symbol::Terminal),
        symbol_id_strategy().prop_map(Symbol::NonTerminal),
        symbol_id_strategy().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

/// Nested symbol strategy (up to 2 levels deep).
fn symbol_strategy() -> impl Strategy<Value = Symbol> {
    leaf_symbol_strategy().prop_recursive(2, 8, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..=3).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..=3).prop_map(Symbol::Sequence),
        ]
    })
}

fn token_pattern_strategy() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-z]{1,10}".prop_map(TokenPattern::String),
        Just(TokenPattern::Regex(r"\d+".to_string())),
        Just(TokenPattern::Regex(r"[a-zA-Z_]+".to_string())),
    ]
}

/// Safe rule-name strategy (lowercase identifier).
fn rule_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}"
}

/// Safe token-name strategy (uppercase identifier).
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,8}"
}

// ---------------------------------------------------------------------------
// 1. SymbolId / StateId / RuleId / ProductionId / FieldId u16 roundtrips
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn symbol_id_u16_roundtrip(v in any::<u16>()) {
        let id = SymbolId(v);
        prop_assert_eq!(id.0, v);
        let cloned = id;
        prop_assert_eq!(id, cloned);
    }

    #[test]
    fn state_id_u16_roundtrip(v in any::<u16>()) {
        let id = StateId(v);
        prop_assert_eq!(id.0, v);
        prop_assert_eq!(id, StateId(v));
    }

    #[test]
    fn rule_id_u16_roundtrip(v in any::<u16>()) {
        let id = RuleId(v);
        prop_assert_eq!(id.0, v);
        prop_assert_eq!(id, RuleId(v));
    }

    #[test]
    fn production_id_u16_roundtrip(v in any::<u16>()) {
        let id = ProductionId(v);
        prop_assert_eq!(id.0, v);
        prop_assert_eq!(id, ProductionId(v));
    }

    #[test]
    fn field_id_u16_roundtrip(v in any::<u16>()) {
        let id = FieldId(v);
        prop_assert_eq!(id.0, v);
        prop_assert_eq!(id, FieldId(v));
    }

    // Boundary values
    #[test]
    fn symbol_id_boundary(v in prop_oneof![Just(0u16), Just(1u16), Just(u16::MAX), Just(u16::MAX - 1)]) {
        let id = SymbolId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: SymbolId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn state_id_boundary(v in prop_oneof![Just(0u16), Just(1u16), Just(u16::MAX), Just(u16::MAX - 1)]) {
        let id = StateId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: StateId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }
}

// ---------------------------------------------------------------------------
// 2. Grammar with many rules (random rule counts 1..50)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn grammar_many_rules(count in 1usize..50) {
        let mut builder = GrammarBuilder::new("many_rules");
        builder = builder.token("NUM", r"\d+");
        for i in 0..count {
            builder = builder.rule(&format!("r{i}"), vec!["NUM"]);
        }
        let grammar = builder.build();
        // Each unique rule name gets its own entry in the rules map
        prop_assert!(!grammar.rules.is_empty());
        let total_rules: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total_rules, count);
    }

    #[test]
    fn grammar_many_rules_serde_roundtrip(count in 1usize..30) {
        let mut builder = GrammarBuilder::new("serde_many");
        builder = builder.token("TOK", "tok");
        for i in 0..count {
            builder = builder.rule(&format!("rule{i}"), vec!["TOK"]);
        }
        let grammar = builder.build();
        let json = serde_json::to_string(&grammar).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar.name, back.name);
        let orig_count: usize = grammar.rules.values().map(|v| v.len()).sum();
        let back_count: usize = back.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(orig_count, back_count);
    }
}

// ---------------------------------------------------------------------------
// 3. Grammar normalize idempotency
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_idempotent_simple(count in 1usize..10) {
        let mut builder = GrammarBuilder::new("idem");
        builder = builder.token("X", "x");
        for i in 0..count {
            builder = builder.rule(&format!("n{i}"), vec!["X"]);
        }
        let mut grammar = builder.build();

        grammar.normalize();
        let after_first = serde_json::to_string(&grammar).unwrap();

        grammar.normalize();
        let after_second = serde_json::to_string(&grammar).unwrap();

        prop_assert_eq!(after_first, after_second);
    }

    #[test]
    fn normalize_idempotent_with_complex_symbols(_dummy in 0..5i32) {
        // Build a grammar with Optional / Repeat / Choice symbols manually
        let mut grammar = Grammar::new("complex_norm".to_string());
        let lhs = SymbolId(1);
        let inner = SymbolId(2);

        grammar.tokens.insert(
            inner,
            Token { name: "a".into(), pattern: TokenPattern::String("a".into()), fragile: false },
        );
        grammar.rule_names.insert(lhs, "start".into());

        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(inner)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

        grammar.normalize();
        let snap1 = serde_json::to_string(&grammar).unwrap();

        grammar.normalize();
        let snap2 = serde_json::to_string(&grammar).unwrap();

        prop_assert_eq!(snap1, snap2);
    }
}

// ---------------------------------------------------------------------------
// 4. Symbol enum variants through serde roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn symbol_serde_roundtrip(sym in symbol_strategy()) {
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&sym, &back);
    }

    #[test]
    fn symbol_terminal_serde(id in symbol_id_strategy()) {
        let sym = Symbol::Terminal(id);
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }

    #[test]
    fn symbol_nonterminal_serde(id in symbol_id_strategy()) {
        let sym = Symbol::NonTerminal(id);
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }

    #[test]
    fn symbol_external_serde(id in symbol_id_strategy()) {
        let sym = Symbol::External(id);
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }

    #[test]
    fn symbol_optional_serde(inner in leaf_symbol_strategy()) {
        let sym = Symbol::Optional(Box::new(inner));
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }

    #[test]
    fn symbol_epsilon_serde(_dummy in 0..1i32) {
        let sym = Symbol::Epsilon;
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }
}

// ---------------------------------------------------------------------------
// 5. Rule struct with all combinations of precedence/associativity
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn rule_no_prec_no_assoc(lhs in symbol_id_strategy(), pid in production_id_strategy()) {
        let rule = Rule {
            lhs,
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: pid,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule, back);
    }

    #[test]
    fn rule_with_static_prec_and_assoc(
        lhs in symbol_id_strategy(),
        prec in any::<i16>(),
        assoc in associativity_strategy(),
        pid in production_id_strategy(),
    ) {
        let rule = Rule {
            lhs,
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(prec)),
            associativity: Some(assoc),
            fields: vec![],
            production_id: pid,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule.precedence, back.precedence);
        prop_assert_eq!(rule.associativity, back.associativity);
    }

    #[test]
    fn rule_with_dynamic_prec(
        lhs in symbol_id_strategy(),
        prec in any::<i16>(),
        pid in production_id_strategy(),
    ) {
        let rule = Rule {
            lhs,
            rhs: vec![Symbol::NonTerminal(SymbolId(5))],
            precedence: Some(PrecedenceKind::Dynamic(prec)),
            associativity: None,
            fields: vec![],
            production_id: pid,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule, back);
    }

    #[test]
    fn rule_with_fields(
        lhs in symbol_id_strategy(),
        fid in field_id_strategy(),
        pos in 0u16..20,
        pid in production_id_strategy(),
    ) {
        let rule = Rule {
            lhs,
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![(fid, pos as usize)],
            production_id: pid,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule.fields, back.fields);
    }

    #[test]
    fn rule_all_prec_assoc_combos(
        prec in prop::option::of(precedence_kind_strategy()),
        assoc in prop::option::of(associativity_strategy()),
        pid in production_id_strategy(),
    ) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(2))],
            precedence: prec,
            associativity: assoc,
            fields: vec![],
            production_id: pid,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule, back);
    }
}

// ---------------------------------------------------------------------------
// 6. Grammar validate on randomly generated grammars (should not panic)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn validate_random_grammar_no_panic(
        n_tokens in 0usize..8,
        n_rules in 0usize..10,
    ) {
        let mut builder = GrammarBuilder::new("rand_validate");
        for i in 0..n_tokens {
            builder = builder.token(&format!("T{i}"), &format!("t{i}"));
        }
        if n_tokens > 0 {
            for i in 0..n_rules {
                builder = builder.rule(&format!("r{i}"), vec!["T0"]);
            }
        }
        let grammar = builder.build();
        // Should never panic regardless of structure
        let _ = grammar.validate();
    }

    #[test]
    fn validator_random_grammar_no_panic(
        n_tokens in 1usize..6,
        n_rules in 1usize..8,
    ) {
        let mut builder = GrammarBuilder::new("validator_rand");
        for i in 0..n_tokens {
            builder = builder.token(&format!("T{i}"), &format!("t{i}"));
        }
        for i in 0..n_rules {
            builder = builder.rule(&format!("r{i}"), vec!["T0"]);
        }
        let grammar = builder.build();
        let mut validator = GrammarValidator::new();
        let _result = validator.validate(&grammar);
        // Just assert no panic
    }

    #[test]
    fn validate_empty_grammar_no_panic(_dummy in 0..1i32) {
        let grammar = Grammar::new("empty".to_string());
        let _ = grammar.validate();
        let mut validator = GrammarValidator::new();
        let _result = validator.validate(&grammar);
    }
}

// ---------------------------------------------------------------------------
// 7. GrammarBuilder with random token/rule names
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn builder_random_tokens(
        names in prop::collection::vec(token_name_strategy(), 1..=10),
    ) {
        let mut builder = GrammarBuilder::new("rand_tok");
        for name in &names {
            builder = builder.token(name, &name.to_lowercase());
        }
        let grammar = builder.build();
        // Unique names should each produce a token
        let unique: std::collections::HashSet<_> = names.iter().collect();
        prop_assert_eq!(grammar.tokens.len(), unique.len());
    }

    #[test]
    fn builder_random_rules(
        rule_names in prop::collection::vec(rule_name_strategy(), 1..=8),
    ) {
        let mut builder = GrammarBuilder::new("rand_rule")
            .token("A", "a");
        for name in &rule_names {
            builder = builder.rule(name, vec!["A"]);
        }
        let grammar = builder.build();
        let total: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, rule_names.len());
    }

    #[test]
    fn builder_random_names_serde_roundtrip(
        gname in "[a-z][a-z0-9]{0,10}",
        tok_name in token_name_strategy(),
        rule_name in rule_name_strategy(),
    ) {
        let grammar = GrammarBuilder::new(&gname)
            .token(&tok_name, "pat")
            .rule(&rule_name, vec![&tok_name])
            .start(&rule_name)
            .build();
        let json = serde_json::to_string(&grammar).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&grammar.name, &back.name);
    }
}

// ---------------------------------------------------------------------------
// Additional edge-case property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    // Token patterns through serde roundtrip
    #[test]
    fn token_pattern_serde_roundtrip(pat in token_pattern_strategy()) {
        let json = serde_json::to_string(&pat).unwrap();
        let back: TokenPattern = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(pat, back);
    }

    // PrecedenceKind serde roundtrip
    #[test]
    fn precedence_kind_serde_roundtrip(pk in precedence_kind_strategy()) {
        let json = serde_json::to_string(&pk).unwrap();
        let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(pk, back);
    }

    // Associativity serde roundtrip
    #[test]
    fn associativity_serde_roundtrip(a in associativity_strategy()) {
        let json = serde_json::to_string(&a).unwrap();
        let back: Associativity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(a, back);
    }

    // SymbolId ordering is consistent
    #[test]
    fn symbol_id_ordering(a in any::<u16>(), b in any::<u16>()) {
        let sa = SymbolId(a);
        let sb = SymbolId(b);
        prop_assert_eq!(sa.cmp(&sb), a.cmp(&b));
    }

    // SymbolId hash consistency (equal values produce equal hashes)
    #[test]
    fn symbol_id_hash_consistency(v in any::<u16>()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let a = SymbolId(v);
        let b = SymbolId(v);
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        prop_assert_eq!(ha.finish(), hb.finish());
    }

    // Symbol clone equality
    #[test]
    fn symbol_clone_eq(sym in symbol_strategy()) {
        let cloned = sym.clone();
        prop_assert_eq!(&sym, &cloned);
    }

    // Display format sanity for all ID types
    #[test]
    fn id_display_contains_value(v in any::<u16>()) {
        let vs = v.to_string();
        let s1 = format!("{}", SymbolId(v));
        let s2 = format!("{}", RuleId(v));
        let s3 = format!("{}", StateId(v));
        let s4 = format!("{}", FieldId(v));
        let s5 = format!("{}", ProductionId(v));
        prop_assert!(s1.contains(&vs));
        prop_assert!(s2.contains(&vs));
        prop_assert!(s3.contains(&vs));
        prop_assert!(s4.contains(&vs));
        prop_assert!(s5.contains(&vs));
    }
}

// ---------------------------------------------------------------------------
// Grammar-level edge cases
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // Grammar with precedence rules does not panic on normalize
    #[test]
    fn grammar_with_precedence_normalize(prec in any::<i16>(), assoc in associativity_strategy()) {
        let grammar = GrammarBuilder::new("prec_norm")
            .token("PLUS", "+")
            .token("NUM", r"\d+")
            .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], prec, assoc)
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let mut g = grammar;
        g.normalize();
        // Should produce valid rules
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert!(total >= 2);
    }

    // check_empty_terminals never panics
    #[test]
    fn check_empty_terminals_no_panic(
        n_tokens in 0usize..10,
    ) {
        let mut builder = GrammarBuilder::new("empty_term_check");
        for i in 0..n_tokens {
            builder = builder.token(&format!("T{i}"), &format!("t{i}"));
        }
        let grammar = builder.build();
        let _ = grammar.check_empty_terminals();
    }

    // build_registry never panics on random grammars
    #[test]
    fn build_registry_no_panic(
        n_tokens in 1usize..6,
        n_rules in 0usize..6,
    ) {
        let mut builder = GrammarBuilder::new("reg_build");
        for i in 0..n_tokens {
            builder = builder.token(&format!("T{i}"), &format!("t{i}"));
        }
        for i in 0..n_rules {
            builder = builder.rule(&format!("r{i}"), vec!["T0"]);
        }
        let grammar = builder.build();
        let _registry = grammar.build_registry();
    }
}
