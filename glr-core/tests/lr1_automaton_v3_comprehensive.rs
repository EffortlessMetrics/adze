#![cfg(feature = "test-api")]

//! Comprehensive V3 tests for LR(1) automaton construction.
//!
//! Categories:
//! 1. Simple grammars produce valid automaton (10 tests)
//! 2. State count is reasonable (5 tests)
//! 3. Accept action exists for valid grammars (8 tests)
//! 4. Shift actions lead to valid states (8 tests)
//! 5. Reduce actions reference valid rules (5 tests)
//! 6. Goto table consistency (5 tests)
//! 7. Determinism (5 tests)
//! 8. Complex grammars (5 tests)
//! 9. Edge cases (4 tests)

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

fn initial_shifts(table: &adze_glr_core::ParseTable, terminal: SymbolId) -> bool {
    table
        .actions(table.initial_state, terminal)
        .iter()
        .any(|a| matches!(a, Action::Shift(_)))
}

fn any_state_shifts(table: &adze_glr_core::ParseTable, terminal: SymbolId) -> bool {
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), terminal)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

fn collect_all_shifts(table: &adze_glr_core::ParseTable) -> Vec<(StateId, SymbolId, StateId)> {
    let mut out = Vec::new();
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        for (&sym, _) in &table.symbol_to_index {
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
        for (&sym, _) in &table.symbol_to_index {
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

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

// ===========================================================================
// 1. Simple grammars produce valid automaton (10 tests)
// ===========================================================================

#[test]
fn simple_single_token_grammar_valid() {
    let g = GrammarBuilder::new("s1")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 1);
    assert!(!table.rules.is_empty());
}

#[test]
fn simple_two_token_sequence_valid() {
    let g = GrammarBuilder::new("s2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
    assert!(!table.rules.is_empty());
}

#[test]
fn simple_two_alternatives_valid() {
    let g = GrammarBuilder::new("s3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
}

#[test]
fn simple_nonterminal_chain_valid() {
    let g = GrammarBuilder::new("s4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn simple_left_recursive_valid() {
    let g = GrammarBuilder::new("s5")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
}

#[test]
fn simple_right_recursive_valid() {
    let g = GrammarBuilder::new("s6")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
}

#[test]
fn simple_three_token_sequence_valid() {
    let g = GrammarBuilder::new("s7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 4);
}

#[test]
fn simple_multiple_nonterminals_valid() {
    let g = GrammarBuilder::new("s8")
        .token("x", "x")
        .token("y", "y")
        .rule("left", vec!["x"])
        .rule("right", vec!["y"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 3);
}

#[test]
fn simple_three_alternatives_valid() {
    let g = GrammarBuilder::new("s9")
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
}

#[test]
fn simple_nested_nonterminals_valid() {
    let g = GrammarBuilder::new("s10")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(!table.rules.is_empty());
}

// ===========================================================================
// 2. State count is reasonable (5 tests)
// ===========================================================================

#[test]
fn state_count_single_rule_bounded() {
    let g = GrammarBuilder::new("sc1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3 && table.state_count <= 10,
        "single rule: expected 3..=10 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_two_alt_bounded() {
    let g = GrammarBuilder::new("sc2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3 && table.state_count <= 12,
        "two alts: expected 3..=12 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_left_recursive_bounded() {
    let g = GrammarBuilder::new("sc3")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count <= 20,
        "left recursive: expected <= 20 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_long_sequence_bounded() {
    let g = GrammarBuilder::new("sc4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 5 && table.state_count <= 15,
        "4-token seq: expected 5..=15 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_multi_nonterminal_bounded() {
    let g = GrammarBuilder::new("sc5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("start", vec!["x", "y", "z"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count <= 30,
        "3-nonterminal chain: expected <= 30 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 3. Accept action exists for valid grammars (8 tests)
// ===========================================================================

#[test]
fn accept_single_terminal() {
    let g = GrammarBuilder::new("ac1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_two_alternatives() {
    let g = GrammarBuilder::new("ac2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_sequence() {
    let g = GrammarBuilder::new("ac3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_nonterminal_chain() {
    let g = GrammarBuilder::new("ac4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_left_recursive() {
    let g = GrammarBuilder::new("ac5")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_right_recursive() {
    let g = GrammarBuilder::new("ac6")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_nullable_grammar() {
    let g = GrammarBuilder::new("ac7")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(has_accept(&build_table(&g)));
}

#[test]
fn accept_arithmetic_grammar() {
    let g = GrammarBuilder::new("ac8")
        .token("n", "[0-9]+")
        .token("plus", "\\+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    assert!(has_accept(&build_table(&g)));
}

// ===========================================================================
// 4. Shift actions lead to valid states (8 tests)
// ===========================================================================

#[test]
fn shift_targets_within_bounds_single_rule() {
    let g = GrammarBuilder::new("sh1")
        .token("a", "a")
        .rule("start", vec!["a"])
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
fn shift_targets_within_bounds_sequence() {
    let g = GrammarBuilder::new("sh2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (_, _, target) in collect_all_shifts(&table) {
        assert!((target.0 as usize) < table.state_count);
    }
}

#[test]
fn shift_targets_within_bounds_left_recursive() {
    let g = GrammarBuilder::new("sh3")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    for (_, _, target) in collect_all_shifts(&table) {
        assert!((target.0 as usize) < table.state_count);
    }
}

#[test]
fn shift_exists_on_initial_for_single_token() {
    let g = GrammarBuilder::new("sh4")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(initial_shifts(&table, tok_id(&g, "x")));
}

#[test]
fn shift_exists_for_both_alternatives() {
    let g = GrammarBuilder::new("sh5")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(initial_shifts(&table, tok_id(&g, "a")));
    assert!(initial_shifts(&table, tok_id(&g, "b")));
}

#[test]
fn shift_first_token_of_sequence() {
    let g = GrammarBuilder::new("sh6")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(initial_shifts(&table, tok_id(&g, "a")));
}

#[test]
fn shift_second_token_exists_somewhere() {
    let g = GrammarBuilder::new("sh7")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_state_shifts(&table, tok_id(&g, "b")));
}

#[test]
fn shift_targets_within_bounds_arithmetic() {
    let g = GrammarBuilder::new("sh8")
        .token("n", "n")
        .token("plus", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    for (_, _, target) in collect_all_shifts(&table) {
        assert!(
            (target.0 as usize) < table.state_count,
            "shift target {} exceeds state_count {}",
            target.0,
            table.state_count
        );
    }
}

// ===========================================================================
// 5. Reduce actions reference valid rules (5 tests)
// ===========================================================================

#[test]
fn reduce_rule_ids_within_bounds_single() {
    let g = GrammarBuilder::new("rd1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (_, _, rid) in collect_all_reduces(&table) {
        assert!(
            (rid.0 as usize) < table.rules.len(),
            "reduce rule_id {} out of bounds (rules.len={})",
            rid.0,
            table.rules.len()
        );
    }
}

#[test]
fn reduce_rule_ids_within_bounds_alternatives() {
    let g = GrammarBuilder::new("rd2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (_, _, rid) in collect_all_reduces(&table) {
        assert!((rid.0 as usize) < table.rules.len());
    }
}

#[test]
fn reduce_rule_ids_within_bounds_left_recursive() {
    let g = GrammarBuilder::new("rd3")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    for (_, _, rid) in collect_all_reduces(&table) {
        assert!((rid.0 as usize) < table.rules.len());
    }
}

#[test]
fn reduce_rule_lhs_is_valid_nonterminal() {
    let g = GrammarBuilder::new("rd4")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (_, _, rid) in collect_all_reduces(&table) {
        let (lhs, _) = table.rule(rid);
        assert!(
            g.rule_names.contains_key(&lhs) || g.rules.contains_key(&lhs),
            "reduce lhs {:?} is not a known nonterminal",
            lhs
        );
    }
}

#[test]
fn reduce_exists_on_eof_for_simple_grammar() {
    let g = GrammarBuilder::new("rd5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let any_reduce = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Reduce(_)))
    });
    assert!(any_reduce, "some state must reduce on EOF");
}

// ===========================================================================
// 6. Goto table consistency (5 tests)
// ===========================================================================

#[test]
fn goto_start_symbol_exists_from_initial() {
    let g = GrammarBuilder::new("gt1")
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
fn goto_inner_nonterminal_exists() {
    let g = GrammarBuilder::new("gt2")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let exists = (0..table.state_count).any(|st| table.goto(StateId(st as u16), inner).is_some());
    assert!(exists, "goto for 'inner' must exist somewhere");
}

#[test]
fn goto_targets_within_bounds() {
    let g = GrammarBuilder::new("gt3")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for st in 0..table.state_count {
        for (&nt, _) in &g.rule_names {
            if let Some(target) = table.goto(StateId(st as u16), nt) {
                assert!(
                    (target.0 as usize) < table.state_count,
                    "goto target {} out of bounds (state_count={})",
                    target.0,
                    table.state_count
                );
            }
        }
    }
}

#[test]
fn goto_left_recursive_nonterminal() {
    let g = GrammarBuilder::new("gt4")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    let lst = nt_id(&g, "lst");
    let exists = (0..table.state_count).any(|st| table.goto(StateId(st as u16), lst).is_some());
    assert!(exists, "goto for left-recursive 'lst' must exist");
}

#[test]
fn goto_multiple_nonterminals_all_reachable() {
    let g = GrammarBuilder::new("gt5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("start", vec!["x", "y", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["x", "y", "start"] {
        let nt = nt_id(&g, name);
        let exists = (0..table.state_count).any(|st| table.goto(StateId(st as u16), nt).is_some());
        assert!(exists, "goto for '{}' must exist somewhere", name);
    }
}

// ===========================================================================
// 7. Determinism — same grammar yields same table (5 tests)
// ===========================================================================

fn grammar_single_token() -> Grammar {
    GrammarBuilder::new("det")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

#[test]
fn determinism_state_count_stable() {
    let g1 = grammar_single_token();
    let g2 = grammar_single_token();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn determinism_rule_count_stable() {
    let g1 = grammar_single_token();
    let g2 = grammar_single_token();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert_eq!(t1.rules.len(), t2.rules.len());
}

#[test]
fn determinism_eof_stable() {
    let g1 = grammar_single_token();
    let g2 = grammar_single_token();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert_eq!(t1.eof(), t2.eof());
}

#[test]
fn determinism_shift_count_stable() {
    let g1 = grammar_single_token();
    let g2 = grammar_single_token();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert_eq!(collect_all_shifts(&t1).len(), collect_all_shifts(&t2).len());
}

#[test]
fn determinism_reduce_count_stable() {
    let g1 = grammar_single_token();
    let g2 = grammar_single_token();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert_eq!(
        collect_all_reduces(&t1).len(),
        collect_all_reduces(&t2).len()
    );
}

// ===========================================================================
// 8. Complex grammars (5 tests)
// ===========================================================================

#[test]
fn complex_arithmetic_with_mul() {
    let g = GrammarBuilder::new("arith2")
        .token("n", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("factor", vec!["n"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("term", vec!["factor"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(
        table.state_count >= 6,
        "arith needs >= 6 states, got {}",
        table.state_count
    );
}

#[test]
fn complex_nested_parentheses() {
    let g = GrammarBuilder::new("paren")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .token("id", "[a-z]+")
        .rule("atom", vec!["id"])
        .rule("atom", vec!["lparen", "expr", "rparen"])
        .rule("expr", vec!["atom"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(initial_shifts(&table, tok_id(&g, "id")));
    assert!(initial_shifts(&table, tok_id(&g, "lparen")));
}

#[test]
fn complex_mutual_recursion_chain() {
    // A → B x, B → A y | z  (mutual recursion via chain)
    let g = GrammarBuilder::new("mutual")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("bval", vec!["aval", "y"])
        .rule("bval", vec!["z"])
        .rule("aval", vec!["bval", "x"])
        .start("aval")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
}

#[test]
fn complex_multiple_levels_of_nesting() {
    let g = GrammarBuilder::new("deep")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("d4", vec!["d"])
        .rule("c3", vec!["c", "d4"])
        .rule("b2", vec!["b", "c3"])
        .rule("start", vec!["a", "b2"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 5);
}

#[test]
fn complex_ambiguous_grammar_builds() {
    // expr → expr '+' expr | n  (inherently ambiguous — GLR handles it)
    let g = GrammarBuilder::new("ambig2")
        .token("n", "n")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
    // Verify some shift/reduce exists (GLR conflict expected)
    assert!(!collect_all_shifts(&table).is_empty());
    assert!(!collect_all_reduces(&table).is_empty());
}

// ===========================================================================
// 9. Edge cases (4 tests)
// ===========================================================================

#[test]
fn edge_nullable_start_only() {
    // start → ε
    let g = GrammarBuilder::new("eps_only")
        .rule("start", vec![])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn edge_nullable_with_nonterminal() {
    // inner → ε, start → inner
    let g = GrammarBuilder::new("eps_chain")
        .rule("inner", vec![])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn edge_eof_not_in_token_set() {
    let g = GrammarBuilder::new("eof_check")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for (tok_sym, _) in &g.tokens {
        assert_ne!(
            eof, *tok_sym,
            "EOF symbol must not collide with any user token"
        );
    }
}

#[test]
fn edge_single_nullable_among_alternatives() {
    // start → ε | a b  — one nullable, one non-nullable alternative
    let g = GrammarBuilder::new("mixed_eps")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec![])
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // Must still be able to shift 'a' from initial state
    assert!(
        initial_shifts(&table, tok_id(&g, "a")),
        "must shift 'a' even though ε alternative exists"
    );
}
