#![cfg(feature = "test-api")]

//! Comprehensive tests for `ItemSetCollection`, `ItemSet`, and `LRItem`.
//!
//! Covers: construction, closure, goto, canonical collection building,
//! edge cases (single rule, recursive, chain, diamond), collection size,
//! symbol classification, augmented collections, and structural invariants.

use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{ProductionId, Rule, Symbol};
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

fn terminal_transitions_from(col: &ItemSetCollection, state: StateId) -> usize {
    col.goto_table
        .iter()
        .filter(|((src, sym), _)| *src == state && col.symbol_is_terminal.get(sym) == Some(&true))
        .count()
}

fn nonterminal_transitions_from(col: &ItemSetCollection, state: StateId) -> usize {
    col.goto_table
        .iter()
        .filter(|((src, sym), _)| *src == state && col.symbol_is_terminal.get(sym) == Some(&false))
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

// ===========================================================================
// 1. LRItem basic construction and properties
// ===========================================================================

#[test]
fn lr_item_new_fields() {
    let item = LRItem::new(RuleId(3), 2, SymbolId(5));
    assert_eq!(item.rule_id, RuleId(3));
    assert_eq!(item.position, 2);
    assert_eq!(item.lookahead, SymbolId(5));
}

#[test]
fn lr_item_at_position_zero_is_not_reduce() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0));
    assert!(!item.is_reduce_item(&g));
}

#[test]
fn lr_item_at_end_is_reduce() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), 1, SymbolId(0));
    assert!(item.is_reduce_item(&g));
}

#[test]
fn lr_item_next_symbol_at_start() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0));
    assert!(item.next_symbol(&g).is_some());
}

#[test]
fn lr_item_next_symbol_at_end_is_none() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), 1, SymbolId(0));
    assert!(item.next_symbol(&g).is_none());
}

#[test]
fn lr_item_ordering_is_deterministic() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(0), 0, SymbolId(2));
    let c = LRItem::new(RuleId(1), 0, SymbolId(1));
    let mut set = BTreeSet::new();
    set.insert(a.clone());
    set.insert(b.clone());
    set.insert(c.clone());
    assert_eq!(set.len(), 3);
}

// ===========================================================================
// 2. ItemSet construction and add_item
// ===========================================================================

#[test]
fn item_set_new_is_empty() {
    let set = ItemSet::new(StateId(0));
    assert!(set.items.is_empty());
    assert_eq!(set.id, StateId(0));
}

#[test]
fn item_set_add_item_increases_count() {
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    assert_eq!(set.items.len(), 1);
}

#[test]
fn item_set_add_duplicate_item_no_increase() {
    let mut set = ItemSet::new(StateId(0));
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    set.add_item(item.clone());
    set.add_item(item);
    assert_eq!(set.items.len(), 1);
}

#[test]
fn item_set_add_different_items() {
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(0), 1, SymbolId(0)));
    assert_eq!(set.items.len(), 3);
}

// ===========================================================================
// 3. ItemSet closure
// ===========================================================================

#[test]
fn closure_on_terminal_only_rule_adds_nothing() {
    // S→a: closure of {S→·a} should not add new items beyond the kernel
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items.len(), before);
}

#[test]
fn closure_expands_nonterminal() {
    // start→mid, mid→a: closure of {start→·mid} should add mid→·a
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    assert!(set.items.len() >= 2, "closure should add mid→·a");
}

#[test]
fn closure_is_idempotent() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    let after_first = set.items.clone();
    set.closure(&g, &ff).unwrap();
    assert_eq!(
        set.items, after_first,
        "second closure should not change items"
    );
}

#[test]
fn closure_three_level_chain() {
    // start→mid, mid→bot, bot→c: closure of {start→·mid} adds mid→·bot and bot→·c
    let mut g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("bot", vec!["c"])
        .rule("mid", vec!["bot"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    // start→·mid, mid→·bot, bot→·c = 3 items minimum
    assert!(
        set.items.len() >= 3,
        "chain closure should have ≥3, got {}",
        set.items.len()
    );
}

// ===========================================================================
// 4. ItemSet goto
// ===========================================================================

#[test]
fn goto_on_terminal_produces_nonempty_set() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);

    let a_sym = g.find_symbol_by_name("a").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(a_sym), &g, &ff);
    assert!(
        !goto_set.items.is_empty(),
        "goto on 'a' should produce items"
    );
}

#[test]
fn goto_on_absent_symbol_produces_empty_set() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);

    let b_sym = g.find_symbol_by_name("b").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(b_sym), &g, &ff);
    assert!(goto_set.items.is_empty(), "goto on 'b' should be empty");
}

#[test]
fn goto_advances_dot_position() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);

    let a_sym = g.find_symbol_by_name("a").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(a_sym), &g, &ff);
    // After shifting 'a', items should have position ≥ 1
    assert!(
        goto_set.items.iter().any(|i| i.position >= 1),
        "goto should advance dot"
    );
}

#[test]
fn goto_on_nonterminal_works() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);

    let mid_nt = g.find_symbol_by_name("mid").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::NonTerminal(mid_nt), &g, &ff);
    assert!(
        !goto_set.items.is_empty(),
        "goto on nonterminal mid should produce items"
    );
}

// ===========================================================================
// 5. Single-rule grammars
// ===========================================================================

#[test]
fn single_terminal_min_states() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 2);
}

#[test]
fn single_rule_three_terminals() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    // Needs states: initial, after-a, after-b, after-c, plus goto-on-S
    assert!(
        col.sets.len() >= 4,
        "S→a b c needs ≥4 states, got {}",
        col.sets.len()
    );
}

#[test]
fn single_rule_goto_table_nonempty() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(!col.goto_table.is_empty());
}

// ===========================================================================
// 6. Recursive grammars
// ===========================================================================

#[test]
fn left_recursive_terminates() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("L", vec!["x"])
        .rule("L", vec!["L", "x"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() <= 20,
        "should not explode: {}",
        col.sets.len()
    );
}

#[test]
fn right_recursive_terminates() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("R", vec!["x"])
        .rule("R", vec!["x", "R"])
        .start("R")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() <= 20,
        "should not explode: {}",
        col.sets.len()
    );
}

#[test]
fn left_recursive_closure_has_nonterminal_item() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("L", vec!["x"])
        .rule("L", vec!["L", "x"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);

    // State 0 should have items with next_symbol = NonTerminal(L)
    let has_nt_item = col.sets[0]
        .items
        .iter()
        .any(|i| matches!(i.next_symbol(&g), Some(Symbol::NonTerminal(_))));
    assert!(has_nt_item, "left-recursive closure should include L→·L x");
}

#[test]
fn mutual_recursion_terminates() {
    // A→x B, B→y A | y
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x", "B"])
        .rule("B", vec!["y", "A"])
        .rule("B", vec!["y"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 4);
    assert!(col.sets.len() <= 40);
}

// ===========================================================================
// 7. Chain grammars (non-terminal chains)
// ===========================================================================

#[test]
fn chain_two_levels() {
    // S→A, A→a
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets[0].items.len() >= 2, "closure should pull in A→·a");
}

#[test]
fn chain_four_levels() {
    // S→A, A→B, B→C, C→d
    let mut g = GrammarBuilder::new("t")
        .token("d", "d")
        .rule("C", vec!["d"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets[0].items.len() >= 4);
}

#[test]
fn chain_produces_goto_for_each_nonterminal() {
    // S→A, A→B, B→c
    let mut g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("B", vec!["c"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 should have goto transitions for S, A, B (nonterminals), and c (terminal)
    let nt_count = nonterminal_transitions_from(&col, StateId(0));
    assert!(
        nt_count >= 2,
        "chain should have ≥2 NT gotos from state 0, got {}",
        nt_count
    );
}

// ===========================================================================
// 8. Diamond grammars
// ===========================================================================

#[test]
fn diamond_grammar_produces_valid_collection() {
    // S→A B, A→x, B→y — two paths diverge then converge
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .rule("S", vec!["A", "B"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 3);
    // All state IDs valid
    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for ((src, _), dst) in &col.goto_table {
        assert!(ids.contains(src));
        assert!(ids.contains(dst));
    }
}

#[test]
fn diamond_shared_prefix() {
    // S→X c | Y c, X→a b, Y→a b
    // Both X and Y have identical RHS so they should share states
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("X", vec!["a", "b"])
        .rule("Y", vec!["a", "b"])
        .rule("S", vec!["X", "c"])
        .rule("S", vec!["Y", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 3);
    // State uniqueness
    for i in 0..col.sets.len() {
        for j in (i + 1)..col.sets.len() {
            assert_ne!(col.sets[i].items, col.sets[j].items);
        }
    }
}

#[test]
fn diamond_two_paths_to_same_terminal() {
    // S→A | B, A→x, B→x  — classic diamond converging on 'x'
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("A", vec!["x"])
        .rule("B", vec!["x"])
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // After shifting x, we should have a state with reduce items for both A and B
    let has_both_reduces = col.sets.iter().any(|s| reduce_items_in(s, &g) >= 2);
    assert!(
        has_both_reduces,
        "diamond should produce state with ≥2 reduce items"
    );
}

// ===========================================================================
// 9. Collection building — state properties
// ===========================================================================

#[test]
fn all_states_have_items() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for s in &col.sets {
        assert!(!s.items.is_empty(), "state {} is empty", s.id.0);
    }
}

#[test]
fn state_ids_sequential_from_zero() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for (i, s) in col.sets.iter().enumerate() {
        assert_eq!(s.id, StateId(i as u16));
    }
}

#[test]
fn no_duplicate_states() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for i in 0..col.sets.len() {
        for j in (i + 1)..col.sets.len() {
            assert_ne!(col.sets[i].items, col.sets[j].items);
        }
    }
}

#[test]
fn goto_targets_are_valid_states() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("X", vec!["a"])
        .rule("S", vec!["X", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for (_, dst) in &col.goto_table {
        assert!(ids.contains(dst));
    }
}

#[test]
fn goto_sources_are_valid_states() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("X", vec!["a"])
        .rule("S", vec!["X", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for ((src, _), _) in &col.goto_table {
        assert!(ids.contains(src));
    }
}

// ===========================================================================
// 10. Symbol classification
// ===========================================================================

#[test]
fn terminal_classified_correctly() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let a_sym = g.find_symbol_by_name("a").unwrap();
    if col.symbol_is_terminal.contains_key(&a_sym) {
        assert_eq!(col.symbol_is_terminal[&a_sym], true);
    }
}

#[test]
fn nonterminal_classified_correctly() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let (col, _) = build(&mut g);

    let mid_nt = g.find_symbol_by_name("mid").unwrap();
    if col.symbol_is_terminal.contains_key(&mid_nt) {
        assert_eq!(col.symbol_is_terminal[&mid_nt], false);
    }
}

#[test]
fn all_goto_symbols_classified() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("X", vec!["a"])
        .rule("S", vec!["X", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for ((_, sym), _) in &col.goto_table {
        assert!(
            col.symbol_is_terminal.contains_key(sym),
            "symbol {:?} not classified",
            sym
        );
    }
}

// ===========================================================================
// 11. Reduce and shift item distribution
// ===========================================================================

#[test]
fn initial_state_has_shift_items() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(shift_items_in(&col.sets[0], &g) >= 1);
}

#[test]
fn some_state_has_only_reduce_items() {
    // S→a: after shifting 'a', the state should have only reduce items for S
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_pure_reduce = col.sets.iter().any(|s| {
        let reduces = reduce_items_in(s, &g);
        let shifts = shift_items_in(s, &g);
        reduces > 0 && shifts == 0
    });
    assert!(has_pure_reduce, "should have a pure reduce state");
}

#[test]
fn shift_reduce_state_exists_for_ambiguous_grammar() {
    // E→E + E | n
    let mut g = GrammarBuilder::new("t")
        .token("n", "n")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);

    let has_sr = col
        .sets
        .iter()
        .any(|s| reduce_items_in(s, &g) > 0 && shift_items_in(s, &g) > 0);
    assert!(
        has_sr,
        "ambiguous grammar should produce shift-reduce state"
    );
}

// ===========================================================================
// 12. Augmented collection
// ===========================================================================

#[test]
fn augmented_collection_nonempty() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let max_id = g
        .rules
        .keys()
        .chain(g.tokens.keys())
        .map(|id| id.0)
        .max()
        .unwrap_or(0);
    let aug_start = SymbolId(max_id + 1);
    let eof = SymbolId(max_id + 2);
    let start = g.start_symbol().unwrap();

    g.add_rule(Rule {
        lhs: aug_start,
        rhs: vec![Symbol::NonTerminal(start)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(max_id + 1),
    });
    g.rule_names.insert(aug_start, "S'".to_string());

    let ff2 = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col =
        ItemSetCollection::build_canonical_collection_augmented(&g, &ff2, aug_start, start, eof);
    assert!(!col.sets.is_empty());
}

#[test]
fn augmented_collection_state_zero_has_eof_lookahead() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let max_id = g
        .rules
        .keys()
        .chain(g.tokens.keys())
        .map(|id| id.0)
        .max()
        .unwrap_or(0);
    let aug_start = SymbolId(max_id + 1);
    let eof = SymbolId(max_id + 2);
    let start = g.start_symbol().unwrap();

    g.add_rule(Rule {
        lhs: aug_start,
        rhs: vec![Symbol::NonTerminal(start)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(max_id + 1),
    });
    g.rule_names.insert(aug_start, "S'".to_string());

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col =
        ItemSetCollection::build_canonical_collection_augmented(&g, &ff, aug_start, start, eof);

    let has_eof = col.sets[0].items.iter().any(|i| i.lookahead == eof);
    assert!(has_eof, "augmented state 0 should have EOF lookahead");
}

// ===========================================================================
// 13. Transition counting
// ===========================================================================

#[test]
fn single_terminal_state_zero_has_terminal_transition() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(terminal_transitions_from(&col, StateId(0)) >= 1);
}

#[test]
fn nonterminal_rule_state_zero_has_nt_transition() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(nonterminal_transitions_from(&col, StateId(0)) >= 1);
}

#[test]
fn flat_grammar_eight_alternatives_eight_transitions() {
    let names = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let mut builder = GrammarBuilder::new("t");
    for n in &names {
        builder = builder.token(n, n);
    }
    for n in &names {
        builder = builder.rule("S", vec![n]);
    }
    let mut g = builder.start("S").build();
    let (col, _) = build(&mut g);
    assert!(
        terminal_transitions_from(&col, StateId(0)) >= 8,
        "state 0 should have 8 terminal transitions, got {}",
        terminal_transitions_from(&col, StateId(0))
    );
}

// ===========================================================================
// 14. Lookahead propagation
// ===========================================================================

#[test]
fn lookahead_from_follow_context() {
    // S→A b, A→c: items for A→·c should carry lookahead 'b'
    let mut g = GrammarBuilder::new("t")
        .token("b", "b")
        .token("c", "c")
        .rule("A", vec!["c"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let b_sym = g.find_symbol_by_name("b").unwrap();
    let a_rule = g
        .all_rules()
        .find(|r| g.rule_names.get(&r.lhs).is_some_and(|n| n == "A"));
    if let Some(ar) = a_rule {
        let has_la = col.sets[0]
            .items
            .iter()
            .any(|i| i.rule_id.0 == ar.production_id.0 && i.position == 0 && i.lookahead == b_sym);
        assert!(has_la, "A→·c should have lookahead b");
    }
}

#[test]
fn lookahead_eof_for_start_symbol() {
    // In non-augmented build, start items get lookahead SymbolId(0) (EOF)
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_eof_la = col.sets[0].items.iter().any(|i| i.lookahead == SymbolId(0));
    assert!(has_eof_la, "start items should have EOF lookahead");
}

// ===========================================================================
// 15. Complex grammars
// ===========================================================================

#[test]
fn arithmetic_with_parens_bounded_states() {
    let mut g = GrammarBuilder::new("t")
        .token("NUM", "num")
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

    assert!(col.sets.len() >= 8);
    assert!(col.sets.len() <= 50);
}

#[test]
fn statement_grammar_terminates() {
    let mut g = GrammarBuilder::new("t")
        .token("ID", "id")
        .token("NUM", "num")
        .token("=", "=")
        .token(";", ";")
        .rule("program", vec!["stmts"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", ";", "stmt"])
        .rule("stmt", vec!["ID", "=", "expr"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["NUM"])
        .start("program")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 5);
    assert!(col.sets.len() <= 60);
}

#[test]
fn multiple_alternatives_many_rules() {
    // S→A | B | C, A→a, B→b, C→c d
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .rule("C", vec!["c", "d"])
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .rule("S", vec!["C"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 should have transitions for a, b, c (terminals) and A, B, C (nonterminals)
    assert!(transitions_from(&col, StateId(0)) >= 3);
}

// ===========================================================================
// 16. Edge cases
// ===========================================================================

#[test]
fn single_nonterminal_chain_to_terminal() {
    // S→A, A→a — simplest nonterminal chain
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Should produce at least 2 states
    assert!(col.sets.len() >= 2);
    assert!(!col.goto_table.is_empty());
}

#[test]
fn two_rules_same_rhs_different_lhs() {
    // A→x, B→x, S→A B (nonsensical but valid)
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .rule("S", vec!["A", "B"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(col.sets.len() >= 3);
}

#[test]
fn long_rule_five_terminals() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a", "b", "c", "d", "e"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // 5 terminals + initial + goto-on-S = at least 6 states
    assert!(
        col.sets.len() >= 6,
        "5-terminal rule needs ≥6 states, got {}",
        col.sets.len()
    );
}

#[test]
fn collection_goto_table_consistent_with_sets() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("X", vec!["b"])
        .rule("S", vec!["a", "X", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Every goto target must be a state in the collection
    let max_state = col.sets.len() as u16;
    for (_, dst) in &col.goto_table {
        assert!(
            dst.0 < max_state,
            "goto target {} exceeds collection size {}",
            dst.0,
            max_state
        );
    }
}

// ===========================================================================
// 17. Structural invariants
// ===========================================================================

#[test]
fn no_terminal_self_loops() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for ((src, sym), dst) in &col.goto_table {
        if col.symbol_is_terminal.get(sym) == Some(&true) {
            assert_ne!(src, dst, "self-loop on terminal in state {}", src.0);
        }
    }
}

#[test]
fn collection_state_zero_always_exists() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(!col.sets.is_empty());
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn every_nonterminal_goto_leads_to_state_with_advanced_position() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for ((_, sym), dst) in &col.goto_table {
        if col.symbol_is_terminal.get(sym) == Some(&false) {
            let dst_state = col.sets.iter().find(|s| s.id == *dst).unwrap();
            // At least one item in the destination should have position > 0
            let has_advanced = dst_state.items.iter().any(|i| i.position > 0);
            assert!(
                has_advanced,
                "NT goto target state {} should have advanced items",
                dst.0
            );
        }
    }
}

// ===========================================================================
// 18. Integration with FirstFollowSets
// ===========================================================================

#[test]
fn first_follow_computation_succeeds_for_simple_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn first_follow_computation_succeeds_for_recursive_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("L", vec!["x"])
        .rule("L", vec!["L", "x"])
        .start("L")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn first_set_contains_expected_terminal() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let s_sym = g.find_symbol_by_name("start").unwrap();
    let a_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "a")
        .copied()
        .unwrap();
    if let Some(first_s) = ff.first(s_sym) {
        assert!(
            first_s.contains(a_sym.0 as usize),
            "FIRST(start) should contain 'a'"
        );
    }
}

// ===========================================================================
// 19. Integration with parse table builder
// ===========================================================================

#[test]
fn collection_feeds_into_parse_table() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let _col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let table = build_lr1_automaton(&g, &ff).expect("parse table should build");
    assert!(table.state_count > 0);
}

#[test]
fn sanity_check_passes_on_simple_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("NUM", "num")
        .token("+", "+")
        .rule("E", vec!["E", "+", "NUM"])
        .rule("E", vec!["NUM"])
        .start("E")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let table = build_lr1_automaton(&g, &ff).expect("parse table should build");
    sanity_check_tables(&table).expect("sanity check should pass");
}

// ===========================================================================
// 20. Misc edge cases
// ===========================================================================

#[test]
fn grammar_with_many_nonterminal_alternatives() {
    // S→A | B | C | D, A→a, B→b, C→c, D→d
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .rule("C", vec!["c"])
        .rule("D", vec!["d"])
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .rule("S", vec!["C"])
        .rule("S", vec!["D"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 closure should have items for S, A, B, C, D
    assert!(col.sets[0].items.len() >= 5);
    assert!(transitions_from(&col, StateId(0)) >= 4);
}

#[test]
fn goto_table_entries_have_unique_keys() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // IndexMap guarantees unique keys, but verify programmatically
    let keys: Vec<_> = col.goto_table.keys().collect();
    let key_set: BTreeSet<_> = col.goto_table.keys().collect();
    assert_eq!(keys.len(), key_set.len());
}

#[test]
fn collection_for_two_terminal_seq_has_chain_structure() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Should form a chain: state 0 →[a]→ state X →[b]→ state Y
    let a_sym = g.find_symbol_by_name("a").unwrap();
    let b_sym = g.find_symbol_by_name("b").unwrap();

    let after_a = col.goto_table.get(&(StateId(0), a_sym));
    assert!(after_a.is_some(), "should have goto on 'a' from state 0");

    if let Some(&state_after_a) = after_a {
        let after_b = col.goto_table.get(&(state_after_a, b_sym));
        assert!(
            after_b.is_some(),
            "should have goto on 'b' from state after 'a'"
        );
    }
}

#[test]
fn multiple_rules_same_lhs_expand_closure() {
    // S→a | b | c
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 should have items for all three alternatives
    let pos0_count = col.sets[0].items.iter().filter(|i| i.position == 0).count();
    assert!(
        pos0_count >= 3,
        "state 0 should have ≥3 initial items, got {}",
        pos0_count
    );
}

#[test]
fn collection_clone_is_equal() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let cloned = col.clone();

    assert_eq!(col.sets.len(), cloned.sets.len());
    assert_eq!(col.goto_table.len(), cloned.goto_table.len());
    for (i, s) in col.sets.iter().enumerate() {
        assert_eq!(s.items, cloned.sets[i].items);
        assert_eq!(s.id, cloned.sets[i].id);
    }
}
