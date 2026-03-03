#![allow(clippy::needless_range_loop)]

//! Property-based tests for grammar name derivation in adze-tool.
//!
//! Validates invariants around how grammar names flow through the pipeline:
//!   - Grammar name extracted from module attribute
//!   - Grammar name in output JSON
//!   - Grammar name in generated code references
//!   - Grammar name casing preservation
//!   - Grammar name with underscores
//!   - Grammar name with numbers
//!   - Grammar name determinism
//!   - Grammar name uniqueness across modules

use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
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

/// Build a two-module source string with distinct grammar names.
fn two_grammar_source(name1: &str, name2: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name1}")]
        mod grammar1 {{
            #[adze::language]
            pub struct Root1 {{
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }}
        }}

        #[adze::grammar("{name2}")]
        mod grammar2 {{
            #[adze::language]
            pub struct Root2 {{
                #[adze::leaf(pattern = r"[a-z]+")]
                pub val: String,
            }}
        }}
        "##,
    )
}

// ===========================================================================
// Strategies
// ===========================================================================

/// A valid lowercase grammar name (safe for Tree-sitter).
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

/// A regex pattern safe for embedding in Rust raw strings.
fn safe_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"[a-z]+".to_string()),
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        Just(r"[0-9]+".to_string()),
        Just(r"[a-f0-9]+".to_string()),
    ]
}

/// Grammar names that contain underscores in various positions.
fn underscore_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]{1,4}_[a-z]{1,4}",
        "[a-z]{1,3}_[a-z]{1,3}_[a-z]{1,3}",
        "[a-z]{1,4}_[a-z0-9]{1,4}",
    ]
}

/// Grammar names that contain digits.
fn numeric_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]{1,4}[0-9]{1,3}",
        "[a-z]{1,3}[0-9]{1,2}[a-z]{1,3}",
        "[a-z]{1,4}_[0-9]{1,3}",
    ]
}

// ===========================================================================
// 1. Grammar name from module name — name extracted from attribute
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// The "name" field in the JSON must exactly equal the string literal
    /// passed to #[adze::grammar("...")].
    #[test]
    fn name_field_equals_attribute_value_struct(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    /// Same property for enum-based grammars.
    #[test]
    fn name_field_equals_attribute_value_enum(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = enum_grammar_source(&name, &type_name, r"\d+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    /// The name field is always a JSON string (not null, number, etc.).
    #[test]
    fn name_field_is_always_a_string(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert!(grammar["name"].is_string(), "name must be a JSON string");
    }
}

// ===========================================================================
// 2. Grammar name in output JSON — structural presence
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// The "name" key is always present at the top level of the grammar JSON.
    #[test]
    fn json_always_contains_name_key(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        let obj = grammar.as_object().unwrap();
        prop_assert!(obj.contains_key("name"), "top-level 'name' key must exist");
    }

    /// The grammar name survives JSON serialization round-trip.
    #[test]
    fn name_survives_json_roundtrip(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let json_str = serde_json::to_string(&grammar).unwrap();
        let reparsed: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(
            reparsed["name"].as_str().unwrap(),
            name.as_str(),
            "name must survive serialization round-trip"
        );
    }

    /// The grammar name appears verbatim in the pretty-printed JSON output.
    #[test]
    fn name_appears_in_pretty_json(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let pretty = serde_json::to_string_pretty(&grammar).unwrap();
        prop_assert!(
            pretty.contains(&format!("\"name\": \"{}\"", name)),
            "pretty JSON must contain the grammar name"
        );
    }
}

// ===========================================================================
// 3. Grammar name in generated code — name propagates to rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// source_file rule always references the root type, independent of the
    /// grammar name. The grammar name does not leak into rule names.
    #[test]
    fn grammar_name_does_not_leak_into_rule_names(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let rules = grammar["rules"].as_object().unwrap();
        for (rule_name, _) in rules.iter() {
            prop_assert!(
                rule_name != &name || rule_name == "source_file",
                "grammar name '{}' should not appear as a rule name (found '{}')",
                name,
                rule_name,
            );
        }
    }

    /// The root type rule is keyed by the type name, not the grammar name.
    #[test]
    fn root_type_keyed_by_type_not_grammar_name(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key(&type_name),
            "rules should contain key '{}' (type name), not '{}'",
            type_name,
            name,
        );
    }

    /// source_file SYMBOL references the root type regardless of grammar name.
    #[test]
    fn source_file_references_type_not_grammar_name(
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
// 4. Grammar name casing — preserved exactly as given
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// Lowercase names are preserved without modification.
    #[test]
    fn lowercase_name_preserved(
        name in "[a-z]{2,10}",
    ) {
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        let out = grammar["name"].as_str().unwrap();
        prop_assert_eq!(out, name.as_str());
        prop_assert!(out.chars().all(|c| c.is_ascii_lowercase()));
    }

    /// Mixed alpha-numeric names preserve exact casing and digit positions.
    #[test]
    fn alphanumeric_casing_preserved(
        name in "[a-z][a-z0-9]{1,8}",
    ) {
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    /// Names with mixed underscores preserve exact character sequence.
    #[test]
    fn underscore_casing_preserved(
        name in underscore_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }
}

// ===========================================================================
// 5. Grammar name with underscores
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// Single underscore separators are preserved.
    #[test]
    fn single_underscore_separator_preserved(
        prefix in "[a-z]{1,5}",
        suffix in "[a-z]{1,5}",
    ) {
        let name = format!("{prefix}_{suffix}");
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
        prop_assert!(name.contains('_'));
    }

    /// Multiple underscore segments are all preserved.
    #[test]
    fn multiple_underscore_segments_preserved(
        a in "[a-z]{1,3}",
        b in "[a-z]{1,3}",
        c in "[a-z]{1,3}",
    ) {
        let name = format!("{a}_{b}_{c}");
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
        prop_assert_eq!(name.matches('_').count(), 2);
    }

    /// Underscore names produce valid JSON structure (not just name check).
    #[test]
    fn underscore_names_produce_valid_grammar(
        name in underscore_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let grammar = extract_one(&src);
        let json_str = serde_json::to_string(&grammar).unwrap();
        let reparsed: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&grammar, &reparsed);
        prop_assert!(grammar["rules"].is_object());
    }
}

// ===========================================================================
// 6. Grammar name with numbers
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// Trailing digits are preserved exactly.
    #[test]
    fn trailing_digits_preserved(
        base in "[a-z]{1,5}",
        num in 0u16..9999,
    ) {
        let name = format!("{base}{num}");
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    /// Interleaved digits are preserved.
    #[test]
    fn interleaved_digits_preserved(
        a in "[a-z]{1,3}",
        d in "[0-9]{1,2}",
        b in "[a-z]{1,3}",
    ) {
        let name = format!("{a}{d}{b}");
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    /// Digits after underscore are preserved.
    #[test]
    fn digits_after_underscore_preserved(
        base in "[a-z]{1,4}",
        num in 0u16..999,
    ) {
        let name = format!("{base}_{num}");
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let grammar = extract_one(&src);
        prop_assert_eq!(grammar["name"].as_str().unwrap(), name.as_str());
    }

    /// Numeric names produce structurally valid grammars.
    #[test]
    fn numeric_names_produce_valid_grammar(
        name in numeric_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = enum_grammar_source(&name, &type_name, r"\d+");
        let grammar = extract_one(&src);
        prop_assert!(grammar["rules"].is_object());
        prop_assert!(grammar.as_object().unwrap().contains_key("name"));
        let json_str = serde_json::to_string(&grammar).unwrap();
        let _: Value = serde_json::from_str(&json_str).unwrap();
    }
}

// ===========================================================================
// 7. Grammar name determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Identical source produces byte-identical grammar name in JSON.
    #[test]
    fn deterministic_name_struct(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(
            g1["name"].as_str().unwrap(),
            g2["name"].as_str().unwrap(),
            "grammar name must be deterministic"
        );
    }

    /// Enum grammars also produce deterministic names.
    #[test]
    fn deterministic_name_enum(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = enum_grammar_source(&name, &type_name, &pattern);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(
            g1["name"].as_str().unwrap(),
            g2["name"].as_str().unwrap(),
        );
    }

    /// The full serialized JSON is byte-identical across runs.
    #[test]
    fn deterministic_full_json(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let s1 = serde_json::to_string(&extract_one(&src)).unwrap();
        let s2 = serde_json::to_string(&extract_one(&src)).unwrap();
        prop_assert_eq!(s1, s2, "serialized JSON must be byte-identical");
    }

    /// Determinism holds for underscore-bearing names.
    #[test]
    fn deterministic_underscore_names(
        name in underscore_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(g1, g2);
    }

    /// Determinism holds for numeric names.
    #[test]
    fn deterministic_numeric_names(
        name in numeric_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, "Root", "val", r"[a-z]+");
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(g1, g2);
    }
}

// ===========================================================================
// 8. Grammar name uniqueness — distinct names yield distinct grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Two grammars with different names produce different "name" fields.
    #[test]
    fn distinct_names_yield_distinct_name_fields(
        a in "[a-z]{2,6}",
        b in "[a-z]{2,6}",
    ) {
        prop_assume!(a != b);
        let src = two_grammar_source(&a, &b);
        let gs = extract(&src);
        prop_assert_eq!(gs.len(), 2, "expected two grammars");
        let names: Vec<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
        prop_assert_ne!(names[0], names[1], "different grammar names must produce different name fields");
    }

    /// Changing only the grammar name (same types/fields) produces a
    /// different grammar JSON.
    #[test]
    fn name_change_alters_output(
        a in grammar_name_strategy(),
        b in grammar_name_strategy(),
    ) {
        prop_assume!(a != b);
        let src_a = struct_grammar_source(&a, "Root", "val", r"[a-z]+");
        let src_b = struct_grammar_source(&b, "Root", "val", r"[a-z]+");
        let ga = extract_one(&src_a);
        let gb = extract_one(&src_b);
        prop_assert_ne!(ga, gb, "different grammar names must produce different JSON");
    }

    /// Grammar names from a two-module file are a proper set (no duplicates).
    #[test]
    fn two_grammar_names_form_a_set(
        a in "[a-z]{2,6}",
        b in "[a-z]{2,6}",
    ) {
        prop_assume!(a != b);
        let src = two_grammar_source(&a, &b);
        let gs = extract(&src);
        let name_set: HashSet<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
        prop_assert_eq!(name_set.len(), 2, "both names must be distinct in output");
    }

    /// Swapping grammar names swaps the "name" fields in output.
    #[test]
    fn swapped_names_swap_output(
        a in "[a-z]{2,6}",
        b in "[a-z]{2,6}",
    ) {
        prop_assume!(a != b);
        let src1 = two_grammar_source(&a, &b);
        let src2 = two_grammar_source(&b, &a);
        let gs1 = extract(&src1);
        let gs2 = extract(&src2);
        let names1: HashSet<&str> = gs1.iter().map(|g| g["name"].as_str().unwrap()).collect();
        let names2: HashSet<&str> = gs2.iter().map(|g| g["name"].as_str().unwrap()).collect();
        prop_assert_eq!(names1, names2, "swapping names should produce the same set of names");
    }
}
