//! Serialization roundtrip tests for IR types.
//!
//! Covers: JSON roundtrip, all symbol types, external tokens, precedence,
//! fragile tokens, empty/default grammars, max-size values, normalization
//! preservation, partial grammars, alias sequences, and nested symbols.
//!
//! Each test verifies: original == deserialized(serialized(original))
//! by comparing `serde_json::Value` representations (Grammar does not derive PartialEq).

use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, Symbol, SymbolId, Token,
    TokenPattern,
};

/// Serialize `grammar` to JSON, deserialize back, re-serialize, and assert the
/// two JSON values are identical.
fn assert_json_roundtrip(grammar: &Grammar) {
    let json_value = serde_json::to_value(grammar).expect("serialize to Value");
    let json_str = serde_json::to_string_pretty(grammar).expect("serialize to string");
    let deserialized: Grammar = serde_json::from_str(&json_str).expect("deserialize from string");
    let roundtrip_value = serde_json::to_value(&deserialized).expect("re-serialize to Value");
    assert_eq!(json_value, roundtrip_value);
}

// ---------------------------------------------------------------------------
// 1. Grammar → JSON → Grammar roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_simple_grammar_json_roundtrip() {
    let mut grammar = Grammar::new("simple".to_string());

    // S -> NUMBER
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(0), "start".to_string());

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// 2. Grammar → bincode → Grammar roundtrip (not supported)
// ---------------------------------------------------------------------------
#[test]
fn test_bincode_not_supported() {
    // bincode is not a dependency of adze-ir; this test documents that fact.
    // If bincode support is added in the future, this test should be updated
    // to perform an actual roundtrip.
}

// ---------------------------------------------------------------------------
// 3. Complex grammar with all symbol types
// ---------------------------------------------------------------------------
#[test]
fn test_complex_grammar_all_symbol_types() {
    let mut grammar = Grammar::new("complex".to_string());

    // Tokens
    grammar.tokens.insert(
        SymbolId(10),
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(11),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(12),
        Token {
            name: "IDENT".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    // External token
    grammar.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(20),
    });

    // Rule using Terminal, NonTerminal, External, Optional, Repeat, RepeatOne,
    // Choice, Sequence, Epsilon
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::External(SymbolId(20)),
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(11)))),
            Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(12)))),
            Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(11)))),
            Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ]),
            Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(12)),
            ]),
            Symbol::Epsilon,
        ],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    });

    // A second rule for the inner non-terminal
    grammar.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(11))],
        precedence: Some(PrecedenceKind::Dynamic(3)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(1),
    });

    grammar
        .rule_names
        .insert(SymbolId(0), "expression".to_string());
    grammar.rule_names.insert(SymbolId(1), "atom".to_string());

    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// 4. Grammar with external tokens
// ---------------------------------------------------------------------------
#[test]
fn test_grammar_with_external_tokens() {
    let mut grammar = Grammar::new("external_tokens".to_string());

    grammar.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });
    grammar.externals.push(ExternalToken {
        name: "DEDENT".to_string(),
        symbol_id: SymbolId(101),
    });
    grammar.externals.push(ExternalToken {
        name: "NEWLINE".to_string(),
        symbol_id: SymbolId(102),
    });

    // Rule referencing externals
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::External(SymbolId(100)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::External(SymbolId(101)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "STMT".to_string(),
            pattern: TokenPattern::Regex(r".+".to_string()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(SymbolId(0), "block".to_string());

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// 5. Grammar with precedence declarations
// ---------------------------------------------------------------------------
#[test]
fn test_grammar_with_precedence_declarations() {
    let mut grammar = Grammar::new("precedence".to_string());

    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10), SymbolId(11)],
    });
    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(12)],
    });
    grammar.precedences.push(Precedence {
        level: 3,
        associativity: Associativity::None,
        symbols: vec![SymbolId(13)],
    });

    // Conflict declarations
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(10), SymbolId(12)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(2)),
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(11), SymbolId(13)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(10), SymbolId(11)],
        resolution: ConflictResolution::GLR,
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(12), SymbolId(13)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(1)),
    });

    // Tokens
    for (id, name) in [(10, "PLUS"), (11, "MINUS"), (12, "STAR"), (13, "SLASH")] {
        grammar.tokens.insert(
            SymbolId(id),
            Token {
                name: name.to_string(),
                pattern: TokenPattern::String(name.to_string()),
                fragile: false,
            },
        );
    }

    // Rules with both static and dynamic precedence
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(10)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(12)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        precedence: Some(PrecedenceKind::Dynamic(2)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(1),
    });

    grammar.rule_names.insert(SymbolId(0), "expr".to_string());

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// 6. Grammar with fragile tokens
// ---------------------------------------------------------------------------
#[test]
fn test_grammar_with_fragile_tokens() {
    let mut grammar = Grammar::new("fragile".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "KEYWORD_IF".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: true,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "IDENT".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            fragile: true,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "LPAREN".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );

    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar
        .rule_names
        .insert(SymbolId(0), "if_stmt".to_string());

    assert_json_roundtrip(&grammar);

    // Verify fragile flags survive the roundtrip
    let json = serde_json::to_string(&grammar).unwrap();
    let deserialized: Grammar = serde_json::from_str(&json).unwrap();
    let tok1 = deserialized.tokens.get(&SymbolId(1)).unwrap();
    let tok2 = deserialized.tokens.get(&SymbolId(2)).unwrap();
    let tok3 = deserialized.tokens.get(&SymbolId(3)).unwrap();
    assert!(tok1.fragile);
    assert!(tok2.fragile);
    assert!(!tok3.fragile);
}

// ---------------------------------------------------------------------------
// 7. Empty grammar roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_empty_grammar_roundtrip() {
    let grammar = Grammar::new("empty".to_string());
    assert_json_roundtrip(&grammar);
}

#[test]
fn test_default_grammar_roundtrip() {
    let grammar = Grammar::default();
    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// 8. Grammar with max-size values
// ---------------------------------------------------------------------------
#[test]
fn test_grammar_with_max_size_values() {
    let mut grammar = Grammar::new("max_values".to_string());

    // Max u16 IDs
    let max_sym = SymbolId(u16::MAX);
    let max_field = FieldId(u16::MAX);
    let max_prod = ProductionId(u16::MAX);
    let max_rule = RuleId(u16::MAX);

    grammar.tokens.insert(
        max_sym,
        Token {
            name: "MAX_TOKEN".to_string(),
            pattern: TokenPattern::String("max".to_string()),
            fragile: false,
        },
    );

    grammar.add_rule(Rule {
        lhs: max_sym,
        rhs: vec![Symbol::Terminal(max_sym)],
        precedence: Some(PrecedenceKind::Static(i16::MAX)),
        associativity: Some(Associativity::Left),
        fields: vec![(max_field, usize::MAX)],
        production_id: max_prod,
    });

    // Negative precedence extremes
    grammar.add_rule(Rule {
        lhs: max_sym,
        rhs: vec![Symbol::Epsilon],
        precedence: Some(PrecedenceKind::Dynamic(i16::MIN)),
        associativity: Some(Associativity::None),
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.precedences.push(Precedence {
        level: i16::MAX,
        associativity: Associativity::Left,
        symbols: vec![max_sym],
    });
    grammar.precedences.push(Precedence {
        level: i16::MIN,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(0)],
    });

    grammar.fields.insert(max_field, "max_field".to_string());

    grammar.production_ids.insert(max_rule, max_prod);
    grammar.max_alias_sequence_length = usize::MAX;

    grammar.rule_names.insert(max_sym, "max_rule".to_string());

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// 9. Normalized grammar roundtrip (verify normalization is preserved)
// ---------------------------------------------------------------------------
#[test]
fn test_normalized_grammar_roundtrip() {
    let mut grammar = Grammar::new("normalized".to_string());

    // Add tokens the rules will reference
    grammar.tokens.insert(
        SymbolId(10),
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(11),
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    // Rule with complex symbols: Optional, Repeat, Choice
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10)))),
            Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(11)))),
            Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ]),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.rule_names.insert(SymbolId(0), "start".to_string());

    // Normalize the grammar (expands complex symbols into auxiliary rules)
    grammar.normalize();

    // After normalization, there should be no Optional/Repeat/Choice in RHS
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Optional(_)
                | Symbol::Repeat(_)
                | Symbol::RepeatOne(_)
                | Symbol::Choice(_) => {
                    panic!("Normalization left complex symbol: {:?}", sym);
                }
                _ => {}
            }
        }
    }

    // Roundtrip the normalized grammar
    assert_json_roundtrip(&grammar);

    // Verify the deserialized grammar is still normalized
    let json = serde_json::to_string(&grammar).unwrap();
    let deserialized: Grammar = serde_json::from_str(&json).unwrap();
    for rule in deserialized.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Optional(_)
                | Symbol::Repeat(_)
                | Symbol::RepeatOne(_)
                | Symbol::Choice(_) => {
                    panic!("Normalization not preserved after roundtrip: {:?}", sym);
                }
                _ => {}
            }
        }
    }

    // Rule count should match
    let original_count = grammar.all_rules().count();
    let deserialized_count = deserialized.all_rules().count();
    assert_eq!(original_count, deserialized_count);
}

// ---------------------------------------------------------------------------
// 10. Partial grammar (missing optional fields)
// ---------------------------------------------------------------------------
#[test]
fn test_partial_grammar_missing_optional_fields() {
    let mut grammar = Grammar::new("partial".to_string());

    // Only set a minimal rule — leave precedences, conflicts, externals,
    // extras, fields, supertypes, inline_rules, alias_sequences, etc. empty.
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // symbol_registry is None
    assert!(grammar.symbol_registry.is_none());

    assert_json_roundtrip(&grammar);

    // Verify all empty/default collections survive
    let json = serde_json::to_string(&grammar).unwrap();
    let deserialized: Grammar = serde_json::from_str(&json).unwrap();
    assert!(deserialized.tokens.is_empty());
    assert!(deserialized.precedences.is_empty());
    assert!(deserialized.conflicts.is_empty());
    assert!(deserialized.externals.is_empty());
    assert!(deserialized.extras.is_empty());
    assert!(deserialized.fields.is_empty());
    assert!(deserialized.supertypes.is_empty());
    assert!(deserialized.inline_rules.is_empty());
    assert!(deserialized.alias_sequences.is_empty());
    assert!(deserialized.production_ids.is_empty());
    assert_eq!(deserialized.max_alias_sequence_length, 0);
    assert!(deserialized.symbol_registry.is_none());
}

// ---------------------------------------------------------------------------
// Bonus: alias sequences and production IDs roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_alias_sequences_roundtrip() {
    let mut grammar = Grammar::new("aliases".to_string());

    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("expr".to_string()), None, Some("term".to_string())],
        },
    );
    grammar.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![None, None],
        },
    );

    grammar.production_ids.insert(RuleId(0), ProductionId(0));
    grammar.production_ids.insert(RuleId(1), ProductionId(1));

    grammar.max_alias_sequence_length = 3;

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// Bonus: extras, supertypes, inline_rules roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_extras_supertypes_inline_rules_roundtrip() {
    let mut grammar = Grammar::new("extras".to_string());

    grammar.extras.push(SymbolId(50));
    grammar.extras.push(SymbolId(51));

    grammar.supertypes.push(SymbolId(60));
    grammar.supertypes.push(SymbolId(61));

    grammar.inline_rules.push(SymbolId(70));

    // Tokens referenced by extras
    grammar.tokens.insert(
        SymbolId(50),
        Token {
            name: "_whitespace".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(51),
        Token {
            name: "_comment".to_string(),
            pattern: TokenPattern::Regex(r"//.*".to_string()),
            fragile: false,
        },
    );

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// Bonus: symbol registry roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_grammar_with_symbol_registry_roundtrip() {
    let mut grammar = Grammar::new("registry".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(SymbolId(0), "start".to_string());

    // Build and attach the registry
    grammar.get_or_build_registry();
    assert!(grammar.symbol_registry.is_some());

    assert_json_roundtrip(&grammar);
}

// ---------------------------------------------------------------------------
// Bonus: deeply nested symbol roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_deeply_nested_symbol_roundtrip() {
    let mut grammar = Grammar::new("nested".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    // Optional(Repeat(RepeatOne(Choice([Terminal, Sequence([Terminal, Epsilon])]))))
    let deep = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::RepeatOne(
        Box::new(Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Sequence(vec![Symbol::Terminal(SymbolId(1)), Symbol::Epsilon]),
        ])),
    )))));

    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![deep],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    assert_json_roundtrip(&grammar);
}
