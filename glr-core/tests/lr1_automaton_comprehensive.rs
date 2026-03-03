//! Comprehensive tests for the LR(1) automaton builder and FIRST/FOLLOW sets.
//!
//! Covers: single-rule grammars, alternation, epsilon/nullable rules,
//! left/right recursion, shift-reduce and reduce-reduce conflicts,
//! GOTO tables, state counts, FIRST/FOLLOW sets, and edge cases.

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
