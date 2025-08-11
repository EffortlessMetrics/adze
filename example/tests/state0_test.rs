/// Test that verifies state 0 has token actions after desugaring
/// This test ensures the "state 0 bug" is fixed where pattern wrappers
/// prevented the parser from accepting any input.

#[cfg(feature = "pure-rust")]
#[test]
fn test_state0_has_token_actions() {
    use std::sync::Once;
    
    // Include the generated parser to get access to constants
    include!(concat!(env!("OUT_DIR"), "/grammar_arithmetic/parser_arithmetic.rs"));
    
    // Access the language structure
    let lang = &LANGUAGE;
    
    // Access the compressed parse table
    let parse_table_data = unsafe {
        std::slice::from_raw_parts(
            SMALL_PARSE_TABLE.as_ptr(),
            SMALL_PARSE_TABLE.len()
        )
    };
    
    let parse_table_map = unsafe {
        std::slice::from_raw_parts(
            SMALL_PARSE_TABLE_MAP.as_ptr(),
            (lang.state_count - lang.large_state_count + 1) as usize
        )
    };
    
    // State 0 starts at index 0 in the map
    let state0_start = parse_table_map[0] as usize;
    let state0_end = parse_table_map[1] as usize;
    
    // Extract state 0's entries
    let state0_entries = &parse_table_data[state0_start..state0_end];
    
    // State 0 entries are pairs: (symbol_index, action)
    assert!(state0_entries.len() >= 2, "State 0 should have at least one entry");
    assert!(state0_entries.len() % 2 == 0, "State 0 entries should be pairs");
    
    // Check if any entry is a token (symbol_index < token_count)
    let mut has_token_action = false;
    for i in (0..state0_entries.len()).step_by(2) {
        let symbol_index = state0_entries[i];
        let action = state0_entries[i + 1];
        
        // Tokens have indices less than token_count
        if symbol_index < lang.token_count as u16 {
            has_token_action = true;
            
            // Verify it's a shift action (action < 0x8000 means shift)
            assert!(action < 0x8000, 
                "Token {} in state 0 should have a shift action, got {:04x}", 
                symbol_index, action);
        }
    }
    
    assert!(has_token_action, 
        "State 0 must have at least one token action to accept input. \
         This indicates the pattern wrapper desugaring didn't work.");
    
    // Print state 0 info once for debugging (using Once to avoid spam)
    static PRINT_ONCE: Once = Once::new();
    PRINT_ONCE.call_once(|| {
        println!("State 0 validation passed:");
        println!("  - Token count: {}", lang.token_count);
        println!("  - State 0 has {} entries", state0_entries.len() / 2);
        
        for i in (0..state0_entries.len()).step_by(2) {
            let symbol = state0_entries[i];
            let action = state0_entries[i + 1];
            if symbol < lang.token_count as u16 {
                println!("  - Token {} -> action 0x{:04x} (shift to state {})", 
                    symbol, action, action & 0x7FFF);
            }
        }
    });
}

/// Integration test: parse arithmetic expression
#[cfg(feature = "pure-rust")]
#[test]
fn test_parse_arithmetic_expression() {
    // Include the generated parser
    include!(concat!(env!("OUT_DIR"), "/grammar_arithmetic/parser_arithmetic.rs"));
    
    // For now, just validate that the parser compiles and the language structure is valid
    let lang = &LANGUAGE;
    
    // Basic validation
    assert!(lang.token_count > 0, "Language should have tokens");
    assert!(lang.state_count > 0, "Language should have states");
    assert!(lang.symbol_count > 0, "Language should have symbols");
    
    // Verify state 0 specifically has the expected structure
    let parse_table_data = unsafe {
        std::slice::from_raw_parts(
            SMALL_PARSE_TABLE.as_ptr(),
            SMALL_PARSE_TABLE.len()
        )
    };
    
    // Check that parse table has data
    assert!(!parse_table_data.is_empty(), "Parse table should not be empty");
    
    println!("Arithmetic parser validation passed:");
    println!("  - Token count: {}", lang.token_count);
    println!("  - State count: {}", lang.state_count);
    println!("  - Symbol count: {}", lang.symbol_count);
    
    // Note: Full parsing with rust_sitter::Parser would require implementing
    // the full pure-rust parser runtime, which is beyond this test's scope.
    // The key validation is that state 0 now has token actions (tested above).
}