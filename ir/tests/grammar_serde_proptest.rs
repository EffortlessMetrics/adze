#![allow(clippy::needless_range_loop)]

//! Property-based tests for Grammar serialization/deserialization in adze-ir.
//!
//! Covers JSON roundtrip, bincode roundtrip, field preservation, corrupt data
//! handling, and large grammar stress tests.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, Symbol, SymbolId, Token,
    TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..500).prop_map(SymbolId)
}

fn arb_field_id() -> impl Strategy<Value = FieldId> {
    (0u16..100).prop_map(FieldId)
}

fn arb_production_id() -> impl Strategy<Value = ProductionId> {
    (0u16..100).prop_map(ProductionId)
}

fn arb_rule_id() -> impl Strategy<Value = RuleId> {
    (0u16..100).prop_map(RuleId)
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

fn arb_token_pattern() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z0-9_]{1,10}".prop_map(TokenPattern::String),
        "[a-zA-Z0-9_.+*?]{1,10}".prop_map(TokenPattern::Regex),
    ]
}

fn arb_symbol_leaf() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        arb_symbol_id().prop_map(Symbol::Terminal),
        arb_symbol_id().prop_map(Symbol::NonTerminal),
        arb_symbol_id().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

fn arb_symbol() -> impl Strategy<Value = Symbol> {
    arb_symbol_leaf().prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..4).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..4).prop_map(Symbol::Sequence),
        ]
    })
}

fn arb_token() -> impl Strategy<Value = Token> {
    (
        "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
        arb_token_pattern(),
        any::<bool>(),
    )
        .prop_map(|(name, pattern, fragile)| Token {
            name,
            pattern,
            fragile,
        })
}

fn arb_rule() -> impl Strategy<Value = Rule> {
    (
        arb_symbol_id(),
        prop::collection::vec(arb_symbol(), 0..5),
        prop::option::of(arb_precedence_kind()),
        prop::option::of(arb_associativity()),
        prop::collection::vec((arb_field_id(), 0usize..10), 0..3),
        arb_production_id(),
    )
        .prop_map(
            |(lhs, rhs, precedence, associativity, fields, production_id)| Rule {
                lhs,
                rhs,
                precedence,
                associativity,
                fields,
                production_id,
            },
        )
}

fn arb_external_token() -> impl Strategy<Value = ExternalToken> {
    ("[a-zA-Z_][a-zA-Z0-9_]{0,10}", arb_symbol_id())
        .prop_map(|(name, symbol_id)| ExternalToken { name, symbol_id })
}

fn arb_precedence() -> impl Strategy<Value = Precedence> {
    (
        -100i16..100,
        arb_associativity(),
        prop::collection::vec(arb_symbol_id(), 0..5),
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
        prop::collection::vec(arb_symbol_id(), 1..5),
        arb_conflict_resolution(),
    )
        .prop_map(|(symbols, resolution)| ConflictDeclaration {
            symbols,
            resolution,
        })
}

fn arb_alias_sequence() -> impl Strategy<Value = AliasSequence> {
    prop::collection::vec(
        prop::option::of("[a-zA-Z_]{1,8}".prop_map(String::from)),
        0..6,
    )
    .prop_map(|aliases| AliasSequence { aliases })
}

/// Build a Grammar with all fields populated from random components.
fn arb_full_grammar() -> impl Strategy<Value = Grammar> {
    (
        "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        prop::collection::vec(arb_rule(), 1..4),
        prop::collection::vec((arb_symbol_id(), arb_token()), 1..4),
        prop::collection::vec(arb_precedence(), 0..3),
        prop::collection::vec(arb_conflict_declaration(), 0..3),
        prop::collection::vec(arb_external_token(), 0..3),
        prop::collection::vec(arb_symbol_id(), 0..3),
        prop::collection::vec(arb_symbol_id(), 0..3),
        prop::collection::vec(arb_symbol_id(), 0..3),
        (0usize..10),
    )
        .prop_map(
            |(
                name,
                rules,
                toks,
                precs,
                conflicts,
                exts,
                extras,
                supertypes,
                inline_rules,
                max_alias,
            )| {
                let mut g = Grammar::new(name);
                for rule in rules {
                    g.add_rule(rule);
                }
                for (id, tok) in toks {
                    g.tokens.insert(id, tok);
                }
                g.precedences = precs;
                g.conflicts = conflicts;
                g.externals = exts;
                g.extras = extras;
                g.supertypes = supertypes;
                g.inline_rules = inline_rules;
                g.max_alias_sequence_length = max_alias;
                g
            },
        )
}

// ---------------------------------------------------------------------------
// Roundtrip helpers
// ---------------------------------------------------------------------------

fn json_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) -> T {
    let json = serde_json::to_string(val).expect("json serialize");
    let back: T = serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(val, &back, "JSON roundtrip mismatch");
    back
}

fn bincode_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) -> T {
    let bytes = bincode::serialize(val).expect("bincode serialize");
    let back: T = bincode::deserialize(&bytes).expect("bincode deserialize");
    assert_eq!(val, &back, "bincode roundtrip mismatch");
    back
}

// ===========================================================================
// Tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // -----------------------------------------------------------------------
    // 1. Grammar JSON roundtrip (empty)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_json_roundtrip_empty(name in "[a-zA-Z][a-zA-Z0-9_]{0,15}") {
        let g = Grammar::new(name);
        let back = json_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 2. Grammar bincode roundtrip (empty)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_bincode_roundtrip_empty(name in "[a-zA-Z][a-zA-Z0-9_]{0,15}") {
        let g = Grammar::new(name);
        let back = bincode_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 3. Grammar with rules roundtrip (JSON)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_rules_json_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        rules in prop::collection::vec(arb_rule(), 1..6),
    ) {
        let mut g = Grammar::new(name);
        for r in rules { g.add_rule(r); }
        let back = json_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 4. Grammar with rules roundtrip (bincode)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_rules_bincode_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        rules in prop::collection::vec(arb_rule(), 1..6),
    ) {
        let mut g = Grammar::new(name);
        for r in rules { g.add_rule(r); }
        let back = bincode_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 5. Grammar with externals roundtrip (JSON)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_externals_json_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        exts in prop::collection::vec(arb_external_token(), 1..5),
    ) {
        let mut g = Grammar::new(name);
        g.externals = exts;
        let back = json_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 6. Grammar with externals roundtrip (bincode)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_externals_bincode_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        exts in prop::collection::vec(arb_external_token(), 1..5),
    ) {
        let mut g = Grammar::new(name);
        g.externals = exts;
        let back = bincode_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 7. Grammar with precedences roundtrip (JSON)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_precedences_json_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        precs in prop::collection::vec(arb_precedence(), 1..5),
    ) {
        let mut g = Grammar::new(name);
        g.precedences = precs;
        let back = json_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 8. Grammar with precedences roundtrip (bincode)
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_precedences_bincode_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        precs in prop::collection::vec(arb_precedence(), 1..5),
    ) {
        let mut g = Grammar::new(name);
        g.precedences = precs;
        let back = bincode_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 9. Serialization preserves all fields (JSON)
    // -----------------------------------------------------------------------
    #[test]
    fn serialization_preserves_all_fields_json(g in arb_full_grammar()) {
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.name, &back.name);
        prop_assert_eq!(&g.rules, &back.rules);
        prop_assert_eq!(&g.tokens, &back.tokens);
        prop_assert_eq!(&g.precedences, &back.precedences);
        prop_assert_eq!(&g.conflicts, &back.conflicts);
        prop_assert_eq!(&g.externals, &back.externals);
        prop_assert_eq!(&g.extras, &back.extras);
        prop_assert_eq!(&g.fields, &back.fields);
        prop_assert_eq!(&g.supertypes, &back.supertypes);
        prop_assert_eq!(&g.inline_rules, &back.inline_rules);
        prop_assert_eq!(&g.alias_sequences, &back.alias_sequences);
        prop_assert_eq!(&g.production_ids, &back.production_ids);
        prop_assert_eq!(g.max_alias_sequence_length, back.max_alias_sequence_length);
        prop_assert_eq!(&g.rule_names, &back.rule_names);
        prop_assert_eq!(&g.symbol_registry, &back.symbol_registry);
    }

    // -----------------------------------------------------------------------
    // 10. Serialization preserves all fields (bincode)
    // -----------------------------------------------------------------------
    #[test]
    fn serialization_preserves_all_fields_bincode(g in arb_full_grammar()) {
        let bytes = bincode::serialize(&g).unwrap();
        let back: Grammar = bincode::deserialize(&bytes).unwrap();
        prop_assert_eq!(&g.name, &back.name);
        prop_assert_eq!(&g.rules, &back.rules);
        prop_assert_eq!(&g.tokens, &back.tokens);
        prop_assert_eq!(&g.precedences, &back.precedences);
        prop_assert_eq!(&g.conflicts, &back.conflicts);
        prop_assert_eq!(&g.externals, &back.externals);
        prop_assert_eq!(&g.extras, &back.extras);
        prop_assert_eq!(&g.fields, &back.fields);
        prop_assert_eq!(&g.supertypes, &back.supertypes);
        prop_assert_eq!(&g.inline_rules, &back.inline_rules);
        prop_assert_eq!(g.max_alias_sequence_length, back.max_alias_sequence_length);
        prop_assert_eq!(&g.symbol_registry, &back.symbol_registry);
    }

    // -----------------------------------------------------------------------
    // 11. Deserialization error for corrupt JSON
    // -----------------------------------------------------------------------
    #[test]
    fn deserialization_error_corrupt_json(
        garbage in prop::collection::vec(any::<u8>(), 1..64),
    ) {
        let corrupt = String::from_utf8_lossy(&garbage).to_string();
        let result = serde_json::from_str::<Grammar>(&corrupt);
        // Most random bytes won't parse as valid Grammar JSON
        if result.is_ok() {
            // If it somehow parsed, it must still roundtrip
            let g = result.unwrap();
            let back = json_roundtrip(&g);
            prop_assert_eq!(g, back);
        }
    }

    // -----------------------------------------------------------------------
    // 12. Deserialization error for corrupt bincode
    // -----------------------------------------------------------------------
    #[test]
    fn deserialization_error_corrupt_bincode(
        garbage in prop::collection::vec(any::<u8>(), 1..32),
    ) {
        let result = bincode::deserialize::<Grammar>(&garbage);
        if result.is_ok() {
            let g = result.unwrap();
            let back = bincode_roundtrip(&g);
            prop_assert_eq!(g, back);
        }
    }

    // -----------------------------------------------------------------------
    // 13. Large grammar roundtrip (JSON)
    // -----------------------------------------------------------------------
    #[test]
    fn large_grammar_json_roundtrip(
        rules in prop::collection::vec(arb_rule(), 10..20),
        toks in prop::collection::vec((arb_symbol_id(), arb_token()), 5..10),
        precs in prop::collection::vec(arb_precedence(), 3..6),
        exts in prop::collection::vec(arb_external_token(), 2..5),
    ) {
        let mut g = Grammar::new("large_grammar".into());
        for r in rules { g.add_rule(r); }
        for (id, tok) in toks { g.tokens.insert(id, tok); }
        g.precedences = precs;
        g.externals = exts;
        let back = json_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 14. Large grammar roundtrip (bincode)
    // -----------------------------------------------------------------------
    #[test]
    fn large_grammar_bincode_roundtrip(
        rules in prop::collection::vec(arb_rule(), 10..20),
        toks in prop::collection::vec((arb_symbol_id(), arb_token()), 5..10),
        precs in prop::collection::vec(arb_precedence(), 3..6),
        exts in prop::collection::vec(arb_external_token(), 2..5),
    ) {
        let mut g = Grammar::new("large_grammar".into());
        for r in rules { g.add_rule(r); }
        for (id, tok) in toks { g.tokens.insert(id, tok); }
        g.precedences = precs;
        g.externals = exts;
        let back = bincode_roundtrip(&g);
        prop_assert_eq!(g, back);
    }

    // -----------------------------------------------------------------------
    // 15. Grammar with conflicts roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_conflicts_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        conflicts in prop::collection::vec(arb_conflict_declaration(), 1..5),
    ) {
        let mut g = Grammar::new(name);
        g.conflicts = conflicts;
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(&g, &j);
        prop_assert_eq!(&g, &b);
    }

    // -----------------------------------------------------------------------
    // 16. Grammar with alias sequences roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_aliases_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        seqs in prop::collection::vec(
            (arb_production_id(), arb_alias_sequence()), 1..5
        ),
    ) {
        let mut g = Grammar::new(name);
        for (pid, seq) in seqs {
            g.alias_sequences.insert(pid, seq);
        }
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(&g, &j);
        prop_assert_eq!(&g, &b);
    }

    // -----------------------------------------------------------------------
    // 17. Grammar with fields (sorted) roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_fields_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        field_names in prop::collection::vec("[a-z]{1,8}", 1..6),
    ) {
        let mut g = Grammar::new(name);
        let mut sorted = field_names;
        sorted.sort();
        sorted.dedup();
        for (i, fname) in sorted.into_iter().enumerate() {
            g.fields.insert(FieldId(i as u16), fname);
        }
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(&g, &j);
        prop_assert_eq!(&g, &b);
    }

    // -----------------------------------------------------------------------
    // 18. Grammar with production_ids roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_production_ids_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        ids in prop::collection::vec((arb_rule_id(), arb_production_id()), 1..6),
    ) {
        let mut g = Grammar::new(name);
        for (rid, pid) in ids {
            g.production_ids.insert(rid, pid);
        }
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(&g, &j);
        prop_assert_eq!(&g, &b);
    }

    // -----------------------------------------------------------------------
    // 19. Grammar with rule_names roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_rule_names_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        rule_names in prop::collection::vec(
            (arb_symbol_id(), "[a-zA-Z_][a-zA-Z0-9_]{0,10}"), 1..6
        ),
    ) {
        let mut g = Grammar::new(name);
        for (sid, rname) in rule_names {
            g.rule_names.insert(sid, rname);
        }
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(&g, &j);
        prop_assert_eq!(&g, &b);
    }

    // -----------------------------------------------------------------------
    // 20. Grammar with extras/supertypes/inline_rules roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_symbol_lists_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        extras in prop::collection::vec(arb_symbol_id(), 0..5),
        supertypes in prop::collection::vec(arb_symbol_id(), 0..5),
        inline_rules in prop::collection::vec(arb_symbol_id(), 0..5),
    ) {
        let mut g = Grammar::new(name);
        g.extras = extras;
        g.supertypes = supertypes;
        g.inline_rules = inline_rules;
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(&g, &j);
        prop_assert_eq!(&g, &b);
    }

    // -----------------------------------------------------------------------
    // 21. Pretty vs compact JSON produce same Grammar
    // -----------------------------------------------------------------------
    #[test]
    fn pretty_vs_compact_json_grammar(g in arb_full_grammar()) {
        let compact = serde_json::to_string(&g).unwrap();
        let pretty = serde_json::to_string_pretty(&g).unwrap();
        let from_compact: Grammar = serde_json::from_str(&compact).unwrap();
        let from_pretty: Grammar = serde_json::from_str(&pretty).unwrap();
        prop_assert_eq!(&from_compact, &from_pretty);
        prop_assert_eq!(&g, &from_compact);
    }

    // -----------------------------------------------------------------------
    // 22. Bincode determinism: same grammar serializes to identical bytes
    // -----------------------------------------------------------------------
    #[test]
    fn bincode_deterministic_grammar(g in arb_full_grammar()) {
        let bytes1 = bincode::serialize(&g).unwrap();
        let bytes2 = bincode::serialize(&g).unwrap();
        prop_assert_eq!(bytes1, bytes2);
    }

    // -----------------------------------------------------------------------
    // 23. Cross-format: JSON and bincode produce equal Grammar values
    // -----------------------------------------------------------------------
    #[test]
    fn cross_format_grammar_roundtrip(g in arb_full_grammar()) {
        let from_json: Grammar = serde_json::from_str(
            &serde_json::to_string(&g).unwrap()
        ).unwrap();
        let from_bincode: Grammar = bincode::deserialize(
            &bincode::serialize(&g).unwrap()
        ).unwrap();
        prop_assert_eq!(&from_json, &from_bincode);
    }

    // -----------------------------------------------------------------------
    // 24. max_alias_sequence_length survives roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn max_alias_sequence_length_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        max_len in 0usize..1000,
    ) {
        let mut g = Grammar::new(name);
        g.max_alias_sequence_length = max_len;
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(g.max_alias_sequence_length, j.max_alias_sequence_length);
        prop_assert_eq!(g.max_alias_sequence_length, b.max_alias_sequence_length);
    }

    // -----------------------------------------------------------------------
    // 25. Double roundtrip idempotence (JSON)
    // -----------------------------------------------------------------------
    #[test]
    fn double_roundtrip_json(g in arb_full_grammar()) {
        let json1 = serde_json::to_string(&g).unwrap();
        let back1: Grammar = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back1).unwrap();
        let back2: Grammar = serde_json::from_str(&json2).unwrap();
        prop_assert_eq!(&g, &back2);
        prop_assert_eq!(json1, json2);
    }

    // -----------------------------------------------------------------------
    // 26. Double roundtrip idempotence (bincode)
    // -----------------------------------------------------------------------
    #[test]
    fn double_roundtrip_bincode(g in arb_full_grammar()) {
        let bytes1 = bincode::serialize(&g).unwrap();
        let back1: Grammar = bincode::deserialize(&bytes1).unwrap();
        let bytes2 = bincode::serialize(&back1).unwrap();
        let back2: Grammar = bincode::deserialize(&bytes2).unwrap();
        prop_assert_eq!(&g, &back2);
        prop_assert_eq!(bytes1, bytes2);
    }

    // -----------------------------------------------------------------------
    // 27. Grammar with tokens roundtrip
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_with_tokens_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        toks in prop::collection::vec((arb_symbol_id(), arb_token()), 1..6),
    ) {
        let mut g = Grammar::new(name);
        for (id, tok) in toks { g.tokens.insert(id, tok); }
        let j = json_roundtrip(&g);
        let b = bincode_roundtrip(&g);
        prop_assert_eq!(&g, &j);
        prop_assert_eq!(&g, &b);
    }

    // -----------------------------------------------------------------------
    // 28. JSON value representation is stable across roundtrips
    // -----------------------------------------------------------------------
    #[test]
    fn json_value_stable(g in arb_full_grammar()) {
        let v1 = serde_json::to_value(&g).unwrap();
        let back: Grammar = serde_json::from_value(v1.clone()).unwrap();
        let v2 = serde_json::to_value(&back).unwrap();
        prop_assert_eq!(v1, v2);
    }

    // -----------------------------------------------------------------------
    // 29. Truncated bincode always fails
    // -----------------------------------------------------------------------
    #[test]
    fn truncated_bincode_fails(g in arb_full_grammar()) {
        let bytes = bincode::serialize(&g).unwrap();
        if bytes.len() > 1 {
            let truncated = &bytes[..bytes.len() / 2];
            let result = bincode::deserialize::<Grammar>(truncated);
            prop_assert!(result.is_err());
        }
    }

    // -----------------------------------------------------------------------
    // 30. Truncated JSON always fails
    // -----------------------------------------------------------------------
    #[test]
    fn truncated_json_fails(g in arb_full_grammar()) {
        let json = serde_json::to_string(&g).unwrap();
        if json.len() > 2 {
            let truncated = &json[..json.len() / 2];
            let result = serde_json::from_str::<Grammar>(truncated);
            prop_assert!(result.is_err());
        }
    }
}

// ===========================================================================
// Non-proptest: builder-based grammar roundtrips
// ===========================================================================

/// Build an arithmetic grammar via the builder API and roundtrip it.
#[test]
fn builder_grammar_json_roundtrip() {
    let g = GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let back = json_roundtrip(&g);
    assert_eq!(g, back);
}

#[test]
fn builder_grammar_bincode_roundtrip() {
    let g = GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let back = bincode_roundtrip(&g);
    assert_eq!(g, back);
}

/// Grammar::default() roundtrips cleanly.
#[test]
fn default_grammar_roundtrip() {
    let g = Grammar::default();
    let j = json_roundtrip(&g);
    let b = bincode_roundtrip(&g);
    assert_eq!(g, j);
    assert_eq!(g, b);
}

/// Complex builder grammar with precedences and extras.
#[test]
fn complex_builder_grammar_roundtrip() {
    let g = GrammarBuilder::new("complex_arith")
        .token("NUMBER", r"\d+(\.\d+)?")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .extra("WS")
        .token("WS", r"[ \t\n]+")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .precedence(2, Associativity::Left, vec!["*", "/"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["IDENTIFIER"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["IDENTIFIER", "(", "args", ")"])
        .rule("args", vec!["expr"])
        .rule("args", vec!["args", ",", "expr"])
        .start("expr")
        .build();

    let json = serde_json::to_string_pretty(&g).unwrap();
    let from_json: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, from_json);

    let bytes = bincode::serialize(&g).unwrap();
    let from_bincode: Grammar = bincode::deserialize(&bytes).unwrap();
    assert_eq!(g, from_bincode);
    assert_eq!(from_json, from_bincode);
}

/// Corrupt JSON with valid-looking but wrong types still fails or roundtrips.
#[test]
fn wrong_type_json_fails() {
    let bad_json = r#"{"name": 42}"#;
    let result = serde_json::from_str::<Grammar>(bad_json);
    assert!(result.is_err());
}
