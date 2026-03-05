//! Comprehensive v5 tests for NODE_TYPES JSON generation in adze-tablegen.
//!
//! Categories:
//!   1. Node types is valid JSON (8 tests)
//!   2. Node types contain grammar symbols (8 tests)
//!   3. Named vs anonymous (8 tests)
//!   4. Field information (7 tests)
//!   5. Node types determinism (8 tests)
//!   6. Node types scale (8 tests)
//!   7. Edge cases (8 tests)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate node types JSON from a grammar and parse into a Vec of JSON values.
fn generate_and_parse(grammar: &Grammar) -> Vec<Value> {
    let generator = NodeTypesGenerator::new(grammar);
    let json = generator
        .generate()
        .expect("NodeTypesGenerator::generate() failed");
    let val: Value = serde_json::from_str(&json).expect("output is not valid JSON");
    val.as_array().expect("output is not a JSON array").to_vec()
}

/// Find a node type entry by its `type` field.
fn find_node<'a>(nodes: &'a [Value], type_name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(type_name))
}

/// Build a simple arithmetic-like grammar with the given name.
fn arithmetic_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("number", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["number"])
        .rule("add", vec!["expr", "+", "expr"])
        .rule("mul", vec!["expr", "*", "expr"])
        .start("add")
        .build()
}

// ===========================================================================
// 1. Node types is valid JSON (8 tests)
// ===========================================================================

#[test]
fn valid_json_single_token() {
    let grammar = GrammarBuilder::new("vj1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .build();
    let generator = NodeTypesGenerator::new(&grammar);
    let json = generator.generate().unwrap();
    let val: Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(val.is_array());
}

#[test]
fn valid_json_multiple_tokens() {
    let grammar = GrammarBuilder::new("vj2")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["x", "y", "z"])
        .build();
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert!(val.is_array());
}

#[test]
fn valid_json_with_regex_tokens() {
    let grammar = GrammarBuilder::new("vj3")
        .token("ident", r"[a-z]+")
        .token("num", r"\d+")
        .rule("pair", vec!["ident", "num"])
        .build();
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn valid_json_with_multiple_rules() {
    let grammar = arithmetic_grammar("vj4");
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert!(val.as_array().is_some());
}

#[test]
fn valid_json_empty_grammar() {
    let grammar = Grammar::new("vj5".to_string());
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert!(val.is_array());
}

#[test]
fn valid_json_entries_have_type_and_named() {
    let grammar = GrammarBuilder::new("vj6")
        .token("lit", "lit")
        .rule("root", vec!["lit"])
        .build();
    for entry in generate_and_parse(&grammar) {
        assert!(entry.get("type").and_then(Value::as_str).is_some());
        assert!(entry.get("named").and_then(Value::as_bool).is_some());
    }
}

#[test]
fn valid_json_no_null_entries() {
    let grammar = GrammarBuilder::new("vj7")
        .token("a", "a")
        .token("b", r"b+")
        .rule("ab", vec!["a", "b"])
        .build();
    let nodes = generate_and_parse(&grammar);
    for entry in &nodes {
        assert!(!entry.is_null(), "null entry found");
    }
}

#[test]
fn valid_json_all_entries_are_objects() {
    let grammar = GrammarBuilder::new("vj8")
        .token("k", "k")
        .token("v", r"\d+")
        .rule("kv", vec!["k", "v"])
        .build();
    let nodes = generate_and_parse(&grammar);
    for entry in &nodes {
        assert!(entry.is_object(), "expected object, got: {entry}");
    }
}

// ===========================================================================
// 2. Node types contain grammar symbols (8 tests)
// ===========================================================================

#[test]
fn contains_token_name_via_rule() {
    let grammar = GrammarBuilder::new("cs1")
        .token("identifier", r"[a-z]+")
        .rule("start", vec!["identifier"])
        .build();
    let nodes = generate_and_parse(&grammar);
    // Token names appear indirectly through rule references; the rule using
    // the token must be present.
    assert!(
        find_node(&nodes, "start").is_some(),
        "rule referencing token should be in output"
    );
}

#[test]
fn contains_rule_name() {
    let grammar = GrammarBuilder::new("cs2")
        .token("num", r"\d+")
        .rule("expression", vec!["num"])
        .build();
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(
        json.contains("expression"),
        "output should mention rule name"
    );
}

#[test]
fn contains_literal_token() {
    let grammar = GrammarBuilder::new("cs3")
        .token("+", "+")
        .token("n", r"\d+")
        .rule("sum", vec!["n", "+", "n"])
        .build();
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(json.contains("+"), "output should contain literal '+'");
}

#[test]
fn contains_all_rule_names() {
    let grammar = arithmetic_grammar("cs4");
    let nodes = generate_and_parse(&grammar);
    assert!(find_node(&nodes, "expr").is_some(), "missing 'expr'");
    assert!(find_node(&nodes, "add").is_some(), "missing 'add'");
    assert!(find_node(&nodes, "mul").is_some(), "missing 'mul'");
}

#[test]
fn contains_rule_using_regex_token() {
    let grammar = GrammarBuilder::new("cs5")
        .token("float", r"\d+\.\d+")
        .rule("value", vec!["float"])
        .build();
    let nodes = generate_and_parse(&grammar);
    assert!(
        find_node(&nodes, "value").is_some(),
        "rule 'value' using regex token should be present"
    );
}

#[test]
fn contains_multiple_string_tokens() {
    let grammar = GrammarBuilder::new("cs6")
        .token("(", "(")
        .token(")", ")")
        .token("id", r"[a-z]+")
        .rule("paren", vec!["(", "id", ")"])
        .build();
    let nodes = generate_and_parse(&grammar);
    assert!(find_node(&nodes, "(").is_some(), "missing '('");
    assert!(find_node(&nodes, ")").is_some(), "missing ')'");
}

#[test]
fn contains_start_rule_name() {
    let grammar = GrammarBuilder::new("cs7")
        .token("tok", "tok")
        .rule("program", vec!["tok"])
        .start("program")
        .build();
    let nodes = generate_and_parse(&grammar);
    assert!(find_node(&nodes, "program").is_some(), "missing 'program'");
}

#[test]
fn contains_alternative_rule_productions() {
    let grammar = GrammarBuilder::new("cs8")
        .token("num", r"\d+")
        .token("str", r#""[^"]*""#)
        .rule("literal", vec!["num"])
        .rule("literal", vec!["str"])
        .build();
    let nodes = generate_and_parse(&grammar);
    assert!(
        find_node(&nodes, "literal").is_some(),
        "missing 'literal' with alternatives"
    );
}

// ===========================================================================
// 3. Named vs anonymous (8 tests)
// ===========================================================================

#[test]
fn named_rule_is_named() {
    let grammar = GrammarBuilder::new("na1")
        .token("n", r"\d+")
        .rule("expression", vec!["n"])
        .build();
    let nodes = generate_and_parse(&grammar);
    let expr = find_node(&nodes, "expression").expect("missing expression");
    assert_eq!(expr["named"], true, "rule should be named");
}

#[test]
fn string_literal_token_is_anonymous() {
    let grammar = GrammarBuilder::new("na2")
        .token(";", ";")
        .token("id", r"[a-z]+")
        .rule("stmt", vec!["id", ";"])
        .build();
    let nodes = generate_and_parse(&grammar);
    let semi = find_node(&nodes, ";").expect("missing ';'");
    assert_eq!(semi["named"], false, "literal ';' should be anonymous");
}

#[test]
fn regex_token_is_named() {
    let grammar = GrammarBuilder::new("na3")
        .token("word", r"[a-z]+")
        .rule("doc", vec!["word"])
        .build();
    let nodes = generate_and_parse(&grammar);
    let anon_words: Vec<_> = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("word") && n["named"] == false)
        .collect();
    assert!(
        anon_words.is_empty(),
        "regex token 'word' should not be anonymous"
    );
}

#[test]
fn multiple_literal_tokens_all_anonymous() {
    let grammar = GrammarBuilder::new("na4")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .token("id", r"[a-z]+")
        .rule("block", vec!["{", "id", "}"])
        .rule("group", vec!["(", "id", ")"])
        .build();
    let nodes = generate_and_parse(&grammar);
    for lit in ["(", ")", "{", "}"] {
        if let Some(node) = find_node(&nodes, lit) {
            assert_eq!(node["named"], false, "literal '{lit}' should be anonymous");
        }
    }
}

#[test]
fn mixed_named_and_anonymous() {
    let grammar = GrammarBuilder::new("na5")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("sum", vec!["num", "+", "num"])
        .build();
    let nodes = generate_and_parse(&grammar);
    let named_count = nodes.iter().filter(|n| n["named"] == true).count();
    let anon_count = nodes.iter().filter(|n| n["named"] == false).count();
    assert!(named_count > 0, "should have named entries");
    assert!(anon_count > 0, "should have anonymous entries");
}

#[test]
fn rule_with_only_literals_still_named() {
    let grammar = GrammarBuilder::new("na6")
        .token("(", "(")
        .token(")", ")")
        .rule("parens", vec!["(", ")"])
        .build();
    let nodes = generate_and_parse(&grammar);
    let parens = find_node(&nodes, "parens").expect("missing 'parens'");
    assert_eq!(parens["named"], true, "rule node should be named");
}

#[test]
fn anonymous_entries_have_no_fields_key() {
    let grammar = GrammarBuilder::new("na7")
        .token(",", ",")
        .token("n", r"\d+")
        .rule("list", vec!["n", ",", "n"])
        .build();
    let nodes = generate_and_parse(&grammar);
    let comma = find_node(&nodes, ",").expect("missing ','");
    assert!(
        comma.get("fields").is_none() || comma["fields"].as_object().is_none_or(|f| f.is_empty()),
        "anonymous node should have no fields"
    );
}

#[test]
fn all_named_nodes_are_objects_with_type() {
    let grammar = arithmetic_grammar("na8");
    let nodes = generate_and_parse(&grammar);
    let named: Vec<_> = nodes.iter().filter(|n| n["named"] == true).collect();
    assert!(!named.is_empty(), "should have some named nodes");
    for node in &named {
        assert!(
            node["type"].as_str().is_some(),
            "named node must have 'type' string"
        );
    }
}

// ===========================================================================
// 4. Field information (7 tests)
// ===========================================================================

/// Helper: build a grammar with fields using raw Grammar API.
fn grammar_with_fields() -> Grammar {
    let mut grammar = Grammar::new("fields_grammar".to_string());

    let num_id = SymbolId(0);
    grammar.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let op_id = SymbolId(1);
    grammar.tokens.insert(
        op_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(10);
    grammar
        .rule_names
        .insert(expr_id, "binary_expr".to_string());

    let left_field = FieldId(0);
    let right_field = FieldId(1);
    grammar.fields.insert(left_field, "left".to_string());
    grammar.fields.insert(right_field, "right".to_string());

    grammar.add_rule(Rule {
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

    grammar
}

#[test]
fn field_names_appear_in_output() {
    let grammar = grammar_with_fields();
    let nodes = generate_and_parse(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let fields = expr.get("fields").expect("binary_expr should have fields");
    assert!(fields.get("left").is_some(), "missing 'left' field");
    assert!(fields.get("right").is_some(), "missing 'right' field");
}

#[test]
fn field_types_are_arrays() {
    let grammar = grammar_with_fields();
    let nodes = generate_and_parse(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let fields = &expr["fields"];
    let left_types = fields["left"]["types"].as_array();
    assert!(left_types.is_some(), "'left' field types should be array");
}

#[test]
fn field_type_entries_have_type_and_named() {
    let grammar = grammar_with_fields();
    let nodes = generate_and_parse(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let left_types = expr["fields"]["left"]["types"]
        .as_array()
        .expect("types array");
    for entry in left_types {
        assert!(entry.get("type").is_some(), "type entry missing 'type'");
        assert!(entry.get("named").is_some(), "type entry missing 'named'");
    }
}

#[test]
fn field_references_correct_symbol() {
    let grammar = grammar_with_fields();
    let nodes = generate_and_parse(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let left_types = expr["fields"]["left"]["types"]
        .as_array()
        .expect("types array");
    assert!(!left_types.is_empty(), "field types should not be empty");
    let first = &left_types[0];
    assert_eq!(first["type"].as_str(), Some("number"));
}

#[test]
fn rule_without_fields_has_empty_or_no_fields() {
    let grammar = GrammarBuilder::new("nf1")
        .token("tok", "tok")
        .rule("simple", vec!["tok"])
        .build();
    let nodes = generate_and_parse(&grammar);
    let simple = find_node(&nodes, "simple").expect("missing 'simple'");
    if let Some(fields) = simple.get("fields") {
        let obj = fields.as_object().expect("fields should be object");
        assert!(obj.is_empty(), "fieldless rule should have empty fields");
    }
}

#[test]
fn single_field_grammar() {
    let mut grammar = Grammar::new("single_field".to_string());

    let tok_id = SymbolId(0);
    grammar.tokens.insert(
        tok_id,
        Token {
            name: "value".to_string(),
            pattern: TokenPattern::Regex(r"\w+".to_string()),
            fragile: false,
        },
    );

    let wrapper_id = SymbolId(10);
    grammar.rule_names.insert(wrapper_id, "wrapper".to_string());

    let content_field = FieldId(0);
    grammar.fields.insert(content_field, "content".to_string());

    grammar.add_rule(Rule {
        lhs: wrapper_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![(content_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&grammar);
    let wrapper = find_node(&nodes, "wrapper").expect("missing wrapper");
    assert!(
        wrapper["fields"].get("content").is_some(),
        "missing 'content' field"
    );
}

#[test]
fn multiple_fields_all_present() {
    let mut grammar = Grammar::new("multi_field".to_string());

    let a_id = SymbolId(0);
    grammar.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::Regex(r"a+".to_string()),
            fragile: false,
        },
    );
    let b_id = SymbolId(1);
    grammar.tokens.insert(
        b_id,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::Regex(r"b+".to_string()),
            fragile: false,
        },
    );
    let c_id = SymbolId(2);
    grammar.tokens.insert(
        c_id,
        Token {
            name: "c".to_string(),
            pattern: TokenPattern::Regex(r"c+".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(10);
    grammar.rule_names.insert(rule_id, "triple".to_string());

    let f0 = FieldId(0);
    let f1 = FieldId(1);
    let f2 = FieldId(2);
    grammar.fields.insert(f0, "first".to_string());
    grammar.fields.insert(f1, "second".to_string());
    grammar.fields.insert(f2, "third".to_string());

    grammar.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
            Symbol::Terminal(c_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(f0, 0), (f1, 1), (f2, 2)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&grammar);
    let triple = find_node(&nodes, "triple").expect("missing triple");
    let fields = triple["fields"]
        .as_object()
        .expect("fields should be object");
    assert!(fields.contains_key("first"));
    assert!(fields.contains_key("second"));
    assert!(fields.contains_key("third"));
}

// ===========================================================================
// 5. Node types determinism (8 tests)
// ===========================================================================

#[test]
fn deterministic_single_token() {
    let make = || {
        GrammarBuilder::new("det1")
            .token("x", "x")
            .rule("s", vec!["x"])
            .build()
    };
    let j1 = NodeTypesGenerator::new(&make()).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&make()).generate().unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_multiple_rules() {
    let make = || arithmetic_grammar("det2");
    let j1 = NodeTypesGenerator::new(&make()).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&make()).generate().unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_json_value_equality() {
    let make = || arithmetic_grammar("det3");
    let v1: Value =
        serde_json::from_str(&NodeTypesGenerator::new(&make()).generate().unwrap()).unwrap();
    let v2: Value =
        serde_json::from_str(&NodeTypesGenerator::new(&make()).generate().unwrap()).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn deterministic_across_ten_invocations() {
    let make = || {
        GrammarBuilder::new("det4")
            .token("a", "a")
            .token("b", "b")
            .rule("ab", vec!["a", "b"])
            .build()
    };
    let baseline = NodeTypesGenerator::new(&make()).generate().unwrap();
    for _ in 0..10 {
        let output = NodeTypesGenerator::new(&make()).generate().unwrap();
        assert_eq!(baseline, output, "output must be identical every time");
    }
}

#[test]
fn deterministic_with_regex_tokens() {
    let make = || {
        GrammarBuilder::new("det5")
            .token("ident", r"[a-zA-Z_]\w*")
            .token("num", r"\d+")
            .rule("pair", vec!["ident", "num"])
            .build()
    };
    let j1 = NodeTypesGenerator::new(&make()).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&make()).generate().unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_with_start_symbol() {
    let make = || {
        GrammarBuilder::new("det6")
            .token("t", "t")
            .rule("root", vec!["t"])
            .start("root")
            .build()
    };
    let j1 = NodeTypesGenerator::new(&make()).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&make()).generate().unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_with_alternatives() {
    let make = || {
        GrammarBuilder::new("det7")
            .token("a", "a")
            .token("b", "b")
            .rule("choice", vec!["a"])
            .rule("choice", vec!["b"])
            .build()
    };
    let j1 = NodeTypesGenerator::new(&make()).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&make()).generate().unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_roundtrip_preserves_equality() {
    let grammar = arithmetic_grammar("det8");
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let v1: Value = serde_json::from_str(&json).unwrap();
    let reserialized = serde_json::to_string_pretty(&v1).unwrap();
    let v2: Value = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(v1, v2, "roundtrip must preserve JSON value equality");
}

// ===========================================================================
// 6. Node types scale (8 tests)
// ===========================================================================

/// Build a grammar with `n` distinct tokens and one rule per token.
fn scaled_grammar(name: &str, n: usize) -> Grammar {
    let mut builder = GrammarBuilder::new(name);
    for i in 0..n {
        let tok_name = format!("tok_{i}");
        let rule_name = format!("rule_{i}");
        builder = builder.token(&tok_name, &tok_name);
        builder = builder.rule(&rule_name, vec![&tok_name]);
    }
    builder.build()
}

#[test]
fn scale_two_rules_produce_entries() {
    let grammar = scaled_grammar("sc1", 2);
    let nodes = generate_and_parse(&grammar);
    assert!(!nodes.is_empty());
}

#[test]
fn scale_five_rules_more_than_two() {
    let nodes_2 = generate_and_parse(&scaled_grammar("sc2a", 2));
    let nodes_5 = generate_and_parse(&scaled_grammar("sc2b", 5));
    assert!(
        nodes_5.len() > nodes_2.len(),
        "5 rules should produce more entries than 2"
    );
}

#[test]
fn scale_ten_rules_more_than_five() {
    let nodes_5 = generate_and_parse(&scaled_grammar("sc3a", 5));
    let nodes_10 = generate_and_parse(&scaled_grammar("sc3b", 10));
    assert!(
        nodes_10.len() > nodes_5.len(),
        "10 rules should produce more entries than 5"
    );
}

#[test]
fn scale_twenty_rules() {
    let grammar = scaled_grammar("sc4", 20);
    let nodes = generate_and_parse(&grammar);
    // At minimum, 20 rules + 20 tokens = 40 entries
    assert!(
        nodes.len() >= 20,
        "20-rule grammar should have many entries"
    );
}

#[test]
fn scale_fifty_rules_still_valid_json() {
    let grammar = scaled_grammar("sc5", 50);
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let val: Value = serde_json::from_str(&json).expect("50-rule JSON must be valid");
    assert!(val.is_array());
}

#[test]
fn scale_entries_monotonically_increase() {
    let counts: Vec<usize> = [1, 3, 7, 15]
        .iter()
        .map(|&n| generate_and_parse(&scaled_grammar(&format!("sc6_{n}"), n)).len())
        .collect();
    for window in counts.windows(2) {
        assert!(
            window[1] > window[0],
            "entry count should increase: {counts:?}"
        );
    }
}

#[test]
fn scale_named_entries_grow_with_rules() {
    let count_named = |n: usize| -> usize {
        generate_and_parse(&scaled_grammar(&format!("sc7_{n}"), n))
            .iter()
            .filter(|e| e["named"] == true)
            .count()
    };
    assert!(count_named(10) > count_named(3));
}

#[test]
fn scale_hundred_rules_all_objects() {
    let grammar = scaled_grammar("sc8", 100);
    let nodes = generate_and_parse(&grammar);
    for entry in &nodes {
        assert!(entry.is_object(), "all entries must be objects");
    }
}

// ===========================================================================
// 7. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_empty_grammar_produces_empty_array() {
    let grammar = Grammar::new("edge1".to_string());
    let nodes = generate_and_parse(&grammar);
    assert!(nodes.is_empty(), "empty grammar should yield []");
}

#[test]
fn edge_single_token_single_rule() {
    let grammar = GrammarBuilder::new("edge2")
        .token("only", "only")
        .rule("root", vec!["only"])
        .build();
    let nodes = generate_and_parse(&grammar);
    assert!(
        !nodes.is_empty(),
        "single-token grammar should produce entries"
    );
}

#[test]
fn edge_many_tokens_no_rules() {
    let mut builder = GrammarBuilder::new("edge3");
    for i in 0..20 {
        builder = builder.token(&format!("t{i}"), &format!("t{i}"));
    }
    // No rules, just tokens — build without rules
    let grammar = builder.build();
    let generator = NodeTypesGenerator::new(&grammar);
    // Should not panic
    let result = generator.generate();
    assert!(result.is_ok(), "many tokens, no rules should not panic");
}

#[test]
fn edge_token_name_with_special_chars() {
    let grammar = GrammarBuilder::new("edge4")
        .token("!=", "!=")
        .token("==", "==")
        .token("id", r"[a-z]+")
        .rule("cmp", vec!["id", "!=", "id"])
        .build();
    let nodes = generate_and_parse(&grammar);
    assert!(find_node(&nodes, "!=").is_some(), "missing '!='");
    // "==" may or may not appear depending on whether it's referenced in a rule
    let _eq_node = find_node(&nodes, "==");
}

#[test]
fn edge_alternative_productions_same_lhs() {
    let grammar = GrammarBuilder::new("edge5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("multi", vec!["a"])
        .rule("multi", vec!["b"])
        .rule("multi", vec!["c"])
        .build();
    let nodes = generate_and_parse(&grammar);
    // 'multi' should appear exactly once
    let multi_count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("multi"))
        .count();
    assert_eq!(
        multi_count, 1,
        "'multi' should appear once despite 3 alternatives"
    );
}

#[test]
fn edge_deeply_nested_rules() {
    let grammar = GrammarBuilder::new("edge6")
        .token("x", "x")
        .rule("level0", vec!["x"])
        .rule("level1", vec!["level0"])
        .rule("level2", vec!["level1"])
        .rule("level3", vec!["level2"])
        .rule("level4", vec!["level3"])
        .build();
    let nodes = generate_and_parse(&grammar);
    for i in 0..5 {
        let name = format!("level{i}");
        assert!(find_node(&nodes, &name).is_some(), "missing '{name}'");
    }
}

#[test]
fn edge_internal_underscore_rule_excluded() {
    let mut grammar = Grammar::new("edge7".to_string());

    let tok_id = SymbolId(0);
    grammar.tokens.insert(
        tok_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    let visible_id = SymbolId(1);
    grammar.rule_names.insert(visible_id, "visible".to_string());
    grammar.add_rule(Rule {
        lhs: visible_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let hidden_id = SymbolId(2);
    grammar
        .rule_names
        .insert(hidden_id, "_internal".to_string());
    grammar.add_rule(Rule {
        lhs: hidden_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let nodes = generate_and_parse(&grammar);
    assert!(
        find_node(&nodes, "_internal").is_none(),
        "_internal should be excluded"
    );
    assert!(
        find_node(&nodes, "visible").is_some(),
        "visible should be present"
    );
}

#[test]
fn edge_grammar_name_does_not_affect_structure() {
    let g1 = GrammarBuilder::new("alpha")
        .token("t", "t")
        .rule("r", vec!["t"])
        .build();
    let g2 = GrammarBuilder::new("beta")
        .token("t", "t")
        .rule("r", vec!["t"])
        .build();

    let n1 = generate_and_parse(&g1);
    let n2 = generate_and_parse(&g2);

    // Same structure: same number of entries, same types
    assert_eq!(
        n1.len(),
        n2.len(),
        "grammar name should not affect entry count"
    );
    let types1: Vec<_> = n1.iter().filter_map(|n| n["type"].as_str()).collect();
    let types2: Vec<_> = n2.iter().filter_map(|n| n["type"].as_str()).collect();
    assert_eq!(types1, types2, "grammar name should not affect types");
}
