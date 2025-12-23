// Tests for grammar normalization functionality

use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

#[test]
fn test_normalize_optional_symbol() {
    let mut grammar = create_test_grammar_with_optional();

    // Before normalization, should have Optional symbol
    let original_rule = &grammar.rules[&SymbolId(1)][0];
    assert!(matches!(original_rule.rhs[0], Symbol::Optional(_)));

    // Normalize
    grammar.normalize();

    // After normalization, Optional should be replaced with auxiliary non-terminal
    let normalized_rule = &grammar.rules[&SymbolId(1)][0];
    assert!(matches!(normalized_rule.rhs[0], Symbol::NonTerminal(_)));

    // Should have created auxiliary rules for the optional symbol
    let aux_symbol_id = match &normalized_rule.rhs[0] {
        Symbol::NonTerminal(id) => *id,
        _ => panic!("Expected NonTerminal"),
    };

    // Auxiliary rules should exist
    assert!(grammar.rules.contains_key(&aux_symbol_id));
    let aux_rules = &grammar.rules[&aux_symbol_id];

    // Should have two rules: aux -> inner, aux -> ε
    assert_eq!(aux_rules.len(), 2);

    // Check the rules
    let has_inner_rule = aux_rules
        .iter()
        .any(|rule| rule.rhs.len() == 1 && matches!(rule.rhs[0], Symbol::Terminal(SymbolId(2))));
    let has_epsilon_rule = aux_rules
        .iter()
        .any(|rule| rule.rhs.len() == 1 && matches!(rule.rhs[0], Symbol::Epsilon));

    assert!(has_inner_rule, "Should have aux -> inner rule");
    assert!(has_epsilon_rule, "Should have aux -> epsilon rule");
}

#[test]
fn test_normalize_repeat_symbol() {
    let mut grammar = create_test_grammar_with_repeat();

    // Normalize
    grammar.normalize();

    // Check that Repeat was replaced with auxiliary non-terminal
    let normalized_rule = &grammar.rules[&SymbolId(1)][0];
    let aux_symbol_id = match &normalized_rule.rhs[0] {
        Symbol::NonTerminal(id) => *id,
        _ => panic!("Expected NonTerminal after normalization"),
    };

    // Auxiliary rules should exist
    assert!(grammar.rules.contains_key(&aux_symbol_id));
    let aux_rules = &grammar.rules[&aux_symbol_id];

    // Should have two rules: aux -> aux inner, aux -> ε
    assert_eq!(aux_rules.len(), 2);

    // Check for recursive rule: aux -> aux inner
    let has_recursive_rule = aux_rules.iter().any(|rule| {
        rule.rhs.len() == 2
            && matches!(rule.rhs[0], Symbol::NonTerminal(id) if id == aux_symbol_id)
            && matches!(rule.rhs[1], Symbol::Terminal(SymbolId(2)))
    });

    // Check for epsilon rule: aux -> ε
    let has_epsilon_rule = aux_rules
        .iter()
        .any(|rule| rule.rhs.len() == 1 && matches!(rule.rhs[0], Symbol::Epsilon));

    assert!(has_recursive_rule, "Should have aux -> aux inner rule");
    assert!(has_epsilon_rule, "Should have aux -> epsilon rule");
}

#[test]
fn test_normalize_sequence_symbol() {
    let mut grammar = create_test_grammar_with_sequence();

    // Normalize
    grammar.normalize();

    // Check that Sequence was handled correctly (should create aux rule for multi-element sequences)
    let normalized_rule = &grammar.rules[&SymbolId(1)][0];

    // For a sequence with multiple elements, should create auxiliary rule
    assert!(!normalized_rule.rhs.is_empty());

    // The sequence should be normalized into the rule
    if normalized_rule.rhs.len() == 1 {
        // Single auxiliary symbol created
        let aux_symbol_id = match &normalized_rule.rhs[0] {
            Symbol::NonTerminal(id) => *id,
            _ => panic!("Expected NonTerminal"),
        };

        // Should have auxiliary rule with the sequence
        assert!(grammar.rules.contains_key(&aux_symbol_id));
        let aux_rules = &grammar.rules[&aux_symbol_id];
        assert_eq!(aux_rules.len(), 1);
        assert_eq!(aux_rules[0].rhs.len(), 2); // The original sequence elements
    } else {
        // Sequence was flattened directly into the rule
        assert_eq!(normalized_rule.rhs.len(), 2);
        assert!(matches!(
            normalized_rule.rhs[0],
            Symbol::Terminal(SymbolId(2))
        ));
        assert!(matches!(
            normalized_rule.rhs[1],
            Symbol::Terminal(SymbolId(3))
        ));
    }
}

#[test]
fn test_normalize_nested_complex_symbols() {
    let mut grammar = create_test_grammar_with_nested_symbols();

    // Normalize
    grammar.normalize();

    // Should have created auxiliary rules for the nested structures
    let normalized_rule = &grammar.rules[&SymbolId(1)][0];
    assert!(matches!(normalized_rule.rhs[0], Symbol::NonTerminal(_)));

    // Check that all complex symbols have been normalized
    for (_, rules) in &grammar.rules {
        for rule in rules {
            for symbol in &rule.rhs {
                assert!(
                    matches!(
                        symbol,
                        Symbol::Terminal(_)
                            | Symbol::NonTerminal(_)
                            | Symbol::External(_)
                            | Symbol::Epsilon
                    ),
                    "Found non-normalized symbol: {:?}",
                    symbol
                );
            }
        }
    }
}

#[test]
fn test_normalize_preserves_existing_rules() {
    let mut grammar = create_test_grammar_mixed();
    let original_simple_rules_count = grammar.rules[&SymbolId(2)].len();

    // Normalize
    grammar.normalize();

    // Simple rules should be preserved unchanged
    let preserved_rules_count = grammar.rules[&SymbolId(2)].len();
    assert_eq!(original_simple_rules_count, preserved_rules_count);

    // Check that simple rule content is unchanged
    let simple_rule = &grammar.rules[&SymbolId(2)][0];
    assert_eq!(simple_rule.rhs.len(), 1);
    assert!(matches!(simple_rule.rhs[0], Symbol::Terminal(SymbolId(3))));
}

#[test]
fn test_normalize_idempotent() {
    let mut grammar1 = create_test_grammar_with_optional();
    let mut grammar2 = grammar1.clone();

    // Normalize both grammars
    grammar1.normalize();
    grammar2.normalize();
    grammar2.normalize(); // Second normalization should do nothing

    // Should be equivalent (both should have same structure)
    assert_eq!(grammar1.rules.len(), grammar2.rules.len());

    // All symbols should be simple after normalization
    for (_, rules) in &grammar2.rules {
        for rule in rules {
            for symbol in &rule.rhs {
                assert!(
                    matches!(
                        symbol,
                        Symbol::Terminal(_)
                            | Symbol::NonTerminal(_)
                            | Symbol::External(_)
                            | Symbol::Epsilon
                    ),
                    "Found complex symbol after double normalization: {:?}",
                    symbol
                );
            }
        }
    }
}

// Helper functions to create test grammars

fn create_test_grammar_with_optional() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Add tokens
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "token".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Add rule with optional symbol: rule -> token?
    grammar.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(2))))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    grammar.rule_names.insert(SymbolId(1), "rule".to_string());

    grammar
}

fn create_test_grammar_with_repeat() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Add tokens
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "token".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Add rule with repeat symbol: rule -> token*
    grammar.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2))))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    grammar.rule_names.insert(SymbolId(1), "rule".to_string());

    grammar
}

fn create_test_grammar_with_sequence() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Add tokens
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "token1".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "token2".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    // Add rule with sequence: rule -> (token1 token2)
    grammar.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(2)),
                Symbol::Terminal(SymbolId(3)),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    grammar.rule_names.insert(SymbolId(1), "rule".to_string());

    grammar
}

fn create_test_grammar_with_nested_symbols() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Add tokens
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "token".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Add rule with nested complex symbols: rule -> (token*)?
    grammar.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
                Symbol::Terminal(SymbolId(2)),
            ))))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    grammar.rule_names.insert(SymbolId(1), "rule".to_string());

    grammar
}

fn create_test_grammar_mixed() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Add tokens
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "token".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(4),
        Token {
            name: "token2".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    // Add rule with complex symbol
    grammar.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(3))))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    // Add rule with simple symbol (should be preserved)
    grammar.rules.insert(
        SymbolId(2),
        vec![Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(3))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );

    grammar.rule_names.insert(SymbolId(1), "rule1".to_string());
    grammar.rule_names.insert(SymbolId(2), "rule2".to_string());

    grammar
}
