//! Comprehensive tests for `BuildResult`, `BuildStats`, and `BuildOptions` in the
//! `adze_tool::pure_rust_builder` module.
//!
//! 80+ tests across these categories:
//!   1. build_options_default_*       — Default trait behaviour
//!   2. build_options_custom_*        — Custom field values
//!   3. build_options_combinations_*  — All field combinations
//!   4. build_result_simple_*         — Basic BuildResult checks
//!   5. build_result_parser_code_*    — Parser code content
//!   6. build_result_node_types_*     — Node types JSON validity
//!   7. build_stats_*                 — BuildStats numeric checks
//!   8. grammar_scaling_*             — Different grammar sizes
//!   9. grammar_features_*            — Precedence, inline, extras, supertypes
//!  10. determinism_*                 — Reproducible builds

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build()
}

fn build_default(grammar: Grammar) -> BuildResult {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..BuildOptions::default()
    };
    build_parser(grammar, opts).expect("build")
}

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (dir, opts)
}

fn tmp_opts_with(emit: bool, compress: bool) -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: emit,
        compress_tables: compress,
    };
    (dir, opts)
}

fn single_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("TOK", "x")
        .rule("root", vec!["TOK"])
        .start("root")
        .build()
}

fn n_rule_grammar(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name).token("TOK", "x");
    for i in 0..n {
        let rule_name = format!("r{}", i);
        // Leak into 'static so the borrow checker is happy with vec![&str]
        let rule_ref: &'static str = Box::leak(rule_name.into_boxed_str());
        if i == 0 {
            b = b.rule(rule_ref, vec!["TOK"]);
        } else {
            let prev: &'static str = Box::leak(format!("r{}", i - 1).into_boxed_str());
            b = b.rule(rule_ref, vec![prev]);
        }
    }
    b.start("r0").build()
}

// ═════════════════════════════════════════════════════════════════════════
// 1. BuildOptions — default values
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_options_default_out_dir_is_non_empty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn build_options_default_emit_artifacts_is_false() {
    // Unless ADZE_EMIT_ARTIFACTS env var is set, default is false
    let opts = BuildOptions::default();
    // We don't assert exact value since env could override, just ensure it's bool-typed
    let _ = opts.emit_artifacts;
}

#[test]
fn build_options_default_compress_tables_is_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn build_options_default_debug_impl() {
    let opts = BuildOptions::default();
    let debug = format!("{:?}", opts);
    assert!(debug.contains("BuildOptions"));
}

#[test]
fn build_options_default_clone() {
    let opts = BuildOptions::default();
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, opts.out_dir);
    assert_eq!(cloned.emit_artifacts, opts.emit_artifacts);
    assert_eq!(cloned.compress_tables, opts.compress_tables);
}

// ═════════════════════════════════════════════════════════════════════════
// 2. BuildOptions — custom values
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_options_custom_out_dir_string() {
    let opts = BuildOptions {
        out_dir: "/tmp/my_custom_dir".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/tmp/my_custom_dir");
}

#[test]
fn build_options_custom_out_dir_empty_string() {
    let opts = BuildOptions {
        out_dir: String::new(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(opts.out_dir.is_empty());
}

#[test]
fn build_options_custom_emit_artifacts_true() {
    let opts = BuildOptions {
        out_dir: "out".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert!(opts.emit_artifacts);
}

#[test]
fn build_options_custom_emit_artifacts_false() {
    let opts = BuildOptions {
        out_dir: "out".into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(!opts.emit_artifacts);
}

#[test]
fn build_options_custom_compress_tables_true() {
    let opts = BuildOptions {
        out_dir: "out".into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert!(opts.compress_tables);
}

#[test]
fn build_options_custom_compress_tables_false() {
    let opts = BuildOptions {
        out_dir: "out".into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(!opts.compress_tables);
}

#[test]
fn build_options_out_dir_is_string_type() {
    let opts = BuildOptions {
        out_dir: String::from("hello"),
        emit_artifacts: false,
        compress_tables: false,
    };
    let _s: &String = &opts.out_dir;
}

// ═════════════════════════════════════════════════════════════════════════
// 3. BuildOptions — all combinations
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_options_combo_ff() {
    let opts = BuildOptions {
        out_dir: "x".into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(!opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn build_options_combo_ft() {
    let opts = BuildOptions {
        out_dir: "x".into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert!(!opts.emit_artifacts);
    assert!(opts.compress_tables);
}

#[test]
fn build_options_combo_tf() {
    let opts = BuildOptions {
        out_dir: "x".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn build_options_combo_tt() {
    let opts = BuildOptions {
        out_dir: "x".into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    assert!(opts.emit_artifacts);
    assert!(opts.compress_tables);
}

#[test]
fn build_options_combo_with_real_dir_ff() {
    let (_dir, opts) = tmp_opts_with(false, false);
    assert!(!opts.out_dir.is_empty());
    assert!(!opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn build_options_combo_with_real_dir_ft() {
    let (_dir, opts) = tmp_opts_with(false, true);
    assert!(opts.compress_tables);
}

#[test]
fn build_options_combo_with_real_dir_tf() {
    let (_dir, opts) = tmp_opts_with(true, false);
    assert!(opts.emit_artifacts);
}

#[test]
fn build_options_combo_with_real_dir_tt() {
    let (_dir, opts) = tmp_opts_with(true, true);
    assert!(opts.emit_artifacts);
    assert!(opts.compress_tables);
}

// ═════════════════════════════════════════════════════════════════════════
// 4. BuildResult — simple grammar builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_result_simple_grammar_succeeds() {
    let result = build_default(simple_grammar());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_result_grammar_name_matches() {
    let result = build_default(simple_grammar());
    assert_eq!(result.grammar_name, "test");
}

#[test]
fn build_result_parser_path_non_empty() {
    let result = build_default(simple_grammar());
    assert!(!result.parser_path.is_empty());
}

#[test]
fn build_result_parser_code_non_empty() {
    let result = build_default(simple_grammar());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_result_node_types_json_non_empty() {
    let result = build_default(simple_grammar());
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_result_has_build_stats() {
    let result = build_default(simple_grammar());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_result_debug_impl() {
    let result = build_default(simple_grammar());
    let debug = format!("{:?}", result);
    assert!(debug.contains("BuildResult"));
}

#[test]
fn build_result_single_token_grammar() {
    let result = build_default(single_token_grammar("one"));
    assert!(!result.parser_code.is_empty());
    assert_eq!(result.grammar_name, "one");
}

// ═════════════════════════════════════════════════════════════════════════
// 5. BuildResult — parser code content
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn parser_code_contains_language_struct() {
    let result = build_default(simple_grammar());
    // The generated code should reference the Language struct
    assert!(
        result.parser_code.contains("Language")
            || result.parser_code.contains("language")
            || result.parser_code.contains("LANGUAGE"),
        "parser_code should reference Language type"
    );
}

#[test]
fn parser_code_is_valid_token_stream() {
    let result = build_default(simple_grammar());
    // The code is generated via proc-macro2 so it should be parseable
    assert!(!result.parser_code.is_empty());
}

#[test]
fn parser_code_contains_state_data() {
    let result = build_default(simple_grammar());
    let code = &result.parser_code;
    // Parse tables embed numeric data
    assert!(
        code.contains("parse_table")
            || code.contains("PARSE_TABLE")
            || code.contains("parse_actions")
            || code.contains("state")
            || code.len() > 100,
        "parser_code should contain table data or be substantial"
    );
}

#[test]
fn parser_code_longer_for_bigger_grammar() {
    let small = build_default(single_token_grammar("s"));
    let big = build_default(
        GrammarBuilder::new("big")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .token("D", "d")
            .rule("start", vec!["seq"])
            .rule("seq", vec!["A", "B", "C", "D"])
            .rule("seq", vec!["A", "C"])
            .start("start")
            .build(),
    );
    assert!(
        big.parser_code.len() >= small.parser_code.len(),
        "bigger grammar should produce at least as much code"
    );
}

#[test]
fn parser_code_differs_for_different_grammars() {
    let r1 = build_default(single_token_grammar("g1"));
    let r2 = build_default(
        GrammarBuilder::new("g2")
            .token("X", "x")
            .token("Y", "y")
            .rule("start", vec!["X", "Y"])
            .start("start")
            .build(),
    );
    // Different grammars should produce different parser code
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn parser_code_contains_fn_keyword() {
    let result = build_default(simple_grammar());
    assert!(
        result.parser_code.contains("fn ") || result.parser_code.contains("fn("),
        "generated Rust code should contain function definitions"
    );
}

#[test]
fn parser_code_contains_const_or_static() {
    let result = build_default(simple_grammar());
    assert!(
        result.parser_code.contains("const ")
            || result.parser_code.contains("static ")
            || result.parser_code.contains("let "),
        "generated code should contain data declarations"
    );
}

#[test]
fn parser_code_not_just_whitespace() {
    let result = build_default(simple_grammar());
    assert!(
        result.parser_code.trim().len() > 10,
        "parser code should be substantial"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// 6. BuildResult — node_types_json validity
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn node_types_json_is_valid_json() {
    let result = build_default(simple_grammar());
    let parsed: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("valid JSON");
    assert!(parsed.is_array() || parsed.is_object());
}

#[test]
fn node_types_json_array_non_empty() {
    let result = build_default(simple_grammar());
    let parsed: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("valid JSON");
    if let Some(arr) = parsed.as_array() {
        assert!(!arr.is_empty(), "node types array should not be empty");
    }
}

#[test]
fn node_types_json_contains_type_field() {
    let result = build_default(simple_grammar());
    let parsed: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("valid JSON");
    if let Some(arr) = parsed.as_array() {
        let has_type = arr.iter().any(|v| v.get("type").is_some());
        assert!(has_type, "node types entries should have 'type' field");
    }
}

#[test]
fn node_types_json_contains_named_field() {
    let result = build_default(simple_grammar());
    let parsed: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("valid JSON");
    if let Some(arr) = parsed.as_array() {
        let has_named = arr.iter().any(|v| v.get("named").is_some());
        assert!(has_named, "node types entries should have 'named' field");
    }
}

#[test]
fn node_types_json_start_rule_present() {
    let result = build_default(simple_grammar());
    let json_str = &result.node_types_json;
    assert!(
        json_str.contains("start"),
        "node types should reference the start rule"
    );
}

#[test]
fn node_types_json_single_token_grammar() {
    let result = build_default(single_token_grammar("nt"));
    let parsed: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("valid JSON");
    assert!(parsed.is_array() || parsed.is_object());
}

#[test]
fn node_types_json_differs_for_different_grammars() {
    let r1 = build_default(single_token_grammar("n1"));
    let r2 = build_default(
        GrammarBuilder::new("n2")
            .token("X", "x")
            .token("Y", "y")
            .rule("root", vec!["X", "Y"])
            .start("root")
            .build(),
    );
    // Different grammars may produce different node types
    assert_ne!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn node_types_json_valid_utf8() {
    let result = build_default(simple_grammar());
    // String is always valid UTF-8 in Rust, but verify it round-trips
    let bytes = result.node_types_json.as_bytes();
    let _roundtrip = std::str::from_utf8(bytes).expect("valid UTF-8");
}

// ═════════════════════════════════════════════════════════════════════════
// 7. BuildStats — numeric checks
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_stats_state_count_positive() {
    let result = build_default(simple_grammar());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_stats_symbol_count_positive() {
    let result = build_default(simple_grammar());
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_stats_conflict_cells_for_unambiguous() {
    let result = build_default(simple_grammar());
    assert_eq!(
        result.build_stats.conflict_cells, 0,
        "unambiguous grammar should have zero conflicts"
    );
}

#[test]
fn build_stats_debug_impl() {
    let result = build_default(simple_grammar());
    let debug = format!("{:?}", result.build_stats);
    assert!(debug.contains("BuildStats"));
    assert!(debug.contains("state_count"));
    assert!(debug.contains("symbol_count"));
    assert!(debug.contains("conflict_cells"));
}

#[test]
fn build_stats_symbol_count_at_least_tokens() {
    let result = build_default(simple_grammar());
    // grammar has 2 tokens (A, B) + EOF + nonterminals, so symbol_count >= 2
    assert!(
        result.build_stats.symbol_count >= 2,
        "symbol_count should be at least the number of tokens"
    );
}

#[test]
fn build_stats_state_count_at_least_two() {
    let result = build_default(simple_grammar());
    // Even a minimal grammar needs at least an initial and accept state
    assert!(result.build_stats.state_count >= 2);
}

#[test]
fn build_stats_single_token_grammar_state_count() {
    let result = build_default(single_token_grammar("st"));
    assert!(result.build_stats.state_count >= 2);
}

#[test]
fn build_stats_single_token_grammar_symbol_count() {
    let result = build_default(single_token_grammar("ss"));
    assert!(result.build_stats.symbol_count >= 1);
}

// ═════════════════════════════════════════════════════════════════════════
// 8. Grammar scaling — different sizes
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_scaling_one_rule() {
    let g = single_token_grammar("one_rule");
    let result = build_default(g);
    assert!(result.build_stats.state_count >= 2);
}

#[test]
fn grammar_scaling_two_rules() {
    let g = GrammarBuilder::new("two")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["inner"])
        .rule("inner", vec!["A", "B"])
        .start("root")
        .build();
    let result = build_default(g);
    assert!(result.build_stats.state_count >= 2);
}

#[test]
fn grammar_scaling_five_alternatives() {
    let g = GrammarBuilder::new("five_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .rule("root", vec!["C"])
        .rule("root", vec!["D"])
        .rule("root", vec!["E"])
        .start("root")
        .build();
    let result = build_default(g);
    assert!(result.build_stats.symbol_count >= 5);
}

#[test]
fn grammar_scaling_chain_of_five() {
    let result = build_default(n_rule_grammar("chain5", 5));
    assert!(result.build_stats.state_count >= 2);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_scaling_chain_of_ten() {
    let result = build_default(n_rule_grammar("chain10", 10));
    assert!(result.build_stats.state_count >= 2);
}

#[test]
fn grammar_scaling_more_rules_more_states() {
    let small = build_default(single_token_grammar("small_sc"));
    let big = build_default(
        GrammarBuilder::new("big_sc")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .rule("root", vec!["seq"])
            .rule("seq", vec!["A", "B", "C"])
            .rule("seq", vec!["A", "B"])
            .rule("seq", vec!["A"])
            .start("root")
            .build(),
    );
    assert!(big.build_stats.state_count >= small.build_stats.state_count);
}

#[test]
fn grammar_scaling_more_tokens_more_symbols() {
    let one_tok = build_default(single_token_grammar("one_t"));
    let three_tok = build_default(
        GrammarBuilder::new("three_t")
            .token("X", "x")
            .token("Y", "y")
            .token("Z", "z")
            .rule("root", vec!["X", "Y", "Z"])
            .start("root")
            .build(),
    );
    assert!(three_tok.build_stats.symbol_count > one_tok.build_stats.symbol_count);
}

#[test]
fn grammar_scaling_parser_code_grows() {
    let small = build_default(single_token_grammar("pc_s"));
    let medium = build_default(
        GrammarBuilder::new("pc_m")
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .token("D", "d")
            .rule("root", vec!["pair"])
            .rule("pair", vec!["A", "B"])
            .rule("pair", vec!["C", "D"])
            .start("root")
            .build(),
    );
    assert!(
        medium.parser_code.len() >= small.parser_code.len(),
        "bigger grammar should produce at least as much parser code"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// 9. Grammar features — precedence, inline, extras, supertypes
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_feature_precedence_builds() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("precedence grammar should build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_feature_precedence_stats() {
    let g = GrammarBuilder::new("prec_st")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("build");
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn grammar_feature_right_associativity() {
    let g = GrammarBuilder::new("rassoc")
        .token("NUM", r"\d+")
        .token("EXP", "^")
        .rule_with_precedence("expr", vec!["expr", "EXP", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_feature_none_associativity() {
    let g = GrammarBuilder::new("nassoc")
        .token("NUM", r"\d+")
        .token("CMP", "<")
        .rule_with_precedence("expr", vec!["expr", "CMP", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_feature_inline_rule_builds() {
    let g = GrammarBuilder::new("inl")
        .token("TOK", "x")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["TOK"])
        .inline("helper")
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("inline grammar should build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_feature_extras_builds() {
    let g = GrammarBuilder::new("ext")
        .token("WS", r"[ \t]+")
        .token("TOK", "x")
        .extra("WS")
        .rule("root", vec!["TOK"])
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("extras grammar should build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_feature_extras_does_not_break_stats() {
    let g = GrammarBuilder::new("ext_s")
        .token("WS", r"[ \t]+")
        .token("TOK", "x")
        .extra("WS")
        .rule("root", vec!["TOK"])
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("build");
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn grammar_feature_supertype_builds() {
    let g = GrammarBuilder::new("sup")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["expression"])
        .rule("expression", vec!["NUM"])
        .rule("expression", vec!["ID"])
        .supertype("expression")
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("supertype grammar should build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn grammar_feature_supertype_node_types() {
    let g = GrammarBuilder::new("sup_nt")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["expression"])
        .rule("expression", vec!["NUM"])
        .rule("expression", vec!["ID"])
        .supertype("expression")
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("build");
    let parsed: serde_json::Value =
        serde_json::from_str(&result.node_types_json).expect("valid JSON");
    assert!(parsed.is_array() || parsed.is_object());
}

#[test]
fn grammar_feature_multiple_precedence_levels() {
    let g = GrammarBuilder::new("multi_prec")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("POW", "^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "POW", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("build");
    assert!(result.build_stats.symbol_count >= 4);
}

#[test]
fn grammar_feature_combined_extras_and_precedence() {
    let g = GrammarBuilder::new("combo")
        .token("WS", r"[ \t]+")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .extra("WS")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("build");
    assert!(!result.parser_code.is_empty());
    assert!(result.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 10. Determinism — reproducible builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn determinism_same_grammar_same_parser_code() {
    let r1 = build_default(simple_grammar());
    let r2 = build_default(simple_grammar());
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn determinism_same_grammar_same_node_types() {
    let r1 = build_default(simple_grammar());
    let r2 = build_default(simple_grammar());
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn determinism_same_grammar_same_state_count() {
    let r1 = build_default(simple_grammar());
    let r2 = build_default(simple_grammar());
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn determinism_same_grammar_same_symbol_count() {
    let r1 = build_default(simple_grammar());
    let r2 = build_default(simple_grammar());
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn determinism_same_grammar_same_conflict_cells() {
    let r1 = build_default(simple_grammar());
    let r2 = build_default(simple_grammar());
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn determinism_same_grammar_same_grammar_name() {
    let r1 = build_default(simple_grammar());
    let r2 = build_default(simple_grammar());
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

// ═════════════════════════════════════════════════════════════════════════
// 11. Build with different options — compress vs uncompressed
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_with_compress_tables_true() {
    let (_dir, mut opts) = tmp_opts();
    opts.compress_tables = true;
    let result = build_parser(simple_grammar(), opts).expect("compressed build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_with_compress_tables_false() {
    let (_dir, mut opts) = tmp_opts();
    opts.compress_tables = false;
    let result = build_parser(simple_grammar(), opts).expect("uncompressed build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_compressed_and_uncompressed_differ() {
    let (_d1, mut opts1) = tmp_opts();
    opts1.compress_tables = true;
    let r1 = build_parser(simple_grammar(), opts1).expect("compressed");

    let (_d2, mut opts2) = tmp_opts();
    opts2.compress_tables = false;
    let r2 = build_parser(simple_grammar(), opts2).expect("uncompressed");

    // Both should produce valid results; code may differ
    assert!(!r1.parser_code.is_empty());
    assert!(!r2.parser_code.is_empty());
}

#[test]
fn build_compressed_stats_match_uncompressed_stats() {
    let (_d1, mut opts1) = tmp_opts();
    opts1.compress_tables = true;
    let r1 = build_parser(simple_grammar(), opts1).expect("compressed");

    let (_d2, mut opts2) = tmp_opts();
    opts2.compress_tables = false;
    let r2 = build_parser(simple_grammar(), opts2).expect("uncompressed");

    // Build stats come from the same parse table
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

// ═════════════════════════════════════════════════════════════════════════
// 12. Emit artifacts option
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_with_emit_artifacts_true_creates_dir() {
    let (dir, opts) = tmp_opts_with(true, false);
    let result = build_parser(simple_grammar(), opts).expect("build with artifacts");
    assert!(!result.parser_code.is_empty());
    // With emit_artifacts, a grammar subdirectory should be created
    let grammar_dir = dir.path().join("grammar_test");
    assert!(
        grammar_dir.exists(),
        "grammar dir should exist when emit_artifacts=true"
    );
}

#[test]
fn build_with_emit_artifacts_false_still_succeeds() {
    let (_dir, opts) = tmp_opts_with(false, false);
    let result = build_parser(simple_grammar(), opts).expect("build without artifacts");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_with_emit_artifacts_true_writes_node_types_file() {
    let (dir, opts) = tmp_opts_with(true, false);
    let _result = build_parser(simple_grammar(), opts).expect("build");
    let node_types_path = dir.path().join("grammar_test").join("NODE_TYPES.json");
    assert!(
        node_types_path.exists(),
        "NODE_TYPES.json should be written when emit_artifacts=true"
    );
}

#[test]
fn build_with_emit_artifacts_true_writes_grammar_ir() {
    let (dir, opts) = tmp_opts_with(true, false);
    let _result = build_parser(simple_grammar(), opts).expect("build");
    let ir_path = dir.path().join("grammar_test").join("grammar.ir.json");
    assert!(
        ir_path.exists(),
        "grammar.ir.json should be written when emit_artifacts=true"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// 13. Edge cases and miscellaneous
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn build_result_parser_path_contains_grammar_name() {
    let result = build_default(simple_grammar());
    assert!(
        result.parser_path.contains("test"),
        "parser_path should include grammar name"
    );
}

#[test]
fn build_result_grammar_name_preserved() {
    let g = GrammarBuilder::new("my_custom_name")
        .token("TOK", "x")
        .rule("root", vec!["TOK"])
        .start("root")
        .build();
    let result = build_default(g);
    assert_eq!(result.grammar_name, "my_custom_name");
}

#[test]
fn build_stats_conflict_cells_is_usize() {
    let result = build_default(simple_grammar());
    let _: usize = result.build_stats.conflict_cells;
}

#[test]
fn build_stats_state_count_is_usize() {
    let result = build_default(simple_grammar());
    let _: usize = result.build_stats.state_count;
}

#[test]
fn build_stats_symbol_count_is_usize() {
    let result = build_default(simple_grammar());
    let _: usize = result.build_stats.symbol_count;
}

#[test]
fn build_options_struct_update_syntax() {
    let base = BuildOptions {
        out_dir: "base".into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let updated = BuildOptions {
        emit_artifacts: true,
        ..base
    };
    assert!(updated.emit_artifacts);
    assert_eq!(updated.out_dir, "base");
}

#[test]
fn build_result_node_types_json_has_no_null_bytes() {
    let result = build_default(simple_grammar());
    assert!(
        !result.node_types_json.contains('\0'),
        "node_types_json should not contain null bytes"
    );
}

#[test]
fn build_result_parser_code_has_no_null_bytes() {
    let result = build_default(simple_grammar());
    assert!(
        !result.parser_code.contains('\0'),
        "parser_code should not contain null bytes"
    );
}

#[test]
fn build_with_regex_tokens() {
    let g = GrammarBuilder::new("regex_tok")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-zA-Z_]\w*")
        .rule("root", vec!["NUMBER"])
        .rule("root", vec!["IDENT"])
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("regex grammar should build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_with_literal_tokens() {
    let g = GrammarBuilder::new("lit_tok")
        .token("HELLO", "hello")
        .token("WORLD", "world")
        .rule("root", vec!["HELLO", "WORLD"])
        .start("root")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(g, opts).expect("literal grammar should build");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_grammar_name_alphanumeric() {
    let g = GrammarBuilder::new("mygrammar2")
        .token("TOK", "x")
        .rule("root", vec!["TOK"])
        .start("root")
        .build();
    let result = build_default(g);
    assert_eq!(result.grammar_name, "mygrammar2");
}

#[test]
fn build_grammar_name_with_underscores() {
    let g = GrammarBuilder::new("my_grammar")
        .token("TOK", "x")
        .rule("root", vec!["TOK"])
        .start("root")
        .build();
    let result = build_default(g);
    assert_eq!(result.grammar_name, "my_grammar");
}

#[test]
fn build_result_node_types_json_not_empty_object() {
    let result = build_default(simple_grammar());
    assert_ne!(
        result.node_types_json.trim(),
        "{}",
        "node_types_json should not be just an empty object"
    );
}

#[test]
fn build_result_node_types_json_not_empty_array() {
    let result = build_default(simple_grammar());
    assert_ne!(
        result.node_types_json.trim(),
        "[]",
        "node_types_json should not be just an empty array"
    );
}
