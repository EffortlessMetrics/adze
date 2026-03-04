//! Comprehensive tests for LR(1) table construction invariants.
//!
//! Validates structural properties: action/goto table dimensions,
//! symbol mapping consistency, rule coverage, and determinism.

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ============================================================================
// Helpers
// ============================================================================

fn build_table(grammar: &mut Grammar) -> ParseTable {
    let ff =
        FirstFollowSets::compute_normalized(grammar).expect("Failed to compute first/follow sets");
    build_lr1_automaton(grammar, &ff).expect("Failed to build automaton")
}

fn find_sym(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .or_else(|| {
            grammar
                .tokens
                .iter()
                .find(|(_, tok)| tok.name == name)
                .map(|(id, _)| *id)
        })
        .unwrap_or_else(|| panic!("Symbol '{}' not found", name))
}

// ============================================================================
// Table dimension invariants
// ============================================================================

#[test]
fn action_rows_eq_state_count() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn goto_rows_eq_state_count() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn action_row_widths_uniform() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    if let Some(first) = t.action_table.first() {
        let w = first.len();
        assert!(t.action_table.iter().all(|r| r.len() == w));
    }
}

#[test]
fn goto_row_widths_uniform() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    if let Some(first) = t.goto_table.first() {
        let w = first.len();
        assert!(t.goto_table.iter().all(|r| r.len() == w));
    }
}

// ============================================================================
// Symbol mapping invariants
// ============================================================================

#[test]
fn eof_in_symbol_to_index() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.symbol_to_index.contains_key(&t.eof_symbol));
}

#[test]
fn terminals_in_symbol_to_index() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.symbol_to_index.contains_key(&find_sym(&g, "x")));
    assert!(t.symbol_to_index.contains_key(&find_sym(&g, "y")));
}

#[test]
fn index_to_symbol_inverse() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    for (&sym, &idx) in &t.symbol_to_index {
        if idx < t.index_to_symbol.len() {
            assert_eq!(t.index_to_symbol[idx], sym);
        }
    }
}

#[test]
fn eof_ne_start() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert_ne!(t.eof_symbol, t.start_symbol);
}

// ============================================================================
// Action presence invariants
// ============================================================================

#[test]
fn table_has_accept() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    let ok = t.action_table.iter().any(|r| {
        r.iter()
            .any(|c| c.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(ok, "Must have Accept");
}

#[test]
fn table_has_shift() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    let ok = t.action_table.iter().any(|r| {
        r.iter()
            .any(|c| c.iter().any(|a| matches!(a, Action::Shift(_))))
    });
    assert!(ok, "Must have Shift");
}

#[test]
fn table_has_reduce() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    let ok = t.action_table.iter().any(|r| {
        r.iter()
            .any(|c| c.iter().any(|a| matches!(a, Action::Reduce(_))))
    });
    assert!(ok, "Must have Reduce");
}

// ============================================================================
// Rules invariants
// ============================================================================

#[test]
fn rules_nonempty() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(!t.rules.is_empty());
}

#[test]
fn rule_lhs_in_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    for r in &t.rules {
        assert!(
            g.rules.contains_key(&r.lhs) || g.tokens.contains_key(&r.lhs),
            "Rule lhs {:?} not in grammar",
            r.lhs
        );
    }
}

// ============================================================================
// Determinism
// ============================================================================

#[test]
fn deterministic_state_count() {
    let mk = || {
        let mut g = GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        build_table(&mut g)
    };
    assert_eq!(mk().state_count, mk().state_count);
}

#[test]
fn deterministic_rule_count() {
    let mk = || {
        let mut g = GrammarBuilder::new("t")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        build_table(&mut g)
    };
    assert_eq!(mk().rules.len(), mk().rules.len());
}

#[test]
fn deterministic_action_table_shape() {
    let mk = || {
        let mut g = GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build();
        build_table(&mut g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(t1.action_table.len(), t2.action_table.len());
    for (r1, r2) in t1.action_table.iter().zip(t2.action_table.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

// ============================================================================
// Grammar complexity scaling
// ============================================================================

#[test]
fn two_rule_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 2);
}

#[test]
fn three_rule_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 2);
}

#[test]
fn chain_grammar_more_states_than_single() {
    let mut g1 = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t1 = build_table(&mut g1);

    let mut g2 = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let t2 = build_table(&mut g2);
    assert!(
        t2.state_count > t1.state_count,
        "Longer sequence should have more states"
    );
}

#[test]
fn nested_nonterminals_build() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("D", vec!["x"])
        .rule("C", vec!["D"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 2);
}

#[test]
fn recursive_grammar_builds() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("A", vec!["a", "A"])
        .rule("A", vec!["a"])
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 2);
}

#[test]
fn arithmetic_grammar_builds() {
    let mut g = GrammarBuilder::new("t")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("T", vec!["num"])
        .rule("E", vec!["E", "plus", "T"])
        .rule("E", vec!["T"])
        .rule("start", vec!["E"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 4);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn single_token_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 2);
}

#[test]
fn epsilon_rule_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("A", vec![])
        .rule("start", vec!["A", "a"])
        .start("start")
        .build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 2);
}

#[test]
fn many_alternatives() {
    let mut builder = GrammarBuilder::new("t");
    for i in 0..8 {
        let name = format!("t{}", i);
        builder = builder.token(&name, &name);
        builder = builder.rule("start", vec![&name]);
    }
    let mut g = builder.start("start").build();
    let t = build_table(&mut g);
    assert!(t.state_count >= 2);
    // Each alternative should be in symbol_to_index
    for i in 0..8 {
        let name = format!("t{}", i);
        assert!(t.symbol_to_index.contains_key(&find_sym(&g, &name)));
    }
}
