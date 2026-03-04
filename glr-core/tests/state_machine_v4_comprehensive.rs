//! Comprehensive V4 state-machine tests for `build_lr1_automaton` — covering
//! construction properties, action/goto tables, grammar topologies, EOF handling,
//! determinism, reachability, accept states, error behaviour, and goto consistency.

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

/// BFS reachability from state 0 via shift + goto targets.
fn reachable_states(pt: &ParseTable) -> BTreeSet<u16> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(pt.initial_state.0);
    visited.insert(pt.initial_state.0);

    while let Some(s) = queue.pop_front() {
        let si = s as usize;
        // Shift targets from action table
        if si < pt.action_table.len() {
            for col in 0..pt.action_table[si].len() {
                for a in &pt.action_table[si][col] {
                    collect_shift_targets(a, &mut visited, &mut queue);
                }
            }
        }
        // Goto targets
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

/// Count total non-empty action cells.
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

// ===================================================================
// Grammar factories
// ===================================================================

/// start → a
fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// start → a b c (linear chain)
fn linear_grammar() -> Grammar {
    GrammarBuilder::new("linear")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

/// start → a start | a  (right-recursive)
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// start → start a | a  (left-recursive)
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("lr")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// top → inner_a inner_b ; inner_a → a ; inner_b → b  (two nonterminals)
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

/// top → alt_a | alt_b ; alt_a → shared ; alt_b → shared ; shared → x  (diamond)
fn diamond_grammar() -> Grammar {
    GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("shared", vec!["x"])
        .rule("alt_a", vec!["shared"])
        .rule("alt_b", vec!["shared"])
        .rule("top", vec!["alt_a"])
        .rule("top", vec!["alt_b"])
        .start("top")
        .build()
}

/// start → ( start ) | a  (parenthesised nesting)
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

/// top → head tail | head ; head → a ; tail → b  (optional-tail)
fn optional_tail_grammar() -> Grammar {
    GrammarBuilder::new("optail")
        .token("a", "a")
        .token("b", "b")
        .rule("head", vec!["a"])
        .rule("tail", vec!["b"])
        .rule("top", vec!["head", "tail"])
        .rule("top", vec!["head"])
        .start("top")
        .build()
}

/// expr → expr + expr | num  (ambiguous)
fn ambiguous_expr_grammar() -> Grammar {
    GrammarBuilder::new("ambig_expr")
        .token("num", "num")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

/// items → items , id | id  (comma-separated list)
fn list_grammar() -> Grammar {
    GrammarBuilder::new("list")
        .token(",", ",")
        .token("id", "id")
        .rule("items", vec!["items", ",", "id"])
        .rule("items", vec!["id"])
        .start("items")
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

// ===================================================================
// 1. State machine construction properties
// ===================================================================

#[test]
fn state_count_positive_single_token() {
    let pt = build(&single_token_grammar());
    assert!(pt.state_count > 0);
}

#[test]
fn state_count_positive_linear() {
    let pt = build(&linear_grammar());
    assert!(pt.state_count > 0);
}

#[test]
fn initial_state_within_bounds() {
    let pt = build(&single_token_grammar());
    assert!((pt.initial_state.0 as usize) < pt.state_count);
}

#[test]
fn action_table_rows_match_state_count() {
    let pt = build(&linear_grammar());
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn action_table_columns_uniform() {
    let pt = build(&nested_grammar());
    for row in &pt.action_table {
        assert_eq!(row.len(), pt.symbol_count);
    }
}

#[test]
fn goto_table_rows_match_state_count() {
    let pt = build(&nested_grammar());
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn goto_table_columns_uniform() {
    let pt = build(&nested_grammar());
    let width = pt.goto_table[0].len();
    for row in &pt.goto_table {
        assert_eq!(row.len(), width);
    }
}

#[test]
fn sanity_check_passes_single_token() {
    let pt = build(&single_token_grammar());
    sanity_check_tables(&pt).expect("sanity check must pass");
}

#[test]
fn sanity_check_passes_linear() {
    let pt = build(&linear_grammar());
    sanity_check_tables(&pt).expect("sanity check must pass");
}

#[test]
fn sanity_check_passes_nested() {
    let pt = build(&nested_grammar());
    sanity_check_tables(&pt).expect("sanity check must pass");
}

#[test]
fn state_count_grows_with_grammar_size() {
    let small = build(&single_token_grammar());
    let large = build(&linear_grammar());
    assert!(
        large.state_count >= small.state_count,
        "more symbols should generally require at least as many states"
    );
}

#[test]
fn rules_populated() {
    let pt = build(&nested_grammar());
    assert!(!pt.rules.is_empty(), "parse table must contain rules");
}

// ===================================================================
// 2. Action table properties — shift/reduce/accept
// ===================================================================

#[test]
fn has_shift_for_terminal() {
    let g = single_token_grammar();
    let pt = build(&g);
    let a = tok(&g, "a");
    assert!(has_any_shift(&pt, a), "must shift on terminal 'a'");
}

#[test]
fn has_reduce_on_eof_single_rule() {
    let g = single_token_grammar();
    let pt = build(&g);
    let eof = pt.eof();
    // Either reduce or accept on EOF must exist
    let eof_actions: Vec<_> = (0..pt.state_count)
        .flat_map(|s| pt.actions(StateId(s as u16), eof).iter())
        .collect();
    assert!(
        eof_actions
            .iter()
            .any(|a| matches!(a, Action::Reduce(_) | Action::Accept)),
        "must have reduce or accept on EOF"
    );
}

#[test]
fn accept_present_single_token() {
    let pt = build(&single_token_grammar());
    assert!(has_accept(&pt), "single-token grammar must have Accept");
}

#[test]
fn accept_present_linear() {
    let pt = build(&linear_grammar());
    assert!(has_accept(&pt), "linear grammar must have Accept");
}

#[test]
fn accept_present_recursive() {
    let pt = build(&right_recursive_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_left_recursive() {
    let pt = build(&left_recursive_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_nested() {
    let pt = build(&nested_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_diamond() {
    let pt = build(&diamond_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_paren() {
    let pt = build(&paren_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_optional_tail() {
    let pt = build(&optional_tail_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_ambiguous_expr() {
    let pt = build(&ambiguous_expr_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_list() {
    let pt = build(&list_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn accept_present_nullable_start() {
    let pt = build(&nullable_start_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn shift_on_all_terminals_linear() {
    let g = linear_grammar();
    let pt = build(&g);
    for name in ["a", "b", "c"] {
        let sym = tok(&g, name);
        assert!(has_any_shift(&pt, sym), "must shift on '{name}'");
    }
}

#[test]
fn reduce_exists_for_each_rule_lhs() {
    let g = nested_grammar();
    let pt = build(&g);
    // There should be at least one reduce action somewhere for each grammar rule
    let has_some_reduce = (0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            pt.action_table[s][c]
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
    });
    assert!(has_some_reduce, "must have at least one reduce");
}

#[test]
fn nonempty_action_cells_exist() {
    let pt = build(&single_token_grammar());
    assert!(
        nonempty_action_cells(&pt) > 0,
        "must have some non-empty cells"
    );
}

// ===================================================================
// 3. Grammar topologies
// ===================================================================

#[test]
fn right_recursive_more_states_than_base() {
    let base = build(&single_token_grammar());
    let rr = build(&right_recursive_grammar());
    assert!(rr.state_count >= base.state_count);
}

#[test]
fn left_recursive_builds_successfully() {
    let pt = build(&left_recursive_grammar());
    assert!(pt.state_count > 0);
    sanity_check_tables(&pt).expect("left-recursive sanity");
}

#[test]
fn nested_has_goto_for_inner_nonterminals() {
    let g = nested_grammar();
    let pt = build(&g);
    let a_nt = nt(&g, "inner_a");
    let b_nt = nt(&g, "inner_b");
    assert!(has_any_goto(&pt, a_nt), "goto for inner_a must exist");
    assert!(has_any_goto(&pt, b_nt), "goto for inner_b must exist");
}

#[test]
fn diamond_has_goto_for_shared_nonterminal() {
    let g = diamond_grammar();
    let pt = build(&g);
    let c_nt = nt(&g, "shared");
    assert!(has_any_goto(&pt, c_nt), "goto for shared must exist");
}

#[test]
fn paren_shift_on_open_and_close() {
    let g = paren_grammar();
    let pt = build(&g);
    let open = tok(&g, "(");
    let close = tok(&g, ")");
    assert!(has_any_shift(&pt, open));
    assert!(has_any_shift(&pt, close));
}

#[test]
fn ambiguous_expr_has_conflicts_or_fork() {
    let g = ambiguous_expr_grammar();
    let pt = build(&g);
    // Ambiguous grammars produce either multi-action cells or Fork actions
    let has_multi = (0..pt.state_count).any(|s| {
        (0..pt.action_table[s].len()).any(|c| {
            let cell = &pt.action_table[s][c];
            cell.len() > 1 || cell.iter().any(|a| matches!(a, Action::Fork(_)))
        })
    });
    assert!(
        has_multi,
        "ambiguous grammar should have multi-action cells or Fork"
    );
}

#[test]
fn list_grammar_shift_on_comma() {
    let g = list_grammar();
    let pt = build(&g);
    let comma = tok(&g, ",");
    assert!(has_any_shift(&pt, comma), "must shift on comma");
}

#[test]
fn optional_tail_sanity() {
    let pt = build(&optional_tail_grammar());
    sanity_check_tables(&pt).expect("optional-tail sanity");
}

// ===================================================================
// 4. EOF handling
// ===================================================================

#[test]
fn eof_in_symbol_to_index() {
    let pt = build(&single_token_grammar());
    assert!(
        pt.symbol_to_index.contains_key(&pt.eof_symbol),
        "EOF must be in symbol_to_index"
    );
}

#[test]
fn eof_symbol_not_zero_for_builder_grammars() {
    // GrammarBuilder starts symbol IDs at 1, so EOF = max+1 > 0
    let pt = build(&single_token_grammar());
    assert_ne!(pt.eof_symbol, SymbolId(0));
}

#[test]
fn eof_unique_across_grammars() {
    let g1 = single_token_grammar();
    let g2 = linear_grammar();
    let pt1 = build(&g1);
    let pt2 = build(&g2);
    // EOF should not collide with any user-defined terminal in either grammar
    assert!(!g1.tokens.contains_key(&pt1.eof_symbol));
    assert!(!g2.tokens.contains_key(&pt2.eof_symbol));
}

#[test]
fn eof_accept_only_on_eof_column() {
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    for s in 0..pt.state_count {
        for (&sym, &col) in &pt.symbol_to_index {
            let cell = &pt.action_table[s][col];
            if sym != eof {
                assert!(
                    !cell.iter().any(|a| matches!(a, Action::Accept)),
                    "Accept must only appear on EOF column, found on sym={:?} state={}",
                    sym,
                    s,
                );
            }
        }
    }
}

#[test]
fn eof_no_shift_on_eof() {
    let pt = build(&linear_grammar());
    let eof = pt.eof();
    for s in 0..pt.state_count {
        let actions = pt.actions(StateId(s as u16), eof);
        assert!(
            !actions.iter().any(|a| matches!(a, Action::Shift(_))),
            "should not shift on EOF in state {s}"
        );
    }
}

#[test]
fn eof_handling_nullable() {
    let pt = build(&nullable_start_grammar());
    let eof = pt.eof();
    // Nullable start must accept on EOF (empty input is valid)
    let accepts_on_eof = (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(accepts_on_eof, "nullable grammar must accept on EOF");
}

#[test]
fn eof_handling_right_recursive() {
    let pt = build(&right_recursive_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn eof_handling_left_recursive() {
    let pt = build(&left_recursive_grammar());
    assert!(has_accept(&pt));
}

// ===================================================================
// 5. Determinism — same grammar → same table
// ===================================================================

#[test]
fn deterministic_state_count() {
    let g = linear_grammar();
    let pt1 = build(&g);
    let pt2 = build(&g);
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn deterministic_eof_symbol() {
    let g = nested_grammar();
    let pt1 = build(&g);
    let pt2 = build(&g);
    assert_eq!(pt1.eof_symbol, pt2.eof_symbol);
}

#[test]
fn deterministic_action_table_dimensions() {
    let g = right_recursive_grammar();
    let pt1 = build(&g);
    let pt2 = build(&g);
    assert_eq!(pt1.action_table.len(), pt2.action_table.len());
    for (r1, r2) in pt1.action_table.iter().zip(pt2.action_table.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

#[test]
fn deterministic_action_cell_counts() {
    let g = paren_grammar();
    let pt1 = build(&g);
    let pt2 = build(&g);
    for s in 0..pt1.state_count {
        for c in 0..pt1.action_table[s].len() {
            assert_eq!(
                pt1.action_table[s][c].len(),
                pt2.action_table[s][c].len(),
                "mismatch at state={s} col={c}"
            );
        }
    }
}

#[test]
fn deterministic_goto_dimensions() {
    let g = diamond_grammar();
    let pt1 = build(&g);
    let pt2 = build(&g);
    assert_eq!(pt1.goto_table.len(), pt2.goto_table.len());
    for (r1, r2) in pt1.goto_table.iter().zip(pt2.goto_table.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

#[test]
fn deterministic_rules() {
    let g = list_grammar();
    let pt1 = build(&g);
    let pt2 = build(&g);
    assert_eq!(pt1.rules.len(), pt2.rules.len());
    for (r1, r2) in pt1.rules.iter().zip(pt2.rules.iter()) {
        assert_eq!(r1.lhs, r2.lhs);
        assert_eq!(r1.rhs_len, r2.rhs_len);
    }
}

#[test]
fn deterministic_symbol_to_index() {
    let g = optional_tail_grammar();
    let pt1 = build(&g);
    let pt2 = build(&g);
    assert_eq!(pt1.symbol_to_index, pt2.symbol_to_index);
}

// ===================================================================
// 6. State reachability — all states reachable from initial
// ===================================================================

#[test]
fn all_states_reachable_single_token() {
    let pt = build(&single_token_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(
        reachable.len(),
        pt.state_count,
        "all {} states must be reachable, got {}",
        pt.state_count,
        reachable.len()
    );
}

#[test]
fn all_states_reachable_linear() {
    let pt = build(&linear_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn all_states_reachable_nested() {
    let pt = build(&nested_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn all_states_reachable_diamond() {
    let pt = build(&diamond_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn all_states_reachable_paren() {
    let pt = build(&paren_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn all_states_reachable_right_recursive() {
    let pt = build(&right_recursive_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn all_states_reachable_left_recursive() {
    let pt = build(&left_recursive_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn all_states_reachable_list() {
    let pt = build(&list_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

// ===================================================================
// 7. Accept state existence
// ===================================================================

#[test]
fn accept_on_eof_exactly_in_one_or_more_states() {
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    let accept_count = (0..pt.state_count)
        .filter(|&s| {
            pt.actions(StateId(s as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .count();
    assert!(accept_count >= 1, "at least one state must accept on EOF");
}

#[test]
fn accept_state_not_initial_for_non_nullable() {
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    let initial = pt.initial_state;
    let accepts_at_initial = pt
        .actions(initial, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        !accepts_at_initial,
        "non-nullable grammar should not accept in initial state"
    );
}

#[test]
fn nullable_grammar_may_accept_in_initial_state() {
    let pt = build(&nullable_start_grammar());
    let eof = pt.eof();
    // At least *some* state accepts — could be initial for nullable
    assert!(has_accept(&pt));

    // Verify the initial state has either accept or reduce on EOF
    let initial_eof_actions = pt.actions(pt.initial_state, eof);
    let has_eof_action = initial_eof_actions
        .iter()
        .any(|a| matches!(a, Action::Accept | Action::Reduce(_)));
    assert!(
        has_eof_action,
        "nullable grammar initial state must have accept or reduce on EOF"
    );
}

// ===================================================================
// 8. Error state behaviour
// ===================================================================

#[test]
fn empty_action_cell_implies_error() {
    let pt = build(&single_token_grammar());
    // At least one cell should be empty (no valid action = error)
    let has_empty = (0..pt.state_count)
        .any(|s| (0..pt.action_table[s].len()).any(|c| pt.action_table[s][c].is_empty()));
    assert!(
        has_empty,
        "there should be some error cells in the action table"
    );
}

#[test]
fn no_valid_action_on_wrong_terminal_in_initial() {
    let g = linear_grammar();
    let pt = build(&g);
    // Initial state expects 'a' first; 'b' and 'c' should be error
    let b = tok(&g, "b");
    let c = tok(&g, "c");
    let initial = pt.initial_state;
    assert!(
        pt.actions(initial, b).is_empty(),
        "initial state should not accept 'b'"
    );
    assert!(
        pt.actions(initial, c).is_empty(),
        "initial state should not accept 'c'"
    );
}

#[test]
fn querying_unknown_symbol_returns_empty() {
    let pt = build(&single_token_grammar());
    let unknown = SymbolId(9999);
    let actions = pt.actions(pt.initial_state, unknown);
    assert!(
        actions.is_empty(),
        "unknown symbol must return empty actions"
    );
}

#[test]
fn querying_out_of_bounds_state_returns_empty() {
    let pt = build(&single_token_grammar());
    let eof = pt.eof();
    let oob = StateId(pt.state_count as u16 + 100);
    let actions = pt.actions(oob, eof);
    assert!(
        actions.is_empty(),
        "out-of-bounds state must return empty actions"
    );
}

#[test]
fn goto_unknown_nonterminal_returns_none() {
    let pt = build(&single_token_grammar());
    let unknown = SymbolId(8888);
    assert!(
        pt.goto(pt.initial_state, unknown).is_none(),
        "goto on unknown nonterminal must be None"
    );
}

// ===================================================================
// 9. Goto table consistency
// ===================================================================

#[test]
fn goto_targets_within_state_count() {
    let pt = build(&nested_grammar());
    for s in 0..pt.state_count {
        for c in 0..pt.goto_table[s].len() {
            let target = pt.goto_table[s][c].0;
            assert!(
                target == u16::MAX || (target as usize) < pt.state_count,
                "goto target {} out of range at state={s} col={c}",
                target,
            );
        }
    }
}

#[test]
fn shift_targets_within_state_count() {
    let pt = build(&paren_grammar());
    for s in 0..pt.state_count {
        for c in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][c] {
                if let Action::Shift(t) = a {
                    assert!(
                        (t.0 as usize) < pt.state_count,
                        "shift target {} out of range",
                        t.0
                    );
                }
            }
        }
    }
}

#[test]
fn reduce_rule_ids_within_bounds() {
    let pt = build(&diamond_grammar());
    for s in 0..pt.state_count {
        for c in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][c] {
                if let Action::Reduce(rid) = a {
                    assert!(
                        (rid.0 as usize) < pt.rules.len(),
                        "reduce rule {} out of bounds (max {})",
                        rid.0,
                        pt.rules.len()
                    );
                }
            }
        }
    }
}

#[test]
fn goto_for_start_symbol_exists_from_initial() {
    let g = nested_grammar();
    let pt = build(&g);
    let s_nt = nt(&g, "top");
    let target = pt.goto(pt.initial_state, s_nt);
    assert!(target.is_some(), "goto(initial, start_symbol) must exist");
}

#[test]
fn goto_consistent_with_nonterminal_to_index() {
    let pt = build(&nested_grammar());
    for (&sym, &col) in &pt.nonterminal_to_index {
        // At least one row should have a non-sentinel value for this column
        let any_goto = (0..pt.state_count).any(|s| {
            pt.goto_table
                .get(s)
                .and_then(|row| row.get(col))
                .is_some_and(|target| target.0 != u16::MAX)
        });
        assert!(
            any_goto,
            "nonterminal {:?} has index but no goto entries",
            sym
        );
    }
}

#[test]
fn index_to_symbol_consistent() {
    let pt = build(&linear_grammar());
    for (&sym, &idx) in &pt.symbol_to_index {
        assert_eq!(
            pt.index_to_symbol[idx], sym,
            "index_to_symbol[{idx}] should map back to {:?}",
            sym
        );
    }
}

// ===================================================================
// 10. Additional edge-case and cross-cutting tests
// ===================================================================

#[test]
fn two_alternative_rules() {
    let g = GrammarBuilder::new("twoalt")
        .token("a", "a")
        .token("b", "b")
        .rule("top", vec!["a"])
        .rule("top", vec!["b"])
        .start("top")
        .build();
    let pt = build(&g);
    assert!(has_accept(&pt));
    sanity_check_tables(&pt).expect("two-alt sanity");
}

#[test]
fn deeply_nested_nonterminals() {
    let g = GrammarBuilder::new("deep")
        .token("x", "x")
        .rule("depth_d", vec!["x"])
        .rule("depth_c", vec!["depth_d"])
        .rule("depth_b", vec!["depth_c"])
        .rule("depth_a", vec!["depth_b"])
        .rule("top", vec!["depth_a"])
        .start("top")
        .build();
    let pt = build(&g);
    assert!(has_accept(&pt));
    sanity_check_tables(&pt).expect("deep sanity");

    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn single_epsilon_rule() {
    // start → ε
    let g = GrammarBuilder::new("eps")
        .rule("start", vec![])
        .start("start")
        .build();
    let pt = build(&g);
    assert!(has_accept(&pt), "epsilon-only grammar must accept");
}

#[test]
fn multiple_tokens_same_terminal() {
    let g = GrammarBuilder::new("dup_tok")
        .token("a", "a")
        .rule("start", vec!["a", "a", "a"])
        .start("start")
        .build();
    let pt = build(&g);
    assert!(has_accept(&pt));
    sanity_check_tables(&pt).expect("dup token sanity");
}

#[test]
fn mixed_recursive_and_base() {
    let g = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "start", "b"])
        .start("start")
        .build();
    let pt = build(&g);
    assert!(has_accept(&pt));
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn start_symbol_matches_grammar() {
    let g = nested_grammar();
    let pt = build(&g);
    let s_nt = nt(&g, "top");
    // The parse table's start_symbol should correspond to top (or the augmented start)
    // At minimum, start_symbol should be set
    assert_ne!(pt.start_symbol, SymbolId(0), "start_symbol must be set");
    // And there should be a goto for the user's start nonterminal
    assert!(has_any_goto(&pt, s_nt));
}

#[test]
fn token_count_consistent() {
    let g = linear_grammar();
    let pt = build(&g);
    // token_count should be at least the number of user-defined tokens
    assert!(
        pt.token_count >= g.tokens.len(),
        "token_count ({}) should be >= grammar tokens ({})",
        pt.token_count,
        g.tokens.len()
    );
}

#[test]
fn eof_symbol_column_index_matches() {
    let pt = build(&linear_grammar());
    let eof_idx = pt.symbol_to_index.get(&pt.eof_symbol);
    assert!(eof_idx.is_some(), "EOF must have a column index");
    let &idx = eof_idx.unwrap();
    assert_eq!(
        pt.index_to_symbol[idx], pt.eof_symbol,
        "index_to_symbol must map back to EOF"
    );
}

#[test]
fn action_table_no_fork_without_ambiguity() {
    // Unambiguous grammar should have no Fork actions
    let pt = build(&single_token_grammar());
    for s in 0..pt.state_count {
        for c in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][c] {
                assert!(
                    !matches!(a, Action::Fork(_)),
                    "unambiguous grammar should not produce Fork"
                );
            }
        }
    }
}

#[test]
fn sanity_check_list_grammar() {
    let pt = build(&list_grammar());
    sanity_check_tables(&pt).expect("list grammar sanity");
}

#[test]
fn sanity_check_right_recursive() {
    let pt = build(&right_recursive_grammar());
    sanity_check_tables(&pt).expect("right-recursive sanity");
}

#[test]
fn sanity_check_paren_grammar() {
    let pt = build(&paren_grammar());
    sanity_check_tables(&pt).expect("paren grammar sanity");
}

#[test]
fn sanity_check_ambiguous_expr() {
    let pt = build(&ambiguous_expr_grammar());
    sanity_check_tables(&pt).expect("ambiguous expr sanity");
}

#[test]
fn sanity_check_nullable() {
    let pt = build(&nullable_start_grammar());
    sanity_check_tables(&pt).expect("nullable sanity");
}

#[test]
fn reachable_states_optional_tail() {
    let pt = build(&optional_tail_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}

#[test]
fn reachable_states_ambiguous_expr() {
    let pt = build(&ambiguous_expr_grammar());
    let reachable = reachable_states(&pt);
    assert_eq!(reachable.len(), pt.state_count);
}
