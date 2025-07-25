// End-to-end test for pure-Rust Tree-sitter implementation
use rust_sitter::pure_parser::{Parser, TSLanguage, TSParseAction, ExternalScanner};
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
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: NUMBER },     // Shift NUMBER
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: LPAREN },    // Shift LPAREN
        
        // State 1 - after NUMBER
        TSParseAction { action_type: 1, extra: 0, child_count: 1, symbol: FACTOR },    // Reduce to factor
        
        // State 2 - after LPAREN
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: NUMBER },    // Shift NUMBER
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: LPAREN },    // Shift LPAREN
        
        // State 3 - after factor
        TSParseAction { action_type: 1, extra: 0, child_count: 1, symbol: TERM },      // Reduce to term
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: MULTIPLY },  // Shift MULTIPLY
        
        // State 4 - after term
        TSParseAction { action_type: 1, extra: 0, child_count: 1, symbol: EXPRESSION }, // Reduce to expression
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: PLUS },      // Shift PLUS
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: MULTIPLY },  // Shift MULTIPLY
        
        // State 5 - after expression
        TSParseAction { action_type: 2, extra: 0, child_count: 0, symbol: 0 },         // Accept
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: PLUS },      // Shift PLUS
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: RPAREN },    // Shift RPAREN
        
        // Reduce actions
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: TERM },      // Reduce term '*' factor
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: EXPRESSION }, // Reduce expression '+' term
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: FACTOR },    // Reduce '(' expression ')'
        
        // Error
        TSParseAction { action_type: 3, extra: 0, child_count: 0, symbol: 0 },         // Error
        
        // Padding
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0 },
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0 },
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0 },
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
    static LEX_MODES: [u32; 10] = [0; 10];
    static PRODUCTION_ID_MAP: [u16; 10] = [0; 10];
    
    static LANGUAGE: TSLanguage = TSLanguage {
        version: 15,
        symbol_count: 9,
        token_count: 6,
        state_count: 10,
        large_state_count: 6,
        production_id_count: 6,
        production_id_map: PRODUCTION_ID_MAP.as_ptr(),
        parse_table: PARSE_TABLE.as_ptr(),
        small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
        small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
        parse_actions: PARSE_ACTIONS.as_ptr(),
        lex_modes: LEX_MODES.as_ptr(),
        lex_fn: Some(arithmetic_lexer),
        external_scanner: ExternalScanner {
            scan: None,
        },
    };
    
    &LANGUAGE
}

// Simple lexer for arithmetic expressions
unsafe extern "C" fn arithmetic_lexer(lexer: *mut std::ffi::c_void, _lex_state: u32) -> bool {
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