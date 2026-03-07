// Test enhanced grammar validation with helpful error messages
use adze::adze_ir as ir;

use ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// NOTE: These tests use internal modules not exported by the public API
#[path = "../src/glr_validation.rs"]
mod glr_validation;

use glr_validation::{ErrorKind, GLRGrammarValidator};

#[test]
fn test_comprehensive_validation() {
    let mut grammar = Grammar::new("comprehensive_test".to_string());

    // Define some tokens
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let times_id = SymbolId(3);
    let lparen_id = SymbolId(4);
    let rparen_id = SymbolId(5);

    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(number_id, "number".to_string());

    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(plus_id, "plus".to_string());

    grammar.tokens.insert(
        times_id,
        Token {
            name: "times".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(times_id, "times".to_string());

    grammar.tokens.insert(
        lparen_id,
        Token {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(lparen_id, "lparen".to_string());

    grammar.tokens.insert(
        rparen_id,
        Token {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(rparen_id, "rparen".to_string());

    // Define rules with left recursion
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expr".to_string());

    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);

    // Should detect left recursion
    assert!(result.stats.has_left_recursion);
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("left recursion"))
    );

    // Should have helpful suggestions
    assert!(!result.suggestions.is_empty());
    assert!(result.suggestions.iter().any(|s| s.contains("GLR")));
}

#[test]
fn test_undefined_symbol_with_typo() {
    let mut grammar = Grammar::new("typo_test".to_string());

    // Define tokens
    let number_id = SymbolId(1);
    let identifier_id = SymbolId(2);

    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(number_id, "number".to_string());

    grammar.tokens.insert(
        identifier_id,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            fragile: false,
        },
    );
    grammar
        .rule_names
        .insert(identifier_id, "identifier".to_string());

    // Create a rule that uses "identifer" (typo) instead of "identifier"
    let expr_id = SymbolId(10);
    let identifer_typo_id = SymbolId(99);

    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(identifer_typo_id)], // typo
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(expr_id, "expr".to_string());
    grammar
        .rule_names
        .insert(identifer_typo_id, "identifer".to_string()); // typo

    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);

    // Should have undefined symbol error
    assert!(!result.is_valid);
    let undefined_error = result
        .errors
        .iter()
        .find(|e| e.kind == ErrorKind::UndefinedSymbol)
        .expect("Should have undefined symbol error");

    // Should suggest "identifier" as a correction
    assert!(!undefined_error.related.is_empty());
    assert!(
        undefined_error
            .related
            .iter()
            .any(|r| r.message.contains("identifier"))
    );

    // Error should have helpful location info
    assert!(undefined_error.location.description.contains("expr"));
}

#[test]
fn test_non_productive_cycle_detection() {
    let mut grammar = Grammar::new("cycle_test".to_string());

    // Create a non-productive cycle: A → B, B → C, C → A
    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    let c_id = SymbolId(3);

    grammar.rules.entry(a_id).or_default().push(Rule {
        lhs: a_id,
        rhs: vec![Symbol::NonTerminal(b_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(a_id, "A".to_string());

    grammar.rules.entry(b_id).or_default().push(Rule {
        lhs: b_id,
        rhs: vec![Symbol::NonTerminal(c_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.rule_names.insert(b_id, "B".to_string());

    grammar.rules.entry(c_id).or_default().push(Rule {
        lhs: c_id,
        rhs: vec![Symbol::NonTerminal(a_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    grammar.rule_names.insert(c_id, "C".to_string());

    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);

    // Should detect non-productive symbols
    assert!(!result.is_valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.kind == ErrorKind::NonProductiveSymbol)
    );

    // Should identify all symbols in the cycle
    let non_prod_errors: Vec<_> = result
        .errors
        .iter()
        .filter(|e| e.kind == ErrorKind::NonProductiveSymbol)
        .collect();
    assert_eq!(non_prod_errors.len(), 3); // All three symbols are non-productive

    // Should have related information about the cycle
    assert!(non_prod_errors.iter().all(|e| !e.related.is_empty()));
}

#[test]
fn test_ambiguous_grammar_detection() {
    let mut grammar = Grammar::new("ambiguous_test".to_string());

    // Create an ambiguous grammar: classic if-then-else
    let if_id = SymbolId(1);
    let then_id = SymbolId(2);
    let else_id = SymbolId(3);
    let expr_id = SymbolId(4);
    let stmt_id = SymbolId(5);
    let if_stmt_id = SymbolId(6);

    // Tokens
    grammar.tokens.insert(
        if_id,
        Token {
            name: "if".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(if_id, "if".to_string());

    grammar.tokens.insert(
        then_id,
        Token {
            name: "then".to_string(),
            pattern: TokenPattern::String("then".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(then_id, "then".to_string());

    grammar.tokens.insert(
        else_id,
        Token {
            name: "else".to_string(),
            pattern: TokenPattern::String("else".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(else_id, "else".to_string());

    // expr is a placeholder
    grammar.tokens.insert(
        expr_id,
        Token {
            name: "expr".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(expr_id, "expr".to_string());

    // Rules that create ambiguity
    // stmt → if_stmt
    grammar.rules.entry(stmt_id).or_default().push(Rule {
        lhs: stmt_id,
        rhs: vec![Symbol::NonTerminal(if_stmt_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(stmt_id, "stmt".to_string());

    // if_stmt → if expr then stmt
    grammar.rules.entry(if_stmt_id).or_default().push(Rule {
        lhs: if_stmt_id,
        rhs: vec![
            Symbol::Terminal(if_id),
            Symbol::Terminal(expr_id),
            Symbol::Terminal(then_id),
            Symbol::NonTerminal(stmt_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.rule_names.insert(if_stmt_id, "if_stmt".to_string());

    // Also allow: if_stmt → if expr then stmt else stmt (creates ambiguity)
    grammar.rules.entry(if_stmt_id).or_default().push(Rule {
        lhs: if_stmt_id,
        rhs: vec![
            Symbol::Terminal(if_id),
            Symbol::Terminal(expr_id),
            Symbol::Terminal(then_id),
            Symbol::NonTerminal(stmt_id),
            Symbol::Terminal(else_id),
            Symbol::NonTerminal(stmt_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });

    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);

    // Should detect that GLR is required
    assert!(result.stats.requires_glr);

    // Should have warnings about ambiguity
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("ambiguity"))
    );

    // Should suggest GLR parsing
    assert!(result.suggestions.iter().any(|s| s.contains("GLR")));
}

#[test]
fn test_token_validation() {
    let mut grammar = Grammar::new("token_test".to_string());

    // Invalid regex token
    let bad_regex_id = SymbolId(1);
    grammar.tokens.insert(
        bad_regex_id,
        Token {
            name: "bad_regex".to_string(),
            pattern: TokenPattern::Regex(r"[a-".to_string()), // Invalid regex
            fragile: false,
        },
    );
    grammar
        .rule_names
        .insert(bad_regex_id, "bad_regex".to_string());

    // Empty string token
    let empty_id = SymbolId(2);
    grammar.tokens.insert(
        empty_id,
        Token {
            name: "empty".to_string(),
            pattern: TokenPattern::String("".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(empty_id, "empty".to_string());

    // Overlapping tokens
    let word1_id = SymbolId(3);
    grammar.tokens.insert(
        word1_id,
        Token {
            name: "word1".to_string(),
            pattern: TokenPattern::Regex(r"\w+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(word1_id, "word1".to_string());

    let word2_id = SymbolId(4);
    grammar.tokens.insert(
        word2_id,
        Token {
            name: "word2".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z]+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(word2_id, "word2".to_string());

    // Add a rule to make grammar non-empty
    let expr_id = SymbolId(10);
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(word1_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(expr_id, "expr".to_string());

    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);

    // Should have errors for invalid tokens
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.kind == ErrorKind::InvalidToken && e.message.contains("regex"))
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.kind == ErrorKind::InvalidToken && e.message.contains("empty"))
    );

    // Should warn about overlapping tokens
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("overlapping"))
    );
}

#[test]
fn test_helpful_error_formatting() {
    let mut grammar = Grammar::new("format_test".to_string());

    // Create an undefined symbol error
    let expr_id = SymbolId(1);
    let undefined_id = SymbolId(99);

    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(undefined_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(expr_id, "expression".to_string());
    grammar
        .rule_names
        .insert(undefined_id, "missing_token".to_string());

    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);

    // Get the error
    let error = result
        .errors
        .iter()
        .find(|e| e.kind == ErrorKind::UndefinedSymbol)
        .expect("Should have undefined symbol error");

    // Format the error
    let formatted = format!("{}", error);

    // Should contain all parts
    assert!(formatted.contains("Error:"));
    assert!(formatted.contains("Location:"));
    assert!(formatted.contains("Suggestion:"));
    assert!(formatted.contains("missing_token"));
    assert!(formatted.contains("expression"));
}

#[test]
fn test_unreachable_symbol_warning() {
    let mut grammar = Grammar::new("unreachable_test".to_string());

    // Define tokens
    let used_token = SymbolId(1);
    let unused_token = SymbolId(2);

    grammar.tokens.insert(
        used_token,
        Token {
            name: "used".to_string(),
            pattern: TokenPattern::String("used".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(used_token, "used".to_string());

    grammar.tokens.insert(
        unused_token,
        Token {
            name: "unused".to_string(),
            pattern: TokenPattern::String("unused".to_string()),
            fragile: false,
        },
    );
    grammar
        .rule_names
        .insert(unused_token, "unused".to_string());

    // Create a rule that only uses one token
    let expr_id = SymbolId(10);
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(used_token)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(expr_id, "expr".to_string());

    // Create an unreachable rule
    let unreachable_rule = SymbolId(11);
    grammar
        .rules
        .entry(unreachable_rule)
        .or_default()
        .push(Rule {
            lhs: unreachable_rule,
            rhs: vec![Symbol::Terminal(unused_token)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        });
    grammar
        .rule_names
        .insert(unreachable_rule, "unreachable_rule".to_string());

    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);

    // Should warn about unused token and unreachable rule
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("unused") && w.message.contains("never used"))
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("unreachable_rule") && w.message.contains("not reachable"))
    );

    // Should still be valid (warnings don't make grammar invalid)
    assert!(result.is_valid);
}
