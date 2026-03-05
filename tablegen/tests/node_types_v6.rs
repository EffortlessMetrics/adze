//! v6 tests for NODE_TYPES JSON generation in adze-tablegen.
//!
//! Categories:
//!   1. Output is valid JSON array (8 tests)
//!   2. Entries have required 'type' and 'named' fields (8 tests)
//!   3. Token types have named=false for anonymous tokens (8 tests)
//!   4. Rule types have named=true (8 tests)
//!   5. Field information present for rules with fields (8 tests)
//!   6. Subtypes for supertype rules (8 tests)
//!   7. JSON determinism: same grammar → same output (8 tests)
//!   8. Edge cases: empty grammar, many types, special characters (8 tests)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gen_json(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate() failed")
}

fn gen_parsed(grammar: &Grammar) -> Vec<Value> {
    let json = gen_json(grammar);
    let val: Value = serde_json::from_str(&json).expect("not valid JSON");
    val.as_array().expect("not a JSON array").to_vec()
}

fn find_node<'a>(nodes: &'a [Value], type_name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(type_name))
}

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("sum", vec!["expr", "+", "expr"])
        .start("sum")
        .build()
}

fn grammar_with_fields() -> Grammar {
    let mut g = Grammar::new("with_fields".to_string());

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
            name: "op".to_string(),
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

fn scaled_grammar(n: usize) -> Grammar {
    let mut builder = GrammarBuilder::new("scaled");
    for i in 0..n {
        let tok = format!("tok_{i}");
        let rul = format!("rule_{i}");
        builder = builder.token(&tok, &tok);
        builder = builder.rule(&rul, vec![&tok]);
    }
    builder.build()
}

// ===========================================================================
// 1. Output is valid JSON array (8 tests)
// ===========================================================================

#[test]
fn json_single_token_grammar_produces_valid_array() {
    let g = GrammarBuilder::new("j1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .build();
    let val: Value = serde_json::from_str(&gen_json(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_multiple_tokens_produce_valid_array() {
    let g = GrammarBuilder::new("j2")
        .token("x", "x")
        .token("y", "y")
        .rule("pair", vec!["x", "y"])
        .build();
    let val: Value = serde_json::from_str(&gen_json(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_regex_token_grammar_is_valid() {
    let g = GrammarBuilder::new("j3")
        .token("id", r"[a-z]+")
        .rule("start", vec!["id"])
        .build();
    assert!(serde_json::from_str::<Value>(&gen_json(&g)).is_ok());
}

#[test]
fn json_mixed_token_types_produce_valid_array() {
    let g = GrammarBuilder::new("j4")
        .token("NUM", r"\d+")
        .token(";", ";")
        .rule("stmt", vec!["NUM", ";"])
        .build();
    let val: Value = serde_json::from_str(&gen_json(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_grammar_with_fields_is_valid() {
    let g = grammar_with_fields();
    let val: Value = serde_json::from_str(&gen_json(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_empty_grammar_produces_valid_array() {
    let g = Grammar::new("j6".to_string());
    let val: Value = serde_json::from_str(&gen_json(&g)).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_arithmetic_grammar_is_valid() {
    let val: Value = serde_json::from_str(&gen_json(&simple_grammar())).unwrap();
    assert!(val.is_array());
}

#[test]
fn json_scaled_grammar_is_valid() {
    let g = scaled_grammar(20);
    let val: Value = serde_json::from_str(&gen_json(&g)).unwrap();
    assert!(val.is_array());
}

// ===========================================================================
// 2. Entries have required 'type' and 'named' fields (8 tests)
// ===========================================================================

#[test]
fn required_fields_present_single_rule() {
    let g = GrammarBuilder::new("r1")
        .token("a", "a")
        .rule("root", vec!["a"])
        .build();
    for entry in gen_parsed(&g) {
        assert!(entry.get("type").is_some(), "missing 'type'");
        assert!(entry.get("named").is_some(), "missing 'named'");
    }
}

#[test]
fn required_fields_type_is_string() {
    let nodes = gen_parsed(&simple_grammar());
    for entry in &nodes {
        assert!(entry["type"].is_string());
    }
}

#[test]
fn required_fields_named_is_bool() {
    let nodes = gen_parsed(&simple_grammar());
    for entry in &nodes {
        assert!(entry["named"].is_boolean());
    }
}

#[test]
fn required_fields_present_regex_tokens() {
    let g = GrammarBuilder::new("r4")
        .token("word", r"[a-z]+")
        .rule("doc", vec!["word"])
        .build();
    for entry in gen_parsed(&g) {
        assert!(entry.get("type").and_then(Value::as_str).is_some());
        assert!(entry.get("named").and_then(Value::as_bool).is_some());
    }
}

#[test]
fn required_fields_present_multiple_rules() {
    let g = GrammarBuilder::new("r5")
        .token("a", "a")
        .token("b", "b")
        .rule("first", vec!["a"])
        .rule("second", vec!["b"])
        .build();
    for entry in gen_parsed(&g) {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn required_fields_present_fields_grammar() {
    let nodes = gen_parsed(&grammar_with_fields());
    for entry in &nodes {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn required_fields_type_not_empty() {
    let nodes = gen_parsed(&simple_grammar());
    for entry in &nodes {
        let t = entry["type"].as_str().unwrap();
        assert!(!t.is_empty(), "type must not be empty");
    }
}

#[test]
fn required_fields_present_scaled_grammar() {
    let nodes = gen_parsed(&scaled_grammar(10));
    assert!(!nodes.is_empty());
    for entry in &nodes {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

// ===========================================================================
// 3. Token types have named=false for anonymous tokens (8 tests)
// ===========================================================================

#[test]
fn anon_string_token_is_unnamed() {
    let g = GrammarBuilder::new("t1")
        .token("+", "+")
        .rule("start", vec!["+"])
        .build();
    let nodes = gen_parsed(&g);
    let plus = find_node(&nodes, "+");
    assert!(plus.is_some(), "'+' should appear");
    assert_eq!(plus.unwrap()["named"], false);
}

#[test]
fn anon_semicolon_token_is_unnamed() {
    let g = GrammarBuilder::new("t2")
        .token(";", ";")
        .rule("start", vec![";"])
        .build();
    let nodes = gen_parsed(&g);
    let semi = find_node(&nodes, ";");
    assert!(semi.is_some());
    assert_eq!(semi.unwrap()["named"], false);
}

#[test]
fn anon_multiple_operators_all_unnamed() {
    let g = GrammarBuilder::new("t3")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("ops", vec!["+"])
        .build();
    let nodes = gen_parsed(&g);
    for op in ["+", "-", "*"] {
        if let Some(n) = find_node(&nodes, op) {
            assert_eq!(n["named"], false, "{op} should be unnamed");
        }
    }
}

#[test]
fn anon_parentheses_are_unnamed() {
    let mut g = Grammar::new("t4".to_string());
    let lp = SymbolId(0);
    g.tokens.insert(
        lp,
        Token {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );
    let rp = SymbolId(1);
    g.tokens.insert(
        rp,
        Token {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );
    let nodes = gen_parsed(&g);
    assert!(find_node(&nodes, "(").is_some_and(|n| n["named"] == false));
    assert!(find_node(&nodes, ")").is_some_and(|n| n["named"] == false));
}

#[test]
fn anon_keyword_literal_is_unnamed() {
    let mut g = Grammar::new("t5".to_string());
    let kw = SymbolId(0);
    g.tokens.insert(
        kw,
        Token {
            name: "if_kw".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: false,
        },
    );
    let nodes = gen_parsed(&g);
    let entry = find_node(&nodes, "if");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap()["named"], false);
}

#[test]
fn anon_comma_token_is_unnamed() {
    let mut g = Grammar::new("t6".to_string());
    let comma = SymbolId(0);
    g.tokens.insert(
        comma,
        Token {
            name: "comma".to_string(),
            pattern: TokenPattern::String(",".to_string()),
            fragile: false,
        },
    );
    let nodes = gen_parsed(&g);
    let entry = find_node(&nodes, ",");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap()["named"], false);
}

#[test]
fn anon_equals_token_is_unnamed() {
    let g = GrammarBuilder::new("t7")
        .token("=", "=")
        .rule("assign", vec!["="])
        .build();
    let nodes = gen_parsed(&g);
    let eq = find_node(&nodes, "=");
    assert!(eq.is_some());
    assert_eq!(eq.unwrap()["named"], false);
}

#[test]
fn anon_colon_token_is_unnamed() {
    let g = GrammarBuilder::new("t8")
        .token(":", ":")
        .rule("start", vec![":"])
        .build();
    let nodes = gen_parsed(&g);
    let colon = find_node(&nodes, ":");
    assert!(colon.is_some());
    assert_eq!(colon.unwrap()["named"], false);
}

// ===========================================================================
// 4. Rule types have named=true (8 tests)
// ===========================================================================

#[test]
fn rule_single_rule_is_named() {
    let g = GrammarBuilder::new("n1")
        .token("a", "a")
        .rule("root", vec!["a"])
        .build();
    let nodes = gen_parsed(&g);
    let root = find_node(&nodes, "root");
    assert!(root.is_some());
    assert_eq!(root.unwrap()["named"], true);
}

#[test]
fn rule_multiple_rules_all_named() {
    let g = GrammarBuilder::new("n2")
        .token("x", "x")
        .rule("alpha", vec!["x"])
        .rule("beta", vec!["x"])
        .build();
    let nodes = gen_parsed(&g);
    assert_eq!(find_node(&nodes, "alpha").unwrap()["named"], true);
    assert_eq!(find_node(&nodes, "beta").unwrap()["named"], true);
}

#[test]
fn rule_with_regex_token_in_rhs_is_named() {
    let g = GrammarBuilder::new("n3")
        .token("ID", r"[a-z]+")
        .rule("ident_ref", vec!["ID"])
        .build();
    let nodes = gen_parsed(&g);
    assert_eq!(find_node(&nodes, "ident_ref").unwrap()["named"], true);
}

#[test]
fn rule_start_symbol_is_named() {
    let g = GrammarBuilder::new("n4")
        .token("a", "a")
        .rule("program", vec!["a"])
        .start("program")
        .build();
    let nodes = gen_parsed(&g);
    assert_eq!(find_node(&nodes, "program").unwrap()["named"], true);
}

#[test]
fn rule_arithmetic_rules_are_named() {
    let nodes = gen_parsed(&simple_grammar());
    assert_eq!(find_node(&nodes, "expr").unwrap()["named"], true);
    assert_eq!(find_node(&nodes, "sum").unwrap()["named"], true);
}

#[test]
fn rule_with_multiple_alternatives_is_named() {
    let g = GrammarBuilder::new("n6")
        .token("a", "a")
        .token("b", "b")
        .rule("choice", vec!["a"])
        .rule("choice", vec!["b"])
        .build();
    let nodes = gen_parsed(&g);
    assert_eq!(find_node(&nodes, "choice").unwrap()["named"], true);
}

#[test]
fn rule_with_nonterminal_rhs_is_named() {
    let g = GrammarBuilder::new("n7")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("wrapper", vec!["leaf"])
        .build();
    let nodes = gen_parsed(&g);
    assert_eq!(find_node(&nodes, "wrapper").unwrap()["named"], true);
}

#[test]
fn rule_chained_nonterminals_all_named() {
    let g = GrammarBuilder::new("n8")
        .token("v", "v")
        .rule("inner", vec!["v"])
        .rule("middle", vec!["inner"])
        .rule("outer", vec!["middle"])
        .build();
    let nodes = gen_parsed(&g);
    assert_eq!(find_node(&nodes, "inner").unwrap()["named"], true);
    assert_eq!(find_node(&nodes, "middle").unwrap()["named"], true);
    assert_eq!(find_node(&nodes, "outer").unwrap()["named"], true);
}

// ===========================================================================
// 5. Field information present for rules with fields (8 tests)
// ===========================================================================

#[test]
fn field_single_field_appears() {
    let mut g = Grammar::new("f1".to_string());
    let tok = SymbolId(0);
    g.tokens.insert(
        tok,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let expr_id = SymbolId(10);
    g.rule_names.insert(expr_id, "expr".to_string());
    let fid = FieldId(0);
    g.fields.insert(fid, "value".to_string());
    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![(fid, 0)],
        production_id: ProductionId(0),
    });
    let nodes = gen_parsed(&g);
    let entry = find_node(&nodes, "expr").unwrap();
    assert!(entry.get("fields").is_some(), "fields key expected");
    assert!(entry["fields"].get("value").is_some());
}

#[test]
fn field_two_fields_both_appear() {
    let nodes = gen_parsed(&grammar_with_fields());
    let entry = find_node(&nodes, "binary_expr").unwrap();
    let fields = entry.get("fields").unwrap();
    assert!(fields.get("left").is_some());
    assert!(fields.get("right").is_some());
}

#[test]
fn field_entry_has_types_array() {
    let nodes = gen_parsed(&grammar_with_fields());
    let entry = find_node(&nodes, "binary_expr").unwrap();
    let left = &entry["fields"]["left"];
    assert!(left["types"].is_array());
}

#[test]
fn field_entry_has_required_key() {
    let nodes = gen_parsed(&grammar_with_fields());
    let entry = find_node(&nodes, "binary_expr").unwrap();
    let left = &entry["fields"]["left"];
    assert!(left.get("required").is_some());
}

#[test]
fn field_entry_has_multiple_key() {
    let nodes = gen_parsed(&grammar_with_fields());
    let entry = find_node(&nodes, "binary_expr").unwrap();
    let left = &entry["fields"]["left"];
    assert!(left.get("multiple").is_some());
}

#[test]
fn field_absent_when_no_fields_defined() {
    let g = GrammarBuilder::new("f6")
        .token("a", "a")
        .rule("plain", vec!["a"])
        .build();
    let nodes = gen_parsed(&g);
    let entry = find_node(&nodes, "plain").unwrap();
    assert!(entry.get("fields").is_none(), "fields should be absent");
}

#[test]
fn field_three_fields_all_present() {
    let mut g = Grammar::new("f7".to_string());
    let t0 = SymbolId(0);
    let t1 = SymbolId(1);
    let t2 = SymbolId(2);
    for (id, name) in [(t0, "a"), (t1, "b"), (t2, "c")] {
        g.tokens.insert(
            id,
            Token {
                name: name.to_string(),
                pattern: TokenPattern::Regex(name.to_string()),
                fragile: false,
            },
        );
    }
    let rule_id = SymbolId(20);
    g.rule_names.insert(rule_id, "triple".to_string());
    let f0 = FieldId(0);
    let f1 = FieldId(1);
    let f2 = FieldId(2);
    g.fields.insert(f0, "first".to_string());
    g.fields.insert(f1, "second".to_string());
    g.fields.insert(f2, "third".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![
            Symbol::Terminal(t0),
            Symbol::Terminal(t1),
            Symbol::Terminal(t2),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(f0, 0), (f1, 1), (f2, 2)],
        production_id: ProductionId(0),
    });
    let nodes = gen_parsed(&g);
    let entry = find_node(&nodes, "triple").unwrap();
    let fields = entry.get("fields").unwrap();
    assert!(fields.get("first").is_some());
    assert!(fields.get("second").is_some());
    assert!(fields.get("third").is_some());
}

#[test]
fn field_types_entry_contains_type_and_named() {
    let nodes = gen_parsed(&grammar_with_fields());
    let entry = find_node(&nodes, "binary_expr").unwrap();
    let types = entry["fields"]["left"]["types"].as_array().unwrap();
    assert!(!types.is_empty());
    let first = &types[0];
    assert!(first.get("type").is_some());
    assert!(first.get("named").is_some());
}

// ===========================================================================
// 6. Subtypes for supertype rules (8 tests)
// ===========================================================================

#[test]
fn supertype_marker_present_in_grammar() {
    let g = GrammarBuilder::new("s1")
        .token("a", "a")
        .rule("stmt", vec!["a"])
        .supertype("stmt")
        .build();
    assert!(!g.supertypes.is_empty());
}

#[test]
fn supertype_rule_still_named() {
    let g = GrammarBuilder::new("s2")
        .token("a", "a")
        .rule("stmt", vec!["a"])
        .supertype("stmt")
        .build();
    let nodes = gen_parsed(&g);
    let entry = find_node(&nodes, "stmt");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap()["named"], true);
}

#[test]
fn supertype_with_alternatives_still_appears() {
    let g = GrammarBuilder::new("s3")
        .token("x", "x")
        .token("y", "y")
        .rule("node_kind", vec!["x"])
        .rule("node_kind", vec!["y"])
        .supertype("node_kind")
        .build();
    let nodes = gen_parsed(&g);
    assert!(find_node(&nodes, "node_kind").is_some());
}

#[test]
fn supertype_grammar_output_is_valid_json() {
    let g = GrammarBuilder::new("s4")
        .token("a", "a")
        .rule("base", vec!["a"])
        .supertype("base")
        .build();
    assert!(serde_json::from_str::<Value>(&gen_json(&g)).is_ok());
}

#[test]
fn supertype_multiple_supertypes_both_present() {
    let g = GrammarBuilder::new("s5")
        .token("a", "a")
        .token("b", "b")
        .rule("type_a", vec!["a"])
        .rule("type_b", vec!["b"])
        .supertype("type_a")
        .supertype("type_b")
        .build();
    let nodes = gen_parsed(&g);
    assert!(find_node(&nodes, "type_a").is_some());
    assert!(find_node(&nodes, "type_b").is_some());
}

#[test]
fn supertype_does_not_suppress_named() {
    let g = GrammarBuilder::new("s6")
        .token("v", "v")
        .rule("declaration", vec!["v"])
        .supertype("declaration")
        .build();
    let nodes = gen_parsed(&g);
    let entry = find_node(&nodes, "declaration").unwrap();
    assert_eq!(entry["named"], true);
}

#[test]
fn supertype_grammar_entries_have_type_field() {
    let g = GrammarBuilder::new("s7")
        .token("k", "k")
        .rule("kind", vec!["k"])
        .supertype("kind")
        .build();
    let nodes = gen_parsed(&g);
    for entry in &nodes {
        assert!(entry.get("type").is_some());
    }
}

#[test]
fn supertype_combined_with_plain_rules() {
    let g = GrammarBuilder::new("s8")
        .token("a", "a")
        .token("b", "b")
        .rule("super_rule", vec!["a"])
        .rule("plain_rule", vec!["b"])
        .supertype("super_rule")
        .build();
    let nodes = gen_parsed(&g);
    assert!(find_node(&nodes, "super_rule").is_some());
    assert!(find_node(&nodes, "plain_rule").is_some());
}

// ===========================================================================
// 7. JSON determinism: same grammar → same output (8 tests)
// ===========================================================================

#[test]
fn determinism_simple_grammar_twice() {
    let g = simple_grammar();
    assert_eq!(gen_json(&g), gen_json(&g));
}

#[test]
fn determinism_fields_grammar_twice() {
    let a = gen_json(&grammar_with_fields());
    let b = gen_json(&grammar_with_fields());
    assert_eq!(a, b);
}

#[test]
fn determinism_empty_grammar_twice() {
    let g = Grammar::new("det3".to_string());
    assert_eq!(gen_json(&g), gen_json(&g));
}

#[test]
fn determinism_scaled_grammar_twice() {
    let g = scaled_grammar(15);
    assert_eq!(gen_json(&g), gen_json(&g));
}

#[test]
fn determinism_ten_invocations() {
    let g = simple_grammar();
    let first = gen_json(&g);
    for _ in 0..10 {
        assert_eq!(gen_json(&g), first);
    }
}

#[test]
fn determinism_regex_token_grammar() {
    let g = GrammarBuilder::new("det6")
        .token("ID", r"[a-z_]+")
        .token("NUM", r"\d+")
        .rule("item", vec!["ID"])
        .rule("count", vec!["NUM"])
        .build();
    assert_eq!(gen_json(&g), gen_json(&g));
}

#[test]
fn determinism_supertype_grammar() {
    let g = GrammarBuilder::new("det7")
        .token("a", "a")
        .rule("base", vec!["a"])
        .supertype("base")
        .build();
    assert_eq!(gen_json(&g), gen_json(&g));
}

#[test]
fn determinism_parsed_values_match() {
    let g = simple_grammar();
    let a = gen_parsed(&g);
    let b = gen_parsed(&g);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x, y);
    }
}

// ===========================================================================
// 8. Edge cases: empty grammar, many types, special characters (8 tests)
// ===========================================================================

#[test]
fn edge_empty_grammar_produces_empty_array() {
    let g = Grammar::new("edge1".to_string());
    let nodes = gen_parsed(&g);
    assert!(nodes.is_empty());
}

#[test]
fn edge_single_anonymous_token_only() {
    let mut g = Grammar::new("edge2".to_string());
    let t = SymbolId(0);
    g.tokens.insert(
        t,
        Token {
            name: "dot".to_string(),
            pattern: TokenPattern::String(".".to_string()),
            fragile: false,
        },
    );
    let nodes = gen_parsed(&g);
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["named"], false);
}

#[test]
fn edge_many_types_all_present() {
    let g = scaled_grammar(50);
    let nodes = gen_parsed(&g);
    // Each rule generates a named entry; string tokens generate anonymous entries
    assert!(
        nodes.len() >= 50,
        "expected >=50 entries, got {}",
        nodes.len()
    );
}

#[test]
fn edge_internal_rule_excluded() {
    let mut g = Grammar::new("edge4".to_string());
    let tok = SymbolId(0);
    g.tokens.insert(
        tok,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::Regex("a".to_string()),
            fragile: false,
        },
    );
    let hidden = SymbolId(10);
    g.rule_names.insert(hidden, "_hidden".to_string());
    g.add_rule(Rule {
        lhs: hidden,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let nodes = gen_parsed(&g);
    assert!(
        find_node(&nodes, "_hidden").is_none(),
        "internal rules excluded"
    );
}

#[test]
fn edge_special_char_token_in_output() {
    let mut g = Grammar::new("edge5".to_string());
    let t = SymbolId(0);
    g.tokens.insert(
        t,
        Token {
            name: "arrow".to_string(),
            pattern: TokenPattern::String("->".to_string()),
            fragile: false,
        },
    );
    let nodes = gen_parsed(&g);
    assert!(find_node(&nodes, "->").is_some());
}

#[test]
fn edge_output_is_sorted_alphabetically() {
    let nodes = gen_parsed(&simple_grammar());
    let types: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = types.clone();
    sorted.sort();
    assert_eq!(types, sorted);
}

#[test]
fn edge_no_null_values_in_output() {
    let nodes = gen_parsed(&simple_grammar());
    for entry in &nodes {
        assert!(!entry["type"].is_null());
        assert!(!entry["named"].is_null());
    }
}

#[test]
fn edge_unicode_token_preserved() {
    let mut g = Grammar::new("edge8".to_string());
    let t = SymbolId(0);
    g.tokens.insert(
        t,
        Token {
            name: "lambda".to_string(),
            pattern: TokenPattern::String("\u{03bb}".to_string()),
            fragile: false,
        },
    );
    let json = gen_json(&g);
    assert!(json.contains('\u{03bb}') || json.contains("\\u03bb") || json.contains("\\u{3bb}"));
}
