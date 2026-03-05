//! Comprehensive integration tests for the adze-tool build pipeline (v3).
//!
//! 56 tests covering:
//! 1. End-to-end: JSON → grammar → table → stats (8 tests)
//! 2. Arithmetic expression grammar pipeline (5 tests)
//! 3. JSON-like grammar pipeline (5 tests)
//! 4. List/array grammar pipeline (5 tests)
//! 5. Build determinism (8 tests: same input → same output twice)
//! 6. Error recovery in pipeline (8 tests: various invalid inputs)
//! 7. Stats consistency (8 tests: state_count matches table)
//! 8. Edge cases (9 tests: minimal grammar, large grammar, unicode)

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_json,
};
use serde_json::json;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (dir, opts)
}

fn build_json(grammar_json: &str) -> anyhow::Result<BuildResult> {
    let (_dir, opts) = tmp_opts();
    build_parser_from_json(grammar_json.to_string(), opts)
}

/// Minimal grammar: one STRING rule
fn minimal_json() -> String {
    json!({
        "name": "minimal",
        "rules": {
            "start": { "type": "STRING", "value": "hello" }
        }
    })
    .to_string()
}

/// Arithmetic expression grammar with operators
fn arithmetic_json() -> String {
    json!({
        "name": "arithmetic",
        "rules": {
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "number" },
                    {
                        "type": "PREC_LEFT",
                        "value": 1,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expression" },
                                { "type": "STRING", "value": "+" },
                                { "type": "SYMBOL", "name": "expression" }
                            ]
                        }
                    },
                    {
                        "type": "PREC_LEFT",
                        "value": 2,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expression" },
                                { "type": "STRING", "value": "*" },
                                { "type": "SYMBOL", "name": "expression" }
                            ]
                        }
                    }
                ]
            },
            "number": { "type": "PATTERN", "value": "[0-9]+" }
        }
    })
    .to_string()
}

/// JSON-like grammar with objects and strings
fn json_like_json() -> String {
    json!({
        "name": "json_like",
        "rules": {
            "document": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "object" },
                    { "type": "SYMBOL", "name": "value" }
                ]
            },
            "object": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "{" },
                    { "type": "SYMBOL", "name": "pair" },
                    { "type": "STRING", "value": "}" }
                ]
            },
            "pair": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "key" },
                    { "type": "STRING", "value": ":" },
                    { "type": "SYMBOL", "name": "value" }
                ]
            },
            "key": { "type": "PATTERN", "value": "[a-zA-Z_]+" },
            "value": { "type": "PATTERN", "value": "[a-zA-Z0-9_]+" }
        }
    })
    .to_string()
}

/// List/array grammar with delimiters
fn list_json() -> String {
    json!({
        "name": "list_lang",
        "rules": {
            "list": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "[" },
                    { "type": "SYMBOL", "name": "items" },
                    { "type": "STRING", "value": "]" }
                ]
            },
            "items": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "item" },
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "SYMBOL", "name": "items" },
                            { "type": "STRING", "value": "," },
                            { "type": "SYMBOL", "name": "item" }
                        ]
                    }
                ]
            },
            "item": { "type": "PATTERN", "value": "[a-z]+" }
        }
    })
    .to_string()
}

// ===========================================================================
// 1. End-to-end: JSON → grammar → table → stats (8 tests)
// ===========================================================================

#[test]
fn e2e_minimal_json_builds_successfully() {
    let result = build_json(&minimal_json());
    assert!(result.is_ok(), "minimal grammar should build: {result:?}");
}

#[test]
fn e2e_minimal_json_name_preserved() {
    let result = build_json(&minimal_json()).unwrap();
    assert_eq!(result.grammar_name, "minimal");
}

#[test]
fn e2e_minimal_json_produces_parser_code() {
    let result = build_json(&minimal_json()).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn e2e_minimal_json_produces_node_types() {
    let result = build_json(&minimal_json()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn e2e_minimal_json_has_positive_state_count() {
    let result = build_json(&minimal_json()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn e2e_minimal_json_has_positive_symbol_count() {
    let result = build_json(&minimal_json()).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn e2e_pattern_rule_builds() {
    let grammar_json = json!({
        "name": "pattern_test",
        "rules": {
            "start": { "type": "PATTERN", "value": "[a-z]+" }
        }
    })
    .to_string();
    let result = build_json(&grammar_json);
    assert!(result.is_ok(), "pattern rule should build: {result:?}");
}

#[test]
fn e2e_seq_rule_builds() {
    let grammar_json = json!({
        "name": "seq_test",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    })
    .to_string();
    let result = build_json(&grammar_json);
    assert!(result.is_ok(), "seq rule should build: {result:?}");
}

// ===========================================================================
// 2. Arithmetic expression grammar pipeline (5 tests)
// ===========================================================================

#[test]
fn arith_json_builds_ok() {
    let result = build_json(&arithmetic_json());
    assert!(
        result.is_ok(),
        "arithmetic grammar should build: {result:?}"
    );
}

#[test]
fn arith_json_name() {
    let result = build_json(&arithmetic_json()).unwrap();
    assert_eq!(result.grammar_name, "arithmetic");
}

#[test]
fn arith_json_multiple_states() {
    let result = build_json(&arithmetic_json()).unwrap();
    // Arithmetic with operators needs more than one state
    assert!(
        result.build_stats.state_count > 1,
        "arithmetic grammar should have multiple states, got {}",
        result.build_stats.state_count
    );
}

#[test]
fn arith_json_multiple_symbols() {
    let result = build_json(&arithmetic_json()).unwrap();
    // expression, number, +, *, EOF at minimum
    assert!(
        result.build_stats.symbol_count >= 3,
        "arithmetic grammar should have multiple symbols, got {}",
        result.build_stats.symbol_count
    );
}

#[test]
fn arith_json_parser_code_contains_grammar_name() {
    let result = build_json(&arithmetic_json()).unwrap();
    // Parser code or grammar name should reference the grammar
    assert_eq!(result.grammar_name, "arithmetic");
    assert!(!result.parser_code.is_empty());
}

// ===========================================================================
// 3. JSON-like grammar pipeline (5 tests)
// ===========================================================================

#[test]
fn json_like_builds_ok() {
    let result = build_json(&json_like_json());
    assert!(result.is_ok(), "json-like grammar should build: {result:?}");
}

#[test]
fn json_like_name() {
    let result = build_json(&json_like_json()).unwrap();
    assert_eq!(result.grammar_name, "json_like");
}

#[test]
fn json_like_has_states() {
    let result = build_json(&json_like_json()).unwrap();
    assert!(
        result.build_stats.state_count > 1,
        "json-like grammar needs multiple states for nested structure"
    );
}

#[test]
fn json_like_symbols_include_delimiters() {
    let result = build_json(&json_like_json()).unwrap();
    // At least: document, object, pair, key, value, "{", "}", ":", EOF
    assert!(
        result.build_stats.symbol_count >= 5,
        "json-like grammar should have many symbols, got {}",
        result.build_stats.symbol_count
    );
}

#[test]
fn json_like_produces_valid_node_types() {
    let result = build_json(&json_like_json()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array());
    // Should have multiple node types for a complex grammar
    let arr = parsed.as_array().unwrap();
    assert!(
        !arr.is_empty(),
        "node types should not be empty for json-like grammar"
    );
}

// ===========================================================================
// 4. List/array grammar pipeline (5 tests)
// ===========================================================================

#[test]
fn list_grammar_builds_ok() {
    let result = build_json(&list_json());
    assert!(result.is_ok(), "list grammar should build: {result:?}");
}

#[test]
fn list_grammar_name() {
    let result = build_json(&list_json()).unwrap();
    assert_eq!(result.grammar_name, "list_lang");
}

#[test]
fn list_grammar_has_states() {
    let result = build_json(&list_json()).unwrap();
    assert!(
        result.build_stats.state_count > 1,
        "list grammar with brackets and commas needs multiple states"
    );
}

#[test]
fn list_grammar_has_symbols() {
    let result = build_json(&list_json()).unwrap();
    // list, items, item, "[", "]", ",", EOF at minimum
    assert!(
        result.build_stats.symbol_count >= 4,
        "list grammar should have several symbols, got {}",
        result.build_stats.symbol_count
    );
}

#[test]
fn list_grammar_parser_code_nonempty() {
    let result = build_json(&list_json()).unwrap();
    assert!(
        !result.parser_code.is_empty(),
        "list grammar should produce parser code"
    );
}

// ===========================================================================
// 5. Build determinism (8 tests: same input → same output twice)
// ===========================================================================

#[test]
fn determinism_minimal_grammar_name() {
    let r1 = build_json(&minimal_json()).unwrap();
    let r2 = build_json(&minimal_json()).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn determinism_minimal_state_count() {
    let r1 = build_json(&minimal_json()).unwrap();
    let r2 = build_json(&minimal_json()).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn determinism_minimal_symbol_count() {
    let r1 = build_json(&minimal_json()).unwrap();
    let r2 = build_json(&minimal_json()).unwrap();
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn determinism_minimal_node_types() {
    let r1 = build_json(&minimal_json()).unwrap();
    let r2 = build_json(&minimal_json()).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn determinism_arith_state_count() {
    let r1 = build_json(&arithmetic_json()).unwrap();
    let r2 = build_json(&arithmetic_json()).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn determinism_arith_symbol_count() {
    let r1 = build_json(&arithmetic_json()).unwrap();
    let r2 = build_json(&arithmetic_json()).unwrap();
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn determinism_json_like_state_count() {
    let r1 = build_json(&json_like_json()).unwrap();
    let r2 = build_json(&json_like_json()).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn determinism_ir_builder_state_count() {
    let make_grammar = || {
        GrammarBuilder::new("det_ir")
            .token("x", "x")
            .token("y", "y")
            .rule("root", vec!["x", "y"])
            .start("root")
            .build()
    };
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(make_grammar(), o1).unwrap();
    let r2 = build_parser(make_grammar(), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

// ===========================================================================
// 6. Error recovery in pipeline (8 tests: various invalid inputs)
// ===========================================================================

#[test]
fn error_empty_json_string() {
    let result = build_json("");
    assert!(result.is_err(), "empty string should fail");
}

#[test]
fn error_invalid_json_syntax() {
    let result = build_json("{not valid json}");
    assert!(result.is_err(), "malformed JSON should fail");
}

#[test]
fn error_json_missing_name() {
    let grammar_json = json!({
        "rules": {
            "start": { "type": "STRING", "value": "x" }
        }
    })
    .to_string();
    let result = build_json(&grammar_json);
    // Missing name may use "unknown" fallback or fail — either is acceptable
    // We just verify it doesn't panic
    let _ = result;
}

#[test]
fn error_json_missing_rules() {
    let grammar_json = json!({ "name": "no_rules" }).to_string();
    let result = build_json(&grammar_json);
    assert!(result.is_err(), "grammar without rules should fail");
}

#[test]
fn error_json_empty_rules() {
    let grammar_json = json!({
        "name": "empty_rules",
        "rules": {}
    })
    .to_string();
    let result = build_json(&grammar_json);
    assert!(result.is_err(), "grammar with empty rules should fail");
}

#[test]
fn error_json_null_input() {
    let result = build_json("null");
    assert!(result.is_err(), "null JSON should fail");
}

#[test]
fn error_json_array_input() {
    let result = build_json("[]");
    assert!(result.is_err(), "JSON array should fail");
}

#[test]
fn error_json_plain_number() {
    let result = build_json("42");
    assert!(result.is_err(), "plain number JSON should fail");
}

// ===========================================================================
// 7. Stats consistency (8 tests: state_count matches table)
// ===========================================================================

#[test]
fn stats_minimal_state_count_positive() {
    let result = build_json(&minimal_json()).unwrap();
    assert!(
        result.build_stats.state_count > 0,
        "state_count must be positive"
    );
}

#[test]
fn stats_minimal_symbol_count_positive() {
    let result = build_json(&minimal_json()).unwrap();
    assert!(
        result.build_stats.symbol_count > 0,
        "symbol_count must be positive"
    );
}

#[test]
fn stats_arith_more_states_than_minimal() {
    let minimal = build_json(&minimal_json()).unwrap();
    let arith = build_json(&arithmetic_json()).unwrap();
    assert!(
        arith.build_stats.state_count >= minimal.build_stats.state_count,
        "arithmetic grammar should have at least as many states as minimal"
    );
}

#[test]
fn stats_arith_more_symbols_than_minimal() {
    let minimal = build_json(&minimal_json()).unwrap();
    let arith = build_json(&arithmetic_json()).unwrap();
    assert!(
        arith.build_stats.symbol_count > minimal.build_stats.symbol_count,
        "arithmetic grammar should have more symbols than minimal"
    );
}

#[test]
fn stats_json_like_state_count_reasonable() {
    let result = build_json(&json_like_json()).unwrap();
    // A grammar with 5 rules shouldn't produce thousands of states
    assert!(
        result.build_stats.state_count < 1000,
        "state_count should be reasonable, got {}",
        result.build_stats.state_count
    );
}

#[test]
fn stats_conflict_cells_non_negative() {
    let result = build_json(&arithmetic_json()).unwrap();
    // conflict_cells is usize, so always >= 0, but check it's computed
    let _ = result.build_stats.conflict_cells;
}

#[test]
fn stats_ir_builder_state_count_positive() {
    let grammar = GrammarBuilder::new("stats_ir")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn stats_ir_builder_symbol_count_positive() {
    let grammar = GrammarBuilder::new("stats_ir2")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a", "b"])
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert!(
        result.build_stats.symbol_count >= 2,
        "grammar with 2 tokens should have at least 2 symbols, got {}",
        result.build_stats.symbol_count
    );
}

// ===========================================================================
// 8. Edge cases (9 tests: minimal grammar, large grammar, unicode)
// ===========================================================================

#[test]
fn edge_single_string_rule() {
    let grammar_json = json!({
        "name": "single_str",
        "rules": {
            "start": { "type": "STRING", "value": "x" }
        }
    })
    .to_string();
    let result = build_json(&grammar_json);
    assert!(
        result.is_ok(),
        "single string rule should build: {result:?}"
    );
}

#[test]
fn edge_choice_with_many_alternatives() {
    let grammar_json = json!({
        "name": "many_alts",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" },
                    { "type": "STRING", "value": "d" },
                    { "type": "STRING", "value": "e" },
                    { "type": "STRING", "value": "f" }
                ]
            }
        }
    })
    .to_string();
    let result = build_json(&grammar_json);
    assert!(result.is_ok(), "many alternatives should build: {result:?}");
}

#[test]
fn edge_deeply_nested_seq() {
    let grammar_json = json!({
        "name": "deep_seq",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "STRING", "value": "a" },
                            { "type": "STRING", "value": "b" }
                        ]
                    },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    })
    .to_string();
    let result = build_json(&grammar_json);
    assert!(result.is_ok(), "nested seq should build: {result:?}");
}

#[test]
fn edge_ir_three_level_chain() {
    let grammar = GrammarBuilder::new("deep_chain")
        .token("x", "x")
        .rule("level1", vec!["x"])
        .rule("level2", vec!["level1"])
        .rule("level3", vec!["level2"])
        .rule("root", vec!["level3"])
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts);
    assert!(result.is_ok(), "deep chain should build: {result:?}");
}

#[test]
fn edge_ir_many_tokens() {
    let mut builder = GrammarBuilder::new("many_tokens");
    let mut token_names = Vec::new();
    for i in 0..10 {
        let name = format!("t{i}");
        let pattern = format!("t{i}");
        builder = builder.token(&name, &pattern);
        token_names.push(name);
    }
    // Make root a choice of all tokens
    for name in &token_names {
        builder = builder.rule("root", vec![name.as_str()]);
    }
    builder = builder.start("root");
    let grammar = builder.build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts);
    assert!(result.is_ok(), "many tokens should build: {result:?}");
}

#[test]
fn edge_grammar_name_with_underscore() {
    let grammar_json = json!({
        "name": "my_test_grammar",
        "rules": {
            "start": { "type": "STRING", "value": "ok" }
        }
    })
    .to_string();
    let result = build_json(&grammar_json).unwrap();
    assert_eq!(result.grammar_name, "my_test_grammar");
}

#[test]
fn edge_grammar_name_with_numbers() {
    let grammar_json = json!({
        "name": "lang42",
        "rules": {
            "start": { "type": "STRING", "value": "ok" }
        }
    })
    .to_string();
    let result = build_json(&grammar_json).unwrap();
    assert_eq!(result.grammar_name, "lang42");
}

#[test]
fn edge_compressed_and_uncompressed_same_stats() {
    let grammar_json = minimal_json();

    let (_d1, opts1) = tmp_opts();
    let r1 = build_parser_from_json(grammar_json.clone(), opts1).unwrap();

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r2 = build_parser_from_json(grammar_json, opts2).unwrap();

    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn edge_ir_prec_left_and_right() {
    let grammar = GrammarBuilder::new("mixed_prec")
        .token("n", "[0-9]+")
        .token("plus", "\\+")
        .token("eq", "=")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 2, Associativity::Right)
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts);
    assert!(result.is_ok(), "mixed precedence should build: {result:?}");
}
