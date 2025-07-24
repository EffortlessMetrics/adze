use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use std::sync::Arc;

// Import the necessary modules
#[path = "runtime/src/glr_lexer.rs"]
mod glr_lexer;
#[path = "runtime/src/glr_parser.rs"]
mod glr_parser;
#[path = "runtime/src/subtree.rs"]
mod subtree;

use glr_lexer::{GLRLexer, TokenWithPosition};
use glr_parser::GLRParser;
use subtree::Subtree;

/// Create a simple expression grammar for testing
fn create_expression_grammar() -> Grammar {
    let mut grammar = Grammar::new("expression".to_string());
    
    // Define terminals (SymbolId(0) is reserved for EOF)
    let number_id = SymbolId(1);
    grammar.tokens.insert(number_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    let plus_id = SymbolId(2);
    grammar.tokens.insert(plus_id, Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    
    let lparen_id = SymbolId(6);
    grammar.tokens.insert(lparen_id, Token {
        name: "lparen".to_string(),
        pattern: TokenPattern::String("(".to_string()),
        fragile: false,
    });
    
    let rparen_id = SymbolId(7);
    grammar.tokens.insert(rparen_id, Token {
        name: "rparen".to_string(),
        pattern: TokenPattern::String(")".to_string()),
        fragile: false,
    });
    
    // Define non-terminals
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expression".to_string());
    
    // Rules
    let add_rule_id = SymbolId(20);
    let paren_rule_id = SymbolId(24);
    let number_rule_id = SymbolId(25);
    
    // expression → expression + expression
    grammar.rules.insert(add_rule_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(expr_id), Symbol::Terminal(plus_id), Symbol::NonTerminal(expr_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    
    // expression → ( expression )
    grammar.rules.insert(paren_rule_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(lparen_id), Symbol::NonTerminal(expr_id), Symbol::Terminal(rparen_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    
    // expression → number
    grammar.rules.insert(number_rule_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(2),
        fields: vec![],
    });
    
    grammar
}

fn parse_tokens(parser: &mut GLRParser, tokens: &[TokenWithPosition]) -> Option<Arc<Subtree>> {
    parser.reset();
    
    for token in tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
    }
    
    parser.process_eof();
    parser.finish().ok()
}

fn main() {
    let grammar = create_expression_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());
    
    // Test cases for nested parentheses
    let test_cases = vec![
        ("1", "single number"),
        ("(1)", "single parentheses"),
        ("((1))", "double parentheses"),
        ("(((1)))", "triple parentheses"),
        ("((((1))))", "quadruple parentheses"),
        ("1 + 2", "simple addition"),
        ("(1 + 2)", "parenthesized addition"),
        ("((1 + 2))", "double parenthesized addition"),
        ("(((1 + 2)))", "triple parenthesized addition"),
        ("((1) + (2))", "parenthesized operands"),
        ("(((1)) + ((2)))", "deeply parenthesized operands"),
    ];
    
    for (input, description) in test_cases {
        println!("\nTesting: {} - {}", input, description);
        
        let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        
        println!("Tokens: {:?}", tokens.iter().map(|t| (t.symbol_id, &t.text)).collect::<Vec<_>>());
        
        match parse_tokens(&mut parser, &tokens) {
            Some(tree) => {
                println!("✓ Parse succeeded");
                print_tree(&tree, 0);
            }
            None => {
                println!("✗ Parse FAILED");
            }
        }
    }
}

fn print_tree(tree: &Arc<Subtree>, indent: usize) {
    let spaces = " ".repeat(indent);
    println!("{}Symbol: {:?}, Text: {:?}", spaces, tree.node.symbol_id, tree.node.text);
    for child in &tree.children {
        print_tree(child, indent + 2);
    }
}