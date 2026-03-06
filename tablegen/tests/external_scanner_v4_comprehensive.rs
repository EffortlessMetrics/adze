//! Comprehensive v4 tests for external scanner support in adze-tablegen.
//!
//! Covers: grammar construction with externals, generated code, ABI builder,
//! multiple externals, mixed grammars, serialization, node types, edge cases.

mod test_helpers;

use adze_ir::builder::GrammarBuilder;
use adze_ir::{ExternalToken, Grammar, SymbolId, Token, TokenPattern};
use adze_tablegen::AbiLanguageBuilder;
use adze_tablegen::NodeTypesGenerator;
use adze_tablegen::StaticLanguageGenerator;
use adze_tablegen::external_scanner::ExternalScannerGenerator as V1Generator;
use adze_tablegen::external_scanner_v2::ExternalScannerGenerator as V2Generator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_grammar() -> Grammar {
    Grammar::new("test".to_string())
}

fn grammar_with_externals(names_and_ids: &[(&str, u16)]) -> Grammar {
    let mut g = Grammar::new("test".to_string());
    for &(name, id) in names_and_ids {
        g.externals.push(ExternalToken {
            name: name.to_string(),
            symbol_id: SymbolId(id),
        });
    }
    g
}

fn make_v2_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    test_helpers::create_minimal_parse_table(grammar.clone())
}

fn make_test_parse_table() -> adze_glr_core::ParseTable {
    let g = Grammar::new("test".to_string());
    test_helpers::create_minimal_parse_table(g)
}

// ===========================================================================
// 1. Grammar with external tokens (8 tests)
// ===========================================================================

#[test]
fn grammar_single_external_token_stored() {
    let mut g = empty_grammar();
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[0].symbol_id, SymbolId(100));
}

#[test]
fn grammar_multiple_external_tokens_ordered() {
    let g = grammar_with_externals(&[("A", 10), ("B", 20), ("C", 30)]);
    assert_eq!(g.externals.len(), 3);
    assert_eq!(g.externals[0].name, "A");
    assert_eq!(g.externals[1].name, "B");
    assert_eq!(g.externals[2].name, "C");
}

#[test]
fn grammar_builder_adds_external() {
    let g = GrammarBuilder::new("lang")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn grammar_builder_external_has_nonzero_symbol_id() {
    let g = GrammarBuilder::new("lang")
        .token("TOK", "TOK")
        .external("TOK")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert_ne!(g.externals[0].symbol_id, SymbolId(0));
}

#[test]
fn grammar_builder_external_symbol_matches_token() {
    let g = GrammarBuilder::new("lang")
        .token("HEREDOC", "HEREDOC")
        .external("HEREDOC")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();
    let token_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "HEREDOC")
        .map(|(id, _)| *id)
        .unwrap();
    assert_eq!(g.externals[0].symbol_id, token_id);
}

#[test]
fn grammar_externals_initially_empty() {
    let g = empty_grammar();
    assert!(g.externals.is_empty());
}

#[test]
fn grammar_externals_push_increments_len() {
    let mut g = empty_grammar();
    for i in 0..5 {
        g.externals.push(ExternalToken {
            name: format!("T{i}"),
            symbol_id: SymbolId(i),
        });
    }
    assert_eq!(g.externals.len(), 5);
}

#[test]
fn grammar_python_like_has_externals() {
    let g = GrammarBuilder::python_like();
    assert!(!g.externals.is_empty());
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"INDENT"));
    assert!(names.contains(&"DEDENT"));
}

// ===========================================================================
// 2. External token in generated code (8 tests)
// ===========================================================================

#[test]
fn v1_empty_grammar_generates_empty_token_stream() {
    let scanner = V1Generator::new(empty_grammar());
    let code = scanner.generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn v1_single_external_generates_states_constant() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_STATES"));
}

#[test]
fn v1_single_external_generates_symbol_map_constant() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_SYMBOL_MAP"));
}

#[test]
fn v1_single_external_generates_scanner_data() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_DATA"));
}

#[test]
fn v2_empty_grammar_generates_empty_token_stream() {
    let g = empty_grammar();
    let table = make_v2_table(&g);
    let code = V2Generator::new(g, table).generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn v2_single_external_generates_token_count_const() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let table = make_v2_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(code.contains("EXTERNAL_TOKEN_COUNT"));
}

#[test]
fn v2_single_external_generates_state_count_const() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let table = make_v2_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(code.contains("STATE_COUNT"));
}

#[test]
fn static_lang_generator_node_types_includes_externals() {
    let mut g = Grammar::new("test_lang".to_string());
    g.externals.push(ExternalToken {
        name: "HEREDOC".to_string(),
        symbol_id: SymbolId(100),
    });
    let table = make_v2_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    let node_types = slg.generate_node_types();
    assert!(node_types.contains("HEREDOC"));
}

// ===========================================================================
// 3. External token in ABI builder (8 tests)
// ===========================================================================

#[test]
fn abi_builder_grammar_with_no_externals() {
    let g = Grammar::new("test".to_string());
    let table = make_test_parse_table();
    let builder = AbiLanguageBuilder::new(&g, &table);
    let code = builder.generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_external_scanner_code_when_externals_present() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(50),
    });
    let table = make_v2_table(&g);
    let builder = AbiLanguageBuilder::new(&g, &table);
    let code = builder.generate().to_string();
    assert!(code.contains("external_scanner") || code.contains("ExternalScanner"));
}

#[test]
fn abi_builder_no_external_scanner_code_without_externals() {
    let g = Grammar::new("test".to_string());
    let table = make_test_parse_table();
    let builder = AbiLanguageBuilder::new(&g, &table);
    let code = builder.generate().to_string();
    // Without externals, external scanner struct uses null pointers
    assert!(code.contains("null"));
}

#[test]
fn abi_builder_external_token_count_in_output() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "TOK".to_string(),
        symbol_id: SymbolId(10),
    });
    let table = make_v2_table(&g);
    let builder = AbiLanguageBuilder::new(&g, &table);
    let code = builder.generate().to_string();
    assert!(code.contains("external_token_count"));
}

#[test]
fn abi_builder_two_externals_count() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "A".to_string(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "B".to_string(),
        symbol_id: SymbolId(20),
    });
    let table = make_v2_table(&g);
    let builder = AbiLanguageBuilder::new(&g, &table);
    let code = builder.generate().to_string();
    assert!(code.contains("external_token_count"));
}

#[test]
fn abi_builder_constructs_with_externals() {
    let mut g = Grammar::new("abi_test".to_string());
    g.externals.push(ExternalToken {
        name: "SCANNER".to_string(),
        symbol_id: SymbolId(77),
    });
    let table = make_v2_table(&g);
    // Just verify construction succeeds
    let _builder = AbiLanguageBuilder::new(&g, &table);
}

#[test]
fn abi_builder_generate_returns_non_empty_for_externals() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: SymbolId(5),
    });
    let table = make_v2_table(&g);
    let builder = AbiLanguageBuilder::new(&g, &table);
    let code = builder.generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_with_multiple_externals_generates_code() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "TOK_A".to_string(),
        symbol_id: SymbolId(99),
    });
    g.externals.push(ExternalToken {
        name: "TOK_B".to_string(),
        symbol_id: SymbolId(100),
    });
    let table = make_v2_table(&g);
    let builder = AbiLanguageBuilder::new(&g, &table);
    let code = builder.generate().to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 4. Multiple external tokens (5 tests)
// ===========================================================================

#[test]
fn v1_three_externals_symbol_map_order() {
    let g = grammar_with_externals(&[("A", 10), ("B", 20), ("C", 30)]);
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [10, 20, 30]);
}

#[test]
fn v2_five_externals_count() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2), ("C", 3), ("D", 4), ("E", 5)]);
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    assert_eq!(scanner.external_token_count(), 5);
}

#[test]
fn v1_bitmap_rows_match_state_count() {
    let g = grammar_with_externals(&[("X", 10), ("Y", 20)]);
    let bitmap = V1Generator::new(g).generate_state_bitmap(7);
    assert_eq!(bitmap.len(), 7);
    for row in &bitmap {
        assert_eq!(row.len(), 2);
    }
}

#[test]
fn v2_bitmap_from_table_with_three_states() {
    let g = grammar_with_externals(&[("X", 10), ("Y", 20)]);
    let mut table = make_v2_table(&g);
    table.external_scanner_states = vec![vec![true, false], vec![false, true], vec![true, true]];
    let scanner = V2Generator::new(g, table);
    let bitmap = scanner.generate_state_bitmap();
    assert_eq!(bitmap.len(), 3);
    assert_eq!(bitmap[0], [true, false]);
    assert_eq!(bitmap[1], [false, true]);
    assert_eq!(bitmap[2], [true, true]);
}

#[test]
fn grammar_builder_three_externals_preserve_order() {
    let g = GrammarBuilder::new("lang")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("NEWLINE", "NEWLINE")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[1].name, "DEDENT");
    assert_eq!(g.externals[2].name, "NEWLINE");
}

// ===========================================================================
// 5. External + regular tokens mixed (5 tests)
// ===========================================================================

#[test]
fn mixed_grammar_external_count_excludes_regular() {
    let g = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.external_token_count(), 1);
}

#[test]
fn mixed_grammar_symbol_map_only_has_external_ids() {
    let g = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let scanner = V1Generator::new(g.clone());
    let map = scanner.generate_symbol_map();
    assert_eq!(map.len(), 1);
    let indent_id = g.externals[0].symbol_id.0;
    assert_eq!(map[0], indent_id);
}

#[test]
fn mixed_grammar_v2_has_external_tokens() {
    let g = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = make_v2_table(&g);
    assert!(V2Generator::new(g, table).has_external_tokens());
}

#[test]
fn mixed_grammar_tokens_and_externals_separate_counts() {
    let g = GrammarBuilder::new("mixed")
        .token("A", "a")
        .token("B", "b")
        .token("EXT", "EXT")
        .external("EXT")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert_eq!(g.tokens.len(), 3);
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn javascript_like_grammar_has_no_externals() {
    let g = GrammarBuilder::javascript_like();
    assert!(g.externals.is_empty());
}

// ===========================================================================
// 6. External token serialization (5 tests)
// ===========================================================================

#[test]
fn external_token_serialize_json() {
    let tok = ExternalToken {
        name: "HEREDOC".to_string(),
        symbol_id: SymbolId(42),
    };
    let json = serde_json::to_string(&tok).unwrap();
    assert!(json.contains("HEREDOC"));
    assert!(json.contains("42"));
}

#[test]
fn external_token_deserialize_json() {
    let json = r#"{"name":"TEMPLATE","symbol_id":99}"#;
    let tok: ExternalToken = serde_json::from_str(json).unwrap();
    assert_eq!(tok.name, "TEMPLATE");
    assert_eq!(tok.symbol_id, SymbolId(99));
}

#[test]
fn external_token_roundtrip_json() {
    let original = ExternalToken {
        name: "SCANNER_TOK".to_string(),
        symbol_id: SymbolId(77),
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: ExternalToken = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn grammar_externals_roundtrip_json() {
    let mut g = Grammar::new("roundtrip".to_string());
    g.externals.push(ExternalToken {
        name: "A".to_string(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "B".to_string(),
        symbol_id: SymbolId(20),
    });
    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.externals.len(), 2);
    assert_eq!(restored.externals[0].name, "A");
    assert_eq!(restored.externals[1].name, "B");
}

#[test]
fn empty_externals_serializes_as_empty_array() {
    let g = empty_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let externals = val.get("externals").unwrap();
    assert!(externals.is_array());
    assert!(externals.as_array().unwrap().is_empty());
}

// ===========================================================================
// 7. Node types with externals (8 tests)
// ===========================================================================

#[test]
fn node_types_generator_no_externals_succeeds() {
    let g = Grammar::new("test".to_string());
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_generator_with_external_succeeds() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "HEREDOC".to_string(),
        symbol_id: SymbolId(100),
    });
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn static_lang_gen_node_types_includes_visible_external() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "TEMPLATE_STRING".to_string(),
        symbol_id: SymbolId(200),
    });
    let table = make_v2_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    let json = slg.generate_node_types();
    assert!(json.contains("TEMPLATE_STRING"));
}

#[test]
fn static_lang_gen_node_types_excludes_hidden_external() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "_hidden_ext".to_string(),
        symbol_id: SymbolId(200),
    });
    let table = make_v2_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    let json = slg.generate_node_types();
    assert!(!json.contains("_hidden_ext"));
}

#[test]
fn static_lang_gen_node_types_multiple_externals() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });
    g.externals.push(ExternalToken {
        name: "DEDENT".to_string(),
        symbol_id: SymbolId(101),
    });
    let table = make_v2_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    let json = slg.generate_node_types();
    assert!(json.contains("INDENT"));
    assert!(json.contains("DEDENT"));
}

#[test]
fn static_lang_gen_node_types_external_marked_named() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "MY_EXT".to_string(),
        symbol_id: SymbolId(50),
    });
    let table = make_v2_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    let json = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    let ext_entry = arr.iter().find(|v| v["type"] == "MY_EXT").unwrap();
    assert_eq!(ext_entry["named"], true);
}

#[test]
fn serializer_includes_external_token_count() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: SymbolId(10),
    });
    let table = make_v2_table(&g);
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["external_token_count"], 1);
}

#[test]
fn serializer_external_names_in_symbol_names() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "SCANNER_EXT".to_string(),
        symbol_id: SymbolId(10),
    });
    let table = make_v2_table(&g);
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let sym_names = val["symbol_names"].as_array().unwrap();
    assert!(sym_names.iter().any(|n| n == "SCANNER_EXT"));
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn external_symbol_id_zero() {
    let g = grammar_with_externals(&[("ZERO", 0)]);
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [0]);
}

#[test]
fn external_symbol_id_max_u16() {
    let g = grammar_with_externals(&[("MAX", u16::MAX)]);
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [u16::MAX]);
}

#[test]
fn v1_bitmap_zero_states_is_empty() {
    let g = grammar_with_externals(&[("T", 1)]);
    let bitmap = V1Generator::new(g).generate_state_bitmap(0);
    assert!(bitmap.is_empty());
}

#[test]
fn v1_bitmap_single_state_single_token() {
    let g = grammar_with_externals(&[("T", 1)]);
    let bitmap = V1Generator::new(g).generate_state_bitmap(1);
    assert_eq!(bitmap.len(), 1);
    assert_eq!(bitmap[0], [true]);
}

#[test]
fn duplicate_external_names_both_stored() {
    let g = grammar_with_externals(&[("DUP", 10), ("DUP", 20)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.external_token_count(), 2);
    assert_eq!(scanner.generate_symbol_map(), [10, 20]);
}

#[test]
fn empty_name_external_token() {
    let g = grammar_with_externals(&[("", 1)]);
    let scanner = V1Generator::new(g);
    assert!(scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 1);
}

#[test]
fn v2_compute_validity_equals_bitmap() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2)]);
    let mut table = make_v2_table(&g);
    table.external_scanner_states = vec![vec![true, false], vec![false, true]];
    let scanner = V2Generator::new(g, table);
    assert_eq!(
        scanner.compute_state_validity(),
        scanner.generate_state_bitmap()
    );
}

#[test]
fn large_external_token_set_50() {
    let tokens: Vec<(String, u16)> = (0..50).map(|i| (format!("T{i}"), i)).collect();
    let refs: Vec<(&str, u16)> = tokens.iter().map(|(n, id)| (n.as_str(), *id)).collect();
    let g = grammar_with_externals(&refs);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.external_token_count(), 50);
    let map = scanner.generate_symbol_map();
    assert_eq!(map.len(), 50);
    for (i, &val) in map.iter().enumerate() {
        assert_eq!(val, i as u16);
    }
}

// ===========================================================================
// Additional tests to reach 55+ total
// ===========================================================================

#[test]
fn external_token_debug_contains_name() {
    let tok = ExternalToken {
        name: "DBG".to_string(),
        symbol_id: SymbolId(1),
    };
    let dbg_str = format!("{:?}", tok);
    assert!(dbg_str.contains("DBG"));
    assert!(dbg_str.contains("ExternalToken"));
}

#[test]
fn external_token_clone_equals_original() {
    let tok = ExternalToken {
        name: "ORIG".to_string(),
        symbol_id: SymbolId(5),
    };
    let cloned = tok.clone();
    assert_eq!(tok, cloned);
}

#[test]
fn external_token_ne_different_name() {
    let a = ExternalToken {
        name: "A".to_string(),
        symbol_id: SymbolId(1),
    };
    let b = ExternalToken {
        name: "B".to_string(),
        symbol_id: SymbolId(1),
    };
    assert_ne!(a, b);
}

#[test]
fn external_token_ne_different_id() {
    let a = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(1),
    };
    let b = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(2),
    };
    assert_ne!(a, b);
}

#[test]
fn symbol_id_is_copy_no_clone_needed() {
    let tok = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(5),
    };
    let id = tok.symbol_id;
    assert_eq!(id, tok.symbol_id);
}

#[test]
fn v1_has_external_tokens_true() {
    let g = grammar_with_externals(&[("A", 10)]);
    assert!(V1Generator::new(g).has_external_tokens());
}

#[test]
fn v1_has_external_tokens_false() {
    assert!(!V1Generator::new(empty_grammar()).has_external_tokens());
}

#[test]
fn v2_has_external_tokens_true() {
    let g = grammar_with_externals(&[("B", 20)]);
    let table = make_v2_table(&g);
    assert!(V2Generator::new(g, table).has_external_tokens());
}

#[test]
fn v2_has_external_tokens_false() {
    let g = empty_grammar();
    let table = make_v2_table(&g);
    assert!(!V2Generator::new(g, table).has_external_tokens());
}

#[test]
fn static_lang_gen_with_external_constructs() {
    let mut g = Grammar::new("ext_test".to_string());
    g.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: SymbolId(42),
    });
    let table = make_v2_table(&g);
    let _slg = StaticLanguageGenerator::new(g, table);
}

#[test]
fn serializable_language_zero_externals() {
    let g = Grammar::new("test".to_string());
    let table = make_test_parse_table();
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["external_token_count"], 0);
}

#[test]
fn generate_language_builder_with_external() {
    let mut g = Grammar {
        name: "test".to_string(),
        ..Default::default()
    };
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.externals.push(ExternalToken {
        name: "comment".to_string(),
        symbol_id: SymbolId(100),
    });
    let table = make_v2_table(&g);
    let builder = adze_tablegen::LanguageBuilder::new(g, table);
    let result = builder.generate_language();
    assert!(result.is_ok());
    let lang = result.unwrap();
    assert_eq!(lang.external_token_count, 1);
}

#[test]
fn v2_debug_print_validity_does_not_panic() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2)]);
    let mut table = make_v2_table(&g);
    table.external_scanner_states = vec![vec![true, false], vec![false, true]];
    let scanner = V2Generator::new(g, table);
    // Should not panic
    scanner.debug_print_validity();
}

#[test]
fn v1_symbol_map_matches_declaration_ids() {
    let g = grammar_with_externals(&[("FIRST", 50), ("SECOND", 60), ("THIRD", 70)]);
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [50, 60, 70]);
}

#[test]
fn grammar_with_hidden_and_visible_externals() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "_hidden".to_string(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "VISIBLE".to_string(),
        symbol_id: SymbolId(20),
    });
    let table = make_v2_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    let json = slg.generate_node_types();
    assert!(json.contains("VISIBLE"));
    assert!(!json.contains("_hidden"));
}
