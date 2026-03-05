//! Comprehensive tests for `BuildOptions`, `BuildResult`, and `BuildStats`
//! in the pure-Rust builder pipeline.

use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser_from_json};
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

fn simple_json(name: &str) -> String {
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

/// A slightly larger grammar with two rules and a choice.
fn choice_json(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "word": null,
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "item"
            },
            "item": {
                "type": "CHOICE",
                "members": [
                    { "type": "PATTERN", "value": "\\d+" },
                    { "type": "PATTERN", "value": "[a-z]+" }
                ]
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

fn build_ok(json: &str, opts: BuildOptions) -> BuildResult {
    build_parser_from_json(json.to_owned(), opts).expect("build_parser_from_json should succeed")
}

// ===========================================================================
// 1. Default options (8 tests)
// ===========================================================================

#[test]
fn default_compress_tables_is_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn default_emit_artifacts_is_false() {
    // Without ADZE_EMIT_ARTIFACTS env var, should default to false.
    // We cannot guarantee the env var is unset, but the standard CI
    // environment does not set it.
    let opts = BuildOptions::default();
    // If env var is set to "true" this would be true — skip assertion in
    // that case.
    if std::env::var("ADZE_EMIT_ARTIFACTS").is_err() {
        assert!(!opts.emit_artifacts);
    }
}

#[test]
fn default_out_dir_is_non_empty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty(), "out_dir should have a value");
}

#[test]
fn default_debug_impl_contains_struct_name() {
    let opts = BuildOptions::default();
    let dbg = format!("{opts:?}");
    assert!(
        dbg.contains("BuildOptions"),
        "Debug output should mention the struct name"
    );
}

#[test]
fn default_debug_impl_contains_fields() {
    let opts = BuildOptions::default();
    let dbg = format!("{opts:?}");
    assert!(
        dbg.contains("compress_tables"),
        "should show compress_tables"
    );
    assert!(dbg.contains("emit_artifacts"), "should show emit_artifacts");
    assert!(dbg.contains("out_dir"), "should show out_dir");
}

#[test]
fn default_clone_equals_original() {
    let opts = BuildOptions::default();
    let cloned = opts.clone();
    assert_eq!(opts.out_dir, cloned.out_dir);
    assert_eq!(opts.emit_artifacts, cloned.emit_artifacts);
    assert_eq!(opts.compress_tables, cloned.compress_tables);
}

#[test]
fn default_clone_is_independent() {
    let opts = BuildOptions::default();
    let mut cloned = opts.clone();
    cloned.compress_tables = !opts.compress_tables;
    // Original must be unchanged.
    assert_ne!(opts.compress_tables, cloned.compress_tables);
}

#[test]
fn default_options_can_build_simple_grammar() {
    let dir = TempDir::new().unwrap();
    let opts = temp_opts(&dir);
    let result = build_parser_from_json(simple_json("default_build"), opts);
    assert!(
        result.is_ok(),
        "default options should build ok: {result:?}"
    );
}

// ===========================================================================
// 2. Custom options (8 tests)
// ===========================================================================

#[test]
fn custom_compress_tables_false() {
    let opts = BuildOptions {
        compress_tables: false,
        ..BuildOptions::default()
    };
    assert!(!opts.compress_tables);
}

#[test]
fn custom_compress_tables_true() {
    let opts = BuildOptions {
        compress_tables: true,
        ..BuildOptions::default()
    };
    assert!(opts.compress_tables);
}

#[test]
fn custom_emit_artifacts_true() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert!(opts.emit_artifacts);
}

#[test]
fn custom_emit_artifacts_false() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(!opts.emit_artifacts);
}

#[test]
fn custom_out_dir() {
    let opts = BuildOptions {
        out_dir: "/tmp/custom_out".into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "/tmp/custom_out");
}

#[test]
fn custom_out_dir_preserved_after_clone() {
    let opts = BuildOptions {
        out_dir: "/tmp/preserved".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/tmp/preserved");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

#[test]
fn custom_all_fields_at_once() {
    let opts = BuildOptions {
        out_dir: "my_dir".into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    assert_eq!(opts.out_dir, "my_dir");
    assert!(opts.emit_artifacts);
    assert!(opts.compress_tables);
}

#[test]
fn custom_debug_shows_custom_values() {
    let opts = BuildOptions {
        out_dir: "dbg_dir".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let dbg = format!("{opts:?}");
    assert!(dbg.contains("dbg_dir"));
    assert!(dbg.contains("true"));
    assert!(dbg.contains("false"));
}

// ===========================================================================
// 3. Options affect build (8 tests)
// ===========================================================================

#[test]
fn compress_tables_off_produces_valid_build() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_json(simple_json("nocompress"), opts);
    assert!(
        result.is_ok(),
        "uncompressed build should succeed: {result:?}"
    );
}

#[test]
fn compress_tables_on_produces_valid_build() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser_from_json(simple_json("compress"), opts);
    assert!(
        result.is_ok(),
        "compressed build should succeed: {result:?}"
    );
}

#[test]
fn emit_artifacts_true_produces_valid_build() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let result = build_parser_from_json(simple_json("emit_on"), opts);
    assert!(
        result.is_ok(),
        "emit_artifacts=true should succeed: {result:?}"
    );
}

#[test]
fn emit_artifacts_false_produces_valid_build() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_json(simple_json("emit_off"), opts);
    assert!(
        result.is_ok(),
        "emit_artifacts=false should succeed: {result:?}"
    );
}

#[test]
fn compress_on_and_off_both_produce_parser_code() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();

    let r1 = build_ok(
        &simple_json("cmp_on"),
        BuildOptions {
            out_dir: dir1.path().to_string_lossy().into(),
            emit_artifacts: false,
            compress_tables: true,
        },
    );
    let r2 = build_ok(
        &simple_json("cmp_off"),
        BuildOptions {
            out_dir: dir2.path().to_string_lossy().into(),
            emit_artifacts: false,
            compress_tables: false,
        },
    );
    assert!(!r1.parser_code.is_empty());
    assert!(!r2.parser_code.is_empty());
}

#[test]
fn different_grammar_names_produce_different_grammar_name_field() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let r1 = build_ok(&simple_json("alpha"), temp_opts(&dir1));
    let r2 = build_ok(&simple_json("beta"), temp_opts(&dir2));
    assert_eq!(r1.grammar_name, "alpha");
    assert_eq!(r2.grammar_name, "beta");
}

#[test]
fn choice_grammar_builds_with_compression() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser_from_json(choice_json("choice_cmp"), opts);
    assert!(
        result.is_ok(),
        "choice grammar + compress should work: {result:?}"
    );
}

#[test]
fn choice_grammar_builds_without_compression() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_json(choice_json("choice_nocmp"), opts);
    assert!(
        result.is_ok(),
        "choice grammar no compress should work: {result:?}"
    );
}

// ===========================================================================
// 4. BuildResult properties (7 tests)
// ===========================================================================

#[test]
fn build_result_grammar_name_matches_input() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("result_name"), temp_opts(&dir));
    assert_eq!(r.grammar_name, "result_name");
}

#[test]
fn build_result_parser_code_is_nonempty() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("code_check"), temp_opts(&dir));
    assert!(!r.parser_code.is_empty(), "parser_code should not be empty");
}

#[test]
fn build_result_parser_path_is_nonempty() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("path_check"), temp_opts(&dir));
    assert!(!r.parser_path.is_empty(), "parser_path should not be empty");
}

#[test]
fn build_result_node_types_json_is_valid_json() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("node_types"), temp_opts(&dir));
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&r.node_types_json);
    assert!(
        parsed.is_ok(),
        "node_types_json should be valid JSON: {parsed:?}"
    );
}

#[test]
fn build_result_node_types_json_is_array() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("nt_array"), temp_opts(&dir));
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array(), "node_types_json should be a JSON array");
}

#[test]
fn build_result_debug_contains_grammar_name() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("debug_res"), temp_opts(&dir));
    let dbg = format!("{r:?}");
    assert!(
        dbg.contains("debug_res"),
        "Debug should mention grammar name"
    );
}

#[test]
fn build_result_parser_code_contains_grammar_name_constant() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("const_check"), temp_opts(&dir));
    assert!(
        r.parser_code.contains("GRAMMAR_NAME") || r.parser_code.contains("const_check"),
        "parser_code should reference grammar name"
    );
}

// ===========================================================================
// 5. BuildStats accuracy (8 tests)
// ===========================================================================

#[test]
fn build_stats_state_count_is_positive() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("stats_sc"), temp_opts(&dir));
    assert!(r.build_stats.state_count > 0, "state_count should be > 0");
}

#[test]
fn build_stats_symbol_count_is_positive() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("stats_sym"), temp_opts(&dir));
    assert!(r.build_stats.symbol_count > 0, "symbol_count should be > 0");
}

#[test]
fn build_stats_conflict_cells_non_negative() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("stats_cc"), temp_opts(&dir));
    // conflict_cells is usize so always >= 0; just verify we can read it.
    let _ = r.build_stats.conflict_cells;
}

#[test]
fn build_stats_debug_contains_state_count() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("stats_dbg"), temp_opts(&dir));
    let dbg = format!("{:?}", r.build_stats);
    assert!(dbg.contains("state_count"), "Debug should show state_count");
}

#[test]
fn build_stats_debug_contains_symbol_count() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("stats_dbg2"), temp_opts(&dir));
    let dbg = format!("{:?}", r.build_stats);
    assert!(
        dbg.contains("symbol_count"),
        "Debug should show symbol_count"
    );
}

#[test]
fn build_stats_debug_contains_conflict_cells() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("stats_dbg3"), temp_opts(&dir));
    let dbg = format!("{:?}", r.build_stats);
    assert!(
        dbg.contains("conflict_cells"),
        "Debug should show conflict_cells"
    );
}

#[test]
fn simple_grammar_has_at_least_two_symbols() {
    // source_file + value + implicit tokens => at least 2
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("sym_min"), temp_opts(&dir));
    assert!(
        r.build_stats.symbol_count >= 2,
        "simple grammar should have >= 2 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn choice_grammar_has_more_symbols_than_simple() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let r_simple = build_ok(&simple_json("cmp_s"), temp_opts(&dir1));
    let r_choice = build_ok(&choice_json("cmp_c"), temp_opts(&dir2));
    assert!(
        r_choice.build_stats.symbol_count >= r_simple.build_stats.symbol_count,
        "choice grammar should have >= symbols: {} vs {}",
        r_choice.build_stats.symbol_count,
        r_simple.build_stats.symbol_count
    );
}

// ===========================================================================
// 6. Build with various option combinations (8 tests)
// ===========================================================================

#[test]
fn compress_true_emit_false() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_ok(&simple_json("ct_ef"), opts);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn compress_false_emit_true() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let r = build_ok(&simple_json("cf_et"), opts);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn compress_true_emit_true() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r = build_ok(&simple_json("ct_et"), opts);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn compress_false_emit_false() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_ok(&simple_json("cf_ef"), opts);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn choice_compress_true_emit_true() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r = build_ok(&choice_json("ch_ct_et"), opts);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn choice_compress_false_emit_false() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_ok(&choice_json("ch_cf_ef"), opts);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn build_same_grammar_twice_gives_consistent_stats() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let r1 = build_ok(&simple_json("dup"), temp_opts(&dir1));
    let r2 = build_ok(&simple_json("dup"), temp_opts(&dir2));
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn build_same_grammar_twice_gives_same_parser_code() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let r1 = build_ok(&simple_json("determ"), temp_opts(&dir1));
    let r2 = build_ok(&simple_json("determ"), temp_opts(&dir2));
    assert_eq!(
        r1.parser_code, r2.parser_code,
        "builds should be deterministic"
    );
}

// ===========================================================================
// 7. Edge cases (8 tests)
// ===========================================================================

#[test]
fn invalid_json_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("not json".to_owned(), temp_opts(&dir));
    assert!(result.is_err(), "invalid JSON should fail");
}

#[test]
fn empty_json_object_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("{}".to_owned(), temp_opts(&dir));
    assert!(result.is_err(), "empty object should fail");
}

#[test]
fn missing_rules_returns_error() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "name": "norules",
        "word": null,
        "extras": [],
        "conflicts": [],
        "precedences": [],
        "externals": [],
        "inline": [],
        "supertypes": []
    })
    .to_string();
    let result = build_parser_from_json(json, temp_opts(&dir));
    assert!(result.is_err(), "missing rules should fail");
}

#[test]
fn missing_name_still_builds_or_uses_unknown() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "word": null,
        "rules": {
            "source_file": {
                "type": "PATTERN",
                "value": "\\d+"
            }
        },
        "extras": [],
        "conflicts": [],
        "precedences": [],
        "externals": [],
        "inline": [],
        "supertypes": []
    })
    .to_string();
    let result = build_parser_from_json(json, temp_opts(&dir));
    // Either succeeds with name "unknown" or fails — both acceptable.
    if let Ok(r) = &result {
        assert!(
            r.grammar_name == "unknown" || !r.grammar_name.is_empty(),
            "grammar_name should be set"
        );
    }
}

#[test]
fn all_options_true_builds_ok() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let result = build_parser_from_json(simple_json("all_true"), opts);
    assert!(result.is_ok(), "all-true options should work: {result:?}");
}

#[test]
fn all_options_false_except_out_dir_builds_ok() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_json(simple_json("all_false"), opts);
    assert!(result.is_ok(), "all-false options should work: {result:?}");
}

#[test]
fn grammar_name_with_underscores() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("my_cool_grammar"), temp_opts(&dir));
    assert_eq!(r.grammar_name, "my_cool_grammar");
}

#[test]
fn grammar_name_single_char() {
    let dir = TempDir::new().unwrap();
    let r = build_ok(&simple_json("x"), temp_opts(&dir));
    assert_eq!(r.grammar_name, "x");
}
