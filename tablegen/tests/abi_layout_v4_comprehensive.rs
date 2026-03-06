#![allow(clippy::needless_range_loop)]

//! Comprehensive v4 ABI-layout tests for `AbiLanguageBuilder` covering
//! construction, output properties, determinism, grammar topologies,
//! field validation, ABI compatibility, complex grammars, and edge cases.
//!
//! Target: 55+ tests exercising the public API via the full pipeline
//! (GrammarBuilder → FirstFollowSets → build_lr1_automaton → AbiLanguageBuilder).

use adze_glr_core::ParseTable;
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::AbiLanguageBuilder;

// ===========================================================================
// Helpers
// ===========================================================================

/// Full pipeline: GrammarBuilder → normalize → FIRST/FOLLOW → LR(1) → ABI code.
fn pipeline(builder: GrammarBuilder) -> (String, Grammar, ParseTable) {
    let mut grammar = builder.build();
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) build failed");
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    (code, grammar, table)
}

/// Shorthand: build grammar + table only.
fn build_gt(builder: GrammarBuilder) -> (Grammar, ParseTable) {
    let mut grammar = builder.build();
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) build failed");
    (grammar, table)
}

/// Generate ABI code string from grammar + table.
fn gen_code(g: &Grammar, t: &ParseTable) -> String {
    AbiLanguageBuilder::new(g, t).generate().to_string()
}

/// Simple single-token grammar: S → x
fn single_token(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
}

/// Two-alternative grammar: S → a | b
fn two_alt(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
}

/// Sequence grammar: S → a b
fn sequence(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
}

/// Chain grammar: S → A, A → x
fn chain(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("A", vec!["x"])
        .rule("S", vec!["A"])
        .start("S")
}

/// Left-recursive grammar: S → x | S x
fn left_rec(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("S", vec!["x"])
        .rule("S", vec!["S", "x"])
        .start("S")
}

/// Right-recursive grammar: S → x | x S
fn right_rec(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("S", vec!["x"])
        .rule("S", vec!["x", "S"])
        .start("S")
}

/// Wide grammar: S → t0 | t1 | ... | t_{n-1}
fn wide(name: &str, n: usize) -> GrammarBuilder {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok = format!("t{i}");
        let pat = format!("{i}");
        b = b.token(&tok, &pat);
    }
    for i in 0..n {
        let tok = format!("t{i}");
        b = b.rule("S", vec![&tok]);
    }
    b.start("S")
}

// ===========================================================================
// 1. ABI builder construction (8 tests)
// ===========================================================================

#[test]
fn construct_single_token() {
    let (g, t) = build_gt(single_token("c1"));
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_two_alternatives() {
    let (g, t) = build_gt(two_alt("c2"));
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_sequence() {
    let (g, t) = build_gt(sequence("c3"));
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_chain() {
    let (g, t) = build_gt(chain("c4"));
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_left_recursive() {
    let (g, t) = build_gt(left_rec("c5"));
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_right_recursive() {
    let (g, t) = build_gt(right_rec("c6"));
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_wide() {
    let (g, t) = build_gt(wide("c7", 10));
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_multiple_builders_same_input() {
    let (g, t) = build_gt(single_token("c8"));
    let _b1 = AbiLanguageBuilder::new(&g, &t);
    let _b2 = AbiLanguageBuilder::new(&g, &t);
    let _b3 = AbiLanguageBuilder::new(&g, &t);
}

// ===========================================================================
// 2. ABI output properties (8 tests)
// ===========================================================================

#[test]
fn output_is_nonempty() {
    let (code, _, _) = pipeline(single_token("op1"));
    assert!(!code.is_empty());
}

#[test]
fn output_contains_language_static() {
    let (code, _, _) = pipeline(single_token("op2"));
    assert!(code.contains("LANGUAGE"), "must have LANGUAGE static");
}

#[test]
fn output_contains_tslanguage_type() {
    let (code, _, _) = pipeline(single_token("op3"));
    assert!(code.contains("TSLanguage"), "must reference TSLanguage");
}

#[test]
fn output_contains_symbol_count_field() {
    let (code, _, _) = pipeline(single_token("op4"));
    assert!(code.contains("symbol_count"));
}

#[test]
fn output_contains_state_count_field() {
    let (code, _, _) = pipeline(single_token("op5"));
    assert!(code.contains("state_count"));
}

#[test]
fn output_contains_parse_actions() {
    let (code, _, _) = pipeline(single_token("op6"));
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn output_contains_symbol_metadata() {
    let (code, _, _) = pipeline(single_token("op7"));
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn output_has_substantial_length() {
    let (code, _, _) = pipeline(single_token("op8"));
    assert!(
        code.len() > 200,
        "generated code should be substantial, got {} bytes",
        code.len()
    );
}

// ===========================================================================
// 3. ABI determinism (5 tests)
// ===========================================================================

#[test]
fn determinism_single_token() {
    let (g, t) = build_gt(single_token("d1"));
    assert_eq!(gen_code(&g, &t), gen_code(&g, &t));
}

#[test]
fn determinism_two_alt() {
    let (g, t) = build_gt(two_alt("d2"));
    assert_eq!(gen_code(&g, &t), gen_code(&g, &t));
}

#[test]
fn determinism_left_recursive() {
    let (g, t) = build_gt(left_rec("d3"));
    assert_eq!(gen_code(&g, &t), gen_code(&g, &t));
}

#[test]
fn determinism_wide_grammar() {
    let (g, t) = build_gt(wide("d4", 8));
    assert_eq!(gen_code(&g, &t), gen_code(&g, &t));
}

#[test]
fn determinism_three_consecutive() {
    let (g, t) = build_gt(single_token("d5"));
    let a = gen_code(&g, &t);
    let b = gen_code(&g, &t);
    let c = gen_code(&g, &t);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ===========================================================================
// 4. Various grammar topologies (10 tests)
// ===========================================================================

#[test]
fn topology_single_rule() {
    let (code, _, _) = pipeline(single_token("tp1"));
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn topology_two_alternatives() {
    let (code, _, _) = pipeline(two_alt("tp2"));
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn topology_sequence_two_tokens() {
    let (code, _, _) = pipeline(sequence("tp3"));
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn topology_chain_nonterminals() {
    let (code, _, _) = pipeline(
        GrammarBuilder::new("tp4")
            .token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("S", vec!["A"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("TS_RULES"));
}

#[test]
fn topology_left_recursive() {
    let (code, _, table) = pipeline(left_rec("tp5"));
    assert!(code.contains("LANGUAGE"));
    assert!(
        table.state_count > 1,
        "recursive grammar needs multiple states"
    );
}

#[test]
fn topology_right_recursive() {
    let (code, _, table) = pipeline(right_rec("tp6"));
    assert!(code.contains("LANGUAGE"));
    assert!(table.state_count > 1);
}

#[test]
fn topology_diamond() {
    // S → A | B, A → x, B → x
    let (code, _, _) = pipeline(
        GrammarBuilder::new("tp7")
            .token("x", "x")
            .rule("A", vec!["x"])
            .rule("B", vec!["x"])
            .rule("S", vec!["A"])
            .rule("S", vec!["B"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn topology_wide_choice() {
    let (g, t) = build_gt(wide("tp8", 6));
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(t.symbol_count >= 6, "wide grammar must have enough symbols");
}

#[test]
fn topology_binary_tree() {
    // S → A B, A → x, B → y
    let (code, _, _) = pipeline(
        GrammarBuilder::new("tp9")
            .token("x", "x")
            .token("y", "y")
            .rule("A", vec!["x"])
            .rule("B", vec!["y"])
            .rule("S", vec!["A", "B"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn topology_nested_nonterminals() {
    // S → A, A → B, B → C, C → x
    let (code, _, _) = pipeline(
        GrammarBuilder::new("tp10")
            .token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("S", vec!["A"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

// ===========================================================================
// 5. ABI field validation (8 tests)
// ===========================================================================

#[test]
fn field_symbol_names_array() {
    let (code, _, _) = pipeline(single_token("fv1"));
    assert!(
        code.contains("SYMBOL_NAME_"),
        "must have symbol name statics"
    );
    assert!(
        code.contains("SYMBOL_NAME_PTRS"),
        "must have SYMBOL_NAME_PTRS"
    );
}

#[test]
fn field_public_symbol_map() {
    let (code, _, _) = pipeline(single_token("fv2"));
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn field_primary_state_ids() {
    let (code, _, _) = pipeline(single_token("fv3"));
    assert!(code.contains("PRIMARY_STATE_IDS"));
}

#[test]
fn field_production_id_map() {
    let (code, _, _) = pipeline(single_token("fv4"));
    assert!(code.contains("PRODUCTION_ID_MAP"));
}

#[test]
fn field_production_lhs_index() {
    let (code, _, _) = pipeline(single_token("fv5"));
    assert!(code.contains("PRODUCTION_LHS_INDEX"));
}

#[test]
fn field_lex_modes() {
    let (code, _, _) = pipeline(single_token("fv6"));
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn field_small_parse_table_and_map() {
    let (code, _, _) = pipeline(single_token("fv7"));
    assert!(code.contains("SMALL_PARSE_TABLE"));
    assert!(code.contains("SMALL_PARSE_TABLE_MAP"));
}

#[test]
fn field_map_slices_and_entries() {
    let (code, _, _) = pipeline(single_token("fv8"));
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

// ===========================================================================
// 6. ABI compatibility (5 tests)
// ===========================================================================

#[test]
fn compat_version_field_present() {
    let (code, _, _) = pipeline(single_token("cp1"));
    assert!(code.contains("version"), "must have version field");
}

#[test]
fn compat_eof_symbol_field() {
    let (code, _, _) = pipeline(single_token("cp2"));
    assert!(code.contains("eof_symbol"), "must have eof_symbol");
}

#[test]
fn compat_production_count_field() {
    let (code, _, _) = pipeline(single_token("cp3"));
    assert!(
        code.contains("production_count") || code.contains("production_id_count"),
        "must contain production count"
    );
}

#[test]
fn compat_lexer_fn_reference() {
    let (code, _, _) = pipeline(single_token("cp4"));
    assert!(code.contains("lexer_fn"), "must reference lexer_fn");
}

#[test]
fn compat_different_names_produce_different_fns() {
    let (g1, t1) = build_gt(single_token("alpha"));
    let (g2, t2) = build_gt(single_token("beta"));
    let code1 = gen_code(&g1, &t1);
    let code2 = gen_code(&g2, &t2);
    assert!(code1.contains("tree_sitter_alpha"));
    assert!(code2.contains("tree_sitter_beta"));
    assert!(!code1.contains("tree_sitter_beta"));
}

// ===========================================================================
// 7. Complex grammars (5 tests)
// ===========================================================================

#[test]
fn complex_three_level_chain() {
    let (code, _, _) = pipeline(
        GrammarBuilder::new("cx1")
            .token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("S", vec!["A"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("TS_RULES"));
}

#[test]
fn complex_multiple_terminals_per_rule() {
    let (code, _, _) = pipeline(
        GrammarBuilder::new("cx2")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("S", vec!["a", "b", "c"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn complex_mixed_terminals_and_nonterminals() {
    let (code, _, _) = pipeline(
        GrammarBuilder::new("cx3")
            .token("x", "x")
            .token("y", "y")
            .rule("A", vec!["x"])
            .rule("S", vec!["A", "y"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn complex_wide_with_many_alternatives() {
    let (g, t) = build_gt(wide("cx4", 12));
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(t.symbol_count >= 12);
}

#[test]
fn complex_recursive_with_alternatives() {
    // S → a | b | S a | S b
    let (code, _, table) = pipeline(
        GrammarBuilder::new("cx5")
            .token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .rule("S", vec!["S", "a"])
            .rule("S", vec!["S", "b"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    assert!(table.state_count > 1);
}

// ===========================================================================
// 8. Edge cases (6 tests)
// ===========================================================================

#[test]
fn edge_single_char_token() {
    let (code, _, _) = pipeline(
        GrammarBuilder::new("e1")
            .token("z", "z")
            .rule("S", vec!["z"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn edge_multi_char_token() {
    let (code, _, _) = pipeline(
        GrammarBuilder::new("e2")
            .token("hello", "hello")
            .rule("S", vec!["hello"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn edge_symbol_count_matches_table() {
    let (g, t) = build_gt(two_alt("e3"));
    let code = gen_code(&g, &t);
    // The symbol_count value from the table should appear somewhere in the output
    let count_val = format!("{}", t.symbol_count);
    let count_u32 = format!("{}u32", t.symbol_count);
    let count_space = format!("{} u32", t.symbol_count);
    assert!(
        code.contains(&count_val) || code.contains(&count_u32) || code.contains(&count_space),
        "symbol_count value {} must appear in generated code",
        t.symbol_count
    );
}

#[test]
fn edge_wide_grammar_all_symbol_names() {
    let (g, t) = build_gt(wide("e4", 5));
    let code = gen_code(&g, &t);
    for i in 0..t.symbol_count {
        let expected = format!("SYMBOL_NAME_{i}");
        assert!(
            code.contains(&expected),
            "missing {} in generated code",
            expected
        );
    }
}

#[test]
fn edge_recursive_more_states_than_simple() {
    let (_, _, t_simple) = pipeline(single_token("e5a"));
    let (_, _, t_rec) = pipeline(left_rec("e5b"));
    assert!(
        t_rec.state_count >= t_simple.state_count,
        "recursive grammar should have at least as many states"
    );
}

#[test]
fn edge_chain_at_least_as_many_symbols_as_single() {
    let (_, _, t_single) = pipeline(single_token("e6a"));
    let (_, _, t_chain) = pipeline(chain("e6b"));
    assert!(
        t_chain.symbol_count >= t_single.symbol_count,
        "chain grammar should have at least as many symbols"
    );
}
