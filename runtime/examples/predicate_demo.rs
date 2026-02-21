//! Demonstrates query predicate evaluation in adze

use adze::{
    parser::ParseNode,
    query::{ast::Predicate, predicate_eval::PredicateContext},
};
use adze_ir::SymbolId;
use std::collections::HashMap;

fn main() {
    // Example source code
    let source = "hello world test hello";

    // Create a simple parse tree (normally this would come from a parser)
    let tree = ParseNode {
        symbol: SymbolId(0),    // program
        symbol_id: SymbolId(0), // program
        children: vec![
            ParseNode {
                symbol: SymbolId(1),    // identifier
                symbol_id: SymbolId(1), // identifier
                children: vec![],
                start_byte: 0,
                end_byte: 5,
                field_name: None,
            },
            ParseNode {
                symbol: SymbolId(1),    // identifier
                symbol_id: SymbolId(1), // identifier
                children: vec![],
                start_byte: 6,
                end_byte: 11,
                field_name: None,
            },
            ParseNode {
                symbol: SymbolId(1),    // identifier
                symbol_id: SymbolId(1), // identifier
                children: vec![],
                start_byte: 12,
                end_byte: 16,
                field_name: None,
            },
            ParseNode {
                symbol: SymbolId(1),    // identifier
                symbol_id: SymbolId(1), // identifier
                children: vec![],
                start_byte: 17,
                end_byte: 22,
                field_name: None,
            },
        ],
        start_byte: 0,
        end_byte: 22,
        field_name: None,
    };

    // Create predicate context with source text
    let predicate_ctx = PredicateContext::new(source);

    // Example 1: #eq? predicate with value
    println!("=== Testing #eq? predicate with value ===");
    let mut captures = HashMap::new();
    captures.insert(0, tree.children[0].clone()); // "hello"

    let eq_pred = Predicate::Eq {
        capture1: 0,
        capture2: None,
        value: Some("hello".to_string()),
    };

    let result = predicate_ctx.evaluate(&eq_pred, &captures);
    println!("Is first identifier 'hello'? {}", result);

    // Example 2: #eq? predicate between captures
    println!("\n=== Testing #eq? predicate between captures ===");
    captures.insert(1, tree.children[3].clone()); // also "hello"

    let eq_captures_pred = Predicate::Eq {
        capture1: 0,
        capture2: Some(1),
        value: None,
    };

    let result = predicate_ctx.evaluate(&eq_captures_pred, &captures);
    println!("Are capture 0 and capture 1 equal? {}", result);

    // Example 3: #match? predicate
    println!("\n=== Testing #match? predicate ===");
    captures.clear();
    captures.insert(0, tree.children[1].clone()); // "world"

    let match_pred = Predicate::Match {
        capture: 0,
        regex: r"^w\w+d$".to_string(), // matches words starting with 'w' and ending with 'd'
    };

    let result = predicate_ctx.evaluate(&match_pred, &captures);
    println!("Does 'world' match pattern ^w\\w+d$? {}", result);

    // Example 4: #any-of? predicate
    println!("\n=== Testing #any-of? predicate ===");
    let any_of_pred = Predicate::AnyOf {
        capture: 0,
        values: vec!["hello".to_string(), "world".to_string(), "test".to_string()],
    };

    let result = predicate_ctx.evaluate(&any_of_pred, &captures);
    println!("Is 'world' in [hello, world, test]? {}", result);

    // Example 5: #not-eq? predicate
    println!("\n=== Testing #not-eq? predicate ===");
    let not_eq_pred = Predicate::NotEq {
        capture1: 0,
        capture2: None,
        value: Some("hello".to_string()),
    };

    let result = predicate_ctx.evaluate(&not_eq_pred, &captures);
    println!("Is 'world' NOT equal to 'hello'? {}", result);

    // Show how predicates filter query matches
    println!("\n=== Demonstrating predicate filtering ===");
    println!("Without predicates: 4 identifiers matched");
    println!("With #eq? @id 'hello': only 2 matches (positions 0 and 17)");
    println!("With #match? @id '^h': only 2 matches starting with 'h'");
    println!("With #any-of? @id ['test', 'world']: only 2 matches");
}
