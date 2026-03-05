//! Comprehensive tests for BuildOptions, BuildResult, BuildStats, and ToolError
//! in the adze-tool crate (v2).

use adze_ir::builder::GrammarBuilder;
use adze_tool::error::ToolError;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser, build_parser_from_json};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_opts(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn simple_json_grammar(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "word": null,
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "value"
            },
            "value": {
                "type": "PATTERN",
                "value": "\\d+"
            }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ],
        "conflicts": [],
        "precedences": [],
        "externals": [],
        "inline": [],
        "supertypes": []
    })
    .to_string()
}

fn builder_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("+", r"\+")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "NUMBER"])
        .start("expr")
        .build()
}

// =========================================================================
// 1. BuildOptions default values (8 tests)
// =========================================================================

#[test]
fn test_default_compress_tables_is_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn test_default_emit_artifacts_is_false() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn test_default_out_dir_is_nonempty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn test_default_out_dir_fallback_without_env() {
    // When OUT_DIR is not set, fallback should contain "target"
    // (unless something else sets it in the test environment)
    let opts = BuildOptions::default();
    // Just verify it is a non-empty string
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn test_custom_options_all_fields() {
    let opts = BuildOptions {
        out_dir: "/custom/path".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/custom/path");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn test_custom_options_compress_true() {
    let opts = BuildOptions {
        out_dir: String::new(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert!(opts.compress_tables);
}

#[test]
fn test_custom_options_emit_artifacts_true() {
    let opts = BuildOptions {
        out_dir: String::new(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert!(opts.emit_artifacts);
}

#[test]
fn test_custom_options_empty_out_dir() {
    let opts = BuildOptions {
        out_dir: String::new(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(opts.out_dir.is_empty());
}

// =========================================================================
// 2. BuildOptions Debug/Clone (5 tests)
// =========================================================================

#[test]
fn test_options_debug_contains_field_names() {
    let opts = BuildOptions {
        out_dir: "/tmp/debug_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let dbg = format!("{:?}", opts);
    assert!(dbg.contains("out_dir"));
    assert!(dbg.contains("emit_artifacts"));
    assert!(dbg.contains("compress_tables"));
}

#[test]
fn test_options_debug_contains_values() {
    let opts = BuildOptions {
        out_dir: "/sentinel/path".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let dbg = format!("{:?}", opts);
    assert!(dbg.contains("/sentinel/path"));
    assert!(dbg.contains("true"));
    assert!(dbg.contains("false"));
}

#[test]
fn test_options_clone_produces_equal_values() {
    let opts = BuildOptions {
        out_dir: "/a/b/c".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(opts.out_dir, cloned.out_dir);
    assert_eq!(opts.emit_artifacts, cloned.emit_artifacts);
    assert_eq!(opts.compress_tables, cloned.compress_tables);
}

#[test]
fn test_options_clone_is_independent() {
    let opts = BuildOptions {
        out_dir: "/original".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let mut cloned = opts.clone();
    cloned.out_dir = "/modified".to_string();
    cloned.emit_artifacts = false;
    cloned.compress_tables = true;
    // Original should be unchanged
    assert_eq!(opts.out_dir, "/original");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn test_options_clone_default() {
    let opts = BuildOptions::default();
    let cloned = opts.clone();
    assert_eq!(opts.out_dir, cloned.out_dir);
    assert_eq!(opts.emit_artifacts, cloned.emit_artifacts);
    assert_eq!(opts.compress_tables, cloned.compress_tables);
}

// =========================================================================
// 3. BuildResult field access (8 tests)
// =========================================================================

#[test]
fn test_result_grammar_name_matches_input() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("alpha");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    assert_eq!(result.grammar_name, "alpha");
}

#[test]
fn test_result_parser_path_is_nonempty() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("beta");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    assert!(!result.parser_path.is_empty());
}

#[test]
fn test_result_parser_code_is_nonempty() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("gamma");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_result_node_types_json_is_valid_json() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("delta");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_result_parser_code_contains_grammar_name() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("myparser");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    assert!(result.parser_code.contains("myparser"));
}

#[test]
fn test_result_build_stats_present() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("epsilon");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    // build_stats should be accessible
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn test_result_debug_impl() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("zeta");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    let dbg = format!("{:?}", result);
    assert!(dbg.contains("grammar_name"));
    assert!(dbg.contains("zeta"));
}

#[test]
fn test_result_from_ir_grammar() {
    let dir = TempDir::new().unwrap();
    let grammar = builder_grammar("ir_test");
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert_eq!(result.grammar_name, "ir_test");
    assert!(!result.parser_code.is_empty());
}

// =========================================================================
// 4. BuildStats properties (8 tests)
// =========================================================================

#[test]
fn test_stats_state_count_positive() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("stats1");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn test_stats_symbol_count_positive() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("stats2");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn test_stats_conflict_cells_non_negative() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("stats3");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    // conflict_cells is usize, so always >= 0; just verify accessible
    let _ = result.build_stats.conflict_cells;
}

#[test]
fn test_stats_debug_impl() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("stats4");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    let dbg = format!("{:?}", result.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn test_stats_simple_grammar_has_few_states() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("small");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    // A very simple grammar should have a relatively small number of states
    assert!(result.build_stats.state_count < 100);
}

#[test]
fn test_stats_simple_grammar_has_few_symbols() {
    let dir = TempDir::new().unwrap();
    let json = simple_json_grammar("small2");
    let result = build_parser_from_json(json, temp_opts(&dir)).unwrap();
    assert!(result.build_stats.symbol_count < 50);
}

#[test]
fn test_stats_ir_grammar_state_count() {
    let dir = TempDir::new().unwrap();
    let grammar = builder_grammar("ir_stats");
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn test_stats_ir_grammar_symbol_count() {
    let dir = TempDir::new().unwrap();
    let grammar = builder_grammar("ir_sym");
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert!(result.build_stats.symbol_count >= 2);
}

// =========================================================================
// 5. BuildError / ToolError types and messages (8 tests)
// =========================================================================

#[test]
fn test_tool_error_multiple_word_rules_display() {
    let err = ToolError::MultipleWordRules;
    let msg = format!("{}", err);
    assert!(msg.contains("word rule"));
}

#[test]
fn test_tool_error_multiple_precedence_display() {
    let err = ToolError::MultiplePrecedenceAttributes;
    let msg = format!("{}", err);
    assert!(msg.contains("prec"));
}

#[test]
fn test_tool_error_expected_string_literal() {
    let err = ToolError::ExpectedStringLiteral {
        context: "token".to_string(),
        actual: "42".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("token"));
    assert!(msg.contains("42"));
}

#[test]
fn test_tool_error_grammar_validation() {
    let err = ToolError::grammar_validation("missing start symbol");
    let msg = format!("{}", err);
    assert!(msg.contains("missing start symbol"));
}

#[test]
fn test_tool_error_string_too_long() {
    let err = ToolError::string_too_long("extraction", 999);
    let msg = format!("{}", err);
    assert!(msg.contains("extraction"));
    assert!(msg.contains("999"));
}

#[test]
fn test_tool_error_complex_symbols_not_normalized() {
    let err = ToolError::complex_symbols_not_normalized("compression");
    let msg = format!("{}", err);
    assert!(msg.contains("compression"));
}

#[test]
fn test_tool_error_other_variant() {
    let err = ToolError::Other("custom problem".to_string());
    let msg = format!("{}", err);
    assert_eq!(msg, "custom problem");
}

#[test]
fn test_tool_error_from_string() {
    let err: ToolError = "something broke".into();
    let msg = format!("{}", err);
    assert!(msg.contains("something broke"));
}

// =========================================================================
// 6. Build with different options (8 tests)
// =========================================================================

#[test]
fn test_build_with_compression_enabled() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let json = simple_json_grammar("compressed");
    let result = build_parser_from_json(json, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_build_with_compression_disabled() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let json = simple_json_grammar("uncompressed");
    let result = build_parser_from_json(json, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn test_build_with_artifacts_emitted() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let json = simple_json_grammar("artifacts");
    let result = build_parser_from_json(json, opts).unwrap();
    // When emit_artifacts is true, additional files should be created
    let grammar_dir = dir.path().join("grammar_artifacts");
    assert!(grammar_dir.exists());
    assert_eq!(result.grammar_name, "artifacts");
}

#[test]
fn test_build_artifacts_include_ir_json() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let json = simple_json_grammar("artir");
    build_parser_from_json(json, opts).unwrap();
    let ir_path = dir.path().join("grammar_artir").join("grammar.ir.json");
    assert!(ir_path.exists());
}

#[test]
fn test_build_artifacts_include_node_types() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let json = simple_json_grammar("artnt");
    build_parser_from_json(json, opts).unwrap();
    let nt_path = dir.path().join("grammar_artnt").join("NODE_TYPES.json");
    assert!(nt_path.exists());
}

#[test]
fn test_build_without_artifacts_no_ir_file() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let json = simple_json_grammar("noart");
    build_parser_from_json(json, opts).unwrap();
    let ir_path = dir.path().join("grammar_noart").join("grammar.ir.json");
    assert!(!ir_path.exists());
}

#[test]
fn test_build_ir_grammar_uncompressed() {
    let dir = TempDir::new().unwrap();
    let grammar = builder_grammar("ir_uncomp");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "ir_uncomp");
}

#[test]
fn test_build_ir_grammar_compressed() {
    let dir = TempDir::new().unwrap();
    let grammar = builder_grammar("ir_comp");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "ir_comp");
}

// =========================================================================
// 7. Build result consistency (5 tests)
// =========================================================================

#[test]
fn test_same_grammar_produces_same_name() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let json1 = simple_json_grammar("consistent");
    let json2 = simple_json_grammar("consistent");
    let r1 = build_parser_from_json(json1, temp_opts(&dir1)).unwrap();
    let r2 = build_parser_from_json(json2, temp_opts(&dir2)).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn test_same_grammar_produces_same_node_types() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let json1 = simple_json_grammar("consistent2");
    let json2 = simple_json_grammar("consistent2");
    let r1 = build_parser_from_json(json1, temp_opts(&dir1)).unwrap();
    let r2 = build_parser_from_json(json2, temp_opts(&dir2)).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn test_same_grammar_produces_same_stats() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let json1 = simple_json_grammar("consistent3");
    let json2 = simple_json_grammar("consistent3");
    let r1 = build_parser_from_json(json1, temp_opts(&dir1)).unwrap();
    let r2 = build_parser_from_json(json2, temp_opts(&dir2)).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn test_different_names_produce_different_grammar_name() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let r1 = build_parser_from_json(simple_json_grammar("aaa"), temp_opts(&dir1)).unwrap();
    let r2 = build_parser_from_json(simple_json_grammar("bbb"), temp_opts(&dir2)).unwrap();
    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn test_compressed_and_uncompressed_same_stats() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let json1 = simple_json_grammar("cmp1");
    let json2 = simple_json_grammar("cmp1");
    let opts1 = BuildOptions {
        compress_tables: true,
        ..temp_opts(&dir1)
    };
    let opts2 = BuildOptions {
        compress_tables: false,
        ..temp_opts(&dir2)
    };
    let r1 = build_parser_from_json(json1, opts1).unwrap();
    let r2 = build_parser_from_json(json2, opts2).unwrap();
    // Stats come from the same parse table, so they should match
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

// =========================================================================
// 8. Edge cases (5 tests)
// =========================================================================

#[test]
fn test_invalid_json_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("not valid json".to_string(), temp_opts(&dir));
    assert!(result.is_err());
}

#[test]
fn test_empty_json_object_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("{}".to_string(), temp_opts(&dir));
    assert!(result.is_err());
}

#[test]
fn test_json_missing_rules_returns_error() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "name": "norules",
        "rules": {}
    })
    .to_string();
    let result = build_parser_from_json(json, temp_opts(&dir));
    assert!(result.is_err());
}

#[test]
fn test_tool_error_is_debug() {
    let err = ToolError::Other("test".to_string());
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("Other"));
}

#[test]
fn test_tool_error_nested_option_display() {
    let err = ToolError::NestedOptionType;
    let msg = format!("{}", err);
    assert!(msg.contains("Option"));
}
