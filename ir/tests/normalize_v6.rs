//! Tests for `Grammar::normalize()` — v6 suite.
//!
//! 64 tests across 8 categories covering normalization of Optional, Repeat,
//! Choice, Sequence, idempotency, validation, token preservation, complex
//! grammars, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Assert every RHS symbol is simple (no Optional/Repeat/Choice/Sequence).
fn assert_fully_normalized(grammar: &Grammar) {
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

/// Build a minimal grammar and inject a complex symbol into the root rule RHS.
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

/// Check that at least one rule has an epsilon production for the given LHS.
fn has_epsilon_rule(grammar: &Grammar, lhs: SymbolId) -> bool {
    grammar.rules.get(&lhs).is_some_and(|rules| {
        rules
            .iter()
            .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon))
    })
}

/// Count the number of distinct LHS non-terminals (rule groups).
fn rule_group_count(grammar: &Grammar) -> usize {
    grammar.rules.len()
}

// ===================================================================
// 1. Normalize produces valid grammar (validate passes)
// ===================================================================

#[test]
fn valid_after_normalize_simple_terminal_rule() {
    let mut g = GrammarBuilder::new("v")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    g.normalize();
    assert!(g.validate().is_ok());
}

#[test]
fn valid_after_normalize_optional() {
    let b_id = SymbolId(99);
    let mut g = with_complex(Symbol::Optional(Box::new(Symbol::Terminal(b_id))));
    // Put token B at the id used in the grammar
    let real_b = token_id(&g, "B");
    g.rules
        .get_mut(&g.find_symbol_by_name("root").unwrap())
        .unwrap()[0]
        .rhs[1] = Symbol::Optional(Box::new(Symbol::Terminal(real_b)));
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn valid_after_normalize_repeat() {
    let mut g = GrammarBuilder::new("v")
        .token("T", "t")
        .rule("root", vec!["T"])
        .start("root")
        .build();
    let t_id = token_id(&g, "T");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(t_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn valid_after_normalize_choice() {
    let mut g = GrammarBuilder::new("v")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Choice(vec![
                Symbol::Terminal(a_id),
                Symbol::Terminal(b_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn valid_after_normalize_sequence() {
    let mut g = GrammarBuilder::new("v")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Sequence(vec![
                Symbol::Terminal(a_id),
                Symbol::Terminal(b_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn valid_after_normalize_repeat_one() {
    let mut g = GrammarBuilder::new("v")
        .token("Z", "z")
        .rule("root", vec!["Z"])
        .start("root")
        .build();
    let z_id = token_id(&g, "Z");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(z_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn valid_after_normalize_multiple_rules() {
    let mut g = GrammarBuilder::new("v")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .start("root")
        .build();
    g.normalize();
    assert!(g.validate().is_ok());
}

#[test]
fn valid_after_normalize_already_simple() {
    let mut g = GrammarBuilder::new("v")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    g.normalize();
    assert!(g.validate().is_ok());
    assert_fully_normalized(&g);
}

// ===================================================================
// 2. Optional rules generate auxiliary rules
// ===================================================================

fn build_with_optional() -> Grammar {
    let mut g = GrammarBuilder::new("opt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![
                Symbol::Terminal(a_id),
                Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

#[test]
fn optional_creates_auxiliary_rule_group() {
    let mut g = build_with_optional();
    let groups_before = rule_group_count(&g);
    g.normalize();
    assert!(rule_group_count(&g) > groups_before);
}

#[test]
fn optional_auxiliary_has_epsilon_production() {
    let mut g = build_with_optional();
    g.normalize();
    // Auxiliary rules have IDs >= 1000 offset
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    assert!(!aux_ids.is_empty());
    assert!(aux_ids.iter().any(|&id| has_epsilon_rule(&g, id)));
}

#[test]
fn optional_auxiliary_has_inner_production() {
    let mut g = build_with_optional();
    let b_id = token_id(&g, "B");
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    let has_inner = aux_ids.iter().any(|&id| {
        g.rules[&id]
            .iter()
            .any(|r| r.rhs.contains(&Symbol::Terminal(b_id)))
    });
    assert!(has_inner);
}

#[test]
fn optional_replaces_symbol_with_nonterminal() {
    let mut g = build_with_optional();
    g.normalize();
    assert_fully_normalized(&g);
    let root_id = g.find_symbol_by_name("root").unwrap();
    let root_rules = &g.rules[&root_id];
    // The root rule should reference the auxiliary via NonTerminal
    let has_nt = root_rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|s| matches!(s, Symbol::NonTerminal(id) if id.0 >= 1000))
    });
    assert!(has_nt);
}

#[test]
fn optional_produces_exactly_two_aux_rules() {
    let mut g = build_with_optional();
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    assert_eq!(aux_ids.len(), 1);
    assert_eq!(g.rules[&aux_ids[0]].len(), 2); // inner + epsilon
}

#[test]
fn optional_nested_in_optional() {
    let mut g = GrammarBuilder::new("nested_opt")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let x_id = token_id(&g, "X");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Optional(Box::new(
                Symbol::Terminal(x_id),
            ))))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn optional_with_nonterminal_inner() {
    let mut g = GrammarBuilder::new("opt_nt")
        .token("A", "a")
        .rule("root", vec!["A"])
        .rule("item", vec!["A"])
        .start("root")
        .build();
    let item_id = g.find_symbol_by_name("item").unwrap();
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.get_mut(&root_id).unwrap()[0].rhs =
        vec![Symbol::Optional(Box::new(Symbol::NonTerminal(item_id)))];
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn optional_multiple_in_same_rule() {
    let mut g = GrammarBuilder::new("multi_opt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![
                Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
                Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
    // Two distinct auxiliary groups created
    let aux_count = g.rules.keys().filter(|id| id.0 >= 1000).count();
    assert_eq!(aux_count, 2);
}

// ===================================================================
// 3. Repeat rules generate auxiliary rules
// ===================================================================

fn build_with_repeat() -> Grammar {
    let mut g = GrammarBuilder::new("rep")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

#[test]
fn repeat_creates_auxiliary_rule_group() {
    let mut g = build_with_repeat();
    let groups_before = rule_group_count(&g);
    g.normalize();
    assert!(rule_group_count(&g) > groups_before);
}

#[test]
fn repeat_auxiliary_has_epsilon_production() {
    let mut g = build_with_repeat();
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    assert!(!aux_ids.is_empty());
    assert!(aux_ids.iter().any(|&id| has_epsilon_rule(&g, id)));
}

#[test]
fn repeat_auxiliary_has_left_recursive_production() {
    let mut g = build_with_repeat();
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    // aux -> aux inner (left recursive: first symbol is NonTerminal(aux))
    let has_recursive = aux_ids.iter().any(|&id| {
        g.rules[&id]
            .iter()
            .any(|r| r.rhs.len() == 2 && matches!(r.rhs[0], Symbol::NonTerminal(nt) if nt == id))
    });
    assert!(has_recursive);
}

#[test]
fn repeat_produces_exactly_two_aux_rules() {
    let mut g = build_with_repeat();
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    assert_eq!(aux_ids.len(), 1);
    assert_eq!(g.rules[&aux_ids[0]].len(), 2); // recursive + epsilon
}

#[test]
fn repeat_replaces_symbol_with_nonterminal() {
    let mut g = build_with_repeat();
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn repeat_one_creates_auxiliary() {
    let mut g = GrammarBuilder::new("rep1")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(a_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
    let aux_count = g.rules.keys().filter(|id| id.0 >= 1000).count();
    assert!(aux_count >= 1);
}

#[test]
fn repeat_one_has_no_epsilon() {
    let mut g = GrammarBuilder::new("rep1_no_eps")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(a_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    // RepeatOne: aux -> aux inner | inner  (NO epsilon)
    for &id in &aux_ids {
        assert!(!has_epsilon_rule(&g, id));
    }
}

#[test]
fn repeat_one_has_base_case_production() {
    let mut g = GrammarBuilder::new("rep1_base")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(a_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    // One of the aux rules should be: aux -> inner (single terminal)
    let has_base = aux_ids.iter().any(|&id| {
        g.rules[&id]
            .iter()
            .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Terminal(t) if t == a_id))
    });
    assert!(has_base);
}

// ===================================================================
// 4. Choice rules generate auxiliary rules
// ===================================================================

fn build_with_choice() -> Grammar {
    let mut g = GrammarBuilder::new("ch")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let c_id = token_id(&g, "C");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Choice(vec![
                Symbol::Terminal(a_id),
                Symbol::Terminal(b_id),
                Symbol::Terminal(c_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

#[test]
fn choice_creates_auxiliary_rule_group() {
    let mut g = build_with_choice();
    let groups_before = rule_group_count(&g);
    g.normalize();
    assert!(rule_group_count(&g) > groups_before);
}

#[test]
fn choice_auxiliary_has_one_rule_per_alternative() {
    let mut g = build_with_choice();
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    assert_eq!(aux_ids.len(), 1);
    assert_eq!(g.rules[&aux_ids[0]].len(), 3); // one per choice
}

#[test]
fn choice_replaces_symbol_with_nonterminal() {
    let mut g = build_with_choice();
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn choice_two_alternatives() {
    let mut g = GrammarBuilder::new("ch2")
        .token("X", "x")
        .token("Y", "y")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let x_id = token_id(&g, "X");
    let y_id = token_id(&g, "Y");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Choice(vec![
                Symbol::Terminal(x_id),
                Symbol::Terminal(y_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    assert_eq!(g.rules[&aux_ids[0]].len(), 2);
}

#[test]
fn choice_with_nonterminal_alternatives() {
    let mut g = GrammarBuilder::new("ch_nt")
        .token("A", "a")
        .rule("root", vec!["A"])
        .rule("item", vec!["A"])
        .rule("other", vec!["A"])
        .start("root")
        .build();
    let item_id = g.find_symbol_by_name("item").unwrap();
    let other_id = g.find_symbol_by_name("other").unwrap();
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Choice(vec![
                Symbol::NonTerminal(item_id),
                Symbol::NonTerminal(other_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn choice_no_epsilon_unless_explicit() {
    let mut g = build_with_choice();
    g.normalize();
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    for &id in &aux_ids {
        assert!(!has_epsilon_rule(&g, id));
    }
}

#[test]
fn choice_containing_optional_fully_normalizes() {
    let mut g = GrammarBuilder::new("ch_opt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Choice(vec![
                Symbol::Terminal(a_id),
                Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn choice_containing_repeat_fully_normalizes() {
    let mut g = GrammarBuilder::new("ch_rep")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Choice(vec![
                Symbol::Terminal(a_id),
                Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

// ===================================================================
// 5. Normalize is idempotent (second call is no-op)
// ===================================================================

#[test]
fn idempotent_simple_grammar() {
    let mut g = GrammarBuilder::new("idem")
        .token("N", r"\d+")
        .rule("root", vec!["N"])
        .start("root")
        .build();
    g.normalize();
    let snap1 = total_rules(&g);
    g.normalize();
    assert_eq!(total_rules(&g), snap1);
}

#[test]
fn idempotent_optional() {
    let mut g = build_with_optional();
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

#[test]
fn idempotent_repeat() {
    let mut g = build_with_repeat();
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

#[test]
fn idempotent_choice() {
    let mut g = build_with_choice();
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

#[test]
fn idempotent_sequence() {
    let mut g = GrammarBuilder::new("seq_idem")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Sequence(vec![
                Symbol::Terminal(a_id),
                Symbol::Terminal(b_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

#[test]
fn idempotent_mixed_complex() {
    let mut g = GrammarBuilder::new("mix_idem")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
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
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

#[test]
fn idempotent_empty_grammar() {
    let mut g = Grammar::default();
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

#[test]
fn idempotent_repeat_one() {
    let mut g = GrammarBuilder::new("rep1_idem")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(a_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

// ===================================================================
// 6. Token count preserved through normalization
// ===================================================================

#[test]
fn tokens_preserved_simple() {
    let mut g = GrammarBuilder::new("tp")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn tokens_preserved_optional() {
    let mut g = build_with_optional();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn tokens_preserved_repeat() {
    let mut g = build_with_repeat();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn tokens_preserved_choice() {
    let mut g = build_with_choice();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn tokens_preserved_many_tokens() {
    let mut g = GrammarBuilder::new("mt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("root", vec!["A", "B", "C", "D", "E"])
        .start("root")
        .build();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn tokens_preserved_mixed_complex() {
    let mut g = GrammarBuilder::new("mc")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let c_id = token_id(&g, "C");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![
                Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
                Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
                Symbol::Choice(vec![Symbol::Terminal(c_id), Symbol::Epsilon]),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn tokens_preserved_sequence() {
    let mut g = GrammarBuilder::new("ts")
        .token("X", "x")
        .token("Y", "y")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let x_id = token_id(&g, "X");
    let y_id = token_id(&g, "Y");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Sequence(vec![
                Symbol::Terminal(x_id),
                Symbol::Terminal(y_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn tokens_preserved_after_double_normalize() {
    let mut g = build_with_optional();
    let count = g.tokens.len();
    g.normalize();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

// ===================================================================
// 7. Complex grammars: arithmetic, JSON-like expressions
// ===================================================================

fn build_arithmetic() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("LPAREN", "(")
        .token("RPAREN", ")")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("factor", vec!["NUM"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .start("expr")
        .build()
}

#[test]
fn arithmetic_normalizes_cleanly() {
    let mut g = build_arithmetic();
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn arithmetic_token_count_unchanged() {
    let mut g = build_arithmetic();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn arithmetic_validate_passes() {
    let mut g = build_arithmetic();
    g.normalize();
    assert!(g.validate().is_ok());
}

#[test]
fn arithmetic_idempotent() {
    let mut g = build_arithmetic();
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

fn build_json_like() -> Grammar {
    GrammarBuilder::new("json")
        .token("LBRACE", "{")
        .token("RBRACE", "}")
        .token("LBRACKET", "[")
        .token("RBRACKET", "]")
        .token("COLON", ":")
        .token("COMMA", ",")
        .token("STRING", r#""[^"]*""#)
        .token("NUMBER", r"\d+")
        .token("TRUE", "true")
        .token("FALSE", "false")
        .token("NULL", "null")
        .rule("value", vec!["STRING"])
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["TRUE"])
        .rule("value", vec!["FALSE"])
        .rule("value", vec!["NULL"])
        .rule("value", vec!["object"])
        .rule("value", vec!["array"])
        .rule("object", vec!["LBRACE", "RBRACE"])
        .rule("object", vec!["LBRACE", "members", "RBRACE"])
        .rule("members", vec!["pair"])
        .rule("members", vec!["members", "COMMA", "pair"])
        .rule("pair", vec!["STRING", "COLON", "value"])
        .rule("array", vec!["LBRACKET", "RBRACKET"])
        .rule("array", vec!["LBRACKET", "elements", "RBRACKET"])
        .rule("elements", vec!["value"])
        .rule("elements", vec!["elements", "COMMA", "value"])
        .start("value")
        .build()
}

#[test]
fn json_like_normalizes_cleanly() {
    let mut g = build_json_like();
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn json_like_token_count_preserved() {
    let mut g = build_json_like();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), count);
}

#[test]
fn json_like_validate_passes() {
    let mut g = build_json_like();
    g.normalize();
    assert!(g.validate().is_ok());
}

#[test]
fn json_like_idempotent() {
    let mut g = build_json_like();
    g.normalize();
    let snap = g.rules.clone();
    g.normalize();
    assert_eq!(g.rules, snap);
}

// ===================================================================
// 8. Edge cases: empty grammar, single rule, deeply nested
// ===================================================================

#[test]
fn edge_empty_grammar() {
    let mut g = Grammar::default();
    g.normalize();
    assert_eq!(total_rules(&g), 0);
    assert_fully_normalized(&g);
}

#[test]
fn edge_single_terminal_rule() {
    let mut g = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let rules_before = total_rules(&g);
    g.normalize();
    assert_eq!(total_rules(&g), rules_before);
    assert_fully_normalized(&g);
}

#[test]
fn edge_epsilon_only_rule() {
    let mut g = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("root", vec![])
        .start("root")
        .build();
    g.normalize();
    assert_fully_normalized(&g);
    let root_id = g.find_symbol_by_name("root").unwrap();
    assert!(has_epsilon_rule(&g, root_id));
}

#[test]
fn edge_deeply_nested_optional() {
    let mut g = GrammarBuilder::new("deep")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let x_id = token_id(&g, "X");
    let root_id = g.find_symbol_by_name("root").unwrap();
    // Optional(Optional(Optional(X)))
    let deep = Symbol::Optional(Box::new(Symbol::Optional(Box::new(Symbol::Optional(
        Box::new(Symbol::Terminal(x_id)),
    )))));
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![deep],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn edge_deeply_nested_repeat_in_choice() {
    let mut g = GrammarBuilder::new("deep_rc")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    // Choice(Repeat(A), Optional(B))
    let complex = Symbol::Choice(vec![
        Symbol::Repeat(Box::new(Symbol::Terminal(a_id))),
        Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
    ]);
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![complex],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
}

#[test]
fn edge_sequence_flattened_no_aux() {
    let mut g = GrammarBuilder::new("flat")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Sequence(vec![
                Symbol::Terminal(a_id),
                Symbol::Terminal(b_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    // Sequence is flattened in-place — no auxiliary rule groups added
    let aux_count = g.rules.keys().filter(|id| id.0 >= 1000).count();
    assert_eq!(aux_count, 0);
    // But the rule now has two terminals
    let root_rules = &g.rules[&root_id];
    assert!(root_rules.iter().any(|r| r.rhs.len() == 2));
}

#[test]
fn edge_many_alternatives_in_single_rule() {
    let mut g = GrammarBuilder::new("many_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let ids: Vec<SymbolId> = ["A", "B", "C", "D", "E"]
        .iter()
        .map(|n| token_id(&g, n))
        .collect();
    let root_id = g.find_symbol_by_name("root").unwrap();
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Choice(
                ids.iter().map(|&id| Symbol::Terminal(id)).collect(),
            )],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
    let aux_ids: Vec<SymbolId> = g.rules.keys().filter(|id| id.0 >= 1000).copied().collect();
    assert_eq!(aux_ids.len(), 1);
    assert_eq!(g.rules[&aux_ids[0]].len(), 5);
}

#[test]
fn edge_repeat_inside_sequence() {
    let mut g = GrammarBuilder::new("rep_seq")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let a_id = token_id(&g, "A");
    let b_id = token_id(&g, "B");
    let root_id = g.find_symbol_by_name("root").unwrap();
    // Sequence(A, Repeat(B))
    g.rules.insert(
        root_id,
        vec![Rule {
            lhs: root_id,
            rhs: vec![Symbol::Sequence(vec![
                Symbol::Terminal(a_id),
                Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.normalize();
    assert_fully_normalized(&g);
    // Repeat inside sequence produces an auxiliary
    let aux_count = g.rules.keys().filter(|id| id.0 >= 1000).count();
    assert!(aux_count >= 1);
}
