//! Comprehensive tests for NodeTypesGenerator in adze-tablegen.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::NodeTypesGenerator;

fn build_grammar(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> adze_ir::Grammar {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    g
}

// ── Basic construction ──

#[test]
fn node_types_generator_creation() {
    let g = build_grammar("test", &[("a", "a")], &[("s", vec!["a"])], "s");
    let _gen = NodeTypesGenerator::new(&g);
}

#[test]
fn generate_returns_ok() {
    let g = build_grammar("test", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let result = gen_instance.generate();
    assert!(result.is_ok());
}

#[test]
fn generate_returns_json() {
    let g = build_grammar("test", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    assert!(!json.is_empty());
}

#[test]
fn generate_returns_valid_json() {
    let g = build_grammar("test", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
}

#[test]
fn generate_returns_json_array() {
    let g = build_grammar("test", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
}

// ── Node type entries ──

#[test]
fn node_types_has_entries() {
    let g = build_grammar("basic", &[("x", "x")], &[("root", vec!["x"])], "root");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn node_types_entries_have_type_field() {
    let g = build_grammar("typed", &[("x", "x")], &[("root", vec!["x"])], "root");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "Entry missing 'type': {:?}",
            entry
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let g = build_grammar("named", &[("x", "x")], &[("root", vec!["x"])], "root");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "Entry missing 'named': {:?}",
            entry
        );
    }
}

// ── Multiple tokens ──

#[test]
fn node_types_multi_token() {
    let g = build_grammar(
        "multi",
        &[("a", "a"), ("b", "b")],
        &[("root", vec!["a", "b"])],
        "root",
    );
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.as_array().unwrap().len() >= 2);
}

#[test]
fn node_types_three_tokens() {
    let g = build_grammar(
        "three",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("root", vec!["a", "b", "c"])],
        "root",
    );
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.as_array().unwrap().len() >= 3);
}

// ── Multiple rules ──

#[test]
fn node_types_multiple_rules() {
    let g = build_grammar(
        "multirule",
        &[("x", "x"), ("y", "y")],
        &[
            ("expr", vec!["x"]),
            ("expr", vec!["y"]),
            ("root", vec!["expr"]),
        ],
        "root",
    );
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.as_array().unwrap().len() >= 2);
}

// ── Consistency ──

#[test]
fn generate_deterministic() {
    let g = build_grammar("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json1 = gen_instance.generate().unwrap();
    let json2 = gen_instance.generate().unwrap();
    assert_eq!(json1, json2);
}

// ── With precedence ──

#[test]
fn node_types_with_precedence() {
    let mut g = GrammarBuilder::new("prec")
        .token("x", "x")
        .token("y", "y")
        .rule_with_precedence("expr", vec!["x"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["y"], 2, Associativity::Right)
        .rule("root", vec!["expr"])
        .start("root")
        .build();
    g.normalize();
    let gen_instance = NodeTypesGenerator::new(&g);
    let result = gen_instance.generate();
    assert!(result.is_ok());
}

// ── JSON structure ──

#[test]
fn json_entries_are_objects() {
    let g = build_grammar("obj", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry.is_object());
    }
}

#[test]
fn json_type_values_are_strings() {
    let g = build_grammar("strtype", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry["type"].is_string());
    }
}

#[test]
fn json_named_values_are_booleans() {
    let g = build_grammar("boolnamed", &[("a", "a")], &[("s", vec!["a"])], "s");
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry["named"].is_boolean());
    }
}

// ── Edge cases ──

#[test]
fn many_tokens_grammar() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..15 {
        let n = format!("tok{}", i);
        b = b.token(&n, &n);
    }
    b = b.rule("start", vec!["tok0", "tok1"]).start("start");
    let mut g = b.build();
    g.normalize();
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn many_alternatives_grammar() {
    let mut b = GrammarBuilder::new("alts");
    for i in 0..8 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..8 {
        let tok = format!("t{}", i);
        b = b.rule("start", vec![&tok]);
    }
    b = b.start("start");
    let mut g = b.build();
    g.normalize();
    let gen_instance = NodeTypesGenerator::new(&g);
    let result = gen_instance.generate();
    assert!(result.is_ok());
}

#[test]
fn chain_rules_grammar() {
    let g = build_grammar(
        "chain",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    let gen_instance = NodeTypesGenerator::new(&g);
    let json = gen_instance.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.as_array().unwrap().len() >= 3);
}
