#![allow(clippy::needless_range_loop)]

//! Comprehensive serialization/deserialization tests for the full Grammar struct.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, Symbol, SymbolId,
    SymbolMetadata, SymbolRegistry, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> T {
    let json = serde_json::to_string(val).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

fn roundtrip_pretty<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> T {
    let json = serde_json::to_string_pretty(val).expect("serialize pretty");
    serde_json::from_str(&json).expect("deserialize pretty")
}

/// A complex arithmetic grammar with precedence, extras, and multiple rules.
fn complex_arith() -> Grammar {
    GrammarBuilder::new("complex_arith")
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
        .build()
}

// ===========================================================================
// 1. Full Grammar struct: every field survives roundtrip
// ===========================================================================
#[test]
fn test_all_grammar_fields_roundtrip() {
    let mut g = Grammar::new("all_fields".into());
    // rules
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(3)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(7),
    });
    // tokens
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "NUM".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    // precedences
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });
    // conflicts
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    // externals
    g.externals.push(ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(10),
    });
    // extras
    g.extras.push(SymbolId(20));
    // fields
    g.fields.insert(FieldId(0), "value".into());
    // supertypes
    g.supertypes.push(SymbolId(30));
    // inline_rules
    g.inline_rules.push(SymbolId(40));
    // alias_sequences
    g.alias_sequences.insert(
        ProductionId(7),
        AliasSequence {
            aliases: vec![Some("alias_a".into()), None],
        },
    );
    // production_ids
    g.production_ids.insert(RuleId(0), ProductionId(7));
    // max_alias_sequence_length
    g.max_alias_sequence_length = 2;
    // rule_names
    g.rule_names.insert(SymbolId(1), "start".into());

    let g2 = roundtrip(&g);

    assert_eq!(g.name, g2.name);
    assert_eq!(g.rules.len(), g2.rules.len());
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.precedences.len(), g2.precedences.len());
    assert_eq!(g.conflicts.len(), g2.conflicts.len());
    assert_eq!(g.externals.len(), g2.externals.len());
    assert_eq!(g.extras, g2.extras);
    assert_eq!(g.fields.len(), g2.fields.len());
    assert_eq!(g.supertypes, g2.supertypes);
    assert_eq!(g.inline_rules, g2.inline_rules);
    assert_eq!(g.alias_sequences.len(), g2.alias_sequences.len());
    assert_eq!(g.production_ids.len(), g2.production_ids.len());
    assert_eq!(g.max_alias_sequence_length, g2.max_alias_sequence_length);
    assert_eq!(g.rule_names.len(), g2.rule_names.len());
}

// ===========================================================================
// 2. Complex builder grammar: compact & pretty JSON agree
// ===========================================================================
#[test]
fn test_compact_vs_pretty_equivalence() {
    let g = complex_arith();
    let compact: Grammar = roundtrip(&g);
    let pretty: Grammar = roundtrip_pretty(&g);

    let json_c = serde_json::to_string(&compact).unwrap();
    let json_p = serde_json::to_string(&pretty).unwrap();
    assert_eq!(json_c, json_p);
}

// ===========================================================================
// 3. Multiple rules per LHS survive roundtrip with order preserved
// ===========================================================================
#[test]
fn test_multiple_alternatives_order_preserved() {
    let g = complex_arith();
    let g2 = roundtrip(&g);

    for (sym, rules) in &g.rules {
        let rules2 = &g2.rules[sym];
        assert_eq!(rules.len(), rules2.len());
        for i in 0..rules.len() {
            assert_eq!(rules[i].rhs, rules2[i].rhs);
            assert_eq!(rules[i].precedence, rules2[i].precedence);
            assert_eq!(rules[i].associativity, rules2[i].associativity);
            assert_eq!(rules[i].production_id, rules2[i].production_id);
        }
    }
}

// ===========================================================================
// 4. Token patterns: string vs regex distinction preserved
// ===========================================================================
#[test]
fn test_token_pattern_type_preserved() {
    let g = GrammarBuilder::new("patterns")
        .token("PLUS", "+")
        .token("NUM", r"\d+")
        .rule("e", vec!["NUM", "PLUS", "NUM"])
        .start("e")
        .build();

    let g2 = roundtrip(&g);

    for (id, tok) in &g.tokens {
        let tok2 = &g2.tokens[id];
        assert_eq!(tok.pattern, tok2.pattern);
    }
}

// ===========================================================================
// 5. Fragile token flag preserved
// ===========================================================================
#[test]
fn test_fragile_flag_preserved() {
    let g = GrammarBuilder::new("f")
        .fragile_token("ERR", "ERR")
        .token("OK", "OK")
        .rule("s", vec!["OK"])
        .start("s")
        .build();

    let g2 = roundtrip(&g);
    let err = g2.tokens.values().find(|t| t.name == "ERR").unwrap();
    let ok = g2.tokens.values().find(|t| t.name == "OK").unwrap();
    assert!(err.fragile);
    assert!(!ok.fragile);
}

// ===========================================================================
// 6. Precedence levels: static and dynamic roundtrip
// ===========================================================================
#[test]
fn test_precedence_static_dynamic_roundtrip() {
    let cases = [
        PrecedenceKind::Static(i16::MIN),
        PrecedenceKind::Static(0),
        PrecedenceKind::Static(i16::MAX),
        PrecedenceKind::Dynamic(-1),
        PrecedenceKind::Dynamic(0),
        PrecedenceKind::Dynamic(100),
    ];
    for pk in &cases {
        assert_eq!(&roundtrip(pk), pk);
    }
}

// ===========================================================================
// 7. All Symbol enum variants roundtrip
// ===========================================================================
#[test]
fn test_every_symbol_variant_roundtrip() {
    let variants: Vec<Symbol> = vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::NonTerminal(SymbolId(u16::MAX)),
        Symbol::External(SymbolId(42)),
        Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))),
        Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(2)))),
        Symbol::RepeatOne(Box::new(Symbol::External(SymbolId(3)))),
        Symbol::Choice(vec![Symbol::Epsilon, Symbol::Terminal(SymbolId(4))]),
        Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(5)),
            Symbol::NonTerminal(SymbolId(6)),
        ]),
        Symbol::Epsilon,
    ];
    for v in &variants {
        assert_eq!(&roundtrip(v), v);
    }
}

// ===========================================================================
// 8. Deeply nested symbols roundtrip
// ===========================================================================
#[test]
fn test_deeply_nested_symbol_serde() {
    // Optional(Repeat(RepeatOne(Choice([Sequence([T, NT]), External]))))
    let sym = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::RepeatOne(
        Box::new(Symbol::Choice(vec![
            Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::NonTerminal(SymbolId(2)),
            ]),
            Symbol::External(SymbolId(3)),
        ])),
    )))));
    assert_eq!(roundtrip(&sym), sym);
}

// ===========================================================================
// 9. ConflictResolution all variants
// ===========================================================================
#[test]
fn test_conflict_resolution_all_variants() {
    let variants = vec![
        ConflictResolution::Precedence(PrecedenceKind::Static(5)),
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(-3)),
        ConflictResolution::Associativity(Associativity::Left),
        ConflictResolution::Associativity(Associativity::Right),
        ConflictResolution::Associativity(Associativity::None),
        ConflictResolution::GLR,
    ];
    for v in &variants {
        assert_eq!(&roundtrip(v), v);
    }
}

// ===========================================================================
// 10. Associativity all variants
// ===========================================================================
#[test]
fn test_associativity_all_variants() {
    for a in [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ] {
        assert_eq!(roundtrip(&a), a);
    }
}

// ===========================================================================
// 11. SymbolMetadata boolean combos
// ===========================================================================
#[test]
fn test_symbol_metadata_all_combos() {
    for vis in [true, false] {
        for named in [true, false] {
            for hidden in [true, false] {
                for terminal in [true, false] {
                    let m = SymbolMetadata {
                        visible: vis,
                        named,
                        hidden,
                        terminal,
                    };
                    assert_eq!(roundtrip(&m), m);
                }
            }
        }
    }
}

// ===========================================================================
// 12. AliasSequence with mixed Some/None aliases
// ===========================================================================
#[test]
fn test_alias_sequence_mixed_roundtrip() {
    let seq = AliasSequence {
        aliases: vec![Some("first".into()), None, None, Some("last".into()), None],
    };
    let json = serde_json::to_string(&seq).unwrap();
    let seq2: AliasSequence = serde_json::from_str(&json).unwrap();
    assert_eq!(seq.aliases.len(), seq2.aliases.len());
    for i in 0..seq.aliases.len() {
        assert_eq!(seq.aliases[i], seq2.aliases[i]);
    }
}

// ===========================================================================
// 13. SymbolRegistry roundtrip
// ===========================================================================
#[test]
fn test_symbol_registry_roundtrip() {
    let mut reg = SymbolRegistry::new();
    reg.register(
        "plus",
        SymbolMetadata {
            visible: true,
            named: false,
            hidden: false,
            terminal: true,
        },
    );
    reg.register(
        "expression",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );

    let reg2 = roundtrip(&reg);
    assert_eq!(reg.len(), reg2.len());
    assert_eq!(reg.get_id("plus"), reg2.get_id("plus"));
    assert_eq!(reg.get_id("expression"), reg2.get_id("expression"));
    assert_eq!(
        reg.get_metadata(reg.get_id("plus").unwrap()),
        reg2.get_metadata(reg2.get_id("plus").unwrap())
    );
}

// ===========================================================================
// 14. Grammar with registry embedded
// ===========================================================================
#[test]
fn test_grammar_with_registry_embedded() {
    let mut g = complex_arith();
    let _ = g.get_or_build_registry();
    assert!(g.symbol_registry.is_some());

    let g2 = roundtrip(&g);
    assert!(g2.symbol_registry.is_some());
    let r1 = g.symbol_registry.as_ref().unwrap();
    let r2 = g2.symbol_registry.as_ref().unwrap();
    assert_eq!(r1.len(), r2.len());
}

// ===========================================================================
// 15. Grammar without registry (None)
// ===========================================================================
#[test]
fn test_grammar_without_registry() {
    let g = Grammar::new("no_reg".into());
    assert!(g.symbol_registry.is_none());
    let g2 = roundtrip(&g);
    assert!(g2.symbol_registry.is_none());
}

// ===========================================================================
// 16. IndexMap key order preserved for rules
// ===========================================================================
#[test]
fn test_rules_indexmap_order_preserved() {
    let g = complex_arith();
    let g2 = roundtrip(&g);

    let keys1: Vec<_> = g.rules.keys().collect();
    let keys2: Vec<_> = g2.rules.keys().collect();
    assert_eq!(keys1, keys2);
}

// ===========================================================================
// 17. IndexMap key order preserved for tokens
// ===========================================================================
#[test]
fn test_tokens_indexmap_order_preserved() {
    let g = complex_arith();
    let g2 = roundtrip(&g);

    let keys1: Vec<_> = g.tokens.keys().collect();
    let keys2: Vec<_> = g2.tokens.keys().collect();
    assert_eq!(keys1, keys2);
}

// ===========================================================================
// 18. Extras list preserved
// ===========================================================================
#[test]
fn test_extras_preserved() {
    let g = GrammarBuilder::new("ext")
        .token("WS", r"\s+")
        .token("COMMENT", r"//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .rule("a", vec![])
        .start("a")
        .build();

    let g2 = roundtrip(&g);
    assert_eq!(g.extras, g2.extras);
}

// ===========================================================================
// 19. External tokens preserved
// ===========================================================================
#[test]
fn test_external_tokens_preserved() {
    let g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("NEWLINE", "NEWLINE")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .rule("s", vec![])
        .start("s")
        .build();

    let g2 = roundtrip(&g);
    assert_eq!(g.externals.len(), g2.externals.len());
    for i in 0..g.externals.len() {
        assert_eq!(g.externals[i].name, g2.externals[i].name);
        assert_eq!(g.externals[i].symbol_id, g2.externals[i].symbol_id);
    }
}

// ===========================================================================
// 20. Precedence declarations preserved
// ===========================================================================
#[test]
fn test_precedence_declarations_preserved() {
    let g = complex_arith();
    let g2 = roundtrip(&g);

    assert_eq!(g.precedences.len(), g2.precedences.len());
    for i in 0..g.precedences.len() {
        assert_eq!(g.precedences[i].level, g2.precedences[i].level);
        assert_eq!(
            g.precedences[i].associativity,
            g2.precedences[i].associativity
        );
        assert_eq!(g.precedences[i].symbols, g2.precedences[i].symbols);
    }
}

// ===========================================================================
// 21. Python-like grammar roundtrip (nullable start + externals)
// ===========================================================================
#[test]
fn test_python_like_full_roundtrip() {
    let g = GrammarBuilder::python_like();
    let g2 = roundtrip(&g);

    assert_eq!(g.name, g2.name);
    assert_eq!(g.rules.len(), g2.rules.len());
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.externals.len(), g2.externals.len());
    assert_eq!(g.extras, g2.extras);

    // Verify the nullable start rule survived
    for (sym, rules) in &g.rules {
        let rules2 = &g2.rules[sym];
        assert_eq!(rules.len(), rules2.len());
    }
}

// ===========================================================================
// 22. JavaScript-like grammar roundtrip (precedence rules)
// ===========================================================================
#[test]
fn test_javascript_like_full_roundtrip() {
    let g = GrammarBuilder::javascript_like();
    let g2 = roundtrip(&g);

    let prec_count_orig = g.all_rules().filter(|r| r.precedence.is_some()).count();
    let prec_count_rt = g2.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count_orig, prec_count_rt);
    assert!(prec_count_rt >= 4);
}

// ===========================================================================
// 23. Double roundtrip produces identical JSON (idempotency)
// ===========================================================================
#[test]
fn test_triple_roundtrip_idempotent() {
    let g = complex_arith();
    let j1 = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&g2).unwrap();
    let g3: Grammar = serde_json::from_str(&j2).unwrap();
    let j3 = serde_json::to_string(&g3).unwrap();
    assert_eq!(j1, j2);
    assert_eq!(j2, j3);
}

// ===========================================================================
// 24. from_macro_output works with serialized grammar
// ===========================================================================
#[test]
fn test_from_macro_output_complex_grammar() {
    let g = complex_arith();
    let json = serde_json::to_string(&g).unwrap();
    let g2 = Grammar::from_macro_output(&json).unwrap();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.rules.len(), g2.rules.len());
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

// ===========================================================================
// 25. from_macro_output rejects invalid JSON
// ===========================================================================
#[test]
fn test_from_macro_output_rejects_garbage() {
    assert!(Grammar::from_macro_output("}{invalid").is_err());
    assert!(Grammar::from_macro_output("").is_err());
    assert!(Grammar::from_macro_output("null").is_err());
    assert!(Grammar::from_macro_output("42").is_err());
}

// ===========================================================================
// 26. Empty epsilon rule roundtrip
// ===========================================================================
#[test]
fn test_epsilon_rule_roundtrip() {
    let mut g = Grammar::new("eps".into());
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "empty".into());

    let g2 = roundtrip(&g);
    let rules2 = &g2.rules[&SymbolId(1)];
    assert_eq!(rules2.len(), 1);
    assert_eq!(rules2[0].rhs, vec![Symbol::Epsilon]);
}

// ===========================================================================
// 27. Large symbol IDs (boundary values)
// ===========================================================================
#[test]
fn test_boundary_symbol_ids() {
    let ids = [
        SymbolId(0),
        SymbolId(1),
        SymbolId(u16::MAX / 2),
        SymbolId(u16::MAX - 1),
        SymbolId(u16::MAX),
    ];
    for id in &ids {
        assert_eq!(&roundtrip(id), id);
    }
}

// ===========================================================================
// 28. Rule with all optional fields set to None
// ===========================================================================
#[test]
fn test_rule_with_none_optionals() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let rule2 = roundtrip(&rule);
    assert_eq!(rule, rule2);
    assert!(rule2.precedence.is_none());
    assert!(rule2.associativity.is_none());
    assert!(rule2.fields.is_empty());
}

// ===========================================================================
// 29. Rule with all optional fields populated
// ===========================================================================
#[test]
fn test_rule_with_all_optionals_set() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(SymbolId(4)),
        ],
        precedence: Some(PrecedenceKind::Dynamic(-5)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(99),
    };
    let rule2 = roundtrip(&rule);
    assert_eq!(rule, rule2);
}

// ===========================================================================
// 30. Unicode in token names and patterns
// ===========================================================================
#[test]
fn test_unicode_in_tokens() {
    let mut g = Grammar::new("unicode".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "数字".into(),
            pattern: TokenPattern::Regex(r"[０-９]+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "演算子".into(),
            pattern: TokenPattern::String("＋".into()),
            fragile: false,
        },
    );

    let g2 = roundtrip(&g);
    assert_eq!(g2.tokens[&SymbolId(1)].name, "数字");
    assert_eq!(
        g2.tokens[&SymbolId(1)].pattern,
        TokenPattern::Regex(r"[０-９]+".into())
    );
    assert_eq!(g2.tokens[&SymbolId(2)].name, "演算子");
    assert_eq!(
        g2.tokens[&SymbolId(2)].pattern,
        TokenPattern::String("＋".into())
    );
}

// ===========================================================================
// 31. Grammar Default trait roundtrip
// ===========================================================================
#[test]
fn test_default_grammar_roundtrip() {
    let g: Grammar = Grammar::default();
    let g2 = roundtrip(&g);
    assert_eq!(g.name, g2.name);
    assert!(g2.rules.is_empty());
    assert!(g2.tokens.is_empty());
    assert!(g2.symbol_registry.is_none());
}

// ===========================================================================
// 32. Normalized grammar roundtrip
// ===========================================================================
#[test]
fn test_normalized_grammar_serde() {
    let mut g = Grammar::new("norm".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "y".into(),
            pattern: TokenPattern::String("y".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1)))),
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(2)))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(10), "start".into());

    g.normalize();
    let total_before: usize = g.rules.values().map(|r| r.len()).sum();
    assert!(total_before > 1);

    let g2 = roundtrip(&g);
    let total_after: usize = g2.rules.values().map(|r| r.len()).sum();
    assert_eq!(total_before, total_after);
}

// ===========================================================================
// 33. Many conflict declarations roundtrip
// ===========================================================================
#[test]
fn test_many_conflicts_roundtrip() {
    let mut g = Grammar::new("many_conflicts".into());
    for i in 0..20 {
        g.conflicts.push(ConflictDeclaration {
            symbols: vec![SymbolId(i), SymbolId(i + 100)],
            resolution: if i % 3 == 0 {
                ConflictResolution::GLR
            } else if i % 3 == 1 {
                ConflictResolution::Precedence(PrecedenceKind::Static(i as i16))
            } else {
                ConflictResolution::Associativity(Associativity::Left)
            },
        });
    }

    let g2 = roundtrip(&g);
    assert_eq!(g2.conflicts.len(), 20);
    for i in 0..20 {
        assert_eq!(g.conflicts[i].symbols, g2.conflicts[i].symbols);
        assert_eq!(g.conflicts[i].resolution, g2.conflicts[i].resolution);
    }
}

// ===========================================================================
// 34. Many alias sequences roundtrip
// ===========================================================================
#[test]
fn test_many_alias_sequences_roundtrip() {
    let mut g = Grammar::new("aliases".into());
    for i in 0..15u16 {
        let aliases: Vec<Option<String>> = (0..i + 1)
            .map(|j| {
                if j % 2 == 0 {
                    Some(format!("alias_{}_{}", i, j))
                } else {
                    None
                }
            })
            .collect();
        g.alias_sequences
            .insert(ProductionId(i), AliasSequence { aliases });
    }
    g.max_alias_sequence_length = 15;

    let g2 = roundtrip(&g);
    assert_eq!(g2.alias_sequences.len(), 15);
    assert_eq!(g2.max_alias_sequence_length, 15);
    for (pid, seq) in &g.alias_sequences {
        let seq2 = &g2.alias_sequences[pid];
        assert_eq!(seq.aliases, seq2.aliases);
    }
}

// ===========================================================================
// 35. JSON contains expected top-level keys
// ===========================================================================
#[test]
fn test_json_contains_all_top_level_keys() {
    let g = complex_arith();
    let json = serde_json::to_string(&g).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = val.as_object().unwrap();

    let expected_keys = [
        "name",
        "rules",
        "tokens",
        "precedences",
        "conflicts",
        "externals",
        "extras",
        "fields",
        "supertypes",
        "inline_rules",
        "alias_sequences",
        "production_ids",
        "max_alias_sequence_length",
        "rule_names",
        "symbol_registry",
    ];
    for key in &expected_keys {
        assert!(obj.contains_key(*key), "missing key: {}", key);
    }
}

// ===========================================================================
// 36. Deserialize with extra unknown fields (forward compatibility)
// ===========================================================================
#[test]
fn test_deserialize_with_extra_fields() {
    let g = Grammar::new("compat".into());
    let json = serde_json::to_string(&g).unwrap();
    // Inject an unknown field before the final closing brace
    let json = format!(r#"{},"future_field":"hello"}}"#, &json[..json.len() - 1]);

    // serde default behavior: unknown fields are ignored (Grammar doesn't deny_unknown_fields)
    let result: Result<Grammar, _> = serde_json::from_str(&json);
    assert!(result.is_ok(), "should ignore unknown fields");
    assert_eq!(result.unwrap().name, "compat");
}

// ===========================================================================
// 37. Serialized JSON size is stable across identical grammars
// ===========================================================================
#[test]
fn test_json_size_deterministic() {
    let g1 = complex_arith();
    let g2 = complex_arith();
    let j1 = serde_json::to_string(&g1).unwrap();
    let j2 = serde_json::to_string(&g2).unwrap();
    assert_eq!(j1.len(), j2.len());
    assert_eq!(j1, j2);
}

// ===========================================================================
// 38. Empty Choice and Sequence roundtrip
// ===========================================================================
#[test]
fn test_empty_choice_and_sequence() {
    let empty_choice = Symbol::Choice(vec![]);
    let empty_seq = Symbol::Sequence(vec![]);

    assert_eq!(roundtrip(&empty_choice), empty_choice);
    assert_eq!(roundtrip(&empty_seq), empty_seq);
}

// ===========================================================================
// 39. Production IDs map with many entries
// ===========================================================================
#[test]
fn test_production_ids_many_entries() {
    let mut g = Grammar::new("pids".into());
    for i in 0..50u16 {
        g.production_ids.insert(RuleId(i), ProductionId(i * 10 + 1));
    }

    let g2 = roundtrip(&g);
    assert_eq!(g2.production_ids.len(), 50);
    for i in 0..50u16 {
        assert_eq!(g2.production_ids[&RuleId(i)], ProductionId(i * 10 + 1));
    }
}

// ===========================================================================
// 40. Supertypes and inline_rules with many entries
// ===========================================================================
#[test]
fn test_supertypes_inline_rules_many() {
    let mut g = Grammar::new("many".into());
    g.supertypes = (0..25).map(SymbolId).collect();
    g.inline_rules = (100..120).map(SymbolId).collect();

    let g2 = roundtrip(&g);
    assert_eq!(g2.supertypes.len(), 25);
    assert_eq!(g2.inline_rules.len(), 20);
    for i in 0..25 {
        assert_eq!(g2.supertypes[i], SymbolId(i as u16));
    }
    for i in 0..20 {
        assert_eq!(g2.inline_rules[i], SymbolId(100 + i as u16));
    }
}
