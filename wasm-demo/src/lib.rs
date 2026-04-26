use wasm_bindgen::prelude::*;

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
pub fn init_wasm_demo() {
    // Set panic hook for better error messages in browser console
    // console_error_panic_hook::set_once();

    web_sys::console::log_1(&"adze WASM demo initialized".into());
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
    match adze_example::arithmetic::grammar::parse(source) {
        Ok(ast) => format!("Parse successful! {:?}", ast),
        Err(_) => "Parse failed".to_string(),
    }
}

/// Minimal parser-facing WASM smoke path.
///
/// This proves `wasm-demo` can compile a parser entrypoint that calls into
/// generated parsing code (for arithmetic grammar).
#[wasm_bindgen]
pub fn parser_facing_smoke() -> bool {
    adze_example::arithmetic::grammar::parse("1 + 2 * 3").is_ok()
}

/// Get GLR statistics from the last parse
#[wasm_bindgen]
pub fn get_parser_stats() -> String {
    // This would need to be stored in a global or passed back differently
    // For now, just return a placeholder
    "Stats: To be implemented".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_facing_smoke() {
        assert!(parser_facing_smoke());
    }
}
