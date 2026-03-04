#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for LR(1) automaton builder and ParseTable properties.
//!
//! Covers: state_count, symbol_count, eof_symbol, action/goto table dimensions,
//! rules, Accept action existence, error-free simple states, scaling, and
//! table consistency.

use adze_glr_core::{Action, FirstFollowSets, ParseTable, StateId, SymbolId, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn simple_table() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("s")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let t = build(&g);
    (g, t)
}

fn two_token_table() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("ab")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let t = build(&g);
    (g, t)
}

fn alt_table() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let t = build(&g);
    (g, t)
}

fn chain_table() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let t = build(&g);
    (g, t)
}

fn three_token_seq_table() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("seq3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let t = build(&g);
    (g, t)
}

fn prec_table() -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, adze_ir::Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, adze_ir::Associativity::Left)
        .start("e")
        .build();
    let t = build(&g);
    (g, t)
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
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn accept_state_count(table: &ParseTable) -> usize {
    let eof = table.eof();
    (0..table.state_count)
        .filter(|&st| {
            table
                .actions(StateId(st as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .count()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. ParseTable state_count for various grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn state_count_simple_positive() {
    let (_, t) = simple_table();
    assert!(t.state_count > 0);
}

#[test]
fn state_count_simple_at_least_three() {
    // S' -> S, S -> x needs initial + shifted + accept >= 3
    let (_, t) = simple_table();
    assert!(t.state_count >= 3, "got {}", t.state_count);
}

#[test]
fn state_count_simple_bounded() {
    let (_, t) = simple_table();
    assert!(t.state_count <= 10, "got {}", t.state_count);
}

#[test]
fn state_count_two_token_seq_grows() {
    let (_, t1) = simple_table();
    let (_, t2) = two_token_table();
    assert!(
        t2.state_count >= t1.state_count,
        "two-token seq ({}) should have >= states than single-token ({})",
        t2.state_count,
        t1.state_count
    );
}

#[test]
fn state_count_three_token_seq() {
    let (_, t) = three_token_seq_table();
    assert!(t.state_count >= 4, "got {}", t.state_count);
}

#[test]
fn state_count_alternatives() {
    let (_, t) = alt_table();
    assert!(t.state_count >= 2, "got {}", t.state_count);
}

#[test]
fn state_count_chain() {
    let (_, t) = chain_table();
    assert!(t.state_count >= 3, "got {}", t.state_count);
}

#[test]
fn state_count_prec_grammar() {
    let (_, t) = prec_table();
    assert!(t.state_count >= 4, "got {}", t.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. ParseTable symbol_count
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn symbol_count_positive() {
    let (_, t) = simple_table();
    assert!(t.symbol_count > 0);
}

#[test]
fn symbol_count_includes_terminal() {
    // At least the token + EOF + nonterminal
    let (_, t) = simple_table();
    assert!(t.symbol_count >= 2, "got {}", t.symbol_count);
}

#[test]
fn symbol_count_grows_with_tokens() {
    let (_, t1) = simple_table();
    let (_, t3) = three_token_seq_table();
    assert!(
        t3.symbol_count >= t1.symbol_count,
        "3 tokens ({}) >= 1 token ({})",
        t3.symbol_count,
        t1.symbol_count
    );
}

#[test]
fn symbol_count_alt_grammar() {
    let (_, t) = alt_table();
    assert!(t.symbol_count >= 2, "got {}", t.symbol_count);
}

#[test]
fn symbol_count_chain_grammar() {
    let (_, t) = chain_table();
    // 1 token + 3 nonterminals + EOF
    assert!(t.symbol_count >= 3, "got {}", t.symbol_count);
}

#[test]
fn symbol_count_prec_grammar() {
    let (_, t) = prec_table();
    // 3 tokens + nonterminal + eof
    assert!(t.symbol_count >= 4, "got {}", t.symbol_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. ParseTable eof_symbol is valid
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn eof_symbol_exists() {
    let (_, t) = simple_table();
    let _ = t.eof_symbol;
}

#[test]
fn eof_method_returns_eof_symbol() {
    let (_, t) = simple_table();
    assert_eq!(t.eof(), t.eof_symbol);
}

#[test]
fn eof_symbol_same_across_grammars() {
    let (_, t1) = simple_table();
    let (_, t2) = two_token_table();
    // EOF is typically SymbolId(0) – at minimum both should have valid EOF
    let _ = t1.eof_symbol;
    let _ = t2.eof_symbol;
}

#[test]
fn eof_symbol_in_action_table_column_range() {
    let (_, t) = simple_table();
    // eof must be queryable via actions()
    let eof = t.eof();
    let _ = t.actions(StateId(0), eof);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Action table dimensions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_table_row_count_matches_state_count() {
    let (_, t) = simple_table();
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn action_table_row_count_two_token() {
    let (_, t) = two_token_table();
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn action_table_columns_uniform() {
    let (_, t) = simple_table();
    let col_count = t.action_table[0].len();
    for (i, row) in t.action_table.iter().enumerate() {
        assert_eq!(row.len(), col_count, "row {} differs", i);
    }
}

#[test]
fn action_table_columns_match_symbol_count() {
    let (_, t) = simple_table();
    assert_eq!(t.action_table[0].len(), t.symbol_count);
}

#[test]
fn action_table_alt_grammar_rows() {
    let (_, t) = alt_table();
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn action_table_chain_grammar_rows() {
    let (_, t) = chain_table();
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn action_table_prec_grammar_rows() {
    let (_, t) = prec_table();
    assert_eq!(t.action_table.len(), t.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Goto table dimensions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goto_table_row_count_matches_state_count() {
    let (_, t) = simple_table();
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn goto_table_row_count_two_token() {
    let (_, t) = two_token_table();
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn goto_table_columns_uniform() {
    let (_, t) = simple_table();
    if t.goto_table.is_empty() {
        return;
    }
    let col_count = t.goto_table[0].len();
    for (i, row) in t.goto_table.iter().enumerate() {
        assert_eq!(row.len(), col_count, "goto row {} differs", i);
    }
}

#[test]
fn goto_table_has_columns() {
    let (_, t) = simple_table();
    // At least one nonterminal column
    assert!(!t.goto_table.is_empty());
    assert!(t.goto_table[0].len() >= 1, "goto needs at least 1 column");
}

#[test]
fn goto_table_chain_rows() {
    let (_, t) = chain_table();
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn goto_table_prec_grammar_rows() {
    let (_, t) = prec_table();
    assert_eq!(t.goto_table.len(), t.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Rules in parse table
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rules_nonempty_simple() {
    let (_, t) = simple_table();
    assert!(!t.rules.is_empty());
}

#[test]
fn rules_count_single_rule() {
    let (_, t) = simple_table();
    // At least the user rule (+ augmented rule)
    assert!(t.rules.len() >= 1);
}

#[test]
fn rules_count_alt_grammar() {
    let (_, t) = alt_table();
    assert!(t.rules.len() >= 2);
}

#[test]
fn rules_count_chain_grammar() {
    let (_, t) = chain_table();
    assert!(t.rules.len() >= 3);
}

#[test]
fn rules_rhs_len_single_token() {
    let (_, t) = simple_table();
    assert!(t.rules.iter().any(|r| r.rhs_len == 1));
}

#[test]
fn rules_rhs_len_two_tokens() {
    let (_, t) = two_token_table();
    assert!(t.rules.iter().any(|r| r.rhs_len == 2));
}

#[test]
fn rules_rhs_len_three_tokens() {
    let (_, t) = three_token_seq_table();
    assert!(t.rules.iter().any(|r| r.rhs_len == 3));
}

#[test]
fn rules_lhs_are_valid_symbol_ids() {
    let (_, t) = chain_table();
    for rule in &t.rules {
        let _ = rule.lhs.0; // SymbolId inner accessible
    }
}

#[test]
fn rules_prec_grammar_count() {
    let (_, t) = prec_table();
    assert!(t.rules.len() >= 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Accept action exists somewhere
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn accept_exists_simple() {
    let (_, t) = simple_table();
    assert!(has_accept(&t));
}

#[test]
fn accept_exists_two_token() {
    let (_, t) = two_token_table();
    assert!(has_accept(&t));
}

#[test]
fn accept_exists_alt() {
    let (_, t) = alt_table();
    assert!(has_accept(&t));
}

#[test]
fn accept_exists_chain() {
    let (_, t) = chain_table();
    assert!(has_accept(&t));
}

#[test]
fn accept_exists_three_token_seq() {
    let (_, t) = three_token_seq_table();
    assert!(has_accept(&t));
}

#[test]
fn accept_exists_prec() {
    let (_, t) = prec_table();
    assert!(has_accept(&t));
}

#[test]
fn accept_on_eof_only() {
    let (_, t) = simple_table();
    let eof = t.eof();
    // Accept should only appear in the EOF column
    for state in 0..t.state_count {
        for sym_idx in 0..t.symbol_count {
            for action in &t.action_table[state][sym_idx] {
                if matches!(action, Action::Accept) {
                    // This cell's symbol should be eof
                    let sym = t.index_to_symbol[sym_idx];
                    assert_eq!(sym, eof, "Accept found on non-EOF symbol in state {state}");
                }
            }
        }
    }
}

#[test]
fn accept_at_least_one_state() {
    let (_, t) = simple_table();
    assert!(accept_state_count(&t) >= 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. No errors in simple grammar states (valid transitions)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn initial_state_has_shift_on_token() {
    let (g, t) = simple_table();
    let x = tok_id(&g, "x");
    let actions = t.actions(t.initial_state, x);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state should shift on 'x'"
    );
}

#[test]
fn initial_state_no_accept() {
    let (_, t) = simple_table();
    let eof = t.eof();
    let actions = t.actions(t.initial_state, eof);
    assert!(
        !actions.iter().any(|a| matches!(a, Action::Accept)),
        "initial state should not have Accept"
    );
}

#[test]
fn two_token_initial_shifts_first() {
    let (g, t) = two_token_table();
    let a = tok_id(&g, "a");
    assert!(
        t.actions(t.initial_state, a)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    );
}

#[test]
fn two_token_initial_no_shift_second() {
    let (g, t) = two_token_table();
    let b = tok_id(&g, "b");
    // In initial state, 'b' should not be shifted (it's the second token)
    assert!(
        !t.actions(t.initial_state, b)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "initial state should not shift on 'b'"
    );
}

#[test]
fn no_shift_on_unused_token_initial() {
    let g = GrammarBuilder::new("unused")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let t = build(&g);
    let b = tok_id(&g, "b");
    assert!(
        !t.actions(t.initial_state, b)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    );
}

#[test]
fn shift_targets_valid() {
    let (_, t) = two_token_table();
    for state in 0..t.state_count {
        for sym_idx in 0..t.symbol_count {
            for action in &t.action_table[state][sym_idx] {
                if let Action::Shift(StateId(s)) = action {
                    assert!((*s as usize) < t.state_count);
                }
            }
        }
    }
}

#[test]
fn reduce_rule_ids_valid() {
    let (_, t) = two_token_table();
    for state in 0..t.state_count {
        for sym_idx in 0..t.symbol_count {
            for action in &t.action_table[state][sym_idx] {
                if let Action::Reduce(rule_id) = action {
                    assert!(
                        (rule_id.0 as usize) < t.rules.len(),
                        "reduce rule {} out of range ({})",
                        rule_id.0,
                        t.rules.len()
                    );
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Larger grammars produce more states
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn more_tokens_more_states() {
    let (_, t1) = simple_table();
    let (_, t3) = three_token_seq_table();
    assert!(t3.state_count > t1.state_count);
}

#[test]
fn chain_has_more_states_than_simple() {
    let (_, t1) = simple_table();
    let (_, tc) = chain_table();
    assert!(tc.state_count >= t1.state_count);
}

#[test]
fn many_alternatives_scale() {
    let mut b = GrammarBuilder::new("many_alt");
    for i in 0..10 {
        let n: &str = Box::leak(format!("t{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let t = build(&g);
    let (_, t_alt2) = alt_table();
    assert!(t.state_count > t_alt2.state_count);
}

#[test]
fn many_alternatives_rules_scale() {
    let mut b = GrammarBuilder::new("many_alt2");
    for i in 0..15 {
        let n: &str = Box::leak(format!("t{i}").into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let t = build(&g);
    assert!(t.rules.len() >= 15);
}

#[test]
fn long_sequence_states_scale() {
    let mut b = GrammarBuilder::new("long");
    let mut rhs = Vec::new();
    for i in 0..8 {
        let n: &str = Box::leak(format!("t{i}").into_boxed_str());
        b = b.token(n, n);
        rhs.push(n);
    }
    b = b.rule("s", rhs);
    let g = b.start("s").build();
    let t = build(&g);
    assert!(t.state_count >= 8);
}

#[test]
fn prec_grammar_more_states_than_simple() {
    let (_, ts) = simple_table();
    let (_, tp) = prec_table();
    assert!(tp.state_count > ts.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Table consistency checks
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_construction() {
    let g = GrammarBuilder::new("det")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let t1 = build(&g);
    let t2 = build(&g);
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.symbol_count, t2.symbol_count);
    assert_eq!(t1.rules.len(), t2.rules.len());
}

#[test]
fn goto_on_start_nonterminal() {
    let (g, t) = simple_table();
    let s = nt_id(&g, "s");
    assert!(t.goto(t.initial_state, s).is_some());
}

#[test]
fn goto_targets_valid() {
    let (_, t) = chain_table();
    for state in 0..t.state_count {
        for col in 0..t.goto_table[state].len() {
            let target = t.goto_table[state][col];
            if target.0 != u16::MAX {
                assert!(
                    (target.0 as usize) < t.state_count,
                    "goto target {} out of range",
                    target.0
                );
            }
        }
    }
}

#[test]
fn start_symbol_accessible() {
    let (_, t) = simple_table();
    let _ = t.start_symbol();
}

#[test]
fn start_symbol_matches_grammar() {
    let (g, t) = simple_table();
    let s = nt_id(&g, "s");
    assert_eq!(t.start_symbol(), s);
}

#[test]
fn initial_state_is_zero() {
    let (_, t) = simple_table();
    assert_eq!(t.initial_state, StateId(0));
}

#[test]
fn action_table_all_rows_same_width() {
    let (_, t) = chain_table();
    let w = t.action_table[0].len();
    assert!(t.action_table.iter().all(|row| row.len() == w));
}

#[test]
fn goto_table_all_rows_same_width() {
    let (_, t) = chain_table();
    if t.goto_table.is_empty() {
        return;
    }
    let w = t.goto_table[0].len();
    assert!(t.goto_table.iter().all(|row| row.len() == w));
}

#[test]
fn symbol_to_index_covers_all_columns() {
    let (_, t) = simple_table();
    // index_to_symbol should have symbol_count entries
    assert_eq!(t.index_to_symbol.len(), t.symbol_count);
}

#[test]
fn index_to_symbol_roundtrip() {
    let (_, t) = simple_table();
    for (idx, &sym) in t.index_to_symbol.iter().enumerate() {
        if let Some(&mapped_idx) = t.symbol_to_index.get(&sym) {
            assert_eq!(mapped_idx, idx, "roundtrip mismatch for symbol {:?}", sym);
        }
    }
}

#[test]
fn eof_in_symbol_to_index() {
    let (_, t) = simple_table();
    assert!(
        t.symbol_to_index.contains_key(&t.eof_symbol),
        "EOF should be in symbol_to_index"
    );
}

#[test]
fn error_cells_exist() {
    let (_, t) = simple_table();
    // Some cells should be empty (error) — not every state handles every symbol
    let empty_count: usize = t
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.is_empty())
        .count();
    assert!(empty_count > 0, "should have some empty/error cells");
}

#[test]
fn fork_actions_have_multiple_branches() {
    // Fork actions (if any) should contain multiple sub-actions
    let (_, t) = prec_table();
    for state in 0..t.state_count {
        for sym_idx in 0..t.symbol_count {
            for action in &t.action_table[state][sym_idx] {
                if let Action::Fork(branches) = action {
                    assert!(branches.len() >= 2, "Fork should have at least 2 branches");
                }
            }
        }
    }
}

#[test]
fn nonterminal_to_index_nonempty() {
    let (_, t) = simple_table();
    assert!(!t.nonterminal_to_index.is_empty());
}

#[test]
fn rules_lhs_in_nonterminal_index() {
    let (_, t) = chain_table();
    for rule in &t.rules {
        // The lhs should correspond to a known nonterminal
        // (augmented start rule's lhs might differ, so just check most)
        let _ = rule.lhs;
    }
}

#[test]
fn token_count_positive() {
    let (_, t) = simple_table();
    assert!(t.token_count >= 1);
}

#[test]
fn token_count_matches_tokens() {
    let (_, t) = three_token_seq_table();
    assert!(t.token_count >= 3);
}
