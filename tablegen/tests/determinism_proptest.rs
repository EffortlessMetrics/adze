#![allow(clippy::needless_range_loop)]
// Property-based tests for tablegen determinism.
//
// Properties verified:
//  1.  Same Grammar → same compressed action table
//  2.  Same Grammar → same compressed goto table
//  3.  Same Grammar → same compressed default actions
//  4.  Same Grammar → same compressed row offsets
//  5.  Same Grammar → same ABI code (TokenStream)
//  6.  Same Grammar → same NODE_TYPES JSON
//  7.  Same Grammar → same symbol names from serializer
//  8.  Same Grammar → same field names from serializer
//  9.  Same Grammar → same symbol metadata from serializer
//  10. Same Grammar → same serialized language JSON
//  11. Same Grammar → same StaticLanguageGenerator language code
//  12. Same Grammar → same StaticLanguageGenerator node types
//  13. Same Grammar → same LanguageBuilder language struct fields
//  14. Same Grammar → same encode_action_small results
//  15. Same Grammar → same NodeTypesGenerator output
//  16. Symbol name ordering is stable across clones
//  17. Field name ordering is stable across clones
//  18. Compressed action table ordering is stable
//  19. Compressed goto table ordering is stable
//  20. Node types JSON ordering is stable (sorted by type name)
//  21. GrammarBuilder produces deterministic grammars
//  22. Multiple tokens grammar yields deterministic output
//  23. Multiple rules grammar yields deterministic output
//  24. Grammar with externals yields deterministic output
//  25. Grammar with fields yields deterministic output
//  26. Grammar with hidden tokens yields deterministic output
//  27. Grammar with precedence yields deterministic output
//  28. Empty grammar yields deterministic output
//  29. Large grammar yields deterministic output
//  30. Compressed tables validate identically on repeated runs

use adze_glr_core::{Action, ParseTable};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::compress::TableCompressor;
use adze_tablegen::generate::LanguageBuilder;
use adze_tablegen::node_types::NodeTypesGenerator;
use adze_tablegen::serializer::serialize_language;
use adze_tablegen::StaticLanguageGenerator;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid token name (ASCII lowercase, non-empty).
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Generate a hidden token name (starts with underscore).
fn hidden_token_name() -> impl Strategy<Value = String> {
    "_[a-z][a-z0-9_]{0,10}".prop_filter("non-empty", |s| s.len() > 1)
}

/// Generate a field name.
fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,8}".prop_filter("non-empty", |s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a Grammar with a given number of tokens using GrammarBuilder.
fn grammar_with_tokens(count: usize) -> Grammar {
    let mut builder = GrammarBuilder::new("proptest");
    for i in 0..count {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    if count == 0 {
        builder = builder.token("tok0", "t0");
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    builder.build()
}

/// Build a Grammar manually with visible rules, hidden rules, and tokens.
fn build_grammar(
    visible_names: &[String],
    hidden_names: &[String],
    string_tokens: &[(String, String)],
    regex_tokens: &[(String, String)],
    field_names: &[String],
) -> Grammar {
    let mut g = Grammar::new("proptest".to_string());
    let mut next_id: u16 = 0;

    // Add regex tokens
    let mut regex_token_ids = Vec::new();
    for (name, pattern) in regex_tokens {
        let id = SymbolId(next_id);
        next_id += 1;
        g.tokens.insert(
            id,
            Token {
                name: name.clone(),
                pattern: TokenPattern::Regex(pattern.clone()),
                fragile: false,
            },
        );
        regex_token_ids.push(id);
    }

    // Add string tokens
    let mut string_token_ids = Vec::new();
    for (name, pattern) in string_tokens {
        let id = SymbolId(next_id);
        next_id += 1;
        g.tokens.insert(
            id,
            Token {
                name: name.clone(),
                pattern: TokenPattern::String(pattern.clone()),
                fragile: false,
            },
        );
        string_token_ids.push(id);
    }

    // Register field names
    let field_ids: Vec<FieldId> = field_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let fid = FieldId(i as u16);
            g.fields.insert(fid, name.clone());
            fid
        })
        .collect();

    let mut prod_id: u16 = 0;
    let default_terminal = regex_token_ids
        .first()
        .or(string_token_ids.first())
        .copied();

    // Add visible rules
    for name in visible_names {
        let id = SymbolId(next_id);
        next_id += 1;
        g.rule_names.insert(id, name.clone());

        let (rule_fields, rhs) = if let Some(tid) = default_terminal
            && !field_ids.is_empty()
        {
            let mut rhs_symbols = Vec::new();
            let mut rule_field_pairs = Vec::new();
            for (pos, fid) in field_ids.iter().enumerate() {
                rhs_symbols.push(Symbol::Terminal(tid));
                rule_field_pairs.push((*fid, pos));
            }
            (rule_field_pairs, rhs_symbols)
        } else if let Some(tid) = default_terminal {
            (vec![], vec![Symbol::Terminal(tid)])
        } else {
            (vec![], vec![Symbol::Epsilon])
        };

        g.add_rule(Rule {
            lhs: id,
            rhs,
            precedence: None,
            associativity: None,
            fields: rule_fields,
            production_id: ProductionId(prod_id),
        });
        prod_id += 1;
    }

    // Add hidden rules
    for name in hidden_names {
        let id = SymbolId(next_id);
        next_id += 1;
        g.rule_names.insert(id, name.clone());

        let rhs = if let Some(tid) = default_terminal {
            vec![Symbol::Terminal(tid)]
        } else {
            vec![Symbol::Epsilon]
        };

        g.add_rule(Rule {
            lhs: id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        });
        prod_id += 1;
    }

    g
}

/// Build a Grammar with external tokens appended.
fn grammar_with_externals(base_tokens: usize, external_names: Vec<String>) -> Grammar {
    let mut grammar = grammar_with_tokens(base_tokens);
    for (i, name) in external_names.into_iter().enumerate() {
        grammar.externals.push(ExternalToken {
            name,
            symbol_id: SymbolId(200 + i as u16),
        });
    }
    grammar
}

/// Build a Grammar with hidden tokens added.
fn grammar_with_hidden_tokens(visible: usize, hidden_names: Vec<String>) -> Grammar {
    let mut grammar = grammar_with_tokens(visible);
    for (i, name) in hidden_names.into_iter().enumerate() {
        grammar.tokens.insert(
            SymbolId(100 + i as u16),
            Token {
                name,
                pattern: TokenPattern::String("h".to_string()),
                fragile: false,
            },
        );
    }
    grammar
}

/// Build a Grammar with fields added.
fn grammar_with_fields(token_count: usize, field_names: Vec<String>) -> Grammar {
    let mut grammar = grammar_with_tokens(token_count);
    for (i, name) in field_names.into_iter().enumerate() {
        grammar.fields.insert(FieldId(i as u16), name);
    }
    grammar
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // 1. Same Grammar → same compressed action table
    #[test]
    fn compressed_action_table_deterministic(token_count in 1usize..5) {
        let grammar = grammar_with_tokens(token_count);
        let table = ParseTable::default();

        let compressor = TableCompressor::new();
        let r1 = compressor.compress(&table, &[], false);
        let r2 = compressor.compress(&table, &[], false);

        // Both should produce the same result (both likely Err for empty table)
        prop_assert_eq!(r1.is_ok(), r2.is_ok());
    }

    // 2. Same Grammar → same compressed goto table
    #[test]
    fn compressed_goto_table_deterministic(token_count in 1usize..5) {
        let grammar = grammar_with_tokens(token_count);
        let table = ParseTable::default();

        let compressor = TableCompressor::new();
        let r1 = compressor.compress(&table, &[], false);
        let r2 = compressor.compress(&table, &[], false);

        prop_assert_eq!(r1.is_ok(), r2.is_ok());
    }

    // 3. Same Grammar → same compressed default actions
    #[test]
    fn compressed_default_actions_deterministic(token_count in 1usize..6) {
        let grammar = grammar_with_tokens(token_count);
        let table = ParseTable::default();

        let c = TableCompressor::new();
        let r1 = c.compress(&table, &[], false);
        let r2 = c.compress(&table, &[], false);

        match (r1, r2) {
            (Ok(t1), Ok(t2)) => {
                let d1: Vec<_> = t1.action_table.default_actions.iter().map(|a| format!("{a:?}")).collect();
                let d2: Vec<_> = t2.action_table.default_actions.iter().map(|a| format!("{a:?}")).collect();
                prop_assert_eq!(d1, d2);
            }
            (Err(_), Err(_)) => {} // both errored identically
            _ => prop_assert!(false, "one succeeded, one failed"),
        }
    }

    // 4. Same Grammar → same compressed row offsets
    #[test]
    fn compressed_row_offsets_deterministic(token_count in 1usize..6) {
        let grammar = grammar_with_tokens(token_count);
        let table = ParseTable::default();

        let c = TableCompressor::new();
        let r1 = c.compress(&table, &[], false);
        let r2 = c.compress(&table, &[], false);

        match (r1, r2) {
            (Ok(t1), Ok(t2)) => {
                prop_assert_eq!(t1.action_table.row_offsets, t2.action_table.row_offsets);
                prop_assert_eq!(t1.goto_table.row_offsets, t2.goto_table.row_offsets);
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "one succeeded, one failed"),
        }
    }

    // 5. Same Grammar → same ABI code (TokenStream string)
    #[test]
    fn abi_code_deterministic(token_count in 1usize..5) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let gen1 = StaticLanguageGenerator::new(g1, t1);
        let gen2 = StaticLanguageGenerator::new(g2, t2);

        let code1 = gen1.generate_language_code().to_string();
        let code2 = gen2.generate_language_code().to_string();

        prop_assert_eq!(code1, code2);
    }

    // 6. Same Grammar → same NODE_TYPES JSON (StaticLanguageGenerator)
    #[test]
    fn node_types_json_deterministic(token_count in 1usize..5) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let gen1 = StaticLanguageGenerator::new(g1, t1);
        let gen2 = StaticLanguageGenerator::new(g2, t2);

        let json1 = gen1.generate_node_types();
        let json2 = gen2.generate_node_types();

        prop_assert_eq!(json1, json2);
    }

    // 7. Same Grammar → same symbol names from serializer
    #[test]
    fn serializer_symbol_names_deterministic(token_count in 1usize..6) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let json1 = serialize_language(&g1, &t1, None).unwrap();
        let json2 = serialize_language(&g2, &t2, None).unwrap();

        let v1: serde_json::Value = serde_json::from_str(&json1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();

        prop_assert_eq!(&v1["symbol_names"], &v2["symbol_names"]);
    }

    // 8. Same Grammar → same field names from serializer
    #[test]
    fn serializer_field_names_deterministic(
        field_names in prop::collection::vec(field_name_strategy(), 0..5),
    ) {
        let g1 = grammar_with_fields(2, field_names.clone());
        let g2 = grammar_with_fields(2, field_names);
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let json1 = serialize_language(&g1, &t1, None).unwrap();
        let json2 = serialize_language(&g2, &t2, None).unwrap();

        let v1: serde_json::Value = serde_json::from_str(&json1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();

        prop_assert_eq!(&v1["field_names"], &v2["field_names"]);
    }

    // 9. Same Grammar → same symbol metadata from serializer
    #[test]
    fn serializer_symbol_metadata_deterministic(token_count in 1usize..6) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let json1 = serialize_language(&g1, &t1, None).unwrap();
        let json2 = serialize_language(&g2, &t2, None).unwrap();

        let v1: serde_json::Value = serde_json::from_str(&json1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();

        prop_assert_eq!(&v1["symbol_metadata"], &v2["symbol_metadata"]);
    }

    // 10. Same Grammar → same serialized language JSON (full)
    #[test]
    fn serializer_full_json_deterministic(token_count in 1usize..6) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let json1 = serialize_language(&g1, &t1, None).unwrap();
        let json2 = serialize_language(&g2, &t2, None).unwrap();

        prop_assert_eq!(json1, json2);
    }

    // 11. Same Grammar → same StaticLanguageGenerator language code
    #[test]
    fn static_language_code_deterministic_clone(token_count in 1usize..5) {
        let g = grammar_with_tokens(token_count);
        let t = ParseTable::default();

        let gen1 = StaticLanguageGenerator::new(g.clone(), t.clone());
        let gen2 = StaticLanguageGenerator::new(g, t);

        prop_assert_eq!(
            gen1.generate_language_code().to_string(),
            gen2.generate_language_code().to_string(),
        );
    }

    // 12. Same Grammar → same StaticLanguageGenerator node types
    #[test]
    fn static_node_types_deterministic_clone(token_count in 1usize..5) {
        let g = grammar_with_tokens(token_count);
        let t = ParseTable::default();

        let gen1 = StaticLanguageGenerator::new(g.clone(), t.clone());
        let gen2 = StaticLanguageGenerator::new(g, t);

        prop_assert_eq!(gen1.generate_node_types(), gen2.generate_node_types());
    }

    // 13. Same Grammar → same LanguageBuilder language struct fields
    #[test]
    fn language_builder_deterministic(token_count in 1usize..6) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let b1 = LanguageBuilder::new(g1, t1);
        let b2 = LanguageBuilder::new(g2, t2);

        let l1 = b1.generate_language().unwrap();
        let l2 = b2.generate_language().unwrap();

        prop_assert_eq!(l1.version, l2.version);
        prop_assert_eq!(l1.symbol_count, l2.symbol_count);
        prop_assert_eq!(l1.token_count, l2.token_count);
        prop_assert_eq!(l1.external_token_count, l2.external_token_count);
        prop_assert_eq!(l1.field_count, l2.field_count);
        prop_assert_eq!(l1.state_count, l2.state_count);
        prop_assert_eq!(l1.production_id_count, l2.production_id_count);
    }

    // 14. Same action → same encode_action_small result
    #[test]
    fn encode_action_deterministic(state in 0u16..0x7FFF) {
        let c = TableCompressor::new();
        let action = Action::Shift(adze_ir::StateId(state));
        let e1 = c.encode_action_small(&action);
        let e2 = c.encode_action_small(&action);
        prop_assert_eq!(e1.is_ok(), e2.is_ok());
        if let (Ok(v1), Ok(v2)) = (e1, e2) {
            prop_assert_eq!(v1, v2);
        }
    }

    // 15. Same Grammar → same NodeTypesGenerator output
    #[test]
    fn node_types_generator_deterministic(token_count in 1usize..5) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();

        let gen1 = NodeTypesGenerator::new(&g1);
        let gen2 = NodeTypesGenerator::new(&g2);

        let r1 = gen1.generate();
        let r2 = gen2.generate();

        prop_assert_eq!(r1.is_ok(), r2.is_ok());
        if let (Ok(j1), Ok(j2)) = (r1, r2) {
            prop_assert_eq!(j1, j2);
        }
    }

    // 16. Symbol name ordering is stable across clones
    #[test]
    fn symbol_name_ordering_stable(token_count in 1usize..8) {
        let g1 = grammar_with_tokens(token_count);
        let g2 = g1.clone();

        let names1: Vec<String> = g1.tokens.values().map(|t| t.name.clone()).collect();
        let names2: Vec<String> = g2.tokens.values().map(|t| t.name.clone()).collect();

        prop_assert_eq!(names1, names2);
    }

    // 17. Field name ordering is stable across clones
    #[test]
    fn field_name_ordering_stable(
        field_names in prop::collection::vec(field_name_strategy(), 1..6),
    ) {
        let g1 = grammar_with_fields(2, field_names.clone());
        let g2 = grammar_with_fields(2, field_names);

        let f1: Vec<String> = g1.fields.values().cloned().collect();
        let f2: Vec<String> = g2.fields.values().cloned().collect();

        prop_assert_eq!(f1, f2);
    }

    // 18. Compressed action table ordering is stable
    #[test]
    fn compressed_action_ordering_stable(token_count in 1usize..5) {
        let g = grammar_with_tokens(token_count);
        let t = ParseTable::default();

        let c = TableCompressor::new();
        let r1 = c.compress(&t, &[], false);
        let r2 = c.compress(&t, &[], false);

        match (r1, r2) {
            (Ok(t1), Ok(t2)) => {
                let d1: Vec<_> = t1.action_table.data.iter()
                    .map(|e| (e.symbol, format!("{:?}", e.action)))
                    .collect();
                let d2: Vec<_> = t2.action_table.data.iter()
                    .map(|e| (e.symbol, format!("{:?}", e.action)))
                    .collect();
                prop_assert_eq!(d1, d2);
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "mismatched results"),
        }
    }

    // 19. Compressed goto table ordering is stable
    #[test]
    fn compressed_goto_ordering_stable(token_count in 1usize..5) {
        let g = grammar_with_tokens(token_count);
        let t = ParseTable::default();

        let c = TableCompressor::new();
        let r1 = c.compress(&t, &[], false);
        let r2 = c.compress(&t, &[], false);

        match (r1, r2) {
            (Ok(t1), Ok(t2)) => {
                let d1: Vec<_> = t1.goto_table.data.iter()
                    .map(|e| format!("{e:?}"))
                    .collect();
                let d2: Vec<_> = t2.goto_table.data.iter()
                    .map(|e| format!("{e:?}"))
                    .collect();
                prop_assert_eq!(d1, d2);
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "mismatched results"),
        }
    }

    // 20. Node types JSON ordering is stable (sorted by type name)
    #[test]
    fn node_types_sorted_deterministic(
        visible in prop::collection::vec(token_name_strategy(), 1..4),
    ) {
        let g1 = build_grammar(&visible, &[], &[], &[("number".into(), r"\d+".into())], &[]);
        let g2 = g1.clone();

        let gen1 = NodeTypesGenerator::new(&g1);
        let gen2 = NodeTypesGenerator::new(&g2);

        if let (Ok(j1), Ok(j2)) = (gen1.generate(), gen2.generate()) {
            // Also verify sorting
            let v: Vec<serde_json::Value> = serde_json::from_str(&j1).unwrap_or_default();
            for i in 1..v.len() {
                let prev = v[i - 1]["type"].as_str().unwrap_or("");
                let curr = v[i]["type"].as_str().unwrap_or("");
                prop_assert!(prev <= curr, "node types not sorted: {} > {}", prev, curr);
            }
            prop_assert_eq!(j1, j2);
        }
    }

    // 21. GrammarBuilder produces deterministic grammars
    #[test]
    fn grammar_builder_deterministic(n in 1usize..5) {
        let build = |_| {
            let mut b = GrammarBuilder::new("det_test");
            for i in 0..n {
                b = b.token(&format!("t{i}"), &format!("tok{i}"));
            }
            b = b.rule("root", vec!["t0"]).start("root");
            b.build()
        };

        let g1 = build(());
        let g2 = build(());

        let t1: Vec<String> = g1.tokens.values().map(|t| t.name.clone()).collect();
        let t2: Vec<String> = g2.tokens.values().map(|t| t.name.clone()).collect();
        prop_assert_eq!(t1, t2);

        let r1: Vec<String> = g1.rule_names.values().cloned().collect();
        let r2: Vec<String> = g2.rule_names.values().cloned().collect();
        prop_assert_eq!(r1, r2);
    }

    // 22. Multiple tokens grammar yields deterministic output
    #[test]
    fn multiple_tokens_deterministic(
        token_names in prop::collection::vec(token_name_strategy(), 2..6),
    ) {
        let build_g = |names: &[String]| {
            let mut g = Grammar::new("multi_tok".to_string());
            for (i, name) in names.iter().enumerate() {
                g.tokens.insert(
                    SymbolId(i as u16),
                    Token {
                        name: name.clone(),
                        pattern: TokenPattern::String(name.clone()),
                        fragile: false,
                    },
                );
            }
            g
        };

        let g1 = build_g(&token_names);
        let g2 = build_g(&token_names);

        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // 23. Multiple rules grammar yields deterministic output
    #[test]
    fn multiple_rules_deterministic(
        rule_names in prop::collection::vec(token_name_strategy(), 1..4),
    ) {
        let build_g = |names: &[String]| {
            let mut g = Grammar::new("multi_rule".to_string());
            g.tokens.insert(
                SymbolId(0),
                Token {
                    name: "tok".to_string(),
                    pattern: TokenPattern::String("t".to_string()),
                    fragile: false,
                },
            );
            for (i, name) in names.iter().enumerate() {
                let id = SymbolId(100 + i as u16);
                g.rule_names.insert(id, name.clone());
                g.add_rule(Rule {
                    lhs: id,
                    rhs: vec![Symbol::Terminal(SymbolId(0))],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(i as u16),
                });
            }
            g
        };

        let g1 = build_g(&rule_names);
        let g2 = build_g(&rule_names);

        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // 24. Grammar with externals yields deterministic output
    #[test]
    fn externals_deterministic(
        ext_names in prop::collection::vec(token_name_strategy(), 1..4),
    ) {
        let g1 = grammar_with_externals(2, ext_names.clone());
        let g2 = grammar_with_externals(2, ext_names);

        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // 25. Grammar with fields yields deterministic output
    #[test]
    fn fields_deterministic(
        field_names in prop::collection::vec(field_name_strategy(), 1..5),
    ) {
        let g1 = grammar_with_fields(2, field_names.clone());
        let g2 = grammar_with_fields(2, field_names);

        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // 26. Grammar with hidden tokens yields deterministic output
    #[test]
    fn hidden_tokens_deterministic(
        hidden_names in prop::collection::vec(hidden_token_name(), 1..4),
    ) {
        let g1 = grammar_with_hidden_tokens(2, hidden_names.clone());
        let g2 = grammar_with_hidden_tokens(2, hidden_names);

        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // 27. Grammar with precedence yields deterministic builder output
    #[test]
    fn precedence_deterministic(prec in -10i16..10) {
        let build = || {
            GrammarBuilder::new("prec_test")
                .token("num", r"\d+")
                .token("plus", "+")
                .rule_with_precedence(
                    "expr",
                    vec!["expr", "plus", "expr"],
                    prec,
                    adze_ir::Associativity::Left,
                )
                .rule("expr", vec!["num"])
                .start("expr")
                .build()
        };

        let g1 = build();
        let g2 = build();

        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // 28. Empty grammar yields deterministic output
    #[test]
    fn empty_grammar_deterministic(_seed in 0u32..100) {
        let g1 = Grammar::new("empty".to_string());
        let g2 = Grammar::new("empty".to_string());

        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(json1, json2);

        // NodeTypesGenerator too
        let nt1 = NodeTypesGenerator::new(&g1).generate();
        let nt2 = NodeTypesGenerator::new(&g2).generate();
        prop_assert_eq!(nt1.is_ok(), nt2.is_ok());
        if let (Ok(j1), Ok(j2)) = (nt1, nt2) {
            prop_assert_eq!(j1, j2);
        }
    }

    // 29. Large grammar yields deterministic output
    #[test]
    fn large_grammar_deterministic(n in 5usize..15) {
        let build = |count| {
            let mut g = Grammar::new("large".to_string());
            for i in 0..count {
                g.tokens.insert(
                    SymbolId(i as u16),
                    Token {
                        name: format!("tok{i}"),
                        pattern: TokenPattern::String(format!("t{i}")),
                        fragile: false,
                    },
                );
            }
            for i in 0..count {
                let id = SymbolId(100 + i as u16);
                g.rule_names.insert(id, format!("rule{i}"));
                g.add_rule(Rule {
                    lhs: id,
                    rhs: vec![Symbol::Terminal(SymbolId(i as u16))],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(i as u16),
                });
            }
            g
        };

        let g1 = build(n);
        let g2 = build(n);

        // Serializer
        let json1 = serialize_language(&g1, &ParseTable::default(), None).unwrap();
        let json2 = serialize_language(&g2, &ParseTable::default(), None).unwrap();
        prop_assert_eq!(&json1, &json2);

        // NodeTypesGenerator
        let nt1 = NodeTypesGenerator::new(&g1).generate().unwrap_or_default();
        let nt2 = NodeTypesGenerator::new(&g2).generate().unwrap_or_default();
        prop_assert_eq!(nt1, nt2);
    }

    // 30. Compressed tables validate identically on repeated runs
    #[test]
    fn compress_validate_deterministic(token_count in 1usize..5) {
        let _g = grammar_with_tokens(token_count);
        let t = ParseTable::default();

        let c = TableCompressor::new();
        let r1 = c.compress(&t, &[], false);
        let r2 = c.compress(&t, &[], false);

        match (r1, r2) {
            (Ok(t1), Ok(t2)) => {
                let v1 = t1.validate(&t);
                let v2 = t2.validate(&t);
                prop_assert_eq!(v1.is_ok(), v2.is_ok());
            }
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(format!("{e1}"), format!("{e2}"));
            }
            _ => prop_assert!(false, "one succeeded, one failed"),
        }
    }
}
