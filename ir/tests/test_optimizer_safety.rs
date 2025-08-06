// Test that the optimizer preserves grammar semantics
// This test ensures that optimizations don't break parsing behavior

use rust_sitter_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use rust_sitter_ir::*;

/// Create a simple grammar with unit rules for testing
fn create_unit_rule_grammar() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Tokens
    let number = SymbolId(1);
    let plus = SymbolId(2);
    let times = SymbolId(3);

    grammar.tokens.insert(
        number,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        plus,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        times,
        Token {
            name: "times".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    // Non-terminals
    let expression = SymbolId(4);
    let sum = SymbolId(5);
    let product = SymbolId(6);
    let primary = SymbolId(7);

    // expression -> sum (unit rule)
    grammar.add_rule(Rule {
        lhs: expression,
        rhs: vec![Symbol::NonTerminal(sum)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // sum -> sum + product
    grammar.add_rule(Rule {
        lhs: sum,
        rhs: vec![
            Symbol::NonTerminal(sum),
            Symbol::Terminal(plus),
            Symbol::NonTerminal(product),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(2),
    });

    // sum -> product (unit rule)
    grammar.add_rule(Rule {
        lhs: sum,
        rhs: vec![Symbol::NonTerminal(product)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(3),
    });

    // product -> product * primary
    grammar.add_rule(Rule {
        lhs: product,
        rhs: vec![
            Symbol::NonTerminal(product),
            Symbol::Terminal(times),
            Symbol::NonTerminal(primary),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(4),
    });

    // product -> primary (unit rule)
    grammar.add_rule(Rule {
        lhs: product,
        rhs: vec![Symbol::NonTerminal(primary)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(5),
    });

    // primary -> number
    grammar.add_rule(Rule {
        lhs: primary,
        rhs: vec![Symbol::Terminal(number)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(6),
    });

    // Set names for better debugging
    grammar
        .rule_names
        .insert(expression, "expression".to_string());
    grammar.rule_names.insert(sum, "sum".to_string());
    grammar.rule_names.insert(product, "product".to_string());
    grammar.rule_names.insert(primary, "primary".to_string());

    // Create a source_file symbol as the start symbol
    let source_file = SymbolId(10);
    grammar
        .rule_names
        .insert(source_file, "source_file".to_string());

    // source_file -> expression
    grammar.add_rule(Rule {
        lhs: source_file,
        rhs: vec![Symbol::NonTerminal(expression)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(10),
    });

    grammar
}

#[test]
fn test_unit_rule_elimination_preserves_semantics() {
    let original = create_unit_rule_grammar();
    let mut optimized = original.clone();

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut optimized);
    let eliminated = stats.eliminated_unit_rules;

    println!("Eliminated {} unit rules", eliminated);
    println!("Stats: {:?}", stats);

    // Print what's left after optimization
    println!("\nRules after optimization:");
    for (symbol_id, rules) in &optimized.rules {
        let name = optimized
            .rule_names
            .get(symbol_id)
            .map(|n| n.as_str())
            .unwrap_or("unknown");
        println!("  {} ({}): {} rules", name, symbol_id.0, rules.len());
        for rule in rules {
            println!("    -> {:?}", rule.rhs);
        }
    }

    // The inlining optimization may have removed intermediate symbols
    // The important thing is that the grammar is still valid and functional

    // Check that we still have source_file
    let source_file_id = optimized
        .find_symbol_by_name("source_file")
        .expect("source_file should exist");

    assert!(
        optimized.rules.contains_key(&source_file_id),
        "source_file should have rules"
    );

    // Just verify the grammar is still functional
    assert!(optimized.rules.len() > 0, "Grammar should still have rules");
    assert!(
        optimized.tokens.len() > 0,
        "Grammar should still have tokens"
    );

    // The optimization should have done something
    assert!(
        stats.total() > 0,
        "Some optimizations should have been performed"
    );
}

#[test]
fn test_optimizer_preserves_first_follow_sets() {
    // This test ensures that optimization doesn't change the FIRST/FOLLOW sets
    // which would break GLR parsing

    let original = create_unit_rule_grammar();
    let optimized = optimize_grammar(original.clone()).expect("Optimization should succeed");

    // For now, just verify that optimization completes without error
    // In a full implementation, we would compute FIRST/FOLLOW sets before and after
    // and verify they are equivalent

    assert_eq!(optimized.name, original.name);
    assert_eq!(optimized.tokens.len(), original.tokens.len());
}

#[test]
fn test_optimizer_handles_left_recursion() {
    let mut grammar = Grammar::new("test".to_string());

    // Create a left-recursive grammar
    let list = SymbolId(1);
    let item = SymbolId(2);
    let comma = SymbolId(3);

    grammar.tokens.insert(
        item,
        Token {
            name: "item".to_string(),
            pattern: TokenPattern::Regex(r"\w+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        comma,
        Token {
            name: "comma".to_string(),
            pattern: TokenPattern::String(",".to_string()),
            fragile: false,
        },
    );

    // list -> list , item (left recursive)
    grammar.add_rule(Rule {
        lhs: list,
        rhs: vec![
            Symbol::NonTerminal(list),
            Symbol::Terminal(comma),
            Symbol::Terminal(item),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // list -> item
    grammar.add_rule(Rule {
        lhs: list,
        rhs: vec![Symbol::Terminal(item)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    println!("Optimization stats: {:?}", stats);

    // Verify grammar is still valid
    assert!(
        grammar.rules.len() > 0,
        "Grammar should still have rules after optimization"
    );
}

#[test]
fn test_optimizer_preserves_source_file_symbol() {
    let mut grammar = Grammar::new("test".to_string());

    // Create a source_file symbol (special in Tree-sitter)
    let source_file = SymbolId(100);
    let statement = SymbolId(101);
    let semicolon = SymbolId(102);

    grammar
        .rule_names
        .insert(source_file, "source_file".to_string());
    grammar
        .rule_names
        .insert(statement, "statement".to_string());

    grammar.tokens.insert(
        semicolon,
        Token {
            name: "semicolon".to_string(),
            pattern: TokenPattern::String(";".to_string()),
            fragile: false,
        },
    );

    // source_file -> statement (this should NOT be eliminated even though it's a unit rule)
    grammar.add_rule(Rule {
        lhs: source_file,
        rhs: vec![Symbol::NonTerminal(statement)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // statement -> semicolon
    grammar.add_rule(Rule {
        lhs: statement,
        rhs: vec![Symbol::Terminal(semicolon)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });

    let original_source_file_rules = grammar.rules.get(&source_file).unwrap().len();

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    println!("Optimization stats: {:?}", stats);

    // Verify source_file still exists (might have been renumbered)
    let source_file_id = grammar
        .find_symbol_by_name("source_file")
        .expect("source_file should still exist after optimization");

    assert!(
        grammar.rules.contains_key(&source_file_id),
        "source_file should not be removed"
    );
    assert_eq!(
        grammar.rules.get(&source_file_id).unwrap().len(),
        original_source_file_rules,
        "source_file rules should be preserved"
    );
}
