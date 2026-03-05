//! V6 state-machine tests for `build_lr1_automaton` — 64 tests across 8 categories:
//! construction, actions, initial state, accept state, reduce, shift, fork (GLR), deterministic.

#![cfg(feature = "test-api")]
#![allow(clippy::needless_range_loop)]

use adze_glr_core::{
    Action, FirstFollowSets, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use std::collections::{BTreeSet, VecDeque};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn has_accept(pt: &ParseTable) -> bool {
    let eof = pt.eof();
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn reachable_states(pt: &ParseTable) -> BTreeSet<u16> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(pt.initial_state.0);
    visited.insert(pt.initial_state.0);

    while let Some(s) = queue.pop_front() {
        let si = s as usize;
        if si < pt.action_table.len() {
            for col in 0..pt.action_table[si].len() {
                for a in &pt.action_table[si][col] {
                    collect_shift_targets(a, &mut visited, &mut queue);
                }
            }
        }
        if si < pt.goto_table.len() {
            for col in 0..pt.goto_table[si].len() {
                let t = pt.goto_table[si][col].0;
                if t != u16::MAX && visited.insert(t) {
                    queue.push_back(t);
                }
            }
        }
    }
    visited
}

fn collect_shift_targets(a: &Action, visited: &mut BTreeSet<u16>, queue: &mut VecDeque<u16>) {
    match a {
        Action::Shift(t) => {
            if visited.insert(t.0) {
                queue.push_back(t.0);
            }
        }
        Action::Fork(inner) => {
            for ia in inner {
                collect_shift_targets(ia, visited, queue);
            }
        }
        _ => {}
    }
}

fn has_any_shift(pt: &ParseTable, sym: SymbolId) -> bool {
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

fn has_any_reduce(pt: &ParseTable, sym: SymbolId) -> bool {
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Reduce(_)))
    })
}

fn has_any_goto(pt: &ParseTable, ntsym: SymbolId) -> bool {
    (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), ntsym).is_some())
}

fn nonempty_action_cells(pt: &ParseTable) -> usize {
    let mut count = 0;
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            if !pt.action_table[s][col].is_empty() {
                count += 1;
            }
        }
    }
    count
}

fn has_conflict_or_fork(pt: &ParseTable) -> bool {
    (0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    })
}

fn count_reduce_actions(pt: &ParseTable) -> usize {
    let mut n = 0;
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                if matches!(a, Action::Reduce(_)) {
                    n += 1;
                }
            }
        }
    }
    n
}

fn count_shift_actions(pt: &ParseTable) -> usize {
    let mut n = 0;
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                if matches!(a, Action::Shift(_)) {
                    n += 1;
                }
            }
        }
    }
    n
}

// ---------------------------------------------------------------------------
// Grammar factories
// ---------------------------------------------------------------------------

/// start → a
fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// start → a b c
fn linear_grammar() -> Grammar {
    GrammarBuilder::new("linear")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

/// start → a start | a
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// start → start a | a
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("lr")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// expr → expr + expr | num  (ambiguous — triggers GLR)
fn ambiguous_expr_grammar() -> Grammar {
    GrammarBuilder::new("ambig_expr")
        .token("num", "num")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

/// top → inner_a inner_b ; inner_a → a ; inner_b → b
fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("a", "a")
        .token("b", "b")
        .rule("inner_a", vec!["a"])
        .rule("inner_b", vec!["b"])
        .rule("top", vec!["inner_a", "inner_b"])
        .start("top")
        .build()
}

/// start → ( start ) | a
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

/// items → items , id | id
fn list_grammar() -> Grammar {
    GrammarBuilder::new("list")
        .token(",", ",")
        .token("id", "id")
        .rule("items", vec!["items", ",", "id"])
        .rule("items", vec!["id"])
        .start("items")
        .build()
}

/// start → a | b  (two alternatives, single tokens)
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

/// stmt → if expr then stmt | id ; expr → num
fn if_then_grammar() -> Grammar {
    GrammarBuilder::new("if_then")
        .token("if", "if")
        .token("then", "then")
        .token("id", "id")
        .token("num", "num")
        .rule("expr", vec!["num"])
        .rule("stmt", vec!["if", "expr", "then", "stmt"])
        .rule("stmt", vec!["id"])
        .start("stmt")
        .build()
}

/// expr → expr * expr | expr + expr | num  (double ambiguity)
fn double_ambig_grammar() -> Grammar {
    GrammarBuilder::new("double_ambig")
        .token("num", "num")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

/// start → a b | a c  (shared prefix)
fn shared_prefix_grammar() -> Grammar {
    GrammarBuilder::new("shared_prefix")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["a", "c"])
        .start("start")
        .build()
}

/// start → ε | a  (nullable start)
fn nullable_start_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// chain → wrap ; wrap → inner ; inner → x
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("wrap", vec!["inner"])
        .rule("chain", vec!["wrap"])
        .start("chain")
        .build()
}

/// expr → expr + term | term ; term → num  (classic expression)
fn expr_term_grammar() -> Grammar {
    GrammarBuilder::new("expr_term")
        .token("num", "num")
        .token("+", "+")
        .rule("term", vec!["num"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build()
}

// ===========================================================================
// Category 1: state_construction_* (8 tests)
// ===========================================================================

#[test]
fn state_construction_single_token_nonzero_states() {
    let pt = build(&single_token_grammar());
    assert!(pt.state_count > 0, "automaton must have at least one state");
}

#[test]
fn state_construction_linear_state_count() {
    // A linear grammar S → a b c needs states for each shift position + accept
    let pt = build(&linear_grammar());
    assert!(
        pt.state_count >= 4,
        "linear grammar with 3 tokens needs at least 4 states, got {}",
        pt.state_count
    );
}

#[test]
fn state_construction_recursive_has_more_states_than_single() {
    let pt_single = build(&single_token_grammar());
    let pt_rr = build(&right_recursive_grammar());
    assert!(
        pt_rr.state_count >= pt_single.state_count,
        "recursive grammar should have at least as many states as single-token"
    );
}

#[test]
fn state_construction_left_recursive_builds_ok() {
    let g = left_recursive_grammar();
    let pt = build(&g);
    assert!(pt.state_count > 0);
    sanity_check_tables(&pt).expect("left-recursive should pass sanity check");
}

#[test]
fn state_construction_nested_builds_ok() {
    let g = nested_grammar();
    let pt = build(&g);
    assert!(pt.state_count > 0);
    sanity_check_tables(&pt).expect("nested grammar sanity");
}

#[test]
fn state_construction_paren_builds_ok() {
    let pt = build(&paren_grammar());
    assert!(pt.state_count > 0);
    sanity_check_tables(&pt).expect("paren grammar sanity");
}

#[test]
fn state_construction_all_states_reachable() {
    let pt = build(&linear_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(
        reachable.len(),
        pt.state_count,
        "all states should be reachable from initial state"
    );
}

#[test]
fn state_construction_chain_nonterminals_have_goto() {
    let g = chain_grammar();
    let pt = build(&g);
    // Each nonterminal in the chain should have at least one goto entry
    for name in &["inner", "wrap", "chain"] {
        let sym = nt(&g, name);
        assert!(
            has_any_goto(&pt, sym),
            "nonterminal '{name}' should have a goto entry"
        );
    }
}

// ===========================================================================
// Category 2: state_actions_* (8 tests)
// ===========================================================================

#[test]
fn state_actions_single_token_has_nonempty_cells() {
    let pt = build(&single_token_grammar());
    assert!(
        nonempty_action_cells(&pt) > 0,
        "action table must have at least one non-empty cell"
    );
}

#[test]
fn state_actions_linear_terminal_coverage() {
    let g = linear_grammar();
    let pt = build(&g);
    // Every terminal should appear as a shift or reduce somewhere
    for name in &["a", "b", "c"] {
        let sym = tok(&g, name);
        let has_action =
            (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), sym).is_empty());
        assert!(
            has_action,
            "terminal '{name}' should have at least one action"
        );
    }
}

#[test]
fn state_actions_eof_appears_in_table() {
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    let has_eof_action =
        (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), eof).is_empty());
    assert!(
        has_eof_action,
        "EOF must appear somewhere in the action table"
    );
}

#[test]
fn state_actions_two_alt_both_tokens_have_actions() {
    let g = two_alt_grammar();
    let pt = build(&g);
    let a = tok(&g, "a");
    let b = tok(&g, "b");
    let has_a = (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), a).is_empty());
    let has_b = (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), b).is_empty());
    assert!(has_a, "token 'a' must have actions");
    assert!(has_b, "token 'b' must have actions");
}

#[test]
fn state_actions_list_comma_has_action() {
    let g = list_grammar();
    let pt = build(&g);
    let comma = tok(&g, ",");
    let has_comma = (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), comma).is_empty());
    assert!(
        has_comma,
        "comma terminal must have actions in list grammar"
    );
}

#[test]
fn state_actions_paren_all_tokens_present() {
    let g = paren_grammar();
    let pt = build(&g);
    for name in &["(", ")", "a"] {
        let sym = tok(&g, name);
        let present = (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), sym).is_empty());
        assert!(
            present,
            "token '{name}' must be present in paren grammar actions"
        );
    }
}

#[test]
fn state_actions_invalid_symbol_returns_empty() {
    let pt = build(&single_token_grammar());
    // SymbolId far beyond any real symbol should yield empty
    let bogus = SymbolId(60000);
    let actions = pt.actions(StateId(0), bogus);
    assert!(
        actions.is_empty(),
        "invalid symbol should give empty actions"
    );
}

#[test]
fn state_actions_invalid_state_returns_empty() {
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    // State far beyond table should not panic — actions() guards bounds
    let actions = pt.actions(StateId(60000), eof);
    assert!(
        actions.is_empty(),
        "out-of-bounds state should give empty actions"
    );
}

// ===========================================================================
// Category 3: state_initial_* (8 tests)
// ===========================================================================

#[test]
fn state_initial_exists_in_range() {
    let pt = build(&single_token_grammar());
    assert!(
        (pt.initial_state.0 as usize) < pt.state_count,
        "initial state must be within state_count"
    );
}

#[test]
fn state_initial_has_actions() {
    let pt = build(&single_token_grammar());
    let row = pt.initial_state.0 as usize;
    let has_any = (0..pt.action_table[row].len()).any(|c| !pt.action_table[row][c].is_empty());
    assert!(has_any, "initial state must have at least one action");
}

#[test]
fn state_initial_linear_shifts_first_token() {
    let g = linear_grammar();
    let pt = build(&g);
    let a = tok(&g, "a");
    let actions = pt.actions(pt.initial_state, a);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift the first token 'a'"
    );
}

#[test]
fn state_initial_not_accept() {
    // For a non-nullable grammar, the initial state should not have Accept
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    let actions = pt.actions(pt.initial_state, eof);
    assert!(
        !actions.iter().any(|a| matches!(a, Action::Accept)),
        "initial state of non-nullable grammar should not accept on EOF"
    );
}

#[test]
fn state_initial_two_alt_shifts_both() {
    let g = two_alt_grammar();
    let pt = build(&g);
    let a = tok(&g, "a");
    let b = tok(&g, "b");
    let shifts_a = pt
        .actions(pt.initial_state, a)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    let shifts_b = pt
        .actions(pt.initial_state, b)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    assert!(shifts_a, "initial state should shift 'a'");
    assert!(shifts_b, "initial state should shift 'b'");
}

#[test]
fn state_initial_is_reachable() {
    let pt = build(&linear_grammar());
    let reachable = reachable_states(&pt);
    assert!(
        reachable.contains(&pt.initial_state.0),
        "initial state must be in the reachable set"
    );
}

#[test]
fn state_initial_paren_shifts_open_and_a() {
    let g = paren_grammar();
    let pt = build(&g);
    let open = tok(&g, "(");
    let a = tok(&g, "a");
    let shifts_open = pt
        .actions(pt.initial_state, open)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    let shifts_a = pt
        .actions(pt.initial_state, a)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    assert!(shifts_open, "initial should shift '('");
    assert!(shifts_a, "initial should shift 'a'");
}

#[test]
fn state_initial_nullable_grammar_accepts_or_shifts() {
    let g = nullable_start_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    let a = tok(&g, "a");
    let actions_eof = pt.actions(pt.initial_state, eof);
    let actions_a = pt.actions(pt.initial_state, a);
    // Nullable start: initial state should accept on EOF or reduce epsilon,
    // AND should shift 'a' for the non-empty alternative
    let has_eof_action = !actions_eof.is_empty();
    let has_a_action = !actions_a.is_empty();
    assert!(
        has_eof_action || has_a_action,
        "nullable grammar initial state must have actions on EOF or 'a'"
    );
}

// ===========================================================================
// Category 4: state_accept_* (8 tests)
// ===========================================================================

#[test]
fn state_accept_single_token_present() {
    let pt = build(&single_token_grammar());
    assert!(has_accept(&pt), "single-token grammar must have Accept");
}

#[test]
fn state_accept_linear_present() {
    let pt = build(&linear_grammar());
    assert!(has_accept(&pt), "linear grammar must have Accept");
}

#[test]
fn state_accept_recursive_present() {
    let pt = build(&right_recursive_grammar());
    assert!(has_accept(&pt), "right-recursive grammar must have Accept");
}

#[test]
fn state_accept_left_recursive_present() {
    let pt = build(&left_recursive_grammar());
    assert!(has_accept(&pt), "left-recursive grammar must have Accept");
}

#[test]
fn state_accept_only_on_eof() {
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    // Accept should only appear in the EOF column
    for s in 0..pt.state_count {
        for (&sym, &col) in &pt.symbol_to_index {
            if sym == eof {
                continue;
            }
            let row = s;
            if row < pt.action_table.len() && col < pt.action_table[row].len() {
                for a in &pt.action_table[row][col] {
                    assert!(
                        !matches!(a, Action::Accept),
                        "Accept should only appear on EOF, found on state {s} symbol {:?}",
                        sym
                    );
                }
            }
        }
    }
}

#[test]
fn state_accept_paren_present() {
    let pt = build(&paren_grammar());
    assert!(has_accept(&pt), "paren grammar must have Accept");
}

#[test]
fn state_accept_nullable_present() {
    let pt = build(&nullable_start_grammar());
    assert!(has_accept(&pt), "nullable grammar must have Accept");
}

#[test]
fn state_accept_ambiguous_present() {
    let pt = build(&ambiguous_expr_grammar());
    assert!(has_accept(&pt), "ambiguous grammar must have Accept");
}

// ===========================================================================
// Category 5: state_reduce_* (8 tests)
// ===========================================================================

#[test]
fn state_reduce_single_token_has_reduce() {
    let g = single_token_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    assert!(
        has_any_reduce(&pt, eof),
        "single-token grammar should reduce on EOF"
    );
}

#[test]
fn state_reduce_count_at_least_one() {
    let pt = build(&single_token_grammar());
    assert!(
        count_reduce_actions(&pt) >= 1,
        "must have at least one reduce action"
    );
}

#[test]
fn state_reduce_linear_on_eof() {
    let g = linear_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    assert!(
        has_any_reduce(&pt, eof),
        "linear grammar should have reduce on EOF"
    );
}

#[test]
fn state_reduce_rule_lhs_is_valid() {
    let pt = build(&linear_grammar());
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                if let Action::Reduce(rid) = a {
                    let (lhs, rhs_len) = pt.rule(*rid);
                    assert!(
                        lhs.0 != u16::MAX,
                        "reduce rule LHS should be a valid symbol"
                    );
                    assert!(
                        rhs_len <= 100,
                        "reduce rule RHS length {rhs_len} is unreasonably large"
                    );
                }
            }
        }
    }
}

#[test]
fn state_reduce_list_grammar_has_reduce() {
    let g = list_grammar();
    let pt = build(&g);
    assert!(
        count_reduce_actions(&pt) >= 1,
        "list grammar must have reduce actions"
    );
}

#[test]
fn state_reduce_left_recursive_reduces_on_eof() {
    let g = left_recursive_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    assert!(
        has_any_reduce(&pt, eof),
        "left-recursive grammar should reduce on EOF"
    );
}

#[test]
fn state_reduce_nested_has_multiple_reduces() {
    let g = nested_grammar();
    let pt = build(&g);
    // inner_a → a, inner_b → b, top → inner_a inner_b: at least 3 reductions
    assert!(
        count_reduce_actions(&pt) >= 3,
        "nested grammar should have at least 3 reduce actions, got {}",
        count_reduce_actions(&pt)
    );
}

#[test]
fn state_reduce_chain_has_reduces_for_each_level() {
    let g = chain_grammar();
    let pt = build(&g);
    // chain → wrap → inner → x: at least 3 reduce actions
    assert!(
        count_reduce_actions(&pt) >= 3,
        "chain grammar needs at least 3 reduces, got {}",
        count_reduce_actions(&pt)
    );
}

// ===========================================================================
// Category 6: state_shift_* (8 tests)
// ===========================================================================

#[test]
fn state_shift_single_token_shifts_a() {
    let g = single_token_grammar();
    let pt = build(&g);
    let a = tok(&g, "a");
    assert!(has_any_shift(&pt, a), "must shift 'a'");
}

#[test]
fn state_shift_linear_all_tokens() {
    let g = linear_grammar();
    let pt = build(&g);
    for name in &["a", "b", "c"] {
        let sym = tok(&g, name);
        assert!(has_any_shift(&pt, sym), "must shift '{name}'");
    }
}

#[test]
fn state_shift_count_at_least_one() {
    let pt = build(&single_token_grammar());
    assert!(
        count_shift_actions(&pt) >= 1,
        "must have at least one shift action"
    );
}

#[test]
fn state_shift_targets_within_range() {
    let pt = build(&linear_grammar());
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                if let Action::Shift(target) = a {
                    assert!(
                        (target.0 as usize) < pt.state_count,
                        "shift target {} must be < state_count {}",
                        target.0,
                        pt.state_count
                    );
                }
            }
        }
    }
}

#[test]
fn state_shift_paren_open_and_close() {
    let g = paren_grammar();
    let pt = build(&g);
    let open = tok(&g, "(");
    let close = tok(&g, ")");
    assert!(has_any_shift(&pt, open), "must shift '('");
    assert!(has_any_shift(&pt, close), "must shift ')'");
}

#[test]
fn state_shift_right_recursive_shifts_a() {
    let g = right_recursive_grammar();
    let pt = build(&g);
    let a = tok(&g, "a");
    assert!(has_any_shift(&pt, a), "right-recursive must shift 'a'");
}

#[test]
fn state_shift_if_then_shifts_keywords() {
    let g = if_then_grammar();
    let pt = build(&g);
    for name in &["if", "then", "id", "num"] {
        let sym = tok(&g, name);
        assert!(has_any_shift(&pt, sym), "must shift '{name}'");
    }
}

#[test]
fn state_shift_shared_prefix_shifts_a() {
    let g = shared_prefix_grammar();
    let pt = build(&g);
    let a = tok(&g, "a");
    assert!(
        has_any_shift(&pt, a),
        "shared-prefix grammar must shift 'a'"
    );
}

// ===========================================================================
// Category 7: state_fork_* (8 tests)
// ===========================================================================

#[test]
fn state_fork_ambiguous_expr_has_conflict() {
    let pt = build(&ambiguous_expr_grammar());
    assert!(
        has_conflict_or_fork(&pt),
        "ambiguous expr grammar should produce conflicts or Fork actions"
    );
}

#[test]
fn state_fork_double_ambig_has_conflict() {
    let pt = build(&double_ambig_grammar());
    assert!(
        has_conflict_or_fork(&pt),
        "double-ambiguous grammar should produce conflicts"
    );
}

#[test]
fn state_fork_unambiguous_no_fork() {
    let pt = build(&linear_grammar());
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                assert!(
                    !matches!(a, Action::Fork(_)),
                    "unambiguous linear grammar should not produce Fork"
                );
            }
        }
    }
}

#[test]
fn state_fork_single_token_no_fork() {
    let pt = build(&single_token_grammar());
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                assert!(
                    !matches!(a, Action::Fork(_)),
                    "single-token grammar should have no Fork"
                );
            }
        }
    }
}

#[test]
fn state_fork_ambiguous_still_accepts() {
    // Even with Fork/conflicts, grammar must still produce Accept
    let pt = build(&ambiguous_expr_grammar());
    assert!(
        has_accept(&pt),
        "ambiguous grammar with forks must still accept"
    );
}

#[test]
fn state_fork_ambiguous_sanity_check_passes() {
    let pt = build(&ambiguous_expr_grammar());
    sanity_check_tables(&pt).expect("ambiguous grammar should pass sanity checks");
}

#[test]
fn state_fork_double_ambig_sanity_check_passes() {
    let pt = build(&double_ambig_grammar());
    sanity_check_tables(&pt).expect("double-ambiguous grammar should pass sanity checks");
}

#[test]
fn state_fork_fork_contains_at_least_two_actions() {
    let pt = build(&ambiguous_expr_grammar());
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                if let Action::Fork(inner) = a {
                    assert!(
                        inner.len() >= 2,
                        "Fork must contain at least 2 alternatives, got {}",
                        inner.len()
                    );
                }
            }
        }
    }
}

// ===========================================================================
// Category 8: state_deterministic_* (8 tests)
// ===========================================================================

#[test]
fn state_deterministic_single_token_no_multi_action() {
    let pt = build(&single_token_grammar());
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            assert!(
                cell.len() <= 1,
                "single-token grammar should be fully deterministic, got {} actions in state {s}",
                cell.len()
            );
        }
    }
}

#[test]
fn state_deterministic_linear_no_multi_action() {
    let pt = build(&linear_grammar());
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            assert!(
                cell.len() <= 1,
                "linear grammar should be deterministic, state {s} has {} actions",
                cell.len()
            );
        }
    }
}

#[test]
fn state_deterministic_list_grammar_is_deterministic() {
    let pt = build(&list_grammar());
    // items → items , id | id is SLR(1)-parsable, so should be deterministic
    let is_det = !(0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    });
    assert!(is_det, "list grammar should be deterministic");
}

#[test]
fn state_deterministic_paren_is_deterministic() {
    let pt = build(&paren_grammar());
    let is_det = !(0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    });
    assert!(is_det, "paren grammar should be deterministic");
}

#[test]
fn state_deterministic_nested_is_deterministic() {
    let pt = build(&nested_grammar());
    let is_det = !(0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    });
    assert!(is_det, "nested grammar should be deterministic");
}

#[test]
fn state_deterministic_expr_term_is_deterministic() {
    // expr → expr + term | term ; term → num  is LR(1) and unambiguous
    let pt = build(&expr_term_grammar());
    let is_det = !(0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    });
    assert!(is_det, "expr/term grammar should be deterministic");
}

#[test]
fn state_deterministic_two_alt_is_deterministic() {
    // start → a | b  is trivially deterministic (different lookaheads)
    let pt = build(&two_alt_grammar());
    let is_det = !(0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    });
    assert!(is_det, "two-alt grammar should be deterministic");
}

#[test]
fn state_deterministic_shared_prefix_is_deterministic() {
    // start → a b | a c  is LR(1) (lookahead after 'a' distinguishes b vs c)
    let pt = build(&shared_prefix_grammar());
    let is_det = !(0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    });
    assert!(
        is_det,
        "shared-prefix grammar should be LR(1) deterministic"
    );
}
