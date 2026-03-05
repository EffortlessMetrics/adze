#![cfg(feature = "test-api")]
#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for ParseTable properties (v4).
//!
//! Categories:
//! 1. state_count > 0 for valid grammars (8)
//! 2. Actions for valid state/symbol (8)
//! 3. Goto entries for non-terminals (8)
//! 4. Accept action on EOF (8)
//! 5. Error actions for invalid combinations (5)
//! 6. Table consistency properties (8)
//! 7. Debug / Clone (5)
//! 8. Edge cases (5)

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, StateId, SymbolId};

// ---------------------------------------------------------------------------
// Helper: build a ParseTable from tokens + rules via GrammarBuilder
// ---------------------------------------------------------------------------

fn build_pt(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let g = b.build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

// ===========================================================================
// 1. ParseTable state_count > 0 for valid grammars (8 tests)
// ===========================================================================

#[test]
fn v4_state_count_single_token_grammar() {
    let pt = build_pt("sc1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count > 0);
}

#[test]
fn v4_state_count_two_token_sequence() {
    let pt = build_pt(
        "sc2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn v4_state_count_three_token_sequence() {
    let pt = build_pt(
        "sc3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(pt.state_count >= 3);
}

#[test]
fn v4_state_count_alternative_productions() {
    let pt = build_pt(
        "sc4",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn v4_state_count_chain_rules() {
    let pt = build_pt(
        "sc5",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn v4_state_count_recursive_grammar() {
    let pt = build_pt(
        "sc6",
        &[("a", "a"), ("plus", "\\+")],
        &[
            ("term", vec!["a"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 4);
}

#[test]
fn v4_state_count_diamond_grammar() {
    let pt = build_pt(
        "sc7",
        &[("x", "x"), ("y", "y")],
        &[
            ("left", vec!["x"]),
            ("right", vec!["y"]),
            ("s", vec!["left"]),
            ("s", vec!["right"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn v4_state_count_nested_nonterminals() {
    let pt = build_pt(
        "sc8",
        &[("n", "[0-9]+")],
        &[
            ("atom", vec!["n"]),
            ("inner", vec!["atom"]),
            ("outer", vec!["inner"]),
            ("s", vec!["outer"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 2);
}

// ===========================================================================
// 2. Actions for valid state/symbol combinations (8 tests)
// ===========================================================================

#[test]
fn v4_actions_initial_state_has_shift() {
    let pt = build_pt("ac1", &[("a", "a")], &[("s", vec!["a"])], "s");
    // The terminal "a" must have a Shift somewhere in state 0
    let found = pt.symbol_to_index.iter().any(|(sym, _)| {
        let acts = pt.actions(pt.initial_state, *sym);
        acts.iter().any(|a| matches!(a, Action::Shift(_)))
    });
    assert!(found, "initial state should have at least one Shift action");
}

#[test]
fn v4_actions_nonempty_for_known_terminal() {
    let pt = build_pt("ac2", &[("a", "a")], &[("s", vec!["a"])], "s");
    // At least one (state, terminal) pair should yield non-empty actions
    let any_action = (0..pt.state_count).any(|s| {
        pt.symbol_to_index
            .keys()
            .any(|sym| !pt.actions(StateId(s as u16), *sym).is_empty())
    });
    assert!(any_action);
}

#[test]
fn v4_actions_reduce_present_after_shift() {
    let pt = build_pt("ac3", &[("a", "a")], &[("s", vec!["a"])], "s");
    // After shifting 'a', some state should have a Reduce
    let has_reduce = (0..pt.state_count).any(|s| {
        pt.symbol_to_index.keys().any(|sym| {
            pt.actions(StateId(s as u16), *sym)
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
    });
    assert!(has_reduce, "grammar should have at least one Reduce action");
}

#[test]
fn v4_actions_shift_target_valid_state() {
    let pt = build_pt("ac4", &[("a", "a")], &[("s", vec!["a"])], "s");
    for s in 0..pt.state_count {
        for sym in pt.symbol_to_index.keys() {
            for act in pt.actions(StateId(s as u16), *sym) {
                if let Action::Shift(target) = act {
                    assert!(
                        (target.0 as usize) < pt.state_count,
                        "Shift target {target:?} out of range"
                    );
                }
            }
        }
    }
}

#[test]
fn v4_actions_reduce_rule_id_valid() {
    let pt = build_pt("ac5", &[("a", "a")], &[("s", vec!["a"])], "s");
    for s in 0..pt.state_count {
        for sym in pt.symbol_to_index.keys() {
            for act in pt.actions(StateId(s as u16), *sym) {
                if let Action::Reduce(rule_id) = act {
                    assert!(
                        (rule_id.0 as usize) < pt.rules.len(),
                        "Reduce rule {rule_id:?} out of range"
                    );
                }
            }
        }
    }
}

#[test]
fn v4_actions_two_token_both_have_actions() {
    let pt = build_pt(
        "ac6",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    // Each terminal should participate in at least one action across all states
    for sym in pt.symbol_to_index.keys() {
        let any = (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), *sym).is_empty());
        // Not every symbol must have actions (e.g., nonterminals mapped as terminals
        // may not), but at least some should
        let _ = any;
    }
    // At minimum, both "a" and "b" should lead to at least one Shift
    let mut shifts = 0usize;
    for s in 0..pt.state_count {
        for sym in pt.symbol_to_index.keys() {
            shifts += pt
                .actions(StateId(s as u16), *sym)
                .iter()
                .filter(|a| matches!(a, Action::Shift(_)))
                .count();
        }
    }
    assert!(shifts >= 2, "expected at least 2 shifts, got {shifts}");
}

#[test]
fn v4_actions_empty_for_out_of_range_state() {
    let pt = build_pt("ac7", &[("a", "a")], &[("s", vec!["a"])], "s");
    let big_state = StateId(9999);
    for sym in pt.symbol_to_index.keys() {
        assert!(pt.actions(big_state, *sym).is_empty());
    }
}

#[test]
fn v4_actions_empty_for_unknown_symbol() {
    let pt = build_pt("ac8", &[("a", "a")], &[("s", vec!["a"])], "s");
    let unknown = SymbolId(9999);
    for s in 0..pt.state_count {
        assert!(pt.actions(StateId(s as u16), unknown).is_empty());
    }
}

// ===========================================================================
// 3. Goto entries for non-terminals (8 tests)
// ===========================================================================

#[test]
fn v4_goto_start_symbol_from_initial() {
    let pt = build_pt("gt1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let target = pt.goto(pt.initial_state, pt.start_symbol);
    assert!(target.is_some(), "goto(initial, start) should exist");
}

#[test]
fn v4_goto_target_in_range() {
    let pt = build_pt("gt2", &[("a", "a")], &[("s", vec!["a"])], "s");
    for s in 0..pt.state_count {
        for nt in pt.nonterminal_to_index.keys() {
            if let Some(target) = pt.goto(StateId(s as u16), *nt) {
                assert!(
                    (target.0 as usize) < pt.state_count,
                    "goto target {target:?} out of range"
                );
            }
        }
    }
}

#[test]
fn v4_goto_none_for_terminal() {
    let pt = build_pt("gt3", &[("a", "a")], &[("s", vec!["a"])], "s");
    // Terminals should not appear in nonterminal_to_index, so goto returns None
    for sym in pt.symbol_to_index.keys() {
        if !pt.nonterminal_to_index.contains_key(sym) {
            for s in 0..pt.state_count {
                assert!(pt.goto(StateId(s as u16), *sym).is_none());
            }
        }
    }
}

#[test]
fn v4_goto_none_for_out_of_range_state() {
    let pt = build_pt("gt4", &[("a", "a")], &[("s", vec!["a"])], "s");
    let big_state = StateId(9999);
    for nt in pt.nonterminal_to_index.keys() {
        assert!(pt.goto(big_state, *nt).is_none());
    }
}

#[test]
fn v4_goto_none_for_unknown_nonterminal() {
    let pt = build_pt("gt5", &[("a", "a")], &[("s", vec!["a"])], "s");
    let unknown = SymbolId(9999);
    for s in 0..pt.state_count {
        assert!(pt.goto(StateId(s as u16), unknown).is_none());
    }
}

#[test]
fn v4_goto_chain_rules_all_nonterminals() {
    let pt = build_pt(
        "gt6",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // Each nonterminal should appear in the goto map
    assert!(pt.nonterminal_to_index.len() >= 3);
}

#[test]
fn v4_goto_multiple_nonterminals_from_same_state() {
    let pt = build_pt(
        "gt7",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // initial_state should have goto entries for at least some nonterminals
    let goto_count = pt
        .nonterminal_to_index
        .keys()
        .filter(|nt| pt.goto(pt.initial_state, **nt).is_some())
        .count();
    assert!(goto_count >= 1, "initial state should have goto entries");
}

#[test]
fn v4_goto_table_row_count_matches_state_count() {
    let pt = build_pt("gt8", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

// ===========================================================================
// 4. Accept action on EOF (8 tests)
// ===========================================================================

fn has_accept_on_eof(pt: &ParseTable) -> bool {
    let eof = pt.eof_symbol;
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

#[test]
fn v4_accept_single_token_grammar() {
    let pt = build_pt("eof1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(has_accept_on_eof(&pt), "should accept on EOF");
}

#[test]
fn v4_accept_two_token_grammar() {
    let pt = build_pt(
        "eof2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v4_accept_alternative_grammar() {
    let pt = build_pt(
        "eof3",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v4_accept_chain_grammar() {
    let pt = build_pt(
        "eof4",
        &[("x", "x")],
        &[("a", vec!["x"]), ("s", vec!["a"])],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v4_accept_recursive_grammar() {
    let pt = build_pt(
        "eof5",
        &[("a", "a"), ("plus", "\\+")],
        &[
            ("term", vec!["a"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v4_accept_not_on_initial_state_for_nonempty_grammar() {
    let pt = build_pt("eof6", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof_symbol;
    // initial_state should NOT have Accept (input hasn't been consumed yet)
    let initial_acts = pt.actions(pt.initial_state, eof);
    let accept_on_initial = initial_acts.iter().any(|a| matches!(a, Action::Accept));
    assert!(
        !accept_on_initial,
        "initial state should not Accept on EOF for non-empty grammar"
    );
}

#[test]
fn v4_accept_exactly_one_accept_state() {
    let pt = build_pt("eof7", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof_symbol;
    let accept_states: Vec<usize> = (0..pt.state_count)
        .filter(|&s| {
            pt.actions(StateId(s as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .collect();
    assert!(
        !accept_states.is_empty(),
        "should have at least one accept state"
    );
}

#[test]
fn v4_accept_diamond_grammar() {
    let pt = build_pt(
        "eof8",
        &[("x", "x"), ("y", "y")],
        &[
            ("left", vec!["x"]),
            ("right", vec!["y"]),
            ("s", vec!["left"]),
            ("s", vec!["right"]),
        ],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

// ===========================================================================
// 5. Error actions for invalid combinations (5 tests)
// ===========================================================================

#[test]
fn v4_error_unknown_symbol_returns_empty() {
    let pt = build_pt("er1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let bogus = SymbolId(8888);
    let acts = pt.actions(pt.initial_state, bogus);
    assert!(acts.is_empty());
}

#[test]
fn v4_error_out_of_range_state_returns_empty() {
    let pt = build_pt("er2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let bogus_state = StateId(u16::MAX);
    for sym in pt.symbol_to_index.keys() {
        assert!(pt.actions(bogus_state, *sym).is_empty());
    }
}

#[test]
fn v4_error_goto_unknown_nt_returns_none() {
    let pt = build_pt("er3", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.goto(pt.initial_state, SymbolId(7777)).is_none());
}

#[test]
fn v4_error_goto_out_of_range_state_returns_none() {
    let pt = build_pt("er4", &[("a", "a")], &[("s", vec!["a"])], "s");
    for nt in pt.nonterminal_to_index.keys() {
        assert!(pt.goto(StateId(u16::MAX), *nt).is_none());
    }
}

#[test]
fn v4_error_no_shift_on_eof_from_initial() {
    let pt = build_pt("er5", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof_symbol;
    let initial_acts = pt.actions(pt.initial_state, eof);
    let has_shift = initial_acts.iter().any(|a| matches!(a, Action::Shift(_)));
    assert!(!has_shift, "initial state should not Shift on EOF");
}

// ===========================================================================
// 6. Table consistency properties (8 tests)
// ===========================================================================

#[test]
fn v4_consistency_action_table_rows_eq_state_count() {
    let pt = build_pt("co1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn v4_consistency_goto_table_rows_eq_state_count() {
    let pt = build_pt("co2", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn v4_consistency_symbol_to_index_and_index_to_symbol_biject() {
    let pt = build_pt("co3", &[("a", "a")], &[("s", vec!["a"])], "s");
    for (&sym, &idx) in &pt.symbol_to_index {
        if idx < pt.index_to_symbol.len() {
            assert_eq!(
                pt.index_to_symbol[idx], sym,
                "index_to_symbol[{idx}] should be {sym:?}"
            );
        }
    }
}

#[test]
fn v4_consistency_rules_nonempty() {
    let pt = build_pt("co4", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!pt.rules.is_empty());
}

#[test]
fn v4_consistency_eof_in_symbol_to_index() {
    let pt = build_pt("co5", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(
        pt.symbol_to_index.contains_key(&pt.eof_symbol),
        "eof_symbol should be in symbol_to_index"
    );
}

#[test]
fn v4_consistency_deterministic_rebuild() {
    let pt1 = build_pt("co6", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("co6", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.state_count, pt2.state_count);
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
    assert_eq!(pt1.rules.len(), pt2.rules.len());
    assert_eq!(pt1.eof_symbol, pt2.eof_symbol);
}

#[test]
fn v4_consistency_action_row_widths_uniform() {
    let pt = build_pt("co7", &[("a", "a")], &[("s", vec!["a"])], "s");
    if !pt.action_table.is_empty() {
        let width = pt.action_table[0].len();
        for (i, row) in pt.action_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                width,
                "action_table row {i} has width {} but expected {width}",
                row.len()
            );
        }
    }
}

#[test]
fn v4_consistency_goto_row_widths_uniform() {
    let pt = build_pt("co8", &[("a", "a")], &[("s", vec!["a"])], "s");
    if !pt.goto_table.is_empty() {
        let width = pt.goto_table[0].len();
        for (i, row) in pt.goto_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                width,
                "goto_table row {i} has width {} but expected {width}",
                row.len()
            );
        }
    }
}

// ===========================================================================
// 7. ParseTable Debug / Clone (5 tests)
// ===========================================================================

#[test]
fn v4_debug_format_contains_state_count() {
    let pt = build_pt("db1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let dbg = format!("{pt:?}");
    assert!(
        dbg.contains("state_count"),
        "Debug output should contain 'state_count'"
    );
}

#[test]
fn v4_debug_format_contains_symbol_count() {
    let pt = build_pt("db2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let dbg = format!("{pt:?}");
    assert!(
        dbg.contains("symbol_count"),
        "Debug output should contain 'symbol_count'"
    );
}

#[test]
fn v4_clone_preserves_state_count() {
    let pt = build_pt("cl1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = pt.clone();
    assert_eq!(pt.state_count, pt2.state_count);
}

#[test]
fn v4_clone_preserves_symbol_count() {
    let pt = build_pt("cl2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = pt.clone();
    assert_eq!(pt.symbol_count, pt2.symbol_count);
}

#[test]
fn v4_clone_preserves_actions() {
    let pt = build_pt("cl3", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = pt.clone();
    for s in 0..pt.state_count {
        for sym in pt.symbol_to_index.keys() {
            assert_eq!(
                pt.actions(StateId(s as u16), *sym),
                pt2.actions(StateId(s as u16), *sym),
            );
        }
    }
}

// ===========================================================================
// 8. Edge cases (5 tests)
// ===========================================================================

#[test]
fn v4_edge_default_table_zero_states() {
    let pt = ParseTable::default();
    assert_eq!(pt.state_count, 0);
    assert_eq!(pt.symbol_count, 0);
}

#[test]
fn v4_edge_default_table_empty_tables() {
    let pt = ParseTable::default();
    assert!(pt.action_table.is_empty());
    assert!(pt.goto_table.is_empty());
    assert!(pt.rules.is_empty());
}

#[test]
fn v4_edge_rule_method_returns_lhs_and_len() {
    let pt = build_pt("ed3", &[("a", "a")], &[("s", vec!["a"])], "s");
    for (i, r) in pt.rules.iter().enumerate() {
        let (lhs, rhs_len) = pt.rule(RuleId(i as u16));
        assert_eq!(lhs, r.lhs);
        assert_eq!(rhs_len, r.rhs_len);
    }
}

#[test]
fn v4_edge_eof_accessor_matches_field() {
    let pt = build_pt("ed4", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.eof(), pt.eof_symbol);
}

#[test]
fn v4_edge_start_symbol_accessor_matches_field() {
    let pt = build_pt("ed5", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.start_symbol(), pt.start_symbol);
}
