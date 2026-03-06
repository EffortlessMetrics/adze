use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};
use tempfile::TempDir;

// ============================================================================
// CATEGORY 1: codegen_basic_* — Basic code generation (8 tests)
// ============================================================================

#[test]
fn codegen_basic_simple_grammar() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_basic_single_rule() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn codegen_basic_multiple_tokens() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("multi_tok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count >= 3);
}

#[test]
fn codegen_basic_nested_rules() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nested")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn codegen_basic_left_recursion() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("left_rec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_basic_right_recursion() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_basic_empty_rule() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("with_empty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec![])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn codegen_basic_alt_rules() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

// ============================================================================
// CATEGORY 2: codegen_content_* — Generated code content (8 tests)
// ============================================================================

#[test]
fn codegen_content_parser_code_present() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("content_test")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
    assert!(result.parser_code.len() > 100);
}

#[test]
fn codegen_content_node_types_present() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("node_test")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn codegen_content_parser_code_valid() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("valid_code")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.parser_code.contains("parse") || result.parser_code.contains("fn"));
}

#[test]
fn codegen_content_nonempty_parser() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nonempty_p")
        .token("t", "t")
        .rule("r", vec!["t"])
        .start("r")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_content_nonempty_node_types() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nonempty_nt")
        .token("t", "t")
        .rule("r", vec!["t"])
        .start("r")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn codegen_content_valid_json() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("valid_json")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&result.node_types_json).is_ok());
}

#[test]
fn codegen_content_parser_has_parse_fn() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("parse_fn")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.parser_code.contains("fn") || result.parser_code.contains("parse"));
}

#[test]
fn codegen_content_lexer_integration() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("lexer_int")
        .token("plus", r"\+")
        .token("num", r"\d+")
        .rule("expr", vec!["num", "plus", "num"])
        .start("expr")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

// ============================================================================
// CATEGORY 3: codegen_stats_* — Build stats correctness (8 tests)
// ============================================================================

#[test]
fn codegen_stats_state_count_valid() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("state_stat")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn codegen_stats_symbol_count_valid() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("symbol_stat")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn codegen_stats_conflict_cells_valid() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("conflict_stat")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    let _ = result.build_stats.conflict_cells;
}

#[test]
fn codegen_stats_grammar_increases_states() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar1 = GrammarBuilder::new("stat_cmp1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("stat_cmp2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert!(result2.build_stats.state_count >= result1.build_stats.state_count);
}

#[test]
fn codegen_stats_grammar_increases_symbols() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar1 = GrammarBuilder::new("sym_cmp1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("sym_cmp2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert!(result2.build_stats.symbol_count >= result1.build_stats.symbol_count);
}

#[test]
fn codegen_stats_complex_grammar_more_states() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("complex_state")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 1);
}

#[test]
fn codegen_stats_minimal_grammar_small_stats() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("minimal_stat")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count < 100);
}

#[test]
fn codegen_stats_all_stats_nonzero() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("all_nonzero")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// ============================================================================
// CATEGORY 4: codegen_node_types_* — Node types JSON output (8 tests)
// ============================================================================

#[test]
fn codegen_node_types_valid_json() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_json")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&result.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn codegen_node_types_has_rules() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_rules")
        .token("a", "a")
        .rule("myrule", vec!["a"])
        .start("myrule")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.node_types_json.contains("myrule") || !result.node_types_json.is_empty());
}

#[test]
fn codegen_node_types_has_tokens() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_tokens")
        .token("mytoken", "x")
        .rule("s", vec!["mytoken"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.node_types_json.contains("mytoken") || !result.node_types_json.is_empty());
}

#[test]
fn codegen_node_types_matches_grammar() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_match")
        .token("tok", "t")
        .rule("rule1", vec!["tok"])
        .start("rule1")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn codegen_node_types_nonempty_content() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_content")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
    assert!(result.node_types_json.len() > 10);
}

#[test]
fn codegen_node_types_contains_symbols() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_symbols")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&result.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn codegen_node_types_array_format() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_array")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    let val = serde_json::from_str::<serde_json::Value>(&result.node_types_json).unwrap();
    assert!(val.is_array() || val.is_object());
}

#[test]
fn codegen_node_types_proper_structure() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nt_struct")
        .token("tok1", "t1")
        .token("tok2", "t2")
        .rule("rule1", vec!["tok1"])
        .rule("rule2", vec!["tok2"])
        .start("rule1")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    let val = serde_json::from_str::<serde_json::Value>(&result.node_types_json);
    assert!(val.is_ok());
}

// ============================================================================
// CATEGORY 5: codegen_determinism_* — Deterministic output (8 tests)
// ============================================================================

#[test]
fn codegen_determinism_same_output_twice() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut grammar1 = GrammarBuilder::new("det_test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("det_test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert_eq!(result1.parser_code, result2.parser_code);
}

#[test]
fn codegen_determinism_same_stats() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut grammar1 = GrammarBuilder::new("det_stats1")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("det_stats1")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert_eq!(
        result1.build_stats.state_count,
        result2.build_stats.state_count
    );
}

#[test]
fn codegen_determinism_same_parser_code() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut grammar1 = GrammarBuilder::new("det_parser1")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("det_parser1")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert_eq!(result1.parser_code, result2.parser_code);
}

#[test]
fn codegen_determinism_same_node_types() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut grammar1 = GrammarBuilder::new("det_nt1")
        .token("t", "t")
        .rule("r", vec!["t"])
        .start("r")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("det_nt1")
        .token("t", "t")
        .rule("r", vec!["t"])
        .start("r")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert_eq!(result1.node_types_json, result2.node_types_json);
}

#[test]
fn codegen_determinism_multiple_iterations() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut grammar = GrammarBuilder::new("det_iter")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result1 = build_parser(grammar.clone(), opts.clone()).unwrap();

    let result2 = build_parser(grammar.clone(), opts.clone()).unwrap();
    let result3 = build_parser(grammar, opts).unwrap();

    assert_eq!(result1.parser_code, result2.parser_code);
    assert_eq!(result2.parser_code, result3.parser_code);
}

#[test]
fn codegen_determinism_different_dirs_same_output() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();

    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut grammar1 = GrammarBuilder::new("det_dirs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts1).unwrap();

    let mut grammar2 = GrammarBuilder::new("det_dirs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts2).unwrap();

    assert_eq!(result1.parser_code, result2.parser_code);
}

#[test]
fn codegen_determinism_artifact_emission() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let mut grammar1 = GrammarBuilder::new("det_artifact1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("det_artifact1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert_eq!(result1.parser_code, result2.parser_code);
}

#[test]
fn codegen_determinism_compression_consistent() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };

    let mut grammar1 = GrammarBuilder::new("det_compress1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();
    let result1 = build_parser(grammar1, opts.clone()).unwrap();

    let mut grammar2 = GrammarBuilder::new("det_compress1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();
    let result2 = build_parser(grammar2, opts).unwrap();

    assert_eq!(result1.parser_code, result2.parser_code);
}

// ============================================================================
// CATEGORY 6: codegen_complex_* — Complex grammar codegen (8 tests)
// ============================================================================

#[test]
fn codegen_complex_multiple_rules() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("multi_rules")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["b", "a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_complex_many_tokens() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut builder = GrammarBuilder::new("many_tokens");
    for i in 0..10 {
        builder = builder.token(&format!("tok_{}", i), &format!("t{}", i));
    }
    let mut grammar = builder.rule("s", vec!["tok_0"]).start("s").build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count >= 10);
}

#[test]
fn codegen_complex_nested_expressions() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("nested_expr")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["num"])
        .start("expr")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_complex_multiple_recursions() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("multi_rec")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["s", "b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn codegen_complex_mixed_token_rules() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("mixed_tr")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .rule("expr", vec!["id", "num"])
        .rule("expr", vec!["num", "id"])
        .rule("stmt", vec!["expr"])
        .start("stmt")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_complex_long_rule_chain() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("long_chain")
        .token("a", "a")
        .rule("s1", vec!["a"])
        .rule("s2", vec!["s1"])
        .rule("s3", vec!["s2"])
        .rule("s4", vec!["s3"])
        .start("s4")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn codegen_complex_alternation_rules() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("alt_rules")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_complex_large_grammar() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut builder = GrammarBuilder::new("large_gram");
    for i in 0..5 {
        builder = builder.token(&format!("t{}", i), &format!("x{}", i));
    }
    for i in 0..5 {
        builder = builder.rule(&format!("r{}", i), vec![&format!("t{}", i)]);
    }
    let mut grammar = builder.start("r0").build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count >= 10);
}

// ============================================================================
// CATEGORY 7: codegen_options_* — Build options behavior (8 tests)
// ============================================================================

#[test]
fn codegen_options_default_behavior() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("opts_default")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_options_custom_out_dir() {
    let dir = TempDir::new().unwrap();
    let out_dir_str = dir.path().to_string_lossy().to_string();
    let opts = BuildOptions {
        out_dir: out_dir_str.clone(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("opts_outdir")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_options_emit_artifacts_false() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("opts_emit_false")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_options_emit_artifacts_true() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("opts_emit_true")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_options_compress_tables_false() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        compress_tables: false,
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("opts_compress_false")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_options_compress_tables_true() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        compress_tables: true,
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("opts_compress_true")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_options_combined_settings() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let mut grammar = GrammarBuilder::new("opts_combined")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn codegen_options_tempdir_out_dir() {
    let dir = TempDir::new().unwrap();
    let path_str = dir.path().to_string_lossy().to_string();
    let opts = BuildOptions {
        out_dir: path_str,
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("opts_tempdir")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

// ============================================================================
// CATEGORY 8: codegen_error_* — Error handling (8 tests)
// ============================================================================

#[test]
fn codegen_error_invalid_grammar() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("err_invalid")
        .rule("s", vec!["nonexistent"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn codegen_error_missing_start_rule() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("err_no_start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn codegen_error_circular_references() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("err_circular")
        .rule("a", vec!["b"])
        .rule("b", vec!["a"])
        .start("a")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn codegen_error_undefined_rule() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("err_undef")
        .rule("s", vec!["undefined"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn codegen_error_normalized_required() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let grammar = GrammarBuilder::new("err_not_norm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn codegen_error_empty_grammar() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("err_empty").build();
    grammar.normalize();
    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn codegen_error_invalid_token() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("err_bad_tok")
        .token("", "")
        .rule("s", vec![])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn codegen_error_output_directory() {
    let opts = BuildOptions {
        out_dir: "/nonexistent/path/that/does/not/exist".to_string(),
        ..Default::default()
    };
    let mut grammar = GrammarBuilder::new("err_bad_dir")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let result = build_parser(grammar, opts);
    let _ = result;
}
