use wasm_bindgen::prelude::*;

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
pub fn main() {
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

/// Minimal parser-facing smoke entrypoint for WASM checks.
///
/// Returns `true` when the bundled arithmetic grammar can parse a tiny input.
#[wasm_bindgen]
pub fn parser_smoke() -> bool {
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
    #[test]
    fn test_parser_smoke_entrypoint() {
        assert!(super::parser_smoke());
    }
}
