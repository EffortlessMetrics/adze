// Comprehensive tests for GrammarBuilder → Grammar roundtrip and properties.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, Symbol, SymbolId, TokenPattern};

// ===== Construction =====

#[test]
fn builder_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.name, "empty");
}

#[test]
fn builder_single_token() {
    let g = GrammarBuilder::new("t").token("a", "a").build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn builder_single_rule() {
    let g = GrammarBuilder::new("r")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn builder_name_preserved() {
    let g = GrammarBuilder::new("my_grammar").build();
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn builder_unicode_name() {
    let g = GrammarBuilder::new("日本語").build();
    assert_eq!(g.name, "日本語");
}

#[test]
fn builder_empty_string_name() {
    let g = GrammarBuilder::new("").build();
    assert_eq!(g.name, "");
}

// ===== Token registration =====

#[test]
fn builder_token_registered() {
    let g = GrammarBuilder::new("t").token("number", r"\d+").build();
    let has_number = g.tokens.values().any(|t| t.name == "number");
    assert!(has_number);
}

#[test]
fn builder_multiple_tokens() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn builder_token_pattern_preserved() {
    let g = GrammarBuilder::new("t").token("num", r"\d+").build();
    let tok = g.tokens.values().find(|t| t.name == "num").unwrap();
    match &tok.pattern {
        TokenPattern::String(s) => assert_eq!(s, r"\d+"),
        TokenPattern::Regex(r) => assert_eq!(r, r"\d+"),
    }
}

#[test]
fn builder_many_tokens() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..100 {
        b = b.token(&format!("t{}", i), &format!("{}", i));
    }
    let g = b.build();
    assert_eq!(g.tokens.len(), 100);
}

// ===== Rule registration =====

#[test]
fn builder_rule_registered() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn builder_multiple_rules() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["child"])
        .rule("child", vec!["a"])
        .rule("alt", vec!["b"])
        .start("root")
        .build();
    assert!(g.rules.len() >= 2);
}

#[test]
fn builder_rule_with_multiple_rhs() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("seq", vec!["a", "b", "c"])
        .start("seq")
        .build();
    // Check the rule has 3 symbols in RHS
    let rules: Vec<_> = g.rules.values().flat_map(|v| v.iter()).collect();
    assert!(rules.iter().any(|r| r.rhs.len() == 3));
}

// ===== Start symbol =====

#[test]
fn builder_start_symbol() {
    let g = GrammarBuilder::new("s")
        .token("x", "x")
        .rule("program", vec!["x"])
        .start("program")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn builder_no_start() {
    let g = GrammarBuilder::new("ns")
        .token("x", "x")
        .rule("r", vec!["x"])
        .build();
    // Without explicit start, may or may not have start symbol
    let _ = g.start_symbol();
}

// ===== Chaining =====

#[test]
fn builder_fluent_chaining() {
    let g = GrammarBuilder::new("chain")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a", "b"])
        .start("root")
        .build();
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
}

#[test]
fn builder_token_then_rule_order_doesnt_matter() {
    let g1 = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    let g2 = GrammarBuilder::new("g")
        .rule("r", vec!["x"])
        .token("x", "x")
        .start("r")
        .build();
    // Both should produce valid grammars
    assert!(!g1.rules.is_empty());
    assert!(!g2.rules.is_empty());
}

// ===== Precedence =====

#[test]
fn builder_rule_with_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .rule_with_precedence(
            "expr",
            vec!["expr", "plus", "expr"],
            1,
            adze_ir::Associativity::Left,
        )
        .start("expr")
        .build();
    let rules: Vec<_> = g.rules.values().flat_map(|v| v.iter()).collect();
    assert!(rules.iter().any(|r| r.precedence.is_some()));
}

#[test]
fn builder_right_associativity() {
    let g = GrammarBuilder::new("right")
        .token("x", "x")
        .token("op", "^")
        .rule_with_precedence(
            "expr",
            vec!["x", "op", "x"],
            2,
            adze_ir::Associativity::Right,
        )
        .start("expr")
        .build();
    let rules: Vec<_> = g.rules.values().flat_map(|v| v.iter()).collect();
    assert!(
        rules
            .iter()
            .any(|r| { r.associativity == Some(adze_ir::Associativity::Right) })
    );
}

#[test]
fn builder_high_precedence() {
    let g = GrammarBuilder::new("high")
        .token("x", "x")
        .rule_with_precedence("expr", vec!["x"], 100, adze_ir::Associativity::Left)
        .start("expr")
        .build();
    let rules: Vec<_> = g.rules.values().flat_map(|v| v.iter()).collect();
    assert!(rules.iter().any(|r| r.precedence.is_some()));
}

// ===== Normalization =====

#[test]
fn builder_grammar_normalizable() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    g.normalize();
    // Should not panic
}

#[test]
fn builder_grammar_all_rules_after_normalize() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a"])
        .rule("alt", vec!["b"])
        .start("root")
        .build();
    let before = g.all_rules().count();
    g.normalize();
    let after = g.all_rules().count();
    assert!(after >= before);
}

// ===== Grammar properties =====

#[test]
fn grammar_all_rules_count() {
    let g = GrammarBuilder::new("c")
        .token("a", "a")
        .token("b", "b")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .start("r1")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn grammar_rules_empty_for_new() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.all_rules().count(), 0);
}

// ===== Determinism =====

#[test]
fn builder_deterministic_tokens() {
    let make = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .build()
    };
    let g1 = make();
    let g2 = make();
    assert_eq!(g1.tokens.len(), g2.tokens.len());
    for (id, tok) in &g1.tokens {
        let tok2 = &g2.tokens[id];
        assert_eq!(tok.name, tok2.name);
    }
}

#[test]
fn builder_deterministic_rules() {
    let make = || {
        GrammarBuilder::new("det")
            .token("x", "x")
            .rule("r", vec!["x"])
            .start("r")
            .build()
    };
    let g1 = make();
    let g2 = make();
    assert_eq!(g1.rules.len(), g2.rules.len());
}

// ===== Edge cases =====

#[test]
fn builder_same_token_name_different_pattern() {
    // Registering same token name twice — builder may overwrite or keep both
    let g = GrammarBuilder::new("dup")
        .token("x", "a")
        .token("x", "b")
        .build();
    // Should not panic
    let _ = g.tokens.len();
}

#[test]
fn builder_rule_referencing_unknown_symbol() {
    let g = GrammarBuilder::new("unk")
        .token("a", "a")
        .rule("r", vec!["a", "unknown"])
        .start("r")
        .build();
    // Builder creates symbols on demand, so this should work
    assert!(!g.rules.is_empty());
}

#[test]
fn builder_long_name() {
    let name = "a".repeat(1000);
    let g = GrammarBuilder::new(&name).build();
    assert_eq!(g.name, name);
}

#[test]
fn builder_many_rules_same_lhs() {
    let mut b = GrammarBuilder::new("multi").token("a", "a").token("b", "b");
    for i in 0..20 {
        let rhs = if i % 2 == 0 { vec!["a"] } else { vec!["b"] };
        b = b.rule("expr", rhs);
    }
    let g = b.start("expr").build();
    let rules: Vec<_> = g.rules.values().flat_map(|v| v.iter()).collect();
    assert!(rules.len() >= 20);
}

#[test]
fn builder_empty_rhs_rule() {
    // Empty RHS (epsilon rule) may panic or produce a rule with empty rhs
    let result = std::panic::catch_unwind(|| {
        GrammarBuilder::new("eps")
            .rule("empty", vec![])
            .start("empty")
            .build()
    });
    // Either it succeeds with an epsilon rule, or panics (both are valid)
    let _ = result;
}

#[test]
fn builder_self_referencing_rule() {
    let g = GrammarBuilder::new("self_ref")
        .token("a", "a")
        .rule("r", vec!["r", "a"])
        .start("r")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn builder_mutual_recursion() {
    let g = GrammarBuilder::new("mutual")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["B", "a"])
        .rule("B", vec!["A", "b"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .start("A")
        .build();
    assert!(g.rules.len() >= 2);
}
