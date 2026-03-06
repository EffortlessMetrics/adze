//! Pipeline v4 integration tests for adze-tool.
//!
//! 55+ tests covering:
//!   1. IR → GLR → tablegen full pipeline (12 tests)
//!   2. JSON → pipeline roundtrip (10 tests)
//!   3. BuildOptions configuration (8 tests)
//!   4. BuildStats accuracy and scaling (8 tests)
//!   5. Determinism — same input → same output (7 tests)
//!   6. Error handling for invalid input (10 tests)

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

fn single_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("single_v4")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn two_alt_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("two_alt_v4")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain_v4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build()
}

fn seq_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("seq_v4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build()
}

fn regex_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("regex_v4")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build()
}

fn prec_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("prec_v4")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["b"], 2, Associativity::Right)
        .start("s")
        .build()
}

fn many_tokens_grammar(count: usize) -> adze_ir::Grammar {
    let mut builder = GrammarBuilder::new("many_v4");
    let names: Vec<String> = (0..count).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("s", vec![name]);
    }
    builder = builder.start("s");
    builder.build()
}

fn deep_chain_grammar(depth: usize) -> adze_ir::Grammar {
    let mut builder = GrammarBuilder::new("deep_v4");
    builder = builder.token("leaf", "leaf");
    let names: Vec<String> = (0..depth).map(|i| format!("n{i}")).collect();
    // n0 -> leaf
    builder = builder.rule(&names[0], vec!["leaf"]);
    // n_{i+1} -> n_i
    for i in 1..depth {
        builder = builder.rule(&names[i], vec![&names[i - 1]]);
    }
    builder = builder.start(&names[depth - 1]);
    builder.build()
}

fn simple_json(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "rules": {
            "source": {"type": "PATTERN", "value": "[a-z]+"}
        }
    })
    .to_string()
}

// =========================================================================
// 1. IR → GLR → tablegen full pipeline (12 tests)
// =========================================================================

#[test]
fn v4_pipeline_single_token_produces_parser() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "single_v4");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v4_pipeline_two_alternatives_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(two_alt_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "two_alt_v4");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v4_pipeline_chain_rule_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(chain_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "chain_v4");
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v4_pipeline_sequence_rule_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(seq_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "seq_v4");
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn v4_pipeline_regex_token_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(regex_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "regex_v4");
}

#[test]
fn v4_pipeline_precedence_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(prec_grammar(), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v4_pipeline_many_tokens_8_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(many_tokens_grammar(8), opts).unwrap();
    assert_eq!(result.grammar_name, "many_v4");
}

#[test]
fn v4_pipeline_deep_chain_5_levels() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(deep_chain_grammar(5), opts).unwrap();
    assert_eq!(result.grammar_name, "deep_v4");
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v4_pipeline_deep_chain_10_levels() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(deep_chain_grammar(10), opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v4_pipeline_shared_token_across_rules() {
    let grammar = GrammarBuilder::new("shared_v4")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["b", "a"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "shared_v4");
}

#[test]
fn v4_pipeline_single_char_operator_tokens() {
    let grammar = GrammarBuilder::new("ops_v4")
        .token("plus", "+")
        .token("minus", "-")
        .token("n", "0")
        .rule("s", vec!["n", "plus", "n"])
        .rule("s", vec!["n", "minus", "n"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "ops_v4");
}

#[test]
fn v4_pipeline_mixed_precedence_and_plain_rules() {
    let grammar = GrammarBuilder::new("mixed_v4")
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

// =========================================================================
// 2. JSON → pipeline roundtrip (10 tests)
// =========================================================================

#[test]
fn v4_json_simple_pattern_roundtrip() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(simple_json("jp_v4"), opts).unwrap();
    assert_eq!(result.grammar_name, "jp_v4");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v4_json_string_literal_rule() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jstr_v4",
        "rules": {
            "source": {"type": "STRING", "value": "hello"}
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jstr_v4");
}

#[test]
fn v4_json_seq_rule_roundtrip() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jseq_v4",
        "rules": {
            "source": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "hello"},
                    {"type": "STRING", "value": "world"}
                ]
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jseq_v4");
}

#[test]
fn v4_json_choice_rule_roundtrip() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jchoice_v4",
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {"type": "STRING", "value": "foo"},
                    {"type": "STRING", "value": "bar"}
                ]
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jchoice_v4");
}

#[test]
fn v4_json_repeat_rule_roundtrip() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jrepeat_v4",
        "rules": {
            "source": {
                "type": "REPEAT",
                "content": {"type": "PATTERN", "value": "[a-z]+"}
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jrepeat_v4");
}

#[test]
fn v4_json_repeat1_rule_roundtrip() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jrepeat1_v4",
        "rules": {
            "source": {
                "type": "REPEAT1",
                "content": {"type": "PATTERN", "value": "[0-9]+"}
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jrepeat1_v4");
}

#[test]
fn v4_json_optional_rule_roundtrip() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jopt_v4",
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
    assert_eq!(result.grammar_name, "jopt_v4");
}

#[test]
fn v4_json_prec_left_rule_roundtrip() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jprec_v4",
        "rules": {
            "source": {
                "type": "PREC_LEFT",
                "value": 1,
                "content": {"type": "PATTERN", "value": "[a-z]+"}
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jprec_v4");
}

#[test]
fn v4_json_node_types_is_valid_array() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(simple_json("jnt_v4"), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn v4_json_multi_rule_grammar() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jmulti_v4",
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {"type": "SYMBOL", "name": "word"},
                    {"type": "SYMBOL", "name": "digit"}
                ]
            },
            "word": {"type": "PATTERN", "value": "[a-z]+"},
            "digit": {"type": "PATTERN", "value": "[0-9]+"}
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jmulti_v4");
    assert!(result.build_stats.symbol_count >= 2);
}

// =========================================================================
// 3. BuildOptions configuration (8 tests)
// =========================================================================

#[test]
fn v4_opts_default_compress_enabled() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn v4_opts_default_emit_artifacts_disabled() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn v4_opts_default_out_dir_nonempty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn v4_opts_custom_all_fields() {
    let opts = BuildOptions {
        out_dir: "/v4/custom".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/v4/custom");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn v4_opts_clone_preserves_values() {
    let opts = BuildOptions {
        out_dir: "/v4/cloned".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/v4/cloned");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

#[test]
fn v4_opts_debug_format_includes_all_fields() {
    let opts = BuildOptions {
        out_dir: "/v4/dbg".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let dbg = format!("{opts:?}");
    assert!(dbg.contains("out_dir"));
    assert!(dbg.contains("emit_artifacts"));
    assert!(dbg.contains("compress_tables"));
}

#[test]
fn v4_opts_emit_artifacts_writes_parser_file() {
    let (_dir, opts) = tmp_opts_with_artifacts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let parser_path = std::path::Path::new(&result.parser_path);
    assert!(parser_path.exists());
}

#[test]
fn v4_opts_no_compress_still_produces_code() {
    let (_dir, opts) = tmp_opts_no_compress();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

// =========================================================================
// 4. BuildStats accuracy and scaling (8 tests)
// =========================================================================

#[test]
fn v4_stats_state_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v4_stats_symbol_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v4_stats_symbol_count_at_least_two() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.symbol_count >= 2);
}

#[test]
fn v4_stats_two_alt_at_least_as_many_states_as_single() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(two_alt_grammar(), o2).unwrap();
    assert!(r2.build_stats.state_count >= r1.build_stats.state_count);
}

#[test]
fn v4_stats_many_tokens_symbol_count_scales() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r_small = build_parser(many_tokens_grammar(3), o1).unwrap();
    let r_large = build_parser(many_tokens_grammar(10), o2).unwrap();
    assert!(r_large.build_stats.symbol_count > r_small.build_stats.symbol_count);
}

#[test]
fn v4_stats_consistent_across_compress_modes() {
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
fn v4_stats_debug_format_includes_fields() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let dbg = format!("{:?}", result.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn v4_stats_conflict_cells_accessible() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    // conflict_cells is usize so always >= 0; verify it doesn't panic
    let _ = result.build_stats.conflict_cells;
}

// =========================================================================
// 5. Determinism — same input → same output (7 tests)
// =========================================================================

#[test]
fn v4_determinism_parser_code_single_token() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v4_determinism_node_types_single_token() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn v4_determinism_stats_single_token() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn v4_determinism_two_alt_parser_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(two_alt_grammar(), o1).unwrap();
    let r2 = build_parser(two_alt_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v4_determinism_chain_parser_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(chain_grammar(), o1).unwrap();
    let r2 = build_parser(chain_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v4_determinism_seq_parser_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(seq_grammar(), o1).unwrap();
    let r2 = build_parser(seq_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v4_determinism_json_grammar_roundtrip() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser_from_json(simple_json("det_v4"), o1).unwrap();
    let r2 = build_parser_from_json(simple_json("det_v4"), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// =========================================================================
// 6. Error handling for invalid input (10 tests)
// =========================================================================

#[test]
fn v4_error_json_empty_string() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json(String::new(), opts).is_err());
}

#[test]
fn v4_error_json_invalid_syntax() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("{bad json".into(), opts).is_err());
}

#[test]
fn v4_error_json_array_instead_of_object() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("[]".into(), opts).is_err());
}

#[test]
fn v4_error_json_number_literal() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("42".into(), opts).is_err());
}

#[test]
fn v4_error_json_null_literal() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("null".into(), opts).is_err());
}

#[test]
fn v4_error_json_boolean_literal() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("true".into(), opts).is_err());
}

#[test]
fn v4_error_json_missing_rules_key() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({"name": "norules"}).to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

#[test]
fn v4_error_json_empty_object() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("{}".into(), opts).is_err());
}

#[test]
fn v4_error_json_string_literal() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("\"hello\"".into(), opts).is_err());
}

#[test]
fn v4_error_json_rules_is_not_object() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "bad_rules_v4",
        "rules": "not_an_object"
    })
    .to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

// =========================================================================
// 7. BuildResult structural properties (extra coverage)
// =========================================================================

#[test]
fn v4_result_parser_path_nonempty() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_path.is_empty());
}

#[test]
fn v4_result_parser_path_contains_grammar_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(
        result.parser_path.contains("single_v4"),
        "parser_path should reference the grammar name"
    );
}

#[test]
fn v4_result_node_types_entries_are_objects() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(entry.is_object(), "node_types entries must be objects");
    }
}

#[test]
fn v4_result_node_types_entries_have_type_field() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "each node_types entry should have a 'type' field"
        );
    }
}

#[test]
fn v4_result_debug_format_includes_grammar_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let dbg = format!("{result:?}");
    assert!(dbg.contains("single_v4"));
}

#[test]
fn v4_result_grammar_name_with_underscores() {
    let grammar = GrammarBuilder::new("my_parser_v4")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "my_parser_v4");
}

#[test]
fn v4_result_grammar_name_with_digits() {
    let grammar = GrammarBuilder::new("lang44")
        .token("z", "z")
        .rule("s", vec!["z"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "lang44");
}

#[test]
fn v4_result_json_and_ir_both_produce_node_types_array() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r_ir = build_parser(single_token_grammar(), o1).unwrap();
    let r_json = build_parser_from_json(simple_json("cmp_v4"), o2).unwrap();
    let v_ir: serde_json::Value = serde_json::from_str(&r_ir.node_types_json).unwrap();
    let v_json: serde_json::Value = serde_json::from_str(&r_json.node_types_json).unwrap();
    assert!(v_ir.is_array());
    assert!(v_json.is_array());
}
