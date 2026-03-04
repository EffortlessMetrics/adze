//! Comprehensive tests for Grammar normalize() behavior.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn three_token_grammar() -> Grammar {
    GrammarBuilder::new("three")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

// ── normalize preserves basic properties ──

#[test]
fn normalize_preserves_name() {
    let mut g = simple_grammar();
    g.normalize();
    assert_eq!(g.name, "test");
}

#[test]
fn normalize_preserves_tokens() {
    let mut g = simple_grammar();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn normalize_preserves_start() {
    let mut g = simple_grammar();
    g.normalize();
    assert!(g.start_symbol().is_some());
}

#[test]
fn normalize_idempotent() {
    let mut g = simple_grammar();
    g.normalize();
    let after_first = format!("{:?}", g);
    g.normalize();
    let after_second = format!("{:?}", g);
    assert_eq!(after_first, after_second);
}

#[test]
fn normalize_double_idempotent() {
    let mut g = three_token_grammar();
    g.normalize();
    g.normalize();
    g.normalize();
    assert_eq!(g.name, "three");
}

// ── normalize on various grammar shapes ──

#[test]
fn normalize_single_rule() {
    let mut g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    g.normalize();
    assert!(g.rules.len() >= 1);
}

#[test]
fn normalize_multi_rule() {
    let mut g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    g.normalize();
    assert!(g.rules.len() >= 2);
}

#[test]
fn normalize_chain_rules() {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    g.normalize();
    assert!(g.start_symbol().is_some());
}

#[test]
fn normalize_wide_rule() {
    let mut g = GrammarBuilder::new("wide")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 1);
}

// ── Clone after normalize ──

#[test]
fn clone_after_normalize() {
    let mut g = simple_grammar();
    g.normalize();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
}

#[test]
fn clone_preserves_rule_count() {
    let mut g = three_token_grammar();
    g.normalize();
    let g2 = g.clone();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

// ── Serialization after normalize ──

#[test]
fn serialize_after_normalize() {
    let mut g = simple_grammar();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("test"));
}

#[test]
fn roundtrip_after_normalize() {
    let mut g = simple_grammar();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

// ── normalize on grammar with precedence ──

#[test]
fn normalize_with_precedence() {
    use adze_ir::Associativity;
    let mut g = GrammarBuilder::new("prec")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("start", vec!["a", "b"], 1, Associativity::Left)
        .start("start")
        .build();
    g.normalize();
    assert!(g.start_symbol().is_some());
}

#[test]
fn normalize_preserves_all_rules_with_precedence() {
    use adze_ir::Associativity;
    let mut g = GrammarBuilder::new("prec2")
        .token("x", "x")
        .token("y", "y")
        .rule_with_precedence("expr", vec!["x"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["y"], 2, Associativity::Right)
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    g.normalize();
    assert!(g.rules.len() >= 2);
}

// ── Start symbol robustness ──

#[test]
fn start_symbol_survives_normalize() {
    let mut g = simple_grammar();
    let s1 = g.start_symbol();
    g.normalize();
    let s2 = g.start_symbol();
    assert_eq!(s1, s2);
}

// ── Token IDs survive normalize ──

#[test]
fn token_ids_stable_after_normalize() {
    let mut g = simple_grammar();
    let ids_before: Vec<SymbolId> = g.tokens.keys().copied().collect();
    g.normalize();
    let ids_after: Vec<SymbolId> = g.tokens.keys().copied().collect();
    assert_eq!(ids_before, ids_after);
}

// ── Rule names survive normalize ──

#[test]
fn rule_names_survive_normalize() {
    let mut g = simple_grammar();
    let names_before: Vec<String> = g.rule_names.values().cloned().collect();
    g.normalize();
    for name in &names_before {
        assert!(g.rule_names.values().any(|n| n == name));
    }
}

// ── Edge cases ──

#[test]
fn normalize_grammar_with_many_tokens() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..20 {
        let name = format!("t{}", i);
        b = b.token(&name, &name);
    }
    b = b.rule("start", vec!["t0", "t1"]).start("start");
    let mut g = b.build();
    g.normalize();
    assert_eq!(g.tokens.len(), 20);
}

#[test]
fn normalize_grammar_with_many_alternatives() {
    let mut b = GrammarBuilder::new("alts");
    for i in 0..10 {
        let name = format!("t{}", i);
        b = b.token(&name, &name);
    }
    for i in 0..10 {
        let tok = format!("t{}", i);
        b = b.rule("start", vec![&tok]);
    }
    b = b.start("start");
    let mut g = b.build();
    g.normalize();
    assert!(g.all_rules().count() >= 10);
}

// ── Debug output ──

#[test]
fn debug_output_after_normalize() {
    let mut g = simple_grammar();
    g.normalize();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("test"));
}
