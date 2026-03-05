//! Comprehensive tests for grammar JSON generation via `build_parser_from_json`.
//!
//! 55+ tests covering:
//! 1. Valid grammar JSON structure (10 tests)
//! 2. node_types_json is valid JSON array (8 tests)
//! 3. BuildStats properties (8 tests)
//! 4. Grammar name in output (5 tests)
//! 5. Multiple grammars produce different outputs (5 tests)
//! 6. Deterministic output (5 tests)
//! 7. Error handling for invalid inputs (8 tests)
//! 8. Edge cases (6 tests)

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_opts() -> BuildOptions {
    BuildOptions::default()
}

fn minimal_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "STRING",
                "value": "hello"
            }
        }
    })
    .to_string()
}

fn pattern_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "PATTERN",
                "value": "[a-z]+"
            }
        }
    })
    .to_string()
}

fn choice_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    })
    .to_string()
}

fn seq_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "STRING", "value": "y" }
                ]
            }
        }
    })
    .to_string()
}

fn extras_json(name: &str) -> String {
    json!({
        "name": name,
        "extras": [{ "type": "PATTERN", "value": "\\s+" }],
        "rules": {
            "source_file": {
                "type": "PATTERN",
                "value": "\\w+"
            }
        }
    })
    .to_string()
}

fn multi_rule_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "statement" },
            "statement": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "assignment" },
                    { "type": "SYMBOL", "name": "expression" }
                ]
            },
            "assignment": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "identifier" },
                    { "type": "STRING", "value": "=" },
                    { "type": "SYMBOL", "name": "expression" }
                ]
            },
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "identifier" },
                    { "type": "PATTERN", "value": "\\d+" }
                ]
            },
            "identifier": { "type": "PATTERN", "value": "[a-zA-Z_][a-zA-Z0-9_]*" }
        }
    })
    .to_string()
}

fn repeat_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "PATTERN", "value": "[a-z]+" }
            }
        }
    })
    .to_string()
}

fn prec_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "PREC_LEFT",
                "value": 1,
                "content": {
                    "type": "SEQ",
                    "members": [
                        { "type": "PATTERN", "value": "\\d+" },
                        { "type": "STRING", "value": "+" },
                        { "type": "PATTERN", "value": "\\d+" }
                    ]
                }
            }
        }
    })
    .to_string()
}

// ===========================================================================
// 1. Valid grammar JSON structure (10 tests)
// ===========================================================================

#[test]
fn json_structure_minimal_string_builds_ok() {
    let r = build_parser_from_json(minimal_json("s01"), default_opts());
    assert!(
        r.is_ok(),
        "minimal STRING grammar should build: {:?}",
        r.err()
    );
}

#[test]
fn json_structure_pattern_builds_ok() {
    let r = build_parser_from_json(pattern_json("s02"), default_opts());
    assert!(r.is_ok(), "PATTERN grammar should build: {:?}", r.err());
}

#[test]
fn json_structure_choice_builds_ok() {
    let r = build_parser_from_json(choice_json("s03"), default_opts());
    assert!(r.is_ok(), "CHOICE grammar should build: {:?}", r.err());
}

#[test]
fn json_structure_seq_builds_ok() {
    let r = build_parser_from_json(seq_json("s04"), default_opts());
    assert!(r.is_ok(), "SEQ grammar should build: {:?}", r.err());
}

#[test]
fn json_structure_extras_builds_ok() {
    let r = build_parser_from_json(extras_json("s05"), default_opts());
    assert!(r.is_ok(), "grammar with extras should build: {:?}", r.err());
}

#[test]
fn json_structure_multi_rule_builds_ok() {
    let r = build_parser_from_json(multi_rule_json("s06"), default_opts());
    assert!(r.is_ok(), "multi-rule grammar should build: {:?}", r.err());
}

#[test]
fn json_structure_repeat_builds_ok() {
    let r = build_parser_from_json(repeat_json("s07"), default_opts());
    assert!(r.is_ok(), "REPEAT grammar should build: {:?}", r.err());
}

#[test]
fn json_structure_prec_builds_ok() {
    let r = build_parser_from_json(prec_json("s08"), default_opts());
    assert!(r.is_ok(), "PREC_LEFT grammar should build: {:?}", r.err());
}

#[test]
fn json_structure_result_has_parser_code() {
    let r = build_parser_from_json(minimal_json("s09"), default_opts()).unwrap();
    assert!(!r.parser_code.is_empty(), "parser_code should be non-empty");
}

#[test]
fn json_structure_result_has_parser_path() {
    let r = build_parser_from_json(minimal_json("s10"), default_opts()).unwrap();
    assert!(!r.parser_path.is_empty(), "parser_path should be non-empty");
}

// ===========================================================================
// 2. node_types_json is valid JSON array (8 tests)
// ===========================================================================

#[test]
fn node_types_is_valid_json() {
    let r = build_parser_from_json(minimal_json("nt01"), default_opts()).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("node_types_json should be valid JSON");
    assert!(v.is_array(), "node_types_json should be an array");
}

#[test]
fn node_types_array_not_empty_for_multi_rule() {
    let r = build_parser_from_json(multi_rule_json("nt02"), default_opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    let arr = v.as_array().expect("should be array");
    assert!(
        !arr.is_empty(),
        "multi-rule grammar should produce node types"
    );
}

#[test]
fn node_types_entries_have_type_field() {
    let r = build_parser_from_json(multi_rule_json("nt03"), default_opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "each node type entry should have a 'type' field: {entry}"
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let r = build_parser_from_json(multi_rule_json("nt04"), default_opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "each node type entry should have a 'named' field: {entry}"
        );
    }
}

#[test]
fn node_types_valid_json_for_pattern_grammar() {
    let r = build_parser_from_json(pattern_json("nt05"), default_opts()).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("should be valid JSON");
    assert!(v.is_array());
}

#[test]
fn node_types_valid_json_for_choice_grammar() {
    let r = build_parser_from_json(choice_json("nt06"), default_opts()).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("should be valid JSON");
    assert!(v.is_array());
}

#[test]
fn node_types_valid_json_for_repeat_grammar() {
    let r = build_parser_from_json(repeat_json("nt07"), default_opts()).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("should be valid JSON");
    assert!(v.is_array());
}

#[test]
fn node_types_valid_json_for_extras_grammar() {
    let r = build_parser_from_json(extras_json("nt08"), default_opts()).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("should be valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 3. BuildStats properties (8 tests)
// ===========================================================================

#[test]
fn stats_state_count_positive_minimal() {
    let r = build_parser_from_json(minimal_json("bs01"), default_opts()).unwrap();
    assert!(r.build_stats.state_count > 0, "state_count should be > 0");
}

#[test]
fn stats_symbol_count_positive_minimal() {
    let r = build_parser_from_json(minimal_json("bs02"), default_opts()).unwrap();
    assert!(r.build_stats.symbol_count > 0, "symbol_count should be > 0");
}

#[test]
fn stats_state_count_positive_multi_rule() {
    let r = build_parser_from_json(multi_rule_json("bs03"), default_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn stats_symbol_count_positive_multi_rule() {
    let r = build_parser_from_json(multi_rule_json("bs04"), default_opts()).unwrap();
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn stats_multi_rule_more_states_than_minimal() {
    let r_min = build_parser_from_json(minimal_json("bs05a"), default_opts()).unwrap();
    let r_multi = build_parser_from_json(multi_rule_json("bs05b"), default_opts()).unwrap();
    assert!(
        r_multi.build_stats.state_count >= r_min.build_stats.state_count,
        "multi-rule grammar should have >= states: {} vs {}",
        r_multi.build_stats.state_count,
        r_min.build_stats.state_count
    );
}

#[test]
fn stats_multi_rule_more_symbols_than_minimal() {
    let r_min = build_parser_from_json(minimal_json("bs06a"), default_opts()).unwrap();
    let r_multi = build_parser_from_json(multi_rule_json("bs06b"), default_opts()).unwrap();
    assert!(
        r_multi.build_stats.symbol_count >= r_min.build_stats.symbol_count,
        "multi-rule grammar should have >= symbols: {} vs {}",
        r_multi.build_stats.symbol_count,
        r_min.build_stats.symbol_count
    );
}

#[test]
fn stats_conflict_cells_non_negative() {
    let r = build_parser_from_json(multi_rule_json("bs07"), default_opts()).unwrap();
    // conflict_cells is usize, always >= 0, but verify it's accessible
    let _ = r.build_stats.conflict_cells;
}

#[test]
fn stats_debug_format() {
    let r = build_parser_from_json(minimal_json("bs08"), default_opts()).unwrap();
    let debug = format!("{:?}", r.build_stats);
    assert!(debug.contains("state_count"));
    assert!(debug.contains("symbol_count"));
    assert!(debug.contains("conflict_cells"));
}

// ===========================================================================
// 4. Grammar name in output (5 tests)
// ===========================================================================

#[test]
fn name_preserved_in_result() {
    let r = build_parser_from_json(minimal_json("my_grammar"), default_opts()).unwrap();
    assert_eq!(r.grammar_name, "my_grammar");
}

#[test]
fn name_preserved_multi_rule() {
    let r = build_parser_from_json(multi_rule_json("lang_test"), default_opts()).unwrap();
    assert_eq!(r.grammar_name, "lang_test");
}

#[test]
fn name_with_underscores() {
    let r = build_parser_from_json(minimal_json("my_cool_lang"), default_opts()).unwrap();
    assert_eq!(r.grammar_name, "my_cool_lang");
}

#[test]
fn name_appears_in_parser_path() {
    let r = build_parser_from_json(minimal_json("pathcheck"), default_opts()).unwrap();
    assert!(
        r.parser_path.contains("pathcheck"),
        "parser_path should contain grammar name: {}",
        r.parser_path
    );
}

#[test]
fn name_appears_in_parser_code() {
    let r = build_parser_from_json(minimal_json("codename"), default_opts()).unwrap();
    assert!(
        r.parser_code.contains("codename"),
        "parser_code should reference grammar name"
    );
}

// ===========================================================================
// 5. Multiple grammars produce different outputs (5 tests)
// ===========================================================================

#[test]
fn different_grammars_different_names() {
    let r1 = build_parser_from_json(minimal_json("diff_a"), default_opts()).unwrap();
    let r2 = build_parser_from_json(pattern_json("diff_b"), default_opts()).unwrap();
    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn different_grammars_different_parser_code() {
    let r1 = build_parser_from_json(minimal_json("diffcode_a"), default_opts()).unwrap();
    let r2 = build_parser_from_json(multi_rule_json("diffcode_b"), default_opts()).unwrap();
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn different_grammars_different_node_types() {
    let r1 = build_parser_from_json(minimal_json("diffnt_a"), default_opts()).unwrap();
    let r2 = build_parser_from_json(multi_rule_json("diffnt_b"), default_opts()).unwrap();
    assert_ne!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn different_grammars_different_stats() {
    let r1 = build_parser_from_json(minimal_json("diffst_a"), default_opts()).unwrap();
    let r2 = build_parser_from_json(multi_rule_json("diffst_b"), default_opts()).unwrap();
    let same_states = r1.build_stats.state_count == r2.build_stats.state_count;
    let same_symbols = r1.build_stats.symbol_count == r2.build_stats.symbol_count;
    assert!(
        !same_states || !same_symbols,
        "different grammars should differ in at least one stat dimension"
    );
}

#[test]
fn string_vs_pattern_different_output() {
    let r1 = build_parser_from_json(minimal_json("svp_a"), default_opts()).unwrap();
    let r2 = build_parser_from_json(pattern_json("svp_b"), default_opts()).unwrap();
    assert_ne!(
        r1.parser_code, r2.parser_code,
        "STRING vs PATTERN should generate different code"
    );
}

// ===========================================================================
// 6. Deterministic output (5 tests)
// ===========================================================================

#[test]
fn deterministic_grammar_name() {
    let json = minimal_json("det_name");
    let r1 = build_parser_from_json(json.clone(), default_opts()).unwrap();
    let r2 = build_parser_from_json(json, default_opts()).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn deterministic_parser_code() {
    let json = minimal_json("det_code");
    let r1 = build_parser_from_json(json.clone(), default_opts()).unwrap();
    let r2 = build_parser_from_json(json, default_opts()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn deterministic_node_types_json() {
    let json = minimal_json("det_nt");
    let r1 = build_parser_from_json(json.clone(), default_opts()).unwrap();
    let r2 = build_parser_from_json(json, default_opts()).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn deterministic_build_stats() {
    let json = minimal_json("det_stats");
    let r1 = build_parser_from_json(json.clone(), default_opts()).unwrap();
    let r2 = build_parser_from_json(json, default_opts()).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn deterministic_multi_rule() {
    let json = multi_rule_json("det_multi");
    let r1 = build_parser_from_json(json.clone(), default_opts()).unwrap();
    let r2 = build_parser_from_json(json, default_opts()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ===========================================================================
// 7. Error handling for invalid inputs (8 tests)
// ===========================================================================

#[test]
fn error_empty_string() {
    let r = build_parser_from_json(String::new(), default_opts());
    assert!(r.is_err(), "empty string should fail");
}

#[test]
fn error_not_json() {
    let r = build_parser_from_json("this is not json".to_string(), default_opts());
    assert!(r.is_err(), "non-JSON should fail");
}

#[test]
fn error_json_number() {
    let r = build_parser_from_json("42".to_string(), default_opts());
    assert!(r.is_err(), "bare number should fail");
}

#[test]
fn error_json_array() {
    let r = build_parser_from_json("[]".to_string(), default_opts());
    assert!(r.is_err(), "array should fail");
}

#[test]
fn error_json_null() {
    let r = build_parser_from_json("null".to_string(), default_opts());
    assert!(r.is_err(), "null should fail");
}

#[test]
fn error_empty_object() {
    let r = build_parser_from_json("{}".to_string(), default_opts());
    assert!(r.is_err(), "empty object should fail");
}

#[test]
fn error_no_rules_key() {
    let json = r#"{"name": "test"}"#.to_string();
    let r = build_parser_from_json(json, default_opts());
    assert!(r.is_err(), "missing 'rules' should fail");
}

#[test]
fn error_message_is_descriptive() {
    let r = build_parser_from_json("invalid json!!!".to_string(), default_opts());
    let err = r.unwrap_err();
    let msg = format!("{err}");
    assert!(!msg.is_empty(), "error message should not be empty");
}

// ===========================================================================
// 8. Edge cases (6 tests)
// ===========================================================================

#[test]
fn edge_single_char_string_rule() {
    let json = json!({
        "name": "single_char",
        "rules": {
            "source_file": { "type": "STRING", "value": "x" }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, default_opts());
    assert!(r.is_ok(), "single char STRING should work: {:?}", r.err());
}

#[test]
fn edge_long_grammar_name() {
    let name = "a_very_long_grammar_name_that_goes_on_and_on";
    let r = build_parser_from_json(minimal_json(name), default_opts()).unwrap();
    assert_eq!(r.grammar_name, name);
}

#[test]
fn edge_deeply_nested_seq() {
    let json = json!({
        "name": "nested_seq",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "STRING", "value": "b" },
                            { "type": "STRING", "value": "c" }
                        ]
                    }
                ]
            }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, default_opts());
    assert!(r.is_ok(), "nested SEQ should work: {:?}", r.err());
}

#[test]
fn edge_repeat1_rule() {
    let json = json!({
        "name": "rep1",
        "rules": {
            "source_file": {
                "type": "REPEAT1",
                "content": { "type": "PATTERN", "value": "[0-9]+" }
            }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, default_opts());
    assert!(r.is_ok(), "REPEAT1 should work: {:?}", r.err());
}

#[test]
fn edge_prec_right_rule() {
    let json = json!({
        "name": "prec_r",
        "rules": {
            "source_file": {
                "type": "PREC_RIGHT",
                "value": 2,
                "content": {
                    "type": "SEQ",
                    "members": [
                        { "type": "PATTERN", "value": "[a-z]+" },
                        { "type": "STRING", "value": "=" },
                        { "type": "PATTERN", "value": "[a-z]+" }
                    ]
                }
            }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, default_opts());
    assert!(r.is_ok(), "PREC_RIGHT should work: {:?}", r.err());
}

#[test]
fn edge_multiple_extras() {
    let json = json!({
        "name": "multi_extras",
        "extras": [
            { "type": "PATTERN", "value": "\\s+" },
            { "type": "PATTERN", "value": "//[^\\n]*" }
        ],
        "rules": {
            "source_file": {
                "type": "PATTERN",
                "value": "[a-z]+"
            }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, default_opts());
    assert!(r.is_ok(), "multiple extras should work: {:?}", r.err());
}
