//! Property-based tests for parse table construction in adze-glr-core (v9).
//!
//! 80+ tests covering: grammar construction, FIRST/FOLLOW computation,
//! `build_lr1_automaton` invariants, structural validation, determinism,
//! precedence, associativity, chain rules, and alternatives.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test proptest_parse_v9 -- --test-threads=2

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};
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

fn all_shift_targets(table: &ParseTable) -> Vec<StateId> {
    let mut targets = Vec::new();
    for s in 0..table.state_count {
        for &sym in table.index_to_symbol.iter() {
            for action in table.actions(StateId(s as u16), sym) {
                if let Action::Shift(target) = action {
                    targets.push(*target);
                }
            }
        }
    }
    targets
}

fn all_goto_targets(table: &ParseTable) -> Vec<StateId> {
    let mut targets = Vec::new();
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(StateId(s as u16), nt) {
                targets.push(target);
            }
        }
    }
    targets
}

fn collect_reduce_rule_ids(table: &ParseTable) -> Vec<RuleId> {
    let mut ids = Vec::new();
    for s in 0..table.state_count {
        for &sym in table.index_to_symbol.iter() {
            for action in table.actions(StateId(s as u16), sym) {
                if let Action::Reduce(rid) = action {
                    ids.push(*rid);
                }
            }
        }
    }
    ids
}

fn make_grammar(name: &str, tokens: &[&str], rules: &[(&str, Vec<&str>)], start: &str) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for &tok in tokens {
        b = b.token(tok, tok);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b.start(start).build()
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

const TOK_NAMES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const TOK_PATS: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

fn arb_token_count() -> impl Strategy<Value = usize> {
    1usize..8
}

fn arb_rule_count() -> impl Strategy<Value = usize> {
    1usize..6
}

/// Grammar with N tokens (1..8), single rule `start -> tok_0`.
fn n_token_grammar() -> impl Strategy<Value = Grammar> {
    arb_token_count().prop_map(|n| {
        let mut b = GrammarBuilder::new(&format!("pv9_{n}tok"));
        for i in 0..n {
            b = b.token(TOK_NAMES[i], TOK_PATS[i]);
        }
        b = b.rule("start", vec![TOK_NAMES[0]]);
        b = b.start("start");
        b.build()
    })
}

/// Grammar with N tokens and up to N alternative rules for `start`.
fn n_alt_grammar() -> impl Strategy<Value = Grammar> {
    arb_rule_count().prop_map(|n| {
        let tok_count = n.min(7) + 1;
        let mut b = GrammarBuilder::new(&format!("pv9_{n}alt"));
        for i in 0..tok_count {
            b = b.token(TOK_NAMES[i], TOK_PATS[i]);
        }
        for i in 0..n {
            let idx = i % tok_count;
            b = b.rule("start", vec![TOK_NAMES[idx]]);
        }
        b = b.start("start");
        b.build()
    })
}

/// Grammar with chain: start -> nt1 -> nt2 -> ... -> a.
fn chain_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4).prop_map(|depth| {
        let mut b = GrammarBuilder::new(&format!("pv9_chain{depth}"));
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
        let mut b = GrammarBuilder::new(&format!("pv9_seq{n}"));
        for i in 0..n {
            b = b.token(TOK_NAMES[i], TOK_PATS[i]);
        }
        let rhs: Vec<&str> = TOK_NAMES[..n].to_vec();
        b = b.rule("start", rhs);
        b = b.start("start");
        b.build()
    })
}

/// Grammar with precedence.
fn prec_grammar() -> impl Strategy<Value = (i16, Grammar)> {
    (-50i16..=50).prop_map(|prec| {
        let g = GrammarBuilder::new(&format!("pv9_prec{prec}"))
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
        GrammarBuilder::new(&format!("pv9_assoc_{label}"))
            .token("a", "a")
            .token("+", "\\+")
            .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, assoc)
            .rule("expr", vec!["a"])
            .start("expr")
            .build()
    })
}

/// Two-nonterminal grammar: start -> inner, inner -> tok*.
fn two_nt_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 0usize..=2)
        .prop_flat_map(|(n_tok, n_extra)| {
            let extras = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), extras)
        })
        .prop_map(|(n_tok, extras)| {
            let mut b = GrammarBuilder::new(&format!("pv9_2nt{n_tok}"));
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

/// Combined strategy.
fn any_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![
        n_token_grammar(),
        n_alt_grammar(),
        chain_grammar(),
        seq_grammar(),
        two_nt_grammar(),
    ]
}

// ===========================================================================
// 1. Any valid grammar → build_lr1_automaton succeeds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_01_automaton_succeeds(grammar in any_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok(), "build_lr1_automaton failed: {:?}", result.err());
    }
}

// ===========================================================================
// 2. state_count > 0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_02_state_count_positive(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0);
    }
}

// ===========================================================================
// 3. symbol_count > 0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_03_symbol_count_positive(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count > 0);
    }
}

// ===========================================================================
// 4. eof_symbol is a known symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_04_eof_is_known_symbol(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "eof_symbol {:?} not in symbol_to_index", table.eof_symbol
        );
    }
}

// ===========================================================================
// 5. state 0 is valid
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_05_state0_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 1, "must have at least state 0");
        // state 0 must appear in action_table
        prop_assert!(!table.action_table.is_empty(), "action_table must include state 0");
    }
}

// ===========================================================================
// 6. actions for state 0 exist
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_06_state0_has_actions(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let has_action = table.index_to_symbol.iter().any(|&sym| {
            !table.actions(StateId(0), sym).is_empty()
        });
        prop_assert!(has_action, "state 0 has no actions for any symbol");
    }
}

// ===========================================================================
// 7. All shift targets < state_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_07_shift_targets_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!(
                (target.0 as usize) < table.state_count,
                "shift target {} >= state_count {}", target.0, table.state_count
            );
        }
    }
}

// ===========================================================================
// 8. All goto targets < state_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_08_goto_targets_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for target in all_goto_targets(&table) {
            prop_assert!(
                (target.0 as usize) < table.state_count,
                "goto target {} >= state_count {}", target.0, table.state_count
            );
        }
    }
}

// ===========================================================================
// 9. All reduce rule IDs are valid
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_09_reduce_rule_ids_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let n_rules = table.rules.len();
        for rid in collect_reduce_rule_ids(&table) {
            prop_assert!(
                (rid.0 as usize) < n_rules,
                "reduce rule_id {} >= rules len {}", rid.0, n_rules
            );
        }
    }
}

// ===========================================================================
// 10. Accept exists somewhere
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_10_accept_exists(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_accept_anywhere(&table), "no Accept action in table");
    }
}

// ===========================================================================
// 11. Table is deterministic (rebuild → same)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_11_deterministic_state_count(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.state_count, t2.state_count);
    }

    #[test]
    fn prop_11b_deterministic_symbol_count(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
    }

    #[test]
    fn prop_11c_deterministic_eof(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.eof_symbol, t2.eof_symbol);
    }

    #[test]
    fn prop_11d_deterministic_rules_len(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.rules.len(), t2.rules.len());
    }
}

// ===========================================================================
// 12. state_count >= 2
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_12_state_count_ge_two(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.state_count >= 2,
            "expected state_count >= 2, got {}", table.state_count
        );
    }
}

// ===========================================================================
// 13. symbol_count >= number of tokens
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_13_symbol_count_ge_tokens(grammar in n_token_grammar()) {
        let n_tokens = grammar.tokens.len();
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_count >= n_tokens,
            "symbol_count {} < token count {}", table.symbol_count, n_tokens
        );
    }
}

// ===========================================================================
// 14. FIRST/FOLLOW compute succeeds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_14_ff_succeeds(grammar in any_grammar()) {
        let result = FirstFollowSets::compute(&grammar);
        prop_assert!(result.is_ok(), "FirstFollowSets::compute failed: {:?}", result.err());
    }
}

// ===========================================================================
// 15. FirstFollowSets is deterministic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_15_ff_deterministic(grammar in any_grammar()) {
        // Build twice and verify same table output — FF internals are private
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let t1 = build_lr1_automaton(&grammar, &ff1).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        let t2 = build_lr1_automaton(&grammar, &ff2).unwrap();
        prop_assert_eq!(t1.state_count, t2.state_count, "FF determinism: state_count differs");
        prop_assert_eq!(t1.symbol_count, t2.symbol_count, "FF determinism: symbol_count differs");
    }
}

// ===========================================================================
// Additional proptest properties (16–40)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // 16. N-token grammar → automaton succeeds
    #[test]
    fn prop_16_n_token_builds(grammar in n_token_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok());
    }

    // 17. N-alt grammar → automaton succeeds
    #[test]
    fn prop_17_n_alt_builds(grammar in n_alt_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok());
    }

    // 18. Chain grammar → automaton succeeds
    #[test]
    fn prop_18_chain_builds(grammar in chain_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok());
    }

    // 19. Seq grammar → automaton succeeds
    #[test]
    fn prop_19_seq_builds(grammar in seq_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok());
    }

    // 20. Two-NT grammar → automaton succeeds
    #[test]
    fn prop_20_two_nt_builds(grammar in two_nt_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok());
    }

    // 21. Precedence grammar builds
    #[test]
    fn prop_21_precedence_builds((_prec, grammar) in prec_grammar()) {
        let result = try_build(&grammar);
        prop_assert!(result.is_some(), "precedence grammar failed to build");
    }

    // 22. Associativity grammar builds
    #[test]
    fn prop_22_assoc_builds(grammar in assoc_grammar()) {
        let result = try_build(&grammar);
        prop_assert!(result.is_some(), "assoc grammar failed to build");
    }

    // 23. Accept only on EOF
    #[test]
    fn prop_23_accept_only_on_eof(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let eof = table.eof();
        for s in 0..table.state_count {
            for &sym in table.index_to_symbol.iter() {
                if sym == eof { continue; }
                for a in table.actions(StateId(s as u16), sym) {
                    prop_assert!(
                        !matches!(a, Action::Accept),
                        "Accept on non-EOF symbol {:?} state {}", sym, s
                    );
                }
            }
        }
    }

    // 24. rule() accessor doesn't panic
    #[test]
    fn prop_24_rule_no_panic(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for i in 0..table.rules.len() {
            let (lhs, _rhs_len) = table.rule(RuleId(i as u16));
            prop_assert!(
                (lhs.0 as usize) < table.symbol_count,
                "rule {} lhs {} >= symbol_count {}", i, lhs.0, table.symbol_count
            );
        }
    }

    // 25. action_table length matches state_count
    #[test]
    fn prop_25_action_table_len(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }

    // 26. goto_table length matches state_count
    #[test]
    fn prop_26_goto_table_len(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
    }

    // 27. index_to_symbol is non-empty
    #[test]
    fn prop_27_index_to_symbol_nonempty(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.index_to_symbol.is_empty());
    }

    // 28. symbol_to_index is non-empty
    #[test]
    fn prop_28_symbol_to_index_nonempty(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.symbol_to_index.is_empty());
    }

    // 29. symbol_to_index ↔ index_to_symbol roundtrip
    #[test]
    fn prop_29_symbol_index_roundtrip(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for (&sym, &idx) in &table.symbol_to_index {
            prop_assert!(idx < table.index_to_symbol.len());
            prop_assert_eq!(table.index_to_symbol[idx], sym);
        }
    }

    // 30. N-token: shift actions exist
    #[test]
    fn prop_30_shifts_exist(grammar in n_token_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!all_shift_targets(&table).is_empty(), "must have shift actions");
    }

    // 31. Reduce actions exist
    #[test]
    fn prop_31_reduces_exist(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!collect_reduce_rule_ids(&table).is_empty(), "must have reduce actions");
    }

    // 32. initial_state is valid
    #[test]
    fn prop_32_initial_state_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!((table.initial_state.0 as usize) < table.state_count);
    }

    // 33. Chain grammars: state_count grows with depth
    #[test]
    fn prop_33_chain_state_monotonic(depth in 1usize..=3) {
        let g_small = GrammarBuilder::new("pv9_cmon_s")
            .token("a", "a")
            .rule("nt0", vec!["a"])
            .start("nt0")
            .build();
        let t_small = build_table(&g_small);

        let mut b = GrammarBuilder::new("pv9_cmon_l");
        b = b.token("a", "a");
        let nt_names: Vec<String> = (0..=depth).map(|i| format!("nt{i}")).collect();
        for i in 0..depth {
            b = b.rule(&nt_names[i], vec![nt_names[i + 1].as_str()]);
        }
        b = b.rule(&nt_names[depth], vec!["a"]);
        b = b.start(&nt_names[0]);
        let t_deep = build_table(&b.build());

        prop_assert!(
            t_deep.state_count >= t_small.state_count,
            "deeper chain {} < base {}", t_deep.state_count, t_small.state_count
        );
    }

    // 34. More alternatives → more or equal states
    #[test]
    fn prop_34_more_alts_more_states(n_extra in 0usize..=3) {
        let base = GrammarBuilder::new("pv9_base34")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let t_base = build_table(&base);

        let mut bld = GrammarBuilder::new("pv9_ext34")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["a"]);
        for &name in TOK_NAMES.iter().skip(1).take(n_extra.min(3)) {
            bld = bld.rule("start", vec![name]);
        }
        let t_ext = build_table(&bld.start("start").build());

        prop_assert!(
            t_ext.state_count >= t_base.state_count,
            "extended {} < base {}", t_ext.state_count, t_base.state_count
        );
    }

    // 35. FF succeeds for n-token grammars
    #[test]
    fn prop_35_ff_n_token(grammar in n_token_grammar()) {
        prop_assert!(FirstFollowSets::compute(&grammar).is_ok());
    }

    // 36. FF succeeds for alt grammars
    #[test]
    fn prop_36_ff_alt(grammar in n_alt_grammar()) {
        prop_assert!(FirstFollowSets::compute(&grammar).is_ok());
    }

    // 37. FF succeeds for chain grammars
    #[test]
    fn prop_37_ff_chain(grammar in chain_grammar()) {
        prop_assert!(FirstFollowSets::compute(&grammar).is_ok());
    }

    // 38. FF succeeds for seq grammars
    #[test]
    fn prop_38_ff_seq(grammar in seq_grammar()) {
        prop_assert!(FirstFollowSets::compute(&grammar).is_ok());
    }

    // 39. FF succeeds for two-NT grammars
    #[test]
    fn prop_39_ff_two_nt(grammar in two_nt_grammar()) {
        prop_assert!(FirstFollowSets::compute(&grammar).is_ok());
    }

    // 40. nonterminal_to_index has entries for two-NT grammars
    #[test]
    fn prop_40_nt_index_populated(grammar in two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.nonterminal_to_index.is_empty());
    }
}

// ===========================================================================
// More proptest properties (41–60)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    // 41. Accept count >= 1
    #[test]
    fn prop_41_accept_count_ge_one(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let eof = table.eof();
        let count: usize = (0..table.state_count)
            .map(|s| {
                table.actions(StateId(s as u16), eof)
                    .iter()
                    .filter(|a| matches!(a, Action::Accept))
                    .count()
            })
            .sum();
        prop_assert!(count >= 1, "must have at least one Accept, got {}", count);
    }

    // 42. Deterministic action_table shape
    #[test]
    fn prop_42_deterministic_action_shape(grammar in any_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.action_table.len(), t2.action_table.len());
        prop_assert_eq!(t1.goto_table.len(), t2.goto_table.len());
    }

    // 43. Seq grammar: state_count >= seq length + 1
    #[test]
    fn prop_43_seq_state_count(n in 1usize..=5) {
        let mut b = GrammarBuilder::new(&format!("pv9_sq{n}"));
        for i in 0..n {
            b = b.token(TOK_NAMES[i], TOK_PATS[i]);
        }
        let rhs: Vec<&str> = TOK_NAMES[..n].to_vec();
        b = b.rule("start", rhs);
        b = b.start("start");
        let table = build_table(&b.build());
        prop_assert!(
            table.state_count > n,
            "seq of {} tokens needs > {} states, got {}",
            n, n, table.state_count
        );
    }

    // 44. Chain grammar: accept exists
    #[test]
    fn prop_44_chain_has_accept(grammar in chain_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_accept_anywhere(&table));
    }

    // 45. Two-NT grammar: goto entries exist
    #[test]
    fn prop_45_two_nt_goto_exists(grammar in two_nt_grammar()) {
        let table = build_table(&grammar);
        let targets = all_goto_targets(&table);
        prop_assert!(!targets.is_empty(), "two-NT grammar must have goto entries");
    }

    // 46. N-alt: shift targets valid
    #[test]
    fn prop_46_alt_shift_targets(grammar in n_alt_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    // 47. N-alt: reduce rule IDs valid
    #[test]
    fn prop_47_alt_reduce_ids(grammar in n_alt_grammar()) {
        let table = build_table(&grammar);
        let n_rules = table.rules.len();
        for rid in collect_reduce_rule_ids(&table) {
            prop_assert!((rid.0 as usize) < n_rules);
        }
    }

    // 48. Chain: shift targets valid
    #[test]
    fn prop_48_chain_shift_targets(grammar in chain_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    // 49. Chain: goto targets valid
    #[test]
    fn prop_49_chain_goto_targets(grammar in chain_grammar()) {
        let table = build_table(&grammar);
        for target in all_goto_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    // 50. Seq: accept exists
    #[test]
    fn prop_50_seq_has_accept(grammar in seq_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_accept_anywhere(&table));
    }

    // 51. Precedence: state_count >= 2
    #[test]
    fn prop_51_prec_state_count((_prec, grammar) in prec_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!(table.state_count >= 2);
        }
    }

    // 52. Assoc: accept exists
    #[test]
    fn prop_52_assoc_has_accept(grammar in assoc_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!(has_accept_anywhere(&table));
        }
    }

    // 53. Precedence: eof is known symbol
    #[test]
    fn prop_53_prec_eof_valid((_prec, grammar) in prec_grammar()) {
        if let Some(table) = try_build(&grammar) {
            prop_assert!(table.symbol_to_index.contains_key(&table.eof_symbol));
        }
    }

    // 54. N-token: eof valid
    #[test]
    fn prop_54_n_token_eof(grammar in n_token_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_to_index.contains_key(&table.eof_symbol));
    }

    // 55. N-alt: accept exists
    #[test]
    fn prop_55_alt_has_accept(grammar in n_alt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(has_accept_anywhere(&table));
    }

    // 56. Two-NT: shift targets valid
    #[test]
    fn prop_56_two_nt_shift_valid(grammar in two_nt_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    // 57. Two-NT: reduce IDs valid
    #[test]
    fn prop_57_two_nt_reduce_valid(grammar in two_nt_grammar()) {
        let table = build_table(&grammar);
        let n_rules = table.rules.len();
        for rid in collect_reduce_rule_ids(&table) {
            prop_assert!((rid.0 as usize) < n_rules);
        }
    }

    // 58. Seq: shift targets valid
    #[test]
    fn prop_58_seq_shift_valid(grammar in seq_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    // 59. Seq: reduce IDs valid
    #[test]
    fn prop_59_seq_reduce_valid(grammar in seq_grammar()) {
        let table = build_table(&grammar);
        let n_rules = table.rules.len();
        for rid in collect_reduce_rule_ids(&table) {
            prop_assert!((rid.0 as usize) < n_rules);
        }
    }

    // 60. FF deterministic for chains
    #[test]
    fn prop_60_ff_deterministic_chain(grammar in chain_grammar()) {
        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let t1 = build_lr1_automaton(&grammar, &ff1).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();
        let t2 = build_lr1_automaton(&grammar, &ff2).unwrap();
        prop_assert_eq!(t1.state_count, t2.state_count);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
    }
}

// ===========================================================================
// Even more proptest properties (61–65)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // 61. grammar() accessor returns grammar with same name
    #[test]
    fn prop_61_grammar_name(n in 1usize..=5) {
        let name = format!("pv9_name{n}");
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let table = build_table(&g);
        prop_assert_eq!(&table.grammar().name, &name);
    }

    // 62. start_symbol is set for all grammars
    #[test]
    fn prop_62_start_symbol_set(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_to_index.contains_key(&table.start_symbol));
    }

    // 63. Deterministic action content
    #[test]
    fn prop_63_deterministic_actions(grammar in n_token_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        for s in 0..t1.state_count {
            for &sym in t1.index_to_symbol.iter() {
                let a1 = t1.actions(StateId(s as u16), sym);
                let a2 = t2.actions(StateId(s as u16), sym);
                prop_assert_eq!(a1, a2, "non-deterministic actions at state {} sym {:?}", s, sym);
            }
        }
    }

    // 64. Deterministic goto content
    #[test]
    fn prop_64_deterministic_goto(grammar in two_nt_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        for s in 0..t1.state_count {
            for &nt in t1.nonterminal_to_index.keys() {
                let g1 = t1.goto(StateId(s as u16), nt);
                let g2 = t2.goto(StateId(s as u16), nt);
                prop_assert_eq!(g1, g2, "non-deterministic goto at state {} nt {:?}", s, nt);
            }
        }
    }

    // 65. rules have valid lhs symbols
    #[test]
    fn prop_65_rules_lhs_in_range(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for i in 0..table.rules.len() {
            let (lhs, _) = table.rule(RuleId(i as u16));
            prop_assert!(
                (lhs.0 as usize) < table.symbol_count,
                "rule {} lhs {} out of range", i, lhs.0
            );
        }
    }
}

// ===========================================================================
// Unit tests (66–85+)
// ===========================================================================

// 66. Minimal grammar table
#[test]
fn test_66_minimal_grammar_table() {
    let g = make_grammar("pv9_min", &["a"], &[("start", vec!["a"])], "start");
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(table.symbol_count > 0);
    assert!(has_accept_anywhere(&table));
}

// 67. Arithmetic grammar table
#[test]
fn test_67_arithmetic_grammar() {
    let g = GrammarBuilder::new("pv9_arith")
        .token("num", "[0-9]+")
        .token("+", "\\+")
        .token("*", "\\*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
    assert!(!all_shift_targets(&table).is_empty());
    assert!(!collect_reduce_rule_ids(&table).is_empty());
}

// 68. Grammar with precedence
#[test]
fn test_68_grammar_with_precedence() {
    let g = GrammarBuilder::new("pv9_prec_unit")
        .token("a", "a")
        .token("+", "\\+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 10, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let table = try_build(&g);
    assert!(table.is_some(), "precedence grammar should build");
    let table = table.unwrap();
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
}

// 69. Grammar with alternatives
#[test]
fn test_69_grammar_with_alternatives() {
    let g = make_grammar(
        "pv9_alts",
        &["a", "b", "c"],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
        ],
        "start",
    );
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
    // At least 3 shift actions in state 0 (one per alternative)
    let shifts_at_0: usize = table
        .index_to_symbol
        .iter()
        .map(|&sym| {
            table
                .actions(StateId(0), sym)
                .iter()
                .filter(|a| matches!(a, Action::Shift(_)))
                .count()
        })
        .sum();
    assert!(
        shifts_at_0 >= 3,
        "expected >= 3 shifts, got {}",
        shifts_at_0
    );
}

// 70. Grammar with chain rules
#[test]
fn test_70_grammar_with_chain_rules() {
    let g = make_grammar(
        "pv9_chain_unit",
        &["x"],
        &[
            ("start", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
        ],
        "start",
    );
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
    let gotos = all_goto_targets(&table);
    assert!(!gotos.is_empty(), "chain grammar must have goto entries");
}

// 71. Single-token single-rule baseline
#[test]
fn test_71_single_token_baseline() {
    let g = make_grammar("pv9_one", &["x"], &[("start", vec!["x"])], "start");
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
    assert_eq!(table.goto_table.len(), table.state_count);
}

// 72. Two-rule grammar has at least two rules
#[test]
fn test_72_two_rule_grammar() {
    let g = make_grammar(
        "pv9_2r",
        &["a", "b"],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    let table = build_table(&g);
    assert!(
        table.rules.len() >= 2,
        "expected >= 2 rules, got {}",
        table.rules.len()
    );
}

// 73. Eof symbol is in symbol_to_index
#[test]
fn test_73_eof_in_symbol_map() {
    let g = make_grammar("pv9_eof", &["a"], &[("start", vec!["a"])], "start");
    let table = build_table(&g);
    assert!(table.symbol_to_index.contains_key(&table.eof_symbol));
}

// 74. Shift targets in range for arithmetic
#[test]
fn test_74_arith_shift_targets() {
    let g = GrammarBuilder::new("pv9_arith_shift")
        .token("num", "[0-9]+")
        .token("+", "\\+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    for target in all_shift_targets(&table) {
        assert!(
            (target.0 as usize) < table.state_count,
            "shift target {} out of range",
            target.0
        );
    }
}

// 75. Reduce IDs valid for arithmetic
#[test]
fn test_75_arith_reduce_ids() {
    let g = GrammarBuilder::new("pv9_arith_red")
        .token("num", "[0-9]+")
        .token("+", "\\+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let n_rules = table.rules.len();
    for rid in collect_reduce_rule_ids(&table) {
        assert!(
            (rid.0 as usize) < n_rules,
            "reduce rule_id {} out of range",
            rid.0
        );
    }
}

// 76. Right-associative grammar builds
#[test]
fn test_76_right_assoc() {
    let g = GrammarBuilder::new("pv9_rassoc")
        .token("a", "a")
        .token("+", "\\+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    assert!(try_build(&g).is_some());
}

// 77. None-associativity grammar builds
#[test]
fn test_77_none_assoc() {
    let g = GrammarBuilder::new("pv9_nassoc")
        .token("a", "a")
        .token("+", "\\+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::None)
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    assert!(try_build(&g).is_some());
}

// 78. Deeply chained grammar (depth 4)
#[test]
fn test_78_deep_chain() {
    let g = make_grammar(
        "pv9_deep",
        &["z"],
        &[
            ("start", vec!["l1"]),
            ("l1", vec!["l2"]),
            ("l2", vec!["l3"]),
            ("l3", vec!["l4"]),
            ("l4", vec!["z"]),
        ],
        "start",
    );
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
}

// 79. Left-recursive grammar
#[test]
fn test_79_left_recursive() {
    let g = GrammarBuilder::new("pv9_leftrec")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
}

// 80. Right-recursive grammar
#[test]
fn test_80_right_recursive() {
    let g = GrammarBuilder::new("pv9_rightrec")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
}

// 81. Sequence grammar (a b c)
#[test]
fn test_81_sequence() {
    let g = make_grammar(
        "pv9_seq3",
        &["a", "b", "c"],
        &[("start", vec!["a", "b", "c"])],
        "start",
    );
    let table = build_table(&g);
    assert!(
        table.state_count >= 4,
        "3-token seq needs >= 4 states, got {}",
        table.state_count
    );
    assert!(has_accept_anywhere(&table));
}

// 82. Deterministic rebuild for minimal grammar
#[test]
fn test_82_deterministic_minimal() {
    let g = make_grammar("pv9_det", &["a"], &[("start", vec!["a"])], "start");
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.symbol_count, t2.symbol_count);
    assert_eq!(t1.eof_symbol, t2.eof_symbol);
    assert_eq!(t1.rules.len(), t2.rules.len());
}

// 83. Multiple precedence levels
#[test]
fn test_83_multiple_precedences() {
    let g = GrammarBuilder::new("pv9_multiprec")
        .token("a", "a")
        .token("+", "\\+")
        .token("*", "\\*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let result = try_build(&g);
    assert!(result.is_some(), "multi-precedence grammar should build");
}

// 84. Grammar with two non-terminals and alternatives
#[test]
fn test_84_two_nt_alternatives() {
    let g = make_grammar(
        "pv9_2nt_alt",
        &["x", "y"],
        &[
            ("start", vec!["inner"]),
            ("inner", vec!["x"]),
            ("inner", vec!["y"]),
        ],
        "start",
    );
    let table = build_table(&g);
    assert!(table.state_count >= 2);
    assert!(has_accept_anywhere(&table));
    assert!(!table.nonterminal_to_index.is_empty());
}

// 85. Grammar name preserved through table build
#[test]
fn test_85_name_preserved() {
    let g = make_grammar("pv9_keepname", &["a"], &[("start", vec!["a"])], "start");
    let table = build_table(&g);
    assert_eq!(table.grammar().name, "pv9_keepname");
}
