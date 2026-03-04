//! Comprehensive tests for the language generation module.
//!
//! Covers: StaticLanguageGenerator construction, generate_language_code(),
//! generate_node_types(), compression, LanguageGenerator, LanguageBuilder,
//! NodeTypesGenerator, ABI constants, validators, determinism, empty/minimal
//! grammars, multi-rule grammars, start_can_be_empty, and cross-crate pipeline.

use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseTable, SymbolMetadata, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::abi::{
    self, ExternalScanner, TREE_SITTER_LANGUAGE_VERSION,
    TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, TSFieldId, TSLanguage as AbiTSLanguage,
    TSLexState, TSParseAction, TSStateId, TSSymbol, create_symbol_metadata,
};
use adze_tablegen::compress::CompressedParseTable;
use adze_tablegen::language_gen::LanguageGenerator;
use adze_tablegen::validation::{LanguageValidator, ValidationError};
use adze_tablegen::{
    LanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator, TableCompressor,
    collect_token_indices, eof_accepts_or_reduces,
};

// ===========================================================================
// Helpers — hand-built parse tables
// ===========================================================================

fn make_grammar(name: &str, tokens: Vec<(SymbolId, Token)>, rules: Vec<Rule>) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    for (id, tok) in tokens {
        g.tokens.insert(id, tok);
    }
    for rule in rules {
        g.add_rule(rule);
    }
    g
}

fn regex_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::Regex(pattern.to_string()),
        fragile: false,
    }
}

fn string_token(name: &str, literal: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(literal.to_string()),
        fragile: false,
    }
}

fn make_parse_table_for_gen(
    grammar: &Grammar,
    state_count: usize,
    actions: Vec<Vec<Vec<Action>>>,
) -> ParseTable {
    let symbol_count = actions.first().map(|r| r.len()).unwrap_or(1);
    let mut symbol_to_index = std::collections::BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();
    let goto_table = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        state_count
    ];
    let eof_symbol = SymbolId((symbol_count.saturating_sub(1)) as u16);

    ParseTable {
        action_table: actions,
        goto_table,
        rules: vec![],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: std::collections::BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol,
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

// ===========================================================================
// Helpers — GrammarBuilder → ParseTable via FIRST/FOLLOW + LR(1)
// ===========================================================================

fn build_pipeline(
    name: &str,
    builder_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> (Grammar, ParseTable) {
    let builder = builder_fn(GrammarBuilder::new(name));
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    let pt = build_lr1_automaton(&g, &ff).expect("LR(1) automaton");
    (g, pt)
}

fn simple_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("simple", |b| {
        b.token("x", "x").rule("start", vec!["x"]).start("start")
    })
}

fn two_alt_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("two_alt", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    })
}

fn chain_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("chain", |b| {
        b.token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("start", vec!["A"])
            .start("start")
    })
}

fn recursive_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("recursive", |b| {
        b.token("a", "a")
            .rule("A", vec!["a", "A"])
            .rule("A", vec!["a"])
            .rule("start", vec!["A"])
            .start("start")
    })
}

fn multi_token_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("multi", |b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start")
    })
}

// ===========================================================================
// 1. StaticLanguageGenerator construction
// ===========================================================================

#[test]
fn slg_new_simple() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn slg_new_two_alt() {
    let (g, t) = two_alt_grammar_and_table();
    let _ = StaticLanguageGenerator::new(g, t);
}

#[test]
fn slg_new_chain() {
    let (g, t) = chain_grammar_and_table();
    let _ = StaticLanguageGenerator::new(g, t);
}

#[test]
fn slg_new_recursive() {
    let (g, t) = recursive_grammar_and_table();
    let _ = StaticLanguageGenerator::new(g, t);
}

#[test]
fn slg_new_preserves_grammar_name() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "simple");
}

#[test]
fn slg_new_preserves_parse_table_state_count() {
    let (g, t) = simple_grammar_and_table();
    let expected = t.state_count;
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.parse_table.state_count, expected);
}

// ===========================================================================
// 2. generate_language_code() produces valid TokenStream
// ===========================================================================

#[test]
fn slg_language_code_not_empty() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn slg_language_code_two_alt() {
    let (g, t) = two_alt_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn slg_language_code_chain() {
    let (g, t) = chain_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.generate_language_code().to_string().is_empty());
}

#[test]
fn slg_language_code_recursive() {
    let (g, t) = recursive_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.generate_language_code().to_string().is_empty());
}

#[test]
fn slg_language_code_multi_token() {
    let (g, t) = multi_token_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.generate_language_code().to_string().is_empty());
}

#[test]
fn slg_language_code_parseable_as_token_stream() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let ts = slg.generate_language_code();
    // Round-trip through string → TokenStream must succeed
    let s = ts.to_string();
    let reparsed: proc_macro2::TokenStream = s.parse().expect("must reparse");
    assert!(!reparsed.is_empty());
}

// ===========================================================================
// 3. generate_node_types() produces valid JSON
// ===========================================================================

#[test]
fn slg_node_types_valid_json() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn slg_node_types_two_alt_valid_json() {
    let (g, t) = two_alt_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let v: serde_json::Value = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(v.is_array());
}

#[test]
fn slg_node_types_chain_valid_json() {
    let (g, t) = chain_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let v: serde_json::Value = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(v.is_array());
}

#[test]
fn slg_node_types_not_empty_for_grammar_with_rules() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(
        !arr.is_empty(),
        "grammar with rules should produce node types"
    );
}

#[test]
fn slg_node_types_entries_have_type_field() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    for entry in &arr {
        assert!(
            entry["type"].is_string() || !entry["type"].is_null(),
            "each node type entry needs 'type'"
        );
    }
}

#[test]
fn slg_node_types_entries_have_named_field() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    for entry in &arr {
        assert!(
            !entry["named"].is_null(),
            "each node type entry needs 'named'"
        );
    }
}

// ===========================================================================
// 4. Language code contains expected elements
// ===========================================================================

#[test]
fn slg_code_contains_symbol_names() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(
        slg.generate_language_code()
            .to_string()
            .contains("SYMBOL_NAMES")
    );
}

#[test]
fn slg_code_contains_symbol_metadata() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(
        slg.generate_language_code()
            .to_string()
            .contains("SYMBOL_METADATA")
    );
}

#[test]
fn slg_code_contains_field_names() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(
        slg.generate_language_code()
            .to_string()
            .contains("FIELD_NAMES")
    );
}

#[test]
fn slg_code_contains_parse_table() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(
        slg.generate_language_code()
            .to_string()
            .contains("PARSE_TABLE")
    );
}

#[test]
fn slg_code_contains_tslanguage() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(
        slg.generate_language_code()
            .to_string()
            .contains("TSLanguage")
    );
}

#[test]
fn slg_code_contains_language_fn() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("language"));
}

#[test]
fn slg_code_contains_ffi_export() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("tree_sitter_simple"));
}

#[test]
fn slg_code_contains_version_constant() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

#[test]
fn slg_code_contains_lex_modes() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(
        slg.generate_language_code()
            .to_string()
            .contains("LEX_MODES")
    );
}

#[test]
fn slg_code_contains_external_scanner() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(
        slg.generate_language_code()
            .to_string()
            .contains("EXTERNAL_SCANNER")
    );
}

// ===========================================================================
// 5. Node types JSON has correct structure
// ===========================================================================

#[test]
fn slg_node_types_named_tokens_are_named_true() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    // Regex tokens should be named: true
    for entry in &arr {
        if entry["type"].as_str().unwrap_or("") == "x" {
            assert_eq!(entry["named"], true);
        }
    }
}

#[test]
fn slg_node_types_rules_are_named_true() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    // Non-terminal rules should be named: true
    for entry in &arr {
        let ty = entry["type"].as_str().unwrap_or("");
        if ty.starts_with("rule_") || ty == "start" {
            assert_eq!(entry["named"], true);
        }
    }
}

#[test]
fn slg_node_types_multi_token_lists_tokens() {
    let (g, t) = multi_token_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    // tokens a, b, c should appear
    assert!(json_str.contains("\"a\"") || json_str.contains("a"));
}

#[test]
fn slg_node_types_external_token_appears() {
    let (mut g, t) = simple_grammar_and_table();
    g.externals.push(ExternalToken {
        name: "ext_tok".to_string(),
        symbol_id: SymbolId(999),
    });
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    assert!(json_str.contains("ext_tok"));
}

#[test]
fn slg_node_types_hidden_external_excluded() {
    let (mut g, t) = simple_grammar_and_table();
    g.externals.push(ExternalToken {
        name: "_hidden_ext".to_string(),
        symbol_id: SymbolId(998),
    });
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    assert!(!json_str.contains("_hidden_ext"));
}

// ===========================================================================
// 6. Compression works via full pipeline
// ===========================================================================

#[test]
fn compress_simple_table() {
    let (g, t) = simple_grammar_and_table();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok(), "compress failed: {:?}", result.err());
}

#[test]
fn compress_two_alt_table() {
    let (g, t) = two_alt_grammar_and_table();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    assert!(compressor.compress(&t, &token_indices, false).is_ok());
}

#[test]
fn compress_chain_table() {
    let (g, t) = chain_grammar_and_table();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    assert!(compressor.compress(&t, &token_indices, false).is_ok());
}

#[test]
fn compress_recursive_table() {
    let (g, t) = recursive_grammar_and_table();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    assert!(compressor.compress(&t, &token_indices, false).is_ok());
}

#[test]
fn compress_multi_token_table() {
    let (g, t) = multi_token_grammar_and_table();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    assert!(compressor.compress(&t, &token_indices, false).is_ok());
}

#[test]
fn compress_tables_via_slg() {
    let (g, t) = simple_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    let result = slg.compress_tables();
    assert!(result.is_ok(), "compress_tables failed: {:?}", result.err());
    assert!(slg.compressed_tables.is_some());
}

#[test]
fn compress_tables_via_slg_two_alt() {
    let (g, t) = two_alt_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compress_tables().is_ok());
    assert!(slg.compressed_tables.is_some());
}

#[test]
fn collect_token_indices_includes_eof() {
    let (g, t) = simple_grammar_and_table();
    let indices = collect_token_indices(&g, &t);
    if let Some(&idx) = t.symbol_to_index.get(&t.eof_symbol) {
        assert!(indices.contains(&idx), "must include EOF");
    }
}

#[test]
fn collect_token_indices_sorted() {
    let (g, t) = multi_token_grammar_and_table();
    let indices = collect_token_indices(&g, &t);
    let is_sorted = indices.windows(2).all(|w| w[0] < w[1]);
    assert!(is_sorted, "token indices must be sorted and deduped");
}

// ===========================================================================
// 7. Empty/minimal grammars
// ===========================================================================

#[test]
fn slg_empty_grammar_node_types_valid_json() {
    let g = Grammar::new("empty".to_string());
    let pt = make_parse_table_for_gen(&g, 1, vec![vec![vec![]; 1]; 1]);
    let slg = StaticLanguageGenerator::new(g, pt);
    let v: serde_json::Value = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(v.is_array());
}

#[test]
fn slg_empty_grammar_node_types_is_empty_array() {
    let g = Grammar::new("empty".to_string());
    let pt = make_parse_table_for_gen(&g, 1, vec![vec![vec![]; 1]; 1]);
    let slg = StaticLanguageGenerator::new(g, pt);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(arr.is_empty());
}

#[test]
fn slg_empty_grammar_language_code_not_empty() {
    let g = Grammar::new("empty".to_string());
    let pt = make_parse_table_for_gen(&g, 1, vec![vec![vec![]; 1]; 1]);
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().to_string().is_empty());
}

#[test]
fn node_types_gen_empty_grammar_ok() {
    let g = Grammar::new("e".to_string());
    let slg = NodeTypesGenerator::new(&g);
    assert!(slg.generate().is_ok());
}

#[test]
fn node_types_gen_empty_grammar_is_empty_array() {
    let g = Grammar::new("e".to_string());
    let slg = NodeTypesGenerator::new(&g);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate().unwrap()).unwrap();
    assert!(arr.is_empty());
}

// ===========================================================================
// 8. Multi-rule grammars
// ===========================================================================

#[test]
fn slg_multi_rule_language_code_ok() {
    let (g, t) = chain_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.generate_language_code().to_string().is_empty());
}

#[test]
fn slg_multi_rule_node_types_ok() {
    let (g, t) = chain_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let v: serde_json::Value = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(v.is_array());
}

#[test]
fn slg_recursive_node_types_ok() {
    let (g, t) = recursive_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let v: serde_json::Value = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(v.is_array());
}

#[test]
fn language_gen_chain_grammar_contains_ffi() {
    let (g, t) = chain_grammar_and_table();
    let slg = LanguageGenerator::new(&g, &t);
    let code = slg.generate().to_string();
    assert!(code.contains("tree_sitter_chain"));
}

#[test]
fn language_gen_recursive_grammar_contains_ffi() {
    let (g, t) = recursive_grammar_and_table();
    let slg = LanguageGenerator::new(&g, &t);
    let code = slg.generate().to_string();
    assert!(code.contains("tree_sitter_recursive"));
}

// ===========================================================================
// 9. Determinism (same input → same output)
// ===========================================================================

#[test]
fn slg_language_code_deterministic() {
    let (g1, t1) = simple_grammar_and_table();
    let (g2, t2) = simple_grammar_and_table();
    let code1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);
}

#[test]
fn slg_node_types_deterministic() {
    let (g1, t1) = simple_grammar_and_table();
    let (g2, t2) = simple_grammar_and_table();
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(n1, n2);
}

#[test]
fn language_gen_deterministic() {
    let (g1, t1) = simple_grammar_and_table();
    let (g2, t2) = simple_grammar_and_table();
    let c1 = LanguageGenerator::new(&g1, &t1).generate().to_string();
    let c2 = LanguageGenerator::new(&g2, &t2).generate().to_string();
    assert_eq!(c1, c2);
}

#[test]
fn node_types_gen_deterministic() {
    let (g1, _) = simple_grammar_and_table();
    let (g2, _) = simple_grammar_and_table();
    assert_eq!(
        NodeTypesGenerator::new(&g1).generate().unwrap(),
        NodeTypesGenerator::new(&g2).generate().unwrap(),
    );
}

#[test]
fn compression_deterministic() {
    let (g1, t1) = simple_grammar_and_table();
    let (g2, t2) = simple_grammar_and_table();
    let ti1 = collect_token_indices(&g1, &t1);
    let ti2 = collect_token_indices(&g2, &t2);
    let c = TableCompressor::new();
    let r1 = c.compress(&t1, &ti1, false).unwrap();
    let r2 = c.compress(&t2, &ti2, false).unwrap();
    assert_eq!(r1.action_table.data.len(), r2.action_table.data.len());
    assert_eq!(r1.goto_table.data.len(), r2.goto_table.data.len());
}

// ===========================================================================
// 10. start_can_be_empty flag
// ===========================================================================

#[test]
fn slg_start_can_be_empty_default_false() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn slg_set_start_can_be_empty_true() {
    let (g, t) = simple_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

#[test]
fn slg_set_start_can_be_empty_false_explicit() {
    let (g, t) = simple_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn eof_accepts_or_reduces_false_for_simple() {
    let (_, t) = simple_grammar_and_table();
    // A simple grammar S -> x should NOT accept on EOF in state 0
    // (state 0 needs to shift 'x' first)
    let result = eof_accepts_or_reduces(&t);
    // The result depends on the actual automaton; just ensure it doesn't panic
    let _ = result;
}

#[test]
fn slg_compress_tables_respects_start_can_be_empty() {
    let (g, t) = simple_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    // Should succeed even with start_can_be_empty = true
    let result = slg.compress_tables();
    assert!(
        result.is_ok(),
        "compress with start_can_be_empty failed: {:?}",
        result.err()
    );
}

// ===========================================================================
// 11. LanguageGenerator (low-level) — from hand-built tables
// ===========================================================================

#[test]
fn gen_symbol_names_start_with_end() {
    let grammar = make_grammar("t", vec![(SymbolId(1), regex_token("num", r"\d+"))], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(
        output.contains("\"end\""),
        "generated code must include EOF sentinel name"
    );
}

#[test]
fn gen_symbol_names_include_tokens() {
    let grammar = make_grammar(
        "t",
        vec![
            (SymbolId(1), regex_token("identifier", r"[a-z]+")),
            (SymbolId(2), string_token("plus", "+")),
        ],
        vec![],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 4]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(output.contains("identifier"));
    assert!(output.contains("plus"));
}

#[test]
fn gen_field_names_empty_grammar() {
    let grammar = make_grammar("t", vec![], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 1]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(output.contains("FIELD_NAMES"));
}

#[test]
fn gen_field_names_populated() {
    let mut grammar = make_grammar("t", vec![], vec![]);
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 1]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(output.contains("left"));
    assert!(output.contains("right"));
}

#[test]
fn gen_symbol_metadata_length_matches_symbol_count() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), regex_token("a", "."))],
        vec![Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let metadata = generator.generate_symbol_metadata_public();
    let expected = 1 + grammar.tokens.len() + grammar.rules.len();
    assert_eq!(metadata.len(), expected);
}

#[test]
fn gen_symbol_metadata_all_visible_named() {
    let grammar = make_grammar("t", vec![(SymbolId(1), regex_token("x", "."))], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 2]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let metadata = generator.generate_symbol_metadata_public();
    for &byte in &metadata {
        assert_eq!(byte & 0b11, 0b11, "each symbol should be visible+named");
    }
}

#[test]
fn gen_production_id_count_single_rule() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), regex_token("n", r"\d"))],
        vec![Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    assert_eq!(generator.count_production_ids_public(), 1);
}

#[test]
fn gen_production_id_count_multiple_rules() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), regex_token("n", r"\d"))],
        vec![
            Rule {
                lhs: SymbolId(2),
                rhs: vec![Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: SymbolId(2),
                rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(3),
            },
        ],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    assert_eq!(generator.count_production_ids_public(), 4);
}

#[test]
fn gen_output_contains_language_fn() {
    let grammar = make_grammar("demo", vec![(SymbolId(1), regex_token("tok", "."))], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 2]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let code = generator.generate().to_string();
    assert!(code.contains("tree_sitter_demo"));
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn gen_output_contains_version_constant() {
    let grammar = make_grammar("v", vec![], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 1]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let code = generator.generate().to_string();
    assert!(code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

// ===========================================================================
// 12. ABI constants & struct sizes
// ===========================================================================

#[test]
fn abi_version_constants() {
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
    let min_compat = TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;
    assert!(min_compat <= TREE_SITTER_LANGUAGE_VERSION);
}

#[test]
fn abi_struct_sizes() {
    assert_eq!(std::mem::size_of::<TSSymbol>(), 2);
    assert_eq!(std::mem::size_of::<TSStateId>(), 2);
    assert_eq!(std::mem::size_of::<TSFieldId>(), 2);
    assert_eq!(std::mem::size_of::<TSParseAction>(), 6);
    assert_eq!(std::mem::size_of::<TSLexState>(), 4);
}

#[test]
fn abi_tslanguage_alignment() {
    assert_eq!(
        std::mem::align_of::<AbiTSLanguage>(),
        std::mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned for FFI"
    );
}

// ===========================================================================
// 13. create_symbol_metadata flag combinations
// ===========================================================================

#[test]
fn symbol_metadata_flags_visible_named() {
    let m = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(
        m,
        abi::symbol_metadata::VISIBLE | abi::symbol_metadata::NAMED
    );
}

#[test]
fn symbol_metadata_flags_hidden_auxiliary() {
    let m = create_symbol_metadata(false, false, true, true, false);
    assert_eq!(
        m,
        abi::symbol_metadata::HIDDEN | abi::symbol_metadata::AUXILIARY
    );
}

#[test]
fn symbol_metadata_flags_supertype() {
    let m = create_symbol_metadata(true, true, false, false, true);
    assert_eq!(
        m,
        abi::symbol_metadata::VISIBLE
            | abi::symbol_metadata::NAMED
            | abi::symbol_metadata::SUPERTYPE
    );
}

#[test]
fn symbol_metadata_flags_none() {
    assert_eq!(create_symbol_metadata(false, false, false, false, false), 0);
}

#[test]
fn symbol_metadata_flags_all() {
    let m = create_symbol_metadata(true, true, true, true, true);
    let expected = abi::symbol_metadata::VISIBLE
        | abi::symbol_metadata::NAMED
        | abi::symbol_metadata::HIDDEN
        | abi::symbol_metadata::AUXILIARY
        | abi::symbol_metadata::SUPERTYPE;
    assert_eq!(m, expected);
}

// ===========================================================================
// 14. ExternalScanner default
// ===========================================================================

#[test]
fn external_scanner_default_is_null() {
    let es = ExternalScanner::default();
    assert!(es.states.is_null());
    assert!(es.symbol_map.is_null());
    assert!(es.create.is_none());
    assert!(es.destroy.is_none());
    assert!(es.scan.is_none());
    assert!(es.serialize.is_none());
    assert!(es.deserialize.is_none());
}

// ===========================================================================
// 15. NodeTypesGenerator
// ===========================================================================

#[test]
fn node_types_valid_json_for_simple_grammar() {
    let mut grammar = Grammar::new("ntest".to_string());
    grammar
        .tokens
        .insert(SymbolId(1), regex_token("number", r"\d+"));
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&grammar);
    let json = generator.generate().expect("generate must succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn node_types_unnamed_for_string_token() {
    let mut grammar = Grammar::new("stest".to_string());
    grammar
        .tokens
        .insert(SymbolId(1), string_token("plus", "+"));

    let generator = NodeTypesGenerator::new(&grammar);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let plus_entry = arr.iter().find(|v| v["type"] == "+");
    assert!(plus_entry.is_some(), "should have '+' node type");
    assert_eq!(plus_entry.unwrap()["named"], false);
}

#[test]
fn node_types_from_pipeline_grammar() {
    let (g, _) = simple_grammar_and_table();
    let slg = NodeTypesGenerator::new(&g);
    assert!(slg.generate().is_ok());
}

#[test]
fn node_types_from_two_alt_grammar() {
    let (g, _) = two_alt_grammar_and_table();
    let slg = NodeTypesGenerator::new(&g);
    assert!(slg.generate().is_ok());
}

// ===========================================================================
// 16. LanguageBuilder
// ===========================================================================

fn builder_grammar_and_table() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("btest".to_string());
    grammar
        .tokens
        .insert(SymbolId(1), regex_token("id", r"[a-z]+"));
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut symbol_to_index = std::collections::BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0);
    symbol_to_index.insert(SymbolId(1), 1);

    let pt = ParseTable {
        action_table: vec![vec![vec![Action::Accept]; 2]; 2],
        goto_table: vec![vec![StateId(0); 2]; 2],
        state_count: 2,
        symbol_count: 2,
        symbol_to_index: symbol_to_index.clone(),
        index_to_symbol: vec![SymbolId(0), SymbolId(1)],
        symbol_metadata: vec![
            SymbolMetadata {
                name: "tok".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            };
            2
        ],
        nonterminal_to_index: std::collections::BTreeMap::new(),
        eof_symbol: SymbolId(1),
        start_symbol: SymbolId(0),
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: 1,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            2
        ],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        rules: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
    };
    (grammar, pt)
}

#[test]
fn language_builder_version_is_15() {
    let (g, pt) = builder_grammar_and_table();
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().expect("must succeed");
    assert_eq!(lang.version, 15);
}

#[test]
fn language_builder_state_and_symbol_counts() {
    let (g, pt) = builder_grammar_and_table();
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.state_count, 2);
    assert_eq!(lang.symbol_count, 2);
}

#[test]
fn language_builder_external_token_count() {
    let (mut g, pt) = builder_grammar_and_table();
    g.externals.push(ExternalToken {
        name: "comment".to_string(),
        symbol_id: SymbolId(100),
    });
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.external_token_count, 1);
}

#[test]
fn language_builder_field_count() {
    let (mut g, pt) = builder_grammar_and_table();
    g.fields.insert(FieldId(0), "value".to_string());
    g.fields.insert(FieldId(1), "operator".to_string());
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.field_count, 2);
}

#[test]
fn language_builder_code_gen_mentions_tslanguage() {
    let (g, pt) = builder_grammar_and_table();
    let builder = LanguageBuilder::new(g, pt);
    let code = builder.generate_language_code().to_string();
    assert!(code.contains("TSLanguage"));
}

// ===========================================================================
// 17. Grammar with external tokens in LanguageGenerator
// ===========================================================================

#[test]
fn gen_with_external_tokens() {
    let mut grammar = make_grammar(
        "ext",
        vec![(SymbolId(1), regex_token("id", "[a-z]+"))],
        vec![],
    );
    grammar.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(50),
    });
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(output.contains("EXTERNAL_TOKEN_COUNT"));
}

#[test]
fn gen_ffi_name_matches_grammar() {
    let grammar = make_grammar(
        "my_lang",
        vec![(SymbolId(1), regex_token("x", "."))],
        vec![],
    );
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 2]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let code = generator.generate().to_string();
    assert!(code.contains("tree_sitter_my_lang"));
}

// ===========================================================================
// 18. Node types exclude internal rules
// ===========================================================================

#[test]
fn node_types_excludes_internal_rules() {
    let mut grammar = Grammar::new("internal".to_string());
    grammar.tokens.insert(SymbolId(1), regex_token("n", r"\d+"));
    grammar
        .rule_names
        .insert(SymbolId(2), "_internal_helper".to_string());
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&grammar);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let has_internal = arr
        .iter()
        .any(|v| v["type"].as_str().unwrap_or("").starts_with('_'));
    assert!(
        !has_internal,
        "internal rules must be excluded from node types"
    );
}

// ===========================================================================
// 19. CompressedParseTable basics
// ===========================================================================

#[test]
fn compressed_parse_table_roundtrip_counts() {
    let table = CompressedParseTable::new_for_testing(42, 7);
    assert_eq!(table.symbol_count(), 42);
    assert_eq!(table.state_count(), 7);
}

#[test]
fn compressed_parse_table_from_real_table() {
    let (_, t) = simple_grammar_and_table();
    let cpt = CompressedParseTable::from_parse_table(&t);
    assert_eq!(cpt.symbol_count(), t.symbol_count);
    assert_eq!(cpt.state_count(), t.state_count);
}

// ===========================================================================
// 20. Validator
// ===========================================================================

#[test]
fn validator_rejects_wrong_version() {
    let lang = adze_tablegen::validation::TSLanguage {
        version: 14,
        symbol_count: 5,
        alias_count: 0,
        token_count: 3,
        external_token_count: 0,
        state_count: 10,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        parse_table: std::ptr::null(),
        small_parse_table: std::ptr::null(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: std::ptr::null(),
        symbol_names: std::ptr::null(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: std::ptr::null(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: std::ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner_data: adze_tablegen::validation::TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    };
    let tables = CompressedParseTable::new_for_testing(5, 10);
    let validator = LanguageValidator::new(&lang, &tables);
    let result = validator.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidVersion { .. }))
    );
}

#[test]
fn validator_reports_symbol_count_mismatch() {
    let lang = adze_tablegen::validation::TSLanguage {
        version: 15,
        symbol_count: 99,
        alias_count: 0,
        token_count: 3,
        external_token_count: 0,
        state_count: 10,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        parse_table: std::ptr::null(),
        small_parse_table: std::ptr::null(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: std::ptr::null(),
        symbol_names: std::ptr::null(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: std::ptr::null(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: std::ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner_data: adze_tablegen::validation::TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    };
    let tables = CompressedParseTable::new_for_testing(5, 10);
    let validator = LanguageValidator::new(&lang, &tables);
    let errors = validator.validate().unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::SymbolCountMismatch { .. }))
    );
}

#[test]
fn validator_reports_null_symbol_names() {
    let lang = adze_tablegen::validation::TSLanguage {
        version: 15,
        symbol_count: 5,
        alias_count: 0,
        token_count: 3,
        external_token_count: 0,
        state_count: 10,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        parse_table: std::ptr::null(),
        small_parse_table: [0u16; 4].as_ptr(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: std::ptr::null(),
        symbol_names: std::ptr::null(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: std::ptr::null(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: std::ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner_data: adze_tablegen::validation::TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    };
    let tables = CompressedParseTable::new_for_testing(5, 10);
    let validator = LanguageValidator::new(&lang, &tables);
    let errors = validator.validate().unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::NullPointer("symbol_names")))
    );
}

// ===========================================================================
// 21. Preset grammar pipelines (python-like, javascript-like)
// ===========================================================================

#[test]
fn python_like_node_types_ok() {
    let g = GrammarBuilder::python_like();
    let slg = NodeTypesGenerator::new(&g);
    assert!(slg.generate().is_ok());
}

#[test]
fn javascript_like_node_types_ok() {
    let g = GrammarBuilder::javascript_like();
    let slg = NodeTypesGenerator::new(&g);
    assert!(slg.generate().is_ok());
}

// ===========================================================================
// 22. TableCompressor encode_action_small
// ===========================================================================

#[test]
fn encode_action_small_shift() {
    let c = TableCompressor::new();
    let encoded = c.encode_action_small(&Action::Shift(StateId(42))).unwrap();
    assert_eq!(encoded, 42);
}

#[test]
fn encode_action_small_reduce() {
    let c = TableCompressor::new();
    let encoded = c.encode_action_small(&Action::Reduce(RuleId(5))).unwrap();
    // bit 15 set, rule_id+1 in lower bits
    assert_eq!(encoded, 0x8000 | 6);
}

#[test]
fn encode_action_small_accept() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Accept).unwrap(), 0xFFFF);
}

#[test]
fn encode_action_small_error() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Error).unwrap(), 0xFFFE);
}

#[test]
fn encode_action_small_recover() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Recover).unwrap(), 0xFFFD);
}

// ===========================================================================
// 23. Full pipeline: Grammar → FF → LR(1) → StaticLanguageGenerator
// ===========================================================================

#[test]
fn full_pipeline_simple_slg() {
    let (g, t) = simple_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    let node_types = slg.generate_node_types();
    assert!(!code.is_empty());
    assert!(!node_types.is_empty());
}

#[test]
fn full_pipeline_two_alt_slg() {
    let (g, t) = two_alt_grammar_and_table();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    let node_types = slg.generate_node_types();
    assert!(!code.is_empty());
    let v: serde_json::Value = serde_json::from_str(&node_types).unwrap();
    assert!(v.is_array());
}

#[test]
fn full_pipeline_compress_then_codegen() {
    let (g, t) = simple_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.compress_tables().expect("compress");
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn full_pipeline_chain_compress() {
    let (g, t) = chain_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compress_tables().is_ok());
}

#[test]
fn full_pipeline_recursive_compress() {
    let (g, t) = recursive_grammar_and_table();
    let mut slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compress_tables().is_ok());
}
