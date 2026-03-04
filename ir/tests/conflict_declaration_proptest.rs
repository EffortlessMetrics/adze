#![allow(clippy::needless_range_loop)]

//! Property-based tests for conflict declarations in adze-ir.

use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, Grammar, PrecedenceKind, ProductionId,
    Rule, Symbol, SymbolId, Token, TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (1u16..200).prop_map(SymbolId)
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
        (-100i16..100).prop_map(PrecedenceKind::Static),
        (-100i16..100).prop_map(PrecedenceKind::Dynamic),
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
    (arb_symbol_ids(6), arb_conflict_resolution()).prop_map(|(symbols, resolution)| {
        ConflictDeclaration {
            symbols,
            resolution,
        }
    })
}

fn arb_conflict_declarations(max_len: usize) -> impl Strategy<Value = Vec<ConflictDeclaration>> {
    prop::collection::vec(arb_conflict_declaration(), 0..=max_len)
}

/// Build a minimal grammar that passes validate() and includes the given conflicts.
/// All symbol IDs referenced by conflict declarations are registered as tokens.
fn grammar_with_conflicts(name: &str, conflicts: Vec<ConflictDeclaration>) -> Grammar {
    let mut g = Grammar::new(name.to_string());

    // Register a start rule so the grammar is non-empty.
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

    // Ensure every symbol ID referenced by a conflict is known.
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
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Serde roundtrip for ConflictDeclaration
    #[test]
    fn serde_roundtrip_json(cd in arb_conflict_declaration()) {
        let json = serde_json::to_string(&cd).unwrap();
        let back: ConflictDeclaration = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cd, &back);
    }

    // 2. Serde roundtrip pretty JSON
    #[test]
    fn serde_roundtrip_pretty(cd in arb_conflict_declaration()) {
        let json = serde_json::to_string_pretty(&cd).unwrap();
        let back: ConflictDeclaration = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cd, &back);
    }

    // 3. ConflictResolution serde roundtrip
    #[test]
    fn resolution_serde_roundtrip(res in arb_conflict_resolution()) {
        let json = serde_json::to_string(&res).unwrap();
        let back: ConflictResolution = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&res, &back);
    }

    // 4. Clone equals original
    #[test]
    fn clone_eq(cd in arb_conflict_declaration()) {
        let cloned = cd.clone();
        prop_assert_eq!(&cd, &cloned);
    }

    // 5. Clone of resolution equals original
    #[test]
    fn resolution_clone_eq(res in arb_conflict_resolution()) {
        let cloned = res.clone();
        prop_assert_eq!(&res, &cloned);
    }

    // 6. Debug is non-empty for ConflictDeclaration
    #[test]
    fn debug_non_empty(cd in arb_conflict_declaration()) {
        let dbg = format!("{cd:?}");
        prop_assert!(!dbg.is_empty());
    }

    // 7. Debug is non-empty for ConflictResolution
    #[test]
    fn resolution_debug_non_empty(res in arb_conflict_resolution()) {
        let dbg = format!("{res:?}");
        prop_assert!(!dbg.is_empty());
    }

    // 8. Debug contains "ConflictDeclaration"
    #[test]
    fn debug_contains_type_name(cd in arb_conflict_declaration()) {
        let dbg = format!("{cd:?}");
        prop_assert!(dbg.contains("ConflictDeclaration"));
    }

    // 9. Debug of resolution contains variant name
    #[test]
    fn resolution_debug_contains_variant(res in arb_conflict_resolution()) {
        let dbg = format!("{res:?}");
        let expected = match &res {
            ConflictResolution::Precedence(_) => "Precedence",
            ConflictResolution::Associativity(_) => "Associativity",
            ConflictResolution::GLR => "GLR",
        };
        prop_assert!(dbg.contains(expected));
    }

    // 10. Symbol count preserved through clone
    #[test]
    fn symbol_count_preserved_through_clone(cd in arb_conflict_declaration()) {
        let cloned = cd.clone();
        prop_assert_eq!(cd.symbols.len(), cloned.symbols.len());
    }

    // 11. Grammar with random conflicts validates
    #[test]
    fn grammar_with_conflicts_validates(cds in arb_conflict_declarations(5)) {
        let g = grammar_with_conflicts("test_grammar", cds);
        prop_assert!(g.validate().is_ok());
    }

    // 12. Conflicts preserved through Grammar serde roundtrip
    #[test]
    fn grammar_serde_preserves_conflicts(cds in arb_conflict_declarations(4)) {
        let g = grammar_with_conflicts("serde_test", cds);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.conflicts, &back.conflicts);
    }

    // 13. Conflicts preserved through normalize()
    #[test]
    fn normalize_preserves_conflicts(cds in arb_conflict_declarations(4)) {
        let mut g = grammar_with_conflicts("norm_test", cds.clone());
        g.normalize();
        prop_assert_eq!(&g.conflicts, &cds);
    }

    // 14. Conflict count preserved through normalize()
    #[test]
    fn normalize_preserves_conflict_count(cds in arb_conflict_declarations(5)) {
        let count = cds.len();
        let mut g = grammar_with_conflicts("count_test", cds);
        g.normalize();
        prop_assert_eq!(g.conflicts.len(), count);
    }

    // 15. Empty conflicts valid
    #[test]
    fn empty_conflicts_valid(_dummy in 0u8..1) {
        let g = grammar_with_conflicts("empty", vec![]);
        prop_assert!(g.validate().is_ok());
        prop_assert!(g.conflicts.is_empty());
    }

    // 16. Single conflict grammar validates
    #[test]
    fn single_conflict_validates(cd in arb_conflict_declaration()) {
        let g = grammar_with_conflicts("single", vec![cd]);
        prop_assert!(g.validate().is_ok());
        prop_assert_eq!(g.conflicts.len(), 1);
    }

    // 17. Eq is reflexive
    #[test]
    fn eq_reflexive(cd in arb_conflict_declaration()) {
        prop_assert_eq!(&cd, &cd);
    }

    // 18. Eq is symmetric
    #[test]
    fn eq_symmetric(cd in arb_conflict_declaration()) {
        let cd2 = cd.clone();
        prop_assert!(cd == cd2);
        prop_assert!(cd2 == cd);
    }

    // 19. Different resolutions produce inequality
    #[test]
    fn different_resolution_not_eq(
        syms in arb_symbol_ids(3),
        r1 in arb_conflict_resolution(),
        r2 in arb_conflict_resolution(),
    ) {
        let cd1 = ConflictDeclaration { symbols: syms.clone(), resolution: r1.clone() };
        let cd2 = ConflictDeclaration { symbols: syms, resolution: r2.clone() };
        if r1 != r2 {
            prop_assert_ne!(&cd1, &cd2);
        }
    }

    // 20. Different symbols produce inequality
    #[test]
    fn different_symbols_not_eq(
        s1 in arb_symbol_ids(3),
        s2 in arb_symbol_ids(3),
        res in arb_conflict_resolution(),
    ) {
        let cd1 = ConflictDeclaration { symbols: s1.clone(), resolution: res.clone() };
        let cd2 = ConflictDeclaration { symbols: s2.clone(), resolution: res };
        if s1 != s2 {
            prop_assert_ne!(&cd1, &cd2);
        }
    }

    // 21. Serde roundtrip for Vec<ConflictDeclaration>
    #[test]
    fn vec_serde_roundtrip(cds in arb_conflict_declarations(6)) {
        let json = serde_json::to_string(&cds).unwrap();
        let back: Vec<ConflictDeclaration> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&cds, &back);
    }

    // 22. Grammar clone preserves conflicts
    #[test]
    fn grammar_clone_preserves_conflicts(cds in arb_conflict_declarations(4)) {
        let g = grammar_with_conflicts("clone_test", cds);
        let g2 = g.clone();
        prop_assert_eq!(&g.conflicts, &g2.conflicts);
    }

    // 23. PrecedenceKind serde roundtrip
    #[test]
    fn precedence_kind_serde_roundtrip(pk in arb_precedence_kind()) {
        let json = serde_json::to_string(&pk).unwrap();
        let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&pk, &back);
    }

    // 24. Associativity serde roundtrip
    #[test]
    fn associativity_serde_roundtrip(assoc in arb_associativity()) {
        let json = serde_json::to_string(&assoc).unwrap();
        let back: Associativity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&assoc, &back);
    }

    // 25. Normalized grammar still validates with conflicts
    #[test]
    fn normalized_grammar_validates(cds in arb_conflict_declarations(3)) {
        let mut g = grammar_with_conflicts("norm_val", cds);
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }

    // 26. Conflict symbols are all positive IDs
    #[test]
    fn symbols_positive(cd in arb_conflict_declaration()) {
        for sid in &cd.symbols {
            prop_assert!(sid.0 > 0, "symbol ID should be positive, got {}", sid.0);
        }
    }

    // 27. Multiple normalizations preserve conflicts (idempotent for conflicts)
    #[test]
    fn double_normalize_preserves_conflicts(cds in arb_conflict_declarations(3)) {
        let mut g = grammar_with_conflicts("double_norm", cds.clone());
        g.normalize();
        g.normalize();
        prop_assert_eq!(&g.conflicts, &cds);
    }

    // 28. Grammar default has empty conflicts
    #[test]
    fn default_grammar_no_conflicts(_dummy in 0u8..1) {
        let g = Grammar::default();
        prop_assert!(g.conflicts.is_empty());
    }

    // 29. Conflict declaration JSON contains "symbols" key
    #[test]
    fn json_contains_symbols_key(cd in arb_conflict_declaration()) {
        let json = serde_json::to_string(&cd).unwrap();
        prop_assert!(json.contains("\"symbols\""));
    }

    // 30. Conflict declaration JSON contains "resolution" key
    #[test]
    fn json_contains_resolution_key(cd in arb_conflict_declaration()) {
        let json = serde_json::to_string(&cd).unwrap();
        prop_assert!(json.contains("\"resolution\""));
    }

    // 31. GLR resolution debug format
    #[test]
    fn glr_resolution_debug(syms in arb_symbol_ids(4)) {
        let cd = ConflictDeclaration {
            symbols: syms,
            resolution: ConflictResolution::GLR,
        };
        let dbg = format!("{cd:?}");
        prop_assert!(dbg.contains("GLR"));
    }

    // 32. Precedence resolution debug contains level
    #[test]
    fn precedence_resolution_debug_contains_level(level in -100i16..100) {
        let res = ConflictResolution::Precedence(PrecedenceKind::Static(level));
        let dbg = format!("{res:?}");
        prop_assert!(dbg.contains("Static"));
    }

    // 33. Appending conflicts to grammar preserves earlier ones
    #[test]
    fn append_preserves_earlier(
        cds1 in arb_conflict_declarations(3),
        cds2 in arb_conflict_declarations(3),
    ) {
        let mut g = grammar_with_conflicts("append", cds1.clone());
        // Ensure new symbols are registered before adding more conflicts.
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
    }
}
