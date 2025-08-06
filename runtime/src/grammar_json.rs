use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use rust_sitter_ir::{TokenPattern, SymbolId};

/// Load token patterns from a Tree-sitter grammar.json file
#[cfg(feature = "serialization")]
pub fn load_patterns_from_grammar_json(path: &Path) -> Result<HashMap<String, TokenPattern>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let json: serde_json::Value = serde_json::from_reader(file)?;
    let mut patterns = HashMap::new();
    
    // The rules object contains all grammar rules
    if let Some(rules) = json.get("rules").and_then(|r| r.as_object()) {
        for (symbol_name, rule) in rules {
            // Extract pattern from the rule
            let pattern = extract_pattern_from_rule(rule);
            if let Some(p) = pattern {
                patterns.insert(symbol_name.clone(), p);
            }
        }
    }
    
    Ok(patterns)
}

/// Extract a TokenPattern from a grammar rule JSON value
#[cfg(feature = "serialization")]
fn extract_pattern_from_rule(rule: &serde_json::Value) -> Option<TokenPattern> {
    // Handle different rule types
    match rule.get("type").and_then(|t| t.as_str()) {
        Some("STRING") => {
            // String literal: { "type": "STRING", "value": "def" }
            rule.get("value")
                .and_then(|v| v.as_str())
                .map(|s| TokenPattern::String(s.to_string()))
        }
        Some("PATTERN") => {
            // Regex pattern: { "type": "PATTERN", "value": "[a-zA-Z_][a-zA-Z0-9_]*" }
            rule.get("value")
                .and_then(|v| v.as_str())
                .map(|s| TokenPattern::Regex(s.to_string()))
        }
        Some("TOKEN") => {
            // Token with immediate content: { "type": "TOKEN", "content": { ... } }
            rule.get("content").and_then(extract_pattern_from_rule)
        }
        Some("IMMEDIATE_TOKEN") => {
            // Immediate token: { "type": "IMMEDIATE_TOKEN", "content": { ... } }
            rule.get("content").and_then(extract_pattern_from_rule)
        }
        Some("ALIAS") => {
            // Alias wraps another rule: { "type": "ALIAS", "content": { ... } }
            rule.get("content").and_then(extract_pattern_from_rule)
        }
        Some("CHOICE") => {
            // For CHOICE, we can't easily represent it as a single pattern
            // We'd need to combine alternatives into a regex, which is complex
            // For now, skip CHOICE rules
            None
        }
        Some("SYMBOL") => {
            // Reference to another rule, not a terminal pattern
            None
        }
        Some("SEQ") | Some("REPEAT") | Some("REPEAT1") | Some("PREC") | 
        Some("PREC_LEFT") | Some("PREC_RIGHT") | Some("PREC_DYNAMIC") => {
            // These are non-terminals or complex rules
            None
        }
        _ => {
            // Unknown or complex rule type
            None
        }
    }
}

/// Load patterns and create a symbol name to ID mapping
#[cfg(feature = "serialization")]
pub fn load_patterns_with_symbol_map(
    grammar_json_path: &Path, 
    symbol_names: &[String]
) -> Result<HashMap<SymbolId, TokenPattern>, Box<dyn std::error::Error>> {
    let patterns_by_name = load_patterns_from_grammar_json(grammar_json_path)?;
    let mut patterns_by_id = HashMap::new();
    
    // Map patterns from name to symbol ID
    for (idx, name) in symbol_names.iter().enumerate() {
        if let Some(pattern) = patterns_by_name.get(name) {
            patterns_by_id.insert(SymbolId(idx as u16), pattern.clone());
        }
    }
    
    Ok(patterns_by_id)
}