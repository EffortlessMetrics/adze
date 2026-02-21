// WASM support for Adze
#![cfg(target_arch = "wasm32")]

use crate::pure_incremental::{Edit as IncrementalEdit, Tree};
use crate::pure_parser::{ParseResult, ParsedNode, Parser, TSLanguage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// WASM-compatible parser wrapper
#[wasm_bindgen]
pub struct WasmParser {
    parser: Parser,
}

#[wasm_bindgen]
impl WasmParser {
    /// Create a new parser
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmParser {
            parser: Parser::new(),
        }
    }

    /// Set the language (requires a language ID to look up)
    #[wasm_bindgen]
    pub fn set_language(&mut self, language_id: &str) -> Result<(), JsValue> {
        // In a real implementation, you would:
        // 1. Look up the language by ID from a registry
        // 2. Get the static TSLanguage reference
        // 3. Set it on the parser

        Err(JsValue::from_str("Language lookup not implemented"))
    }

    /// Parse a string
    #[wasm_bindgen]
    pub fn parse(&mut self, source: &str) -> Result<WasmParseResult, JsValue> {
        let result = self.parser.parse_string(source);
        Ok(WasmParseResult::from(result))
    }

    /// Set timeout in microseconds
    #[wasm_bindgen]
    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.parser.set_timeout_micros(timeout);
    }
}

/// WASM-compatible parse result
#[wasm_bindgen]
pub struct WasmParseResult {
    has_root: bool,
    error_count: usize,
    #[wasm_bindgen(skip)]
    pub root: Option<ParsedNode>,
    #[wasm_bindgen(skip)]
    pub errors: Vec<crate::pure_parser::ParseError>,
}

#[wasm_bindgen]
impl WasmParseResult {
    /// Check if parsing succeeded
    #[wasm_bindgen]
    pub fn has_root(&self) -> bool {
        self.has_root
    }

    /// Get number of errors
    #[wasm_bindgen]
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get the root node as JSON
    #[wasm_bindgen]
    pub fn root_to_json(&self) -> Result<String, JsValue> {
        if let Some(root) = &self.root {
            Ok(node_to_json(root))
        } else {
            Err(JsValue::from_str("No root node"))
        }
    }

    /// Get errors as JSON
    #[wasm_bindgen]
    pub fn errors_to_json(&self) -> String {
        serde_json::to_string(
            &self
                .errors
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "position": e.position,
                        "point": {
                            "row": e.point.row,
                            "column": e.point.column
                        },
                        "expected": e.expected,
                        "found": e.found
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|_| "[]".to_string())
    }
}

impl From<ParseResult> for WasmParseResult {
    fn from(result: ParseResult) -> Self {
        WasmParseResult {
            has_root: result.root.is_some(),
            error_count: result.errors.len(),
            root: result.root,
            errors: result.errors,
        }
    }
}

/// Convert a parsed node to JSON
fn node_to_json(node: &ParsedNode) -> String {
    let mut obj = serde_json::json!({
        "symbol": node.symbol(),
        "startByte": node.start_byte(),
        "endByte": node.end_byte(),
        "startPoint": {
            "row": node.start_point().row,
            "column": node.start_point().column
        },
        "endPoint": {
            "row": node.end_point().row,
            "column": node.end_point().column
        },
        "isExtra": node.is_extra(),
        "isError": node.is_error(),
        "childCount": node.child_count()
    });

    if node.child_count() > 0 {
        let children: Vec<serde_json::Value> = node
            .children()
            .iter()
            .map(|child| serde_json::from_str(&node_to_json(child)).unwrap())
            .collect();
        obj["children"] = serde_json::Value::Array(children);
    }

    serde_json::to_string(&obj).unwrap_or_else(|_| "{}".to_string())
}

/// Initialize WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages in WASM
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Language registry for WASM
pub mod registry {
    use super::*;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::sync::Mutex;

    static LANGUAGE_REGISTRY: Lazy<Mutex<HashMap<String, &'static TSLanguage>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    /// Register a language
    pub fn register_language(name: &str, language: &'static TSLanguage) {
        LANGUAGE_REGISTRY
            .lock()
            .unwrap()
            .insert(name.to_string(), language);
    }

    /// Get a language by name
    pub fn get_language(name: &str) -> Option<&'static TSLanguage> {
        LANGUAGE_REGISTRY.lock().unwrap().get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_parser_creation() {
        let parser = WasmParser::new();
        assert!(parser.parser.language().is_none());
    }

    #[test]
    fn test_node_to_json() {
        let node = ParsedNode {
            symbol: 1,
            children: vec![],
            start_byte: 0,
            end_byte: 5,
            start_point: crate::pure_parser::Point { row: 0, column: 0 },
            end_point: crate::pure_parser::Point { row: 0, column: 5 },
            is_extra: false,
            is_error: false,
            is_missing: false,
            is_named: true,
            field_id: None,
            language: None,
        };

        let json = node_to_json(&node);
        assert!(json.contains("\"symbol\":1"));
        assert!(json.contains("\"startByte\":0"));
        assert!(json.contains("\"endByte\":5"));
    }
}
