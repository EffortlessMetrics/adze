//! Test GLR parser against tree-sitter-json extracted tables

use rust_sitter_glr_core::{
    Driver, ParseTable, Action, ParseRule, SymbolMetadata, ActionCell
};
use rust_sitter_ir::{StateId, SymbolId, RuleId, Grammar};
use std::fs;
use std::collections::BTreeMap;

#[test]
fn test_json_simple_object() {
    // Load the extracted JSON grammar tables
    let json_data = fs::read_to_string("/tmp/json-grammar.json")
        .expect("Run: cargo run -p ts-bridge --features with-grammars --bin extract-json > /tmp/json-grammar.json");
    
    let extracted: serde_json::Value = serde_json::from_str(&json_data)
        .expect("Failed to parse extracted data");
    
    // Get basic metadata
    let symbol_count = extracted["symbol_count"].as_u64().unwrap() as usize;
    let state_count = extracted["state_count"].as_u64().unwrap() as usize;
    let start_symbol = SymbolId(extracted["start_symbol"].as_u64().unwrap() as u16);
    let eof_symbol = SymbolId(extracted["eof_symbol"].as_u64().unwrap() as u16);
    
    println!("JSON grammar: {} symbols, {} states", symbol_count, state_count);
    println!("Start symbol: {}, EOF symbol: {}", start_symbol.0, eof_symbol.0);
    
    // Build action table (state x symbol -> Vec<Action>)
    let mut action_table: Vec<Vec<ActionCell>> = vec![vec![vec![]; symbol_count]; state_count];
    
    if let Some(action_cells) = extracted["actions"].as_array() {
        for cell in action_cells {
            let state = cell["state"].as_u64().unwrap() as usize;
            let symbol = cell["symbol"].as_u64().unwrap() as usize;
            
            if let Some(cell_actions) = cell["actions"].as_array() {
                for action in cell_actions {
                    let action_type = action["k"].as_str().unwrap();
                    match action_type {
                        "S" => {
                            let next_state = StateId(action["state"].as_u64().unwrap() as u16);
                            action_table[state][symbol].push(Action::Shift(next_state));
                        }
                        "R" => {
                            let rule = RuleId(action["rule"].as_u64().unwrap() as u16);
                            action_table[state][symbol].push(Action::Reduce(rule));
                        }
                        "A" => {
                            action_table[state][symbol].push(Action::Accept);
                        }
                        "E" => {
                            // Error recovery action - skip for now
                        }
                        _ => panic!("Unknown action type: {}", action_type),
                    }
                }
            }
        }
    }
    
    // Build goto table (for nonterminals only)
    let mut goto_table: Vec<Vec<StateId>> = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    let mut nonterminal_to_index = BTreeMap::new();
    
    if let Some(goto_cells) = extracted["gotos"].as_array() {
        for cell in goto_cells {
            let state = cell["state"].as_u64().unwrap() as usize;
            let symbol = cell["symbol"].as_u64().unwrap() as usize;
            
            if let Some(next) = cell["next_state"].as_u64() {
                goto_table[state][symbol] = StateId(next as u16);
                // Track nonterminal indices
                nonterminal_to_index.insert(SymbolId(symbol as u16), symbol);
            }
        }
    }
    
    // Build rules
    let mut rules = Vec::new();
    if let Some(rule_list) = extracted["rules"].as_array() {
        for rule in rule_list {
            let lhs = SymbolId(rule["lhs"].as_u64().unwrap() as u16);
            let rhs_len = rule["rhs_len"].as_u64().unwrap() as u16;
            rules.push(ParseRule { lhs, rhs_len });
        }
    }
    
    // Build symbol metadata
    let mut symbol_metadata = Vec::new();
    if let Some(symbols) = extracted["symbols"].as_array() {
        for sym in symbols {
            symbol_metadata.push(SymbolMetadata {
                name: sym["name"].as_str().unwrap_or("").to_string(),
                visible: sym["visible"].as_bool().unwrap_or(false),
                named: sym["named"].as_bool().unwrap_or(false),
                supertype: false,
            });
        }
    }
    
    // Build symbol_to_index map
    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    
    // Create a minimal Grammar (required by ParseTable but not really used in our test)
    let grammar = Grammar::new("json".to_string());
    
    // Create ParseTable
    let parse_table = ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count,
        symbol_count,
        symbol_to_index,
        external_scanner_states: vec![vec![false; symbol_count]; state_count],
        rules,
        nonterminal_to_index,
        eof_symbol,
        start_symbol,
        grammar,
    };
    
    // Create driver and parse some JSON
    let mut driver = Driver::new(&parse_table);
    
    // Simple JSON tokenizer that returns token stream
    fn tokenize_json(input: &str) -> Vec<(u32, u32, u32)> {
        let mut tokens = Vec::new();
        let bytes = input.as_bytes();
        let mut pos = 0;
        
        while pos < bytes.len() {
            // Skip whitespace
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            
            if pos >= bytes.len() {
                break;
            }
            
            let start = pos;
            let ch = bytes[pos] as char;
            
            // Map to tree-sitter-json token IDs (from extracted data)
            match ch {
                '{' => {
                    tokens.push((2, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                '}' => {
                    tokens.push((3, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                '[' => {
                    tokens.push((4, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                ']' => {
                    tokens.push((5, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                ':' => {
                    tokens.push((6, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                ',' => {
                    tokens.push((7, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                '"' => {
                    // String token - find closing quote
                    pos += 1;
                    while pos < bytes.len() {
                        if bytes[pos] == b'"' && (pos == 0 || bytes[pos-1] != b'\\') {
                            tokens.push((8, start as u32, (pos + 1) as u32)); // string
                            pos += 1;
                            break;
                        }
                        pos += 1;
                    }
                }
                't' if bytes[start..].starts_with(b"true") => {
                    tokens.push((11, start as u32, (start + 4) as u32)); // true
                    pos += 4;
                }
                'f' if bytes[start..].starts_with(b"false") => {
                    tokens.push((12, start as u32, (start + 5) as u32)); // false
                    pos += 5;
                }
                'n' if bytes[start..].starts_with(b"null") => {
                    tokens.push((13, start as u32, (start + 4) as u32)); // null
                    pos += 4;
                }
                '-' | '0'..='9' => {
                    // Number token
                    while pos < bytes.len() && (bytes[pos].is_ascii_digit() || 
                           bytes[pos] == b'.' || bytes[pos] == b'e' || 
                           bytes[pos] == b'E' || bytes[pos] == b'-' || 
                           bytes[pos] == b'+') {
                        pos += 1;
                    }
                    tokens.push((10, start as u32, pos as u32)); // number
                }
                _ => {
                    pos += 1; // Skip unknown characters
                }
            }
        }
        
        // Add EOF token
        tokens.push((0, pos as u32, pos as u32));
        tokens
    }
    
    // Test parsing a simple JSON object
    let input = r#"{"key": "value"}"#;
    let tokens = tokenize_json(input);
    
    println!("Tokens: {:?}", tokens);
    
    // Debug: check action table for state 0
    println!("\nState 0 actions:");
    for (sym, actions) in action_table[0].iter().enumerate() {
        if !actions.is_empty() {
            println!("  Symbol {}: {:?}", sym, actions);
        }
    }
    
    match driver.parse_tokens(tokens) {
        Ok(forest) => {
            println!("Successfully parsed JSON!");
            // The forest is already a trait object, so we can't inspect it directly
            // but the test passes if we get here
        }
        Err(e) => {
            panic!("Failed to parse JSON: {:?}", e);
        }
    }
}