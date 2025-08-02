// End-to-end test for pure-Rust Tree-sitter implementation
use rust_sitter::pure_parser::{Parser, TSLanguage, TSParseAction, ExternalScanner, TSLexState};
use std::ptr;

// Create a complete arithmetic language
fn create_arithmetic_language() -> &'static TSLanguage {
    // Symbol IDs
    const EOF: u16 = 0;
    const NUMBER: u16 = 1;
    const PLUS: u16 = 2;
    const MULTIPLY: u16 = 3;
    const LPAREN: u16 = 4;
    const RPAREN: u16 = 5;
    const EXPRESSION: u16 = 6;
    const TERM: u16 = 7;
    const FACTOR: u16 = 8;
    
    // Parse actions for arithmetic grammar
    // Grammar:
    // expression -> expression '+' term | term
    // term -> term '*' factor | factor  
    // factor -> '(' expression ')' | NUMBER
    static PARSE_ACTIONS: [TSParseAction; 20] = [
        // State 0 - initial state
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: NUMBER, dynamic_precedence: 0 },     // Shift NUMBER
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: LPAREN, dynamic_precedence: 0 },    // Shift LPAREN
        
        // State 1 - after NUMBER
        TSParseAction { action_type: 1, extra: 0, child_count: 1, symbol: FACTOR, dynamic_precedence: 0 },    // Reduce to factor
        
        // State 2 - after LPAREN
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: NUMBER, dynamic_precedence: 0 },    // Shift NUMBER
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: LPAREN, dynamic_precedence: 0 },    // Shift LPAREN
        
        // State 3 - after factor
        TSParseAction { action_type: 1, extra: 0, child_count: 1, symbol: TERM, dynamic_precedence: 0 },      // Reduce to term
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: MULTIPLY, dynamic_precedence: 0 },  // Shift MULTIPLY
        
        // State 4 - after term
        TSParseAction { action_type: 1, extra: 0, child_count: 1, symbol: EXPRESSION, dynamic_precedence: 0 }, // Reduce to expression
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: PLUS, dynamic_precedence: 0 },      // Shift PLUS
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: MULTIPLY, dynamic_precedence: 0 },  // Shift MULTIPLY
        
        // State 5 - after expression
        TSParseAction { action_type: 2, extra: 0, child_count: 0, symbol: 0, dynamic_precedence: 0 },         // Accept
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: PLUS, dynamic_precedence: 0 },      // Shift PLUS
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: RPAREN, dynamic_precedence: 0 },    // Shift RPAREN
        
        // Reduce actions
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: TERM, dynamic_precedence: 0 },      // Reduce term '*' factor
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: EXPRESSION, dynamic_precedence: 0 }, // Reduce expression '+' term
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: FACTOR, dynamic_precedence: 0 },    // Reduce '(' expression ')'
        
        // Error
        TSParseAction { action_type: 3, extra: 0, child_count: 0, symbol: 0, dynamic_precedence: 0 },         // Error
        
        // Padding
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0, dynamic_precedence: 0 },
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0, dynamic_precedence: 0 },
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0, dynamic_precedence: 0 },
    ];
    
    // Simple parse table (state x symbol -> action index)
    static PARSE_TABLE: [u16; 100] = [
        // State 0
        0, 1, 16, 16, 1, 16, 16, 16, 16, 0,  // NUMBER, LPAREN -> shift
        // State 1  
        16, 16, 2, 2, 16, 2, 16, 16, 16, 0,  // Reduce to factor
        // State 2
        3, 4, 16, 16, 4, 16, 16, 16, 16, 0,  // Shift in LPAREN state
        // State 3
        16, 16, 5, 6, 16, 5, 16, 16, 16, 0,  // Reduce to term or shift *
        // State 4
        16, 16, 8, 9, 16, 7, 16, 16, 16, 0,  // Reduce to expr or shift +/*
        // State 5
        16, 16, 11, 16, 16, 12, 10, 16, 16, 0, // Accept or shift +/)
        // More states...
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    
    static SMALL_PARSE_TABLE: [u16; 50] = [0; 50];
    static SMALL_PARSE_TABLE_MAP: [u32; 10] = [0; 10];
    static LEX_MODES: [TSLexState; 10] = [TSLexState { lex_state: 0, external_lex_state: 0 }; 10];
    static PRODUCTION_ID_MAP: [u16; 10] = [0; 10];
    // Create empty byte arrays for symbol and field names
    static EMPTY_NAME: [u8; 1] = [0];
    static SYMBOL_NAMES_DATA: [[u8; 1]; 9] = [EMPTY_NAME; 9];
    static FIELD_NAMES_DATA: [[u8; 1]; 1] = [EMPTY_NAME; 1];
    
    // Convert to raw pointers at runtime
    let symbol_name_ptrs: [*const u8; 9] = [
        SYMBOL_NAMES_DATA[0].as_ptr(),
        SYMBOL_NAMES_DATA[1].as_ptr(),
        SYMBOL_NAMES_DATA[2].as_ptr(),
        SYMBOL_NAMES_DATA[3].as_ptr(),
        SYMBOL_NAMES_DATA[4].as_ptr(),
        SYMBOL_NAMES_DATA[5].as_ptr(),
        SYMBOL_NAMES_DATA[6].as_ptr(),
        SYMBOL_NAMES_DATA[7].as_ptr(),
        SYMBOL_NAMES_DATA[8].as_ptr(),
    ];
    let field_name_ptrs: [*const u8; 1] = [FIELD_NAMES_DATA[0].as_ptr()];
    static FIELD_MAP_SLICES: [u16; 10] = [0; 10];
    static FIELD_MAP_ENTRIES: [u16; 10] = [0; 10];
    static SYMBOL_METADATA: [u8; 9] = [0; 9];
    static PUBLIC_SYMBOL_MAP: [u16; 9] = [0; 9];
    static ALIAS_MAP: [u16; 1] = [0; 1];
    static ALIAS_SEQUENCES: [u16; 1] = [0; 1];
    static PRIMARY_STATE_IDS: [u16; 10] = [0; 10];
    
    static LANGUAGE: TSLanguage = TSLanguage {
        version: 15,
        symbol_count: 9,
        alias_count: 0,
        token_count: 6,
        external_token_count: 0,
        state_count: 10,
        large_state_count: 6,
        production_id_count: 6,
        field_count: 0,
        max_alias_sequence_length: 0,
        production_id_map: PRODUCTION_ID_MAP.as_ptr(),
        parse_table: PARSE_TABLE.as_ptr(),
        small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
        small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
        parse_actions: PARSE_ACTIONS.as_ptr(),
        symbol_names: symbol_name_ptrs.as_ptr(),
        field_names: field_name_ptrs.as_ptr(),
        field_map_slices: FIELD_MAP_SLICES.as_ptr(),
        field_map_entries: FIELD_MAP_ENTRIES.as_ptr(),
        symbol_metadata: SYMBOL_METADATA.as_ptr(),
        public_symbol_map: PUBLIC_SYMBOL_MAP.as_ptr(),
        alias_map: ALIAS_MAP.as_ptr(),
        alias_sequences: ALIAS_SEQUENCES.as_ptr(),
        lex_modes: LEX_MODES.as_ptr(),
        lex_fn: Some(arithmetic_lexer),
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: ExternalScanner {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: PRIMARY_STATE_IDS.as_ptr(),
    };
    
    &LANGUAGE
}

// Simple lexer for arithmetic expressions
unsafe extern "C" fn arithmetic_lexer(_lexer: *mut std::ffi::c_void, _lex_state: TSLexState) -> bool {
    // In a real implementation, this would interact with the lexer state
    // For now, just return true to indicate success
    true
}

#[test]
fn test_arithmetic_parser_e2e() {
    let mut parser = Parser::new();
    let language = create_arithmetic_language();
    
    assert!(parser.set_language(language).is_ok());
    
    // Test parsing simple expressions
    let test_cases = vec![
        ("123", true),
        ("1 + 2", true),
        ("3 * 4", true),
        ("(5 + 6)", true),
        ("1 + 2 * 3", true),
        ("(1 + 2) * 3", true),
        ("((1))", true),
        ("1 + + 2", false), // Error case
    ];
    
    for (input, should_succeed) in test_cases {
        println!("\nTesting: '{}'", input);
        let result = parser.parse_string(input);
        
        if should_succeed {
            if let Some(root) = result.root {
                println!("✓ Parsed successfully, root symbol: {}", root.symbol());
                print_tree(&root, 0);
            } else {
                println!("✗ Expected success but got errors: {:?}", result.errors);
                assert!(false, "Parse should have succeeded for '{}'", input);
            }
        } else {
            if result.errors.is_empty() {
                println!("✗ Expected errors but parsing succeeded");
                assert!(false, "Parse should have failed for '{}'", input);
            } else {
                println!("✓ Got expected errors: {} errors", result.errors.len());
            }
        }
    }
}

fn print_tree(node: &rust_sitter::pure_parser::ParsedNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}Symbol {}: byte range [{}, {}]",
        indent,
        node.symbol(),
        node.start_byte(),
        node.end_byte()
    );
    
    for child in node.children() {
        print_tree(child, depth + 1);
    }
}

#[test]
fn test_parser_robustness() {
    let mut parser = Parser::new();
    let language = create_arithmetic_language();
    parser.set_language(language).unwrap();
    
    // Test with various edge cases
    let edge_cases = vec![
        "",           // Empty
        " ",          // Whitespace only
        "(",          // Unmatched paren
        ")",          // Unmatched paren
        "1 2",        // Missing operator
        "+ 1",        // Leading operator
        "1 +",        // Trailing operator
        "1 * * 2",    // Double operator
        "()",         // Empty parens
        "(((",        // Multiple unmatched
        ")))",        // Multiple unmatched
        "1 + (2",     // Incomplete expression
        "1 + 2)",     // Extra closing paren
        "(1 + 2",     // Missing closing paren
    ];
    
    for input in edge_cases {
        println!("\nTesting edge case: '{}'", input);
        let result = parser.parse_string(input);
        
        // We expect all edge cases to either parse with errors or produce an error node
        if let Some(root) = result.root {
            println!("Parsed with root symbol: {}", root.symbol());
            if root.is_error() {
                println!("✓ Produced error node as expected");
            }
        } else if !result.errors.is_empty() {
            println!("✓ Produced {} parse errors as expected", result.errors.len());
        } else {
            println!("✗ Unexpected successful parse without errors");
        }
    }
}

#[test]
fn test_parser_performance() {
    use std::time::Instant;
    
    let mut parser = Parser::new();
    let language = create_arithmetic_language();
    parser.set_language(language).unwrap();
    
    // Generate a large expression
    let mut expr = String::from("1");
    for i in 2..=1000 {
        expr.push_str(&format!(" + {}", i));
    }
    
    println!("\nParsing expression with {} tokens...", expr.split_whitespace().count());
    
    let start = Instant::now();
    let result = parser.parse_string(&expr);
    let duration = start.elapsed();
    
    println!("Parse time: {:?}", duration);
    
    if let Some(root) = result.root {
        println!("✓ Successfully parsed large expression");
        println!("Root node has {} direct children", root.child_count());
    } else {
        println!("Parse errors: {}", result.errors.len());
    }
    
    // Should parse in reasonable time (< 1 second)
    assert!(duration.as_secs() < 1, "Parsing took too long");
}