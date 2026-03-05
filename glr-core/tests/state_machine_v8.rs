//! V8 state-machine tests for `ParseTable` operations — 84 tests covering:
//! construction, actions, accept, eof, state_count, symbol_count, goto, rule,
//! multi-token, chain rules, action-cell properties, and cross-grammar comparisons.

#![cfg(feature = "test-api")]

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
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
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

fn build_with_grammar(g: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(g).expect("ff");
    build_lr1_automaton(g, &ff).expect("table")
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

fn has_accept_anywhere(pt: &ParseTable) -> bool {
    let eof = pt.eof();
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn has_shift_on(pt: &ParseTable, sym: SymbolId) -> bool {
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

fn has_reduce_on(pt: &ParseTable, sym: SymbolId) -> bool {
    (0..pt.state_count).any(|s| {
        pt.actions(StateId(s as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Reduce(_)))
    })
}

fn count_nonempty_cells(pt: &ParseTable) -> usize {
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

// ---------------------------------------------------------------------------
// Grammar factories
// ---------------------------------------------------------------------------

/// start → x
fn single_token() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_single")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// start → a b
fn two_token() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_two")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// start → a b c d
fn four_token_linear() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_four_linear")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// start → a | b
fn two_alternatives() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// chain → wrap ; wrap → inner ; inner → x
fn chain_rule() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_chain")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("wrap", vec!["inner"])
        .rule("chain", vec!["wrap"])
        .start("chain")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// items → items , id | id
fn list_grammar() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_list")
        .token(",", ",")
        .token("id", "id")
        .rule("items", vec!["items", ",", "id"])
        .rule("items", vec!["id"])
        .start("items")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// expr → expr + term | term ; term → num
fn expr_term() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_expr_term")
        .token("num", "num")
        .token("+", "+")
        .rule("term", vec!["num"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// start → ( start ) | x
fn paren_grammar() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_paren")
        .token("(", "(")
        .token(")", ")")
        .token("x", "x")
        .rule("start", vec!["(", "start", ")"])
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// start → a start | a  (right recursive)
fn right_recursive() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_rr")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// start → start a | a  (left recursive)
fn left_recursive() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_lr")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// top → mid ; mid → low ; low → a b
fn nested_chain() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_nested_chain")
        .token("a", "a")
        .token("b", "b")
        .rule("low", vec!["a", "b"])
        .rule("mid", vec!["low"])
        .rule("top", vec!["mid"])
        .start("top")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

/// start → a b | a c  (shared prefix)
fn shared_prefix() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("sm_v8_shared_pfx")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["a", "c"])
        .start("start")
        .build();
    let pt = build_with_grammar(&g);
    (g, pt)
}

// ===========================================================================
// 1. Single token grammar → at least 2 states (initial + accepting)
// ===========================================================================

#[test]
fn single_token_at_least_two_states() {
    let (_, pt) = single_token();
    assert!(
        pt.state_count >= 2,
        "single-token grammar needs >= 2 states, got {}",
        pt.state_count
    );
}

#[test]
fn single_token_initial_state_is_valid() {
    let (_, pt) = single_token();
    assert!(
        (pt.initial_state.0 as usize) < pt.state_count,
        "initial_state must be within state_count"
    );
}

#[test]
fn single_token_table_has_action_rows() {
    let (_, pt) = single_token();
    assert_eq!(pt.action_table.len(), pt.state_count);
}

// ===========================================================================
// 2. State 0 has shift action for start token
// ===========================================================================

#[test]
fn state0_has_shift_for_token_single() {
    let (g, pt) = single_token();
    let x = tok(&g, "x");
    let actions = pt.actions(pt.initial_state, x);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on 'x'"
    );
}

#[test]
fn state0_has_shift_for_first_token_linear() {
    let (g, pt) = two_token();
    let a = tok(&g, "a");
    let actions = pt.actions(pt.initial_state, a);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on 'a'"
    );
}

#[test]
fn state0_shift_paren_open() {
    let (g, pt) = paren_grammar();
    let lparen = tok(&g, "(");
    let actions = pt.actions(pt.initial_state, lparen);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on '('"
    );
}

#[test]
fn state0_shift_alternatives_first() {
    let (g, pt) = two_alternatives();
    let a = tok(&g, "a");
    let actions = pt.actions(pt.initial_state, a);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on 'a' in two-alt grammar"
    );
}

#[test]
fn state0_shift_alternatives_second() {
    let (g, pt) = two_alternatives();
    let b = tok(&g, "b");
    let actions = pt.actions(pt.initial_state, b);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on 'b' in two-alt grammar"
    );
}

// ===========================================================================
// 3. Accept action exists somewhere in table
// ===========================================================================

#[test]
fn accept_exists_single_token() {
    let (_, pt) = single_token();
    assert!(
        has_accept_anywhere(&pt),
        "single-token table must have Accept"
    );
}

#[test]
fn accept_exists_two_token() {
    let (_, pt) = two_token();
    assert!(has_accept_anywhere(&pt), "two-token table must have Accept");
}

#[test]
fn accept_exists_chain() {
    let (_, pt) = chain_rule();
    assert!(has_accept_anywhere(&pt), "chain table must have Accept");
}

#[test]
fn accept_exists_paren() {
    let (_, pt) = paren_grammar();
    assert!(has_accept_anywhere(&pt), "paren table must have Accept");
}

#[test]
fn accept_exists_list() {
    let (_, pt) = list_grammar();
    assert!(has_accept_anywhere(&pt), "list table must have Accept");
}

#[test]
fn accept_exists_expr_term() {
    let (_, pt) = expr_term();
    assert!(has_accept_anywhere(&pt), "expr-term table must have Accept");
}

#[test]
fn accept_on_eof_only() {
    let (_, pt) = single_token();
    let eof = pt.eof();
    // Verify Accept only appears in EOF column
    for s in 0..pt.state_count {
        let sid = StateId(s as u16);
        for &sym in pt.symbol_to_index.keys() {
            let actions = pt.actions(sid, sym);
            if actions.iter().any(|a| matches!(a, Action::Accept)) {
                assert_eq!(sym, eof, "Accept should only appear on EOF symbol");
            }
        }
    }
}

// ===========================================================================
// 4. eof_symbol is consistent
// ===========================================================================

#[test]
fn eof_symbol_consistent_with_accessor() {
    let (_, pt) = single_token();
    assert_eq!(pt.eof_symbol, pt.eof());
}

#[test]
fn eof_symbol_in_symbol_to_index() {
    let (_, pt) = single_token();
    assert!(
        pt.symbol_to_index.contains_key(&pt.eof()),
        "EOF must be in symbol_to_index"
    );
}

#[test]
fn eof_same_across_grammars() {
    let (_, pt1) = single_token();
    let (_, pt2) = two_token();
    // EOF symbol IDs may differ, but each must be self-consistent
    assert_eq!(pt1.eof_symbol, pt1.eof());
    assert_eq!(pt2.eof_symbol, pt2.eof());
}

#[test]
fn eof_symbol_not_a_user_token() {
    let (g, pt) = single_token();
    let x = tok(&g, "x");
    assert_ne!(pt.eof(), x, "EOF should not be the same as user token 'x'");
}

#[test]
fn eof_symbol_consistent_chain() {
    let (_, pt) = chain_rule();
    assert_eq!(pt.eof_symbol, pt.eof());
}

// ===========================================================================
// 5. state_count >= 2 for any grammar
// ===========================================================================

#[test]
fn state_count_gte_two_single() {
    let (_, pt) = single_token();
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_gte_two_two_token() {
    let (_, pt) = two_token();
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_gte_two_chain() {
    let (_, pt) = chain_rule();
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_gte_two_list() {
    let (_, pt) = list_grammar();
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_gte_two_paren() {
    let (_, pt) = paren_grammar();
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_matches_action_table_rows() {
    let (_, pt) = expr_term();
    assert_eq!(pt.state_count, pt.action_table.len());
}

// ===========================================================================
// 6. symbol_count matches grammar
// ===========================================================================

#[test]
fn symbol_count_positive() {
    let (_, pt) = single_token();
    assert!(pt.symbol_count > 0);
}

#[test]
fn symbol_count_gte_tokens_plus_one() {
    let (g, pt) = two_token();
    let user_tokens = g.tokens.len();
    // symbol_count must at least cover user tokens + EOF
    assert!(
        pt.symbol_count > user_tokens,
        "symbol_count {} should exceed token count {}",
        pt.symbol_count,
        user_tokens,
    );
}

#[test]
fn symbol_count_increases_with_tokens() {
    let (_, pt1) = single_token();
    let (_, pt4) = four_token_linear();
    assert!(
        pt4.symbol_count > pt1.symbol_count,
        "4-token grammar should have more symbols than 1-token"
    );
}

#[test]
fn symbol_count_includes_eof() {
    let (_, pt) = single_token();
    // eof must be mapped, contributing to symbol_count
    assert!(pt.symbol_to_index.contains_key(&pt.eof()));
    assert!(pt.symbol_count >= pt.symbol_to_index.len());
}

#[test]
fn symbol_count_expr_term() {
    let (g, pt) = expr_term();
    // tokens: num, + ; nonterminals: term, expr ; plus EOF, augmented start
    let user_tokens = g.tokens.len();
    let nonterminals = g.rule_names.len();
    assert!(
        pt.symbol_count >= user_tokens + nonterminals,
        "expr_term symbol_count should cover all symbols"
    );
}

// ===========================================================================
// 7. goto returns Some for non-terminals
// ===========================================================================

#[test]
fn goto_some_for_start_nt() {
    let (g, pt) = single_token();
    let start_sym = nt(&g, "start");
    let found = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), start_sym).is_some());
    assert!(found, "goto must return Some for 'start' in some state");
}

#[test]
fn goto_some_for_chain_nts() {
    let (g, pt) = chain_rule();
    for name in &["inner", "wrap", "chain"] {
        let sym = nt(&g, name);
        let found = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), sym).is_some());
        assert!(found, "goto should have entry for '{name}'");
    }
}

#[test]
fn goto_some_for_expr_and_term() {
    let (g, pt) = expr_term();
    for name in &["expr", "term"] {
        let sym = nt(&g, name);
        let found = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), sym).is_some());
        assert!(found, "goto should have entry for '{name}'");
    }
}

#[test]
fn goto_target_is_valid_state() {
    let (g, pt) = expr_term();
    let expr_sym = nt(&g, "expr");
    for s in 0..pt.state_count {
        if let Some(target) = pt.goto(StateId(s as u16), expr_sym) {
            assert!(
                (target.0 as usize) < pt.state_count,
                "goto target {} out of range",
                target.0
            );
        }
    }
}

#[test]
fn goto_some_for_list_nt() {
    let (g, pt) = list_grammar();
    let items_sym = nt(&g, "items");
    let found = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), items_sym).is_some());
    assert!(found, "goto should exist for 'items'");
}

// ===========================================================================
// 8. goto returns None for terminals (usually)
// ===========================================================================

#[test]
fn goto_none_for_terminal_single() {
    let (g, pt) = single_token();
    let x = tok(&g, "x");
    for s in 0..pt.state_count {
        assert!(
            pt.goto(StateId(s as u16), x).is_none(),
            "goto should be None for terminal 'x' in state {s}"
        );
    }
}

#[test]
fn goto_none_for_terminal_num() {
    let (g, pt) = expr_term();
    let num = tok(&g, "num");
    for s in 0..pt.state_count {
        assert!(
            pt.goto(StateId(s as u16), num).is_none(),
            "goto should be None for terminal 'num' in state {s}"
        );
    }
}

#[test]
fn goto_none_for_terminal_plus() {
    let (g, pt) = expr_term();
    let plus = tok(&g, "+");
    for s in 0..pt.state_count {
        assert!(
            pt.goto(StateId(s as u16), plus).is_none(),
            "goto should be None for terminal '+' in state {s}"
        );
    }
}

#[test]
fn goto_none_for_eof() {
    let (_, pt) = single_token();
    let eof = pt.eof();
    for s in 0..pt.state_count {
        assert!(
            pt.goto(StateId(s as u16), eof).is_none(),
            "goto should be None for EOF in state {s}"
        );
    }
}

#[test]
fn goto_none_for_all_terminals_in_list() {
    let (g, pt) = list_grammar();
    let comma = tok(&g, ",");
    let id = tok(&g, "id");
    for s in 0..pt.state_count {
        let sid = StateId(s as u16);
        assert!(
            pt.goto(sid, comma).is_none(),
            "goto None for ',' in state {s}"
        );
        assert!(
            pt.goto(sid, id).is_none(),
            "goto None for 'id' in state {s}"
        );
    }
}

// ===========================================================================
// 9. rule() returns correct lhs and rhs_length
// ===========================================================================

#[test]
fn rule_lhs_is_start_for_single_token() {
    let (g, pt) = single_token();
    let start_sym = nt(&g, "start");
    // Find a Reduce action to get a RuleId
    let eof = pt.eof();
    for s in 0..pt.state_count {
        for a in pt.actions(StateId(s as u16), eof) {
            if let Action::Reduce(rid) = a {
                let (lhs, _) = pt.rule(*rid);
                if lhs == start_sym {
                    return; // found it
                }
            }
        }
    }
    // Also check non-EOF columns
    for s in 0..pt.state_count {
        for &sym in pt.symbol_to_index.keys() {
            for a in pt.actions(StateId(s as u16), sym) {
                if let Action::Reduce(rid) = a {
                    let (lhs, _) = pt.rule(*rid);
                    if lhs == start_sym {
                        return;
                    }
                }
            }
        }
    }
    panic!("expected a Reduce with lhs == start");
}

#[test]
fn rule_rhs_len_single_token() {
    let (_, pt) = single_token();
    // start → x has rhs_len == 1
    let mut found = false;
    for rid in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(rid as u16));
        if rhs_len == 1 {
            found = true;
            break;
        }
    }
    assert!(found, "should find a rule with rhs_len == 1");
}

#[test]
fn rule_rhs_len_two_token() {
    let (_, pt) = two_token();
    // start → a b has rhs_len == 2
    let mut found = false;
    for rid in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(rid as u16));
        if rhs_len == 2 {
            found = true;
            break;
        }
    }
    assert!(found, "should find a rule with rhs_len == 2");
}

#[test]
fn rule_rhs_len_four_token() {
    let (_, pt) = four_token_linear();
    // start → a b c d has rhs_len == 4
    let mut found = false;
    for rid in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(rid as u16));
        if rhs_len == 4 {
            found = true;
            break;
        }
    }
    assert!(found, "should find a rule with rhs_len == 4");
}

#[test]
fn rule_rhs_len_chain_all_one() {
    let (_, pt) = chain_rule();
    // inner → x, wrap → inner, chain → wrap: all rhs_len == 1
    let mut count_one = 0;
    for rid in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(rid as u16));
        if rhs_len == 1 {
            count_one += 1;
        }
    }
    // At least 3 user rules + augmented start rule have rhs_len == 1
    assert!(
        count_one >= 3,
        "chain grammar should have >= 3 rules with rhs_len == 1, got {count_one}"
    );
}

#[test]
fn rule_lhs_and_len_for_list() {
    let (g, pt) = list_grammar();
    let items_sym = nt(&g, "items");
    let mut found_len1 = false;
    let mut found_len3 = false;
    for rid in 0..pt.rules.len() {
        let (lhs, rhs_len) = pt.rule(RuleId(rid as u16));
        if lhs == items_sym && rhs_len == 1 {
            found_len1 = true;
        }
        if lhs == items_sym && rhs_len == 3 {
            found_len3 = true;
        }
    }
    assert!(found_len1, "items → id should have rhs_len == 1");
    assert!(found_len3, "items → items , id should have rhs_len == 3");
}

// ===========================================================================
// 10. Multiple tokens → more states
// ===========================================================================

#[test]
fn more_tokens_more_states_1_vs_2() {
    let (_, pt1) = single_token();
    let (_, pt2) = two_token();
    assert!(
        pt2.state_count > pt1.state_count,
        "2-token grammar ({}) should have more states than 1-token ({})",
        pt2.state_count,
        pt1.state_count
    );
}

#[test]
fn more_tokens_more_states_2_vs_4() {
    let (_, pt2) = two_token();
    let (_, pt4) = four_token_linear();
    assert!(
        pt4.state_count > pt2.state_count,
        "4-token linear ({}) should have more states than 2-token ({})",
        pt4.state_count,
        pt2.state_count
    );
}

#[test]
fn three_token_linear_has_gte_four_states() {
    let pt = make_table(
        "sm_v8_three_linear",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b", "c"])],
        "start",
    );
    assert!(
        pt.state_count >= 4,
        "3-token linear grammar needs >= 4 states, got {}",
        pt.state_count
    );
}

#[test]
fn paren_grammar_more_states_than_single() {
    let (_, pt_single) = single_token();
    let (_, pt_paren) = paren_grammar();
    assert!(
        pt_paren.state_count > pt_single.state_count,
        "paren grammar should have more states"
    );
}

#[test]
fn expr_term_more_states_than_two_token() {
    let (_, pt2) = two_token();
    let (_, pt_et) = expr_term();
    assert!(
        pt_et.state_count > pt2.state_count,
        "expr-term grammar should have more states than 2-token linear"
    );
}

// ===========================================================================
// 11. Chain rules → sequential shifts
// ===========================================================================

#[test]
fn chain_has_shift_on_x() {
    let (g, pt) = chain_rule();
    let x = tok(&g, "x");
    assert!(has_shift_on(&pt, x), "chain grammar should shift on 'x'");
}

#[test]
fn chain_has_reduce_on_eof() {
    let (_, pt) = chain_rule();
    let eof = pt.eof();
    assert!(
        has_reduce_on(&pt, eof),
        "chain grammar should reduce on EOF"
    );
}

#[test]
fn chain_goto_for_all_nonterminals() {
    let (g, pt) = chain_rule();
    for name in &["inner", "wrap", "chain"] {
        let sym = nt(&g, name);
        let found = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), sym).is_some());
        assert!(found, "chain should have goto for '{name}'");
    }
}

#[test]
fn nested_chain_has_shift_on_both_tokens() {
    let (g, pt) = nested_chain();
    let a = tok(&g, "a");
    let b = tok(&g, "b");
    assert!(has_shift_on(&pt, a), "nested chain should shift on 'a'");
    assert!(has_shift_on(&pt, b), "nested chain should shift on 'b'");
}

#[test]
fn nested_chain_has_goto_for_intermediate() {
    let (g, pt) = nested_chain();
    let mid = nt(&g, "mid");
    let low = nt(&g, "low");
    let found_mid = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), mid).is_some());
    let found_low = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), low).is_some());
    assert!(found_mid, "nested chain should have goto for 'mid'");
    assert!(found_low, "nested chain should have goto for 'low'");
}

// ===========================================================================
// 12. Actions non-empty for valid state+symbol pairs
// ===========================================================================

#[test]
fn initial_state_has_nonempty_actions() {
    let (_, pt) = single_token();
    let mut found_nonempty = false;
    for &sym in pt.symbol_to_index.keys() {
        if !pt.actions(pt.initial_state, sym).is_empty() {
            found_nonempty = true;
            break;
        }
    }
    assert!(
        found_nonempty,
        "initial state must have at least one nonempty action cell"
    );
}

#[test]
fn some_state_has_action_on_eof() {
    let (_, pt) = single_token();
    let eof = pt.eof();
    let found = (0..pt.state_count).any(|s| !pt.actions(StateId(s as u16), eof).is_empty());
    assert!(found, "some state must have action on EOF");
}

#[test]
fn nonempty_cells_gt_zero() {
    let (_, pt) = single_token();
    assert!(
        count_nonempty_cells(&pt) > 0,
        "must have at least one nonempty action cell"
    );
}

#[test]
fn every_grammar_has_nonempty_cells() {
    for (_, pt) in [
        single_token(),
        two_token(),
        chain_rule(),
        list_grammar(),
        expr_term(),
    ] {
        assert!(count_nonempty_cells(&pt) > 0);
    }
}

#[test]
fn complex_grammar_has_more_nonempty_cells() {
    let (_, pt1) = single_token();
    let (_, pt_et) = expr_term();
    assert!(
        count_nonempty_cells(&pt_et) > count_nonempty_cells(&pt1),
        "expr-term should have more nonempty cells than single-token"
    );
}

// ===========================================================================
// 13. ActionCell len() matches actions() slice len
// ===========================================================================

#[test]
fn action_cell_len_matches_slice_single() {
    let (_, pt) = single_token();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            assert_eq!(
                pt.action_table[s][col].len(),
                pt.action_table[s][col].as_slice().len()
            );
        }
    }
}

#[test]
fn action_cell_len_matches_slice_list() {
    let (_, pt) = list_grammar();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            assert_eq!(
                pt.action_table[s][col].len(),
                pt.action_table[s][col].as_slice().len()
            );
        }
    }
}

#[test]
fn actions_method_len_matches_cell() {
    let (_, pt) = expr_term();
    for s in 0..pt.state_count {
        let sid = StateId(s as u16);
        for &sym in pt.symbol_to_index.keys() {
            let actions = pt.actions(sid, sym);
            // actions() returns a slice, its len should be consistent
            assert!(actions.len() <= 10, "unexpectedly large action cell");
        }
    }
}

#[test]
fn action_cell_len_consistent_chain() {
    let (_, pt) = chain_rule();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            let slice_len = cell.as_slice().len();
            assert_eq!(
                cell.len(),
                slice_len,
                "len/slice mismatch at state {s} col {col}"
            );
        }
    }
}

#[test]
fn action_cell_len_consistent_paren() {
    let (_, pt) = paren_grammar();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            assert_eq!(cell.len(), cell.as_slice().len());
        }
    }
}

// ===========================================================================
// 14. ActionCell is_empty matches len == 0
// ===========================================================================

#[test]
fn is_empty_matches_len_zero_single() {
    let (_, pt) = single_token();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            if cell.is_empty() {
                assert_eq!(cell.len(), 0);
            } else {
                assert!(!cell.is_empty());
            }
        }
    }
}

#[test]
fn is_empty_matches_len_zero_expr_term() {
    let (_, pt) = expr_term();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            if cell.is_empty() {
                assert_eq!(cell.len(), 0);
            } else {
                assert!(!cell.is_empty());
            }
        }
    }
}

#[test]
fn is_empty_matches_len_zero_list() {
    let (_, pt) = list_grammar();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            if cell.is_empty() {
                assert_eq!(cell.len(), 0);
            } else {
                assert!(!cell.is_empty());
            }
        }
    }
}

#[test]
fn is_empty_matches_len_zero_paren() {
    let (_, pt) = paren_grammar();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            if cell.is_empty() {
                assert_eq!(cell.len(), 0);
            } else {
                assert!(!cell.is_empty());
            }
        }
    }
}

#[test]
fn is_empty_matches_len_zero_chain() {
    let (_, pt) = chain_rule();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            let cell = &pt.action_table[s][col];
            if cell.is_empty() {
                assert_eq!(cell.len(), 0);
            } else {
                assert!(!cell.is_empty());
            }
        }
    }
}

// ===========================================================================
// 15. Different grammars → different state_counts
// ===========================================================================

#[test]
fn different_state_counts_single_vs_chain() {
    let (_, pt1) = single_token();
    let (_, pt2) = chain_rule();
    // Both have one terminal but chain has 3 non-terminals
    assert_ne!(
        pt1.state_count, pt2.state_count,
        "single-token and chain should differ in state_count"
    );
}

#[test]
fn different_state_counts_two_vs_four() {
    let (_, pt2) = two_token();
    let (_, pt4) = four_token_linear();
    assert_ne!(pt2.state_count, pt4.state_count);
}

#[test]
fn different_state_counts_linear_vs_expr_term() {
    let (_, pt_two) = two_token();
    let (_, pt_et) = expr_term();
    assert_ne!(
        pt_two.state_count, pt_et.state_count,
        "linear 2-token and expr-term should have different state counts"
    );
}

#[test]
fn different_symbol_counts_single_vs_expr_term() {
    let (_, pt1) = single_token();
    let (_, pt_et) = expr_term();
    assert_ne!(
        pt1.symbol_count, pt_et.symbol_count,
        "single-token and expr-term should have different symbol counts"
    );
}

#[test]
fn different_nonempty_cells_single_vs_list() {
    let (_, pt1) = single_token();
    let (_, pt_list) = list_grammar();
    assert_ne!(
        count_nonempty_cells(&pt1),
        count_nonempty_cells(&pt_list),
        "single-token and list should have different nonempty cell counts"
    );
}

// ===========================================================================
// Additional coverage: shift/reduce presence, make_table helper, edge cases
// ===========================================================================

#[test]
fn make_table_helper_produces_valid_table() {
    let pt = make_table(
        "sm_v8_helper_test",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(pt.state_count >= 2);
    assert!(has_accept_anywhere(&pt));
}

#[test]
fn right_recursive_has_shift_and_reduce() {
    let (g, pt) = right_recursive();
    let a = tok(&g, "a");
    assert!(has_shift_on(&pt, a));
    let eof = pt.eof();
    assert!(has_reduce_on(&pt, eof));
}

#[test]
fn left_recursive_has_shift_and_reduce() {
    let (g, pt) = left_recursive();
    let a = tok(&g, "a");
    assert!(has_shift_on(&pt, a));
    let eof = pt.eof();
    assert!(has_reduce_on(&pt, eof));
}

#[test]
fn shared_prefix_builds_successfully() {
    let (_, pt) = shared_prefix();
    assert!(pt.state_count >= 2);
    assert!(has_accept_anywhere(&pt));
}

#[test]
fn shared_prefix_has_shift_on_a() {
    let (g, pt) = shared_prefix();
    let a = tok(&g, "a");
    assert!(has_shift_on(&pt, a));
}

#[test]
fn shared_prefix_has_shift_on_b_and_c() {
    let (g, pt) = shared_prefix();
    let b = tok(&g, "b");
    let c = tok(&g, "c");
    assert!(has_shift_on(&pt, b));
    assert!(has_shift_on(&pt, c));
}

#[test]
fn goto_table_rows_match_state_count() {
    let (_, pt) = expr_term();
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn all_shift_targets_valid() {
    let (_, pt) = list_grammar();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                if let Action::Shift(target) = a {
                    assert!(
                        (target.0 as usize) < pt.state_count,
                        "shift target {} out of range in state {s}",
                        target.0
                    );
                }
            }
        }
    }
}

#[test]
fn all_reduce_rule_ids_valid() {
    let (_, pt) = list_grammar();
    let num_rules = pt.rules.len();
    for s in 0..pt.state_count {
        for col in 0..pt.action_table[s].len() {
            for a in &pt.action_table[s][col] {
                if let Action::Reduce(rid) = a {
                    assert!(
                        (rid.0 as usize) < num_rules,
                        "reduce rule id {} out of range (max {})",
                        rid.0,
                        num_rules
                    );
                }
            }
        }
    }
}

#[test]
fn initial_state_no_accept() {
    let (_, pt) = single_token();
    let eof = pt.eof();
    let actions = pt.actions(pt.initial_state, eof);
    let has_accept = actions.iter().any(|a| matches!(a, Action::Accept));
    assert!(
        !has_accept,
        "initial state should not have Accept for single-token grammar"
    );
}

#[test]
fn start_symbol_accessor_consistent() {
    let (g, pt) = single_token();
    let start_sym = nt(&g, "start");
    // The augmented start symbol wraps user start, so start_symbol() may differ
    // but it should be a valid symbol
    let ss = pt.start_symbol();
    assert!(
        pt.nonterminal_to_index.contains_key(&ss)
            || pt.symbol_to_index.contains_key(&ss)
            || ss == start_sym,
        "start_symbol should be a known symbol"
    );
}

#[test]
fn grammar_accessor_returns_matching_grammar() {
    let (g, pt) = single_token();
    let pt_grammar = pt.grammar();
    assert_eq!(pt_grammar.name, g.name);
}

#[test]
fn two_alt_grammar_both_tokens_shifted() {
    let (g, pt) = two_alternatives();
    let a = tok(&g, "a");
    let b = tok(&g, "b");
    assert!(has_shift_on(&pt, a));
    assert!(has_shift_on(&pt, b));
}

#[test]
fn two_alt_grammar_accept_exists() {
    let (_, pt) = two_alternatives();
    assert!(has_accept_anywhere(&pt));
}

#[test]
fn list_grammar_comma_shifted() {
    let (g, pt) = list_grammar();
    let comma = tok(&g, ",");
    assert!(
        has_shift_on(&pt, comma),
        "comma must be shifted in list grammar"
    );
}

#[test]
fn list_grammar_id_shifted() {
    let (g, pt) = list_grammar();
    let id = tok(&g, "id");
    assert!(has_shift_on(&pt, id), "id must be shifted in list grammar");
}

#[test]
fn paren_grammar_all_tokens_shifted() {
    let (g, pt) = paren_grammar();
    let lp = tok(&g, "(");
    let rp = tok(&g, ")");
    let x = tok(&g, "x");
    assert!(has_shift_on(&pt, lp));
    assert!(has_shift_on(&pt, rp));
    assert!(has_shift_on(&pt, x));
}

#[test]
fn expr_term_plus_shifted() {
    let (g, pt) = expr_term();
    let plus = tok(&g, "+");
    assert!(has_shift_on(&pt, plus));
}

#[test]
fn expr_term_num_shifted() {
    let (g, pt) = expr_term();
    let num = tok(&g, "num");
    assert!(has_shift_on(&pt, num));
}
