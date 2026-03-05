//! Comprehensive v5 tests for NODE_TYPES JSON generation in adze-tablegen.
//!
//! Categories:
//!   1. Node types output — valid JSON, non-empty (8 tests)
//!   2. Named/anonymous nodes — correct categorization (7 tests)
//!   3. Token types — leaf nodes in node types (7 tests)
//!   4. Nonterminal types — interior nodes in node types (7 tests)
//!   5. Children info — child types per node (7 tests)
//!   6. Determinism — same grammar → same node types (8 tests)
//!   7. Complex grammars — expressions, recursive, many types (7 tests)
//!   8. Edge cases — minimal grammar, many rules (8 tests)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate node types JSON string from a grammar.
fn gen_json(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("NodeTypesGenerator::generate() failed")
}

/// Generate node types JSON from a grammar and parse into a Vec of JSON values.
fn gen_parsed(grammar: &Grammar) -> Vec<Value> {
    let json = gen_json(grammar);
    let val: Value = serde_json::from_str(&json).expect("output is not valid JSON");
    val.as_array().expect("output is not a JSON array").to_vec()
}

/// Find a node type entry by its `type` field.
fn find_node<'a>(nodes: &'a [Value], type_name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(type_name))
}

/// Collect type names of all named entries.
fn named_types(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

/// Collect type names of all anonymous entries.
fn anon_types(nodes: &[Value]) -> Vec<String> {
    nodes
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect()
}

/// Build a simple arithmetic-like grammar.
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

/// Build a grammar with fields using the raw Grammar API.
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

/// Build a scaled grammar with `n` distinct literal tokens and one rule per token.
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

// ===========================================================================
// 1. Node types output — valid JSON, non-empty (8 tests)
// ===========================================================================

#[test]
fn output_single_token_is_valid_json() {
    let grammar = GrammarBuilder::new("o1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .build();
    let json = gen_json(&grammar);
    let val: Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(val.is_array());
}

#[test]
fn output_multiple_tokens_is_valid_json() {
    let grammar = GrammarBuilder::new("o2")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["x", "y", "z"])
        .build();
    let val: Value = serde_json::from_str(&gen_json(&grammar)).unwrap();
    assert!(val.is_array());
}

#[test]
fn output_with_regex_tokens_is_valid_json() {
    let grammar = GrammarBuilder::new("o3")
        .token("ident", r"[a-z]+")
        .token("num", r"\d+")
        .rule("pair", vec!["ident", "num"])
        .build();
    assert!(serde_json::from_str::<Value>(&gen_json(&grammar)).is_ok());
}

#[test]
fn output_arithmetic_grammar_is_array() {
    let val: Value = serde_json::from_str(&gen_json(&arithmetic_grammar())).unwrap();
    assert!(val.as_array().is_some());
}

#[test]
fn output_empty_grammar_is_valid_json() {
    let grammar = Grammar::new("o5".to_string());
    let val: Value = serde_json::from_str(&gen_json(&grammar)).unwrap();
    assert!(val.is_array());
}

#[test]
fn output_entries_have_type_and_named() {
    let grammar = GrammarBuilder::new("o6")
        .token("lit", "lit")
        .rule("root", vec!["lit"])
        .build();
    for entry in gen_parsed(&grammar) {
        assert!(entry.get("type").and_then(Value::as_str).is_some());
        assert!(entry.get("named").and_then(Value::as_bool).is_some());
    }
}

#[test]
fn output_contains_no_null_entries() {
    let grammar = GrammarBuilder::new("o7")
        .token("a", "a")
        .token("b", r"b+")
        .rule("ab", vec!["a", "b"])
        .build();
    for entry in &gen_parsed(&grammar) {
        assert!(!entry.is_null(), "null entry found");
    }
}

#[test]
fn output_all_entries_are_objects() {
    let grammar = GrammarBuilder::new("o8")
        .token("k", "k")
        .token("v", r"\d+")
        .rule("kv", vec!["k", "v"])
        .build();
    for entry in &gen_parsed(&grammar) {
        assert!(entry.is_object(), "expected object, got: {entry}");
    }
}

// ===========================================================================
// 2. Named/anonymous nodes — correct categorization (7 tests)
// ===========================================================================

#[test]
fn named_rule_appears_as_named() {
    let grammar = GrammarBuilder::new("na1")
        .token("n", r"\d+")
        .rule("expression", vec!["n"])
        .build();
    let nodes = gen_parsed(&grammar);
    let expr = find_node(&nodes, "expression").expect("missing expression");
    assert_eq!(expr["named"], true);
}

#[test]
fn string_literal_token_appears_as_anonymous() {
    let grammar = GrammarBuilder::new("na2")
        .token(";", ";")
        .token("id", r"[a-z]+")
        .rule("stmt", vec!["id", ";"])
        .build();
    let nodes = gen_parsed(&grammar);
    let semi = find_node(&nodes, ";").expect("missing ';'");
    assert_eq!(semi["named"], false);
}

#[test]
fn multiple_literals_all_anonymous() {
    let grammar = GrammarBuilder::new("na3")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .token("id", r"[a-z]+")
        .rule("block", vec!["{", "id", "}"])
        .rule("group", vec!["(", "id", ")"])
        .build();
    let nodes = gen_parsed(&grammar);
    for lit in ["(", ")", "{", "}"] {
        if let Some(node) = find_node(&nodes, lit) {
            assert_eq!(node["named"], false, "literal '{lit}' should be anonymous");
        }
    }
}

#[test]
fn mixed_grammar_has_both_named_and_anonymous() {
    let grammar = GrammarBuilder::new("na4")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("sum", vec!["num", "+", "num"])
        .build();
    let nodes = gen_parsed(&grammar);
    assert!(!named_types(&nodes).is_empty(), "should have named entries");
    assert!(
        !anon_types(&nodes).is_empty(),
        "should have anonymous entries"
    );
}

#[test]
fn rule_with_only_literals_is_still_named() {
    let grammar = GrammarBuilder::new("na5")
        .token("(", "(")
        .token(")", ")")
        .rule("parens", vec!["(", ")"])
        .build();
    let nodes = gen_parsed(&grammar);
    let parens = find_node(&nodes, "parens").expect("missing 'parens'");
    assert_eq!(parens["named"], true, "rule node should be named");
}

#[test]
fn anonymous_node_has_no_fields() {
    let grammar = GrammarBuilder::new("na6")
        .token(",", ",")
        .token("n", r"\d+")
        .rule("list", vec!["n", ",", "n"])
        .build();
    let nodes = gen_parsed(&grammar);
    let comma = find_node(&nodes, ",").expect("missing ','");
    assert!(
        comma.get("fields").is_none() || comma["fields"].as_object().is_none_or(|f| f.is_empty()),
        "anonymous node should have no fields"
    );
}

#[test]
fn all_named_nodes_have_type_string() {
    let nodes = gen_parsed(&arithmetic_grammar());
    let named: Vec<_> = nodes.iter().filter(|n| n["named"] == true).collect();
    assert!(!named.is_empty());
    for node in &named {
        assert!(
            node["type"].as_str().is_some(),
            "named node must have 'type' string"
        );
    }
}

// ===========================================================================
// 3. Token types — leaf nodes in node types (7 tests)
// ===========================================================================

#[test]
fn literal_token_appears_in_output() {
    let grammar = GrammarBuilder::new("tk1")
        .token("+", "+")
        .token("n", r"\d+")
        .rule("sum", vec!["n", "+", "n"])
        .build();
    let nodes = gen_parsed(&grammar);
    assert!(find_node(&nodes, "+").is_some(), "literal '+' must appear");
}

#[test]
fn literal_token_is_leaf_with_no_children() {
    let grammar = GrammarBuilder::new("tk2")
        .token(";", ";")
        .token("id", r"[a-z]+")
        .rule("stmt", vec!["id", ";"])
        .build();
    let nodes = gen_parsed(&grammar);
    let semi = find_node(&nodes, ";").expect("missing ';'");
    assert!(
        semi.get("children").is_none(),
        "token leaf should have no children"
    );
}

#[test]
fn literal_token_has_no_subtypes() {
    let grammar = GrammarBuilder::new("tk3")
        .token(".", ".")
        .token("id", r"[a-z]+")
        .rule("access", vec!["id", ".", "id"])
        .build();
    let nodes = gen_parsed(&grammar);
    let dot = find_node(&nodes, ".").expect("missing '.'");
    assert!(
        dot.get("subtypes").is_none(),
        "token leaf should have no subtypes"
    );
}

#[test]
fn multiple_literal_tokens_all_present() {
    let grammar = GrammarBuilder::new("tk4")
        .token("(", "(")
        .token(")", ")")
        .token("id", r"[a-z]+")
        .rule("paren", vec!["(", "id", ")"])
        .build();
    let nodes = gen_parsed(&grammar);
    assert!(find_node(&nodes, "(").is_some(), "missing '('");
    assert!(find_node(&nodes, ")").is_some(), "missing ')'");
}

#[test]
fn regex_token_not_top_level_anonymous() {
    // Regex tokens are named=true and don't appear as top-level anonymous nodes.
    let grammar = GrammarBuilder::new("tk5")
        .token("word", r"[a-z]+")
        .rule("doc", vec!["word"])
        .build();
    let nodes = gen_parsed(&grammar);
    let anon_words: Vec<_> = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("word") && n["named"] == false)
        .collect();
    assert!(anon_words.is_empty(), "regex token should not be anonymous");
}

#[test]
fn special_char_tokens_appear() {
    let grammar = GrammarBuilder::new("tk6")
        .token("!=", "!=")
        .token("==", "==")
        .token("id", r"[a-z]+")
        .rule("cmp", vec!["id", "!=", "id"])
        .rule("eq", vec!["id", "==", "id"])
        .build();
    let nodes = gen_parsed(&grammar);
    assert!(find_node(&nodes, "!=").is_some(), "missing '!='");
    assert!(find_node(&nodes, "==").is_some(), "missing '=='");
}

#[test]
fn unused_literal_token_still_appears() {
    // String-pattern tokens appear even if no rule references them.
    let grammar = GrammarBuilder::new("tk7")
        .token("used", "used")
        .token("unused", "unused")
        .rule("r", vec!["used"])
        .build();
    let nodes = gen_parsed(&grammar);
    // Both are string-literal tokens so both should appear as anonymous
    assert!(find_node(&nodes, "used").is_some(), "missing 'used'");
    assert!(find_node(&nodes, "unused").is_some(), "missing 'unused'");
}

// ===========================================================================
// 4. Nonterminal types — interior nodes in node types (7 tests)
// ===========================================================================

#[test]
fn nonterminal_rule_appears_as_named() {
    let grammar = GrammarBuilder::new("nt1")
        .token("num", r"\d+")
        .rule("expression", vec!["num"])
        .build();
    let nodes = gen_parsed(&grammar);
    let expr = find_node(&nodes, "expression").expect("missing 'expression'");
    assert_eq!(expr["named"], true);
}

#[test]
fn all_rules_appear_in_output() {
    let nodes = gen_parsed(&arithmetic_grammar());
    assert!(find_node(&nodes, "expr").is_some(), "missing 'expr'");
    assert!(find_node(&nodes, "add").is_some(), "missing 'add'");
    assert!(find_node(&nodes, "mul").is_some(), "missing 'mul'");
}

#[test]
fn rule_with_start_symbol_appears() {
    let grammar = GrammarBuilder::new("nt3")
        .token("tok", "tok")
        .rule("program", vec!["tok"])
        .start("program")
        .build();
    let nodes = gen_parsed(&grammar);
    assert!(find_node(&nodes, "program").is_some(), "missing 'program'");
}

#[test]
fn alternative_productions_yield_single_entry() {
    let grammar = GrammarBuilder::new("nt4")
        .token("num", r"\d+")
        .token("str", r#""[^"]*""#)
        .rule("literal", vec!["num"])
        .rule("literal", vec!["str"])
        .build();
    let nodes = gen_parsed(&grammar);
    let count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("literal"))
        .count();
    assert_eq!(count, 1, "'literal' should appear exactly once");
}

#[test]
fn nonterminal_without_fields_has_no_fields_key() {
    let grammar = GrammarBuilder::new("nt5")
        .token("tok", "tok")
        .rule("simple", vec!["tok"])
        .build();
    let nodes = gen_parsed(&grammar);
    let simple = find_node(&nodes, "simple").expect("missing 'simple'");
    if let Some(fields) = simple.get("fields") {
        let obj = fields.as_object().expect("fields should be object");
        assert!(obj.is_empty(), "fieldless rule should have empty fields");
    }
}

#[test]
fn nonterminal_with_fields_has_fields_key() {
    let grammar = grammar_with_fields();
    let nodes = gen_parsed(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    assert!(
        expr.get("fields").is_some(),
        "binary_expr should have fields"
    );
}

#[test]
fn internal_underscore_rule_excluded_from_output() {
    let mut grammar = Grammar::new("nt7".to_string());

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

    let nodes = gen_parsed(&grammar);
    assert!(
        find_node(&nodes, "_internal").is_none(),
        "_internal should be excluded"
    );
    assert!(
        find_node(&nodes, "visible").is_some(),
        "visible should be present"
    );
}

// ===========================================================================
// 5. Children info — child types per node (7 tests)
// ===========================================================================

#[test]
fn field_names_appear_in_node_with_fields() {
    let grammar = grammar_with_fields();
    let nodes = gen_parsed(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let fields = expr.get("fields").expect("binary_expr should have fields");
    assert!(fields.get("left").is_some(), "missing 'left' field");
    assert!(fields.get("right").is_some(), "missing 'right' field");
}

#[test]
fn field_types_entry_is_array() {
    let grammar = grammar_with_fields();
    let nodes = gen_parsed(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let left_types = expr["fields"]["left"]["types"].as_array();
    assert!(left_types.is_some(), "'left' field types should be array");
}

#[test]
fn field_type_entry_has_type_and_named() {
    let grammar = grammar_with_fields();
    let nodes = gen_parsed(&grammar);
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
fn field_references_correct_child_symbol() {
    let grammar = grammar_with_fields();
    let nodes = gen_parsed(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let left_types = expr["fields"]["left"]["types"]
        .as_array()
        .expect("types array");
    assert!(!left_types.is_empty(), "field types should not be empty");
    assert_eq!(left_types[0]["type"].as_str(), Some("number"));
}

#[test]
fn single_field_appears_in_output() {
    let mut grammar = Grammar::new("ch5".to_string());

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

    let nodes = gen_parsed(&grammar);
    let wrapper = find_node(&nodes, "wrapper").expect("missing wrapper");
    assert!(
        wrapper["fields"].get("content").is_some(),
        "missing 'content' field"
    );
}

#[test]
fn three_fields_all_present() {
    let mut grammar = Grammar::new("ch6".to_string());

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

    let nodes = gen_parsed(&grammar);
    let triple = find_node(&nodes, "triple").expect("missing triple");
    let fields = triple["fields"]
        .as_object()
        .expect("fields should be object");
    assert!(fields.contains_key("first"));
    assert!(fields.contains_key("second"));
    assert!(fields.contains_key("third"));
}

#[test]
fn field_required_flag_is_present() {
    let grammar = grammar_with_fields();
    let nodes = gen_parsed(&grammar);
    let expr = find_node(&nodes, "binary_expr").expect("missing binary_expr");
    let left = &expr["fields"]["left"];
    assert!(
        left.get("required").is_some(),
        "field should have 'required' flag"
    );
    assert!(
        left.get("multiple").is_some(),
        "field should have 'multiple' flag"
    );
}

// ===========================================================================
// 6. Determinism — same grammar → same node types (8 tests)
// ===========================================================================

#[test]
fn deterministic_single_token() {
    let make = || {
        GrammarBuilder::new("det1")
            .token("x", "x")
            .rule("s", vec!["x"])
            .build()
    };
    let j1 = gen_json(&make());
    let j2 = gen_json(&make());
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_multiple_rules() {
    let j1 = gen_json(&arithmetic_grammar());
    let j2 = gen_json(&arithmetic_grammar());
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_json_value_equality() {
    let v1: Value = serde_json::from_str(&gen_json(&arithmetic_grammar())).unwrap();
    let v2: Value = serde_json::from_str(&gen_json(&arithmetic_grammar())).unwrap();
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
    let baseline = gen_json(&make());
    for _ in 0..10 {
        assert_eq!(baseline, gen_json(&make()), "must be identical every time");
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
    assert_eq!(gen_json(&make()), gen_json(&make()));
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
    assert_eq!(gen_json(&make()), gen_json(&make()));
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
    assert_eq!(gen_json(&make()), gen_json(&make()));
}

#[test]
fn deterministic_roundtrip_preserves_equality() {
    let json = gen_json(&arithmetic_grammar());
    let v1: Value = serde_json::from_str(&json).unwrap();
    let reserialized = serde_json::to_string_pretty(&v1).unwrap();
    let v2: Value = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(v1, v2, "roundtrip must preserve JSON value equality");
}

// ===========================================================================
// 7. Complex grammars — expressions, recursive, many types (7 tests)
// ===========================================================================

#[test]
fn complex_arithmetic_has_all_rules() {
    let nodes = gen_parsed(&arithmetic_grammar());
    for name in ["expr", "add", "mul"] {
        assert!(find_node(&nodes, name).is_some(), "missing '{name}'");
    }
}

#[test]
fn complex_deeply_nested_rules_all_present() {
    let grammar = GrammarBuilder::new("cx2")
        .token("x", "x")
        .rule("level0", vec!["x"])
        .rule("level1", vec!["level0"])
        .rule("level2", vec!["level1"])
        .rule("level3", vec!["level2"])
        .rule("level4", vec!["level3"])
        .build();
    let nodes = gen_parsed(&grammar);
    for i in 0..5 {
        let name = format!("level{i}");
        assert!(find_node(&nodes, &name).is_some(), "missing '{name}'");
    }
}

#[test]
fn complex_many_operators() {
    let grammar = GrammarBuilder::new("cx3")
        .token("num", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("%", "%")
        .rule("expr", vec!["num"])
        .rule("add", vec!["expr", "+", "expr"])
        .rule("sub", vec!["expr", "-", "expr"])
        .rule("mul", vec!["expr", "*", "expr"])
        .rule("div", vec!["expr", "/", "expr"])
        .rule("modulo", vec!["expr", "%", "expr"])
        .start("add")
        .build();
    let nodes = gen_parsed(&grammar);
    for rule in ["expr", "add", "sub", "mul", "div", "modulo"] {
        assert!(find_node(&nodes, rule).is_some(), "missing '{rule}'");
    }
    for op in ["+", "-", "*", "/", "%"] {
        assert!(find_node(&nodes, op).is_some(), "missing '{op}'");
    }
}

#[test]
fn complex_statement_grammar() {
    let grammar = GrammarBuilder::new("cx4")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .token("=", "=")
        .token(";", ";")
        .token("if", "if")
        .token("else", "else")
        .token("{", "{")
        .token("}", "}")
        .rule("assign", vec!["id", "=", "num", ";"])
        .rule("block", vec!["{", "assign", "}"])
        .rule("if_stmt", vec!["if", "id", "block"])
        .rule("if_else", vec!["if", "id", "block", "else", "block"])
        .start("if_else")
        .build();
    let nodes = gen_parsed(&grammar);
    for rule in ["assign", "block", "if_stmt", "if_else"] {
        assert!(find_node(&nodes, rule).is_some(), "missing '{rule}'");
    }
}

#[test]
fn complex_twenty_rules_valid_json() {
    let grammar = scaled_grammar("cx5", 20);
    let nodes = gen_parsed(&grammar);
    assert!(
        nodes.len() >= 20,
        "20-rule grammar should have many entries"
    );
}

#[test]
fn complex_fifty_rules_all_objects() {
    let grammar = scaled_grammar("cx6", 50);
    let nodes = gen_parsed(&grammar);
    for entry in &nodes {
        assert!(entry.is_object(), "all entries must be objects");
    }
}

#[test]
fn complex_hundred_rules_entry_count() {
    let grammar = scaled_grammar("cx7", 100);
    let nodes = gen_parsed(&grammar);
    // 100 rules + 100 literal tokens = 200 entries
    assert!(
        nodes.len() >= 100,
        "100-rule grammar should have >= 100 entries"
    );
}

// ===========================================================================
// 8. Edge cases — minimal grammar, many rules (8 tests)
// ===========================================================================

#[test]
fn edge_empty_grammar_produces_empty_array() {
    let grammar = Grammar::new("e1".to_string());
    let nodes = gen_parsed(&grammar);
    assert!(nodes.is_empty(), "empty grammar should yield []");
}

#[test]
fn edge_single_token_single_rule() {
    let grammar = GrammarBuilder::new("e2")
        .token("only", "only")
        .rule("root", vec!["only"])
        .build();
    let nodes = gen_parsed(&grammar);
    assert!(
        !nodes.is_empty(),
        "single-token grammar should produce entries"
    );
}

#[test]
fn edge_many_tokens_no_rules_does_not_panic() {
    let mut builder = GrammarBuilder::new("e3");
    for i in 0..20 {
        builder = builder.token(&format!("t{i}"), &format!("t{i}"));
    }
    let grammar = builder.build();
    let result = NodeTypesGenerator::new(&grammar).generate();
    assert!(result.is_ok(), "many tokens, no rules should not panic");
}

#[test]
fn edge_alternative_productions_produce_one_entry() {
    let grammar = GrammarBuilder::new("e4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("multi", vec!["a"])
        .rule("multi", vec!["b"])
        .rule("multi", vec!["c"])
        .build();
    let nodes = gen_parsed(&grammar);
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
fn edge_grammar_name_does_not_affect_structure() {
    let g1 = GrammarBuilder::new("alpha")
        .token("t", "t")
        .rule("r", vec!["t"])
        .build();
    let g2 = GrammarBuilder::new("beta")
        .token("t", "t")
        .rule("r", vec!["t"])
        .build();

    let n1 = gen_parsed(&g1);
    let n2 = gen_parsed(&g2);

    assert_eq!(
        n1.len(),
        n2.len(),
        "grammar name should not affect entry count"
    );
    let types1: Vec<_> = n1.iter().filter_map(|n| n["type"].as_str()).collect();
    let types2: Vec<_> = n2.iter().filter_map(|n| n["type"].as_str()).collect();
    assert_eq!(types1, types2, "grammar name should not affect types");
}

#[test]
fn edge_output_is_sorted_by_type_name() {
    let nodes = gen_parsed(&arithmetic_grammar());
    let types: Vec<_> = nodes
        .iter()
        .filter_map(|n| n["type"].as_str().map(String::from))
        .collect();
    let mut sorted = types.clone();
    sorted.sort();
    assert_eq!(types, sorted, "output should be sorted by type name");
}

#[test]
fn edge_entries_monotonically_increase_with_grammar_size() {
    let counts: Vec<usize> = [1, 3, 7, 15]
        .iter()
        .map(|&n| gen_parsed(&scaled_grammar(&format!("e7_{n}"), n)).len())
        .collect();
    for window in counts.windows(2) {
        assert!(
            window[1] > window[0],
            "entry count should increase: {counts:?}"
        );
    }
}

#[test]
fn edge_different_grammars_produce_different_output() {
    let g1 = GrammarBuilder::new("d1")
        .token("a", "a")
        .rule("r1", vec!["a"])
        .build();
    let g2 = GrammarBuilder::new("d2")
        .token("b", "b")
        .rule("r2", vec!["b"])
        .build();
    let v1: Value = serde_json::from_str(&gen_json(&g1)).unwrap();
    let v2: Value = serde_json::from_str(&gen_json(&g2)).unwrap();
    assert_ne!(v1, v2, "different grammars should produce different output");
}
