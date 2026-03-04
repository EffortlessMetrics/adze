//! Comprehensive tests for adze_ir Grammar serialization roundtrip.

use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("e", vec!["num"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, adze_ir::Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, adze_ir::Associativity::Left)
        .start("e")
        .build()
}

// ── JSON serialization ──

#[test]
fn json_serialize_simple() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn json_serialize_arith() {
    let g = arith_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn json_serialize_pretty() {
    let g = simple_grammar();
    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains('\n'));
}

// ── JSON roundtrip ──

#[test]
fn json_roundtrip_name() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn json_roundtrip_tokens() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn json_roundtrip_rules() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn json_roundtrip_arith_name() {
    let g = arith_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn json_roundtrip_arith_tokens() {
    let g = arith_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn json_roundtrip_arith_rules() {
    let g = arith_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

// ── JSON value structure ──

#[test]
fn json_value_is_object() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_object());
}

#[test]
fn json_value_has_name() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("name").is_some());
}

#[test]
fn json_value_name_matches() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["name"].as_str().unwrap(), "simple");
}

// ── Double roundtrip ──

#[test]
fn double_roundtrip() {
    let g = simple_grammar();
    let j1 = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&g2).unwrap();
    assert_eq!(j1, j2);
}

// ── Multi-token grammar ──

#[test]
fn multi_token_roundtrip() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

// ── Alternative grammar ──

#[test]
fn alt_roundtrip() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

// ── Chain grammar ──

#[test]
fn chain_roundtrip() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

// ── Normalized grammar ──

#[test]
fn normalized_roundtrip() {
    let mut g = GrammarBuilder::new("norm")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

// ── Large grammar ──

#[test]
fn large_grammar_roundtrip() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..30 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

// ── JSON string content ──

#[test]
fn json_contains_grammar_name() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("simple"));
}

#[test]
fn json_contains_rules() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("rules"));
}

#[test]
fn json_contains_tokens() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("tokens"));
}
