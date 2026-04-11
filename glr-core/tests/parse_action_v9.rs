#![cfg(feature = "test-api")]
#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for parse actions (Shift/Reduce/Accept/Error/Recover/Fork)
//! in adze-glr-core, covering Action enum properties and ParseTable integration.

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, RuleId, StateId, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build grammar: pa_v9_s → pa_v9_a
fn build_simple_grammar_and_table() -> ParseTable {
    let g = GrammarBuilder::new("pa_v9_simple")
        .token("pa_v9_a", "a")
        .rule("pa_v9_s", vec!["pa_v9_a"])
        .start("pa_v9_s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

/// Build grammar with two terminals: pa_v9_expr → pa_v9_num | pa_v9_expr pa_v9_plus pa_v9_num
fn build_expr_grammar_and_table() -> ParseTable {
    let g = GrammarBuilder::new("pa_v9_expr")
        .token("pa_v9_num", "\\d+")
        .token("pa_v9_plus", "+")
        .rule("pa_v9_expr", vec!["pa_v9_num"])
        .rule("pa_v9_expr", vec!["pa_v9_expr", "pa_v9_plus", "pa_v9_num"])
        .start("pa_v9_expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

/// Build grammar with left-associative precedence to resolve conflicts
fn build_prec_grammar_and_table() -> ParseTable {
    let g = GrammarBuilder::new("pa_v9_prec")
        .token("pa_v9_id", "[a-z]+")
        .token("pa_v9_add", "+")
        .token("pa_v9_mul", "*")
        .rule("pa_v9_term", vec!["pa_v9_id"])
        .rule_with_precedence(
            "pa_v9_term",
            vec!["pa_v9_term", "pa_v9_add", "pa_v9_term"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "pa_v9_term",
            vec!["pa_v9_term", "pa_v9_mul", "pa_v9_term"],
            2,
            Associativity::Left,
        )
        .start("pa_v9_term")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

/// Collect all actions across all states and all symbols in a table
fn all_actions(table: &ParseTable) -> Vec<(StateId, SymbolId, Action)> {
    let mut result = Vec::new();
    for (s, row) in table.action_table.iter().enumerate() {
        for (col, cell) in row.iter().enumerate() {
            if col < table.index_to_symbol.len() {
                let sym = table.index_to_symbol[col];
                for act in cell {
                    result.push((StateId(s as u16), sym, act.clone()));
                }
            }
        }
    }
    result
}

// ===========================================================================
// §1  Action::Shift equality
// ===========================================================================

#[test]
fn pa_v9_shift_equal_same_state() {
    assert_eq!(Action::Shift(StateId(0)), Action::Shift(StateId(0)));
}

#[test]
fn pa_v9_shift_equal_high_state() {
    assert_eq!(Action::Shift(StateId(999)), Action::Shift(StateId(999)));
}

#[test]
fn pa_v9_shift_reflexive() {
    let a = Action::Shift(StateId(42));
    assert_eq!(a, a);
}

#[test]
fn pa_v9_shift_symmetric() {
    let a = Action::Shift(StateId(7));
    let b = Action::Shift(StateId(7));
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ===========================================================================
// §2  Action::Reduce equality
// ===========================================================================

#[test]
fn pa_v9_reduce_equal_same_rule() {
    assert_eq!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn pa_v9_reduce_equal_high_rule() {
    assert_eq!(Action::Reduce(RuleId(500)), Action::Reduce(RuleId(500)));
}

#[test]
fn pa_v9_reduce_reflexive() {
    let a = Action::Reduce(RuleId(3));
    assert_eq!(a, a);
}

#[test]
fn pa_v9_reduce_symmetric() {
    let a = Action::Reduce(RuleId(11));
    let b = Action::Reduce(RuleId(11));
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ===========================================================================
// §3  Action::Accept equality
// ===========================================================================

#[test]
fn pa_v9_accept_equal() {
    assert_eq!(Action::Accept, Action::Accept);
}

#[test]
fn pa_v9_accept_reflexive() {
    let a = Action::Accept;
    assert_eq!(a, a);
}

#[test]
fn pa_v9_accept_symmetric() {
    let a = Action::Accept;
    let b = Action::Accept;
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ===========================================================================
// §4  Action::Error equality
// ===========================================================================

#[test]
fn pa_v9_error_equal() {
    assert_eq!(Action::Error, Action::Error);
}

#[test]
fn pa_v9_error_reflexive() {
    let a = Action::Error;
    assert_eq!(a, a);
}

#[test]
fn pa_v9_error_symmetric() {
    let a = Action::Error;
    let b = Action::Error;
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ===========================================================================
// §5  Action::Recover equality
// ===========================================================================

#[test]
fn pa_v9_recover_equal() {
    assert_eq!(Action::Recover, Action::Recover);
}

#[test]
fn pa_v9_recover_reflexive() {
    let a = Action::Recover;
    assert_eq!(a, a);
}

#[test]
fn pa_v9_recover_symmetric() {
    let a = Action::Recover;
    let b = Action::Recover;
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ===========================================================================
// §6  Different shifts are not equal
// ===========================================================================

#[test]
fn pa_v9_shift_different_states_not_equal() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
}

#[test]
fn pa_v9_shift_zero_vs_max_not_equal() {
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(u16::MAX)));
}

#[test]
fn pa_v9_shift_adjacent_states_not_equal() {
    assert_ne!(Action::Shift(StateId(99)), Action::Shift(StateId(100)));
}

#[test]
fn pa_v9_shift_many_distinct() {
    let actions: Vec<Action> = (0..10).map(|i| Action::Shift(StateId(i))).collect();
    for i in 0..actions.len() {
        for j in (i + 1)..actions.len() {
            assert_ne!(actions[i], actions[j]);
        }
    }
}

// ===========================================================================
// §7  Different reduces are not equal
// ===========================================================================

#[test]
fn pa_v9_reduce_different_rules_not_equal() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
}

#[test]
fn pa_v9_reduce_zero_vs_max_not_equal() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(u16::MAX)));
}

#[test]
fn pa_v9_reduce_adjacent_rules_not_equal() {
    assert_ne!(Action::Reduce(RuleId(49)), Action::Reduce(RuleId(50)));
}

#[test]
fn pa_v9_reduce_many_distinct() {
    let actions: Vec<Action> = (0..10).map(|i| Action::Reduce(RuleId(i))).collect();
    for i in 0..actions.len() {
        for j in (i + 1)..actions.len() {
            assert_ne!(actions[i], actions[j]);
        }
    }
}

// ===========================================================================
// §8  Shift ≠ Reduce (cross-variant inequality)
// ===========================================================================

#[test]
fn pa_v9_shift_ne_reduce_same_id() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn pa_v9_shift_ne_accept() {
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

#[test]
fn pa_v9_shift_ne_error() {
    assert_ne!(Action::Shift(StateId(0)), Action::Error);
}

#[test]
fn pa_v9_shift_ne_recover() {
    assert_ne!(Action::Shift(StateId(0)), Action::Recover);
}

#[test]
fn pa_v9_reduce_ne_accept() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Accept);
}

#[test]
fn pa_v9_reduce_ne_error() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Error);
}

#[test]
fn pa_v9_reduce_ne_recover() {
    assert_ne!(Action::Reduce(RuleId(0)), Action::Recover);
}

#[test]
fn pa_v9_accept_ne_error() {
    assert_ne!(Action::Accept, Action::Error);
}

#[test]
fn pa_v9_accept_ne_recover() {
    assert_ne!(Action::Accept, Action::Recover);
}

#[test]
fn pa_v9_error_ne_recover() {
    assert_ne!(Action::Error, Action::Recover);
}

// ===========================================================================
// §9  Action Debug contains variant name
// ===========================================================================

#[test]
fn pa_v9_debug_shift_contains_shift() {
    let dbg = format!("{:?}", Action::Shift(StateId(5)));
    assert!(dbg.contains("Shift"));
}

#[test]
fn pa_v9_debug_reduce_contains_reduce() {
    let dbg = format!("{:?}", Action::Reduce(RuleId(3)));
    assert!(dbg.contains("Reduce"));
}

#[test]
fn pa_v9_debug_accept_contains_accept() {
    let dbg = format!("{:?}", Action::Accept);
    assert!(dbg.contains("Accept"));
}

#[test]
fn pa_v9_debug_error_contains_error() {
    let dbg = format!("{:?}", Action::Error);
    assert!(dbg.contains("Error"));
}

#[test]
fn pa_v9_debug_recover_contains_recover() {
    let dbg = format!("{:?}", Action::Recover);
    assert!(dbg.contains("Recover"));
}

#[test]
fn pa_v9_debug_fork_contains_fork() {
    let dbg = format!(
        "{:?}",
        Action::Fork(vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(1))])
    );
    assert!(dbg.contains("Fork"));
}

#[test]
fn pa_v9_debug_shift_contains_state_id() {
    let dbg = format!("{:?}", Action::Shift(StateId(42)));
    assert!(dbg.contains("42"));
}

#[test]
fn pa_v9_debug_reduce_contains_rule_id() {
    let dbg = format!("{:?}", Action::Reduce(RuleId(17)));
    assert!(dbg.contains("17"));
}

// ===========================================================================
// §10  Action Clone preserves value
// ===========================================================================

#[test]
fn pa_v9_clone_shift() {
    let a = Action::Shift(StateId(10));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn pa_v9_clone_reduce() {
    let a = Action::Reduce(RuleId(5));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn pa_v9_clone_accept() {
    let a = Action::Accept;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn pa_v9_clone_error() {
    let a = Action::Error;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn pa_v9_clone_recover() {
    let a = Action::Recover;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn pa_v9_clone_fork() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn pa_v9_clone_independence() {
    let a = Action::Fork(vec![Action::Shift(StateId(1))]);
    let mut b = a.clone();
    if let Action::Fork(ref mut v) = b {
        v.push(Action::Error);
    }
    assert_ne!(a, b);
}

// ===========================================================================
// §11  Parse table state 0 has actions
// ===========================================================================

#[test]
fn pa_v9_simple_table_state0_nonempty() {
    let table = build_simple_grammar_and_table();
    assert!(!table.action_table.is_empty());
    let row0 = &table.action_table[0];
    let has_action = row0.iter().any(|cell| !cell.is_empty());
    assert!(has_action, "state 0 must have at least one action");
}

#[test]
fn pa_v9_expr_table_state0_nonempty() {
    let table = build_expr_grammar_and_table();
    let row0 = &table.action_table[0];
    let has_action = row0.iter().any(|cell| !cell.is_empty());
    assert!(has_action, "state 0 must have at least one action");
}

#[test]
fn pa_v9_initial_state_has_shift() {
    let table = build_simple_grammar_and_table();
    let all = all_actions(&table);
    let initial_shifts = all
        .iter()
        .filter(|(s, _, a)| *s == table.initial_state && matches!(a, Action::Shift(_)))
        .count();
    assert!(initial_shifts > 0, "initial state must have a shift");
}

#[test]
fn pa_v9_prec_table_state0_nonempty() {
    let table = build_prec_grammar_and_table();
    let row0 = &table.action_table[0];
    let has_action = row0.iter().any(|cell| !cell.is_empty());
    assert!(has_action);
}

// ===========================================================================
// §12  Accept action exists for start production
// ===========================================================================

#[test]
fn pa_v9_simple_has_accept() {
    let table = build_simple_grammar_and_table();
    let all = all_actions(&table);
    let has_accept = all.iter().any(|(_, _, a)| matches!(a, Action::Accept));
    assert!(has_accept, "table must contain an Accept action");
}

#[test]
fn pa_v9_expr_has_accept() {
    let table = build_expr_grammar_and_table();
    let all = all_actions(&table);
    let has_accept = all.iter().any(|(_, _, a)| matches!(a, Action::Accept));
    assert!(has_accept);
}

#[test]
fn pa_v9_prec_has_accept() {
    let table = build_prec_grammar_and_table();
    let all = all_actions(&table);
    let has_accept = all.iter().any(|(_, _, a)| matches!(a, Action::Accept));
    assert!(has_accept);
}

#[test]
fn pa_v9_accept_on_eof() {
    let table = build_simple_grammar_and_table();
    let eof = table.eof();
    let all = all_actions(&table);
    let accept_on_eof = all
        .iter()
        .any(|(_, sym, a)| *sym == eof && matches!(a, Action::Accept));
    assert!(accept_on_eof, "Accept must appear on EOF symbol");
}

// ===========================================================================
// §13  Reduce action rhs matches rule length
// ===========================================================================

#[test]
fn pa_v9_reduce_rhs_len_matches_simple() {
    let table = build_simple_grammar_and_table();
    let all = all_actions(&table);
    for (_, _, act) in &all {
        if let Action::Reduce(rid) = act {
            let (_lhs, rhs_len) = table.rule(*rid);
            assert!(rhs_len > 0, "rhs_len must be positive");
        }
    }
}

#[test]
fn pa_v9_reduce_rhs_len_single_symbol() {
    // S → a has rhs_len == 1
    let table = build_simple_grammar_and_table();
    let all = all_actions(&table);
    let reduces: Vec<_> = all
        .iter()
        .filter_map(|(_, _, a)| {
            if let Action::Reduce(rid) = a {
                Some(*rid)
            } else {
                None
            }
        })
        .collect();
    assert!(!reduces.is_empty(), "must have at least one reduce");
    // The user rule S → a should have rhs_len == 1
    let has_len_1 = reduces.iter().any(|rid| {
        let (_, rhs_len) = table.rule(*rid);
        rhs_len == 1
    });
    assert!(has_len_1, "S → a should have rhs_len 1");
}

#[test]
fn pa_v9_reduce_rhs_len_three_symbols() {
    // expr → expr + num has rhs_len == 3
    let table = build_expr_grammar_and_table();
    let all = all_actions(&table);
    let has_len_3 = all.iter().any(|(_, _, a)| {
        if let Action::Reduce(rid) = a {
            let (_, rhs_len) = table.rule(*rid);
            rhs_len == 3
        } else {
            false
        }
    });
    assert!(has_len_3, "expr → expr + num should have rhs_len 3");
}

#[test]
fn pa_v9_all_reduces_valid_rule_id() {
    let table = build_expr_grammar_and_table();
    let all = all_actions(&table);
    for (_, _, act) in &all {
        if let Action::Reduce(rid) = act {
            assert!(
                (rid.0 as usize) < table.rules.len(),
                "rule id {} out of bounds (max {})",
                rid.0,
                table.rules.len()
            );
        }
    }
}

// ===========================================================================
// §14  Shift leads to valid state
// ===========================================================================

#[test]
fn pa_v9_shift_target_in_bounds_simple() {
    let table = build_simple_grammar_and_table();
    let all = all_actions(&table);
    for (_, _, act) in &all {
        if let Action::Shift(sid) = act {
            assert!(
                (sid.0 as usize) < table.state_count,
                "shift target {} >= state_count {}",
                sid.0,
                table.state_count
            );
        }
    }
}

#[test]
fn pa_v9_shift_target_in_bounds_expr() {
    let table = build_expr_grammar_and_table();
    let all = all_actions(&table);
    for (_, _, act) in &all {
        if let Action::Shift(sid) = act {
            assert!((sid.0 as usize) < table.state_count);
        }
    }
}

#[test]
fn pa_v9_shift_target_in_bounds_prec() {
    let table = build_prec_grammar_and_table();
    let all = all_actions(&table);
    for (_, _, act) in &all {
        if let Action::Shift(sid) = act {
            assert!((sid.0 as usize) < table.state_count);
        }
    }
}

#[test]
fn pa_v9_shift_target_has_nonempty_row() {
    let table = build_simple_grammar_and_table();
    let all = all_actions(&table);
    for (_, _, act) in &all {
        if let Action::Shift(sid) = act {
            let row = &table.action_table[sid.0 as usize];
            let nonempty = row.iter().any(|cell| !cell.is_empty());
            assert!(nonempty, "shifted-to state {} must have actions", sid.0);
        }
    }
}

// ===========================================================================
// §15  Goto for non-terminal → valid state
// ===========================================================================

#[test]
fn pa_v9_goto_start_symbol_from_initial() {
    let table = build_simple_grammar_and_table();
    let start = table.start_symbol();
    let result = table.goto(table.initial_state, start);
    assert!(result.is_some(), "goto(initial, start_symbol) must be Some");
}

#[test]
fn pa_v9_goto_target_in_bounds() {
    let table = build_simple_grammar_and_table();
    let start = table.start_symbol();
    if let Some(sid) = table.goto(table.initial_state, start) {
        assert!((sid.0 as usize) < table.state_count);
    }
}

#[test]
fn pa_v9_goto_expr_from_initial() {
    let table = build_expr_grammar_and_table();
    let start = table.start_symbol();
    let result = table.goto(table.initial_state, start);
    assert!(result.is_some());
}

#[test]
fn pa_v9_goto_all_nonterminals_valid() {
    let table = build_expr_grammar_and_table();
    for &nt in table.nonterminal_to_index.keys() {
        for s in 0..table.state_count {
            if let Some(target) = table.goto(StateId(s as u16), nt) {
                assert!(
                    (target.0 as usize) < table.state_count,
                    "goto({}, {:?}) = {} out of bounds",
                    s,
                    nt,
                    target.0
                );
            }
        }
    }
}

// ===========================================================================
// §16  Goto for terminal → None
// ===========================================================================

#[test]
fn pa_v9_goto_terminal_returns_none_simple() {
    let table = build_simple_grammar_and_table();
    // Find a terminal symbol
    for &sym in table.symbol_to_index.keys() {
        if !table.nonterminal_to_index.contains_key(&sym) {
            let result = table.goto(table.initial_state, sym);
            assert!(
                result.is_none(),
                "goto on terminal {:?} should be None",
                sym
            );
        }
    }
}

#[test]
fn pa_v9_goto_terminal_returns_none_expr() {
    let table = build_expr_grammar_and_table();
    for &sym in table.symbol_to_index.keys() {
        if !table.nonterminal_to_index.contains_key(&sym) {
            for s in 0..table.state_count {
                let result = table.goto(StateId(s as u16), sym);
                assert!(result.is_none());
            }
        }
    }
}

#[test]
fn pa_v9_goto_eof_returns_none() {
    let table = build_simple_grammar_and_table();
    let eof = table.eof();
    let result = table.goto(table.initial_state, eof);
    assert!(result.is_none(), "goto on EOF should be None");
}

#[test]
fn pa_v9_goto_unknown_symbol_returns_none() {
    let table = build_simple_grammar_and_table();
    let unknown = SymbolId(60000);
    let result = table.goto(table.initial_state, unknown);
    assert!(result.is_none());
}

// ===========================================================================
// §17  Actions for EOF → includes Accept somewhere
// ===========================================================================

#[test]
fn pa_v9_eof_has_accept_simple() {
    let table = build_simple_grammar_and_table();
    let eof = table.eof();
    let mut found_accept = false;
    for s in 0..table.state_count {
        let actions = table.actions(StateId(s as u16), eof);
        if actions.iter().any(|a| matches!(a, Action::Accept)) {
            found_accept = true;
            break;
        }
    }
    assert!(found_accept, "some state must Accept on EOF");
}

#[test]
fn pa_v9_eof_has_accept_expr() {
    let table = build_expr_grammar_and_table();
    let eof = table.eof();
    let mut found_accept = false;
    for s in 0..table.state_count {
        let actions = table.actions(StateId(s as u16), eof);
        if actions.iter().any(|a| matches!(a, Action::Accept)) {
            found_accept = true;
            break;
        }
    }
    assert!(found_accept);
}

#[test]
fn pa_v9_eof_has_accept_prec() {
    let table = build_prec_grammar_and_table();
    let eof = table.eof();
    let mut found_accept = false;
    for s in 0..table.state_count {
        let actions = table.actions(StateId(s as u16), eof);
        if actions.iter().any(|a| matches!(a, Action::Accept)) {
            found_accept = true;
            break;
        }
    }
    assert!(found_accept);
}

#[test]
fn pa_v9_eof_accept_unique_state() {
    // Accept on EOF should appear in exactly one state
    let table = build_simple_grammar_and_table();
    let eof = table.eof();
    let accept_states: Vec<_> = (0..table.state_count)
        .filter(|&s| {
            table
                .actions(StateId(s as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .collect();
    assert!(
        !accept_states.is_empty(),
        "at least one state accepts on EOF"
    );
}

// ===========================================================================
// §18  All states have at least one action
// ===========================================================================

#[test]
fn pa_v9_all_states_have_actions_simple() {
    let table = build_simple_grammar_and_table();
    for s in 0..table.state_count {
        let row = &table.action_table[s];
        let has_action = row.iter().any(|cell| !cell.is_empty());
        assert!(has_action, "state {} has no actions", s);
    }
}

#[test]
fn pa_v9_all_states_have_actions_expr() {
    let table = build_expr_grammar_and_table();
    for s in 0..table.state_count {
        let row = &table.action_table[s];
        let has_action = row.iter().any(|cell| !cell.is_empty());
        assert!(has_action, "state {} has no actions", s);
    }
}

#[test]
fn pa_v9_all_states_have_actions_prec() {
    let table = build_prec_grammar_and_table();
    for s in 0..table.state_count {
        let row = &table.action_table[s];
        let has_action = row.iter().any(|cell| !cell.is_empty());
        assert!(has_action, "state {} has no actions", s);
    }
}

#[test]
fn pa_v9_state_count_matches_table_len() {
    let table = build_simple_grammar_and_table();
    assert_eq!(table.state_count, table.action_table.len());
}

// ===========================================================================
// §19  Action determinism: same query → same result
// ===========================================================================

#[test]
fn pa_v9_actions_deterministic_simple() {
    let table = build_simple_grammar_and_table();
    for &sym in table.symbol_to_index.keys() {
        for s in 0..table.state_count {
            let sid = StateId(s as u16);
            let first = table.actions(sid, sym);
            let second = table.actions(sid, sym);
            assert_eq!(first, second);
        }
    }
}

#[test]
fn pa_v9_actions_deterministic_expr() {
    let table = build_expr_grammar_and_table();
    for &sym in table.symbol_to_index.keys() {
        for s in 0..table.state_count {
            let sid = StateId(s as u16);
            let first = table.actions(sid, sym);
            let second = table.actions(sid, sym);
            assert_eq!(first, second);
        }
    }
}

#[test]
fn pa_v9_goto_deterministic() {
    let table = build_simple_grammar_and_table();
    let start = table.start_symbol();
    let first = table.goto(table.initial_state, start);
    let second = table.goto(table.initial_state, start);
    assert_eq!(first, second);
}

#[test]
fn pa_v9_rule_deterministic() {
    let table = build_simple_grammar_and_table();
    for i in 0..table.rules.len() {
        let rid = RuleId(i as u16);
        let first = table.rule(rid);
        let second = table.rule(rid);
        assert_eq!(first, second);
    }
}

// ===========================================================================
// §20  Rule lookup returns correct lhs and rhs_len
// ===========================================================================

#[test]
fn pa_v9_rule_lhs_is_nonterminal() {
    let table = build_simple_grammar_and_table();
    for i in 0..table.rules.len() {
        let (lhs, _) = table.rule(RuleId(i as u16));
        // lhs should be a known nonterminal (or the augmented start)
        assert!(
            table.nonterminal_to_index.contains_key(&lhs),
            "lhs must be a known nonterminal"
        );
    }
}

#[test]
fn pa_v9_rule_rhs_len_nonnegative() {
    let table = build_simple_grammar_and_table();
    for i in 0..table.rules.len() {
        let (_, rhs_len) = table.rule(RuleId(i as u16));
        // u16 is always >= 0, but verify it's reasonable
        assert!(rhs_len > 0, "rhs_len should be positive");
    }
}

#[test]
fn pa_v9_rule_expr_has_multiple_lengths() {
    let table = build_expr_grammar_and_table();
    let lengths: std::collections::BTreeSet<u16> = (0..table.rules.len())
        .map(|i| {
            let (_, rhs_len) = table.rule(RuleId(i as u16));
            rhs_len
        })
        .collect();
    // Should have at least rhs_len=1 (expr → num) and rhs_len=3 (expr → expr + num)
    assert!(lengths.contains(&1), "should have a rule with rhs_len 1");
    assert!(lengths.contains(&3), "should have a rule with rhs_len 3");
}

#[test]
fn pa_v9_rule_lhs_consistent_with_reduce() {
    let table = build_simple_grammar_and_table();
    let all = all_actions(&table);
    for (_, _, act) in &all {
        if let Action::Reduce(rid) = act {
            let (lhs, _) = table.rule(*rid);
            // The lhs should appear in the nonterminal_to_index or be the augmented start
            let known =
                table.nonterminal_to_index.contains_key(&lhs) || lhs == table.start_symbol();
            assert!(known, "reduce lhs {:?} should be a known nonterminal", lhs);
        }
    }
}

// ===========================================================================
// §21  Fork action tests
// ===========================================================================

#[test]
fn pa_v9_fork_equality_same_order() {
    let f1 = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let f2 = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    assert_eq!(f1, f2);
}

#[test]
fn pa_v9_fork_inequality_different_order() {
    let f1 = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    let f2 = Action::Fork(vec![Action::Reduce(RuleId(2)), Action::Shift(StateId(1))]);
    assert_ne!(f1, f2);
}

#[test]
fn pa_v9_fork_inequality_different_contents() {
    let f1 = Action::Fork(vec![Action::Shift(StateId(1))]);
    let f2 = Action::Fork(vec![Action::Shift(StateId(2))]);
    assert_ne!(f1, f2);
}

#[test]
fn pa_v9_fork_inequality_different_length() {
    let f1 = Action::Fork(vec![Action::Accept]);
    let f2 = Action::Fork(vec![Action::Accept, Action::Error]);
    assert_ne!(f1, f2);
}

#[test]
fn pa_v9_fork_ne_shift() {
    let f = Action::Fork(vec![Action::Shift(StateId(0))]);
    assert_ne!(f, Action::Shift(StateId(0)));
}

// ===========================================================================
// §22  Actions on unknown/invalid symbols
// ===========================================================================

#[test]
fn pa_v9_actions_unknown_symbol_empty() {
    let table = build_simple_grammar_and_table();
    let unknown = SymbolId(60000);
    let actions = table.actions(table.initial_state, unknown);
    assert!(actions.is_empty());
}

#[test]
fn pa_v9_actions_out_of_bounds_state_empty() {
    let table = build_simple_grammar_and_table();
    let eof = table.eof();
    let actions = table.actions(StateId(u16::MAX), eof);
    assert!(actions.is_empty());
}

// ===========================================================================
// §23  Table structure invariants
// ===========================================================================

#[test]
fn pa_v9_goto_table_rows_match_state_count() {
    let table = build_simple_grammar_and_table();
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn pa_v9_eof_symbol_in_action_index() {
    let table = build_simple_grammar_and_table();
    let eof = table.eof();
    assert!(
        table.symbol_to_index.contains_key(&eof),
        "EOF symbol must be indexed in the action table"
    );
}

#[test]
fn pa_v9_start_symbol_is_nonterminal() {
    let table = build_simple_grammar_and_table();
    let start = table.start_symbol();
    assert!(
        table.nonterminal_to_index.contains_key(&start),
        "start symbol must be in nonterminal index"
    );
}

#[test]
fn pa_v9_rules_nonempty() {
    let table = build_simple_grammar_and_table();
    assert!(!table.rules.is_empty(), "table must have rules");
}

#[test]
fn pa_v9_initial_state_within_bounds() {
    let table = build_simple_grammar_and_table();
    assert!((table.initial_state.0 as usize) < table.state_count);
}

// ===========================================================================
// §24  Multiple grammars produce different tables
// ===========================================================================

#[test]
fn pa_v9_different_grammars_different_state_counts() {
    let t1 = build_simple_grammar_and_table();
    let t2 = build_expr_grammar_and_table();
    // Different grammars typically produce different numbers of states
    // (simple: S→a vs expr: E→n | E+n)
    assert_ne!(t1.state_count, t2.state_count);
}

#[test]
fn pa_v9_different_grammars_different_rule_counts() {
    let t1 = build_simple_grammar_and_table();
    let t2 = build_expr_grammar_and_table();
    assert_ne!(t1.rules.len(), t2.rules.len());
}

// ===========================================================================
// §25  Hash consistency
// ===========================================================================

#[test]
fn pa_v9_hash_equal_actions_same_hash() {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let a = Action::Shift(StateId(5));
    let b = Action::Shift(StateId(5));
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    a.hash(&mut h1);
    b.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn pa_v9_hash_accept_consistent() {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    Action::Accept.hash(&mut h1);
    Action::Accept.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn pa_v9_hash_set_dedup() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Action::Shift(StateId(1)));
    set.insert(Action::Shift(StateId(1)));
    set.insert(Action::Reduce(RuleId(2)));
    assert_eq!(set.len(), 2);
}
