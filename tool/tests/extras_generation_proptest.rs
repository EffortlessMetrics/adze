#![allow(clippy::needless_range_loop)]

//! Property-based tests for extras (whitespace/comments) generation in adze-tool.
//!
//! Uses proptest to validate invariants of the "extras" array in generated
//! grammar JSON produced by `adze_tool::generate_grammars`.

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

/// Return the "extras" array from a grammar value.
fn extras_array(g: &Value) -> &Vec<Value> {
    g["extras"].as_array().expect("extras should be an array")
}

/// Collect SYMBOL names from the extras array.
fn extras_names(g: &Value) -> Vec<String> {
    extras_array(g)
        .iter()
        .filter_map(|e| e["name"].as_str().map(String::from))
        .collect()
}

// ===========================================================================
// Strategies
// ===========================================================================

/// A valid Rust identifier safe for use as a grammar name.
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A PascalCase type name for extras struct names.
fn extra_type_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{2,8}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A safe regex pattern that can appear in a `#[adze::leaf(pattern = ...)]`.
fn safe_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"\s".to_string()),
        Just(r"\s+".to_string()),
        Just(r"[ \t]+".to_string()),
        Just(r"[ \t\r\n]+".to_string()),
        Just(r"\r?\n".to_string()),
    ]
}

/// Patterns that look like comment regexes.
fn comment_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"//[^\n]*".to_string()),
        Just(r"#[^\n]*".to_string()),
        Just(r";[^\n]*".to_string()),
        Just(r"--[^\n]*".to_string()),
    ]
}

/// Build grammar source with no extras.
fn grammar_no_extras(name: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct Root {{
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }}
        }}
        "##,
    )
}

/// Build grammar source with one extra.
fn grammar_one_extra(name: &str, extra_name: &str, pattern: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct Root {{
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }}

            #[adze::extra]
            struct {extra_name} {{
                #[adze::leaf(pattern = r"{pattern}")]
                _tok: (),
            }}
        }}
        "##,
    )
}

/// Build grammar source with two extras.
fn grammar_two_extras(
    name: &str,
    extra1: &str,
    pat1: &str,
    extra2: &str,
    pat2: &str,
) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct Root {{
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }}

            #[adze::extra]
            struct {extra1} {{
                #[adze::leaf(pattern = r"{pat1}")]
                _tok: (),
            }}

            #[adze::extra]
            struct {extra2} {{
                #[adze::leaf(pattern = r"{pat2}")]
                _tok2: (),
            }}
        }}
        "##,
    )
}

// ===========================================================================
// 1. Extras array always present in grammar JSON
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn extras_key_always_present_no_extras(
        name in grammar_name_strategy(),
    ) {
        let src = grammar_no_extras(&name);
        let g = extract_one(&src);
        prop_assert!(g.get("extras").is_some(), "extras key must always be present");
        prop_assert!(g["extras"].is_array(), "extras must be an array");
    }

    #[test]
    fn extras_key_always_present_with_extras(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        prop_assert!(g.get("extras").is_some(), "extras key must always be present");
        prop_assert!(g["extras"].is_array(), "extras must be an array");
    }
}

// ===========================================================================
// 2. Default extras (whitespace patterns)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn whitespace_extra_pattern_preserved(
        name in grammar_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, "Whitespace", &pattern);
        let g = extract_one(&src);
        let names = extras_names(&g);
        prop_assert!(names.contains(&"Whitespace".to_string()));
        // The pattern should appear somewhere in the rules (JSON-escaped)
        let rules_str = serde_json::to_string(&g["rules"]).unwrap();
        let json_escaped = serde_json::to_string(&pattern).unwrap();
        // Strip surrounding quotes from the JSON string
        let needle = &json_escaped[1..json_escaped.len() - 1];
        prop_assert!(rules_str.contains(needle), "pattern {pattern} not found in rules");
    }

    #[test]
    fn whitespace_extra_generates_rule(
        name in grammar_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, "Whitespace", &pattern);
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key("Whitespace"), "Whitespace rule must be generated");
    }

    #[test]
    fn whitespace_extras_are_symbol_type(
        name in grammar_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, "Whitespace", &pattern);
        let g = extract_one(&src);
        for entry in extras_array(&g) {
            prop_assert_eq!(entry["type"].as_str().unwrap(), "SYMBOL");
        }
    }
}

// ===========================================================================
// 3. Custom extras
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn custom_extra_name_appears_in_extras_array(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        let names = extras_names(&g);
        prop_assert!(
            names.contains(&extra),
            "extras array should contain {extra}, got {names:?}"
        );
    }

    #[test]
    fn custom_extra_generates_rule_entry(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key(&extra),
            "rules should contain key for custom extra {extra}"
        );
    }

    #[test]
    fn custom_extra_is_symbol_ref(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, r"\s");
        let g = extract_one(&src);
        let entry = extras_array(&g)
            .iter()
            .find(|e| e["name"].as_str() == Some(extra.as_str()));
        prop_assert!(entry.is_some(), "extras should contain {extra}");
        prop_assert_eq!(entry.unwrap()["type"].as_str().unwrap(), "SYMBOL");
    }
}

// ===========================================================================
// 4. Extras with comment patterns
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn comment_extra_appears_in_extras(
        name in grammar_name_strategy(),
        pattern in comment_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, "Comment", &pattern);
        let g = extract_one(&src);
        let names = extras_names(&g);
        prop_assert!(names.contains(&"Comment".to_string()));
    }

    #[test]
    fn comment_pattern_preserved_in_rules(
        name in grammar_name_strategy(),
        pattern in comment_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, "Comment", &pattern);
        let g = extract_one(&src);
        let rules_str = serde_json::to_string(&g["rules"]).unwrap();
        let json_escaped = serde_json::to_string(&pattern).unwrap();
        let needle = &json_escaped[1..json_escaped.len() - 1];
        prop_assert!(
            rules_str.contains(needle),
            "comment pattern should be in rules"
        );
    }

    #[test]
    fn comment_and_whitespace_extras_coexist(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        let src = grammar_two_extras(&name, "Whitespace", &ws_pat, "Comment", &cmt_pat);
        let g = extract_one(&src);
        let names = extras_names(&g);
        prop_assert!(names.contains(&"Whitespace".to_string()));
        prop_assert!(names.contains(&"Comment".to_string()));
    }
}

// ===========================================================================
// 5. Extras ordering — extras appear in source declaration order
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn extras_order_matches_source_order(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        let src = grammar_two_extras(&name, "Whitespace", &ws_pat, "Comment", &cmt_pat);
        let g = extract_one(&src);
        let names = extras_names(&g);
        // Whitespace is declared first in the source
        if let (Some(ws_idx), Some(cmt_idx)) = (
            names.iter().position(|n| n == "Whitespace"),
            names.iter().position(|n| n == "Comment"),
        ) {
            prop_assert!(ws_idx < cmt_idx, "Whitespace should come before Comment");
        }
    }

    #[test]
    fn reversed_declaration_order_reflected(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        // Declare Comment first, then Whitespace
        let src = grammar_two_extras(&name, "Comment", &cmt_pat, "Whitespace", &ws_pat);
        let g = extract_one(&src);
        let names = extras_names(&g);
        if let (Some(cmt_idx), Some(ws_idx)) = (
            names.iter().position(|n| n == "Comment"),
            names.iter().position(|n| n == "Whitespace"),
        ) {
            prop_assert!(cmt_idx < ws_idx, "Comment should come before Whitespace");
        }
    }
}

// ===========================================================================
// 6. Extras determinism — same input always gives same output
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn extras_deterministic_single(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(
            serde_json::to_string(&g1["extras"]).unwrap(),
            serde_json::to_string(&g2["extras"]).unwrap(),
            "extras must be deterministic"
        );
    }

    #[test]
    fn extras_deterministic_multiple(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        let src = grammar_two_extras(&name, "Whitespace", &ws_pat, "Comment", &cmt_pat);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(
            serde_json::to_string(&g1["extras"]).unwrap(),
            serde_json::to_string(&g2["extras"]).unwrap(),
        );
    }

    #[test]
    fn full_grammar_deterministic_with_extras(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        prop_assert_eq!(g1, g2, "full grammar must be deterministic");
    }
}

// ===========================================================================
// 7. No extras case — empty extras array
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn no_extras_yields_empty_array(
        name in grammar_name_strategy(),
    ) {
        let src = grammar_no_extras(&name);
        let g = extract_one(&src);
        let extras = extras_array(&g);
        prop_assert!(extras.is_empty(), "extras should be empty when none declared");
    }

    #[test]
    fn no_extras_still_has_valid_grammar(
        name in grammar_name_strategy(),
    ) {
        let src = grammar_no_extras(&name);
        let g = extract_one(&src);
        prop_assert!(g.get("name").is_some());
        prop_assert!(g.get("rules").is_some());
        prop_assert!(g.get("extras").is_some());
        prop_assert_eq!(g["name"].as_str().unwrap(), name.as_str());
    }

    #[test]
    fn no_extras_grammar_roundtrips(
        name in grammar_name_strategy(),
    ) {
        let src = grammar_no_extras(&name);
        let g = extract_one(&src);
        let json_str = serde_json::to_string(&g).unwrap();
        let reparsed: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&g["extras"], &reparsed["extras"]);
    }
}

// ===========================================================================
// 8. Multiple extras — count and uniqueness
// ===========================================================================

#[test]
fn three_extras_all_present() {
    let src = r#"
    #[adze::grammar("three_extras")]
    mod grammar {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
        }

        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _tok: (),
        }

        #[adze::extra]
        struct LineComment {
            #[adze::leaf(pattern = r"//[^\n]*")]
            _tok2: (),
        }

        #[adze::extra]
        struct Newline {
            #[adze::leaf(pattern = r"\r")]
            _tok3: (),
        }
    }
    "#;
    let g = extract_one(src);
    let names = extras_names(&g);
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"Whitespace".to_string()));
    assert!(names.contains(&"LineComment".to_string()));
    assert!(names.contains(&"Newline".to_string()));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn two_extras_yield_count_two(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        let src = grammar_two_extras(&name, "Whitespace", &ws_pat, "Comment", &cmt_pat);
        let g = extract_one(&src);
        let extras = extras_array(&g);
        prop_assert_eq!(extras.len(), 2, "should have exactly 2 extras");
    }

    #[test]
    fn extras_names_are_unique(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        let src = grammar_two_extras(&name, "Whitespace", &ws_pat, "Comment", &cmt_pat);
        let g = extract_one(&src);
        let names = extras_names(&g);
        let unique: HashSet<_> = names.iter().collect();
        prop_assert_eq!(names.len(), unique.len(), "extras names must be unique");
    }

    #[test]
    fn each_extra_has_corresponding_rule(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        let src = grammar_two_extras(&name, "Whitespace", &ws_pat, "Comment", &cmt_pat);
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        for extra_name in extras_names(&g) {
            prop_assert!(
                rules.contains_key(&extra_name),
                "rule missing for extra {extra_name}"
            );
        }
    }
}

// ===========================================================================
// 9. Extras entries structure invariants
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn every_extras_entry_has_type_and_name(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        for entry in extras_array(&g) {
            prop_assert!(entry.get("type").is_some(), "entry must have 'type'");
            prop_assert!(entry.get("name").is_some(), "entry must have 'name'");
        }
    }

    #[test]
    fn extras_do_not_appear_in_source_file_rule(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        // source_file should reference the language root, not any extra
        let sf = &g["rules"]["source_file"];
        let sf_str = serde_json::to_string(sf).unwrap();
        prop_assert!(
            !sf_str.contains(&extra),
            "source_file should not reference extra {extra}"
        );
    }

    #[test]
    fn extras_json_serialization_roundtrips(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        let json = serde_json::to_string(&g["extras"]).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g["extras"], &parsed);
    }
}
