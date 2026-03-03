#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for grammar metadata extraction in adze-tool.
//!
//! Covers: grammar name, word rules, extras, externals, conflict declarations,
//! combined metadata, empty metadata fields, JSON output structure, and validation.

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

/// Helper: attempt extraction expecting an error.
fn extract_err(src: &str) -> String {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path)
        .expect_err("expected error")
        .to_string()
}

// ---------------------------------------------------------------------------
// 1. Grammar name extraction
// ---------------------------------------------------------------------------

#[test]
fn grammar_name_extraction() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "my_lang");
    let obj = g.as_object().unwrap();
    assert!(obj.contains_key("name"), "JSON must have 'name' key");
}

#[test]
fn grammar_name_with_underscores() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_complex_language_v2")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub num: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "my_complex_language_v2");
}

// ---------------------------------------------------------------------------
// 2. Word rule identification
// ---------------------------------------------------------------------------

#[test]
fn word_rule_on_struct() {
    let g = extract_one(
        r#"
        #[adze::grammar("word_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                pub kw: Keyword,
            }

            #[adze::word]
            pub struct Keyword {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                pub value: String,
            }
        }
        "#,
    );
    assert_eq!(g["word"].as_str().unwrap(), "Keyword");
}

#[test]
fn word_rule_on_leaf_field() {
    let g = extract_one(
        r#"
        #[adze::grammar("word_field")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::word]
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                pub identifier: String,
            }
        }
        "#,
    );
    // Field-level word path is prefixed with parent struct name
    assert_eq!(g["word"].as_str().unwrap(), "Root_identifier");
}

#[test]
fn no_word_rule_yields_null() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_word")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub num: String,
            }
        }
        "#,
    );
    assert!(g["word"].is_null(), "word should be null when not set");
}

#[test]
fn multiple_word_rules_rejected() {
    let err = extract_err(
        r#"
        #[adze::grammar("double_word")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                pub w1: WordA,
            }

            #[adze::word]
            pub struct WordA {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }

            #[adze::word]
            pub struct WordB {
                #[adze::leaf(pattern = r"[A-Z]+")]
                pub val: String,
            }
        }
        "#,
    );
    // Should contain an indication that multiple word rules are not allowed
    let err_lower = err.to_lowercase();
    assert!(
        err_lower.contains("word") || err_lower.contains("multiple"),
        "error should mention word rule conflict: {err}"
    );
}

// ---------------------------------------------------------------------------
// 3. Extras collection
// ---------------------------------------------------------------------------

#[test]
fn single_extra_collected() {
    let g = extract_one(
        r#"
        #[adze::grammar("extras_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                pub ws: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    assert!(!extras.is_empty(), "extras should not be empty");
    let names: Vec<&str> = extras
        .iter()
        .filter_map(|e| e["name"].as_str())
        .collect();
    assert!(names.contains(&"Whitespace"), "extras should include Whitespace");
}

#[test]
fn multiple_extras_collected() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                pub ws: String,
            }

            #[adze::extra]
            pub struct LineComment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                pub comment: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    let names: Vec<&str> = extras
        .iter()
        .filter_map(|e| e["name"].as_str())
        .collect();
    assert!(names.contains(&"Whitespace"));
    assert!(names.contains(&"LineComment"));
}

#[test]
fn no_extras_yields_empty_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    assert!(extras.is_empty(), "extras should be empty when none declared");
}

#[test]
fn extra_has_symbol_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("extra_type")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::extra]
            pub struct Space {
                #[adze::leaf(pattern = r" ")]
                pub s: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    for extra in extras {
        assert_eq!(extra["type"].as_str().unwrap(), "SYMBOL");
    }
}

// ---------------------------------------------------------------------------
// 4. External token listing
// ---------------------------------------------------------------------------

#[test]
fn single_external_token() {
    let g = extract_one(
        r#"
        #[adze::grammar("ext_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::external]
            pub struct HeredocContent {
                #[adze::leaf(pattern = r".*")]
                pub content: String,
            }
        }
        "#,
    );
    let externals = g["externals"].as_array().unwrap();
    let names: Vec<&str> = externals
        .iter()
        .filter_map(|e| e["name"].as_str())
        .collect();
    assert!(names.contains(&"HeredocContent"));
}

#[test]
fn multiple_external_tokens() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_ext")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::external]
            pub struct Indent {
                #[adze::leaf(pattern = r"\t+")]
                pub indent: String,
            }

            #[adze::external]
            pub struct Dedent {
                #[adze::leaf(pattern = r"")]
                pub dedent: String,
            }
        }
        "#,
    );
    let externals = g["externals"].as_array().unwrap();
    let names: Vec<&str> = externals
        .iter()
        .filter_map(|e| e["name"].as_str())
        .collect();
    assert!(names.contains(&"Indent"));
    assert!(names.contains(&"Dedent"));
    assert_eq!(externals.len(), 2);
}

#[test]
fn no_externals_omits_key() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_ext")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }
        }
        "#,
    );
    // externals key is only inserted when non-empty
    assert!(
        g.get("externals").is_none() || g["externals"].as_array().unwrap().is_empty(),
        "externals should be absent or empty when none declared"
    );
}

#[test]
fn external_tokens_also_added_to_extras() {
    let g = extract_one(
        r#"
        #[adze::grammar("ext_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::external]
            pub struct Newline {
                #[adze::leaf(pattern = r"\n")]
                pub nl: String,
            }
        }
        "#,
    );
    // External tokens are added to extras list as well
    let extras = g["extras"].as_array().unwrap();
    let extra_names: Vec<&str> = extras
        .iter()
        .filter_map(|e| e["name"].as_str())
        .collect();
    assert!(
        extra_names.contains(&"Newline"),
        "external tokens should also appear in extras"
    );
}

#[test]
fn external_has_symbol_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("ext_sym")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::external]
            pub struct Scanner {
                #[adze::leaf(pattern = r".")]
                pub s: String,
            }
        }
        "#,
    );
    let externals = g["externals"].as_array().unwrap();
    for ext in externals {
        assert_eq!(ext["type"].as_str().unwrap(), "SYMBOL");
    }
}

// ---------------------------------------------------------------------------
// 5. Conflict declarations (via precedence attributes)
// ---------------------------------------------------------------------------

#[test]
fn precedence_left_generates_prec_left() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_lang")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }
        "#,
    );
    // The grammar should have rules generated with PREC_LEFT
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expr"));
    let json_str = serde_json::to_string(&g).unwrap();
    assert!(
        json_str.contains("PREC_LEFT"),
        "should contain PREC_LEFT in generated rules"
    );
}

#[test]
fn precedence_right_generates_prec_right() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_r")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(2)]
                Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    assert!(
        json_str.contains("PREC_RIGHT"),
        "should contain PREC_RIGHT in generated rules"
    );
}

#[test]
fn multiple_precedence_levels() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_prec")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    // Both precedence levels should appear
    assert!(json_str.contains("PREC_LEFT"));
    // Check rules exist
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expr"));
}

// ---------------------------------------------------------------------------
// 6. Grammar with all metadata fields
// ---------------------------------------------------------------------------

#[test]
fn all_metadata_fields_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("full_meta")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                pub ident: Identifier,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                pub name: String,
            }

            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                pub ws: String,
            }

            #[adze::external]
            pub struct StringContent {
                #[adze::leaf(pattern = r"[^.]+")]
                pub content: String,
            }
        }
        "#,
    );
    let obj = g.as_object().unwrap();
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("word"));
    assert!(obj.contains_key("rules"));
    assert!(obj.contains_key("extras"));
    assert!(obj.contains_key("externals"));

    assert_eq!(g["name"], "full_meta");
    assert_eq!(g["word"], "Identifier");
}

#[test]
fn all_metadata_extras_include_both_extra_and_external() {
    let g = extract_one(
        r#"
        #[adze::grammar("combo_meta")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::extra]
            pub struct Space {
                #[adze::leaf(pattern = r" ")]
                pub s: String,
            }

            #[adze::external]
            pub struct Indent {
                #[adze::leaf(pattern = r"\t")]
                pub ind: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    let extra_names: Vec<&str> = extras
        .iter()
        .filter_map(|e| e["name"].as_str())
        .collect();
    // Space declared as extra
    assert!(extra_names.contains(&"Space"));
    // Indent declared as external is also added to extras
    assert!(extra_names.contains(&"Indent"));
}

// ---------------------------------------------------------------------------
// 7. Empty metadata fields
// ---------------------------------------------------------------------------

#[test]
fn minimal_grammar_has_required_keys() {
    let g = extract_one(
        r#"
        #[adze::grammar("minimal")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r".")]
                pub any: String,
            }
        }
        "#,
    );
    let obj = g.as_object().unwrap();
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("rules"));
    assert!(obj.contains_key("extras"));
    // word is present but null
    assert!(obj.contains_key("word"));
    assert!(g["word"].is_null());
}

#[test]
fn empty_extras_is_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("empty_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"x")]
                pub x: String,
            }
        }
        "#,
    );
    assert!(g["extras"].is_array());
    assert_eq!(g["extras"].as_array().unwrap().len(), 0);
}

#[test]
fn source_file_rule_references_language_root() {
    let g = extract_one(
        r#"
        #[adze::grammar("src_ref")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub code: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("source_file"), "rules must always contain source_file");
    let source_file = &rules["source_file"];
    assert_eq!(source_file["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(source_file["name"].as_str().unwrap(), "Program");
}

// ---------------------------------------------------------------------------
// 8. Metadata in JSON output
// ---------------------------------------------------------------------------

#[test]
fn json_roundtrip_preserves_metadata() {
    let g = extract_one(
        r#"
        #[adze::grammar("roundtrip")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::extra]
            pub struct WS {
                #[adze::leaf(pattern = r"\s")]
                pub ws: String,
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["name"], "roundtrip");
    assert!(parsed["extras"].is_array());
    assert!(parsed["rules"].is_object());
}

#[test]
fn json_pretty_print_contains_all_metadata_keys() {
    let g = extract_one(
        r#"
        #[adze::grammar("pretty")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                pub id: Ident,
            }

            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub v: String,
            }
        }
        "#,
    );
    let pretty = serde_json::to_string_pretty(&g).unwrap();
    assert!(pretty.contains("\"name\""));
    assert!(pretty.contains("\"word\""));
    assert!(pretty.contains("\"rules\""));
    assert!(pretty.contains("\"extras\""));
}

#[test]
fn json_field_types_correct() {
    let g = extract_one(
        r#"
        #[adze::grammar("type_check")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub t: String,
            }
        }
        "#,
    );
    assert!(g["name"].is_string());
    assert!(g["rules"].is_object());
    assert!(g["extras"].is_array());
}

// ---------------------------------------------------------------------------
// 9. Metadata validation
// ---------------------------------------------------------------------------

#[test]
fn grammar_without_language_root_panics() {
    let result = std::panic::catch_unwind(|| {
        extract_one(
            r#"
            #[adze::grammar("no_root")]
            mod grammar {
                pub struct NotRoot {
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub tok: String,
                }
            }
            "#,
        );
    });
    assert!(result.is_err(), "missing #[adze::language] should panic");
}

#[test]
fn enum_language_root_generates_choice() {
    let g = extract_one(
        r#"
        #[adze::grammar("enum_root")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Alpha(#[adze::leaf(pattern = r"[a-z]+")] String),
                Digit(#[adze::leaf(pattern = r"[0-9]+")] String),
            }
        }
        "#,
    );
    let token_rule = &g["rules"]["Token"];
    assert_eq!(
        token_rule["type"].as_str().unwrap(),
        "CHOICE",
        "enum language root should generate a CHOICE rule"
    );
}

#[test]
fn struct_language_root_generates_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("struct_root")]
        mod grammar {
            #[adze::language]
            pub struct Pair {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub left: String,
                #[adze::leaf(pattern = r"[0-9]+")]
                pub right: String,
            }
        }
        "#,
    );
    let pair_rule = &g["rules"]["Pair"];
    assert_eq!(
        pair_rule["type"].as_str().unwrap(),
        "SEQ",
        "struct with multiple fields should generate a SEQ rule"
    );
}

#[test]
fn leaf_pattern_generates_pattern_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("pat_rule")]
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
    // Field-level rules are prefixed with parent struct name
    assert!(rules.contains_key("Root_tok"), "pattern field should generate a named rule");
    assert_eq!(rules["Root_tok"]["type"].as_str().unwrap(), "PATTERN");
    assert_eq!(rules["Root_tok"]["value"].as_str().unwrap(), "[a-z]+");
}

#[test]
fn leaf_text_generates_string_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("text_rule")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "hello")]
                pub kw: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    // Field-level rules are prefixed with parent struct name
    assert!(rules.contains_key("Root_kw"));
    assert_eq!(rules["Root_kw"]["type"].as_str().unwrap(), "STRING");
    assert_eq!(rules["Root_kw"]["value"].as_str().unwrap(), "hello");
}

#[test]
fn multiple_grammars_extracted_independently() {
    let gs = extract(
        r#"
        #[adze::grammar("lang_a")]
        mod grammar_a {
            #[adze::language]
            pub struct RootA {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub a: String,
            }
        }

        #[adze::grammar("lang_b")]
        mod grammar_b {
            #[adze::language]
            pub struct RootB {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub b: String,
            }
        }
        "#,
    );
    assert_eq!(gs.len(), 2);
    let names: Vec<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"lang_a"));
    assert!(names.contains(&"lang_b"));
}

#[test]
fn word_rule_struct_also_appears_in_rules() {
    let g = extract_one(
        r#"
        #[adze::grammar("word_in_rules")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                pub kw: Keyword,
            }

            #[adze::word]
            pub struct Keyword {
                #[adze::leaf(pattern = r"[a-zA-Z]+")]
                pub val: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(
        rules.contains_key("Keyword"),
        "word rule struct should also appear in the rules map"
    );
}

#[test]
fn extra_struct_also_appears_in_rules() {
    let g = extract_one(
        r#"
        #[adze::grammar("extra_in_rules")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                pub ws: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(
        rules.contains_key("Whitespace"),
        "extra struct should generate a rule"
    );
}

#[test]
fn external_struct_not_in_rules_map() {
    let g = extract_one(
        r#"
        #[adze::grammar("ext_not_in_rules")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }

            #[adze::external]
            pub struct Scanner {
                #[adze::leaf(pattern = r".")]
                pub s: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    // External-only structs do not generate rules (they are handled externally)
    assert!(
        !rules.contains_key("Scanner"),
        "external-only struct should not appear in rules"
    );
}
