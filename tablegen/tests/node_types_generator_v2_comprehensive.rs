//! Comprehensive tests for `NodeTypesGenerator` (v2).
//!
//! Covers: constructor, JSON validity, output contents, determinism,
//! grammar sizes, nonterminals, JSON structure, error handling, large grammars.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_and_generate(mut grammar: Grammar) -> String {
    grammar.normalize();
    let generator = NodeTypesGenerator::new(&grammar);
    generator.generate().expect("generate must succeed")
}

fn build_and_parse(grammar: Grammar) -> Vec<Value> {
    let json = build_and_generate(grammar);
    serde_json::from_str::<Vec<Value>>(&json).expect("must be valid JSON array")
}

fn find_node<'a>(nodes: &'a [Value], type_name: &str) -> Option<&'a Value> {
    nodes
        .iter()
        .find(|n| n.get("type").and_then(|t| t.as_str()) == Some(type_name))
}

fn single_rule_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("IDENT", r"[a-z]+")
        .rule("root", vec!["IDENT"])
        .start("root")
        .build()
}

fn binary_op_grammar() -> Grammar {
    GrammarBuilder::new("binop")
        .token("NUM", r"[0-9]+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build()
}

fn multi_nonterminal_grammar() -> Grammar {
    GrammarBuilder::new("multi_nt")
        .token("ID", r"[a-z]+")
        .token("NUM", r"[0-9]+")
        .token("=", "=")
        .token(";", ";")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["assign"])
        .rule("assign", vec!["ID", "=", "val"])
        .rule("val", vec!["NUM"])
        .start("program")
        .build()
}

fn keyword_heavy_grammar() -> Grammar {
    GrammarBuilder::new("keywords")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("WHILE", "while")
        .token("FOR", "for")
        .token("RETURN", "return")
        .token("IDENT", r"[a-z]+")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["IF", "(", "IDENT", ")", "{", "}"])
        .start("program")
        .build()
}

// ---------------------------------------------------------------------------
// 1. Constructor tests
// ---------------------------------------------------------------------------

#[test]
fn v2_constructor_with_empty_grammar() {
    let grammar = Grammar::new("empty".to_string());
    let _gen = NodeTypesGenerator::new(&grammar);
}

#[test]
fn v2_constructor_with_single_rule_grammar() {
    let mut g = single_rule_grammar();
    g.normalize();
    let _gen = NodeTypesGenerator::new(&g);
}

#[test]
fn v2_constructor_accepts_normalized_grammar() {
    let mut g = binary_op_grammar();
    g.normalize();
    let ntg = NodeTypesGenerator::new(&g);
    assert!(ntg.generate().is_ok());
}

#[test]
fn v2_constructor_accepts_unnormalized_grammar() {
    let g = single_rule_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    // Should still produce output even without normalization
    assert!(ntg.generate().is_ok());
}

#[test]
fn v2_constructor_with_preset_python() {
    let mut g = GrammarBuilder::python_like();
    g.normalize();
    let ntg = NodeTypesGenerator::new(&g);
    assert!(ntg.generate().is_ok());
}

#[test]
fn v2_constructor_with_preset_javascript() {
    let mut g = GrammarBuilder::javascript_like();
    g.normalize();
    let ntg = NodeTypesGenerator::new(&g);
    assert!(ntg.generate().is_ok());
}

// ---------------------------------------------------------------------------
// 2. Generate produces valid JSON
// ---------------------------------------------------------------------------

#[test]
fn v2_single_rule_generates_valid_json() {
    let json = build_and_generate(single_rule_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn v2_binary_op_generates_valid_json() {
    let json = build_and_generate(binary_op_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn v2_multi_nonterminal_generates_valid_json() {
    let json = build_and_generate(multi_nonterminal_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn v2_keyword_grammar_generates_valid_json() {
    let json = build_and_generate(keyword_heavy_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn v2_empty_grammar_generates_valid_json() {
    let mut g = Grammar::new("empty".to_string());
    g.normalize();
    let ntg = NodeTypesGenerator::new(&g);
    let result = ntg.generate();
    assert!(result.is_ok());
    let json: Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(json.is_array());
}

// ---------------------------------------------------------------------------
// 3. Output contains node type names
// ---------------------------------------------------------------------------

#[test]
fn v2_output_contains_root_rule_name() {
    let nodes = build_and_parse(single_rule_grammar());
    assert!(find_node(&nodes, "root").is_some());
}

#[test]
fn v2_output_contains_expr_rule_name() {
    let nodes = build_and_parse(binary_op_grammar());
    assert!(find_node(&nodes, "expr").is_some());
}

#[test]
fn v2_output_contains_all_nonterminal_names() {
    let nodes = build_and_parse(multi_nonterminal_grammar());
    for name in &["program", "stmt", "assign", "val"] {
        assert!(
            find_node(&nodes, name).is_some(),
            "missing nonterminal: {name}"
        );
    }
}

#[test]
fn v2_output_contains_anonymous_operator() {
    let nodes = build_and_parse(binary_op_grammar());
    assert!(find_node(&nodes, "+").is_some());
}

#[test]
fn v2_output_contains_anonymous_keywords() {
    let nodes = build_and_parse(keyword_heavy_grammar());
    assert!(find_node(&nodes, "if").is_some());
}

#[test]
fn v2_output_contains_anonymous_punctuation() {
    let nodes = build_and_parse(keyword_heavy_grammar());
    for sym in &["(", ")", "{", "}"] {
        assert!(
            find_node(&nodes, sym).is_some(),
            "missing punctuation: {sym}"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Output is deterministic
// ---------------------------------------------------------------------------

#[test]
fn v2_determinism_single_rule() {
    let a = build_and_generate(single_rule_grammar());
    let b = build_and_generate(single_rule_grammar());
    assert_eq!(a, b);
}

#[test]
fn v2_determinism_binary_op() {
    let a = build_and_generate(binary_op_grammar());
    let b = build_and_generate(binary_op_grammar());
    assert_eq!(a, b);
}

#[test]
fn v2_determinism_multi_nonterminal() {
    let a = build_and_generate(multi_nonterminal_grammar());
    let b = build_and_generate(multi_nonterminal_grammar());
    assert_eq!(a, b);
}

#[test]
fn v2_determinism_ten_iterations() {
    let baseline = build_and_generate(keyword_heavy_grammar());
    for _ in 0..10 {
        assert_eq!(build_and_generate(keyword_heavy_grammar()), baseline);
    }
}

#[test]
fn v2_determinism_separate_generators_same_grammar() {
    let mut g = binary_op_grammar();
    g.normalize();
    let a = NodeTypesGenerator::new(&g).generate().unwrap();
    let b = NodeTypesGenerator::new(&g).generate().unwrap();
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// 5. Various grammar sizes
// ---------------------------------------------------------------------------

#[test]
fn v2_zero_rule_grammar_returns_array() {
    let nodes = build_and_parse(Grammar::new("zero".to_string()));
    assert!(nodes.is_empty() || nodes.iter().all(|n| n.is_object()));
}

#[test]
fn v2_one_token_one_rule() {
    let nodes = build_and_parse(single_rule_grammar());
    assert!(!nodes.is_empty());
}

#[test]
fn v2_five_tokens_one_rule() {
    let g = GrammarBuilder::new("five_tok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("root", vec!["A", "B", "C", "D", "E"])
        .start("root")
        .build();
    let nodes = build_and_parse(g);
    // At least the root rule + anonymous tokens
    assert!(nodes.len() >= 2);
}

#[test]
fn v2_ten_rules() {
    let mut b = GrammarBuilder::new("ten_rules");
    b = b.token("X", r"[a-z]");
    for i in 0..10 {
        let name: String = format!("r{i}");
        // Leak the string to get a &'static str for the rule name
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        b = b.rule(name_ref, vec!["X"]);
    }
    b = b.start("r0");
    let nodes = build_and_parse(b.build());
    let named_count = nodes.iter().filter(|n| n["named"] == true).count();
    assert!(
        named_count >= 10,
        "expected >=10 named rules, got {named_count}"
    );
}

#[test]
fn v2_grammar_with_extras() {
    let g = GrammarBuilder::new("with_extras")
        .token("WS", r"\s+")
        .token("NUM", r"[0-9]+")
        .extra("WS")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let nodes = build_and_parse(g);
    assert!(find_node(&nodes, "expr").is_some());
}

// ---------------------------------------------------------------------------
// 6. Grammars with many symbols
// ---------------------------------------------------------------------------

#[test]
fn v2_twenty_tokens_all_anonymous() {
    let mut b = GrammarBuilder::new("many_anon");
    let mut token_names: Vec<&'static str> = Vec::new();
    for i in 0..20 {
        let name: String = format!("t{i}");
        let pattern: String = format!("tok{i}");
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        let pat_ref: &'static str = Box::leak(pattern.into_boxed_str());
        b = b.token(name_ref, pat_ref);
        token_names.push(name_ref);
    }
    b = b.rule("root", vec![token_names[0]]).start("root");
    let nodes = build_and_parse(b.build());
    let anon_count = nodes.iter().filter(|n| n["named"] == false).count();
    assert!(
        anon_count >= 20,
        "expected >=20 anonymous tokens, got {anon_count}"
    );
}

#[test]
fn v2_twenty_regex_tokens_are_named() {
    let mut b = GrammarBuilder::new("many_regex");
    let mut token_names: Vec<&'static str> = Vec::new();
    for i in 0..20 {
        let name: String = format!("T{i}");
        let pattern: String = format!("[a-z]{{{i}}}");
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        let pat_ref: &'static str = Box::leak(pattern.into_boxed_str());
        b = b.token(name_ref, pat_ref);
        token_names.push(name_ref);
    }
    b = b.rule("root", vec![token_names[0]]).start("root");
    let nodes = build_and_parse(b.build());
    // Regex tokens are named — but only appear in node_types if they contribute as rules
    // The root rule should at least be present
    assert!(find_node(&nodes, "root").is_some());
}

#[test]
fn v2_mixed_string_and_regex_tokens() {
    let g = GrammarBuilder::new("mixed")
        .token("IDENT", r"[a-z]+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("NUM", r"[0-9]+")
        .rule("expr", vec!["IDENT"])
        .start("expr")
        .build();
    let nodes = build_and_parse(g);
    // String tokens are anonymous
    assert!(
        find_node(&nodes, "+")
            .map(|n| n["named"] == false)
            .unwrap_or(false)
    );
    // Rule is named
    assert!(
        find_node(&nodes, "expr")
            .map(|n| n["named"] == true)
            .unwrap_or(false)
    );
}

#[test]
fn v2_grammar_with_external_token() {
    let g = GrammarBuilder::new("ext")
        .token("NUM", r"[0-9]+")
        .external("INDENT")
        .rule("block", vec!["NUM"])
        .start("block")
        .build();
    let json = build_and_generate(g);
    let parsed: Vec<Value> = serde_json::from_str(&json).unwrap();
    assert!(find_node(&parsed, "block").is_some());
}

#[test]
fn v2_grammar_with_fragile_token() {
    let g = GrammarBuilder::new("fragile")
        .token("NUM", r"[0-9]+")
        .fragile_token("ERR", r".")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let nodes = build_and_parse(g);
    assert!(find_node(&nodes, "expr").is_some());
}

// ---------------------------------------------------------------------------
// 7. Grammars with nonterminals
// ---------------------------------------------------------------------------

#[test]
fn v2_chain_of_nonterminals() {
    let g = GrammarBuilder::new("chain")
        .token("X", r"x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["X"])
        .start("a")
        .build();
    let nodes = build_and_parse(g);
    for name in &["a", "b", "c"] {
        assert!(find_node(&nodes, name).is_some(), "missing: {name}");
    }
}

#[test]
fn v2_nonterminal_used_in_multiple_rules() {
    let g = GrammarBuilder::new("shared_nt")
        .token("NUM", r"[0-9]+")
        .token("+", "+")
        .token("*", "*")
        .rule("sum", vec!["atom", "+", "atom"])
        .rule("prod", vec!["atom", "*", "atom"])
        .rule("atom", vec!["NUM"])
        .start("sum")
        .build();
    let nodes = build_and_parse(g);
    assert!(find_node(&nodes, "atom").is_some());
    assert!(find_node(&nodes, "sum").is_some());
    assert!(find_node(&nodes, "prod").is_some());
}

#[test]
fn v2_nonterminals_are_all_named_true() {
    let nodes = build_and_parse(multi_nonterminal_grammar());
    for name in &["program", "stmt", "assign", "val"] {
        let node = find_node(&nodes, name).unwrap();
        assert_eq!(node["named"], true, "{name} should be named");
    }
}

#[test]
fn v2_deep_nonterminal_chain() {
    let mut b = GrammarBuilder::new("deep_chain");
    b = b.token("LEAF", r"leaf");
    let depth = 15;
    let mut names: Vec<&'static str> = Vec::new();
    for i in 0..depth {
        let name: String = format!("level{i}");
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        names.push(name_ref);
    }
    for i in 0..(depth - 1) {
        b = b.rule(names[i], vec![names[i + 1]]);
    }
    b = b.rule(names[depth - 1], vec!["LEAF"]);
    b = b.start(names[0]);
    let nodes = build_and_parse(b.build());
    for name in &names {
        assert!(find_node(&nodes, name).is_some(), "missing level: {name}");
    }
}

#[test]
fn v2_nonterminal_with_multiple_tokens_in_rhs() {
    let g = GrammarBuilder::new("multi_tok_rhs")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("seq", vec!["A", "B", "C"])
        .start("seq")
        .build();
    let nodes = build_and_parse(g);
    assert!(find_node(&nodes, "seq").is_some());
    // Anonymous string tokens
    for tok in &["a", "b", "c"] {
        assert!(find_node(&nodes, tok).is_some(), "missing token: {tok}");
    }
}

// ---------------------------------------------------------------------------
// 8. JSON structure validation
// ---------------------------------------------------------------------------

#[test]
fn v2_json_top_level_is_array() {
    let json = build_and_generate(single_rule_grammar());
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn v2_every_element_is_object() {
    let nodes = build_and_parse(multi_nonterminal_grammar());
    for (i, node) in nodes.iter().enumerate() {
        assert!(node.is_object(), "element {i} is not an object");
    }
}

#[test]
fn v2_every_element_has_type_field() {
    let nodes = build_and_parse(keyword_heavy_grammar());
    for node in &nodes {
        assert!(node.get("type").is_some(), "missing type field");
    }
}

#[test]
fn v2_every_element_has_named_field() {
    let nodes = build_and_parse(keyword_heavy_grammar());
    for node in &nodes {
        assert!(node.get("named").is_some(), "missing named field");
    }
}

#[test]
fn v2_type_field_is_always_string() {
    let nodes = build_and_parse(multi_nonterminal_grammar());
    for node in &nodes {
        assert!(node["type"].is_string(), "type field is not a string");
    }
}

#[test]
fn v2_named_field_is_always_boolean() {
    let nodes = build_and_parse(multi_nonterminal_grammar());
    for node in &nodes {
        assert!(node["named"].is_boolean(), "named field is not boolean");
    }
}

#[test]
fn v2_type_field_is_nonempty_string() {
    let nodes = build_and_parse(binary_op_grammar());
    for node in &nodes {
        let t = node["type"].as_str().unwrap();
        assert!(!t.is_empty(), "type field is empty");
    }
}

#[test]
fn v2_output_sorted_alphabetically_by_type() {
    let nodes = build_and_parse(keyword_heavy_grammar());
    let types: Vec<&str> = nodes.iter().map(|n| n["type"].as_str().unwrap()).collect();
    let mut sorted = types.clone();
    sorted.sort();
    assert_eq!(types, sorted, "output not sorted by type");
}

#[test]
fn v2_no_duplicate_type_entries() {
    let nodes = build_and_parse(multi_nonterminal_grammar());
    let types: Vec<&str> = nodes.iter().map(|n| n["type"].as_str().unwrap()).collect();
    let unique: HashSet<&str> = types.iter().copied().collect();
    assert_eq!(types.len(), unique.len(), "duplicate type entries found");
}

#[test]
fn v2_output_is_pretty_printed() {
    let json = build_and_generate(single_rule_grammar());
    assert!(
        json.contains('\n'),
        "output should be pretty-printed with newlines"
    );
}

#[test]
fn v2_output_starts_with_open_bracket() {
    let json = build_and_generate(single_rule_grammar());
    assert!(json.starts_with('['), "output must start with [");
}

#[test]
fn v2_output_ends_with_close_bracket() {
    let json = build_and_generate(single_rule_grammar());
    assert!(json.trim_end().ends_with(']'), "output must end with ]");
}

// ---------------------------------------------------------------------------
// 9. Error handling
// ---------------------------------------------------------------------------

#[test]
fn v2_generate_returns_ok_for_valid_grammar() {
    let mut g = single_rule_grammar();
    g.normalize();
    let ntg = NodeTypesGenerator::new(&g);
    assert!(ntg.generate().is_ok());
}

#[test]
fn v2_generate_ok_for_empty_grammar() {
    let mut g = Grammar::new("nothing".to_string());
    g.normalize();
    let ntg = NodeTypesGenerator::new(&g);
    assert!(ntg.generate().is_ok());
}

#[test]
fn v2_generate_result_unwraps_to_string() {
    let mut g = binary_op_grammar();
    g.normalize();
    let ntg = NodeTypesGenerator::new(&g);
    let output: String = ntg.generate().unwrap();
    assert!(!output.is_empty());
}

// ---------------------------------------------------------------------------
// 10. Large grammars
// ---------------------------------------------------------------------------

#[test]
fn v2_fifty_rules_generates_ok() {
    let mut b = GrammarBuilder::new("large50");
    b = b.token("TOK", r"[a-z]+");
    for i in 0..50 {
        let name: String = format!("rule{i}");
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        b = b.rule(name_ref, vec!["TOK"]);
    }
    b = b.start("rule0");
    let json = build_and_generate(b.build());
    let parsed: Vec<Value> = serde_json::from_str(&json).unwrap();
    let named_count = parsed.iter().filter(|n| n["named"] == true).count();
    assert!(
        named_count >= 50,
        "expected >=50 named rules, got {named_count}"
    );
}

#[test]
fn v2_hundred_rules_generates_ok() {
    let mut b = GrammarBuilder::new("large100");
    b = b.token("TOK", r"[a-z]+");
    for i in 0..100 {
        let name: String = format!("sym{i}");
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        b = b.rule(name_ref, vec!["TOK"]);
    }
    b = b.start("sym0");
    let json = build_and_generate(b.build());
    let parsed: Vec<Value> = serde_json::from_str(&json).unwrap();
    let named_count = parsed.iter().filter(|n| n["named"] == true).count();
    assert!(
        named_count >= 100,
        "expected >=100 named rules, got {named_count}"
    );
}

#[test]
fn v2_large_grammar_output_is_sorted() {
    let mut b = GrammarBuilder::new("sort_large");
    b = b.token("TOK", r"[a-z]+");
    for i in 0..30 {
        let name: String = format!("node{i:03}");
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        b = b.rule(name_ref, vec!["TOK"]);
    }
    b = b.start("node000");
    let nodes = build_and_parse(b.build());
    let types: Vec<&str> = nodes.iter().map(|n| n["type"].as_str().unwrap()).collect();
    let mut sorted = types.clone();
    sorted.sort();
    assert_eq!(types, sorted);
}

#[test]
fn v2_large_grammar_deterministic() {
    let make = || {
        let mut b = GrammarBuilder::new("det_large");
        b = b.token("TOK", r"[a-z]+");
        for i in 0..40 {
            let name: String = format!("n{i}");
            let name_ref: &'static str = Box::leak(name.into_boxed_str());
            b = b.rule(name_ref, vec!["TOK"]);
        }
        b = b.start("n0");
        build_and_generate(b.build())
    };
    assert_eq!(make(), make());
}

#[test]
fn v2_large_grammar_all_elements_have_required_fields() {
    let mut b = GrammarBuilder::new("fields_large");
    b = b.token("TOK", r"[a-z]+");
    for i in 0..25 {
        let name: String = format!("item{i}");
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        b = b.rule(name_ref, vec!["TOK"]);
    }
    b = b.start("item0");
    let nodes = build_and_parse(b.build());
    for node in &nodes {
        assert!(node.get("type").is_some());
        assert!(node.get("named").is_some());
    }
}

// ---------------------------------------------------------------------------
// Additional coverage: precedence, associativity, named vs anonymous
// ---------------------------------------------------------------------------

#[test]
fn v2_precedence_grammar_generates_ok() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"[0-9]+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let nodes = build_and_parse(g);
    assert!(find_node(&nodes, "expr").is_some());
}

#[test]
fn v2_right_associative_grammar() {
    let g = GrammarBuilder::new("right_assoc")
        .token("NUM", r"[0-9]+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let nodes = build_and_parse(g);
    assert!(find_node(&nodes, "expr").is_some());
    assert!(find_node(&nodes, "^").is_some());
}

#[test]
fn v2_string_literal_token_is_anonymous() {
    let g = GrammarBuilder::new("anon_lit")
        .token("+", "+")
        .token("NUM", r"[0-9]+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build();
    let nodes = build_and_parse(g);
    let plus = find_node(&nodes, "+").expect("should have + node");
    assert_eq!(
        plus["named"], false,
        "string literal token should be anonymous"
    );
}

#[test]
fn v2_regex_token_in_rule_is_named() {
    let nodes = build_and_parse(single_rule_grammar());
    let root = find_node(&nodes, "root").expect("root must exist");
    assert_eq!(root["named"], true);
}

#[test]
fn v2_internal_rules_excluded_from_output() {
    // Internal rules (starting with _) should not appear
    let mut g = Grammar::new("internal_test".to_string());

    let tok_id = SymbolId(1);
    g.tokens.insert(
        tok_id,
        Token {
            name: "ID".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(100);
    g.rule_names.insert(rule_id, "_hidden".to_string());
    g.rules.insert(
        rule_id,
        vec![Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: Default::default(),
            production_id: ProductionId(0),
        }],
    );

    let visible_id = SymbolId(101);
    g.rule_names.insert(visible_id, "visible".to_string());
    g.rules.insert(
        visible_id,
        vec![Rule {
            lhs: visible_id,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: Default::default(),
            production_id: ProductionId(1),
        }],
    );

    g.normalize();
    let nodes = build_and_parse(g);
    assert!(
        find_node(&nodes, "_hidden").is_none(),
        "internal rules should be excluded"
    );
    assert!(
        find_node(&nodes, "visible").is_some(),
        "visible rules should be present"
    );
}

#[test]
fn v2_fields_key_absent_when_no_fields() {
    let nodes = build_and_parse(single_rule_grammar());
    let root = find_node(&nodes, "root").unwrap();
    // When fields are empty, the key should be absent (null in JSON)
    assert!(
        root.get("fields").is_none() || root["fields"].is_null(),
        "fields should be absent or null when empty"
    );
}

#[test]
fn v2_children_key_absent_when_not_applicable() {
    let nodes = build_and_parse(single_rule_grammar());
    let root = find_node(&nodes, "root").unwrap();
    assert!(
        root.get("children").is_none() || root["children"].is_null(),
        "children should be absent or null"
    );
}

#[test]
fn v2_subtypes_key_absent_when_not_applicable() {
    let nodes = build_and_parse(single_rule_grammar());
    let root = find_node(&nodes, "root").unwrap();
    assert!(
        root.get("subtypes").is_none() || root["subtypes"].is_null(),
        "subtypes should be absent or null"
    );
}

#[test]
fn v2_different_grammars_produce_different_output() {
    let a = build_and_generate(single_rule_grammar());
    let b = build_and_generate(binary_op_grammar());
    assert_ne!(a, b);
}

#[test]
fn v2_grammar_name_does_not_affect_node_types() {
    let g1 = GrammarBuilder::new("name_a")
        .token("X", r"x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let g2 = GrammarBuilder::new("name_b")
        .token("X", r"x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    assert_eq!(build_and_generate(g1), build_and_generate(g2));
}

#[test]
fn v2_node_count_increases_with_rules() {
    let small = build_and_parse(single_rule_grammar());
    let large = build_and_parse(multi_nonterminal_grammar());
    assert!(
        large.len() > small.len(),
        "more rules should produce more node types"
    );
}

#[test]
fn v2_python_like_contains_expected_rules() {
    let nodes = build_and_parse(GrammarBuilder::python_like());
    // Python-like grammar should have module and function_def
    assert!(find_node(&nodes, "module").is_some());
    assert!(find_node(&nodes, "function_def").is_some());
}

#[test]
fn v2_javascript_like_contains_expected_rules() {
    let nodes = build_and_parse(GrammarBuilder::javascript_like());
    assert!(find_node(&nodes, "program").is_some());
    assert!(find_node(&nodes, "expression").is_some());
}

#[test]
fn v2_all_anonymous_nodes_have_named_false() {
    let nodes = build_and_parse(keyword_heavy_grammar());
    let anon_nodes: Vec<_> = nodes.iter().filter(|n| n["named"] == false).collect();
    assert!(!anon_nodes.is_empty(), "should have some anonymous nodes");
    for node in &anon_nodes {
        assert_eq!(node["named"], false);
    }
}

#[test]
fn v2_all_rule_nodes_have_named_true() {
    let nodes = build_and_parse(keyword_heavy_grammar());
    // Rules (program, stmt) should be named
    for name in &["program", "stmt"] {
        let node = find_node(&nodes, name).unwrap();
        assert_eq!(node["named"], true, "{name} should have named=true");
    }
}

#[test]
fn v2_no_null_type_names() {
    let nodes = build_and_parse(multi_nonterminal_grammar());
    for node in &nodes {
        assert!(!node["type"].is_null(), "type should not be null");
    }
}

#[test]
fn v2_generate_output_is_utf8_string() {
    let json = build_and_generate(binary_op_grammar());
    // If we got here, it's already a valid UTF-8 String
    assert!(std::str::from_utf8(json.as_bytes()).is_ok());
}
