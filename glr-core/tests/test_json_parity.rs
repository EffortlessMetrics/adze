//! Test GLR parser against tree-sitter-json extracted tables
#![cfg(feature = "json-parity")]

use rust_sitter_glr_core::{
    Action, ActionCell, Driver, LexMode, ParseRule, ParseTable, SymbolMetadata,
};
use rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;
use std::fs;

#[test]
fn test_json_simple_object() {
    // Load the extracted JSON grammar tables
    let json_data = fs::read_to_string("/tmp/json-grammar.json")
        .expect("Run: cargo run -p ts-bridge --features with-grammars --bin extract-json > /tmp/json-grammar.json");

    let extracted: serde_json::Value =
        serde_json::from_str(&json_data).expect("Failed to parse extracted data");

    // Get basic metadata
    let symbol_count = extracted["symbol_count"].as_u64().unwrap() as usize;
    let state_count = extracted["state_count"].as_u64().unwrap() as usize;
    let token_count = extracted["token_count"].as_u64().unwrap() as usize;
    let external_count = extracted["external_token_count"].as_u64().unwrap_or(0) as usize;
    let terminal_boundary = token_count + external_count; // Terminals are [0..terminal_boundary)

    // Get start symbol from rules[0].lhs (Tree-sitter convention)
    let start_symbol = if let Some(rules) = extracted["rules"].as_array() {
        if let Some(first_rule) = rules.first() {
            SymbolId(first_rule["lhs"].as_u64().unwrap() as u16)
        } else {
            // Fallback to extracted value if no rules
            SymbolId(extracted["start_symbol"].as_u64().unwrap() as u16)
        }
    } else {
        SymbolId(extracted["start_symbol"].as_u64().unwrap() as u16)
    };

    let eof_symbol = SymbolId(extracted["eof_symbol"].as_u64().unwrap() as u16);

    println!(
        "JSON grammar: {} symbols, {} states",
        symbol_count, state_count
    );
    println!(
        "Token count: {}, External count: {}, Terminal boundary: {}",
        token_count, external_count, terminal_boundary
    );
    println!(
        "Start symbol: {}, EOF symbol: {}",
        start_symbol.0, eof_symbol.0
    );

    // Build dense action table (state x symbol -> Vec<Action>)
    // Actions are only for terminals [0..terminal_boundary)
    let mut action_table: Vec<Vec<ActionCell>> = vec![vec![vec![]; symbol_count]; state_count];

    if let Some(action_cells) = extracted["actions"].as_array() {
        for cell in action_cells {
            let state = cell["state"].as_u64().unwrap() as usize;
            let symbol = cell["symbol"].as_u64().unwrap() as usize; // This is already a terminal index

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
                        "V" => {
                            // Recover action (Tree-sitter error recovery)
                            action_table[state][symbol].push(Action::Recover);
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
    let mut goto_table: Vec<Vec<StateId>> =
        vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    let mut nonterminal_to_index = BTreeMap::new();

    if let Some(goto_cells) = extracted["gotos"].as_array() {
        for cell in goto_cells {
            let state = cell["state"].as_u64().unwrap() as usize;
            let symbol = cell["symbol"].as_u64().unwrap() as usize; // This is a nonterminal index
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
                is_terminal: sym["type"].as_str() == Some("TERMINAL"),
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(symbol_metadata.len() as u16),
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
    let rule_count = rules.len();
    let parse_table = ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol: vec![],
        external_scanner_states: vec![vec![false; symbol_count]; state_count],
        rules,
        nonterminal_to_index,
        eof_symbol,
        start_symbol,
        grammar,
        initial_state: StateId(1), // Tree-sitter uses state 1 as initial (state 0 is error recovery)
        token_count,
        external_token_count: external_count,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            state_count
        ],
        extras: vec![], // TODO: Extract from tree-sitter
        dynamic_prec_by_rule: vec![0; rule_count],
        rule_assoc_by_rule: vec![0; rule_count],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: rust_sitter_glr_core::GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
    };

    // Create driver and parse some JSON
    let mut driver = Driver::new(&parse_table);

    // Map symbol names to IDs for robust tokenization
    let symbol_names: Vec<String> = if let Some(symbols) = extracted["symbols"].as_array() {
        symbols
            .iter()
            .map(|s| s["name"].as_str().unwrap_or("").to_string())
            .collect()
    } else {
        vec![]
    };

    /// Map a JSON input into the terminal sequence expected by Tree-sitter JSON
    /// (as extracted by ts-bridge).
    ///
    /// - `symbol_names`: the names array from the extracted JSON
    /// - `term_boundary`: token_count + external_token_count (terminals are < term_boundary)
    fn tokenize_json_ts_terminals(
        input: &str,
        symbol_names: &[String],
        term_boundary: usize,
    ) -> Vec<(u32, u32, u32)> {
        // Helper: ID by exact name; ensure it's a terminal ID (< term_boundary)
        let id = |name: &str| -> Option<u32> {
            symbol_names
                .iter()
                .position(|s| s == name)
                .filter(|&i| i < term_boundary)
                .map(|i| i as u32)
        };

        // Common JSON terminals
        let lbrace = id("{");
        let rbrace = id("}");
        let lbrack = id("[");
        let rbrack = id("]");
        let colon = id(":");
        let comma = id(",");

        // Scalars
        let number = id("number");
        let kw_true = id("true");
        let kw_false = id("false");
        let kw_null = id("null");

        // Strings: many TS JSON grammars use '"' + string_content* + '"'
        let quote = id("\""); // opening/closing quote
        let str_cont = id("string_content"); // optional; emit only if present and content non-empty

        let b = input.as_bytes();
        let mut pos = 0usize;
        let mut toks: Vec<(u32, u32, u32)> = Vec::new();

        let is_ws = |x: u8| matches!(x, b' ' | b'\t' | b'\n' | b'\r');

        while pos < b.len() {
            // Skip extras (whitespace; comments if you want to support them later)
            while pos < b.len() && is_ws(b[pos]) {
                pos += 1;
            }
            if pos >= b.len() {
                break;
            }

            let start = pos;
            match b[pos] as char {
                '{' => {
                    if let Some(t) = lbrace {
                        toks.push((t, start as u32, (start + 1) as u32));
                    }
                    pos += 1;
                }
                '}' => {
                    if let Some(t) = rbrace {
                        toks.push((t, start as u32, (start + 1) as u32));
                    }
                    pos += 1;
                }
                '[' => {
                    if let Some(t) = lbrack {
                        toks.push((t, start as u32, (start + 1) as u32));
                    }
                    pos += 1;
                }
                ']' => {
                    if let Some(t) = rbrack {
                        toks.push((t, start as u32, (start + 1) as u32));
                    }
                    pos += 1;
                }
                ':' => {
                    if let Some(t) = colon {
                        toks.push((t, start as u32, (start + 1) as u32));
                    }
                    pos += 1;
                }
                ',' => {
                    if let Some(t) = comma {
                        toks.push((t, start as u32, (start + 1) as u32));
                    }
                    pos += 1;
                }

                '"' => {
                    // Simple string: '"' [string_content] '"'
                    pos += 1;
                    let content_start = pos;
                    while pos < b.len() {
                        if b[pos] == b'"' && (pos == 0 || b[pos - 1] != b'\\') {
                            break;
                        }
                        pos += 1;
                    }
                    let content_end = pos.min(b.len());

                    if let Some(t) = quote {
                        toks.push((t, start as u32, (start + 1) as u32));
                    }
                    if let Some(sc) = str_cont {
                        if content_end > content_start {
                            toks.push((sc, content_start as u32, content_end as u32));
                        }
                    }
                    if pos < b.len() && b[pos] == b'"' {
                        if let Some(t) = quote {
                            toks.push((t, pos as u32, (pos + 1) as u32));
                        }
                        pos += 1;
                    }
                }

                't' if b[start..].starts_with(b"true") => {
                    if let Some(t) = kw_true {
                        toks.push((t, start as u32, (start + 4) as u32));
                    }
                    pos = start + 4;
                }
                'f' if b[start..].starts_with(b"false") => {
                    if let Some(t) = kw_false {
                        toks.push((t, start as u32, (start + 5) as u32));
                    }
                    pos = start + 5;
                }
                'n' if b[start..].starts_with(b"null") => {
                    if let Some(t) = kw_null {
                        toks.push((t, start as u32, (start + 4) as u32));
                    }
                    pos = start + 4;
                }

                '-' | '0'..='9' => {
                    pos += 1;
                    while pos < b.len() {
                        let c = b[pos];
                        if c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-') {
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    if let Some(t) = number {
                        toks.push((t, start as u32, pos as u32));
                    }
                }

                _ => {
                    pos += 1;
                } // skip unknowns
            }
        }

        toks
    }

    // Test parsing empty object (simpler than strings)
    let input = r#"{}"#;
    let tokens = tokenize_json_ts_terminals(input, &symbol_names, terminal_boundary);

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
        println!(
            "\nFirst token is {} ({})",
            first_tok,
            symbol_names.get(first_tok).unwrap_or(&"?".to_string())
        );
        if !parse_table.action_table[1][first_tok].is_empty() {
            println!(
                "  State 1 has actions for this token: {:?}",
                parse_table.action_table[1][first_tok]
            );
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
