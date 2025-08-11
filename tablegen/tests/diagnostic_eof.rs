// Diagnostic test to demonstrate EOF handling in state 0
use rust_sitter_glr_core::{Action, ParseTable, RuleId, StateId, SymbolId};
use rust_sitter_ir::{Grammar, Production, Rule};
use rust_sitter_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};
use rust_sitter_tablegen::TableCompressor;
use std::collections::BTreeMap;

#[test]
fn diagnostic_eof_handling() {
    println!("\n=== DIAGNOSTIC: State 0 EOF Handling ===\n");
    
    // Create a grammar with nullable start symbol (like Python's module)
    let mut grammar = Grammar {
        name: "diagnostic_test".to_string(),
        start_symbol: SymbolId(1), // module
        rules: BTreeMap::new(),
        tokens: BTreeMap::new(),
        variables: BTreeMap::new(),
        external_tokens: vec![],
        precedence_rules: vec![],
        fields: BTreeMap::new(),
        extra_symbols: vec![],
        word_token: None,
        supertype_symbols: vec![],
    };
    
    // Add EOF token (always symbol 0)
    grammar.tokens.insert(SymbolId(0), "EOF".to_string());
    
    // Add a statement token
    grammar.tokens.insert(SymbolId(2), "statement".to_string());
    
    // Add module variable (start symbol)
    grammar.variables.insert(SymbolId(1), "module".to_string());
    
    // Add rule: module -> ε (empty)
    grammar.rules.insert(
        RuleId(0),
        Rule {
            id: RuleId(0),
            lhs: SymbolId(1), // module
            rhs: vec![], // empty production (nullable)
            alias: None,
            precedence: None,
            associativity: None,
            is_fragile: false,
            production_id: None,
        },
    );
    
    // Add rule: module -> statement module
    grammar.rules.insert(
        RuleId(1),
        Rule {
            id: RuleId(1),
            lhs: SymbolId(1), // module
            rhs: vec![Production::Symbol(SymbolId(2)), Production::Symbol(SymbolId(1))],
            alias: None,
            precedence: None,
            associativity: None,
            is_fragile: false,
            production_id: None,
        },
    );
    
    // Create parse table with state 0 having both shift and reduce actions
    let mut parse_table = ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 3,
        symbol_to_index: BTreeMap::new(),
        external_scanner_states: vec![],
    };
    
    // Map symbols to column indices
    parse_table.symbol_to_index.insert(SymbolId(0), 0); // EOF at column 0
    parse_table.symbol_to_index.insert(SymbolId(1), 1); // module at column 1
    parse_table.symbol_to_index.insert(SymbolId(2), 2); // statement at column 2
    
    // State 0: Initial state with GLR conflict
    // - EOF: Accept (empty module) or Reduce(0) for explicit empty rule
    // - statement: Shift to state 1 (non-empty module)
    parse_table.action_table.push(vec![
        vec![Action::Accept, Action::Reduce(RuleId(0))], // EOF: both accept and reduce (nullable start)
        vec![Action::Error], // module: error
        vec![Action::Shift(StateId(1))], // statement: shift
    ]);
    
    parse_table.state_count = 1;
    
    println!("1. Grammar structure:");
    println!("   - Start symbol: module (symbol {})", grammar.start_symbol.0);
    println!("   - Rule 0: module → ε (empty/nullable)");
    println!("   - Rule 1: module → statement module");
    println!();
    
    println!("2. Symbol to column mapping:");
    for (symbol, col) in &parse_table.symbol_to_index {
        let name = if symbol.0 == 0 {
            "EOF"
        } else if symbol.0 == 1 {
            "module"
        } else {
            "statement"
        };
        println!("   - Symbol {} ({}) → column {}", symbol.0, name, col);
    }
    println!();
    
    println!("3. State 0 action table:");
    let state0 = &parse_table.action_table[0];
    for (col, actions) in state0.iter().enumerate() {
        if !actions.is_empty() {
            let symbol_name = parse_table.symbol_to_index
                .iter()
                .find(|(_, &c)| c == col)
                .map(|(s, _)| {
                    if s.0 == 0 { "EOF" }
                    else if s.0 == 1 { "module" }
                    else { "statement" }
                })
                .unwrap_or("unknown");
            
            print!("   - Column {} ({}): ", col, symbol_name);
            for action in actions {
                match action {
                    Action::Accept => print!("Accept "),
                    Action::Reduce(r) => print!("Reduce({}) ", r.0),
                    Action::Shift(s) => print!("Shift({}) ", s.0),
                    Action::Error => print!("Error "),
                    _ => print!("Other "),
                }
            }
            println!();
        }
    }
    println!();
    
    // Test helper function
    println!("4. Helper function results:");
    let token_indices = collect_token_indices(&grammar, &parse_table);
    println!("   - collect_token_indices: {:?}", token_indices);
    assert!(token_indices.contains(&0), "EOF must be in token_indices");
    
    let has_eof_accept_or_reduce = eof_accepts_or_reduces(&parse_table);
    println!("   - eof_accepts_or_reduces: {}", has_eof_accept_or_reduce);
    assert!(has_eof_accept_or_reduce, "State 0 should have EOF Accept/Reduce");
    println!();
    
    // Test compression with the fix
    println!("5. Table compression (with fix):");
    let compressor = TableCompressor::new();
    let result = compressor.compress(&parse_table, &token_indices, has_eof_accept_or_reduce);
    
    match result {
        Ok(_) => println!("   ✅ Compression successful with nullable start!"),
        Err(e) => println!("   ❌ Compression failed: {:?}", e),
    }
    println!();
    
    println!("6. Key insight:");
    println!("   The fix correctly identifies that state 0 has Accept/Reduce actions");
    println!("   in the EOF column, indicating a nullable start symbol. This allows");
    println!("   proper handling of both empty files and files with content.");
    println!();
    
    println!("=== END DIAGNOSTIC ===\n");
}