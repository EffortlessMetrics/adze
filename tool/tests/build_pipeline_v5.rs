//! Build-pipeline v5 integration tests for adze-tool.
//!
//! 58 tests covering JSON grammar → parse tables:
//!   1. Build from valid JSON grammar strings (10 tests)
//!   2. Build with different BuildOptions combinations (8 tests)
//!   3. BuildResult contains expected grammar/table/stats (8 tests)
//!   4. Build stats are reasonable — non-zero counts (8 tests)
//!   5. Invalid JSON produces errors (8 tests)
//!   6. Empty grammar handling (4 tests)
//!   7. Complex grammars — arithmetic, JSON-like, nested (8 tests)
//!   8. Build determinism — same input → same output (4 tests)

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser, build_parser_from_json};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, opts)
}

fn tmp_opts_no_compress() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (dir, opts)
}

fn tmp_opts_with_artifacts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    (dir, opts)
}

fn pattern_json(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "rules": {
            "source": {"type": "PATTERN", "value": "[a-z]+"}
        }
    })
    .to_string()
}

fn string_json(name: &str, value: &str) -> String {
    serde_json::json!({
        "name": name,
        "rules": {
            "source": {"type": "STRING", "value": value}
        }
    })
    .to_string()
}

fn single_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("single_v5")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn two_alt_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("twoalt_v5")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn seq_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("seq_v5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build()
}

fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain_v5")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build()
}

// =========================================================================
// 1. Build from valid JSON grammar strings (10 tests)
// =========================================================================

#[test]
fn v5_json_pattern_rule_builds_successfully() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(pattern_json("pat_v5"), opts).unwrap();
    assert_eq!(result.grammar_name, "pat_v5");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v5_json_string_literal_rule_builds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(string_json("str_v5", "hello"), opts).unwrap();
    assert_eq!(result.grammar_name, "str_v5");
}

#[test]
fn v5_json_seq_of_two_strings() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "seq2_v5",
        "rules": {
            "source": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "alpha"},
                    {"type": "STRING", "value": "beta"}
                ]
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "seq2_v5");
}

#[test]
fn v5_json_choice_of_patterns() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "choice_v5",
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {"type": "PATTERN", "value": "[a-z]+"},
                    {"type": "PATTERN", "value": "[0-9]+"}
                ]
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "choice_v5");
}

#[test]
fn v5_json_repeat_rule() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "rep_v5",
        "rules": {
            "source": {
                "type": "REPEAT",
                "content": {"type": "PATTERN", "value": "[a-z]+"}
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "rep_v5");
}

#[test]
fn v5_json_repeat1_rule() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "rep1_v5",
        "rules": {
            "source": {
                "type": "REPEAT1",
                "content": {"type": "STRING", "value": "x"}
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "rep1_v5");
}

#[test]
fn v5_json_prec_left_rule() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "pleft_v5",
        "rules": {
            "source": {
                "type": "PREC_LEFT",
                "value": 2,
                "content": {"type": "PATTERN", "value": "[a-z]+"}
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "pleft_v5");
}

#[test]
fn v5_json_prec_right_rule() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "pright_v5",
        "rules": {
            "source": {
                "type": "PREC_RIGHT",
                "value": 3,
                "content": {"type": "STRING", "value": "tok"}
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "pright_v5");
}

#[test]
fn v5_json_multi_rule_with_symbols() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "multi_v5",
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {"type": "SYMBOL", "name": "word"},
                    {"type": "SYMBOL", "name": "num"}
                ]
            },
            "word": {"type": "PATTERN", "value": "[a-z]+"},
            "num": {"type": "PATTERN", "value": "[0-9]+"}
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "multi_v5");
    assert!(result.build_stats.symbol_count >= 2);
}

#[test]
fn v5_json_optional_via_choice_blank() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "optblank_v5",
        "rules": {
            "source": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "start"},
                    {
                        "type": "CHOICE",
                        "members": [
                            {"type": "STRING", "value": "mid"},
                            {"type": "BLANK"}
                        ]
                    }
                ]
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "optblank_v5");
}

// =========================================================================
// 2. Build with different BuildOptions combinations (8 tests)
// =========================================================================

#[test]
fn v5_opts_default_compress_is_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn v5_opts_default_emit_artifacts_is_false() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn v5_opts_default_out_dir_nonempty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn v5_opts_custom_all_fields() {
    let opts = BuildOptions {
        out_dir: "/v5/custom".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/v5/custom");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn v5_opts_compress_produces_parser_code() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v5_opts_no_compress_produces_parser_code() {
    let (_dir, opts) = tmp_opts_no_compress();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v5_opts_emit_artifacts_writes_parser_file() {
    let (_dir, opts) = tmp_opts_with_artifacts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let parser_path = std::path::Path::new(&result.parser_path);
    assert!(parser_path.exists());
}

#[test]
fn v5_opts_clone_preserves_values() {
    let opts = BuildOptions {
        out_dir: "/v5/cloned".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/v5/cloned");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

// =========================================================================
// 3. BuildResult contains expected grammar/table/stats (8 tests)
// =========================================================================

#[test]
fn v5_result_grammar_name_matches_input() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "single_v5");
}

#[test]
fn v5_result_parser_code_nonempty() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v5_result_parser_path_nonempty() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_path.is_empty());
}

#[test]
fn v5_result_parser_path_contains_grammar_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.parser_path.contains("single_v5"));
}

#[test]
fn v5_result_node_types_is_valid_json_array() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn v5_result_node_types_entries_are_objects() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(entry.is_object());
    }
}

#[test]
fn v5_result_node_types_entries_have_type_field() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(entry.get("type").is_some());
    }
}

#[test]
fn v5_result_debug_format_includes_grammar_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let dbg = format!("{result:?}");
    assert!(dbg.contains("single_v5"));
}

// =========================================================================
// 4. Build stats are reasonable — non-zero counts (8 tests)
// =========================================================================

#[test]
fn v5_stats_state_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v5_stats_symbol_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v5_stats_symbol_count_at_least_two() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    // At minimum: the token + EOF
    assert!(result.build_stats.symbol_count >= 2);
}

#[test]
fn v5_stats_two_alt_more_symbols_than_single() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(two_alt_grammar(), o2).unwrap();
    assert!(r2.build_stats.symbol_count >= r1.build_stats.symbol_count);
}

#[test]
fn v5_stats_seq_grammar_state_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(seq_grammar(), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v5_stats_consistent_across_compress_modes() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts_no_compress();
    let r_comp = build_parser(single_token_grammar(), o1).unwrap();
    let r_nocomp = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(
        r_comp.build_stats.state_count,
        r_nocomp.build_stats.state_count
    );
    assert_eq!(
        r_comp.build_stats.symbol_count,
        r_nocomp.build_stats.symbol_count
    );
}

#[test]
fn v5_stats_debug_format_includes_all_fields() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let dbg = format!("{:?}", result.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn v5_stats_conflict_cells_is_accessible() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    // conflict_cells is usize, always >= 0; just verify no panic
    let _cells = result.build_stats.conflict_cells;
}

// =========================================================================
// 5. Invalid JSON produces errors (8 tests)
// =========================================================================

#[test]
fn v5_error_empty_string() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json(String::new(), opts).is_err());
}

#[test]
fn v5_error_malformed_json() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("{bad".into(), opts).is_err());
}

#[test]
fn v5_error_json_array() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("[]".into(), opts).is_err());
}

#[test]
fn v5_error_json_number() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("42".into(), opts).is_err());
}

#[test]
fn v5_error_json_null() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("null".into(), opts).is_err());
}

#[test]
fn v5_error_json_boolean() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("true".into(), opts).is_err());
}

#[test]
fn v5_error_json_missing_rules() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({"name": "norules_v5"}).to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

#[test]
fn v5_error_json_rules_not_object() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "badrules_v5",
        "rules": "not_an_object"
    })
    .to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

// =========================================================================
// 6. Empty grammar handling (4 tests)
// =========================================================================

#[test]
fn v5_empty_json_object_errors() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("{}".into(), opts).is_err());
}

#[test]
fn v5_empty_rules_object_errors() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "emptyrules_v5",
        "rules": {}
    })
    .to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

#[test]
fn v5_json_string_literal_errors() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("\"hello\"".into(), opts).is_err());
}

#[test]
fn v5_name_only_no_rules_key_errors() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({"name": "lonely_v5"}).to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

// =========================================================================
// 7. Complex grammars — arithmetic, JSON-like, nested (8 tests)
// =========================================================================

#[test]
fn v5_complex_arithmetic_grammar_via_ir() {
    let grammar = GrammarBuilder::new("arith_v5")
        .token("num", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "arith_v5");
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count >= 3);
}

#[test]
fn v5_complex_nested_seq_in_choice_json() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "nestedsc_v5",
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "SEQ",
                        "members": [
                            {"type": "STRING", "value": "if"},
                            {"type": "SYMBOL", "name": "cond"}
                        ]
                    },
                    {"type": "STRING", "value": "else"}
                ]
            },
            "cond": {"type": "PATTERN", "value": "[a-z]+"}
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "nestedsc_v5");
}

#[test]
fn v5_complex_repeat_inside_seq_json() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "repseq_v5",
        "rules": {
            "source": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "["},
                    {
                        "type": "REPEAT",
                        "content": {"type": "SYMBOL", "name": "item"}
                    },
                    {"type": "STRING", "value": "]"}
                ]
            },
            "item": {"type": "PATTERN", "value": "[a-z]+"}
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "repseq_v5");
}

#[test]
fn v5_complex_json_like_structure() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jsonlike_v5",
        "rules": {
            "source": {"type": "SYMBOL", "name": "value"},
            "value": {
                "type": "CHOICE",
                "members": [
                    {"type": "SYMBOL", "name": "object_lit"},
                    {"type": "SYMBOL", "name": "array_lit"},
                    {"type": "SYMBOL", "name": "primitive"}
                ]
            },
            "object_lit": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "{"},
                    {"type": "STRING", "value": "}"}
                ]
            },
            "array_lit": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "["},
                    {"type": "STRING", "value": "]"}
                ]
            },
            "primitive": {"type": "PATTERN", "value": "[a-z0-9]+"}
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jsonlike_v5");
    assert!(result.build_stats.symbol_count >= 4);
}

#[test]
fn v5_complex_deep_chain_via_ir() {
    let mut builder = GrammarBuilder::new("deep_v5");
    builder = builder.token("leaf", "leaf");
    let depth = 8;
    let names: Vec<String> = (0..depth).map(|i| format!("n{i}")).collect();
    builder = builder.rule(&names[0], vec!["leaf"]);
    for i in 1..depth {
        builder = builder.rule(&names[i], vec![&names[i - 1]]);
    }
    builder = builder.start(&names[depth - 1]);
    let grammar = builder.build();

    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "deep_v5");
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v5_complex_many_alternatives_via_ir() {
    let count = 12;
    let mut builder = GrammarBuilder::new("manyalt_v5");
    let names: Vec<String> = (0..count).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("s", vec![name]);
    }
    builder = builder.start("s");
    let grammar = builder.build();

    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "manyalt_v5");
    assert!(result.build_stats.symbol_count >= count);
}

#[test]
fn v5_complex_mixed_prec_and_plain_via_ir() {
    let grammar = GrammarBuilder::new("mixedp_v5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["b"], 2, Associativity::Right)
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v5_complex_operator_tokens_via_ir() {
    let grammar = GrammarBuilder::new("ops_v5")
        .token("plus", "+")
        .token("minus", "-")
        .token("n", "0")
        .rule("s", vec!["n", "plus", "n"])
        .rule("s", vec!["n", "minus", "n"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "ops_v5");
}

// =========================================================================
// 8. Build determinism — same input → same output (4 tests)
// =========================================================================

#[test]
fn v5_determinism_parser_code_from_ir() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v5_determinism_node_types_from_ir() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn v5_determinism_stats_from_ir() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn v5_determinism_chain_grammar_parser_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(chain_grammar(), o1).unwrap();
    let r2 = build_parser(chain_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v5_determinism_json_roundtrip() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser_from_json(pattern_json("detjson_v5"), o1).unwrap();
    let r2 = build_parser_from_json(pattern_json("detjson_v5"), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}
