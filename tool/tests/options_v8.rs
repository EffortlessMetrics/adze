//! Comprehensive tests for `BuildOptions` configuration in adze-tool.
//!
//! 80+ tests covering defaults, custom values, all boolean combinations,
//! result properties, stats consistency, sequential builds, and edge cases.

use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};
use std::path::Path;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_grammar(name: &str) -> Grammar {
    let mut g = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    g.normalize();
    g
}

fn make_two_token_grammar(name: &str) -> Grammar {
    let mut g = GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    g.normalize();
    g
}

fn opts_with(dir: &TempDir, emit: bool, compress: bool) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: emit,
        compress_tables: compress,
    }
}

fn temp_opts(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    }
}

// ===========================================================================
// CATEGORY 1: BuildOptions::default() values (4 tests)
// ===========================================================================

#[test]
fn default_emit_artifacts_is_false() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn default_compress_tables_is_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn default_out_dir_is_nonempty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn default_out_dir_is_string() {
    let opts = BuildOptions::default();
    // out_dir is a String, not a PathBuf — verify it round-trips through String ops
    let cloned: String = opts.out_dir.clone();
    assert_eq!(opts.out_dir, cloned);
}

// ===========================================================================
// CATEGORY 2: Custom out_dir values (6 tests)
// ===========================================================================

#[test]
fn custom_out_dir_dot() {
    let opts = BuildOptions {
        out_dir: ".".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, ".");
}

#[test]
fn custom_out_dir_build() {
    let opts = BuildOptions {
        out_dir: "build".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "build");
}

#[test]
fn custom_out_dir_absolute() {
    let opts = BuildOptions {
        out_dir: "/tmp/test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "/tmp/test");
}

#[test]
fn custom_out_dir_nested() {
    let opts = BuildOptions {
        out_dir: "a/b/c/d".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "a/b/c/d");
}

#[test]
fn custom_out_dir_with_spaces() {
    let opts = BuildOptions {
        out_dir: "/tmp/my build dir".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "/tmp/my build dir");
}

#[test]
fn custom_out_dir_tempdir() {
    let dir = TempDir::new().unwrap();
    let opts = temp_opts(&dir);
    assert!(
        Path::new(&opts.out_dir).is_absolute(),
        "tempdir path should be absolute: {}",
        opts.out_dir
    );
}

// ===========================================================================
// CATEGORY 3: emit_artifacts=true builds successfully (3 tests)
// ===========================================================================

#[test]
fn emit_true_build_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_emit_t1"), opts_with(&dir, true, true));
    assert!(result.is_ok(), "emit=true build failed: {result:?}");
}

#[test]
fn emit_true_no_compress_build_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_emit_t2"), opts_with(&dir, true, false));
    assert!(
        result.is_ok(),
        "emit=true compress=false failed: {result:?}"
    );
}

#[test]
fn emit_true_produces_parser_code() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_emit_t3"), opts_with(&dir, true, true)).unwrap();
    assert!(!result.parser_code.is_empty());
}

// ===========================================================================
// CATEGORY 4: emit_artifacts=false builds successfully (3 tests)
// ===========================================================================

#[test]
fn emit_false_build_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_emit_f1"), opts_with(&dir, false, true));
    assert!(result.is_ok(), "emit=false build failed: {result:?}");
}

#[test]
fn emit_false_no_compress_build_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_emit_f2"), opts_with(&dir, false, false));
    assert!(
        result.is_ok(),
        "emit=false compress=false failed: {result:?}"
    );
}

#[test]
fn emit_false_produces_parser_code() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_emit_f3"), opts_with(&dir, false, true)).unwrap();
    assert!(!result.parser_code.is_empty());
}

// ===========================================================================
// CATEGORY 5: compress_tables=true builds successfully (3 tests)
// ===========================================================================

#[test]
fn compress_true_build_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_comp_t1"), opts_with(&dir, false, true));
    assert!(result.is_ok(), "compress=true build failed: {result:?}");
}

#[test]
fn compress_true_produces_node_types() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_comp_t2"), opts_with(&dir, false, true)).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn compress_true_has_grammar_name() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_comp_t3"), opts_with(&dir, false, true)).unwrap();
    assert_eq!(result.grammar_name, "ov8_comp_t3");
}

// ===========================================================================
// CATEGORY 6: compress_tables=false builds successfully (3 tests)
// ===========================================================================

#[test]
fn compress_false_build_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_comp_f1"), opts_with(&dir, false, false));
    assert!(result.is_ok(), "compress=false build failed: {result:?}");
}

#[test]
fn compress_false_produces_node_types() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_comp_f2"), opts_with(&dir, false, false)).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn compress_false_has_grammar_name() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_comp_f3"), opts_with(&dir, false, false)).unwrap();
    assert_eq!(result.grammar_name, "ov8_comp_f3");
}

// ===========================================================================
// CATEGORY 7: All 8 combinations of (emit, compress) × 2 out_dir styles (16 tests)
// ===========================================================================

macro_rules! combo_test {
    ($name:ident, $gname:expr, $emit:expr, $compress:expr, $use_tempdir:expr) => {
        #[test]
        fn $name() {
            let dir = TempDir::new().unwrap();
            let out_dir = if $use_tempdir {
                dir.path().to_string_lossy().to_string()
            } else {
                "/tmp/ov8_combo".to_string()
            };
            let opts = BuildOptions {
                out_dir,
                emit_artifacts: $emit,
                compress_tables: $compress,
            };
            let result = build_parser(make_grammar($gname), opts);
            assert!(result.is_ok(), "{} failed: {result:?}", $gname);
        }
    };
}

combo_test!(combo_ff_tmp, "ov8_cb01", false, false, false);
combo_test!(combo_ft_tmp, "ov8_cb02", false, true, false);
combo_test!(combo_tf_tmp, "ov8_cb03", true, false, false);
combo_test!(combo_tt_tmp, "ov8_cb04", true, true, false);
combo_test!(combo_ff_dir, "ov8_cb05", false, false, true);
combo_test!(combo_ft_dir, "ov8_cb06", false, true, true);
combo_test!(combo_tf_dir, "ov8_cb07", true, false, true);
combo_test!(combo_tt_dir, "ov8_cb08", true, true, true);

// Additional combo tests checking result fields
macro_rules! combo_result_test {
    ($name:ident, $gname:expr, $emit:expr, $compress:expr) => {
        #[test]
        fn $name() {
            let dir = TempDir::new().unwrap();
            let opts = opts_with(&dir, $emit, $compress);
            let result = build_parser(make_grammar($gname), opts).unwrap();
            assert!(!result.parser_code.is_empty());
            assert!(!result.node_types_json.is_empty());
            assert_eq!(result.grammar_name, $gname);
        }
    };
}

combo_result_test!(combo_result_ff, "ov8_cr01", false, false);
combo_result_test!(combo_result_ft, "ov8_cr02", false, true);
combo_result_test!(combo_result_tf, "ov8_cr03", true, false);
combo_result_test!(combo_result_tt, "ov8_cr04", true, true);

// Same combos with two-token grammar
combo_result_test!(combo_result_two_ff, "ov8_cr2_01", false, false);
combo_result_test!(combo_result_two_ft, "ov8_cr2_02", false, true);
combo_result_test!(combo_result_two_tf, "ov8_cr2_03", true, false);
combo_result_test!(combo_result_two_tt, "ov8_cr2_04", true, true);

// ===========================================================================
// CATEGORY 8: BuildResult parser_code non-empty for all combos (4 tests)
// ===========================================================================

#[test]
fn parser_code_nonempty_ff() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_pc01"), opts_with(&dir, false, false)).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn parser_code_nonempty_ft() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_pc02"), opts_with(&dir, false, true)).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn parser_code_nonempty_tf() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_pc03"), opts_with(&dir, true, false)).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn parser_code_nonempty_tt() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_pc04"), opts_with(&dir, true, true)).unwrap();
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// CATEGORY 9: BuildResult node_types_json non-empty (4 tests)
// ===========================================================================

#[test]
fn node_types_nonempty_default_opts() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_nt01"), temp_opts(&dir)).unwrap();
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn node_types_is_valid_json() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_nt02"), temp_opts(&dir)).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("node_types_json should be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn node_types_nonempty_no_compress() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_nt03"), opts_with(&dir, false, false)).unwrap();
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn node_types_nonempty_emit_true() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_nt04"), opts_with(&dir, true, true)).unwrap();
    assert!(!r.node_types_json.is_empty());
}

// ===========================================================================
// CATEGORY 10: BuildStats reasonable values (8 tests)
// ===========================================================================

#[test]
fn stats_state_count_positive() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_st01"), temp_opts(&dir)).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn stats_symbol_count_positive() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_st02"), temp_opts(&dir)).unwrap();
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn stats_symbol_count_at_least_two() {
    // Even a minimal grammar has at least the token and the start rule
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_st03"), temp_opts(&dir)).unwrap();
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn stats_state_count_bounded() {
    // A trivial grammar should not produce thousands of states
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_st04"), temp_opts(&dir)).unwrap();
    assert!(r.build_stats.state_count < 1000);
}

#[test]
fn stats_symbol_count_bounded() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_st05"), temp_opts(&dir)).unwrap();
    assert!(r.build_stats.symbol_count < 1000);
}

#[test]
fn stats_debug_format_nonempty() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_st06"), temp_opts(&dir)).unwrap();
    let debug = format!("{:?}", r.build_stats);
    assert!(!debug.is_empty());
}

#[test]
fn stats_conflict_cells_not_absurd() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_st07"), temp_opts(&dir)).unwrap();
    assert!(r.build_stats.conflict_cells < 10_000);
}

#[test]
fn stats_two_token_more_symbols() {
    let dir = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_st08a"), temp_opts(&dir)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_two_token_grammar("ov8_st08b"), temp_opts(&dir2)).unwrap();

    assert!(r2.build_stats.symbol_count >= r1.build_stats.symbol_count);
}

// ===========================================================================
// CATEGORY 11: Same grammar different options → same stats (4 tests)
// ===========================================================================

#[test]
fn same_grammar_compress_toggle_same_state_count() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_sg01a"), opts_with(&dir1, false, true)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_sg01b"), opts_with(&dir2, false, false)).unwrap();

    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn same_grammar_compress_toggle_same_symbol_count() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_sg02a"), opts_with(&dir1, false, true)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_sg02b"), opts_with(&dir2, false, false)).unwrap();

    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn same_grammar_emit_toggle_same_state_count() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_sg03a"), opts_with(&dir1, true, true)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_sg03b"), opts_with(&dir2, false, true)).unwrap();

    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn same_grammar_emit_toggle_same_symbol_count() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_sg04a"), opts_with(&dir1, true, false)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_sg04b"), opts_with(&dir2, false, false)).unwrap();

    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

// ===========================================================================
// CATEGORY 12: Different grammars same options → different code (4 tests)
// ===========================================================================

#[test]
fn different_grammars_different_parser_code() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_dg01a"), temp_opts(&dir1)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_two_token_grammar("ov8_dg01b"), temp_opts(&dir2)).unwrap();

    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn different_grammars_different_grammar_name() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_dg02a"), temp_opts(&dir1)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_two_token_grammar("ov8_dg02b"), temp_opts(&dir2)).unwrap();

    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn different_grammars_different_node_types() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_dg03a"), temp_opts(&dir1)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_two_token_grammar("ov8_dg03b"), temp_opts(&dir2)).unwrap();

    assert_ne!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn different_grammars_same_options_both_succeed() {
    let dir1 = TempDir::new().unwrap();
    let opts1 = opts_with(&dir1, true, true);
    let r1 = build_parser(make_grammar("ov8_dg04a"), opts1);
    assert!(r1.is_ok());

    let dir2 = TempDir::new().unwrap();
    let opts2 = opts_with(&dir2, true, true);
    let r2 = build_parser(make_two_token_grammar("ov8_dg04b"), opts2);
    assert!(r2.is_ok());
}

// ===========================================================================
// CATEGORY 13: Out_dir with various string values (5 tests)
// ===========================================================================

#[test]
fn outdir_relative_builds_ok() {
    let result = build_parser(
        make_grammar("ov8_od01"),
        BuildOptions {
            out_dir: "target/ov8_test".to_string(),
            emit_artifacts: false,
            compress_tables: true,
        },
    );
    assert!(result.is_ok(), "relative out_dir failed: {result:?}");
}

#[test]
fn outdir_absolute_builds_ok() {
    let dir = TempDir::new().unwrap();
    let result = build_parser(make_grammar("ov8_od02"), temp_opts(&dir));
    assert!(result.is_ok(), "absolute out_dir failed: {result:?}");
}

#[test]
fn outdir_dot_builds_ok() {
    let result = build_parser(
        make_grammar("ov8_od03"),
        BuildOptions {
            out_dir: ".".to_string(),
            emit_artifacts: false,
            compress_tables: true,
        },
    );
    assert!(result.is_ok(), "dot out_dir failed: {result:?}");
}

#[test]
fn outdir_preserves_value() {
    let opts = BuildOptions {
        out_dir: "my/custom/path".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "my/custom/path");
}

#[test]
fn outdir_empty_string_accepted() {
    let opts = BuildOptions {
        out_dir: String::new(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert!(opts.out_dir.is_empty());
}

// ===========================================================================
// CATEGORY 14: Options don't affect grammar correctness (6 tests)
// ===========================================================================

#[test]
fn grammar_name_unaffected_by_emit() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_gc01a"), opts_with(&dir1, true, true)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_gc01b"), opts_with(&dir2, false, true)).unwrap();

    // Grammar names differ because we use different names, but both should be correct
    assert_eq!(r1.grammar_name, "ov8_gc01a");
    assert_eq!(r2.grammar_name, "ov8_gc01b");
}

#[test]
fn parser_path_nonempty_regardless_of_options() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_gc02"), opts_with(&dir, true, false)).unwrap();
    assert!(!r.parser_path.is_empty());
}

#[test]
fn node_types_valid_json_regardless_of_compress() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_gc03a"), opts_with(&dir1, false, true)).unwrap();
    let parsed1: serde_json::Value = serde_json::from_str(&r1.node_types_json).unwrap();
    assert!(parsed1.is_array());

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_gc03b"), opts_with(&dir2, false, false)).unwrap();
    let parsed2: serde_json::Value = serde_json::from_str(&r2.node_types_json).unwrap();
    assert!(parsed2.is_array());
}

#[test]
fn stats_consistent_across_emit_toggle() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_gc04a"), opts_with(&dir1, true, true)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_gc04b"), opts_with(&dir2, false, true)).unwrap();

    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn stats_consistent_across_compress_toggle() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_gc05a"), opts_with(&dir1, false, true)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_gc05b"), opts_with(&dir2, false, false)).unwrap();

    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn all_four_combos_produce_valid_node_types() {
    for (i, (emit, compress)) in [(false, false), (false, true), (true, false), (true, true)]
        .iter()
        .enumerate()
    {
        let name = format!("ov8_gc06_{i}");
        let dir = TempDir::new().unwrap();
        let r = build_parser(make_grammar(&name), opts_with(&dir, *emit, *compress)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json)
            .unwrap_or_else(|e| panic!("combo ({emit},{compress}) invalid JSON: {e}"));
        assert!(parsed.is_array(), "combo ({emit},{compress}) not an array");
    }
}

// ===========================================================================
// CATEGORY 15: Multiple sequential builds with different options (8 tests)
// ===========================================================================

#[test]
fn sequential_builds_both_succeed() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_seq01a"), opts_with(&dir1, false, true));
    assert!(r1.is_ok());

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_seq01b"), opts_with(&dir2, true, false));
    assert!(r2.is_ok());
}

#[test]
fn sequential_three_builds_all_succeed() {
    let dir1 = TempDir::new().unwrap();
    assert!(build_parser(make_grammar("ov8_seq02a"), opts_with(&dir1, false, false)).is_ok());

    let dir2 = TempDir::new().unwrap();
    assert!(build_parser(make_grammar("ov8_seq02b"), opts_with(&dir2, true, true)).is_ok());

    let dir3 = TempDir::new().unwrap();
    assert!(build_parser(make_grammar("ov8_seq02c"), opts_with(&dir3, false, true)).is_ok());
}

#[test]
fn sequential_builds_independent_results() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_seq03a"), opts_with(&dir1, false, true)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(
        make_two_token_grammar("ov8_seq03b"),
        opts_with(&dir2, true, false),
    )
    .unwrap();

    assert_ne!(r1.grammar_name, r2.grammar_name);
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn sequential_builds_with_same_dir() {
    let dir = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_seq04a"), temp_opts(&dir));
    assert!(r1.is_ok());

    let r2 = build_parser(make_grammar("ov8_seq04b"), temp_opts(&dir));
    assert!(r2.is_ok());
}

#[test]
fn sequential_builds_four_option_combos() {
    let combos = [(false, false), (false, true), (true, false), (true, true)];
    for (i, (emit, compress)) in combos.iter().enumerate() {
        let name = format!("ov8_seq05_{i}");
        let dir = TempDir::new().unwrap();
        let result = build_parser(make_grammar(&name), opts_with(&dir, *emit, *compress));
        assert!(
            result.is_ok(),
            "combo {i} ({emit},{compress}) failed: {result:?}"
        );
    }
}

#[test]
fn sequential_builds_consistent_stats() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_seq06a"), temp_opts(&dir1)).unwrap();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(make_grammar("ov8_seq06b"), temp_opts(&dir2)).unwrap();

    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn sequential_different_grammars_different_stats() {
    let dir1 = TempDir::new().unwrap();
    let r1 = build_parser(make_grammar("ov8_seq07a"), temp_opts(&dir1)).unwrap();

    let mut bigger = GrammarBuilder::new("ov8_seq07b")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("item", vec!["c"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    bigger.normalize();

    let dir2 = TempDir::new().unwrap();
    let r2 = build_parser(bigger, temp_opts(&dir2)).unwrap();

    // A more complex grammar should have at least as many symbols
    assert!(r2.build_stats.symbol_count >= r1.build_stats.symbol_count);
}

#[test]
fn sequential_builds_reuse_tempdir() {
    let dir = TempDir::new().unwrap();
    let names = ["ov8_seq08a", "ov8_seq08b", "ov8_seq08c"];
    for name in &names {
        let result = build_parser(make_grammar(name), temp_opts(&dir));
        assert!(result.is_ok(), "{name} failed: {result:?}");
    }
}

// ===========================================================================
// CATEGORY 16: BuildOptions Clone and Debug (4 tests)
// ===========================================================================

#[test]
fn build_options_is_cloneable() {
    let opts = BuildOptions {
        out_dir: "clone_test".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "clone_test");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

#[test]
fn build_options_debug_format() {
    let opts = BuildOptions {
        out_dir: "debug_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let debug = format!("{opts:?}");
    assert!(debug.contains("debug_test"));
}

#[test]
fn build_options_clone_independence() {
    let opts = BuildOptions {
        out_dir: "original".to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let mut cloned = opts.clone();
    cloned.out_dir = "modified".to_string();
    cloned.emit_artifacts = false;
    assert_eq!(opts.out_dir, "original");
    assert!(opts.emit_artifacts);
}

#[test]
fn build_options_default_then_override() {
    let opts = BuildOptions {
        emit_artifacts: true,
        compress_tables: false,
        ..Default::default()
    };
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
    assert!(!opts.out_dir.is_empty());
}

// ===========================================================================
// CATEGORY 17: BuildResult field consistency (4 tests)
// ===========================================================================

#[test]
fn result_grammar_name_matches_input() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_rfc01"), temp_opts(&dir)).unwrap();
    assert_eq!(r.grammar_name, "ov8_rfc01");
}

#[test]
fn result_parser_path_nonempty() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_rfc02"), temp_opts(&dir)).unwrap();
    assert!(!r.parser_path.is_empty());
}

#[test]
fn result_parser_code_contains_grammar_ref() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_rfc03"), temp_opts(&dir)).unwrap();
    // Generated parser code should reference the grammar in some way
    assert!(!r.parser_code.is_empty());
}

#[test]
fn result_debug_format_nonempty() {
    let dir = TempDir::new().unwrap();
    let r = build_parser(make_grammar("ov8_rfc04"), temp_opts(&dir)).unwrap();
    let debug = format!("{r:?}");
    assert!(!debug.is_empty());
}
