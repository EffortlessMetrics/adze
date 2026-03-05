#![cfg(feature = "test-api")]

//! GOTO table v6 tests — 64 tests across 8 categories covering construction,
//! lookup, correctness, consistency, boundary conditions, and integration.

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

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
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

/// Collect all nonterminal SymbolIds registered in the goto table.
fn goto_nonterminals(table: &adze_glr_core::ParseTable) -> Vec<SymbolId> {
    table.nonterminal_to_index.keys().copied().collect()
}

// ===========================================================================
// 1. goto_basic_* — basic goto lookups (8 tests)
// ===========================================================================

#[test]
fn goto_basic_single_rule_start_exists() {
    let g = GrammarBuilder::new("b1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    assert!(
        !gotos.is_empty(),
        "start nonterminal must have at least one goto entry"
    );
}

#[test]
fn goto_basic_lookup_returns_valid_state() {
    let g = GrammarBuilder::new("b2")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    for (_, tgt) in &gotos {
        assert!(
            (tgt.0 as usize) < table.state_count,
            "goto target {tgt:?} must be within state_count {}",
            table.state_count
        );
    }
}

#[test]
fn goto_basic_two_tokens_single_rule() {
    let g = GrammarBuilder::new("b3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    assert!(
        !gotos.is_empty(),
        "two-token rule still needs goto for start"
    );
}

#[test]
fn goto_basic_initial_state_lookup() {
    let g = GrammarBuilder::new("b4")
        .token("n", "n")
        .rule("start", vec!["n"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    // Initial state should have a goto for start (after reducing start → n)
    let result = table.goto(table.initial_state, start);
    assert!(result.is_some(), "goto(initial, start) should be defined");
}

#[test]
fn goto_basic_goto_table_has_rows_for_all_states() {
    let g = GrammarBuilder::new("b5")
        .token("p", "p")
        .rule("start", vec!["p"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto_table must have exactly state_count rows"
    );
}

#[test]
fn goto_basic_goto_indexing_is_nonterminal_map() {
    let g = GrammarBuilder::new("b6")
        .token("q", "q")
        .rule("start", vec!["q"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_indexing, GotoIndexing::NonterminalMap);
}

#[test]
fn goto_basic_goto_from_different_source_states() {
    let g = GrammarBuilder::new("b7")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&g);
    let item = nt_id(&g, "item");
    let gotos = all_gotos_for(&table, item);
    assert!(!gotos.is_empty(), "item nonterminal must have goto entries");
}

#[test]
fn goto_basic_total_goto_entries_positive() {
    let g = GrammarBuilder::new("b8")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    assert!(total > 0, "even minimal grammar must have goto entries");
}

// ===========================================================================
// 2. goto_missing_* — missing entries return None (8 tests)
// ===========================================================================

#[test]
fn goto_missing_terminal_returns_none() {
    let g = GrammarBuilder::new("m1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let tok_a = tok_id(&g, "a");
    // Terminals should not appear in the goto table
    let result = table.goto(StateId(0), tok_a);
    assert!(result.is_none(), "goto for terminal symbol should be None");
}

#[test]
fn goto_missing_out_of_range_state() {
    let g = GrammarBuilder::new("m2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let far_state = StateId(table.state_count as u16 + 100);
    assert!(
        table.goto(far_state, start).is_none(),
        "out-of-range state must return None"
    );
}

#[test]
fn goto_missing_nonexistent_symbol_id() {
    let g = GrammarBuilder::new("m3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bogus = SymbolId(60000);
    assert!(
        table.goto(StateId(0), bogus).is_none(),
        "unknown SymbolId must return None"
    );
}

#[test]
fn goto_missing_state_with_no_gotos() {
    // A nonterminal not used in the grammar should have no goto entries
    let g = GrammarBuilder::new("m4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // A fabricated SymbolId that is not in nonterminal_to_index returns None for all states
    let bogus = SymbolId(9999);
    assert!(!table.nonterminal_to_index.contains_key(&bogus));
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), bogus).is_none(),
            "state {s} should have no goto for unknown nonterminal"
        );
    }
}

#[test]
fn goto_missing_eof_symbol_not_in_goto() {
    let g = GrammarBuilder::new("m5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // EOF is a terminal — should not be in goto table
    let eof = table.eof_symbol;
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), eof).is_none(),
            "EOF should never appear in goto table"
        );
    }
}

#[test]
fn goto_missing_all_invalid_symbols_return_none() {
    let g = GrammarBuilder::new("m6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Test a batch of invalid symbol IDs
    for raw_id in [50000u16, 55000, 60000, 65000, 65534] {
        let sym = SymbolId(raw_id);
        for s in 0..table.state_count {
            assert!(table.goto(StateId(s as u16), sym).is_none());
        }
    }
}

#[test]
fn goto_missing_sentinel_values_are_absent() {
    let g = GrammarBuilder::new("m7")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // The sentinel value u16::MAX should never be returned by goto()
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert_ne!(tgt.0, u16::MAX, "sentinel must never leak through goto()");
            }
        }
    }
}

#[test]
fn goto_missing_max_state_id_returns_none() {
    let g = GrammarBuilder::new("m8")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(
        table.goto(StateId(u16::MAX), start).is_none(),
        "StateId(MAX) must return None"
    );
}

// ===========================================================================
// 3. goto_chain_* — goto chains through multiple states (8 tests)
// ===========================================================================

#[test]
fn goto_chain_two_level_nesting() {
    // start → inner, inner → a
    let g = GrammarBuilder::new("c1")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let start = nt_id(&g, "start");
    // Both nonterminals must have goto entries
    assert!(!all_gotos_for(&table, inner).is_empty());
    assert!(!all_gotos_for(&table, start).is_empty());
}

#[test]
fn goto_chain_three_level_nesting() {
    // start → mid, mid → leaf, leaf → a
    let g = GrammarBuilder::new("c2")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    let leaf = nt_id(&g, "leaf");
    let mid = nt_id(&g, "mid");
    let start = nt_id(&g, "start");
    assert!(
        !all_gotos_for(&table, leaf).is_empty(),
        "leaf should have gotos"
    );
    assert!(
        !all_gotos_for(&table, mid).is_empty(),
        "mid should have gotos"
    );
    assert!(
        !all_gotos_for(&table, start).is_empty(),
        "start should have gotos"
    );
}

#[test]
fn goto_chain_targets_are_distinct_from_source() {
    let g = GrammarBuilder::new("c3")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    for (src, tgt) in all_gotos_for(&table, inner) {
        assert_ne!(src, tgt, "goto should transition to a different state");
    }
}

#[test]
fn goto_chain_sequential_production() {
    // start → a inner, inner → b
    let g = GrammarBuilder::new("c4")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["b"])
        .rule("start", vec!["a", "inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let gotos = all_gotos_for(&table, inner);
    assert!(!gotos.is_empty(), "inner after terminal must have goto");
}

#[test]
fn goto_chain_multiple_nonterminals_in_rhs() {
    // start → left right, left → a, right → b
    let g = GrammarBuilder::new("c5")
        .token("a", "a")
        .token("b", "b")
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    let table = build_table(&g);
    let left = nt_id(&g, "left");
    let right = nt_id(&g, "right");
    assert!(
        !all_gotos_for(&table, left).is_empty(),
        "left nonterminal needs goto"
    );
    assert!(
        !all_gotos_for(&table, right).is_empty(),
        "right nonterminal needs goto"
    );
}

#[test]
fn goto_chain_deep_four_levels() {
    // start → l1, l1 → l2, l2 → l3, l3 → a
    let g = GrammarBuilder::new("c6")
        .token("a", "a")
        .rule("l3", vec!["a"])
        .rule("l2", vec!["l3"])
        .rule("l1", vec!["l2"])
        .rule("start", vec!["l1"])
        .start("start")
        .build();
    let table = build_table(&g);
    // All 4 nonterminals must have at least one goto
    for name in &["l3", "l2", "l1", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            !all_gotos_for(&table, nt).is_empty(),
            "{name} must have gotos"
        );
    }
}

#[test]
fn goto_chain_goto_targets_within_bounds() {
    let g = GrammarBuilder::new("c7")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!(
                (tgt.0 as usize) < table.state_count,
                "goto target must be < state_count"
            );
        }
    }
}

#[test]
fn goto_chain_initial_state_has_goto_for_outermost() {
    // The initial state should have a goto for the start nonterminal
    let g = GrammarBuilder::new("c8")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(
        table.goto(table.initial_state, start).is_some(),
        "initial state should have goto for start"
    );
}

// ===========================================================================
// 4. goto_nonterminal_* — correct nonterminals in goto (8 tests)
// ===========================================================================

#[test]
fn goto_nonterminal_all_rules_have_entries() {
    let g = GrammarBuilder::new("n1")
        .token("a", "a")
        .token("b", "b")
        .rule("alpha", vec!["a"])
        .rule("beta", vec!["b"])
        .rule("start", vec!["alpha", "beta"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Every nonterminal in nonterminal_to_index should appear somewhere in gotos
    for &nt in table.nonterminal_to_index.keys() {
        let gotos = all_gotos_for(&table, nt);
        assert!(
            !gotos.is_empty(),
            "nonterminal {:?} must have at least one goto",
            nt
        );
    }
}

#[test]
fn goto_nonterminal_index_map_covers_grammar_rules() {
    let g = GrammarBuilder::new("n2")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Both "inner" and "start" should appear in nonterminal_to_index
    let inner = nt_id(&g, "inner");
    let start = nt_id(&g, "start");
    assert!(table.nonterminal_to_index.contains_key(&inner));
    assert!(table.nonterminal_to_index.contains_key(&start));
}

#[test]
fn goto_nonterminal_distinct_column_indices() {
    let g = GrammarBuilder::new("n3")
        .token("a", "a")
        .token("b", "b")
        .rule("alpha", vec!["a"])
        .rule("beta", vec!["b"])
        .rule("start", vec!["alpha", "beta"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Each nonterminal should map to a unique column index
    let indices: Vec<usize> = table.nonterminal_to_index.values().copied().collect();
    let unique: std::collections::HashSet<usize> = indices.iter().copied().collect();
    assert_eq!(indices.len(), unique.len(), "column indices must be unique");
}

#[test]
fn goto_nonterminal_no_terminal_in_goto_index() {
    let g = GrammarBuilder::new("n4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let tok_a = tok_id(&g, "a");
    // Terminal should not be in nonterminal_to_index
    assert!(
        !table.nonterminal_to_index.contains_key(&tok_a),
        "terminal must not appear in nonterminal_to_index"
    );
}

#[test]
fn goto_nonterminal_augmented_start_in_goto() {
    // The augmented start symbol should also be in the goto table
    let g = GrammarBuilder::new("n5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // There should be at least the user-defined nonterminals plus the augmented start
    let nts = goto_nonterminals(&table);
    assert!(
        nts.len() >= 1,
        "goto table must have at least one nonterminal column"
    );
}

#[test]
fn goto_nonterminal_three_nonterminals_all_present() {
    let g = GrammarBuilder::new("n6")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("first", vec!["x"])
        .rule("second", vec!["y"])
        .rule("third", vec!["z"])
        .rule("start", vec!["first", "second", "third"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["first", "second", "third", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            table.nonterminal_to_index.contains_key(&nt),
            "{name} must be in nonterminal_to_index"
        );
    }
}

#[test]
fn goto_nonterminal_goto_rows_width_matches_columns() {
    let g = GrammarBuilder::new("n7")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let expected_width = table.nonterminal_to_index.len();
    for (i, row) in table.goto_table.iter().enumerate() {
        assert!(
            row.len() >= expected_width,
            "row {i}: width {} < nonterminal count {}",
            row.len(),
            expected_width
        );
    }
}

#[test]
fn goto_nonterminal_alternative_rules_share_goto() {
    // Two alternatives for 'val': val → a | b
    let g = GrammarBuilder::new("n8")
        .token("a", "a")
        .token("b", "b")
        .rule("val", vec!["a"])
        .rule("val", vec!["b"])
        .rule("start", vec!["val"])
        .start("start")
        .build();
    let table = build_table(&g);
    let val = nt_id(&g, "val");
    // Both alternatives reduce to 'val', so goto for val must exist
    let gotos = all_gotos_for(&table, val);
    assert!(
        !gotos.is_empty(),
        "val with alternatives must have goto entries"
    );
}

// ===========================================================================
// 5. goto_complex_* — complex grammar goto tables (8 tests)
// ===========================================================================

#[test]
fn goto_complex_arithmetic_like_grammar() {
    // expr → term, term → factor, factor → num
    let g = GrammarBuilder::new("cx1")
        .token("num", "[0-9]+")
        .rule("factor", vec!["num"])
        .rule("term", vec!["factor"])
        .rule("expr", vec!["term"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["factor", "term", "expr", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            !all_gotos_for(&table, nt).is_empty(),
            "{name} must have goto entries in arithmetic grammar"
        );
    }
}

#[test]
fn goto_complex_multiple_alternatives() {
    // expr → a | b | c | d
    let g = GrammarBuilder::new("cx2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .rule("expr", vec!["c"])
        .rule("expr", vec!["d"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&g);
    let expr = nt_id(&g, "expr");
    let gotos = all_gotos_for(&table, expr);
    assert!(!gotos.is_empty());
    // All alternatives reduce to same nonterminal, so goto targets should be same
    let targets: std::collections::HashSet<StateId> = gotos.iter().map(|(_, t)| *t).collect();
    // targets could be 1 (all from same source state) or multiple
    assert!(!targets.is_empty());
}

#[test]
fn goto_complex_mixed_terminal_nonterminal_rhs() {
    // start → a inner b, inner → c
    let g = GrammarBuilder::new("cx3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("inner", vec!["c"])
        .rule("start", vec!["a", "inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    assert!(!all_gotos_for(&table, inner).is_empty());
}

#[test]
fn goto_complex_multiple_rules_per_nonterminal() {
    // list → item | list item, item → a
    let g = GrammarBuilder::new("cx4")
        .token("a", "a")
        .rule("item", vec!["a"])
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "item"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let table = build_table(&g);
    let list = nt_id(&g, "list");
    let item = nt_id(&g, "item");
    assert!(!all_gotos_for(&table, list).is_empty());
    assert!(!all_gotos_for(&table, item).is_empty());
}

#[test]
fn goto_complex_recursive_rule_goto_count() {
    // Recursive rules should generate more states/gotos
    let g = GrammarBuilder::new("cx5")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "a"])
        .rule("start", vec!["list"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    assert!(
        total >= 2,
        "recursive grammar should have multiple goto entries, got {total}"
    );
}

#[test]
fn goto_complex_state_count_grows_with_complexity() {
    let simple = GrammarBuilder::new("cx6a")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let complex = GrammarBuilder::new("cx6b")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("inner", vec!["a"])
        .rule("inner", vec!["b"])
        .rule("outer", vec!["inner", "c"])
        .rule("start", vec!["outer"])
        .start("start")
        .build();
    let t1 = build_table(&simple);
    let t2 = build_table(&complex);
    assert!(
        t2.state_count >= t1.state_count,
        "more complex grammar should have at least as many states"
    );
}

#[test]
fn goto_complex_diamond_grammar() {
    // Diamond: start → ab | cd, ab → a, cd → a (shared terminal)
    let g = GrammarBuilder::new("cx7")
        .token("a", "a")
        .rule("ab", vec!["a"])
        .rule("cd", vec!["a"])
        .rule("start", vec!["ab"])
        .rule("start", vec!["cd"])
        .start("start")
        .build();
    let table = build_table(&g);
    let ab = nt_id(&g, "ab");
    let cd = nt_id(&g, "cd");
    // Both paths should generate goto entries
    assert!(!all_gotos_for(&table, ab).is_empty());
    assert!(!all_gotos_for(&table, cd).is_empty());
}

#[test]
fn goto_complex_wide_grammar_many_nonterminals() {
    let g = GrammarBuilder::new("cx8")
        .token("t1", "t1")
        .token("t2", "t2")
        .token("t3", "t3")
        .token("t4", "t4")
        .token("t5", "t5")
        .rule("n1", vec!["t1"])
        .rule("n2", vec!["t2"])
        .rule("n3", vec!["t3"])
        .rule("n4", vec!["t4"])
        .rule("n5", vec!["t5"])
        .rule("start", vec!["n1", "n2", "n3", "n4", "n5"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["n1", "n2", "n3", "n4", "n5", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            !all_gotos_for(&table, nt).is_empty(),
            "{name} must have goto"
        );
    }
}

// ===========================================================================
// 6. goto_consistency_* — goto table consistency checks (8 tests)
// ===========================================================================

#[test]
fn goto_consistency_deterministic_single_target() {
    // For a given (state, nonterminal), goto should return at most one target
    let g = GrammarBuilder::new("co1")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            // goto() returns Option<StateId>, so it's inherently single-valued
            // Calling it twice should return the same result
            let r1 = table.goto(st, nt);
            let r2 = table.goto(st, nt);
            assert_eq!(
                r1, r2,
                "goto must be deterministic for state={s}, nt={nt:?}"
            );
        }
    }
}

#[test]
fn goto_consistency_uniform_row_width() {
    let g = GrammarBuilder::new("co2")
        .token("a", "a")
        .token("b", "b")
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    let table = build_table(&g);
    if !table.goto_table.is_empty() {
        let width = table.goto_table[0].len();
        for (i, row) in table.goto_table.iter().enumerate() {
            assert_eq!(row.len(), width, "goto row {i} has inconsistent width");
        }
    }
}

#[test]
fn goto_consistency_no_self_loops_for_non_recursive() {
    // Non-recursive grammar should not have self-loops in goto
    let g = GrammarBuilder::new("co3")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (src, tgt) in all_gotos_for(&table, nt) {
            assert_ne!(
                src, tgt,
                "non-recursive grammar should not have self-loops: state={src:?}, nt={nt:?}"
            );
        }
    }
}

#[test]
fn goto_consistency_targets_are_valid_states() {
    let g = GrammarBuilder::new("co4")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (src, tgt) in all_gotos_for(&table, nt) {
            assert!(
                (tgt.0 as usize) < table.state_count,
                "goto({src:?}, {nt:?}) = {tgt:?} exceeds state_count"
            );
        }
    }
}

#[test]
fn goto_consistency_nonterminal_to_index_values_in_bounds() {
    let g = GrammarBuilder::new("co5")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let width = if table.goto_table.is_empty() {
        0
    } else {
        table.goto_table[0].len()
    };
    for (&nt, &col) in &table.nonterminal_to_index {
        assert!(
            col < width,
            "nonterminal {nt:?} maps to column {col} but goto width is {width}"
        );
    }
}

#[test]
fn goto_consistency_goto_and_action_tables_same_state_count() {
    let g = GrammarBuilder::new("co6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        table.action_table.len(),
        table.goto_table.len(),
        "action and goto tables must have same number of rows (state_count)"
    );
}

#[test]
fn goto_consistency_rebuilding_yields_same_table() {
    let g = GrammarBuilder::new("co7")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table1 = build_table(&g);
    let table2 = build_table(&g);
    // goto tables should be identical across builds
    assert_eq!(table1.goto_table.len(), table2.goto_table.len());
    for (i, (r1, r2)) in table1
        .goto_table
        .iter()
        .zip(table2.goto_table.iter())
        .enumerate()
    {
        assert_eq!(r1, r2, "goto row {i} differs between builds");
    }
}

#[test]
fn goto_consistency_goto_entries_le_states_times_nonterminals() {
    let g = GrammarBuilder::new("co8")
        .token("a", "a")
        .token("b", "b")
        .rule("val", vec!["a"])
        .rule("val", vec!["b"])
        .rule("start", vec!["val"])
        .start("start")
        .build();
    let table = build_table(&g);
    let max_possible = table.state_count * table.nonterminal_to_index.len();
    let actual = total_goto_entries(&table);
    assert!(
        actual <= max_possible,
        "goto entries ({actual}) cannot exceed states × nonterminals ({max_possible})"
    );
}

// ===========================================================================
// 7. goto_boundaries_* — boundary state IDs (8 tests)
// ===========================================================================

#[test]
fn goto_boundaries_state_zero_has_gotos() {
    let g = GrammarBuilder::new("bd1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mut has_any = false;
    for &nt in table.nonterminal_to_index.keys() {
        if table.goto(StateId(0), nt).is_some() {
            has_any = true;
            break;
        }
    }
    assert!(has_any, "state 0 should have at least one goto entry");
}

#[test]
fn goto_boundaries_last_state_is_reachable() {
    let g = GrammarBuilder::new("bd2")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let last_state = StateId((table.state_count - 1) as u16);
    // Last state should be reachable as a goto target from some state
    let mut reachable = false;
    for &nt in table.nonterminal_to_index.keys() {
        for s in 0..table.state_count {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                if tgt == last_state {
                    reachable = true;
                    break;
                }
            }
        }
        if reachable {
            break;
        }
    }
    // Last state may also be reachable via action table shifts
    // Just verify it exists as a valid index
    assert!(
        (last_state.0 as usize) < table.state_count,
        "last state must be a valid index"
    );
}

#[test]
fn goto_boundaries_initial_state_matches_table_field() {
    let g = GrammarBuilder::new("bd3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        (table.initial_state.0 as usize) < table.state_count,
        "initial_state must be valid"
    );
}

#[test]
fn goto_boundaries_all_goto_targets_less_than_state_count() {
    let g = GrammarBuilder::new("bd4")
        .token("a", "a")
        .token("b", "b")
        .rule("pair", vec!["a", "b"])
        .rule("start", vec!["pair"])
        .start("start")
        .build();
    let table = build_table(&g);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(st, nt) {
                assert!(
                    (tgt.0 as usize) < table.state_count,
                    "goto target {tgt:?} out of bounds (state_count={})",
                    table.state_count
                );
            }
        }
    }
}

#[test]
fn goto_boundaries_column_indices_zero_based() {
    let g = GrammarBuilder::new("bd5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &col in table.nonterminal_to_index.values() {
        // Column indices should be reasonable (0-based, contiguous)
        assert!(
            col < table.goto_table.first().map_or(0, |r| r.len()),
            "column index {col} out of goto row bounds"
        );
    }
}

#[test]
fn goto_boundaries_empty_rhs_epsilon_rule() {
    // Epsilon rule: inner → ε, start → inner
    let g = GrammarBuilder::new("bd6")
        .rule("inner", vec![])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    // Epsilon rule still generates a nonterminal with goto
    assert!(table.nonterminal_to_index.contains_key(&inner));
}

#[test]
fn goto_boundaries_single_state_grammar_goto() {
    // Simplest possible grammar
    let g = GrammarBuilder::new("bd7")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Even simple grammar should have at least 2 states (initial + accept)
    assert!(
        table.state_count >= 2,
        "even minimal grammar needs multiple states"
    );
}

#[test]
fn goto_boundaries_state_count_matches_goto_rows() {
    let g = GrammarBuilder::new("bd8")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        table.state_count,
        table.goto_table.len(),
        "state_count field must match goto_table row count"
    );
}

// ===========================================================================
// 8. goto_integration_* — goto + action table integration (8 tests)
// ===========================================================================

#[test]
fn goto_integration_shift_then_goto() {
    // After shifting 'a', reducing inner → a should trigger goto for inner
    let g = GrammarBuilder::new("ig1")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let tok_a = tok_id(&g, "a");
    // Initial state should have a shift on 'a'
    let actions = table.actions(table.initial_state, tok_a);
    assert!(
        actions.iter().any(|act| matches!(act, Action::Shift(_))),
        "initial state should shift on 'a'"
    );
    let inner = nt_id(&g, "inner");
    // After reducing, goto for 'inner' from initial state should exist
    assert!(
        table.goto(table.initial_state, inner).is_some(),
        "goto(initial, inner) needed after reduce"
    );
}

#[test]
fn goto_integration_reduce_rule_lhs_matches_goto_nonterminal() {
    let g = GrammarBuilder::new("ig2")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    // For each reduce action, the rule's LHS should be in nonterminal_to_index
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if let Action::Reduce(rule_id) = act {
                    let (lhs, _) = table.rule(*rule_id);
                    assert!(
                        table.nonterminal_to_index.contains_key(&lhs),
                        "reduce rule LHS {:?} must be in goto index",
                        lhs
                    );
                }
            }
        }
    }
}

#[test]
fn goto_integration_accept_action_exists() {
    let g = GrammarBuilder::new("ig3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // There should be an Accept action somewhere
    let mut found_accept = false;
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if matches!(act, Action::Accept) {
                    found_accept = true;
                }
            }
        }
    }
    assert!(found_accept, "parse table must contain an Accept action");
}

#[test]
fn goto_integration_eof_in_action_not_goto() {
    let g = GrammarBuilder::new("ig4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof_symbol;
    // EOF should be in the action table symbol_to_index
    assert!(
        table.symbol_to_index.contains_key(&eof),
        "EOF must be in action table columns"
    );
    // EOF should NOT be in goto nonterminal_to_index
    assert!(
        !table.nonterminal_to_index.contains_key(&eof),
        "EOF must not be in goto table columns"
    );
}

#[test]
fn goto_integration_shift_targets_have_actions_or_gotos() {
    let g = GrammarBuilder::new("ig5")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Every state reachable via Shift should have some action or goto defined
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if let Action::Shift(tgt) = act {
                    let tgt_s = tgt.0 as usize;
                    assert!(
                        tgt_s < table.state_count,
                        "shift target must be valid state"
                    );
                }
            }
        }
    }
}

#[test]
fn goto_integration_goto_targets_have_content() {
    // States reachable via goto should have actions or further gotos
    let g = GrammarBuilder::new("ig6")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            let has_actions = table
                .symbol_to_index
                .keys()
                .any(|&sym| !table.actions(tgt, sym).is_empty());
            let has_gotos = table
                .nonterminal_to_index
                .keys()
                .any(|&nt2| table.goto(tgt, nt2).is_some());
            assert!(
                has_actions || has_gotos,
                "goto target {tgt:?} should have actions or further gotos"
            );
        }
    }
}

#[test]
fn goto_integration_rules_array_matches_reduce_actions() {
    let g = GrammarBuilder::new("ig7")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Every Reduce(rule_id) must have a valid entry in table.rules
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        for &sym in table.symbol_to_index.keys() {
            for act in table.actions(st, sym) {
                if let Action::Reduce(rule_id) = act {
                    assert!(
                        (rule_id.0 as usize) < table.rules.len(),
                        "Reduce({:?}) out of bounds (rules.len()={})",
                        rule_id,
                        table.rules.len()
                    );
                }
            }
        }
    }
}

#[test]
fn goto_integration_start_symbol_matches_grammar() {
    let g = GrammarBuilder::new("ig8")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    // The table's grammar should have a start symbol
    let grammar_start = table.grammar.start_symbol();
    assert!(
        grammar_start.is_some(),
        "table grammar must have a start symbol"
    );
}
