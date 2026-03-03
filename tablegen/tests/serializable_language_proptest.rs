#![allow(clippy::needless_range_loop)]

//! Property-based tests for `SerializableLanguage` in `adze_tablegen::serializer`.
//!
//! Tests cover:
//! - Creation from grammar + parse table
//! - Serde roundtrip (serialize → deserialize → compare)
//! - JSON serialization structure
//! - Determinism (same input → same output)
//! - Various grammar shapes (empty, tokens-only, rules, externals, fields)
//! - Field completeness
//! - Equality semantics

use std::collections::BTreeMap;

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::serializer::{SerializableLanguage, serialize_language};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: adze_ir::StateId = adze_ir::StateId(u16::MAX);

/// Construct a minimal ParseTable for unit-level tests.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);
    let token_count = eof_idx - externals;

    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (symbol_id, index) in &symbol_to_index {
        index_to_symbol[*index] = *symbol_id;
    }

    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        states
    ];

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count,
        external_token_count: externals,
        eof_symbol,
        start_symbol,
        initial_state: adze_ir::StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: Grammar::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build a grammar with the given token and rule names.
fn grammar_with(
    name: &str,
    token_names: &[String],
    rule_names: &[String],
) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());
    for (i, tname) in token_names.iter().enumerate() {
        grammar.tokens.insert(
            SymbolId((i + 1) as u16),
            Token {
                name: tname.clone(),
                pattern: TokenPattern::String(tname.clone()),
                fragile: false,
            },
        );
    }
    let base = token_names.len() + 1;
    for (i, rname) in rule_names.iter().enumerate() {
        let sid = SymbolId((base + i) as u16);
        grammar.rules.entry(sid).or_default().push(Rule {
            lhs: sid,
            rhs: vec![],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
        grammar.rule_names.insert(sid, rname.clone());
    }
    grammar
}

/// Serialize a grammar to a `SerializableLanguage` via JSON roundtrip.
fn roundtrip(grammar: &Grammar, pt: &ParseTable) -> SerializableLanguage {
    let json = serialize_language(grammar, pt, None).expect("serialization must succeed");
    serde_json::from_str(&json).expect("deserialization must succeed")
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}".prop_filter("non-empty", |s| !s.is_empty())
}

fn rule_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,8}".prop_filter("non-empty", |s| !s.is_empty())
}

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,6}".prop_filter("non-empty", |s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// =========================================================================
// 1. SerializableLanguage creation
// =========================================================================

#[test]
fn creation_empty_grammar() {
    let grammar = Grammar::new("empty".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let lang = roundtrip(&grammar, &pt);
    assert_eq!(lang.version, 15);
    assert_eq!(lang.symbol_names[0], "end");
}

#[test]
fn creation_single_token() {
    let grammar = grammar_with("one_tok", &["plus".into()], &[]);
    let pt = make_empty_table(1, 1, 0, 0);
    let lang = roundtrip(&grammar, &pt);
    assert_eq!(lang.token_count, 1);
    assert!(lang.symbol_names.contains(&"plus".to_string()));
}

#[test]
fn creation_multiple_tokens_and_rules() {
    let grammar = grammar_with(
        "multi",
        &["a".into(), "b".into()],
        &["expr".into(), "stmt".into()],
    );
    let pt = make_empty_table(2, 2, 2, 0);
    let lang = roundtrip(&grammar, &pt);
    assert_eq!(lang.token_count, 2);
    assert!(lang.symbol_names.contains(&"expr".to_string()));
    assert!(lang.symbol_names.contains(&"stmt".to_string()));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // =========================================================================
    // 2. Serde roundtrip
    // =========================================================================

    #[test]
    fn serde_roundtrip_preserves_equality(
        tokens in prop::collection::vec(token_name_strategy(), 0..5),
        rules in prop::collection::vec(rule_name_strategy(), 0..4),
    ) {
        let grammar = grammar_with("rt", &tokens, &rules);
        let pt = make_empty_table(1, tokens.len(), rules.len(), 0);
        let json = serialize_language(&grammar, &pt, None).unwrap();
        let first: SerializableLanguage = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string_pretty(&first).unwrap();
        let second: SerializableLanguage = serde_json::from_str(&json2).unwrap();
        prop_assert_eq!(first, second);
    }

    #[test]
    fn serde_roundtrip_version_preserved(
        tokens in prop::collection::vec(token_name_strategy(), 1..4),
    ) {
        let grammar = grammar_with("ver", &tokens, &[]);
        let pt = make_empty_table(1, tokens.len(), 0, 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.version, 15u32);
    }

    // =========================================================================
    // 3. JSON serialization
    // =========================================================================

    #[test]
    fn json_is_valid_and_parseable(
        tokens in prop::collection::vec(token_name_strategy(), 0..6),
    ) {
        let grammar = grammar_with("json_valid", &tokens, &[]);
        let pt = make_empty_table(1, tokens.len(), 0, 0);
        let json = serialize_language(&grammar, &pt, None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(parsed.is_object());
    }

    #[test]
    fn json_contains_required_keys(
        tokens in prop::collection::vec(token_name_strategy(), 1..3),
    ) {
        let grammar = grammar_with("keys", &tokens, &[]);
        let pt = make_empty_table(1, tokens.len(), 0, 0);
        let json = serialize_language(&grammar, &pt, None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        for key in &[
            "version", "symbol_count", "alias_count", "token_count",
            "external_token_count", "state_count", "large_state_count",
            "production_id_count", "field_count", "symbol_names",
            "field_names", "symbol_metadata", "parse_table",
            "small_parse_table_map", "lex_modes",
        ] {
            prop_assert!(obj.contains_key(*key), "missing key: {}", key);
        }
    }

    #[test]
    fn json_symbol_names_is_array(
        tokens in prop::collection::vec(token_name_strategy(), 0..4),
    ) {
        let grammar = grammar_with("arr", &tokens, &[]);
        let pt = make_empty_table(1, tokens.len(), 0, 0);
        let json = serialize_language(&grammar, &pt, None).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(v["symbol_names"].is_array());
    }

    // =========================================================================
    // 4. Determinism
    // =========================================================================

    #[test]
    fn determinism_same_grammar_same_json(
        tokens in prop::collection::vec(token_name_strategy(), 1..5),
        rules in prop::collection::vec(rule_name_strategy(), 0..3),
    ) {
        let grammar = grammar_with("det", &tokens, &rules);
        let pt = make_empty_table(1, tokens.len(), rules.len(), 0);
        let json1 = serialize_language(&grammar, &pt, None).unwrap();
        let json2 = serialize_language(&grammar, &pt, None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    #[test]
    fn determinism_cloned_grammar_same_output(
        tokens in prop::collection::vec(token_name_strategy(), 1..4),
    ) {
        let g1 = grammar_with("clone", &tokens, &[]);
        let g2 = g1.clone();
        let pt = make_empty_table(1, tokens.len(), 0, 0);
        let json1 = serialize_language(&g1, &pt, None).unwrap();
        let json2 = serialize_language(&g2, &pt, None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    #[test]
    fn determinism_struct_equality_across_runs(
        tokens in prop::collection::vec(token_name_strategy(), 0..5),
        rules in prop::collection::vec(rule_name_strategy(), 0..3),
    ) {
        let grammar = grammar_with("eq", &tokens, &rules);
        let pt = make_empty_table(1, tokens.len(), rules.len(), 0);
        let l1 = roundtrip(&grammar, &pt);
        let l2 = roundtrip(&grammar, &pt);
        prop_assert_eq!(l1, l2);
    }

    // =========================================================================
    // 5. Various grammars
    // =========================================================================

    #[test]
    fn grammar_with_externals(
        base_count in 1usize..4,
        ext_names in prop::collection::vec(token_name_strategy(), 1..3),
    ) {
        let mut grammar = grammar_with(
            "ext",
            &(0..base_count).map(|i| format!("tok{i}")).collect::<Vec<_>>(),
            &[],
        );
        for (i, name) in ext_names.iter().enumerate() {
            grammar.externals.push(ExternalToken {
                name: name.clone(),
                symbol_id: SymbolId(200 + i as u16),
            });
        }
        let pt = make_empty_table(1, base_count, 0, ext_names.len());
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.external_token_count, ext_names.len() as u32);
        // External token names appear in symbol_names
        for name in &ext_names {
            prop_assert!(
                lang.symbol_names.contains(name),
                "missing external: {}",
                name
            );
        }
    }

    #[test]
    fn grammar_with_fields(
        field_names in prop::collection::vec(field_name_strategy(), 1..5),
    ) {
        let mut grammar = grammar_with("fields", &["tok".into()], &[]);
        for (i, name) in field_names.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), name.clone());
        }
        let pt = make_empty_table(1, 1, 0, 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.field_count, field_names.len() as u32);
        prop_assert_eq!(lang.field_names.len(), field_names.len());
    }

    #[test]
    fn grammar_with_hidden_tokens(
        visible in prop::collection::vec(token_name_strategy(), 1..3),
        hidden in prop::collection::vec(
            "_[a-z][a-z0-9]{0,6}".prop_filter("len>1", |s| s.len() > 1),
            1..3
        ),
    ) {
        let all_tokens: Vec<String> = visible.iter().chain(hidden.iter()).cloned().collect();
        let grammar = grammar_with("hidden", &all_tokens, &[]);
        let pt = make_empty_table(1, all_tokens.len(), 0, 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.token_count as usize, all_tokens.len());
    }

    #[test]
    fn grammar_with_regex_tokens(
        count in 1usize..5,
    ) {
        let mut grammar = Grammar::new("regex".to_string());
        for i in 0..count {
            grammar.tokens.insert(
                SymbolId((i + 1) as u16),
                Token {
                    name: format!("re{i}"),
                    pattern: TokenPattern::Regex(format!("[a-z]{{{i}}}")),
                    fragile: false,
                },
            );
        }
        let pt = make_empty_table(1, count, 0, 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.token_count, count as u32);
    }

    #[test]
    fn grammar_with_multiple_states(
        states in 1usize..8,
        tokens in 1usize..4,
    ) {
        let tok_names: Vec<String> = (0..tokens).map(|i| format!("t{i}")).collect();
        let grammar = grammar_with("states", &tok_names, &[]);
        let pt = make_empty_table(states, tokens, 0, 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.state_count, states as u32);
        prop_assert_eq!(lang.lex_modes.len(), states);
    }

    #[test]
    fn grammar_with_supertypes(
        rule_names in prop::collection::vec(rule_name_strategy(), 2..5),
    ) {
        let mut grammar = grammar_with("super", &["tok".into()], &rule_names);
        // Mark first rule as a supertype
        if let Some((&sid, _)) = grammar.rules.iter().next() {
            grammar.supertypes.push(sid);
        }
        let pt = make_empty_table(1, 1, rule_names.len(), 0);
        let lang = roundtrip(&grammar, &pt);
        // Supertype flag is encoded in symbol_metadata
        prop_assert!(!lang.symbol_metadata.is_empty());
    }

    // =========================================================================
    // 6. Field completeness
    // =========================================================================

    #[test]
    fn field_names_sorted_lexicographically(
        field_names in prop::collection::vec(field_name_strategy(), 2..6),
    ) {
        let mut grammar = grammar_with("fsort", &["tok".into()], &[]);
        for (i, name) in field_names.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), name.clone());
        }
        let pt = make_empty_table(1, 1, 0, 0);
        let lang = roundtrip(&grammar, &pt);
        let mut sorted = lang.field_names.clone();
        sorted.sort();
        prop_assert_eq!(lang.field_names, sorted);
    }

    #[test]
    fn symbol_names_start_with_end(
        tokens in prop::collection::vec(token_name_strategy(), 0..5),
    ) {
        let grammar = grammar_with("end_first", &tokens, &[]);
        let pt = make_empty_table(1, tokens.len(), 0, 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(&lang.symbol_names[0], "end");
    }

    #[test]
    fn symbol_count_matches_names_len(
        tokens in prop::collection::vec(token_name_strategy(), 0..5),
        rules in prop::collection::vec(rule_name_strategy(), 0..4),
    ) {
        let grammar = grammar_with("cnt", &tokens, &rules);
        let pt = make_empty_table(1, tokens.len(), rules.len(), 0);
        let lang = roundtrip(&grammar, &pt);
        // symbol_count = 1 (EOF) + tokens + rules + externals
        let expected = 1 + tokens.len() + rules.len();
        prop_assert_eq!(lang.symbol_count as usize, expected);
        prop_assert_eq!(lang.symbol_names.len(), expected);
    }

    #[test]
    fn symbol_metadata_len_matches_symbol_count(
        tokens in prop::collection::vec(token_name_strategy(), 0..4),
        rules in prop::collection::vec(rule_name_strategy(), 0..3),
    ) {
        let grammar = grammar_with("meta", &tokens, &rules);
        let pt = make_empty_table(1, tokens.len(), rules.len(), 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.symbol_metadata.len(), lang.symbol_count as usize);
    }

    #[test]
    fn lex_modes_len_matches_state_count(
        states in 1usize..6,
    ) {
        let grammar = grammar_with("lex", &["tok".into()], &[]);
        let pt = make_empty_table(states, 1, 0, 0);
        let lang = roundtrip(&grammar, &pt);
        prop_assert_eq!(lang.lex_modes.len(), lang.state_count as usize);
    }

    #[test]
    fn production_id_count_matches_rules(
        rules in prop::collection::vec(rule_name_strategy(), 1..5),
    ) {
        let grammar = grammar_with("prod", &["tok".into()], &rules);
        let pt = make_empty_table(1, 1, rules.len(), 0);
        let lang = roundtrip(&grammar, &pt);
        // Each rule name gets one Rule with one production
        prop_assert_eq!(lang.production_id_count as usize, rules.len());
    }

    // =========================================================================
    // 7. Equality
    // =========================================================================

    #[test]
    fn equality_identical_grammars(
        tokens in prop::collection::vec(token_name_strategy(), 0..4),
        rules in prop::collection::vec(rule_name_strategy(), 0..3),
    ) {
        let g = grammar_with("eq_id", &tokens, &rules);
        let pt = make_empty_table(1, tokens.len(), rules.len(), 0);
        let l1 = roundtrip(&g, &pt);
        let l2 = roundtrip(&g, &pt);
        prop_assert_eq!(l1, l2);
    }

    #[test]
    fn equality_different_grammar_names_same_structure(
        tokens in prop::collection::vec(token_name_strategy(), 1..3),
    ) {
        // Grammar name does not appear in SerializableLanguage, so different
        // names with the same structure should produce equal outputs.
        let g1 = grammar_with("name_a", &tokens, &[]);
        let g2 = grammar_with("name_b", &tokens, &[]);
        let pt = make_empty_table(1, tokens.len(), 0, 0);
        let l1 = roundtrip(&g1, &pt);
        let l2 = roundtrip(&g2, &pt);
        prop_assert_eq!(l1, l2);
    }
}

// =========================================================================
// Additional non-proptest tests for edge cases
// =========================================================================

#[test]
fn empty_grammar_has_zero_alias_count() {
    let grammar = Grammar::new("e".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let lang = roundtrip(&grammar, &pt);
    assert_eq!(lang.alias_count, 0);
}

#[test]
fn empty_grammar_has_zero_large_state_count() {
    let grammar = Grammar::new("e".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let lang = roundtrip(&grammar, &pt);
    assert_eq!(lang.large_state_count, 0);
}

#[test]
fn empty_grammar_parse_table_and_map_empty() {
    let grammar = Grammar::new("e".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let lang = roundtrip(&grammar, &pt);
    // Without compressed tables, these are empty
    assert!(lang.parse_table.is_empty());
    assert!(lang.small_parse_table_map.is_empty());
}

#[test]
fn token_names_appear_in_symbol_names() {
    let names: Vec<String> = vec!["alpha".into(), "beta".into(), "gamma".into()];
    let grammar = grammar_with("tnames", &names, &[]);
    let pt = make_empty_table(1, 3, 0, 0);
    let lang = roundtrip(&grammar, &pt);
    for name in &names {
        assert!(
            lang.symbol_names.contains(name),
            "symbol_names missing token: {name}"
        );
    }
}

#[test]
fn rule_names_appear_in_symbol_names() {
    let rules: Vec<String> = vec!["expression".into(), "statement".into()];
    let grammar = grammar_with("rnames", &["tok".into()], &rules);
    let pt = make_empty_table(1, 1, 2, 0);
    let lang = roundtrip(&grammar, &pt);
    for name in &rules {
        assert!(
            lang.symbol_names.contains(name),
            "symbol_names missing rule: {name}"
        );
    }
}

#[test]
fn json_roundtrip_preserves_all_numeric_fields() {
    let mut grammar = grammar_with("nums", &["tok".into()], &["rule".into()]);
    grammar.externals.push(ExternalToken {
        name: "ext".into(),
        symbol_id: SymbolId(200),
    });
    grammar.fields.insert(FieldId(0), "field_a".into());
    let pt = make_empty_table(3, 1, 1, 1);
    let lang = roundtrip(&grammar, &pt);

    assert_eq!(lang.version, 15);
    assert_eq!(lang.state_count, 3);
    assert_eq!(lang.token_count, 1);
    assert_eq!(lang.external_token_count, 1);
    assert_eq!(lang.field_count, 1);
}

#[test]
fn deserialized_equals_reserialized() {
    let grammar = grammar_with("reser", &["a".into(), "b".into()], &["r".into()]);
    let pt = make_empty_table(2, 2, 1, 0);
    let json1 = serialize_language(&grammar, &pt, None).unwrap();
    let lang: SerializableLanguage = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&lang).unwrap();
    let lang2: SerializableLanguage = serde_json::from_str(&json2).unwrap();
    assert_eq!(lang, lang2);
}
