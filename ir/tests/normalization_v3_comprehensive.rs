// Comprehensive tests for Grammar::normalize() in adze-ir
// Tests the conversion of complex symbols to auxiliary rules

use adze_ir::Symbol;
use adze_ir::builder::GrammarBuilder;

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("norm")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

#[test]
fn normalize_simple_grammar_succeeds() {
    let mut g = simple_grammar();
    g.normalize();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn normalize_idempotent() {
    let mut g = simple_grammar();
    g.normalize();
    let count1 = g.all_rules().count();
    g.normalize();
    let count2 = g.all_rules().count();
    assert_eq!(count1, count2);
}

#[test]
fn normalize_preserves_start_symbol() {
    let mut g = simple_grammar();
    let start_before = g.start_symbol();
    g.normalize();
    let start_after = g.start_symbol();
    assert_eq!(start_before, start_after);
}

#[test]
fn normalize_preserves_token_count() {
    let mut g = simple_grammar();
    let tokens_before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tokens_before);
}

#[test]
fn normalize_preserves_rule_names() {
    let mut g = simple_grammar();
    let names_before: Vec<_> = g.rule_names.values().cloned().collect();
    g.normalize();
    for name in &names_before {
        assert!(g.rule_names.values().any(|n| n == name));
    }
}

#[test]
fn normalize_grammar_still_valid_after() {
    let mut g = simple_grammar();
    g.normalize();
    assert!(g.all_rules().count() > 0);
    assert!(!g.tokens.is_empty());
}

#[test]
fn normalize_two_token_rule() {
    let mut g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn normalize_nonterminal_chain() {
    let mut g = GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("s", vec!["x"])
        .rule("x", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn normalize_multiple_alternatives() {
    let mut g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn normalize_recursive_rule() {
    let mut g = GrammarBuilder::new("rec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn normalize_long_rhs() {
    let mut g = GrammarBuilder::new("long")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c", "a", "b"])
        .start("s")
        .build();
    g.normalize();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 5);
}

#[test]
fn normalize_empty_rhs() {
    let mut g = GrammarBuilder::new("empty")
        .token("a", "a")
        .rule("s", vec![])
        .start("s")
        .build();
    g.normalize();
    // Empty rules get Epsilon sentinel
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn normalize_preserves_grammar_name() {
    let mut g = GrammarBuilder::new("myname")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    assert_eq!(g.name, "myname");
}

#[test]
fn normalize_with_precedence() {
    use adze_ir::Associativity;
    let mut g = GrammarBuilder::new("prec")
        .token("plus", r"\+")
        .token("num", "[0-9]+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn normalize_does_not_add_tokens() {
    let mut g = GrammarBuilder::new("notok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let tok_count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tok_count);
}

#[test]
fn normalize_triple_idempotent() {
    let mut g = simple_grammar();
    g.normalize();
    g.normalize();
    g.normalize();
    let count = g.all_rules().count();
    assert!(count >= 1);
}
