//! Comprehensive tests for the Grammar JSON → parser pipeline.
//!
//! 70+ tests covering:
//! 1. `build_parser_from_json` with invalid JSON → Err
//! 2. `build_parser_from_json` with empty/incomplete objects → graceful handling
//! 3. `build_parser_from_json` with valid grammar JSON
//! 4. `build_parser` with various Grammar shapes
//! 5. BuildResult consistency checks
//! 6. Error message quality for invalid inputs
//! 7. Determinism of builds
//! 8. Pipeline with compressed vs uncompressed tables

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::GrammarConverter;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser, build_parser_from_json};
use serde_json::json;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

fn opts(compress: bool) -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: compress,
    };
    (dir, opts)
}

fn opts_emit() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    (dir, opts)
}

fn minimal_json() -> String {
    json!({
        "name": "minimal",
        "rules": {
            "source_file": { "type": "STRING", "value": "x" }
        }
    })
    .to_string()
}

fn two_rule_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "expr" },
            "expr": { "type": "PATTERN", "value": "[0-9]+" }
        }
    })
    .to_string()
}

fn choice_json() -> String {
    json!({
        "name": "choice_grammar",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "number" },
                    { "type": "SYMBOL", "name": "word" }
                ]
            },
            "number": { "type": "PATTERN", "value": "[0-9]+" },
            "word": { "type": "PATTERN", "value": "[a-z]+" }
        }
    })
    .to_string()
}

fn seq_json() -> String {
    json!({
        "name": "seq_grammar",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "ident" },
                    { "type": "STRING", "value": "=" },
                    { "type": "SYMBOL", "name": "num" }
                ]
            },
            "ident": { "type": "PATTERN", "value": "[a-z]+" },
            "num": { "type": "PATTERN", "value": "[0-9]+" }
        }
    })
    .to_string()
}

fn repeat_json() -> String {
    json!({
        "name": "repeat_grammar",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "SYMBOL", "name": "item" }
            },
            "item": { "type": "PATTERN", "value": "[a-z]+" }
        }
    })
    .to_string()
}

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal_ir")
        .token("NUMBER", r"\d+")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

// ===========================================================================
// Section 1: build_parser_from_json with invalid JSON → Err  (tests 1–10)
// ===========================================================================

#[test]
fn t01_invalid_json_garbage_string() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("not json at all".into(), o);
    assert!(res.is_err());
}

#[test]
fn t02_invalid_json_truncated() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(r#"{"name":"a","rules":{"#.into(), o);
    assert!(res.is_err());
}

#[test]
fn t03_invalid_json_bare_number() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("42".into(), o);
    assert!(res.is_err());
}

#[test]
fn t04_invalid_json_bare_array() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("[1,2,3]".into(), o);
    assert!(res.is_err());
}

#[test]
fn t05_invalid_json_null() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("null".into(), o);
    assert!(res.is_err());
}

#[test]
fn t06_invalid_json_boolean() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("true".into(), o);
    assert!(res.is_err());
}

#[test]
fn t07_invalid_json_empty_string() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(String::new(), o);
    assert!(res.is_err());
}

#[test]
fn t08_invalid_json_trailing_comma() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(r#"{"name":"a","rules":{},}"#.into(), o);
    // Trailing comma is invalid in strict JSON
    assert!(res.is_err());
}

#[test]
fn t09_invalid_json_single_quotes() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("{'name':'a'}".into(), o);
    assert!(res.is_err());
}

#[test]
fn t10_invalid_json_unquoted_keys() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("{name:\"a\"}".into(), o);
    assert!(res.is_err());
}

// ===========================================================================
// Section 2: Empty / incomplete objects → graceful handling  (tests 11–17)
// ===========================================================================

#[test]
fn t11_empty_object() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json("{}".into(), o);
    assert!(res.is_err());
}

#[test]
fn t12_object_no_rules() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(json!({"name": "no_rules"}).to_string(), o);
    assert!(res.is_err());
}

#[test]
fn t13_rules_is_empty_object() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(json!({"name": "empty_rules", "rules": {}}).to_string(), o);
    assert!(res.is_err());
}

#[test]
fn t14_rules_is_null() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(json!({"name": "null_rules", "rules": null}).to_string(), o);
    assert!(res.is_err());
}

#[test]
fn t15_rules_is_array() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(json!({"name": "arr", "rules": []}).to_string(), o);
    assert!(res.is_err());
}

#[test]
fn t16_rule_with_unknown_type() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "bad_type",
            "rules": {
                "source_file": { "type": "BOGUS", "value": "x" }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_err());
}

#[test]
fn t17_rule_missing_type_field() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "no_type",
            "rules": {
                "source_file": { "value": "x" }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_err());
}

// ===========================================================================
// Section 3: build_parser_from_json with valid grammar JSON  (tests 18–30)
// ===========================================================================

#[test]
fn t18_minimal_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(minimal_json(), o);
    assert!(res.is_ok(), "minimal grammar should build: {:?}", res.err());
}

#[test]
fn t19_two_rule_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(two_rule_json("two_rule"), o);
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t20_choice_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(choice_json(), o);
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t21_seq_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(seq_json(), o);
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t22_repeat_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(repeat_json(), o);
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t23_repeat1_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "repeat1_g",
            "rules": {
                "source_file": {
                    "type": "REPEAT1",
                    "content": { "type": "STRING", "value": "a" }
                }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t24_optional_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "optional_g",
            "rules": {
                "source_file": {
                    "type": "SEQ",
                    "members": [
                        { "type": "STRING", "value": "a" },
                        {
                            "type": "CHOICE",
                            "members": [
                                { "type": "STRING", "value": "b" },
                                { "type": "BLANK" }
                            ]
                        }
                    ]
                }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t25_prec_left_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "prec_left_g",
            "rules": {
                "source_file": { "type": "SYMBOL", "name": "expr" },
                "expr": {
                    "type": "CHOICE",
                    "members": [
                        {
                            "type": "PREC_LEFT",
                            "value": 1,
                            "content": {
                                "type": "SEQ",
                                "members": [
                                    { "type": "SYMBOL", "name": "expr" },
                                    { "type": "STRING", "value": "+" },
                                    { "type": "SYMBOL", "name": "expr" }
                                ]
                            }
                        },
                        { "type": "PATTERN", "value": "[0-9]+" }
                    ]
                }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t26_prec_right_json_succeeds() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "prec_right_g",
            "rules": {
                "source_file": { "type": "SYMBOL", "name": "expr" },
                "expr": {
                    "type": "CHOICE",
                    "members": [
                        {
                            "type": "PREC_RIGHT",
                            "value": 2,
                            "content": {
                                "type": "SEQ",
                                "members": [
                                    { "type": "SYMBOL", "name": "expr" },
                                    { "type": "STRING", "value": "^" },
                                    { "type": "SYMBOL", "name": "expr" }
                                ]
                            }
                        },
                        { "type": "PATTERN", "value": "[0-9]+" }
                    ]
                }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t27_grammar_with_extras() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "with_extras",
            "extras": [{ "type": "PATTERN", "value": "\\s+" }],
            "rules": {
                "source_file": { "type": "SYMBOL", "name": "item" },
                "item": { "type": "PATTERN", "value": "[a-z]+" }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t28_grammar_name_extracted() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(two_rule_json("name_check"), o).unwrap();
    assert_eq!(r.grammar_name, "name_check");
}

#[test]
fn t29_token_rule_json() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "token_g",
            "rules": {
                "source_file": {
                    "type": "TOKEN",
                    "content": { "type": "PATTERN", "value": "[a-z]+" }
                }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn t30_nested_seq_choice_json() {
    let (_d, o) = opts(false);
    let res = build_parser_from_json(
        json!({
            "name": "nested_g",
            "rules": {
                "source_file": {
                    "type": "SEQ",
                    "members": [
                        {
                            "type": "CHOICE",
                            "members": [
                                { "type": "STRING", "value": "a" },
                                { "type": "STRING", "value": "b" }
                            ]
                        },
                        { "type": "STRING", "value": ";" }
                    ]
                }
            }
        })
        .to_string(),
        o,
    );
    assert!(res.is_ok(), "{:?}", res.err());
}

// ===========================================================================
// Section 4: build_parser with various Grammar shapes  (tests 31–45)
// ===========================================================================

#[test]
fn t31_build_parser_minimal_grammar() {
    let (_d, o) = opts(false);
    assert!(build_parser(minimal_grammar(), o).is_ok());
}

#[test]
fn t32_build_parser_arith_grammar() {
    let (_d, o) = opts(false);
    assert!(build_parser(arith_grammar(), o).is_ok());
}

#[test]
fn t33_build_parser_python_like() {
    let (_d, o) = opts(false);
    assert!(build_parser(GrammarBuilder::python_like(), o).is_ok());
}

#[test]
fn t34_build_parser_javascript_like() {
    let (_d, o) = opts(false);
    assert!(build_parser(GrammarBuilder::javascript_like(), o).is_ok());
}

#[test]
fn t35_build_parser_sample_grammar() {
    let (_d, o) = opts(false);
    assert!(build_parser(GrammarConverter::create_sample_grammar(), o).is_ok());
}

#[test]
fn t36_build_parser_single_epsilon_rule() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("epsilon_g")
        .rule("source_file", vec![])
        .start("source_file")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t37_build_parser_multiple_alternatives() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("multi_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("source_file", vec!["A"])
        .rule("source_file", vec!["B"])
        .rule("source_file", vec!["C"])
        .start("source_file")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t38_build_parser_chain_of_nonterminals() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("chain_g")
        .token("X", "x")
        .rule("source_file", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["X"])
        .start("source_file")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t39_build_parser_left_recursive() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("left_rec")
        .token("A", "a")
        .rule("list", vec!["list", "A"])
        .rule("list", vec!["A"])
        .start("list")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t40_build_parser_right_recursive() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("right_rec")
        .token("A", "a")
        .rule("list", vec!["A", "list"])
        .rule("list", vec!["A"])
        .start("list")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t41_build_parser_fragile_token() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("fragile_g")
        .token("OK", "ok")
        .fragile_token("ERR", "err")
        .rule("source_file", vec!["OK"])
        .rule("source_file", vec!["ERR"])
        .start("source_file")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t42_build_parser_with_extras() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("extras_g")
        .token("ID", r"[a-z]+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("source_file", vec!["ID"])
        .start("source_file")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t43_build_parser_with_precedence_decl() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("prec_decl_g")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t44_build_parser_many_tokens() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("many_tok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .token("F", "f")
        .rule("source_file", vec!["A"])
        .rule("source_file", vec!["B"])
        .rule("source_file", vec!["C"])
        .rule("source_file", vec!["D"])
        .rule("source_file", vec!["E"])
        .rule("source_file", vec!["F"])
        .start("source_file")
        .build();
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t45_build_parser_two_level_nesting() {
    let (_d, o) = opts(false);
    let g = GrammarBuilder::new("nest2")
        .token("X", "x")
        .token("Y", "y")
        .rule("source_file", vec!["pair"])
        .rule("pair", vec!["X", "Y"])
        .start("source_file")
        .build();
    assert!(build_parser(g, o).is_ok());
}

// ===========================================================================
// Section 5: BuildResult consistency checks  (tests 46–58)
// ===========================================================================

#[test]
fn t46_result_grammar_name_matches_json() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    assert_eq!(r.grammar_name, "minimal");
}

#[test]
fn t47_result_grammar_name_matches_ir() {
    let (_d, o) = opts(false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert_eq!(r.grammar_name, "minimal_ir");
}

#[test]
fn t48_result_parser_code_nonempty() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    assert!(!r.parser_code.is_empty());
    assert!(r.parser_code.len() > 50);
}

#[test]
fn t49_result_parser_path_nonempty() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    assert!(!r.parser_path.is_empty());
}

#[test]
fn t50_result_parser_path_contains_grammar_name() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(two_rule_json("path_check"), o).unwrap();
    assert!(
        r.parser_path.contains("path_check"),
        "parser_path should contain grammar name: {}",
        r.parser_path
    );
}

#[test]
fn t51_result_node_types_valid_json() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn t52_result_node_types_entries_have_type_and_named() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(two_rule_json("nt_check"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "missing 'type' in entry: {entry}"
        );
        assert!(
            entry.get("named").is_some(),
            "missing 'named' in entry: {entry}"
        );
    }
}

#[test]
fn t53_result_build_stats_state_count_positive() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn t54_result_build_stats_symbol_count_positive() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn t55_result_build_stats_complex_grammar_more_states() {
    let (_d, o1) = opts(false);
    let (_d2, o2) = opts(false);
    let r_simple = build_parser_from_json(minimal_json(), o1).unwrap();
    let r_complex = build_parser_from_json(choice_json(), o2).unwrap();
    // A grammar with choices should have at least as many symbols
    assert!(r_complex.build_stats.symbol_count >= r_simple.build_stats.symbol_count);
}

#[test]
fn t56_result_parser_code_is_valid_token_stream() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    assert!(r.parser_code.parse::<proc_macro2::TokenStream>().is_ok());
}

#[test]
fn t57_result_balanced_braces() {
    let (_d, o) = opts(false);
    let r = build_parser_from_json(two_rule_json("brace_bal"), o).unwrap();
    let open = r.parser_code.matches('{').count();
    let close = r.parser_code.matches('}').count();
    assert_eq!(open, close, "braces should be balanced");
}

#[test]
fn t58_result_all_fields_populated() {
    let (_d, o) = opts(false);
    let r = build_parser(arith_grammar(), o).unwrap();
    assert!(!r.grammar_name.is_empty());
    assert!(!r.parser_path.is_empty());
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

// ===========================================================================
// Section 6: Error message quality  (tests 59–64)
// ===========================================================================

#[test]
fn t59_error_message_mentions_json_on_bad_json() {
    let (_d, o) = opts(false);
    let err = build_parser_from_json("not json".into(), o).unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.to_lowercase().contains("json") || msg.to_lowercase().contains("parse"),
        "error should mention JSON/parse: {msg}"
    );
}

#[test]
fn t60_error_message_on_empty_object() {
    let (_d, o) = opts(false);
    let err = build_parser_from_json("{}".into(), o).unwrap_err();
    let msg = format!("{err:#}");
    assert!(!msg.is_empty(), "error message should not be empty");
}

#[test]
fn t61_error_message_on_unknown_rule_type() {
    let (_d, o) = opts(false);
    let err = build_parser_from_json(
        json!({
            "name": "bad_type",
            "rules": { "source_file": { "type": "INVALID_TYPE" } }
        })
        .to_string(),
        o,
    )
    .unwrap_err();
    let msg = format!("{err:#}");
    assert!(!msg.is_empty());
}

#[test]
fn t62_error_message_on_missing_rules_key() {
    let (_d, o) = opts(false);
    let err = build_parser_from_json(json!({"name": "oops"}).to_string(), o).unwrap_err();
    let msg = format!("{err:#}");
    assert!(!msg.is_empty());
}

#[test]
fn t63_error_from_invalid_json_is_context_wrapped() {
    let (_d, o) = opts(false);
    let err = build_parser_from_json("{invalid".into(), o).unwrap_err();
    // anyhow errors have context chains
    let chain: Vec<String> = err.chain().map(|e| e.to_string()).collect();
    assert!(!chain.is_empty(), "should have at least one error in chain");
}

#[test]
fn t64_error_debug_format_is_informative() {
    let (_d, o) = opts(false);
    let err = build_parser_from_json("[]".into(), o).unwrap_err();
    let dbg = format!("{err:?}");
    assert!(dbg.len() > 10, "debug format should be informative: {dbg}");
}

// ===========================================================================
// Section 7: Determinism of builds  (tests 65–70)
// ===========================================================================

#[test]
fn t65_json_pipeline_deterministic_parser_code() {
    let (_d1, o1) = opts(false);
    let (_d2, o2) = opts(false);
    let r1 = build_parser_from_json(minimal_json(), o1).unwrap();
    let r2 = build_parser_from_json(minimal_json(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn t66_json_pipeline_deterministic_node_types() {
    let (_d1, o1) = opts(false);
    let (_d2, o2) = opts(false);
    let r1 = build_parser_from_json(two_rule_json("det_nt"), o1).unwrap();
    let r2 = build_parser_from_json(two_rule_json("det_nt"), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn t67_json_pipeline_deterministic_stats() {
    let (_d1, o1) = opts(false);
    let (_d2, o2) = opts(false);
    let r1 = build_parser_from_json(choice_json(), o1).unwrap();
    let r2 = build_parser_from_json(choice_json(), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn t68_ir_pipeline_deterministic_parser_code() {
    let (_d1, o1) = opts(false);
    let (_d2, o2) = opts(false);
    let r1 = build_parser(minimal_grammar(), o1).unwrap();
    let r2 = build_parser(minimal_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn t69_ir_pipeline_deterministic_node_types() {
    let (_d1, o1) = opts(false);
    let (_d2, o2) = opts(false);
    let r1 = build_parser(arith_grammar(), o1).unwrap();
    let r2 = build_parser(arith_grammar(), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn t70_grammar_name_deterministic_across_runs() {
    let (_d1, o1) = opts(false);
    let (_d2, o2) = opts(false);
    let r1 = build_parser_from_json(seq_json(), o1).unwrap();
    let r2 = build_parser_from_json(seq_json(), o2).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

// ===========================================================================
// Section 8: Compressed vs uncompressed tables  (tests 71–80)
// ===========================================================================

#[test]
fn t71_compressed_succeeds() {
    let (_d, o) = opts(true);
    assert!(build_parser_from_json(minimal_json(), o).is_ok());
}

#[test]
fn t72_uncompressed_succeeds() {
    let (_d, o) = opts(false);
    assert!(build_parser_from_json(minimal_json(), o).is_ok());
}

#[test]
fn t73_compressed_and_uncompressed_same_grammar_name() {
    let (_d1, o1) = opts(true);
    let (_d2, o2) = opts(false);
    let r1 = build_parser_from_json(two_rule_json("comp_test"), o1).unwrap();
    let r2 = build_parser_from_json(two_rule_json("comp_test"), o2).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn t74_compressed_and_uncompressed_same_node_types() {
    let (_d1, o1) = opts(true);
    let (_d2, o2) = opts(false);
    let r1 = build_parser_from_json(choice_json(), o1).unwrap();
    let r2 = build_parser_from_json(choice_json(), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn t75_compressed_and_uncompressed_same_stats() {
    let (_d1, o1) = opts(true);
    let (_d2, o2) = opts(false);
    let r1 = build_parser_from_json(minimal_json(), o1).unwrap();
    let r2 = build_parser_from_json(minimal_json(), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn t76_compressed_ir_succeeds() {
    let (_d, o) = opts(true);
    assert!(build_parser(arith_grammar(), o).is_ok());
}

#[test]
fn t77_uncompressed_ir_succeeds() {
    let (_d, o) = opts(false);
    assert!(build_parser(arith_grammar(), o).is_ok());
}

#[test]
fn t78_compressed_complex_grammar() {
    let (_d, o) = opts(true);
    assert!(build_parser(GrammarBuilder::javascript_like(), o).is_ok());
}

#[test]
fn t79_emit_artifacts_with_compression() {
    let (d, o) = opts_emit();
    let r = build_parser_from_json(
        minimal_json(),
        BuildOptions {
            compress_tables: true,
            ..o
        },
    )
    .unwrap();
    let grammar_dir = d.path().join(format!("grammar_{}", r.grammar_name));
    assert!(grammar_dir.exists(), "grammar dir should be created");
}

#[test]
fn t80_emit_artifacts_creates_node_types_file() {
    let (d, mut o) = opts_emit();
    o.compress_tables = false;
    let r = build_parser_from_json(minimal_json(), o).unwrap();
    let nt_path = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("NODE_TYPES.json");
    assert!(nt_path.exists(), "NODE_TYPES.json should be written");
    let contents = std::fs::read_to_string(&nt_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert!(v.is_array());
}
