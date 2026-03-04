//! Comprehensive tests for tool crate build_parser_from_json function.

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

// ── Invalid JSON ──

#[test]
fn from_json_empty_string() {
    let r = build_parser_from_json("".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

#[test]
fn from_json_garbage() {
    let r = build_parser_from_json("not json at all".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

#[test]
fn from_json_number() {
    let r = build_parser_from_json("42".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

#[test]
fn from_json_array() {
    let r = build_parser_from_json("[]".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

#[test]
fn from_json_null() {
    let r = build_parser_from_json("null".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

#[test]
fn from_json_true() {
    let r = build_parser_from_json("true".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

#[test]
fn from_json_string() {
    let r = build_parser_from_json("\"hello\"".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

// ── Incomplete JSON ──

#[test]
fn from_json_empty_object() {
    let r = build_parser_from_json("{}".to_string(), BuildOptions::default());
    assert!(r.is_err());
}

#[test]
fn from_json_no_rules() {
    let r = build_parser_from_json(r#"{"name":"test"}"#.to_string(), BuildOptions::default());
    assert!(r.is_err());
}

// ── Valid minimal grammar JSON ──

#[test]
fn from_json_minimal_string_rule() {
    let json = r#"{
        "name": "test",
        "rules": {
            "start": {
                "type": "STRING",
                "value": "hello"
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default());
    assert!(r.is_ok());
}

#[test]
fn from_json_pattern_rule() {
    let json = r#"{
        "name": "test",
        "rules": {
            "start": {
                "type": "PATTERN",
                "value": "[a-z]+"
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default());
    assert!(r.is_ok());
}

// ── Error messages ──

#[test]
fn from_json_error_is_descriptive() {
    let r = build_parser_from_json("invalid".to_string(), BuildOptions::default());
    let err = r.unwrap_err();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

// ── Determinism ──

#[test]
fn from_json_deterministic() {
    let json = r#"{
        "name": "det",
        "rules": {
            "start": {
                "type": "STRING",
                "value": "x"
            }
        }
    }"#;
    let r1 = build_parser_from_json(json.to_string(), BuildOptions::default()).unwrap();
    let r2 = build_parser_from_json(json.to_string(), BuildOptions::default()).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

// ── BuildResult fields ──

#[test]
fn from_json_result_name() {
    let json = r#"{
        "name": "mygrammar",
        "rules": {
            "start": {
                "type": "STRING",
                "value": "x"
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default()).unwrap();
    assert_eq!(r.grammar_name, "mygrammar");
}

#[test]
fn from_json_result_code_nonempty() {
    let json = r#"{
        "name": "code",
        "rules": {
            "start": {
                "type": "STRING",
                "value": "x"
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default()).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn from_json_result_node_types_json() {
    let json = r#"{
        "name": "nodes",
        "rules": {
            "start": {
                "type": "STRING",
                "value": "x"
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default()).unwrap();
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
}

// ── Complex grammars ──

#[test]
fn from_json_choice_rule() {
    let json = r#"{
        "name": "choice",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    {"type": "STRING", "value": "a"},
                    {"type": "STRING", "value": "b"}
                ]
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default());
    assert!(r.is_ok());
}

#[test]
fn from_json_seq_rule() {
    let json = r#"{
        "name": "seq",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "a"},
                    {"type": "STRING", "value": "b"}
                ]
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default());
    assert!(r.is_ok());
}

#[test]
fn from_json_repeat_rule() {
    let json = r#"{
        "name": "rep",
        "rules": {
            "start": {
                "type": "REPEAT",
                "content": {"type": "STRING", "value": "x"}
            }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), BuildOptions::default());
    assert!(r.is_ok());
}
