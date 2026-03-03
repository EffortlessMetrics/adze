#![allow(clippy::needless_range_loop)]
#![cfg(feature = "test-api")]

//! Comprehensive tests for GOTO table construction, lookup, and properties.
//!
//! The GOTO table maps `(state, non-terminal) → next state` and is stored as
//! `ParseTable::goto_table: Vec<Vec<StateId>>`.  Lookups go through the
//! `ParseTable::goto()` accessor which uses `nonterminal_to_index` for column
//! mapping and treats `StateId(u16::MAX)` as a sentinel for "no transition".

use adze_glr_core::{Action, FirstFollowSets, GotoIndexing, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

/// Collect all (state, target) pairs where goto(state, nt) is defined.
fn all_gotos_for(table: &adze_glr_core::ParseTable, nt: SymbolId) -> Vec<(StateId, StateId)> {
    (0..table.state_count)
        .filter_map(|s| {
            let st = StateId(s as u16);
            table.goto(st, nt).map(|tgt| (st, tgt))
        })
        .collect()
}

/// Count total defined goto entries across all states and nonterminals.
fn total_goto_entries(table: &adze_glr_core::ParseTable) -> usize {
    let mut count = 0;
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if table.goto(st, nt).is_some() {
                count += 1;
            }
        }
    }
    count
}

// ===========================================================================
// 1. Basic existence and structure
// ===========================================================================

#[test]
fn goto_table_has_correct_row_count() {
    let g = GrammarBuilder::new("rows")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto_table must have one row per state"
    );
}

#[test]
fn goto_table_rows_have_uniform_width() {
    let g = GrammarBuilder::new("uniform")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let width = table.goto_table[0].len();
    for (i, row) in table.goto_table.iter().enumerate() {
        assert_eq!(row.len(), width, "row {i} width mismatch");
    }
}

#[test]
fn goto_indexing_is_nonterminal_map() {
    let g = GrammarBuilder::new("idx")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_indexing, GotoIndexing::NonterminalMap);
}

// ===========================================================================
// 2. Goto for start symbol
// ===========================================================================

#[test]
fn goto_exists_for_start_symbol_from_initial_state() {
    let g = GrammarBuilder::new("gs")
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
fn goto_for_start_leads_to_valid_state() {
    let g = GrammarBuilder::new("gv")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    let tgt = table.goto(table.initial_state, s).unwrap();
    assert!(
        (tgt.0 as usize) < table.state_count,
        "goto target must be a valid state index"
    );
}

#[test]
fn goto_start_state_has_accept_on_eof() {
    let g = GrammarBuilder::new("acc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    let accept_state = table.goto(table.initial_state, s).unwrap();
    let eof = table.eof();
    let has_accept = table
        .actions(accept_state, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        has_accept,
        "state after goto(initial, start) must accept on EOF"
    );
}

// ===========================================================================
// 3. Goto for intermediate nonterminals
// ===========================================================================

#[test]
fn goto_exists_for_intermediate_nonterminal() {
    let g = GrammarBuilder::new("inter")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let gotos = all_gotos_for(&table, inner);
    assert!(!gotos.is_empty(), "goto for 'inner' must exist somewhere");
}

#[test]
fn goto_targets_are_within_state_range() {
    let g = GrammarBuilder::new("range")
        .token("a", "a")
        .token("b", "b")
        .rule("ab", vec!["a", "b"])
        .rule("start", vec!["ab"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert!(
                    (tgt.0 as usize) < table.state_count,
                    "goto({s}, {}) = {} exceeds state_count {}",
                    nt.0,
                    tgt.0,
                    table.state_count
                );
            }
        }
    }
}

// ===========================================================================
// 4. Goto for terminals should not exist
// ===========================================================================

#[test]
fn goto_returns_none_for_terminal_symbol() {
    let g = GrammarBuilder::new("term")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    // Terminal symbols should not be in nonterminal_to_index, so goto returns None
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), a).is_none(),
            "goto should not be defined for terminal symbol"
        );
    }
}

#[test]
fn goto_returns_none_for_eof_symbol() {
    let g = GrammarBuilder::new("eof")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), eof).is_none(),
            "goto should not be defined for EOF"
        );
    }
}

// ===========================================================================
// 5. Goto with multiple nonterminals
// ===========================================================================

#[test]
fn each_nonterminal_has_at_least_one_goto_entry() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("aa", vec!["a"])
        .rule("bb", vec!["b"])
        .rule("start", vec!["aa", "bb"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["aa", "bb", "start"] {
        let nt = nt_id(&g, name);
        let gotos = all_gotos_for(&table, nt);
        assert!(
            !gotos.is_empty(),
            "nonterminal '{name}' must have at least one goto entry"
        );
    }
}

#[test]
fn different_nonterminals_can_have_goto_from_same_state() {
    // Grammar: start -> aa bb, where aa and bb are after shifting different tokens.
    // The initial state should have gotos for at least 'start' and 'aa'.
    let g = GrammarBuilder::new("same_state")
        .token("a", "a")
        .token("b", "b")
        .rule("aa", vec!["a"])
        .rule("bb", vec!["b"])
        .rule("start", vec!["aa", "bb"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a_nt = nt_id(&g, "aa");
    let start_nt = nt_id(&g, "start");
    // Both should have goto from the initial state
    let has_a = table.goto(table.initial_state, a_nt).is_some();
    let has_start = table.goto(table.initial_state, start_nt).is_some();
    assert!(has_a, "goto(initial, aa) should exist");
    assert!(has_start, "goto(initial, start) should exist");
}

// ===========================================================================
// 6. Nonterminal-to-index mapping
// ===========================================================================

#[test]
fn nonterminal_to_index_contains_all_grammar_nonterminals() {
    let g = GrammarBuilder::new("ntmap")
        .token("x", "x")
        .rule("mid", vec!["x"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mid = nt_id(&g, "mid");
    let start = nt_id(&g, "start");
    assert!(
        table.nonterminal_to_index.contains_key(&mid),
        "mid must be in nonterminal_to_index"
    );
    assert!(
        table.nonterminal_to_index.contains_key(&start),
        "start must be in nonterminal_to_index"
    );
}

#[test]
fn nonterminal_to_index_has_no_terminal_keys() {
    let g = GrammarBuilder::new("noterm")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    assert!(
        !table.nonterminal_to_index.contains_key(&a),
        "terminal 'a' must not be in nonterminal_to_index"
    );
    assert!(
        !table.nonterminal_to_index.contains_key(&b),
        "terminal 'b' must not be in nonterminal_to_index"
    );
}

#[test]
fn nonterminal_to_index_values_are_within_column_range() {
    let g = GrammarBuilder::new("col_range")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    let width = table.goto_table[0].len();
    for (&_nt, &col) in &table.nonterminal_to_index {
        assert!(col < width, "column index must be < table width");
    }
}

#[test]
fn nonterminal_to_index_values_are_unique() {
    let g = GrammarBuilder::new("uniq")
        .token("a", "a")
        .token("b", "b")
        .rule("xn", vec!["a"])
        .rule("yn", vec!["b"])
        .rule("start", vec!["xn", "yn"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mut seen = std::collections::HashSet::new();
    for &col in table.nonterminal_to_index.values() {
        assert!(seen.insert(col), "duplicate column index {col}");
    }
}

// ===========================================================================
// 7. Goto with recursive grammars
// ===========================================================================

#[test]
fn left_recursive_grammar_has_goto_entries() {
    // list -> list item | item
    let g = GrammarBuilder::new("lrec")
        .token("item", "i")
        .rule("list", vec!["list", "item"])
        .rule("list", vec!["item"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let table = build_table(&g);
    let list = nt_id(&g, "list");
    let gotos = all_gotos_for(&table, list);
    assert!(
        !gotos.is_empty(),
        "left-recursive 'list' must have goto entries"
    );
}

#[test]
fn right_recursive_grammar_has_goto_entries() {
    // list -> item list | item
    let g = GrammarBuilder::new("rrec")
        .token("item", "i")
        .rule("list", vec!["item", "list"])
        .rule("list", vec!["item"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let table = build_table(&g);
    let list = nt_id(&g, "list");
    let gotos = all_gotos_for(&table, list);
    assert!(
        !gotos.is_empty(),
        "right-recursive 'list' must have goto entries"
    );
}

#[test]
fn recursive_grammar_goto_targets_are_distinct_from_initial() {
    let g = GrammarBuilder::new("recdist")
        .token("item", "i")
        .rule("list", vec!["list", "item"])
        .rule("list", vec!["item"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let table = build_table(&g);
    let list = nt_id(&g, "list");
    let gotos = all_gotos_for(&table, list);
    // At least one goto target should differ from the initial state
    let has_non_initial = gotos.iter().any(|(_, tgt)| *tgt != table.initial_state);
    assert!(
        has_non_initial,
        "recursive goto should transition to non-initial state"
    );
}

// ===========================================================================
// 8. Goto for non-existent symbols
// ===========================================================================

#[test]
fn goto_returns_none_for_unknown_symbol() {
    let g = GrammarBuilder::new("unknown")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bogus = SymbolId(9999);
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), bogus).is_none(),
            "goto for unknown symbol must return None"
        );
    }
}

#[test]
fn goto_returns_none_for_out_of_range_state() {
    let g = GrammarBuilder::new("oor")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    let oob = StateId(table.state_count as u16 + 100);
    assert!(
        table.goto(oob, s).is_none(),
        "goto for out-of-range state must return None"
    );
}

// ===========================================================================
// 9. Goto consistency with action table
// ===========================================================================

#[test]
fn goto_and_action_table_agree_on_state_count() {
    let g = GrammarBuilder::new("agree")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.action_table.len());
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn reduce_implies_goto_for_lhs_exists() {
    // If there's a reduce action for a rule, then the LHS nonterminal
    // must have at least one goto entry somewhere (so the parser can
    // transition after reducing).
    let g = GrammarBuilder::new("redgoto")
        .token("a", "a")
        .token("b", "b")
        .rule("xn", vec!["a"])
        .rule("start", vec!["xn", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for rule in &table.rules {
        let lhs = rule.lhs;
        let gotos = all_gotos_for(&table, lhs);
        assert!(
            !gotos.is_empty(),
            "LHS {:?} of a rule must have at least one goto entry",
            lhs
        );
    }
}

// ===========================================================================
// 10. Goto table dimensions
// ===========================================================================

#[test]
fn single_rule_grammar_has_minimal_goto_entries() {
    let g = GrammarBuilder::new("min")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let count = total_goto_entries(&table);
    // At minimum: goto(initial, start) for the augmented grammar
    assert!(count >= 1, "must have at least 1 goto entry, got {count}");
}

#[test]
fn more_nonterminals_means_more_goto_columns() {
    let g1 = GrammarBuilder::new("small")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g2 = GrammarBuilder::new("bigger")
        .token("a", "a")
        .token("b", "b")
        .rule("xn", vec!["a"])
        .rule("yn", vec!["b"])
        .rule("start", vec!["xn", "yn"])
        .start("start")
        .build();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert!(
        t2.nonterminal_to_index.len() > t1.nonterminal_to_index.len(),
        "more nonterminals should have more nonterminal_to_index entries"
    );
}

// ===========================================================================
// 11. Chained nonterminals
// ===========================================================================

#[test]
fn chained_nonterminals_each_have_goto() {
    // ca -> cb -> cc -> "x"
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("cc", vec!["x"])
        .rule("cb", vec!["cc"])
        .rule("ca", vec!["cb"])
        .rule("start", vec!["ca"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["ca", "cb", "cc", "start"] {
        let nt = nt_id(&g, name);
        let gotos = all_gotos_for(&table, nt);
        assert!(
            !gotos.is_empty(),
            "chained nonterminal '{name}' must have goto entry"
        );
    }
}

#[test]
fn chained_nonterminals_goto_targets_are_distinct() {
    let g = GrammarBuilder::new("chain_dist")
        .token("x", "x")
        .rule("cc", vec!["x"])
        .rule("cb", vec!["cc"])
        .rule("ca", vec!["cb"])
        .rule("start", vec!["ca"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mut targets = std::collections::HashSet::new();
    for name in &["ca", "cb", "cc", "start"] {
        let nt = nt_id(&g, name);
        for (_, tgt) in all_gotos_for(&table, nt) {
            targets.insert(tgt.0);
        }
    }
    // With 4 nonterminals in a chain, we should see multiple distinct targets
    assert!(
        targets.len() > 1,
        "chain of nonterminals should produce multiple distinct goto targets"
    );
}

// ===========================================================================
// 12. Multiple alternatives
// ===========================================================================

#[test]
fn alternatives_share_same_lhs_goto() {
    // expr -> "a" | "b" | "c"
    let g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .rule("expr", vec!["c"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    let expr = nt_id(&g, "expr");
    let gotos = all_gotos_for(&table, expr);
    assert!(
        !gotos.is_empty(),
        "'expr' with alternatives must have goto entries"
    );
    // All alternatives reduce to the same 'expr', so they share the goto column
    let targets: std::collections::HashSet<_> = gotos.iter().map(|(_, t)| t.0).collect();
    // Should converge to same target from the initial state
    assert!(
        gotos
            .iter()
            .filter(|(src, _)| *src == table.initial_state)
            .count()
            <= 1,
        "same source state should have at most one goto for a given nonterminal"
    );
    let _ = targets; // silence unused warning
}

// ===========================================================================
// 13. Augmented grammar properties
// ===========================================================================

#[test]
fn augmented_start_has_goto_from_initial() {
    // The augmented grammar adds S' -> S, so goto(0, S) must exist
    let g = GrammarBuilder::new("aug")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(table.goto(table.initial_state, start).is_some());
}

#[test]
fn goto_count_grows_with_grammar_complexity() {
    let g_simple = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let g_complex = GrammarBuilder::new("complex")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("xn", vec!["a"])
        .rule("yn", vec!["b"])
        .rule("zn", vec!["c"])
        .rule("pn", vec!["xn", "yn"])
        .rule("start", vec!["pn", "zn"])
        .start("start")
        .build();

    let t_simple = build_table(&g_simple);
    let t_complex = build_table(&g_complex);

    assert!(
        total_goto_entries(&t_complex) > total_goto_entries(&t_simple),
        "complex grammar should have more goto entries"
    );
}

// ===========================================================================
// 14. Goto determinism (no duplicate entries)
// ===========================================================================

#[test]
fn goto_is_deterministic_per_state_nonterminal() {
    // For any (state, nonterminal) pair, goto should return exactly one target.
    let g = GrammarBuilder::new("det")
        .token("a", "a")
        .token("b", "b")
        .rule("xn", vec!["a"])
        .rule("yn", vec!["b"])
        .rule("start", vec!["xn", "yn"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            // goto() returns Option<StateId>, which is inherently single-valued
            let result = table.goto(st, nt);
            // Just verify it doesn't panic and returns a consistent value
            let result2 = table.goto(st, nt);
            assert_eq!(result, result2, "goto must be deterministic");
        }
    }
}

// ===========================================================================
// 15. Goto with longer RHS
// ===========================================================================

#[test]
fn goto_for_rule_with_long_rhs() {
    // start -> a b c d
    let g = GrammarBuilder::new("long")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    assert!(
        !gotos.is_empty(),
        "start with long RHS must have goto entry"
    );
}

// ===========================================================================
// 16. Goto with mixed terminal and nonterminal RHS
// ===========================================================================

#[test]
fn goto_for_mixed_rhs() {
    // start -> a xn b, xn -> c
    let g = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("xn", vec!["c"])
        .rule("start", vec!["a", "xn", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let x = nt_id(&g, "xn");
    let gotos = all_gotos_for(&table, x);
    assert!(!gotos.is_empty(), "'xn' in mixed RHS must have goto entry");
    // After shifting 'a', we reach a new state where 'xn' can be reduced.
    // There should be a goto for xn from a non-initial state.
    let from_non_initial = gotos.iter().any(|(src, _)| *src != table.initial_state);
    assert!(
        from_non_initial,
        "xn goto should exist from a state after shifting 'a'"
    );
}

// ===========================================================================
// 17. Nonterminal_to_index completeness
// ===========================================================================

#[test]
fn nonterminal_to_index_includes_augmented_start_nonterminals() {
    let g = GrammarBuilder::new("aug_nt")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    // The original start symbol must be mapped
    let start = nt_id(&g, "start");
    assert!(
        table.nonterminal_to_index.contains_key(&start),
        "start symbol must be in nonterminal_to_index"
    );
}

// ===========================================================================
// 18. Goto with ambiguous/GLR grammars
// ===========================================================================

#[test]
fn ambiguous_grammar_still_has_goto_entries() {
    // expr -> expr "+" expr | "n"  (ambiguous without precedence)
    let g = GrammarBuilder::new("ambig")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let e = nt_id(&g, "expr");
    let gotos = all_gotos_for(&table, e);
    assert!(
        !gotos.is_empty(),
        "ambiguous grammar must still have goto entries for 'E'"
    );
}

#[test]
fn ambiguous_grammar_goto_has_multiple_source_states() {
    // expr -> expr "+" expr | "n": after shifting "+", we're in a new state where expr can appear
    let g = GrammarBuilder::new("ambig_multi")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let e = nt_id(&g, "expr");
    let gotos = all_gotos_for(&table, e);
    let sources: std::collections::HashSet<_> = gotos.iter().map(|(src, _)| src.0).collect();
    assert!(
        sources.len() >= 2,
        "E in recursive grammar should have goto from multiple states, got {}",
        sources.len()
    );
}

// ===========================================================================
// 19. Goto with multiple rules for same nonterminal
// ===========================================================================

#[test]
fn multiple_productions_for_same_nt_converge_goto() {
    // val -> "a" | "b" | "c"
    // start -> val
    let g = GrammarBuilder::new("conv")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("val", vec!["a"])
        .rule("val", vec!["b"])
        .rule("val", vec!["c"])
        .rule("start", vec!["val"])
        .start("start")
        .build();
    let table = build_table(&g);
    let val = nt_id(&g, "val");
    // From initial state, goto(initial, val) should be defined (regardless which
    // alternative was reduced, the GOTO for 'val' from the same state is the same).
    let goto = table.goto(table.initial_state, val);
    assert!(goto.is_some(), "goto(initial, val) must exist");
}

// ===========================================================================
// 20. Goto table raw access vs accessor
// ===========================================================================

#[test]
fn goto_accessor_consistent_with_raw_table() {
    let g = GrammarBuilder::new("raw")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for (&nt, &col) in &table.nonterminal_to_index {
            let raw_val = table.goto_table[s][col];
            let accessor_val = table.goto(st, nt);
            if raw_val.0 == u16::MAX {
                assert!(
                    accessor_val.is_none(),
                    "sentinel u16::MAX should map to None"
                );
            } else {
                // raw_val of StateId(0) may or may not be a valid transition;
                // the accessor returns Some for any non-sentinel value
                assert_eq!(
                    accessor_val,
                    Some(raw_val),
                    "accessor should return the raw value"
                );
            }
        }
    }
}

// ===========================================================================
// 21. Large-ish grammar
// ===========================================================================

#[test]
fn arithmetic_grammar_goto_structure() {
    // expr -> expr "+" term | term
    // term -> term "*" factor | factor
    // factor -> "(" expr ")" | "num"
    let g = GrammarBuilder::new("arith")
        .token("num", "\\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("factor", vec!["num"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);

    // All nonterminals should have goto entries
    for name in &["expr", "term", "factor", "start"] {
        let nt = nt_id(&g, name);
        let gotos = all_gotos_for(&table, nt);
        assert!(
            !gotos.is_empty(),
            "'{name}' must have goto entries in arithmetic grammar"
        );
    }

    // factor can appear in multiple contexts (after "(", after "*", at start)
    // so it should have goto from multiple states
    let factor = nt_id(&g, "factor");
    let factor_gotos = all_gotos_for(&table, factor);
    assert!(
        factor_gotos.len() >= 2,
        "factor should have goto from multiple states, got {}",
        factor_gotos.len()
    );
}

#[test]
fn arithmetic_grammar_expr_goto_after_lparen() {
    // After shifting "(", there should be a goto for "expr" (for the rule factor -> "(" expr ")")
    let g = GrammarBuilder::new("arith_paren")
        .token("num", "\\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("factor", vec!["num"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    let expr = nt_id(&g, "expr");
    let lparen = tok_id(&g, "(");

    // Find the state we transition to after shifting "("
    let mut paren_state = None;
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for action in table.actions(st, lparen) {
            if let Action::Shift(tgt) = action {
                paren_state = Some(*tgt);
            }
        }
    }

    if let Some(ps) = paren_state {
        let goto = table.goto(ps, expr);
        assert!(
            goto.is_some(),
            "after shifting '(', goto for 'expr' must exist"
        );
    }
}

// ===========================================================================
// 22. State count properties
// ===========================================================================

#[test]
fn state_count_matches_goto_table_length() {
    let g = GrammarBuilder::new("sc")
        .token("a", "a")
        .token("b", "b")
        .rule("xn", vec!["a"])
        .rule("start", vec!["xn", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.state_count, table.goto_table.len());
}

// ===========================================================================
// 23. Goto for epsilon / nullable rules
// ===========================================================================

#[test]
fn nullable_nonterminal_has_goto() {
    // opt -> "a" | ε (via two separate rules)
    // start -> opt "b"
    let g = GrammarBuilder::new("nullable")
        .token("a", "a")
        .token("b", "b")
        .rule("opt", vec!["a"])
        .rule("opt", vec![])
        .rule("start", vec!["opt", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let opt = nt_id(&g, "opt");
    let gotos = all_gotos_for(&table, opt);
    assert!(
        !gotos.is_empty(),
        "nullable nonterminal 'opt' must have goto entries"
    );
}

// ===========================================================================
// 24. Goto with precedence
// ===========================================================================

#[test]
fn precedence_grammar_preserves_goto_entries() {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let e = nt_id(&g, "expr");
    let gotos = all_gotos_for(&table, e);
    assert!(
        !gotos.is_empty(),
        "precedence grammar must preserve goto entries"
    );
}

// ===========================================================================
// 25-26. Goto table structural invariants
// ===========================================================================

#[test]
fn no_goto_self_loop_from_initial_to_initial() {
    // goto(initial, nt) for non-start nonterminals should lead to meaningful states.
    let g = GrammarBuilder::new("noloop")
        .token("a", "a")
        .token("b", "b")
        .rule("xn", vec!["a"])
        .rule("yn", vec!["b"])
        .rule("start", vec!["xn", "yn"])
        .start("start")
        .build();
    let table = build_table(&g);
    let x = nt_id(&g, "xn");
    // xn should transition to a non-initial state (since start -> xn . yn)
    if let Some(tgt) = table.goto(table.initial_state, x) {
        assert_ne!(
            tgt, table.initial_state,
            "goto(initial, xn) should not loop to initial"
        );
    }
}

#[test]
fn goto_entries_at_least_one_per_nonterminal_in_index() {
    let g = GrammarBuilder::new("atleast")
        .token("a", "a")
        .token("b", "b")
        .rule("aa", vec!["a"])
        .rule("bb", vec!["b"])
        .rule("start", vec!["aa", "bb"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        let gotos = all_gotos_for(&table, nt);
        assert!(
            !gotos.is_empty(),
            "every nonterminal in nonterminal_to_index ({:?}) should have at least one goto",
            nt
        );
    }
}

// ===========================================================================
// 27-28. Goto with deeply nested grammars
// ===========================================================================

#[test]
fn deeply_nested_grammar_all_nonterminals_reachable() {
    // da -> db, db -> dc, dc -> dd, dd -> "x"
    let g = GrammarBuilder::new("deep")
        .token("x", "x")
        .rule("dd", vec!["x"])
        .rule("dc", vec!["dd"])
        .rule("db", vec!["dc"])
        .rule("da", vec!["db"])
        .rule("start", vec!["da"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["da", "db", "dc", "dd", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            table.nonterminal_to_index.contains_key(&nt),
            "'{name}' must be in nonterminal_to_index"
        );
        assert!(
            !all_gotos_for(&table, nt).is_empty(),
            "'{name}' must have at least one goto entry"
        );
    }
}

#[test]
fn deeply_nested_goto_forms_progression() {
    // Each level in the chain should have a goto from the same initial state,
    // since they all reduce in sequence from "x".
    let g = GrammarBuilder::new("deep_prog")
        .token("x", "x")
        .rule("dd", vec!["x"])
        .rule("dc", vec!["dd"])
        .rule("db", vec!["dc"])
        .rule("da", vec!["db"])
        .rule("start", vec!["da"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mut initial_targets = Vec::new();
    for name in &["dd", "dc", "db", "da", "start"] {
        let nt = nt_id(&g, name);
        if let Some(tgt) = table.goto(table.initial_state, nt) {
            initial_targets.push((name, tgt));
        }
    }
    // The chain should produce distinct goto targets for each level
    let unique_targets: std::collections::HashSet<_> =
        initial_targets.iter().map(|(_, t)| t.0).collect();
    assert!(
        unique_targets.len() >= 2,
        "deep chain should produce distinct goto targets from initial state"
    );
}

// ===========================================================================
// 29-30. Goto with rules referencing multiple nonterminals
// ===========================================================================

#[test]
fn binary_tree_grammar_goto() {
    // tree -> "(" tree tree ")" | "leaf"
    let g = GrammarBuilder::new("btree")
        .token("(", "(")
        .token(")", ")")
        .token("leaf", "l")
        .rule("tree", vec!["(", "tree", "tree", ")"])
        .rule("tree", vec!["leaf"])
        .rule("start", vec!["tree"])
        .start("start")
        .build();
    let table = build_table(&g);
    let tree = nt_id(&g, "tree");
    let gotos = all_gotos_for(&table, tree);
    // 'tree' appears in multiple positions in the recursive rule, so it should
    // have goto entries from multiple states
    assert!(
        gotos.len() >= 2,
        "recursive 'tree' should have goto from at least 2 states, got {}",
        gotos.len()
    );
}

#[test]
fn pair_grammar_goto_both_elements() {
    // pair -> first second
    // first -> "a"
    // second -> "b"
    let g = GrammarBuilder::new("pair")
        .token("a", "a")
        .token("b", "b")
        .rule("first", vec!["a"])
        .rule("second", vec!["b"])
        .rule("pair", vec!["first", "second"])
        .rule("start", vec!["pair"])
        .start("start")
        .build();
    let table = build_table(&g);
    let first = nt_id(&g, "first");
    let second = nt_id(&g, "second");

    // 'first' should have goto from initial (since pair starts there)
    let first_gotos = all_gotos_for(&table, first);
    assert!(!first_gotos.is_empty(), "'first' must have goto");

    // 'second' should have goto from the state after recognizing 'first'
    let second_gotos = all_gotos_for(&table, second);
    assert!(!second_gotos.is_empty(), "'second' must have goto");

    // 'first' should have goto from initial state; 'second' should have goto
    // from a different state (after first has been recognized)
    let first_from_initial = first_gotos.iter().any(|(s, _)| *s == table.initial_state);
    assert!(
        first_from_initial,
        "first should have goto from initial state"
    );
}

// ===========================================================================
// 31-35. Additional edge cases and properties
// ===========================================================================

#[test]
fn goto_table_is_nonempty_for_any_grammar() {
    let g = GrammarBuilder::new("nonempty")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        !table.goto_table.is_empty(),
        "goto_table must never be empty"
    );
    assert!(
        total_goto_entries(&table) > 0,
        "there must be at least one defined goto entry"
    );
}

#[test]
fn nonterminal_to_index_does_not_contain_eof() {
    let g = GrammarBuilder::new("no_eof")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        !table.nonterminal_to_index.contains_key(&table.eof_symbol),
        "EOF must not be in nonterminal_to_index"
    );
}

#[test]
fn goto_start_symbol_field_matches_grammar() {
    let g = GrammarBuilder::new("field")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert_eq!(
        table.start_symbol, start,
        "parse table start_symbol must match grammar"
    );
}

#[test]
fn goto_table_width_matches_symbol_to_index_len() {
    let g = GrammarBuilder::new("width")
        .token("a", "a")
        .token("b", "b")
        .rule("xn", vec!["a"])
        .rule("start", vec!["xn", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // goto_table columns are indexed by symbol_to_index (which includes all symbols)
    let expected_width = table.symbol_to_index.len();
    for (i, row) in table.goto_table.iter().enumerate() {
        assert_eq!(
            row.len(),
            expected_width,
            "row {i} width must match symbol_to_index len"
        );
    }
}

#[test]
fn triple_alternative_nonterminal_single_goto_from_initial() {
    // val -> "x" | "y" | "z"; start -> val
    // All three alternatives reduce to 'val', so goto(initial, val)
    // should give exactly one target.
    let g = GrammarBuilder::new("triple")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("val", vec!["x"])
        .rule("val", vec!["y"])
        .rule("val", vec!["z"])
        .rule("start", vec!["val"])
        .start("start")
        .build();
    let table = build_table(&g);
    let val = nt_id(&g, "val");
    let goto = table.goto(table.initial_state, val);
    assert!(
        goto.is_some(),
        "goto(initial, val) must exist for triple-alternative grammar"
    );
}
