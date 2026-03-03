//! Comprehensive tests for `Grammar::normalize()`.
//!
//! Covers: empty/terminal-only grammars, Optional, Repeat, RepeatOne, Choice,
//! Sequence, nested complex symbols, preservation invariants, idempotency,
//! serialization after normalize, and auxiliary ID allocation.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Assert every symbol in every rule is "simple" (no complex wrappers).
fn assert_fully_normalized(grammar: &Grammar) {
    for (lhs, rules) in &grammar.rules {
        for rule in rules {
            for sym in &rule.rhs {
                assert!(
                    matches!(
                        sym,
                        Symbol::Terminal(_)
                            | Symbol::NonTerminal(_)
                            | Symbol::External(_)
                            | Symbol::Epsilon
                    ),
                    "Non-normalized symbol in rule for {lhs}: {sym:?}",
                );
            }
        }
    }
}

/// Collect all `SymbolId`s that appear as LHS of rules produced by normalization.
fn aux_lhs_ids(grammar: &Grammar) -> Vec<SymbolId> {
    grammar.rules.keys().copied().collect()
}

/// Build a grammar and manually inject a rule whose RHS contains `symbol`.
fn grammar_with_complex_rhs(symbol: Symbol) -> Grammar {
    let mut g = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"]) // placeholder – will be overwritten
        .start("root")
        .build();

    // Find root symbol id
    let root_id = g.find_symbol_by_name("root").unwrap();
    let a_id = *g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "A")
        .map(|(id, _)| id)
        .unwrap();

    // Replace the placeholder rule with one that has the complex symbol
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Terminal(a_id), symbol],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    g
}

// ---------------------------------------------------------------------------
// 1. Empty grammar (no rules, no tokens) – normalize is a no-op
// ---------------------------------------------------------------------------

#[test]
fn normalize_empty_grammar_is_noop() {
    let mut g = Grammar::new("empty".to_string());
    let result = g.normalize();
    assert!(result.is_empty());
    assert!(g.rules.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Grammar with only terminals – normalize is a no-op
// ---------------------------------------------------------------------------

#[test]
fn normalize_terminal_only_grammar_is_noop() {
    let mut g = GrammarBuilder::new("terminals")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();

    let rules_before: usize = g.rules.values().map(|v| v.len()).sum();
    g.normalize();
    let rules_after: usize = g.rules.values().map(|v| v.len()).sum();

    assert_eq!(rules_before, rules_after);
    assert_fully_normalized(&g);
}

// ---------------------------------------------------------------------------
// 3. Optional symbol → creates aux rule (inner | ε)
// ---------------------------------------------------------------------------

#[test]
fn normalize_optional_creates_aux_rule() {
    let a_sym = Symbol::Terminal(SymbolId(1)); // token A
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(a_sym)));

    g.normalize();
    assert_fully_normalized(&g);

    // The original rule should now reference a NonTerminal for the aux
    let root_id = g.find_symbol_by_name("root").unwrap();
    let root_rules = &g.rules[&root_id];
    assert_eq!(root_rules.len(), 1);

    let aux_id = match &root_rules[0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };

    // aux should have exactly 2 alternatives: inner and ε
    let aux_rules = &g.rules[&aux_id];
    assert_eq!(aux_rules.len(), 2);

    let has_inner = aux_rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Terminal(_)));
    let has_eps = aux_rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon));

    assert!(has_inner, "aux should have inner alternative");
    assert!(has_eps, "aux should have epsilon alternative");
}

// ---------------------------------------------------------------------------
// 4. Repeat (zero-or-more) symbol → aux → aux inner | ε
// ---------------------------------------------------------------------------

#[test]
fn normalize_repeat_creates_recursive_aux() {
    let b_sym = Symbol::Terminal(SymbolId(2)); // token B
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(b_sym)));

    g.normalize();
    assert_fully_normalized(&g);

    let root_id = g.find_symbol_by_name("root").unwrap();
    let aux_id = match &g.rules[&root_id][0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };

    let aux_rules = &g.rules[&aux_id];
    assert_eq!(aux_rules.len(), 2);

    // One rule should be recursive: aux → aux B
    let has_recursive = aux_rules
        .iter()
        .any(|r| r.rhs.len() == 2 && matches!(r.rhs[0], Symbol::NonTerminal(id) if id == aux_id));
    let has_eps = aux_rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]);

    assert!(has_recursive, "aux should have recursive rule");
    assert!(has_eps, "aux should have epsilon rule");
}

// ---------------------------------------------------------------------------
// 5. RepeatOne (one-or-more) symbol → aux → aux inner | inner
// ---------------------------------------------------------------------------

#[test]
fn normalize_repeat_one_creates_recursive_aux_no_epsilon() {
    let a_sym = Symbol::Terminal(SymbolId(1));
    let mut g = grammar_with_complex_rhs(Symbol::RepeatOne(Box::new(a_sym)));

    g.normalize();
    assert_fully_normalized(&g);

    let root_id = g.find_symbol_by_name("root").unwrap();
    let aux_id = match &g.rules[&root_id][0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };

    let aux_rules = &g.rules[&aux_id];
    assert_eq!(aux_rules.len(), 2);

    // Should have recursive and base-case, but NO epsilon
    let has_eps = aux_rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]);
    assert!(!has_eps, "RepeatOne should NOT produce an epsilon rule");

    let has_base = aux_rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Terminal(_)));
    assert!(has_base, "RepeatOne should have a base inner rule");
}

// ---------------------------------------------------------------------------
// 6. Choice symbol → aux → choice_i  for each alternative
// ---------------------------------------------------------------------------

#[test]
fn normalize_choice_creates_aux_per_alternative() {
    let a = Symbol::Terminal(SymbolId(1));
    let b = Symbol::Terminal(SymbolId(2));
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![a, b]));

    g.normalize();
    assert_fully_normalized(&g);

    let root_id = g.find_symbol_by_name("root").unwrap();
    let aux_id = match &g.rules[&root_id][0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };

    let aux_rules = &g.rules[&aux_id];
    assert_eq!(aux_rules.len(), 2, "Choice with 2 alternatives → 2 rules");
}

#[test]
fn normalize_choice_three_alternatives() {
    let a = Symbol::Terminal(SymbolId(1));
    let b = Symbol::Terminal(SymbolId(2));
    let eps = Symbol::Epsilon;
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![a, b, eps]));

    g.normalize();
    assert_fully_normalized(&g);

    let root_id = g.find_symbol_by_name("root").unwrap();
    let aux_id = match &g.rules[&root_id][0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };

    assert_eq!(g.rules[&aux_id].len(), 3);
}

// ---------------------------------------------------------------------------
// 7. Sequence symbol → flattened into parent rule
// ---------------------------------------------------------------------------

#[test]
fn normalize_sequence_flattened_into_parent() {
    let a = Symbol::Terminal(SymbolId(1));
    let b = Symbol::Terminal(SymbolId(2));
    let mut g = grammar_with_complex_rhs(Symbol::Sequence(vec![a, b]));

    g.normalize();
    assert_fully_normalized(&g);

    // Sequence is flattened: root → A, A, B  (original A + flattened A, B)
    let root_id = g.find_symbol_by_name("root").unwrap();
    let rhs = &g.rules[&root_id][0].rhs;
    assert_eq!(rhs.len(), 3, "Sequence of 2 flattened after leading A");
}

// ---------------------------------------------------------------------------
// 8. Nested: Optional(Repeat(terminal))
// ---------------------------------------------------------------------------

#[test]
fn normalize_nested_optional_repeat() {
    let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(inner)));

    g.normalize();
    assert_fully_normalized(&g);

    // Should produce at least 2 aux symbols (one for Optional, one for Repeat)
    // Original grammar had 1 rule LHS (root). After normalize we need more.
    assert!(
        g.rules.len() >= 3,
        "nested complex should produce multiple aux LHS symbols, got {}",
        g.rules.len()
    );
}

// ---------------------------------------------------------------------------
// 9. Nested: Choice containing Optional
// ---------------------------------------------------------------------------

#[test]
fn normalize_choice_containing_optional() {
    let opt = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let b = Symbol::Terminal(SymbolId(2));
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![opt, b]));

    g.normalize();
    assert_fully_normalized(&g);

    // All symbols should be simple
    for rules in g.rules.values() {
        for rule in rules {
            for sym in &rule.rhs {
                assert!(!matches!(sym, Symbol::Optional(_) | Symbol::Choice(_)));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 10. Nested: Sequence containing Repeat
// ---------------------------------------------------------------------------

#[test]
fn normalize_sequence_containing_repeat() {
    let rep = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2))));
    let a = Symbol::Terminal(SymbolId(1));
    let mut g = grammar_with_complex_rhs(Symbol::Sequence(vec![a, rep]));

    g.normalize();
    assert_fully_normalized(&g);
}

// ---------------------------------------------------------------------------
// 11. Preserves start symbol
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_start_symbol() {
    let mut g = GrammarBuilder::new("start_check")
        .token("X", "x")
        .rule("entry", vec!["X"])
        .start("entry")
        .build();

    let before = g.start_symbol();
    g.normalize();
    let after = g.start_symbol();
    assert_eq!(before, after);
}

// ---------------------------------------------------------------------------
// 12. Preserves tokens
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_token_definitions() {
    let mut g = GrammarBuilder::new("tok")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["NUM", "ID"])
        .start("root")
        .build();

    let tokens_before = g.tokens.clone();
    g.normalize();
    assert_eq!(g.tokens, tokens_before);
}

// ---------------------------------------------------------------------------
// 13. Preserves extras
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_extras() {
    let mut g = GrammarBuilder::new("ex")
        .token("A", "a")
        .token("WS", r"\s+")
        .rule("root", vec!["A"])
        .extra("WS")
        .start("root")
        .build();

    let extras_before = g.extras.clone();
    g.normalize();
    assert_eq!(g.extras, extras_before);
}

// ---------------------------------------------------------------------------
// 14. Preserves externals
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_externals() {
    let mut g = GrammarBuilder::new("ext")
        .token("A", "a")
        .rule("root", vec!["A"])
        .external("INDENT")
        .external("DEDENT")
        .start("root")
        .build();

    let ext_before = g.externals.len();
    g.normalize();
    assert_eq!(g.externals.len(), ext_before);
}

// ---------------------------------------------------------------------------
// 15. Double normalize (idempotency)
// ---------------------------------------------------------------------------

#[test]
fn normalize_is_idempotent() {
    let a = Symbol::Terminal(SymbolId(1));
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(a)));

    g.normalize();
    let snapshot_rule_count: usize = g.rules.values().map(|v| v.len()).sum();
    let snapshot_lhs_count = g.rules.len();

    g.normalize(); // second pass
    let after_rule_count: usize = g.rules.values().map(|v| v.len()).sum();
    let after_lhs_count = g.rules.len();

    assert_eq!(snapshot_rule_count, after_rule_count);
    assert_eq!(snapshot_lhs_count, after_lhs_count);
    assert_fully_normalized(&g);
}

#[test]
fn triple_normalize_still_stable() {
    let rep = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    let opt = Symbol::Optional(Box::new(rep));
    let mut g = grammar_with_complex_rhs(opt);

    g.normalize();
    g.normalize();
    g.normalize();
    assert_fully_normalized(&g);
}

// ---------------------------------------------------------------------------
// 16. Normalized grammar can be serialized (round-trip)
// ---------------------------------------------------------------------------

#[test]
fn normalized_grammar_serializes_to_json() {
    let a = Symbol::Terminal(SymbolId(1));
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(a)));

    g.normalize();

    let json = serde_json::to_string(&g).expect("serialize after normalize");
    assert!(!json.is_empty());

    let _deser: Grammar = serde_json::from_str(&json).expect("deserialize after normalize");
}

#[test]
fn normalized_grammar_roundtrips_through_json() {
    let inner = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(inner)));
    g.normalize();

    let json = serde_json::to_string_pretty(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.rules.len(), g2.rules.len());
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

// ---------------------------------------------------------------------------
// 17. Aux rule IDs don't collide with existing IDs
// ---------------------------------------------------------------------------

#[test]
fn aux_ids_start_above_max_existing_plus_1000() {
    let mut g = GrammarBuilder::new("id_check")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    // Manually inject a high-ID rule
    let high_id = SymbolId(500);
    g.rules.insert(
        high_id,
        vec![Rule {
            lhs: high_id,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rule_names.insert(high_id, "high".to_string());

    g.normalize();

    let all_ids = aux_lhs_ids(&g);
    // Any ID that isn't root (small) or 500 must be >= 500 + 1000 = 1500
    for id in &all_ids {
        if id.0 > 500 {
            assert!(
                id.0 >= 1500,
                "aux id {} should be >= 1500 (max_existing 500 + 1000)",
                id.0
            );
        }
    }
}

#[test]
fn aux_ids_no_collision_with_tokens() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    g.normalize();

    let token_ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    let rule_lhs: Vec<SymbolId> = g.rules.keys().copied().collect();

    // Aux rule LHS ids that are NOT in the original token set should be high
    for lhs in &rule_lhs {
        if !token_ids.contains(lhs) && g.find_symbol_by_name("root") != Some(*lhs) {
            assert!(
                lhs.0 >= 1000,
                "aux id {} should be well above token range",
                lhs.0
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 18. Multiple complex symbols in one rule
// ---------------------------------------------------------------------------

#[test]
fn normalize_multiple_complex_in_single_rule() {
    let mut g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![
                Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))),
                Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    g.normalize();
    assert_fully_normalized(&g);

    // Root should now reference two distinct NonTerminal aux symbols
    let root_rhs = &g.rules[&root_id][0].rhs;
    assert_eq!(root_rhs.len(), 2);
    assert!(matches!(root_rhs[0], Symbol::NonTerminal(_)));
    assert!(matches!(root_rhs[1], Symbol::NonTerminal(_)));

    // The two aux symbols should be different
    if let (Symbol::NonTerminal(a), Symbol::NonTerminal(b)) = (&root_rhs[0], &root_rhs[1]) {
        assert_ne!(
            a, b,
            "two different complex symbols should get distinct aux IDs"
        );
    }
}

// ---------------------------------------------------------------------------
// 19. Normalize returns all rules
// ---------------------------------------------------------------------------

#[test]
fn normalize_returns_all_rules() {
    let a = Symbol::Terminal(SymbolId(1));
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(a)));

    let returned = g.normalize();

    // The returned vec should contain every rule in the grammar
    let total_in_grammar: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(returned.len(), total_in_grammar);
}

// ---------------------------------------------------------------------------
// 20. Deeply nested: Optional(Choice([Repeat(A), B]))
// ---------------------------------------------------------------------------

#[test]
fn normalize_deeply_nested_complex() {
    let deep = Symbol::Optional(Box::new(Symbol::Choice(vec![
        Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1)))),
        Symbol::Terminal(SymbolId(2)),
    ])));

    let mut g = grammar_with_complex_rhs(deep);
    g.normalize();
    assert_fully_normalized(&g);

    // Should create aux symbols for Optional, Choice, and Repeat
    assert!(
        g.rules.len() >= 4,
        "deeply nested should produce ≥4 rule groups, got {}",
        g.rules.len()
    );
}

// ---------------------------------------------------------------------------
// 21. Normalize with epsilon-only rule is a no-op
// ---------------------------------------------------------------------------

#[test]
fn normalize_epsilon_only_rule_is_noop() {
    let mut g = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("root", vec![]) // builder turns empty vec into [Epsilon]
        .start("root")
        .build();

    let before: usize = g.rules.values().map(|v| v.len()).sum();
    g.normalize();
    let after: usize = g.rules.values().map(|v| v.len()).sum();

    assert_eq!(before, after);
    assert_fully_normalized(&g);
}

// ---------------------------------------------------------------------------
// 22. Normalize preserves precedence on simple rules
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_precedence() {
    let mut g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    g.normalize();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let prec_rule = g.rules[&expr_id]
        .iter()
        .find(|r| r.precedence.is_some())
        .expect("precedence rule should survive normalization");

    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

// ---------------------------------------------------------------------------
// 23. Normalize preserves rule_names for original symbols
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_rule_names() {
    let a = Symbol::Terminal(SymbolId(1));
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(a)));

    let names_before = g.rule_names.clone();
    g.normalize();

    // All original names should still be present
    for (id, name) in &names_before {
        assert_eq!(
            g.rule_names.get(id),
            Some(name),
            "rule_name for {id} should be preserved"
        );
    }
}
