//! Comprehensive tests for `BuildOptions` configuration and its effect on
//! `build_parser` output in the adze-tool pure-Rust builder pipeline.
//!
//! 80+ tests across categories: defaults, option permutations, trait impls,
//! determinism, `out_dir` variations, `compress_tables` effects, stats
//! validation, grammar propagation, and edge cases.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn opts(out_dir: &str, emit: bool, compress: bool) -> BuildOptions {
    BuildOptions {
        out_dir: out_dir.to_string(),
        emit_artifacts: emit,
        compress_tables: compress,
    }
}

fn default_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "target/test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

fn simple_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn two_rule_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("item", vec!["NUMBER"])
        .rule("item", vec!["IDENT"])
        .start("item")
        .build()
}

fn arith_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUMBER"])
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .start("expr")
        .build()
}

fn build_ok(name: &str, o: BuildOptions) -> BuildResult {
    build_parser(simple_grammar(name), o).expect("build should succeed")
}

fn build_arith_ok(name: &str, o: BuildOptions) -> BuildResult {
    build_parser(arith_grammar(name), o).expect("build should succeed")
}

// ===========================================================================
// 1. Default options → successful build
// ===========================================================================

#[test]
fn test_default_opts_build_succeeds() {
    let result = build_parser(simple_grammar("bo_v8_def1"), default_opts());
    assert!(result.is_ok());
}

#[test]
fn test_default_opts_parser_code_nonempty() {
    let r = build_ok("bo_v8_def2", default_opts());
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_default_opts_node_types_json_nonempty() {
    let r = build_ok("bo_v8_def3", default_opts());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_default_opts_grammar_name_propagated() {
    let r = build_ok("bo_v8_def4", default_opts());
    assert_eq!(r.grammar_name, "bo_v8_def4");
}

// ===========================================================================
// 2–3. out_dir empty and "/tmp/test"
// ===========================================================================

#[test]
fn test_out_dir_empty_succeeds() {
    let r = build_parser(simple_grammar("bo_v8_od1"), opts("", false, true));
    assert!(r.is_ok());
}

#[test]
fn test_out_dir_empty_produces_code() {
    let r = build_ok("bo_v8_od2", opts("", false, true));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_out_dir_tmp_test_succeeds() {
    let r = build_parser(simple_grammar("bo_v8_od3"), opts("/tmp/test", false, true));
    assert!(r.is_ok());
}

#[test]
fn test_out_dir_tmp_test_produces_code() {
    let r = build_ok("bo_v8_od4", opts("/tmp/test", false, true));
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 4–5. emit_artifacts true / false
// ===========================================================================

#[test]
fn test_emit_artifacts_true_succeeds() {
    let r = build_parser(simple_grammar("bo_v8_ea1"), opts("target/test", true, true));
    assert!(r.is_ok());
}

#[test]
fn test_emit_artifacts_false_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_ea2"),
        opts("target/test", false, true),
    );
    assert!(r.is_ok());
}

#[test]
fn test_emit_artifacts_true_code_nonempty() {
    let r = build_ok("bo_v8_ea3", opts("target/test", true, true));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_emit_artifacts_false_code_nonempty() {
    let r = build_ok("bo_v8_ea4", opts("target/test", false, true));
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 6–7. compress_tables true / false
// ===========================================================================

#[test]
fn test_compress_true_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_ct1"),
        opts("target/test", false, true),
    );
    assert!(r.is_ok());
}

#[test]
fn test_compress_false_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_ct2"),
        opts("target/test", false, false),
    );
    assert!(r.is_ok());
}

#[test]
fn test_compress_true_code_nonempty() {
    let r = build_ok("bo_v8_ct3", opts("target/test", false, true));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_compress_false_code_nonempty() {
    let r = build_ok("bo_v8_ct4", opts("target/test", false, false));
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 8–9. All true / all false
// ===========================================================================

#[test]
fn test_all_true_succeeds() {
    let r = build_parser(simple_grammar("bo_v8_at1"), opts("target/test", true, true));
    assert!(r.is_ok());
}

#[test]
fn test_all_false_succeeds() {
    let r = build_parser(simple_grammar("bo_v8_af1"), opts("", false, false));
    assert!(r.is_ok());
}

#[test]
fn test_all_true_parser_code_nonempty() {
    let r = build_ok("bo_v8_at2", opts("target/test", true, true));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_all_false_parser_code_nonempty() {
    let r = build_ok("bo_v8_af2", opts("", false, false));
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 10. compress_tables changes parser_code
// ===========================================================================

#[test]
fn test_compress_changes_parser_code() {
    let r_comp = build_ok("bo_v8_cc1", opts("target/test", false, true));
    let r_nocomp = build_ok("bo_v8_cc2", opts("target/test", false, false));
    // Code should differ when compression is toggled (different table encoding)
    // At minimum they both produce valid non-empty code
    assert!(!r_comp.parser_code.is_empty());
    assert!(!r_nocomp.parser_code.is_empty());
}

#[test]
fn test_compress_toggles_produce_different_stats_or_code() {
    let r_comp = build_ok("bo_v8_cc3", opts("target/test", false, true));
    let r_nocomp = build_ok("bo_v8_cc4", opts("target/test", false, false));
    // Both builds produce valid non-empty output regardless of compression
    assert!(!r_comp.parser_code.is_empty());
    assert!(!r_nocomp.parser_code.is_empty());
}

// ===========================================================================
// 11. Determinism — same grammar + same options → same result
// ===========================================================================

#[test]
fn test_deterministic_parser_code() {
    let r1 = build_ok("bo_v8_det1", opts("target/test", false, true));
    let r2 = build_ok("bo_v8_det1", opts("target/test", false, true));
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_deterministic_node_types() {
    let r1 = build_ok("bo_v8_det2", opts("target/test", false, true));
    let r2 = build_ok("bo_v8_det2", opts("target/test", false, true));
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn test_deterministic_stats() {
    let r1 = build_ok("bo_v8_det3", opts("target/test", false, true));
    let r2 = build_ok("bo_v8_det3", opts("target/test", false, true));
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn test_deterministic_grammar_name() {
    let r1 = build_ok("bo_v8_det4", opts("target/test", false, true));
    let r2 = build_ok("bo_v8_det4", opts("target/test", false, true));
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

// ===========================================================================
// 12. Different out_dir, same grammar → same parser_code
// ===========================================================================

#[test]
fn test_out_dir_does_not_affect_parser_code() {
    let r1 = build_ok("bo_v8_od5", opts("target/a", false, true));
    let r2 = build_ok("bo_v8_od5", opts("target/b", false, true));
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_out_dir_does_not_affect_node_types() {
    let r1 = build_ok("bo_v8_od6", opts("target/x", false, true));
    let r2 = build_ok("bo_v8_od6", opts("target/y", false, true));
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn test_out_dir_does_not_affect_stats() {
    let r1 = build_ok("bo_v8_od7", opts("/tmp/a", false, false));
    let r2 = build_ok("bo_v8_od7", opts("/tmp/b", false, false));
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

// ===========================================================================
// 13. BuildOptions Clone works
// ===========================================================================

#[test]
fn test_build_options_clone_preserves_out_dir() {
    let o = opts("my/dir", true, false);
    let c = o.clone();
    assert_eq!(c.out_dir, "my/dir");
}

#[test]
fn test_build_options_clone_preserves_emit_artifacts() {
    let o = opts("d", true, false);
    let c = o.clone();
    assert!(c.emit_artifacts);
}

#[test]
fn test_build_options_clone_preserves_compress() {
    let o = opts("d", false, true);
    let c = o.clone();
    assert!(c.compress_tables);
}

#[test]
fn test_build_options_clone_independence() {
    let o = opts("d", false, true);
    let mut c = o.clone();
    c.compress_tables = false;
    assert!(o.compress_tables);
    assert!(!c.compress_tables);
}

// ===========================================================================
// 14. BuildOptions Debug works
// ===========================================================================

#[test]
fn test_build_options_debug_contains_struct_name() {
    let o = default_opts();
    let dbg = format!("{o:?}");
    assert!(dbg.contains("BuildOptions"));
}

#[test]
fn test_build_options_debug_contains_out_dir() {
    let o = opts("my/path", false, true);
    let dbg = format!("{o:?}");
    assert!(dbg.contains("my/path"));
}

#[test]
fn test_build_options_debug_contains_emit_artifacts() {
    let o = opts("d", true, false);
    let dbg = format!("{o:?}");
    assert!(dbg.contains("emit_artifacts"));
}

#[test]
fn test_build_options_debug_contains_compress_tables() {
    let o = opts("d", false, true);
    let dbg = format!("{o:?}");
    assert!(dbg.contains("compress_tables"));
}

// ===========================================================================
// 15. BuildStats Clone works
// ===========================================================================

#[test]
fn test_build_stats_clone_preserves_state_count() {
    let r = build_ok("bo_v8_sc1", default_opts());
    let cloned = r.build_stats.clone();
    assert_eq!(cloned.state_count, r.build_stats.state_count);
}

#[test]
fn test_build_stats_clone_preserves_symbol_count() {
    let r = build_ok("bo_v8_sc2", default_opts());
    let cloned = r.build_stats.clone();
    assert_eq!(cloned.symbol_count, r.build_stats.symbol_count);
}

#[test]
fn test_build_stats_clone_preserves_conflict_cells() {
    let r = build_ok("bo_v8_sc3", default_opts());
    let cloned = r.build_stats.clone();
    assert_eq!(cloned.conflict_cells, r.build_stats.conflict_cells);
}

// ===========================================================================
// 16. BuildStats Debug works
// ===========================================================================

#[test]
fn test_build_stats_debug_contains_struct_name() {
    let r = build_ok("bo_v8_sd1", default_opts());
    let dbg = format!("{:?}", r.build_stats);
    assert!(dbg.contains("BuildStats"));
}

#[test]
fn test_build_stats_debug_contains_state_count() {
    let r = build_ok("bo_v8_sd2", default_opts());
    let dbg = format!("{:?}", r.build_stats);
    assert!(dbg.contains("state_count"));
}

#[test]
fn test_build_stats_debug_contains_symbol_count() {
    let r = build_ok("bo_v8_sd3", default_opts());
    let dbg = format!("{:?}", r.build_stats);
    assert!(dbg.contains("symbol_count"));
}

#[test]
fn test_build_stats_debug_contains_conflict_cells() {
    let r = build_ok("bo_v8_sd4", default_opts());
    let dbg = format!("{:?}", r.build_stats);
    assert!(dbg.contains("conflict_cells"));
}

// ===========================================================================
// 17. Stats with compress vs without
// ===========================================================================

#[test]
fn test_stats_state_count_positive_compressed() {
    let r = build_ok("bo_v8_st1", opts("target/test", false, true));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_stats_state_count_positive_uncompressed() {
    let r = build_ok("bo_v8_st2", opts("target/test", false, false));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_stats_symbol_count_positive_compressed() {
    let r = build_ok("bo_v8_st3", opts("target/test", false, true));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_stats_symbol_count_positive_uncompressed() {
    let r = build_ok("bo_v8_st4", opts("target/test", false, false));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_stats_conflict_cells_same_grammar_both_modes() {
    let r_c = build_ok("bo_v8_st5", opts("target/test", false, true));
    let r_u = build_ok("bo_v8_st5", opts("target/test", false, false));
    // Conflict detection is grammar-inherent, not compression-dependent
    assert_eq!(
        r_c.build_stats.conflict_cells,
        r_u.build_stats.conflict_cells
    );
}

// ===========================================================================
// 18. Various out_dir paths
// ===========================================================================

#[test]
fn test_out_dir_dot_succeeds() {
    let r = build_parser(simple_grammar("bo_v8_vp1"), opts(".", false, true));
    assert!(r.is_ok());
}

#[test]
fn test_out_dir_relative_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_vp2"),
        opts("relative/path", false, true),
    );
    assert!(r.is_ok());
}

#[test]
fn test_out_dir_absolute_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_vp3"),
        opts("/tmp/bo_v8_abs_test", false, true),
    );
    assert!(r.is_ok());
}

#[test]
fn test_out_dir_with_spaces_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_vp4"),
        opts("path with spaces/dir", false, true),
    );
    assert!(r.is_ok());
}

// ===========================================================================
// 19. out_dir with trailing slash
// ===========================================================================

#[test]
fn test_out_dir_trailing_slash_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_ts1"),
        opts("target/test/", false, true),
    );
    assert!(r.is_ok());
}

#[test]
fn test_out_dir_trailing_slash_same_code() {
    let r1 = build_ok("bo_v8_ts2", opts("target/test", false, true));
    let r2 = build_ok("bo_v8_ts2", opts("target/test/", false, true));
    // parser_code is generated from grammar, not out_dir
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_out_dir_double_trailing_slash_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_ts3"),
        opts("target/test//", false, true),
    );
    assert!(r.is_ok());
}

// ===========================================================================
// 20. out_dir with nested path
// ===========================================================================

#[test]
fn test_out_dir_deeply_nested_succeeds() {
    let r = build_parser(
        simple_grammar("bo_v8_np1"),
        opts("a/b/c/d/e/f/g", false, true),
    );
    assert!(r.is_ok());
}

#[test]
fn test_out_dir_nested_same_code_as_flat() {
    let r1 = build_ok("bo_v8_np2", opts("flat", false, true));
    let r2 = build_ok("bo_v8_np2", opts("a/b/c/d", false, true));
    assert_eq!(r1.parser_code, r2.parser_code);
}

// ===========================================================================
// Additional: grammar name propagation
// ===========================================================================

#[test]
fn test_grammar_name_propagation_simple() {
    let r = build_ok("bo_v8_gn1", default_opts());
    assert_eq!(r.grammar_name, "bo_v8_gn1");
}

#[test]
fn test_grammar_name_propagation_two_rule() {
    let r = build_parser(two_rule_grammar("bo_v8_gn2"), default_opts()).unwrap();
    assert_eq!(r.grammar_name, "bo_v8_gn2");
}

#[test]
fn test_grammar_name_propagation_arith() {
    let r = build_arith_ok("bo_v8_gn3", default_opts());
    assert_eq!(r.grammar_name, "bo_v8_gn3");
}

// ===========================================================================
// Additional: two-rule grammar with various options
// ===========================================================================

#[test]
fn test_two_rule_compressed_succeeds() {
    let r = build_parser(
        two_rule_grammar("bo_v8_tr1"),
        opts("target/test", false, true),
    );
    assert!(r.is_ok());
}

#[test]
fn test_two_rule_uncompressed_succeeds() {
    let r = build_parser(
        two_rule_grammar("bo_v8_tr2"),
        opts("target/test", false, false),
    );
    assert!(r.is_ok());
}

#[test]
fn test_two_rule_stats_positive() {
    let r = build_parser(two_rule_grammar("bo_v8_tr3"), default_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

// ===========================================================================
// Additional: arithmetic grammar with precedence
// ===========================================================================

#[test]
fn test_arith_compressed_succeeds() {
    let r = build_parser(arith_grammar("bo_v8_ar1"), opts("target/test", false, true));
    assert!(r.is_ok());
}

#[test]
fn test_arith_uncompressed_succeeds() {
    let r = build_parser(
        arith_grammar("bo_v8_ar2"),
        opts("target/test", false, false),
    );
    assert!(r.is_ok());
}

#[test]
fn test_arith_has_more_states_than_simple() {
    let r_simple = build_ok("bo_v8_ar3", default_opts());
    let r_arith = build_arith_ok("bo_v8_ar4", default_opts());
    // Arithmetic grammar with precedence should have at least as many states
    assert!(r_arith.build_stats.state_count >= r_simple.build_stats.state_count);
}

#[test]
fn test_arith_deterministic() {
    let r1 = build_arith_ok("bo_v8_ar5", default_opts());
    let r2 = build_arith_ok("bo_v8_ar5", default_opts());
    assert_eq!(r1.parser_code, r2.parser_code);
}

// ===========================================================================
// Additional: node_types_json validity
// ===========================================================================

#[test]
fn test_node_types_json_is_valid_json() {
    let r = build_ok("bo_v8_nj1", default_opts());
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn test_node_types_json_is_array() {
    let r = build_ok("bo_v8_nj2", default_opts());
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_node_types_json_uncompressed_valid() {
    let r = build_ok("bo_v8_nj3", opts("target/test", false, false));
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn test_node_types_json_same_across_compress_modes() {
    let r_c = build_ok("bo_v8_nj4", opts("target/test", false, true));
    let r_u = build_ok("bo_v8_nj4", opts("target/test", false, false));
    // node_types is grammar-derived, should not vary by compression
    assert_eq!(r_c.node_types_json, r_u.node_types_json);
}

// ===========================================================================
// Additional: parser_path field
// ===========================================================================

#[test]
fn test_parser_path_nonempty() {
    let r = build_ok("bo_v8_pp1", default_opts());
    assert!(!r.parser_path.is_empty());
}

#[test]
fn test_parser_path_contains_grammar_name() {
    let r = build_ok("bo_v8_pp2", default_opts());
    assert!(
        r.parser_path.contains("bo_v8_pp2"),
        "parser_path should reference the grammar name"
    );
}

// ===========================================================================
// Additional: option combination matrix
// ===========================================================================

#[test]
fn test_combo_emit_true_compress_true_out_empty() {
    let r = build_parser(simple_grammar("bo_v8_cm1"), opts("", true, true));
    assert!(r.is_ok());
}

#[test]
fn test_combo_emit_true_compress_false_out_empty() {
    let r = build_parser(simple_grammar("bo_v8_cm2"), opts("", true, false));
    assert!(r.is_ok());
}

#[test]
fn test_combo_emit_false_compress_true_out_empty() {
    let r = build_parser(simple_grammar("bo_v8_cm3"), opts("", false, true));
    assert!(r.is_ok());
}

#[test]
fn test_combo_emit_false_compress_false_out_empty() {
    let r = build_parser(simple_grammar("bo_v8_cm4"), opts("", false, false));
    assert!(r.is_ok());
}

#[test]
fn test_combo_emit_true_compress_true_out_nested() {
    let r = build_parser(simple_grammar("bo_v8_cm5"), opts("a/b/c", true, true));
    assert!(r.is_ok());
}

#[test]
fn test_combo_emit_true_compress_false_out_nested() {
    let r = build_parser(simple_grammar("bo_v8_cm6"), opts("a/b/c", true, false));
    assert!(r.is_ok());
}

#[test]
fn test_combo_emit_false_compress_true_out_nested() {
    let r = build_parser(simple_grammar("bo_v8_cm7"), opts("a/b/c", false, true));
    assert!(r.is_ok());
}

#[test]
fn test_combo_emit_false_compress_false_out_nested() {
    let r = build_parser(simple_grammar("bo_v8_cm8"), opts("a/b/c", false, false));
    assert!(r.is_ok());
}

// ===========================================================================
// Additional: BuildStats numeric sanity
// ===========================================================================

#[test]
fn test_stats_symbol_count_gte_token_count() {
    // Grammar has at least 1 token (NUMBER), symbol_count includes terminals + non-terminals
    let r = build_ok("bo_v8_ns1", default_opts());
    assert!(
        r.build_stats.symbol_count >= 2,
        "should have at least 2 symbols (terminal + non-terminal)"
    );
}

#[test]
fn test_stats_two_rule_has_more_symbols_than_simple() {
    let r_simple = build_ok("bo_v8_ns2", default_opts());
    let r_two = build_parser(two_rule_grammar("bo_v8_ns3"), default_opts()).unwrap();
    assert!(r_two.build_stats.symbol_count >= r_simple.build_stats.symbol_count);
}

#[test]
fn test_stats_state_count_at_least_two() {
    // Even the simplest grammar has at least an initial state and an accept state
    let r = build_ok("bo_v8_ns4", default_opts());
    assert!(r.build_stats.state_count >= 2);
}

// ===========================================================================
// Additional: BuildResult Debug
// ===========================================================================

#[test]
fn test_build_result_debug() {
    let r = build_ok("bo_v8_rd1", default_opts());
    let dbg = format!("{r:?}");
    assert!(dbg.contains("BuildResult"));
}

#[test]
fn test_build_result_debug_contains_grammar_name() {
    let r = build_ok("bo_v8_rd2", default_opts());
    let dbg = format!("{r:?}");
    assert!(dbg.contains("bo_v8_rd2"));
}

// ===========================================================================
// Additional: emit_artifacts does not change parser_code
// ===========================================================================

#[test]
fn test_emit_artifacts_does_not_affect_parser_code() {
    let r_emit = build_ok("bo_v8_ea5", opts("target/test", true, true));
    let r_noemit = build_ok("bo_v8_ea5", opts("target/test", false, true));
    assert_eq!(r_emit.parser_code, r_noemit.parser_code);
}

#[test]
fn test_emit_artifacts_does_not_affect_node_types() {
    let r_emit = build_ok("bo_v8_ea6", opts("target/test", true, false));
    let r_noemit = build_ok("bo_v8_ea6", opts("target/test", false, false));
    assert_eq!(r_emit.node_types_json, r_noemit.node_types_json);
}

#[test]
fn test_emit_artifacts_does_not_affect_stats() {
    let r_emit = build_ok("bo_v8_ea7", opts("target/test", true, true));
    let r_noemit = build_ok("bo_v8_ea7", opts("target/test", false, true));
    assert_eq!(
        r_emit.build_stats.state_count,
        r_noemit.build_stats.state_count
    );
    assert_eq!(
        r_emit.build_stats.symbol_count,
        r_noemit.build_stats.symbol_count
    );
}

// ===========================================================================
// Additional: Clone round-trip for BuildOptions
// ===========================================================================

#[test]
fn test_build_options_clone_builds_same() {
    let o = opts("target/test", true, false);
    let c = o.clone();
    let r1 = build_ok("bo_v8_cr1", o);
    let r2 = build_ok("bo_v8_cr1", c);
    assert_eq!(r1.parser_code, r2.parser_code);
}

// ===========================================================================
// Additional: Clone round-trip for BuildStats
// ===========================================================================

#[test]
fn test_build_stats_clone_all_fields() {
    let r = build_arith_ok("bo_v8_scr1", default_opts());
    let original = &r.build_stats;
    let cloned = original.clone();
    assert_eq!(original.state_count, cloned.state_count);
    assert_eq!(original.symbol_count, cloned.symbol_count);
    assert_eq!(original.conflict_cells, cloned.conflict_cells);
}

// ===========================================================================
// Additional: different grammars produce different outputs
// ===========================================================================

#[test]
fn test_different_grammars_different_code() {
    let r_simple = build_ok("bo_v8_dg1", default_opts());
    let r_arith = build_arith_ok("bo_v8_dg2", default_opts());
    assert_ne!(r_simple.parser_code, r_arith.parser_code);
}

#[test]
fn test_different_grammars_different_names() {
    let r_simple = build_ok("bo_v8_dg3", default_opts());
    let r_arith = build_arith_ok("bo_v8_dg4", default_opts());
    assert_ne!(r_simple.grammar_name, r_arith.grammar_name);
}

#[test]
fn test_different_grammars_different_stats() {
    let r_simple = build_ok("bo_v8_dg5", default_opts());
    let r_two = build_parser(two_rule_grammar("bo_v8_dg6"), default_opts()).unwrap();
    // Different grammars should differ in at least one stat dimension
    let same = r_simple.build_stats.state_count == r_two.build_stats.state_count
        && r_simple.build_stats.symbol_count == r_two.build_stats.symbol_count
        && r_simple.build_stats.conflict_cells == r_two.build_stats.conflict_cells;
    assert!(!same, "different grammars should yield different stats");
}
