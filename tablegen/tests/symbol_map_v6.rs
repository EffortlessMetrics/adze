#![allow(clippy::needless_range_loop)]

//! Tests for symbol mapping, indexing, and name-to-ID resolution in table
//! generation.  64 tests across 8 categories (8 each):
//!
//! 1. sym_name_*       — symbol name resolution
//! 2. sym_index_*      — symbol indexing operations
//! 3. sym_token_*      — token symbol mapping
//! 4. sym_rule_*       — rule symbol mapping
//! 5. sym_field_*      — field symbol mapping
//! 6. sym_external_*   — external symbol mapping
//! 7. sym_alias_*      — alias symbol resolution
//! 8. sym_roundtrip_*  — symbol mapping roundtrip verification

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    AliasSequence, ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId,
    Token, TokenPattern,
};
use adze_tablegen::StaticLanguageGenerator;
use adze_tablegen::abi_builder::AbiLanguageBuilder;
use adze_tablegen::language_gen::LanguageGenerator;
use adze_tablegen::serializer::serialize_language;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

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

fn simple_rule(lhs: u16, rhs: Vec<Symbol>, prod_id: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        production_id: ProductionId(prod_id),
        fields: vec![],
        precedence: None,
        associativity: None,
    }
}

fn make_grammar(
    name: &str,
    tokens: Vec<(SymbolId, Token)>,
    rules: Vec<Rule>,
    rule_names: Vec<(SymbolId, String)>,
) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    for (id, tok) in tokens {
        g.tokens.insert(id, tok);
    }
    for rule in rules {
        g.add_rule(rule);
    }
    for (id, rn) in rule_names {
        g.rule_names.insert(id, rn);
    }
    g
}

/// Build a ParseTable for AbiLanguageBuilder with explicit symbol_to_index.
fn make_abi_table(
    grammar: &Grammar,
    symbol_to_index: BTreeMap<SymbolId, usize>,
    eof: SymbolId,
) -> ParseTable {
    let symbol_count = symbol_to_index.len();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (&sid, &idx) in &symbol_to_index {
        if idx < symbol_count {
            index_to_symbol[idx] = sid;
        }
    }
    ParseTable {
        action_table: vec![vec![vec![]; symbol_count]; 1],
        goto_table: vec![vec![INVALID; symbol_count]; 1],
        rules: vec![],
        state_count: 1,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol: eof,
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build a ParseTable for LanguageGenerator (identity index 0..N).
fn make_lang_gen_table(grammar: &Grammar, states: usize, symbol_count: usize) -> ParseTable {
    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();
    ParseTable {
        action_table: vec![vec![vec![]; symbol_count]; states],
        goto_table: vec![vec![INVALID; symbol_count]; states],
        rules: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            states
        ],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Identity-indexed symbol_to_index map for 0..count.
fn identity_s2i(count: usize) -> BTreeMap<SymbolId, usize> {
    let mut m = BTreeMap::new();
    for i in 0..count {
        m.insert(SymbolId(i as u16), i);
    }
    m
}

/// Render ABI builder output as String.
fn abi_output(grammar: &Grammar, s2i: BTreeMap<SymbolId, usize>, eof: SymbolId) -> String {
    let pt = make_abi_table(grammar, s2i, eof);
    AbiLanguageBuilder::new(grammar, &pt).generate().to_string()
}

/// Render LanguageGenerator output as String.
fn lang_gen_output(grammar: &Grammar, pt: &ParseTable) -> String {
    LanguageGenerator::new(grammar, pt).generate().to_string()
}

/// Deserialize JSON from serialize_language into symbol_names.
fn serialized_names(grammar: &Grammar, pt: &ParseTable) -> Vec<String> {
    let json = serialize_language(grammar, pt, None).expect("serialization succeeds");
    let lang: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    lang["symbol_names"]
        .as_array()
        .expect("symbol_names array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// Check that a symbol name's null-terminated byte encoding appears in ABI
/// output (byte-array style: `112u8 , 108u8 , ...`).
fn abi_has_symbol_name(output: &str, name: &str) -> bool {
    let bytes: Vec<u8> = format!("{name}\0").into_bytes();
    let byte_strs: Vec<String> = bytes.iter().map(|b| format!("{b}u8")).collect();
    let first = &byte_strs[0];
    if let Some(start) = output.find(first) {
        let after = &output[start..];
        byte_strs.iter().all(|bs| after.contains(bs))
    } else {
        false
    }
}

// ===========================================================================
// 1. sym_name — symbol name resolution
// ===========================================================================

#[test]
fn sym_name_eof_sentinel_is_end_in_serialized() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let names = serialized_names(&grammar, &pt);
    assert_eq!(names[0], "end");
}

#[test]
fn sym_name_token_appears_in_lang_gen() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token("identifier", "[a-z]+"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let output = lang_gen_output(&grammar, &pt);
    assert!(output.contains("identifier"), "token name must appear");
}

#[test]
fn sym_name_rule_named_appears_in_abi() {
    let grammar = make_grammar(
        "g",
        vec![],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "expression".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(abi_has_symbol_name(&output, "expression"));
}

#[test]
fn sym_name_unnamed_rule_gets_generated_name() {
    let grammar = make_grammar("g", vec![], vec![simple_rule(10, vec![], 0)], vec![]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(abi_has_symbol_name(&output, "rule_10"));
}

#[test]
fn sym_name_multiple_tokens_all_present_serialized() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("plus", "+")),
            (SymbolId(2), string_token("minus", "-")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"plus".to_string()));
    assert!(names.contains(&"minus".to_string()));
}

#[test]
fn sym_name_hidden_token_still_in_serialized() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("_ws", " "))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"_ws".to_string()));
}

#[test]
fn sym_name_eof_always_first_in_serialized() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), regex_token("alpha", "[a-z]")),
            (SymbolId(2), regex_token("digit", "[0-9]")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    assert_eq!(names[0], "end", "EOF must be first");
}

#[test]
fn sym_name_string_token_bytes_in_abi() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("star", "*"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // 's' = 115
    assert!(output.contains("115u8"), "'s' byte must appear");
}

// ===========================================================================
// 2. sym_index — symbol indexing operations
// ===========================================================================

#[test]
fn sym_index_identity_mapping_preserved() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "start".to_string())],
    );
    let s2i = identity_s2i(3);
    let pt = make_abi_table(&grammar, s2i.clone(), SymbolId(0));
    for (&sid, &idx) in &s2i {
        assert_eq!(pt.index_to_symbol[idx], sid);
    }
}

#[test]
fn sym_index_count_matches_symbol_to_index_len() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![],
        vec![],
    );
    let s2i = identity_s2i(3);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    assert_eq!(pt.symbol_count, pt.symbol_to_index.len());
}

#[test]
fn sym_index_eof_at_zero() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    assert_eq!(pt.index_to_symbol[0], SymbolId(0));
    assert_eq!(pt.eof_symbol, SymbolId(0));
}

#[test]
fn sym_index_non_contiguous_ids_mapped() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(5), string_token("tok", "t"))],
        vec![simple_rule(20, vec![], 0)],
        vec![(SymbolId(20), "rule_a".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(5), 1);
    s2i.insert(SymbolId(20), 2);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    assert_eq!(pt.index_to_symbol[1], SymbolId(5));
    assert_eq!(pt.index_to_symbol[2], SymbolId(20));
}

#[test]
fn sym_index_symbol_count_in_abi_output() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "start".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    s2i.insert(SymbolId(2), 2);
    s2i.insert(SymbolId(10), 3);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(output.contains("symbol_count : 4"), "4 symbols total");
}

#[test]
fn sym_index_index_to_symbol_roundtrip() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    let s2i = identity_s2i(2);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    for (idx, &sid) in pt.index_to_symbol.iter().enumerate() {
        assert_eq!(pt.symbol_to_index[&sid], idx);
    }
}

#[test]
fn sym_index_large_symbol_set() {
    let mut tokens = Vec::new();
    for i in 1..=50u16 {
        tokens.push((
            SymbolId(i),
            string_token(&format!("t{i}"), &format!("v{i}")),
        ));
    }
    let grammar = make_grammar("g", tokens, vec![], vec![]);
    let s2i = identity_s2i(51);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    assert_eq!(pt.symbol_count, 51);
    assert_eq!(pt.index_to_symbol.len(), 51);
}

#[test]
fn sym_index_single_symbol_table() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    assert_eq!(pt.symbol_count, 1);
    assert_eq!(pt.index_to_symbol.len(), 1);
}

// ===========================================================================
// 3. sym_token — token symbol mapping
// ===========================================================================

#[test]
fn sym_token_string_token_in_serialized() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("semicolon", ";"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"semicolon".to_string()));
}

#[test]
fn sym_token_regex_token_in_serialized() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), regex_token("number", r"\d+"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"number".to_string()));
}

#[test]
fn sym_token_count_in_serialized() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
            (SymbolId(3), regex_token("c", "c")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["token_count"].as_u64().unwrap(), 3);
}

#[test]
fn sym_token_order_follows_id() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("alpha", "a")),
            (SymbolId(2), string_token("bravo", "b")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    let alpha_pos = names.iter().position(|n| n == "alpha").unwrap();
    let bravo_pos = names.iter().position(|n| n == "bravo").unwrap();
    assert!(alpha_pos < bravo_pos, "alpha before bravo by SymbolId");
}

#[test]
fn sym_token_fragile_flag_does_not_affect_name() {
    let mut token = string_token("kw", "if");
    token.fragile = true;
    let grammar = make_grammar("g", vec![(SymbolId(1), token)], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"kw".to_string()));
}

#[test]
fn sym_token_abi_byte_encoding_for_short_name() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("ab", "x"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // "ab\0" → 97u8, 98u8, 0u8
    assert!(output.contains("97u8"), "'a' byte");
    assert!(output.contains("98u8"), "'b' byte");
    assert!(output.contains("0u8"), "null terminator");
}

#[test]
fn sym_token_in_lang_gen_output() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("dot", "."))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let output = lang_gen_output(&grammar, &pt);
    assert!(output.contains("dot"), "token must appear in SYMBOL_NAMES");
}

#[test]
fn sym_token_many_tokens_all_present() {
    let tokens: Vec<_> = (1..=10u16)
        .map(|i| {
            (
                SymbolId(i),
                string_token(&format!("tok_{i}"), &format!("v{i}")),
            )
        })
        .collect();
    let grammar = make_grammar("g", tokens, vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 11);
    let names = serialized_names(&grammar, &pt);
    for i in 1..=10 {
        assert!(
            names.contains(&format!("tok_{i}")),
            "tok_{i} must be present"
        );
    }
}

// ===========================================================================
// 4. sym_rule — rule symbol mapping
// ===========================================================================

#[test]
fn sym_rule_single_named_rule() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "program".to_string())],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"program".to_string()));
}

#[test]
fn sym_rule_multiple_rules_all_named() {
    let grammar = make_grammar(
        "g",
        vec![],
        vec![simple_rule(10, vec![], 0), simple_rule(11, vec![], 1)],
        vec![
            (SymbolId(10), "stmt".to_string()),
            (SymbolId(11), "decl".to_string()),
        ],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"stmt".to_string()));
    assert!(names.contains(&"decl".to_string()));
}

#[test]
fn sym_rule_unnamed_gets_rule_prefix() {
    let grammar = make_grammar("g", vec![], vec![simple_rule(7, vec![], 0)], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"rule_7".to_string()));
}

#[test]
fn sym_rule_lhs_id_preserved_in_table() {
    let grammar = make_grammar(
        "g",
        vec![],
        vec![simple_rule(15, vec![], 0)],
        vec![(SymbolId(15), "func".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(15), 1);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    assert_eq!(pt.symbol_to_index[&SymbolId(15)], 1);
    assert_eq!(pt.index_to_symbol[1], SymbolId(15));
}

#[test]
fn sym_rule_with_rhs_terminals() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("lp", "(")),
            (SymbolId(2), string_token("rp", ")")),
        ],
        vec![simple_rule(
            10,
            vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
            0,
        )],
        vec![(SymbolId(10), "group".to_string())],
    );
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"lp".to_string()));
    assert!(names.contains(&"rp".to_string()));
    assert!(names.contains(&"group".to_string()));
}

#[test]
fn sym_rule_with_nonterminal_rhs() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![
            simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0),
            simple_rule(11, vec![Symbol::NonTerminal(SymbolId(10))], 1),
        ],
        vec![
            (SymbolId(10), "inner".to_string()),
            (SymbolId(11), "outer".to_string()),
        ],
    );
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"inner".to_string()));
    assert!(names.contains(&"outer".to_string()));
}

#[test]
fn sym_rule_supertype_in_abi() {
    let mut grammar = make_grammar(
        "g",
        vec![],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "expr".to_string())],
    );
    grammar.supertypes.push(SymbolId(10));
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(abi_has_symbol_name(&output, "expr"));
}

#[test]
fn sym_rule_many_rules_in_serialized() {
    let rules: Vec<Rule> = (10..=17u16)
        .map(|i| simple_rule(i, vec![], i - 10))
        .collect();
    let rule_names: Vec<(SymbolId, String)> = (10..=17u16)
        .map(|i| (SymbolId(i), format!("rule_{i}")))
        .collect();
    let grammar = make_grammar("g", vec![], rules, rule_names);
    let pt = make_lang_gen_table(&grammar, 1, 9);
    let names = serialized_names(&grammar, &pt);
    for i in 10..=17 {
        assert!(names.contains(&format!("rule_{i}")));
    }
}

// ===========================================================================
// 5. sym_field — field symbol mapping
// ===========================================================================

#[test]
fn sym_field_single_field_in_serialized() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            production_id: ProductionId(0),
            fields: vec![(FieldId(0), 0)],
            precedence: None,
            associativity: None,
        }],
        vec![(SymbolId(10), "program".to_string())],
    );
    grammar.fields.insert(FieldId(0), "value".to_string());
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let field_names = val["field_names"].as_array().unwrap();
    let names: Vec<&str> = field_names.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(names.contains(&"value"));
}

#[test]
fn sym_field_count_in_serialized() {
    let mut grammar = make_grammar("g", vec![], vec![simple_rule(10, vec![], 0)], vec![]);
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["field_count"].as_u64().unwrap(), 2);
}

#[test]
fn sym_field_lexicographic_ordering() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    grammar.fields.insert(FieldId(0), "zebra".to_string());
    grammar.fields.insert(FieldId(1), "alpha".to_string());
    grammar.fields.insert(FieldId(2), "mid".to_string());
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let field_names: Vec<String> = val["field_names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let mut sorted = field_names.clone();
    sorted.sort();
    assert_eq!(field_names, sorted, "fields must be lexicographic");
}

#[test]
fn sym_field_empty_fields() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["field_count"].as_u64().unwrap(), 0);
    let field_names = val["field_names"].as_array().unwrap();
    assert!(field_names.is_empty());
}

#[test]
fn sym_field_abi_output_contains_field_names() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            production_id: ProductionId(0),
            fields: vec![(FieldId(0), 0)],
            precedence: None,
            associativity: None,
        }],
        vec![(SymbolId(10), "stmt".to_string())],
    );
    grammar.fields.insert(FieldId(0), "body".to_string());
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    s2i.insert(SymbolId(10), 2);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        output.contains("FIELD_NAMES") || output.contains("field_names"),
        "field names section must be present"
    );
}

#[test]
fn sym_field_multiple_fields_all_present() {
    let mut grammar = make_grammar("g", vec![], vec![simple_rule(10, vec![], 0)], vec![]);
    grammar.fields.insert(FieldId(0), "condition".to_string());
    grammar.fields.insert(FieldId(1), "body".to_string());
    grammar.fields.insert(FieldId(2), "alternative".to_string());
    let pt = make_lang_gen_table(&grammar, 1, 2);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let field_names: Vec<String> = val["field_names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(field_names.contains(&"condition".to_string()));
    assert!(field_names.contains(&"body".to_string()));
    assert!(field_names.contains(&"alternative".to_string()));
}

#[test]
fn sym_field_field_id_in_rule() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        production_id: ProductionId(0),
        fields: vec![(FieldId(0), 0), (FieldId(1), 0)],
        precedence: None,
        associativity: None,
    };
    assert_eq!(rule.fields.len(), 2);
    assert_eq!(rule.fields[0].0, FieldId(0));
    assert_eq!(rule.fields[1].0, FieldId(1));
}

#[test]
fn sym_field_field_count_matches_grammar() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    for i in 0..5u16 {
        grammar.fields.insert(FieldId(i), format!("field_{i}"));
    }
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["field_count"].as_u64().unwrap(), 5);
}

// ===========================================================================
// 6. sym_external — external symbol mapping
// ===========================================================================

#[test]
fn sym_external_single_external_in_serialized() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    grammar.externals.push(ExternalToken {
        name: "ext_newline".to_string(),
        symbol_id: SymbolId(50),
    });
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"ext_newline".to_string()));
}

#[test]
fn sym_external_count_in_serialized() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "ext_a".to_string(),
        symbol_id: SymbolId(50),
    });
    grammar.externals.push(ExternalToken {
        name: "ext_b".to_string(),
        symbol_id: SymbolId(51),
    });
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["external_token_count"].as_u64().unwrap(), 2);
}

#[test]
fn sym_external_names_after_rules_in_serialized() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("tok", "t"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "start".to_string())],
    );
    grammar.externals.push(ExternalToken {
        name: "ext_indent".to_string(),
        symbol_id: SymbolId(60),
    });
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let names = serialized_names(&grammar, &pt);
    let rule_pos = names.iter().position(|n| n == "start").unwrap();
    let ext_pos = names.iter().position(|n| n == "ext_indent").unwrap();
    assert!(
        ext_pos > rule_pos,
        "externals come after rules in serialized output"
    );
}

#[test]
fn sym_external_in_abi_output() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "ext_scanner".to_string(),
        symbol_id: SymbolId(100),
    });
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(100), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        output.contains("external_token_count : 1"),
        "external_token_count must be 1"
    );
}

#[test]
fn sym_external_empty_externals() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["external_token_count"].as_u64().unwrap(), 0);
}

#[test]
fn sym_external_multiple_externals_all_present() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    for i in 0..4u16 {
        grammar.externals.push(ExternalToken {
            name: format!("ext_{i}"),
            symbol_id: SymbolId(50 + i),
        });
    }
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let names = serialized_names(&grammar, &pt);
    for i in 0..4 {
        assert!(names.contains(&format!("ext_{i}")));
    }
}

#[test]
fn sym_external_symbol_id_not_in_tokens() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    grammar.externals.push(ExternalToken {
        name: "ext_scan".to_string(),
        symbol_id: SymbolId(99),
    });
    // External symbol IDs should not collide with tokens
    assert!(!grammar.tokens.contains_key(&SymbolId(99)));
}

#[test]
fn sym_external_preserves_declaration_order() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "ext_z".to_string(),
        symbol_id: SymbolId(50),
    });
    grammar.externals.push(ExternalToken {
        name: "ext_a".to_string(),
        symbol_id: SymbolId(51),
    });
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let names = serialized_names(&grammar, &pt);
    let z_pos = names.iter().position(|n| n == "ext_z").unwrap();
    let a_pos = names.iter().position(|n| n == "ext_a").unwrap();
    assert!(z_pos < a_pos, "externals should preserve declaration order");
}

// ===========================================================================
// 7. sym_alias — alias symbol resolution
// ===========================================================================

#[test]
fn sym_alias_empty_alias_sequences() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    assert!(grammar.alias_sequences.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

#[test]
fn sym_alias_single_alias_in_grammar() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "program".to_string())],
    );
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("renamed".to_string())],
        },
    );
    grammar.max_alias_sequence_length = 1;
    assert_eq!(grammar.alias_sequences.len(), 1);
    let seq = &grammar.alias_sequences[&ProductionId(0)];
    assert_eq!(seq.aliases[0], Some("renamed".to_string()));
}

#[test]
fn sym_alias_alias_count_in_serialized() {
    let grammar = make_grammar("g", vec![], vec![], vec![]);
    let pt = make_lang_gen_table(&grammar, 1, 1);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    // alias_count is 0 for grammars without aliases
    assert_eq!(val["alias_count"].as_u64().unwrap(), 0);
}

#[test]
fn sym_alias_sequence_with_none_entries() {
    let seq = AliasSequence {
        aliases: vec![None, Some("alias1".to_string()), None],
    };
    assert_eq!(seq.aliases.len(), 3);
    assert!(seq.aliases[0].is_none());
    assert_eq!(seq.aliases[1], Some("alias1".to_string()));
    assert!(seq.aliases[2].is_none());
}

#[test]
fn sym_alias_multiple_sequences() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("a1".to_string())],
        },
    );
    grammar.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![Some("a2".to_string()), Some("a3".to_string())],
        },
    );
    grammar.max_alias_sequence_length = 2;
    assert_eq!(grammar.alias_sequences.len(), 2);
}

#[test]
fn sym_alias_max_length_tracked() {
    let mut grammar = make_grammar("g", vec![], vec![], vec![]);
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![
                Some("a".to_string()),
                None,
                Some("b".to_string()),
                None,
                Some("c".to_string()),
            ],
        },
    );
    grammar.max_alias_sequence_length = 5;
    assert_eq!(grammar.max_alias_sequence_length, 5);
}

#[test]
fn sym_alias_does_not_affect_symbol_names() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "program".to_string())],
    );
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("alias_name".to_string())],
        },
    );
    grammar.max_alias_sequence_length = 1;
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    // Original names should still be present
    assert!(names.contains(&"x".to_string()));
    assert!(names.contains(&"program".to_string()));
}

#[test]
fn sym_alias_empty_alias_string() {
    let seq = AliasSequence {
        aliases: vec![Some(String::new())],
    };
    assert_eq!(seq.aliases[0], Some(String::new()));
}

// ===========================================================================
// 8. sym_roundtrip — symbol mapping roundtrip verification
// ===========================================================================

#[test]
fn sym_roundtrip_index_to_symbol_and_back() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "start".to_string())],
    );
    let s2i = identity_s2i(4);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    for idx in 0..pt.symbol_count {
        let sid = pt.index_to_symbol[idx];
        assert_eq!(pt.symbol_to_index[&sid], idx, "roundtrip at index {idx}");
    }
}

#[test]
fn sym_roundtrip_serialized_names_match_lang_gen() {
    // LanguageGenerator resolves names from parse_table.index_to_symbol → grammar,
    // so we only check that token names present in the serialized output also
    // appear in the LanguageGenerator output.
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("plus", "+")),
            (SymbolId(2), regex_token("num", r"\d+")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let names = serialized_names(&grammar, &pt);
    let lang_output = lang_gen_output(&grammar, &pt);
    for name in &names {
        if name != "end" {
            assert!(
                lang_output.contains(name),
                "name '{name}' must appear in lang gen output"
            );
        }
    }
}

#[test]
fn sym_roundtrip_abi_names_match_serialized() {
    let grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("kw", "if"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "stmt".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    s2i.insert(SymbolId(10), 2);
    let abi_out = abi_output(&grammar, s2i.clone(), SymbolId(0));
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    let names = serialized_names(&grammar, &pt);
    // Every serialized name should appear as bytes in ABI output
    for name in &names {
        if name != "end" {
            assert!(
                abi_has_symbol_name(&abi_out, name),
                "name '{name}' must appear in ABI output"
            );
        }
    }
}

#[test]
fn sym_roundtrip_static_gen_contains_all_tokens() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("lp", "(")),
            (SymbolId(2), string_token("rp", ")")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 1, 3);
    let static_gen = StaticLanguageGenerator::new(grammar.clone(), pt);
    let output = static_gen.generate_language_code().to_string();
    assert!(output.contains("lp"));
    assert!(output.contains("rp"));
}

#[test]
fn sym_roundtrip_symbol_count_consistent() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "rule_a".to_string())],
    );
    let s2i = identity_s2i(4);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let serialized_count = val["symbol_names"].as_array().unwrap().len();
    // Serialized names include EOF + tokens + rules + externals
    let expected = 1 + grammar.tokens.len() + grammar.rules.len() + grammar.externals.len();
    assert_eq!(serialized_count, expected);
}

#[test]
fn sym_roundtrip_find_symbol_by_name() {
    let grammar = make_grammar(
        "g",
        vec![],
        vec![simple_rule(10, vec![], 0), simple_rule(11, vec![], 1)],
        vec![
            (SymbolId(10), "alpha".to_string()),
            (SymbolId(11), "beta".to_string()),
        ],
    );
    assert_eq!(grammar.find_symbol_by_name("alpha"), Some(SymbolId(10)));
    assert_eq!(grammar.find_symbol_by_name("beta"), Some(SymbolId(11)));
    assert_eq!(grammar.find_symbol_by_name("gamma"), None);
}

#[test]
fn sym_roundtrip_deterministic_output() {
    let grammar = make_grammar(
        "g",
        vec![
            (SymbolId(1), string_token("x", "x")),
            (SymbolId(2), string_token("y", "y")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "prog".to_string())],
    );
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let output1 = lang_gen_output(&grammar, &pt);
    let output2 = lang_gen_output(&grammar, &pt);
    assert_eq!(output1, output2, "output must be deterministic");
}

#[test]
fn sym_roundtrip_externals_in_serialized_roundtrip() {
    let mut grammar = make_grammar(
        "g",
        vec![(SymbolId(1), string_token("tok", "t"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "start".to_string())],
    );
    grammar.externals.push(ExternalToken {
        name: "ext_scan".to_string(),
        symbol_id: SymbolId(80),
    });
    let pt = make_lang_gen_table(&grammar, 1, 4);
    let names = serialized_names(&grammar, &pt);
    assert!(names.contains(&"end".to_string()));
    assert!(names.contains(&"tok".to_string()));
    assert!(names.contains(&"start".to_string()));
    assert!(names.contains(&"ext_scan".to_string()));
}
