#![allow(clippy::needless_range_loop)]

//! Property-based tests for rule generation in adze-tool.
//!
//! Validates that annotated Rust types are correctly converted into
//! Tree-sitter grammar rule nodes: SEQ, CHOICE, REPEAT, FIELD, OPTIONAL,
//! STRING, PATTERN, PREC, SYMBOL — with determinism and naming conventions.

use proptest::prelude::*;
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

/// Recursively collect every JSON node whose "type" equals `type_name`.
fn collect_nodes_of_type(val: &Value, type_name: &str) -> Vec<Value> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some(type_name) {
                out.push(val.clone());
            }
            for v in map.values() {
                out.extend(collect_nodes_of_type(v, type_name));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.extend(collect_nodes_of_type(v, type_name));
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

fn text_token_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("+".to_string()),
        Just("-".to_string()),
        Just("*".to_string()),
        Just("=".to_string()),
        Just(";".to_string()),
        Just(",".to_string()),
        Just("(".to_string()),
        Just(")".to_string()),
    ]
}

// ===========================================================================
// Source-generation helpers
// ===========================================================================

fn struct_single_field_src(name: &str, ty: &str, field: &str, pattern: &str) -> String {
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

fn struct_two_field_src(name: &str, ty: &str, f1: &str, p1: &str, f2: &str, p2: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct {ty} {{
                #[adze::leaf(pattern = r"{p1}")]
                pub {f1}: String,
                #[adze::leaf(pattern = r"{p2}")]
                pub {f2}: String,
            }}
        }}
        "##,
    )
}

fn enum_two_variant_src(name: &str, ty: &str) -> String {
    format!(
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
    )
}

// ===========================================================================
// 1. Struct with one field → FIELD rule (not SEQ)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn struct_single_field_produces_field_rule(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let root = &g["rules"][&ty];
        prop_assert_eq!(root["type"].as_str().unwrap(), "FIELD");
    }
}

// ===========================================================================
// 2. Struct with two fields → SEQ rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn struct_two_fields_produce_seq(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = struct_two_field_src(&name, &ty, &f1, r"[a-z]+", &f2, r"\d+");
        let g = extract_one(&src);
        let root = &g["rules"][&ty];
        prop_assert_eq!(root["type"].as_str().unwrap(), "SEQ");
        let members = root["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), 2);
    }
}

// ===========================================================================
// 3. SEQ members are all FIELD nodes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn seq_members_are_fields(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = struct_two_field_src(&name, &ty, &f1, r"[a-z]+", &f2, r"\d+");
        let g = extract_one(&src);
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        for m in members {
            prop_assert_eq!(m["type"].as_str().unwrap(), "FIELD");
        }
    }
}

// ===========================================================================
// 4. SEQ preserves field declaration order
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn seq_preserves_field_order(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = struct_two_field_src(&name, &ty, &f1, r"[a-z]+", &f2, r"\d+");
        let g = extract_one(&src);
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        prop_assert_eq!(members[0]["name"].as_str().unwrap(), f1.as_str());
        prop_assert_eq!(members[1]["name"].as_str().unwrap(), f2.as_str());
    }
}

// ===========================================================================
// 5. Enum produces CHOICE rule type
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_produces_choice(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_two_variant_src(&name, &ty);
        let g = extract_one(&src);
        prop_assert_eq!(g["rules"][&ty]["type"].as_str().unwrap(), "CHOICE");
    }
}

// ===========================================================================
// 6. Enum CHOICE has correct member count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_choice_member_count(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_two_variant_src(&name, &ty);
        let g = extract_one(&src);
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), 2);
    }
}

// ===========================================================================
// 7. Enum variant with named fields + prec creates intermediate rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn enum_prec_variant_creates_intermediate_rule(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    #[adze::prec_left(1)]
                    Add {{
                        left: Box<{ty}>,
                        #[adze::leaf(text = "+")]
                        op: String,
                        right: Box<{ty}>,
                    }},
                    Lit(#[adze::leaf(pattern = r"\d+")] i32),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let names = rule_names(&g);
        let variant_name = format!("{ty}_Add");
        prop_assert!(
            names.contains(&variant_name),
            "expected intermediate rule '{}' in {:?}", variant_name, names
        );
    }
}

// ===========================================================================
// 8. SEQ inside PREC for multi-field prec variant
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prec_variant_content_is_seq(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    #[adze::prec_left(2)]
                    Mul {{
                        left: Box<{ty}>,
                        #[adze::leaf(text = "*")]
                        op: String,
                        right: Box<{ty}>,
                    }},
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let variant = &g["rules"][format!("{ty}_Mul")];
        prop_assert_eq!(variant["type"].as_str().unwrap(), "PREC_LEFT");
        prop_assert_eq!(variant["content"]["type"].as_str().unwrap(), "SEQ");
    }
}

// ===========================================================================
// 9. Vec field produces REPEAT1 contents rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn vec_field_produces_repeat1(
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
                    pub {field}: Vec<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let contents_key = format!("{ty}_{field}_vec_contents");
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key(&contents_key),
            "expected '{}' in {:?}", contents_key, rules.keys().collect::<Vec<_>>()
        );
        prop_assert_eq!(rules[&contents_key]["type"].as_str().unwrap(), "REPEAT1");
    }
}

// ===========================================================================
// 10. Default Vec wraps reference in CHOICE for empty
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn vec_default_wraps_in_choice(
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
                    pub {field}: Vec<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let root = &g["rules"][&ty];
        let content = &root["content"];
        prop_assert_eq!(content["type"].as_str().unwrap(), "CHOICE");
        let members = content["members"].as_array().unwrap();
        prop_assert!(members.iter().any(|c| c["type"].as_str() == Some("BLANK")));
    }
}

// ===========================================================================
// 11. Vec with non_empty=true references SYMBOL directly
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn vec_non_empty_is_direct_symbol(
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
                    #[adze::leaf(pattern = r"\d+")]
                    #[adze::repeat(non_empty = true)]
                    pub {field}: Vec<i32>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let content = &g["rules"][&ty]["content"];
        prop_assert_eq!(
            content["type"].as_str().unwrap(), "SYMBOL",
            "non_empty Vec should produce direct SYMBOL"
        );
    }
}

// ===========================================================================
// 12. Option field wraps in CHOICE with BLANK
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn option_field_choice_with_blank(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f1}: String,
                    #[adze::leaf(pattern = r"\d+")]
                    pub {f2}: Option<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        let opt = &members[1];
        prop_assert_eq!(opt["type"].as_str().unwrap(), "CHOICE");
        let choices = opt["members"].as_array().unwrap();
        prop_assert!(choices.iter().any(|c| c["type"].as_str() == Some("BLANK")));
        prop_assert!(choices.iter().any(|c| c["type"].as_str() == Some("FIELD")));
    }
}

// ===========================================================================
// 13. Option CHOICE has exactly two members
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn option_choice_has_two_members(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f1}: String,
                    #[adze::leaf(pattern = r"\d+")]
                    pub {f2}: Option<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        let opt_choices = members[1]["members"].as_array().unwrap();
        prop_assert_eq!(opt_choices.len(), 2);
    }
}

// ===========================================================================
// 14. FIELD "content" references generated SYMBOL
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn field_content_is_symbol(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let root = &g["rules"][&ty];
        prop_assert_eq!(root["content"]["type"].as_str().unwrap(), "SYMBOL");
    }
}

// ===========================================================================
// 15. FIELD content SYMBOL name follows Type_field convention
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn field_symbol_name_convention(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let expected = format!("{}_{}", ty, field);
        let actual = g["rules"][&ty]["content"]["name"].as_str().unwrap();
        prop_assert_eq!(actual, expected.as_str());
    }
}

// ===========================================================================
// 16. Pattern leaf generates PATTERN rule in rules map
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn pattern_leaf_creates_pattern_rule(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let rule_key = format!("{}_{}", ty, field);
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key(&rule_key));
        prop_assert_eq!(rules[&rule_key]["type"].as_str().unwrap(), "PATTERN");
        prop_assert_eq!(rules[&rule_key]["value"].as_str().unwrap(), pat.as_str());
    }
}

// ===========================================================================
// 17. Text leaf generates STRING rule in rules map
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn text_leaf_creates_string_rule(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        tok in text_token_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(text = "{tok}")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let rule_key = format!("{}_{}", ty, field);
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key(&rule_key));
        prop_assert_eq!(rules[&rule_key]["type"].as_str().unwrap(), "STRING");
        prop_assert_eq!(rules[&rule_key]["value"].as_str().unwrap(), tok.as_str());
    }
}

// ===========================================================================
// 18. Determinism: same source produces identical JSON
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn rule_generation_is_deterministic(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, &pat);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(g1, g2, "same source must produce identical grammar JSON");
    }
}

// ===========================================================================
// 19. Determinism: enum source produces identical JSON
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_rule_generation_is_deterministic(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_two_variant_src(&name, &ty);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(g1, g2);
    }
}

// ===========================================================================
// 20. source_file always first key and references language type
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn source_file_references_language_type(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let first_key = g["rules"].as_object().unwrap().keys().next().unwrap();
        prop_assert_eq!(first_key, "source_file");
        prop_assert_eq!(g["rules"]["source_file"]["name"].as_str().unwrap(), ty.as_str());
    }
}

// ===========================================================================
// 21. Rule naming: all generated rule keys are non-empty
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn rule_keys_are_nonempty(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        for key in rule_names(&g) {
            prop_assert!(!key.is_empty(), "rule key must be non-empty");
        }
    }
}

// ===========================================================================
// 22. Rule naming: variant rules follow EnumType_VariantName pattern
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn variant_rule_naming_convention(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    #[adze::prec_left(1)]
                    BinOp {{
                        left: Box<{ty}>,
                        #[adze::leaf(text = "+")]
                        op: String,
                        right: Box<{ty}>,
                    }},
                    Lit(#[adze::leaf(pattern = r"\d+")] i32),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let variant_key = format!("{ty}_BinOp");
        let names = rule_names(&g);
        prop_assert!(names.contains(&variant_key));
    }
}

// ===========================================================================
// 23. Vec contents rule follows Type_field_vec_contents pattern
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn vec_contents_naming_convention(
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
                    pub {field}: Vec<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let expected = format!("{ty}_{field}_vec_contents");
        let names = rule_names(&g);
        prop_assert!(
            names.contains(&expected),
            "'{}' not in {:?}", expected, names
        );
    }
}

// ===========================================================================
// 24. PREC_LEFT wraps with correct value
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prec_left_value_preserved(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        prec_val in 0u32..100,
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    #[adze::prec_left({prec_val})]
                    Op {{
                        left: Box<{ty}>,
                        #[adze::leaf(text = "+")]
                        op: String,
                        right: Box<{ty}>,
                    }},
                    Lit(#[adze::leaf(pattern = r"\d+")] i32),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let variant = &g["rules"][format!("{ty}_Op")];
        prop_assert_eq!(variant["type"].as_str().unwrap(), "PREC_LEFT");
        prop_assert_eq!(variant["value"].as_u64().unwrap(), prec_val as u64);
    }
}

// ===========================================================================
// 25. PREC_RIGHT wraps with correct value
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prec_right_value_preserved(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        prec_val in 0u32..100,
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    #[adze::prec_right({prec_val})]
                    Assign {{
                        target: Box<{ty}>,
                        #[adze::leaf(text = "=")]
                        eq: String,
                        value: Box<{ty}>,
                    }},
                    Id(#[adze::leaf(pattern = r"[a-z]+")] String),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let variant = &g["rules"][format!("{ty}_Assign")];
        prop_assert_eq!(variant["type"].as_str().unwrap(), "PREC_RIGHT");
        prop_assert_eq!(variant["value"].as_u64().unwrap(), prec_val as u64);
    }
}

// ===========================================================================
// 26. PREC (non-associative) wraps with correct value
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prec_plain_value_preserved(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        prec_val in 0u32..100,
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    #[adze::prec({prec_val})]
                    Paren {{
                        #[adze::leaf(text = "(")]
                        open: String,
                        inner: Box<{ty}>,
                        #[adze::leaf(text = ")")]
                        close: String,
                    }},
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let variant = &g["rules"][format!("{ty}_Paren")];
        prop_assert_eq!(variant["type"].as_str().unwrap(), "PREC");
        prop_assert_eq!(variant["value"].as_u64().unwrap(), prec_val as u64);
    }
}

// ===========================================================================
// 27. Nested struct creates SYMBOL reference
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn nested_struct_creates_symbol_ref(
        name in grammar_name_strategy(),
        root in type_name_strategy(),
        child in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let child = if child == root { format!("{child}Child") } else { child };
        let f2 = if f2 == f1 { format!("{f2}_b") } else { f2 };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {root} {{
                    pub {f1}: {child},
                }}
                pub struct {child} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f2}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let root_rule = &g["rules"][&root];
        prop_assert_eq!(root_rule["content"]["type"].as_str().unwrap(), "SYMBOL");
        prop_assert_eq!(root_rule["content"]["name"].as_str().unwrap(), child.as_str());
    }
}

// ===========================================================================
// 28. All rule types are valid Tree-sitter types
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn all_rule_types_are_valid(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{f2}_b") } else { f2 };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f1}: String,
                    #[adze::leaf(pattern = r"\d+")]
                    pub {f2}: Option<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let valid_types = [
            "SEQ", "CHOICE", "REPEAT", "REPEAT1", "FIELD", "SYMBOL",
            "STRING", "PATTERN", "PREC", "PREC_LEFT", "PREC_RIGHT", "BLANK",
            "TOKEN", "IMMEDIATE_TOKEN", "ALIAS",
        ];
        let all_nodes = collect_nodes_of_type(&g["rules"], "SEQ")
            .into_iter()
            .chain(collect_nodes_of_type(&g["rules"], "CHOICE"))
            .chain(collect_nodes_of_type(&g["rules"], "FIELD"))
            .chain(collect_nodes_of_type(&g["rules"], "SYMBOL"))
            .chain(collect_nodes_of_type(&g["rules"], "STRING"))
            .chain(collect_nodes_of_type(&g["rules"], "PATTERN"))
            .chain(collect_nodes_of_type(&g["rules"], "BLANK"));
        for node in all_nodes {
            let ty_str = node["type"].as_str().unwrap();
            prop_assert!(
                valid_types.contains(&ty_str),
                "unexpected rule type: {}", ty_str
            );
        }
    }
}

// ===========================================================================
// 29. Grammar name in output matches grammar attribute
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn grammar_name_matches_attribute(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_single_field_src(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 30. REPEAT1 content inside vec is always a FIELD node
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn repeat1_content_is_field(
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
                    pub {field}: Vec<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let contents_key = format!("{ty}_{field}_vec_contents");
        let contents = &g["rules"][&contents_key];
        prop_assert_eq!(contents["content"]["type"].as_str().unwrap(), "FIELD");
    }
}

// ===========================================================================
// 31. REPEAT1 element field follows naming convention
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn repeat1_element_field_naming(
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
                    pub {field}: Vec<String>,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let contents_key = format!("{ty}_{field}_vec_contents");
        let expected_element = format!("{ty}_{field}_vec_element");
        let inner = &g["rules"][&contents_key]["content"];
        prop_assert_eq!(inner["name"].as_str().unwrap(), expected_element.as_str());
    }
}

// ===========================================================================
// 32. Three-field struct SEQ has exactly 3 members
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn three_field_seq_has_three_members(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(text = "(")]
                    pub open: String,
                    #[adze::leaf(pattern = r"\d+")]
                    pub val: String,
                    #[adze::leaf(text = ")")]
                    pub close: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let rule = &g["rules"][&ty];
        prop_assert_eq!(rule["type"].as_str().unwrap(), "SEQ");
        prop_assert_eq!(rule["members"].as_array().unwrap().len(), 3);
    }
}

// ===========================================================================
// 33. Inline single-leaf enum variants produce PATTERN/STRING directly
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn inline_single_leaf_variant_type(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {ty} {{
                    Word(#[adze::leaf(pattern = r"[a-z]+")] String),
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let members = g["rules"][&ty]["members"].as_array().unwrap();
        for m in members {
            let mt = m["type"].as_str().unwrap();
            prop_assert!(
                mt == "PATTERN" || mt == "SYMBOL" || mt == "STRING",
                "inline variant should be PATTERN/STRING/SYMBOL, got: {}", mt
            );
        }
    }
}

// ===========================================================================
// 34. Field names from struct are all present in FIELD nodes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn all_field_names_present(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{f2}_b") } else { f2 };
        let src = struct_two_field_src(&name, &ty, &f1, r"[a-z]+", &f2, r"\d+");
        let g = extract_one(&src);
        let field_names = collect_field_names(&g["rules"][&ty]);
        prop_assert!(
            field_names.contains(&f1),
            "field '{}' not found in {:?}", f1, field_names
        );
        prop_assert!(
            field_names.contains(&f2),
            "field '{}' not found in {:?}", f2, field_names
        );
    }
}

// ===========================================================================
// 35. Determinism: complex grammar with Vec + Option
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn complex_grammar_determinism(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2 = if f2 == f1 { format!("{f2}_b") } else { f2 };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f1}: Vec<String>,
                    #[adze::leaf(pattern = r"\d+")]
                    pub {f2}: Option<String>,
                }}
            }}
            "##,
        );
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(
            serde_json::to_string(&g1).unwrap(),
            serde_json::to_string(&g2).unwrap(),
            "complex grammar must produce identical JSON across runs"
        );
    }
}
