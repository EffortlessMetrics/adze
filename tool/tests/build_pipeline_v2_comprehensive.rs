//! Comprehensive tests for the build pipeline v2: BuildOptions, BuildResult,
//! BuildStats, determinism, error handling, JSON input, and edge cases.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser, build_parser_from_json};
use tempfile::TempDir;

// ── Helpers ──

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/bpv2_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

fn test_opts_no_compress() -> BuildOptions {
    BuildOptions {
        compress_tables: false,
        ..test_opts()
    }
}

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn two_alt_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("twoalt")
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

fn seq_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
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

fn simple_json_grammar(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "rules": {
            "source": {"type": "PATTERN", "value": "[a-z]+"}
        }
    })
    .to_string()
}

// =========================================================================
// 1. build_parser with valid simple grammars (10 tests)
// =========================================================================

#[test]
fn test_build_single_token_grammar_succeeds() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert_eq!(result.grammar_name, "simple");
}

#[test]
fn test_build_two_alternative_grammar_succeeds() {
    let result = build_parser(two_alt_grammar(), test_opts()).unwrap();
    assert_eq!(result.grammar_name, "twoalt");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_build_chain_grammar_succeeds() {
    let result = build_parser(chain_grammar(), test_opts()).unwrap();
    assert_eq!(result.grammar_name, "chain");
}

#[test]
fn test_build_sequence_grammar_succeeds() {
    let result = build_parser(seq_grammar(), test_opts()).unwrap();
    assert_eq!(result.grammar_name, "seq");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_build_many_tokens_grammar_succeeds() {
    let result = build_parser(many_tokens_grammar(8), test_opts()).unwrap();
    assert_eq!(result.grammar_name, "many");
}

#[test]
fn test_build_grammar_with_regex_token() {
    let grammar = GrammarBuilder::new("regex_tok")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let result = build_parser(grammar, test_opts()).unwrap();
    assert_eq!(result.grammar_name, "regex_tok");
}

#[test]
fn test_build_grammar_with_precedence() {
    let grammar = GrammarBuilder::new("prec")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["b"], 2, Associativity::Right)
        .start("s")
        .build();
    let result = build_parser(grammar, test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn test_build_grammar_produces_nonempty_parser_code() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_build_grammar_produces_nonempty_node_types() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn test_build_grammar_name_preserved_custom() {
    let grammar = GrammarBuilder::new("my_custom_parser")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let result = build_parser(grammar, test_opts()).unwrap();
    assert_eq!(result.grammar_name, "my_custom_parser");
}

// =========================================================================
// 2. BuildResult node_types_json is valid JSON (5 tests)
// =========================================================================

#[test]
fn test_node_types_json_parses_as_valid_json() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn test_node_types_json_is_array_for_two_alt() {
    let result = build_parser(two_alt_grammar(), test_opts()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn test_node_types_json_is_array_for_chain() {
    let result = build_parser(chain_grammar(), test_opts()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn test_node_types_json_contains_objects() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = val.as_array().unwrap();
    // All elements in node_types should be objects
    for entry in arr {
        assert!(entry.is_object(), "node_types entries must be objects");
    }
}

#[test]
fn test_node_types_json_entries_have_type_field() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = val.as_array().unwrap();
    for entry in arr {
        assert!(
            entry.get("type").is_some(),
            "each node_types entry should have a 'type' field"
        );
    }
}

// =========================================================================
// 3. BuildStats field consistency (10 tests)
// =========================================================================

#[test]
fn test_build_stats_state_count_positive() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn test_build_stats_symbol_count_positive() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn test_build_stats_conflict_cells_non_negative() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    // conflict_cells is usize so it is always >= 0; verify it doesn't panic
    let _ = result.build_stats.conflict_cells;
}

#[test]
fn test_build_stats_two_alt_has_more_states_than_one_token() {
    let r_one = build_parser(simple_grammar(), test_opts()).unwrap();
    let r_two = build_parser(two_alt_grammar(), test_opts()).unwrap();
    // Two alternatives should require at least as many states
    assert!(r_two.build_stats.state_count >= r_one.build_stats.state_count);
}

#[test]
fn test_build_stats_chain_has_positive_symbol_count() {
    let result = build_parser(chain_grammar(), test_opts()).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn test_build_stats_seq_has_positive_state_count() {
    let result = build_parser(seq_grammar(), test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn test_build_stats_many_tokens_symbol_count_scales() {
    let r_small = build_parser(many_tokens_grammar(3), test_opts()).unwrap();
    let r_large = build_parser(many_tokens_grammar(8), test_opts()).unwrap();
    assert!(r_large.build_stats.symbol_count >= r_small.build_stats.symbol_count);
}

#[test]
fn test_build_stats_debug_format_includes_fields() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    let dbg = format!("{:?}", result.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn test_build_stats_state_count_consistent_across_compress_modes() {
    let r_comp = build_parser(simple_grammar(), test_opts()).unwrap();
    let r_nocomp = build_parser(simple_grammar(), test_opts_no_compress()).unwrap();
    // Stats come from the same parse table regardless of compression
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
fn test_build_stats_symbol_count_at_least_two_for_simple() {
    // At minimum: the token and the nonterminal + EOF
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(result.build_stats.symbol_count >= 2);
}

// =========================================================================
// 4. build_parser determinism — same input same output (5 tests)
// =========================================================================

#[test]
fn test_determinism_simple_grammar_parser_code() {
    let r1 = build_parser(simple_grammar(), test_opts()).unwrap();
    let r2 = build_parser(simple_grammar(), test_opts()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_determinism_simple_grammar_node_types() {
    let r1 = build_parser(simple_grammar(), test_opts()).unwrap();
    let r2 = build_parser(simple_grammar(), test_opts()).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn test_determinism_simple_grammar_stats() {
    let r1 = build_parser(simple_grammar(), test_opts()).unwrap();
    let r2 = build_parser(simple_grammar(), test_opts()).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn test_determinism_two_alt_grammar() {
    let r1 = build_parser(two_alt_grammar(), test_opts()).unwrap();
    let r2 = build_parser(two_alt_grammar(), test_opts()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn test_determinism_chain_grammar() {
    let r1 = build_parser(chain_grammar(), test_opts()).unwrap();
    let r2 = build_parser(chain_grammar(), test_opts()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

// =========================================================================
// 5. Error handling for malformed inputs (10 tests)
// =========================================================================

#[test]
fn test_error_from_json_empty_string() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json(String::new(), opts).is_err());
}

#[test]
fn test_error_from_json_invalid_json_syntax() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json("{bad json".into(), opts).is_err());
}

#[test]
fn test_error_from_json_array_instead_of_object() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json("[]".into(), opts).is_err());
}

#[test]
fn test_error_from_json_number_literal() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json("42".into(), opts).is_err());
}

#[test]
fn test_error_from_json_null_literal() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json("null".into(), opts).is_err());
}

#[test]
fn test_error_from_json_boolean_literal() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json("true".into(), opts).is_err());
}

#[test]
fn test_error_from_json_string_literal() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json(r#""just a string""#.into(), opts).is_err());
}

#[test]
fn test_error_from_json_missing_rules_key() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    let json = serde_json::json!({"name": "norules"}).to_string();
    assert!(build_parser_from_json(json, opts).is_err());
}

#[test]
fn test_error_from_json_empty_object() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    let result = build_parser_from_json("{}".into(), opts);
    assert!(result.is_err());
}

#[test]
fn test_error_from_json_truncated_json() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    assert!(build_parser_from_json(r#"{"name": "x", "rules": {"#.into(), opts).is_err());
}

// =========================================================================
// 6. BuildOptions configuration (5 tests)
// =========================================================================

#[test]
fn test_build_options_default_has_compress_enabled() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn test_build_options_default_has_emit_artifacts_disabled() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn test_build_options_custom_out_dir() {
    let opts = BuildOptions {
        out_dir: "/custom/path".to_string(),
        ..BuildOptions::default()
    };
    assert_eq!(opts.out_dir, "/custom/path");
}

#[test]
fn test_build_options_all_fields_custom() {
    let opts = BuildOptions {
        out_dir: "/my/dir".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/my/dir");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn test_build_options_clone_preserves_values() {
    let opts = BuildOptions {
        out_dir: "/cloned".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/cloned");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

// =========================================================================
// 7. build_parser_from_json (5 tests)
// =========================================================================

#[test]
fn test_from_json_minimal_pattern_rule() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    let result = build_parser_from_json(simple_json_grammar("jpat"), opts).unwrap();
    assert_eq!(result.grammar_name, "jpat");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_from_json_seq_rule() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    let json = serde_json::json!({
        "name": "jseq",
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
    assert_eq!(result.grammar_name, "jseq");
}

#[test]
fn test_from_json_choice_rule() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    let json = serde_json::json!({
        "name": "jchoice",
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {"type": "STRING", "value": "a"},
                    {"type": "STRING", "value": "b"}
                ]
            }
        }
    })
    .to_string();
    let result = build_parser_from_json(json, opts).unwrap();
    assert_eq!(result.grammar_name, "jchoice");
}

#[test]
fn test_from_json_produces_valid_node_types() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    let result = build_parser_from_json(simple_json_grammar("jnt"), opts).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn test_from_json_stats_are_positive() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..test_opts()
    };
    let result = build_parser_from_json(simple_json_grammar("jstats"), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 8. Edge cases (5 tests)
// =========================================================================

#[test]
fn test_build_with_compression_disabled_still_produces_code() {
    let result = build_parser(simple_grammar(), test_opts_no_compress()).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_build_compressed_and_uncompressed_produce_same_stats() {
    let r_comp = build_parser(simple_grammar(), test_opts()).unwrap();
    let r_nocomp = build_parser(simple_grammar(), test_opts_no_compress()).unwrap();
    assert_eq!(
        r_comp.build_stats.state_count,
        r_nocomp.build_stats.state_count
    );
}

#[test]
fn test_build_grammar_name_with_underscores() {
    let grammar = GrammarBuilder::new("my_parser_v2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let result = build_parser(grammar, test_opts()).unwrap();
    assert_eq!(result.grammar_name, "my_parser_v2");
}

#[test]
fn test_build_grammar_name_with_digits() {
    let grammar = GrammarBuilder::new("lang42")
        .token("z", "z")
        .rule("s", vec!["z"])
        .start("s")
        .build();
    let result = build_parser(grammar, test_opts()).unwrap();
    assert_eq!(result.grammar_name, "lang42");
}

#[test]
fn test_build_result_parser_path_is_nonempty() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(!result.parser_path.is_empty());
}
