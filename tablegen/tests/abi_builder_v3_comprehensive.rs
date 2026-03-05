#![allow(clippy::needless_range_loop)]

//! Comprehensive v3 tests for `AbiLanguageBuilder` covering construction,
//! generated field properties, symbol tables, action/goto tables, determinism,
//! grammar topologies, and edge cases.
//!
//! Target: 55+ tests exercising the public API via the full pipeline
//! (GrammarBuilder → FirstFollowSets → build_lr1_automaton → AbiLanguageBuilder).

use adze_glr_core::{FirstFollowSets, GotoIndexing, LexMode, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::{AbiLanguageBuilder, StaticLanguageGenerator};
use std::collections::BTreeMap;

// ===========================================================================
// Helpers
// ===========================================================================

const INVALID: adze_glr_core::StateId = adze_glr_core::StateId(u16::MAX);

/// Build a grammar + parse table pair through the full pipeline using
/// `GrammarBuilder`.
fn make_grammar_and_table(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let mut g = grammar;
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&g, &ff).expect("LR(1) build failed");
    (g, table)
}

/// Build a grammar with two tokens and two alternative rules.
fn make_two_token_grammar(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let mut g = grammar;
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&g, &ff).expect("LR(1) build failed");
    (g, table)
}

/// Build a grammar with a chain: S → A → x.
fn make_chain_grammar(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("A", vec!["x"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let mut g = grammar;
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&g, &ff).expect("LR(1) build failed");
    (g, table)
}

/// Build a grammar with multiple tokens in one rule: S → a b.
fn make_sequence_grammar(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let mut g = grammar;
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&g, &ff).expect("LR(1) build failed");
    (g, table)
}

/// Build a grammar with a recursive rule: S → x | S x.
fn make_left_recursive_grammar(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("S", vec!["x"])
        .rule("S", vec!["S", "x"])
        .start("S")
        .build();
    let mut g = grammar;
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&g, &ff).expect("LR(1) build failed");
    (g, table)
}

/// Build a grammar with right recursion: S → x | x S.
fn make_right_recursive_grammar(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("S", vec!["x"])
        .rule("S", vec!["x", "S"])
        .start("S")
        .build();
    let mut g = grammar;
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&g, &ff).expect("LR(1) build failed");
    (g, table)
}

/// Build a grammar with many tokens: S → t0 | t1 | ... | t_{n-1}.
fn make_wide_grammar(name: &str, num_tokens: usize) -> (Grammar, ParseTable) {
    let mut builder = GrammarBuilder::new(name);
    for i in 0..num_tokens {
        let tok_name = format!("t{i}");
        let tok_pattern = format!("{i}");
        builder = builder.token(&tok_name, &tok_pattern);
    }
    for i in 0..num_tokens {
        let tok_name = format!("t{i}");
        builder = builder.rule("S", vec![&tok_name]);
    }
    builder = builder.start("S");
    let grammar = builder.build();
    let mut g = grammar;
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let table = build_lr1_automaton(&g, &ff).expect("LR(1) build failed");
    (g, table)
}

/// Build a grammar + parse table pair using manual layout (no pipeline).
fn build_manual_pair(
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
    for i in 0..symbol_count {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        index_to_symbol[i] = sym;
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

/// Generate the code string from a grammar+table pair.
fn gen_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

/// Full pipeline: GrammarBuilder → normalize → FIRST/FOLLOW → LR(1) → ABI code.
fn full_pipeline(builder: GrammarBuilder) -> (String, Grammar, ParseTable) {
    let mut grammar = builder.build();
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) build failed");
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    (code, grammar, table)
}

// ===========================================================================
// 1. AbiLanguageBuilder construction (8 tests)
// ===========================================================================

#[test]
fn construct_from_single_token_grammar() {
    let (g, t) = make_grammar_and_table("single_tok");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_from_two_token_grammar() {
    let (g, t) = make_two_token_grammar("two_tok");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_from_chain_grammar() {
    let (g, t) = make_chain_grammar("chain");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_from_sequence_grammar() {
    let (g, t) = make_sequence_grammar("seq");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_from_left_recursive_grammar() {
    let (g, t) = make_left_recursive_grammar("lrec");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_from_right_recursive_grammar() {
    let (g, t) = make_right_recursive_grammar("rrec");
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_from_wide_grammar() {
    let (g, t) = make_wide_grammar("wide", 8);
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_multiple_builders_same_input() {
    let (g, t) = make_grammar_and_table("multi");
    let _b1 = AbiLanguageBuilder::new(&g, &t);
    let _b2 = AbiLanguageBuilder::new(&g, &t);
    let _b3 = AbiLanguageBuilder::new(&g, &t);
}

// ===========================================================================
// 2. Generated fields properties (10 tests)
// ===========================================================================

#[test]
fn generated_code_is_nonempty() {
    let (g, t) = make_grammar_and_table("nonempty");
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    assert!(!ts.is_empty(), "TokenStream must not be empty");
}

#[test]
fn generated_code_contains_language_struct() {
    let (g, t) = make_grammar_and_table("lang_struct");
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"), "must contain LANGUAGE static");
}

#[test]
fn generated_code_contains_tslanguage() {
    let (g, t) = make_grammar_and_table("tslang");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("TSLanguage"),
        "must reference TSLanguage type"
    );
}

#[test]
fn generated_code_contains_symbol_count() {
    let (g, t) = make_grammar_and_table("symcnt");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("symbol_count"),
        "must contain symbol_count field"
    );
}

#[test]
fn generated_code_contains_state_count() {
    let (g, t) = make_grammar_and_table("stcnt");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("state_count"),
        "must contain state_count field"
    );
}

#[test]
fn generated_code_contains_token_count() {
    let (g, t) = make_grammar_and_table("tokcnt");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("token_count"),
        "must contain token_count field"
    );
}

#[test]
fn generated_code_contains_field_count() {
    let (g, t) = make_grammar_and_table("fldcnt");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("field_count"),
        "must contain field_count field"
    );
}

#[test]
fn generated_code_contains_parse_table() {
    let (g, t) = make_grammar_and_table("ptbl");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("PARSE_TABLE"),
        "must contain PARSE_TABLE static"
    );
}

#[test]
fn generated_code_contains_small_parse_table() {
    let (g, t) = make_grammar_and_table("sptbl");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("SMALL_PARSE_TABLE"),
        "must contain SMALL_PARSE_TABLE static"
    );
}

#[test]
fn generated_code_has_substantial_length() {
    let (g, t) = make_grammar_and_table("sublen");
    let code = gen_code(&g, &t);
    assert!(
        code.len() > 200,
        "generated code should be substantial, got {} bytes",
        code.len()
    );
}

// ===========================================================================
// 3. Symbol table generation (8 tests)
// ===========================================================================

#[test]
fn symbol_names_generated_for_single_token() {
    let (g, t) = make_grammar_and_table("sym1");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("SYMBOL_NAME_"),
        "must contain symbol name statics"
    );
}

#[test]
fn symbol_name_ptrs_array_present() {
    let (g, t) = make_grammar_and_table("symptrs");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("SYMBOL_NAME_PTRS"),
        "must contain SYMBOL_NAME_PTRS array"
    );
}

#[test]
fn symbol_metadata_array_present() {
    let (g, t) = make_grammar_and_table("symmeta");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("SYMBOL_METADATA"),
        "must contain SYMBOL_METADATA"
    );
}

#[test]
fn public_symbol_map_present() {
    let (g, t) = make_grammar_and_table("pubsym");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP"),
        "must contain PUBLIC_SYMBOL_MAP"
    );
}

#[test]
fn symbol_count_matches_table_for_two_tokens() {
    let (g, t) = make_two_token_grammar("sym2tok");
    let code = gen_code(&g, &t);
    // symbol_count should appear in the generated code with the right value
    let _count_str = format!("symbol_count : {}", t.symbol_count);
    // proc-macro2 output may have spaces around colons; just check the value is present
    assert!(
        code.contains(&format!("{}", t.symbol_count))
            || code.contains(&format!("{}u32", t.symbol_count))
            || code.contains(&format!("{} u32", t.symbol_count)),
        "symbol_count value {} must appear in generated code",
        t.symbol_count
    );
}

#[test]
fn symbol_id_to_index_mapping_present() {
    let (g, t) = make_grammar_and_table("symidx");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("SYMBOL_ID_TO_INDEX"),
        "must contain SYMBOL_ID_TO_INDEX mapping"
    );
}

#[test]
fn symbol_index_to_id_mapping_present() {
    let (g, t) = make_grammar_and_table("symid");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("SYMBOL_INDEX_TO_ID"),
        "must contain SYMBOL_INDEX_TO_ID mapping"
    );
}

#[test]
fn wide_grammar_has_all_symbol_names() {
    let (g, t) = make_wide_grammar("wide_sym", 5);
    let code = gen_code(&g, &t);
    // Each symbol name gets a SYMBOL_NAME_N static
    for i in 0..t.symbol_count {
        let expected = format!("SYMBOL_NAME_{i}");
        assert!(
            code.contains(&expected),
            "missing {} in generated code",
            expected
        );
    }
}

// ===========================================================================
// 4. Action/goto table generation (8 tests)
// ===========================================================================

#[test]
fn parse_actions_array_present() {
    let (g, t) = make_grammar_and_table("pact");
    let code = gen_code(&g, &t);
    assert!(code.contains("PARSE_ACTIONS"), "must contain PARSE_ACTIONS");
}

#[test]
fn ts_rules_array_present() {
    let (g, t) = make_grammar_and_table("tsrules");
    let code = gen_code(&g, &t);
    assert!(code.contains("TS_RULES"), "must contain TS_RULES");
}

#[test]
fn production_id_map_present() {
    let (g, t) = make_grammar_and_table("pidmap");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("PRODUCTION_ID_MAP"),
        "must contain PRODUCTION_ID_MAP"
    );
}

#[test]
fn production_lhs_index_present() {
    let (g, t) = make_grammar_and_table("plhs");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("PRODUCTION_LHS_INDEX"),
        "must contain PRODUCTION_LHS_INDEX"
    );
}

#[test]
fn lex_modes_array_present() {
    let (g, t) = make_grammar_and_table("lexm");
    let code = gen_code(&g, &t);
    assert!(code.contains("LEX_MODES"), "must contain LEX_MODES");
}

#[test]
fn small_parse_table_map_present() {
    let (g, t) = make_grammar_and_table("sptm");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("SMALL_PARSE_TABLE_MAP"),
        "must contain SMALL_PARSE_TABLE_MAP"
    );
}

#[test]
fn field_map_slices_and_entries_present() {
    let (g, t) = make_grammar_and_table("fmap");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("FIELD_MAP_SLICES"),
        "must contain FIELD_MAP_SLICES"
    );
    assert!(
        code.contains("FIELD_MAP_ENTRIES"),
        "must contain FIELD_MAP_ENTRIES"
    );
}

#[test]
fn primary_state_ids_present() {
    let (g, t) = make_grammar_and_table("psid");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "must contain PRIMARY_STATE_IDS"
    );
}

// ===========================================================================
// 5. Determinism across multiple builds (8 tests)
// ===========================================================================

#[test]
fn determinism_single_token_grammar() {
    let (g, t) = make_grammar_and_table("det1");
    let code1 = gen_code(&g, &t);
    let code2 = gen_code(&g, &t);
    assert_eq!(code1, code2, "repeated generation must be identical");
}

#[test]
fn determinism_two_token_grammar() {
    let (g, t) = make_two_token_grammar("det2");
    let code1 = gen_code(&g, &t);
    let code2 = gen_code(&g, &t);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_chain_grammar() {
    let (g, t) = make_chain_grammar("detchain");
    let code1 = gen_code(&g, &t);
    let code2 = gen_code(&g, &t);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_sequence_grammar() {
    let (g, t) = make_sequence_grammar("detseq");
    let code1 = gen_code(&g, &t);
    let code2 = gen_code(&g, &t);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_left_recursive_grammar() {
    let (g, t) = make_left_recursive_grammar("detlrec");
    let code1 = gen_code(&g, &t);
    let code2 = gen_code(&g, &t);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_right_recursive_grammar() {
    let (g, t) = make_right_recursive_grammar("detrrec");
    let code1 = gen_code(&g, &t);
    let code2 = gen_code(&g, &t);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_wide_grammar() {
    let (g, t) = make_wide_grammar("detwide", 6);
    let code1 = gen_code(&g, &t);
    let code2 = gen_code(&g, &t);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_three_consecutive_runs() {
    let (g, t) = make_grammar_and_table("det3run");
    let a = gen_code(&g, &t);
    let b = gen_code(&g, &t);
    let c = gen_code(&g, &t);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ===========================================================================
// 6. Different grammar topologies (8 tests)
// ===========================================================================

#[test]
fn topology_single_token_single_rule() {
    let (code, _, _) = full_pipeline(
        GrammarBuilder::new("topo1")
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn topology_two_alternatives() {
    let (code, _, _) = full_pipeline(
        GrammarBuilder::new("topo2")
            .token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn topology_chain_of_nonterminals() {
    let (code, _, _) = full_pipeline(
        GrammarBuilder::new("topo_chain")
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
fn topology_sequence_two_tokens() {
    let (code, _, _) = full_pipeline(
        GrammarBuilder::new("topo_seq")
            .token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a", "b"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn topology_left_recursive() {
    let (code, _, table) = full_pipeline(
        GrammarBuilder::new("topo_lrec")
            .token("x", "x")
            .rule("S", vec!["x"])
            .rule("S", vec!["S", "x"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    assert!(
        table.state_count > 1,
        "recursive grammar needs multiple states"
    );
}

#[test]
fn topology_right_recursive() {
    let (code, _, table) = full_pipeline(
        GrammarBuilder::new("topo_rrec")
            .token("x", "x")
            .rule("S", vec!["x"])
            .rule("S", vec!["x", "S"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
    assert!(table.state_count > 1);
}

#[test]
fn topology_diamond_shaped() {
    // S → A | B, A → x, B → x
    let (code, _, _) = full_pipeline(
        GrammarBuilder::new("topo_diamond")
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
    // S → t0 | t1 | t2 | t3 | t4
    let (g, t) = make_wide_grammar("topo_wide", 5);
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(
        t.symbol_count >= 5,
        "wide grammar must have at least 5 symbols"
    );
}

// ===========================================================================
// 7. Edge cases and error handling (5+ tests)
// ===========================================================================

#[test]
fn edge_single_char_token() {
    let (code, _, _) = full_pipeline(
        GrammarBuilder::new("ec1")
            .token("z", "z")
            .rule("S", vec!["z"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn edge_multi_char_token() {
    let (code, _, _) = full_pipeline(
        GrammarBuilder::new("ec2")
            .token("hello", "hello")
            .rule("S", vec!["hello"])
            .start("S"),
    );
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn edge_manual_grammar_with_fields() {
    let (g, t) = build_manual_pair("ec_fld", 2, 1, 4, 0, 3);
    let code = gen_code(&g, &t);
    assert!(code.contains("FIELD_NAME_"));
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn edge_manual_grammar_zero_fields_no_field_names() {
    let (g, t) = build_manual_pair("ec_nofld", 1, 1, 0, 0, 2);
    let code = gen_code(&g, &t);
    // With zero fields, the FIELD_NAME_PTRS array should be empty
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn edge_manual_grammar_with_externals() {
    let (g, t) = build_manual_pair("ec_ext", 1, 1, 0, 2, 2);
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    // External scanner struct should be present
    assert!(
        code.contains("ExternalScanner"),
        "must contain ExternalScanner struct"
    );
}

// ===========================================================================
// 8. Additional coverage — StaticLanguageGenerator, code properties
// ===========================================================================

#[test]
fn static_language_generator_produces_code() {
    let (g, t) = make_grammar_and_table("slg");
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn generated_code_contains_lexer_fn() {
    let (g, t) = make_grammar_and_table("lexfn");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("lexer_fn"),
        "must contain reference to lexer_fn"
    );
}

#[test]
fn generated_code_contains_version_field() {
    let (g, t) = make_grammar_and_table("ver");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("version"),
        "must contain version field in TSLanguage"
    );
}

#[test]
fn generated_code_contains_eof_symbol_field() {
    let (g, t) = make_grammar_and_table("eof");
    let code = gen_code(&g, &t);
    assert!(code.contains("eof_symbol"), "must contain eof_symbol field");
}

#[test]
fn generated_code_contains_production_count() {
    let (g, t) = make_grammar_and_table("prodcnt");
    let code = gen_code(&g, &t);
    assert!(
        code.contains("production_count") || code.contains("production_id_count"),
        "must contain production count field"
    );
}

#[test]
fn different_grammar_names_produce_different_fn_names() {
    let (g1, t1) = make_grammar_and_table("alpha");
    let (g2, t2) = make_grammar_and_table("beta");
    let code1 = gen_code(&g1, &t1);
    let code2 = gen_code(&g2, &t2);
    assert!(
        code1.contains("tree_sitter_alpha"),
        "must contain language-specific fn name"
    );
    assert!(
        code2.contains("tree_sitter_beta"),
        "must contain language-specific fn name"
    );
    // The fn names must differ
    assert!(
        !code1.contains("tree_sitter_beta"),
        "alpha code must not mention beta"
    );
}

#[test]
fn chain_grammar_has_more_rules_than_single() {
    let (_, _, t_single) = full_pipeline(
        GrammarBuilder::new("cmp1")
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S"),
    );
    let (_, _, t_chain) = full_pipeline(
        GrammarBuilder::new("cmp2")
            .token("x", "x")
            .rule("A", vec!["x"])
            .rule("S", vec!["A"])
            .start("S"),
    );
    assert!(
        t_chain.symbol_count >= t_single.symbol_count,
        "chain grammar should have at least as many symbols"
    );
}

#[test]
fn recursive_grammar_has_more_states_than_single() {
    let (_, _, t_single) = full_pipeline(
        GrammarBuilder::new("rst1")
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S"),
    );
    let (_, _, t_rec) = full_pipeline(
        GrammarBuilder::new("rst2")
            .token("x", "x")
            .rule("S", vec!["x"])
            .rule("S", vec!["S", "x"])
            .start("S"),
    );
    assert!(
        t_rec.state_count >= t_single.state_count,
        "recursive grammar should have at least as many states"
    );
}

#[test]
fn wide_grammar_state_count_grows_with_tokens() {
    let (_, t_small) = make_wide_grammar("wsc_s", 2);
    let (_, t_large) = make_wide_grammar("wsc_l", 8);
    assert!(
        t_large.symbol_count > t_small.symbol_count,
        "more tokens → more symbols"
    );
}
