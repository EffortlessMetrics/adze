//! v6 tests for external scanner code generation in adze-tablegen.
//!
//! 8 categories × 8 tests = 64 tests covering:
//! 1. No externals → no scanner code
//! 2. Externals → scanner signatures present
//! 3. External token names preserved
//! 4. Multiple external tokens handled
//! 5. ExternalScanner struct defaults
//! 6. Mixed regular and external tokens
//! 7. Generated code structure for scanner integration
//! 8. Edge cases: single external, many externals

mod test_helpers;

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{ExternalToken, Grammar, SymbolId};
use adze_tablegen::AbiLanguageBuilder;
use adze_tablegen::StaticLanguageGenerator;
use adze_tablegen::external_scanner::ExternalScannerGenerator as V1Generator;
use adze_tablegen::external_scanner_v2::ExternalScannerGenerator as V2Generator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a real LR(1) parse table from a grammar.
fn build_table(grammar: &mut Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW computation failed");
    build_lr1_automaton(grammar, &ff).expect("LR(1) automaton construction failed")
}

/// Create a minimal parse table via test_helpers (no LR build).
fn minimal_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    test_helpers::create_minimal_parse_table(grammar.clone())
}

/// Grammar with one rule, one token, and N externals created via GrammarBuilder.
fn grammar_with_ext(ext_names: &[&str]) -> Grammar {
    let mut b = GrammarBuilder::new("ext_v6")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["ID"])
        .start("root");
    for name in ext_names {
        b = b.token(name, name).external(name);
    }
    b.build()
}

/// Grammar with no externals.
fn grammar_no_ext() -> Grammar {
    GrammarBuilder::new("plain")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["ID"])
        .start("root")
        .build()
}

/// Push N raw ExternalTokens into a Grammar.
fn push_externals(g: &mut Grammar, n: u16, base_id: u16) {
    for i in 0..n {
        g.externals.push(ExternalToken {
            name: format!("EXT_{i}"),
            symbol_id: SymbolId(base_id + i),
        });
    }
}

// ===========================================================================
// 1. Grammar with no externals produces no scanner code (8 tests)
// ===========================================================================

#[test]
fn test_no_ext_v1_has_external_tokens_false() {
    let g = Grammar::new("empty".to_string());
    assert!(!V1Generator::new(g).has_external_tokens());
}

#[test]
fn test_no_ext_v1_external_token_count_zero() {
    let g = Grammar::new("empty".to_string());
    assert_eq!(V1Generator::new(g).external_token_count(), 0);
}

#[test]
fn test_no_ext_v1_scanner_interface_empty() {
    let g = Grammar::new("empty".to_string());
    let code = V1Generator::new(g).generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn test_no_ext_v1_symbol_map_empty() {
    let g = Grammar::new("empty".to_string());
    let map = V1Generator::new(g).generate_symbol_map();
    assert!(map.is_empty());
}

#[test]
fn test_no_ext_v2_has_external_tokens_false() {
    let g = Grammar::new("empty".to_string());
    let table = minimal_table(&g);
    assert!(!V2Generator::new(g, table).has_external_tokens());
}

#[test]
fn test_no_ext_v2_scanner_interface_empty() {
    let g = Grammar::new("empty".to_string());
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table).generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn test_no_ext_abi_builder_no_scanner_data() {
    let g = Grammar::new("empty".to_string());
    let table = minimal_table(&g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    // Without externals the code should reference null / 0 for external scanner
    assert!(!code.contains("EXTERNAL_SCANNER_STATES"));
}

#[test]
fn test_no_ext_grammar_builder_externals_empty() {
    let g = grammar_no_ext();
    assert!(g.externals.is_empty());
}

// ===========================================================================
// 2. Grammar with externals includes scanner signatures (8 tests)
// ===========================================================================

#[test]
fn test_ext_v1_has_external_tokens_true() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 50);
    assert!(V1Generator::new(g).has_external_tokens());
}

#[test]
fn test_ext_v1_scanner_interface_nonempty() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 50);
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_ext_v1_interface_contains_states() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 2, 10);
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_STATES"));
}

#[test]
fn test_ext_v1_interface_contains_symbol_map() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 30);
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_SYMBOL_MAP"));
}

#[test]
fn test_ext_v2_scanner_interface_nonempty() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 50);
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_ext_v2_interface_has_token_count_const() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 2, 10);
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(code.contains("EXTERNAL_TOKEN_COUNT"));
}

#[test]
fn test_ext_v2_interface_has_state_count_const() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 10);
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(code.contains("STATE_COUNT"));
}

#[test]
fn test_ext_abi_builder_mentions_scanner_when_present() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 50);
    let table = minimal_table(&g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(code.contains("external_scanner") || code.contains("ExternalScanner"));
}

// ===========================================================================
// 3. External token names preserved in generated code (8 tests)
// ===========================================================================

#[test]
fn test_name_preserved_in_externals_vec() {
    let g = grammar_with_ext(&["HEREDOC"]);
    assert_eq!(g.externals[0].name, "HEREDOC");
}

#[test]
fn test_name_preserved_across_multiple() {
    let g = grammar_with_ext(&["INDENT", "DEDENT", "NEWLINE"]);
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["INDENT", "DEDENT", "NEWLINE"]);
}

#[test]
fn test_name_in_node_types_output() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "TEMPLATE_STRING".to_string(),
        symbol_id: SymbolId(200),
    });
    let table = minimal_table(&g);
    let json = StaticLanguageGenerator::new(g, table).generate_node_types();
    assert!(json.contains("TEMPLATE_STRING"));
}

#[test]
fn test_name_in_serializer_symbol_names() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "REGEX_BODY".to_string(),
        symbol_id: SymbolId(77),
    });
    let table = minimal_table(&g);
    let json_str = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let sym_names = val["symbol_names"].as_array().unwrap();
    assert!(sym_names.iter().any(|n| n == "REGEX_BODY"));
}

#[test]
fn test_name_lowercase_preserved() {
    let g = grammar_with_ext(&["indent"]);
    assert_eq!(g.externals[0].name, "indent");
}

#[test]
fn test_name_underscore_prefix_preserved() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "_hidden_scan".to_string(),
        symbol_id: SymbolId(5),
    });
    assert_eq!(g.externals[0].name, "_hidden_scan");
}

#[test]
fn test_name_order_matches_insertion() {
    let names = ["ALPHA", "BETA", "GAMMA", "DELTA"];
    let g = grammar_with_ext(&names);
    for (i, &expected) in names.iter().enumerate() {
        assert_eq!(g.externals[i].name, expected);
    }
}

#[test]
fn test_name_survives_serialization_roundtrip() {
    let tok = ExternalToken {
        name: "LONG_EXTERNAL_TOKEN_NAME".to_string(),
        symbol_id: SymbolId(123),
    };
    let json = serde_json::to_string(&tok).unwrap();
    let restored: ExternalToken = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.name, "LONG_EXTERNAL_TOKEN_NAME");
}

// ===========================================================================
// 4. Multiple external tokens handled (8 tests)
// ===========================================================================

#[test]
fn test_multi_ext_v1_count() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 5, 10);
    assert_eq!(V1Generator::new(g).external_token_count(), 5);
}

#[test]
fn test_multi_ext_v1_symbol_map_length() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 4, 100);
    assert_eq!(V1Generator::new(g).generate_symbol_map().len(), 4);
}

#[test]
fn test_multi_ext_v1_symbol_map_values() {
    let mut g = Grammar::new("test".to_string());
    for (i, id) in [10u16, 20, 30].iter().enumerate() {
        g.externals.push(ExternalToken {
            name: format!("T{i}"),
            symbol_id: SymbolId(*id),
        });
    }
    assert_eq!(V1Generator::new(g).generate_symbol_map(), [10, 20, 30]);
}

#[test]
fn test_multi_ext_v1_bitmap_rows_equal_states() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 3, 1);
    let bitmap = V1Generator::new(g).generate_state_bitmap(7);
    assert_eq!(bitmap.len(), 7);
}

#[test]
fn test_multi_ext_v1_bitmap_cols_equal_externals() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 6, 1);
    let bitmap = V1Generator::new(g).generate_state_bitmap(2);
    for row in &bitmap {
        assert_eq!(row.len(), 6);
    }
}

#[test]
fn test_multi_ext_v2_count() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 3, 40);
    let table = minimal_table(&g);
    assert_eq!(V2Generator::new(g, table).external_token_count(), 3);
}

#[test]
fn test_multi_ext_builder_stores_all() {
    let g = grammar_with_ext(&["A", "B", "C", "D"]);
    assert_eq!(g.externals.len(), 4);
}

#[test]
fn test_multi_ext_serializer_count() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 5, 10);
    let table = minimal_table(&g);
    let json_str = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(val["external_token_count"], 5);
}

// ===========================================================================
// 5. External scanner struct defaults (8 tests)
// ===========================================================================

#[test]
fn test_default_grammar_externals_empty() {
    let g = Grammar::default();
    assert!(g.externals.is_empty());
}

#[test]
fn test_external_token_eq() {
    let a = ExternalToken {
        name: "X".to_string(),
        symbol_id: SymbolId(1),
    };
    let b = ExternalToken {
        name: "X".to_string(),
        symbol_id: SymbolId(1),
    };
    assert_eq!(a, b);
}

#[test]
fn test_external_token_ne_name() {
    let a = ExternalToken {
        name: "X".to_string(),
        symbol_id: SymbolId(1),
    };
    let b = ExternalToken {
        name: "Y".to_string(),
        symbol_id: SymbolId(1),
    };
    assert_ne!(a, b);
}

#[test]
fn test_external_token_ne_id() {
    let a = ExternalToken {
        name: "X".to_string(),
        symbol_id: SymbolId(1),
    };
    let b = ExternalToken {
        name: "X".to_string(),
        symbol_id: SymbolId(2),
    };
    assert_ne!(a, b);
}

#[test]
fn test_external_token_debug_contains_name() {
    let tok = ExternalToken {
        name: "MY_TOKEN".to_string(),
        symbol_id: SymbolId(99),
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("MY_TOKEN"));
}

#[test]
fn test_external_token_debug_contains_id() {
    let tok = ExternalToken {
        name: "TOK".to_string(),
        symbol_id: SymbolId(42),
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("42"));
}

#[test]
fn test_symbol_id_is_copy() {
    let id = SymbolId(7);
    let id2 = id; // Copy, not move
    let id3 = id; // Still accessible
    assert_eq!(id2, id3);
}

#[test]
fn test_external_token_clone_matches_original() {
    let tok = ExternalToken {
        name: "SCAN".to_string(),
        symbol_id: SymbolId(55),
    };
    let tok2 = tok.clone();
    assert_eq!(tok, tok2);
}

// ===========================================================================
// 6. Mixed regular and external tokens (8 tests)
// ===========================================================================

#[test]
fn test_mixed_regular_not_in_externals() {
    let g = grammar_with_ext(&["INDENT"]);
    let ext_names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(!ext_names.contains(&"ID"));
}

#[test]
fn test_mixed_external_in_tokens_map() {
    let g = grammar_with_ext(&["INDENT"]);
    let tok_names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(tok_names.contains(&"INDENT"));
}

#[test]
fn test_mixed_regular_and_external_both_in_tokens() {
    let g = grammar_with_ext(&["DEDENT"]);
    let tok_names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(tok_names.contains(&"ID"));
    assert!(tok_names.contains(&"DEDENT"));
}

#[test]
fn test_mixed_v1_count_excludes_regular() {
    let g = grammar_with_ext(&["EXT_ONE"]);
    // V1 should only count externals, not regular tokens
    assert_eq!(V1Generator::new(g).external_token_count(), 1);
}

#[test]
fn test_mixed_abi_builder_produces_code() {
    let mut g = grammar_with_ext(&["INDENT"]);
    let table = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_mixed_abi_builder_reports_external_count() {
    let mut g = grammar_with_ext(&["INDENT"]);
    let table = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(code.contains("external_token_count"));
}

#[test]
fn test_mixed_language_builder_external_count_correct() {
    let g = grammar_with_ext(&["SCAN_A", "SCAN_B"]);
    let table = minimal_table(&g);
    let builder = adze_tablegen::LanguageBuilder::new(g, table);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.external_token_count, 2);
}

#[test]
fn test_mixed_node_types_includes_external_name() {
    let g = grammar_with_ext(&["HEREDOC"]);
    let table = minimal_table(&g);
    let json = StaticLanguageGenerator::new(g, table).generate_node_types();
    assert!(json.contains("HEREDOC"));
}

// ===========================================================================
// 7. Generated code structure for scanner integration (8 tests)
// ===========================================================================

#[test]
fn test_codegen_v1_interface_has_scanner_data_struct() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 10);
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(code.contains("TSExternalScannerData"));
}

#[test]
fn test_codegen_v2_interface_has_helper_fn() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 2, 10);
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(code.contains("get_valid_external_tokens"));
}

#[test]
fn test_codegen_v2_interface_has_scanner_data() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 10);
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(code.contains("EXTERNAL_SCANNER_DATA"));
}

#[test]
fn test_codegen_static_lang_gen_produces_code() {
    let mut g = grammar_with_ext(&["INDENT"]);
    let table = build_table(&mut g);
    let slg = StaticLanguageGenerator::new(g, table);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_codegen_abi_builder_null_scanner_without_ext() {
    let g = Grammar::new("test".to_string());
    let table = minimal_table(&g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(code.contains("null"));
}

#[test]
fn test_codegen_v1_state_bitmap_all_true() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 2, 10);
    let bitmap = V1Generator::new(g).generate_state_bitmap(3);
    // v1 returns all-true since it cannot compute state-based validity
    for row in &bitmap {
        assert!(row.iter().all(|&v| v));
    }
}

#[test]
fn test_codegen_v2_bitmap_from_parse_table() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 2, 10);
    let mut table = minimal_table(&g);
    table.external_scanner_states = vec![vec![true, false], vec![false, true]];
    let bitmap = V2Generator::new(g, table).generate_state_bitmap();
    assert_eq!(bitmap, vec![vec![true, false], vec![false, true]]);
}

#[test]
fn test_codegen_v2_validity_equals_bitmap() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 3, 10);
    let mut table = minimal_table(&g);
    table.external_scanner_states = vec![vec![true, true, false]];
    let scanner = V2Generator::new(g, table);
    assert_eq!(
        scanner.compute_state_validity(),
        scanner.generate_state_bitmap()
    );
}

// ===========================================================================
// 8. Edge cases: single external, many externals (8 tests)
// ===========================================================================

#[test]
fn test_edge_single_external_v1_map_len_one() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 77);
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map.len(), 1);
    assert_eq!(map[0], 77);
}

#[test]
fn test_edge_single_external_v2_count() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 1, 42);
    let table = minimal_table(&g);
    assert_eq!(V2Generator::new(g, table).external_token_count(), 1);
}

#[test]
fn test_edge_many_externals_20() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 20, 100);
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map.len(), 20);
    for (i, &val) in map.iter().enumerate() {
        assert_eq!(val, 100 + i as u16);
    }
}

#[test]
fn test_edge_symbol_id_zero() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "ZERO".to_string(),
        symbol_id: SymbolId(0),
    });
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [0]);
}

#[test]
fn test_edge_symbol_id_u16_max() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "MAX".to_string(),
        symbol_id: SymbolId(u16::MAX),
    });
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [u16::MAX]);
}

#[test]
fn test_edge_hidden_external_excluded_from_node_types() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "_hidden".to_string(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "VISIBLE".to_string(),
        symbol_id: SymbolId(20),
    });
    let table = minimal_table(&g);
    let json = StaticLanguageGenerator::new(g, table).generate_node_types();
    assert!(json.contains("VISIBLE"));
    assert!(!json.contains("_hidden"));
}

#[test]
fn test_edge_v1_bitmap_zero_states() {
    let mut g = Grammar::new("test".to_string());
    push_externals(&mut g, 2, 10);
    let bitmap = V1Generator::new(g).generate_state_bitmap(0);
    assert!(bitmap.is_empty());
}

#[test]
fn test_edge_pipeline_five_externals_real_table() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("E1", "E1")
        .token("E2", "E2")
        .token("E3", "E3")
        .token("E4", "E4")
        .token("E5", "E5")
        .external("E1")
        .external("E2")
        .external("E3")
        .external("E4")
        .external("E5")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    let json_str = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(val["external_token_count"], 5);
}
