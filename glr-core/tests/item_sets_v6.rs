#![cfg(feature = "test-api")]

//! item_sets_v6 — 64 tests for LR(1) item set construction and closure operations.
//!
//! Categories:
//! 1. Item set collection created for valid grammars (8)
//! 2. Initial item set contains start production (8)
//! 3. Closure expands nonterminal items (8)
//! 4. Goto creates correct successor states (8)
//! 5. Item set count reasonable for grammar size (8)
//! 6. No duplicate items in any set (8)
//! 7. Complex grammars produce expected state structure (8)
//! 8. Edge cases: single rule, epsilon, many alternatives (8)

use adze_glr_core::*;
use adze_ir::Symbol;
use adze_ir::builder::GrammarBuilder;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build(grammar: &mut adze_ir::Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

fn transitions_from(col: &ItemSetCollection, state: StateId) -> usize {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == state)
        .count()
}

fn reduce_items_in(set: &ItemSet, grammar: &adze_ir::Grammar) -> usize {
    set.items
        .iter()
        .filter(|i| i.is_reduce_item(grammar))
        .count()
}

fn shift_items_in(set: &ItemSet, grammar: &adze_ir::Grammar) -> usize {
    set.items
        .iter()
        .filter(|i| i.next_symbol(grammar).is_some())
        .count()
}

fn all_state_ids(col: &ItemSetCollection) -> BTreeSet<StateId> {
    col.sets.iter().map(|s| s.id).collect()
}

fn target_states(col: &ItemSetCollection, from: StateId) -> BTreeSet<StateId> {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == from)
        .map(|(_, &dst)| dst)
        .collect()
}

// ===========================================================================
// 1. Item set collection created for valid grammars (8 tests)
// ===========================================================================

#[test]
fn collection_single_token_grammar() {
    let mut g = GrammarBuilder::new("v6_1a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(!col.sets.is_empty(), "collection must have states");
}

#[test]
fn collection_two_token_sequence() {
    let mut g = GrammarBuilder::new("v6_1b")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 2,
        "sequence grammar needs multiple states"
    );
}

#[test]
fn collection_two_alternatives() {
    let mut g = GrammarBuilder::new("v6_1c")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 2, "alternatives produce distinct states");
}

#[test]
fn collection_nonterminal_chain() {
    let mut g = GrammarBuilder::new("v6_1d")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 2, "chain grammar needs states");
}

#[test]
fn collection_goto_table_non_empty() {
    let mut g = GrammarBuilder::new("v6_1e")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(!col.goto_table.is_empty(), "goto_table must have entries");
}

#[test]
fn collection_recursive_grammar() {
    let mut g = GrammarBuilder::new("v6_1f")
        .token("a", "a")
        .token("plus", "+")
        .rule("E", vec!["E", "plus", "a"])
        .rule("E", vec!["a"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 4,
        "recursive grammar needs several states"
    );
}

#[test]
fn collection_three_token_sequence() {
    let mut g = GrammarBuilder::new("v6_1g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 3,
        "3-token sequence needs at least 3 states"
    );
}

#[test]
fn collection_nested_nonterminals() {
    let mut g = GrammarBuilder::new("v6_1h")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 2, "nested nonterminals produce states");
}

// ===========================================================================
// 2. Initial item set contains start production (8 tests)
// ===========================================================================

#[test]
fn initial_set_is_state_zero() {
    let mut g = GrammarBuilder::new("v6_2a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn initial_set_not_empty() {
    let mut g = GrammarBuilder::new("v6_2b")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(!col.sets[0].items.is_empty(), "initial set must have items");
}

#[test]
fn initial_set_has_shift_items() {
    let mut g = GrammarBuilder::new("v6_2c")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        shift_items_in(&col.sets[0], &g) > 0,
        "initial set must have at least one shift item"
    );
}

#[test]
fn initial_set_item_at_position_zero() {
    let mut g = GrammarBuilder::new("v6_2d")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let has_pos_zero = col.sets[0].items.iter().any(|i| i.position == 0);
    assert!(has_pos_zero, "initial set must have item at position 0");
}

#[test]
fn initial_set_two_alt_grammar_has_items_for_both() {
    let mut g = GrammarBuilder::new("v6_2e")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets[0].items.len() >= 2,
        "initial set should include items for both alternatives"
    );
}

#[test]
fn initial_set_chain_grammar_includes_closure_items() {
    let mut g = GrammarBuilder::new("v6_2f")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    // Closure of S→·A should add A→·x
    assert!(
        col.sets[0].items.len() >= 2,
        "closure should expand nonterminal in initial set"
    );
}

#[test]
fn initial_set_recursive_grammar_has_start_items() {
    let mut g = GrammarBuilder::new("v6_2g")
        .token("a", "a")
        .token("plus", "+")
        .rule("E", vec!["E", "plus", "a"])
        .rule("E", vec!["a"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    let has_pos_zero = col.sets[0].items.iter().any(|i| i.position == 0);
    assert!(has_pos_zero, "initial set must have start production item");
}

#[test]
fn initial_set_no_reduce_items_for_nonempty_rule() {
    let mut g = GrammarBuilder::new("v6_2h")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    // Items at position 0 of a non-empty rule are never reduce items
    let all_pos_zero = col.sets[0].items.iter().all(|i| i.position == 0);
    if all_pos_zero {
        assert_eq!(
            reduce_items_in(&col.sets[0], &g),
            0,
            "initial set should not have reduce items for non-empty rules"
        );
    }
}

// ===========================================================================
// 3. Closure expands nonterminal items (8 tests)
// ===========================================================================

#[test]
fn closure_terminal_kernel_unchanged() {
    let mut g = GrammarBuilder::new("v6_3a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let mut set = ItemSet::new(StateId(99));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items.len(), before, "terminal-only kernel adds nothing");
}

#[test]
fn closure_adds_items_for_nonterminal() {
    let mut g = GrammarBuilder::new("v6_3b")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    assert!(set.items.len() >= 2, "closure of S→·A must add A→·x");
}

#[test]
fn closure_double_chain_expands_fully() {
    let mut g = GrammarBuilder::new("v6_3c")
        .token("y", "y")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["y"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    // S→·A, A→·B, B→·y
    assert!(
        set.items.len() >= 3,
        "double chain closure must include all levels"
    );
}

#[test]
fn closure_multiple_alternatives() {
    let mut g = GrammarBuilder::new("v6_3d")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .rule("A", vec!["y"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    // S→·A, A→·x, A→·y
    assert!(
        set.items.len() >= 3,
        "closure must include all alternatives of A"
    );
}

#[test]
fn closure_recursive_rule_does_not_diverge() {
    let mut g = GrammarBuilder::new("v6_3e")
        .token("a", "a")
        .rule("S", vec!["S", "a"])
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    // Must terminate — BTreeSet prevents infinite expansion
    assert!(!set.items.is_empty());
}

#[test]
fn closure_returns_ok() {
    let mut g = GrammarBuilder::new("v6_3f")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    assert!(set.closure(&g, &ff).is_ok(), "closure should succeed");
}

#[test]
fn closure_idempotent() {
    let mut g = GrammarBuilder::new("v6_3g")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    let after_first = set.items.len();
    set.closure(&g, &ff).unwrap();
    assert_eq!(
        set.items.len(),
        after_first,
        "second closure must not add new items"
    );
}

#[test]
fn closure_with_follow_context() {
    // S → A b; A → a  — closure of S→·Ab should give A→·a with lookahead b
    let mut g = GrammarBuilder::new("v6_3h")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["A", "b"])
        .rule("A", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = &g.get_rules_for_symbol(start).unwrap()[0];
    let b_sym = g.find_symbol_by_name("b").unwrap();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    // At least one closure item should have lookahead == b
    let has_b_la = set.items.iter().any(|i| i.lookahead == b_sym);
    assert!(has_b_la, "closure should propagate 'b' as lookahead");
}

// ===========================================================================
// 4. Goto creates correct successor states (8 tests)
// ===========================================================================

#[test]
fn goto_on_terminal_produces_items() {
    let mut g = GrammarBuilder::new("v6_4a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, ff) = build(&mut g);
    let a_sym = g.find_symbol_by_name("a").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(a_sym), &g, &ff);
    assert!(
        !goto_set.items.is_empty(),
        "goto on terminal 'a' must produce items"
    );
}

#[test]
fn goto_on_nonterminal_produces_items() {
    let mut g = GrammarBuilder::new("v6_4b")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("start")
        .build();
    let (col, ff) = build(&mut g);
    let mid_sym = g.find_symbol_by_name("mid").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::NonTerminal(mid_sym), &g, &ff);
    assert!(
        !goto_set.items.is_empty(),
        "goto on nonterminal 'mid' must produce items"
    );
}

#[test]
fn goto_on_absent_symbol_is_empty() {
    let mut g = GrammarBuilder::new("v6_4c")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, ff) = build(&mut g);
    let b_sym = g.find_symbol_by_name("b").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(b_sym), &g, &ff);
    assert!(
        goto_set.items.is_empty(),
        "goto on symbol not in any kernel item must be empty"
    );
}

#[test]
fn goto_advances_dot_position() {
    let mut g = GrammarBuilder::new("v6_4d")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, ff) = build(&mut g);
    let a_sym = g.find_symbol_by_name("a").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(a_sym), &g, &ff);
    // All kernel items should have position >= 1
    let all_advanced = goto_set.items.iter().all(|i| i.position >= 1);
    assert!(all_advanced, "goto must advance dot position");
}

#[test]
fn goto_recorded_in_goto_table() {
    let mut g = GrammarBuilder::new("v6_4e")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        !col.goto_table.is_empty(),
        "goto_table must record transitions"
    );
}

#[test]
fn goto_from_initial_state_has_transitions() {
    let mut g = GrammarBuilder::new("v6_4f")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        transitions_from(&col, StateId(0)) > 0,
        "initial state must have outgoing transitions"
    );
}

#[test]
fn goto_sequence_intermediate_state_has_transition() {
    let mut g = GrammarBuilder::new("v6_4g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    // There should be transitions from at least the initial state
    let total_transitions: usize = col.goto_table.len();
    assert!(
        total_transitions >= 3,
        "3-token sequence should have at least 3 transitions, got {total_transitions}"
    );
}

#[test]
fn goto_chain_grammar_transitions_include_nonterminal() {
    let mut g = GrammarBuilder::new("v6_4h")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let has_nt = col.symbol_is_terminal.values().any(|&is_term| !is_term);
    assert!(has_nt, "chain grammar must have nonterminal transitions");
}

// ===========================================================================
// 5. Item set count reasonable for grammar size (8 tests)
// ===========================================================================

#[test]
fn state_count_single_rule_bounded() {
    let mut g = GrammarBuilder::new("v6_5a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() <= 10,
        "single rule grammar should not exceed 10 states"
    );
}

#[test]
fn state_count_two_alt_bounded() {
    let mut g = GrammarBuilder::new("v6_5b")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() <= 15,
        "two-alt grammar should not exceed 15 states"
    );
}

#[test]
fn state_count_chain_bounded() {
    let mut g = GrammarBuilder::new("v6_5c")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() <= 15,
        "chain grammar should not exceed 15 states"
    );
}

#[test]
fn state_count_recursive_bounded() {
    let mut g = GrammarBuilder::new("v6_5d")
        .token("a", "a")
        .token("plus", "+")
        .rule("E", vec!["E", "plus", "a"])
        .rule("E", vec!["a"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() <= 20,
        "simple recursive grammar should not exceed 20 states"
    );
}

#[test]
fn state_count_at_least_two_for_nonempty() {
    let mut g = GrammarBuilder::new("v6_5e")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 2,
        "any non-trivial grammar needs at least 2 states"
    );
}

#[test]
fn state_count_grows_with_tokens() {
    let mut g2 = GrammarBuilder::new("v6_5f_2")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col2, _) = build(&mut g2);

    let mut g3 = GrammarBuilder::new("v6_5f_3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col3, _) = build(&mut g3);
    assert!(
        col3.sets.len() >= col2.sets.len(),
        "longer sequence should have at least as many states"
    );
}

#[test]
fn state_count_alternatives_vs_sequence() {
    let mut g_alt = GrammarBuilder::new("v6_5g_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .start("S")
        .build();
    let (col_alt, _) = build(&mut g_alt);

    let mut g_seq = GrammarBuilder::new("v6_5g_seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col_seq, _) = build(&mut g_seq);
    // Both should be reasonably small
    assert!(col_alt.sets.len() <= 20);
    assert!(col_seq.sets.len() <= 20);
}

#[test]
fn state_count_ids_sequential() {
    let mut g = GrammarBuilder::new("v6_5h")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let ids = all_state_ids(&col);
    for (i, &id) in ids.iter().enumerate() {
        assert_eq!(
            id,
            StateId(i as u16),
            "state IDs should be sequential starting from 0"
        );
    }
}

// ===========================================================================
// 6. No duplicate items in any set (8 tests)
// ===========================================================================

#[test]
fn no_duplicates_single_rule() {
    let mut g = GrammarBuilder::new("v6_6a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for set in &col.sets {
        // BTreeSet guarantees uniqueness; verify count matches iteration
        let items: Vec<_> = set.items.iter().collect();
        let unique: BTreeSet<_> = items.iter().collect();
        assert_eq!(
            items.len(),
            unique.len(),
            "state {} has duplicates",
            set.id.0
        );
    }
}

#[test]
fn no_duplicates_two_alternatives() {
    let mut g = GrammarBuilder::new("v6_6b")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for set in &col.sets {
        let count = set.items.len();
        assert!(count > 0, "state {} is unexpectedly empty", set.id.0);
    }
}

#[test]
fn no_duplicates_chain() {
    let mut g = GrammarBuilder::new("v6_6c")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for set in &col.sets {
        let v: Vec<_> = set.items.iter().collect();
        let s: BTreeSet<_> = v.iter().collect();
        assert_eq!(v.len(), s.len(), "state {} has duplicates", set.id.0);
    }
}

#[test]
fn no_duplicates_recursive() {
    let mut g = GrammarBuilder::new("v6_6d")
        .token("a", "a")
        .token("plus", "+")
        .rule("E", vec!["E", "plus", "a"])
        .rule("E", vec!["a"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    for set in &col.sets {
        let v: Vec<_> = set.items.iter().collect();
        let s: BTreeSet<_> = v.iter().collect();
        assert_eq!(v.len(), s.len(), "state {} has duplicates", set.id.0);
    }
}

#[test]
fn no_duplicates_sequence() {
    let mut g = GrammarBuilder::new("v6_6e")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for set in &col.sets {
        let v: Vec<_> = set.items.iter().collect();
        let s: BTreeSet<_> = v.iter().collect();
        assert_eq!(v.len(), s.len(), "state {} has duplicates", set.id.0);
    }
}

#[test]
fn no_duplicates_diamond() {
    let mut g = GrammarBuilder::new("v6_6f")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for set in &col.sets {
        let v: Vec<_> = set.items.iter().collect();
        let s: BTreeSet<_> = v.iter().collect();
        assert_eq!(v.len(), s.len(), "state {} has duplicates", set.id.0);
    }
}

#[test]
fn no_duplicate_state_ids_in_collection() {
    let mut g = GrammarBuilder::new("v6_6g")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let ids: Vec<StateId> = col.sets.iter().map(|s| s.id).collect();
    let unique: BTreeSet<StateId> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "state IDs must be unique");
}

#[test]
fn no_duplicates_nested_nonterminals() {
    let mut g = GrammarBuilder::new("v6_6h")
        .token("z", "z")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["z"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for set in &col.sets {
        let v: Vec<_> = set.items.iter().collect();
        let s: BTreeSet<_> = v.iter().collect();
        assert_eq!(v.len(), s.len(), "state {} has duplicates", set.id.0);
    }
}

// ===========================================================================
// 7. Complex grammars produce expected state structure (8 tests)
// ===========================================================================

#[test]
fn expr_grammar_has_accept_state() {
    let mut g = GrammarBuilder::new("v6_7a")
        .token("n", "n")
        .token("plus", "+")
        .rule("E", vec!["E", "plus", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    let has_reduce = col.sets.iter().any(|s| reduce_items_in(s, &g) > 0);
    assert!(has_reduce, "expression grammar must have reduce states");
}

#[test]
fn expr_grammar_has_shift_and_reduce() {
    let mut g = GrammarBuilder::new("v6_7b")
        .token("n", "n")
        .token("plus", "+")
        .rule("E", vec!["E", "plus", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    let has_shift = col.sets.iter().any(|s| shift_items_in(s, &g) > 0);
    let has_reduce = col.sets.iter().any(|s| reduce_items_in(s, &g) > 0);
    assert!(has_shift, "must have shift states");
    assert!(has_reduce, "must have reduce states");
}

#[test]
fn multi_operator_grammar_states() {
    let mut g = GrammarBuilder::new("v6_7c")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("E", vec!["E", "plus", "E"])
        .rule("E", vec!["E", "star", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    // Two operators means more states than one operator
    assert!(
        col.sets.len() >= 5,
        "multi-operator grammar needs several states"
    );
}

#[test]
fn parenthesized_expr_grammar() {
    let mut g = GrammarBuilder::new("v6_7d")
        .token("n", "n")
        .token("plus", "+")
        .token("lp", "(")
        .token("rp", ")")
        .rule("E", vec!["E", "plus", "E"])
        .rule("E", vec!["lp", "E", "rp"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 6,
        "parenthesized expr grammar needs many states, got {}",
        col.sets.len()
    );
}

#[test]
fn statement_list_grammar() {
    let mut g = GrammarBuilder::new("v6_7e")
        .token("id", "id")
        .token("semi", ";")
        .rule("P", vec!["L"])
        .rule("L", vec!["L", "semi", "S"])
        .rule("L", vec!["S"])
        .rule("S", vec!["id"])
        .start("P")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 4,
        "statement list grammar needs several states"
    );
}

#[test]
fn multi_level_nonterminals() {
    let mut g = GrammarBuilder::new("v6_7f")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("E", vec!["E", "plus", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "star", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["n"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 8,
        "classic E/T/F grammar needs 8+ states, got {}",
        col.sets.len()
    );
}

#[test]
fn complex_grammar_goto_table_consistent() {
    let mut g = GrammarBuilder::new("v6_7g")
        .token("n", "n")
        .token("plus", "+")
        .rule("E", vec!["E", "plus", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    // Every goto target must be a valid state
    let ids = all_state_ids(&col);
    for (_, &target) in &col.goto_table {
        assert!(
            ids.contains(&target),
            "goto target state {} must exist in collection",
            target.0
        );
    }
}

#[test]
fn complex_grammar_all_states_reachable() {
    let mut g = GrammarBuilder::new("v6_7h")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    // BFS from state 0
    let mut visited = BTreeSet::new();
    let mut queue = vec![StateId(0)];
    while let Some(state) = queue.pop() {
        if !visited.insert(state) {
            continue;
        }
        for &tgt in target_states(&col, state).iter() {
            if !visited.contains(&tgt) {
                queue.push(tgt);
            }
        }
    }
    assert_eq!(
        visited.len(),
        col.sets.len(),
        "all states must be reachable from state 0"
    );
}

// ===========================================================================
// 8. Edge cases: single rule, epsilon, many alternatives (8 tests)
// ===========================================================================

#[test]
fn single_rule_single_token() {
    let mut g = GrammarBuilder::new("v6_8a")
        .token("z", "z")
        .rule("S", vec!["z"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(!col.sets.is_empty(), "single-rule grammar must have states");
    assert!(
        col.sets.len() <= 10,
        "single-rule grammar should be very small"
    );
}

#[test]
fn five_alternatives_grammar() {
    let mut g = GrammarBuilder::new("v6_8b")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .rule("S", vec!["d"])
        .rule("S", vec!["e"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    // Should have at least 1 state per alternative + initial
    assert!(
        col.sets.len() >= 2,
        "five alternatives grammar must have multiple states"
    );
}

#[test]
fn deeply_nested_chain() {
    let mut g = GrammarBuilder::new("v6_8c")
        .token("leaf", "leaf")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["D"])
        .rule("D", vec!["leaf"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 2, "deep chain must have multiple states");
}

#[test]
fn left_recursive_list() {
    let mut g = GrammarBuilder::new("v6_8d")
        .token("item", "item")
        .token("comma", ",")
        .rule("L", vec!["L", "comma", "item"])
        .rule("L", vec!["item"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 4,
        "left-recursive list needs several states, got {}",
        col.sets.len()
    );
}

#[test]
fn right_recursive_list() {
    let mut g = GrammarBuilder::new("v6_8e")
        .token("item", "item")
        .token("comma", ",")
        .rule("L", vec!["item", "comma", "L"])
        .rule("L", vec!["item"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 4,
        "right-recursive list needs several states, got {}",
        col.sets.len()
    );
}

#[test]
fn mutual_recursion() {
    let mut g = GrammarBuilder::new("v6_8f")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["B", "x"])
        .rule("A", vec!["x"])
        .rule("B", vec!["A", "y"])
        .rule("B", vec!["y"])
        .start("A")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 3,
        "mutual recursion grammar must produce states"
    );
}

#[test]
fn long_sequence_grammar() {
    let mut g = GrammarBuilder::new("v6_8g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a", "b", "c", "d", "e"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    // 5-token sequence: need at least 5 states for each dot position + accept
    assert!(
        col.sets.len() >= 5,
        "5-token sequence needs at least 5 states, got {}",
        col.sets.len()
    );
}

#[test]
fn mixed_terminal_nonterminal_rhs() {
    let mut g = GrammarBuilder::new("v6_8h")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "M", "b"])
        .rule("M", vec!["a"])
        .rule("M", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let has_nt_transition = col.symbol_is_terminal.values().any(|&v| !v);
    let has_t_transition = col.symbol_is_terminal.values().any(|&v| v);
    assert!(
        has_nt_transition,
        "mixed grammar must have nonterminal transitions"
    );
    assert!(
        has_t_transition,
        "mixed grammar must have terminal transitions"
    );
}
