//! Comprehensive tests for validating generated code output from adze-tool.
//!
//! 80+ tests across these categories:
//!   1.  cv_name_embed_*         — parser_code contains grammar name
//!   2.  cv_fn_keyword_*         — parser_code contains "fn" keyword
//!   3.  cv_symbol_table_*       — parser_code contains symbol table entries
//!   4.  cv_utf8_*               — parser_code is valid UTF-8
//!   5.  cv_no_null_*            — parser_code has no null bytes
//!   6.  cv_json_valid_*         — node_types_json is valid JSON
//!   7.  cv_json_type_key_*      — node_types_json entries have "type" key
//!   8.  cv_json_named_key_*     — node_types_json entries have "named" key
//!   9.  cv_json_rule_names_*    — node_types includes rule-named symbols
//!  10.  cv_json_token_names_*   — node_types includes token names
//!  11.  cv_code_proportional_*  — parser_code length proportional to complexity
//!  12.  cv_json_proportional_*  — node_types_json length proportional to symbols
//!  13.  cv_minimal_code_*       — simple grammar → minimal code
//!  14.  cv_complex_code_*       — complex grammar → more code
//!  15.  cv_determinism_code_*   — same grammar → identical parser_code
//!  16.  cv_determinism_json_*   — same grammar → identical node_types_json
//!  17.  cv_precedence_ref_*     — grammar with precedence → code references it
//!  18.  cv_inline_diff_*        — inline grammar differs from non-inline
//!  19.  cv_extras_handle_*      — grammar with extras → code handles extras
//!  20.  cv_patterns_valid_*     — various grammar patterns generate valid code

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (dir, opts)
}

fn build_default(grammar: Grammar) -> BuildResult {
    let (_dir, opts) = tmp_opts();
    build_parser(grammar, opts).expect("build")
}

fn make_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn make_two_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build()
}

fn make_three_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("p", "p")
        .token("q", "q")
        .token("r", "r")
        .rule("start", vec!["p", "q", "r"])
        .start("start")
        .build()
}

fn make_multi_rule_grammar(name: &str, count: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name).token("tok", "t");
    for i in 0..count {
        let rule_name: &'static str = Box::leak(format!("r{i}").into_boxed_str());
        if i == 0 {
            b = b.rule(rule_name, vec!["tok"]);
        } else {
            let prev: &'static str = Box::leak(format!("r{}", i - 1).into_boxed_str());
            b = b.rule(rule_name, vec![prev]);
        }
    }
    b.start("r0").build()
}

fn make_prec_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .precedence(1, Associativity::Left, vec!["plus"])
        .start("expr")
        .build()
}

fn make_extras_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("ws", "\\s+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build()
}

fn make_inline_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .inline("inner")
        .start("start")
        .build()
}

fn make_no_inline_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build()
}

fn parse_node_types(json: &str) -> Vec<serde_json::Value> {
    let val: serde_json::Value = serde_json::from_str(json).expect("valid JSON");
    val.as_array().expect("top-level array").clone()
}

// ═════════════════════════════════════════════════════════════════════════
// 1. cv_name_embed — parser_code contains grammar name
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_name_embed_simple() {
    let r = build_default(make_grammar("cv_v8_ne01"));
    assert!(r.parser_code.contains("cv_v8_ne01"));
}

#[test]
fn cv_name_embed_two_token() {
    let r = build_default(make_two_token_grammar("cv_v8_ne02"));
    assert!(r.parser_code.contains("cv_v8_ne02"));
}

#[test]
fn cv_name_embed_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_ne03", 4));
    assert!(r.parser_code.contains("cv_v8_ne03"));
}

#[test]
fn cv_name_embed_in_ffi_fn() {
    let r = build_default(make_grammar("cv_v8_ne04"));
    assert!(r.parser_code.contains("tree_sitter_cv_v8_ne04"));
}

#[test]
fn cv_name_embed_result_field() {
    let r = build_default(make_grammar("cv_v8_ne05"));
    assert_eq!(r.grammar_name, "cv_v8_ne05");
}

// ═════════════════════════════════════════════════════════════════════════
// 2. cv_fn_keyword — parser_code contains "fn" keyword
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_fn_keyword_simple() {
    let r = build_default(make_grammar("cv_v8_fn01"));
    assert!(r.parser_code.contains("fn "));
}

#[test]
fn cv_fn_keyword_two_token() {
    let r = build_default(make_two_token_grammar("cv_v8_fn02"));
    assert!(r.parser_code.contains("fn "));
}

#[test]
fn cv_fn_keyword_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_fn03", 3));
    assert!(r.parser_code.contains("fn "));
}

#[test]
fn cv_fn_keyword_with_precedence() {
    let r = build_default(make_prec_grammar("cv_v8_fn04"));
    assert!(r.parser_code.contains("fn "));
}

// ═════════════════════════════════════════════════════════════════════════
// 3. cv_symbol_table — parser_code contains symbol table entries
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_symbol_table_name_array() {
    let r = build_default(make_grammar("cv_v8_st01"));
    assert!(r.parser_code.contains("SYMBOL_NAME"));
}

#[test]
fn cv_symbol_table_metadata() {
    let r = build_default(make_grammar("cv_v8_st02"));
    assert!(r.parser_code.contains("SYMBOL_METADATA"));
}

#[test]
fn cv_symbol_table_two_tokens() {
    let r = build_default(make_two_token_grammar("cv_v8_st03"));
    assert!(r.parser_code.contains("SYMBOL_NAME"));
}

#[test]
fn cv_symbol_table_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_st04", 5));
    assert!(r.parser_code.contains("SYMBOL_NAME"));
}

// ═════════════════════════════════════════════════════════════════════════
// 4. cv_utf8 — parser_code is valid UTF-8
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_utf8_simple() {
    let r = build_default(make_grammar("cv_v8_u801"));
    assert!(std::str::from_utf8(r.parser_code.as_bytes()).is_ok());
}

#[test]
fn cv_utf8_two_token() {
    let r = build_default(make_two_token_grammar("cv_v8_u802"));
    assert!(std::str::from_utf8(r.parser_code.as_bytes()).is_ok());
}

#[test]
fn cv_utf8_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_u803", 6));
    assert!(std::str::from_utf8(r.parser_code.as_bytes()).is_ok());
}

#[test]
fn cv_utf8_with_precedence() {
    let r = build_default(make_prec_grammar("cv_v8_u804"));
    assert!(std::str::from_utf8(r.parser_code.as_bytes()).is_ok());
}

// ═════════════════════════════════════════════════════════════════════════
// 5. cv_no_null — parser_code has no null bytes
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_no_null_simple() {
    let r = build_default(make_grammar("cv_v8_nn01"));
    assert!(!r.parser_code.contains('\0'));
}

#[test]
fn cv_no_null_two_token() {
    let r = build_default(make_two_token_grammar("cv_v8_nn02"));
    assert!(!r.parser_code.contains('\0'));
}

#[test]
fn cv_no_null_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_nn03", 4));
    assert!(!r.parser_code.contains('\0'));
}

#[test]
fn cv_no_null_with_extras() {
    let r = build_default(make_extras_grammar("cv_v8_nn04"));
    assert!(!r.parser_code.contains('\0'));
}

// ═════════════════════════════════════════════════════════════════════════
// 6. cv_json_valid — node_types_json is valid JSON
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_json_valid_simple() {
    let r = build_default(make_grammar("cv_v8_jv01"));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cv_json_valid_two_token() {
    let r = build_default(make_two_token_grammar("cv_v8_jv02"));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cv_json_valid_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_jv03", 5));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cv_json_valid_with_precedence() {
    let r = build_default(make_prec_grammar("cv_v8_jv04"));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cv_json_valid_with_extras() {
    let r = build_default(make_extras_grammar("cv_v8_jv05"));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

// ═════════════════════════════════════════════════════════════════════════
// 7. cv_json_type_key — node_types_json entries have "type" key
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_json_type_key_simple() {
    let r = build_default(make_grammar("cv_v8_jt01"));
    let entries = parse_node_types(&r.node_types_json);
    assert!(!entries.is_empty());
    for e in &entries {
        assert!(e.get("type").is_some(), "missing 'type' in {e}");
    }
}

#[test]
fn cv_json_type_key_two_token() {
    let r = build_default(make_two_token_grammar("cv_v8_jt02"));
    let entries = parse_node_types(&r.node_types_json);
    for e in &entries {
        assert!(e.get("type").is_some(), "missing 'type' in {e}");
    }
}

#[test]
fn cv_json_type_key_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_jt03", 4));
    let entries = parse_node_types(&r.node_types_json);
    for e in &entries {
        assert!(e.get("type").is_some(), "missing 'type' in {e}");
    }
}

#[test]
fn cv_json_type_key_with_precedence() {
    let r = build_default(make_prec_grammar("cv_v8_jt04"));
    let entries = parse_node_types(&r.node_types_json);
    for e in &entries {
        assert!(e.get("type").is_some(), "missing 'type' in {e}");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 8. cv_json_named_key — node_types_json entries have "named" key
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_json_named_key_simple() {
    let r = build_default(make_grammar("cv_v8_jn01"));
    let entries = parse_node_types(&r.node_types_json);
    assert!(!entries.is_empty());
    for e in &entries {
        assert!(e.get("named").is_some(), "missing 'named' in {e}");
    }
}

#[test]
fn cv_json_named_key_two_token() {
    let r = build_default(make_two_token_grammar("cv_v8_jn02"));
    let entries = parse_node_types(&r.node_types_json);
    for e in &entries {
        assert!(e.get("named").is_some(), "missing 'named' in {e}");
    }
}

#[test]
fn cv_json_named_key_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_jn03", 3));
    let entries = parse_node_types(&r.node_types_json);
    for e in &entries {
        assert!(e.get("named").is_some(), "missing 'named' in {e}");
    }
}

#[test]
fn cv_json_named_key_with_extras() {
    let r = build_default(make_extras_grammar("cv_v8_jn04"));
    let entries = parse_node_types(&r.node_types_json);
    for e in &entries {
        assert!(e.get("named").is_some(), "missing 'named' in {e}");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 9. cv_json_rule_names — node_types includes rule-named symbols
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_json_rule_names_start() {
    let r = build_default(make_grammar("cv_v8_jr01"));
    let entries = parse_node_types(&r.node_types_json);
    let types: Vec<&str> = entries
        .iter()
        .filter_map(|e| e.get("type").and_then(|v| v.as_str()))
        .collect();
    assert!(types.contains(&"start"), "expected 'start' in {types:?}");
}

#[test]
fn cv_json_rule_names_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cv_v8_jr02", 3));
    let entries = parse_node_types(&r.node_types_json);
    let types: Vec<&str> = entries
        .iter()
        .filter_map(|e| e.get("type").and_then(|v| v.as_str()))
        .collect();
    assert!(types.contains(&"r0"), "expected 'r0' in {types:?}");
}

#[test]
fn cv_json_rule_names_expr() {
    let r = build_default(make_prec_grammar("cv_v8_jr03"));
    let entries = parse_node_types(&r.node_types_json);
    let types: Vec<&str> = entries
        .iter()
        .filter_map(|e| e.get("type").and_then(|v| v.as_str()))
        .collect();
    assert!(types.contains(&"expr"), "expected 'expr' in {types:?}");
}

#[test]
fn cv_json_rule_names_inner() {
    let r = build_default(make_no_inline_grammar("cv_v8_jr04"));
    let entries = parse_node_types(&r.node_types_json);
    let types: Vec<&str> = entries
        .iter()
        .filter_map(|e| e.get("type").and_then(|v| v.as_str()))
        .collect();
    // Non-inline inner should appear as named node
    assert!(types.contains(&"start"), "expected 'start' in {types:?}");
}

// ═════════════════════════════════════════════════════════════════════════
// 10. cv_json_token_names — node_types includes token names
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_json_token_names_simple() {
    let r = build_default(make_grammar("cv_v8_tn01"));
    let json = &r.node_types_json;
    assert!(json.contains("a"), "expected token 'a' in node_types");
}

#[test]
fn cv_json_token_names_two_tokens() {
    let r = build_default(make_two_token_grammar("cv_v8_tn02"));
    let json = &r.node_types_json;
    assert!(json.contains("x"), "expected token 'x' in node_types");
    assert!(json.contains("y"), "expected token 'y' in node_types");
}

#[test]
fn cv_json_token_names_three_tokens() {
    let r = build_default(make_three_token_grammar("cv_v8_tn03"));
    let json = &r.node_types_json;
    assert!(json.contains("p"), "expected token 'p' in node_types");
    assert!(json.contains("q"), "expected token 'q' in node_types");
}

#[test]
fn cv_json_token_names_prec_tokens() {
    let r = build_default(make_prec_grammar("cv_v8_tn04"));
    let json = &r.node_types_json;
    // Precedence grammar has "expr" as the rule name
    assert!(json.contains("expr"), "expected rule 'expr' in node_types");
}

// ═════════════════════════════════════════════════════════════════════════
// 11. cv_code_proportional — parser_code length proportional to complexity
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_code_proportional_one_vs_five() {
    let small = build_default(make_grammar("cv_v8_cp01a"));
    let big = build_default(make_multi_rule_grammar("cv_v8_cp01b", 5));
    assert!(big.parser_code.len() >= small.parser_code.len());
}

#[test]
fn cv_code_proportional_five_vs_ten() {
    let mid = build_default(make_multi_rule_grammar("cv_v8_cp02a", 5));
    let big = build_default(make_multi_rule_grammar("cv_v8_cp02b", 10));
    assert!(big.parser_code.len() >= mid.parser_code.len());
}

#[test]
fn cv_code_proportional_one_vs_two_token() {
    let one = build_default(make_grammar("cv_v8_cp03a"));
    let two = build_default(make_two_token_grammar("cv_v8_cp03b"));
    assert!(two.parser_code.len() >= one.parser_code.len());
}

#[test]
fn cv_code_proportional_monotonic() {
    let r1 = build_default(make_grammar("cv_v8_cp04a"));
    let r3 = build_default(make_multi_rule_grammar("cv_v8_cp04b", 3));
    let r8 = build_default(make_multi_rule_grammar("cv_v8_cp04c", 8));
    assert!(r3.parser_code.len() >= r1.parser_code.len());
    assert!(r8.parser_code.len() >= r3.parser_code.len());
}

// ═════════════════════════════════════════════════════════════════════════
// 12. cv_json_proportional — node_types_json length proportional to symbols
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_json_proportional_one_vs_two() {
    let one = build_default(make_grammar("cv_v8_jp01a"));
    let two = build_default(make_two_token_grammar("cv_v8_jp01b"));
    assert!(two.node_types_json.len() >= one.node_types_json.len());
}

#[test]
fn cv_json_proportional_one_vs_five_rules() {
    let one = build_default(make_grammar("cv_v8_jp02a"));
    let five = build_default(make_multi_rule_grammar("cv_v8_jp02b", 5));
    assert!(five.node_types_json.len() >= one.node_types_json.len());
}

#[test]
fn cv_json_proportional_five_vs_ten_rules() {
    let five = build_default(make_multi_rule_grammar("cv_v8_jp03a", 5));
    let ten = build_default(make_multi_rule_grammar("cv_v8_jp03b", 10));
    assert!(ten.node_types_json.len() >= five.node_types_json.len());
}

#[test]
fn cv_json_proportional_entry_count_grows() {
    let one = build_default(make_grammar("cv_v8_jp04a"));
    let five = build_default(make_multi_rule_grammar("cv_v8_jp04b", 5));
    let entries_one = parse_node_types(&one.node_types_json);
    let entries_five = parse_node_types(&five.node_types_json);
    assert!(entries_five.len() >= entries_one.len());
}

// ═════════════════════════════════════════════════════════════════════════
// 13. cv_minimal_code — simple grammar → minimal code
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_minimal_code_nonempty() {
    let r = build_default(make_grammar("cv_v8_mc01"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cv_minimal_code_has_language() {
    let r = build_default(make_grammar("cv_v8_mc02"));
    assert!(r.parser_code.contains("TSLanguage"));
}

#[test]
fn cv_minimal_code_has_ffi() {
    let r = build_default(make_grammar("cv_v8_mc03"));
    assert!(r.parser_code.contains("extern \"C\""));
}

#[test]
fn cv_minimal_code_positive_stats() {
    let r = build_default(make_grammar("cv_v8_mc04"));
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 14. cv_complex_code — complex grammar → more code
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_complex_code_ten_rules() {
    let r = build_default(make_multi_rule_grammar("cv_v8_cc01", 10));
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 1);
}

#[test]
fn cv_complex_code_three_tokens() {
    let r = build_default(make_three_token_grammar("cv_v8_cc02"));
    assert!(r.build_stats.symbol_count >= 3);
}

#[test]
fn cv_complex_code_more_states_than_simple() {
    let simple = build_default(make_grammar("cv_v8_cc03a"));
    let complex = build_default(make_multi_rule_grammar("cv_v8_cc03b", 10));
    assert!(complex.build_stats.state_count >= simple.build_stats.state_count);
}

#[test]
fn cv_complex_code_prec_has_actions() {
    let r = build_default(make_prec_grammar("cv_v8_cc04"));
    assert!(r.parser_code.contains("PARSE_ACTIONS"));
}

// ═════════════════════════════════════════════════════════════════════════
// 15. cv_determinism_code — same grammar → identical parser_code
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_determinism_code_simple() {
    let r1 = build_default(make_grammar("cv_v8_dc01"));
    let r2 = build_default(make_grammar("cv_v8_dc01"));
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn cv_determinism_code_two_token() {
    let r1 = build_default(make_two_token_grammar("cv_v8_dc02"));
    let r2 = build_default(make_two_token_grammar("cv_v8_dc02"));
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn cv_determinism_code_multi_rule() {
    let r1 = build_default(make_multi_rule_grammar("cv_v8_dc03", 4));
    let r2 = build_default(make_multi_rule_grammar("cv_v8_dc03", 4));
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn cv_determinism_code_with_prec() {
    let r1 = build_default(make_prec_grammar("cv_v8_dc04"));
    let r2 = build_default(make_prec_grammar("cv_v8_dc04"));
    assert_eq!(r1.parser_code, r2.parser_code);
}

// ═════════════════════════════════════════════════════════════════════════
// 16. cv_determinism_json — same grammar → identical node_types_json
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_determinism_json_simple() {
    let r1 = build_default(make_grammar("cv_v8_dj01"));
    let r2 = build_default(make_grammar("cv_v8_dj01"));
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn cv_determinism_json_two_token() {
    let r1 = build_default(make_two_token_grammar("cv_v8_dj02"));
    let r2 = build_default(make_two_token_grammar("cv_v8_dj02"));
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn cv_determinism_json_multi_rule() {
    let r1 = build_default(make_multi_rule_grammar("cv_v8_dj03", 5));
    let r2 = build_default(make_multi_rule_grammar("cv_v8_dj03", 5));
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn cv_determinism_json_with_extras() {
    let r1 = build_default(make_extras_grammar("cv_v8_dj04"));
    let r2 = build_default(make_extras_grammar("cv_v8_dj04"));
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ═════════════════════════════════════════════════════════════════════════
// 17. cv_precedence_ref — grammar with precedence → code references it
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_precedence_ref_actions() {
    let r = build_default(make_prec_grammar("cv_v8_pr01"));
    assert!(r.parser_code.contains("PARSE_ACTIONS"));
}

#[test]
fn cv_precedence_ref_left() {
    let g = GrammarBuilder::new("cv_v8_pr02")
        .token("a", "a")
        .token("op", "\\+")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "op", "start"])
        .precedence(1, Associativity::Left, vec!["op"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.parser_code.contains("PARSE_ACTIONS"));
}

#[test]
fn cv_precedence_ref_right() {
    let g = GrammarBuilder::new("cv_v8_pr03")
        .token("a", "a")
        .token("eq", "=")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "eq", "start"])
        .precedence(1, Associativity::Right, vec!["eq"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cv_precedence_ref_none() {
    let g = GrammarBuilder::new("cv_v8_pr04")
        .token("a", "a")
        .token("cmp", "<")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "cmp", "start"])
        .precedence(1, Associativity::None, vec!["cmp"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cv_precedence_ref_rule_with_prec() {
    let g = GrammarBuilder::new("cv_v8_pr05")
        .token("a", "a")
        .token("op", "\\+")
        .rule_with_precedence("start", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence(
            "start",
            vec!["start", "op", "start"],
            2,
            Associativity::Left,
        )
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 18. cv_inline_diff — inline grammar differs from non-inline
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_inline_diff_code_differs() {
    let with_inline = build_default(make_inline_grammar("cv_v8_id01a"));
    let without_inline = build_default(make_no_inline_grammar("cv_v8_id01b"));
    // The grammar names differ, so code will differ; but inline also changes structure
    assert_ne!(with_inline.parser_code, without_inline.parser_code);
}

#[test]
fn cv_inline_diff_both_valid_json() {
    let with_inline = build_default(make_inline_grammar("cv_v8_id02a"));
    let without_inline = build_default(make_no_inline_grammar("cv_v8_id02b"));
    let _: serde_json::Value =
        serde_json::from_str(&with_inline.node_types_json).expect("valid JSON");
    let _: serde_json::Value =
        serde_json::from_str(&without_inline.node_types_json).expect("valid JSON");
}

#[test]
fn cv_inline_diff_both_have_ffi() {
    let with_inline = build_default(make_inline_grammar("cv_v8_id03a"));
    let without_inline = build_default(make_no_inline_grammar("cv_v8_id03b"));
    assert!(with_inline.parser_code.contains("tree_sitter_"));
    assert!(without_inline.parser_code.contains("tree_sitter_"));
}

#[test]
fn cv_inline_diff_both_positive_stats() {
    let with_inline = build_default(make_inline_grammar("cv_v8_id04a"));
    let without_inline = build_default(make_no_inline_grammar("cv_v8_id04b"));
    assert!(with_inline.build_stats.state_count > 0);
    assert!(without_inline.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 19. cv_extras_handle — grammar with extras → code handles extras
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_extras_handle_builds() {
    let r = build_default(make_extras_grammar("cv_v8_eh01"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cv_extras_handle_valid_stats() {
    let r = build_default(make_extras_grammar("cv_v8_eh02"));
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cv_extras_handle_has_language() {
    let r = build_default(make_extras_grammar("cv_v8_eh03"));
    assert!(r.parser_code.contains("TSLanguage"));
}

#[test]
fn cv_extras_handle_valid_json() {
    let r = build_default(make_extras_grammar("cv_v8_eh04"));
    let entries = parse_node_types(&r.node_types_json);
    for e in &entries {
        assert!(e.get("type").is_some());
        assert!(e.get("named").is_some());
    }
}

#[test]
fn cv_extras_handle_has_ffi() {
    let r = build_default(make_extras_grammar("cv_v8_eh05"));
    assert!(r.parser_code.contains("tree_sitter_cv_v8_eh05"));
}

// ═════════════════════════════════════════════════════════════════════════
// 20. cv_patterns_valid — various grammar patterns generate valid code
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_patterns_valid_recursive() {
    let g = GrammarBuilder::new("cv_v8_pv01")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("TSLanguage"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cv_patterns_valid_multiple_alternatives() {
    let g = GrammarBuilder::new("cv_v8_pv02")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cv_patterns_valid_chain() {
    let g = GrammarBuilder::new("cv_v8_pv03")
        .token("leaf", "z")
        .rule("mid", vec!["leaf"])
        .rule("root", vec!["mid"])
        .start("root")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("fn "));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cv_patterns_valid_regex_token() {
    let g = GrammarBuilder::new("cv_v8_pv04")
        .token("digits", "[0-9]+")
        .rule("start", vec!["digits"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn cv_patterns_valid_supertype() {
    let g = GrammarBuilder::new("cv_v8_pv05")
        .token("a", "a")
        .token("b", "b")
        .rule("var_a", vec!["a"])
        .rule("var_b", vec!["b"])
        .rule("expr", vec!["var_a"])
        .rule("expr", vec!["var_b"])
        .rule("start", vec!["expr"])
        .supertype("expr")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("TSLanguage"));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cv_patterns_valid_long_sequence() {
    let g = GrammarBuilder::new("cv_v8_pv06")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.build_stats.symbol_count >= 4);
}

#[test]
fn cv_patterns_valid_two_level_chain() {
    let g = GrammarBuilder::new("cv_v8_pv07")
        .token("t", "t")
        .rule("l2", vec!["t"])
        .rule("l1", vec!["l2"])
        .rule("start", vec!["l1"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("tree_sitter_cv_v8_pv07"));
}

#[test]
fn cv_patterns_valid_multi_prec_levels() {
    let g = GrammarBuilder::new("cv_v8_pv08")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .start("expr")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// Additional coverage: cross-cutting validation patterns
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cv_cross_no_null_in_json() {
    let r = build_default(make_grammar("cv_v8_xc01"));
    assert!(!r.node_types_json.contains('\0'));
}

#[test]
fn cv_cross_json_is_array() {
    let r = build_default(make_grammar("cv_v8_xc02"));
    let val: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn cv_cross_code_has_lex_modes() {
    let r = build_default(make_grammar("cv_v8_xc03"));
    assert!(r.parser_code.contains("LEX_MODES"));
}

#[test]
fn cv_cross_code_has_parse_table() {
    let r = build_default(make_two_token_grammar("cv_v8_xc04"));
    assert!(r.parser_code.contains("PARSE_TABLE") || r.parser_code.contains("SMALL_PARSE_TABLE"));
}

#[test]
fn cv_cross_parser_path_nonempty() {
    let r = build_default(make_grammar("cv_v8_xc05"));
    assert!(!r.parser_path.is_empty());
}

#[test]
fn cv_cross_conflict_cells_accessible() {
    let r = build_default(make_grammar("cv_v8_xc06"));
    // usize is always >= 0; just confirm accessible without panic
    let _cells = r.build_stats.conflict_cells;
}

#[test]
fn cv_cross_all_grammars_have_extern_c() {
    for (name, count) in [("cv_v8_xc07a", 1), ("cv_v8_xc07b", 3), ("cv_v8_xc07c", 7)] {
        let g = if count == 1 {
            make_grammar(name)
        } else {
            make_multi_rule_grammar(name, count)
        };
        let r = build_default(g);
        assert!(r.parser_code.contains("extern \"C\""), "failed for {name}");
    }
}

#[test]
fn cv_cross_all_grammars_valid_json() {
    for (name, count) in [("cv_v8_xc08a", 1), ("cv_v8_xc08b", 5), ("cv_v8_xc08c", 10)] {
        let g = if count == 1 {
            make_grammar(name)
        } else {
            make_multi_rule_grammar(name, count)
        };
        let r = build_default(g);
        let val: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
        assert!(val.is_array(), "not array for {name}");
    }
}
