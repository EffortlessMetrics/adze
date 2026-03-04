//! Comprehensive tests for GrammarBuilder fluent API edge cases v2.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use std::panic::catch_unwind;

// ── Basic builder patterns ──

#[test]
fn builder_v2_minimal() {
    let g = GrammarBuilder::new("min")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn builder_v2_grammar_name() {
    let g = GrammarBuilder::new("named")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "named");
}

#[test]
fn builder_v2_multi_tokens() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    assert!(g.tokens.len() >= 3);
}

// ── Token regex patterns ──

#[test]
fn builder_v2_token_regex() {
    let g = GrammarBuilder::new("regex")
        .token("num", "[0-9]+")
        .rule("s", vec!["num"])
        .start("s")
        .build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn builder_v2_token_escaped_plus() {
    let g = GrammarBuilder::new("esc")
        .token("plus", "\\+")
        .rule("s", vec!["plus"])
        .start("s")
        .build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn builder_v2_token_word_pattern() {
    let g = GrammarBuilder::new("word")
        .token("id", "[a-zA-Z_][a-zA-Z0-9_]*")
        .rule("s", vec!["id"])
        .start("s")
        .build();
    assert!(!g.tokens.is_empty());
}

// ── Alternative rules ──

#[test]
fn builder_v2_three_alternatives() {
    let g = GrammarBuilder::new("alt")
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

// ── Chain grammars ──

#[test]
fn builder_v2_chain() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 3);
}

#[test]
fn builder_v2_deep_chain() {
    let g = GrammarBuilder::new("deep")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("d", vec!["c"])
        .rule("s", vec!["d"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 5);
}

// ── Precedence rules ──

#[test]
fn builder_v2_left_assoc() {
    let g = GrammarBuilder::new("left")
        .token("n", "n")
        .token("plus", "\\+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_v2_right_assoc() {
    let g = GrammarBuilder::new("right")
        .token("n", "n")
        .token("eq", "=")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "eq", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_v2_multi_prec() {
    let g = GrammarBuilder::new("multiprec")
        .token("n", "n")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 3);
}

// ── Normalize ──

#[test]
fn builder_v2_normalize() {
    let mut g = GrammarBuilder::new("norm")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 2);
}

// ── Clone and Debug ──

#[test]
fn builder_v2_grammar_clone() {
    let g = GrammarBuilder::new("cl")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let c = g.clone();
    assert_eq!(g.name, c.name);
}

#[test]
fn builder_v2_grammar_debug() {
    let g = GrammarBuilder::new("dbg")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let d = format!("{:?}", g);
    assert!(d.contains("dbg"));
}

// ── Error cases ──

#[test]
fn builder_v2_empty_name() {
    let g = GrammarBuilder::new("")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_eq!(g.name, "");
}

// ── Serialization roundtrip ──

#[test]
fn builder_v2_serde_roundtrip() {
    let g = GrammarBuilder::new("ser")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn builder_v2_serde_pretty() {
    let g = GrammarBuilder::new("pretty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains("pretty"));
}

// ── Start symbol ──

#[test]
fn builder_v2_start_symbol_some() {
    let g = GrammarBuilder::new("st")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.start_symbol().is_some());
}

// ── Rule names ──

#[test]
fn builder_v2_rule_names_populated() {
    let g = GrammarBuilder::new("rn")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(!g.rule_names.is_empty());
}

// ── Many tokens scale ──

#[test]
fn builder_v2_scale_50_tokens() {
    let mut b = GrammarBuilder::new("scale");
    for i in 0..50 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n);
    }
    let g = b.rule("s", vec!["t0"]).start("s").build();
    assert!(g.tokens.len() >= 50);
}

#[test]
fn builder_v2_scale_30_rules() {
    let mut b = GrammarBuilder::new("scale_rules");
    for i in 0..30 {
        let n: &str = Box::leak(format!("tok{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    assert!(g.all_rules().count() >= 30);
}
