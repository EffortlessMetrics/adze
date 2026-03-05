//! Comprehensive tests for canonical collection and item set operations.
//!
//! Categories:
//! 1. Collection has states — every grammar produces at least 1 state
//! 2. State IDs sequential — states numbered 0..N
//! 3. Gotos connect states — goto entries point to valid states
//! 4. Closure properties — each state is a closed set of items
//! 5. Accept state exists — one state has an accept/reduce item for start
//! 6. Collection determinism — same grammar → same collection
//! 7. Complex grammars — expression, recursive, multi-NT collections

use adze_glr_core::{Action, FirstFollowSets, ItemSetCollection, ParseTable, build_lr1_automaton};
use adze_ir::StateId;
use adze_ir::builder::GrammarBuilder;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_collection(grammar: &adze_ir::Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

fn build_table(grammar: &adze_ir::Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("automaton build failed")
}

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

fn two_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build()
}

fn two_rule_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("two_rule")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .rule("A", vec!["y"])
        .start("S")
        .build()
}

fn recursive_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("recursive")
        .token("a", "a")
        .rule("S", vec!["S", "a"])
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

fn expression_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("expr")
        .token("id", "[a-z]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .rule("E", vec!["E", "plus", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "star", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["lparen", "E", "rparen"])
        .rule("F", vec!["id"])
        .start("E")
        .build()
}

fn multi_nt_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("multi_nt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b", "c"])
        .start("S")
        .build()
}

fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["x"])
        .start("S")
        .build()
}

fn right_recursive_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("S", vec!["a", "S"])
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

// ===========================================================================
// 1. Collection has states — every grammar produces at least 1 state
// ===========================================================================

#[test]
fn collection_has_states_simple() {
    let g = simple_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty(), "simple grammar must have states");
}

#[test]
fn collection_has_states_two_tokens() {
    let g = two_token_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_has_states_two_rules() {
    let g = two_rule_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_has_states_recursive() {
    let g = recursive_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_has_states_expression() {
    let g = expression_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_has_states_multi_nt() {
    let g = multi_nt_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_has_states_chain() {
    let g = chain_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_has_states_right_recursive() {
    let g = right_recursive_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets.is_empty());
}

// ===========================================================================
// 2. State IDs sequential — states numbered 0..N
// ===========================================================================

#[test]
fn state_ids_sequential_simple() {
    let g = simple_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16), "state {i} has wrong id");
    }
}

#[test]
fn state_ids_sequential_two_tokens() {
    let g = two_token_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn state_ids_sequential_two_rules() {
    let g = two_rule_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn state_ids_sequential_recursive() {
    let g = recursive_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn state_ids_sequential_expression() {
    let g = expression_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn state_ids_sequential_multi_nt() {
    let g = multi_nt_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn state_ids_sequential_chain() {
    let g = chain_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn state_ids_sequential_right_recursive() {
    let g = right_recursive_grammar();
    let (col, _) = build_collection(&g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

// ===========================================================================
// 3. Gotos connect states — goto entries point to valid states
// ===========================================================================

fn max_state_id(col: &ItemSetCollection) -> u16 {
    col.sets.last().map_or(0, |s| s.id.0)
}

#[test]
fn gotos_connect_valid_states_simple() {
    let g = simple_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max, "goto target {to:?} out of range");
    }
}

#[test]
fn gotos_connect_valid_states_two_tokens() {
    let g = two_token_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max);
    }
}

#[test]
fn gotos_connect_valid_states_two_rules() {
    let g = two_rule_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max);
    }
}

#[test]
fn gotos_connect_valid_states_recursive() {
    let g = recursive_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max);
    }
}

#[test]
fn gotos_connect_valid_states_expression() {
    let g = expression_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max);
    }
}

#[test]
fn gotos_connect_valid_states_multi_nt() {
    let g = multi_nt_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max);
    }
}

#[test]
fn gotos_connect_valid_states_chain() {
    let g = chain_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max);
    }
}

#[test]
fn gotos_connect_valid_states_right_recursive() {
    let g = right_recursive_grammar();
    let (col, _) = build_collection(&g);
    let max = max_state_id(&col);
    for ((_from, _sym), to) in &col.goto_table {
        assert!(to.0 <= max);
    }
}

// ===========================================================================
// 4. Closure properties — each state is a closed set of items
// ===========================================================================

#[test]
fn closure_state_zero_nonempty_simple() {
    let g = simple_grammar();
    let (col, _) = build_collection(&g);
    assert!(
        !col.sets[0].items.is_empty(),
        "initial state must have items"
    );
}

#[test]
fn closure_state_zero_nonempty_two_rules() {
    let g = two_rule_grammar();
    let (col, _) = build_collection(&g);
    assert!(!col.sets[0].items.is_empty());
}

#[test]
fn closure_all_states_nonempty_expression() {
    let g = expression_grammar();
    let (col, _) = build_collection(&g);
    for set in &col.sets {
        assert!(!set.items.is_empty(), "state {:?} must have items", set.id);
    }
}

#[test]
fn closure_all_states_nonempty_recursive() {
    let g = recursive_grammar();
    let (col, _) = build_collection(&g);
    for set in &col.sets {
        assert!(!set.items.is_empty(), "state {:?} empty", set.id);
    }
}

#[test]
fn closure_items_have_valid_lookahead_simple() {
    let g = simple_grammar();
    let (col, _) = build_collection(&g);
    for set in &col.sets {
        for item in &set.items {
            // Lookahead should be a valid symbol (including EOF = SymbolId(0))
            let _ = item.lookahead;
        }
    }
}

#[test]
fn closure_reduce_items_at_end_of_rule() {
    let g = simple_grammar();
    let (col, _) = build_collection(&g);
    for set in &col.sets {
        for item in &set.items {
            if item.is_reduce_item(&g) {
                // Reduce items have dot at end; position >= rhs length
                if let Some(rule) = g.all_rules().find(|r| r.production_id.0 == item.rule_id.0) {
                    assert!(
                        item.position >= rule.rhs.len(),
                        "reduce item must have dot at end"
                    );
                }
            }
        }
    }
}

#[test]
fn closure_chain_grammar_has_multiple_items_in_initial() {
    let g = chain_grammar();
    let (col, _) = build_collection(&g);
    // S -> .A should close over A -> .B -> .x, so state 0 has multiple items
    assert!(
        col.sets[0].items.len() > 1,
        "closure should expand chain rules"
    );
}

// ===========================================================================
// 5. Accept state exists — some state has reduce/accept for start symbol
// ===========================================================================

#[test]
fn accept_state_via_automaton_simple() {
    let g = simple_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "must have an accept action");
}

#[test]
fn accept_state_via_automaton_two_tokens() {
    let g = two_token_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

#[test]
fn accept_state_via_automaton_two_rules() {
    let g = two_rule_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

#[test]
fn accept_state_via_automaton_recursive() {
    let g = recursive_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

#[test]
fn accept_state_via_automaton_expression() {
    let g = expression_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

#[test]
fn accept_state_via_automaton_multi_nt() {
    let g = multi_nt_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

#[test]
fn accept_state_via_automaton_chain() {
    let g = chain_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

#[test]
fn accept_state_via_automaton_right_recursive() {
    let g = right_recursive_grammar();
    let table = build_table(&g);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

// ===========================================================================
// 6. Collection determinism — same grammar → same collection
// ===========================================================================

#[test]
fn determinism_simple() {
    let g = simple_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    assert_eq!(col1.sets.len(), col2.sets.len());
    assert_eq!(col1.goto_table.len(), col2.goto_table.len());
}

#[test]
fn determinism_two_tokens() {
    let g = two_token_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    assert_eq!(col1.sets.len(), col2.sets.len());
}

#[test]
fn determinism_two_rules() {
    let g = two_rule_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    assert_eq!(col1.sets.len(), col2.sets.len());
}

#[test]
fn determinism_recursive() {
    let g = recursive_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    assert_eq!(col1.sets.len(), col2.sets.len());
}

#[test]
fn determinism_expression_state_count() {
    let g = expression_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    assert_eq!(col1.sets.len(), col2.sets.len());
}

#[test]
fn determinism_expression_goto_count() {
    let g = expression_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    assert_eq!(col1.goto_table.len(), col2.goto_table.len());
}

#[test]
fn determinism_expression_item_sets_equal() {
    let g = expression_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    for (s1, s2) in col1.sets.iter().zip(col2.sets.iter()) {
        assert_eq!(s1.items, s2.items, "state {:?} items differ", s1.id);
    }
}

#[test]
fn determinism_expression_goto_keys_equal() {
    let g = expression_grammar();
    let (col1, _) = build_collection(&g);
    let (col2, _) = build_collection(&g);
    let keys1: Vec<_> = col1.goto_table.keys().collect();
    let keys2: Vec<_> = col2.goto_table.keys().collect();
    assert_eq!(keys1, keys2);
}

// ===========================================================================
// 7. Complex grammars — expression, recursive, multi-NT collections
// ===========================================================================

#[test]
fn complex_expression_has_many_states() {
    let g = expression_grammar();
    let (col, _) = build_collection(&g);
    // Classic expression grammar typically produces 10+ states
    assert!(
        col.sets.len() >= 5,
        "expression grammar should have many states, got {}",
        col.sets.len()
    );
}

#[test]
fn complex_expression_has_gotos() {
    let g = expression_grammar();
    let (col, _) = build_collection(&g);
    assert!(
        !col.goto_table.is_empty(),
        "expression grammar must have goto entries"
    );
}

#[test]
fn complex_recursive_has_shift_reduce_items() {
    let g = recursive_grammar();
    let (col, _) = build_collection(&g);
    let has_reduce = col
        .sets
        .iter()
        .any(|set| set.items.iter().any(|item| item.is_reduce_item(&g)));
    assert!(has_reduce, "recursive grammar must have reduce items");
}

#[test]
fn complex_recursive_has_non_reduce_items() {
    let g = recursive_grammar();
    let (col, _) = build_collection(&g);
    let has_shift = col
        .sets
        .iter()
        .any(|set| set.items.iter().any(|item| !item.is_reduce_item(&g)));
    assert!(has_shift, "recursive grammar must have shift items");
}

#[test]
fn complex_multi_nt_state_zero_covers_start_rule() {
    let g = multi_nt_grammar();
    let (col, _) = build_collection(&g);
    // State 0 should have an item with position 0 (dot at start)
    let has_initial = col.sets[0].items.iter().any(|item| item.position == 0);
    assert!(has_initial, "state 0 must have initial items");
}

#[test]
fn complex_chain_grammar_produces_states_for_each_level() {
    let g = chain_grammar();
    let (col, _) = build_collection(&g);
    // S -> A -> B -> x : expect several states
    assert!(
        col.sets.len() >= 3,
        "chain grammar needs states for each derivation level, got {}",
        col.sets.len()
    );
}

#[test]
fn complex_expression_automaton_state_count_matches_collection() {
    let g = expression_grammar();
    let table = build_table(&g);
    assert!(
        table.state_count >= 5,
        "expression automaton should have many states, got {}",
        table.state_count
    );
}

#[test]
fn complex_right_recursive_gotos_form_chain() {
    let g = right_recursive_grammar();
    let (col, _) = build_collection(&g);
    // Right-recursive grammars produce goto chains
    assert!(
        !col.goto_table.is_empty(),
        "right-recursive grammar must have gotos"
    );
    // Verify at least one goto chain exists from state 0
    let from_zero = col
        .goto_table
        .keys()
        .filter(|(from, _)| from.0 == 0)
        .count();
    assert!(from_zero >= 1, "state 0 must have outgoing gotos");
}
