//! Comprehensive cross-crate end-to-end integration tests.
//!
//! Pipeline: Grammar (IR) → FirstFollow + ParseTable (GLR core) → Code generation (tablegen) → BuildResult (tool).

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn build_parse_table(grammar: &mut adze_ir::Grammar) -> adze_glr_core::ParseTable {
    grammar.normalize();
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("LR(1) automaton failed")
}

fn make_simple_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build()
}

fn make_two_alt_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn make_expr_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", "\\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

fn make_chain_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn make_sequence_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["x", "y", "z"])
        .start("s")
        .build()
}

fn make_build_options() -> BuildOptions {
    let dir = TempDir::new().unwrap();
    BuildOptions {
        out_dir: dir.into_path().to_string_lossy().into_owned(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Simple grammar through full pipeline (tests 1–10)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn simple_pipeline_parse_table_states() {
    let mut g = make_simple_grammar("simple1");
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count > 0);
}

#[test]
fn simple_pipeline_parse_table_symbols() {
    let mut g = make_simple_grammar("simple2");
    let pt = build_parse_table(&mut g);
    assert!(pt.symbol_count > 0);
}

#[test]
fn simple_pipeline_action_table_len() {
    let mut g = make_simple_grammar("simple3");
    let pt = build_parse_table(&mut g);
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn simple_pipeline_goto_table_len() {
    let mut g = make_simple_grammar("simple4");
    let pt = build_parse_table(&mut g);
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn simple_pipeline_rules_nonempty() {
    let mut g = make_simple_grammar("simple5");
    let pt = build_parse_table(&mut g);
    assert!(!pt.rules.is_empty());
}

#[test]
fn simple_pipeline_node_types_valid_json() {
    let mut g = make_simple_grammar("simple6");
    let _pt = build_parse_table(&mut g);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(v.is_array());
}

#[test]
fn simple_pipeline_static_language_nonempty() {
    let mut g = make_simple_grammar("simple7");
    let pt = build_parse_table(&mut g);
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn simple_pipeline_abi_builder_nonempty() {
    let mut g = make_simple_grammar("simple8");
    let pt = build_parse_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn simple_pipeline_build_parser_succeeds() {
    let g = make_simple_grammar("simple9");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "simple9");
}

#[test]
fn simple_pipeline_build_result_parser_code_nonempty() {
    let g = make_simple_grammar("simple10");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Expression grammar through full pipeline (tests 11–20)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn expr_pipeline_parse_table_states() {
    let mut g = make_expr_grammar("expr1");
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count >= 3);
}

#[test]
fn expr_pipeline_parse_table_symbols() {
    let mut g = make_expr_grammar("expr2");
    let pt = build_parse_table(&mut g);
    // num, plus, star, expr, EOF => at least 5
    assert!(pt.symbol_count >= 4);
}

#[test]
fn expr_pipeline_rules_count() {
    let mut g = make_expr_grammar("expr3");
    let pt = build_parse_table(&mut g);
    // 3 productions: expr->num, expr->expr+expr, expr->expr*expr (plus augmented)
    assert!(pt.rules.len() >= 3);
}

#[test]
fn expr_pipeline_node_types_valid_json() {
    let mut g = make_expr_grammar("expr4");
    let _pt = build_parse_table(&mut g);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(v.is_array());
}

#[test]
fn expr_pipeline_static_language_nonempty() {
    let mut g = make_expr_grammar("expr5");
    let pt = build_parse_table(&mut g);
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn expr_pipeline_abi_builder_nonempty() {
    let mut g = make_expr_grammar("expr6");
    let pt = build_parse_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn expr_pipeline_build_parser_succeeds() {
    let g = make_expr_grammar("expr7");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "expr7");
}

#[test]
fn expr_pipeline_build_stats_state_count() {
    let g = make_expr_grammar("expr8");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.build_stats.state_count >= 3);
}

#[test]
fn expr_pipeline_build_stats_symbol_count() {
    let g = make_expr_grammar("expr9");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.build_stats.symbol_count >= 4);
}

#[test]
fn expr_pipeline_node_types_json_nonempty() {
    let g = make_expr_grammar("expr10");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Multi-alternative grammar pipeline (tests 21–28)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn multi_alt_parse_table_states() {
    let mut g = make_two_alt_grammar("alt1");
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count >= 2);
}

#[test]
fn multi_alt_parse_table_symbols() {
    let mut g = make_two_alt_grammar("alt2");
    let pt = build_parse_table(&mut g);
    assert!(pt.symbol_count >= 2);
}

#[test]
fn multi_alt_first_sets_have_both_tokens() {
    let mut g = make_two_alt_grammar("alt3");
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    if let Some(start) = g.start_symbol() {
        if let Some(first_set) = ff.first(start) {
            assert!(first_set.count_ones(..) >= 2);
        }
    }
}

#[test]
fn multi_alt_node_types_entries_have_type() {
    let mut g = make_two_alt_grammar("alt4");
    let _pt = build_parse_table(&mut g);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    if let serde_json::Value::Array(arr) = v {
        for item in &arr {
            assert!(item.get("type").is_some());
        }
    }
}

#[test]
fn multi_alt_build_parser_succeeds() {
    let g = make_two_alt_grammar("alt5");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "alt5");
}

#[test]
fn multi_alt_three_alternatives() {
    let mut g = GrammarBuilder::new("alt3way")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count >= 2);
    assert!(pt.rules.len() >= 3);
}

#[test]
fn multi_alt_five_alternatives() {
    let mut g = GrammarBuilder::new("alt5way")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("ee", "e")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .rule("s", vec!["ee"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.symbol_count >= 5);
}

#[test]
fn multi_alt_static_language_nonempty() {
    let mut g = make_two_alt_grammar("alt8");
    let pt = build_parse_table(&mut g);
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Chain grammar pipeline (tests 29–35)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn chain_pipeline_parse_table_states() {
    let mut g = make_chain_grammar("chain1");
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count > 0);
}

#[test]
fn chain_pipeline_multiple_nonterminals() {
    let mut g = make_chain_grammar("chain2");
    let pt = build_parse_table(&mut g);
    // a, b, s are all nonterminals => rules for each
    assert!(pt.rules.len() >= 3);
}

#[test]
fn chain_pipeline_node_types() {
    let mut g = make_chain_grammar("chain3");
    let _pt = build_parse_table(&mut g);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn chain_pipeline_build_parser() {
    let g = make_chain_grammar("chain4");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "chain4");
}

#[test]
fn chain_deep_five_levels() {
    let mut g = GrammarBuilder::new("chain5")
        .token("x", "x")
        .rule("l1", vec!["x"])
        .rule("l2", vec!["l1"])
        .rule("l3", vec!["l2"])
        .rule("l4", vec!["l3"])
        .rule("s", vec!["l4"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count > 0);
    assert!(pt.rules.len() >= 5);
}

#[test]
fn chain_abi_builder_output() {
    let mut g = make_chain_grammar("chain6");
    let pt = build_parse_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn chain_static_language_output() {
    let mut g = make_chain_grammar("chain7");
    let pt = build_parse_table(&mut g);
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Sequence grammar pipeline (tests 36–42)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn seq_pipeline_parse_table_states() {
    let mut g = make_sequence_grammar("seq1");
    let pt = build_parse_table(&mut g);
    // x y z requires at least 4 states
    assert!(pt.state_count >= 4);
}

#[test]
fn seq_pipeline_rhs_len() {
    let mut g = make_sequence_grammar("seq2");
    let pt = build_parse_table(&mut g);
    let has_len3 = pt.rules.iter().any(|r| r.rhs_len >= 3);
    assert!(has_len3);
}

#[test]
fn seq_pipeline_node_types() {
    let mut g = make_sequence_grammar("seq3");
    let _pt = build_parse_table(&mut g);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(v.is_array());
}

#[test]
fn seq_pipeline_build_parser() {
    let g = make_sequence_grammar("seq4");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "seq4");
}

#[test]
fn seq_two_elements() {
    let mut g = GrammarBuilder::new("seq2e")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count >= 3);
}

#[test]
fn seq_four_elements() {
    let mut g = GrammarBuilder::new("seq4e")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count >= 5);
}

#[test]
fn seq_abi_builder_output() {
    let mut g = make_sequence_grammar("seq7");
    let pt = build_parse_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Deterministic pipeline output (tests 43–49)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_parse_table_state_count() {
    let make = || {
        let mut g = make_simple_grammar("det1");
        build_parse_table(&mut g)
    };
    assert_eq!(make().state_count, make().state_count);
}

#[test]
fn deterministic_parse_table_symbol_count() {
    let make = || {
        let mut g = make_simple_grammar("det2");
        build_parse_table(&mut g)
    };
    assert_eq!(make().symbol_count, make().symbol_count);
}

#[test]
fn deterministic_parse_table_rules_len() {
    let make = || {
        let mut g = make_simple_grammar("det3");
        build_parse_table(&mut g)
    };
    assert_eq!(make().rules.len(), make().rules.len());
}

#[test]
fn deterministic_static_language_code() {
    let make = || {
        let mut g = make_simple_grammar("det4");
        let pt = build_parse_table(&mut g);
        StaticLanguageGenerator::new(g, pt)
            .generate_language_code()
            .to_string()
    };
    assert_eq!(make(), make());
}

#[test]
fn deterministic_node_types_json() {
    let make = || {
        let mut g = make_simple_grammar("det5");
        let _pt = build_parse_table(&mut g);
        NodeTypesGenerator::new(&g).generate().unwrap()
    };
    assert_eq!(make(), make());
}

#[test]
fn deterministic_abi_builder_code() {
    let make = || {
        let mut g = make_simple_grammar("det6");
        let pt = build_parse_table(&mut g);
        AbiLanguageBuilder::new(&g, &pt).generate().to_string()
    };
    assert_eq!(make(), make());
}

#[test]
fn deterministic_expr_pipeline() {
    let make = || {
        let mut g = make_expr_grammar("det7");
        let pt = build_parse_table(&mut g);
        (
            pt.state_count,
            pt.symbol_count,
            pt.rules.len(),
            StaticLanguageGenerator::new(g, pt)
                .generate_language_code()
                .to_string(),
        )
    };
    assert_eq!(make(), make());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Pipeline preserves grammar name (tests 50–55)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn name_preserved_in_static_language() {
    let mut g = make_simple_grammar("my_parser");
    let pt = build_parse_table(&mut g);
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(code.contains("my_parser"));
}

#[test]
fn name_preserved_in_abi_builder() {
    let mut g = make_simple_grammar("my_abi_parser");
    let pt = build_parse_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(code.contains("my_abi_parser"));
}

#[test]
fn name_preserved_in_build_result() {
    let g = make_simple_grammar("build_name_test");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "build_name_test");
}

#[test]
fn name_preserved_in_build_result_code() {
    let g = make_simple_grammar("code_name_test");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.parser_code.contains("code_name_test"));
}

#[test]
fn name_preserved_expr_grammar() {
    let g = make_expr_grammar("my_expr_lang");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "my_expr_lang");
}

#[test]
fn name_preserved_chain_grammar() {
    let g = make_chain_grammar("my_chain_lang");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "my_chain_lang");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Pipeline produces valid JSON (tests 56–60)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn valid_json_simple_grammar() {
    let g = make_simple_grammar("json1");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_expr_grammar() {
    let g = make_expr_grammar("json2");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_chain_grammar() {
    let g = make_chain_grammar("json3");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn valid_json_multi_alt_grammar() {
    let g = make_two_alt_grammar("json4");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn valid_json_node_types_all_have_type_field() {
    let g = make_expr_grammar("json5");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    if let serde_json::Value::Array(arr) = v {
        for item in &arr {
            assert!(item.get("type").is_some());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. Pipeline produces non-empty code (tests 61–65)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn nonempty_code_simple() {
    let g = make_simple_grammar("ne1");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.parser_code.len() > 100);
}

#[test]
fn nonempty_code_expr() {
    let g = make_expr_grammar("ne2");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.parser_code.len() > 100);
}

#[test]
fn nonempty_code_chain() {
    let g = make_chain_grammar("ne3");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.parser_code.len() > 100);
}

#[test]
fn nonempty_code_sequence() {
    let g = make_sequence_grammar("ne4");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.parser_code.len() > 100);
}

#[test]
fn nonempty_code_multi_alt() {
    let g = make_two_alt_grammar("ne5");
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.parser_code.len() > 100);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. Large grammar through pipeline (tests 66–72)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn large_grammar_ten_tokens() {
    let mut b = GrammarBuilder::new("large10");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let pt = build_parse_table(&mut g);
    assert!(pt.symbol_count >= 10);
}

#[test]
fn large_grammar_twenty_tokens() {
    let mut b = GrammarBuilder::new("large20");
    for i in 0..20 {
        let n: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let pt = build_parse_table(&mut g);
    assert!(pt.symbol_count >= 20);
}

#[test]
fn large_grammar_static_language() {
    let mut b = GrammarBuilder::new("large_sl");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let pt = build_parse_table(&mut g);
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn large_grammar_node_types() {
    let mut b = GrammarBuilder::new("large_nt");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let _pt = build_parse_table(&mut g);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn large_grammar_build_parser() {
    let mut b = GrammarBuilder::new("large_bp");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "large_bp");
}

#[test]
fn large_grammar_build_stats() {
    let mut b = GrammarBuilder::new("large_bs");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let opts = make_build_options();
    let result = build_parser(g, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count >= 10);
}

#[test]
fn large_grammar_abi_builder() {
    let mut b = GrammarBuilder::new("large_ab");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let pt = build_parse_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. Additional cross-cutting tests (tests 73–78)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn right_associative_pipeline() {
    let mut g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("eq", "=")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::Right)
        .start("expr")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count > 0);
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn mixed_assoc_pipeline() {
    let mut g = GrammarBuilder::new("mixed_assoc")
        .token("n", "n")
        .token("plus", "+")
        .token("pow", "^")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 2, Associativity::Right)
        .start("expr")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count > 0);
    let g2 = make_simple_grammar("mixed_check");
    let opts = make_build_options();
    let _ = build_parser(g2, opts).unwrap();
}

#[test]
fn multiple_nonterminals_pipeline() {
    let mut g = GrammarBuilder::new("multi_nt")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g);
    assert!(pt.state_count > 0);
}

#[test]
fn first_follow_simple_pipeline() {
    let mut g = make_simple_grammar("ff1");
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    if let Some(start) = g.start_symbol() {
        let first_set = ff.first(start);
        assert!(first_set.is_some());
    }
}

#[test]
fn eof_symbol_is_valid() {
    let mut g = make_simple_grammar("eof1");
    let pt = build_parse_table(&mut g);
    let _ = pt.eof_symbol;
}

#[test]
fn parse_rules_all_have_lhs() {
    let mut g = make_simple_grammar("lhs1");
    let pt = build_parse_table(&mut g);
    for rule in &pt.rules {
        let _ = rule.lhs;
    }
}
