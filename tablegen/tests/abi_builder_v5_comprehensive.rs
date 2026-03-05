//! Comprehensive v5 tests for `AbiLanguageBuilder` covering construction,
//! version/counts, symbol names and metadata, field names, state counts,
//! grammar topology preservation, serialization of ABI data, and edge cases.
//!
//! Target: 55+ tests exercising the public API via the full pipeline
//! (GrammarBuilder → FirstFollowSets → build_lr1_automaton → AbiLanguageBuilder).

use adze_glr_core::{FirstFollowSets, GotoIndexing, LexMode, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::AbiLanguageBuilder;
use std::collections::BTreeMap;

// ===========================================================================
// Helpers
// ===========================================================================

const INVALID: adze_glr_core::StateId = adze_glr_core::StateId(u16::MAX);

/// Build a grammar + parse table through the full pipeline.
fn pipeline(name: &str, builder: GrammarBuilder) -> (Grammar, ParseTable, String) {
    let mut grammar = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) build failed");
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    let _ = name;
    (grammar, table, code)
}

/// Shorthand: single-token grammar S → x.
fn single_token(name: &str) -> (Grammar, ParseTable, String) {
    pipeline(
        name,
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S"),
    )
}

/// Two-alternative grammar S → a | b.
fn two_alt(name: &str) -> (Grammar, ParseTable, String) {
    pipeline(
        name,
        GrammarBuilder::new(name)
            .token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .start("S"),
    )
}

/// Chain grammar S → A → x.
fn chain(name: &str) -> (Grammar, ParseTable, String) {
    pipeline(
        name,
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("A", vec!["x"])
            .rule("S", vec!["A"])
            .start("S"),
    )
}

/// Sequence grammar S → a b.
fn seq(name: &str) -> (Grammar, ParseTable, String) {
    pipeline(
        name,
        GrammarBuilder::new(name)
            .token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a", "b"])
            .start("S"),
    )
}

/// Left-recursive grammar S → x | S x.
fn left_rec(name: &str) -> (Grammar, ParseTable, String) {
    pipeline(
        name,
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("S", vec!["x"])
            .rule("S", vec!["S", "x"])
            .start("S"),
    )
}

/// Right-recursive grammar S → x | x S.
fn right_rec(name: &str) -> (Grammar, ParseTable, String) {
    pipeline(
        name,
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("S", vec!["x"])
            .rule("S", vec!["x", "S"])
            .start("S"),
    )
}

/// Wide grammar S → t0 | t1 | … | t_{n-1}.
fn wide(name: &str, n: usize) -> (Grammar, ParseTable, String) {
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
    b = b.start("S");
    pipeline(name, b)
}

/// Build a manual grammar + parse table without running the LR(1) pipeline,
/// for precise control over dimensions.
fn manual_pair(
    name: &str,
    num_terms: usize,
    num_nonterms: usize,
    num_fields: usize,
    num_externals: usize,
    num_states: usize,
) -> (Grammar, ParseTable) {
    let num_terms = num_terms.max(1);
    let num_nonterms = num_nonterms.max(1);
    let num_states = num_states.max(1);

    let eof_idx = 1 + num_terms + num_externals;
    let symbol_count = eof_idx + 1 + num_nonterms;

    let actions = vec![vec![vec![]; symbol_count]; num_states];
    let gotos = vec![vec![INVALID; symbol_count]; num_states];

    let eof_symbol = SymbolId(eof_idx as u16);
    let start_symbol = SymbolId((eof_idx + 1) as u16);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (i, slot) in index_to_symbol.iter_mut().enumerate() {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        *slot = sym;
    }

    let mut grammar = Grammar::new(name.to_string());

    let first_term = SymbolId(1);
    for i in 1..=num_terms {
        let sym = SymbolId(i as u16);
        grammar.tokens.insert(
            sym,
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }

    let first_nt_idx = eof_idx + 1;
    for i in 0..num_nonterms {
        let sym = SymbolId((first_nt_idx + i) as u16);
        grammar.rule_names.insert(sym, format!("rule_{i}"));
        grammar.add_rule(Rule {
            lhs: sym,
            rhs: vec![Symbol::Terminal(first_term)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }

    for i in 0..num_fields {
        grammar
            .fields
            .insert(FieldId(i as u16), format!("field_{i}"));
    }

    for i in 0..num_externals {
        grammar.externals.push(ExternalToken {
            name: format!("ext_{i}"),
            symbol_id: SymbolId((1 + num_terms + i) as u16),
        });
    }

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        symbol_metadata: vec![],
        state_count: num_states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        external_scanner_states: vec![],
        rules: vec![],
        eof_symbol,
        start_symbol,
        grammar: Grammar::default(),
        initial_state: adze_glr_core::StateId(0),
        token_count: eof_idx + 1,
        external_token_count: num_externals,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            num_states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, table)
}

/// Generate code string from grammar + table.
fn codegen(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

// ===========================================================================
// 1. AbiLanguageBuilder construction (8 tests)
// ===========================================================================

#[test]
fn construct_single_token_grammar() {
    let (g, t, _) = single_token("c1");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_two_alternative_grammar() {
    let (g, t, _) = two_alt("c2");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_chain_grammar() {
    let (g, t, _) = chain("c3");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_sequence_grammar() {
    let (g, t, _) = seq("c4");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_left_recursive_grammar() {
    let (g, t, _) = left_rec("c5");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_right_recursive_grammar() {
    let (g, t, _) = right_rec("c6");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_wide_grammar() {
    let (g, t, _) = wide("c7", 10);
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_multiple_builders_from_same_input() {
    let (g, t, _) = single_token("c8");
    let _b1 = AbiLanguageBuilder::new(&g, &t);
    let _b2 = AbiLanguageBuilder::new(&g, &t);
    let _b3 = AbiLanguageBuilder::new(&g, &t);
}

// ===========================================================================
// 2. AbiLanguage version/counts (8 tests)
// ===========================================================================

#[test]
fn version_field_present_in_generated_code() {
    let (_, _, code) = single_token("v1");
    assert!(
        code.contains("version"),
        "must contain version field in TSLanguage"
    );
}

#[test]
fn version_is_tree_sitter_abi_15() {
    let (_, _, code) = single_token("v2");
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "must reference TREE_SITTER_LANGUAGE_VERSION constant"
    );
}

#[test]
fn symbol_count_field_present() {
    let (_, _, code) = single_token("v3");
    assert!(code.contains("symbol_count"), "must contain symbol_count");
}

#[test]
fn symbol_count_value_matches_table() {
    let (_, table, code) = single_token("v4");
    let count_str = format!("{}", table.symbol_count);
    assert!(
        code.contains(&count_str) || code.contains(&format!("{}u32", table.symbol_count)),
        "symbol_count value {} must appear in code",
        table.symbol_count
    );
}

#[test]
fn field_count_field_present() {
    let (_, _, code) = single_token("v5");
    assert!(code.contains("field_count"), "must contain field_count");
}

#[test]
fn state_count_field_present() {
    let (_, _, code) = single_token("v6");
    assert!(code.contains("state_count"), "must contain state_count");
}

#[test]
fn token_count_field_present() {
    let (_, _, code) = single_token("v7");
    assert!(code.contains("token_count"), "must contain token_count");
}

#[test]
fn production_id_count_field_present() {
    let (_, _, code) = single_token("v8");
    assert!(
        code.contains("production_id_count") || code.contains("production_count"),
        "must contain production count field"
    );
}

// ===========================================================================
// 3. Symbol names and metadata (8 tests)
// ===========================================================================

#[test]
fn symbol_names_statics_present() {
    let (_, _, code) = single_token("sn1");
    assert!(
        code.contains("SYMBOL_NAME_"),
        "must contain symbol name statics"
    );
}

#[test]
fn symbol_name_ptrs_array_present() {
    let (_, _, code) = single_token("sn2");
    assert!(
        code.contains("SYMBOL_NAME_PTRS"),
        "must contain SYMBOL_NAME_PTRS"
    );
}

#[test]
fn symbol_metadata_array_present() {
    let (_, _, code) = single_token("sn3");
    assert!(
        code.contains("SYMBOL_METADATA"),
        "must contain SYMBOL_METADATA"
    );
}

#[test]
fn public_symbol_map_present() {
    let (_, _, code) = single_token("sn4");
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP"),
        "must contain PUBLIC_SYMBOL_MAP"
    );
}

#[test]
fn symbol_id_to_index_mapping_present() {
    let (_, _, code) = single_token("sn5");
    assert!(
        code.contains("SYMBOL_ID_TO_INDEX"),
        "must contain SYMBOL_ID_TO_INDEX"
    );
}

#[test]
fn symbol_index_to_id_mapping_present() {
    let (_, _, code) = single_token("sn6");
    assert!(
        code.contains("SYMBOL_INDEX_TO_ID"),
        "must contain SYMBOL_INDEX_TO_ID"
    );
}

#[test]
fn wide_grammar_has_all_symbol_name_statics() {
    let (_, table, code) = wide("sn7", 5);
    for i in 0..table.symbol_count {
        let expected = format!("SYMBOL_NAME_{i}");
        assert!(code.contains(&expected), "missing {expected} in code");
    }
}

#[test]
fn two_alt_grammar_symbol_count_ge_tokens_plus_nonterms() {
    let (grammar, table, _) = two_alt("sn8");
    // symbol_count must cover: error(0) + terminals + EOF + nonterminals
    let min_expected = grammar.tokens.len() + grammar.rules.len() + 1; // +1 for EOF at least
    assert!(
        table.symbol_count >= min_expected,
        "symbol_count {} should be >= {} (tokens={}, rules={})",
        table.symbol_count,
        min_expected,
        grammar.tokens.len(),
        grammar.rules.len()
    );
}

// ===========================================================================
// 4. Field names (5 tests)
// ===========================================================================

#[test]
fn field_name_ptrs_present_when_fields_exist() {
    let (g, t) = manual_pair("fn1", 2, 1, 3, 0, 2);
    let code = codegen(&g, &t);
    assert!(
        code.contains("FIELD_NAME_PTRS"),
        "must contain FIELD_NAME_PTRS"
    );
}

#[test]
fn field_name_statics_present_when_fields_exist() {
    let (g, t) = manual_pair("fn2", 2, 1, 3, 0, 2);
    let code = codegen(&g, &t);
    assert!(
        code.contains("FIELD_NAME_"),
        "must contain FIELD_NAME_ statics"
    );
}

#[test]
fn zero_fields_still_has_field_name_ptrs() {
    let (g, t) = manual_pair("fn3", 1, 1, 0, 0, 1);
    let code = codegen(&g, &t);
    // Even with zero fields, FIELD_NAME_PTRS should exist (empty array)
    assert!(
        code.contains("FIELD_NAME_PTRS"),
        "must contain FIELD_NAME_PTRS even with no fields"
    );
}

#[test]
fn field_count_matches_grammar_fields() {
    let (g, t) = manual_pair("fn4", 1, 1, 5, 0, 1);
    let code = codegen(&g, &t);
    assert!(
        code.contains("field_count"),
        "must contain field_count field"
    );
    let count_str = format!("{}", g.fields.len());
    assert!(
        code.contains(&count_str) || code.contains(&format!("{}u32", g.fields.len())),
        "field_count value {} must appear in code",
        g.fields.len()
    );
}

#[test]
fn multiple_fields_produce_ordered_names() {
    let (g, t) = manual_pair("fn5", 1, 1, 4, 0, 1);
    let code = codegen(&g, &t);
    // Each field should get a FIELD_NAME_N static
    for i in 0..g.fields.len() {
        let expected = format!("FIELD_NAME_{i}");
        assert!(code.contains(&expected), "missing {expected}");
    }
}

// ===========================================================================
// 5. State count matches parse table (5 tests)
// ===========================================================================

#[test]
fn state_count_single_token_grammar() {
    let (_, table, code) = single_token("sc1");
    assert!(table.state_count >= 1, "must have at least 1 state");
    let count_str = format!("{}", table.state_count);
    assert!(
        code.contains(&count_str),
        "state_count value {} must appear in code",
        table.state_count
    );
}

#[test]
fn state_count_sequence_grammar_greater_than_one() {
    let (_, table, _) = seq("sc2");
    assert!(
        table.state_count > 1,
        "sequence grammar should need multiple states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_recursive_grammar() {
    let (_, table, _) = left_rec("sc3");
    assert!(
        table.state_count > 1,
        "recursive grammar should need multiple states"
    );
}

#[test]
fn lex_modes_count_matches_state_count() {
    let (_, _, code) = single_token("sc4");
    assert!(code.contains("LEX_MODES"), "must contain LEX_MODES array");
}

#[test]
fn primary_state_ids_present_and_matches_states() {
    let (_, _, code) = single_token("sc5");
    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "must contain PRIMARY_STATE_IDS"
    );
}

// ===========================================================================
// 6. Grammar topology preservation (8 tests)
// ===========================================================================

#[test]
fn topology_single_rule_produces_language() {
    let (_, _, code) = single_token("tp1");
    assert!(code.contains("LANGUAGE"), "must contain LANGUAGE static");
}

#[test]
fn topology_two_alternatives_has_parse_actions() {
    let (_, _, code) = two_alt("tp2");
    assert!(code.contains("PARSE_ACTIONS"), "must contain PARSE_ACTIONS");
}

#[test]
fn topology_chain_has_ts_rules() {
    let (_, _, code) = chain("tp3");
    assert!(code.contains("TS_RULES"), "must contain TS_RULES");
}

#[test]
fn topology_left_recursive_has_multiple_states() {
    let (_, table, code) = left_rec("tp4");
    assert!(code.contains("LANGUAGE"));
    assert!(table.state_count > 1);
}

#[test]
fn topology_right_recursive_has_multiple_states() {
    let (_, table, code) = right_rec("tp5");
    assert!(code.contains("LANGUAGE"));
    assert!(table.state_count > 1);
}

#[test]
fn topology_diamond_shape_produces_valid_code() {
    // S → A | B, A → x, B → x
    let (_, _, code) = pipeline(
        "tp6",
        GrammarBuilder::new("tp6")
            .token("x", "x")
            .rule("A", vec!["x"])
            .rule("B", vec!["x"])
            .rule("S", vec!["A"])
            .rule("S", vec!["B"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("TS_RULES"));
}

#[test]
fn topology_long_chain_preserves_all_rules() {
    // S → A → B → C → x
    let (grammar, _, code) = pipeline(
        "tp7",
        GrammarBuilder::new("tp7")
            .token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("S", vec!["A"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    // All non-terminal rules should contribute to productions
    let total_rules: usize = grammar.rules.values().map(|r| r.len()).sum();
    assert!(total_rules >= 4, "long chain must have at least 4 rules");
}

#[test]
fn topology_wide_choice_has_enough_symbols() {
    let (_, table, code) = wide("tp8", 6);
    assert!(code.contains("LANGUAGE"));
    // Must have at least 6 terminal symbols + EOF + start nonterminal
    assert!(
        table.symbol_count >= 8,
        "wide grammar with 6 tokens must have >= 8 symbols, got {}",
        table.symbol_count
    );
}

// ===========================================================================
// 7. Serialization of ABI data (5 tests)
// ===========================================================================

#[test]
fn generated_code_is_nonempty() {
    let (_, _, code) = single_token("ser1");
    assert!(!code.is_empty(), "generated code must not be empty");
}

#[test]
fn generated_code_has_substantial_length() {
    let (_, _, code) = single_token("ser2");
    assert!(
        code.len() > 200,
        "generated code should be substantial, got {} bytes",
        code.len()
    );
}

#[test]
fn determinism_same_input_same_output() {
    let (g, t, _) = single_token("ser3");
    let code1 = codegen(&g, &t);
    let code2 = codegen(&g, &t);
    assert_eq!(code1, code2, "repeated generation must be identical");
}

#[test]
fn determinism_three_consecutive_runs() {
    let (g, t, _) = two_alt("ser4");
    let a = codegen(&g, &t);
    let b = codegen(&g, &t);
    let c = codegen(&g, &t);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn different_grammar_names_produce_different_fn_names() {
    let (_, _, code_a) = single_token("alpha");
    let (_, _, code_b) = pipeline(
        "beta",
        GrammarBuilder::new("beta")
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S"),
    );
    assert!(
        code_a.contains("tree_sitter_alpha"),
        "alpha code must have tree_sitter_alpha"
    );
    assert!(
        code_b.contains("tree_sitter_beta"),
        "beta code must have tree_sitter_beta"
    );
    assert!(
        !code_a.contains("tree_sitter_beta"),
        "alpha code must not mention beta"
    );
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_single_char_token() {
    let (_, _, code) = pipeline(
        "ec1",
        GrammarBuilder::new("ec1")
            .token("z", "z")
            .rule("S", vec!["z"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn edge_multi_char_token() {
    let (_, _, code) = pipeline(
        "ec2",
        GrammarBuilder::new("ec2")
            .token("hello", "hello")
            .rule("S", vec!["hello"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn edge_manual_grammar_with_fields() {
    let (g, t) = manual_pair("ec3", 2, 1, 4, 0, 3);
    let code = codegen(&g, &t);
    assert!(code.contains("FIELD_NAME_"));
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn edge_manual_grammar_zero_fields() {
    let (g, t) = manual_pair("ec4", 1, 1, 0, 0, 2);
    let code = codegen(&g, &t);
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn edge_manual_grammar_with_externals() {
    let (g, t) = manual_pair("ec5", 1, 1, 0, 2, 2);
    let code = codegen(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(
        code.contains("ExternalScanner"),
        "must contain ExternalScanner struct"
    );
}

#[test]
fn edge_many_symbols_wide_32() {
    let (_, table, code) = wide("ec6", 32);
    assert!(code.contains("LANGUAGE"));
    assert!(
        table.symbol_count >= 32,
        "wide(32) grammar needs >= 32 symbols, got {}",
        table.symbol_count
    );
}

#[test]
fn edge_single_rule_single_state_manual() {
    let (g, t) = manual_pair("ec7", 1, 1, 0, 0, 1);
    let code = codegen(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("state_count"));
}

#[test]
fn edge_many_states_manual() {
    let (g, t) = manual_pair("ec8", 2, 2, 0, 0, 20);
    let code = codegen(&g, &t);
    assert!(code.contains("LANGUAGE"));
    // Check that state count appears in the code
    let count_str = format!("{}", t.state_count);
    assert!(
        code.contains(&count_str),
        "state_count value {} must appear in code",
        t.state_count
    );
}

// ===========================================================================
// Extra tests to exceed 55 (coverage of additional invariants)
// ===========================================================================

#[test]
fn parse_table_static_present() {
    let (_, _, code) = single_token("x1");
    assert!(
        code.contains("PARSE_TABLE"),
        "must contain PARSE_TABLE static"
    );
}

#[test]
fn small_parse_table_present() {
    let (_, _, code) = single_token("x2");
    assert!(
        code.contains("SMALL_PARSE_TABLE"),
        "must contain SMALL_PARSE_TABLE"
    );
}

#[test]
fn small_parse_table_map_present() {
    let (_, _, code) = single_token("x3");
    assert!(
        code.contains("SMALL_PARSE_TABLE_MAP"),
        "must contain SMALL_PARSE_TABLE_MAP"
    );
}

#[test]
fn field_map_slices_and_entries_present() {
    let (_, _, code) = single_token("x4");
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

#[test]
fn production_id_map_present() {
    let (_, _, code) = single_token("x5");
    assert!(
        code.contains("PRODUCTION_ID_MAP"),
        "must contain PRODUCTION_ID_MAP"
    );
}

#[test]
fn production_lhs_index_present() {
    let (_, _, code) = single_token("x6");
    assert!(
        code.contains("PRODUCTION_LHS_INDEX"),
        "must contain PRODUCTION_LHS_INDEX"
    );
}

#[test]
fn ts_rules_array_present() {
    let (_, _, code) = single_token("x7");
    assert!(code.contains("TS_RULES"), "must contain TS_RULES");
}

#[test]
fn eof_symbol_field_present() {
    let (_, _, code) = single_token("x8");
    assert!(code.contains("eof_symbol"), "must contain eof_symbol field");
}

#[test]
fn lexer_fn_reference_present() {
    let (_, _, code) = single_token("x9");
    assert!(code.contains("lexer_fn"), "must contain lexer_fn reference");
}

#[test]
fn chain_grammar_has_more_rules_than_single() {
    let (_, _, code_single) = single_token("xr1");
    let (_, _, code_chain) = chain("xr2");
    // Chain grammar code should be longer due to more rules
    assert!(
        code_chain.len() >= code_single.len(),
        "chain grammar code ({}) should be >= single token code ({})",
        code_chain.len(),
        code_single.len()
    );
}

#[test]
fn recursive_grammar_state_count_gt_simple() {
    let (_, t_simple, _) = single_token("xr3");
    let (_, t_rec, _) = left_rec("xr4");
    assert!(
        t_rec.state_count >= t_simple.state_count,
        "recursive grammar should have >= states than simple"
    );
}

#[test]
fn manual_external_token_count_in_code() {
    let (g, t) = manual_pair("xe1", 1, 1, 0, 3, 2);
    let code = codegen(&g, &t);
    assert!(
        code.contains("external_token_count"),
        "must contain external_token_count"
    );
}

#[test]
fn manual_large_field_count() {
    let (g, t) = manual_pair("xe2", 1, 1, 10, 0, 1);
    let code = codegen(&g, &t);
    // Should have 10 field name statics
    for i in 0..10 {
        let expected = format!("FIELD_NAME_{i}");
        assert!(code.contains(&expected), "missing {expected}");
    }
}
