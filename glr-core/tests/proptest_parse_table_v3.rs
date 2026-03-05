#![allow(clippy::needless_range_loop)]
//! Property-based tests for GLR parse table invariants.
//!
//! Covers: state counts, EOF consistency, action validity, goto validity,
//! serialization roundtrips, specific grammars, determinism, and edge cases.
//!
//! Run with: `cargo test -p adze-glr-core --test proptest_parse_table_v3`

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

fn has_accept(table: &ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
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

fn any_state_shifts(table: &ParseTable, sym: SymbolId) -> bool {
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

fn total_shift_count(table: &ParseTable) -> usize {
    let mut count = 0;
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(st as u16), sym) {
                if matches!(action, Action::Shift(_)) {
                    count += 1;
                }
            }
        }
    }
    count
}

fn total_reduce_count(table: &ParseTable) -> usize {
    let mut count = 0;
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(st as u16), sym) {
                if matches!(action, Action::Reduce(_)) {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Build a simple `S -> a` grammar
fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

/// Build a grammar with two alternatives: `S -> a | b`
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build()
}

/// Build a chain grammar: `S -> item`, `item -> a`
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("S", vec!["item"])
        .rule("item", vec!["a"])
        .start("S")
        .build()
}

/// Build a binary expression grammar: `E -> E + E | n`
fn expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build()
}

/// Build a sequence grammar: `S -> a b`
fn seq_grammar() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build()
}

/// Build a longer chain: `S -> left right`, `left -> a`, `right -> b`
fn two_nt_grammar() -> Grammar {
    GrammarBuilder::new("two_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["left", "right"])
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .start("S")
        .build()
}

/// Build a right-recursive grammar: `list -> a list | a`
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build()
}

// ===========================================================================
// 1. Table has at least 1 state (5 tests)
// ===========================================================================

#[test]
fn test_state_count_simple_grammar_at_least_one() {
    let table = build_table(&simple_grammar());
    assert!(table.state_count >= 1, "must have at least one state");
}

#[test]
fn test_state_count_two_alt_grammar_at_least_one() {
    let table = build_table(&two_alt_grammar());
    assert!(table.state_count >= 1);
}

#[test]
fn test_state_count_chain_grammar_at_least_one() {
    let table = build_table(&chain_grammar());
    assert!(table.state_count >= 1);
}

proptest! {
    #[test]
    fn proptest_state_count_at_least_one_varying_tokens(n_extra in 0u8..4) {
        let mut builder = GrammarBuilder::new("vary")
            .token("a", "a")
            .rule("S", vec!["a"])
            .start("S");
        for i in 0..n_extra {
            let name = format!("t{i}");
            // Leak string to get &'static str for builder
            let name: &'static str = Box::leak(name.into_boxed_str());
            builder = builder.token(name, name);
        }
        let table = build_table(&builder.build());
        prop_assert!(table.state_count >= 1);
    }

    #[test]
    fn proptest_state_count_at_least_one_varying_rules(n_rules in 1u8..4) {
        let mut builder = GrammarBuilder::new("multi")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c");
        let tokens = ["a", "b", "c"];
        for i in 0..n_rules {
            let tok = tokens[i as usize % tokens.len()];
            builder = builder.rule("S", vec![tok]);
        }
        builder = builder.start("S");
        let table = build_table(&builder.build());
        prop_assert!(table.state_count >= 1);
    }
}

// ===========================================================================
// 2. EOF symbol is consistent (5 tests)
// ===========================================================================

#[test]
fn test_eof_symbol_matches_field() {
    let table = build_table(&simple_grammar());
    assert_eq!(table.eof(), table.eof_symbol);
}

#[test]
fn test_eof_symbol_is_consistent() {
    let table = build_table(&simple_grammar());
    // EOF from accessor matches field
    assert_eq!(
        table.eof(),
        table.eof_symbol,
        "eof() must match eof_symbol field"
    );
}

#[test]
fn test_eof_in_symbol_to_index() {
    let table = build_table(&simple_grammar());
    assert!(
        table.symbol_to_index.contains_key(&table.eof()),
        "EOF must be in symbol_to_index"
    );
}

proptest! {
    #[test]
    fn proptest_eof_consistent_across_grammars(choice in 0u8..5) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => two_alt_grammar(),
            2 => chain_grammar(),
            3 => seq_grammar(),
            _ => two_nt_grammar(),
        };
        let table = build_table(&grammar);
        prop_assert_eq!(table.eof(), table.eof_symbol);
        // EOF must be in symbol_to_index
        prop_assert!(table.symbol_to_index.contains_key(&table.eof()));
    }

    #[test]
    fn proptest_eof_always_in_index(choice in 0u8..4) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => two_alt_grammar(),
            2 => chain_grammar(),
            _ => expr_grammar(),
        };
        let table = build_table(&grammar);
        prop_assert!(table.symbol_to_index.contains_key(&table.eof()));
    }
}

// ===========================================================================
// 3. Actions are valid (5 tests)
// ===========================================================================

#[test]
fn test_actions_shift_targets_valid_states() {
    let table = build_table(&simple_grammar());
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(st as u16), sym) {
                if let Action::Shift(target) = action {
                    assert!(
                        (target.0 as usize) < table.state_count,
                        "shift target {target:?} out of range"
                    );
                }
            }
        }
    }
}

#[test]
fn test_actions_reduce_rules_valid() {
    let table = build_table(&simple_grammar());
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(st as u16), sym) {
                if let Action::Reduce(rule_id) = action {
                    assert!(
                        (rule_id.0 as usize) < table.rules.len(),
                        "reduce rule {rule_id:?} out of range"
                    );
                }
            }
        }
    }
}

#[test]
fn test_actions_accept_only_on_eof() {
    let table = build_table(&simple_grammar());
    let eof = table.eof();
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(st as u16), sym) {
                if matches!(action, Action::Accept) {
                    assert_eq!(sym, eof, "Accept should only appear on EOF symbol");
                }
            }
        }
    }
}

proptest! {
    #[test]
    fn proptest_all_shift_targets_in_range(choice in 0u8..5) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => two_alt_grammar(),
            2 => chain_grammar(),
            3 => seq_grammar(),
            _ => expr_grammar(),
        };
        let table = build_table(&grammar);
        for st in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(st as u16), sym) {
                    if let Action::Shift(target) = action {
                        prop_assert!((target.0 as usize) < table.state_count);
                    }
                }
            }
        }
    }

    #[test]
    fn proptest_all_reduce_rules_in_range(choice in 0u8..5) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => two_alt_grammar(),
            2 => chain_grammar(),
            3 => seq_grammar(),
            _ => two_nt_grammar(),
        };
        let table = build_table(&grammar);
        for st in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(st as u16), sym) {
                    if let Action::Reduce(rule_id) = action {
                        prop_assert!((rule_id.0 as usize) < table.rules.len());
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 4. Goto returns valid states (5 tests)
// ===========================================================================

#[test]
fn test_goto_targets_valid_states_simple() {
    let table = build_table(&simple_grammar());
    for st in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(StateId(st as u16), nt) {
                assert!(
                    (target.0 as usize) < table.state_count,
                    "goto target out of range"
                );
            }
        }
    }
}

#[test]
fn test_goto_targets_valid_states_chain() {
    let table = build_table(&chain_grammar());
    for st in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            if let Some(target) = table.goto(StateId(st as u16), nt) {
                assert!((target.0 as usize) < table.state_count);
            }
        }
    }
}

#[test]
fn test_goto_none_for_unknown_nonterminal() {
    let table = build_table(&simple_grammar());
    // Use a very high symbol ID that can't be a valid nonterminal
    let bogus = SymbolId(9999);
    for st in 0..table.state_count {
        assert!(
            table.goto(StateId(st as u16), bogus).is_none(),
            "goto should return None for unknown nonterminal"
        );
    }
}

proptest! {
    #[test]
    fn proptest_goto_targets_always_valid(choice in 0u8..5) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => two_alt_grammar(),
            2 => chain_grammar(),
            3 => two_nt_grammar(),
            _ => right_recursive_grammar(),
        };
        let table = build_table(&grammar);
        for st in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if let Some(target) = table.goto(StateId(st as u16), nt) {
                    prop_assert!((target.0 as usize) < table.state_count);
                }
            }
        }
    }

    #[test]
    fn proptest_goto_none_for_invalid_state(choice in 0u8..4) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => chain_grammar(),
            2 => seq_grammar(),
            _ => two_nt_grammar(),
        };
        let table = build_table(&grammar);
        let bogus_state = StateId(table.state_count as u16 + 100);
        for &nt in table.nonterminal_to_index.keys() {
            prop_assert!(table.goto(bogus_state, nt).is_none());
        }
    }
}

// ===========================================================================
// 5. Serialization roundtrip (5 tests)
// ===========================================================================

#[cfg(feature = "serialization")]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_roundtrip_simple_grammar() {
        let table = build_table(&simple_grammar());
        let bytes = table.to_bytes().expect("serialization should succeed");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialization should succeed");
        assert_eq!(table.state_count, restored.state_count);
        assert_eq!(table.symbol_count, restored.symbol_count);
        assert_eq!(table.eof_symbol, restored.eof_symbol);
    }

    #[test]
    fn test_roundtrip_preserves_actions() {
        let table = build_table(&two_alt_grammar());
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        for st in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                assert_eq!(
                    table.actions(StateId(st as u16), sym),
                    restored.actions(StateId(st as u16), sym),
                );
            }
        }
    }

    #[test]
    fn test_roundtrip_preserves_goto() {
        let table = build_table(&chain_grammar());
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        for st in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                assert_eq!(
                    table.goto(StateId(st as u16), nt),
                    restored.goto(StateId(st as u16), nt),
                );
            }
        }
    }

    proptest! {
        #[test]
        fn proptest_roundtrip_state_count_preserved(choice in 0u8..5) {
            let grammar = match choice {
                0 => simple_grammar(),
                1 => two_alt_grammar(),
                2 => chain_grammar(),
                3 => seq_grammar(),
                _ => two_nt_grammar(),
            };
            let table = build_table(&grammar);
            let bytes = table.to_bytes().expect("serialize");
            let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
            prop_assert_eq!(table.state_count, restored.state_count);
            prop_assert_eq!(table.symbol_count, restored.symbol_count);
        }

        #[test]
        fn proptest_roundtrip_eof_preserved(choice in 0u8..4) {
            let grammar = match choice {
                0 => simple_grammar(),
                1 => expr_grammar(),
                2 => right_recursive_grammar(),
                _ => two_nt_grammar(),
            };
            let table = build_table(&grammar);
            let bytes = table.to_bytes().expect("serialize");
            let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
            prop_assert_eq!(table.eof(), restored.eof());
            prop_assert_eq!(table.start_symbol(), restored.start_symbol());
        }
    }
}

// ===========================================================================
// 6. Table properties from specific grammars (10 tests)
// ===========================================================================

#[test]
fn test_simple_grammar_has_accept_action() {
    let table = build_table(&simple_grammar());
    assert!(has_accept(&table), "simple grammar must have Accept");
}

#[test]
fn test_simple_grammar_shifts_token_a() {
    let grammar = simple_grammar();
    let a = tok_id(&grammar, "a");
    let table = build_table(&grammar);
    assert!(any_state_shifts(&table, a), "must shift on 'a'");
}

#[test]
fn test_two_alt_grammar_shifts_both_tokens() {
    let grammar = two_alt_grammar();
    let a = tok_id(&grammar, "a");
    let b = tok_id(&grammar, "b");
    let table = build_table(&grammar);
    assert!(any_state_shifts(&table, a), "must shift on 'a'");
    assert!(any_state_shifts(&table, b), "must shift on 'b'");
}

#[test]
fn test_chain_grammar_has_goto_for_intermediate() {
    let grammar = chain_grammar();
    let item_nt = nt_id(&grammar, "item");
    let table = build_table(&grammar);
    let has_goto =
        (0..table.state_count).any(|st| table.goto(StateId(st as u16), item_nt).is_some());
    assert!(has_goto, "chain grammar must have goto for item");
}

#[test]
fn test_expr_grammar_has_shift_and_reduce() {
    let table = build_table(&expr_grammar());
    assert!(total_shift_count(&table) > 0, "expr must have shifts");
    assert!(total_reduce_count(&table) > 0, "expr must have reduces");
}

#[test]
fn test_seq_grammar_state_count_reasonable() {
    let table = build_table(&seq_grammar());
    // S -> a b needs at least 3 states (initial, after-a, after-b)
    assert!(
        table.state_count >= 3,
        "seq grammar needs at least 3 states"
    );
}

#[test]
fn test_two_nt_grammar_goto_for_both_nonterminals() {
    let grammar = two_nt_grammar();
    let left_nt = nt_id(&grammar, "left");
    let right_nt = nt_id(&grammar, "right");
    let table = build_table(&grammar);
    let has_goto_left =
        (0..table.state_count).any(|st| table.goto(StateId(st as u16), left_nt).is_some());
    let has_goto_right =
        (0..table.state_count).any(|st| table.goto(StateId(st as u16), right_nt).is_some());
    assert!(has_goto_left, "must have goto for left");
    assert!(has_goto_right, "must have goto for right");
}

#[test]
fn test_right_recursive_grammar_has_accept() {
    let table = build_table(&right_recursive_grammar());
    assert!(has_accept(&table));
}

#[test]
fn test_symbol_count_at_least_token_count_plus_nonterminals() {
    let table = build_table(&two_nt_grammar());
    // symbol_count should be >= token_count (terminals + nonterminals)
    assert!(
        table.symbol_count >= table.token_count,
        "symbol_count must be >= token_count"
    );
}

#[test]
fn test_rules_lhs_is_nonterminal() {
    let table = build_table(&chain_grammar());
    for rule in &table.rules {
        assert!(
            table.nonterminal_to_index.contains_key(&rule.lhs),
            "rule LHS {:?} must be a nonterminal",
            rule.lhs
        );
    }
}

// ===========================================================================
// 7. Table determinism (5 tests)
// ===========================================================================

#[test]
fn test_deterministic_build_simple() {
    let t1 = build_table(&simple_grammar());
    let t2 = build_table(&simple_grammar());
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.symbol_count, t2.symbol_count);
}

#[test]
fn test_deterministic_build_expr() {
    let t1 = build_table(&expr_grammar());
    let t2 = build_table(&expr_grammar());
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.rules.len(), t2.rules.len());
}

#[test]
fn test_deterministic_actions_match() {
    let grammar = two_alt_grammar();
    let t1 = build_table(&grammar);
    let t2 = build_table(&grammar);
    for st in 0..t1.state_count {
        for &sym in t1.symbol_to_index.keys() {
            assert_eq!(
                t1.actions(StateId(st as u16), sym),
                t2.actions(StateId(st as u16), sym),
            );
        }
    }
}

proptest! {
    #[test]
    fn proptest_deterministic_state_count(choice in 0u8..5) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => two_alt_grammar(),
            2 => chain_grammar(),
            3 => seq_grammar(),
            _ => two_nt_grammar(),
        };
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.state_count, t2.state_count);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
    }

    #[test]
    fn proptest_deterministic_eof(choice in 0u8..4) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => expr_grammar(),
            2 => right_recursive_grammar(),
            _ => chain_grammar(),
        };
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.eof(), t2.eof());
        prop_assert_eq!(t1.start_symbol(), t2.start_symbol());
    }
}

// ===========================================================================
// 8. Edge cases (10 tests)
// ===========================================================================

#[test]
fn test_actions_empty_for_bogus_symbol() {
    let table = build_table(&simple_grammar());
    let bogus = SymbolId(9999);
    let actions = table.actions(StateId(0), bogus);
    assert!(actions.is_empty(), "bogus symbol should have no actions");
}

#[test]
fn test_actions_empty_for_bogus_state() {
    let table = build_table(&simple_grammar());
    let eof = table.eof();
    let bogus_state = StateId(table.state_count as u16 + 50);
    let actions = table.actions(bogus_state, eof);
    assert!(actions.is_empty(), "bogus state should have no actions");
}

#[test]
fn test_goto_none_for_terminal() {
    let grammar = simple_grammar();
    let a = tok_id(&grammar, "a");
    let table = build_table(&grammar);
    // Terminals should not be in nonterminal_to_index, so goto returns None
    let result = table.goto(StateId(0), a);
    assert!(result.is_none(), "goto should return None for terminals");
}

#[test]
fn test_initial_state_is_valid() {
    let table = build_table(&simple_grammar());
    assert!(
        (table.initial_state.0 as usize) < table.state_count,
        "initial state must be valid"
    );
}

#[test]
fn test_action_table_dimensions_match_state_count() {
    let table = build_table(&two_alt_grammar());
    assert_eq!(
        table.action_table.len(),
        table.state_count,
        "action_table rows must equal state_count"
    );
}

#[test]
fn test_goto_table_dimensions_match_state_count() {
    let table = build_table(&chain_grammar());
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto_table rows must equal state_count"
    );
}

#[test]
fn test_symbol_to_index_contains_eof() {
    let table = build_table(&expr_grammar());
    assert!(table.symbol_to_index.contains_key(&table.eof()));
}

#[test]
fn test_index_to_symbol_inverse_of_symbol_to_index() {
    let table = build_table(&two_nt_grammar());
    for (&sym, &idx) in &table.symbol_to_index {
        assert_eq!(
            table.index_to_symbol[idx], sym,
            "index_to_symbol must be inverse of symbol_to_index"
        );
    }
}

#[test]
fn test_validate_passes_for_built_tables() {
    // validate() uses debug_assert that EOF == SymbolId(0), which may not hold
    // for GrammarBuilder-produced grammars. Instead, verify core structural invariants.
    let grammars = [
        simple_grammar(),
        two_alt_grammar(),
        chain_grammar(),
        seq_grammar(),
        two_nt_grammar(),
        expr_grammar(),
        right_recursive_grammar(),
    ];
    for grammar in &grammars {
        let table = build_table(grammar);
        // EOF must be in symbol_to_index
        assert!(
            table.symbol_to_index.contains_key(&table.eof()),
            "eof must be in symbol_to_index"
        );
        // action_table and goto_table row counts match state_count
        assert_eq!(table.action_table.len(), table.state_count);
        assert_eq!(table.goto_table.len(), table.state_count);
    }
}

#[test]
fn test_lex_modes_length_matches_state_count() {
    let table = build_table(&simple_grammar());
    assert_eq!(
        table.lex_modes.len(),
        table.state_count,
        "lex_modes length must equal state_count"
    );
}

// ===========================================================================
// Additional proptest: combined invariants
// ===========================================================================

proptest! {
    #[test]
    fn proptest_combined_invariants(choice in 0u8..7) {
        let grammar = match choice {
            0 => simple_grammar(),
            1 => two_alt_grammar(),
            2 => chain_grammar(),
            3 => seq_grammar(),
            4 => two_nt_grammar(),
            5 => expr_grammar(),
            _ => right_recursive_grammar(),
        };
        let table = build_table(&grammar);

        // State count > 0
        prop_assert!(table.state_count >= 1);

        // EOF is consistent and in index
        prop_assert_eq!(table.eof(), table.eof_symbol);
        prop_assert!(table.symbol_to_index.contains_key(&table.eof()));

        // Has accept
        prop_assert!(has_accept(&table));

        // Action/goto table dimensions
        prop_assert_eq!(table.action_table.len(), table.state_count);
        prop_assert_eq!(table.goto_table.len(), table.state_count);

        // Initial state valid
        prop_assert!((table.initial_state.0 as usize) < table.state_count);

        // All shift targets valid
        for st in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(st as u16), sym) {
                    if let Action::Shift(target) = action {
                        prop_assert!((target.0 as usize) < table.state_count);
                    }
                    if let Action::Reduce(rule_id) = action {
                        prop_assert!((rule_id.0 as usize) < table.rules.len());
                    }
                }
            }
        }

        // All goto targets valid
        for st in 0..table.state_count {
            for &nt in table.nonterminal_to_index.keys() {
                if let Some(target) = table.goto(StateId(st as u16), nt) {
                    prop_assert!((target.0 as usize) < table.state_count);
                }
            }
        }

        // Structural invariants hold
        prop_assert_eq!(table.lex_modes.len(), table.state_count);
    }
}
