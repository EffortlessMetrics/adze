#![allow(clippy::needless_range_loop)]

//! Property-based tests for the Rust-to-grammar converter in adze-tool.
//!
//! Validates invariants of `generate_grammars` when converting annotated Rust
//! source into Tree-sitter JSON grammars:
//!   - Conversion is deterministic
//!   - Grammar name matches the `#[adze::grammar("...")]` attribute
//!   - Rule count matches struct/enum count
//!   - Terminal tokens are all named
//!   - Extras array is always present
//!   - `source_file` rule references the root type
//!   - Conversion preserves field names

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
                | "gen"
                | "do"
                | "abstract"
                | "become"
                | "final"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
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

/// Recursively collect every JSON node whose "type" is "FIELD" and return
/// the associated "name" strings.
fn collect_field_names(val: &Value) -> Vec<String> {
    let mut names = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("FIELD")
                && let Some(n) = map.get("name").and_then(|v| v.as_str())
            {
                names.push(n.to_string());
            }
            for v in map.values() {
                names.extend(collect_field_names(v));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                names.extend(collect_field_names(v));
            }
        }
        _ => {}
    }
    names
}

/// Recursively collect every JSON node whose "type" is "STRING" and return
/// the "value" strings.
fn collect_string_tokens(val: &Value) -> Vec<String> {
    let mut tokens = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("STRING")
                && let Some(v) = map.get("value").and_then(|v| v.as_str())
            {
                tokens.push(v.to_string());
            }
            for v in map.values() {
                tokens.extend(collect_string_tokens(v));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                tokens.extend(collect_string_tokens(v));
            }
        }
        _ => {}
    }
    tokens
}

/// Recursively collect every JSON node whose "type" is "PATTERN" and return
/// the "value" strings.
fn collect_pattern_tokens(val: &Value) -> Vec<String> {
    let mut tokens = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("PATTERN")
                && let Some(v) = map.get("value").and_then(|v| v.as_str())
            {
                tokens.push(v.to_string());
            }
            for v in map.values() {
                tokens.extend(collect_pattern_tokens(v));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                tokens.extend(collect_pattern_tokens(v));
            }
        }
        _ => {}
    }
    tokens
}

/// Collect all rule names (keys in the top-level "rules" object).
fn rule_names(grammar: &Value) -> Vec<String> {
    grammar["rules"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

// ===========================================================================
// 1. Conversion is deterministic — struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn deterministic_struct_conversion(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let a = extract_one(&src);
        let b = extract_one(&src);
        prop_assert_eq!(&a, &b, "conversion must be deterministic");
    }
}

// ===========================================================================
// 2. Conversion is deterministic — enum
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn deterministic_enum_conversion(
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
// 3. Deterministic serialized JSON bytes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn deterministic_json_bytes(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let s1 = serde_json::to_string(&extract_one(&src)).unwrap();
        let s2 = serde_json::to_string(&extract_one(&src)).unwrap();
        prop_assert_eq!(s1, s2, "byte-level serialized form must match");
    }
}

// ===========================================================================
// 4. Grammar name matches #[adze::grammar("...")] — struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    #[test]
    fn grammar_name_matches_struct(
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
// 5. Grammar name matches — enum
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    #[test]
    fn grammar_name_matches_enum(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_source(&name, &ty, r"\d+");
        let g = extract_one(&src);
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 6. Rule count: struct with one leaf produces exactly source_file + root + leaf rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn struct_single_leaf_rule_count(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        // source_file + root type + leaf pattern rule = 3
        prop_assert_eq!(
            rules.len(), 3,
            "expected 3 rules (source_file + root + leaf), got {}: {:?}",
            rules.len(), rules.keys().collect::<Vec<_>>()
        );
    }
}

// ===========================================================================
// 7. Rule count: struct with child struct produces >= 4 rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn struct_with_child_rule_count(
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
        let rules = g["rules"].as_object().unwrap();
        // source_file + root + child + child's leaf = 4
        prop_assert!(
            rules.len() >= 4,
            "expected >= 4 rules, got {}: {:?}",
            rules.len(), rules.keys().collect::<Vec<_>>()
        );
    }
}

// ===========================================================================
// 8. Rule count: enum with N variants produces N variant-derived entries
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_two_variant_rule_count(
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
        let rules = g["rules"].as_object().unwrap();
        // source_file + enum type (CHOICE) + at least inline references
        prop_assert!(
            rules.len() >= 2,
            "expected >= 2 rules, got {}: {:?}",
            rules.len(), rules.keys().collect::<Vec<_>>()
        );
        // The enum type itself must exist as a CHOICE rule
        let enum_rule = &g["rules"][&ty];
        prop_assert_eq!(enum_rule["type"].as_str().unwrap(), "CHOICE");
    }
}

// ===========================================================================
// 9. Terminal tokens (PATTERN) are never empty
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn pattern_tokens_never_empty(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let patterns = collect_pattern_tokens(&g);
        for p in &patterns {
            prop_assert!(!p.is_empty(), "PATTERN token must not be empty");
        }
    }
}

// ===========================================================================
// 10. Terminal tokens (STRING) are never empty when from text
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn string_tokens_from_text_not_empty(
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
        let strings = collect_string_tokens(&g);
        for s in &strings {
            prop_assert!(!s.is_empty(), "STRING tokens from text must not be empty");
        }
    }
}

// ===========================================================================
// 11. All terminal rule keys are non-empty (named)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn all_rule_keys_are_named(
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
// 12. Extras array is always present — struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn extras_present_struct(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(g.get("extras").is_some(), "'extras' key must be present");
        prop_assert!(g["extras"].is_array(), "'extras' must be an array");
    }
}

// ===========================================================================
// 13. Extras array is always present — enum
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn extras_present_enum(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_source(&name, &ty, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(g.get("extras").is_some());
        prop_assert!(g["extras"].is_array());
    }
}

// ===========================================================================
// 14. Extras contains whitespace when declared
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn extras_contains_declared_whitespace(
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
        prop_assert!(!extras.is_empty(), "extras must include Whitespace");
        let has_ws = extras.iter().any(|e| {
            e.get("name").and_then(|n| n.as_str()) == Some("Whitespace")
        });
        prop_assert!(has_ws, "extras must reference Whitespace symbol");
    }
}

// ===========================================================================
// 15. source_file rule always exists — struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn source_file_exists_struct(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(
            g["rules"].as_object().unwrap().contains_key("source_file"),
            "source_file rule must exist"
        );
    }
}

// ===========================================================================
// 16. source_file rule always exists — enum
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn source_file_exists_enum(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_source(&name, &ty, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(g["rules"].as_object().unwrap().contains_key("source_file"));
    }
}

// ===========================================================================
// 17. source_file references root type — struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    #[test]
    fn source_file_refs_root_struct(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let sf = &g["rules"]["source_file"];
        prop_assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
        prop_assert_eq!(sf["name"].as_str().unwrap(), ty.as_str());
    }
}

// ===========================================================================
// 18. source_file references root type — enum
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    #[test]
    fn source_file_refs_root_enum(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let src = enum_source(&name, &ty, r"\d+");
        let g = extract_one(&src);
        let sf = &g["rules"]["source_file"];
        prop_assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
        prop_assert_eq!(sf["name"].as_str().unwrap(), ty.as_str());
    }
}

// ===========================================================================
// 19. Conversion preserves field names — struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn field_name_preserved_struct(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let fields = collect_field_names(&g);
        prop_assert!(
            fields.contains(&field),
            "field '{}' must appear in output, got {:?}",
            field, fields,
        );
    }
}

// ===========================================================================
// 20. Conversion preserves multiple field names
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn multiple_field_names_preserved(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2_name = if f2 == f1 { format!("{}_x", f2) } else { f2 };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {ty} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f1}: String,
                    #[adze::leaf(pattern = r"\d+")]
                    pub {f2_name}: String,
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let fields = collect_field_names(&g);
        prop_assert!(fields.contains(&f1), "field '{}' missing", f1);
        prop_assert!(fields.contains(&f2_name), "field '{}' missing", f2_name);
    }
}

// ===========================================================================
// 21. Root type appears in rules map
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn root_type_in_rules(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let names = rule_names(&g);
        prop_assert!(names.contains(&ty), "'{}' must be in rules", ty);
    }
}

// ===========================================================================
// 22. word field is null when no #[adze::word]
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn word_null_without_annotation(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(g["word"].is_null(), "word should be null when no #[adze::word]");
    }
}

// ===========================================================================
// 23. No externals when none declared
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn no_externals_by_default(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        prop_assert!(
            !g.as_object().unwrap().contains_key("externals"),
            "externals should not be present without #[adze::external]"
        );
    }
}

// ===========================================================================
// 24. Enum CHOICE has correct member count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn enum_choice_member_count(
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
        let rule = &g["rules"][&ty];
        prop_assert_eq!(rule["type"].as_str().unwrap(), "CHOICE");
        let members = rule["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), 3, "3 variants → 3 CHOICE members");
    }
}

// ===========================================================================
// 25. Optional field wraps in CHOICE with BLANK
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn optional_field_produces_choice_blank(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        f1 in field_name_strategy(),
    ) {
        let f2 = format!("{f1}_opt");
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
        let json_str = serde_json::to_string_pretty(&g).unwrap();
        // BLANK must appear somewhere in the output for the optional field
        prop_assert!(json_str.contains("BLANK"), "optional field must produce BLANK");
    }
}

// ===========================================================================
// 26. Top-level keys always include name, rules, extras, word
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn required_top_level_keys(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let obj = g.as_object().unwrap();
        for key in &["name", "rules", "extras", "word"] {
            prop_assert!(obj.contains_key(*key), "missing top-level key '{}'", key);
        }
    }
}

// ===========================================================================
// 27. Pattern leaf generates a PATTERN-typed rule entry
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn pattern_leaf_type_correct(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        // The leaf pattern rule key is "{ty}_{field}"
        let leaf_key = format!("{}_{}", ty, field);
        let leaf_rule = rules.get(&leaf_key);
        prop_assert!(leaf_rule.is_some(), "leaf rule '{}' must exist", leaf_key);
        let leaf_rule = leaf_rule.unwrap();
        prop_assert_eq!(leaf_rule["type"].as_str().unwrap(), "PATTERN");
        prop_assert_eq!(leaf_rule["value"].as_str().unwrap(), pat.as_str());
    }
}

// ===========================================================================
// 28. Text leaf generates a STRING-typed rule entry
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn text_leaf_type_correct(
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
        let rules = g["rules"].as_object().unwrap();
        let text_key = format!("{}_op", ty);
        let text_rule = rules.get(&text_key);
        prop_assert!(text_rule.is_some(), "text rule '{}' must exist", text_key);
        let text_rule = text_rule.unwrap();
        prop_assert_eq!(text_rule["type"].as_str().unwrap(), "STRING");
        prop_assert_eq!(text_rule["value"].as_str().unwrap(), tok.as_str());
    }
}

// ===========================================================================
// 29. prec_left wraps in PREC_LEFT node
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn prec_left_produces_prec_left_node(
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
                    #[adze::prec_left(1)]
                    Add(
                        Box<{ty}>,
                        #[adze::leaf(text = "+")]
                        (),
                        Box<{ty}>,
                    ),
                }}
            }}
            "##,
        );
        let g = extract_one(&src);
        let json_str = serde_json::to_string(&g).unwrap();
        prop_assert!(json_str.contains("PREC_LEFT"), "PREC_LEFT must appear in output");
    }
}

// ===========================================================================
// 30. Conversion roundtrips through JSON serialization
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn json_roundtrip(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        let json_str = serde_json::to_string(&g).unwrap();
        let reparsed: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&g, &reparsed);
    }
}

// ===========================================================================
// 31. Vec field generates REPEAT rule
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn vec_field_generates_repeat(
        name in grammar_name_strategy(),
        root in type_name_strategy(),
        child in type_name_strategy(),
    ) {
        let child_name = if child == root { format!("{}Item", child) } else { child };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {root} {{
                    pub items: Vec<{child_name}>,
                }}
                pub struct {child_name} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
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
        let json_str = serde_json::to_string(&g).unwrap();
        // REPEAT1 is used for vec contents
        prop_assert!(
            json_str.contains("REPEAT1"),
            "Vec field must generate a REPEAT1 rule"
        );
    }
}

// ===========================================================================
// 32. source_file is always the first rule key
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn source_file_is_first_rule(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, r"[a-z]+");
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        let first_key = rules.keys().next().unwrap();
        prop_assert_eq!(first_key, "source_file", "source_file must be the first rule");
    }
}

// ===========================================================================
// 33. Grammar output is always a JSON object
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn output_is_json_object(
        name in grammar_name_strategy(),
        ty in type_name_strategy(),
        field in field_name_strategy(),
        pat in safe_pattern_strategy(),
    ) {
        let src = struct_source(&name, &ty, &field, &pat);
        let g = extract_one(&src);
        prop_assert!(g.is_object(), "grammar output must be a JSON object");
    }
}
