//! Comprehensive tests for grammar construction and build validation in adze-tool.
//!
//! 80+ tests covering:
//!   1.  Single token grammar builds
//!   2.  Two token grammar builds
//!   3.  Five token grammar builds
//!   4.  Grammar with alternatives builds
//!   5.  Grammar with chain rules builds
//!   6.  Grammar with left precedence builds
//!   7.  Grammar with right precedence builds
//!   8.  Grammar with None associativity builds
//!   9.  Grammar with inline rules builds
//!  10.  Grammar with extra tokens builds
//!  11.  Grammar with external tokens builds
//!  12.  Grammar with conflicts (via precedence) builds
//!  13.  Grammar with supertypes builds
//!  14.  Complex arithmetic grammar builds
//!  15.  Grammar with regex-like token patterns builds
//!  16.  Build stats scale with complexity
//!  17.  node_types_json reflects grammar structure
//!  18.  Different grammars produce different code
//!  19.  Same grammar structure → deterministic stats
//!  20.  Extremely simple grammar (1 token, 1 rule)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, o)
}

fn build_grammar(g: Grammar) -> BuildResult {
    let (_dir, o) = opts();
    build_parser(g, o).expect("build should succeed")
}

fn single_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build()
}

fn two_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn five_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build()
}

fn alt_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn chain_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("c", vec!["x"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn left_prec_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

fn right_prec_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("eq", "=")
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

fn none_prec_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("cmp", "<")
        .rule_with_precedence("expr", vec!["expr", "cmp", "expr"], 1, Associativity::None)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

fn inline_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .rule("helper", vec!["x"])
        .rule("helper", vec!["y"])
        .rule("start", vec!["helper"])
        .inline("helper")
        .start("start")
        .build()
}

fn extras_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("word", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .rule("start", vec!["word"])
        .extra("ws")
        .start("start")
        .build()
}

fn external_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("id", r"[a-z]+")
        .token("indent", "INDENT")
        .token("dedent", "DEDENT")
        .external("indent")
        .external("dedent")
        .rule("start", vec!["id"])
        .start("start")
        .build()
}

fn supertype_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .rule("literal", vec!["num"])
        .rule("identifier", vec!["id"])
        .rule("expression", vec!["literal"])
        .rule("expression", vec!["identifier"])
        .supertype("expression")
        .rule("start", vec!["expression"])
        .start("start")
        .build()
}

fn arithmetic_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

fn regex_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .token("number", r"[0-9]+(\.[0-9]+)?")
        .rule("start", vec!["ident"])
        .rule("start", vec!["number"])
        .start("start")
        .build()
}

fn parse_node_types(json: &str) -> serde_json::Value {
    serde_json::from_str(json).expect("node_types_json should be valid JSON")
}

// ═════════════════════════════════════════════════════════════════════════
// 1. Single token grammar builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_single_token_builds_ok() {
    let r = build_grammar(single_token("gb_v8_st_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_single_token_state_count_positive() {
    let r = build_grammar(single_token("gb_v8_st_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_single_token_symbol_count_positive() {
    let r = build_grammar(single_token("gb_v8_st_sym"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_single_token_grammar_name() {
    let r = build_grammar(single_token("gb_v8_st_name"));
    assert_eq!(r.grammar_name, "gb_v8_st_name");
}

// ═════════════════════════════════════════════════════════════════════════
// 2. Two token grammar builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_two_token_builds_ok() {
    let r = build_grammar(two_token("gb_v8_tt_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_two_token_symbol_count_gte_two() {
    let r = build_grammar(two_token("gb_v8_tt_sym"));
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn test_two_token_grammar_name() {
    let r = build_grammar(two_token("gb_v8_tt_name"));
    assert_eq!(r.grammar_name, "gb_v8_tt_name");
}

#[test]
fn test_two_token_node_types_valid_json() {
    let r = build_grammar(two_token("gb_v8_tt_json"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 3. Five token grammar builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_five_token_builds_ok() {
    let r = build_grammar(five_token("gb_v8_ft_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_five_token_symbol_count_gte_five() {
    let r = build_grammar(five_token("gb_v8_ft_sym"));
    assert!(r.build_stats.symbol_count >= 5);
}

#[test]
fn test_five_token_state_count_positive() {
    let r = build_grammar(five_token("gb_v8_ft_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_five_token_parser_code_contains_name() {
    let r = build_grammar(five_token("gb_v8_ft_pc"));
    assert!(r.parser_code.contains("gb_v8_ft_pc"));
}

// ═════════════════════════════════════════════════════════════════════════
// 4. Grammar with alternatives builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_alternatives_builds_ok() {
    let r = build_grammar(alt_grammar("gb_v8_alt_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_alternatives_state_count_positive() {
    let r = build_grammar(alt_grammar("gb_v8_alt_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_alternatives_node_types_array() {
    let r = build_grammar(alt_grammar("gb_v8_alt_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_alternatives_grammar_name() {
    let r = build_grammar(alt_grammar("gb_v8_alt_gn"));
    assert_eq!(r.grammar_name, "gb_v8_alt_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 5. Grammar with chain rules builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_chain_builds_ok() {
    let r = build_grammar(chain_grammar("gb_v8_ch_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_chain_state_count_positive() {
    let r = build_grammar(chain_grammar("gb_v8_ch_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_chain_symbol_count_gte_three() {
    let r = build_grammar(chain_grammar("gb_v8_ch_sym"));
    // b, c, start are non-terminals; x is a terminal
    assert!(r.build_stats.symbol_count >= 3);
}

#[test]
fn test_chain_parser_path_non_empty() {
    let r = build_grammar(chain_grammar("gb_v8_ch_pp"));
    assert!(!r.parser_path.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 6. Grammar with left precedence builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_left_prec_builds_ok() {
    let r = build_grammar(left_prec_grammar("gb_v8_lp_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_left_prec_state_count_positive() {
    let r = build_grammar(left_prec_grammar("gb_v8_lp_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_left_prec_symbol_count_positive() {
    let r = build_grammar(left_prec_grammar("gb_v8_lp_sym"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_left_prec_node_types_valid() {
    let r = build_grammar(left_prec_grammar("gb_v8_lp_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 7. Grammar with right precedence builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_right_prec_builds_ok() {
    let r = build_grammar(right_prec_grammar("gb_v8_rp_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_right_prec_state_count_positive() {
    let r = build_grammar(right_prec_grammar("gb_v8_rp_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_right_prec_grammar_name() {
    let r = build_grammar(right_prec_grammar("gb_v8_rp_gn"));
    assert_eq!(r.grammar_name, "gb_v8_rp_gn");
}

#[test]
fn test_right_prec_parser_code_contains_name() {
    let r = build_grammar(right_prec_grammar("gb_v8_rp_pc"));
    assert!(r.parser_code.contains("gb_v8_rp_pc"));
}

// ═════════════════════════════════════════════════════════════════════════
// 8. Grammar with None associativity builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_none_prec_builds_ok() {
    let r = build_grammar(none_prec_grammar("gb_v8_np_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_none_prec_state_count_positive() {
    let r = build_grammar(none_prec_grammar("gb_v8_np_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_none_prec_symbol_count_positive() {
    let r = build_grammar(none_prec_grammar("gb_v8_np_sym"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_none_prec_node_types_valid() {
    let r = build_grammar(none_prec_grammar("gb_v8_np_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 9. Grammar with inline rules builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_inline_builds_ok() {
    let r = build_grammar(inline_grammar("gb_v8_inl_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_inline_state_count_positive() {
    let r = build_grammar(inline_grammar("gb_v8_inl_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_inline_grammar_name() {
    let r = build_grammar(inline_grammar("gb_v8_inl_gn"));
    assert_eq!(r.grammar_name, "gb_v8_inl_gn");
}

#[test]
fn test_inline_parser_path_non_empty() {
    let r = build_grammar(inline_grammar("gb_v8_inl_pp"));
    assert!(!r.parser_path.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 10. Grammar with extra tokens builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_extras_builds_ok() {
    let r = build_grammar(extras_grammar("gb_v8_ext_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_extras_state_count_positive() {
    let r = build_grammar(extras_grammar("gb_v8_ext_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_extras_node_types_valid() {
    let r = build_grammar(extras_grammar("gb_v8_ext_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_extras_grammar_name() {
    let r = build_grammar(extras_grammar("gb_v8_ext_gn"));
    assert_eq!(r.grammar_name, "gb_v8_ext_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 11. Grammar with external tokens builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_external_builds_ok() {
    let r = build_grammar(external_grammar("gb_v8_extn_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_external_state_count_positive() {
    let r = build_grammar(external_grammar("gb_v8_extn_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_external_symbol_count_positive() {
    let r = build_grammar(external_grammar("gb_v8_extn_sym"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_external_grammar_name() {
    let r = build_grammar(external_grammar("gb_v8_extn_gn"));
    assert_eq!(r.grammar_name, "gb_v8_extn_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 12. Grammar with conflicts (via competing precedence) builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_conflict_prec_builds_ok() {
    // Two operators at the same precedence level create a conflict scenario
    let g = GrammarBuilder::new("gb_v8_conf_ok")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("minus", "-")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_conflict_prec_state_count_positive() {
    let g = GrammarBuilder::new("gb_v8_conf_sc")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("minus", "-")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_conflict_mixed_assoc_builds() {
    let g = GrammarBuilder::new("gb_v8_conf_mix")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("eq", "=")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 2, Associativity::Right)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_conflict_prec_node_types_valid() {
    let g = GrammarBuilder::new("gb_v8_conf_nt")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("minus", "-")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let r = build_grammar(g);
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 13. Grammar with supertypes builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_supertype_builds_ok() {
    let r = build_grammar(supertype_grammar("gb_v8_sup_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_supertype_state_count_positive() {
    let r = build_grammar(supertype_grammar("gb_v8_sup_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_supertype_symbol_count_positive() {
    let r = build_grammar(supertype_grammar("gb_v8_sup_sym"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_supertype_grammar_name() {
    let r = build_grammar(supertype_grammar("gb_v8_sup_gn"));
    assert_eq!(r.grammar_name, "gb_v8_sup_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 14. Complex arithmetic grammar builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_arithmetic_builds_ok() {
    let r = build_grammar(arithmetic_grammar("gb_v8_arith_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_arithmetic_state_count_positive() {
    let r = build_grammar(arithmetic_grammar("gb_v8_arith_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_arithmetic_symbol_count_gte_five() {
    let r = build_grammar(arithmetic_grammar("gb_v8_arith_sym"));
    // num, plus, star, lparen, rparen, expr, start
    assert!(r.build_stats.symbol_count >= 5);
}

#[test]
fn test_arithmetic_node_types_valid() {
    let r = build_grammar(arithmetic_grammar("gb_v8_arith_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 15. Grammar with regex-like token patterns builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_regex_builds_ok() {
    let r = build_grammar(regex_grammar("gb_v8_rx_ok"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_regex_state_count_positive() {
    let r = build_grammar(regex_grammar("gb_v8_rx_sc"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_regex_symbol_count_positive() {
    let r = build_grammar(regex_grammar("gb_v8_rx_sym"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_regex_grammar_name() {
    let r = build_grammar(regex_grammar("gb_v8_rx_gn"));
    assert_eq!(r.grammar_name, "gb_v8_rx_gn");
}

// ═════════════════════════════════════════════════════════════════════════
// 16. Build stats scale with complexity
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_stats_single_vs_five_token_states() {
    let r1 = build_grammar(single_token("gb_v8_scale_s1"));
    let r5 = build_grammar(five_token("gb_v8_scale_s5"));
    assert!(r5.build_stats.state_count >= r1.build_stats.state_count);
}

#[test]
fn test_stats_single_vs_five_token_symbols() {
    let r1 = build_grammar(single_token("gb_v8_scale_sy1"));
    let r5 = build_grammar(five_token("gb_v8_scale_sy5"));
    assert!(r5.build_stats.symbol_count >= r1.build_stats.symbol_count);
}

#[test]
fn test_stats_chain_vs_arithmetic_states() {
    let rc = build_grammar(chain_grammar("gb_v8_scale_ch"));
    let ra = build_grammar(arithmetic_grammar("gb_v8_scale_ar"));
    assert!(ra.build_stats.state_count >= rc.build_stats.state_count);
}

#[test]
fn test_stats_chain_vs_arithmetic_symbols() {
    let rc = build_grammar(chain_grammar("gb_v8_scale_chs"));
    let ra = build_grammar(arithmetic_grammar("gb_v8_scale_ars"));
    assert!(ra.build_stats.symbol_count >= rc.build_stats.symbol_count);
}

// ═════════════════════════════════════════════════════════════════════════
// 17. node_types_json reflects grammar structure
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_node_types_single_token_is_array() {
    let r = build_grammar(single_token("gb_v8_nt_st"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_node_types_entries_have_type_field() {
    let r = build_grammar(two_token("gb_v8_nt_type"));
    let v = parse_node_types(&r.node_types_json);
    let arr = v.as_array().expect("array");
    for entry in arr {
        assert!(entry.get("type").is_some(), "entry missing 'type' field");
    }
}

#[test]
fn test_node_types_entries_have_named_field() {
    let r = build_grammar(two_token("gb_v8_nt_named"));
    let v = parse_node_types(&r.node_types_json);
    let arr = v.as_array().expect("array");
    for entry in arr {
        assert!(entry.get("named").is_some(), "entry missing 'named' field");
    }
}

#[test]
fn test_node_types_type_values_non_empty() {
    let r = build_grammar(alt_grammar("gb_v8_nt_tv"));
    let v = parse_node_types(&r.node_types_json);
    let arr = v.as_array().expect("array");
    for entry in arr {
        let t = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
        assert!(!t.is_empty(), "type value should be non-empty");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 18. Different grammars produce different code
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_different_grammars_different_code_single_vs_two() {
    let r1 = build_grammar(single_token("gb_v8_diff_s"));
    let r2 = build_grammar(two_token("gb_v8_diff_t"));
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_different_grammars_different_code_chain_vs_alt() {
    let r1 = build_grammar(chain_grammar("gb_v8_diff_ch"));
    let r2 = build_grammar(alt_grammar("gb_v8_diff_al"));
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_different_grammars_different_name() {
    let r1 = build_grammar(single_token("gb_v8_dn_a"));
    let r2 = build_grammar(single_token("gb_v8_dn_b"));
    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn test_different_grammars_different_parser_path() {
    let r1 = build_grammar(single_token("gb_v8_dp_a"));
    let r2 = build_grammar(single_token("gb_v8_dp_b"));
    assert_ne!(r1.parser_path, r2.parser_path);
}

// ═════════════════════════════════════════════════════════════════════════
// 19. Same grammar structure → deterministic stats
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_deterministic_state_count() {
    let r1 = build_grammar(single_token("gb_v8_det_a"));
    let r2 = build_grammar(single_token("gb_v8_det_b"));
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn test_deterministic_symbol_count() {
    let r1 = build_grammar(single_token("gb_v8_detsym_a"));
    let r2 = build_grammar(single_token("gb_v8_detsym_b"));
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn test_deterministic_conflict_cells() {
    let r1 = build_grammar(single_token("gb_v8_detcc_a"));
    let r2 = build_grammar(single_token("gb_v8_detcc_b"));
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn test_deterministic_two_token_stats() {
    let r1 = build_grammar(two_token("gb_v8_det2_a"));
    let r2 = build_grammar(two_token("gb_v8_det2_b"));
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

// ═════════════════════════════════════════════════════════════════════════
// 20. Extremely simple grammar (1 token, 1 rule)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_minimal_grammar_builds() {
    let g = GrammarBuilder::new("gb_v8_min")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_minimal_grammar_state_count_bounded() {
    let g = GrammarBuilder::new("gb_v8_min_sc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.state_count < 100);
}

#[test]
fn test_minimal_grammar_symbol_count_bounded() {
    let g = GrammarBuilder::new("gb_v8_min_sym")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.symbol_count < 50);
}

#[test]
fn test_minimal_grammar_parser_path_contains_name() {
    let g = GrammarBuilder::new("gb_v8_min_pp")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.parser_path.contains("gb_v8_min_pp"));
}

// ═════════════════════════════════════════════════════════════════════════
// Additional coverage: cross-cutting concerns
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_parser_code_no_null_bytes() {
    let r = build_grammar(single_token("gb_v8_nonull"));
    assert!(!r.parser_code.contains('\0'));
}

#[test]
fn test_node_types_json_no_null_bytes() {
    let r = build_grammar(two_token("gb_v8_ntnull"));
    assert!(!r.node_types_json.contains('\0'));
}

#[test]
fn test_parser_path_no_null_bytes() {
    let r = build_grammar(single_token("gb_v8_ppnull"));
    assert!(!r.parser_path.contains('\0'));
}

#[test]
fn test_grammar_name_roundtrip() {
    let r = build_grammar(arithmetic_grammar("gb_v8_roundtrip"));
    assert_eq!(r.grammar_name, "gb_v8_roundtrip");
}

#[test]
fn test_inline_plus_extras_builds() {
    let g = GrammarBuilder::new("gb_v8_inl_ext")
        .token("x", "x")
        .token("y", "y")
        .token("ws", r"[ \t]+")
        .rule("helper", vec!["x"])
        .rule("helper", vec!["y"])
        .rule("start", vec!["helper"])
        .inline("helper")
        .extra("ws")
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_supertype_plus_extras_builds() {
    let g = GrammarBuilder::new("gb_v8_sup_ext")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .rule("literal", vec!["num"])
        .rule("identifier", vec!["id"])
        .rule("expression", vec!["literal"])
        .rule("expression", vec!["identifier"])
        .supertype("expression")
        .extra("ws")
        .rule("start", vec!["expression"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_left_prec_different_levels_builds() {
    let g = GrammarBuilder::new("gb_v8_lp_levels")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_right_prec_different_levels_builds() {
    let g = GrammarBuilder::new("gb_v8_rp_levels")
        .token("num", r"\d+")
        .token("eq", "=")
        .token("arrow", "=>")
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::Right)
        .rule_with_precedence(
            "expr",
            vec!["expr", "arrow", "expr"],
            2,
            Associativity::Right,
        )
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_three_alt_grammar_builds() {
    let g = GrammarBuilder::new("gb_v8_3alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_deep_chain_builds() {
    let g = GrammarBuilder::new("gb_v8_deep")
        .token("x", "x")
        .rule("e", vec!["x"])
        .rule("d", vec!["e"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_external_plus_prec_builds() {
    let g = GrammarBuilder::new("gb_v8_ext_prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("indent", "INDENT")
        .external("indent")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_multi_token_regex_builds() {
    let g = GrammarBuilder::new("gb_v8_multirx")
        .token("ident", r"[a-zA-Z_]+")
        .token("number", r"[0-9]+")
        .token("string", r#""[^"]*""#)
        .rule("start", vec!["ident"])
        .rule("start", vec!["number"])
        .rule("start", vec!["string"])
        .start("start")
        .build();
    let r = build_grammar(g);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_build_stats_debug_format() {
    let r = build_grammar(single_token("gb_v8_dbg_bs"));
    let debug = format!("{:?}", r.build_stats);
    assert!(debug.contains("BuildStats"));
}

#[test]
fn test_build_result_debug_format() {
    let r = build_grammar(single_token("gb_v8_dbg_br"));
    let debug = format!("{:?}", r);
    assert!(debug.contains("BuildResult"));
}

#[test]
fn test_build_options_default_compress_true() {
    let d = BuildOptions::default();
    assert!(d.compress_tables);
}

#[test]
fn test_build_options_default_emit_false() {
    let d = BuildOptions::default();
    assert!(!d.emit_artifacts);
}

#[test]
fn test_build_options_debug_format() {
    let d = BuildOptions::default();
    let debug = format!("{:?}", d);
    assert!(debug.contains("BuildOptions"));
}

#[test]
fn test_node_types_no_null_entries() {
    let r = build_grammar(arithmetic_grammar("gb_v8_nt_nonull"));
    let v = parse_node_types(&r.node_types_json);
    let arr = v.as_array().expect("array");
    for entry in arr {
        assert!(!entry.is_null());
    }
}

#[test]
fn test_five_token_node_types_valid() {
    let r = build_grammar(five_token("gb_v8_ft_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_chain_node_types_valid() {
    let r = build_grammar(chain_grammar("gb_v8_ch_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_inline_node_types_valid() {
    let r = build_grammar(inline_grammar("gb_v8_inl_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_supertype_node_types_valid() {
    let r = build_grammar(supertype_grammar("gb_v8_sup_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_external_node_types_valid() {
    let r = build_grammar(external_grammar("gb_v8_extn_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

#[test]
fn test_extras_parser_path_contains_name() {
    let r = build_grammar(extras_grammar("gb_v8_ext_pp"));
    assert!(r.parser_path.contains("gb_v8_ext_pp"));
}

#[test]
fn test_arithmetic_parser_code_contains_name() {
    let r = build_grammar(arithmetic_grammar("gb_v8_arith_pc"));
    assert!(r.parser_code.contains("gb_v8_arith_pc"));
}

#[test]
fn test_stats_not_absurdly_large_state_count() {
    let r = build_grammar(arithmetic_grammar("gb_v8_abs_sc"));
    assert!(r.build_stats.state_count < 10_000);
}

#[test]
fn test_stats_not_absurdly_large_symbol_count() {
    let r = build_grammar(arithmetic_grammar("gb_v8_abs_sym"));
    assert!(r.build_stats.symbol_count < 10_000);
}
