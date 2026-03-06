//! Comprehensive determinism tests for the adze-tool build pipeline.
//!
//! 80+ tests proving that `build_parser` is fully deterministic: identical
//! grammars with identical options always produce bit-for-bit identical output
//! across any number of repeated builds.
//!
//! Categories:
//!   1. Single-field determinism (parser_code, node_types_json, stats)
//!   2. Repetition counts (3×, 5×, 10×)
//!   3. Option variants (compress, emit_artifacts)
//!   4. Grammar complexity (minimal → large)
//!   5. Grammar features (extras, inline, extern, supertype, precedence, conflicts)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

fn tmp_opts(compress: bool, emit: bool) -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: emit,
        compress_tables: compress,
    };
    (dir, opts)
}

fn build_once(grammar: Grammar, compress: bool, emit: bool) -> BuildResult {
    let (_dir, opts) = tmp_opts(compress, emit);
    build_parser(grammar, opts).expect("build_parser should succeed")
}

fn build_n(make_grammar: impl Fn() -> Grammar, n: usize) -> Vec<BuildResult> {
    (0..n)
        .map(|_| build_once(make_grammar(), true, false))
        .collect()
}

fn assert_all_parser_code_equal(results: &[BuildResult]) {
    let first = &results[0].parser_code;
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(first, &r.parser_code, "parser_code diverged at build {i}");
    }
}

fn assert_all_node_types_equal(results: &[BuildResult]) {
    let first = &results[0].node_types_json;
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(
            first, &r.node_types_json,
            "node_types_json diverged at build {i}"
        );
    }
}

fn assert_all_stats_equal(results: &[BuildResult]) {
    let first = &results[0].build_stats;
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(
            first.state_count, r.build_stats.state_count,
            "state_count diverged at build {i}"
        );
        assert_eq!(
            first.symbol_count, r.build_stats.symbol_count,
            "symbol_count diverged at build {i}"
        );
        assert_eq!(
            first.conflict_cells, r.build_stats.conflict_cells,
            "conflict_cells diverged at build {i}"
        );
    }
}

fn assert_fully_deterministic(results: &[BuildResult]) {
    assert_all_parser_code_equal(results);
    assert_all_node_types_equal(results);
    assert_all_stats_equal(results);
}

// ---------------------------------------------------------------------------
// Grammar factories — each name is unique with "det_v8_" prefix
// ---------------------------------------------------------------------------

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_minimal")
        .token("NUMBER", r"\d+")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn prec_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_prec")
        .token("ID", r"[a-z]+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .precedence(2, Associativity::Left, vec!["*"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["ID"])
        .start("expr")
        .build()
}

fn extras_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_extras")
        .token("WORD", r"[a-z]+")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("source_file", vec!["WORD"])
        .start("source_file")
        .build()
}

fn inline_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_inline")
        .token("A", "a")
        .token("B", "b")
        .rule("source_file", vec!["helper"])
        .rule("helper", vec!["A", "B"])
        .inline("helper")
        .start("source_file")
        .build()
}

fn extern_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_extern")
        .token("TOK", "x")
        .external("INDENT")
        .rule("source_file", vec!["TOK"])
        .start("source_file")
        .build()
}

fn supertype_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_super")
        .token("NUM", r"\d+")
        .token("STR", r#""[^"]*""#)
        .rule("literal", vec!["NUM"])
        .rule("literal", vec!["STR"])
        .supertype("literal")
        .rule("source_file", vec!["literal"])
        .start("source_file")
        .build()
}

fn conflict_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_conflict")
        .token("ID", r"[a-z]+")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["call"])
        .rule("call", vec!["ID", "(", "args", ")"])
        .rule("args", vec!["expr"])
        .rule("args", vec!["args", ",", "expr"])
        .start("source_file")
        .build()
}

fn large_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_large")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .token(";", ";")
        .token("=", "=")
        .token(",", ",")
        .token("LET", "let")
        .rule("source_file", vec!["statement_list"])
        .rule("statement_list", vec!["statement"])
        .rule("statement_list", vec!["statement_list", ";", "statement"])
        .rule("statement", vec!["assignment"])
        .rule("statement", vec!["expr"])
        .rule("assignment", vec!["LET", "ID", "=", "expr"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["call"])
        .rule("call", vec!["ID", "(", "arg_list", ")"])
        .rule("arg_list", vec!["expr"])
        .rule("arg_list", vec!["arg_list", ",", "expr"])
        .start("source_file")
        .build()
}

fn two_rule_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_two")
        .token("A", "a")
        .token("B", "b")
        .rule("source_file", vec!["A"])
        .rule("source_file", vec!["B"])
        .start("source_file")
        .build()
}

fn right_assoc_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_rassoc")
        .token("ID", r"[a-z]+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["ID"])
        .start("expr")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_chain")
        .token("X", "x")
        .rule("source_file", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["X"])
        .start("source_file")
        .build()
}

fn multi_token_grammar() -> Grammar {
    GrammarBuilder::new("det_v8_mtok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("source_file", vec!["A", "B", "C", "D", "E"])
        .start("source_file")
        .build()
}

// ===========================================================================
// 1. Same grammar, same options → same parser_code
// ===========================================================================

#[test]
fn test_parser_code_deterministic_minimal() {
    let results = build_n(minimal_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_parser_code_deterministic_arith() {
    let results = build_n(arith_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_parser_code_deterministic_prec() {
    let results = build_n(prec_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_parser_code_deterministic_large() {
    let results = build_n(large_grammar, 2);
    assert_all_parser_code_equal(&results);
}

// ===========================================================================
// 2. Same grammar, same options → same node_types_json
// ===========================================================================

#[test]
fn test_node_types_deterministic_minimal() {
    let results = build_n(minimal_grammar, 2);
    assert_all_node_types_equal(&results);
}

#[test]
fn test_node_types_deterministic_arith() {
    let results = build_n(arith_grammar, 2);
    assert_all_node_types_equal(&results);
}

#[test]
fn test_node_types_deterministic_prec() {
    let results = build_n(prec_grammar, 2);
    assert_all_node_types_equal(&results);
}

#[test]
fn test_node_types_deterministic_large() {
    let results = build_n(large_grammar, 2);
    assert_all_node_types_equal(&results);
}

// ===========================================================================
// 3–5. Same grammar, same options → same state_count / symbol_count / conflict_cells
// ===========================================================================

#[test]
fn test_state_count_deterministic_minimal() {
    let results = build_n(minimal_grammar, 2);
    assert_eq!(
        results[0].build_stats.state_count,
        results[1].build_stats.state_count
    );
}

#[test]
fn test_state_count_deterministic_arith() {
    let results = build_n(arith_grammar, 2);
    assert_eq!(
        results[0].build_stats.state_count,
        results[1].build_stats.state_count
    );
}

#[test]
fn test_symbol_count_deterministic_minimal() {
    let results = build_n(minimal_grammar, 2);
    assert_eq!(
        results[0].build_stats.symbol_count,
        results[1].build_stats.symbol_count
    );
}

#[test]
fn test_symbol_count_deterministic_arith() {
    let results = build_n(arith_grammar, 2);
    assert_eq!(
        results[0].build_stats.symbol_count,
        results[1].build_stats.symbol_count
    );
}

#[test]
fn test_conflict_cells_deterministic_minimal() {
    let results = build_n(minimal_grammar, 2);
    assert_eq!(
        results[0].build_stats.conflict_cells,
        results[1].build_stats.conflict_cells
    );
}

#[test]
fn test_conflict_cells_deterministic_arith() {
    let results = build_n(arith_grammar, 2);
    assert_eq!(
        results[0].build_stats.conflict_cells,
        results[1].build_stats.conflict_cells
    );
}

#[test]
fn test_conflict_cells_deterministic_conflict_grammar() {
    let results = build_n(conflict_grammar, 2);
    assert_eq!(
        results[0].build_stats.conflict_cells,
        results[1].build_stats.conflict_cells
    );
}

// ===========================================================================
// 6. Build 3 times → all identical
// ===========================================================================

#[test]
fn test_3x_deterministic_minimal() {
    let results = build_n(minimal_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_3x_deterministic_arith() {
    let results = build_n(arith_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_3x_deterministic_large() {
    let results = build_n(large_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_3x_deterministic_prec() {
    let results = build_n(prec_grammar, 3);
    assert_fully_deterministic(&results);
}

// ===========================================================================
// 7. Build 5 times → all identical
// ===========================================================================

#[test]
fn test_5x_deterministic_minimal() {
    let results = build_n(minimal_grammar, 5);
    assert_fully_deterministic(&results);
}

#[test]
fn test_5x_deterministic_arith() {
    let results = build_n(arith_grammar, 5);
    assert_fully_deterministic(&results);
}

#[test]
fn test_5x_deterministic_large() {
    let results = build_n(large_grammar, 5);
    assert_fully_deterministic(&results);
}

#[test]
fn test_5x_deterministic_prec() {
    let results = build_n(prec_grammar, 5);
    assert_fully_deterministic(&results);
}

// ===========================================================================
// 8. Build 10 times → all identical
// ===========================================================================

#[test]
fn test_10x_deterministic_minimal() {
    let results = build_n(minimal_grammar, 10);
    assert_fully_deterministic(&results);
}

#[test]
fn test_10x_deterministic_arith() {
    let results = build_n(arith_grammar, 10);
    assert_fully_deterministic(&results);
}

// ===========================================================================
// 9. compress_tables=true → deterministic
// ===========================================================================

#[test]
fn test_compressed_deterministic_minimal() {
    let a = build_once(minimal_grammar(), true, false);
    let b = build_once(minimal_grammar(), true, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
    assert_eq!(a.build_stats.state_count, b.build_stats.state_count);
}

#[test]
fn test_compressed_deterministic_arith() {
    let a = build_once(arith_grammar(), true, false);
    let b = build_once(arith_grammar(), true, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

#[test]
fn test_compressed_deterministic_large() {
    let a = build_once(large_grammar(), true, false);
    let b = build_once(large_grammar(), true, false);
    assert_eq!(a.parser_code, b.parser_code);
}

#[test]
fn test_compressed_deterministic_prec() {
    let a = build_once(prec_grammar(), true, false);
    let b = build_once(prec_grammar(), true, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.build_stats.conflict_cells, b.build_stats.conflict_cells);
}

// ===========================================================================
// 10. compress_tables=false → deterministic
// ===========================================================================

#[test]
fn test_uncompressed_deterministic_minimal() {
    let a = build_once(minimal_grammar(), false, false);
    let b = build_once(minimal_grammar(), false, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
    assert_eq!(a.build_stats.state_count, b.build_stats.state_count);
}

#[test]
fn test_uncompressed_deterministic_arith() {
    let a = build_once(arith_grammar(), false, false);
    let b = build_once(arith_grammar(), false, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

#[test]
fn test_uncompressed_deterministic_large() {
    let a = build_once(large_grammar(), false, false);
    let b = build_once(large_grammar(), false, false);
    assert_eq!(a.parser_code, b.parser_code);
}

#[test]
fn test_uncompressed_deterministic_prec() {
    let a = build_once(prec_grammar(), false, false);
    let b = build_once(prec_grammar(), false, false);
    assert_eq!(a.parser_code, b.parser_code);
}

// ===========================================================================
// 11. emit_artifacts=true → deterministic
// ===========================================================================

#[test]
fn test_emit_artifacts_deterministic_minimal() {
    let a = build_once(minimal_grammar(), true, true);
    let b = build_once(minimal_grammar(), true, true);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

#[test]
fn test_emit_artifacts_deterministic_arith() {
    let a = build_once(arith_grammar(), true, true);
    let b = build_once(arith_grammar(), true, true);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

#[test]
fn test_emit_artifacts_deterministic_large() {
    let a = build_once(large_grammar(), true, true);
    let b = build_once(large_grammar(), true, true);
    assert_eq!(a.parser_code, b.parser_code);
}

#[test]
fn test_emit_artifacts_stats_deterministic() {
    let a = build_once(arith_grammar(), true, true);
    let b = build_once(arith_grammar(), true, true);
    assert_eq!(a.build_stats.state_count, b.build_stats.state_count);
    assert_eq!(a.build_stats.symbol_count, b.build_stats.symbol_count);
    assert_eq!(a.build_stats.conflict_cells, b.build_stats.conflict_cells);
}

// ===========================================================================
// 12. Minimal grammar → deterministic across 5 builds
// ===========================================================================

#[test]
fn test_minimal_5x_parser_code() {
    let results = build_n(minimal_grammar, 5);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_minimal_5x_node_types() {
    let results = build_n(minimal_grammar, 5);
    assert_all_node_types_equal(&results);
}

#[test]
fn test_minimal_5x_stats() {
    let results = build_n(minimal_grammar, 5);
    assert_all_stats_equal(&results);
}

// ===========================================================================
// 13. Arithmetic grammar → deterministic across 5 builds
// ===========================================================================

#[test]
fn test_arith_5x_parser_code() {
    let results = build_n(arith_grammar, 5);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_arith_5x_node_types() {
    let results = build_n(arith_grammar, 5);
    assert_all_node_types_equal(&results);
}

#[test]
fn test_arith_5x_stats() {
    let results = build_n(arith_grammar, 5);
    assert_all_stats_equal(&results);
}

// ===========================================================================
// 14. Precedence grammar → deterministic
// ===========================================================================

#[test]
fn test_prec_grammar_full_determinism() {
    let results = build_n(prec_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_prec_grammar_parser_code_stable() {
    let a = build_once(prec_grammar(), true, false);
    let b = build_once(prec_grammar(), true, false);
    assert_eq!(a.parser_code, b.parser_code);
}

#[test]
fn test_prec_grammar_stats_stable() {
    let a = build_once(prec_grammar(), true, false);
    let b = build_once(prec_grammar(), true, false);
    assert_eq!(a.build_stats.state_count, b.build_stats.state_count);
    assert_eq!(a.build_stats.symbol_count, b.build_stats.symbol_count);
}

// ===========================================================================
// 15. Grammar with extras → deterministic
// ===========================================================================

#[test]
fn test_extras_deterministic_full() {
    let results = build_n(extras_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_extras_deterministic_parser_code() {
    let results = build_n(extras_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_extras_deterministic_node_types() {
    let results = build_n(extras_grammar, 2);
    assert_all_node_types_equal(&results);
}

// ===========================================================================
// 16. Grammar with inline → deterministic
// ===========================================================================

#[test]
fn test_inline_deterministic_full() {
    let results = build_n(inline_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_inline_deterministic_parser_code() {
    let results = build_n(inline_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_inline_deterministic_stats() {
    let results = build_n(inline_grammar, 2);
    assert_all_stats_equal(&results);
}

// ===========================================================================
// 17. Grammar with extern → deterministic
// ===========================================================================

#[test]
fn test_extern_deterministic_full() {
    let results = build_n(extern_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_extern_deterministic_parser_code() {
    let results = build_n(extern_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_extern_deterministic_stats() {
    let results = build_n(extern_grammar, 2);
    assert_all_stats_equal(&results);
}

// ===========================================================================
// 18. Grammar with supertype → deterministic
// ===========================================================================

#[test]
fn test_supertype_deterministic_full() {
    let results = build_n(supertype_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_supertype_deterministic_parser_code() {
    let results = build_n(supertype_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_supertype_deterministic_node_types() {
    let results = build_n(supertype_grammar, 2);
    assert_all_node_types_equal(&results);
}

// ===========================================================================
// 19. Grammar with conflicts → deterministic
// ===========================================================================

#[test]
fn test_conflict_grammar_deterministic_full() {
    let results = build_n(conflict_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_conflict_grammar_deterministic_code() {
    let results = build_n(conflict_grammar, 2);
    assert_all_parser_code_equal(&results);
}

#[test]
fn test_conflict_grammar_deterministic_stats() {
    let results = build_n(conflict_grammar, 2);
    assert_all_stats_equal(&results);
}

// ===========================================================================
// 20. Large grammar (10+ rules) → deterministic
// ===========================================================================

#[test]
fn test_large_grammar_deterministic_full() {
    let results = build_n(large_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_large_grammar_deterministic_5x() {
    let results = build_n(large_grammar, 5);
    assert_fully_deterministic(&results);
}

#[test]
fn test_large_grammar_compressed_deterministic() {
    let a = build_once(large_grammar(), true, false);
    let b = build_once(large_grammar(), true, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
    assert_eq!(a.build_stats.state_count, b.build_stats.state_count);
    assert_eq!(a.build_stats.symbol_count, b.build_stats.symbol_count);
    assert_eq!(a.build_stats.conflict_cells, b.build_stats.conflict_cells);
}

// ===========================================================================
// Additional determinism — two-rule, right-assoc, chain, multi-token
// ===========================================================================

#[test]
fn test_two_rule_deterministic_full() {
    let results = build_n(two_rule_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_two_rule_deterministic_5x() {
    let results = build_n(two_rule_grammar, 5);
    assert_fully_deterministic(&results);
}

#[test]
fn test_right_assoc_deterministic_full() {
    let results = build_n(right_assoc_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_right_assoc_deterministic_5x() {
    let results = build_n(right_assoc_grammar, 5);
    assert_fully_deterministic(&results);
}

#[test]
fn test_chain_deterministic_full() {
    let results = build_n(chain_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_chain_deterministic_5x() {
    let results = build_n(chain_grammar, 5);
    assert_fully_deterministic(&results);
}

#[test]
fn test_multi_token_deterministic_full() {
    let results = build_n(multi_token_grammar, 3);
    assert_fully_deterministic(&results);
}

#[test]
fn test_multi_token_deterministic_5x() {
    let results = build_n(multi_token_grammar, 5);
    assert_fully_deterministic(&results);
}

// ===========================================================================
// Cross-option determinism: same grammar, same option → deterministic
// ===========================================================================

#[test]
fn test_cross_option_compressed_emit_deterministic() {
    let a = build_once(arith_grammar(), true, true);
    let b = build_once(arith_grammar(), true, true);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

#[test]
fn test_cross_option_uncompressed_emit_deterministic() {
    let a = build_once(arith_grammar(), false, true);
    let b = build_once(arith_grammar(), false, true);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

#[test]
fn test_cross_option_compressed_no_emit_deterministic() {
    let a = build_once(large_grammar(), true, false);
    let b = build_once(large_grammar(), true, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

#[test]
fn test_cross_option_uncompressed_no_emit_deterministic() {
    let a = build_once(large_grammar(), false, false);
    let b = build_once(large_grammar(), false, false);
    assert_eq!(a.parser_code, b.parser_code);
    assert_eq!(a.node_types_json, b.node_types_json);
}

// ===========================================================================
// Non-empty output sanity checks (preconditions for determinism to matter)
// ===========================================================================

#[test]
fn test_minimal_output_nonempty() {
    let r = build_once(minimal_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_arith_output_nonempty() {
    let r = build_once(arith_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_large_output_nonempty() {
    let r = build_once(large_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn test_extras_output_nonempty() {
    let r = build_once(extras_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_inline_output_nonempty() {
    let r = build_once(inline_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_extern_output_nonempty() {
    let r = build_once(extern_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_supertype_output_nonempty() {
    let r = build_once(supertype_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_conflict_output_nonempty() {
    let r = build_once(conflict_grammar(), true, false);
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}
