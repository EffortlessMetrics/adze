//! Comprehensive grammar roundtrip and invariant tests.
//!
//! These tests verify that build→normalize→inspect roundtrips preserve
//! grammar invariants and that edge cases are handled correctly.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Count all rules across every LHS in the grammar.
fn total_rule_count(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

/// Return true when every RHS in the grammar contains only simple symbols
/// (Terminal, NonTerminal, External, Epsilon) – i.e. no complex wrappers.
fn all_symbols_simple(g: &Grammar) -> bool {
    g.all_rules().all(|rule| {
        rule.rhs.iter().all(|s| {
            matches!(
                s,
                Symbol::Terminal(_)
                    | Symbol::NonTerminal(_)
                    | Symbol::External(_)
                    | Symbol::Epsilon
            )
        })
    })
}

/// Collect the set of LHS SymbolIds that appear in the grammar.
fn lhs_ids(g: &Grammar) -> Vec<SymbolId> {
    g.rules.keys().copied().collect()
}

// ===========================================================================
// 1. Basic build→inspect roundtrips
// ===========================================================================

#[test]
fn build_minimal_grammar_preserves_name_and_counts() {
    let g = GrammarBuilder::new("tiny")
        .token("A", "a")
        .rule("S", vec!["A"])
        .start("S")
        .build();

    assert_eq!(g.name, "tiny");
    assert_eq!(g.tokens.len(), 1);
    assert_eq!(g.rules.len(), 1); // one LHS
    assert_eq!(total_rule_count(&g), 1);
}

#[test]
fn build_preserves_multiple_alternatives() {
    let g = GrammarBuilder::new("alts")
        .token("A", "a")
        .token("B", "b")
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .start("S")
        .build();

    assert_eq!(g.rules.len(), 1); // single LHS
    assert_eq!(total_rule_count(&g), 2); // two alternatives
}

#[test]
fn start_symbol_is_first_in_rules() {
    let g = GrammarBuilder::new("order")
        .token("X", "x")
        .rule("beta", vec!["X"])
        .rule("alpha", vec!["beta"])
        .start("alpha")
        .build();

    let first_lhs = *g.rules.keys().next().unwrap();
    let start_name = g.rule_names.get(&first_lhs).unwrap();
    assert_eq!(start_name, "alpha");
}

// ===========================================================================
// 2. Precedence & associativity roundtrips
// ===========================================================================

#[test]
fn rule_with_precedence_roundtrip() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let e_id = g.find_symbol_by_name("expr").unwrap();
    let e_rules = g.get_rules_for_symbol(e_id).unwrap();

    // The rule without precedence should have None
    let plain = e_rules.iter().find(|r| r.precedence.is_none()).unwrap();
    assert_eq!(plain.rhs.len(), 1);

    // Precedence values should be distinct
    let prec_vals: Vec<i16> = e_rules
        .iter()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(prec_vals.len(), 2);
    assert_ne!(prec_vals[0], prec_vals[1]);
}

#[test]
fn right_associativity_preserved() {
    let g = GrammarBuilder::new("rassoc")
        .token("N", "n")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let e_rules = g
        .get_rules_for_symbol(g.find_symbol_by_name("expr").unwrap())
        .unwrap();
    let exp_rule = e_rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::Right))
        .expect("expected right-associative rule");
    assert_eq!(exp_rule.precedence, Some(PrecedenceKind::Static(3)));
}

// ===========================================================================
// 3. Normalize roundtrips
// ===========================================================================

#[test]
fn normalize_optional_creates_two_aux_rules() {
    let mut g = Grammar::new("opt".into());

    // Manually add a token and a rule with an Optional symbol
    let tok_id = SymbolId(1);
    g.tokens.insert(
        tok_id,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let s_id = SymbolId(2);
    g.rule_names.insert(s_id, "S".into());
    g.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    // After normalization, no complex symbols should remain
    assert!(all_symbols_simple(&g));

    // The aux non-terminal should have exactly 2 productions (inner | ε)
    let aux_rules_count: usize = g
        .rules
        .iter()
        .filter(|(id, _)| **id != s_id)
        .map(|(_, rs)| rs.len())
        .sum();
    assert_eq!(aux_rules_count, 2);
}

#[test]
fn normalize_repeat_creates_two_aux_rules() {
    let mut g = Grammar::new("rep".into());

    let tok_id = SymbolId(1);
    g.tokens.insert(
        tok_id,
        Token {
            name: "X".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    let s_id = SymbolId(2);
    g.rule_names.insert(s_id, "S".into());
    g.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    assert!(all_symbols_simple(&g));

    // aux -> aux X | ε  →  2 rules for the aux symbol
    let aux_rules_count: usize = g
        .rules
        .iter()
        .filter(|(id, _)| **id != s_id)
        .map(|(_, rs)| rs.len())
        .sum();
    assert_eq!(aux_rules_count, 2);
}

#[test]
fn normalize_repeat_one_creates_two_aux_rules() {
    let mut g = Grammar::new("rep1".into());

    let tok_id = SymbolId(1);
    g.tokens.insert(
        tok_id,
        Token {
            name: "Y".into(),
            pattern: TokenPattern::String("y".into()),
            fragile: false,
        },
    );
    let s_id = SymbolId(2);
    g.rule_names.insert(s_id, "S".into());
    g.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    assert!(all_symbols_simple(&g));

    // aux -> aux Y | Y  →  2 rules, neither is epsilon
    let aux_lhs: Vec<SymbolId> = g.rules.keys().copied().filter(|id| *id != s_id).collect();
    assert_eq!(aux_lhs.len(), 1);
    let aux_rules = &g.rules[&aux_lhs[0]];
    assert_eq!(aux_rules.len(), 2);
    // RepeatOne should NOT produce an epsilon rule
    let has_epsilon = aux_rules
        .iter()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)));
    assert!(!has_epsilon);
}

#[test]
fn normalize_choice_fans_out_to_alternatives() {
    let mut g = Grammar::new("choice".into());

    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    g.tokens.insert(
        a_id,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b_id,
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    let s_id = SymbolId(3);
    g.rule_names.insert(s_id, "S".into());
    g.add_rule(Rule {
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

    g.normalize();

    assert!(all_symbols_simple(&g));

    // The aux symbol should have exactly 2 productions
    let aux_rules_count: usize = g
        .rules
        .iter()
        .filter(|(id, _)| **id != s_id)
        .map(|(_, rs)| rs.len())
        .sum();
    assert_eq!(aux_rules_count, 2);
}

#[test]
fn normalize_sequence_flattens_into_rhs() {
    let mut g = Grammar::new("seq".into());

    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    g.tokens.insert(
        a_id,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b_id,
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    let s_id = SymbolId(3);
    g.rule_names.insert(s_id, "S".into());
    g.add_rule(Rule {
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

    g.normalize();

    assert!(all_symbols_simple(&g));
    // Sequence should be flattened – no new LHS symbols introduced
    assert_eq!(g.rules.len(), 1);
    let s_rules = &g.rules[&s_id];
    assert_eq!(s_rules[0].rhs.len(), 2);
}

#[test]
fn normalize_is_idempotent() {
    let mut g = GrammarBuilder::new("idem")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["N"])
        .start("E")
        .build();

    g.normalize();
    let snapshot_1: Vec<_> = g.all_rules().cloned().collect();

    g.normalize();
    let snapshot_2: Vec<_> = g.all_rules().cloned().collect();

    assert_eq!(snapshot_1.len(), snapshot_2.len());
    for (a, b) in snapshot_1.iter().zip(snapshot_2.iter()) {
        assert_eq!(a.lhs, b.lhs);
        assert_eq!(a.rhs, b.rhs);
    }
}

// ===========================================================================
// 4. Nested complex symbols
// ===========================================================================

#[test]
fn normalize_nested_optional_repeat() {
    // S -> Optional(Repeat(A)) should fully expand
    let mut g = Grammar::new("nested".into());

    let a_id = SymbolId(1);
    g.tokens.insert(
        a_id,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let s_id = SymbolId(2);
    g.rule_names.insert(s_id, "S".into());
    g.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
            Symbol::Terminal(a_id),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    assert!(all_symbols_simple(&g));
    // We should have the original S plus aux symbols for both Optional and Repeat
    assert!(g.rules.len() >= 3);
}

// ===========================================================================
// 5. Structural invariants
// ===========================================================================

#[test]
fn every_rule_lhs_appears_as_key() {
    let g = GrammarBuilder::javascript_like();

    for rule in g.all_rules() {
        assert!(
            g.rules.contains_key(&rule.lhs),
            "LHS {:?} not found as rules key",
            rule.lhs
        );
    }
}

#[test]
fn production_ids_are_unique_within_lhs() {
    let g = GrammarBuilder::javascript_like();

    for (_lhs, rules) in &g.rules {
        let mut seen = std::collections::HashSet::new();
        for rule in rules {
            assert!(
                seen.insert(rule.production_id),
                "Duplicate production_id {:?}",
                rule.production_id
            );
        }
    }
}

#[test]
fn find_symbol_by_name_consistent_with_rule_names() {
    let g = GrammarBuilder::python_like();

    for (id, name) in &g.rule_names {
        let found = g.find_symbol_by_name(name);
        assert_eq!(found, Some(*id), "find_symbol_by_name mismatch for {name}");
    }
}

// ===========================================================================
// 6. Edge cases
// ===========================================================================

#[test]
fn epsilon_rule_survives_normalize() {
    let mut g = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("start", vec![]) // epsilon
        .rule("start", vec!["A"])
        .start("start")
        .build();

    g.normalize();

    let s_id = g.find_symbol_by_name("start").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    let has_eps = s_rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon));
    assert!(has_eps, "epsilon production should survive normalize");
}

#[test]
fn extras_and_externals_preserved_through_normalize() {
    let mut g = GrammarBuilder::new("ext")
        .token("A", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .external("INDENT")
        .rule("S", vec!["A"])
        .start("S")
        .build();

    let extras_before = g.extras.clone();
    let externals_before_len = g.externals.len();

    g.normalize();

    assert_eq!(g.extras, extras_before);
    assert_eq!(g.externals.len(), externals_before_len);
}

#[test]
fn fragile_token_flag_preserved() {
    let g = GrammarBuilder::new("frag")
        .fragile_token("SEMI", ";")
        .token("A", "a")
        .rule("S", vec!["A", "SEMI"])
        .start("S")
        .build();

    let fragile_count = g.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 1);

    let semi = g.tokens.values().find(|t| t.name == "SEMI").unwrap();
    assert!(semi.fragile);
}

#[test]
fn empty_grammar_normalize_is_noop() {
    let mut g = Grammar::new("empty".into());
    g.normalize();
    assert_eq!(total_rule_count(&g), 0);
    assert!(g.tokens.is_empty());
}

#[test]
fn serde_json_roundtrip_preserves_grammar() {
    let g = GrammarBuilder::new("serde")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("E", vec!["E", "+", "E"], 1, Associativity::Left)
        .rule("E", vec!["N"])
        .start("E")
        .build();

    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(total_rule_count(&g), total_rule_count(&g2));
    assert_eq!(lhs_ids(&g), lhs_ids(&g2));

    // Verify precedence survived
    for (r1, r2) in g.all_rules().zip(g2.all_rules()) {
        assert_eq!(r1.precedence, r2.precedence);
        assert_eq!(r1.associativity, r2.associativity);
    }
}

#[test]
fn normalize_then_serde_roundtrip() {
    let mut g = Grammar::new("norm_serde".into());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    let s = SymbolId(2);
    g.rule_names.insert(s, "S".into());
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();
    assert!(all_symbols_simple(&g));

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(total_rule_count(&g), total_rule_count(&g2));
    assert!(all_symbols_simple(&g2));
}
