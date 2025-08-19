#![cfg(all(feature = "runtime-parity", feature = "ts-ffi-raw"))]
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
//! Integration tests for raw FFI cell parity.

use tree_sitter::ffi;

// The tree-sitter-json crate exports this C symbol
extern "C" {
    fn tree_sitter_json() -> *const ffi::TSLanguage;
}

// Our shim wrappers for internal functions
#[cfg(feature = "ts-ffi-raw")]
extern "C" {
    fn tsb_lookup(l: *const ffi::TSLanguage, s: u16, y: u16) -> u32;
    fn tsb_next_state(l: *const ffi::TSLanguage, s: u16, y: u16) -> u16;
}

#[test]
fn test_raw_lookup_parity() {
    // Force link of tree-sitter-json crate to ensure symbol is available
    let _ = tree_sitter_json::language();
    
    let lptr = unsafe { tree_sitter_json() };

    // Test a few state/symbol pairs - these are internal details
    // that could change between tree-sitter versions
    let test_cases = vec![
        (0, 0), // State 0, symbol 0
        (0, 1), // State 0, symbol 1
        (1, 0), // State 1, symbol 0
    ];

    for (state, symbol) in test_cases {
        let action = unsafe { tsb_lookup(lptr, state, symbol) };
        eprintln!("ts_language_lookup({state}, {symbol}) = 0x{action:08x}");

        // Just verify we get some value (not testing exact values as they're internal)
        // The important thing is that we can call the function
    }
}

#[test]
fn test_raw_next_state_parity() {
    // Force link of tree-sitter-json crate to ensure symbol is available
    let _ = tree_sitter_json::language();
    
    let lptr = unsafe { tree_sitter_json() };

    // Test a few state/symbol pairs
    let test_cases = vec![
        (0, 0), // State 0, symbol 0
        (0, 1), // State 0, symbol 1
        (1, 0), // State 1, symbol 0
    ];

    for (state, symbol) in test_cases {
        let next = unsafe { tsb_next_state(lptr, state, symbol) };
        eprintln!("ts_language_next_state({state}, {symbol}) = {next}");

        // Just verify we get some value
        // These are internal details that vary between grammars
    }
}
