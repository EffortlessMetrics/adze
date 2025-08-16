//! Debug test for JSON parsing issues
#![cfg(feature = "json-parity")]

use rust_sitter_ir::SymbolId;
use std::fs;

#[test]
fn test_json_empty_object_debug() {
    // Load the extracted JSON parse table
    let json_data =
        fs::read_to_string("/tmp/json-grammar.json").expect("Failed to read JSON grammar");
    let extracted: serde_json::Value =
        serde_json::from_str(&json_data).expect("Failed to parse JSON");

    // Extract counts
    let state_count = extracted["state_count"].as_u64().unwrap() as usize;
    let symbol_count = extracted["symbol_count"].as_u64().unwrap() as usize;
    let token_count = extracted["token_count"].as_u64().unwrap() as usize;
    let external_count = extracted["external_token_count"].as_u64().unwrap_or(0) as usize;

    // Get start symbol from rules[0].lhs
    let start_symbol = if let Some(rules) = extracted["rules"].as_array() {
        if let Some(first_rule) = rules.first() {
            SymbolId(first_rule["lhs"].as_u64().unwrap() as u16)
        } else {
            SymbolId(15)
        }
    } else {
        SymbolId(15)
    };

    let eof_symbol = SymbolId(extracted["eof_symbol"].as_u64().unwrap() as u16);

    println!("=== JSON Grammar Info ===");
    println!("State count: {}", state_count);
    println!("Symbol count: {}", symbol_count);
    println!("Token count: {}", token_count);
    println!("Start symbol: {} (expected document/15)", start_symbol.0);
    println!("EOF symbol: {}", eof_symbol.0);

    // Extract actions for debugging
    println!("\n=== Key States for {} Parsing ===", "{}");

    // State 1 (initial)
    println!("\nState 1 (initial):");
    if let Some(actions) = extracted["actions"].as_array() {
        for action in actions {
            if action["state"].as_u64() == Some(1) {
                let sym = action["symbol"].as_u64().unwrap();
                if sym == 1 {
                    // { symbol
                    println!("  Symbol {} ({}) -> {:?}", sym, "{", action["actions"]);
                }
            }
        }
    }

    // State 16 (after {)
    println!("\nState 16 (after '{{'):");
    if let Some(actions) = extracted["actions"].as_array() {
        for action in actions {
            if action["state"].as_u64() == Some(16) {
                let sym = action["symbol"].as_u64().unwrap();
                if sym == 2 {
                    // } symbol
                    println!("  Symbol {} (}}) -> {:?}", sym, action["actions"]);
                }
            }
        }
    }

    // State 17 (after {})
    println!("\nState 17 (after '{{}}'):");
    if let Some(actions) = extracted["actions"].as_array() {
        for action in actions {
            if action["state"].as_u64() == Some(17) {
                let sym = action["symbol"].as_u64().unwrap();
                println!("  Symbol {} -> {:?}", sym, action["actions"]);
            }
        }
    }

    // Look for reduce rules that could apply
    println!("\n=== Relevant Rules ===");
    if let Some(rules) = extracted["rules"].as_array() {
        for (i, rule) in rules.iter().enumerate() {
            let lhs = rule["lhs"].as_u64().unwrap();
            let rhs_len = rule["rhs_len"].as_u64().unwrap();

            // Look for rules that could match {}
            if lhs == 17 && rhs_len == 2 {
                // object -> { }
                println!(
                    "Rule {}: Symbol {} with {} children (object -> {{ }})",
                    i, lhs, rhs_len
                );
            }
            if lhs == 16 {
                // value rules
                println!(
                    "Rule {}: Symbol {} with {} children (value rule)",
                    i, lhs, rhs_len
                );
            }
            if lhs == 15 {
                // document rules
                println!(
                    "Rule {}: Symbol {} with {} children (document rule)",
                    i, lhs, rhs_len
                );
            }
        }
    }

    // Now trace through what SHOULD happen
    println!("\n=== Expected Parse Sequence for '{{}}' ===");
    println!("1. Start in state 1");
    println!("2. See '{{', shift to state 16");
    println!("3. See '}}', shift to state 17");
    println!("4. Now we have consumed '{{}}' and are in state 17");
    println!("5. Need to reduce to create object node");
    println!("6. After reducing object, need to reduce to value");
    println!("7. After reducing value, need to reduce to document");
    println!("8. Accept on EOF");

    // Check what symbols correspond to what
    println!("\n=== Symbol Mapping ===");
    if let Some(symbols) = extracted["symbols"].as_array() {
        for (i, sym) in symbols.iter().enumerate() {
            if i <= 20 || i == start_symbol.0 as usize {
                println!("Symbol {}: {}", i, sym["name"].as_str().unwrap_or("?"));
            }
        }
    }
}
