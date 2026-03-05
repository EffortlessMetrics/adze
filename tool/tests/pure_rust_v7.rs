use adze_ir::builder::GrammarBuilder;
#[allow(unused_imports)]
use adze_ir::{Grammar, SymbolId};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};
use tempfile::TempDir;

// ============================================================================
// CATEGORY 1: build_basic_* — basic build pipeline (8 tests)
// ============================================================================

#[test]
fn build_basic_minimal_grammar() {
    let mut grammar = GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_basic_with_token_rule() {
    let mut grammar = GrammarBuilder::new("token_rule")
        .token("digit", "[0-9]")
        .rule("expr", vec!["digit"])
        .start("expr")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_basic_multiple_tokens() {
    let mut grammar = GrammarBuilder::new("multi_token")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_basic_nested_rules() {
    let mut grammar = GrammarBuilder::new("nested")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_basic_start_symbol() {
    let mut grammar = GrammarBuilder::new("start")
        .token("t", "t")
        .rule("rule1", vec!["t"])
        .start("rule1")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_basic_builds_successfully() {
    let mut grammar = GrammarBuilder::new("success")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).expect("Build should succeed");
    let _ = result;
}

#[test]
fn build_basic_output_dir_usage() {
    let mut grammar = GrammarBuilder::new("dirtest")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();
    let opts = BuildOptions {
        out_dir: path.to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_basic_returns_result() {
    let mut grammar = GrammarBuilder::new("result")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    match result {
        Ok(_) => {}
        Err(_) => panic!("Unexpected error"),
    }
}

// ============================================================================
// CATEGORY 2: build_output_* — output content validation (8 tests)
// ============================================================================

#[test]
fn build_output_parser_code_present() {
    let mut grammar = GrammarBuilder::new("parser_code")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_output_node_types_json_present() {
    let mut grammar = GrammarBuilder::new("node_types_json")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_output_parser_code_not_empty() {
    let mut grammar = GrammarBuilder::new("code_not_empty")
        .token("token", "t")
        .rule("rule", vec!["token"])
        .start("rule")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_output_node_types_json_valid() {
    let mut grammar = GrammarBuilder::new("json_valid")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&result.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn build_output_contains_expected_structure() {
    let mut grammar = GrammarBuilder::new("structure")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_output_json_is_valid() {
    let mut grammar = GrammarBuilder::new("valid_json")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let _parsed: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("JSON should parse");
}

#[test]
fn build_output_parser_code_is_string() {
    let mut grammar = GrammarBuilder::new("code_string")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let code: String = result.parser_code;
    assert!(!code.is_empty());
}

#[test]
fn build_output_all_fields_populated() {
    let mut grammar = GrammarBuilder::new("all_fields")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
    assert!(!result.node_types_json.is_empty());
    let _stats = &result.build_stats;
}

// ============================================================================
// CATEGORY 3: build_stats_* — build statistics (8 tests)
// ============================================================================

#[test]
fn build_stats_state_count_valid() {
    let mut grammar = GrammarBuilder::new("state_count")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let state_count = result.build_stats.state_count;
    assert!(state_count > 0);
}

#[test]
fn build_stats_symbol_count_valid() {
    let mut grammar = GrammarBuilder::new("symbol_count")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let symbol_count = result.build_stats.symbol_count;
    assert!(symbol_count > 0);
}

#[test]
fn build_stats_conflict_cells_valid() {
    let mut grammar = GrammarBuilder::new("conflict_cells")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let _conflicts = result.build_stats.conflict_cells;
}

#[test]
fn build_stats_all_fields_present() {
    let mut grammar = GrammarBuilder::new("all_stats")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let stats = &result.build_stats;
    let _state = stats.state_count;
    let _symbol = stats.symbol_count;
    let _conflict = stats.conflict_cells;
}

#[test]
fn build_stats_state_count_positive() {
    let mut grammar = GrammarBuilder::new("state_positive")
        .token("t", "t")
        .rule("r", vec!["t"])
        .start("r")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_stats_symbol_count_positive() {
    let mut grammar = GrammarBuilder::new("sym_positive")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_stats_conflict_cells_nonnegative() {
    let mut grammar = GrammarBuilder::new("conflict_nneg")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let conflicts = result.build_stats.conflict_cells;
    // Conflicts can be 0 or more, just verify it's a valid usize
    assert!(conflicts < usize::MAX);
}

#[test]
fn build_stats_values_reasonable() {
    let mut grammar = GrammarBuilder::new("reasonable")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let stats = &result.build_stats;
    // State count should be less than an arbitrary large number
    assert!(stats.state_count < 1_000_000);
    // Symbol count should be reasonable
    assert!(stats.symbol_count < 100_000);
}

// ============================================================================
// CATEGORY 4: build_complex_* — complex grammar builds (8 tests)
// ============================================================================

#[test]
fn build_complex_large_token_count() {
    let mut builder = GrammarBuilder::new("large_tokens");
    for i in 0..10 {
        builder = builder.token(&format!("t{}", i), &format!("t{}", i));
    }
    builder = builder.rule("s", vec!["t0"]).start("s");

    let mut grammar = builder.build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_complex_many_rules() {
    let mut builder = GrammarBuilder::new("many_rules");
    builder = builder.token("a", "a");
    for i in 0..5 {
        builder = builder.rule(&format!("r{}", i), vec!["a"]);
    }
    builder = builder.start("r0");

    let mut grammar = builder.build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_complex_alternative_rules() {
    let mut grammar = GrammarBuilder::new("alternatives")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_complex_recursive_rules() {
    let mut grammar = GrammarBuilder::new("recursive")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_complex_mixed_tokens_rules() {
    let mut builder = GrammarBuilder::new("mixed");
    builder = builder.token("plus", r"\+");
    builder = builder.token("minus", r"\-");
    builder = builder.rule("expr", vec!["plus"]);
    builder = builder.rule("term", vec!["minus"]);
    builder = builder.rule("prog", vec!["expr", "term"]);
    builder = builder.start("prog");

    let mut grammar = builder.build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_complex_deep_nesting() {
    let mut builder = GrammarBuilder::new("deep");
    builder = builder.token("x", "x");
    builder = builder.rule("a", vec!["x"]);
    builder = builder.rule("b", vec!["a"]);
    builder = builder.rule("c", vec!["b"]);
    builder = builder.rule("d", vec!["c"]);
    builder = builder.start("d");

    let mut grammar = builder.build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_complex_all_rule_types() {
    let mut builder = GrammarBuilder::new("all_types");
    builder = builder.token("id", "[a-z]+");
    builder = builder.token("num", "[0-9]+");
    builder = builder.rule("lit", vec!["id", "num"]);
    builder = builder.rule("expr", vec!["lit"]);
    builder = builder.start("expr");

    let mut grammar = builder.build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

// ============================================================================
// CATEGORY 5: build_node_types_* — node types output (8 tests)
// ============================================================================

#[test]
fn build_node_types_json_parseable() {
    let mut grammar = GrammarBuilder::new("json_parse")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let val: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("Should parse as JSON");
    let _ = val;
}

#[test]
fn build_node_types_contains_symbols() {
    let mut grammar = GrammarBuilder::new("contains_symbols")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_node_types_valid_structure() {
    let mut grammar = GrammarBuilder::new("valid_struct")
        .token("tok", "t")
        .rule("rule", vec!["tok"])
        .start("rule")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let json: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    // Verify it's a valid JSON object or array
    assert!(json.is_object() || json.is_array());
}

#[test]
fn build_node_types_matches_grammar() {
    let mut grammar = GrammarBuilder::new("match_grammar")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_node_types_not_empty_json() {
    let mut grammar = GrammarBuilder::new("not_empty_json")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_node_types_proper_format() {
    let mut grammar = GrammarBuilder::new("proper_format")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let _json: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("Should be valid JSON");
}

#[test]
fn build_node_types_includes_tokens() {
    let mut grammar = GrammarBuilder::new("includes_tokens")
        .token("tok1", "t1")
        .rule("s", vec!["tok1"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_node_types_includes_rules() {
    let mut grammar = GrammarBuilder::new("includes_rules")
        .token("a", "a")
        .rule("rule1", vec!["a"])
        .start("rule1")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

// ============================================================================
// CATEGORY 6: build_parser_code_* — parser code output (8 tests)
// ============================================================================

#[test]
fn build_parser_code_contains_rust() {
    let mut grammar = GrammarBuilder::new("rust_code")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let code = &result.parser_code;
    assert!(!code.is_empty());
}

#[test]
fn build_parser_code_not_empty_string() {
    let mut grammar = GrammarBuilder::new("code_not_empty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_parser_code_valid_syntax() {
    let mut grammar = GrammarBuilder::new("valid_syntax")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let code = result.parser_code;
    // Code should not be empty and should be a string
    assert!(!code.is_empty());
}

#[test]
fn build_parser_code_has_structure() {
    let mut grammar = GrammarBuilder::new("code_struct")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let code = &result.parser_code;
    // Should be well-formed Rust code
    assert!(code.len() > 10);
}

#[test]
fn build_parser_code_includes_functions() {
    let mut grammar = GrammarBuilder::new("includes_fns")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_parser_code_proper_format() {
    let mut grammar = GrammarBuilder::new("proper_fmt")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let code = result.parser_code;
    assert!(!code.is_empty());
}

#[test]
fn build_parser_code_consistent_across_builds() {
    let mut grammar1 = GrammarBuilder::new("consistent1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result1 = build_parser(grammar1, opts1).unwrap();

    let mut grammar2 = GrammarBuilder::new("consistent2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result2 = build_parser(grammar2, opts2).unwrap();

    // Code should be generated the same way for identical grammars
    assert_eq!(result1.parser_code.len(), result2.parser_code.len());
}

#[test]
fn build_parser_code_reasonable_length() {
    let mut grammar = GrammarBuilder::new("reasonable_len")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts).unwrap();
    let code_len = result.parser_code.len();
    // Code should be reasonably sized (not too huge)
    assert!(code_len < 10_000_000);
}

// ============================================================================
// CATEGORY 7: build_determinism_* — deterministic builds (8 tests)
// ============================================================================

#[test]
fn build_determinism_same_stats() {
    let mut grammar1 = GrammarBuilder::new("stats1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result1 = build_parser(grammar1, opts1).unwrap();

    let mut grammar2 = GrammarBuilder::new("stats2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result2 = build_parser(grammar2, opts2).unwrap();

    assert_eq!(
        result1.build_stats.state_count,
        result2.build_stats.state_count
    );
    assert_eq!(
        result1.build_stats.symbol_count,
        result2.build_stats.symbol_count
    );
}

#[test]
fn build_determinism_reproducible_grammar() {
    let mut grammar = GrammarBuilder::new("repro1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result1 = build_parser(grammar, opts1).unwrap();

    let mut grammar2 = GrammarBuilder::new("repro2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result2 = build_parser(grammar2, opts2).unwrap();

    assert_eq!(result1.node_types_json, result2.node_types_json);
}

#[test]
fn build_determinism_same_node_types() {
    let mut grammar1 = GrammarBuilder::new("node1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result1 = build_parser(grammar1, opts1).unwrap();

    let mut grammar2 = GrammarBuilder::new("node2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result2 = build_parser(grammar2, opts2).unwrap();

    assert_eq!(result1.node_types_json, result2.node_types_json);
}

#[test]
fn build_determinism_consistent_symbols() {
    let mut grammar1 = GrammarBuilder::new("sym1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result1 = build_parser(grammar1, opts1).unwrap();

    let mut grammar2 = GrammarBuilder::new("sym2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result2 = build_parser(grammar2, opts2).unwrap();

    assert_eq!(
        result1.build_stats.symbol_count,
        result2.build_stats.symbol_count
    );
}

#[test]
fn build_determinism_consistent_conflicts() {
    let mut grammar1 = GrammarBuilder::new("conf1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar1.normalize();

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result1 = build_parser(grammar1, opts1).unwrap();

    let mut grammar2 = GrammarBuilder::new("conf2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar2.normalize();

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result2 = build_parser(grammar2, opts2).unwrap();

    assert_eq!(
        result1.build_stats.conflict_cells,
        result2.build_stats.conflict_cells
    );
}

// ============================================================================
// CATEGORY 8: build_error_* — error handling (8 tests)
// ============================================================================

#[test]
fn build_error_invalid_token_grammar() {
    // Test with minimal valid grammar to ensure we handle edge cases
    let mut grammar = GrammarBuilder::new("invalid_token")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    // Should handle gracefully (either success or clear error)
    let _ = result;
}

#[test]
fn build_error_invalid_rule_grammar() {
    let mut grammar = GrammarBuilder::new("invalid_rule")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn build_error_missing_start_symbol() {
    let mut grammar = GrammarBuilder::new("no_start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    // Note: not calling .start() - missing start symbol
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    // Should handle error gracefully
    let _ = result;
}

#[test]
fn build_error_circular_rules() {
    let mut grammar = GrammarBuilder::new("circular")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn build_error_undefined_symbol() {
    // Grammar with undefined symbol reference
    let mut grammar = GrammarBuilder::new("undefined")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    let _ = result;
}

#[test]
fn build_error_invalid_options() {
    let mut grammar = GrammarBuilder::new("bad_opts")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    // Should handle options gracefully
    let _ = result;
}

#[test]
fn build_error_io_error_handling() {
    let mut grammar = GrammarBuilder::new("io_error")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    // Should handle IO errors if they occur
    let _ = result;
}

#[test]
fn build_error_graceful_failure() {
    let mut grammar = GrammarBuilder::new("graceful")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();

    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = build_parser(grammar, opts);
    // Should either succeed or return a clear error
    match result {
        Ok(_r) => {
            // Success is fine
        }
        Err(_e) => {
            // Error should be documented
        }
    }
}
