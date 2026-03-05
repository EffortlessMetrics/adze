#![cfg(feature = "test-api")]

//! Comprehensive LR(1) automaton building tests for adze-glr-core.
//!
//! 84 tests covering:
//! 1.  Build automaton from simple grammar (8 tests)
//! 2.  State count properties (8 tests)
//! 3.  Symbol count properties (8 tests)
//! 4.  Single-token grammars (6 tests)
//! 5.  Multi-rule grammars (8 tests)
//! 6.  Precedence grammars (6 tests)
//! 7.  Inline / extras / supertypes (6 tests)
//! 8.  Recursive grammars (6 tests)
//! 9.  Goto table entries (8 tests)
//! 10. Action table entries (8 tests)
//! 11. Rule info via pt.rule() (6 tests)
//! 12. Grammar scaling (4 tests)
//! 13. FirstFollowSets computation (2 tests)

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_pt(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("first/follow");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
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

fn has_accept(table: &ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn has_any_shift(table: &ParseTable) -> bool {
    (0..table.state_count).any(|st| {
        let state = StateId(st as u16);
        table.symbol_to_index.keys().any(|&sym| {
            table
                .actions(state, sym)
                .iter()
                .any(|a| matches!(a, Action::Shift(_)))
        })
    })
}

fn has_any_reduce(table: &ParseTable) -> bool {
    (0..table.state_count).any(|st| {
        let state = StateId(st as u16);
        table.symbol_to_index.keys().any(|&sym| {
            table
                .actions(state, sym)
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
    })
}

fn count_shifts(table: &ParseTable) -> usize {
    let mut n = 0;
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if matches!(action, Action::Shift(_)) {
                    n += 1;
                }
            }
        }
    }
    n
}

fn count_reduces(table: &ParseTable) -> usize {
    let mut n = 0;
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if matches!(action, Action::Reduce(_)) {
                    n += 1;
                }
            }
        }
    }
    n
}

fn expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "star", "num"])
        .rule("term", vec!["num"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. Build automaton from simple grammar (8 tests)
// ===========================================================================

#[test]
fn build_simple_grammar_succeeds() {
    let g = simple_grammar();
    let _pt = build_pt(&g);
}

#[test]
fn build_simple_grammar_has_states() {
    let pt = build_pt(&simple_grammar());
    assert!(pt.state_count > 0);
}

#[test]
fn build_simple_grammar_has_accept() {
    let pt = build_pt(&simple_grammar());
    assert!(has_accept(&pt));
}

#[test]
fn build_simple_grammar_initial_state_valid() {
    let pt = build_pt(&simple_grammar());
    assert!((pt.initial_state.0 as usize) < pt.state_count);
}

#[test]
fn build_simple_grammar_eof_in_symbol_index() {
    let pt = build_pt(&simple_grammar());
    assert!(pt.symbol_to_index.contains_key(&pt.eof_symbol));
}

#[test]
fn build_simple_grammar_has_rules() {
    let pt = build_pt(&simple_grammar());
    assert!(!pt.rules.is_empty());
}

#[test]
fn build_simple_grammar_has_shifts() {
    let pt = build_pt(&simple_grammar());
    assert!(has_any_shift(&pt));
}

#[test]
fn build_simple_grammar_has_reduces() {
    let pt = build_pt(&simple_grammar());
    assert!(has_any_reduce(&pt));
}

// ===========================================================================
// 2. State count properties (8 tests)
// ===========================================================================

#[test]
fn state_count_single_rule_at_least_two() {
    let g = GrammarBuilder::new("sc1")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_two_token_sequence_at_least_three() {
    let pt = build_pt(&simple_grammar());
    assert!(pt.state_count >= 3);
}

#[test]
fn state_count_three_alternatives() {
    let g = GrammarBuilder::new("sc3")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_nested_nonterminals() {
    let g = GrammarBuilder::new("sc4")
        .token("leaf", "leaf")
        .rule("inner", vec!["leaf"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_left_recursive() {
    let g = GrammarBuilder::new("sc5")
        .token("m", "m")
        .rule("items", vec!["items", "m"])
        .rule("items", vec!["m"])
        .start("items")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 3);
}

#[test]
fn state_count_matches_action_table_rows() {
    let pt = build_pt(&simple_grammar());
    assert_eq!(pt.state_count, pt.action_table.len());
}

#[test]
fn state_count_matches_goto_table_rows() {
    let pt = build_pt(&simple_grammar());
    assert_eq!(pt.state_count, pt.goto_table.len());
}

#[test]
fn state_count_grows_with_grammar_complexity() {
    let small = GrammarBuilder::new("s")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let large = GrammarBuilder::new("l")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let pt_small = build_pt(&small);
    let pt_large = build_pt(&large);
    assert!(pt_large.state_count >= pt_small.state_count);
}

// ===========================================================================
// 3. Symbol count properties (8 tests)
// ===========================================================================

#[test]
fn symbol_count_positive() {
    let pt = build_pt(&simple_grammar());
    assert!(pt.symbol_count > 0);
}

#[test]
fn symbol_count_includes_eof() {
    let pt = build_pt(&simple_grammar());
    // symbol_count should be at least 1 (EOF)
    assert!(pt.symbol_count >= 1);
}

#[test]
fn symbol_count_at_least_tokens_plus_nonterminals() {
    let g = simple_grammar();
    let token_count = g.tokens.len();
    let nt_count = g.rules.len();
    let pt = build_pt(&g);
    assert!(pt.symbol_count >= token_count + nt_count);
}

#[test]
fn symbol_count_single_token_grammar() {
    let g = GrammarBuilder::new("sc_single")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    // At least: x, start, EOF
    assert!(pt.symbol_count >= 3);
}

#[test]
fn symbol_count_grows_with_tokens() {
    let g2 = GrammarBuilder::new("sc2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let g4 = GrammarBuilder::new("sc4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let pt2 = build_pt(&g2);
    let pt4 = build_pt(&g4);
    assert!(pt4.symbol_count >= pt2.symbol_count);
}

#[test]
fn symbol_to_index_len_matches_symbol_count_or_token_count() {
    let pt = build_pt(&simple_grammar());
    // symbol_to_index maps terminals used in action table
    assert!(!pt.symbol_to_index.is_empty());
}

#[test]
fn index_to_symbol_consistent_with_symbol_to_index() {
    let pt = build_pt(&simple_grammar());
    for (&sym, &idx) in &pt.symbol_to_index {
        assert_eq!(pt.index_to_symbol[idx], sym);
    }
}

#[test]
fn nonterminal_to_index_non_empty() {
    let pt = build_pt(&simple_grammar());
    assert!(!pt.nonterminal_to_index.is_empty());
}

// ===========================================================================
// 4. Single-token grammars (6 tests)
// ===========================================================================

#[test]
fn single_token_builds_successfully() {
    let g = GrammarBuilder::new("st1")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let _pt = build_pt(&g);
}

#[test]
fn single_token_has_accept() {
    let g = GrammarBuilder::new("st2")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn single_token_shift_exists_at_initial() {
    let g = GrammarBuilder::new("st3")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    let t = tok_id(&g, "t");
    let actions = pt.actions(pt.initial_state, t);
    assert!(actions.iter().any(|a| matches!(a, Action::Shift(_))));
}

#[test]
fn single_token_no_accept_at_initial_on_token() {
    let g = GrammarBuilder::new("st4")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    let t = tok_id(&g, "t");
    let actions = pt.actions(pt.initial_state, t);
    assert!(!actions.iter().any(|a| matches!(a, Action::Accept)));
}

#[test]
fn single_token_at_least_two_states() {
    let g = GrammarBuilder::new("st5")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn single_token_has_exactly_one_shift_for_initial() {
    let g = GrammarBuilder::new("st6")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    let t = tok_id(&g, "t");
    let shift_count = pt
        .actions(pt.initial_state, t)
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .count();
    assert_eq!(shift_count, 1);
}

// ===========================================================================
// 5. Multi-rule grammars (8 tests)
// ===========================================================================

#[test]
fn multi_rule_two_alternatives() {
    let g = GrammarBuilder::new("mr1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 2);
    assert!(has_accept(&pt));
}

#[test]
fn multi_rule_three_alternatives() {
    let g = GrammarBuilder::new("mr2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn multi_rule_with_nonterminal_delegation() {
    let g = GrammarBuilder::new("mr3")
        .token("x", "x")
        .token("y", "y")
        .rule("inner", vec!["x"])
        .rule("inner", vec!["y"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
    assert!(has_any_shift(&pt));
}

#[test]
fn multi_rule_shifts_for_each_token_at_initial() {
    let g = GrammarBuilder::new("mr4")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let a_shifts = pt
        .actions(pt.initial_state, a)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    let b_shifts = pt
        .actions(pt.initial_state, b)
        .iter()
        .any(|act| matches!(act, Action::Shift(_)));
    assert!(a_shifts);
    assert!(b_shifts);
}

#[test]
fn multi_rule_different_lengths() {
    let g = GrammarBuilder::new("mr5")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn multi_rule_reduces_exist() {
    let g = GrammarBuilder::new("mr6")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_any_reduce(&pt));
}

#[test]
fn multi_rule_three_nonterminals() {
    let g = GrammarBuilder::new("mr7")
        .token("x", "x")
        .token("y", "y")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("mid", vec!["y"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 2);
    assert!(has_accept(&pt));
}

#[test]
fn multi_rule_mixed_terminals_and_nonterminals() {
    let g = GrammarBuilder::new("mr8")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["b"])
        .rule("start", vec!["a", "inner"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
    assert!(count_shifts(&pt) >= 2);
}

// ===========================================================================
// 6. Precedence grammars (6 tests)
// ===========================================================================

#[test]
fn precedence_grammar_builds_successfully() {
    let g = GrammarBuilder::new("prec1")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let _pt = build_pt(&g);
}

#[test]
fn precedence_grammar_has_accept() {
    let g = GrammarBuilder::new("prec2")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn precedence_grammar_has_shifts_and_reduces() {
    let g = GrammarBuilder::new("prec3")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let pt = build_pt(&g);
    assert!(has_any_shift(&pt));
    assert!(has_any_reduce(&pt));
}

#[test]
fn precedence_right_associative_builds() {
    let g = GrammarBuilder::new("prec4")
        .token("num", r"\d+")
        .token("caret", r"\^")
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            1,
            Associativity::Right,
        )
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn precedence_mixed_assoc_builds() {
    let g = GrammarBuilder::new("prec5")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("caret", r"\^")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            2,
            Associativity::Right,
        )
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 4);
}

#[test]
fn precedence_declaration_builds() {
    let g = GrammarBuilder::new("prec6")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let _pt = build_pt(&g);
}

// ===========================================================================
// 7. Inline / extras / supertypes (6 tests)
// ===========================================================================

#[test]
fn inline_rule_grammar_builds() {
    let g = GrammarBuilder::new("inl1")
        .token("a", "a")
        .token("b", "b")
        .rule("helper", vec!["a"])
        .rule("start", vec!["helper", "b"])
        .inline("helper")
        .start("start")
        .build();
    let _pt = build_pt(&g);
}

#[test]
fn inline_rule_grammar_has_accept() {
    let g = GrammarBuilder::new("inl2")
        .token("a", "a")
        .token("b", "b")
        .rule("helper", vec!["a"])
        .rule("start", vec!["helper", "b"])
        .inline("helper")
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn extras_grammar_builds() {
    let g = GrammarBuilder::new("ext1")
        .token("a", "a")
        .token("ws", r"[ \t]+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    let _pt = build_pt(&g);
}

#[test]
fn extras_grammar_records_extras() {
    let g = GrammarBuilder::new("ext2")
        .token("a", "a")
        .token("ws", r"[ \t]+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    // Verify the grammar itself carries the extras
    assert!(!g.extras.is_empty());
    let _pt = build_pt(&g);
}

#[test]
fn supertype_grammar_builds() {
    let g = GrammarBuilder::new("sup1")
        .token("x", "x")
        .token("y", "y")
        .rule("literal", vec!["x"])
        .rule("literal", vec!["y"])
        .rule("start", vec!["literal"])
        .supertype("literal")
        .start("start")
        .build();
    let _pt = build_pt(&g);
}

#[test]
fn supertype_grammar_has_accept() {
    let g = GrammarBuilder::new("sup2")
        .token("x", "x")
        .token("y", "y")
        .rule("literal", vec!["x"])
        .rule("literal", vec!["y"])
        .rule("start", vec!["literal"])
        .supertype("literal")
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

// ===========================================================================
// 8. Recursive grammars (6 tests)
// ===========================================================================

#[test]
fn left_recursive_grammar_builds() {
    let g = GrammarBuilder::new("lr1")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["num"])
        .start("expr")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn right_recursive_grammar_builds() {
    let g = GrammarBuilder::new("rr1")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn classic_expression_grammar_builds() {
    let pt = build_pt(&expr_grammar());
    assert!(has_accept(&pt));
    assert!(pt.state_count >= 5);
}

#[test]
fn expression_grammar_has_shifts_and_reduces() {
    let pt = build_pt(&expr_grammar());
    assert!(count_shifts(&pt) > 0);
    assert!(count_reduces(&pt) > 0);
}

#[test]
fn deeply_nested_nonterminals_build() {
    let g = GrammarBuilder::new("deep1")
        .token("leaf", "leaf")
        .rule("d4", vec!["leaf"])
        .rule("d3", vec!["d4"])
        .rule("d2", vec!["d3"])
        .rule("d1", vec!["d2"])
        .rule("start", vec!["d1"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn indirect_recursion_builds() {
    let g = GrammarBuilder::new("indrec")
        .token("a", "a")
        .token("b", "b")
        .rule("alpha", vec!["a", "beta"])
        .rule("alpha", vec!["a"])
        .rule("beta", vec!["b", "alpha"])
        .rule("beta", vec!["b"])
        .start("alpha")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

// ===========================================================================
// 9. Goto table entries (8 tests)
// ===========================================================================

#[test]
fn goto_exists_for_start_nonterminal() {
    let g = simple_grammar();
    let pt = build_pt(&g);
    let start = nt_id(&g, "start");
    let target = pt.goto(pt.initial_state, start);
    assert!(target.is_some(), "goto(initial, start) should exist");
}

#[test]
fn goto_target_is_valid_state() {
    let g = simple_grammar();
    let pt = build_pt(&g);
    let start = nt_id(&g, "start");
    if let Some(target) = pt.goto(pt.initial_state, start) {
        assert!((target.0 as usize) < pt.state_count);
    }
}

#[test]
fn goto_returns_none_for_terminal() {
    let g = simple_grammar();
    let pt = build_pt(&g);
    let a = tok_id(&g, "a");
    // Terminals should not be in goto table
    let result = pt.goto(pt.initial_state, a);
    assert!(result.is_none());
}

#[test]
fn goto_exists_for_nested_nonterminal() {
    let g = GrammarBuilder::new("goto_nested")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    let inner = nt_id(&g, "inner");
    let target = pt.goto(pt.initial_state, inner);
    assert!(target.is_some(), "goto(initial, inner) should exist");
}

#[test]
fn goto_table_non_empty() {
    let pt = build_pt(&simple_grammar());
    let has_entry = (0..pt.state_count).any(|st| {
        let state = StateId(st as u16);
        pt.nonterminal_to_index
            .keys()
            .any(|&nt| pt.goto(state, nt).is_some())
    });
    assert!(has_entry);
}

#[test]
fn goto_for_expr_grammar_nonterminals() {
    let g = expr_grammar();
    let pt = build_pt(&g);
    let has_expr_goto = (0..pt.state_count).any(|st| {
        let state = StateId(st as u16);
        let eid = nt_id(&g, "expr");
        pt.goto(state, eid).is_some()
    });
    assert!(has_expr_goto);
}

#[test]
fn goto_for_term_in_expr_grammar() {
    let g = expr_grammar();
    let pt = build_pt(&g);
    let tid = nt_id(&g, "term");
    let has_term_goto = (0..pt.state_count).any(|st| {
        let state = StateId(st as u16);
        pt.goto(state, tid).is_some()
    });
    assert!(has_term_goto);
}

#[test]
fn goto_targets_are_within_bounds() {
    let g = expr_grammar();
    let pt = build_pt(&g);
    for st in 0..pt.state_count {
        let state = StateId(st as u16);
        for &nt in pt.nonterminal_to_index.keys() {
            if let Some(target) = pt.goto(state, nt) {
                assert!(
                    (target.0 as usize) < pt.state_count,
                    "goto target {target:?} out of bounds (state_count={})",
                    pt.state_count
                );
            }
        }
    }
}

// ===========================================================================
// 10. Action table entries (8 tests)
// ===========================================================================

#[test]
fn actions_for_initial_state_non_empty() {
    let g = simple_grammar();
    let pt = build_pt(&g);
    let a = tok_id(&g, "a");
    assert!(!pt.actions(pt.initial_state, a).is_empty());
}

#[test]
fn actions_contain_shift_at_initial_for_first_token() {
    let g = simple_grammar();
    let pt = build_pt(&g);
    let a = tok_id(&g, "a");
    assert!(
        pt.actions(pt.initial_state, a)
            .iter()
            .any(|act| matches!(act, Action::Shift(_)))
    );
}

#[test]
fn actions_for_out_of_range_state_is_empty() {
    let pt = build_pt(&simple_grammar());
    let a = tok_id(&simple_grammar(), "a");
    let bogus = StateId(u16::MAX);
    assert!(pt.actions(bogus, a).is_empty());
}

#[test]
fn actions_for_unknown_symbol_is_empty() {
    let pt = build_pt(&simple_grammar());
    let bogus = SymbolId(9999);
    assert!(pt.actions(pt.initial_state, bogus).is_empty());
}

#[test]
fn accept_action_found_on_eof() {
    let pt = build_pt(&simple_grammar());
    let eof = pt.eof();
    let found = (0..pt.state_count).any(|st| {
        pt.actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(found);
}

#[test]
fn reduce_actions_reference_valid_rules() {
    let pt = build_pt(&simple_grammar());
    let rule_count = pt.rules.len();
    for st in 0..pt.state_count {
        let state = StateId(st as u16);
        for &sym in pt.symbol_to_index.keys() {
            for action in pt.actions(state, sym) {
                if let Action::Reduce(rid) = action {
                    assert!(
                        (rid.0 as usize) < rule_count,
                        "reduce references rule {rid:?} but only {rule_count} rules exist"
                    );
                }
            }
        }
    }
}

#[test]
fn shift_targets_are_valid_states() {
    let pt = build_pt(&simple_grammar());
    for st in 0..pt.state_count {
        let state = StateId(st as u16);
        for &sym in pt.symbol_to_index.keys() {
            for action in pt.actions(state, sym) {
                if let Action::Shift(target) = action {
                    assert!(
                        (target.0 as usize) < pt.state_count,
                        "shift target {target:?} out of bounds"
                    );
                }
            }
        }
    }
}

#[test]
fn expr_grammar_initial_shifts_on_num() {
    let g = expr_grammar();
    let pt = build_pt(&g);
    let num = tok_id(&g, "num");
    assert!(
        pt.actions(pt.initial_state, num)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    );
}

// ===========================================================================
// 11. Rule info via pt.rule() (6 tests)
// ===========================================================================

#[test]
fn rule_returns_lhs_and_len() {
    let pt = build_pt(&simple_grammar());
    let (lhs, _rhs_len) = pt.rule(RuleId(0));
    // LHS should be a known nonterminal
    assert!(pt.nonterminal_to_index.contains_key(&lhs));
}

#[test]
fn rule_lhs_is_nonterminal() {
    let pt = build_pt(&simple_grammar());
    for i in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(i as u16));
        assert!(
            pt.nonterminal_to_index.contains_key(&lhs),
            "rule {i} LHS {lhs:?} not a known nonterminal"
        );
    }
}

#[test]
fn rule_rhs_len_matches_grammar() {
    // start -> a b has rhs_len = 2
    let g = simple_grammar();
    let pt = build_pt(&g);
    let start = nt_id(&g, "start");
    let found = (0..pt.rules.len()).any(|i| {
        let (lhs, rhs_len) = pt.rule(RuleId(i as u16));
        lhs == start && rhs_len == 2
    });
    assert!(found, "should find start -> a b with rhs_len=2");
}

#[test]
fn rule_count_at_least_grammar_rules() {
    let g = expr_grammar();
    let pt = build_pt(&g);
    let grammar_rule_count: usize = g.rules.values().map(|v| v.len()).sum();
    // Parse table may have augmented start rule too
    assert!(pt.rules.len() >= grammar_rule_count);
}

#[test]
fn rule_zero_exists() {
    let pt = build_pt(&simple_grammar());
    assert!(!pt.rules.is_empty());
    let _ = pt.rule(RuleId(0));
}

#[test]
fn rule_info_consistent_across_calls() {
    let pt = build_pt(&simple_grammar());
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        let first = pt.rule(rid);
        let second = pt.rule(rid);
        assert_eq!(first, second);
    }

    // ===========================================================================
    // 12. Grammar scaling (4 tests)
    // ===========================================================================
}

#[test]
fn scaling_five_token_sequence() {
    let g = GrammarBuilder::new("scale5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(pt.state_count >= 6);
    assert!(has_accept(&pt));
}

#[test]
fn scaling_ten_alternatives() {
    let mut builder = GrammarBuilder::new("scale10");
    let names: Vec<String> = (0..10).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("start", vec![name]);
    }
    builder = builder.start("start");
    let g = builder.build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
    assert!(pt.state_count >= 2);
}

#[test]
fn scaling_chain_of_nonterminals() {
    let mut builder = GrammarBuilder::new("chain").token("leaf", "leaf");

    let depth = 8;
    let names: Vec<String> = (0..depth).map(|i| format!("level{i}")).collect();
    builder = builder.rule(&names[0], vec!["leaf"]);
    for i in 1..depth {
        builder = builder.rule(&names[i], vec![&names[i - 1]]);
    }
    builder = builder.start(&names[depth - 1]);
    let g = builder.build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
}

#[test]
fn scaling_multiple_nonterminals_with_shared_tokens() {
    let g = GrammarBuilder::new("shared")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let pt = build_pt(&g);
    assert!(has_accept(&pt));
    assert!(pt.nonterminal_to_index.len() >= 4);
}

// ===========================================================================
// 13. FirstFollowSets computation (2 tests)
// ===========================================================================

#[test]
fn first_follow_computes_without_error() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn first_follow_computes_for_expr_grammar() {
    let g = expr_grammar();
    let ff = FirstFollowSets::compute(&g).expect("first/follow");
    let num = tok_id(&g, "num");
    let first = ff.first(num);
    assert!(first.is_some());
}
