#![cfg(feature = "test-api")]
#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for closure and goto computation properties,
//! verified through the `ParseTable` interface.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test closure_goto_v2_comprehensive --features test-api

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

/// Check if any state has a shift action on the given terminal.
fn has_shift_on(table: &ParseTable, sym: SymbolId) -> bool {
    (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

/// Check if any state has a reduce action.
fn has_any_reduce(table: &ParseTable) -> bool {
    (0..table.state_count).any(|s| {
        let st = StateId(s as u16);
        table.symbol_to_index.keys().any(|&sym| {
            table
                .actions(st, sym)
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
    })
}

/// Check if accept is reachable in the table.
fn has_accept(table: &ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

/// Collect all shift targets from a given state.
fn shift_targets(table: &ParseTable, state: StateId) -> Vec<StateId> {
    let mut targets = Vec::new();
    for &sym in table.symbol_to_index.keys() {
        for action in table.actions(state, sym) {
            if let Action::Shift(tgt) = action {
                targets.push(*tgt);
            }
        }
    }
    targets
}

/// Collect all goto targets from a given state.
fn goto_targets(table: &ParseTable, state: StateId) -> Vec<StateId> {
    table
        .nonterminal_to_index
        .keys()
        .filter_map(|&nt| table.goto(state, nt))
        .collect()
}

// ===========================================================================
// 1. Initial state has shift actions (10 tests)
// ===========================================================================

#[test]
fn initial_state_has_shift_single_token() {
    let g = GrammarBuilder::new("t1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state must shift on 'a'"
    );
}

#[test]
fn initial_state_shift_two_tokens() {
    let g = GrammarBuilder::new("t2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state must shift on first token 'a'"
    );
}

#[test]
fn initial_state_shift_alternative_rules() {
    let g = GrammarBuilder::new("t3")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .start("start")
        .build();
    let table = build_table(&g);
    let x = tok_id(&g, "x");
    let y = tok_id(&g, "y");
    assert!(
        table
            .actions(table.initial_state, x)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "initial state must shift on 'x'"
    );
    assert!(
        table
            .actions(table.initial_state, y)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "initial state must shift on 'y'"
    );
}

#[test]
fn initial_state_shift_with_nonterminal_prefix() {
    // start -> inner; inner -> a
    let g = GrammarBuilder::new("t4")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    // Closure of initial state expands start -> .inner -> .a, so 'a' should have shift
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "initial state closure must propagate shift on 'a' through nonterminal"
    );
}

#[test]
fn initial_state_shift_nested_nonterminals() {
    // start -> mid; mid -> inner; inner -> a
    let g = GrammarBuilder::new("t5")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("mid", vec!["inner"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "double nesting must still propagate shift"
    );
}

#[test]
fn initial_state_shift_count_matches_reachable_terminals() {
    // start -> a | b | c
    let g = GrammarBuilder::new("t6")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let shifts: Vec<_> = ["a", "b", "c"]
        .iter()
        .filter(|&&name| {
            let sym = tok_id(&g, name);
            table
                .actions(table.initial_state, sym)
                .iter()
                .any(|a| matches!(a, Action::Shift(_)))
        })
        .collect();
    assert_eq!(
        shifts.len(),
        3,
        "all three tokens must be shiftable from initial state"
    );
}

#[test]
fn initial_state_shift_target_is_valid_state() {
    let g = GrammarBuilder::new("t7")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let targets = shift_targets(&table, table.initial_state);
    for tgt in &targets {
        assert!(
            (tgt.0 as usize) < table.state_count,
            "shift target {tgt:?} out of range"
        );
    }
}

#[test]
fn initial_state_shift_targets_distinct_for_different_tokens() {
    let g = GrammarBuilder::new("t8")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["b", "a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let tgt_a: Vec<StateId> = table
        .actions(table.initial_state, a)
        .iter()
        .filter_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    let tgt_b: Vec<StateId> = table
        .actions(table.initial_state, b)
        .iter()
        .filter_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    // Both should exist, and differ in at least one element
    assert!(!tgt_a.is_empty(), "shift on 'a' must exist");
    assert!(!tgt_b.is_empty(), "shift on 'b' must exist");
    assert_ne!(
        tgt_a, tgt_b,
        "distinct tokens should shift to distinct states"
    );
}

#[test]
fn initial_state_no_shift_on_unreachable_token() {
    // start -> a, token 'z' defined but not in any rule
    let g = GrammarBuilder::new("t9")
        .token("a", "a")
        .token("z", "z")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let z = tok_id(&g, "z");
    let actions = table.actions(table.initial_state, z);
    assert!(
        !actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "unreachable token 'z' must not have a shift in initial state"
    );
}

#[test]
fn initial_state_has_shift_with_recursive_rule() {
    // start -> a | start a (left-recursive)
    let g = GrammarBuilder::new("t10")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "recursive grammar initial state must still shift on 'a'"
    );
}

// ===========================================================================
// 2. Goto targets are valid states (8 tests)
// ===========================================================================

#[test]
fn goto_target_valid_simple() {
    let g = GrammarBuilder::new("g1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    if let Some(tgt) = table.goto(table.initial_state, s) {
        assert!((tgt.0 as usize) < table.state_count);
    }
}

#[test]
fn goto_targets_valid_all_states() {
    let g = GrammarBuilder::new("g2")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert!(
                    (tgt.0 as usize) < table.state_count,
                    "goto({s}, {nt:?}) = {tgt:?} out of range"
                );
            }
        }
    }
}

#[test]
fn goto_from_initial_for_start_exists() {
    let g = GrammarBuilder::new("g3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    assert!(
        table.goto(table.initial_state, s).is_some(),
        "goto(initial, start) must exist"
    );
}

#[test]
fn goto_intermediate_nt_exists() {
    let g = GrammarBuilder::new("g4")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let table = build_table(&g);
    let leaf = nt_id(&g, "leaf");
    let exists = (0..table.state_count).any(|s| table.goto(StateId(s as u16), leaf).is_some());
    assert!(
        exists,
        "goto for intermediate nonterminal 'leaf' must exist"
    );
}

#[test]
fn goto_multi_nonterminal_all_valid() {
    let g = GrammarBuilder::new("g5")
        .token("a", "a")
        .token("b", "b")
        .rule("first", vec!["a"])
        .rule("second", vec!["b"])
        .rule("start", vec!["first", "second"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for s in 0..table.state_count {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!((tgt.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_undefined_returns_none() {
    let g = GrammarBuilder::new("g6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // A bogus nonterminal that doesn't exist
    let bogus = SymbolId(9999);
    assert!(
        table.goto(table.initial_state, bogus).is_none(),
        "goto with unknown nonterminal must be None"
    );
}

#[test]
fn goto_out_of_range_state_returns_none() {
    let g = GrammarBuilder::new("g7")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    let far = StateId(table.state_count as u16 + 100);
    assert!(
        table.goto(far, s).is_none(),
        "goto with out-of-range state must be None"
    );
}

#[test]
fn goto_chain_of_nonterminals() {
    // start -> mid; mid -> leaf; leaf -> a
    let g = GrammarBuilder::new("g8")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Each nonterminal should have at least one goto defined
    for name in &["start", "mid", "leaf"] {
        let nt = nt_id(&g, name);
        let exists = (0..table.state_count).any(|s| table.goto(StateId(s as u16), nt).is_some());
        assert!(exists, "goto must exist for nonterminal '{name}'");
    }
}

// ===========================================================================
// 3. Reduction states have reduce actions (8 tests)
// ===========================================================================

#[test]
fn reduce_exists_simple_grammar() {
    let g = GrammarBuilder::new("r1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        has_any_reduce(&table),
        "simple grammar must have reduce actions"
    );
}

#[test]
fn reduce_on_eof_after_single_token() {
    let g = GrammarBuilder::new("r2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    // After shifting 'a', the resulting state should have a reduce
    let shift_state = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("must have shift on 'a'");
    let eof = table.eof();
    let has_reduce = table
        .actions(shift_state, eof)
        .iter()
        .any(|a| matches!(a, Action::Reduce(_)));
    assert!(has_reduce, "after shifting 'a', must reduce on EOF");
}

#[test]
fn reduce_action_references_valid_rule() {
    let g = GrammarBuilder::new("r3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(st, sym) {
                if let Action::Reduce(rule_id) = action {
                    assert!(
                        (rule_id.0 as usize) < table.rules.len(),
                        "reduce rule {rule_id:?} out of range"
                    );
                }
            }
        }
    }
}

#[test]
fn reduce_lhs_is_known_nonterminal() {
    let g = GrammarBuilder::new("r4")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for rule in &table.rules {
        assert!(
            table.nonterminal_to_index.contains_key(&rule.lhs),
            "reduce rule LHS {:?} must be in nonterminal_to_index",
            rule.lhs
        );
    }
}

#[test]
fn reduce_count_at_least_one_per_grammar_rule() {
    let g = GrammarBuilder::new("r5")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Count distinct reduce rule IDs used in the table
    let mut used_rules = std::collections::BTreeSet::new();
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(st, sym) {
                if let Action::Reduce(rid) = action {
                    used_rules.insert(rid.0);
                }
            }
        }
    }
    // The table has augmented rules, but original rules should all be represented
    assert!(
        used_rules.len() >= 2,
        "at least 2 distinct reduce rules expected, got {}",
        used_rules.len()
    );
}

#[test]
fn reduce_for_multi_symbol_rule() {
    // start -> a b
    let g = GrammarBuilder::new("r6")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // There must be a rule with rhs_len == 2
    let has_len2 = table.rules.iter().any(|r| r.rhs_len == 2);
    assert!(has_len2, "must have a rule with rhs_len == 2");
    assert!(has_any_reduce(&table));
}

#[test]
fn reduce_epsilon_rule() {
    let g = GrammarBuilder::new("r7")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Epsilon rules: GrammarBuilder produces `rhs = [Symbol::Epsilon]` (len 1),
    // so rhs_len may be 0 or 1 depending on normalization.
    let has_eps = table.rules.iter().any(|r| r.rhs_len <= 1);
    assert!(has_eps, "epsilon rule must produce rhs_len <= 1");
    assert!(has_any_reduce(&table));
}

#[test]
fn reduce_recursive_grammar() {
    // start -> a | start a
    let g = GrammarBuilder::new("r8")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        has_any_reduce(&table),
        "recursive grammar must have reduce actions"
    );
}

// ===========================================================================
// 4. Accept state reachable (5 tests)
// ===========================================================================

#[test]
fn accept_reachable_simple() {
    let g = GrammarBuilder::new("a1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table), "simple grammar must reach accept");
}

#[test]
fn accept_reachable_multi_rule() {
    let g = GrammarBuilder::new("a2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table), "multi-rule grammar must reach accept");
}

#[test]
fn accept_reachable_nested() {
    let g = GrammarBuilder::new("a3")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table), "nested grammar must reach accept");
}

#[test]
fn accept_only_on_eof() {
    let g = GrammarBuilder::new("a4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            let actions = table.actions(st, sym);
            if actions.iter().any(|a| matches!(a, Action::Accept)) {
                assert_eq!(
                    sym, eof,
                    "Accept must only appear on EOF symbol, but found on {sym:?}"
                );
            }
        }
    }
}

#[test]
fn accept_state_via_goto_of_start() {
    let g = GrammarBuilder::new("a5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    let accept_st = table.goto(table.initial_state, s);
    assert!(accept_st.is_some(), "goto(initial, start) must exist");
    let eof = table.eof();
    let actions = table.actions(accept_st.unwrap(), eof);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Accept)),
        "goto(initial, start) state must accept on EOF"
    );
}

// ===========================================================================
// 5. No self-loops in goto (5 tests)
// ===========================================================================

#[test]
fn no_goto_self_loop_simple() {
    let g = GrammarBuilder::new("sl1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert_ne!(st, tgt, "goto({s}, {nt:?}) is a self-loop");
            }
        }
    }
}

#[test]
fn no_goto_self_loop_two_rules() {
    let g = GrammarBuilder::new("sl2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert_ne!(st, tgt, "goto self-loop in state {s}");
            }
        }
    }
}

#[test]
fn no_goto_self_loop_nested() {
    let g = GrammarBuilder::new("sl3")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert_ne!(st, tgt, "self-loop in nested grammar state {s}");
            }
        }
    }
}

#[test]
fn no_goto_self_loop_chain() {
    let g = GrammarBuilder::new("sl4")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert_ne!(st, tgt, "self-loop in chain grammar state {s}");
            }
        }
    }
}

#[test]
fn no_goto_self_loop_recursive() {
    // start -> a | start a
    let g = GrammarBuilder::new("sl5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert_ne!(st, tgt, "self-loop in recursive grammar state {s}");
            }
        }
    }
}

// ===========================================================================
// 6. Action/goto consistency (8 tests)
// ===========================================================================

#[test]
fn shift_targets_have_actions_or_gotos() {
    let g = GrammarBuilder::new("c1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for tgt in shift_targets(&table, st) {
            let has_action = table
                .symbol_to_index
                .keys()
                .any(|&sym| !table.actions(tgt, sym).is_empty());
            let has_goto = table
                .nonterminal_to_index
                .keys()
                .any(|&nt| table.goto(tgt, nt).is_some());
            assert!(
                has_action || has_goto,
                "shift target {tgt:?} from state {s} has no actions or gotos"
            );
        }
    }
}

#[test]
fn goto_targets_have_actions_or_gotos() {
    let g = GrammarBuilder::new("c2")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for tgt in goto_targets(&table, st) {
            let has_action = table
                .symbol_to_index
                .keys()
                .any(|&sym| !table.actions(tgt, sym).is_empty());
            let has_goto = table
                .nonterminal_to_index
                .keys()
                .any(|&nt| table.goto(tgt, nt).is_some());
            assert!(
                has_action || has_goto,
                "goto target {tgt:?} from state {s} has no actions or gotos"
            );
        }
    }
}

#[test]
fn reduce_rule_lhs_in_goto_table() {
    let g = GrammarBuilder::new("c3")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for rule in &table.rules {
        assert!(
            table.nonterminal_to_index.contains_key(&rule.lhs),
            "reduce LHS {:?} must be in goto columns",
            rule.lhs
        );
    }
}

#[test]
fn state_count_positive() {
    let g = GrammarBuilder::new("c4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0, "must have at least one state");
}

#[test]
fn action_table_rows_equal_state_count() {
    let g = GrammarBuilder::new("c5")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        table.action_table.len(),
        table.state_count,
        "action table row count must equal state_count"
    );
}

#[test]
fn goto_table_rows_equal_state_count() {
    let g = GrammarBuilder::new("c6")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto table row count must equal state_count"
    );
}

#[test]
fn eof_symbol_in_symbol_to_index() {
    let g = GrammarBuilder::new("c7")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.symbol_to_index.contains_key(&table.eof()),
        "EOF must be in symbol_to_index"
    );
}

#[test]
fn start_symbol_in_nonterminal_to_index() {
    let g = GrammarBuilder::new("c8")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(
        table.nonterminal_to_index.contains_key(&start),
        "start symbol must be in nonterminal_to_index"
    );
}

// ===========================================================================
// 7. Various grammar topologies (6 tests)
// ===========================================================================

#[test]
fn topology_linear_chain() {
    // start -> a b c
    let g = GrammarBuilder::new("top1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // Should have at least 4 states: initial, after-a, after-b, after-c, accept
    assert!(table.state_count >= 4, "linear chain needs multiple states");
}

#[test]
fn topology_branching() {
    // start -> a | b | c
    let g = GrammarBuilder::new("top2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let c = tok_id(&g, "c");
    assert!(has_shift_on(&table, a));
    assert!(has_shift_on(&table, b));
    assert!(has_shift_on(&table, c));
}

#[test]
fn topology_left_recursive() {
    // list -> item | list item; item -> a
    let g = GrammarBuilder::new("top3")
        .token("a", "a")
        .rule("item", vec!["a"])
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "item"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(has_any_reduce(&table));
}

#[test]
fn topology_right_recursive() {
    // list -> item | item list; item -> a
    let g = GrammarBuilder::new("top4")
        .token("a", "a")
        .rule("item", vec!["a"])
        .rule("list", vec!["item"])
        .rule("list", vec!["item", "list"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(has_any_reduce(&table));
}

#[test]
fn topology_diamond() {
    // start -> left | right; left -> a; right -> a
    let g = GrammarBuilder::new("top5")
        .token("a", "a")
        .rule("left", vec!["a"])
        .rule("right", vec!["a"])
        .rule("start", vec!["left"])
        .rule("start", vec!["right"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn topology_wide_fanout() {
    // start -> a | b | c | d | e
    let g = GrammarBuilder::new("top6")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .rule("start", vec!["e"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // All 5 tokens must be shiftable from initial
    for name in &["a", "b", "c", "d", "e"] {
        let sym = tok_id(&g, name);
        assert!(
            table
                .actions(table.initial_state, sym)
                .iter()
                .any(|a| matches!(a, Action::Shift(_))),
            "token '{name}' must be shiftable from initial"
        );
    }
}

// ===========================================================================
// 8. Edge cases (5 tests)
// ===========================================================================

#[test]
fn edge_case_epsilon_only() {
    // start -> ε
    let g = GrammarBuilder::new("e1")
        .token("a", "a") // at least one token for a well-formed grammar
        .rule("start", vec![])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table), "epsilon-only grammar must reach accept");
}

#[test]
fn edge_case_single_token_grammar() {
    let g = GrammarBuilder::new("e2")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2, "must have at least 2 states");
    assert!(has_accept(&table));
    assert!(has_any_reduce(&table));
}

#[test]
fn edge_case_long_rule() {
    // start -> a b c d e f
    let g = GrammarBuilder::new("e3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("start", vec!["a", "b", "c", "d", "e", "f"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // The rule with 6 symbols should exist
    let has_6 = table.rules.iter().any(|r| r.rhs_len == 6);
    assert!(has_6, "long rule with rhs_len == 6 must exist");
}

#[test]
fn edge_case_multiple_epsilon_alternatives() {
    // start -> ε | a
    let g = GrammarBuilder::new("e4")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // Both paths must work: epsilon reduce and shift on 'a'
    let a = tok_id(&g, "a");
    assert!(has_shift_on(&table, a), "must be able to shift 'a'");
    assert!(has_any_reduce(&table), "must have reduce for epsilon");
}

#[test]
fn edge_case_deeply_nested() {
    // start -> l1; l1 -> l2; l2 -> l3; l3 -> a
    let g = GrammarBuilder::new("e5")
        .token("a", "a")
        .rule("l3", vec!["a"])
        .rule("l2", vec!["l3"])
        .rule("l1", vec!["l2"])
        .rule("start", vec!["l1"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // Closure must propagate through 4 levels of nonterminals
    let a = tok_id(&g, "a");
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "deeply nested closure must expose 'a' in initial state"
    );
    // All intermediate nonterminals should have goto entries
    for name in &["l1", "l2", "l3", "start"] {
        let nt = nt_id(&g, name);
        let exists = (0..table.state_count).any(|s| table.goto(StateId(s as u16), nt).is_some());
        assert!(exists, "goto must exist for '{name}'");
    }
}
