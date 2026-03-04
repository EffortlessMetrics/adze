#![allow(clippy::needless_range_loop)]
#![cfg(feature = "test-api")]

//! Comprehensive tests for LR(1)/LALR construction algorithms.
//!
//! Covers: canonical collection building, FIRST/FOLLOW sets, augmented grammar
//! construction, kernel vs closure items, state transition tables, and known
//! grammar state counts.

use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build canonical collection via the standard pipeline.
fn build_collection(grammar: &mut Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

/// Build the full LR(1) parse table.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

/// Count transitions originating from the given state.
fn transition_count(col: &ItemSetCollection, state: StateId) -> usize {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == state)
        .count()
}

/// Find a symbol ID by name in a grammar built by GrammarBuilder.
/// Searches tokens then rule_names.
fn find_symbol(grammar: &Grammar, name: &str) -> SymbolId {
    for (&id, tok) in &grammar.tokens {
        if tok.name == name {
            return id;
        }
    }
    for (&id, rname) in &grammar.rule_names {
        if rname == name {
            return id;
        }
    }
    panic!("symbol '{}' not found in grammar", name);
}

// ===========================================================================
// 1. FIRST set tests
// ===========================================================================

#[test]
fn first_set_single_terminal_rule() {
    let mut g = GrammarBuilder::new("ff1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = find_symbol(&g, "start");
    let a = find_symbol(&g, "a");

    let first_s = ff.first(s).expect("FIRST(start) must exist");
    assert!(
        first_s.contains(a.0 as usize),
        "FIRST(start) must contain 'a'"
    );
}

#[test]
fn first_set_two_alternatives() {
    let mut g = GrammarBuilder::new("ff2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = find_symbol(&g, "start");
    let a = find_symbol(&g, "a");
    let b = find_symbol(&g, "b");

    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(a.0 as usize));
    assert!(first_s.contains(b.0 as usize));
}

#[test]
fn first_set_indirect_nonterminal() {
    let mut g = GrammarBuilder::new("ff3")
        .token("a", "a")
        .token("b", "b")
        .rule("alt", vec!["a"])
        .rule("alt", vec!["b"])
        .rule("start", vec!["alt"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = find_symbol(&g, "start");
    let a = find_symbol(&g, "a");
    let b = find_symbol(&g, "b");

    let first_s = ff.first(s).unwrap();
    assert!(
        first_s.contains(a.0 as usize),
        "FIRST(start) must contain 'a'"
    );
    assert!(
        first_s.contains(b.0 as usize),
        "FIRST(start) must contain 'b'"
    );
}

#[test]
fn first_set_left_recursive_grammar() {
    let mut g = GrammarBuilder::new("ff4")
        .token("n", "n")
        .token("+", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = find_symbol(&g, "expr");
    let n = find_symbol(&g, "n");

    let first_e = ff.first(e).unwrap();
    assert!(
        first_e.contains(n.0 as usize),
        "FIRST(expr) must contain 'n'"
    );
}

#[test]
fn first_set_nullable_nonterminal() {
    let mut g = GrammarBuilder::new("ff5")
        .token("a", "a")
        .token("b", "b")
        .rule("opt_a", vec![])
        .rule("opt_a", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["opt_a", "item"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = find_symbol(&g, "start");
    let a = find_symbol(&g, "a");
    let b = find_symbol(&g, "b");

    let first_s = ff.first(s).unwrap();
    assert!(
        first_s.contains(a.0 as usize),
        "FIRST(start) must contain 'a'"
    );
    assert!(
        first_s.contains(b.0 as usize),
        "FIRST(start) must contain 'b' via nullable opt_a"
    );
}

// ===========================================================================
// 2. FOLLOW set tests
// ===========================================================================

#[test]
fn follow_set_start_contains_eof() {
    let mut g = GrammarBuilder::new("fo1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = find_symbol(&g, "start");

    let follow_s = ff.follow(s).expect("FOLLOW(start) must exist");
    assert!(
        follow_s.contains(0),
        "FOLLOW(start) must contain EOF (id 0)"
    );
}

#[test]
fn follow_set_nonterminal_before_terminal() {
    let mut g = GrammarBuilder::new("fo2")
        .token("a", "a")
        .token("b", "b")
        .rule("prefix", vec!["a"])
        .rule("start", vec!["prefix", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pfx = find_symbol(&g, "prefix");
    let b = find_symbol(&g, "b");

    let follow_pfx = ff.follow(pfx).unwrap();
    assert!(
        follow_pfx.contains(b.0 as usize),
        "FOLLOW(prefix) must contain 'b'"
    );
}

#[test]
fn follow_propagation_from_lhs() {
    let mut g = GrammarBuilder::new("fo3")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let inner = find_symbol(&g, "inner");

    let follow_inner = ff.follow(inner).unwrap();
    assert!(
        follow_inner.contains(0),
        "FOLLOW(inner) must contain EOF via FOLLOW(start)"
    );
}

#[test]
fn follow_set_multiple_rhs_positions() {
    let mut g = GrammarBuilder::new("fo4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("p1", vec!["a"])
        .rule("p2", vec!["b"])
        .rule("p3", vec!["c"])
        .rule("start", vec!["p1", "p2", "p3"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let p1 = find_symbol(&g, "p1");
    let p2 = find_symbol(&g, "p2");
    let b_tok = find_symbol(&g, "b");
    let c_tok = find_symbol(&g, "c");

    assert!(ff.follow(p1).unwrap().contains(b_tok.0 as usize));
    assert!(ff.follow(p2).unwrap().contains(c_tok.0 as usize));
}

// ===========================================================================
// 3. Nullability tests
// ===========================================================================

#[test]
fn nullable_epsilon_rule() {
    let mut g = GrammarBuilder::new("nu1")
        .token("a", "a")
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .rule("start", vec!["opt", "a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let opt = find_symbol(&g, "opt");

    assert!(
        ff.is_nullable(opt),
        "opt with ε production must be nullable"
    );
}

#[test]
fn non_nullable_terminal_only() {
    let mut g = GrammarBuilder::new("nu2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = find_symbol(&g, "start");

    assert!(!ff.is_nullable(s), "start -> 'a' is not nullable");
}

#[test]
fn transitively_nullable() {
    let mut g = GrammarBuilder::new("nu3")
        .rule("leaf", vec![])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = find_symbol(&g, "start");
    let m = find_symbol(&g, "mid");
    let l = find_symbol(&g, "leaf");

    assert!(ff.is_nullable(l));
    assert!(ff.is_nullable(m));
    assert!(ff.is_nullable(s));
}

// ===========================================================================
// 4. Canonical collection – basic structure
// ===========================================================================

#[test]
fn single_rule_collection_has_states() {
    let mut g = GrammarBuilder::new("cc1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2, "start->a needs at least 2 states");
}

#[test]
fn initial_state_is_zero() {
    let mut g = GrammarBuilder::new("cc2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn initial_state_nonempty() {
    let mut g = GrammarBuilder::new("cc3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets[0].items.is_empty());
}

#[test]
fn two_terminal_rule_more_states() {
    let mut g = GrammarBuilder::new("cc4")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() >= 3,
        "start->ab needs ≥3 states, got {}",
        col.sets.len()
    );
}

#[test]
fn state_ids_are_unique() {
    let mut g = GrammarBuilder::new("cc5")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    assert_eq!(ids.len(), col.sets.len(), "state IDs must be unique");
}

#[test]
fn collection_with_left_recursion() {
    let mut g = GrammarBuilder::new("cc6")
        .token("n", "n")
        .token("+", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .start("expr")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() >= 4,
        "left-recursive expr+term needs several states"
    );
}

// ===========================================================================
// 5. GOTO transitions
// ===========================================================================

#[test]
fn initial_state_has_transitions() {
    let mut g = GrammarBuilder::new("gt1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        transition_count(&col, StateId(0)) >= 1,
        "state 0 must have outgoing edges"
    );
}

#[test]
fn goto_table_references_valid_states() {
    let mut g = GrammarBuilder::new("gt2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    let max_id = col.sets.iter().map(|s| s.id).max().unwrap();
    for &tgt in col.goto_table.values() {
        assert!(
            tgt.0 <= max_id.0,
            "goto target {} exceeds max state {}",
            tgt.0,
            max_id.0
        );
    }
}

#[test]
fn goto_table_tracks_terminal_nonterminal() {
    let mut g = GrammarBuilder::new("gt3")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    let a = find_symbol(&g, "a");
    let inner = find_symbol(&g, "inner");

    if let Some(&is_term) = col.symbol_is_terminal.get(&a) {
        assert!(is_term, "'a' should be marked as terminal");
    }
    if let Some(&is_term) = col.symbol_is_terminal.get(&inner) {
        assert!(!is_term, "'inner' should be marked as non-terminal");
    }
}

// ===========================================================================
// 6. Kernel vs closure items
// ===========================================================================

#[test]
fn closure_adds_items_for_nonterminal() {
    let mut g = GrammarBuilder::new("kc1")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);

    let state0 = &col.sets[0];
    let rule_ids: BTreeSet<_> = state0.items.iter().map(|item| item.rule_id).collect();
    assert!(
        rule_ids.len() >= 2,
        "closure should add items from inner's rules"
    );
}

#[test]
fn kernel_items_dot_at_nonzero_position() {
    let mut g = GrammarBuilder::new("kc2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    let a = find_symbol(&g, "a");

    if let Some(&target) = col.goto_table.get(&(StateId(0), a)) {
        let target_set = col.sets.iter().find(|s| s.id == target).unwrap();
        assert!(
            target_set.items.iter().any(|item| item.position >= 1),
            "after shift, kernel items should have position >= 1"
        );
    }
}

// ===========================================================================
// 7. Augmented grammar construction via build_lr1_automaton
// ===========================================================================

#[test]
fn augmented_grammar_produces_accept_action() {
    let g = GrammarBuilder::new("ag1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();

    let has_accept = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(
        has_accept,
        "augmented grammar must produce an Accept action"
    );
}

#[test]
fn augmented_grammar_has_shift_on_terminal() {
    let g = GrammarBuilder::new("ag2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = find_symbol(&g, "a");

    let has_shift = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), a)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    });
    assert!(has_shift, "should have Shift action on terminal 'a'");
}

#[test]
fn augmented_grammar_start_symbol_preserved() {
    let g = GrammarBuilder::new("ag3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = find_symbol(&g, "start");
    assert_eq!(
        table.start_symbol(),
        s,
        "start symbol must be the original start"
    );
}

#[test]
fn augmented_grammar_eof_not_zero() {
    let g = GrammarBuilder::new("ag4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_ne!(
        table.eof(),
        SymbolId(0),
        "EOF should be a fresh symbol, not 0"
    );
}

// ===========================================================================
// 8. State counts for known grammars
// ===========================================================================

#[test]
fn state_count_single_terminal() {
    let g = GrammarBuilder::new("sc1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3 && table.state_count <= 6,
        "start->a should have 3-6 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_two_alternatives() {
    let g = GrammarBuilder::new("sc2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3,
        "start->a|b should have ≥3 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_chain_grammar() {
    let g = GrammarBuilder::new("sc3")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3,
        "chain start->mid->leaf->x should have ≥3 states, got {}",
        table.state_count
    );
}

#[test]
fn state_count_left_recursive() {
    let g = GrammarBuilder::new("sc4")
        .token("a", "a")
        .rule("lst", vec!["a"])
        .rule("lst", vec!["lst", "a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3,
        "left-recursive lst needs ≥3 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 9. Parse table structural invariants
// ===========================================================================

#[test]
fn parse_table_has_rules() {
    let g = GrammarBuilder::new("pt1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(!table.rules.is_empty(), "parse table must have rules");
}

#[test]
fn parse_table_symbol_mapping_consistent() {
    let g = GrammarBuilder::new("pt2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);

    for (&sym, &idx) in &table.symbol_to_index {
        assert_eq!(
            table.index_to_symbol[idx], sym,
            "index_to_symbol[{}] should map back to {:?}",
            idx, sym
        );
    }
}

#[test]
fn parse_table_action_table_dimensions() {
    let g = GrammarBuilder::new("pt3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);

    assert_eq!(table.action_table.len(), table.state_count);
    for row in &table.action_table {
        assert!(!row.is_empty(), "action row should not be empty");
    }
}

#[test]
fn parse_table_goto_table_dimensions() {
    let g = GrammarBuilder::new("pt4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);

    assert_eq!(table.goto_table.len(), table.state_count);
}

// ===========================================================================
// 10. Reduce items and shift items coexist
// ===========================================================================

#[test]
fn collection_has_reduce_items() {
    let mut g = GrammarBuilder::new("ri1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);

    let has_reduce = col
        .sets
        .iter()
        .any(|set| set.items.iter().any(|item| item.is_reduce_item(&g)));
    assert!(
        has_reduce,
        "some state must have a reduce item for start->a"
    );
}

#[test]
fn collection_has_shift_items() {
    let mut g = GrammarBuilder::new("ri2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);

    let has_shift = col.sets.iter().any(|set| {
        set.items
            .iter()
            .any(|item| !item.is_reduce_item(&g) && item.next_symbol(&g).is_some())
    });
    assert!(has_shift, "some state must have a shift item");
}

// ===========================================================================
// 11. Expression grammar (classic)
// ===========================================================================

#[test]
fn expr_grammar_first_follow() {
    let mut g = GrammarBuilder::new("expr")
        .token("id", "id")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("factor", vec!["id"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = find_symbol(&g, "expr");
    let t = find_symbol(&g, "term");
    let f = find_symbol(&g, "factor");
    let id_tok = find_symbol(&g, "id");
    let lparen = find_symbol(&g, "(");

    let first_f = ff.first(f).unwrap();
    assert!(first_f.contains(id_tok.0 as usize));
    assert!(first_f.contains(lparen.0 as usize));

    let first_t = ff.first(t).unwrap();
    assert!(first_t.contains(id_tok.0 as usize));
    assert!(first_t.contains(lparen.0 as usize));

    let first_e = ff.first(e).unwrap();
    assert!(first_e.contains(id_tok.0 as usize));
    assert!(first_e.contains(lparen.0 as usize));
}

#[test]
fn expr_grammar_follow_sets() {
    let mut g = GrammarBuilder::new("expr_fo")
        .token("id", "id")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("factor", vec!["id"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = find_symbol(&g, "expr");
    let plus = find_symbol(&g, "+");
    let rparen = find_symbol(&g, ")");

    let follow_e = ff.follow(e).unwrap();
    assert!(follow_e.contains(0), "FOLLOW(expr) must contain EOF");
    assert!(
        follow_e.contains(rparen.0 as usize),
        "FOLLOW(expr) must contain ')'"
    );
    assert!(
        follow_e.contains(plus.0 as usize),
        "FOLLOW(expr) must contain '+'"
    );
}

#[test]
fn expr_grammar_state_count_reasonable() {
    let g = GrammarBuilder::new("expr_sc")
        .token("id", "id")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("factor", vec!["id"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 8 && table.state_count <= 40,
        "expression grammar should have 8-40 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 12. Multiple nonterminals / deeper grammars
// ===========================================================================

#[test]
fn collection_multiple_nonterminals() {
    let mut g = GrammarBuilder::new("mn1")
        .token("a", "a")
        .token("b", "b")
        .rule("part_a", vec!["a"])
        .rule("part_b", vec!["b"])
        .rule("start", vec!["part_a", "part_b"])
        .start("start")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() >= 3,
        "start->AB needs ≥3 states, got {}",
        col.sets.len()
    );
}

#[test]
fn right_recursive_grammar() {
    let mut g = GrammarBuilder::new("rr1")
        .token("a", "a")
        .rule("rlist", vec!["a"])
        .rule("rlist", vec!["a", "rlist"])
        .start("rlist")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() >= 3,
        "right-recursive rlist needs ≥3 states, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 13. Augmented collection via build_canonical_collection_augmented
// ===========================================================================

#[test]
fn augmented_collection_has_more_states_or_equal() {
    let mut g = GrammarBuilder::new("ac1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col_basic = ItemSetCollection::build_canonical_collection(&g, &ff);

    let original_start = g.start_symbol().unwrap();
    let max_id = g
        .tokens
        .keys()
        .chain(g.rule_names.keys())
        .map(|s| s.0)
        .max()
        .unwrap_or(0);
    let eof = SymbolId(max_id + 1);
    let aug_start = SymbolId(max_id + 2);
    let max_prod = g.all_rules().map(|r| r.production_id.0).max().unwrap_or(0);

    let mut aug_g = g.clone();
    aug_g.rules.insert(
        aug_start,
        vec![Rule {
            lhs: aug_start,
            rhs: vec![Symbol::NonTerminal(original_start)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(max_prod + 1),
        }],
    );
    aug_g.rule_names.insert(aug_start, "$start".to_string());

    let ff_aug = FirstFollowSets::compute(&aug_g).unwrap();
    let col_aug = ItemSetCollection::build_canonical_collection_augmented(
        &aug_g,
        &ff_aug,
        aug_start,
        original_start,
        eof,
    );

    assert!(
        col_aug.sets.len() >= col_basic.sets.len(),
        "augmented collection should have ≥ basic states"
    );
}

// ===========================================================================
// 14. Epsilon / empty rule handling (via full parse table)
// ===========================================================================

#[test]
fn epsilon_rule_builds_table() {
    let g = GrammarBuilder::new("ep1")
        .token("a", "a")
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .rule("start", vec!["opt", "a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 2,
        "grammar with ε rule should produce states"
    );
}

// ===========================================================================
// 15. Conflict detection (shift-reduce)
// ===========================================================================

#[test]
fn ambiguous_grammar_builds_without_panic() {
    let g = GrammarBuilder::new("sr1")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["n"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let _table = build_lr1_automaton(&g, &ff).unwrap();
}

// ===========================================================================
// 16. build_lr1_automaton result checks
// ===========================================================================

#[test]
fn lr1_automaton_has_correct_token_count() {
    let g = GrammarBuilder::new("tc1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.token_count >= 2,
        "should have ≥2 tokens, got {}",
        table.token_count
    );
}

#[test]
fn lr1_automaton_goto_on_nonterminal() {
    let g = GrammarBuilder::new("gnt1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = find_symbol(&g, "start");

    let goto = table.goto(table.initial_state, s);
    assert!(goto.is_some(), "goto(initial, start) should exist");
}

#[test]
fn lr1_automaton_no_accept_on_non_eof() {
    let g = GrammarBuilder::new("nae1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = find_symbol(&g, "a");

    let has_accept_on_a = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), a)
            .iter()
            .any(|act| matches!(act, Action::Accept))
    });
    assert!(
        !has_accept_on_a,
        "Accept should only appear on EOF, not on terminal 'a'"
    );
}

#[test]
fn lr1_automaton_reduce_action_exists() {
    let g = GrammarBuilder::new("ra1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();

    let has_reduce = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|act| matches!(act, Action::Reduce(_)))
    });
    let has_accept = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|act| matches!(act, Action::Accept))
    });
    assert!(
        has_reduce || has_accept,
        "table must have Reduce or Accept on EOF"
    );
}

// ===========================================================================
// 17. Larger grammar stress test
// ===========================================================================

#[test]
fn larger_grammar_with_many_rules() {
    let g = GrammarBuilder::new("lg1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("suffix", vec!["d"])
        .rule("alt_a", vec!["a", "suffix"])
        .rule("alt_b", vec!["b", "suffix"])
        .rule("alt_c", vec!["c", "suffix"])
        .rule("start", vec!["alt_a"])
        .rule("start", vec!["alt_b"])
        .rule("start", vec!["alt_c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 5,
        "larger grammar needs ≥5 states, got {}",
        table.state_count
    );
}

#[test]
fn list_grammar_state_count() {
    let g = GrammarBuilder::new("list")
        .token("x", "x")
        .token(",", ",")
        .rule("item", vec!["x"])
        .rule("lst", vec!["item"])
        .rule("lst", vec!["lst", ",", "item"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 4 && table.state_count <= 20,
        "list grammar should have 4-20 states, got {}",
        table.state_count
    );
}
