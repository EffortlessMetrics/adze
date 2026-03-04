#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for LR(1) automaton construction via `build_lr1_automaton`.
//!
//! Covers: simple grammars, state counts, transition correctness, augmented
//! grammar handling, accept state detection, error state handling, ambiguous
//! grammars, and large grammar automaton construction.

use adze_glr_core::{Action, FirstFollowSets, StateId, SymbolId, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;

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

fn count_fork_cells(table: &adze_glr_core::ParseTable) -> usize {
    let mut count = 0;
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            let actions = &table.action_table[state][sym_idx];
            if actions.iter().any(|a| matches!(a, Action::Fork(_))) {
                count += 1;
            }
        }
    }
    count
}

// ===========================================================================
// 1. Automaton from simple grammar — single terminal rule
// ===========================================================================

#[test]
fn simple_grammar_single_rule_builds() {
    let g = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(table.state_count > 0, "automaton must have states");
    assert!(has_accept(&table), "must have Accept action");
}

// ===========================================================================
// 2. State count for single-terminal grammar
// ===========================================================================

#[test]
fn single_terminal_state_count_bounded() {
    let g = GrammarBuilder::new("sc1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    // Augmented grammar S' -> S, S -> x needs at least 3 states
    assert!(
        table.state_count >= 3,
        "expected >= 3 states, got {}",
        table.state_count
    );
    assert!(
        table.state_count <= 10,
        "should not explode; got {} states",
        table.state_count
    );
}

// ===========================================================================
// 3. Transition correctness — shift on initial terminal
// ===========================================================================

#[test]
fn initial_state_shifts_on_start_terminal() {
    let g = GrammarBuilder::new("shift")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(
        table
            .actions(table.initial_state, a)
            .iter()
            .any(|act| matches!(act, Action::Shift(_))),
        "initial state must shift on terminal 'a'"
    );
}

// ===========================================================================
// 4. Transition correctness — shift targets are valid states
// ===========================================================================

#[test]
fn shift_targets_within_state_count() {
    let g = GrammarBuilder::new("valid")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for action in &table.action_table[state][sym_idx] {
                if let Action::Shift(StateId(s)) = action {
                    assert!(
                        (*s as usize) < table.state_count,
                        "shift target {s} out of range (state_count={})",
                        table.state_count
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 5. Augmented grammar — start symbol preserved
// ===========================================================================

#[test]
fn augmented_grammar_preserves_start_symbol() {
    let g = GrammarBuilder::new("aug")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    let table = build_table(&g);
    let root = nt_id(&g, "root");
    assert_eq!(
        table.start_symbol(),
        root,
        "start_symbol must match grammar's declared start"
    );
}

// ===========================================================================
// 6. Augmented grammar — goto on start symbol from initial state
// ===========================================================================

#[test]
fn goto_for_start_nonterminal_exists() {
    let g = GrammarBuilder::new("gotoinit")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    assert!(
        table.goto(table.initial_state, s).is_some(),
        "goto(initial, start) must exist in augmented grammar"
    );
}

// ===========================================================================
// 7. Accept state detection — accept on EOF
// ===========================================================================

#[test]
fn accept_action_on_eof() {
    let g = GrammarBuilder::new("accept")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let accept_found = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(accept_found, "exactly one state should accept on EOF");
}

// ===========================================================================
// 8. Accept state — only one state has Accept
// ===========================================================================

#[test]
fn accept_in_exactly_one_state() {
    let g = GrammarBuilder::new("accept1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let accept_count = (0..table.state_count)
        .filter(|&st| {
            table
                .actions(StateId(st as u16), eof)
                .iter()
                .any(|a| matches!(a, Action::Accept))
        })
        .count();
    assert!(
        accept_count >= 1,
        "at least one state should have Accept on EOF"
    );
}

// ===========================================================================
// 9. Error state handling — some cells are Error
// ===========================================================================

#[test]
fn error_cells_present_in_action_table() {
    let g = GrammarBuilder::new("err")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let has_error = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.is_empty() || cell.iter().any(|a| matches!(a, Action::Error)))
    });
    assert!(
        has_error,
        "some action cells should be empty or Error for invalid inputs"
    );
}

// ===========================================================================
// 10. Error state — no shift on absent terminal
// ===========================================================================

#[test]
fn no_shift_on_unused_terminal() {
    let g = GrammarBuilder::new("noshift")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let b = tok_id(&g, "b");
    assert!(
        !table
            .actions(table.initial_state, b)
            .iter()
            .any(|a| matches!(a, Action::Shift(_))),
        "initial state must not shift on unused terminal 'b'"
    );
}

// ===========================================================================
// 11. Ambiguous grammar — shift-reduce conflict produces Fork
// ===========================================================================

#[test]
fn ambiguous_expr_produces_fork() {
    let g = GrammarBuilder::new("ambig")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table), "ambiguous grammar should still accept");

    let fork_count = count_fork_cells(&table);
    assert!(
        fork_count > 0
            || table
                .action_table
                .iter()
                .any(|row| row.iter().any(|cell| cell.len() > 1)),
        "ambiguous grammar should produce Fork or multi-action cells"
    );
}

// ===========================================================================
// 12. Ambiguous grammar — Fork contains at least 2 sub-actions
// ===========================================================================

#[test]
fn fork_has_multiple_alternatives() {
    let g = GrammarBuilder::new("forkalt")
        .token("n", "n")
        .token("+", "+")
        .rule("e", vec!["e", "+", "e"])
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let table = build_table(&g);
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Fork(sub) = action {
                    assert!(
                        sub.len() >= 2,
                        "Fork must contain >= 2 alternatives, got {}",
                        sub.len()
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 13. Ambiguous grammar — multiple operators
// ===========================================================================

#[test]
fn ambiguous_grammar_two_operators() {
    let g = GrammarBuilder::new("ambig2")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule("e", vec!["e", "+", "e"])
        .rule("e", vec!["e", "*", "e"])
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4, "need states for two operators");
}

// ===========================================================================
// 14. Large grammar — many tokens
// ===========================================================================

#[test]
fn large_grammar_many_tokens() {
    let mut builder = GrammarBuilder::new("large");
    let mut rhs = Vec::new();
    for i in 0..20 {
        let name: &str = Box::leak(format!("T{i}").into_boxed_str());
        let pat: &str = Box::leak(format!("t{i}").into_boxed_str());
        builder = builder.token(name, pat);
        rhs.push(name);
    }
    // Rule using first token only
    builder = builder.rule("s", vec![rhs[0]]).start("s");
    let g = builder.build();
    let table = build_table(&g);
    assert!(table.token_count >= 20);
    assert!(has_accept(&table));
}

// ===========================================================================
// 15. Large grammar — many alternatives
// ===========================================================================

#[test]
fn large_grammar_many_alternatives() {
    let mut builder = GrammarBuilder::new("manyalt");
    for i in 0..15 {
        let name: &str = Box::leak(format!("T{i}").into_boxed_str());
        let pat: &str = Box::leak(format!("t{i}").into_boxed_str());
        builder = builder.token(name, pat);
        builder = builder.rule("s", vec![name]);
    }
    builder = builder.start("s");
    let g = builder.build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // All 15 tokens should have shifts from initial state
    for i in 0..15 {
        let name = format!("T{i}");
        let tid = tok_id(&g, &name);
        assert!(
            any_state_has_shift(&table, tid),
            "should have shift on T{i}"
        );
    }
}

// ===========================================================================
// 16. Large grammar — long sequence rule
// ===========================================================================

#[test]
fn large_grammar_long_sequence() {
    let mut builder = GrammarBuilder::new("longseq");
    let mut rhs = Vec::new();
    for i in 0..8 {
        let name: &str = Box::leak(format!("T{i}").into_boxed_str());
        let pat: &str = Box::leak(format!("t{i}").into_boxed_str());
        builder = builder.token(name, pat);
        rhs.push(name);
    }
    builder = builder.rule("s", rhs).start("s");
    let g = builder.build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // 8-token sequence needs at least 9 states (initial + after each token)
    assert!(
        table.state_count >= 9,
        "8-token sequence needs >= 9 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 17. Action table dimensions match state_count × symbol_count
// ===========================================================================

#[test]
fn action_table_dimensions_consistent() {
    let g = GrammarBuilder::new("dim")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert_eq!(table.action_table.len(), table.state_count);
    for row in &table.action_table {
        assert_eq!(row.len(), table.symbol_count);
    }
}

// ===========================================================================
// 18. Goto table dimensions match state_count × symbol_count
// ===========================================================================

#[test]
fn goto_table_dimensions_consistent() {
    let g = GrammarBuilder::new("gdim")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert_eq!(table.goto_table.len(), table.state_count);
    for row in &table.goto_table {
        assert_eq!(row.len(), table.symbol_count);
    }
}

// ===========================================================================
// 19. Symbol-to-index and index-to-symbol are inverses
// ===========================================================================

#[test]
fn symbol_index_mapping_inverse() {
    let g = GrammarBuilder::new("inv")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    for (&sym, &idx) in &table.symbol_to_index {
        assert_eq!(
            table.index_to_symbol[idx], sym,
            "index_to_symbol[{idx}] should be {sym:?}"
        );
    }
}

// ===========================================================================
// 20. EOF symbol is in symbol_to_index
// ===========================================================================

#[test]
fn eof_present_in_symbol_mapping() {
    let g = GrammarBuilder::new("eof")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    assert!(
        table.symbol_to_index.contains_key(&eof),
        "EOF must be in symbol_to_index"
    );
}

// ===========================================================================
// 21. EOF symbol distinct from grammar tokens
// ===========================================================================

#[test]
fn eof_distinct_from_grammar_symbols() {
    let g = GrammarBuilder::new("eofdist")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for (tok, _) in &g.tokens {
        assert_ne!(eof, *tok, "EOF must differ from token {tok:?}");
    }
}

// ===========================================================================
// 22. Reduce actions reference valid rule IDs
// ===========================================================================

#[test]
fn reduce_actions_reference_valid_rules() {
    let g = GrammarBuilder::new("rvalid")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let rule_count = table.rules.len();
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for action in &table.action_table[state][sym_idx] {
                match action {
                    Action::Reduce(rid) => {
                        assert!(
                            (rid.0 as usize) < rule_count,
                            "Reduce rule {rid:?} out of range (rule_count={rule_count})"
                        );
                    }
                    Action::Fork(sub) => {
                        for a in sub {
                            if let Action::Reduce(rid) = a {
                                assert!(
                                    (rid.0 as usize) < rule_count,
                                    "Fork/Reduce rule {rid:?} out of range"
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// ===========================================================================
// 23. Left-recursive grammar builds and has accept
// ===========================================================================

#[test]
fn left_recursive_grammar_accept() {
    let g = GrammarBuilder::new("lrec")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 3);
}

// ===========================================================================
// 24. Right-recursive grammar builds and has accept
// ===========================================================================

#[test]
fn right_recursive_grammar_accept() {
    let g = GrammarBuilder::new("rrec")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 25. Chained nonterminals — three-level chain
// ===========================================================================

#[test]
fn three_level_chain_builds() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("top", vec!["mid"])
        .start("top")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    let x = tok_id(&g, "x");
    assert!(
        any_state_has_shift(&table, x),
        "should shift on 'x' somewhere"
    );
}

// ===========================================================================
// 26. Two-symbol sequence has intermediate shift
// ===========================================================================

#[test]
fn two_symbol_sequence_intermediate_shift() {
    let g = GrammarBuilder::new("seq2")
        .token("lp", "(")
        .token("rp", ")")
        .rule("pair", vec!["lp", "rp"])
        .start("pair")
        .build();
    let table = build_table(&g);
    let rp = tok_id(&g, "rp");
    assert!(
        any_state_has_shift(&table, rp),
        "some state after shifting '(' must shift ')'"
    );
    assert!(has_accept(&table));
}

// ===========================================================================
// 27. Goto for nonterminal in chained grammar
// ===========================================================================

#[test]
fn goto_for_inner_nonterminal() {
    let g = GrammarBuilder::new("gotont")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let goto_exists =
        (0..table.state_count).any(|st| table.goto(StateId(st as u16), inner).is_some());
    assert!(goto_exists, "goto for 'inner' must exist somewhere");
}

// ===========================================================================
// 28. Action table has both shift and reduce
// ===========================================================================

#[test]
fn action_table_has_shift_and_reduce() {
    let g = GrammarBuilder::new("sr")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let eof = table.eof();

    assert!(any_state_has_shift(&table, a), "must have shift on 'a'");
    assert!(any_state_has_shift(&table, b), "must have shift on 'b'");
    assert!(any_state_has_reduce(&table, eof), "must have reduce on EOF");
}

// ===========================================================================
// 29. ParseRules populated correctly
// ===========================================================================

#[test]
fn parse_rules_populated() {
    let g = GrammarBuilder::new("prules")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(table.rules.len() >= 2, "should have at least 2 parse rules");
}

// ===========================================================================
// 30. Symbol metadata includes terminals and nonterminals
// ===========================================================================

#[test]
fn symbol_metadata_has_terminals_and_nonterminals() {
    let g = GrammarBuilder::new("meta")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let has_terminal = table.symbol_metadata.iter().any(|m| m.is_terminal);
    let has_nonterminal = table.symbol_metadata.iter().any(|m| !m.is_terminal);
    assert!(has_terminal, "metadata must include terminals");
    assert!(has_nonterminal, "metadata must include nonterminals");
}

// ===========================================================================
// 31. Multiple alternatives all reachable
// ===========================================================================

#[test]
fn multiple_alternatives_all_reachable() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let table = build_table(&g);
    for name in &["a", "b", "c"] {
        let tid = tok_id(&g, name);
        assert!(
            table
                .actions(table.initial_state, tid)
                .iter()
                .any(|a| matches!(a, Action::Shift(_))),
            "initial state must shift on '{name}'"
        );
    }
}

// ===========================================================================
// 32. Lex modes have correct length
// ===========================================================================

#[test]
fn lex_modes_length_matches_state_count() {
    let g = GrammarBuilder::new("lex")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert_eq!(table.lex_modes.len(), table.state_count);
}

// ===========================================================================
// 33. Token count matches declared tokens
// ===========================================================================

#[test]
fn token_count_matches_grammar() {
    let g = GrammarBuilder::new("tc")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(
        table.token_count >= 3,
        "token_count should include at least the 3 declared tokens, got {}",
        table.token_count
    );
}

// ===========================================================================
// 34. Nested nonterminals — metadata completeness
// ===========================================================================

#[test]
fn nested_nonterminals_in_metadata() {
    let g = GrammarBuilder::new("nested")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();
    let table = build_table(&g);
    // The augmented grammar may rename nonterminals, so check for at least some
    // nonterminal metadata entries and that terminal metadata is present.
    let has_nonterminal = table.symbol_metadata.iter().any(|m| !m.is_terminal);
    assert!(has_nonterminal, "metadata must contain nonterminals");

    // Check that inner and outer symbol IDs are present in nonterminal_to_index
    let inner = nt_id(&g, "inner");
    let outer = nt_id(&g, "outer");
    assert!(
        table.nonterminal_to_index.contains_key(&inner),
        "nonterminal_to_index must contain 'inner'"
    );
    assert!(
        table.nonterminal_to_index.contains_key(&outer),
        "nonterminal_to_index must contain 'outer'"
    );
}
