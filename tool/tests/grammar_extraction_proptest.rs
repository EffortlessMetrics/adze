#![allow(clippy::needless_range_loop)]

//! Property-based tests for grammar extraction from Rust code in adze-tool.
//!
//! Tests the `generate_grammars` public API which parses annotated Rust source
//! files and produces Tree-sitter JSON grammar values. Covers:
//!   - Extract grammar from simple struct
//!   - Extract grammar from enum
//!   - Extract grammar from nested types
//!   - Grammar JSON output format
//!   - Grammar with multiple rules
//!   - Grammar extraction determinism
//!   - Empty grammar handling

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

fn _try_extract(src: &str) -> adze_tool::ToolResult<Vec<Value>> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path)
}

fn extract_one(src: &str) -> Value {
    let gs = extract(src);
    assert_eq!(gs.len(), 1, "expected 1 grammar, got {}", gs.len());
    gs.into_iter().next().unwrap()
}

/// Recursively collect every JSON node with a given "type" and return
/// the associated "value" strings.
fn collect_typed_values(val: &Value, type_name: &str) -> Vec<String> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some(type_name)
                && let Some(v) = map.get("value").and_then(|v| v.as_str())
            {
                out.push(v.to_string());
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

/// Recursively collect "name" values from FIELD nodes.
fn collect_field_names(val: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("FIELD")
                && let Some(n) = map.get("name").and_then(|v| v.as_str())
            {
                out.push(n.to_string());
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

fn rule_names(grammar: &Value) -> Vec<String> {
    grammar["rules"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

// ===========================================================================
// Keyword Checking
// ===========================================================================

/// Check if a string is a Rust keyword that would fail to parse as an identifier.
fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        // Keywords from the task specification
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn"
            | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl"
            | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub"
            | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
            | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while"
            | "yield" | "do" | "gen"
            // Reserved but not yet used keywords
            | "abstract" | "become" | "box" | "final" | "macro" | "override"
            | "priv" | "try" | "typeof" | "unsized" | "virtual"
    )
}

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}"
        .prop_filter("non-empty", |s| !s.is_empty())
        .prop_filter("not a keyword", |s| !is_rust_keyword(s))
}

fn type_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}"
        .prop_filter("non-empty", |s| !s.is_empty())
        .prop_filter("not a keyword", |s| !is_rust_keyword(s))
}

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}".prop_filter("avoid keywords", |s| !is_rust_keyword(s))
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

fn text_token_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("+".to_string()),
        Just("-".to_string()),
        Just("*".to_string()),
        Just("=".to_string()),
        Just(";".to_string()),
        Just(",".to_string()),
    ]
}

fn struct_source(name: &str, ty: &str, field: &str, pattern: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct {ty} {{
                #[adze::leaf(pattern = r"{pattern}")]
                pub {field}: String,
            }}
        }}
        "##,
    )
}

fn enum_source(name: &str, ty: &str, pattern: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub enum {ty} {{
                Leaf(
                    #[adze::leaf(pattern = r"{pattern}")]
                    String
                ),
            }}
        }}
        "##,
    )
}

// ===========================================================================
// 1. Simple struct: extraction succeeds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn struct_extraction_succeeds(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 1);
    }
}

// ===========================================================================
// 2. Simple struct: grammar name matches attribute
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn struct_grammar_name_matches(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 3. Simple struct: root type appears in source_file rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn struct_source_file_references_root(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"\d+");
        let g = extract_one(&src);
        let sf = &g["rules"]["source_file"];
        prop_assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
        prop_assert_eq!(sf["name"].as_str().unwrap(), ty.as_str());
    }
}

// ===========================================================================
// 4. Simple struct: field name preserved as FIELD in JSON
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn struct_field_name_preserved(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let fields = collect_field_names(&g["rules"][&ty]);
        prop_assert!(
            fields.contains(&field),
            "field '{}' not found in FIELD nodes: {:?}", field, fields
        );
    }
}

// ===========================================================================
// 5. Simple struct: pattern value preserved in PATTERN node
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn struct_pattern_preserved(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let patterns = collect_typed_values(&g, "PATTERN");
        prop_assert!(
            patterns.contains(&pat),
            "pattern '{}' not in PATTERN nodes: {:?}", pat, patterns
        );
    }
}

// ===========================================================================
// 6. Enum: extraction produces CHOICE rule for root type
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn enum_produces_choice_rule(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_source(&name, &ty, r"\d+");
        let g = extract_one(&src);
        let enum_rule = &g["rules"][&ty];
        prop_assert_eq!(
            enum_rule["type"].as_str().unwrap(), "CHOICE",
            "enum root must be CHOICE"
        );
    }
}

// ===========================================================================
// 7. Enum: grammar name matches attribute
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn enum_grammar_name_matches(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_source(&name, &ty, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 8. Enum: two-variant enum has 2 CHOICE members
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_two_variants_two_members(
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
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), 2);
    }
}

// ===========================================================================
// 9. Enum: single-variant enum still produces CHOICE
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_single_variant_is_choice(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_source(&name, &ty, r"\d+");
        let g = extract_one(&src);
        let enum_rule = &g["rules"][&ty];
        prop_assert_eq!(enum_rule["type"].as_str().unwrap(), "CHOICE");
        prop_assert!(!enum_rule["members"].as_array().unwrap().is_empty());
    }
}

// ===========================================================================
// 10. Enum: recursive Box<Self> reference produces SYMBOL
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn enum_recursive_ref(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    Num(#[adze::leaf(pattern = r"\d+")] String),
                    Neg(
                        #[adze::leaf(text = "-")]
                        (),
                        Box<{ty}>
                    ),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let json_str = serde_json::to_string(&g).unwrap();
        // The recursive reference should produce a SYMBOL node pointing at the enum
        prop_assert!(
            json_str.contains(&format!(r#""name":"{}""#, ty))
                || json_str.contains(&format!(r#""name": "{}""#, ty)),
            "recursive SYMBOL reference to '{}' not found", ty
        );
    }
}

// ===========================================================================
// 11. Nested types: struct referencing child struct creates both rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn nested_struct_creates_both_rules(
        name in grammar_name_strategy(),
        root in type_name_strategy(),
        child in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let child_name = if child == root { format!("{}Child", child) } else { child };
        let f2_name = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {root} {{
                    pub {f1}: {child_name},
                }}
                pub struct {child_name} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f2_name}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&root), "root '{}' not in rules", root);
        prop_assert!(names.contains(&child_name), "child '{}' not in rules", child_name);
    }
}

// ===========================================================================
// 12. Nested types: enum referencing child struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn nested_enum_with_child_struct(
        name in grammar_name_strategy(),
        root in type_name_strategy(),
        child in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let child_name = if child == root { format!("{}Item", child) } else { child };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {root} {{
                    Wrapped({child_name}),
                    Lit(#[adze::leaf(pattern = r"\d+")] String),
                }}
                pub struct {child_name} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&root));
        prop_assert!(names.contains(&child_name));
    }
}

// ===========================================================================
// 13. Nested types: three-level nesting produces all rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn three_level_nesting(name in grammar_name_strategy()) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct Root {{
                    pub mid: Middle,
                }}
                pub struct Middle {{
                    pub leaf: Leaf,
                }}
                pub struct Leaf {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&"Root".to_string()));
        prop_assert!(names.contains(&"Middle".to_string()));
        prop_assert!(names.contains(&"Leaf".to_string()));
    }
}

// ===========================================================================
// 14. Nested types: multiple children all present in rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn multiple_children_all_present(name in grammar_name_strategy()) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct Root {{
                    pub a: ChildA,
                    pub b: ChildB,
                }}
                pub struct ChildA {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
                pub struct ChildB {{
                    #[adze::leaf(pattern = r"\d+")]
                    pub num: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        for expected in &["Root", "ChildA", "ChildB"] {
            prop_assert!(
                names.contains(&expected.to_string()),
                "'{}' not in rules: {:?}", expected, names
            );
        }
    }
}

// ===========================================================================
// 15. JSON format: "name" key is a string
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn json_name_is_string(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(g["name"].is_string());
    }
}

// ===========================================================================
// 16. JSON format: "rules" key is an object
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn json_rules_is_object(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"\d+");
        let g = extract_one(&src);
        prop_assert!(g["rules"].is_object(), "'rules' must be an object");
    }
}

// ===========================================================================
// 17. JSON format: "extras" key is always an array
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn json_extras_is_array(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(g.get("extras").is_some(), "'extras' key must exist");
        prop_assert!(g["extras"].is_array(), "'extras' must be an array");
    }
}

// ===========================================================================
// 18. JSON format: "word" key is present
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn json_word_key_present(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(g.get("word").is_some(), "'word' key must be present");
    }
}

// ===========================================================================
// 19. JSON format: source_file always first rule key
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn json_source_file_is_first_rule(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let first_key = g["rules"].as_object().unwrap().keys().next().unwrap();
        prop_assert_eq!(first_key, "source_file");
    }
}

// ===========================================================================
// 20. JSON format: all rule keys are non-empty strings
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn json_rule_keys_nonempty(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        for key in rule_names(&g) {
            prop_assert!(!key.is_empty(), "rule key must be non-empty");
        }
    }
}

// ===========================================================================
// 21. Multiple rules: struct + extra produces extras entry
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn struct_with_extra_has_extras(
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
                struct Whitespace {{
                    #[adze::leaf(pattern = r"\s")]
                    _ws: (),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let extras = g["extras"].as_array().unwrap();
        prop_assert!(!extras.is_empty(), "extras should contain whitespace entry");
    }
}

// ===========================================================================
// 22. Multiple rules: enum + sibling struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn enum_with_sibling_struct(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let child = format!("{}Num", ty);
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    Wrapped({child}),
                    Lit(#[adze::leaf(pattern = r"[a-z]+")] String),
                }}
                pub struct {child} {{
                    #[adze::leaf(pattern = r"\d+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&ty));
        prop_assert!(names.contains(&child));
        prop_assert!(names.contains(&"source_file".to_string()));
    }
}

// ===========================================================================
// 23. Multiple rules: rule count scales with type count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn rule_count_scales(name in grammar_name_strategy()) {
        let src_one = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct Root {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let src_two = format!(
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
        let g1 = extract_one(&src_one);
        let g2 = extract_one(&src_two);
        let count1 = g1["rules"].as_object().unwrap().len();
        let count2 = g2["rules"].as_object().unwrap().len();
        prop_assert!(
            count2 > count1,
            "adding a child type should increase rule count: {} vs {}", count1, count2
        );
    }
}

// ===========================================================================
// 24. Multiple rules: struct with text token produces STRING node
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn struct_text_token_produces_string(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        tok in text_token_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(text = "{tok}")]
                    pub op: (),
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let strings = collect_typed_values(&g, "STRING");
        prop_assert!(
            strings.contains(&tok),
            "text token '{}' not found in STRING nodes: {:?}", tok, strings
        );
    }
}

// ===========================================================================
// 25. Determinism: same struct source extracts identically
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn deterministic_struct_extraction(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let a = extract_one(&src);
        let b = extract_one(&src);
        prop_assert_eq!(&a, &b, "extraction must be deterministic");
    }
}

// ===========================================================================
// 26. Determinism: same enum source extracts identically
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn deterministic_enum_extraction(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = enum_source(&name, &ty, &pat);
        let a = extract_one(&src);
        let b = extract_one(&src);
        prop_assert_eq!(&a, &b);
    }
}

// ===========================================================================
// 27. Determinism: serialized JSON bytes match across runs
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn deterministic_json_bytes(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let s1 = serde_json::to_string(&extract_one(&src)).unwrap();
        let s2 = serde_json::to_string(&extract_one(&src)).unwrap();
        prop_assert_eq!(s1, s2, "serialized JSON must be byte-identical");
    }
}

// ===========================================================================
// 28. Determinism: nested grammar extraction is stable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn deterministic_nested_extraction(name in grammar_name_strategy()) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct Root {{
                    pub mid: Middle,
                }}
                pub struct Middle {{
                    pub leaf: Leaf,
                }}
                pub struct Leaf {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
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
// 29. Empty grammar: no grammar attribute yields empty list
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn no_grammar_attr_yields_empty(_dummy in 0..1i32) {
        let src = r#"
            mod not_a_grammar {
                pub struct Foo {
                    pub x: i32,
                }
            }
        "#;
        let gs = extract(src);
        prop_assert_eq!(gs.len(), 0, "no #[adze::grammar] should yield 0 grammars");
    }
}

// ===========================================================================
// 30. Empty grammar: file with no modules yields empty list
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn empty_file_yields_empty(_dummy in 0..1i32) {
        let gs = extract("// empty file\n");
        prop_assert_eq!(gs.len(), 0);
    }
}

// ===========================================================================
// 31. Empty grammar: minimal struct (single field) succeeds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn minimal_struct_succeeds(
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
        prop_assert!(g["rules"].as_object().unwrap().len() >= 2);
    }
}

// ===========================================================================
// 32. PATTERN tokens are never empty strings
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn pattern_tokens_never_empty(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let patterns = collect_typed_values(&g, "PATTERN");
        for p in &patterns {
            prop_assert!(!p.is_empty(), "PATTERN must not be empty");
        }
    }
}

// ===========================================================================
// 33. Enum with Optional field
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn enum_optional_field_extracts(
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
        prop_assert!(names.contains(&"source_file".to_string()));
        prop_assert!(names.contains(&ty));
    }
}
