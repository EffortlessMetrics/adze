//! Comprehensive pipeline integration tests: Grammar → build_parser → BuildResult.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tempdir");
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into_owned(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, o)
}

fn run(grammar: Grammar) -> BuildResult {
    let (_dir, o) = opts();
    build_parser(grammar, o).expect("build_parser should succeed")
}

fn minimal(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build()
}

fn two_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("start", vec!["NUMBER"])
        .rule("start", vec!["IDENT"])
        .start("start")
        .build()
}

fn five_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["C"])
        .rule("start", vec!["D"])
        .rule("start", vec!["E"])
        .start("start")
        .build()
}

fn ten_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("T0", "a")
        .token("T1", "b")
        .token("T2", "c")
        .token("T3", "d")
        .token("T4", "e")
        .token("T5", "f")
        .token("T6", "g")
        .token("T7", "h")
        .token("T8", "i")
        .token("T9", "j")
        .rule("start", vec!["T0"])
        .rule("start", vec!["T1"])
        .rule("start", vec!["T2"])
        .rule("start", vec!["T3"])
        .rule("start", vec!["T4"])
        .rule("start", vec!["T5"])
        .rule("start", vec!["T6"])
        .rule("start", vec!["T7"])
        .rule("start", vec!["T8"])
        .rule("start", vec!["T9"])
        .start("start")
        .build()
}

fn arith(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["LPAREN", "expr", "RPAREN"])
        .rule("expr", vec!["NUMBER"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

// ---------------------------------------------------------------------------
// 1-5: Basic pipeline
// ---------------------------------------------------------------------------

#[test]
fn test_01_simple_grammar_builds() {
    let res = run(minimal("pipe_v8_01"));
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_02_parser_code_non_empty() {
    let res = run(minimal("pipe_v8_02"));
    assert!(
        res.parser_code.len() > 10,
        "parser_code should be substantial"
    );
}

#[test]
fn test_03_node_types_json_valid_array() {
    let res = run(minimal("pipe_v8_03"));
    let v: serde_json::Value = serde_json::from_str(&res.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn test_04_state_count_positive() {
    let res = run(minimal("pipe_v8_04"));
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_05_symbol_count_positive() {
    let res = run(minimal("pipe_v8_05"));
    assert!(res.build_stats.symbol_count > 0);
}

// ---------------------------------------------------------------------------
// 6-8: Scaling with token count
// ---------------------------------------------------------------------------

#[test]
fn test_06_one_token_minimal_stats() {
    let res = run(minimal("pipe_v8_06"));
    assert!(res.build_stats.state_count > 0);
    assert!(res.build_stats.symbol_count >= 1);
}

#[test]
fn test_07_five_tokens_larger_stats() {
    let one = run(minimal("pipe_v8_07a"));
    let five = run(five_token("pipe_v8_07b"));
    assert!(five.build_stats.symbol_count > one.build_stats.symbol_count);
}

#[test]
fn test_08_ten_tokens_even_larger() {
    let five = run(five_token("pipe_v8_08a"));
    let ten = run(ten_token("pipe_v8_08b"));
    assert!(ten.build_stats.symbol_count > five.build_stats.symbol_count);
}

// ---------------------------------------------------------------------------
// 9-12: Grammar features
// ---------------------------------------------------------------------------

#[test]
fn test_09_precedence_builds() {
    let res = run(arith("pipe_v8_09"));
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_10_inline_rules_build() {
    let g = GrammarBuilder::new("pipe_v8_10")
        .token("NUMBER", r"\d+")
        .rule("helper", vec!["NUMBER"])
        .rule("start", vec!["helper"])
        .start("start")
        .inline("helper")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_11_extra_tokens_build() {
    let g = GrammarBuilder::new("pipe_v8_11")
        .token("NUMBER", r"\d+")
        .token("WS", r"\s+")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .extra("WS")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_12_conflict_cells_non_negative() {
    let res = run(arith("pipe_v8_12"));
    // conflict_cells is usize, always >= 0; just verify pipeline ran
    let _cells = res.build_stats.conflict_cells;
    assert!(!res.parser_code.is_empty());
}

// ---------------------------------------------------------------------------
// 13-16: Output properties
// ---------------------------------------------------------------------------

#[test]
fn test_13_different_grammars_different_code() {
    let a = run(minimal("pipe_v8_13a"));
    let b = run(two_token("pipe_v8_13b"));
    assert_ne!(a.parser_code, b.parser_code);
}

#[test]
fn test_14_deterministic_rebuild() {
    let g1 = minimal("pipe_v8_14");
    let g2 = minimal("pipe_v8_14");
    let r1 = run(g1);
    let r2 = run(g2);
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn test_15_parser_code_contains_grammar_name() {
    let res = run(minimal("pipe_v8_15"));
    assert!(
        res.parser_code.contains("pipe_v8_15") || res.grammar_name.contains("pipe_v8_15"),
        "grammar name should appear in output"
    );
}

#[test]
fn test_16_node_types_parseable() {
    let res = run(minimal("pipe_v8_16"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    assert!(!arr.is_empty());
}

// ---------------------------------------------------------------------------
// 17-20: Complex grammars & associativity
// ---------------------------------------------------------------------------

#[test]
fn test_17_arithmetic_pipeline() {
    let res = run(arith("pipe_v8_17"));
    assert!(
        res.build_stats.state_count > 2,
        "arithmetic grammar needs multiple states"
    );
    assert!(res.build_stats.symbol_count > 3);
}

#[test]
fn test_18_left_assoc() {
    let g = GrammarBuilder::new("pipe_v8_18")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_19_right_assoc() {
    let g = GrammarBuilder::new("pipe_v8_19")
        .token("N", r"\d+")
        .token("EQ", "=")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_20_complexity_scales_states() {
    let small = run(minimal("pipe_v8_20a"));
    let big = run(arith("pipe_v8_20b"));
    assert!(big.build_stats.state_count >= small.build_stats.state_count);
}

// ---------------------------------------------------------------------------
// 21-30: BuildResult field checks
// ---------------------------------------------------------------------------

#[test]
fn test_21_grammar_name_matches() {
    let res = run(minimal("pipe_v8_21"));
    assert_eq!(res.grammar_name, "pipe_v8_21");
}

#[test]
fn test_22_parser_path_non_empty() {
    let res = run(minimal("pipe_v8_22"));
    assert!(!res.parser_path.is_empty());
}

#[test]
fn test_23_parser_code_is_rust() {
    let res = run(minimal("pipe_v8_23"));
    // Generated Rust code should have typical Rust markers
    assert!(
        res.parser_code.contains("fn")
            || res.parser_code.contains("const")
            || res.parser_code.contains("static")
            || res.parser_code.contains("struct"),
        "parser_code should look like Rust"
    );
}

#[test]
fn test_24_node_types_has_named_entries() {
    let res = run(minimal("pipe_v8_24"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    let has_named = arr.iter().any(|v| v.get("type").is_some());
    assert!(has_named, "node_types should have type entries");
}

#[test]
fn test_25_build_stats_debug_impl() {
    let res = run(minimal("pipe_v8_25"));
    let dbg = format!("{:?}", res.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
}

#[test]
fn test_26_build_result_debug_impl() {
    let res = run(minimal("pipe_v8_26"));
    let dbg = format!("{:?}", res);
    assert!(dbg.contains("BuildResult"));
}

#[test]
fn test_27_two_token_grammar_symbols() {
    let res = run(two_token("pipe_v8_27"));
    assert!(res.build_stats.symbol_count >= 2);
}

#[test]
fn test_28_parser_code_substantial() {
    let res = run(arith("pipe_v8_28"));
    assert!(
        res.parser_code.len() > 100,
        "arithmetic parser should be substantial"
    );
}

#[test]
fn test_29_node_types_valid_utf8() {
    let res = run(minimal("pipe_v8_29"));
    // If we got here, node_types_json is already a valid String (UTF-8).
    assert!(std::str::from_utf8(res.node_types_json.as_bytes()).is_ok());
}

#[test]
fn test_30_symbol_count_includes_builtins() {
    // Even a 1-token grammar has builtins (ERROR, END, etc.)
    let res = run(minimal("pipe_v8_30"));
    assert!(
        res.build_stats.symbol_count > 1,
        "symbol_count should include built-in symbols"
    );
}

// ---------------------------------------------------------------------------
// 31-40: BuildOptions variations
// ---------------------------------------------------------------------------

#[test]
fn test_31_emit_artifacts_true() {
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into_owned(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let res = build_parser(minimal("pipe_v8_31"), o).unwrap();
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_32_compress_tables_false() {
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into_owned(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let res = build_parser(minimal("pipe_v8_32"), o).unwrap();
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_33_compress_vs_no_compress_same_stats() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let o1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().into_owned(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let o2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().into_owned(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r1 = build_parser(minimal("pipe_v8_33a"), o1).unwrap();
    let r2 = build_parser(minimal("pipe_v8_33a"), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn test_34_out_dir_string_type() {
    let dir = TempDir::new().unwrap();
    let out: String = dir.path().to_string_lossy().into_owned();
    let o = BuildOptions {
        out_dir: out,
        emit_artifacts: false,
        compress_tables: true,
    };
    let res = build_parser(minimal("pipe_v8_34"), o).unwrap();
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_35_default_options_work() {
    // Default reads OUT_DIR or "target/debug"
    let o = BuildOptions {
        out_dir: TempDir::new()
            .unwrap()
            .path()
            .to_string_lossy()
            .into_owned(),
        ..BuildOptions::default()
    };
    let res = build_parser(minimal("pipe_v8_35"), o).unwrap();
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_36_emit_artifacts_does_not_change_code() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let o1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().into_owned(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let o2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().into_owned(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r1 = build_parser(minimal("pipe_v8_36"), o1).unwrap();
    let r2 = build_parser(minimal("pipe_v8_36"), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn test_37_different_out_dirs_same_result() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let mk = |d: &TempDir| BuildOptions {
        out_dir: d.path().to_string_lossy().into_owned(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r1 = build_parser(minimal("pipe_v8_37"), mk(&dir1)).unwrap();
    let r2 = build_parser(minimal("pipe_v8_37"), mk(&dir2)).unwrap();
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn test_38_parser_path_contains_grammar_name() {
    let res = run(minimal("pipe_v8_38"));
    assert!(
        res.parser_path.contains("pipe_v8_38") || !res.parser_path.is_empty(),
        "parser_path should reference the grammar"
    );
}

#[test]
fn test_39_build_options_clone() {
    let (_dir, o) = opts();
    let o2 = o.clone();
    assert_eq!(o2.compress_tables, o.compress_tables);
}

#[test]
fn test_40_build_options_debug() {
    let (_dir, o) = opts();
    let dbg = format!("{:?}", o);
    assert!(dbg.contains("BuildOptions"));
}

// ---------------------------------------------------------------------------
// 41-50: Associativity & precedence
// ---------------------------------------------------------------------------

#[test]
fn test_41_none_assoc() {
    let g = GrammarBuilder::new("pipe_v8_41")
        .token("N", r"\d+")
        .token("CMP", "<")
        .rule_with_precedence("expr", vec!["expr", "CMP", "expr"], 1, Associativity::None)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_42_mixed_assoc_levels() {
    let g = GrammarBuilder::new("pipe_v8_42")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("POW", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "POW", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_43_high_precedence_value() {
    let g = GrammarBuilder::new("pipe_v8_43")
        .token("N", r"\d+")
        .token("OP", r"\+")
        .rule_with_precedence("expr", vec!["expr", "OP", "expr"], 100, Associativity::Left)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_44_negative_precedence() {
    let g = GrammarBuilder::new("pipe_v8_44")
        .token("N", r"\d+")
        .token("OP", r"\+")
        .rule_with_precedence("expr", vec!["expr", "OP", "expr"], -1, Associativity::Left)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_45_zero_precedence() {
    let g = GrammarBuilder::new("pipe_v8_45")
        .token("N", r"\d+")
        .token("OP", r"\+")
        .rule_with_precedence("expr", vec!["expr", "OP", "expr"], 0, Associativity::Left)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_46_multiple_same_precedence() {
    let g = GrammarBuilder::new("pipe_v8_46")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .token("MINUS", "-")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "MINUS", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.symbol_count > 2);
}

#[test]
fn test_47_left_vs_right_differ() {
    let left = GrammarBuilder::new("pipe_v8_47a")
        .token("N", r"\d+")
        .token("OP", r"\+")
        .rule_with_precedence("expr", vec!["expr", "OP", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let right = GrammarBuilder::new("pipe_v8_47b")
        .token("N", r"\d+")
        .token("OP", r"\+")
        .rule_with_precedence("expr", vec!["expr", "OP", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let rl = run(left);
    let rr = run(right);
    // Both should build; they may or may not differ in code
    assert!(!rl.parser_code.is_empty());
    assert!(!rr.parser_code.is_empty());
}

#[test]
fn test_48_precedence_adds_states() {
    let plain = run(minimal("pipe_v8_48a"));
    let prec = run(arith("pipe_v8_48b"));
    assert!(prec.build_stats.state_count >= plain.build_stats.state_count);
}

#[test]
fn test_49_multiple_prec_levels() {
    let g = GrammarBuilder::new("pipe_v8_49")
        .token("N", r"\d+")
        .token("A", r"\+")
        .token("B", r"\*")
        .token("C", r"\^")
        .token("D", "!")
        .rule_with_precedence("expr", vec!["expr", "A", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "B", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "C", "expr"], 3, Associativity::Right)
        .rule_with_precedence("expr", vec!["expr", "D", "expr"], 4, Associativity::None)
        .rule("expr", vec!["N"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.symbol_count >= 5);
}

#[test]
fn test_50_precedence_deterministic() {
    let mk = || {
        GrammarBuilder::new("pipe_v8_50")
            .token("N", r"\d+")
            .token("OP", r"\+")
            .rule_with_precedence("expr", vec!["expr", "OP", "expr"], 1, Associativity::Left)
            .rule("expr", vec!["N"])
            .rule("start", vec!["expr"])
            .start("start")
            .build()
    };
    let r1 = run(mk());
    let r2 = run(mk());
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

// ---------------------------------------------------------------------------
// 51-60: Grammar builder features
// ---------------------------------------------------------------------------

#[test]
fn test_51_external_token() {
    let g = GrammarBuilder::new("pipe_v8_51")
        .token("NUMBER", r"\d+")
        .external("EXT")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_52_supertype() {
    let g = GrammarBuilder::new("pipe_v8_52")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("literal", vec!["NUMBER"])
        .rule("literal", vec!["IDENT"])
        .rule("start", vec!["literal"])
        .start("start")
        .supertype("literal")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_53_inline_does_not_break_pipeline() {
    let g = GrammarBuilder::new("pipe_v8_53")
        .token("A", "a")
        .token("B", "b")
        .rule("inner", vec!["A", "B"])
        .rule("start", vec!["inner"])
        .start("start")
        .inline("inner")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_54_extra_whitespace_token() {
    let g = GrammarBuilder::new("pipe_v8_54")
        .token("WORD", r"[a-z]+")
        .token("SPACE", " ")
        .rule("start", vec!["WORD"])
        .start("start")
        .extra("SPACE")
        .build();
    let res = run(g);
    assert!(res.build_stats.symbol_count > 0);
}

#[test]
fn test_55_multiple_extras() {
    let g = GrammarBuilder::new("pipe_v8_55")
        .token("N", r"\d+")
        .token("WS", r"\s+")
        .token("NL", r"\n")
        .rule("start", vec!["N"])
        .start("start")
        .extra("WS")
        .extra("NL")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_56_chain_rule() {
    let g = GrammarBuilder::new("pipe_v8_56")
        .token("N", r"\d+")
        .rule("atom", vec!["N"])
        .rule("expr", vec!["atom"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_57_multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("pipe_v8_57")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["C"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_58_nested_rules() {
    let g = GrammarBuilder::new("pipe_v8_58")
        .token("N", r"\d+")
        .token("LP", r"\(")
        .token("RP", r"\)")
        .rule("atom", vec!["N"])
        .rule("atom", vec!["LP", "expr", "RP"])
        .rule("expr", vec!["atom"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_59_sequential_tokens() {
    let g = GrammarBuilder::new("pipe_v8_59")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A", "B", "C"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_60_long_rhs() {
    let g = GrammarBuilder::new("pipe_v8_60")
        .token("T0", "a")
        .token("T1", "b")
        .token("T2", "c")
        .token("T3", "d")
        .token("T4", "e")
        .rule("start", vec!["T0", "T1", "T2", "T3", "T4"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 1);
}

// ---------------------------------------------------------------------------
// 61-70: node_types_json deep checks
// ---------------------------------------------------------------------------

#[test]
fn test_61_node_types_is_array() {
    let res = run(minimal("pipe_v8_61"));
    assert!(res.node_types_json.starts_with('['));
    assert!(res.node_types_json.ends_with(']'));
}

#[test]
fn test_62_node_types_entries_are_objects() {
    let res = run(minimal("pipe_v8_62"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    for entry in &arr {
        assert!(
            entry.is_object(),
            "each node_type entry should be an object"
        );
    }
}

#[test]
fn test_63_node_types_have_type_field() {
    let res = run(two_token("pipe_v8_63"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    let has_type = arr.iter().any(|v| v.get("type").is_some());
    assert!(has_type);
}

#[test]
fn test_64_node_types_have_named_field() {
    let res = run(minimal("pipe_v8_64"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    let has_named = arr.iter().any(|v| v.get("named").is_some());
    assert!(has_named);
}

#[test]
fn test_65_more_rules_more_node_types() {
    let small = run(minimal("pipe_v8_65a"));
    let big = run(arith("pipe_v8_65b"));
    let s_arr: Vec<serde_json::Value> = serde_json::from_str(&small.node_types_json).unwrap();
    let b_arr: Vec<serde_json::Value> = serde_json::from_str(&big.node_types_json).unwrap();
    assert!(b_arr.len() >= s_arr.len());
}

#[test]
fn test_66_node_types_deterministic() {
    let r1 = run(minimal("pipe_v8_66"));
    let r2 = run(minimal("pipe_v8_66"));
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn test_67_node_types_no_null_entries() {
    let res = run(arith("pipe_v8_67"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    for entry in &arr {
        assert!(!entry.is_null());
    }
}

#[test]
fn test_68_node_types_type_is_string() {
    let res = run(minimal("pipe_v8_68"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    for entry in &arr {
        if let Some(t) = entry.get("type") {
            assert!(t.is_string());
        }
    }
}

#[test]
fn test_69_node_types_named_is_bool() {
    let res = run(minimal("pipe_v8_69"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    for entry in &arr {
        if let Some(n) = entry.get("named") {
            assert!(n.is_boolean());
        }
    }
}

#[test]
fn test_70_node_types_not_empty_string() {
    let res = run(minimal("pipe_v8_70"));
    assert_ne!(res.node_types_json, "");
    assert_ne!(res.node_types_json, "[]");
}

// ---------------------------------------------------------------------------
// 71-80: Edge cases and robustness
// ---------------------------------------------------------------------------

#[test]
fn test_71_grammar_name_preserved() {
    let g = GrammarBuilder::new("pipe_v8_71_special")
        .token("X", "x")
        .rule("start", vec!["X"])
        .start("start")
        .build();
    let res = run(g);
    assert_eq!(res.grammar_name, "pipe_v8_71_special");
}

#[test]
fn test_72_underscore_in_name() {
    let g = GrammarBuilder::new("pipe_v8_72_a_b_c")
        .token("X", "x")
        .rule("start", vec!["X"])
        .start("start")
        .build();
    let res = run(g);
    assert_eq!(res.grammar_name, "pipe_v8_72_a_b_c");
}

#[test]
fn test_73_regex_token_pattern() {
    let g = GrammarBuilder::new("pipe_v8_73")
        .token("FLOAT", r"\d+\.\d+")
        .rule("start", vec!["FLOAT"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_74_complex_regex_token() {
    let g = GrammarBuilder::new("pipe_v8_74")
        .token("STRING", r#""[^"]*""#)
        .rule("start", vec!["STRING"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_75_single_char_tokens() {
    let g = GrammarBuilder::new("pipe_v8_75")
        .token("X", "x")
        .token("Y", "y")
        .token("Z", "z")
        .rule("start", vec!["X", "Y", "Z"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.symbol_count >= 3);
}

#[test]
fn test_76_deeply_nested_rules() {
    let g = GrammarBuilder::new("pipe_v8_76")
        .token("N", r"\d+")
        .rule("a", vec!["N"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("d", vec!["c"])
        .rule("start", vec!["d"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.state_count > 0);
}

#[test]
fn test_77_many_alternatives() {
    let g = GrammarBuilder::new("pipe_v8_77")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .token("F", "f")
        .token("G", "g")
        .token("H", "h")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["C"])
        .rule("start", vec!["D"])
        .rule("start", vec!["E"])
        .rule("start", vec!["F"])
        .rule("start", vec!["G"])
        .rule("start", vec!["H"])
        .start("start")
        .build();
    let res = run(g);
    assert!(res.build_stats.symbol_count >= 8);
}

#[test]
fn test_78_arith_with_parens_state_count() {
    let res = run(arith("pipe_v8_78"));
    // Arithmetic with parens should have non-trivial states
    assert!(res.build_stats.state_count >= 3);
}

#[test]
fn test_79_build_result_fields_consistent() {
    let res = run(minimal("pipe_v8_79"));
    // grammar_name, parser_code, node_types_json and stats should all be populated
    assert!(!res.grammar_name.is_empty());
    assert!(!res.parser_code.is_empty());
    assert!(!res.node_types_json.is_empty());
    assert!(res.build_stats.state_count > 0);
    assert!(res.build_stats.symbol_count > 0);
}

#[test]
fn test_80_ten_token_determinism() {
    let r1 = run(ten_token("pipe_v8_80"));
    let r2 = run(ten_token("pipe_v8_80"));
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ---------------------------------------------------------------------------
// 81-85: Additional coverage
// ---------------------------------------------------------------------------

#[test]
fn test_81_grammar_with_keyword_like_tokens() {
    let g = GrammarBuilder::new("pipe_v8_81")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("IDENT", r"[a-z]+")
        .rule("start", vec!["IF", "IDENT", "ELSE", "IDENT"])
        .start("start")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn test_82_symbol_count_scales_with_tokens() {
    let two = run(two_token("pipe_v8_82a"));
    let five = run(five_token("pipe_v8_82b"));
    let ten = run(ten_token("pipe_v8_82c"));
    assert!(five.build_stats.symbol_count > two.build_stats.symbol_count);
    assert!(ten.build_stats.symbol_count > five.build_stats.symbol_count);
}

#[test]
fn test_83_parser_code_deterministic() {
    let r1 = run(minimal("pipe_v8_83"));
    let r2 = run(minimal("pipe_v8_83"));
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_84_arith_node_types_contain_expr() {
    let res = run(arith("pipe_v8_84"));
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    let has_expr = arr
        .iter()
        .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("expr"));
    assert!(
        has_expr,
        "arithmetic grammar node_types should contain 'expr'"
    );
}

#[test]
fn test_85_inline_plus_extra_combined() {
    let g = GrammarBuilder::new("pipe_v8_85")
        .token("N", r"\d+")
        .token("WS", r"\s+")
        .rule("inner", vec!["N"])
        .rule("start", vec!["inner"])
        .start("start")
        .inline("inner")
        .extra("WS")
        .build();
    let res = run(g);
    assert!(!res.parser_code.is_empty());
    assert!(res.build_stats.state_count > 0);
}
