//! Tests for grammar JSON format output stability.
//!
//! Verifies that grammars built with GrammarBuilder produce
//! expected JSON structure when serialized.

use adze_ir::builder::GrammarBuilder;

#[test]
fn grammar_json_has_name() {
    let g = GrammarBuilder::new("test_grammar")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let json = serde_json::to_value(&g).expect("serialize");
    assert_eq!(json["name"].as_str().unwrap(), "test_grammar");
}

#[test]
fn grammar_json_has_rules() {
    let g = GrammarBuilder::new("rules")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let json = serde_json::to_value(&g).expect("serialize");
    assert!(json["rules"].is_object());
}

#[test]
fn grammar_json_has_tokens() {
    let g = GrammarBuilder::new("tokens")
        .token("num", r"\d+")
        .token("ws", r"\s+")
        .rule("start", vec!["num"])
        .build();
    let json = serde_json::to_value(&g).expect("serialize");
    assert!(json["tokens"].is_object());
}

#[test]
fn grammar_json_roundtrip() {
    let g = GrammarBuilder::new("roundtrip")
        .token("a", "a")
        .rule("start", vec!["a"])
        .build();
    let json_str = serde_json::to_string(&g).expect("serialize");
    let g2: adze_ir::Grammar = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn grammar_json_extras_preserved() {
    let g = GrammarBuilder::new("extras")
        .token("num", r"\d+")
        .token("ws", r"\s+")
        .rule("expr", vec!["num"])
        .extra("ws")
        .build();
    let json = serde_json::to_value(&g).expect("serialize");
    let extras = json["extras"].as_array().expect("extras array");
    assert!(!extras.is_empty());
}

#[test]
fn grammar_json_externals_preserved() {
    let g = GrammarBuilder::new("externals")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .external("indent")
        .build();
    let json = serde_json::to_value(&g).expect("serialize");
    let externals = json["externals"].as_array().expect("externals array");
    assert!(!externals.is_empty());
}

#[test]
fn grammar_json_precedence_preserved() {
    use adze_ir::Associativity;
    let g = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .build();
    let json = serde_json::to_value(&g).expect("serialize");
    let prec = &json["precedences"];
    assert!(prec.is_array());
}
