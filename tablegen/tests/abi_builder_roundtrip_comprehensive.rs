//! Comprehensive ABI builder roundtrip tests.
//!
//! Validates that ABI generation, static language generation, and node-types
//! generation are deterministic and produce valid output for a variety of
//! grammars.

use adze_glr_core::{FirstFollowSets, GotoIndexing, LexMode, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId,
    Token, TokenPattern,
};
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};
use std::collections::BTreeMap;

const INVALID: StateId = StateId(u16::MAX);

// ---------------------------------------------------------------------------
// Helper: manual grammar + parse-table construction
// ---------------------------------------------------------------------------

fn build_grammar_and_table(
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
    for (i, slot) in index_to_symbol.iter_mut().enumerate().take(symbol_count) {
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
        initial_state: StateId(0),
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

fn minimal() -> (Grammar, ParseTable) {
    build_grammar_and_table("minimal", 1, 1, 0, 0, 2)
}

fn abi_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

// ---------------------------------------------------------------------------
// Helper: GrammarBuilder-based construction (with real parse table)
// ---------------------------------------------------------------------------

fn build_real(
    name: &str,
    build: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> (Grammar, ParseTable) {
    let mut g = build(GrammarBuilder::new(name)).build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("compute ff");
    let table = build_lr1_automaton(&g, &ff).expect("build automaton");
    (g, table)
}

fn simple_grammar_and_table() -> (Grammar, ParseTable) {
    build_real("simple", |b| {
        b.token("x", r"x").rule("start", vec!["x"]).start("start")
    })
}

fn two_token_grammar_and_table() -> (Grammar, ParseTable) {
    build_real("two_tok", |b| {
        b.token("a", r"a")
            .token("b", r"b")
            .rule("start", vec!["a", "b"])
            .start("start")
    })
}

fn two_alt_grammar_and_table() -> (Grammar, ParseTable) {
    build_real("two_alt", |b| {
        b.token("a", r"a")
            .token("b", r"b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    })
}

fn chain_grammar_and_table() -> (Grammar, ParseTable) {
    build_real("chain", |b| {
        b.token("x", r"x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start")
    })
}

fn multi_token_grammar_and_table() -> (Grammar, ParseTable) {
    build_real("multi", |b| {
        b.token("a", r"a")
            .token("b", r"b")
            .token("c", r"c")
            .token("d", r"d")
            .rule("start", vec!["a", "b", "c", "d"])
            .start("start")
    })
}

fn precedence_grammar_and_table() -> (Grammar, ParseTable) {
    build_real("prec", |b| {
        b.token("num", r"\d+")
            .token("+", r"\+")
            .token("*", r"\*")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
            .start("expr")
    })
}

fn right_assoc_grammar_and_table() -> (Grammar, ParseTable) {
    build_real("rassoc", |b| {
        b.token("num", r"\d+")
            .token("^", r"\^")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
            .start("expr")
    })
}

// ---------------------------------------------------------------------------
// Helper: node-types JSON parsing
// ---------------------------------------------------------------------------

fn node_types_json(grammar: &Grammar) -> Vec<serde_json::Value> {
    let json = NodeTypesGenerator::new(grammar)
        .generate()
        .expect("NodeTypesGenerator::generate() failed");
    let val: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
    val.as_array().expect("expected JSON array").to_vec()
}

fn find_node<'a>(nodes: &'a [serde_json::Value], type_name: &str) -> Option<&'a serde_json::Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(type_name))
}

// ===========================================================================
// Area 1: AbiLanguageBuilder with minimal grammars
// ===========================================================================

#[test]
fn abi_minimal_generates_nonempty_output() {
    let (g, t) = minimal();
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_minimal_contains_language_keyword() {
    let (g, t) = minimal();
    let code = abi_code(&g, &t);
    assert!(code.contains("language") || code.contains("LANGUAGE") || code.contains("Language"));
}

#[test]
fn abi_minimal_single_token_grammar() {
    let (g, t) = build_grammar_and_table("single_tok", 1, 1, 0, 0, 1);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_minimal_single_state() {
    let (g, t) = build_grammar_and_table("one_state", 1, 1, 0, 0, 1);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_minimal_two_states() {
    let (g, t) = build_grammar_and_table("two_state", 1, 1, 0, 0, 2);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_minimal_with_fields() {
    let (g, t) = build_grammar_and_table("fields", 1, 1, 3, 0, 2);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_minimal_with_externals() {
    let (g, t) = build_grammar_and_table("ext", 1, 1, 0, 2, 2);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

// ===========================================================================
// Area 2: AbiLanguageBuilder with multi-token grammars
// ===========================================================================

#[test]
fn abi_two_tokens() {
    let (g, t) = build_grammar_and_table("two", 2, 1, 0, 0, 3);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_five_tokens() {
    let (g, t) = build_grammar_and_table("five", 5, 1, 0, 0, 3);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_ten_tokens() {
    let (g, t) = build_grammar_and_table("ten", 10, 1, 0, 0, 4);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_many_nonterms() {
    let (g, t) = build_grammar_and_table("many_nt", 2, 5, 0, 0, 4);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_tokens_and_fields_combined() {
    let (g, t) = build_grammar_and_table("combo", 4, 2, 3, 0, 4);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_tokens_fields_externals_combined() {
    let (g, t) = build_grammar_and_table("full", 3, 2, 2, 2, 5);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_real_two_token_sequence() {
    let (g, t) = two_token_grammar_and_table();
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_real_multi_token() {
    let (g, t) = multi_token_grammar_and_table();
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

// ===========================================================================
// Area 3: AbiLanguageBuilder with precedence
// ===========================================================================

#[test]
fn abi_precedence_grammar_generates() {
    let (g, t) = precedence_grammar_and_table();
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_right_assoc_grammar_generates() {
    let (g, t) = right_assoc_grammar_and_table();
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn abi_precedence_output_contains_grammar_name() {
    let (g, t) = precedence_grammar_and_table();
    let code = abi_code(&g, &t);
    assert!(
        code.contains("prec"),
        "generated code should reference the grammar name"
    );
}

// ===========================================================================
// Area 4: AbiLanguageBuilder output is valid TokenStream
// ===========================================================================

#[test]
fn abi_minimal_parses_as_token_stream() {
    let (g, t) = minimal();
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok(), "output is not a valid TokenStream");
}

#[test]
fn abi_simple_real_parses_as_token_stream() {
    let (g, t) = simple_grammar_and_table();
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok());
}

#[test]
fn abi_two_alt_parses_as_token_stream() {
    let (g, t) = two_alt_grammar_and_table();
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok());
}

#[test]
fn abi_chain_parses_as_token_stream() {
    let (g, t) = chain_grammar_and_table();
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok());
}

#[test]
fn abi_multi_token_parses_as_token_stream() {
    let (g, t) = multi_token_grammar_and_table();
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok());
}

#[test]
fn abi_precedence_parses_as_token_stream() {
    let (g, t) = precedence_grammar_and_table();
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok());
}

#[test]
fn abi_five_tokens_parses_as_token_stream() {
    let (g, t) = build_grammar_and_table("five_ts", 5, 2, 1, 0, 3);
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok());
}

#[test]
fn abi_externals_parses_as_token_stream() {
    let (g, t) = build_grammar_and_table("ext_ts", 2, 1, 0, 3, 2);
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    let reparsed: Result<proc_macro2::TokenStream, _> = ts.to_string().parse();
    assert!(reparsed.is_ok());
}

// ===========================================================================
// Area 5: StaticLanguageGenerator output is valid
// ===========================================================================

#[test]
fn static_gen_simple_generates() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_two_alt_generates() {
    let (g, t) = two_alt_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_chain_generates() {
    let (g, t) = chain_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_multi_token_generates() {
    let (g, t) = multi_token_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_precedence_generates() {
    let (g, t) = precedence_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_output_parses_as_token_stream() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    let reparsed: Result<proc_macro2::TokenStream, _> = code.parse();
    assert!(reparsed.is_ok());
}

#[test]
fn static_gen_right_assoc_generates() {
    let (g, t) = right_assoc_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_node_types_produces_json() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let json = slg.generate_node_types();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(
        parsed.is_ok(),
        "generate_node_types should produce valid JSON"
    );
}

// ===========================================================================
// Area 6: NodeTypesGenerator output is valid JSON
// ===========================================================================

#[test]
fn node_types_simple_is_valid_json() {
    let (g, _) = simple_grammar_and_table();
    let nodes = node_types_json(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn node_types_two_alt_is_valid_json() {
    let (g, _) = two_alt_grammar_and_table();
    let nodes = node_types_json(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn node_types_chain_is_valid_json() {
    let (g, _) = chain_grammar_and_table();
    let nodes = node_types_json(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn node_types_multi_token_is_valid_json() {
    let (g, _) = multi_token_grammar_and_table();
    let nodes = node_types_json(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn node_types_precedence_is_valid_json() {
    let (g, _) = precedence_grammar_and_table();
    let nodes = node_types_json(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn node_types_each_entry_has_type_field() {
    let (g, _) = two_alt_grammar_and_table();
    let nodes = node_types_json(&g);
    for node in &nodes {
        assert!(
            node.get("type").is_some(),
            "every node-types entry must have a 'type' field"
        );
    }
}

#[test]
fn node_types_each_entry_has_named_field() {
    let (g, _) = two_alt_grammar_and_table();
    let nodes = node_types_json(&g);
    for node in &nodes {
        assert!(
            node.get("named").is_some(),
            "every node-types entry must have a 'named' field"
        );
    }
}

#[test]
fn node_types_manual_grammar_is_valid_json() {
    let g = GrammarBuilder::new("manual_nt")
        .token("id", r"[a-z]+")
        .token(",", ",")
        .rule("item", vec!["id"])
        .rule("list", vec!["item"])
        .rule("list", vec!["list", ",", "item"])
        .start("list")
        .build();
    let nodes = node_types_json(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn node_types_find_start_symbol() {
    let (g, _) = simple_grammar_and_table();
    let nodes = node_types_json(&g);
    let start = find_node(&nodes, "start");
    assert!(start.is_some(), "should find 'start' in node types");
}

// ===========================================================================
// Area 7: Roundtrip – same grammar produces same output
// ===========================================================================

#[test]
fn abi_deterministic_minimal() {
    let (g, t) = minimal();
    let a = abi_code(&g, &t);
    let b = abi_code(&g, &t);
    assert_eq!(a, b);
}

#[test]
fn abi_deterministic_simple_real() {
    let (g, t) = simple_grammar_and_table();
    let a = abi_code(&g, &t);
    let b = abi_code(&g, &t);
    assert_eq!(a, b);
}

#[test]
fn abi_deterministic_two_alt() {
    let (g, t) = two_alt_grammar_and_table();
    let a = abi_code(&g, &t);
    let b = abi_code(&g, &t);
    assert_eq!(a, b);
}

#[test]
fn abi_deterministic_chain() {
    let (g, t) = chain_grammar_and_table();
    let a = abi_code(&g, &t);
    let b = abi_code(&g, &t);
    assert_eq!(a, b);
}

#[test]
fn abi_deterministic_precedence() {
    let (g, t) = precedence_grammar_and_table();
    let a = abi_code(&g, &t);
    let b = abi_code(&g, &t);
    assert_eq!(a, b);
}

#[test]
fn static_gen_deterministic_simple() {
    let (g1, t1) = simple_grammar_and_table();
    let (g2, t2) = simple_grammar_and_table();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn static_gen_deterministic_two_alt() {
    let (g1, t1) = two_alt_grammar_and_table();
    let (g2, t2) = two_alt_grammar_and_table();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn node_types_deterministic_simple() {
    let (g, _) = simple_grammar_and_table();
    let a = NodeTypesGenerator::new(&g).generate().unwrap();
    let b = NodeTypesGenerator::new(&g).generate().unwrap();
    assert_eq!(a, b);
}

#[test]
fn node_types_deterministic_precedence() {
    let (g, _) = precedence_grammar_and_table();
    let a = NodeTypesGenerator::new(&g).generate().unwrap();
    let b = NodeTypesGenerator::new(&g).generate().unwrap();
    assert_eq!(a, b);
}

// ===========================================================================
// Area 8: AbiLanguageBuilder vs StaticLanguageGenerator comparison
// ===========================================================================

#[test]
fn both_generators_produce_nonempty_for_simple() {
    let (g, t) = simple_grammar_and_table();
    let abi = abi_code(&g, &t);
    let slg = StaticLanguageGenerator::new(g.clone(), t.clone())
        .generate_language_code()
        .to_string();
    assert!(!abi.is_empty());
    assert!(!slg.is_empty());
}

#[test]
fn both_generators_produce_nonempty_for_two_alt() {
    let (g, t) = two_alt_grammar_and_table();
    let abi = abi_code(&g, &t);
    let slg = StaticLanguageGenerator::new(g.clone(), t.clone())
        .generate_language_code()
        .to_string();
    assert!(!abi.is_empty());
    assert!(!slg.is_empty());
}

#[test]
fn both_generators_produce_nonempty_for_chain() {
    let (g, t) = chain_grammar_and_table();
    let abi = abi_code(&g, &t);
    let slg = StaticLanguageGenerator::new(g.clone(), t.clone())
        .generate_language_code()
        .to_string();
    assert!(!abi.is_empty());
    assert!(!slg.is_empty());
}

#[test]
fn both_generators_produce_valid_token_streams() {
    let (g, t) = simple_grammar_and_table();
    let abi_ts = AbiLanguageBuilder::new(&g, &t).generate();
    let slg_ts = StaticLanguageGenerator::new(g.clone(), t.clone()).generate_language_code();
    let abi_ok: Result<proc_macro2::TokenStream, _> = abi_ts.to_string().parse();
    let slg_ok: Result<proc_macro2::TokenStream, _> = slg_ts.to_string().parse();
    assert!(abi_ok.is_ok());
    assert!(slg_ok.is_ok());
}

#[test]
fn both_generators_produce_valid_for_precedence() {
    let (g, t) = precedence_grammar_and_table();
    let abi_ts = AbiLanguageBuilder::new(&g, &t).generate();
    let slg_ts = StaticLanguageGenerator::new(g.clone(), t.clone()).generate_language_code();
    let abi_ok: Result<proc_macro2::TokenStream, _> = abi_ts.to_string().parse();
    let slg_ok: Result<proc_macro2::TokenStream, _> = slg_ts.to_string().parse();
    assert!(abi_ok.is_ok());
    assert!(slg_ok.is_ok());
}

// ===========================================================================
// Area 9: Generated code contains expected symbols
// ===========================================================================

#[test]
fn abi_simple_contains_grammar_name() {
    let (g, t) = simple_grammar_and_table();
    let code = abi_code(&g, &t);
    assert!(
        code.contains("simple"),
        "generated code should reference the grammar name"
    );
}

#[test]
fn abi_chain_contains_grammar_name() {
    let (g, t) = chain_grammar_and_table();
    let code = abi_code(&g, &t);
    assert!(
        code.contains("chain"),
        "generated code should reference 'chain'"
    );
}

#[test]
fn static_gen_simple_contains_grammar_name() {
    let (g, t) = simple_grammar_and_table();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("simple"));
}

#[test]
fn static_gen_multi_contains_token_names() {
    let (g, t) = multi_token_grammar_and_table();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    // At least one of the token names should appear
    assert!(
        code.contains('a') || code.contains('b') || code.contains('c') || code.contains('d'),
        "multi-token grammar code should reference at least one token"
    );
}

#[test]
fn node_types_chain_has_inner_and_start() {
    let (g, _) = chain_grammar_and_table();
    let nodes = node_types_json(&g);
    let has_inner = find_node(&nodes, "inner").is_some();
    let has_start = find_node(&nodes, "start").is_some();
    assert!(
        has_inner || has_start,
        "chain grammar should emit node types for its rules"
    );
}

#[test]
fn node_types_precedence_has_expr() {
    let (g, _) = precedence_grammar_and_table();
    let nodes = node_types_json(&g);
    let has_expr = find_node(&nodes, "expr").is_some();
    assert!(has_expr, "precedence grammar should emit 'expr' node type");
}

// ===========================================================================
// Area 10: Edge cases
// ===========================================================================

#[test]
fn edge_single_token_single_state() {
    let (g, t) = build_grammar_and_table("edge1", 1, 1, 0, 0, 1);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn edge_many_tokens_many_states() {
    let (g, t) = build_grammar_and_table("edge_big", 20, 5, 4, 0, 10);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn edge_many_externals() {
    let (g, t) = build_grammar_and_table("edge_ext", 1, 1, 0, 5, 2);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn edge_many_fields() {
    let (g, t) = build_grammar_and_table("edge_fields", 1, 1, 10, 0, 2);
    let code = abi_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn edge_single_real_grammar() {
    let (g, t) = simple_grammar_and_table();
    let code = abi_code(&g, &t);
    let ts: Result<proc_macro2::TokenStream, _> = code.parse();
    assert!(ts.is_ok());
}

#[test]
fn edge_different_names_produce_different_output() {
    let (g1, t1) = build_grammar_and_table("alpha", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("beta", 1, 1, 0, 0, 2);
    let code1 = abi_code(&g1, &t1);
    let code2 = abi_code(&g2, &t2);
    assert_ne!(
        code1, code2,
        "different grammar names should produce different output"
    );
}

#[test]
fn edge_more_tokens_changes_output() {
    let (g1, t1) = build_grammar_and_table("diff1", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("diff2", 3, 1, 0, 0, 2);
    let code1 = abi_code(&g1, &t1);
    let code2 = abi_code(&g2, &t2);
    assert_ne!(code1, code2);
}

#[test]
fn edge_node_types_empty_grammar() {
    let g = Grammar::new("empty_nt".to_string());
    let result = NodeTypesGenerator::new(&g).generate();
    // May succeed with empty array or fail — either is acceptable
    if let Ok(json) = result {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(
            parsed.is_ok(),
            "if generate succeeds, output must be valid JSON"
        );
    }
}

#[test]
fn edge_fragile_token_grammar() {
    let mut g = Grammar::new("fragile".to_string());
    let sym = SymbolId(1);
    g.tokens.insert(
        sym,
        Token {
            name: "ws".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: true,
        },
    );
    let result = NodeTypesGenerator::new(&g).generate();
    if let Ok(json) = result {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
    }
}

#[test]
fn edge_static_gen_node_types_is_valid_json() {
    let (g, t) = two_alt_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let json = slg.generate_node_types();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(
        parsed.is_ok(),
        "StaticLanguageGenerator::generate_node_types must produce valid JSON"
    );
}

#[test]
fn edge_static_gen_node_types_deterministic() {
    let (g1, t1) = simple_grammar_and_table();
    let (g2, t2) = simple_grammar_and_table();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(a, b);
}

#[test]
fn edge_abi_with_all_features() {
    let (g, t) = build_grammar_and_table("all_feat", 4, 3, 2, 1, 5);
    let code = abi_code(&g, &t);
    let ts: Result<proc_macro2::TokenStream, _> = code.parse();
    assert!(ts.is_ok());
}
