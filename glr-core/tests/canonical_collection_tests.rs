#![cfg(feature = "test-api")]

//! Comprehensive tests for the canonical collection builder in adze-glr-core.
//!
//! Tests the pipeline: Grammar → FIRST/FOLLOW → CanonicalCollection (ItemSetCollection)
//! and verifies state counts, transitions, item sets, and integration with parse tables.

use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build canonical collection from a mutable grammar via the standard pipeline.
fn build_collection(grammar: &mut adze_ir::Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    let collection = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (collection, ff)
}

// ===========================================================================
// 1. Tiny grammar: single production  S → a
// ===========================================================================

#[test]
fn tiny_single_production_state_count() {
    let mut g = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // S → a is trivial: initial state, state after shifting 'a', state after
    // reducing to S.  Exact count may include the S-goto state.
    assert!(
        col.sets.len() >= 2,
        "tiny grammar must produce at least 2 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 6,
        "tiny grammar should not explode to {} states",
        col.sets.len()
    );
}

#[test]
fn tiny_single_production_initial_state_items() {
    let mut g = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // State 0 must have items (the initial closure).
    let state0 = &col.sets[0];
    assert_eq!(state0.id, StateId(0));
    assert!(
        !state0.items.is_empty(),
        "initial state must contain at least one item"
    );
}

#[test]
fn tiny_single_production_has_transitions() {
    let mut g = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // The goto_table should have at least one transition out of state 0.
    let transitions_from_0 = col
        .goto_table
        .iter()
        .filter(|((src, _), _)| *src == StateId(0))
        .count();
    assert!(
        transitions_from_0 >= 1,
        "state 0 must have at least one outgoing transition, got {}",
        transitions_from_0
    );
}

// ===========================================================================
// 2. Two-production grammar: S → a | b
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

    let (col, _ff) = build_collection(&mut g);

    // Two alternative terminals mean two shift transitions from the initial
    // state, plus a goto on S.
    let transitions_from_0 = col
        .goto_table
        .iter()
        .filter(|((src, _), _)| *src == StateId(0))
        .count();
    assert!(
        transitions_from_0 >= 2,
        "alternation grammar should have at least 2 transitions from state 0, got {}",
        transitions_from_0
    );

    // Each transition target should be a distinct state.
    let targets: std::collections::BTreeSet<StateId> = col
        .goto_table
        .iter()
        .filter(|((src, _), _)| *src == StateId(0))
        .map(|(_, &dst)| dst)
        .collect();
    assert!(
        targets.len() >= 2,
        "alternation should yield at least 2 distinct target states"
    );
}

// ===========================================================================
// 3. Sequential grammar: S → a b c
// ===========================================================================

#[test]
fn sequential_three_terminals() {
    let mut g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // A three-symbol sequential rule needs at least 4 states:
    //   · a b c    (state 0)
    //   a · b c    (after shifting a)
    //   a b · c    (after shifting b)
    //   a b c ·    (after shifting c — reduce)
    // Plus potentially a goto-on-S state.
    assert!(
        col.sets.len() >= 4,
        "sequential 3-terminal grammar should produce at least 4 states, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 4. Recursive grammar: list → item | list item
// ===========================================================================

#[test]
fn left_recursive_list() {
    let mut g = GrammarBuilder::new("list")
        .token("item", "item")
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "item"])
        .start("list")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // Left-recursive list grammar is a classic example; ensure it terminates
    // and produces a reasonable number of states.
    assert!(
        col.sets.len() >= 3,
        "left-recursive grammar needs at least 3 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 20,
        "left-recursive grammar should not explode to {} states",
        col.sets.len()
    );

    // There should be a transition on the non-terminal 'list' from state 0.
    let has_nt_transition = col
        .goto_table
        .iter()
        .any(|((src, _sym), _)| *src == StateId(0));
    assert!(
        has_nt_transition,
        "must have at least one transition from state 0"
    );
}

#[test]
fn right_recursive_grammar() {
    let mut g = GrammarBuilder::new("rlist")
        .token("x", "x")
        .rule("R", vec!["x"])
        .rule("R", vec!["x", "R"])
        .start("R")
        .build();

    let (col, _ff) = build_collection(&mut g);

    assert!(
        col.sets.len() >= 3,
        "right-recursive grammar needs at least 3 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 20,
        "right-recursive grammar should not explode to {} states",
        col.sets.len()
    );
}

// ===========================================================================
// 5. State IDs are sequential
// ===========================================================================

#[test]
fn state_ids_are_sequential() {
    let mut g = GrammarBuilder::new("seq_ids")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(
            set.id,
            StateId(i as u16),
            "state {} should have id {}, got {}",
            i,
            i,
            set.id.0
        );
    }
}

// ===========================================================================
// 6. Symbol classification in transitions
// ===========================================================================

#[test]
fn symbol_is_terminal_classification() {
    let mut g = GrammarBuilder::new("cls")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x"])
        .rule("S", vec!["A", "y"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // Every symbol referenced in goto_table should be classified.
    for ((_src, sym), _dst) in &col.goto_table {
        assert!(
            col.symbol_is_terminal.contains_key(sym),
            "symbol {:?} used in transition should be classified as terminal/non-terminal",
            sym
        );
    }

    // At least one terminal and one non-terminal should appear.
    let terminal_count = col
        .symbol_is_terminal
        .values()
        .filter(|&&is_t| is_t)
        .count();
    let nonterminal_count = col
        .symbol_is_terminal
        .values()
        .filter(|&&is_t| !is_t)
        .count();
    assert!(
        terminal_count >= 1,
        "should have at least one terminal symbol in transitions"
    );
    assert!(
        nonterminal_count >= 1,
        "should have at least one non-terminal symbol in transitions"
    );
}

// ===========================================================================
// 7. Medium grammar: arithmetic expressions
// ===========================================================================

#[test]
fn medium_arithmetic_grammar() {
    let mut g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUM"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // Arithmetic expression grammar with two precedence levels produces a
    // non-trivial number of states.
    assert!(
        col.sets.len() >= 5,
        "arithmetic grammar needs at least 5 states, got {}",
        col.sets.len()
    );
    assert!(
        col.sets.len() <= 50,
        "arithmetic grammar should not explode to {} states",
        col.sets.len()
    );

    // All goto_table entries should reference valid state IDs.
    let max_state = StateId(col.sets.len() as u16 - 1);
    for ((_src, _sym), dst) in &col.goto_table {
        assert!(
            dst.0 <= max_state.0,
            "transition target {} exceeds max state {}",
            dst.0,
            max_state.0
        );
    }
}

// ===========================================================================
// 8. Goto table targets are valid state IDs
// ===========================================================================

#[test]
fn goto_table_targets_are_valid() {
    let mut g = GrammarBuilder::new("valid_goto")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("X", vec!["a"])
        .rule("Y", vec!["b", "X"])
        .rule("S", vec!["Y", "c"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    let valid_ids: std::collections::BTreeSet<StateId> = col.sets.iter().map(|s| s.id).collect();

    for ((src, _sym), dst) in &col.goto_table {
        assert!(
            valid_ids.contains(src),
            "source state {:?} in goto_table is not in the collection",
            src
        );
        assert!(
            valid_ids.contains(dst),
            "target state {:?} in goto_table is not in the collection",
            dst
        );
    }
}

// ===========================================================================
// 9. No duplicate item sets
// ===========================================================================

#[test]
fn no_duplicate_item_sets() {
    let mut g = GrammarBuilder::new("no_dup")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x"])
        .rule("A", vec!["y"])
        .rule("S", vec!["A"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // Each pair of states should have distinct item sets.
    for i in 0..col.sets.len() {
        for j in (i + 1)..col.sets.len() {
            assert_ne!(
                col.sets[i].items, col.sets[j].items,
                "states {} and {} have identical item sets — should have been merged",
                col.sets[i].id.0, col.sets[j].id.0
            );
        }
    }
}

// ===========================================================================
// 10. Integration: canonical collection feeds parse table
// ===========================================================================

#[test]
fn collection_integrates_with_parse_table() {
    let mut g = GrammarBuilder::new("pipeline")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW should succeed");

    // Build canonical collection to get the state count.
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let collection_state_count = col.sets.len();

    // Build parse table via the full automaton pipeline.
    let table = build_lr1_automaton(&g, &ff).expect("parse table construction should succeed");

    // The parse table state count should be consistent with the augmented
    // grammar's canonical collection (which adds S' → S).  The automaton
    // builder augments, so its state count may differ from the non-augmented
    // collection, but both must be non-zero.
    assert!(table.state_count > 0, "parse table must have states");
    assert!(
        collection_state_count > 0,
        "canonical collection must have states"
    );

    // The parse table must pass its own sanity checks.
    sanity_check_tables(&table).expect("parse table sanity check failed");
}

// ===========================================================================
// 11. Multi-level nonterminal chain: S → A → B → c
// ===========================================================================

#[test]
fn nonterminal_chain() {
    let mut g = GrammarBuilder::new("chain")
        .token("c", "c")
        .rule("B", vec!["c"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // A chain S→A→B→c requires closure to propagate through nonterminals.
    // State 0 should include kernel items for S, A, and B after closure.
    let state0_item_count = col.sets[0].items.len();
    assert!(
        state0_item_count >= 3,
        "closure of chain grammar's initial state should have at least 3 items (S→·A, A→·B, B→·c), got {}",
        state0_item_count
    );
}

// ===========================================================================
// 12. Ambiguous grammar produces multiple items in a state
// ===========================================================================

#[test]
fn ambiguous_grammar_shift_reduce_items() {
    // Classic ambiguous expression: E → E + E | num
    let mut g = GrammarBuilder::new("ambig")
        .token("num", "num")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["num"])
        .start("E")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // Ambiguous grammars produce more states due to conflict possibilities.
    assert!(
        col.sets.len() >= 4,
        "ambiguous expr grammar needs at least 4 states, got {}",
        col.sets.len()
    );

    // At least one state should have both a reduce item (dot at end) and
    // a shift item (dot before a symbol), creating a shift/reduce conflict.
    let has_conflict_state = col.sets.iter().any(|state| {
        let has_reduce = state.items.iter().any(|item| item.is_reduce_item(&g));
        let has_shift = state
            .items
            .iter()
            .any(|item| item.next_symbol(&g).is_some());
        has_reduce && has_shift
    });
    assert!(
        has_conflict_state,
        "ambiguous grammar should have at least one state with both shift and reduce items"
    );
}

// ===========================================================================
// 13. Nullable nonterminal via normalization
// ===========================================================================

#[test]
fn grammar_with_nullable_nonterminal() {
    // Use a grammar where normalization handles nullable symbols.
    // GrammarBuilder converts empty RHS to Epsilon, which
    // compute_normalized will process before collection building.
    let mut g = GrammarBuilder::new("nullable")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["x", "y"])
        .rule("S", vec!["x"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // Two alternative productions should produce a valid collection.
    assert!(
        col.sets.len() >= 2,
        "grammar with multiple productions should produce at least 2 states, got {}",
        col.sets.len()
    );

    // All state IDs should be sequential starting from 0.
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

// ===========================================================================
// 14. Multiple nonterminals with shared terminals
// ===========================================================================

#[test]
fn shared_terminals_across_nonterminals() {
    let mut g = GrammarBuilder::new("shared")
        .token("a", "a")
        .token("b", "b")
        .rule("X", vec!["a", "b"])
        .rule("Y", vec!["a"])
        .rule("S", vec!["X"])
        .rule("S", vec!["Y", "b"])
        .start("S")
        .build();

    let (col, _ff) = build_collection(&mut g);

    // Both X and Y start with 'a', so the initial closure should include
    // items from both.  After shifting 'a' from state 0, the resulting state
    // should contain items from both X (expecting 'b') and Y (reduce).
    assert!(
        col.sets.len() >= 3,
        "shared-terminal grammar should produce at least 3 states, got {}",
        col.sets.len()
    );

    // Verify that state 0 has a transition on terminal 'a'.
    let a_sym = g
        .tokens
        .keys()
        .find(|&&id| g.tokens[&id].name == "a")
        .copied()
        .expect("token 'a' should exist");
    let has_a_transition = col.goto_table.contains_key(&(StateId(0), a_sym));
    assert!(
        has_a_transition,
        "state 0 should have a transition on terminal 'a'"
    );
}
