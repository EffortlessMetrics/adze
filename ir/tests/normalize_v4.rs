//! Tests for `Grammar::normalize()` — v4 suite.
//!
//! Covers idempotency, start preservation, rule-count growth, token
//! preservation, determinism, complex grammars, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Assert every RHS symbol is simple (no Optional/Repeat/Choice/Sequence).
fn assert_normalized(grammar: &Grammar) {
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            assert!(
                matches!(
                    sym,
                    Symbol::Terminal(_)
                        | Symbol::NonTerminal(_)
                        | Symbol::External(_)
                        | Symbol::Epsilon
                ),
                "non-normalized symbol in rule for {:?}: {sym:?}",
                rule.lhs,
            );
        }
    }
}

/// Total production count across all LHS symbols.
fn total_rules(grammar: &Grammar) -> usize {
    grammar.rules.values().map(|v| v.len()).sum()
}

/// Find a token's SymbolId by name.
fn token_id(grammar: &Grammar, name: &str) -> SymbolId {
    *grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| id)
        .unwrap_or_else(|| panic!("token {name:?} not found"))
}

/// Build a grammar with a complex RHS symbol injected into `root`.
fn with_complex(symbol: Symbol) -> Grammar {
    let mut g = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    let a_id = token_id(&g, "A");

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

// ===================================================================
// 1. Normalize idempotency
// ===================================================================

#[test]
fn idempotent_empty_grammar() {
    let mut g = Grammar::default();
    g.normalize();
    let r1 = total_rules(&g);
    g.normalize();
    assert_eq!(r1, total_rules(&g));
}

#[test]
fn idempotent_simple_terminal_grammar() {
    let mut g = GrammarBuilder::new("s")
        .token("a", "a")
        .rule("r", vec!["a"])
        .start("r")
        .build();
    g.normalize();
    let r1 = total_rules(&g);
    g.normalize();
    assert_eq!(r1, total_rules(&g));
}

#[test]
fn idempotent_two_rules() {
    let mut g = GrammarBuilder::new("s")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    g.normalize();
    let r1 = total_rules(&g);
    g.normalize();
    assert_eq!(r1, total_rules(&g));
}

#[test]
fn idempotent_after_optional() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(sym);
    g.normalize();
    let r1 = total_rules(&g);
    let lhs1 = g.rules.len();
    g.normalize();
    assert_eq!(r1, total_rules(&g));
    assert_eq!(lhs1, g.rules.len());
}

#[test]
fn idempotent_after_repeat() {
    let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2))));
    let mut g = with_complex(sym);
    g.normalize();
    let snap = total_rules(&g);
    g.normalize();
    assert_eq!(snap, total_rules(&g));
}

#[test]
fn idempotent_after_choice() {
    let sym = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let mut g = with_complex(sym);
    g.normalize();
    let snap = total_rules(&g);
    g.normalize();
    assert_eq!(snap, total_rules(&g));
}

#[test]
fn idempotent_after_sequence() {
    let sym = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let mut g = with_complex(sym);
    g.normalize();
    let snap = total_rules(&g);
    g.normalize();
    assert_eq!(snap, total_rules(&g));
}

#[test]
fn idempotent_triple_normalize() {
    let sym = Symbol::Repeat(Box::new(Symbol::Optional(Box::new(Symbol::Terminal(
        SymbolId(1),
    )))));
    let mut g = with_complex(sym);
    g.normalize();
    let r1 = total_rules(&g);
    g.normalize();
    g.normalize();
    assert_eq!(r1, total_rules(&g));
    assert_normalized(&g);
}

#[test]
fn idempotent_five_normalizations() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .rule("e", vec!["a", "b"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    g.normalize();
    let snap = total_rules(&g);
    for _ in 0..4 {
        g.normalize();
        assert_eq!(snap, total_rules(&g));
    }
}

// ===================================================================
// 2. Normalize preserves start symbol
// ===================================================================

#[test]
fn start_preserved_simple() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("program", vec!["a"])
        .start("program")
        .build();
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

#[test]
fn start_preserved_multiple_rules() {
    let mut g = GrammarBuilder::new("g")
        .token("x", "x")
        .token("y", "y")
        .rule("entry", vec!["x"])
        .rule("entry", vec!["y"])
        .rule("other", vec!["x", "y"])
        .start("entry")
        .build();
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

#[test]
fn start_preserved_with_complex_rhs() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(sym);
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

#[test]
fn start_preserved_after_repeat_normalize() {
    let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2))));
    let mut g = with_complex(sym);
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

#[test]
fn start_preserved_empty_grammar() {
    let mut g = Grammar::default();
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
    assert!(before.is_none());
}

#[test]
fn start_preserved_after_double_normalize() {
    let mut g = GrammarBuilder::new("g")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    let before = g.start_symbol();
    g.normalize();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

// ===================================================================
// 3. Rule count grows (auxiliary rules added)
// ===================================================================

#[test]
fn rule_count_grows_with_optional() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(sym);
    let before = total_rules(&g);
    g.normalize();
    assert!(
        total_rules(&g) > before,
        "optional should add auxiliary rules"
    );
}

#[test]
fn rule_count_grows_with_repeat() {
    let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2))));
    let mut g = with_complex(sym);
    let before = total_rules(&g);
    g.normalize();
    assert!(total_rules(&g) > before, "repeat should add auxiliary rules");
}

#[test]
fn rule_count_grows_with_choice() {
    let sym = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let mut g = with_complex(sym);
    let before = total_rules(&g);
    g.normalize();
    assert!(total_rules(&g) > before, "choice should add auxiliary rules");
}

#[test]
fn rule_count_grows_with_repeat_one() {
    let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(sym);
    let before = total_rules(&g);
    g.normalize();
    assert!(
        total_rules(&g) > before,
        "repeat-one should add auxiliary rules"
    );
}

#[test]
fn rule_count_unchanged_for_simple_grammar() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("r", vec!["a"])
        .start("r")
        .build();
    let before = total_rules(&g);
    g.normalize();
    assert_eq!(total_rules(&g), before);
}

#[test]
fn lhs_count_grows_with_optional() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(sym);
    let lhs_before = g.rules.len();
    g.normalize();
    assert!(g.rules.len() > lhs_before);
}

#[test]
fn nested_complex_grows_more() {
    let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    let outer = Symbol::Optional(Box::new(inner));
    let mut g = with_complex(outer);
    let before = total_rules(&g);
    g.normalize();
    // Nested should produce more rules than a single layer
    assert!(total_rules(&g) >= before + 3);
}

// ===================================================================
// 4. Token preservation
// ===================================================================

#[test]
fn tokens_unchanged_after_normalize() {
    let mut g = GrammarBuilder::new("g")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("e", vec!["NUM"])
        .start("e")
        .build();
    let before = g.tokens.clone();
    g.normalize();
    assert_eq!(g.tokens, before);
}

#[test]
fn tokens_unchanged_with_complex_normalize() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(sym);
    let before = g.tokens.clone();
    g.normalize();
    assert_eq!(g.tokens, before);
}

#[test]
fn token_count_stable_across_normalizations() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("r", vec!["a", "b", "c"])
        .start("r")
        .build();
    let count = g.tokens.len();
    assert_eq!(count, 3);
    for _ in 0..5 {
        g.normalize();
        assert_eq!(g.tokens.len(), count);
    }
}

#[test]
fn tokens_empty_grammar_still_empty() {
    let mut g = Grammar::default();
    assert!(g.tokens.is_empty());
    g.normalize();
    assert!(g.tokens.is_empty());
}

#[test]
fn many_tokens_preserved() {
    let mut builder = GrammarBuilder::new("g");
    for i in 0..10 {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    let mut g = builder.rule("r", vec!["T0"]).start("r").build();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn token_patterns_preserved() {
    let mut g = GrammarBuilder::new("g")
        .token("ID", r"[a-z]+")
        .token("NUM", r"[0-9]+")
        .rule("r", vec!["ID", "NUM"])
        .start("r")
        .build();
    let patterns_before: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    g.normalize();
    let patterns_after: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    assert_eq!(patterns_before, patterns_after);
}

// ===================================================================
// 5. Normalize determinism
// ===================================================================

#[test]
fn deterministic_same_grammar_same_result() {
    let build = || {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
        let mut g = with_complex(sym);
        g.normalize();
        g
    };
    let g1 = build();
    let g2 = build();
    assert_eq!(g1.rules.len(), g2.rules.len());
    assert_eq!(total_rules(&g1), total_rules(&g2));
}

#[test]
fn deterministic_repeat_grammar() {
    let build = || {
        let sym = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2))));
        let mut g = with_complex(sym);
        g.normalize();
        g
    };
    let g1 = build();
    let g2 = build();
    assert_eq!(g1.rules.len(), g2.rules.len());
    assert_eq!(total_rules(&g1), total_rules(&g2));
}

#[test]
fn deterministic_choice_grammar() {
    let build = || {
        let sym = Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ]);
        let mut g = with_complex(sym);
        g.normalize();
        g
    };
    let g1 = build();
    let g2 = build();
    assert_eq!(g1.rules.len(), g2.rules.len());
    assert_eq!(total_rules(&g1), total_rules(&g2));
}

#[test]
fn deterministic_nested_grammar() {
    let build = || {
        let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
        let outer = Symbol::Optional(Box::new(inner));
        let mut g = with_complex(outer);
        g.normalize();
        g
    };
    let g1 = build();
    let g2 = build();
    assert_eq!(g1.rules.len(), g2.rules.len());
    assert_eq!(total_rules(&g1), total_rules(&g2));
}

#[test]
fn deterministic_builder_grammar() {
    let build = || {
        let mut g = GrammarBuilder::new("d")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .rule("s", vec!["a"])
            .start("s")
            .build();
        g.normalize();
        total_rules(&g)
    };
    assert_eq!(build(), build());
}

#[test]
fn deterministic_rule_names_match() {
    let build = || {
        let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
        let mut g = with_complex(sym);
        g.normalize();
        g.rule_names.clone()
    };
    assert_eq!(build(), build());
}

// ===================================================================
// 6. Complex normalize — expressions, recursion, nesting
// ===================================================================

#[test]
fn complex_expression_grammar() {
    let mut g = GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["LPAREN", "expr", "RPAREN"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let before = total_rules(&g);
    g.normalize();
    // Already simple — count should not change
    assert_eq!(total_rules(&g), before);
    assert_normalized(&g);
}

#[test]
fn complex_recursive_with_optional() {
    let opt = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(opt);
    // Add a self-referencing rule for root
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.get_mut(&root_id).unwrap().push(Rule {
        lhs: root_id,
        rhs: vec![Symbol::NonTerminal(root_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.normalize();
    assert_normalized(&g);
}

#[test]
fn complex_nested_repeat_in_choice() {
    let rep = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2))));
    let sym = Symbol::Choice(vec![Symbol::Terminal(SymbolId(1)), rep]);
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
    // Must have grown: choice + repeat each add aux rules
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_optional_in_sequence() {
    let opt = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(2))));
    let seq = Symbol::Sequence(vec![Symbol::Terminal(SymbolId(1)), opt]);
    let mut g = with_complex(seq);
    g.normalize();
    assert_normalized(&g);
}

#[test]
fn complex_repeat_one_in_optional() {
    let rep1 = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(1))));
    let sym = Symbol::Optional(Box::new(rep1));
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_triple_nested() {
    let inner = Symbol::Terminal(SymbolId(1));
    let rep = Symbol::Repeat(Box::new(inner));
    let opt = Symbol::Optional(Box::new(rep));
    let choice = Symbol::Choice(vec![opt, Symbol::Terminal(SymbolId(2))]);
    let mut g = with_complex(choice);
    g.normalize();
    assert_normalized(&g);
    // Three layers of nesting should produce several auxiliary symbols
    assert!(g.rules.len() >= 4);
}

#[test]
fn complex_choice_with_three_alternatives() {
    let sym = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
        Symbol::Epsilon,
    ]);
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
}

#[test]
fn complex_sequence_of_three() {
    let sym = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
        Symbol::Terminal(SymbolId(1)),
    ]);
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
    // Sequence flattens: root → A, A, B, A (original A + 3 from seq)
    let root_id = g.find_symbol_by_name("root").unwrap();
    let rhs_len = g.rules[&root_id][0].rhs.len();
    assert!(rhs_len >= 4, "flattened sequence should be long, got {rhs_len}");
}

#[test]
fn complex_repeat_of_repeat() {
    let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    let outer = Symbol::Repeat(Box::new(inner));
    let mut g = with_complex(outer);
    g.normalize();
    assert_normalized(&g);
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_deeply_nested_optional() {
    let mut sym = Symbol::Terminal(SymbolId(1));
    for _ in 0..4 {
        sym = Symbol::Optional(Box::new(sym));
    }
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
}

// ===================================================================
// 7. Edge cases
// ===================================================================

#[test]
fn edge_already_normalized_no_change() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .rule("r", vec!["a", "b"])
        .start("r")
        .build();
    let before = total_rules(&g);
    g.normalize();
    assert_eq!(total_rules(&g), before);
}

#[test]
fn edge_single_rule_single_terminal() {
    let mut g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    assert_normalized(&g);
    assert_eq!(total_rules(&g), 1);
}

#[test]
fn edge_no_rules_at_all() {
    let mut g = Grammar::default();
    let result = g.normalize();
    assert!(result.is_empty());
    assert!(g.rules.is_empty());
}

#[test]
fn edge_epsilon_only_rule() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("r", vec![])
        .start("r")
        .build();
    g.normalize();
    assert_normalized(&g);
}

#[test]
fn edge_multiple_alternatives_same_lhs() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let before = total_rules(&g);
    g.normalize();
    assert_eq!(total_rules(&g), before);
    assert_normalized(&g);
}

#[test]
fn edge_grammar_name_preserved() {
    let mut g = GrammarBuilder::new("my_lang")
        .token("a", "a")
        .rule("r", vec!["a"])
        .start("r")
        .build();
    g.normalize();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn edge_extras_preserved() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("WS", r"\s+")
        .rule("r", vec!["a"])
        .extra("WS")
        .start("r")
        .build();
    let extras = g.extras.clone();
    g.normalize();
    assert_eq!(g.extras, extras);
}

#[test]
fn edge_externals_preserved() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("r", vec!["a"])
        .external("INDENT")
        .start("r")
        .build();
    let ext_count = g.externals.len();
    g.normalize();
    assert_eq!(g.externals.len(), ext_count);
}

#[test]
fn edge_normalize_returns_rules() {
    let a_id = SymbolId(1);
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(a_id)));
    let mut g = with_complex(sym);
    let result = g.normalize();
    // normalize() returns all rules in the grammar after normalization
    assert!(!result.is_empty(), "normalize should return rules");
    // Aux rules should have been added
    assert!(result.len() >= 3, "should include original + aux rules");
}

#[test]
fn edge_normalize_simple_returns_all_rules() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("r", vec!["a"])
        .start("r")
        .build();
    let before = total_rules(&g);
    let result = g.normalize();
    // Returns all rules — for a simple grammar that's just the existing ones
    assert_eq!(result.len(), before);
}

#[test]
fn edge_optional_epsilon_normalizes() {
    let sym = Symbol::Optional(Box::new(Symbol::Epsilon));
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
}

#[test]
fn edge_choice_single_alternative() {
    let sym = Symbol::Choice(vec![Symbol::Terminal(SymbolId(1))]);
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
}

#[test]
fn edge_sequence_single_element() {
    let sym = Symbol::Sequence(vec![Symbol::Terminal(SymbolId(1))]);
    let mut g = with_complex(sym);
    g.normalize();
    assert_normalized(&g);
}

#[test]
fn edge_all_rules_iterator_works_after_normalize() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = with_complex(sym);
    g.normalize();
    let count = g.all_rules().count();
    assert!(count >= 3, "should have original + aux rules, got {count}");
}
