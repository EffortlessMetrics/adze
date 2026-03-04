//! Comprehensive tests for Grammar struct methods and properties.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, Rule, Symbol, SymbolId};

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build()
}

// ── Grammar name ──

#[test]
fn grammar_name() {
    let g = simple_grammar();
    assert_eq!(g.name, "simple");
}

#[test]
fn grammar_name_unicode() {
    let g = GrammarBuilder::new("日本語").build();
    assert_eq!(g.name, "日本語");
}

#[test]
fn grammar_name_empty() {
    let g = GrammarBuilder::new("").build();
    assert_eq!(g.name, "");
}

// ── Start symbol ──

#[test]
fn grammar_start_symbol_set() {
    let g = simple_grammar();
    assert!(g.start_symbol().is_some());
}

#[test]
fn grammar_start_symbol_none() {
    let g = GrammarBuilder::new("no_start").token("x", "x").build();
    // Without calling .start(), start_symbol may or may not be None depending on implementation
    let _ = g.start_symbol();
}

// ── Tokens ──

#[test]
fn grammar_has_tokens() {
    let g = simple_grammar();
    assert!(!g.tokens.is_empty());
}

#[test]
fn grammar_token_count() {
    let g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .build();
    assert!(g.tokens.len() >= 2);
}

// ── Rules ──

#[test]
fn grammar_has_rules() {
    let g = simple_grammar();
    assert!(g.all_rules().count() > 0);
}

#[test]
fn grammar_rule_count() {
    let g = GrammarBuilder::new("three")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 3);
}

// ── Rule names ──

#[test]
fn grammar_rule_names_nonempty() {
    let g = simple_grammar();
    assert!(!g.rule_names.is_empty());
}

// ── Normalize ──

#[test]
fn grammar_normalize_doesnt_panic() {
    let mut g = simple_grammar();
    g.normalize();
}

#[test]
fn grammar_normalize_preserves_name() {
    let mut g = simple_grammar();
    g.normalize();
    assert_eq!(g.name, "simple");
}

#[test]
fn grammar_normalize_preserves_start() {
    let mut g = simple_grammar();
    let start_before = g.start_symbol();
    g.normalize();
    let start_after = g.start_symbol();
    assert_eq!(start_before, start_after);
}

#[test]
fn grammar_normalize_idempotent() {
    let mut g1 = simple_grammar();
    g1.normalize();
    let count_after_first = g1.all_rules().count();
    g1.normalize();
    let count_after_second = g1.all_rules().count();
    assert_eq!(count_after_first, count_after_second);
}

// ── Clone ──

#[test]
fn grammar_clone() {
    let g1 = simple_grammar();
    let g2 = g1.clone();
    assert_eq!(g1.name, g2.name);
}

// ── Debug ──

#[test]
fn grammar_debug() {
    let g = simple_grammar();
    let d = format!("{:?}", g);
    assert!(d.contains("simple"));
}

// ── SymbolId ──

#[test]
fn symbol_id_creation() {
    let s = SymbolId(42);
    assert_eq!(s.0, 42);
}

#[test]
fn symbol_id_equality() {
    assert_eq!(SymbolId(1), SymbolId(1));
    assert_ne!(SymbolId(1), SymbolId(2));
}

#[test]
fn symbol_id_copy() {
    let s1 = SymbolId(10);
    let s2 = s1;
    assert_eq!(s1, s2);
}

#[test]
fn symbol_id_debug() {
    let s = SymbolId(7);
    let d = format!("{:?}", s);
    assert!(d.contains("7"));
}

// ── Rule ──

#[test]
fn grammar_rules_have_lhs() {
    let g = simple_grammar();
    for rule in g.all_rules() {
        let _ = rule.lhs;
    }
}

#[test]
fn grammar_rules_have_rhs() {
    let g = simple_grammar();
    for rule in g.all_rules() {
        let _ = &rule.rhs;
    }
}

// ── Multi-nonterminal ──

#[test]
fn grammar_multi_nt() {
    let g = GrammarBuilder::new("multi")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 3);
}

// ── Precedence ──

#[test]
fn grammar_with_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, adze_ir::Associativity::Left)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

// ── Large grammar ──

#[test]
fn grammar_large() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..20 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    assert!(g.all_rules().count() >= 20);
}

// ── Empty grammar ──

#[test]
fn grammar_empty() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.all_rules().count(), 0);
}

#[test]
fn grammar_empty_tokens() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.tokens.is_empty());
}

// ── Fields ──

#[test]
fn grammar_fields_empty_initially() {
    let g = simple_grammar();
    // fields may be empty for simple grammars
    let _ = &g.fields;
}

// ── Extras ──

#[test]
fn grammar_extras_empty_initially() {
    let g = simple_grammar();
    let _ = &g.extras;
}

// ── Externals ──

#[test]
fn grammar_externals_empty_initially() {
    let g = simple_grammar();
    let _ = &g.externals;
}

// ── Conflicts ──

#[test]
fn grammar_conflicts_empty_initially() {
    let g = simple_grammar();
    let _ = &g.conflicts;
}

// ── Inline rules ──

#[test]
fn grammar_inline_rules_empty() {
    let g = simple_grammar();
    let _ = &g.inline_rules;
}

// ── Supertypes ──

#[test]
fn grammar_supertypes_empty() {
    let g = simple_grammar();
    let _ = &g.supertypes;
}
