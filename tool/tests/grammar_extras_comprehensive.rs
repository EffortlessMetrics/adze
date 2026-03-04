#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for extras handling in grammar generation by adze-tool.
//!
//! Extras are types like whitespace and comments that can appear anywhere
//! in the parsed input. These tests cover: default extras (whitespace),
//! custom extras types, multiple extras, extras in grammar JSON, regex and
//! text pattern extras, grammars without extras, and extras interaction with rules.

use std::fs;
use tempfile::TempDir;

/// Helper: write Rust source to a temp file and extract grammars.
fn extract(src: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

/// Helper: extract exactly one grammar and return it.
fn extract_one(src: &str) -> serde_json::Value {
    let gs = extract(src);
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

/// Helper: return the "extras" array from a grammar value.
fn extras_array(g: &serde_json::Value) -> &Vec<serde_json::Value> {
    g["extras"].as_array().expect("extras should be an array")
}

/// Helper: collect names referenced in an extras array (SYMBOL entries).
fn extras_names(g: &serde_json::Value) -> Vec<String> {
    extras_array(g)
        .iter()
        .filter_map(|e| e["name"].as_str().map(String::from))
        .collect()
}

// ---------------------------------------------------------------------------
// 1. Default extras (whitespace)
// ---------------------------------------------------------------------------

#[test]
fn whitespace_extra_appears_in_extras_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("ws")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Whitespace".to_string()));
}

#[test]
fn whitespace_extra_generates_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("ws_rule")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(
        rules.contains_key("Whitespace"),
        "Whitespace rule should exist"
    );
}

#[test]
fn whitespace_extra_rule_contains_pattern() {
    let g = extract_one(
        r#"
        #[adze::grammar("ws_pat")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    // The extra struct generates a FIELD rule plus a sub-rule with the PATTERN.
    // Serialize all rules to confirm the pattern is present somewhere.
    let all_rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(
        all_rules_str.contains("PATTERN"),
        "rules should contain a PATTERN"
    );
    assert!(
        all_rules_str.contains(r"\s"),
        "rules should contain \\s pattern"
    );
}

#[test]
fn whitespace_extra_is_symbol_ref_in_extras() {
    let g = extract_one(
        r#"
        #[adze::grammar("ws_sym")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let extras = extras_array(&g);
    let ws_entry = extras
        .iter()
        .find(|e| e["name"].as_str() == Some("Whitespace"))
        .expect("Whitespace should be in extras");
    assert_eq!(ws_entry["type"].as_str().unwrap(), "SYMBOL");
}

// ---------------------------------------------------------------------------
// 2. Custom extras type
// ---------------------------------------------------------------------------

#[test]
fn custom_named_extra_appears_in_extras() {
    let g = extract_one(
        r#"
        #[adze::grammar("custom_extra")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct LineComment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _comment: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"LineComment".to_string()));
}

#[test]
fn custom_extra_generates_its_own_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("custom_rule")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct BlockComment {
                #[adze::leaf(pattern = r"/\*[^*]*\*/")]
                _comment: (),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("BlockComment"));
}

// ---------------------------------------------------------------------------
// 3. Multiple extras
// ---------------------------------------------------------------------------

#[test]
fn multiple_extras_all_appear_in_extras_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct LineComment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _comment: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Whitespace".to_string()));
    assert!(names.contains(&"LineComment".to_string()));
}

#[test]
fn multiple_extras_count_matches() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_count")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r";;.*")]
                _c: (),
            }

            #[adze::extra]
            struct Newline {
                #[adze::leaf(pattern = r"\r")]
                _nl: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert_eq!(names.len(), 3, "should have exactly 3 extras");
}

#[test]
fn multiple_extras_all_generate_rules() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_rules")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r";;.*")]
                _c: (),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Ws"));
    assert!(rules.contains_key("Comment"));
}

// ---------------------------------------------------------------------------
// 4. Extras in grammar JSON (extras array structure)
// ---------------------------------------------------------------------------

#[test]
fn extras_key_exists_in_grammar_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("extras_key")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    assert!(
        g.get("extras").is_some(),
        "grammar JSON should have 'extras' key"
    );
    assert!(g["extras"].is_array(), "'extras' should be an array");
}

#[test]
fn extras_entries_have_type_and_name() {
    let g = extract_one(
        r#"
        #[adze::grammar("entry_shape")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    for entry in extras_array(&g) {
        assert!(
            entry.get("type").is_some(),
            "each extras entry should have 'type'"
        );
        assert!(
            entry.get("name").is_some(),
            "each extras entry should have 'name'"
        );
    }
}

#[test]
fn extras_entries_are_symbol_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("sym_type")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    for entry in extras_array(&g) {
        assert_eq!(
            entry["type"].as_str().unwrap(),
            "SYMBOL",
            "extras entries should be SYMBOL references"
        );
    }
}

#[test]
fn extras_json_roundtrips_through_serde() {
    let g = extract_one(
        r#"
        #[adze::grammar("roundtrip")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(g["extras"], parsed["extras"]);
}

// ---------------------------------------------------------------------------
// 5. Extras as regex pattern
// ---------------------------------------------------------------------------

#[test]
fn regex_pattern_extra_preserved_in_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("regex_extra")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s+")]
                _ws: (),
            }
        }
        "#,
    );
    // The pattern lives in the sub-rule (e.g. Ws__ws); check all rules.
    let all_rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(
        all_rules_str.contains(r"\s+"),
        "regex pattern should be preserved in the rules"
    );
}

#[test]
fn complex_regex_pattern_extra() {
    let g = extract_one(
        r#"
        #[adze::grammar("complex_regex")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct MultiLineComment {
                #[adze::leaf(pattern = r"/\*.*\*/")]
                _comment: (),
            }
        }
        "#,
    );
    // PATTERN lives in sub-rule; check all rules.
    let all_rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(
        all_rules_str.contains("PATTERN"),
        "complex regex extra should produce a PATTERN rule"
    );
}

#[test]
fn tab_regex_pattern_extra() {
    let g = extract_one(
        r#"
        #[adze::grammar("tab_regex")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Tab {
                #[adze::leaf(pattern = r"\t")]
                _tab: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Tab".to_string()));
    let all_rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(all_rules_str.contains(r"\t"));
}

// ---------------------------------------------------------------------------
// 6. Extras with text pattern
// ---------------------------------------------------------------------------

#[test]
fn text_pattern_extra_preserved_in_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("text_extra")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Semicolon {
                #[adze::leaf(text = ";")]
                _semi: (),
            }
        }
        "#,
    );
    // STRING lives in the sub-rule (e.g. Semicolon__semi); check all rules.
    let all_rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(
        all_rules_str.contains("STRING"),
        "text extra should produce a STRING rule"
    );
    assert!(
        all_rules_str.contains(";"),
        "text value should be preserved"
    );
}

#[test]
fn text_extra_appears_in_extras_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("text_extra_arr")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Comma {
                #[adze::leaf(text = ",")]
                _comma: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Comma".to_string()));
}

// ---------------------------------------------------------------------------
// 7. Grammar without extras
// ---------------------------------------------------------------------------

#[test]
fn no_extras_produces_empty_extras_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    let extras = extras_array(&g);
    assert!(
        extras.is_empty(),
        "grammar without #[adze::extra] should have empty extras"
    );
}

#[test]
fn no_extras_grammar_still_has_extras_key() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_extras_key")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    assert!(
        g.get("extras").is_some(),
        "even without extras the key should exist"
    );
}

#[test]
fn no_extras_grammar_rules_unaffected() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_extras_rules")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("source_file"));
    assert!(rules.contains_key("Root"));
    // No extras-related rule names should be present
    let rule_names: Vec<&String> = rules.keys().collect();
    assert!(
        !rule_names
            .iter()
            .any(|n| n.contains("Whitespace") || n.contains("Comment")),
        "no extras rules should leak in"
    );
}

// ---------------------------------------------------------------------------
// 8. Extras interaction with rules
// ---------------------------------------------------------------------------

#[test]
fn extras_do_not_appear_as_regular_rule_references() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_ref")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    // source_file should reference Root, not Ws
    let source_file = &g["rules"]["source_file"];
    let sf_str = serde_json::to_string(source_file).unwrap();
    assert!(
        !sf_str.contains("\"Ws\""),
        "source_file should not reference the extra"
    );
}

#[test]
fn extras_coexist_with_enum_language() {
    let g = extract_one(
        r#"
        #[adze::grammar("enum_extras")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    i32
                ),
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expr"));
    assert!(rules.contains_key("Ws"));
    let names = extras_names(&g);
    assert!(names.contains(&"Ws".to_string()));
}

#[test]
fn extras_coexist_with_struct_fields() {
    let g = extract_one(
        r#"
        #[adze::grammar("struct_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                age: i32,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Root"));
    assert!(rules.contains_key("Ws"));
    let names = extras_names(&g);
    assert_eq!(names.len(), 1);
}

#[test]
fn extras_coexist_with_repeat_rules() {
    let g = extract_one(
        r#"
        #[adze::grammar("repeat_extras")]
        mod grammar {
            #[adze::language]
            pub struct NumberList {
                #[adze::repeat(non_empty = true)]
                numbers: Vec<Number>,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: i32,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Ws".to_string()));
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("NumberList"));
    assert!(rules.contains_key("Number"));
}

#[test]
fn extras_coexist_with_optional_fields() {
    let g = extract_one(
        r#"
        #[adze::grammar("opt_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                age: Option<i32>,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Ws".to_string()));
    // The Root rule should still work with its optional field
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Root"));
}

#[test]
fn extras_coexist_with_precedence_rules() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_extras")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(
                    Box<Expression>,
                    #[adze::leaf(text = "+", transform = |v| ())]
                    (),
                    Box<Expression>
                ),
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Ws".to_string()));
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expression"));
}

#[test]
fn extras_with_delimited_repeat() {
    let g = extract_one(
        r#"
        #[adze::grammar("delim_extras")]
        mod grammar {
            #[adze::language]
            pub struct NumberList {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                numbers: Vec<Number>,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: i32,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(names.contains(&"Ws".to_string()));
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("NumberList"));
}

#[test]
fn non_extra_struct_not_in_extras_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("non_extra")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Child,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert!(
        !names.contains(&"Child".to_string()),
        "non-extra struct should not be in extras"
    );
    assert!(
        !names.contains(&"Root".to_string()),
        "language root should not be in extras"
    );
    assert!(names.contains(&"Ws".to_string()));
}

#[test]
fn extras_order_matches_declaration_order() {
    let g = extract_one(
        r#"
        #[adze::grammar("order")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Alpha {
                #[adze::leaf(pattern = r"a")]
                _a: (),
            }

            #[adze::extra]
            struct Beta {
                #[adze::leaf(pattern = r"b")]
                _b: (),
            }

            #[adze::extra]
            struct Gamma {
                #[adze::leaf(pattern = r"c")]
                _g: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert_eq!(names, vec!["Alpha", "Beta", "Gamma"]);
}

#[test]
fn extra_does_not_affect_grammar_name() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_grammar")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "my_grammar");
}

#[test]
fn extras_do_not_affect_word_field() {
    let g = extract_one(
        r#"
        #[adze::grammar("word_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    // word should be null when no #[adze::word] is present
    assert!(
        g["word"].is_null(),
        "word field should not be set by extras"
    );
}

#[test]
fn mixed_regex_and_text_extras() {
    let g = extract_one(
        r#"
        #[adze::grammar("mixed_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct Semi {
                #[adze::leaf(text = ";")]
                _semi: (),
            }
        }
        "#,
    );
    let names = extras_names(&g);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Ws".to_string()));
    assert!(names.contains(&"Semi".to_string()));

    // Verify the different rule types via the full rules serialization
    let all_rules_str = serde_json::to_string(&g["rules"]).unwrap();
    assert!(all_rules_str.contains("PATTERN"));
    assert!(all_rules_str.contains("STRING"));
}
