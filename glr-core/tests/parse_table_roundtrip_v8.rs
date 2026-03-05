//! ParseTable construction, structure, and roundtrip invariants.
//!
//! 80+ tests across 10 categories (8+ per category):
//!   ptr_v8_construct_*    — table construction from grammar
//!   ptr_v8_state_*        — state_count invariants
//!   ptr_v8_symbol_*       — symbol_count and eof_symbol invariants
//!   ptr_v8_action_*       — action query correctness
//!   ptr_v8_goto_*         — goto table entries
//!   ptr_v8_rule_*         — rule() accessor invariants
//!   ptr_v8_rt_field_*     — roundtrip preserves individual fields
//!   ptr_v8_rt_action_*    — roundtrip preserves actions/gotos
//!   ptr_v8_rt_stable_*    — double/triple roundtrip stability
//!   ptr_v8_rt_bytes_*     — byte-level properties (determinism, size)
//!
//! Run with:
//!   cargo test -p adze-glr-core --test parse_table_roundtrip_v8 --features serialization -- --test-threads=2

#![cfg(feature = "serialization")]

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_pt(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("first/follow");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn roundtrip(pt: &ParseTable) -> ParseTable {
    let bytes = pt.to_bytes().expect("serialize");
    ParseTable::from_bytes(&bytes).expect("deserialize")
}

// ---------------------------------------------------------------------------
// Grammar factories — all names prefixed "ptr_v8_"
// ---------------------------------------------------------------------------

/// S -> a
fn grammar_single() -> Grammar {
    GrammarBuilder::new("ptr_v8_single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S -> a b
fn grammar_pair() -> Grammar {
    GrammarBuilder::new("ptr_v8_pair")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build()
}

/// S -> a | b
fn grammar_alt() -> Grammar {
    GrammarBuilder::new("ptr_v8_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

/// S -> A, A -> a
fn grammar_chain() -> Grammar {
    GrammarBuilder::new("ptr_v8_chain")
        .token("a", "a")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["a"])
        .start("s")
        .build()
}

/// S -> S a | a  (left-recursive)
fn grammar_left_rec() -> Grammar {
    GrammarBuilder::new("ptr_v8_left_rec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S -> a S | a  (right-recursive)
fn grammar_right_rec() -> Grammar {
    GrammarBuilder::new("ptr_v8_right_rec")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S -> a | ε
fn grammar_epsilon() -> Grammar {
    GrammarBuilder::new("ptr_v8_epsilon")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec![])
        .start("s")
        .build()
}

/// A -> B, B -> C, C -> t
fn grammar_deep_chain() -> Grammar {
    GrammarBuilder::new("ptr_v8_deep_chain")
        .token("t", "t")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["t"])
        .start("a")
        .build()
}

/// E -> E + E | n  (ambiguous)
fn grammar_ambiguous() -> Grammar {
    GrammarBuilder::new("ptr_v8_ambiguous")
        .token("n", r"\d+")
        .token("plus", r"\+")
        .rule("e", vec!["e", "plus", "e"])
        .rule("e", vec!["n"])
        .start("e")
        .build()
}

/// E -> E + T | T, T -> T * F | F, F -> n | ( E )
fn grammar_arith() -> Grammar {
    GrammarBuilder::new("ptr_v8_arith")
        .token("n", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("lp", r"\(")
        .token("rp", r"\)")
        .rule("e", vec!["e", "plus", "t"])
        .rule("e", vec!["t"])
        .rule("t", vec!["t", "star", "f"])
        .rule("t", vec!["f"])
        .rule("f", vec!["n"])
        .rule("f", vec!["lp", "e", "rp"])
        .start("e")
        .build()
}

/// S -> A B C, A -> a, B -> b, C -> c
fn grammar_three_nt() -> Grammar {
    GrammarBuilder::new("ptr_v8_three_nt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["x", "y", "z"])
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .start("s")
        .build()
}

/// S -> a | b | c | d | e
fn grammar_five_alt() -> Grammar {
    GrammarBuilder::new("ptr_v8_five_alt")
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
        .build()
}

/// S -> A B, A -> a | ε, B -> b
fn grammar_nullable_prefix() -> Grammar {
    GrammarBuilder::new("ptr_v8_nullable_prefix")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["opt", "tail"])
        .rule("opt", vec!["a"])
        .rule("opt", vec![])
        .rule("tail", vec!["b"])
        .start("s")
        .build()
}

// =========================================================================
// CATEGORY 1: ptr_v8_construct_* — table construction from grammar (8 tests)
// =========================================================================

#[test]
fn ptr_v8_construct_single_rule_succeeds() {
    let pt = build_pt(&grammar_single());
    assert!(pt.state_count > 0);
}

#[test]
fn ptr_v8_construct_pair_rule_succeeds() {
    let pt = build_pt(&grammar_pair());
    assert!(pt.state_count > 0);
}

#[test]
fn ptr_v8_construct_alt_rule_succeeds() {
    let pt = build_pt(&grammar_alt());
    assert!(pt.state_count > 0);
}

#[test]
fn ptr_v8_construct_chain_succeeds() {
    let pt = build_pt(&grammar_chain());
    assert!(pt.state_count > 0);
}

#[test]
fn ptr_v8_construct_left_rec_succeeds() {
    let pt = build_pt(&grammar_left_rec());
    assert!(pt.state_count > 1);
}

#[test]
fn ptr_v8_construct_right_rec_succeeds() {
    let pt = build_pt(&grammar_right_rec());
    assert!(pt.state_count > 1);
}

#[test]
fn ptr_v8_construct_epsilon_succeeds() {
    let pt = build_pt(&grammar_epsilon());
    assert!(pt.state_count > 0);
}

#[test]
fn ptr_v8_construct_arith_succeeds() {
    let pt = build_pt(&grammar_arith());
    assert!(pt.state_count > 3);
}

// =========================================================================
// CATEGORY 2: ptr_v8_state_* — state_count invariants (8 tests)
// =========================================================================

#[test]
fn ptr_v8_state_count_positive() {
    let pt = build_pt(&grammar_single());
    assert!(pt.state_count > 0);
}

#[test]
fn ptr_v8_state_count_grows_with_complexity() {
    let simple = build_pt(&grammar_single());
    let complex = build_pt(&grammar_arith());
    assert!(complex.state_count > simple.state_count);
}

#[test]
fn ptr_v8_state_count_recursive_exceeds_two() {
    let pt = build_pt(&grammar_left_rec());
    assert!(pt.state_count > 2);
}

#[test]
fn ptr_v8_state_count_deterministic() {
    let a = build_pt(&grammar_pair());
    let b = build_pt(&grammar_pair());
    assert_eq!(a.state_count, b.state_count);
}

#[test]
fn ptr_v8_state_count_deep_chain_exceeds_depth() {
    let pt = build_pt(&grammar_deep_chain());
    // chain A->B->C->t needs at least 3 states
    assert!(pt.state_count >= 3);
}

#[test]
fn ptr_v8_state_count_epsilon_positive() {
    let pt = build_pt(&grammar_epsilon());
    assert!(pt.state_count > 0);
}

#[test]
fn ptr_v8_state_count_five_alt_exceeds_alt_count() {
    let pt = build_pt(&grammar_five_alt());
    assert!(pt.state_count > 3);
}

#[test]
fn ptr_v8_state_count_valid_index_range() {
    let pt = build_pt(&grammar_pair());
    let max = pt.state_count;
    // Every state id from 0..state_count should be queryable
    for i in 0..max {
        let _ = pt.actions(StateId(i as u16), pt.eof_symbol);
    }
}

// =========================================================================
// CATEGORY 3: ptr_v8_symbol_* — symbol_count and eof_symbol (8 tests)
// =========================================================================

#[test]
fn ptr_v8_symbol_count_positive() {
    let pt = build_pt(&grammar_single());
    assert!(pt.symbol_count > 0);
}

#[test]
fn ptr_v8_symbol_count_covers_terminals() {
    let pt = build_pt(&grammar_pair());
    // At least 2 terminals + 1 non-terminal + EOF
    assert!(pt.symbol_count >= 3);
}

#[test]
fn ptr_v8_symbol_count_grows_with_tokens() {
    let few = build_pt(&grammar_single());
    let many = build_pt(&grammar_five_alt());
    assert!(many.symbol_count > few.symbol_count);
}

#[test]
fn ptr_v8_symbol_count_deterministic() {
    let a = build_pt(&grammar_chain());
    let b = build_pt(&grammar_chain());
    assert_eq!(a.symbol_count, b.symbol_count);
}

#[test]
fn ptr_v8_eof_symbol_valid() {
    let pt = build_pt(&grammar_single());
    // eof_symbol should be a meaningful SymbolId (may be outside symbol_count for sentinel)
    assert!(pt.eof_symbol.0 < u16::MAX);
}

#[test]
fn ptr_v8_eof_matches_eof_accessor() {
    let pt = build_pt(&grammar_pair());
    assert_eq!(pt.eof_symbol, pt.eof());
}

#[test]
fn ptr_v8_eof_deterministic() {
    let a = build_pt(&grammar_alt());
    let b = build_pt(&grammar_alt());
    assert_eq!(a.eof_symbol, b.eof_symbol);
}

#[test]
fn ptr_v8_start_symbol_differs_from_eof() {
    let pt = build_pt(&grammar_chain());
    assert_ne!(pt.start_symbol(), pt.eof());
}

// =========================================================================
// CATEGORY 4: ptr_v8_action_* — action query correctness (8 tests)
// =========================================================================

#[test]
fn ptr_v8_action_shift_exists() {
    let pt = build_pt(&grammar_single());
    let mut has_shift = false;
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let actions = pt.actions(StateId(s as u16), SymbolId(sym as u16));
            if actions.iter().any(|a| matches!(a, Action::Shift(_))) {
                has_shift = true;
            }
        }
    }
    assert!(has_shift);
}

#[test]
fn ptr_v8_action_reduce_exists() {
    let pt = build_pt(&grammar_single());
    let mut has_reduce = false;
    for s in 0..pt.state_count {
        let actions = pt.actions(StateId(s as u16), pt.eof_symbol);
        if actions.iter().any(|a| matches!(a, Action::Reduce(_))) {
            has_reduce = true;
        }
    }
    assert!(has_reduce);
}

#[test]
fn ptr_v8_action_accept_exists() {
    let pt = build_pt(&grammar_pair());
    let eof = pt.eof_symbol;
    let mut found = false;
    for s in 0..pt.state_count {
        if pt
            .actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
        {
            found = true;
            break;
        }
    }
    assert!(found);
}

#[test]
fn ptr_v8_action_accept_single_occurrence() {
    let pt = build_pt(&grammar_single());
    let eof = pt.eof_symbol;
    let mut count = 0usize;
    for s in 0..pt.state_count {
        for a in pt.actions(StateId(s as u16), eof) {
            if matches!(a, Action::Accept) {
                count += 1;
            }
        }
    }
    assert_eq!(count, 1);
}

#[test]
fn ptr_v8_action_query_all_states_no_panic() {
    let pt = build_pt(&grammar_arith());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let _ = pt.actions(StateId(s as u16), SymbolId(sym as u16));
        }
    }
}

#[test]
fn ptr_v8_action_epsilon_grammar_has_actions() {
    let pt = build_pt(&grammar_epsilon());
    let mut total_actions = 0usize;
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            total_actions += pt.actions(StateId(s as u16), SymbolId(sym as u16)).len();
        }
    }
    assert!(total_actions > 0);
}

#[test]
fn ptr_v8_action_shift_targets_valid_states() {
    let pt = build_pt(&grammar_pair());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            for a in pt.actions(StateId(s as u16), SymbolId(sym as u16)) {
                if let Action::Shift(target) = a {
                    assert!((target.0 as usize) < pt.state_count);
                }
            }
        }
    }
}

#[test]
fn ptr_v8_action_reduce_has_valid_rule_id() {
    let pt = build_pt(&grammar_chain());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            for a in pt.actions(StateId(s as u16), SymbolId(sym as u16)) {
                if let Action::Reduce(rid) = a {
                    let (lhs, _rhs_len) = pt.rule(*rid);
                    assert!((lhs.0 as usize) < pt.symbol_count);
                }
            }
        }
    }
}

// =========================================================================
// CATEGORY 5: ptr_v8_goto_* — goto table entries (8 tests)
// =========================================================================

#[test]
fn ptr_v8_goto_nonterminal_queryable() {
    let pt = build_pt(&grammar_chain());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let _ = pt.goto(StateId(s as u16), SymbolId(sym as u16));
        }
    }
}

#[test]
fn ptr_v8_goto_returns_valid_state() {
    let pt = build_pt(&grammar_chain());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            if let Some(target) = pt.goto(StateId(s as u16), SymbolId(sym as u16)) {
                assert!((target.0 as usize) < pt.state_count);
            }
        }
    }
}

#[test]
fn ptr_v8_goto_deterministic() {
    let pt = build_pt(&grammar_three_nt());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let a = pt.goto(StateId(s as u16), SymbolId(sym as u16));
            let b = pt.goto(StateId(s as u16), SymbolId(sym as u16));
            assert_eq!(a, b);
        }
    }
}

#[test]
fn ptr_v8_goto_chain_nonterminal_reaches_next() {
    let pt = build_pt(&grammar_chain());
    // At least one goto entry should be Some for a grammar with non-terminals
    let mut has_some = false;
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            if pt.goto(StateId(s as u16), SymbolId(sym as u16)).is_some() {
                has_some = true;
            }
        }
    }
    assert!(has_some);
}

#[test]
fn ptr_v8_goto_epsilon_grammar_queryable() {
    let pt = build_pt(&grammar_epsilon());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let _ = pt.goto(StateId(s as u16), SymbolId(sym as u16));
        }
    }
}

#[test]
fn ptr_v8_goto_three_nt_has_entries() {
    let pt = build_pt(&grammar_three_nt());
    let mut count = 0usize;
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            if pt.goto(StateId(s as u16), SymbolId(sym as u16)).is_some() {
                count += 1;
            }
        }
    }
    assert!(count > 0);
}

#[test]
fn ptr_v8_goto_arith_has_multiple_entries() {
    let pt = build_pt(&grammar_arith());
    let mut count = 0usize;
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            if pt.goto(StateId(s as u16), SymbolId(sym as u16)).is_some() {
                count += 1;
            }
        }
    }
    // Arithmetic grammar has E, T, F non-terminals — expect several gotos
    assert!(count >= 3);
}

#[test]
fn ptr_v8_goto_nullable_prefix_queryable() {
    let pt = build_pt(&grammar_nullable_prefix());
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let _ = pt.goto(StateId(s as u16), SymbolId(sym as u16));
        }
    }
}

// =========================================================================
// CATEGORY 6: ptr_v8_rule_* — rule() accessor invariants (8 tests)
// =========================================================================

#[test]
fn ptr_v8_rule_accessor_no_panic() {
    let pt = build_pt(&grammar_single());
    for i in 0..pt.rules.len() {
        let _ = pt.rule(RuleId(i as u16));
    }
}

#[test]
fn ptr_v8_rule_lhs_in_range() {
    let pt = build_pt(&grammar_chain());
    for i in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(i as u16));
        assert!((lhs.0 as usize) < pt.symbol_count);
    }
}

#[test]
fn ptr_v8_rule_rhs_len_nonnegative() {
    let pt = build_pt(&grammar_pair());
    for i in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(i as u16));
        // u16 is always >= 0, but verify the value makes sense
        assert!(rhs_len <= 100);
    }
}

#[test]
fn ptr_v8_rule_epsilon_grammar_has_rules() {
    let pt = build_pt(&grammar_epsilon());
    assert!(!pt.rules.is_empty());
    // Epsilon grammar should have at least 2 rules (S -> a and S -> ε or augmented)
    assert!(pt.rules.len() >= 2);
}

#[test]
fn ptr_v8_rule_pair_has_rhs_two() {
    let pt = build_pt(&grammar_pair());
    let mut has_two = false;
    for i in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(i as u16));
        if rhs_len == 2 {
            has_two = true;
        }
    }
    assert!(has_two);
}

#[test]
fn ptr_v8_rule_arith_multiple_rules() {
    let pt = build_pt(&grammar_arith());
    // E->E+T, E->T, T->T*F, T->F, F->n, F->(E) = 6 rules + augmented start
    assert!(pt.rules.len() >= 6);
}

#[test]
fn ptr_v8_rule_deterministic() {
    let pt = build_pt(&grammar_alt());
    for i in 0..pt.rules.len() {
        let (lhs1, rhs1) = pt.rule(RuleId(i as u16));
        let (lhs2, rhs2) = pt.rule(RuleId(i as u16));
        assert_eq!(lhs1, lhs2);
        assert_eq!(rhs1, rhs2);
    }
}

#[test]
fn ptr_v8_rule_three_nt_has_rhs_three() {
    let pt = build_pt(&grammar_three_nt());
    let mut has_three = false;
    for i in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(i as u16));
        if rhs_len == 3 {
            has_three = true;
        }
    }
    assert!(has_three);
}

// =========================================================================
// CATEGORY 7: ptr_v8_rt_field_* — roundtrip preserves individual fields (8 tests)
// =========================================================================

#[test]
fn ptr_v8_rt_field_state_count() {
    let pt = build_pt(&grammar_pair());
    let rt = roundtrip(&pt);
    assert_eq!(pt.state_count, rt.state_count);
}

#[test]
fn ptr_v8_rt_field_symbol_count() {
    let pt = build_pt(&grammar_alt());
    let rt = roundtrip(&pt);
    assert_eq!(pt.symbol_count, rt.symbol_count);
}

#[test]
fn ptr_v8_rt_field_eof_symbol() {
    let pt = build_pt(&grammar_chain());
    let rt = roundtrip(&pt);
    assert_eq!(pt.eof_symbol, rt.eof_symbol);
}

#[test]
fn ptr_v8_rt_field_start_symbol() {
    let pt = build_pt(&grammar_arith());
    let rt = roundtrip(&pt);
    assert_eq!(pt.start_symbol(), rt.start_symbol());
}

#[test]
fn ptr_v8_rt_field_rule_count() {
    let pt = build_pt(&grammar_arith());
    let rt = roundtrip(&pt);
    assert_eq!(pt.rules.len(), rt.rules.len());
}

#[test]
fn ptr_v8_rt_field_token_count() {
    let pt = build_pt(&grammar_five_alt());
    let rt = roundtrip(&pt);
    assert_eq!(pt.token_count, rt.token_count);
}

#[test]
fn ptr_v8_rt_field_initial_state() {
    let pt = build_pt(&grammar_single());
    let rt = roundtrip(&pt);
    assert_eq!(pt.initial_state, rt.initial_state);
}

#[test]
fn ptr_v8_rt_field_extras() {
    let pt = build_pt(&grammar_nullable_prefix());
    let rt = roundtrip(&pt);
    assert_eq!(pt.extras, rt.extras);
}

// =========================================================================
// CATEGORY 8: ptr_v8_rt_action_* — roundtrip preserves actions/gotos (9 tests)
// =========================================================================

#[test]
fn ptr_v8_rt_action_table_shape() {
    let pt = build_pt(&grammar_pair());
    let rt = roundtrip(&pt);
    assert_eq!(pt.action_table.len(), rt.action_table.len());
    for (row_orig, row_rt) in pt.action_table.iter().zip(rt.action_table.iter()) {
        assert_eq!(row_orig.len(), row_rt.len());
    }
}

#[test]
fn ptr_v8_rt_action_all_cells_preserved() {
    let pt = build_pt(&grammar_alt());
    let rt = roundtrip(&pt);
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let orig = pt.actions(StateId(s as u16), SymbolId(sym as u16));
            let rest = rt.actions(StateId(s as u16), SymbolId(sym as u16));
            assert_eq!(orig, rest, "mismatch at state={s}, sym={sym}");
        }
    }
}

#[test]
fn ptr_v8_rt_action_arith_cells_preserved() {
    let pt = build_pt(&grammar_arith());
    let rt = roundtrip(&pt);
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            let orig = pt.actions(StateId(s as u16), SymbolId(sym as u16));
            let rest = rt.actions(StateId(s as u16), SymbolId(sym as u16));
            assert_eq!(orig, rest);
        }
    }
}

#[test]
fn ptr_v8_rt_action_left_rec_cells_preserved() {
    let pt = build_pt(&grammar_left_rec());
    let rt = roundtrip(&pt);
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            assert_eq!(
                pt.actions(StateId(s as u16), SymbolId(sym as u16)),
                rt.actions(StateId(s as u16), SymbolId(sym as u16)),
            );
        }
    }
}

#[test]
fn ptr_v8_rt_goto_all_cells_preserved() {
    let pt = build_pt(&grammar_chain());
    let rt = roundtrip(&pt);
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            assert_eq!(
                pt.goto(StateId(s as u16), SymbolId(sym as u16)),
                rt.goto(StateId(s as u16), SymbolId(sym as u16)),
                "goto mismatch at state={s}, sym={sym}",
            );
        }
    }
}

#[test]
fn ptr_v8_rt_goto_arith_preserved() {
    let pt = build_pt(&grammar_arith());
    let rt = roundtrip(&pt);
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            assert_eq!(
                pt.goto(StateId(s as u16), SymbolId(sym as u16)),
                rt.goto(StateId(s as u16), SymbolId(sym as u16)),
            );
        }
    }
}

#[test]
fn ptr_v8_rt_rules_preserved() {
    let pt = build_pt(&grammar_three_nt());
    let rt = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let (lhs1, rhs1) = pt.rule(RuleId(i as u16));
        let (lhs2, rhs2) = rt.rule(RuleId(i as u16));
        assert_eq!(lhs1, lhs2);
        assert_eq!(rhs1, rhs2);
    }
}

#[test]
fn ptr_v8_rt_action_ambiguous_preserved() {
    let pt = build_pt(&grammar_ambiguous());
    let rt = roundtrip(&pt);
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            assert_eq!(
                pt.actions(StateId(s as u16), SymbolId(sym as u16)),
                rt.actions(StateId(s as u16), SymbolId(sym as u16)),
            );
        }
    }
}

#[test]
fn ptr_v8_rt_goto_nullable_prefix_preserved() {
    let pt = build_pt(&grammar_nullable_prefix());
    let rt = roundtrip(&pt);
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            assert_eq!(
                pt.goto(StateId(s as u16), SymbolId(sym as u16)),
                rt.goto(StateId(s as u16), SymbolId(sym as u16)),
            );
        }
    }
}

// =========================================================================
// CATEGORY 9: ptr_v8_rt_stable_* — double/triple roundtrip stability (8 tests)
// =========================================================================

#[test]
fn ptr_v8_rt_stable_double_same_bytes() {
    let pt = build_pt(&grammar_pair());
    let bytes1 = pt.to_bytes().expect("ser1");
    let rt = ParseTable::from_bytes(&bytes1).expect("de1");
    let bytes2 = rt.to_bytes().expect("ser2");
    assert_eq!(bytes1, bytes2);
}

#[test]
fn ptr_v8_rt_stable_triple_same_bytes() {
    let pt = build_pt(&grammar_arith());
    let b1 = pt.to_bytes().expect("s1");
    let b2 = ParseTable::from_bytes(&b1)
        .expect("d1")
        .to_bytes()
        .expect("s2");
    let b3 = ParseTable::from_bytes(&b2)
        .expect("d2")
        .to_bytes()
        .expect("s3");
    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

#[test]
fn ptr_v8_rt_stable_double_state_count() {
    let pt = build_pt(&grammar_left_rec());
    let rt1 = roundtrip(&pt);
    let rt2 = roundtrip(&rt1);
    assert_eq!(pt.state_count, rt2.state_count);
}

#[test]
fn ptr_v8_rt_stable_double_symbol_count() {
    let pt = build_pt(&grammar_five_alt());
    let rt1 = roundtrip(&pt);
    let rt2 = roundtrip(&rt1);
    assert_eq!(pt.symbol_count, rt2.symbol_count);
}

#[test]
fn ptr_v8_rt_stable_double_eof() {
    let pt = build_pt(&grammar_chain());
    let rt1 = roundtrip(&pt);
    let rt2 = roundtrip(&rt1);
    assert_eq!(pt.eof_symbol, rt2.eof_symbol);
}

#[test]
fn ptr_v8_rt_stable_double_actions_chain() {
    let pt = build_pt(&grammar_chain());
    let rt2 = roundtrip(&roundtrip(&pt));
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            assert_eq!(
                pt.actions(StateId(s as u16), SymbolId(sym as u16)),
                rt2.actions(StateId(s as u16), SymbolId(sym as u16)),
            );
        }
    }
}

#[test]
fn ptr_v8_rt_stable_double_gotos_three_nt() {
    let pt = build_pt(&grammar_three_nt());
    let rt2 = roundtrip(&roundtrip(&pt));
    for s in 0..pt.state_count {
        for sym in 0..pt.symbol_count {
            assert_eq!(
                pt.goto(StateId(s as u16), SymbolId(sym as u16)),
                rt2.goto(StateId(s as u16), SymbolId(sym as u16)),
            );
        }
    }
}

#[test]
fn ptr_v8_rt_stable_double_rules_arith() {
    let pt = build_pt(&grammar_arith());
    let rt2 = roundtrip(&roundtrip(&pt));
    for i in 0..pt.rules.len() {
        let (l1, r1) = pt.rule(RuleId(i as u16));
        let (l2, r2) = rt2.rule(RuleId(i as u16));
        assert_eq!(l1, l2);
        assert_eq!(r1, r2);
    }
}

// =========================================================================
// CATEGORY 10: ptr_v8_rt_bytes_* — byte-level properties (9 tests)
// =========================================================================

#[test]
fn ptr_v8_rt_bytes_deterministic_serialize() {
    let pt = build_pt(&grammar_pair());
    let b1 = pt.to_bytes().expect("s1");
    let b2 = pt.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn ptr_v8_rt_bytes_deterministic_arith() {
    let pt = build_pt(&grammar_arith());
    let b1 = pt.to_bytes().expect("s1");
    let b2 = pt.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn ptr_v8_rt_bytes_different_grammars_differ() {
    let bytes_single = build_pt(&grammar_single()).to_bytes().expect("s");
    let bytes_arith = build_pt(&grammar_arith()).to_bytes().expect("a");
    assert_ne!(bytes_single, bytes_arith);
}

#[test]
fn ptr_v8_rt_bytes_larger_grammar_more_bytes() {
    let small = build_pt(&grammar_single()).to_bytes().expect("s");
    let big = build_pt(&grammar_arith()).to_bytes().expect("a");
    assert!(big.len() > small.len());
}

#[test]
fn ptr_v8_rt_bytes_not_empty() {
    let bytes = build_pt(&grammar_single()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn ptr_v8_rt_bytes_five_alt_differs_from_single() {
    let b1 = build_pt(&grammar_single()).to_bytes().expect("s");
    let b2 = build_pt(&grammar_five_alt()).to_bytes().expect("f");
    assert_ne!(b1, b2);
}

#[test]
fn ptr_v8_rt_bytes_left_rec_differs_from_right_rec() {
    let bl = build_pt(&grammar_left_rec()).to_bytes().expect("l");
    let br = build_pt(&grammar_right_rec()).to_bytes().expect("r");
    assert_ne!(bl, br);
}

#[test]
fn ptr_v8_rt_bytes_same_grammar_same_bytes() {
    let g1 = grammar_chain();
    let g2 = grammar_chain();
    let b1 = build_pt(&g1).to_bytes().expect("1");
    let b2 = build_pt(&g2).to_bytes().expect("2");
    assert_eq!(b1, b2);
}

#[test]
fn ptr_v8_rt_bytes_roundtrip_all_grammars() {
    let grammars: Vec<Grammar> = vec![
        grammar_single(),
        grammar_pair(),
        grammar_alt(),
        grammar_chain(),
        grammar_left_rec(),
        grammar_right_rec(),
        grammar_epsilon(),
        grammar_deep_chain(),
        grammar_ambiguous(),
        grammar_arith(),
        grammar_three_nt(),
        grammar_five_alt(),
        grammar_nullable_prefix(),
    ];
    for g in &grammars {
        let pt = build_pt(g);
        let bytes = pt.to_bytes().expect("serialize");
        let rt = ParseTable::from_bytes(&bytes).expect("deserialize");
        assert_eq!(pt.state_count, rt.state_count);
        assert_eq!(pt.symbol_count, rt.symbol_count);
        assert_eq!(pt.eof_symbol, rt.eof_symbol);
    }
}
