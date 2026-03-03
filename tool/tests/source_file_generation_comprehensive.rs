#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for source_file rule generation in adze-tool.
//!
//! Every grammar produces a `source_file` top-level rule that references
//! the `#[adze::language]`-annotated root type. These tests validate that
//! contract across struct roots, enum roots, extras, grammar names, and
//! various structural scenarios.

use std::fs;
use tempfile::TempDir;

/// Helper: write Rust source to a temp file and extract exactly one grammar.
fn extract_one(src: &str) -> serde_json::Value {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    let gs = adze_tool::generate_grammars(&path).unwrap();
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

/// Helper: extract and return the rules map.
fn rules_of(g: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
    g["rules"].as_object().unwrap()
}

// ---------------------------------------------------------------------------
// 1. source_file rule always present
// ---------------------------------------------------------------------------

#[test]
fn source_file_present_for_struct_root() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_struct")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub v: String,
            }
        }
        "#,
    );
    assert!(rules_of(&g).contains_key("source_file"));
}

#[test]
fn source_file_present_for_enum_root() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_enum")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    assert!(rules_of(&g).contains_key("source_file"));
}

#[test]
fn source_file_present_with_extras() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub n: String,
            }
            #[adze::extra]
            pub struct Ws {
                #[adze::leaf(pattern = r"\s")]
                pub _ws: String,
            }
        }
        "#,
    );
    assert!(rules_of(&g).contains_key("source_file"));
}

// ---------------------------------------------------------------------------
// 2. source_file references root type
// ---------------------------------------------------------------------------

#[test]
fn source_file_references_struct_root_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_ref_struct")]
        mod grammar {
            #[adze::language]
            pub struct MyRoot {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub name: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "MyRoot");
}

#[test]
fn source_file_references_enum_root_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_ref_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "Expr");
}

#[test]
fn source_file_symbol_name_matches_root_ident_exactly() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_exact_name")]
        mod grammar {
            #[adze::language]
            pub struct PascalCaseRoot {
                #[adze::leaf(pattern = r".+")]
                pub v: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "PascalCaseRoot");
}

// ---------------------------------------------------------------------------
// 3. source_file in grammar JSON
// ---------------------------------------------------------------------------

#[test]
fn source_file_is_first_rule_key() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_first")]
        mod grammar {
            #[adze::language]
            pub struct First {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub a: String,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    let first_key = rules.keys().next().unwrap();
    assert_eq!(first_key, "source_file");
}

#[test]
fn source_file_type_is_symbol() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_sym")]
        mod grammar {
            #[adze::language]
            pub struct Lang {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub w: String,
            }
        }
        "#,
    );
    assert_eq!(g["rules"]["source_file"]["type"].as_str().unwrap(), "SYMBOL");
}

#[test]
fn source_file_has_exactly_two_keys() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_keys")]
        mod grammar {
            #[adze::language]
            pub struct K {
                #[adze::leaf(pattern = r"\w+")]
                pub w: String,
            }
        }
        "#,
    );
    let sf = g["rules"]["source_file"].as_object().unwrap();
    assert_eq!(sf.len(), 2, "source_file should have exactly 'type' and 'name'");
    assert!(sf.contains_key("type"));
    assert!(sf.contains_key("name"));
}

#[test]
fn grammar_json_contains_source_file_key() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_json")]
        mod grammar {
            #[adze::language]
            pub struct P {
                #[adze::leaf(pattern = r"\d+")]
                pub n: String,
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    assert!(json_str.contains("\"source_file\""));
}

// ---------------------------------------------------------------------------
// 4. source_file with single root type (struct)
// ---------------------------------------------------------------------------

#[test]
fn struct_root_single_field_source_file_points_to_it() {
    let g = extract_one(
        r#"
        #[adze::grammar("single_field")]
        mod grammar {
            #[adze::language]
            pub struct Number {
                #[adze::leaf(pattern = r"\d+")]
                pub val: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "Number");
    // Also ensure the referenced rule exists
    assert!(rules_of(&g).contains_key("Number"));
}

#[test]
fn struct_root_multi_field_source_file_points_to_it() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_field")]
        mod grammar {
            #[adze::language]
            pub struct Pair {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub key: String,
                #[adze::leaf(pattern = r"\d+")]
                pub val: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "Pair");
}

#[test]
fn struct_root_with_child_struct_source_file_still_references_root() {
    let g = extract_one(
        r#"
        #[adze::grammar("with_child")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                pub expr: Expression,
            }
            pub struct Expression {
                #[adze::leaf(pattern = r"\d+")]
                pub val: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "Program");
}

// ---------------------------------------------------------------------------
// 5. source_file with enum root (CHOICE)
// ---------------------------------------------------------------------------

#[test]
fn enum_root_generates_choice_and_source_file_references_it() {
    let g = extract_one(
        r#"
        #[adze::grammar("enum_choice")]
        mod grammar {
            #[adze::language]
            pub enum Value {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "Value");
    // The referenced root should be a CHOICE
    let root = &g["rules"]["Value"];
    assert_eq!(root["type"].as_str().unwrap(), "CHOICE");
}

#[test]
fn enum_root_choice_members_count_matches_variants() {
    let g = extract_one(
        r#"
        #[adze::grammar("choice_count")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                A(#[adze::leaf(pattern = r"a")] String),
                B(#[adze::leaf(pattern = r"b")] String),
                C(#[adze::leaf(pattern = r"c")] String),
            }
        }
        "#,
    );
    let root = &g["rules"]["Tok"];
    let members = root["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);
}

#[test]
fn enum_root_recursive_still_referenced_by_source_file() {
    let g = extract_one(
        r#"
        #[adze::grammar("enum_recurse")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Neg(
                    #[adze::leaf(text = "-")] (),
                    Box<Expression>,
                ),
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "Expression");
}

// ---------------------------------------------------------------------------
// 6. Grammar name influences source_file
// ---------------------------------------------------------------------------

#[test]
fn grammar_name_does_not_affect_source_file_rule_name() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_fancy_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"x")]
                pub x: String,
            }
        }
        "#,
    );
    // source_file is always called "source_file", regardless of grammar name
    assert!(rules_of(&g).contains_key("source_file"));
    assert_eq!(g["rules"]["source_file"]["name"].as_str().unwrap(), "Root");
    assert_eq!(g["name"].as_str().unwrap(), "my_fancy_lang");
}

#[test]
fn different_grammar_names_produce_identical_source_file_structure() {
    let make = |name: &str| {
        extract_one(&format!(
            r#"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct R {{
                    #[adze::leaf(pattern = r"x")]
                    pub x: String,
                }}
            }}
            "#,
        ))
    };
    let a = make("alpha");
    let b = make("beta");
    assert_eq!(a["rules"]["source_file"], b["rules"]["source_file"]);
}

#[test]
fn grammar_name_stored_separately_from_source_file() {
    let g = extract_one(
        r#"
        #[adze::grammar("separate_name")]
        mod grammar {
            #[adze::language]
            pub struct Ast {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub id: String,
            }
        }
        "#,
    );
    // "name" is at the grammar level, not inside source_file
    assert_eq!(g["name"].as_str().unwrap(), "separate_name");
    assert!(g["rules"]["source_file"]["name"].as_str().unwrap() != "separate_name");
}

// ---------------------------------------------------------------------------
// 7. source_file rule structure
// ---------------------------------------------------------------------------

#[test]
fn source_file_is_symbol_not_field_or_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("sf_structure")]
        mod grammar {
            #[adze::language]
            pub struct S {
                #[adze::leaf(pattern = r"\d+")]
                pub n: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    let ty = sf["type"].as_str().unwrap();
    assert_eq!(ty, "SYMBOL", "source_file must be a SYMBOL, got {ty}");
}

#[test]
fn source_file_does_not_wrap_in_extra_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_seq_wrap")]
        mod grammar {
            #[adze::language]
            pub struct W {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub a: String,
                #[adze::leaf(pattern = r"\d+")]
                pub b: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    // Even for multi-field roots, source_file is a SYMBOL to the root, not a SEQ
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
}

#[test]
fn source_file_does_not_contain_repeat() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_repeat")]
        mod grammar {
            #[adze::language]
            pub struct Item {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub w: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_ne!(sf["type"].as_str().unwrap(), "REPEAT");
    assert_ne!(sf["type"].as_str().unwrap(), "REPEAT1");
}

#[test]
fn source_file_distinct_from_root_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("distinct")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub v: String,
            }
        }
        "#,
    );
    let rules = rules_of(&g);
    // source_file and the root rule are both present and different objects
    assert!(rules.contains_key("source_file"));
    assert!(rules.contains_key("Root"));
    assert_ne!(rules["source_file"], rules["Root"]);
}

#[test]
fn source_file_name_field_is_string() {
    let g = extract_one(
        r#"
        #[adze::grammar("name_str")]
        mod grammar {
            #[adze::language]
            pub struct T {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub d: String,
            }
        }
        "#,
    );
    assert!(g["rules"]["source_file"]["name"].is_string());
}

// ---------------------------------------------------------------------------
// 8. Extras (whitespace) interaction with source_file
// ---------------------------------------------------------------------------

#[test]
fn extras_not_in_source_file_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("extras_separate")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub n: String,
            }
            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                pub _whitespace: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    // source_file should only be a SYMBOL reference – extras live separately
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    let sf_str = serde_json::to_string(sf).unwrap();
    assert!(!sf_str.contains("Whitespace"));
}

#[test]
fn extras_appear_in_extras_array_not_source_file() {
    let g = extract_one(
        r#"
        #[adze::grammar("extras_arr")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub n: String,
            }
            #[adze::extra]
            pub struct Ws {
                #[adze::leaf(pattern = r"\s")]
                pub _ws: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    assert!(!extras.is_empty(), "extras array should have entries");
    let has_ws = extras
        .iter()
        .any(|e| e["name"].as_str() == Some("Ws"));
    assert!(has_ws, "Ws should be in extras array");
}

#[test]
fn no_extras_yields_empty_extras_array() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub v: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    assert!(extras.is_empty());
}

#[test]
fn source_file_unchanged_with_or_without_extras() {
    let with_extras = extract_one(
        r#"
        #[adze::grammar("we")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"x")]
                pub x: String,
            }
            #[adze::extra]
            pub struct Sp {
                #[adze::leaf(pattern = r"\s")]
                pub _s: String,
            }
        }
        "#,
    );
    let without_extras = extract_one(
        r#"
        #[adze::grammar("woe")]
        mod grammar {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r"x")]
                pub x: String,
            }
        }
        "#,
    );
    assert_eq!(
        with_extras["rules"]["source_file"],
        without_extras["rules"]["source_file"],
    );
}

#[test]
fn multiple_extras_dont_affect_source_file() {
    let g = extract_one(
        r#"
        #[adze::grammar("multi_extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub w: String,
            }
            #[adze::extra]
            pub struct Ws {
                #[adze::leaf(pattern = r"\s")]
                pub _ws: String,
            }
            #[adze::extra]
            pub struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                pub _c: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "Root");
}

// ---------------------------------------------------------------------------
// 9. Additional structural / edge-case tests
// ---------------------------------------------------------------------------

#[test]
fn source_file_not_duplicated_in_rules() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_dup")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                pub v: String,
            }
        }
        "#,
    );
    // OrderMap keys are unique, but verify serialized JSON has exactly one "source_file"
    let json_str = serde_json::to_string_pretty(&g["rules"]).unwrap();
    let count = json_str.matches("\"source_file\"").count();
    assert_eq!(count, 1, "source_file should appear exactly once as a key");
}

#[test]
fn source_file_survives_roundtrip_serialization() {
    let g = extract_one(
        r#"
        #[adze::grammar("roundtrip")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub n: String,
            }
        }
        "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(
        parsed["rules"]["source_file"],
        g["rules"]["source_file"],
    );
}

#[test]
fn source_file_for_optional_field_root() {
    let g = extract_one(
        r#"
        #[adze::grammar("opt_field")]
        mod grammar {
            #[adze::language]
            pub struct MaybeNum {
                #[adze::leaf(pattern = r"\d+")]
                pub v: Option<String>,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "MaybeNum");
}

#[test]
fn source_file_for_vec_field_root() {
    let g = extract_one(
        r#"
        #[adze::grammar("vec_root")]
        mod grammar {
            #[adze::language]
            pub struct NumList {
                pub nums: Vec<Num>,
            }
            pub struct Num {
                #[adze::leaf(pattern = r"\d+")]
                pub v: String,
            }
            #[adze::extra]
            pub struct Ws {
                #[adze::leaf(pattern = r"\s")]
                pub _ws: String,
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "NumList");
}

#[test]
fn source_file_for_enum_with_precedence() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "Expr");
}

#[test]
fn source_file_for_deeply_nested_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("deep")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                pub stmt: Statement,
            }
            pub struct Statement {
                pub expr: Expression,
            }
            pub struct Expression {
                #[adze::leaf(pattern = r"\d+")]
                pub val: String,
            }
        }
        "#,
    );
    // source_file always points to the #[adze::language] root, not deeper types
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "Program");
}

#[test]
fn source_file_never_appears_as_extras() {
    let g = extract_one(
        r#"
        #[adze::grammar("no_sf_extra")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                pub w: String,
            }
            #[adze::extra]
            pub struct Ws {
                #[adze::leaf(pattern = r"\s")]
                pub _ws: String,
            }
        }
        "#,
    );
    let extras = g["extras"].as_array().unwrap();
    let has_source_file = extras
        .iter()
        .any(|e| e["name"].as_str() == Some("source_file"));
    assert!(!has_source_file, "source_file must not appear in extras");
}
