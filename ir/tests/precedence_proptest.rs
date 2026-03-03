#![allow(clippy::needless_range_loop)]

//! Property-based tests for Precedence, PrecedenceKind, and Associativity in adze-ir.

use adze_ir::{
    Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
    Token, TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (1u16..200).prop_map(SymbolId)
}

fn arb_symbol_ids(max_len: usize) -> impl Strategy<Value = Vec<SymbolId>> {
    prop::collection::vec(arb_symbol_id(), 0..=max_len)
}

fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

fn arb_precedence_kind() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        any::<i16>().prop_map(PrecedenceKind::Static),
        any::<i16>().prop_map(PrecedenceKind::Dynamic),
    ]
}

fn arb_precedence() -> impl Strategy<Value = Precedence> {
    (any::<i16>(), arb_associativity(), arb_symbol_ids(8)).prop_map(
        |(level, associativity, symbols)| Precedence {
            level,
            associativity,
            symbols,
        },
    )
}

fn arb_precedences(max_len: usize) -> impl Strategy<Value = Vec<Precedence>> {
    prop::collection::vec(arb_precedence(), 0..=max_len)
}

/// Build a minimal grammar seeded with precedence declarations.
fn grammar_with_precedences(name: &str, precedences: Vec<Precedence>) -> Grammar {
    let mut g = Grammar::new(name.to_string());

    let start = SymbolId(0);
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(start, "start".into());
    g.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // Register every symbol referenced by a precedence declaration.
    for prec in &precedences {
        for sid in &prec.symbols {
            if !g.tokens.contains_key(sid) && !g.rules.contains_key(sid) {
                g.tokens.insert(
                    *sid,
                    Token {
                        name: format!("t{}", sid.0),
                        pattern: TokenPattern::String(format!("t{}", sid.0)),
                        fragile: false,
                    },
                );
            }
        }
    }

    g.precedences = precedences;
    g
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Precedence JSON serde roundtrip
    #[test]
    fn precedence_serde_roundtrip_json(p in arb_precedence()) {
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&p, &back);
    }

    // 2. Precedence pretty-JSON serde roundtrip
    #[test]
    fn precedence_serde_roundtrip_pretty(p in arb_precedence()) {
        let json = serde_json::to_string_pretty(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&p, &back);
    }

    // 3. PrecedenceKind JSON serde roundtrip
    #[test]
    fn precedence_kind_serde_roundtrip(pk in arb_precedence_kind()) {
        let json = serde_json::to_string(&pk).unwrap();
        let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&pk, &back);
    }

    // 4. Associativity JSON serde roundtrip
    #[test]
    fn associativity_serde_roundtrip(a in arb_associativity()) {
        let json = serde_json::to_string(&a).unwrap();
        let back: Associativity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&a, &back);
    }

    // 5. Clone equals original for Precedence
    #[test]
    fn precedence_clone_eq(p in arb_precedence()) {
        let cloned = p.clone();
        prop_assert_eq!(&p, &cloned);
    }

    // 6. Clone equals original for PrecedenceKind
    #[test]
    fn precedence_kind_clone_eq(pk in arb_precedence_kind()) {
        let cloned = pk;
        prop_assert_eq!(&pk, &cloned);
    }

    // 7. Clone equals original for Associativity
    #[test]
    fn associativity_clone_eq(a in arb_associativity()) {
        let cloned = a;
        prop_assert_eq!(&a, &cloned);
    }

    // 8. Debug is non-empty for Precedence
    #[test]
    fn precedence_debug_non_empty(p in arb_precedence()) {
        let dbg = format!("{p:?}");
        prop_assert!(!dbg.is_empty());
    }

    // 9. Debug contains "Precedence" for Precedence
    #[test]
    fn precedence_debug_contains_type(p in arb_precedence()) {
        let dbg = format!("{p:?}");
        prop_assert!(dbg.contains("Precedence"));
    }

    // 10. Debug contains variant name for PrecedenceKind
    #[test]
    fn precedence_kind_debug_variant(pk in arb_precedence_kind()) {
        let dbg = format!("{pk:?}");
        let expected = match pk {
            PrecedenceKind::Static(_) => "Static",
            PrecedenceKind::Dynamic(_) => "Dynamic",
        };
        prop_assert!(dbg.contains(expected));
    }

    // 11. Debug contains variant name for Associativity
    #[test]
    fn associativity_debug_variant(a in arb_associativity()) {
        let dbg = format!("{a:?}");
        let expected = match a {
            Associativity::Left => "Left",
            Associativity::Right => "Right",
            Associativity::None => "None",
        };
        prop_assert!(dbg.contains(expected));
    }

    // 12. PrecedenceKind Static != Dynamic even with same value
    #[test]
    fn static_ne_dynamic_same_value(v in any::<i16>()) {
        prop_assert_ne!(PrecedenceKind::Static(v), PrecedenceKind::Dynamic(v));
    }

    // 13. PrecedenceKind equality is reflexive
    #[test]
    fn precedence_kind_reflexive(pk in arb_precedence_kind()) {
        prop_assert_eq!(&pk, &pk);
    }

    // 14. Precedence equality is reflexive
    #[test]
    fn precedence_reflexive(p in arb_precedence()) {
        prop_assert_eq!(&p, &p);
    }

    // 15. Associativity equality is reflexive
    #[test]
    fn associativity_reflexive(a in arb_associativity()) {
        prop_assert_eq!(&a, &a);
    }

    // 16. Precedence with different levels are not equal
    #[test]
    fn different_levels_not_equal(
        a in -1000i16..1000,
        b in -1000i16..1000,
        assoc in arb_associativity(),
        syms in arb_symbol_ids(4),
    ) {
        prop_assume!(a != b);
        let p1 = Precedence { level: a, associativity: assoc, symbols: syms.clone() };
        let p2 = Precedence { level: b, associativity: assoc, symbols: syms };
        prop_assert_ne!(&p1, &p2);
    }

    // 17. Precedence with different associativity are not equal
    #[test]
    fn different_assoc_not_equal(
        level in any::<i16>(),
        syms in arb_symbol_ids(4),
    ) {
        let p1 = Precedence { level, associativity: Associativity::Left, symbols: syms.clone() };
        let p2 = Precedence { level, associativity: Associativity::Right, symbols: syms };
        prop_assert_ne!(&p1, &p2);
    }

    // 18. Precedence symbols field preserved through serde
    #[test]
    fn precedence_symbols_preserved(p in arb_precedence()) {
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(p.symbols.len(), back.symbols.len());
        for i in 0..p.symbols.len() {
            prop_assert_eq!(p.symbols[i], back.symbols[i]);
        }
    }

    // 19. Precedence level preserved through serde
    #[test]
    fn precedence_level_preserved(p in arb_precedence()) {
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(p.level, back.level);
    }

    // 20. PrecedenceKind Copy semantics
    #[test]
    fn precedence_kind_copy(pk in arb_precedence_kind()) {
        let copy = pk;
        prop_assert_eq!(&pk, &copy);
    }

    // 21. Associativity Copy semantics
    #[test]
    fn associativity_copy(a in arb_associativity()) {
        let copy = a;
        prop_assert_eq!(&a, &copy);
    }

    // 22. Grammar with precedences roundtrips through JSON
    #[test]
    fn grammar_with_precs_serde_roundtrip(precs in arb_precedences(5)) {
        let g = grammar_with_precedences("test_grammar", precs);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(g.precedences.len(), back.precedences.len());
        for i in 0..g.precedences.len() {
            prop_assert_eq!(&g.precedences[i], &back.precedences[i]);
        }
    }

    // 23. Grammar precedences count matches input
    #[test]
    fn grammar_prec_count_matches(precs in arb_precedences(8)) {
        let count = precs.len();
        let g = grammar_with_precedences("count_test", precs);
        prop_assert_eq!(g.precedences.len(), count);
    }

    // 24. Multiple precedences maintain order in grammar
    #[test]
    fn grammar_prec_order_preserved(precs in arb_precedences(6)) {
        let levels: Vec<i16> = precs.iter().map(|p| p.level).collect();
        let g = grammar_with_precedences("order_test", precs);
        let stored_levels: Vec<i16> = g.precedences.iter().map(|p| p.level).collect();
        prop_assert_eq!(&levels, &stored_levels);
    }

    // 25. Sorting precedences by level is stable relative to level ordering
    #[test]
    fn precedences_sortable_by_level(precs in arb_precedences(8)) {
        let mut sorted = precs.clone();
        sorted.sort_by_key(|p| p.level);
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1].level <= sorted[i].level);
        }
    }

    // 26. PrecedenceKind inner value extractable
    #[test]
    fn precedence_kind_inner_value(v in any::<i16>()) {
        let s = PrecedenceKind::Static(v);
        let d = PrecedenceKind::Dynamic(v);
        let sv = match s { PrecedenceKind::Static(x) => x, PrecedenceKind::Dynamic(x) => x };
        let dv = match d { PrecedenceKind::Static(x) => x, PrecedenceKind::Dynamic(x) => x };
        prop_assert_eq!(sv, v);
        prop_assert_eq!(dv, v);
    }

    // 27. Associativity exhaustive — all three variants are reachable
    #[test]
    fn associativity_all_variants(idx in 0u8..3) {
        let a = match idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Associativity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&a, &back);
    }

    // 28. PrecedenceKind i16 boundary roundtrip
    #[test]
    fn precedence_kind_boundary_roundtrip(is_static in proptest::bool::ANY) {
        let vals = [i16::MIN, -1, 0, 1, i16::MAX];
        for &v in &vals {
            let pk = if is_static {
                PrecedenceKind::Static(v)
            } else {
                PrecedenceKind::Dynamic(v)
            };
            let json = serde_json::to_string(&pk).unwrap();
            let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&pk, &back);
        }
    }

    // 29. Precedence with empty symbols roundtrips
    #[test]
    fn precedence_empty_symbols_roundtrip(level in any::<i16>(), assoc in arb_associativity()) {
        let p = Precedence { level, associativity: assoc, symbols: vec![] };
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&p, &back);
        prop_assert!(back.symbols.is_empty());
    }

    // 30. Grammar precedences accessible after clone
    #[test]
    fn grammar_clone_preserves_precs(precs in arb_precedences(4)) {
        let g = grammar_with_precedences("clone_test", precs);
        let g2 = g.clone();
        prop_assert_eq!(&g.precedences, &g2.precedences);
    }

    // 31. Rule with precedence and associativity roundtrips
    #[test]
    fn rule_with_prec_roundtrip(pk in arb_precedence_kind(), assoc in arb_associativity()) {
        let rule = Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(pk),
            associativity: Some(assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule.precedence, &back.precedence);
        prop_assert_eq!(&rule.associativity, &back.associativity);
    }

    // 32. Rule with no precedence roundtrips
    #[test]
    fn rule_without_prec_roundtrip(assoc in arb_associativity()) {
        let rule = Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: Some(assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert!(back.precedence.is_none());
        prop_assert_eq!(&rule.associativity, &back.associativity);
    }

    // 33. Precedence level sign is preserved
    #[test]
    fn precedence_level_sign_preserved(level in any::<i16>()) {
        let p = Precedence { level, associativity: Associativity::None, symbols: vec![] };
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(level.signum(), back.level.signum());
    }
}
