#![cfg(feature = "runtime-parity")]

use std::ffi::CStr;
use tree_sitter::{ffi, Parser};
use tree_sitter_json as ts_json;

// The tree-sitter-json crate exports this C symbol
extern "C" {
    fn tree_sitter_json() -> *const ffi::TSLanguage;
}

/// Discover symbol ID by name without guessing
fn symbol_id_by_name(lptr: *const ffi::TSLanguage, name: &str) -> u16 {
    let n_syms = unsafe { ffi::ts_language_symbol_count(lptr) } as u16;
    
    for i in 0..n_syms {
        let c_name = unsafe { ffi::ts_language_symbol_name(lptr, i) };
        if !c_name.is_null() {
            let symbol_name = unsafe { CStr::from_ptr(c_name) }.to_str().unwrap();
            if symbol_name == name {
                return i;
            }
        }
    }
    
    panic!("symbol `{name}` not found in language");
}

#[test]
fn runtime_parses_empty_object() {
    let mut parser = Parser::new();
    // tree-sitter 0.25.x: LANGUAGE is a const; `.into()` yields `Language`.
    parser.set_language(&ts_json::LANGUAGE.into()).unwrap();

    let tree = parser.parse("{}", None).expect("parse returned None");
    assert!(!tree.root_node().has_error(), "runtime reported a syntax error");
}

#[test]
fn runtime_parses_single_pair() {
    let mut parser = Parser::new();
    parser.set_language(&ts_json::LANGUAGE.into()).unwrap();

    let tree = parser.parse(r#"{"key": "value"}"#, None).expect("parse returned None");
    assert!(!tree.root_node().has_error(), "runtime reported a syntax error on single pair");
}

#[test]
fn symbol_discovery_works() {
    let lptr = unsafe { tree_sitter_json() };
    
    // Verify we can find common JSON symbols
    let symbols = vec![
        ("{", "left brace"),
        ("}", "right brace"),
        ("[", "left bracket"),
        ("]", "right bracket"),
        (":", "colon"),
        (",", "comma"),
    ];
    
    for (sym_name, desc) in symbols {
        let sym_id = symbol_id_by_name(lptr, sym_name);
        eprintln!("Symbol '{}' ({}) has ID: {}", sym_name, desc, sym_id);
        assert!(sym_id < 1000, "symbol ID should be reasonable");
    }
}

#[test]
fn language_metadata_available() {
    let lptr = unsafe { tree_sitter_json() };
    
    // Check basic language metadata
    let symbol_count = unsafe { ffi::ts_language_symbol_count(lptr) };
    let state_count = unsafe { ffi::ts_language_state_count(lptr) };
    
    eprintln!("JSON language has {} symbols and {} states", symbol_count, state_count);
    
    assert!(symbol_count > 10, "JSON should have more than 10 symbols");
    assert!(state_count > 10, "JSON should have more than 10 states");
}

// Note: Direct table access functions like ts_language_lookup and ts_language_next_state
// are not exported by the tree-sitter library, so we can't test them directly.
// For full parity testing, we would need to either:
// 1. Link against the tree-sitter C library directly with a custom build script
// 2. Use the ts-bridge tool to extract and compare tables
// 3. Implement our own C shim that exposes these functions