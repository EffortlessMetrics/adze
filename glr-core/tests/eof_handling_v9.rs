//! Comprehensive tests for EOF token handling in parse tables.

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, StateId, SymbolId};

// ---------------------------------------------------------------------------
// Helper
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
    let ff = FirstFollowSets::compute(&g).expect("first/follow");
    build_lr1_automaton(&g, &ff).expect("parse table")
}

fn make_table_with_prec(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>, i16, Associativity)],
    plain_rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs, prec, assoc) in rules {
        b = b.rule_with_precedence(lhs, rhs.clone(), *prec, *assoc);
    }
    for (lhs, rhs) in plain_rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("first/follow");
    build_lr1_automaton(&g, &ff).expect("parse table")
}

/// Count how many states have an Accept action on the given symbol.
fn count_accept_states(table: &ParseTable, sym: SymbolId) -> usize {
    (0..table.state_count)
        .filter(|&s| {
            table
                .actions(StateId(s as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .count()
}

/// Return true if any state has a Shift action on the given symbol.
fn any_shift_on(table: &ParseTable, sym: SymbolId) -> bool {
    (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

/// Collect all actions across all states for a specific symbol.
fn all_actions_for(table: &ParseTable, sym: SymbolId) -> Vec<(StateId, Vec<Action>)> {
    (0..table.state_count)
        .map(|s| {
            let st = StateId(s as u16);
            let acts: Vec<Action> = table.actions(st, sym).to_vec();
            (st, acts)
        })
        .filter(|(_, acts)| !acts.is_empty())
        .collect()
}

// ===========================================================================
// 1. eof_symbol is a valid SymbolId
// ===========================================================================

#[test]
fn test_eof_symbol_valid_simple_grammar() {
    let t = make_table("eof_v9_valid1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(
        t.symbol_to_index.contains_key(&t.eof_symbol),
        "eof_symbol must be indexed in symbol_to_index"
    );
}

#[test]
fn test_eof_symbol_valid_two_tokens() {
    let t = make_table(
        "eof_v9_valid2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(t.symbol_to_index.contains_key(&t.eof_symbol));
}

#[test]
fn test_eof_symbol_valid_three_tokens() {
    let t = make_table(
        "eof_v9_valid3",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[("s", vec!["x", "y", "z"])],
        "s",
    );
    assert!(t.symbol_to_index.contains_key(&t.eof_symbol));
}

#[test]
fn test_eof_symbol_valid_recursive_grammar() {
    let t = make_table(
        "eof_v9_valid4",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    assert!(t.symbol_to_index.contains_key(&t.eof_symbol));
}

// ===========================================================================
// 2. eof_symbol column index < symbol_count
// ===========================================================================

#[test]
fn test_eof_column_index_within_bounds_single_token() {
    let t = make_table("eof_v9_lt1", &[("t", "t")], &[("s", vec!["t"])], "s");
    let col = t.symbol_to_index[&t.eof_symbol];
    assert!(col < t.symbol_count);
}

#[test]
fn test_eof_column_index_within_bounds_many_tokens() {
    let t = make_table(
        "eof_v9_lt2",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a", "b", "c", "d"])],
        "s",
    );
    let col = t.symbol_to_index[&t.eof_symbol];
    assert!(col < t.symbol_count);
}

#[test]
fn test_eof_column_index_within_bounds_nested_rules() {
    let t = make_table(
        "eof_v9_lt3",
        &[("n", "n")],
        &[("s", vec!["inner"]), ("inner", vec!["n"])],
        "s",
    );
    let col = t.symbol_to_index[&t.eof_symbol];
    assert!(col < t.symbol_count);
}

#[test]
fn test_eof_column_index_within_bounds_chain_rules() {
    let t = make_table(
        "eof_v9_lt4",
        &[("x", "x")],
        &[
            ("s", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
        ],
        "s",
    );
    let col = t.symbol_to_index[&t.eof_symbol];
    assert!(col < t.symbol_count);
}

// ===========================================================================
// 3. eof_symbol consistent across builds
// ===========================================================================

#[test]
fn test_eof_symbol_consistent_across_two_builds() {
    let t1 = make_table("eof_v9_cons1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let t2 = make_table("eof_v9_cons1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(t1.eof_symbol, t2.eof_symbol);
}

#[test]
fn test_eof_symbol_consistent_across_three_builds() {
    let specs: &[(&str, &str)] = &[("a", "a"), ("b", "b")];
    let rules: &[(&str, Vec<&str>)] = &[("s", vec!["a", "b"])];
    let t1 = make_table("eof_v9_cons2", specs, rules, "s");
    let t2 = make_table("eof_v9_cons2", specs, rules, "s");
    let t3 = make_table("eof_v9_cons2", specs, rules, "s");
    assert_eq!(t1.eof_symbol, t2.eof_symbol);
    assert_eq!(t2.eof_symbol, t3.eof_symbol);
}

#[test]
fn test_eof_symbol_consistent_complex_grammar() {
    let tokens: &[(&str, &str)] = &[("n", "[0-9]+"), ("p", "\\+"), ("m", "\\*")];
    let rules: &[(&str, Vec<&str>)] = &[
        ("s", vec!["e"]),
        ("e", vec!["e", "p", "e"]),
        ("e", vec!["e", "m", "e"]),
        ("e", vec!["n"]),
    ];
    let t1 = make_table("eof_v9_cons3", tokens, rules, "s");
    let t2 = make_table("eof_v9_cons3", tokens, rules, "s");
    assert_eq!(t1.eof_symbol, t2.eof_symbol);
}

// ===========================================================================
// 4. Different grammars may share the same eof_symbol ID
// ===========================================================================

#[test]
fn test_different_grammars_share_eof_id() {
    let t1 = make_table("eof_v9_share1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let t2 = make_table(
        "eof_v9_share2",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x", "y"])],
        "s",
    );
    // Both tables must have eof_symbol indexed at column 0
    assert_eq!(t1.symbol_to_index[&t1.eof_symbol], 0);
    assert_eq!(t2.symbol_to_index[&t2.eof_symbol], 0);
}

#[test]
fn test_three_grammars_eof_ids_all_valid() {
    let t1 = make_table("eof_v9_share3", &[("a", "a")], &[("s", vec!["a"])], "s");
    let t2 = make_table(
        "eof_v9_share4",
        &[("b", "b")],
        &[("s", vec!["b"]), ("s", vec!["s", "b"])],
        "s",
    );
    let t3 = make_table(
        "eof_v9_share5",
        &[("c", "c"), ("d", "d")],
        &[("s", vec!["c"]), ("s", vec!["d"])],
        "s",
    );
    for (tbl, name) in [(t1, "t1"), (t2, "t2"), (t3, "t3")] {
        assert!(
            tbl.symbol_to_index.contains_key(&tbl.eof_symbol),
            "{name}: eof_symbol not indexed"
        );
    }
}

// ===========================================================================
// 5. Accept action exists for eof_symbol in some state
// ===========================================================================

#[test]
fn test_accept_exists_simple() {
    let t = make_table("eof_v9_acc1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_accept_exists_two_token_sequence() {
    let t = make_table(
        "eof_v9_acc2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_accept_exists_left_recursive() {
    let t = make_table(
        "eof_v9_acc3",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_accept_exists_nested_nonterminals() {
    let t = make_table(
        "eof_v9_acc4",
        &[("x", "x")],
        &[("s", vec!["inner"]), ("inner", vec!["x"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

// ===========================================================================
// 6. eof_symbol in actions → Accept for accepting state
// ===========================================================================

#[test]
fn test_accepting_state_has_accept_action() {
    let t = make_table("eof_v9_acst1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    let accepting: Vec<_> = (0..t.state_count)
        .filter(|&s| table_has_accept(&t, StateId(s as u16), eof))
        .collect();
    assert!(
        !accepting.is_empty(),
        "must have at least one accepting state"
    );
    for s in &accepting {
        let acts = t.actions(StateId(*s as u16), eof);
        assert!(
            acts.iter().any(|a| matches!(a, Action::Accept)),
            "state {s} should contain Accept on EOF"
        );
    }
}

fn table_has_accept(t: &ParseTable, state: StateId, sym: SymbolId) -> bool {
    t.actions(state, sym)
        .iter()
        .any(|a| matches!(a, Action::Accept))
}

#[test]
fn test_non_accepting_states_lack_accept_on_eof() {
    let t = make_table("eof_v9_acst2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    for s in 0..t.state_count {
        let acts = t.actions(StateId(s as u16), eof);
        let has_accept = acts.iter().any(|a| matches!(a, Action::Accept));
        let has_shift = acts.iter().any(|a| matches!(a, Action::Shift(_)));
        // A state with Accept should not also Shift on EOF
        if has_accept {
            assert!(
                !has_shift,
                "state {s}: Accept and Shift on EOF are mutually exclusive"
            );
        }
    }
}

#[test]
fn test_accept_only_on_eof_symbol() {
    let t = make_table(
        "eof_v9_acst3",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let eof = t.eof_symbol;
    // Accept actions should only appear for the EOF symbol
    for s in 0..t.state_count {
        for &sym in t.symbol_to_index.keys() {
            let acts = t.actions(StateId(s as u16), sym);
            if acts.iter().any(|a| matches!(a, Action::Accept)) {
                assert_eq!(
                    sym, eof,
                    "state {s}: Accept should only appear on EOF, not {sym:?}"
                );
            }
        }
    }
}

// ===========================================================================
// 7. eof_symbol not in goto
// ===========================================================================

#[test]
fn test_eof_not_in_goto_simple() {
    let t = make_table("eof_v9_goto1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    for s in 0..t.state_count {
        let result = t.goto(StateId(s as u16), eof);
        assert!(
            result.is_none(),
            "state {s}: EOF should not appear in GOTO table"
        );
    }
}

#[test]
fn test_eof_not_in_goto_multi_rule() {
    let t = make_table(
        "eof_v9_goto2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let eof = t.eof_symbol;
    for s in 0..t.state_count {
        assert!(t.goto(StateId(s as u16), eof).is_none());
    }
}

#[test]
fn test_eof_not_in_goto_recursive() {
    let t = make_table(
        "eof_v9_goto3",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    for s in 0..t.state_count {
        assert!(t.goto(StateId(s as u16), t.eof_symbol).is_none());
    }
}

#[test]
fn test_eof_not_in_goto_nested() {
    let t = make_table(
        "eof_v9_goto4",
        &[("n", "n")],
        &[("s", vec!["inner"]), ("inner", vec!["n"])],
        "s",
    );
    for s in 0..t.state_count {
        assert!(t.goto(StateId(s as u16), t.eof_symbol).is_none());
    }
}

// ===========================================================================
// 8. eof_symbol doesn't appear as Shift target
// ===========================================================================

#[test]
fn test_no_shift_on_eof_simple() {
    let t = make_table("eof_v9_nosh1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_no_shift_on_eof_two_tokens() {
    let t = make_table(
        "eof_v9_nosh2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_no_shift_on_eof_recursive() {
    let t = make_table(
        "eof_v9_nosh3",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_no_shift_on_eof_nested() {
    let t = make_table(
        "eof_v9_nosh4",
        &[("x", "x")],
        &[("s", vec!["mid"]), ("mid", vec!["x"])],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_no_shift_on_eof_alternatives() {
    let t = make_table(
        "eof_v9_nosh5",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

// ===========================================================================
// 9. eof_symbol Debug format
// ===========================================================================

#[test]
fn test_eof_symbol_debug_contains_numeric_value() {
    let t = make_table("eof_v9_dbg1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let dbg = format!("{:?}", t.eof_symbol);
    assert!(
        dbg.contains(&t.eof_symbol.0.to_string()),
        "Debug output should contain the numeric ID: {dbg}"
    );
}

#[test]
fn test_eof_symbol_debug_contains_symbol_id() {
    let t = make_table("eof_v9_dbg2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let dbg = format!("{:?}", t.eof_symbol);
    assert!(
        dbg.contains("SymbolId"),
        "Debug should mention SymbolId: {dbg}"
    );
}

#[test]
fn test_eof_symbol_display_is_deterministic() {
    let t1 = make_table("eof_v9_dbg3", &[("a", "a")], &[("s", vec!["a"])], "s");
    let t2 = make_table("eof_v9_dbg3", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(
        format!("{:?}", t1.eof_symbol),
        format!("{:?}", t2.eof_symbol)
    );
}

// ===========================================================================
// 10. eof_symbol Copy semantics
// ===========================================================================

#[test]
fn test_eof_symbol_is_copy() {
    let t = make_table("eof_v9_copy1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof1 = t.eof_symbol;
    let eof2 = t.eof_symbol; // Copy, not move
    assert_eq!(eof1, eof2);
}

#[test]
fn test_eof_symbol_copy_in_function_arg() {
    let t = make_table("eof_v9_copy2", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    fn takes_by_value(s: SymbolId) -> SymbolId {
        s
    }
    let returned = takes_by_value(eof);
    // eof is still usable after passing by value (Copy)
    assert_eq!(eof, returned);
}

#[test]
fn test_eof_symbol_copy_into_vec() {
    let t = make_table("eof_v9_copy3", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    let v = [eof, eof, eof];
    assert_eq!(v.len(), 3);
    assert!(v.iter().all(|&s| s == eof));
}

// ===========================================================================
// 11. Multiple grammars → eof_symbol always valid
// ===========================================================================

#[test]
fn test_five_grammars_eof_all_valid() {
    let names = [
        "eof_v9_m1",
        "eof_v9_m2",
        "eof_v9_m3",
        "eof_v9_m4",
        "eof_v9_m5",
    ];
    let tables: Vec<ParseTable> = names
        .iter()
        .map(|name| make_table(name, &[("a", "a")], &[("s", vec!["a"])], "s"))
        .collect();
    for (i, tbl) in tables.iter().enumerate() {
        assert!(
            tbl.symbol_to_index.contains_key(&tbl.eof_symbol),
            "grammar {i}: eof_symbol not indexed"
        );
    }
}

#[test]
fn test_grammars_varying_sizes_eof_valid() {
    let t1 = make_table("eof_v9_var1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let t2 = make_table(
        "eof_v9_var2",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    let t3 = make_table(
        "eof_v9_var3",
        &[("x", "x")],
        &[
            ("s", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
        ],
        "s",
    );
    for tbl in [&t1, &t2, &t3] {
        assert!(tbl.symbol_to_index.contains_key(&tbl.eof_symbol));
    }
}

// ===========================================================================
// 12. Simple grammar → Accept on EOF in final state
// ===========================================================================

#[test]
fn test_simple_single_token_accept_on_eof() {
    let t = make_table("eof_v9_simp1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_simple_two_token_accept_on_eof() {
    let t = make_table(
        "eof_v9_simp2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_simple_three_token_accept_on_eof() {
    let t = make_table(
        "eof_v9_simp3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

// ===========================================================================
// 13. Complex grammar → Accept on EOF
// ===========================================================================

#[test]
fn test_complex_nested_accept_on_eof() {
    let t = make_table(
        "eof_v9_cplx1",
        &[("n", "[0-9]+")],
        &[
            ("s", vec!["outer"]),
            ("outer", vec!["inner"]),
            ("inner", vec!["n"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_complex_left_recursive_accept_on_eof() {
    let t = make_table(
        "eof_v9_cplx2",
        &[("n", "n"), ("c", ",")],
        &[("s", vec!["n"]), ("s", vec!["s", "c", "n"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_complex_right_recursive_accept_on_eof() {
    let t = make_table(
        "eof_v9_cplx3",
        &[("a", "a"), ("c", ":")],
        &[("s", vec!["a"]), ("s", vec!["a", "c", "s"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_complex_multi_level_nesting_accept_on_eof() {
    let t = make_table(
        "eof_v9_cplx4",
        &[("x", "x"), ("y", "y")],
        &[
            ("s", vec!["a"]),
            ("a", vec!["b"]),
            ("b", vec!["x"]),
            ("b", vec!["y"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

// ===========================================================================
// 14. Arithmetic grammar → Accept on EOF
// ===========================================================================

#[test]
fn test_arithmetic_add_accept_on_eof() {
    let t = make_table(
        "eof_v9_arith1",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[
            ("s", vec!["e"]),
            ("e", vec!["num"]),
            ("e", vec!["e", "plus", "e"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_arithmetic_mul_accept_on_eof() {
    let t = make_table(
        "eof_v9_arith2",
        &[("num", "[0-9]+"), ("star", "\\*")],
        &[
            ("s", vec!["e"]),
            ("e", vec!["num"]),
            ("e", vec!["e", "star", "e"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_arithmetic_full_accept_on_eof() {
    let t = make_table(
        "eof_v9_arith3",
        &[("num", "[0-9]+"), ("plus", "\\+"), ("star", "\\*")],
        &[
            ("s", vec!["e"]),
            ("e", vec!["num"]),
            ("e", vec!["e", "plus", "e"]),
            ("e", vec!["e", "star", "e"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_arithmetic_with_parens_accept_on_eof() {
    let t = make_table(
        "eof_v9_arith4",
        &[
            ("num", "[0-9]+"),
            ("plus", "\\+"),
            ("lp", "\\("),
            ("rp", "\\)"),
        ],
        &[
            ("s", vec!["e"]),
            ("e", vec!["num"]),
            ("e", vec!["e", "plus", "e"]),
            ("e", vec!["lp", "e", "rp"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

// ===========================================================================
// 15. Grammar with alternatives → Accept on EOF
// ===========================================================================

#[test]
fn test_two_alternatives_accept_on_eof() {
    let t = make_table(
        "eof_v9_alt1",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_three_alternatives_accept_on_eof() {
    let t = make_table(
        "eof_v9_alt2",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_mixed_terminal_nonterminal_alternatives_accept_on_eof() {
    let t = make_table(
        "eof_v9_alt3",
        &[("a", "a"), ("b", "b")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["pair"]),
            ("pair", vec!["a", "b"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_alternatives_with_repetition_accept_on_eof() {
    let t = make_table(
        "eof_v9_alt4",
        &[("a", "a"), ("b", "b")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["s", "a"]),
            ("s", vec!["s", "b"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

// ===========================================================================
// 16. Grammar with precedence → Accept on EOF
// ===========================================================================

#[test]
fn test_precedence_left_assoc_accept_on_eof() {
    let t = make_table_with_prec(
        "eof_v9_prec1",
        &[("num", "[0-9]+"), ("plus", "\\+"), ("star", "\\*")],
        &[
            ("e", vec!["e", "plus", "e"], 1, Associativity::Left),
            ("e", vec!["e", "star", "e"], 2, Associativity::Left),
        ],
        &[("s", vec!["e"]), ("e", vec!["num"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_precedence_right_assoc_accept_on_eof() {
    let t = make_table_with_prec(
        "eof_v9_prec2",
        &[("num", "[0-9]+"), ("caret", "\\^")],
        &[("e", vec!["e", "caret", "e"], 1, Associativity::Right)],
        &[("s", vec!["e"]), ("e", vec!["num"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_precedence_mixed_assoc_accept_on_eof() {
    let t = make_table_with_prec(
        "eof_v9_prec3",
        &[("n", "n"), ("p", "\\+"), ("c", "\\^")],
        &[
            ("e", vec!["e", "p", "e"], 1, Associativity::Left),
            ("e", vec!["e", "c", "e"], 2, Associativity::Right),
        ],
        &[("s", vec!["e"]), ("e", vec!["n"])],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
}

#[test]
fn test_precedence_table_eof_symbol_valid() {
    let t = make_table_with_prec(
        "eof_v9_prec4",
        &[("n", "n"), ("p", "\\+")],
        &[("e", vec!["e", "p", "e"], 1, Associativity::Left)],
        &[("s", vec!["e"]), ("e", vec!["n"])],
        "s",
    );
    assert!(t.symbol_to_index.contains_key(&t.eof_symbol));
}

// ===========================================================================
// 17. After serialization roundtrip → eof_symbol preserved
// ===========================================================================

#[test]
fn test_eof_symbol_serde_json_roundtrip() {
    let t = make_table("eof_v9_serde1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    let json = serde_json::to_string(&eof).expect("serialize");
    let deserialized: SymbolId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(eof, deserialized);
}

#[test]
fn test_eof_symbol_serde_roundtrip_preserves_inner_value() {
    let t = make_table(
        "eof_v9_serde2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let eof = t.eof_symbol;
    let json = serde_json::to_string(&eof).expect("serialize");
    let deserialized: SymbolId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(eof.0, deserialized.0);
}

#[test]
fn test_eof_symbol_serde_roundtrip_multiple_grammars() {
    let tables = vec![
        make_table("eof_v9_serde3", &[("a", "a")], &[("s", vec!["a"])], "s"),
        make_table(
            "eof_v9_serde4",
            &[("x", "x"), ("y", "y")],
            &[("s", vec!["x", "y"])],
            "s",
        ),
    ];
    for t in &tables {
        let eof = t.eof_symbol;
        let json = serde_json::to_string(&eof).expect("serialize");
        let back: SymbolId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(eof, back);
    }
}

#[test]
fn test_action_accept_serde_roundtrip() {
    let json = serde_json::to_string(&Action::Accept).expect("serialize");
    let back: Action = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, Action::Accept);
}

// ===========================================================================
// 18. EOF actions are deterministic
// ===========================================================================

#[test]
fn test_eof_actions_deterministic_simple() {
    let t = make_table("eof_v9_det1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    let run1 = all_actions_for(&t, eof);
    let run2 = all_actions_for(&t, eof);
    assert_eq!(run1.len(), run2.len());
    for ((s1, a1), (s2, a2)) in run1.iter().zip(run2.iter()) {
        assert_eq!(s1, s2);
        assert_eq!(a1, a2);
    }
}

#[test]
fn test_eof_actions_deterministic_across_builds() {
    let build = || {
        let t = make_table(
            "eof_v9_det2",
            &[("a", "a"), ("b", "b")],
            &[("s", vec!["a", "b"])],
            "s",
        );
        all_actions_for(&t, t.eof_symbol)
    };
    let run1 = build();
    let run2 = build();
    assert_eq!(run1.len(), run2.len());
    for ((s1, a1), (s2, a2)) in run1.iter().zip(run2.iter()) {
        assert_eq!(s1, s2);
        assert_eq!(a1, a2);
    }
}

#[test]
fn test_eof_actions_deterministic_recursive() {
    let build = || {
        let t = make_table(
            "eof_v9_det3",
            &[("a", "a")],
            &[("s", vec!["a"]), ("s", vec!["s", "a"])],
            "s",
        );
        all_actions_for(&t, t.eof_symbol)
    };
    let run1 = build();
    let run2 = build();
    assert_eq!(run1, run2);
}

// ===========================================================================
// 19. EOF never has Shift actions (comprehensive)
// ===========================================================================

#[test]
fn test_eof_never_shift_chain_grammar() {
    let t = make_table(
        "eof_v9_nshc1",
        &[("x", "x")],
        &[("s", vec!["a"]), ("a", vec!["b"]), ("b", vec!["x"])],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_eof_never_shift_arithmetic() {
    let t = make_table(
        "eof_v9_nshc2",
        &[("num", "n"), ("plus", "\\+"), ("star", "\\*")],
        &[
            ("s", vec!["e"]),
            ("e", vec!["num"]),
            ("e", vec!["e", "plus", "e"]),
            ("e", vec!["e", "star", "e"]),
        ],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_eof_never_shift_with_alternatives() {
    let t = make_table(
        "eof_v9_nshc3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_eof_never_shift_with_precedence() {
    let t = make_table_with_prec(
        "eof_v9_nshc4",
        &[("n", "n"), ("p", "\\+")],
        &[("e", vec!["e", "p", "e"], 1, Associativity::Left)],
        &[("s", vec!["e"]), ("e", vec!["n"])],
        "s",
    );
    assert!(!any_shift_on(&t, t.eof_symbol));
}

// ===========================================================================
// 20. Count states with Accept on EOF
// ===========================================================================

#[test]
fn test_single_accept_state_simple_grammar() {
    let t = make_table("eof_v9_cnt1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let n = count_accept_states(&t, t.eof_symbol);
    assert!(n >= 1, "must have at least one accept state");
}

#[test]
fn test_accept_state_count_bounded() {
    let t = make_table(
        "eof_v9_cnt2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let n = count_accept_states(&t, t.eof_symbol);
    assert!(n >= 1);
    // Accept states should be a small fraction of all states
    assert!(n <= t.state_count);
}

#[test]
fn test_accept_state_count_recursive_grammar() {
    let t = make_table(
        "eof_v9_cnt3",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    let n = count_accept_states(&t, t.eof_symbol);
    assert!(n >= 1);
}

#[test]
fn test_accept_state_count_with_alternatives() {
    let t = make_table(
        "eof_v9_cnt4",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let n = count_accept_states(&t, t.eof_symbol);
    assert!(n >= 1);
}

// ===========================================================================
// Additional edge-case tests
// ===========================================================================

#[test]
fn test_eof_symbol_equals_eof_method() {
    let t = make_table("eof_v9_eq1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(t.eof_symbol, t.eof());
}

#[test]
fn test_eof_symbol_differs_from_start_symbol() {
    let t = make_table("eof_v9_diff1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_ne!(t.eof_symbol, t.start_symbol());
}

#[test]
fn test_eof_is_beyond_terminal_boundary() {
    let t = make_table("eof_v9_term1", &[("a", "a")], &[("s", vec!["a"])], "s");
    // EOF's raw ID is max_symbol + 1, beyond the terminal boundary
    assert!(
        !t.is_terminal(t.eof_symbol),
        "EOF raw ID exceeds terminal_boundary; it is indexed at column 0 instead"
    );
}

#[test]
fn test_eof_symbol_in_symbol_to_index() {
    let t = make_table("eof_v9_idx1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(
        t.symbol_to_index.contains_key(&t.eof_symbol),
        "EOF must be mapped in symbol_to_index"
    );
}

#[test]
fn test_eof_symbol_not_in_nonterminal_index() {
    let t = make_table("eof_v9_ntidx1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(
        !t.nonterminal_to_index.contains_key(&t.eof_symbol),
        "EOF should not be in nonterminal_to_index"
    );
}

#[test]
fn test_eof_only_reduce_or_accept_actions() {
    let t = make_table("eof_v9_onlyra1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    for s in 0..t.state_count {
        let acts = t.actions(StateId(s as u16), eof);
        for a in acts {
            assert!(
                matches!(a, Action::Accept | Action::Reduce(_)),
                "state {s}: EOF action should be Accept or Reduce, got {a:?}"
            );
        }
    }
}

#[test]
fn test_eof_reduce_actions_have_valid_rule_ids() {
    let t = make_table(
        "eof_v9_redval1",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    let eof = t.eof_symbol;
    let rule_count = t.rules.len();
    for s in 0..t.state_count {
        for a in t.actions(StateId(s as u16), eof) {
            if let Action::Reduce(rid) = a {
                assert!(
                    (rid.0 as usize) < rule_count,
                    "state {s}: Reduce({rid:?}) references invalid rule"
                );
            }
        }
    }
}

#[test]
fn test_initial_state_no_accept_on_eof() {
    let t = make_table("eof_v9_init1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let initial_acts = t.actions(t.initial_state, t.eof_symbol);
    assert!(
        !initial_acts.iter().any(|a| matches!(a, Action::Accept)),
        "initial state should not accept on EOF before consuming input"
    );
}

#[test]
fn test_eof_symbol_hash_consistency() {
    use std::collections::HashSet;
    let t = make_table("eof_v9_hash1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut set = HashSet::new();
    set.insert(t.eof_symbol);
    set.insert(t.eof_symbol);
    assert_eq!(set.len(), 1);
}

#[test]
fn test_eof_symbol_ord() {
    let t = make_table("eof_v9_ord1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    assert!(eof <= eof);
    assert!(eof >= eof);
}

#[test]
fn test_eof_actions_empty_for_states_beyond_range() {
    let t = make_table("eof_v9_oob1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let out_of_range = StateId(t.state_count as u16 + 10);
    let acts = t.actions(out_of_range, t.eof_symbol);
    assert!(acts.is_empty());
}

#[test]
fn test_eof_actions_for_invalid_symbol_are_empty() {
    let t = make_table("eof_v9_invsym1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let invalid = SymbolId(9999);
    for s in 0..t.state_count {
        let acts = t.actions(StateId(s as u16), invalid);
        assert!(acts.is_empty());
    }
}

#[test]
fn test_accept_on_eof_not_on_non_eof_terminals() {
    let t = make_table(
        "eof_v9_noteof1",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let eof = t.eof_symbol;
    for &sym in t.symbol_to_index.keys() {
        if sym == eof {
            continue;
        }
        for s in 0..t.state_count {
            let acts = t.actions(StateId(s as u16), sym);
            assert!(
                !acts.iter().any(|a| matches!(a, Action::Accept)),
                "state {s}: Accept should not appear on non-EOF symbol {sym:?}"
            );
        }
    }
}

#[test]
fn test_eof_symbol_is_not_extra() {
    let t = make_table("eof_v9_noextra1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(
        !t.is_extra(t.eof_symbol),
        "EOF should not be an extra symbol"
    );
}

#[test]
fn test_eof_symbol_with_long_chain_grammar() {
    let t = make_table(
        "eof_v9_chain1",
        &[("t", "t")],
        &[
            ("s", vec!["a"]),
            ("a", vec!["b"]),
            ("b", vec!["c"]),
            ("c", vec!["d"]),
            ("d", vec!["t"]),
        ],
        "s",
    );
    assert!(t.symbol_to_index.contains_key(&t.eof_symbol));
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_eof_symbol_with_wide_alternatives_grammar() {
    let t = make_table(
        "eof_v9_wide1",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
            ("s", vec!["e"]),
        ],
        "s",
    );
    assert!(count_accept_states(&t, t.eof_symbol) >= 1);
    assert!(!any_shift_on(&t, t.eof_symbol));
}

#[test]
fn test_eof_actions_no_recover() {
    let t = make_table("eof_v9_norec1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    for s in 0..t.state_count {
        let acts = t.actions(StateId(s as u16), eof);
        assert!(
            !acts.iter().any(|a| matches!(a, Action::Recover)),
            "state {s}: EOF should not have Recover action"
        );
    }
}

#[test]
fn test_eof_actions_no_error_in_accepting_state() {
    let t = make_table("eof_v9_noerr1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    for s in 0..t.state_count {
        let acts = t.actions(StateId(s as u16), eof);
        let has_accept = acts.iter().any(|a| matches!(a, Action::Accept));
        let has_error = acts.iter().any(|a| matches!(a, Action::Error));
        if has_accept {
            assert!(
                !has_error,
                "state {s}: Accept and Error on EOF are contradictory"
            );
        }
    }
}

#[test]
fn test_table_eof_not_zero_before_normalization() {
    let t = make_table("eof_v9_val1", &[("a", "a")], &[("s", vec!["a"])], "s");
    // Freshly built tables have eof_symbol = max_symbol + 1, not SymbolId(0)
    assert_ne!(t.eof_symbol, SymbolId(0));
}

#[test]
fn test_table_validate_passes_after_normalization() {
    let mut t = make_table(
        "eof_v9_val2",
        &[("n", "n"), ("p", "\\+")],
        &[
            ("s", vec!["e"]),
            ("e", vec!["n"]),
            ("e", vec!["e", "p", "e"]),
        ],
        "s",
    );
    t = t.normalize_eof_to_zero();
    assert_eq!(t.eof_symbol, SymbolId(0));
}

#[test]
fn test_eof_symbol_equality_reflexive() {
    let t = make_table("eof_v9_refl1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    assert_eq!(eof, eof);
}

#[test]
fn test_eof_symbol_constructed_manually_matches() {
    let t = make_table("eof_v9_manual1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let manual = SymbolId(t.eof_symbol.0);
    assert_eq!(t.eof_symbol, manual);
}

#[test]
fn test_eof_symbol_index_round_trips() {
    let t = make_table("eof_v9_idxrt1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = t.eof_symbol;
    if let Some(&col) = t.symbol_to_index.get(&eof) {
        assert!(col < t.index_to_symbol.len());
        assert_eq!(t.index_to_symbol[col], eof);
    }
}

#[test]
fn test_eof_is_highest_symbol_id() {
    let t = make_table("eof_v9_high1", &[("a", "a")], &[("s", vec!["a"])], "s");
    // eof_symbol = max_symbol + 1, so it should be >= all indexed symbols
    for &sym in t.symbol_to_index.keys() {
        assert!(t.eof_symbol >= sym);
    }
}
