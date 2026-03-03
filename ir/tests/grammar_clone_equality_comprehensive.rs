#![allow(clippy::needless_range_loop)]

use adze_ir::*;
use indexmap::IndexMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar with one rule and one token.
fn minimal_grammar() -> Grammar {
    let mut g = Grammar::new("minimal".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(0), "expr".to_string());
    g
}

/// Build a fully-populated grammar with every field type set.
fn fully_populated_grammar() -> Grammar {
    let mut g = Grammar::new("full".to_string());

    // Tokens
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: true,
        },
    );

    // Rules
    g.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(10)),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // Precedences
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });

    // Conflicts
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(10), SymbolId(10)],
        resolution: ConflictResolution::GLR,
    });

    // Externals
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(100),
    });

    // Extras
    g.extras.push(SymbolId(200));

    // Fields (lexicographic order)
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "right".to_string());

    // Supertypes
    g.supertypes.push(SymbolId(10));

    // Inline rules
    g.inline_rules.push(SymbolId(10));

    // Alias sequences
    let mut aliases = IndexMap::new();
    aliases.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("add_expr".to_string()), None, None],
        },
    );
    g.alias_sequences = aliases;

    // Production IDs
    g.production_ids.insert(RuleId(0), ProductionId(0));
    g.production_ids.insert(RuleId(1), ProductionId(1));

    g.max_alias_sequence_length = 3;

    // Rule names
    g.rule_names.insert(SymbolId(10), "Expression".to_string());

    g
}

// ===========================================================================
// 1. Clone produces identical grammar
// ===========================================================================

#[test]
fn test_clone_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_default_grammar() {
    let g = Grammar::default();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_grammar_with_name() {
    let g = Grammar::new("my_lang".to_string());
    let cloned = g.clone();
    assert_eq!(cloned.name, "my_lang");
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_grammar_with_rules() {
    let g = minimal_grammar();
    let cloned = g.clone();
    assert_eq!(g.rules.len(), cloned.rules.len());
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_grammar_with_tokens() {
    let g = minimal_grammar();
    let cloned = g.clone();
    assert_eq!(g.tokens.len(), cloned.tokens.len());
    assert_eq!(
        g.tokens[&SymbolId(1)].name,
        cloned.tokens[&SymbolId(1)].name
    );
}

#[test]
fn test_clone_grammar_with_precedences() {
    let mut g = Grammar::new("prec".to_string());
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1), SymbolId(2)],
    });
    let cloned = g.clone();
    assert_eq!(g, cloned);
    assert_eq!(cloned.precedences[0].level, 5);
}

#[test]
fn test_clone_grammar_with_conflicts() {
    let mut g = Grammar::new("conflicts".to_string());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_grammar_with_externals() {
    let mut g = Grammar::new("ext".to_string());
    g.externals.push(ExternalToken {
        name: "newline".to_string(),
        symbol_id: SymbolId(50),
    });
    let cloned = g.clone();
    assert_eq!(g, cloned);
    assert_eq!(cloned.externals[0].name, "newline");
}

// ===========================================================================
// 2. Clone independence — mutating clone does not affect original
// ===========================================================================

#[test]
fn test_clone_independence_name() {
    let g = Grammar::new("original".to_string());
    let mut cloned = g.clone();
    cloned.name = "modified".to_string();
    assert_eq!(g.name, "original");
    assert_ne!(g, cloned);
}

#[test]
fn test_clone_independence_rules() {
    let g = minimal_grammar();
    let mut cloned = g.clone();
    cloned.add_rule(Rule {
        lhs: SymbolId(5),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(9),
    });
    assert_eq!(g.rules.len(), 1);
    assert_eq!(cloned.rules.len(), 2);
    assert_ne!(g, cloned);
}

#[test]
fn test_clone_independence_tokens() {
    let g = minimal_grammar();
    let mut cloned = g.clone();
    cloned.tokens.insert(
        SymbolId(99),
        Token {
            name: "EXTRA".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    assert_eq!(g.tokens.len(), 1);
    assert_eq!(cloned.tokens.len(), 2);
}

#[test]
fn test_clone_independence_fields() {
    let mut g = Grammar::new("f".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    let mut cloned = g.clone();
    cloned.fields.insert(FieldId(1), "beta".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(cloned.fields.len(), 2);
}

#[test]
fn test_clone_independence_precedences() {
    let mut g = Grammar::new("p".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![],
    });
    let mut cloned = g.clone();
    cloned.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![],
    });
    assert_eq!(g.precedences.len(), 1);
    assert_eq!(cloned.precedences.len(), 2);
}

#[test]
fn test_clone_independence_nested_symbol() {
    let mut g = Grammar::new("nested".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut cloned = g.clone();
    // Replace the rule's rhs in the clone
    let rules = cloned.rules.get_mut(&SymbolId(0)).unwrap();
    rules[0].rhs = vec![Symbol::Epsilon];

    // Original is unchanged
    let orig_rhs = &g.rules[&SymbolId(0)][0].rhs;
    assert!(matches!(&orig_rhs[0], Symbol::Optional(_)));
}

// ===========================================================================
// 3. Equality comparison — same/different structures
// ===========================================================================

#[test]
fn test_equality_identical_grammars_built_separately() {
    let g1 = minimal_grammar();
    let g2 = minimal_grammar();
    assert_eq!(g1, g2);
}

#[test]
fn test_equality_different_names() {
    let g1 = Grammar::new("alpha".to_string());
    let g2 = Grammar::new("beta".to_string());
    assert_ne!(g1, g2);
}

#[test]
fn test_equality_different_rules() {
    let g1 = minimal_grammar();
    let mut g2 = minimal_grammar();
    g2.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    assert_ne!(g1, g2);
}

#[test]
fn test_equality_different_tokens() {
    let g1 = minimal_grammar();
    let mut g2 = minimal_grammar();
    g2.tokens.get_mut(&SymbolId(1)).unwrap().fragile = true;
    assert_ne!(g1, g2);
}

// ===========================================================================
// 4. Deep nested grammar cloning
// ===========================================================================

#[test]
fn test_clone_deeply_nested_symbols() {
    let mut g = Grammar::new("deep".to_string());
    // Optional(Repeat(Choice(Terminal(1), NonTerminal(2))))
    let deep_sym = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ])))));
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(SymbolId(2), "nt".to_string());
    g.rules.insert(SymbolId(2), vec![]);
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![deep_sym],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let cloned = g.clone();
    assert_eq!(g, cloned);

    // Verify the deep structure survived
    let rhs = &cloned.rules[&SymbolId(0)][0].rhs[0];
    match rhs {
        Symbol::Optional(inner) => match inner.as_ref() {
            Symbol::Repeat(inner2) => match inner2.as_ref() {
                Symbol::Choice(choices) => {
                    assert_eq!(choices.len(), 2);
                    assert_eq!(choices[0], Symbol::Terminal(SymbolId(1)));
                    assert_eq!(choices[1], Symbol::NonTerminal(SymbolId(2)));
                }
                other => panic!("Expected Choice, got {:?}", other),
            },
            other => panic!("Expected Repeat, got {:?}", other),
        },
        other => panic!("Expected Optional, got {:?}", other),
    }
}

#[test]
fn test_clone_grammar_with_complex_rhs() {
    let mut g = Grammar::new("complex_rhs".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(1)))),
            ]),
            Symbol::Epsilon,
        ],
        precedence: Some(PrecedenceKind::Dynamic(3)),
        associativity: Some(Associativity::None),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    });

    let cloned = g.clone();
    assert_eq!(g, cloned);
}

// ===========================================================================
// 5. Grammar with all field types populated
// ===========================================================================

#[test]
fn test_clone_fully_populated_grammar() {
    let g = fully_populated_grammar();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_fully_populated_field_by_field() {
    let g = fully_populated_grammar();
    let cloned = g.clone();

    assert_eq!(g.name, cloned.name);
    assert_eq!(g.rules, cloned.rules);
    assert_eq!(g.tokens, cloned.tokens);
    assert_eq!(g.precedences, cloned.precedences);
    assert_eq!(g.conflicts, cloned.conflicts);
    assert_eq!(g.externals, cloned.externals);
    assert_eq!(g.extras, cloned.extras);
    assert_eq!(g.fields, cloned.fields);
    assert_eq!(g.supertypes, cloned.supertypes);
    assert_eq!(g.inline_rules, cloned.inline_rules);
    assert_eq!(g.alias_sequences, cloned.alias_sequences);
    assert_eq!(g.production_ids, cloned.production_ids);
    assert_eq!(
        g.max_alias_sequence_length,
        cloned.max_alias_sequence_length
    );
    assert_eq!(g.rule_names, cloned.rule_names);
}

// ===========================================================================
// 6. Clone after normalization
// ===========================================================================

#[test]
fn test_clone_after_normalization() {
    let mut g = Grammar::new("norm".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.normalize();

    let cloned = g.clone();
    assert_eq!(g, cloned);
    // After normalization, Optional should have been expanded into auxiliary rules
    assert!(g.rules.len() > 1);
}

#[test]
fn test_normalize_clone_independence() {
    let mut g = Grammar::new("norm_ind".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "Y".to_string(),
            pattern: TokenPattern::String("y".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.normalize();

    let mut cloned = g.clone();
    assert_eq!(g, cloned);

    // Mutate the clone
    cloned.name = "mutated".to_string();
    cloned.rules.clear();
    assert_ne!(g, cloned);
    // Original still has rules
    assert!(!g.rules.is_empty());
}

#[test]
fn test_clone_before_and_after_normalization() {
    let mut g = Grammar::new("ba".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "Z".to_string(),
            pattern: TokenPattern::String("z".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Epsilon,
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let before = g.clone();
    g.normalize();
    let after = g.clone();

    // Before and after normalization should differ (normalization expands complex symbols)
    assert_ne!(before, after);
    // But the after and its clone should match
    assert_eq!(g, after);
}

// ===========================================================================
// 7. PartialEq edge cases
// ===========================================================================

#[test]
fn test_partial_eq_reflexive() {
    let g = fully_populated_grammar();
    assert_eq!(g, g);
}

#[test]
fn test_partial_eq_symmetric() {
    let g1 = minimal_grammar();
    let g2 = minimal_grammar();
    assert_eq!(g1, g2);
    assert_eq!(g2, g1);
}

#[test]
fn test_partial_eq_transitive() {
    let g1 = minimal_grammar();
    let g2 = minimal_grammar();
    let g3 = minimal_grammar();
    assert_eq!(g1, g2);
    assert_eq!(g2, g3);
    assert_eq!(g1, g3);
}

#[test]
fn test_partial_eq_indexmap_order_independent() {
    // IndexMap PartialEq compares by key-value pairs regardless of
    // insertion order, so two maps with the same entries are equal.
    let mut g1 = Grammar::new("order".to_string());
    g1.rule_names.insert(SymbolId(1), "alpha".to_string());
    g1.rule_names.insert(SymbolId(2), "beta".to_string());

    let mut g2 = Grammar::new("order".to_string());
    g2.rule_names.insert(SymbolId(2), "beta".to_string());
    g2.rule_names.insert(SymbolId(1), "alpha".to_string());

    assert_eq!(g1, g2);
}

#[test]
fn test_partial_eq_empty_vs_populated() {
    let empty = Grammar::new("g".to_string());
    let populated = minimal_grammar();
    assert_ne!(empty, populated);
}

#[test]
fn test_partial_eq_max_alias_sequence_length_differs() {
    let mut g1 = Grammar::new("len".to_string());
    g1.max_alias_sequence_length = 0;
    let mut g2 = Grammar::new("len".to_string());
    g2.max_alias_sequence_length = 5;
    assert_ne!(g1, g2);
}

#[test]
fn test_partial_eq_symbol_registry_none_vs_some() {
    let mut g1 = Grammar::new("reg".to_string());
    g1.symbol_registry = None;
    let mut g2 = Grammar::new("reg".to_string());
    g2.symbol_registry = Some(SymbolRegistry::new());
    assert_ne!(g1, g2);
}

#[test]
fn test_partial_eq_conflict_resolution_variants() {
    let mut g1 = Grammar::new("cr".to_string());
    g1.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::GLR,
    });
    let mut g2 = Grammar::new("cr".to_string());
    g2.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });
    assert_ne!(g1, g2);
}

#[test]
fn test_partial_eq_token_pattern_variants() {
    let mut g1 = Grammar::new("tp".to_string());
    g1.tokens.insert(
        SymbolId(1),
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let mut g2 = Grammar::new("tp".to_string());
    g2.tokens.insert(
        SymbolId(1),
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::Regex("x".to_string()),
            fragile: false,
        },
    );
    assert_ne!(g1, g2);
}

#[test]
fn test_clone_eq_after_multiple_mutations() {
    let mut g = fully_populated_grammar();
    // Clone, mutate original, clone again
    let snapshot1 = g.clone();
    g.extras.push(SymbolId(999));
    g.max_alias_sequence_length = 42;
    let snapshot2 = g.clone();

    assert_ne!(snapshot1, snapshot2);
    assert_eq!(g, snapshot2);
    // snapshot1 retains old state
    assert_eq!(snapshot1.max_alias_sequence_length, 3);
    assert_eq!(snapshot2.max_alias_sequence_length, 42);
}
