//! Comprehensive tests for the full table generation pipeline.
//!
//! Covers StaticLanguageGenerator construction, language struct generation,
//! table compression, symbol name resolution, state/action counts,
//! grammar topologies, edge cases, and determinism.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{
    AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator, TableCompressor,
    collect_token_indices,
};
use serde_json::Value;

// =====================================================================
// Helpers
// =====================================================================

fn build_pipeline(grammar_fn: impl FnOnce() -> Grammar) -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = grammar_fn();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Single token, single rule: s -> a
fn single_token_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("single_token")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build()
    })
}

/// Two tokens in sequence: s -> a b
fn seq_two_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("seq_two")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s")
            .build()
    })
}

/// Two alternatives: s -> a | b
fn alt_two_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("alt_two")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build()
    })
}

/// Left-recursive: s -> a | s a
fn left_recursive_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("left_rec")
            .token("a", "a")
            .rule("s", vec!["a"])
            .rule("s", vec!["s", "a"])
            .start("s")
            .build()
    })
}

/// Right-recursive: s -> a | a s
fn right_recursive_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("right_rec")
            .token("a", "a")
            .rule("s", vec!["a"])
            .rule("s", vec!["a", "s"])
            .start("s")
            .build()
    })
}

/// Linear chain of non-terminals: s -> mid, mid -> leaf, leaf -> a
fn linear_chain_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("linear_chain")
            .token("a", "a")
            .rule("leaf", vec!["a"])
            .rule("mid", vec!["leaf"])
            .rule("s", vec!["mid"])
            .start("s")
            .build()
    })
}

/// Tree-shaped: s -> left right, left -> a, right -> b
fn tree_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("tree")
            .token("a", "a")
            .token("b", "b")
            .rule("left", vec!["a"])
            .rule("right", vec!["b"])
            .rule("s", vec!["left", "right"])
            .start("s")
            .build()
    })
}

/// Diamond: s -> p q, p -> x, q -> x, x -> a
fn diamond_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("diamond")
            .token("a", "a")
            .rule("x", vec!["a"])
            .rule("p", vec!["x"])
            .rule("q", vec!["x"])
            .rule("s", vec!["p", "q"])
            .start("s")
            .build()
    })
}

/// Long chain of tokens: s -> a b c d e
fn five_token_chain_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("five_tok")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .token("e", "e")
            .rule("s", vec!["a", "b", "c", "d", "e"])
            .start("s")
            .build()
    })
}

/// Three alternatives: s -> a | b | c
fn three_alt_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("three_alt")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .rule("s", vec!["c"])
            .start("s")
            .build()
    })
}

/// Precedence grammar: expr -> num | expr + expr | expr * expr
fn precedence_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("prec")
            .token("num", r"\d+")
            .token("plus", "+")
            .token("star", "*")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .start("expr")
            .build()
    })
}

/// Multi-level non-terminal: s -> inner b, inner -> a
fn two_nonterminal_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("two_nt")
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("s", vec!["inner", "b"])
            .start("s")
            .build()
    })
}

/// Mixed alternatives + sequence: s -> a b | c
fn mixed_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("mixed")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("s", vec!["a", "b"])
            .rule("s", vec!["c"])
            .start("s")
            .build()
    })
}

/// Wider tree: s -> l m r, l -> a, m -> b, r -> c
fn wide_tree_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("wide_tree")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("l", vec!["a"])
            .rule("m", vec!["b"])
            .rule("r", vec!["c"])
            .rule("s", vec!["l", "m", "r"])
            .start("s")
            .build()
    })
}

/// Right-assoc precedence: expr -> num | expr ^ expr (right)
fn right_assoc_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(|| {
        GrammarBuilder::new("right_assoc")
            .token("num", r"\d+")
            .token("caret", "^")
            .rule("expr", vec!["num"])
            .rule_with_precedence(
                "expr",
                vec!["expr", "caret", "expr"],
                1,
                Associativity::Right,
            )
            .start("expr")
            .build()
    })
}

// =====================================================================
// 1. StaticLanguageGenerator construction
// =====================================================================

#[test]
fn construct_single_token() {
    let (g, pt) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "single_token");
}

#[test]
fn construct_seq_two() {
    let (g, pt) = seq_two_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "seq_two");
}

#[test]
fn construct_alt_two() {
    let (g, pt) = alt_two_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "alt_two");
}

#[test]
fn construct_left_recursive() {
    let (g, pt) = left_recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "left_rec");
}

#[test]
fn construct_right_recursive() {
    let (g, pt) = right_recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "right_rec");
}

#[test]
fn construct_linear_chain() {
    let (g, pt) = linear_chain_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "linear_chain");
}

#[test]
fn construct_tree() {
    let (g, pt) = tree_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "tree");
}

#[test]
fn construct_diamond() {
    let (g, pt) = diamond_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "diamond");
}

#[test]
fn construct_five_token_chain() {
    let (g, pt) = five_token_chain_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "five_tok");
}

#[test]
fn construct_precedence() {
    let (g, pt) = precedence_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "prec");
}

#[test]
fn construct_wide_tree() {
    let (g, pt) = wide_tree_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "wide_tree");
}

#[test]
fn construct_right_assoc() {
    let (g, pt) = right_assoc_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "right_assoc");
}

// =====================================================================
// 2. Default field values
// =====================================================================

#[test]
fn default_compressed_tables_none() {
    let (g, pt) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn default_start_can_be_empty_false() {
    let (g, pt) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn set_start_can_be_empty_true() {
    let (g, pt) = single_token_grammar();
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

#[test]
fn set_start_can_be_empty_toggle() {
    let (g, pt) = single_token_grammar();
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.set_start_can_be_empty(true);
    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);
}

// =====================================================================
// 3. Language code generation produces non-empty output
// =====================================================================

#[test]
fn codegen_nonempty_single_token() {
    let (g, pt) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_seq_two() {
    let (g, pt) = seq_two_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_alt_two() {
    let (g, pt) = alt_two_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_left_recursive() {
    let (g, pt) = left_recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_right_recursive() {
    let (g, pt) = right_recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_linear_chain() {
    let (g, pt) = linear_chain_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_tree() {
    let (g, pt) = tree_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_diamond() {
    let (g, pt) = diamond_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_five_token_chain() {
    let (g, pt) = five_token_chain_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_three_alt() {
    let (g, pt) = three_alt_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_precedence() {
    let (g, pt) = precedence_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_mixed() {
    let (g, pt) = mixed_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_wide_tree() {
    let (g, pt) = wide_tree_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn codegen_nonempty_right_assoc() {
    let (g, pt) = right_assoc_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

// =====================================================================
// 4. Generated code contains language-related identifiers
// =====================================================================

#[test]
fn codegen_contains_language_ident() {
    let (g, pt) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("LANGUAGE") || code.contains("language") || code.contains("Language"),
        "generated code should reference a language identifier"
    );
}

#[test]
fn codegen_contains_numeric_literals() {
    let (g, pt) = seq_two_grammar();
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        code.chars().any(|c| c.is_ascii_digit()),
        "generated code should contain numeric literals for counts"
    );
}

// =====================================================================
// 5. Node types generation
// =====================================================================

#[test]
fn node_types_valid_json_single_token() {
    let (g, pt) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let json_str = slg.generate_node_types();
    let val: Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_valid_json_tree() {
    let (g, pt) = tree_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let val: Value = serde_json::from_str(&slg.generate_node_types()).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_valid_json_diamond() {
    let (g, pt) = diamond_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let val: Value = serde_json::from_str(&slg.generate_node_types()).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_entries_have_type_and_named() {
    let (g, pt) = two_nonterminal_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let arr: Vec<Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    for entry in &arr {
        assert!(entry.get("type").is_some(), "every entry must have 'type'");
        assert!(
            entry.get("named").is_some(),
            "every entry must have 'named'"
        );
    }
}

#[test]
fn node_types_more_rules_more_entries() {
    let (g_simple, pt_simple) = single_token_grammar();
    let (g_tree, pt_tree) = wide_tree_grammar();
    let arr_simple: Vec<Value> = serde_json::from_str(
        &StaticLanguageGenerator::new(g_simple, pt_simple).generate_node_types(),
    )
    .unwrap();
    let arr_tree: Vec<Value> =
        serde_json::from_str(&StaticLanguageGenerator::new(g_tree, pt_tree).generate_node_types())
            .unwrap();
    assert!(
        arr_tree.len() >= arr_simple.len(),
        "wider grammar should produce at least as many node types"
    );
}

// =====================================================================
// 6. State count / symbol count properties
// =====================================================================

#[test]
fn state_count_positive_single_token() {
    let (_, pt) = single_token_grammar();
    assert!(pt.state_count > 0);
}

#[test]
fn state_count_positive_recursive() {
    let (_, pt) = left_recursive_grammar();
    assert!(pt.state_count > 0);
}

#[test]
fn state_count_positive_precedence() {
    let (_, pt) = precedence_grammar();
    assert!(pt.state_count > 0);
}

#[test]
fn symbol_count_positive_single_token() {
    let (_, pt) = single_token_grammar();
    assert!(pt.symbol_count > 0);
}

#[test]
fn more_tokens_at_least_as_many_states() {
    let (_, pt_one) = single_token_grammar();
    let (_, pt_five) = five_token_chain_grammar();
    assert!(
        pt_five.state_count >= pt_one.state_count,
        "5-token chain should have at least as many states as 1-token grammar"
    );
}

#[test]
fn more_alternatives_at_least_as_many_states() {
    let (_, pt_alt2) = alt_two_grammar();
    let (_, pt_alt3) = three_alt_grammar();
    assert!(
        pt_alt3.state_count >= pt_alt2.state_count,
        "3 alternatives should yield at least as many states as 2"
    );
}

#[test]
fn action_table_rows_match_state_count() {
    let (_, pt) = left_recursive_grammar();
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn goto_table_rows_match_state_count() {
    let (_, pt) = tree_grammar();
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn symbol_metadata_nonempty() {
    let (_, pt) = single_token_grammar();
    assert!(!pt.symbol_metadata.is_empty());
}

#[test]
fn rules_nonempty() {
    let (_, pt) = single_token_grammar();
    assert!(
        !pt.rules.is_empty(),
        "parse table should contain at least one rule"
    );
}

// =====================================================================
// 7. Table compression pipeline
// =====================================================================

fn compress_ok(g: &Grammar, pt: &adze_glr_core::ParseTable) {
    let ti = collect_token_indices(g, pt);
    TableCompressor::new()
        .compress(pt, &ti, false)
        .expect("compression should succeed");
}

#[test]
fn compress_single_token() {
    let (g, pt) = single_token_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_seq_two() {
    let (g, pt) = seq_two_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_alt_two() {
    let (g, pt) = alt_two_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_left_recursive() {
    let (g, pt) = left_recursive_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_right_recursive() {
    let (g, pt) = right_recursive_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_linear_chain() {
    let (g, pt) = linear_chain_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_tree() {
    let (g, pt) = tree_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_diamond() {
    let (g, pt) = diamond_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_five_token_chain() {
    let (g, pt) = five_token_chain_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_three_alt() {
    let (g, pt) = three_alt_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_precedence() {
    let (g, pt) = precedence_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_mixed() {
    let (g, pt) = mixed_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_wide_tree() {
    let (g, pt) = wide_tree_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compress_right_assoc() {
    let (g, pt) = right_assoc_grammar();
    compress_ok(&g, &pt);
}

#[test]
fn compressed_action_data_nonempty() {
    let (g, pt) = left_recursive_grammar();
    let ti = collect_token_indices(&g, &pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, false).unwrap();
    assert!(
        !compressed.action_table.data.is_empty(),
        "compressed action data should not be empty"
    );
}

#[test]
fn compressed_small_table_threshold_positive() {
    let (g, pt) = single_token_grammar();
    let ti = collect_token_indices(&g, &pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, false).unwrap();
    assert!(
        compressed.small_table_threshold > 0,
        "threshold must be positive"
    );
}

// =====================================================================
// 8. ABI builder integration
// =====================================================================

#[test]
fn abi_builder_produces_code_single_token() {
    let (g, pt) = single_token_grammar();
    let code = AbiLanguageBuilder::new(&g, &pt).generate();
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_produces_code_precedence() {
    let (g, pt) = precedence_grammar();
    let code = AbiLanguageBuilder::new(&g, &pt).generate();
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_with_compressed_tables_produces_code() {
    let (g, pt) = alt_two_grammar();
    let ti = collect_token_indices(&g, &pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, false).unwrap();
    let code = AbiLanguageBuilder::new(&g, &pt)
        .with_compressed_tables(&compressed)
        .generate();
    assert!(!code.is_empty());
}

// =====================================================================
// 9. NodeTypesGenerator via standalone API
// =====================================================================

#[test]
fn ntg_ok_single_token() {
    let (g, _) = single_token_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntg_ok_diamond() {
    let (g, _) = diamond_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntg_ok_precedence() {
    let (g, _) = precedence_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntg_ok_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// =====================================================================
// 10. Symbol name resolution
// =====================================================================

#[test]
fn rule_names_contain_start_symbol() {
    let (g, _) = single_token_grammar();
    let has_s = g.rule_names.values().any(|n| n == "s");
    assert!(has_s, "rule_names should contain 's'");
}

#[test]
fn rule_names_contain_all_nonterminals() {
    let (g, _) = tree_grammar();
    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"s"), "should contain 's'");
    assert!(names.contains(&"left"), "should contain 'left'");
    assert!(names.contains(&"right"), "should contain 'right'");
}

#[test]
fn token_names_present() {
    let (g, _) = seq_two_grammar();
    let token_names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(token_names.contains(&"a"));
    assert!(token_names.contains(&"b"));
}

#[test]
fn symbol_to_index_contains_eof() {
    let (_, pt) = single_token_grammar();
    assert!(
        pt.symbol_to_index.contains_key(&pt.eof_symbol),
        "symbol_to_index should contain the EOF symbol"
    );
}

#[test]
fn index_to_symbol_roundtrip() {
    let (_, pt) = left_recursive_grammar();
    for (&sym, &idx) in &pt.symbol_to_index {
        assert_eq!(
            pt.index_to_symbol[idx], sym,
            "index_to_symbol[idx] should map back to the original symbol"
        );
    }
}

// =====================================================================
// 11. Determinism: same input → same output
// =====================================================================

#[test]
fn deterministic_codegen_single_token() {
    let (g1, pt1) = single_token_grammar();
    let (g2, pt2) = single_token_grammar();
    let c1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_codegen_left_recursive() {
    let (g1, pt1) = left_recursive_grammar();
    let (g2, pt2) = left_recursive_grammar();
    let c1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_codegen_precedence() {
    let (g1, pt1) = precedence_grammar();
    let (g2, pt2) = precedence_grammar();
    let c1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_node_types_single_token() {
    let (g1, pt1) = single_token_grammar();
    let (g2, pt2) = single_token_grammar();
    let n1 = StaticLanguageGenerator::new(g1, pt1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, pt2).generate_node_types();
    assert_eq!(n1, n2);
}

#[test]
fn deterministic_node_types_diamond() {
    let (g1, pt1) = diamond_grammar();
    let (g2, pt2) = diamond_grammar();
    let n1 = StaticLanguageGenerator::new(g1, pt1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, pt2).generate_node_types();
    assert_eq!(n1, n2);
}

#[test]
fn deterministic_abi_builder() {
    let (g1, pt1) = alt_two_grammar();
    let (g2, pt2) = alt_two_grammar();
    let a1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let a2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(a1, a2);
}

#[test]
fn deterministic_compression() {
    let (g1, pt1) = left_recursive_grammar();
    let (g2, pt2) = left_recursive_grammar();
    let ti1 = collect_token_indices(&g1, &pt1);
    let ti2 = collect_token_indices(&g2, &pt2);
    let c1 = TableCompressor::new().compress(&pt1, &ti1, false).unwrap();
    let c2 = TableCompressor::new().compress(&pt2, &ti2, false).unwrap();
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
    assert_eq!(c1.small_table_threshold, c2.small_table_threshold);
}

#[test]
fn deterministic_state_count() {
    let (_, pt1) = five_token_chain_grammar();
    let (_, pt2) = five_token_chain_grammar();
    assert_eq!(pt1.state_count, pt2.state_count);
}

// =====================================================================
// 12. Edge cases
// =====================================================================

#[test]
fn many_alternatives_builds_and_compresses() {
    let (g, pt) = build_pipeline(|| {
        let mut b = GrammarBuilder::new("many_alt");
        for c in b'a'..=b'z' {
            let name = String::from(c as char);
            b = b.token(&name, &name);
        }
        for c in b'a'..=b'z' {
            let name = String::from(c as char);
            b = b.rule("s", vec![Box::leak(name.into_boxed_str()) as &str]);
        }
        b.start("s").build()
    });
    assert!(pt.state_count > 0);
    let ti = collect_token_indices(&g, &pt);
    assert!(TableCompressor::new().compress(&pt, &ti, false).is_ok());
}

#[test]
fn deeply_nested_chain_builds() {
    let (g, pt) = build_pipeline(|| {
        let mut b = GrammarBuilder::new("deep").token("x", "x");
        let names: Vec<String> = (0..10).map(|i| format!("n{i}")).collect();
        b = b.rule(&names[0], vec!["x"]);
        for i in 1..10 {
            let leaked: &'static str = Box::leak(names[i - 1].clone().into_boxed_str());
            b = b.rule(&names[i], vec![leaked]);
        }
        let last: &'static str = Box::leak(names[9].clone().into_boxed_str());
        b = b.rule("s", vec![last]);
        b.start("s").build()
    });
    assert!(pt.state_count > 0);
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn multi_operator_precedence_builds() {
    let (g, pt) = build_pipeline(|| {
        GrammarBuilder::new("multi_prec")
            .token("num", r"\d+")
            .token("plus", "+")
            .token("minus", "-")
            .token("star", "*")
            .token("slash", "SLASH")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence(
                "expr",
                vec!["expr", "minus", "expr"],
                1,
                Associativity::Left,
            )
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .rule_with_precedence(
                "expr",
                vec!["expr", "slash", "expr"],
                2,
                Associativity::Left,
            )
            .start("expr")
            .build()
    });
    assert!(pt.state_count > 0);
    compress_ok(&g, &pt);
}

#[test]
fn mixed_assoc_precedence_builds() {
    let (g, pt) = build_pipeline(|| {
        GrammarBuilder::new("mixed_assoc")
            .token("num", r"\d+")
            .token("plus", "+")
            .token("caret", "^")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence(
                "expr",
                vec!["expr", "caret", "expr"],
                2,
                Associativity::Right,
            )
            .start("expr")
            .build()
    });
    assert!(pt.state_count > 0);
    compress_ok(&g, &pt);
}

// =====================================================================
// 13. Token indices helper properties
// =====================================================================

#[test]
fn token_indices_sorted_and_deduped() {
    let (g, pt) = precedence_grammar();
    let ti = collect_token_indices(&g, &pt);
    let mut sorted = ti.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ti, sorted, "token indices must be sorted and unique");
}

#[test]
fn token_indices_nonempty() {
    let (g, pt) = single_token_grammar();
    let ti = collect_token_indices(&g, &pt);
    assert!(!ti.is_empty(), "should contain at least EOF");
}

#[test]
fn token_indices_includes_eof() {
    let (g, pt) = single_token_grammar();
    let ti = collect_token_indices(&g, &pt);
    if let Some(&eof_idx) = pt.symbol_to_index.get(&pt.eof_symbol) {
        assert!(
            ti.contains(&eof_idx),
            "token indices should contain EOF index"
        );
    }
}

// =====================================================================
// 14. Parse table structure invariants
// =====================================================================

#[test]
fn eof_symbol_in_symbol_to_index() {
    let (_, pt) = tree_grammar();
    assert!(pt.symbol_to_index.contains_key(&pt.eof_symbol));
}

#[test]
fn start_symbol_set() {
    let (g, pt) = single_token_grammar();
    // The start symbol should correspond to one of the grammar's rule LHS symbols
    let start = pt.start_symbol;
    let rule_lhs_ids: Vec<_> = g.rules.keys().copied().collect();
    assert!(
        rule_lhs_ids.contains(&start),
        "start symbol should be a rule LHS"
    );
}

#[test]
fn lex_modes_length_matches_state_count() {
    let (_, pt) = left_recursive_grammar();
    assert_eq!(pt.lex_modes.len(), pt.state_count);
}

#[test]
fn nonterminal_to_index_covers_all_rules() {
    let (g, pt) = tree_grammar();
    for &lhs in g.rules.keys() {
        assert!(
            pt.nonterminal_to_index.contains_key(&lhs),
            "nonterminal_to_index should contain LHS symbol {:?}",
            lhs
        );
    }
}

// =====================================================================
// 15. Grammar topology: different shapes all produce valid output
// =====================================================================

#[test]
fn topology_linear_chain_full_pipeline() {
    let (g, pt) = linear_chain_grammar();
    let slg = StaticLanguageGenerator::new(g.clone(), pt.clone());
    assert!(!slg.generate_language_code().is_empty());
    assert!(!slg.generate_node_types().is_empty());
    compress_ok(&g, &pt);
}

#[test]
fn topology_tree_full_pipeline() {
    let (g, pt) = tree_grammar();
    let slg = StaticLanguageGenerator::new(g.clone(), pt.clone());
    assert!(!slg.generate_language_code().is_empty());
    assert!(!slg.generate_node_types().is_empty());
    compress_ok(&g, &pt);
}

#[test]
fn topology_diamond_full_pipeline() {
    let (g, pt) = diamond_grammar();
    let slg = StaticLanguageGenerator::new(g.clone(), pt.clone());
    assert!(!slg.generate_language_code().is_empty());
    assert!(!slg.generate_node_types().is_empty());
    compress_ok(&g, &pt);
}

#[test]
fn topology_wide_tree_full_pipeline() {
    let (g, pt) = wide_tree_grammar();
    let slg = StaticLanguageGenerator::new(g.clone(), pt.clone());
    assert!(!slg.generate_language_code().is_empty());
    assert!(!slg.generate_node_types().is_empty());
    compress_ok(&g, &pt);
}
