#![allow(clippy::needless_range_loop)]

//! Property-based and deterministic tests for rule naming conventions
//! in adze-tool grammar generation.

use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

fn extract_one(src: &str) -> Value {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    let gs = adze_tool::generate_grammars(&path).unwrap();
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

fn rule_names(grammar: &Value) -> Vec<String> {
    grammar["rules"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

fn has_rule(grammar: &Value, name: &str) -> bool {
    grammar["rules"]
        .as_object()
        .map(|m| m.contains_key(name))
        .unwrap_or(false)
}

/// Recursively collect every "name" from SYMBOL nodes.
fn collect_symbol_names(val: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("SYMBOL")
                && let Some(n) = map.get("name").and_then(|v| v.as_str())
            {
                out.push(n.to_string());
            }
            for v in map.values() {
                out.extend(collect_symbol_names(v));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.extend(collect_symbol_names(v));
            }
        }
        _ => {}
    }
    out
}

// ===========================================================================
// 1. Struct names become rule names
// ===========================================================================

#[test]
fn struct_name_becomes_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct MyToken {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(has_rule(&g, "MyToken"), "struct name should become a rule");
}

#[test]
fn struct_name_preserves_case() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct CamelCaseName {
                #[adze::leaf(pattern = r"\d+")]
                pub num: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "CamelCaseName"),
        "PascalCase struct name should be preserved exactly"
    );
}

#[test]
fn non_root_struct_becomes_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                pub child: Child,
            }
            pub struct Child {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(has_rule(&g, "Root"), "root struct should be a rule");
    assert!(has_rule(&g, "Child"), "child struct should also be a rule");
}

// ===========================================================================
// 2. Enum names become rule names
// ===========================================================================

#[test]
fn enum_name_becomes_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Alpha(#[adze::leaf(pattern = r"[a-z]+")] String),
                Digit(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    "#;
    let g = extract_one(src);
    assert!(has_rule(&g, "Expr"), "enum name should become a rule");
}

#[test]
fn enum_rule_is_choice() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    "#;
    let g = extract_one(src);
    let rule = &g["rules"]["Token"];
    assert_eq!(
        rule["type"].as_str().unwrap(),
        "CHOICE",
        "enum rule should be CHOICE"
    );
}

#[test]
fn enum_name_preserves_case() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum BinaryOp {
                Plus(#[adze::leaf(text = "+")] String),
                Minus(#[adze::leaf(text = "-")] String),
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "BinaryOp"),
        "PascalCase enum name should be preserved"
    );
}

// ===========================================================================
// 3. Variant names in enum rules
// ===========================================================================

#[test]
fn variant_names_use_enum_prefix() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Value {
                #[adze::prec(1)]
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec(2)]
                Word(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
    "#;
    let g = extract_one(src);
    // Variants with prec are not inlined so they get separate rules
    assert!(
        has_rule(&g, "Value_Num"),
        "variant rule should be EnumName_VariantName"
    );
    assert!(
        has_rule(&g, "Value_Word"),
        "variant rule should be EnumName_VariantName"
    );
}

#[test]
fn unit_variant_creates_separate_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Kind {
                Alpha(#[adze::leaf(pattern = r"[a-z]+")] String),
                Empty,
            }
        }
    "#;
    let g = extract_one(src);
    // Unit variants are never inlined per expansion.rs line 500-502
    assert!(
        has_rule(&g, "Kind_Empty"),
        "unit variant should create EnumName_VariantName rule"
    );
}

#[test]
fn no_inline_variant_creates_separate_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                #[adze::no_inline]
                Letter(#[adze::leaf(pattern = r"[a-z]+")] String),
                Digit(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Tok_Letter"),
        "no_inline variant should create a separate rule"
    );
}

#[test]
fn variant_with_prec_not_inlined() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Op {
                #[adze::prec_left(1)]
                Add(#[adze::leaf(text = "+")] String),
                #[adze::prec_left(2)]
                Mul(#[adze::leaf(text = "*")] String),
            }
        }
    "#;
    let g = extract_one(src);
    assert!(has_rule(&g, "Op_Add"), "prec variant should not be inlined");
    assert!(has_rule(&g, "Op_Mul"), "prec variant should not be inlined");
}

// ===========================================================================
// 4. Auxiliary rule naming (_vec_contents, optional, etc.)
// ===========================================================================

#[test]
fn vec_field_creates_vec_contents_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct List {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub items: Vec<String>,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "List_items_vec_contents"),
        "Vec field should create Type_field_vec_contents rule"
    );
}

#[test]
fn vec_contents_rule_uses_repeat() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Items {
                #[adze::leaf(pattern = r"\w+")]
                pub vals: Vec<String>,
            }
        }
    "#;
    let g = extract_one(src);
    let contents_rule = &g["rules"]["Items_vals_vec_contents"];
    assert_eq!(
        contents_rule["type"].as_str().unwrap(),
        "REPEAT1",
        "vec_contents rule should use REPEAT1"
    );
}

#[test]
fn delimited_vec_creates_delimiter_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Csv {
                #[adze::leaf(pattern = r"[a-z]+")]
                #[adze::delimited(#[adze::leaf(text = ",")] String)]
                pub items: Vec<String>,
            }
        }
    "#;
    let g = extract_one(src);
    let names = rule_names(&g);
    assert!(
        names.iter().any(|n| n.contains("vec_delimiter")),
        "delimited vec should create a delimiter rule"
    );
}

#[test]
fn optional_field_does_not_create_extra_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Pair {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub first: String,
                #[adze::leaf(pattern = r"\d+")]
                pub second: Option<String>,
            }
        }
    "#;
    let g = extract_one(src);
    let names = rule_names(&g);
    // Option<T> doesn't create a separate rule; it wraps in CHOICE/BLANK
    assert!(
        !names.iter().any(|n| n.contains("_optional")),
        "Option should not create a separate _optional rule"
    );
}

// ===========================================================================
// 5. Child struct rule naming
// ===========================================================================

#[test]
fn child_struct_rule_uses_struct_name() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Parent {
                pub child: Inner,
            }
            pub struct Inner {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Inner"),
        "child struct uses its own struct name as rule name"
    );
    // Parent references Inner via a SYMBOL
    let parent_syms = collect_symbol_names(&g["rules"]["Parent"]);
    assert!(
        parent_syms.contains(&"Inner".to_string()),
        "parent should reference child by struct name"
    );
}

#[test]
fn deeply_nested_child_structs_each_have_rules() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct A {
                pub b: B,
            }
            pub struct B {
                pub c: C,
            }
            pub struct C {
                #[adze::leaf(pattern = r"\d+")]
                pub val: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(has_rule(&g, "A"));
    assert!(has_rule(&g, "B"));
    assert!(has_rule(&g, "C"));
}

#[test]
fn child_struct_field_generates_named_leaf_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Wrapper {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub name: String,
                #[adze::leaf(pattern = r"\d+")]
                pub count: String,
            }
        }
    "#;
    let g = extract_one(src);
    let names = rule_names(&g);
    assert!(
        names.iter().any(|n| n.contains("name")),
        "leaf field should generate a rule containing field name"
    );
}

// ===========================================================================
// 6. Leaf rule naming
// ===========================================================================

#[test]
fn leaf_pattern_creates_named_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Lex {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub ident: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Lex_ident"),
        "leaf field should generate Struct_field rule"
    );
    let leaf_rule = &g["rules"]["Lex_ident"];
    assert_eq!(leaf_rule["type"].as_str().unwrap(), "PATTERN");
}

#[test]
fn leaf_text_creates_string_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Kw {
                #[adze::leaf(text = "let")]
                pub keyword: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Kw_keyword"),
        "text leaf should generate Struct_field rule"
    );
    let leaf_rule = &g["rules"]["Kw_keyword"];
    assert_eq!(leaf_rule["type"].as_str().unwrap(), "STRING");
    assert_eq!(leaf_rule["value"].as_str().unwrap(), "let");
}

#[test]
fn leaf_field_name_matches_struct_field() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Tok {
                #[adze::leaf(pattern = r"[0-9]+")]
                pub my_number: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Tok_my_number"),
        "leaf rule name should incorporate field name"
    );
}

#[test]
fn multiple_leaf_fields_each_get_own_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Pair {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub key: String,
                #[adze::leaf(pattern = r"\d+")]
                pub value: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Pair_key"),
        "first leaf field gets its own rule"
    );
    assert!(
        has_rule(&g, "Pair_value"),
        "second leaf field gets its own rule"
    );
}

// ===========================================================================
// 7. Source file rule naming
// ===========================================================================

#[test]
fn source_file_rule_always_present() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "source_file"),
        "source_file rule must always be present"
    );
}

#[test]
fn source_file_references_root_type() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub code: String,
            }
        }
    "#;
    let g = extract_one(src);
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(
        sf["name"].as_str().unwrap(),
        "Program",
        "source_file should reference the #[adze::language] type"
    );
}

#[test]
fn source_file_is_first_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Start {
                #[adze::leaf(pattern = r"\w+")]
                pub tok: String,
            }
        }
    "#;
    let g = extract_one(src);
    let names = rule_names(&g);
    assert_eq!(
        names.first().map(|s| s.as_str()),
        Some("source_file"),
        "source_file must be the first rule"
    );
}

#[test]
fn source_file_references_enum_root() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Entry {
                Lit(#[adze::leaf(pattern = r"[a-z]+")] String),
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    "#;
    let g = extract_one(src);
    let sf = &g["rules"]["source_file"];
    assert_eq!(sf["name"].as_str().unwrap(), "Entry");
}

// ===========================================================================
// 8. Rule naming determinism
// ===========================================================================

#[test]
fn same_source_produces_identical_rules() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Token {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub word: String,
                #[adze::leaf(pattern = r"\d+")]
                pub num: String,
            }
        }
    "#;
    let g1 = extract_one(src);
    let g2 = extract_one(src);
    assert_eq!(g1, g2, "grammar generation must be deterministic");
}

#[test]
fn rule_names_stable_across_runs() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::prec(1)]
                Lit(#[adze::leaf(pattern = r"[a-z]+")] String),
                #[adze::prec(2)]
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    "#;
    let names1 = rule_names(&extract_one(src));
    let names2 = rule_names(&extract_one(src));
    assert_eq!(names1, names2, "rule names should be identical across runs");
}

#[test]
fn rule_order_is_deterministic() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Doc {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub alpha: String,
                #[adze::leaf(pattern = r"\d+")]
                pub beta: String,
                #[adze::leaf(pattern = r"[A-Z]+")]
                pub gamma: String,
            }
        }
    "#;
    for _ in 0..5 {
        let names = rule_names(&extract_one(src));
        assert_eq!(
            names.first().map(|s| s.as_str()),
            Some("source_file"),
            "first rule must always be source_file"
        );
    }
}

#[test]
fn determinism_with_vec_fields() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Items {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub words: Vec<String>,
            }
        }
    "#;
    let g1 = extract_one(src);
    let g2 = extract_one(src);
    assert_eq!(
        rule_names(&g1),
        rule_names(&g2),
        "vec rule naming must be deterministic"
    );
}

#[test]
fn determinism_with_enum_variants() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Kind {
                #[adze::prec(1)]
                A(#[adze::leaf(text = "a")] String),
                #[adze::prec(2)]
                B(#[adze::leaf(text = "b")] String),
                #[adze::prec(3)]
                C(#[adze::leaf(text = "c")] String),
            }
        }
    "#;
    let g1 = extract_one(src);
    let g2 = extract_one(src);
    assert_eq!(g1, g2, "enum variant rules must be deterministic");
}

// ===========================================================================
// Additional coverage: edge cases and combinations
// ===========================================================================

#[test]
fn grammar_name_from_attribute() {
    let src = r#"
        #[adze::grammar("my_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert_eq!(
        g["name"].as_str().unwrap(),
        "my_lang",
        "grammar name should match the attribute"
    );
}

#[test]
fn enum_with_struct_variant_naming() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Stmt {
                #[adze::no_inline]
                Assign {
                    #[adze::leaf(pattern = r"[a-z]+")]
                    name: String,
                    #[adze::leaf(text = "=")]
                    eq: String,
                    #[adze::leaf(pattern = r"\d+")]
                    value: String,
                },
                #[adze::no_inline]
                Print {
                    #[adze::leaf(text = "print")]
                    kw: String,
                    #[adze::leaf(pattern = r"[a-z]+")]
                    arg: String,
                },
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Stmt_Assign"),
        "struct-like variant should create Enum_Variant rule"
    );
    assert!(
        has_rule(&g, "Stmt_Print"),
        "struct-like variant should create Enum_Variant rule"
    );
}

#[test]
fn extra_struct_still_creates_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }
            #[adze::extra]
            pub struct Whitespace {
                #[adze::leaf(pattern = r"\s+")]
                pub ws: String,
            }
        }
    "#;
    let g = extract_one(src);
    assert!(
        has_rule(&g, "Whitespace"),
        "extra struct should still create a rule"
    );
}

#[test]
fn vec_element_field_naming() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Nums {
                #[adze::leaf(pattern = r"\d+")]
                pub digits: Vec<String>,
            }
        }
    "#;
    let g = extract_one(src);
    let contents = &g["rules"]["Nums_digits_vec_contents"];
    // The REPEAT1 content should contain a FIELD with name ending in _vec_element
    let content = &contents["content"];
    assert_eq!(content["type"].as_str().unwrap(), "FIELD");
    assert_eq!(
        content["name"].as_str().unwrap(),
        "Nums_digits_vec_element",
        "vec element field should be named Struct_field_vec_element"
    );
}

#[test]
fn leaf_in_enum_variant_creates_typed_rule() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                #[adze::prec(1)]
                Ident(#[adze::leaf(pattern = r"[a-z_]+")] String),
                #[adze::prec(2)]
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    "#;
    let g = extract_one(src);
    // Non-inlined variants with prec create named rules
    assert!(has_rule(&g, "Token_Ident"));
    assert!(has_rule(&g, "Token_Number"));
}

#[test]
fn all_rules_are_non_empty_strings() {
    let src = r#"
        #[adze::grammar("g")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                pub child: Inner,
                #[adze::leaf(pattern = r"[a-z]+")]
                pub items: Vec<String>,
            }
            pub struct Inner {
                #[adze::leaf(pattern = r"\d+")]
                pub val: String,
            }
        }
    "#;
    let g = extract_one(src);
    for name in rule_names(&g) {
        assert!(!name.is_empty(), "rule name should not be empty");
        assert!(
            !name.starts_with(' ') && !name.ends_with(' '),
            "rule name '{}' should not have leading/trailing spaces",
            name
        );
    }
}
