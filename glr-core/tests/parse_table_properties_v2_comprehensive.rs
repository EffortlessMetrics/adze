//! Comprehensive tests for ParseTable properties v2.
//! Covers: ParseTable fields, ParseRule, Action enum, table lookups,
//! empty/single/multi-state tables, and full pipeline integration.
#![cfg(feature = "test-api")]

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ── Helpers ──

fn build(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for &(tn, tp) in tokens {
        b = b.token(tn, tp);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let gram = b.start(start).build();
    let ff = FirstFollowSets::compute(&gram).unwrap();
    build_lr1_automaton(&gram, &ff).unwrap()
}

fn single_token_table() -> ParseTable {
    build("single", &[("a", "a")], &[("s", vec!["a"])], "s")
}

fn two_token_seq_table() -> ParseTable {
    build(
        "seq",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    )
}

fn alt_table() -> ParseTable {
    build(
        "alt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    )
}

fn chain_table() -> ParseTable {
    build(
        "chain",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    )
}

// ===========================================================================
// 1. ParseTable field access basics
// ===========================================================================

#[test]
fn pt_state_count_positive() {
    let pt = single_token_table();
    assert!(
        pt.state_count > 0,
        "state_count must be > 0 for any grammar"
    );
}

#[test]
fn pt_symbol_count_positive() {
    let pt = single_token_table();
    assert!(pt.symbol_count > 0, "symbol_count must be > 0");
}

#[test]
fn pt_action_table_len_matches_state_count() {
    let pt = single_token_table();
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn pt_goto_table_len_matches_state_count() {
    let pt = single_token_table();
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn pt_rules_nonempty() {
    let pt = single_token_table();
    assert!(
        !pt.rules.is_empty(),
        "pipeline must produce at least one rule"
    );
}

#[test]
fn pt_eof_symbol_accessible() {
    let pt = single_token_table();
    // eof_symbol can be any value; just verify we can read it
    let _eof = pt.eof_symbol;
}

#[test]
fn pt_eof_method_matches_field() {
    let pt = single_token_table();
    assert_eq!(pt.eof(), pt.eof_symbol);
}

#[test]
fn pt_start_symbol_accessible() {
    let pt = single_token_table();
    let _start = pt.start_symbol;
}

#[test]
fn pt_start_symbol_method_matches_field() {
    let pt = single_token_table();
    assert_eq!(pt.start_symbol(), pt.start_symbol);
}

#[test]
fn pt_initial_state_is_zero() {
    let pt = single_token_table();
    assert_eq!(pt.initial_state, StateId(0));
}

// ===========================================================================
// 2. ParseTable — default / empty table
// ===========================================================================

#[test]
fn pt_default_state_count_zero() {
    let pt = ParseTable::default();
    assert_eq!(pt.state_count, 0);
}

#[test]
fn pt_default_symbol_count_zero() {
    let pt = ParseTable::default();
    assert_eq!(pt.symbol_count, 0);
}

#[test]
fn pt_default_action_table_empty() {
    let pt = ParseTable::default();
    assert!(pt.action_table.is_empty());
}

#[test]
fn pt_default_goto_table_empty() {
    let pt = ParseTable::default();
    assert!(pt.goto_table.is_empty());
}

#[test]
fn pt_default_rules_empty() {
    let pt = ParseTable::default();
    assert!(pt.rules.is_empty());
}

#[test]
fn pt_default_eof_symbol() {
    let pt = ParseTable::default();
    assert_eq!(pt.eof_symbol, SymbolId(0));
}

#[test]
fn pt_default_token_count_zero() {
    let pt = ParseTable::default();
    assert_eq!(pt.token_count, 0);
}

#[test]
fn pt_default_field_names_empty() {
    let pt = ParseTable::default();
    assert!(pt.field_names.is_empty());
}

#[test]
fn pt_default_field_map_empty() {
    let pt = ParseTable::default();
    assert!(pt.field_map.is_empty());
}

// ===========================================================================
// 3. ParseRule struct tests
// ===========================================================================

#[test]
fn parse_rule_lhs_is_symbol_id() {
    let pt = single_token_table();
    for rule in &pt.rules {
        let _id: u16 = rule.lhs.0; // lhs is SymbolId(u16)
    }
}

#[test]
fn parse_rule_rhs_len_type() {
    let pt = single_token_table();
    for rule in &pt.rules {
        let _len: u16 = rule.rhs_len; // rhs_len is u16
    }
}

#[test]
fn parse_rule_single_rhs_has_len_1() {
    let pt = single_token_table();
    assert!(
        pt.rules.iter().any(|r| r.rhs_len == 1),
        "s -> a should produce a rule with rhs_len == 1"
    );
}

#[test]
fn parse_rule_two_rhs_has_len_2() {
    let pt = two_token_seq_table();
    assert!(
        pt.rules.iter().any(|r| r.rhs_len == 2),
        "s -> a b should produce a rule with rhs_len == 2"
    );
}

#[test]
fn parse_rule_method_returns_lhs_and_len() {
    let pt = single_token_table();
    for (i, rule) in pt.rules.iter().enumerate() {
        let (lhs, len) = pt.rule(RuleId(i as u16));
        assert_eq!(lhs, rule.lhs);
        assert_eq!(len, rule.rhs_len);
    }
}

#[test]
fn parse_rule_alt_produces_multiple_rules() {
    let pt = alt_table();
    // Two alternative productions for 's'
    assert!(pt.rules.len() >= 2);
}

#[test]
fn parse_rule_chain_grammar_rules() {
    let pt = chain_table();
    // a -> x, b -> a, s -> b => at least 3 user rules
    assert!(pt.rules.len() >= 3);
}

#[test]
fn parse_rule_debug_impl() {
    let pt = single_token_table();
    let dbg = format!("{:?}", pt.rules[0]);
    assert!(dbg.contains("lhs"));
    assert!(dbg.contains("rhs_len"));
}

// ===========================================================================
// 4. Action enum — construction and equality
// ===========================================================================

#[test]
fn action_shift_construction() {
    let a = Action::Shift(StateId(5));
    assert_eq!(a, Action::Shift(StateId(5)));
}

#[test]
fn action_reduce_construction() {
    let a = Action::Reduce(RuleId(3));
    assert_eq!(a, Action::Reduce(RuleId(3)));
}

#[test]
fn action_accept_construction() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn action_error_construction() {
    assert_eq!(Action::Error, Action::Error);
}

#[test]
fn action_fork_empty() {
    let a = Action::Fork(vec![]);
    assert_eq!(a, Action::Fork(vec![]));
}

#[test]
fn action_fork_with_children() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    match &a {
        Action::Fork(children) => assert_eq!(children.len(), 2),
        _ => panic!("expected Fork"),
    }
}

// ===========================================================================
// 5. Action — Clone
// ===========================================================================

#[test]
fn action_clone_shift() {
    let a = Action::Shift(StateId(42));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn action_clone_reduce() {
    let a = Action::Reduce(RuleId(7));
    assert_eq!(a.clone(), a);
}

#[test]
fn action_clone_accept() {
    assert_eq!(Action::Accept.clone(), Action::Accept);
}

#[test]
fn action_clone_error() {
    assert_eq!(Action::Error.clone(), Action::Error);
}

#[test]
fn action_clone_fork() {
    let a = Action::Fork(vec![Action::Accept, Action::Error]);
    let b = a.clone();
    assert_eq!(a, b);
}

// ===========================================================================
// 6. Action — PartialEq / inequality
// ===========================================================================

#[test]
fn action_shift_ne_reduce() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn action_accept_ne_error() {
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn action_shift_ne_different_state() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
}

#[test]
fn action_reduce_ne_different_rule() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
}

#[test]
fn action_fork_ne_empty_vs_nonempty() {
    assert_ne!(Action::Fork(vec![]), Action::Fork(vec![Action::Accept]),);
}

// ===========================================================================
// 7. Action — Debug
// ===========================================================================

#[test]
fn action_debug_shift() {
    let dbg = format!("{:?}", Action::Shift(StateId(10)));
    assert!(dbg.contains("Shift"));
}

#[test]
fn action_debug_reduce() {
    let dbg = format!("{:?}", Action::Reduce(RuleId(2)));
    assert!(dbg.contains("Reduce"));
}

#[test]
fn action_debug_accept() {
    let dbg = format!("{:?}", Action::Accept);
    assert!(dbg.contains("Accept"));
}

#[test]
fn action_debug_error() {
    let dbg = format!("{:?}", Action::Error);
    assert!(dbg.contains("Error"));
}

#[test]
fn action_debug_fork() {
    let dbg = format!("{:?}", Action::Fork(vec![Action::Accept]));
    assert!(dbg.contains("Fork"));
}

// ===========================================================================
// 8. Table lookups — actions() method
// ===========================================================================

#[test]
fn actions_out_of_range_state_returns_empty() {
    let pt = single_token_table();
    let acts = pt.actions(StateId(u16::MAX), SymbolId(0));
    assert!(acts.is_empty());
}

#[test]
fn actions_unknown_symbol_returns_empty() {
    let pt = single_token_table();
    let acts = pt.actions(StateId(0), SymbolId(9999));
    assert!(acts.is_empty());
}

#[test]
fn actions_initial_state_has_at_least_one_action() {
    let pt = single_token_table();
    // The initial state must have an action on some terminal
    let has_action = pt
        .symbol_to_index
        .keys()
        .any(|&sym| !pt.actions(pt.initial_state, sym).is_empty());
    assert!(has_action, "initial state must have at least one action");
}

#[test]
fn actions_contain_accept_somewhere() {
    let pt = single_token_table();
    let has_accept = (0..pt.state_count).any(|s| {
        pt.symbol_to_index.keys().any(|&sym| {
            pt.actions(StateId(s as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
    });
    assert!(
        has_accept,
        "table must contain Accept somewhere for valid grammar"
    );
}

#[test]
fn actions_contain_shift_somewhere() {
    let pt = single_token_table();
    let has_shift = (0..pt.state_count).any(|s| {
        pt.symbol_to_index.keys().any(|&sym| {
            pt.actions(StateId(s as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Shift(_)))
        })
    });
    assert!(has_shift, "table must contain at least one Shift");
}

#[test]
fn actions_contain_reduce_somewhere() {
    let pt = single_token_table();
    let has_reduce = (0..pt.state_count).any(|s| {
        pt.symbol_to_index.keys().any(|&sym| {
            pt.actions(StateId(s as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
    });
    assert!(has_reduce, "table must contain at least one Reduce");
}

// ===========================================================================
// 9. Table lookups — goto() method
// ===========================================================================

#[test]
fn goto_unknown_nonterminal_returns_none() {
    let pt = single_token_table();
    assert_eq!(pt.goto(StateId(0), SymbolId(9999)), None);
}

#[test]
fn goto_out_of_range_state_returns_none() {
    let pt = single_token_table();
    let some_nt = pt.nonterminal_to_index.keys().next().copied();
    if let Some(nt) = some_nt {
        assert_eq!(pt.goto(StateId(u16::MAX), nt), None);
    }
}

#[test]
fn goto_has_valid_entry_for_start_symbol() {
    let pt = single_token_table();
    // From some state, GOTO on the start nonterminal should exist
    let start = pt.start_symbol;
    let has_goto = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), start).is_some());
    assert!(has_goto, "GOTO table should have entry for start symbol");
}

// ===========================================================================
// 10. Multi-state table properties
// ===========================================================================

#[test]
fn multi_state_two_token_seq() {
    let pt = two_token_seq_table();
    // s -> a b needs: initial, after a, after b, accept-ish
    assert!(pt.state_count >= 3);
}

#[test]
fn multi_state_chain_has_more_states() {
    let pt = chain_table();
    // a->x, b->a, s->b : chained nonterminals
    assert!(pt.state_count >= 2);
}

#[test]
fn multi_state_action_rows_consistent_width() {
    let pt = two_token_seq_table();
    if !pt.action_table.is_empty() {
        let width = pt.action_table[0].len();
        for row in &pt.action_table {
            assert_eq!(row.len(), width, "all ACTION rows must have same width");
        }
    }
}

#[test]
fn multi_state_goto_rows_consistent_width() {
    let pt = two_token_seq_table();
    if !pt.goto_table.is_empty() {
        let width = pt.goto_table[0].len();
        for row in &pt.goto_table {
            assert_eq!(row.len(), width, "all GOTO rows must have same width");
        }
    }
}

// ===========================================================================
// 11. eof_symbol — no constraint that eof < symbol_count
// ===========================================================================

#[test]
fn eof_symbol_can_be_any_value() {
    let pt = single_token_table();
    // Just verify we can read and compare it — do NOT assert < symbol_count
    let _v = pt.eof_symbol.0;
}

#[test]
fn eof_on_default_table_is_zero() {
    let pt = ParseTable::default();
    assert_eq!(pt.eof_symbol, SymbolId(0));
}

// ===========================================================================
// 12. ID types are u16
// ===========================================================================

#[test]
fn symbol_id_is_u16() {
    let id = SymbolId(65535);
    assert_eq!(id.0, u16::MAX);
}

#[test]
fn rule_id_is_u16() {
    let id = RuleId(65535);
    assert_eq!(id.0, u16::MAX);
}

#[test]
fn state_id_is_u16() {
    let id = StateId(65535);
    assert_eq!(id.0, u16::MAX);
}

// ===========================================================================
// 13. Determinism — same grammar yields same table
// ===========================================================================

#[test]
fn deterministic_state_count() {
    let a = single_token_table();
    let b = single_token_table();
    assert_eq!(a.state_count, b.state_count);
}

#[test]
fn deterministic_symbol_count() {
    let a = single_token_table();
    let b = single_token_table();
    assert_eq!(a.symbol_count, b.symbol_count);
}

#[test]
fn deterministic_rules_len() {
    let a = single_token_table();
    let b = single_token_table();
    assert_eq!(a.rules.len(), b.rules.len());
}

// ===========================================================================
// 14. symbol_to_index / index_to_symbol consistency
// ===========================================================================

#[test]
fn symbol_to_index_nonempty() {
    let pt = single_token_table();
    assert!(!pt.symbol_to_index.is_empty());
}

#[test]
fn index_to_symbol_len_matches() {
    let pt = single_token_table();
    assert_eq!(pt.index_to_symbol.len(), pt.symbol_to_index.len());
}

#[test]
fn symbol_to_index_roundtrip() {
    let pt = single_token_table();
    for (&sym, &idx) in &pt.symbol_to_index {
        assert_eq!(
            pt.index_to_symbol[idx], sym,
            "roundtrip failed for {:?}",
            sym
        );
    }
}

// ===========================================================================
// 15. nonterminal_to_index
// ===========================================================================

#[test]
fn nonterminal_to_index_nonempty() {
    let pt = single_token_table();
    assert!(
        !pt.nonterminal_to_index.is_empty(),
        "there must be at least one nonterminal (start symbol)"
    );
}

#[test]
fn nonterminal_to_index_contains_start() {
    let pt = single_token_table();
    assert!(
        pt.nonterminal_to_index.contains_key(&pt.start_symbol),
        "nonterminal_to_index should include the start symbol"
    );
}

// ===========================================================================
// 16. Action in real table cells
// ===========================================================================

#[test]
fn action_cell_is_vec_of_actions() {
    let pt = single_token_table();
    for row in &pt.action_table {
        for cell in row {
            // ActionCell = Vec<Action>; each element is an Action
            for action in cell {
                let _ = format!("{:?}", action);
            }
        }
    }
}

#[test]
fn no_fork_in_simple_grammar() {
    let pt = single_token_table();
    for row in &pt.action_table {
        for cell in row {
            for action in cell {
                assert!(
                    !matches!(action, Action::Fork(_)),
                    "simple grammar should not produce Fork actions"
                );
            }
        }
    }
}

// ===========================================================================
// 17. ParseTable Clone and Debug
// ===========================================================================

#[test]
fn parse_table_clone() {
    let pt = single_token_table();
    let pt2 = pt.clone();
    assert_eq!(pt2.state_count, pt.state_count);
    assert_eq!(pt2.symbol_count, pt.symbol_count);
    assert_eq!(pt2.rules.len(), pt.rules.len());
}

#[test]
fn parse_table_debug() {
    let pt = single_token_table();
    let dbg = format!("{:?}", pt);
    assert!(dbg.contains("ParseTable"));
}
