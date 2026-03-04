//! Comprehensive NODE_TYPES JSON generation tests for adze-tablegen.
//!
//! 80+ tests covering: JSON structure, named/anonymous nodes, fields, children,
//! supertypes, empty grammars, complex grammars, round-trip, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ===========================================================================
// Helpers
// ===========================================================================

fn generate_json(g: &Grammar) -> String {
    NodeTypesGenerator::new(g).generate().expect("generate ok")
}

fn parse(json: &str) -> Vec<Value> {
    serde_json::from_str(json).expect("valid JSON array")
}

fn gen_parsed(g: &Grammar) -> Vec<Value> {
    parse(&generate_json(g))
}

fn find_node<'a>(arr: &'a [Value], type_name: &str) -> Option<&'a Value> {
    arr.iter().find(|n| n["type"].as_str() == Some(type_name))
}

fn named_types(arr: &[Value]) -> Vec<&str> {
    arr.iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect()
}

fn anon_types(arr: &[Value]) -> Vec<&str> {
    arr.iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect()
}

/// Build a grammar with fields directly (GrammarBuilder doesn't expose field API).
fn grammar_with_fields() -> Grammar {
    let mut g = Grammar::new("fields".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let plus_id = SymbolId(1);
    g.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let left_fid = FieldId(0);
    let op_fid = FieldId(1);
    let right_fid = FieldId(2);
    g.fields.insert(left_fid, "left".to_string());
    g.fields.insert(op_fid, "operator".to_string());
    g.fields.insert(right_fid, "right".to_string());

    let bin_id = SymbolId(10);
    g.rule_names.insert(bin_id, "binary_expression".to_string());
    g.add_rule(Rule {
        lhs: bin_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(plus_id),
            Symbol::Terminal(num_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(left_fid, 0), (op_fid, 1), (right_fid, 2)],
        production_id: ProductionId(0),
    });

    g
}

// ===========================================================================
// 1. Empty grammar → empty node types
// ===========================================================================

#[test]
fn empty_grammar_produces_valid_json() {
    let arr = gen_parsed(&Grammar::new("empty".to_string()));
    assert!(arr.is_empty());
}

#[test]
fn empty_grammar_top_level_is_array() {
    let json = generate_json(&Grammar::new("empty".to_string()));
    let val: Value = serde_json::from_str(&json).unwrap();
    assert!(val.is_array());
}

#[test]
fn empty_grammar_json_is_brackets() {
    let json = generate_json(&Grammar::new("empty".to_string()));
    let trimmed = json.trim();
    assert!(trimmed.starts_with('['));
    assert!(trimmed.ends_with(']'));
}

// ===========================================================================
// 2. JSON structure validation
// ===========================================================================

#[test]
fn output_is_valid_json() {
    let g = GrammarBuilder::new("basic")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    assert!(serde_json::from_str::<Value>(&generate_json(&g)).is_ok());
}

#[test]
fn output_is_array_of_objects() {
    let g = GrammarBuilder::new("a")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .build();
    for node in gen_parsed(&g) {
        assert!(node.is_object(), "each entry must be an object");
    }
}

#[test]
fn every_entry_has_type_field() {
    let g = GrammarBuilder::new("a")
        .token("ID", r"[a-z]+")
        .token("+", "+")
        .rule("stmt", vec!["ID"])
        .build();
    for node in gen_parsed(&g) {
        assert!(node["type"].is_string(), "missing 'type': {node}");
    }
}

#[test]
fn every_entry_has_named_field() {
    let g = GrammarBuilder::new("a")
        .token("ID", r"[a-z]+")
        .token("+", "+")
        .rule("stmt", vec!["ID"])
        .build();
    for node in gen_parsed(&g) {
        assert!(node["named"].is_boolean(), "missing 'named': {node}");
    }
}

#[test]
fn type_field_is_never_empty() {
    let g = GrammarBuilder::new("a")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    for node in gen_parsed(&g) {
        assert!(!node["type"].as_str().unwrap().is_empty());
    }
}

#[test]
fn json_is_pretty_printed() {
    let g = GrammarBuilder::new("a")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    assert!(generate_json(&g).contains('\n'));
}

#[test]
fn no_null_type_names() {
    let g = grammar_with_fields();
    for node in gen_parsed(&g) {
        assert!(!node["type"].is_null());
    }
}

// ===========================================================================
// 3. Named vs anonymous node types
// ===========================================================================

#[test]
fn rule_node_is_named() {
    let g = GrammarBuilder::new("a")
        .token("NUM", r"\d+")
        .rule("expression", vec!["NUM"])
        .build();
    let arr = gen_parsed(&g);
    let expr = find_node(&arr, "expression").expect("expression present");
    assert_eq!(expr["named"], true);
}

#[test]
fn string_literal_token_is_anonymous() {
    let mut g = Grammar::new("anon".to_string());
    let plus_id = SymbolId(0);
    g.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let arr = gen_parsed(&g);
    let plus = find_node(&arr, "+").expect("'+' present");
    assert_eq!(plus["named"], false);
}

#[test]
fn regex_token_not_emitted_as_anonymous() {
    let mut g = Grammar::new("reg".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let arr = gen_parsed(&g);
    let anon = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("number") && n["named"] == false);
    assert!(anon.is_none(), "regex token should not appear anonymous");
}

#[test]
fn anonymous_nodes_have_no_fields() {
    let g = grammar_with_fields();
    for node in gen_parsed(&g) {
        if node["named"] == false {
            assert!(
                node.get("fields").is_none() || node["fields"].is_null(),
                "anonymous '{}' should lack fields",
                node["type"]
            );
        }
    }
}

#[test]
fn anonymous_nodes_have_no_children() {
    let g = grammar_with_fields();
    for node in gen_parsed(&g) {
        if node["named"] == false {
            assert!(
                node.get("children").is_none() || node["children"].is_null(),
                "anonymous '{}' should lack children",
                node["type"]
            );
        }
    }
}

#[test]
fn multiple_anonymous_tokens() {
    let mut g = Grammar::new("multi_anon".to_string());
    for (i, lit) in ["+", "-", "*", "/", "(", ")", ";", ","].iter().enumerate() {
        g.tokens.insert(
            SymbolId(i as u16),
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(lit.to_string()),
                fragile: false,
            },
        );
    }
    let arr = gen_parsed(&g);
    let anon = anon_types(&arr);
    for lit in ["+", "-", "*", "/", "(", ")", ";", ","] {
        assert!(anon.contains(&lit), "missing anonymous '{lit}'");
    }
}

#[test]
fn named_and_anonymous_coexist() {
    let g = GrammarBuilder::new("mix")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("sum", vec!["NUM", "+", "NUM"])
        .build();
    let arr = gen_parsed(&g);
    assert!(!named_types(&arr).is_empty());
    assert!(!anon_types(&arr).is_empty());
}

// ===========================================================================
// 4. Field definitions in node types
// ===========================================================================

#[test]
fn fields_appear_in_output() {
    let arr = gen_parsed(&grammar_with_fields());
    let bin = find_node(&arr, "binary_expression").unwrap();
    let fields = bin.get("fields").expect("fields present");
    assert!(fields.get("left").is_some());
    assert!(fields.get("operator").is_some());
    assert!(fields.get("right").is_some());
}

#[test]
fn field_has_types_array() {
    let arr = gen_parsed(&grammar_with_fields());
    let bin = find_node(&arr, "binary_expression").unwrap();
    for name in ["left", "operator", "right"] {
        let f = &bin["fields"][name];
        assert!(
            f["types"].is_array(),
            "field '{name}' must have types array"
        );
        assert!(!f["types"].as_array().unwrap().is_empty());
    }
}

#[test]
fn field_has_required_flag() {
    let arr = gen_parsed(&grammar_with_fields());
    let bin = find_node(&arr, "binary_expression").unwrap();
    for name in ["left", "operator", "right"] {
        assert!(
            bin["fields"][name].get("required").is_some(),
            "'{name}' missing required"
        );
    }
}

#[test]
fn field_has_multiple_flag() {
    let arr = gen_parsed(&grammar_with_fields());
    let bin = find_node(&arr, "binary_expression").unwrap();
    for name in ["left", "operator", "right"] {
        assert!(
            bin["fields"][name].get("multiple").is_some(),
            "'{name}' missing multiple"
        );
    }
}

#[test]
fn field_type_ref_has_type_and_named() {
    let arr = gen_parsed(&grammar_with_fields());
    let bin = find_node(&arr, "binary_expression").unwrap();
    let left_types = bin["fields"]["left"]["types"].as_array().unwrap();
    for tr in left_types {
        assert!(tr["type"].is_string());
        assert!(tr["named"].is_boolean());
    }
}

#[test]
fn node_without_fields_omits_fields_key() {
    let g = GrammarBuilder::new("nf")
        .token("X", r"x")
        .rule("thing", vec!["X"])
        .build();
    let arr = gen_parsed(&g);
    let thing = find_node(&arr, "thing").unwrap();
    assert!(
        thing.get("fields").is_none() || thing["fields"].is_null(),
        "no-field node should omit fields"
    );
}

#[test]
fn single_field_grammar() {
    let mut g = Grammar::new("one_field".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let fid = FieldId(0);
    g.fields.insert(fid, "value".to_string());
    let rule_id = SymbolId(10);
    g.rule_names.insert(rule_id, "wrapper".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![(fid, 0)],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    let w = find_node(&arr, "wrapper").unwrap();
    assert!(w["fields"]["value"].is_object());
}

// ===========================================================================
// 5. Children specifications
// ===========================================================================

#[test]
fn children_key_absent_when_not_applicable() {
    let g = GrammarBuilder::new("nochild")
        .token("X", r"x")
        .rule("leaf", vec!["X"])
        .build();
    let arr = gen_parsed(&g);
    let leaf = find_node(&arr, "leaf").unwrap();
    assert!(
        leaf.get("children").is_none() || leaf["children"].is_null(),
        "children should be absent"
    );
}

// ===========================================================================
// 6. Supertype nodes
// ===========================================================================

#[test]
fn supertype_grammar_both_nodes_present() {
    let mut g = Grammar::new("super".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let lit_id = SymbolId(10);
    g.rule_names.insert(lit_id, "literal".to_string());
    g.add_rule(Rule {
        lhs: lit_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let expr_id = SymbolId(11);
    g.rule_names.insert(expr_id, "expression".to_string());
    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(lit_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.supertypes.push(expr_id);

    let arr = gen_parsed(&g);
    let names = named_types(&arr);
    assert!(names.contains(&"expression"));
    assert!(names.contains(&"literal"));
}

#[test]
fn supertype_declared_in_grammar_ir() {
    let mut g = Grammar::new("s".to_string());
    let id = SymbolId(1);
    g.supertypes.push(id);
    assert!(!g.supertypes.is_empty());
}

// ===========================================================================
// 7. Sorting
// ===========================================================================

#[test]
fn output_sorted_alphabetically() {
    let g = GrammarBuilder::new("sorted")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("zebra", vec!["NUM"])
        .rule("alpha", vec!["NUM"])
        .rule("middle", vec!["NUM"])
        .build();
    let arr = gen_parsed(&g);
    let names: Vec<&str> = arr.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn sorting_with_anonymous_and_named() {
    let g = GrammarBuilder::new("mix")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .token(":", ":")
        .rule("decl", vec!["ID"])
        .build();
    let arr = gen_parsed(&g);
    let names: Vec<&str> = arr.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

// ===========================================================================
// 8. Internal rules excluded
// ===========================================================================

#[test]
fn internal_rule_excluded_from_output() {
    let mut g = Grammar::new("internal".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let hidden_id = SymbolId(10);
    g.rule_names.insert(hidden_id, "_hidden".to_string());
    g.add_rule(Rule {
        lhs: hidden_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(
        !arr.iter()
            .any(|n| n["type"].as_str().is_some_and(|s| s.starts_with('_')))
    );
}

#[test]
fn public_rule_alongside_internal_rule() {
    let mut g = Grammar::new("mixed".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let hidden_id = SymbolId(10);
    g.rule_names.insert(hidden_id, "_hidden".to_string());
    g.add_rule(Rule {
        lhs: hidden_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let pub_id = SymbolId(11);
    g.rule_names.insert(pub_id, "visible".to_string());
    g.add_rule(Rule {
        lhs: pub_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "visible").is_some());
    assert!(find_node(&arr, "_hidden").is_none());
}

// ===========================================================================
// 9. Round-trip: generate → parse → validate
// ===========================================================================

#[test]
fn roundtrip_empty_grammar() {
    let json = generate_json(&Grammar::new("rt".to_string()));
    let reparsed: Vec<Value> = serde_json::from_str(&json).unwrap();
    assert!(reparsed.is_empty());
}

#[test]
fn roundtrip_single_rule() {
    let g = GrammarBuilder::new("rt")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    let json = generate_json(&g);
    let reparsed: Vec<Value> = serde_json::from_str(&json).unwrap();
    assert!(!reparsed.is_empty());
    for n in &reparsed {
        assert!(n["type"].is_string());
        assert!(n["named"].is_boolean());
    }
}

#[test]
fn roundtrip_fields_grammar() {
    let g = grammar_with_fields();
    let json = generate_json(&g);
    let reparsed: Vec<Value> = serde_json::from_str(&json).unwrap();
    let bin = find_node(&reparsed, "binary_expression").unwrap();
    assert!(bin["fields"].is_object());
}

#[test]
fn roundtrip_to_typed_struct() {
    // Deserialize into typed serde structures, not just Value
    #[derive(serde::Deserialize)]
    struct NodeType {
        #[serde(rename = "type")]
        type_name: String,
        named: bool,
    }
    let g = GrammarBuilder::new("typed")
        .token("A", r"a")
        .rule("foo", vec!["A"])
        .build();
    let json = generate_json(&g);
    let nodes: Vec<NodeType> = serde_json::from_str(&json).unwrap();
    assert!(nodes.iter().any(|n| n.type_name == "foo" && n.named));
}

#[test]
fn roundtrip_reserialize_matches() {
    let g = GrammarBuilder::new("reser")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("sum", vec!["N", "+", "N"])
        .build();
    let json = generate_json(&g);
    let val: Value = serde_json::from_str(&json).unwrap();
    let reserialized = serde_json::to_string_pretty(&val).unwrap();
    let val2: Value = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(val, val2);
}

// ===========================================================================
// 10. Complex grammars → rich node types
// ===========================================================================

#[test]
fn python_like_grammar_generates() {
    let g = GrammarBuilder::python_like();
    let arr = gen_parsed(&g);
    assert!(!arr.is_empty());
}

#[test]
fn python_like_has_module_node() {
    let arr = gen_parsed(&GrammarBuilder::python_like());
    assert!(find_node(&arr, "module").is_some());
}

#[test]
fn python_like_has_function_def() {
    let arr = gen_parsed(&GrammarBuilder::python_like());
    assert!(find_node(&arr, "function_def").is_some());
}

#[test]
fn python_like_has_statement() {
    let arr = gen_parsed(&GrammarBuilder::python_like());
    assert!(find_node(&arr, "statement").is_some());
}

#[test]
fn javascript_like_grammar_generates() {
    let g = GrammarBuilder::javascript_like();
    let arr = gen_parsed(&g);
    assert!(!arr.is_empty());
}

#[test]
fn javascript_like_has_program() {
    let arr = gen_parsed(&GrammarBuilder::javascript_like());
    assert!(find_node(&arr, "program").is_some());
}

#[test]
fn javascript_like_has_expression() {
    let arr = gen_parsed(&GrammarBuilder::javascript_like());
    assert!(find_node(&arr, "expression").is_some());
}

#[test]
fn javascript_like_has_var_declaration() {
    let arr = gen_parsed(&GrammarBuilder::javascript_like());
    assert!(find_node(&arr, "var_declaration").is_some());
}

#[test]
fn javascript_like_has_anonymous_operators() {
    let arr = gen_parsed(&GrammarBuilder::javascript_like());
    let anon = anon_types(&arr);
    assert!(anon.contains(&";"));
}

#[test]
fn complex_grammar_many_rules() {
    let g = GrammarBuilder::new("big")
        .token("A", r"a")
        .token("B", r"b")
        .token("C", r"c")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("r1", vec!["A"])
        .rule("r2", vec!["B"])
        .rule("r3", vec!["C"])
        .rule("r4", vec!["A", "+", "B"])
        .rule("r5", vec!["B", "-", "C"])
        .rule("r6", vec!["A", "*", "C"])
        .build();
    let arr = gen_parsed(&g);
    let names = named_types(&arr);
    for r in ["r1", "r2", "r3", "r4", "r5", "r6"] {
        assert!(names.contains(&r), "missing rule '{r}'");
    }
}

// ===========================================================================
// 11. Determinism
// ===========================================================================

#[test]
fn deterministic_output_simple() {
    let g = GrammarBuilder::new("det")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    let a = generate_json(&g);
    let b = generate_json(&g);
    assert_eq!(a, b);
}

#[test]
fn deterministic_output_complex() {
    let g = grammar_with_fields();
    let a: Value = serde_json::from_str(&generate_json(&g)).unwrap();
    let b: Value = serde_json::from_str(&generate_json(&g)).unwrap();
    assert_eq!(a, b);
}

// ===========================================================================
// 12. Different grammars → different output
// ===========================================================================

#[test]
fn different_grammars_different_output() {
    let g1 = GrammarBuilder::new("a")
        .token("X", r"x")
        .rule("alpha", vec!["X"])
        .build();
    let g2 = GrammarBuilder::new("b")
        .token("Y", r"y")
        .rule("beta", vec!["Y"])
        .build();
    assert_ne!(generate_json(&g1), generate_json(&g2));
}

#[test]
fn adding_rule_changes_output() {
    let g1 = GrammarBuilder::new("a")
        .token("X", r"x")
        .rule("r1", vec!["X"])
        .build();
    let g2 = GrammarBuilder::new("a")
        .token("X", r"x")
        .rule("r1", vec!["X"])
        .rule("r2", vec!["X"])
        .build();
    assert_ne!(generate_json(&g1), generate_json(&g2));
}

#[test]
fn adding_token_changes_output() {
    let g1 = GrammarBuilder::new("a")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    let mut g2 = GrammarBuilder::new("a")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    g2.tokens.insert(
        SymbolId(99),
        Token {
            name: "dot".to_string(),
            pattern: TokenPattern::String(".".to_string()),
            fragile: false,
        },
    );
    assert_ne!(generate_json(&g1), generate_json(&g2));
}

// ===========================================================================
// 13. Symbol variants in rules
// ===========================================================================

#[test]
fn optional_symbol_generates() {
    let mut g = Grammar::new("opt".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "maybe_num".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(num_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "maybe_num").is_some());
}

#[test]
fn repeat_symbol_generates() {
    let mut g = Grammar::new("rep".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "nums".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(num_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "nums").is_some());
}

#[test]
fn repeat_one_symbol_generates() {
    let mut g = Grammar::new("rep1".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "nonempty".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(num_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "nonempty").is_some());
}

#[test]
fn choice_symbol_generates() {
    let mut g = Grammar::new("ch".to_string());
    let a_id = SymbolId(0);
    g.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::Regex(r"a".to_string()),
            fragile: false,
        },
    );
    let b_id = SymbolId(1);
    g.tokens.insert(
        b_id,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::Regex(r"b".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "either".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "either").is_some());
}

#[test]
fn sequence_symbol_generates() {
    let mut g = Grammar::new("seq".to_string());
    let a_id = SymbolId(0);
    g.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::Regex(r"a".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "seq_rule".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(a_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "seq_rule").is_some());
}

#[test]
fn epsilon_rule_generates() {
    let mut g = Grammar::new("eps".to_string());
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "empty_rule".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "empty_rule").is_some());
}

#[test]
fn external_symbol_in_rule() {
    let mut g = Grammar::new("ext".to_string());
    let ext_id = SymbolId(50);
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: ext_id,
    });
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "block".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::External(ext_id), Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "block").is_some());
}

// ===========================================================================
// 14. Edge cases: special characters in names
// ===========================================================================

#[test]
fn rule_name_with_underscores() {
    let g = GrammarBuilder::new("u")
        .token("X", r"x")
        .rule("my_special_rule", vec!["X"])
        .build();
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "my_special_rule").is_some());
}

#[test]
fn rule_name_with_digits() {
    let mut g = Grammar::new("d".to_string());
    let x_id = SymbolId(0);
    g.tokens.insert(
        x_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::Regex(r"x".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "rule42".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Terminal(x_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "rule42").is_some());
}

#[test]
fn anonymous_token_special_chars() {
    let mut g = Grammar::new("spec".to_string());
    for (i, lit) in ["->", "=>", "::", "..."].iter().enumerate() {
        g.tokens.insert(
            SymbolId(i as u16),
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(lit.to_string()),
                fragile: false,
            },
        );
    }
    let arr = gen_parsed(&g);
    let anon = anon_types(&arr);
    for lit in ["->", "=>", "::", "..."] {
        assert!(anon.contains(&lit), "missing '{lit}'");
    }
}

#[test]
fn single_char_rule_name() {
    let mut g = Grammar::new("sc".to_string());
    let x_id = SymbolId(0);
    g.tokens.insert(
        x_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::Regex(r"x".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "a".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Terminal(x_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "a").is_some());
}

#[test]
fn long_rule_name() {
    let long_name = "a".repeat(200);
    let mut g = Grammar::new("long".to_string());
    let x_id = SymbolId(0);
    g.tokens.insert(
        x_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::Regex(r"x".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, long_name.clone());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Terminal(x_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, &long_name).is_some());
}

// ===========================================================================
// 15. Many fields
// ===========================================================================

#[test]
fn many_fields_on_one_rule() {
    let mut g = Grammar::new("manyf".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let field_count = 10;
    let mut rhs = Vec::new();
    let mut fields = Vec::new();
    for i in 0..field_count {
        let fid = FieldId(i);
        g.fields.insert(fid, format!("field_{i}"));
        rhs.push(Symbol::Terminal(num_id));
        fields.push((fid, i as usize));
    }

    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "wide_node".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs,
        precedence: None,
        associativity: None,
        fields,
        production_id: ProductionId(0),
    });

    let arr = gen_parsed(&g);
    let node = find_node(&arr, "wide_node").unwrap();
    let f = node["fields"].as_object().unwrap();
    assert_eq!(f.len(), field_count as usize);
    for i in 0..field_count {
        assert!(f.contains_key(&format!("field_{i}")));
    }
}

// ===========================================================================
// 16. Multiple rules with same LHS (alternatives)
// ===========================================================================

#[test]
fn multiple_alternatives_single_node() {
    let g = GrammarBuilder::new("alt")
        .token("A", r"a")
        .token("B", r"b")
        .rule("thing", vec!["A"])
        .rule("thing", vec!["B"])
        .build();
    let arr = gen_parsed(&g);
    let count = arr
        .iter()
        .filter(|n| n["type"].as_str() == Some("thing") && n["named"] == true)
        .count();
    assert_eq!(count, 1, "'thing' should appear exactly once");
}

// ===========================================================================
// 17. Fragile tokens
// ===========================================================================

#[test]
fn fragile_token_appears_as_anonymous() {
    let mut g = Grammar::new("fragile".to_string());
    let f_id = SymbolId(0);
    g.tokens.insert(
        f_id,
        Token {
            name: "semi".to_string(),
            pattern: TokenPattern::String(";".to_string()),
            fragile: true,
        },
    );
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, ";").is_some());
}

// ===========================================================================
// 18. Non-terminal references
// ===========================================================================

#[test]
fn non_terminal_reference_in_rule() {
    let g = GrammarBuilder::new("nt")
        .token("NUM", r"\d+")
        .rule("atom", vec!["NUM"])
        .rule("expr", vec!["atom"])
        .build();
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "atom").is_some());
    assert!(find_node(&arr, "expr").is_some());
}

// ===========================================================================
// 19. generate() returns Ok for all well-formed grammars
// ===========================================================================

#[test]
fn generate_ok_for_all_builder_grammars() {
    let grammars = vec![
        GrammarBuilder::new("a")
            .token("X", r"x")
            .rule("r", vec!["X"])
            .build(),
        GrammarBuilder::python_like(),
        GrammarBuilder::javascript_like(),
    ];
    for g in &grammars {
        assert!(
            NodeTypesGenerator::new(g).generate().is_ok(),
            "failed for {}",
            g.name
        );
    }
}

// ===========================================================================
// 20. Grammar with only tokens (no rules)
// ===========================================================================

#[test]
fn tokens_only_grammar() {
    let mut g = Grammar::new("tok_only".to_string());
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "minus".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );
    let arr = gen_parsed(&g);
    assert_eq!(anon_types(&arr).len(), 2);
    assert!(named_types(&arr).is_empty());
}

// ===========================================================================
// 21. Grammar with only rules (no tokens)
// ===========================================================================

#[test]
fn rules_only_grammar() {
    let mut g = Grammar::new("rules_only".to_string());
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "empty_rule".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "empty_rule").is_some());
    assert!(anon_types(&arr).is_empty());
}

// ===========================================================================
// 22. Deeply nested symbol types
// ===========================================================================

#[test]
fn nested_optional_repeat() {
    let mut g = Grammar::new("nest".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "deep".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
            Symbol::Terminal(num_id),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "deep").is_some());
}

#[test]
fn nested_choice_in_repeat() {
    let mut g = Grammar::new("cr".to_string());
    let a_id = SymbolId(0);
    g.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::Regex(r"a".to_string()),
            fragile: false,
        },
    );
    let b_id = SymbolId(1);
    g.tokens.insert(
        b_id,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::Regex(r"b".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "list".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "list").is_some());
}

#[test]
fn triple_nested_symbols() {
    let mut g = Grammar::new("triple".to_string());
    let x_id = SymbolId(0);
    g.tokens.insert(
        x_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::Regex(r"x".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "deep3".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::RepeatOne(Box::new(
            Symbol::Choice(vec![Symbol::Terminal(x_id)]),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "deep3").is_some());
}

// ===========================================================================
// 23. Fallback rule name
// ===========================================================================

#[test]
fn rule_without_explicit_name_gets_fallback() {
    let mut g = Grammar::new("fallback".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    // Add a rule without inserting into rule_names
    let r_id = SymbolId(10);
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    // Should get a fallback name like "rule_10"
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str().is_some_and(|s| s.starts_with("rule_"))),
        "expected fallback name"
    );
}

// ===========================================================================
// 24. Empty choice
// ===========================================================================

#[test]
fn empty_choice_symbol() {
    let mut g = Grammar::new("ec".to_string());
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "empty_choice".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Choice(vec![])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // Should not crash
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "empty_choice").is_some());
}

// ===========================================================================
// 25. Empty sequence
// ===========================================================================

#[test]
fn empty_sequence_symbol() {
    let mut g = Grammar::new("es".to_string());
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "empty_seq".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Sequence(vec![])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "empty_seq").is_some());
}

// ===========================================================================
// 26. Unicode in token literals
// ===========================================================================

#[test]
fn unicode_token_literal() {
    let mut g = Grammar::new("uni".to_string());
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "arrow".to_string(),
            pattern: TokenPattern::String("→".to_string()),
            fragile: false,
        },
    );
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "→").is_some());
}

#[test]
fn unicode_rule_name() {
    let mut g = Grammar::new("uni2".to_string());
    let x_id = SymbolId(0);
    g.tokens.insert(
        x_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::Regex(r"x".to_string()),
            fragile: false,
        },
    );
    let r_id = SymbolId(10);
    g.rule_names.insert(r_id, "règle".to_string());
    g.add_rule(Rule {
        lhs: r_id,
        rhs: vec![Symbol::Terminal(x_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "règle").is_some());
}

// ===========================================================================
// 27. Large grammar stress
// ===========================================================================

#[test]
fn large_grammar_50_rules() {
    let mut g = Grammar::new("large".to_string());
    let tok_id = SymbolId(0);
    g.tokens.insert(
        tok_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::Regex(r"x".to_string()),
            fragile: false,
        },
    );
    for i in 0u16..50 {
        let r_id = SymbolId(100 + i);
        let name = format!("rule_{i:03}");
        g.rule_names.insert(r_id, name);
        g.add_rule(Rule {
            lhs: r_id,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        });
    }
    let arr = gen_parsed(&g);
    let named = named_types(&arr);
    assert!(named.len() >= 50);
}

#[test]
fn large_grammar_50_anonymous_tokens() {
    let mut g = Grammar::new("many_tok".to_string());
    for i in 0u16..50 {
        g.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }
    let arr = gen_parsed(&g);
    assert!(anon_types(&arr).len() >= 50);
}

// ===========================================================================
// 28. Fields with non-terminal type references
// ===========================================================================

#[test]
fn field_referencing_non_terminal() {
    let mut g = Grammar::new("fnt".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let inner_id = SymbolId(10);
    g.rule_names.insert(inner_id, "inner".to_string());
    g.add_rule(Rule {
        lhs: inner_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let fid = FieldId(0);
    g.fields.insert(fid, "child".to_string());
    let outer_id = SymbolId(11);
    g.rule_names.insert(outer_id, "outer".to_string());
    g.add_rule(Rule {
        lhs: outer_id,
        rhs: vec![Symbol::NonTerminal(inner_id)],
        precedence: None,
        associativity: None,
        fields: vec![(fid, 0)],
        production_id: ProductionId(1),
    });

    let arr = gen_parsed(&g);
    let outer = find_node(&arr, "outer").unwrap();
    let child_field = &outer["fields"]["child"];
    assert!(child_field["types"].is_array());
    let type_ref = &child_field["types"][0];
    assert_eq!(type_ref["type"].as_str(), Some("inner"));
    assert_eq!(type_ref["named"], true);
}

// ===========================================================================
// 29. Multiple productions merging fields
// ===========================================================================

#[test]
fn multiple_productions_fields_merged() {
    let mut g = Grammar::new("merge".to_string());
    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let plus_id = SymbolId(1);
    g.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let minus_id = SymbolId(2);
    g.tokens.insert(
        minus_id,
        Token {
            name: "minus".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );

    let left_fid = FieldId(0);
    let op_fid = FieldId(1);
    let right_fid = FieldId(2);
    g.fields.insert(left_fid, "left".to_string());
    g.fields.insert(op_fid, "operator".to_string());
    g.fields.insert(right_fid, "right".to_string());

    let bin_id = SymbolId(10);
    g.rule_names.insert(bin_id, "binary".to_string());
    // Production 1: num + num
    g.add_rule(Rule {
        lhs: bin_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(plus_id),
            Symbol::Terminal(num_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(left_fid, 0), (op_fid, 1), (right_fid, 2)],
        production_id: ProductionId(0),
    });
    // Production 2: num - num
    g.add_rule(Rule {
        lhs: bin_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(minus_id),
            Symbol::Terminal(num_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(left_fid, 0), (op_fid, 1), (right_fid, 2)],
        production_id: ProductionId(1),
    });

    let arr = gen_parsed(&g);
    let bin = find_node(&arr, "binary").unwrap();
    let fields = bin["fields"].as_object().unwrap();
    assert!(fields.contains_key("left"));
    assert!(fields.contains_key("operator"));
    assert!(fields.contains_key("right"));
}

// ===========================================================================
// 30. JSON encoding correctness
// ===========================================================================

#[test]
fn json_does_not_contain_nan_or_infinity() {
    let g = grammar_with_fields();
    let json = generate_json(&g);
    assert!(!json.contains("NaN"));
    assert!(!json.contains("Infinity"));
}

#[test]
fn json_uses_double_quotes() {
    let g = GrammarBuilder::new("q")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    let json = generate_json(&g);
    // JSON must use double quotes for strings
    assert!(json.contains('"'));
}

#[test]
fn json_booleans_are_lowercase() {
    let g = GrammarBuilder::new("b")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    let json = generate_json(&g);
    assert!(json.contains("true") || json.contains("false"));
    assert!(!json.contains("True"));
    assert!(!json.contains("False"));
}

// ===========================================================================
// 31. Token with same name as rule
// ===========================================================================

#[test]
fn token_and_rule_name_collision() {
    // When a token (regex) shares a symbol ID with a rule, the token
    // lookup takes precedence in get_rule_name. Just verify it doesn't crash.
    let mut g = Grammar::new("collision".to_string());
    let shared_id = SymbolId(5);
    g.tokens.insert(
        shared_id,
        Token {
            name: "thing".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(shared_id, "thing".to_string());
    g.add_rule(Rule {
        lhs: shared_id,
        rhs: vec![Symbol::Terminal(shared_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // Should not panic
    let _ = gen_parsed(&g);
}

// ===========================================================================
// 32. Extras are not emitted as node types
// ===========================================================================

#[test]
fn extras_grammar_generates_ok() {
    let g = GrammarBuilder::new("extras")
        .token("NUM", r"\d+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("expr", vec!["NUM"])
        .build();
    let arr = gen_parsed(&g);
    assert!(find_node(&arr, "expr").is_some());
}

// ===========================================================================
// 33. Precedence rules
// ===========================================================================

#[test]
fn precedence_rules_generate_correctly() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence(
            "expr",
            vec!["expr", "+", "expr"],
            1,
            adze_ir::Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "*", "expr"],
            2,
            adze_ir::Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let arr = gen_parsed(&g);
    // expr should appear exactly once as named
    let count = arr
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr") && n["named"] == true)
        .count();
    assert_eq!(count, 1);
}

// ===========================================================================
// 34. Null-safety in output
// ===========================================================================

#[test]
fn no_null_values_in_required_fields() {
    let g = GrammarBuilder::javascript_like();
    let arr = gen_parsed(&g);
    for node in &arr {
        assert!(!node["type"].is_null());
        assert!(!node["named"].is_null());
    }
}

// ===========================================================================
// 35. JSON round-trip stability
// ===========================================================================

#[test]
fn json_roundtrip_stability_python() {
    let g = GrammarBuilder::python_like();
    let json1 = generate_json(&g);
    let val: Value = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&val).unwrap();
    let val2: Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(val, val2);
}

#[test]
fn json_roundtrip_stability_javascript() {
    let g = GrammarBuilder::javascript_like();
    let json1 = generate_json(&g);
    let val: Value = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&val).unwrap();
    let val2: Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(val, val2);
}

// ===========================================================================
// 36. Field type ref for string-literal token
// ===========================================================================

#[test]
fn field_type_ref_for_string_token_is_anonymous() {
    let arr = gen_parsed(&grammar_with_fields());
    let bin = find_node(&arr, "binary_expression").unwrap();
    let op_types = bin["fields"]["operator"]["types"].as_array().unwrap();
    // The operator field points to "+", which is a string-literal token → named: false
    let op_ref = &op_types[0];
    assert_eq!(op_ref["named"], false);
    assert_eq!(op_ref["type"].as_str(), Some("+"));
}

#[test]
fn field_type_ref_for_regex_token_is_named() {
    let arr = gen_parsed(&grammar_with_fields());
    let bin = find_node(&arr, "binary_expression").unwrap();
    let left_types = bin["fields"]["left"]["types"].as_array().unwrap();
    // The left field points to a regex "number" token → named: true
    let left_ref = &left_types[0];
    assert_eq!(left_ref["named"], true);
    assert_eq!(left_ref["type"].as_str(), Some("number"));
}

// ===========================================================================
// 37. Multiple internal rules all excluded
// ===========================================================================

#[test]
fn multiple_internal_rules_excluded() {
    let mut g = Grammar::new("multi_internal".to_string());
    let x_id = SymbolId(0);
    g.tokens.insert(
        x_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::Regex(r"x".to_string()),
            fragile: false,
        },
    );
    for i in 0u16..5 {
        let r_id = SymbolId(10 + i);
        g.rule_names.insert(r_id, format!("_internal_{i}"));
        g.add_rule(Rule {
            lhs: r_id,
            rhs: vec![Symbol::Terminal(x_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        });
    }
    let arr = gen_parsed(&g);
    for node in &arr {
        assert!(
            !node["type"].as_str().is_some_and(|s| s.starts_with('_')),
            "internal rule leaked: {}",
            node["type"]
        );
    }
}

// ===========================================================================
// 38. Total count checks
// ===========================================================================

#[test]
fn count_matches_rules_plus_anon_tokens() {
    let mut g = Grammar::new("count".to_string());
    // 2 anonymous tokens
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "p".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "m".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );
    // 1 regex token (not emitted as anonymous)
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    // 2 named rules
    let r1 = SymbolId(10);
    g.rule_names.insert(r1, "expr".to_string());
    g.add_rule(Rule {
        lhs: r1,
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r2 = SymbolId(11);
    g.rule_names.insert(r2, "stmt".to_string());
    g.add_rule(Rule {
        lhs: r2,
        rhs: vec![Symbol::NonTerminal(r1)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let arr = gen_parsed(&g);
    // 2 named + 2 anonymous = 4
    assert_eq!(arr.len(), 4);
}

// ===========================================================================
// 39. Subtypes key absent when not set
// ===========================================================================

#[test]
fn subtypes_absent_when_not_supertype() {
    let g = GrammarBuilder::new("nosub")
        .token("X", r"x")
        .rule("r", vec!["X"])
        .build();
    let arr = gen_parsed(&g);
    let r = find_node(&arr, "r").unwrap();
    assert!(
        r.get("subtypes").is_none() || r["subtypes"].is_null(),
        "subtypes should be absent"
    );
}

// ===========================================================================
// 40. GrammarBuilder helper grammars
// ===========================================================================

#[test]
fn python_like_grammar_has_suite_node() {
    let arr = gen_parsed(&GrammarBuilder::python_like());
    assert!(find_node(&arr, "suite").is_some());
}

#[test]
fn javascript_like_grammar_has_block_node() {
    let arr = gen_parsed(&GrammarBuilder::javascript_like());
    assert!(find_node(&arr, "block").is_some());
}

#[test]
fn javascript_like_grammar_has_function_declaration() {
    let arr = gen_parsed(&GrammarBuilder::javascript_like());
    assert!(find_node(&arr, "function_declaration").is_some());
}

#[test]
fn javascript_like_grammar_has_expression_statement() {
    let arr = gen_parsed(&GrammarBuilder::javascript_like());
    assert!(find_node(&arr, "expression_statement").is_some());
}
