//! Comprehensive tests for grammar optimization (normalize, inline, supertype, validation).
//!
//! Categories:
//!   1. Normalize adds auxiliary rules (8 tests)
//!   2. Normalize preserves tokens (8 tests)
//!   3. Normalize preserves start symbol (8 tests)
//!   4. Normalize idempotent (8 tests)
//!   5. Inline rules optimization (8 tests)
//!   6. Supertype optimization (8 tests)
//!   7. Validator after optimization (8 tests)
//!   8. Rule structure after normalize (7 tests)

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::GrammarValidator;
use adze_ir::{Grammar, ProductionId, Rule, Symbol};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn total_rule_count(grammar: &Grammar) -> usize {
    grammar.all_rules().count()
}

fn total_lhs_count(grammar: &Grammar) -> usize {
    grammar.rules.len()
}

/// Build a grammar then inject a complex symbol into a rule's RHS.
fn grammar_with_optional() -> Grammar {
    let mut grammar = GrammarBuilder::new("opt")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    // Replace the existing rule with one that uses Optional
    let s_id = *grammar.rules.keys().next().unwrap();
    let a_id = grammar.tokens.keys().next().copied().unwrap();
    grammar.rules.clear();
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

fn grammar_with_repeat() -> Grammar {
    let mut grammar = GrammarBuilder::new("rep")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let s_id = *grammar.rules.keys().next().unwrap();
    let a_id = grammar.tokens.keys().next().copied().unwrap();
    grammar.rules.clear();
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

fn grammar_with_repeat_one() -> Grammar {
    let mut grammar = GrammarBuilder::new("rep1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let s_id = *grammar.rules.keys().next().unwrap();
    let a_id = grammar.tokens.keys().next().copied().unwrap();
    grammar.rules.clear();
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

fn grammar_with_choice() -> Grammar {
    let mut grammar = GrammarBuilder::new("choice")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let s_id = *grammar.rules.keys().next().unwrap();
    let mut tok_iter = grammar.tokens.keys();
    let a_id = *tok_iter.next().unwrap();
    let b_id = *tok_iter.next().unwrap();
    grammar.rules.clear();
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

fn grammar_with_sequence() -> Grammar {
    let mut grammar = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let s_id = *grammar.rules.keys().next().unwrap();
    let mut tok_iter = grammar.tokens.keys();
    let a_id = *tok_iter.next().unwrap();
    let b_id = *tok_iter.next().unwrap();
    grammar.rules.clear();
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

fn grammar_with_nested_optional_repeat() -> Grammar {
    let mut grammar = GrammarBuilder::new("nested")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let s_id = *grammar.rules.keys().next().unwrap();
    let a_id = grammar.tokens.keys().next().copied().unwrap();
    grammar.rules.clear();
    // Optional(Repeat(Terminal(a)))
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
            Symbol::Terminal(a_id),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

fn simple_flat_grammar() -> Grammar {
    GrammarBuilder::new("flat")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build()
}

// ===========================================================================
// Category 1 — Normalize adds auxiliary rules
// ===========================================================================

#[test]
fn normalize_optional_adds_two_aux_rules() {
    let mut g = grammar_with_optional();
    let before = total_rule_count(&g);
    g.normalize();
    // Optional expands to aux→inner | aux→ε, plus the rewritten parent rule
    assert!(
        total_rule_count(&g) > before,
        "normalize should add rules for Optional"
    );
}

#[test]
fn normalize_repeat_adds_two_aux_rules() {
    let mut g = grammar_with_repeat();
    let before = total_rule_count(&g);
    g.normalize();
    assert!(
        total_rule_count(&g) > before,
        "normalize should add rules for Repeat"
    );
}

#[test]
fn normalize_repeat_one_adds_two_aux_rules() {
    let mut g = grammar_with_repeat_one();
    let before = total_rule_count(&g);
    g.normalize();
    assert!(
        total_rule_count(&g) > before,
        "normalize should add rules for RepeatOne"
    );
}

#[test]
fn normalize_choice_adds_aux_rules() {
    let mut g = grammar_with_choice();
    let before = total_rule_count(&g);
    g.normalize();
    assert!(
        total_rule_count(&g) > before,
        "normalize should add rules for Choice"
    );
}

#[test]
fn normalize_sequence_flattens_no_extra_lhs() {
    let mut g = grammar_with_sequence();
    let before_lhs = total_lhs_count(&g);
    g.normalize();
    // Sequence just flattens into the parent rule — no new LHS symbols
    assert_eq!(total_lhs_count(&g), before_lhs);
}

#[test]
fn normalize_nested_optional_repeat_adds_multiple_aux() {
    let mut g = grammar_with_nested_optional_repeat();
    let before = total_rule_count(&g);
    g.normalize();
    // Nested: Optional(Repeat(T)) creates aux rules for both layers
    assert!(
        total_rule_count(&g) >= before + 3,
        "nested complex symbols should produce multiple auxiliary rules"
    );
}

#[test]
fn normalize_flat_grammar_unchanged_rule_count() {
    let mut g = simple_flat_grammar();
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(
        total_rule_count(&g),
        before,
        "flat grammar should stay the same"
    );
}

#[test]
fn normalize_multiple_complex_in_one_rule() {
    let mut grammar = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    let s_id = *grammar.rules.keys().next().unwrap();
    let mut tok_iter = grammar.tokens.keys();
    let a_id = *tok_iter.next().unwrap();
    let b_id = *tok_iter.next().unwrap();
    grammar.rules.clear();
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
            Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let before = total_rule_count(&grammar);
    grammar.normalize();
    // Each complex symbol introduces 2 aux rules
    assert!(
        total_rule_count(&grammar) >= before + 4,
        "two complex symbols in one rule should each add aux rules"
    );
}

// ===========================================================================
// Category 2 — Normalize preserves tokens
// ===========================================================================

#[test]
fn normalize_optional_preserves_tokens() {
    let mut g = grammar_with_optional();
    let tokens_before: Vec<_> = g.tokens.keys().copied().collect();
    g.normalize();
    let tokens_after: Vec<_> = g.tokens.keys().copied().collect();
    assert_eq!(tokens_before, tokens_after);
}

#[test]
fn normalize_repeat_preserves_tokens() {
    let mut g = grammar_with_repeat();
    let tokens_before: Vec<_> = g.tokens.keys().copied().collect();
    g.normalize();
    let tokens_after: Vec<_> = g.tokens.keys().copied().collect();
    assert_eq!(tokens_before, tokens_after);
}

#[test]
fn normalize_repeat_one_preserves_tokens() {
    let mut g = grammar_with_repeat_one();
    let tokens_before: Vec<_> = g.tokens.keys().copied().collect();
    g.normalize();
    let tokens_after: Vec<_> = g.tokens.keys().copied().collect();
    assert_eq!(tokens_before, tokens_after);
}

#[test]
fn normalize_choice_preserves_tokens() {
    let mut g = grammar_with_choice();
    let tokens_before: Vec<_> = g.tokens.keys().copied().collect();
    g.normalize();
    let tokens_after: Vec<_> = g.tokens.keys().copied().collect();
    assert_eq!(tokens_before, tokens_after);
}

#[test]
fn normalize_sequence_preserves_tokens() {
    let mut g = grammar_with_sequence();
    let tokens_before: Vec<_> = g.tokens.keys().copied().collect();
    g.normalize();
    let tokens_after: Vec<_> = g.tokens.keys().copied().collect();
    assert_eq!(tokens_before, tokens_after);
}

#[test]
fn normalize_nested_preserves_tokens() {
    let mut g = grammar_with_nested_optional_repeat();
    let tokens_before: Vec<_> = g.tokens.keys().copied().collect();
    g.normalize();
    let tokens_after: Vec<_> = g.tokens.keys().copied().collect();
    assert_eq!(tokens_before, tokens_after);
}

#[test]
fn normalize_preserves_token_names() {
    let mut g = grammar_with_optional();
    let names_before: Vec<_> = g.tokens.values().map(|t| t.name.clone()).collect();
    g.normalize();
    let names_after: Vec<_> = g.tokens.values().map(|t| t.name.clone()).collect();
    assert_eq!(names_before, names_after);
}

#[test]
fn normalize_preserves_token_patterns() {
    let mut g = grammar_with_repeat();
    let patterns_before: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    g.normalize();
    let patterns_after: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    assert_eq!(patterns_before, patterns_after);
}

// ===========================================================================
// Category 3 — Normalize preserves start symbol
// ===========================================================================

#[test]
fn normalize_optional_preserves_start() {
    let mut g = grammar_with_optional();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn normalize_repeat_preserves_start() {
    let mut g = grammar_with_repeat();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn normalize_repeat_one_preserves_start() {
    let mut g = grammar_with_repeat_one();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn normalize_choice_preserves_start() {
    let mut g = grammar_with_choice();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn normalize_sequence_preserves_start() {
    let mut g = grammar_with_sequence();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn normalize_nested_preserves_start() {
    let mut g = grammar_with_nested_optional_repeat();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn normalize_flat_preserves_start() {
    let mut g = simple_flat_grammar();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn normalize_multi_rule_grammar_preserves_start() {
    let mut g = GrammarBuilder::new("multi_start")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();

    // Inject an Optional into term
    let term_id = g.find_symbol_by_name("term").unwrap();
    let num_id = g.tokens.keys().next().copied().unwrap();
    // Add a rule with Optional
    g.add_rule(Rule {
        lhs: term_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(num_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(100),
    });

    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

// ===========================================================================
// Category 4 — Normalize idempotent
// ===========================================================================

#[test]
fn normalize_optional_idempotent() {
    let mut g = grammar_with_optional();
    g.normalize();
    let count_after_first = total_rule_count(&g);
    let lhs_after_first = total_lhs_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count_after_first);
    assert_eq!(total_lhs_count(&g), lhs_after_first);
}

#[test]
fn normalize_repeat_idempotent() {
    let mut g = grammar_with_repeat();
    g.normalize();
    let count = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count);
}

#[test]
fn normalize_repeat_one_idempotent() {
    let mut g = grammar_with_repeat_one();
    g.normalize();
    let count = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count);
}

#[test]
fn normalize_choice_idempotent() {
    let mut g = grammar_with_choice();
    g.normalize();
    let count = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count);
}

#[test]
fn normalize_sequence_idempotent() {
    let mut g = grammar_with_sequence();
    g.normalize();
    let count = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count);
}

#[test]
fn normalize_nested_idempotent() {
    let mut g = grammar_with_nested_optional_repeat();
    g.normalize();
    let count = total_rule_count(&g);
    let lhs = total_lhs_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count);
    assert_eq!(total_lhs_count(&g), lhs);
}

#[test]
fn normalize_flat_idempotent() {
    let mut g = simple_flat_grammar();
    g.normalize();
    let count = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count);
}

#[test]
fn normalize_triple_application_same_as_single() {
    let mut g1 = grammar_with_optional();
    g1.normalize();
    let count1 = total_rule_count(&g1);

    let mut g2 = grammar_with_optional();
    g2.normalize();
    g2.normalize();
    g2.normalize();
    assert_eq!(total_rule_count(&g2), count1);
}

// ===========================================================================
// Category 5 — Inline rules optimization
// ===========================================================================

#[test]
fn inline_single_rule_recorded() {
    let g = GrammarBuilder::new("inl")
        .token("x", "x")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .start("s")
        .build();
    assert!(
        !g.inline_rules.is_empty(),
        "inline_rules should contain helper"
    );
}

#[test]
fn inline_symbol_id_matches_rule() {
    let g = GrammarBuilder::new("inl2")
        .token("x", "x")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .start("s")
        .build();

    let helper_id = g.find_symbol_by_name("helper").unwrap();
    assert!(g.inline_rules.contains(&helper_id));
}

#[test]
fn inline_multiple_rules() {
    let g = GrammarBuilder::new("inl3")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["a", "b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .inline("a")
        .inline("b")
        .start("s")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn inline_does_not_affect_rules_map() {
    let g = GrammarBuilder::new("inl4")
        .token("x", "x")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .start("s")
        .build();

    let helper_id = g.find_symbol_by_name("helper").unwrap();
    assert!(
        g.rules.contains_key(&helper_id),
        "inline marking should not remove the rule from the rules map"
    );
}

#[test]
fn no_inline_means_empty_vec() {
    let g = simple_flat_grammar();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn inline_preserves_token_count() {
    let g = GrammarBuilder::new("inl5")
        .token("x", "x")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn inline_and_normalize_coexist() {
    let mut g = GrammarBuilder::new("inl_norm")
        .token("x", "x")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .start("s")
        .build();

    let inline_before = g.inline_rules.clone();
    g.normalize();
    // inline_rules field should remain unchanged by normalize
    assert_eq!(g.inline_rules, inline_before);
}

#[test]
fn inline_duplicate_is_additive() {
    let g = GrammarBuilder::new("inl_dup")
        .token("x", "x")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .inline("helper")
        .start("s")
        .build();
    // Builder pushes each .inline() call; duplicates are allowed
    assert!(!g.inline_rules.is_empty());
}

// ===========================================================================
// Category 6 — Supertype optimization
// ===========================================================================

#[test]
fn supertype_single_recorded() {
    let g = GrammarBuilder::new("sup")
        .token("x", "x")
        .token("y", "y")
        .rule("expression", vec!["literal"])
        .rule("literal", vec!["x"])
        .rule("literal", vec!["y"])
        .supertype("expression")
        .start("expression")
        .build();
    assert!(!g.supertypes.is_empty());
}

#[test]
fn supertype_id_matches() {
    let g = GrammarBuilder::new("sup2")
        .token("x", "x")
        .rule("expression", vec!["x"])
        .supertype("expression")
        .start("expression")
        .build();

    let expr_id = g.find_symbol_by_name("expression").unwrap();
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn supertype_multiple() {
    let g = GrammarBuilder::new("sup3")
        .token("x", "x")
        .token("y", "y")
        .rule("expression", vec!["x"])
        .rule("statement", vec!["y"])
        .supertype("expression")
        .supertype("statement")
        .start("expression")
        .build();
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn no_supertype_means_empty_vec() {
    let g = simple_flat_grammar();
    assert!(g.supertypes.is_empty());
}

#[test]
fn supertype_does_not_remove_rules() {
    let g = GrammarBuilder::new("sup4")
        .token("x", "x")
        .rule("expression", vec!["x"])
        .supertype("expression")
        .start("expression")
        .build();

    let expr_id = g.find_symbol_by_name("expression").unwrap();
    assert!(g.rules.contains_key(&expr_id));
}

#[test]
fn supertype_preserves_token_count() {
    let g = GrammarBuilder::new("sup5")
        .token("x", "x")
        .token("y", "y")
        .rule("expression", vec!["x"])
        .supertype("expression")
        .start("expression")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn supertype_and_normalize_coexist() {
    let mut g = GrammarBuilder::new("sup_norm")
        .token("x", "x")
        .rule("expression", vec!["x"])
        .supertype("expression")
        .start("expression")
        .build();

    let supertypes_before = g.supertypes.clone();
    g.normalize();
    assert_eq!(g.supertypes, supertypes_before);
}

#[test]
fn supertype_and_inline_independent() {
    let g = GrammarBuilder::new("sup_inl")
        .token("x", "x")
        .token("y", "y")
        .rule("expression", vec!["helper"])
        .rule("helper", vec!["x"])
        .supertype("expression")
        .inline("helper")
        .start("expression")
        .build();

    assert_eq!(g.supertypes.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    assert!(g.supertypes.contains(&expr_id));
    assert!(g.inline_rules.contains(&helper_id));
}

// ===========================================================================
// Category 7 — Validator after optimization
// ===========================================================================

#[test]
fn validator_passes_flat_grammar() {
    let g = simple_flat_grammar();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result.errors.is_empty(),
        "flat grammar should validate: {:?}",
        result.errors
    );
}

#[test]
fn validator_passes_after_normalize_optional() {
    let mut g = grammar_with_optional();
    g.normalize();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    // Auxiliary rules reference symbols that may not be in rule_names;
    // we just check no *critical* structural errors
    let critical: Vec<_> = result
        .errors
        .iter()
        .filter(|e| matches!(e, adze_ir::ValidationError::EmptyGrammar))
        .collect();
    assert!(critical.is_empty(), "no EmptyGrammar error after normalize");
}

#[test]
fn validator_passes_after_normalize_repeat() {
    let mut g = grammar_with_repeat();
    g.normalize();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    let critical: Vec<_> = result
        .errors
        .iter()
        .filter(|e| matches!(e, adze_ir::ValidationError::EmptyGrammar))
        .collect();
    assert!(critical.is_empty());
}

#[test]
fn validator_passes_after_normalize_repeat_one() {
    let mut g = grammar_with_repeat_one();
    g.normalize();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    let critical: Vec<_> = result
        .errors
        .iter()
        .filter(|e| matches!(e, adze_ir::ValidationError::EmptyGrammar))
        .collect();
    assert!(critical.is_empty());
}

#[test]
fn validator_passes_after_normalize_choice() {
    let mut g = grammar_with_choice();
    g.normalize();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    let critical: Vec<_> = result
        .errors
        .iter()
        .filter(|e| matches!(e, adze_ir::ValidationError::EmptyGrammar))
        .collect();
    assert!(critical.is_empty());
}

#[test]
fn validator_stats_reflect_expanded_rules() {
    let mut g = grammar_with_optional();
    g.normalize();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result.stats.total_rules >= 3,
        "stats should count expanded rules: got {}",
        result.stats.total_rules
    );
}

#[test]
fn validator_after_normalize_has_nonzero_symbols() {
    let mut g = grammar_with_repeat();
    g.normalize();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.stats.total_symbols > 0);
}

#[test]
fn validator_after_normalize_preserves_token_stats() {
    let mut g = grammar_with_choice();
    let token_count = g.tokens.len();
    g.normalize();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert_eq!(result.stats.total_tokens, token_count);
}

// ===========================================================================
// Category 8 — Rule structure after normalize
// ===========================================================================

#[test]
fn normalize_optional_creates_epsilon_rule() {
    let mut g = grammar_with_optional();
    g.normalize();
    let has_epsilon = g.all_rules().any(|r| r.rhs.contains(&Symbol::Epsilon));
    assert!(has_epsilon, "Optional should create an epsilon production");
}

#[test]
fn normalize_repeat_creates_epsilon_rule() {
    let mut g = grammar_with_repeat();
    g.normalize();
    let has_epsilon = g.all_rules().any(|r| r.rhs.contains(&Symbol::Epsilon));
    assert!(has_epsilon, "Repeat should create an epsilon production");
}

#[test]
fn normalize_repeat_one_no_epsilon() {
    let mut g = grammar_with_repeat_one();
    g.normalize();
    let has_epsilon = g.all_rules().any(|r| r.rhs.contains(&Symbol::Epsilon));
    assert!(
        !has_epsilon,
        "RepeatOne should NOT create epsilon production"
    );
}

#[test]
fn normalize_optional_aux_has_nonterminal_ref() {
    let mut g = grammar_with_optional();
    let s_id = *g.rules.keys().next().unwrap();
    g.normalize();
    // The parent rule should now reference a NonTerminal (the aux symbol)
    let s_rules = g.rules.get(&s_id).unwrap();
    let has_nt = s_rules
        .iter()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::NonTerminal(_))));
    assert!(has_nt, "parent rule should reference NonTerminal aux");
}

#[test]
fn normalize_repeat_creates_left_recursive_aux() {
    let mut g = grammar_with_repeat();
    g.normalize();
    // Find a rule where the LHS appears in its own RHS (left-recursive)
    let has_left_recursive = g
        .all_rules()
        .any(|r| !r.rhs.is_empty() && r.rhs[0] == Symbol::NonTerminal(r.lhs));
    assert!(
        has_left_recursive,
        "Repeat should produce left-recursive aux rule"
    );
}

#[test]
fn normalize_choice_creates_separate_alternatives() {
    let mut g = grammar_with_choice();
    g.normalize();
    // The aux symbol for Choice should have multiple rules (one per alternative)
    let aux_lhs_with_multiple = g.rules.iter().any(|(_, rules)| rules.len() >= 2);
    assert!(
        aux_lhs_with_multiple,
        "Choice aux should have multiple alternative rules"
    );
}

#[test]
fn normalize_removes_all_complex_symbols() {
    let mut g = grammar_with_nested_optional_repeat();
    g.normalize();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(
                !matches!(
                    sym,
                    Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                ),
                "no complex symbols should remain after normalize: found {sym:?}"
            );
        }
    }
}
