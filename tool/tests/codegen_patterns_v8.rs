//! Comprehensive tests for parser code generation patterns in adze-tool.
//!
//! 80+ tests across these categories:
//!   1.  cg_lang_struct_*        — TSLanguage struct in parser_code
//!   2.  cg_lexer_*              — Lexer function presence
//!   3.  cg_ffi_prefix_*         — tree_sitter_ prefix function
//!   4.  cg_grammar_name_*       — Grammar name embedded in output
//!   5.  cg_parse_table_*        — PARSE_TABLE / SMALL_PARSE_TABLE arrays
//!   6.  cg_symbol_names_*       — Symbol name arrays
//!   7.  cg_extern_c_*           — extern "C" linkage
//!   8.  cg_multi_token_*        — Multiple token patterns
//!   9.  cg_precedence_*         — Precedence / parse actions
//!  10.  cg_node_json_valid_*    — node_types_json validity
//!  11.  cg_node_json_nonempty_* — node_types_json non-empty
//!  12.  cg_stats_states_*       — build_stats state_count > 0
//!  13.  cg_stats_symbols_*      — build_stats symbol_count > 0
//!  14.  cg_name_differs_*       — Different grammar names → different functions
//!  15.  cg_extras_*             — Grammar with extras
//!  16.  cg_inline_*             — Grammar with inline rules
//!  17.  cg_supertype_*          — Grammar with supertypes
//!  18.  cg_compress_*           — Compress vs non-compress tables
//!  19.  cg_emit_*               — Emit artifacts option
//!  20.  cg_complexity_*         — Various grammar complexities

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        ..BuildOptions::default()
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

// ═════════════════════════════════════════════════════════════════════════
// 1. cg_lang_struct — TSLanguage struct in parser_code
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_lang_struct_present() {
    let r = build_default(make_grammar("cgls01"));
    assert!(r.parser_code.contains("TSLanguage"));
}

#[test]
fn cg_lang_struct_static_binding() {
    let r = build_default(make_grammar("cgls02"));
    assert!(r.parser_code.contains("LANGUAGE"));
}

#[test]
fn cg_lang_struct_version_field() {
    let r = build_default(make_grammar("cgls03"));
    assert!(r.parser_code.contains("version"));
}

#[test]
fn cg_lang_struct_symbol_count_field() {
    let r = build_default(make_grammar("cgls04"));
    assert!(r.parser_code.contains("symbol_count"));
}

#[test]
fn cg_lang_struct_state_count_field() {
    let r = build_default(make_grammar("cgls05"));
    assert!(r.parser_code.contains("state_count"));
}

// ═════════════════════════════════════════════════════════════════════════
// 2. cg_lexer — Lexer function presence
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_lexer_fn_present() {
    let r = build_default(make_grammar("cglx01"));
    assert!(r.parser_code.contains("lexer_fn"));
}

#[test]
fn cg_lexer_fn_in_language_struct() {
    let r = build_default(make_grammar("cglx02"));
    assert!(r.parser_code.contains("lex_fn"));
}

#[test]
fn cg_lexer_fn_with_two_tokens() {
    let r = build_default(make_two_token_grammar("cglx03"));
    assert!(r.parser_code.contains("lexer_fn"));
}

#[test]
fn cg_lexer_fn_with_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cglx04", 3));
    assert!(r.parser_code.contains("lexer_fn"));
}

// ═════════════════════════════════════════════════════════════════════════
// 3. cg_ffi_prefix — tree_sitter_ prefix function
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_ffi_prefix_present() {
    let r = build_default(make_grammar("cgfp01"));
    assert!(r.parser_code.contains("tree_sitter_"));
}

#[test]
fn cg_ffi_prefix_with_grammar_name() {
    let r = build_default(make_grammar("cgfp02"));
    assert!(r.parser_code.contains("tree_sitter_cgfp02"));
}

#[test]
fn cg_ffi_prefix_returns_language_ptr() {
    let r = build_default(make_grammar("cgfp03"));
    // TokenStream may render `* const` with a space
    assert!(
        r.parser_code.contains("* const TSLanguage") || r.parser_code.contains("*const TSLanguage")
    );
}

#[test]
fn cg_ffi_prefix_two_token_grammar() {
    let r = build_default(make_two_token_grammar("cgfp04"));
    assert!(r.parser_code.contains("tree_sitter_cgfp04"));
}

// ═════════════════════════════════════════════════════════════════════════
// 4. cg_grammar_name — Grammar name embedded in output
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_grammar_name_in_function() {
    let r = build_default(make_grammar("cgname01"));
    assert!(r.parser_code.contains("cgname01"));
}

#[test]
fn cg_grammar_name_in_result() {
    let r = build_default(make_grammar("cgname02"));
    assert_eq!(r.grammar_name, "cgname02");
}

#[test]
fn cg_grammar_name_two_token() {
    let r = build_default(make_two_token_grammar("cgname03"));
    assert!(r.parser_code.contains("cgname03"));
}

#[test]
fn cg_grammar_name_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cgname04", 5));
    assert!(r.parser_code.contains("cgname04"));
}

// ═════════════════════════════════════════════════════════════════════════
// 5. cg_parse_table — PARSE_TABLE / SMALL_PARSE_TABLE arrays
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_parse_table_present() {
    let r = build_default(make_grammar("cgpt01"));
    assert!(r.parser_code.contains("PARSE_TABLE") || r.parser_code.contains("SMALL_PARSE_TABLE"));
}

#[test]
fn cg_parse_table_small_variant() {
    let r = build_default(make_grammar("cgpt02"));
    assert!(r.parser_code.contains("SMALL_PARSE_TABLE"));
}

#[test]
fn cg_parse_table_map_present() {
    let r = build_default(make_grammar("cgpt03"));
    assert!(r.parser_code.contains("SMALL_PARSE_TABLE_MAP"));
}

#[test]
fn cg_parse_table_two_tokens() {
    let r = build_default(make_two_token_grammar("cgpt04"));
    assert!(r.parser_code.contains("PARSE_TABLE") || r.parser_code.contains("SMALL_PARSE_TABLE"));
}

#[test]
fn cg_parse_table_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cgpt05", 4));
    assert!(r.parser_code.contains("SMALL_PARSE_TABLE"));
}

// ═════════════════════════════════════════════════════════════════════════
// 6. cg_symbol_names — Symbol name arrays
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_symbol_names_array_present() {
    let r = build_default(make_grammar("cgsn01"));
    assert!(r.parser_code.contains("SYMBOL_NAME"));
}

#[test]
fn cg_symbol_names_ptrs_array() {
    let r = build_default(make_grammar("cgsn02"));
    assert!(r.parser_code.contains("SYMBOL_NAME_PTRS"));
}

#[test]
fn cg_symbol_names_metadata_present() {
    let r = build_default(make_grammar("cgsn03"));
    assert!(r.parser_code.contains("SYMBOL_METADATA"));
}

#[test]
fn cg_symbol_names_two_token_grammar() {
    let r = build_default(make_two_token_grammar("cgsn04"));
    assert!(r.parser_code.contains("SYMBOL_NAME"));
}

// ═════════════════════════════════════════════════════════════════════════
// 7. cg_extern_c — extern "C" linkage
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_extern_c_present() {
    let r = build_default(make_grammar("cgec01"));
    assert!(r.parser_code.contains("extern \"C\""));
}

#[test]
fn cg_extern_c_on_language_fn() {
    let r = build_default(make_grammar("cgec02"));
    // The extern "C" fn should reference the tree_sitter_ function
    let code = &r.parser_code;
    assert!(code.contains("extern \"C\""));
    assert!(code.contains("tree_sitter_cgec02"));
}

#[test]
fn cg_extern_c_unsafe_qualifier() {
    let r = build_default(make_grammar("cgec03"));
    assert!(r.parser_code.contains("unsafe"));
}

#[test]
fn cg_extern_c_two_token_grammar() {
    let r = build_default(make_two_token_grammar("cgec04"));
    assert!(r.parser_code.contains("extern \"C\""));
}

// ═════════════════════════════════════════════════════════════════════════
// 8. cg_multi_token — Multiple token patterns
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_multi_token_both_present() {
    let g = GrammarBuilder::new("cgmt01")
        .token("alpha", "a")
        .token("beta", "b")
        .rule("start", vec!["alpha", "beta"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn cg_multi_token_three_tokens() {
    let g = GrammarBuilder::new("cgmt02")
        .token("p", "p")
        .token("q", "q")
        .token("r", "r")
        .rule("start", vec!["p", "q", "r"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.build_stats.symbol_count >= 3);
}

#[test]
fn cg_multi_token_symbol_count_scales() {
    let g2 = make_two_token_grammar("cgmt03a");
    let g3 = GrammarBuilder::new("cgmt03b")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let r2 = build_default(g2);
    let r3 = build_default(g3);
    assert!(r3.build_stats.symbol_count >= r2.build_stats.symbol_count);
}

#[test]
fn cg_multi_token_generates_lexer() {
    let g = GrammarBuilder::new("cgmt04")
        .token("m", "m")
        .token("n", "n")
        .rule("start", vec!["m", "n"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("lexer_fn"));
}

// ═════════════════════════════════════════════════════════════════════════
// 9. cg_precedence — Precedence / parse actions
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_precedence_actions_present() {
    let g = GrammarBuilder::new("cgpr01")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .precedence(1, Associativity::Left, vec!["plus"])
        .start("expr")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("PARSE_ACTIONS"));
}

#[test]
fn cg_precedence_left_assoc_builds() {
    let g = GrammarBuilder::new("cgpr02")
        .token("a", "a")
        .token("op", "\\+")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "op", "start"])
        .precedence(1, Associativity::Left, vec!["op"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_precedence_right_assoc_builds() {
    let g = GrammarBuilder::new("cgpr03")
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
fn cg_precedence_none_assoc_builds() {
    let g = GrammarBuilder::new("cgpr04")
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
fn cg_precedence_parse_actions_nonempty() {
    let g = GrammarBuilder::new("cgpr05")
        .token("a", "a")
        .token("op", "\\+")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "op", "start"])
        .precedence(2, Associativity::Left, vec!["op"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("PARSE_ACTIONS"));
}

// ═════════════════════════════════════════════════════════════════════════
// 10. cg_node_json_valid — node_types_json is valid JSON
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_node_json_valid_simple() {
    let r = build_default(make_grammar("cgnj01"));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cg_node_json_valid_two_tokens() {
    let r = build_default(make_two_token_grammar("cgnj02"));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cg_node_json_valid_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cgnj03", 5));
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

#[test]
fn cg_node_json_valid_with_precedence() {
    let g = GrammarBuilder::new("cgnj04")
        .token("a", "a")
        .token("op", "\\+")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "op", "start"])
        .precedence(1, Associativity::Left, vec!["op"])
        .start("start")
        .build();
    let r = build_default(g);
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

// ═════════════════════════════════════════════════════════════════════════
// 11. cg_node_json_nonempty — node_types_json is non-empty
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_node_json_nonempty_simple() {
    let r = build_default(make_grammar("cgne01"));
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn cg_node_json_nonempty_two_tokens() {
    let r = build_default(make_two_token_grammar("cgne02"));
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn cg_node_json_nonempty_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cgne03", 3));
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn cg_node_json_nonempty_with_precedence() {
    let g = GrammarBuilder::new("cgne04")
        .token("a", "a")
        .token("op", "\\+")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "op", "start"])
        .precedence(1, Associativity::Left, vec!["op"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.node_types_json.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 12. cg_stats_states — build_stats state_count > 0
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_stats_states_simple() {
    let r = build_default(make_grammar("cgss01"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_stats_states_two_tokens() {
    let r = build_default(make_two_token_grammar("cgss02"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_stats_states_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cgss03", 4));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_stats_states_increase_with_complexity() {
    let small = build_default(make_grammar("cgss04a"));
    let big = build_default(make_multi_rule_grammar("cgss04b", 6));
    assert!(big.build_stats.state_count >= small.build_stats.state_count);
}

// ═════════════════════════════════════════════════════════════════════════
// 13. cg_stats_symbols — build_stats symbol_count > 0
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_stats_symbols_simple() {
    let r = build_default(make_grammar("cgsym01"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cg_stats_symbols_two_tokens() {
    let r = build_default(make_two_token_grammar("cgsym02"));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cg_stats_symbols_multi_rule() {
    let r = build_default(make_multi_rule_grammar("cgsym03", 5));
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cg_stats_symbols_increase_with_tokens() {
    let one = build_default(make_grammar("cgsym04a"));
    let two = build_default(make_two_token_grammar("cgsym04b"));
    assert!(two.build_stats.symbol_count >= one.build_stats.symbol_count);
}

// ═════════════════════════════════════════════════════════════════════════
// 14. cg_name_differs — Different grammar names → different functions
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_name_differs_function_names() {
    let r1 = build_default(make_grammar("cgnd01a"));
    let r2 = build_default(make_grammar("cgnd01b"));
    assert!(r1.parser_code.contains("tree_sitter_cgnd01a"));
    assert!(r2.parser_code.contains("tree_sitter_cgnd01b"));
    assert!(!r1.parser_code.contains("tree_sitter_cgnd01b"));
    assert!(!r2.parser_code.contains("tree_sitter_cgnd01a"));
}

#[test]
fn cg_name_differs_grammar_name_field() {
    let r1 = build_default(make_grammar("cgnd02a"));
    let r2 = build_default(make_grammar("cgnd02b"));
    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn cg_name_differs_parser_code_differs() {
    let r1 = build_default(make_grammar("cgnd03a"));
    let r2 = build_default(make_grammar("cgnd03b"));
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn cg_name_differs_node_json_may_differ() {
    let r1 = build_default(make_grammar("cgnd04a"));
    let r2 = build_default(make_two_token_grammar("cgnd04b"));
    // Different grammars should produce different node_types
    assert_ne!(r1.node_types_json, r2.node_types_json);
}

// ═════════════════════════════════════════════════════════════════════════
// 15. cg_extras — Grammar with extras
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_extras_whitespace_builds() {
    let g = GrammarBuilder::new("cgex01")
        .token("a", "a")
        .token("ws", "\\s+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_extras_whitespace_valid_stats() {
    let g = GrammarBuilder::new("cgex02")
        .token("a", "a")
        .token("ws", "\\s+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cg_extras_still_has_language_struct() {
    let g = GrammarBuilder::new("cgex03")
        .token("a", "a")
        .token("ws", "\\s+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("TSLanguage"));
}

#[test]
fn cg_extras_valid_json() {
    let g = GrammarBuilder::new("cgex04")
        .token("a", "a")
        .token("ws", "\\s+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    let r = build_default(g);
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

// ═════════════════════════════════════════════════════════════════════════
// 16. cg_inline — Grammar with inline rules
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_inline_rule_builds() {
    let g = GrammarBuilder::new("cgin01")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .inline("inner")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_inline_rule_valid_stats() {
    let g = GrammarBuilder::new("cgin02")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .inline("inner")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_inline_rule_has_ffi() {
    let g = GrammarBuilder::new("cgin03")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .inline("inner")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("tree_sitter_cgin03"));
}

#[test]
fn cg_inline_rule_valid_json() {
    let g = GrammarBuilder::new("cgin04")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .inline("inner")
        .start("start")
        .build();
    let r = build_default(g);
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

// ═════════════════════════════════════════════════════════════════════════
// 17. cg_supertype — Grammar with supertypes
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_supertype_builds() {
    let g = GrammarBuilder::new("cgst01")
        .token("a", "a")
        .token("b", "b")
        .rule("variant_a", vec!["a"])
        .rule("variant_b", vec!["b"])
        .rule("expr", vec!["variant_a"])
        .rule("expr", vec!["variant_b"])
        .rule("start", vec!["expr"])
        .supertype("expr")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_supertype_valid_stats() {
    let g = GrammarBuilder::new("cgst02")
        .token("a", "a")
        .token("b", "b")
        .rule("variant_a", vec!["a"])
        .rule("variant_b", vec!["b"])
        .rule("expr", vec!["variant_a"])
        .rule("expr", vec!["variant_b"])
        .rule("start", vec!["expr"])
        .supertype("expr")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cg_supertype_has_ffi() {
    let g = GrammarBuilder::new("cgst03")
        .token("a", "a")
        .token("b", "b")
        .rule("variant_a", vec!["a"])
        .rule("variant_b", vec!["b"])
        .rule("expr", vec!["variant_a"])
        .rule("expr", vec!["variant_b"])
        .rule("start", vec!["expr"])
        .supertype("expr")
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("tree_sitter_cgst03"));
}

#[test]
fn cg_supertype_valid_json() {
    let g = GrammarBuilder::new("cgst04")
        .token("a", "a")
        .token("b", "b")
        .rule("variant_a", vec!["a"])
        .rule("variant_b", vec!["b"])
        .rule("expr", vec!["variant_a"])
        .rule("expr", vec!["variant_b"])
        .rule("start", vec!["expr"])
        .supertype("expr")
        .start("start")
        .build();
    let r = build_default(g);
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
}

// ═════════════════════════════════════════════════════════════════════════
// 18. cg_compress — Compress vs non-compress tables
// ═════════════════════════════════════════════════════════════════════════

fn build_with_compress(grammar: Grammar, compress: bool) -> BuildResult {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: compress,
    };
    build_parser(grammar, opts).expect("build")
}

#[test]
fn cg_compress_enabled_succeeds() {
    let r = build_with_compress(make_grammar("cgcm01"), true);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_compress_disabled_succeeds() {
    let r = build_with_compress(make_grammar("cgcm02"), false);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_compress_both_have_language() {
    let r_on = build_with_compress(make_grammar("cgcm03a"), true);
    let r_off = build_with_compress(make_grammar("cgcm03b"), false);
    assert!(r_on.parser_code.contains("TSLanguage"));
    assert!(r_off.parser_code.contains("TSLanguage"));
}

#[test]
fn cg_compress_both_have_valid_stats() {
    let r_on = build_with_compress(make_grammar("cgcm04a"), true);
    let r_off = build_with_compress(make_grammar("cgcm04b"), false);
    assert!(r_on.build_stats.state_count > 0);
    assert!(r_off.build_stats.state_count > 0);
}

#[test]
fn cg_compress_both_have_ffi() {
    let r_on = build_with_compress(make_grammar("cgcm05a"), true);
    let r_off = build_with_compress(make_grammar("cgcm05b"), false);
    assert!(r_on.parser_code.contains("tree_sitter_"));
    assert!(r_off.parser_code.contains("tree_sitter_"));
}

#[test]
fn cg_compress_tables_may_differ() {
    let r_on = build_with_compress(make_two_token_grammar("cgcm06a"), true);
    let r_off = build_with_compress(make_two_token_grammar("cgcm06b"), false);
    // Both succeed; code may or may not differ depending on implementation
    assert!(!r_on.parser_code.is_empty());
    assert!(!r_off.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════��═══════════
// 19. cg_emit — Emit artifacts option
// ═════════════════════════════════════════════════════════════════════════

fn build_with_emit(grammar: Grammar, emit: bool) -> (TempDir, BuildResult) {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: emit,
        compress_tables: true,
    };
    let result = build_parser(grammar, opts).expect("build");
    (dir, result)
}

#[test]
fn cg_emit_true_succeeds() {
    let (_dir, r) = build_with_emit(make_grammar("cgem01"), true);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_emit_false_succeeds() {
    let (_dir, r) = build_with_emit(make_grammar("cgem02"), false);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_emit_true_valid_stats() {
    let (_dir, r) = build_with_emit(make_grammar("cgem03"), true);
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn cg_emit_false_valid_stats() {
    let (_dir, r) = build_with_emit(make_grammar("cgem04"), false);
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 20. cg_complexity — Various grammar complexities
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_complexity_one_rule() {
    let r = build_default(make_grammar("cgcx01"));
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_complexity_two_rules() {
    let r = build_default(make_multi_rule_grammar("cgcx02", 2));
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_complexity_five_rules() {
    let r = build_default(make_multi_rule_grammar("cgcx03", 5));
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_complexity_ten_rules() {
    let r = build_default(make_multi_rule_grammar("cgcx04", 10));
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn cg_complexity_states_grow() {
    let r1 = build_default(make_grammar("cgcx05a"));
    let r5 = build_default(make_multi_rule_grammar("cgcx05b", 5));
    let r10 = build_default(make_multi_rule_grammar("cgcx05c", 10));
    assert!(r5.build_stats.state_count >= r1.build_stats.state_count);
    assert!(r10.build_stats.state_count >= r5.build_stats.state_count);
}

#[test]
fn cg_complexity_symbols_grow() {
    let r1 = build_default(make_grammar("cgcx06a"));
    let r5 = build_default(make_multi_rule_grammar("cgcx06b", 5));
    let r10 = build_default(make_multi_rule_grammar("cgcx06c", 10));
    assert!(r5.build_stats.symbol_count >= r1.build_stats.symbol_count);
    assert!(r10.build_stats.symbol_count >= r5.build_stats.symbol_count);
}

#[test]
fn cg_complexity_all_have_language() {
    for (name, count) in [("cgcx07a", 1), ("cgcx07b", 5), ("cgcx07c", 10)] {
        let g = if count == 1 {
            make_grammar(name)
        } else {
            make_multi_rule_grammar(name, count)
        };
        let r = build_default(g);
        assert!(r.parser_code.contains("TSLanguage"), "failed for {name}");
    }
}

#[test]
fn cg_complexity_all_have_valid_json() {
    for (name, count) in [("cgcx08a", 1), ("cgcx08b", 5), ("cgcx08c", 10)] {
        let g = if count == 1 {
            make_grammar(name)
        } else {
            make_multi_rule_grammar(name, count)
        };
        let r = build_default(g);
        let _: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid JSON");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// Additional coverage: cross-cutting patterns
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cg_cross_lex_modes_present() {
    let r = build_default(make_grammar("cgxc01"));
    assert!(r.parser_code.contains("LEX_MODES"));
}

#[test]
fn cg_cross_public_symbol_map() {
    let r = build_default(make_grammar("cgxc02"));
    assert!(r.parser_code.contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn cg_cross_primary_state_ids() {
    let r = build_default(make_grammar("cgxc03"));
    assert!(r.parser_code.contains("PRIMARY_STATE_IDS"));
}

#[test]
fn cg_cross_ts_rules_present() {
    let r = build_default(make_grammar("cgxc04"));
    assert!(r.parser_code.contains("TS_RULES"));
}

#[test]
fn cg_cross_field_map_present() {
    let r = build_default(make_grammar("cgxc05"));
    assert!(r.parser_code.contains("FIELD_MAP"));
}

#[test]
fn cg_cross_production_id_map() {
    let r = build_default(make_grammar("cgxc06"));
    assert!(r.parser_code.contains("PRODUCTION_ID_MAP"));
}

#[test]
fn cg_cross_parser_code_nonempty() {
    let r = build_default(make_grammar("cgxc07"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn cg_cross_parser_path_nonempty() {
    let r = build_default(make_grammar("cgxc08"));
    assert!(!r.parser_path.is_empty());
}

#[test]
fn cg_cross_conflict_cells_non_negative() {
    let r = build_default(make_grammar("cgxc09"));
    // conflict_cells is usize, always >= 0; just ensure accessible
    let _ = r.build_stats.conflict_cells;
}

#[test]
fn cg_cross_recursive_grammar() {
    let g = GrammarBuilder::new("cgxc10")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_default(g);
    assert!(r.parser_code.contains("TSLanguage"));
    assert!(r.build_stats.state_count > 0);
}
