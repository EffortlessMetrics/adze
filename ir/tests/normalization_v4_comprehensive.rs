//! Comprehensive V4 tests for `Grammar::normalize()`.
//!
//! Categories:
//! 1. Simple grammar normalization (no change for basic grammars) — 10 tests
//! 2. Grammar name preservation through normalization — 5 tests
//! 3. Token preservation through normalization — 5 tests
//! 4. Rule count changes after normalization — 10 tests
//! 5. Normalization idempotency — 8 tests
//! 6. Start symbol preservation — 5 tests
//! 7. Serde roundtrip after normalization — 5 tests
//! 8. Complex grammars (multiple rules, deep nesting) — 10 tests
//! 9. Edge cases — 7 tests

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Assert every RHS symbol in every rule is simple (no complex wrappers).
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

/// Build a grammar and inject a rule whose RHS contains `symbol`.
fn grammar_with_complex_rhs(symbol: Symbol) -> Grammar {
    let mut g = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    let a_id = *g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "A")
        .map(|(id, _)| id)
        .unwrap();

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

/// Build a simple grammar with only terminals (no complex symbols).
fn simple_terminal_grammar(name: &str, num_tokens: usize) -> Grammar {
    let mut builder = GrammarBuilder::new(name);
    let mut token_names: Vec<String> = Vec::new();
    for i in 0..num_tokens {
        let tname = format!("T{i}");
        let pat = format!("t{i}");
        builder = builder.token(&tname, &pat);
        token_names.push(tname);
    }
    let refs: Vec<&str> = token_names.iter().map(|s| s.as_str()).collect();
    builder = builder.rule("root", refs).start("root");
    builder.build()
}

/// Get token ID by name.
fn token_id(grammar: &Grammar, name: &str) -> SymbolId {
    *grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| id)
        .unwrap()
}

/// Count total number of individual rules across all LHS symbols.
fn total_rule_count(grammar: &Grammar) -> usize {
    grammar.rules.values().map(|v| v.len()).sum()
}

// ===========================================================================
// 1. Simple grammar normalization (no change for basic grammars) — 10 tests
// ===========================================================================

#[test]
fn simple_single_token_rule_unchanged() {
    let mut g = GrammarBuilder::new("s")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
    assert_fully_normalized(&g);
}

#[test]
fn simple_two_token_rule_unchanged() {
    let mut g = GrammarBuilder::new("s")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
}

#[test]
fn simple_three_token_rule_unchanged() {
    let mut g = simple_terminal_grammar("three", 3);
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
}

#[test]
fn simple_nonterminal_chain_unchanged() {
    let mut g = GrammarBuilder::new("chain")
        .token("T", "t")
        .rule("inner", vec!["T"])
        .rule("root", vec!["inner"])
        .start("root")
        .build();
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
    assert_fully_normalized(&g);
}

#[test]
fn simple_two_rules_same_lhs_unchanged() {
    let mut g = GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .start("root")
        .build();
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
}

#[test]
fn simple_multiple_nonterminals_unchanged() {
    let mut g = GrammarBuilder::new("multi")
        .token("T", "t")
        .rule("a", vec!["T"])
        .rule("b", vec!["a"])
        .rule("root", vec!["b"])
        .start("root")
        .build();
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
}

#[test]
fn simple_long_rule_unchanged() {
    let mut g = simple_terminal_grammar("long", 6);
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
    assert_fully_normalized(&g);
}

#[test]
fn simple_grammar_all_rules_returned() {
    let mut g = GrammarBuilder::new("ret")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let result = g.normalize();
    assert_eq!(result.len(), total_rule_count(&g));
}

#[test]
fn simple_grammar_rules_match_all_rules_iter() {
    let mut g = GrammarBuilder::new("iter")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let result = g.normalize();
    let from_iter: Vec<&Rule> = g.all_rules().collect();
    assert_eq!(result.len(), from_iter.len());
}

#[test]
fn simple_grammar_normalized_has_no_complex_symbols() {
    let mut g = simple_terminal_grammar("clean", 4);
    g.normalize();
    assert_fully_normalized(&g);
}

// ===========================================================================
// 2. Grammar name preservation through normalization — 5 tests
// ===========================================================================

#[test]
fn name_preserved_simple_grammar() {
    let mut g = GrammarBuilder::new("my_grammar")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    g.normalize();
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn name_preserved_after_optional_normalization() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    g.name = "optname".to_string();
    g.normalize();
    assert_eq!(g.name, "optname");
}

#[test]
fn name_preserved_after_repeat_normalization() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    g.name = "rptname".to_string();
    g.normalize();
    assert_eq!(g.name, "rptname");
}

#[test]
fn name_preserved_after_choice_normalization() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]));
    g.name = "choicename".to_string();
    g.normalize();
    assert_eq!(g.name, "choicename");
}

#[test]
fn name_preserved_empty_grammar() {
    let mut g = Grammar::new("empty_name".to_string());
    g.normalize();
    assert_eq!(g.name, "empty_name");
}

// ===========================================================================
// 3. Token preservation through normalization — 5 tests
// ===========================================================================

#[test]
fn tokens_unchanged_after_simple_normalize() {
    let mut g = GrammarBuilder::new("tp")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A", "B", "C"])
        .start("root")
        .build();
    let tokens_before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tokens_before);
}

#[test]
fn tokens_unchanged_after_optional_normalize() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    let tokens_before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tokens_before);
}

#[test]
fn tokens_unchanged_after_repeat_normalize() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    let tokens_before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tokens_before);
}

#[test]
fn tokens_unchanged_after_choice_normalize() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]));
    let tokens_before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tokens_before);
}

#[test]
fn token_names_preserved_after_normalize() {
    let mut g = GrammarBuilder::new("tn")
        .token("FOO", "foo")
        .token("BAR", "bar")
        .rule("root", vec!["FOO", "BAR"])
        .start("root")
        .build();
    g.normalize();
    let names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"FOO"));
    assert!(names.contains(&"BAR"));
}

// ===========================================================================
// 4. Rule count changes after normalization — 10 tests
// ===========================================================================

#[test]
fn rule_count_increases_with_optional() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    let before = total_rule_count(&g);
    g.normalize();
    assert!(
        total_rule_count(&g) > before,
        "Optional normalization should add aux rules"
    );
}

#[test]
fn rule_count_increases_with_repeat() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    let before = total_rule_count(&g);
    g.normalize();
    assert!(total_rule_count(&g) > before);
}

#[test]
fn rule_count_increases_with_repeat_one() {
    let mut g =
        grammar_with_complex_rhs(Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(1)))));
    let before = total_rule_count(&g);
    g.normalize();
    assert!(total_rule_count(&g) > before);
}

#[test]
fn rule_count_increases_with_choice() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]));
    let before = total_rule_count(&g);
    g.normalize();
    assert!(total_rule_count(&g) > before);
}

#[test]
fn optional_adds_exactly_two_aux_rules() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    g.normalize();
    // Original rule remains 1, plus 2 aux rules (inner | ε)
    let root_id = g.find_symbol_by_name("root").unwrap();
    let aux_id = match &g.rules[&root_id][0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };
    assert_eq!(g.rules[&aux_id].len(), 2);
}

#[test]
fn repeat_adds_exactly_two_aux_rules() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    g.normalize();
    let root_id = g.find_symbol_by_name("root").unwrap();
    let aux_id = match &g.rules[&root_id][0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };
    assert_eq!(g.rules[&aux_id].len(), 2);
}

#[test]
fn choice_with_four_alternatives_adds_four_aux_rules() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
        Symbol::Terminal(SymbolId(1)),
        Symbol::Epsilon,
    ]));
    g.normalize();
    let root_id = g.find_symbol_by_name("root").unwrap();
    let aux_id = match &g.rules[&root_id][0].rhs[1] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("expected NonTerminal, got {other:?}"),
    };
    assert_eq!(g.rules[&aux_id].len(), 4);
}

#[test]
fn simple_grammar_rule_count_unchanged() {
    let mut g = simple_terminal_grammar("stable", 3);
    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
}

#[test]
fn lhs_count_increases_with_optional() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    let lhs_before = g.rules.len();
    g.normalize();
    assert!(
        g.rules.len() > lhs_before,
        "Optional should introduce a new LHS symbol"
    );
}

#[test]
fn lhs_count_increases_with_repeat() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    let lhs_before = g.rules.len();
    g.normalize();
    assert!(g.rules.len() > lhs_before);
}

// ===========================================================================
// 5. Normalization idempotency — 8 tests
// ===========================================================================

#[test]
fn idempotent_simple_grammar() {
    let mut g = simple_terminal_grammar("idem", 3);
    g.normalize();
    let snap1 = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), snap1);
}

#[test]
fn idempotent_optional() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    g.normalize();
    let count1 = total_rule_count(&g);
    let lhs1 = g.rules.len();
    g.normalize();
    assert_eq!(total_rule_count(&g), count1);
    assert_eq!(g.rules.len(), lhs1);
}

#[test]
fn idempotent_repeat() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    g.normalize();
    let count1 = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count1);
}

#[test]
fn idempotent_repeat_one() {
    let mut g =
        grammar_with_complex_rhs(Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(1)))));
    g.normalize();
    let count1 = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count1);
}

#[test]
fn idempotent_choice() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]));
    g.normalize();
    let count1 = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count1);
}

#[test]
fn idempotent_triple_normalize() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
        Symbol::Terminal(SymbolId(1)),
    )))));
    g.normalize();
    let count1 = total_rule_count(&g);
    g.normalize();
    g.normalize();
    assert_eq!(total_rule_count(&g), count1);
}

#[test]
fn idempotent_fully_normalized_check() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
        Symbol::Epsilon,
    ]));
    g.normalize();
    assert_fully_normalized(&g);
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn idempotent_empty_grammar() {
    let mut g = Grammar::new("empty".to_string());
    g.normalize();
    let count1 = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), count1);
}

// ===========================================================================
// 6. Start symbol preservation — 5 tests
// ===========================================================================

#[test]
fn start_symbol_preserved_simple() {
    let mut g = GrammarBuilder::new("ss")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn start_symbol_preserved_after_optional() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn start_symbol_preserved_after_repeat() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn start_symbol_preserved_after_choice() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]));
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn start_symbol_preserved_complex_nested() {
    let nested = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Terminal(
        SymbolId(1),
    )))));
    let mut g = grammar_with_complex_rhs(nested);
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

// ===========================================================================
// 7. Serde roundtrip after normalization — 5 tests
// ===========================================================================

#[test]
fn serde_roundtrip_simple_normalized() {
    let mut g = GrammarBuilder::new("serde_simple")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    g.normalize();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, g2);
}

#[test]
fn serde_roundtrip_optional_normalized() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
    g.normalize();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(total_rule_count(&g), total_rule_count(&g2));
}

#[test]
fn serde_roundtrip_repeat_normalized() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))));
    g.normalize();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, g2);
}

#[test]
fn serde_roundtrip_choice_normalized() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]));
    g.normalize();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, g2);
}

#[test]
fn serde_roundtrip_empty_normalized() {
    let mut g = Grammar::new("empty_serde".to_string());
    g.normalize();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, g2);
}

// ===========================================================================
// 8. Complex grammars (multiple rules, deep nesting) — 10 tests
// ===========================================================================

#[test]
fn complex_two_rules_with_optional_each() {
    let mut g = GrammarBuilder::new("multi_opt")
        .token("A", "a")
        .token("B", "b")
        .rule("inner", vec!["A"])
        .rule("root", vec!["inner", "B"])
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    let inner_id = g.find_symbol_by_name("inner").unwrap();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");

    // Inject Optional into both rules
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![
                Symbol::NonTerminal(inner_id),
                Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rules.insert(
        inner_id,
        vec![Rule {
            lhs: inner_id,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );

    g.normalize();
    assert_fully_normalized(&g);
    // Should have original 2 LHS + 2 aux LHS
    assert!(g.rules.len() >= 4);
}

#[test]
fn complex_nested_optional_repeat() {
    let inner = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(inner)));
    g.normalize();
    assert_fully_normalized(&g);
    // outer Optional + inner Repeat = at least 2 aux symbols
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_nested_repeat_choice() {
    let choice = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(choice)));
    g.normalize();
    assert_fully_normalized(&g);
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_choice_with_optional_alternatives() {
    let opt_a = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let opt_b = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(2))));
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![opt_a, opt_b]));
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn complex_three_level_nesting() {
    let level3 = Symbol::Terminal(SymbolId(1));
    let level2 = Symbol::Repeat(Box::new(level3));
    let level1 = Symbol::Optional(Box::new(level2));
    let mut g = grammar_with_complex_rhs(level1);
    g.normalize();
    assert_fully_normalized(&g);
    // At least: root + Optional aux + Repeat aux
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_sequence_is_flattened() {
    let seq = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let mut g = grammar_with_complex_rhs(seq);
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn complex_sequence_with_optional_inside() {
    let seq = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(2)))),
    ]);
    let mut g = grammar_with_complex_rhs(seq);
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn complex_multiple_complex_symbols_in_one_rule() {
    let mut g = GrammarBuilder::new("multi_complex")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");

    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![
                Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
                Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    g.normalize();
    assert_fully_normalized(&g);
    // root + Optional aux + Repeat aux
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_repeat_one_with_nested_choice() {
    let choice = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let mut g = grammar_with_complex_rhs(Symbol::RepeatOne(Box::new(choice)));
    g.normalize();
    assert_fully_normalized(&g);
    assert!(g.rules.len() >= 3);
}

#[test]
fn complex_grammar_builder_helpers_normalized() {
    let mut g = GrammarBuilder::new("builder")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .start("expr")
        .build();
    let before = total_rule_count(&g);
    g.normalize();
    // No complex symbols were added, so count should be the same
    assert_eq!(total_rule_count(&g), before);
    assert_fully_normalized(&g);
}

// ===========================================================================
// 9. Edge cases — 7 tests
// ===========================================================================

#[test]
fn edge_empty_grammar_no_rules() {
    let mut g = Grammar::new("edge_empty".to_string());
    let result = g.normalize();
    assert!(result.is_empty());
    assert!(g.rules.is_empty());
}

#[test]
fn edge_single_epsilon_rule() {
    let mut g = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let before = total_rule_count(&g);
    g.normalize();
    assert_eq!(total_rule_count(&g), before);
    assert_fully_normalized(&g);
}

#[test]
fn edge_single_rule_grammar() {
    let mut g = GrammarBuilder::new("single")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    g.normalize();
    assert_eq!(total_rule_count(&g), 1);
    assert_fully_normalized(&g);
}

#[test]
fn edge_choice_with_single_alternative() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![Symbol::Terminal(SymbolId(1))]));
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn edge_optional_of_epsilon() {
    let mut g = grammar_with_complex_rhs(Symbol::Optional(Box::new(Symbol::Epsilon)));
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn edge_repeat_of_epsilon() {
    let mut g = grammar_with_complex_rhs(Symbol::Repeat(Box::new(Symbol::Epsilon)));
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn edge_choice_with_epsilon_only() {
    let mut g = grammar_with_complex_rhs(Symbol::Choice(vec![Symbol::Epsilon, Symbol::Epsilon]));
    g.normalize();
    assert_fully_normalized(&g);
}
