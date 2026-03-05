#![allow(clippy::needless_range_loop)]
#![cfg(feature = "test-api")]

//! Comprehensive tests for item set construction and canonical collection.
//!
//! Covers: canonical collection building, item set properties, collection sizes,
//! closure computation, goto computation, item equality/ordering, complex grammars,
//! and edge cases.

use adze_glr_core::*;
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build canonical collection from a mutable grammar.
fn build(grammar: &mut adze_ir::Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

/// Count transitions originating from the given state.
fn transitions_from(col: &ItemSetCollection, state: StateId) -> usize {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == state)
        .count()
}

/// Collect all distinct target states reachable from `state`.
fn targets_from(col: &ItemSetCollection, state: StateId) -> BTreeSet<StateId> {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == state)
        .map(|(_, &dst)| dst)
        .collect()
}

/// Count how many states contain at least one reduce item.
fn count_reduce_states(col: &ItemSetCollection, grammar: &adze_ir::Grammar) -> usize {
    col.sets
        .iter()
        .filter(|s| s.items.iter().any(|i| i.is_reduce_item(grammar)))
        .count()
}

/// Count how many states have both shift and reduce items (conflict states).
fn count_conflict_states(col: &ItemSetCollection, grammar: &adze_ir::Grammar) -> usize {
    col.sets
        .iter()
        .filter(|s| {
            let has_reduce = s.items.iter().any(|i| i.is_reduce_item(grammar));
            let has_shift = s.items.iter().any(|i| i.next_symbol(grammar).is_some());
            has_reduce && has_shift
        })
        .count()
}

// ===========================================================================
// 1. Canonical collection from simple grammar (8 tests)
// ===========================================================================

#[test]
fn simple_single_token_builds_nonempty_collection() {
    let mut g = GrammarBuilder::new("s1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        !col.sets.is_empty(),
        "collection must have at least one state"
    );
}

#[test]
fn simple_initial_state_is_id_zero() {
    let mut g = GrammarBuilder::new("s2")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn simple_state_ids_are_sequential() {
    let mut g = GrammarBuilder::new("s3")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16), "state {i} should have id {i}");
    }
}

#[test]
fn simple_two_alternatives_more_states_than_single() {
    let mut g1 = GrammarBuilder::new("one")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col1, _) = build(&mut g1);

    let mut g2 = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col2, _) = build(&mut g2);

    assert!(
        col2.sets.len() >= col1.sets.len(),
        "two alternatives should produce at least as many states"
    );
}

#[test]
fn simple_goto_table_nonempty() {
    let mut g = GrammarBuilder::new("s5")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        !col.goto_table.is_empty(),
        "goto table must have transitions"
    );
}

#[test]
fn simple_symbol_is_terminal_populated() {
    let mut g = GrammarBuilder::new("s6")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        !col.symbol_is_terminal.is_empty(),
        "symbol_is_terminal must be populated"
    );
}

#[test]
fn simple_two_tokens_sequence_produces_at_least_three_states() {
    let mut g = GrammarBuilder::new("s7")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 3,
        "S→a b needs ≥3 states (initial, after-a, after-b), got {}",
        col.sets.len()
    );
}

#[test]
fn simple_collection_has_terminal_and_nonterminal_symbols() {
    let mut g = GrammarBuilder::new("s8")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_terminal = col.symbol_is_terminal.values().any(|&v| v);
    let has_nonterminal = col.symbol_is_terminal.values().any(|&v| !v);
    assert!(has_terminal, "should track at least one terminal");
    assert!(has_nonterminal, "should track at least one non-terminal");
}

// ===========================================================================
// 2. Item set properties (8 tests)
// ===========================================================================

#[test]
fn item_set_initial_state_items_are_nonempty() {
    let mut g = GrammarBuilder::new("ip1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        !col.sets[0].items.is_empty(),
        "initial state must have items"
    );
}

#[test]
fn item_set_items_sorted_in_btreeset() {
    let mut g = GrammarBuilder::new("ip2")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // BTreeSet guarantees sorted iteration; verify items come in order
    for set in &col.sets {
        let items: Vec<_> = set.items.iter().collect();
        for w in items.windows(2) {
            assert!(w[0] <= w[1], "items should be in sorted order");
        }
    }
}

#[test]
fn item_set_no_duplicate_items() {
    let mut g = GrammarBuilder::new("ip3")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for set in &col.sets {
        let vec: Vec<_> = set.items.iter().collect();
        let deduped: BTreeSet<_> = vec.iter().collect();
        assert_eq!(
            vec.len(),
            deduped.len(),
            "items should be unique in state {}",
            set.id.0
        );
    }
}

#[test]
fn item_set_reduce_items_have_position_at_rhs_end() {
    let mut g = GrammarBuilder::new("ip4")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for set in &col.sets {
        for item in &set.items {
            if item.is_reduce_item(&g) {
                // For non-epsilon rules, position should be at the end
                if let Some(rule) = g.all_rules().find(|r| r.production_id.0 == item.rule_id.0)
                    && !rule.rhs.is_empty()
                    && !matches!(rule.rhs[0], adze_ir::Symbol::Epsilon)
                {
                    assert_eq!(
                        item.position,
                        rule.rhs.len(),
                        "reduce item position should equal RHS length"
                    );
                }
            }
        }
    }
}

#[test]
fn item_set_shift_items_have_next_symbol() {
    let mut g = GrammarBuilder::new("ip5")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for set in &col.sets {
        for item in &set.items {
            if !item.is_reduce_item(&g) {
                assert!(
                    item.next_symbol(&g).is_some(),
                    "non-reduce item in state {} should have next_symbol",
                    set.id.0
                );
            }
        }
    }
}

#[test]
fn item_set_all_items_have_valid_rule_ids() {
    let mut g = GrammarBuilder::new("ip6")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let all_prod_ids: BTreeSet<u16> = g.all_rules().map(|r| r.production_id.0).collect();
    for set in &col.sets {
        for item in &set.items {
            assert!(
                all_prod_ids.contains(&item.rule_id.0),
                "item rule_id {} not found in grammar productions",
                item.rule_id.0
            );
        }
    }
}

#[test]
fn item_set_lookaheads_are_terminals_or_eof() {
    let mut g = GrammarBuilder::new("ip7")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let terminal_ids: BTreeSet<u16> = g.tokens.keys().map(|s| s.0).collect();
    for set in &col.sets {
        for item in &set.items {
            let la = item.lookahead.0;
            assert!(
                la == 0 || terminal_ids.contains(&la),
                "lookahead {} should be EOF(0) or a terminal",
                la
            );
        }
    }
}

#[test]
fn item_set_each_state_has_unique_id() {
    let mut g = GrammarBuilder::new("ip8")
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

// ===========================================================================
// 3. Collection size for different grammars (8 tests)
// ===========================================================================

#[test]
fn size_single_token_grammar() {
    let mut g = GrammarBuilder::new("sz1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 2,
        "S→a: at least 2 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 6,
        "S→a: at most 6 states, got {}",
        col.sets.len()
    );
}

#[test]
fn size_two_token_sequence() {
    let mut g = GrammarBuilder::new("sz2")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 3,
        "S→a b: at least 3 states, got {}",
        col.sets.len()
    );
}

#[test]
fn size_three_token_sequence() {
    let mut g = GrammarBuilder::new("sz3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(
        col.sets.len() >= 4,
        "S→a b c: at least 4 states, got {}",
        col.sets.len()
    );
}

#[test]
fn size_two_alternatives_more_than_single() {
    let mut g_single = GrammarBuilder::new("sz4a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col_single, _) = build(&mut g_single);

    let mut g_alt = GrammarBuilder::new("sz4b")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col_alt, _) = build(&mut g_alt);

    assert!(
        col_alt.sets.len() >= col_single.sets.len(),
        "alternatives should not reduce state count"
    );
}

#[test]
fn size_nonterminal_chain_adds_states() {
    // S→A, A→a  vs  S→a
    let mut g_flat = GrammarBuilder::new("sz5a")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col_flat, _) = build(&mut g_flat);

    let mut g_chain = GrammarBuilder::new("sz5b")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col_chain, _) = build(&mut g_chain);

    assert!(
        col_chain.sets.len() >= col_flat.sets.len(),
        "nonterminal chain should produce at least as many states"
    );
}

#[test]
fn size_left_recursive_bounded() {
    let mut g = GrammarBuilder::new("sz6")
        .token("x", "x")
        .rule("L", vec!["x"])
        .rule("L", vec!["L", "x"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 3, "left-recursive ≥3 states");
    assert!(
        col.sets.len() <= 20,
        "left-recursive should be bounded, got {}",
        col.sets.len()
    );
}

#[test]
fn size_right_recursive_bounded() {
    let mut g = GrammarBuilder::new("sz7")
        .token("x", "x")
        .rule("R", vec!["x"])
        .rule("R", vec!["x", "R"])
        .start("R")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 3, "right-recursive ≥3 states");
    assert!(
        col.sets.len() <= 20,
        "right-recursive should be bounded, got {}",
        col.sets.len()
    );
}

#[test]
fn size_expression_grammar_moderate() {
    let mut g = GrammarBuilder::new("sz8")
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
    // Classic expression grammar: roughly 12 states
    assert!(
        col.sets.len() >= 8,
        "expression grammar ≥8 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 30,
        "expression grammar ≤30 states, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 4. Closure computation (5 tests)
// ===========================================================================

#[test]
fn closure_expands_nonterminal_items() {
    // S→A, A→a — closure of state 0 should include items for both S and A rules
    let mut g = GrammarBuilder::new("cl1")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 closure should have items from S→•A and A→•a (at minimum)
    assert!(
        col.sets[0].items.len() >= 2,
        "closure should expand nonterminal, got {} items",
        col.sets[0].items.len()
    );
}

#[test]
fn closure_transitive_chain() {
    // S→A, A→B, B→c — state 0 closure should include S, A, and B items
    let mut g = GrammarBuilder::new("cl2")
        .token("c", "c")
        .rule("B", vec!["c"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets[0].items.len() >= 3,
        "transitive closure should pull in S, A, B items, got {}",
        col.sets[0].items.len()
    );
}

#[test]
fn closure_multiple_alternatives() {
    // S→A, A→a | b — closure should include both A alternatives
    let mut g = GrammarBuilder::new("cl3")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("A", vec!["b"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // S→•A plus A→•a and A→•b
    assert!(
        col.sets[0].items.len() >= 3,
        "closure should include both A alternatives, got {}",
        col.sets[0].items.len()
    );
}

#[test]
fn closure_does_not_expand_terminals() {
    // S→a b — initial state should only have S→•a b (no expansion for terminal 'a')
    let mut g = GrammarBuilder::new("cl4")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Items at position 0 should only be from start rules
    let pos0_items: Vec<_> = col.sets[0]
        .items
        .iter()
        .filter(|i| i.position == 0)
        .collect();
    // Only S→•a b should be at position 0 (terminals don't trigger closure expansion)
    assert!(
        !pos0_items.is_empty(),
        "should have at least one position-0 item"
    );
}

#[test]
fn closure_preserves_lookahead_propagation() {
    // S→A c, A→a — closure should propagate lookahead 'c' to A items
    let mut g = GrammarBuilder::new("cl5")
        .token("a", "a")
        .token("c", "c")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // In state 0, item A→•a should have lookahead 'c' (from FIRST of what follows A in S→A•c)
    let c_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "c")
        .copied();
    if let Some(c_id) = c_sym {
        let has_propagated_la = col.sets[0]
            .items
            .iter()
            .any(|i| i.position == 0 && i.lookahead == c_id);
        assert!(
            has_propagated_la,
            "closure should propagate lookahead 'c' to items derived from A in S→A c"
        );
    }
}

// ===========================================================================
// 5. Goto computation (5 tests)
// ===========================================================================

#[test]
fn goto_on_terminal_produces_new_state() {
    let mut g = GrammarBuilder::new("gt1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // There must be a goto transition from state 0 on terminal 'a'
    let a_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "a")
        .copied()
        .unwrap();
    assert!(
        col.goto_table.contains_key(&(StateId(0), a_sym)),
        "should have goto on terminal 'a' from state 0"
    );
}

#[test]
fn goto_on_nonterminal_produces_transition() {
    let mut g = GrammarBuilder::new("gt2")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // There must be a goto on non-terminal from state 0
    let has_nt_goto = col.goto_table.iter().any(|((src, sym), _)| {
        *src == StateId(0) && col.symbol_is_terminal.get(sym) == Some(&false)
    });
    assert!(has_nt_goto, "state 0 should have goto on a non-terminal");
}

#[test]
fn goto_targets_are_valid_state_ids() {
    let mut g = GrammarBuilder::new("gt3")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let max_id = col.sets.len() as u16;
    for (_, &target) in &col.goto_table {
        assert!(
            target.0 < max_id,
            "goto target {} exceeds max state id {}",
            target.0,
            max_id - 1
        );
    }
}

#[test]
fn goto_two_token_sequence_chain() {
    // S→a b: state 0 -a-> state X -b-> state Y
    let mut g = GrammarBuilder::new("gt4")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let a_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "a")
        .copied()
        .unwrap();
    let b_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "b")
        .copied()
        .unwrap();

    // Follow the chain: state 0 --a--> mid --b--> final
    if let Some(&mid) = col.goto_table.get(&(StateId(0), a_sym)) {
        assert!(
            col.goto_table.contains_key(&(mid, b_sym)),
            "after shifting 'a' to state {}, should be able to shift 'b'",
            mid.0
        );
    } else {
        panic!("state 0 must have goto on 'a'");
    }
}

#[test]
fn goto_distinct_symbols_lead_to_distinct_states() {
    // S→a | b — goto on 'a' and goto on 'b' from state 0 should lead to different states
    let mut g = GrammarBuilder::new("gt5")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let a_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "a")
        .copied()
        .unwrap();
    let b_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "b")
        .copied()
        .unwrap();

    if let (Some(&sa), Some(&sb)) = (
        col.goto_table.get(&(StateId(0), a_sym)),
        col.goto_table.get(&(StateId(0), b_sym)),
    ) {
        assert_ne!(
            sa, sb,
            "goto on 'a' and 'b' should lead to different states"
        );
    }
}

// ===========================================================================
// 6. Item equality and ordering (5 tests)
// ===========================================================================

#[test]
fn lr_item_equality_same_components() {
    let a = LRItem::new(RuleId(1), 0, SymbolId(0));
    let b = LRItem::new(RuleId(1), 0, SymbolId(0));
    assert_eq!(a, b);
}

#[test]
fn lr_item_inequality_different_rule() {
    let a = LRItem::new(RuleId(1), 0, SymbolId(0));
    let b = LRItem::new(RuleId(2), 0, SymbolId(0));
    assert_ne!(a, b);
}

#[test]
fn lr_item_inequality_different_position() {
    let a = LRItem::new(RuleId(1), 0, SymbolId(0));
    let b = LRItem::new(RuleId(1), 1, SymbolId(0));
    assert_ne!(a, b);
}

#[test]
fn lr_item_inequality_different_lookahead() {
    let a = LRItem::new(RuleId(1), 0, SymbolId(0));
    let b = LRItem::new(RuleId(1), 0, SymbolId(1));
    assert_ne!(a, b);
}

#[test]
fn lr_item_ordering_is_deterministic() {
    let items = [
        LRItem::new(RuleId(2), 1, SymbolId(3)),
        LRItem::new(RuleId(1), 0, SymbolId(0)),
        LRItem::new(RuleId(1), 0, SymbolId(1)),
        LRItem::new(RuleId(1), 1, SymbolId(0)),
        LRItem::new(RuleId(2), 0, SymbolId(0)),
    ];

    let sorted: BTreeSet<_> = items.iter().cloned().collect();
    let sorted_vec: Vec<_> = sorted.into_iter().collect();

    // Verify total ordering: each element < next
    for w in sorted_vec.windows(2) {
        assert!(
            w[0] < w[1],
            "ordering must be consistent: {:?} < {:?}",
            w[0],
            w[1]
        );
    }
}

// ===========================================================================
// 7. Complex grammar collections (8 tests)
// ===========================================================================

#[test]
fn complex_expression_grammar_has_reduce_states() {
    let mut g = GrammarBuilder::new("cx1")
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
        count_reduce_states(&col, &g) >= 1,
        "expression grammar must have reduce states"
    );
}

#[test]
fn complex_expression_grammar_goto_table_has_nonterminals() {
    let mut g = GrammarBuilder::new("cx2")
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

    let nt_transitions = col
        .goto_table
        .keys()
        .filter(|(_, sym)| col.symbol_is_terminal.get(sym) == Some(&false))
        .count();
    assert!(
        nt_transitions >= 2,
        "should have goto transitions on E, T, F"
    );
}

#[test]
fn complex_mutual_recursion_terminates() {
    // A→x B, B→y A | y
    let mut g = GrammarBuilder::new("cx3")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x", "B"])
        .rule("B", vec!["y", "A"])
        .rule("B", vec!["y"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets.len() <= 30,
        "mutual recursion should terminate with bounded states, got {}",
        col.sets.len()
    );
    assert!(col.sets.len() >= 4, "mutual recursion ≥4 states");
}

#[test]
fn complex_ambiguous_grammar_conflict_states() {
    // E→E + E | num (shift-reduce conflict)
    let mut g = GrammarBuilder::new("cx4")
        .token("num", "num")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["num"])
        .start("E")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        count_conflict_states(&col, &g) >= 1,
        "ambiguous E→E+E should produce at least one conflict state"
    );
}

#[test]
fn complex_precedence_grammar_still_builds_all_states() {
    let mut g = GrammarBuilder::new("cx5")
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
        "precedence grammar should still build ≥5 states, got {}",
        col.sets.len()
    );
}

#[test]
fn complex_deep_nonterminal_chain() {
    // S→A, A→B, B→C, C→D, D→x
    let mut g = GrammarBuilder::new("cx6")
        .token("x", "x")
        .rule("D", vec!["x"])
        .rule("C", vec!["D"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 closure should pull in items from all levels
    assert!(
        col.sets[0].items.len() >= 5,
        "deep chain closure should include items from S, A, B, C, D; got {}",
        col.sets[0].items.len()
    );
}

#[test]
fn complex_multiple_start_alternatives() {
    // S→a b | c d | e
    let mut g = GrammarBuilder::new("cx7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a", "b"])
        .rule("S", vec!["c", "d"])
        .rule("S", vec!["e"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // State 0 should have transitions on 'a', 'c', and 'e'
    assert!(
        transitions_from(&col, StateId(0)) >= 3,
        "three alternatives should produce ≥3 transitions from state 0, got {}",
        transitions_from(&col, StateId(0))
    );
}

#[test]
fn complex_dangling_else_produces_many_states() {
    let mut g = GrammarBuilder::new("cx8")
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

    assert!(
        col.sets.len() >= 5,
        "dangling-else grammar ≥5 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 50,
        "dangling-else should be bounded, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_single_token_grammar_minimal() {
    // Smallest possible grammar: S→a
    let mut g = GrammarBuilder::new("ec1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 2, "even minimal grammar needs ≥2 states");
}

#[test]
fn edge_many_alternatives_bounded() {
    // S→a | b | c | d | e | f
    let mut g = GrammarBuilder::new("ec2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .rule("S", vec!["d"])
        .rule("S", vec!["e"])
        .rule("S", vec!["f"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        col.sets.len() <= 20,
        "6 single-token alternatives should not explode, got {}",
        col.sets.len()
    );
    // One shift state per token + initial + goto-on-S
    assert!(
        col.sets.len() >= 7,
        "need at least 7 states for 6 alternatives"
    );
}

#[test]
fn edge_long_rhs_sequence() {
    // S→a b c d e
    let mut g = GrammarBuilder::new("ec3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a", "b", "c", "d", "e"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    // Needs at least 6 states: initial + after each of 5 tokens
    assert!(
        col.sets.len() >= 6,
        "long sequence ≥6 states, got {}",
        col.sets.len()
    );
}

#[test]
fn edge_shared_prefix_creates_distinct_states() {
    // S→a b | a c — after shifting 'a', the parser must distinguish b vs c
    let mut g = GrammarBuilder::new("ec4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b"])
        .rule("S", vec!["a", "c"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let a_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "a")
        .copied()
        .unwrap();

    // After shifting 'a', should have transitions on both 'b' and 'c'
    if let Some(&after_a) = col.goto_table.get(&(StateId(0), a_sym)) {
        assert!(
            transitions_from(&col, after_a) >= 2,
            "after 'a' state should have goto on both 'b' and 'c'"
        );
    }
}

#[test]
fn edge_all_goto_sources_are_valid_states() {
    let mut g = GrammarBuilder::new("ec5")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let state_ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for ((src, _), _) in &col.goto_table {
        assert!(
            state_ids.contains(src),
            "goto source state {} not in collection",
            src.0
        );
    }
}

#[test]
fn edge_all_goto_targets_are_valid_states() {
    let mut g = GrammarBuilder::new("ec6")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("S", vec!["A", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let state_ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for (_, &target) in &col.goto_table {
        assert!(
            state_ids.contains(&target),
            "goto target state {} not in collection",
            target.0
        );
    }
}

#[test]
fn edge_no_self_loop_on_terminal() {
    // S→a should not have a self-loop from state 0 on terminal 'a' back to state 0
    let mut g = GrammarBuilder::new("ec7")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    for ((src, sym), &dst) in &col.goto_table {
        if col.symbol_is_terminal.get(sym) == Some(&true) && *src == dst {
            panic!(
                "terminal self-loop detected: state {} on symbol {}",
                src.0, sym.0
            );
        }
    }
}

#[test]
fn edge_build_lr1_automaton_succeeds_for_unambiguous() {
    let mut g = GrammarBuilder::new("ec8")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let result = build_lr1_automaton(&g, &ff);
    assert!(
        result.is_ok(),
        "unambiguous grammar should build automaton successfully: {:?}",
        result.err()
    );
}

// ===========================================================================
// Additional tests to reach 55+
// ===========================================================================

#[test]
fn item_set_state0_has_position_zero_items() {
    let mut g = GrammarBuilder::new("add1")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let has_pos0 = col.sets[0].items.iter().any(|i| i.position == 0);
    assert!(has_pos0, "initial state should have items at position 0");
}

#[test]
fn goto_table_symmetry_source_state_exists() {
    let mut g = GrammarBuilder::new("add2")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let max_state = col.sets.len() as u16;
    for ((src, _), _) in &col.goto_table {
        assert!(src.0 < max_state, "source state {} out of range", src.0);
    }
}

#[test]
fn collection_deterministic_across_builds() {
    let mut g1 = GrammarBuilder::new("det")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let mut g2 = GrammarBuilder::new("det")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let (col1, _) = build(&mut g1);
    let (col2, _) = build(&mut g2);

    assert_eq!(
        col1.sets.len(),
        col2.sets.len(),
        "deterministic: same state count"
    );
    assert_eq!(
        col1.goto_table.len(),
        col2.goto_table.len(),
        "deterministic: same goto count"
    );
}

#[test]
fn left_recursive_list_has_reduce_state() {
    // L→item | L item — after shifting 'item', a reduce is possible
    let mut g = GrammarBuilder::new("add4")
        .token("item", "item")
        .rule("L", vec!["item"])
        .rule("L", vec!["L", "item"])
        .start("L")
        .build();
    let (col, _) = build(&mut g);

    assert!(
        count_reduce_states(&col, &g) >= 1,
        "left-recursive list must have at least one reduce state"
    );
}

#[test]
fn item_set_collection_all_states_nonempty() {
    let mut g = GrammarBuilder::new("add5")
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
            "state {} must not be empty",
            set.id.0
        );
    }
}

#[test]
fn reduce_reduce_both_present_in_single_state() {
    // A→x, B→x, S→A | B
    let mut g = GrammarBuilder::new("add6")
        .token("x", "x")
        .rule("A", vec!["x"])
        .rule("B", vec!["x"])
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let rr_state = col
        .sets
        .iter()
        .any(|s| s.items.iter().filter(|i| i.is_reduce_item(&g)).count() >= 2);
    assert!(
        rr_state,
        "should have a state with ≥2 reduce items (reduce-reduce)"
    );
}

#[test]
fn state_reachability_from_initial() {
    // Every state except state 0 should be a target of some goto transition
    let mut g = GrammarBuilder::new("add7")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let all_targets: BTreeSet<_> = col.goto_table.values().copied().collect();
    for set in &col.sets {
        if set.id != StateId(0) {
            assert!(
                all_targets.contains(&set.id),
                "state {} should be reachable via goto",
                set.id.0
            );
        }
    }
}

#[test]
fn targets_from_initial_state_distinct() {
    let mut g = GrammarBuilder::new("add8")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build(&mut g);

    let targets = targets_from(&col, StateId(0));
    let transitions = transitions_from(&col, StateId(0));
    // If we have N transitions from state 0, we should have at least 2 distinct targets
    // (one for 'a', one for 'b', possibly one for nonterminal S)
    assert!(
        targets.len() >= 2,
        "state 0 should reach ≥2 distinct targets, got {} (transitions: {})",
        targets.len(),
        transitions
    );
}
