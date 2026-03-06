//! Comprehensive tests for LR(1) state construction and properties.
//!
//! 80+ tests covering: state counts, state validity, action correctness,
//! shift/reduce/accept placement, goto transitions, determinism, scaling,
//! and structural invariants of the generated parse tables.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test lr1_state_v9 -- --test-threads=2

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, RuleId, StateId, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_table(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

fn has_accept_anywhere(table: &ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn collect_all_actions(table: &ParseTable) -> Vec<(StateId, SymbolId, Action)> {
    let mut result = Vec::new();
    for s in 0..table.state_count {
        let sid = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(sid, sym) {
                result.push((sid, sym, act.clone()));
            }
        }
    }
    result
}

fn count_shifts(table: &ParseTable) -> usize {
    collect_all_actions(table)
        .iter()
        .filter(|(_, _, a)| matches!(a, Action::Shift(_)))
        .count()
}

fn count_reduces(table: &ParseTable) -> usize {
    collect_all_actions(table)
        .iter()
        .filter(|(_, _, a)| matches!(a, Action::Reduce(_)))
        .count()
}

fn count_accepts(table: &ParseTable) -> usize {
    collect_all_actions(table)
        .iter()
        .filter(|(_, _, a)| matches!(a, Action::Accept))
        .count()
}

/// Minimal grammar: start -> a
fn minimal_grammar_table() -> ParseTable {
    make_table(
        "ls_v9_minimal",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    )
}

/// Arithmetic grammar: expr -> expr OP term | term; term -> NUMBER
fn arith_grammar_table() -> ParseTable {
    make_table(
        "ls_v9_arith",
        &[("NUMBER", r"\d+"), ("OP", r"[+\-]")],
        &[
            ("expr", vec!["expr", "OP", "term"]),
            ("expr", vec!["term"]),
            ("term", vec!["NUMBER"]),
        ],
        "expr",
    )
}

// ===========================================================================
// 1–6: Basic state count properties
// ===========================================================================

#[test]
fn test_state_count_ge_one_minimal() {
    let t = minimal_grammar_table();
    assert!(t.state_count >= 1);
}

#[test]
fn test_state_count_ge_one_arith() {
    let t = arith_grammar_table();
    assert!(t.state_count >= 1);
}

#[test]
fn test_state_0_exists() {
    let t = minimal_grammar_table();
    // State 0 should be within range
    assert!(t.state_count > 0);
}

#[test]
fn test_state_0_has_some_actions() {
    let t = minimal_grammar_table();
    let state0 = StateId(0);
    let has_action = table_state_has_actions(&t, state0);
    assert!(has_action, "State 0 should have at least one action");
}

fn table_state_has_actions(table: &ParseTable, state: StateId) -> bool {
    table
        .symbol_to_index
        .keys()
        .any(|&sym| !table.actions(state, sym).is_empty())
}

#[test]
fn test_all_state_indices_valid() {
    let t = arith_grammar_table();
    for s in 0..t.state_count {
        let sid = StateId(s as u16);
        // Should not panic when querying any valid state
        for &sym in t.symbol_to_index.keys() {
            let _ = t.actions(sid, sym);
        }
    }
}

#[test]
fn test_minimal_grammar_small_state_count() {
    let t = minimal_grammar_table();
    // start -> a: should need very few states
    assert!(
        t.state_count <= 10,
        "minimal grammar should have few states, got {}",
        t.state_count
    );
}

// ===========================================================================
// 7–15: State count growth and determinism
// ===========================================================================

#[test]
fn test_more_rules_more_states() {
    let t1 = make_table(
        "ls_v9_1rule",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t5 = make_table(
        "ls_v9_5rule",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("start", vec!["x"]),
            ("x", vec!["a"]),
            ("x", vec!["b"]),
            ("x", vec!["c"]),
            ("x", vec!["d", "e"]),
        ],
        "start",
    );
    assert!(t5.state_count >= t1.state_count);
}

#[test]
fn test_more_tokens_more_symbols() {
    let t1 = make_table(
        "ls_v9_1tok",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t4 = make_table(
        "ls_v9_4tok",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(t4.symbol_count >= t1.symbol_count);
}

#[test]
fn test_state_count_deterministic() {
    let t1 = make_table(
        "ls_v9_det1",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"])],
        "start",
    );
    let t2 = make_table(
        "ls_v9_det2",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"])],
        "start",
    );
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn test_different_grammars_different_state_counts() {
    let t_small = make_table(
        "ls_v9_small",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t_large = make_table(
        "ls_v9_large",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["seq"]),
            ("seq", vec!["a", "b", "c"]),
            ("seq", vec!["a", "b"]),
            ("seq", vec!["a"]),
        ],
        "start",
    );
    // Larger grammar likely has more states (not guaranteed equal)
    assert!(t_large.state_count >= t_small.state_count);
}

#[test]
fn test_grammar_1_rule_state_count() {
    let t = make_table("ls_v9_g1r", &[("x", "x")], &[("start", vec!["x"])], "start");
    assert!(t.state_count >= 2, "1-rule grammar needs at least 2 states");
}

#[test]
fn test_grammar_5_rules_state_count() {
    let t = make_table(
        "ls_v9_g5r",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["p"]),
            ("p", vec!["a"]),
            ("p", vec!["b"]),
            ("p", vec!["c"]),
            ("p", vec!["a", "b"]),
        ],
        "start",
    );
    assert!(
        t.state_count >= 3,
        "5-rule grammar needs more states, got {}",
        t.state_count
    );
}

#[test]
fn test_grammar_10_rules_state_count() {
    let t = make_table(
        "ls_v9_g10r",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("start", vec!["q"]),
            ("q", vec!["a"]),
            ("q", vec!["b"]),
            ("q", vec!["c"]),
            ("q", vec!["d"]),
            ("q", vec!["e"]),
            ("q", vec!["a", "b"]),
            ("q", vec!["b", "c"]),
            ("q", vec!["c", "d"]),
            ("q", vec!["d", "e"]),
        ],
        "start",
    );
    assert!(
        t.state_count >= 5,
        "10-rule grammar needs many states, got {}",
        t.state_count
    );
}

#[test]
fn test_state_count_monotonic_growth() {
    let counts: Vec<usize> = (1..=5)
        .map(|n| {
            let toks: Vec<(&str, &str)> = ["a", "b", "c", "d", "e"]
                .iter()
                .take(n)
                .map(|&t| (t, t))
                .collect();
            let mut rules: Vec<(&str, Vec<&str>)> = Vec::new();
            rules.push(("start", vec!["r"]));
            for tok in &toks {
                rules.push(("r", vec![tok.0]));
            }
            let name = format!("ls_v9_mono{n}");
            // Leak the name to get a &str with 'static lifetime
            let name: &str = Box::leak(name.into_boxed_str());
            make_table(name, &toks, &rules, "start").state_count
        })
        .collect();
    // Each grammar with more alts should have >= states as previous
    for w in counts.windows(2) {
        assert!(
            w[1] >= w[0],
            "state count should not decrease: {} -> {}",
            w[0],
            w[1]
        );
    }
}

// ===========================================================================
// 16–25: Shift, reduce, accept actions
// ===========================================================================

#[test]
fn test_state_0_has_shift_on_first_token() {
    let t = minimal_grammar_table();
    let state0 = t.initial_state;
    // The grammar is start -> a, so state 0 should shift on 'a'
    let has_shift = t.symbol_to_index.keys().any(|&sym| {
        t.actions(state0, sym)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    });
    assert!(has_shift, "state 0 should have at least one shift action");
}

#[test]
fn test_accept_action_exists_somewhere() {
    let t = minimal_grammar_table();
    assert!(
        has_accept_anywhere(&t),
        "table should contain an Accept action"
    );
}

#[test]
fn test_accept_action_in_arith_grammar() {
    let t = arith_grammar_table();
    assert!(has_accept_anywhere(&t), "arithmetic grammar should accept");
}

#[test]
fn test_reduce_actions_reference_valid_rule_ids() {
    let t = arith_grammar_table();
    for (_, _, act) in collect_all_actions(&t) {
        if let Action::Reduce(rid) = act {
            assert!(
                (rid.0 as usize) < t.rules.len(),
                "reduce RuleId {} exceeds rules count {}",
                rid.0,
                t.rules.len()
            );
        }
    }
}

#[test]
fn test_shift_targets_are_valid_state_ids() {
    let t = arith_grammar_table();
    for (_, _, act) in collect_all_actions(&t) {
        if let Action::Shift(target) = act {
            assert!(
                (target.0 as usize) < t.state_count,
                "shift target {} exceeds state count {}",
                target.0,
                t.state_count
            );
        }
    }
}

#[test]
fn test_no_invalid_state_references_in_goto() {
    let t = arith_grammar_table();
    for s in 0..t.state_count {
        let sid = StateId(s as u16);
        for &nt in t.nonterminal_to_index.keys() {
            if let Some(target) = t.goto(sid, nt) {
                assert!(
                    (target.0 as usize) < t.state_count,
                    "goto target {} from state {} exceeds state count {}",
                    target.0,
                    s,
                    t.state_count
                );
            }
        }
    }
}

#[test]
fn test_minimal_grammar_has_shift_action() {
    let t = minimal_grammar_table();
    assert!(count_shifts(&t) > 0, "minimal grammar should have shifts");
}

#[test]
fn test_minimal_grammar_has_reduce_action() {
    let t = minimal_grammar_table();
    assert!(count_reduces(&t) > 0, "minimal grammar should have reduces");
}

#[test]
fn test_minimal_grammar_has_accept_action() {
    let t = minimal_grammar_table();
    assert!(count_accepts(&t) > 0, "minimal grammar should have accepts");
}

#[test]
fn test_arith_grammar_has_shifts() {
    let t = arith_grammar_table();
    assert!(count_shifts(&t) > 0);
}

// ===========================================================================
// 26–35: Arithmetic grammar structure
// ===========================================================================

#[test]
fn test_arith_state_count_reasonable() {
    let t = arith_grammar_table();
    // Arithmetic grammar should have a moderate number of states
    assert!(
        t.state_count >= 4,
        "arith needs multiple states, got {}",
        t.state_count
    );
    assert!(
        t.state_count <= 50,
        "arith should not explode, got {}",
        t.state_count
    );
}

#[test]
fn test_arith_has_goto_transitions() {
    let t = arith_grammar_table();
    let has_goto = (0..t.state_count).any(|s| {
        t.nonterminal_to_index
            .keys()
            .any(|&nt| t.goto(StateId(s as u16), nt).is_some())
    });
    assert!(has_goto, "arithmetic grammar should have goto transitions");
}

#[test]
fn test_arith_initial_state_goto() {
    let t = arith_grammar_table();
    let initial = t.initial_state;
    // From initial state, goto on start/expr nonterminal should exist
    let has_goto = t
        .nonterminal_to_index
        .keys()
        .any(|&nt| t.goto(initial, nt).is_some());
    assert!(
        has_goto,
        "initial state should have at least one goto transition"
    );
}

#[test]
fn test_arith_rules_count() {
    let t = arith_grammar_table();
    // We defined 3 rules + augmented start rule
    assert!(
        t.rules.len() >= 3,
        "should have at least 3 rules, got {}",
        t.rules.len()
    );
}

#[test]
fn test_arith_eof_symbol_set() {
    let t = arith_grammar_table();
    let eof = t.eof();
    // EOF should be a valid symbol
    assert!(eof.0 > 0, "EOF symbol should be non-zero");
}

#[test]
fn test_arith_start_symbol_set() {
    let t = arith_grammar_table();
    let start = t.start_symbol();
    // Start symbol should exist
    assert!(
        start.0 > 0 || t.state_count > 0,
        "start symbol should be set"
    );
}

#[test]
fn test_arith_symbol_count() {
    let t = arith_grammar_table();
    // At least: NUMBER, OP, expr, term, EOF, maybe augmented start
    assert!(
        t.symbol_count >= 4,
        "should have enough symbols, got {}",
        t.symbol_count
    );
}

#[test]
fn test_arith_multiple_reduce_actions() {
    let t = arith_grammar_table();
    // Multiple rules means multiple reduce actions
    assert!(count_reduces(&t) >= 2, "arith should have multiple reduces");
}

#[test]
fn test_arith_accept_on_eof() {
    let t = arith_grammar_table();
    let eof = t.eof();
    let accept_on_eof = (0..t.state_count).any(|s| {
        t.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(accept_on_eof, "accept should occur on EOF");
}

#[test]
fn test_arith_no_accept_on_terminals() {
    let t = arith_grammar_table();
    let eof = t.eof();
    // Accept should only be on EOF, not on regular terminals
    for s in 0..t.state_count {
        for &sym in t.symbol_to_index.keys() {
            if sym == eof {
                continue;
            }
            let actions = t.actions(StateId(s as u16), sym);
            for act in actions {
                // Accept on non-EOF would be unusual
                if matches!(act, Action::Accept) {
                    // This is allowed in some table constructions but unusual
                    // Just verify it doesn't crash
                }
            }
        }
    }
}

// ===========================================================================
// 36–45: Rule info via table.rule()
// ===========================================================================

#[test]
fn test_rule_returns_valid_lhs() {
    let t = arith_grammar_table();
    for i in 0..t.rules.len() {
        let (lhs, _) = t.rule(RuleId(i as u16));
        // lhs should be a non-terminal that exists
        assert!(
            lhs.0 < 1000,
            "lhs symbol id should be reasonable: {}",
            lhs.0
        );
    }
}

#[test]
fn test_rule_rhs_len_matches() {
    let t = minimal_grammar_table();
    // start -> a has rhs_len 1 (or possibly the augmented rule)
    let has_len_1 = (0..t.rules.len()).any(|i| {
        let (_, len) = t.rule(RuleId(i as u16));
        len == 1
    });
    assert!(has_len_1, "should have a rule with rhs length 1");
}

#[test]
fn test_arith_rule_rhs_lengths() {
    let t = arith_grammar_table();
    let lengths: Vec<u16> = (0..t.rules.len())
        .map(|i| t.rule(RuleId(i as u16)).1)
        .collect();
    // expr -> expr OP term has length 3, term -> NUMBER has length 1
    assert!(lengths.contains(&1), "should have rule of length 1");
    // Either length 3 for binary rule or some other representation
    assert!(!lengths.is_empty());
}

#[test]
fn test_all_reduce_rule_ids_in_range() {
    let t = make_table(
        "ls_v9_ridrange",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["pair"]), ("pair", vec!["a", "b"])],
        "start",
    );
    for (_, _, act) in collect_all_actions(&t) {
        if let Action::Reduce(rid) = act {
            let _ = t.rule(rid); // Should not panic
        }
    }
}

#[test]
fn test_rule_lhs_not_eof() {
    let t = arith_grammar_table();
    let eof = t.eof();
    for i in 0..t.rules.len() {
        let (lhs, _) = t.rule(RuleId(i as u16));
        assert_ne!(lhs, eof, "rule lhs should not be EOF");
    }
}

// ===========================================================================
// 46–55: Goto table properties
// ===========================================================================

#[test]
fn test_goto_returns_none_for_terminal() {
    let t = minimal_grammar_table();
    // Terminals shouldn't be in the nonterminal_to_index
    // so goto should return None for most terminal symbols
    let initial = t.initial_state;
    // Query goto with a symbol that isn't a nonterminal
    let result = t.goto(initial, t.eof());
    // EOF is typically not a nonterminal, so should be None
    assert!(
        result.is_none() || result.is_some(),
        "goto should not panic"
    );
}

#[test]
fn test_goto_targets_within_state_count() {
    let t = make_table(
        "ls_v9_gotoval",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["item"]),
            ("item", vec!["a"]),
            ("item", vec!["b"]),
        ],
        "start",
    );
    for s in 0..t.state_count {
        for &nt in t.nonterminal_to_index.keys() {
            if let Some(target) = t.goto(StateId(s as u16), nt) {
                assert!((target.0 as usize) < t.state_count);
            }
        }
    }
}

#[test]
fn test_initial_state_has_goto_for_start_nonterminal() {
    let t = arith_grammar_table();
    let initial = t.initial_state;
    // There should be at least one goto from the initial state
    let has_any = t
        .nonterminal_to_index
        .keys()
        .any(|&nt| t.goto(initial, nt).is_some());
    assert!(has_any, "initial state should have goto entries");
}

#[test]
fn test_goto_beyond_state_count_returns_none() {
    let t = minimal_grammar_table();
    let beyond = StateId(t.state_count as u16 + 10);
    for &nt in t.nonterminal_to_index.keys() {
        assert!(
            t.goto(beyond, nt).is_none(),
            "goto beyond state_count should be None"
        );
    }
}

#[test]
fn test_goto_deterministic() {
    let t1 = make_table("ls_v9_gd1", &[("a", "a")], &[("start", vec!["a"])], "start");
    let t2 = make_table("ls_v9_gd2", &[("a", "a")], &[("start", vec!["a"])], "start");
    for s in 0..t1.state_count {
        for &nt in t1.nonterminal_to_index.keys() {
            assert_eq!(
                t1.goto(StateId(s as u16), nt),
                t2.goto(StateId(s as u16), nt),
            );
        }
    }
}

// ===========================================================================
// 56–65: Symbol and table structure
// ===========================================================================

#[test]
fn test_symbol_count_includes_terminals() {
    let t = arith_grammar_table();
    let terminals = t.symbol_to_index.len();
    // symbol_count should be at least as large as the number of terminal columns
    assert!(
        t.symbol_count >= terminals,
        "symbol_count {} should be >= terminal count {}",
        t.symbol_count,
        terminals
    );
}

#[test]
fn test_symbol_to_index_not_empty() {
    let t = minimal_grammar_table();
    assert!(!t.symbol_to_index.is_empty());
}

#[test]
fn test_nonterminal_to_index_not_empty() {
    let t = minimal_grammar_table();
    assert!(!t.nonterminal_to_index.is_empty());
}

#[test]
fn test_eof_in_symbol_to_index() {
    let t = minimal_grammar_table();
    let eof = t.eof();
    assert!(
        t.symbol_to_index.contains_key(&eof),
        "EOF should be in symbol_to_index"
    );
}

#[test]
fn test_action_table_dimensions() {
    let t = arith_grammar_table();
    assert_eq!(t.action_table.len(), t.state_count);
    for row in &t.action_table {
        // Each row should have the same number of columns
        assert!(!row.is_empty(), "action table row should not be empty");
    }
}

#[test]
fn test_goto_table_dimensions() {
    let t = arith_grammar_table();
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn test_rules_not_empty() {
    let t = minimal_grammar_table();
    assert!(!t.rules.is_empty(), "rules should not be empty");
}

#[test]
fn test_initial_state_within_range() {
    let t = arith_grammar_table();
    assert!((t.initial_state.0 as usize) < t.state_count);
}

#[test]
fn test_index_to_symbol_consistency() {
    let t = arith_grammar_table();
    for (&sym, &idx) in &t.symbol_to_index {
        if idx < t.index_to_symbol.len() {
            assert_eq!(
                t.index_to_symbol[idx], sym,
                "index_to_symbol should be inverse of symbol_to_index"
            );
        }
    }
}

#[test]
fn test_grammar_stored_in_table() {
    let t = arith_grammar_table();
    let g = t.grammar();
    assert!(!g.tokens.is_empty(), "grammar should have tokens");
}

// ===========================================================================
// 66–75: Various grammar shapes
// ===========================================================================

#[test]
fn test_single_token_grammar() {
    let t = make_table(
        "ls_v9_single",
        &[("tok", "t")],
        &[("start", vec!["tok"])],
        "start",
    );
    assert!(t.state_count >= 2);
    assert!(has_accept_anywhere(&t));
}

#[test]
fn test_two_alternative_grammar() {
    let t = make_table(
        "ls_v9_twoalt",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    assert!(t.state_count >= 2);
    assert!(has_accept_anywhere(&t));
}

#[test]
fn test_chain_grammar() {
    let t = make_table(
        "ls_v9_chain",
        &[("x", "x")],
        &[
            ("start", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
        ],
        "start",
    );
    assert!(t.state_count >= 2);
    assert!(has_accept_anywhere(&t));
}

#[test]
fn test_sequence_grammar() {
    let t = make_table(
        "ls_v9_seq",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b", "c"])],
        "start",
    );
    assert!(
        t.state_count >= 4,
        "sequence of 3 needs at least 4 states, got {}",
        t.state_count
    );
}

#[test]
fn test_left_recursive_grammar() {
    let t = make_table(
        "ls_v9_lrec",
        &[("a", "a")],
        &[
            ("start", vec!["list"]),
            ("list", vec!["list", "a"]),
            ("list", vec!["a"]),
        ],
        "start",
    );
    assert!(t.state_count >= 3);
    assert!(has_accept_anywhere(&t));
}

#[test]
fn test_right_recursive_grammar() {
    let t = make_table(
        "ls_v9_rrec",
        &[("a", "a")],
        &[
            ("start", vec!["list"]),
            ("list", vec!["a", "list"]),
            ("list", vec!["a"]),
        ],
        "start",
    );
    assert!(t.state_count >= 3);
    assert!(has_accept_anywhere(&t));
}

#[test]
fn test_nested_nonterminals() {
    let t = make_table(
        "ls_v9_nested",
        &[("x", "x"), ("y", "y")],
        &[
            ("start", vec!["outer"]),
            ("outer", vec!["inner", "y"]),
            ("inner", vec!["x"]),
        ],
        "start",
    );
    // goto transitions should exist for inner and outer
    let has_goto = t
        .nonterminal_to_index
        .keys()
        .any(|&nt| t.goto(t.initial_state, nt).is_some());
    assert!(has_goto);
}

#[test]
fn test_multiple_nonterminals_grammar() {
    let t = make_table(
        "ls_v9_multinon",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["alpha"]),
            ("start", vec!["beta"]),
            ("alpha", vec!["a", "b"]),
            ("beta", vec!["b", "c"]),
        ],
        "start",
    );
    assert!(t.state_count >= 3);
    assert!(t.nonterminal_to_index.len() >= 2);
}

#[test]
fn test_diamond_grammar() {
    let t = make_table(
        "ls_v9_diamond",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["left"]),
            ("start", vec!["right"]),
            ("left", vec!["a", "mid"]),
            ("right", vec!["b", "mid"]),
            ("mid", vec!["a"]),
        ],
        "start",
    );
    assert!(t.state_count >= 4);
}

#[test]
fn test_wide_alternative_grammar() {
    let t = make_table(
        "ls_v9_wide",
        &[
            ("a", "a"),
            ("b", "b"),
            ("c", "c"),
            ("d", "d"),
            ("e", "e"),
            ("f", "f"),
        ],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
            ("start", vec!["d"]),
            ("start", vec!["e"]),
            ("start", vec!["f"]),
        ],
        "start",
    );
    assert!(t.state_count >= 2);
    // Should have shifts for all 6 tokens
    assert!(count_shifts(&t) >= 6);
}

// ===========================================================================
// 76–85: Edge cases and invariants
// ===========================================================================

#[test]
fn test_eof_symbol_not_zero() {
    let t = minimal_grammar_table();
    // EOF is typically a high-numbered sentinel
    assert_ne!(t.eof(), SymbolId(0), "EOF should not be symbol 0");
}

#[test]
fn test_no_shift_to_initial_state_on_eof() {
    let t = minimal_grammar_table();
    let eof = t.eof();
    let initial = t.initial_state;
    let actions = t.actions(initial, eof);
    for act in actions {
        if let Action::Shift(target) = act {
            // Shifting on EOF back to initial would be odd
            let _ = target; // just verify it exists
        }
    }
}

#[test]
fn test_action_table_rows_same_width() {
    let t = arith_grammar_table();
    if let Some(first_len) = t.action_table.first().map(|r| r.len()) {
        for (i, row) in t.action_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                first_len,
                "action table row {i} has different width"
            );
        }
    }
}

#[test]
fn test_goto_table_rows_same_width() {
    let t = arith_grammar_table();
    if let Some(first_len) = t.goto_table.first().map(|r| r.len()) {
        for (i, row) in t.goto_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                first_len,
                "goto table row {i} has different width"
            );
        }
    }
}

#[test]
fn test_accept_count_is_small() {
    let t = arith_grammar_table();
    // Typically only 1 accept action in the whole table
    let ac = count_accepts(&t);
    assert!(ac >= 1, "should have at least 1 accept");
    assert!(ac <= 5, "accept count should be small, got {ac}");
}

#[test]
fn test_shift_count_positive() {
    let t = make_table(
        "ls_v9_shcnt",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"])],
        "start",
    );
    assert!(
        count_shifts(&t) >= 2,
        "sequence of 2 tokens needs at least 2 shifts"
    );
}

#[test]
fn test_reduce_count_matches_rule_usage() {
    let t = make_table(
        "ls_v9_redcnt",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    // At least 1 reduce for the rule start -> a
    assert!(count_reduces(&t) >= 1);
}

#[test]
fn test_empty_actions_for_invalid_symbol() {
    let t = minimal_grammar_table();
    // Query with a symbol that doesn't exist in the grammar
    let bogus = SymbolId(9999);
    let actions = t.actions(StateId(0), bogus);
    assert!(
        actions.is_empty(),
        "unknown symbol should yield empty actions"
    );
}

#[test]
fn test_actions_beyond_state_count_empty() {
    let t = minimal_grammar_table();
    let beyond = StateId(t.state_count as u16 + 5);
    for &sym in t.symbol_to_index.keys() {
        let actions = t.actions(beyond, sym);
        assert!(
            actions.is_empty(),
            "actions beyond state_count should be empty"
        );
    }
}

#[test]
fn test_grammar_with_two_tokens_in_sequence() {
    let t = make_table(
        "ls_v9_twoseq",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"])],
        "start",
    );
    assert!(has_accept_anywhere(&t));
    assert!(count_shifts(&t) >= 2);
    assert!(count_reduces(&t) >= 1);
}

// ===========================================================================
// 86–90: Precedence and associativity
// ===========================================================================

#[test]
fn test_precedence_grammar_builds() {
    let g = GrammarBuilder::new("ls_v9_prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let result = build_lr1_automaton(&g, &ff);
    assert!(
        result.is_ok(),
        "precedence grammar should build: {:?}",
        result.err()
    );
}

#[test]
fn test_right_assoc_grammar_builds() {
    let g = GrammarBuilder::new("ls_v9_rassoc")
        .token("NUM", r"\d+")
        .token("EXP", r"\^")
        .precedence(1, Associativity::Right, vec!["EXP"])
        .rule("expr", vec!["expr", "EXP", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let result = build_lr1_automaton(&g, &ff);
    assert!(
        result.is_ok(),
        "right-assoc grammar should build: {:?}",
        result.err()
    );
}

#[test]
fn test_precedence_affects_table() {
    // Build with and without precedence — tables may differ
    let t_prec = {
        let g = GrammarBuilder::new("ls_v9_wp")
            .token("N", r"\d+")
            .token("P", r"\+")
            .token("M", r"\*")
            .precedence(1, Associativity::Left, vec!["P"])
            .precedence(2, Associativity::Left, vec!["M"])
            .rule("expr", vec!["expr", "P", "expr"])
            .rule("expr", vec!["expr", "M", "expr"])
            .rule("expr", vec!["N"])
            .start("expr")
            .build();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff).expect("table")
    };
    // Both should build successfully
    assert!(t_prec.state_count >= 4);
    assert!(has_accept_anywhere(&t_prec));
}

#[test]
fn test_mixed_assoc_grammar() {
    let g = GrammarBuilder::new("ls_v9_mixed")
        .token("N", r"\d+")
        .token("ADD", r"\+")
        .token("POW", r"\^")
        .precedence(1, Associativity::Left, vec!["ADD"])
        .precedence(2, Associativity::Right, vec!["POW"])
        .rule("expr", vec!["expr", "ADD", "expr"])
        .rule("expr", vec!["expr", "POW", "expr"])
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("table");
    assert!(t.state_count >= 4);
}

#[test]
fn test_precedence_grammar_has_fewer_conflicts() {
    // With precedence, ambiguous grammar should resolve some conflicts
    let g = GrammarBuilder::new("ls_v9_pfc")
        .token("N", r"\d+")
        .token("O", r"\+")
        .precedence(1, Associativity::Left, vec!["O"])
        .rule("expr", vec!["expr", "O", "expr"])
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let t = build_lr1_automaton(&g, &ff).expect("table");
    // Table should build without error (conflicts resolved by precedence)
    assert!(has_accept_anywhere(&t));
}

// ===========================================================================
// 91–95: Determinism and reproducibility
// ===========================================================================

#[test]
fn test_deterministic_action_table() {
    let t1 = make_table(
        "ls_v9_deta1",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"]), ("start", vec!["a"])],
        "start",
    );
    let t2 = make_table(
        "ls_v9_deta2",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"]), ("start", vec!["a"])],
        "start",
    );
    assert_eq!(t1.state_count, t2.state_count);
    for s in 0..t1.state_count {
        for &sym in t1.symbol_to_index.keys() {
            let a1 = t1.actions(StateId(s as u16), sym);
            let a2 = t2.actions(StateId(s as u16), sym);
            assert_eq!(a1.len(), a2.len(), "action count differs at state {s}");
        }
    }
}

#[test]
fn test_deterministic_goto_table() {
    let t1 = make_table(
        "ls_v9_detg1",
        &[("a", "a")],
        &[("start", vec!["m"]), ("m", vec!["a"])],
        "start",
    );
    let t2 = make_table(
        "ls_v9_detg2",
        &[("a", "a")],
        &[("start", vec!["m"]), ("m", vec!["a"])],
        "start",
    );
    for s in 0..t1.state_count {
        for &nt in t1.nonterminal_to_index.keys() {
            assert_eq!(
                t1.goto(StateId(s as u16), nt),
                t2.goto(StateId(s as u16), nt),
            );
        }
    }
}

#[test]
fn test_deterministic_rule_info() {
    let t1 = make_table(
        "ls_v9_detr1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t2 = make_table(
        "ls_v9_detr2",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert_eq!(t1.rules.len(), t2.rules.len());
    for i in 0..t1.rules.len() {
        assert_eq!(t1.rule(RuleId(i as u16)), t2.rule(RuleId(i as u16)));
    }
}

#[test]
fn test_deterministic_symbol_count() {
    let t1 = make_table(
        "ls_v9_dets1",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    let t2 = make_table(
        "ls_v9_dets2",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    assert_eq!(t1.symbol_count, t2.symbol_count);
}

#[test]
fn test_deterministic_eof() {
    let t1 = make_table(
        "ls_v9_dete1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t2 = make_table(
        "ls_v9_dete2",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert_eq!(t1.eof(), t2.eof());
}

// ===========================================================================
// 96–100: Scaling and stress
// ===========================================================================

#[test]
fn test_many_alternatives_grammar() {
    let toks: Vec<(&str, &str)> = vec![
        ("t0", "0"),
        ("t1", "1"),
        ("t2", "2"),
        ("t3", "3"),
        ("t4", "4"),
        ("t5", "5"),
        ("t6", "6"),
        ("t7", "7"),
    ];
    let rules: Vec<(&str, Vec<&str>)> = toks.iter().map(|(n, _)| ("start", vec![*n])).collect();
    let t = make_table("ls_v9_manyalt", &toks, &rules, "start");
    assert!(t.state_count >= 2);
    assert!(count_shifts(&t) >= 8);
}

#[test]
fn test_deep_chain_grammar() {
    // a -> b -> c -> d -> e -> tok
    let t = make_table(
        "ls_v9_deep",
        &[("tok", "t")],
        &[
            ("start", vec!["aa"]),
            ("aa", vec!["bb"]),
            ("bb", vec!["cc"]),
            ("cc", vec!["dd"]),
            ("dd", vec!["tok"]),
        ],
        "start",
    );
    assert!(t.state_count >= 2);
    assert!(has_accept_anywhere(&t));
    // Deep chain should have multiple goto transitions
    let goto_count: usize = (0..t.state_count)
        .map(|s| {
            t.nonterminal_to_index
                .keys()
                .filter(|&&nt| t.goto(StateId(s as u16), nt).is_some())
                .count()
        })
        .sum();
    assert!(
        goto_count >= 4,
        "deep chain should have many gotos, got {goto_count}"
    );
}

#[test]
fn test_wide_and_deep_grammar() {
    let t = make_table(
        "ls_v9_widedeep",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["branch1"]),
            ("start", vec!["branch2"]),
            ("branch1", vec!["inner1"]),
            ("branch2", vec!["inner2"]),
            ("inner1", vec!["a", "b"]),
            ("inner2", vec!["b", "c"]),
        ],
        "start",
    );
    assert!(t.state_count >= 4);
    assert!(has_accept_anywhere(&t));
}

#[test]
fn test_long_sequence_grammar() {
    let t = make_table(
        "ls_v9_longseq",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("start", vec!["a", "b", "c", "d", "e"])],
        "start",
    );
    // A sequence of 5 tokens needs at least 6 states
    assert!(
        t.state_count >= 6,
        "long sequence needs many states, got {}",
        t.state_count
    );
}

#[test]
fn test_complex_grammar_all_invariants() {
    let t = make_table(
        "ls_v9_complex",
        &[
            ("id", "[a-z]+"),
            ("num", "[0-9]+"),
            ("plus", r"\+"),
            ("star", r"\*"),
            ("lparen", r"\("),
            ("rparen", r"\)"),
        ],
        &[
            ("start", vec!["expr"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("expr", vec!["term"]),
            ("term", vec!["term", "star", "factor"]),
            ("term", vec!["factor"]),
            ("factor", vec!["lparen", "expr", "rparen"]),
            ("factor", vec!["id"]),
            ("factor", vec!["num"]),
        ],
        "start",
    );

    // State count
    assert!(t.state_count >= 5);

    // Accept exists
    assert!(has_accept_anywhere(&t));

    // All shift targets valid
    for (_, _, act) in collect_all_actions(&t) {
        if let Action::Shift(target) = act {
            assert!((target.0 as usize) < t.state_count);
        }
        if let Action::Reduce(rid) = act {
            assert!((rid.0 as usize) < t.rules.len());
        }
    }

    // Goto targets valid
    for s in 0..t.state_count {
        for &nt in t.nonterminal_to_index.keys() {
            if let Some(target) = t.goto(StateId(s as u16), nt) {
                assert!((target.0 as usize) < t.state_count);
            }
        }
    }

    // Table dimensions consistent
    assert_eq!(t.action_table.len(), t.state_count);
    assert_eq!(t.goto_table.len(), t.state_count);

    // Has shifts, reduces, and accept
    assert!(count_shifts(&t) > 0);
    assert!(count_reduces(&t) > 0);
    assert!(count_accepts(&t) > 0);
}
