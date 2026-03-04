//! Comprehensive tests for GrammarBuilder error handling and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use std::panic::{AssertUnwindSafe, catch_unwind};

// ── Successful builds ──

#[test]
fn build_minimal() {
    let g = GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "minimal");
}

#[test]
fn build_two_tokens() {
    let g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn build_three_rules() {
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

// ── Builder chaining ──

#[test]
fn chain_many_tokens() {
    let mut b = GrammarBuilder::new("chain");
    for i in 0..20 {
        let n = format!("tok{}", i);
        b = b.token(&n, &n);
    }
    b = b.rule("s", vec!["tok0"]).start("s");
    let g = b.build();
    assert_eq!(g.tokens.len(), 20);
}

#[test]
fn chain_many_rules() {
    let mut b = GrammarBuilder::new("rules");
    for i in 0..10 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..10 {
        let tok = format!("t{}", i);
        b = b.rule("expr", vec![&tok]);
    }
    b = b.rule("s", vec!["expr"]).start("s");
    let g = b.build();
    assert!(g.all_rules().count() >= 10);
}

// ── Precedence rules ──

#[test]
fn precedence_left() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn precedence_right() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Right)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn precedence_none() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 0, Associativity::Left)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn negative_precedence() {
    let g = GrammarBuilder::new("neg")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], -1, Associativity::Left)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn high_precedence() {
    let g = GrammarBuilder::new("high")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 100, Associativity::Right)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

// ── Empty RHS panics ──

#[test]
fn empty_rhs_builds() {
    // Empty RHS is now allowed (represents epsilon rules)
    let g = GrammarBuilder::new("empty")
        .token("a", "a")
        .rule("s", vec![])
        .start("s")
        .build();
    assert_eq!(g.name, "empty");
}

// ── Grammar names ──

#[test]
fn name_simple() {
    let g = GrammarBuilder::new("hello")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "hello");
}

#[test]
fn name_with_underscore() {
    let g = GrammarBuilder::new("my_grammar")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn name_with_numbers() {
    let g = GrammarBuilder::new("grammar123")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "grammar123");
}

#[test]
fn name_single_char() {
    let g = GrammarBuilder::new("x")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "x");
}

#[test]
fn name_long() {
    let name = "a".repeat(100);
    let g = GrammarBuilder::new(&name)
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name.len(), 100);
}

// ── Token patterns ──

#[test]
fn token_single_char() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn token_regex_pattern() {
    let g = GrammarBuilder::new("t")
        .token("num", "[0-9]+")
        .rule("s", vec!["num"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn token_multi_char() {
    let g = GrammarBuilder::new("t")
        .token("kw", "function")
        .rule("s", vec!["kw"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

// ── Clone and Debug ──

#[test]
fn grammar_clone() {
    let g = GrammarBuilder::new("clone")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
}

#[test]
fn grammar_debug() {
    let g = GrammarBuilder::new("debug")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("debug"));
}

// ── Serialize/Deserialize ──

#[test]
fn grammar_serialize() {
    let g = GrammarBuilder::new("ser")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("ser"));
}

#[test]
fn grammar_roundtrip() {
    let g = GrammarBuilder::new("rt")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

// ── Start symbol ──

#[test]
fn start_symbol_present() {
    let g = GrammarBuilder::new("start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.start_symbol().is_some());
}

// ── Rule-related ──

#[test]
fn all_rules_nonempty() {
    let g = GrammarBuilder::new("rules")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn rule_lhs_is_symbol_id() {
    let g = GrammarBuilder::new("lhs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _id = rule.lhs;
    }
}

#[test]
fn rule_rhs_nonempty() {
    let g = GrammarBuilder::new("rhs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        assert!(!rule.rhs.is_empty());
    }
}
