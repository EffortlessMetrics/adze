//! Property-based tests for `NodeTypesGenerator` in adze-tablegen.
//!
//! 80+ tests organized as:
//!   - Property tests (proptest): core invariants over random grammars
//!   - Unit tests: specific grammar patterns, edge cases, feature combos
//!
//! Properties verified:
//!  1. Any valid grammar → generate succeeds
//!  2. Output is valid JSON
//!  3. Output is a JSON array
//!  4. Array is non-empty (when grammar has visible symbols)
//!  5. Each entry has "type" field
//!  6. Each entry has "named" field
//!  7. "type" values are strings
//!  8. "named" values are booleans
//!  9. Output is deterministic
//! 10. Output length > 0

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::NodeTypesGenerator;
use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;

// ───────────────────────────────────────────────────────────────────────
// Strategies
// ───────────────────────────────────────────────────────────────────────

fn arb_token_count() -> impl Strategy<Value = usize> {
    1usize..8
}

fn arb_rule_count() -> impl Strategy<Value = usize> {
    1usize..5
}

/// Build a grammar with `n_tokens` tokens and `n_rules` rules via GrammarBuilder.
/// All names are lowercase and unique (include index).
fn make_grammar(n_tokens: usize, n_rules: usize) -> adze_ir::Grammar {
    let mut b = GrammarBuilder::new("propv9");
    for i in 0..n_tokens {
        let name = format!("tok{i}");
        b = b.token(&name, &name);
    }
    // First token name for rule RHS references
    let first_tok = "tok0".to_string();
    for i in 0..n_rules {
        let name = format!("rule{i}");
        b = b.rule(&name, vec![&first_tok]);
    }
    if n_rules > 0 {
        b = b.start("rule0");
    }
    b.build()
}

/// Build a grammar with multiple tokens per rule.
fn make_multi_token_grammar(n_tokens: usize, n_rules: usize) -> adze_ir::Grammar {
    let mut b = GrammarBuilder::new("propv9multi");
    let tok_names: Vec<String> = (0..n_tokens).map(|i| format!("mtok{i}")).collect();
    for name in &tok_names {
        b = b.token(name, name);
    }
    for i in 0..n_rules {
        let rname = format!("mrule{i}");
        // RHS references the first two tokens (or just the first if only one)
        let rhs: Vec<&str> = tok_names.iter().take(2).map(|s| s.as_str()).collect();
        b = b.rule(&rname, rhs);
    }
    if n_rules > 0 {
        b = b.start("mrule0");
    }
    b.build()
}

fn generate(grammar: &adze_ir::Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate must succeed")
}

fn parse_array(json: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(json).expect("valid JSON");
    v.as_array().expect("JSON array").clone()
}

// ───────────────────────────────────────────────────────────────────────
// Property tests — core invariants
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Any valid grammar → generate succeeds
    #[test]
    fn prop_generate_succeeds(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok(), "generate failed: {:?}", result.err());
    }

    // 2. Output is valid JSON
    #[test]
    fn prop_output_is_valid_json(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let output = generate(&g);
        let parsed: Result<Value, _> = serde_json::from_str(&output);
        prop_assert!(parsed.is_ok(), "invalid JSON: {}", output);
    }

    // 3. Output is a JSON array
    #[test]
    fn prop_output_is_json_array(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let output = generate(&g);
        let v: Value = serde_json::from_str(&output).unwrap();
        prop_assert!(v.is_array(), "expected JSON array, got: {}", v);
    }

    // 4. Array is non-empty (grammar has visible symbols)
    #[test]
    fn prop_array_is_non_empty(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        prop_assert!(!arr.is_empty(), "array should not be empty");
    }

    // 5. Each entry has "type" field
    #[test]
    fn prop_every_entry_has_type(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        for (i, entry) in arr.iter().enumerate() {
            prop_assert!(
                entry.get("type").is_some(),
                "entry {i} missing 'type': {entry}"
            );
        }
    }

    // 6. Each entry has "named" field
    #[test]
    fn prop_every_entry_has_named(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        for (i, entry) in arr.iter().enumerate() {
            prop_assert!(
                entry.get("named").is_some(),
                "entry {i} missing 'named': {entry}"
            );
        }
    }

    // 7. "type" values are strings
    #[test]
    fn prop_type_values_are_strings(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        for (i, entry) in arr.iter().enumerate() {
            let ty = entry.get("type").unwrap();
            prop_assert!(ty.is_string(), "entry {i} 'type' is not a string: {ty}");
        }
    }

    // 8. "named" values are booleans
    #[test]
    fn prop_named_values_are_booleans(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        for (i, entry) in arr.iter().enumerate() {
            let named = entry.get("named").unwrap();
            prop_assert!(named.is_boolean(), "entry {i} 'named' is not boolean: {named}");
        }
    }

    // 9. Output is deterministic
    #[test]
    fn prop_output_is_deterministic(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let a = generate(&g);
        let b = generate(&g);
        prop_assert_eq!(a, b, "two runs differ for same grammar");
    }

    // 10. Output length > 0
    #[test]
    fn prop_output_length_positive(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let output = generate(&g);
        prop_assert!(!output.is_empty(), "output must not be empty");
    }

    // 11. Type names are non-empty strings
    #[test]
    fn prop_type_names_non_empty(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        for entry in &arr {
            let name = entry["type"].as_str().unwrap();
            prop_assert!(!name.is_empty(), "type name must not be empty");
        }
    }

    // 12. No duplicate (type, named) pairs
    #[test]
    fn prop_no_duplicate_type_named_pairs(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        let mut seen = HashSet::new();
        for entry in &arr {
            let key = format!("{}:{}", entry["type"], entry["named"]);
            prop_assert!(seen.insert(key.clone()), "duplicate entry: {key}");
        }
    }

    // 13. Multi-token grammar: generate succeeds
    #[test]
    fn prop_multi_token_generate_ok(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_multi_token_grammar(n_tok, n_rule);
        prop_assert!(NodeTypesGenerator::new(&g).generate().is_ok());
    }

    // 14. Multi-token grammar: output is valid JSON array
    #[test]
    fn prop_multi_token_valid_json_array(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_multi_token_grammar(n_tok, n_rule);
        let output = generate(&g);
        let v: Value = serde_json::from_str(&output).unwrap();
        prop_assert!(v.is_array());
    }

    // 15. Multi-token grammar: each entry has required fields
    #[test]
    fn prop_multi_token_entries_have_fields(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_multi_token_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        for entry in &arr {
            prop_assert!(entry.get("type").is_some());
            prop_assert!(entry.get("named").is_some());
        }
    }

    // 16. Multi-token grammar: deterministic
    #[test]
    fn prop_multi_token_deterministic(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_multi_token_grammar(n_tok, n_rule);
        let a = generate(&g);
        let b = generate(&g);
        prop_assert_eq!(a, b);
    }

    // 17. JSON roundtrip: parse → serialize → parse yields same structure
    #[test]
    fn prop_json_roundtrip(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let output = generate(&g);
        let v1: Value = serde_json::from_str(&output).unwrap();
        let reserialized = serde_json::to_string_pretty(&v1).unwrap();
        let v2: Value = serde_json::from_str(&reserialized).unwrap();
        prop_assert_eq!(v1, v2);
    }

    // 18. Output starts with '[' (trimmed)
    #[test]
    fn prop_output_starts_with_bracket(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let output = generate(&g);
        prop_assert!(output.trim_start().starts_with('['), "must start with [");
    }

    // 19. Output ends with ']' (trimmed)
    #[test]
    fn prop_output_ends_with_bracket(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let output = generate(&g);
        prop_assert!(output.trim_end().ends_with(']'), "must end with ]");
    }

    // 20. Only allowed top-level keys in each entry
    #[test]
    fn prop_only_allowed_keys(n_tok in arb_token_count(), n_rule in arb_rule_count()) {
        let g = make_grammar(n_tok, n_rule);
        let arr = parse_array(&generate(&g));
        let allowed: HashSet<&str> =
            ["type", "named", "fields", "children", "subtypes"].iter().copied().collect();
        for entry in &arr {
            if let Some(obj) = entry.as_object() {
                for key in obj.keys() {
                    prop_assert!(
                        allowed.contains(key.as_str()),
                        "unexpected key '{key}' in entry"
                    );
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// Unit tests — specific grammar patterns
// ───────────────────────────────────────────────────────────────────────

// 21. Single-rule single-token grammar
#[test]
fn unit_single_rule_single_token() {
    let g = GrammarBuilder::new("u21")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(!arr.is_empty());
}

// 22. Grammar with regex token
#[test]
fn unit_regex_token() {
    let g = GrammarBuilder::new("u22")
        .token("num", r"\d+")
        .rule("root", vec!["num"])
        .start("root")
        .build();
    let output = generate(&g);
    let arr = parse_array(&output);
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("root")));
}

// 23. Grammar with multiple rules
#[test]
fn unit_multiple_rules() {
    let g = GrammarBuilder::new("u23")
        .token("a", "a")
        .token("b", "b")
        .rule("alpha", vec!["a"])
        .rule("beta", vec!["b"])
        .start("alpha")
        .build();
    let arr = parse_array(&generate(&g));
    let names: Vec<_> = arr.iter().filter_map(|e| e["type"].as_str()).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

// 24. Grammar with precedence
#[test]
fn unit_precedence_grammar() {
    let g = GrammarBuilder::new("u24")
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

// 25. Precedence grammar output is valid JSON
#[test]
fn unit_precedence_valid_json() {
    let g = GrammarBuilder::new("u25")
        .token("n", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let output = generate(&g);
    let v: Value = serde_json::from_str(&output).unwrap();
    assert!(v.is_array());
}

// 26. Grammar with extras
#[test]
fn unit_grammar_with_extras() {
    let g = GrammarBuilder::new("u26")
        .token("a", "a")
        .token("ws", r"\s+")
        .rule("root", vec!["a"])
        .extra("ws")
        .start("root")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 27. Grammar with inline rules
#[test]
fn unit_grammar_with_inline() {
    let g = GrammarBuilder::new("u27")
        .token("x", "x")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 28. Grammar with supertypes
#[test]
fn unit_grammar_with_supertype() {
    let g = GrammarBuilder::new("u28")
        .token("n", r"\d+")
        .token("s", r#""[^"]*""#)
        .rule("literal", vec!["n"])
        .rule("literal", vec!["s"])
        .rule("expr", vec!["literal"])
        .supertype("expr")
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 29. Grammar with externals
#[test]
fn unit_grammar_with_external() {
    let g = GrammarBuilder::new("u29")
        .token("a", "a")
        .rule("start", vec!["a"])
        .external("indent")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 30. Large grammar — 20 tokens + 10 rules
#[test]
fn unit_large_grammar() {
    let mut b = GrammarBuilder::new("u30");
    for i in 0..20 {
        let name = format!("ltok{i}");
        b = b.token(&name, &name);
    }
    for i in 0..10 {
        let rname = format!("lrule{i}");
        b = b.rule(&rname, vec!["ltok0"]);
    }
    b = b.start("lrule0");
    let g = b.build();
    let arr = parse_array(&generate(&g));
    assert!(arr.len() >= 10);
}

// 31. Large grammar output is valid JSON array
#[test]
fn unit_large_grammar_valid_json() {
    let mut b = GrammarBuilder::new("u31");
    for i in 0..15 {
        let name = format!("t{i}");
        b = b.token(&name, &name);
    }
    for i in 0..8 {
        let name = format!("r{i}");
        b = b.rule(&name, vec!["t0"]);
    }
    b = b.start("r0");
    let g = b.build();
    let v: Value = serde_json::from_str(&generate(&g)).unwrap();
    assert!(v.is_array());
}

// 32. Each entry in large grammar has "type" and "named"
#[test]
fn unit_large_grammar_entries_have_fields() {
    let mut b = GrammarBuilder::new("u32");
    for i in 0..10 {
        let name = format!("tk{i}");
        b = b.token(&name, &name);
    }
    for i in 0..5 {
        let name = format!("rl{i}");
        b = b.rule(&name, vec!["tk0"]);
    }
    b = b.start("rl0");
    let arr = parse_array(&generate(&b.build()));
    for entry in &arr {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

// 33. Determinism for a specific grammar
#[test]
fn unit_determinism_specific() {
    let build = || {
        GrammarBuilder::new("u33")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s")
            .build()
    };
    let a = generate(&build());
    let b = generate(&build());
    assert_eq!(a, b);
}

// 34. String tokens appear as anonymous (named=false)
#[test]
fn unit_string_tokens_anonymous() {
    let g = GrammarBuilder::new("u34")
        .token("+", "+")
        .token("n", r"\d+")
        .rule("expr", vec!["n", "+", "n"])
        .start("expr")
        .build();
    let arr = parse_array(&generate(&g));
    if let Some(plus) = arr.iter().find(|e| e["type"].as_str() == Some("+")) {
        assert_eq!(plus["named"], false);
    }
}

// 35. Named rules have named=true
#[test]
fn unit_named_rules_are_named() {
    let g = GrammarBuilder::new("u35")
        .token("x", "x")
        .rule("stmt", vec!["x"])
        .start("stmt")
        .build();
    let arr = parse_array(&generate(&g));
    if let Some(stmt) = arr.iter().find(|e| e["type"].as_str() == Some("stmt")) {
        assert_eq!(stmt["named"], true);
    }
}

// 36. Grammar with right associativity
#[test]
fn unit_right_assoc() {
    let g = GrammarBuilder::new("u36")
        .token("n", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let output = generate(&g);
    let v: Value = serde_json::from_str(&output).unwrap();
    assert!(v.is_array());
}

// 37. Grammar with non-associative operator
#[test]
fn unit_none_assoc() {
    let g = GrammarBuilder::new("u37")
        .token("n", r"\d+")
        .token("==", "==")
        .rule_with_precedence("cmp", vec!["expr", "==", "expr"], 1, Associativity::None)
        .rule("expr", vec!["n"])
        .start("cmp")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 38. Grammar with multiple alternative productions for same rule
#[test]
fn unit_multiple_alternatives() {
    let g = GrammarBuilder::new("u38")
        .token("n", r"\d+")
        .token("s", r#""[^"]*""#)
        .token("t", "true")
        .rule("value", vec!["n"])
        .rule("value", vec!["s"])
        .rule("value", vec!["t"])
        .start("value")
        .build();
    let arr = parse_array(&generate(&g));
    // "value" should appear once, not three times
    let value_count = arr
        .iter()
        .filter(|e| e["type"].as_str() == Some("value"))
        .count();
    assert_eq!(value_count, 1);
}

// 39. All feature types combined
#[test]
fn unit_all_features_combined() {
    let g = GrammarBuilder::new("u39")
        .token("n", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("ws", r"\s+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["n"])
        .rule("program", vec!["expr"])
        .extra("ws")
        .external("indent")
        .start("program")
        .build();
    let output = generate(&g);
    let v: Value = serde_json::from_str(&output).unwrap();
    assert!(v.is_array());
    let arr = v.as_array().unwrap();
    assert!(!arr.is_empty());
}

// 40. "fields" is an object when present
#[test]
fn unit_fields_is_object_when_present() {
    let g = GrammarBuilder::new("u40")
        .token("n", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["n", "+", "n"])
        .start("expr")
        .build();
    let arr = parse_array(&generate(&g));
    for entry in &arr {
        if let Some(fields) = entry.get("fields") {
            assert!(fields.is_object(), "fields must be object: {fields}");
        }
    }
}

// 41. "children" is an object when present
#[test]
fn unit_children_is_object_when_present() {
    let g = GrammarBuilder::new("u41")
        .token("n", r"\d+")
        .rule("list", vec!["item"])
        .rule("item", vec!["n"])
        .start("list")
        .build();
    let arr = parse_array(&generate(&g));
    for entry in &arr {
        if let Some(children) = entry.get("children") {
            assert!(children.is_object(), "children must be object: {children}");
        }
    }
}

// 42. "subtypes" is an array when present
#[test]
fn unit_subtypes_is_array_when_present() {
    let g = GrammarBuilder::new("u42")
        .token("n", r"\d+")
        .rule("expr", vec!["n"])
        .supertype("expr")
        .start("expr")
        .build();
    let arr = parse_array(&generate(&g));
    for entry in &arr {
        if let Some(subtypes) = entry.get("subtypes") {
            assert!(subtypes.is_array(), "subtypes must be array: {subtypes}");
        }
    }
}

// 43. Tree-like grammar: nested rules
#[test]
fn unit_tree_grammar() {
    let g = GrammarBuilder::new("u43")
        .token("n", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .rule("tree", vec!["n"])
        .rule("tree", vec!["(", "tree", ",", "tree", ")"])
        .start("tree")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 44. List-like grammar: repetitive structure
#[test]
fn unit_list_grammar() {
    let g = GrammarBuilder::new("u44")
        .token("item", r"[a-z]+")
        .token(",", ",")
        .rule("list", vec!["item"])
        .rule("list", vec!["list", ",", "item"])
        .start("list")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("list")));
}

// 45. Expression grammar pattern
#[test]
fn unit_expr_grammar_pattern() {
    let g = GrammarBuilder::new("u45")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["id"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("expr")));
}

// 46. Statement/block grammar pattern
#[test]
fn unit_statement_block_grammar() {
    let g = GrammarBuilder::new("u46")
        .token("id", r"[a-z]+")
        .token(";", ";")
        .token("{", "{")
        .token("}", "}")
        .rule("stmt", vec!["id", ";"])
        .rule("block", vec!["{", "stmt", "}"])
        .rule("program", vec!["block"])
        .start("program")
        .build();
    let arr = parse_array(&generate(&g));
    let names: HashSet<_> = arr.iter().filter_map(|e| e["type"].as_str()).collect();
    assert!(names.contains("stmt"));
    assert!(names.contains("block"));
    assert!(names.contains("program"));
}

// 47. Grammar with only string tokens (all anonymous)
#[test]
fn unit_only_string_tokens() {
    let g = GrammarBuilder::new("u47")
        .token("+", "+")
        .token("-", "-")
        .rule("op", vec!["+"])
        .rule("op", vec!["-"])
        .start("op")
        .build();
    let arr = parse_array(&generate(&g));
    // "op" should be named, tokens should be anonymous
    for entry in &arr {
        let is_named = entry["named"].as_bool().unwrap();
        let ty = entry["type"].as_str().unwrap();
        if ty == "op" {
            assert!(is_named);
        }
    }
}

// 48. Scaled grammar: 1 token, 1 rule (minimal)
#[test]
fn unit_minimal_grammar() {
    let g = make_grammar(1, 1);
    let arr = parse_array(&generate(&g));
    assert!(!arr.is_empty());
}

// 49. Scaled grammar: 7 tokens, 4 rules (max strategy values)
#[test]
fn unit_max_strategy_grammar() {
    let g = make_grammar(7, 4);
    let arr = parse_array(&generate(&g));
    assert!(arr.len() >= 4);
}

// 50. Output contains no null entries
#[test]
fn unit_no_null_entries() {
    let g = make_grammar(3, 2);
    let arr = parse_array(&generate(&g));
    for entry in &arr {
        assert!(!entry.is_null(), "null entry in output");
    }
}

// 51. Type values contain no embedded newlines
#[test]
fn unit_type_names_no_newlines() {
    let g = make_grammar(4, 3);
    let arr = parse_array(&generate(&g));
    for entry in &arr {
        let ty = entry["type"].as_str().unwrap();
        assert!(!ty.contains('\n'), "type name has newline: {ty:?}");
    }
}

// 52. Grammar name doesn't affect output structure
#[test]
fn unit_grammar_name_independent() {
    let build = |name: &str| {
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build()
    };
    let a = parse_array(&generate(&build("name_a")));
    let b = parse_array(&generate(&build("name_b")));
    assert_eq!(a.len(), b.len());
    for (ea, eb) in a.iter().zip(b.iter()) {
        assert_eq!(ea["type"], eb["type"]);
        assert_eq!(ea["named"], eb["named"]);
    }
}

// 53. Two-token grammar with both string and regex
#[test]
fn unit_mixed_token_types() {
    let g = GrammarBuilder::new("u53")
        .token("+", "+")
        .token("num", r"\d+")
        .rule("add", vec!["num", "+", "num"])
        .start("add")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("add")));
}

// 54. Chained rules: a → b → c
#[test]
fn unit_chained_rules() {
    let g = GrammarBuilder::new("u54")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
        .build();
    let arr = parse_array(&generate(&g));
    let names: HashSet<_> = arr.iter().filter_map(|e| e["type"].as_str()).collect();
    assert!(names.contains("a"));
    assert!(names.contains("b"));
    assert!(names.contains("c"));
}

// 55. Recursive rule
#[test]
fn unit_recursive_rule() {
    let g = GrammarBuilder::new("u55")
        .token("n", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "n"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 56. Mutual recursion
#[test]
fn unit_mutual_recursion() {
    let g = GrammarBuilder::new("u56")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["b", "x"])
        .rule("b", vec!["a", "y"])
        .rule("a", vec!["x"])
        .start("a")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 57. Many tokens, single rule
#[test]
fn unit_many_tokens_one_rule() {
    let mut b = GrammarBuilder::new("u57");
    for i in 0..15 {
        let name = format!("t{i}");
        b = b.token(&name, &name);
    }
    b = b.rule("start", vec!["t0"]).start("start");
    let arr = parse_array(&generate(&b.build()));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("start")));
}

// 58. Single token, many rules
#[test]
fn unit_one_token_many_rules() {
    let mut b = GrammarBuilder::new("u58").token("x", "x");
    for i in 0..10 {
        let name = format!("r{i}");
        b = b.rule(&name, vec!["x"]);
    }
    b = b.start("r0");
    let arr = parse_array(&generate(&b.build()));
    assert!(arr.len() >= 10);
}

// 59. Output sorted by type name
#[test]
fn unit_output_sorted() {
    let g = GrammarBuilder::new("u59")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("zeta", vec!["a"])
        .rule("alpha", vec!["b"])
        .rule("mu", vec!["c"])
        .start("alpha")
        .build();
    let arr = parse_array(&generate(&g));
    let names: Vec<_> = arr.iter().filter_map(|e| e["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

// 60. Precedence levels don't break JSON structure
#[test]
fn unit_precedence_levels_json_ok() {
    let g = GrammarBuilder::new("u60")
        .token("n", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("/", "/")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let output = generate(&g);
    let v: Value = serde_json::from_str(&output).unwrap();
    assert!(v.is_array());
}

// 61. Python-like grammar pattern
#[test]
fn unit_python_like() {
    let g = GrammarBuilder::new("u61")
        .token("id", r"[a-z_]+")
        .token("num", r"\d+")
        .token("=", "=")
        .token(":", ":")
        .token("if", "if")
        .rule("assign", vec!["id", "=", "expr"])
        .rule("expr", vec!["num"])
        .rule("expr", vec!["id"])
        .rule("ifstmt", vec!["if", "expr", ":", "stmt"])
        .rule("stmt", vec!["assign"])
        .rule("stmt", vec!["ifstmt"])
        .start("stmt")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 62. JavaScript-like grammar pattern
#[test]
fn unit_js_like() {
    let g = GrammarBuilder::new("u62")
        .token("id", r"[a-zA-Z_$]+")
        .token("num", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .token("function", "function")
        .rule("func", vec!["function", "id", "(", ")", "{", "}"])
        .rule("call", vec!["id", "(", ")"])
        .rule("program", vec!["func"])
        .start("program")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("func")));
}

// 63. Go-like grammar pattern
#[test]
fn unit_go_like() {
    let g = GrammarBuilder::new("u63")
        .token("id", r"[a-z]+")
        .token("func", "func")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .rule("funcdecl", vec!["func", "id", "(", ")", "{", "}"])
        .rule("source", vec!["funcdecl"])
        .start("source")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 64. Ensure every named entry has a non-empty string type
#[test]
fn unit_named_entries_valid() {
    let g = GrammarBuilder::new("u64")
        .token("x", "x")
        .token("y", "y")
        .rule("foo", vec!["x"])
        .rule("bar", vec!["y"])
        .start("foo")
        .build();
    let arr = parse_array(&generate(&g));
    for entry in arr.iter().filter(|e| e["named"] == true) {
        let ty = entry["type"].as_str().unwrap();
        assert!(!ty.is_empty());
    }
}

// 65. Ensure anonymous entries have non-empty type
#[test]
fn unit_anonymous_entries_valid() {
    let g = GrammarBuilder::new("u65")
        .token("+", "+")
        .token("n", r"\d+")
        .rule("expr", vec!["n", "+", "n"])
        .start("expr")
        .build();
    let arr = parse_array(&generate(&g));
    for entry in arr.iter().filter(|e| e["named"] == false) {
        let ty = entry["type"].as_str().unwrap();
        assert!(!ty.is_empty());
    }
}

// 66. Scaled grammar: various sizes produce valid output
#[test]
fn unit_scaled_sizes() {
    for n in 1..=7 {
        let g = make_grammar(n, n.min(4));
        let arr = parse_array(&generate(&g));
        assert!(!arr.is_empty(), "empty for n={n}");
    }
}

// 67. All entries are JSON objects
#[test]
fn unit_all_entries_are_objects() {
    let g = make_grammar(4, 3);
    let arr = parse_array(&generate(&g));
    for (i, entry) in arr.iter().enumerate() {
        assert!(entry.is_object(), "entry {i} is not an object");
    }
}

// 68. Grammar with both extras and externals
#[test]
fn unit_extras_and_externals() {
    let g = GrammarBuilder::new("u68")
        .token("n", r"\d+")
        .token("ws", r"\s+")
        .rule("root", vec!["n"])
        .extra("ws")
        .external("comment")
        .start("root")
        .build();
    let output = generate(&g);
    let v: Value = serde_json::from_str(&output).unwrap();
    assert!(v.is_array());
}

// 69. Grammar with inline + supertype
#[test]
fn unit_inline_and_supertype() {
    let g = GrammarBuilder::new("u69")
        .token("n", r"\d+")
        .rule("expr", vec!["primary"])
        .rule("primary", vec!["n"])
        .inline("primary")
        .supertype("expr")
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 70. Deep chain: 10 levels of rules
#[test]
fn unit_deep_chain() {
    let mut b = GrammarBuilder::new("u70").token("leaf", "leaf");
    let names: Vec<String> = (0..10).map(|i| format!("level{i}")).collect();
    for i in (1..10).rev() {
        b = b.rule(&names[i], vec![&names[i - 1]]);
    }
    b = b.rule(&names[0], vec!["leaf"]).start(&names[9]);
    let g = b.build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 71. Empty-ish grammar: only a token, no rules
#[test]
fn unit_token_only_no_rules() {
    let g = GrammarBuilder::new("u71").token("a", "a").build();
    // Should still succeed even with no rules
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 72. Grammar with two supertypes
#[test]
fn unit_two_supertypes() {
    let g = GrammarBuilder::new("u72")
        .token("n", r"\d+")
        .token("s", r"[a-z]+")
        .rule("expr", vec!["n"])
        .rule("stmt", vec!["s"])
        .supertype("expr")
        .supertype("stmt")
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 73. Verify re-generation from same builder yields same JSON
#[test]
fn unit_builder_determinism() {
    let build = || {
        GrammarBuilder::new("u73")
            .token("a", "a")
            .token("b", "b")
            .rule("root", vec!["a", "b"])
            .start("root")
            .build()
    };
    assert_eq!(generate(&build()), generate(&build()));
}

// 74. Grammar with 3 precedence levels
#[test]
fn unit_three_prec_levels() {
    let g = GrammarBuilder::new("u74")
        .token("n", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("expr")));
}

// 75. Grammar with all three associativities
#[test]
fn unit_all_associativities() {
    let g = GrammarBuilder::new("u75")
        .token("n", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .token("==", "==")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 2, Associativity::Right)
        .rule_with_precedence("expr", vec!["expr", "==", "expr"], 0, Associativity::None)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 76. Multiple extras don't break output
#[test]
fn unit_multiple_extras() {
    let g = GrammarBuilder::new("u76")
        .token("x", "x")
        .token("ws", r"\s+")
        .token("nl", r"\n")
        .rule("start", vec!["x"])
        .extra("ws")
        .extra("nl")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 77. Multiple externals
#[test]
fn unit_multiple_externals() {
    let g = GrammarBuilder::new("u77")
        .token("x", "x")
        .rule("start", vec!["x"])
        .external("indent")
        .external("dedent")
        .external("newline")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 78. Verify output contains at least the rule count of named entries
#[test]
fn unit_named_count_ge_rules() {
    let g = GrammarBuilder::new("u78")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("r3", vec!["c"])
        .start("r1")
        .build();
    let arr = parse_array(&generate(&g));
    let named_count = arr.iter().filter(|e| e["named"] == true).count();
    assert!(
        named_count >= 3,
        "expected >= 3 named entries, got {named_count}"
    );
}

// 79. Binary expression pattern
#[test]
fn unit_binary_expr_pattern() {
    let g = GrammarBuilder::new("u79")
        .token("n", r"\d+")
        .token("+", "+")
        .rule("binexpr", vec!["n", "+", "n"])
        .start("binexpr")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("binexpr")));
}

// 80. Unary expression pattern
#[test]
fn unit_unary_expr_pattern() {
    let g = GrammarBuilder::new("u80")
        .token("n", r"\d+")
        .token("-", "-")
        .rule("neg", vec!["-", "n"])
        .start("neg")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("neg")));
}

// 81. Conditional grammar pattern
#[test]
fn unit_conditional_pattern() {
    let g = GrammarBuilder::new("u81")
        .token("id", r"[a-z]+")
        .token("if", "if")
        .token("then", "then")
        .token("else", "else")
        .rule("cond", vec!["if", "id", "then", "id", "else", "id"])
        .start("cond")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 82. Assignment grammar pattern
#[test]
fn unit_assignment_pattern() {
    let g = GrammarBuilder::new("u82")
        .token("id", r"[a-z]+")
        .token("=", "=")
        .token("n", r"\d+")
        .rule("assign", vec!["id", "=", "n"])
        .start("assign")
        .build();
    let arr = parse_array(&generate(&g));
    assert!(arr.iter().any(|e| e["type"].as_str() == Some("assign")));
}

// 83. Grammar name with special characters
#[test]
fn unit_grammar_name_special() {
    let g = GrammarBuilder::new("test-grammar_v2")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// 84. Verify JSON pretty-printing (contains newlines)
#[test]
fn unit_json_is_pretty_printed() {
    let g = GrammarBuilder::new("u84")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let output = generate(&g);
    assert!(output.contains('\n'), "output should be pretty-printed");
}

// 85. Grammar with 5 rules, each referencing different tokens
#[test]
fn unit_five_rules_different_tokens() {
    let g = GrammarBuilder::new("u85")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("ra", vec!["a"])
        .rule("rb", vec!["b"])
        .rule("rc", vec!["c"])
        .rule("rd", vec!["d"])
        .rule("re", vec!["e"])
        .start("ra")
        .build();
    let arr = parse_array(&generate(&g));
    for name in &["ra", "rb", "rc", "rd", "re"] {
        assert!(
            arr.iter().any(|e| e["type"].as_str() == Some(name)),
            "missing rule {name}"
        );
    }
}
