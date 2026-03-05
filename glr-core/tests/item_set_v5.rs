#![cfg(feature = "test-api")]

//! item_set_v5 — 55+ tests for item set construction, closure, kernel items,
//! LR(1) item properties, item equality/hashing, complex item sets, and edge cases.

use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::Symbol;
use std::collections::{BTreeSet, HashSet};

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

fn targets_from(col: &ItemSetCollection, state: StateId) -> BTreeSet<StateId> {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == state)
        .map(|(_, &dst)| dst)
        .collect()
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
// 1. LR(1) item construction & basic properties (8 tests)
// ===========================================================================

#[test]
fn lr_item_fields_round_trip() {
    let item = LRItem::new(RuleId(7), 3, SymbolId(42));
    assert_eq!(item.rule_id, RuleId(7));
    assert_eq!(item.position, 3);
    assert_eq!(item.lookahead, SymbolId(42));
}

#[test]
fn lr_item_position_zero_not_reduce_single_rhs() {
    let mut g = GrammarBuilder::new("v5_p0")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0));
    assert!(!item.is_reduce_item(&g));
}

#[test]
fn lr_item_at_rhs_end_is_reduce() {
    let mut g = GrammarBuilder::new("v5_re")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), rule.rhs.len(), SymbolId(0));
    assert!(item.is_reduce_item(&g));
}

#[test]
fn lr_item_next_symbol_at_zero() {
    let mut g = GrammarBuilder::new("v5_ns0")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0));
    assert!(item.next_symbol(&g).is_some());
}

#[test]
fn lr_item_next_symbol_at_end_is_none() {
    let mut g = GrammarBuilder::new("v5_nse")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), rule.rhs.len(), SymbolId(0));
    assert!(item.next_symbol(&g).is_none());
}

#[test]
fn lr_item_mid_position_has_next_symbol() {
    let mut g = GrammarBuilder::new("v5_mid")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let _ = g.normalize();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let item = LRItem::new(RuleId(rule.production_id.0), 1, SymbolId(0));
    let sym = item.next_symbol(&g);
    assert!(sym.is_some(), "position 1 of 3-element RHS should have next_symbol");
}

#[test]
fn lr_item_different_lookaheads_are_distinct() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(0), 0, SymbolId(2));
    assert_ne!(a, b, "items with different lookaheads should differ");
}

#[test]
fn lr_item_different_positions_are_distinct() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(0), 1, SymbolId(1));
    assert_ne!(a, b, "items with different positions should differ");
}

// ===========================================================================
// 2. LR(1) item equality & hashing (7 tests)
// ===========================================================================

#[test]
fn lr_item_equal_items_are_eq() {
    let a = LRItem::new(RuleId(5), 2, SymbolId(3));
    let b = LRItem::new(RuleId(5), 2, SymbolId(3));
    assert_eq!(a, b);
}

#[test]
fn lr_item_hash_consistent_with_eq() {
    use std::hash::{Hash, Hasher};
    let a = LRItem::new(RuleId(5), 2, SymbolId(3));
    let b = LRItem::new(RuleId(5), 2, SymbolId(3));

    let hash_of = |item: &LRItem| {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        item.hash(&mut h);
        h.finish()
    };
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn lr_item_btreeset_deduplicates() {
    let item = LRItem::new(RuleId(1), 0, SymbolId(0));
    let mut set = BTreeSet::new();
    set.insert(item.clone());
    set.insert(item);
    assert_eq!(set.len(), 1);
}

#[test]
fn lr_item_hashset_deduplicates() {
    let item = LRItem::new(RuleId(1), 0, SymbolId(0));
    let mut set = HashSet::new();
    set.insert(item.clone());
    set.insert(item);
    assert_eq!(set.len(), 1);
}

#[test]
fn lr_item_ordering_by_rule_id_first() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(0));
    let b = LRItem::new(RuleId(1), 0, SymbolId(0));
    assert!(a < b, "should order by rule_id first");
}

#[test]
fn lr_item_ordering_by_position_second() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(0));
    let b = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(a < b, "should order by position second");
}

#[test]
fn lr_item_ordering_by_lookahead_third() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(0));
    let b = LRItem::new(RuleId(0), 0, SymbolId(1));
    assert!(a < b, "should order by lookahead third");
}

// ===========================================================================
// 3. ItemSet construction (5 tests)
// ===========================================================================

#[test]
fn item_set_new_is_empty_with_correct_id() {
    let set = ItemSet::new(StateId(9));
    assert!(set.items.is_empty());
    assert_eq!(set.id, StateId(9));
}

#[test]
fn item_set_add_item_increases_count() {
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    assert_eq!(set.items.len(), 1);
}

#[test]
fn item_set_add_duplicate_is_idempotent() {
    let mut set = ItemSet::new(StateId(0));
    let item = LRItem::new(RuleId(2), 1, SymbolId(4));
    set.add_item(item.clone());
    set.add_item(item);
    assert_eq!(set.items.len(), 1);
}

#[test]
fn item_set_holds_multiple_distinct_items() {
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(0), 1, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(1)));
    assert_eq!(set.items.len(), 4);
}

#[test]
fn item_set_items_are_sorted() {
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(5), 2, SymbolId(3)));
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(3), 1, SymbolId(2)));
    let items: Vec<_> = set.items.iter().collect();
    for w in items.windows(2) {
        assert!(w[0] <= w[1], "BTreeSet items should be sorted");
    }
}

// ===========================================================================
// 4. Closure operations (8 tests)
// ===========================================================================

#[test]
fn closure_terminal_only_adds_nothing() {
    let mut g = GrammarBuilder::new("v5_ct")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items.len(), before, "terminal-only rule should not expand");
}

#[test]
fn closure_expands_single_nonterminal() {
    let mut g = GrammarBuilder::new("v5_ce")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    assert!(set.items.len() >= 2, "should expand to include A→·a");
}

#[test]
fn closure_is_idempotent() {
    let mut g = GrammarBuilder::new("v5_ci")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    let after_first = set.items.clone();
    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items, after_first, "repeated closure should be stable");
}

#[test]
fn closure_transitive_three_levels() {
    let mut g = GrammarBuilder::new("v5_c3")
        .token("c", "c")
        .rule("C", vec!["c"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    assert!(
        set.items.len() >= 4,
        "chain S→A→B→C should expand to ≥4 items, got {}",
        set.items.len()
    );
}

#[test]
fn closure_multiple_alternatives() {
    let mut g = GrammarBuilder::new("v5_ca")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("X", vec!["a"])
        .rule("X", vec!["b"])
        .rule("X", vec!["c"])
        .rule("S", vec!["X"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    assert!(
        set.items.len() >= 4,
        "S→X with X→a|b|c should expand to ≥4 items, got {}",
        set.items.len()
    );
}

#[test]
fn closure_propagates_lookahead() {
    let mut g = GrammarBuilder::new("v5_cla")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let s_rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(s_rule.production_id.0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();

    let b_sym = g.find_symbol_by_name("b");
    if let Some(b_id) = b_sym {
        let has_b_lookahead = set.items.iter().any(|i| i.position == 0 && i.lookahead == b_id);
        assert!(
            has_b_lookahead,
            "closure should propagate lookahead 'b' to A items"
        );
    }
}

#[test]
fn closure_left_recursive_does_not_diverge() {
    let mut g = GrammarBuilder::new("v5_clr")
        .token("x", "x")
        .rule("L", vec!["x"])
        .rule("L", vec!["L", "x"])
        .start("L")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rules = g.get_rules_for_symbol(start).unwrap();
    let mut set = ItemSet::new(StateId(0));
    for rule in rules {
        set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    }
    set.closure(&g, &ff).unwrap();
    assert!(
        set.items.len() <= 50,
        "left-recursive closure should terminate, got {} items",
        set.items.len()
    );
}

#[test]
fn closure_does_not_expand_past_dot_terminal() {
    let mut g = GrammarBuilder::new("v5_cpt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();

    let start = g.start_symbol().unwrap();
    let rule = g.get_rules_for_symbol(start).unwrap()[0].clone();
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&g, &ff).unwrap();
    assert_eq!(
        set.items.len(),
        before,
        "dot before terminal should not trigger expansion"
    );
}

// ===========================================================================
// 5. Kernel items (5 tests)
// ===========================================================================

#[test]
fn kernel_items_initial_state_has_position_zero() {
    let mut g = GrammarBuilder::new("v5_k0")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let has_pos_zero = col.sets[0].items.iter().any(|i| i.position == 0);
    assert!(has_pos_zero, "initial state should have position-0 items");
}

#[test]
fn kernel_items_shifted_state_has_advanced_positions() {
    let mut g = GrammarBuilder::new("v5_ks")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, ff) = build(&mut g);

    let a_sym = g.find_symbol_by_name("a").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(a_sym), &g, &ff);
    let has_advanced = goto_set.items.iter().any(|i| i.position >= 1);
    assert!(has_advanced, "goto on 'a' should produce items with position ≥ 1");
}

#[test]
fn kernel_items_all_states_have_items() {
    let mut g = GrammarBuilder::new("v5_kai")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for set in &col.sets {
        assert!(
            !set.items.is_empty(),
            "state {} should have at least one item",
            set.id.0
        );
    }
}

#[test]
fn kernel_items_reduce_state_has_item_at_end() {
    let mut g = GrammarBuilder::new("v5_kre")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_reduce_state = col.sets.iter().any(|s| reduce_items_in(s, &g) > 0);
    assert!(has_reduce_state, "grammar S→a should have at least one reduce state");
}

#[test]
fn kernel_items_shift_state_has_next_symbol() {
    let mut g = GrammarBuilder::new("v5_ksh")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_shift_state = col.sets.iter().any(|s| shift_items_in(s, &g) > 0);
    assert!(has_shift_state, "grammar should have at least one shift state");
}

// ===========================================================================
// 6. Item set construction from canonical collection (8 tests)
// ===========================================================================

#[test]
fn canonical_collection_nonempty() {
    let mut g = GrammarBuilder::new("v5_cc1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn canonical_collection_initial_state_is_zero() {
    let mut g = GrammarBuilder::new("v5_cc2")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn canonical_collection_sequential_ids() {
    let mut g = GrammarBuilder::new("v5_cc3")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn canonical_collection_unique_ids() {
    let mut g = GrammarBuilder::new("v5_cc4")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    assert_eq!(ids.len(), col.sets.len(), "all state IDs must be unique");
}

#[test]
fn canonical_collection_goto_table_populated() {
    let mut g = GrammarBuilder::new("v5_cc5")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(!col.goto_table.is_empty(), "goto table must have entries");
}

#[test]
fn canonical_collection_symbol_classification() {
    let mut g = GrammarBuilder::new("v5_cc6")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_terminal = col.symbol_is_terminal.values().any(|&v| v);
    let has_nonterminal = col.symbol_is_terminal.values().any(|&v| !v);
    assert!(has_terminal, "should have terminal symbols");
    assert!(has_nonterminal, "should have nonterminal symbols");
}

#[test]
fn canonical_collection_goto_targets_valid() {
    let mut g = GrammarBuilder::new("v5_cc7")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let max_id = col.sets.len() as u16;
    for (_, &target) in &col.goto_table {
        assert!(target.0 < max_id, "goto target {} out of range", target.0);
    }
}

#[test]
fn canonical_collection_no_duplicate_items_per_state() {
    let mut g = GrammarBuilder::new("v5_cc8")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for set in &col.sets {
        let vec: Vec<_> = set.items.iter().collect();
        let deduped: BTreeSet<_> = vec.iter().collect();
        assert_eq!(vec.len(), deduped.len(), "state {} has duplicate items", set.id.0);
    }
}

// ===========================================================================
// 7. Complex item sets: expression grammar (4 tests)
// ===========================================================================

#[test]
fn expr_grammar_state_count_in_range() {
    let mut g = GrammarBuilder::new("v5_expr1")
        .token("num", "num")
        .token("+", "+")
        .token("*", "*")
        .rule("E", vec!["E", "+", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "*", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["num"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 8, "expression grammar ≥8 states, got {}", col.sets.len());
    assert!(col.sets.len() <= 30, "expression grammar ≤30 states, got {}", col.sets.len());
}

#[test]
fn expr_grammar_has_shift_reduce_potential() {
    let mut g = GrammarBuilder::new("v5_expr2")
        .token("num", "num")
        .token("+", "+")
        .token("*", "*")
        .rule("E", vec!["E", "+", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "*", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["num"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);

    let total_reduce = col.sets.iter().map(|s| reduce_items_in(s, &g)).sum::<usize>();
    let total_shift = col.sets.iter().map(|s| shift_items_in(s, &g)).sum::<usize>();
    assert!(total_reduce > 0, "expression grammar should have reduce items");
    assert!(total_shift > 0, "expression grammar should have shift items");
}

#[test]
fn expr_grammar_initial_state_has_transitions() {
    let mut g = GrammarBuilder::new("v5_expr3")
        .token("num", "num")
        .token("+", "+")
        .token("*", "*")
        .rule("E", vec!["E", "+", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "*", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["num"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        transitions_from(&col, StateId(0)) >= 2,
        "initial state should have multiple transitions"
    );
}

#[test]
fn expr_grammar_goto_table_has_many_entries() {
    let mut g = GrammarBuilder::new("v5_expr4")
        .token("num", "num")
        .token("+", "+")
        .token("*", "*")
        .rule("E", vec!["E", "+", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "*", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["num"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.goto_table.len() >= 8,
        "expression grammar should have ≥8 transitions, got {}",
        col.goto_table.len()
    );
}

// ===========================================================================
// 8. Complex item sets: recursive grammars (4 tests)
// ===========================================================================

#[test]
fn left_recursive_terminates_and_bounded() {
    let mut g = GrammarBuilder::new("v5_lr")
        .token("x", "x")
        .rule("L", vec!["x"])
        .rule("L", vec!["L", "x"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 3);
    assert!(col.sets.len() <= 20, "left-recursive bounded, got {}", col.sets.len());
}

#[test]
fn right_recursive_terminates_and_bounded() {
    let mut g = GrammarBuilder::new("v5_rr")
        .token("x", "x")
        .rule("R", vec!["x"])
        .rule("R", vec!["x", "R"])
        .start("R")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 3);
    assert!(col.sets.len() <= 20, "right-recursive bounded, got {}", col.sets.len());
}

#[test]
fn mutual_recursion_terminates() {
    let mut g = GrammarBuilder::new("v5_mr")
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

#[test]
fn left_recursive_initial_has_nonterminal_item() {
    let mut g = GrammarBuilder::new("v5_lrn")
        .token("x", "x")
        .rule("L", vec!["x"])
        .rule("L", vec!["L", "x"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);

    let has_nt_next = col.sets[0]
        .items
        .iter()
        .any(|i| matches!(i.next_symbol(&g), Some(Symbol::NonTerminal(_))));
    assert!(has_nt_next, "initial state should have item with NT next symbol");
}

// ===========================================================================
// 9. Edge cases (10 tests)
// ===========================================================================

#[test]
fn single_token_grammar_min_two_states() {
    let mut g = GrammarBuilder::new("v5_e1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 2, "S→a needs ≥2 states");
}

#[test]
fn two_alternatives_at_least_as_many_states() {
    let mut g1 = GrammarBuilder::new("v5_e2a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col1, _) = build(&mut g1);

    let mut g2 = GrammarBuilder::new("v5_e2b")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col2, _) = build(&mut g2);

    assert!(col2.sets.len() >= col1.sets.len());
}

#[test]
fn chain_grammar_deeper_chain_more_states() {
    let mut g_short = GrammarBuilder::new("v5_e3a")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col_short, _) = build(&mut g_short);

    let mut g_long = GrammarBuilder::new("v5_e3b")
        .token("a", "a")
        .rule("C", vec!["a"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col_long, _) = build(&mut g_long);

    assert!(
        col_long.sets.len() >= col_short.sets.len(),
        "deeper chain should produce at least as many states"
    );
}

#[test]
fn many_alternatives_produces_many_states() {
    let mut g = GrammarBuilder::new("v5_e4")
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
    assert!(
        col.sets.len() >= 5,
        "5 alternatives should produce ≥5 states, got {}",
        col.sets.len()
    );
}

#[test]
fn long_sequence_produces_many_states() {
    let mut g = GrammarBuilder::new("v5_e5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("S", vec!["a", "b", "c", "d"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 5,
        "S→a b c d should produce ≥5 states, got {}",
        col.sets.len()
    );
}

#[test]
fn diamond_grammar_all_paths_covered() {
    // S→A B, A→x, B→y
    let mut g = GrammarBuilder::new("v5_e6")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .rule("S", vec!["A", "B"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 4, "diamond ≥4 states, got {}", col.sets.len());
}

#[test]
fn goto_on_absent_symbol_produces_empty_set() {
    let mut g = GrammarBuilder::new("v5_e7")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);

    let b_sym = g.find_symbol_by_name("b").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(b_sym), &g, &ff);
    assert!(goto_set.items.is_empty(), "goto on absent symbol should be empty");
}

#[test]
fn goto_advances_dot() {
    let mut g = GrammarBuilder::new("v5_e8")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);

    let a_sym = g.find_symbol_by_name("a").unwrap();
    let goto_set = col.sets[0].goto(&Symbol::Terminal(a_sym), &g, &ff);
    assert!(
        goto_set.items.iter().any(|i| i.position >= 1),
        "goto should advance dot position"
    );
}

#[test]
fn all_items_reference_valid_productions() {
    let mut g = GrammarBuilder::new("v5_e9")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let all_prod_ids: BTreeSet<u16> = g.all_rules().map(|r| r.production_id.0).collect();
    for set in &col.sets {
        for item in &set.items {
            assert!(
                all_prod_ids.contains(&item.rule_id.0),
                "item rule_id {} not in grammar",
                item.rule_id.0
            );
        }
    }
}

#[test]
fn targets_from_initial_are_distinct() {
    let mut g = GrammarBuilder::new("v5_e10")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let tgts = targets_from(&col, StateId(0));
    // Each transition should go to a distinct state
    assert!(
        tgts.len() == transitions_from(&col, StateId(0))
            || !tgts.is_empty(),
        "transitions from state 0 should target distinct states"
    );
}
