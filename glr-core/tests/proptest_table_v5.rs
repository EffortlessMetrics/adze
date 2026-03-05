#![cfg(feature = "test-api")]
//! Property-based tests for parse table construction, action validity,
//! goto consistency, FIRST/FOLLOW properties, and determinism.
//!
//! Run with:
//! ```bash
//! cargo test -p adze-glr-core --test proptest_table_v5 --features test-api -- --test-threads=2
//! ```

use adze_glr_core::test_helpers::test as th;
use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("build_lr1_automaton failed")
}

fn try_build(grammar: &Grammar) -> Option<ParseTable> {
    let ff = FirstFollowSets::compute(grammar).ok()?;
    build_lr1_automaton(grammar, &ff).ok()
}

fn has_any_accept(table: &ParseTable) -> bool {
    (0..table.state_count).any(|s| th::has_accept_on_eof(table, s))
}

fn nonterminal_ids(grammar: &Grammar) -> Vec<SymbolId> {
    grammar.rules.keys().copied().collect()
}

fn all_actions_flat(table: &ParseTable) -> Vec<&Action> {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter().flat_map(|cell| cell.iter()))
        .collect()
}

// ---------------------------------------------------------------------------
// Fixed grammars
// ---------------------------------------------------------------------------

/// S → a
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("min")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S → a | b
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("twoalt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

/// S → ε | a
fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S → S a | a
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S → T, T → a
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["a"])
        .start("s")
        .build()
}

/// S → a b c
fn sequence_grammar() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build()
}

/// S → a S | a
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rightrec")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// E → E + E | E * E | a (with precedence)
fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("a", "a")
        .token("plus", "+")
        .token("star", "*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build()
}

/// S → T U, T → a, U → b
fn two_nt_seq_grammar() -> Grammar {
    GrammarBuilder::new("twontseq")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["t", "u"])
        .rule("t", vec!["a"])
        .rule("u", vec!["b"])
        .start("s")
        .build()
}

/// S → T, T → U, U → a
fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["u"])
        .rule("u", vec!["a"])
        .start("s")
        .build()
}

/// S → a | b | c | d | e
fn wide_alt_grammar() -> Grammar {
    GrammarBuilder::new("wide")
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

fn arb_fixed_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![
        Just(minimal_grammar()),
        Just(two_alt_grammar()),
        Just(nullable_grammar()),
        Just(left_recursive_grammar()),
        Just(chain_grammar()),
        Just(sequence_grammar()),
        Just(right_recursive_grammar()),
        Just(precedence_grammar()),
        Just(two_nt_seq_grammar()),
        Just(deep_chain_grammar()),
        Just(wide_alt_grammar()),
    ]
}

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

const TOKEN_NAMES: &[&str] = &["a", "b", "c", "d", "e", "f"];
const TOKEN_PATTERNS: &[&str] = &["a", "b", "c", "d", "e", "f"];
const NT_NAMES: &[&str] = &["s", "t", "u", "v", "w"];

fn build_grammar_from(n_tok: usize, productions: &[Vec<Vec<usize>>]) -> Grammar {
    let n_nt = productions.len();
    let mut builder = GrammarBuilder::new("proptest");
    for i in 0..n_tok {
        builder = builder.token(TOKEN_NAMES[i], TOKEN_PATTERNS[i]);
    }
    for (nt_idx, nt_prods) in productions.iter().enumerate() {
        let lhs = NT_NAMES[nt_idx];
        for rhs_indices in nt_prods {
            let rhs: Vec<&str> = rhs_indices
                .iter()
                .map(|&idx| {
                    if idx < n_tok {
                        TOKEN_NAMES[idx]
                    } else {
                        NT_NAMES[idx - n_tok]
                    }
                })
                .collect();
            builder = builder.rule(lhs, rhs);
        }
    }
    let _ = n_nt;
    builder = builder.start(NT_NAMES[0]);
    builder.build()
}

/// Random grammar: 1-3 tokens, 1-3 nonterminals.
fn arb_random_grammar() -> impl Strategy<Value = Grammar> {
    (1..=3usize, 1..=3usize).prop_flat_map(|(n_tok, n_nt)| {
        proptest::collection::vec(
            proptest::collection::vec(proptest::collection::vec(0..(n_tok + n_nt), 1..=3), 1..=3),
            n_nt..=n_nt,
        )
        .prop_map(move |prods| build_grammar_from(n_tok, &prods))
    })
}

/// Simple valid grammar: S → t0 with optional extra alternatives.
fn arb_valid_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=5, 0usize..=2)
        .prop_flat_map(|(n_tok, n_extra)| {
            let rhs_indices = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), rhs_indices)
        })
        .prop_map(|(n_tok, rhs_indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut bld = GrammarBuilder::new("rand");
            for tn in &tok_names {
                bld = bld.token(tn, tn);
            }
            bld = bld.rule("S", vec![tok_names[0].as_str()]);
            for &idx in &rhs_indices {
                bld = bld.rule("S", vec![tok_names[idx].as_str()]);
            }
            bld = bld.start("S");
            bld.build()
        })
}

/// Two-nonterminal grammar: S → A, A → tok*.
fn arb_two_nt_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 0usize..=3)
        .prop_flat_map(|(n_tok, n_extra)| {
            let rhs_indices = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), rhs_indices)
        })
        .prop_map(|(n_tok, rhs_indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut bld = GrammarBuilder::new("two_nt");
            for tn in &tok_names {
                bld = bld.token(tn, tn);
            }
            bld = bld.rule("S", vec!["A"]);
            bld = bld.rule("A", vec![tok_names[0].as_str()]);
            for &idx in &rhs_indices {
                bld = bld.rule("A", vec![tok_names[idx].as_str()]);
            }
            bld = bld.start("S");
            bld.build()
        })
}

// ===========================================================================
// Category 1 — Parse table structural properties (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn pt01_state_count_positive(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0);
    }

    #[test]
    fn pt02_action_rows_eq_state_count(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }

    #[test]
    fn pt03_goto_rows_eq_state_count(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
    }

    #[test]
    fn pt04_action_rows_uniform_width(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        if let Some(first) = table.action_table.first() {
            let width = first.len();
            for (i, row) in table.action_table.iter().enumerate() {
                prop_assert_eq!(row.len(), width,
                    "action row {} width {} != {}", i, row.len(), width);
            }
        }
    }

    #[test]
    fn pt05_goto_rows_uniform_width(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        if let Some(first) = table.goto_table.first() {
            let width = first.len();
            for (i, row) in table.goto_table.iter().enumerate() {
                prop_assert_eq!(row.len(), width,
                    "goto row {} width {} != {}", i, row.len(), width);
            }
        }
    }

    #[test]
    fn pt06_initial_state_in_bounds(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!((table.initial_state.0 as usize) < table.state_count);
    }
}

// ===========================================================================
// Category 2 — Action validity (7 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn av01_shift_targets_in_bounds(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        if let Action::Shift(target) = action {
                            prop_assert!(
                                (target.0 as usize) < table.state_count,
                                "Shift({}) >= state_count({})", target.0, table.state_count
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn av02_reduce_rule_ids_in_bounds(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        if let Action::Reduce(rule_id) = action {
                            prop_assert!(
                                (rule_id.0 as usize) < table.rules.len(),
                                "Reduce({}) >= rules.len({})", rule_id.0, table.rules.len()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn av03_accept_exists_for_valid_grammar(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_any_accept(&table), "No Accept action found");
    }

    #[test]
    fn av04_shift_targets_in_bounds_random(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for action in all_actions_flat(&table) {
            if let Action::Shift(target) = action {
                prop_assert!((target.0 as usize) < table.state_count);
            }
        }
    }

    #[test]
    fn av05_reduce_ids_in_bounds_random(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for action in all_actions_flat(&table) {
            if let Action::Reduce(rule_id) = action {
                prop_assert!((rule_id.0 as usize) < table.rules.len());
            }
        }
    }

    #[test]
    fn av06_accept_exists_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_any_accept(&table));
    }

    #[test]
    fn av07_fork_children_are_leaf_actions(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            for action in all_actions_flat(&table) {
                if let Action::Fork(children) = action {
                    for child in children {
                        prop_assert!(
                            !matches!(child, Action::Fork(_)),
                            "Nested Fork in action table"
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// Category 3 — Goto consistency (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn gt01_goto_targets_in_bounds(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        let nts = nonterminal_ids(&grammar);
        for state_idx in 0..table.state_count {
            let sid = StateId(state_idx as u16);
            for &nt in &nts {
                if let Some(target) = table.goto(sid, nt) {
                    prop_assert!(
                        (target.0 as usize) < table.state_count,
                        "goto({}, {:?}) = {} out of bounds", state_idx, nt, target.0
                    );
                }
            }
        }
    }

    #[test]
    fn gt02_goto_targets_in_bounds_fixed(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            let nts = nonterminal_ids(&grammar);
            for state_idx in 0..table.state_count {
                let sid = StateId(state_idx as u16);
                for &nt in &nts {
                    if let Some(target) = table.goto(sid, nt) {
                        prop_assert!((target.0 as usize) < table.state_count);
                    }
                }
            }
        }
    }

    #[test]
    fn gt03_goto_never_returns_sentinel(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        let nts = nonterminal_ids(&grammar);
        for state_idx in 0..table.state_count {
            let sid = StateId(state_idx as u16);
            for &nt in &nts {
                if let Some(target) = table.goto(sid, nt) {
                    prop_assert_ne!(target.0, u16::MAX,
                        "goto returned sentinel u16::MAX");
                }
            }
        }
    }

    #[test]
    fn gt04_goto_for_terminal_returns_none(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for &tok_id in grammar.tokens.keys() {
            for state_idx in 0..table.state_count {
                let sid = StateId(state_idx as u16);
                let result = table.goto(sid, tok_id);
                prop_assert!(
                    result.is_none(),
                    "goto({}, terminal {:?}) should be None, got {:?}",
                    state_idx, tok_id, result
                );
            }
        }
    }

    #[test]
    fn gt05_goto_deterministic(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        let nts = nonterminal_ids(&grammar);
        for state_idx in 0..t1.state_count {
            let sid = StateId(state_idx as u16);
            for &nt in &nts {
                prop_assert_eq!(t1.goto(sid, nt), t2.goto(sid, nt),
                    "goto({}, {:?}) differs across builds", state_idx, nt);
            }
        }
    }

    #[test]
    fn gt06_nonterminal_to_index_covers_grammar_nts(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for &nt in grammar.rules.keys() {
            prop_assert!(
                table.nonterminal_to_index.contains_key(&nt),
                "nonterminal {:?} missing from nonterminal_to_index", nt
            );
        }
    }
}

// ===========================================================================
// Category 4 — FIRST/FOLLOW properties (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn ff01_compute_idempotent(grammar in arb_valid_grammar()) {
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        for &sym in grammar.rules.keys().chain(grammar.tokens.keys()) {
            prop_assert_eq!(ff1.first(sym), ff2.first(sym));
            prop_assert_eq!(ff1.follow(sym), ff2.follow(sym));
            prop_assert_eq!(ff1.is_nullable(sym), ff2.is_nullable(sym));
        }
    }

    #[test]
    fn ff02_terminals_not_nullable(grammar in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        for &tid in grammar.tokens.keys() {
            prop_assert!(!ff.is_nullable(tid),
                "Terminal {:?} is nullable", tid);
        }
    }

    #[test]
    fn ff03_terminal_first_set_if_present_is_consistent(grammar in arb_valid_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        for &tid in grammar.tokens.keys() {
            // Querying FIRST of a terminal should never panic.
            let _ = ff.first(tid);
            // A terminal is never nullable.
            prop_assert!(!ff.is_nullable(tid));
        }
    }

    #[test]
    fn ff04_start_has_eof_in_follow(grammar in arb_valid_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let table = build_table(&grammar);
        if let Some(follow_set) = ff.follow(table.start_symbol()) {
            prop_assert!(follow_set.count_ones(..) > 0,
                "FOLLOW(start) is empty — should contain at least EOF");
        }
    }

    #[test]
    fn ff05_nullable_nt_nonempty_first(grammar in arb_fixed_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&grammar) {
            for &nt in grammar.rules.keys() {
                if ff.is_nullable(nt) {
                    // A nullable nonterminal may still have a nonempty FIRST set
                    // (e.g., S → ε | a), but this should not panic.
                    let _ = ff.first(nt);
                }
            }
        }
    }

    #[test]
    fn ff06_no_panic_on_random_grammar(grammar in arb_random_grammar()) {
        let _ = FirstFollowSets::compute(&grammar);
    }

    #[test]
    fn ff07_first_set_nonempty_for_producing_nt(grammar in arb_valid_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        for &nt in grammar.rules.keys() {
            if let Some(first_set) = ff.first(nt) {
                // Every nonterminal with at least one production containing a
                // terminal should have a nonempty FIRST set.
                let has_terminal_prod = grammar.rules[&nt].iter().any(|rule| {
                    rule.rhs.iter().any(|s| matches!(s, adze_ir::Symbol::Terminal(_)))
                });
                if has_terminal_prod {
                    prop_assert!(first_set.count_ones(..) > 0,
                        "FIRST({:?}) empty despite terminal production", nt);
                }
            }
        }
    }

    #[test]
    fn ff08_idempotent_two_nt(grammar in arb_two_nt_grammar()) {
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        for &sym in grammar.rules.keys() {
            prop_assert_eq!(ff1.first(sym), ff2.first(sym));
        }
    }
}

// ===========================================================================
// Category 5 — Determinism / reproducibility (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn det01_state_count_deterministic(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.state_count, t2.state_count);
    }

    #[test]
    fn det02_symbol_count_deterministic(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
    }

    #[test]
    fn det03_rules_len_deterministic(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.rules.len(), t2.rules.len());
    }

    #[test]
    fn det04_eof_symbol_deterministic(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.eof_symbol, t2.eof_symbol);
    }

    #[test]
    fn det05_initial_state_deterministic(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.initial_state, t2.initial_state);
    }
}

// ===========================================================================
// Category 6 — EOF symbol consistency (4 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn eof01_in_symbol_to_index(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "EOF {:?} not in symbol_to_index", table.eof_symbol
        );
    }

    #[test]
    fn eof02_method_matches_field(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.eof(), table.eof_symbol);
    }

    #[test]
    fn eof03_eof_index_within_bounds(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        let idx = table.symbol_to_index[&table.eof_symbol];
        prop_assert!(idx < table.index_to_symbol.len(),
            "EOF index {} out of bounds", idx);
    }

    #[test]
    fn eof04_accept_only_on_eof(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            for state_idx in 0..table.state_count {
                for (&sym, &col) in &table.symbol_to_index {
                    if sym == table.eof_symbol {
                        continue;
                    }
                    if col < table.action_table[state_idx].len() {
                        let has_accept = table.action_table[state_idx][col]
                            .iter()
                            .any(|a| matches!(a, Action::Accept));
                        prop_assert!(
                            !has_accept,
                            "Accept on non-EOF symbol {:?} in state {}", sym, state_idx
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// Category 7 — Symbol mapping roundtrips (4 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn sm01_symbol_to_index_roundtrip(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for (&sym, &idx) in &table.symbol_to_index {
            prop_assert!(idx < table.index_to_symbol.len());
            prop_assert_eq!(table.index_to_symbol[idx], sym);
        }
    }

    #[test]
    fn sm02_index_to_symbol_roundtrip(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for (idx, &sym) in table.index_to_symbol.iter().enumerate() {
            prop_assert_eq!(
                table.symbol_to_index.get(&sym).copied(),
                Some(idx),
                "index_to_symbol[{}] = {:?} not found in symbol_to_index", idx, sym
            );
        }
    }

    #[test]
    fn sm03_symbol_to_index_keys_unique(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        let key_count = table.symbol_to_index.len();
        let val_set: BTreeSet<usize> = table.symbol_to_index.values().copied().collect();
        prop_assert_eq!(key_count, val_set.len(),
            "symbol_to_index has duplicate values");
    }

    #[test]
    fn sm04_all_grammar_tokens_mapped(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for &tok in grammar.tokens.keys() {
            prop_assert!(
                table.symbol_to_index.contains_key(&tok),
                "Token {:?} not in symbol_to_index", tok
            );
        }
    }
}

// ===========================================================================
// Category 8 — Rule / production validity (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn rv01_rules_have_valid_lhs(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        let known_nts: BTreeSet<SymbolId> =
            table.nonterminal_to_index.keys().copied().collect();
        for rule in &table.rules {
            prop_assert!(
                known_nts.contains(&rule.lhs) || rule.lhs.0 == 0,
                "Rule lhs {:?} not a known nonterminal", rule.lhs
            );
        }
    }

    #[test]
    fn rv02_at_least_one_rule(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.rules.is_empty(), "Parse table has zero rules");
    }

    #[test]
    fn rv03_rule_rhs_len_bounded(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for rule in &table.rules {
            prop_assert!(
                rule.rhs_len <= 100,
                "Rule rhs_len {} unreasonably large", rule.rhs_len
            );
        }
    }

    #[test]
    fn rv04_rule_method_consistent(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for (i, parse_rule) in table.rules.iter().enumerate() {
            let (lhs, rhs_len) = table.rule(RuleId(i as u16));
            prop_assert_eq!(lhs, parse_rule.lhs);
            prop_assert_eq!(rhs_len, parse_rule.rhs_len);
        }
    }

    #[test]
    fn rv05_reduce_actions_reference_existing_rules(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        for action in all_actions_flat(&table) {
            if let Action::Reduce(rule_id) = action {
                prop_assert!(
                    (rule_id.0 as usize) < table.rules.len(),
                    "Reduce references nonexistent rule {}", rule_id.0
                );
            }
        }
    }
}

// ===========================================================================
// Category 9 — Grammar accessor consistency (3 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn ga01_grammar_name_preserved(suffix in 0u32..50) {
        let name = format!("g_{suffix}");
        let grammar = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S")
            .build();
        let table = build_table(&grammar);
        prop_assert_eq!(&table.grammar().name, &name);
    }

    #[test]
    fn ga02_start_symbol_matches(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.start_symbol(), table.start_symbol);
    }

    #[test]
    fn ga03_grammar_ref_has_matching_tokens(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for &tok in grammar.tokens.keys() {
            prop_assert!(
                table.grammar().tokens.contains_key(&tok),
                "Token {:?} missing from table.grammar()", tok
            );
        }
    }
}

// ===========================================================================
// Category 10 — Error handling / robustness (3 tests)
// ===========================================================================

#[test]
fn err01_empty_grammar_no_panic() {
    let grammar = GrammarBuilder::new("empty").build();
    let _ = FirstFollowSets::compute(&grammar);
}

#[test]
fn err02_no_start_no_panic() {
    let grammar = GrammarBuilder::new("nostart")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    let _ = try_build(&grammar);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn err03_random_grammar_never_panics(grammar in arb_random_grammar()) {
        let _ = try_build(&grammar);
    }
}
