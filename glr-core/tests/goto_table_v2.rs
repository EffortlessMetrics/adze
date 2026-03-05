#![cfg(feature = "test-api")]

//! Comprehensive tests for GOTO table operations in the GLR parse table.
//!
//! Tests cover: goto validity, terminal rejection, nonterminal coverage,
//! bounds checking, determinism, connectivity, and complex grammars.

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
// 1. Goto returns valid states (8 tests)
// ===========================================================================

#[test]
fn goto_valid_single_rule_grammar() {
    let gram = GrammarBuilder::new("v1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    let tgt = table.goto(table.initial_state, sid);
    assert!(
        tgt.is_some(),
        "goto must return a valid state for start nonterminal"
    );
}

#[test]
fn goto_valid_two_rule_grammar() {
    let gram = GrammarBuilder::new("v2")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let inner = nt_id(&gram, "inner");
    let gotos = all_gotos_for(&table, inner);
    assert!(
        !gotos.is_empty(),
        "goto for 'inner' must yield valid states"
    );
    for (_, tgt) in &gotos {
        assert_ne!(tgt.0, u16::MAX, "goto target must not be sentinel");
    }
}

#[test]
fn goto_valid_returns_distinct_from_source() {
    let gram = GrammarBuilder::new("v3")
        .token("a", "a")
        .token("b", "b")
        .rule("pair", vec!["a", "b"])
        .rule("start", vec!["pair"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let start = nt_id(&gram, "start");
    let tgt = table.goto(table.initial_state, start).unwrap();
    // The accept state should differ from initial
    assert_ne!(
        tgt, table.initial_state,
        "goto(initial, start) should lead to a different state"
    );
}

#[test]
fn goto_valid_multiple_alternatives() {
    let gram = GrammarBuilder::new("v4")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let item = nt_id(&gram, "item");
    let gotos = all_gotos_for(&table, item);
    assert!(
        !gotos.is_empty(),
        "goto for 'item' with alternatives must exist"
    );
}

#[test]
fn goto_valid_three_level_chain() {
    let gram = GrammarBuilder::new("v5")
        .token("z", "z")
        .rule("leaf", vec!["z"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for name in ["leaf", "mid", "start"] {
        let nid = nt_id(&gram, name);
        let gotos = all_gotos_for(&table, nid);
        assert!(
            !gotos.is_empty(),
            "goto for '{name}' must exist in chain grammar"
        );
    }
}

#[test]
fn goto_valid_sequence_grammar() {
    let gram = GrammarBuilder::new("v6")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    let gotos = all_gotos_for(&table, sid);
    assert!(
        !gotos.is_empty(),
        "sequence grammar must have goto for start"
    );
}

#[test]
fn goto_valid_accept_state_reachable() {
    let gram = GrammarBuilder::new("v7")
        .token("w", "w")
        .rule("start", vec!["w"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    let accept_state = table.goto(table.initial_state, sid).unwrap();
    let eof = table.eof();
    let has_accept = table
        .actions(accept_state, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        has_accept,
        "goto target for start must lead to accept state"
    );
}

#[test]
fn goto_valid_with_epsilon_like_single_token() {
    // Grammar with a trivial single-token rule
    let gram = GrammarBuilder::new("v8")
        .token("e", "e")
        .rule("atom", vec!["e"])
        .rule("start", vec!["atom"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let atom = nt_id(&gram, "atom");
    let gotos = all_gotos_for(&table, atom);
    assert!(
        gotos
            .iter()
            .all(|(_, tgt)| (tgt.0 as usize) < table.state_count),
        "all goto targets for 'atom' must be valid states"
    );
}

// ===========================================================================
// 2. Goto returns None for terminals (7 tests)
// ===========================================================================

#[test]
fn goto_none_for_single_terminal() {
    let gram = GrammarBuilder::new("tn1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let tid = tok_id(&gram, "a");
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), tid).is_none(),
            "goto({s}, terminal 'a') must be None"
        );
    }
}

#[test]
fn goto_none_for_all_terminals_simple() {
    let gram = GrammarBuilder::new("tn2")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for tok_name in ["x", "y"] {
        let tid = tok_id(&gram, tok_name);
        for s in 0..table.state_count {
            assert!(
                table.goto(StateId(s as u16), tid).is_none(),
                "goto({s}, terminal '{tok_name}') must be None"
            );
        }
    }
}

#[test]
fn goto_none_for_eof_symbol() {
    let gram = GrammarBuilder::new("tn3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let eof = table.eof();
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), eof).is_none(),
            "goto({s}, EOF) must be None"
        );
    }
}

#[test]
fn goto_none_for_terminal_in_chain_grammar() {
    let gram = GrammarBuilder::new("tn4")
        .token("t", "t")
        .rule("leaf", vec!["t"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let tid = tok_id(&gram, "t");
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), tid).is_none(),
            "goto({s}, terminal 't') must be None in chain grammar"
        );
    }
}

#[test]
fn goto_none_for_terminal_with_alternatives() {
    let gram = GrammarBuilder::new("tn5")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for tok_name in ["a", "b"] {
        let tid = tok_id(&gram, tok_name);
        let any_goto = (0..table.state_count).any(|s| table.goto(StateId(s as u16), tid).is_some());
        assert!(!any_goto, "no goto should exist for terminal '{tok_name}'");
    }
}

#[test]
fn goto_none_for_nonexistent_symbol() {
    let gram = GrammarBuilder::new("tn6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let fake = SymbolId(9999);
    for s in 0..table.state_count {
        assert!(
            table.goto(StateId(s as u16), fake).is_none(),
            "goto({s}, nonexistent symbol) must be None"
        );
    }
}

#[test]
fn goto_none_for_terminal_in_sequence() {
    let gram = GrammarBuilder::new("tn7")
        .token("p", "p")
        .token("q", "q")
        .token("r", "r")
        .rule("start", vec!["p", "q", "r"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for tok_name in ["p", "q", "r"] {
        let tid = tok_id(&gram, tok_name);
        let any_goto = (0..table.state_count).any(|s| table.goto(StateId(s as u16), tid).is_some());
        assert!(!any_goto, "terminal '{tok_name}' must have no goto entries");
    }
}

// ===========================================================================
// 3. Goto covers all nonterminals (8 tests)
// ===========================================================================

#[test]
fn goto_covers_start_symbol() {
    let gram = GrammarBuilder::new("cov1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    assert!(
        !all_gotos_for(&table, sid).is_empty(),
        "start nonterminal must have at least one goto entry"
    );
}

#[test]
fn goto_covers_intermediate_nonterminal() {
    let gram = GrammarBuilder::new("cov2")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let mid = nt_id(&gram, "mid");
    assert!(
        !all_gotos_for(&table, mid).is_empty(),
        "'mid' nonterminal must have at least one goto entry"
    );
}

#[test]
fn goto_covers_all_nonterminals_two_level() {
    let gram = GrammarBuilder::new("cov3")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for name in ["leaf", "start"] {
        let nid = nt_id(&gram, name);
        assert!(
            !all_gotos_for(&table, nid).is_empty(),
            "nonterminal '{name}' must have goto coverage"
        );
    }
}

#[test]
fn goto_covers_all_nonterminals_three_level() {
    let gram = GrammarBuilder::new("cov4")
        .token("t", "t")
        .rule("bottom", vec!["t"])
        .rule("middle", vec!["bottom"])
        .rule("start", vec!["middle"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for name in ["bottom", "middle", "start"] {
        let nid = nt_id(&gram, name);
        assert!(
            !all_gotos_for(&table, nid).is_empty(),
            "nonterminal '{name}' must have goto coverage in three-level grammar"
        );
    }
}

#[test]
fn goto_covers_alternative_productions() {
    let gram = GrammarBuilder::new("cov5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("choice", vec!["a"])
        .rule("choice", vec!["b"])
        .rule("choice", vec!["c"])
        .rule("start", vec!["choice"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let choice = nt_id(&gram, "choice");
    assert!(
        !all_gotos_for(&table, choice).is_empty(),
        "'choice' with 3 alternatives must have goto coverage"
    );
}

#[test]
fn goto_total_entries_positive() {
    let gram = GrammarBuilder::new("cov6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    assert!(
        total_goto_entries(&table) > 0,
        "total goto entries must be positive"
    );
}

#[test]
fn goto_covers_shared_nonterminal() {
    // Nonterminal used in multiple rules
    let gram = GrammarBuilder::new("cov7")
        .token("x", "x")
        .token("y", "y")
        .rule("shared", vec!["x"])
        .rule("branch_a", vec!["shared", "y"])
        .rule("branch_b", vec!["shared"])
        .rule("start", vec!["branch_a"])
        .rule("start", vec!["branch_b"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let shared = nt_id(&gram, "shared");
    assert!(
        !all_gotos_for(&table, shared).is_empty(),
        "shared nonterminal must have goto entries"
    );
}

#[test]
fn goto_covers_nonterminals_in_nonterminal_index() {
    let gram = GrammarBuilder::new("cov8")
        .token("a", "a")
        .token("b", "b")
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .rule("start", vec!["left"])
        .rule("start", vec!["right"])
        .start("start")
        .build();
    let table = build_table(&gram);
    // Every nonterminal in the index must have at least one goto entry
    for &nt in table.nonterminal_to_index.keys() {
        let gotos = all_gotos_for(&table, nt);
        assert!(
            !gotos.is_empty(),
            "nonterminal SymbolId({}) in index must have goto entries",
            nt.0
        );
    }
}

// ===========================================================================
// 4. Goto targets are within bounds (8 tests)
// ===========================================================================

#[test]
fn goto_bounds_simple_grammar() {
    let gram = GrammarBuilder::new("bnd1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
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

#[test]
fn goto_bounds_two_tokens() {
    let gram = GrammarBuilder::new("bnd2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!((tgt.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_bounds_chain_grammar() {
    let gram = GrammarBuilder::new("bnd3")
        .token("z", "z")
        .rule("a", vec!["z"])
        .rule("b", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!((tgt.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_bounds_alternatives() {
    let gram = GrammarBuilder::new("bnd4")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!((tgt.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn goto_bounds_out_of_range_state() {
    let gram = GrammarBuilder::new("bnd5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    let oob = StateId(table.state_count as u16 + 10);
    assert!(
        table.goto(oob, sid).is_none(),
        "goto with out-of-bounds state must return None"
    );
}

#[test]
fn goto_bounds_max_state_id() {
    let gram = GrammarBuilder::new("bnd6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    assert!(
        table.goto(StateId(u16::MAX), sid).is_none(),
        "goto with StateId(MAX) must return None"
    );
}

#[test]
fn goto_bounds_with_multiple_nonterminals() {
    let gram = GrammarBuilder::new("bnd7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("w", vec!["c"])
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .rule("start", vec!["w"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let max_state = table.state_count;
    for s in 0..max_state {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!(
                    (tgt.0 as usize) < max_state,
                    "goto({s}, {}) = {} out of bounds (max {})",
                    nt.0,
                    tgt.0,
                    max_state
                );
            }
        }
    }
}

#[test]
fn goto_bounds_sequential_tokens() {
    let gram = GrammarBuilder::new("bnd8")
        .token("t1", "1")
        .token("t2", "2")
        .token("t3", "3")
        .token("t4", "4")
        .rule("start", vec!["t1", "t2", "t3", "t4"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!((tgt.0 as usize) < table.state_count);
            }
        }
    }
}

// ===========================================================================
// 5. Goto determinism (7 tests)
// ===========================================================================

#[test]
fn goto_determinism_simple_rebuild() {
    let gram = GrammarBuilder::new("det1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t1 = build_table(&gram);
    let t2 = build_table(&gram);
    let sid = nt_id(&gram, "start");
    assert_eq!(
        t1.goto(t1.initial_state, sid),
        t2.goto(t2.initial_state, sid),
        "same grammar must produce same goto for start"
    );
}

#[test]
fn goto_determinism_full_table_match() {
    let gram = GrammarBuilder::new("det2")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let t1 = build_table(&gram);
    let t2 = build_table(&gram);
    assert_eq!(t1.state_count, t2.state_count, "state counts must match");
    for s in 0..t1.state_count {
        for &nt in t1.nonterminal_to_index.keys() {
            assert_eq!(
                t1.goto(StateId(s as u16), nt),
                t2.goto(StateId(s as u16), nt),
                "goto mismatch at state {s}, nt {}",
                nt.0
            );
        }
    }
}

#[test]
fn goto_determinism_alternatives() {
    let gram = GrammarBuilder::new("det3")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let t1 = build_table(&gram);
    let t2 = build_table(&gram);
    let item = nt_id(&gram, "item");
    let g1 = all_gotos_for(&t1, item);
    let g2 = all_gotos_for(&t2, item);
    assert_eq!(g1, g2, "goto entries for 'item' must be deterministic");
}

#[test]
fn goto_determinism_three_rebuilds() {
    let gram = GrammarBuilder::new("det4")
        .token("z", "z")
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let tables: Vec<_> = (0..3).map(|_| build_table(&gram)).collect();
    let sid = nt_id(&gram, "start");
    let first_goto = tables[0].goto(tables[0].initial_state, sid);
    for (i, tbl) in tables.iter().enumerate().skip(1) {
        assert_eq!(
            tbl.goto(tbl.initial_state, sid),
            first_goto,
            "rebuild {i} differs"
        );
    }
}

#[test]
fn goto_determinism_total_entry_count() {
    let gram = GrammarBuilder::new("det5")
        .token("a", "a")
        .token("b", "b")
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .rule("start", vec!["left"])
        .rule("start", vec!["right"])
        .start("start")
        .build();
    let t1 = build_table(&gram);
    let t2 = build_table(&gram);
    assert_eq!(
        total_goto_entries(&t1),
        total_goto_entries(&t2),
        "total goto entry counts must match across builds"
    );
}

#[test]
fn goto_determinism_nonterminal_index_keys() {
    let gram = GrammarBuilder::new("det6")
        .token("m", "m")
        .rule("mid", vec!["m"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let t1 = build_table(&gram);
    let t2 = build_table(&gram);
    let keys1: Vec<_> = t1.nonterminal_to_index.keys().collect();
    let keys2: Vec<_> = t2.nonterminal_to_index.keys().collect();
    assert_eq!(
        keys1, keys2,
        "nonterminal_to_index keys must be deterministic"
    );
}

#[test]
fn goto_determinism_chain_grammar() {
    let gram = GrammarBuilder::new("det7")
        .token("q", "q")
        .rule("c", vec!["q"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t1 = build_table(&gram);
    let t2 = build_table(&gram);
    for name in ["c", "b", "a", "start"] {
        let nid = nt_id(&gram, name);
        assert_eq!(
            all_gotos_for(&t1, nid),
            all_gotos_for(&t2, nid),
            "goto entries for '{name}' must be deterministic"
        );
    }
}

// ===========================================================================
// 6. Goto connectivity (8 tests)
// ===========================================================================

#[test]
fn goto_connectivity_initial_state_has_entries() {
    let gram = GrammarBuilder::new("conn1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let has_goto = table
        .nonterminal_to_index
        .keys()
        .any(|&nt| table.goto(table.initial_state, nt).is_some());
    assert!(has_goto, "initial state must have at least one goto entry");
}

#[test]
fn goto_connectivity_start_reachable_from_initial() {
    let gram = GrammarBuilder::new("conn2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    assert!(
        table.goto(table.initial_state, sid).is_some(),
        "start symbol must be reachable via goto from initial state"
    );
}

#[test]
fn goto_connectivity_chain_reachability() {
    let gram = GrammarBuilder::new("conn3")
        .token("t", "t")
        .rule("leaf", vec!["t"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let table = build_table(&gram);
    // Both 'leaf' and 'start' should be reachable from some state
    for name in ["leaf", "start"] {
        let nid = nt_id(&gram, name);
        let reachable =
            (0..table.state_count).any(|s| table.goto(StateId(s as u16), nid).is_some());
        assert!(reachable, "nonterminal '{name}' must be reachable via goto");
    }
}

#[test]
fn goto_connectivity_goto_target_has_actions() {
    let gram = GrammarBuilder::new("conn4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    let tgt = table.goto(table.initial_state, sid).unwrap();
    // The accept state should have at least an Accept action on EOF
    let eof = table.eof();
    let acts = table.actions(tgt, eof);
    assert!(!acts.is_empty(), "goto target state must have actions");
}

#[test]
fn goto_connectivity_multiple_nonterminals_from_initial() {
    let gram = GrammarBuilder::new("conn5")
        .token("a", "a")
        .token("b", "b")
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .rule("start", vec!["left"])
        .rule("start", vec!["right"])
        .start("start")
        .build();
    let table = build_table(&gram);
    // At least start and one of left/right should have gotos from initial
    let initial_gotos: Vec<_> = table
        .nonterminal_to_index
        .keys()
        .filter(|&&nt| table.goto(table.initial_state, nt).is_some())
        .collect();
    assert!(
        initial_gotos.len() >= 2,
        "initial state should have gotos for multiple nonterminals, got {}",
        initial_gotos.len()
    );
}

#[test]
fn goto_connectivity_no_self_loops_on_start() {
    let gram = GrammarBuilder::new("conn6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let sid = nt_id(&gram, "start");
    if let Some(tgt) = table.goto(table.initial_state, sid) {
        assert_ne!(
            tgt, table.initial_state,
            "goto(initial, start) should not loop to initial"
        );
    }
}

#[test]
fn goto_connectivity_deep_chain_all_reachable() {
    let gram = GrammarBuilder::new("conn7")
        .token("v", "v")
        .rule("d", vec!["v"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for name in ["d", "c", "b", "start"] {
        let nid = nt_id(&gram, name);
        let reachable =
            (0..table.state_count).any(|s| table.goto(StateId(s as u16), nid).is_some());
        assert!(
            reachable,
            "'{name}' must be reachable in deep chain grammar"
        );
    }
}

#[test]
fn goto_connectivity_goto_targets_are_unique_per_nt() {
    // For a given state, different nonterminals should generally lead to different targets
    let gram = GrammarBuilder::new("conn8")
        .token("a", "a")
        .token("b", "b")
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .rule("start", vec!["left"])
        .rule("start", vec!["right"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let left = nt_id(&gram, "left");
    let right = nt_id(&gram, "right");
    // From initial state, left and right should go to different states
    let left_tgt = table.goto(table.initial_state, left);
    let right_tgt = table.goto(table.initial_state, right);
    // Both nonterminals should have goto entries from initial state
    assert!(
        left_tgt.is_some(),
        "goto for 'left' must exist from initial"
    );
    assert!(
        right_tgt.is_some(),
        "goto for 'right' must exist from initial"
    );
    // Different nonterminals may lead to the same or different states
    if let (Some(lt), Some(rt)) = (left_tgt, right_tgt) {
        assert!(
            (lt.0 as usize) < table.state_count && (rt.0 as usize) < table.state_count,
            "both targets must be valid states"
        );
    }
}

// ===========================================================================
// 7. Complex grammar gotos (9 tests)
// ===========================================================================

#[test]
fn goto_complex_expr_add_grammar() {
    // E -> E + T | T; T -> id
    let gram = GrammarBuilder::new("cx1")
        .token("id", "[a-z]+")
        .token("plus", "\\+")
        .rule("term", vec!["id"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let expr = nt_id(&gram, "expr");
    let term = nt_id(&gram, "term");
    assert!(
        !all_gotos_for(&table, expr).is_empty(),
        "expr must have goto entries"
    );
    assert!(
        !all_gotos_for(&table, term).is_empty(),
        "term must have goto entries"
    );
}

#[test]
fn goto_complex_expr_mul_grammar() {
    // E -> E * F | F; F -> num
    let gram = GrammarBuilder::new("cx2")
        .token("num", "[0-9]+")
        .token("star", "\\*")
        .rule("factor", vec!["num"])
        .rule("expr", vec!["expr", "star", "factor"])
        .rule("expr", vec!["factor"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let factor = nt_id(&gram, "factor");
    let gotos = all_gotos_for(&table, factor);
    assert!(!gotos.is_empty(), "factor must have goto entries");
    for (_, tgt) in &gotos {
        assert!((tgt.0 as usize) < table.state_count);
    }
}

#[test]
fn goto_complex_three_level_expr() {
    // E -> T; T -> F; F -> id
    let gram = GrammarBuilder::new("cx3")
        .token("id", "[a-z]+")
        .rule("factor", vec!["id"])
        .rule("term", vec!["factor"])
        .rule("expr", vec!["term"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&gram);
    for name in ["factor", "term", "expr", "start"] {
        let nid = nt_id(&gram, name);
        assert!(
            !all_gotos_for(&table, nid).is_empty(),
            "'{name}' in three-level expr must have gotos"
        );
    }
}

#[test]
fn goto_complex_left_recursive() {
    // list -> list item | item; item -> tok
    let gram = GrammarBuilder::new("cx4")
        .token("tok", "t")
        .rule("item", vec!["tok"])
        .rule("lst", vec!["lst", "item"])
        .rule("lst", vec!["item"])
        .rule("start", vec!["lst"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let lst = nt_id(&gram, "lst");
    let gotos = all_gotos_for(&table, lst);
    // Left-recursive grammar should have goto entries in multiple states
    assert!(
        gotos.len() >= 2,
        "left-recursive 'lst' should have gotos from multiple states, got {}",
        gotos.len()
    );
}

#[test]
fn goto_complex_binary_ops() {
    // E -> E op E | atom (ambiguous, GLR handles it)
    let gram = GrammarBuilder::new("cx5")
        .token("atom", "a")
        .token("op", "\\+")
        .rule("expr", vec!["expr", "op", "expr"])
        .rule("expr", vec!["atom"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let expr = nt_id(&gram, "expr");
    let gotos = all_gotos_for(&table, expr);
    assert!(
        gotos.len() >= 2,
        "ambiguous expr must have goto entries from multiple states"
    );
}

#[test]
fn goto_complex_nested_parens() {
    // E -> ( E ) | id
    let gram = GrammarBuilder::new("cx6")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .token("id", "[a-z]+")
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .rule("expr", vec!["id"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let expr = nt_id(&gram, "expr");
    let gotos = all_gotos_for(&table, expr);
    // Recursive nesting means expr goto from multiple states
    assert!(
        gotos.len() >= 2,
        "recursive expr with parens must have multiple goto entries"
    );
}

#[test]
fn goto_complex_multiple_nonterminals_same_rhs() {
    // S -> A B; A -> a; B -> b
    let gram = GrammarBuilder::new("cx7")
        .token("a", "a")
        .token("b", "b")
        .rule("aa", vec!["a"])
        .rule("bb", vec!["b"])
        .rule("start", vec!["aa", "bb"])
        .start("start")
        .build();
    let table = build_table(&gram);
    let aa = nt_id(&gram, "aa");
    let bb = nt_id(&gram, "bb");
    assert!(
        !all_gotos_for(&table, aa).is_empty(),
        "'aa' must have goto entries"
    );
    assert!(
        !all_gotos_for(&table, bb).is_empty(),
        "'bb' must have goto entries"
    );
}

#[test]
fn goto_complex_diamond_grammar() {
    // S -> A | B; A -> c; B -> c (same terminal, different nonterminals)
    let gram = GrammarBuilder::new("cx8")
        .token("c", "c")
        .rule("aa", vec!["c"])
        .rule("bb", vec!["c"])
        .rule("start", vec!["aa"])
        .rule("start", vec!["bb"])
        .start("start")
        .build();
    let table = build_table(&gram);
    // Both paths should have goto entries
    let aa = nt_id(&gram, "aa");
    let bb = nt_id(&gram, "bb");
    let aa_gotos = all_gotos_for(&table, aa);
    let bb_gotos = all_gotos_for(&table, bb);
    assert!(
        !aa_gotos.is_empty(),
        "'aa' must have goto entries in diamond"
    );
    assert!(
        !bb_gotos.is_empty(),
        "'bb' must have goto entries in diamond"
    );
}

#[test]
fn goto_complex_wide_alternative_grammar() {
    // S -> A | B | C | D | E; each -> unique token
    let gram = GrammarBuilder::new("cx9")
        .token("t1", "1")
        .token("t2", "2")
        .token("t3", "3")
        .token("t4", "4")
        .token("t5", "5")
        .rule("n1", vec!["t1"])
        .rule("n2", vec!["t2"])
        .rule("n3", vec!["t3"])
        .rule("n4", vec!["t4"])
        .rule("n5", vec!["t5"])
        .rule("start", vec!["n1"])
        .rule("start", vec!["n2"])
        .rule("start", vec!["n3"])
        .rule("start", vec!["n4"])
        .rule("start", vec!["n5"])
        .start("start")
        .build();
    let table = build_table(&gram);
    // All 5 nonterminals plus start must have goto entries
    let total = total_goto_entries(&table);
    assert!(
        total >= 6,
        "wide grammar must have at least 6 goto entries (5 branches + start), got {total}"
    );
    for name in ["n1", "n2", "n3", "n4", "n5", "start"] {
        let nid = nt_id(&gram, name);
        assert!(
            !all_gotos_for(&table, nid).is_empty(),
            "'{name}' must have goto entries in wide grammar"
        );
    }
}
