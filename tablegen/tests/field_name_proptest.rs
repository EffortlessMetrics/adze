#![allow(clippy::needless_range_loop)]
//! Property-based and unit tests for field name generation in adze-tablegen.
//!
//! Covers: field names from grammar fields, ordering, count, presence in
//! generated Language, underscores, determinism, empty fields, and complex grammars.

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::AbiLanguageBuilder;
use adze_tablegen::serializer::{SerializableLanguage, serialize_language};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

fn string_token(name: &str, literal: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(literal.to_string()),
        fragile: false,
    }
}

fn regex_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::Regex(pattern.to_string()),
        fragile: false,
    }
}

/// Build a grammar + parse table pair suitable for field-name tests.
fn build_grammar_and_table(
    name: &str,
    tokens: Vec<(SymbolId, Token)>,
    rules: Vec<Rule>,
    fields: Vec<(FieldId, String)>,
    num_states: usize,
) -> (Grammar, ParseTable) {
    let num_terms = tokens.len().max(1);
    let num_nonterms = {
        let mut syms: std::collections::HashSet<SymbolId> = std::collections::HashSet::new();
        for r in &rules {
            syms.insert(r.lhs);
        }
        syms.len().max(1)
    };
    let num_states = num_states.max(1);

    let eof_idx = 1 + num_terms;
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

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let mut grammar = Grammar::new(name.to_string());
    for (id, tok) in tokens {
        grammar.tokens.insert(id, tok);
    }
    for rule in rules {
        grammar.add_rule(rule);
    }
    for (fid, fname) in fields {
        grammar.fields.insert(fid, fname);
    }

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
        external_token_count: 0,
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

/// Build grammar+table with only fields (one dummy token/rule).
fn fields_only(fields: Vec<(FieldId, String)>) -> (Grammar, ParseTable) {
    let tok = (SymbolId(1), string_token("tok", "x"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    build_grammar_and_table("test", vec![tok], vec![rule], fields, 1)
}

/// Deserialize the output of `serialize_language` for assertions.
fn serialized(grammar: &Grammar, table: &ParseTable) -> SerializableLanguage {
    let json = serialize_language(grammar, table, None).expect("serialization must succeed");
    serde_json::from_str(&json).expect("deserialization must succeed")
}

// ===========================================================================
// 1. Field names from grammar fields
// ===========================================================================

#[test]
fn single_field_name_from_grammar() {
    let (g, t) = fields_only(vec![(FieldId(0), "value".into())]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["value"]);
}

#[test]
fn multiple_field_names_from_grammar() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "left".into()),
        (FieldId(1), "operator".into()),
        (FieldId(2), "right".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["left", "operator", "right"]);
}

#[test]
fn field_names_reflect_grammar_values_not_ids() {
    let (g, t) = fields_only(vec![
        (FieldId(100), "body".into()),
        (FieldId(200), "name".into()),
    ]);
    let lang = serialized(&g, &t);
    assert!(lang.field_names.contains(&"body".to_string()));
    assert!(lang.field_names.contains(&"name".to_string()));
}

// ===========================================================================
// 2. Field name ordering
// ===========================================================================

#[test]
fn field_names_are_lexicographically_sorted() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "zebra".into()),
        (FieldId(1), "apple".into()),
        (FieldId(2), "mango".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["apple", "mango", "zebra"]);
}

#[test]
fn field_names_sorted_regardless_of_field_id_order() {
    let (g, t) = fields_only(vec![
        (FieldId(5), "gamma".into()),
        (FieldId(2), "alpha".into()),
        (FieldId(9), "beta".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn field_names_sorted_with_prefixes() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "body_end".into()),
        (FieldId(1), "body".into()),
        (FieldId(2), "body_start".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["body", "body_end", "body_start"]);
}

#[test]
fn field_names_sorted_case_sensitive() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "Name".into()),
        (FieldId(1), "body".into()),
        (FieldId(2), "Alpha".into()),
    ]);
    let lang = serialized(&g, &t);
    // Uppercase comes before lowercase in ASCII/lexicographic order
    assert_eq!(lang.field_names, vec!["Alpha", "Name", "body"]);
}

// ===========================================================================
// 3. Field name count matches grammar
// ===========================================================================

#[test]
fn field_count_zero_with_no_fields() {
    let (g, t) = fields_only(vec![]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 0);
    assert!(lang.field_names.is_empty());
}

#[test]
fn field_count_matches_single_field() {
    let (g, t) = fields_only(vec![(FieldId(0), "x".into())]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 1);
    assert_eq!(lang.field_names.len(), 1);
}

#[test]
fn field_count_matches_many_fields() {
    let fields: Vec<_> = (0..10)
        .map(|i| (FieldId(i), format!("field_{}", i)))
        .collect();
    let (g, t) = fields_only(fields);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 10);
    assert_eq!(lang.field_names.len(), 10);
}

#[test]
fn field_count_equals_field_names_len() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "a".into()),
        (FieldId(1), "b".into()),
        (FieldId(2), "c".into()),
        (FieldId(3), "d".into()),
        (FieldId(4), "e".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count as usize, lang.field_names.len());
}

// ===========================================================================
// 4. Field names in generated Language (ABI codegen)
// ===========================================================================

#[test]
fn abi_codegen_contains_field_name_entries() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "left".into()),
        (FieldId(1), "right".into()),
    ]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    // Field names are encoded as byte arrays (e.g., FIELD_NAME_0, FIELD_NAME_1)
    assert!(
        code.contains("FIELD_NAME_0"),
        "generated code must contain FIELD_NAME_0"
    );
    assert!(
        code.contains("FIELD_NAME_1"),
        "generated code must contain FIELD_NAME_1"
    );
}

#[test]
fn abi_codegen_field_count_literal() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "alpha".into()),
        (FieldId(1), "beta".into()),
        (FieldId(2), "gamma".into()),
    ]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    assert!(
        code.contains("field_count"),
        "generated code must contain field_count"
    );
}

#[test]
fn abi_codegen_field_names_array_present() {
    let (g, t) = fields_only(vec![(FieldId(0), "name".into())]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    assert!(
        code.contains("FIELD_NAME"),
        "generated code must contain FIELD_NAME array"
    );
}

// ===========================================================================
// 5. Field names with underscores
// ===========================================================================

#[test]
fn field_name_with_single_underscore() {
    let (g, t) = fields_only(vec![(FieldId(0), "my_field".into())]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["my_field"]);
}

#[test]
fn field_name_with_multiple_underscores() {
    let (g, t) = fields_only(vec![(FieldId(0), "a_b_c_d".into())]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["a_b_c_d"]);
}

#[test]
fn field_name_with_leading_underscore() {
    let (g, t) = fields_only(vec![(FieldId(0), "_hidden".into())]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["_hidden"]);
}

#[test]
fn underscore_fields_sorted_correctly() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "z_field".into()),
        (FieldId(1), "_private".into()),
        (FieldId(2), "a_field".into()),
    ]);
    let lang = serialized(&g, &t);
    // Underscore comes before letters in ASCII
    assert_eq!(lang.field_names, vec!["_private", "a_field", "z_field"]);
}

// ===========================================================================
// 6. Field name determinism
// ===========================================================================

#[test]
fn field_name_generation_is_deterministic() {
    let fields = vec![
        (FieldId(0), "delta".into()),
        (FieldId(1), "alpha".into()),
        (FieldId(2), "charlie".into()),
        (FieldId(3), "bravo".into()),
    ];
    let (g1, t1) = fields_only(fields.clone());
    let (g2, t2) = fields_only(fields);

    let lang1 = serialized(&g1, &t1);
    let lang2 = serialized(&g2, &t2);

    assert_eq!(lang1.field_names, lang2.field_names);
    assert_eq!(lang1.field_count, lang2.field_count);
}

#[test]
fn abi_codegen_is_deterministic_for_fields() {
    let fields = vec![
        (FieldId(0), "x".into()),
        (FieldId(1), "y".into()),
        (FieldId(2), "z".into()),
    ];
    let (g1, t1) = fields_only(fields.clone());
    let (g2, t2) = fields_only(fields);

    let code1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let code2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();

    assert_eq!(code1, code2);
}

#[test]
fn serialization_roundtrip_preserves_field_names() {
    let (g, t) = fields_only(vec![(FieldId(0), "foo".into()), (FieldId(1), "bar".into())]);
    let json = serialize_language(&g, &t, None).expect("serialize");
    let lang1: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string_pretty(&lang1).expect("re-serialize");
    let lang2: SerializableLanguage = serde_json::from_str(&json2).expect("deserialize again");
    assert_eq!(lang1.field_names, lang2.field_names);
}

// ===========================================================================
// 7. Empty field names (no fields)
// ===========================================================================

#[test]
fn empty_grammar_has_no_field_names() {
    let (g, t) = fields_only(vec![]);
    let lang = serialized(&g, &t);
    assert!(lang.field_names.is_empty());
    assert_eq!(lang.field_count, 0);
}

#[test]
fn abi_codegen_handles_no_fields() {
    let (g, t) = fields_only(vec![]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    // Should still compile without field arrays
    assert!(
        code.contains("field_count : 0"),
        "field_count should be 0 in generated code: {}",
        &code[..code.len().min(500)]
    );
}

// ===========================================================================
// 8. Field names with complex grammars
// ===========================================================================

#[test]
fn field_names_with_multiple_tokens_and_rules() {
    let tokens = vec![
        (SymbolId(1), string_token("plus", "+")),
        (SymbolId(2), regex_token("number", "[0-9]+")),
    ];
    let rule = Rule {
        lhs: SymbolId(4),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(0),
    };
    let fields = vec![(FieldId(0), "left".into()), (FieldId(1), "right".into())];
    let (g, t) = build_grammar_and_table("arith", tokens, vec![rule], fields, 2);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["left", "right"]);
    assert_eq!(lang.field_count, 2);
}

#[test]
fn field_names_with_external_tokens() {
    let tokens = vec![(SymbolId(1), string_token("tok", "x"))];
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let fields = vec![
        (FieldId(0), "content".into()),
        (FieldId(1), "delimiter".into()),
    ];
    let (mut g, t) = build_grammar_and_table("ext", tokens, vec![rule], fields, 1);
    g.externals.push(ExternalToken {
        name: "heredoc".to_string(),
        symbol_id: SymbolId(10),
    });
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["content", "delimiter"]);
}

#[test]
fn field_names_with_multiple_rules_sharing_fields() {
    let tokens = vec![
        (SymbolId(1), string_token("a", "a")),
        (SymbolId(2), string_token("b", "b")),
    ];
    let rule1 = Rule {
        lhs: SymbolId(4),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let rule2 = Rule {
        lhs: SymbolId(4),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(1), 0)],
        production_id: ProductionId(1),
    };
    let fields = vec![(FieldId(0), "first".into()), (FieldId(1), "second".into())];
    let (g, t) = build_grammar_and_table("multi", tokens, vec![rule1, rule2], fields, 2);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["first", "second"]);
}

// ===========================================================================
// Property-based tests
// ===========================================================================

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_map(|s| s)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_field_count_matches_grammar(
        n in 0..8usize,
    ) {
        let fields: Vec<_> = (0..n)
            .map(|i| (FieldId(i as u16), format!("f{}", i)))
            .collect();
        let (g, t) = fields_only(fields);
        let lang = serialized(&g, &t);
        prop_assert_eq!(lang.field_count as usize, n);
        prop_assert_eq!(lang.field_names.len(), n);
    }

    #[test]
    fn prop_field_names_always_sorted(
        names in proptest::collection::vec(field_name_strategy(), 1..8),
    ) {
        let fields: Vec<_> = names.iter().enumerate()
            .map(|(i, name)| (FieldId(i as u16), name.clone()))
            .collect();
        let (g, t) = fields_only(fields);
        let lang = serialized(&g, &t);

        // Verify sorted
        for i in 1..lang.field_names.len() {
            prop_assert!(
                lang.field_names[i - 1] <= lang.field_names[i],
                "field_names not sorted: {:?} > {:?}",
                lang.field_names[i - 1],
                lang.field_names[i]
            );
        }
    }

    #[test]
    fn prop_field_names_deterministic(
        names in proptest::collection::vec(field_name_strategy(), 1..6),
    ) {
        let fields: Vec<_> = names.iter().enumerate()
            .map(|(i, name)| (FieldId(i as u16), name.clone()))
            .collect();

        let (g1, t1) = fields_only(fields.clone());
        let (g2, t2) = fields_only(fields);

        let lang1 = serialized(&g1, &t1);
        let lang2 = serialized(&g2, &t2);

        prop_assert_eq!(&lang1.field_names, &lang2.field_names);
    }

    #[test]
    fn prop_all_field_names_present_in_output(
        names in proptest::collection::vec(field_name_strategy(), 1..6),
    ) {
        let fields: Vec<_> = names.iter().enumerate()
            .map(|(i, name)| (FieldId(i as u16), name.clone()))
            .collect();
        let (g, t) = fields_only(fields);
        let lang = serialized(&g, &t);

        for name in &names {
            prop_assert!(
                lang.field_names.contains(name),
                "field name '{}' missing from output {:?}",
                name,
                lang.field_names
            );
        }
    }

    #[test]
    fn prop_underscore_field_names_preserved(
        base in "[a-z]{1,8}",
    ) {
        let name = format!("_{}", base);
        let (g, t) = fields_only(vec![(FieldId(0), name.clone())]);
        let lang = serialized(&g, &t);
        prop_assert_eq!(&lang.field_names[0], &name);
    }
}
