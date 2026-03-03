//! Comprehensive fuzz-like edge case tests for the IR crate.
//!
//! These tests exercise extreme/boundary conditions of the Grammar IR:
//! - Empty and huge grammars
//! - Unicode and special characters in names
//! - Deep nesting
//! - Circular references
//! - Boundary values
//! - All variants combined

use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use indexmap::IndexMap;

// ============================================================================
// Test 1: Grammar with 0 rules (empty grammar)
// ============================================================================
#[test]
fn test_empty_grammar() {
    let grammar = Grammar::new("empty".to_string());
    assert_eq!(grammar.rules.len(), 0);
    assert_eq!(grammar.tokens.len(), 0);
    assert_eq!(grammar.precedences.len(), 0);
    assert_eq!(grammar.conflicts.len(), 0);
    assert_eq!(grammar.externals.len(), 0);
    assert_eq!(grammar.extras.len(), 0);
    assert_eq!(grammar.name, "empty");
    // Should validate successfully even though empty
    assert!(grammar.validate().is_ok());
}

// ============================================================================
// Test 2: Grammar with 1000+ rules
// ============================================================================
#[test]
fn test_grammar_with_1000_rules() {
    let mut grammar = Grammar::new("large".to_string());

    // Create 1000 rules with unique LHS symbols
    for i in 0..1000 {
        let lhs = SymbolId(i as u16);
        let rhs = vec![Symbol::Terminal(SymbolId((i + 1) as u16))];
        let rule = Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        };
        grammar.add_rule(rule);
        grammar.rule_names.insert(lhs, format!("rule_{}", i));
    }

    assert_eq!(grammar.rules.len(), 1000);
    // Count total rules
    let total_rules: usize = grammar.rules.values().map(|v| v.len()).sum();
    assert_eq!(total_rules, 1000);
}

// ============================================================================
// Test 3: Symbol names with unicode characters
// ============================================================================
#[test]
fn test_symbol_names_with_unicode() {
    let mut grammar = Grammar::new("unicode_test".to_string());

    let unicode_names = vec![
        "ñame",
        "名前",
        "имя",
        "όνομα",
        "שם",
        "नाम",
        "🎉emoji",
        "Ώμέγα",
    ];

    for (i, name) in unicode_names.iter().enumerate() {
        let symbol_id = SymbolId(i as u16);
        grammar.rule_names.insert(symbol_id, name.to_string());
    }

    assert_eq!(grammar.rule_names.len(), 8);
    // Verify all unicode names are preserved
    for (i, name) in unicode_names.iter().enumerate() {
        let symbol_id = SymbolId(i as u16);
        assert_eq!(grammar.rule_names.get(&symbol_id), Some(&name.to_string()));
    }
}

// ============================================================================
// Test 4: Symbol names that are empty strings
// ============================================================================
#[test]
fn test_empty_symbol_names() {
    let mut grammar = Grammar::new("empty_names".to_string());

    // Create rules with empty names
    let lhs = SymbolId(1);
    let rule = Rule {
        lhs,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };
    grammar.add_rule(rule);
    grammar.rule_names.insert(lhs, String::new());

    // Should be able to store and retrieve empty names
    assert_eq!(grammar.rule_names.get(&lhs), Some(&String::new()));
}

// ============================================================================
// Test 5: Symbol names with max-length strings (1000+ chars)
// ============================================================================
#[test]
fn test_symbol_names_with_max_length() {
    let mut grammar = Grammar::new("long_names".to_string());

    // Create a very long name (1000+ chars)
    let long_name = "a".repeat(1500);
    let symbol_id = SymbolId(1);
    grammar.rule_names.insert(symbol_id, long_name.clone());

    assert_eq!(grammar.rule_names.get(&symbol_id).unwrap().len(), 1500);
    assert_eq!(grammar.rule_names.get(&symbol_id).unwrap(), &long_name);
}

// ============================================================================
// Test 6: Deeply nested Symbol trees (50 levels deep)
// ============================================================================
#[test]
fn test_deeply_nested_symbols() {
    let mut symbol = Symbol::Terminal(SymbolId(1));

    // Create 50 levels of Optional nesting
    for _ in 0..50 {
        symbol = Symbol::Optional(Box::new(symbol));
    }

    // Verify the nesting exists by pattern matching
    let mut current = &symbol;
    let mut depth = 0;
    loop {
        match current {
            Symbol::Optional(inner) => {
                depth += 1;
                current = inner;
            }
            Symbol::Terminal(_) => {
                break;
            }
            _ => panic!("Unexpected symbol type"),
        }
    }
    assert_eq!(depth, 50);
}

// ============================================================================
// Test 7: Circular rule references (A -> B, B -> A)
// ============================================================================
#[test]
fn test_circular_rule_references() {
    let mut grammar = Grammar::new("circular".to_string());

    let a_id = SymbolId(1);
    let b_id = SymbolId(2);

    // Rule A -> B
    let rule_a = Rule {
        lhs: a_id,
        rhs: vec![Symbol::NonTerminal(b_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    // Rule B -> A
    let rule_b = Rule {
        lhs: b_id,
        rhs: vec![Symbol::NonTerminal(a_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    };

    grammar.add_rule(rule_a);
    grammar.add_rule(rule_b);
    grammar.rule_names.insert(a_id, "A".to_string());
    grammar.rule_names.insert(b_id, "B".to_string());

    assert_eq!(grammar.rules.len(), 2);
    // Grammar structure can represent circularity even if validation would flag it
}

// ============================================================================
// Test 8: Grammar with duplicate rule names
// ============================================================================
#[test]
fn test_duplicate_rule_names() {
    let mut grammar = Grammar::new("duplicates".to_string());

    let a_id = SymbolId(1);
    let b_id = SymbolId(2);

    // Both rules get the same name
    grammar.rule_names.insert(a_id, "rule".to_string());
    grammar.rule_names.insert(b_id, "rule".to_string());

    // Both entries exist in the map with different keys
    assert_eq!(grammar.rule_names.len(), 2);
    // But both have the same value
    assert_eq!(grammar.rule_names.get(&a_id), Some(&"rule".to_string()));
    assert_eq!(grammar.rule_names.get(&b_id), Some(&"rule".to_string()));
}

// ============================================================================
// Test 9: Grammar with all symbol types
// ============================================================================
#[test]
fn test_grammar_with_all_symbol_types() {
    let mut grammar = Grammar::new("all_types".to_string());

    let symbols = vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::External(SymbolId(3)),
        Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(4)))),
        Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(5)))),
        Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(6)))),
        Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(7)),
            Symbol::Terminal(SymbolId(8)),
        ]),
        Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(9)),
            Symbol::NonTerminal(SymbolId(10)),
        ]),
        Symbol::Epsilon,
    ];

    let lhs = SymbolId(100);
    for (i, symbol) in symbols.iter().enumerate() {
        let rule = Rule {
            lhs,
            rhs: vec![symbol.clone()],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        };
        grammar.add_rule(rule);
    }

    assert_eq!(grammar.all_rules().count(), 9);
}

// ============================================================================
// Test 10: Rules with empty sequences
// ============================================================================
#[test]
fn test_rules_with_empty_sequences() {
    let mut grammar = Grammar::new("empty_sequences".to_string());

    let lhs = SymbolId(1);

    // Rule with empty RHS (epsilon production)
    let rule = Rule {
        lhs,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.add_rule(rule);
    assert_eq!(grammar.get_rules_for_symbol(lhs).unwrap().len(), 1);
    assert_eq!(grammar.get_rules_for_symbol(lhs).unwrap()[0].rhs.len(), 0);
}

// ============================================================================
// Test 11: Rules with single-element choices
// ============================================================================
#[test]
fn test_rules_with_single_element_choices() {
    let mut grammar = Grammar::new("single_choice".to_string());

    let lhs = SymbolId(1);
    let rule = Rule {
        lhs,
        rhs: vec![Symbol::Choice(vec![Symbol::Terminal(SymbolId(2))])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.add_rule(rule);
    let rules = grammar.get_rules_for_symbol(lhs).unwrap();
    assert_eq!(rules.len(), 1);
    if let Symbol::Choice(choices) = &rules[0].rhs[0] {
        assert_eq!(choices.len(), 1);
    } else {
        panic!("Expected Choice");
    }
}

// ============================================================================
// Test 12: Precedence values at i16::MIN and i16::MAX
// ============================================================================
#[test]
fn test_precedence_boundary_values() {
    let mut grammar = Grammar::new("prec_bounds".to_string());

    let min_prec = Precedence {
        level: i16::MIN,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };

    let max_prec = Precedence {
        level: i16::MAX,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)],
    };

    grammar.precedences.push(min_prec);
    grammar.precedences.push(max_prec);

    assert_eq!(grammar.precedences[0].level, i16::MIN);
    assert_eq!(grammar.precedences[1].level, i16::MAX);
}

// ============================================================================
// Test 13: Grammar with max alias_sequences
// ============================================================================
#[test]
fn test_grammar_with_many_alias_sequences() {
    let mut grammar = Grammar::new("many_aliases".to_string());

    // Create 500 alias sequences
    for i in 0..500 {
        let alias_seq = AliasSequence {
            aliases: vec![Some(format!("alias_{}", i)); 10],
        };
        grammar
            .alias_sequences
            .insert(ProductionId(i as u16), alias_seq);
    }

    assert_eq!(grammar.alias_sequences.len(), 500);
}

// ============================================================================
// Test 14: Grammar with deeply nested Choice variants
// ============================================================================
#[test]
fn test_deeply_nested_choice_variants() {
    // Create a deeply nested choice structure
    let mut choices = vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))];

    // Nest choices 20 levels deep
    for i in 3..23 {
        let choice = Symbol::Choice(choices);
        choices = vec![choice, Symbol::Terminal(SymbolId(i))];
    }

    let final_symbol = Symbol::Choice(choices);

    // Verify it was created
    assert!(matches!(final_symbol, Symbol::Choice(_)));
}

// ============================================================================
// Test 15: Symbol IDs at u16::MAX boundary
// ============================================================================
#[test]
fn test_symbol_ids_at_boundary() {
    let mut grammar = Grammar::new("id_boundary".to_string());

    // Test at maximum u16 value
    let max_id = SymbolId(u16::MAX);
    grammar.rule_names.insert(max_id, "max_id".to_string());

    // Test at 0
    let min_id = SymbolId(0);
    grammar.rule_names.insert(min_id, "min_id".to_string());

    assert_eq!(grammar.rule_names.get(&max_id), Some(&"max_id".to_string()));
    assert_eq!(grammar.rule_names.get(&min_id), Some(&"min_id".to_string()));
}

// ============================================================================
// Test 16: Rules with empty pattern strings
// ============================================================================
#[test]
fn test_rules_with_empty_pattern_strings() {
    let mut grammar = Grammar::new("empty_patterns".to_string());

    let token_id = SymbolId(1);
    let token = Token {
        name: "empty_token".to_string(),
        pattern: TokenPattern::String(String::new()),
        fragile: false,
    };

    grammar.tokens.insert(token_id, token);

    // Can create tokens with empty patterns (validation might catch this)
    assert!(grammar.tokens.contains_key(&token_id));
    assert_eq!(
        grammar.tokens.get(&token_id).unwrap().pattern,
        TokenPattern::String(String::new())
    );
}

// ============================================================================
// Test 17: Grammar with all fields of AliasSequence populated
// ============================================================================
#[test]
fn test_alias_sequence_fully_populated() {
    let mut grammar = Grammar::new("full_aliases".to_string());

    // Create an AliasSequence with many populated fields
    let alias_seq = AliasSequence {
        aliases: vec![
            Some("alias_0".to_string()),
            Some("alias_1".to_string()),
            Some("alias_2".to_string()),
            None,
            Some("alias_4".to_string()),
            Some("alias_5".to_string()),
        ],
    };

    grammar
        .alias_sequences
        .insert(ProductionId(1), alias_seq.clone());

    let retrieved = grammar.alias_sequences.get(&ProductionId(1)).unwrap();
    assert_eq!(retrieved.aliases.len(), 6);
    assert_eq!(retrieved.aliases[0], Some("alias_0".to_string()));
    assert_eq!(retrieved.aliases[3], None);
}

// ============================================================================
// Test 18: Grammar with ConflictDeclaration containing duplicates
// ============================================================================
#[test]
fn test_conflict_declaration_with_duplicates() {
    let mut grammar = Grammar::new("conflict_dupes".to_string());

    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(1), SymbolId(2), SymbolId(1)],
        resolution: ConflictResolution::GLR,
    };

    grammar.conflicts.push(conflict);

    assert_eq!(grammar.conflicts.len(), 1);
    assert_eq!(grammar.conflicts[0].symbols.len(), 4);
    // Note: duplicates are stored as-is (deduplication may happen elsewhere)
}

// ============================================================================
// Test 19: ExternalToken with all fields populated
// ============================================================================
#[test]
fn test_external_token_fully_populated() {
    let mut grammar = Grammar::new("external_test".to_string());

    let external = ExternalToken {
        name: "external_scanner".to_string(),
        symbol_id: SymbolId(42),
    };

    grammar.externals.push(external.clone());

    assert_eq!(grammar.externals.len(), 1);
    assert_eq!(grammar.externals[0].name, "external_scanner");
    assert_eq!(grammar.externals[0].symbol_id, SymbolId(42));
}

// ============================================================================
// Test 20: Empty grammar (all fields empty/default)
// ============================================================================
#[test]
fn test_completely_empty_grammar() {
    let grammar = Grammar::default();

    assert_eq!(grammar.name, "");
    assert!(grammar.rules.is_empty());
    assert!(grammar.tokens.is_empty());
    assert!(grammar.precedences.is_empty());
    assert!(grammar.conflicts.is_empty());
    assert!(grammar.externals.is_empty());
    assert!(grammar.extras.is_empty());
    assert!(grammar.fields.is_empty());
    assert!(grammar.supertypes.is_empty());
    assert!(grammar.inline_rules.is_empty());
    assert!(grammar.alias_sequences.is_empty());
    assert!(grammar.production_ids.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
    assert!(grammar.rule_names.is_empty());
    assert!(grammar.symbol_registry.is_none());
}

// ============================================================================
// Test 21: Grammar normalization on already-normalized grammar
// ============================================================================
#[test]
fn test_normalization_on_normalized_grammar() {
    let mut grammar = Grammar::new("normalized".to_string());

    // Create a simple rule with no complex symbols (already normalized)
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(3)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.add_rule(rule.clone());

    let initial_rule_count = grammar.all_rules().count();
    let _ = grammar.normalize();
    let final_rule_count = grammar.all_rules().count();

    // Already-normalized grammar may not change or may add epsilon rules
    assert!(final_rule_count >= initial_rule_count);
}

// ============================================================================
// Test 22: Grammar validation on valid grammars
// ============================================================================
#[test]
fn test_validation_on_valid_grammar() {
    let mut grammar = Grammar::new("valid".to_string());

    let a_id = SymbolId(1);
    let b_id = SymbolId(2);

    // Create tokens that are referenced
    grammar.tokens.insert(
        b_id,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    // Create a valid rule A -> B
    let rule = Rule {
        lhs: a_id,
        rhs: vec![Symbol::Terminal(b_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.add_rule(rule);
    grammar.rule_names.insert(a_id, "A".to_string());

    // Should validate successfully
    assert!(grammar.validate().is_ok());
}

// ============================================================================
// Test 23: Grammar validation on invalid grammars (missing rules)
// ============================================================================
#[test]
fn test_validation_on_invalid_grammar() {
    let mut grammar = Grammar::new("invalid".to_string());

    let a_id = SymbolId(1);
    let b_id = SymbolId(2); // This symbol is referenced but doesn't exist

    // Rule references undefined symbol B
    let rule = Rule {
        lhs: a_id,
        rhs: vec![Symbol::NonTerminal(b_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.add_rule(rule);
    grammar.rule_names.insert(a_id, "A".to_string());

    // Should fail validation - b_id is not defined
    assert!(grammar.validate().is_err());
}

// ============================================================================
// Test 24: Optimizer on trivially simple grammar
// ============================================================================
#[test]
fn test_optimizer_on_simple_grammar() {
    let mut grammar = Grammar::new("simple".to_string());

    // Single simple rule
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.add_rule(rule);

    let initial_rules = grammar.all_rules().count();
    grammar.optimize();
    let final_rules = grammar.all_rules().count();

    // Optimizer shouldn't break simple grammars
    assert_eq!(initial_rules, final_rules);
}

// ============================================================================
// Test 25: Optimizer on grammar with no optimization opportunities
// ============================================================================
#[test]
fn test_optimizer_on_no_optimization_grammar() {
    let mut grammar = Grammar::new("no_opt".to_string());

    // Create multiple diverse rules
    for i in 0..10 {
        let rule = Rule {
            lhs: SymbolId(i),
            rhs: vec![
                Symbol::Terminal(SymbolId(i + 100)),
                Symbol::NonTerminal(SymbolId(i + 200)),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        };
        grammar.add_rule(rule);
    }

    let initial_rules = grammar.all_rules().count();
    grammar.optimize();
    let final_rules = grammar.all_rules().count();

    // Should preserve rule count
    assert_eq!(initial_rules, final_rules);
}

// ============================================================================
// Test 26: Symbol registry with sequential IDs
// ============================================================================
#[test]
fn test_symbol_registry_sequential_ids() {
    let mut grammar = Grammar::new("seq_ids".to_string());

    // Add symbols with sequential IDs
    for i in 0..100 {
        grammar
            .rule_names
            .insert(SymbolId(i as u16), format!("symbol_{}", i));
    }

    assert_eq!(grammar.rule_names.len(), 100);

    // Verify all are present
    for i in 0..100 {
        assert_eq!(
            grammar.rule_names.get(&SymbolId(i as u16)),
            Some(&format!("symbol_{}", i))
        );
    }
}

// ============================================================================
// Test 27: Symbol registry with gaps in IDs
// ============================================================================
#[test]
fn test_symbol_registry_with_gaps() {
    let mut grammar = Grammar::new("gap_ids".to_string());

    // Add symbols with gaps in IDs
    let ids = vec![0, 5, 10, 20, 50, 100];
    for id in ids.iter() {
        grammar
            .rule_names
            .insert(SymbolId(*id as u16), format!("symbol_{}", id));
    }

    assert_eq!(grammar.rule_names.len(), 6);

    // Verify sparse IDs are stored correctly
    for id in ids {
        assert_eq!(
            grammar.rule_names.get(&SymbolId(id as u16)),
            Some(&format!("symbol_{}", id))
        );
    }
}

// ============================================================================
// Test 28: Grammar clone preserves all fields
// ============================================================================
#[test]
fn test_grammar_clone_preserves_fields() {
    let mut original = Grammar::new("clone_test".to_string());

    // Populate all fields
    original.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );

    original.tokens.insert(
        SymbolId(2),
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );

    original.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });

    original.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::GLR,
    });

    original.externals.push(ExternalToken {
        name: "ext".to_string(),
        symbol_id: SymbolId(3),
    });

    original.extras.push(SymbolId(4));

    let mut fields = IndexMap::new();
    fields.insert(FieldId(1), "field1".to_string());
    original.fields = fields;

    original.supertypes.push(SymbolId(5));
    original.inline_rules.push(SymbolId(6));

    original.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![Some("alias".to_string())],
        },
    );

    original
        .production_ids
        .insert(adze_ir::RuleId(1), ProductionId(1));

    original.max_alias_sequence_length = 100;
    original.rule_names.insert(SymbolId(1), "rule1".to_string());

    // Clone the grammar
    let cloned = original.clone();

    // Verify all fields are preserved
    assert_eq!(cloned.name, original.name);
    assert_eq!(cloned.rules, original.rules);
    assert_eq!(cloned.tokens, original.tokens);
    assert_eq!(cloned.precedences, original.precedences);
    assert_eq!(cloned.conflicts, original.conflicts);
    assert_eq!(cloned.externals, original.externals);
    assert_eq!(cloned.extras, original.extras);
    assert_eq!(cloned.fields, original.fields);
    assert_eq!(cloned.supertypes, original.supertypes);
    assert_eq!(cloned.inline_rules, original.inline_rules);
    assert_eq!(cloned.alias_sequences, original.alias_sequences);
    assert_eq!(cloned.production_ids, original.production_ids);
    assert_eq!(
        cloned.max_alias_sequence_length,
        original.max_alias_sequence_length
    );
    assert_eq!(cloned.rule_names, original.rule_names);
}

// ============================================================================
// Test 29: Grammar serialization roundtrip with extreme values
// ============================================================================
#[test]
fn test_serialization_roundtrip_extreme_values() {
    let mut original = Grammar::new("extreme".to_string());

    // Add rules with extreme precedence values
    original.precedences.push(Precedence {
        level: i16::MIN,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(0), SymbolId(u16::MAX)],
    });

    original.precedences.push(Precedence {
        level: i16::MAX,
        associativity: Associativity::Right,
        symbols: vec![],
    });

    // Add conflict with extreme symbol IDs
    original.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(0), SymbolId(u16::MAX / 2)],
        resolution: ConflictResolution::GLR,
    });

    // Serialize
    let serialized = serde_json::to_string(&original).expect("serialization failed");

    // Deserialize
    let deserialized: Grammar = serde_json::from_str(&serialized).expect("deserialization failed");

    // Verify roundtrip
    assert_eq!(deserialized.name, original.name);
    assert_eq!(deserialized.precedences, original.precedences);
    assert_eq!(deserialized.conflicts, original.conflicts);
}

// ============================================================================
// Test 30: Grammar with conflicting precedence declarations
// ============================================================================
#[test]
fn test_grammar_with_conflicting_precedences() {
    let mut grammar = Grammar::new("conflict_prec".to_string());

    let symbol = SymbolId(1);

    // Add the same symbol with different precedence levels
    grammar.precedences.push(Precedence {
        level: 10,
        associativity: Associativity::Left,
        symbols: vec![symbol],
    });

    grammar.precedences.push(Precedence {
        level: 20,
        associativity: Associativity::Right,
        symbols: vec![symbol],
    });

    // Grammar structure allows storing conflicting precedences
    // (resolution/validation may happen elsewhere)
    assert_eq!(grammar.precedences.len(), 2);
    assert_eq!(grammar.precedences[0].level, 10);
    assert_eq!(grammar.precedences[1].level, 20);
}

// ============================================================================
// Test 31: Complex nested symbol with all wrapper types
// ============================================================================
#[test]
fn test_complex_nested_symbol_all_wrappers() {
    let symbol = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::RepeatOne(
        Box::new(Symbol::Choice(vec![
            Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::NonTerminal(SymbolId(2)),
            ]),
            Symbol::External(SymbolId(3)),
        ])),
    )))));

    // Verify structure compiles and is constructible
    assert!(matches!(symbol, Symbol::Optional(_)));
}

// ============================================================================
// Test 32: Grammar with all Associativity variants
// ============================================================================
#[test]
fn test_grammar_with_all_associativity_variants() {
    let mut grammar = Grammar::new("assoc".to_string());

    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });

    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)],
    });

    grammar.precedences.push(Precedence {
        level: 3,
        associativity: Associativity::None,
        symbols: vec![SymbolId(3)],
    });

    assert_eq!(grammar.precedences.len(), 3);
    assert_eq!(grammar.precedences[0].associativity, Associativity::Left);
    assert_eq!(grammar.precedences[1].associativity, Associativity::Right);
    assert_eq!(grammar.precedences[2].associativity, Associativity::None);
}

// ============================================================================
// Test 33: Grammar field ordering validation
// ============================================================================
#[test]
fn test_grammar_field_ordering() {
    let mut grammar = Grammar::new("field_order".to_string());

    // Insert fields in correct lexicographic order
    let mut fields = IndexMap::new();
    fields.insert(FieldId(0), "aaa".to_string());
    fields.insert(FieldId(1), "bbb".to_string());
    fields.insert(FieldId(2), "ccc".to_string());

    grammar.fields = fields;

    // Should validate successfully due to correct ordering
    // (This test verifies the field ordering requirement)
    let field_names: Vec<_> = grammar.fields.values().collect();
    let mut sorted = field_names.clone();
    sorted.sort();
    assert_eq!(field_names, sorted);
}

// ============================================================================
// Test 34: Token with regex pattern boundary
// ============================================================================
#[test]
fn test_token_with_regex_pattern() {
    let mut grammar = Grammar::new("regex_token".to_string());

    let complex_regex = r"[a-zA-Z_][a-zA-Z0-9_]*";
    let token = Token {
        name: "identifier".to_string(),
        pattern: TokenPattern::Regex(complex_regex.to_string()),
        fragile: true,
    };

    grammar.tokens.insert(SymbolId(1), token);

    let retrieved = grammar.tokens.get(&SymbolId(1)).unwrap();
    assert_eq!(retrieved.name, "identifier");
    assert!(matches!(retrieved.pattern, TokenPattern::Regex(_)));
    assert!(retrieved.fragile);
}

// ============================================================================
// Test 35: Grammar with all conflict resolution types
// ============================================================================
#[test]
fn test_conflict_resolution_all_types() {
    let mut grammar = Grammar::new("conflict_types".to_string());

    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(10)),
    });

    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3), SymbolId(4)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(5)),
    });

    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(5), SymbolId(6)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });

    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(7), SymbolId(8)],
        resolution: ConflictResolution::GLR,
    });

    assert_eq!(grammar.conflicts.len(), 4);

    match &grammar.conflicts[0].resolution {
        ConflictResolution::Precedence(PrecedenceKind::Static(n)) => assert_eq!(*n, 10),
        _ => panic!("Expected Static precedence"),
    }

    match &grammar.conflicts[1].resolution {
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(n)) => assert_eq!(*n, 5),
        _ => panic!("Expected Dynamic precedence"),
    }

    match &grammar.conflicts[2].resolution {
        ConflictResolution::Associativity(Associativity::Left) => {}
        _ => panic!("Expected Associativity::Left"),
    }

    assert!(matches!(
        grammar.conflicts[3].resolution,
        ConflictResolution::GLR
    ));
}

// ============================================================================
// Test 36: Grammar find_symbol_by_name functionality
// ============================================================================
#[test]
fn test_find_symbol_by_name() {
    let mut grammar = Grammar::new("find_test".to_string());

    grammar
        .rule_names
        .insert(SymbolId(1), "expression".to_string());
    grammar
        .rule_names
        .insert(SymbolId(2), "statement".to_string());
    grammar
        .rule_names
        .insert(SymbolId(3), "program".to_string());

    assert_eq!(grammar.find_symbol_by_name("expression"), Some(SymbolId(1)));
    assert_eq!(grammar.find_symbol_by_name("statement"), Some(SymbolId(2)));
    assert_eq!(grammar.find_symbol_by_name("program"), Some(SymbolId(3)));
    assert_eq!(grammar.find_symbol_by_name("nonexistent"), None);
}

// ============================================================================
// Test 37: Grammar get_rules_for_symbol
// ============================================================================
#[test]
fn test_get_rules_for_symbol() {
    let mut grammar = Grammar::new("rules_test".to_string());

    let lhs = SymbolId(1);

    // Add multiple rules with the same LHS
    for i in 0..5 {
        let rule = Rule {
            lhs,
            rhs: vec![Symbol::Terminal(SymbolId((i + 2) as u16))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        };
        grammar.add_rule(rule);
    }

    let rules = grammar.get_rules_for_symbol(lhs).unwrap();
    assert_eq!(rules.len(), 5);

    // Non-existent LHS should return None
    assert_eq!(grammar.get_rules_for_symbol(SymbolId(999)), None);
}

// ============================================================================
// Test 38: Grammar all_rules iterator
// ============================================================================
#[test]
fn test_all_rules_iterator() {
    let mut grammar = Grammar::new("all_rules_test".to_string());

    let mut total_rules = 0;
    for i in 0..10 {
        let lhs = SymbolId(i as u16);
        for j in 0..3 {
            let rule = Rule {
                lhs,
                rhs: vec![Symbol::Terminal(SymbolId((i * 10 + j) as u16))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId((i * 10 + j) as u16),
            };
            grammar.add_rule(rule);
            total_rules += 1;
        }
    }

    assert_eq!(grammar.all_rules().count(), total_rules);
}

// ============================================================================
// Test 39: Grammar equality and PartialEq
// ============================================================================
#[test]
fn test_grammar_equality() {
    let mut grammar1 = Grammar::new("eq_test".to_string());
    let mut grammar2 = Grammar::new("eq_test".to_string());

    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar1.add_rule(rule.clone());
    grammar2.add_rule(rule);

    assert_eq!(grammar1, grammar2);
}

// ============================================================================
// Test 40: Symbol enum equality for all variants
// ============================================================================
#[test]
fn test_symbol_equality_all_variants() {
    let sym1 = Symbol::Terminal(SymbolId(1));
    let sym2 = Symbol::Terminal(SymbolId(1));
    assert_eq!(sym1, sym2);

    let sym3 = Symbol::Terminal(SymbolId(2));
    assert_ne!(sym1, sym3);

    let optional1 = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let optional2 = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    assert_eq!(optional1, optional2);

    let choice1 = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let choice2 = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    assert_eq!(choice1, choice2);
}
