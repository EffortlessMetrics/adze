//! End-to-end pipeline tests: Grammar construction → FIRST/FOLLOW → LR(1) automaton → ParseTable.
//!
//! These tests verify the full IR → GLR core pipeline produces valid, consistent results
//! across a variety of grammar shapes.

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

/// Helper: build a grammar through the full pipeline, returning the parse table.
fn full_pipeline(grammar: &mut Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

/// Helper: check parse table basic invariants
fn assert_table_valid(table: &ParseTable) {
    assert!(table.state_count > 0, "Must have at least 1 state");
    assert_eq!(
        table.action_table.len(),
        table.state_count,
        "Action table rows must match state count"
    );
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "Goto table rows must match state count"
    );
}

// =============================================================================
// Single-rule grammars
// =============================================================================

#[test]
fn pipeline_single_terminal_rule() {
    let mut grammar = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

#[test]
fn pipeline_two_terminal_sequence() {
    let mut grammar = GrammarBuilder::new("seq2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
    // A→B sequence needs at least 3 states: initial, after A, after B (accept)
    assert!(
        table.state_count >= 3,
        "A→B sequence needs >= 3 states, got {}",
        table.state_count
    );
}

#[test]
fn pipeline_three_terminal_sequence() {
    let mut grammar = GrammarBuilder::new("seq3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

// =============================================================================
// Multi-rule grammars
// =============================================================================

#[test]
fn pipeline_two_rules_chain() {
    let mut grammar = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

#[test]
fn pipeline_three_rules_chain() {
    let mut grammar = GrammarBuilder::new("deep_chain")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

#[test]
fn pipeline_mixed_terminals_and_nonterminals() {
    let mut grammar = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

// =============================================================================
// Determinism
// =============================================================================

#[test]
fn pipeline_deterministic_same_grammar() {
    let build = || {
        let mut grammar = GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        full_pipeline(&mut grammar)
    };
    let table1 = build();
    let table2 = build();
    assert_eq!(table1.state_count, table2.state_count);
    assert_eq!(table1.action_table.len(), table2.action_table.len());
}

// =============================================================================
// Accept action presence
// =============================================================================

#[test]
fn pipeline_has_accept_action() {
    let mut grammar = GrammarBuilder::new("acc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);

    let has_accept = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .any(|action| matches!(action, Action::Accept));
    assert!(
        has_accept,
        "Parse table must contain at least one Accept action"
    );
}

// =============================================================================
// Shift and Reduce actions
// =============================================================================

#[test]
fn pipeline_has_shift_actions() {
    let mut grammar = GrammarBuilder::new("shifts")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);

    let has_shift = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .any(|action| matches!(action, Action::Shift(_)));
    assert!(has_shift, "Parse table must contain shift actions");
}

#[test]
fn pipeline_has_reduce_actions() {
    let mut grammar = GrammarBuilder::new("reduces")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);

    let has_reduce = table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .any(|action| matches!(action, Action::Reduce(_)));
    assert!(has_reduce, "Parse table must contain reduce actions");
}

// =============================================================================
// State count properties
// =============================================================================

#[test]
fn pipeline_more_rules_generally_more_states() {
    let mut g1 = GrammarBuilder::new("small")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t1 = full_pipeline(&mut g1);

    let mut g2 = GrammarBuilder::new("bigger")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let t2 = full_pipeline(&mut g2);

    assert!(
        t2.state_count >= t1.state_count,
        "Larger grammar should have >= states: {} vs {}",
        t2.state_count,
        t1.state_count
    );
}

// =============================================================================
// FIRST/FOLLOW set properties
// =============================================================================

#[test]
fn first_follow_computes_successfully() {
    let mut grammar = GrammarBuilder::new("ff")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    grammar.normalize();
    let ff = FirstFollowSets::compute(&grammar);
    assert!(ff.is_ok(), "FIRST/FOLLOW should compute: {:?}", ff.err());
}

#[test]
fn first_follow_compute_normalized() {
    let mut grammar = GrammarBuilder::new("ffn")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar);
    assert!(
        ff.is_ok(),
        "compute_normalized should succeed: {:?}",
        ff.err()
    );
}

// =============================================================================
// Symbol mapping consistency
// =============================================================================

#[test]
fn pipeline_symbol_to_index_consistent() {
    let mut grammar = GrammarBuilder::new("sym")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);

    // Every symbol_to_index entry should map to a valid column
    for (sym, &idx) in &table.symbol_to_index {
        for state_row in &table.action_table {
            assert!(
                idx < state_row.len(),
                "Symbol {:?} maps to index {} but row has only {} columns",
                sym,
                idx,
                state_row.len()
            );
        }
    }
}

#[test]
fn pipeline_eof_symbol_in_table() {
    let mut grammar = GrammarBuilder::new("eof")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);

    // EOF symbol should be present in symbol_to_index
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF symbol {:?} should be in symbol_to_index",
        table.eof_symbol
    );
}

// =============================================================================
// Precedence grammars
// =============================================================================

#[test]
fn pipeline_with_precedence() {
    use adze_ir::Associativity;
    let mut grammar = GrammarBuilder::new("prec")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("expr", vec!["num"])
        .rule("add", vec!["expr", "plus", "expr"])
        .rule("mul", vec!["expr", "star", "expr"])
        .start("expr")
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

// =============================================================================
// Grammar with extras
// =============================================================================

#[test]
fn pipeline_with_extras() {
    let mut grammar = GrammarBuilder::new("extras")
        .token("a", "a")
        .token("ws", "\\s+")
        .rule("start", vec!["a"])
        .start("start")
        .extra("ws")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

// =============================================================================
// Python-like and JavaScript-like grammars
// =============================================================================

#[test]
fn pipeline_python_like_grammar() {
    let mut grammar = GrammarBuilder::python_like();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
    assert!(
        table.state_count >= 3,
        "Python-like grammar should have >= 3 states, got {}",
        table.state_count
    );
}

#[test]
fn pipeline_javascript_like_grammar() {
    let mut grammar = GrammarBuilder::javascript_like();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

// =============================================================================
// Goto table consistency
// =============================================================================

#[test]
fn pipeline_goto_table_rows_match_states() {
    let mut grammar = GrammarBuilder::new("goto")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_eq!(table.goto_table.len(), table.state_count);
}

// =============================================================================
// Idempotency of normalization
// =============================================================================

#[test]
fn normalize_is_idempotent() {
    let mut grammar = GrammarBuilder::new("idem")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();

    grammar.normalize();
    let rules_after_first = grammar.all_rules().count();
    grammar.normalize();
    let rules_after_second = grammar.all_rules().count();
    assert_eq!(
        rules_after_first, rules_after_second,
        "Normalization should be idempotent"
    );
}

// =============================================================================
// Table compression tests are in tablegen crate (requires tablegen dep)
// =============================================================================

// =============================================================================
// Grammar validation
// =============================================================================

#[test]
fn pipeline_valid_grammar_validates() {
    let grammar = GrammarBuilder::new("valid")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let result = grammar.validate();
    assert!(
        result.is_ok(),
        "Valid grammar should validate: {:?}",
        result.err()
    );
}

// =============================================================================
// Grammar with externals
// =============================================================================

#[test]
fn pipeline_with_externals() {
    let mut grammar = GrammarBuilder::new("ext")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .external("indent")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn pipeline_grammar_with_single_token() {
    let mut grammar = GrammarBuilder::new("single_tok")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
    assert!(table.state_count >= 2);
}

#[test]
fn pipeline_long_sequence() {
    let mut gb = GrammarBuilder::new("long");
    let mut tokens = Vec::new();
    for i in 0..10 {
        let name = format!("t{}", i);
        gb = gb.token(&name, &name);
        tokens.push(name);
    }
    let token_refs: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
    gb = gb.rule("start", token_refs).start("start");
    let mut grammar = gb.build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
    // 10 tokens → at least 11 states (shift each token + final state)
    assert!(
        table.state_count >= 10,
        "10-token sequence should have >= 10 states, got {}",
        table.state_count
    );
}

#[test]
fn pipeline_multiple_independent_rules() {
    let mut grammar = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("r3", vec!["c"])
        .rule("start", vec!["r1"])
        .start("start")
        .build();
    let table = full_pipeline(&mut grammar);
    assert_table_valid(&table);
}
