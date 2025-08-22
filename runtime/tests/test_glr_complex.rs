// Complex GLR parser tests with real-world grammar patterns

use rust_sitter::glr_parser::GLRParser;
use rust_sitter_glr_core::{
    CompareResult, FirstFollowSets, VersionInfo, build_lr1_automaton, compare_versions_with_symbols,
};
use rust_sitter_ir::{
    Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
    Token, TokenPattern,
};

/// Build a grammar with ternary operator (nested precedence)
fn build_ternary_grammar() -> Grammar {
    let mut grammar = Grammar::new("ternary".to_string());

    // Tokens
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "question".to_string(),
            pattern: TokenPattern::String("?".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "colon".to_string(),
            pattern: TokenPattern::String(":".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(4),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    // Non-terminals
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expression".to_string());

    // Precedence (ternary is right-associative and lower than +)
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)], // ?
    });

    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(4)], // +
    });

    // Rules
    // expr -> identifier
    grammar
        .rules
        .entry(SymbolId(11))
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: expr_id,
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

    // expr -> expr ? expr : expr
    grammar
        .rules
        .entry(SymbolId(12))
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(SymbolId(2)), // ?
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(SymbolId(3)), // :
                Symbol::NonTerminal(expr_id),
            ],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Right),
            fields: vec![],
            production_id: ProductionId(1),
        });

    // expr -> expr + expr
    grammar
        .rules
        .entry(SymbolId(13))
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(SymbolId(4)), // +
                Symbol::NonTerminal(expr_id),
            ],
            precedence: Some(PrecedenceKind::Static(2)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(2),
        });

    grammar
}

/// Build a grammar with reduce/reduce conflicts
fn build_reduce_reduce_grammar() -> Grammar {
    let mut grammar = Grammar::new("reduce_reduce".to_string());

    // Tokens
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "c".to_string(),
            pattern: TokenPattern::String("c".to_string()),
            fragile: false,
        },
    );

    // Non-terminals
    let s_id = SymbolId(10);
    let x_id = SymbolId(11);
    let y_id = SymbolId(12);

    grammar.rule_names.insert(s_id, "S".to_string());
    grammar.rule_names.insert(x_id, "X".to_string());
    grammar.rule_names.insert(y_id, "Y".to_string());

    // Rules with reduce/reduce conflict
    // S -> X c
    grammar
        .rules
        .entry(SymbolId(20))
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: s_id,
            rhs: vec![Symbol::NonTerminal(x_id), Symbol::Terminal(SymbolId(3))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

    // S -> Y c
    grammar
        .rules
        .entry(SymbolId(21))
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: s_id,
            rhs: vec![Symbol::NonTerminal(y_id), Symbol::Terminal(SymbolId(3))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        });

    // X -> a b
    grammar
        .rules
        .entry(SymbolId(22))
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: x_id,
            rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        });

    // Y -> a b (same as X, causing reduce/reduce conflict)
    grammar
        .rules
        .entry(SymbolId(23))
        .or_insert_with(Vec::new)
        .push(Rule {
            lhs: y_id,
            rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(3),
        });

    grammar
}

#[test]
fn test_ternary_operator() {
    let grammar = build_ternary_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    let mut parser = GLRParser::new(table, grammar);

    // Parse "a ? b : c + d"
    // Should parse as "a ? b : (c + d)" due to precedence
    parser.process_token(SymbolId(1), "a", 0);
    parser.process_token(SymbolId(2), "?", 2);
    parser.process_token(SymbolId(1), "b", 4);
    parser.process_token(SymbolId(3), ":", 6);
    parser.process_token(SymbolId(1), "c", 8);
    parser.process_token(SymbolId(4), "+", 10);
    parser.process_token(SymbolId(1), "d", 12);
    parser.process_eof(14); // Total bytes in "a ? b : c + d"

    let result = parser.get_best_parse();
    assert!(result.is_some());

    // Just verify we got a parse tree
    let tree = result.unwrap();
    // Basic sanity check - we parsed something
    assert!(tree.node.symbol_id.0 > 0);
}

#[test]
fn test_nested_ternary() {
    let grammar = build_ternary_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    let mut parser = GLRParser::new(table, grammar);

    // Parse "a ? b : c ? d : e"
    // Should parse as "a ? b : (c ? d : e)" due to right associativity
    parser.process_token(SymbolId(1), "a", 0);
    parser.process_token(SymbolId(2), "?", 2);
    parser.process_token(SymbolId(1), "b", 4);
    parser.process_token(SymbolId(3), ":", 6);
    parser.process_token(SymbolId(1), "c", 8);
    parser.process_token(SymbolId(2), "?", 10);
    parser.process_token(SymbolId(1), "d", 12);
    parser.process_token(SymbolId(3), ":", 14);
    parser.process_token(SymbolId(1), "e", 16);
    parser.process_eof(17); // Total bytes in "a ? b : c ? d : e"

    let result = parser.get_best_parse();
    assert!(result.is_some());
}

#[test]
fn test_reduce_reduce_conflict() {
    let grammar = build_reduce_reduce_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    let mut parser = GLRParser::new(table, grammar);

    // Parse "a b c"
    // This has a reduce/reduce conflict: could be X c or Y c
    parser.process_token(SymbolId(1), "a", 0);
    parser.process_token(SymbolId(2), "b", 2);
    parser.process_token(SymbolId(3), "c", 4);
    parser.process_eof(5); // Total bytes in "a b c"

    let result = parser.get_best_parse();
    assert!(result.is_some());

    // Without precedence, it should choose based on rule order or symbol ID
}

#[test]
fn test_symbol_comparison_tiebreaker() {
    // Test that symbol comparison works as final tie-breaker
    let v1 = VersionInfo::new();
    let v2 = VersionInfo::new();

    // Same versions, different symbols
    assert_eq!(
        compare_versions_with_symbols(&v1, &v2, SymbolId(10), SymbolId(20)),
        CompareResult::TakeLeft
    );

    assert_eq!(
        compare_versions_with_symbols(&v1, &v2, SymbolId(30), SymbolId(15)),
        CompareResult::TakeRight
    );
}
