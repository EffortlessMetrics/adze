use rust_sitter::unified_parser::Parser;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
pub fn main() {
    // Set panic hook for better error messages in browser console
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"rust-sitter WASM demo initialized".into());
}

/// Parse Python source code and return S-expression representation
#[wasm_bindgen]
pub fn parse_python(source: &str) -> String {
    // Register the Python scanner
    rust_sitter_python::register_scanner();

    let mut parser = Parser::new();
    match parser.set_language_with_name(rust_sitter_python::get_language(), "python") {
        Ok(_) => {}
        Err(e) => return format!("Failed to set language: {}", e),
    }

    match parser.parse(source, None) {
        Some(tree) => {
            // For now, just return basic info about the tree
            format!(
                "Parse successful! Root kind: {}, Errors: {}",
                tree.root_kind(),
                tree.error_count()
            )
        }
        None => "Parse failed".to_string(),
    }
}

/// Parse arithmetic expressions and return S-expression representation
#[wasm_bindgen]
pub fn parse_arithmetic(source: &str) -> String {
    let mut parser = Parser::new();
    match parser.set_language(rust_sitter_example::get_arithmetic_language()) {
        Ok(_) => {}
        Err(e) => return format!("Failed to set language: {}", e),
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
