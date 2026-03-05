//! Comprehensive v3 tests for external scanner support in adze-tablegen.
//!
//! Covers: grammar construction with externals, external token properties,
//! code generation, multiple externals, serialization, mixed grammars,
//! trait implementations, and edge cases.

mod test_helpers;

use adze_ir::builder::GrammarBuilder;
use adze_ir::{ExternalToken, Grammar, SymbolId};
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

// ===========================================================================
// 1. Grammar with external tokens builds (8 tests)
// ===========================================================================

#[test]
fn build_grammar_with_single_external() {
    let g = GrammarBuilder::new("lang")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn build_grammar_with_two_externals() {
    let g = GrammarBuilder::new("lang")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .token("ID", r"[a-z]+")
        .rule("stmt", vec!["ID"])
        .start("stmt")
        .build();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn builder_external_assigns_symbol_id() {
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
fn builder_external_shares_token_symbol_id() {
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
fn v1_from_grammar_with_externals() {
    let g = grammar_with_externals(&[("A", 10)]);
    let scanner = V1Generator::new(g);
    assert!(scanner.has_external_tokens());
}

#[test]
fn v1_empty_grammar_no_externals() {
    let scanner = V1Generator::new(empty_grammar());
    assert!(!scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 0);
}

#[test]
fn v2_from_grammar_with_externals() {
    let g = grammar_with_externals(&[("B", 20)]);
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    assert!(scanner.has_external_tokens());
}

#[test]
fn v2_empty_grammar_no_externals() {
    let g = empty_grammar();
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    assert!(!scanner.has_external_tokens());
}

// ===========================================================================
// 2. External token properties (8 tests)
// ===========================================================================

#[test]
fn external_token_name_preserved() {
    let tok = ExternalToken {
        name: "MY_TOKEN".to_string(),
        symbol_id: SymbolId(42),
    };
    assert_eq!(tok.name, "MY_TOKEN");
}

#[test]
fn external_token_symbol_id_preserved() {
    let tok = ExternalToken {
        name: "X".to_string(),
        symbol_id: SymbolId(999),
    };
    assert_eq!(tok.symbol_id, SymbolId(999));
}

#[test]
fn v1_token_count_matches_externals() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2), ("C", 3)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.external_token_count(), 3);
}

#[test]
fn v1_symbol_map_ids_match_declaration_order() {
    let g = grammar_with_externals(&[("FIRST", 50), ("SECOND", 60), ("THIRD", 70)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.generate_symbol_map(), [50, 60, 70]);
}

#[test]
fn v2_token_count_matches_externals() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2)]);
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    assert_eq!(scanner.external_token_count(), 2);
}

#[test]
fn v2_symbol_map_ids_match_declaration_order() {
    let g = grammar_with_externals(&[("X", 100), ("Y", 200)]);
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    assert_eq!(scanner.generate_symbol_map(), [100, 200]);
}

#[test]
fn grammar_externals_field_is_vec() {
    let mut g = empty_grammar();
    assert!(g.externals.is_empty());
    g.externals.push(ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(1),
    });
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn external_token_symbol_id_is_copy() {
    let tok = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(5),
    };
    let id = tok.symbol_id;
    // SymbolId is Copy — using both is fine without clone
    assert_eq!(id, tok.symbol_id);
}

// ===========================================================================
// 3. External token in code generation (8 tests)
// ===========================================================================

#[test]
fn v1_empty_grammar_emits_no_code() {
    let scanner = V1Generator::new(empty_grammar());
    let code = scanner.generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn v1_non_empty_grammar_emits_code() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(!code.is_empty());
}

#[test]
fn v1_generated_code_contains_scanner_states() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_STATES"));
}

#[test]
fn v1_generated_code_contains_symbol_map() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_SYMBOL_MAP"));
}

#[test]
fn v1_generated_code_contains_scanner_data() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_DATA"));
}

#[test]
fn v2_empty_grammar_emits_no_code() {
    let g = empty_grammar();
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    let code = scanner.generate_scanner_interface();
    assert!(code.is_empty());
}

#[test]
fn v2_non_empty_grammar_emits_code() {
    let g = grammar_with_externals(&[("TOK", 10)]);
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(!code.is_empty());
}

#[test]
fn v2_generated_code_contains_token_count_constant() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2)]);
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_TOKEN_COUNT"));
}

// ===========================================================================
// 4. Multiple external tokens (5 tests)
// ===========================================================================

#[test]
fn three_external_tokens_all_in_symbol_map() {
    let g = grammar_with_externals(&[("A", 10), ("B", 20), ("C", 30)]);
    let scanner = V1Generator::new(g);
    let map = scanner.generate_symbol_map();
    assert_eq!(map.len(), 3);
    assert_eq!(map, [10, 20, 30]);
}

#[test]
fn five_tokens_v2_count() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2), ("C", 3), ("D", 4), ("E", 5)]);
    let table = make_v2_table(&g);
    let scanner = V2Generator::new(g, table);
    assert_eq!(scanner.external_token_count(), 5);
}

#[test]
fn multiple_externals_bitmap_rows_match_states() {
    let g = grammar_with_externals(&[("X", 10), ("Y", 20)]);
    let scanner = V1Generator::new(g);
    let bitmap = scanner.generate_state_bitmap(7);
    assert_eq!(bitmap.len(), 7);
    for row in &bitmap {
        assert_eq!(row.len(), 2);
    }
}

#[test]
fn multiple_externals_v2_state_bitmap_from_table() {
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
fn multiple_externals_builder_preserves_order() {
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
// 5. External token serialization (5 tests)
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
fn grammar_with_externals_roundtrip_json() {
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
    // externals should appear as []
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    let externals = val.get("externals").unwrap();
    assert!(externals.is_array());
    assert_eq!(externals.as_array().unwrap().len(), 0);
}

// ===========================================================================
// 6. Grammar with both regular and external tokens (8 tests)
// ===========================================================================

#[test]
fn python_like_grammar_has_externals() {
    let g = GrammarBuilder::python_like();
    assert!(!g.externals.is_empty());
}

#[test]
fn python_like_grammar_external_names() {
    let g = GrammarBuilder::python_like();
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"INDENT"));
    assert!(names.contains(&"DEDENT"));
}

#[test]
fn python_like_grammar_externals_have_matching_tokens() {
    let g = GrammarBuilder::python_like();
    for ext in &g.externals {
        assert!(
            g.tokens.contains_key(&ext.symbol_id),
            "External token {} should also be in tokens map",
            ext.name
        );
    }
}

#[test]
fn javascript_like_grammar_has_no_externals() {
    let g = GrammarBuilder::javascript_like();
    assert!(g.externals.is_empty());
}

#[test]
fn mixed_grammar_v1_counts_only_externals() {
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
fn mixed_grammar_regular_tokens_not_in_symbol_map() {
    let g = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let scanner = V1Generator::new(g.clone());
    let map = scanner.generate_symbol_map();
    // Only the external token symbol should be in the map
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
    let scanner = V2Generator::new(g, table);
    assert!(scanner.has_external_tokens());
}

#[test]
fn mixed_grammar_builder_tokens_and_externals_separate() {
    let g = GrammarBuilder::new("mixed")
        .token("A", "a")
        .token("B", "b")
        .token("EXT", "EXT")
        .external("EXT")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    // 3 tokens, 1 external
    assert_eq!(g.tokens.len(), 3);
    assert_eq!(g.externals.len(), 1);
}

// ===========================================================================
// 7. ExternalToken Debug/Clone/PartialEq (5 tests)
// ===========================================================================

#[test]
fn external_token_debug_format() {
    let tok = ExternalToken {
        name: "DBG".to_string(),
        symbol_id: SymbolId(1),
    };
    let dbg = format!("{:?}", tok);
    assert!(dbg.contains("DBG"));
    assert!(dbg.contains("ExternalToken"));
}

#[test]
fn external_token_clone() {
    let tok = ExternalToken {
        name: "ORIG".to_string(),
        symbol_id: SymbolId(5),
    };
    let cloned = tok.clone();
    assert_eq!(tok, cloned);
}

#[test]
fn external_token_partial_eq_same() {
    let a = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(1),
    };
    let b = ExternalToken {
        name: "T".to_string(),
        symbol_id: SymbolId(1),
    };
    assert_eq!(a, b);
}

#[test]
fn external_token_partial_eq_different_name() {
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
fn external_token_partial_eq_different_id() {
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

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn symbol_id_zero_as_external() {
    let g = grammar_with_externals(&[("ZERO", 0)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.generate_symbol_map(), [0]);
}

#[test]
fn symbol_id_max_as_external() {
    let g = grammar_with_externals(&[("MAX", u16::MAX)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.generate_symbol_map(), [u16::MAX]);
}

#[test]
fn v1_bitmap_zero_states() {
    let g = grammar_with_externals(&[("T", 1)]);
    let scanner = V1Generator::new(g);
    let bitmap = scanner.generate_state_bitmap(0);
    assert!(bitmap.is_empty());
}

#[test]
fn v1_bitmap_single_state_single_token() {
    let g = grammar_with_externals(&[("T", 1)]);
    let scanner = V1Generator::new(g);
    let bitmap = scanner.generate_state_bitmap(1);
    assert_eq!(bitmap.len(), 1);
    assert_eq!(bitmap[0], [true]);
}

#[test]
fn duplicate_external_names_allowed_in_grammar() {
    let g = grammar_with_externals(&[("DUP", 10), ("DUP", 20)]);
    let scanner = V1Generator::new(g);
    // Both appear — no dedup
    assert_eq!(scanner.external_token_count(), 2);
    assert_eq!(scanner.generate_symbol_map(), [10, 20]);
}

#[test]
fn external_token_empty_name() {
    let g = grammar_with_externals(&[("", 1)]);
    let scanner = V1Generator::new(g);
    assert!(scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 1);
}

#[test]
fn v2_compute_state_validity_matches_bitmap() {
    let g = grammar_with_externals(&[("A", 1), ("B", 2)]);
    let mut table = make_v2_table(&g);
    table.external_scanner_states = vec![vec![true, false], vec![false, true]];
    let scanner = V2Generator::new(g, table);
    let validity = scanner.compute_state_validity();
    let bitmap = scanner.generate_state_bitmap();
    assert_eq!(validity, bitmap);
}

#[test]
fn large_external_token_set() {
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
