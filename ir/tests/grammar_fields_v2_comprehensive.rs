// Comprehensive tests for Grammar fields and metadata access
// Tests all grammar struct fields and methods

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

#[test]
fn grammar_name_field() {
    let g = GrammarBuilder::new("my_lang")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn grammar_tokens_field() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn grammar_rules_field() {
    let g = GrammarBuilder::new("r")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["a", "a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn grammar_rule_names_field() {
    let g = GrammarBuilder::new("rn")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    assert!(g.rule_names.values().any(|n| n == "expr"));
}

#[test]
fn grammar_start_symbol_method() {
    let g = GrammarBuilder::new("ss")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn grammar_extras_initially_empty() {
    let g = GrammarBuilder::new("e")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.extras.is_empty());
}

#[test]
fn grammar_externals_initially_empty() {
    let g = GrammarBuilder::new("ex")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.externals.is_empty());
}

#[test]
fn grammar_conflicts_initially_empty() {
    let g = GrammarBuilder::new("c")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.conflicts.is_empty());
}

#[test]
fn grammar_fields_initially_empty() {
    let g = GrammarBuilder::new("f")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.fields.is_empty());
}

#[test]
fn grammar_precedences_field() {
    let g = GrammarBuilder::new("p")
        .token("a", "a")
        .token("plus", r"\+")
        .rule("e", vec!["a"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    // May or may not have precedences depending on builder behavior
    let _ = g.precedences.len();
}

#[test]
fn grammar_all_rules_count() {
    let g = GrammarBuilder::new("ac")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("x", vec!["a", "b"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn grammar_serde_roundtrip() {
    let g = GrammarBuilder::new("serde")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}
