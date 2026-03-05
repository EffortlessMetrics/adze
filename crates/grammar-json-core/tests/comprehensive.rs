use adze_grammar_json_core::{load_patterns_from_grammar_json, load_patterns_with_symbol_map};
use adze_ir::{SymbolId, TokenPattern};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn loads_string_and_pattern_rules() {
    let tmp = NamedTempFile::new().expect("temp file");
    fs::write(
        tmp.path(),
        r#"{
          "rules": {
            "kw_def": {"type": "STRING", "value": "def"},
            "ident": {"type": "PATTERN", "value": "[a-z]+"},
            "skip": {"type": "SYMBOL", "name": "other"}
          }
        }"#,
    )
    .expect("write");

    let loaded = load_patterns_from_grammar_json(tmp.path()).expect("load patterns");
    assert_eq!(
        loaded.get("kw_def"),
        Some(&TokenPattern::String("def".into()))
    );
    assert_eq!(
        loaded.get("ident"),
        Some(&TokenPattern::Regex("[a-z]+".into()))
    );
    assert!(!loaded.contains_key("skip"));
}

#[test]
fn maps_patterns_to_symbol_ids_by_name() {
    let tmp = NamedTempFile::new().expect("temp file");
    fs::write(
        tmp.path(),
        r#"{
          "rules": {
            "ident": {"type": "PATTERN", "value": "[a-z]+"},
            "number": {"type": "PATTERN", "value": "[0-9]+"}
          }
        }"#,
    )
    .expect("write");

    let symbols = vec![
        "ident".to_string(),
        "missing".to_string(),
        "number".to_string(),
    ];
    let mapped = load_patterns_with_symbol_map(tmp.path(), &symbols).expect("mapped");

    assert_eq!(
        mapped.get(&SymbolId(0)),
        Some(&TokenPattern::Regex("[a-z]+".into()))
    );
    assert_eq!(
        mapped.get(&SymbolId(2)),
        Some(&TokenPattern::Regex("[0-9]+".into()))
    );
    assert!(!mapped.contains_key(&SymbolId(1)));
}
