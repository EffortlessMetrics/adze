// Simple grammar converter for demonstration
// This would be expanded with actual grammar extraction logic

use rust_sitter_ir::{
    Grammar, Rule, Symbol, SymbolId, Token, TokenPattern, 
    RuleId, ProductionId, FieldId, PrecedenceKind, Associativity
};
use std::collections::HashMap;

/// Simplified grammar converter
pub struct GrammarConverter;

impl GrammarConverter {
    /// Create a sample grammar for testing
    pub fn create_sample_grammar() -> Grammar {
        let mut grammar = Grammar::new("sample".to_string());
        
        // Define some basic tokens
        let id_symbol = SymbolId(1);
        let num_symbol = SymbolId(2);
        let plus_symbol = SymbolId(3);
        let expr_symbol = SymbolId(4);
        
        // Add tokens
        grammar.tokens.insert(id_symbol, Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            fragile: false,
        });
        
        grammar.tokens.insert(num_symbol, Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        });
        
        grammar.tokens.insert(plus_symbol, Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        });
        
        // Add rules
        // expr -> identifier
        grammar.rules.insert(expr_symbol, Rule {
            lhs: expr_symbol,
            rhs: vec![Symbol::Terminal(id_symbol)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        
        // expr -> number
        grammar.rules.insert(expr_symbol, Rule {
            lhs: expr_symbol,
            rhs: vec![Symbol::Terminal(num_symbol)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        });
        
        // expr -> expr + expr
        grammar.rules.insert(expr_symbol, Rule {
            lhs: expr_symbol,
            rhs: vec![
                Symbol::NonTerminal(expr_symbol),
                Symbol::Terminal(plus_symbol),
                Symbol::NonTerminal(expr_symbol),
            ],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![(FieldId(1), 0), (FieldId(2), 2)], // left, right
            production_id: ProductionId(2),
        });
        
        // Add field names
        grammar.fields.insert(FieldId(1), "left".to_string());
        grammar.fields.insert(FieldId(2), "right".to_string());
        
        grammar
    }
}