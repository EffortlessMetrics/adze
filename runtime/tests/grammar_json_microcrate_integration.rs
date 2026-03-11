#![cfg(feature = "serialization")]

use adze::grammar_json::load_patterns_from_grammar_json;
use adze_grammar_json_core as core;
use std::fs;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("grammar.json");
    fs::write(&path, r#"{"rules":{"kw":{"type":"STRING","value":"def"}}}"#)
        .expect("write grammar json");

    let runtime_result = load_patterns_from_grammar_json(&path).expect("runtime result");
    let microcrate_result =
        core::load_patterns_from_grammar_json(&path).expect("microcrate result");

    assert_eq!(runtime_result, microcrate_result);
}
