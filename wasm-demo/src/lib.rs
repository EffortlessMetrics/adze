use rust_sitter::ts_compat::Parser;
use wasm_bindgen::prelude::*;

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
pub fn main() {
    // Set panic hook for better error messages in browser console
    // console_error_panic_hook::set_once();

    web_sys::console::log_1(&"rust-sitter WASM demo initialized".into());
}

/// Parse Python source code and return S-expression representation
#[wasm_bindgen]
pub fn parse_python(_source: &str) -> String {
    // Temporarily disabled - Python ts_compat helper not yet implemented
    "Python parser temporarily disabled - needs ts_compat implementation".to_string()
}

/// Parse arithmetic expressions and return S-expression representation
#[wasm_bindgen]
pub fn parse_arithmetic(source: &str) -> String {
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();

    if parser.set_language(lang).is_err() {
        return "Failed to set language".to_string();
    }

    match parser.parse(source, None) {
        Some(tree) => {
            format!(
                "Parse successful! Root kind: {}, Errors: {}",
                tree.root_kind(),
                tree.error_count()
            )
        }
        None => "Parse failed".to_string(),
    }
}

/// Get GLR statistics from the last parse
#[wasm_bindgen]
pub fn get_parser_stats() -> String {
    // This would need to be stored in a global or passed back differently
    // For now, just return a placeholder
    "Stats: To be implemented".to_string()
}
