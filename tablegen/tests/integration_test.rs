// Integration tests for the pure-Rust parser generation

use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::*;
use rust_sitter_tablegen::AbiLanguageBuilder;

#[test]
fn test_parentheses_grammar_generation() {
    // Create a simple parentheses grammar
    let mut grammar = Grammar::new("parens".to_string());

    // Add tokens
    let lparen = Token {
        name: "(".to_string(),
        pattern: TokenPattern::String("(".to_string()),
        fragile: false,
    };
    let rparen = Token {
        name: ")".to_string(),
        pattern: TokenPattern::String(")".to_string()),
        fragile: false,
    };

    grammar.tokens.insert(SymbolId(1), lparen);
    grammar.tokens.insert(SymbolId(2), rparen);

    // Add rules: expr -> '(' expr ')' | ε
    let expr_id = SymbolId(3);

    // Rule 1: expr -> '(' expr ')'
    let rule1 = Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(SymbolId(1)), // '('
            Symbol::NonTerminal(expr_id),  // expr
            Symbol::Terminal(SymbolId(2)), // ')'
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };

    // Rule 2: expr -> ε
    let rule2 = Rule {
        lhs: expr_id,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    // Add rules (first rule's LHS is the start symbol)
    grammar.add_rule(rule1);
    grammar.add_rule(rule2);

    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Generate language using ABI builder
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);

    let code = builder.generate();
    let code_str = code.to_string();

    // Verify generated code contains expected elements
    assert!(code_str.contains("TSLanguage"));
    assert!(code_str.contains("LANGUAGE_VERSION"));
    assert!(code_str.contains("symbol_count"));
    assert!(code_str.contains("PARSE_TABLE"));
    assert!(code_str.contains("tree_sitter_parens"));
}

#[test]
fn test_arithmetic_grammar_generation() {
    // Create arithmetic expression grammar
    let mut grammar = Grammar::new("arithmetic".to_string());

    // Add tokens
    let number = Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    let plus = Token {
        name: "+".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    };
    let times = Token {
        name: "*".to_string(),
        pattern: TokenPattern::String("*".to_string()),
        fragile: false,
    };

    grammar.tokens.insert(SymbolId(1), number);
    grammar.tokens.insert(SymbolId(2), plus);
    grammar.tokens.insert(SymbolId(3), times);

    // Add non-terminals
    let expr_id = SymbolId(4);
    let term_id = SymbolId(5);
    let factor_id = SymbolId(6);

    // expr -> expr '+' term | term
    let expr_rule1 = Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(SymbolId(2)), // '+'
            Symbol::NonTerminal(term_id),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    };

    let expr_rule2 = Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(term_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    // term -> term '*' factor | factor
    let term_rule1 = Rule {
        lhs: term_id,
        rhs: vec![
            Symbol::NonTerminal(term_id),
            Symbol::Terminal(SymbolId(3)), // '*'
            Symbol::NonTerminal(factor_id),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(2),
    };

    let term_rule2 = Rule {
        lhs: term_id,
        rhs: vec![Symbol::NonTerminal(factor_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(3),
    };

    // factor -> number
    let factor_rule = Rule {
        lhs: factor_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))], // number
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(4),
    };

    grammar.add_rule(expr_rule1);
    grammar.add_rule(expr_rule2);
    grammar.add_rule(term_rule1);
    grammar.add_rule(term_rule2);
    grammar.add_rule(factor_rule);

    // Add rule names
    grammar.rule_names.insert(expr_id, "expression".to_string());
    grammar.rule_names.insert(term_id, "term".to_string());
    grammar.rule_names.insert(factor_id, "factor".to_string());

    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Generate language using ABI builder
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);

    let code = builder.generate();
    let code_str = code.to_string();

    // Verify the code contains expected elements
    assert!(code_str.contains("TSLanguage"));
    assert!(code_str.contains("tree_sitter_arithmetic"));
    assert!(code_str.contains("SYMBOL_NAME_"));

    // Verify counts are correct
    assert!(code_str.contains("symbol_count : 7u32")); // 3 tokens + 4 non-terminals (EOF, number, +, *, expr, term, factor)
    assert!(code_str.contains("token_count : 3u32")); // number, +, *

    // Verify table compression
    assert!(code_str.contains("PARSE_TABLE"));
    assert!(code_str.contains("symbol_count"));
}

#[test]
fn test_field_mapping_generation() {
    // Create grammar with field names
    let mut grammar = Grammar::new("fields".to_string());

    // Add tokens
    let id = Token {
        name: "identifier".to_string(),
        pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
        fragile: false,
    };
    let eq = Token {
        name: "=".to_string(),
        pattern: TokenPattern::String("=".to_string()),
        fragile: false,
    };

    grammar.tokens.insert(SymbolId(1), id);
    grammar.tokens.insert(SymbolId(2), eq);

    // Add field names
    grammar.fields.insert(FieldId(0), "name".to_string());
    grammar.fields.insert(FieldId(1), "value".to_string());

    // assignment -> identifier '=' identifier
    let assignment_id = SymbolId(3);
    let assignment_rule = Rule {
        lhs: assignment_id,
        rhs: vec![
            Symbol::Terminal(SymbolId(1)), // identifier
            Symbol::Terminal(SymbolId(2)), // '='
            Symbol::Terminal(SymbolId(1)), // identifier
        ],
        precedence: None,
        associativity: None,
        fields: vec![
            (FieldId(0), 0), // name field at position 0
            (FieldId(1), 2), // value field at position 2
        ],
        production_id: ProductionId(0),
    };

    grammar.add_rule(assignment_rule);
    grammar
        .rule_names
        .insert(assignment_id, "assignment".to_string());

    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Generate language using ABI builder
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);

    let code = builder.generate();
    let code_str = code.to_string();

    // Verify field count is correct
    assert!(code_str.contains("field_count : 2u32"));
    assert!(code_str.contains("FIELD_NAME_"));
}
