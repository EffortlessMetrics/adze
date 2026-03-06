#![allow(clippy::needless_range_loop)]

//! Property-based tests for ConflictResolution and ConflictDeclaration in adze-ir.

use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, Grammar, Precedence, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (1u16..150).prop_map(SymbolId)
}

fn arb_symbol_ids(max_len: usize) -> impl Strategy<Value = Vec<SymbolId>> {
    prop::collection::vec(arb_symbol_id(), 1..=max_len)
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
        (-50i16..50).prop_map(PrecedenceKind::Static),
        (-50i16..50).prop_map(PrecedenceKind::Dynamic),
    ]
}

fn arb_conflict_resolution() -> impl Strategy<Value = ConflictResolution> {
    prop_oneof![
        arb_precedence_kind().prop_map(ConflictResolution::Precedence),
        arb_associativity().prop_map(ConflictResolution::Associativity),
        Just(ConflictResolution::GLR),
    ]
}

fn arb_conflict_declaration() -> impl Strategy<Value = ConflictDeclaration> {
    (arb_symbol_ids(5), arb_conflict_resolution()).prop_map(|(symbols, resolution)| {
        ConflictDeclaration {
            symbols,
            resolution,
        }
    })
}

fn arb_conflict_declarations(max_len: usize) -> impl Strategy<Value = Vec<ConflictDeclaration>> {
    prop::collection::vec(arb_conflict_declaration(), 0..=max_len)
}

/// Build a minimal grammar that includes the given conflicts and registers all
/// referenced symbol IDs so the grammar passes validation.
fn make_grammar(name: &str, conflicts: Vec<ConflictDeclaration>) -> Grammar {
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

    for cd in &conflicts {
        for sid in &cd.symbols {
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

    g.conflicts = conflicts;
    g
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 1. ConflictResolution::GLR is always equal to itself
    #[test]
    fn glr_variant_eq(_dummy in 0u8..1) {
        let a = ConflictResolution::GLR;
        let b = ConflictResolution::GLR;
        prop_assert_eq!(a, b);
    }

    // 2. ConflictResolution::Precedence roundtrips through serde JSON
    #[test]
    fn precedence_variant_serde_roundtrip(pk in arb_precedence_kind()) {
        let res = ConflictResolution::Precedence(pk);
        let json = serde_json::to_string(&res).unwrap();
        let back: ConflictResolution = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(res, back);
    }

    // 3. ConflictResolution::Associativity roundtrips through serde JSON
    #[test]
    fn associativity_variant_serde_roundtrip(a in arb_associativity()) {
        let res = ConflictResolution::Associativity(a);
        let json = serde_json::to_string(&res).unwrap();
        let back: ConflictResolution = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(res, back);
    }

    // 4. ConflictResolution::GLR roundtrips through serde JSON
    #[test]
    fn glr_variant_serde_roundtrip(_dummy in 0u8..1) {
        let res = ConflictResolution::GLR;
        let json = serde_json::to_string(&res).unwrap();
        let back: ConflictResolution = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(res, back);
    }

    // 5. ConflictResolution clone produces equal value
    #[test]
    fn resolution_clone_equality(res in arb_conflict_resolution()) {
        let cloned = res.clone();
        prop_assert_eq!(&res, &cloned);
    }

    // 6. ConflictResolution Debug output is non-empty
    #[test]
    fn resolution_debug_nonempty(res in arb_conflict_resolution()) {
        let dbg = format!("{:?}", res);
        prop_assert!(!dbg.is_empty());
    }

    // 7. ConflictResolution Debug contains variant discriminant
    #[test]
    fn resolution_debug_discriminant(res in arb_conflict_resolution()) {
        let dbg = format!("{res:?}");
        match &res {
            ConflictResolution::Precedence(_) => prop_assert!(dbg.contains("Precedence")),
            ConflictResolution::Associativity(_) => prop_assert!(dbg.contains("Associativity")),
            ConflictResolution::GLR => prop_assert!(dbg.contains("GLR")),
        }
    }

    // 8. Two resolutions built from different variants are not equal
    #[test]
    fn different_variant_not_eq(pk in arb_precedence_kind()) {
        let a = ConflictResolution::Precedence(pk);
        let b = ConflictResolution::GLR;
        prop_assert_ne!(a, b);
    }

    // 9. Precedence(Static(n)) != Precedence(Dynamic(n)) for same n
    #[test]
    fn static_ne_dynamic(n in -50i16..50) {
        let a = ConflictResolution::Precedence(PrecedenceKind::Static(n));
        let b = ConflictResolution::Precedence(PrecedenceKind::Dynamic(n));
        prop_assert_ne!(a, b);
    }

    // 10. Associativity(Left) != Associativity(Right)
    #[test]
    fn left_ne_right(_dummy in 0u8..1) {
        let a = ConflictResolution::Associativity(Associativity::Left);
        let b = ConflictResolution::Associativity(Associativity::Right);
        prop_assert_ne!(a, b);
    }

    // 11. ConflictDeclaration serde JSON roundtrip
    #[test]
    fn declaration_serde_roundtrip(cd in arb_conflict_declaration()) {
        let json = serde_json::to_string(&cd).unwrap();
        let back: ConflictDeclaration = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cd, &back);
    }

    // 12. ConflictDeclaration serde pretty-JSON roundtrip
    #[test]
    fn declaration_serde_pretty_roundtrip(cd in arb_conflict_declaration()) {
        let json = serde_json::to_string_pretty(&cd).unwrap();
        let back: ConflictDeclaration = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cd, &back);
    }

    // 13. ConflictDeclaration clone preserves symbol count
    #[test]
    fn declaration_clone_symbol_count(cd in arb_conflict_declaration()) {
        prop_assert_eq!(cd.symbols.len(), cd.clone().symbols.len());
    }

    // 14. ConflictDeclaration clone preserves resolution
    #[test]
    fn declaration_clone_resolution(cd in arb_conflict_declaration()) {
        prop_assert_eq!(&cd.resolution, &cd.clone().resolution);
    }

    // 15. ConflictDeclaration Debug contains "ConflictDeclaration"
    #[test]
    fn declaration_debug_typename(cd in arb_conflict_declaration()) {
        let dbg = format!("{:?}", cd);
        prop_assert!(dbg.contains("ConflictDeclaration"));
    }

    // 16. ConflictDeclaration JSON has "symbols" and "resolution" keys
    #[test]
    fn declaration_json_keys(cd in arb_conflict_declaration()) {
        let json = serde_json::to_string(&cd).unwrap();
        prop_assert!(json.contains("\"symbols\""));
        prop_assert!(json.contains("\"resolution\""));
    }

    // 17. ConflictDeclaration equality is reflexive
    #[test]
    fn declaration_eq_reflexive(cd in arb_conflict_declaration()) {
        prop_assert_eq!(&cd, &cd);
    }

    // 18. ConflictDeclaration equality is symmetric
    #[test]
    fn declaration_eq_symmetric(cd in arb_conflict_declaration()) {
        let cd2 = cd.clone();
        prop_assert!(cd == cd2);
    }

    // 19. ConflictDeclaration with different symbols are not equal
    #[test]
    fn declaration_diff_symbols_ne(
        s1 in arb_symbol_ids(3),
        s2 in arb_symbol_ids(3),
        res in arb_conflict_resolution(),
    ) {
        let a = ConflictDeclaration { symbols: s1.clone(), resolution: res.clone() };
        let b = ConflictDeclaration { symbols: s2.clone(), resolution: res };
        if s1 != s2 {
            prop_assert_ne!(&a, &b);
        }
    }

    // 20. ConflictDeclaration with different resolutions are not equal
    #[test]
    fn declaration_diff_resolution_ne(
        syms in arb_symbol_ids(3),
        r1 in arb_conflict_resolution(),
        r2 in arb_conflict_resolution(),
    ) {
        let a = ConflictDeclaration { symbols: syms.clone(), resolution: r1.clone() };
        let b = ConflictDeclaration { symbols: syms, resolution: r2.clone() };
        if r1 != r2 {
            prop_assert_ne!(&a, &b);
        }
    }

    // 21. Grammar with conflicts validates successfully
    #[test]
    fn grammar_with_conflicts_validates(cds in arb_conflict_declarations(5)) {
        let g = make_grammar("validate_test", cds);
        prop_assert!(g.validate().is_ok());
    }

    // 22. Grammar serde roundtrip preserves conflicts
    #[test]
    fn grammar_serde_preserves_conflicts(cds in arb_conflict_declarations(4)) {
        let g = make_grammar("grammar_serde", cds);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.conflicts, &back.conflicts);
    }

    // 23. Grammar normalize preserves conflicts
    #[test]
    fn grammar_normalize_preserves_conflicts(cds in arb_conflict_declarations(4)) {
        let mut g = make_grammar("norm", cds.clone());
        g.normalize();
        prop_assert_eq!(&g.conflicts, &cds);
    }

    // 24. Grammar clone preserves conflicts
    #[test]
    fn grammar_clone_preserves_conflicts(cds in arb_conflict_declarations(4)) {
        let g = make_grammar("clone_g", cds);
        let g2 = g.clone();
        prop_assert_eq!(&g.conflicts, &g2.conflicts);
    }

    // 25. Vec<ConflictDeclaration> serde roundtrip
    #[test]
    fn vec_declarations_serde_roundtrip(cds in arb_conflict_declarations(6)) {
        let json = serde_json::to_string(&cds).unwrap();
        let back: Vec<ConflictDeclaration> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cds, &back);
    }

    // 26. Conflict with Static precedence — Debug contains "Static"
    #[test]
    fn static_precedence_debug(level in -50i16..50) {
        let res = ConflictResolution::Precedence(PrecedenceKind::Static(level));
        let dbg = format!("{:?}", res);
        prop_assert!(dbg.contains("Static"));
    }

    // 27. Conflict with Dynamic precedence — Debug contains "Dynamic"
    #[test]
    fn dynamic_precedence_debug(level in -50i16..50) {
        let res = ConflictResolution::Precedence(PrecedenceKind::Dynamic(level));
        let dbg = format!("{:?}", res);
        prop_assert!(dbg.contains("Dynamic"));
    }

    // 28. ConflictDeclaration symbols are all positive
    #[test]
    fn declaration_symbols_positive(cd in arb_conflict_declaration()) {
        for sid in &cd.symbols {
            prop_assert!(sid.0 > 0);
        }
    }

    // 29. Grammar default has empty conflicts
    #[test]
    fn default_grammar_empty_conflicts(_dummy in 0u8..1) {
        let g = Grammar::default();
        prop_assert!(g.conflicts.is_empty());
    }

    // 30. Double normalize is idempotent for conflicts
    #[test]
    fn double_normalize_idempotent(cds in arb_conflict_declarations(3)) {
        let mut g = make_grammar("double_norm", cds.clone());
        g.normalize();
        g.normalize();
        prop_assert_eq!(&g.conflicts, &cds);
    }

    // 31. Conflict with Precedence interacts with Grammar precedence declarations
    #[test]
    fn conflict_with_grammar_precedences(
        level in -20i16..20,
        assoc in arb_associativity(),
        syms in arb_symbol_ids(3),
    ) {
        let mut g = make_grammar("prec_test", vec![
            ConflictDeclaration {
                symbols: syms.clone(),
                resolution: ConflictResolution::Precedence(PrecedenceKind::Static(level)),
            },
        ]);
        g.precedences.push(Precedence {
            level,
            associativity: assoc,
            symbols: syms,
        });
        prop_assert!(g.validate().is_ok());
    }

    // 32. Multiple conflicts with mixed resolution strategies
    #[test]
    fn mixed_resolution_strategies(
        syms1 in arb_symbol_ids(3),
        syms2 in arb_symbol_ids(3),
        syms3 in arb_symbol_ids(3),
        pk in arb_precedence_kind(),
        assoc in arb_associativity(),
    ) {
        let conflicts = vec![
            ConflictDeclaration { symbols: syms1, resolution: ConflictResolution::Precedence(pk) },
            ConflictDeclaration { symbols: syms2, resolution: ConflictResolution::Associativity(assoc) },
            ConflictDeclaration { symbols: syms3, resolution: ConflictResolution::GLR },
        ];
        let g = make_grammar("mixed", conflicts.clone());
        prop_assert_eq!(g.conflicts.len(), 3);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.conflicts, &back.conflicts);
    }

    // 33. Appending conflicts preserves ordering
    #[test]
    fn append_preserves_order(
        cds1 in arb_conflict_declarations(3),
        cds2 in arb_conflict_declarations(3),
    ) {
        let mut g = make_grammar("append", cds1.clone());
        for cd in &cds2 {
            for sid in &cd.symbols {
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
        g.conflicts.extend(cds2.clone());
        let total = cds1.len() + cds2.len();
        prop_assert_eq!(g.conflicts.len(), total);
        for i in 0..cds1.len() {
            prop_assert_eq!(&g.conflicts[i], &cds1[i]);
        }
        for i in 0..cds2.len() {
            prop_assert_eq!(&g.conflicts[cds1.len() + i], &cds2[i]);
        }
    }

    // 34. Resolution variant discriminant preserved through JSON
    #[test]
    fn resolution_json_variant_tag(res in arb_conflict_resolution()) {
        let json = serde_json::to_string(&res).unwrap();
        match &res {
            ConflictResolution::Precedence(_) => prop_assert!(json.contains("Precedence")),
            ConflictResolution::Associativity(_) => prop_assert!(json.contains("Associativity")),
            ConflictResolution::GLR => prop_assert!(json.contains("GLR")),
        }
    }

    // 35. Single conflict grammar validates after normalize
    #[test]
    fn single_conflict_normalize_validates(cd in arb_conflict_declaration()) {
        let mut g = make_grammar("single_norm", vec![cd]);
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }
}
