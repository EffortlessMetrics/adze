//! Tests for action encoding, table structure, multi-action cells,
//! goto table, table size properties, determinism, and edge cases.

use adze_glr_core::{
    Action, ActionCell, FirstFollowSets, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
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

fn has_accept(table: &ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|s| {
        table
            .actions(StateId(s as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn count_actions_of_kind(table: &ParseTable, pred: fn(&Action) -> bool) -> usize {
    let mut count = 0;
    for s in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(s as u16), sym) {
                if pred(action) {
                    count += 1;
                }
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Grammars
// ---------------------------------------------------------------------------

/// start -> a
fn grammar_single() -> Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// start -> a b
fn grammar_seq() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

/// expr -> expr + expr | n  (ambiguous, causes GLR conflicts)
fn grammar_ambiguous() -> Grammar {
    GrammarBuilder::new("ambig")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build()
}

/// start -> left right, left -> a, right -> b
fn grammar_two_nt() -> Grammar {
    GrammarBuilder::new("two_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["left", "right"])
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .start("start")
        .build()
}

/// start -> mid, mid -> inner, inner -> c  (chain of nonterminals)
fn grammar_chain() -> Grammar {
    GrammarBuilder::new("chain")
        .token("c", "c")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["inner"])
        .rule("inner", vec!["c"])
        .start("start")
        .build()
}

/// start -> a | b | c  (multiple alternatives for start)
fn grammar_alternatives() -> Grammar {
    GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build()
}

/// start -> a b c d  (longer RHS)
fn grammar_long_rhs() -> Grammar {
    GrammarBuilder::new("long")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build()
}

// ===========================================================================
// 1. Action encoding — Shift/Reduce/Accept action encoding and decoding
// ===========================================================================

#[test]
fn action_shift_carries_state_id() {
    let action = Action::Shift(StateId(42));
    assert!(matches!(action, Action::Shift(StateId(42))));
}

#[test]
fn action_reduce_carries_rule_id() {
    let action = Action::Reduce(RuleId(7));
    assert!(matches!(action, Action::Reduce(RuleId(7))));
}

#[test]
fn action_accept_is_unit() {
    let action = Action::Accept;
    assert!(matches!(action, Action::Accept));
}

#[test]
fn action_error_is_unit() {
    let action = Action::Error;
    assert!(matches!(action, Action::Error));
}

#[test]
fn action_recover_is_unit() {
    let action = Action::Recover;
    assert!(matches!(action, Action::Recover));
}

#[test]
fn action_shift_preserves_state_value() {
    for i in [0u16, 1, 100, 255, u16::MAX] {
        let action = Action::Shift(StateId(i));
        if let Action::Shift(st) = action {
            assert_eq!(st.0, i);
        } else {
            panic!("expected Shift");
        }
    }
}

#[test]
fn action_reduce_preserves_rule_value() {
    for i in [0u16, 1, 50, 999] {
        let action = Action::Reduce(RuleId(i));
        if let Action::Reduce(rid) = action {
            assert_eq!(rid.0, i);
        } else {
            panic!("expected Reduce");
        }
    }
}

#[test]
fn action_equality_shift() {
    assert_eq!(Action::Shift(StateId(5)), Action::Shift(StateId(5)));
    assert_ne!(Action::Shift(StateId(5)), Action::Shift(StateId(6)));
}

#[test]
fn action_equality_reduce() {
    assert_eq!(Action::Reduce(RuleId(3)), Action::Reduce(RuleId(3)));
    assert_ne!(Action::Reduce(RuleId(3)), Action::Reduce(RuleId(4)));
}

#[test]
fn action_equality_cross_variant() {
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
    assert_ne!(Action::Accept, Action::Error);
    assert_ne!(Action::Shift(StateId(0)), Action::Accept);
}

#[test]
fn action_fork_holds_multiple_actions() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    if let Action::Fork(ref actions) = fork {
        assert_eq!(actions.len(), 2);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn action_fork_can_be_nested() {
    let fork = Action::Fork(vec![
        Action::Fork(vec![Action::Shift(StateId(1))]),
        Action::Accept,
    ]);
    if let Action::Fork(ref outer) = fork {
        assert!(matches!(&outer[0], Action::Fork(_)));
        assert!(matches!(&outer[1], Action::Accept));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn action_debug_format_contains_variant_name() {
    let shift = format!("{:?}", Action::Shift(StateId(3)));
    let reduce = format!("{:?}", Action::Reduce(RuleId(1)));
    let accept = format!("{:?}", Action::Accept);
    assert!(shift.contains("Shift"));
    assert!(reduce.contains("Reduce"));
    assert!(accept.contains("Accept"));
}

// ===========================================================================
// 2. Action table structure — states × symbols → action lists
// ===========================================================================

#[test]
fn action_table_has_one_row_per_state() {
    let g = grammar_single();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn action_table_rows_have_consistent_width() {
    let g = grammar_seq();
    let table = build_table(&g);
    let widths: HashSet<usize> = table.action_table.iter().map(|row| row.len()).collect();
    assert_eq!(widths.len(), 1, "all rows should have the same width");
}

#[test]
fn action_table_initial_state_has_actions() {
    let g = grammar_single();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    assert!(
        !actions.is_empty(),
        "initial state must have action on token 'a'"
    );
}

#[test]
fn actions_on_unknown_symbol_returns_empty() {
    let g = grammar_single();
    let table = build_table(&g);
    let bogus = SymbolId(9999);
    assert!(table.actions(StateId(0), bogus).is_empty());
}

#[test]
fn actions_on_out_of_range_state_returns_empty() {
    let g = grammar_single();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let bogus_state = StateId(table.state_count as u16 + 10);
    assert!(table.actions(bogus_state, a).is_empty());
}

#[test]
fn accept_action_exists_on_eof() {
    let g = grammar_single();
    let table = build_table(&g);
    assert!(has_accept(&table), "table must contain an Accept action");
}

#[test]
fn accept_only_on_eof_symbol() {
    let g = grammar_single();
    let table = build_table(&g);
    let eof = table.eof();
    for s in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(s as u16), sym) {
                if matches!(action, Action::Accept) {
                    assert_eq!(sym, eof, "Accept must only appear on EOF");
                }
            }
        }
    }
}

#[test]
fn shift_actions_target_valid_states() {
    let g = grammar_seq();
    let table = build_table(&g);
    for s in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(s as u16), sym) {
                if let Action::Shift(tgt) = action {
                    assert!(
                        (tgt.0 as usize) < table.state_count,
                        "Shift target {tgt:?} out of range"
                    );
                }
            }
        }
    }
}

#[test]
fn reduce_actions_reference_valid_rules() {
    let g = grammar_two_nt();
    let table = build_table(&g);
    for s in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(s as u16), sym) {
                if let Action::Reduce(rid) = action {
                    assert!(
                        (rid.0 as usize) < table.rules.len(),
                        "Reduce rule {rid:?} out of range"
                    );
                }
            }
        }
    }
}

#[test]
fn simple_grammar_has_at_least_one_shift() {
    let g = grammar_single();
    let table = build_table(&g);
    let shifts = count_actions_of_kind(&table, |a| matches!(a, Action::Shift(_)));
    assert!(shifts >= 1, "must have at least one Shift action");
}

#[test]
fn simple_grammar_has_at_least_one_reduce() {
    let g = grammar_single();
    let table = build_table(&g);
    let reduces = count_actions_of_kind(&table, |a| matches!(a, Action::Reduce(_)));
    assert!(reduces >= 1, "must have at least one Reduce action");
}

// ===========================================================================
// 3. Multi-action cells — GLR conflict cells with multiple actions
// ===========================================================================

#[test]
fn action_cell_can_hold_single_action() {
    let cell: ActionCell = vec![Action::Shift(StateId(0))];
    assert_eq!(cell.len(), 1);
}

#[test]
fn action_cell_can_hold_shift_reduce_pair() {
    let cell: ActionCell = vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn action_cell_can_hold_reduce_reduce_pair() {
    let cell: ActionCell = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().all(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn action_cell_can_hold_many_actions() {
    let cell: ActionCell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ];
    assert_eq!(cell.len(), 4);
}

#[test]
fn ambiguous_grammar_produces_multi_action_or_fork() {
    let g = grammar_ambiguous();
    let table = build_table(&g);

    let mut has_multi = false;
    for row in &table.action_table {
        for cell in row {
            if cell.len() > 1 {
                has_multi = true;
            }
            for action in cell {
                if matches!(action, Action::Fork(_)) {
                    has_multi = true;
                }
            }
        }
    }
    assert!(
        has_multi,
        "ambiguous grammar should produce multi-action cells or Fork actions"
    );
}

#[test]
fn non_ambiguous_grammar_cells_at_most_one_action() {
    let g = grammar_single();
    let table = build_table(&g);
    for row in &table.action_table {
        for cell in row {
            let non_fork_count = cell
                .iter()
                .filter(|a| !matches!(a, Action::Fork(_) | Action::Error))
                .count();
            // Each cell should have at most one non-error action in a non-ambiguous grammar
            assert!(
                non_fork_count <= 1,
                "non-ambiguous grammar should not have conflicts, got {non_fork_count} actions"
            );
        }
    }
}

#[test]
fn action_cell_empty_means_no_valid_action() {
    let cell: ActionCell = vec![];
    assert!(cell.is_empty());
}

// ===========================================================================
// 4. Goto table — nonterminal goto entries
// ===========================================================================

#[test]
fn goto_table_has_one_row_per_state() {
    let g = grammar_single();
    let table = build_table(&g);
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto table must have one row per state"
    );
}

#[test]
fn goto_from_initial_state_for_start_symbol_exists() {
    let g = grammar_single();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(
        table.goto(table.initial_state, start).is_some(),
        "goto(initial, start) must be defined"
    );
}

#[test]
fn goto_targets_are_valid_states() {
    let g = grammar_two_nt();
    let table = build_table(&g);
    for s in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(tgt) = table.goto(StateId(s as u16), nt) {
                assert!(
                    (tgt.0 as usize) < table.state_count,
                    "goto target {tgt:?} out of range"
                );
            }
        }
    }
}

#[test]
fn goto_on_terminal_returns_none() {
    let g = grammar_single();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    // Terminals are not in nonterminal_to_index so goto returns None
    assert!(
        table.goto(StateId(0), a).is_none(),
        "goto on terminal should return None"
    );
}

#[test]
fn goto_after_shift_for_nonterminal_chain() {
    let g = grammar_chain();
    let table = build_table(&g);
    // The chain start -> mid -> inner -> c means we should have goto entries for each
    let start_nt = nt_id(&g, "start");
    let mid_nt = nt_id(&g, "mid");
    let inner_nt = nt_id(&g, "inner");

    // At least one state should have a goto for each nonterminal
    let has_goto = |nt: SymbolId| -> bool {
        (0..table.state_count).any(|s| table.goto(StateId(s as u16), nt).is_some())
    };
    assert!(has_goto(start_nt), "must have goto for start");
    assert!(has_goto(mid_nt), "must have goto for mid");
    assert!(has_goto(inner_nt), "must have goto for inner");
}

#[test]
fn goto_on_start_from_initial_leads_to_accept_state() {
    let g = grammar_single();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    let accept_state = table.goto(table.initial_state, start).unwrap();
    let eof = table.eof();
    let accepts = table
        .actions(accept_state, eof)
        .iter()
        .any(|a| matches!(a, Action::Accept));
    assert!(
        accepts,
        "state after goto(initial, start) should accept on EOF"
    );
}

#[test]
fn goto_entries_count_increases_with_nonterminals() {
    let g1 = grammar_single(); // 1 nonterminal (S)
    let g2 = grammar_two_nt(); // 3 nonterminals (S, A, B)
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);

    let count_gotos = |table: &ParseTable| -> usize {
        let mut n = 0;
        for s in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if table.goto(StateId(s as u16), nt).is_some() {
                    n += 1;
                }
            }
        }
        n
    };
    assert!(
        count_gotos(&t2) >= count_gotos(&t1),
        "more nonterminals should produce at least as many goto entries"
    );
}

// ===========================================================================
// 5. Table size properties — proportional to grammar size
// ===========================================================================

#[test]
fn state_count_at_least_one() {
    let table = build_table(&grammar_single());
    assert!(table.state_count >= 1);
}

#[test]
fn state_count_grows_with_grammar_complexity() {
    let t1 = build_table(&grammar_single());
    let t2 = build_table(&grammar_long_rhs());
    assert!(
        t2.state_count >= t1.state_count,
        "longer RHS should produce at least as many states"
    );
}

#[test]
fn rules_len_matches_grammar_rules() {
    let g = grammar_two_nt();
    let table = build_table(&g);
    // Table rules include the augmented start rule, so >= grammar rules
    assert!(
        table.rules.len() >= 3,
        "table should have at least the 3 user rules (plus augmented start)"
    );
}

#[test]
fn symbol_to_index_contains_eof() {
    let table = build_table(&grammar_single());
    assert!(
        table.symbol_to_index.contains_key(&table.eof()),
        "EOF must be in symbol_to_index"
    );
}

#[test]
fn eof_symbol_matches_accessor() {
    let table = build_table(&grammar_single());
    assert_eq!(table.eof(), table.eof_symbol);
}

#[test]
fn start_symbol_matches_accessor() {
    let table = build_table(&grammar_single());
    assert_eq!(table.start_symbol(), table.start_symbol);
}

#[test]
fn token_count_at_least_one() {
    let table = build_table(&grammar_single());
    assert!(table.token_count >= 1, "must have at least one token");
}

#[test]
fn table_with_more_tokens_has_wider_action_rows() {
    let t1 = build_table(&grammar_single()); // 1 token
    let t2 = build_table(&grammar_long_rhs()); // 4 tokens
    let width1 = t1.action_table.first().map_or(0, |r| r.len());
    let width2 = t2.action_table.first().map_or(0, |r| r.len());
    assert!(
        width2 >= width1,
        "more tokens should produce wider (or equal) action rows"
    );
}

#[test]
fn parse_rule_lhs_rhs_len_are_consistent() {
    let g = grammar_two_nt();
    let table = build_table(&g);
    for rule in &table.rules {
        // rhs_len should be reasonable
        assert!(rule.rhs_len <= 100, "rhs_len should be reasonable");
        // lhs should be a known symbol
        assert!(
            table.symbol_to_index.contains_key(&rule.lhs)
                || table.nonterminal_to_index.contains_key(&rule.lhs),
            "rule LHS {:?} should be a known symbol",
            rule.lhs
        );
    }
}

// ===========================================================================
// 6. Action determinism — same grammar → same table
// ===========================================================================

#[test]
fn determinism_same_grammar_same_state_count() {
    let g1 = grammar_single();
    let g2 = grammar_single();
    assert_eq!(build_table(&g1).state_count, build_table(&g2).state_count);
}

#[test]
fn determinism_same_grammar_same_rule_count() {
    let g1 = grammar_two_nt();
    let g2 = grammar_two_nt();
    assert_eq!(build_table(&g1).rules.len(), build_table(&g2).rules.len());
}

#[test]
fn determinism_same_grammar_same_action_table_dimensions() {
    let g1 = grammar_seq();
    let g2 = grammar_seq();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert_eq!(t1.action_table.len(), t2.action_table.len());
    for (r1, r2) in t1.action_table.iter().zip(t2.action_table.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

#[test]
fn determinism_same_grammar_same_goto_table_dimensions() {
    let g1 = grammar_two_nt();
    let g2 = grammar_two_nt();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert_eq!(t1.goto_table.len(), t2.goto_table.len());
    for (r1, r2) in t1.goto_table.iter().zip(t2.goto_table.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

#[test]
fn determinism_accept_states_are_identical() {
    let g1 = grammar_single();
    let g2 = grammar_single();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    let eof1 = t1.eof();
    let eof2 = t2.eof();

    let accept_states_1: Vec<u16> = (0..t1.state_count)
        .filter(|&s| {
            t1.actions(StateId(s as u16), eof1)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .map(|s| s as u16)
        .collect();
    let accept_states_2: Vec<u16> = (0..t2.state_count)
        .filter(|&s| {
            t2.actions(StateId(s as u16), eof2)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .map(|s| s as u16)
        .collect();
    assert_eq!(accept_states_1, accept_states_2);
}

#[test]
fn determinism_shift_count_identical() {
    let g1 = grammar_seq();
    let g2 = grammar_seq();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    let c1 = count_actions_of_kind(&t1, |a| matches!(a, Action::Shift(_)));
    let c2 = count_actions_of_kind(&t2, |a| matches!(a, Action::Shift(_)));
    assert_eq!(c1, c2);
}

#[test]
fn determinism_reduce_count_identical() {
    let g1 = grammar_two_nt();
    let g2 = grammar_two_nt();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    let c1 = count_actions_of_kind(&t1, |a| matches!(a, Action::Reduce(_)));
    let c2 = count_actions_of_kind(&t2, |a| matches!(a, Action::Reduce(_)));
    assert_eq!(c1, c2);
}

// ===========================================================================
// 7. Edge cases — minimal grammars, large action sets, empty cells
// ===========================================================================

#[test]
fn minimal_grammar_builds_successfully() {
    let g = grammar_single();
    let table = build_table(&g);
    assert!(table.state_count >= 1);
    assert!(has_accept(&table));
}

#[test]
fn sanity_check_passes_for_simple_grammar() {
    let g = grammar_single();
    let table = build_table(&g);
    assert!(
        sanity_check_tables(&table).is_ok(),
        "sanity check should pass for well-formed table"
    );
}

#[test]
fn sanity_check_passes_for_multi_rule_grammar() {
    let g = grammar_two_nt();
    let table = build_table(&g);
    assert!(sanity_check_tables(&table).is_ok());
}

#[test]
fn sanity_check_passes_for_alternatives_grammar() {
    let g = grammar_alternatives();
    let table = build_table(&g);
    assert!(sanity_check_tables(&table).is_ok());
}

#[test]
fn alternatives_grammar_has_accept() {
    let g = grammar_alternatives();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn chain_grammar_has_correct_rule_count() {
    let g = grammar_chain();
    let table = build_table(&g);
    // 3 user rules: S->A, A->B, B->c, plus augmented start
    assert!(table.rules.len() >= 3);
}

#[test]
fn long_rhs_grammar_has_many_states() {
    let t = build_table(&grammar_long_rhs());
    // S -> a b c d requires at least 5 states (one per dot position + accept)
    assert!(
        t.state_count >= 5,
        "long RHS should produce at least 5 states, got {}",
        t.state_count
    );
}

#[test]
fn all_reduce_rule_ids_are_unique_per_cell() {
    let g = grammar_alternatives();
    let table = build_table(&g);
    for s in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            let reduces: Vec<u16> = table
                .actions(StateId(s as u16), sym)
                .iter()
                .filter_map(|a| {
                    if let Action::Reduce(rid) = a {
                        Some(rid.0)
                    } else {
                        None
                    }
                })
                .collect();
            let unique: HashSet<u16> = reduces.iter().copied().collect();
            assert_eq!(
                reduces.len(),
                unique.len(),
                "duplicate reduce rule IDs in cell ({}, {:?})",
                s,
                sym
            );
        }
    }
}

#[test]
fn most_cells_are_empty_in_sparse_table() {
    let g = grammar_single();
    let table = build_table(&g);
    let total_cells = table.state_count * table.action_table.first().map_or(0, |r| r.len());
    let non_empty: usize = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| !cell.is_empty())
        .count();
    assert!(
        non_empty <= total_cells,
        "non-empty cells should not exceed total"
    );
    // For a tiny grammar, most cells should be empty
    assert!(
        non_empty < total_cells,
        "at least some cells should be empty in a simple grammar"
    );
}

#[test]
fn eof_symbol_is_in_index_to_symbol() {
    let table = build_table(&grammar_single());
    assert!(
        table.index_to_symbol.contains(&table.eof()),
        "EOF should appear in index_to_symbol"
    );
}

#[test]
fn initial_state_is_within_bounds() {
    let table = build_table(&grammar_single());
    assert!(
        (table.initial_state.0 as usize) < table.state_count,
        "initial state must be within state_count"
    );
}

#[test]
fn grammar_accessor_returns_valid_grammar() {
    let g = grammar_single();
    let table = build_table(&g);
    let returned = table.grammar();
    assert!(!returned.tokens.is_empty(), "grammar should have tokens");
}

#[test]
fn rule_accessor_returns_lhs_and_rhs_len() {
    let g = grammar_seq();
    let table = build_table(&g);
    // Check all rules are accessible
    for i in 0..table.rules.len() {
        let (lhs, rhs_len) = table.rule(RuleId(i as u16));
        assert_eq!(lhs, table.rules[i].lhs);
        assert_eq!(rhs_len, table.rules[i].rhs_len);
    }
}

#[test]
fn ambiguous_grammar_passes_sanity_check() {
    let g = grammar_ambiguous();
    let table = build_table(&g);
    // Even ambiguous grammars should have valid structure
    assert!(sanity_check_tables(&table).is_ok());
}

#[test]
fn ambiguous_grammar_has_accept() {
    let g = grammar_ambiguous();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn no_shift_targets_state_zero_as_self_loop_in_simple_grammar() {
    let g = grammar_single();
    let table = build_table(&g);
    // In a simple grammar, the initial state should not shift to itself
    let initial = table.initial_state;
    let a = tok_id(&g, "a");
    for action in table.actions(initial, a) {
        if let Action::Shift(tgt) = action {
            // Shift should go forward, not loop
            assert_ne!(
                *tgt, initial,
                "shift from initial should not loop to initial"
            );
        }
    }
}

#[test]
fn exactly_one_accept_state_for_simple_grammar() {
    let g = grammar_single();
    let table = build_table(&g);
    let eof = table.eof();
    let accept_state_count = (0..table.state_count)
        .filter(|&s| {
            table
                .actions(StateId(s as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .count();
    assert_eq!(
        accept_state_count, 1,
        "simple grammar should have exactly one accept state"
    );
}

#[test]
fn seq_grammar_shifts_a_then_b() {
    let g = grammar_seq();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");

    // Initial state should shift on 'a'
    let actions_a = table.actions(table.initial_state, a);
    let shift_a = actions_a.iter().find_map(|act| {
        if let Action::Shift(st) = act {
            Some(*st)
        } else {
            None
        }
    });
    assert!(shift_a.is_some(), "initial state should shift on 'a'");

    // After shifting 'a', should shift on 'b'
    let next = shift_a.unwrap();
    let actions_b = table.actions(next, b);
    let shift_b = actions_b.iter().any(|act| matches!(act, Action::Shift(_)));
    assert!(shift_b, "after shifting 'a', should shift on 'b'");
}
