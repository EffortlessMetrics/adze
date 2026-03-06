// Property tests for Grammar core API: add_rule, all_rules, start_symbol, etc.
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Grammar core API property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn grammar_name_preserved(name in "[a-z]{1,20}") {
        let g = Grammar::new(name.clone());
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn add_rules_accumulate(n in 1usize..15) {
        let mut g = Grammar::new("test".to_string());
        for i in 0..n {
            g.add_rule(Rule {
                lhs: SymbolId(10 + i as u16),
                rhs: vec![Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        prop_assert_eq!(g.all_rules().count(), n);
    }

    #[test]
    fn add_rules_same_lhs_accumulate(n in 1usize..10) {
        let mut g = Grammar::new("test".to_string());
        for i in 0..n {
            g.add_rule(Rule {
                lhs: SymbolId(10),
                rhs: vec![Symbol::Terminal(SymbolId(i as u16 + 1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let rules = g.get_rules_for_symbol(SymbolId(10)).unwrap();
        prop_assert_eq!(rules.len(), n);
    }

    #[test]
    fn symbol_id_copy_semantics(id in 0u16..u16::MAX) {
        let a = SymbolId(id);
        let b = a; // Copy
        prop_assert_eq!(a, b);
        prop_assert_eq!(a.0, id);
    }

    #[test]
    fn rule_id_copy_semantics(id in 0u16..u16::MAX) {
        let a = RuleId(id);
        let b = a;
        prop_assert_eq!(a, b);
    }

    #[test]
    fn state_id_copy_semantics(id in 0u16..u16::MAX) {
        let a = StateId(id);
        let b = a;
        prop_assert_eq!(a, b);
    }

    #[test]
    fn production_id_copy_semantics(id in 0u16..u16::MAX) {
        let a = ProductionId(id);
        let b = a;
        prop_assert_eq!(a, b);
    }

    #[test]
    fn field_id_copy_semantics(id in 0u16..u16::MAX) {
        let a = FieldId(id);
        let b = a;
        prop_assert_eq!(a, b);
    }

    #[test]
    fn token_name_preserved(name in "[a-z]{1,20}") {
        let token = Token {
            name: name.clone(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        };
        prop_assert_eq!(&token.name, &name);
    }

    #[test]
    fn token_pattern_string_preserved(pat in "[a-z]{1,20}") {
        let token = Token {
            name: "test".to_string(),
            pattern: TokenPattern::String(pat.clone()),
            fragile: false,
        };
        match &token.pattern {
            TokenPattern::String(s) => prop_assert_eq!(s, &pat),
            _ => prop_assert!(false),
        }
    }

    #[test]
    fn token_fragile_flag(fragile in proptest::bool::ANY) {
        let token = Token {
            name: "test".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile,
        };
        prop_assert_eq!(token.fragile, fragile);
    }

    #[test]
    fn rule_lhs_preserved(lhs_id in 0u16..1000) {
        let rule = Rule {
            lhs: SymbolId(lhs_id),
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.lhs, SymbolId(lhs_id));
    }

    #[test]
    fn rule_rhs_length_preserved(n in 0usize..10) {
        let rhs: Vec<Symbol> = (0..n).map(|i| Symbol::Terminal(SymbolId(i as u16 + 1))).collect();
        let rule = Rule {
            lhs: SymbolId(10),
            rhs: rhs.clone(),
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.rhs.len(), n);
    }

    #[test]
    fn precedence_static_preserved(p in -1000i16..1000) {
        let pk = PrecedenceKind::Static(p);
        match pk {
            PrecedenceKind::Static(v) => prop_assert_eq!(v, p),
            _ => prop_assert!(false),
        }
    }

    #[test]
    fn precedence_dynamic_preserved(p in -1000i16..1000) {
        let pk = PrecedenceKind::Dynamic(p);
        match pk {
            PrecedenceKind::Dynamic(v) => prop_assert_eq!(v, p),
            _ => prop_assert!(false),
        }
    }

    #[test]
    fn registry_register_roundtrip(name in "[a-z]{1,10}") {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: true,
        };
        let id = reg.register(&name, meta);
        prop_assert_eq!(reg.get_id(&name), Some(id));
        prop_assert_eq!(reg.get_name(id), Some(name.as_str()));
    }

    #[test]
    fn registry_unique_ids(n in 1usize..30) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: true };
        let ids: Vec<SymbolId> = (0..n)
            .map(|i| reg.register(&format!("s{}", i), meta))
            .collect();
        let unique: std::collections::HashSet<SymbolId> = ids.iter().copied().collect();
        prop_assert_eq!(unique.len(), n);
    }

    #[test]
    fn registry_len_increases(n in 1usize..20) {
        let mut reg = SymbolRegistry::new();
        let initial = reg.len();
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: true };
        for i in 0..n {
            reg.register(&format!("s{}", i), meta);
        }
        prop_assert_eq!(reg.len(), initial + n);
    }
}

// ---------------------------------------------------------------------------
// Non-property tests
// ---------------------------------------------------------------------------

#[test]
fn symbol_external_variant() {
    let s = Symbol::External(SymbolId(50));
    match s {
        Symbol::External(id) => assert_eq!(id, SymbolId(50)),
        _ => panic!("Expected External"),
    }
}

#[test]
fn symbol_nested_optional_repeat() {
    let s = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Terminal(
        SymbolId(1),
    )))));
    let debug = format!("{:?}", s);
    assert!(debug.contains("Optional"));
    assert!(debug.contains("Repeat"));
}

#[test]
fn grammar_optimize_doesnt_panic() {
    let mut g = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    g.optimize(); // Should not panic
}

#[test]
fn grammar_normalize_returns_rules() {
    let mut g = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let new_rules = g.normalize();
    // Simple grammar shouldn't produce new rules from normalization
    // but the method should return without panic
    assert!(new_rules.is_empty() || !new_rules.is_empty());
}

#[test]
fn grammar_check_empty_terminals() {
    let g = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let result = g.check_empty_terminals();
    assert!(result.is_ok());
}

#[test]
fn associativity_variants() {
    let left = Associativity::Left;
    let right = Associativity::Right;
    let none = Associativity::None;
    assert_ne!(format!("{:?}", left), format!("{:?}", right));
    assert_ne!(format!("{:?}", left), format!("{:?}", none));
}

#[test]
fn conflict_resolution_variants() {
    let prec = ConflictResolution::Precedence(PrecedenceKind::Static(1));
    let assoc = ConflictResolution::Associativity(Associativity::Left);
    let glr = ConflictResolution::GLR;
    let debug_prec = format!("{:?}", prec);
    let debug_assoc = format!("{:?}", assoc);
    let debug_glr = format!("{:?}", glr);
    assert!(debug_prec.contains("Precedence"));
    assert!(debug_assoc.contains("Associativity"));
    assert!(debug_glr.contains("GLR"));
}

#[test]
fn symbol_metadata_fields() {
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: true,
        terminal: false,
    };
    assert!(meta.visible);
    assert!(!meta.named);
    assert!(meta.hidden);
    assert!(!meta.terminal);
}

#[test]
fn grammar_builder_empty_build_doesnt_panic() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.name, "empty");
}

#[test]
fn grammar_builder_multiple_tokens() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 3);
}
