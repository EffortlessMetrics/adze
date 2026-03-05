//! Comprehensive tests for JSON grammar schema handling via `build_parser_from_json`.
//!
//! 55+ tests covering:
//! 1. Valid JSON grammar parsing (10 tests)
//! 2. Invalid JSON formats (8 tests)
//! 3. Missing required fields (8 tests)
//! 4. Token pattern formats (5 tests)
//! 5. Rule definition formats (5 tests)
//! 6. JSON output validation (5 tests)
//! 7. Schema evolution compatibility (5 tests)
//! 8. Edge cases (9 tests)

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn opts() -> BuildOptions {
    BuildOptions::default()
}

fn grammar_json(name: &str, rules: serde_json::Value) -> String {
    json!({ "name": name, "rules": rules }).to_string()
}

fn string_rule(val: &str) -> serde_json::Value {
    json!({ "type": "STRING", "value": val })
}

fn pattern_rule(val: &str) -> serde_json::Value {
    json!({ "type": "PATTERN", "value": val })
}

fn symbol_rule(name: &str) -> serde_json::Value {
    json!({ "type": "SYMBOL", "name": name })
}

fn choice_rule(members: Vec<serde_json::Value>) -> serde_json::Value {
    json!({ "type": "CHOICE", "members": members })
}

fn seq_rule(members: Vec<serde_json::Value>) -> serde_json::Value {
    json!({ "type": "SEQ", "members": members })
}

fn repeat_rule(content: serde_json::Value) -> serde_json::Value {
    json!({ "type": "REPEAT", "content": content })
}

fn repeat1_rule(content: serde_json::Value) -> serde_json::Value {
    json!({ "type": "REPEAT1", "content": content })
}

fn minimal(name: &str) -> String {
    grammar_json(name, json!({ "source_file": string_rule("x") }))
}

// ===========================================================================
// 1. Valid JSON grammar parsing (10 tests)
// ===========================================================================

#[test]
fn valid_minimal_string_grammar() {
    let r = build_parser_from_json(minimal("v01"), opts());
    assert!(r.is_ok(), "minimal STRING grammar failed: {:?}", r.err());
}

#[test]
fn valid_pattern_grammar() {
    let json = grammar_json("v02", json!({ "source_file": pattern_rule("[a-z]+") }));
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "PATTERN grammar failed: {:?}", r.err());
}

#[test]
fn valid_choice_grammar() {
    let json = grammar_json(
        "v03",
        json!({
            "source_file": choice_rule(vec![string_rule("a"), string_rule("b")])
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "CHOICE grammar failed: {:?}", r.err());
}

#[test]
fn valid_seq_grammar() {
    let json = grammar_json(
        "v04",
        json!({
            "source_file": seq_rule(vec![string_rule("x"), string_rule("y")])
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "SEQ grammar failed: {:?}", r.err());
}

#[test]
fn valid_repeat_grammar() {
    let json = grammar_json(
        "v05",
        json!({ "source_file": repeat_rule(string_rule("a")) }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "REPEAT grammar failed: {:?}", r.err());
}

#[test]
fn valid_repeat1_grammar() {
    let json = grammar_json(
        "v06",
        json!({ "source_file": repeat1_rule(string_rule("a")) }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "REPEAT1 grammar failed: {:?}", r.err());
}

#[test]
fn valid_multi_rule_grammar() {
    let json = grammar_json(
        "v07",
        json!({
            "source_file": symbol_rule("expr"),
            "expr": pattern_rule("[0-9]+")
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "multi-rule grammar failed: {:?}", r.err());
}

#[test]
fn valid_prec_left_grammar() {
    let json = grammar_json(
        "v08",
        json!({
            "source_file": {
                "type": "PREC_LEFT",
                "value": 1,
                "content": seq_rule(vec![pattern_rule("\\d+"), string_rule("+"), pattern_rule("\\d+")])
            }
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "PREC_LEFT grammar failed: {:?}", r.err());
}

#[test]
fn valid_prec_right_grammar() {
    let json = grammar_json(
        "v09",
        json!({
            "source_file": {
                "type": "PREC_RIGHT",
                "value": 2,
                "content": seq_rule(vec![pattern_rule("\\d+"), string_rule("^"), pattern_rule("\\d+")])
            }
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "PREC_RIGHT grammar failed: {:?}", r.err());
}

#[test]
fn valid_grammar_with_extras() {
    let json = json!({
        "name": "v10",
        "extras": [{ "type": "PATTERN", "value": "\\s+" }],
        "rules": {
            "source_file": pattern_rule("\\w+")
        }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "extras grammar failed: {:?}", r.err());
}

// ===========================================================================
// 2. Invalid JSON formats (8 tests)
// ===========================================================================

#[test]
fn invalid_empty_string() {
    let r = build_parser_from_json(String::new(), opts());
    assert!(r.is_err());
}

#[test]
fn invalid_not_json() {
    let r = build_parser_from_json("this is not json".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn invalid_bare_number() {
    let r = build_parser_from_json("42".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn invalid_bare_null() {
    let r = build_parser_from_json("null".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn invalid_bare_boolean() {
    let r = build_parser_from_json("true".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn invalid_bare_array() {
    let r = build_parser_from_json("[]".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn invalid_bare_string() {
    let r = build_parser_from_json("\"hello\"".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn invalid_truncated_json() {
    let r = build_parser_from_json("{\"name\":".to_string(), opts());
    assert!(r.is_err());
}

// ===========================================================================
// 3. Missing required fields (8 tests)
// ===========================================================================

#[test]
fn missing_everything_empty_object() {
    let r = build_parser_from_json("{}".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn missing_rules_field() {
    let r = build_parser_from_json(json!({"name": "noRules"}).to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn missing_name_field() {
    // Without "name", from_tree_sitter_json returns error
    let r = build_parser_from_json(
        json!({"rules": { "start": string_rule("x") }}).to_string(),
        opts(),
    );
    assert!(r.is_err());
}

#[test]
fn missing_rule_type_field() {
    let json = json!({
        "name": "mrt",
        "rules": {
            "source_file": { "value": "hello" }
        }
    })
    .to_string();
    // A rule without "type" cannot be parsed; results in empty grammar => error
    let r = build_parser_from_json(json, opts());
    assert!(r.is_err());
}

#[test]
fn missing_string_value() {
    let json = json!({
        "name": "msv",
        "rules": {
            "source_file": { "type": "STRING" }
        }
    })
    .to_string();
    // STRING without "value" fails rule parsing => empty grammar => error
    let r = build_parser_from_json(json, opts());
    assert!(r.is_err());
}

#[test]
fn missing_pattern_value() {
    let json = json!({
        "name": "mpv",
        "rules": {
            "source_file": { "type": "PATTERN" }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(r.is_err());
}

#[test]
fn missing_symbol_name() {
    let json = json!({
        "name": "msn",
        "rules": {
            "source_file": { "type": "SYMBOL" }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(r.is_err());
}

#[test]
fn missing_seq_members() {
    let json = json!({
        "name": "msm",
        "rules": {
            "source_file": { "type": "SEQ" }
        }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(r.is_err());
}

// ===========================================================================
// 4. Token pattern formats (5 tests)
// ===========================================================================

#[test]
fn token_pattern_simple_regex() {
    let json = grammar_json("tp01", json!({ "source_file": pattern_rule("[a-z]+") }));
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "simple regex pattern failed: {:?}", r.err());
}

#[test]
fn token_pattern_digit_class() {
    let json = grammar_json("tp02", json!({ "source_file": pattern_rule("\\d+") }));
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "\\d+ pattern failed: {:?}", r.err());
}

#[test]
fn token_pattern_word_class() {
    let json = grammar_json("tp03", json!({ "source_file": pattern_rule("\\w+") }));
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "\\w+ pattern failed: {:?}", r.err());
}

#[test]
fn token_pattern_alternation() {
    let json = grammar_json(
        "tp04",
        json!({ "source_file": pattern_rule("foo|bar|baz") }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "alternation pattern failed: {:?}", r.err());
}

#[test]
fn token_pattern_char_range() {
    let json = grammar_json(
        "tp05",
        json!({ "source_file": pattern_rule("[A-Za-z_][A-Za-z0-9_]*") }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "char-range pattern failed: {:?}", r.err());
}

// ===========================================================================
// 5. Rule definition formats (5 tests)
// ===========================================================================

#[test]
fn rule_def_nested_choice_in_seq() {
    let json = grammar_json(
        "rd01",
        json!({
            "source_file": seq_rule(vec![
                choice_rule(vec![string_rule("a"), string_rule("b")]),
                string_rule(";"),
            ])
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "nested choice-in-seq failed: {:?}", r.err());
}

#[test]
fn rule_def_nested_seq_in_choice() {
    let json = grammar_json(
        "rd02",
        json!({
            "source_file": choice_rule(vec![
                seq_rule(vec![string_rule("a"), string_rule("b")]),
                seq_rule(vec![string_rule("c"), string_rule("d")]),
            ])
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "nested seq-in-choice failed: {:?}", r.err());
}

#[test]
fn rule_def_repeat_of_choice() {
    let json = grammar_json(
        "rd03",
        json!({
            "source_file": repeat_rule(choice_rule(vec![string_rule("a"), string_rule("b")]))
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "repeat-of-choice failed: {:?}", r.err());
}

#[test]
fn rule_def_prec_dynamic() {
    let json = grammar_json(
        "rd04",
        json!({
            "source_file": {
                "type": "PREC_DYNAMIC",
                "value": 5,
                "content": string_rule("dyn")
            }
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "PREC_DYNAMIC failed: {:?}", r.err());
}

#[test]
fn rule_def_token_wrapper() {
    let json = grammar_json(
        "rd05",
        json!({
            "source_file": {
                "type": "TOKEN",
                "content": pattern_rule("[a-z]+")
            }
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "TOKEN wrapper failed: {:?}", r.err());
}

// ===========================================================================
// 6. JSON output validation (5 tests)
// ===========================================================================

#[test]
fn output_grammar_name_matches() {
    let r = build_parser_from_json(minimal("out_name"), opts()).unwrap();
    assert_eq!(r.grammar_name, "out_name");
}

#[test]
fn output_parser_code_nonempty() {
    let r = build_parser_from_json(minimal("out_code"), opts()).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn output_parser_path_nonempty() {
    let r = build_parser_from_json(minimal("out_path"), opts()).unwrap();
    assert!(!r.parser_path.is_empty());
}

#[test]
fn output_node_types_is_valid_json_array() {
    let r = build_parser_from_json(minimal("out_nt"), opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array(), "node_types_json must be a JSON array");
}

#[test]
fn output_build_stats_populated() {
    let r = build_parser_from_json(minimal("out_stats"), opts()).unwrap();
    // A minimal grammar must have at least 1 state and 1 symbol
    assert!(r.build_stats.state_count >= 1);
    assert!(r.build_stats.symbol_count >= 1);
}

// ===========================================================================
// 7. Schema evolution compatibility (5 tests)
// ===========================================================================

#[test]
fn compat_unknown_top_level_fields_ignored() {
    let json = json!({
        "name": "compat01",
        "version": 42,
        "author": "test",
        "rules": {
            "source_file": string_rule("hi")
        }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(
        r.is_ok(),
        "unknown top-level fields should be ignored: {:?}",
        r.err()
    );
}

#[test]
fn compat_extras_field_accepted() {
    let json = json!({
        "name": "compat02",
        "extras": [{ "type": "PATTERN", "value": "\\s" }],
        "rules": { "source_file": string_rule("x") }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "extras should be accepted: {:?}", r.err());
}

#[test]
fn compat_conflicts_field_accepted() {
    let json = json!({
        "name": "compat03",
        "conflicts": [],
        "rules": { "source_file": string_rule("x") }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(
        r.is_ok(),
        "empty conflicts should be accepted: {:?}",
        r.err()
    );
}

#[test]
fn compat_inline_field_accepted() {
    let json = json!({
        "name": "compat04",
        "inline": [],
        "rules": { "source_file": string_rule("x") }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "inline field should be accepted: {:?}", r.err());
}

#[test]
fn compat_supertypes_field_accepted() {
    let json = json!({
        "name": "compat05",
        "supertypes": [],
        "rules": { "source_file": string_rule("x") }
    })
    .to_string();
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "supertypes should be accepted: {:?}", r.err());
}

// ===========================================================================
// 8. Edge cases (9 tests)
// ===========================================================================

#[test]
fn edge_single_char_string_rule() {
    let json = grammar_json("edge01", json!({ "source_file": string_rule("x") }));
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "single-char STRING failed: {:?}", r.err());
}

#[test]
fn edge_long_string_literal() {
    let long_val = "a".repeat(200);
    let json = grammar_json("edge02", json!({ "source_file": string_rule(&long_val) }));
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "long string literal failed: {:?}", r.err());
}

#[test]
fn edge_grammar_name_with_underscores() {
    let r = build_parser_from_json(minimal("my_test_grammar"), opts()).unwrap();
    assert_eq!(r.grammar_name, "my_test_grammar");
}

#[test]
fn edge_grammar_name_with_numbers() {
    let r = build_parser_from_json(minimal("grammar123"), opts()).unwrap();
    assert_eq!(r.grammar_name, "grammar123");
}

#[test]
fn edge_deterministic_output() {
    let json = minimal("det_test");
    let r1 = build_parser_from_json(json.clone(), opts()).unwrap();
    let r2 = build_parser_from_json(json, opts()).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn edge_choice_with_blank_member() {
    let json = grammar_json(
        "edge06",
        json!({
            "source_file": choice_rule(vec![
                string_rule("a"),
                json!({ "type": "BLANK" }),
            ])
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "CHOICE with BLANK failed: {:?}", r.err());
}

#[test]
fn edge_optional_rule() {
    let json = grammar_json(
        "edge07",
        json!({
            "source_file": seq_rule(vec![
                string_rule("a"),
                choice_rule(vec![string_rule("b"), json!({"type": "BLANK"})]),
            ])
        }),
    );
    let r = build_parser_from_json(json, opts());
    assert!(r.is_ok(), "optional (CHOICE + BLANK) failed: {:?}", r.err());
}

#[test]
fn edge_multi_rule_node_types_valid_json() {
    let json = grammar_json(
        "edge08",
        json!({
            "source_file": symbol_rule("stmt"),
            "stmt": choice_rule(vec![
                symbol_rule("ident"),
                symbol_rule("num"),
            ]),
            "ident": pattern_rule("[a-z]+"),
            "num": pattern_rule("[0-9]+")
        }),
    );
    let r = build_parser_from_json(json, opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
    assert!(
        !v.as_array().unwrap().is_empty(),
        "multi-rule should produce node types"
    );
}

#[test]
fn edge_build_stats_for_multi_rule() {
    let json = grammar_json(
        "edge09",
        json!({
            "source_file": symbol_rule("item"),
            "item": choice_rule(vec![string_rule("a"), string_rule("b"), string_rule("c")])
        }),
    );
    let r = build_parser_from_json(json, opts()).unwrap();
    assert!(r.build_stats.state_count >= 1);
    assert!(r.build_stats.symbol_count >= 2);
}
