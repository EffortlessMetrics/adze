//! Property-based roundtrip tests for adze-ir types.
//!
//! Uses proptest to generate random grammars and verify roundtrip properties:
//! serialization, normalization idempotency, validation, and structural equality.

use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, GrammarValidator, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, Symbol,
    SymbolId, Token, TokenPattern, ValidationError,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies for generating arbitrary IR types
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..200).prop_map(SymbolId)
}

fn arb_field_id() -> impl Strategy<Value = FieldId> {
    (0u16..50).prop_map(FieldId)
}

fn arb_production_id() -> impl Strategy<Value = ProductionId> {
    (0u16..100).prop_map(ProductionId)
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
        (-500i16..500).prop_map(PrecedenceKind::Static),
        (-500i16..500).prop_map(PrecedenceKind::Dynamic),
    ]
}

fn arb_token_pattern() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z_+\\-*/=<>!]{1,10}".prop_map(TokenPattern::String),
        "[a-z]{1,5}".prop_map(|s| TokenPattern::Regex(format!("[{s}]+"))),
    ]
}

fn arb_token() -> impl Strategy<Value = Token> {
    (
        "[A-Z_][A-Z0-9_]{1,8}",
        arb_token_pattern(),
        proptest::bool::ANY,
    )
        .prop_map(|(name, pattern, fragile)| Token {
            name,
            pattern,
            fragile,
        })
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

fn arb_external_token() -> impl Strategy<Value = ExternalToken> {
    ("[A-Z_]{2,8}", arb_symbol_id()).prop_map(|(name, symbol_id)| ExternalToken { name, symbol_id })
}

fn arb_precedence() -> impl Strategy<Value = Precedence> {
    (
        -100i16..100,
        arb_associativity(),
        prop::collection::vec(arb_symbol_id(), 1..4),
    )
        .prop_map(|(level, associativity, symbols)| Precedence {
            level,
            associativity,
            symbols,
        })
}

fn arb_conflict_resolution() -> impl Strategy<Value = ConflictResolution> {
    prop_oneof![
        arb_precedence_kind().prop_map(ConflictResolution::Precedence),
        arb_associativity().prop_map(ConflictResolution::Associativity),
        Just(ConflictResolution::GLR),
    ]
}

fn arb_conflict_declaration() -> impl Strategy<Value = ConflictDeclaration> {
    (
        prop::collection::vec(arb_symbol_id(), 2..5),
        arb_conflict_resolution(),
    )
        .prop_map(|(symbols, resolution)| ConflictDeclaration {
            symbols,
            resolution,
        })
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

fn arb_rule_with_fields(lhs: SymbolId) -> impl Strategy<Value = Rule> {
    (
        prop::collection::vec(arb_symbol(), 1..5),
        prop::option::of(arb_precedence_kind()),
        prop::option::of(arb_associativity()),
        arb_production_id(),
        prop::collection::vec((arb_field_id(), 0usize..10), 0..3),
    )
        .prop_map(
            move |(rhs, precedence, associativity, production_id, fields)| Rule {
                lhs,
                rhs,
                precedence,
                associativity,
                fields,
                production_id,
            },
        )
}

/// Generate a grammar with `n` symbols, each with up to `max_rules_per` rules
/// and tokens registered so symbols are "defined".
fn arb_grammar(max_symbols: usize, max_rules_per: usize) -> impl Strategy<Value = Grammar> {
    (1..=max_symbols).prop_flat_map(move |n| {
        let rule_strats: Vec<_> = (0..n)
            .map(|i| {
                let lhs = SymbolId(i as u16);
                prop::collection::vec(arb_rule(lhs), 1..=max_rules_per)
            })
            .collect();
        ("[a-z]{3,10}", rule_strats).prop_map(move |(name, all_rules)| {
            let mut g = Grammar::new(name);
            for (i, rules) in all_rules.into_iter().enumerate() {
                let sid = SymbolId(i as u16);
                g.tokens.insert(
                    sid,
                    Token {
                        name: format!("TOK_{i}"),
                        pattern: TokenPattern::String(format!("t{i}")),
                        fragile: false,
                    },
                );
                g.rule_names.insert(sid, format!("rule_{i}"));
                for r in rules {
                    g.add_rule(r);
                }
            }
            g
        })
    })
}

/// Generate a richer grammar including externals, extras, precedences, conflicts.
fn arb_rich_grammar() -> impl Strategy<Value = Grammar> {
    (
        arb_grammar(5, 3),
        prop::collection::vec(arb_external_token(), 0..3),
        prop::collection::vec(arb_precedence(), 0..3),
        prop::collection::vec(arb_conflict_declaration(), 0..2),
    )
        .prop_map(|(mut g, externals, precs, conflicts)| {
            // Assign external tokens to high IDs to avoid collision
            for (i, mut ext) in externals.into_iter().enumerate() {
                ext.symbol_id = SymbolId(500 + i as u16);
                g.externals.push(ext);
            }
            // Use existing symbol IDs for extras
            let sym_count = g.tokens.len();
            if sym_count > 0 {
                g.extras.push(SymbolId(0));
            }
            g.precedences = precs;
            g.conflicts = conflicts;
            g
        })
}

// ---------------------------------------------------------------------------
// Helper: compare two grammars via JSON Value equality
// ---------------------------------------------------------------------------

fn grammar_json(g: &Grammar) -> serde_json::Value {
    serde_json::to_value(g).expect("serialize to Value")
}

fn json_roundtrip(g: &Grammar) -> Grammar {
    let json = serde_json::to_string(g).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Grammar → JSON → Grammar preserves rule count
    #[test]
    fn json_roundtrip_preserves_rule_count(g in arb_grammar(6, 3)) {
        let original_count = g.all_rules().count();
        let rt = json_roundtrip(&g);
        prop_assert_eq!(original_count, rt.all_rules().count());
    }

    // 2. Grammar → JSON → Grammar preserves symbol names
    #[test]
    fn json_roundtrip_preserves_symbol_names(g in arb_grammar(6, 3)) {
        let rt = json_roundtrip(&g);
        let orig_names: Vec<_> = g.rule_names.values().cloned().collect();
        let rt_names: Vec<_> = rt.rule_names.values().cloned().collect();
        prop_assert_eq!(orig_names, rt_names);
    }

    // 3. Grammar → JSON → Grammar preserves precedence values
    #[test]
    fn json_roundtrip_preserves_precedence_values(g in arb_rich_grammar()) {
        let rt = json_roundtrip(&g);
        prop_assert_eq!(g.precedences.len(), rt.precedences.len());
        for (a, b) in g.precedences.iter().zip(rt.precedences.iter()) {
            prop_assert_eq!(a.level, b.level);
        }
    }

    // 4. Grammar → JSON → Grammar preserves associativity
    #[test]
    fn json_roundtrip_preserves_associativity(g in arb_rich_grammar()) {
        let rt = json_roundtrip(&g);
        // Check rule-level associativity
        for (a, b) in g.all_rules().zip(rt.all_rules()) {
            prop_assert_eq!(a.associativity, b.associativity);
        }
        // Check precedence-level associativity
        for (a, b) in g.precedences.iter().zip(rt.precedences.iter()) {
            prop_assert_eq!(a.associativity, b.associativity);
        }
    }

    // 5. Grammar → JSON → Grammar preserves external token count
    #[test]
    fn json_roundtrip_preserves_external_token_count(g in arb_rich_grammar()) {
        let rt = json_roundtrip(&g);
        prop_assert_eq!(g.externals.len(), rt.externals.len());
        for (a, b) in g.externals.iter().zip(rt.externals.iter()) {
            prop_assert_eq!(&a.name, &b.name);
            prop_assert_eq!(a.symbol_id, b.symbol_id);
        }
    }

    // 6. Grammar → JSON → Grammar preserves extras
    #[test]
    fn json_roundtrip_preserves_extras(g in arb_rich_grammar()) {
        let rt = json_roundtrip(&g);
        prop_assert_eq!(&g.extras, &rt.extras);
    }

    // 7. Grammar → normalize → normalize is idempotent (no new rules added)
    #[test]
    fn normalize_is_idempotent(g in arb_grammar(4, 2)) {
        let mut g1 = g.clone();
        g1.normalize();
        let count_after_first = g1.all_rules().count();
        let json_after_first = serde_json::to_string(&g1).expect("serialize");

        g1.normalize();
        let count_after_second = g1.all_rules().count();
        let json_after_second = serde_json::to_string(&g1).expect("serialize");

        prop_assert_eq!(count_after_first, count_after_second,
            "second normalize changed rule count");
        prop_assert_eq!(json_after_first, json_after_second,
            "second normalize changed grammar JSON");
    }

    // 8. Grammar → validate → valid grammars pass, invalid (empty) fail
    #[test]
    fn empty_grammar_fails_validation(name in "[a-z]{3,10}") {
        let g = Grammar::new(name);
        let mut v = GrammarValidator::new();
        let result = v.validate(&g);
        prop_assert!(
            result.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)),
            "expected EmptyGrammar error"
        );
    }

    #[test]
    fn nonempty_grammar_has_no_empty_error(g in arb_grammar(3, 2)) {
        let mut v = GrammarValidator::new();
        let result = v.validate(&g);
        prop_assert!(
            !result.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)),
            "non-empty grammar should not get EmptyGrammar error"
        );
    }

    // 9. SymbolId → u16 → SymbolId roundtrip
    #[test]
    fn symbol_id_u16_roundtrip(val in 0u16..=u16::MAX) {
        let sid = SymbolId(val);
        let raw: u16 = sid.0;
        let back = SymbolId(raw);
        prop_assert_eq!(sid, back);
    }

    // 10. Token → JSON → Token preserves pattern and priority
    #[test]
    fn token_json_roundtrip(tok in arb_token()) {
        let json = serde_json::to_string(&tok).expect("serialize token");
        let rt: Token = serde_json::from_str(&json).expect("deserialize token");
        prop_assert_eq!(tok.name, rt.name);
        prop_assert_eq!(tok.pattern, rt.pattern);
        prop_assert_eq!(tok.fragile, rt.fragile);
    }

    // 11. Rule → JSON → Rule preserves symbols and production info
    #[test]
    fn rule_json_roundtrip(rule in arb_rule_with_fields(SymbolId(0))) {
        let json = serde_json::to_string(&rule).expect("serialize rule");
        let rt: Rule = serde_json::from_str(&json).expect("deserialize rule");
        prop_assert_eq!(rule, rt);
    }

    // 12. Grammar equality: structurally identical grammars are equal (via JSON)
    #[test]
    fn structural_equality_via_json(g in arb_grammar(5, 3)) {
        let g2 = g.clone();
        prop_assert_eq!(grammar_json(&g), grammar_json(&g2));
    }

    // -- Additional roundtrip properties (13-24) --

    // 13. Full JSON Value roundtrip (double serialization)
    #[test]
    fn full_json_value_roundtrip(g in arb_rich_grammar()) {
        let v1 = grammar_json(&g);
        let rt = json_roundtrip(&g);
        let v2 = grammar_json(&rt);
        prop_assert_eq!(v1, v2);
    }

    // 14. PrecedenceKind → JSON → PrecedenceKind roundtrip
    #[test]
    fn precedence_kind_json_roundtrip(pk in arb_precedence_kind()) {
        let json = serde_json::to_string(&pk).expect("serialize");
        let rt: PrecedenceKind = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(pk, rt);
    }

    // 15. Associativity → JSON → Associativity roundtrip
    #[test]
    fn associativity_json_roundtrip(a in arb_associativity()) {
        let json = serde_json::to_string(&a).expect("serialize");
        let rt: Associativity = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(a, rt);
    }

    // 16. ConflictResolution → JSON → ConflictResolution roundtrip
    #[test]
    fn conflict_resolution_json_roundtrip(cr in arb_conflict_resolution()) {
        let json = serde_json::to_string(&cr).expect("serialize");
        let rt: ConflictResolution = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(cr, rt);
    }

    // 17. Symbol → JSON → Symbol roundtrip
    #[test]
    fn symbol_json_roundtrip(sym in arb_symbol()) {
        let json = serde_json::to_string(&sym).expect("serialize");
        let rt: Symbol = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(sym, rt);
    }

    // 18. Grammar name preserved through roundtrip
    #[test]
    fn json_roundtrip_preserves_name(g in arb_grammar(3, 2)) {
        let rt = json_roundtrip(&g);
        prop_assert_eq!(&g.name, &rt.name);
    }

    // 19. Grammar → JSON → Grammar preserves token map
    #[test]
    fn json_roundtrip_preserves_tokens(g in arb_grammar(5, 2)) {
        let rt = json_roundtrip(&g);
        prop_assert_eq!(g.tokens.len(), rt.tokens.len());
        for (id, tok) in &g.tokens {
            let rt_tok = rt.tokens.get(id).expect("token should exist");
            prop_assert_eq!(&tok.name, &rt_tok.name);
            prop_assert_eq!(&tok.pattern, &rt_tok.pattern);
            prop_assert_eq!(tok.fragile, rt_tok.fragile);
        }
    }

    // 20. Grammar → JSON → Grammar preserves conflicts
    #[test]
    fn json_roundtrip_preserves_conflicts(g in arb_rich_grammar()) {
        let rt = json_roundtrip(&g);
        prop_assert_eq!(g.conflicts.len(), rt.conflicts.len());
        for (a, b) in g.conflicts.iter().zip(rt.conflicts.iter()) {
            prop_assert_eq!(&a.symbols, &b.symbols);
            prop_assert_eq!(&a.resolution, &b.resolution);
        }
    }

    // 21. Normalize then roundtrip preserves normalized form
    #[test]
    fn normalize_then_roundtrip_stable(g in arb_grammar(4, 2)) {
        let mut norm = g.clone();
        norm.normalize();
        let rt = json_roundtrip(&norm);
        let norm_json = serde_json::to_string(&norm).expect("serialize");
        let rt_json = serde_json::to_string(&rt).expect("serialize");
        prop_assert_eq!(norm_json, rt_json);
    }

    // 22. RuleId → u16 → RuleId roundtrip
    #[test]
    fn rule_id_u16_roundtrip(val in 0u16..=u16::MAX) {
        let rid = RuleId(val);
        let raw: u16 = rid.0;
        prop_assert_eq!(rid, RuleId(raw));
    }

    // 23. ProductionId → u16 → ProductionId roundtrip
    #[test]
    fn production_id_u16_roundtrip(val in 0u16..=u16::MAX) {
        let pid = ProductionId(val);
        let raw: u16 = pid.0;
        prop_assert_eq!(pid, ProductionId(raw));
    }

    // 24. FieldId → u16 → FieldId roundtrip
    #[test]
    fn field_id_u16_roundtrip(val in 0u16..=u16::MAX) {
        let fid = FieldId(val);
        let raw: u16 = fid.0;
        prop_assert_eq!(fid, FieldId(raw));
    }

    // 25. Alias sequence roundtrip
    #[test]
    fn alias_sequence_json_roundtrip(
        aliases in prop::collection::vec(prop::option::of("[a-z]{2,6}"), 0..6)
    ) {
        let seq = AliasSequence { aliases };
        let json = serde_json::to_string(&seq).expect("serialize");
        let rt: AliasSequence = serde_json::from_str(&json).expect("deserialize");
        // AliasSequence doesn't derive PartialEq, compare via re-serialization
        let json2 = serde_json::to_string(&rt).expect("re-serialize");
        prop_assert_eq!(json, json2);
    }

    // 26. ExternalToken → JSON → ExternalToken roundtrip
    #[test]
    fn external_token_json_roundtrip(ext in arb_external_token()) {
        let json = serde_json::to_string(&ext).expect("serialize");
        let rt: ExternalToken = serde_json::from_str(&json).expect("deserialize");
        // Compare fields directly
        prop_assert_eq!(&ext.name, &rt.name);
        prop_assert_eq!(ext.symbol_id, rt.symbol_id);
    }

    // 27. Precedence declaration → JSON roundtrip
    #[test]
    fn precedence_json_roundtrip(prec in arb_precedence()) {
        let json = serde_json::to_string(&prec).expect("serialize");
        let rt: Precedence = serde_json::from_str(&json).expect("deserialize");
        let json2 = serde_json::to_string(&rt).expect("re-serialize");
        prop_assert_eq!(json, json2);
    }

    // 28. Normalization removes all complex symbols from RHS
    #[test]
    fn normalize_removes_complex_symbols(g in arb_grammar(4, 2)) {
        let mut norm = g;
        norm.normalize();
        for rule in norm.all_rules() {
            for sym in &rule.rhs {
                match sym {
                    Symbol::Optional(_) | Symbol::Repeat(_) |
                    Symbol::RepeatOne(_) | Symbol::Choice(_) => {
                        prop_assert!(false,
                            "complex symbol {:?} found after normalization", sym);
                    }
                    _ => {}
                }
            }
        }
    }

    // 29. Normalization never decreases rule count
    #[test]
    fn normalize_never_decreases_rule_count(g in arb_grammar(4, 2)) {
        let before = g.all_rules().count();
        let mut norm = g;
        norm.normalize();
        let after = norm.all_rules().count();
        prop_assert!(after >= before,
            "normalize decreased rule count: {} -> {}", before, after);
    }

    // 30. Grammar clone produces identical JSON
    #[test]
    fn clone_produces_identical_json(g in arb_rich_grammar()) {
        let cloned = g.clone();
        let j1 = serde_json::to_string(&g).expect("serialize original");
        let j2 = serde_json::to_string(&cloned).expect("serialize clone");
        prop_assert_eq!(j1, j2);
    }
}
