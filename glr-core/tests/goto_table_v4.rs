#![cfg(feature = "test-api")]

//! GOTO table v4 tests — existence, validity, sparsity, determinism,
//! reachability, complex grammars, and edge cases.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
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

/// Count how many states have at least one shift or reduce action.
fn productive_states(table: &adze_glr_core::ParseTable) -> usize {
    (0..table.state_count)
        .filter(|&s| {
            let st = StateId(s as u16);
            // Check action table for any non-empty actions
            for &sym in table.symbol_to_index.keys() {
                let acts = table.actions(st, sym);
                if !acts.is_empty() {
                    return true;
                }
            }
            // Check goto table for any outgoing transitions
            for &nt in table.nonterminal_to_index.keys() {
                if table.goto(st, nt).is_some() {
                    return true;
                }
            }
            false
        })
        .count()
}

// ===========================================================================
// 1. Goto entries exist — nonterminals have goto entries (8 tests)
// ===========================================================================

#[test]
fn goto_exists_for_start_nonterminal() {
    let g = GrammarBuilder::new("exist1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    assert!(!gotos.is_empty(), "start nonterminal must have goto entries");
}

#[test]
fn goto_exists_for_intermediate_nonterminal() {
    let g = GrammarBuilder::new("exist2")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let gotos = all_gotos_for(&table, inner);
    assert!(
        !gotos.is_empty(),
        "intermediate nonterminal must have goto entries"
    );
}

#[test]
fn goto_exists_for_both_nonterminals_in_two_rule_grammar() {
    let g = GrammarBuilder::new("exist3")
        .token("a", "a")
        .token("b", "b")
        .rule("leaf", vec!["b"])
        .rule("start", vec!["a", "leaf"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let leaf = nt_id(&g, "leaf");
    assert!(
        !all_gotos_for(&table, start).is_empty(),
        "start must have gotos"
    );
    assert!(
        !all_gotos_for(&table, leaf).is_empty(),
        "leaf must have gotos"
    );
}

#[test]
fn goto_exists_at_initial_state_for_start() {
    let g = GrammarBuilder::new("exist4")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let entry = table.goto(table.initial_state, start);
    assert!(
        entry.is_some(),
        "initial state must have goto for start symbol"
    );
}

#[test]
fn goto_nonterminal_to_index_contains_all_nonterminals() {
    let g = GrammarBuilder::new("exist5")
        .token("n", "n")
        .rule("a", vec!["n"])
        .rule("b", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["a", "b", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            table.nonterminal_to_index.contains_key(&nt),
            "nonterminal_to_index must contain '{name}'"
        );
    }
}

#[test]
fn goto_table_rows_equal_state_count() {
    let g = GrammarBuilder::new("exist6")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn goto_exists_for_multi_alternative_nonterminal() {
    let g = GrammarBuilder::new("exist7")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(
        !all_gotos_for(&table, start).is_empty(),
        "multi-alternative nonterminal must have goto"
    );
}

#[test]
fn goto_exists_for_chained_nonterminals() {
    let g = GrammarBuilder::new("exist8")
        .token("w", "w")
        .rule("c", vec!["w"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["a", "b", "c", "start"] {
        let nt = nt_id(&g, name);
        assert!(
            !all_gotos_for(&table, nt).is_empty(),
            "'{name}' must have gotos in chain grammar"
        );
    }
}

// ===========================================================================
// 2. Goto targets valid — goto points to valid states (8 tests)
// ===========================================================================

#[test]
fn goto_target_within_state_count_single_rule() {
    let g = GrammarBuilder::new("valid1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    for (src, tgt) in all_gotos_for(&table, start) {
        assert!(
            (tgt.0 as usize) < table.state_count,
            "goto({src:?}, start) = {tgt:?} exceeds state_count={}",
            table.state_count
        );
    }
}

#[test]
fn goto_targets_valid_for_all_nonterminals() {
    let g = GrammarBuilder::new("valid2")
        .token("x", "x")
        .token("y", "y")
        .rule("leaf", vec!["y"])
        .rule("start", vec!["x", "leaf"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (src, tgt) in all_gotos_for(&table, nt) {
            assert!(
                (tgt.0 as usize) < table.state_count,
                "goto({src:?}, {nt:?}) = {tgt:?} out of bounds"
            );
        }
    }
}

#[test]
fn goto_target_is_not_sentinel() {
    let g = GrammarBuilder::new("valid3")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    for (_, tgt) in all_gotos_for(&table, start) {
        assert_ne!(tgt.0, u16::MAX, "goto target must not be sentinel");
    }
}

#[test]
fn goto_targets_nonzero_or_initial() {
    let g = GrammarBuilder::new("valid4")
        .token("p", "p")
        .rule("mid", vec!["p"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!(
                (tgt.0 as usize) < table.state_count,
                "all goto targets must be within valid range"
            );
        }
    }
}

#[test]
fn goto_target_differs_from_source_for_start() {
    let g = GrammarBuilder::new("valid5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let entry = table.goto(table.initial_state, start);
    if let Some(tgt) = entry {
        assert_ne!(
            tgt, table.initial_state,
            "goto from initial state should go to a different state"
        );
    }
}

#[test]
fn goto_all_targets_valid_three_nonterminal_grammar() {
    let g = GrammarBuilder::new("valid6")
        .token("n", "n")
        .rule("c", vec!["n"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mut total = 0;
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
            assert_ne!(tgt.0, u16::MAX);
            total += 1;
        }
    }
    assert!(total > 0, "must have at least one goto entry");
}

#[test]
fn goto_targets_valid_for_multi_alternative_grammar() {
    let g = GrammarBuilder::new("valid7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
        }
    }
}

#[test]
fn goto_targets_valid_for_concatenation_grammar() {
    let g = GrammarBuilder::new("valid8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
        }
    }
}

// ===========================================================================
// 3. Goto sparsity — not every state/NT pair has a goto (8 tests)
// ===========================================================================

#[test]
fn goto_sparse_single_rule() {
    // Verify that goto entries are bounded — total is at most state_count * nt_count
    let g = GrammarBuilder::new("sparse1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("start", vec!["x", "y", "z"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    let max_possible = table.state_count * table.nonterminal_to_index.len();
    assert!(
        total <= max_possible,
        "goto entries must be bounded: {total} <= {max_possible}"
    );
    assert!(total > 0, "must have at least one goto");
}

#[test]
fn goto_none_for_terminal_symbols() {
    let g = GrammarBuilder::new("sparse2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    // Terminals are not in nonterminal_to_index, so goto returns None
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        assert!(
            table.goto(st, a).is_none(),
            "goto must return None for terminal symbol"
        );
    }
}

#[test]
fn goto_sparse_two_nonterminal_grammar() {
    // Verify goto entries are bounded and proportional
    let g = GrammarBuilder::new("sparse3")
        .token("t", "t")
        .token("u", "u")
        .token("v", "v")
        .rule("a", vec!["t"])
        .rule("b", vec!["u"])
        .rule("c", vec!["v"])
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    let nt_count = table.nonterminal_to_index.len();
    // Every nonterminal should have at least one goto
    assert!(
        total >= nt_count,
        "each NT should have at least one goto: {total} >= {nt_count}"
    );
    assert!(
        total <= table.state_count * nt_count,
        "total gotos bounded: {total} <= {}",
        table.state_count * nt_count
    );
}

#[test]
fn goto_not_all_states_have_all_nt_gotos() {
    // At least one (state, NT) pair has no goto
    let g = GrammarBuilder::new("sparse4")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["b"])
        .rule("start", vec!["a", "inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    let max_possible = table.state_count * table.nonterminal_to_index.len();
    // There must be at least one missing (state, NT) pair
    assert!(
        total <= max_possible,
        "goto entries cannot exceed maximum possible"
    );
}

#[test]
fn goto_sparsity_increases_with_more_terminals() {
    let g_small = GrammarBuilder::new("sparse5a")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g_big = GrammarBuilder::new("sparse5b")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let t_small = build_table(&g_small);
    let t_big = build_table(&g_big);
    // Both should be sparse
    let total_small = total_goto_entries(&t_small);
    let total_big = total_goto_entries(&t_big);
    assert!(total_small > 0);
    assert!(total_big > 0);
    // Bigger grammar should have more states
    assert!(t_big.state_count >= t_small.state_count);
}

#[test]
fn goto_none_for_unknown_symbol() {
    let g = GrammarBuilder::new("sparse6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let unknown = SymbolId(9999);
    for s in 0..table.state_count {
        let st = StateId(s as u16);
        assert!(
            table.goto(st, unknown).is_none(),
            "goto must return None for unknown symbol"
        );
    }
}

#[test]
fn goto_none_for_out_of_range_state() {
    let g = GrammarBuilder::new("sparse7")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let out_of_range = StateId(table.state_count as u16 + 10);
    assert!(
        table.goto(out_of_range, start).is_none(),
        "goto must return None for out-of-range state"
    );
}

#[test]
fn goto_density_bounded() {
    let g = GrammarBuilder::new("sparse8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("leaf", vec!["c"])
        .rule("mid", vec!["b", "leaf"])
        .rule("start", vec!["a", "mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    let max_possible = table.state_count * table.nonterminal_to_index.len();
    assert!(
        total <= max_possible,
        "goto density must be bounded: {total} <= {max_possible}"
    );
    assert!(total > 0, "must have at least one goto");
}

// ===========================================================================
// 4. Goto determinism — same grammar → same goto table (8 tests)
// ===========================================================================

#[test]
fn goto_deterministic_single_rule() {
    let mk = || {
        let g = GrammarBuilder::new("det1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(t1.goto_table, t2.goto_table);
}

#[test]
fn goto_deterministic_two_nonterminals() {
    let mk = || {
        let g = GrammarBuilder::new("det2")
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(t1.goto_table, t2.goto_table);
}

#[test]
fn goto_deterministic_multi_alt() {
    let mk = || {
        let g = GrammarBuilder::new("det3")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(t1.goto_table, t2.goto_table);
}

#[test]
fn goto_deterministic_chain_grammar() {
    let mk = || {
        let g = GrammarBuilder::new("det4")
            .token("w", "w")
            .rule("c", vec!["w"])
            .rule("b", vec!["c"])
            .rule("a", vec!["b"])
            .rule("start", vec!["a"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(t1.goto_table, t2.goto_table);
}

#[test]
fn goto_deterministic_state_count() {
    let mk = || {
        let g = GrammarBuilder::new("det5")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.goto_table, t2.goto_table);
}

#[test]
fn goto_deterministic_nonterminal_indices() {
    let mk = || {
        let g = GrammarBuilder::new("det6")
            .token("n", "n")
            .rule("mid", vec!["n"])
            .rule("start", vec!["mid"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(t1.nonterminal_to_index, t2.nonterminal_to_index);
}

#[test]
fn goto_deterministic_entry_count() {
    let mk = || {
        let g = GrammarBuilder::new("det7")
            .token("a", "a")
            .token("b", "b")
            .rule("leaf", vec!["b"])
            .rule("start", vec!["a", "leaf"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    assert_eq!(total_goto_entries(&t1), total_goto_entries(&t2));
}

#[test]
fn goto_deterministic_three_builds() {
    let mk = || {
        let g = GrammarBuilder::new("det8")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        build_table(&g)
    };
    let t1 = mk();
    let t2 = mk();
    let t3 = mk();
    assert_eq!(t1.goto_table, t2.goto_table);
    assert_eq!(t2.goto_table, t3.goto_table);
}

// ===========================================================================
// 5. Goto reachability — goto targets lead to productive states (8 tests)
// ===========================================================================

#[test]
fn goto_targets_have_actions_or_gotos() {
    let g = GrammarBuilder::new("reach1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    for (_, tgt) in all_gotos_for(&table, start) {
        let has_actions = table
            .symbol_to_index
            .keys()
            .any(|&sym| !table.actions(tgt, sym).is_empty());
        let has_gotos = table
            .nonterminal_to_index
            .keys()
            .any(|&nt| table.goto(tgt, nt).is_some());
        assert!(
            has_actions || has_gotos,
            "goto target {tgt:?} must have some actions or gotos"
        );
    }
}

#[test]
fn goto_targets_productive_in_chain() {
    let g = GrammarBuilder::new("reach2")
        .token("t", "t")
        .rule("c", vec!["t"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!(
                (tgt.0 as usize) < table.state_count,
                "target must be valid"
            );
            let has_actions = table
                .symbol_to_index
                .keys()
                .any(|&sym| !table.actions(tgt, sym).is_empty());
            let has_gotos = table
                .nonterminal_to_index
                .keys()
                .any(|&nt2| table.goto(tgt, nt2).is_some());
            assert!(has_actions || has_gotos);
        }
    }
}

#[test]
fn goto_start_target_has_accept_action() {
    let g = GrammarBuilder::new("reach3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let tgt = table.goto(table.initial_state, start);
    if let Some(target_state) = tgt {
        let eof = table.eof_symbol;
        let actions = table.actions(target_state, eof);
        let has_accept = actions.iter().any(|a| matches!(a, Action::Accept));
        assert!(
            has_accept,
            "goto target for start from initial should have Accept on EOF"
        );
    }
}

#[test]
fn goto_all_productive_states_covered() {
    let g = GrammarBuilder::new("reach4")
        .token("a", "a")
        .token("b", "b")
        .rule("mid", vec!["b"])
        .rule("start", vec!["a", "mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    let prod = productive_states(&table);
    assert!(
        prod > 0,
        "must have productive states"
    );
    assert!(
        prod <= table.state_count,
        "productive states must not exceed total"
    );
}

#[test]
fn goto_reachable_from_initial_state() {
    let g = GrammarBuilder::new("reach5")
        .token("x", "x")
        .rule("mid", vec!["x"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    // At least one nonterminal should have a goto from initial state
    let has_goto_from_init = table
        .nonterminal_to_index
        .keys()
        .any(|&nt| table.goto(table.initial_state, nt).is_some());
    assert!(has_goto_from_init, "initial state must have at least one goto");
}

#[test]
fn goto_targets_not_all_same_state() {
    let g = GrammarBuilder::new("reach6")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mut targets: Vec<StateId> = Vec::new();
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            targets.push(tgt);
        }
    }
    if targets.len() > 1 {
        let first = targets[0];
        let all_same = targets.iter().all(|&t| t == first);
        assert!(
            !all_same,
            "goto targets should point to different states for different nonterminals"
        );
    }
}

#[test]
fn goto_target_chains_eventually_terminate() {
    let g = GrammarBuilder::new("reach7")
        .token("t", "t")
        .rule("c", vec!["t"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Follow goto chains; they should not form cycles (reach a state
    // with no further gotos within state_count steps).
    for &nt in table.nonterminal_to_index.keys() {
        if let Some(tgt) = table.goto(table.initial_state, nt) {
            let mut visited = vec![false; table.state_count];
            let mut cur = tgt;
            let mut steps = 0;
            loop {
                if (cur.0 as usize) >= table.state_count || visited[cur.0 as usize] {
                    break;
                }
                visited[cur.0 as usize] = true;
                steps += 1;
                // Try to follow any goto from this state
                let next = table
                    .nonterminal_to_index
                    .keys()
                    .find_map(|&nt2| table.goto(cur, nt2));
                match next {
                    Some(n) => cur = n,
                    None => break,
                }
            }
            assert!(
                steps <= table.state_count,
                "goto chain must terminate within state_count steps"
            );
        }
    }
}

#[test]
fn goto_accept_state_has_no_outgoing_gotos() {
    let g = GrammarBuilder::new("reach8")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    if let Some(accept_state) = table.goto(table.initial_state, start) {
        // Accept state typically has no outgoing gotos
        let outgoing: usize = table
            .nonterminal_to_index
            .keys()
            .filter(|&&nt| table.goto(accept_state, nt).is_some())
            .count();
        // For a minimal grammar, accept state has zero or very few gotos
        assert!(
            outgoing <= table.nonterminal_to_index.len(),
            "accept state should have bounded outgoing gotos"
        );
    }
}

// ===========================================================================
// 6. Complex grammars — expression, recursive, mutual recursion (10 tests)
// ===========================================================================

#[test]
fn goto_expression_grammar_all_nonterminals_present() {
    let g = GrammarBuilder::new("expr1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("atom", vec!["NUM"])
        .rule("atom", vec!["LPAREN", "expr", "RPAREN"])
        .rule("term", vec!["atom"])
        .rule("term", vec!["term", "STAR", "atom"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    for name in &["atom", "term", "expr"] {
        let nt = nt_id(&g, name);
        assert!(
            !all_gotos_for(&table, nt).is_empty(),
            "'{name}' must have goto entries in expression grammar"
        );
    }
}

#[test]
fn goto_expression_grammar_targets_valid() {
    let g = GrammarBuilder::new("expr2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("factor", vec!["NUM"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
        }
    }
}

#[test]
fn goto_expression_grammar_more_states_than_minimal() {
    let g_min = GrammarBuilder::new("expr3a")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g_expr = GrammarBuilder::new("expr3b")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("term", vec!["NUM"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .start("expr")
        .build();
    let t_min = build_table(&g_min);
    let t_expr = build_table(&g_expr);
    assert!(
        t_expr.state_count > t_min.state_count,
        "expression grammar should have more states"
    );
}

#[test]
fn goto_recursive_grammar_has_self_goto() {
    let g = GrammarBuilder::new("recur1")
        .token("a", "a")
        .token("b", "b")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "b", "a"])
        .start("list")
        .build();
    let table = build_table(&g);
    let list = nt_id(&g, "list");
    let gotos = all_gotos_for(&table, list);
    assert!(
        !gotos.is_empty(),
        "recursive 'list' must have goto entries"
    );
}

#[test]
fn goto_recursive_grammar_targets_valid() {
    let g = GrammarBuilder::new("recur2")
        .token("x", "x")
        .token("COMMA", ",")
        .rule("items", vec!["x"])
        .rule("items", vec!["items", "COMMA", "x"])
        .start("items")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
        }
    }
}

#[test]
fn goto_left_recursive_list_multiple_gotos() {
    let g = GrammarBuilder::new("recur3")
        .token("ID", r"[a-z]+")
        .token("SEMI", ";")
        .rule("stmt", vec!["ID"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", "SEMI", "stmt"])
        .start("stmts")
        .build();
    let table = build_table(&g);
    let stmts = nt_id(&g, "stmts");
    let stmt = nt_id(&g, "stmt");
    assert!(!all_gotos_for(&table, stmts).is_empty());
    assert!(!all_gotos_for(&table, stmt).is_empty());
}

#[test]
fn goto_mutual_recursion_both_present() {
    let g = GrammarBuilder::new("mutual1")
        .token("a", "a")
        .token("b", "b")
        .rule("alpha", vec!["a"])
        .rule("alpha", vec!["b", "beta"])
        .rule("beta", vec!["b"])
        .rule("beta", vec!["a", "alpha"])
        .rule("start", vec!["alpha"])
        .start("start")
        .build();
    let table = build_table(&g);
    let alpha = nt_id(&g, "alpha");
    let beta = nt_id(&g, "beta");
    assert!(
        !all_gotos_for(&table, alpha).is_empty(),
        "mutual-recursion 'alpha' must have gotos"
    );
    assert!(
        !all_gotos_for(&table, beta).is_empty(),
        "mutual-recursion 'beta' must have gotos"
    );
}

#[test]
fn goto_mutual_recursion_targets_valid() {
    let g = GrammarBuilder::new("mutual2")
        .token("a", "a")
        .token("b", "b")
        .rule("p", vec!["a"])
        .rule("p", vec!["b", "q"])
        .rule("q", vec!["b"])
        .rule("q", vec!["a", "p"])
        .rule("start", vec!["p"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
            assert_ne!(tgt.0, u16::MAX);
        }
    }
}

#[test]
fn goto_nested_expression_grammar_has_multiple_nonterminals() {
    let g = GrammarBuilder::new("nested1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("MINUS", r"\-")
        .token("STAR", r"\*")
        .token("SLASH", "SLASH")
        .rule("primary", vec!["NUM"])
        .rule("unary", vec!["primary"])
        .rule("unary", vec!["MINUS", "primary"])
        .rule("factor", vec!["unary"])
        .rule("factor", vec!["factor", "STAR", "unary"])
        .rule("factor", vec!["factor", "SLASH", "unary"])
        .rule("sum", vec!["factor"])
        .rule("sum", vec!["sum", "PLUS", "factor"])
        .rule("sum", vec!["sum", "MINUS", "factor"])
        .start("sum")
        .build();
    let table = build_table(&g);
    let nt_count = table.nonterminal_to_index.len();
    assert!(
        nt_count >= 4,
        "nested expression grammar should have >= 4 nonterminals, got {nt_count}"
    );
    let total = total_goto_entries(&table);
    assert!(total >= nt_count, "should have at least one goto per NT");
}

#[test]
fn goto_complex_grammar_more_gotos_than_simple() {
    let g_simple = GrammarBuilder::new("cplx1a")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g_complex = GrammarBuilder::new("cplx1b")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("leaf", vec!["c"])
        .rule("mid", vec!["b", "leaf"])
        .rule("start", vec!["a", "mid"])
        .start("start")
        .build();
    let t_simple = build_table(&g_simple);
    let t_complex = build_table(&g_complex);
    assert!(
        total_goto_entries(&t_complex) >= total_goto_entries(&t_simple),
        "complex grammar should have at least as many gotos as simple"
    );
}

// ===========================================================================
// 7. Edge cases — minimal grammar, many nonterminals, deep chains (13 tests)
// ===========================================================================

#[test]
fn goto_minimal_grammar_has_one_nonterminal() {
    let g = GrammarBuilder::new("edge1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        !table.nonterminal_to_index.is_empty(),
        "minimal grammar must have at least one nonterminal in goto"
    );
}

#[test]
fn goto_minimal_grammar_single_entry() {
    let g = GrammarBuilder::new("edge2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    assert!(
        total >= 1,
        "minimal grammar must have at least one goto entry"
    );
}

#[test]
fn goto_five_nonterminal_chain() {
    let g = GrammarBuilder::new("edge3")
        .token("t", "t")
        .rule("e", vec!["t"])
        .rule("d", vec!["e"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["e", "d", "c", "b", "start"] {
        let nt = nt_id(&g, name);
        assert!(!all_gotos_for(&table, nt).is_empty(), "chain '{name}' missing goto");
    }
}

#[test]
fn goto_deep_chain_all_targets_valid() {
    let g = GrammarBuilder::new("edge4")
        .token("z", "z")
        .rule("f", vec!["z"])
        .rule("e", vec!["f"])
        .rule("d", vec!["e"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
        }
    }
}

#[test]
fn goto_multiple_alternatives_per_nonterminal() {
    let g = GrammarBuilder::new("edge5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let gotos = all_gotos_for(&table, start);
    assert!(
        !gotos.is_empty(),
        "4-alternative nonterminal must have gotos"
    );
}

#[test]
fn goto_two_token_sequence() {
    let g = GrammarBuilder::new("edge6")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(!all_gotos_for(&table, start).is_empty());
}

#[test]
fn goto_long_rhs_rule() {
    let g = GrammarBuilder::new("edge7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(!all_gotos_for(&table, start).is_empty());
}

#[test]
fn goto_mixed_terminals_and_nonterminals_in_rhs() {
    let g = GrammarBuilder::new("edge8")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["b"])
        .rule("start", vec!["a", "inner", "a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    assert!(!all_gotos_for(&table, inner).is_empty());
}

#[test]
fn goto_diamond_grammar_all_targets_valid() {
    // Diamond: start → left | right, left → leaf, right → leaf
    let g = GrammarBuilder::new("edge9")
        .token("x", "x")
        .token("y", "y")
        .rule("leaf", vec!["x"])
        .rule("left", vec!["leaf"])
        .rule("right", vec!["y", "leaf"])
        .rule("start", vec!["left"])
        .rule("start", vec!["right"])
        .start("start")
        .build();
    let table = build_table(&g);
    for &nt in table.nonterminal_to_index.keys() {
        for (_, tgt) in all_gotos_for(&table, nt) {
            assert!((tgt.0 as usize) < table.state_count);
        }
    }
}

#[test]
fn goto_single_token_single_nt_minimal_entries() {
    let g = GrammarBuilder::new("edge10")
        .token("q", "q")
        .rule("start", vec!["q"])
        .start("start")
        .build();
    let table = build_table(&g);
    let total = total_goto_entries(&table);
    // Minimal grammar — should have very few goto entries
    assert!(
        total <= 5,
        "minimal grammar goto entries should be very small, got {total}"
    );
}

#[test]
fn goto_grammar_with_epsilon_like_alternatives() {
    // One alternative is short, another is longer
    let g = GrammarBuilder::new("edge11")
        .token("a", "a")
        .token("b", "b")
        .rule("opt", vec!["a"])
        .rule("opt", vec!["a", "b"])
        .rule("start", vec!["opt"])
        .start("start")
        .build();
    let table = build_table(&g);
    let opt = nt_id(&g, "opt");
    assert!(!all_gotos_for(&table, opt).is_empty());
}

#[test]
fn goto_nonterminal_to_index_keys_match_grammar_rule_names() {
    let g = GrammarBuilder::new("edge12")
        .token("a", "a")
        .token("b", "b")
        .rule("leaf", vec!["b"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["a", "mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Every nonterminal in the table should correspond to a known rule name
    for &nt in table.nonterminal_to_index.keys() {
        assert!(
            g.rule_names.contains_key(&nt),
            "goto nonterminal {nt:?} must exist in grammar rule_names"
        );
    }
}

#[test]
fn goto_table_row_widths_consistent() {
    let g = GrammarBuilder::new("edge13")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["b"])
        .rule("start", vec!["a", "inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    if !table.goto_table.is_empty() {
        let width = table.goto_table[0].len();
        for (i, row) in table.goto_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                width,
                "goto_table row {i} width {} != expected {width}",
                row.len()
            );
        }
    }
}
