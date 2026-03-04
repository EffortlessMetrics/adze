// Comprehensive tests for GrammarBuilder fluent API edge cases
// Tests builder patterns and error handling

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

#[test]
fn builder_new_creates_named_grammar() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "test");
}

#[test]
fn builder_single_token_single_rule() {
    let g = GrammarBuilder::new("s1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn builder_multiple_tokens() {
    let g = GrammarBuilder::new("mt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn builder_multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("mr")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_chain_of_nonterminals() {
    let g = GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("s", vec!["m"])
        .rule("m", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_left_recursive() {
    let g = GrammarBuilder::new("lr")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_right_recursive() {
    let g = GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_empty_rhs_gets_epsilon() {
    let g = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("s", vec![])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    // Empty RHS gets Epsilon sentinel
    assert!(!rule.rhs.is_empty());
}

#[test]
fn builder_with_precedence_left() {
    let g = GrammarBuilder::new("pl")
        .token("a", "a")
        .token("plus", r"\+")
        .rule("e", vec!["a"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_with_precedence_right() {
    let g = GrammarBuilder::new("pr")
        .token("a", "a")
        .token("pow", r"\^")
        .rule("e", vec!["a"])
        .rule_with_precedence("e", vec!["e", "pow", "e"], 2, Associativity::Right)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn builder_start_symbol_set() {
    let g = GrammarBuilder::new("st")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn builder_long_rhs() {
    let g = GrammarBuilder::new("long")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d", "a", "b"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 6);
}

#[test]
fn builder_regex_pattern_token() {
    let g = GrammarBuilder::new("re")
        .token("num", "[0-9]+")
        .rule("s", vec!["num"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn builder_grammar_name_preserved() {
    let g = GrammarBuilder::new("my_grammar_name")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "my_grammar_name");
}

#[test]
fn builder_rule_names_populated() {
    let g = GrammarBuilder::new("rn")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(g.rule_names.values().any(|n| n == "start"));
}
