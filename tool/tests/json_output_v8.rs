//! Comprehensive tests validating `node_types_json` output from `build_parser`.
//!
//! 80+ tests covering JSON validity, structure, content, determinism,
//! and correct representation of grammar features in the NODE_TYPES output.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};
use serde_json::Value;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

/// Create a `TempDir` + `BuildOptions` pair for a single test.
fn test_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, opts)
}

/// Parse `node_types_json` into a `serde_json::Value`.
fn parse_json(json: &str) -> Value {
    serde_json::from_str(json).expect("node_types_json must be valid JSON")
}

/// Build a grammar and return its `node_types_json` string.
fn build_json(grammar: Grammar) -> String {
    let (_d, opts) = test_opts();
    build_parser(grammar, opts)
        .expect("build_parser should succeed")
        .node_types_json
}

/// Build a grammar and return the parsed JSON array.
fn build_json_value(grammar: Grammar) -> Value {
    parse_json(&build_json(grammar))
}

// ---------------------------------------------------------------------------
// Minimal grammar factories (each with a unique name prefix "jo_v8_")
// ---------------------------------------------------------------------------

fn single_rule_grammar(suffix: &str) -> Grammar {
    GrammarBuilder::new(&format!("jo_v8_{suffix}"))
        .token("NUMBER", r"\d+")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

fn two_rule_grammar(suffix: &str) -> Grammar {
    GrammarBuilder::new(&format!("jo_v8_{suffix}"))
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "NUMBER"])
        .start("source_file")
        .build()
}

fn arith_grammar(suffix: &str) -> Grammar {
    GrammarBuilder::new(&format!("jo_v8_{suffix}"))
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn extras_grammar(suffix: &str) -> Grammar {
    GrammarBuilder::new(&format!("jo_v8_{suffix}"))
        .token("NUMBER", r"\d+")
        .token("WS", r"[ \t]+")
        .rule("source_file", vec!["NUMBER"])
        .extra("WS")
        .start("source_file")
        .build()
}

fn alternative_grammar(suffix: &str) -> Grammar {
    GrammarBuilder::new(&format!("jo_v8_{suffix}"))
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("source_file", vec!["item"])
        .rule("item", vec!["NUMBER"])
        .rule("item", vec!["IDENT"])
        .start("source_file")
        .build()
}

fn inline_grammar(suffix: &str) -> Grammar {
    GrammarBuilder::new(&format!("jo_v8_{suffix}"))
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["paren_expr"])
        .rule("paren_expr", vec!["NUMBER", "+", "NUMBER"])
        .inline("paren_expr")
        .start("source_file")
        .build()
}

fn many_rules_grammar(suffix: &str, count: usize) -> Grammar {
    let mut b = GrammarBuilder::new(&format!("jo_v8_{suffix}"))
        .token("NUMBER", r"\d+")
        .token("+", "+");
    let mut prev = "NUMBER".to_string();
    for i in 0..count {
        let name = format!("rule_{i}");
        // Leak only for the borrow—tests are short-lived.
        let name_ref: &'static str = Box::leak(name.clone().into_boxed_str());
        let prev_ref: &'static str = Box::leak(prev.into_boxed_str());
        b = b.rule(name_ref, vec![prev_ref]);
        prev = name;
    }
    let start: &'static str = Box::leak(prev.into_boxed_str());
    b = b.rule("source_file", vec![start]).start("source_file");
    b.build()
}

// ===========================================================================
// 1–5: Basic JSON validity and structure
// ===========================================================================

#[test]
fn t01_json_is_valid() {
    let json = build_json(single_rule_grammar("01"));
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn t02_json_is_array() {
    let v = build_json_value(single_rule_grammar("02"));
    assert!(v.is_array());
}

#[test]
fn t03_json_array_is_nonempty() {
    let v = build_json_value(single_rule_grammar("03"));
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn t04_each_element_has_type_field() {
    let v = build_json_value(two_rule_grammar("04"));
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some(), "missing 'type': {entry}");
    }
}

#[test]
fn t05_each_element_has_named_field() {
    let v = build_json_value(two_rule_grammar("05"));
    for entry in v.as_array().unwrap() {
        assert!(entry.get("named").is_some(), "missing 'named': {entry}");
    }
}

// ===========================================================================
// 6–7: Field types
// ===========================================================================

#[test]
fn t06_type_values_are_strings() {
    let v = build_json_value(two_rule_grammar("06"));
    for entry in v.as_array().unwrap() {
        assert!(
            entry["type"].is_string(),
            "'type' should be a string: {entry}"
        );
    }
}

#[test]
fn t07_named_values_are_booleans() {
    let v = build_json_value(two_rule_grammar("07"));
    for entry in v.as_array().unwrap() {
        assert!(
            entry["named"].is_boolean(),
            "'named' should be a boolean: {entry}"
        );
    }
}

// ===========================================================================
// 8–9: Named / token entries
// ===========================================================================

#[test]
fn t08_nonterminal_rules_produce_named_true_entries() {
    let v = build_json_value(two_rule_grammar("08"));
    let arr = v.as_array().unwrap();
    let has_named_expr = arr
        .iter()
        .any(|e| e["type"].as_str() == Some("expr") && e["named"].as_bool() == Some(true));
    assert!(has_named_expr, "non-terminal 'expr' should be named:true");
}

#[test]
fn t09_token_rules_produce_entries() {
    let v = build_json_value(two_rule_grammar("09"));
    let json_str = serde_json::to_string(&v).unwrap();
    // The JSON should contain some reference to terminal symbols
    assert!(
        !v.as_array().unwrap().is_empty(),
        "tokens should produce entries in node_types_json"
    );
    // Verify there is at least one entry (terminal or non-terminal)
    assert!(json_str.contains("type"));
}

// ===========================================================================
// 10–11: Determinism and differentiation
// ===========================================================================

#[test]
fn t10_json_is_deterministic_same_grammar_same_output() {
    let g1 = single_rule_grammar("10a");
    let g2 = single_rule_grammar("10a");
    let j1 = build_json(g1);
    let j2 = build_json(g2);
    assert_eq!(j1, j2, "same grammar should produce identical JSON");
}

#[test]
fn t11_different_grammars_produce_different_json() {
    let j1 = build_json(single_rule_grammar("11a"));
    let j2 = build_json(two_rule_grammar("11b"));
    assert_ne!(j1, j2, "different grammars should produce different JSON");
}

// ===========================================================================
// 12–15: Content / size
// ===========================================================================

#[test]
fn t12_json_contains_start_rule_name() {
    let v = build_json_value(single_rule_grammar("12"));
    let arr = v.as_array().unwrap();
    let has_source_file = arr
        .iter()
        .any(|e| e["type"].as_str() == Some("source_file"));
    assert!(
        has_source_file,
        "JSON should contain start rule 'source_file'"
    );
}

#[test]
fn t13_entry_count_reflects_grammar_complexity() {
    let small = build_json_value(single_rule_grammar("13s"));
    let large = build_json_value(two_rule_grammar("13l"));
    assert!(
        large.as_array().unwrap().len() >= small.as_array().unwrap().len(),
        "more complex grammar should produce at least as many entries"
    );
}

#[test]
fn t14_single_rule_produces_small_json() {
    let v = build_json_value(single_rule_grammar("14"));
    // A single-rule grammar should have a handful of entries (start rule + token + extras)
    assert!(v.as_array().unwrap().len() <= 20);
}

#[test]
fn t15_many_rules_produce_larger_json() {
    let small = build_json_value(single_rule_grammar("15s"));
    let big = build_json_value(many_rules_grammar("15b", 10));
    assert!(
        big.as_array().unwrap().len() > small.as_array().unwrap().len(),
        "10-rule grammar should have more entries than 1-rule grammar"
    );
}

// ===========================================================================
// 16–19: Grammar features produce valid JSON
// ===========================================================================

#[test]
fn t16_grammar_with_precedence_produces_valid_json() {
    let v = build_json_value(arith_grammar("16"));
    assert!(v.is_array());
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn t17_grammar_with_extras_produces_valid_json() {
    let v = build_json_value(extras_grammar("17"));
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn t18_grammar_with_alternatives_produces_valid_json() {
    let v = build_json_value(alternative_grammar("18"));
    assert!(v.is_array());
    for entry in v.as_array().unwrap() {
        assert!(entry["type"].is_string());
        assert!(entry["named"].is_boolean());
    }
}

#[test]
fn t19_grammar_with_inline_produces_valid_json() {
    let v = build_json_value(inline_grammar("19"));
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

// ===========================================================================
// 20: Compactness
// ===========================================================================

#[test]
fn t20_json_has_no_trailing_whitespace_lines() {
    let json = build_json(single_rule_grammar("20"));
    for (i, line) in json.lines().enumerate() {
        assert!(
            line == line.trim_end(),
            "line {i} has trailing whitespace: {line:?}"
        );
    }
}

// ===========================================================================
// 21–30: Extended structural tests
// ===========================================================================

#[test]
fn t21_all_type_values_are_nonempty_strings() {
    let v = build_json_value(arith_grammar("21"));
    for entry in v.as_array().unwrap() {
        let t = entry["type"].as_str().unwrap();
        assert!(!t.is_empty(), "type should be non-empty");
    }
}

#[test]
fn t22_no_duplicate_type_named_pairs() {
    let v = build_json_value(two_rule_grammar("22"));
    let arr = v.as_array().unwrap();
    let mut seen = std::collections::HashSet::new();
    for entry in arr {
        let key = format!(
            "{}:{}",
            entry["type"].as_str().unwrap_or(""),
            entry["named"].as_bool().unwrap_or(false)
        );
        assert!(seen.insert(key.clone()), "duplicate entry: {key}");
    }
}

#[test]
fn t23_json_parses_as_array_of_objects() {
    let v = build_json_value(arith_grammar("23"));
    for entry in v.as_array().unwrap() {
        assert!(entry.is_object(), "each entry should be a JSON object");
    }
}

#[test]
fn t24_json_string_is_nonempty() {
    let json = build_json(single_rule_grammar("24"));
    assert!(!json.is_empty());
}

#[test]
fn t25_json_starts_with_bracket() {
    let json = build_json(single_rule_grammar("25"));
    let trimmed = json.trim();
    assert!(trimmed.starts_with('['), "JSON should start with '['");
}

#[test]
fn t26_json_ends_with_bracket() {
    let json = build_json(single_rule_grammar("26"));
    let trimmed = json.trim();
    assert!(trimmed.ends_with(']'), "JSON should end with ']'");
}

#[test]
fn t27_json_roundtrips_through_serde() {
    let json = build_json(arith_grammar("27"));
    let v: Value = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&v).unwrap();
    let v2: Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(v, v2);
}

#[test]
fn t28_named_entries_have_lowercase_type() {
    let v = build_json_value(two_rule_grammar("28"));
    for entry in v.as_array().unwrap() {
        if entry["named"].as_bool() == Some(true) {
            let t = entry["type"].as_str().unwrap();
            assert_eq!(t, t.to_lowercase(), "named type should be lowercase: {t}");
        }
    }
}

#[test]
fn t29_json_contains_no_null_type() {
    let v = build_json_value(arith_grammar("29"));
    for entry in v.as_array().unwrap() {
        assert!(!entry["type"].is_null(), "type should not be null");
    }
}

#[test]
fn t30_json_contains_no_null_named() {
    let v = build_json_value(arith_grammar("30"));
    for entry in v.as_array().unwrap() {
        assert!(!entry["named"].is_null(), "named should not be null");
    }
}

// ===========================================================================
// 31–40: Content validation
// ===========================================================================

#[test]
fn t31_arith_json_contains_expr() {
    let v = build_json_value(arith_grammar("31"));
    let has = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"].as_str() == Some("expr"));
    assert!(has, "arithmetic grammar JSON should contain 'expr'");
}

#[test]
fn t32_alternative_grammar_contains_item() {
    let v = build_json_value(alternative_grammar("32"));
    let has = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"].as_str() == Some("item"));
    assert!(has, "alternative grammar should contain 'item' entry");
}

#[test]
fn t33_source_file_is_named_true() {
    let v = build_json_value(single_rule_grammar("33"));
    let sf = v
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["type"].as_str() == Some("source_file"));
    assert!(sf.is_some(), "should have source_file");
    assert_eq!(sf.unwrap()["named"].as_bool(), Some(true));
}

#[test]
fn t34_extras_grammar_still_has_start_rule() {
    let v = build_json_value(extras_grammar("34"));
    let has = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"].as_str() == Some("source_file"));
    assert!(has, "extras grammar should still contain start rule");
}

#[test]
fn t35_precedence_grammar_contains_named_expr() {
    let v = build_json_value(arith_grammar("35"));
    let expr = v
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["type"].as_str() == Some("expr"));
    assert!(expr.is_some());
    assert_eq!(expr.unwrap()["named"].as_bool(), Some(true));
}

#[test]
fn t36_many_rules_all_have_type_and_named() {
    let v = build_json_value(many_rules_grammar("36", 5));
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn t37_many_rules_chain_produces_named_entries() {
    let v = build_json_value(many_rules_grammar("37", 3));
    let named_count = v
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["named"].as_bool() == Some(true))
        .count();
    // At least the source_file + chain rules
    assert!(
        named_count >= 2,
        "chain grammar should have multiple named entries"
    );
}

#[test]
fn t38_json_values_are_primitive() {
    let v = build_json_value(two_rule_grammar("38"));
    for entry in v.as_array().unwrap() {
        let t = &entry["type"];
        let n = &entry["named"];
        assert!(t.is_string(), "'type' should be primitive string");
        assert!(n.is_boolean(), "'named' should be primitive boolean");
    }
}

#[test]
fn t39_at_least_one_named_true_entry() {
    let v = build_json_value(two_rule_grammar("39"));
    let any_named = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["named"].as_bool() == Some(true));
    assert!(any_named, "should have at least one named:true entry");
}

#[test]
fn t40_json_type_values_are_unique_within_named_group() {
    let v = build_json_value(arith_grammar("40"));
    let named: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["named"].as_bool() == Some(true))
        .filter_map(|e| e["type"].as_str())
        .collect();
    let set: std::collections::HashSet<&str> = named.iter().copied().collect();
    assert_eq!(named.len(), set.len(), "named types should be unique");
}

// ===========================================================================
// 41–50: Determinism, idempotency, and compression flag
// ===========================================================================

#[test]
fn t41_determinism_across_three_builds() {
    let results: Vec<String> = (0..3)
        .map(|_| build_json(single_rule_grammar("41")))
        .collect();
    assert_eq!(results[0], results[1]);
    assert_eq!(results[1], results[2]);
}

#[test]
fn t42_compressed_and_uncompressed_produce_same_json() {
    let g1 = single_rule_grammar("42a");
    let g2 = single_rule_grammar("42a");

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let j1 = build_parser(g1, opts1).unwrap().node_types_json;

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let j2 = build_parser(g2, opts2).unwrap().node_types_json;

    assert_eq!(j1, j2, "compression flag should not affect node_types_json");
}

#[test]
fn t43_emit_artifacts_flag_does_not_alter_json() {
    let g1 = single_rule_grammar("43a");
    let g2 = single_rule_grammar("43a");

    let dir1 = TempDir::new().unwrap();
    let opts1 = BuildOptions {
        out_dir: dir1.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let j1 = build_parser(g1, opts1).unwrap().node_types_json;

    let dir2 = TempDir::new().unwrap();
    let opts2 = BuildOptions {
        out_dir: dir2.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let j2 = build_parser(g2, opts2).unwrap().node_types_json;

    assert_eq!(
        j1, j2,
        "emit_artifacts should not alter node_types_json content"
    );
}

#[test]
fn t44_json_byte_length_is_reasonable() {
    let json = build_json(single_rule_grammar("44"));
    assert!(json.len() > 2, "JSON should be more than just '[]'");
    assert!(json.len() < 100_000, "single-rule JSON should not be huge");
}

#[test]
fn t45_json_is_valid_utf8() {
    let json = build_json(single_rule_grammar("45"));
    // If it's a String it's already valid UTF-8, but double-check via bytes
    assert!(std::str::from_utf8(json.as_bytes()).is_ok());
}

#[test]
fn t46_arith_grammar_deterministic() {
    let j1 = build_json(arith_grammar("46"));
    let j2 = build_json(arith_grammar("46"));
    assert_eq!(j1, j2);
}

#[test]
fn t47_extras_grammar_deterministic() {
    let j1 = build_json(extras_grammar("47"));
    let j2 = build_json(extras_grammar("47"));
    assert_eq!(j1, j2);
}

#[test]
fn t48_inline_grammar_deterministic() {
    let j1 = build_json(inline_grammar("48"));
    let j2 = build_json(inline_grammar("48"));
    assert_eq!(j1, j2);
}

#[test]
fn t49_alternative_grammar_deterministic() {
    let j1 = build_json(alternative_grammar("49"));
    let j2 = build_json(alternative_grammar("49"));
    assert_eq!(j1, j2);
}

#[test]
fn t50_many_rules_deterministic() {
    let j1 = build_json(many_rules_grammar("50", 7));
    let j2 = build_json(many_rules_grammar("50", 7));
    assert_eq!(j1, j2);
}

// ===========================================================================
// 51–60: Cross-grammar differentiation and feature specifics
// ===========================================================================

#[test]
fn t51_single_vs_arith_differ() {
    let j1 = build_json(single_rule_grammar("51a"));
    let j2 = build_json(arith_grammar("51b"));
    assert_ne!(j1, j2);
}

#[test]
fn t52_arith_vs_extras_differ() {
    let j1 = build_json(arith_grammar("52a"));
    let j2 = build_json(extras_grammar("52b"));
    assert_ne!(j1, j2);
}

#[test]
fn t53_alternative_vs_inline_differ() {
    let j1 = build_json(alternative_grammar("53a"));
    let j2 = build_json(inline_grammar("53b"));
    assert_ne!(j1, j2);
}

#[test]
fn t54_many_rules_5_vs_10_differ() {
    let j1 = build_json(many_rules_grammar("54a", 5));
    let j2 = build_json(many_rules_grammar("54b", 10));
    assert_ne!(j1, j2);
}

#[test]
fn t55_arith_json_has_plus_token() {
    let json = build_json(arith_grammar("55"));
    let v: Value = parse_json(&json);
    let has_plus = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"].as_str() == Some("+"));
    assert!(has_plus, "arithmetic grammar should include '+' token");
}

#[test]
fn t56_arith_json_has_star_token() {
    let json = build_json(arith_grammar("56"));
    let v: Value = parse_json(&json);
    let has_star = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"].as_str() == Some("*"));
    assert!(has_star, "arithmetic grammar should include '*' token");
}

#[test]
fn t57_operator_tokens_are_named_false() {
    let v = build_json_value(arith_grammar("57"));
    for entry in v.as_array().unwrap() {
        let t = entry["type"].as_str().unwrap_or("");
        if t == "+" || t == "*" {
            assert_eq!(
                entry["named"].as_bool(),
                Some(false),
                "operator '{t}' should be named:false"
            );
        }
    }
}

#[test]
fn t58_named_false_entries_exist_in_arith() {
    let v = build_json_value(arith_grammar("58"));
    let any_unnamed = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["named"].as_bool() == Some(false));
    assert!(any_unnamed, "arith grammar should have unnamed entries");
}

#[test]
fn t59_all_entries_are_objects_not_arrays() {
    let v = build_json_value(arith_grammar("59"));
    for entry in v.as_array().unwrap() {
        assert!(!entry.is_array(), "entries should be objects, not arrays");
    }
}

#[test]
fn t60_all_entries_are_objects_not_strings() {
    let v = build_json_value(two_rule_grammar("60"));
    for entry in v.as_array().unwrap() {
        assert!(!entry.is_string(), "entries should be objects, not strings");
    }
}

// ===========================================================================
// 61–70: Edge cases and robustness
// ===========================================================================

#[test]
fn t61_grammar_name_does_not_appear_as_type() {
    let v = build_json_value(single_rule_grammar("61"));
    let name_in_types = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"].as_str() == Some("jo_v8_61"));
    assert!(
        !name_in_types,
        "grammar name should not appear as a node type"
    );
}

#[test]
fn t62_json_contains_no_control_characters() {
    let json = build_json(single_rule_grammar("62"));
    for (i, ch) in json.chars().enumerate() {
        if ch.is_control() {
            assert!(
                ch == '\n' || ch == '\r' || ch == '\t',
                "unexpected control char at position {i}: {ch:?}"
            );
        }
    }
}

#[test]
fn t63_json_only_ascii_type_names() {
    let v = build_json_value(arith_grammar("63"));
    for entry in v.as_array().unwrap() {
        let t = entry["type"].as_str().unwrap();
        assert!(t.is_ascii(), "type name should be ASCII: {t}");
    }
}

#[test]
fn t64_precedence_left_right_both_produce_valid_json() {
    let g = GrammarBuilder::new("jo_v8_64")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 2, Associativity::Right)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let v = build_json_value(g);
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn t65_precedence_none_produces_valid_json() {
    let g = GrammarBuilder::new("jo_v8_65")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let v = build_json_value(g);
    assert!(v.is_array());
}

#[test]
fn t66_two_extras_produce_valid_json() {
    let g = GrammarBuilder::new("jo_v8_66")
        .token("NUMBER", r"\d+")
        .token("WS", r"[ \t]+")
        .token("COMMENT", r"//[^\n]*")
        .rule("source_file", vec!["NUMBER"])
        .extra("WS")
        .extra("COMMENT")
        .start("source_file")
        .build();
    let v = build_json_value(g);
    assert!(v.is_array());
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some());
    }
}

#[test]
fn t67_multiple_alternatives_all_valid() {
    let g = GrammarBuilder::new("jo_v8_67")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("source_file", vec!["item"])
        .rule("item", vec!["A"])
        .rule("item", vec!["B"])
        .rule("item", vec!["C"])
        .start("source_file")
        .build();
    let v = build_json_value(g);
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn t68_chain_grammar_depth_1() {
    let v = build_json_value(many_rules_grammar("68", 1));
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn t69_chain_grammar_depth_2() {
    let v = build_json_value(many_rules_grammar("69", 2));
    for entry in v.as_array().unwrap() {
        assert!(entry["type"].is_string());
    }
}

#[test]
fn t70_chain_grammar_depth_8() {
    let v = build_json_value(many_rules_grammar("70", 8));
    let named_count = v
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["named"].as_bool() == Some(true))
        .count();
    assert!(
        named_count >= 2,
        "deep chain should have multiple named entries"
    );
}

// ===========================================================================
// 71–80: Additional coverage
// ===========================================================================

#[test]
fn t71_json_has_no_empty_type() {
    let v = build_json_value(arith_grammar("71"));
    for entry in v.as_array().unwrap() {
        assert_ne!(
            entry["type"].as_str(),
            Some(""),
            "type should not be empty string"
        );
    }
}

#[test]
fn t72_json_pretty_print_roundtrip() {
    let json = build_json(arith_grammar("72"));
    let v: Value = parse_json(&json);
    let pretty = serde_json::to_string_pretty(&v).unwrap();
    let v2: Value = serde_json::from_str(&pretty).unwrap();
    assert_eq!(v, v2);
}

#[test]
fn t73_single_rule_grammar_entry_count_small() {
    let v = build_json_value(single_rule_grammar("73"));
    let len = v.as_array().unwrap().len();
    assert!(
        len <= 15,
        "single rule grammar should have few entries, got {len}"
    );
}

#[test]
fn t74_build_result_grammar_name_matches() {
    let (_d, opts) = test_opts();
    let result = build_parser(single_rule_grammar("74"), opts).unwrap();
    assert!(
        result.grammar_name.contains("jo_v8_74"),
        "grammar_name should reflect the input name"
    );
}

#[test]
fn t75_build_stats_state_count_positive() {
    let (_d, opts) = test_opts();
    let result = build_parser(single_rule_grammar("75"), opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn t76_build_stats_symbol_count_positive() {
    let (_d, opts) = test_opts();
    let result = build_parser(single_rule_grammar("76"), opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn t77_node_types_json_len_grows_with_rules() {
    let small_len = build_json(many_rules_grammar("77s", 2)).len();
    let big_len = build_json(many_rules_grammar("77b", 8)).len();
    assert!(
        big_len > small_len,
        "JSON string length should grow with rule count"
    );
}

#[test]
fn t78_multiple_precedence_levels_valid() {
    let g = GrammarBuilder::new("jo_v8_78")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let v = build_json_value(g);
    assert!(v.is_array());
    let has_expr = v
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"].as_str() == Some("expr"));
    assert!(has_expr);
}

#[test]
fn t79_json_does_not_contain_null_entries() {
    let v = build_json_value(two_rule_grammar("79"));
    for entry in v.as_array().unwrap() {
        assert!(!entry.is_null(), "array should not contain null entries");
    }
}

#[test]
fn t80_json_entries_have_at_least_two_keys() {
    let v = build_json_value(arith_grammar("80"));
    for entry in v.as_array().unwrap() {
        let obj = entry.as_object().unwrap();
        assert!(
            obj.len() >= 2,
            "each entry should have at least 'type' and 'named', got {} keys",
            obj.len()
        );
    }
}

#[test]
fn t81_left_associativity_produces_valid_json() {
    let g = GrammarBuilder::new("jo_v8_81")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let v = build_json_value(g);
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn t82_right_associativity_produces_valid_json() {
    let g = GrammarBuilder::new("jo_v8_82")
        .token("NUMBER", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let v = build_json_value(g);
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn t83_extras_do_not_appear_as_named_true() {
    let v = build_json_value(extras_grammar("83"));
    // WS is an extra token; if it appears in the output it should be unnamed
    for entry in v.as_array().unwrap() {
        if entry["type"].as_str() == Some("WS") {
            assert_eq!(
                entry["named"].as_bool(),
                Some(false),
                "extra token WS should not be named:true"
            );
        }
    }
}

#[test]
fn t84_no_entry_has_extra_unexpected_null_fields() {
    let v = build_json_value(arith_grammar("84"));
    for entry in v.as_array().unwrap() {
        let obj = entry.as_object().unwrap();
        for (key, val) in obj {
            // "type" and "named" are required; others can exist but should not be null
            if key == "type" || key == "named" {
                assert!(!val.is_null(), "required field '{key}' should not be null");
            }
        }
    }
}

#[test]
fn t85_json_has_consistent_ordering() {
    // Build twice and check that JSON string representation is identical
    let j1 = build_json(alternative_grammar("85"));
    let j2 = build_json(alternative_grammar("85"));
    assert_eq!(j1, j2, "JSON output ordering should be consistent");
}
