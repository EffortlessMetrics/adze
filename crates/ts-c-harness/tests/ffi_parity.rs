use std::ffi::CStr;
use tree_sitter::{ffi, Language, Parser};
use tree_sitter_json as ts_json;

// Forward declare the internal Tree-sitter runtime functions
// These are provided by the tree-sitter crate's compiled runtime
extern "C" {
    fn ts_language_lookup(lang: *const std::ffi::c_void, state: u16, symbol: u16) -> u32;
    fn ts_language_next_state(lang: *const std::ffi::c_void, state: u16, symbol: u16) -> u16;
}

/// Discover symbol ID by name without guessing
fn symbol_id_by_name(lang: Language, name: &str) -> u16 {
    let lptr = lang.as_ptr();
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
fn step_from_initial_on_lbrace_is_valid() {
    let lang: Language = ts_json::LANGUAGE.into();
    let lptr = lang.as_ptr() as *const std::ffi::c_void;

    // Discover symbol ids by name (no guessing)
    let lbrace = symbol_id_by_name(lang, "{");
    let rbrace = symbol_id_by_name(&lang, "}");

    // Initial state is 0 in the runtime DFA
    let s0: u16 = 0;
    let s1 = unsafe { ts_language_next_state(lptr, s0, lbrace) };
    assert_ne!(s1, 0, "shifting '{' from state 0 must lead to a valid state");

    // Verify the cell after '{' on '}' has real actions
    let cell = unsafe { ts_language_lookup(lptr, s1, rbrace) };
    assert_ne!(cell, 0, "cell (after '{', on '}') must encode real actions");
}

#[test]
fn hot_cell_has_real_actions() {
    let lang: Language = ts_json::LANGUAGE.into();
    let lptr = lang.as_ptr() as *const std::ffi::c_void;

    let lbrace = symbol_id_by_name(lang, "{");
    let rbrace = symbol_id_by_name(&lang, "}");

    let s0 = 0u16;
    let s1 = unsafe { ts_language_next_state(lptr, s0, lbrace) };
    assert!(s1 != 0, "shift on '{' must lead somewhere");

    let cell = unsafe { ts_language_lookup(lptr, s1, rbrace) };
    assert!(cell != 0, "cell (after '{', on '}') must encode real actions");

    // Log the raw cell value for debugging/comparison
    eprintln!("lookup(state after '{{' = {}, symbol '}}') = 0x{:08x}", s1, cell);

    // Parser-level sanity check
    let mut parser = Parser::new();
    parser.set_language(&lang).unwrap();
    let tree = parser.parse("{}", None).unwrap();
    assert!(!tree.root_node().has_error());
}

#[test]
fn symbol_discovery_works() {
    let lang: Language = ts_json::LANGUAGE.into();
    
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
        let sym_id = symbol_id_by_name(lang, sym_name);
        eprintln!("Symbol '{}' ({}) has ID: {}", sym_name, desc, sym_id);
        assert!(sym_id < 1000, "symbol ID should be reasonable");
    }
}

// Future: Add raw cell equality assertion against extractor output
// #[test]
// fn raw_cell_parity_with_extractor() {
//     let lang: Language = ts_json::LANGUAGE.into();
//     let lptr = lang.as_ptr() as *const std::ffi::c_void;
//     
//     // Load extractor JSON or call extractor directly
//     let extractor_cells = load_extractor_cells();
//     
//     for (state, symbol, expected_cell) in extractor_cells {
//         let actual_cell = unsafe { ts_language_lookup(lptr, state, symbol) };
//         assert_eq!(
//             actual_cell, expected_cell,
//             "mismatch at (state={}, sym={})", state, symbol
//         );
//     }
// }