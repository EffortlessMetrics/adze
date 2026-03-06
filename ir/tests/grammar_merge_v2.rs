//! Tests for grammar merge/combine operations.
//!
//! Covers token combining, rule merging, equality, subsetting,
//! diff detection, normalization comparison, and builder patterns.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, Rule, Symbol, SymbolId, TokenPattern};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all token names from a grammar.
fn token_names(grammar: &Grammar) -> HashSet<String> {
    grammar.tokens.values().map(|t| t.name.clone()).collect()
}

/// Collect all rule LHS names (non-terminal names that have productions).
fn rule_lhs_names(grammar: &Grammar) -> HashSet<String> {
    grammar
        .rules
        .keys()
        .filter_map(|id| grammar.rule_names.get(id).cloned())
        .collect()
}

/// Total number of individual productions across all LHS symbols.
fn total_production_count(grammar: &Grammar) -> usize {
    grammar.rules.values().map(|v| v.len()).sum()
}

/// Build a minimal arithmetic grammar.
fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Build a minimal boolean grammar.
fn bool_grammar() -> Grammar {
    GrammarBuilder::new("bool")
        .token("TRUE", "true")
        .token("FALSE", "false")
        .token("AND", "&&")
        .token("OR", "||")
        .rule("bexpr", vec!["bexpr", "AND", "bexpr"])
        .rule("bexpr", vec!["bexpr", "OR", "bexpr"])
        .rule("bexpr", vec!["TRUE"])
        .rule("bexpr", vec!["FALSE"])
        .start("bexpr")
        .build()
}

/// Build a small string-only grammar.
fn string_grammar() -> Grammar {
    GrammarBuilder::new("strings")
        .token("STRING", r#""[^"]*""#)
        .token("+", "+")
        .rule("concat", vec!["concat", "+", "STRING"])
        .rule("concat", vec!["STRING"])
        .start("concat")
        .build()
}

// ===========================================================================
// 1. Combine two grammars' tokens (8 tests)
// ===========================================================================

#[test]
fn test_combine_tokens_disjoint_sets() {
    let a = arith_grammar();
    let b = bool_grammar();
    let a_names = token_names(&a);
    let b_names = token_names(&b);
    assert!(
        a_names.is_disjoint(&b_names),
        "arith and bool tokens should be disjoint"
    );
}

#[test]
fn test_combine_tokens_union_count() {
    let a = arith_grammar();
    let b = bool_grammar();
    let combined: HashSet<_> = token_names(&a).union(&token_names(&b)).cloned().collect();
    assert_eq!(combined.len(), a.tokens.len() + b.tokens.len());
}

#[test]
fn test_combine_tokens_overlapping_name() {
    let a = arith_grammar();
    let b = string_grammar();
    // Both define "+"
    let a_names = token_names(&a);
    let b_names = token_names(&b);
    let overlap: HashSet<_> = a_names.intersection(&b_names).cloned().collect();
    assert!(overlap.contains("+"));
    assert_eq!(overlap.len(), 1);
}

#[test]
fn test_combine_tokens_empty_grammar_with_nonempty() {
    let empty = GrammarBuilder::new("empty").build();
    let a = arith_grammar();
    let combined: HashSet<_> = token_names(&empty)
        .union(&token_names(&a))
        .cloned()
        .collect();
    assert_eq!(combined.len(), a.tokens.len());
}

#[test]
fn test_combine_tokens_both_empty() {
    let e1 = GrammarBuilder::new("e1").build();
    let e2 = GrammarBuilder::new("e2").build();
    let combined: HashSet<_> = token_names(&e1).union(&token_names(&e2)).cloned().collect();
    assert!(combined.is_empty());
}

#[test]
fn test_combine_tokens_superset_check() {
    let a = arith_grammar();
    let b = string_grammar();
    let combined: HashSet<_> = token_names(&a).union(&token_names(&b)).cloned().collect();
    assert!(token_names(&a).is_subset(&combined));
    assert!(token_names(&b).is_subset(&combined));
}

#[test]
fn test_combine_tokens_preserves_patterns() {
    let a = arith_grammar();
    for token in a.tokens.values() {
        if token.name == "NUMBER" {
            assert!(matches!(&token.pattern, TokenPattern::Regex(p) if p.contains(r"\d")));
        }
    }
}

#[test]
fn test_combine_tokens_no_duplicates_in_single_grammar() {
    let a = arith_grammar();
    let names: Vec<_> = a.tokens.values().map(|t| &t.name).collect();
    let unique: HashSet<_> = names.iter().collect();
    assert_eq!(
        names.len(),
        unique.len(),
        "single grammar should have no duplicate token names"
    );
}

// ===========================================================================
// 2. Merge rule sets (8 tests)
// ===========================================================================

#[test]
fn test_merge_rules_disjoint_lhs() {
    let a = arith_grammar();
    let b = bool_grammar();
    let a_lhs = rule_lhs_names(&a);
    let b_lhs = rule_lhs_names(&b);
    assert!(a_lhs.is_disjoint(&b_lhs));
}

#[test]
fn test_merge_rules_combined_lhs_count() {
    let a = arith_grammar();
    let b = bool_grammar();
    let combined: HashSet<_> = rule_lhs_names(&a)
        .union(&rule_lhs_names(&b))
        .cloned()
        .collect();
    assert_eq!(combined.len(), a.rules.len() + b.rules.len());
}

#[test]
fn test_merge_rules_total_productions() {
    let a = arith_grammar();
    let b = bool_grammar();
    let total = total_production_count(&a) + total_production_count(&b);
    assert_eq!(total, 3 + 4); // arith has 3 prods, bool has 4
}

#[test]
fn test_merge_rules_shared_structure_different_names() {
    // Two grammars with identical shape but different symbol names
    let g1 = GrammarBuilder::new("g1")
        .token("A", "a")
        .rule("start1", vec!["A"])
        .start("start1")
        .build();
    let g2 = GrammarBuilder::new("g2")
        .token("B", "b")
        .rule("start2", vec!["B"])
        .start("start2")
        .build();
    assert_eq!(total_production_count(&g1), total_production_count(&g2));
}

#[test]
fn test_merge_rules_single_rule_grammar() {
    let g = GrammarBuilder::new("tiny")
        .token("X", "x")
        .rule("top", vec!["X"])
        .start("top")
        .build();
    assert_eq!(g.rules.len(), 1);
    assert_eq!(total_production_count(&g), 1);
}

#[test]
fn test_merge_rules_multiple_alternatives() {
    let g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("item", vec!["A"])
        .rule("item", vec!["B"])
        .rule("item", vec!["C"])
        .start("item")
        .build();
    assert_eq!(g.rules.len(), 1);
    assert_eq!(total_production_count(&g), 3);
}

#[test]
fn test_merge_rules_preserves_lhs_identity() {
    let a = arith_grammar();
    // All rules for "expr" share the same LHS SymbolId
    let expr_id = a.find_symbol_by_name("expr").unwrap();
    let rules = a.get_rules_for_symbol(expr_id).unwrap();
    for rule in rules {
        assert_eq!(rule.lhs, expr_id);
    }
}

#[test]
fn test_merge_rules_empty_rhs_is_epsilon() {
    let g = GrammarBuilder::new("nullable")
        .rule("maybe", vec![])
        .start("maybe")
        .build();
    let maybe_id = g.find_symbol_by_name("maybe").unwrap();
    let rules = g.get_rules_for_symbol(maybe_id).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

// ===========================================================================
// 3. Grammar equality / comparison (8 tests)
// ===========================================================================

#[test]
fn test_equality_same_grammar_twice() {
    let g1 = arith_grammar();
    let g2 = arith_grammar();
    assert_eq!(g1, g2);
}

#[test]
fn test_equality_different_names() {
    let g1 = GrammarBuilder::new("alpha")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("beta")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    assert_ne!(g1, g2, "grammars with different names should not be equal");
}

#[test]
fn test_equality_different_tokens() {
    let g1 = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("g")
        .token("B", "b")
        .rule("s", vec!["B"])
        .start("s")
        .build();
    assert_ne!(g1, g2);
}

#[test]
fn test_equality_different_rule_count() {
    let g1 = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    assert_ne!(g1, g2);
}

#[test]
fn test_equality_empty_grammars() {
    let g1 = GrammarBuilder::new("empty").build();
    let g2 = GrammarBuilder::new("empty").build();
    assert_eq!(g1, g2);
}

#[test]
fn test_equality_order_dependent_rules() {
    // IndexMap preserves insertion order — adding in different order yields !=
    let g1 = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["B"])
        .rule("s", vec!["A"])
        .start("s")
        .build();
    // Productions for same LHS are ordered by insertion
    assert_ne!(g1, g2, "different production order should differ");
}

#[test]
fn test_equality_clone_is_equal() {
    let g1 = arith_grammar();
    let g2 = g1.clone();
    assert_eq!(g1, g2);
}

#[test]
fn test_equality_default_grammar() {
    let g1 = Grammar::default();
    let g2 = Grammar::default();
    assert_eq!(g1, g2);
    assert!(g1.rules.is_empty());
}

// ===========================================================================
// 4. Grammar subsetting (8 tests)
// ===========================================================================

#[test]
fn test_subset_tokens_strict_subset() {
    let small = GrammarBuilder::new("small")
        .token("NUMBER", r"\d+")
        .rule("val", vec!["NUMBER"])
        .start("val")
        .build();
    let big = arith_grammar();
    let small_names = token_names(&small);
    let big_names = token_names(&big);
    assert!(small_names.is_subset(&big_names));
}

#[test]
fn test_subset_tokens_equal_is_subset() {
    let a = arith_grammar();
    let b = arith_grammar();
    assert!(token_names(&a).is_subset(&token_names(&b)));
}

#[test]
fn test_subset_tokens_not_subset() {
    let a = arith_grammar();
    let b = bool_grammar();
    assert!(!token_names(&a).is_subset(&token_names(&b)));
}

#[test]
fn test_subset_rules_strict_subset() {
    let small = GrammarBuilder::new("small")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let big = arith_grammar();
    let small_lhs = rule_lhs_names(&small);
    let big_lhs = rule_lhs_names(&big);
    assert!(small_lhs.is_subset(&big_lhs));
}

#[test]
fn test_subset_empty_is_subset_of_anything() {
    let empty = GrammarBuilder::new("empty").build();
    let a = arith_grammar();
    assert!(token_names(&empty).is_subset(&token_names(&a)));
    assert!(rule_lhs_names(&empty).is_subset(&rule_lhs_names(&a)));
}

#[test]
fn test_subset_self_is_subset() {
    let a = arith_grammar();
    assert!(token_names(&a).is_subset(&token_names(&a)));
    assert!(rule_lhs_names(&a).is_subset(&rule_lhs_names(&a)));
}

#[test]
fn test_subset_production_count_monotonic() {
    // Adding productions should not decrease total count
    let base = GrammarBuilder::new("base")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let extended = GrammarBuilder::new("extended")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    assert!(total_production_count(&base) <= total_production_count(&extended));
}

#[test]
fn test_subset_token_count_monotonic() {
    let base = GrammarBuilder::new("base").token("X", "x").build();
    let extended = GrammarBuilder::new("extended")
        .token("X", "x")
        .token("Y", "y")
        .build();
    assert!(base.tokens.len() <= extended.tokens.len());
}

// ===========================================================================
// 5. Grammar diff detection (8 tests)
// ===========================================================================

fn added_tokens(before: &Grammar, after: &Grammar) -> HashSet<String> {
    token_names(after)
        .difference(&token_names(before))
        .cloned()
        .collect()
}

fn removed_tokens(before: &Grammar, after: &Grammar) -> HashSet<String> {
    token_names(before)
        .difference(&token_names(after))
        .cloned()
        .collect()
}

fn added_rules(before: &Grammar, after: &Grammar) -> HashSet<String> {
    rule_lhs_names(after)
        .difference(&rule_lhs_names(before))
        .cloned()
        .collect()
}

fn removed_rules(before: &Grammar, after: &Grammar) -> HashSet<String> {
    rule_lhs_names(before)
        .difference(&rule_lhs_names(after))
        .cloned()
        .collect()
}

#[test]
fn test_diff_no_change() {
    let a = arith_grammar();
    let b = arith_grammar();
    assert!(added_tokens(&a, &b).is_empty());
    assert!(removed_tokens(&a, &b).is_empty());
}

#[test]
fn test_diff_added_token() {
    let before = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let after = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let added = added_tokens(&before, &after);
    assert!(added.contains("B"));
    assert_eq!(added.len(), 1);
}

#[test]
fn test_diff_removed_token() {
    let before = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let after = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let removed = removed_tokens(&before, &after);
    assert!(removed.contains("B"));
}

#[test]
fn test_diff_added_rule() {
    let before = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let after = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .rule("extra", vec!["A"])
        .start("s")
        .build();
    let added = added_rules(&before, &after);
    assert!(added.contains("extra"));
}

#[test]
fn test_diff_removed_rule() {
    let before = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .rule("extra", vec!["A"])
        .start("s")
        .build();
    let after = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let removed = removed_rules(&before, &after);
    assert!(removed.contains("extra"));
}

#[test]
fn test_diff_symmetric_add_remove() {
    let a = arith_grammar();
    let b = bool_grammar();
    let a_to_b_added = added_tokens(&a, &b);
    let b_to_a_removed = removed_tokens(&b, &a);
    assert_eq!(a_to_b_added, b_to_a_removed);
}

#[test]
fn test_diff_production_count_change() {
    let before = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let after = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    assert!(total_production_count(&after) > total_production_count(&before));
}

#[test]
fn test_diff_empty_to_nonempty() {
    let empty = GrammarBuilder::new("g").build();
    let full = arith_grammar();
    assert!(!added_tokens(&empty, &full).is_empty());
    assert!(removed_tokens(&empty, &full).is_empty());
}

// ===========================================================================
// 6. Normalize then compare (8 tests)
// ===========================================================================

#[test]
fn test_normalize_plain_grammar_unchanged_rule_count() {
    let mut g = arith_grammar();
    let before_count = total_production_count(&g);
    let _normalized = g.normalize();
    let after_count = total_production_count(&g);
    // Plain grammar (no Optional/Repeat/Choice symbols) should keep same count
    assert_eq!(before_count, after_count);
}

#[test]
fn test_normalize_optional_adds_rules() {
    let mut g = Grammar::new("opt_test".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt_s = SymbolId(3);
    g.tokens.insert(
        tok_a,
        adze_ir::Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_b,
        adze_ir::Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_s, "s".to_string());
    g.add_rule(Rule {
        lhs: nt_s,
        rhs: vec![
            Symbol::Terminal(tok_a),
            Symbol::Optional(Box::new(Symbol::Terminal(tok_b))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    let before = total_production_count(&g);
    g.normalize();
    let after = total_production_count(&g);
    // Optional expands into aux -> inner | epsilon, so more rules
    assert!(after > before);
}

#[test]
fn test_normalize_repeat_adds_rules() {
    let mut g = Grammar::new("rep_test".to_string());
    let tok_a = SymbolId(1);
    let nt_s = SymbolId(2);
    g.tokens.insert(
        tok_a,
        adze_ir::Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_s, "s".to_string());
    g.add_rule(Rule {
        lhs: nt_s,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok_a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    let before = total_production_count(&g);
    g.normalize();
    let after = total_production_count(&g);
    assert!(after > before);
}

#[test]
fn test_normalize_choice_adds_rules() {
    let mut g = Grammar::new("choice_test".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt_s = SymbolId(3);
    g.tokens.insert(
        tok_a,
        adze_ir::Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_b,
        adze_ir::Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_s, "s".to_string());
    g.add_rule(Rule {
        lhs: nt_s,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(tok_a),
            Symbol::Terminal(tok_b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    let before = total_production_count(&g);
    g.normalize();
    let after = total_production_count(&g);
    assert!(after > before);
}

#[test]
fn test_normalize_increases_lhs_count_for_complex() {
    let mut g = Grammar::new("complex".to_string());
    let tok_a = SymbolId(1);
    let nt_s = SymbolId(2);
    g.tokens.insert(
        tok_a,
        adze_ir::Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_s, "s".to_string());
    g.add_rule(Rule {
        lhs: nt_s,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok_a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    let before_lhs = g.rules.len();
    g.normalize();
    let after_lhs = g.rules.len();
    // Auxiliary rules add new LHS symbols
    assert!(after_lhs > before_lhs);
}

#[test]
fn test_normalize_repeat_one_adds_rules() {
    let mut g = Grammar::new("rep1_test".to_string());
    let tok_a = SymbolId(1);
    let nt_s = SymbolId(2);
    g.tokens.insert(
        tok_a,
        adze_ir::Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_s, "s".to_string());
    g.add_rule(Rule {
        lhs: nt_s,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(tok_a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    let before = total_production_count(&g);
    g.normalize();
    let after = total_production_count(&g);
    // RepeatOne(A) -> aux -> aux A | A  (2 aux rules + 1 original = 3)
    assert!(after > before);
}

#[test]
fn test_normalize_sequence_flattens() {
    let mut g = Grammar::new("seq_test".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt_s = SymbolId(3);
    g.tokens.insert(
        tok_a,
        adze_ir::Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_b,
        adze_ir::Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_s, "s".to_string());
    g.add_rule(Rule {
        lhs: nt_s,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(tok_a),
            Symbol::Terminal(tok_b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    g.normalize();
    // After normalizing, the sequence should be flattened — rule has [A, B] not Sequence
    let rules = g.get_rules_for_symbol(nt_s).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 2);
    assert!(matches!(rules[0].rhs[0], Symbol::Terminal(_)));
    assert!(matches!(rules[0].rhs[1], Symbol::Terminal(_)));
}

#[test]
fn test_normalize_idempotent() {
    let mut g1 = Grammar::new("idempotent".to_string());
    let tok_a = SymbolId(1);
    let nt_s = SymbolId(2);
    g1.tokens.insert(
        tok_a,
        adze_ir::Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g1.rule_names.insert(nt_s, "s".to_string());
    g1.add_rule(Rule {
        lhs: nt_s,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok_a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });
    g1.normalize();
    let count_after_first = total_production_count(&g1);
    g1.normalize();
    let count_after_second = total_production_count(&g1);
    assert_eq!(
        count_after_first, count_after_second,
        "normalize should be idempotent"
    );
}

// ===========================================================================
// 7. Builder chaining patterns (7 tests)
// ===========================================================================

#[test]
fn test_builder_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.name, "empty");
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.externals.is_empty());
}

#[test]
fn test_builder_single_token() {
    let g = GrammarBuilder::new("one_token")
        .token("NUM", r"\d+")
        .build();
    assert_eq!(g.tokens.len(), 1);
    assert!(g.rules.is_empty());
}

#[test]
fn test_builder_many_tokens() {
    let g = GrammarBuilder::new("many")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .build();
    assert_eq!(g.tokens.len(), 5);
}

#[test]
fn test_builder_chain_rules() {
    let g = GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("a", vec!["X"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("c")
        .build();
    assert_eq!(g.rules.len(), 3);
    assert_eq!(total_production_count(&g), 3);
}

#[test]
fn test_builder_deep_nesting_via_rules() {
    // A grammar where rules reference many non-terminals
    let g = GrammarBuilder::new("deep")
        .token("LEAF", "leaf")
        .rule("n0", vec!["LEAF"])
        .rule("n1", vec!["n0"])
        .rule("n2", vec!["n1"])
        .rule("n3", vec!["n2"])
        .rule("n4", vec!["n3"])
        .start("n4")
        .build();
    assert_eq!(g.rules.len(), 5);
}

#[test]
fn test_builder_extras_and_externals() {
    let g = GrammarBuilder::new("full")
        .token("WS", r"[ \t]+")
        .token("INDENT", "INDENT")
        .extra("WS")
        .external("INDENT")
        .build();
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn test_builder_presets_python_like() {
    let g = GrammarBuilder::python_like();
    assert_eq!(g.name, "python_like");
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
    assert!(!g.externals.is_empty());
    // module should be a rule
    assert!(g.find_symbol_by_name("module").is_some());
}
