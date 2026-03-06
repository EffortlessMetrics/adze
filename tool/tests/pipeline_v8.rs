//! Pipeline v8 integration tests for adze-tool.
//!
//! 80+ tests covering:
//!   1. End-to-end: simple grammar → build → parser_code non-empty (10 tests)
//!   2. End-to-end: grammar with tokens + rules → build → check stats (8 tests)
//!   3. Parser code contains expected patterns (8 tests)
//!   4. node_types_json is valid JSON (8 tests)
//!   5. Build with compress_tables=true vs false (6 tests)
//!   6. Build with emit_artifacts=true vs false (6 tests)
//!   7. Grammar with precedence rules (4 tests)
//!   8. Grammar with inline rules (3 tests)
//!   9. Grammar with supertypes (3 tests)
//!  10. Grammar with extras (3 tests)
//!  11. Grammar with externals (3 tests)
//!  12. Grammar with conflicts (2 tests)
//!  13. Various grammar sizes (5 tests)
//!  14. Build stats: state_count > 0 always (3 tests)
//!  15. Build stats: symbol_count > 0 always (3 tests)
//!  16. Build stats: conflict_cells (2 tests)
//!  17. Multiple builds of same grammar produce same stats (4 tests)
//!  18. Different grammars produce different code (4 tests)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn build_with_opts(grammar: Grammar, emit: bool, compress: bool) -> BuildResult {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: emit,
        compress_tables: compress,
    };
    build_parser(grammar, opts).expect("build should succeed")
}

fn build_default(grammar: Grammar) -> BuildResult {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        ..BuildOptions::default()
    };
    build_parser(grammar, opts).expect("build")
}

fn tmp_opts_with_artifacts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    (dir, opts)
}

// ── Grammar factories ────────────────────────────────────────────────────

fn single_token_v8() -> Grammar {
    GrammarBuilder::new("single_v8")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_alt_v8() -> Grammar {
    GrammarBuilder::new("twoalt_v8")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn chain_v8() -> Grammar {
    GrammarBuilder::new("chain_v8")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build()
}

fn seq_v8() -> Grammar {
    GrammarBuilder::new("seq_v8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

fn regex_v8() -> Grammar {
    GrammarBuilder::new("regex_v8")
        .token("num", r"\d+")
        .rule("start", vec!["num"])
        .start("start")
        .build()
}

fn arithmetic_v8() -> Grammar {
    GrammarBuilder::new("arith_v8")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn many_tokens_v8(count: usize) -> Grammar {
    let mut builder = GrammarBuilder::new("many_v8");
    let names: Vec<String> = (0..count).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("start", vec![name]);
    }
    builder = builder.start("start");
    builder.build()
}

fn deep_chain_v8(depth: usize) -> Grammar {
    let mut builder = GrammarBuilder::new("deep_v8");
    builder = builder.token("leaf", "leaf");
    let names: Vec<String> = (0..depth).map(|i| format!("n{i}")).collect();
    builder = builder.rule(&names[0], vec!["leaf"]);
    for i in 1..depth {
        builder = builder.rule(&names[i], vec![&names[i - 1]]);
    }
    builder = builder.start(&names[depth - 1]);
    builder.build()
}

fn prec_right_v8() -> Grammar {
    GrammarBuilder::new("precright_v8")
        .token("a", "a")
        .token("op", "^")
        .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["a"])
        .start("expr")
        .build()
}

fn prec_none_v8() -> Grammar {
    GrammarBuilder::new("precnone_v8")
        .token("a", "a")
        .token("eq", "=")
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::None)
        .rule("expr", vec!["a"])
        .start("expr")
        .build()
}

fn inline_v8() -> Grammar {
    GrammarBuilder::new("inline_v8")
        .token("x", "x")
        .token("y", "y")
        .rule("helper", vec!["x"])
        .rule("helper", vec!["y"])
        .rule("start", vec!["helper"])
        .inline("helper")
        .start("start")
        .build()
}

fn supertype_v8() -> Grammar {
    GrammarBuilder::new("supertype_v8")
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

fn extras_v8() -> Grammar {
    GrammarBuilder::new("extras_v8")
        .token("word", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .rule("start", vec!["word"])
        .extra("ws")
        .start("start")
        .build()
}

fn externals_v8() -> Grammar {
    GrammarBuilder::new("externals_v8")
        .token("id", r"[a-z]+")
        .token("indent", "INDENT")
        .token("dedent", "DEDENT")
        .external("indent")
        .external("dedent")
        .rule("start", vec!["id"])
        .start("start")
        .build()
}

fn five_rules_v8() -> Grammar {
    GrammarBuilder::new("fiverules_v8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("r3", vec!["c"])
        .rule("r4", vec!["d"])
        .rule("r5", vec!["e"])
        .rule("start", vec!["r1"])
        .rule("start", vec!["r2"])
        .rule("start", vec!["r3"])
        .rule("start", vec!["r4"])
        .rule("start", vec!["r5"])
        .start("start")
        .build()
}

fn twenty_rules_v8() -> Grammar {
    let mut builder = GrammarBuilder::new("twenty_v8");
    for i in 0..20 {
        let tok = format!("tok{i}");
        let rule = format!("rule{i}");
        builder = builder.token(&tok, &tok);
        builder = builder.rule(&rule, vec![&tok]);
        builder = builder.rule("start", vec![&rule]);
    }
    builder = builder.start("start");
    builder.build()
}

fn multi_prec_v8() -> Grammar {
    GrammarBuilder::new("multiprec_v8")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .token("minus", "-")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn mixed_assoc_v8() -> Grammar {
    GrammarBuilder::new("mixedassoc_v8")
        .token("a", "a")
        .token("plus", "+")
        .token("caret", "^")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            2,
            Associativity::Right,
        )
        .rule("expr", vec!["a"])
        .start("expr")
        .build()
}

fn ops_v8() -> Grammar {
    GrammarBuilder::new("ops_v8")
        .token("plus", "+")
        .token("minus", "-")
        .token("n", "0")
        .rule("start", vec!["n", "plus", "n"])
        .rule("start", vec!["n", "minus", "n"])
        .start("start")
        .build()
}

// =========================================================================
// 1. End-to-end: simple grammar → build → parser_code non-empty (10 tests)
// =========================================================================

#[test]
fn v8_e2e_single_token_produces_parser_code() {
    let result = build_default(single_token_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_e2e_single_token_grammar_name() {
    let result = build_default(single_token_v8());
    assert_eq!(result.grammar_name, "single_v8");
}

#[test]
fn v8_e2e_two_alt_produces_parser_code() {
    let result = build_default(two_alt_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_e2e_chain_produces_parser_code() {
    let result = build_default(chain_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_e2e_seq_produces_parser_code() {
    let result = build_default(seq_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_e2e_regex_token_produces_parser_code() {
    let result = build_default(regex_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_e2e_arithmetic_produces_parser_code() {
    let result = build_default(arithmetic_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_e2e_ops_produces_parser_code() {
    let result = build_default(ops_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_e2e_parser_path_nonempty() {
    let result = build_default(single_token_v8());
    assert!(!result.parser_path.is_empty());
}

#[test]
fn v8_e2e_parser_path_contains_grammar_name() {
    let result = build_default(single_token_v8());
    assert!(
        result.parser_path.contains("single_v8"),
        "parser_path should contain grammar name, got: {}",
        result.parser_path,
    );
}

// =========================================================================
// 2. End-to-end: grammar with tokens + rules → build → check stats (8 tests)
// =========================================================================

#[test]
fn v8_stats_single_token_state_count() {
    let result = build_default(single_token_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_stats_single_token_symbol_count() {
    let result = build_default(single_token_v8());
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v8_stats_two_alt_symbol_count_at_least_two() {
    let result = build_default(two_alt_v8());
    assert!(result.build_stats.symbol_count >= 2);
}

#[test]
fn v8_stats_chain_state_count_positive() {
    let result = build_default(chain_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_stats_seq_state_count_positive() {
    let result = build_default(seq_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_stats_regex_symbol_count_positive() {
    let result = build_default(regex_v8());
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v8_stats_arithmetic_state_and_symbol_positive() {
    let result = build_default(arithmetic_v8());
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v8_stats_ops_state_and_symbol_positive() {
    let result = build_default(ops_v8());
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 3. Parser code contains expected patterns (8 tests)
// =========================================================================

#[test]
fn v8_code_contains_language_struct() {
    let result = build_default(single_token_v8());
    assert!(
        result.parser_code.contains("LANGUAGE")
            || result.parser_code.contains("Language")
            || result.parser_code.contains("language"),
        "parser_code should reference Language",
    );
}

#[test]
fn v8_code_contains_symbol_names() {
    let result = build_default(single_token_v8());
    assert!(
        result.parser_code.contains("SYMBOL_NAMES") || result.parser_code.contains("symbol_names"),
        "parser_code should contain symbol names",
    );
}

#[test]
fn v8_code_contains_parse_table_ref() {
    let result = build_default(single_token_v8());
    assert!(
        result.parser_code.contains("PARSE_TABLE")
            || result.parser_code.contains("parse_table")
            || result.parser_code.contains("action_table"),
        "parser_code should contain parse table data",
    );
}

#[test]
fn v8_code_contains_tree_sitter_fn() {
    let result = build_default(single_token_v8());
    assert!(
        result.parser_code.contains("tree_sitter_single_v8")
            || result.parser_code.contains("language"),
        "parser_code should contain the FFI entry point",
    );
}

#[test]
fn v8_code_contains_symbol_metadata() {
    let result = build_default(single_token_v8());
    assert!(
        result.parser_code.contains("SYMBOL_METADATA")
            || result.parser_code.contains("symbol_metadata"),
        "parser_code should contain symbol metadata",
    );
}

#[test]
fn v8_code_contains_lex_modes() {
    let result = build_default(single_token_v8());
    assert!(
        result.parser_code.contains("LEX_MODES")
            || result.parser_code.contains("lex_modes")
            || result.parser_code.contains("lex_fn"),
        "parser_code should contain lex mode data",
    );
}

#[test]
fn v8_code_two_alt_contains_grammar_name() {
    let result = build_default(two_alt_v8());
    assert!(
        result.parser_code.contains("twoalt_v8"),
        "parser_code should embed grammar name",
    );
}

#[test]
fn v8_code_arithmetic_contains_grammar_name() {
    let result = build_default(arithmetic_v8());
    assert!(
        result.parser_code.contains("arith_v8"),
        "parser_code should embed grammar name",
    );
}

// =========================================================================
// 4. node_types_json is valid JSON (8 tests)
// =========================================================================

#[test]
fn v8_json_single_token_is_valid() {
    let result = build_default(single_token_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json)
        .expect("node_types_json should be valid JSON");
    assert!(val.is_array());
}

#[test]
fn v8_json_two_alt_is_valid_array() {
    let result = build_default(two_alt_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn v8_json_chain_is_valid_array() {
    let result = build_default(chain_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn v8_json_seq_is_valid_array() {
    let result = build_default(seq_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn v8_json_entries_are_objects() {
    let result = build_default(single_token_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(entry.is_object(), "each node_types entry must be an object");
    }
}

#[test]
fn v8_json_entries_have_type_field() {
    let result = build_default(single_token_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "each node_types entry should have a 'type' field",
        );
    }
}

#[test]
fn v8_json_arithmetic_is_valid() {
    let result = build_default(arithmetic_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn v8_json_node_types_nonempty() {
    let result = build_default(single_token_v8());
    assert!(!result.node_types_json.is_empty());
}

// =========================================================================
// 5. Build with compress_tables=true vs false (6 tests)
// =========================================================================

#[test]
fn v8_compress_true_produces_parser_code() {
    let result = build_with_opts(single_token_v8(), false, true);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_compress_false_produces_parser_code() {
    let result = build_with_opts(single_token_v8(), false, false);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_compress_stats_same_state_count() {
    let r_comp = build_with_opts(single_token_v8(), false, true);
    let r_nocomp = build_with_opts(single_token_v8(), false, false);
    assert_eq!(
        r_comp.build_stats.state_count,
        r_nocomp.build_stats.state_count,
    );
}

#[test]
fn v8_compress_stats_same_symbol_count() {
    let r_comp = build_with_opts(single_token_v8(), false, true);
    let r_nocomp = build_with_opts(single_token_v8(), false, false);
    assert_eq!(
        r_comp.build_stats.symbol_count,
        r_nocomp.build_stats.symbol_count,
    );
}

#[test]
fn v8_compress_two_alt_stats_consistent() {
    let r_comp = build_with_opts(two_alt_v8(), false, true);
    let r_nocomp = build_with_opts(two_alt_v8(), false, false);
    assert_eq!(
        r_comp.build_stats.state_count,
        r_nocomp.build_stats.state_count,
    );
    assert_eq!(
        r_comp.build_stats.symbol_count,
        r_nocomp.build_stats.symbol_count,
    );
}

#[test]
fn v8_compress_both_produce_valid_node_types() {
    let r_comp = build_with_opts(single_token_v8(), false, true);
    let r_nocomp = build_with_opts(single_token_v8(), false, false);
    let v1: serde_json::Value = serde_json::from_str(&r_comp.node_types_json).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&r_nocomp.node_types_json).unwrap();
    assert!(v1.is_array());
    assert!(v2.is_array());
}

// =========================================================================
// 6. Build with emit_artifacts=true vs false (6 tests)
// =========================================================================

#[test]
fn v8_emit_false_produces_parser_code() {
    let result = build_with_opts(single_token_v8(), false, true);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_emit_true_produces_parser_code() {
    let result = build_with_opts(single_token_v8(), true, true);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_emit_true_writes_parser_file() {
    let (_dir, opts) = tmp_opts_with_artifacts();
    let result = build_parser(single_token_v8(), opts).unwrap();
    let parser_path = std::path::Path::new(&result.parser_path);
    assert!(parser_path.exists());
}

#[test]
fn v8_emit_true_parser_file_nonempty() {
    let (_dir, opts) = tmp_opts_with_artifacts();
    let result = build_parser(single_token_v8(), opts).unwrap();
    let content = std::fs::read_to_string(&result.parser_path).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn v8_emit_true_stats_match_false() {
    let r_emit = build_with_opts(single_token_v8(), true, true);
    let r_noemit = build_with_opts(single_token_v8(), false, true);
    assert_eq!(
        r_emit.build_stats.state_count,
        r_noemit.build_stats.state_count,
    );
    assert_eq!(
        r_emit.build_stats.symbol_count,
        r_noemit.build_stats.symbol_count,
    );
}

#[test]
fn v8_emit_true_node_types_same() {
    let r_emit = build_with_opts(single_token_v8(), true, true);
    let r_noemit = build_with_opts(single_token_v8(), false, true);
    assert_eq!(r_emit.node_types_json, r_noemit.node_types_json);
}

// =========================================================================
// 7. Grammar with precedence rules (4 tests)
// =========================================================================

#[test]
fn v8_prec_left_builds_successfully() {
    let result = build_default(arithmetic_v8());
    assert!(!result.parser_code.is_empty());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_prec_right_builds_successfully() {
    let result = build_default(prec_right_v8());
    assert!(!result.parser_code.is_empty());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_prec_none_builds_successfully() {
    let result = build_default(prec_none_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_prec_mixed_associativity_builds() {
    let result = build_default(mixed_assoc_v8());
    assert!(!result.parser_code.is_empty());
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 8. Grammar with inline rules (3 tests)
// =========================================================================

#[test]
fn v8_inline_rule_builds_successfully() {
    let result = build_default(inline_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_inline_rule_state_count_positive() {
    let result = build_default(inline_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_inline_rule_node_types_valid() {
    let result = build_default(inline_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

// =========================================================================
// 9. Grammar with supertypes (3 tests)
// =========================================================================

#[test]
fn v8_supertype_builds_successfully() {
    let result = build_default(supertype_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_supertype_state_count_positive() {
    let result = build_default(supertype_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_supertype_node_types_valid() {
    let result = build_default(supertype_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

// =========================================================================
// 10. Grammar with extras (3 tests)
// =========================================================================

#[test]
fn v8_extras_builds_successfully() {
    let result = build_default(extras_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_extras_state_count_positive() {
    let result = build_default(extras_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_extras_node_types_valid() {
    let result = build_default(extras_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

// =========================================================================
// 11. Grammar with externals (3 tests)
// =========================================================================

#[test]
fn v8_externals_builds_successfully() {
    let result = build_default(externals_v8());
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_externals_state_count_positive() {
    let result = build_default(externals_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_externals_node_types_valid() {
    let result = build_default(externals_v8());
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

// =========================================================================
// 12. Grammar with conflicts (2 tests)
// =========================================================================

#[test]
fn v8_conflict_grammar_ambiguous_builds() {
    // A grammar with shift/reduce conflict resolved by precedence
    let grammar = GrammarBuilder::new("conflict_v8")
        .token("id", r"[a-z]+")
        .token("plus", "+")
        .token("star", "*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["id"])
        .start("expr")
        .build();
    let result = build_default(grammar);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn v8_conflict_grammar_multi_prec_levels() {
    let result = build_default(multi_prec_v8());
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 13. Various grammar sizes (5 tests)
// =========================================================================

#[test]
fn v8_size_one_rule() {
    let result = build_default(single_token_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_size_five_rules() {
    let result = build_default(five_rules_v8());
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v8_size_twenty_rules() {
    let result = build_default(twenty_rules_v8());
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v8_size_many_tokens_15() {
    let result = build_default(many_tokens_v8(15));
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_size_deep_chain_8() {
    let result = build_default(deep_chain_v8(8));
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 14. Build stats: state_count > 0 always (3 tests)
// =========================================================================

#[test]
fn v8_state_count_positive_single() {
    let result = build_default(single_token_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_state_count_positive_complex() {
    let result = build_default(arithmetic_v8());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn v8_state_count_positive_deep() {
    let result = build_default(deep_chain_v8(6));
    assert!(result.build_stats.state_count > 0);
}

// =========================================================================
// 15. Build stats: symbol_count > 0 always (3 tests)
// =========================================================================

#[test]
fn v8_symbol_count_positive_single() {
    let result = build_default(single_token_v8());
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v8_symbol_count_positive_multi_alt() {
    let result = build_default(two_alt_v8());
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn v8_symbol_count_positive_twenty_rules() {
    let result = build_default(twenty_rules_v8());
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 16. Build stats: conflict_cells (2 tests)
// =========================================================================

#[test]
fn v8_conflict_cells_accessible() {
    let result = build_default(single_token_v8());
    // For an unambiguous grammar, conflict_cells may be 0
    let _ = result.build_stats.conflict_cells;
}

#[test]
fn v8_conflict_cells_non_negative() {
    // conflict_cells is usize, always >= 0; just verify it computes without panic
    let result = build_default(arithmetic_v8());
    let _cells = result.build_stats.conflict_cells;
}

// =========================================================================
// 17. Multiple builds of same grammar produce same stats (4 tests)
// =========================================================================

#[test]
fn v8_determinism_single_token_parser_code() {
    let r1 = build_default(single_token_v8());
    let r2 = build_default(single_token_v8());
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v8_determinism_single_token_stats() {
    let r1 = build_default(single_token_v8());
    let r2 = build_default(single_token_v8());
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells,);
}

#[test]
fn v8_determinism_two_alt_parser_code() {
    let r1 = build_default(two_alt_v8());
    let r2 = build_default(two_alt_v8());
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn v8_determinism_node_types_json() {
    let r1 = build_default(single_token_v8());
    let r2 = build_default(single_token_v8());
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// =========================================================================
// 18. Different grammars produce different code (4 tests)
// =========================================================================

#[test]
fn v8_different_grammars_different_code_single_vs_two() {
    let r1 = build_default(single_token_v8());
    let r2 = build_default(two_alt_v8());
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn v8_different_grammars_different_code_chain_vs_seq() {
    let r1 = build_default(chain_v8());
    let r2 = build_default(seq_v8());
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn v8_different_grammars_different_grammar_name() {
    let r1 = build_default(single_token_v8());
    let r2 = build_default(two_alt_v8());
    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn v8_different_grammars_different_stats() {
    let r_small = build_default(many_tokens_v8(3));
    let r_large = build_default(many_tokens_v8(12));
    assert!(r_large.build_stats.symbol_count > r_small.build_stats.symbol_count);
}

// =========================================================================
// Additional coverage: BuildOptions and BuildResult structural checks
// =========================================================================

#[test]
fn v8_opts_default_compress_enabled() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn v8_opts_default_emit_artifacts_disabled() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn v8_opts_default_out_dir_nonempty() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty());
}

#[test]
fn v8_opts_clone_preserves_values() {
    let opts = BuildOptions {
        out_dir: "/v8/test".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/v8/test");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

#[test]
fn v8_opts_debug_format_includes_all_fields() {
    let opts = BuildOptions {
        out_dir: "/v8/dbg".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let dbg = format!("{opts:?}");
    assert!(dbg.contains("out_dir"));
    assert!(dbg.contains("emit_artifacts"));
    assert!(dbg.contains("compress_tables"));
}

#[test]
fn v8_result_debug_format_includes_grammar_name() {
    let result = build_default(single_token_v8());
    let dbg = format!("{result:?}");
    assert!(dbg.contains("single_v8"));
}

#[test]
fn v8_stats_debug_format_includes_fields() {
    let result = build_default(single_token_v8());
    let dbg = format!("{:?}", result.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn v8_result_grammar_name_with_underscores() {
    let grammar = GrammarBuilder::new("my_parser_v8")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let result = build_default(grammar);
    assert_eq!(result.grammar_name, "my_parser_v8");
}

#[test]
fn v8_result_grammar_name_with_digits() {
    let grammar = GrammarBuilder::new("lang88")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let result = build_default(grammar);
    assert_eq!(result.grammar_name, "lang88");
}

#[test]
fn v8_two_alt_more_or_equal_states_than_single() {
    let r1 = build_default(single_token_v8());
    let r2 = build_default(two_alt_v8());
    assert!(r2.build_stats.state_count >= r1.build_stats.state_count);
}
