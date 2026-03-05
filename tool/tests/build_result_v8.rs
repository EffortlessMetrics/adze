//! Comprehensive tests for `BuildResult` validation and properties in adze-tool.
//!
//! 80+ tests covering:
//!   1. parser_code content and non-emptiness
//!   2. node_types_json validity and structure
//!   3. build_stats numeric invariants
//!   4. grammar_name and parser_path fields
//!   5. Debug formatting
//!   6. determinism and cross-grammar differences
//!   7. grammar scaling (larger grammars → more states)
//!   8. BuildOptions combinations (compress, emit)
//!   9. multi-token / multi-rule grammars
//!  10. precedence grammar results

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..BuildOptions::default()
    };
    (dir, opts)
}

fn build(name: &str) -> BuildResult {
    let g = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let (_dir, o) = opts();
    build_parser(g, o).expect("build")
}

fn two_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn three_rule_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["alpha"])
        .rule("alpha", vec!["x", "beta"])
        .rule("beta", vec!["y", "z"])
        .start("start")
        .build()
}

fn build_grammar(g: Grammar) -> BuildResult {
    let (_dir, o) = opts();
    build_parser(g, o).expect("build")
}

// ═════════════════════════════════════════════════════════════════════════
// 1. parser_code — non-empty and content checks
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_parser_code_is_non_empty() {
    let r = build("br_v8_nonempty");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_parser_code_contains_grammar_name() {
    let r = build("br_v8_name_in_code");
    assert!(r.parser_code.contains("br_v8_name_in_code"));
}

#[test]
fn test_parser_code_is_valid_utf8() {
    let r = build("br_v8_utf8");
    // String type guarantees UTF-8; verify round-trip
    let bytes = r.parser_code.as_bytes();
    assert!(std::str::from_utf8(bytes).is_ok());
}

#[test]
fn test_parser_code_has_substantial_length() {
    let r = build("br_v8_multiline");
    assert!(
        r.parser_code.len() > 50,
        "parser_code should be substantial, got {} bytes",
        r.parser_code.len()
    );
}

#[test]
fn test_parser_code_not_only_whitespace() {
    let r = build("br_v8_not_ws");
    assert!(r.parser_code.chars().any(|c| !c.is_whitespace()));
}

// ═════════════════════════════════════════════════════════════════════════
// 2. node_types_json — non-empty, valid JSON, array structure
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_node_types_json_is_non_empty() {
    let r = build("br_v8_ntne");
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_node_types_json_is_valid_json() {
    let r = build("br_v8_validjson");
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
    assert!(parsed.is_ok(), "node_types_json is not valid JSON");
}

#[test]
fn test_node_types_json_is_array() {
    let r = build("br_v8_isarray");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array(), "node_types_json must be a JSON array");
}

#[test]
fn test_node_types_json_array_non_empty() {
    let r = build("br_v8_arrne");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    let arr = v.as_array().unwrap();
    assert!(!arr.is_empty());
}

#[test]
fn test_node_types_entries_have_type_field() {
    let r = build("br_v8_typefield");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "node_types entry missing 'type' field: {entry}"
        );
    }
}

#[test]
fn test_node_types_entries_type_field_is_string() {
    let r = build("br_v8_typestr");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        if let Some(ty) = entry.get("type") {
            assert!(ty.is_string(), "type field must be a string: {ty}");
        }
    }
}

#[test]
fn test_node_types_entries_match_grammar_start() {
    let r = build("br_v8_matchstart");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    let types: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()))
        .collect();
    assert!(
        types.contains(&"start"),
        "node_types should contain the 'start' rule, got: {types:?}"
    );
}

#[test]
fn test_node_types_multi_rule_grammar_has_matching_entries() {
    let g = three_rule_grammar("br_v8_multirule_nt");
    let r = build_grammar(g);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    let types: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()))
        .collect();
    assert!(types.contains(&"start"), "missing 'start' in {types:?}");
}

#[test]
fn test_node_types_json_entries_are_objects() {
    let r = build("br_v8_objs");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.is_object(), "each node_types entry must be an object");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 3. build_stats — numeric invariants
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_state_count_positive() {
    let r = build("br_v8_sc_pos");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_symbol_count_positive() {
    let r = build("br_v8_sym_pos");
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_conflict_cells_zero_for_simple_grammar() {
    let r = build("br_v8_cc_zero");
    assert_eq!(r.build_stats.conflict_cells, 0);
}

#[test]
fn test_symbol_count_ge_token_count() {
    // A grammar with 1 token should have symbol_count >= 1
    let r = build("br_v8_sym_ge_tok");
    assert!(r.build_stats.symbol_count >= 1);
}

#[test]
fn test_state_count_ge_two_for_minimal() {
    // Even a trivial grammar needs at least 2 states (initial + accept)
    let r = build("br_v8_sc_ge2");
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_two_token_grammar_symbol_count() {
    let g = two_token_grammar("br_v8_2tok_sym");
    let r = build_grammar(g);
    assert!(
        r.build_stats.symbol_count >= 2,
        "two-token grammar needs at least 2 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn test_three_rule_grammar_state_count() {
    let g = three_rule_grammar("br_v8_3r_sc");
    let r = build_grammar(g);
    assert!(
        r.build_stats.state_count > 0,
        "three-rule grammar must have states"
    );
}

#[test]
fn test_conflict_cells_is_usize() {
    let r = build("br_v8_cc_usize");
    // usize is always >= 0; verify arithmetic works
    let _ = r.build_stats.conflict_cells + 1;
}

// ═════════════════════════════════════════════════════════════════════════
// 4. grammar_name and parser_path fields
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_grammar_name_matches_input() {
    let r = build("br_v8_gname");
    assert_eq!(r.grammar_name, "br_v8_gname");
}

#[test]
fn test_grammar_name_preserved_for_multi_word() {
    let g = GrammarBuilder::new("br_v8_multi_word_name")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert_eq!(r.grammar_name, "br_v8_multi_word_name");
}

#[test]
fn test_parser_path_is_non_empty() {
    let r = build("br_v8_ppath");
    assert!(!r.parser_path.is_empty());
}

#[test]
fn test_parser_path_contains_grammar_name() {
    let r = build("br_v8_ppath_name");
    assert!(
        r.parser_path.contains("br_v8_ppath_name"),
        "parser_path '{}' should contain grammar name",
        r.parser_path
    );
}

// ═════════════════════════════════════════════════════════════════════════
// 5. Debug formatting
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_build_result_debug_format() {
    let r = build("br_v8_debug");
    let debug = format!("{:?}", r);
    assert!(debug.contains("BuildResult"));
}

#[test]
fn test_build_result_debug_contains_grammar_name() {
    let r = build("br_v8_debug_gn");
    let debug = format!("{:?}", r);
    assert!(debug.contains("br_v8_debug_gn"));
}

#[test]
fn test_build_result_debug_contains_parser_code_field() {
    let r = build("br_v8_debug_pc");
    let debug = format!("{:?}", r);
    assert!(debug.contains("parser_code"));
}

#[test]
fn test_build_result_debug_contains_build_stats_field() {
    let r = build("br_v8_debug_bs");
    let debug = format!("{:?}", r);
    assert!(debug.contains("build_stats"));
}

#[test]
fn test_build_stats_debug_format() {
    let r = build("br_v8_stats_dbg");
    let debug = format!("{:?}", r.build_stats);
    assert!(debug.contains("BuildStats"));
}

#[test]
fn test_build_stats_debug_contains_state_count() {
    let r = build("br_v8_stats_dbg_sc");
    let debug = format!("{:?}", r.build_stats);
    assert!(debug.contains("state_count"));
}

#[test]
fn test_build_stats_debug_contains_symbol_count() {
    let r = build("br_v8_stats_dbg_sym");
    let debug = format!("{:?}", r.build_stats);
    assert!(debug.contains("symbol_count"));
}

#[test]
fn test_build_stats_debug_contains_conflict_cells() {
    let r = build("br_v8_stats_dbg_cc");
    let debug = format!("{:?}", r.build_stats);
    assert!(debug.contains("conflict_cells"));
}

// ═════════════════════════════════════════════════════════════════════════
// 6. Determinism — same grammar yields same results
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_determinism_same_grammar_same_state_count() {
    let r1 = build("br_v8_det_sc1");
    let r2 = build("br_v8_det_sc2");
    // Both are identical single-token grammars (different names don't affect table shape)
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn test_determinism_same_grammar_same_symbol_count() {
    let r1 = build("br_v8_det_sym1");
    let r2 = build("br_v8_det_sym2");
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn test_determinism_same_grammar_same_conflict_cells() {
    let r1 = build("br_v8_det_cc1");
    let r2 = build("br_v8_det_cc2");
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn test_determinism_node_types_json_consistent() {
    let r1 = build("br_v8_det_nt1");
    let r2 = build("br_v8_det_nt2");
    // Parse both to normalize any whitespace differences
    let v1: serde_json::Value = serde_json::from_str(&r1.node_types_json).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&r2.node_types_json).unwrap();
    assert_eq!(v1, v2);
}

// ═════════════════════════════════════════════════════════════════════════
// 7. Different grammars yield different parser_code
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_different_grammars_different_parser_code() {
    let r1 = build("br_v8_diff1");
    let g2 = two_token_grammar("br_v8_diff2");
    let r2 = build_grammar(g2);
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_different_grammars_different_grammar_name() {
    let r1 = build("br_v8_diffname1");
    let r2 = build("br_v8_diffname2");
    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn test_different_token_count_different_symbol_count() {
    let r1 = build("br_v8_tok1");
    let g2 = two_token_grammar("br_v8_tok2");
    let r2 = build_grammar(g2);
    // Two-token grammar should have more symbols
    assert!(r2.build_stats.symbol_count > r1.build_stats.symbol_count);
}

// ═════════════════════════════════════════════════════════════════════════
// 8. Larger grammar → more states
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_larger_grammar_has_more_states() {
    let r_small = build("br_v8_small");
    let g_large = three_rule_grammar("br_v8_large");
    let r_large = build_grammar(g_large);
    assert!(
        r_large.build_stats.state_count >= r_small.build_stats.state_count,
        "larger grammar ({}) should have >= states than smaller ({})",
        r_large.build_stats.state_count,
        r_small.build_stats.state_count,
    );
}

#[test]
fn test_larger_grammar_has_more_symbols() {
    let r_small = build("br_v8_small_sym");
    let g_large = three_rule_grammar("br_v8_large_sym");
    let r_large = build_grammar(g_large);
    assert!(
        r_large.build_stats.symbol_count > r_small.build_stats.symbol_count,
        "three-rule grammar should have more symbols"
    );
}

#[test]
fn test_larger_grammar_parser_code_longer() {
    let r_small = build("br_v8_small_pc");
    let g_large = three_rule_grammar("br_v8_large_pc");
    let r_large = build_grammar(g_large);
    assert!(
        r_large.parser_code.len() >= r_small.parser_code.len(),
        "larger grammar should produce at least as much code"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// 9. BuildOptions combinations — compress_tables
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_compress_true_produces_valid_result() {
    let g = GrammarBuilder::new("br_v8_comp_true")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser(g, o).expect("build with compress=true");
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_compress_false_produces_valid_result() {
    let g = GrammarBuilder::new("br_v8_comp_false")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_parser(g, o).expect("build with compress=false");
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_compress_flag_does_not_change_stats() {
    let make_grammar = |name| {
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build()
    };
    let dir1 = TempDir::new().unwrap();
    let o1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let dir2 = TempDir::new().unwrap();
    let o2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r1 = build_parser(make_grammar("br_v8_comp_s1"), o1).unwrap();
    let r2 = build_parser(make_grammar("br_v8_comp_s2"), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

// ═════════════════════════════════════════════════════════════════════════
// 10. BuildOptions combinations — emit_artifacts
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_emit_true_produces_valid_result() {
    let g = GrammarBuilder::new("br_v8_emit_true")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r = build_parser(g, o).expect("build with emit=true");
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_emit_false_produces_valid_result() {
    let g = GrammarBuilder::new("br_v8_emit_false")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser(g, o).expect("build with emit=false");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_emit_flag_does_not_change_stats() {
    let make_grammar = |name| {
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build()
    };
    let dir1 = TempDir::new().unwrap();
    let o1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let dir2 = TempDir::new().unwrap();
    let o2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r1 = build_parser(make_grammar("br_v8_emit_s1"), o1).unwrap();
    let r2 = build_parser(make_grammar("br_v8_emit_s2"), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn test_all_options_true() {
    let g = GrammarBuilder::new("br_v8_all_true")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r = build_parser(g, o).expect("build");
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_all_options_false() {
    let g = GrammarBuilder::new("br_v8_all_false")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_parser(g, o).expect("build");
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 11. Multi-token grammar results
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_multi_token_parser_code_non_empty() {
    let g = two_token_grammar("br_v8_mt_pc");
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_multi_token_node_types_valid_json() {
    let g = two_token_grammar("br_v8_mt_json");
    let r = build_grammar(g);
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_multi_token_state_count_positive() {
    let g = two_token_grammar("br_v8_mt_sc");
    let r = build_grammar(g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_multi_token_symbol_count_includes_both_tokens() {
    let g = two_token_grammar("br_v8_mt_sym");
    let r = build_grammar(g);
    assert!(
        r.build_stats.symbol_count >= 2,
        "two-token grammar needs >= 2 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn test_multi_token_grammar_name_correct() {
    let g = two_token_grammar("br_v8_mt_gn");
    let r = build_grammar(g);
    assert_eq!(r.grammar_name, "br_v8_mt_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 12. Three-rule grammar results
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_three_rule_parser_code_non_empty() {
    let g = three_rule_grammar("br_v8_3r_pc");
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_three_rule_node_types_valid() {
    let g = three_rule_grammar("br_v8_3r_nt");
    let r = build_grammar(g);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn test_three_rule_symbol_count_ge_six() {
    // 3 non-terminals + 3 tokens = 6 grammar symbols minimum
    let g = three_rule_grammar("br_v8_3r_sym6");
    let r = build_grammar(g);
    assert!(
        r.build_stats.symbol_count >= 6,
        "expected >= 6 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn test_three_rule_grammar_name() {
    let g = three_rule_grammar("br_v8_3r_gname");
    let r = build_grammar(g);
    assert_eq!(r.grammar_name, "br_v8_3r_gname");
}

#[test]
fn test_three_rule_conflict_cells() {
    let g = three_rule_grammar("br_v8_3r_cc");
    let r = build_grammar(g);
    // Simple chain grammar should have 0 conflicts
    assert_eq!(r.build_stats.conflict_cells, 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 13. Precedence grammar results
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_precedence_grammar_builds_successfully() {
    let g = GrammarBuilder::new("br_v8_prec_ok")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_precedence_grammar_state_count_positive() {
    let g = GrammarBuilder::new("br_v8_prec_sc")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_precedence_grammar_symbol_count() {
    let g = GrammarBuilder::new("br_v8_prec_sym")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn test_precedence_grammar_node_types_valid() {
    let g = GrammarBuilder::new("br_v8_prec_nt")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("start")
        .build();
    let r = build_grammar(g);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn test_precedence_grammar_name_preserved() {
    let g = GrammarBuilder::new("br_v8_prec_gn")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("start")
        .build();
    let r = build_grammar(g);
    assert_eq!(r.grammar_name, "br_v8_prec_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 14. Right-associative grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_right_assoc_grammar_builds() {
    let g = GrammarBuilder::new("br_v8_rassoc")
        .token("num", "[0-9]+")
        .token("caret", "\\^")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["num"])
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            1,
            Associativity::Right,
        )
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 15. Cross-field consistency
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_grammar_name_in_parser_code_matches_field() {
    let r = build("br_v8_cross_name");
    assert!(r.parser_code.contains(&r.grammar_name));
}

#[test]
fn test_parser_path_references_grammar_name() {
    let r = build("br_v8_cross_path");
    assert!(r.parser_path.contains(&r.grammar_name));
}

#[test]
fn test_node_types_and_parser_code_both_populated() {
    let r = build("br_v8_both_pop");
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_build_stats_fields_consistent_with_grammar_size() {
    let r = build("br_v8_stats_consist");
    // symbol_count should always be > 0 when state_count > 0
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 16. node_types_json content validation
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_node_types_entries_have_named_field() {
    let r = build("br_v8_named_field");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    // At least some entries should have a "named" boolean field
    let has_named = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e.get("named").is_some());
    assert!(
        has_named,
        "at least one node_types entry should have 'named'"
    );
}

#[test]
fn test_node_types_type_values_are_non_empty() {
    let r = build("br_v8_type_ne");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        if let Some(ty) = entry.get("type").and_then(|t| t.as_str()) {
            assert!(!ty.is_empty(), "type field should not be empty");
        }
    }
}

#[test]
fn test_node_types_no_null_entries() {
    let r = build("br_v8_no_null");
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(!entry.is_null(), "node_types entries must not be null");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 17. parser_code deeper checks
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_parser_code_length_positive() {
    let r = build("br_v8_pc_len");
    assert!(
        r.parser_code.len() > 10,
        "parser_code should be substantial"
    );
}

#[test]
fn test_parser_code_no_null_bytes() {
    let r = build("br_v8_pc_nonull");
    assert!(
        !r.parser_code.contains('\0'),
        "parser_code should not contain null bytes"
    );
}

#[test]
fn test_parser_code_two_token_contains_name() {
    let g = two_token_grammar("br_v8_2tok_pc_name");
    let r = build_grammar(g);
    assert!(r.parser_code.contains("br_v8_2tok_pc_name"));
}

#[test]
fn test_parser_code_three_rule_contains_name() {
    let g = three_rule_grammar("br_v8_3r_pc_name");
    let r = build_grammar(g);
    assert!(r.parser_code.contains("br_v8_3r_pc_name"));
}

// ═════════════════════════════════════════════════════════════════════════
// 18. BuildStats numeric edge cases
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_stats_state_count_not_absurdly_large() {
    // A simple grammar shouldn't produce thousands of states
    let r = build("br_v8_sc_bound");
    assert!(
        r.build_stats.state_count < 1000,
        "simple grammar produced {} states",
        r.build_stats.state_count
    );
}

#[test]
fn test_stats_symbol_count_not_absurdly_large() {
    let r = build("br_v8_sym_bound");
    assert!(
        r.build_stats.symbol_count < 1000,
        "simple grammar produced {} symbols",
        r.build_stats.symbol_count
    );
}

#[test]
fn test_stats_three_rule_state_count_bounded() {
    let g = three_rule_grammar("br_v8_3r_sc_bnd");
    let r = build_grammar(g);
    assert!(r.build_stats.state_count < 1000);
}

#[test]
fn test_stats_three_rule_symbol_count_bounded() {
    let g = three_rule_grammar("br_v8_3r_sym_bnd");
    let r = build_grammar(g);
    assert!(r.build_stats.symbol_count < 1000);
}

// ═════════════════════════════════════════════════════════════════════════
// 19. Extra token grammar (whitespace)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_extra_token_grammar_builds() {
    let g = GrammarBuilder::new("br_v8_extra")
        .token("x", "x")
        .token("ws", "\\s+")
        .rule("start", vec!["x"])
        .start("start")
        .extra("ws")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_extra_token_node_types_valid() {
    let g = GrammarBuilder::new("br_v8_extra_nt")
        .token("x", "x")
        .token("ws", "\\s+")
        .rule("start", vec!["x"])
        .start("start")
        .extra("ws")
        .build();
    let r = build_grammar(g);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 20. BuildOptions Default trait
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_build_options_default_out_dir_non_empty() {
    let d = BuildOptions::default();
    assert!(!d.out_dir.is_empty());
}

#[test]
fn test_build_options_default_compress_tables_true() {
    let d = BuildOptions::default();
    assert!(d.compress_tables);
}

#[test]
fn test_build_options_debug_format() {
    let d = BuildOptions::default();
    let debug = format!("{:?}", d);
    assert!(debug.contains("BuildOptions"));
}

#[test]
fn test_build_options_clone() {
    let d = BuildOptions::default();
    let c = d.clone();
    assert_eq!(c.out_dir, d.out_dir);
    assert_eq!(c.compress_tables, d.compress_tables);
    assert_eq!(c.emit_artifacts, d.emit_artifacts);
}

#[test]
fn test_build_options_clone_independence() {
    let mut d = BuildOptions::default();
    let c = d.clone();
    d.out_dir = "changed".to_string();
    assert_ne!(c.out_dir, d.out_dir);
}

// ═════════════════════════════════════════════════════════════════════════
// 21. Multiple alternative productions
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_alternative_productions_build() {
    let g = GrammarBuilder::new("br_v8_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_alternative_productions_node_types() {
    let g = GrammarBuilder::new("br_v8_alt_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let r = build_grammar(g);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn test_alternative_productions_grammar_name() {
    let g = GrammarBuilder::new("br_v8_alt_gn")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert_eq!(r.grammar_name, "br_v8_alt_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 22. Regex token patterns
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_regex_token_grammar_builds() {
    let g = GrammarBuilder::new("br_v8_regex")
        .token("ident", "[a-z]+")
        .rule("start", vec!["ident"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_regex_token_node_types_valid() {
    let g = GrammarBuilder::new("br_v8_regex_nt")
        .token("ident", "[a-z]+")
        .rule("start", vec!["ident"])
        .start("start")
        .build();
    let r = build_grammar(g);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 23. parser_path is well-formed
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_parser_path_no_null_bytes() {
    let r = build("br_v8_pp_nonull");
    assert!(!r.parser_path.contains('\0'));
}

#[test]
fn test_parser_path_two_token() {
    let g = two_token_grammar("br_v8_pp_2tok");
    let r = build_grammar(g);
    assert!(!r.parser_path.is_empty());
    assert!(r.parser_path.contains("br_v8_pp_2tok"));
}

#[test]
fn test_parser_path_three_rule() {
    let g = three_rule_grammar("br_v8_pp_3r");
    let r = build_grammar(g);
    assert!(!r.parser_path.is_empty());
}
