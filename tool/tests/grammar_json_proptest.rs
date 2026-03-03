#![allow(clippy::needless_range_loop)]

//! Property-based tests for grammar JSON output in adze-tool.
//!
//! Uses proptest to validate invariants of the JSON grammar produced by
//! `adze_tool::generate_grammars`:
//!   - Output is always valid, parseable JSON
//!   - Grammar name is reflected in the output
//!   - Required top-level keys are always present
//!   - Multiple rules yield valid JSON
//!   - Generation is deterministic
//!   - Special characters in names are handled gracefully
//!   - Various complexity levels produce well-formed output

use proptest::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

/// Write Rust source to a temp file and extract grammars via the public API.
fn extract(src: &str) -> Vec<Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

/// Extract exactly one grammar.
fn extract_one(src: &str) -> Value {
    let gs = extract(src);
    assert_eq!(gs.len(), 1, "expected exactly one grammar, got {}", gs.len());
    gs.into_iter().next().unwrap()
}

// ===========================================================================
// Strategies
// ===========================================================================

/// A valid Rust identifier that is also a safe Tree-sitter grammar name.
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A valid Rust type-name (PascalCase).
fn type_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A valid Rust field name (snake_case, not a keyword).
fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}".prop_filter("avoid keywords", |s| {
        !matches!(
            s.as_str(),
            "type" | "fn" | "let" | "mut" | "ref" | "pub" | "mod" | "use" | "self" | "super"
                | "crate" | "struct" | "enum" | "impl" | "trait" | "where" | "for" | "loop"
                | "while" | "if" | "else" | "match" | "return" | "break" | "continue" | "as"
                | "in" | "move" | "box" | "dyn" | "async" | "await" | "try" | "yield"
                | "macro" | "const" | "static" | "unsafe" | "extern"
        )
    })
}

/// A regex pattern that is safe to embed in a Rust raw string.
fn safe_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"[a-z]+".to_string()),
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        Just(r"[0-9]+".to_string()),
        Just(r"[a-f0-9]+".to_string()),
    ]
}

/// A literal text token.
fn text_token_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("+".to_string()),
        Just("-".to_string()),
        Just("*".to_string()),
        Just("=".to_string()),
        Just(";".to_string()),
        Just(",".to_string()),
        Just(":".to_string()),
        Just("(".to_string()),
        Just(")".to_string()),
    ]
}

/// Build a minimal struct-based grammar source string.
fn struct_grammar_source(name: &str, type_name: &str, field: &str, pattern: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct {type_name} {{
                #[adze::leaf(pattern = r"{pattern}")]
                pub {field}: String,
            }}
        }}
        "##,
    )
}

/// Build an enum-based grammar source string with one variant.
fn enum_grammar_source(name: &str, type_name: &str, pattern: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub enum {type_name} {{
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
// 1. Generated JSON is always valid JSON
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn json_roundtrips_through_serialization(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        // Serialize to string and re-parse — must roundtrip.
        let json_str = serde_json::to_string(&grammar).unwrap();
        let reparsed: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&grammar, &reparsed);
    }

    #[test]
    fn pretty_json_is_also_valid(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        let pretty = serde_json::to_string_pretty(&grammar).unwrap();
        let reparsed: Value = serde_json::from_str(&pretty).unwrap();
        prop_assert_eq!(&grammar, &reparsed);
    }

    #[test]
    fn enum_grammar_produces_valid_json(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = enum_grammar_source(&name, &type_name, &pattern);
        let grammar = extract_one(&src);
        let json_str = serde_json::to_string(&grammar).unwrap();
        let _: Value = serde_json::from_str(&json_str).unwrap();
    }
}

// ===========================================================================
// 2. Grammar name appears in output
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn name_field_matches_input(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    #[test]
    fn enum_name_field_matches(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = enum_grammar_source(&name, &type_name, r"\d+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 3. Rules section is always present
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn rules_key_always_present_struct(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        prop_assert!(grammar.get("rules").is_some(), "missing 'rules' key");
        prop_assert!(grammar["rules"].is_object(), "'rules' must be an object");
    }

    #[test]
    fn rules_key_always_present_enum(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = enum_grammar_source(&name, &type_name, &pattern);
        let grammar = extract_one(&src);
        prop_assert!(grammar.get("rules").is_some());
        prop_assert!(grammar["rules"].is_object());
    }

    #[test]
    fn source_file_rule_always_exists(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key("source_file"),
            "source_file rule must always exist"
        );
    }

    #[test]
    fn source_file_references_root_type(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let sf = &grammar["rules"]["source_file"];
        prop_assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
        prop_assert_eq!(sf["name"].as_str().unwrap(), type_name.as_str());
    }
}

// ===========================================================================
// 4. Multiple rules produce valid JSON
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn struct_with_child_struct_valid_json(
        name in grammar_name_strategy(),
        root in type_name_strategy(),
        child in type_name_strategy().prop_filter("differ from root", |s| s.len() > 2),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        // Ensure names differ to avoid collision.
        let child_name = if child == root {
            format!("{}Child", child)
        } else {
            child
        };
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
        let grammar = extract_one(&src);
        let json_str = serde_json::to_string(&grammar).unwrap();
        let _: Value = serde_json::from_str(&json_str).unwrap();
        let rules = grammar["rules"].as_object().unwrap();
        // Should have at least source_file + root + child
        prop_assert!(rules.len() >= 3, "expected >=3 rules, got {}", rules.len());
    }

    #[test]
    fn enum_with_multiple_variants_valid_json(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {type_name} {{
                    Alpha(
                        #[adze::leaf(pattern = r"[a-z]+")]
                        String
                    ),
                    Digit(
                        #[adze::leaf(pattern = r"\d+")]
                        String
                    ),
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let json_str = serde_json::to_string(&grammar).unwrap();
        let reparsed: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&grammar, &reparsed);
    }
}

// ===========================================================================
// 5. JSON is deterministic (same input → same output)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn deterministic_struct_grammar(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(&g1, &g2, "generation must be deterministic");
    }

    #[test]
    fn deterministic_enum_grammar(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = enum_grammar_source(&name, &type_name, &pattern);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(&g1, &g2);
    }

    #[test]
    fn deterministic_serialized_form(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let g1 = serde_json::to_string(&extract_one(&src)).unwrap();
        let g2 = serde_json::to_string(&extract_one(&src)).unwrap();
        prop_assert_eq!(g1, g2, "serialized JSON must be byte-identical");
    }
}

// ===========================================================================
// 6. Special characters in names are handled
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn grammar_name_with_underscores(
        prefix in "[a-z]{1,4}",
        suffix in "[a-z]{1,4}",
    ) {
        let name = format!("{prefix}_{suffix}");
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    #[test]
    fn grammar_name_with_digits(
        base in "[a-z]{1,4}",
        num in 0u16..999,
    ) {
        let name = format!("{base}{num}");
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    #[test]
    fn type_name_variations_produce_valid_json(
        type_name in type_name_strategy(),
    ) {
        let src = struct_grammar_source("test_tn", &type_name, "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert!(grammar["rules"].as_object().unwrap().contains_key(&type_name));
    }
}

// ===========================================================================
// 7. Various grammar complexity levels
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn grammar_with_text_leaf(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        tok in text_token_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {type_name} {{
                    #[adze::leaf(text = "{tok}")]
                    pub op: (),
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub val: String,
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        prop_assert!(grammar["rules"].is_object());
        let json_str = serde_json::to_string(&grammar).unwrap();
        let _: Value = serde_json::from_str(&json_str).unwrap();
    }

    #[test]
    fn grammar_with_extra_whitespace(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {type_name} {{
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
        let grammar = extract_one(&src);
        let extras = grammar["extras"].as_array().unwrap();
        prop_assert!(!extras.is_empty(), "extras must include the Whitespace rule");
    }

    #[test]
    fn grammar_with_optional_field(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        f1 in field_name_strategy(),
    ) {
        let f2 = format!("{f1}_opt");
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {type_name} {{
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub {f1}: String,
                    #[adze::leaf(pattern = r"\d+")]
                    pub {f2}: Option<String>,
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let json_str = serde_json::to_string(&grammar).unwrap();
        let _: Value = serde_json::from_str(&json_str).unwrap();
    }

    #[test]
    fn grammar_with_repeat_field(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        child in type_name_strategy(),
    ) {
        let child_name = if child == type_name {
            format!("{}Item", child)
        } else {
            child
        };
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {type_name} {{
                    #[adze::repeat(non_empty = true)]
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
        let grammar = extract_one(&src);
        prop_assert!(grammar["rules"].is_object());
        let json_str = serde_json::to_string(&grammar).unwrap();
        let _: Value = serde_json::from_str(&json_str).unwrap();
    }

    #[test]
    fn grammar_with_prec_left(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {type_name} {{
                    Num(
                        #[adze::leaf(pattern = r"\d+")]
                        String
                    ),
                    #[adze::prec_left(1)]
                    Add(
                        Box<{type_name}>,
                        #[adze::leaf(text = "+")]
                        (),
                        Box<{type_name}>,
                    ),
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let json_str = serde_json::to_string(&grammar).unwrap();
        let _: Value = serde_json::from_str(&json_str).unwrap();
    }
}

// ===========================================================================
// 8. Structural invariants
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn top_level_keys_always_present(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let obj = grammar.as_object().unwrap();
        prop_assert!(obj.contains_key("name"), "missing 'name'");
        prop_assert!(obj.contains_key("rules"), "missing 'rules'");
        prop_assert!(obj.contains_key("extras"), "missing 'extras'");
        prop_assert!(obj.contains_key("word"), "missing 'word'");
    }

    #[test]
    fn extras_is_always_array(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert!(grammar["extras"].is_array(), "extras must be an array");
    }

    #[test]
    fn word_field_is_null_when_no_word_rule(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        // Without an explicit #[adze::word], the word field should be null.
        prop_assert!(grammar["word"].is_null(), "word should be null without #[adze::word]");
    }

    #[test]
    fn rules_always_has_at_least_two_entries(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        // source_file + the root type = at least 2 entries.
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(
            rules.len() >= 2,
            "expected at least 2 rules (source_file + root), got {}",
            rules.len()
        );
    }

    #[test]
    fn root_type_rule_exists(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key(&type_name),
            "root type '{}' should be in rules",
            type_name
        );
    }

    #[test]
    fn enum_choice_has_members(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {type_name} {{
                    A(#[adze::leaf(pattern = r"[a-z]+")] String),
                    B(#[adze::leaf(pattern = r"\d+")] String),
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let rule = &grammar["rules"][&type_name];
        prop_assert_eq!(rule["type"].as_str().unwrap(), "CHOICE");
        let members = rule["members"].as_array().unwrap();
        prop_assert_eq!(members.len(), 2, "expected 2 CHOICE members");
    }
}

// ===========================================================================
// 9. No externals by default
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn no_externals_when_none_declared(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let obj = grammar.as_object().unwrap();
        // externals key should be absent when none declared.
        prop_assert!(
            !obj.contains_key("externals"),
            "externals should not be present when none declared"
        );
    }
}
