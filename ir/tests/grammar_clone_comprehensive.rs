//! Comprehensive tests for Grammar Clone, Debug, Serialize, Deserialize behavior.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, Symbol, SymbolId};

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("clone_test")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build()
}

fn complex_grammar() -> Grammar {
    GrammarBuilder::new("complex")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["a"])
        .rule("A", vec!["a", "A"])
        .rule("B", vec!["b"])
        .rule("B", vec!["b", "c"])
        .start("S")
        .build()
}

#[test]
fn clone_preserves_name() {
    let g = simple_grammar();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
}

#[test]
fn clone_preserves_rules() {
    let g = complex_grammar();
    let g2 = g.clone();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn clone_preserves_tokens() {
    let g = simple_grammar();
    let g2 = g.clone();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn clone_preserves_rule_names() {
    let g = complex_grammar();
    let g2 = g.clone();
    assert_eq!(g.rule_names.len(), g2.rule_names.len());
    for (id, name) in &g.rule_names {
        assert_eq!(g2.rule_names.get(id), Some(name));
    }
}

#[test]
fn clone_preserves_start_symbol() {
    let g = simple_grammar();
    let g2 = g.clone();
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

#[test]
fn clone_is_independent() {
    let g = simple_grammar();
    let mut g2 = g.clone();
    g2.name = "modified".to_string();
    assert_ne!(g.name, g2.name);
}

#[test]
fn debug_contains_name() {
    let g = simple_grammar();
    let dbg = format!("{:?}", g);
    assert!(
        dbg.contains("clone_test"),
        "Debug output should contain grammar name"
    );
}

#[test]
fn debug_not_empty() {
    let g = simple_grammar();
    let dbg = format!("{:?}", g);
    assert!(!dbg.is_empty());
}

#[test]
fn serialize_roundtrip_json() {
    let g = simple_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn serialize_roundtrip_complex() {
    let g = complex_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn serialize_roundtrip_preserves_rule_names() {
    let g = complex_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    for (id, name) in &g.rule_names {
        assert_eq!(
            g2.rule_names.get(id).map(|s| s.as_str()),
            Some(name.as_str())
        );
    }
}

#[test]
fn empty_grammar_clone() {
    let g = GrammarBuilder::new("empty").build();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.all_rules().count(), 0);
    assert_eq!(g2.all_rules().count(), 0);
}

#[test]
fn empty_grammar_serialize() {
    let g = GrammarBuilder::new("empty").build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn symbol_id_clone() {
    let id = SymbolId(42);
    let id2 = id;
    assert_eq!(id, id2);
}

#[test]
fn symbol_clone() {
    let s = Symbol::Terminal(SymbolId(1));
    let s2 = s.clone();
    assert_eq!(format!("{:?}", s), format!("{:?}", s2));
}
