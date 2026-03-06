#![cfg(feature = "test-api")]

//! lr1_item_v9 — 84 tests for LR(1) item sets and canonical collection.
//!
//! Categories:
//! 1.  build_canonical_collection succeeds for simple grammars (8)
//! 2.  Collection len() > 0 and >= 2 (8)
//! 3.  Collection len() matches ParseTable state_count (8)
//! 4.  Single / two token grammar → specific collection sizes (8)
//! 5.  Grammars with alternatives / chains → state counts (8)
//! 6.  Determinism: same grammar → same len() (8)
//! 7.  Different grammars → different len() (4)
//! 8.  Precedence / inline / extras → collection builds (8)
//! 9.  Arithmetic grammar → reasonable state count (4)
//! 10. FirstFollowSets compute doesn't panic (8)
//! 11. Large grammar (10+ tokens) → collection builds (4)
//! 12. Grammars with conflicts → collection builds (4)
//! 13. ParseTable from collection is consistent (4)

use adze_glr_core::{FirstFollowSets, ItemSetCollection, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, StateId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_collection(grammar: &mut Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW computation should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

/// Build both collection and table from the same normalized grammar.
fn build_both(grammar: &mut Grammar) -> (ItemSetCollection, ParseTable) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    let table = build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed");
    (col, table)
}

// ===========================================================================
// 1. build_canonical_collection succeeds for simple grammars (8 tests)
// ===========================================================================

#[test]
fn li_v9_build_single_token() {
    let mut g = GrammarBuilder::new("li_v9_1a")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_build_two_tokens() {
    let mut g = GrammarBuilder::new("li_v9_1b")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_build_alternative() {
    let mut g = GrammarBuilder::new("li_v9_1c")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_build_chain_rule() {
    let mut g = GrammarBuilder::new("li_v9_1d")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_build_recursive() {
    let mut g = GrammarBuilder::new("li_v9_1e")
        .token("a", "a")
        .token("plus", "+")
        .rule("e", vec!["e", "plus", "a"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_build_three_token_seq() {
    let mut g = GrammarBuilder::new("li_v9_1f")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_build_nested_nonterminals() {
    let mut g = GrammarBuilder::new("li_v9_1g")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_build_goto_table_populated() {
    let mut g = GrammarBuilder::new("li_v9_1h")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.goto_table.is_empty());
}

// ===========================================================================
// 2. Collection len() > 0 and >= 2 (8 tests)
// ===========================================================================

#[test]
fn li_v9_len_positive_single() {
    let mut g = GrammarBuilder::new("li_v9_2a")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty(), "must have at least one state");
}

#[test]
fn li_v9_len_ge2_single() {
    let mut g = GrammarBuilder::new("li_v9_2b")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2, "start + at least one shift state");
}

#[test]
fn li_v9_len_ge2_sequence() {
    let mut g = GrammarBuilder::new("li_v9_2c")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2);
}

#[test]
fn li_v9_len_ge2_alternatives() {
    let mut g = GrammarBuilder::new("li_v9_2d")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2);
}

#[test]
fn li_v9_len_ge2_chain() {
    let mut g = GrammarBuilder::new("li_v9_2e")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2);
}

#[test]
fn li_v9_len_ge2_recursive() {
    let mut g = GrammarBuilder::new("li_v9_2f")
        .token("a", "a")
        .token("plus", "+")
        .rule("e", vec!["e", "plus", "a"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2);
}

#[test]
fn li_v9_len_ge2_three_token() {
    let mut g = GrammarBuilder::new("li_v9_2g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2);
}

#[test]
fn li_v9_len_ge2_nested() {
    let mut g = GrammarBuilder::new("li_v9_2h")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2);
}

// ===========================================================================
// 3. Collection and table state counts are consistent (8 tests)
//
// The automaton builder augments the grammar (S' → S), so its state count
// may be larger than the non-augmented canonical collection.  We verify
// both are positive and the table has at least as many states.
// ===========================================================================

#[test]
fn li_v9_match_single_token() {
    let mut g = GrammarBuilder::new("li_v9_3a")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

#[test]
fn li_v9_match_sequence() {
    let mut g = GrammarBuilder::new("li_v9_3b")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

#[test]
fn li_v9_match_alternatives() {
    let mut g = GrammarBuilder::new("li_v9_3c")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

#[test]
fn li_v9_match_chain() {
    let mut g = GrammarBuilder::new("li_v9_3d")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("s")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

#[test]
fn li_v9_match_recursive() {
    let mut g = GrammarBuilder::new("li_v9_3e")
        .token("a", "a")
        .token("plus", "+")
        .rule("e", vec!["e", "plus", "a"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

#[test]
fn li_v9_match_three_tokens() {
    let mut g = GrammarBuilder::new("li_v9_3f")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

#[test]
fn li_v9_match_nested() {
    let mut g = GrammarBuilder::new("li_v9_3g")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("s")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

#[test]
fn li_v9_match_four_alternatives() {
    let mut g = GrammarBuilder::new("li_v9_3h")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .start("s")
        .build();
    let (col, table) = build_both(&mut g);
    assert!(table.state_count >= col.sets.len());
}

// ===========================================================================
// 4. Single / two token grammar → specific collection sizes (8 tests)
// ===========================================================================

#[test]
fn li_v9_single_tok_ge2_states() {
    let mut g = GrammarBuilder::new("li_v9_4a")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() >= 2,
        "single-token grammar needs >= 2 states, got {}",
        col.sets.len()
    );
}

#[test]
fn li_v9_single_tok_le6_states() {
    let mut g = GrammarBuilder::new("li_v9_4b")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() <= 6,
        "single-token grammar should not explode to {} states",
        col.sets.len()
    );
}

#[test]
fn li_v9_two_tok_more_states_than_one() {
    let mut g1 = GrammarBuilder::new("li_v9_4c1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col1, _) = build_collection(&mut g1);

    let mut g2 = GrammarBuilder::new("li_v9_4c2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col2, _) = build_collection(&mut g2);

    assert!(
        col2.sets.len() >= col1.sets.len(),
        "two-token seq ({}) should have >= states than single-token ({})",
        col2.sets.len(),
        col1.sets.len()
    );
}

#[test]
fn li_v9_two_tok_seq_ge3() {
    let mut g = GrammarBuilder::new("li_v9_4d")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3, "two-token seq needs >= 3 states");
}

#[test]
fn li_v9_two_tok_alt_ge3() {
    let mut g = GrammarBuilder::new("li_v9_4e")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3, "two-token alt needs >= 3 states");
}

#[test]
fn li_v9_three_tok_seq_ge4() {
    let mut g = GrammarBuilder::new("li_v9_4f")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4, "three-token seq needs >= 4 states");
}

#[test]
fn li_v9_four_tok_seq_ge5() {
    let mut g = GrammarBuilder::new("li_v9_4g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 5, "four-token seq needs >= 5 states");
}

#[test]
fn li_v9_alt_more_than_chain() {
    let mut g_alt = GrammarBuilder::new("li_v9_4h1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let (col_alt, _) = build_collection(&mut g_alt);

    let mut g_chain = GrammarBuilder::new("li_v9_4h2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col_chain, _) = build_collection(&mut g_chain);

    assert!(
        col_alt.sets.len() >= col_chain.sets.len(),
        "alternatives ({}) should produce >= states than single rule ({})",
        col_alt.sets.len(),
        col_chain.sets.len()
    );
}

// ===========================================================================
// 5. Grammars with alternatives / chains → state counts (8 tests)
// ===========================================================================

#[test]
fn li_v9_alt_3_rules() {
    let mut g = GrammarBuilder::new("li_v9_5a")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3);
}

#[test]
fn li_v9_chain_2_nonterminals() {
    let mut g = GrammarBuilder::new("li_v9_5b")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3);
}

#[test]
fn li_v9_chain_3_nonterminals() {
    let mut g = GrammarBuilder::new("li_v9_5c")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3);
}

#[test]
fn li_v9_mixed_chain_alt() {
    let mut g = GrammarBuilder::new("li_v9_5d")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["x"])
        .rule("mid", vec!["y"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3);
}

#[test]
fn li_v9_left_recursive() {
    let mut g = GrammarBuilder::new("li_v9_5e")
        .token("a", "a")
        .token("plus", "+")
        .rule("e", vec!["e", "plus", "a"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4);
}

#[test]
fn li_v9_right_recursive() {
    let mut g = GrammarBuilder::new("li_v9_5f")
        .token("a", "a")
        .token("plus", "+")
        .rule("e", vec!["a", "plus", "e"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4);
}

#[test]
fn li_v9_alt_with_sequence() {
    let mut g = GrammarBuilder::new("li_v9_5g")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3);
}

#[test]
fn li_v9_multi_nonterminal_alt() {
    let mut g = GrammarBuilder::new("li_v9_5h")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["lhs"])
        .rule("s", vec!["rhs"])
        .rule("lhs", vec!["x"])
        .rule("rhs", vec!["y"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3);
}

// ===========================================================================
// 6. Determinism: same grammar → same len() (8 tests)
// ===========================================================================

fn make_det_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build()
}

#[test]
fn li_v9_determinism_run1_vs_run2() {
    let mut g1 = make_det_grammar("li_v9_6a1");
    let mut g2 = make_det_grammar("li_v9_6a2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn li_v9_determinism_single_token() {
    let mk = |n: &str| {
        GrammarBuilder::new(n)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build()
    };
    let mut g1 = mk("li_v9_6b1");
    let mut g2 = mk("li_v9_6b2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn li_v9_determinism_alt() {
    let mk = |n: &str| {
        GrammarBuilder::new(n)
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build()
    };
    let mut g1 = mk("li_v9_6c1");
    let mut g2 = mk("li_v9_6c2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn li_v9_determinism_chain() {
    let mk = |n: &str| {
        GrammarBuilder::new(n)
            .token("x", "x")
            .rule("s", vec!["mid"])
            .rule("mid", vec!["x"])
            .start("s")
            .build()
    };
    let mut g1 = mk("li_v9_6d1");
    let mut g2 = mk("li_v9_6d2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn li_v9_determinism_recursive() {
    let mk = |n: &str| {
        GrammarBuilder::new(n)
            .token("a", "a")
            .token("plus", "+")
            .rule("e", vec!["e", "plus", "a"])
            .rule("e", vec!["a"])
            .start("e")
            .build()
    };
    let mut g1 = mk("li_v9_6e1");
    let mut g2 = mk("li_v9_6e2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn li_v9_determinism_three_seq() {
    let mk = |n: &str| {
        GrammarBuilder::new(n)
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("s", vec!["a", "b", "c"])
            .start("s")
            .build()
    };
    let mut g1 = mk("li_v9_6f1");
    let mut g2 = mk("li_v9_6f2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn li_v9_determinism_nested() {
    let mk = |n: &str| {
        GrammarBuilder::new(n)
            .token("x", "x")
            .rule("s", vec!["mid"])
            .rule("mid", vec!["leaf"])
            .rule("leaf", vec!["x"])
            .start("s")
            .build()
    };
    let mut g1 = mk("li_v9_6g1");
    let mut g2 = mk("li_v9_6g2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn li_v9_determinism_four_alt() {
    let mk = |n: &str| {
        GrammarBuilder::new(n)
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .rule("s", vec!["c"])
            .rule("s", vec!["d"])
            .start("s")
            .build()
    };
    let mut g1 = mk("li_v9_6h1");
    let mut g2 = mk("li_v9_6h2");
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

// ===========================================================================
// 7. Different grammars → different len() (4 tests)
// ===========================================================================

#[test]
fn li_v9_diff_single_vs_seq() {
    let mut g1 = GrammarBuilder::new("li_v9_7a1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut g2 = GrammarBuilder::new("li_v9_7a2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d"])
        .start("s")
        .build();
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_ne!(
        c1.sets.len(),
        c2.sets.len(),
        "single-token vs 4-token seq should differ"
    );
}

#[test]
fn li_v9_diff_single_vs_recursive() {
    let mut g1 = GrammarBuilder::new("li_v9_7b1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut g2 = GrammarBuilder::new("li_v9_7b2")
        .token("a", "a")
        .token("plus", "+")
        .rule("e", vec!["e", "plus", "a"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_ne!(
        c1.sets.len(),
        c2.sets.len(),
        "single-token vs recursive should differ"
    );
}

#[test]
fn li_v9_diff_seq2_vs_seq4() {
    let mut g1 = GrammarBuilder::new("li_v9_7c1")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let mut g2 = GrammarBuilder::new("li_v9_7c2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d"])
        .start("s")
        .build();
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_ne!(
        c1.sets.len(),
        c2.sets.len(),
        "2-token seq vs 4-token seq should differ"
    );
}

#[test]
fn li_v9_diff_single_alt_vs_many_alt() {
    let mut g1 = GrammarBuilder::new("li_v9_7d1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut g2 = GrammarBuilder::new("li_v9_7d2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .rule("s", vec!["e"])
        .start("s")
        .build();
    let (c1, _) = build_collection(&mut g1);
    let (c2, _) = build_collection(&mut g2);
    assert_ne!(
        c1.sets.len(),
        c2.sets.len(),
        "1 rule vs 5 alternatives should differ"
    );
}

// ===========================================================================
// 8. Precedence / inline / extras → collection builds (8 tests)
// ===========================================================================

#[test]
fn li_v9_precedence_left_builds() {
    let mut g = GrammarBuilder::new("li_v9_8a")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_precedence_right_builds() {
    let mut g = GrammarBuilder::new("li_v9_8b")
        .token("num", "[0-9]+")
        .token("pow", "\\*\\*")
        .rule_with_precedence("e", vec!["e", "pow", "e"], 2, Associativity::Right)
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_precedence_none_builds() {
    let mut g = GrammarBuilder::new("li_v9_8c")
        .token("num", "[0-9]+")
        .token("eq", "==")
        .rule_with_precedence("e", vec!["e", "eq", "e"], 0, Associativity::None)
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_inline_rule_builds() {
    let mut g = GrammarBuilder::new("li_v9_8d")
        .token("x", "x")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .inline("helper")
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_extra_whitespace_builds() {
    let mut g = GrammarBuilder::new("li_v9_8e")
        .token("a", "a")
        .token("ws", "\\s+")
        .extra("ws")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_extra_does_not_reduce_states() {
    let mut g_no_extra = GrammarBuilder::new("li_v9_8f1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col_no, _) = build_collection(&mut g_no_extra);

    let mut g_extra = GrammarBuilder::new("li_v9_8f2")
        .token("a", "a")
        .token("ws", "\\s+")
        .extra("ws")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col_ex, _) = build_collection(&mut g_extra);

    assert!(
        col_ex.sets.len() >= col_no.sets.len(),
        "extras should not reduce states"
    );
}

#[test]
fn li_v9_precedence_two_levels_builds() {
    let mut g = GrammarBuilder::new("li_v9_8g")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4);
}

#[test]
fn li_v9_inline_preserves_states() {
    let mut g = GrammarBuilder::new("li_v9_8h")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .rule("helper", vec!["y"])
        .inline("helper")
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2);
}

// ===========================================================================
// 9. Arithmetic grammar → reasonable state count (4 tests)
// ===========================================================================

#[test]
fn li_v9_arith_simple_builds() {
    let mut g = GrammarBuilder::new("li_v9_9a")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["expr", "plus", "num"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4, "arithmetic grammar needs states");
}

#[test]
fn li_v9_arith_add_mul() {
    let mut g = GrammarBuilder::new("li_v9_9b")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 5);
}

#[test]
fn li_v9_arith_parens() {
    let mut g = GrammarBuilder::new("li_v9_9c")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 6);
}

#[test]
fn li_v9_arith_state_count_le_30() {
    let mut g = GrammarBuilder::new("li_v9_9d")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["lparen", "expr", "rparen"])
        .rule("factor", vec!["num"])
        .start("expr")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() <= 30,
        "classic arith grammar should be <= 30 states, got {}",
        col.sets.len()
    );
}

// ===========================================================================
// 10. FirstFollowSets compute doesn't panic (8 tests)
// ===========================================================================

#[test]
fn li_v9_ff_single_token() {
    let mut g = GrammarBuilder::new("li_v9_10a")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn li_v9_ff_sequence() {
    let mut g = GrammarBuilder::new("li_v9_10b")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn li_v9_ff_alternatives() {
    let mut g = GrammarBuilder::new("li_v9_10c")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn li_v9_ff_chain() {
    let mut g = GrammarBuilder::new("li_v9_10d")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn li_v9_ff_recursive() {
    let mut g = GrammarBuilder::new("li_v9_10e")
        .token("a", "a")
        .token("plus", "+")
        .rule("e", vec!["e", "plus", "a"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn li_v9_ff_nested() {
    let mut g = GrammarBuilder::new("li_v9_10f")
        .token("x", "x")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn li_v9_ff_four_alt() {
    let mut g = GrammarBuilder::new("li_v9_10g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn li_v9_ff_arith() {
    let mut g = GrammarBuilder::new("li_v9_10h")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("e", vec!["e", "plus", "e"])
        .rule("e", vec!["e", "star", "e"])
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

// ===========================================================================
// 11. Large grammar (10+ tokens) → collection builds (4 tests)
// ===========================================================================

#[test]
fn li_v9_large_10_alternatives() {
    let mut g = GrammarBuilder::new("li_v9_11a")
        .token("t0", "t0")
        .token("t1", "t1")
        .token("t2", "t2")
        .token("t3", "t3")
        .token("t4", "t4")
        .token("t5", "t5")
        .token("t6", "t6")
        .token("t7", "t7")
        .token("t8", "t8")
        .token("t9", "t9")
        .rule("s", vec!["t0"])
        .rule("s", vec!["t1"])
        .rule("s", vec!["t2"])
        .rule("s", vec!["t3"])
        .rule("s", vec!["t4"])
        .rule("s", vec!["t5"])
        .rule("s", vec!["t6"])
        .rule("s", vec!["t7"])
        .rule("s", vec!["t8"])
        .rule("s", vec!["t9"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 10);
}

#[test]
fn li_v9_large_10_seq() {
    let mut g = GrammarBuilder::new("li_v9_11b")
        .token("t0", "t0")
        .token("t1", "t1")
        .token("t2", "t2")
        .token("t3", "t3")
        .token("t4", "t4")
        .token("t5", "t5")
        .token("t6", "t6")
        .token("t7", "t7")
        .token("t8", "t8")
        .token("t9", "t9")
        .rule(
            "s",
            vec!["t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8", "t9"],
        )
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 10);
}

#[test]
fn li_v9_large_multi_nonterminal() {
    let mut g = GrammarBuilder::new("li_v9_11c")
        .token("t0", "t0")
        .token("t1", "t1")
        .token("t2", "t2")
        .token("t3", "t3")
        .token("t4", "t4")
        .token("t5", "t5")
        .token("t6", "t6")
        .token("t7", "t7")
        .token("t8", "t8")
        .token("t9", "t9")
        .rule("s", vec!["a", "b"])
        .rule("a", vec!["t0", "t1", "t2", "t3", "t4"])
        .rule("b", vec!["t5", "t6", "t7", "t8", "t9"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 10);
}

#[test]
fn li_v9_large_mixed_operators() {
    let mut g = GrammarBuilder::new("li_v9_11d")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("minus", "\\-")
        .token("star", "\\*")
        .token("slash", "\\/")
        .token("modulo", "%%")
        .token("pow", "\\^")
        .token("amp", "&&")
        .token("pipe", "\\|\\|")
        .token("tilde", "~~")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .rule("e", vec!["e", "plus", "e"])
        .rule("e", vec!["e", "minus", "e"])
        .rule("e", vec!["e", "star", "e"])
        .rule("e", vec!["e", "slash", "e"])
        .rule("e", vec!["lparen", "e", "rparen"])
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 8);
}

// ===========================================================================
// 12. Grammars with conflicts → collection builds (4 tests)
// ===========================================================================

#[test]
fn li_v9_conflict_dangling_else() {
    let mut g = GrammarBuilder::new("li_v9_12a")
        .token("if_kw", "if")
        .token("else_kw", "else")
        .token("expr", "e")
        .token("stmt", "s")
        .rule("s", vec!["if_kw", "expr", "s"])
        .rule("s", vec!["if_kw", "expr", "s", "else_kw", "s"])
        .rule("s", vec!["stmt"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_conflict_ambiguous_binary() {
    let mut g = GrammarBuilder::new("li_v9_12b")
        .token("num", "[0-9]+")
        .token("op", "\\+")
        .rule("e", vec!["e", "op", "e"])
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4);
}

#[test]
fn li_v9_conflict_multiple_reduce() {
    let mut g = GrammarBuilder::new("li_v9_12c")
        .token("a", "a")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .rule("x", vec!["a"])
        .rule("y", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn li_v9_conflict_sr_resolved_by_prec() {
    let mut g = GrammarBuilder::new("li_v9_12d")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 5);
}

// ===========================================================================
// 13. ParseTable from collection is consistent (4 tests)
// ===========================================================================

#[test]
fn li_v9_table_state_count_positive() {
    let g = GrammarBuilder::new("li_v9_13a")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

#[test]
fn li_v9_table_has_initial_state() {
    let g = GrammarBuilder::new("li_v9_13b")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert_eq!(table.initial_state, StateId(0));
}

#[test]
fn li_v9_table_eof_symbol_present() {
    let g = GrammarBuilder::new("li_v9_13c")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    // EOF symbol should be indexable
    assert!(table.symbol_to_index.contains_key(&eof));
}

#[test]
fn li_v9_table_rules_non_empty() {
    let g = GrammarBuilder::new("li_v9_13d")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!table.rules.is_empty(), "table must have parse rules");
}
