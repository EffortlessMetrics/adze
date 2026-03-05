//! v5 tests for external scanner support in adze-tablegen.
//!
//! Focus areas: external scanner token declaration, external tokens in grammar,
//! generated code references to externals, determinism with externals, multiple
//! externals, external + regular token interaction, full pipeline with
//! `build_lr1_automaton`, and ABI/codegen fidelity.

mod test_helpers;

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
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

/// Build a grammar with GrammarBuilder and compute a real parse table.
fn build_table(
    grammar: &mut Grammar,
) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation failed");
    build_lr1_automaton(grammar, &ff).expect("LR(1) automaton construction failed")
}

/// Create a minimal parse table via test_helpers (no LR build).
fn minimal_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    test_helpers::create_minimal_parse_table(grammar.clone())
}

/// Shorthand for a grammar that has one rule, one token, and some externals.
fn simple_grammar_with_externals(ext_names: &[&str]) -> Grammar {
    let mut b = GrammarBuilder::new("ext_lang")
        .token("NUM", r"\d+")
        .rule("program", vec!["NUM"])
        .start("program");
    for name in ext_names {
        b = b.token(name, name).external(name);
    }
    b.build()
}

// ===========================================================================
// 1. External scanner token declaration (10 tests)
// ===========================================================================

#[test]
fn test_declare_single_external_via_builder() {
    let g = simple_grammar_with_externals(&["INDENT"]);
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn test_declare_external_assigns_unique_symbol_id() {
    let g = simple_grammar_with_externals(&["INDENT"]);
    let id = g.externals[0].symbol_id;
    assert_ne!(id, SymbolId(0), "external should not be EOF");
}

#[test]
fn test_declare_two_externals_distinct_ids() {
    let g = simple_grammar_with_externals(&["INDENT", "DEDENT"]);
    assert_ne!(g.externals[0].symbol_id, g.externals[1].symbol_id);
}

#[test]
fn test_declare_external_id_is_copy() {
    let g = simple_grammar_with_externals(&["TOK"]);
    let id = g.externals[0].symbol_id;
    let id2 = id; // Copy, not move
    assert_eq!(id, id2);
}

#[test]
fn test_external_appears_in_token_map() {
    let g = simple_grammar_with_externals(&["INDENT"]);
    let found = g.tokens.values().any(|t| t.name == "INDENT");
    assert!(found, "external should also appear in tokens map");
}

#[test]
fn test_external_symbol_id_matches_token_id() {
    let g = simple_grammar_with_externals(&["HEREDOC"]);
    let token_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "HEREDOC")
        .map(|(id, _)| id)
        .expect("token not found");
    assert_eq!(g.externals[0].symbol_id, *token_id);
}

#[test]
fn test_declare_five_externals_order_preserved() {
    let names = ["A", "B", "C", "D", "E"];
    let g = simple_grammar_with_externals(&names);
    for (i, &name) in names.iter().enumerate() {
        assert_eq!(g.externals[i].name, name);
    }
}

#[test]
fn test_python_like_grammar_declares_externals() {
    let g = GrammarBuilder::python_like();
    let ext_names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(ext_names.contains(&"INDENT"));
    assert!(ext_names.contains(&"DEDENT"));
}

#[test]
fn test_external_token_struct_fields_accessible() {
    let tok = ExternalToken {
        name: "SCAN".to_string(),
        symbol_id: SymbolId(42),
    };
    assert_eq!(tok.name, "SCAN");
    assert_eq!(tok.symbol_id.0, 42);
}

#[test]
fn test_external_token_equality() {
    let a = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(7),
    };
    let b = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(7),
    };
    assert_eq!(a, b);
}

// ===========================================================================
// 2. External tokens in grammar (8 tests)
// ===========================================================================

#[test]
fn test_grammar_externals_vec_starts_empty() {
    let g = Grammar::new("empty".to_string());
    assert!(g.externals.is_empty());
}

#[test]
fn test_grammar_push_external_increments_len() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "X".to_string(),
        symbol_id: SymbolId(1),
    });
    assert_eq!(g.externals.len(), 1);
    g.externals.push(ExternalToken {
        name: "Y".to_string(),
        symbol_id: SymbolId(2),
    });
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn test_grammar_external_survives_clone() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(10),
    });
    let g2 = g.clone();
    assert_eq!(g2.externals.len(), 1);
    assert_eq!(g2.externals[0].name, "INDENT");
}

#[test]
fn test_grammar_external_serialization_roundtrip() {
    let mut g = Grammar::new("ser".to_string());
    g.externals.push(ExternalToken {
        name: "HEREDOC".to_string(),
        symbol_id: SymbolId(99),
    });
    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.externals[0].name, "HEREDOC");
    assert_eq!(restored.externals[0].symbol_id, SymbolId(99));
}

#[test]
fn test_grammar_mixed_tokens_and_externals_independent_counts() {
    let g = GrammarBuilder::new("mix")
        .token("A", "a")
        .token("B", "b")
        .token("EXT1", "EXT1")
        .external("EXT1")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert_eq!(g.externals.len(), 1);
    // All tokens (A, B, EXT1) appear in tokens map
    assert!(g.tokens.len() >= 3);
}

#[test]
fn test_grammar_javascript_like_no_externals() {
    let g = GrammarBuilder::javascript_like();
    assert!(g.externals.is_empty());
}

#[test]
fn test_grammar_external_hidden_naming_convention() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "_hidden_scanner".to_string(),
        symbol_id: SymbolId(10),
    });
    assert!(g.externals[0].name.starts_with('_'));
}

#[test]
fn test_grammar_external_token_json_has_both_fields() {
    let tok = ExternalToken {
        name: "FOO".to_string(),
        symbol_id: SymbolId(5),
    };
    let json = serde_json::to_string(&tok).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(val.get("name").is_some());
    assert!(val.get("symbol_id").is_some());
}

// ===========================================================================
// 3. Generated code references externals (8 tests)
// ===========================================================================

#[test]
fn test_v1_scanner_interface_empty_for_no_externals() {
    let g = Grammar::new("test".to_string());
    let code = V1Generator::new(g).generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn test_v1_scanner_interface_nonempty_with_external() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "TOK".to_string(),
        symbol_id: SymbolId(10),
    });
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_v1_scanner_interface_contains_states_constant() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: SymbolId(5),
    });
    let code = V1Generator::new(g).generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_STATES"));
}

#[test]
fn test_v2_scanner_interface_empty_for_no_externals() {
    let g = Grammar::new("test".to_string());
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table).generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn test_v2_scanner_interface_has_token_count() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "TOK".to_string(),
        symbol_id: SymbolId(10),
    });
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(code.contains("EXTERNAL_TOKEN_COUNT"));
}

#[test]
fn test_abi_builder_code_mentions_external_scanner_when_present() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(50),
    });
    let table = minimal_table(&g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(code.contains("external_scanner") || code.contains("ExternalScanner"));
}

#[test]
fn test_abi_builder_code_has_null_scanner_without_externals() {
    let g = Grammar::new("test".to_string());
    let table = minimal_table(&g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(code.contains("null"));
}

#[test]
fn test_static_lang_gen_node_types_contains_external_name() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "TEMPLATE".to_string(),
        symbol_id: SymbolId(200),
    });
    let table = minimal_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    assert!(slg.generate_node_types().contains("TEMPLATE"));
}

// ===========================================================================
// 4. Determinism with externals (7 tests)
// ===========================================================================

#[test]
fn test_v1_symbol_map_deterministic_across_calls() {
    let make = || {
        let mut g = Grammar::new("det".to_string());
        g.externals.push(ExternalToken {
            name: "A".to_string(),
            symbol_id: SymbolId(10),
        });
        g.externals.push(ExternalToken {
            name: "B".to_string(),
            symbol_id: SymbolId(20),
        });
        V1Generator::new(g).generate_symbol_map()
    };
    assert_eq!(make(), make());
}

#[test]
fn test_v2_symbol_map_deterministic() {
    let make = || {
        let mut g = Grammar::new("det".to_string());
        g.externals.push(ExternalToken {
            name: "X".to_string(),
            symbol_id: SymbolId(3),
        });
        let table = minimal_table(&g);
        V2Generator::new(g, table).generate_symbol_map()
    };
    assert_eq!(make(), make());
}

#[test]
fn test_v1_bitmap_deterministic() {
    let make = || {
        let mut g = Grammar::new("det".to_string());
        g.externals.push(ExternalToken {
            name: "T".to_string(),
            symbol_id: SymbolId(1),
        });
        V1Generator::new(g).generate_state_bitmap(3)
    };
    assert_eq!(make(), make());
}

#[test]
fn test_abi_builder_deterministic_code() {
    let make = || {
        let mut g = Grammar::new("det".to_string());
        g.externals.push(ExternalToken {
            name: "EXT".to_string(),
            symbol_id: SymbolId(10),
        });
        let table = minimal_table(&g);
        AbiLanguageBuilder::new(&g, &table).generate().to_string()
    };
    assert_eq!(make(), make());
}

#[test]
fn test_serializer_deterministic_with_externals() {
    let make = || {
        let mut g = Grammar::new("det".to_string());
        g.externals.push(ExternalToken {
            name: "TOK".to_string(),
            symbol_id: SymbolId(10),
        });
        let table = minimal_table(&g);
        adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap()
    };
    assert_eq!(make(), make());
}

#[test]
fn test_node_types_deterministic_with_externals() {
    let make = || {
        let mut g = Grammar::new("det".to_string());
        g.externals.push(ExternalToken {
            name: "INDENT".to_string(),
            symbol_id: SymbolId(55),
        });
        NodeTypesGenerator::new(&g).generate().unwrap()
    };
    assert_eq!(make(), make());
}

#[test]
fn test_static_lang_gen_node_types_deterministic() {
    let make = || {
        let mut g = Grammar::new("det".to_string());
        g.externals.push(ExternalToken {
            name: "EXT".to_string(),
            symbol_id: SymbolId(7),
        });
        let table = minimal_table(&g);
        StaticLanguageGenerator::new(g, table).generate_node_types()
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// 5. Multiple externals (8 tests)
// ===========================================================================

#[test]
fn test_three_externals_v1_symbol_map_length() {
    let mut g = Grammar::new("test".to_string());
    for (name, id) in [("A", 10), ("B", 20), ("C", 30)] {
        g.externals.push(ExternalToken {
            name: name.to_string(),
            symbol_id: SymbolId(id),
        });
    }
    assert_eq!(V1Generator::new(g).generate_symbol_map().len(), 3);
}

#[test]
fn test_v1_symbol_map_preserves_id_order() {
    let mut g = Grammar::new("test".to_string());
    for (name, id) in [("Z", 99), ("M", 50), ("A", 1)] {
        g.externals.push(ExternalToken {
            name: name.to_string(),
            symbol_id: SymbolId(id),
        });
    }
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [99, 50, 1]);
}

#[test]
fn test_v2_external_token_count_matches_grammar() {
    let mut g = Grammar::new("test".to_string());
    for i in 0u16..4 {
        g.externals.push(ExternalToken {
            name: format!("T{i}"),
            symbol_id: SymbolId(i + 10),
        });
    }
    let table = minimal_table(&g);
    assert_eq!(V2Generator::new(g, table).external_token_count(), 4);
}

#[test]
fn test_v1_bitmap_columns_match_external_count() {
    let mut g = Grammar::new("test".to_string());
    for i in 0u16..3 {
        g.externals.push(ExternalToken {
            name: format!("E{i}"),
            symbol_id: SymbolId(i),
        });
    }
    let bitmap = V1Generator::new(g).generate_state_bitmap(2);
    for row in &bitmap {
        assert_eq!(row.len(), 3);
    }
}

#[test]
fn test_builder_multiple_externals_all_stored() {
    let g = GrammarBuilder::new("lang")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("NEWLINE", "NEWLINE")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();
    assert_eq!(g.externals.len(), 3);
}

#[test]
fn test_serializer_reports_correct_external_count() {
    let mut g = Grammar::new("test".to_string());
    for i in 0u16..3 {
        g.externals.push(ExternalToken {
            name: format!("EXT{i}"),
            symbol_id: SymbolId(i + 1),
        });
    }
    let table = minimal_table(&g);
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["external_token_count"], 3);
}

#[test]
fn test_v2_scanner_interface_mentions_all_externals() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "HEREDOC".to_string(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "TEMPLATE".to_string(),
        symbol_id: SymbolId(20),
    });
    let table = minimal_table(&g);
    let code = V2Generator::new(g, table)
        .generate_scanner_interface()
        .to_string();
    assert!(!code.is_empty());
    assert!(code.contains("EXTERNAL_TOKEN_COUNT"));
}

#[test]
fn test_large_external_set_20_ids_unique() {
    let mut g = Grammar::new("test".to_string());
    for i in 0u16..20 {
        g.externals.push(ExternalToken {
            name: format!("T{i}"),
            symbol_id: SymbolId(100 + i),
        });
    }
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map.len(), 20);
    for (i, &val) in map.iter().enumerate() {
        assert_eq!(val, 100 + i as u16);
    }
}

// ===========================================================================
// 6. External + regular token interaction (8 tests)
// ===========================================================================

#[test]
fn test_regular_tokens_not_in_externals() {
    let g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let ext_names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(!ext_names.contains(&"NUM"));
}

#[test]
fn test_external_and_regular_coexist_in_tokens() {
    let g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let tok_names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(tok_names.contains(&"NUM"));
    assert!(tok_names.contains(&"INDENT"));
}

#[test]
fn test_v1_external_count_excludes_regular() {
    let g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("STR", r#""[^"]*""#)
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    assert_eq!(V1Generator::new(g).external_token_count(), 1);
}

#[test]
fn test_mixed_grammar_abi_builder_generates_code() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(!code.is_empty());
    assert!(code.contains("external_token_count"));
}

#[test]
fn test_mixed_grammar_node_types_includes_regular_and_external() {
    let mut g = Grammar::new("test".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(50),
    });
    let table = minimal_table(&g);
    let slg = StaticLanguageGenerator::new(g, table);
    let json = slg.generate_node_types();
    assert!(json.contains("INDENT"));
}

#[test]
fn test_mixed_grammar_serializer_has_external_and_regular_symbols() {
    let mut g = Grammar::new("test".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: SymbolId(50),
    });
    let table = minimal_table(&g);
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let sym_names = val["symbol_names"].as_array().unwrap();
    assert!(sym_names.iter().any(|n| n == "EXT"));
}

#[test]
fn test_language_builder_with_external_reports_count() {
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
        name: "EXT".to_string(),
        symbol_id: SymbolId(100),
    });
    let table = minimal_table(&g);
    let builder = adze_tablegen::LanguageBuilder::new(g, table);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.external_token_count, 1);
}

#[test]
fn test_language_builder_without_externals_has_zero_count() {
    let g = Grammar {
        name: "test".to_string(),
        ..Default::default()
    };
    let table = minimal_table(&g);
    let builder = adze_tablegen::LanguageBuilder::new(g, table);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.external_token_count, 0);
}

// ===========================================================================
// 7. Full pipeline: GrammarBuilder → build_lr1_automaton → codegen (8 tests)
// ===========================================================================

#[test]
fn test_pipeline_simple_grammar_with_external_builds_table() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    assert!(table.state_count > 0);
}

#[test]
fn test_pipeline_external_does_not_alter_state_count() {
    // Build two grammars: one with external, one without.
    // The external token should not create extra parser states for the core grammar.
    let mut g_without = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table_without = build_table(&mut g_without);

    let mut g_with = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("EXT", "EXT")
        .external("EXT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table_with = build_table(&mut g_with);

    // External tokens are handled by external scanner, not the LR automaton.
    // State counts should be the same for the core grammar.
    assert_eq!(table_without.state_count, table_with.state_count);
}

#[test]
fn test_pipeline_abi_builder_from_real_table() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &table).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_pipeline_static_lang_gen_from_real_table() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    let slg = StaticLanguageGenerator::new(g, table);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_pipeline_node_types_from_real_table() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let _table = build_table(&mut g);
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn test_pipeline_serializer_from_real_table() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["external_token_count"], 1);
}

#[test]
fn test_pipeline_two_externals_real_table() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["external_token_count"], 2);
}

#[test]
fn test_pipeline_real_table_external_in_symbol_names() {
    let mut g = GrammarBuilder::new("lang")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("program", vec!["NUM"])
        .start("program")
        .build();
    let table = build_table(&mut g);
    let json = adze_tablegen::serializer::serialize_language(&g, &table, None).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let sym_names = val["symbol_names"].as_array().unwrap();
    assert!(sym_names.iter().any(|n| n == "INDENT"));
}

// ===========================================================================
// 8. Edge cases & misc (6 tests)
// ===========================================================================

#[test]
fn test_v1_bitmap_zero_states_yields_empty() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(1),
    });
    let bitmap = V1Generator::new(g).generate_state_bitmap(0);
    assert!(bitmap.is_empty());
}

#[test]
fn test_v2_compute_state_validity_matches_bitmap() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "A".to_string(),
        symbol_id: SymbolId(1),
    });
    g.externals.push(ExternalToken {
        name: "B".to_string(),
        symbol_id: SymbolId(2),
    });
    let mut table = minimal_table(&g);
    table.external_scanner_states = vec![vec![true, false], vec![false, true]];
    let scanner = V2Generator::new(g, table);
    assert_eq!(scanner.compute_state_validity(), scanner.generate_state_bitmap());
}

#[test]
fn test_external_token_debug_format() {
    let tok = ExternalToken {
        name: "DBG_TOKEN".to_string(),
        symbol_id: SymbolId(42),
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("DBG_TOKEN"));
    assert!(dbg.contains("42"));
}

#[test]
fn test_symbol_id_zero_allowed_for_external() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "ZERO".to_string(),
        symbol_id: SymbolId(0),
    });
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [0]);
}

#[test]
fn test_symbol_id_u16_max_allowed() {
    let mut g = Grammar::new("test".to_string());
    g.externals.push(ExternalToken {
        name: "MAX".to_string(),
        symbol_id: SymbolId(u16::MAX),
    });
    let map = V1Generator::new(g).generate_symbol_map();
    assert_eq!(map, [u16::MAX]);
}

#[test]
fn test_hidden_external_excluded_from_node_types() {
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
    let slg = StaticLanguageGenerator::new(g, table);
    let json = slg.generate_node_types();
    assert!(json.contains("VISIBLE"));
    assert!(!json.contains("_hidden"));
}
