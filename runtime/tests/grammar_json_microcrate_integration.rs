#![cfg(feature = "serialization")]

use adze::grammar_json::{
    load_patterns_from_grammar_json as runtime_load, load_patterns_with_symbol_map as runtime_map,
};
use adze_grammar_json_core::{
    load_patterns_from_grammar_json as core_load, load_patterns_with_symbol_map as core_map,
};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let tmp = NamedTempFile::new().expect("temp file");
    fs::write(
        tmp.path(),
        r#"{
          "rules": {
            "ident": {"type": "PATTERN", "value": "[a-z]+"},
            "kw_let": {"type": "STRING", "value": "let"},
            "ignored": {"type": "SYMBOL", "name": "foo"}
          }
        }"#,
    )
    .expect("write grammar json");

    let runtime_patterns = runtime_load(tmp.path()).expect("runtime load");
    let core_patterns = core_load(tmp.path()).expect("core load");
    assert_eq!(runtime_patterns, core_patterns);

    let symbols = vec![
        "kw_let".to_string(),
        "missing".to_string(),
        "ident".to_string(),
    ];
    let runtime_ids = runtime_map(tmp.path(), &symbols).expect("runtime map");
    let core_ids = core_map(tmp.path(), &symbols).expect("core map");
    assert_eq!(runtime_ids, core_ids);
}
