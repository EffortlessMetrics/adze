// Comprehensive tests for Grammar clone, debug, and equality patterns
// Tests the derived trait behaviors

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, SymbolId};

#[test]
fn grammar_clone_preserves_name() {
    let g = GrammarBuilder::new("clone")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
}

#[test]
fn grammar_clone_preserves_tokens() {
    let g = GrammarBuilder::new("tok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn grammar_clone_preserves_rules() {
    let g = GrammarBuilder::new("rul")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["a", "a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn grammar_clone_preserves_start() {
    let g = GrammarBuilder::new("st")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

#[test]
fn grammar_debug_non_empty() {
    let g = GrammarBuilder::new("dbg")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let dbg = format!("{:?}", g);
    assert!(!dbg.is_empty());
    assert!(dbg.contains("dbg"));
}

#[test]
fn grammar_clone_independent() {
    let g = GrammarBuilder::new("ind")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut g2 = g.clone();
    g2.name = "modified".to_string();
    assert_ne!(g.name, g2.name);
}

#[test]
fn symbol_id_clone() {
    let s = SymbolId(42);
    let s2 = s;
    assert_eq!(s, s2);
}

#[test]
fn symbol_id_debug() {
    let s = SymbolId(7);
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("7"));
}

#[test]
fn symbol_id_equality() {
    assert_eq!(SymbolId(0), SymbolId(0));
    assert_ne!(SymbolId(0), SymbolId(1));
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(100) > SymbolId(99));
}

#[test]
fn symbol_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1));
    assert_eq!(set.len(), 2);
}

#[test]
fn associativity_clone_eq() {
    let a = Associativity::Left;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn associativity_debug() {
    let dbg = format!("{:?}", Associativity::Right);
    assert!(dbg.contains("Right"));
}

#[test]
fn grammar_rule_names_after_clone() {
    let g = GrammarBuilder::new("rn")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let g2 = g.clone();
    assert!(g2.rule_names.values().any(|n| n == "expr"));
}
