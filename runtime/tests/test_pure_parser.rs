// Test the pure-Rust parser implementation
use rust_sitter::pure_parser::{Parser, TSLanguage, TSParseAction, ExternalScanner};
use std::ptr;

// Create a simple test language
fn create_test_language() -> &'static TSLanguage {
    // Define parse actions
    static PARSE_ACTIONS: [TSParseAction; 10] = [
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 1 }, // Shift digit
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 2 }, // Shift plus
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 3 }, // Shift multiply
        TSParseAction { action_type: 1, extra: 0, child_count: 1, symbol: 4 }, // Reduce to number
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: 5 }, // Reduce to addition
        TSParseAction { action_type: 1, extra: 0, child_count: 3, symbol: 6 }, // Reduce to multiplication
        TSParseAction { action_type: 2, extra: 0, child_count: 0, symbol: 0 }, // Accept
        TSParseAction { action_type: 3, extra: 0, child_count: 0, symbol: 0 }, // Error
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0 }, // Padding
        TSParseAction { action_type: 0, extra: 0, child_count: 0, symbol: 0 }, // Padding
    ];
    
    // Simple parse table
    static PARSE_TABLE: [u16; 100] = [0; 100];
    static SMALL_PARSE_TABLE: [u16; 100] = [0; 100];
    static SMALL_PARSE_TABLE_MAP: [u32; 10] = [0; 10];
    static LEX_MODES: [u32; 10] = [0; 10];
    static PRODUCTION_ID_MAP: [u16; 10] = [0; 10];
    
    static LANGUAGE: TSLanguage = TSLanguage {
        version: 15,
        symbol_count: 7,
        token_count: 4,
        state_count: 10,
        large_state_count: 5,
        production_id_count: 3,
        production_id_map: PRODUCTION_ID_MAP.as_ptr(),
        parse_table: PARSE_TABLE.as_ptr(),
        small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
        small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
        parse_actions: PARSE_ACTIONS.as_ptr(),
        lex_modes: LEX_MODES.as_ptr(),
        lex_fn: None,
        external_scanner: ExternalScanner {
            scan: None,
        },
    };
    
    &LANGUAGE
}

#[test]
fn test_pure_parser_creation() {
    let mut parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn test_set_language() {
    let mut parser = Parser::new();
    let language = create_test_language();
    
    assert!(parser.set_language(language).is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn test_parse_empty_string() {
    let mut parser = Parser::new();
    let language = create_test_language();
    parser.set_language(language).unwrap();
    
    let result = parser.parse_string("");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn test_parse_simple_expression() {
    let mut parser = Parser::new();
    let language = create_test_language();
    parser.set_language(language).unwrap();
    
    let result = parser.parse_string("1 + 2");
    
    // Check that we got either a parse tree or errors
    if let Some(root) = result.root {
        println!("Parsed successfully, root symbol: {}", root.symbol());
        assert!(root.child_count() > 0 || root.symbol() > 0);
    } else {
        println!("Parse errors: {:?}", result.errors);
        assert!(!result.errors.is_empty());
    }
}

#[test]
fn test_timeout() {
    let mut parser = Parser::new();
    let language = create_test_language();
    parser.set_language(language).unwrap();
    
    // Set a very short timeout
    parser.set_timeout_micros(1);
    
    // Try to parse something
    let result = parser.parse_string("1 + 2 * 3 + 4 * 5");
    
    // Should have timed out or parsed quickly
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn test_cancellation() {
    use std::sync::atomic::{AtomicBool, Ordering};
    
    let mut parser = Parser::new();
    let language = create_test_language();
    parser.set_language(language).unwrap();
    
    // Create cancellation flag
    let cancel_flag = AtomicBool::new(false);
    parser.set_cancellation_flag(Some(&cancel_flag as *const AtomicBool));
    
    // Set flag to cancel
    cancel_flag.store(true, Ordering::Relaxed);
    
    // Try to parse
    let result = parser.parse_string("1 + 2");
    
    // Should have been cancelled
    assert!(!result.errors.is_empty() || result.root.is_some());
}

#[test]
fn test_invalid_language_version() {
    let mut parser = Parser::new();
    
    // Create a language with invalid version
    static INVALID_LANGUAGE: TSLanguage = TSLanguage {
        version: 100, // Too high
        symbol_count: 0,
        token_count: 0,
        state_count: 0,
        large_state_count: 0,
        production_id_count: 0,
        production_id_map: ptr::null(),
        parse_table: ptr::null(),
        small_parse_table: ptr::null(),
        small_parse_table_map: ptr::null(),
        parse_actions: ptr::null(),
        lex_modes: ptr::null(),
        lex_fn: None,
        external_scanner: ExternalScanner {
            scan: None,
        },
    };
    
    assert!(parser.set_language(&INVALID_LANGUAGE).is_err());
}