//! Property-based tests for FIRST/FOLLOW and parse table invariants.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, SymbolId};
use proptest::prelude::*;

/// Build a simple choice grammar with n tokens, compute FIRST/FOLLOW.
fn build_choice_grammar(n_tokens: usize) -> (adze_ir::Grammar, FirstFollowSets) {
    let mut builder = GrammarBuilder::new("test");
    for i in 0..n_tokens {
        let name = format!("tok{}", i);
        builder = builder.token(&name, &format!("t{}", i));
    }
    for i in 0..n_tokens {
        let name = format!("tok{}", i);
        builder = builder.rule("start", vec![&name]);
    }
    let mut grammar = builder.start("start").build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    (grammar, ff)
}

/// Build a chain grammar: start -> level0 -> level1 -> ... -> leaf
fn build_chain_grammar(depth: usize) -> (adze_ir::Grammar, FirstFollowSets) {
    let mut builder = GrammarBuilder::new("chain").token("leaf", "x");

    let names: Vec<String> = (0..depth).map(|i| format!("level{}", i)).collect();

    if depth == 0 {
        builder = builder.rule("start", vec!["leaf"]);
    } else {
        builder = builder.rule("start", vec![&names[0]]);
        for i in 0..depth - 1 {
            builder = builder.rule(&names[i], vec![&names[i + 1]]);
        }
        builder = builder.rule(&names[depth - 1], vec!["leaf"]);
    }

    let mut grammar = builder.start("start").build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    (grammar, ff)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn first_sets_exist_for_terminals(n_tokens in 1usize..6) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        for &tok_id in grammar.tokens.keys() {
            let first = ff.first(tok_id);
            prop_assert!(first.is_some(),
                "Terminal {:?} should have a FIRST set", tok_id);
        }
    }

    #[test]
    fn first_set_of_nonterminal_start_nonempty(n_tokens in 1usize..6) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        if let Some(start) = grammar.start_symbol() {
            if let Some(first) = ff.first(start) {
                prop_assert!(first.count_ones(..) > 0,
                    "FIRST(start) should not be empty for {} tokens", n_tokens);
            }
        }
    }

    #[test]
    fn follow_set_of_start_contains_eof(n_tokens in 1usize..6) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        if let Some(start) = grammar.start_symbol() {
            if let Some(follow) = ff.follow(start) {
                // EOF is SymbolId(0)
                prop_assert!(follow.contains(0),
                    "Follow(start) should contain EOF");
            }
        }
    }

    #[test]
    fn chain_grammar_first_propagates(depth in 1usize..5) {
        let (grammar, ff) = build_chain_grammar(depth);
        if let Some(start) = grammar.start_symbol() {
            let first = ff.first(start);
            prop_assert!(first.is_some(),
                "Chain grammar FIRST(start) should exist at depth {}", depth);
            if let Some(f) = first {
                prop_assert!(f.count_ones(..) > 0,
                    "Chain grammar FIRST(start) should not be empty at depth {}", depth);
            }
        }
    }

    #[test]
    fn automaton_builds_for_valid_grammar(n_tokens in 1usize..5) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok(),
            "Automaton should build for valid grammar with {} tokens", n_tokens);
    }

    #[test]
    fn parse_table_has_at_least_two_states(n_tokens in 1usize..5) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        let table = build_lr1_automaton(&grammar, &ff).unwrap();
        prop_assert!(table.action_table.len() >= 2,
            "Parse table should have >= 2 states, got {}", table.action_table.len());
    }

    #[test]
    fn action_and_goto_table_same_row_count(n_tokens in 1usize..5) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        let table = build_lr1_automaton(&grammar, &ff).unwrap();
        prop_assert_eq!(table.action_table.len(), table.goto_table.len(),
            "Action and goto tables should have the same number of states");
    }

    #[test]
    fn action_table_columns_consistent(n_tokens in 1usize..5) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        let table = build_lr1_automaton(&grammar, &ff).unwrap();
        if let Some(first_row) = table.action_table.first() {
            let col_count = first_row.len();
            for (i, row) in table.action_table.iter().enumerate() {
                prop_assert_eq!(row.len(), col_count,
                    "State {} has {} columns, expected {}", i, row.len(), col_count);
            }
        }
    }

    #[test]
    fn eof_symbol_mapped(n_tokens in 1usize..5) {
        let (grammar, ff) = build_choice_grammar(n_tokens);
        let table = build_lr1_automaton(&grammar, &ff).unwrap();
        // EOF symbol should be mapped in either symbol_to_index or index_to_symbol
        let eof = table.eof_symbol;
        prop_assert!(table.symbol_to_index.contains_key(&eof) ||
            table.index_to_symbol.contains(&eof),
            "EOF {:?} should be in symbol maps", eof);
    }

    #[test]
    fn precedence_grammars_build(prec in -10i16..10) {
        let mut grammar = GrammarBuilder::new("test")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["num"])
            .rule_with_precedence(
                "expr",
                vec!["expr", "plus", "expr"],
                prec,
                Associativity::Left,
            )
            .start("expr")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok(), "Should build with precedence {}", prec);
    }
}

// ── Fixed tests ──

#[test]
fn left_recursive_grammar_computes() {
    let mut grammar = GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "num"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    assert!(table.action_table.len() >= 2);
}

#[test]
fn right_recursive_grammar_computes() {
    let mut grammar = GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["num", "plus", "expr"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    assert!(table.action_table.len() >= 2);
}

#[test]
fn optional_grammar_with_epsilon() {
    let mut grammar = GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .rule("start", vec!["num"])
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    assert!(table.action_table.len() >= 1);
}

#[test]
fn symbol_to_index_and_index_to_symbol_inverse() {
    let mut grammar = GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();

    for (&sym, &idx) in &table.symbol_to_index {
        assert_eq!(table.index_to_symbol[idx], sym);
    }
}
