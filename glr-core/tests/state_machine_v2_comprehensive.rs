//! Comprehensive V2 state-machine tests for adze-glr-core covering StateId,
//! Action variants, ParseTable construction, action/goto lookups, multi-action
//! cells (GLR conflicts), table dimensions, and edge cases.

#![allow(clippy::needless_range_loop)]

use adze_glr_core::{
    Action, FirstFollowSets, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ===========================================================================
// Helpers
// ===========================================================================

fn build(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn tok(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

fn nt(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

/// S → a
fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// S → a b c
fn linear_grammar() -> Grammar {
    GrammarBuilder::new("linear")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

/// S → a S | a  (right-recursive)
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// S → S a | a  (left-recursive)
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("lr")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Two non-terminal layers: top → inner ; inner → a
fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("top", vec!["inner"])
        .start("top")
        .build()
}

/// Ambiguous expression grammar: expr → expr + expr | NUM
fn ambiguous_expr_grammar() -> Grammar {
    GrammarBuilder::new("ambig")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Parenthesised nesting: S → ( S ) | a
fn paren_grammar() -> Grammar {
    GrammarBuilder::new("paren")
        .token("(", "(")
        .token(")", ")")
        .token("a", "a")
        .rule("start", vec!["(", "start", ")"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Multiple alternatives: S → a | b | c
fn multi_alt_grammar() -> Grammar {
    GrammarBuilder::new("multi_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build()
}

// ===========================================================================
// 1. StateId construction and properties (5 tests)
// ===========================================================================

#[test]
fn state_id_zero() {
    let s = StateId(0);
    assert_eq!(s.0, 0);
}

#[test]
fn state_id_max() {
    let s = StateId(u16::MAX);
    assert_eq!(s.0, u16::MAX);
}

#[test]
fn state_id_copy_semantics() {
    let s = StateId(42);
    let s2 = s; // Copy, not move
    assert_eq!(s, s2);
}

#[test]
fn state_id_equality() {
    assert_eq!(StateId(5), StateId(5));
    assert_ne!(StateId(5), StateId(6));
}

#[test]
fn state_id_debug_format() {
    let s = StateId(7);
    let dbg = format!("{s:?}");
    assert!(dbg.contains("7"), "Debug should contain the inner value");
}

// ===========================================================================
// 2. Action variant construction (8 tests)
// ===========================================================================

#[test]
fn action_shift_variant() {
    let a = Action::Shift(StateId(3));
    assert!(matches!(a, Action::Shift(StateId(3))));
}

#[test]
fn action_reduce_variant() {
    let a = Action::Reduce(RuleId(5));
    assert!(matches!(a, Action::Reduce(RuleId(5))));
}

#[test]
fn action_accept_variant() {
    assert!(matches!(Action::Accept, Action::Accept));
}

#[test]
fn action_error_variant() {
    assert!(matches!(Action::Error, Action::Error));
}

#[test]
fn action_recover_variant() {
    assert!(matches!(Action::Recover, Action::Recover));
}

#[test]
fn action_fork_with_two_actions() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    match &a {
        Action::Fork(inner) => assert_eq!(inner.len(), 2),
        _ => panic!("expected Fork"),
    }
}

#[test]
fn action_equality() {
    assert_eq!(Action::Shift(StateId(1)), Action::Shift(StateId(1)));
    assert_ne!(Action::Shift(StateId(1)), Action::Shift(StateId(2)));
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn action_clone_preserves_value() {
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    let b = a.clone();
    assert_eq!(a, b);
}

// ===========================================================================
// 3. ParseTable from simple grammars (8 tests)
// ===========================================================================

#[test]
fn single_token_builds_successfully() {
    let g = single_token_grammar();
    let pt = build(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn single_token_sanity_check() {
    let g = single_token_grammar();
    let pt = build(&g);
    sanity_check_tables(&pt).expect("sanity check");
}

#[test]
fn linear_grammar_builds() {
    let g = linear_grammar();
    let pt = build(&g);
    assert!(pt.state_count >= 4, "a→b→c chain needs at least 4 states");
}

#[test]
fn right_recursive_grammar_builds() {
    let g = right_recursive_grammar();
    let pt = build(&g);
    sanity_check_tables(&pt).expect("sanity check");
}

#[test]
fn left_recursive_grammar_builds() {
    let g = left_recursive_grammar();
    let pt = build(&g);
    sanity_check_tables(&pt).expect("sanity check");
}

#[test]
fn nested_grammar_builds() {
    let g = nested_grammar();
    let pt = build(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn paren_grammar_builds() {
    let g = paren_grammar();
    let pt = build(&g);
    sanity_check_tables(&pt).expect("sanity check");
}

#[test]
fn multi_alt_grammar_builds() {
    let g = multi_alt_grammar();
    let pt = build(&g);
    sanity_check_tables(&pt).expect("sanity check");
}

// ===========================================================================
// 4. Actions lookup on built tables (8 tests)
// ===========================================================================

#[test]
fn initial_state_has_shift_on_token() {
    let g = single_token_grammar();
    let pt = build(&g);
    let a_sym = tok(&g, "a");
    let actions = pt.actions(pt.initial_state, a_sym);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on terminal 'a'"
    );
}

#[test]
fn accept_exists_on_eof() {
    let g = single_token_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    let found = (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(found, "some state must Accept on EOF");
}

#[test]
fn unknown_symbol_returns_empty_actions() {
    let g = single_token_grammar();
    let pt = build(&g);
    let bogus = SymbolId(9999);
    assert!(pt.actions(pt.initial_state, bogus).is_empty());
}

#[test]
fn out_of_range_state_returns_empty_actions() {
    let g = single_token_grammar();
    let pt = build(&g);
    let a_sym = tok(&g, "a");
    let far = StateId(pt.state_count as u16 + 100);
    assert!(pt.actions(far, a_sym).is_empty());
}

#[test]
fn linear_shift_sequence() {
    let g = linear_grammar();
    let pt = build(&g);
    let a_sym = tok(&g, "a");
    // Initial state should shift on 'a'
    let actions = pt.actions(pt.initial_state, a_sym);
    assert!(actions.iter().any(|a| matches!(a, Action::Shift(_))));
}

#[test]
fn linear_second_token_reachable() {
    let g = linear_grammar();
    let pt = build(&g);
    let a_sym = tok(&g, "a");
    let b_sym = tok(&g, "b");
    // Find the state we shift to on 'a'
    let actions = pt.actions(pt.initial_state, a_sym);
    let shift_state = actions.iter().find_map(|a| match a {
        Action::Shift(s) => Some(*s),
        _ => None,
    });
    assert!(shift_state.is_some(), "should shift on 'a'");
    let next = shift_state.unwrap();
    let b_actions = pt.actions(next, b_sym);
    assert!(
        b_actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "after 'a', state should shift on 'b'"
    );
}

#[test]
fn reduce_action_present_after_full_rhs() {
    let g = single_token_grammar();
    let pt = build(&g);
    let a_sym = tok(&g, "a");
    let eof = pt.eof();
    // Shift on 'a' from initial state
    let shift_target = pt
        .actions(pt.initial_state, a_sym)
        .iter()
        .find_map(|a| match a {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift on 'a'");
    // After shifting 'a', we should see Reduce or Accept on EOF
    let after = pt.actions(shift_target, eof);
    assert!(
        after
            .iter()
            .any(|a| matches!(a, Action::Reduce(_) | Action::Accept)),
        "after complete RHS, expect Reduce or Accept on EOF"
    );
}

#[test]
fn multi_alt_all_tokens_shift_from_initial() {
    let g = multi_alt_grammar();
    let pt = build(&g);
    for name in &["a", "b", "c"] {
        let sym = tok(&g, name);
        let actions = pt.actions(pt.initial_state, sym);
        assert!(
            actions.iter().any(|a| matches!(a, Action::Shift(_))),
            "initial state should shift on '{name}'"
        );
    }
}

// ===========================================================================
// 5. Goto lookup on built tables (5 tests)
// ===========================================================================

#[test]
fn goto_exists_for_start_from_initial() {
    let g = single_token_grammar();
    let pt = build(&g);
    let start_sym = nt(&g, "start");
    let target = pt.goto(pt.initial_state, start_sym);
    assert!(target.is_some(), "goto(initial, start) should exist");
}

#[test]
fn goto_returns_none_for_terminal() {
    let g = single_token_grammar();
    let pt = build(&g);
    let a_sym = tok(&g, "a");
    // Terminals should not appear in the GOTO table
    let target = pt.goto(pt.initial_state, a_sym);
    assert!(target.is_none(), "goto on terminal should be None");
}

#[test]
fn goto_returns_none_for_bogus_symbol() {
    let g = single_token_grammar();
    let pt = build(&g);
    let bogus = SymbolId(9999);
    assert!(pt.goto(pt.initial_state, bogus).is_none());
}

#[test]
fn goto_returns_none_for_out_of_range_state() {
    let g = single_token_grammar();
    let pt = build(&g);
    let start_sym = nt(&g, "start");
    let far = StateId(pt.state_count as u16 + 100);
    assert!(pt.goto(far, start_sym).is_none());
}

#[test]
fn nested_goto_inner_nonterminal() {
    let g = nested_grammar();
    let pt = build(&g);
    let inner = nt(&g, "inner");
    // Some state should have a goto entry for the inner nonterminal
    let found = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), inner).is_some());
    assert!(found, "at least one state should have goto for 'inner'");
}

// ===========================================================================
// 6. Multi-action cells / GLR conflicts (8 tests)
// ===========================================================================

#[test]
fn ambiguous_grammar_builds() {
    let g = ambiguous_expr_grammar();
    let pt = build(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn ambiguous_grammar_has_accept() {
    let g = ambiguous_expr_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    let found = (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(found, "ambiguous grammar should still have Accept");
}

#[test]
fn ambiguous_expr_has_multi_action_or_fork() {
    let g = ambiguous_expr_grammar();
    let pt = build(&g);
    // In a shift/reduce conflict grammar, at least one cell should have
    // multiple actions or a Fork action.
    let has_multi = (0..pt.state_count).any(|s| {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            if cell.len() > 1 {
                return true;
            }
            if cell.iter().any(|a| matches!(a, Action::Fork(_))) {
                return true;
            }
        }
        false
    });
    assert!(
        has_multi,
        "ambiguous grammar should produce multi-action cells or Fork"
    );
}

#[test]
fn fork_contains_shift_and_reduce() {
    let g = ambiguous_expr_grammar();
    let pt = build(&g);
    // Find any cell with a Fork containing both Shift and Reduce
    let mut found_shift_reduce = false;
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            let has_shift = cell.iter().any(|a| {
                matches!(a, Action::Shift(_))
                    || matches!(a, Action::Fork(inner) if inner.iter().any(|i| matches!(i, Action::Shift(_))))
            });
            let has_reduce = cell.iter().any(|a| {
                matches!(a, Action::Reduce(_))
                    || matches!(a, Action::Fork(inner) if inner.iter().any(|i| matches!(i, Action::Reduce(_))))
            });
            if has_shift && has_reduce {
                found_shift_reduce = true;
            }
        }
    }
    assert!(
        found_shift_reduce,
        "ambiguous expr should have shift/reduce conflict"
    );
}

#[test]
fn multi_action_cell_all_valid_actions() {
    let g = ambiguous_expr_grammar();
    let pt = build(&g);
    // Every action in every cell should be a valid variant
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                assert!(
                    matches!(
                        a,
                        Action::Shift(_)
                            | Action::Reduce(_)
                            | Action::Accept
                            | Action::Error
                            | Action::Recover
                            | Action::Fork(_)
                    ),
                    "unexpected action variant: {a:?}"
                );
            }
        }
    }
}

#[test]
fn left_recursive_no_ambiguity() {
    // S → S a | a has no ambiguity under LR(1)
    let g = left_recursive_grammar();
    let pt = build(&g);
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            // No Fork expected
            assert!(
                !cell.iter().any(|a| matches!(a, Action::Fork(_))),
                "left-recursive S→Sa|a should not produce Fork"
            );
        }
    }
}

#[test]
fn right_recursive_no_ambiguity() {
    // S → a S | a should not be ambiguous under LR(1)
    let g = right_recursive_grammar();
    let pt = build(&g);
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            assert!(
                cell.len() <= 1,
                "right-recursive S→aS|a should have at most 1 action per cell, got {} in state {s} col {col}",
                cell.len()
            );
        }
    }
}

#[test]
fn ambiguous_grammar_sanity_check_passes() {
    // Even with conflicts, sanity check should pass (GLR allows them)
    let g = ambiguous_expr_grammar();
    let pt = build(&g);
    sanity_check_tables(&pt).expect("sanity check should pass for GLR table");
}

// ===========================================================================
// 7. State count and symbol count (5 tests)
// ===========================================================================

#[test]
fn state_count_matches_action_table_rows() {
    let g = single_token_grammar();
    let pt = build(&g);
    assert_eq!(
        pt.state_count,
        pt.action_table.len(),
        "state_count must equal action_table row count"
    );
}

#[test]
fn state_count_matches_goto_table_rows() {
    let g = single_token_grammar();
    let pt = build(&g);
    assert_eq!(
        pt.state_count,
        pt.goto_table.len(),
        "state_count must equal goto_table row count"
    );
}

#[test]
fn symbol_to_index_consistent_with_index_to_symbol() {
    let g = linear_grammar();
    let pt = build(&g);
    for (&sym, &idx) in &pt.symbol_to_index {
        assert!(
            idx < pt.index_to_symbol.len(),
            "index {idx} out of range for index_to_symbol"
        );
        assert_eq!(
            pt.index_to_symbol[idx], sym,
            "round-trip: index_to_symbol[{idx}] should be {sym:?}"
        );
    }
}

#[test]
fn more_states_for_more_tokens() {
    let g1 = single_token_grammar();
    let g2 = linear_grammar();
    let pt1 = build(&g1);
    let pt2 = build(&g2);
    assert!(
        pt2.state_count >= pt1.state_count,
        "3-token chain should have at least as many states as 1-token grammar"
    );
}

#[test]
fn initial_state_within_range() {
    let g = paren_grammar();
    let pt = build(&g);
    assert!(
        (pt.initial_state.0 as usize) < pt.state_count,
        "initial_state must be a valid state index"
    );
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn eof_symbol_not_zero_for_builder_grammars() {
    // Builder grammars assign SymbolIds starting from 1; EOF is typically
    // max_symbol+1, so it should not be 0.
    let g = single_token_grammar();
    let pt = build(&g);
    assert_ne!(
        pt.eof().0,
        0,
        "builder grammar EOF should not be SymbolId(0)"
    );
}

#[test]
fn eof_in_symbol_to_index() {
    let g = single_token_grammar();
    let pt = build(&g);
    assert!(
        pt.symbol_to_index.contains_key(&pt.eof()),
        "EOF symbol must be in symbol_to_index"
    );
}

#[test]
fn rules_vec_nonempty() {
    let g = single_token_grammar();
    let pt = build(&g);
    assert!(
        !pt.rules.is_empty(),
        "parse table rules should not be empty"
    );
}

#[test]
fn rule_method_returns_valid_data() {
    let g = single_token_grammar();
    let pt = build(&g);
    // Rule 0 should exist for the augmented grammar
    let (lhs, rhs_len) = pt.rule(RuleId(0));
    assert!(lhs.0 > 0 || rhs_len > 0, "rule(0) should return valid data");
}

#[test]
fn default_parse_table_has_zero_states() {
    let pt = ParseTable::default();
    assert_eq!(pt.state_count, 0);
    assert!(pt.action_table.is_empty());
    assert!(pt.goto_table.is_empty());
}

#[test]
fn default_parse_table_actions_returns_empty() {
    let pt = ParseTable::default();
    assert!(pt.actions(StateId(0), SymbolId(0)).is_empty());
}

#[test]
fn default_parse_table_goto_returns_none() {
    let pt = ParseTable::default();
    assert!(pt.goto(StateId(0), SymbolId(0)).is_none());
}

#[test]
fn paren_grammar_deeper_nesting_states() {
    // S → ( S ) | a needs enough states to handle nested parens
    let g = paren_grammar();
    let pt = build(&g);
    let lparen = tok(&g, "(");
    // From initial state, both '(' and 'a' should be shiftable
    let a_sym = tok(&g, "a");
    let lp_actions = pt.actions(pt.initial_state, lparen);
    let a_actions = pt.actions(pt.initial_state, a_sym);
    assert!(
        lp_actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on '('"
    );
    assert!(
        a_actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on 'a'"
    );
}

// ===========================================================================
// Bonus: cross-cutting properties (additional tests to reach 55+)
// ===========================================================================

#[test]
fn all_action_table_rows_same_width() {
    let g = linear_grammar();
    let pt = build(&g);
    if let Some(first_row) = pt.action_table.first() {
        let width = first_row.len();
        for (i, row) in pt.action_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                width,
                "action_table row {i} width mismatch: expected {width}, got {}",
                row.len()
            );
        }
    }
}

#[test]
fn all_goto_table_rows_same_width() {
    let g = linear_grammar();
    let pt = build(&g);
    if let Some(first_row) = pt.goto_table.first() {
        let width = first_row.len();
        for (i, row) in pt.goto_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                width,
                "goto_table row {i} width mismatch: expected {width}, got {}",
                row.len()
            );
        }
    }
}

#[test]
fn grammar_ref_preserved_in_table() {
    let g = single_token_grammar();
    let pt = build(&g);
    // The embedded grammar should have the same name
    assert!(
        pt.grammar().name.contains("single"),
        "grammar name should be preserved"
    );
}

#[test]
fn multiple_grammars_produce_distinct_tables() {
    let pt1 = build(&single_token_grammar());
    let pt2 = build(&linear_grammar());
    // Different grammars should produce different state counts or action tables
    let differ =
        pt1.state_count != pt2.state_count || pt1.action_table.len() != pt2.action_table.len();
    assert!(differ, "different grammars should produce distinct tables");
}

#[test]
fn no_accept_in_initial_state_eof_for_multi_token() {
    // For a multi-token grammar, the initial state on EOF should NOT Accept
    // (you need to consume tokens first).
    let g = linear_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    let initial_eof_actions = pt.actions(pt.initial_state, eof);
    assert!(
        !initial_eof_actions
            .iter()
            .any(|a| matches!(a, Action::Accept)),
        "initial state should not Accept on EOF for multi-token grammar"
    );
}

#[test]
fn symbol_id_copy_semantics() {
    let s = SymbolId(42);
    let s2 = s; // Copy
    assert_eq!(s, s2);
}

#[test]
fn rule_id_copy_semantics() {
    let r = RuleId(10);
    let r2 = r; // Copy
    assert_eq!(r, r2);
}
