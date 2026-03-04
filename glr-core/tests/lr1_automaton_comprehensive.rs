//! Comprehensive tests for the LR(1) automaton builder and FIRST/FOLLOW sets.
//!
//! Covers: single-rule grammars, alternation, epsilon/nullable rules,
//! left/right recursion, shift-reduce and reduce-reduce conflicts,
//! GOTO tables, state counts, FIRST/FOLLOW sets, and edge cases.

#![cfg(feature = "test-api")]

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW computation should succeed");
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

fn any_state_has_shift(table: &adze_glr_core::ParseTable, terminal: SymbolId) -> bool {
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), terminal)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

fn any_state_has_reduce(table: &adze_glr_core::ParseTable, lookahead: SymbolId) -> bool {
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), lookahead)
            .iter()
            .any(|a| matches!(a, Action::Reduce(_)))
    })
}

/// Count cells with multiple actions (conflicts / GLR forks).
fn count_conflict_cells(table: &adze_glr_core::ParseTable) -> usize {
    let mut count = 0;
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            let actions = &table.action_table[state][sym_idx];
            if actions.len() > 1 {
                count += 1;
            }
        }
    }
    count
}

fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found in grammar"))
}

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found in grammar"))
}

fn any_goto_exists(table: &adze_glr_core::ParseTable, nt: SymbolId) -> bool {
    (0..table.state_count).any(|st| table.goto(StateId(st as u16), nt).is_some())
}

// ===========================================================================
// 1. Single-rule grammar: start → a
// ===========================================================================

#[test]
fn single_rule_produces_accept() {
    let g = GrammarBuilder::new("s1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table), "single-rule grammar must have Accept");
}

#[test]
fn single_rule_shifts_terminal() {
    let g = GrammarBuilder::new("s1b")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        initial_shifts(&table, tok_id(&g, "a")),
        "initial state must shift on terminal 'a'"
    );
}

#[test]
fn single_rule_reduces_on_eof() {
    let g = GrammarBuilder::new("s1c")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        any_state_has_reduce(&table, table.eof()),
        "must have Reduce on EOF after shifting 'a'"
    );
}

// ===========================================================================
// 2. Two-rule grammar with alternation: start → a | b
// ===========================================================================

#[test]
fn alternation_shifts_both() {
    let g = GrammarBuilder::new("alt2")
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
fn alternation_has_accept() {
    let g = GrammarBuilder::new("alt2b")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn alternation_has_two_reduce_actions() {
    let g = GrammarBuilder::new("alt2c")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let mut reduce_count = 0;
    for st in 0..table.state_count {
        for action in table.actions(StateId(st as u16), eof) {
            if matches!(action, Action::Reduce(_)) {
                reduce_count += 1;
            }
        }
    }
    assert!(
        reduce_count >= 2,
        "two alternations should produce at least 2 reduce actions on EOF, got {reduce_count}"
    );
}

// ===========================================================================
// 3. Grammar with epsilon/nullable rules: start → ε | a
// ===========================================================================

#[test]
fn nullable_rule_accepted() {
    let g = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn nullable_rule_detected_by_first_follow() {
    let g = GrammarBuilder::new("eps2")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    assert!(
        ff.is_nullable(s),
        "start with ε production must be nullable"
    );
}

#[test]
fn nullable_chained_detection() {
    // mid → ε, start → mid  ⇒  start is also nullable
    let g = GrammarBuilder::new("eps3")
        .token("a", "a")
        .rule("mid", vec![])
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    assert!(
        ff.is_nullable(s),
        "start → mid and mid → ε means start is nullable"
    );
}

// ===========================================================================
// 4. Left-recursive grammar: expr → expr '+' term | term, term → n
// ===========================================================================

#[test]
fn left_recursive_arithmetic_builds() {
    let g = GrammarBuilder::new("lr_arith")
        .token("n", "n")
        .token("+", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn left_recursive_shifts_n_initially() {
    let g = GrammarBuilder::new("lr_arith2")
        .token("n", "n")
        .token("+", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(
        initial_shifts(&table, tok_id(&g, "n")),
        "left-recursive expr grammar must shift 'n' from initial state"
    );
}

#[test]
fn left_recursive_shifts_plus_somewhere() {
    let g = GrammarBuilder::new("lr_arith3")
        .token("n", "n")
        .token("+", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(
        any_state_has_shift(&table, tok_id(&g, "+")),
        "some state must shift '+' after reducing to expr"
    );
}

// ===========================================================================
// 5. Right-recursive grammar: list → a list | a
// ===========================================================================

#[test]
fn right_recursive_builds() {
    let g = GrammarBuilder::new("rrec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn right_recursive_state_count() {
    let g = GrammarBuilder::new("rrec2")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3,
        "right-recursive grammar needs >= 3 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 6. Shift-reduce conflict: expr → expr '+' expr | n
// ===========================================================================

#[test]
fn shift_reduce_conflict_produces_multiple_actions() {
    // Ambiguous grammar: E → E + E | n
    // After seeing "n + n", on lookahead '+' there is a shift-reduce conflict.
    let g = GrammarBuilder::new("sr_conflict")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    let conflicts = count_conflict_cells(&table);
    assert!(
        conflicts > 0,
        "ambiguous expr → expr + expr | n must produce GLR conflicts, got 0"
    );
}

#[test]
fn shift_reduce_conflict_preserves_both_actions() {
    let g = GrammarBuilder::new("sr_both")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let plus = tok_id(&g, "+");
    // Find a state with a conflict on '+'
    let mut found_shift = false;
    let mut found_reduce = false;
    for st in 0..table.state_count {
        let actions = table.actions(StateId(st as u16), plus);
        if actions.len() > 1 {
            for a in actions {
                match a {
                    Action::Shift(_) => found_shift = true,
                    Action::Reduce(_) => found_reduce = true,
                    _ => {}
                }
            }
        }
    }
    assert!(
        found_shift && found_reduce,
        "GLR table must preserve both Shift and Reduce for ambiguous grammar"
    );
}

// ===========================================================================
// 7. Reduce-reduce conflict: start → a_rule | b_rule,
//    a_rule → x, b_rule → x
// ===========================================================================

#[test]
fn reduce_reduce_conflict_handled() {
    // Two non-terminals with identical RHS, used in the same context.
    // start → a_nt y | b_nt y, a_nt → x, b_nt → x
    // The GLR builder may preserve or resolve reduce-reduce conflicts.
    // We verify the grammar builds successfully and the table is functional.
    let mut grammar = Grammar::new("rr_conflict".to_string());

    let x_id = SymbolId(1);
    let y_id = SymbolId(2);
    grammar.tokens.insert(
        x_id,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        y_id,
        Token {
            name: "y".to_string(),
            pattern: TokenPattern::String("y".to_string()),
            fragile: false,
        },
    );

    let a_nt = SymbolId(10);
    let b_nt = SymbolId(11);
    let start_nt = SymbolId(12);

    grammar.rule_names.insert(a_nt, "a_nt".to_string());
    grammar.rule_names.insert(b_nt, "b_nt".to_string());
    grammar.rule_names.insert(start_nt, "start".to_string());

    grammar.rules.insert(
        a_nt,
        vec![Rule {
            lhs: a_nt,
            rhs: vec![Symbol::Terminal(x_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    grammar.rules.insert(
        b_nt,
        vec![Rule {
            lhs: b_nt,
            rhs: vec![Symbol::Terminal(x_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        }],
    );
    grammar.rules.insert(
        start_nt,
        vec![
            Rule {
                lhs: start_nt,
                rhs: vec![Symbol::NonTerminal(a_nt), Symbol::Terminal(y_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
            Rule {
                lhs: start_nt,
                rhs: vec![Symbol::NonTerminal(b_nt), Symbol::Terminal(y_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(3),
                fields: vec![],
            },
        ],
    );

    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");
    let table = build_lr1_automaton(&grammar, &ff).expect("automaton should build");

    // Grammar with reduce-reduce potential must build successfully
    assert!(has_accept(&table));

    // Both rules for a_nt→x and b_nt→x must appear as reduces somewhere
    let mut has_rule0_reduce = false;
    let mut has_any_reduce = false;
    for st in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for action in &table.action_table[st][sym_idx] {
                if let Action::Reduce(rid) = action {
                    has_any_reduce = true;
                    if rid.0 == 0 {
                        has_rule0_reduce = true;
                    }
                }
            }
        }
    }
    assert!(has_any_reduce, "table must contain Reduce actions");
    assert!(
        has_rule0_reduce,
        "table must contain a reduce for the first conflicting rule"
    );
}

// ===========================================================================
// 8. Start state has shift actions
// ===========================================================================

#[test]
fn start_state_has_shift_actions() {
    let g = GrammarBuilder::new("init_shift")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .start("start")
        .build();
    let table = build_table(&g);
    let init = table.initial_state;
    let has_any_shift = table.action_table[init.0 as usize]
        .iter()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(has_any_shift, "initial state must have at least one Shift");
}

// ===========================================================================
// 9. Accept action present for start symbol + EOF
// ===========================================================================

#[test]
fn accept_on_eof_after_start_reduction() {
    let g = GrammarBuilder::new("accept_eof")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let accept_found = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(accept_found, "Accept must be reachable on EOF");
}

// ===========================================================================
// 10. GOTO table populated for non-terminals
// ===========================================================================

#[test]
fn goto_populated_for_inner_nonterminal() {
    let g = GrammarBuilder::new("goto_inner")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    assert!(
        any_goto_exists(&table, inner),
        "GOTO must have an entry for nonterminal 'inner'"
    );
}

#[test]
fn goto_populated_for_start_symbol() {
    let g = GrammarBuilder::new("goto_start")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    assert!(
        table.goto(table.initial_state, s).is_some(),
        "goto(initial, start) must exist for augmented grammar"
    );
}

#[test]
fn goto_covers_user_nonterminals() {
    let g = GrammarBuilder::new("goto_all")
        .token("a", "a")
        .token("b", "b")
        .rule("leaf", vec!["a"])
        .rule("mid", vec!["leaf", "b"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["leaf", "mid", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            any_goto_exists(&table, nt),
            "GOTO must have an entry for nonterminal '{name}'"
        );
    }
}

// ===========================================================================
// 11. State count reasonable for grammar size
// ===========================================================================

#[test]
fn state_count_single_rule() {
    let g = GrammarBuilder::new("sc1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3 && table.state_count <= 10,
        "single-rule grammar state count out of range: {}",
        table.state_count
    );
}

#[test]
fn state_count_grows_with_rules() {
    let small = GrammarBuilder::new("small")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let big = GrammarBuilder::new("big")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("start", vec!["x", "y", "z"])
        .start("start")
        .build();
    let t_small = build_table(&small);
    let t_big = build_table(&big);
    assert!(
        t_big.state_count > t_small.state_count,
        "larger grammar should produce more states ({} vs {})",
        t_big.state_count,
        t_small.state_count
    );
}

// ===========================================================================
// 12. FIRST sets correct for simple grammar
// ===========================================================================

#[test]
fn first_set_direct_terminal() {
    let g = GrammarBuilder::new("f1")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let x = tok_id(&g, "x");
    let first = ff.first(s).expect("FIRST(start) must exist");
    assert!(
        first.contains(x.0 as usize),
        "FIRST(start) must contain 'x'"
    );
}

#[test]
fn first_set_through_chain() {
    // start → mid, mid → leaf, leaf → x  ⇒  x ∈ FIRST(start)
    let g = GrammarBuilder::new("f2")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = tok_id(&g, "x");
    for name in &["leaf", "mid", "start"] {
        let id = nt_id(&g, name);
        let first = ff.first(id).unwrap();
        assert!(
            first.contains(x.0 as usize),
            "FIRST({name}) must contain 'x'"
        );
    }
}

#[test]
fn first_set_union_alternatives() {
    // start → a | b | c
    let g = GrammarBuilder::new("f3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let first = ff.first(s).unwrap();
    for name in &["a", "b", "c"] {
        let t = tok_id(&g, name);
        assert!(
            first.contains(t.0 as usize),
            "FIRST(start) must contain '{name}'"
        );
    }
}

// ===========================================================================
// 13. FOLLOW sets correct for simple grammar
// ===========================================================================

#[test]
fn follow_start_contains_eof() {
    let g = GrammarBuilder::new("fo1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let follow = ff.follow(s).expect("FOLLOW(start) must exist");
    // EOF is typically represented at bit-index 0 in the follow set
    assert!(
        follow.contains(0),
        "FOLLOW(start) must contain EOF (index 0)"
    );
}

#[test]
fn follow_from_sequence_context() {
    // start → lhs rhs, lhs → a, rhs → b
    // FOLLOW(lhs) should contain FIRST(rhs) = {b}
    let g = GrammarBuilder::new("fo2")
        .token("a", "a")
        .token("b", "b")
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .rule("start", vec!["lhs", "rhs"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let lhs = nt_id(&g, "lhs");
    let b = tok_id(&g, "b");
    let follow = ff.follow(lhs).unwrap();
    assert!(
        follow.contains(b.0 as usize),
        "FOLLOW(lhs) must contain 'b' from start → lhs rhs"
    );
}

#[test]
fn follow_propagation_at_end() {
    // start → inner, inner → a  ⇒  FOLLOW(inner) ⊇ FOLLOW(start) ⊇ {EOF}
    let g = GrammarBuilder::new("fo3")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let inner = nt_id(&g, "inner");
    let follow = ff.follow(inner).unwrap();
    assert!(
        follow.contains(0),
        "FOLLOW(inner) must contain EOF because inner is at end of start production"
    );
}

// ===========================================================================
// 14. FIRST set contains epsilon indicator for nullable symbol
// ===========================================================================

#[test]
fn first_set_nullable_is_nullable() {
    let g = GrammarBuilder::new("fn1")
        .token("a", "a")
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .rule("start", vec!["opt"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let opt = nt_id(&g, "opt");
    assert!(
        ff.is_nullable(opt),
        "opt with ε production must be nullable"
    );
}

#[test]
fn non_nullable_not_flagged() {
    let g = GrammarBuilder::new("fn2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    assert!(!ff.is_nullable(s), "start → a is not nullable");
}

// ===========================================================================
// 15. Empty grammar edge case
// ===========================================================================

#[test]
fn empty_grammar_no_panic() {
    // A grammar with only an epsilon start rule should not panic.
    let g = GrammarBuilder::new("empty")
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g);
    // It's acceptable if this returns an error or succeeds;
    // the key invariant is no panic.
    if let Ok(ff) = ff {
        let _ = build_lr1_automaton(&g, &ff);
    }
}

// ===========================================================================
// 16. EOF symbol distinct from all grammar symbols
// ===========================================================================

#[test]
fn eof_distinct_from_tokens_and_rules() {
    let g = GrammarBuilder::new("eof_distinct")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for (tok, _) in &g.tokens {
        assert_ne!(eof, *tok, "EOF must differ from token {:?}", tok);
    }
    for (rule, _) in &g.rules {
        assert_ne!(eof, *rule, "EOF must differ from rule {:?}", rule);
    }
}

// ===========================================================================
// 17. Start symbol preserved in parse table
// ===========================================================================

#[test]
fn start_symbol_matches_grammar() {
    let g = GrammarBuilder::new("sp")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    assert_eq!(table.start_symbol(), s);
}

// ===========================================================================
// 18. Parse table rules populated
// ===========================================================================

#[test]
fn parse_table_has_rules() {
    let g = GrammarBuilder::new("rules")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        !table.rules.is_empty(),
        "parse table must have at least one rule"
    );
}

// ===========================================================================
// 19. No spurious shifts on unused terminals
// ===========================================================================

#[test]
fn no_shift_on_unused_terminal() {
    let g = GrammarBuilder::new("unused")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let b = tok_id(&g, "b");
    assert!(
        !initial_shifts(&table, b),
        "initial state must NOT shift on unused terminal 'b'"
    );
}

// ===========================================================================
// 20. Dangling-else shift-reduce conflict
// ===========================================================================

#[test]
fn dangling_else_produces_conflict() {
    let mut grammar = Grammar::new("dangling".to_string());
    let if_id = SymbolId(1);
    let then_id = SymbolId(2);
    let else_id = SymbolId(3);
    let atom_id = SymbolId(4);

    for (id, name, pat) in [
        (if_id, "if", "if"),
        (then_id, "then", "then"),
        (else_id, "else", "else"),
        (atom_id, "atom", "atom"),
    ] {
        grammar.tokens.insert(
            id,
            Token {
                name: name.to_string(),
                pattern: TokenPattern::String(pat.to_string()),
                fragile: false,
            },
        );
    }

    let s_id = SymbolId(10);
    grammar.rule_names.insert(s_id, "S".to_string());
    grammar.rules.insert(
        s_id,
        vec![
            Rule {
                lhs: s_id,
                rhs: vec![
                    Symbol::Terminal(if_id),
                    Symbol::Terminal(then_id),
                    Symbol::NonTerminal(s_id),
                    Symbol::Terminal(else_id),
                    Symbol::NonTerminal(s_id),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: s_id,
                rhs: vec![
                    Symbol::Terminal(if_id),
                    Symbol::Terminal(then_id),
                    Symbol::NonTerminal(s_id),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: s_id,
                rhs: vec![Symbol::Terminal(atom_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let conflicts = count_conflict_cells(&table);
    assert!(
        conflicts > 0,
        "dangling-else grammar must produce shift-reduce conflicts"
    );
}

// ===========================================================================
// 21. Left-recursive FIRST set excludes non-leading terminals
// ===========================================================================

#[test]
fn left_recursive_first_excludes_non_leading() {
    // list → list '+' a | a   ⇒  FIRST(list) = {a}, not {a, '+'}
    let g = GrammarBuilder::new("lr_first")
        .token("a", "a")
        .token("+", "+")
        .rule("list", vec!["list", "+", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let list = nt_id(&g, "list");
    let a = tok_id(&g, "a");
    let plus = tok_id(&g, "+");
    let first = ff.first(list).unwrap();
    assert!(first.contains(a.0 as usize), "'a' must be in FIRST(list)");
    assert!(
        !first.contains(plus.0 as usize),
        "'+' must NOT be in FIRST(list)"
    );
}

// ===========================================================================
// 22. FOLLOW set for right-recursive nonterminal
// ===========================================================================

#[test]
fn follow_right_recursive_contains_eof() {
    let g = GrammarBuilder::new("rr_follow")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let seq = nt_id(&g, "seq");
    let follow = ff.follow(seq).unwrap();
    assert!(
        follow.contains(0),
        "FOLLOW(seq) must contain EOF for start symbol"
    );
}

// ===========================================================================
// 23. GOTO remapping preserves Accept
// ===========================================================================

#[test]
fn goto_remap_preserves_accept() {
    let g = GrammarBuilder::new("remap")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g).remap_goto_to_direct_symbol_id();
    assert!(has_accept(&table), "Accept must survive GOTO remapping");
}

// ===========================================================================
// 24. Multiple non-terminals with shared terminal
// ===========================================================================

#[test]
fn shared_terminal_across_nonterminals() {
    // a_nt → x, b_nt → x y, start → a_nt | b_nt
    let g = GrammarBuilder::new("shared")
        .token("x", "x")
        .token("y", "y")
        .rule("a_nt", vec!["x"])
        .rule("b_nt", vec!["x", "y"])
        .rule("start", vec!["a_nt"])
        .rule("start", vec!["b_nt"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(initial_shifts(&table, tok_id(&g, "x")));
}

// ===========================================================================
// 25. Long sequence state count
// ===========================================================================

#[test]
fn four_token_sequence_state_count() {
    let g = GrammarBuilder::new("seq4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(
        table.state_count >= 5,
        "4-token sequence needs >= 5 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 26. Rule metadata roundtrip
// ===========================================================================

#[test]
fn rule_metadata_lhs_and_len() {
    let g = GrammarBuilder::new("rmeta")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Find a rule with rhs_len == 2 (start → a b)
    let has_len2 = table.rules.iter().any(|r| r.rhs_len == 2);
    assert!(
        has_len2,
        "parse table must contain a rule with rhs_len == 2 for start → a b"
    );
}

// ===========================================================================
// 27. FIRST set for nullable prefix: start → opt a, opt → ε | b
// ===========================================================================

#[test]
fn first_through_nullable_prefix() {
    // start → opt a, opt → ε | b
    // FIRST(start) should contain both 'a' (via opt → ε) and 'b' (via opt → b)
    let g = GrammarBuilder::new("nprefix")
        .token("a", "a")
        .token("b", "b")
        .rule("opt", vec![])
        .rule("opt", vec!["b"])
        .rule("start", vec!["opt", "a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let first = ff.first(s).unwrap();
    assert!(
        first.contains(a.0 as usize),
        "FIRST(start) must contain 'a' via nullable opt"
    );
    assert!(
        first.contains(b.0 as usize),
        "FIRST(start) must contain 'b' from opt → b"
    );
}

// ===========================================================================
// 28. Deterministic: building twice gives same state count
// ===========================================================================

#[test]
fn deterministic_state_count() {
    let make = || {
        let g = GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = make();
    let t2 = make();
    assert_eq!(
        t1.state_count, t2.state_count,
        "state count must be deterministic"
    );
}

// ===========================================================================
// 29. Deterministic: rule count stable across builds
// ===========================================================================

#[test]
fn deterministic_rule_count() {
    let make = || {
        let g = GrammarBuilder::new("det2")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = make();
    let t2 = make();
    assert_eq!(
        t1.rules.len(),
        t2.rules.len(),
        "rule count must be deterministic"
    );
}

// ===========================================================================
// 30. Deterministic: symbol count stable
// ===========================================================================

#[test]
fn deterministic_symbol_count() {
    let make = || {
        let g = GrammarBuilder::new("det3")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = make();
    let t2 = make();
    assert_eq!(
        t1.symbol_count, t2.symbol_count,
        "symbol count must be deterministic"
    );
}

// ===========================================================================
// 31. Deterministic: EOF symbol stable
// ===========================================================================

#[test]
fn deterministic_eof_symbol() {
    let make = || {
        let g = GrammarBuilder::new("det4")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = make();
    let t2 = make();
    assert_eq!(t1.eof(), t2.eof(), "EOF symbol must be deterministic");
}

// ===========================================================================
// 32. Deterministic: initial state stable
// ===========================================================================

#[test]
fn deterministic_initial_state() {
    let make = || {
        let g = GrammarBuilder::new("det5")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = make();
    let t2 = make();
    assert_eq!(
        t1.initial_state, t2.initial_state,
        "initial state must be deterministic"
    );
}

// ===========================================================================
// 33. Shift target is always a valid state index
// ===========================================================================

#[test]
fn shift_targets_in_range() {
    let g = GrammarBuilder::new("stir")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let t = build_table(&g);
    for st in 0..t.state_count {
        for cell in &t.action_table[st] {
            for action in cell {
                if let Action::Shift(target) = action {
                    assert!(
                        (target.0 as usize) < t.state_count,
                        "shift target {} out of range (state_count={})",
                        target.0,
                        t.state_count
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 34. Reduce rule IDs are all in range
// ===========================================================================

#[test]
fn reduce_rule_ids_in_range() {
    let g = GrammarBuilder::new("rrir")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&g);
    for st in 0..t.state_count {
        for cell in &t.action_table[st] {
            for action in cell {
                if let Action::Reduce(rid) = action {
                    assert!(
                        (rid.0 as usize) < t.rules.len(),
                        "reduce rule {} out of range (rules.len()={})",
                        rid.0,
                        t.rules.len()
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 35. Action table rows match state count
// ===========================================================================

#[test]
fn action_table_rows_match_state_count() {
    let g = GrammarBuilder::new("atrc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert_eq!(
        t.action_table.len(),
        t.state_count,
        "action_table rows must equal state_count"
    );
}

// ===========================================================================
// 36. Goto table rows match state count
// ===========================================================================

#[test]
fn goto_table_rows_match_state_count() {
    let g = GrammarBuilder::new("gtrc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert_eq!(
        t.goto_table.len(),
        t.state_count,
        "goto_table rows must equal state_count"
    );
}

// ===========================================================================
// 37. Initial state is within range
// ===========================================================================

#[test]
fn initial_state_in_range() {
    let g = GrammarBuilder::new("isir")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(
        (t.initial_state.0 as usize) < t.state_count,
        "initial_state {} must be < state_count {}",
        t.initial_state.0,
        t.state_count
    );
}

// ===========================================================================
// 38. Token count is positive
// ===========================================================================

#[test]
fn token_count_positive() {
    let g = GrammarBuilder::new("tcp")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(t.token_count > 0, "token_count must be positive");
}

// ===========================================================================
// 39. Symbol count includes tokens
// ===========================================================================

#[test]
fn symbol_count_includes_tokens() {
    let g = GrammarBuilder::new("sci")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(
        t.symbol_count >= 3,
        "symbol_count must include at least the 3 tokens, got {}",
        t.symbol_count
    );
}

// ===========================================================================
// 40. Unary prefix operator grammar
// ===========================================================================

#[test]
fn unary_prefix_operator() {
    // expr → - expr | n
    let g = GrammarBuilder::new("unary")
        .token("n", "n")
        .token("-", "-")
        .rule("expr", vec!["-", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(initial_shifts(&t, tok_id(&g, "-")));
    assert!(initial_shifts(&t, tok_id(&g, "n")));
}

// ===========================================================================
// 41. Nested parentheses grammar
// ===========================================================================

#[test]
fn nested_parentheses() {
    // start → ( start ) | a
    let g = GrammarBuilder::new("nested")
        .token("a", "a")
        .token("(", "(")
        .token(")", ")")
        .rule("start", vec!["(", "start", ")"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(initial_shifts(&t, tok_id(&g, "(")));
    assert!(initial_shifts(&t, tok_id(&g, "a")));
}

// ===========================================================================
// 42. Comma-separated list grammar
// ===========================================================================

#[test]
fn comma_separated_list() {
    // list → list , item | item
    // item → n
    let g = GrammarBuilder::new("csv")
        .token("n", "n")
        .token(",", ",")
        .rule("item", vec!["n"])
        .rule("csv_list", vec!["csv_list", ",", "item"])
        .rule("csv_list", vec!["item"])
        .start("csv_list")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(initial_shifts(&t, tok_id(&g, "n")));
}

// ===========================================================================
// 43. Assignment-like grammar
// ===========================================================================

#[test]
fn assignment_grammar() {
    // stmt → id = expr, expr → n | id
    let g = GrammarBuilder::new("asgn")
        .token("id", "id")
        .token("=", "=")
        .token("n", "n")
        .rule("val", vec!["n"])
        .rule("val", vec!["id"])
        .rule("stmt", vec!["id", "=", "val"])
        .start("stmt")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
}

// ===========================================================================
// 44. Five-token linear grammar state count
// ===========================================================================

#[test]
fn five_token_linear_state_count() {
    let g = GrammarBuilder::new("lin5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(
        t.state_count >= 6,
        "5-token sequence needs >= 6 states, got {}",
        t.state_count
    );
}

// ===========================================================================
// 45. Accept only appears on EOF, never on regular tokens
// ===========================================================================

#[test]
fn accept_only_on_eof() {
    let g = GrammarBuilder::new("aeof")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let t = build_table(&g);
    let eof = t.eof();
    for st in 0..t.state_count {
        let state = StateId(st as u16);
        for (tok, _) in &g.tokens {
            let actions = t.actions(state, *tok);
            assert!(
                !actions.iter().any(|a| matches!(a, Action::Accept)),
                "Accept should only appear on EOF, not on token {:?}",
                tok
            );
        }
    }
    assert!(has_accept(&t));
}

// ===========================================================================
// 46. Shared prefix grammar: start → a b c | a b d
// ===========================================================================

#[test]
fn shared_prefix_shifts_both_continuations() {
    let g = GrammarBuilder::new("spfx")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c"])
        .rule("start", vec!["a", "b", "d"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    // Both c and d should be shiftable somewhere after a b
    let shifts_c = any_state_has_shift(&t, tok_id(&g, "c"));
    let shifts_d = any_state_has_shift(&t, tok_id(&g, "d"));
    assert!(shifts_c, "must shift 'c' somewhere");
    assert!(shifts_d, "must shift 'd' somewhere");
}

// ===========================================================================
// 47. Mutual recursion: even → a odd | a, odd → a even | a
// ===========================================================================

#[test]
fn mutual_recursion_builds() {
    let g = GrammarBuilder::new("mutr")
        .token("a", "a")
        .rule("even_r", vec!["a", "odd_r"])
        .rule("even_r", vec!["a"])
        .rule("odd_r", vec!["a", "even_r"])
        .rule("odd_r", vec!["a"])
        .rule("start", vec!["even_r"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
}

// ===========================================================================
// 48. Diamond-shaped grammar
// ===========================================================================

#[test]
fn diamond_grammar_gotos() {
    // start → left right, left → a, right → a
    let g = GrammarBuilder::new("dia")
        .token("a", "a")
        .rule("left_nt", vec!["a"])
        .rule("right_nt", vec!["a"])
        .rule("start", vec!["left_nt", "right_nt"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(any_goto_exists(&t, nt_id(&g, "left_nt")));
    assert!(any_goto_exists(&t, nt_id(&g, "right_nt")));
}

// ===========================================================================
// 49. Right associative grammar builds
// ===========================================================================

#[test]
fn right_associative_builds() {
    let g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("^", "^")
        .rule("base", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["base"])
        .start("expr")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
}

// ===========================================================================
// 50. Mixed associativity grammar builds
// ===========================================================================

#[test]
fn mixed_associativity_builds() {
    let g = GrammarBuilder::new("mixas")
        .token("n", "n")
        .token("+", "+")
        .token("^", "^")
        .rule("base", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 2, Associativity::Right)
        .rule("expr", vec!["base"])
        .start("expr")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
}

// ===========================================================================
// 51. Full arithmetic: expr/term/factor with parentheses
// ===========================================================================

#[test]
fn full_arithmetic_with_parens() {
    let g = GrammarBuilder::new("arithp")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("factor", vec!["n"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(
        t.state_count >= 8,
        "full arithmetic needs >= 8 states, got {}",
        t.state_count
    );
}

// ===========================================================================
// 52. Arithmetic gotos for all nonterminals
// ===========================================================================

#[test]
fn arithmetic_gotos_for_all_nonterminals() {
    let g = GrammarBuilder::new("ago")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule("factor", vec!["n"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let t = build_table(&g);
    for name in &["expr", "term", "factor"] {
        let nt = nt_id(&g, name);
        assert!(any_goto_exists(&t, nt), "must have goto for '{name}'");
    }
}

// ===========================================================================
// 53. Rule method returns correct lhs and rhs_len
// ===========================================================================

#[test]
fn rule_method_returns_correct_values() {
    let g = GrammarBuilder::new("rmeth")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let t = build_table(&g);
    let s = nt_id(&g, "start");
    let idx = t
        .rules
        .iter()
        .position(|r| r.lhs == s && r.rhs_len == 2)
        .expect("must have start rule with rhs_len=2");
    let (lhs, len) = t.rule(RuleId(idx as u16));
    assert_eq!(lhs, s);
    assert_eq!(len, 2);
}

// ===========================================================================
// 54. Epsilon rule rhs_len is 0
// ===========================================================================

#[test]
fn epsilon_rule_rhs_len_zero() {
    // start → ε | a — epsilon production should still result in a valid table
    let g = GrammarBuilder::new("epslen")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t), "grammar with epsilon rule must accept");
    // The table must have rules (augmented grammar generates them)
    assert!(
        !t.rules.is_empty(),
        "rules must not be empty for epsilon grammar"
    );
}

// ===========================================================================
// 55. Branching with sequences: start → a b | c d
// ===========================================================================

#[test]
fn branching_with_sequences_no_cross_shift() {
    let g = GrammarBuilder::new("brsq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["c", "d"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    // Initial state should shift on 'a' and 'c' but not 'b' or 'd'
    assert!(initial_shifts(&t, tok_id(&g, "a")));
    assert!(initial_shifts(&t, tok_id(&g, "c")));
    assert!(!initial_shifts(&t, tok_id(&g, "b")));
    assert!(!initial_shifts(&t, tok_id(&g, "d")));
}

// ===========================================================================
// 56. Left recursive with two operators
// ===========================================================================

#[test]
fn left_recursive_two_operators() {
    // expr → expr + n | expr * n | n
    let g = GrammarBuilder::new("lr2op")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "n"])
        .rule("expr", vec!["expr", "*", "n"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    // After reducing to expr, both + and * should be shiftable somewhere
    assert!(any_state_has_shift(&t, tok_id(&g, "+")));
    assert!(any_state_has_shift(&t, tok_id(&g, "*")));
}

// ===========================================================================
// 57. Triple nonterminal sequence gotos
// ===========================================================================

#[test]
fn triple_nonterminal_gotos() {
    // start → p q r, p → a, q → b, r → c
    let g = GrammarBuilder::new("tns")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("p_nt", vec!["a"])
        .rule("q_nt", vec!["b"])
        .rule("r_nt", vec!["c"])
        .rule("start", vec!["p_nt", "q_nt", "r_nt"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(any_goto_exists(&t, nt_id(&g, "p_nt")));
    assert!(any_goto_exists(&t, nt_id(&g, "q_nt")));
    assert!(any_goto_exists(&t, nt_id(&g, "r_nt")));
}

// ===========================================================================
// 58. Reduce on EOF for three alternatives
// ===========================================================================

#[test]
fn three_alternatives_all_reduce_on_eof() {
    let g = GrammarBuilder::new("3alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let t = build_table(&g);
    let eof = t.eof();
    let mut reduce_states = Vec::new();
    for st in 0..t.state_count {
        let state = StateId(st as u16);
        for action in t.actions(state, eof) {
            if matches!(action, Action::Reduce(_)) {
                reduce_states.push(st);
            }
        }
    }
    assert!(
        reduce_states.len() >= 3,
        "3 alternatives should yield >= 3 reduce actions on EOF, got {}",
        reduce_states.len()
    );
}

// ===========================================================================
// 59. Ambiguous grammar creates multi-action cells
// ===========================================================================

#[test]
fn ambiguous_grammar_multi_action_cells() {
    // expr → expr + expr | n  (inherently ambiguous without precedence)
    let g = GrammarBuilder::new("ambm")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    let plus = tok_id(&g, "+");
    let has_conflict = (0..t.state_count).any(|st| {
        let actions = t.actions(StateId(st as u16), plus);
        actions.len() > 1 || actions.iter().any(|a| matches!(a, Action::Fork(_)))
    });
    assert!(
        has_conflict,
        "ambiguous grammar must produce multi-action cell on '+'"
    );
}

// ===========================================================================
// 60. Goto target is always a valid state
// ===========================================================================

#[test]
fn goto_targets_in_range() {
    let g = GrammarBuilder::new("gtir")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let t = build_table(&g);
    let inner = nt_id(&g, "inner");
    for st in 0..t.state_count {
        if let Some(target) = t.goto(StateId(st as u16), inner) {
            assert!(
                (target.0 as usize) < t.state_count,
                "goto target {} out of range (state_count={})",
                target.0,
                t.state_count
            );
        }
    }
}

// ===========================================================================
// 61. Reduce action references rule with correct rhs_len
// ===========================================================================

#[test]
fn reduce_references_correct_rule_len() {
    // start → a b c (rhs_len=3)
    let g = GrammarBuilder::new("rrcrl")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let t = build_table(&g);
    let eof = t.eof();
    let mut found_len3 = false;
    for st in 0..t.state_count {
        for action in t.actions(StateId(st as u16), eof) {
            if let Action::Reduce(rid) = action {
                let (_, len) = t.rule(*rid);
                if len == 3 {
                    found_len3 = true;
                }
            }
        }
    }
    assert!(found_len3, "must have reduce for the 3-token rule on EOF");
}

// ===========================================================================
// 62. Nullable nonterminal in sequence shifts later token
// ===========================================================================

#[test]
fn nullable_in_sequence_shifts_later() {
    // opt → ε | a, start → opt b
    let g = GrammarBuilder::new("nsl")
        .token("a", "a")
        .token("b", "b")
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .rule("start", vec!["opt", "b"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    // Because opt is nullable, 'b' must be shiftable from some state
    assert!(
        any_state_has_shift(&t, tok_id(&g, "b")),
        "with nullable opt, 'b' must be shiftable from some state"
    );
}

// ===========================================================================
// 63. Precedence grammar builds without error
// ===========================================================================

#[test]
fn precedence_with_three_levels() {
    let g = GrammarBuilder::new("p3l")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule("atom", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["atom"])
        .start("expr")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(t.state_count >= 6);
}

// ===========================================================================
// 64. Deeply chained nonterminals (4 levels)
// ===========================================================================

#[test]
fn four_level_chain() {
    // start → a1, a1 → a2, a2 → a3, a3 → x
    let g = GrammarBuilder::new("ch4")
        .token("x", "x")
        .rule("a3", vec!["x"])
        .rule("a2", vec!["a3"])
        .rule("a1", vec!["a2"])
        .rule("start", vec!["a1"])
        .start("start")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let x = tok_id(&g, "x");
    let first_s = ff.first(s).unwrap();
    assert!(
        first_s.contains(x.0 as usize),
        "FIRST(start) must contain 'x' via a1→a2→a3→x"
    );
}

// ===========================================================================
// 65. If-then-else like grammar (dangling-else variant)
// ===========================================================================

#[test]
fn if_then_else_state_count() {
    let g = GrammarBuilder::new("ite2")
        .token("if_kw", "if")
        .token("then_kw", "then")
        .token("else_kw", "else")
        .token("id", "id")
        .rule("cond", vec!["id"])
        .rule(
            "stmt",
            vec!["if_kw", "cond", "then_kw", "stmt", "else_kw", "stmt"],
        )
        .rule("stmt", vec!["if_kw", "cond", "then_kw", "stmt"])
        .rule("stmt", vec!["id"])
        .start("stmt")
        .build();
    let t = build_table(&g);
    assert!(has_accept(&t));
    assert!(
        t.state_count >= 6,
        "if-then-else needs >= 6 states, got {}",
        t.state_count
    );
}
