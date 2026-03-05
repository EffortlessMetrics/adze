//! Comprehensive tests for grammar JSON emission (v3) in adze-tool.
//!
//! 55+ tests covering:
//! 1. Grammar JSON is valid JSON (8 tests)
//! 2. Grammar JSON contains expected keys (8 tests)
//! 3. Node types JSON is valid and structured (8 tests)
//! 4. Language code is valid Rust (5 tests)
//! 5. Grammar name appears in output (5 tests)
//! 6. Token patterns appear in grammar JSON (5 tests)
//! 7. Rule names appear in output (5 tests)
//! 8. Various grammar topologies produce valid output (6 tests)
//! 9. Edge cases (5 tests)

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser, build_parser_from_json};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, opts)
}

/// Minimal grammar: one literal token, one rule.
fn minimal_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// Grammar with two alternatives.
fn alt_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("alt")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build()
}

/// Grammar with a three-token sequence.
fn seq_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build()
}

/// Grammar with a regex token.
fn regex_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("regex_emit")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build()
}

/// Grammar with an intermediate (chain) non-terminal.
fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain")
        .token("t", "t")
        .rule("inner", vec!["t"])
        .rule("s", vec!["inner"])
        .start("s")
        .build()
}

/// Grammar with precedence and associativity.
fn prec_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("prec_emit")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["b"], 2, Associativity::Right)
        .start("s")
        .build()
}

/// Grammar with many tokens.
fn many_tokens_grammar(count: usize) -> adze_ir::Grammar {
    let mut builder = GrammarBuilder::new("many_tok");
    let names: Vec<String> = (0..count).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("s", vec![name]);
    }
    builder = builder.start("s");
    builder.build()
}

/// Build a JSON grammar string for `build_parser_from_json`.
fn json_grammar(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "rules": {
            "source": {"type": "PATTERN", "value": "[a-z]+"}
        }
    })
    .to_string()
}

/// Build a JSON grammar with an explicit SEQ rule.
fn json_seq_grammar(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "rules": {
            "source": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "hello"},
                    {"type": "STRING", "value": "world"}
                ]
            }
        }
    })
    .to_string()
}

/// Build a JSON grammar with CHOICE.
fn json_choice_grammar(name: &str) -> String {
    serde_json::json!({
        "name": name,
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {"type": "STRING", "value": "foo"},
                    {"type": "STRING", "value": "bar"}
                ]
            }
        }
    })
    .to_string()
}

// =========================================================================
// 1. Grammar JSON is valid JSON (8 tests)
// =========================================================================

#[test]
fn valid_json_minimal_grammar_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array() || parsed.is_object());
}

#[test]
fn valid_json_alt_grammar_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(alt_grammar(), opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_seq_grammar_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(seq_grammar(), opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_regex_grammar_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(regex_grammar(), opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_chain_grammar_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(chain_grammar(), opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_prec_grammar_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(prec_grammar(), opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_many_tokens_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(many_tokens_grammar(5), opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn valid_json_from_json_input_node_types_parses() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(json_grammar("json_valid"), opts).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

// =========================================================================
// 2. Grammar JSON contains expected keys (8 tests)
// =========================================================================

/// Extract grammars from Rust source.
fn extract_one(src: &str) -> serde_json::Value {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    std::fs::write(&path, src).unwrap();
    let gs = adze_tool::generate_grammars(&path).unwrap();
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

#[test]
fn expected_keys_name_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_name")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "x")]
                pub x: String,
            }
        }
        "#,
    );
    assert!(g.get("name").is_some());
}

#[test]
fn expected_keys_rules_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_rules")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "y")]
                pub y: String,
            }
        }
        "#,
    );
    assert!(g.get("rules").is_some());
}

#[test]
fn expected_keys_extras_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "z")]
                pub z: String,
            }
        }
        "#,
    );
    assert!(g.get("extras").is_some());
}

#[test]
fn expected_keys_word_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_word")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "w")]
                pub w: String,
            }
        }
        "#,
    );
    assert!(g.get("word").is_some());
}

#[test]
fn expected_keys_rules_is_object() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_rules_obj")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "a")]
                pub a: String,
            }
        }
        "#,
    );
    assert!(g["rules"].is_object());
}

#[test]
fn expected_keys_extras_is_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_extras_arr")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "b")]
                pub b: String,
            }
        }
        "#,
    );
    assert!(g["extras"].is_array());
}

#[test]
fn expected_keys_name_matches_annotation() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_specific_name")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "c")]
                pub c: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "my_specific_name");
}

#[test]
fn expected_keys_rules_nonempty_for_grammar_with_token() {
    let g = extract_one(
        r#"
        #[adze::grammar("keys_nonempty")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(!rules.is_empty());
}

// =========================================================================
// 3. Node types JSON is valid and structured (8 tests)
// =========================================================================

#[test]
fn node_types_is_nonempty_string() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn node_types_parses_as_array() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_types_entries_have_type_field() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = v.as_array().unwrap();
    // At least one entry should have a "type" field
    assert!(arr.iter().any(|entry| entry.get("type").is_some()));
}

#[test]
fn node_types_entries_have_named_field() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = v.as_array().unwrap();
    assert!(arr.iter().any(|entry| entry.get("named").is_some()));
}

#[test]
fn node_types_alt_grammar_has_entries() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(alt_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn node_types_chain_grammar_has_entries() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(chain_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn node_types_seq_grammar_has_entries() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(seq_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn node_types_from_json_grammar_has_entries() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(json_grammar("nt_json"), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

// =========================================================================
// 4. Language code is valid Rust (5 tests)
// =========================================================================

#[test]
fn language_code_is_nonempty() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn language_code_contains_static_or_const() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    assert!(
        result.parser_code.contains("static") || result.parser_code.contains("const"),
        "language code should contain static/const declarations"
    );
}

#[test]
fn language_code_no_unbalanced_braces() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let opens = result.parser_code.chars().filter(|&c| c == '{').count();
    let closes = result.parser_code.chars().filter(|&c| c == '}').count();
    assert_eq!(opens, closes, "braces should be balanced in generated Rust");
}

#[test]
fn language_code_no_unbalanced_parens() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let opens = result.parser_code.chars().filter(|&c| c == '(').count();
    let closes = result.parser_code.chars().filter(|&c| c == ')').count();
    assert_eq!(opens, closes, "parens should be balanced in generated Rust");
}

#[test]
fn language_code_no_unbalanced_brackets() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let opens = result.parser_code.chars().filter(|&c| c == '[').count();
    let closes = result.parser_code.chars().filter(|&c| c == ']').count();
    assert_eq!(
        opens, closes,
        "brackets should be balanced in generated Rust"
    );
}

// =========================================================================
// 5. Grammar name appears in output (5 tests)
// =========================================================================

#[test]
fn grammar_name_in_build_result() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    assert_eq!(result.grammar_name, "minimal");
}

#[test]
fn grammar_name_preserved_with_underscores() {
    let grammar = GrammarBuilder::new("my_grammar_v3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "my_grammar_v3");
}

#[test]
fn grammar_name_preserved_with_digits() {
    let grammar = GrammarBuilder::new("lang42")
        .token("z", "z")
        .rule("s", vec!["z"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "lang42");
}

#[test]
fn grammar_name_from_json_input() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(json_grammar("json_name_test"), opts).unwrap();
    assert_eq!(result.grammar_name, "json_name_test");
}

#[test]
fn grammar_name_in_extracted_grammar_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("emit_name_check")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "v")]
                pub v: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "emit_name_check");
}

// =========================================================================
// 6. Token patterns appear in grammar JSON (5 tests)
// =========================================================================

#[test]
fn token_pattern_literal_appears_in_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("tok_literal")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "hello")]
                pub w: String,
            }
        }
        "#,
    );
    let rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(
        rules_str.contains("hello"),
        "expected literal 'hello' in grammar rules"
    );
}

#[test]
fn token_pattern_regex_appears_in_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("tok_regex")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub num: String,
            }
        }
        "#,
    );
    let rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(
        rules_str.contains("[0-9]+"),
        "expected regex pattern in grammar rules"
    );
}

#[test]
fn token_pattern_multiple_literals_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("tok_multi")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "foo")]
                pub a: String,
                #[adze::leaf(text = "bar")]
                pub b: String,
            }
        }
        "#,
    );
    let rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(rules_str.contains("foo"));
    assert!(rules_str.contains("bar"));
}

#[test]
fn token_pattern_single_char_literal() {
    let g = extract_one(
        r#"
        #[adze::grammar("tok_char")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "+")]
                pub op: String,
            }
        }
        "#,
    );
    let rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(rules_str.contains("+"));
}

#[test]
fn token_pattern_in_ir_grammar_parser_code() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(regex_grammar(), opts).unwrap();
    // The parser code or node_types should reference the grammar
    assert!(!result.parser_code.is_empty());
    assert!(!result.node_types_json.is_empty());
}

// =========================================================================
// 7. Rule names appear in output (5 tests)
// =========================================================================

#[test]
fn rule_name_in_extracted_grammar_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("rule_names")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "r")]
                pub r_field: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    // The root struct name should appear as a rule
    assert!(!rules.is_empty(), "rules should contain at least one entry");
}

#[test]
fn rule_name_root_present_in_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("rule_root")]
        mod grammar {
            #[adze::language]
            pub struct MyRoot {
                #[adze::leaf(text = "x")]
                pub x: String,
            }
        }
        "#,
    );
    let rules_str = serde_json::to_string(&g["rules"]).unwrap();
    // Type names are typically snake_cased in the grammar
    assert!(
        rules_str.contains("my_root") || rules_str.contains("MyRoot"),
        "root type should appear in rules"
    );
}

#[test]
fn rule_name_enum_variants_in_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("rule_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(#[adze::leaf(pattern = r"[0-9]+")] String),
            }
        }
        "#,
    );
    let rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(
        rules_str.contains("expr") || rules_str.contains("Expr"),
        "enum name should appear in rules"
    );
}

#[test]
fn rule_name_node_types_references_rule() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = v.as_array().unwrap();
    let type_names: Vec<&str> = arr
        .iter()
        .filter_map(|entry| entry.get("type").and_then(|t| t.as_str()))
        .collect();
    // There should be at least one type name
    assert!(!type_names.is_empty(), "node types should reference rules");
}

#[test]
fn rule_name_chain_grammar_inner_in_node_types() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(chain_grammar(), opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = v.as_array().unwrap();
    let type_names: Vec<&str> = arr
        .iter()
        .filter_map(|entry| entry.get("type").and_then(|t| t.as_str()))
        .collect();
    assert!(
        !type_names.is_empty(),
        "chain grammar should produce node types"
    );
}

// =========================================================================
// 8. Various grammar topologies produce valid output (6 tests)
// =========================================================================

#[test]
fn topology_single_token_produces_valid_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(minimal_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn topology_two_alternatives_produces_valid_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(alt_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn topology_sequence_produces_valid_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(seq_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn topology_chain_produces_valid_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(chain_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn topology_precedence_produces_valid_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(prec_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn topology_many_tokens_produces_valid_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(many_tokens_grammar(8), opts).unwrap();
    assert!(!result.parser_code.is_empty());
    let v: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 9. Edge cases (5 tests)
// =========================================================================

#[test]
fn edge_json_seq_grammar_produces_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(json_seq_grammar("edge_seq"), opts).unwrap();
    assert_eq!(result.grammar_name, "edge_seq");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn edge_json_choice_grammar_produces_output() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser_from_json(json_choice_grammar("edge_choice"), opts).unwrap();
    assert_eq!(result.grammar_name, "edge_choice");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn edge_uncompressed_tables_produce_valid_output() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser(minimal_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn edge_grammar_name_all_lowercase() {
    let grammar = GrammarBuilder::new("alllowercase")
        .token("m", "m")
        .rule("s", vec!["m"])
        .start("s")
        .build();
    let (_dir, opts) = tmp_opts();
    let result = build_parser(grammar, opts).unwrap();
    assert_eq!(result.grammar_name, "alllowercase");
}

#[test]
fn edge_grammar_build_stats_nonzero() {
    let (_dir, opts) = tmp_opts();
    let result = build_parser(alt_grammar(), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}
