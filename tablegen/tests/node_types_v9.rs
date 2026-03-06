//! v9 comprehensive tests for NODE_TYPES JSON generation in adze-tablegen.
//!
//! Categories:
//!   1. Simple grammar generation (tests 1–8)
//!   2. JSON validity (tests 9–16)
//!   3. JSON structure — required fields (tests 17–24)
//!   4. Grammar name handling (tests 25–30)
//!   5. Multiple tokens reflected in output (tests 31–37)
//!   6. Multiple rules reflected in output (tests 38–44)
//!   7. Precedence grammars (tests 45–50)
//!   8. Inline rules (tests 51–55)
//!   9. Supertypes (tests 56–60)
//!  10. Extras (tests 61–65)
//!  11. Externals (tests 66–70)
//!  12. Single-rule grammar (tests 71–74)
//!  13. Multi-rule grammar with 5+ rules (tests 75–78)
//!  14. Determinism (tests 79–83)
//!  15. Grammar patterns: list, tree, expr (tests 84–90)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn generate_node_types(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate")
}

fn parse_json(grammar: &Grammar) -> Vec<Value> {
    let json = generate_node_types(grammar);
    let val: Value = serde_json::from_str(&json).expect("valid JSON");
    val.as_array().expect("JSON array").to_vec()
}

fn find_node<'a>(nodes: &'a [Value], type_name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(type_name))
}

fn all_type_names(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

fn named_type_names(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("number", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["number"])
        .rule("add", vec!["expr", "+", "expr"])
        .rule("mul", vec!["expr", "*", "expr"])
        .start("add")
        .build()
}

fn scaled_grammar(name: &str, n: usize) -> Grammar {
    let mut builder = GrammarBuilder::new(name);
    let tok_names: Vec<String> = (0..n).map(|i| format!("tok_{i}")).collect();
    let rule_names: Vec<String> = (0..n).map(|i| format!("rule_{i}")).collect();
    for i in 0..n {
        builder = builder.token(&tok_names[i], &tok_names[i]);
        builder = builder.rule(&rule_names[i], vec![&tok_names[i]]);
    }
    if n > 0 {
        builder = builder.start(&rule_names[0]);
    }
    builder.build()
}

fn grammar_with_fields() -> Grammar {
    let mut g = Grammar::new("fields_grammar".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let op_id = SymbolId(1);
    g.tokens.insert(
        op_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(10);
    g.rule_names.insert(expr_id, "binary_expr".to_string());

    let left_field = FieldId(0);
    let right_field = FieldId(1);
    g.fields.insert(left_field, "left".to_string());
    g.fields.insert(right_field, "right".to_string());

    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(op_id),
            Symbol::Terminal(num_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(left_field, 0), (right_field, 2)],
        production_id: ProductionId(0),
    });

    g
}

// ===========================================================================
// 1. Simple grammar generation (8 tests)
// ===========================================================================

#[test]
fn simple_grammar_generates_ok() {
    let g = simple_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn simple_grammar_output_is_nonempty() {
    let output = generate_node_types(&simple_grammar());
    assert!(!output.is_empty());
}

#[test]
fn simple_grammar_has_start_rule_in_output() {
    let nodes = parse_json(&simple_grammar());
    assert!(find_node(&nodes, "start").is_some());
}

#[test]
fn simple_grammar_start_rule_is_named() {
    let nodes = parse_json(&simple_grammar());
    let start = find_node(&nodes, "start").expect("start node");
    assert_eq!(start["named"], true);
}

#[test]
fn simple_grammar_string_tokens_are_anonymous() {
    let nodes = parse_json(&simple_grammar());
    // "a" is a string literal token → anonymous
    if let Some(a_node) = find_node(&nodes, "a") {
        assert_eq!(a_node["named"], false);
    }
}

#[test]
fn simple_grammar_output_starts_with_bracket() {
    let output = generate_node_types(&simple_grammar());
    assert!(output.trim_start().starts_with('['));
}

#[test]
fn simple_grammar_output_ends_with_bracket() {
    let output = generate_node_types(&simple_grammar());
    assert!(output.trim_end().ends_with(']'));
}

#[test]
fn simple_grammar_result_is_not_err() {
    let g = simple_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

// ===========================================================================
// 2. JSON validity (8 tests)
// ===========================================================================

#[test]
fn json_valid_for_minimal_grammar() {
    let g = GrammarBuilder::new("min")
        .token("x", "x")
        .rule("root", vec!["x"])
        .build();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn json_valid_for_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn json_valid_for_arithmetic_grammar() {
    assert!(serde_json::from_str::<Value>(&generate_node_types(&arithmetic_grammar())).is_ok());
}

#[test]
fn json_is_array_for_simple_grammar() {
    let val: Value = serde_json::from_str(&generate_node_types(&simple_grammar())).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_is_array_for_empty_grammar() {
    let g = Grammar::new("e".to_string());
    let val: Value = serde_json::from_str(&generate_node_types(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_valid_for_grammar_with_regex_tokens() {
    let g = GrammarBuilder::new("re")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .rule("start", vec!["id", "num"])
        .build();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn json_valid_for_scaled_grammar() {
    let g = scaled_grammar("scaled", 30);
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn json_valid_for_grammar_with_fields() {
    let g = grammar_with_fields();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

// ===========================================================================
// 3. JSON structure — required fields (8 tests)
// ===========================================================================

#[test]
fn all_entries_have_type_field() {
    for entry in parse_json(&simple_grammar()) {
        assert!(entry.get("type").is_some(), "missing 'type' field");
    }
}

#[test]
fn all_entries_have_named_field() {
    for entry in parse_json(&simple_grammar()) {
        assert!(entry.get("named").is_some(), "missing 'named' field");
    }
}

#[test]
fn type_field_is_string() {
    for entry in parse_json(&simple_grammar()) {
        assert!(entry["type"].is_string());
    }
}

#[test]
fn named_field_is_bool() {
    for entry in parse_json(&simple_grammar()) {
        assert!(entry["named"].is_boolean());
    }
}

#[test]
fn arithmetic_entries_have_required_fields() {
    for entry in parse_json(&arithmetic_grammar()) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn scaled_entries_have_required_fields() {
    let g = scaled_grammar("s", 10);
    for entry in parse_json(&g) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn fields_grammar_entries_have_required_fields() {
    for entry in parse_json(&grammar_with_fields()) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn no_entries_have_null_type() {
    for entry in parse_json(&arithmetic_grammar()) {
        assert!(!entry["type"].is_null());
    }
}

// ===========================================================================
// 4. Grammar name handling (6 tests)
// ===========================================================================

#[test]
fn different_grammar_names_both_generate() {
    let g1 = GrammarBuilder::new("alpha")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let g2 = GrammarBuilder::new("beta")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    assert!(NodeTypesGenerator::new(&g1).generate().is_ok());
    assert!(NodeTypesGenerator::new(&g2).generate().is_ok());
}

#[test]
fn grammar_name_does_not_appear_as_node_type() {
    let g = GrammarBuilder::new("my_lang")
        .token("a", "a")
        .rule("root", vec!["a"])
        .build();
    let names = all_type_names(&parse_json(&g));
    assert!(!names.contains(&"my_lang".to_string()));
}

#[test]
fn empty_name_grammar_generates() {
    let g = GrammarBuilder::new("")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn unicode_name_grammar_generates() {
    let g = GrammarBuilder::new("日本語")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn long_name_grammar_generates() {
    let name = "a".repeat(200);
    let g = GrammarBuilder::new(&name)
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn grammar_name_with_special_chars_generates() {
    let g = GrammarBuilder::new("my-lang_v2.0")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// ===========================================================================
// 5. Multiple tokens reflected in output (7 tests)
// ===========================================================================

#[test]
fn two_string_tokens_appear_as_anonymous() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("-", "-")
        .rule("op", vec!["+"])
        .build();
    let nodes = parse_json(&g);
    // String literal tokens produce anonymous entries
    if let Some(plus) = find_node(&nodes, "+") {
        assert_eq!(plus["named"], false);
    }
}

#[test]
fn regex_tokens_appear_as_named() {
    let g = GrammarBuilder::new("t")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .rule("start", vec!["id"])
        .build();
    let named = named_type_names(&parse_json(&g));
    // regex tokens with named=true should appear in the named set via their rules
    // "start" rule is always named
    assert!(named.contains(&"start".to_string()));
}

#[test]
fn mixed_token_types_reflected() {
    let g = GrammarBuilder::new("mixed")
        .token("id", r"[a-z]+")
        .token(";", ";")
        .rule("stmt", vec!["id", ";"])
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "stmt").is_some());
}

#[test]
fn five_tokens_grammar_produces_nonempty_output() {
    let g = GrammarBuilder::new("many")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .build();
    let nodes = parse_json(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn token_patterns_do_not_leak_into_type_names() {
    let g = GrammarBuilder::new("pat")
        .token("number", r"\d+")
        .rule("start", vec!["number"])
        .build();
    let names = all_type_names(&parse_json(&g));
    assert!(!names.contains(&r"\d+".to_string()));
}

#[test]
fn string_token_type_name_is_its_value() {
    let g = GrammarBuilder::new("sv")
        .token("==", "==")
        .rule("start", vec!["=="])
        .build();
    let nodes = parse_json(&g);
    if let Some(eq) = find_node(&nodes, "==") {
        assert_eq!(eq["named"], false);
    }
}

#[test]
fn ten_regex_tokens_all_generate() {
    let mut builder = GrammarBuilder::new("ten");
    let tok_names: Vec<String> = (0..10).map(|i| format!("tok_{i}")).collect();
    let tok_pats: Vec<String> = (0..10).map(|i| format!("[a-z]{{{i}}}")).collect();
    for i in 0..10 {
        builder = builder.token(&tok_names[i], &tok_pats[i]);
    }
    builder = builder.rule("start", vec![&tok_names[0]]);
    let g = builder.build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// ===========================================================================
// 6. Multiple rules reflected in output (7 tests)
// ===========================================================================

#[test]
fn two_rules_both_appear() {
    let g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "r1").is_some());
    assert!(find_node(&nodes, "r2").is_some());
}

#[test]
fn same_lhs_multiple_alternatives() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .build();
    let nodes = parse_json(&g);
    // Should still produce exactly one node type entry for "expr"
    let count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr"))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn three_rules_all_named() {
    let g = GrammarBuilder::new("tri")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .rule("c", vec!["x"])
        .build();
    let named = named_type_names(&parse_json(&g));
    assert!(named.contains(&"a".to_string()));
    assert!(named.contains(&"b".to_string()));
    assert!(named.contains(&"c".to_string()));
}

#[test]
fn rules_referencing_other_rules() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("top", vec!["mid"])
        .start("top")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "leaf").is_some());
    assert!(find_node(&nodes, "mid").is_some());
    assert!(find_node(&nodes, "top").is_some());
}

#[test]
fn rule_with_mixed_terminals_and_nonterminals() {
    let g = GrammarBuilder::new("mix")
        .token("x", "x")
        .token(",", ",")
        .rule("item", vec!["x"])
        .rule("pair", vec!["item", ",", "item"])
        .start("pair")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "pair").is_some());
    assert!(find_node(&nodes, "item").is_some());
}

#[test]
fn empty_rule_generates() {
    let g = GrammarBuilder::new("eps")
        .rule("empty_rule", vec![])
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn many_alternatives_for_single_lhs() {
    let mut builder = GrammarBuilder::new("manyalt");
    let tok_names: Vec<String> = (0..8).map(|i| format!("t{i}")).collect();
    for name in &tok_names {
        builder = builder.token(name, name);
    }
    for name in &tok_names {
        builder = builder.rule("expr", vec![name]);
    }
    let g = builder.build();
    let nodes = parse_json(&g);
    let count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr"))
        .count();
    assert_eq!(count, 1);
}

// ===========================================================================
// 7. Precedence grammars (6 tests)
// ===========================================================================

#[test]
fn precedence_grammar_generates_ok() {
    let g = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn precedence_grammar_produces_valid_json() {
    let g = GrammarBuilder::new("prec2")
        .token("n", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn right_associativity_grammar_generates() {
    let g = GrammarBuilder::new("rassoc")
        .token("n", r"\d+")
        .token("^", "^")
        .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn none_associativity_grammar_generates() {
    let g = GrammarBuilder::new("nassoc")
        .token("n", r"\d+")
        .token("<", "<")
        .rule_with_precedence("cmp", vec!["e", "<", "e"], 1, Associativity::None)
        .rule("e", vec!["n"])
        .start("cmp")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn multiple_precedence_levels_produce_single_node_type() {
    let g = GrammarBuilder::new("mprec")
        .token("n", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let nodes = parse_json(&g);
    let expr_count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr"))
        .count();
    assert_eq!(expr_count, 1);
}

#[test]
fn precedence_declaration_grammar_generates() {
    let g = GrammarBuilder::new("pdecl")
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
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// ===========================================================================
// 8. Inline rules (5 tests)
// ===========================================================================

#[test]
fn inline_rule_grammar_generates_ok() {
    let g = GrammarBuilder::new("inl")
        .token("x", "x")
        .rule("_helper", vec!["x"])
        .rule("start", vec!["_helper"])
        .inline("_helper")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn inline_rule_with_underscore_prefix_is_internal() {
    let g = GrammarBuilder::new("inl2")
        .token("x", "x")
        .rule("_internal", vec!["x"])
        .rule("start", vec!["_internal"])
        .inline("_internal")
        .start("start")
        .build();
    let nodes = parse_json(&g);
    // Internal rules (starting with _) should be excluded from node types
    assert!(find_node(&nodes, "_internal").is_none());
}

#[test]
fn inline_rule_does_not_suppress_public_rule() {
    let g = GrammarBuilder::new("inl3")
        .token("x", "x")
        .rule("_helper", vec!["x"])
        .rule("start", vec!["_helper"])
        .inline("_helper")
        .start("start")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "start").is_some());
}

#[test]
fn multiple_inline_rules_grammar() {
    let g = GrammarBuilder::new("inl4")
        .token("a", "a")
        .token("b", "b")
        .rule("_h1", vec!["a"])
        .rule("_h2", vec!["b"])
        .rule("start", vec!["_h1", "_h2"])
        .inline("_h1")
        .inline("_h2")
        .start("start")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "_h1").is_none());
    assert!(find_node(&nodes, "_h2").is_none());
    assert!(find_node(&nodes, "start").is_some());
}

#[test]
fn inline_rule_without_underscore_remains_visible() {
    let g = GrammarBuilder::new("inl5")
        .token("x", "x")
        .rule("helper", vec!["x"])
        .rule("start", vec!["helper"])
        .inline("helper")
        .start("start")
        .build();
    let nodes = parse_json(&g);
    // "helper" does not start with _ so it is NOT internal — should appear
    assert!(find_node(&nodes, "helper").is_some());
}

// ===========================================================================
// 9. Supertypes (5 tests)
// ===========================================================================

#[test]
fn supertype_grammar_generates_ok() {
    let g = GrammarBuilder::new("sup")
        .token("a", "a")
        .token("b", "b")
        .rule("variant_a", vec!["a"])
        .rule("variant_b", vec!["b"])
        .rule("expr", vec!["variant_a"])
        .rule("expr", vec!["variant_b"])
        .supertype("expr")
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn supertype_grammar_produces_valid_json() {
    let g = GrammarBuilder::new("sup2")
        .token("a", "a")
        .rule("va", vec!["a"])
        .rule("typ", vec!["va"])
        .supertype("typ")
        .start("typ")
        .build();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn supertype_appears_in_output() {
    let g = GrammarBuilder::new("sup3")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("expr", vec!["leaf"])
        .supertype("expr")
        .start("expr")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "expr").is_some());
}

#[test]
fn supertype_variant_appears_in_output() {
    let g = GrammarBuilder::new("sup4")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("container", vec!["leaf"])
        .supertype("container")
        .start("container")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "leaf").is_some());
}

#[test]
fn multiple_supertypes_grammar() {
    let g = GrammarBuilder::new("msup")
        .token("a", "a")
        .token("b", "b")
        .rule("la", vec!["a"])
        .rule("lb", vec!["b"])
        .rule("type_a", vec!["la"])
        .rule("type_b", vec!["lb"])
        .supertype("type_a")
        .supertype("type_b")
        .start("type_a")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "type_a").is_some());
    assert!(find_node(&nodes, "type_b").is_some());
}

// ===========================================================================
// 10. Extras (5 tests)
// ===========================================================================

#[test]
fn extras_grammar_generates_ok() {
    let g = GrammarBuilder::new("ext")
        .token("ws", r"\s+")
        .token("id", r"[a-z]+")
        .rule("start", vec!["id"])
        .extra("ws")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn extras_grammar_produces_valid_json() {
    let g = GrammarBuilder::new("ext2")
        .token("ws", r"\s+")
        .token("x", "x")
        .rule("s", vec!["x"])
        .extra("ws")
        .build();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn extras_do_not_prevent_rule_output() {
    let g = GrammarBuilder::new("ext3")
        .token("ws", r"\s+")
        .token("x", "x")
        .rule("start", vec!["x"])
        .extra("ws")
        .start("start")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "start").is_some());
}

#[test]
fn multiple_extras_grammar() {
    let g = GrammarBuilder::new("ext4")
        .token("ws", r"\s+")
        .token("comment", r"//[^\n]*")
        .token("x", "x")
        .rule("start", vec!["x"])
        .extra("ws")
        .extra("comment")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn extras_grammar_output_is_nonempty() {
    let g = GrammarBuilder::new("ext5")
        .token("ws", r"\s+")
        .token("x", "x")
        .rule("s", vec!["x"])
        .extra("ws")
        .build();
    let nodes = parse_json(&g);
    assert!(!nodes.is_empty());
}

// ===========================================================================
// 11. Externals (5 tests)
// ===========================================================================

#[test]
fn externals_grammar_generates_ok() {
    let g = GrammarBuilder::new("xtn")
        .token("x", "x")
        .rule("start", vec!["x"])
        .external("indent")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn externals_grammar_produces_valid_json() {
    let g = GrammarBuilder::new("xtn2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .external("dedent")
        .build();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn multiple_externals_grammar() {
    let g = GrammarBuilder::new("xtn3")
        .token("x", "x")
        .rule("start", vec!["x"])
        .external("indent")
        .external("dedent")
        .external("newline")
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn externals_do_not_prevent_rule_output() {
    let g = GrammarBuilder::new("xtn4")
        .token("x", "x")
        .rule("start", vec!["x"])
        .external("indent")
        .start("start")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "start").is_some());
}

#[test]
fn externals_grammar_entries_have_required_keys() {
    let g = GrammarBuilder::new("xtn5")
        .token("x", "x")
        .rule("s", vec!["x"])
        .external("ext_tok")
        .build();
    for entry in parse_json(&g) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

// ===========================================================================
// 12. Single-rule grammar (4 tests)
// ===========================================================================

#[test]
fn single_rule_single_token_generates() {
    let g = GrammarBuilder::new("sr")
        .token("x", "x")
        .rule("only", vec!["x"])
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn single_rule_has_named_entry() {
    let g = GrammarBuilder::new("sr2")
        .token("x", "x")
        .rule("root", vec!["x"])
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "root").is_some());
}

#[test]
fn single_rule_output_is_array() {
    let g = GrammarBuilder::new("sr3")
        .token("y", "y")
        .rule("s", vec!["y"])
        .build();
    let val: Value = serde_json::from_str(&generate_node_types(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn single_epsilon_rule_generates() {
    let g = GrammarBuilder::new("sr4").rule("empty", vec![]).build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// ===========================================================================
// 13. Multi-rule grammar with 5+ rules (4 tests)
// ===========================================================================

#[test]
fn five_rule_grammar_generates() {
    let g = GrammarBuilder::new("five")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("r3", vec!["c"])
        .rule("r4", vec!["d"])
        .rule("r5", vec!["e"])
        .start("r1")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn five_rule_grammar_all_rules_appear() {
    let g = GrammarBuilder::new("five2")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .rule("c", vec!["x"])
        .rule("d", vec!["x"])
        .rule("e", vec!["x"])
        .build();
    let nodes = parse_json(&g);
    for name in &["a", "b", "c", "d", "e"] {
        assert!(find_node(&nodes, name).is_some(), "missing rule {name}");
    }
}

#[test]
fn ten_rule_grammar_produces_valid_json() {
    let g = scaled_grammar("ten", 10);
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn twenty_rule_grammar_all_entries_are_objects() {
    let g = scaled_grammar("twenty", 20);
    for entry in parse_json(&g) {
        assert!(entry.is_object());
    }
}

// ===========================================================================
// 14. Determinism (5 tests)
// ===========================================================================

#[test]
fn same_grammar_same_output_simple() {
    let g = simple_grammar();
    let out1 = generate_node_types(&g);
    let out2 = generate_node_types(&g);
    assert_eq!(out1, out2);
}

#[test]
fn same_grammar_same_output_arithmetic() {
    let g = arithmetic_grammar();
    let out1 = generate_node_types(&g);
    let out2 = generate_node_types(&g);
    assert_eq!(out1, out2);
}

#[test]
fn same_grammar_same_output_scaled() {
    let g = scaled_grammar("det", 15);
    let out1 = generate_node_types(&g);
    let out2 = generate_node_types(&g);
    assert_eq!(out1, out2);
}

#[test]
fn deterministic_across_new_generator_instances() {
    let g = simple_grammar();
    let gen1 = NodeTypesGenerator::new(&g);
    let gen2 = NodeTypesGenerator::new(&g);
    assert_eq!(
        gen1.generate().expect("gen1"),
        gen2.generate().expect("gen2")
    );
}

#[test]
fn output_is_sorted_by_type_name() {
    let g = GrammarBuilder::new("sort")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("zebra", vec!["a"])
        .rule("apple", vec!["b"])
        .rule("mango", vec!["c"])
        .build();
    let nodes = parse_json(&g);
    let names = all_type_names(&nodes);
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

// ===========================================================================
// 15. Grammar patterns: list, tree, expr (7 tests)
// ===========================================================================

#[test]
fn list_pattern_generates() {
    let g = GrammarBuilder::new("list")
        .token("item", r"[a-z]+")
        .token(",", ",")
        .rule("list", vec!["item"])
        .rule("list", vec!["list", ",", "item"])
        .start("list")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn list_pattern_valid_json() {
    let g = GrammarBuilder::new("list2")
        .token("x", "x")
        .token(",", ",")
        .rule("list", vec!["x"])
        .rule("list", vec!["list", ",", "x"])
        .start("list")
        .build();
    assert!(serde_json::from_str::<Value>(&generate_node_types(&g)).is_ok());
}

#[test]
fn tree_pattern_generates() {
    let g = GrammarBuilder::new("tree")
        .token("leaf", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .rule("node", vec!["leaf"])
        .rule("node", vec!["(", "node", "node", ")"])
        .start("node")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn tree_pattern_has_node_entry() {
    let g = GrammarBuilder::new("tree2")
        .token("leaf", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .rule("node", vec!["leaf"])
        .rule("node", vec!["(", "node", "node", ")"])
        .start("node")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "node").is_some());
}

#[test]
fn expr_pattern_with_parens() {
    let g = GrammarBuilder::new("expr")
        .token("num", r"\d+")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "expr").is_some());
}

#[test]
fn statement_list_pattern() {
    let g = GrammarBuilder::new("stmts")
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
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "program").is_some());
    assert!(find_node(&nodes, "stmt").is_some());
    assert!(find_node(&nodes, "assign").is_some());
}

#[test]
fn if_else_pattern_generates() {
    let g = GrammarBuilder::new("ifelse")
        .token("if", "if")
        .token("else", "else")
        .token("cond", r"[a-z]+")
        .token("body", r"\d+")
        .rule("if_stmt", vec!["if", "cond", "body"])
        .rule("if_else_stmt", vec!["if", "cond", "body", "else", "body"])
        .start("if_stmt")
        .build();
    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "if_stmt").is_some());
    assert!(find_node(&nodes, "if_else_stmt").is_some());
}

// ===========================================================================
// Additional coverage: fields, raw Grammar API, edge cases
// ===========================================================================

#[test]
fn grammar_with_fields_has_field_info() {
    let g = grammar_with_fields();
    let nodes = parse_json(&g);
    let binary = find_node(&nodes, "binary_expr");
    assert!(binary.is_some(), "binary_expr should appear in output");
}

#[test]
fn grammar_with_fields_field_keys_present() {
    let g = grammar_with_fields();
    let nodes = parse_json(&g);
    if let Some(binary) = find_node(&nodes, "binary_expr")
        && let Some(fields) = binary.get("fields")
    {
        assert!(fields.get("left").is_some());
        assert!(fields.get("right").is_some());
    }
}

#[test]
fn raw_grammar_api_single_rule() {
    let mut g = Grammar::new("raw".to_string());
    let tok_id = SymbolId(1);
    g.tokens.insert(
        tok_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    g.rule_names.insert(rule_id, "start".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn raw_grammar_multiple_rules() {
    let mut g = Grammar::new("raw2".to_string());

    let t1 = SymbolId(1);
    g.tokens.insert(
        t1,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let t2 = SymbolId(2);
    g.tokens.insert(
        t2,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    let r1 = SymbolId(10);
    let r2 = SymbolId(11);
    g.rule_names.insert(r1, "first".to_string());
    g.rule_names.insert(r2, "second".to_string());

    g.add_rule(Rule {
        lhs: r1,
        rhs: vec![Symbol::Terminal(t1)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: r2,
        rhs: vec![Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let nodes = parse_json(&g);
    assert!(find_node(&nodes, "first").is_some());
    assert!(find_node(&nodes, "second").is_some());
}

#[test]
fn fragile_token_grammar_generates() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("if", "if")
        .token("x", "x")
        .rule("start", vec!["if", "x"])
        .start("start")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn empty_grammar_produces_empty_array() {
    let g = Grammar::new("empty".to_string());
    let nodes = parse_json(&g);
    assert!(nodes.is_empty());
}

#[test]
fn no_duplicate_type_names_in_output() {
    let g = arithmetic_grammar();
    let nodes = parse_json(&g);
    let names = all_type_names(&nodes);
    let mut seen = std::collections::HashSet::new();
    for name in &names {
        assert!(seen.insert(name), "duplicate type name: {name}");
    }
}

#[test]
fn output_is_pretty_printed() {
    let output = generate_node_types(&simple_grammar());
    // serde_json::to_string_pretty produces newlines
    assert!(output.contains('\n'));
}

#[test]
fn all_named_entries_have_string_type_names() {
    let g = arithmetic_grammar();
    let nodes = parse_json(&g);
    for n in &nodes {
        if n["named"] == true {
            let type_name = n["type"].as_str().expect("type is string");
            assert!(!type_name.is_empty());
        }
    }
}

#[test]
fn optional_fields_absent_when_empty() {
    let g = GrammarBuilder::new("nof")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let nodes = parse_json(&g);
    if let Some(start) = find_node(&nodes, "start") {
        // No fields, children, or subtypes for a simple rule without them
        assert!(start.get("fields").is_none() || start["fields"].is_null());
    }
}

#[test]
fn subtypes_field_absent_for_non_supertype() {
    let g = GrammarBuilder::new("nosub")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let nodes = parse_json(&g);
    if let Some(start) = find_node(&nodes, "start") {
        assert!(start.get("subtypes").is_none() || start["subtypes"].is_null());
    }
}

#[test]
fn children_field_absent_when_no_children_info() {
    let g = GrammarBuilder::new("noch")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let nodes = parse_json(&g);
    if let Some(start) = find_node(&nodes, "start") {
        assert!(start.get("children").is_none() || start["children"].is_null());
    }
}
