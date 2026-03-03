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
    assert_eq!(
        gs.len(),
        1,
        "expected exactly one grammar, got {}",
        gs.len()
    );
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
fn grammar_two_extras(name: &str, extra1: &str, pat1: &str, extra2: &str, pat2: &str) -> String {
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

// ===========================================================================
// 10. Block comment extras
// ===========================================================================

#[test]
fn block_comment_extra_appears_in_extras() {
    let src = r#"
    #[adze::grammar("block_cmt")]
    mod grammar {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
        }

        #[adze::extra]
        struct BlockComment {
            #[adze::leaf(pattern = r"/\*[^*]*\*/")]
            _tok: (),
        }
    }
    "#;
    let g = extract_one(src);
    let names = extras_names(&g);
    assert!(names.contains(&"BlockComment".to_string()));
}

#[test]
fn line_and_block_comment_extras_coexist() {
    let src = r#"
    #[adze::grammar("both_cmt")]
    mod grammar {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
        }

        #[adze::extra]
        struct LineComment {
            #[adze::leaf(pattern = r"//[^\n]*")]
            _tok: (),
        }

        #[adze::extra]
        struct BlockComment {
            #[adze::leaf(pattern = r"/\*[^*]*\*/")]
            _tok2: (),
        }
    }
    "#;
    let g = extract_one(src);
    let names = extras_names(&g);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"LineComment".to_string()));
    assert!(names.contains(&"BlockComment".to_string()));
}

// ===========================================================================
// 11. Extras interaction with enum rules
// ===========================================================================

#[test]
fn extras_with_enum_language_rule() {
    let src = r#"
    #[adze::grammar("enum_extras")]
    mod grammar {
        #[adze::language]
        pub enum Expression {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                i32
            ),
        }

        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _whitespace: (),
        }
    }
    "#;
    let g = extract_one(src);
    let names = extras_names(&g);
    assert!(names.contains(&"Whitespace".to_string()));
    // The enum rule should not reference the extra
    let expr_rule = serde_json::to_string(&g["rules"]["Expression"]).unwrap();
    assert!(!expr_rule.contains("Whitespace"));
}

#[test]
fn extras_not_included_in_enum_choice_members() {
    let src = r#"
    #[adze::grammar("enum_choice")]
    mod grammar {
        #[adze::language]
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
            Id(#[adze::leaf(pattern = r"[a-z]+")] String),
        }

        #[adze::extra]
        struct Ws {
            #[adze::leaf(pattern = r"\s+")]
            _w: (),
        }
    }
    "#;
    let g = extract_one(src);
    let expr = &g["rules"]["Expr"];
    let members = expr["members"].as_array().unwrap();
    // None of the choice members should reference the extra
    for member in members {
        let member_str = serde_json::to_string(member).unwrap();
        assert!(
            !member_str.contains("\"Ws\""),
            "enum choice should not reference extra Ws"
        );
    }
}

// ===========================================================================
// 12. Extras interaction with repeat/vec rules
// ===========================================================================

#[test]
fn extras_with_repeat_rule() {
    let src = r#"
    #[adze::grammar("repeat_extras")]
    mod grammar {
        #[adze::language]
        pub struct NumberList {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            numbers: Vec<Number>,
        }

        pub struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            v: i32,
        }

        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _whitespace: (),
        }
    }
    "#;
    let g = extract_one(src);
    let names = extras_names(&g);
    assert!(names.contains(&"Whitespace".to_string()));
    // Repeat rules should exist alongside extras
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("NumberList"));
    assert!(rules.contains_key("Number"));
    assert!(rules.contains_key("Whitespace"));
}

// ===========================================================================
// 13. Extras interaction with precedence rules
// ===========================================================================

#[test]
fn extras_with_prec_left_rule() {
    let src = r#"
    #[adze::grammar("prec_extras")]
    mod grammar {
        #[adze::language]
        pub enum Expression {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                i32
            ),
            #[adze::prec_left(1)]
            Sub(
                Box<Expression>,
                #[adze::leaf(text = "-", transform = |v| ())]
                (),
                Box<Expression>
            ),
        }

        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    }
    "#;
    let g = extract_one(src);
    let names = extras_names(&g);
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"Whitespace".to_string()));
    // The prec rule should still be generated
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expression"));
}

// ===========================================================================
// 14. Extra rule internal structure
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn extra_rule_contains_field_wrapper(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        let rule = &g["rules"][&extra];
        // The extra struct rule wraps its content in a FIELD
        prop_assert_eq!(
            rule["type"].as_str().unwrap(),
            "FIELD",
            "extra rule should be a FIELD wrapper"
        );
    }

    #[test]
    fn extra_inner_pattern_rule_exists(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        let rules = g["rules"].as_object().unwrap();
        // The inner pattern rule is named like "ExtraName__tok"
        let inner_key = format!("{extra}__tok");
        prop_assert!(
            rules.contains_key(&inner_key),
            "inner pattern rule {inner_key} should exist in rules"
        );
    }

    #[test]
    fn extra_inner_rule_is_pattern_type(
        name in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = grammar_one_extra(&name, &extra, &pattern);
        let g = extract_one(&src);
        let inner_key = format!("{extra}__tok");
        let inner_rule = &g["rules"][&inner_key];
        prop_assert_eq!(
            inner_rule["type"].as_str().unwrap(),
            "PATTERN",
            "inner extra rule should be a PATTERN"
        );
    }
}

// ===========================================================================
// 15. Extras grammar name independence
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    #[test]
    fn extras_independent_of_grammar_name(
        name1 in grammar_name_strategy(),
        name2 in grammar_name_strategy(),
        extra in extra_type_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let g1 = extract_one(&grammar_one_extra(&name1, &extra, &pattern));
        let g2 = extract_one(&grammar_one_extra(&name2, &extra, &pattern));
        // Extras array should be identical regardless of grammar name
        prop_assert_eq!(
            serde_json::to_string(&g1["extras"]).unwrap(),
            serde_json::to_string(&g2["extras"]).unwrap(),
        );
    }

    #[test]
    fn extras_count_invariant_across_runs(
        name in grammar_name_strategy(),
        ws_pat in safe_pattern_strategy(),
        cmt_pat in comment_pattern_strategy(),
    ) {
        let src = grammar_two_extras(&name, "Whitespace", &ws_pat, "Comment", &cmt_pat);
        let counts: Vec<usize> = (0..3)
            .map(|_| extras_array(&extract_one(&src)).len())
            .collect();
        prop_assert!(counts.iter().all(|&c| c == counts[0]));
    }
}

// ===========================================================================
// 16. Extras with word rule interaction
// ===========================================================================

#[test]
fn extras_alongside_word_rule() {
    let src = r#"
    #[adze::grammar("word_extras")]
    mod grammar {
        #[adze::language]
        pub struct Root {
            ident: Identifier,
        }

        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }

        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    }
    "#;
    let g = extract_one(src);
    // word rule should be set
    assert_eq!(g["word"].as_str().unwrap(), "Identifier");
    // extras should still work
    let names = extras_names(&g);
    assert!(names.contains(&"Whitespace".to_string()));
}

// ===========================================================================
// 17. Grammar JSON top-level structure with extras
// ===========================================================================

#[test]
fn grammar_json_has_required_keys_with_extras() {
    let src = r#"
    #[adze::grammar("json_keys")]
    mod grammar {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
        }

        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    }
    "#;
    let g = extract_one(src);
    let obj = g.as_object().unwrap();
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("rules"));
    assert!(obj.contains_key("extras"));
    assert!(obj.contains_key("word"));
}

#[test]
fn extras_do_not_duplicate_on_repeated_generation() {
    let src = r#"
    #[adze::grammar("dedup")]
    mod grammar {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
        }

        #[adze::extra]
        struct Ws {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    }
    "#;
    let g1 = extract_one(src);
    let g2 = extract_one(src);
    assert_eq!(extras_array(&g1).len(), extras_array(&g2).len());
    assert_eq!(extras_array(&g1).len(), 1);
}
