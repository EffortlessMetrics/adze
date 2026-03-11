//! Focused helpers for extracting token patterns from Tree-sitter `grammar.json` files.

#[cfg(feature = "serialization")]
use adze_ir::{SymbolId, TokenPattern};
#[cfg(feature = "serialization")]
use std::collections::HashMap;
#[cfg(feature = "serialization")]
use std::fs::File;
#[cfg(feature = "serialization")]
use std::path::Path;

/// Load token patterns from a Tree-sitter grammar.json file.
#[cfg(feature = "serialization")]
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

#[cfg(feature = "serialization")]
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

/// Load patterns and map them to symbol ids.
#[cfg(feature = "serialization")]
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

#[cfg(all(test, feature = "serialization"))]
mod tests {
    use super::load_patterns_from_grammar_json;
    use adze_ir::TokenPattern;
    use std::fs;

    #[test]
    fn extracts_string_and_regex_rules() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("grammar.json");
        fs::write(
            &path,
            r#"{
                "rules": {
                    "kw_def": {"type":"STRING", "value":"def"},
                    "identifier": {"type":"PATTERN", "value":"[a-z]+"},
                    "wrapped": {"type":"TOKEN", "content": {"type":"STRING", "value":"x"}}
                }
            }"#,
        )
        .expect("write grammar json");

        let patterns = load_patterns_from_grammar_json(&path).expect("patterns");

        assert_eq!(
            patterns.get("kw_def"),
            Some(&TokenPattern::String("def".into()))
        );
        assert_eq!(
            patterns.get("identifier"),
            Some(&TokenPattern::Regex("[a-z]+".into()))
        );
        assert_eq!(
            patterns.get("wrapped"),
            Some(&TokenPattern::String("x".into()))
        );
    }
}
