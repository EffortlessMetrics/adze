//! Comprehensive tests for the adze-tool codegen pipeline v3.
//!
//! 55+ tests covering:
//! 1. build_parser with simple grammars (10 tests)
//! 2. BuildOptions configuration (8 tests)
//! 3. BuildResult properties (8 tests)
//! 4. BuildStats consistency (5 tests)
//! 5. Error handling for invalid grammars (8 tests)
//! 6. Determinism — same input → same output (8 tests)
//! 7. Token/rule interaction patterns (4 tests)
//! 8. Edge cases (4 tests)

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

fn single_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("single_tok")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn two_alt_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build()
}

fn sequence_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build()
}

fn regex_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("regex_tok")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build()
}

fn prec_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("prec")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["b"], 2, Associativity::Right)
        .start("s")
        .build()
}

fn many_tokens_grammar(count: usize) -> adze_ir::Grammar {
    let mut builder = GrammarBuilder::new("many");
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
// 1. build_parser with simple grammars (10 tests)
// =========================================================================

#[test]
fn v3_simple_single_token_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "single_tok");
}

#[test]
fn v3_simple_two_alternative_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(two_alt_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "two_alt");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v3_simple_chain_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(chain_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "chain");
}

#[test]
fn v3_simple_sequence_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(sequence_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "seq");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v3_simple_regex_token_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(regex_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "regex_tok");
}

#[test]
fn v3_simple_precedence_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(prec_grammar(), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v3_simple_many_tokens_grammar_succeeds() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(many_tokens_grammar(6), opts).unwrap();
    assert_eq!(result.grammar_name, "many");
}

#[test]
fn v3_simple_grammar_produces_nonempty_parser_code() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v3_simple_grammar_produces_nonempty_node_types() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn v3_simple_custom_name_preserved() {
    let grammar = GrammarBuilder::new("my_custom_v3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "my_custom_v3");
}

// =========================================================================
// 2. BuildOptions configuration (8 tests)
// =========================================================================

#[test]
fn v3_opts_default_compress_enabled() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn v3_opts_default_emit_artifacts_disabled() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn v3_opts_default_out_dir_nonempty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn v3_opts_custom_out_dir() {
    let opts = BuildOptions {
        out_dir: "/custom/v3/path".to_string(),
        ..BuildOptions::default()
    };
    assert_eq!(opts.out_dir, "/custom/v3/path");
}

#[test]
fn v3_opts_all_fields_custom() {
    let opts = BuildOptions {
        out_dir: "/v3/dir".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/v3/dir");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn v3_opts_clone_preserves_values() {
    let opts = BuildOptions {
        out_dir: "/cloned/v3".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/cloned/v3");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

#[test]
fn v3_opts_debug_format_includes_fields() {
    let opts = BuildOptions {
        out_dir: "/dbg".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let dbg = format!("{:?}", opts);
    assert!(dbg.contains("out_dir"));
    assert!(dbg.contains("emit_artifacts"));
    assert!(dbg.contains("compress_tables"));
}

#[test]
fn v3_opts_emit_artifacts_creates_files() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let result = build_parser(single_token_grammar(), opts).unwrap();
    // With emit_artifacts=true, parser_path should point to a real file
    let parser_path = std::path::Path::new(&result.parser_path);
    assert!(parser_path.exists());
}

// =========================================================================
// 3. BuildResult properties (8 tests)
// =========================================================================

#[test]
fn v3_result_grammar_name_matches_input() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "single_tok");
}

#[test]
fn v3_result_parser_path_nonempty() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_path.is_empty());
}

#[test]
fn v3_result_parser_code_nonempty() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v3_result_node_types_valid_json() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn v3_result_node_types_entries_are_objects() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = val.as_array().unwrap();
    for entry in arr {
        assert!(entry.is_object(), "node_types entries must be objects");
    }
}

#[test]
fn v3_result_node_types_entries_have_type_field() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = val.as_array().unwrap();
    for entry in arr {
        assert!(
            entry.get("type").is_some(),
            "each node_types entry should have a 'type' field"
        );
    }
}

#[test]
fn v3_result_debug_format_includes_grammar_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    let dbg = format!("{:?}", result);
    assert!(dbg.contains("single_tok"));
}

#[test]
fn v3_result_parser_path_contains_grammar_name() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(
        result.parser_path.contains("single_tok"),
        "parser_path should contain the grammar name"
    );
}

// =========================================================================
// 4. BuildStats consistency (5 tests)
// =========================================================================

#[test]
fn v3_stats_state_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v3_stats_symbol_count_positive() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v3_stats_symbol_count_at_least_two() {
    // At minimum: the token, the nonterminal, and EOF
    let (_dir, opts) = tmp_opts();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(result.build_stats.symbol_count >= 2);
}

#[test]
fn v3_stats_consistent_across_compress_modes() {
    let (_dir1, opts1) = tmp_opts();
    let (_dir2, opts2) = tmp_opts_no_compress();
    let r_comp = build_parser(single_token_grammar(), opts1).unwrap();
    let r_nocomp = build_parser(single_token_grammar(), opts2).unwrap();
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
fn v3_stats_many_tokens_symbol_count_scales() {
    let (_dir1, opts1) = tmp_opts();
    let (_dir2, opts2) = tmp_opts();
    let r_small = build_parser(many_tokens_grammar(3), opts1).unwrap();
    let r_large = build_parser(many_tokens_grammar(8), opts2).unwrap();
    assert!(r_large.build_stats.symbol_count >= r_small.build_stats.symbol_count);
}

// =========================================================================
// 5. Error handling for invalid grammars (8 tests)
// =========================================================================

#[test]
fn v3_error_json_empty_string() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json(String::new(), opts).is_err());
}

#[test]
fn v3_error_json_invalid_syntax() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("{bad json".into(), opts).is_err());
}

#[test]
fn v3_error_json_array_instead_of_object() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("[]".into(), opts).is_err());
}

#[test]
fn v3_error_json_number_literal() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("42".into(), opts).is_err());
}

#[test]
fn v3_error_json_null_literal() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("null".into(), opts).is_err());
}

#[test]
fn v3_error_json_boolean_literal() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("true".into(), opts).is_err());
}

#[test]
fn v3_error_json_missing_rules_key() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({"name": "norules"}).to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

#[test]
fn v3_error_json_empty_object() {
    let (_dir, opts) = tmp_opts();
    assert!(build_parser_from_json("{}".into(), opts).is_err());
}

// =========================================================================
// 6. Determinism — same input → same output (8 tests)
// =========================================================================

#[test]
fn v3_determinism_parser_code_single_token() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v3_determinism_node_types_single_token() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn v3_determinism_stats_single_token() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(single_token_grammar(), o1).unwrap();
    let r2 = build_parser(single_token_grammar(), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn v3_determinism_two_alt_grammar_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(two_alt_grammar(), o1).unwrap();
    let r2 = build_parser(two_alt_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v3_determinism_two_alt_grammar_node_types() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(two_alt_grammar(), o1).unwrap();
    let r2 = build_parser(two_alt_grammar(), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn v3_determinism_chain_grammar_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(chain_grammar(), o1).unwrap();
    let r2 = build_parser(chain_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v3_determinism_sequence_grammar_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser(sequence_grammar(), o1).unwrap();
    let r2 = build_parser(sequence_grammar(), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v3_determinism_json_grammar_code() {
    let (_d1, o1) = tmp_opts();
    let (_d2, o2) = tmp_opts();
    let r1 = build_parser_from_json(simple_json("det_json"), o1).unwrap();
    let r2 = build_parser_from_json(simple_json("det_json"), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// =========================================================================
// 7. Token/rule interaction patterns (4 tests)
// =========================================================================

#[test]
fn v3_token_rule_multiple_tokens_in_sequence() {
    let grammar = GrammarBuilder::new("multi_seq")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["x", "y", "z"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "multi_seq");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v3_token_rule_shared_token_across_rules() {
    let grammar = GrammarBuilder::new("shared_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["b", "a"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v3_token_rule_single_char_tokens() {
    let grammar = GrammarBuilder::new("char_toks")
        .token("p", "+")
        .token("m", "-")
        .token("n", "0")
        .rule("s", vec!["n", "p", "n"])
        .rule("s", vec!["n", "m", "n"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "char_toks");
}

#[test]
fn v3_token_rule_precedence_left_right() {
    let grammar = GrammarBuilder::new("lr_prec")
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
// 8. Edge cases (4 tests)
// =========================================================================

#[test]
fn v3_edge_compression_disabled_still_produces_code() {
    let (_dir, opts) = tmp_opts_no_compress();
    let result = build_parser(single_token_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v3_edge_grammar_name_with_underscores() {
    let grammar = GrammarBuilder::new("my_parser_v3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "my_parser_v3");
}

#[test]
fn v3_edge_grammar_name_with_digits() {
    let grammar = GrammarBuilder::new("lang99")
        .token("z", "z")
        .rule("s", vec!["z"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "lang99");
}

#[test]
fn v3_edge_json_seq_rule() {
    let (_dir, opts) = tmp_opts();
    let json = serde_json::json!({
        "name": "jseq_v3",
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
    assert_eq!(result.grammar_name, "jseq_v3");
}
