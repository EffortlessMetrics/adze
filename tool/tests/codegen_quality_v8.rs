//! Code generation quality tests for the `adze_tool::pure_rust_builder` pipeline.
//!
//! 80+ tests validating output quality across these dimensions:
//! - Basic output validity (non-empty, UTF-8, JSON structure)
//! - Determinism (repeated builds produce identical output)
//! - Scaling (output grows with grammar complexity)
//! - Content correctness (names, tokens, rules present in output)
//! - Statistics consistency (state_count, symbol_count, conflict_cells)
//! - Configuration variants (compress_tables, emit_artifacts)
//! - Grammar features (precedence, extras, associativity, multiple rules)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{build_parser, BuildOptions, BuildResult};
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

fn build_ok(grammar: Grammar) -> BuildResult {
    let (_dir, opts) = tmp_opts(true, false);
    build_parser(grammar, opts).expect("build_parser should succeed")
}

fn build_with(grammar: Grammar, compress: bool, emit: bool) -> BuildResult {
    let (_dir, opts) = tmp_opts(compress, emit);
    build_parser(grammar, opts).expect("build_parser should succeed")
}

// --- Grammar factories (each name unique with "cq_v8_" prefix) ---

fn gram_minimal() -> Grammar {
    GrammarBuilder::new("cq_v8_minimal")
        .token("NUMBER", r"\d+")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

fn gram_two_tokens() -> Grammar {
    GrammarBuilder::new("cq_v8_two_tokens")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("source_file", vec!["NUMBER"])
        .rule("source_file", vec!["IDENT"])
        .start("source_file")
        .build()
}

fn gram_arith() -> Grammar {
    GrammarBuilder::new("cq_v8_arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn gram_right_assoc() -> Grammar {
    GrammarBuilder::new("cq_v8_right_assoc")
        .token("NUMBER", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn gram_extras() -> Grammar {
    GrammarBuilder::new("cq_v8_extras")
        .token("NUMBER", r"\d+")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

fn gram_multi_rule() -> Grammar {
    GrammarBuilder::new("cq_v8_multi_rule")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("=", "=")
        .rule("source_file", vec!["statement"])
        .rule("statement", vec!["assignment"])
        .rule("statement", vec!["expression"])
        .rule("assignment", vec!["IDENT", "=", "expression"])
        .rule("expression", vec!["NUMBER"])
        .rule("expression", vec!["IDENT"])
        .start("source_file")
        .build()
}

fn gram_chain() -> Grammar {
    GrammarBuilder::new("cq_v8_chain")
        .token("A", "a")
        .rule("source_file", vec!["middle"])
        .rule("middle", vec!["leaf"])
        .rule("leaf", vec!["A"])
        .start("source_file")
        .build()
}

fn gram_wide() -> Grammar {
    GrammarBuilder::new("cq_v8_wide")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("source_file", vec!["A"])
        .rule("source_file", vec!["B"])
        .rule("source_file", vec!["C"])
        .rule("source_file", vec!["D"])
        .rule("source_file", vec!["E"])
        .start("source_file")
        .build()
}

fn gram_deep_prec() -> Grammar {
    GrammarBuilder::new("cq_v8_deep_prec")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn gram_single_token() -> Grammar {
    GrammarBuilder::new("cq_v8_single_tok")
        .token("X", "x")
        .rule("source_file", vec!["X"])
        .start("source_file")
        .build()
}

fn gram_keyword_heavy() -> Grammar {
    GrammarBuilder::new("cq_v8_keywords")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("THEN", "then")
        .token("END", "end")
        .token("IDENT", r"[a-z]+")
        .rule("source_file", vec!["if_stmt"])
        .rule("if_stmt", vec!["IF", "IDENT", "THEN", "IDENT", "END"])
        .rule("if_stmt", vec!["IF", "IDENT", "THEN", "IDENT", "ELSE", "IDENT", "END"])
        .start("source_file")
        .build()
}

fn gram_nested() -> Grammar {
    GrammarBuilder::new("cq_v8_nested")
        .token("(", "(")
        .token(")", ")")
        .token("ATOM", r"[a-z]+")
        .rule("source_file", vec!["sexp"])
        .rule("sexp", vec!["ATOM"])
        .rule("sexp", vec!["(", "sexp", ")"])
        .start("source_file")
        .build()
}

// ===========================================================================
// 1. Basic output validity
// ===========================================================================

#[test]
fn cq01_parser_code_non_empty() {
    let res = build_ok(gram_minimal());
    assert!(!res.parser_code.is_empty());
}

#[test]
fn cq02_parser_code_non_empty_arith() {
    let res = build_ok(gram_arith());
    assert!(!res.parser_code.is_empty());
}

#[test]
fn cq03_parser_code_non_empty_multi_rule() {
    let res = build_ok(gram_multi_rule());
    assert!(!res.parser_code.is_empty());
}

#[test]
fn cq04_node_types_non_empty() {
    let res = build_ok(gram_minimal());
    assert!(!res.node_types_json.is_empty());
}

#[test]
fn cq05_node_types_valid_json() {
    let res = build_ok(gram_minimal());
    let parsed: serde_json::Value =
        serde_json::from_str(&res.node_types_json).expect("node_types_json must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn cq06_node_types_valid_json_arith() {
    let res = build_ok(gram_arith());
    let parsed: serde_json::Value = serde_json::from_str(&res.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn cq07_node_types_valid_json_multi_rule() {
    let res = build_ok(gram_multi_rule());
    let parsed: serde_json::Value = serde_json::from_str(&res.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn cq08_node_types_array_has_entries() {
    let res = build_ok(gram_minimal());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    assert!(!arr.is_empty());
}

// ===========================================================================
// 2. Node-type structure: each entry has "type" and "named"
// ===========================================================================

fn assert_node_types_have_required_fields(json: &str) {
    let arr: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
    for (i, entry) in arr.iter().enumerate() {
        assert!(
            entry.get("type").is_some(),
            "node_types[{i}] missing \"type\" field"
        );
        assert!(
            entry.get("named").is_some(),
            "node_types[{i}] missing \"named\" field"
        );
    }
}

#[test]
fn cq09_node_type_fields_minimal() {
    let res = build_ok(gram_minimal());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq10_node_type_fields_arith() {
    let res = build_ok(gram_arith());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq11_node_type_fields_multi_rule() {
    let res = build_ok(gram_multi_rule());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq12_node_type_fields_chain() {
    let res = build_ok(gram_chain());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq13_node_type_fields_wide() {
    let res = build_ok(gram_wide());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq14_node_type_fields_extras() {
    let res = build_ok(gram_extras());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq15_node_type_fields_nested() {
    let res = build_ok(gram_nested());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq16_node_type_fields_keywords() {
    let res = build_ok(gram_keyword_heavy());
    assert_node_types_have_required_fields(&res.node_types_json);
}

// ===========================================================================
// 3. Determinism — repeated builds yield identical output
// ===========================================================================

#[test]
fn cq17_parser_code_deterministic() {
    let g1 = gram_minimal();
    let g2 = gram_minimal();
    let r1 = build_ok(g1);
    let r2 = build_ok(g2);
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn cq18_node_types_deterministic() {
    let r1 = build_ok(gram_minimal());
    let r2 = build_ok(gram_minimal());
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn cq19_parser_code_deterministic_arith() {
    let r1 = build_ok(gram_arith());
    let r2 = build_ok(gram_arith());
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn cq20_node_types_deterministic_arith() {
    let r1 = build_ok(gram_arith());
    let r2 = build_ok(gram_arith());
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn cq21_stats_deterministic() {
    let r1 = build_ok(gram_arith());
    let r2 = build_ok(gram_arith());
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn cq22_determinism_multi_rule() {
    let r1 = build_ok(gram_multi_rule());
    let r2 = build_ok(gram_multi_rule());
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ===========================================================================
// 4. Scaling — output grows with grammar complexity
// ===========================================================================

#[test]
fn cq23_parser_code_grows_with_grammar_size() {
    let small = build_ok(gram_minimal());
    let large = build_ok(gram_multi_rule());
    assert!(
        large.parser_code.len() > small.parser_code.len(),
        "multi-rule grammar should produce more code than minimal"
    );
}

#[test]
fn cq24_node_types_grow_with_grammar_size() {
    let small: Vec<serde_json::Value> =
        serde_json::from_str(&build_ok(gram_minimal()).node_types_json).unwrap();
    let large: Vec<serde_json::Value> =
        serde_json::from_str(&build_ok(gram_multi_rule()).node_types_json).unwrap();
    assert!(
        large.len() >= small.len(),
        "more rules should produce at least as many node types"
    );
}

#[test]
fn cq25_state_count_grows_with_complexity() {
    let simple = build_ok(gram_single_token());
    let complex = build_ok(gram_arith());
    assert!(
        complex.build_stats.state_count >= simple.build_stats.state_count,
        "arithmetic grammar needs at least as many states as single-token"
    );
}

#[test]
fn cq26_symbol_count_grows_with_tokens() {
    let few = build_ok(gram_minimal());
    let many = build_ok(gram_keyword_heavy());
    assert!(
        many.build_stats.symbol_count > few.build_stats.symbol_count,
        "keyword-heavy grammar should have more symbols"
    );
}

// ===========================================================================
// 5. Content correctness — names in output
// ===========================================================================

#[test]
fn cq27_grammar_name_in_parser_code() {
    let res = build_ok(gram_minimal());
    assert!(
        res.parser_code.contains("cq_v8_minimal"),
        "grammar name should appear in parser_code"
    );
}

#[test]
fn cq28_grammar_name_in_result() {
    let res = build_ok(gram_arith());
    assert_eq!(res.grammar_name, "cq_v8_arith");
}

#[test]
fn cq29_grammar_name_multi_rule() {
    let res = build_ok(gram_multi_rule());
    assert_eq!(res.grammar_name, "cq_v8_multi_rule");
}

fn node_type_names(json: &str) -> Vec<String> {
    let arr: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
    arr.iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()).map(String::from))
        .collect()
}

#[test]
fn cq30_rule_names_in_node_types() {
    let res = build_ok(gram_multi_rule());
    let names = node_type_names(&res.node_types_json);
    assert!(
        names.iter().any(|n| n == "source_file"),
        "source_file should appear in node_types"
    );
}

#[test]
fn cq31_statement_rule_in_node_types() {
    let res = build_ok(gram_multi_rule());
    let names = node_type_names(&res.node_types_json);
    assert!(
        names.iter().any(|n| n == "statement"),
        "statement should appear in node_types"
    );
}

#[test]
fn cq32_assignment_rule_in_node_types() {
    let res = build_ok(gram_multi_rule());
    let names = node_type_names(&res.node_types_json);
    assert!(
        names.iter().any(|n| n == "assignment"),
        "assignment should appear in node_types"
    );
}

#[test]
fn cq33_expression_rule_in_node_types() {
    let res = build_ok(gram_multi_rule());
    let names = node_type_names(&res.node_types_json);
    assert!(
        names.iter().any(|n| n == "expression"),
        "expression should appear in node_types"
    );
}

#[test]
fn cq34_token_pattern_in_parser_code() {
    let res = build_ok(gram_minimal());
    // The lexer function references digit matching from the NUMBER token pattern
    assert!(
        res.parser_code.contains("digit") || res.parser_code.contains("ascii"),
        "parser_code should reference digit/ascii logic from NUMBER token"
    );
}

#[test]
fn cq35_parser_code_contains_lexer_logic() {
    let res = build_ok(gram_two_tokens());
    // Two-token grammar should produce more lexer logic than single-token
    let single = build_ok(gram_single_token());
    assert!(
        res.parser_code.len() > single.parser_code.len(),
        "two-token grammar should produce more code than single-token"
    );
}

#[test]
fn cq36_expr_in_node_types_arith() {
    let res = build_ok(gram_arith());
    let names = node_type_names(&res.node_types_json);
    assert!(
        names.iter().any(|n| n == "expr"),
        "expr should appear in node_types for arith grammar"
    );
}

#[test]
fn cq37_chain_rule_names_in_node_types() {
    let res = build_ok(gram_chain());
    let names = node_type_names(&res.node_types_json);
    assert!(names.iter().any(|n| n == "middle"), "middle should be in node_types");
    assert!(names.iter().any(|n| n == "leaf"), "leaf should be in node_types");
}

#[test]
fn cq38_nested_rule_in_node_types() {
    let res = build_ok(gram_nested());
    let names = node_type_names(&res.node_types_json);
    assert!(names.iter().any(|n| n == "sexp"), "sexp should be in node_types");
}

// ===========================================================================
// 6. Build statistics consistency
// ===========================================================================

#[test]
fn cq39_state_count_positive_minimal() {
    let res = build_ok(gram_minimal());
    assert!(res.build_stats.state_count >= 2, "minimal grammar needs ≥2 states");
}

#[test]
fn cq40_state_count_positive_arith() {
    let res = build_ok(gram_arith());
    assert!(res.build_stats.state_count >= 2, "arith grammar needs ≥2 states");
}

#[test]
fn cq41_state_count_positive_multi_rule() {
    let res = build_ok(gram_multi_rule());
    assert!(res.build_stats.state_count >= 2);
}

#[test]
fn cq42_symbol_count_at_least_tokens_plus_nonterminals_minimal() {
    // minimal: 1 token (NUMBER) + 1 non-terminal (source_file) = 2
    let res = build_ok(gram_minimal());
    assert!(
        res.build_stats.symbol_count >= 2,
        "symbol_count should be >= tokens + non-terminals"
    );
}

#[test]
fn cq43_symbol_count_at_least_tokens_plus_nonterminals_arith() {
    // arith: 3 tokens (NUMBER, +, *) + 1 non-terminal (expr) = 4
    let res = build_ok(gram_arith());
    assert!(
        res.build_stats.symbol_count >= 4,
        "arith symbol_count should be >= 4"
    );
}

#[test]
fn cq44_symbol_count_at_least_tokens_plus_nonterminals_multi() {
    // multi_rule: 3 tokens + 4 non-terminals = 7
    let res = build_ok(gram_multi_rule());
    assert!(
        res.build_stats.symbol_count >= 7,
        "multi_rule symbol_count should be >= 7"
    );
}

#[test]
fn cq45_conflict_cells_zero_for_unambiguous() {
    let res = build_ok(gram_minimal());
    assert_eq!(
        res.build_stats.conflict_cells, 0,
        "unambiguous grammar should have zero conflict cells"
    );
}

#[test]
fn cq46_conflict_cells_zero_for_chain() {
    let res = build_ok(gram_chain());
    assert_eq!(res.build_stats.conflict_cells, 0);
}

#[test]
fn cq47_conflict_cells_zero_for_single_token() {
    let res = build_ok(gram_single_token());
    assert_eq!(res.build_stats.conflict_cells, 0);
}

#[test]
fn cq48_symbol_count_wide_grammar() {
    // wide: 5 tokens + 1 non-terminal = 6
    let res = build_ok(gram_wide());
    assert!(res.build_stats.symbol_count >= 6);
}

#[test]
fn cq49_symbol_count_keyword_heavy() {
    // keywords: 5 tokens + 2 non-terminals = 7
    let res = build_ok(gram_keyword_heavy());
    assert!(res.build_stats.symbol_count >= 7);
}

// ===========================================================================
// 7. Configuration variants — compress_tables and emit_artifacts
// ===========================================================================

#[test]
fn cq50_compress_true_valid_output() {
    let res = build_with(gram_minimal(), true, false);
    assert!(!res.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&res.node_types_json).unwrap();
}

#[test]
fn cq51_compress_false_valid_output() {
    let res = build_with(gram_minimal(), false, false);
    assert!(!res.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&res.node_types_json).unwrap();
}

#[test]
fn cq52_emit_artifacts_true_valid_output() {
    let res = build_with(gram_minimal(), true, true);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn cq53_emit_artifacts_false_valid_output() {
    let res = build_with(gram_minimal(), true, false);
    assert!(!res.parser_code.is_empty());
}

#[test]
fn cq54_compress_true_arith_valid() {
    let res = build_with(gram_arith(), true, false);
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq55_compress_false_arith_valid() {
    let res = build_with(gram_arith(), false, false);
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq56_compress_true_multi_rule_valid() {
    let res = build_with(gram_multi_rule(), true, false);
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq57_compress_false_multi_rule_valid() {
    let res = build_with(gram_multi_rule(), false, false);
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq58_compress_both_produce_same_json() {
    let r1 = build_with(gram_minimal(), true, false);
    let r2 = build_with(gram_minimal(), false, false);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn cq59_compress_both_same_grammar_name() {
    let r1 = build_with(gram_minimal(), true, false);
    let r2 = build_with(gram_minimal(), false, false);
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn cq60_compress_both_same_stats() {
    let r1 = build_with(gram_minimal(), true, false);
    let r2 = build_with(gram_minimal(), false, false);
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

// ===========================================================================
// 8. Precedence and associativity grammars
// ===========================================================================

#[test]
fn cq61_precedence_grammar_valid_output() {
    let res = build_ok(gram_arith());
    assert!(!res.parser_code.is_empty());
    let parsed: serde_json::Value = serde_json::from_str(&res.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn cq62_right_assoc_valid_output() {
    let res = build_ok(gram_right_assoc());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq63_right_assoc_grammar_name() {
    let res = build_ok(gram_right_assoc());
    assert_eq!(res.grammar_name, "cq_v8_right_assoc");
}

#[test]
fn cq64_deep_prec_valid_output() {
    let res = build_ok(gram_deep_prec());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq65_deep_prec_has_expr() {
    let res = build_ok(gram_deep_prec());
    let names = node_type_names(&res.node_types_json);
    assert!(names.iter().any(|n| n == "expr"));
}

#[test]
fn cq66_deep_prec_state_count_at_least_2() {
    let res = build_ok(gram_deep_prec());
    assert!(res.build_stats.state_count >= 2);
}

#[test]
fn cq67_deep_prec_symbol_count() {
    // 4 tokens (NUMBER, +, *, -) + 1 non-terminal (expr) = 5
    let res = build_ok(gram_deep_prec());
    assert!(res.build_stats.symbol_count >= 5);
}

// ===========================================================================
// 9. Extras grammar
// ===========================================================================

#[test]
fn cq68_extras_valid_output() {
    let res = build_ok(gram_extras());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq69_extras_grammar_name() {
    let res = build_ok(gram_extras());
    assert_eq!(res.grammar_name, "cq_v8_extras");
}

#[test]
fn cq70_extras_state_count() {
    let res = build_ok(gram_extras());
    assert!(res.build_stats.state_count >= 2);
}

#[test]
fn cq71_extras_symbol_count() {
    // 2 tokens + 1 non-terminal = 3
    let res = build_ok(gram_extras());
    assert!(res.build_stats.symbol_count >= 3);
}

// ===========================================================================
// 10. Nested / recursive grammar
// ===========================================================================

#[test]
fn cq72_nested_valid_output() {
    let res = build_ok(gram_nested());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq73_nested_grammar_name() {
    let res = build_ok(gram_nested());
    assert_eq!(res.grammar_name, "cq_v8_nested");
}

#[test]
fn cq74_nested_state_count() {
    let res = build_ok(gram_nested());
    assert!(res.build_stats.state_count >= 2);
}

#[test]
fn cq75_nested_deterministic() {
    let r1 = build_ok(gram_nested());
    let r2 = build_ok(gram_nested());
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ===========================================================================
// 11. Wide grammar (many alternatives)
// ===========================================================================

#[test]
fn cq76_wide_valid_output() {
    let res = build_ok(gram_wide());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq77_wide_grammar_name() {
    let res = build_ok(gram_wide());
    assert_eq!(res.grammar_name, "cq_v8_wide");
}

#[test]
fn cq78_wide_has_source_file_in_node_types() {
    let res = build_ok(gram_wide());
    let names = node_type_names(&res.node_types_json);
    assert!(names.iter().any(|n| n == "source_file"));
}

// ===========================================================================
// 12. Keyword-heavy grammar
// ===========================================================================

#[test]
fn cq79_keyword_heavy_valid_output() {
    let res = build_ok(gram_keyword_heavy());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq80_keyword_heavy_grammar_name() {
    let res = build_ok(gram_keyword_heavy());
    assert_eq!(res.grammar_name, "cq_v8_keywords");
}

#[test]
fn cq81_keyword_heavy_has_if_stmt() {
    let res = build_ok(gram_keyword_heavy());
    let names = node_type_names(&res.node_types_json);
    assert!(names.iter().any(|n| n == "if_stmt"));
}

#[test]
fn cq82_keyword_heavy_deterministic() {
    let r1 = build_ok(gram_keyword_heavy());
    let r2 = build_ok(gram_keyword_heavy());
    assert_eq!(r1.parser_code, r2.parser_code);
}

// ===========================================================================
// 13. Cross-grammar comparisons
// ===========================================================================

#[test]
fn cq83_different_grammars_different_code() {
    let r1 = build_ok(gram_minimal());
    let r2 = build_ok(gram_arith());
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn cq84_different_grammars_different_names() {
    let r1 = build_ok(gram_minimal());
    let r2 = build_ok(gram_arith());
    assert_ne!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn cq85_all_grammars_produce_valid_json() {
    let grammars: Vec<Grammar> = vec![
        gram_minimal(),
        gram_two_tokens(),
        gram_arith(),
        gram_right_assoc(),
        gram_extras(),
        gram_multi_rule(),
        gram_chain(),
        gram_wide(),
        gram_deep_prec(),
        gram_single_token(),
        gram_keyword_heavy(),
        gram_nested(),
    ];
    for g in grammars {
        let name = g.name.clone();
        let res = build_ok(g);
        let parsed: Result<Vec<serde_json::Value>, _> =
            serde_json::from_str(&res.node_types_json);
        assert!(parsed.is_ok(), "grammar {name} should produce valid JSON array");
    }
}

#[test]
fn cq86_all_grammars_have_node_type_fields() {
    let grammars: Vec<Grammar> = vec![
        gram_minimal(),
        gram_two_tokens(),
        gram_arith(),
        gram_right_assoc(),
        gram_extras(),
        gram_multi_rule(),
        gram_chain(),
        gram_wide(),
        gram_deep_prec(),
        gram_single_token(),
        gram_keyword_heavy(),
        gram_nested(),
    ];
    for g in grammars {
        let name = g.name.clone();
        let res = build_ok(g);
        let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json)
            .unwrap_or_else(|_| panic!("grammar {name}: invalid JSON"));
        for (i, entry) in arr.iter().enumerate() {
            assert!(
                entry.get("type").is_some(),
                "grammar {name}: node_types[{i}] missing \"type\""
            );
            assert!(
                entry.get("named").is_some(),
                "grammar {name}: node_types[{i}] missing \"named\""
            );
        }
    }
}

#[test]
fn cq87_all_grammars_state_count_positive() {
    let grammars: Vec<Grammar> = vec![
        gram_minimal(),
        gram_arith(),
        gram_multi_rule(),
        gram_chain(),
        gram_wide(),
        gram_nested(),
    ];
    for g in grammars {
        let name = g.name.clone();
        let res = build_ok(g);
        assert!(
            res.build_stats.state_count >= 2,
            "grammar {name}: state_count should be >= 2"
        );
    }
}

#[test]
fn cq88_all_grammars_symbol_count_positive() {
    let grammars: Vec<Grammar> = vec![
        gram_minimal(),
        gram_arith(),
        gram_multi_rule(),
        gram_chain(),
        gram_wide(),
        gram_nested(),
    ];
    for g in grammars {
        let name = g.name.clone();
        let res = build_ok(g);
        assert!(
            res.build_stats.symbol_count >= 2,
            "grammar {name}: symbol_count should be >= 2"
        );
    }
}

// ===========================================================================
// 14. Parser path and output directory
// ===========================================================================

#[test]
fn cq89_parser_path_non_empty() {
    let res = build_ok(gram_minimal());
    assert!(!res.parser_path.is_empty());
}

#[test]
fn cq90_parser_path_contains_grammar_name() {
    let res = build_ok(gram_minimal());
    assert!(
        res.parser_path.contains("cq_v8_minimal"),
        "parser_path should reference grammar name"
    );
}

// ===========================================================================
// 15. Named vs anonymous node types
// ===========================================================================

#[test]
fn cq91_node_types_have_named_booleans() {
    let res = build_ok(gram_multi_rule());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    for (i, entry) in arr.iter().enumerate() {
        let named = entry.get("named").unwrap();
        assert!(
            named.is_boolean(),
            "node_types[{i}] \"named\" should be boolean, got {named}"
        );
    }
}

#[test]
fn cq92_node_types_type_is_string() {
    let res = build_ok(gram_multi_rule());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    for (i, entry) in arr.iter().enumerate() {
        let ty = entry.get("type").unwrap();
        assert!(
            ty.is_string(),
            "node_types[{i}] \"type\" should be string, got {ty}"
        );
    }
}

#[test]
fn cq93_has_both_named_and_anonymous_types() {
    let res = build_ok(gram_arith());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&res.node_types_json).unwrap();
    let has_named = arr.iter().any(|e| e.get("named").and_then(|v| v.as_bool()) == Some(true));
    let has_anon = arr.iter().any(|e| e.get("named").and_then(|v| v.as_bool()) == Some(false));
    assert!(has_named, "arith grammar should have named node types");
    assert!(has_anon, "arith grammar should have anonymous node types");
}

// ===========================================================================
// 16. Additional edge-case and robustness tests
// ===========================================================================

#[test]
fn cq94_two_tokens_valid_output() {
    let res = build_ok(gram_two_tokens());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq95_two_tokens_grammar_name() {
    let res = build_ok(gram_two_tokens());
    assert_eq!(res.grammar_name, "cq_v8_two_tokens");
}

#[test]
fn cq96_single_token_valid_output() {
    let res = build_ok(gram_single_token());
    assert!(!res.parser_code.is_empty());
    assert_node_types_have_required_fields(&res.node_types_json);
}

#[test]
fn cq97_chain_grammar_name() {
    let res = build_ok(gram_chain());
    assert_eq!(res.grammar_name, "cq_v8_chain");
}

#[test]
fn cq98_chain_deterministic() {
    let r1 = build_ok(gram_chain());
    let r2 = build_ok(gram_chain());
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn cq99_parser_code_not_just_whitespace() {
    let res = build_ok(gram_minimal());
    let trimmed = res.parser_code.trim();
    assert!(!trimmed.is_empty(), "parser_code should not be just whitespace");
}

#[test]
fn cq100_node_types_json_not_just_whitespace() {
    let res = build_ok(gram_minimal());
    let trimmed = res.node_types_json.trim();
    assert!(!trimmed.is_empty(), "node_types_json should not be just whitespace");
}
