#![allow(clippy::needless_range_loop)]

//! Property-based tests for Precedence and Associativity in adze-ir.

use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, Grammar, Precedence, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
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
    (any::<i16>(), arb_associativity(), arb_symbol_ids(6)).prop_map(
        |(level, associativity, symbols)| Precedence {
            level,
            associativity,
            symbols,
        },
    )
}

fn arb_rule_with_prec() -> impl Strategy<Value = Rule> {
    (
        arb_symbol_id(),
        arb_symbol_id(),
        prop::option::of(arb_precedence_kind()),
        prop::option::of(arb_associativity()),
    )
        .prop_map(|(lhs, rhs_sym, prec, assoc)| Rule {
            lhs,
            rhs: vec![Symbol::Terminal(rhs_sym)],
            precedence: prec,
            associativity: assoc,
            fields: vec![],
            production_id: ProductionId(0),
        })
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

    // 1. Precedence creation preserves numeric level
    #[test]
    fn precedence_level_preserved(level in any::<i16>()) {
        let p = Precedence {
            level,
            associativity: Associativity::Left,
            symbols: vec![],
        };
        prop_assert_eq!(p.level, level);
    }

    // 2. Precedence equality is reflexive
    #[test]
    fn precedence_eq_reflexive(p in arb_precedence()) {
        prop_assert_eq!(&p, &p);
    }

    // 3. Precedence equality is symmetric
    #[test]
    fn precedence_eq_symmetric(p in arb_precedence()) {
        let q = p.clone();
        prop_assert_eq!(&p, &q);
        prop_assert_eq!(&q, &p);
    }

    // 4. Different levels produce unequal Precedences
    #[test]
    fn precedence_different_level_ne(a in any::<i16>(), b in any::<i16>()) {
        prop_assume!(a != b);
        let pa = Precedence { level: a, associativity: Associativity::Left, symbols: vec![] };
        let pb = Precedence { level: b, associativity: Associativity::Left, symbols: vec![] };
        prop_assert_ne!(&pa, &pb);
    }

    // 5. Precedence comparison via level ordering
    #[test]
    fn precedence_level_ordering(a in -1000i16..1000, b in -1000i16..1000) {
        let pa = Precedence { level: a, associativity: Associativity::Left, symbols: vec![] };
        let pb = Precedence { level: b, associativity: Associativity::Left, symbols: vec![] };
        if a == b {
            prop_assert_eq!(&pa, &pb);
        } else {
            prop_assert_ne!(&pa, &pb);
        }
        // Higher numeric level means higher precedence by convention
        prop_assert_eq!(pa.level < pb.level, a < b);
    }

    // 6. Precedence serde JSON roundtrip
    #[test]
    fn precedence_serde_json_roundtrip(p in arb_precedence()) {
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&p, &back);
    }

    // 7. Precedence serde pretty-JSON roundtrip
    #[test]
    fn precedence_serde_pretty_roundtrip(p in arb_precedence()) {
        let json = serde_json::to_string_pretty(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&p, &back);
    }

    // 8. Precedence clone preserves all fields
    #[test]
    fn precedence_clone_preserves_fields(p in arb_precedence()) {
        let c = p.clone();
        prop_assert_eq!(p.level, c.level);
        prop_assert_eq!(p.associativity, c.associativity);
        prop_assert_eq!(&p.symbols, &c.symbols);
    }

    // 9. Precedence debug output is non-empty
    #[test]
    fn precedence_debug_nonempty(p in arb_precedence()) {
        let dbg = format!("{:?}", p);
        prop_assert!(!dbg.is_empty());
    }

    // 10. Associativity variants are exactly three
    #[test]
    fn associativity_variant_coverage(a in arb_associativity()) {
        let valid = matches!(a, Associativity::Left | Associativity::Right | Associativity::None);
        prop_assert!(valid);
    }

    // 11. Associativity equality is reflexive
    #[test]
    fn associativity_eq_reflexive(a in arb_associativity()) {
        prop_assert_eq!(a, a);
    }

    // 12. Associativity serde roundtrip
    #[test]
    fn associativity_serde_roundtrip(a in arb_associativity()) {
        let json = serde_json::to_string(&a).unwrap();
        let back: Associativity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(a, back);
    }

    // 13. Associativity debug contains variant name
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

    // 14. Associativity Left != Right
    #[test]
    fn assoc_left_ne_right(_dummy in 0..1u8) {
        prop_assert_ne!(Associativity::Left, Associativity::Right);
    }

    // 15. Associativity Left != NonAssoc
    #[test]
    fn assoc_left_ne_none(_dummy in 0..1u8) {
        prop_assert_ne!(Associativity::Left, Associativity::None);
    }

    // 16. Associativity Right != NonAssoc
    #[test]
    fn assoc_right_ne_none(_dummy in 0..1u8) {
        prop_assert_ne!(Associativity::Right, Associativity::None);
    }

    // 17. Associativity in Rule: presence preserved through clone
    #[test]
    fn rule_assoc_preserved(r in arb_rule_with_prec()) {
        let c = r.clone();
        prop_assert_eq!(r.associativity, c.associativity);
    }

    // 18. PrecedenceKind in Rule: presence preserved through clone
    #[test]
    fn rule_prec_preserved(r in arb_rule_with_prec()) {
        let c = r.clone();
        prop_assert_eq!(r.precedence, c.precedence);
    }

    // 19. Precedence + Associativity combination: changing assoc changes Precedence
    #[test]
    fn prec_assoc_combination_changes(level in any::<i16>(), syms in arb_symbol_ids(4)) {
        let p1 = Precedence { level, associativity: Associativity::Left, symbols: syms.clone() };
        let p2 = Precedence { level, associativity: Associativity::Right, symbols: syms };
        prop_assert_ne!(&p1, &p2);
    }

    // 20. Precedence with same fields are equal regardless of construction order
    #[test]
    fn prec_field_equality(level in any::<i16>(), a in arb_associativity(), syms in arb_symbol_ids(4)) {
        let p1 = Precedence { level, associativity: a, symbols: syms.clone() };
        let p2 = Precedence { level, associativity: a, symbols: syms };
        prop_assert_eq!(&p1, &p2);
    }

    // 21. Precedence symbols count preserved through serde
    #[test]
    fn prec_symbols_count_serde(p in arb_precedence()) {
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(p.symbols.len(), back.symbols.len());
    }

    // 22. Grammar with precedences: precedence count matches
    #[test]
    fn grammar_prec_count(precs in prop::collection::vec(arb_precedence(), 0..=5)) {
        let count = precs.len();
        let g = grammar_with_precedences("test", precs);
        prop_assert_eq!(g.precedences.len(), count);
    }

    // 23. Grammar with precedences serde roundtrip preserves precedences
    #[test]
    fn grammar_prec_serde_roundtrip(precs in prop::collection::vec(arb_precedence(), 0..=3)) {
        let g = grammar_with_precedences("roundtrip", precs);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.precedences, &back.precedences);
    }

    // 24. PrecedenceKind Static equality
    #[test]
    fn prec_kind_static_eq(v in any::<i16>()) {
        prop_assert_eq!(PrecedenceKind::Static(v), PrecedenceKind::Static(v));
    }

    // 25. PrecedenceKind Dynamic equality
    #[test]
    fn prec_kind_dynamic_eq(v in any::<i16>()) {
        prop_assert_eq!(PrecedenceKind::Dynamic(v), PrecedenceKind::Dynamic(v));
    }

    // 26. PrecedenceKind Static != Dynamic even with same value
    #[test]
    fn prec_kind_static_ne_dynamic(v in any::<i16>()) {
        prop_assert_ne!(PrecedenceKind::Static(v), PrecedenceKind::Dynamic(v));
    }

    // 27. PrecedenceKind serde roundtrip
    #[test]
    fn prec_kind_serde_roundtrip(pk in arb_precedence_kind()) {
        let json = serde_json::to_string(&pk).unwrap();
        let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(pk, back);
    }

    // 28. ConflictDeclaration with Precedence resolution serde roundtrip
    #[test]
    fn conflict_prec_serde_roundtrip(
        syms in prop::collection::vec(arb_symbol_id(), 1..=4),
        pk in arb_precedence_kind(),
    ) {
        let cd = ConflictDeclaration {
            symbols: syms,
            resolution: ConflictResolution::Precedence(pk),
        };
        let json = serde_json::to_string(&cd).unwrap();
        let back: ConflictDeclaration = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cd, &back);
    }

    // 29. ConflictDeclaration with Associativity resolution serde roundtrip
    #[test]
    fn conflict_assoc_serde_roundtrip(
        syms in prop::collection::vec(arb_symbol_id(), 1..=4),
        a in arb_associativity(),
    ) {
        let cd = ConflictDeclaration {
            symbols: syms,
            resolution: ConflictResolution::Associativity(a),
        };
        let json = serde_json::to_string(&cd).unwrap();
        let back: ConflictDeclaration = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cd, &back);
    }

    // 30. Precedence with empty symbols is valid
    #[test]
    fn prec_empty_symbols(level in any::<i16>(), a in arb_associativity()) {
        let p = Precedence { level, associativity: a, symbols: vec![] };
        prop_assert!(p.symbols.is_empty());
        prop_assert_eq!(p.level, level);
    }

    // 31. Rule with both precedence and associativity roundtrips through serde
    #[test]
    fn rule_prec_assoc_serde(r in arb_rule_with_prec()) {
        let json = serde_json::to_string(&r).unwrap();
        let back: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(r.precedence, back.precedence);
        prop_assert_eq!(r.associativity, back.associativity);
    }

    // 32. Precedence different symbols produce inequality
    #[test]
    fn prec_different_symbols_ne(
        level in any::<i16>(),
        a in arb_associativity(),
        s1 in 1u16..100,
        s2 in 101u16..200,
    ) {
        let p1 = Precedence { level, associativity: a, symbols: vec![SymbolId(s1)] };
        let p2 = Precedence { level, associativity: a, symbols: vec![SymbolId(s2)] };
        prop_assert_ne!(&p1, &p2);
    }

    // 33. ConflictDeclaration with precedence: clone equals original
    #[test]
    fn conflict_prec_clone_eq(
        syms in prop::collection::vec(arb_symbol_id(), 1..=4),
        pk in arb_precedence_kind(),
    ) {
        let cd = ConflictDeclaration {
            symbols: syms,
            resolution: ConflictResolution::Precedence(pk),
        };
        let cloned = cd.clone();
        prop_assert_eq!(&cd, &cloned);
    }
}
