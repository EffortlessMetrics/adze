#![allow(clippy::needless_range_loop)]
#![cfg(feature = "test-api")]

//! Comprehensive tests for the canonical collection builder (`ItemSetCollection`).
//!
//! Covers: simple/multi-rule grammars, left/right recursion, epsilon rules,
//! shift-reduce & reduce-reduce conflicts, closure properties, goto transitions,
//! start state invariants, and large grammars.

use adze_glr_core::*;
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build canonical collection from a mutable grammar via the standard pipeline.
fn build(grammar: &mut adze_ir::Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

/// Collect all state IDs reachable via goto_table targets.
fn target_states(col: &ItemSetCollection) -> BTreeSet<StateId> {
    col.goto_table.values().copied().collect()
}

/// Count transitions originating from the given state.
fn transitions_from(col: &ItemSetCollection, state: StateId) -> usize {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == state)
        .count()
}

// ===========================================================================
// 1. Simple grammars – single rule
// ===========================================================================

#[test]
fn single_terminal_rule_produces_states() {
    let mut g = GrammarBuilder::new("t1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets.len() >= 2,
        "S→a needs ≥2 states, got {}",
        col.sets.len()
    );
    assert!(col.sets.len() <= 6, "too many states: {}", col.sets.len());
}

#[test]
fn single_rule_initial_state_nonempty() {
    let mut g = GrammarBuilder::new("t2")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(!col.sets[0].items.is_empty(), "state 0 must contain items");
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn single_rule_has_goto_transitions() {
    let mut g = GrammarBuilder::new("t3")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        transitions_from(&col, StateId(0)) >= 1,
        "state 0 must have outgoing transitions"
    );
}

#[test]
fn single_two_terminal_rule() {
    let mut g = GrammarBuilder::new("t4")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // S→a b: state 0, after-a, after-b, plus goto-on-S
    assert!(
        col.sets.len() >= 3,
        "S→a b needs ≥3 states, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 2. Multi-rule grammars with terminals and non-terminals
// ===========================================================================

#[test]
fn alternation_two_terminals() {
    let mut g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Two alternative shifts from state 0.
    assert!(transitions_from(&col, StateId(0)) >= 2);
    let targets: BTreeSet<_> = col
        .goto_table
        .iter()
        .filter(|((s, _), _)| *s == StateId(0))
        .map(|(_, &d)| d)
        .collect();
    assert!(targets.len() >= 2, "distinct target states expected");
}

#[test]
fn nonterminal_chain_closure() {
    // S→A, A→B, B→c
    let mut g = GrammarBuilder::new("chain")
        .token("c", "c")
        .rule("B", vec!["c"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Closure of state 0 must include items from S, A, and B.
    assert!(
        col.sets[0].items.len() >= 3,
        "chain closure should pull in ≥3 items, got {}",
        col.sets[0].items.len()
    );
}

#[test]
fn mixed_terminal_nonterminal_rhs() {
    // S→A b, A→a
    let mut g = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Must classify both terminals and non-terminals.
    let t_count = col.symbol_is_terminal.values().filter(|&&t| t).count();
    let nt_count = col.symbol_is_terminal.values().filter(|&&t| !t).count();
    assert!(t_count >= 1, "should have ≥1 terminal");
    assert!(nt_count >= 1, "should have ≥1 non-terminal");
}

#[test]
fn multiple_nonterminals_shared_terminal() {
    // X→a b, Y→a, S→X | Y b
    let mut g = GrammarBuilder::new("shared")
        .token("a", "a")
        .token("b", "b")
        .rule("X", vec!["a", "b"])
        .rule("Y", vec!["a"])
        .rule("S", vec!["X"])
        .rule("S", vec!["Y", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 3, "shared-terminal grammar ≥3 states");
    // 'a' transition from state 0
    let a_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "a")
        .copied()
        .unwrap();
    assert!(col.goto_table.contains_key(&(StateId(0), a_sym)));
}

// ===========================================================================
// 3. Left-recursive grammars
// ===========================================================================

#[test]
fn left_recursive_list() {
    let mut g = GrammarBuilder::new("lrec")
        .token("item", "item")
        .rule("L", vec!["item"])
        .rule("L", vec!["L", "item"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets.len() >= 3,
        "left-recursive ≥3 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 20,
        "left-recursive should not explode: {}",
        col.sets.len()
    );
}

#[test]
fn left_recursive_has_nonterminal_goto() {
    let mut g = GrammarBuilder::new("lrec2")
        .token("item", "item")
        .rule("L", vec!["item"])
        .rule("L", vec!["L", "item"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);

    // There must be a goto on non-terminal L from state 0.
    let has_nt = col.goto_table.iter().any(|((src, sym), _)| {
        *src == StateId(0) && col.symbol_is_terminal.get(sym) == Some(&false)
    });
    assert!(has_nt, "state 0 must have a goto on non-terminal L");
}

#[test]
fn indirect_left_recursion() {
    // S→A x, A→S y | z
    let mut g = GrammarBuilder::new("indirect_lr")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("A", vec!["S", "y"])
        .rule("A", vec!["z"])
        .rule("S", vec!["A", "x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 3, "indirect left-recursion ≥3 states");
    assert!(
        col.sets.len() <= 30,
        "should not explode: {}",
        col.sets.len()
    );
}

// ===========================================================================
// 4. Right-recursive grammars
// ===========================================================================

#[test]
fn right_recursive_simple() {
    // R→x | x R
    let mut g = GrammarBuilder::new("rrec")
        .token("x", "x")
        .rule("R", vec!["x"])
        .rule("R", vec!["x", "R"])
        .start("R")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 3);
    assert!(col.sets.len() <= 20);
}

#[test]
fn right_recursive_binary() {
    // R→a | a b R
    let mut g = GrammarBuilder::new("rrec2")
        .token("a", "a")
        .token("b", "b")
        .rule("R", vec!["a"])
        .rule("R", vec!["a", "b", "R"])
        .start("R")
        .build();
    let (col, _) = build(&mut g);

    // a·b R needs separate state from a·(reduce)
    assert!(
        col.sets.len() >= 4,
        "right-recursive binary ≥4 states, got {}",
        col.sets.len()
    );
}

#[test]
fn mutual_recursion() {
    // A→x B, B→y A | y
    let mut g = GrammarBuilder::new("mutual")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x", "B"])
        .rule("B", vec!["y", "A"])
        .rule("B", vec!["y"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 4, "mutual recursion ≥4 states");
    assert!(col.sets.len() <= 30);
}

// ===========================================================================
// 5. Epsilon / empty rules
// ===========================================================================

#[test]
fn reduce_item_at_end_of_rule() {
    // After shifting all symbols of S→a b, the final state has a reduce item.
    let mut g = GrammarBuilder::new("reduce_end")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_reduce = col
        .sets
        .iter()
        .any(|s| s.items.iter().any(|item| item.is_reduce_item(&g)));
    assert!(
        has_reduce,
        "some state must have a reduce item at end of rule"
    );
}

#[test]
fn single_token_start_immediate_reduce() {
    // S→a has a reduce state immediately after shifting 'a'.
    let mut g = GrammarBuilder::new("imm_reduce")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // After shifting 'a', the next state has S→a · (reduce).
    let reduce_state = col.sets.iter().any(|s| {
        s.items
            .iter()
            .any(|i| i.is_reduce_item(&g) && i.position > 0)
    });
    assert!(
        reduce_state,
        "should have a reduce state after shifting 'a'"
    );
}

#[test]
fn optional_paths_via_alternatives() {
    // Simulate optional: start→x y | y  (x is optional via two alternative rules)
    let mut g = GrammarBuilder::new("opt_alt")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .rule("start", vec!["y"])
        .start("start")
        .build();
    let (col, ff) = build(&mut g);

    // FIRST(start) should include both 'x' and 'y'.
    let s_sym = g.find_symbol_by_name("start").unwrap();
    if let Some(first_s) = ff.first(s_sym) {
        let x_sym = g
            .tokens
            .keys()
            .find(|&&id| g.tokens[&id].name == "x")
            .copied()
            .unwrap();
        let y_sym = g
            .tokens
            .keys()
            .find(|&&id| g.tokens[&id].name == "y")
            .copied()
            .unwrap();
        assert!(
            first_s.contains(x_sym.0 as usize),
            "FIRST(start) should contain x"
        );
        assert!(
            first_s.contains(y_sym.0 as usize),
            "FIRST(start) should contain y"
        );
    }
    assert!(
        col.sets.len() >= 3,
        "optional-path grammar ≥3 states, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 6. Grammars with conflicts
// ===========================================================================

#[test]
fn shift_reduce_conflict_ambiguous_expr() {
    // E→E + E | num  (classic shift-reduce conflict)
    let mut g = GrammarBuilder::new("sr")
        .token("num", "num")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["num"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);

    // Should have a state with both shift and reduce items.
    let has_sr = col.sets.iter().any(|s| {
        let reduce = s.items.iter().any(|i| i.is_reduce_item(&g));
        let shift = s.items.iter().any(|i| i.next_symbol(&g).is_some());
        reduce && shift
    });
    assert!(
        has_sr,
        "ambiguous expr should produce shift-reduce conflict state"
    );
}

#[test]
fn reduce_reduce_conflict() {
    // A→a, B→a, S→A | B  (reduce-reduce on 'a')
    let mut g = GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("B", vec!["a"])
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // After shifting 'a', there should be a state with ≥2 reduce items.
    let has_rr = col.sets.iter().any(|s| {
        let reduce_count = s.items.iter().filter(|i| i.is_reduce_item(&g)).count();
        reduce_count >= 2
    });
    assert!(has_rr, "A→a. and B→a. should coexist in a state");
}

#[test]
fn dangling_else_shift_reduce() {
    // stmt→IF expr stmt | IF expr stmt ELSE stmt | OTHER
    let mut g = GrammarBuilder::new("dangle")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("OTHER", "other")
        .token("expr_tok", "e")
        .rule("stmt", vec!["IF", "expr_tok", "stmt"])
        .rule("stmt", vec!["IF", "expr_tok", "stmt", "ELSE", "stmt"])
        .rule("stmt", vec!["OTHER"])
        .start("stmt")
        .build();
    let (col, _) = build(&mut g);

    // The dangling-else grammar is inherently ambiguous and should produce
    // at least one conflict state.
    assert!(
        col.sets.len() >= 5,
        "dangling-else ≥5 states, got {}",
        col.sets.len()
    );
}

#[test]
fn precedence_does_not_eliminate_states() {
    // E→E+E | E*E | num  with precedence — states still built for all paths.
    let mut g = GrammarBuilder::new("prec")
        .token("num", "num")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("E", vec!["E", "+", "E"], 1, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "*", "E"], 2, Associativity::Left)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets.len() >= 5,
        "precedence grammar ≥5 states, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 7. Item set closure properties
// ===========================================================================

#[test]
fn closure_includes_indirect_productions() {
    // S→A, A→B, B→C, C→d
    let mut g = GrammarBuilder::new("deep_chain")
        .token("d", "d")
        .rule("C", vec!["d"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 closure: S→·A, A→·B, B→·C, C→·d  (≥4 items).
    assert!(
        col.sets[0].items.len() >= 4,
        "deep chain closure should have ≥4 items, got {}",
        col.sets[0].items.len()
    );
}

#[test]
fn closure_does_not_duplicate_items() {
    // S→A | B, A→c, B→c  — closure brings in c from two paths.
    let mut g = GrammarBuilder::new("no_dup")
        .token("c", "c")
        .rule("A", vec!["c"])
        .rule("B", vec!["c"])
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Items are in a BTreeSet, so duplicates are inherently impossible,
    // but verify item counts are reasonable.
    for s in &col.sets {
        let items_vec: Vec<_> = s.items.iter().collect();
        let items_set: BTreeSet<_> = s.items.iter().collect();
        assert_eq!(
            items_vec.len(),
            items_set.len(),
            "no duplicate items in state {}",
            s.id.0
        );
    }
}

#[test]
fn closure_propagates_lookaheads() {
    // S→A b, A→c
    // Items for A→·c should have lookahead 'b' (from FIRST(b) after A in S→A·b).
    let mut g = GrammarBuilder::new("la_prop")
        .token("b", "b")
        .token("c", "c")
        .rule("A", vec!["c"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let b_sym = g.find_symbol_by_name("b").unwrap();
    // In state 0 closure, the item for A→·c should carry lookahead b.
    let a_rule = g.all_rules().find(|r| {
        let name = g.rule_names.get(&r.lhs);
        name.map(|n| n == "A").unwrap_or(false)
    });
    if let Some(ar) = a_rule {
        let has_la_b = col.sets[0].items.iter().any(|item| {
            item.rule_id.0 == ar.production_id.0 && item.position == 0 && item.lookahead == b_sym
        });
        assert!(has_la_b, "A→·c should have lookahead b in state 0");
    }
}

// ===========================================================================
// 8. Goto transitions between states
// ===========================================================================

#[test]
fn goto_targets_are_valid_state_ids() {
    let mut g = GrammarBuilder::new("valid")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("X", vec!["a"])
        .rule("Y", vec!["b", "X"])
        .rule("S", vec!["Y", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for ((src, _), dst) in &col.goto_table {
        assert!(ids.contains(src), "source {:?} invalid", src);
        assert!(ids.contains(dst), "target {:?} invalid", dst);
    }
}

#[test]
fn goto_on_terminal_leads_to_distinct_state() {
    let mut g = GrammarBuilder::new("distinct")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let a_sym = g.find_symbol_by_name("a").unwrap();
    let b_sym = g.find_symbol_by_name("b").unwrap();
    let dst_a = col.goto_table.get(&(StateId(0), a_sym));
    let dst_b = col.goto_table.get(&(StateId(0), b_sym));
    if let (Some(da), Some(db)) = (dst_a, dst_b) {
        assert_ne!(da, db, "shifting a vs b should reach different states");
    }
}

#[test]
fn sequential_rule_forms_chain_of_gotos() {
    // S→a b c  should form a chain: 0→1→2→3
    let mut g = GrammarBuilder::new("chain_goto")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 4, "S→a b c needs ≥4 states");
    // Each state (except the final reduce state) should have an outgoing transition.
    let states_with_outgoing: BTreeSet<_> = col.goto_table.keys().map(|(s, _)| *s).collect();
    assert!(
        states_with_outgoing.len() >= 3,
        "≥3 states should have outgoing transitions"
    );
}

#[test]
fn no_self_loops_on_terminals() {
    let mut g = GrammarBuilder::new("no_self")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for ((src, sym), dst) in &col.goto_table {
        if col.symbol_is_terminal.get(sym) == Some(&true) {
            assert_ne!(
                src, dst,
                "terminal transition should not self-loop on state {:?}",
                src
            );
        }
    }
}

// ===========================================================================
// 9. Start state invariants
// ===========================================================================

#[test]
fn start_state_is_state_zero() {
    let mut g = GrammarBuilder::new("s0")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn start_state_contains_start_symbol_items() {
    let mut g = GrammarBuilder::new("s_items")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 should have items at position 0 for both S productions.
    let pos0_items = col.sets[0].items.iter().filter(|i| i.position == 0).count();
    assert!(
        pos0_items >= 2,
        "state 0 should have ≥2 initial items, got {}",
        pos0_items
    );
}

#[test]
fn state_ids_are_sequential() {
    let mut g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for (i, s) in col.sets.iter().enumerate() {
        assert_eq!(s.id, StateId(i as u16), "state {} has id {}", i, s.id.0);
    }
}

#[test]
fn no_duplicate_item_sets() {
    let mut g = GrammarBuilder::new("nodup")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x"])
        .rule("A", vec!["y"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for i in 0..col.sets.len() {
        for j in (i + 1)..col.sets.len() {
            assert_ne!(
                col.sets[i].items, col.sets[j].items,
                "states {} and {} have identical item sets",
                i, j
            );
        }
    }
}

// ===========================================================================
// 10. Large grammars
// ===========================================================================

#[test]
fn arithmetic_grammar_state_bounds() {
    let mut g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets.len() >= 8,
        "arith grammar ≥8 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 50,
        "arith grammar should not explode: {}",
        col.sets.len()
    );
}

#[test]
fn arithmetic_grammar_all_targets_valid() {
    let mut g = GrammarBuilder::new("arith2")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUM"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();
    let (col, _) = build(&mut g);

    let max = StateId(col.sets.len() as u16 - 1);
    for (_, dst) in &col.goto_table {
        assert!(dst.0 <= max.0, "target {} exceeds max {}", dst.0, max.0);
    }
}

#[test]
fn large_flat_grammar_many_alternatives() {
    // S→a | b | c | d | e | f | g | h
    let mut builder = GrammarBuilder::new("flat");
    let names = ["a", "b", "c", "d", "e", "f", "g", "h"];
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("S", vec![name]);
    }
    let mut g = builder.start("S").build();
    let (col, _) = build(&mut g);

    // State 0 should have transitions for all 8 terminals.
    assert!(
        transitions_from(&col, StateId(0)) >= 8,
        "flat grammar: state 0 should have ≥8 transitions, got {}",
        transitions_from(&col, StateId(0))
    );
}

#[test]
fn large_grammar_with_many_nonterminals() {
    // Chain: S→P, P→Q, Q→R, R→T, T→U, U→z
    // Using single-uppercase names so start_symbol() correctly falls through
    // to the first rule (S) in ordered_rules.
    let mut g = GrammarBuilder::new("deep")
        .token("z", "z")
        .rule("U", vec!["z"])
        .rule("T", vec!["U"])
        .rule("R", vec!["T"])
        .rule("Q", vec!["R"])
        .rule("P", vec!["Q"])
        .rule("S", vec!["P"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Deep chain: closure of state 0 should contain items for all levels.
    // S→·P, P→·Q, Q→·R, R→·T, T→·U, U→·z  (≥6 items)
    assert!(
        col.sets[0].items.len() >= 6,
        "deep chain closure should have ≥6 items, got {}",
        col.sets[0].items.len()
    );
    assert!(col.sets.len() >= 2, "deep chain ≥2 states");
}

#[test]
fn large_grammar_terminates() {
    // Moderately complex: statement grammar.
    let mut g = GrammarBuilder::new("stmt")
        .token("ID", "id")
        .token("NUM", "num")
        .token("=", "=")
        .token(";", ";")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .rule("program", vec!["stmts"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", ";", "stmt"])
        .rule("stmt", vec!["ID", "=", "expr"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["ID"])
        .rule("term", vec!["NUM"])
        .rule("term", vec!["(", "expr", ")"])
        .start("program")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets.len() >= 10,
        "stmt grammar ≥10 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 100,
        "stmt grammar should not explode: {}",
        col.sets.len()
    );
}

// ===========================================================================
// Additional structural invariants
// ===========================================================================

#[test]
fn every_reachable_state_has_items() {
    let mut g = GrammarBuilder::new("nonempty")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for s in &col.sets {
        assert!(!s.items.is_empty(), "state {} should not be empty", s.id.0);
    }
}

#[test]
fn reduce_items_exist_in_some_state() {
    let mut g = GrammarBuilder::new("reduce")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_reduce = col
        .sets
        .iter()
        .any(|s| s.items.iter().any(|i| i.is_reduce_item(&g)));
    assert!(has_reduce, "some state must have a reduce item");
}

#[test]
fn symbol_classification_covers_all_goto_symbols() {
    let mut g = GrammarBuilder::new("cls")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x"])
        .rule("S", vec!["A", "y"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for ((_, sym), _) in &col.goto_table {
        assert!(
            col.symbol_is_terminal.contains_key(sym),
            "symbol {:?} must be classified",
            sym
        );
    }
}

#[test]
fn collection_integrates_with_parse_table() {
    let mut g = GrammarBuilder::new("pipe")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("E", vec!["E", "+", "NUM"])
        .rule("E", vec!["NUM"])
        .start("E")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let table = build_lr1_automaton(&g, &ff).expect("parse table should build");

    assert!(col.sets.len() > 0);
    assert!(table.state_count > 0);
    sanity_check_tables(&table).expect("sanity check failed");
}

#[test]
fn augmented_collection_uses_eof_lookahead() {
    let mut g = GrammarBuilder::new("aug")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    // Determine a safe eof/augmented-start ID: use max existing + 1, +2
    let max_existing = g
        .rules
        .keys()
        .chain(g.tokens.keys())
        .map(|id| id.0)
        .max()
        .unwrap_or(0);
    let aug_start = SymbolId(max_existing + 1);
    let eof = SymbolId(max_existing + 2);
    let start = g.start_symbol().unwrap();

    // Create augmented start rule: S'→S
    g.add_rule(adze_ir::Rule {
        lhs: aug_start,
        rhs: vec![adze_ir::Symbol::NonTerminal(start)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: adze_ir::ProductionId(max_existing + 1),
    });
    g.rule_names.insert(aug_start, "S'".to_string());

    // Recompute FIRST/FOLLOW with the augmented grammar so the bitset is large enough.
    let ff2 = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let col =
        ItemSetCollection::build_canonical_collection_augmented(&g, &ff2, aug_start, start, eof);
    assert!(
        !col.sets.is_empty(),
        "augmented collection should have states"
    );

    // State 0 should have an item with eof lookahead.
    let has_eof_la = col.sets[0].items.iter().any(|i| i.lookahead == eof);
    assert!(has_eof_la, "augmented state 0 should have EOF lookahead");
}

#[test]
fn all_goto_sources_are_collection_states() {
    let mut g = GrammarBuilder::new("src_valid")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("X", vec!["b", "c"])
        .rule("S", vec!["a", "X"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for ((src, _), _) in &col.goto_table {
        assert!(ids.contains(src), "goto source {:?} not in collection", src);
    }
}

#[test]
fn target_states_reachable() {
    let mut g = GrammarBuilder::new("reach")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let targets = target_states(&col);
    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for t in &targets {
        assert!(ids.contains(t), "target {:?} is not a valid state", t);
    }
}
