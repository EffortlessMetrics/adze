//! Test streaming lexer implementation for Tree-sitter compatibility

use adze_glr_core::ts_lexer::NextToken;
use adze_glr_core::{Driver, LexMode, ParseTable};
use adze_ir::{StateId, SymbolId};
use std::collections::BTreeMap;

/// Simple test lexer that mimics Tree-sitter's JSON lexer
fn json_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    if pos >= bytes.len() {
        return None;
    }

    // Skip whitespace (Tree-sitter lexer does this internally)
    let mut p = pos;
    while p < bytes.len() && matches!(bytes[p], b' ' | b'\t' | b'\n' | b'\r') {
        p += 1;
    }

    if p >= bytes.len() {
        return None;
    }

    // Simple JSON tokens
    let start = p;
    let ch = bytes[p];

    match ch {
        b'{' => Some(NextToken {
            kind: 1,
            start: start as u32,
            end: (start + 1) as u32,
        }), // {
        b'}' => Some(NextToken {
            kind: 2,
            start: start as u32,
            end: (start + 1) as u32,
        }), // }
        b'[' => Some(NextToken {
            kind: 3,
            start: start as u32,
            end: (start + 1) as u32,
        }), // [
        b']' => Some(NextToken {
            kind: 4,
            start: start as u32,
            end: (start + 1) as u32,
        }), // ]
        b':' => Some(NextToken {
            kind: 5,
            start: start as u32,
            end: (start + 1) as u32,
        }), // :
        b',' => Some(NextToken {
            kind: 6,
            start: start as u32,
            end: (start + 1) as u32,
        }), // ,
        b'"' => {
            // Simple string scanning
            let mut end = start + 1;
            while end < bytes.len() && bytes[end] != b'"' {
                if bytes[end] == b'\\' && end + 1 < bytes.len() {
                    end += 2; // Skip escaped char
                } else {
                    end += 1;
                }
            }
            if end < bytes.len() {
                end += 1; // Include closing quote
            }
            Some(NextToken {
                kind: 11,
                start: start as u32,
                end: end as u32,
            }) // string
        }
        b't' if bytes[p..].starts_with(b"true") => {
            Some(NextToken {
                kind: 8,
                start: start as u32,
                end: (start + 4) as u32,
            }) // true
        }
        b'f' if bytes[p..].starts_with(b"false") => {
            Some(NextToken {
                kind: 9,
                start: start as u32,
                end: (start + 5) as u32,
            }) // false
        }
        b'n' if bytes[p..].starts_with(b"null") => {
            Some(NextToken {
                kind: 10,
                start: start as u32,
                end: (start + 4) as u32,
            }) // null
        }
        b'0'..=b'9' | b'-' => {
            // Simple number scanning
            let mut end = start;
            if bytes[end] == b'-' {
                end += 1;
            }
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end < bytes.len() && bytes[end] == b'.' {
                end += 1;
                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }
            }
            if end < bytes.len() && matches!(bytes[end], b'e' | b'E') {
                end += 1;
                if end < bytes.len() && matches!(bytes[end], b'+' | b'-') {
                    end += 1;
                }
                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }
            }
            Some(NextToken {
                kind: 12,
                start: start as u32,
                end: end as u32,
            }) // number
        }
        _ => None,
    }
}

#[test]
#[ignore = "WIP: uses stub lexer; keep committed but skipped"]
fn test_streaming_json_parser() {
    // Load the extracted JSON parse table
    let json_data =
        std::fs::read_to_string("/tmp/json-grammar.json").expect("Failed to read JSON grammar");
    let extracted: serde_json::Value =
        serde_json::from_str(&json_data).expect("Failed to parse JSON");

    // Extract basic counts
    let state_count = extracted["state_count"].as_u64().unwrap() as usize;
    let symbol_count = extracted["symbol_count"].as_u64().unwrap() as usize;
    let token_count = extracted["token_count"].as_u64().unwrap() as usize;
    let external_count = extracted["external_token_count"].as_u64().unwrap_or(0) as usize;

    // Get start symbol from rules[0].lhs
    let start_symbol = if let Some(rules) = extracted["rules"].as_array() {
        if let Some(first_rule) = rules.first() {
            SymbolId(first_rule["lhs"].as_u64().unwrap() as u16)
        } else {
            SymbolId(15) // Fallback to document
        }
    } else {
        SymbolId(15)
    };

    let eof_symbol = SymbolId(extracted["eof_symbol"].as_u64().unwrap() as u16);

    println!(
        "JSON grammar: {} symbols, {} states",
        symbol_count, state_count
    );
    println!(
        "Start symbol: {}, EOF symbol: {}",
        start_symbol.0, eof_symbol.0
    );

    // Build minimal parse table for testing
    // (In real usage, this would be extracted from the JSON)
    let parse_table = ParseTable {
        action_table: vec![], // Would be filled from extracted data
        goto_table: vec![],   // Would be filled from extracted data
        rules: vec![],        // Would be filled from extracted data
        state_count,
        symbol_count,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol,
        grammar: adze_ir::Grammar::new("json".to_string()),
        initial_state: StateId(1), // Tree-sitter uses 1 (0 is error recovery)
        token_count,
        external_token_count: external_count,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        symbol_metadata: vec![], // Would be filled from extracted data
    };

    // Test parsing with streaming lexer
    let mut driver = Driver::new(&parse_table);

    // Test simple JSON objects
    let test_cases = vec![
        "{}",
        "[]",
        r#"{"key": "value"}"#,
        r#"[1, 2, 3]"#,
        r#"{"nested": {"object": true}}"#,
    ];

    for input in test_cases {
        println!("\nTesting: {}", input);

        // Note: We can't actually run this without the full parse table,
        // but this shows the API usage
        let result = driver.parse_streaming(
            input,
            json_lexer,
            None::<fn(&str, usize, &[bool], LexMode) -> Option<NextToken>>,
        );

        match result {
            Ok(_forest) => println!("✓ Successfully parsed"),
            Err(e) => println!("✗ Parse error: {}", e),
        }
    }
}
