#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for grammar JSON generation in adze-tool.
//!
//! Covers: JSON grammar structure correctness, rule serialization,
//! token/pattern representation, grammar metadata, edge cases,
//! special characters in rule names, and Unicode in grammar definitions.

use std::fs;
use tempfile::TempDir;

/// Write Rust source to a temp file and extract grammars.
fn extract(src: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

/// Extract exactly one grammar and return it.
fn extract_one(src: &str) -> serde_json::Value {
    let gs = extract(src);
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

// ===========================================================================
// 1. JSON grammar structure correctness
// ===========================================================================

#[test]
fn top_level_json_is_object_with_required_keys() {
    let g = extract_one(
        r#"
        #[adze::grammar("structure_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }
        }
        "#,
    );
    let obj = g.as_object().unwrap();
    assert!(obj.contains_key("name"), "missing 'name'");
    assert!(obj.contains_key("rules"), "missing 'rules'");
    assert!(obj.contains_key("extras"), "missing 'extras'");
    assert!(obj.contains_key("word"), "missing 'word'");
}

#[test]
fn rules_value_is_object() {
    let g = extract_one(
        r#"
        #[adze::grammar("rules_obj")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "x")]
                pub x: String,
            }
        }
        "#,
    );
    assert!(g["rules"].is_object(), "rules should be a JSON object");
}

#[test]
fn extras_value_is_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("extras_arr")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "y")]
                pub y: String,
            }
        }
        "#,
    );
    assert!(g["extras"].is_array(), "extras should be a JSON array");
}

#[test]
fn source_file_is_always_first_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("first_rule")]
        mod grammar {
            #[adze::language]
            pub struct Alpha {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let first_key = rules.keys().next().unwrap();
    assert_eq!(first_key, "source_file", "source_file must be the first rule key");
}

#[test]
fn source_file_references_language_root_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("ref_root")]
        mod grammar {
            #[adze::language]
            pub struct MyLang {
                #[adze::leaf(pattern = r".+")]
                pub content: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "MyLang");
}

// ===========================================================================
// 2. Rule serialization to JSON
// ===========================================================================

#[test]
fn struct_with_two_fields_serializes_as_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("seq_test")]
        mod grammar {
            #[adze::language]
            pub struct Pair {
                #[adze::leaf(text = "(")]
                pub open: String,
                #[adze::leaf(text = ")")]
                pub close: String,
            }
        }
        "#,
    );
    let rule = &g["rules"]["Pair"];
    assert_eq!(rule["type"].as_str().unwrap(), "SEQ");
    let members = rule["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
}

#[test]
fn struct_with_single_field_serializes_as_field() {
    let g = extract_one(
        r#"
        #[adze::grammar("single_field")]
        mod grammar {
            #[adze::language]
            pub struct Mono {
                #[adze::leaf(pattern = r"\w+")]
                pub word: String,
            }
        }
        "#,
    );
    let rule = &g["rules"]["Mono"];
    // Single field struct produces a FIELD node directly (not SEQ)
    assert_eq!(rule["type"].as_str().unwrap(), "FIELD");
    assert_eq!(rule["name"].as_str().unwrap(), "word");
}

#[test]
fn enum_choice_members_are_correct_json_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("choice_json")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                A(#[adze::leaf(text = "a")] String),
                B(#[adze::leaf(text = "b")] String),
            }
        }
        "#,
    );
    let choice = &g["rules"]["Tok"];
    assert_eq!(choice["type"].as_str().unwrap(), "CHOICE");
    for member in choice["members"].as_array().unwrap() {
        assert!(member.is_object(), "each CHOICE member must be a JSON object");
    }
}

#[test]
fn grammar_json_roundtrips_through_serde_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("roundtrip_json")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub num: String,
            }
        }
        "#,
    );
    let serialized = serde_json::to_string(&g).unwrap();
    let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(g, deserialized);
}

// ===========================================================================
// 3. Token/pattern representation
// ===========================================================================

#[test]
fn pattern_leaf_produces_pattern_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("pat_type")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"[0-9a-fA-F]+")]
                pub hex: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let pat_rule = rules.get("R_hex").unwrap();
    assert_eq!(pat_rule["type"].as_str().unwrap(), "PATTERN");
    assert_eq!(pat_rule["value"].as_str().unwrap(), "[0-9a-fA-F]+");
}

#[test]
fn text_leaf_produces_string_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("str_type")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "hello")]
                pub greeting: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let str_rule = rules.get("R_greeting").unwrap();
    assert_eq!(str_rule["type"].as_str().unwrap(), "STRING");
    assert_eq!(str_rule["value"].as_str().unwrap(), "hello");
}

#[test]
fn pattern_value_preserves_regex_escapes() {
    let g = extract_one(
        r#"
        #[adze::grammar("regex_esc")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"\d+\.\d+")]
                pub decimal: String,
            }
        }
        "#,
    );
    let pat = g["rules"].as_object().unwrap().get("R_decimal").unwrap();
    assert_eq!(pat["value"].as_str().unwrap(), r"\d+\.\d+");
}

#[test]
fn text_with_special_json_chars_preserved() {
    let g = extract_one(
        r#"
        #[adze::grammar("json_chars")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "\"")]
                pub quote: String,
            }
        }
        "#,
    );
    let str_rule = g["rules"].as_object().unwrap().get("R_quote").unwrap();
    assert_eq!(str_rule["value"].as_str().unwrap(), "\"");
}

// ===========================================================================
// 4. Grammar metadata in JSON
// ===========================================================================

#[test]
fn grammar_name_matches_annotation() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_special_grammar")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "a")]
                pub a: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "my_special_grammar");
}

#[test]
fn word_field_is_null_when_no_word_annotation() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_word")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub tok: String,
            }
        }
        "#,
    );
    assert!(g["word"].is_null(), "word should be null when no #[adze::word] is present");
}

#[test]
fn word_field_set_when_word_annotation_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("with_word")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"[a-z]+")]
                #[adze::word]
                pub ident: String,
            }
        }
        "#,
    );
    assert_eq!(g["word"].as_str().unwrap(), "R_ident");
}

#[test]
fn extras_empty_when_no_extra_structs() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_extras")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "ok")]
                pub ok: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    assert!(extras.is_empty(), "extras should be empty when no #[adze::extra] structs");
}

#[test]
fn externals_key_absent_when_no_external_structs() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_ext")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "z")]
                pub z: String,
            }
        }
        "#,
    );
    assert!(
        g.get("externals").is_none(),
        "externals key should not be present when there are no external structs"
    );
}

#[test]
fn externals_present_when_external_struct_defined() {
    let g = extract_one(
        r#"
        #[adze::grammar("has_ext")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }

            #[adze::external]
            pub struct Newline {
                #[adze::leaf(pattern = r"\n")]
                pub nl: String,
            }
        }
        "#,
    );
    let externals = g.get("externals").expect("externals key should exist");
    let ext_arr = externals.as_array().unwrap();
    assert!(
        ext_arr.iter().any(|e| e["name"].as_str() == Some("Newline")),
        "externals should contain Newline"
    );
}

// ===========================================================================
// 5. Edge cases in JSON output
// ===========================================================================

#[test]
fn option_field_wraps_in_choice_with_blank() {
    let g = extract_one(
        r#"
        #[adze::grammar("opt_edge")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub required: String,
                #[adze::leaf(pattern = r"[0-9]+")]
                pub maybe: Option<String>,
            }
        }
        "#,
    );
    let root = &g["rules"]["R"];
    assert_eq!(root["type"].as_str().unwrap(), "SEQ");
    let members = root["members"].as_array().unwrap();
    // The optional (second) member should be CHOICE with a BLANK member
    let opt = &members[1];
    assert_eq!(opt["type"].as_str().unwrap(), "CHOICE");
    let choices = opt["members"].as_array().unwrap();
    assert!(
        choices.iter().any(|c| c["type"].as_str() == Some("BLANK")),
        "Option field must have a BLANK alternative"
    );
}

#[test]
fn vec_field_generates_vec_contents_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("vec_edge")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"\d+")]
                pub nums: Vec<i32>,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(
        rules.contains_key("R_nums_vec_contents"),
        "Vec should generate a _vec_contents rule, got: {:?}",
        rules.keys().collect::<Vec<_>>()
    );
    let contents = &rules["R_nums_vec_contents"];
    assert_eq!(contents["type"].as_str().unwrap(), "REPEAT1");
}

#[test]
fn multiple_grammars_in_nested_modules() {
    let gs = extract(
        r#"
        #[adze::grammar("gram_a")]
        mod grammar_a {
            #[adze::language]
            pub struct A {
                #[adze::leaf(text = "a")]
                pub a: String,
            }
        }

        #[adze::grammar("gram_b")]
        mod grammar_b {
            #[adze::language]
            pub struct B {
                #[adze::leaf(text = "b")]
                pub b: String,
            }
        }
        "#,
    );
    assert_eq!(gs.len(), 2, "should extract two grammars");
    let names: Vec<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"gram_a"));
    assert!(names.contains(&"gram_b"));
}

#[test]
fn no_grammar_module_yields_empty_vec() {
    let gs = extract("pub fn nothing() {}");
    assert!(gs.is_empty());
}

#[test]
fn prec_left_wraps_content_with_correct_value() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_json")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::prec_left(3)]
                Add {
                    lhs: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    op: String,
                    rhs: Box<Expr>,
                },
                Lit(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let add = g["rules"].as_object().unwrap().get("Expr_Add").unwrap();
    assert_eq!(add["type"].as_str().unwrap(), "PREC_LEFT");
    assert_eq!(add["value"].as_u64().unwrap(), 3);
    assert!(add["content"].is_object(), "PREC_LEFT should have a content field");
}

// ===========================================================================
// 6. Special characters in rule names
// ===========================================================================

#[test]
fn rule_name_with_underscore_prefix() {
    let g = extract_one(
        r#"
        #[adze::grammar("underscore_name")]
        mod grammar {
            #[adze::language]
            pub struct _Internal {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("_Internal"), "rule named _Internal should exist");
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "_Internal");
}

#[test]
fn rule_name_with_numeric_suffix() {
    let g = extract_one(
        r#"
        #[adze::grammar("num_suffix")]
        mod grammar {
            #[adze::language]
            pub struct Token42 {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub n: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Token42"), "rule Token42 should exist");
}

#[test]
fn enum_variant_names_include_parent() {
    let g = extract_one(
        r#"
        #[adze::grammar("variant_names")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::prec_left(1)]
                Add {
                    left: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    op: String,
                    right: Box<Expr>,
                },
                Num(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(
        rules.contains_key("Expr_Add"),
        "Enum variant should be named Parent_Variant, got: {:?}",
        rules.keys().collect::<Vec<_>>()
    );
}

// ===========================================================================
// 7. Unicode in grammar definitions
// ===========================================================================

#[test]
fn unicode_pattern_preserved_in_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("unicode_pat")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"[\u00C0-\u00FF]+")]
                pub accented: String,
            }
        }
        "#,
    );
    let pat = g["rules"].as_object().unwrap().get("R_accented").unwrap();
    assert_eq!(pat["type"].as_str().unwrap(), "PATTERN");
    assert_eq!(pat["value"].as_str().unwrap(), r"[\u00C0-\u00FF]+");
}

#[test]
fn unicode_text_literal_preserved_in_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("unicode_text")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "→")]
                pub arrow: String,
            }
        }
        "#,
    );
    let str_rule = g["rules"].as_object().unwrap().get("R_arrow").unwrap();
    assert_eq!(str_rule["type"].as_str().unwrap(), "STRING");
    assert_eq!(str_rule["value"].as_str().unwrap(), "→");
}

#[test]
fn unicode_grammar_name() {
    // Grammar names are strings — they should support any valid string
    let g = extract_one(
        r#"
        #[adze::grammar("grammaire_française")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "oui")]
                pub mot: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "grammaire_française");
}

#[test]
fn cjk_text_literal_preserved() {
    let g = extract_one(
        r#"
        #[adze::grammar("cjk_test")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "你好")]
                pub greeting: String,
            }
        }
        "#,
    );
    let str_rule = g["rules"].as_object().unwrap().get("R_greeting").unwrap();
    assert_eq!(str_rule["value"].as_str().unwrap(), "你好");
}

#[test]
fn emoji_text_literal_preserved() {
    let g = extract_one(
        r#"
        #[adze::grammar("emoji_test")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "🚀")]
                pub rocket: String,
            }
        }
        "#,
    );
    let str_rule = g["rules"].as_object().unwrap().get("R_rocket").unwrap();
    assert_eq!(str_rule["value"].as_str().unwrap(), "🚀");
}

// ===========================================================================
// 8. Additional edge cases and structure tests
// ===========================================================================

#[test]
fn extra_struct_rule_and_extras_entry_both_present() {
    let g = extract_one(
        r#"
        #[adze::grammar("extra_rule")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "x")]
                pub x: String,
            }

            #[adze::extra]
            pub struct WS {
                #[adze::leaf(pattern = r"\s+")]
                pub ws: String,
            }
        }
        "#,
    );
    // The extra struct should appear both as a rule and in extras list
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("WS"), "extra struct should generate a rule");
    let extras = g["extras"].as_array().unwrap();
    assert!(
        extras.iter().any(|e| e["name"].as_str() == Some("WS")),
        "extra struct should appear in extras array"
    );
}

#[test]
fn complex_pattern_with_alternation() {
    let g = extract_one(
        r#"
        #[adze::grammar("complex_regex")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"(true|false|null)")]
                pub keyword: String,
            }
        }
        "#,
    );
    let pat = g["rules"].as_object().unwrap().get("R_keyword").unwrap();
    assert_eq!(pat["value"].as_str().unwrap(), "(true|false|null)");
}

#[test]
fn field_node_contains_name_and_content() {
    let g = extract_one(
        r#"
        #[adze::grammar("field_shape")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(text = "val")]
                pub my_field: String,
            }
        }
        "#,
    );
    let rule = &g["rules"]["R"];
    assert_eq!(rule["type"].as_str().unwrap(), "FIELD");
    assert_eq!(rule["name"].as_str().unwrap(), "my_field");
    assert!(rule["content"].is_object(), "FIELD should have content");
    assert_eq!(rule["content"]["type"].as_str().unwrap(), "SYMBOL");
}
