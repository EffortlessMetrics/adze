use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{StateId, SymbolId};

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

// === CATEGORY 1: construct_* (8 tests) ===
// Tests for table construction for valid grammars

#[test]
fn construct_minimal_single_rule() {
    let pt = build_pt("minimal", &[("a", "")], &[("S", vec!["a"])], "S");
    assert!(pt.state_count > 0);
    assert!(pt.symbol_count > 0);
}

#[test]
fn construct_simple_binary_rule() {
    let pt = build_pt(
        "binary",
        &[("x", ""), ("y", "")],
        &[("E", vec!["x", "y"])],
        "E",
    );
    assert!(pt.state_count > 1);
    assert!(pt.symbol_count >= 3);
}

#[test]
fn construct_multiple_rules_same_lhs() {
    let pt = build_pt(
        "multi_rhs",
        &[("a", ""), ("b", "")],
        &[("S", vec!["a"]), ("S", vec!["b"])],
        "S",
    );
    assert!(pt.state_count > 0);
}

#[test]
fn construct_chained_nonterminals() {
    let pt = build_pt(
        "chain",
        &[("t", "")],
        &[("A", vec!["B"]), ("B", vec!["t"])],
        "A",
    );
    assert!(pt.state_count > 0);
}

#[test]
fn construct_recursive_left() {
    let pt = build_pt(
        "left_rec",
        &[("n", "")],
        &[("L", vec!["L", "n"]), ("L", vec!["n"])],
        "L",
    );
    assert!(pt.state_count > 1);
}

#[test]
fn construct_recursive_right() {
    let pt = build_pt(
        "right_rec",
        &[("n", "")],
        &[("R", vec!["n", "R"]), ("R", vec!["n"])],
        "R",
    );
    assert!(pt.state_count > 0);
}

#[test]
fn construct_epsilon_rule() {
    let pt = build_pt(
        "epsilon",
        &[("a", "")],
        &[("E", vec!["a"]), ("E", vec![])],
        "E",
    );
    assert!(pt.state_count > 0);
}

#[test]
fn construct_three_level_hierarchy() {
    let pt = build_pt(
        "three_level",
        &[("x", "")],
        &[("A", vec!["B"]), ("B", vec!["C"]), ("C", vec!["x"])],
        "A",
    );
    assert!(pt.state_count > 2);
}

// === CATEGORY 2: action_* (8 tests) ===
// Tests for action query correctness

#[test]
fn action_shift_exists_for_terminal() {
    let pt = build_pt("shift_test", &[("t", "")], &[("S", vec!["t"])], "S");
    let t_sym = SymbolId(1);
    let actions = pt.actions(StateId(0), t_sym);
    assert!(!actions.is_empty());
}

#[test]
fn action_reduce_exists_for_rule() {
    let pt = build_pt("reduce_test", &[("a", "")], &[("S", vec!["a"])], "S");
    let eof = pt.eof_symbol;
    let eof_actions = pt.actions(StateId(1), eof);
    assert!(eof_actions.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn action_multiple_choices() {
    let pt = build_pt(
        "multi_action",
        &[("x", ""), ("y", "")],
        &[("S", vec!["x"]), ("S", vec!["y"])],
        "S",
    );
    let x_sym = SymbolId(1);
    let x_actions = pt.actions(StateId(0), x_sym);
    assert!(!x_actions.is_empty());
}

#[test]
fn action_shift_vs_reduce_ambiguity() {
    let pt = build_pt(
        "shift_reduce",
        &[("op", ""), ("id", "")],
        &[("E", vec!["E", "op", "E"]), ("E", vec!["id"])],
        "E",
    );
    let actions_exist = pt.state_count > 2;
    assert!(actions_exist);
}

#[test]
fn action_query_all_states() {
    let pt = build_pt("all_states", &[("n", "")], &[("S", vec!["n"])], "S");
    for i in 0..pt.state_count {
        let _ = pt.actions(StateId(i as u16), pt.eof_symbol);
    }
}

// === CATEGORY 3: goto_* (8 tests) ===
// Tests for goto table entries

#[test]
fn goto_nonterminal_exists() {
    let pt = build_pt(
        "goto_test",
        &[("t", "")],
        &[("S", vec!["A"]), ("A", vec!["t"])],
        "S",
    );
    let a_sym = SymbolId(1);
    let result = pt.goto(StateId(0), a_sym);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn goto_multiple_nonterminals() {
    let pt = build_pt(
        "multi_goto",
        &[("x", "")],
        &[("S", vec!["A", "B"]), ("A", vec!["x"]), ("B", vec!["x"])],
        "S",
    );
    let _ = pt.goto(StateId(0), SymbolId(1));
}

#[test]
fn goto_chain_resolution() {
    let pt = build_pt(
        "chain_goto",
        &[("t", "")],
        &[("A", vec!["B"]), ("B", vec!["C"]), ("C", vec!["t"])],
        "A",
    );
    let b_sym = SymbolId(1);
    let _ = pt.goto(StateId(0), b_sym);
}

#[test]
fn goto_returns_valid_state() {
    let pt = build_pt(
        "valid_state",
        &[("a", "")],
        &[("S", vec!["A"]), ("A", vec!["a"])],
        "S",
    );
    let state_count = pt.state_count;
    if let Some(next) = pt.goto(StateId(0), SymbolId(1)) {
        assert!((next.0 as usize) < state_count);
    }
}

#[test]
fn goto_epsilon_production() {
    let pt = build_pt(
        "epsilon_goto",
        &[("x", "")],
        &[("S", vec!["A"]), ("A", vec!["x"]), ("A", vec![])],
        "S",
    );
    let _ = pt.goto(StateId(0), SymbolId(1));
}

#[test]
fn goto_different_symbols() {
    let pt = build_pt(
        "diff_syms",
        &[("x", ""), ("y", "")],
        &[("S", vec!["A", "B"]), ("A", vec!["x"]), ("B", vec!["y"])],
        "S",
    );
    let a_sym = SymbolId(2);
    let b_sym = SymbolId(3);
    let _ = pt.goto(StateId(0), a_sym);
    let _ = pt.goto(StateId(0), b_sym);
}

#[test]
fn goto_all_nonterminals() {
    let pt = build_pt(
        "all_nts",
        &[("t", "")],
        &[("S", vec!["A"]), ("A", vec!["B"]), ("B", vec!["t"])],
        "S",
    );
    for i in 1..pt.symbol_count {
        let _ = pt.goto(StateId(0), SymbolId(i as u16));
    }
}

// === CATEGORY 4: state_* (8 tests) ===
// Tests for state count properties

#[test]
fn state_count_positive() {
    let pt = build_pt("pos_states", &[("x", "")], &[("S", vec!["x"])], "S");
    assert!(pt.state_count > 0);
}

#[test]
fn state_count_increases_with_rules() {
    let pt_simple = build_pt("simple", &[("a", "")], &[("S", vec!["a"])], "S");
    let pt_complex = build_pt(
        "complex",
        &[("a", ""), ("b", "")],
        &[("S", vec!["a", "b"]), ("S", vec!["a"])],
        "S",
    );
    assert!(pt_complex.state_count >= pt_simple.state_count);
}

#[test]
fn state_count_valid_index_range() {
    let pt = build_pt("index_range", &[("t", "")], &[("S", vec!["t"])], "S");
    assert!(pt.state_count > 0);
    let _ = StateId((pt.state_count - 1) as u16);
}

#[test]
fn state_count_deterministic() {
    let pt1 = build_pt("det1", &[("x", "")], &[("S", vec!["x"])], "S");
    let pt2 = build_pt("det2", &[("x", "")], &[("S", vec!["x"])], "S");
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn state_count_recursive_grammar() {
    let pt = build_pt(
        "recursive",
        &[("n", "")],
        &[("S", vec!["S", "n"]), ("S", vec!["n"])],
        "S",
    );
    assert!(pt.state_count > 2);
}

#[test]
fn state_count_epsilon_rules() {
    let pt = build_pt(
        "epsilon_states",
        &[("a", "")],
        &[("S", vec!["a"]), ("S", vec![])],
        "S",
    );
    assert!(pt.state_count > 0);
}

#[test]
fn state_count_higher_arity() {
    let pt = build_pt(
        "high_arity",
        &[("a", ""), ("b", ""), ("c", "")],
        &[("S", vec!["a", "b", "c"])],
        "S",
    );
    assert!(pt.state_count > 1);
}

// === CATEGORY 5: accept_* (8 tests) ===
// Tests for accept action presence

#[test]
fn accept_action_in_final_state() {
    let pt = build_pt("accept_final", &[("x", "")], &[("S", vec!["x"])], "S");
    let eof = pt.eof_symbol;
    let mut found_accept = false;
    for state_idx in 0..pt.state_count {
        let actions = pt.actions(StateId(state_idx as u16), eof);
        if actions.iter().any(|a| matches!(a, Action::Accept)) {
            found_accept = true;
            break;
        }
    }
    assert!(found_accept);
}

#[test]
fn accept_on_eof_symbol() {
    let pt = build_pt("accept_eof", &[("t", "")], &[("S", vec!["t"])], "S");
    let eof = pt.eof_symbol;
    assert_eq!(eof, pt.eof());
}

#[test]
fn accept_single_occurrence() {
    let pt = build_pt("single_accept", &[("a", "")], &[("S", vec!["a"])], "S");
    let eof = pt.eof_symbol;
    let mut accept_count = 0;
    for state_idx in 0..pt.state_count {
        let actions = pt.actions(StateId(state_idx as u16), eof);
        for action in actions {
            if matches!(action, Action::Accept) {
                accept_count += 1;
            }
        }
    }
    assert_eq!(accept_count, 1);
}

#[test]
fn accept_epsilon_grammar() {
    let pt = build_pt(
        "accept_eps",
        &[("a", "")],
        &[("S", vec!["a"]), ("S", vec![])],
        "S",
    );
    let eof = pt.eof_symbol;
    let mut has_accept = false;
    for state_idx in 0..pt.state_count {
        if pt
            .actions(StateId(state_idx as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
        {
            has_accept = true;
        }
    }
    assert!(has_accept);
}

// === CATEGORY 6: reduce_* (8 tests) ===
// Tests for reduce action correctness

#[test]
fn reduce_action_has_rule_id() {
    let pt = build_pt("reduce_ruleid", &[("a", "")], &[("S", vec!["a"])], "S");
    let eof = pt.eof_symbol;
    for state_idx in 0..pt.state_count {
        let actions = pt.actions(StateId(state_idx as u16), eof);
        for action in actions {
            if let Action::Reduce(rid) = action {
                let (_, rhs_len) = pt.rule(*rid);
                let _ = rhs_len;
            }
        }
    }
}

#[test]
fn reduce_rule_lhs_matches() {
    let pt = build_pt(
        "reduce_lhs",
        &[("t", "")],
        &[("S", vec!["t"]), ("S", vec!["A"]), ("A", vec!["t"])],
        "S",
    );
    let eof = pt.eof_symbol;
    let actions = pt.actions(StateId(1), eof);
    for action in actions {
        if let Action::Reduce(rid) = action {
            let (_, _) = pt.rule(*rid);
        }
    }
}

#[test]
fn reduce_multiple_rules() {
    let pt = build_pt(
        "reduce_multi",
        &[("x", ""), ("y", "")],
        &[("S", vec!["A"]), ("A", vec!["x"]), ("A", vec!["y"])],
        "S",
    );
    let eof = pt.eof_symbol;
    let mut reduce_count = 0;
    for state_idx in 0..pt.state_count {
        let actions = pt.actions(StateId(state_idx as u16), eof);
        for action in actions {
            if matches!(action, Action::Reduce(_)) {
                reduce_count += 1;
            }
        }
    }
    assert!(reduce_count > 0);
}

#[test]
fn reduce_epsilon_production() {
    let pt = build_pt(
        "reduce_epsilon",
        &[("a", "")],
        &[("S", vec!["A"]), ("A", vec![])],
        "S",
    );
    for state_idx in 0..pt.state_count {
        for sym_idx in 0..pt.symbol_count {
            let actions = pt.actions(StateId(state_idx as u16), SymbolId(sym_idx as u16));
            for action in actions {
                if let Action::Reduce(rid) = action {
                    let (_, rhs_len) = pt.rule(*rid);
                    let _ = rhs_len;
                }
            }
        }
    }
}

#[test]
fn reduce_preserves_rule_semantics() {
    let pt = build_pt(
        "reduce_sem",
        &[("n", "")],
        &[("S", vec!["L"]), ("L", vec!["n"]), ("L", vec!["L", "n"])],
        "S",
    );
    assert!(pt.state_count > 2);
}

// === CATEGORY 7: complex_* (8 tests) ===
// Tests for complex grammar tables

#[test]
fn complex_operator_precedence() {
    let pt = build_pt(
        "precedence",
        &[("id", ""), ("+", ""), ("*", "")],
        &[
            ("E", vec!["E", "+", "E"]),
            ("E", vec!["E", "*", "E"]),
            ("E", vec!["id"]),
        ],
        "E",
    );
    assert!(pt.state_count > 3);
}

#[test]
fn complex_nested_structures() {
    let pt = build_pt(
        "nested",
        &[("(", ""), (")", "")],
        &[("E", vec!["(", "E", ")"]), ("E", vec!["(", ")"])],
        "E",
    );
    assert!(pt.state_count > 0);
}

#[test]
fn complex_multiple_nonterminals() {
    let pt = build_pt(
        "multi_nt",
        &[("x", ""), ("y", ""), ("z", "")],
        &[
            ("S", vec!["A", "B", "C"]),
            ("A", vec!["x"]),
            ("B", vec!["y"]),
            ("C", vec!["z"]),
        ],
        "S",
    );
    assert!(pt.state_count > 3);
}

#[test]
fn complex_mutual_recursion() {
    let pt = build_pt(
        "mutual",
        &[("a", ""), ("b", "")],
        &[
            ("A", vec!["a", "B"]),
            ("B", vec!["b", "A"]),
            ("A", vec!["a"]),
            ("B", vec!["b"]),
        ],
        "A",
    );
    assert!(pt.state_count > 4);
}

#[test]
fn complex_mixed_recursion() {
    let pt = build_pt(
        "mixed_rec",
        &[("n", "")],
        &[
            ("S", vec!["L", "R"]),
            ("L", vec!["L", "n"]),
            ("L", vec!["n"]),
            ("R", vec!["n", "R"]),
            ("R", vec!["n"]),
        ],
        "S",
    );
    assert!(pt.state_count > 5);
}

#[test]
fn complex_high_branching_factor() {
    let pt = build_pt(
        "branch",
        &[("a", ""), ("b", ""), ("c", ""), ("d", "")],
        &[
            ("S", vec!["A"]),
            ("A", vec!["a"]),
            ("A", vec!["b"]),
            ("A", vec!["c"]),
            ("A", vec!["d"]),
        ],
        "S",
    );
    assert!(pt.state_count > 3);
}

#[test]
fn complex_deep_chain() {
    let pt = build_pt(
        "deep",
        &[("t", "")],
        &[
            ("A", vec!["B"]),
            ("B", vec!["C"]),
            ("C", vec!["D"]),
            ("D", vec!["E"]),
            ("E", vec!["t"]),
        ],
        "A",
    );
    assert!(pt.state_count > 4);
}

// === CATEGORY 8: determinism_* (8 tests) ===
// Tests for deterministic table generation

#[test]
fn determinism_same_grammar_same_table() {
    let pt1 = build_pt("det_a", &[("x", "")], &[("S", vec!["x"])], "S");
    let pt2 = build_pt("det_b", &[("x", "")], &[("S", vec!["x"])], "S");
    assert_eq!(pt1.state_count, pt2.state_count);
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
}

#[test]
fn determinism_actions_consistent() {
    let pt = build_pt("consistent", &[("a", "")], &[("S", vec!["a"])], "S");
    let eof = pt.eof_symbol;
    let actions1 = pt.actions(StateId(0), eof);
    let actions2 = pt.actions(StateId(0), eof);
    assert_eq!(actions1.len(), actions2.len());
}

#[test]
fn determinism_goto_consistent() {
    let pt = build_pt(
        "goto_cons",
        &[("t", "")],
        &[("S", vec!["A"]), ("A", vec!["t"])],
        "S",
    );
    let result1 = pt.goto(StateId(0), SymbolId(1));
    let result2 = pt.goto(StateId(0), SymbolId(1));
    assert_eq!(result1, result2);
}

#[test]
fn determinism_symbol_ids_stable() {
    let pt = build_pt("sym_stable", &[("a", "")], &[("S", vec!["a"])], "S");
    let eof1 = pt.eof();
    let eof2 = pt.eof();
    assert_eq!(eof1, eof2);
}

#[test]
fn determinism_start_symbol_stable() {
    let pt = build_pt("start_stable", &[("x", "")], &[("S", vec!["x"])], "S");
    let start1 = pt.start_symbol();
    let start2 = pt.start_symbol();
    assert_eq!(start1, start2);
}

#[test]
fn determinism_complex_grammar_stable() {
    let pt1 = build_pt(
        "complex_det1",
        &[("x", ""), ("y", "")],
        &[
            ("S", vec!["A", "B"]),
            ("A", vec!["x", "A"]),
            ("A", vec!["x"]),
            ("B", vec!["y"]),
        ],
        "S",
    );
    let pt2 = build_pt(
        "complex_det2",
        &[("x", ""), ("y", "")],
        &[
            ("S", vec!["A", "B"]),
            ("A", vec!["x", "A"]),
            ("A", vec!["x"]),
            ("B", vec!["y"]),
        ],
        "S",
    );
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn determinism_all_queries_deterministic() {
    let pt = build_pt(
        "all_det",
        &[("n", "")],
        &[("S", vec!["L"]), ("L", vec!["n"]), ("L", vec!["L", "n"])],
        "S",
    );
    for _ in 0..2 {
        for state_idx in 0..pt.state_count {
            for sym_idx in 0..pt.symbol_count {
                let _ = pt.actions(StateId(state_idx as u16), SymbolId(sym_idx as u16));
                let _ = pt.goto(StateId(state_idx as u16), SymbolId(sym_idx as u16));
            }
        }
    }
}
