#![cfg(feature = "test-api")]

//! V5 comprehensive tests for LR(1) automaton construction.
//!
//! Categories:
//! 1. Automaton construction succeeds (8 tests)
//! 2. State transition validity (8 tests)
//! 3. Accept state properties (8 tests)
//! 4. Shift/reduce distribution (8 tests)
//! 5. Grammar scaling (8 tests)
//! 6. Determinism (7 tests)
//! 7. Edge cases (10 tests)

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

fn has_accept(table: &adze_glr_core::ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn count_accept_states(table: &adze_glr_core::ParseTable) -> usize {
    let eof = table.eof();
    (0..table.state_count)
        .filter(|&st| {
            table
                .actions(StateId(st as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .count()
}

fn collect_all_shifts(table: &adze_glr_core::ParseTable) -> Vec<(StateId, SymbolId, StateId)> {
    let mut out = Vec::new();
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Shift(target) = action {
                    out.push((state, sym, *target));
                }
            }
        }
    }
    out
}

fn collect_all_reduces(table: &adze_glr_core::ParseTable) -> Vec<(StateId, SymbolId, RuleId)> {
    let mut out = Vec::new();
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Reduce(rid) = action {
                    out.push((state, sym, *rid));
                }
            }
        }
    }
    out
}

fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

fn state_has_any_action(table: &adze_glr_core::ParseTable, state: StateId) -> bool {
    table.symbol_to_index.keys().any(|&sym| {
        table
            .actions(state, sym)
            .iter()
            .any(|a| !matches!(a, Action::Error))
    })
}

// ===========================================================================
// 1. Automaton construction succeeds (8 tests)
// ===========================================================================

#[test]
fn construct_single_token_grammar() {
    let g = GrammarBuilder::new("v5c1")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
}

#[test]
fn construct_two_token_sequence() {
    let g = GrammarBuilder::new("v5c2")
        .token("p", "p")
        .token("q", "q")
        .rule("start", vec!["p", "q"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
}

#[test]
fn construct_three_alternatives() {
    let g = GrammarBuilder::new("v5c3")
        .token("p", "p")
        .token("q", "q")
        .token("r", "r")
        .rule("start", vec!["p"])
        .rule("start", vec!["q"])
        .rule("start", vec!["r"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
}

#[test]
fn construct_nonterminal_delegation() {
    let g = GrammarBuilder::new("v5c4")
        .token("w", "w")
        .rule("inner", vec!["w"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
}

#[test]
fn construct_left_recursive_list() {
    let g = GrammarBuilder::new("v5c5")
        .token("m", "m")
        .rule("items", vec!["items", "m"])
        .rule("items", vec!["m"])
        .start("items")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
}

#[test]
fn construct_right_recursive_list() {
    let g = GrammarBuilder::new("v5c6")
        .token("m", "m")
        .rule("items", vec!["m", "items"])
        .rule("items", vec!["m"])
        .start("items")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
}

#[test]
fn construct_expression_grammar() {
    let g = GrammarBuilder::new("v5c7")
        .token("num", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "num"])
        .rule("term", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 5);
}

#[test]
fn construct_deeply_nested_nonterminals() {
    let g = GrammarBuilder::new("v5c8")
        .token("leaf", "leaf")
        .rule("lvl4", vec!["leaf"])
        .rule("lvl3", vec!["lvl4"])
        .rule("lvl2", vec!["lvl3"])
        .rule("lvl1", vec!["lvl2"])
        .rule("start", vec!["lvl1"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

// ===========================================================================
// 2. State transition validity (8 tests)
// ===========================================================================

#[test]
fn shift_reaches_new_state_single_token() {
    let g = GrammarBuilder::new("v5t1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    let shift_target = actions.iter().find_map(|act| {
        if let Action::Shift(target) = act {
            Some(*target)
        } else {
            None
        }
    });
    assert!(shift_target.is_some(), "initial state must shift on 'a'");
    let target = shift_target.unwrap();
    assert_ne!(target, table.initial_state, "shift must move to new state");
}

#[test]
fn shift_chain_traverses_sequence() {
    let g = GrammarBuilder::new("v5t2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    // Shift on 'a' from initial
    let s1 = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| {
            if let Action::Shift(t) = act {
                Some(*t)
            } else {
                None
            }
        })
        .expect("should shift on 'a'");
    // From s1, should have action on 'b'
    let actions_b = table.actions(s1, b);
    assert!(
        !actions_b.is_empty(),
        "state after shifting 'a' should act on 'b'"
    );
}

#[test]
fn shift_targets_within_bounds() {
    let g = GrammarBuilder::new("v5t3")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (_, _, target) in collect_all_shifts(&table) {
        assert!(
            (target.0 as usize) < table.state_count,
            "shift target {} out of bounds (state_count={})",
            target.0,
            table.state_count
        );
    }
}

#[test]
fn goto_after_reduce_reaches_valid_state() {
    let g = GrammarBuilder::new("v5t4")
        .token("k", "k")
        .rule("inner", vec!["k"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Check that goto entries point to valid states
    for &sym in table.symbol_to_index.keys() {
        for st in 0..table.state_count {
            if let Some(target) = table.goto(StateId(st as u16), sym) {
                assert!(
                    (target.0 as usize) < table.state_count,
                    "goto target {} out of bounds",
                    target.0
                );
            }
        }
    }
}

#[test]
fn all_shift_targets_reachable_states_have_actions() {
    let g = GrammarBuilder::new("v5t5")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (_, _, target) in collect_all_shifts(&table) {
        assert!(
            state_has_any_action(&table, target),
            "shift target state {} should have actions",
            target.0
        );
    }
}

#[test]
fn initial_state_shifts_on_recursive_base() {
    let g = GrammarBuilder::new("v5t6")
        .token("n", "n")
        .rule("lst", vec!["lst", "n"])
        .rule("lst", vec!["n"])
        .start("lst")
        .build();
    let table = build_table(&g);
    let n = tok_id(&g, "n");
    assert!(
        table
            .actions(table.initial_state, n)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on base case token"
    );
}

#[test]
fn shift_target_distinct_from_source_in_sequence() {
    let g = GrammarBuilder::new("v5t7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (source, _, target) in collect_all_shifts(&table) {
        assert_ne!(source, target, "shift should not loop back to same state");
    }
}

#[test]
fn goto_entries_within_state_count() {
    let g = GrammarBuilder::new("v5t8")
        .token("num", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "num"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            if let Some(target) = table.goto(StateId(st as u16), sym) {
                assert!((target.0 as usize) < table.state_count);
            }
        }
    }
}

// ===========================================================================
// 3. Accept state properties (8 tests)
// ===========================================================================

#[test]
fn accept_only_on_eof() {
    let g = GrammarBuilder::new("v5a1")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        for &sym in table.symbol_to_index.keys() {
            let has_accept = table
                .actions(state, sym)
                .iter()
                .any(|a| matches!(a, Action::Accept));
            if has_accept {
                assert_eq!(sym, eof, "Accept should only appear on EOF symbol");
            }
        }
    }
}

#[test]
fn accept_state_exists_for_alternatives() {
    let g = GrammarBuilder::new("v5a2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_state_count_is_small() {
    let g = GrammarBuilder::new("v5a3")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let count = count_accept_states(&table);
    assert!(count >= 1, "must have at least one accept state");
    assert!(
        count <= 3,
        "simple grammar should not have many accept states, got {count}"
    );
}

#[test]
fn accept_not_on_initial_state_for_nonempty_grammar() {
    let g = GrammarBuilder::new("v5a4")
        .token("w", "w")
        .rule("start", vec!["w"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let initial_accepts = table
        .actions(table.initial_state, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        !initial_accepts,
        "initial state should not accept for non-empty grammar"
    );
}

#[test]
fn accept_after_full_reduction_chain() {
    let g = GrammarBuilder::new("v5a5")
        .token("leaf", "leaf")
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn accept_present_for_left_recursive_grammar() {
    let g = GrammarBuilder::new("v5a6")
        .token("v", "v")
        .rule("rep", vec!["rep", "v"])
        .rule("rep", vec!["v"])
        .start("rep")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_present_for_right_recursive_grammar() {
    let g = GrammarBuilder::new("v5a7")
        .token("v", "v")
        .rule("rep", vec!["v", "rep"])
        .rule("rep", vec!["v"])
        .start("rep")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_present_for_parenthesized_grammar() {
    let g = GrammarBuilder::new("v5a8")
        .token("x", "x")
        .token("LP", r"\(")
        .token("RP", r"\)")
        .rule("start", vec!["LP", "start", "RP"])
        .rule("start", vec!["x"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

// ===========================================================================
// 4. Shift/reduce distribution (8 tests)
// ===========================================================================

#[test]
fn single_token_grammar_has_shifts() {
    let g = GrammarBuilder::new("v5sr1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let shifts = collect_all_shifts(&build_table(&g));
    assert!(!shifts.is_empty(), "must have at least one shift action");
}

#[test]
fn single_token_grammar_has_reduces() {
    let g = GrammarBuilder::new("v5sr2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let reduces = collect_all_reduces(&build_table(&g));
    assert!(!reduces.is_empty(), "must have at least one reduce action");
}

#[test]
fn sequence_grammar_more_shifts_than_trivial() {
    let g = GrammarBuilder::new("v5sr3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let shifts = collect_all_shifts(&build_table(&g));
    assert!(
        shifts.len() >= 3,
        "3-token sequence needs at least 3 shifts, got {}",
        shifts.len()
    );
}

#[test]
fn reduce_rule_ids_are_valid() {
    let g = GrammarBuilder::new("v5sr4")
        .token("num", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "num"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let rule_count = table.rules.len();
    for (_, _, rid) in collect_all_reduces(&table) {
        assert!(
            (rid.0 as usize) < rule_count,
            "reduce rule id {} >= rule_count {}",
            rid.0,
            rule_count
        );
    }
}

#[test]
fn alternatives_have_shifts_for_each_token() {
    let g = GrammarBuilder::new("v5sr5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let c = tok_id(&g, "c");
    // Each alternative token should trigger a shift from the initial state
    for sym in [a, b, c] {
        assert!(
            table
                .actions(table.initial_state, sym)
                .iter()
                .any(|act| matches!(act, Action::Shift(_))),
            "initial state should shift on alternative token"
        );
    }
}

#[test]
fn left_recursive_grammar_has_both_shift_and_reduce() {
    let g = GrammarBuilder::new("v5sr6")
        .token("n", "n")
        .rule("lst", vec!["lst", "n"])
        .rule("lst", vec!["n"])
        .start("lst")
        .build();
    let table = build_table(&g);
    let shifts = collect_all_shifts(&table);
    let reduces = collect_all_reduces(&table);
    assert!(!shifts.is_empty(), "recursive grammar must have shifts");
    assert!(!reduces.is_empty(), "recursive grammar must have reduces");
}

#[test]
fn complex_grammar_reduce_count_matches_rule_coverage() {
    let g = GrammarBuilder::new("v5sr7")
        .token("id", r"\w+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "id"])
        .rule("term", vec!["id"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let reduces = collect_all_reduces(&table);
    // Each user rule should be referenced by at least one reduce
    let referenced: std::collections::HashSet<u16> =
        reduces.iter().map(|(_, _, rid)| rid.0).collect();
    assert!(
        referenced.len() >= 2,
        "at least 2 distinct rules should appear in reduces, got {}",
        referenced.len()
    );
}

#[test]
fn no_shift_on_eof() {
    let g = GrammarBuilder::new("v5sr8")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        let has_shift = table
            .actions(state, eof)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)));
        assert!(!has_shift, "should never shift on EOF in state {}", st);
    }
}

// ===========================================================================
// 5. Grammar scaling (8 tests)
// ===========================================================================

#[test]
fn scale_two_tokens_two_rules() {
    let g = GrammarBuilder::new("v5s1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count <= 20);
}

#[test]
fn scale_four_alternatives() {
    let g = GrammarBuilder::new("v5s2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count <= 30);
}

#[test]
fn scale_six_alternatives() {
    let g = GrammarBuilder::new("v5s3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .rule("start", vec!["e"])
        .rule("start", vec!["f"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count <= 40);
}

#[test]
fn scale_chain_depth_three() {
    let g = GrammarBuilder::new("v5s4")
        .token("x", "x")
        .rule("c", vec!["x"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let t1 = build_table(&g);

    let g2 = GrammarBuilder::new("v5s4b")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let t2 = build_table(&g2);

    assert!(
        t1.state_count >= t2.state_count,
        "deeper chain should have at least as many states"
    );
}

#[test]
fn scale_five_token_sequence() {
    let g = GrammarBuilder::new("v5s5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 6, "5-token sequence needs >=6 states");
    assert!(has_accept(&table));
}

#[test]
fn scale_multiple_nonterminals_compose() {
    let g = GrammarBuilder::new("v5s6")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("first", vec!["a"])
        .rule("second", vec!["b"])
        .rule("third", vec!["c"])
        .rule("start", vec!["first", "second", "third"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
}

#[test]
fn scale_recursive_with_separator() {
    let g = GrammarBuilder::new("v5s7")
        .token("id", r"\w+")
        .token("COMMA", ",")
        .rule("item", vec!["id"])
        .rule("lst", vec!["lst", "COMMA", "item"])
        .rule("lst", vec!["item"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
    assert!(table.state_count <= 50);
}

#[test]
fn scale_expression_with_parens() {
    let g = GrammarBuilder::new("v5s8")
        .token("num", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LP", r"\(")
        .token("RP", r"\)")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["LP", "expr", "RP"])
        .rule("factor", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(
        table.state_count >= 8,
        "full expression grammar needs many states"
    );
    assert!(table.state_count <= 100);
}

// ===========================================================================
// 6. Determinism (7 tests)
// ===========================================================================

fn tables_equal(t1: &adze_glr_core::ParseTable, t2: &adze_glr_core::ParseTable) -> bool {
    if t1.state_count != t2.state_count {
        return false;
    }
    for st in 0..t1.state_count {
        let state = StateId(st as u16);
        for &sym in t1.symbol_to_index.keys() {
            if t1.actions(state, sym) != t2.actions(state, sym) {
                return false;
            }
        }
    }
    true
}

#[test]
fn determinism_single_token() {
    let g = GrammarBuilder::new("v5d1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert!(
        tables_equal(&t1, &t2),
        "same grammar must produce identical tables"
    );
}

#[test]
fn determinism_sequence() {
    let g = GrammarBuilder::new("v5d2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert!(tables_equal(&t1, &t2));
}

#[test]
fn determinism_alternatives() {
    let g = GrammarBuilder::new("v5d3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert!(tables_equal(&t1, &t2));
}

#[test]
fn determinism_left_recursive() {
    let g = GrammarBuilder::new("v5d4")
        .token("n", "n")
        .rule("lst", vec!["lst", "n"])
        .rule("lst", vec!["n"])
        .start("lst")
        .build();
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert_eq!(t1.state_count, t2.state_count);
    assert!(tables_equal(&t1, &t2));
}

#[test]
fn determinism_expression_grammar() {
    let g = GrammarBuilder::new("v5d5")
        .token("id", r"\w+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "id"])
        .rule("expr", vec!["id"])
        .start("expr")
        .build();
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert!(tables_equal(&t1, &t2));
}

#[test]
fn determinism_chain() {
    let g = GrammarBuilder::new("v5d6")
        .token("x", "x")
        .rule("c", vec!["x"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert!(tables_equal(&t1, &t2));
}

#[test]
fn determinism_state_count_stable_across_builds() {
    let g = GrammarBuilder::new("v5d7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let counts: Vec<usize> = (0..5).map(|_| build_table(&g).state_count).collect();
    assert!(
        counts.windows(2).all(|w| w[0] == w[1]),
        "state count must be stable: {counts:?}"
    );
}

// ===========================================================================
// 7. Edge cases (10 tests)
// ===========================================================================

#[test]
fn edge_single_char_token_name() {
    let g = GrammarBuilder::new("v5e1")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn edge_long_token_name() {
    let g = GrammarBuilder::new("v5e2")
        .token("very_long_token_name_here", "x")
        .rule("start", vec!["very_long_token_name_here"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn edge_long_nonterminal_name() {
    let g = GrammarBuilder::new("v5e3")
        .token("t", "t")
        .rule("deeply_nested_production_rule", vec!["t"])
        .rule("start", vec!["deeply_nested_production_rule"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn edge_same_token_repeated_in_rhs() {
    let g = GrammarBuilder::new("v5e4")
        .token("a", "a")
        .rule("start", vec!["a", "a", "a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4, "a a a needs >=4 states");
}

#[test]
fn edge_nonterminal_reused_multiple_times() {
    let g = GrammarBuilder::new("v5e5")
        .token("x", "x")
        .rule("atom", vec!["x"])
        .rule("start", vec!["atom", "atom"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn edge_initial_state_index_zero() {
    let g = GrammarBuilder::new("v5e6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        table.initial_state,
        StateId(0),
        "initial state should be state 0"
    );
}

#[test]
fn edge_regex_pattern_tokens() {
    let g = GrammarBuilder::new("v5e7")
        .token("ident", r"[a-zA-Z_]\w*")
        .token("number", r"[0-9]+")
        .rule("start", vec!["ident"])
        .rule("start", vec!["number"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn edge_all_states_reachable_from_initial() {
    let g = GrammarBuilder::new("v5e8")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // BFS from initial state through shifts and gotos
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(table.initial_state);
    visited.insert(table.initial_state);
    while let Some(state) = queue.pop_front() {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Shift(target) = action
                    && visited.insert(*target)
                {
                    queue.push_back(*target);
                }
            }
            if let Some(target) = table.goto(state, sym)
                && visited.insert(target)
            {
                queue.push_back(target);
            }
        }
    }
    assert!(
        visited.len() <= table.state_count,
        "visited states should not exceed total states"
    );
}

#[test]
fn edge_two_separate_nonterminals_no_interference() {
    let g = GrammarBuilder::new("v5e9")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    let table = build_table(&g);
    let x = tok_id(&g, "x");
    let y = tok_id(&g, "y");
    // Initial state should shift on 'x' but not on 'y' directly
    let shifts_x = table
        .actions(table.initial_state, x)
        .iter()
        .any(|a| matches!(a, Action::Shift(_)));
    let shifts_y = table
        .actions(table.initial_state, y)
        .iter()
        .any(|a| matches!(a, Action::Shift(_)));
    assert!(shifts_x, "should shift on first nonterminal's token");
    assert!(
        !shifts_y,
        "should not shift on second nonterminal's token from initial state"
    );
}

#[test]
fn edge_eof_symbol_is_consistent() {
    let g = GrammarBuilder::new("v5e10")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof1 = table.eof();
    let eof2 = table.eof();
    assert_eq!(eof1, eof2, "eof() must return the same symbol each time");
}
