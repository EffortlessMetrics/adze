//! Comprehensive tests for Grammar struct methods, SymbolId, and construction.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, SymbolId};

// ── Grammar construction via builder ──

#[test]
fn grammar_via_builder_basic() {
    let g = GrammarBuilder::new("test").build();
    assert_eq!(g.name, "test");
}

#[test]
fn grammar_with_single_token() {
    let g = GrammarBuilder::new("t").token("a", "a").build();
    assert!(g.find_symbol_by_name("a").is_some());
}

#[test]
fn grammar_with_rule_and_start() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.start_symbol().is_some());
}

// ── find_symbol_by_name ──

#[test]
fn find_existing_token() {
    let g = GrammarBuilder::new("f1").token("plus", "+").build();
    assert!(g.find_symbol_by_name("plus").is_some());
}

#[test]
fn find_nonexistent() {
    let g = GrammarBuilder::new("f2").token("a", "a").build();
    assert!(g.find_symbol_by_name("xyz").is_none());
}

#[test]
fn find_rule_name() {
    let g = GrammarBuilder::new("f3")
        .token("x", "x")
        .rule("expr", vec!["x"])
        .start("expr")
        .build();
    assert!(g.find_symbol_by_name("expr").is_some());
}

// ── all_rules ──

#[test]
fn rules_count_one() {
    let g = GrammarBuilder::new("rc1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn rules_count_multi() {
    let g = GrammarBuilder::new("rc2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn rules_empty_grammar() {
    let g = GrammarBuilder::new("rc0").build();
    assert_eq!(g.all_rules().count(), 0);
}

// ── normalize ──

#[test]
fn normalize_simple_grammar() {
    let mut g = GrammarBuilder::new("ns")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ = g.normalize();
}

#[test]
fn normalize_recursive_grammar() {
    let mut g = GrammarBuilder::new("nr")
        .token("x", "x")
        .token("p", "+")
        .rule("e", vec!["x"])
        .rule("e", vec!["e", "p", "x"])
        .start("e")
        .build();
    let _ = g.normalize();
}

#[test]
fn normalize_twice_idempotent() {
    let mut g = GrammarBuilder::new("ni")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ = g.normalize();
    let c1 = g.all_rules().count();
    let _ = g.normalize();
    assert_eq!(c1, g.all_rules().count());
}

// ── check_empty_terminals ──

#[test]
fn empty_terminals_ok() {
    let g = GrammarBuilder::new("ce")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.check_empty_terminals().is_ok());
}

// ── build_registry ──

#[test]
fn build_registry() {
    let g = GrammarBuilder::new("br")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _reg = g.build_registry();
}

// ── get_or_build_registry ──

#[test]
fn get_build_registry() {
    let mut g = GrammarBuilder::new("gbr")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _reg = g.get_or_build_registry();
}

// ── Grammar name ──

#[test]
fn name_preserved() {
    assert_eq!(GrammarBuilder::new("my_grammar").build().name, "my_grammar");
}

#[test]
fn name_unicode() {
    assert_eq!(GrammarBuilder::new("日本語").build().name, "日本語");
}

#[test]
fn name_empty() {
    assert_eq!(GrammarBuilder::new("").build().name, "");
}

// ── determinism ──

#[test]
fn grammar_deterministic_build() {
    let make = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build()
    };
    assert_eq!(format!("{:?}", make()), format!("{:?}", make()));
}

// ── precedence ──

#[test]
fn grammar_left_assoc() {
    let g = GrammarBuilder::new("la")
        .token("n", "n")
        .token("p", "+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "p", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn grammar_right_assoc() {
    let g = GrammarBuilder::new("ra")
        .token("n", "n")
        .token("pow", "^")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "pow", "e"], 3, Associativity::Right)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn grammar_multi_prec() {
    let g = GrammarBuilder::new("mp")
        .token("n", "n")
        .token("p", "+")
        .token("m", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "p", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "m", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 3);
}

// ── large ──

#[test]
fn grammar_30_tokens() {
    let mut b = GrammarBuilder::new("big");
    for i in 0..30 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    assert!(g.all_rules().count() >= 30);
}

#[test]
fn grammar_chain_4() {
    let g = GrammarBuilder::new("ch")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 4);
}

// ── Grammar::new directly ──

#[test]
fn grammar_direct_new() {
    let g = Grammar::new("direct".to_string());
    assert_eq!(g.name, "direct");
    assert_eq!(g.all_rules().count(), 0);
}

// ── Grammar traits ──

#[test]
fn grammar_is_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<Grammar>();
}

#[test]
fn grammar_is_clone() {
    fn check<T: Clone>() {}
    check::<Grammar>();
}

#[test]
fn grammar_clone_equal() {
    let g = GrammarBuilder::new("cl")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let c = g.clone();
    assert_eq!(g.name, c.name);
    assert_eq!(g.all_rules().count(), c.all_rules().count());
}

// ── SymbolId ──

#[test]
fn symbol_id_zero() {
    assert_eq!(SymbolId(0).0, 0);
}

#[test]
fn symbol_id_max() {
    assert_eq!(SymbolId(u16::MAX).0, u16::MAX);
}

#[test]
fn symbol_id_eq() {
    assert_eq!(SymbolId(1), SymbolId(1));
}

#[test]
fn symbol_id_ne() {
    assert_ne!(SymbolId(1), SymbolId(2));
}

#[test]
fn symbol_id_debug() {
    assert!(format!("{:?}", SymbolId(42)).contains("42"));
}

#[test]
fn symbol_id_clone() {
    let a = SymbolId(5);
    assert_eq!(a, a.clone());
}

#[test]
fn symbol_id_copy() {
    let a = SymbolId(5);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn symbol_id_hash() {
    use std::collections::HashSet;
    let mut s = HashSet::new();
    s.insert(SymbolId(0));
    s.insert(SymbolId(1));
    s.insert(SymbolId(0));
    assert_eq!(s.len(), 2);
}

#[test]
fn symbol_id_ord() {
    assert!(SymbolId(0) < SymbolId(1));
}

#[test]
fn symbol_id_sort() {
    let mut v = vec![SymbolId(3), SymbolId(1), SymbolId(2)];
    v.sort();
    assert_eq!(v, vec![SymbolId(1), SymbolId(2), SymbolId(3)]);
}

#[test]
fn symbol_id_btree() {
    use std::collections::BTreeSet;
    let mut s = BTreeSet::new();
    s.insert(SymbolId(2));
    s.insert(SymbolId(0));
    s.insert(SymbolId(1));
    assert_eq!(s.len(), 3);
    assert_eq!(*s.iter().next().unwrap(), SymbolId(0));
}
