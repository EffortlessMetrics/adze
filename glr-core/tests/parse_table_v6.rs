//! ParseTable construction, querying, and properties — v6.
//!
//! Categories (60+ tests):
//! 1. Table construction succeeds for valid grammars (8)
//! 2. Action queries return valid actions (8)
//! 3. Goto entries are valid state IDs (8)
//! 4. State count matches expected range (8)
//! 5. Accept action present for start symbol + EOF (8)
//! 6. Table properties: invalid states, empty actions for unmatched symbols (8)
//! 7. Complex grammars produce larger tables (8)
//! 8. Determinism: same grammar → same table (8)

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{StateId, SymbolId};

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

fn has_accept_on_eof(pt: &ParseTable) -> bool {
    let eof = pt.eof_symbol;
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn count_action_kind(pt: &ParseTable, pred: fn(&Action) -> bool) -> usize {
    let mut n = 0;
    for s in 0..pt.state_count {
        for sym in pt.symbol_to_index.keys() {
            n += pt
                .actions(StateId(s as u16), *sym)
                .iter()
                .filter(|a| pred(a))
                .count();
        }
    }
    n
}

// ===========================================================================
// 1. Table construction succeeds for valid grammars (8 tests)
// ===========================================================================

#[test]
fn v6_construct_single_token() {
    let pt = build_pt("c1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count > 0);
}

#[test]
fn v6_construct_two_token_sequence() {
    let pt = build_pt(
        "c2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(!pt.rules.is_empty());
}

#[test]
fn v6_construct_three_token_sequence() {
    let pt = build_pt(
        "c3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(!pt.action_table.is_empty());
}

#[test]
fn v6_construct_alternative_productions() {
    let pt = build_pt(
        "c4",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.state_count > 0);
    assert!(pt.rules.len() >= 2);
}

#[test]
fn v6_construct_chain_rules() {
    let pt = build_pt(
        "c5",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(pt.nonterminal_to_index.len() >= 2);
}

#[test]
fn v6_construct_recursive_grammar() {
    let pt = build_pt(
        "c6",
        &[("n", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["n"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 4);
}

#[test]
fn v6_construct_diamond_grammar() {
    let pt = build_pt(
        "c7",
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
fn v6_construct_nested_nonterminals() {
    let pt = build_pt(
        "c8",
        &[("tok", "t")],
        &[
            ("atom", vec!["tok"]),
            ("inner", vec!["atom"]),
            ("outer", vec!["inner"]),
            ("s", vec!["outer"]),
        ],
        "s",
    );
    assert!(pt.nonterminal_to_index.len() >= 4);
}

// ===========================================================================
// 2. Action queries return valid actions (8 tests)
// ===========================================================================

#[test]
fn v6_actions_initial_has_shift() {
    let pt = build_pt("a1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let found = pt.symbol_to_index.keys().any(|sym| {
        pt.actions(pt.initial_state, *sym)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    });
    assert!(found, "initial state should have at least one Shift");
}

#[test]
fn v6_actions_shift_targets_in_range() {
    let pt = build_pt(
        "a2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
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
fn v6_actions_reduce_present() {
    let pt = build_pt("a3", &[("a", "a")], &[("s", vec!["a"])], "s");
    let n = count_action_kind(&pt, |a| matches!(a, Action::Reduce(_)));
    assert!(n > 0, "grammar should have at least one Reduce");
}

#[test]
fn v6_actions_reduce_rule_ids_valid() {
    let pt = build_pt("a4", &[("a", "a")], &[("s", vec!["a"])], "s");
    for s in 0..pt.state_count {
        for sym in pt.symbol_to_index.keys() {
            for act in pt.actions(StateId(s as u16), *sym) {
                if let Action::Reduce(rule_id) = act {
                    assert!(
                        (rule_id.0 as usize) < pt.rules.len(),
                        "Reduce rule {rule_id:?} out of range (max {})",
                        pt.rules.len()
                    );
                }
            }
        }
    }
}

#[test]
fn v6_actions_nonempty_for_some_state_symbol_pair() {
    let pt = build_pt("a5", &[("a", "a")], &[("s", vec!["a"])], "s");
    let any_action = (0..pt.state_count).any(|s| {
        pt.symbol_to_index
            .keys()
            .any(|sym| !pt.actions(StateId(s as u16), *sym).is_empty())
    });
    assert!(
        any_action,
        "table should have at least one non-empty action cell"
    );
}

#[test]
fn v6_actions_sequence_grammar_has_multiple_shifts() {
    let pt = build_pt(
        "a6",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    let shifts = count_action_kind(&pt, |a| matches!(a, Action::Shift(_)));
    assert!(
        shifts >= 3,
        "three-token sequence needs >= 3 shifts, got {shifts}"
    );
}

#[test]
fn v6_actions_alternative_grammar_has_shifts_for_both() {
    let pt = build_pt(
        "a7",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let shifts = count_action_kind(&pt, |a| matches!(a, Action::Shift(_)));
    assert!(
        shifts >= 2,
        "alternative grammar needs >= 2 shifts, got {shifts}"
    );
}

#[test]
fn v6_actions_rule_method_consistent_with_reduce() {
    let pt = build_pt("a8", &[("a", "a")], &[("s", vec!["a"])], "s");
    for s in 0..pt.state_count {
        for sym in pt.symbol_to_index.keys() {
            for act in pt.actions(StateId(s as u16), *sym) {
                if let Action::Reduce(rid) = act {
                    let (lhs, rhs_len) = pt.rule(*rid);
                    assert!(pt.nonterminal_to_index.contains_key(&lhs));
                    assert!(rhs_len <= 64, "rhs_len {rhs_len} unexpectedly large");
                }
            }
        }
    }
}

// ===========================================================================
// 3. Goto entries are valid state IDs (8 tests)
// ===========================================================================

#[test]
fn v6_goto_start_from_initial() {
    let pt = build_pt("g1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let target = pt.goto(pt.initial_state, pt.start_symbol());
    assert!(target.is_some(), "goto(initial, start) should exist");
}

#[test]
fn v6_goto_targets_in_range() {
    let pt = build_pt("g2", &[("a", "a")], &[("s", vec!["a"])], "s");
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
fn v6_goto_none_for_terminals() {
    let pt = build_pt("g3", &[("a", "a")], &[("s", vec!["a"])], "s");
    for sym in pt.symbol_to_index.keys() {
        if !pt.nonterminal_to_index.contains_key(sym) {
            for s in 0..pt.state_count {
                assert!(pt.goto(StateId(s as u16), *sym).is_none());
            }
        }
    }
}

#[test]
fn v6_goto_none_for_unknown_nt() {
    let pt = build_pt("g4", &[("a", "a")], &[("s", vec!["a"])], "s");
    let bogus = SymbolId(9999);
    for s in 0..pt.state_count {
        assert!(pt.goto(StateId(s as u16), bogus).is_none());
    }
}

#[test]
fn v6_goto_none_for_out_of_range_state() {
    let pt = build_pt("g5", &[("a", "a")], &[("s", vec!["a"])], "s");
    for nt in pt.nonterminal_to_index.keys() {
        assert!(pt.goto(StateId(9999), *nt).is_none());
    }
}

#[test]
fn v6_goto_chain_rules_all_nts_present() {
    let pt = build_pt(
        "g6",
        &[("x", "x")],
        &[
            ("inner", vec!["x"]),
            ("mid", vec!["inner"]),
            ("s", vec!["mid"]),
        ],
        "s",
    );
    assert!(pt.nonterminal_to_index.len() >= 3);
}

#[test]
fn v6_goto_table_rows_match_state_count() {
    let pt = build_pt("g7", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn v6_goto_row_widths_uniform() {
    let pt = build_pt(
        "g8",
        &[("a", "a"), ("b", "b")],
        &[("mid", vec!["a"]), ("s", vec!["mid", "b"])],
        "s",
    );
    if !pt.goto_table.is_empty() {
        let width = pt.goto_table[0].len();
        for (i, row) in pt.goto_table.iter().enumerate() {
            assert_eq!(row.len(), width, "goto row {i} width mismatch");
        }
    }
}

// ===========================================================================
// 4. State count matches expected range (8 tests)
// ===========================================================================

#[test]
fn v6_state_count_positive_for_minimal() {
    let pt = build_pt("sc1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count > 0);
}

#[test]
fn v6_state_count_at_least_two_for_sequence() {
    let pt = build_pt(
        "sc2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn v6_state_count_at_least_three_for_triple() {
    let pt = build_pt(
        "sc3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(pt.state_count >= 3);
}

#[test]
fn v6_state_count_grows_with_tokens() {
    let pt2 = build_pt(
        "sc4a",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let pt4 = build_pt(
        "sc4b",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a", "b", "c", "d"])],
        "s",
    );
    assert!(
        pt4.state_count > pt2.state_count,
        "longer sequence should have more states: {} vs {}",
        pt4.state_count,
        pt2.state_count
    );
}

#[test]
fn v6_state_count_at_least_four_for_recursive() {
    let pt = build_pt(
        "sc5",
        &[("n", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["n"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 4);
}

#[test]
fn v6_state_count_action_table_consistent() {
    let pt = build_pt("sc6", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn v6_state_count_initial_state_in_range() {
    let pt = build_pt("sc7", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!((pt.initial_state.0 as usize) < pt.state_count);
}

#[test]
fn v6_state_count_default_table_is_zero() {
    let pt = ParseTable::default();
    assert_eq!(pt.state_count, 0);
}

// ===========================================================================
// 5. Accept action present for start symbol + EOF (8 tests)
// ===========================================================================

#[test]
fn v6_accept_single_token() {
    let pt = build_pt("eof1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v6_accept_two_token_sequence() {
    let pt = build_pt(
        "eof2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v6_accept_alternative_productions() {
    let pt = build_pt(
        "eof3",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v6_accept_chain_grammar() {
    let pt = build_pt(
        "eof4",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v6_accept_recursive_grammar() {
    let pt = build_pt(
        "eof5",
        &[("n", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["n"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    assert!(has_accept_on_eof(&pt));
}

#[test]
fn v6_accept_not_on_initial_state() {
    let pt = build_pt("eof6", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof_symbol;
    let initial_acts = pt.actions(pt.initial_state, eof);
    let on_initial = initial_acts.iter().any(|a| matches!(a, Action::Accept));
    assert!(
        !on_initial,
        "initial state should not Accept for non-empty grammar"
    );
}

#[test]
fn v6_accept_at_least_one_accept_state() {
    let pt = build_pt("eof7", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof_symbol;
    let accept_count = (0..pt.state_count)
        .filter(|&s| {
            pt.actions(StateId(s as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .count();
    assert!(accept_count >= 1, "expected at least one accept state");
}

#[test]
fn v6_accept_diamond_grammar() {
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
// 6. Table properties: invalid states, empty actions for unmatched (8 tests)
// ===========================================================================

#[test]
fn v6_props_unknown_symbol_empty() {
    let pt = build_pt("p1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let bogus = SymbolId(8888);
    assert!(pt.actions(pt.initial_state, bogus).is_empty());
}

#[test]
fn v6_props_out_of_range_state_empty() {
    let pt = build_pt("p2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let big = StateId(u16::MAX);
    for sym in pt.symbol_to_index.keys() {
        assert!(pt.actions(big, *sym).is_empty());
    }
}

#[test]
fn v6_props_goto_out_of_range_none() {
    let pt = build_pt("p3", &[("a", "a")], &[("s", vec!["a"])], "s");
    for nt in pt.nonterminal_to_index.keys() {
        assert!(pt.goto(StateId(u16::MAX), *nt).is_none());
    }
}

#[test]
fn v6_props_goto_bogus_nt_none() {
    let pt = build_pt("p4", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.goto(pt.initial_state, SymbolId(7777)).is_none());
}

#[test]
fn v6_props_no_shift_on_eof_from_initial() {
    let pt = build_pt("p5", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof_symbol;
    let initial_acts = pt.actions(pt.initial_state, eof);
    let has_shift = initial_acts.iter().any(|a| matches!(a, Action::Shift(_)));
    assert!(!has_shift, "should not Shift on EOF from initial state");
}

#[test]
fn v6_props_eof_in_symbol_to_index() {
    let pt = build_pt("p6", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.symbol_to_index.contains_key(&pt.eof_symbol));
}

#[test]
fn v6_props_default_table_empty_actions() {
    let pt = ParseTable::default();
    assert!(pt.action_table.is_empty());
    assert!(pt.goto_table.is_empty());
}

#[test]
fn v6_props_action_row_widths_uniform() {
    let pt = build_pt(
        "p8",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    if !pt.action_table.is_empty() {
        let width = pt.action_table[0].len();
        for (i, row) in pt.action_table.iter().enumerate() {
            assert_eq!(row.len(), width, "action row {i} width mismatch");
        }
    }
}

// ===========================================================================
// 7. Complex grammars produce larger tables (8 tests)
// ===========================================================================

#[test]
fn v6_complex_more_tokens_more_states() {
    let small = build_pt("cx1a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let big = build_pt(
        "cx1b",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(big.state_count > small.state_count);
}

#[test]
fn v6_complex_more_rules_more_rules_vec() {
    let small = build_pt("cx2a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let big = build_pt(
        "cx2b",
        &[("a", "a"), ("b", "b")],
        &[("mid", vec!["a"]), ("s", vec!["mid", "b"])],
        "s",
    );
    assert!(big.rules.len() >= small.rules.len());
}

#[test]
fn v6_complex_recursive_more_states_than_flat() {
    let flat = build_pt("cx3a", &[("n", "[0-9]+")], &[("s", vec!["n"])], "s");
    let recursive = build_pt(
        "cx3b",
        &[("n", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["n"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    assert!(recursive.state_count > flat.state_count);
}

#[test]
fn v6_complex_more_nonterminals_in_goto() {
    let small = build_pt("cx4a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let big = build_pt(
        "cx4b",
        &[("a", "a")],
        &[
            ("atom", vec!["a"]),
            ("inner", vec!["atom"]),
            ("s", vec!["inner"]),
        ],
        "s",
    );
    assert!(big.nonterminal_to_index.len() > small.nonterminal_to_index.len());
}

#[test]
fn v6_complex_four_alternatives_more_shifts() {
    let two = build_pt(
        "cx5a",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let four = build_pt(
        "cx5b",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
        ],
        "s",
    );
    let shifts_two = count_action_kind(&two, |a| matches!(a, Action::Shift(_)));
    let shifts_four = count_action_kind(&four, |a| matches!(a, Action::Shift(_)));
    assert!(
        shifts_four > shifts_two,
        "four alts should have more shifts: {shifts_four} vs {shifts_two}"
    );
}

#[test]
fn v6_complex_deeper_chain_more_nonterminals() {
    let shallow = build_pt(
        "cx6a",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    let deep = build_pt(
        "cx6b",
        &[("x", "x")],
        &[
            ("d", vec!["x"]),
            ("c", vec!["d"]),
            ("b", vec!["c"]),
            ("s", vec!["b"]),
        ],
        "s",
    );
    assert!(deep.nonterminal_to_index.len() > shallow.nonterminal_to_index.len());
}

#[test]
fn v6_complex_longer_sequence_more_symbols_in_index() {
    let short = build_pt("cx7a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let long = build_pt(
        "cx7b",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    assert!(long.symbol_to_index.len() > short.symbol_to_index.len());
}

#[test]
fn v6_complex_multi_level_more_rules() {
    let one_level = build_pt("cx8a", &[("x", "x")], &[("s", vec!["x"])], "s");
    let multi = build_pt(
        "cx8b",
        &[("x", "x"), ("y", "y")],
        &[
            ("leaf", vec!["x"]),
            ("branch", vec!["leaf", "y"]),
            ("s", vec!["branch"]),
        ],
        "s",
    );
    assert!(multi.rules.len() > one_level.rules.len());
}

// ===========================================================================
// 8. Determinism: same grammar → same table (8 tests)
// ===========================================================================

#[test]
fn v6_determinism_state_count() {
    let pt1 = build_pt("d1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("d1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn v6_determinism_symbol_count() {
    let pt1 = build_pt("d2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("d2", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
}

#[test]
fn v6_determinism_rules_count() {
    let pt1 = build_pt("d3", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("d3", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.rules.len(), pt2.rules.len());
}

#[test]
fn v6_determinism_eof_symbol() {
    let pt1 = build_pt("d4", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("d4", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.eof_symbol, pt2.eof_symbol);
}

#[test]
fn v6_determinism_initial_state() {
    let pt1 = build_pt("d5", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("d5", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt1.initial_state, pt2.initial_state);
}

#[test]
fn v6_determinism_actions_match() {
    let pt1 = build_pt("d6", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("d6", &[("a", "a")], &[("s", vec!["a"])], "s");
    for s in 0..pt1.state_count {
        for sym in pt1.symbol_to_index.keys() {
            assert_eq!(
                pt1.actions(StateId(s as u16), *sym),
                pt2.actions(StateId(s as u16), *sym),
            );
        }
    }
}

#[test]
fn v6_determinism_goto_match() {
    let pt1 = build_pt("d7", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("d7", &[("a", "a")], &[("s", vec!["a"])], "s");
    for s in 0..pt1.state_count {
        for nt in pt1.nonterminal_to_index.keys() {
            assert_eq!(
                pt1.goto(StateId(s as u16), *nt),
                pt2.goto(StateId(s as u16), *nt),
            );
        }
    }
}

#[test]
fn v6_determinism_complex_grammar() {
    let tokens = &[("n", "[0-9]+"), ("plus", "\\+"), ("star", "\\*")];
    let rules: &[(&str, Vec<&str>)] = &[
        ("factor", vec!["n"]),
        ("term", vec!["factor"]),
        ("term", vec!["term", "star", "factor"]),
        ("expr", vec!["term"]),
        ("expr", vec!["expr", "plus", "term"]),
        ("s", vec!["expr"]),
    ];
    let pt1 = build_pt("d8", tokens, rules, "s");
    let pt2 = build_pt("d8", tokens, rules, "s");
    assert_eq!(pt1.state_count, pt2.state_count);
    assert_eq!(pt1.rules.len(), pt2.rules.len());
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
    for s in 0..pt1.state_count {
        for sym in pt1.symbol_to_index.keys() {
            assert_eq!(
                pt1.actions(StateId(s as u16), *sym),
                pt2.actions(StateId(s as u16), *sym),
            );
        }
        for nt in pt1.nonterminal_to_index.keys() {
            assert_eq!(
                pt1.goto(StateId(s as u16), *nt),
                pt2.goto(StateId(s as u16), *nt),
            );
        }
    }
}
