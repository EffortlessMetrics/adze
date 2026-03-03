//! Tests for the grammar converter module.

use adze_tool::GrammarConverter;

#[test]
fn create_sample_grammar_has_tokens() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(
        !grammar.tokens.is_empty(),
        "sample grammar should have tokens"
    );
}

#[test]
fn create_sample_grammar_has_rules() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(
        !grammar.rules.is_empty(),
        "sample grammar should have rules"
    );
}

#[test]
fn create_sample_grammar_has_name() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(
        !grammar.name.is_empty(),
        "sample grammar should have a name"
    );
}

#[test]
fn create_sample_grammar_has_start_symbol() {
    let grammar = GrammarConverter::create_sample_grammar();
    let start = grammar.start_symbol();
    assert!(start.is_some(), "sample grammar should have a start symbol");
}

#[test]
fn create_sample_grammar_normalizes() {
    let mut grammar = GrammarConverter::create_sample_grammar();
    let original_rule_count = grammar.rules.len();
    let expanded = grammar.normalize();
    // Normalization should produce at least as many rules (expanding complex symbols)
    assert!(
        expanded.len() >= original_rule_count,
        "normalization should not lose rules"
    );
}
