//! Serde edge-case tests for adze-ir types.
//!
//! Covers JSON roundtrip fidelity, malformed input rejection, field ordering,
//! large grammars, newtype ID serialization, token patterns, precedence /
//! associativity, Symbol enum variants, and miscellaneous edge cases (empty
//! fields, special characters, unicode).

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar through the builder API.
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("mini")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build()
}

/// Build an arithmetic grammar with precedence.
fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// JSON roundtrip helper – serialize then deserialize, return the result.
fn json_roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> T {
    let json = serde_json::to_string(val).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

// =========================================================================
// 1. Grammar JSON roundtrip preserves all fields (10 tests)
// =========================================================================

#[test]
fn roundtrip_minimal_grammar() {
    let g = minimal_grammar();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn roundtrip_arith_grammar() {
    let g = arith_grammar();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn roundtrip_grammar_name_preserved() {
    let g = GrammarBuilder::new("fancy_name").build();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g2.name, "fancy_name");
}

#[test]
fn roundtrip_grammar_extras_preserved() {
    let g = GrammarBuilder::new("ws")
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g.extras, g2.extras);
}

#[test]
fn roundtrip_grammar_externals_preserved() {
    let g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .build();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g.externals, g2.externals);
}

#[test]
fn roundtrip_grammar_fields_preserved() {
    let mut g = Grammar::new("fields_test".into());
    g.fields.insert(FieldId(0), "alpha".into());
    g.fields.insert(FieldId(1), "beta".into());
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g.fields, g2.fields);
}

#[test]
fn roundtrip_grammar_supertypes_preserved() {
    let mut g = Grammar::new("st".into());
    g.supertypes = vec![SymbolId(1), SymbolId(2)];
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g.supertypes, g2.supertypes);
}

#[test]
fn roundtrip_grammar_inline_rules_preserved() {
    let mut g = Grammar::new("inl".into());
    g.inline_rules = vec![SymbolId(5), SymbolId(10)];
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g.inline_rules, g2.inline_rules);
}

#[test]
fn roundtrip_alias_sequences_preserved() {
    let mut g = Grammar::new("alias".into());
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("foo".into()), None, Some("bar".into())],
        },
    );
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g.alias_sequences, g2.alias_sequences);
}

#[test]
fn roundtrip_max_alias_sequence_length_preserved() {
    let mut g = Grammar::new("mal".into());
    g.max_alias_sequence_length = 42;
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g2.max_alias_sequence_length, 42);
}

// =========================================================================
// 2. Deserialization of malformed JSON (8 tests)
// =========================================================================

#[test]
fn deser_empty_string_fails() {
    let res = serde_json::from_str::<Grammar>("");
    assert!(res.is_err());
}

#[test]
fn deser_null_fails() {
    let res = serde_json::from_str::<Grammar>("null");
    assert!(res.is_err());
}

#[test]
fn deser_missing_name_field_fails() {
    // Grammar requires `name`; an object without it should fail.
    let json = r#"{"rules":{},"tokens":{},"precedences":[],"conflicts":[],"externals":[],"extras":[],"fields":{},"supertypes":[],"inline_rules":[],"alias_sequences":{},"production_ids":{},"max_alias_sequence_length":0,"rule_names":{},"symbol_registry":null}"#;
    let res = serde_json::from_str::<Grammar>(json);
    assert!(res.is_err());
}

#[test]
fn deser_wrong_type_for_name_fails() {
    let json = r#"{"name":123,"rules":{},"tokens":{},"precedences":[],"conflicts":[],"externals":[],"extras":[],"fields":{},"supertypes":[],"inline_rules":[],"alias_sequences":{},"production_ids":{},"max_alias_sequence_length":0,"rule_names":{},"symbol_registry":null}"#;
    let res = serde_json::from_str::<Grammar>(json);
    assert!(res.is_err());
}

#[test]
fn deser_symbol_id_negative_fails() {
    // SymbolId wraps u16 — a negative number must fail.
    let res = serde_json::from_str::<SymbolId>("-1");
    assert!(res.is_err());
}

#[test]
fn deser_symbol_id_overflow_fails() {
    // u16::MAX + 1 = 65536
    let res = serde_json::from_str::<SymbolId>("65536");
    assert!(res.is_err());
}

#[test]
fn deser_rule_id_float_fails() {
    let res = serde_json::from_str::<RuleId>("1.5");
    assert!(res.is_err());
}

#[test]
fn deser_bad_associativity_variant() {
    let res = serde_json::from_str::<Associativity>(r#""Up""#);
    assert!(res.is_err());
}

// =========================================================================
// 3. Field ordering in serialized output (5 tests)
// =========================================================================

#[test]
fn serialized_grammar_contains_name_key() {
    let g = minimal_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains(r#""name""#));
}

#[test]
fn serialized_grammar_contains_rules_key() {
    let g = minimal_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains(r#""rules""#));
}

#[test]
fn serialized_grammar_contains_tokens_key() {
    let g = minimal_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains(r#""tokens""#));
}

#[test]
fn serialized_indexmap_preserves_insertion_order() {
    let mut g = Grammar::new("order".into());
    g.rule_names.insert(SymbolId(3), "charlie".into());
    g.rule_names.insert(SymbolId(1), "alpha".into());
    g.rule_names.insert(SymbolId(2), "bravo".into());

    let json = serde_json::to_string(&g.rule_names).unwrap();
    let pos_c = json.find("charlie").unwrap();
    let pos_a = json.find("alpha").unwrap();
    let pos_b = json.find("bravo").unwrap();
    // Insertion order: charlie, alpha, bravo
    assert!(pos_c < pos_a);
    assert!(pos_a < pos_b);
}

#[test]
fn serialized_fields_maintain_order_after_roundtrip() {
    let mut g = Grammar::new("fo".into());
    g.fields.insert(FieldId(0), "aaa".into());
    g.fields.insert(FieldId(1), "bbb".into());
    g.fields.insert(FieldId(2), "ccc".into());

    let g2: Grammar = json_roundtrip(&g);
    let keys: Vec<_> = g2.fields.keys().collect();
    assert_eq!(keys, vec![&FieldId(0), &FieldId(1), &FieldId(2)]);
}

// =========================================================================
// 4. Large grammar serialization (5 tests)
// =========================================================================

#[test]
fn large_grammar_many_tokens() {
    let mut b = GrammarBuilder::new("large_tok");
    for i in 0..200 {
        let name = format!("TOK_{i}");
        let pat = format!("t{i}");
        b = b.token(
            Box::leak(name.into_boxed_str()),
            Box::leak(pat.into_boxed_str()),
        );
    }
    let g = b.build();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g, g2);
    assert_eq!(g2.tokens.len(), 200);
}

#[test]
fn large_grammar_many_rules() {
    let mut b = GrammarBuilder::new("large_rule").token("X", "x");
    for i in 0..100 {
        let name = format!("rule_{i}");
        b = b.rule(Box::leak(name.into_boxed_str()), vec!["X"]);
    }
    let g = b.build();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn large_grammar_many_precedences() {
    let mut g = Grammar::new("prec_large".into());
    for i in 0..50i16 {
        g.precedences.push(Precedence {
            level: i,
            associativity: Associativity::Left,
            symbols: vec![SymbolId(i as u16)],
        });
    }
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g.precedences.len(), g2.precedences.len());
}

#[test]
fn large_grammar_serialization_size_reasonable() {
    // A grammar with 100 tokens should serialize to < 100 KB of JSON.
    let mut b = GrammarBuilder::new("size_check");
    for i in 0..100 {
        let name = format!("T{i}");
        let pat = format!("p{i}");
        b = b.token(
            Box::leak(name.into_boxed_str()),
            Box::leak(pat.into_boxed_str()),
        );
    }
    let g = b.build();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.len() < 100_000, "JSON is {} bytes", json.len());
}

#[test]
fn large_grammar_pretty_json_roundtrip() {
    let g = arith_grammar();
    let pretty = serde_json::to_string_pretty(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&pretty).unwrap();
    assert_eq!(g, g2);
}

// =========================================================================
// 5. SymbolId / RuleId / StateId serialization (8 tests)
// =========================================================================

#[test]
fn symbol_id_roundtrip() {
    let id = SymbolId(999);
    let id2: SymbolId = json_roundtrip(&id);
    assert_eq!(id, id2);
}

#[test]
fn rule_id_roundtrip() {
    let id = RuleId(0);
    let id2: RuleId = json_roundtrip(&id);
    assert_eq!(id, id2);
}

#[test]
fn state_id_roundtrip() {
    let id = StateId(u16::MAX);
    let id2: StateId = json_roundtrip(&id);
    assert_eq!(id, id2);
}

#[test]
fn field_id_roundtrip() {
    let id = FieldId(42);
    let id2: FieldId = json_roundtrip(&id);
    assert_eq!(id, id2);
}

#[test]
fn production_id_roundtrip() {
    let id = ProductionId(7);
    let id2: ProductionId = json_roundtrip(&id);
    assert_eq!(id, id2);
}

#[test]
fn symbol_id_zero() {
    let id = SymbolId(0);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "0");
    let id2: SymbolId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, id2);
}

#[test]
fn symbol_id_max() {
    let id = SymbolId(u16::MAX);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "65535");
    let id2: SymbolId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, id2);
}

#[test]
fn id_vec_roundtrip() {
    let ids = vec![SymbolId(1), SymbolId(2), SymbolId(3)];
    let ids2: Vec<SymbolId> = json_roundtrip(&ids);
    assert_eq!(ids, ids2);
}

// =========================================================================
// 6. Token pattern serialization (5 tests)
// =========================================================================

#[test]
fn token_pattern_string_roundtrip() {
    let tp = TokenPattern::String("hello".into());
    let tp2: TokenPattern = json_roundtrip(&tp);
    assert_eq!(tp, tp2);
}

#[test]
fn token_pattern_regex_roundtrip() {
    let tp = TokenPattern::Regex(r"\d+".into());
    let tp2: TokenPattern = json_roundtrip(&tp);
    assert_eq!(tp, tp2);
}

#[test]
fn token_roundtrip() {
    let tok = Token {
        name: "NUMBER".into(),
        pattern: TokenPattern::Regex(r"[0-9]+".into()),
        fragile: false,
    };
    let tok2: Token = json_roundtrip(&tok);
    assert_eq!(tok, tok2);
}

#[test]
fn token_fragile_flag_preserved() {
    let tok = Token {
        name: "FRAG".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: true,
    };
    let tok2: Token = json_roundtrip(&tok);
    assert!(tok2.fragile);
}

#[test]
fn token_pattern_regex_with_special_chars() {
    let tp = TokenPattern::Regex(r#"[a-zA-Z_][a-zA-Z0-9_]*"#.into());
    let json = serde_json::to_string(&tp).unwrap();
    let tp2: TokenPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(tp, tp2);
}

// =========================================================================
// 7. Precedence and associativity serialization (5 tests)
// =========================================================================

#[test]
fn precedence_kind_static_roundtrip() {
    let pk = PrecedenceKind::Static(5);
    let pk2: PrecedenceKind = json_roundtrip(&pk);
    assert_eq!(pk, pk2);
}

#[test]
fn precedence_kind_dynamic_roundtrip() {
    let pk = PrecedenceKind::Dynamic(-3);
    let pk2: PrecedenceKind = json_roundtrip(&pk);
    assert_eq!(pk, pk2);
}

#[test]
fn associativity_all_variants_roundtrip() {
    for assoc in [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ] {
        let a2: Associativity = json_roundtrip(&assoc);
        assert_eq!(assoc, a2);
    }
}

#[test]
fn precedence_struct_roundtrip() {
    let p = Precedence {
        level: -10,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let p2: Precedence = json_roundtrip(&p);
    assert_eq!(p, p2);
}

#[test]
fn precedence_kind_negative_static() {
    let pk = PrecedenceKind::Static(i16::MIN);
    let pk2: PrecedenceKind = json_roundtrip(&pk);
    assert_eq!(pk, pk2);
}

// =========================================================================
// 8. Symbol enum variant serialization (5 tests)
// =========================================================================

#[test]
fn symbol_terminal_roundtrip() {
    let s = Symbol::Terminal(SymbolId(1));
    let s2: Symbol = json_roundtrip(&s);
    assert_eq!(s, s2);
}

#[test]
fn symbol_nonterminal_roundtrip() {
    let s = Symbol::NonTerminal(SymbolId(99));
    let s2: Symbol = json_roundtrip(&s);
    assert_eq!(s, s2);
}

#[test]
fn symbol_epsilon_roundtrip() {
    let s = Symbol::Epsilon;
    let s2: Symbol = json_roundtrip(&s);
    assert_eq!(s, s2);
}

#[test]
fn symbol_optional_roundtrip() {
    let s = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(5))));
    let s2: Symbol = json_roundtrip(&s);
    assert_eq!(s, s2);
}

#[test]
fn symbol_nested_choice_sequence_roundtrip() {
    let s = Symbol::Choice(vec![
        Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
        ]),
        Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(3)))),
        Symbol::RepeatOne(Box::new(Symbol::External(SymbolId(4)))),
    ]);
    let s2: Symbol = json_roundtrip(&s);
    assert_eq!(s, s2);
}

// =========================================================================
// 9. Edge cases: empty fields, special chars, unicode (4 tests)
// =========================================================================

#[test]
fn grammar_empty_name_roundtrip() {
    let g = Grammar::new(String::new());
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g2.name, "");
}

#[test]
fn token_name_with_special_chars() {
    let tok = Token {
        name: r#"<=>!@#$%^&*()"#.into(),
        pattern: TokenPattern::String(r#"<=>!@#$%^&*()"#.into()),
        fragile: false,
    };
    let tok2: Token = json_roundtrip(&tok);
    assert_eq!(tok, tok2);
}

#[test]
fn grammar_name_unicode() {
    let g = Grammar::new("日本語の文法".into());
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g2.name, "日本語の文法");
}

#[test]
fn token_pattern_with_unicode_regex() {
    let tp = TokenPattern::Regex(r"[\p{L}\p{N}]+".into());
    let tp2: TokenPattern = json_roundtrip(&tp);
    assert_eq!(tp, tp2);
}

// =========================================================================
// Bonus tests to reach 55+ (conflict resolution, external tokens, rules)
// =========================================================================

#[test]
fn conflict_declaration_roundtrip() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    };
    let cd2: ConflictDeclaration = json_roundtrip(&cd);
    assert_eq!(cd, cd2);
}

#[test]
fn conflict_resolution_precedence_roundtrip() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Static(3));
    let cr2: ConflictResolution = json_roundtrip(&cr);
    assert_eq!(cr, cr2);
}

#[test]
fn conflict_resolution_associativity_roundtrip() {
    let cr = ConflictResolution::Associativity(Associativity::None);
    let cr2: ConflictResolution = json_roundtrip(&cr);
    assert_eq!(cr, cr2);
}

#[test]
fn external_token_roundtrip() {
    let et = ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(10),
    };
    let et2: ExternalToken = json_roundtrip(&et);
    assert_eq!(et, et2);
}

#[test]
fn rule_roundtrip() {
    let r = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(3)),
        ],
        precedence: Some(PrecedenceKind::Dynamic(1)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(5),
    };
    let r2: Rule = json_roundtrip(&r);
    assert_eq!(r, r2);
}

#[test]
fn rule_no_precedence_roundtrip() {
    let r = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let r2: Rule = json_roundtrip(&r);
    assert_eq!(r, r2);
}

#[test]
fn alias_sequence_roundtrip() {
    let a = AliasSequence {
        aliases: vec![None, Some("x".into()), None],
    };
    let a2: AliasSequence = json_roundtrip(&a);
    assert_eq!(a, a2);
}

#[test]
fn symbol_metadata_roundtrip() {
    let m = SymbolMetadata {
        visible: true,
        named: false,
        hidden: true,
        terminal: false,
    };
    let m2: SymbolMetadata = json_roundtrip(&m);
    assert_eq!(m, m2);
}

#[test]
fn grammar_default_roundtrip() {
    let g = Grammar::default();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn python_like_grammar_roundtrip() {
    let g = GrammarBuilder::python_like();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn javascript_like_grammar_roundtrip() {
    let g = GrammarBuilder::javascript_like();
    let g2: Grammar = json_roundtrip(&g);
    assert_eq!(g, g2);
}
