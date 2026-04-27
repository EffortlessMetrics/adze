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

    // 34. Precedence determinism: constructing twice with same inputs yields equal values
    #[test]
    fn precedence_deterministic_construction(
        level in any::<i16>(),
        a in arb_associativity(),
        syms in arb_symbol_ids(5),
    ) {
        let p1 = Precedence { level, associativity: a, symbols: syms.clone() };
        let p2 = Precedence { level, associativity: a, symbols: syms };
        prop_assert_eq!(&p1, &p2);
        // Serializations must also be identical (determinism)
        let j1 = serde_json::to_string(&p1).unwrap();
        let j2 = serde_json::to_string(&p2).unwrap();
        prop_assert_eq!(j1, j2);
    }

    // 35. Precedence equality is transitive
    #[test]
    fn precedence_eq_transitive(p in arb_precedence()) {
        let q = p.clone();
        let r = q.clone();
        prop_assert_eq!(&p, &q);
        prop_assert_eq!(&q, &r);
        prop_assert_eq!(&p, &r);
    }

    // 36. Associativity Copy semantics: copied value equals original
    #[test]
    fn associativity_copy_semantics(a in arb_associativity()) {
        let b = a; // Copy
        let c = a; // Copy again
        prop_assert_eq!(a, b);
        prop_assert_eq!(a, c);
    }

    // 37. PrecedenceKind Clone preserves value
    #[test]
    fn prec_kind_clone_eq(pk in arb_precedence_kind()) {
        let cloned = pk;
        prop_assert_eq!(pk, cloned);
    }

    // 38. PrecedenceKind Debug is non-empty and contains variant name
    #[test]
    fn prec_kind_debug_contains_variant(pk in arb_precedence_kind()) {
        let dbg = format!("{pk:?}");
        prop_assert!(!dbg.is_empty());
        match pk {
            PrecedenceKind::Static(_) => prop_assert!(dbg.contains("Static")),
            PrecedenceKind::Dynamic(_) => prop_assert!(dbg.contains("Dynamic")),
        }
    }

    // 39. Precedence debug contains the level value
    #[test]
    fn precedence_debug_contains_level(level in -500i16..500) {
        let p = Precedence { level, associativity: Associativity::Left, symbols: vec![] };
        let dbg = format!("{p:?}");
        prop_assert!(dbg.contains(&level.to_string()));
    }

    // 40. Multiple precedence levels in grammar: each level is retrievable
    #[test]
    fn grammar_multiple_prec_levels_accessible(
        levels in prop::collection::vec(-100i16..100, 2..=5),
    ) {
        let precs: Vec<Precedence> = levels
            .iter()
            .enumerate()
            .map(|(i, &lv)| Precedence {
                level: lv,
                associativity: Associativity::Left,
                symbols: vec![SymbolId((i as u16) + 10)],
            })
            .collect();
        let g = grammar_with_precedences("multi", precs.clone());
        for (i, expected) in precs.iter().enumerate() {
            prop_assert_eq!(&g.precedences[i], expected);
        }
    }

    // 41. Precedence postcard serde roundtrip
    #[test]
    fn precedence_bincode_roundtrip(p in arb_precedence()) {
        let bytes = postcard::to_stdvec(&p).unwrap();
        let back: Precedence = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(&p, &back);
    }

    // 42. Associativity postcard serde roundtrip
    #[test]
    fn associativity_bincode_roundtrip(a in arb_associativity()) {
        let bytes = postcard::to_stdvec(&a).unwrap();
        let back: Associativity = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(a, back);
    }

    // 43. PrecedenceKind postcard roundtrip
    #[test]
    fn prec_kind_bincode_roundtrip(pk in arb_precedence_kind()) {
        let bytes = postcard::to_stdvec(&pk).unwrap();
        let back: PrecedenceKind = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(pk, back);
    }

    // 44. Precedence with max/min i16 levels works correctly
    #[test]
    fn precedence_extreme_levels(selector in 0u8..4) {
        let level = match selector {
            0 => i16::MIN,
            1 => i16::MAX,
            2 => 0i16,
            _ => -1i16,
        };
        let p = Precedence { level, associativity: Associativity::None, symbols: vec![] };
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(p.level, back.level);
    }

    // 45. PrecedenceKind different values within same variant are unequal
    #[test]
    fn prec_kind_different_values_ne(a in any::<i16>(), b in any::<i16>()) {
        prop_assume!(a != b);
        prop_assert_ne!(PrecedenceKind::Static(a), PrecedenceKind::Static(b));
        prop_assert_ne!(PrecedenceKind::Dynamic(a), PrecedenceKind::Dynamic(b));
    }

    // 46. Precedence symbols order matters for equality
    #[test]
    fn precedence_symbol_order_matters(
        level in any::<i16>(),
        a in arb_associativity(),
        s1 in 1u16..100,
        s2 in 101u16..200,
    ) {
        let p1 = Precedence { level, associativity: a, symbols: vec![SymbolId(s1), SymbolId(s2)] };
        let p2 = Precedence { level, associativity: a, symbols: vec![SymbolId(s2), SymbolId(s1)] };
        prop_assert_ne!(&p1, &p2);
    }

    // 47. ConflictResolution GLR variant does not interfere with Precedence variant
    #[test]
    fn conflict_resolution_variants_distinct(pk in arb_precedence_kind()) {
        let res_prec = ConflictResolution::Precedence(pk);
        let res_glr = ConflictResolution::GLR;
        prop_assert_ne!(&res_prec, &res_glr);
    }

    // 48. ConflictResolution Associativity variant roundtrips all three kinds
    #[test]
    fn conflict_resolution_assoc_variant_roundtrip(a in arb_associativity()) {
        let res = ConflictResolution::Associativity(a);
        let json = serde_json::to_string(&res).unwrap();
        let back: ConflictResolution = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&res, &back);
    }

    // 49. Precedence determinism: JSON output is byte-identical across repeated serializations
    #[test]
    fn precedence_json_deterministic(p in arb_precedence()) {
        let j1 = serde_json::to_string(&p).unwrap();
        let j2 = serde_json::to_string(&p).unwrap();
        let j3 = serde_json::to_string(&p).unwrap();
        prop_assert_eq!(&j1, &j2);
        prop_assert_eq!(&j2, &j3);
    }

    // 50. Grammar precedences survive clone
    #[test]
    fn grammar_prec_clone_preserves(precs in prop::collection::vec(arb_precedence(), 0..=3)) {
        let g = grammar_with_precedences("clone_test", precs);
        let g2 = g.clone();
        prop_assert_eq!(&g.precedences, &g2.precedences);
    }

    // 51. Rule with None precedence and Some associativity are independent
    #[test]
    fn rule_independent_prec_assoc(a in arb_associativity()) {
        let r = Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(2))],
            precedence: Option::None,
            associativity: Some(a),
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert!(r.precedence.is_none());
        prop_assert_eq!(r.associativity, Some(a));
    }

    // 52. Rule with Some precedence and None associativity are independent
    #[test]
    fn rule_prec_without_assoc(pk in arb_precedence_kind()) {
        let r = Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(2))],
            precedence: Some(pk),
            associativity: Option::None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(r.precedence, Some(pk));
        prop_assert!(r.associativity.is_none());
    }

    // 53. Precedence with large symbol list roundtrips
    #[test]
    fn precedence_large_symbols_roundtrip(
        level in any::<i16>(),
        a in arb_associativity(),
        syms in prop::collection::vec(arb_symbol_id(), 10..=20),
    ) {
        let p = Precedence { level, associativity: a, symbols: syms };
        let json = serde_json::to_string(&p).unwrap();
        let back: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&p, &back);
    }
}
