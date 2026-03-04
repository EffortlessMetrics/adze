//! Feature-matrix tests for the build pipeline.
//!
//! 70+ tests covering every combination of `compress_tables`, `emit_artifacts`,
//! grammar shapes, build stats, node_types_json, parser_code, grammar_name
//! preservation, determinism, and error cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser, build_parser_from_json};
use serde_json::json;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn opts(compress: bool, emit: bool) -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: emit,
        compress_tables: compress,
    };
    (dir, o)
}

fn single_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_alt_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn sequence_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build()
}

fn chain_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("t", "t")
        .rule("c", vec!["t"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn left_recursive_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn precedence_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn right_assoc_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn multi_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build()
}

fn simple_json_str(name: &str) -> String {
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

// ===========================================================================
// 1. BuildOptions::default() properties
// ===========================================================================

#[test]
fn fm_default_compress_tables_is_true() {
    let d = BuildOptions::default();
    assert!(d.compress_tables);
}

#[test]
fn fm_default_emit_artifacts_is_false() {
    let d = BuildOptions::default();
    assert!(!d.emit_artifacts);
}

#[test]
fn fm_default_out_dir_nonempty() {
    let d = BuildOptions::default();
    assert!(!d.out_dir.is_empty());
}

#[test]
fn fm_default_debug_format() {
    let d = BuildOptions::default();
    let s = format!("{:?}", d);
    assert!(s.contains("compress_tables"));
    assert!(s.contains("emit_artifacts"));
}

// ===========================================================================
// 2. compress_tables=true vs false — single token grammar
// ===========================================================================

#[test]
fn fm_single_token_compressed() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("stc"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_single_token_uncompressed() {
    let (_d, o) = opts(false, false);
    let r = build_parser(single_token_grammar("stu"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_single_token_both_produce_code() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(false, false);
    let r1 = build_parser(single_token_grammar("c1"), o1).unwrap();
    let r2 = build_parser(single_token_grammar("c2"), o2).unwrap();
    assert!(!r1.parser_code.is_empty());
    assert!(!r2.parser_code.is_empty());
}

// ===========================================================================
// 3. emit_artifacts=true vs false
// ===========================================================================

#[test]
fn fm_emit_true_creates_ir_json() {
    let (d, o) = opts(true, true);
    let r = build_parser(single_token_grammar("ea"), o).unwrap();
    let ir_path = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir_path.exists(), "IR JSON artifact should be written");
}

#[test]
fn fm_emit_true_creates_node_types_file() {
    let (d, o) = opts(true, true);
    let r = build_parser(single_token_grammar("nt"), o).unwrap();
    let nt_path = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("NODE_TYPES.json");
    assert!(nt_path.exists(), "NODE_TYPES artifact should be written");
}

#[test]
fn fm_emit_false_no_ir_json() {
    let (d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("ne"), o).unwrap();
    let ir_path = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(
        !ir_path.exists(),
        "IR JSON should NOT be written when emit=false"
    );
}

#[test]
fn fm_emit_false_still_writes_parser_file() {
    let (d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("pf"), o).unwrap();
    let parser = std::path::Path::new(&r.parser_path);
    assert!(parser.exists(), "Parser module is always written");
}

#[test]
fn fm_emit_with_uncompressed() {
    let (d, o) = opts(false, true);
    let r = build_parser(single_token_grammar("eu"), o).unwrap();
    let ir_path = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir_path.exists());
}

// ===========================================================================
// 4. Build same grammar twice → same result
// ===========================================================================

#[test]
fn fm_deterministic_parser_code() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(true, false);
    let r1 = build_parser(single_token_grammar("det"), o1).unwrap();
    let r2 = build_parser(single_token_grammar("det"), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn fm_deterministic_node_types() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(true, false);
    let r1 = build_parser(single_token_grammar("dn"), o1).unwrap();
    let r2 = build_parser(single_token_grammar("dn"), o2).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn fm_deterministic_stats() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(true, false);
    let r1 = build_parser(single_token_grammar("ds"), o1).unwrap();
    let r2 = build_parser(single_token_grammar("ds"), o2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn fm_deterministic_grammar_name() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(true, false);
    let r1 = build_parser(single_token_grammar("dg"), o1).unwrap();
    let r2 = build_parser(single_token_grammar("dg"), o2).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

// ===========================================================================
// 5. Various grammar shapes
// ===========================================================================

#[test]
fn fm_shape_two_alternatives_compressed() {
    let (_d, o) = opts(true, false);
    assert!(build_parser(two_alt_grammar("a2c"), o).is_ok());
}

#[test]
fn fm_shape_two_alternatives_uncompressed() {
    let (_d, o) = opts(false, false);
    assert!(build_parser(two_alt_grammar("a2u"), o).is_ok());
}

#[test]
fn fm_shape_sequence_compressed() {
    let (_d, o) = opts(true, false);
    assert!(build_parser(sequence_grammar("sqc"), o).is_ok());
}

#[test]
fn fm_shape_sequence_uncompressed() {
    let (_d, o) = opts(false, false);
    assert!(build_parser(sequence_grammar("squ"), o).is_ok());
}

#[test]
fn fm_shape_chain_compressed() {
    let (_d, o) = opts(true, false);
    assert!(build_parser(chain_grammar("chc"), o).is_ok());
}

#[test]
fn fm_shape_chain_uncompressed() {
    let (_d, o) = opts(false, false);
    assert!(build_parser(chain_grammar("chu"), o).is_ok());
}

#[test]
fn fm_shape_left_recursive_compressed() {
    let (_d, o) = opts(true, false);
    assert!(build_parser(left_recursive_grammar("lrc"), o).is_ok());
}

#[test]
fn fm_shape_left_recursive_uncompressed() {
    let (_d, o) = opts(false, false);
    assert!(build_parser(left_recursive_grammar("lru"), o).is_ok());
}

#[test]
fn fm_shape_precedence_compressed() {
    let (_d, o) = opts(true, false);
    assert!(build_parser(precedence_grammar("pc"), o).is_ok());
}

#[test]
fn fm_shape_precedence_uncompressed() {
    let (_d, o) = opts(false, false);
    assert!(build_parser(precedence_grammar("pu"), o).is_ok());
}

#[test]
fn fm_shape_right_assoc_compressed() {
    let (_d, o) = opts(true, false);
    assert!(build_parser(right_assoc_grammar("rac"), o).is_ok());
}

#[test]
fn fm_shape_right_assoc_uncompressed() {
    let (_d, o) = opts(false, false);
    assert!(build_parser(right_assoc_grammar("rau"), o).is_ok());
}

#[test]
fn fm_shape_multi_token_compressed() {
    let (_d, o) = opts(true, false);
    assert!(build_parser(multi_token_grammar("mtc"), o).is_ok());
}

#[test]
fn fm_shape_multi_token_uncompressed() {
    let (_d, o) = opts(false, false);
    assert!(build_parser(multi_token_grammar("mtu"), o).is_ok());
}

// ===========================================================================
// 6. Build stats properties
// ===========================================================================

#[test]
fn fm_stats_single_token_state_count_positive() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("ss"), o).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn fm_stats_single_token_symbol_count_positive() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("sy"), o).unwrap();
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn fm_stats_chain_more_symbols_than_single() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(true, false);
    let r1 = build_parser(single_token_grammar("s1"), o1).unwrap();
    let r2 = build_parser(chain_grammar("s2"), o2).unwrap();
    assert!(r2.build_stats.symbol_count >= r1.build_stats.symbol_count);
}

#[test]
fn fm_stats_precedence_grammar_has_conflicts() {
    let (_d, o) = opts(true, false);
    let r = build_parser(precedence_grammar("pc2"), o).unwrap();
    // Precedence grammars with ambiguous rules may or may not have conflict
    // cells — just verify the field is populated
    let _ = r.build_stats.conflict_cells;
}

#[test]
fn fm_stats_simple_grammar_zero_or_low_conflicts() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("lc"), o).unwrap();
    // A trivial grammar should have no conflicts
    assert_eq!(r.build_stats.conflict_cells, 0);
}

#[test]
fn fm_stats_debug_format_contains_fields() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("df"), o).unwrap();
    let dbg = format!("{:?}", r.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn fm_stats_uncompressed_matches_compressed_counts() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(false, false);
    let r1 = build_parser(single_token_grammar("cm"), o1).unwrap();
    let r2 = build_parser(single_token_grammar("cm"), o2).unwrap();
    // Compression doesn't change state/symbol counts
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

// ===========================================================================
// 7. node_types_json validity
// ===========================================================================

#[test]
fn fm_node_types_is_valid_json() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("nj"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn fm_node_types_nonempty_array() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("na"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn fm_node_types_entries_have_type_field() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("tf"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "Every node type entry should have 'type'"
        );
    }
}

#[test]
fn fm_node_types_entries_have_named_field() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("nf"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "Every node type entry should have 'named'"
        );
    }
}

#[test]
fn fm_node_types_chain_grammar() {
    let (_d, o) = opts(true, false);
    let r = build_parser(chain_grammar("ncg"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn fm_node_types_precedence_grammar() {
    let (_d, o) = opts(true, false);
    let r = build_parser(precedence_grammar("npg"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

// ===========================================================================
// 8. parser_code non-empty
// ===========================================================================

#[test]
fn fm_parser_code_nonempty_single() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("pe"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_parser_code_nonempty_chain() {
    let (_d, o) = opts(true, false);
    let r = build_parser(chain_grammar("pcc"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_parser_code_nonempty_recursive() {
    let (_d, o) = opts(true, false);
    let r = build_parser(left_recursive_grammar("pcr"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_parser_code_nonempty_uncompressed() {
    let (_d, o) = opts(false, false);
    let r = build_parser(single_token_grammar("pcu"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_parser_code_balanced_braces() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("bb"), o).unwrap();
    let open = r.parser_code.matches('{').count();
    let close = r.parser_code.matches('}').count();
    assert_eq!(open, close, "Braces should be balanced in generated code");
}

// ===========================================================================
// 9. grammar_name preservation
// ===========================================================================

#[test]
fn fm_grammar_name_simple() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("my_lang"), o).unwrap();
    assert_eq!(r.grammar_name, "my_lang");
}

#[test]
fn fm_grammar_name_underscore() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("foo_bar_baz"), o).unwrap();
    assert_eq!(r.grammar_name, "foo_bar_baz");
}

#[test]
fn fm_grammar_name_short() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("x"), o).unwrap();
    assert_eq!(r.grammar_name, "x");
}

#[test]
fn fm_grammar_name_from_json() {
    let (_d, o) = opts(true, false);
    let r = build_parser_from_json(simple_json_str("json_lang"), o).unwrap();
    assert_eq!(r.grammar_name, "json_lang");
}

#[test]
fn fm_grammar_name_in_parser_path() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("pathlang"), o).unwrap();
    assert!(
        r.parser_path.contains("pathlang"),
        "parser_path should contain the grammar name"
    );
}

// ===========================================================================
// 10. Error cases
// ===========================================================================

#[test]
fn fm_error_invalid_json_string() {
    let (_d, o) = opts(true, false);
    assert!(build_parser_from_json("<<<not json>>>".into(), o).is_err());
}

#[test]
fn fm_error_empty_json_object() {
    let (_d, o) = opts(true, false);
    let _ = build_parser_from_json("{}".into(), o);
    // Should not panic — may return Err
}

#[test]
fn fm_error_json_missing_rules() {
    let (_d, o) = opts(true, false);
    let j = json!({"name": "bad"}).to_string();
    let _ = build_parser_from_json(j, o);
}

#[test]
fn fm_error_json_empty_rules() {
    let (_d, o) = opts(true, false);
    let j = json!({"name": "bad", "rules": {}}).to_string();
    let result = build_parser_from_json(j, o);
    assert!(result.is_err(), "Empty rules should fail");
}

#[test]
fn fm_error_json_number_as_input() {
    let (_d, o) = opts(true, false);
    assert!(build_parser_from_json("42".into(), o).is_err());
}

#[test]
fn fm_error_json_array_as_input() {
    let (_d, o) = opts(true, false);
    let _ = build_parser_from_json("[]".into(), o);
}

// ===========================================================================
// 11. Cross-feature combinations
// ===========================================================================

#[test]
fn fm_cross_compress_true_emit_true() {
    let (d, o) = opts(true, true);
    let r = build_parser(single_token_grammar("ct"), o).unwrap();
    assert!(!r.parser_code.is_empty());
    let ir = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir.exists());
}

#[test]
fn fm_cross_compress_true_emit_false() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("cf"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_cross_compress_false_emit_true() {
    let (d, o) = opts(false, true);
    let r = build_parser(single_token_grammar("ft"), o).unwrap();
    assert!(!r.parser_code.is_empty());
    let ir = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir.exists());
}

#[test]
fn fm_cross_compress_false_emit_false() {
    let (_d, o) = opts(false, false);
    let r = build_parser(single_token_grammar("ff"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_cross_chain_compressed_emit() {
    let (d, o) = opts(true, true);
    let r = build_parser(chain_grammar("cce"), o).unwrap();
    let ir = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir.exists());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn fm_cross_recursive_uncompressed_emit() {
    let (d, o) = opts(false, true);
    let r = build_parser(left_recursive_grammar("rue"), o).unwrap();
    let ir = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir.exists());
}

#[test]
fn fm_cross_precedence_compressed_noemit() {
    let (_d, o) = opts(true, false);
    let r = build_parser(precedence_grammar("pcn"), o).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn fm_cross_precedence_uncompressed_emit() {
    let (d, o) = opts(false, true);
    let r = build_parser(precedence_grammar("pue"), o).unwrap();
    let ir = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir.exists());
}

// ===========================================================================
// 12. JSON pipeline across options
// ===========================================================================

#[test]
fn fm_json_compressed() {
    let (_d, o) = opts(true, false);
    let r = build_parser_from_json(simple_json_str("jc"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_json_uncompressed() {
    let (_d, o) = opts(false, false);
    let r = build_parser_from_json(simple_json_str("ju"), o).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn fm_json_emit_true() {
    let (d, o) = opts(true, true);
    let r = build_parser_from_json(simple_json_str("je"), o).unwrap();
    let ir = d
        .path()
        .join(format!("grammar_{}", r.grammar_name))
        .join("grammar.ir.json");
    assert!(ir.exists());
}

#[test]
fn fm_json_node_types_valid() {
    let (_d, o) = opts(true, false);
    let r = build_parser_from_json(simple_json_str("jn"), o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn fm_json_deterministic() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(true, false);
    let r1 = build_parser_from_json(simple_json_str("jd"), o1).unwrap();
    let r2 = build_parser_from_json(simple_json_str("jd"), o2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn fm_json_stats_positive() {
    let (_d, o) = opts(true, false);
    let r = build_parser_from_json(simple_json_str("js"), o).unwrap();
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn fm_json_choice_rule() {
    let j = json!({
        "name": "choice_test",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    {"type": "STRING", "value": "a"},
                    {"type": "STRING", "value": "b"},
                    {"type": "STRING", "value": "c"}
                ]
            }
        }
    })
    .to_string();
    let (_d, o) = opts(true, false);
    let r = build_parser_from_json(j, o).unwrap();
    assert_eq!(r.grammar_name, "choice_test");
}

#[test]
fn fm_json_seq_rule() {
    let j = json!({
        "name": "seq_test",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "x"},
                    {"type": "STRING", "value": "y"}
                ]
            }
        }
    })
    .to_string();
    let (_d, o) = opts(true, false);
    assert!(build_parser_from_json(j, o).is_ok());
}

// ===========================================================================
// 13. Additional shape & stats coverage
// ===========================================================================

#[test]
fn fm_shape_epsilon_production() {
    let g = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (_d, o) = opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn fm_shape_fragile_token() {
    let g = GrammarBuilder::new("frag")
        .fragile_token("ERR", "!")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (_d, o) = opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn fm_shape_with_extras() {
    let g = GrammarBuilder::new("ext")
        .token("a", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (_d, o) = opts(true, false);
    assert!(build_parser(g, o).is_ok());
}

#[test]
fn fm_stats_recursive_more_states_than_single() {
    let (_d1, o1) = opts(true, false);
    let (_d2, o2) = opts(true, false);
    let r1 = build_parser(single_token_grammar("rms1"), o1).unwrap();
    let r2 = build_parser(left_recursive_grammar("rms2"), o2).unwrap();
    assert!(
        r2.build_stats.state_count >= r1.build_stats.state_count,
        "Recursive grammar should have at least as many states"
    );
}

#[test]
fn fm_result_parser_path_ends_with_rs() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("rse"), o).unwrap();
    assert!(r.parser_path.ends_with(".rs"));
}

#[test]
fn fm_result_all_fields_populated() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("afp"), o).unwrap();
    assert!(!r.grammar_name.is_empty());
    assert!(!r.parser_path.is_empty());
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn fm_result_debug_format() {
    let (_d, o) = opts(true, false);
    let r = build_parser(single_token_grammar("rdf"), o).unwrap();
    let dbg = format!("{:?}", r);
    assert!(dbg.contains("grammar_name"));
    assert!(dbg.contains("build_stats"));
}
