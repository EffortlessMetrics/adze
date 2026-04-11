#![cfg(feature = "test-api")]
//! Property-based tests for LR(1) automaton construction in adze-glr-core.
//!
//! Run with:
//! ```bash
//! cargo test -p adze-glr-core --test proptest_automaton_v3 --features test-api
//! ```

use adze_glr_core::test_helpers::test as th;
use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a parse table from a grammar via the standard pipeline.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("build_lr1_automaton failed")
}

/// Try to build a table, returning None on failure.
fn try_build(grammar: &Grammar) -> Option<ParseTable> {
    let ff = FirstFollowSets::compute(grammar).ok()?;
    build_lr1_automaton(grammar, &ff).ok()
}

/// Check if any state in the table has an Accept action on EOF.
fn has_any_accept(table: &ParseTable) -> bool {
    (0..table.state_count).any(|s| th::has_accept_on_eof(table, s))
}

/// Collect all non-terminal SymbolIds from a grammar.
fn nonterminal_ids(grammar: &Grammar) -> Vec<SymbolId> {
    grammar.rules.keys().copied().collect()
}

// ---------------------------------------------------------------------------
// Fixed grammar constructors
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

/// S → S a | a  (left-recursive)
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S → T, T → a  (chain)
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["a"])
        .start("s")
        .build()
}

/// E → E + E | E * E | a  (precedence)
fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("a", "a")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build()
}

/// S → a b c  (sequence)
fn sequence_grammar() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build()
}

/// S → T, T → U, U → a  (deep chain)
fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["u"])
        .rule("u", vec!["a"])
        .start("s")
        .build()
}

/// S → a S | a  (right-recursive)
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rightrec")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// S → a | b | c | d | e  (wide alternatives)
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

/// S → T U, T → a, U → b  (two nonterminals in sequence)
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

/// Strategy yielding one of the fixed grammars.
fn arb_fixed_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![
        Just(minimal_grammar()),
        Just(two_alt_grammar()),
        Just(nullable_grammar()),
        Just(left_recursive_grammar()),
        Just(chain_grammar()),
        Just(precedence_grammar()),
        Just(sequence_grammar()),
        Just(deep_chain_grammar()),
        Just(right_recursive_grammar()),
        Just(wide_alt_grammar()),
        Just(two_nt_seq_grammar()),
    ]
}

// ---------------------------------------------------------------------------
// Proptest strategies — random grammar generators
// ---------------------------------------------------------------------------

const TOKEN_NAMES: &[&str] = &["a", "b", "c", "d", "e", "f"];
const TOKEN_PATTERNS: &[&str] = &["a", "b", "c", "d", "e", "f"];
const NT_NAMES: &[&str] = &["s", "t", "u", "v", "w", "x"];

/// Build a grammar from numeric parameters.
fn build_grammar(n_tok: usize, _n_nt: usize, productions: &[Vec<Vec<usize>>]) -> Grammar {
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
    builder = builder.start(NT_NAMES[0]);
    builder.build()
}

/// Random grammar with 1-3 tokens and 1-3 non-terminals.
fn arb_simple_grammar() -> impl Strategy<Value = Grammar> {
    (1..=3usize, 1..=3usize).prop_flat_map(|(n_tok, n_nt)| {
        let n_tok2 = n_tok;
        let n_nt2 = n_nt;
        proptest::collection::vec(
            proptest::collection::vec(proptest::collection::vec(0..(n_tok2 + n_nt2), 1..=3), 1..=3),
            n_nt2..=n_nt2,
        )
        .prop_map(move |productions| build_grammar(n_tok, n_nt, &productions))
    })
}

/// Random valid grammar: 1-5 tokens, S → one of them, possibly more alternatives.
fn arb_valid_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=5, 0usize..=2)
        .prop_flat_map(|(n_tok, n_extra)| {
            let rhs_indices = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), rhs_indices)
        })
        .prop_map(|(n_tok, rhs_indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut bld = GrammarBuilder::new("rand_grammar");
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

/// Random grammar with two nonterminals: S → A, A → tok*.
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
// Category 1: ParseTable state_count is always > 0 for valid grammars (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn state_count_positive_random(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0, "state_count must be > 0");
    }

    #[test]
    fn state_count_positive_fixed(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!(table.state_count > 0);
        }
    }

    #[test]
    fn state_count_positive_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0);
    }
}

#[test]
fn state_count_positive_minimal() {
    let table = build_table(&minimal_grammar());
    assert!(table.state_count > 0);
}

#[test]
fn state_count_positive_chain() {
    let table = build_table(&chain_grammar());
    assert!(table.state_count > 0);
}

// ===========================================================================
// Category 2: First/Follow set determinism (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn ff_idempotent_random(grammar in arb_valid_grammar()) {
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        for &sym in grammar.rules.keys().chain(grammar.tokens.keys()) {
            prop_assert_eq!(ff1.first(sym), ff2.first(sym),
                "FIRST not idempotent for {:?}", sym);
            prop_assert_eq!(ff1.follow(sym), ff2.follow(sym),
                "FOLLOW not idempotent for {:?}", sym);
            prop_assert_eq!(ff1.is_nullable(sym), ff2.is_nullable(sym));
        }
    }

    #[test]
    fn ff_idempotent_fixed(grammar in arb_fixed_grammar()) {
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        for &sym in grammar.rules.keys().chain(grammar.tokens.keys()) {
            prop_assert_eq!(ff1.first(sym), ff2.first(sym));
            prop_assert_eq!(ff1.follow(sym), ff2.follow(sym));
        }
    }

    #[test]
    fn ff_idempotent_two_nt(grammar in arb_two_nt_grammar()) {
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        for &sym in grammar.rules.keys() {
            prop_assert_eq!(ff1.first(sym), ff2.first(sym));
        }
    }

    #[test]
    fn ff_no_panic_random(grammar in arb_simple_grammar()) {
        let _ = FirstFollowSets::compute(&grammar);
    }

    #[test]
    fn ff_terminals_never_nullable(grammar in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        for &tid in grammar.tokens.keys() {
            prop_assert!(!ff.is_nullable(tid),
                "Terminal {:?} should not be nullable", tid);
        }
    }
}

// ===========================================================================
// Category 3: Table construction determinism (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn table_deterministic_state_count(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.state_count, t2.state_count);
    }

    #[test]
    fn table_deterministic_symbol_count(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
    }

    #[test]
    fn table_deterministic_rules_len(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.rules.len(), t2.rules.len());
    }

    #[test]
    fn table_deterministic_action_len(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.action_table.len(), t2.action_table.len());
    }

    #[test]
    fn table_deterministic_goto_len(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.goto_table.len(), t2.goto_table.len());
    }
}

// ===========================================================================
// Category 4: State 0 always exists (3)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn initial_state_in_bounds_random(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!((table.initial_state.0 as usize) < table.state_count,
            "initial_state {} out of bounds (state_count={})",
            table.initial_state.0, table.state_count);
    }

    #[test]
    fn initial_state_in_bounds_fixed(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!((table.initial_state.0 as usize) < table.state_count);
        }
    }

    #[test]
    fn initial_state_in_bounds_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!((table.initial_state.0 as usize) < table.state_count);
    }
}

// ===========================================================================
// Category 5: EOF symbol is consistent (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn eof_in_symbol_to_index(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "EOF symbol {:?} not in symbol_to_index", table.eof_symbol
        );
    }

    #[test]
    fn eof_consistent_random(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.eof_symbol, t2.eof_symbol, "EOF symbol differs across builds");
    }

    #[test]
    fn eof_consistent_fixed(grammar in arb_fixed_grammar()) {
        if let Some(t1) = try_build(&grammar) && let Some(t2) = try_build(&grammar) {
            prop_assert_eq!(t1.eof_symbol, t2.eof_symbol);
        }
    }

    #[test]
    fn eof_method_matches_field(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.eof(), table.eof_symbol,
            "eof() != eof_symbol field");
    }

    #[test]
    fn eof_method_matches_field_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.eof(), table.eof_symbol);
    }
}

// ===========================================================================
// Category 6: Accept action exists somewhere (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn accept_exists_random(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_any_accept(&table),
            "No Accept action found in any state on EOF");
    }

    #[test]
    fn accept_exists_fixed(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!(has_any_accept(&table));
        }
    }

    #[test]
    fn accept_exists_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_any_accept(&table));
    }
}

#[test]
fn accept_exists_minimal() {
    let table = build_table(&minimal_grammar());
    assert!(has_any_accept(&table));
}

#[test]
fn accept_exists_deep_chain() {
    let table = build_table(&deep_chain_grammar());
    assert!(has_any_accept(&table));
}

// ===========================================================================
// Category 7: Goto table returns valid StateIds (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn goto_returns_valid_state_random(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        let nts = nonterminal_ids(&grammar);
        for state_idx in 0..table.state_count {
            let sid = adze_ir::StateId(state_idx as u16);
            for &nt in &nts {
                if let Some(target) = table.goto(sid, nt) {
                    prop_assert!((target.0 as usize) < table.state_count,
                        "goto({}, {:?}) = {} out of bounds (state_count={})",
                        state_idx, nt, target.0, table.state_count);
                }
            }
        }
    }

    #[test]
    fn goto_returns_valid_state_fixed(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            let nts = nonterminal_ids(&grammar);
            for state_idx in 0..table.state_count {
                let sid = adze_ir::StateId(state_idx as u16);
                for &nt in &nts {
                    if let Some(target) = table.goto(sid, nt) {
                        prop_assert!((target.0 as usize) < table.state_count);
                    }
                }
            }
        }
    }

    #[test]
    fn goto_returns_valid_state_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        let nts = nonterminal_ids(&grammar);
        for state_idx in 0..table.state_count {
            let sid = adze_ir::StateId(state_idx as u16);
            for &nt in &nts {
                if let Some(target) = table.goto(sid, nt) {
                    prop_assert!((target.0 as usize) < table.state_count);
                }
            }
        }
    }

    #[test]
    fn goto_table_rows_match_state_count(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.goto_table.len(), table.state_count,
            "goto_table.len() != state_count");
    }

    #[test]
    fn goto_table_uniform_width(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        if let Some(first_row) = table.goto_table.first() {
            let width = first_row.len();
            for (i, row) in table.goto_table.iter().enumerate() {
                prop_assert_eq!(row.len(), width,
                    "goto row {} has width {} but expected {}", i, row.len(), width);
            }
        }
    }
}

// ===========================================================================
// Category 8: Various grammar topologies produce valid tables (10)
// ===========================================================================

#[test]
fn topology_minimal() {
    let table = build_table(&minimal_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn topology_two_alt() {
    let table = build_table(&two_alt_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_nullable() {
    let table = build_table(&nullable_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_left_recursive() {
    let table = build_table(&left_recursive_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_right_recursive() {
    let table = build_table(&right_recursive_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_chain() {
    let table = build_table(&chain_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_deep_chain() {
    let table = build_table(&deep_chain_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_sequence() {
    let table = build_table(&sequence_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_precedence() {
    let table = build_table(&precedence_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

#[test]
fn topology_wide_alternatives() {
    let table = build_table(&wide_alt_grammar());
    assert!(table.state_count > 0);
    assert!(has_any_accept(&table));
}

// ===========================================================================
// Category 9: Table properties scale with grammar size (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn action_table_len_matches_state_count(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }

    #[test]
    fn action_rows_uniform_width(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        if let Some(first_row) = table.action_table.first() {
            let width = first_row.len();
            for (i, row) in table.action_table.iter().enumerate() {
                prop_assert_eq!(row.len(), width,
                    "action row {} has width {} but expected {}", i, row.len(), width);
            }
        }
    }

    #[test]
    fn symbol_to_index_roundtrip(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for (&sym, &idx) in &table.symbol_to_index {
            prop_assert!(idx < table.index_to_symbol.len(),
                "index {} out of bounds", idx);
            prop_assert_eq!(table.index_to_symbol[idx], sym,
                "roundtrip failed for {:?}", sym);
        }
    }

    #[test]
    fn more_alternatives_no_fewer_states(n_extra in 0usize..=3) {
        let base = GrammarBuilder::new("base")
            .token("t0", "t0")
            .token("t1", "t1")
            .token("t2", "t2")
            .rule("S", vec!["t0"])
            .start("S")
            .build();
        let base_table = build_table(&base);

        let all_toks = ["t0", "t1", "t2"];
        let mut bld = GrammarBuilder::new("ext")
            .token("t0", "t0")
            .token("t1", "t1")
            .token("t2", "t2")
            .rule("S", vec!["t0"]);
        for &tok in &all_toks[1..=(n_extra.min(2))] {
            bld = bld.rule("S", vec![tok]);
        }
        let ext_table = build_table(&bld.start("S").build());

        prop_assert!(
            ext_table.state_count >= base_table.state_count,
            "extended {} < base {}", ext_table.state_count, base_table.state_count
        );
    }

    #[test]
    fn grammar_name_preserved(suffix in 0u32..50) {
        let name = format!("g_{suffix}");
        let grammar = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S")
            .build();
        let table = build_table(&grammar);
        prop_assert_eq!(&table.grammar().name, &name);
    }
}

// ===========================================================================
// Category 10: Error handling for invalid grammars (5)
// ===========================================================================

#[test]
fn error_empty_grammar_no_panic() {
    let grammar = GrammarBuilder::new("empty").build();
    let result = FirstFollowSets::compute(&grammar);
    // Should either succeed or return an error, but never panic.
    let _ = result;
}

#[test]
fn error_no_start_symbol_no_panic() {
    let grammar = GrammarBuilder::new("nostart")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    // Omitting .start() — pipeline should handle gracefully
    let _ = try_build(&grammar);
}

#[test]
fn error_unreachable_nonterminal_no_panic() {
    // T is defined but never referenced from S
    let grammar = GrammarBuilder::new("unreachable")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("t", vec!["b"])
        .start("s")
        .build();
    let _ = try_build(&grammar);
}

#[test]
fn error_self_recursive_only_no_panic() {
    // S → S (purely self-recursive, no base case)
    let grammar = GrammarBuilder::new("selfrec")
        .rule("s", vec!["s"])
        .start("s")
        .build();
    let _ = try_build(&grammar);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn random_grammar_never_panics(grammar in arb_simple_grammar()) {
        // The pipeline must not panic regardless of input.
        let _ = try_build(&grammar);
    }
}

// ===========================================================================
// Bonus: Additional structural invariants (3)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn rules_have_valid_lhs(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        let known_nts: BTreeSet<SymbolId> = table.nonterminal_to_index.keys().copied().collect();
        for rule in &table.rules {
            prop_assert!(
                known_nts.contains(&rule.lhs) || rule.lhs.0 == 0,
                "rule lhs {:?} not in nonterminal_to_index", rule.lhs
            );
        }
    }

    #[test]
    fn shift_targets_in_bounds(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            for state_row in &table.action_table {
                for cell in state_row {
                    for action in cell {
                        if let Action::Shift(target) = action {
                            prop_assert!(
                                (target.0 as usize) < table.state_count,
                                "Shift target {} out of bounds", target.0
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn reduce_rule_indices_in_bounds(grammar in arb_fixed_grammar()) {
        if let Some(table) = try_build(&grammar) {
            for state_row in &table.action_table {
                for cell in state_row {
                    for action in cell {
                        if let Action::Reduce(rule_id) = action {
                            prop_assert!(
                                (rule_id.0 as usize) < table.rules.len(),
                                "Reduce rule {} out of bounds (rules.len()={})",
                                rule_id.0, table.rules.len()
                            );
                        }
                    }
                }
            }
        }
    }
}
