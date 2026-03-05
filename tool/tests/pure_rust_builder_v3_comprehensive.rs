//! Comprehensive tests for `adze_tool::pure_rust_builder` (v3).
//!
//! 55+ tests covering:
//! 1. Build simple grammar (8 tests)
//! 2. BuildOptions defaults and configuration (8 tests)
//! 3. BuildResult properties (8 tests)
//! 4. BuildStats validation (5 tests)
//! 5. Error handling for build failures (8 tests)
//! 6. Invalid grammar inputs (8 tests)
//! 7. Grammar JSON format edge cases (5 tests)
//! 8. Complex grammars (5 tests)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser, build_parser_from_json};
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

fn tmp_opts_compressed() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, opts)
}

fn tmp_opts_emit() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    (dir, opts)
}

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single_tok")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build()
}

fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a"])
        .rule("root", vec!["b"])
        .start("root")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("root", vec!["b"])
        .start("root")
        .build()
}

fn seq_grammar() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("root", vec!["a", "b", "c"])
        .start("root")
        .build()
}

fn simple_json_str(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "item"
            },
            "item": {
                "type": "PATTERN",
                "value": "[a-z]+"
            }
        }
    })
    .to_string()
}

// ===========================================================================
// 1. Build simple grammar (8 tests)
// ===========================================================================

#[test]
fn v3_build_single_token_ok() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts);
    assert!(
        result.is_ok(),
        "single-token grammar should build: {result:?}"
    );
}

#[test]
fn v3_build_single_token_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "single_tok");
}

#[test]
fn v3_build_single_token_code_nonempty() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v3_build_two_alternatives() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(two_alt_grammar(), opts);
    assert!(result.is_ok());
}

#[test]
fn v3_build_chain_grammar() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(chain_grammar(), opts);
    assert!(result.is_ok());
}

#[test]
fn v3_build_sequence_grammar() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(seq_grammar(), opts);
    assert!(result.is_ok());
}

#[test]
fn v3_build_arith_grammar() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(arith_grammar(), opts);
    assert!(result.is_ok(), "arith grammar should build: {result:?}");
}

#[test]
fn v3_build_with_compression() {
    let (_dir, opts) = tmp_opts_compressed();
    let result = build_parser(single_token_grammar(), opts);
    assert!(
        result.is_ok(),
        "compressed build should succeed: {result:?}"
    );
}

// ===========================================================================
// 2. BuildOptions defaults and configuration (8 tests)
// ===========================================================================

#[test]
fn v3_opts_default_compress_tables_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn v3_opts_default_emit_artifacts_false() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn v3_opts_default_out_dir_nonempty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn v3_opts_clone_preserves_fields() {
    let opts = BuildOptions {
        out_dir: "/tmp/test_clone".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/tmp/test_clone");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

#[test]
fn v3_opts_debug_fmt() {
    let opts = BuildOptions::default();
    let dbg = format!("{opts:?}");
    assert!(dbg.contains("BuildOptions"));
}

#[test]
fn v3_opts_custom_out_dir() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(
        opts.out_dir
            .contains(dir.path().file_name().unwrap().to_str().unwrap())
    );
}

#[test]
fn v3_opts_emit_artifacts_creates_files() {
    let (_dir, opts) = tmp_opts_emit();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    // When emit_artifacts is true, the parser path should exist
    let path = std::path::Path::new(&result.parser_path);
    assert!(
        path.exists(),
        "parser file should exist at {}",
        result.parser_path
    );
}

#[test]
fn v3_opts_no_emit_still_writes_parser() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let path = std::path::Path::new(&result.parser_path);
    assert!(
        path.exists(),
        "parser file should exist even without emit_artifacts"
    );
}

// ===========================================================================
// 3. BuildResult properties (8 tests)
// ===========================================================================

#[test]
fn v3_result_grammar_name_matches() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(arith_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "arith");
}

#[test]
fn v3_result_parser_path_contains_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(arith_grammar(), opts).unwrap();
    assert!(
        result.parser_path.contains("arith"),
        "parser path should contain grammar name: {}",
        result.parser_path
    );
}

#[test]
fn v3_result_parser_code_is_valid_rust_tokens() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    // The parser code should contain something recognizable as Rust
    assert!(
        result.parser_code.contains("static")
            || result.parser_code.contains("const")
            || result.parser_code.contains("fn")
            || result.parser_code.contains("LANGUAGE"),
        "parser code should contain Rust-like tokens"
    );
}

#[test]
fn v3_result_node_types_is_valid_json() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array(), "NODE_TYPES should be a JSON array");
}

#[test]
fn v3_result_node_types_has_entries() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty(), "NODE_TYPES should have at least one entry");
}

#[test]
fn v3_result_debug_fmt() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let dbg = format!("{result:?}");
    assert!(dbg.contains("BuildResult"));
}

#[test]
fn v3_result_deterministic_name() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn v3_result_deterministic_node_types() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ===========================================================================
// 4. BuildStats validation (5 tests)
// ===========================================================================

#[test]
fn v3_stats_state_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(
        result.build_stats.state_count > 0,
        "should have at least one state"
    );
}

#[test]
fn v3_stats_symbol_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(
        result.build_stats.symbol_count > 0,
        "should have at least one symbol"
    );
}

#[test]
fn v3_stats_arith_more_states_than_single() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(arith_grammar(), o2).unwrap();
    assert!(
        r2.build_stats.state_count >= r1.build_stats.state_count,
        "arith grammar should have >= states as single-token ({} vs {})",
        r2.build_stats.state_count,
        r1.build_stats.state_count,
    );
}

#[test]
fn v3_stats_arith_more_symbols_than_single() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(arith_grammar(), o2).unwrap();
    assert!(
        r2.build_stats.symbol_count >= r1.build_stats.symbol_count,
        "arith grammar should have >= symbols as single-token ({} vs {})",
        r2.build_stats.symbol_count,
        r1.build_stats.symbol_count,
    );
}

#[test]
fn v3_stats_debug_fmt() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let dbg = format!("{:?}", result.build_stats);
    assert!(
        dbg.contains("state_count"),
        "debug output should show state_count"
    );
    assert!(
        dbg.contains("symbol_count"),
        "debug output should show symbol_count"
    );
}

// ===========================================================================
// 5. Error handling for build failures (8 tests)
// ===========================================================================

#[test]
fn v3_err_empty_json_string() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(String::new(), opts);
    assert!(result.is_err(), "empty JSON string should fail");
}

#[test]
fn v3_err_invalid_json_syntax() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json("{not valid json".to_string(), opts);
    assert!(result.is_err(), "invalid JSON should fail");
}

#[test]
fn v3_err_json_missing_name() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "rules": {
            "source_file": { "type": "PATTERN", "value": "[a-z]+" }
        }
    })
    .to_string();
    // Missing "name" field — may still parse with default name or fail
    let result = build_parser_from_json(input, opts);
    // Whether it succeeds or fails, the function should not panic
    let _ = result;
}

#[test]
fn v3_err_json_missing_rules() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "norules"
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_err(), "JSON with no rules should fail");
}

#[test]
fn v3_err_json_empty_rules() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "emptyrules",
        "rules": {}
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_err(), "JSON with empty rules should fail");
}

#[test]
fn v3_err_json_null_body() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json("null".to_string(), opts);
    assert!(result.is_err(), "null JSON body should fail");
}

#[test]
fn v3_err_json_number_body() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json("42".to_string(), opts);
    assert!(result.is_err(), "numeric JSON body should fail");
}

#[test]
fn v3_err_json_array_body() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json("[1, 2, 3]".to_string(), opts);
    assert!(result.is_err(), "JSON array body should fail");
}

// ===========================================================================
// 6. Invalid grammar inputs (8 tests)
// ===========================================================================

#[test]
fn v3_invalid_json_string_value_body() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(r#""hello""#.to_string(), opts);
    assert!(result.is_err(), "JSON string value should fail");
}

#[test]
fn v3_invalid_json_rules_not_object() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "bad",
        "rules": "not_an_object"
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_err(), "rules as string should fail");
}

#[test]
fn v3_invalid_json_rules_is_array() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "bad",
        "rules": [1, 2, 3]
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_err(), "rules as array should fail");
}

#[test]
fn v3_invalid_json_rule_bad_type() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "badtype",
        "rules": {
            "source_file": { "type": "NONEXISTENT_TYPE", "value": "abc" }
        }
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_err(), "unknown rule type should fail");
}

#[test]
fn v3_invalid_json_rule_missing_type() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "notype",
        "rules": {
            "source_file": { "value": "abc" }
        }
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_err(), "rule without type field should fail");
}

#[test]
fn v3_invalid_json_symbol_references_missing() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "dangling",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "nonexistent" }
        }
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_err(), "dangling symbol reference should fail");
}

#[test]
fn v3_invalid_json_deeply_nested_empty() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "deep",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": []
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    // Empty CHOICE members is an edge case; should either fail or produce minimal grammar
    let _ = result;
}

#[test]
fn v3_invalid_json_boolean_body() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json("true".to_string(), opts);
    assert!(result.is_err(), "boolean JSON body should fail");
}

// ===========================================================================
// 7. Grammar JSON format edge cases (5 tests)
// ===========================================================================

#[test]
fn v3_json_simple_pattern() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(simple_json_str("pat_test"), opts);
    assert!(
        result.is_ok(),
        "simple pattern JSON should build: {result:?}"
    );
}

#[test]
fn v3_json_grammar_name_preserved() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(simple_json_str("my_grammar"), opts).unwrap();
    assert_eq!(result.grammar_name, "my_grammar");
}

#[test]
fn v3_json_with_extras_field() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "with_extras",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "PATTERN", "value": "[a-z]+" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ]
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(
        result.is_ok(),
        "grammar with extras should build: {result:?}"
    );
}

#[test]
fn v3_json_with_word_field() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "with_word",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "identifier" },
            "identifier": { "type": "PATTERN", "value": "[a-z_]+" }
        },
        "word": "identifier"
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(result.is_ok(), "grammar with word should build: {result:?}");
}

#[test]
fn v3_json_choice_with_multiple_members() {
    let (_dir, opts) = tmp_opts();
    let input = json!({
        "name": "multi_choice",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "alpha" },
                    { "type": "SYMBOL", "name": "beta" }
                ]
            },
            "alpha": { "type": "PATTERN", "value": "[a-z]+" },
            "beta": { "type": "PATTERN", "value": "[0-9]+" }
        }
    })
    .to_string();
    let result = build_parser_from_json(input, opts);
    assert!(
        result.is_ok(),
        "multi-choice grammar should build: {result:?}"
    );
}

// ===========================================================================
// 8. Complex grammars (5 tests)
// ===========================================================================

#[test]
fn v3_complex_multi_token_sequence() {
    let (_dir, opts) = tmp_opts();
    let grammar = GrammarBuilder::new("multi_seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("root", vec!["a", "b", "c", "d"])
        .start("root")
        .build();
    let result = build_parser(grammar, opts);
    assert!(result.is_ok(), "4-token sequence should build: {result:?}");
}

#[test]
fn v3_complex_many_alternatives() {
    let (_dir, opts) = tmp_opts();
    let grammar = GrammarBuilder::new("many_alt")
        .token("t0", "a")
        .token("t1", "b")
        .token("t2", "c")
        .token("t3", "d")
        .token("t4", "e")
        .rule("root", vec!["t0"])
        .rule("root", vec!["t1"])
        .rule("root", vec!["t2"])
        .rule("root", vec!["t3"])
        .rule("root", vec!["t4"])
        .start("root")
        .build();
    let result = build_parser(grammar, opts);
    assert!(
        result.is_ok(),
        "5-alternative grammar should build: {result:?}"
    );
}

#[test]
fn v3_complex_nested_non_terminals() {
    let (_dir, opts) = tmp_opts();
    let grammar = GrammarBuilder::new("nested")
        .token("x", "x")
        .rule("d", vec!["x"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let result = build_parser(grammar, opts);
    assert!(
        result.is_ok(),
        "deeply nested grammar should build: {result:?}"
    );
}

#[test]
fn v3_complex_arith_with_compression() {
    let (_dir, opts) = tmp_opts_compressed();
    let result = build_parser(arith_grammar(), opts);
    assert!(
        result.is_ok(),
        "arith with compression should build: {result:?}"
    );
}

#[test]
fn v3_complex_scale_10_tokens() {
    let (_dir, opts) = tmp_opts();
    let mut builder = GrammarBuilder::new("scale10");
    for i in 0..10 {
        let name: &str = Box::leak(format!("tok{i}").into_boxed_str());
        builder = builder.token(name, name).rule("root", vec![name]);
    }
    let grammar = builder.start("root").build();
    let result = build_parser(grammar, opts);
    assert!(result.is_ok(), "10-token grammar should build: {result:?}");
}
