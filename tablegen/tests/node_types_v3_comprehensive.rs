//! Comprehensive v3 tests for `NodeTypesGenerator`.
//!
//! 57 tests covering: simple grammar node types, JSON structure validity,
//! named/anonymous node types, fields, children, determinism, complex grammars,
//! and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, FieldId, Grammar};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ===========================================================================
// Helpers
// ===========================================================================

fn generate_json(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate failed")
}

fn generate_parsed(grammar: &Grammar) -> Vec<Value> {
    let json = generate_json(grammar);
    serde_json::from_str::<Value>(&json)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn find_by_type<'a>(nodes: &'a [Value], name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(name))
}

fn named_entries(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

fn anonymous_entries(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

// ===========================================================================
// Grammar factory helpers
// ===========================================================================

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build()
}

fn regex_token_grammar() -> Grammar {
    GrammarBuilder::new("regex")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn alternative_grammar() -> Grammar {
    GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
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

fn many_rules_grammar() -> Grammar {
    GrammarBuilder::new("many")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["r1"])
        .rule("s", vec!["r2"])
        .rule("r1", vec!["a", "b"])
        .rule("r2", vec!["c", "d"])
        .start("s")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep")
        .token("t", "t")
        .rule("n1", vec!["n2"])
        .rule("n2", vec!["n3"])
        .rule("n3", vec!["n4"])
        .rule("n4", vec!["n5"])
        .rule("n5", vec!["t"])
        .start("n1")
        .build()
}

fn multi_nonterminal_grammar() -> Grammar {
    GrammarBuilder::new("multi_nt")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token(";", ";")
        .token("=", "=")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["assign"])
        .rule("assign", vec!["ID", "=", "val", ";"])
        .rule("val", vec!["NUM"])
        .rule("val", vec!["ID"])
        .start("program")
        .build()
}

fn many_tokens_grammar() -> Grammar {
    GrammarBuilder::new("many_tok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("s", vec!["a", "b", "c", "d", "e", "f"])
        .start("s")
        .build()
}

/// Build a grammar where a rule has a single field pointing at a terminal.
fn grammar_with_single_field() -> Grammar {
    let mut g = GrammarBuilder::new("single_field")
        .token("NUM", r"\d+")
        .rule("value", vec!["NUM"])
        .start("value")
        .build();

    g.fields.insert(FieldId(0), "operand".to_string());

    let value_id = g
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "value")
        .map(|(id, _)| *id)
        .unwrap();

    if let Some(rules) = g.rules.get_mut(&value_id) {
        rules[0].fields = vec![(FieldId(0), 0)];
    }
    g
}

/// Build a grammar where a rule has two fields.
fn grammar_with_two_fields() -> Grammar {
    let mut g = GrammarBuilder::new("two_fields")
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
        // left → position 0 (NUM), right → position 2 (NUM)
        rules[0].fields = vec![(FieldId(0), 0), (FieldId(1), 2)];
    }
    g
}

/// Build a grammar where a field points at a non-terminal.
fn grammar_with_nonterminal_field() -> Grammar {
    let mut g = GrammarBuilder::new("nt_field")
        .token("NUM", r"\d+")
        .rule("wrapper", vec!["inner"])
        .rule("inner", vec!["NUM"])
        .start("wrapper")
        .build();

    g.fields.insert(FieldId(0), "body".to_string());

    let wrapper_id = g
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "wrapper")
        .map(|(id, _)| *id)
        .unwrap();

    if let Some(rules) = g.rules.get_mut(&wrapper_id) {
        rules[0].fields = vec![(FieldId(0), 0)];
    }
    g
}

// ===========================================================================
// 1. Simple grammar node types (10 tests)
// ===========================================================================

#[test]
fn simple_grammar_generates_ok() {
    let g = simple_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn simple_grammar_has_rule_and_token() {
    let nodes = generate_parsed(&simple_grammar());
    assert!(find_by_type(&nodes, "s").is_some());
    assert!(find_by_type(&nodes, "a").is_some());
}

#[test]
fn simple_grammar_exactly_two_entries() {
    let nodes = generate_parsed(&simple_grammar());
    assert_eq!(nodes.len(), 2);
}

#[test]
fn two_token_grammar_generates_ok() {
    assert!(
        NodeTypesGenerator::new(&two_token_grammar())
            .generate()
            .is_ok()
    );
}

#[test]
fn two_token_grammar_has_rule_and_both_tokens() {
    let nodes = generate_parsed(&two_token_grammar());
    assert!(find_by_type(&nodes, "s").is_some());
    assert!(find_by_type(&nodes, "a").is_some());
    assert!(find_by_type(&nodes, "b").is_some());
}

#[test]
fn regex_token_grammar_generates_ok() {
    assert!(
        NodeTypesGenerator::new(&regex_token_grammar())
            .generate()
            .is_ok()
    );
}

#[test]
fn alternative_grammar_generates_ok() {
    assert!(
        NodeTypesGenerator::new(&alternative_grammar())
            .generate()
            .is_ok()
    );
}

#[test]
fn alternative_grammar_single_rule_entry() {
    let nodes = generate_parsed(&alternative_grammar());
    let count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("s"))
        .count();
    assert_eq!(
        count, 1,
        "multiple productions should produce one node type"
    );
}

#[test]
fn chain_grammar_generates_ok() {
    assert!(NodeTypesGenerator::new(&chain_grammar()).generate().is_ok());
}

#[test]
fn chain_grammar_has_all_nonterminals_and_leaf() {
    let nodes = generate_parsed(&chain_grammar());
    for name in &["a", "b", "c"] {
        assert!(find_by_type(&nodes, name).is_some(), "missing '{name}'");
    }
    assert!(find_by_type(&nodes, "x").is_some());
}

// ===========================================================================
// 2. JSON structure validity (8 tests)
// ===========================================================================

#[test]
fn output_is_valid_json() {
    let json = generate_json(&simple_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn output_is_json_array() {
    let v: Value = serde_json::from_str(&generate_json(&simple_grammar())).unwrap();
    assert!(v.is_array());
}

#[test]
fn each_entry_has_type_field() {
    for n in &generate_parsed(&multi_nonterminal_grammar()) {
        assert!(n.get("type").is_some(), "missing 'type' in {n}");
    }
}

#[test]
fn each_entry_has_named_field() {
    for n in &generate_parsed(&multi_nonterminal_grammar()) {
        assert!(n.get("named").is_some(), "missing 'named' in {n}");
    }
}

#[test]
fn type_field_is_string() {
    for n in &generate_parsed(&arithmetic_grammar()) {
        assert!(n["type"].is_string(), "type should be string in {n}");
    }
}

#[test]
fn named_field_is_boolean() {
    for n in &generate_parsed(&arithmetic_grammar()) {
        assert!(n["named"].is_boolean(), "named should be bool in {n}");
    }
}

#[test]
fn output_is_pretty_printed() {
    let json = generate_json(&simple_grammar());
    assert!(json.contains('\n'), "output should be pretty-printed");
}

#[test]
fn output_sorted_by_type_name() {
    let nodes = generate_parsed(&many_tokens_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "entries should be sorted by type name");
}

// ===========================================================================
// 3. Named / anonymous node types (8 tests)
// ===========================================================================

#[test]
fn string_token_is_anonymous() {
    let nodes = generate_parsed(&simple_grammar());
    let a = find_by_type(&nodes, "a").expect("should contain token 'a'");
    assert_eq!(a["named"], false);
}

#[test]
fn regex_token_rule_is_named() {
    let nodes = generate_parsed(&regex_token_grammar());
    let expr = find_by_type(&nodes, "expr").expect("should contain rule 'expr'");
    assert_eq!(expr["named"], true);
}

#[test]
fn nonterminal_rules_are_named() {
    let nodes = generate_parsed(&chain_grammar());
    for name in &["a", "b", "c"] {
        let n = find_by_type(&nodes, name).unwrap();
        assert_eq!(n["named"], true, "'{name}' should be named");
    }
}

#[test]
fn punctuation_tokens_are_anonymous() {
    let nodes = generate_parsed(&recursive_grammar());
    for tok in &["(", ")"] {
        let n = find_by_type(&nodes, tok).unwrap();
        assert_eq!(n["named"], false, "'{tok}' should be anonymous");
    }
}

#[test]
fn operator_tokens_are_anonymous() {
    let nodes = generate_parsed(&arithmetic_grammar());
    for tok in &["+", "*"] {
        let n = find_by_type(&nodes, tok).unwrap();
        assert_eq!(n["named"], false, "'{tok}' should be anonymous");
    }
}

#[test]
fn named_entries_for_multi_nonterminal() {
    let nodes = generate_parsed(&multi_nonterminal_grammar());
    let named = named_entries(&nodes);
    for nt in &["program", "stmt", "assign", "val"] {
        assert!(
            named.contains(&nt.to_string()),
            "missing named entry '{nt}'"
        );
    }
}

#[test]
fn anonymous_entries_for_multi_nonterminal() {
    let nodes = generate_parsed(&multi_nonterminal_grammar());
    let anon = anonymous_entries(&nodes);
    for tok in &[";", "="] {
        assert!(
            anon.contains(&tok.to_string()),
            "missing anon entry '{tok}'"
        );
    }
}

#[test]
fn named_count_matches_nonterminal_count() {
    let nodes = generate_parsed(&simple_grammar());
    let named = named_entries(&nodes);
    assert_eq!(named.len(), 1);
    assert_eq!(named[0], "s");
}

// ===========================================================================
// 4. Node types with fields (5 tests)
// ===========================================================================

#[test]
fn field_appears_in_output() {
    let nodes = generate_parsed(&grammar_with_single_field());
    let value_node = find_by_type(&nodes, "value").expect("should contain 'value'");
    assert!(value_node.get("fields").is_some(), "should have fields key");
}

#[test]
fn field_has_required_and_multiple_flags() {
    let nodes = generate_parsed(&grammar_with_single_field());
    let fields = &find_by_type(&nodes, "value").unwrap()["fields"];
    let operand = &fields["operand"];
    assert_eq!(operand["required"], true);
    assert_eq!(operand["multiple"], false);
}

#[test]
fn field_has_types_array() {
    let nodes = generate_parsed(&grammar_with_single_field());
    let fields = &find_by_type(&nodes, "value").unwrap()["fields"];
    let types = fields["operand"]["types"]
        .as_array()
        .expect("types should be array");
    assert!(!types.is_empty());
}

#[test]
fn two_fields_both_present() {
    let nodes = generate_parsed(&grammar_with_two_fields());
    let fields = &find_by_type(&nodes, "add").unwrap()["fields"];
    assert!(fields.get("left").is_some(), "should have 'left' field");
    assert!(fields.get("right").is_some(), "should have 'right' field");
}

#[test]
fn nonterminal_field_type_is_named() {
    let nodes = generate_parsed(&grammar_with_nonterminal_field());
    let fields = &find_by_type(&nodes, "wrapper").unwrap()["fields"];
    let body_types = fields["body"]["types"].as_array().unwrap();
    assert_eq!(body_types.len(), 1);
    assert_eq!(body_types[0]["named"], true);
}

// ===========================================================================
// 5. Node types with children (5 tests)
// ===========================================================================

#[test]
fn simple_grammar_no_children_key() {
    let nodes = generate_parsed(&simple_grammar());
    for n in &nodes {
        assert!(
            n.get("children").is_none(),
            "children should be absent: {n}"
        );
    }
}

#[test]
fn chain_grammar_no_children_key() {
    let nodes = generate_parsed(&chain_grammar());
    for n in &nodes {
        assert!(
            n.get("children").is_none(),
            "children should be absent: {n}"
        );
    }
}

#[test]
fn recursive_grammar_no_children_key() {
    let nodes = generate_parsed(&recursive_grammar());
    for n in &nodes {
        assert!(
            n.get("children").is_none(),
            "children should be absent: {n}"
        );
    }
}

#[test]
fn multi_rule_grammar_no_children_key() {
    let nodes = generate_parsed(&many_rules_grammar());
    for n in &nodes {
        assert!(
            n.get("children").is_none(),
            "children should be absent: {n}"
        );
    }
}

#[test]
fn grammar_with_fields_still_no_children_key() {
    let nodes = generate_parsed(&grammar_with_two_fields());
    for n in &nodes {
        assert!(
            n.get("children").is_none(),
            "children should be absent: {n}"
        );
    }
}

// ===========================================================================
// 6. Determinism (5 tests)
// ===========================================================================

#[test]
fn same_grammar_same_output() {
    let g = arithmetic_grammar();
    assert_eq!(generate_json(&g), generate_json(&g));
}

#[test]
fn identical_grammars_same_output() {
    assert_eq!(
        generate_json(&simple_grammar()),
        generate_json(&simple_grammar())
    );
}

#[test]
fn deterministic_with_alternatives() {
    let g1 = alternative_grammar();
    let g2 = alternative_grammar();
    assert_eq!(generate_json(&g1), generate_json(&g2));
}

#[test]
fn deterministic_with_many_rules() {
    let g1 = many_rules_grammar();
    let g2 = many_rules_grammar();
    assert_eq!(generate_json(&g1), generate_json(&g2));
}

#[test]
fn deterministic_with_fields() {
    let g1 = grammar_with_two_fields();
    let g2 = grammar_with_two_fields();
    let v1: Value = serde_json::from_str(&generate_json(&g1)).unwrap();
    let v2: Value = serde_json::from_str(&generate_json(&g2)).unwrap();
    assert_eq!(v1, v2);
}

// ===========================================================================
// 7. Complex grammars (8 tests)
// ===========================================================================

#[test]
fn arithmetic_expr_is_named() {
    let nodes = generate_parsed(&arithmetic_grammar());
    assert_eq!(find_by_type(&nodes, "expr").unwrap()["named"], true);
}

#[test]
fn arithmetic_single_expr_entry() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn deep_chain_all_nonterminals_present_and_named() {
    let nodes = generate_parsed(&deep_chain_grammar());
    for name in &["n1", "n2", "n3", "n4", "n5"] {
        let n = find_by_type(&nodes, name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(n["named"], true);
    }
}

#[test]
fn recursive_grammar_rule_and_parens() {
    let nodes = generate_parsed(&recursive_grammar());
    assert_eq!(find_by_type(&nodes, "expr").unwrap()["named"], true);
    assert_eq!(find_by_type(&nodes, "(").unwrap()["named"], false);
    assert_eq!(find_by_type(&nodes, ")").unwrap()["named"], false);
}

#[test]
fn diamond_grammar_shape() {
    let g = GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .start("s")
        .build();
    let nodes = generate_parsed(&g);
    for name in &["s", "a", "b"] {
        assert_eq!(find_by_type(&nodes, name).unwrap()["named"], true);
    }
    assert_eq!(find_by_type(&nodes, "x").unwrap()["named"], false);
}

#[test]
fn ten_nonterminals_all_present() {
    let g = GrammarBuilder::new("ten")
        .token("t", "t")
        .rule("r0", vec!["r1"])
        .rule("r1", vec!["r2"])
        .rule("r2", vec!["r3"])
        .rule("r3", vec!["r4"])
        .rule("r4", vec!["r5"])
        .rule("r5", vec!["r6"])
        .rule("r6", vec!["r7"])
        .rule("r7", vec!["r8"])
        .rule("r8", vec!["r9"])
        .rule("r9", vec!["t"])
        .start("r0")
        .build();
    let nodes = generate_parsed(&g);
    for i in 0..10 {
        let name = format!("r{i}");
        let n = find_by_type(&nodes, &name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(n["named"], true);
    }
}

#[test]
fn right_recursive_list() {
    let g = GrammarBuilder::new("rlist")
        .token("ITEM", r"[a-z]+")
        .rule("list", vec!["ITEM", "list"])
        .rule("list", vec!["ITEM"])
        .start("list")
        .build();
    let nodes = generate_parsed(&g);
    assert_eq!(find_by_type(&nodes, "list").unwrap()["named"], true);
}

#[test]
fn left_recursive_list() {
    let g = GrammarBuilder::new("llist")
        .token("ITEM", r"[a-z]+")
        .rule("list", vec!["list", "ITEM"])
        .rule("list", vec!["ITEM"])
        .start("list")
        .build();
    let nodes = generate_parsed(&g);
    assert_eq!(find_by_type(&nodes, "list").unwrap()["named"], true);
}

// ===========================================================================
// 8. Edge cases (6 tests)
// ===========================================================================

#[test]
fn grammar_with_extra_token() {
    let g = GrammarBuilder::new("ws")
        .token("WS", r"\s+")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .extra("WS")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn grammar_with_fragile_token() {
    let g = GrammarBuilder::new("frag")
        .token("a", "a")
        .fragile_token("ERR", r".")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn grammar_with_external_token() {
    let g = GrammarBuilder::new("ext")
        .token("a", "a")
        .external("INDENT")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn no_duplicate_type_names() {
    let nodes = generate_parsed(&many_rules_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len(), "duplicate entries found");
}

#[test]
fn mixed_regex_and_string_tokens() {
    let g = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token(";", ";")
        .rule("s", vec!["NUM", "+", "NUM", ";"])
        .start("s")
        .build();
    let nodes = generate_parsed(&g);
    assert_eq!(find_by_type(&nodes, "s").unwrap()["named"], true);
    assert_eq!(find_by_type(&nodes, "+").unwrap()["named"], false);
    assert_eq!(find_by_type(&nodes, ";").unwrap()["named"], false);
}

#[test]
fn empty_grammar_output_is_empty_array() {
    let g = Grammar::new("empty".to_string());
    let nodes = generate_parsed(&g);
    assert!(nodes.is_empty(), "empty grammar should produce empty array");
}
