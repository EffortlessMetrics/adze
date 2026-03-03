#![allow(clippy::needless_range_loop)]

//! Property-based tests for module discovery and processing in adze-tool.
//!
//! Tests the `generate_grammars` public API with focus on how grammar modules
//! are found, named, attributed, and how structs/enums/nested modules are
//! discovered within them. Covers:
//!   1. Find grammar modules in source files
//!   2. Module name extraction
//!   3. Module attribute detection
//!   4. Struct discovery within modules
//!   5. Enum discovery within modules
//!   6. Nested module handling
//!   7. Module discovery determinism
//!   8. Empty module handling

use proptest::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

fn extract(src: &str) -> Vec<Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

fn extract_one(src: &str) -> Value {
    let gs = extract(src);
    assert_eq!(gs.len(), 1, "expected 1 grammar, got {}", gs.len());
    gs.into_iter().next().unwrap()
}

fn rule_names(grammar: &Value) -> Vec<String> {
    grammar["rules"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

fn collect_typed_values(val: &Value, type_name: &str) -> Vec<String> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some(type_name) {
                if let Some(v) = map.get("value").and_then(|v| v.as_str()) {
                    out.push(v.to_string());
                }
            }
            for v in map.values() {
                out.extend(collect_typed_values(v, type_name));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.extend(collect_typed_values(v, type_name));
            }
        }
        _ => {}
    }
    out
}

fn collect_field_names(val: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("FIELD") {
                if let Some(n) = map.get("name").and_then(|v| v.as_str()) {
                    out.push(n.to_string());
                }
            }
            for v in map.values() {
                out.extend(collect_field_names(v));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.extend(collect_field_names(v));
            }
        }
        _ => {}
    }
    out
}

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("non-empty", |s| !s.is_empty())
}

fn type_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}".prop_filter("non-empty", |s| !s.is_empty())
}

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}".prop_filter("avoid keywords", |s| {
        !matches!(
            s.as_str(),
            "type"
                | "fn"
                | "let"
                | "mut"
                | "ref"
                | "pub"
                | "mod"
                | "use"
                | "self"
                | "super"
                | "crate"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "where"
                | "for"
                | "loop"
                | "while"
                | "if"
                | "else"
                | "match"
                | "return"
                | "break"
                | "continue"
                | "as"
                | "in"
                | "move"
                | "box"
                | "dyn"
                | "async"
                | "await"
                | "try"
                | "yield"
                | "macro"
                | "const"
                | "static"
                | "unsafe"
                | "extern"
        )
    })
}

fn safe_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"[a-z]+".to_string()),
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        Just(r"[0-9]+".to_string()),
        Just(r"[a-f0-9]+".to_string()),
    ]
}

// ===========================================================================
// 1. Find grammar modules: single grammar module is discovered
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn single_grammar_module_discovered(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1, "exactly one grammar module should be discovered");
    }
}

// ===========================================================================
// 2. Find grammar modules: two distinct grammar modules produce two grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn two_grammar_modules_discovered(
        name1 in grammar_name_strategy(),
        name2 in grammar_name_strategy(),
        ty1 in type_name_strategy(),
        ty2 in type_name_strategy(),
    ) {
        let ty2_safe = if ty2 == ty1 { format!("{}Alt", ty2) } else { ty2 };
        let name2_safe = if name2 == name1 { format!("{}_b", name2) } else { name2 };
        let src = format!(
            r##"
            #[adze::grammar("{name1}")]
            mod grammar_a {{
                #[adze::language]
                pub struct {ty1} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            #[adze::grammar("{name2_safe}")]
            mod grammar_b {{
                #[adze::language]
                pub struct {ty2_safe} {{
                    #[adze::leaf(pattern = r"\d+")]
                    pub num: String,
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 2, "two grammar modules should produce two grammars");
    }
}

// ===========================================================================
// 3. Find grammar modules: module without grammar attr is ignored
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn module_without_grammar_attr_ignored(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            mod not_annotated {{
                pub struct Foo {{
                    pub x: i32,
                }}
            }}
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1, "only annotated module should be discovered");
    }
}

// ===========================================================================
// 4. Module name extraction: grammar name from attribute
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn grammar_name_from_attribute(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"\d+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 5. Module name extraction: two grammars have distinct names
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn two_grammars_have_distinct_names(
        name1 in grammar_name_strategy(),
        name2 in grammar_name_strategy(),
    ) {
        let name2_safe = if name2 == name1 { format!("{}_x", name2) } else { name2 };
        let src = format!(
            r##"
            #[adze::grammar("{name1}")]
            mod grammar_a {{
                #[adze::language]
                pub struct Root {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            #[adze::grammar("{name2_safe}")]
            mod grammar_b {{
                #[adze::language]
                pub struct Item {{
                    #[adze::leaf(pattern = r"\d+")]
                    pub num: String,
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 2);
        let n1 = gs[0]["name"].as_str().unwrap();
        let n2 = gs[1]["name"].as_str().unwrap();
        prop_assert_ne!(n1, n2, "grammar names must be distinct");
    }
}

// ===========================================================================
// 6. Module name extraction: name preserved in JSON output
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn grammar_name_preserved_in_json(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let json_str = serde_json::to_string(&g).unwrap();
        prop_assert!(json_str.contains(&name), "JSON should contain grammar name");
    }
}

// ===========================================================================
// 7. Module attribute detection: grammar attr is required
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn grammar_attr_required_for_discovery(_dummy in 0..1i32) {
        let src = r#"
            mod grammar {
                pub struct Foo {
                    pub x: i32,
                }
            }
        "#;
        let gs = extract(src);
        prop_assert_eq!(gs.len(), 0, "module without #[adze::grammar] should not be found");
    }
}

// ===========================================================================
// 8. Module attribute detection: language attr marks root type
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn language_attr_marks_root_type(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let sf = &g["rules"]["source_file"];
        prop_assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
        prop_assert_eq!(sf["name"].as_str().unwrap(), ty.as_str());
    }
}

// ===========================================================================
// 9. Module attribute detection: extra attr produces extras entry
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn extra_attr_produces_extras_entry(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {field}: String,
                }}
                #[adze::extra]
                struct Ws {{
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let extras = g["extras"].as_array().unwrap();
        prop_assert!(!extras.is_empty(), "extra attr should produce non-empty extras");
    }
}

// ===========================================================================
// 10. Struct discovery: single struct with one field
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn single_struct_discovery(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"{pat}")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&ty), "struct type should appear as rule");
    }
}

// ===========================================================================
// 11. Struct discovery: struct field names appear as FIELD nodes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn struct_fields_discovered(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let fields = collect_field_names(&g["rules"][&ty]);
        prop_assert!(
            fields.contains(&field),
            "field '{}' should be discovered in struct rule", field
        );
    }
}

// ===========================================================================
// 12. Struct discovery: multiple structs in one module
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn multiple_structs_discovered(
        name in grammar_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct Root {{
                    pub child: Child,
                }}
                pub struct Child {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&"Root".to_string()));
        prop_assert!(names.contains(&"Child".to_string()));
    }
}

// ===========================================================================
// 13. Struct discovery: struct patterns are preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn struct_patterns_preserved(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"{pat}")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let patterns = collect_typed_values(&g, "PATTERN");
        prop_assert!(
            patterns.contains(&pat),
            "pattern '{}' should be preserved in PATTERN node", pat
        );
    }
}

// ===========================================================================
// 14. Enum discovery: enum becomes CHOICE rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_discovered_as_choice(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    Alpha(#[adze::leaf(pattern = r"[a-z]+")] String),
                    Digit(#[adze::leaf(pattern = r"\d+")] String),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let enum_rule = &g["rules"][&ty];
        prop_assert_eq!(
            enum_rule["type"].as_str().unwrap(), "CHOICE",
            "enum should produce a CHOICE rule"
        );
    }
}

// ===========================================================================
// 15. Enum discovery: variant count matches member count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn enum_variant_count_matches(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    A(#[adze::leaf(pattern = r"[a-z]+")] String),
                    B(#[adze::leaf(pattern = r"\d+")] String),
                    C(#[adze::leaf(pattern = r"[A-Z]+")] String),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), 3, "3 variants should produce 3 CHOICE members");
    }
}

// ===========================================================================
// 16. Enum discovery: enum with child struct discovers both
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn enum_with_child_struct_discovers_both(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let child = format!("{}Item", ty);
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    Wrapped({child}),
                    Lit(#[adze::leaf(pattern = r"\d+")] String),
                }}
                pub struct {child} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&ty));
        prop_assert!(names.contains(&child));
    }
}

// ===========================================================================
// 17. Enum discovery: recursive enum reference
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn enum_recursive_reference(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    Leaf(#[adze::leaf(pattern = r"\d+")] String),
                    Wrapped(
                        #[adze::leaf(text = "(")]
                        (),
                        Box<{ty}>,
                        #[adze::leaf(text = ")")]
                        (),
                    ),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let json_str = serde_json::to_string(&g).unwrap();
        prop_assert!(
            json_str.contains(&format!(r#""name":"{}""#, ty))
                || json_str.contains(&format!(r#""name": "{}""#, ty)),
            "recursive SYMBOL reference to '{}' not found", ty
        );
    }
}

// ===========================================================================
// 18. Nested module handling: grammar inside outer module
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn grammar_inside_outer_module(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            mod outer {{
                #[adze::grammar("{name}")]
                mod grammar {{
                    #[adze::language]
                    pub struct {ty} {{
                        #[adze::leaf(pattern = r"[a-z]+")]
                        pub val: String,
                    }}
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1, "grammar nested inside outer module should be found");
        prop_assert_eq!(gs[0]["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 19. Nested module handling: deeply nested grammar still discovered
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn deeply_nested_grammar_discovered(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            mod level1 {{
                mod level2 {{
                    #[adze::grammar("{name}")]
                    mod grammar {{
                        #[adze::language]
                        pub struct {ty} {{
                            #[adze::leaf(pattern = r"\d+")]
                            pub val: String,
                        }}
                    }}
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1, "deeply nested grammar should be found");
    }
}

// ===========================================================================
// 20. Nested module handling: multiple grammars in nested modules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn multiple_grammars_in_nested_modules(
        name1 in grammar_name_strategy(),
        name2 in grammar_name_strategy(),
    ) {
        let name2_safe = if name2 == name1 { format!("{}_z", name2) } else { name2 };
        let src = format!(
            r##"
            mod outer_a {{
                #[adze::grammar("{name1}")]
                mod grammar {{
                    #[adze::language]
                    pub struct Alpha {{
                        #[adze::leaf(pattern = r"[a-z]+")]
                        pub val: String,
                    }}
                }}
            }}
            mod outer_b {{
                #[adze::grammar("{name2_safe}")]
                mod grammar {{
                    #[adze::language]
                    pub struct Beta {{
                        #[adze::leaf(pattern = r"\d+")]
                        pub num: String,
                    }}
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 2, "two nested grammars should produce two results");
    }
}

// ===========================================================================
// 21. Nested module handling: grammar sibling of plain module
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn grammar_sibling_of_plain_module(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            mod helpers {{
                pub fn add(a: i32, b: i32) -> i32 {{ a + b }}
            }}
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1, "only the grammar module should be discovered");
    }
}

// ===========================================================================
// 22. Module discovery determinism: same source yields same result
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn deterministic_module_discovery(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let a = extract_one(&src);
        let b = extract_one(&src);
        prop_assert_eq!(&a, &b, "module discovery must be deterministic");
    }
}

// ===========================================================================
// 23. Module discovery determinism: nested grammar is stable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn deterministic_nested_module_discovery(
        name in grammar_name_strategy(),
    ) {
        let src = format!(
            r##"
            mod outer {{
                #[adze::grammar("{name}")]
                mod grammar {{
                    #[adze::language]
                    pub struct Root {{
                        pub child: Child,
                    }}
                    pub struct Child {{
                        #[adze::leaf(pattern = r"[a-z]+")]
                        pub val: String,
                    }}
                }}
            }}
            "##,
        );
        let a = extract_one(&src);
        let b = extract_one(&src);
        prop_assert_eq!(&a, &b);
    }
}

// ===========================================================================
// 24. Module discovery determinism: multiple grammar order is stable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn deterministic_multi_grammar_order(_dummy in 0..1i32) {
        let src = r##"
            #[adze::grammar("alpha")]
            mod grammar_a {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }
            }
            #[adze::grammar("beta")]
            mod grammar_b {
                #[adze::language]
                pub struct Item {
                    #[adze::leaf(pattern = r"\d+")]
                    pub num: String,
                }
            }
        "##;
        let a = extract(src);
        let b = extract(src);
        prop_assert_eq!(a.len(), b.len());
        for i in 0..a.len() {
            prop_assert_eq!(&a[i], &b[i], "grammar at index {} must match", i);
        }
    }
}

// ===========================================================================
// 25. Empty module handling: empty file yields no grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn empty_file_yields_no_grammars(_dummy in 0..1i32) {
        let gs = extract("// nothing here\n");
        prop_assert_eq!(gs.len(), 0);
    }
}

// ===========================================================================
// 26. Empty module handling: module with only plain items yields nothing
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn plain_items_only_yields_nothing(_dummy in 0..1i32) {
        let src = r#"
            fn foo() -> i32 { 42 }
            struct Bar { x: i32 }
            const C: i32 = 0;
        "#;
        let gs = extract(src);
        prop_assert_eq!(gs.len(), 0);
    }
}

// ===========================================================================
// 27. Empty module handling: grammar module with minimal content
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn minimal_grammar_module_succeeds(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"x")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1);
        prop_assert!(gs[0]["rules"].as_object().unwrap().len() >= 2);
    }
}

// ===========================================================================
// 28. Struct discovery: struct with Optional field is discovered
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn struct_with_optional_field_discovered(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"\d+")]
                    pub val: Option<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&ty));
        prop_assert!(names.contains(&"source_file".to_string()));
    }
}

// ===========================================================================
// 29. Struct discovery: struct with Vec field is discovered
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn struct_with_vec_field_discovered(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::repeat(non_empty = true)]
                    #[adze::leaf(pattern = r"\d+")]
                    pub vals: Vec<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&ty));
    }
}

// ===========================================================================
// 30. Module attribute detection: pub mod also works
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn pub_mod_grammar_discovered(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            pub mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1);
        prop_assert_eq!(gs[0]["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 31. Enum discovery: single-variant enum produces rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn single_variant_enum_discovered(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    Only(#[adze::leaf(pattern = r"\d+")] String),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&ty));
    }
}

// ===========================================================================
// 32. Find grammar modules: grammar count matches annotation count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn grammar_count_matches_annotation_count(_dummy in 0..1i32) {
        let src = r##"
            #[adze::grammar("g1")]
            mod grammar_1 {
                #[adze::language]
                pub struct A {
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }
            }
            #[adze::grammar("g2")]
            mod grammar_2 {
                #[adze::language]
                pub struct B {
                    #[adze::leaf(pattern = r"\d+")]
                    pub num: String,
                }
            }
            #[adze::grammar("g3")]
            mod grammar_3 {
                #[adze::language]
                pub struct C {
                    #[adze::leaf(pattern = r"[A-Z]+")]
                    pub upper: String,
                }
            }
        "##;
        let gs = extract(src);
        prop_assert_eq!(gs.len(), 3, "three annotations should produce three grammars");
    }
}

// ===========================================================================
// 33. Nested module: non-grammar nested module does not pollute results
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn non_grammar_nested_module_ignored(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            mod outer {{
                mod inner_plain {{
                    pub struct NotAGrammar {{
                        pub x: i32,
                    }}
                }}
                #[adze::grammar("{name}")]
                mod grammar {{
                    #[adze::language]
                    pub struct {ty} {{
                        #[adze::leaf(pattern = r"[a-z]+")]
                        pub val: String,
                    }}
                }}
            }}
            "##,
        );
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1, "only the annotated module should produce a grammar");
    }
}
