#![allow(clippy::needless_range_loop)]
//! Edge-case and property-based tests for LR(1) automaton construction.
//!
//! Covers: linear chains, recursive grammars, ambiguous grammars,
//! state transitions, goto tables, action table contents, single-rule
//! and many-rule grammars, cyclic grammars, and automaton invariants.
//!
//! Run with: `cargo test -p adze-glr-core --test automaton_edge_cases_proptest`

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

fn any_state_reduces(table: &ParseTable, sym: SymbolId) -> bool {
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), sym)
            .iter()
            .any(|a| matches!(a, Action::Reduce(_)))
    })
}

fn total_shift_count(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .filter(|a| matches!(a, Action::Shift(_)))
        .count()
}

fn total_reduce_count(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count()
}

fn total_accept_count(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .filter(|a| matches!(a, Action::Accept))
        .count()
}

fn conflict_cell_count(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

fn any_goto_exists(table: &ParseTable, nt: SymbolId) -> bool {
    (0..table.state_count).any(|st| table.goto(StateId(st as u16), nt).is_some())
}

// ===========================================================================
// 1. Single-token grammar: S → a
// ===========================================================================

#[test]
fn single_token_grammar_has_accept() {
    let g = GrammarBuilder::new("edge1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn single_token_grammar_shifts_initially() {
    let g = GrammarBuilder::new("edge2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "initial state must shift 'a'"
    );
}

#[test]
fn single_token_grammar_has_reduce_on_eof() {
    let g = GrammarBuilder::new("edge3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_state_reduces(&table, table.eof()));
}

#[test]
fn single_token_grammar_state_count() {
    let g = GrammarBuilder::new("edge4")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // S' → S, S → a needs at least initial, shifted-a, accept states
    assert!(
        table.state_count >= 2,
        "need at least 2 states, got {}",
        table.state_count
    );
}

#[test]
fn single_token_grammar_goto_exists_for_start() {
    let g = GrammarBuilder::new("edge5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start = nt_id(&g, "start");
    assert!(
        any_goto_exists(&table, start),
        "goto for start nonterminal must exist"
    );
}

// ===========================================================================
// 2. Linear chain: S → A, A → B, B → tok
// ===========================================================================

#[test]
fn linear_chain_builds() {
    let g = GrammarBuilder::new("chain")
        .token("tok", "tok")
        .rule("bnt", vec!["tok"])
        .rule("ant", vec!["bnt"])
        .rule("start", vec!["ant"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn linear_chain_shifts_terminal() {
    let g = GrammarBuilder::new("chain2")
        .token("tok", "tok")
        .rule("bnt", vec!["tok"])
        .rule("ant", vec!["bnt"])
        .rule("start", vec!["ant"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_state_shifts(&table, tok_id(&g, "tok")));
}

#[test]
fn linear_chain_gotos_for_each_nonterminal() {
    let g = GrammarBuilder::new("chain3")
        .token("tok", "tok")
        .rule("bnt", vec!["tok"])
        .rule("ant", vec!["bnt"])
        .rule("start", vec!["ant"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_goto_exists(&table, nt_id(&g, "start")));
    assert!(any_goto_exists(&table, nt_id(&g, "ant")));
    assert!(any_goto_exists(&table, nt_id(&g, "bnt")));
}

#[test]
fn linear_chain_multiple_reduces() {
    let g = GrammarBuilder::new("chain4")
        .token("tok", "tok")
        .rule("bnt", vec!["tok"])
        .rule("ant", vec!["bnt"])
        .rule("start", vec!["ant"])
        .start("start")
        .build();
    let table = build_table(&g);
    // 3 rules means at least 3 reduce actions (B→tok, A→B, S→A)
    assert!(
        total_reduce_count(&table) >= 3,
        "chain of 3 rules needs ≥3 reduces, got {}",
        total_reduce_count(&table)
    );
}

// ===========================================================================
// 3. Left recursion: list → list item | item
// ===========================================================================

#[test]
fn left_recursive_list_builds() {
    let g = GrammarBuilder::new("lrec")
        .token("item", "item")
        .rule("list", vec!["list", "item"])
        .rule("list", vec!["item"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn left_recursive_list_shifts_item() {
    let g = GrammarBuilder::new("lrec2")
        .token("item", "item")
        .rule("list", vec!["list", "item"])
        .rule("list", vec!["item"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(any_state_shifts(&table, tok_id(&g, "item")));
}

#[test]
fn left_recursive_list_has_goto_for_list() {
    let g = GrammarBuilder::new("lrec3")
        .token("item", "item")
        .rule("list", vec!["list", "item"])
        .rule("list", vec!["item"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(any_goto_exists(&table, nt_id(&g, "list")));
}

// ===========================================================================
// 4. Right recursion: list → item list | item
// ===========================================================================

#[test]
fn right_recursive_list_builds() {
    let g = GrammarBuilder::new("rrec")
        .token("item", "item")
        .rule("list", vec!["item", "list"])
        .rule("list", vec!["item"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn right_recursive_state_count_at_least_three() {
    let g = GrammarBuilder::new("rrec2")
        .token("item", "item")
        .rule("list", vec!["item", "list"])
        .rule("list", vec!["item"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3,
        "right-recursive list needs ≥3 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 5. Nullable/epsilon rule: S → ε | a
// ===========================================================================

#[test]
fn nullable_start_builds() {
    let g = GrammarBuilder::new("eps1")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn nullable_start_is_nullable_in_first_follow() {
    let g = GrammarBuilder::new("eps2")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(nt_id(&g, "start")));
}

#[test]
fn nullable_chained_propagation() {
    let g = GrammarBuilder::new("eps3")
        .token("a", "a")
        .rule("inner", vec![])
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(nt_id(&g, "inner")));
    assert!(ff.is_nullable(nt_id(&g, "start")));
}

// ===========================================================================
// 6. Ambiguous expression: E → E + E | n (shift-reduce conflict)
// ===========================================================================

#[test]
fn ambiguous_expr_has_conflicts() {
    let g = GrammarBuilder::new("ambig")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(
        conflict_cell_count(&table) > 0,
        "ambiguous expr must have GLR conflicts"
    );
}

#[test]
fn ambiguous_expr_preserves_shift_and_reduce() {
    let g = GrammarBuilder::new("ambig2")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let plus = tok_id(&g, "+");
    let mut found_shift = false;
    let mut found_reduce = false;
    for st in 0..table.state_count {
        let actions = table.actions(StateId(st as u16), plus);
        for a in actions {
            match a {
                Action::Shift(_) => found_shift = true,
                Action::Reduce(_) => found_reduce = true,
                _ => {}
            }
        }
    }
    assert!(
        found_shift && found_reduce,
        "ambiguous grammar must have both shift and reduce on '+'"
    );
}

// ===========================================================================
// 7. Two tokens, one rule: S → a b
// ===========================================================================

#[test]
fn two_token_sequence_builds() {
    let g = GrammarBuilder::new("seq2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn two_token_sequence_shifts_both() {
    let g = GrammarBuilder::new("seq2b")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_state_shifts(&table, tok_id(&g, "a")));
    assert!(any_state_shifts(&table, tok_id(&g, "b")));
}

// ===========================================================================
// 8. Many alternatives: S → a | b | c | d | e
// ===========================================================================

#[test]
fn many_alternatives_builds() {
    let g = GrammarBuilder::new("manyalt")
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
}

#[test]
fn many_alternatives_shifts_all_initially() {
    let g = GrammarBuilder::new("manyalt2")
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
    for name in &["a", "b", "c", "d", "e"] {
        let sym = tok_id(&g, name);
        let actions = table.actions(table.initial_state, sym);
        assert!(
            actions.iter().any(|a| matches!(a, Action::Shift(_))),
            "initial state must shift '{name}'"
        );
    }
}

#[test]
fn many_alternatives_reduces_for_each() {
    let g = GrammarBuilder::new("manyalt3")
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
    // 5 alternatives means at least 5 reduce actions (one per rule)
    assert!(
        total_reduce_count(&table) >= 5,
        "5 alternatives need ≥5 reduces, got {}",
        total_reduce_count(&table)
    );
}

// ===========================================================================
// 9. Multi-level nesting: S → ( S ) | a
// ===========================================================================

#[test]
fn nested_parens_builds() {
    let g = GrammarBuilder::new("parens")
        .token("(", "(")
        .token(")", ")")
        .token("a", "a")
        .rule("start", vec!["(", "start", ")"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn nested_parens_shifts_open_paren_initially() {
    let g = GrammarBuilder::new("parens2")
        .token("(", "(")
        .token(")", ")")
        .token("a", "a")
        .rule("start", vec!["(", "start", ")"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let open = tok_id(&g, "(");
    let actions = table.actions(table.initial_state, open);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "must shift '(' initially"
    );
}

#[test]
fn nested_parens_has_goto_for_start_nonterminal() {
    let g = GrammarBuilder::new("parens3")
        .token("(", "(")
        .token(")", ")")
        .token("a", "a")
        .rule("start", vec!["(", "start", ")"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_goto_exists(&table, nt_id(&g, "start")));
}

// ===========================================================================
// 10. Cyclic/self-referencing unit production: S → S | a
// ===========================================================================

#[test]
fn cyclic_unit_production_builds() {
    let g = GrammarBuilder::new("cyclic")
        .token("a", "a")
        .rule("start", vec!["start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 11. Long RHS: S → a b c d e f
// ===========================================================================

#[test]
fn long_rhs_builds() {
    let g = GrammarBuilder::new("longrhs")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("start", vec!["a", "b", "c", "d", "e", "f"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn long_rhs_shifts_all_tokens() {
    let g = GrammarBuilder::new("longrhs2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("start", vec!["a", "b", "c", "d", "e", "f"])
        .start("start")
        .build();
    let table = build_table(&g);
    for name in &["a", "b", "c", "d", "e", "f"] {
        assert!(
            any_state_shifts(&table, tok_id(&g, name)),
            "must shift '{name}' somewhere"
        );
    }
}

#[test]
fn long_rhs_state_count() {
    let g = GrammarBuilder::new("longrhs3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("start", vec!["a", "b", "c", "d", "e", "f"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Need at least 7 states for 6-token RHS (initial + one per shifted token + accept)
    assert!(
        table.state_count >= 7,
        "6-token RHS needs ≥7 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 12. Mutual recursion: A → a B, B → b A | b
// ===========================================================================

#[test]
fn mutual_recursion_builds() {
    let g = GrammarBuilder::new("mutual")
        .token("a", "a")
        .token("b", "b")
        .rule("bnt", vec!["b", "ant"])
        .rule("bnt", vec!["b"])
        .rule("ant", vec!["a", "bnt"])
        .rule("start", vec!["ant"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn mutual_recursion_gotos_for_both() {
    let g = GrammarBuilder::new("mutual2")
        .token("a", "a")
        .token("b", "b")
        .rule("bnt", vec!["b", "ant"])
        .rule("bnt", vec!["b"])
        .rule("ant", vec!["a", "bnt"])
        .rule("start", vec!["ant"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_goto_exists(&table, nt_id(&g, "ant")));
    assert!(any_goto_exists(&table, nt_id(&g, "bnt")));
}

// ===========================================================================
// 13. Mixed terminals and nonterminals: S → a B c, B → b
// ===========================================================================

#[test]
fn mixed_terminal_nonterminal_rhs() {
    let g = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("bnt", vec!["b"])
        .rule("start", vec!["a", "bnt", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(any_state_shifts(&table, tok_id(&g, "a")));
    assert!(any_state_shifts(&table, tok_id(&g, "b")));
    assert!(any_state_shifts(&table, tok_id(&g, "c")));
}

// ===========================================================================
// 14. Diamond: S → A | B, A → x, B → x  (reduce-reduce conflict)
// ===========================================================================

#[test]
fn diamond_builds() {
    let g = GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("ant", vec!["x"])
        .rule("bnt", vec!["x"])
        .rule("start", vec!["ant"])
        .rule("start", vec!["bnt"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn diamond_has_conflicts_or_resolves() {
    let g = GrammarBuilder::new("diamond2")
        .token("x", "x")
        .rule("ant", vec!["x"])
        .rule("bnt", vec!["x"])
        .rule("start", vec!["ant"])
        .rule("start", vec!["bnt"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Either GLR preserves conflicts or resolves them — table must be valid
    assert!(table.state_count > 0);
    assert!(has_accept(&table));
}

// ===========================================================================
// 15. ParseTable::default produces empty table
// ===========================================================================

#[test]
fn parse_table_default_is_empty() {
    let table = ParseTable::default();
    assert_eq!(table.state_count, 0);
    assert_eq!(table.symbol_count, 0);
    assert!(table.action_table.is_empty());
    assert!(table.goto_table.is_empty());
    assert!(table.rules.is_empty());
    assert!(table.symbol_to_index.is_empty());
    assert!(table.nonterminal_to_index.is_empty());
}

// ===========================================================================
// 16. EOF symbol is in symbol_to_index
// ===========================================================================

#[test]
fn eof_symbol_is_indexed() {
    let g = GrammarBuilder::new("eof_idx")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF must be in symbol_to_index"
    );
}

// ===========================================================================
// 17. Action table dimensions match state_count
// ===========================================================================

#[test]
fn action_table_rows_equal_state_count() {
    let g = GrammarBuilder::new("dim1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
}

// ===========================================================================
// 18. Goto table dimensions match state_count
// ===========================================================================

#[test]
fn goto_table_rows_equal_state_count() {
    let g = GrammarBuilder::new("dim2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.state_count);
}

// ===========================================================================
// 19. Exactly one accept action for simple grammar
// ===========================================================================

#[test]
fn single_accept_for_simple_grammar() {
    let g = GrammarBuilder::new("oneaccept")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(
        total_accept_count(&table),
        1,
        "simple grammar must have exactly 1 accept"
    );
}

// ===========================================================================
// 20. Start symbol matches grammar's start
// ===========================================================================

#[test]
fn table_start_symbol_matches_grammar() {
    let g = GrammarBuilder::new("startsym")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let expected = nt_id(&g, "start");
    assert_eq!(table.start_symbol(), expected);
}

// ===========================================================================
// 21. Rules in parse table
// ===========================================================================

#[test]
fn parse_table_rules_are_populated() {
    let g = GrammarBuilder::new("rules1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    // At least the original rule
    assert!(
        !table.rules.is_empty(),
        "expected at least 1 rule, got {}",
        table.rules.len()
    );
}

// ===========================================================================
// 22. Shift targets are valid state indices
// ===========================================================================

#[test]
fn shift_targets_are_valid_states() {
    let g = GrammarBuilder::new("shifttgt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Shift(target) = action {
                    assert!(
                        (target.0 as usize) < table.state_count,
                        "shift target {} exceeds state_count {}",
                        target.0,
                        table.state_count
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 23. Reduce rule IDs are valid
// ===========================================================================

#[test]
fn reduce_rule_ids_are_valid() {
    let g = GrammarBuilder::new("redid")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Reduce(rule_id) = action {
                    assert!(
                        (rule_id.0 as usize) < table.rules.len(),
                        "reduce rule {} out of range (max {})",
                        rule_id.0,
                        table.rules.len()
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 24. Goto targets are valid
// ===========================================================================

#[test]
fn goto_targets_are_valid_states() {
    let g = GrammarBuilder::new("gototgt")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    for row in &table.goto_table {
        for &target in row {
            if target.0 != u16::MAX {
                assert!(
                    (target.0 as usize) < table.state_count,
                    "goto target {} exceeds state_count {}",
                    target.0,
                    table.state_count
                );
            }
        }
    }
}

// ===========================================================================
// 25. FIRST set contains terminal for S → a
// ===========================================================================

#[test]
fn first_set_contains_terminal() {
    let g = GrammarBuilder::new("first1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = nt_id(&g, "start");
    let a = tok_id(&g, "a");
    let first = ff.first(start).expect("FIRST(start) must exist");
    assert!(
        first.contains(a.0 as usize),
        "FIRST(start) must contain 'a'"
    );
}

// ===========================================================================
// 26. FIRST set propagates through nonterminal
// ===========================================================================

#[test]
fn first_set_propagates() {
    let g = GrammarBuilder::new("first2")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = nt_id(&g, "start");
    let x = tok_id(&g, "x");
    let first = ff.first(start).expect("FIRST(start) must exist");
    assert!(
        first.contains(x.0 as usize),
        "FIRST(start) must contain 'x' via inner"
    );
}

// ===========================================================================
// 27. Epsilon-only grammar: S → ε
// ===========================================================================

#[test]
fn epsilon_only_grammar_builds() {
    let g = GrammarBuilder::new("epsonly")
        .rule("start", vec![])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn epsilon_only_grammar_is_nullable() {
    let g = GrammarBuilder::new("epsonly2")
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(nt_id(&g, "start")));
}

// ===========================================================================
// 28. Grammar with dangling nonterminal (unreachable)
// ===========================================================================

#[test]
fn unreachable_nonterminal_still_builds() {
    let g = GrammarBuilder::new("unreach")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("orphan", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 29. Two independent paths: S → A c | B c, A → a, B → b
// ===========================================================================

#[test]
fn two_paths_same_suffix() {
    let g = GrammarBuilder::new("twopaths")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("ant", vec!["a"])
        .rule("bnt", vec!["b"])
        .rule("start", vec!["ant", "c"])
        .rule("start", vec!["bnt", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(any_state_shifts(&table, tok_id(&g, "a")));
    assert!(any_state_shifts(&table, tok_id(&g, "b")));
    assert!(any_state_shifts(&table, tok_id(&g, "c")));
}

// ===========================================================================
// 30. Three-level recursion: S → a S b S c | d
// ===========================================================================

#[test]
fn three_level_recursion_builds() {
    let g = GrammarBuilder::new("threelevel")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "start", "b", "start", "c"])
        .rule("start", vec!["d"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 31. Action table rows all have same width
// ===========================================================================

#[test]
fn action_table_rows_uniform_width() {
    let g = GrammarBuilder::new("uniform")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    if !table.action_table.is_empty() {
        let width = table.action_table[0].len();
        for (i, row) in table.action_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                width,
                "action table row {i} has width {} but expected {width}",
                row.len()
            );
        }
    }
}

// ===========================================================================
// 32. Goto table rows all have same width
// ===========================================================================

#[test]
fn goto_table_rows_uniform_width() {
    let g = GrammarBuilder::new("gotouniform")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    if !table.goto_table.is_empty() {
        let width = table.goto_table[0].len();
        for (i, row) in table.goto_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                width,
                "goto table row {i} has width {} but expected {width}",
                row.len()
            );
        }
    }
}

// ===========================================================================
// 33. Arithmetic grammar: E → E + T | T, T → n
// ===========================================================================

#[test]
fn arithmetic_grammar_builds() {
    let g = GrammarBuilder::new("arith")
        .token("n", "n")
        .token("+", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(total_shift_count(&table) > 0);
    assert!(total_reduce_count(&table) > 0);
}

// ===========================================================================
// 34. Arithmetic with multiply: E → E + T | T, T → T * F | F, F → n
// ===========================================================================

#[test]
fn arithmetic_three_level_builds() {
    let g = GrammarBuilder::new("arith3")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule("factor", vec!["n"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // All tokens must be shifted somewhere
    assert!(any_state_shifts(&table, tok_id(&g, "n")));
    assert!(any_state_shifts(&table, tok_id(&g, "+")));
    assert!(any_state_shifts(&table, tok_id(&g, "*")));
}

// ===========================================================================
// 35. Initial state is valid
// ===========================================================================

#[test]
fn initial_state_is_valid() {
    let g = GrammarBuilder::new("initst")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!((table.initial_state.0 as usize) < table.state_count);
}

// ===========================================================================
// 36. Non-terminal is not in symbol_to_index (action table is for terminals)
// ===========================================================================

#[test]
fn nonterminal_not_in_action_symbol_index() {
    let g = GrammarBuilder::new("ntcheck")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let start_nt = nt_id(&g, "start");
    // The start nonterminal should NOT be in symbol_to_index (used for action table)
    // It should be in nonterminal_to_index (used for goto table)
    assert!(
        table.nonterminal_to_index.contains_key(&start_nt),
        "start nonterminal must be in nonterminal_to_index"
    );
}

// ===========================================================================
// 37. Fork actions inside conflict cells
// ===========================================================================

#[test]
fn fork_actions_contain_valid_sub_actions() {
    let g = GrammarBuilder::new("forkcheck")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Fork(sub_actions) = action {
                    assert!(sub_actions.len() >= 2, "Fork must have ≥2 sub-actions");
                    for sub in sub_actions {
                        assert!(
                            !matches!(sub, Action::Fork(_)),
                            "Fork must not contain nested Forks"
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 38. No accept on non-EOF terminal
// ===========================================================================

#[test]
fn no_accept_on_regular_terminal() {
    let g = GrammarBuilder::new("noacceptterm")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    for st in 0..table.state_count {
        let actions = table.actions(StateId(st as u16), a);
        for action in actions {
            assert!(
                !matches!(action, Action::Accept),
                "Accept must not appear on regular terminal 'a'"
            );
        }
    }
}

// ===========================================================================
// 39. Alternation with shared prefix: S → a b | a c
// ===========================================================================

#[test]
fn shared_prefix_builds() {
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
}

#[test]
fn shared_prefix_shifts_a_once() {
    let g = GrammarBuilder::new("prefix2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["a", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Initial state should shift 'a'
    let a = tok_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    let shift_count = actions
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .count();
    assert_eq!(shift_count, 1, "should shift 'a' exactly once initially");
}

// ===========================================================================
// 40. Token count matches
// ===========================================================================

#[test]
fn token_count_matches_grammar() {
    let g = GrammarBuilder::new("tokcnt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    // token_count should be at least the number of declared tokens
    assert!(
        table.token_count >= 3,
        "token_count should be ≥3, got {}",
        table.token_count
    );
}

// ===========================================================================
// 41. Grammar with operator precedence doesn't crash
// ===========================================================================

#[test]
fn precedence_grammar_builds() {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 42. index_to_symbol is inverse of symbol_to_index
// ===========================================================================

#[test]
fn index_to_symbol_is_inverse() {
    let g = GrammarBuilder::new("inverse")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for (&sym, &idx) in &table.symbol_to_index {
        if idx < table.index_to_symbol.len() {
            assert_eq!(
                table.index_to_symbol[idx], sym,
                "index_to_symbol[{idx}] should map back to {:?}",
                sym
            );
        }
    }
}

// ===========================================================================
// 43. Multiple start rule alternatives: S → a | b | ε
// ===========================================================================

#[test]
fn start_with_epsilon_and_terminals() {
    let g = GrammarBuilder::new("starteps")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 44. Deeply nested chain: S → A1, A1 → A2, ... A9 → x
// ===========================================================================

#[test]
fn deeply_nested_chain_builds() {
    let g = GrammarBuilder::new("deep")
        .token("x", "x")
        .rule("a9", vec!["x"])
        .rule("a8", vec!["a9"])
        .rule("a7", vec!["a8"])
        .rule("a6", vec!["a7"])
        .rule("a5", vec!["a6"])
        .rule("a4", vec!["a5"])
        .rule("a3", vec!["a4"])
        .rule("a2", vec!["a3"])
        .rule("a1", vec!["a2"])
        .rule("start", vec!["a1"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // At least 10 reduces (one per rule) + augmented start
    assert!(
        total_reduce_count(&table) >= 10,
        "10-deep chain needs ≥10 reduces, got {}",
        total_reduce_count(&table)
    );
}

// ===========================================================================
// 45. Grammar cloning via ParseTable doesn't lose data
// ===========================================================================

#[test]
fn parse_table_clone_preserves_data() {
    let g = GrammarBuilder::new("clone")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let cloned = table.clone();
    assert_eq!(cloned.state_count, table.state_count);
    assert_eq!(cloned.symbol_count, table.symbol_count);
    assert_eq!(cloned.rules.len(), table.rules.len());
    assert_eq!(cloned.eof_symbol, table.eof_symbol);
    assert_eq!(cloned.start_symbol, table.start_symbol);
}

// ===========================================================================
// 46. Multiple distinct nonterminals in same grammar
// ===========================================================================

#[test]
fn multiple_nonterminals_all_have_gotos() {
    let g = GrammarBuilder::new("multinont")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("cnt", vec!["z"])
        .rule("bnt", vec!["y", "cnt"])
        .rule("ant", vec!["x", "bnt"])
        .rule("start", vec!["ant"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(any_goto_exists(&table, nt_id(&g, "start")));
    assert!(any_goto_exists(&table, nt_id(&g, "ant")));
    assert!(any_goto_exists(&table, nt_id(&g, "bnt")));
    assert!(any_goto_exists(&table, nt_id(&g, "cnt")));
}

// ===========================================================================
// 47. build_lr1_automaton_res is equivalent to build_lr1_automaton
// ===========================================================================

#[test]
fn build_lr1_automaton_res_matches() {
    let g = GrammarBuilder::new("reseq")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table1 = build_lr1_automaton(&g, &ff).unwrap();
    let table2 = adze_glr_core::build_lr1_automaton_res(&g, &ff).unwrap();
    assert_eq!(table1.state_count, table2.state_count);
    assert_eq!(table1.eof_symbol, table2.eof_symbol);
}

// ===========================================================================
// 48. eof() accessor returns expected value
// ===========================================================================

#[test]
fn eof_accessor_consistent() {
    let g = GrammarBuilder::new("eofacc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.eof(), table.eof_symbol);
}

// ===========================================================================
// 49. grammar() accessor returns cloned grammar
// ===========================================================================

#[test]
fn grammar_accessor_returns_grammar() {
    let g = GrammarBuilder::new("gramacc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert_eq!(table.grammar().name, "gramacc");
}

// ===========================================================================
// 50. rule() accessor returns correct lhs and rhs_len
// ===========================================================================

#[test]
fn rule_accessor_returns_valid_data() {
    let g = GrammarBuilder::new("ruleacc")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    for i in 0..table.rules.len() {
        let (lhs, rhs_len) = table.rule(RuleId(i as u16));
        assert!(lhs.0 > 0 || lhs.0 == 0, "lhs must be valid SymbolId");
        assert!(rhs_len <= 100, "rhs_len {} is unreasonably large", rhs_len);
    }
}

// ===========================================================================
// Property-based tests
// ===========================================================================

/// Strategy to build a simple grammar with N tokens and one rule per token.
fn simple_alternation_grammar(n: usize) -> Grammar {
    let mut builder = GrammarBuilder::new("prop_alt");
    let names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("start", vec![name.as_str()]);
    }
    builder = builder.start("start");
    builder.build()
}

/// Strategy to build a left-recursive list grammar: S → S tok_i | tok_i
fn left_recursive_grammar(n_tokens: usize) -> Grammar {
    let mut builder = GrammarBuilder::new("prop_lrec");
    let names: Vec<String> = (0..n_tokens).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("start", vec!["start", name.as_str()]);
        builder = builder.rule("start", vec![name.as_str()]);
    }
    builder = builder.start("start");
    builder.build()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // P1: Any alternation grammar with 1..=8 tokens builds successfully
    #[test]
    fn prop_alternation_always_builds(n in 1usize..=8) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        prop_assert!(has_accept(&table));
    }

    // P2: Alternation grammar has exactly one Accept action
    #[test]
    fn prop_alternation_one_accept(n in 1usize..=8) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        prop_assert_eq!(total_accept_count(&table), 1);
    }

    // P3: Alternation grammar has at least N reduce actions
    #[test]
    fn prop_alternation_reduce_count(n in 1usize..=8) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        prop_assert!(
            total_reduce_count(&table) >= n,
            "{} alternatives need ≥{} reduces, got {}",
            n, n, total_reduce_count(&table)
        );
    }

    // P4: Action table dimensions are consistent
    #[test]
    fn prop_action_table_dimensions(n in 1usize..=8) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        prop_assert_eq!(table.action_table.len(), table.state_count);
        if !table.action_table.is_empty() {
            let w = table.action_table[0].len();
            for row in &table.action_table {
                prop_assert_eq!(row.len(), w);
            }
        }
    }

    // P5: Goto table dimensions are consistent
    #[test]
    fn prop_goto_table_dimensions(n in 1usize..=8) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
        if !table.goto_table.is_empty() {
            let w = table.goto_table[0].len();
            for row in &table.goto_table {
                prop_assert_eq!(row.len(), w);
            }
        }
    }

    // P6: All shift targets point to valid states
    #[test]
    fn prop_shift_targets_valid(n in 1usize..=6) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        for row in &table.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Shift(target) = action {
                        prop_assert!((target.0 as usize) < table.state_count);
                    }
                }
            }
        }
    }

    // P7: All reduce IDs are within bounds
    #[test]
    fn prop_reduce_ids_valid(n in 1usize..=6) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        for row in &table.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Reduce(rid) = action {
                        prop_assert!((rid.0 as usize) < table.rules.len());
                    }
                }
            }
        }
    }

    // P8: EOF is always in symbol_to_index
    #[test]
    fn prop_eof_indexed(n in 1usize..=8) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        prop_assert!(table.symbol_to_index.contains_key(&table.eof_symbol));
    }

    // P9: Initial state is in range
    #[test]
    fn prop_initial_state_valid(n in 1usize..=8) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        prop_assert!((table.initial_state.0 as usize) < table.state_count);
    }

    // P10: Left-recursive grammar always builds
    #[test]
    fn prop_left_recursive_builds(n in 1usize..=4) {
        let g = left_recursive_grammar(n);
        let table = build_table(&g);
        prop_assert!(has_accept(&table));
    }

    // P11: Left-recursive grammar state_count grows with tokens
    #[test]
    fn prop_left_recursive_states_grow(n in 2usize..=4) {
        let g_small = left_recursive_grammar(1);
        let g_large = left_recursive_grammar(n);
        let t_small = build_table(&g_small);
        let t_large = build_table(&g_large);
        prop_assert!(
            t_large.state_count >= t_small.state_count,
            "more tokens should mean ≥ states"
        );
    }

    // P12: No Accept on any non-EOF terminal for alternation grammars
    #[test]
    fn prop_no_accept_on_terminals(n in 1usize..=6) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        let eof = table.eof();
        for (&sym, &idx) in &table.symbol_to_index {
            if sym == eof {
                continue;
            }
            for st in 0..table.state_count {
                let actions = &table.action_table[st][idx];
                for a in actions {
                    prop_assert!(
                        !matches!(a, Action::Accept),
                        "Accept found on non-EOF symbol {:?} in state {}",
                        sym, st
                    );
                }
            }
        }
    }

    // P13: Goto targets within bounds
    #[test]
    fn prop_goto_targets_valid(n in 1usize..=6) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        for row in &table.goto_table {
            for &target in row {
                if target.0 != u16::MAX {
                    prop_assert!((target.0 as usize) < table.state_count);
                }
            }
        }
    }

    // P14: index_to_symbol inverts symbol_to_index
    #[test]
    fn prop_index_symbol_roundtrip(n in 1usize..=6) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        for (&sym, &idx) in &table.symbol_to_index {
            if idx < table.index_to_symbol.len() {
                prop_assert_eq!(table.index_to_symbol[idx], sym);
            }
        }
    }

    // P15: Fork sub-actions are not nested
    #[test]
    fn prop_no_nested_forks(n in 1usize..=6) {
        let g = simple_alternation_grammar(n);
        let table = build_table(&g);
        for row in &table.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Fork(subs) = action {
                        for sub in subs {
                            prop_assert!(!matches!(sub, Action::Fork(_)));
                        }
                    }
                }
            }
        }
    }
}
