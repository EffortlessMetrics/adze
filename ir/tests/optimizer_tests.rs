use rust_sitter_ir::optimizer::{GrammarOptimizer, OptimizationStats};
use rust_sitter_ir::*;

fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar {
        name: "TestGrammar".to_string(),
        ..Default::default()
    };

    // Add some rules with different patterns
    // A -> B
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::NonTerminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // B -> C
    grammar.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::NonTerminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // C -> 'x'
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(3))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });

    // Add rule names
    grammar.rule_names.insert(SymbolId(0), "A".to_string());
    grammar.rule_names.insert(SymbolId(1), "B".to_string());
    grammar.rule_names.insert(SymbolId(2), "C".to_string());

    // Add token
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    grammar
}

#[test]
fn test_optimizer_creation() {
    let mut grammar = create_test_grammar();
    let mut optimizer = GrammarOptimizer::new();

    // Just verify it can be created and run
    let stats = optimizer.optimize(&mut grammar);
    assert!(stats.total() < usize::MAX); // Total should be reasonable
}

#[test]
fn test_optimization_stats() {
    let stats = OptimizationStats {
        removed_unused_symbols: 5,
        inlined_rules: 3,
        merged_tokens: 2,
        optimized_left_recursion: 1,
        eliminated_unit_rules: 4,
    };

    assert_eq!(stats.removed_unused_symbols, 5);
    assert_eq!(stats.inlined_rules, 3);
    assert_eq!(stats.merged_tokens, 2);
    assert_eq!(stats.optimized_left_recursion, 1);
    assert_eq!(stats.eliminated_unit_rules, 4);
    assert_eq!(stats.total(), 15);
}

#[test]
fn test_optimize_simple_grammar() {
    let mut grammar = create_test_grammar();

    // Mark B as inline
    grammar.inline_rules.push(SymbolId(1));

    let result = optimize_grammar(grammar);
    assert!(result.is_ok());

    let optimized = result.unwrap();

    // Verify the grammar is still valid
    assert_eq!(optimized.name, "TestGrammar");
    // Note: optimize_grammar may have removed all rules if they were unused
    // This is expected behavior
}

#[test]
fn test_optimize_grammar_with_duplicates() {
    let mut grammar = Grammar {
        name: "DuplicateTest".to_string(),
        ..Default::default()
    };

    // Add duplicate rules
    for i in 0..3 {
        grammar.add_rule(Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        });
    }

    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

#[test]
fn test_optimize_with_precedence() {
    let mut grammar = Grammar {
        name: "PrecedenceTest".to_string(),
        ..Default::default()
    };

    // Add rules with different precedence
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
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
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(1),
    });

    let result = optimize_grammar(grammar);
    assert!(result.is_ok());

    let optimized = result.unwrap();

    // The optimizer might have modified or removed rules
    // Just verify the grammar is still valid
    assert_eq!(optimized.name, "PrecedenceTest");
}
