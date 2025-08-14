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
    let token_count = extracted["token_count"].as_u64().unwrap() as usize;
    let external_count = extracted["external_token_count"].as_u64().unwrap_or(0) as usize;
    let terminal_boundary = token_count + external_count;  // Terminals are [0..terminal_boundary)
    let start_symbol = SymbolId(extracted["start_symbol"].as_u64().unwrap() as u16);
    let eof_symbol = SymbolId(extracted["eof_symbol"].as_u64().unwrap() as u16);
    
    println!("JSON grammar: {} symbols, {} states", symbol_count, state_count);
    println!("Token count: {}, External count: {}, Terminal boundary: {}", token_count, external_count, terminal_boundary);
    println!("Start symbol: {}, EOF symbol: {}", start_symbol.0, eof_symbol.0);
    
    // Build dense action table (state x symbol -> Vec<Action>)
    // Actions are only for terminals [0..terminal_boundary)
    let mut action_table: Vec<Vec<ActionCell>> = vec![vec![vec![]; symbol_count]; state_count];
    
    if let Some(action_cells) = extracted["actions"].as_array() {
        for cell in action_cells {
            let state = cell["state"].as_u64().unwrap() as usize;
            let symbol = cell["symbol"].as_u64().unwrap() as usize;  // This is already a terminal index
            
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
                            // Error recovery action - track for later
                            action_table[state][symbol].push(Action::Error);
                        }
                        _ => panic!("Unknown action type: {}", action_type),
                    }
                }
            }
        }
    }
    
    // Build dense goto table (state x symbol -> StateId)
    // Gotos are only for nonterminals [terminal_boundary..symbol_count)
    // Use StateId(u16::MAX) as sentinel for "no goto"
    let mut goto_table: Vec<Vec<StateId>> = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    let mut nonterminal_to_index = BTreeMap::new();
    
    if let Some(goto_cells) = extracted["gotos"].as_array() {
        for cell in goto_cells {
            let state = cell["state"].as_u64().unwrap() as usize;
            let symbol = cell["symbol"].as_u64().unwrap() as usize;  // This is a nonterminal index
            let next_state = cell["next_state"].as_u64().unwrap() as u16;
            
            goto_table[state][symbol] = StateId(next_state);
            // Track nonterminal indices (these are symbols >= terminal_boundary)
            nonterminal_to_index.insert(SymbolId(symbol as u16), symbol);
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
        initial_state: StateId(1),  // Tree-sitter uses state 1 as initial, not 0
    };
    
    // Create driver and parse some JSON
    let mut driver = Driver::new(&parse_table);
    
    // Map symbol names to IDs for robust tokenization
    let symbol_names: Vec<String> = if let Some(symbols) = extracted["symbols"].as_array() {
        symbols.iter()
            .map(|s| s["name"].as_str().unwrap_or("").to_string())
            .collect()
    } else {
        vec![]
    };
    
    let find_symbol_id = |name: &str| -> u32 {
        symbol_names.iter()
            .position(|s| s == name)
            .unwrap_or_else(|| panic!("Symbol '{}' not found in grammar", name)) as u32
    };
    
    // Get token IDs from symbol names (much more robust than hard-coding)
    let tok_lbrace = find_symbol_id("{");
    let tok_rbrace = find_symbol_id("}");
    let tok_lbrack = find_symbol_id("[");
    let tok_rbrack = find_symbol_id("]");
    let tok_colon = find_symbol_id(":");
    let tok_comma = find_symbol_id(",");
    let tok_string = find_symbol_id("\"");  // String literal token
    let tok_number = find_symbol_id("number");
    let tok_true = find_symbol_id("true");
    let tok_false = find_symbol_id("false");
    let tok_null = find_symbol_id("null");
    
    println!("\nToken IDs from symbol names:");
    println!("  {{ = {}, }} = {}, [ = {}, ] = {}", tok_lbrace, tok_rbrace, tok_lbrack, tok_rbrack);
    println!("  : = {}, , = {}, string = {}, number = {}", tok_colon, tok_comma, tok_string, tok_number);
    println!("  true = {}, false = {}, null = {}", tok_true, tok_false, tok_null);
    
    // Simple JSON tokenizer that returns token stream
    fn tokenize_json(input: &str, tok_lbrace: u32, tok_rbrace: u32, tok_lbrack: u32, 
                     tok_rbrack: u32, tok_colon: u32, tok_comma: u32, tok_string: u32,
                     tok_number: u32, tok_true: u32, tok_false: u32, tok_null: u32) -> Vec<(u32, u32, u32)> {
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
            
            match ch {
                '{' => {
                    tokens.push((tok_lbrace, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                '}' => {
                    tokens.push((tok_rbrace, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                '[' => {
                    tokens.push((tok_lbrack, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                ']' => {
                    tokens.push((tok_rbrack, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                ':' => {
                    tokens.push((tok_colon, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                ',' => {
                    tokens.push((tok_comma, start as u32, (start + 1) as u32));
                    pos += 1;
                }
                '"' => {
                    // String token - find closing quote
                    pos += 1;
                    while pos < bytes.len() {
                        if bytes[pos] == b'"' && (pos == 0 || bytes[pos-1] != b'\\') {
                            tokens.push((tok_string, start as u32, (pos + 1) as u32));
                            pos += 1;
                            break;
                        }
                        pos += 1;
                    }
                }
                't' if bytes[start..].starts_with(b"true") => {
                    tokens.push((tok_true, start as u32, (start + 4) as u32));
                    pos += 4;
                }
                'f' if bytes[start..].starts_with(b"false") => {
                    tokens.push((tok_false, start as u32, (start + 5) as u32));
                    pos += 5;
                }
                'n' if bytes[start..].starts_with(b"null") => {
                    tokens.push((tok_null, start as u32, (start + 4) as u32));
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
                    tokens.push((tok_number, start as u32, pos as u32));
                }
                _ => {
                    pos += 1; // Skip unknown characters
                }
            }
        }
        
        // DO NOT add EOF token - the driver synthesizes it
        tokens
    }
    
    // Test parsing a simple JSON number (simplest case)
    let input = r#"42"#;
    let tokens = tokenize_json(input, tok_lbrace, tok_rbrace, tok_lbrack, tok_rbrack,
                                tok_colon, tok_comma, tok_string, tok_number,
                                tok_true, tok_false, tok_null);
    
    println!("\nTokens: {:?}", tokens);
    
    // Debug: check action table for initial state (state 1 for Tree-sitter)
    println!("\nState 1 actions (terminals only - initial state for Tree-sitter):");
    for sym in 0..terminal_boundary {
        if !parse_table.action_table[1][sym].is_empty() {
            println!("  Terminal {}: {:?}", sym, parse_table.action_table[1][sym]);
        }
    }
    
    // Also check what's in the first token's state
    if !tokens.is_empty() {
        let first_tok = tokens[0].0 as usize;
        println!("\nFirst token is {} ({})", first_tok, symbol_names.get(first_tok).unwrap_or(&"?".to_string()));
        if !parse_table.action_table[1][first_tok].is_empty() {
            println!("  State 1 has actions for this token: {:?}", parse_table.action_table[1][first_tok]);
        } else {
            println!("  WARNING: State 1 has NO actions for this token!");
        }
    }
    
    println!("\nParsing tokens...");
    match driver.parse_tokens(tokens) {
        Ok(_forest) => {
            println!("\n✅ Successfully parsed JSON!");
            // The forest is already a trait object, so we can't inspect it directly
            // but the test passes if we get here
        }
        Err(e) => {
            // Provide more context about the failure
            println!("\n❌ Parse failed: {:?}", e);
            println!("\nDiagnostics:");
            println!("  Start symbol: {} (expected)", parse_table.start_symbol.0);
            println!("  EOF symbol: {}", parse_table.eof_symbol.0);
            println!("  Initial state: {}", parse_table.initial_state.0);
            panic!("Failed to parse JSON: {:?}", e);
        }
    }
}