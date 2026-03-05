#![cfg(feature = "test-api")]

//! V4 comprehensive tests for LR(1) automaton construction.
//!
//! Categories:
//! 1. Automaton has accept state (8 tests)
//! 2. Initial state has actions (8 tests)
//! 3. State count reasonable (8 tests)
//! 4. All states have actions (7 tests)
//! 5. Shift targets valid (8 tests)
//! 6. Reduce rule IDs valid (8 tests)
//! 7. Complex grammars (8 tests)

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

fn state_has_any_action(table: &adze_glr_core::ParseTable, state: StateId) -> bool {
    table.symbol_to_index.keys().any(|&sym| {
        table
            .actions(state, sym)
            .iter()
            .any(|a| !matches!(a, Action::Error))
    })
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

// ===========================================================================
// 1. Automaton has accept state (8 tests)
// ===========================================================================

#[test]
fn accept_single_token_grammar() {
    let g = GrammarBuilder::new("a1")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_two_token_sequence() {
    let g = GrammarBuilder::new("a2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_alternative_rules() {
    let g = GrammarBuilder::new("a3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_nonterminal_chain() {
    let g = GrammarBuilder::new("a4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_left_recursive() {
    let g = GrammarBuilder::new("a5")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_right_recursive() {
    let g = GrammarBuilder::new("a6")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_three_token_sequence() {
    let g = GrammarBuilder::new("a7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_multi_nonterminal() {
    let g = GrammarBuilder::new("a8")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

// ===========================================================================
// 2. Initial state has actions (8 tests)
// ===========================================================================

#[test]
fn initial_has_shift_single_token() {
    let g = GrammarBuilder::new("i1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|act| matches!(act, Action::Shift(_))),
        "initial state should shift on 'a'"
    );
}

#[test]
fn initial_has_shift_first_of_sequence() {
    let g = GrammarBuilder::new("i2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|act| matches!(act, Action::Shift(_)))
    );
}

#[test]
fn initial_has_action_for_alternative() {
    let g = GrammarBuilder::new("i3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let actions_a = table.actions(table.initial_state, a);
    let actions_b = table.actions(table.initial_state, b);
    assert!(
        !actions_a.is_empty() || !actions_b.is_empty(),
        "initial state should have actions for at least one alternative"
    );
}

#[test]
fn initial_has_action_chain() {
    let g = GrammarBuilder::new("i4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(state_has_any_action(&table, table.initial_state));
}

#[test]
fn initial_has_action_left_recursive() {
    let g = GrammarBuilder::new("i5")
        .token("n", "n")
        .rule("lst", vec!["lst", "n"])
        .rule("lst", vec!["n"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(state_has_any_action(&table, table.initial_state));
}

#[test]
fn initial_has_action_right_recursive() {
    let g = GrammarBuilder::new("i6")
        .token("n", "n")
        .rule("seq", vec!["n", "seq"])
        .rule("seq", vec!["n"])
        .start("seq")
        .build();
    let table = build_table(&g);
    assert!(state_has_any_action(&table, table.initial_state));
}

#[test]
fn initial_has_action_three_tokens() {
    let g = GrammarBuilder::new("i7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|act| matches!(act, Action::Shift(_)))
    );
}

#[test]
fn initial_has_action_multi_nt() {
    let g = GrammarBuilder::new("i8")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(state_has_any_action(&table, table.initial_state));
}

// ===========================================================================
// 3. State count reasonable (8 tests)
// ===========================================================================

#[test]
fn state_count_single_rule_at_least_two() {
    let g = GrammarBuilder::new("sc1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2, "S -> a needs at least 2 states");
}

#[test]
fn state_count_sequence_grows() {
    let g = GrammarBuilder::new("sc2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3,
        "S -> a b needs at least 3 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_alternatives_bounded() {
    let g = GrammarBuilder::new("sc3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(table.state_count <= 20, "2 alternatives shouldn't blow up");
}

#[test]
fn state_count_chain_bounded() {
    let g = GrammarBuilder::new("sc4")
        .token("x", "x")
        .rule("c", vec!["x"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(table.state_count <= 30);
}

#[test]
fn state_count_left_recursive_bounded() {
    let g = GrammarBuilder::new("sc5")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
    assert!(table.state_count <= 30);
}

#[test]
fn state_count_right_recursive_bounded() {
    let g = GrammarBuilder::new("sc6")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
    assert!(table.state_count <= 30);
}

#[test]
fn state_count_three_alternatives_bounded() {
    let g = GrammarBuilder::new("sc7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(table.state_count <= 30);
}

#[test]
fn state_count_nonzero() {
    let g = GrammarBuilder::new("sc8")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 1, "must have at least one state");
}

// ===========================================================================
// 4. All states have actions (7 tests)
// ===========================================================================

fn all_states_have_actions(table: &adze_glr_core::ParseTable) -> bool {
    (0..table.state_count).all(|st| state_has_any_action(table, StateId(st as u16)))
}

#[test]
fn all_states_active_single_token() {
    let g = GrammarBuilder::new("as1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(all_states_have_actions(&build_table(&g)));
}

#[test]
fn all_states_active_sequence() {
    let g = GrammarBuilder::new("as2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(all_states_have_actions(&build_table(&g)));
}

#[test]
fn all_states_active_alternatives() {
    let g = GrammarBuilder::new("as3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(all_states_have_actions(&build_table(&g)));
}

#[test]
fn all_states_active_chain() {
    let g = GrammarBuilder::new("as4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    assert!(all_states_have_actions(&build_table(&g)));
}

#[test]
fn all_states_active_left_recursive() {
    let g = GrammarBuilder::new("as5")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    assert!(all_states_have_actions(&build_table(&g)));
}

#[test]
fn all_states_active_right_recursive() {
    let g = GrammarBuilder::new("as6")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    assert!(all_states_have_actions(&build_table(&g)));
}

#[test]
fn all_states_active_multi_nt() {
    let g = GrammarBuilder::new("as7")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    assert!(all_states_have_actions(&build_table(&g)));
}

// ===========================================================================
// 5. Shift targets valid (8 tests)
// ===========================================================================

fn all_shift_targets_valid(table: &adze_glr_core::ParseTable) -> bool {
    collect_all_shifts(table)
        .iter()
        .all(|(_, _, target)| (target.0 as usize) < table.state_count)
}

#[test]
fn shift_valid_single_token() {
    let g = GrammarBuilder::new("sv1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn shift_valid_sequence() {
    let g = GrammarBuilder::new("sv2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn shift_valid_alternatives() {
    let g = GrammarBuilder::new("sv3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn shift_valid_chain() {
    let g = GrammarBuilder::new("sv4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn shift_valid_left_recursive() {
    let g = GrammarBuilder::new("sv5")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn shift_valid_right_recursive() {
    let g = GrammarBuilder::new("sv6")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn shift_valid_three_token_sequence() {
    let g = GrammarBuilder::new("sv7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn shift_valid_multi_nt() {
    let g = GrammarBuilder::new("sv8")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

// ===========================================================================
// 6. Reduce rule IDs valid (8 tests)
// ===========================================================================

fn all_reduce_rule_ids_valid(table: &adze_glr_core::ParseTable) -> bool {
    let rule_count = table.rules.len();
    collect_all_reduces(table)
        .iter()
        .all(|(_, _, rid)| (rid.0 as usize) < rule_count)
}

#[test]
fn reduce_valid_single_token() {
    let g = GrammarBuilder::new("rv1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn reduce_valid_sequence() {
    let g = GrammarBuilder::new("rv2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn reduce_valid_alternatives() {
    let g = GrammarBuilder::new("rv3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn reduce_valid_chain() {
    let g = GrammarBuilder::new("rv4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn reduce_valid_left_recursive() {
    let g = GrammarBuilder::new("rv5")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn reduce_valid_right_recursive() {
    let g = GrammarBuilder::new("rv6")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn reduce_valid_three_token() {
    let g = GrammarBuilder::new("rv7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn reduce_valid_multi_nt() {
    let g = GrammarBuilder::new("rv8")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

// ===========================================================================
// 7. Complex grammars (8 tests)
// ===========================================================================

#[test]
fn complex_expression_grammar_has_accept() {
    // E -> E + T | T; T -> T * F | F; F -> ( E ) | id
    let g = GrammarBuilder::new("cplx1")
        .token("id", r"\w+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .rule("factor", vec!["id"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 5);
}

#[test]
fn complex_expression_shifts_valid() {
    let g = GrammarBuilder::new("cplx2")
        .token("id", r"\w+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .rule("factor", vec!["id"])
        .start("expr")
        .build();
    assert!(all_shift_targets_valid(&build_table(&g)));
}

#[test]
fn complex_expression_reduces_valid() {
    let g = GrammarBuilder::new("cplx3")
        .token("id", r"\w+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .rule("factor", vec!["id"])
        .start("expr")
        .build();
    assert!(all_reduce_rule_ids_valid(&build_table(&g)));
}

#[test]
fn complex_nested_nonterminal_chain() {
    // A -> B; B -> C; C -> D; D -> x
    let g = GrammarBuilder::new("cplx4")
        .token("x", "x")
        .rule("d", vec!["x"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(all_shift_targets_valid(&table));
    assert!(all_reduce_rule_ids_valid(&table));
}

#[test]
fn complex_mutual_recursion_via_nonterminals() {
    // S -> A B; A -> x | A x; B -> y | B y
    let g = GrammarBuilder::new("cplx5")
        .token("x", "x")
        .token("y", "y")
        .rule("aaa", vec!["x"])
        .rule("aaa", vec!["aaa", "x"])
        .rule("bbb", vec!["y"])
        .rule("bbb", vec!["bbb", "y"])
        .rule("start", vec!["aaa", "bbb"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
}

#[test]
fn complex_multiple_start_alternatives() {
    // S -> a | b | c | d | e
    let g = GrammarBuilder::new("cplx6")
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
    assert!(all_states_have_actions(&table));
}

#[test]
fn complex_mixed_recursion_patterns() {
    // lst -> lst COMMA item | item; item -> id
    let g = GrammarBuilder::new("cplx7")
        .token("id", r"\w+")
        .token("COMMA", ",")
        .rule("item", vec!["id"])
        .rule("lst", vec!["lst", "COMMA", "item"])
        .rule("lst", vec!["item"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(all_shift_targets_valid(&table));
    assert!(all_reduce_rule_ids_valid(&table));
    assert!(table.state_count >= 3);
}

#[test]
fn complex_parenthesized_nesting() {
    // S -> ( S ) | x
    let g = GrammarBuilder::new("cplx8")
        .token("x", "x")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("start", vec!["LPAREN", "start", "RPAREN"])
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(all_shift_targets_valid(&table));
    assert!(all_reduce_rule_ids_valid(&table));
    assert!(all_states_have_actions(&table));
}
