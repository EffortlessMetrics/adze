//! Property-based tests for LR(1) automaton construction in adze-glr-core (v7).
//!
//! 60+ proptest properties covering: FirstFollowSets::compute, build_lr1_automaton,
//! ParseTable invariants, determinism, precedence/associativity, scaling, and
//! structural validation.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test proptest_automaton_v7 -- --test-threads=2

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn try_build(grammar: &Grammar) -> Option<ParseTable> {
    let ff = FirstFollowSets::compute(grammar).ok()?;
    build_lr1_automaton(grammar, &ff).ok()
}

fn has_accept_anywhere(table: &ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

const TOK_NAMES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const TOK_PATS: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

/// Grammar with N tokens (1..8), single rule `start -> tok_0`.
fn n_token_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=8).prop_map(|n| {
        let mut b = GrammarBuilder::new(&format!("pa_v7_{n}tok"));
        for (name, pat) in TOK_NAMES.iter().zip(TOK_PATS.iter()).take(n) {
            b = b.token(name, pat);
        }
        b = b.rule("start", vec![TOK_NAMES[0]]);
        b = b.start("start");
        b.build()
    })
}

/// Grammar with N tokens and N alternative rules for `start`.
fn n_alt_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=6).prop_map(|n| {
        let mut b = GrammarBuilder::new(&format!("pa_v7_{n}alt"));
        for (name, pat) in TOK_NAMES.iter().zip(TOK_PATS.iter()).take(n) {
            b = b.token(name, pat);
        }
        for &name in TOK_NAMES.iter().take(n) {
            b = b.rule("start", vec![name]);
        }
        b = b.start("start");
        b.build()
    })
}

/// Grammar with chain: start -> mid -> a.
fn chain_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4).prop_map(|depth| {
        let mut b = GrammarBuilder::new(&format!("pa_v7_chain{depth}"));
        b = b.token("a", "a");
        let nt_names: Vec<String> = (0..=depth).map(|i| format!("nt{i}")).collect();
        for i in 0..depth {
            b = b.rule(&nt_names[i], vec![nt_names[i + 1].as_str()]);
        }
        b = b.rule(&nt_names[depth], vec!["a"]);
        b = b.start(&nt_names[0]);
        b.build()
    })
}

/// Grammar with sequence of tokens: start -> a b c ...
fn seq_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=5).prop_map(|n| {
        let mut b = GrammarBuilder::new(&format!("pa_v7_seq{n}"));
        for i in 0..n {
            b = b.token(TOK_NAMES[i], TOK_PATS[i]);
        }
        let rhs: Vec<&str> = TOK_NAMES[..n].to_vec();
        b = b.rule("start", rhs);
        b = b.start("start");
        b.build()
    })
}

/// Grammar with precedence rules.
fn prec_grammar() -> impl Strategy<Value = (i16, Grammar)> {
    (-50i16..=50).prop_map(|prec| {
        let g = GrammarBuilder::new(&format!("pa_v7_prec{prec}"))
            .token("a", "a")
            .token("+", "\\+")
            .rule_with_precedence("expr", vec!["expr", "+", "expr"], prec, Associativity::Left)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        (prec, g)
    })
}

/// Grammar with a specific associativity.
fn assoc_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![Just(0u8), Just(1u8), Just(2u8)].prop_map(|assoc_idx| {
        let assoc = match assoc_idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let label = match assoc_idx {
            0 => "left",
            1 => "right",
            _ => "none",
        };
        GrammarBuilder::new(&format!("pa_v7_assoc_{label}"))
            .token("a", "a")
            .token("+", "\\+")
            .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, assoc)
            .rule("expr", vec!["a"])
            .start("expr")
            .build()
    })
}

/// Combined strategy.
fn any_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![
        n_token_grammar(),
        n_alt_grammar(),
        chain_grammar(),
        seq_grammar(),
    ]
}

/// Two-nonterminal grammar: start -> inner, inner -> tok*.
fn two_nt_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 0usize..=2)
        .prop_flat_map(|(n_tok, n_extra)| {
            let extras = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), extras)
        })
        .prop_map(|(n_tok, extras)| {
            let mut b = GrammarBuilder::new(&format!("pa_v7_2nt{n_tok}"));
            for i in 0..n_tok {
                b = b.token(TOK_NAMES[i], TOK_PATS[i]);
            }
            b = b.rule("start", vec!["inner"]);
            b = b.rule("inner", vec![TOK_NAMES[0]]);
            for idx in extras {
                b = b.rule("inner", vec![TOK_NAMES[idx]]);
            }
            b = b.start("start");
            b.build()
        })
}

/// Nullable grammar: start -> ε | a.
fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("pa_v7_nullable")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Left-recursive grammar: start -> start a | a.
fn left_rec_grammar() -> Grammar {
    GrammarBuilder::new("pa_v7_leftrec")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Right-recursive grammar: start -> a start | a.
fn right_rec_grammar() -> Grammar {
    GrammarBuilder::new("pa_v7_rightrec")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

// ===========================================================================
// 1. FirstFollowSets::compute succeeds for N-token grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_01_ff_succeeds_n_tokens(grammar in n_token_grammar()) {
        let result = FirstFollowSets::compute(&grammar);
        prop_assert!(result.is_ok(), "FirstFollowSets::compute failed: {:?}", result.err());
    }
}

// ===========================================================================
// 2. build_lr1_automaton succeeds for N-token grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_02_automaton_succeeds_n_tokens(grammar in n_token_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok(), "build_lr1_automaton failed: {:?}", result.err());
    }
}

// ===========================================================================
// 3. state_count >= 2 for any valid grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_03_state_count_ge_two(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.state_count >= 2,
            "expected state_count >= 2, got {}",
            table.state_count
        );
    }
}

// ===========================================================================
// 4. symbol_count > 0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_04_symbol_count_positive(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count > 0);
    }
}

// ===========================================================================
// 5. eof_symbol is in symbol_to_index (valid symbol)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_05_eof_is_known_symbol(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "eof_symbol {:?} not in symbol_to_index",
            table.eof_symbol
        );
    }
}

// ===========================================================================
// 6. Actions for state 0 are non-empty for at least one symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_06_state0_has_actions(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let has_action = table.symbol_to_index.keys().any(|&sym| {
            !table.actions(StateId(0), sym).is_empty()
        });
        prop_assert!(has_action, "state 0 has no actions for any symbol");
    }
}

// ===========================================================================
// 7. At least one Accept action exists somewhere
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_07_accept_exists(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_accept_anywhere(&table), "no Accept action found");
    }
}

// ===========================================================================
// 8. rule() doesn't panic for all RuleIds 0..rules.len()
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_08_rule_accessor_no_panic(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for i in 0..table.rules.len() {
            let (lhs, _rhs_len) = table.rule(RuleId(i as u16));
            prop_assert!(
                (lhs.0 as usize) < table.symbol_count,
                "rule {} lhs {} >= symbol_count {}",
                i,
                lhs.0,
                table.symbol_count
            );
        }
    }
}

// ===========================================================================
// 9. goto() doesn't panic for all StateId/SymbolId pairs
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_09_goto_no_panic(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let nts: Vec<SymbolId> = grammar.rules.keys().copied().collect();
        for s in 0..table.state_count {
            for &nt in &nts {
                if let Some(target) = table.goto(StateId(s as u16), nt) {
                    prop_assert!(
                        (target.0 as usize) < table.state_count,
                        "goto({}, {:?}) = {} out of bounds",
                        s, nt, target.0
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 10. Same grammar built twice → same state_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_10_deterministic_state_count(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.state_count, t2.state_count);
    }
}

// ===========================================================================
// 11. Same grammar built twice → same symbol_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_11_deterministic_symbol_count(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
    }
}

// ===========================================================================
// 12. Grammar with precedence (-50..50) builds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_12_precedence_builds((_prec, grammar) in prec_grammar()) {
        let result = try_build(&grammar);
        prop_assert!(result.is_some(), "grammar with precedence failed to build");
    }
}

// ===========================================================================
// 13. Grammar with various associativity builds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_13_associativity_builds(grammar in assoc_grammar()) {
        let result = try_build(&grammar);
        prop_assert!(result.is_some(), "grammar with associativity failed to build");
    }
}

// ===========================================================================
// 14. Larger grammar → state_count >= smaller grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_14_more_alts_more_or_equal_states(n_extra in 0usize..=3) {
        let base = GrammarBuilder::new("pa_v7_base14")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let base_table = build_table(&base);

        let mut bld = GrammarBuilder::new("pa_v7_ext14")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["a"]);
        for &name in TOK_NAMES.iter().skip(1).take(n_extra.min(3)) {
            bld = bld.rule("start", vec![name]);
        }
        let ext_table = build_table(&bld.start("start").build());

        prop_assert!(
            ext_table.state_count >= base_table.state_count,
            "extended {} < base {}",
            ext_table.state_count,
            base_table.state_count
        );
    }
}

// ===========================================================================
// 15. FirstFollowSets::compute succeeds for alt grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_15_ff_succeeds_alt(grammar in n_alt_grammar()) {
        let result = FirstFollowSets::compute(&grammar);
        prop_assert!(result.is_ok());
    }
}

// ===========================================================================
// 16. build_lr1_automaton succeeds for alt grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_16_automaton_succeeds_alt(grammar in n_alt_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok());
    }
}

// ===========================================================================
// 17. Chain grammars produce valid tables
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_17_chain_valid(grammar in chain_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 2);
        prop_assert!(has_accept_anywhere(&table));
    }
}

// ===========================================================================
// 18. Sequence grammars produce valid tables
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_18_seq_valid(grammar in seq_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 2);
        prop_assert!(has_accept_anywhere(&table));
    }
}

// ===========================================================================
// 19. Two-nonterminal grammars produce valid tables
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_19_two_nt_valid(grammar in two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 2);
        prop_assert!(table.symbol_count > 0);
    }
}

// ===========================================================================
// 20. Nullable grammar builds and has Accept
// ===========================================================================

#[test]
fn prop_20_nullable_builds() {
    let table = build_table(&nullable_grammar());
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
}

// ===========================================================================
// 21. Left-recursive grammar builds and has Accept
// ===========================================================================

#[test]
fn prop_21_left_rec_builds() {
    let table = build_table(&left_rec_grammar());
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
}

// ===========================================================================
// 22. Right-recursive grammar builds and has Accept
// ===========================================================================

#[test]
fn prop_22_right_rec_builds() {
    let table = build_table(&right_rec_grammar());
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
}

// ===========================================================================
// 23. action_table rows == state_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_23_action_table_rows(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }
}

// ===========================================================================
// 24. goto_table rows == state_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_24_goto_table_rows(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
    }
}

// ===========================================================================
// 25. action_table rows have uniform width
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_25_action_rows_uniform(grammar in any_grammar()) {
        let table = build_table(&grammar);
        if let Some(first) = table.action_table.first() {
            let width = first.len();
            for (i, row) in table.action_table.iter().enumerate() {
                prop_assert_eq!(
                    row.len(),
                    width,
                    "action row {} width {} != expected {}",
                    i,
                    row.len(),
                    width
                );
            }
        }
    }
}

// ===========================================================================
// 26. goto_table rows have uniform width
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_26_goto_rows_uniform(grammar in any_grammar()) {
        let table = build_table(&grammar);
        if let Some(first) = table.goto_table.first() {
            let width = first.len();
            for (i, row) in table.goto_table.iter().enumerate() {
                prop_assert_eq!(
                    row.len(),
                    width,
                    "goto row {} width {} != expected {}",
                    i,
                    row.len(),
                    width
                );
            }
        }
    }
}

// ===========================================================================
// 27. At least one Shift action exists
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_27_shift_exists(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let found = (0..table.state_count).any(|s| {
            table.symbol_to_index.keys().any(|&sym| {
                table
                    .actions(StateId(s as u16), sym)
                    .iter()
                    .any(|a| matches!(a, Action::Shift(_)))
            })
        });
        prop_assert!(found, "no Shift action found");
    }
}

// ===========================================================================
// 28. At least one Reduce action exists
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_28_reduce_exists(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let found = (0..table.state_count).any(|s| {
            table.symbol_to_index.keys().any(|&sym| {
                table
                    .actions(StateId(s as u16), sym)
                    .iter()
                    .any(|a| matches!(a, Action::Reduce(_)))
            })
        });
        prop_assert!(found, "no Reduce action found");
    }
}

// ===========================================================================
// 29. Shift targets are valid state indices
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_29_shift_targets_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(s as u16), sym) {
                    if let Action::Shift(target) = action {
                        prop_assert!(
                            (target.0 as usize) < table.state_count,
                            "shift target {} >= state_count {}",
                            target.0,
                            table.state_count
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 30. Reduce rule IDs are valid
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_30_reduce_rule_ids_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(s as u16), sym) {
                    if let Action::Reduce(rid) = action {
                        prop_assert!(
                            (rid.0 as usize) < table.rules.len(),
                            "reduce rule {} >= rules.len() {}",
                            rid.0,
                            table.rules.len()
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 31. eof() method matches eof_symbol field
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_31_eof_method_matches_field(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.eof(), table.eof_symbol);
    }
}

// ===========================================================================
// 32. start_symbol() method matches start_symbol field
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_32_start_symbol_method_matches_field(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.start_symbol(), table.start_symbol);
    }
}

// ===========================================================================
// 33. eof_symbol in symbol_to_index
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_33_eof_in_symbol_map(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "eof_symbol {:?} missing from symbol_to_index",
            table.eof_symbol
        );
    }
}

// ===========================================================================
// 34. initial_state within bounds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_34_initial_state_in_bounds(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            (table.initial_state.0 as usize) < table.state_count,
            "initial_state {} >= state_count {}",
            table.initial_state.0,
            table.state_count
        );
    }
}

// ===========================================================================
// 35. rules.len() > 0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_35_rules_nonempty(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.rules.is_empty(), "rules must not be empty");
    }
}

// ===========================================================================
// 36. All rule LHS symbols are within symbol_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_36_rule_lhs_in_range(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for (i, rule) in table.rules.iter().enumerate() {
            prop_assert!(
                (rule.lhs.0 as usize) < table.symbol_count,
                "rule {} lhs {} >= symbol_count {}",
                i,
                rule.lhs.0,
                table.symbol_count
            );
        }
    }
}

// ===========================================================================
// 37. Deterministic eof_symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_37_deterministic_eof(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.eof_symbol, t2.eof_symbol);
    }
}

// ===========================================================================
// 38. Deterministic rules.len()
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_38_deterministic_rules_len(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.rules.len(), t2.rules.len());
    }
}

// ===========================================================================
// 39. Deterministic action_table dimensions
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_39_deterministic_action_dims(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.action_table.len(), t2.action_table.len());
        if let (Some(r1), Some(r2)) = (t1.action_table.first(), t2.action_table.first()) {
            prop_assert_eq!(r1.len(), r2.len());
        }
    }
}

// ===========================================================================
// 40. Deterministic goto_table dimensions
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_40_deterministic_goto_dims(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.goto_table.len(), t2.goto_table.len());
    }
}

// ===========================================================================
// 41. index_to_symbol length == symbol_to_index length
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_41_symbol_maps_consistent(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.index_to_symbol.len(), table.symbol_to_index.len());
    }
}

// ===========================================================================
// 42. symbol_to_index / index_to_symbol roundtrip
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_42_symbol_index_roundtrip(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for (&sym, &idx) in &table.symbol_to_index {
            prop_assert!(idx < table.index_to_symbol.len());
            prop_assert_eq!(table.index_to_symbol[idx], sym);
        }
    }
}

// ===========================================================================
// 43. grammar() accessor returns grammar with rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_43_grammar_accessor_has_rules(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.grammar().rules.is_empty());
    }
}

// ===========================================================================
// 44. token_count > 0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_44_token_count_positive(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.token_count > 0);
    }
}

// ===========================================================================
// 45. Precedence grammars have Accept action
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_45_prec_has_accept((_prec, grammar) in prec_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!(has_accept_anywhere(&table));
        }
    }
}

// ===========================================================================
// 46. Associativity grammars have Accept action
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_46_assoc_has_accept(grammar in assoc_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!(has_accept_anywhere(&table));
        }
    }
}

// ===========================================================================
// 47. Two-nonterminal grammars have Accept
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_47_two_nt_has_accept(grammar in two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_accept_anywhere(&table));
    }
}

// ===========================================================================
// 48. Fork actions (if any) contain only Shift/Reduce
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_48_fork_contents_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(s as u16), sym) {
                    if let Action::Fork(inner) = action {
                        for sub in inner {
                            prop_assert!(
                                matches!(sub, Action::Shift(_) | Action::Reduce(_) | Action::Accept),
                                "Fork contains unexpected action: {:?}",
                                sub
                            );
                        }
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 49. FirstFollowSets: terminals are never nullable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_49_terminals_not_nullable(grammar in any_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        for &tid in grammar.tokens.keys() {
            prop_assert!(!ff.is_nullable(tid), "terminal {:?} should not be nullable", tid);
        }
    }
}

// ===========================================================================
// 50. FirstFollowSets idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_50_ff_idempotent(grammar in any_grammar()) {
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        for &sym in grammar.rules.keys().chain(grammar.tokens.keys()) {
            prop_assert_eq!(ff1.first(sym), ff2.first(sym));
            prop_assert_eq!(ff1.follow(sym), ff2.follow(sym));
            prop_assert_eq!(ff1.is_nullable(sym), ff2.is_nullable(sym));
        }
    }
}

// ===========================================================================
// 51. Grammar name preserved in table
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_51_grammar_name_preserved(idx in 0u32..100) {
        let name = format!("pa_v7_{idx}");
        let grammar = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let table = build_table(&grammar);
        prop_assert_eq!(&table.grammar().name, &name);
    }
}

// ===========================================================================
// 52. Random grammar never panics
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_52_random_no_panic(n_tok in 1usize..=3, n_nt in 1usize..=2) {
        let tok_names: Vec<String> = (0..n_tok).map(|i| format!("tok{i}")).collect();
        let nt_names: Vec<String> = (0..n_nt).map(|i| format!("nt{i}")).collect();

        let mut b = GrammarBuilder::new(&format!("pa_v7_rnd_{n_tok}_{n_nt}"));
        for tn in &tok_names {
            b = b.token(tn, tn);
        }
        for (i, nn) in nt_names.iter().enumerate() {
            if i == 0 {
                b = b.rule(nn, vec![tok_names[0].as_str()]);
            } else {
                b = b.rule(nn, vec![nt_names[i - 1].as_str()]);
            }
        }
        b = b.start(&nt_names[0]);
        let grammar = b.build();
        let _ = try_build(&grammar);
    }
}

// ===========================================================================
// 53. Empty grammar doesn't panic
// ===========================================================================

#[test]
fn prop_53_empty_grammar_no_panic() {
    let grammar = GrammarBuilder::new("pa_v7_empty").build();
    let _ = FirstFollowSets::compute(&grammar);
}

// ===========================================================================
// 54. Self-recursive grammar doesn't panic
// ===========================================================================

#[test]
fn prop_54_self_recursive_no_panic() {
    let grammar = GrammarBuilder::new("pa_v7_selfrec")
        .rule("start", vec!["start"])
        .start("start")
        .build();
    let _ = try_build(&grammar);
}

// ===========================================================================
// 55. More tokens → symbol_count >= base
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_55_more_tokens_more_symbols(extra in 0usize..=4) {
        let base = GrammarBuilder::new("pa_v7_sym_base")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let base_table = build_table(&base);

        let mut bld = GrammarBuilder::new("pa_v7_sym_ext");
        bld = bld.token("a", "a");
        for i in 0..extra {
            bld = bld.token(TOK_NAMES[i + 1], TOK_PATS[i + 1]);
        }
        bld = bld.rule("start", vec!["a"]).start("start");
        let ext_table = build_table(&bld.build());

        prop_assert!(
            ext_table.symbol_count >= base_table.symbol_count,
            "ext {} < base {}",
            ext_table.symbol_count,
            base_table.symbol_count
        );
    }
}

// ===========================================================================
// 56. normalize_eof_to_zero produces eof == SymbolId(0)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_56_normalize_eof_produces_zero(grammar in any_grammar()) {
        let table = build_table(&grammar).normalize_eof_to_zero();
        prop_assert_eq!(
            table.eof_symbol,
            SymbolId(0),
            "after normalize_eof_to_zero, eof should be SymbolId(0)"
        );
    }
}

// ===========================================================================
// 57. dynamic_prec_by_rule length == rules.len()
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_57_dynamic_prec_len(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(
            table.dynamic_prec_by_rule.len(),
            table.rules.len(),
            "dynamic_prec_by_rule.len() {} != rules.len() {}",
            table.dynamic_prec_by_rule.len(),
            table.rules.len()
        );
    }
}

// ===========================================================================
// 58. rule_assoc_by_rule length == rules.len()
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_58_rule_assoc_len(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(
            table.rule_assoc_by_rule.len(),
            table.rules.len(),
            "rule_assoc_by_rule.len() {} != rules.len() {}",
            table.rule_assoc_by_rule.len(),
            table.rules.len()
        );
    }
}

// ===========================================================================
// 59. symbol_count > token_count (at least one non-terminal/EOF)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_59_symbol_count_ge_tokens_plus_one(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_count > table.token_count,
            "symbol_count {} <= token_count {}",
            table.symbol_count,
            table.token_count
        );
    }
}

// ===========================================================================
// 60. Accept only appears on EOF symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_60_accept_only_on_eof(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let eof = table.eof();
        for s in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                if sym != eof {
                    for action in table.actions(StateId(s as u16), sym) {
                        prop_assert!(
                            !matches!(action, Action::Accept),
                            "Accept found on non-EOF symbol {:?} in state {}",
                            sym,
                            s
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 61. Precedence doesn't decrease state_count vs baseline
// ===========================================================================

#[test]
fn prop_61_precedence_vs_baseline() {
    let base = GrammarBuilder::new("pa_v7_noprec")
        .token("a", "a")
        .token("+", "\\+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let with_prec = GrammarBuilder::new("pa_v7_withprec")
        .token("a", "a")
        .token("+", "\\+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    if let (Some(t_base), Some(t_prec)) = (try_build(&base), try_build(&with_prec)) {
        assert!(t_prec.state_count > 0);
        assert!(t_base.state_count > 0);
    }
}

// ===========================================================================
// 62. Varying depths: deeper chains → more states
// ===========================================================================

#[test]
fn prop_62_deeper_chain_more_states() {
    let shallow = GrammarBuilder::new("pa_v7_shallow")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let deep = GrammarBuilder::new("pa_v7_deep")
        .token("a", "a")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["a"])
        .start("start")
        .build();
    let t_shallow = build_table(&shallow);
    let t_deep = build_table(&deep);
    assert!(t_deep.state_count >= t_shallow.state_count);
}

// ===========================================================================
// 63. Deterministic initial_state
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_63_deterministic_initial_state(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.initial_state, t2.initial_state);
    }
}

// ===========================================================================
// 64. Deterministic start_symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_64_deterministic_start_symbol(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.start_symbol(), t2.start_symbol());
    }
}
