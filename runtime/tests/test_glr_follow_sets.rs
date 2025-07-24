// Test FOLLOW sets computation
use rust_sitter_glr_core::{FirstFollowSets};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern,
    ProductionId, SymbolId,
};

fn build_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new("arithmetic".to_string());
    
    // Tokens
    grammar.tokens.insert(SymbolId(1), Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(2), Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    
    // Non-terminals
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expr".to_string());
    
    // Rules
    // expr -> number
    grammar.rules.insert(expr_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    // expr -> expr + expr
    grammar.rules.insert(SymbolId(11), Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    
    grammar
}

#[test]
fn test_follow_sets() {
    let grammar = build_arithmetic_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    println!("Grammar rules:");
    for (id, rule) in &grammar.rules {
        println!("  Rule {}: {:?}", id.0, rule);
    }
    
    println!("\nChecking FOLLOW(expr):");
    // In our grammar, expr can be followed by:
    // - EOF (since it's the start symbol)
    // - '+' (from expr + expr)
    // So state 1 should have reduce actions for both EOF and '+'
    
    println!("\nThe issue: After shifting 'number' to state 1,");
    println!("we need to reduce 'expr -> number' when we see:");
    println!("- EOF (accept)");
    println!("- '+' (so we can continue parsing expr + expr)");
}