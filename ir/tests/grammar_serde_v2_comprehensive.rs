//! Comprehensive tests for Grammar serialization and deserialization.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("serial")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

// ── JSON Serialization ──

#[test]
fn serialize_to_json() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn serialize_contains_name() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("serial"));
}

#[test]
fn serialize_pretty() {
    let g = simple_grammar();
    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains('\n'));
}

#[test]
fn serialize_compact_no_newlines() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.contains('\n'));
}

// ── JSON Deserialization ──

#[test]
fn deserialize_from_json() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn roundtrip_preserves_name() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn roundtrip_preserves_token_count() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn roundtrip_preserves_rule_count() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
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
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

// ── Alternative rules ──

#[test]
fn alternative_rules_roundtrip() {
    let g = GrammarBuilder::new("alts")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

// ── With precedence ──

#[test]
fn precedence_roundtrip() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

// ── Normalized grammar ──

#[test]
fn normalized_roundtrip() {
    let mut g = simple_grammar();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

// ── serde_json::Value ──

#[test]
fn as_json_value() {
    let g = simple_grammar();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val.is_object());
}

#[test]
fn json_value_has_name() {
    let g = simple_grammar();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert_eq!(val["name"], "serial");
}

#[test]
fn json_value_roundtrip() {
    let g = simple_grammar();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    let g2: Grammar = serde_json::from_value(val).unwrap();
    assert_eq!(g.name, g2.name);
}

// ── Large grammar ──

#[test]
fn large_grammar_roundtrip() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..30 {
        let n = format!("tok{}", i);
        b = b.token(&n, &n);
    }
    b = b.rule("s", vec!["tok0", "tok1"]).start("s");
    let g = b.build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

// ── Clone vs serialize ──

#[test]
fn clone_equals_roundtrip() {
    let g = simple_grammar();
    let cloned = g.clone();
    let json = serde_json::to_string(&g).unwrap();
    let deserialized: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(cloned.name, deserialized.name);
    assert_eq!(cloned.tokens.len(), deserialized.tokens.len());
}

// ── Various names ──

#[test]
fn name_with_special_chars() {
    let g = GrammarBuilder::new("my_lang_v2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g2.name, "my_lang_v2");
}

#[test]
fn name_single_char() {
    let g = GrammarBuilder::new("x")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"x\""));
}

#[test]
fn name_empty() {
    let g = GrammarBuilder::new("")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g2.name, "");
}

// ── Start symbol preservation ──

#[test]
fn start_symbol_preserved() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.start_symbol(), g2.start_symbol());
}
