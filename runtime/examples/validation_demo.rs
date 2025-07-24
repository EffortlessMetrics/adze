// Demonstration of enhanced grammar validation with helpful error messages
use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};

// Import internal validation module (in real usage, this would be exported)
#[path = "../src/glr_validation.rs"]
mod glr_validation;

use glr_validation::GLRGrammarValidator;

fn main() {
    println!("=== Grammar Validation Demo ===\n");
    
    // Example 1: Grammar with typos
    println!("Example 1: Grammar with typos");
    println!("------------------------------");
    validate_grammar_with_typos();
    println!();
    
    // Example 2: Grammar with left recursion
    println!("Example 2: Grammar with left recursion");
    println!("--------------------------------------");
    validate_left_recursive_grammar();
    println!();
    
    // Example 3: Non-productive grammar
    println!("Example 3: Non-productive grammar");
    println!("---------------------------------");
    validate_non_productive_grammar();
    println!();
    
    // Example 4: Ambiguous grammar
    println!("Example 4: Ambiguous grammar");
    println!("----------------------------");
    validate_ambiguous_grammar();
}

fn validate_grammar_with_typos() {
    let mut grammar = Grammar::new("calculator".to_string());
    
    // Define tokens correctly
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let minus_id = SymbolId(3);
    
    grammar.tokens.insert(number_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(number_id, "number".to_string());
    
    grammar.tokens.insert(plus_id, Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(plus_id, "plus".to_string());
    
    grammar.tokens.insert(minus_id, Token {
        name: "minus".to_string(),
        pattern: TokenPattern::String("-".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(minus_id, "minus".to_string());
    
    // Create a rule with a typo: "numbr" instead of "number"
    let expr_id = SymbolId(10);
    let numbr_typo = SymbolId(99);
    
    grammar.rules.insert(expr_id, Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(numbr_typo), // typo!
            Symbol::Terminal(plus_id),
            Symbol::Terminal(number_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(expr_id, "expr".to_string());
    grammar.rule_names.insert(numbr_typo, "numbr".to_string());
    
    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);
    
    if !result.is_valid {
        for error in &result.errors {
            println!("{}", error);
        }
    }
}

fn validate_left_recursive_grammar() {
    let mut grammar = Grammar::new("expression".to_string());
    
    // Tokens
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let times_id = SymbolId(3);
    
    grammar.tokens.insert(number_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(number_id, "number".to_string());
    
    grammar.tokens.insert(plus_id, Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(plus_id, "plus".to_string());
    
    grammar.tokens.insert(times_id, Token {
        name: "times".to_string(),
        pattern: TokenPattern::String("*".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(times_id, "times".to_string());
    
    // Left-recursive rules
    let expr_id = SymbolId(10);
    let term_id = SymbolId(11);
    
    // expr → expr + term (left recursive!)
    grammar.rules.insert(expr_id, Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(term_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(expr_id, "expr".to_string());
    
    // term → number
    grammar.rules.insert(term_id, Rule {
        lhs: term_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.rule_names.insert(term_id, "term".to_string());
    
    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);
    
    // Print warnings
    for warning in &result.warnings {
        println!("Warning: {}", warning.message);
        if let Some(suggestion) = &warning.suggestion {
            println!("  → {}", suggestion);
        }
    }
    
    // Print statistics
    println!("\nGrammar Statistics:");
    println!("  - Has left recursion: {}", result.stats.has_left_recursion);
    println!("  - Requires GLR: {}", result.stats.requires_glr);
    
    // Print suggestions
    if !result.suggestions.is_empty() {
        println!("\nSuggestions:");
        for suggestion in &result.suggestions {
            println!("  • {}", suggestion);
        }
    }
}

fn validate_non_productive_grammar() {
    let mut grammar = Grammar::new("cyclic".to_string());
    
    // Create a cycle: A → B → C → A (no way to derive terminals)
    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    let c_id = SymbolId(3);
    
    grammar.rules.insert(a_id, Rule {
        lhs: a_id,
        rhs: vec![Symbol::NonTerminal(b_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(a_id, "A".to_string());
    
    grammar.rules.insert(b_id, Rule {
        lhs: b_id,
        rhs: vec![Symbol::NonTerminal(c_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.rule_names.insert(b_id, "B".to_string());
    
    grammar.rules.insert(c_id, Rule {
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
    
    if !result.is_valid {
        println!("Validation failed with {} errors:", result.errors.len());
        for error in &result.errors {
            println!("\n{}", error);
        }
    }
}

fn validate_ambiguous_grammar() {
    let mut grammar = Grammar::new("arithmetic".to_string());
    
    // Tokens
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let times_id = SymbolId(3);
    
    grammar.tokens.insert(number_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(number_id, "number".to_string());
    
    grammar.tokens.insert(plus_id, Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(plus_id, "plus".to_string());
    
    grammar.tokens.insert(times_id, Token {
        name: "times".to_string(),
        pattern: TokenPattern::String("*".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(times_id, "times".to_string());
    
    // Ambiguous rules (no precedence)
    let expr_id = SymbolId(10);
    
    // expr → expr + expr (ambiguous!)
    grammar.rules.insert(expr_id, Rule {
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
    
    // Also: expr → expr * expr (more ambiguity!)
    let expr_times_id = SymbolId(11);
    grammar.rules.insert(expr_times_id, Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(times_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    
    // expr → number
    let expr_num_id = SymbolId(12);
    grammar.rules.insert(expr_num_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    
    grammar.rule_names.insert(expr_id, "expr".to_string());
    
    let mut validator = GLRGrammarValidator::new();
    let result = validator.validate(&grammar);
    
    println!("Grammar Analysis:");
    println!("  - Valid: {}", result.is_valid);
    println!("  - Warnings: {}", result.warnings.len());
    println!("  - Requires GLR: {}", result.stats.requires_glr);
    
    for warning in &result.warnings {
        println!("\nWarning: {}", warning.message);
        if let Some(suggestion) = &warning.suggestion {
            println!("  → {}", suggestion);
        }
    }
    
    if !result.suggestions.is_empty() {
        println!("\nRecommendations:");
        for suggestion in &result.suggestions {
            println!("  • {}", suggestion);
        }
    }
}