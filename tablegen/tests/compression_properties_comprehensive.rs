//! Comprehensive property tests for table compression behavior in adze-tablegen.
//!
//! Covers StaticLanguageGenerator construction, code generation, various grammar
//! sizes and shapes, determinism, ParseTable properties, NodeTypesGenerator
//! integration, and sequential builds.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{
    NodeTypesGenerator, StaticLanguageGenerator, TableCompressor, collect_token_indices,
};

// =====================================================================
// Helpers
// =====================================================================

/// Build a grammar with N distinct single-char tokens, each as an alternative
/// for the start rule. Returns (grammar, parse_table).
fn grammar_with_n_tokens(n: usize) -> (Grammar, ParseTable) {
    assert!(n >= 1 && n <= 26);
    let mut builder = GrammarBuilder::new("tok_grammar");
    let names: Vec<String> = (0..n)
        .map(|i| format!("t{}", (b'a' + i as u8) as char))
        .collect();
    let patterns: Vec<String> = (0..n)
        .map(|i| format!("{}", (b'a' + i as u8) as char))
        .collect();
    for i in 0..n {
        builder = builder.token(&names[i], &patterns[i]);
    }
    for i in 0..n {
        builder = builder.rule("start", vec![&names[i]]);
    }
    builder = builder.start("start");
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

fn single_token_grammar() -> (Grammar, ParseTable) {
    grammar_with_n_tokens(1)
}

fn two_token_grammar() -> (Grammar, ParseTable) {
    grammar_with_n_tokens(2)
}

fn five_token_grammar() -> (Grammar, ParseTable) {
    grammar_with_n_tokens(5)
}

fn ten_token_grammar() -> (Grammar, ParseTable) {
    grammar_with_n_tokens(10)
}

fn chain_grammar() -> (Grammar, ParseTable) {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

fn precedence_grammar() -> (Grammar, ParseTable) {
    let mut g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

fn alternatives_grammar() -> (Grammar, ParseTable) {
    let mut g = GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["A", "B"])
        .rule("start", vec!["C"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

// =====================================================================
// 1. StaticLanguageGenerator construction
// =====================================================================

#[test]
fn construct_generator_single_token() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.grammar.name.is_empty());
}

#[test]
fn construct_generator_two_tokens() {
    let (g, t) = two_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "tok_grammar");
}

#[test]
fn construct_generator_five_tokens() {
    let (g, t) = five_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.tokens.len(), 5);
}

#[test]
fn construct_generator_preserves_grammar_name() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "tok_grammar");
}

#[test]
fn construct_generator_preserves_parse_table() {
    let (g, t) = single_token_grammar();
    let state_count = t.state_count;
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.parse_table.state_count, state_count);
}

#[test]
fn construct_generator_default_start_not_empty() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn construct_generator_set_start_can_be_empty() {
    let (g, t) = single_token_grammar();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

#[test]
fn construct_generator_compressed_tables_initially_none() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compressed_tables.is_none());
}

// =====================================================================
// 2. generate_language_code returns non-empty TokenStream
// =====================================================================

#[test]
fn codegen_single_token_nonempty() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn codegen_two_tokens_nonempty() {
    let (g, t) = two_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn codegen_five_tokens_nonempty() {
    let (g, t) = five_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn codegen_ten_tokens_nonempty() {
    let (g, t) = ten_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn codegen_contains_tslanguage() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("TSLanguage"));
}

#[test]
fn codegen_contains_grammar_name() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("tok_grammar"));
}

// =====================================================================
// 3. Various grammar sizes
// =====================================================================

#[test]
fn grammar_size_1_state_count_positive() {
    let (_, t) = grammar_with_n_tokens(1);
    assert!(t.state_count > 0);
}

#[test]
fn grammar_size_2_state_count_positive() {
    let (_, t) = grammar_with_n_tokens(2);
    assert!(t.state_count > 0);
}

#[test]
fn grammar_size_5_state_count_positive() {
    let (_, t) = grammar_with_n_tokens(5);
    assert!(t.state_count > 0);
}

#[test]
fn grammar_size_10_state_count_positive() {
    let (_, t) = grammar_with_n_tokens(10);
    assert!(t.state_count > 0);
}

#[test]
fn grammar_size_scales_symbol_count() {
    let (_, t2) = grammar_with_n_tokens(2);
    let (_, t5) = grammar_with_n_tokens(5);
    assert!(t5.symbol_count >= t2.symbol_count);
}

#[test]
fn grammar_size_1_symbol_count_positive() {
    let (_, t) = grammar_with_n_tokens(1);
    assert!(t.symbol_count > 0);
}

// =====================================================================
// 4. Grammar with precedence
// =====================================================================

#[test]
fn precedence_grammar_constructs() {
    let (g, t) = precedence_grammar();
    let _ = StaticLanguageGenerator::new(g, t);
}

#[test]
fn precedence_grammar_codegen_nonempty() {
    let (g, t) = precedence_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn precedence_grammar_state_count_positive() {
    let (_, t) = precedence_grammar();
    assert!(t.state_count > 0);
}

#[test]
fn precedence_grammar_has_multiple_rules() {
    let (g, _) = precedence_grammar();
    let total_rules: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(
        total_rules >= 3,
        "Expected at least 3 rules, got {total_rules}"
    );
}

// =====================================================================
// 5. Grammar with chain rules
// =====================================================================

#[test]
fn chain_grammar_constructs() {
    let (g, t) = chain_grammar();
    let _ = StaticLanguageGenerator::new(g, t);
}

#[test]
fn chain_grammar_codegen_nonempty() {
    let (g, t) = chain_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn chain_grammar_state_count_positive() {
    let (_, t) = chain_grammar();
    assert!(t.state_count > 0);
}

#[test]
fn chain_grammar_has_three_rules() {
    let (g, _) = chain_grammar();
    assert_eq!(
        g.rules.len(),
        3,
        "Chain grammar should have 3 non-terminal rules"
    );
}

// =====================================================================
// 6. Grammar with alternatives
// =====================================================================

#[test]
fn alternatives_grammar_constructs() {
    let (g, t) = alternatives_grammar();
    let _ = StaticLanguageGenerator::new(g, t);
}

#[test]
fn alternatives_grammar_codegen_nonempty() {
    let (g, t) = alternatives_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn alternatives_grammar_state_count_positive() {
    let (_, t) = alternatives_grammar();
    assert!(t.state_count > 0);
}

#[test]
fn alternatives_grammar_multiple_productions() {
    let (g, _) = alternatives_grammar();
    let start_rules: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(start_rules >= 4);
}

// =====================================================================
// 7. Deterministic output
// =====================================================================

#[test]
fn codegen_deterministic_single_token() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let code1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);
}

#[test]
fn codegen_deterministic_five_tokens() {
    let (g1, t1) = five_token_grammar();
    let (g2, t2) = five_token_grammar();
    let code1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);
}

#[test]
fn codegen_deterministic_chain() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = chain_grammar();
    let code1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);
}

#[test]
fn codegen_deterministic_precedence() {
    let (g1, t1) = precedence_grammar();
    let (g2, t2) = precedence_grammar();
    let code1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);
}

#[test]
fn node_types_deterministic_single() {
    let (g1, _) = single_token_grammar();
    let (g2, _) = single_token_grammar();
    let n1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let n2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(n1, n2);
}

#[test]
fn node_types_deterministic_five() {
    let (g1, _) = five_token_grammar();
    let (g2, _) = five_token_grammar();
    let n1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let n2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(n1, n2);
}

// =====================================================================
// 8. ParseTable properties
// =====================================================================

#[test]
fn parse_table_state_count_gt_zero() {
    let (_, t) = single_token_grammar();
    assert!(t.state_count > 0);
}

#[test]
fn parse_table_symbol_count_gt_zero() {
    let (_, t) = single_token_grammar();
    assert!(t.symbol_count > 0);
}

#[test]
fn parse_table_eof_symbol_has_mapping() {
    let (_, t) = single_token_grammar();
    assert!(
        t.symbol_to_index.contains_key(&t.eof_symbol),
        "EOF symbol should be present in symbol_to_index"
    );
}

#[test]
fn parse_table_action_table_rows_eq_state_count() {
    let (_, t) = single_token_grammar();
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn parse_table_symbol_to_index_nonempty() {
    let (_, t) = single_token_grammar();
    assert!(!t.symbol_to_index.is_empty());
}

#[test]
fn parse_table_index_to_symbol_nonempty() {
    let (_, t) = single_token_grammar();
    assert!(!t.index_to_symbol.is_empty());
}

#[test]
fn parse_table_token_count_positive() {
    let (_, t) = single_token_grammar();
    assert!(t.token_count > 0);
}

#[test]
fn parse_table_five_tokens_more_symbols() {
    let (_, t1) = single_token_grammar();
    let (_, t5) = five_token_grammar();
    assert!(t5.symbol_count > t1.symbol_count);
}

// =====================================================================
// 9. NodeTypesGenerator combined with StaticLanguageGenerator
// =====================================================================

#[test]
fn combined_single_token() {
    let (g, t) = single_token_grammar();
    let node_gen = NodeTypesGenerator::new(&g);
    let node_types = node_gen.generate();
    assert!(node_types.is_ok());
    let lang_gen = StaticLanguageGenerator::new(g, t);
    let code = lang_gen.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn combined_chain_grammar() {
    let (g, t) = chain_grammar();
    let nt = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!nt.is_empty());
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn combined_precedence_grammar() {
    let (g, t) = precedence_grammar();
    let nt = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!nt.is_empty());
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn combined_alternatives_grammar() {
    let (g, t) = alternatives_grammar();
    let nt = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!nt.is_empty());
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn node_types_produces_valid_json_single() {
    let (g, _) = single_token_grammar();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_produces_valid_json_chain() {
    let (g, _) = chain_grammar();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_produces_valid_json_precedence() {
    let (g, _) = precedence_grammar();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

// =====================================================================
// 10. Multiple builds in sequence
// =====================================================================

#[test]
fn sequential_builds_independent() {
    for n in 1..=5 {
        let (g, t) = grammar_with_n_tokens(n);
        let slg = StaticLanguageGenerator::new(g, t);
        let code = slg.generate_language_code();
        assert!(
            !code.to_string().is_empty(),
            "Build {n} produced empty code"
        );
    }
}

#[test]
fn sequential_node_types_independent() {
    for n in 1..=5 {
        let (g, _) = grammar_with_n_tokens(n);
        let result = NodeTypesGenerator::new(&g).generate();
        assert!(result.is_ok(), "NodeTypes build {n} failed");
    }
}

#[test]
fn sequential_builds_different_grammars() {
    let codes: Vec<String> = (1..=3)
        .map(|n| {
            let (g, t) = grammar_with_n_tokens(n);
            StaticLanguageGenerator::new(g, t)
                .generate_language_code()
                .to_string()
        })
        .collect();
    // Different grammar sizes should produce different code
    assert_ne!(codes[0], codes[1]);
    assert_ne!(codes[1], codes[2]);
}

// =====================================================================
// Additional: compression pipeline
// =====================================================================

#[test]
fn compress_single_token_table() {
    let (g, t) = single_token_grammar();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok());
}

#[test]
fn compress_five_token_table() {
    let (g, t) = five_token_grammar();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok());
}

#[test]
fn compress_precedence_table() {
    let (g, t) = precedence_grammar();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok());
}

#[test]
fn compress_chain_table() {
    let (g, t) = chain_grammar();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok());
}

#[test]
fn compress_alternatives_table() {
    let (g, t) = alternatives_grammar();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok());
}

#[test]
fn compress_with_eof_accepts_flag() {
    let (g, t) = single_token_grammar();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result_false = compressor.compress(&t, &token_indices, false);
    let result_true = compressor.compress(&t, &token_indices, true);
    assert!(result_false.is_ok());
    assert!(result_true.is_ok());
}

#[test]
fn token_indices_include_eof_when_mapped() {
    let (g, t) = single_token_grammar();
    let indices = collect_token_indices(&g, &t);
    // EOF should be included if it has a mapping
    if let Some(&idx) = t.symbol_to_index.get(&t.eof_symbol) {
        assert!(indices.contains(&idx));
    }
}

#[test]
fn generate_node_types_string_nonempty() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let nt = slg.generate_node_types();
    assert!(!nt.is_empty());
}

#[test]
fn generate_node_types_valid_json() {
    let (g, t) = five_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let nt = slg.generate_node_types();
    let val: serde_json::Value = serde_json::from_str(&nt).expect("valid JSON");
    assert!(val.is_array());
}

// =====================================================================
// Edge cases and extra coverage
// =====================================================================

#[test]
fn empty_grammar_node_types_ok() {
    let g = Grammar::new("empty".to_string());
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn empty_grammar_node_types_empty_array() {
    let g = Grammar::new("empty".to_string());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.as_array().unwrap().is_empty());
}

#[test]
fn ten_token_codegen_contains_language_struct() {
    let (g, t) = ten_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("TSLanguage"));
}

#[test]
fn preset_python_like_node_types() {
    let g = GrammarBuilder::python_like();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn preset_javascript_like_node_types() {
    let g = GrammarBuilder::javascript_like();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}
