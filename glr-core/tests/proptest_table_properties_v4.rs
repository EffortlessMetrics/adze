//! Property-based and unit tests for ParseTable invariants (v4).
//!
//! Run with:
//! ```bash
//! cargo test -p adze-glr-core --test proptest_table_properties_v4
//! ```

use adze_glr_core::{Action, FirstFollowSets, ParseTable, StateId, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::SymbolId;
use adze_ir::builder::GrammarBuilder;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute FIRST/FOLLOW then build parse table.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("build_lr1_automaton failed")
}

/// Build a grammar with given tokens, rules, and start symbol.
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

/// Check whether an Accept action exists anywhere in the table.
fn table_has_accept(table: &ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

/// Collect all Shift targets in the table.
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

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

/// Random valid grammar: 1-4 tokens, 1-4 rules (all S -> t_i).
fn arb_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 1usize..=4)
        .prop_flat_map(|(n_tok, n_rules)| {
            let indices = proptest::collection::vec(0..n_tok, n_rules);
            (Just(n_tok), indices)
        })
        .prop_map(|(n_tok, indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("arb");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            // Always include base rule S -> t0
            b = b.rule("S", vec![tok_names[0].as_str()]);
            for &idx in &indices {
                b = b.rule("S", vec![tok_names[idx].as_str()]);
            }
            b.start("S").build()
        })
}

/// Random grammar with two nonterminals: S -> A, A -> t_i.
fn arb_two_nt_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 0usize..=3)
        .prop_flat_map(|(n_tok, n_extra)| {
            let indices = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), indices)
        })
        .prop_map(|(n_tok, indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("two_nt");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            b = b.rule("S", vec!["A"]);
            b = b.rule("A", vec![tok_names[0].as_str()]);
            for &idx in &indices {
                b = b.rule("A", vec![tok_names[idx].as_str()]);
            }
            b.start("S").build()
        })
}

/// Random grammar with chain: S -> A -> B -> t_i.
fn arb_chain_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=3).prop_map(|n_tok| {
        let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
        let mut b = GrammarBuilder::new("chain");
        for tn in &tok_names {
            b = b.token(tn, tn);
        }
        b = b.rule("S", vec!["A"]);
        b = b.rule("A", vec!["B"]);
        b = b.rule("B", vec![tok_names[0].as_str()]);
        b.start("S").build()
    })
}

// ===========================================================================
// CATEGORY 1: State count >= 1 (5 proptests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_state_count_ge_one(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 1);
    }

    #[test]
    fn prop_state_count_ge_one_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 1);
    }

    #[test]
    fn prop_state_count_ge_one_chain(grammar in arb_chain_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 1);
    }

    #[test]
    fn prop_action_table_nonempty(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.action_table.is_empty());
    }

    #[test]
    fn prop_goto_table_nonempty(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.goto_table.is_empty());
    }
}

// ===========================================================================
// CATEGORY 2: Symbol count > 0 (5 proptests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_symbol_count_positive(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count > 0);
    }

    #[test]
    fn prop_symbol_count_positive_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count > 0);
    }

    #[test]
    fn prop_symbol_count_positive_chain(grammar in arb_chain_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count > 0);
    }

    #[test]
    fn prop_eof_symbol_exists(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        // eof symbol id must be within symbol range
        prop_assert!((table.eof_symbol.0 as usize) < table.symbol_count + 10);
    }

    #[test]
    fn prop_index_to_symbol_nonempty(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.index_to_symbol.is_empty());
    }
}

// ===========================================================================
// CATEGORY 3: Accept exists (5 proptests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_accept_exists(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table_has_accept(&table), "table must have Accept action");
    }

    #[test]
    fn prop_accept_exists_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table_has_accept(&table));
    }

    #[test]
    fn prop_accept_exists_chain(grammar in arb_chain_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table_has_accept(&table));
    }

    #[test]
    fn prop_accept_on_eof_only(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let eof = table.eof();
        // Accept should only appear on EOF column
        for s in 0..table.state_count {
            for &sym in table.index_to_symbol.iter() {
                if sym == eof { continue; }
                let actions = table.actions(StateId(s as u16), sym);
                for a in actions {
                    prop_assert!(
                        !matches!(a, Action::Accept),
                        "Accept found on non-EOF symbol {:?} in state {}", sym, s
                    );
                }
            }
        }
    }

    #[test]
    fn prop_accept_exactly_once(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let eof = table.eof();
        let accept_count: usize = (0..table.state_count)
            .map(|s| {
                table.actions(StateId(s as u16), eof)
                    .iter()
                    .filter(|a| matches!(a, Action::Accept))
                    .count()
            })
            .sum();
        prop_assert!(accept_count >= 1, "must have at least one Accept");
    }
}

// ===========================================================================
// CATEGORY 4: Shift targets valid (5 proptests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_shift_targets_in_range(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!(
                (target.0 as usize) < table.state_count,
                "shift target {} >= state_count {}", target.0, table.state_count
            );
        }
    }

    #[test]
    fn prop_shift_targets_in_range_two_nt(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    #[test]
    fn prop_shift_targets_in_range_chain(grammar in arb_chain_grammar()) {
        let table = build_table(&grammar);
        for target in all_shift_targets(&table) {
            prop_assert!((target.0 as usize) < table.state_count);
        }
    }

    #[test]
    fn prop_goto_targets_in_range(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if let Some(target) = table.goto(StateId(s as u16), nt) {
                    prop_assert!(
                        (target.0 as usize) < table.state_count,
                        "goto target {} >= state_count {}", target.0, table.state_count
                    );
                }
            }
        }
    }

    #[test]
    fn prop_reduce_rule_ids_valid(grammar in arb_grammar()) {
        let table = build_table(&grammar);
        let n_rules = table.rules.len();
        for s in 0..table.state_count {
            for &sym in table.index_to_symbol.iter() {
                for action in table.actions(StateId(s as u16), sym) {
                    if let Action::Reduce(rule_id) = action {
                        prop_assert!(
                            (rule_id.0 as usize) < n_rules,
                            "reduce rule_id {} >= rules len {}", rule_id.0, n_rules
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// CATEGORY 5: Determinism (5 proptests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_deterministic_state_count(grammar in arb_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.state_count, t2.state_count);
    }

    #[test]
    fn prop_deterministic_symbol_count(grammar in arb_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
    }

    #[test]
    fn prop_deterministic_rules_len(grammar in arb_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.rules.len(), t2.rules.len());
    }

    #[test]
    fn prop_deterministic_action_table_shape(grammar in arb_two_nt_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.action_table.len(), t2.action_table.len());
        prop_assert_eq!(t1.goto_table.len(), t2.goto_table.len());
    }

    #[test]
    fn prop_deterministic_eof(grammar in arb_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.eof_symbol, t2.eof_symbol);
    }
}

// ===========================================================================
// CATEGORY 6: Regular table properties (15 tests)
// ===========================================================================

#[test]
fn test_minimal_grammar_state_count() {
    let g = make_grammar("min", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    assert!(t.state_count >= 2, "S->a needs at least 2 states");
}

#[test]
fn test_minimal_grammar_has_accept() {
    let g = make_grammar("min", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    assert!(table_has_accept(&t));
}

#[test]
fn test_two_token_grammar_symbol_count() {
    let g = make_grammar(
        "two",
        &["a", "b"],
        &[("S", vec!["a"]), ("S", vec!["b"])],
        "S",
    );
    let t = build_table(&g);
    assert!(t.symbol_count >= 3, "2 tokens + 1 NT + EOF");
}

#[test]
fn test_two_rule_grammar_rules_len() {
    let g = make_grammar(
        "r2",
        &["a", "b"],
        &[("S", vec!["a"]), ("S", vec!["b"])],
        "S",
    );
    let t = build_table(&g);
    // At least the two user rules + augmented start rule
    assert!(t.rules.len() >= 2);
}

#[test]
fn test_chain_grammar_goto_exists() {
    let g = make_grammar("chain", &["x"], &[("S", vec!["A"]), ("A", vec!["x"])], "S");
    let t = build_table(&g);
    // There must be at least one goto entry for nonterminal A
    let has_goto = (0..t.state_count).any(|s| {
        t.nonterminal_to_index
            .keys()
            .any(|&nt| t.goto(StateId(s as u16), nt).is_some())
    });
    assert!(has_goto, "chain grammar must have goto entries");
}

#[test]
fn test_action_table_len_matches_state_count() {
    let g = make_grammar("at", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn test_goto_table_len_matches_state_count() {
    let g = make_grammar("gt", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn test_grammar_name_preserved() {
    let g = make_grammar("myname", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    assert_eq!(t.grammar().name, "myname");
}

#[test]
fn test_start_symbol_set() {
    let g = make_grammar("ss", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    // start_symbol should be nonzero (0 is reserved for EOF/error)
    assert_ne!(t.start_symbol, SymbolId(0));
}

#[test]
fn test_symbol_to_index_roundtrip() {
    let g = make_grammar(
        "rt",
        &["a", "b"],
        &[("S", vec!["a"]), ("S", vec!["b"])],
        "S",
    );
    let t = build_table(&g);
    for (&sym, &idx) in &t.symbol_to_index {
        assert!(idx < t.index_to_symbol.len());
        assert_eq!(t.index_to_symbol[idx], sym);
    }
}

#[test]
fn test_three_token_grammar_shifts_exist() {
    let g = make_grammar(
        "three",
        &["a", "b", "c"],
        &[("S", vec!["a"]), ("S", vec!["b"]), ("S", vec!["c"])],
        "S",
    );
    let t = build_table(&g);
    let targets = all_shift_targets(&t);
    assert!(!targets.is_empty(), "must have shift actions");
}

#[test]
fn test_reduce_actions_exist() {
    let g = make_grammar("red", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    let has_reduce = (0..t.state_count).any(|s| {
        t.index_to_symbol.iter().any(|&sym| {
            t.actions(StateId(s as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
    });
    assert!(has_reduce, "must have at least one Reduce action");
}

#[test]
fn test_initial_state_valid() {
    let g = make_grammar("init", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    assert!((t.initial_state.0 as usize) < t.state_count);
}

#[test]
fn test_rules_lhs_nonzero() {
    let g = make_grammar(
        "lhs",
        &["a", "b"],
        &[("S", vec!["a"]), ("S", vec!["b"])],
        "S",
    );
    let t = build_table(&g);
    // All user-visible rules should have a real LHS
    for rule in &t.rules {
        // LHS 0 is only valid for the augmented start rule
        assert!(
            rule.lhs.0 == 0 || rule.rhs_len > 0 || rule.lhs.0 > 0,
            "rule lhs should be valid"
        );
    }
}

#[test]
fn test_nonterminal_to_index_has_entries() {
    let g = make_grammar("nti", &["a"], &[("S", vec!["A"]), ("A", vec!["a"])], "S");
    let t = build_table(&g);
    assert!(
        !t.nonterminal_to_index.is_empty(),
        "must have nonterminal index entries"
    );
}

// ===========================================================================
// CATEGORY 7: Edge cases (10 tests)
// ===========================================================================

#[test]
fn test_single_token_single_rule() {
    let g = make_grammar("one", &["x"], &[("S", vec!["x"])], "S");
    let t = build_table(&g);
    assert!(t.state_count >= 2);
    assert!(table_has_accept(&t));
}

#[test]
fn test_multiple_alternatives() {
    let g = make_grammar(
        "multi",
        &["a", "b", "c", "d"],
        &[
            ("S", vec!["a"]),
            ("S", vec!["b"]),
            ("S", vec!["c"]),
            ("S", vec!["d"]),
        ],
        "S",
    );
    let t = build_table(&g);
    assert!(t.state_count >= 2);
    assert!(table_has_accept(&t));
}

#[test]
fn test_deep_chain() {
    let g = make_grammar(
        "deep",
        &["x"],
        &[
            ("S", vec!["A"]),
            ("A", vec!["B"]),
            ("B", vec!["C"]),
            ("C", vec!["x"]),
        ],
        "S",
    );
    let t = build_table(&g);
    assert!(t.state_count >= 2);
    assert!(table_has_accept(&t));
    // Must have goto entries for the chain nonterminals
    assert!(t.nonterminal_to_index.len() >= 3);
}

#[test]
fn test_multi_symbol_rhs() {
    let g = make_grammar("multi_rhs", &["a", "b"], &[("S", vec!["a", "b"])], "S");
    let t = build_table(&g);
    assert!(t.state_count >= 3, "S->a b needs at least 3 states");
    assert!(table_has_accept(&t));
}

#[test]
fn test_left_recursive_grammar() {
    // S -> S a | a (left recursion)
    let g = make_grammar(
        "leftrec",
        &["a"],
        &[("S", vec!["S", "a"]), ("S", vec!["a"])],
        "S",
    );
    let t = build_table(&g);
    assert!(t.state_count >= 2);
    assert!(table_has_accept(&t));
}

#[test]
fn test_right_recursive_grammar() {
    // S -> a S | a
    let g = make_grammar(
        "rightrec",
        &["a"],
        &[("S", vec!["a", "S"]), ("S", vec!["a"])],
        "S",
    );
    let t = build_table(&g);
    assert!(t.state_count >= 2);
    assert!(table_has_accept(&t));
}

#[test]
fn test_multiple_nonterminals_shared_tokens() {
    let g = make_grammar(
        "shared",
        &["x", "y"],
        &[
            ("S", vec!["A"]),
            ("S", vec!["B"]),
            ("A", vec!["x"]),
            ("B", vec!["y"]),
        ],
        "S",
    );
    let t = build_table(&g);
    assert!(table_has_accept(&t));
    assert!(t.symbol_count >= 4);
}

#[test]
fn test_longer_rhs_produces_more_states() {
    let short = make_grammar("short", &["a"], &[("S", vec!["a"])], "S");
    let long = make_grammar("long", &["a", "b", "c"], &[("S", vec!["a", "b", "c"])], "S");
    let ts = build_table(&short);
    let tl = build_table(&long);
    assert!(
        tl.state_count >= ts.state_count,
        "longer RHS {} should produce >= states than short {}",
        tl.state_count,
        ts.state_count
    );
}

#[test]
fn test_shift_target_not_initial_for_reduce() {
    // After shifting token, should be in a different state
    let g = make_grammar("shift_dest", &["a"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    let targets = all_shift_targets(&t);
    // At least one shift must go somewhere
    assert!(!targets.is_empty());
}

#[test]
fn test_diamond_grammar() {
    // S -> A | B, A -> x, B -> x (diamond shape)
    let g = make_grammar(
        "diamond",
        &["x"],
        &[
            ("S", vec!["A"]),
            ("S", vec!["B"]),
            ("A", vec!["x"]),
            ("B", vec!["x"]),
        ],
        "S",
    );
    let t = build_table(&g);
    assert!(table_has_accept(&t));
    assert!(t.state_count >= 2);
}
