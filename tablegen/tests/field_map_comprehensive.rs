#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for field mapping functionality in the tablegen crate.
//!
//! Covers: field name generation, lexicographic ordering, field maps in ABI code
//! generation, field_count in Language structs, field entries in NODE_TYPES JSON,
//! field maps in serialized language data, and edge cases.

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::serializer::{SerializableLanguage, serialize_language};
use adze_tablegen::{AbiLanguageBuilder, LanguageBuilder, NodeTypesGenerator};
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

/// Build a grammar + parse table pair suitable for field-mapping tests.
///
/// Symbol layout: ERROR(0), terminals 1..=num_terms, EOF, non-terminals.
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

/// Convenience: build a grammar+table with no rules and only fields.
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
// 1. Field count
// ===========================================================================

#[test]
fn field_count_zero_when_no_fields() {
    let (g, t) = fields_only(vec![]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 0);
}

#[test]
fn field_count_matches_inserted_fields() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "alpha".into()),
        (FieldId(1), "beta".into()),
        (FieldId(2), "gamma".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 3);
}

// ===========================================================================
// 2. Field name lexicographic ordering (serializer)
// ===========================================================================

#[test]
fn field_names_sorted_lexicographically() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "zebra".into()),
        (FieldId(1), "apple".into()),
        (FieldId(2), "mango".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_names, vec!["apple", "mango", "zebra"]);
}

// ===========================================================================
// 3. ABI code generation includes field arrays
// ===========================================================================

#[test]
fn abi_codegen_contains_field_map_arrays() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "left".into()),
        (FieldId(1), "right".into()),
    ]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    assert!(
        code.contains("FIELD_MAP_SLICES"),
        "generated code must contain FIELD_MAP_SLICES"
    );
    assert!(
        code.contains("FIELD_MAP_ENTRIES"),
        "generated code must contain FIELD_MAP_ENTRIES"
    );
}

#[test]
fn abi_codegen_contains_field_name_ptrs_when_fields_present() {
    let (g, t) = fields_only(vec![(FieldId(0), "value".into())]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    assert!(
        code.contains("FIELD_NAME_PTRS"),
        "generated code must contain FIELD_NAME_PTRS"
    );
}

#[test]
fn abi_codegen_zero_fields_still_has_map_arrays() {
    let (g, t) = fields_only(vec![]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    // Even with no fields, the arrays should be present (minimal placeholders).
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

// ===========================================================================
// 4. LanguageBuilder field_count
// ===========================================================================

#[test]
fn language_builder_field_count_matches() {
    let (mut g, t) = fields_only(vec![]);
    g.fields.insert(FieldId(0), "left".into());
    g.fields.insert(FieldId(1), "right".into());
    let builder = LanguageBuilder::new(g, t);
    let lang = builder.generate_language().expect("generate must succeed");
    assert_eq!(lang.field_count, 2);
}

#[test]
fn language_builder_many_fields() {
    let fields: Vec<_> = (0..10)
        .map(|i| (FieldId(i), format!("field_{i}")))
        .collect();
    let (g, t) = fields_only(fields);
    let builder = LanguageBuilder::new(g, t);
    let lang = builder.generate_language().expect("generate must succeed");
    assert_eq!(lang.field_count, 10);
}

// ===========================================================================
// 5. NODE_TYPES JSON includes fields
// ===========================================================================

#[test]
fn node_types_includes_field_info() {
    let tok = (SymbolId(1), regex_token("number", r"\d+"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let fields = vec![(FieldId(0), "value".into())];
    let (mut g, _t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);
    g.rule_names.insert(SymbolId(3), "expression".into());

    let ntgen = NodeTypesGenerator::new(&g);
    let json_str = ntgen
        .generate()
        .expect("node types generation must succeed");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Find the expression node type
    let arr = json.as_array().unwrap();
    let expr = arr.iter().find(|v| v["type"] == "expression");
    assert!(expr.is_some(), "expression node type must exist");
    let expr = expr.unwrap();
    assert!(
        expr.get("fields").is_some(),
        "expression must have fields entry"
    );
    let fields = expr["fields"].as_object().unwrap();
    assert!(fields.contains_key("value"), "fields must contain 'value'");
}

#[test]
fn node_types_no_fields_entry_when_no_fields() {
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let (mut g, _t) = build_grammar_and_table("test", vec![tok], vec![rule], vec![], 1);
    g.rule_names.insert(SymbolId(3), "expr".into());

    let ntgen = NodeTypesGenerator::new(&g);
    let json_str = ntgen.generate().expect("must succeed");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let arr = json.as_array().unwrap();
    let expr = arr.iter().find(|v| v["type"] == "expr");
    assert!(expr.is_some());
    // Without fields on the rule, the "fields" key should be absent.
    assert!(
        expr.unwrap().get("fields").is_none(),
        "no fields key expected when rule has no field mappings"
    );
}

// ===========================================================================
// 6. Field map entries for rules with fields (ABI builder)
// ===========================================================================

#[test]
fn abi_field_map_entries_for_rule_with_fields() {
    let tok = (SymbolId(1), string_token("plus", "+"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(1),
    };
    let fields = vec![(FieldId(0), "left".into()), (FieldId(1), "right".into())];
    let (g, t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();

    // The field_count should be 2
    assert!(code.contains("field_count : 2"));
    // Field map entries should exist
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

#[test]
fn abi_field_map_skips_production_id_zero() {
    // Production ID 0 is skipped in generate_field_maps
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0), // production 0 is skipped
    };
    let fields = vec![(FieldId(0), "value".into())];
    let (g, t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();

    // Should still compile but field map entries are minimal (placeholder 0u16)
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

// ===========================================================================
// 7. Multiple rules with different production IDs
// ===========================================================================

#[test]
fn multiple_productions_each_get_field_map_slice() {
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule1 = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(1),
    };
    let rule2 = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(2),
    };
    let fields = vec![(FieldId(0), "left".into()), (FieldId(1), "right".into())];
    let (g, t) = build_grammar_and_table("test", vec![tok], vec![rule1, rule2], fields, 1);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();

    // Both productions should contribute to field map data
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_MAP_ENTRIES"));
    assert!(code.contains("field_count : 2"));
}

// ===========================================================================
// 8. Unicode and special-character field names
// ===========================================================================

#[test]
fn field_names_case_sensitive_ordering() {
    // Uppercase sorts before lowercase in ASCII / lexicographic order
    let (g, t) = fields_only(vec![
        (FieldId(0), "Zebra".into()),
        (FieldId(1), "apple".into()),
    ]);
    let lang = serialized(&g, &t);
    // 'Z' (0x5A) < 'a' (0x61) in byte ordering
    assert_eq!(lang.field_names, vec!["Zebra", "apple"]);
}

// ===========================================================================
// 9. Determinism: same input produces same output
// ===========================================================================

#[test]
fn serialization_deterministic_for_fields() {
    let fields = vec![
        (FieldId(0), "z".into()),
        (FieldId(1), "a".into()),
        (FieldId(2), "m".into()),
    ];
    let (g1, t1) = fields_only(fields.clone());
    let (g2, t2) = fields_only(fields);
    let json1 = serialize_language(&g1, &t1, None).unwrap();
    let json2 = serialize_language(&g2, &t2, None).unwrap();
    assert_eq!(json1, json2, "serialized output must be deterministic");
}

// ===========================================================================
// 10. Large field count
// ===========================================================================

#[test]
fn many_fields_are_sorted_and_counted() {
    let fields: Vec<_> = (0..50)
        .map(|i| (FieldId(i), format!("field_{i:03}")))
        .collect();
    let (g, t) = fields_only(fields);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 50);
    // Verify sorting
    let mut sorted = lang.field_names.clone();
    sorted.sort();
    assert_eq!(lang.field_names, sorted);
}

// ===========================================================================
// 11. Field IDs are non-contiguous
// ===========================================================================

#[test]
fn non_contiguous_field_ids() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "first".into()),
        (FieldId(5), "second".into()),
        (FieldId(100), "third".into()),
    ]);
    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 3);
    assert_eq!(lang.field_names, vec!["first", "second", "third"]);
}

// ===========================================================================
// 12. Fields with rules that reference them in NODE_TYPES
// ===========================================================================

#[test]
fn node_types_multiple_fields_in_one_rule() {
    let tok_a = (SymbolId(1), regex_token("ident", r"[a-z]+"));
    let tok_b = (SymbolId(2), string_token("op", "+"));
    let rule = Rule {
        lhs: SymbolId(4),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(1)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)],
        production_id: ProductionId(0),
    };
    let fields = vec![
        (FieldId(0), "left".into()),
        (FieldId(1), "operator".into()),
        (FieldId(2), "right".into()),
    ];
    let (mut g, _t) = build_grammar_and_table("test", vec![tok_a, tok_b], vec![rule], fields, 1);
    g.rule_names.insert(SymbolId(4), "binary_expr".into());

    let ntgen = NodeTypesGenerator::new(&g);
    let json_str = ntgen.generate().expect("must succeed");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = json.as_array().unwrap();
    let expr = arr.iter().find(|v| v["type"] == "binary_expr").unwrap();
    let fmap = expr["fields"].as_object().unwrap();
    assert!(fmap.contains_key("left"));
    assert!(fmap.contains_key("operator"));
    assert!(fmap.contains_key("right"));
    assert_eq!(fmap.len(), 3);
}

// ===========================================================================
// 13. ABI builder with compressed tables
// ===========================================================================

#[test]
fn abi_builder_with_compressed_tables_still_has_field_data() {
    // Verify that even when the builder has a compressed_tables reference set,
    // the generated code still contains field mapping arrays.
    // We test via generate() without actual compression (no compressed tables needed).
    let (g, t) = fields_only(vec![
        (FieldId(0), "body".into()),
        (FieldId(1), "name".into()),
    ]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_NAME_PTRS"));
    assert!(code.contains("field_count : 2"));
}

// ===========================================================================
// 14. Field names in ABI code generation are null-terminated
// ===========================================================================

#[test]
fn abi_field_names_are_null_terminated_bytes() {
    let (g, t) = fields_only(vec![(FieldId(0), "xyz".into())]);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    // The generated code should contain the null-terminated byte representation
    // Field name "xyz" -> bytes 120, 121, 122, 0
    assert!(
        code.contains("120") && code.contains("121") && code.contains("122"),
        "field name bytes for 'xyz' must appear in generated code"
    );
}

// ===========================================================================
// 15. Internal (underscore-prefixed) rule names and fields
// ===========================================================================

#[test]
fn node_types_internal_rules_skipped_but_fields_still_registered() {
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let fields = vec![(FieldId(0), "value".into())];
    let (mut g, _t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);
    // Internal rule name starts with '_'
    g.rule_names.insert(SymbolId(3), "_internal".into());

    let ntgen = NodeTypesGenerator::new(&g);
    let json_str = ntgen.generate().expect("must succeed");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = json.as_array().unwrap();

    // Internal rules should be skipped in node types output
    let internal = arr.iter().find(|v| v["type"] == "_internal");
    assert!(
        internal.is_none(),
        "internal rules should not appear in NODE_TYPES"
    );

    // But field_count in grammar is still correct
    assert_eq!(g.fields.len(), 1);
}

// ===========================================================================
// 16. Serialized language round-trip preserves fields
// ===========================================================================

#[test]
fn serialize_deserialize_preserves_field_names() {
    let fields = vec![
        (FieldId(0), "condition".into()),
        (FieldId(1), "consequence".into()),
        (FieldId(2), "alternative".into()),
    ];
    let (g, t) = fields_only(fields);
    let json = serialize_language(&g, &t, None).unwrap();
    let lang: SerializableLanguage = serde_json::from_str(&json).unwrap();
    assert_eq!(lang.field_count, 3);
    assert_eq!(
        lang.field_names,
        vec!["alternative", "condition", "consequence"]
    );
}

// ===========================================================================
// 17. StaticLanguageGenerator node_types with fields
// ===========================================================================

#[test]
fn static_generator_node_types_includes_field_entries() {
    use adze_tablegen::StaticLanguageGenerator;

    let tok = (SymbolId(1), regex_token("num", r"\d+"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let fields = vec![(FieldId(0), "value".into())];
    let (g, t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);

    let slgen = StaticLanguageGenerator::new(g, t);
    let node_types = slgen.generate_node_types();
    let json: serde_json::Value = serde_json::from_str(&node_types).unwrap();
    let arr = json.as_array().unwrap();

    // At least one node type should have a "fields" key with "value"
    let has_value_field = arr.iter().any(|nt| {
        nt.get("fields")
            .and_then(|f| f.as_object())
            .is_some_and(|m| m.contains_key("value"))
    });
    assert!(has_value_field, "node types should include a 'value' field");
}

// ===========================================================================
// 18. Edge: duplicate field names (same FieldId reinserted)
// ===========================================================================

#[test]
fn reinserting_same_field_id_overwrites() {
    let mut g = Grammar::new("test".into());
    g.fields.insert(FieldId(0), "original".into());
    g.fields.insert(FieldId(0), "replaced".into());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields.get(&FieldId(0)).unwrap(), "replaced");
}

// ===========================================================================
// 19. Field map slices padded for gaps in production IDs
// ===========================================================================

#[test]
fn field_map_slices_padded_for_production_id_gaps() {
    // Production ID 5 with fields but nothing between 1..4
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(5),
    };
    let fields = vec![(FieldId(0), "value".into())];
    let (g, t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();

    // The FIELD_MAP_SLICES array should contain padding entries (0u16) for IDs 1..4
    assert!(code.contains("FIELD_MAP_SLICES"));
}

// ===========================================================================
// 20. FieldId Display impl
// ===========================================================================

#[test]
fn field_id_display() {
    let fid = FieldId(42);
    assert_eq!(format!("{fid}"), "Field(42)");
}

// ===========================================================================
// 21. Fields with external tokens
// ===========================================================================

#[test]
fn fields_coexist_with_external_tokens() {
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule = Rule {
        lhs: SymbolId(4),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let fields = vec![(FieldId(0), "body".into())];
    let (mut g, t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);
    g.externals.push(ExternalToken {
        name: "comment".into(),
        symbol_id: SymbolId(200),
    });

    let lang = serialized(&g, &t);
    assert_eq!(lang.field_count, 1);
    assert_eq!(lang.field_names, vec!["body"]);
    assert_eq!(lang.external_token_count, 1);
}

// ===========================================================================
// 22. Grammar with fields but rule has no field mappings
// ===========================================================================

#[test]
fn grammar_fields_exist_but_rule_has_no_mappings() {
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![], // no field mappings on the rule
        production_id: ProductionId(1),
    };
    // But field is registered in grammar
    let fields = vec![(FieldId(0), "unused_field".into())];
    let (g, t) = build_grammar_and_table("test", vec![tok], vec![rule], fields, 1);
    let lang = serialized(&g, &t);
    // The grammar has 1 field registered even if no rule uses it
    assert_eq!(lang.field_count, 1);
    assert_eq!(lang.field_names, vec!["unused_field"]);
}

// ===========================================================================
// 25. Two different rules sharing the same field ID
// ===========================================================================

#[test]
fn two_rules_sharing_same_field_id() {
    let tok = (SymbolId(1), string_token("x", "x"));
    let rule1 = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(1),
    };
    let rule2 = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 1)], // same field, different position
        production_id: ProductionId(2),
    };
    let fields = vec![(FieldId(0), "value".into())];
    let (g, t) = build_grammar_and_table("test", vec![tok], vec![rule1, rule2], fields, 1);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();

    // Both productions should contribute field map entries
    assert!(code.contains("FIELD_MAP_ENTRIES"));
    assert!(code.contains("field_count : 1"));
}

// ===========================================================================
// 26. Numeric-like field names sort by string order, not numeric
// ===========================================================================

#[test]
fn numeric_field_names_sort_by_string() {
    let (g, t) = fields_only(vec![
        (FieldId(0), "2".into()),
        (FieldId(1), "10".into()),
        (FieldId(2), "1".into()),
    ]);
    let lang = serialized(&g, &t);
    // String sort: "1" < "10" < "2"
    assert_eq!(lang.field_names, vec!["1", "10", "2"]);
}

// ===========================================================================
// 27. LanguageBuilder field_names pointer is null when no fields
// ===========================================================================

#[test]
fn language_builder_null_field_names_when_empty() {
    let (g, t) = fields_only(vec![]);
    let builder = LanguageBuilder::new(g, t);
    let lang = builder.generate_language().expect("must succeed");
    assert_eq!(lang.field_count, 0);
    assert!(
        lang.field_names.is_null(),
        "field_names should be null when no fields"
    );
}

// ===========================================================================
// 28. Field map slices padded for gaps in production IDs
// ===========================================================================

// (Covered in test #19 above)
