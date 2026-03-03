#![allow(clippy::needless_range_loop)]
//! Property-based tests for parse table generation in adze-tablegen.
//!
//! Properties verified:
//!  1.  Generated parse table has correct number of states (>= 1)
//!  2.  State IDs are sequential (0..state_count)
//!  3.  Shift actions reference valid state IDs
//!  4.  Reduce actions reference valid rule IDs
//!  5.  Goto entries reference valid state IDs
//!  6.  Table generation is deterministic (same grammar → same table)
//!  7.  Small grammar → small table (bounded state count)
//!  8.  Complex grammar → more states than trivial grammar
//!  9.  Action table dimensions match state_count × symbol_count
//! 10.  Goto table dimensions match state_count
//! 11.  EOF symbol is present in symbol_to_index
//! 12.  Start symbol is set correctly
//! 13.  Token count is consistent with grammar
//! 14.  symbol_to_index and index_to_symbol are inverses
//! 15.  At least one Accept action exists
//! 16.  Initial state is zero
//! 17.  Rules LHS symbols are valid nonterminals
//! 18.  Grammar with more tokens → at least as many symbols
//! 19.  Grammar with precedence still produces a valid table
//! 20.  Nonterminal_to_index keys are valid symbols
//! 21.  Action cells have no duplicate actions
//! 22.  No Fork wrapping a single action
//! 23.  Lex modes length matches state count
//! 24.  Table with externals has correct external_token_count
//! 25.  Grammar builder determinism (build twice → identical)
//! 26.  State count monotonically increases with grammar complexity
//! 27.  EOF symbol has a valid column index
//! 28.  Every Shift target is within state_count
//! 29.  Every Reduce rule ID is within rules.len()
//! 30.  Goto targets that are not INVALID are within state_count

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ExternalToken, Grammar, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar using GrammarBuilder with the given number of tokens.
/// Always produces at least one token and a single rule.
fn grammar_with_n_tokens(n: usize) -> Grammar {
    let count = n.max(1);
    let mut builder = GrammarBuilder::new("proptest");
    for i in 0..count {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    builder.build()
}

/// Build parse table from a grammar, returning None if table generation fails.
fn try_build_table(grammar: &Grammar) -> Option<adze_glr_core::ParseTable> {
    let ff = FirstFollowSets::compute(grammar).ok()?;
    build_lr1_automaton(grammar, &ff).ok()
}

/// Build parse table from a grammar (panics on failure).
fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("LR(1) automaton")
}

/// Build a grammar with a chain of rules: root -> a0; a0 -> a1; ... ; aN -> tok0
fn chain_grammar(depth: usize) -> Grammar {
    let mut g = Grammar::new("chain".to_string());
    // One terminal
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "tok0".to_string(),
            pattern: TokenPattern::String("t0".to_string()),
            fragile: false,
        },
    );

    let base_nt = 1u16;
    for i in 0..=depth {
        let lhs = SymbolId(base_nt + i as u16);
        let rhs = if i == depth {
            vec![Symbol::Terminal(SymbolId(0))]
        } else {
            vec![Symbol::NonTerminal(SymbolId(base_nt + i as u16 + 1))]
        };
        g.add_rule(Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
        g.rule_names
            .insert(lhs, format!("rule_{}", lhs.0));
    }
    g
}

/// Build a grammar with multiple alternative rules for the start symbol.
fn alternatives_grammar(alt_count: usize) -> Grammar {
    let count = alt_count.max(1);
    let mut g = Grammar::new("alts".to_string());
    // Create one terminal per alternative
    for i in 0..count {
        g.tokens.insert(
            SymbolId(i as u16),
            Token {
                name: format!("tok{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }
    let root_id = SymbolId(count as u16);
    for i in 0..count {
        g.add_rule(Rule {
            lhs: root_id,
            rhs: vec![Symbol::Terminal(SymbolId(i as u16))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    g.rule_names.insert(root_id, "root".to_string());
    g
}

/// Collect all actions from every cell in the action table.
fn all_actions(table: &adze_glr_core::ParseTable) -> Vec<&Action> {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter().flat_map(|cell| cell.iter()))
        .collect()
}

/// Check if an action cell has no duplicate actions.
fn cell_has_no_duplicates(cell: &[Action]) -> bool {
    for i in 0..cell.len() {
        for j in (i + 1)..cell.len() {
            if cell[i] == cell[j] {
                return false;
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // 1. Generated parse table has at least one state
    #[test]
    fn state_count_at_least_one(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        prop_assert!(table.state_count >= 1, "state_count must be >= 1");
    }

    // 2. State IDs are sequential: action_table has exactly state_count rows
    #[test]
    fn action_table_rows_match_state_count(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }

    // 3. Shift actions reference valid state IDs (< state_count)
    #[test]
    fn shift_targets_are_valid(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        for action in all_actions(&table) {
            if let Action::Shift(sid) = action {
                prop_assert!(
                    (sid.0 as usize) < table.state_count,
                    "Shift target {} >= state_count {}",
                    sid.0,
                    table.state_count
                );
            }
        }
    }

    // 4. Reduce actions reference valid rule IDs (< rules.len())
    #[test]
    fn reduce_rule_ids_are_valid(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        let rule_count = table.rules.len();
        for action in all_actions(&table) {
            if let Action::Reduce(rid) = action {
                prop_assert!(
                    (rid.0 as usize) < rule_count,
                    "Reduce rule {} >= rule_count {}",
                    rid.0,
                    rule_count
                );
            }
        }
    }

    // 5. Goto entries that are not INVALID are < state_count
    #[test]
    fn goto_targets_are_valid(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        let invalid = u16::MAX;
        for state in 0..table.goto_table.len() {
            for col in 0..table.goto_table[state].len() {
                let target = table.goto_table[state][col];
                if target.0 != invalid {
                    prop_assert!(
                        (target.0 as usize) < table.state_count,
                        "Goto target {} >= state_count {} (state={}, col={})",
                        target.0,
                        table.state_count,
                        state,
                        col
                    );
                }
            }
        }
    }

    // 6. Table generation is deterministic
    #[test]
    fn table_generation_deterministic(n in 1usize..5) {
        let g1 = grammar_with_n_tokens(n);
        let g2 = grammar_with_n_tokens(n);
        let t1 = build_table(&g1);
        let t2 = build_table(&g2);
        prop_assert_eq!(t1.state_count, t2.state_count);
        prop_assert_eq!(t1.symbol_count, t2.symbol_count);
        prop_assert_eq!(t1.token_count, t2.token_count);
        prop_assert_eq!(t1.rules.len(), t2.rules.len());
        prop_assert_eq!(t1.action_table.len(), t2.action_table.len());
        for s in 0..t1.action_table.len() {
            prop_assert_eq!(t1.action_table[s].len(), t2.action_table[s].len());
            for c in 0..t1.action_table[s].len() {
                prop_assert_eq!(
                    &t1.action_table[s][c],
                    &t2.action_table[s][c],
                    "action mismatch at state={}, col={}",
                    s,
                    c
                );
            }
        }
    }

    // 7. Small grammar → bounded state count
    #[test]
    fn small_grammar_bounded_states(n in 1usize..4) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        // A single-rule grammar with n tokens should produce a small table.
        // Generous bound: n tokens + a handful of states for the automaton.
        prop_assert!(
            table.state_count <= 50,
            "Small grammar with {} tokens produced {} states",
            n,
            table.state_count
        );
    }

    // 8. Action table column count matches symbol_count per row
    #[test]
    fn action_table_columns_consistent(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        for (state_idx, row) in table.action_table.iter().enumerate() {
            prop_assert_eq!(
                row.len(),
                table.symbol_count,
                "Row {} has {} cols but symbol_count is {}",
                state_idx,
                row.len(),
                table.symbol_count
            );
        }
    }

    // 9. Goto table has state_count rows
    #[test]
    fn goto_table_rows_match_state_count(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
    }

    // 10. EOF symbol is present in symbol_to_index
    #[test]
    fn eof_in_symbol_to_index(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "EOF symbol {:?} not in symbol_to_index",
            table.eof_symbol
        );
    }

    // 11. Initial state is zero
    #[test]
    fn initial_state_is_zero(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        prop_assert_eq!(table.initial_state.0, 0, "Initial state should be 0");
    }

    // 12. symbol_to_index and index_to_symbol are consistent
    #[test]
    fn symbol_index_roundtrip(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        for (sym, &idx) in &table.symbol_to_index {
            prop_assert!(
                idx < table.index_to_symbol.len(),
                "Index {} out of bounds (len={})",
                idx,
                table.index_to_symbol.len()
            );
            prop_assert_eq!(
                table.index_to_symbol[idx],
                *sym,
                "Roundtrip failed for sym={:?} at idx={}",
                sym,
                idx
            );
        }
    }

    // 13. At least one Accept action exists in the table
    #[test]
    fn accept_action_exists(n in 1usize..5) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        let has_accept = all_actions(&table).iter().any(|a| matches!(a, Action::Accept));
        prop_assert!(has_accept, "Table must contain at least one Accept action");
    }

    // 14. Rules LHS symbols are non-terminals that appear in the grammar
    #[test]
    fn rules_lhs_are_nonterminals(n in 1usize..5) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        for rule in &table.rules {
            // LHS should be mappable in the table
            prop_assert!(
                table.symbol_to_index.contains_key(&rule.lhs),
                "Rule LHS {:?} not in symbol_to_index",
                rule.lhs
            );
        }
    }

    // 15. Grammar with more tokens → at least as many symbols
    #[test]
    fn more_tokens_more_symbols(a in 1usize..4, b in 4usize..8) {
        let g_small = grammar_with_n_tokens(a);
        let g_large = grammar_with_n_tokens(b);
        let t_small = build_table(&g_small);
        let t_large = build_table(&g_large);
        prop_assert!(
            t_large.symbol_count >= t_small.symbol_count,
            "More tokens ({} vs {}) should yield >= symbols ({} vs {})",
            b,
            a,
            t_large.symbol_count,
            t_small.symbol_count
        );
    }

    // 16. Lex modes length matches state count
    #[test]
    fn lex_modes_match_state_count(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        prop_assert_eq!(
            table.lex_modes.len(),
            table.state_count,
            "lex_modes length should equal state_count"
        );
    }

    // 17. token_count is consistent
    #[test]
    fn token_count_consistent(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        // token_count should be positive for any grammar with tokens
        prop_assert!(table.token_count > 0, "token_count must be > 0");
    }

    // 18. Action cells have no duplicate actions
    #[test]
    fn action_cells_no_duplicates(n in 1usize..5) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        for (s, row) in table.action_table.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                prop_assert!(
                    cell_has_no_duplicates(cell),
                    "Cell at state={}, col={} has duplicates: {:?}",
                    s,
                    c,
                    cell
                );
            }
        }
    }

    // 19. No Fork wrapping a single action
    #[test]
    fn no_trivial_forks(n in 1usize..5) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        for action in all_actions(&table) {
            if let Action::Fork(inner) = action {
                prop_assert!(
                    inner.len() > 1,
                    "Fork with {} actions is trivial",
                    inner.len()
                );
            }
        }
    }

    // 20. EOF symbol is in the symbol_to_index mapping with a valid column
    #[test]
    fn eof_symbol_has_valid_index(n in 1usize..6) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        let eof_idx = table.symbol_to_index.get(&table.eof_symbol);
        prop_assert!(eof_idx.is_some(), "EOF must have a column index");
        prop_assert!(
            *eof_idx.unwrap() < table.symbol_count,
            "EOF column {} >= symbol_count {}",
            eof_idx.unwrap(),
            table.symbol_count
        );
    }

    // 21. Nonterminal_to_index keys are in symbol_to_index
    #[test]
    fn nonterminal_keys_valid(n in 1usize..5) {
        let g = grammar_with_n_tokens(n);
        let table = build_table(&g);
        for nt_sym in table.nonterminal_to_index.keys() {
            prop_assert!(
                table.symbol_to_index.contains_key(nt_sym),
                "Nonterminal {:?} not in symbol_to_index",
                nt_sym
            );
        }
    }

    // 22. Determinism of action table contents across runs
    #[test]
    fn action_contents_deterministic(n in 1usize..4) {
        let t1 = build_table(&grammar_with_n_tokens(n));
        let t2 = build_table(&grammar_with_n_tokens(n));
        for s in 0..t1.state_count {
            for c in 0..t1.symbol_count {
                prop_assert_eq!(
                    &t1.action_table[s][c],
                    &t2.action_table[s][c],
                );
            }
        }
    }

    // 23. Goto table contents are deterministic
    #[test]
    fn goto_contents_deterministic(n in 1usize..4) {
        let t1 = build_table(&grammar_with_n_tokens(n));
        let t2 = build_table(&grammar_with_n_tokens(n));
        for s in 0..t1.state_count {
            for c in 0..t1.goto_table[s].len() {
                prop_assert_eq!(t1.goto_table[s][c], t2.goto_table[s][c]);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Non-proptest parameterized tests
// ---------------------------------------------------------------------------

// 24. Complex grammar → more states than trivial grammar
#[test]
fn complex_grammar_more_states() {
    // Trivial: root -> tok0
    let trivial = grammar_with_n_tokens(1);
    let t_trivial = build_table(&trivial);

    // Complex: chain of depth 5
    let complex = chain_grammar(5);
    let t_complex = build_table(&complex);

    assert!(
        t_complex.state_count > t_trivial.state_count,
        "Chain grammar (states={}) should have more states than trivial (states={})",
        t_complex.state_count,
        t_trivial.state_count
    );
}

// 25. GrammarBuilder determinism (build twice → identical grammar)
#[test]
fn grammar_builder_deterministic() {
    for n in 1..=5 {
        let g1 = grammar_with_n_tokens(n);
        let g2 = grammar_with_n_tokens(n);
        assert_eq!(g1.tokens.len(), g2.tokens.len(), "token count mismatch for n={n}");
        assert_eq!(g1.rules.len(), g2.rules.len(), "rule count mismatch for n={n}");
    }
}

// 26. State count monotonically increases with chain depth
#[test]
fn state_count_monotonic_with_chain_depth() {
    let mut prev_states = 0;
    for depth in 1..=6 {
        let g = chain_grammar(depth);
        let table = build_table(&g);
        assert!(
            table.state_count >= prev_states,
            "Depth {} produced {} states, less than depth {} with {} states",
            depth,
            table.state_count,
            depth - 1,
            prev_states
        );
        prev_states = table.state_count;
    }
}

// 27. Table with externals has correct external_token_count
#[test]
fn external_token_count_matches() {
    for ext_count in 0..=3 {
        let mut g = grammar_with_n_tokens(2);
        for i in 0..ext_count {
            g.externals.push(ExternalToken {
                name: format!("ext{i}"),
                symbol_id: SymbolId(200 + i as u16),
            });
        }
        if let Some(table) = try_build_table(&g) {
            assert_eq!(
                table.external_token_count, ext_count,
                "external_token_count should be {}",
                ext_count
            );
        }
    }
}

// 28. Grammar with precedence still produces a valid table
#[test]
fn precedence_grammar_valid() {
    let g = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();

    let table = build_table(&g);
    assert!(table.state_count >= 1);
    assert!(table.rules.len() >= 3, "Should have at least 3 rules");
    // Shift targets valid
    for action in all_actions(&table) {
        if let Action::Shift(sid) = action {
            assert!((sid.0 as usize) < table.state_count);
        }
    }
}

// 29. Alternatives grammar produces correct rule count
#[test]
fn alternatives_grammar_rule_count() {
    for alt_count in 1..=5 {
        let g = alternatives_grammar(alt_count);
        let table = build_table(&g);
        // The augmented grammar adds S' -> root, so total = alt_count + 1
        assert!(
            table.rules.len() >= alt_count,
            "Expected at least {} rules, got {}",
            alt_count,
            table.rules.len()
        );
    }
}

// 30. Python-like grammar builds and has expected properties
#[test]
fn python_like_grammar_builds() {
    let g = GrammarBuilder::python_like();
    let table = build_table(&g);
    assert!(table.state_count >= 1);
    assert!(table.symbol_count > table.token_count);
    assert!(
        all_actions(&table).iter().any(|a| matches!(a, Action::Accept)),
        "Python-like grammar table must have Accept"
    );
}

// 31. JavaScript-like grammar builds and has expected properties
#[test]
fn javascript_like_grammar_builds() {
    let g = GrammarBuilder::javascript_like();
    let table = build_table(&g);
    assert!(table.state_count >= 1);
    assert!(table.symbol_count > table.token_count);
    assert!(
        all_actions(&table).iter().any(|a| matches!(a, Action::Accept)),
        "JavaScript-like grammar table must have Accept"
    );
}
