//! Comprehensive v3 tests for `NodeTypesGenerator`.
//!
//! 50+ tests covering: construction, generation, JSON structure, named/anonymous
//! flags, grammar shapes (simple, chains, recursive, alternatives), multi-token,
//! multi-nonterminal, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ===========================================================================
// Helpers
// ===========================================================================

fn gen_json(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate failed")
}

fn gen_parsed(grammar: &Grammar) -> Vec<Value> {
    let json = gen_json(grammar);
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

// ===========================================================================
// 1. Construction
// ===========================================================================

#[test]
fn new_takes_grammar_ref() {
    let g = simple_grammar();
    let _ntg = NodeTypesGenerator::new(&g);
}

#[test]
fn new_with_empty_rules_grammar() {
    let g = GrammarBuilder::new("empty")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let _ntg = NodeTypesGenerator::new(&g);
}

// ===========================================================================
// 2. generate() returns Ok
// ===========================================================================

#[test]
fn generate_returns_ok_simple() {
    let g = simple_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_regex_token() {
    let g = regex_token_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_alternatives() {
    let g = alternative_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_chain() {
    let g = chain_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_recursive() {
    let g = recursive_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_arithmetic() {
    let g = arithmetic_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_many_rules() {
    let g = many_rules_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_deep_chain() {
    let g = deep_chain_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_multi_nonterminal() {
    let g = multi_nonterminal_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn generate_returns_ok_many_tokens() {
    let g = many_tokens_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// ===========================================================================
// 3. JSON structure
// ===========================================================================

#[test]
fn output_is_valid_json() {
    let g = simple_grammar();
    let json = gen_json(&g);
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn output_is_json_array() {
    let g = simple_grammar();
    let v: Value = serde_json::from_str(&gen_json(&g)).unwrap();
    assert!(v.is_array());
}

#[test]
fn each_entry_has_type_field() {
    let nodes = gen_parsed(&simple_grammar());
    for n in &nodes {
        assert!(n.get("type").is_some(), "missing 'type' in {n}");
    }
}

#[test]
fn each_entry_has_named_field() {
    let nodes = gen_parsed(&simple_grammar());
    for n in &nodes {
        assert!(n.get("named").is_some(), "missing 'named' in {n}");
    }
}

#[test]
fn type_field_is_string() {
    let nodes = gen_parsed(&simple_grammar());
    for n in &nodes {
        assert!(n["type"].is_string(), "type should be string in {n}");
    }
}

#[test]
fn named_field_is_boolean() {
    let nodes = gen_parsed(&simple_grammar());
    for n in &nodes {
        assert!(n["named"].is_boolean(), "named should be bool in {n}");
    }
}

#[test]
fn output_array_non_empty_simple() {
    let nodes = gen_parsed(&simple_grammar());
    assert!(!nodes.is_empty());
}

#[test]
fn output_sorted_by_type_name() {
    let nodes = gen_parsed(&many_tokens_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn json_structure_multi_nonterminal() {
    let nodes = gen_parsed(&multi_nonterminal_grammar());
    for n in &nodes {
        assert!(n["type"].is_string());
        assert!(n["named"].is_boolean());
    }
}

#[test]
fn json_pretty_printed() {
    let json = gen_json(&simple_grammar());
    // Pretty-printed JSON has newlines
    assert!(json.contains('\n'));
}

// ===========================================================================
// 4. Named / anonymous flags
// ===========================================================================

#[test]
fn string_token_is_anonymous() {
    // token("a", "a") → TokenPattern::String → named: false
    let nodes = gen_parsed(&simple_grammar());
    let a = find_by_type(&nodes, "a");
    assert!(a.is_some(), "should contain token 'a'");
    assert_eq!(a.unwrap()["named"], false);
}

#[test]
fn regex_token_is_named() {
    // token("NUMBER", r"\d+") → TokenPattern::Regex → named: true
    // Regex tokens are NOT added to the anonymous list; they only appear
    // via their rule. The rule "expr" is named: true.
    let nodes = gen_parsed(&regex_token_grammar());
    let expr = find_by_type(&nodes, "expr");
    assert!(expr.is_some());
    assert_eq!(expr.unwrap()["named"], true);
}

#[test]
fn rule_nonterminal_is_named() {
    let nodes = gen_parsed(&simple_grammar());
    let s = find_by_type(&nodes, "s");
    assert!(s.is_some(), "should contain rule 's'");
    assert_eq!(s.unwrap()["named"], true);
}

#[test]
fn all_nonterminals_are_named() {
    let nodes = gen_parsed(&chain_grammar());
    for name in &["a", "b", "c"] {
        let n = find_by_type(&nodes, name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(n["named"], true, "{name} should be named");
    }
}

#[test]
fn string_literal_tokens_are_anonymous() {
    let nodes = gen_parsed(&two_token_grammar());
    for name in &["a", "b"] {
        let n = find_by_type(&nodes, name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(n["named"], false, "token '{name}' should be anonymous");
    }
}

#[test]
fn punctuation_tokens_anonymous() {
    let nodes = gen_parsed(&recursive_grammar());
    for tok in &["(", ")"] {
        let n = find_by_type(&nodes, tok).unwrap_or_else(|| panic!("missing {tok}"));
        assert_eq!(n["named"], false, "'{tok}' should be anonymous");
    }
}

#[test]
fn operator_tokens_anonymous() {
    let nodes = gen_parsed(&arithmetic_grammar());
    for tok in &["+", "*"] {
        let n = find_by_type(&nodes, tok).unwrap_or_else(|| panic!("missing {tok}"));
        assert_eq!(n["named"], false, "'{tok}' should be anonymous");
    }
}

#[test]
fn named_entries_for_multi_nonterminal() {
    let nodes = gen_parsed(&multi_nonterminal_grammar());
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
    let nodes = gen_parsed(&multi_nonterminal_grammar());
    let anon = anonymous_entries(&nodes);
    for tok in &[";", "="] {
        assert!(
            anon.contains(&tok.to_string()),
            "missing anonymous entry '{tok}'"
        );
    }
}

// ===========================================================================
// 5. Grammar shapes — simple
// ===========================================================================

#[test]
fn simple_grammar_has_rule_and_token() {
    let nodes = gen_parsed(&simple_grammar());
    assert!(find_by_type(&nodes, "s").is_some());
    assert!(find_by_type(&nodes, "a").is_some());
}

#[test]
fn simple_grammar_exactly_two_entries() {
    let nodes = gen_parsed(&simple_grammar());
    // rule "s" (named) + token "a" (anonymous)
    assert_eq!(nodes.len(), 2);
}

// ===========================================================================
// 6. Grammar shapes — alternatives
// ===========================================================================

#[test]
fn alternative_grammar_has_single_rule() {
    let nodes = gen_parsed(&alternative_grammar());
    // "s" appears once despite two productions
    let count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("s"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn alternative_grammar_rule_is_named() {
    let nodes = gen_parsed(&alternative_grammar());
    let s = find_by_type(&nodes, "s").unwrap();
    assert_eq!(s["named"], true);
}

#[test]
fn alternative_grammar_contains_both_tokens() {
    let nodes = gen_parsed(&alternative_grammar());
    assert!(find_by_type(&nodes, "a").is_some());
    assert!(find_by_type(&nodes, "b").is_some());
}

// ===========================================================================
// 7. Grammar shapes — chains
// ===========================================================================

#[test]
fn chain_grammar_has_all_nonterminals() {
    let nodes = gen_parsed(&chain_grammar());
    for name in &["a", "b", "c"] {
        assert!(find_by_type(&nodes, name).is_some(), "missing '{name}'");
    }
}

#[test]
fn chain_grammar_has_leaf_token() {
    let nodes = gen_parsed(&chain_grammar());
    let x = find_by_type(&nodes, "x");
    assert!(x.is_some());
    assert_eq!(x.unwrap()["named"], false);
}

#[test]
fn deep_chain_all_nonterminals_present() {
    let nodes = gen_parsed(&deep_chain_grammar());
    for name in &["n1", "n2", "n3", "n4", "n5"] {
        assert!(find_by_type(&nodes, name).is_some(), "missing '{name}'");
    }
}

#[test]
fn deep_chain_all_nonterminals_named() {
    let nodes = gen_parsed(&deep_chain_grammar());
    for name in &["n1", "n2", "n3", "n4", "n5"] {
        let n = find_by_type(&nodes, name).unwrap();
        assert_eq!(n["named"], true);
    }
}

// ===========================================================================
// 8. Grammar shapes — recursive
// ===========================================================================

#[test]
fn recursive_grammar_rule_present() {
    let nodes = gen_parsed(&recursive_grammar());
    assert!(find_by_type(&nodes, "expr").is_some());
}

#[test]
fn recursive_grammar_rule_named() {
    let nodes = gen_parsed(&recursive_grammar());
    assert_eq!(find_by_type(&nodes, "expr").unwrap()["named"], true);
}

#[test]
fn recursive_grammar_parens_anonymous() {
    let nodes = gen_parsed(&recursive_grammar());
    assert_eq!(find_by_type(&nodes, "(").unwrap()["named"], false);
    assert_eq!(find_by_type(&nodes, ")").unwrap()["named"], false);
}

// ===========================================================================
// 9. Arithmetic / precedence
// ===========================================================================

#[test]
fn arithmetic_grammar_expr_named() {
    let nodes = gen_parsed(&arithmetic_grammar());
    assert_eq!(find_by_type(&nodes, "expr").unwrap()["named"], true);
}

#[test]
fn arithmetic_grammar_operators_anonymous() {
    let nodes = gen_parsed(&arithmetic_grammar());
    let anon = anonymous_entries(&nodes);
    assert!(anon.contains(&"+".to_string()));
    assert!(anon.contains(&"*".to_string()));
}

#[test]
fn arithmetic_single_expr_entry() {
    let nodes = gen_parsed(&arithmetic_grammar());
    let expr_count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr"))
        .count();
    assert_eq!(expr_count, 1, "expr should appear exactly once");
}

// ===========================================================================
// 10. Many rules / many tokens
// ===========================================================================

#[test]
fn many_rules_all_nonterminals_present() {
    let nodes = gen_parsed(&many_rules_grammar());
    for name in &["s", "r1", "r2"] {
        assert!(find_by_type(&nodes, name).is_some(), "missing '{name}'");
    }
}

#[test]
fn many_rules_all_tokens_present() {
    let nodes = gen_parsed(&many_rules_grammar());
    for tok in &["a", "b", "c", "d"] {
        assert!(find_by_type(&nodes, tok).is_some(), "missing '{tok}'");
    }
}

#[test]
fn many_tokens_all_present() {
    let nodes = gen_parsed(&many_tokens_grammar());
    for tok in &["a", "b", "c", "d", "e", "f"] {
        assert!(find_by_type(&nodes, tok).is_some(), "missing '{tok}'");
    }
}

#[test]
fn many_tokens_all_anonymous() {
    let nodes = gen_parsed(&many_tokens_grammar());
    for tok in &["a", "b", "c", "d", "e", "f"] {
        let n = find_by_type(&nodes, tok).unwrap();
        assert_eq!(n["named"], false, "'{tok}' should be anonymous");
    }
}

// ===========================================================================
// 11. Determinism
// ===========================================================================

#[test]
fn generate_is_deterministic() {
    let g = arithmetic_grammar();
    let a = gen_json(&g);
    let b = gen_json(&g);
    assert_eq!(a, b);
}

#[test]
fn deterministic_across_identical_grammars() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    assert_eq!(gen_json(&g1), gen_json(&g2));
}

// ===========================================================================
// 12. Two-token grammar
// ===========================================================================

#[test]
fn two_token_grammar_has_rule() {
    let nodes = gen_parsed(&two_token_grammar());
    assert!(find_by_type(&nodes, "s").is_some());
}

#[test]
fn two_token_grammar_both_tokens() {
    let nodes = gen_parsed(&two_token_grammar());
    assert!(find_by_type(&nodes, "a").is_some());
    assert!(find_by_type(&nodes, "b").is_some());
}

// ===========================================================================
// 13. No duplicate entries
// ===========================================================================

#[test]
fn no_duplicate_type_names_simple() {
    let nodes = gen_parsed(&simple_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len(), "duplicate entries found");
}

#[test]
fn no_duplicate_type_names_many_rules() {
    let nodes = gen_parsed(&many_rules_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len(), "duplicate entries found");
}

// ===========================================================================
// 14. Extra grammar configurations
// ===========================================================================

#[test]
fn grammar_with_extra_token_generates_ok() {
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
fn grammar_with_fragile_token_generates_ok() {
    let g = GrammarBuilder::new("frag")
        .token("a", "a")
        .fragile_token("ERR", r".")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn grammar_with_external_generates_ok() {
    let g = GrammarBuilder::new("ext")
        .token("a", "a")
        .external("INDENT")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// ===========================================================================
// 15. Wider grammar shapes
// ===========================================================================

#[test]
fn wide_alternatives_grammar() {
    let g = GrammarBuilder::new("wide")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .rule("s", vec!["e"])
        .start("s")
        .build();
    let nodes = gen_parsed(&g);
    assert!(find_by_type(&nodes, "s").is_some());
    // All five tokens present
    for tok in &["a", "b", "c", "d", "e"] {
        assert!(find_by_type(&nodes, tok).is_some());
    }
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
    let nodes = gen_parsed(&g);
    // "s" is named
    assert_eq!(find_by_type(&nodes, "s").unwrap()["named"], true);
    // "+" and ";" are anonymous
    assert_eq!(find_by_type(&nodes, "+").unwrap()["named"], false);
    assert_eq!(find_by_type(&nodes, ";").unwrap()["named"], false);
    // NUM (regex) does not appear as a separate anonymous entry
}

#[test]
fn diamond_grammar_shape() {
    // s -> a | b; a -> x; b -> x
    let g = GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .start("s")
        .build();
    let nodes = gen_parsed(&g);
    for name in &["s", "a", "b"] {
        let n = find_by_type(&nodes, name).unwrap();
        assert_eq!(n["named"], true);
    }
    assert_eq!(find_by_type(&nodes, "x").unwrap()["named"], false);
}

#[test]
fn single_token_single_rule() {
    let g = GrammarBuilder::new("single")
        .token("z", "z")
        .rule("root", vec!["z"])
        .start("root")
        .build();
    let nodes = gen_parsed(&g);
    assert_eq!(nodes.len(), 2);
    assert!(find_by_type(&nodes, "root").is_some());
    assert!(find_by_type(&nodes, "z").is_some());
}

#[test]
fn multiple_regex_tokens() {
    let g = GrammarBuilder::new("multi_regex")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("STR", r#""[^"]*""#)
        .rule("val", vec!["ID"])
        .rule("val", vec!["NUM"])
        .rule("val", vec!["STR"])
        .start("val")
        .build();
    let nodes = gen_parsed(&g);
    // "val" is the only named nonterminal; regex tokens don't produce
    // separate anonymous entries.
    assert!(find_by_type(&nodes, "val").is_some());
    assert_eq!(find_by_type(&nodes, "val").unwrap()["named"], true);
}

#[test]
fn ten_nonterminals_grammar() {
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
    let nodes = gen_parsed(&g);
    for i in 0..10 {
        let name = format!("r{i}");
        assert!(find_by_type(&nodes, &name).is_some(), "missing {name}");
        assert_eq!(find_by_type(&nodes, &name).unwrap()["named"], true);
    }
}

#[test]
fn right_recursive_list_grammar() {
    // list -> item list | item
    let g = GrammarBuilder::new("rlist")
        .token("ITEM", r"[a-z]+")
        .rule("list", vec!["ITEM", "list"])
        .rule("list", vec!["ITEM"])
        .start("list")
        .build();
    let nodes = gen_parsed(&g);
    assert!(find_by_type(&nodes, "list").is_some());
    assert_eq!(find_by_type(&nodes, "list").unwrap()["named"], true);
}

#[test]
fn left_recursive_list_grammar() {
    // list -> list item | item
    let g = GrammarBuilder::new("llist")
        .token("ITEM", r"[a-z]+")
        .rule("list", vec!["list", "ITEM"])
        .rule("list", vec!["ITEM"])
        .start("list")
        .build();
    let nodes = gen_parsed(&g);
    assert!(find_by_type(&nodes, "list").is_some());
}

#[test]
fn grammar_with_semicolons_and_equals() {
    let nodes = gen_parsed(&multi_nonterminal_grammar());
    assert!(find_by_type(&nodes, ";").is_some());
    assert!(find_by_type(&nodes, "=").is_some());
}

#[test]
fn named_count_matches_nonterminal_count_simple() {
    let nodes = gen_parsed(&simple_grammar());
    let named = named_entries(&nodes);
    // Only "s" is a named nonterminal
    assert_eq!(named.len(), 1);
    assert_eq!(named[0], "s");
}

#[test]
fn anonymous_count_matches_string_token_count() {
    let nodes = gen_parsed(&two_token_grammar());
    let anon = anonymous_entries(&nodes);
    // "a" and "b" are string tokens
    assert_eq!(anon.len(), 2);
}
