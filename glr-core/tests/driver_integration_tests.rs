#![cfg(feature = "test-api")]
//! Comprehensive integration tests for the GLR Driver.
//!
//! Tests cover Driver creation, token processing, parse table construction
//! from grammars, and end-to-end parsing through the full pipeline.

use adze_glr_core::driver::GlrError;
use adze_glr_core::{
    Action, Driver, FirstFollowSets, Forest, GLRError, GotoIndexing, LexMode, ParseRule,
    ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ─── Helpers ─────────────────────────────────────────────────────────

/// Run normalize → FIRST/FOLLOW → build_lr1_automaton, returning a ParseTable.
fn run_pipeline(grammar: &mut Grammar) -> Result<ParseTable, GLRError> {
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;
    build_lr1_automaton(grammar, &first_follow)
}

/// Build grammar + table, then parse a token stream through the driver.
fn pipeline_parse(
    grammar: &mut Grammar,
    token_stream: &[(SymbolId, u32, u32)],
) -> Result<Forest, GlrError> {
    let table = run_pipeline(grammar).expect("pipeline should produce a table");
    sanity_check_tables(&table).expect("table sanity check");
    let mut driver = Driver::new(&table);
    driver.parse_tokens(
        token_stream
            .iter()
            .map(|&(sym, start, end)| (sym.0 as u32, start, end)),
    )
}

/// Resolve a symbol name to its SymbolId inside a built grammar.
fn sym_id(grammar: &Grammar, name: &str) -> SymbolId {
    for (&id, tok) in &grammar.tokens {
        if tok.name == name {
            return id;
        }
    }
    for (&id, n) in &grammar.rule_names {
        if n == name {
            return id;
        }
    }
    panic!("symbol '{}' not found in grammar", name);
}

type ActionCell = Vec<Action>;

/// Helper to hand-craft a minimal ParseTable for low-level driver tests.
fn create_test_table(
    states: Vec<Vec<ActionCell>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start: SymbolId,
    eof: SymbolId,
) -> ParseTable {
    let symbol_count = states.first().map(|s| s.len()).unwrap_or(0);
    let state_count = states.len();

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        for state_gotos in &gotos {
            if state_gotos[i] != StateId(65535) {
                nonterminal_to_index.insert(SymbolId(i as u16), i);
                break;
            }
        }
    }

    ParseTable {
        action_table: states,
        goto_table: gotos,
        rules: rules.clone(),
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start,
        grammar: Grammar::new("test".to_string()),
        symbol_metadata: vec![],
        initial_state: StateId(0),
        token_count: 2,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; rules.len()],
        rule_assoc_by_rule: vec![0; rules.len()],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
    }
}

// ─── 1. Driver::new with a valid hand-crafted ParseTable ────────────

#[test]
fn driver_new_with_valid_table() {
    let eof = SymbolId(0);
    let _t = SymbolId(1);
    let s = SymbolId(3);

    let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
    let mut actions = vec![vec![vec![]; 4]; 3];
    actions[0][1].push(Action::Shift(StateId(1)));
    actions[1][0].push(Action::Reduce(RuleId(0)));
    actions[2][0].push(Action::Accept);

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 4]; 3];
    gotos[0][3] = StateId(2);

    let table = create_test_table(actions, gotos, rules, s, eof);
    let _driver = Driver::new(&table);
    // Driver creation must not panic for a well-formed table.
}

// ─── 2. Driver::new with pipeline-generated table ───────────────────

#[test]
fn driver_new_from_pipeline_table() {
    let mut grammar = GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline should succeed");
    let _driver = Driver::new(&table);
}

// ─── 3. Driver processes single-token grammar ───────────────────────

#[test]
fn driver_processes_single_token() {
    let mut grammar = GrammarBuilder::new("one_tok")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("single token should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 1);
}

// ─── 4. Driver with simple expression grammar (NUM + NUM) ───────────

#[test]
fn driver_simple_expression_grammar() {
    let mut grammar = GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");

    let forest = pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)])
        .expect("should parse NUM+NUM");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 3);
}

// ─── 5. FIRST/FOLLOW computation succeeds ───────────────────────────

#[test]
fn first_follow_computation_for_expression_grammar() {
    let mut grammar = GrammarBuilder::new("ff_test")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar)
        .expect("FIRST/FOLLOW should compute successfully");

    let num = sym_id(&grammar, "NUM");
    // FIRST(expr) should contain NUM
    let expr_id = sym_id(&grammar, "expr");
    let first_set = ff.first(expr_id).expect("expr should have FIRST set");
    assert!(
        first_set.contains(num.0 as usize),
        "FIRST(expr) must contain NUM"
    );
}

// ─── 6. Canonical collection builds without error ───────────────────

#[test]
fn canonical_collection_builds_for_grammar() {
    let mut grammar = GrammarBuilder::new("cc_test")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW should succeed");

    let table = build_lr1_automaton(&grammar, &ff).expect("build_lr1_automaton should succeed");
    assert!(table.state_count >= 2, "should have multiple states");
    assert!(!table.rules.is_empty(), "should have rules");
}

// ─── 7. Parse table has correct EOF symbol ──────────────────────────

#[test]
fn parse_table_eof_symbol_valid() {
    let mut grammar = GrammarBuilder::new("eof_test")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline should succeed");
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF symbol must be in symbol_to_index"
    );
    // EOF should not collide with any terminal or nonterminal
    assert!(
        !table.nonterminal_to_index.contains_key(&table.eof_symbol),
        "EOF should not be a nonterminal"
    );
}

// ─── 8. Driver rejects invalid token sequence ───────────────────────

#[test]
fn driver_rejects_invalid_token_sequence() {
    // S → 'a' 'b'; feed 'a' 'a' → error
    let eof = SymbolId(0);
    let _a = SymbolId(1);
    let _b = SymbolId(2);
    let s_sym = SymbolId(3);

    let rules = vec![ParseRule {
        lhs: s_sym,
        rhs_len: 2,
    }];

    let mut actions = vec![vec![vec![]; 4]; 4];
    actions[0][1].push(Action::Shift(StateId(1)));
    actions[1][2].push(Action::Shift(StateId(2)));
    actions[2][0].push(Action::Reduce(RuleId(0)));
    actions[3][0].push(Action::Accept);

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 4]; 4];
    gotos[0][3] = StateId(3);

    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(1u32, 0u32, 1u32), (1u32, 1, 2)].iter().copied());
    assert!(result.is_err(), "invalid token sequence should be rejected");
}

// ─── 9. Driver with two-symbol shift-reduce sequence ────────────────

#[test]
fn driver_two_symbol_shift_reduce() {
    // S → 'x' 'y'
    let eof = SymbolId(0);
    let x = SymbolId(1);
    let y = SymbolId(2);
    let s = SymbolId(3);

    let rules = vec![ParseRule { lhs: s, rhs_len: 2 }];

    let mut actions = vec![vec![vec![]; 4]; 4];
    actions[0][1].push(Action::Shift(StateId(1)));
    actions[1][2].push(Action::Shift(StateId(2)));
    actions[2][0].push(Action::Reduce(RuleId(0)));
    actions[3][0].push(Action::Accept);

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 4]; 4];
    gotos[0][3] = StateId(3);

    let table = create_test_table(actions, gotos, rules, s, eof);
    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens(
            [(x.0 as u32, 0u32, 1u32), (y.0 as u32, 1, 2)]
                .iter()
                .copied(),
        )
        .expect("x y should parse via shift-reduce");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 2);
}

// ─── 10. Pipeline-built table passes sanity check ───────────────────

#[test]
fn pipeline_table_passes_sanity_check() {
    let mut grammar = GrammarBuilder::new("sanity")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline should succeed");
    sanity_check_tables(&table).expect("table sanity check should pass");

    // Verify structural properties
    assert!(table.state_count > 0, "must have states");
    assert!(!table.rules.is_empty(), "must have rules");
    assert!(table.symbol_count > 0, "must have symbols");
}

// ─── 11. Parse table contains Accept, Shift, Reduce actions ─────────

#[test]
fn parse_table_has_all_action_types() {
    let mut grammar = GrammarBuilder::new("actions_test")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");

    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    let has_shift = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    });
    let has_reduce = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    });

    assert!(has_accept, "table must contain Accept");
    assert!(has_shift, "table must contain Shift");
    assert!(has_reduce, "table must contain Reduce");
}

// ─── 12. Driver handles left-recursive grammar ──────────────────────

#[test]
fn driver_left_recursive_grammar() {
    // A → A 'a' | 'a'   Input: "aaa"
    let mut grammar = GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("A", vec!["A", "a"])
        .rule("A", vec!["a"])
        .start("A")
        .build();

    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (a, 1, 2), (a, 2, 3)])
        .expect("left-recursive aaa should parse");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 3);
}

// ─── 13. Multiple rule alternatives both accepted ───────────────────

#[test]
fn driver_multiple_alternatives() {
    // S → 'a' | 'b'
    let mut grammar = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    let f1 = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("'a' should parse");
    assert_eq!(f1.view().roots().len(), 1);

    let f2 = pipeline_parse(&mut grammar, &[(b, 0, 1)]).expect("'b' should parse");
    assert_eq!(f2.view().roots().len(), 1);
}

// ─── 14. Driver with multi-level nonterminal chain ──────────────────

#[test]
fn driver_multi_level_nonterminal_chain() {
    // Grammar: expr → term; term → factor; factor → NUM
    let mut grammar = GrammarBuilder::new("chain")
        .token("NUM", r"\d+")
        .rule("expr", vec!["term"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "NUM");
    let forest =
        pipeline_parse(&mut grammar, &[(num, 0, 1)]).expect("nonterminal chain should parse NUM");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 1);
}

// ─── 15. Driver forest span covers full input ───────────────────────

#[test]
fn driver_forest_span_covers_full_input() {
    let mut grammar = GrammarBuilder::new("span_check")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let c = sym_id(&grammar, "c");

    let forest =
        pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2), (c, 2, 3)]).expect("abc should parse");

    let view = forest.view();
    let root = view.roots()[0];
    let span = view.span(root);
    assert_eq!(span.start, 0, "root span should start at 0");
    assert_eq!(span.end, 3, "root span should end at input length");
}

// ─── 16. Debug error stats available on test builds ─────────────────

#[test]
fn driver_forest_debug_error_stats() {
    let mut grammar = GrammarBuilder::new("errstats")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("should parse successfully");

    let (has_error, _missing, _cost) = forest.debug_error_stats();
    assert!(!has_error, "successful parse should have no errors");
}
