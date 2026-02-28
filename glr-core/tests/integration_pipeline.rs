//! Integration test: Grammar → IR → FIRST/FOLLOW → LR(1) → Parse Tables
//!
//! Exercises the full pipeline with a simple arithmetic grammar:
//!   expr → expr PLUS term | term
//!   term → NUMBER

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::StateId;

/// Build a simple arithmetic grammar using the builder API.
fn arithmetic_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("PLUS", "+")
        // expr → expr PLUS term  (left-recursive addition)
        // expr → term
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        // term → NUMBER
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build()
}

#[test]
fn pipeline_grammar_structure() {
    let grammar = arithmetic_grammar();

    // Two tokens: NUMBER, PLUS
    assert_eq!(grammar.tokens.len(), 2, "expected 2 tokens");
    // Two non-terminals with rules: expr, term
    assert_eq!(grammar.rules.len(), 2, "expected 2 non-terminal rule groups");
    // Three productions total
    assert_eq!(grammar.all_rules().count(), 3, "expected 3 productions");

    // Start symbol should resolve
    assert!(grammar.start_symbol().is_some(), "start symbol must exist");
}

#[test]
fn pipeline_normalize_is_idempotent_for_simple_grammar() {
    let mut grammar = arithmetic_grammar();

    let rule_count_before = grammar.all_rules().count();

    // Simple grammar has no complex symbols, so normalize should preserve all rules
    let all_rules = grammar.normalize();

    // normalize() returns all rules (not just new ones); count must be preserved
    assert_eq!(
        all_rules.len(), rule_count_before,
        "simple grammar should have the same number of rules after normalize"
    );
    assert_eq!(grammar.all_rules().count(), rule_count_before);
}

#[test]
fn pipeline_first_follow_sets() {
    let grammar = arithmetic_grammar();

    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW computation should succeed");

    // Locate symbol IDs by name
    let number_id = grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "NUMBER")
        .map(|(id, _)| *id);
    let plus_id = grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "PLUS")
        .map(|(id, _)| *id);
    let expr_id = grammar.find_symbol_by_name("expr");
    let term_id = grammar.find_symbol_by_name("term");

    assert!(number_id.is_some(), "NUMBER symbol must exist");
    assert!(plus_id.is_some(), "PLUS symbol must exist");
    assert!(expr_id.is_some(), "expr symbol must exist");
    assert!(term_id.is_some(), "term symbol must exist");

    let number_id = number_id.unwrap();
    let plus_id = plus_id.unwrap();
    let expr_id = expr_id.unwrap();
    let term_id = term_id.unwrap();

    // FIRST(term) must contain NUMBER
    let first_term = ff.first(term_id).expect("FIRST(term) should exist");
    assert!(
        first_term.contains(number_id.0 as usize),
        "FIRST(term) must contain NUMBER"
    );

    // FIRST(expr) must contain NUMBER (since expr → term → NUMBER)
    let first_expr = ff.first(expr_id).expect("FIRST(expr) should exist");
    assert!(
        first_expr.contains(number_id.0 as usize),
        "FIRST(expr) must contain NUMBER"
    );

    // FOLLOW(expr) must contain PLUS (from expr → expr PLUS term)
    let follow_expr = ff.follow(expr_id).expect("FOLLOW(expr) should exist");
    assert!(
        follow_expr.contains(plus_id.0 as usize),
        "FOLLOW(expr) must contain PLUS"
    );

    // FOLLOW(expr) must also contain EOF sentinel (SymbolId(0))
    assert!(
        follow_expr.contains(0),
        "FOLLOW(expr) must contain EOF (sentinel 0)"
    );

    // expr and term should NOT be nullable
    assert!(!ff.is_nullable(expr_id), "expr must not be nullable");
    assert!(!ff.is_nullable(term_id), "term must not be nullable");
}

#[test]
fn pipeline_lr1_parse_table() {
    let grammar = arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");

    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton construction should succeed");

    // --- basic table dimensions ---
    assert!(table.state_count > 0, "table must have at least one state");
    assert!(
        table.action_table.len() == table.state_count,
        "action_table rows must match state_count"
    );
    assert!(
        table.rules.len() >= 3,
        "table must carry at least the 3 original productions (plus augmented start)"
    );

    // --- EOF symbol should be present in the symbol-to-index map ---
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF symbol must appear in symbol_to_index"
    );

    // --- At least one state must have an Accept action on EOF ---
    let eof_col = *table
        .symbol_to_index
        .get(&table.eof_symbol)
        .expect("EOF column");
    let has_accept = table.action_table.iter().any(|row| {
        row.get(eof_col)
            .is_some_and(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "at least one state must Accept on EOF");

    // --- At least one Shift action should exist (for consuming tokens) ---
    let has_shift = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    });
    assert!(has_shift, "table must contain at least one Shift action");

    // --- At least one Reduce action should exist ---
    let has_reduce = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    });
    assert!(has_reduce, "table must contain at least one Reduce action");
}

#[test]
fn pipeline_goto_table_has_entries() {
    let grammar = arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) should succeed");

    // GOTO table should have one row per state
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto_table rows must match state_count"
    );

    // At least one non-zero GOTO entry must exist (transitions on non-terminals)
    let has_goto = table
        .goto_table
        .iter()
        .any(|row| row.iter().any(|&s| s != StateId(0)));
    assert!(
        has_goto,
        "GOTO table must have at least one non-zero entry for non-terminal transitions"
    );
}

#[test]
fn pipeline_eof_not_colliding_with_grammar_symbols() {
    let grammar = arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) should succeed");

    // EOF symbol must not collide with any grammar token or rule symbol
    for &tok_id in grammar.tokens.keys() {
        assert_ne!(
            table.eof_symbol, tok_id,
            "EOF must not collide with token {:?}",
            tok_id
        );
    }
    for &rule_id in grammar.rules.keys() {
        assert_ne!(
            table.eof_symbol, rule_id,
            "EOF must not collide with non-terminal {:?}",
            rule_id
        );
    }
}

#[test]
fn pipeline_end_to_end_deterministic() {
    // The simple arithmetic grammar (no ambiguity) should produce a deterministic
    // table — every action cell should have at most one action.
    let grammar = arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) should succeed");

    for (state_idx, row) in table.action_table.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            // Filter out Error-only cells
            let non_error: Vec<_> = cell.iter().filter(|a| !matches!(a, Action::Error)).collect();
            assert!(
                non_error.len() <= 1,
                "State {} column {} has {} non-error actions — expected deterministic table \
                 for unambiguous grammar. Actions: {:?}",
                state_idx,
                col_idx,
                non_error.len(),
                cell,
            );
        }
    }
}
