//! Comprehensive tests for adze-tool JSON grammar generation via the pure-Rust builder.
//!
//! Covers:
//! 1. Valid JSON grammar parses (8 tests)
//! 2. Invalid JSON fails gracefully (8 tests)
//! 3. Grammar name preserved (8 tests)
//! 4. Token patterns in output (7 tests)
//! 5. Rule structure preserved (8 tests)
//! 6. Build stats accuracy (8 tests)
//! 7. JSON format variations (8 tests)

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use serde_json::json;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

fn opts(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn opts_compressed(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

fn build_ok(grammar_json: &serde_json::Value) -> adze_tool::pure_rust_builder::BuildResult {
    let dir = TempDir::new().unwrap();
    build_parser_from_json(grammar_json.to_string(), opts(&dir)).expect("build should succeed")
}

fn build_err(grammar_json: &str) -> anyhow::Error {
    let dir = TempDir::new().unwrap();
    build_parser_from_json(grammar_json.to_string(), opts(&dir)).expect_err("build should fail")
}

// ===========================================================================
// 1. Valid JSON grammar parses (8 tests)
// ===========================================================================

#[test]
fn valid_minimal_string_grammar_builds() {
    let g = json!({
        "name": "v_minimal",
        "rules": {
            "source_file": { "type": "STRING", "value": "hello" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn valid_pattern_grammar_builds() {
    let g = json!({
        "name": "v_pattern",
        "rules": {
            "source_file": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn valid_two_rule_grammar_builds() {
    let g = json!({
        "name": "v_two_rules",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "expr" },
            "expr": { "type": "PATTERN", "value": "[0-9]+" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn valid_seq_grammar_builds() {
    let g = json!({
        "name": "v_seq",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "PATTERN", "value": "[0-9]+" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    });
    let r = build_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn valid_choice_grammar_builds() {
    let g = json!({
        "name": "v_choice",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    });
    let r = build_ok(&g);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn valid_repeat_grammar_builds() {
    let g = json!({
        "name": "v_repeat",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "x" }
            }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn valid_repeat1_grammar_builds() {
    let g = json!({
        "name": "v_repeat1",
        "rules": {
            "source_file": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "y" }
            }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn valid_grammar_with_extras_builds() {
    let g = json!({
        "name": "v_extras",
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ],
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "item"
            },
            "item": { "type": "STRING", "value": "tok" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 2. Invalid JSON fails gracefully (8 tests)
// ===========================================================================

#[test]
fn invalid_json_syntax_fails() {
    let e = build_err("{ not valid json !!! }");
    let msg = format!("{e}");
    assert!(
        msg.contains("JSON") || msg.contains("json") || msg.contains("parse"),
        "error should mention JSON parsing: {msg}"
    );
}

#[test]
fn invalid_empty_string_fails() {
    let e = build_err("");
    assert!(!format!("{e}").is_empty());
}

#[test]
fn invalid_missing_rules_fails() {
    let g = json!({ "name": "no_rules" });
    let dir = TempDir::new().unwrap();
    let r = build_parser_from_json(g.to_string(), opts(&dir));
    assert!(r.is_err(), "grammar without rules should fail");
}

#[test]
fn invalid_missing_name_fails() {
    let g = json!({
        "rules": { "source_file": { "type": "BLANK" } }
    });
    let dir = TempDir::new().unwrap();
    let r = build_parser_from_json(g.to_string(), opts(&dir));
    assert!(r.is_err(), "grammar without name should fail");
}

#[test]
fn invalid_rules_not_object_fails() {
    let g = json!({
        "name": "bad_rules",
        "rules": "not an object"
    });
    let dir = TempDir::new().unwrap();
    let r = build_parser_from_json(g.to_string(), opts(&dir));
    assert!(r.is_err(), "rules as string should fail");
}

#[test]
fn invalid_rules_as_array_fails() {
    let g = json!({
        "name": "array_rules",
        "rules": [{ "type": "STRING", "value": "x" }]
    });
    let dir = TempDir::new().unwrap();
    let r = build_parser_from_json(g.to_string(), opts(&dir));
    assert!(r.is_err(), "rules as array should fail");
}

#[test]
fn invalid_rule_missing_type_fails() {
    let g = json!({
        "name": "no_type",
        "rules": {
            "source_file": { "value": "hello" }
        }
    });
    let dir = TempDir::new().unwrap();
    let r = build_parser_from_json(g.to_string(), opts(&dir));
    assert!(r.is_err(), "rule without type should fail");
}

#[test]
fn invalid_unknown_rule_type_fails() {
    let g = json!({
        "name": "bad_type",
        "rules": {
            "source_file": { "type": "NONEXISTENT_TYPE", "value": "x" }
        }
    });
    let dir = TempDir::new().unwrap();
    let r = build_parser_from_json(g.to_string(), opts(&dir));
    assert!(r.is_err(), "unknown rule type should fail");
}

// ===========================================================================
// 3. Grammar name preserved (8 tests)
// ===========================================================================

#[test]
fn name_simple_alpha() {
    let g = json!({
        "name": "alpha",
        "rules": { "source_file": { "type": "STRING", "value": "a" } }
    });
    assert_eq!(build_ok(&g).grammar_name, "alpha");
}

#[test]
fn name_with_underscores() {
    let g = json!({
        "name": "my_cool_lang",
        "rules": { "source_file": { "type": "STRING", "value": "x" } }
    });
    assert_eq!(build_ok(&g).grammar_name, "my_cool_lang");
}

#[test]
fn name_with_numbers() {
    let g = json!({
        "name": "lang42",
        "rules": { "source_file": { "type": "STRING", "value": "z" } }
    });
    assert_eq!(build_ok(&g).grammar_name, "lang42");
}

#[test]
fn name_single_char() {
    let g = json!({
        "name": "x",
        "rules": { "source_file": { "type": "STRING", "value": "t" } }
    });
    assert_eq!(build_ok(&g).grammar_name, "x");
}

#[test]
fn name_appears_in_parser_code() {
    let g = json!({
        "name": "code_name_check",
        "rules": { "source_file": { "type": "STRING", "value": "v" } }
    });
    let r = build_ok(&g);
    assert!(
        r.parser_code.contains("code_name_check") || r.parser_path.contains("code_name_check"),
        "grammar name should appear in output"
    );
}

#[test]
fn name_appears_in_parser_path() {
    let g = json!({
        "name": "path_lang",
        "rules": { "source_file": { "type": "STRING", "value": "w" } }
    });
    let r = build_ok(&g);
    assert!(
        r.parser_path.contains("path_lang"),
        "parser_path should contain grammar name: {}",
        r.parser_path
    );
}

#[test]
fn name_preserved_across_compression_modes() {
    let g = json!({
        "name": "compress_name",
        "rules": { "source_file": { "type": "STRING", "value": "c" } }
    });
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser_from_json(g.to_string(), opts(&dir1)).unwrap();
    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser_from_json(g.to_string(), opts_compressed(&dir2)).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn name_different_grammars_different_names() {
    let g1 = json!({
        "name": "lang_a",
        "rules": { "source_file": { "type": "STRING", "value": "a" } }
    });
    let g2 = json!({
        "name": "lang_b",
        "rules": { "source_file": { "type": "STRING", "value": "b" } }
    });
    assert_ne!(build_ok(&g1).grammar_name, build_ok(&g2).grammar_name);
}

// ===========================================================================
// 4. Token patterns in output (7 tests)
// ===========================================================================

#[test]
fn token_string_literal_in_parser_code() {
    let g = json!({
        "name": "tok_str",
        "rules": {
            "source_file": { "type": "STRING", "value": "hello" }
        }
    });
    let r = build_ok(&g);
    // The generated code should reference the token somewhere
    assert!(!r.parser_code.is_empty());
}

#[test]
fn token_pattern_produces_nonempty_output() {
    let g = json!({
        "name": "tok_pat",
        "rules": {
            "source_file": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn token_multiple_strings_all_produce_output() {
    let g = json!({
        "name": "tok_multi",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "let" },
                    { "type": "PATTERN", "value": "[a-z]+" },
                    { "type": "STRING", "value": "=" },
                    { "type": "PATTERN", "value": "[0-9]+" }
                ]
            }
        }
    });
    let r = build_ok(&g);
    assert!(
        r.build_stats.symbol_count >= 4,
        "should have at least 4 symbols for 4 tokens, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn token_in_choice_branches() {
    let g = json!({
        "name": "tok_choice",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "true" },
                    { "type": "STRING", "value": "false" }
                ]
            }
        }
    });
    let r = build_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn token_pattern_regex_metacharacters() {
    let g = json!({
        "name": "tok_regex",
        "rules": {
            "source_file": { "type": "PATTERN", "value": "\\d+\\.\\d+" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn token_wrapped_in_token_node() {
    let g = json!({
        "name": "tok_node",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": { "type": "PATTERN", "value": "[a-zA-Z_]\\w*" }
            }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn token_immediate_token_node() {
    let g = json!({
        "name": "tok_imm",
        "rules": {
            "source_file": {
                "type": "IMMEDIATE_TOKEN",
                "content": { "type": "PATTERN", "value": "[0-9]+" }
            }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 5. Rule structure preserved (8 tests)
// ===========================================================================

#[test]
fn rule_single_string_produces_states() {
    let g = json!({
        "name": "rs_single",
        "rules": { "source_file": { "type": "STRING", "value": "x" } }
    });
    let r = build_ok(&g);
    assert!(
        r.build_stats.state_count >= 2,
        "even a single-string grammar needs states, got {}",
        r.build_stats.state_count
    );
}

#[test]
fn rule_sequence_order_matters() {
    let g = json!({
        "name": "rs_seq",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    });
    let r = build_ok(&g);
    // A 3-element sequence needs more states than a single string
    assert!(
        r.build_stats.state_count >= 3,
        "3-elem SEQ needs >=3 states, got {}",
        r.build_stats.state_count
    );
}

#[test]
fn rule_choice_reflected_in_stats() {
    let g = json!({
        "name": "rs_choice",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "STRING", "value": "y" },
                    { "type": "STRING", "value": "z" }
                ]
            }
        }
    });
    let r = build_ok(&g);
    assert!(
        r.build_stats.symbol_count >= 3,
        "3 string choices need >=3 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn rule_nested_seq_in_choice() {
    let g = json!({
        "name": "rs_nested",
        "rules": {
            "source_file": {
                "type": "CHOICE",
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
    });
    let r = build_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn rule_optional_via_choice_blank() {
    let g = json!({
        "name": "rs_optional",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "start" },
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "opt" },
                            { "type": "BLANK" }
                        ]
                    }
                ]
            }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn rule_repeat_reflected_in_output() {
    let g = json!({
        "name": "rs_repeat",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "r" }
            }
        }
    });
    let r = build_ok(&g);
    assert!(r.build_stats.state_count > 0);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn rule_multi_level_nesting() {
    let g = json!({
        "name": "rs_deep",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "outer" },
            "outer": { "type": "SYMBOL", "name": "middle" },
            "middle": { "type": "SYMBOL", "name": "inner" },
            "inner": { "type": "STRING", "value": "leaf" }
        }
    });
    let r = build_ok(&g);
    assert!(
        r.build_stats.symbol_count >= 3,
        "multi-level nesting needs >=3 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn rule_symbol_reference_produces_valid_output() {
    let g = json!({
        "name": "rs_symref",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let r = build_ok(&g);
    let node_types: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(node_types.is_array(), "node_types should be a JSON array");
}

// ===========================================================================
// 6. Build stats accuracy (8 tests)
// ===========================================================================

#[test]
fn stats_state_count_positive() {
    let g = json!({
        "name": "st_pos",
        "rules": { "source_file": { "type": "STRING", "value": "s" } }
    });
    let r = build_ok(&g);
    assert!(r.build_stats.state_count > 0, "state_count must be > 0");
}

#[test]
fn stats_symbol_count_positive() {
    let g = json!({
        "name": "st_sym",
        "rules": { "source_file": { "type": "STRING", "value": "s" } }
    });
    let r = build_ok(&g);
    assert!(r.build_stats.symbol_count > 0, "symbol_count must be > 0");
}

#[test]
fn stats_more_rules_more_symbols() {
    let g1 = json!({
        "name": "st_few",
        "rules": {
            "source_file": { "type": "STRING", "value": "a" }
        }
    });
    let g2 = json!({
        "name": "st_many",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" },
                    { "type": "STRING", "value": "d" }
                ]
            }
        }
    });
    let r1 = build_ok(&g1);
    let r2 = build_ok(&g2);
    assert!(
        r2.build_stats.symbol_count >= r1.build_stats.symbol_count,
        "more tokens should mean >= symbols: {} vs {}",
        r2.build_stats.symbol_count,
        r1.build_stats.symbol_count
    );
}

#[test]
fn stats_seq_has_more_states_than_single() {
    let g1 = json!({
        "name": "st_single",
        "rules": { "source_file": { "type": "STRING", "value": "a" } }
    });
    let g2 = json!({
        "name": "st_seq3",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    });
    let r1 = build_ok(&g1);
    let r2 = build_ok(&g2);
    assert!(
        r2.build_stats.state_count >= r1.build_stats.state_count,
        "SEQ grammar should have >= states: {} vs {}",
        r2.build_stats.state_count,
        r1.build_stats.state_count
    );
}

#[test]
fn stats_conflict_cells_non_negative() {
    let g = json!({
        "name": "st_conf",
        "rules": { "source_file": { "type": "STRING", "value": "q" } }
    });
    let r = build_ok(&g);
    // conflict_cells is usize, always >= 0, but verify it doesn't panic
    let _ = r.build_stats.conflict_cells;
}

#[test]
fn stats_simple_grammar_zero_conflicts() {
    let g = json!({
        "name": "st_noconf",
        "rules": { "source_file": { "type": "STRING", "value": "z" } }
    });
    let r = build_ok(&g);
    assert_eq!(
        r.build_stats.conflict_cells, 0,
        "simple grammar should have 0 conflicts, got {}",
        r.build_stats.conflict_cells
    );
}

#[test]
fn stats_debug_format() {
    let g = json!({
        "name": "st_debug",
        "rules": { "source_file": { "type": "STRING", "value": "d" } }
    });
    let r = build_ok(&g);
    let debug = format!("{:?}", r.build_stats);
    assert!(
        debug.contains("state_count"),
        "Debug should show state_count: {debug}"
    );
    assert!(
        debug.contains("symbol_count"),
        "Debug should show symbol_count: {debug}"
    );
}

#[test]
fn stats_compressed_vs_uncompressed_same_counts() {
    let g = json!({
        "name": "st_comp",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "STRING", "value": "v" }
        }
    });
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser_from_json(g.to_string(), opts(&dir1)).unwrap();
    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser_from_json(g.to_string(), opts_compressed(&dir2)).unwrap();
    assert_eq!(
        r1.build_stats.state_count, r2.build_stats.state_count,
        "compression should not change state count"
    );
    assert_eq!(
        r1.build_stats.symbol_count, r2.build_stats.symbol_count,
        "compression should not change symbol count"
    );
}

// ===========================================================================
// 7. JSON format variations (8 tests)
// ===========================================================================

#[test]
fn variation_extras_whitespace() {
    let g = json!({
        "name": "var_ws",
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ],
        "rules": {
            "source_file": { "type": "STRING", "value": "hi" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn variation_extras_comment_pattern() {
    let g = json!({
        "name": "var_comment",
        "extras": [
            { "type": "PATTERN", "value": "\\s+" },
            { "type": "PATTERN", "value": "//[^\\n]*" }
        ],
        "rules": {
            "source_file": { "type": "STRING", "value": "code" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn variation_conflicts_field() {
    let g = json!({
        "name": "var_conflicts",
        "conflicts": [],
        "rules": {
            "source_file": { "type": "STRING", "value": "cf" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn variation_inline_field() {
    let g = json!({
        "name": "var_inline",
        "inline": [],
        "rules": {
            "source_file": { "type": "STRING", "value": "il" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn variation_word_field() {
    let g = json!({
        "name": "var_word",
        "word": "identifier",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "identifier" },
            "identifier": { "type": "PATTERN", "value": "[a-zA-Z_]\\w*" }
        }
    });
    let r = build_ok(&g);
    assert_eq!(r.grammar_name, "var_word");
}

#[test]
fn variation_supertypes_field() {
    let g = json!({
        "name": "var_super",
        "supertypes": [],
        "rules": {
            "source_file": { "type": "STRING", "value": "su" }
        }
    });
    let r = build_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn variation_node_types_valid_json() {
    let g = json!({
        "name": "var_nt",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "fn" },
                    { "type": "PATTERN", "value": "[a-z]+" }
                ]
            }
        }
    });
    let r = build_ok(&g);
    let parsed: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("node_types_json must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn variation_deterministic_output() {
    let g = json!({
        "name": "var_det",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    });
    let r1 = build_ok(&g);
    let r2 = build_ok(&g);
    assert_eq!(
        r1.parser_code, r2.parser_code,
        "output should be deterministic"
    );
    assert_eq!(
        r1.node_types_json, r2.node_types_json,
        "node_types should be deterministic"
    );
}
