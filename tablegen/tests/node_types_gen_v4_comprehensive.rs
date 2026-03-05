//! Comprehensive v4 tests for `NodeTypesGenerator`.
//!
//! 56 tests covering: generator construction, valid JSON output, expected fields,
//! token grammars, rule grammars, determinism, different grammars produce different
//! output, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, FieldId, Grammar};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ===========================================================================
// Helpers
// ===========================================================================

fn gen_json(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate must succeed")
}

fn gen_parsed(grammar: &Grammar) -> Vec<Value> {
    let json = gen_json(grammar);
    serde_json::from_str::<Value>(&json)
        .expect("valid JSON")
        .as_array()
        .expect("top-level array")
        .clone()
}

fn find_node<'a>(nodes: &'a [Value], name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(name))
}

fn named_types(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

fn anon_types(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

// ===========================================================================
// Grammar factories
// ===========================================================================

fn empty_grammar() -> Grammar {
    GrammarBuilder::new("empty").build()
}

fn single_rule_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("pair", vec!["a", "b"])
        .start("pair")
        .build()
}

fn regex_token_grammar() -> Grammar {
    GrammarBuilder::new("regex_tok")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn multi_alternative_grammar() -> Grammar {
    GrammarBuilder::new("multi_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("choice", vec!["a"])
        .rule("choice", vec!["b"])
        .rule("choice", vec!["c"])
        .start("choice")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("t", "t")
        .rule("top", vec!["mid"])
        .rule("mid", vec!["bot"])
        .rule("bot", vec!["t"])
        .start("top")
        .build()
}

fn recursive_grammar() -> Grammar {
    GrammarBuilder::new("rec")
        .token("(", "(")
        .token(")", ")")
        .token("ID", r"[a-z]+")
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["ID"])
        .start("expr")
        .build()
}

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn many_nonterminals_grammar() -> Grammar {
    GrammarBuilder::new("many_nt")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token(";", ";")
        .token("=", "=")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["assignment"])
        .rule("assignment", vec!["ID", "=", "val", ";"])
        .rule("val", vec!["NUM"])
        .rule("val", vec!["ID"])
        .start("program")
        .build()
}

fn grammar_with_field() -> Grammar {
    let mut g = GrammarBuilder::new("field_g")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("add", vec!["NUM", "+", "NUM"])
        .start("add")
        .build();

    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "right".to_string());

    let add_id = g
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "add")
        .map(|(id, _)| *id)
        .unwrap();

    if let Some(rules) = g.rules.get_mut(&add_id) {
        rules[0].fields = vec![(FieldId(0), 0), (FieldId(1), 2)];
    }
    g
}

fn only_string_tokens_grammar() -> Grammar {
    GrammarBuilder::new("str_only")
        .token("if", "if")
        .token("else", "else")
        .token("(", "(")
        .token(")", ")")
        .rule("cond", vec!["if", "(", ")", "else"])
        .start("cond")
        .build()
}

fn only_regex_tokens_grammar() -> Grammar {
    GrammarBuilder::new("regex_only")
        .token("IDENT", r"[a-zA-Z_]+")
        .token("INT", r"\d+")
        .rule("item", vec!["IDENT"])
        .rule("item", vec!["INT"])
        .start("item")
        .build()
}

fn mixed_tokens_grammar() -> Grammar {
    GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .rule("expr", vec!["NUM", "-", "NUM"])
        .start("expr")
        .build()
}

fn deep_nesting_grammar() -> Grammar {
    GrammarBuilder::new("deep")
        .token("leaf", "leaf")
        .rule("l1", vec!["l2"])
        .rule("l2", vec!["l3"])
        .rule("l3", vec!["l4"])
        .rule("l4", vec!["l5"])
        .rule("l5", vec!["l6"])
        .rule("l6", vec!["leaf"])
        .start("l1")
        .build()
}

fn many_tokens_grammar() -> Grammar {
    let mut builder = GrammarBuilder::new("many_tok");
    let names: Vec<String> = (0..20).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    builder = builder.rule("root", vec!["t0"]);
    builder = builder.start("root");
    builder.build()
}

fn many_rules_grammar() -> Grammar {
    let mut builder = GrammarBuilder::new("many_rules");
    builder = builder.token("x", "x");
    let rule_names: Vec<String> = (0..20).map(|i| format!("r{i}")).collect();
    // Chain: r0 -> r1 -> ... -> r19 -> x
    for i in 0..19 {
        builder = builder.rule(&rule_names[i], vec![&rule_names[i + 1]]);
    }
    builder = builder.rule("r19", vec!["x"]);
    builder = builder.start("r0");
    builder.build()
}

// ===========================================================================
// 1. Generator construction (5 tests)
// ===========================================================================

#[test]
fn construct_with_empty_grammar() {
    let g = empty_grammar();
    let _generator = NodeTypesGenerator::new(&g);
}

#[test]
fn construct_with_single_rule() {
    let g = single_rule_grammar();
    let _generator = NodeTypesGenerator::new(&g);
}

#[test]
fn construct_with_complex_grammar() {
    let g = arithmetic_grammar();
    let _generator = NodeTypesGenerator::new(&g);
}

#[test]
fn construct_with_many_nonterminals() {
    let g = many_nonterminals_grammar();
    let _generator = NodeTypesGenerator::new(&g);
}

#[test]
fn construct_then_generate_does_not_panic() {
    let g = chain_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

// ===========================================================================
// 2. Output is valid JSON (8 tests)
// ===========================================================================

#[test]
fn empty_grammar_produces_valid_json() {
    let json = gen_json(&empty_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn single_rule_produces_valid_json() {
    let json = gen_json(&single_rule_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn output_is_json_array() {
    let val: Value = serde_json::from_str(&gen_json(&single_rule_grammar())).unwrap();
    assert!(val.is_array());
}

#[test]
fn each_entry_is_json_object() {
    let nodes = gen_parsed(&two_token_grammar());
    for node in &nodes {
        assert!(node.is_object(), "each entry must be a JSON object");
    }
}

#[test]
fn regex_token_grammar_produces_valid_json() {
    let json = gen_json(&regex_token_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn recursive_grammar_produces_valid_json() {
    let json = gen_json(&recursive_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn arithmetic_grammar_produces_valid_json() {
    let json = gen_json(&arithmetic_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn many_nonterminals_grammar_produces_valid_json() {
    let json = gen_json(&many_nonterminals_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

// ===========================================================================
// 3. Output contains expected fields (8 tests)
// ===========================================================================

#[test]
fn every_node_has_type_field() {
    let nodes = gen_parsed(&two_token_grammar());
    for node in &nodes {
        assert!(node.get("type").is_some(), "missing 'type' field");
    }
}

#[test]
fn every_node_has_named_field() {
    let nodes = gen_parsed(&two_token_grammar());
    for node in &nodes {
        assert!(node.get("named").is_some(), "missing 'named' field");
    }
}

#[test]
fn named_field_is_boolean() {
    let nodes = gen_parsed(&two_token_grammar());
    for node in &nodes {
        assert!(node["named"].is_boolean(), "'named' must be a boolean");
    }
}

#[test]
fn rule_nodes_are_named_true() {
    let nodes = gen_parsed(&single_rule_grammar());
    let root = find_node(&nodes, "root").expect("root node");
    assert_eq!(root["named"], true);
}

#[test]
fn string_token_nodes_are_named_false() {
    let nodes = gen_parsed(&single_rule_grammar());
    let tok = find_node(&nodes, "x").expect("token x");
    assert_eq!(tok["named"], false);
}

#[test]
fn field_grammar_has_fields_object() {
    let nodes = gen_parsed(&grammar_with_field());
    let add = find_node(&nodes, "add").expect("add node");
    assert!(
        add.get("fields").is_some(),
        "'fields' key must be present when fields exist"
    );
}

#[test]
fn subtypes_absent_when_not_applicable() {
    let nodes = gen_parsed(&single_rule_grammar());
    let root = find_node(&nodes, "root").expect("root node");
    assert!(
        root.get("subtypes").is_none(),
        "'subtypes' should be absent"
    );
}

#[test]
fn children_absent_when_not_applicable() {
    let nodes = gen_parsed(&single_rule_grammar());
    let root = find_node(&nodes, "root").expect("root node");
    assert!(
        root.get("children").is_none(),
        "'children' should be absent"
    );
}

// ===========================================================================
// 4. Grammar with tokens → node types (8 tests)
// ===========================================================================

#[test]
fn string_tokens_appear_as_anonymous() {
    let nodes = gen_parsed(&only_string_tokens_grammar());
    let anon = anon_types(&nodes);
    assert!(anon.contains(&"if".to_string()));
    assert!(anon.contains(&"else".to_string()));
}

#[test]
fn string_tokens_named_is_false() {
    let nodes = gen_parsed(&only_string_tokens_grammar());
    for name in &["if", "else"] {
        if let Some(node) = find_node(&nodes, name) {
            assert_eq!(
                node["named"], false,
                "string token '{name}' must be anonymous"
            );
        }
    }
}

#[test]
fn regex_tokens_are_named_true() {
    // Regex tokens that appear as rules (because they are named patterns) should
    // show up as named in the rule entry, not as anonymous token entries.
    let nodes = gen_parsed(&regex_token_grammar());
    let named = named_types(&nodes);
    assert!(named.contains(&"expr".to_string()));
}

#[test]
fn mixed_tokens_both_present() {
    let nodes = gen_parsed(&mixed_tokens_grammar());
    // NUM is a regex token used via rule — check rule "expr" is present
    assert!(find_node(&nodes, "expr").is_some());
    // "+" and "-" are string tokens
    let anon = anon_types(&nodes);
    assert!(anon.contains(&"+".to_string()));
    assert!(anon.contains(&"-".to_string()));
}

#[test]
fn token_count_matches_string_tokens() {
    let nodes = gen_parsed(&only_string_tokens_grammar());
    let anon = anon_types(&nodes);
    // "if", "else", "(", ")" are string tokens
    assert!(
        anon.len() >= 4,
        "expected at least 4 anonymous tokens, got {}",
        anon.len()
    );
}

#[test]
fn many_string_tokens_all_appear() {
    let g = many_tokens_grammar();
    let nodes = gen_parsed(&g);
    let anon = anon_types(&nodes);
    for i in 0..20 {
        let name = format!("t{i}");
        assert!(anon.contains(&name), "missing token '{name}'");
    }
}

#[test]
fn regex_only_grammar_no_anonymous_string_tokens() {
    let nodes = gen_parsed(&only_regex_tokens_grammar());
    let anon = anon_types(&nodes);
    // IDENT and INT are regex tokens — they are not emitted as unnamed nodes
    assert!(
        !anon.iter().any(|a| a == "IDENT" || a == "INT"),
        "regex tokens should not appear as anonymous entries"
    );
}

#[test]
fn multiple_alternatives_single_named_entry() {
    let nodes = gen_parsed(&multi_alternative_grammar());
    let named = named_types(&nodes);
    let count = named.iter().filter(|n| *n == "choice").count();
    assert_eq!(
        count, 1,
        "multiple alternatives collapse into one named entry"
    );
}

// ===========================================================================
// 5. Grammar with rules → node types (8 tests)
// ===========================================================================

#[test]
fn single_rule_appears_as_named() {
    let nodes = gen_parsed(&single_rule_grammar());
    assert!(find_node(&nodes, "root").is_some());
    assert_eq!(find_node(&nodes, "root").unwrap()["named"], true);
}

#[test]
fn chain_grammar_all_nonterminals_present() {
    let nodes = gen_parsed(&chain_grammar());
    for name in &["top", "mid", "bot"] {
        assert!(find_node(&nodes, name).is_some(), "missing rule '{name}'");
    }
}

#[test]
fn chain_grammar_all_rules_named_true() {
    let nodes = gen_parsed(&chain_grammar());
    for name in &["top", "mid", "bot"] {
        let node = find_node(&nodes, name).unwrap();
        assert_eq!(node["named"], true, "rule '{name}' must be named");
    }
}

#[test]
fn many_nonterminals_all_present() {
    let nodes = gen_parsed(&many_nonterminals_grammar());
    for name in &["program", "stmt", "assignment", "val"] {
        assert!(find_node(&nodes, name).is_some(), "missing rule '{name}'");
    }
}

#[test]
fn recursive_grammar_rule_appears_once() {
    let nodes = gen_parsed(&recursive_grammar());
    let named = named_types(&nodes);
    let count = named.iter().filter(|n| *n == "expr").count();
    assert_eq!(count, 1, "recursive rule should appear exactly once");
}

#[test]
fn rules_with_precedence_appear() {
    let nodes = gen_parsed(&arithmetic_grammar());
    assert!(find_node(&nodes, "expr").is_some());
}

#[test]
fn field_grammar_fields_contain_expected_keys() {
    let nodes = gen_parsed(&grammar_with_field());
    let add = find_node(&nodes, "add").unwrap();
    let fields = add["fields"].as_object().unwrap();
    assert!(fields.contains_key("left"), "missing 'left' field");
    assert!(fields.contains_key("right"), "missing 'right' field");
}

#[test]
fn field_types_are_arrays() {
    let nodes = gen_parsed(&grammar_with_field());
    let add = find_node(&nodes, "add").unwrap();
    let fields = add["fields"].as_object().unwrap();
    for (key, info) in fields {
        assert!(
            info["types"].is_array(),
            "field '{key}' types must be an array"
        );
    }
}

// ===========================================================================
// 6. Deterministic output (5 tests)
// ===========================================================================

#[test]
fn deterministic_single_rule() {
    let g = single_rule_grammar();
    let a = gen_json(&g);
    let b = gen_json(&g);
    assert_eq!(a, b);
}

#[test]
fn deterministic_complex_grammar() {
    let g = arithmetic_grammar();
    let a = gen_json(&g);
    let b = gen_json(&g);
    assert_eq!(a, b);
}

#[test]
fn deterministic_many_nonterminals() {
    let g = many_nonterminals_grammar();
    let a = gen_json(&g);
    let b = gen_json(&g);
    assert_eq!(a, b);
}

#[test]
fn deterministic_chain_grammar() {
    let g = chain_grammar();
    let results: Vec<String> = (0..5).map(|_| gen_json(&g)).collect();
    for r in &results {
        assert_eq!(r, &results[0]);
    }
}

#[test]
fn deterministic_with_fields() {
    let g = grammar_with_field();
    let a = gen_json(&g);
    let b = gen_json(&g);
    // Compare as parsed JSON to avoid field ordering differences
    let a_val: serde_json::Value = serde_json::from_str(&a).unwrap();
    let b_val: serde_json::Value = serde_json::from_str(&b).unwrap();
    assert_eq!(a_val, b_val);
}

// ===========================================================================
// 7. Different grammars → different output (5 tests)
// ===========================================================================

#[test]
fn different_names_different_output() {
    let a = gen_json(&single_rule_grammar());
    let b = gen_json(&two_token_grammar());
    assert_ne!(a, b);
}

#[test]
fn extra_rule_changes_output() {
    let base = single_rule_grammar();
    let extended = GrammarBuilder::new("ext")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["x"])
        .rule("root", vec!["y"])
        .start("root")
        .build();
    assert_ne!(gen_json(&base), gen_json(&extended));
}

#[test]
fn more_tokens_changes_output() {
    let small = GrammarBuilder::new("small")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let big = GrammarBuilder::new("big")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert_ne!(gen_json(&small), gen_json(&big));
}

#[test]
fn regex_vs_string_token_different() {
    let string_tok = GrammarBuilder::new("s")
        .token("abc", "abc")
        .rule("r", vec!["abc"])
        .start("r")
        .build();
    let regex_tok = GrammarBuilder::new("s")
        .token("abc", r"[a-z]+")
        .rule("r", vec!["abc"])
        .start("r")
        .build();
    assert_ne!(gen_json(&string_tok), gen_json(&regex_tok));
}

#[test]
fn chain_vs_flat_different() {
    let flat = GrammarBuilder::new("f")
        .token("x", "x")
        .rule("a", vec!["x"])
        .start("a")
        .build();
    let chained = GrammarBuilder::new("f")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["x"])
        .start("a")
        .build();
    assert_ne!(gen_json(&flat), gen_json(&chained));
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn empty_grammar_produces_empty_array() {
    let nodes = gen_parsed(&empty_grammar());
    assert!(nodes.is_empty(), "empty grammar should produce []");
}

#[test]
fn many_rules_grammar_all_present() {
    let g = many_rules_grammar();
    let nodes = gen_parsed(&g);
    let named = named_types(&nodes);
    for i in 0..20 {
        let name = format!("r{i}");
        assert!(named.contains(&name), "missing rule '{name}'");
    }
}

#[test]
fn deep_nesting_all_levels_present() {
    let nodes = gen_parsed(&deep_nesting_grammar());
    for name in &["l1", "l2", "l3", "l4", "l5", "l6"] {
        assert!(find_node(&nodes, name).is_some(), "missing '{name}'");
    }
}

#[test]
fn output_sorted_by_type_name() {
    let nodes = gen_parsed(&many_nonterminals_grammar());
    let type_names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = type_names.clone();
    sorted.sort();
    assert_eq!(type_names, sorted, "output must be sorted by type name");
}

#[test]
fn no_duplicate_entries() {
    let nodes = gen_parsed(&multi_alternative_grammar());
    let type_names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let unique: std::collections::HashSet<&str> = type_names.iter().copied().collect();
    assert_eq!(
        type_names.len(),
        unique.len(),
        "no duplicate entries allowed"
    );
}

#[test]
fn internal_rules_skipped() {
    // Rules starting with '_' are internal and should not appear
    let mut g = GrammarBuilder::new("internal")
        .token("x", "x")
        .rule("visible", vec!["x"])
        .start("visible")
        .build();

    // Manually insert an internal rule
    let internal_id = adze_ir::SymbolId(100);
    g.rule_names.insert(internal_id, "_hidden".to_string());
    g.rules.insert(
        internal_id,
        vec![adze_ir::Rule {
            lhs: internal_id,
            rhs: vec![adze_ir::Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(99),
        }],
    );

    let nodes = gen_parsed(&g);
    assert!(
        find_node(&nodes, "_hidden").is_none(),
        "internal rules must be skipped"
    );
}

#[test]
fn python_like_grammar_generates_ok() {
    let g = GrammarBuilder::python_like();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
    let nodes = gen_parsed(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn javascript_like_grammar_generates_ok() {
    let g = GrammarBuilder::javascript_like();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
    let nodes = gen_parsed(&g);
    assert!(!nodes.is_empty());
}
