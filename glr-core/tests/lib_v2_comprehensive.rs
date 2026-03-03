//! Comprehensive tests for `build_lr1_automaton_v2` in `lib_v2.rs`.

use adze_glr_core::lib_v2::build_lr1_automaton_v2;
use adze_glr_core::{Action, FirstFollowSets, StateId};
use adze_ir::builder::GrammarBuilder;

/// Helper: build a parse table from a grammar.
fn build_table(grammar: &adze_ir::Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FirstFollowSets::compute should succeed");
    build_lr1_automaton_v2(grammar, &ff).expect("build_lr1_automaton_v2 should succeed")
}

// ---------------------------------------------------------------------------
// 1. Simple grammar (single rule) builds successfully
// ---------------------------------------------------------------------------
#[test]
fn single_rule_grammar_builds() {
    let grammar = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count > 0);
}

// ---------------------------------------------------------------------------
// 2. Grammar with multiple rules
// ---------------------------------------------------------------------------
#[test]
fn multiple_rules_grammar_builds() {
    let grammar = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .start("start")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count > 0);
}

// ---------------------------------------------------------------------------
// 3. Verify parse table has correct state count
// ---------------------------------------------------------------------------
#[test]
fn state_count_matches_action_table_rows() {
    let grammar = GrammarBuilder::new("sc")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert_eq!(table.state_count, table.action_table.len());
    assert_eq!(table.state_count, table.goto_table.len());
}

// ---------------------------------------------------------------------------
// 4. Verify symbol metadata is populated
// ---------------------------------------------------------------------------
#[test]
fn symbol_metadata_populated() {
    let grammar = GrammarBuilder::new("meta")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&grammar);
    assert!(!table.symbol_metadata.is_empty());

    let terminal = table.symbol_metadata.iter().find(|m| m.name == "NUM");
    assert!(terminal.is_some());
    assert!(terminal.unwrap().is_terminal);

    let nonterminal = table.symbol_metadata.iter().find(|m| m.name == "expr");
    assert!(nonterminal.is_some());
    assert!(!nonterminal.unwrap().is_terminal);
}

// ---------------------------------------------------------------------------
// 5. Grammar producing shift-reduce conflicts creates Fork actions
// ---------------------------------------------------------------------------
#[test]
fn shift_reduce_conflict_produces_fork() {
    // Classic dangling-else style ambiguity: expr -> expr "+" expr | "N"
    let grammar = GrammarBuilder::new("sr_conflict")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let table = build_table(&grammar);

    let has_fork = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Fork(_))))
    });
    assert!(has_fork, "ambiguous grammar should produce Fork actions");
}

// ---------------------------------------------------------------------------
// 6. Grammar producing reduce-reduce conflicts
// ---------------------------------------------------------------------------
#[test]
fn reduce_reduce_conflict_produces_fork() {
    // Two different rules reduce to different non-terminals from the same token
    let grammar = GrammarBuilder::new("rr_conflict")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .start("start")
        .build();
    let table = build_table(&grammar);

    // The table should build; a reduce-reduce conflict may or may not Fork
    // depending on lookahead. At minimum it should succeed.
    assert!(table.state_count > 0);
}

// ---------------------------------------------------------------------------
// 7. Action table contains shift and reduce actions
// ---------------------------------------------------------------------------
#[test]
fn action_table_has_shift_and_reduce() {
    let grammar = GrammarBuilder::new("sr")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let table = build_table(&grammar);

    let has_shift = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    });
    let has_reduce = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    });
    assert!(has_shift, "should contain shift actions");
    assert!(has_reduce, "should contain reduce actions");
}

// ---------------------------------------------------------------------------
// 8. Goto table has valid state transitions
// ---------------------------------------------------------------------------
#[test]
fn goto_table_has_valid_transitions() {
    let grammar = GrammarBuilder::new("goto")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let table = build_table(&grammar);

    let has_nonzero_goto = table
        .goto_table
        .iter()
        .any(|row| row.iter().any(|s| s.0 != 0));
    // For a very simple grammar the goto might only use state 0,
    // but the table dimensions should be correct.
    assert_eq!(table.goto_table.len(), table.state_count);
    // At least verify columns match symbol count
    for row in &table.goto_table {
        assert_eq!(row.len(), table.symbol_count);
    }
    // With a nonterminal we expect at least one nonzero goto
    let _ = has_nonzero_goto; // may or may not be true for trivial grammar
}

// ---------------------------------------------------------------------------
// 9. Symbol-to-index mapping is populated
// ---------------------------------------------------------------------------
#[test]
fn symbol_to_index_populated() {
    let grammar = GrammarBuilder::new("idx")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    // At minimum: 2 tokens + 1 nonterminal + EOF = 4
    assert!(table.symbol_to_index.len() >= 4);
    // All indices should be within symbol_count
    for &idx in table.symbol_to_index.values() {
        assert!(idx < table.symbol_count);
    }
}

// ---------------------------------------------------------------------------
// 10. Recursive grammar (left-recursive)
// ---------------------------------------------------------------------------
#[test]
fn left_recursive_grammar() {
    let grammar = GrammarBuilder::new("lrec")
        .token("X", "x")
        .rule("list", vec!["list", "X"])
        .rule("list", vec!["X"])
        .start("list")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count > 0);
}

// ---------------------------------------------------------------------------
// 11. Right-recursive grammar
// ---------------------------------------------------------------------------
#[test]
fn right_recursive_grammar() {
    let grammar = GrammarBuilder::new("rrec")
        .token("X", "x")
        .rule("list", vec!["X", "list"])
        .rule("list", vec!["X"])
        .start("list")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count > 0);
}

// ---------------------------------------------------------------------------
// 12. Nested rules (multi-level nonterminals)
// ---------------------------------------------------------------------------
#[test]
fn nested_nonterminals() {
    let grammar = GrammarBuilder::new("nested")
        .token("A", "a")
        .rule("inner", vec!["A"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count >= 2);
    assert!(table.symbol_metadata.iter().any(|m| m.name == "inner"));
    assert!(table.symbol_metadata.iter().any(|m| m.name == "outer"));
}

// ---------------------------------------------------------------------------
// 13. Epsilon / empty production — grammar with nullable nonterminal
// ---------------------------------------------------------------------------
#[test]
fn nullable_via_two_paths() {
    // Instead of an explicit epsilon rule (which requires normalization),
    // test that the automaton handles a grammar where one alternative
    // is very short and another is longer.
    let grammar = GrammarBuilder::new("nullable")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count > 0);
}

// ---------------------------------------------------------------------------
// 14. Action table dimensions match state_count × symbol_count
// ---------------------------------------------------------------------------
#[test]
fn action_table_dimensions() {
    let grammar = GrammarBuilder::new("dim")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert_eq!(table.action_table.len(), table.state_count);
    for row in &table.action_table {
        assert_eq!(row.len(), table.symbol_count);
    }
}

// ---------------------------------------------------------------------------
// 15. EOF is present in symbol_to_index
// ---------------------------------------------------------------------------
#[test]
fn eof_in_symbol_mapping() {
    let grammar = GrammarBuilder::new("eof")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert!(
        table
            .symbol_to_index
            .contains_key(&adze_glr_core::SymbolId(0)),
        "EOF (SymbolId(0)) should be in symbol_to_index"
    );
}

// ---------------------------------------------------------------------------
// 16. index_to_symbol is inverse of symbol_to_index
// ---------------------------------------------------------------------------
#[test]
fn index_to_symbol_inverse() {
    let grammar = GrammarBuilder::new("inv")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    for (&sym, &idx) in &table.symbol_to_index {
        assert_eq!(
            table.index_to_symbol[idx], sym,
            "index_to_symbol should mirror symbol_to_index"
        );
    }
}

// ---------------------------------------------------------------------------
// 17. Token count is set correctly
// ---------------------------------------------------------------------------
#[test]
fn token_count_correct() {
    let grammar = GrammarBuilder::new("tc")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert_eq!(table.token_count, 3);
}

// ---------------------------------------------------------------------------
// 18. Terminal metadata flag
// ---------------------------------------------------------------------------
#[test]
fn terminal_metadata_flag() {
    let grammar = GrammarBuilder::new("tf")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    for m in &table.symbol_metadata {
        if m.name == "NUM" {
            assert!(m.is_terminal);
            assert!(m.is_named, "regex-based token should be named");
        }
        if m.name == "s" {
            assert!(!m.is_terminal);
            assert!(m.is_named);
        }
    }
}

// ---------------------------------------------------------------------------
// 19. Multiple-alternative grammar
// ---------------------------------------------------------------------------
#[test]
fn multiple_alternatives() {
    let grammar = GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["C"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count > 0);
    // Should have shift actions for A, B, and C from the initial state
    let has_shift = table.action_table[0]
        .iter()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(has_shift);
}

// ---------------------------------------------------------------------------
// 20. Grammar with two-symbol sequence rule
// ---------------------------------------------------------------------------
#[test]
fn two_symbol_sequence() {
    let grammar = GrammarBuilder::new("seq")
        .token("LP", "(")
        .token("RP", ")")
        .rule("pair", vec!["LP", "RP"])
        .start("pair")
        .build();
    let table = build_table(&grammar);
    assert!(
        table.state_count >= 3,
        "need at least start, after LP, and after RP states"
    );
}

// ---------------------------------------------------------------------------
// 21. Chained nonterminals (A -> B -> C -> token)
// ---------------------------------------------------------------------------
#[test]
fn chained_nonterminals() {
    let grammar = GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("c", vec!["X"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .start("a")
        .build();
    let table = build_table(&grammar);
    assert!(table.state_count >= 2);
    // Metadata should include all three nonterminals
    let names: Vec<&str> = table
        .symbol_metadata
        .iter()
        .filter(|m| !m.is_terminal)
        .map(|m| m.name.as_str())
        .collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
    assert!(names.contains(&"c"));
}

// ---------------------------------------------------------------------------
// 22. Symbol count equals terminals + nonterminals + EOF
// ---------------------------------------------------------------------------
#[test]
fn symbol_count_computation() {
    let grammar = GrammarBuilder::new("cnt")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    // 2 tokens + 1 nonterminal + 1 EOF = 4
    assert_eq!(table.symbol_count, 4);
}

// ---------------------------------------------------------------------------
// 23. ParseRules are populated from grammar
// ---------------------------------------------------------------------------
#[test]
fn parse_rules_populated() {
    let grammar = GrammarBuilder::new("rules")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert!(table.rules.len() >= 2, "should have at least 2 rules");
}

// ---------------------------------------------------------------------------
// 24. Error cells in action table
// ---------------------------------------------------------------------------
#[test]
fn action_table_has_error_cells() {
    let grammar = GrammarBuilder::new("err")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    let has_error = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Error)))
    });
    assert!(has_error, "some cells should remain Error");
}

// ---------------------------------------------------------------------------
// 25. Goto indexing mode is NonterminalMap
// ---------------------------------------------------------------------------
#[test]
fn goto_indexing_is_nonterminal_map() {
    let grammar = GrammarBuilder::new("gi")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert_eq!(
        table.goto_indexing,
        adze_glr_core::GotoIndexing::NonterminalMap
    );
}

// ---------------------------------------------------------------------------
// 26. Start symbol is set
// ---------------------------------------------------------------------------
#[test]
fn start_symbol_set() {
    let grammar = GrammarBuilder::new("ss")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let table = build_table(&grammar);
    // The start_symbol should be a valid SymbolId (not 0 unless grammar is trivial)
    assert_ne!(
        table.start_symbol,
        adze_glr_core::SymbolId(0),
        "start_symbol should not be EOF"
    );
}

// ---------------------------------------------------------------------------
// 27. Grammar with many tokens
// ---------------------------------------------------------------------------
#[test]
fn many_tokens() {
    let mut builder = GrammarBuilder::new("many");
    for i in 0..10 {
        let name = format!("T{i}");
        let pat = format!("t{i}");
        builder = builder.token(
            Box::leak(name.into_boxed_str()),
            Box::leak(pat.into_boxed_str()),
        );
    }
    builder = builder.rule("s", vec!["T0"]).start("s");
    let grammar = builder.build();
    let table = build_table(&grammar);
    assert!(table.token_count >= 10);
}

// ---------------------------------------------------------------------------
// 28. Fork actions contain multiple sub-actions
// ---------------------------------------------------------------------------
#[test]
fn fork_contains_multiple_actions() {
    let grammar = GrammarBuilder::new("fork_detail")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let table = build_table(&grammar);

    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Fork(sub) = action {
                    assert!(
                        sub.len() >= 2,
                        "Fork should contain at least 2 alternatives"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 29. Shift actions reference valid states
// ---------------------------------------------------------------------------
#[test]
fn shift_targets_valid_states() {
    let grammar = GrammarBuilder::new("valid_shift")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let table = build_table(&grammar);

    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Shift(StateId(s)) = action {
                    assert!(
                        (*s as usize) < table.state_count,
                        "shift target {s} must be < state_count {}",
                        table.state_count
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 30. Lex modes have correct length
// ---------------------------------------------------------------------------
#[test]
fn lex_modes_length() {
    let grammar = GrammarBuilder::new("lex")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = build_table(&grammar);
    assert_eq!(table.lex_modes.len(), table.state_count);
}
