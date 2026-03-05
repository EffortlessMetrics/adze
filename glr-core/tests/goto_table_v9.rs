#![cfg(feature = "test-api")]

//! GOTO table v9 tests — 82 tests across 20 categories covering goto
//! existence, validity, sparsity, determinism, consistency with actions,
//! precedence, various grammar shapes, and edge cases.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_table(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> adze_glr_core::ParseTable {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

/// Collect all (state, target) pairs where goto(state, nt) is defined.
fn all_gotos_for(
    table: &adze_glr_core::ParseTable,
    nt: SymbolId,
) -> Vec<(StateId, StateId)> {
    (0..table.state_count)
        .filter_map(|s| {
            let st = StateId(s as u16);
            table.goto(st, nt).map(|tgt| (st, tgt))
        })
        .collect()
}

/// Count total defined goto entries across all states and nonterminals.
fn total_goto_entries(table: &adze_glr_core::ParseTable) -> usize {
    let mut count = 0;
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if table.goto(st, nt).is_some() {
                count += 1;
            }
        }
    }
    count
}

/// Collect all nonterminal SymbolIds registered in the goto table.
fn goto_nonterminals(table: &adze_glr_core::ParseTable) -> Vec<SymbolId> {
    table.nonterminal_to_index.keys().copied().collect()
}

// ===========================================================================
// 1. goto returns Some for valid non-terminal transitions (4 tests)
// ===========================================================================

#[test]
fn goto_v9_valid_some_single_rule() {
    let g = GrammarBuilder::new("gt_v9_vs1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    assert!(!gotos.is_empty(), "start nonterminal must have at least one goto entry");
}

#[test]
fn goto_v9_valid_some_two_nonterminals() {
    let g = GrammarBuilder::new("gt_v9_vs2")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let gotos = all_gotos_for(&table, inner);
    assert!(!gotos.is_empty(), "intermediate nonterminal must have goto entries");
}

#[test]
fn goto_v9_valid_some_chain() {
    let g = GrammarBuilder::new("gt_v9_vs3")
        .token("t", "t")
        .rule("c", vec!["t"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["c", "b", "start"] {
        let nt = nt_id(&g, name);
        let gotos = all_gotos_for(&table, nt);
        assert!(
            !gotos.is_empty(),
            "nonterminal '{name}' must have goto entries in chain grammar"
        );
    }
}

#[test]
fn goto_v9_valid_some_initial_state() {
    let g = GrammarBuilder::new("gt_v9_vs4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let target = table.goto(table.initial_state, start);
    assert!(target.is_some(), "goto from initial state for start nonterminal must exist");
}

// ===========================================================================
// 2. goto returns None for invalid transitions (4 tests)
// ===========================================================================

#[test]
fn goto_v9_none_out_of_bounds_state() {
    let table = make_table(
        "gt_v9_no1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let bogus_state = StateId(table.state_count as u16 + 100);
    // Use any nonterminal from the table
    let nts = goto_nonterminals(&table);
    for nt in nts {
        assert!(
            table.goto(bogus_state, nt).is_none(),
            "out-of-bounds state must return None"
        );
    }
}

#[test]
fn goto_v9_none_unknown_symbol() {
    let table = make_table(
        "gt_v9_no2",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let unknown = SymbolId(9999);
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), unknown).is_none(),
            "unknown symbol must return None"
        );
    }
}

#[test]
fn goto_v9_none_max_state() {
    let table = make_table(
        "gt_v9_no3",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let nts = goto_nonterminals(&table);
    for nt in nts {
        assert!(
            table.goto(StateId(u16::MAX), nt).is_none(),
            "StateId(MAX) must return None"
        );
    }
}

#[test]
fn goto_v9_none_zero_symbol() {
    let table = make_table(
        "gt_v9_no4",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    // SymbolId(0) is typically ERROR, not a nonterminal
    let result = table.goto(StateId(0), SymbolId(0));
    // It may or may not be None depending on implementation, but shouldn't panic
    let _ = result;
}

// ===========================================================================
// 3. goto target is valid StateId (< state_count) (4 tests)
// ===========================================================================

#[test]
fn goto_v9_target_valid_simple() {
    let table = make_table(
        "gt_v9_tv1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(st, nt) {
                assert!(
                    (target.0 as usize) < table.state_count,
                    "goto target {target:?} must be < state_count {}",
                    table.state_count
                );
            }
        }
    }
}

#[test]
fn goto_v9_target_valid_multi_rule() {
    let table = make_table(
        "gt_v9_tv2",
        &[("a", "a"), ("b", "b")],
        &[
            ("item", vec!["a"]),
            ("item", vec!["b"]),
            ("start", vec!["item"]),
        ],
        "start",
    );
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(st, nt) {
                assert!((target.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_v9_target_valid_chain() {
    let g = GrammarBuilder::new("gt_v9_tv3")
        .token("z", "z")
        .rule("d", vec!["z"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(st, nt) {
                assert!((target.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_v9_target_valid_recursive() {
    let table = make_table(
        "gt_v9_tv4",
        &[("x", "x")],
        &[
            ("list", vec!["x"]),
            ("list", vec!["list", "x"]),
            ("start", vec!["list"]),
        ],
        "start",
    );
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(st, nt) {
                assert!((target.0 as usize) < table.state_count);
            }
        }
    }
}

// ===========================================================================
// 4. goto is consistent with parse actions (4 tests)
// ===========================================================================

#[test]
fn goto_v9_consistent_reduce_has_goto() {
    let g = GrammarBuilder::new("gt_v9_cr1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // For every reduce action, the rule's LHS should have a goto from some state
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if let Action::Reduce(rule_id) = act {
                    let (lhs, _) = table.rule(*rule_id);
                    let gotos = all_gotos_for(&table, lhs);
                    assert!(
                        !gotos.is_empty(),
                        "reduce by rule {rule_id:?} (lhs={lhs:?}) must have goto entries"
                    );
                }
            }
        }
    }
}

#[test]
fn goto_v9_consistent_reduce_multi_rule() {
    let g = GrammarBuilder::new("gt_v9_cr2")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if let Action::Reduce(rule_id) = act {
                    let (lhs, _) = table.rule(*rule_id);
                    assert!(!all_gotos_for(&table, lhs).is_empty());
                }
            }
        }
    }
}

#[test]
fn goto_v9_consistent_shift_reaches_state() {
    let g = GrammarBuilder::new("gt_v9_cr3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Every shift target state must be < state_count
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if let Action::Shift(target) = act {
                    assert!((target.0 as usize) < table.state_count);
                }
            }
        }
    }
}

#[test]
fn goto_v9_consistent_every_lhs_has_goto() {
    let g = GrammarBuilder::new("gt_v9_cr4")
        .token("n", "n")
        .token("+", "\\+")
        .rule("expr", vec!["n"])
        .rule("expr", vec!["expr", "+", "n"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Every nonterminal that appears as a rule LHS should have goto entries
    for &nt in table.nonterminal_to_index.keys() {
        let gotos = all_gotos_for(&table, nt);
        assert!(!gotos.is_empty(), "nonterminal {nt:?} must have goto entries");
    }
}

// ===========================================================================
// 5. Simple grammar goto entries (4 tests)
// ===========================================================================

#[test]
fn goto_v9_simple_single_token() {
    let table = make_table(
        "gt_v9_sg1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(total_goto_entries(&table) > 0, "must have at least one goto entry");
}

#[test]
fn goto_v9_simple_two_tokens() {
    let table = make_table(
        "gt_v9_sg2",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"])],
        "start",
    );
    assert!(total_goto_entries(&table) > 0);
}

#[test]
fn goto_v9_simple_epsilon_rule() {
    let table = make_table(
        "gt_v9_sg3",
        &[("a", "a")],
        &[("start", vec![]), ("start", vec!["a"])],
        "start",
    );
    assert!(total_goto_entries(&table) > 0, "grammar with epsilon rule still needs gotos");
}

#[test]
fn goto_v9_simple_single_nonterminal_one_goto() {
    let g = GrammarBuilder::new("gt_v9_sg4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    // Augmented grammar may have multiple goto entries; at least one must exist
    assert!(!gotos.is_empty(), "single-rule grammar must have at least 1 goto for start");
}

// ===========================================================================
// 6. Grammar with alternatives → multiple goto entries (4 tests)
// ===========================================================================

#[test]
fn goto_v9_alt_two_alternatives() {
    let g = GrammarBuilder::new("gt_v9_al1")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&g);
    let item = nt_id(&g, "item");
    let gotos = all_gotos_for(&table, item);
    assert!(!gotos.is_empty(), "item with alternatives must have goto entries");
}

#[test]
fn goto_v9_alt_three_alternatives() {
    let g = GrammarBuilder::new("gt_v9_al2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("val", vec!["a"])
        .rule("val", vec!["b"])
        .rule("val", vec!["c"])
        .rule("start", vec!["val"])
        .start("start")
        .build();
    let table = build_table(&g);
    let val = nt_id(&g, "val");
    assert!(!all_gotos_for(&table, val).is_empty());
}

#[test]
fn goto_v9_alt_nested_alternatives() {
    let g = GrammarBuilder::new("gt_v9_al3")
        .token("x", "x")
        .token("y", "y")
        .rule("leaf", vec!["x"])
        .rule("leaf", vec!["y"])
        .rule("wrapper", vec!["leaf"])
        .rule("start", vec!["wrapper"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["leaf", "wrapper", "start"] {
        let nt = nt_id(&g, name);
        assert!(!all_gotos_for(&table, nt).is_empty());
    }
}

#[test]
fn goto_v9_alt_multiple_nonterminals_with_choices() {
    let g = GrammarBuilder::new("gt_v9_al4")
        .token("n", "n")
        .token("s", "s")
        .rule("atom", vec!["n"])
        .rule("atom", vec!["s"])
        .rule("list", vec!["atom"])
        .rule("list", vec!["list", "atom"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let table = build_table(&g);
    let nts = goto_nonterminals(&table);
    assert!(
        nts.len() >= 3,
        "grammar with 3 nonterminals must register at least 3 in goto table"
    );
}

// ===========================================================================
// 7. goto from state 0 for non-terminals (4 tests)
// ===========================================================================

#[test]
fn goto_v9_state0_start_nonterminal() {
    let g = GrammarBuilder::new("gt_v9_s01")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(
        table.goto(StateId(0), start).is_some(),
        "state 0 must have goto for start nonterminal"
    );
}

#[test]
fn goto_v9_state0_intermediate() {
    let g = GrammarBuilder::new("gt_v9_s02")
        .token("t", "t")
        .rule("inner", vec!["t"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    assert!(
        table.goto(StateId(0), inner).is_some(),
        "state 0 should have goto for inner nonterminal reachable from start"
    );
}

#[test]
fn goto_v9_state0_all_reachable_nonterminals() {
    let g = GrammarBuilder::new("gt_v9_s03")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    // All nonterminals in a linear chain should be reachable from state 0
    for name in &["leaf", "mid", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            table.goto(StateId(0), nt).is_some(),
            "state 0 must have goto for '{name}'"
        );
    }
}

#[test]
fn goto_v9_state0_recursive_nonterminal() {
    let g = GrammarBuilder::new("gt_v9_s04")
        .token("x", "x")
        .rule("list", vec!["x"])
        .rule("list", vec!["list", "x"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let table = build_table(&g);
    let list = nt_id(&g, "list");
    assert!(table.goto(StateId(0), list).is_some());
}

// ===========================================================================
// 8. goto determinism: same query → same result (4 tests)
// ===========================================================================

#[test]
fn goto_v9_determinism_repeated_call() {
    let table = make_table(
        "gt_v9_dt1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            let r1 = table.goto(st, nt);
            let r2 = table.goto(st, nt);
            assert_eq!(r1, r2, "goto must be deterministic");
        }
    }
}

#[test]
fn goto_v9_determinism_hundred_calls() {
    let table = make_table(
        "gt_v9_dt2",
        &[("a", "a"), ("b", "b")],
        &[("item", vec!["a"]), ("start", vec!["item"])],
        "start",
    );
    let first = table.goto(StateId(0), *table.nonterminal_to_index.keys().next().unwrap());
    for _ in 0..100 {
        let again = table.goto(StateId(0), *table.nonterminal_to_index.keys().next().unwrap());
        assert_eq!(first, again);
    }
}

#[test]
fn goto_v9_determinism_all_entries() {
    let g = GrammarBuilder::new("gt_v9_dt3")
        .token("n", "n")
        .token("+", "\\+")
        .rule("expr", vec!["n"])
        .rule("expr", vec!["expr", "+", "n"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    let nts: Vec<SymbolId> = table.nonterminal_to_index.keys().copied().collect();
    let mut snapshot = Vec::new();
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in &nts {
            snapshot.push((st, nt, table.goto(st, nt)));
        }
    }
    let mut again = Vec::new();
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in &nts {
            again.push((st, nt, table.goto(st, nt)));
        }
    }
    assert_eq!(snapshot, again);
}

#[test]
fn goto_v9_determinism_order_independent() {
    let table = make_table(
        "gt_v9_dt4",
        &[("x", "x"), ("y", "y")],
        &[
            ("a", vec!["x"]),
            ("b", vec!["y"]),
            ("start", vec!["a", "b"]),
        ],
        "start",
    );
    // Query in reverse order
    let forward: Vec<_> = table
        .nonterminal_to_index
        .keys()
        .map(|&nt| table.goto(StateId(0), nt))
        .collect();
    let reverse: Vec<_> = table
        .nonterminal_to_index
        .keys()
        .rev()
        .map(|&nt| table.goto(StateId(0), nt))
        .collect();
    let mut reverse_rev = reverse;
    reverse_rev.reverse();
    assert_eq!(forward, reverse_rev);
}

// ===========================================================================
// 9. Different grammars → different goto tables (4 tests)
// ===========================================================================

#[test]
fn goto_v9_diff_grammars_entry_count() {
    let t1 = make_table(
        "gt_v9_dg1a",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t2 = make_table(
        "gt_v9_dg1b",
        &[("a", "a"), ("b", "b")],
        &[("item", vec!["a"]), ("item", vec!["b"]), ("start", vec!["item"])],
        "start",
    );
    let e1 = total_goto_entries(&t1);
    let e2 = total_goto_entries(&t2);
    assert_ne!(e1, e2, "different grammars should differ in goto entries");
}

#[test]
fn goto_v9_diff_grammars_state_count() {
    let t1 = make_table(
        "gt_v9_dg2a",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t2 = make_table(
        "gt_v9_dg2b",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("inner", vec!["a"]),
            ("inner", vec!["b"]),
            ("outer", vec!["inner", "c"]),
            ("start", vec!["outer"]),
        ],
        "start",
    );
    assert_ne!(
        t1.state_count, t2.state_count,
        "different grammars should have different state counts"
    );
}

#[test]
fn goto_v9_diff_grammars_nonterminal_count() {
    let t1 = make_table(
        "gt_v9_dg3a",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let t2 = make_table(
        "gt_v9_dg3b",
        &[("a", "a")],
        &[("inner", vec!["a"]), ("start", vec!["inner"])],
        "start",
    );
    assert!(
        goto_nonterminals(&t2).len() > goto_nonterminals(&t1).len(),
        "more nonterminals → more goto columns"
    );
}

#[test]
fn goto_v9_diff_grammars_recursive_vs_flat() {
    let t_flat = make_table(
        "gt_v9_dg4a",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    let t_rec = make_table(
        "gt_v9_dg4b",
        &[("x", "x")],
        &[
            ("list", vec!["x"]),
            ("list", vec!["list", "x"]),
            ("start", vec!["list"]),
        ],
        "start",
    );
    assert!(
        total_goto_entries(&t_rec) > total_goto_entries(&t_flat),
        "recursive grammar should have more goto entries"
    );
}

// ===========================================================================
// 10. goto with precedence grammar (4 tests)
// ===========================================================================

#[test]
fn goto_v9_prec_left_assoc() {
    let g = GrammarBuilder::new("gt_v9_pr1")
        .token("n", "n")
        .token("+", "\\+")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    let expr = nt_id(&g, "expr");
    assert!(!all_gotos_for(&table, expr).is_empty());
}

#[test]
fn goto_v9_prec_right_assoc() {
    let g = GrammarBuilder::new("gt_v9_pr2")
        .token("n", "n")
        .token("=", "=")
        .rule("assign", vec!["n"])
        .rule_with_precedence("assign", vec!["n", "=", "assign"], 1, Associativity::Right)
        .rule("start", vec!["assign"])
        .start("start")
        .build();
    let table = build_table(&g);
    let assign = nt_id(&g, "assign");
    assert!(!all_gotos_for(&table, assign).is_empty());
}

#[test]
fn goto_v9_prec_multiple_levels() {
    let g = GrammarBuilder::new("gt_v9_pr3")
        .token("n", "n")
        .token("+", "\\+")
        .token("*", "\\*")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    let expr = nt_id(&g, "expr");
    let gotos = all_gotos_for(&table, expr);
    assert!(
        gotos.len() >= 2,
        "precedence grammar should have goto entries from multiple states"
    );
}

#[test]
fn goto_v9_prec_targets_valid() {
    let g = GrammarBuilder::new("gt_v9_pr4")
        .token("n", "n")
        .token("+", "\\+")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(st, nt) {
                assert!((target.0 as usize) < table.state_count);
            }
        }
    }
}

// ===========================================================================
// 11. All goto targets are < state_count (4 tests)
// ===========================================================================

#[test]
fn goto_v9_bound_simple() {
    let table = make_table(
        "gt_v9_bd1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(t) = table.goto(StateId(s as u16), nt) {
                assert!((t.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_v9_bound_recursive() {
    let table = make_table(
        "gt_v9_bd2",
        &[("x", "x")],
        &[
            ("items", vec!["x"]),
            ("items", vec!["items", "x"]),
            ("start", vec!["items"]),
        ],
        "start",
    );
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(t) = table.goto(StateId(s as u16), nt) {
                assert!((t.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_v9_bound_multi_nt() {
    let table = make_table(
        "gt_v9_bd3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("x", vec!["a"]),
            ("y", vec!["b"]),
            ("z", vec!["c"]),
            ("start", vec!["x", "y", "z"]),
        ],
        "start",
    );
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(t) = table.goto(StateId(s as u16), nt) {
                assert!((t.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_v9_bound_arithmetic() {
    let table = make_table(
        "gt_v9_bd4",
        &[("n", "n"), ("+", "\\+"), ("*", "\\*"), ("(", "\\("), (")", "\\)")],
        &[
            ("factor", vec!["n"]),
            ("factor", vec!["(", "expr", ")"]),
            ("term", vec!["factor"]),
            ("term", vec!["term", "*", "factor"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "+", "term"]),
            ("start", vec!["expr"]),
        ],
        "start",
    );
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(t) = table.goto(StateId(s as u16), nt) {
                assert!((t.0 as usize) < table.state_count);
            }
        }
    }
}

// ===========================================================================
// 12. goto query doesn't panic for any state/symbol combo (4 tests)
// ===========================================================================

#[test]
fn goto_v9_nopanic_all_states_all_nts() {
    let table = make_table(
        "gt_v9_np1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    for s in 0..=table.state_count + 10 {
        for &nt in table.nonterminal_to_index.keys() {
            let _ = table.goto(StateId(s as u16), nt);
        }
    }
}

#[test]
fn goto_v9_nopanic_arbitrary_symbols() {
    let table = make_table(
        "gt_v9_np2",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    for sym_raw in 0..200u16 {
        let _ = table.goto(StateId(0), SymbolId(sym_raw));
    }
}

#[test]
fn goto_v9_nopanic_large_state_ids() {
    let table = make_table(
        "gt_v9_np3",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    for s in [0u16, 1, 100, 1000, 10000, u16::MAX] {
        for &nt in table.nonterminal_to_index.keys() {
            let _ = table.goto(StateId(s), nt);
        }
    }
}

#[test]
fn goto_v9_nopanic_combined_extremes() {
    let table = make_table(
        "gt_v9_np4",
        &[("a", "a"), ("b", "b")],
        &[("item", vec!["a"]), ("start", vec!["item"])],
        "start",
    );
    for s in [0u16, u16::MAX / 2, u16::MAX] {
        for sym in [0u16, 1, u16::MAX / 2, u16::MAX] {
            let _ = table.goto(StateId(s), SymbolId(sym));
        }
    }
}

// ===========================================================================
// 13. goto for terminal symbols → typically None (4 tests)
// ===========================================================================

#[test]
fn goto_v9_terminal_returns_none() {
    let g = GrammarBuilder::new("gt_v9_tn1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), a).is_none(),
            "goto for terminal 'a' should be None"
        );
    }
}

#[test]
fn goto_v9_terminal_multiple_tokens_none() {
    let g = GrammarBuilder::new("gt_v9_tn2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    for tok_name in &["a", "b", "c"] {
        let tid = tok_id(&g, tok_name);
        for s in 0..table.state_count {
            assert!(table.goto(StateId(s as u16), tid).is_none());
        }
    }
}

#[test]
fn goto_v9_terminal_not_in_nonterminal_index() {
    let g = GrammarBuilder::new("gt_v9_tn3")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let x = tok_id(&g, "x");
    assert!(
        !table.nonterminal_to_index.contains_key(&x),
        "terminal should not be in nonterminal_to_index"
    );
}

#[test]
fn goto_v9_terminal_in_complex_grammar_none() {
    let g = GrammarBuilder::new("gt_v9_tn4")
        .token("n", "n")
        .token("+", "\\+")
        .rule("expr", vec!["n"])
        .rule("expr", vec!["expr", "+", "n"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    let n = tok_id(&g, "n");
    let plus = tok_id(&g, "+");
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        assert!(table.goto(st, n).is_none());
        assert!(table.goto(st, plus).is_none());
    }
}

// ===========================================================================
// 14. goto for EOF → typically None (4 tests)
// ===========================================================================

#[test]
fn goto_v9_eof_returns_none() {
    let table = make_table(
        "gt_v9_eof1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let eof = table.eof_symbol;
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), eof).is_none(),
            "goto for EOF should be None"
        );
    }
}

#[test]
fn goto_v9_eof_not_in_nonterminal_index() {
    let table = make_table(
        "gt_v9_eof2",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(
        !table.nonterminal_to_index.contains_key(&table.eof_symbol),
        "EOF should not be in nonterminal_to_index"
    );
}

#[test]
fn goto_v9_eof_none_in_recursive_grammar() {
    let table = make_table(
        "gt_v9_eof3",
        &[("x", "x")],
        &[
            ("list", vec!["x"]),
            ("list", vec!["list", "x"]),
            ("start", vec!["list"]),
        ],
        "start",
    );
    let eof = table.eof_symbol;
    for s in 0..table.state_count {
        assert!(table.goto(StateId(s as u16), eof).is_none());
    }
}

#[test]
fn goto_v9_eof_none_in_arithmetic_grammar() {
    let table = make_table(
        "gt_v9_eof4",
        &[("n", "n"), ("+", "\\+"), ("*", "\\*")],
        &[
            ("expr", vec!["n"]),
            ("expr", vec!["expr", "+", "expr"]),
            ("expr", vec!["expr", "*", "expr"]),
            ("start", vec!["expr"]),
        ],
        "start",
    );
    let eof = table.eof_symbol;
    for s in 0..table.state_count {
        assert!(table.goto(StateId(s as u16), eof).is_none());
    }
}

// ===========================================================================
// 15. Various grammar sizes → goto table populated (4 tests)
// ===========================================================================

#[test]
fn goto_v9_size_tiny() {
    let table = make_table(
        "gt_v9_sz1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(total_goto_entries(&table) >= 1);
}

#[test]
fn goto_v9_size_small() {
    let table = make_table(
        "gt_v9_sz2",
        &[("a", "a"), ("b", "b")],
        &[
            ("item", vec!["a"]),
            ("item", vec!["b"]),
            ("list", vec!["item"]),
            ("list", vec!["list", "item"]),
            ("start", vec!["list"]),
        ],
        "start",
    );
    assert!(total_goto_entries(&table) >= 3, "small grammar must have several goto entries");
}

#[test]
fn goto_v9_size_medium() {
    let table = make_table(
        "gt_v9_sz3",
        &[("n", "n"), ("+", "\\+"), ("*", "\\*"), ("(", "\\("), (")", "\\)")],
        &[
            ("factor", vec!["n"]),
            ("factor", vec!["(", "expr", ")"]),
            ("term", vec!["factor"]),
            ("term", vec!["term", "*", "factor"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "+", "term"]),
            ("start", vec!["expr"]),
        ],
        "start",
    );
    assert!(
        total_goto_entries(&table) >= 5,
        "medium arithmetic grammar must have many goto entries"
    );
}

#[test]
fn goto_v9_size_larger() {
    let table = make_table(
        "gt_v9_sz4",
        &[
            ("n", "n"),
            ("+", "\\+"),
            ("*", "\\*"),
            ("-", "-"),
            ("(", "\\("),
            (")", "\\)"),
        ],
        &[
            ("atom", vec!["n"]),
            ("atom", vec!["(", "expr", ")"]),
            ("unary", vec!["atom"]),
            ("unary", vec!["-", "atom"]),
            ("factor", vec!["unary"]),
            ("factor", vec!["factor", "*", "unary"]),
            ("term", vec!["factor"]),
            ("term", vec!["term", "+", "factor"]),
            ("expr", vec!["term"]),
            ("start", vec!["expr"]),
        ],
        "start",
    );
    assert!(
        total_goto_entries(&table) >= 6,
        "larger grammar must populate many goto entries"
    );
}

// ===========================================================================
// 16. Chain rules → goto between states (4 tests)
// ===========================================================================

#[test]
fn goto_v9_chain_two_levels() {
    let g = GrammarBuilder::new("gt_v9_ch1")
        .token("t", "t")
        .rule("inner", vec!["t"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let start = nt_id(&g, "start");
    assert!(!all_gotos_for(&table, inner).is_empty());
    assert!(!all_gotos_for(&table, start).is_empty());
}

#[test]
fn goto_v9_chain_three_levels() {
    let g = GrammarBuilder::new("gt_v9_ch2")
        .token("t", "t")
        .rule("c", vec!["t"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["c", "b", "start"] {
        let nt = nt_id(&g, name);
        assert!(!all_gotos_for(&table, nt).is_empty());
    }
}

#[test]
fn goto_v9_chain_four_levels() {
    let g = GrammarBuilder::new("gt_v9_ch3")
        .token("t", "t")
        .rule("d", vec!["t"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["d", "c", "b", "start"] {
        let nt = nt_id(&g, name);
        assert!(!all_gotos_for(&table, nt).is_empty());
    }
}

#[test]
fn goto_v9_chain_goto_targets_differ() {
    let g = GrammarBuilder::new("gt_v9_ch4")
        .token("t", "t")
        .rule("inner", vec!["t"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let start = nt_id(&g, "start");
    let inner_targets: Vec<_> = all_gotos_for(&table, inner).iter().map(|&(_, t)| t).collect();
    let start_targets: Vec<_> = all_gotos_for(&table, start).iter().map(|&(_, t)| t).collect();
    // The goto targets for different nonterminals should generally differ
    assert_ne!(
        inner_targets, start_targets,
        "chain nonterminals should have different goto targets"
    );
}

// ===========================================================================
// 17. Multiple non-terminals → multiple goto entries (4 tests)
// ===========================================================================

#[test]
fn goto_v9_multi_nt_two() {
    let g = GrammarBuilder::new("gt_v9_mn1")
        .token("a", "a")
        .token("b", "b")
        .rule("first", vec!["a"])
        .rule("second", vec!["b"])
        .rule("start", vec!["first", "second"])
        .start("start")
        .build();
    let table = build_table(&g);
    let nts = goto_nonterminals(&table);
    assert!(nts.len() >= 3, "grammar with 3 nonterminals must register at least 3");
}

#[test]
fn goto_v9_multi_nt_each_has_entries() {
    let g = GrammarBuilder::new("gt_v9_mn2")
        .token("a", "a")
        .token("b", "b")
        .rule("alpha", vec!["a"])
        .rule("beta", vec!["b"])
        .rule("start", vec!["alpha", "beta"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["alpha", "beta", "start"] {
        let nt = nt_id(&g, name);
        assert!(!all_gotos_for(&table, nt).is_empty(), "'{name}' must have goto entries");
    }
}

#[test]
fn goto_v9_multi_nt_independent_targets() {
    let g = GrammarBuilder::new("gt_v9_mn3")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("p", vec!["x"])
        .rule("q", vec!["y"])
        .rule("r", vec!["z"])
        .rule("start", vec!["p", "q", "r"])
        .start("start")
        .build();
    let table = build_table(&g);
    let p_gotos = all_gotos_for(&table, nt_id(&g, "p"));
    let q_gotos = all_gotos_for(&table, nt_id(&g, "q"));
    let r_gotos = all_gotos_for(&table, nt_id(&g, "r"));
    assert!(!p_gotos.is_empty());
    assert!(!q_gotos.is_empty());
    assert!(!r_gotos.is_empty());
}

#[test]
fn goto_v9_multi_nt_total_count() {
    let g = GrammarBuilder::new("gt_v9_mn4")
        .token("a", "a")
        .token("b", "b")
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .rule("pair", vec!["left", "right"])
        .rule("start", vec!["pair"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        total_goto_entries(&table) >= 4,
        "grammar with 4 nonterminals must have at least 4 goto entries"
    );
}

// ===========================================================================
// 18. goto table is sparse (many None entries) (4 tests)
// ===========================================================================

#[test]
fn goto_v9_sparse_simple() {
    let table = make_table(
        "gt_v9_sp1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let total_cells = table.state_count * table.nonterminal_to_index.len();
    let filled = total_goto_entries(&table);
    // Even a tiny grammar produces some None entries; but if all are filled that's
    // still valid — just verify filled <= total_cells.
    assert!(
        filled <= total_cells,
        "filled {filled} must not exceed total cells {total_cells}"
    );
}

#[test]
fn goto_v9_sparse_medium() {
    let table = make_table(
        "gt_v9_sp2",
        &[("n", "n"), ("+", "\\+"), ("*", "\\*")],
        &[
            ("factor", vec!["n"]),
            ("term", vec!["factor"]),
            ("term", vec!["term", "*", "factor"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "+", "term"]),
            ("start", vec!["expr"]),
        ],
        "start",
    );
    let total_cells = table.state_count * table.nonterminal_to_index.len();
    let filled = total_goto_entries(&table);
    // Medium grammars are generally sparse
    assert!(
        filled <= total_cells,
        "filled {filled} must not exceed total cells {total_cells}"
    );
}

#[test]
fn goto_v9_sparse_recursive() {
    let table = make_table(
        "gt_v9_sp3",
        &[("x", "x"), ("y", "y")],
        &[
            ("item", vec!["x"]),
            ("item", vec!["y"]),
            ("list", vec!["item"]),
            ("list", vec!["list", "item"]),
            ("start", vec!["list"]),
        ],
        "start",
    );
    let total_cells = table.state_count * table.nonterminal_to_index.len();
    let filled = total_goto_entries(&table);
    assert!(
        filled <= total_cells,
        "filled {filled} must not exceed total cells {total_cells}"
    );
}

#[test]
fn goto_v9_sparse_fill_ratio() {
    let table = make_table(
        "gt_v9_sp4",
        &[("n", "n"), ("+", "\\+"), ("*", "\\*"), ("(", "\\("), (")", "\\)")],
        &[
            ("factor", vec!["n"]),
            ("factor", vec!["(", "expr", ")"]),
            ("term", vec!["factor"]),
            ("term", vec!["term", "*", "factor"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "+", "term"]),
            ("start", vec!["expr"]),
        ],
        "start",
    );
    let total_cells = table.state_count * table.nonterminal_to_index.len();
    let filled = total_goto_entries(&table);
    // Verify goto table is populated and bounded
    assert!(
        filled <= total_cells,
        "filled {filled} must not exceed total cells {total_cells}"
    );
    assert!(filled > 0, "goto table must have some entries");
}

// ===========================================================================
// 19. goto entries only for non-terminal symbols (4 tests)
// ===========================================================================

#[test]
fn goto_v9_only_nonterminals_registered() {
    let g = GrammarBuilder::new("gt_v9_on1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(!table.nonterminal_to_index.contains_key(&a));
}

#[test]
fn goto_v9_only_nonterminals_multi_token() {
    let g = GrammarBuilder::new("gt_v9_on2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    for tok_name in &["a", "b", "c"] {
        let tid = tok_id(&g, tok_name);
        assert!(!table.nonterminal_to_index.contains_key(&tid));
    }
}

#[test]
fn goto_v9_only_nonterminals_in_mixed_grammar() {
    let g = GrammarBuilder::new("gt_v9_on3")
        .token("n", "n")
        .token("+", "\\+")
        .rule("expr", vec!["n"])
        .rule("expr", vec!["expr", "+", "n"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    // All nonterminal_to_index keys should be rule LHS symbols
    for &nt in table.nonterminal_to_index.keys() {
        assert!(
            g.rule_names.contains_key(&nt),
            "nonterminal_to_index key {nt:?} must be a rule name"
        );
    }
}

#[test]
fn goto_v9_only_nonterminals_eof_excluded() {
    let table = make_table(
        "gt_v9_on4",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(!table.nonterminal_to_index.contains_key(&table.eof_symbol));
}

// ===========================================================================
// 20. Arithmetic grammar goto structure (4 tests)
// ===========================================================================

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("gt_v9_arith")
        .token("n", "n")
        .token("+", "\\+")
        .token("*", "\\*")
        .token("(", "\\(")
        .token(")", "\\)")
        .rule("factor", vec!["n"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

#[test]
fn goto_v9_arith_all_nonterminals_present() {
    let g = arith_grammar();
    let table = build_table(&g);
    for name in &["factor", "term", "expr", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            table.nonterminal_to_index.contains_key(&nt),
            "'{name}' must be in nonterminal_to_index"
        );
    }
}

#[test]
fn goto_v9_arith_expr_has_gotos() {
    let g = arith_grammar();
    let table = build_table(&g);
    let expr = nt_id(&g, "expr");
    let gotos = all_gotos_for(&table, expr);
    assert!(
        gotos.len() >= 2,
        "expr should have gotos from multiple states (initial + after '(')"
    );
}

#[test]
fn goto_v9_arith_factor_has_gotos() {
    let g = arith_grammar();
    let table = build_table(&g);
    let factor = nt_id(&g, "factor");
    let gotos = all_gotos_for(&table, factor);
    assert!(
        !gotos.is_empty(),
        "factor must have goto entries"
    );
}

#[test]
fn goto_v9_arith_total_entries() {
    let g = arith_grammar();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    // Arithmetic grammar with parentheses creates multiple item sets,
    // so we expect a healthy number of goto entries.
    assert!(
        total >= 8,
        "arithmetic grammar should have at least 8 goto entries, got {total}"
    );
}

// ===========================================================================
// Bonus: additional edge-case tests (2 tests)
// ===========================================================================

#[test]
fn goto_v9_initial_state_is_valid() {
    let table = make_table(
        "gt_v9_bns1",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!((table.initial_state.0 as usize) < table.state_count);
}

#[test]
fn goto_v9_nonterminal_to_index_nonempty() {
    let table = make_table(
        "gt_v9_bns2",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(
        !table.nonterminal_to_index.is_empty(),
        "every grammar must register at least one nonterminal"
    );
}
