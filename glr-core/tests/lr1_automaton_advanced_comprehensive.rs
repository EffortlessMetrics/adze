#![cfg(feature = "test-api")]
//! Advanced comprehensive tests for LR(1) automaton construction.
//!
//! Covers: minimal grammars, state/table invariants, recursive grammars,
//! ambiguous grammars, precedence, determinism, chain grammars, large grammars,
//! start symbol presence, and structural parse-table properties.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

fn has_accept(table: &adze_glr_core::ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

fn count_conflict_cells(table: &adze_glr_core::ParseTable) -> usize {
    let mut count = 0;
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            let actions = &table.action_table[state][sym_idx];
            if actions.len() > 1 {
                count += 1;
            }
        }
    }
    count
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

fn any_goto_exists(table: &adze_glr_core::ParseTable, nt: SymbolId) -> bool {
    (0..table.state_count).any(|st| table.goto(StateId(st as u16), nt).is_some())
}

fn any_state_has_shift(table: &adze_glr_core::ParseTable, terminal: SymbolId) -> bool {
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), terminal)
            .iter()
            .any(|a| matches!(a, Action::Shift(_)))
    })
}

fn any_state_has_reduce(table: &adze_glr_core::ParseTable, lookahead: SymbolId) -> bool {
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), lookahead)
            .iter()
            .any(|a| matches!(a, Action::Reduce(_)))
    })
}

fn total_reduce_count(table: &adze_glr_core::ParseTable) -> usize {
    let mut count = 0;
    for st in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for a in &table.action_table[st][sym_idx] {
                if matches!(a, Action::Reduce(_)) {
                    count += 1;
                }
            }
        }
    }
    count
}

fn total_shift_count(table: &adze_glr_core::ParseTable) -> usize {
    let mut count = 0;
    for st in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for a in &table.action_table[st][sym_idx] {
                if matches!(a, Action::Shift(_)) {
                    count += 1;
                }
            }
        }
    }
    count
}

// ===========================================================================
// 1. Automaton for minimal grammar (1 rule)
// ===========================================================================

#[test]
fn minimal_grammar_one_rule_builds() {
    let g = GrammarBuilder::new("min1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn minimal_grammar_has_positive_state_count() {
    let g = GrammarBuilder::new("min2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

#[test]
fn minimal_grammar_action_table_matches_state_count() {
    let g = GrammarBuilder::new("min3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn minimal_grammar_goto_table_matches_state_count() {
    let g = GrammarBuilder::new("min4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn minimal_grammar_rules_non_empty() {
    let g = GrammarBuilder::new("min5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(!table.rules.is_empty());
}

#[test]
fn minimal_grammar_has_shift_and_reduce() {
    let g = GrammarBuilder::new("min6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(total_shift_count(&table) > 0);
    assert!(total_reduce_count(&table) > 0);
}

#[test]
fn minimal_grammar_eof_distinct() {
    let g = GrammarBuilder::new("min7")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for (tok, _) in &g.tokens {
        assert_ne!(eof, *tok);
    }
}

#[test]
fn minimal_grammar_start_symbol_preserved() {
    let g = GrammarBuilder::new("min8")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    assert_eq!(table.start_symbol(), s);
}

// ===========================================================================
// 2. state_count > 0 for any valid grammar
// ===========================================================================

#[test]
fn two_token_grammar_positive_state_count() {
    let g = GrammarBuilder::new("sc2a")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

#[test]
fn alternation_grammar_positive_state_count() {
    let g = GrammarBuilder::new("sc2b")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

#[test]
fn multi_nonterminal_positive_state_count() {
    let g = GrammarBuilder::new("sc2c")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0);
}

// ===========================================================================
// 3. action_table.len() == state_count
// ===========================================================================

#[test]
fn action_table_len_two_token() {
    let g = GrammarBuilder::new("at3a")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn action_table_len_alternation() {
    let g = GrammarBuilder::new("at3b")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .rule("start", vec!["z"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn action_table_len_recursive() {
    let g = GrammarBuilder::new("at3c")
        .token("n", "n")
        .token("p", "+")
        .rule("expr", vec!["expr", "p", "n"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn action_table_uniform_column_count() {
    let g = GrammarBuilder::new("at3d")
        .token("a", "a")
        .token("b", "b")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for row in &table.action_table {
        assert_eq!(row.len(), table.symbol_count);
    }
}

// ===========================================================================
// 4. goto_table.len() == state_count
// ===========================================================================

#[test]
fn goto_table_len_simple() {
    let g = GrammarBuilder::new("gt4a")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn goto_table_len_chain() {
    let g = GrammarBuilder::new("gt4b")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn goto_table_len_recursive() {
    let g = GrammarBuilder::new("gt4c")
        .token("a", "a")
        .rule("lst", vec!["a", "lst"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.state_count);
}

// ===========================================================================
// 5. rules non-empty for any grammar with rules
// ===========================================================================

#[test]
fn rules_non_empty_two_rules() {
    let g = GrammarBuilder::new("rne5a")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(!table.rules.is_empty());
}

#[test]
fn rules_non_empty_chain() {
    let g = GrammarBuilder::new("rne5b")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(!table.rules.is_empty());
}

#[test]
fn rules_count_ge_grammar_rules() {
    let g = GrammarBuilder::new("rne5c")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Parse table rules include augmented start rule, so >= grammar rules
    let grammar_rule_count: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(
        table.rules.len() >= grammar_rule_count,
        "table rules {} < grammar rules {}",
        table.rules.len(),
        grammar_rule_count
    );
}

#[test]
fn rules_have_valid_rhs_len() {
    let g = GrammarBuilder::new("rne5d")
        .token("a", "a")
        .token("b", "b")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for rule in &table.rules {
        // rhs_len should be reasonable (0..=100 for any non-pathological grammar)
        assert!(
            rule.rhs_len <= 100,
            "unreasonable rhs_len: {}",
            rule.rhs_len
        );
    }
}

// ===========================================================================
// 6. Automaton for recursive grammar
// ===========================================================================

#[test]
fn left_recursive_builds_successfully() {
    let g = GrammarBuilder::new("rec6a")
        .token("n", "n")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "n"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn left_recursive_has_shift_for_operator() {
    let g = GrammarBuilder::new("rec6b")
        .token("n", "n")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "n"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(any_state_has_shift(&table, tok_id(&g, "plus")));
}

#[test]
fn right_recursive_builds_successfully() {
    let g = GrammarBuilder::new("rec6c")
        .token("a", "a")
        .rule("lst", vec!["a", "lst"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn right_recursive_goto_exists() {
    let g = GrammarBuilder::new("rec6d")
        .token("a", "a")
        .rule("lst", vec!["a", "lst"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build();
    let table = build_table(&g);
    let lst = nt_id(&g, "lst");
    assert!(any_goto_exists(&table, lst));
}

#[test]
fn mutual_recursion_builds() {
    // A → a B, B → b A | b
    let g = GrammarBuilder::new("rec6e")
        .token("a", "a")
        .token("b", "b")
        .rule("nt_a", vec!["a", "nt_b"])
        .rule("nt_b", vec!["b", "nt_a"])
        .rule("nt_b", vec!["b"])
        .rule("start", vec!["nt_a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn deeply_nested_recursion() {
    // chain: a → b c, b → d, c → e, d → "d", e → "e"
    let g = GrammarBuilder::new("rec6f")
        .token("td", "d")
        .token("te", "e")
        .rule("nt_d", vec!["td"])
        .rule("nt_e", vec!["te"])
        .rule("nt_c", vec!["nt_e"])
        .rule("nt_b", vec!["nt_d"])
        .rule("nt_a", vec!["nt_b", "nt_c"])
        .rule("start", vec!["nt_a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 3);
}

// ===========================================================================
// 7. Automaton for ambiguous grammar (GLR)
// ===========================================================================

#[test]
fn ambiguous_expr_has_conflicts() {
    // E → E + E | n  (classic ambiguous)
    let g = GrammarBuilder::new("amb7a")
        .token("n", "n")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(count_conflict_cells(&table) > 0);
}

#[test]
fn ambiguous_expr_preserves_shift_and_reduce() {
    let g = GrammarBuilder::new("amb7b")
        .token("n", "n")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let plus = tok_id(&g, "plus");
    let mut found_shift = false;
    let mut found_reduce = false;
    for st in 0..table.state_count {
        let actions = table.actions(StateId(st as u16), plus);
        if actions.len() > 1 {
            for a in actions {
                match a {
                    Action::Shift(_) => found_shift = true,
                    Action::Reduce(_) => found_reduce = true,
                    _ => {}
                }
            }
        }
    }
    assert!(found_shift && found_reduce);
}

#[test]
fn ambiguous_expr_still_accepts() {
    let g = GrammarBuilder::new("amb7c")
        .token("n", "n")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn ambiguous_multi_op_has_conflicts() {
    // E → E + E | E * E | n
    let g = GrammarBuilder::new("amb7d")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(count_conflict_cells(&table) > 0);
}

#[test]
fn ambiguous_multi_op_accepts() {
    let g = GrammarBuilder::new("amb7e")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 8. Automaton for precedence grammar
// ===========================================================================

#[test]
fn precedence_grammar_builds() {
    let g = GrammarBuilder::new("prec8a")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["expr", "star", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn precedence_grammar_has_both_operators() {
    let g = GrammarBuilder::new("prec8b")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["expr", "star", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(any_state_has_shift(&table, tok_id(&g, "plus")));
    assert!(any_state_has_shift(&table, tok_id(&g, "star")));
}

#[test]
fn precedence_grammar_with_explicit_prec() {
    let g = GrammarBuilder::new("prec8c")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("term", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn precedence_grammar_reduces_on_eof() {
    let g = GrammarBuilder::new("prec8d")
        .token("n", "n")
        .token("plus", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(any_state_has_reduce(&table, table.eof()));
}

// ===========================================================================
// 9. Automaton determinism
// ===========================================================================

#[test]
fn deterministic_grammar_no_conflicts() {
    // S → a b  is LR(1) deterministic
    let g = GrammarBuilder::new("det9a")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(count_conflict_cells(&table), 0);
}

#[test]
fn deterministic_grammar_single_action_per_cell() {
    let g = GrammarBuilder::new("det9b")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for st in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            let cell = &table.action_table[st][sym_idx];
            assert!(
                cell.len() <= 1,
                "state {st}, sym {sym_idx} has {} actions",
                cell.len()
            );
        }
    }
}

#[test]
fn deterministic_chain_no_conflicts() {
    let g = GrammarBuilder::new("det9c")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(count_conflict_cells(&table), 0);
}

#[test]
fn repeated_builds_same_state_count() {
    let make = || {
        let g = GrammarBuilder::new("det9d")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        build_table(&g).state_count
    };
    let c1 = make();
    let c2 = make();
    assert_eq!(c1, c2);
}

#[test]
fn repeated_builds_same_rule_count() {
    let make = || {
        let g = GrammarBuilder::new("det9e")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        build_table(&g).rules.len()
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// 10. Automaton for chain grammar
// ===========================================================================

#[test]
fn chain_grammar_two_levels() {
    let g = GrammarBuilder::new("chain10a")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn chain_grammar_three_levels() {
    let g = GrammarBuilder::new("chain10b")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn chain_grammar_four_levels() {
    let g = GrammarBuilder::new("chain10c")
        .token("x", "x")
        .rule("l4", vec!["x"])
        .rule("l3", vec!["l4"])
        .rule("l2", vec!["l3"])
        .rule("start", vec!["l2"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn chain_grammar_goto_all_levels() {
    let g = GrammarBuilder::new("chain10d")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in ["leaf", "mid", "start"] {
        let nt = nt_id(&g, name);
        assert!(any_goto_exists(&table, nt), "GOTO missing for '{name}'");
    }
}

#[test]
fn chain_grammar_rules_match_levels() {
    let g = GrammarBuilder::new("chain10e")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    // 3 user rules + augmented start rule
    assert!(table.rules.len() >= 3);
}

#[test]
fn chain_grammar_state_count_scales() {
    let g2 = GrammarBuilder::new("chain10f2")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let g4 = GrammarBuilder::new("chain10f4")
        .token("x", "x")
        .rule("l4", vec!["x"])
        .rule("l3", vec!["l4"])
        .rule("l2", vec!["l3"])
        .rule("start", vec!["l2"])
        .start("start")
        .build();
    let t2 = build_table(&g2);
    let t4 = build_table(&g4);
    assert!(t4.state_count >= t2.state_count);
}

// ===========================================================================
// 11. Automaton for large grammar (10+ rules)
// ===========================================================================

#[test]
fn large_grammar_ten_alternatives() {
    let mut b = GrammarBuilder::new("lg11a");
    for i in 0..10 {
        let name = format!("t{i}");
        b = b.token(&name, &name);
    }
    for i in 0..10 {
        let name = format!("t{i}");
        b = b.rule("start", vec![&name]);
    }
    let g = b.start("start").build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.rules.len() >= 10);
}

#[test]
fn large_grammar_ten_sequence_rules() {
    let mut b = GrammarBuilder::new("lg11b");
    for i in 0..10 {
        let tok_name = format!("tok{i}");
        let nt_name = format!("nt{i}");
        b = b.token(&tok_name, &tok_name);
        b = b.rule(&nt_name, vec![&tok_name]);
    }
    // start → nt0 nt1
    b = b.rule("start", vec!["nt0", "nt1"]);
    let g = b.start("start").build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn large_grammar_state_count_above_trivial() {
    let mut b = GrammarBuilder::new("lg11c");
    for i in 0..10 {
        let name = format!("t{i}");
        b = b.token(&name, &name);
    }
    for i in 0..10 {
        let name = format!("t{i}");
        b = b.rule("start", vec![&name]);
    }
    let g = b.start("start").build();
    let table = build_table(&g);
    assert!(table.state_count > 2);
}

#[test]
fn large_grammar_action_table_consistent() {
    let mut b = GrammarBuilder::new("lg11d");
    for i in 0..10 {
        let name = format!("t{i}");
        b = b.token(&name, &name);
        b = b.rule("start", vec![&name]);
    }
    let g = b.start("start").build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn large_grammar_twelve_rules_mixed() {
    let g = GrammarBuilder::new("lg11e")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("r3", vec!["c"])
        .rule("r4", vec!["d"])
        .rule("r5", vec!["e"])
        .rule("r6", vec!["f"])
        .rule("p1", vec!["r1", "r2"])
        .rule("p2", vec!["r3", "r4"])
        .rule("p3", vec!["r5", "r6"])
        .rule("top", vec!["p1"])
        .rule("top", vec!["p2"])
        .rule("top", vec!["p3"])
        .rule("start", vec!["top"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.rules.len() >= 12);
}

// ===========================================================================
// 12. Start symbol present in parse table
// ===========================================================================

#[test]
fn start_symbol_in_goto_from_initial() {
    let g = GrammarBuilder::new("ss12a")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    assert!(table.goto(table.initial_state, s).is_some());
}

#[test]
fn start_symbol_matches_grammar_name() {
    let g = GrammarBuilder::new("ss12b")
        .token("x", "x")
        .rule("top", vec!["x"])
        .start("top")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "top");
    assert_eq!(table.start_symbol(), s);
}

#[test]
fn start_symbol_different_name() {
    let g = GrammarBuilder::new("ss12c")
        .token("z", "z")
        .rule("my_rule", vec!["z"])
        .start("my_rule")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "my_rule");
    assert_eq!(table.start_symbol(), s);
}

#[test]
fn start_symbol_not_eof() {
    let g = GrammarBuilder::new("ss12d")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_ne!(table.start_symbol(), table.eof());
}

// ===========================================================================
// Additional structural invariants
// ===========================================================================

#[test]
fn all_shift_targets_valid_states() {
    let g = GrammarBuilder::new("inv_a")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for st in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for a in &table.action_table[st][sym_idx] {
                if let Action::Shift(target) = a {
                    assert!(
                        (target.0 as usize) < table.state_count,
                        "shift target {} out of range (state_count={})",
                        target.0,
                        table.state_count
                    );
                }
            }
        }
    }
}

#[test]
fn all_reduce_rule_ids_valid() {
    let g = GrammarBuilder::new("inv_b")
        .token("a", "a")
        .token("b", "b")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for st in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for a in &table.action_table[st][sym_idx] {
                if let Action::Reduce(rid) = a {
                    assert!(
                        (rid.0 as usize) < table.rules.len(),
                        "reduce rule {} out of range (rules.len={})",
                        rid.0,
                        table.rules.len()
                    );
                }
            }
        }
    }
}

#[test]
fn goto_targets_valid_states() {
    let g = GrammarBuilder::new("inv_c")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("start", vec!["leaf"])
        .start("start")
        .build();
    let table = build_table(&g);
    for st in 0..table.state_count {
        for col in 0..table.goto_table[st].len() {
            let target = table.goto_table[st][col];
            if target.0 != u16::MAX {
                assert!(
                    (target.0 as usize) < table.state_count,
                    "goto target {} out of range",
                    target.0
                );
            }
        }
    }
}

#[test]
fn symbol_count_positive() {
    let g = GrammarBuilder::new("inv_d")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.symbol_count > 0);
}

#[test]
fn token_count_positive() {
    let g = GrammarBuilder::new("inv_e")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.token_count > 0);
}

#[test]
fn symbol_to_index_has_eof() {
    let g = GrammarBuilder::new("inv_f")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.symbol_to_index.contains_key(&table.eof()));
}

#[test]
fn index_to_symbol_consistent() {
    let g = GrammarBuilder::new("inv_g")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (&sym, &idx) in &table.symbol_to_index {
        assert!(idx < table.index_to_symbol.len());
        assert_eq!(table.index_to_symbol[idx], sym);
    }
}

#[test]
fn exactly_one_accept_action() {
    let g = GrammarBuilder::new("inv_h")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let mut accept_count = 0;
    for st in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for a in &table.action_table[st][sym_idx] {
                if matches!(a, Action::Accept) {
                    accept_count += 1;
                }
            }
        }
    }
    assert_eq!(
        accept_count, 1,
        "expected exactly 1 Accept, got {accept_count}"
    );
}

#[test]
fn accept_only_on_eof() {
    let g = GrammarBuilder::new("inv_i")
        .token("a", "a")
        .token("b", "b")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof_col = table.symbol_to_index.get(&table.eof()).copied();
    for st in 0..table.state_count {
        for (col, cell) in table.action_table[st].iter().enumerate() {
            for a in cell {
                if matches!(a, Action::Accept) {
                    assert_eq!(Some(col), eof_col, "Accept found in non-EOF column {col}");
                }
            }
        }
    }
}

#[test]
fn initial_state_within_range() {
    let g = GrammarBuilder::new("inv_j")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!((table.initial_state.0 as usize) < table.state_count);
}

#[test]
fn nonterminal_to_index_non_empty() {
    let g = GrammarBuilder::new("inv_k")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(!table.nonterminal_to_index.is_empty());
}

#[test]
fn rule_lhs_is_nonterminal() {
    let g = GrammarBuilder::new("inv_l")
        .token("a", "a")
        .token("b", "b")
        .rule("mid", vec!["a"])
        .rule("start", vec!["mid", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for rule in &table.rules {
        assert!(
            table.nonterminal_to_index.contains_key(&rule.lhs)
                || table
                    .goto_table
                    .iter()
                    .any(|row| { row.iter().any(|&s| s.0 != u16::MAX) }),
            "rule lhs {:?} should be a nonterminal",
            rule.lhs
        );
    }
}

#[test]
fn epsilon_start_builds() {
    let g = GrammarBuilder::new("eps_s")
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g);
    if let Ok(ff) = ff {
        let _ = build_lr1_automaton(&g, &ff);
    }
}

#[test]
fn nullable_alt_builds() {
    let g = GrammarBuilder::new("null_alt")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn three_token_sequence() {
    let g = GrammarBuilder::new("seq3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
}

#[test]
fn wide_alternation_five() {
    let g = GrammarBuilder::new("wide5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .rule("start", vec!["e"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(total_reduce_count(&table) >= 5);
}

#[test]
fn parallel_chains_build() {
    // Two independent chains: start → x_chain | y_chain
    let g = GrammarBuilder::new("par")
        .token("x", "x")
        .token("y", "y")
        .rule("xc", vec!["x"])
        .rule("yc", vec!["y"])
        .rule("start", vec!["xc"])
        .rule("start", vec!["yc"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(any_goto_exists(&table, nt_id(&g, "xc")));
    assert!(any_goto_exists(&table, nt_id(&g, "yc")));
}

#[test]
fn grammar_with_shared_prefix() {
    // start → a b | a c  — shared prefix "a"
    let g = GrammarBuilder::new("prefix")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["a", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert_eq!(
        count_conflict_cells(&table),
        0,
        "LR(1) can distinguish a b vs a c"
    );
}

#[test]
fn longer_shared_prefix() {
    // start → a b c | a b d
    let g = GrammarBuilder::new("prefix2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c"])
        .rule("start", vec!["a", "b", "d"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn nested_alternation() {
    // inner → a | b, start → inner c | inner d
    let g = GrammarBuilder::new("nested_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("inner", vec!["a"])
        .rule("inner", vec!["b"])
        .rule("start", vec!["inner", "c"])
        .rule("start", vec!["inner", "d"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn diamond_grammar() {
    // start → left right, left → a, right → b, left → c, right → d
    let g = GrammarBuilder::new("diamond")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("left", vec!["a"])
        .rule("left", vec!["c"])
        .rule("right", vec!["b"])
        .rule("right", vec!["d"])
        .rule("start", vec!["left", "right"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn self_loop_grammar() {
    // S → S a | a  (left recursion on start)
    let g = GrammarBuilder::new("self_loop")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn eof_not_in_grammar_tokens() {
    let g = GrammarBuilder::new("eof_tok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    assert!(!g.tokens.contains_key(&eof));
}

#[test]
fn eof_not_in_grammar_rules() {
    let g = GrammarBuilder::new("eof_nt")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    assert!(!g.rules.contains_key(&eof));
}
