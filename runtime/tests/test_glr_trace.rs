// Trace through LR(1) item set construction
use rust_sitter_glr_core::{
    FirstFollowSets, ItemSetCollection,
};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern,
    ProductionId, SymbolId,
};

fn build_simple_expr_grammar() -> Grammar {
    let mut grammar = Grammar::new("simple_expr".to_string());
    
    // Token: number
    grammar.tokens.insert(SymbolId(1), Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    // Non-terminal: expr
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expr".to_string());
    
    // Rule: expr -> number
    grammar.rules.insert(expr_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    grammar
}

#[test]
fn test_trace_item_sets() {
    let grammar = build_simple_expr_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    println!("Grammar rules:");
    for (id, rule) in &grammar.rules {
        println!("  Rule {}: {:?}", id.0, rule);
    }
    
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &first_follow);
    
    println!("\nItem sets:");
    for (i, set) in collection.sets.iter().enumerate() {
        println!("\nState {}:", i);
        for item in &set.items {
            println!("  Item: rule_id={}, position={}, lookahead={}", 
                     item.rule_id.0, item.position, item.lookahead.0);
        }
    }
    
    println!("\nGoto table:");
    for ((from, symbol), to) in &collection.goto_table {
        println!("  State {} --[Symbol {}]--> State {}", from.0, symbol.0, to.0);
    }
}