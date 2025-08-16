#![cfg(feature = "runtime-parity")]

use std::ffi::CStr;
use tree_sitter::{ffi, Parser};
use tree_sitter_json as ts_json;

// The tree-sitter-json crate exports this C symbol
extern "C" {
    fn tree_sitter_json() -> *const ffi::TSLanguage;
}

/// Discover a symbol id by its display name (no magic numbers)
fn symbol_id_by_name(lptr: *const ffi::TSLanguage, name: &str) -> u16 {
    let n_syms = unsafe { ffi::ts_language_symbol_count(lptr) } as u16;
    for i in 0..n_syms {
        let c_name = unsafe { ffi::ts_language_symbol_name(lptr, i) };
        if !c_name.is_null() {
            let s = unsafe { CStr::from_ptr(c_name) }.to_str().unwrap();
            if s == name {
                return i;
            }
        }
    }
    panic!("symbol `{name}` not found");
}

#[test]
fn runtime_parses_empty_object() {
    let mut p = Parser::new();
    p.set_language(&ts_json::LANGUAGE.into()).unwrap();
    let tree = p.parse("{}", None).expect("parse returned None");
    assert!(
        !tree.root_node().has_error(),
        "runtime reported a syntax error for {{}}"
    );
}

#[test]
fn runtime_parses_single_pair() {
    let mut p = Parser::new();
    p.set_language(&ts_json::LANGUAGE.into()).unwrap();
    let tree = p
        .parse(r#"{"key":"value"}"#, None)
        .expect("parse returned None");
    assert!(
        !tree.root_node().has_error(),
        "runtime reported a syntax error on single pair"
    );
}

#[test]
fn symbol_discovery_works() {
    let lptr = unsafe { tree_sitter_json() };
    for &(name, why) in &[("{", "lbrace"), ("}", "rbrace"), ("null", "null literal")] {
        let id = symbol_id_by_name(lptr, name);
        eprintln!("symbol `{name}` ({why}) -> id={id}");
        assert!(id < 1000);
    }
}

#[test]
fn language_metadata_available() {
    let lptr = unsafe { tree_sitter_json() };
    let sym_count = unsafe { ffi::ts_language_symbol_count(lptr) };
    let state_count = unsafe { ffi::ts_language_state_count(lptr) };
    eprintln!("json language: {sym_count} symbols, {state_count} states");
    assert!(sym_count > 10);
    assert!(state_count > 10);
}
