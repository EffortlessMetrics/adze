// Comprehensive IR transformation tests for normalization, optimization, and validation.

use adze_ir::{
    Grammar, GrammarOptimizer, GrammarValidator, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern, ValidationError,
};

// ---------------------------------------------------------------------------
// Helper: build a minimal grammar with the given tokens and rules
// ---------------------------------------------------------------------------

fn token(id: u16, name: &str, pattern: &str) -> (SymbolId, Token) {
    (
        SymbolId(id),
        Token {
            name: name.to_string(),
            pattern: TokenPattern::String(pattern.to_string()),
            fragile: false,
        },
    )
}

fn simple_rule(lhs: u16, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

/// Assert every RHS symbol in the grammar is "simple" (no complex wrappers).
fn assert_all_normalized(grammar: &Grammar) {
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
                    "Non-normalized symbol {:?} in rule for {:?}",
                    sym,
                    lhs
                );
            }
        }
    }
}

// ===========================================================================
// 1. Normalization of Optional symbols → auxiliary rules
// ===========================================================================

#[test]
fn normalize_optional_creates_aux_rules() {
    let mut g = Grammar::new("opt".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();

    // The original rule's RHS should now reference a NonTerminal aux symbol.
    let rhs0 = &g.rules[&SymbolId(1)][0].rhs[0];
    let aux_id = match rhs0 {
        Symbol::NonTerminal(id) => *id,
        other => panic!("Expected NonTerminal, got {:?}", other),
    };

    // Aux rules: aux → Terminal(10) | Epsilon
    let aux = &g.rules[&aux_id];
    assert_eq!(aux.len(), 2);
    assert!(
        aux.iter()
            .any(|r| r.rhs == vec![Symbol::Terminal(SymbolId(10))])
    );
    assert!(aux.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

#[test]
fn normalize_optional_nonterminal_inner() {
    let mut g = Grammar::new("opt_nt".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    // rule2 → Terminal(10)
    g.rules.insert(
        SymbolId(2),
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(10))], 1)],
    );
    // rule1 → Optional(NonTerminal(2))
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Optional(Box::new(Symbol::NonTerminal(SymbolId(2))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());
    g.rule_names.insert(SymbolId(2), "inner".into());

    g.normalize();
    assert_all_normalized(&g);

    // Aux should have rule → NonTerminal(2) | Epsilon
    let aux_id = match &g.rules[&SymbolId(1)][0].rhs[0] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("Expected NonTerminal, got {:?}", other),
    };
    let aux = &g.rules[&aux_id];
    assert!(
        aux.iter()
            .any(|r| r.rhs == vec![Symbol::NonTerminal(SymbolId(2))])
    );
    assert!(aux.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

// ===========================================================================
// 2. Normalization of Repeat symbols → auxiliary rules
// ===========================================================================

#[test]
fn normalize_repeat_creates_recursive_aux() {
    let mut g = Grammar::new("rep".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);

    let aux_id = match &g.rules[&SymbolId(1)][0].rhs[0] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("Expected NonTerminal, got {:?}", other),
    };
    let aux = &g.rules[&aux_id];
    assert_eq!(aux.len(), 2);
    // aux → aux Terminal(10)
    assert!(aux.iter().any(|r| r.rhs.len() == 2
        && r.rhs[0] == Symbol::NonTerminal(aux_id)
        && r.rhs[1] == Symbol::Terminal(SymbolId(10))));
    // aux → Epsilon
    assert!(aux.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

// ===========================================================================
// 3. Normalization of RepeatOne symbols → auxiliary rules
// ===========================================================================

#[test]
fn normalize_repeat_one_creates_aux_with_base() {
    let mut g = Grammar::new("rep1".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(10))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);

    let aux_id = match &g.rules[&SymbolId(1)][0].rhs[0] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("Expected NonTerminal, got {:?}", other),
    };
    let aux = &g.rules[&aux_id];
    assert_eq!(aux.len(), 2);
    // aux → aux Terminal(10)
    assert!(aux.iter().any(|r| r.rhs.len() == 2
        && r.rhs[0] == Symbol::NonTerminal(aux_id)
        && r.rhs[1] == Symbol::Terminal(SymbolId(10))));
    // aux → Terminal(10)   (base case, NOT Epsilon)
    assert!(
        aux.iter()
            .any(|r| r.rhs == vec![Symbol::Terminal(SymbolId(10))])
    );
    // No epsilon rule for RepeatOne
    assert!(!aux.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

// ===========================================================================
// 4. Normalization of Choice symbols → auxiliary rules
// ===========================================================================

#[test]
fn normalize_choice_creates_alternative_rules() {
    let mut g = Grammar::new("choice".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ])],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);

    let aux_id = match &g.rules[&SymbolId(1)][0].rhs[0] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("Expected NonTerminal, got {:?}", other),
    };
    let aux = &g.rules[&aux_id];
    assert_eq!(aux.len(), 2);
    assert!(
        aux.iter()
            .any(|r| r.rhs == vec![Symbol::Terminal(SymbolId(10))])
    );
    assert!(
        aux.iter()
            .any(|r| r.rhs == vec![Symbol::Terminal(SymbolId(11))])
    );
}

#[test]
fn normalize_choice_three_alternatives() {
    let mut g = Grammar::new("choice3".into());
    for id in 10..=12 {
        let (tid, tok) = token(
            id,
            &format!("t{}", id),
            &format!("{}", (b'a' + (id - 10) as u8) as char),
        );
        g.tokens.insert(tid, tok);
    }
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
                Symbol::Terminal(SymbolId(12)),
            ])],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);

    let aux_id = match &g.rules[&SymbolId(1)][0].rhs[0] {
        Symbol::NonTerminal(id) => *id,
        other => panic!("Expected NonTerminal, got {:?}", other),
    };
    assert_eq!(g.rules[&aux_id].len(), 3);
}

// ===========================================================================
// 5. Normalization of Sequence symbols → auxiliary rules
// ===========================================================================

#[test]
fn normalize_sequence_flattens_into_rule() {
    let mut g = Grammar::new("seq".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ])],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);

    // Sequence should be flattened directly into the parent rule
    let rule = &g.rules[&SymbolId(1)][0];
    assert_eq!(rule.rhs.len(), 2);
    assert_eq!(rule.rhs[0], Symbol::Terminal(SymbolId(10)));
    assert_eq!(rule.rhs[1], Symbol::Terminal(SymbolId(11)));
}

#[test]
fn normalize_sequence_three_elements() {
    let mut g = Grammar::new("seq3".into());
    for id in 10..=12 {
        let (tid, tok) = token(id, &format!("t{}", id), "x");
        g.tokens.insert(tid, tok);
    }
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
                Symbol::Terminal(SymbolId(12)),
            ])],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    let rule = &g.rules[&SymbolId(1)][0];
    assert_eq!(rule.rhs.len(), 3);
}

// ===========================================================================
// 6. Normalization of deeply nested symbols
// ===========================================================================

#[test]
fn normalize_nested_choice_optional_repeat() {
    // rule → Choice(Optional(Repeat(Terminal(10))))
    let mut g = Grammar::new("nested".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Choice(vec![Symbol::Optional(Box::new(
                Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10)))),
            ))])],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);
    // Multiple auxiliary rules should have been created
    assert!(
        g.rules.len() >= 3,
        "Expected ≥3 rule groups, got {}",
        g.rules.len()
    );
}

#[test]
fn normalize_optional_repeat_one() {
    // rule → Optional(RepeatOne(Terminal(10)))
    let mut g = Grammar::new("opt_rep1".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Optional(Box::new(Symbol::RepeatOne(Box::new(
                Symbol::Terminal(SymbolId(10)),
            ))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);
    // At least: start, opt-aux, repeat1-aux
    assert!(g.rules.len() >= 3);
}

#[test]
fn normalize_repeat_of_choice() {
    // rule → Repeat(Choice(Terminal(10), Terminal(11)))
    let mut g = Grammar::new("rep_choice".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Repeat(Box::new(Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ])))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);
    // start, repeat-aux (references choice-aux), choice-aux
    assert!(g.rules.len() >= 3);
}

// ===========================================================================
// 7. Normalization preserves existing simple rules
// ===========================================================================

#[test]
fn normalize_preserves_simple_terminal_rule() {
    let mut g = Grammar::new("simple".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();

    assert_eq!(g.rules[&SymbolId(1)].len(), 1);
    assert_eq!(
        g.rules[&SymbolId(1)][0].rhs,
        vec![Symbol::Terminal(SymbolId(10))]
    );
}

#[test]
fn normalize_preserves_multi_symbol_simple_rule() {
    let mut g = Grammar::new("multi".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();

    let rule = &g.rules[&SymbolId(1)][0];
    assert_eq!(rule.rhs.len(), 2);
    assert_eq!(rule.rhs[0], Symbol::Terminal(SymbolId(10)));
    assert_eq!(rule.rhs[1], Symbol::Terminal(SymbolId(11)));
}

#[test]
fn normalize_preserves_simple_alongside_complex() {
    let mut g = Grammar::new("mixed".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);

    // Complex rule
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
            0,
        )],
    );
    // Simple rule
    g.rules.insert(
        SymbolId(2),
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(11))], 1)],
    );
    g.rule_names.insert(SymbolId(1), "complex".into());
    g.rule_names.insert(SymbolId(2), "simple".into());

    g.normalize();

    assert_eq!(g.rules[&SymbolId(2)].len(), 1);
    assert_eq!(
        g.rules[&SymbolId(2)][0].rhs,
        vec![Symbol::Terminal(SymbolId(11))]
    );
}

// ===========================================================================
// 8. Symbol ID allocation starts at max_existing_id + 1000
// ===========================================================================

#[test]
fn normalize_aux_ids_start_above_max_plus_1000() {
    let mut g = Grammar::new("ids".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(50),
        vec![simple_rule(
            50,
            vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(50), "start".into());

    g.normalize();

    // max existing id was 50, so aux IDs should start at 1050
    for id in g.rules.keys() {
        if id.0 != 50 {
            assert!(
                id.0 >= 1050,
                "Auxiliary SymbolId {} should be >= 1050",
                id.0
            );
        }
    }
}

#[test]
fn normalize_aux_ids_high_existing() {
    let mut g = Grammar::new("high_ids".into());
    let (tid, tok) = token(5000, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(5000),
        vec![simple_rule(
            5000,
            vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(5000))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(5000), "start".into());

    g.normalize();

    for id in g.rules.keys() {
        if id.0 != 5000 {
            assert!(
                id.0 >= 6000,
                "Auxiliary SymbolId {} should be >= 6000",
                id.0
            );
        }
    }
}

// ===========================================================================
// 9. Multiple normalizations are idempotent
// ===========================================================================

#[test]
fn normalize_idempotent_optional() {
    let mut g = Grammar::new("idem".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    let count_after_first = g.rules.len();
    assert_all_normalized(&g);

    g.normalize();
    assert_eq!(
        g.rules.len(),
        count_after_first,
        "Second normalize should be no-op"
    );
    assert_all_normalized(&g);
}

#[test]
fn normalize_idempotent_complex_nested() {
    let mut g = Grammar::new("idem_nested".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![Symbol::Repeat(Box::new(Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10)))),
            ])))],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    let snapshot: Vec<_> = g.rules.keys().copied().collect();
    assert_all_normalized(&g);

    g.normalize();
    let snapshot2: Vec<_> = g.rules.keys().copied().collect();
    assert_eq!(snapshot, snapshot2);
}

#[test]
fn normalize_idempotent_already_simple() {
    let mut g = Grammar::new("already_simple".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    g.normalize();
    g.normalize();

    assert_eq!(g.rules.len(), 1);
    assert_eq!(
        g.rules[&SymbolId(1)][0].rhs,
        vec![Symbol::Terminal(SymbolId(10))]
    );
}

// ===========================================================================
// 10. Optimizer removes unreachable rules
// ===========================================================================

#[test]
fn optimizer_removes_unreachable_tokens() {
    let mut g = Grammar::new("unreach".into());
    let (t1, tok1) = token(10, "used", "a");
    let (t2, tok2) = token(11, "unused", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // Token 11 is unused and should have been removed
    assert!(stats.removed_unused_symbols > 0);
    assert!(!g.tokens.contains_key(&SymbolId(11)));
}

// ===========================================================================
// 11. Optimizer merges identical rules (equivalent tokens)
// ===========================================================================

#[test]
fn optimizer_merges_equivalent_tokens() {
    let mut g = Grammar::new("merge".into());
    // Two tokens with the same string pattern
    let (t1, tok1) = token(10, "plus1", "+");
    let (t2, tok2) = token(11, "plus2", "+");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![
            simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0),
            simple_rule(1, vec![Symbol::Terminal(SymbolId(11))], 1),
        ],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    assert!(stats.merged_tokens > 0);
    // Only one of the two tokens should remain
    assert_eq!(g.tokens.len(), 1);
}

// ===========================================================================
// 12. Optimizer preserves rule ordering constraints
// ===========================================================================

#[test]
fn optimizer_preserves_start_symbol() {
    let mut g = Grammar::new("order".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0)],
    );
    g.rules.insert(
        SymbolId(2),
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(10))], 1)],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());
    g.rule_names.insert(SymbolId(2), "expr".into());

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);

    // source_file must still exist
    assert!(
        g.find_symbol_by_name("source_file").is_some(),
        "source_file should survive optimization"
    );
}

#[test]
fn optimizer_preserves_start_rule_with_multiple_rules() {
    let mut g = Grammar::new("multi_rules".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![
            simple_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0),
            simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 2),
        ],
    );
    g.rules.insert(
        SymbolId(2),
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(11))], 1)],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());
    g.rule_names.insert(SymbolId(2), "item".into());

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);

    // source_file should still have rules
    let sf_id = g.find_symbol_by_name("source_file").unwrap();
    assert!(g.rules.contains_key(&sf_id));
}

// ===========================================================================
// 13. Validation catches invalid symbol references
// ===========================================================================

#[test]
fn validation_catches_undefined_symbol_in_rhs() {
    let mut g = Grammar::new("undef".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    // rule references NonTerminal(99) which doesn't exist
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::NonTerminal(SymbolId(99))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(99))),
        "Should detect undefined symbol 99, errors: {:?}",
        result.errors
    );
}

#[test]
fn validation_catches_undefined_terminal_ref() {
    let mut g = Grammar::new("undef_term".into());
    // No tokens defined but rule references Terminal(42)
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::Terminal(SymbolId(42))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result.errors.iter().any(|e| matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(42)
        )),
        "Should detect undefined terminal 42"
    );
}

// ===========================================================================
// 14. Validation catches circular rule dependencies
// ===========================================================================

#[test]
fn validation_catches_direct_cycle() {
    let mut g = Grammar::new("cycle".into());
    // A → B, B → A  (cycle without any terminal base-case)
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0)],
    );
    g.rules.insert(
        SymbolId(2),
        vec![simple_rule(2, vec![Symbol::NonTerminal(SymbolId(1))], 1)],
    );
    g.rule_names.insert(SymbolId(1), "A".into());
    g.rule_names.insert(SymbolId(2), "B".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "Should detect cycle between A and B, errors: {:?}",
        result.errors
    );
}

#[test]
fn validation_catches_self_referencing_cycle() {
    let mut g = Grammar::new("self_cycle".into());
    // A → A (direct self-reference with no base)
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::NonTerminal(SymbolId(1))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "A".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "Should detect self-referencing cycle"
    );
}

#[test]
fn validation_catches_transitive_cycle() {
    let mut g = Grammar::new("trans_cycle".into());
    // A → B, B → C, C → A
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0)],
    );
    g.rules.insert(
        SymbolId(2),
        vec![simple_rule(2, vec![Symbol::NonTerminal(SymbolId(3))], 1)],
    );
    g.rules.insert(
        SymbolId(3),
        vec![simple_rule(3, vec![Symbol::NonTerminal(SymbolId(1))], 2)],
    );
    g.rule_names.insert(SymbolId(1), "A".into());
    g.rule_names.insert(SymbolId(2), "B".into());
    g.rule_names.insert(SymbolId(3), "C".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "Should detect transitive cycle A→B→C→A"
    );
}

// ===========================================================================
// 15. Validation catches missing start symbol / empty grammar
// ===========================================================================

#[test]
fn validation_catches_empty_grammar() {
    let g = Grammar::new("empty".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "Should detect empty grammar"
    );
}

#[test]
fn validation_no_errors_for_valid_grammar() {
    let mut g = Grammar::new("valid".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    let critical_errors: Vec<_> = result
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::NoExplicitStartRule))
        .collect();
    assert!(
        critical_errors.is_empty(),
        "Valid grammar should have no critical errors, got: {:?}",
        critical_errors
    );
}

// ===========================================================================
// Additional transformation tests for broader coverage
// ===========================================================================

#[test]
fn normalize_multiple_complex_in_single_rule() {
    // rule → Optional(t1) Repeat(t2)
    let mut g = Grammar::new("multi_complex".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![
                Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10)))),
                Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(11)))),
            ],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();
    assert_all_normalized(&g);

    // Parent rule should now reference two different aux NonTerminals
    let rule = &g.rules[&SymbolId(1)][0];
    assert_eq!(rule.rhs.len(), 2);
    assert!(matches!(rule.rhs[0], Symbol::NonTerminal(_)));
    assert!(matches!(rule.rhs[1], Symbol::NonTerminal(_)));
    // They should be different aux symbols
    let id0 = match &rule.rhs[0] {
        Symbol::NonTerminal(id) => *id,
        _ => unreachable!(),
    };
    let id1 = match &rule.rhs[1] {
        Symbol::NonTerminal(id) => *id,
        _ => unreachable!(),
    };
    assert_ne!(id0, id1);
}

#[test]
fn normalize_epsilon_passthrough() {
    let mut g = Grammar::new("eps".into());
    g.rules
        .insert(SymbolId(1), vec![simple_rule(1, vec![Symbol::Epsilon], 0)]);
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();

    assert_eq!(g.rules[&SymbolId(1)][0].rhs, vec![Symbol::Epsilon]);
}

#[test]
fn normalize_external_passthrough() {
    use adze_ir::ExternalToken;

    let mut g = Grammar::new("ext".into());
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(20),
    });
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::External(SymbolId(20))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();

    assert_eq!(
        g.rules[&SymbolId(1)][0].rhs,
        vec![Symbol::External(SymbolId(20))]
    );
}

#[test]
fn normalize_preserves_precedence_and_associativity() {
    use adze_ir::{Associativity, PrecedenceKind};

    let mut g = Grammar::new("prec".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();

    let rule = &g.rules[&SymbolId(1)][0];
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(5)));
    assert_eq!(rule.associativity, Some(Associativity::Left));
}

#[test]
fn normalize_preserves_fields() {
    use adze_ir::FieldId;

    let mut g = Grammar::new("fields".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
            production_id: ProductionId(0),
        }],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    g.normalize();

    let rule = &g.rules[&SymbolId(1)][0];
    assert_eq!(rule.fields, vec![(FieldId(0), 0), (FieldId(1), 1)]);
}

#[test]
fn validation_catches_non_productive_symbols() {
    let mut g = Grammar::new("nonprod".into());
    // A → B, B → A: neither can produce a terminal string
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0)],
    );
    g.rules.insert(
        SymbolId(2),
        vec![simple_rule(2, vec![Symbol::NonTerminal(SymbolId(1))], 1)],
    );
    g.rule_names.insert(SymbolId(1), "A".into());
    g.rule_names.insert(SymbolId(2), "B".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. })),
        "Should detect non-productive symbols"
    );
}

#[test]
fn validation_stats_are_populated() {
    let mut g = Grammar::new("stats".into());
    let (t1, tok1) = token(10, "a", "a");
    let (t2, tok2) = token(11, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(
            1,
            vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ],
            0,
        )],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(result.stats.total_rules >= 1);
    assert!(result.stats.total_tokens >= 2);
    assert!(result.stats.max_rule_length >= 2);
}

#[test]
fn optimizer_returns_stats() {
    let mut g = Grammar::new("stats".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // Stats should be non-negative (just check it doesn't panic)
    let _ = stats.total();
}

#[test]
fn optimize_grammar_convenience_function() {
    use adze_ir::optimize_grammar;

    let mut g = Grammar::new("conv".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.rules.insert(
        SymbolId(1),
        vec![simple_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0)],
    );
    g.rule_names.insert(SymbolId(1), "source_file".into());

    let result = optimize_grammar(g);
    assert!(result.is_ok());
    let optimized = result.unwrap();
    assert!(!optimized.rules.is_empty());
}

#[test]
fn validation_catches_invalid_field_index() {
    use adze_ir::FieldId;

    let mut g = Grammar::new("bad_field".into());
    let (tid, tok) = token(10, "a", "a");
    g.tokens.insert(tid, tok);
    // Field index 5 but rhs has only 1 element
    g.rules.insert(
        SymbolId(1),
        vec![Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(10))],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(0), 5)],
            production_id: ProductionId(0),
        }],
    );
    g.rule_names.insert(SymbolId(1), "start".into());

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. })),
        "Should detect invalid field index"
    );
}
