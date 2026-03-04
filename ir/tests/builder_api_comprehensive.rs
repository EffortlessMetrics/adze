//! Comprehensive tests for GrammarBuilder fluent API.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol};

// ============================================================================
// Tests: Basic construction
// ============================================================================

#[test]
fn builder_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.name, "empty");
}

#[test]
fn builder_with_one_token() {
    let g = GrammarBuilder::new("t").token("a", "a").build();
    assert_eq!(g.tokens.len(), 1);
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.name, "a");
}

#[test]
fn builder_with_multiple_tokens() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn builder_with_rule() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn builder_with_multiple_rules() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let start_id = g.start_symbol().unwrap();
    let rules = g.get_rules_for_symbol(start_id).unwrap();
    assert_eq!(rules.len(), 2);
}

// ============================================================================
// Tests: Start symbol
// ============================================================================

#[test]
fn start_symbol_set() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn start_symbol_not_set() {
    let g = GrammarBuilder::new("t").token("a", "a").build();
    assert!(g.start_symbol().is_none());
}

// ============================================================================
// Tests: Rule RHS types
// ============================================================================

#[test]
fn rule_rhs_with_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let start_id = g.start_symbol().unwrap();
    let rules = g.get_rules_for_symbol(start_id).unwrap();
    assert_eq!(rules.len(), 1);
    match &rules[0].rhs[0] {
        Symbol::Terminal(_) => {}
        other => panic!("Expected Terminal, got {:?}", other),
    }
}

#[test]
fn rule_rhs_with_nonterminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let start_id = g.start_symbol().unwrap();
    let rules = g.get_rules_for_symbol(start_id).unwrap();
    match &rules[0].rhs[0] {
        Symbol::NonTerminal(_) => {}
        other => panic!("Expected NonTerminal, got {:?}", other),
    }
}

#[test]
fn rule_rhs_epsilon() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .start("start")
        .build();
    let start_id = g.start_symbol().unwrap();
    let rules = g.get_rules_for_symbol(start_id).unwrap();
    assert!(rules[0].rhs.contains(&Symbol::Epsilon));
}

// ============================================================================
// Tests: Precedence
// ============================================================================

#[test]
fn rule_with_precedence() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule_with_precedence("start", vec!["a"], 5, Associativity::Left)
        .start("start")
        .build();
    let start_id = g.start_symbol().unwrap();
    let rules = g.get_rules_for_symbol(start_id).unwrap();
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(5)));
    assert_eq!(rules[0].associativity, Some(Associativity::Left));
}

// ============================================================================
// Tests: Fragile token
// ============================================================================

#[test]
fn fragile_token_flagged() {
    let g = GrammarBuilder::new("t")
        .fragile_token("kw", "keyword")
        .build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert!(tok.fragile, "Fragile token should have fragile=true");
}

#[test]
fn regular_token_not_fragile() {
    let g = GrammarBuilder::new("t").token("a", "a").build();
    let (_, tok) = g.tokens.iter().next().unwrap();
    assert!(!tok.fragile);
}

// ============================================================================
// Tests: Grammar properties
// ============================================================================

#[test]
fn grammar_name_preserved() {
    let g = GrammarBuilder::new("my_grammar").build();
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn all_rules_count() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("other", vec!["a", "b"])
        .start("start")
        .build();
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn rule_names_populated() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(g.rule_names.values().any(|n| n == "start"));
}

// ============================================================================
// Tests: Preset grammars
// ============================================================================

#[test]
fn python_like_preset_builds() {
    let g = GrammarBuilder::python_like();
    assert_eq!(g.name, "python_like");
    assert!(g.start_symbol().is_some());
    assert!(!g.tokens.is_empty());
}

#[test]
fn javascript_like_preset_builds() {
    let g = GrammarBuilder::javascript_like();
    assert_eq!(g.name, "javascript_like");
    assert!(g.start_symbol().is_some());
    assert!(!g.tokens.is_empty());
}

// ============================================================================
// Tests: Chaining
// ============================================================================

#[test]
fn builder_fluent_chaining() {
    let g = GrammarBuilder::new("t")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("factor", vec!["num"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 3);
    assert_eq!(g.all_rules().count(), 6);
}

// ============================================================================
// Tests: Serde roundtrip
// ============================================================================

#[test]
fn grammar_serde_roundtrip() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: adze_ir::Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn grammar_serde_preserves_rules() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}
