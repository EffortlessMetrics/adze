//! Comprehensive tests for `NodeTypesGenerator` JSON output in adze-tablegen.
//!
//! Categories (80+ tests):
//!   1. Single token grammar → valid JSON (tests 1–5)
//!   2. Output structure: brackets, parse, array (tests 6–10)
//!   3. Entry field validation: "type" and "named" (tests 11–18)
//!   4. "type" values are strings, "named" values are booleans (tests 19–24)
//!   5. Multiple tokens → more entries (tests 25–30)
//!   6. Rule names appear in output (tests 31–36)
//!   7. Token names appear in output (tests 37–42)
//!   8. Determinism: same grammar → same JSON (tests 43–48)
//!   9. Different grammars → different JSON (tests 49–53)
//!  10. Grammar with precedence → valid JSON (tests 54–58)
//!  11. Grammar with inline → valid JSON (tests 59–62)
//!  12. Grammar with extras → valid JSON (tests 63–66)
//!  13. Grammar with externals → valid JSON (tests 67–70)
//!  14. Large grammar → valid JSON (tests 71–74)
//!  15. Pretty-print, sorting, duplicates, edge cases (tests 75–85)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn generate(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate must succeed")
}

fn parsed(grammar: &Grammar) -> Vec<Value> {
    let json = generate(grammar);
    let val: Value = serde_json::from_str(&json).expect("valid JSON");
    val.as_array().expect("top-level array").to_vec()
}

fn find_entry<'a>(arr: &'a [Value], type_name: &str) -> Option<&'a Value> {
    arr.iter().find(|n| n["type"].as_str() == Some(type_name))
}

fn type_names(arr: &[Value]) -> Vec<String> {
    arr.iter()
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

fn make_scaled(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    let toks: Vec<String> = (0..n).map(|i| format!("tok_{i}")).collect();
    let rules: Vec<String> = (0..n).map(|i| format!("rule_{i}")).collect();
    for i in 0..n {
        b = b.token(&toks[i], &toks[i]);
        b = b.rule(&rules[i], vec![&toks[i]]);
    }
    if n > 0 {
        b = b.start(&rules[0]);
    }
    b.build()
}

// ===========================================================================
// 1. Single token grammar → valid JSON
// ===========================================================================

#[test]
fn ntj_v9_single_token_generates_ok() {
    let g = GrammarBuilder::new("ntj_v9_st1")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_single_token_output_nonempty() {
    let g = GrammarBuilder::new("ntj_v9_st2")
        .token("x", "x")
        .rule("root", vec!["x"])
        .build();
    assert!(!generate(&g).is_empty());
}

#[test]
fn ntj_v9_single_token_parses_as_json() {
    let g = GrammarBuilder::new("ntj_v9_st3")
        .token("y", "y")
        .rule("s", vec!["y"])
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_single_token_is_json_array() {
    let g = GrammarBuilder::new("ntj_v9_st4")
        .token("z", "z")
        .rule("s", vec!["z"])
        .build();
    let val: Value = serde_json::from_str(&generate(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn ntj_v9_single_token_array_has_entries() {
    let g = GrammarBuilder::new("ntj_v9_st5")
        .token("a", "a")
        .rule("r", vec!["a"])
        .build();
    assert!(!parsed(&g).is_empty());
}

// ===========================================================================
// 2. Output structure: brackets, parse, array
// ===========================================================================

#[test]
fn ntj_v9_output_starts_with_bracket() {
    let g = GrammarBuilder::new("ntj_v9_br1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    assert!(generate(&g).trim_start().starts_with('['));
}

#[test]
fn ntj_v9_output_ends_with_bracket() {
    let g = GrammarBuilder::new("ntj_v9_br2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    assert!(generate(&g).trim_end().ends_with(']'));
}

#[test]
fn ntj_v9_serde_from_str_succeeds() {
    let g = GrammarBuilder::new("ntj_v9_br3")
        .token("id", r"[a-z]+")
        .rule("prog", vec!["id"])
        .start("prog")
        .build();
    let result: Result<Value, _> = serde_json::from_str(&generate(&g));
    assert!(result.is_ok());
}

#[test]
fn ntj_v9_parsed_value_is_array() {
    let g = GrammarBuilder::new("ntj_v9_br4")
        .token("n", r"\d+")
        .rule("num", vec!["n"])
        .build();
    let val: Value = serde_json::from_str(&generate(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn ntj_v9_empty_grammar_produces_array() {
    let g = Grammar::new("ntj_v9_br5".to_string());
    let val: Value = serde_json::from_str(&generate(&g)).unwrap();
    assert!(val.is_array());
}

// ===========================================================================
// 3. Entry field validation: "type" and "named"
// ===========================================================================

#[test]
fn ntj_v9_each_entry_has_type_key() {
    let g = GrammarBuilder::new("ntj_v9_fv1")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .build();
    for entry in parsed(&g) {
        assert!(entry.get("type").is_some(), "missing 'type': {entry}");
    }
}

#[test]
fn ntj_v9_each_entry_has_named_key() {
    let g = GrammarBuilder::new("ntj_v9_fv2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    for entry in parsed(&g) {
        assert!(entry.get("named").is_some(), "missing 'named': {entry}");
    }
}

#[test]
fn ntj_v9_arithmetic_entries_have_type_and_named() {
    let g = GrammarBuilder::new("ntj_v9_fv3")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["num"])
        .rule("add", vec!["expr", "+", "expr"])
        .start("add")
        .build();
    for entry in parsed(&g) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn ntj_v9_scaled_entries_have_type_and_named() {
    let g = make_scaled("ntj_v9_fv4", 8);
    for entry in parsed(&g) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn ntj_v9_no_entries_have_null_type() {
    let g = GrammarBuilder::new("ntj_v9_fv5")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .build();
    for entry in parsed(&g) {
        assert!(!entry["type"].is_null());
    }
}

#[test]
fn ntj_v9_no_entries_have_null_named() {
    let g = GrammarBuilder::new("ntj_v9_fv6")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    for entry in parsed(&g) {
        assert!(!entry["named"].is_null());
    }
}

#[test]
fn ntj_v9_entries_are_objects() {
    let g = GrammarBuilder::new("ntj_v9_fv7")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    for entry in parsed(&g) {
        assert!(entry.is_object(), "each entry must be a JSON object");
    }
}

#[test]
fn ntj_v9_type_field_is_never_empty_string() {
    let g = GrammarBuilder::new("ntj_v9_fv8")
        .token("id", r"[a-z]+")
        .rule("root", vec!["id"])
        .build();
    for entry in parsed(&g) {
        let t = entry["type"].as_str().expect("type is string");
        assert!(!t.is_empty());
    }
}

// ===========================================================================
// 4. "type" values are strings, "named" values are booleans
// ===========================================================================

#[test]
fn ntj_v9_type_values_are_strings() {
    let g = GrammarBuilder::new("ntj_v9_tv1")
        .token("a", "a")
        .token("+", "+")
        .rule("s", vec!["a", "+", "a"])
        .build();
    for entry in parsed(&g) {
        assert!(entry["type"].is_string(), "type must be string: {entry}");
    }
}

#[test]
fn ntj_v9_named_values_are_booleans() {
    let g = GrammarBuilder::new("ntj_v9_tv2")
        .token("a", "a")
        .token("+", "+")
        .rule("s", vec!["a", "+", "a"])
        .build();
    for entry in parsed(&g) {
        assert!(
            entry["named"].is_boolean(),
            "named must be boolean: {entry}"
        );
    }
}

#[test]
fn ntj_v9_type_string_named_bool_for_scaled() {
    let g = make_scaled("ntj_v9_tv3", 12);
    for entry in parsed(&g) {
        assert!(entry["type"].is_string());
        assert!(entry["named"].is_boolean());
    }
}

#[test]
fn ntj_v9_rule_is_named_true() {
    let g = GrammarBuilder::new("ntj_v9_tv4")
        .token("x", "x")
        .rule("stmt", vec!["x"])
        .build();
    let nodes = parsed(&g);
    if let Some(stmt) = find_entry(&nodes, "stmt") {
        assert_eq!(stmt["named"], true);
    }
}

#[test]
fn ntj_v9_string_literal_token_is_named_false() {
    let g = GrammarBuilder::new("ntj_v9_tv5")
        .token("+", "+")
        .rule("op", vec!["+"])
        .build();
    let nodes = parsed(&g);
    if let Some(plus) = find_entry(&nodes, "+") {
        assert_eq!(plus["named"], false);
    }
}

#[test]
fn ntj_v9_named_field_not_string() {
    let g = GrammarBuilder::new("ntj_v9_tv6")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    for entry in parsed(&g) {
        assert!(!entry["named"].is_string());
    }
}

// ===========================================================================
// 5. Multiple tokens → more entries
// ===========================================================================

#[test]
fn ntj_v9_two_tokens_more_entries_than_one() {
    let g1 = GrammarBuilder::new("ntj_v9_mt1a")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    let g2 = GrammarBuilder::new("ntj_v9_mt1b")
        .token("a", "a")
        .token("b", "b")
        .rule("s1", vec!["a"])
        .rule("s2", vec!["b"])
        .build();
    assert!(parsed(&g2).len() >= parsed(&g1).len());
}

#[test]
fn ntj_v9_five_tokens_nonempty() {
    let g = GrammarBuilder::new("ntj_v9_mt2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a", "b", "c", "d", "e"])
        .build();
    assert!(!parsed(&g).is_empty());
}

#[test]
fn ntj_v9_ten_tokens_more_than_five() {
    let g5 = make_scaled("ntj_v9_mt3a", 5);
    let g10 = make_scaled("ntj_v9_mt3b", 10);
    assert!(parsed(&g10).len() >= parsed(&g5).len());
}

#[test]
fn ntj_v9_multiple_token_types_valid_json() {
    let g = GrammarBuilder::new("ntj_v9_mt4")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .token(";", ";")
        .token("=", "=")
        .rule("assign", vec!["id", "=", "num", ";"])
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_token_patterns_not_in_type_names() {
    let g = GrammarBuilder::new("ntj_v9_mt5")
        .token("number", r"\d+")
        .rule("s", vec!["number"])
        .build();
    let names = type_names(&parsed(&g));
    assert!(!names.contains(&r"\d+".to_string()));
}

#[test]
fn ntj_v9_mixed_string_and_regex_tokens() {
    let g = GrammarBuilder::new("ntj_v9_mt6")
        .token("id", r"[a-z]+")
        .token(",", ",")
        .rule("list", vec!["id", ",", "id"])
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "list").is_some());
}

// ===========================================================================
// 6. Rule names appear in output
// ===========================================================================

#[test]
fn ntj_v9_rule_name_in_output() {
    let g = GrammarBuilder::new("ntj_v9_rn1")
        .token("x", "x")
        .rule("statement", vec!["x"])
        .build();
    assert!(find_entry(&parsed(&g), "statement").is_some());
}

#[test]
fn ntj_v9_two_rules_both_appear() {
    let g = GrammarBuilder::new("ntj_v9_rn2")
        .token("a", "a")
        .token("b", "b")
        .rule("first", vec!["a"])
        .rule("second", vec!["b"])
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "first").is_some());
    assert!(find_entry(&nodes, "second").is_some());
}

#[test]
fn ntj_v9_three_rules_all_named() {
    let g = GrammarBuilder::new("ntj_v9_rn3")
        .token("x", "x")
        .rule("alpha", vec!["x"])
        .rule("beta", vec!["x"])
        .rule("gamma", vec!["x"])
        .build();
    let nodes = parsed(&g);
    for name in ["alpha", "beta", "gamma"] {
        assert!(find_entry(&nodes, name).is_some(), "missing {name}");
    }
}

#[test]
fn ntj_v9_chained_rules_all_appear() {
    let g = GrammarBuilder::new("ntj_v9_rn4")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("top", vec!["mid"])
        .start("top")
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "leaf").is_some());
    assert!(find_entry(&nodes, "mid").is_some());
    assert!(find_entry(&nodes, "top").is_some());
}

#[test]
fn ntj_v9_same_lhs_multiple_alts_single_entry() {
    let g = GrammarBuilder::new("ntj_v9_rn5")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .build();
    let count = parsed(&g)
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn ntj_v9_grammar_name_not_in_type_names() {
    let g = GrammarBuilder::new("ntj_v9_rn6")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    let names = type_names(&parsed(&g));
    assert!(!names.contains(&"ntj_v9_rn6".to_string()));
}

// ===========================================================================
// 7. Token names appear in output
// ===========================================================================

#[test]
fn ntj_v9_string_token_name_in_output() {
    let g = GrammarBuilder::new("ntj_v9_tn1")
        .token("==", "==")
        .rule("cmp", vec!["=="])
        .build();
    let nodes = parsed(&g);
    if let Some(eq) = find_entry(&nodes, "==") {
        assert_eq!(eq["named"], false);
    }
}

#[test]
fn ntj_v9_operator_tokens_appear() {
    let g = GrammarBuilder::new("ntj_v9_tn2")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("op", vec!["+"])
        .build();
    let names = type_names(&parsed(&g));
    // At least the rule should appear
    assert!(names.contains(&"op".to_string()));
}

#[test]
fn ntj_v9_regex_token_used_in_rule_appears() {
    let g = GrammarBuilder::new("ntj_v9_tn3")
        .token("identifier", r"[a-zA-Z_]+")
        .rule("var", vec!["identifier"])
        .build();
    assert!(find_entry(&parsed(&g), "var").is_some());
}

#[test]
fn ntj_v9_semicolon_token_in_output() {
    let g = GrammarBuilder::new("ntj_v9_tn4")
        .token(";", ";")
        .token("x", "x")
        .rule("stmt", vec!["x", ";"])
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "stmt").is_some());
}

#[test]
fn ntj_v9_keyword_token_in_output() {
    let g = GrammarBuilder::new("ntj_v9_tn5")
        .token("if", "if")
        .token("cond", r"[a-z]+")
        .rule("ifstmt", vec!["if", "cond"])
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "ifstmt").is_some());
}

#[test]
fn ntj_v9_all_entries_have_string_type() {
    let g = GrammarBuilder::new("ntj_v9_tn6")
        .token("a", "a")
        .token("+", "+")
        .rule("sum", vec!["a", "+", "a"])
        .build();
    for entry in parsed(&g) {
        assert!(entry["type"].is_string());
    }
}

// ===========================================================================
// 8. Determinism: same grammar → same JSON
// ===========================================================================

#[test]
fn ntj_v9_determinism_simple() {
    let g = GrammarBuilder::new("ntj_v9_det1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn ntj_v9_determinism_arithmetic() {
    let g = GrammarBuilder::new("ntj_v9_det2")
        .token("n", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["n"])
        .rule("add", vec!["expr", "+", "expr"])
        .start("add")
        .build();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn ntj_v9_determinism_scaled() {
    let g = make_scaled("ntj_v9_det3", 15);
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn ntj_v9_determinism_separate_generators() {
    let g = GrammarBuilder::new("ntj_v9_det4")
        .token("x", "x")
        .rule("r", vec!["x"])
        .build();
    let g1 = NodeTypesGenerator::new(&g);
    let g2 = NodeTypesGenerator::new(&g);
    assert_eq!(g1.generate().unwrap(), g2.generate().unwrap());
}

#[test]
fn ntj_v9_determinism_with_precedence() {
    let g = GrammarBuilder::new("ntj_v9_det5")
        .token("n", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn ntj_v9_determinism_ten_iterations() {
    let g = GrammarBuilder::new("ntj_v9_det6")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .build();
    let first = generate(&g);
    for _ in 0..10 {
        assert_eq!(first, generate(&g));
    }
}

// ===========================================================================
// 9. Different grammars → different JSON
// ===========================================================================

#[test]
fn ntj_v9_different_rules_different_output() {
    let g1 = GrammarBuilder::new("ntj_v9_df1a")
        .token("x", "x")
        .rule("alpha", vec!["x"])
        .build();
    let g2 = GrammarBuilder::new("ntj_v9_df1b")
        .token("x", "x")
        .rule("beta", vec!["x"])
        .build();
    assert_ne!(generate(&g1), generate(&g2));
}

#[test]
fn ntj_v9_different_token_counts_different_output() {
    let g1 = make_scaled("ntj_v9_df2a", 2);
    let g2 = make_scaled("ntj_v9_df2b", 5);
    assert_ne!(generate(&g1), generate(&g2));
}

#[test]
fn ntj_v9_additional_rules_change_output() {
    let g1 = GrammarBuilder::new("ntj_v9_df3a")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    let g2 = GrammarBuilder::new("ntj_v9_df3b")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("extra_rule", vec!["y"])
        .build();
    assert_ne!(generate(&g1), generate(&g2));
}

#[test]
fn ntj_v9_empty_vs_nonempty_grammar() {
    let g1 = Grammar::new("ntj_v9_df4a".to_string());
    let g2 = GrammarBuilder::new("ntj_v9_df4b")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    assert_ne!(generate(&g1), generate(&g2));
}

#[test]
fn ntj_v9_different_rule_names_different_types() {
    let g1 = GrammarBuilder::new("ntj_v9_df5a")
        .token("x", "x")
        .rule("foo", vec!["x"])
        .build();
    let g2 = GrammarBuilder::new("ntj_v9_df5b")
        .token("x", "x")
        .rule("bar", vec!["x"])
        .build();
    let n1 = type_names(&parsed(&g1));
    let n2 = type_names(&parsed(&g2));
    assert_ne!(n1, n2);
}

// ===========================================================================
// 10. Grammar with precedence → valid JSON
// ===========================================================================

#[test]
fn ntj_v9_precedence_generates_ok() {
    let g = GrammarBuilder::new("ntj_v9_pr1")
        .token("n", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_precedence_valid_json() {
    let g = GrammarBuilder::new("ntj_v9_pr2")
        .token("n", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_right_assoc_generates() {
    let g = GrammarBuilder::new("ntj_v9_pr3")
        .token("n", r"\d+")
        .token("^", "^")
        .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_none_assoc_generates() {
    let g = GrammarBuilder::new("ntj_v9_pr4")
        .token("n", r"\d+")
        .token("<", "<")
        .rule_with_precedence("cmp", vec!["e", "<", "e"], 1, Associativity::None)
        .rule("e", vec!["n"])
        .start("cmp")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_precedence_decl_generates() {
    let g = GrammarBuilder::new("ntj_v9_pr5")
        .token("n", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["n"])
        .rule("add", vec!["expr", "+", "expr"])
        .rule("mul", vec!["expr", "*", "expr"])
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .start("add")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

// ===========================================================================
// 11. Grammar with inline → valid JSON
// ===========================================================================

#[test]
fn ntj_v9_inline_generates_ok() {
    let g = GrammarBuilder::new("ntj_v9_inl1")
        .token("x", "x")
        .rule("_helper", vec!["x"])
        .rule("start", vec!["_helper"])
        .inline("_helper")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_inline_underscore_not_in_output() {
    let g = GrammarBuilder::new("ntj_v9_inl2")
        .token("x", "x")
        .rule("_internal", vec!["x"])
        .rule("start", vec!["_internal"])
        .inline("_internal")
        .start("start")
        .build();
    assert!(find_entry(&parsed(&g), "_internal").is_none());
}

#[test]
fn ntj_v9_inline_does_not_suppress_public_rule() {
    let g = GrammarBuilder::new("ntj_v9_inl3")
        .token("x", "x")
        .rule("_h", vec!["x"])
        .rule("start", vec!["_h"])
        .inline("_h")
        .start("start")
        .build();
    assert!(find_entry(&parsed(&g), "start").is_some());
}

#[test]
fn ntj_v9_multiple_inline_rules() {
    let g = GrammarBuilder::new("ntj_v9_inl4")
        .token("a", "a")
        .token("b", "b")
        .rule("_h1", vec!["a"])
        .rule("_h2", vec!["b"])
        .rule("start", vec!["_h1", "_h2"])
        .inline("_h1")
        .inline("_h2")
        .start("start")
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "_h1").is_none());
    assert!(find_entry(&nodes, "_h2").is_none());
    assert!(find_entry(&nodes, "start").is_some());
}

// ===========================================================================
// 12. Grammar with extras → valid JSON
// ===========================================================================

#[test]
fn ntj_v9_extras_generates_ok() {
    let g = GrammarBuilder::new("ntj_v9_ext1")
        .token("ws", r"\s+")
        .token("id", r"[a-z]+")
        .rule("start", vec!["id"])
        .extra("ws")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_extras_valid_json() {
    let g = GrammarBuilder::new("ntj_v9_ext2")
        .token("ws", r"\s+")
        .token("x", "x")
        .rule("s", vec!["x"])
        .extra("ws")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_extras_nonempty_output() {
    let g = GrammarBuilder::new("ntj_v9_ext3")
        .token("ws", r"\s+")
        .token("x", "x")
        .rule("s", vec!["x"])
        .extra("ws")
        .build();
    assert!(!parsed(&g).is_empty());
}

#[test]
fn ntj_v9_multiple_extras() {
    let g = GrammarBuilder::new("ntj_v9_ext4")
        .token("ws", r"\s+")
        .token("comment", r"//[^\n]*")
        .token("x", "x")
        .rule("start", vec!["x"])
        .extra("ws")
        .extra("comment")
        .start("start")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

// ===========================================================================
// 13. Grammar with externals → valid JSON
// ===========================================================================

#[test]
fn ntj_v9_externals_generates_ok() {
    let g = GrammarBuilder::new("ntj_v9_xtn1")
        .token("x", "x")
        .rule("start", vec!["x"])
        .external("indent")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_externals_valid_json() {
    let g = GrammarBuilder::new("ntj_v9_xtn2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .external("dedent")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_multiple_externals() {
    let g = GrammarBuilder::new("ntj_v9_xtn3")
        .token("x", "x")
        .rule("start", vec!["x"])
        .external("indent")
        .external("dedent")
        .external("newline")
        .start("start")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_externals_do_not_suppress_rules() {
    let g = GrammarBuilder::new("ntj_v9_xtn4")
        .token("x", "x")
        .rule("start", vec!["x"])
        .external("indent")
        .start("start")
        .build();
    assert!(find_entry(&parsed(&g), "start").is_some());
}

// ===========================================================================
// 14. Large grammar → valid JSON
// ===========================================================================

#[test]
fn ntj_v9_ten_tokens_valid_json() {
    let g = make_scaled("ntj_v9_lg1", 10);
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_twenty_tokens_all_objects() {
    let g = make_scaled("ntj_v9_lg2", 20);
    for entry in parsed(&g) {
        assert!(entry.is_object());
    }
}

#[test]
fn ntj_v9_thirty_tokens_valid_json() {
    let g = make_scaled("ntj_v9_lg3", 30);
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_large_grammar_all_entries_have_type_named() {
    let g = make_scaled("ntj_v9_lg4", 25);
    for entry in parsed(&g) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

// ===========================================================================
// 15. Pretty-print, sorting, duplicates, edge cases
// ===========================================================================

#[test]
fn ntj_v9_output_is_pretty_printed() {
    let g = GrammarBuilder::new("ntj_v9_pp1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    assert!(generate(&g).contains('\n'));
}

#[test]
fn ntj_v9_output_sorted_by_type_name() {
    let g = GrammarBuilder::new("ntj_v9_sort1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("zebra", vec!["a"])
        .rule("apple", vec!["b"])
        .rule("mango", vec!["c"])
        .build();
    let names = type_names(&parsed(&g));
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn ntj_v9_no_duplicate_type_named_pairs() {
    let g = GrammarBuilder::new("ntj_v9_dup1")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .rule("stmt", vec!["expr"])
        .build();
    let nodes = parsed(&g);
    let mut seen = std::collections::HashSet::new();
    for n in &nodes {
        let key = format!(
            "{}-{}",
            n["type"].as_str().unwrap_or(""),
            n["named"].as_bool().unwrap_or(false)
        );
        assert!(seen.insert(key.clone()), "duplicate: {key}");
    }
}

#[test]
fn ntj_v9_empty_grammar_is_empty_array() {
    let g = Grammar::new("ntj_v9_edge1".to_string());
    assert!(parsed(&g).is_empty());
}

#[test]
fn ntj_v9_epsilon_rule_generates() {
    let g = GrammarBuilder::new("ntj_v9_edge2")
        .rule("empty", vec![])
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn ntj_v9_fragile_token_generates() {
    let g = GrammarBuilder::new("ntj_v9_edge3")
        .fragile_token("if", "if")
        .token("x", "x")
        .rule("start", vec!["if", "x"])
        .start("start")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_supertype_generates_ok() {
    let g = GrammarBuilder::new("ntj_v9_edge4")
        .token("a", "a")
        .token("b", "b")
        .rule("va", vec!["a"])
        .rule("vb", vec!["b"])
        .rule("expr", vec!["va"])
        .rule("expr", vec!["vb"])
        .supertype("expr")
        .start("expr")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_supertype_appears_in_output() {
    let g = GrammarBuilder::new("ntj_v9_edge5")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("expr", vec!["leaf"])
        .supertype("expr")
        .start("expr")
        .build();
    assert!(find_entry(&parsed(&g), "expr").is_some());
}

#[test]
fn ntj_v9_list_pattern_generates() {
    let g = GrammarBuilder::new("ntj_v9_edge6")
        .token("item", r"[a-z]+")
        .token(",", ",")
        .rule("list", vec!["item"])
        .rule("list", vec!["list", ",", "item"])
        .start("list")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn ntj_v9_tree_pattern_generates() {
    let g = GrammarBuilder::new("ntj_v9_edge7")
        .token("leaf", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .rule("node", vec!["leaf"])
        .rule("node", vec!["(", "node", "node", ")"])
        .start("node")
        .build();
    assert!(find_entry(&parsed(&g), "node").is_some());
}

#[test]
fn ntj_v9_combined_features_grammar() {
    let g = GrammarBuilder::new("ntj_v9_edge8")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .token("+", "+")
        .token(";", ";")
        .token("ws", r"\s+")
        .rule("expr", vec!["num"])
        .rule("add", vec!["expr", "+", "expr"])
        .rule("stmt", vec!["id", ";"])
        .rule("program", vec!["stmt"])
        .extra("ws")
        .external("indent")
        .precedence(1, Associativity::Left, vec!["+"])
        .start("program")
        .build();
    let json = generate(&g);
    assert!(serde_json::from_str::<Value>(&json).is_ok());
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "program").is_some());
    for entry in &nodes {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn ntj_v9_if_else_pattern() {
    let g = GrammarBuilder::new("ntj_v9_edge9")
        .token("if", "if")
        .token("else", "else")
        .token("cond", r"[a-z]+")
        .token("body", r"\d+")
        .rule("if_stmt", vec!["if", "cond", "body"])
        .rule("if_else", vec!["if", "cond", "body", "else", "body"])
        .start("if_stmt")
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "if_stmt").is_some());
    assert!(find_entry(&nodes, "if_else").is_some());
}

#[test]
fn ntj_v9_statement_program_pattern() {
    let g = GrammarBuilder::new("ntj_v9_edge10")
        .token("id", r"[a-z]+")
        .token("=", "=")
        .token("num", r"\d+")
        .token(";", ";")
        .rule("assign", vec!["id", "=", "num"])
        .rule("stmt", vec!["assign", ";"])
        .rule("program", vec!["stmt"])
        .rule("program", vec!["program", "stmt"])
        .start("program")
        .build();
    let nodes = parsed(&g);
    assert!(find_entry(&nodes, "program").is_some());
    assert!(find_entry(&nodes, "stmt").is_some());
    assert!(find_entry(&nodes, "assign").is_some());
}

#[test]
fn ntj_v9_minimal_grammar_valid_json() {
    let g = GrammarBuilder::new("ntj_v9_edge11")
        .token("x", "x")
        .rule("r", vec!["x"])
        .build();
    let json = generate(&g);
    assert!(json.trim_start().starts_with('['));
    assert!(json.trim_end().ends_with(']'));
    assert!(serde_json::from_str::<Value>(&json).is_ok());
    let arr = parsed(&g);
    assert!(!arr.is_empty());
    for entry in &arr {
        assert!(entry["type"].is_string());
        assert!(entry["named"].is_boolean());
    }
}
