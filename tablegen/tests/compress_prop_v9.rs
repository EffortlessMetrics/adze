//! Property-based and unit tests for table compression in adze-tablegen (v9).
//!
//! Covers: `TableCompressor`, `StaticLanguageGenerator`, `AbiLanguageBuilder`,
//! `NodeTypesGenerator`, determinism, output validation, grammar variations,
//! and compression pipeline properties.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::abi_builder::AbiLanguageBuilder;
use adze_tablegen::{
    NodeTypesGenerator, StaticLanguageGenerator, TableCompressor, collect_token_indices,
};
use proptest::prelude::*;

// =====================================================================
// Strategies
// =====================================================================

fn arb_token_count() -> impl Strategy<Value = usize> {
    1usize..6
}

fn arb_rule_count() -> impl Strategy<Value = usize> {
    1usize..5
}

fn arb_prec() -> impl Strategy<Value = i16> {
    -5i16..10
}

// =====================================================================
// Helper: build a grammar with `n` tokens, each as an alternative for "start".
// Grammar name includes `idx` for uniqueness in property tests.
// =====================================================================

fn make_grammar(n: usize, idx: usize) -> (Grammar, ParseTable) {
    assert!((1..=26).contains(&n));
    let name = format!("g{idx}");
    let mut builder = GrammarBuilder::new(&name);
    let names: Vec<String> = (0..n)
        .map(|i| format!("t{}", (b'a' + i as u8) as char))
        .collect();
    let patterns: Vec<String> = (0..n)
        .map(|i| format!("{}", (b'a' + i as u8) as char))
        .collect();
    for (i, tok_name) in names.iter().enumerate() {
        builder = builder.token(tok_name, &patterns[i]);
    }
    for tok_name in &names {
        builder = builder.rule("start", vec![tok_name.as_str()]);
    }
    builder = builder.start("start");
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

/// Build a grammar with multiple nonterminal rules chaining into tokens.
fn make_chain_grammar(depth: usize, idx: usize) -> (Grammar, ParseTable) {
    let name = format!("chain{idx}");
    let mut builder = GrammarBuilder::new(&name);
    builder = builder.token("x", "x");
    // chain: r0 -> r1 -> ... -> r{depth-1} -> x
    let rule_names: Vec<String> = (0..depth).map(|i| format!("r{i}")).collect();
    for i in 0..depth - 1 {
        builder = builder.rule(&rule_names[i], vec![rule_names[i + 1].as_str()]);
    }
    builder = builder.rule(&rule_names[depth - 1], vec!["x"]);
    builder = builder.start(&rule_names[0]);
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

/// Build a grammar with precedence on an expression rule.
fn make_prec_grammar(prec: i16, idx: usize) -> (Grammar, ParseTable) {
    let name = format!("prec{idx}");
    let mut g = GrammarBuilder::new(&name)
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .rule_with_precedence(
            "expr",
            vec!["expr", "PLUS", "expr"],
            prec,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

/// Build a grammar with multiple operators at different precedences.
fn make_multi_prec_grammar(prec1: i16, prec2: i16, idx: usize) -> (Grammar, ParseTable) {
    let name = format!("mprec{idx}");
    let mut g = GrammarBuilder::new(&name)
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule_with_precedence(
            "expr",
            vec!["expr", "PLUS", "expr"],
            prec1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "STAR", "expr"],
            prec2,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

/// Build a grammar with right-associative operators.
fn make_right_assoc_grammar(prec: i16, idx: usize) -> (Grammar, ParseTable) {
    let name = format!("rassoc{idx}");
    let mut g = GrammarBuilder::new(&name)
        .token("NUM", r"\d+")
        .token("EXP", "^")
        .rule_with_precedence(
            "expr",
            vec!["expr", "EXP", "expr"],
            prec,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("lr1");
    (g, t)
}

// =====================================================================
// 1-10. Core proptest properties
// =====================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // 1. Any valid grammar → TableCompressor compress succeeds
    #[test]
    fn prop_compressor_succeeds(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 1);
        let indices = collect_token_indices(&g, &t);
        let compressor = TableCompressor::new();
        let result = compressor.compress(&t, &indices, false);
        prop_assert!(result.is_ok(), "compress failed for {n} tokens");
    }

    // 2. Any valid grammar → StaticLanguageGenerator succeeds
    #[test]
    fn prop_static_generator_succeeds(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 2);
        let slg = StaticLanguageGenerator::new(g, t);
        let code = slg.generate_language_code();
        prop_assert!(!code.to_string().is_empty());
    }

    // 3. Any valid grammar → AbiLanguageBuilder succeeds
    #[test]
    fn prop_abi_builder_succeeds(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 3);
        let abi = AbiLanguageBuilder::new(&g, &t);
        let code = abi.generate();
        prop_assert!(!code.to_string().is_empty());
    }

    // 4. Any valid grammar → NodeTypesGenerator succeeds
    #[test]
    fn prop_node_types_succeeds(n in arb_token_count()) {
        let (g, _) = make_grammar(n, 4);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok());
    }

    // 5. Compressed output is deterministic
    #[test]
    fn prop_compression_deterministic(n in arb_token_count()) {
        let (g1, t1) = make_grammar(n, 5);
        let (g2, t2) = make_grammar(n, 5);
        let idx1 = collect_token_indices(&g1, &t1);
        let idx2 = collect_token_indices(&g2, &t2);
        let c = TableCompressor::new();
        let r1 = c.compress(&t1, &idx1, false).unwrap();
        let r2 = c.compress(&t2, &idx2, false).unwrap();
        prop_assert_eq!(r1.small_table_threshold, r2.small_table_threshold);
    }

    // 6. Static code is non-empty
    #[test]
    fn prop_static_code_nonempty(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 6);
        let code = StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string();
        prop_assert!(!code.is_empty());
    }

    // 7. ABI output is non-empty
    #[test]
    fn prop_abi_output_nonempty(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 7);
        let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(!code.is_empty());
    }

    // 8. Node types JSON is non-empty
    #[test]
    fn prop_node_types_nonempty(n in arb_token_count()) {
        let (g, _) = make_grammar(n, 8);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        prop_assert!(!json_str.is_empty());
    }

    // 9. Node types JSON is valid JSON
    #[test]
    fn prop_node_types_valid_json(n in arb_token_count()) {
        let (g, _) = make_grammar(n, 9);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        prop_assert!(val.is_array());
    }

    // 10. Grammar with precedence → all generators work
    #[test]
    fn prop_precedence_all_generators(prec in arb_prec()) {
        let (g, t) = make_prec_grammar(prec, 10);
        let indices = collect_token_indices(&g, &t);
        let compress_ok = TableCompressor::new().compress(&t, &indices, false).is_ok();
        prop_assert!(compress_ok);
        let slg_code = StaticLanguageGenerator::new(g.clone(), t.clone())
            .generate_language_code()
            .to_string();
        prop_assert!(!slg_code.is_empty());
        let abi_code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(!abi_code.is_empty());
        let nt = NodeTypesGenerator::new(&g).generate();
        prop_assert!(nt.is_ok());
    }
}

// =====================================================================
// 11-20. Proptest: chain grammars, associativity, multi-prec
// =====================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // 11. Chain grammar compresses successfully (uses arb_rule_count + 1 for depth >= 2)
    #[test]
    fn prop_chain_compressor(rc in arb_rule_count()) {
        let depth = rc + 1; // ensure depth >= 2
        let (g, t) = make_chain_grammar(depth, 11);
        let indices = collect_token_indices(&g, &t);
        let result = TableCompressor::new().compress(&t, &indices, false);
        prop_assert!(result.is_ok());
    }

    // 12. Chain grammar → static generator
    #[test]
    fn prop_chain_static_gen(rc in arb_rule_count()) {
        let depth = rc + 1;
        let (g, t) = make_chain_grammar(depth, 12);
        let code = StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string();
        prop_assert!(!code.is_empty());
    }

    // 13. Chain grammar → ABI builder
    #[test]
    fn prop_chain_abi(rc in arb_rule_count()) {
        let depth = rc + 1;
        let (g, t) = make_chain_grammar(depth, 13);
        let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(!code.is_empty());
    }

    // 14. Chain grammar → node types
    #[test]
    fn prop_chain_node_types(rc in arb_rule_count()) {
        let depth = rc + 1;
        let (g, _) = make_chain_grammar(depth, 14);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok());
    }

    // 15. Multi-precedence grammar compresses
    #[test]
    fn prop_multi_prec_compress(p1 in arb_prec(), p2 in arb_prec()) {
        let (g, t) = make_multi_prec_grammar(p1, p2, 15);
        let indices = collect_token_indices(&g, &t);
        let result = TableCompressor::new().compress(&t, &indices, false);
        prop_assert!(result.is_ok());
    }

    // 16. Multi-precedence → static gen
    #[test]
    fn prop_multi_prec_static(p1 in arb_prec(), p2 in arb_prec()) {
        let (g, t) = make_multi_prec_grammar(p1, p2, 16);
        let code = StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string();
        prop_assert!(!code.is_empty());
    }

    // 17. Multi-precedence → ABI
    #[test]
    fn prop_multi_prec_abi(p1 in arb_prec(), p2 in arb_prec()) {
        let (g, t) = make_multi_prec_grammar(p1, p2, 17);
        let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(!code.is_empty());
    }

    // 18. Multi-precedence → node types valid JSON
    #[test]
    fn prop_multi_prec_node_types_json(p1 in arb_prec(), p2 in arb_prec()) {
        let (g, _) = make_multi_prec_grammar(p1, p2, 18);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        prop_assert!(val.is_array());
    }

    // 19. Right-associative grammar → all generators
    #[test]
    fn prop_right_assoc_all(prec in arb_prec()) {
        let (g, t) = make_right_assoc_grammar(prec, 19);
        let indices = collect_token_indices(&g, &t);
        prop_assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
        let code = StaticLanguageGenerator::new(g.clone(), t.clone())
            .generate_language_code()
            .to_string();
        prop_assert!(!code.is_empty());
        let abi = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(!abi.is_empty());
    }

    // 20. Determinism: static code identical across two runs
    #[test]
    fn prop_static_deterministic(n in arb_token_count()) {
        let (g1, t1) = make_grammar(n, 20);
        let (g2, t2) = make_grammar(n, 20);
        let c1 = StaticLanguageGenerator::new(g1, t1)
            .generate_language_code()
            .to_string();
        let c2 = StaticLanguageGenerator::new(g2, t2)
            .generate_language_code()
            .to_string();
        prop_assert_eq!(c1, c2);
    }
}

// =====================================================================
// 21-30. Proptest: output content validation
// =====================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // 21. Static code contains TSLanguage
    #[test]
    fn prop_static_contains_tslanguage(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 21);
        let code = StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string();
        prop_assert!(code.contains("TSLanguage"));
    }

    // 22. Static code contains grammar name
    #[test]
    fn prop_static_contains_grammar_name(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 22);
        let code = StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string();
        let expected_name = format!("g{}", 22);
        prop_assert!(code.contains(&expected_name));
    }

    // 23. Node types JSON is an array
    #[test]
    fn prop_node_types_is_array(n in arb_token_count()) {
        let (g, _) = make_grammar(n, 23);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        prop_assert!(val.is_array());
    }

    // 24. Node types array has at least one entry per non-terminal
    #[test]
    fn prop_node_types_entries(n in arb_token_count()) {
        let (g, _) = make_grammar(n, 24);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let arr = val.as_array().unwrap();
        // Grammar with tokens has at least one rule ("start"), so non-empty
        prop_assert!(!arr.is_empty());
    }

    // 25. Compress with start_can_be_empty=true still succeeds
    #[test]
    fn prop_compress_start_empty_flag(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 25);
        let indices = collect_token_indices(&g, &t);
        let result = TableCompressor::new().compress(&t, &indices, true);
        prop_assert!(result.is_ok());
    }

    // 26. Token indices include EOF mapping when present
    #[test]
    fn prop_token_indices_eof(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 26);
        let indices = collect_token_indices(&g, &t);
        if let Some(&idx) = t.symbol_to_index.get(&t.eof_symbol) {
            prop_assert!(indices.contains(&idx));
        }
    }

    // 27. Token indices are sorted
    #[test]
    fn prop_token_indices_sorted(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 27);
        let indices = collect_token_indices(&g, &t);
        for window in indices.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }

    // 28. Token indices are deduplicated
    #[test]
    fn prop_token_indices_unique(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 28);
        let indices = collect_token_indices(&g, &t);
        let mut deduped = indices.clone();
        deduped.dedup();
        prop_assert_eq!(indices.len(), deduped.len());
    }

    // 29. Parse table state count is positive
    #[test]
    fn prop_state_count_positive(n in arb_token_count()) {
        let (_, t) = make_grammar(n, 29);
        prop_assert!(t.state_count > 0);
    }

    // 30. Parse table symbol count grows with tokens
    #[test]
    fn prop_symbol_count_grows(n in 2usize..6) {
        let (_, t_small) = make_grammar(1, 30);
        let (_, t_large) = make_grammar(n, 30);
        prop_assert!(t_large.symbol_count >= t_small.symbol_count);
    }
}

// =====================================================================
// 31-40. Proptest: ABI builder variations
// =====================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // 31. ABI code contains struct-like output
    #[test]
    fn prop_abi_has_content(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 31);
        let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(code.len() > 10);
    }

    // 32. ABI deterministic
    #[test]
    fn prop_abi_deterministic(n in arb_token_count()) {
        let (g1, t1) = make_grammar(n, 32);
        let (g2, t2) = make_grammar(n, 32);
        let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
        prop_assert_eq!(c1, c2);
    }

    // 33. ABI for chain grammar (uses arb_rule_count)
    #[test]
    fn prop_abi_chain(rc in arb_rule_count()) {
        let depth = rc + 1;
        let (g, t) = make_chain_grammar(depth, 33);
        let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(!code.is_empty());
    }

    // 34. ABI for precedence grammar
    #[test]
    fn prop_abi_prec(prec in arb_prec()) {
        let (g, t) = make_prec_grammar(prec, 34);
        let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        prop_assert!(!code.is_empty());
    }

    // 35. Node types deterministic across runs
    #[test]
    fn prop_node_types_deterministic(n in arb_token_count()) {
        let (g1, _) = make_grammar(n, 35);
        let (g2, _) = make_grammar(n, 35);
        let n1 = NodeTypesGenerator::new(&g1).generate().unwrap();
        let n2 = NodeTypesGenerator::new(&g2).generate().unwrap();
        prop_assert_eq!(n1, n2);
    }

    // 36. StaticLanguageGenerator generate_node_types non-empty
    #[test]
    fn prop_slg_node_types_nonempty(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 36);
        let slg = StaticLanguageGenerator::new(g, t);
        let nt = slg.generate_node_types();
        prop_assert!(!nt.is_empty());
    }

    // 37. StaticLanguageGenerator generate_node_types is valid JSON
    #[test]
    fn prop_slg_node_types_json(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 37);
        let slg = StaticLanguageGenerator::new(g, t);
        let nt = slg.generate_node_types();
        let val: serde_json::Value = serde_json::from_str(&nt).expect("valid JSON");
        prop_assert!(val.is_array());
    }

    // 38. Compressed tables threshold is consistent
    #[test]
    fn prop_compress_threshold(n in arb_token_count()) {
        let (g, t) = make_grammar(n, 38);
        let indices = collect_token_indices(&g, &t);
        let tables = TableCompressor::new().compress(&t, &indices, false).unwrap();
        prop_assert_eq!(tables.small_table_threshold, 32768);
    }

    // 39. Action table dimensions match state count
    #[test]
    fn prop_action_table_rows(n in arb_token_count()) {
        let (_, t) = make_grammar(n, 39);
        prop_assert_eq!(t.action_table.len(), t.state_count);
    }

    // 40. Goto table dimensions match state count
    #[test]
    fn prop_goto_table_rows(n in arb_token_count()) {
        let (_, t) = make_grammar(n, 40);
        prop_assert_eq!(t.goto_table.len(), t.state_count);
    }
}

// =====================================================================
// 41-55. Unit tests: specific grammars
// =====================================================================

#[test]
fn unit_single_token_compress() {
    let (g, t) = make_grammar(1, 41);
    let indices = collect_token_indices(&g, &t);
    let result = TableCompressor::new().compress(&t, &indices, false);
    assert!(result.is_ok());
}

#[test]
fn unit_single_token_static_gen() {
    let (g, t) = make_grammar(1, 42);
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn unit_single_token_abi() {
    let (g, t) = make_grammar(1, 43);
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn unit_single_token_node_types() {
    let (g, _) = make_grammar(1, 44);
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn unit_five_tokens_all_generators() {
    let (g, t) = make_grammar(5, 45);
    let indices = collect_token_indices(&g, &t);
    assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
    let slg_code = StaticLanguageGenerator::new(g.clone(), t.clone())
        .generate_language_code()
        .to_string();
    assert!(!slg_code.is_empty());
    let abi_code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!abi_code.is_empty());
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn unit_chain_depth2_compress() {
    let (g, t) = make_chain_grammar(2, 46);
    let indices = collect_token_indices(&g, &t);
    assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
}

#[test]
fn unit_chain_depth5_compress() {
    let (g, t) = make_chain_grammar(5, 47);
    let indices = collect_token_indices(&g, &t);
    assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
}

#[test]
fn unit_prec_grammar_compress() {
    let (g, t) = make_prec_grammar(1, 48);
    let indices = collect_token_indices(&g, &t);
    assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
}

#[test]
fn unit_right_assoc_compress() {
    let (g, t) = make_right_assoc_grammar(2, 49);
    let indices = collect_token_indices(&g, &t);
    assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
}

#[test]
fn unit_multi_prec_compress() {
    let (g, t) = make_multi_prec_grammar(1, 2, 50);
    let indices = collect_token_indices(&g, &t);
    assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
}

#[test]
fn unit_empty_grammar_node_types() {
    let g = Grammar::new("empty".to_string());
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn unit_empty_grammar_node_types_empty_array() {
    let g = Grammar::new("empty".to_string());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.as_array().unwrap().is_empty());
}

#[test]
fn unit_start_can_be_empty_flag() {
    let (g, t) = make_grammar(1, 53);
    let mut slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

#[test]
fn unit_compressed_tables_initially_none() {
    let (g, t) = make_grammar(1, 54);
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn unit_static_codegen_contains_tslanguage() {
    let (g, t) = make_grammar(2, 55);
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("TSLanguage"));
}

// =====================================================================
// 56-65. Unit tests: determinism, output shape
// =====================================================================

#[test]
fn unit_static_deterministic_single() {
    let (g1, t1) = make_grammar(1, 56);
    let (g2, t2) = make_grammar(1, 56);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn unit_abi_deterministic_single() {
    let (g1, t1) = make_grammar(1, 57);
    let (g2, t2) = make_grammar(1, 57);
    let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let c2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_eq!(c1, c2);
}

#[test]
fn unit_node_types_deterministic_single() {
    let (g1, _) = make_grammar(1, 58);
    let (g2, _) = make_grammar(1, 58);
    let n1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let n2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(n1, n2);
}

#[test]
fn unit_different_grammar_sizes_differ() {
    let (g1, t1) = make_grammar(1, 59);
    let (g2, t2) = make_grammar(3, 59);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(c1, c2);
}

#[test]
fn unit_sequential_builds_independent() {
    for n in 1..=5 {
        let (g, t) = make_grammar(n, 60);
        let code = StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string();
        assert!(!code.is_empty(), "Build {n} produced empty code");
    }
}

#[test]
fn unit_sequential_node_types_independent() {
    for n in 1..=5 {
        let (g, _) = make_grammar(n, 61);
        let result = NodeTypesGenerator::new(&g).generate();
        assert!(result.is_ok(), "NodeTypes build {n} failed");
    }
}

#[test]
fn unit_sequential_compress_independent() {
    for n in 1..=5 {
        let (g, t) = make_grammar(n, 62);
        let indices = collect_token_indices(&g, &t);
        let result = TableCompressor::new().compress(&t, &indices, false);
        assert!(result.is_ok(), "Compress {n} failed");
    }
}

#[test]
fn unit_node_types_json_array_structure() {
    let (g, _) = make_grammar(3, 63);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    let arr = val.as_array().unwrap();
    for entry in arr {
        assert!(
            entry.is_object(),
            "Each node type entry should be an object"
        );
    }
}

#[test]
fn unit_compress_both_eof_flags() {
    let (g, t) = make_grammar(2, 64);
    let indices = collect_token_indices(&g, &t);
    let c = TableCompressor::new();
    assert!(c.compress(&t, &indices, false).is_ok());
    assert!(c.compress(&t, &indices, true).is_ok());
}

#[test]
fn unit_token_indices_nonempty() {
    let (g, t) = make_grammar(3, 65);
    let indices = collect_token_indices(&g, &t);
    assert!(!indices.is_empty());
}

// =====================================================================
// 66-75. Unit tests: grammar structure and parse table properties
// =====================================================================

#[test]
fn unit_grammar_name_preserved() {
    let (g, t) = make_grammar(1, 66);
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "g66");
}

#[test]
fn unit_grammar_token_count_matches() {
    let (g, _) = make_grammar(4, 67);
    assert_eq!(g.tokens.len(), 4);
}

#[test]
fn unit_parse_table_eof_in_symbol_map() {
    let (_, t) = make_grammar(1, 68);
    assert!(t.symbol_to_index.contains_key(&t.eof_symbol));
}

#[test]
fn unit_parse_table_has_rules() {
    let (_, t) = make_grammar(2, 69);
    assert!(!t.rules.is_empty());
}

#[test]
fn unit_chain_grammar_rule_count() {
    let (g, _) = make_chain_grammar(3, 70);
    assert_eq!(g.rules.len(), 3);
}

#[test]
fn unit_prec_grammar_multiple_rules() {
    let (g, _) = make_prec_grammar(1, 71);
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total >= 2, "Expected at least 2 rules, got {total}");
}

#[test]
fn unit_multi_prec_grammar_rules() {
    let (g, _) = make_multi_prec_grammar(1, 2, 72);
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total >= 3, "Expected at least 3 rules, got {total}");
}

#[test]
fn unit_right_assoc_grammar_rules() {
    let (g, _) = make_right_assoc_grammar(1, 73);
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total >= 2, "Expected at least 2 rules, got {total}");
}

#[test]
fn unit_symbol_to_index_nonempty() {
    let (_, t) = make_grammar(1, 74);
    assert!(!t.symbol_to_index.is_empty());
}

#[test]
fn unit_index_to_symbol_nonempty() {
    let (_, t) = make_grammar(1, 75);
    assert!(!t.index_to_symbol.is_empty());
}

// =====================================================================
// 76-85. Unit tests: edge cases and cross-feature combinations
// =====================================================================

#[test]
fn unit_slg_generate_node_types_valid_json() {
    let (g, t) = make_grammar(3, 76);
    let slg = StaticLanguageGenerator::new(g, t);
    let nt = slg.generate_node_types();
    let val: serde_json::Value = serde_json::from_str(&nt).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn unit_compress_chain_with_eof_flag() {
    let (g, t) = make_chain_grammar(3, 77);
    let indices = collect_token_indices(&g, &t);
    let c = TableCompressor::new();
    assert!(c.compress(&t, &indices, true).is_ok());
}

#[test]
fn unit_compress_prec_with_eof_flag() {
    let (g, t) = make_prec_grammar(5, 78);
    let indices = collect_token_indices(&g, &t);
    let c = TableCompressor::new();
    assert!(c.compress(&t, &indices, true).is_ok());
}

#[test]
fn unit_abi_chain_depth4() {
    let (g, t) = make_chain_grammar(4, 79);
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn unit_abi_multi_prec() {
    let (g, t) = make_multi_prec_grammar(-3, 7, 80);
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn unit_abi_right_assoc() {
    let (g, t) = make_right_assoc_grammar(0, 81);
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn unit_node_types_chain_valid_json() {
    let (g, _) = make_chain_grammar(4, 82);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn unit_node_types_multi_prec_valid_json() {
    let (g, _) = make_multi_prec_grammar(0, 5, 83);
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn unit_compress_deterministic_across_calls() {
    let (g1, t1) = make_grammar(3, 84);
    let (g2, t2) = make_grammar(3, 84);
    let idx1 = collect_token_indices(&g1, &t1);
    let idx2 = collect_token_indices(&g2, &t2);
    let c = TableCompressor::new();
    let r1 = c.compress(&t1, &idx1, false).unwrap();
    let r2 = c.compress(&t2, &idx2, false).unwrap();
    assert_eq!(r1.small_table_threshold, r2.small_table_threshold);
}

#[test]
fn unit_all_generators_on_max_tokens() {
    let (g, t) = make_grammar(5, 85);
    let indices = collect_token_indices(&g, &t);
    assert!(TableCompressor::new().compress(&t, &indices, false).is_ok());
    let slg_code = StaticLanguageGenerator::new(g.clone(), t.clone())
        .generate_language_code()
        .to_string();
    assert!(!slg_code.is_empty());
    let abi_code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!abi_code.is_empty());
    let nt = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&nt).expect("valid JSON");
    assert!(val.is_array());
}
