//! Comprehensive tests for the pure Rust builder in `adze_tool::pure_rust_builder`.
//!
//! 50+ tests covering:
//! 1. Builder construction and configuration (BuildOptions)
//! 2. Code generation output (BuildResult, parser code, NODE_TYPES)
//! 3. Grammar conversion pipeline (IR, grammar.js, JSON)
//! 4. Error handling (invalid inputs, missing files)
//! 5. Edge cases (names, recursion, epsilon, fragile tokens, extras, precedence)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::GrammarConverter;
use adze_tool::grammar_js::{GrammarJs, GrammarJsConverter, Rule as GjsRule};
use adze_tool::pure_rust_builder::{
    BuildOptions, build_parser, build_parser_from_grammar_js, build_parser_from_json,
};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp_opts(compress: bool, emit: bool) -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: emit,
        compress_tables: compress,
    };
    (dir, opts)
}

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("NUMBER", r"\d+")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn simple_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [{ "type": "SYMBOL", "name": "expression" }]
            },
            "expression": { "type": "PATTERN", "value": r"\d+" }
        }
    })
    .to_string()
}

fn multi_rule_json(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "statement" },
            "statement": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "assignment" },
                    { "type": "SYMBOL", "name": "expression" }
                ]
            },
            "assignment": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "identifier" },
                    { "type": "STRING", "value": "=" },
                    { "type": "SYMBOL", "name": "expression" }
                ]
            },
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "identifier" },
                    { "type": "PATTERN", "value": r"\d+" }
                ]
            },
            "identifier": { "type": "PATTERN", "value": r"[a-zA-Z_][a-zA-Z0-9_]*" }
        }
    })
    .to_string()
}

fn write_grammar_js(dir: &Path, content: &str) -> std::path::PathBuf {
    let p = dir.join("grammar.js");
    fs::write(&p, content).unwrap();
    p
}

// ===========================================================================
// 1. BuildOptions — construction and configuration (tests 1–5)
// ===========================================================================

#[test]
fn t01_build_options_default_values() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
    assert!(opts.compress_tables);
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn t02_build_options_custom_values() {
    let opts = BuildOptions {
        out_dir: "/custom".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/custom");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn t03_build_options_clone() {
    let a = BuildOptions {
        out_dir: "x".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let b = a.clone();
    assert_eq!(a.out_dir, b.out_dir);
    assert_eq!(a.emit_artifacts, b.emit_artifacts);
    assert_eq!(a.compress_tables, b.compress_tables);
}

#[test]
fn t04_build_options_debug_format() {
    let opts = BuildOptions {
        out_dir: "/d".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let d = format!("{:?}", opts);
    assert!(d.contains("BuildOptions"));
    assert!(d.contains("out_dir"));
    assert!(d.contains("emit_artifacts"));
    assert!(d.contains("compress_tables"));
}

#[test]
fn t05_build_options_default_out_dir_nonempty() {
    // Even without OUT_DIR env, the default should fall back
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

// ===========================================================================
// 2. build_parser — basic success paths (tests 6–12)
// ===========================================================================

#[test]
fn t06_build_parser_minimal_compressed() {
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(minimal_grammar(), o).is_ok());
}

#[test]
fn t07_build_parser_minimal_uncompressed() {
    let (_d, o) = tmp_opts(false, false);
    assert!(build_parser(minimal_grammar(), o).is_ok());
}

#[test]
fn t08_build_parser_arithmetic() {
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(arith_grammar(), o).is_ok());
}

#[test]
fn t09_build_parser_sample_grammar() {
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(GrammarConverter::create_sample_grammar(), o).is_ok());
}

#[test]
fn t10_build_parser_python_like() {
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(GrammarBuilder::python_like(), o).is_ok());
}

#[test]
fn t11_build_parser_javascript_like() {
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(GrammarBuilder::javascript_like(), o).is_ok());
}

#[test]
fn t12_build_parser_sample_uncompressed() {
    let (_d, o) = tmp_opts(false, false);
    assert!(build_parser(GrammarConverter::create_sample_grammar(), o).is_ok());
}

// ===========================================================================
// 3. BuildResult — inspecting output (tests 13–22)
// ===========================================================================

#[test]
fn t13_result_grammar_name() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert_eq!(r.grammar_name, "minimal");
}

#[test]
fn t14_result_parser_path_contains_name() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(r.parser_path.contains("minimal"));
}

#[test]
fn t15_result_parser_code_nonempty() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(!r.parser_code.is_empty());
    assert!(r.parser_code.len() > 50);
}

#[test]
fn t16_result_node_types_valid_json() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn t17_result_debug_format() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let d = format!("{:?}", r);
    assert!(d.contains("BuildResult"));
    assert!(d.contains("minimal"));
}

#[test]
fn t18_result_parser_code_valid_tokens() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(r.parser_code.parse::<proc_macro2::TokenStream>().is_ok());
}

#[test]
fn t19_result_balanced_braces() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let open = r.parser_code.matches('{').count();
    let close = r.parser_code.matches('}').count();
    assert_eq!(open, close);
}

#[test]
fn t20_result_node_types_contains_source_file() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    let has = v.as_array().unwrap().iter().any(|e| {
        e.get("type")
            .and_then(|t| t.as_str())
            .is_some_and(|s| s == "source_file")
    });
    assert!(has, "NODE_TYPES should contain source_file");
}

#[test]
fn t21_node_types_entries_have_named() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("named").is_some(), "missing 'named': {entry}");
    }
}

#[test]
fn t22_result_all_fields_present() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(!r.grammar_name.is_empty());
    assert!(!r.parser_path.is_empty());
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

// ===========================================================================
// 4. BuildStats (tests 23–26)
// ===========================================================================

#[test]
fn t23_stats_positive_counts() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn t24_stats_debug_format() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let d = format!("{:?}", r.build_stats);
    assert!(d.contains("BuildStats"));
}

#[test]
fn t25_stats_python_like_nontrivial() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(GrammarBuilder::python_like(), o).unwrap();
    assert!(r.build_stats.state_count > 1);
    assert!(r.build_stats.symbol_count > 3);
}

#[test]
fn t26_stats_js_like_nontrivial() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(GrammarBuilder::javascript_like(), o).unwrap();
    assert!(r.build_stats.state_count > 1);
    assert!(r.build_stats.symbol_count > 5);
}

// ===========================================================================
// 5. Emit artifacts (tests 27–31)
// ===========================================================================

#[test]
fn t27_emit_creates_grammar_dir() {
    let (d, o) = tmp_opts(true, true);
    build_parser(minimal_grammar(), o).unwrap();
    assert!(d.path().join("grammar_minimal").exists());
}

#[test]
fn t28_emit_writes_ir_json() {
    let (d, o) = tmp_opts(true, true);
    build_parser(minimal_grammar(), o).unwrap();
    let p = d.path().join("grammar_minimal/grammar.ir.json");
    assert!(p.exists());
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    assert!(v.is_object());
}

#[test]
fn t29_emit_writes_node_types() {
    let (d, o) = tmp_opts(true, true);
    build_parser(minimal_grammar(), o).unwrap();
    assert!(d.path().join("grammar_minimal/NODE_TYPES.json").exists());
}

#[test]
fn t30_emit_creates_parser_file() {
    let (_d, o) = tmp_opts(true, true);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(Path::new(&r.parser_path).exists());
}

#[test]
fn t31_no_emit_still_writes_parser() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(Path::new(&r.parser_path).exists());
}

// ===========================================================================
// 6. Parser file content (tests 32–35)
// ===========================================================================

#[test]
fn t32_parser_file_header() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let c = fs::read_to_string(&r.parser_path).unwrap();
    assert!(c.contains("Auto-generated parser"));
    assert!(c.contains("adze pure-Rust builder"));
}

#[test]
fn t33_parser_file_grammar_name_const() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let c = fs::read_to_string(&r.parser_path).unwrap();
    assert!(c.contains("GRAMMAR_NAME"));
    assert!(c.contains("\"minimal\""));
}

#[test]
fn t34_parser_file_ends_with_newline() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    let c = fs::read_to_string(&r.parser_path).unwrap();
    assert!(c.ends_with('\n'));
}

#[test]
fn t35_parser_path_inside_out_dir() {
    let (d, o) = tmp_opts(true, false);
    let r = build_parser(minimal_grammar(), o).unwrap();
    assert!(r.parser_path.starts_with(&*d.path().to_string_lossy()));
}

// ===========================================================================
// 7. build_parser_from_grammar_js (tests 36–40)
// ===========================================================================

#[test]
fn t36_from_grammar_js_simple() {
    let (d, o) = tmp_opts(true, false);
    let p = write_grammar_js(
        d.path(),
        r#"
module.exports = grammar({
  name: 'simple',
  rules: { source_file: $ => $.expression, expression: $ => /\d+/ }
});
"#,
    );
    let r = build_parser_from_grammar_js(&p, o).unwrap();
    assert_eq!(r.grammar_name, "simple");
}

#[test]
fn t37_from_grammar_js_with_comments() {
    let (d, o) = tmp_opts(true, false);
    let p = write_grammar_js(
        d.path(),
        r#"
// comment
module.exports = grammar({
  name: 'commented',
  rules: { source_file: $ => $.expression, expression: $ => /\d+/ /* inline */ }
});
"#,
    );
    assert!(build_parser_from_grammar_js(&p, o).is_ok());
}

#[test]
fn t38_from_grammar_js_with_artifacts() {
    let (d, o) = tmp_opts(false, true);
    let p = write_grammar_js(
        d.path(),
        r#"
module.exports = grammar({
  name: 'art',
  rules: { source_file: $ => /\w+/ }
});
"#,
    );
    let r = build_parser_from_grammar_js(&p, o).unwrap();
    assert!(
        d.path()
            .join(format!("grammar_{}", r.grammar_name))
            .exists()
    );
}

#[test]
fn t39_from_grammar_js_nonexistent_file() {
    let (d, o) = tmp_opts(false, false);
    assert!(build_parser_from_grammar_js(&d.path().join("missing.js"), o).is_err());
}

#[test]
fn t40_from_grammar_js_invalid_content() {
    let (d, o) = tmp_opts(false, false);
    let p = write_grammar_js(d.path(), "this is not a valid grammar");
    assert!(build_parser_from_grammar_js(&p, o).is_err());
}

// ===========================================================================
// 8. build_parser_from_json (tests 41–48)
// ===========================================================================

#[test]
fn t41_from_json_simple() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser_from_json(simple_json("jt"), o).unwrap();
    assert_eq!(r.grammar_name, "jt");
}

#[test]
fn t42_from_json_multi_rule() {
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser_from_json(multi_rule_json("mr"), o).unwrap();
    assert_eq!(r.grammar_name, "mr");
}

#[test]
fn t43_from_json_invalid() {
    let (_d, o) = tmp_opts(false, false);
    assert!(build_parser_from_json("not json".into(), o).is_err());
}

#[test]
fn t44_from_json_empty_object() {
    let (_d, o) = tmp_opts(false, false);
    // Empty JSON may fail during conversion — just ensure no panic
    let _ = build_parser_from_json("{}".into(), o);
}

#[test]
fn t45_from_json_choice() {
    let j = json!({
        "name": "cj",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    {"type": "STRING", "value": "a"},
                    {"type": "STRING", "value": "b"}
                ]
            }
        }
    });
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser_from_json(j.to_string(), o).is_ok());
}

#[test]
fn t46_from_json_seq() {
    let j = json!({
        "name": "sj",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "x"},
                    {"type": "STRING", "value": "y"}
                ]
            }
        }
    });
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser_from_json(j.to_string(), o).is_ok());
}

#[test]
fn t47_from_json_repeat() {
    let j = json!({
        "name": "rj",
        "rules": {
            "source_file": { "type": "REPEAT", "content": {"type": "STRING", "value": "z"} }
        }
    });
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser_from_json(j.to_string(), o).is_ok());
}

#[test]
fn t48_from_json_with_extras() {
    let j = json!({
        "name": "ej",
        "extras": [{"type": "PATTERN", "value": "\\s+"}],
        "rules": {
            "source_file": {"type": "PATTERN", "value": "\\w+"}
        }
    });
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser_from_json(j.to_string(), o).is_ok());
}

// ===========================================================================
// 9. GrammarJsConverter pipeline (tests 49–55)
// ===========================================================================

#[test]
fn t49_converter_string_rule() {
    let mut gjs = GrammarJs::new("kw".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::String {
            value: "hello".into(),
        },
    );
    let g = GrammarJsConverter::new(gjs).convert().unwrap();
    assert_eq!(g.name, "kw");
}

#[test]
fn t50_converter_pattern_rule() {
    let mut gjs = GrammarJs::new("pat".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::Pattern {
            value: r"\d+".into(),
        },
    );
    let g = GrammarJsConverter::new(gjs).convert().unwrap();
    assert!(!g.tokens.is_empty());
}

#[test]
fn t51_converter_seq() {
    let mut gjs = GrammarJs::new("sq".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::Seq {
            members: vec![
                GjsRule::String { value: "a".into() },
                GjsRule::String { value: "b".into() },
            ],
        },
    );
    assert!(GrammarJsConverter::new(gjs).convert().is_ok());
}

#[test]
fn t52_converter_choice() {
    let mut gjs = GrammarJs::new("ch".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::Choice {
            members: vec![
                GjsRule::String { value: "a".into() },
                GjsRule::String { value: "b".into() },
            ],
        },
    );
    assert!(GrammarJsConverter::new(gjs).convert().is_ok());
}

#[test]
fn t53_converter_optional() {
    let mut gjs = GrammarJs::new("opt".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::Optional {
            value: Box::new(GjsRule::String {
                value: "maybe".into(),
            }),
        },
    );
    assert!(GrammarJsConverter::new(gjs).convert().is_ok());
}

#[test]
fn t54_converter_prec_left() {
    let mut gjs = GrammarJs::new("pl".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::PrecLeft {
            value: 1,
            content: Box::new(GjsRule::String { value: "t".into() }),
        },
    );
    assert!(GrammarJsConverter::new(gjs).convert().is_ok());
}

#[test]
fn t55_converter_prec_right() {
    let mut gjs = GrammarJs::new("pr".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::PrecRight {
            value: 2,
            content: Box::new(GjsRule::String { value: "r".into() }),
        },
    );
    assert!(GrammarJsConverter::new(gjs).convert().is_ok());
}

// ===========================================================================
// 10. GrammarBuilder → build_parser round-trips (tests 56–65)
// ===========================================================================

#[test]
fn t56_single_terminal() {
    let g = GrammarBuilder::new("st")
        .token("ID", r"[a-z]+")
        .rule("source_file", vec!["ID"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t57_multi_alternatives() {
    let g = GrammarBuilder::new("ma")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("source_file", vec!["A"])
        .rule("source_file", vec!["B"])
        .rule("source_file", vec!["C"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t58_nested_nonterminals() {
    let g = GrammarBuilder::new("nn")
        .token("X", "x")
        .rule("source_file", vec!["inner"])
        .rule("inner", vec!["X"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t59_left_recursive() {
    let g = GrammarBuilder::new("lr")
        .token("A", "a")
        .rule("source_file", vec!["source_file", "A"])
        .rule("source_file", vec!["A"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t60_right_recursive() {
    let g = GrammarBuilder::new("rr")
        .token("B", "b")
        .rule("source_file", vec!["B", "source_file"])
        .rule("source_file", vec!["B"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t61_with_extras() {
    let g = GrammarBuilder::new("we")
        .token("NUM", r"\d+")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("source_file", vec!["NUM"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t62_with_precedence() {
    let g = GrammarBuilder::new("wp")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t63_epsilon_production() {
    let g = GrammarBuilder::new("ep")
        .token("A", "a")
        .rule("source_file", vec!["A"])
        .rule("source_file", vec![]) // epsilon
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t64_deeply_nested() {
    let g = GrammarBuilder::new("dn")
        .token("X", "x")
        .rule("source_file", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["X"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t65_fragile_token() {
    let g = GrammarBuilder::new("ft")
        .token("NUM", r"\d+")
        .fragile_token("ERR", r".")
        .rule("source_file", vec!["NUM"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

// ===========================================================================
// 11. Compressed vs uncompressed parity (tests 66–67)
// ===========================================================================

#[test]
fn t66_both_modes_succeed() {
    let (_d1, o1) = tmp_opts(true, false);
    let (_d2, o2) = tmp_opts(false, false);
    let r1 = build_parser(minimal_grammar(), o1).unwrap();
    let r2 = build_parser(minimal_grammar(), o2).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
    assert!(!r1.parser_code.is_empty());
    assert!(!r2.parser_code.is_empty());
}

#[test]
fn t67_same_stats_both_modes() {
    let (_d1, o1) = tmp_opts(true, false);
    let (_d2, o2) = tmp_opts(false, false);
    let r1 = build_parser(minimal_grammar(), o1).unwrap();
    let r2 = build_parser(minimal_grammar(), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

// ===========================================================================
// 12. Edge cases (tests 68–74)
// ===========================================================================

#[test]
fn t68_name_with_hyphens_panics() {
    // Grammar names with hyphens cause a panic in code generation because
    // `tree_sitter_my-grammar` is not a valid Rust identifier. This is a
    // known limitation — the test documents the behaviour.
    let g = GrammarBuilder::new("my-grammar")
        .token("T", "t")
        .rule("source_file", vec!["T"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    let result = std::panic::catch_unwind(move || build_parser(g, o));
    assert!(
        result.is_err(),
        "hyphenated name should panic during codegen"
    );
}

#[test]
fn t69_name_with_underscores() {
    let g = GrammarBuilder::new("my_grammar")
        .token("T", "t")
        .rule("source_file", vec!["T"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    let r = build_parser(g, o).unwrap();
    assert_eq!(r.grammar_name, "my_grammar");
}

#[test]
fn t70_many_string_tokens() {
    let g = GrammarBuilder::new("mt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("source_file", vec!["A", "B", "C", "D"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t71_mixed_token_types() {
    let g = GrammarBuilder::new("mix")
        .token("ID", r"[a-z]+")
        .token("EQ", "=")
        .token("NUM", r"\d+")
        .token("SEMI", ";")
        .rule("source_file", vec!["ID", "EQ", "NUM", "SEMI"])
        .start("source_file")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn t72_determinism() {
    let j = simple_json("det");
    let (_d1, o1) = tmp_opts(true, false);
    let (_d2, o2) = tmp_opts(true, false);
    let r1 = build_parser_from_json(j.clone(), o1).unwrap();
    let r2 = build_parser_from_json(j, o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn t73_emit_overwrites_previous() {
    let (d, o1) = tmp_opts(true, true);
    build_parser(minimal_grammar(), o1).unwrap();
    let o2 = BuildOptions {
        out_dir: d.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r2 = build_parser(minimal_grammar(), o2).unwrap();
    assert!(Path::new(&r2.parser_path).exists());
}

#[test]
fn t74_from_json_nested_seq_choice() {
    let j = json!({
        "name": "nsc",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "SEQ",
                        "members": [
                            {"type": "STRING", "value": "a"},
                            {"type": "STRING", "value": "b"}
                        ]
                    },
                    {"type": "STRING", "value": "c"}
                ]
            }
        }
    });
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser_from_json(j.to_string(), o).is_ok());
}

// ===========================================================================
// 13. Grammar.js extended features (tests 75–78)
// ===========================================================================

#[test]
fn t75_grammar_js_choice() {
    let (d, o) = tmp_opts(false, false);
    let p = write_grammar_js(
        d.path(),
        r#"
module.exports = grammar({
  name: 'gchoice',
  rules: { source_file: $ => choice('a','b','c') }
});
"#,
    );
    assert!(build_parser_from_grammar_js(&p, o).is_ok());
}

#[test]
fn t76_grammar_js_seq() {
    let (d, o) = tmp_opts(false, false);
    let p = write_grammar_js(
        d.path(),
        r#"
module.exports = grammar({
  name: 'gseq',
  rules: { source_file: $ => seq('a','b') }
});
"#,
    );
    assert!(build_parser_from_grammar_js(&p, o).is_ok());
}

#[test]
fn t77_grammar_js_repeat() {
    let (d, o) = tmp_opts(false, false);
    let p = write_grammar_js(
        d.path(),
        r#"
module.exports = grammar({
  name: 'grep',
  rules: { source_file: $ => repeat($.item), item: $ => /\w+/ }
});
"#,
    );
    assert!(build_parser_from_grammar_js(&p, o).is_ok());
}

#[test]
fn t78_from_json_empty_rules_fails() {
    let j = json!({ "name": "er", "rules": {} });
    let (_d, o) = tmp_opts(false, false);
    assert!(build_parser_from_json(j.to_string(), o).is_err());
}

// ===========================================================================
// 14. JSON with extra/ignored fields, missing name (tests 79–80)
// ===========================================================================

#[test]
fn t79_from_json_extra_fields_ignored() {
    let j = json!({
        "name": "xf",
        "rules": { "source_file": {"type": "PATTERN", "value": "."} },
        "extra_field": 42,
        "nested": {"a": "b"}
    });
    let (_d, o) = tmp_opts(false, false);
    // Should succeed or fail gracefully — no panic
    let _ = build_parser_from_json(j.to_string(), o);
}

#[test]
fn t80_from_json_missing_name() {
    let j = json!({
        "rules": { "source_file": {"type": "PATTERN", "value": "x"} }
    });
    let (_d, o) = tmp_opts(false, false);
    // Should not panic
    let _ = build_parser_from_json(j.to_string(), o);
}

// ===========================================================================
// 15. Multi-name grammar name tests (test 81)
// ===========================================================================

#[test]
fn t81_various_grammar_names() {
    for name in ["alpha", "test123", "CamelCase", "lower_snake"] {
        let (_d, o) = tmp_opts(true, false);
        let r = build_parser_from_json(simple_json(name), o).unwrap();
        assert_eq!(r.grammar_name, name);
    }
}

// ===========================================================================
// 16. Converter produces tokens for repeat/repeat1 (tests 82–83)
// ===========================================================================

#[test]
fn t82_converter_repeat() {
    let mut gjs = GrammarJs::new("rp".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::Repeat {
            content: Box::new(GjsRule::String { value: "x".into() }),
        },
    );
    assert!(GrammarJsConverter::new(gjs).convert().is_ok());
}

#[test]
fn t83_converter_repeat1() {
    let mut gjs = GrammarJs::new("r1".into());
    gjs.rules.insert(
        "source_file".into(),
        GjsRule::Repeat1 {
            content: Box::new(GjsRule::String { value: "y".into() }),
        },
    );
    assert!(GrammarJsConverter::new(gjs).convert().is_ok());
}

// ===========================================================================
// 17. JSON with string literals (test 84)
// ===========================================================================

#[test]
fn t84_json_string_literal_keywords() {
    let j = json!({
        "name": "slk",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "stmt" },
            "stmt": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "let"},
                    {"type": "SYMBOL", "name": "id"}
                ]
            },
            "id": { "type": "PATTERN", "value": r"[a-zA-Z_]\w*" }
        }
    });
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser_from_json(j.to_string(), o).is_ok());
}

// ===========================================================================
// 18. Right-associativity (test 85)
// ===========================================================================

#[test]
fn t85_right_assoc_precedence() {
    let g = GrammarBuilder::new("ra")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let (_d, o) = tmp_opts(true, false);
    assert!(build_parser(g, o).is_ok());
}
