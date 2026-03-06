#![allow(clippy::needless_range_loop)]
#![cfg(feature = "test-api")]

//! Comprehensive tests for state transitions in the GLR parser.
//!
//! Covers action table lookups, shift/reduce/accept state transitions,
//! goto transitions after reductions, and boundary/edge-case behavior.

use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseTable, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use std::collections::BTreeMap;

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

/// Hand-build a ParseTable for unit-level tests that don't need a full grammar.
fn hand_built_table(
    action_table: Vec<Vec<Vec<Action>>>,
    goto_table: Vec<Vec<StateId>>,
    symbol_to_index: BTreeMap<SymbolId, usize>,
    nonterminal_to_index: BTreeMap<SymbolId, usize>,
    rules: Vec<(SymbolId, u16)>,
    eof_symbol: SymbolId,
    start_symbol: SymbolId,
) -> ParseTable {
    let state_count = action_table.len();
    let symbol_count = if state_count > 0 {
        action_table[0].len()
    } else {
        0
    };
    let mut index_to_symbol = vec![SymbolId(u16::MAX); symbol_to_index.len()];
    for (sym, &idx) in &symbol_to_index {
        if idx < index_to_symbol.len() {
            index_to_symbol[idx] = *sym;
        }
    }
    let parse_rules = rules
        .into_iter()
        .map(|(lhs, rhs_len)| adze_glr_core::ParseRule { lhs, rhs_len })
        .collect();
    ParseTable {
        action_table,
        goto_table,
        symbol_metadata: vec![],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules: parse_rules,
        nonterminal_to_index,
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol,
        grammar: Grammar::new("test".into()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

// ===========================================================================
// 1. Action table: basic lookups
// ===========================================================================

#[test]
fn action_lookup_shift_returns_target_state() {
    let g = GrammarBuilder::new("shift")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    let shifts: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert!(!shifts.is_empty(), "initial state must shift on 'a'");
    for s in &shifts {
        assert!(
            (s.0 as usize) < table.state_count,
            "shift target must be valid state"
        );
    }
}

#[test]
fn action_lookup_on_unmapped_symbol_returns_empty() {
    let g = GrammarBuilder::new("unmapped")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bogus = SymbolId(9999);
    assert!(
        table.actions(table.initial_state, bogus).is_empty(),
        "unmapped symbol must yield empty actions"
    );
}

#[test]
fn action_lookup_on_out_of_range_state_returns_empty() {
    let g = GrammarBuilder::new("oor")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let far_state = StateId(table.state_count as u16 + 100);
    assert!(
        table.actions(far_state, a).is_empty(),
        "out-of-range state must yield empty actions"
    );
}

// ===========================================================================
// 2. Shift transitions move to the correct next state
// ===========================================================================

#[test]
fn shift_on_first_token_transitions_away_from_initial() {
    let g = GrammarBuilder::new("s1")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let x = tok_id(&g, "x");
    let shifts: Vec<_> = table
        .actions(table.initial_state, x)
        .iter()
        .filter_map(|a| match a {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert!(!shifts.is_empty());
    for s in &shifts {
        assert_ne!(
            *s, table.initial_state,
            "shift must transition to a different state"
        );
    }
}

#[test]
fn shift_chain_two_tokens() {
    // Grammar: S → a b
    let g = GrammarBuilder::new("chain2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");

    // Shift on 'a' from initial
    let after_a: Vec<_> = table
        .actions(table.initial_state, a)
        .iter()
        .filter_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert!(!after_a.is_empty(), "must shift on 'a'");

    // From state after 'a', shift on 'b'
    let state_after_a = after_a[0];
    let after_b: Vec<_> = table
        .actions(state_after_a, b)
        .iter()
        .filter_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert!(!after_b.is_empty(), "must shift on 'b' after 'a'");

    // All three states must be distinct
    let state_after_b = after_b[0];
    assert_ne!(table.initial_state, state_after_a);
    assert_ne!(state_after_a, state_after_b);
}

#[test]
fn shift_chain_three_tokens() {
    // Grammar: S → a b c
    let g = GrammarBuilder::new("chain3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let c = tok_id(&g, "c");

    let s1 = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift on a");
    let s2 = table
        .actions(s1, b)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift on b");
    let s3 = table
        .actions(s2, c)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift on c");

    // After consuming all three tokens, a reduce on EOF should be possible
    let eof = table.eof();
    let eof_actions = table.actions(s3, eof);
    let has_reduce_or_accept = eof_actions
        .iter()
        .any(|a| matches!(a, Action::Reduce(_) | Action::Accept));
    assert!(
        has_reduce_or_accept,
        "after shifting all tokens, reduce/accept on EOF expected"
    );
}

// ===========================================================================
// 3. Reduce transitions
// ===========================================================================

#[test]
fn reduce_on_eof_after_single_token() {
    // S → a  →  after shifting 'a', reduce S→a on EOF
    let g = GrammarBuilder::new("red1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let eof = table.eof();

    let after_a = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift on a");

    let reduces: Vec<_> = table
        .actions(after_a, eof)
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .collect();
    assert!(
        !reduces.is_empty(),
        "must have reduce on EOF after shifting the only token"
    );
}

#[test]
fn reduce_yields_rule_with_correct_lhs_and_rhs_len() {
    let g = GrammarBuilder::new("rinfo")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let eof = table.eof();

    let s1 = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .unwrap();
    let s2 = table
        .actions(s1, b)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .unwrap();

    let rule_id = table
        .actions(s2, eof)
        .iter()
        .find_map(|act| match act {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .expect("reduce on EOF");

    let (lhs, rhs_len) = table.rule(rule_id);
    let start_nt = nt_id(&g, "start");
    assert_eq!(lhs, start_nt, "reduction LHS must be start nonterminal");
    assert_eq!(rhs_len, 2, "S → a b has rhs_len = 2");
}

// ===========================================================================
// 4. Accept transitions
// ===========================================================================

#[test]
fn accept_on_eof_exists_after_reducing_to_start() {
    let g = GrammarBuilder::new("acc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start_nt = nt_id(&g, "start");

    let accept_state = table
        .goto(table.initial_state, start_nt)
        .expect("goto(initial, start) must exist");
    let eof = table.eof();
    let has_accept = table
        .actions(accept_state, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        has_accept,
        "state after goto(initial, start) must accept on EOF"
    );
}

#[test]
fn accept_only_on_eof_not_on_terminals() {
    let g = GrammarBuilder::new("acc_only")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start_nt = nt_id(&g, "start");
    let accept_state = table.goto(table.initial_state, start_nt).expect("goto");

    // Accept should not appear on terminal 'a'
    let a = tok_id(&g, "a");
    let no_accept = !table
        .actions(accept_state, a)
        .iter()
        .any(|act| matches!(act, Action::Accept));
    assert!(no_accept, "Accept must not appear on a non-EOF terminal");
}

#[test]
fn exactly_one_accept_state_exists() {
    let g = GrammarBuilder::new("one_acc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();

    let accept_states: Vec<usize> = (0..table.state_count)
        .filter(|&s| {
            table
                .actions(StateId(s as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .collect();
    assert_eq!(
        accept_states.len(),
        1,
        "exactly one state should accept on EOF"
    );
}

// ===========================================================================
// 5. Goto transitions
// ===========================================================================

#[test]
fn goto_for_start_symbol_defined_from_initial() {
    let g = GrammarBuilder::new("goto_start")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start_nt = nt_id(&g, "start");
    assert!(
        table.goto(table.initial_state, start_nt).is_some(),
        "goto(initial, start) must be defined"
    );
}

#[test]
fn goto_for_intermediate_nt_defined() {
    // S → inner ; inner → a
    let g = GrammarBuilder::new("goto_inter")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");

    let has_goto = (0..table.state_count).any(|s| table.goto(StateId(s as u16), inner).is_some());
    assert!(has_goto, "goto for 'inner' must exist in some state");
}

#[test]
fn goto_target_is_within_state_count() {
    let g = GrammarBuilder::new("goto_range")
        .token("a", "a")
        .token("b", "b")
        .rule("pair", vec!["a", "b"])
        .rule("start", vec!["pair"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!(
                    (tgt.0 as usize) < table.state_count,
                    "goto target state {} must be < state_count {}",
                    tgt.0,
                    table.state_count
                );
            }
        }
    }
}

#[test]
fn goto_on_unmapped_nonterminal_returns_none() {
    let g = GrammarBuilder::new("goto_none")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bogus_nt = SymbolId(9999);
    assert!(
        table.goto(table.initial_state, bogus_nt).is_none(),
        "goto for unmapped nonterminal must be None"
    );
}

#[test]
fn goto_from_out_of_range_state_returns_none() {
    let g = GrammarBuilder::new("goto_oor")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start_nt = nt_id(&g, "start");
    let far = StateId(table.state_count as u16 + 100);
    assert!(table.goto(far, start_nt).is_none());
}

// ===========================================================================
// 6. Shift-then-goto round-trip
// ===========================================================================

#[test]
fn shift_reduce_goto_round_trip_simple() {
    // S → a
    // Simulate: shift 'a' → reduce S→a → goto(initial, S) → accept
    let g = GrammarBuilder::new("rtrip")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let eof = table.eof();
    let start_nt = nt_id(&g, "start");

    // Step 1: shift 'a'
    let s1 = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift a");

    // Step 2: reduce on EOF → pops 1 symbol, exposes initial_state
    let rid = table
        .actions(s1, eof)
        .iter()
        .find_map(|act| match act {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .expect("reduce");
    let (lhs, rhs_len) = table.rule(rid);
    assert_eq!(lhs, start_nt);
    assert_eq!(rhs_len, 1);

    // Step 3: goto(initial, start)
    let accept_st = table.goto(table.initial_state, lhs).expect("goto");

    // Step 4: accept
    let accepted = table
        .actions(accept_st, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(accepted, "must accept after goto");
}

#[test]
fn shift_reduce_goto_round_trip_two_level() {
    // inner → a ; start → inner
    let g = GrammarBuilder::new("rtrip2")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let eof = table.eof();
    let inner_nt = nt_id(&g, "inner");
    let start_nt = nt_id(&g, "start");

    // Shift 'a'
    let s1 = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift a");

    // Reduce inner → a
    let rid = table
        .actions(s1, eof)
        .iter()
        .find_map(|act| match act {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .expect("reduce inner");
    let (lhs, _) = table.rule(rid);
    assert_eq!(lhs, inner_nt);

    // Goto(initial, inner)
    let s2 = table
        .goto(table.initial_state, inner_nt)
        .expect("goto inner");

    // Reduce start → inner
    let rid2 = table
        .actions(s2, eof)
        .iter()
        .find_map(|act| match act {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .expect("reduce start");
    let (lhs2, _) = table.rule(rid2);
    assert_eq!(lhs2, start_nt);

    // Goto(initial, start) → accept
    let accept_st = table
        .goto(table.initial_state, start_nt)
        .expect("goto start");
    assert!(
        table
            .actions(accept_st, eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    );
}

// ===========================================================================
// 7. Multiple terminals — selective transitions
// ===========================================================================

#[test]
fn initial_state_shifts_only_on_expected_first_tokens() {
    // S → a | b  — initial should shift on both 'a' and 'b'
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");

    let has_shift_a = table
        .actions(table.initial_state, a)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    let has_shift_b = table
        .actions(table.initial_state, b)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    assert!(has_shift_a, "must shift on 'a'");
    assert!(has_shift_b, "must shift on 'b'");
}

#[test]
fn no_shift_on_wrong_token_after_partial_parse() {
    // S → a b  — after shifting 'a', should NOT shift on 'a' again
    let g = GrammarBuilder::new("wrong")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");

    let after_a = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift a");

    let shifts_a_again = table
        .actions(after_a, a)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    assert!(
        !shifts_a_again,
        "after 'a', should not shift on 'a' again (expect 'b')"
    );
}

// ===========================================================================
// 8. Hand-built table: direct action lookup
// ===========================================================================

#[test]
fn hand_built_shift_lookup() {
    let mut sym = BTreeMap::new();
    sym.insert(SymbolId(1), 0);
    sym.insert(SymbolId(99), 1); // EOF
    let table = hand_built_table(
        vec![
            vec![vec![Action::Shift(StateId(1))], vec![]],
            vec![vec![], vec![Action::Accept]],
        ],
        vec![vec![StateId(0)], vec![StateId(0)]],
        sym,
        BTreeMap::new(),
        vec![],
        SymbolId(99),
        SymbolId(10),
    );
    assert_eq!(
        table.actions(StateId(0), SymbolId(1)),
        &[Action::Shift(StateId(1))]
    );
    assert!(table.actions(StateId(0), SymbolId(99)).is_empty());
    assert_eq!(table.actions(StateId(1), SymbolId(99)), &[Action::Accept]);
}

#[test]
fn hand_built_goto_lookup() {
    let mut sym = BTreeMap::new();
    sym.insert(SymbolId(1), 0);
    let mut nt_idx = BTreeMap::new();
    nt_idx.insert(SymbolId(10), 0);
    let table = hand_built_table(
        vec![vec![vec![Action::Shift(StateId(1))]]],
        vec![vec![StateId(2)]],
        sym,
        nt_idx,
        vec![],
        SymbolId(99),
        SymbolId(10),
    );
    assert_eq!(table.goto(StateId(0), SymbolId(10)), Some(StateId(2)));
    assert_eq!(table.goto(StateId(0), SymbolId(11)), None);
}

#[test]
fn hand_built_multiple_actions_in_cell() {
    let mut sym = BTreeMap::new();
    sym.insert(SymbolId(1), 0);
    let table = hand_built_table(
        vec![vec![vec![
            Action::Shift(StateId(3)),
            Action::Reduce(RuleId(0)),
        ]]],
        vec![vec![StateId(0)]],
        sym,
        BTreeMap::new(),
        vec![],
        SymbolId(99),
        SymbolId(10),
    );
    let cell = table.actions(StateId(0), SymbolId(1));
    assert_eq!(cell.len(), 2, "GLR cell should hold multiple actions");
    assert!(cell.contains(&Action::Shift(StateId(3))));
    assert!(cell.contains(&Action::Reduce(RuleId(0))));
}

// ===========================================================================
// 9. Hand-built goto: sentinel values
// ===========================================================================

#[test]
fn goto_sentinel_max_returns_none() {
    let mut sym = BTreeMap::new();
    sym.insert(SymbolId(1), 0);
    let mut nt_idx = BTreeMap::new();
    nt_idx.insert(SymbolId(10), 0);
    // StateId(u16::MAX) is the sentinel for "no transition"
    let table = hand_built_table(
        vec![vec![vec![]]],
        vec![vec![StateId(u16::MAX)]],
        sym,
        nt_idx,
        vec![],
        SymbolId(99),
        SymbolId(10),
    );
    assert!(
        table.goto(StateId(0), SymbolId(10)).is_none(),
        "u16::MAX sentinel must be treated as no-transition"
    );
}

// ===========================================================================
// 10. State count and table dimensions
// ===========================================================================

#[test]
fn state_count_matches_action_table_rows() {
    let g = GrammarBuilder::new("dim")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn action_table_rows_have_uniform_width() {
    let g = GrammarBuilder::new("uniform")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    if let Some(first_row) = table.action_table.first() {
        let width = first_row.len();
        for (i, row) in table.action_table.iter().enumerate() {
            assert_eq!(row.len(), width, "action row {i} width mismatch");
        }
    }
}

// ===========================================================================
// 11. EOF symbol identity
// ===========================================================================

#[test]
fn eof_symbol_in_symbol_to_index() {
    let g = GrammarBuilder::new("eofmap")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF must be in symbol_to_index"
    );
}

#[test]
fn eof_accessor_matches_field() {
    let g = GrammarBuilder::new("eofacc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.eof(), table.eof_symbol);
}

// ===========================================================================
// 12. More complex grammars — recursive & multi-rule
// ===========================================================================

#[test]
fn recursive_grammar_state_transitions() {
    // list → item | list item ; item → a
    let g = GrammarBuilder::new("recurse")
        .token("a", "a")
        .rule("item", vec!["a"])
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "item"])
        .start("list")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let item_nt = nt_id(&g, "item");
    let list_nt = nt_id(&g, "list");

    // Initial state must shift on 'a'
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|act| matches!(act, Action::Shift(_)))
    );

    // Goto for both item and list must exist somewhere
    let has_item_goto =
        (0..table.state_count).any(|s| table.goto(StateId(s as u16), item_nt).is_some());
    let has_list_goto =
        (0..table.state_count).any(|s| table.goto(StateId(s as u16), list_nt).is_some());
    assert!(has_item_goto, "goto for item must exist");
    assert!(has_list_goto, "goto for list must exist");
}

#[test]
fn recursive_grammar_can_shift_after_reduce() {
    // list → item | list item ; item → a
    // After parsing one 'a' (shift + reduce to list), should still shift on 'a'
    let g = GrammarBuilder::new("rec_cont")
        .token("a", "a")
        .rule("item", vec!["a"])
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "item"])
        .start("list")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let list_nt = nt_id(&g, "list");

    // After goto(initial, list), should be able to shift 'a' again for continuation
    if let Some(after_list) = table.goto(table.initial_state, list_nt) {
        let can_shift_a = table
            .actions(after_list, a)
            .iter()
            .any(|act| matches!(act, Action::Shift(_)));
        assert!(
            can_shift_a,
            "recursive grammar: after reducing to list, must still shift 'a'"
        );
    }
}

// ===========================================================================
// 13. Multi-alternative rules
// ===========================================================================

#[test]
fn alternative_rules_lead_to_same_accept_path() {
    // S → a | b — both alternatives should eventually reach accept
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let eof = table.eof();
    let start_nt = nt_id(&g, "start");

    for tok in [a, b] {
        let s1 = table
            .actions(table.initial_state, tok)
            .iter()
            .find_map(|act| match act {
                Action::Shift(s) => Some(*s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("must shift on {:?}", tok));

        let has_reduce = table
            .actions(s1, eof)
            .iter()
            .any(|act| matches!(act, Action::Reduce(_)));
        assert!(has_reduce, "must reduce on EOF after shifting {:?}", tok);
    }

    // Accept state via goto
    let accept_st = table
        .goto(table.initial_state, start_nt)
        .expect("goto start");
    assert!(
        table
            .actions(accept_st, eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    );
}

// ===========================================================================
// 14. Initial state properties
// ===========================================================================

#[test]
fn initial_state_is_zero_by_default() {
    let g = GrammarBuilder::new("init")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.initial_state, StateId(0));
}

#[test]
fn initial_state_has_no_accept() {
    let g = GrammarBuilder::new("no_acc_init")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let no_accept = !table
        .actions(table.initial_state, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        no_accept,
        "initial state must not accept (nothing consumed yet)"
    );
}

// ===========================================================================
// 15. Rule accessor
// ===========================================================================

#[test]
fn rule_accessor_returns_correct_info() {
    let g = GrammarBuilder::new("rule_info")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);

    for i in 0..table.rules.len() {
        let (lhs, rhs_len) = table.rule(RuleId(i as u16));
        // LHS must be a known nonterminal
        assert!(
            table.nonterminal_to_index.contains_key(&lhs)
                || table.symbol_to_index.contains_key(&lhs),
            "rule LHS must be a known symbol"
        );
        assert!(rhs_len <= 20, "rhs_len should be reasonable");
    }
}

// ===========================================================================
// 16. Goto table structure invariants
// ===========================================================================

#[test]
fn goto_table_rows_uniform_width() {
    let g = GrammarBuilder::new("gt_uni")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    if let Some(first) = table.goto_table.first() {
        let width = first.len();
        for (i, row) in table.goto_table.iter().enumerate() {
            assert_eq!(row.len(), width, "goto row {i} width mismatch");
        }
    }
}

#[test]
fn nonterminal_to_index_covers_all_grammar_nonterminals() {
    let g = GrammarBuilder::new("nt_cov")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let start_nt = nt_id(&g, "start");
    assert!(table.nonterminal_to_index.contains_key(&inner));
    assert!(table.nonterminal_to_index.contains_key(&start_nt));
}

// ===========================================================================
// 17. Every shift target has at least one action
// ===========================================================================

#[test]
fn every_shift_target_has_at_least_one_action() {
    let g = GrammarBuilder::new("shift_tgt")
        .token("a", "a")
        .token("b", "b")
        .rule("pair", vec!["a", "b"])
        .rule("start", vec!["pair"])
        .start("start")
        .build();
    let table = build_table(&g);

    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if let Action::Shift(tgt) = act {
                    let tgt_has_action = table
                        .symbol_to_index
                        .keys()
                        .any(|&sym2| !table.actions(*tgt, sym2).is_empty());
                    assert!(
                        tgt_has_action,
                        "shift target state {} must have at least one action",
                        tgt.0
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 18. Every goto target has at least one action or further goto
// ===========================================================================

#[test]
fn every_goto_target_is_reachable_and_useful() {
    let g = GrammarBuilder::new("goto_useful")
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
                let has_any_action = table
                    .symbol_to_index
                    .keys()
                    .any(|&sym| !table.actions(tgt, sym).is_empty());
                assert!(
                    has_any_action,
                    "goto target state {} from ({}, {:?}) must have at least one action",
                    tgt.0, s, nt
                );
            }
        }
    }
}

// ===========================================================================
// 19. Default ParseTable has no transitions
// ===========================================================================

#[test]
fn default_table_has_no_transitions() {
    let table = ParseTable::default();
    assert_eq!(table.state_count, 0);
    assert!(table.actions(StateId(0), SymbolId(0)).is_empty());
    assert!(table.goto(StateId(0), SymbolId(0)).is_none());
}

// ===========================================================================
// 20. Deeper nesting: S → A B ; A → a ; B → b
// ===========================================================================

#[test]
fn sequence_with_nested_nonterminals() {
    let g = GrammarBuilder::new("nest")
        .token("a", "a")
        .token("b", "b")
        .rule("big_a", vec!["a"])
        .rule("big_b", vec!["b"])
        .rule("start", vec!["big_a", "big_b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let big_a = nt_id(&g, "big_a");

    // Shift 'a'
    let _s1 = table
        .actions(table.initial_state, a)
        .iter()
        .find_map(|act| match act {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .expect("shift a");

    // Reduce A → a, then goto(initial, A)
    let after_a = table.goto(table.initial_state, big_a).expect("goto A");

    // After goto to A-state, should shift 'b'
    let has_shift_b = table
        .actions(after_a, b)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    assert!(has_shift_b, "after A, must shift 'b'");

    // Should NOT shift 'a' again
    let no_shift_a = !table
        .actions(after_a, a)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    assert!(no_shift_a, "after A, should not shift 'a'");
}
