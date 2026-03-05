//! Core helpers for loading token patterns from Tree-sitter `grammar.json` files.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

use adze_ir::{SymbolId, TokenPattern};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Load token patterns from a Tree-sitter `grammar.json` file.
pub fn load_patterns_from_grammar_json(
    path: &Path,
) -> Result<HashMap<String, TokenPattern>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let json: serde_json::Value = serde_json::from_reader(file)?;
    let mut patterns = HashMap::new();

    if let Some(rules) = json.get("rules").and_then(|r| r.as_object()) {
        for (symbol_name, rule) in rules {
            if let Some(pattern) = extract_pattern_from_rule(rule) {
                patterns.insert(symbol_name.clone(), pattern);
            }
        }
    }

    Ok(patterns)
}

/// Load patterns and map them to symbol IDs using an index-aligned symbol name table.
pub fn load_patterns_with_symbol_map(
    grammar_json_path: &Path,
    symbol_names: &[String],
) -> Result<HashMap<SymbolId, TokenPattern>, Box<dyn std::error::Error>> {
    let patterns_by_name = load_patterns_from_grammar_json(grammar_json_path)?;
    let mut patterns_by_id = HashMap::new();

    for (idx, name) in symbol_names.iter().enumerate() {
        if let Some(pattern) = patterns_by_name.get(name) {
            patterns_by_id.insert(SymbolId(idx as u16), pattern.clone());
        }
    }

    Ok(patterns_by_id)
}

fn extract_pattern_from_rule(rule: &serde_json::Value) -> Option<TokenPattern> {
    match rule.get("type").and_then(|t| t.as_str()) {
        Some("STRING") => rule
            .get("value")
            .and_then(|v| v.as_str())
            .map(|s| TokenPattern::String(s.to_string())),
        Some("PATTERN") => rule
            .get("value")
            .and_then(|v| v.as_str())
            .map(|s| TokenPattern::Regex(s.to_string())),
        Some("TOKEN") | Some("IMMEDIATE_TOKEN") | Some("ALIAS") => {
            rule.get("content").and_then(extract_pattern_from_rule)
        }
        Some("CHOICE") | Some("SYMBOL") | Some("SEQ") | Some("REPEAT") | Some("REPEAT1")
        | Some("PREC") | Some("PREC_LEFT") | Some("PREC_RIGHT") | Some("PREC_DYNAMIC") => None,
        _ => None,
    }
}
