//! Comprehensive tests for Grammar rule_names and start_symbol patterns.

use adze_ir::builder::GrammarBuilder;

// ── Rule names ──

#[test]
fn rule_names_contains_start() {
    let g = GrammarBuilder::new("rn")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.rule_names.values().any(|n| n == "s"));
}

#[test]
fn rule_names_contains_all() {
    let g = GrammarBuilder::new("rn2")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("s", vec!["expr"])
        .start("s")
        .build();
    assert!(g.rule_names.values().any(|n| n == "s"));
    assert!(g.rule_names.values().any(|n| n == "expr"));
}

#[test]
fn rule_names_count() {
    let g = GrammarBuilder::new("cnt")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.rule_names.len() >= 3);
}

// ── Start symbol ──

#[test]
fn start_symbol_some() {
    let g = GrammarBuilder::new("ss")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn start_symbol_matches_name() {
    let g = GrammarBuilder::new("ss2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let start_id = g.start_symbol().unwrap();
    assert!(g.rule_names.get(&start_id).is_some_and(|n| n == "start"));
}

// ── Token names ──

#[test]
fn single_token_in_tokens() {
    let g = GrammarBuilder::new("tok")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn multiple_tokens() {
    let g = GrammarBuilder::new("mtok")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("s", vec!["num"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

// ── Rules collection ──

#[test]
fn rules_count_single() {
    let g = GrammarBuilder::new("rc")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn rules_count_multiple() {
    let g = GrammarBuilder::new("rcm")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

// ── Rule lhs and rhs ──

#[test]
fn rule_lhs_is_valid() {
    let g = GrammarBuilder::new("lhs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        let _ = rule.lhs;
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

// ── Normalize effects ──

#[test]
fn normalize_preserves_rule_names() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let names_before: Vec<String> = g.rule_names.values().cloned().collect();
    g.normalize();
    for name in &names_before {
        assert!(g.rule_names.values().any(|n| n == name));
    }
}

#[test]
fn normalize_preserves_start_symbol() {
    let mut g = GrammarBuilder::new("ns")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let start = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start);
}

// ── Clone preservation ──

#[test]
fn clone_preserves_rule_names() {
    let g = GrammarBuilder::new("cp")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.rule_names.len(), g2.rule_names.len());
}

#[test]
fn clone_preserves_start() {
    let g = GrammarBuilder::new("cs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

// ── Serialization preserves ──

#[test]
fn serde_preserves_rule_names_count() {
    let g = GrammarBuilder::new("ser")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.rule_names.len(), g2.rule_names.len());
}

#[test]
fn serde_preserves_start() {
    let g = GrammarBuilder::new("serst")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

// ── Grammar with many rules ──

#[test]
fn many_rules_all_named() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..10 {
        let tok = format!("t{}", i);
        b = b.token(&tok, &tok);
    }
    for i in 0..10 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    assert!(g.rule_names.values().any(|n| n == "s"));
}
