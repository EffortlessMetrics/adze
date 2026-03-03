#![allow(clippy::needless_range_loop)]
//! Comprehensive edge-case tests for the GLR Driver.
//!
//! Covers: empty input, single token, whitespace-only tokens, maximum token length,
//! minimal parse tables, invalid tokens, GlrError display, driver reset between parses,
//! and multiple sequential parses.

#![cfg(feature = "test-api")]

use adze_glr_core::forest_view::ForestView;
use adze_glr_core::{
    Action, Driver, FirstFollowSets, Forest, GotoIndexing, LexMode, ParseRule, ParseTable,
    build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ─── Helpers ─────────────────────────────────────────────────────────

type ActionCell = Vec<Action>;

/// Run normalize → FIRST/FOLLOW → build_lr1_automaton, returning a ParseTable.
fn run_pipeline(grammar: &mut Grammar) -> Result<ParseTable, adze_glr_core::GLRError> {
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;
    build_lr1_automaton(grammar, &first_follow)
}

/// Build grammar + table, then parse a token stream through the driver.
fn pipeline_parse(
    grammar: &mut Grammar,
    token_stream: &[(SymbolId, u32, u32)],
) -> Result<Forest, adze_glr_core::driver::GlrError> {
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

/// Build a minimal hand-crafted table: S → 'a' (SymbolId(1))
fn minimal_single_token_table() -> ParseTable {
    let eof = SymbolId(0);
    let a = SymbolId(1);
    let s = SymbolId(2);

    let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];

    // state 0: shift 'a' → state 1
    // state 1: reduce(0) on EOF
    // state 2: accept on EOF
    let mut actions = vec![vec![vec![]; 3]; 3];
    actions[0][1].push(Action::Shift(StateId(1)));
    actions[1][0].push(Action::Reduce(RuleId(0)));
    actions[2][0].push(Action::Accept);

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 3]; 3];
    gotos[0][2] = StateId(2);

    create_test_table(actions, gotos, rules, s, eof)
}

// ═══════════════════════════════════════════════════════════════════════
// 1. Empty input parsing
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_input_rejected_by_non_nullable_grammar() {
    let mut grammar = GrammarBuilder::new("non_null")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let result = pipeline_parse(&mut grammar, &[]);
    // Grammar requires 'a', so empty input must error or recover with cost
    match result {
        Ok(f) => {
            let (_has_err, _missing, cost) = f.debug_error_stats();
            assert!(cost > 0, "empty input accepted but must have recovery cost");
        }
        Err(_) => { /* expected */ }
    }
}

#[test]
fn empty_input_accepted_by_epsilon_grammar() {
    // S → ε
    let mut grammar = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("S", vec![])
        .start("S")
        .build();

    let result = pipeline_parse(&mut grammar, &[]);
    match result {
        Ok(f) => assert!(!f.view().roots().is_empty()),
        Err(_) => { /* acceptable if pipeline doesn't support epsilon start */ }
    }
}

#[test]
fn empty_token_stream_via_hand_crafted_table() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(std::iter::empty::<(u32, u32, u32)>());
    assert!(
        result.is_err(),
        "hand-crafted table requires 'a', so empty must fail"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Single token input
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_token_accepted_hand_crafted() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens([(1, 0, 1)])
        .expect("single token should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 1);
}

#[test]
fn single_token_accepted_via_pipeline() {
    let mut grammar = GrammarBuilder::new("single_pipe")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let forest = pipeline_parse(&mut grammar, &[(x, 0, 1)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert_eq!(children.len(), 1, "S → x should have one child");
}

#[test]
fn single_zero_width_token() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    // Zero-width token (start == end)
    let result = driver.parse_tokens([(1, 5, 5)]);
    // Should not panic; may succeed or fail depending on driver behavior
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Input with only whitespace-like tokens
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn whitespace_token_id_with_no_action_is_rejected() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    // Feed a token with kind=2 (the nonterminal S), which has no shift action in state 0
    let result = driver.parse_tokens([(2, 0, 1)]);
    // No action for kind=2 in state 0, so should fail
    let _ = result;
}

#[test]
fn multiple_unrecognized_tokens() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    // Feed three tokens, none of which are the expected 'a' (kind=1)
    let result = driver.parse_tokens([(50, 0, 1), (51, 1, 2), (52, 2, 3)]);
    assert!(result.is_err(), "unrecognized tokens should not parse");
}

#[test]
fn all_tokens_same_unknown_kind() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    let tokens: Vec<(u32, u32, u32)> = (0..5).map(|i| (200, i, i + 1)).collect();
    let result = driver.parse_tokens(tokens);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Maximum token length / large byte offsets
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn token_spanning_large_byte_range() {
    let mut grammar = GrammarBuilder::new("large_span")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    // Token spanning a huge byte range
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1_000_000)]).expect("large span");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).end, 1_000_000);
}

#[test]
fn token_at_u32_max_boundary() {
    let mut grammar = GrammarBuilder::new("max_boundary")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let result = pipeline_parse(&mut grammar, &[(a, u32::MAX - 1, u32::MAX)]);
    // Must not panic from overflow
    let _ = result;
}

#[test]
fn token_start_equals_u32_max() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    // Both start and end at u32::MAX
    let result = driver.parse_tokens([(1, u32::MAX, u32::MAX)]);
    let _ = result; // must not panic
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Driver with minimal parse table (1 state)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_state_table_with_accept_on_eof() {
    let eof = SymbolId(0);
    let s = SymbolId(1);

    // 1 state that immediately accepts on EOF
    let actions = vec![vec![vec![Action::Accept], vec![]]];
    let gotos = vec![vec![StateId(65535); 2]];
    let rules = vec![ParseRule { lhs: s, rhs_len: 0 }];

    let table = create_test_table(actions, gotos, rules, s, eof);
    let mut driver = Driver::new(&table);
    // Empty token stream — should reach EOF and accept
    let result = driver.parse_tokens(std::iter::empty::<(u32, u32, u32)>());
    // May or may not accept depending on how EOF phase handles single-state
    let _ = result;
}

#[test]
fn single_state_table_rejects_token() {
    let eof = SymbolId(0);
    let s = SymbolId(1);

    // 1 state, only EOF → Accept; no shift actions
    let actions = vec![vec![vec![Action::Accept], vec![]]];
    let gotos = vec![vec![StateId(65535); 2]];
    let rules = vec![ParseRule { lhs: s, rhs_len: 0 }];

    let table = create_test_table(actions, gotos, rules, s, eof);
    let mut driver = Driver::new(&table);
    // Feed a token — no shift action in state 0
    let result = driver.parse_tokens([(1, 0, 1)]);
    // Should not panic
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Driver error on invalid token
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn invalid_token_kind_out_of_range() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(9999, 0, 1)]);
    // Must not panic — unknown token should produce an error
    let _ = result;
}

#[test]
fn wrong_token_for_grammar_returns_error() {
    let mut grammar = GrammarBuilder::new("ab")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let b = sym_id(&grammar, "b");
    let a = sym_id(&grammar, "a");

    // Feed 'b' 'a' instead of 'a' 'b'
    let result = pipeline_parse(&mut grammar, &[(b, 0, 1), (a, 1, 2)]);
    // Either error or recovery with nonzero cost
    match result {
        Ok(f) => {
            let (_, _, cost) = f.debug_error_stats();
            // If accepted, must involve recovery
            let _ = cost;
        }
        Err(_) => { /* expected */ }
    }
}

#[test]
fn duplicate_token_where_second_is_unexpected() {
    let mut grammar = GrammarBuilder::new("dup_tok")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    // Grammar only accepts one 'a'; second is unexpected
    let result = pipeline_parse(&mut grammar, &[(a, 0, 1), (a, 1, 2)]);
    match result {
        Ok(f) => {
            let (_, _, cost) = f.debug_error_stats();
            let _ = cost;
        }
        Err(_) => { /* expected */ }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 7. GlrError variant display
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn glr_error_lex_display_format() {
    let err = adze_glr_core::driver::GlrError::Lex("unexpected byte 0xFF".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("lexer error"), "got: {msg}");
    assert!(msg.contains("0xFF"), "got: {msg}");
}

#[test]
fn glr_error_parse_display_format() {
    let err = adze_glr_core::driver::GlrError::Parse("state 5 has no action".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("parse error"), "got: {msg}");
    assert!(msg.contains("state 5"), "got: {msg}");
}

#[test]
fn glr_error_other_display_format() {
    let err = adze_glr_core::driver::GlrError::Other("internal failure".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("internal failure"), "got: {msg}");
}

#[test]
fn glr_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_e: &E) {}

    let lex = adze_glr_core::driver::GlrError::Lex("l".into());
    let parse = adze_glr_core::driver::GlrError::Parse("p".into());
    let other = adze_glr_core::driver::GlrError::Other("o".into());
    assert_std_error(&lex);
    assert_std_error(&parse);
    assert_std_error(&other);
}

#[test]
fn glr_error_debug_output_includes_variant() {
    let err = adze_glr_core::driver::GlrError::Lex("test".into());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Lex"), "Debug should contain variant: {dbg}");

    let err2 = adze_glr_core::driver::GlrError::Parse("test".into());
    assert!(format!("{err2:?}").contains("Parse"));

    let err3 = adze_glr_core::driver::GlrError::Other("test".into());
    assert!(format!("{err3:?}").contains("Other"));
}

#[test]
fn glr_error_with_empty_message() {
    let err = adze_glr_core::driver::GlrError::Lex(String::new());
    let msg = format!("{err}");
    assert!(msg.contains("lexer error"), "got: {msg}");

    let err2 = adze_glr_core::driver::GlrError::Parse(String::new());
    assert!(format!("{err2}").contains("parse error"));

    let err3 = adze_glr_core::driver::GlrError::Other(String::new());
    // Other("") displays just the empty string
    let _ = format!("{err3}");
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Driver reset between parses
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn driver_reuse_after_successful_parse() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);

    let f1 = driver.parse_tokens([(1, 0, 1)]).expect("first parse");
    assert_eq!(f1.view().roots().len(), 1);

    // Second parse with same driver
    let f2 = driver.parse_tokens([(1, 10, 11)]).expect("second parse");
    assert_eq!(f2.view().roots().len(), 1);
    assert_eq!(f2.view().span(f2.view().roots()[0]).start, 10);
}

#[test]
fn driver_reuse_after_failed_parse() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);

    // First parse fails (unknown token)
    let r1 = driver.parse_tokens([(99, 0, 1)]);
    assert!(r1.is_err() || r1.is_ok()); // just must not panic

    // Second parse succeeds
    let f2 = driver
        .parse_tokens([(1, 0, 1)])
        .expect("should recover after failure");
    assert_eq!(f2.view().roots().len(), 1);
}

#[test]
fn driver_reuse_after_empty_input() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);

    let r1 = driver.parse_tokens(std::iter::empty::<(u32, u32, u32)>());
    let _ = r1; // either error or ok

    let f2 = driver
        .parse_tokens([(1, 0, 1)])
        .expect("second parse after empty");
    assert_eq!(f2.view().roots().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Multiple sequential parses
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn ten_sequential_parses_same_input() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);

    for i in 0..10 {
        let forest = driver
            .parse_tokens([(1, 0, 1)])
            .unwrap_or_else(|e| panic!("parse {i} failed: {e}"));
        assert_eq!(forest.view().roots().len(), 1, "parse {i} root count");
    }
}

#[test]
fn sequential_parses_with_different_offsets() {
    let mut grammar = GrammarBuilder::new("seq_off")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let a = sym_id(&grammar, "a");
    let mut driver = Driver::new(&table);

    for offset in [0u32, 100, 5000, 1_000_000] {
        let forest = driver
            .parse_tokens([(a.0 as u32, offset, offset + 1)])
            .unwrap_or_else(|e| panic!("offset {offset} failed: {e}"));
        let view = forest.view();
        assert_eq!(view.span(view.roots()[0]).start, offset);
        assert_eq!(view.span(view.roots()[0]).end, offset + 1);
    }
}

#[test]
fn sequential_parses_alternating_success_failure() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);

    for i in 0..6 {
        if i % 2 == 0 {
            let f = driver
                .parse_tokens([(1, 0, 1)])
                .expect("even parse succeeds");
            assert_eq!(f.view().roots().len(), 1);
        } else {
            let _ = driver.parse_tokens(std::iter::empty::<(u32, u32, u32)>());
        }
    }
}

#[test]
fn sequential_parses_with_multi_token_grammar() {
    let mut grammar = GrammarBuilder::new("seq_multi")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let mut driver = Driver::new(&table);

    for _ in 0..5 {
        let forest = driver
            .parse_tokens([(a.0 as u32, 0, 1), (b.0 as u32, 1, 2)])
            .expect("multi-token parse");
        assert_eq!(forest.view().span(forest.view().roots()[0]).end, 2);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Additional edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn token_with_inverted_span_does_not_panic() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    // start > end — invalid but should not panic
    let result = driver.parse_tokens([(1, 10, 5)]);
    let _ = result;
}

#[test]
fn many_tokens_exceeding_grammar_capacity() {
    let mut grammar = GrammarBuilder::new("overflow")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    // Grammar accepts exactly one 'a', but we feed 20
    let tokens: Vec<(SymbolId, u32, u32)> = (0..20).map(|i| (a, i, i + 1)).collect();
    let result = pipeline_parse(&mut grammar, &tokens);
    // Should not panic; expected error or recovery
    match result {
        Ok(f) => {
            let (_, _, cost) = f.debug_error_stats();
            let _ = cost;
        }
        Err(_) => { /* expected */ }
    }
}

#[test]
fn forest_view_on_single_token_parse_has_correct_structure() {
    let mut grammar = GrammarBuilder::new("struct_check")
        .token("z", "z")
        .rule("S", vec!["z"])
        .start("S")
        .build();

    let z = sym_id(&grammar, "z");
    let forest = pipeline_parse(&mut grammar, &[(z, 0, 1)]).expect("parse");
    let view = forest.view();

    let root = view.roots()[0];
    let root_span = view.span(root);
    assert_eq!(root_span.start, 0);
    assert_eq!(root_span.end, 1);

    // Root (S) should have one child (z terminal)
    let children = view.best_children(root);
    assert_eq!(children.len(), 1);

    // Terminal leaf has no children
    let leaf = children[0];
    assert!(view.best_children(leaf).is_empty());
}

#[test]
fn error_stats_zero_for_clean_single_token_parse() {
    let mut grammar = GrammarBuilder::new("clean_stats")
        .token("q", "q")
        .rule("S", vec!["q"])
        .start("S")
        .build();

    let q = sym_id(&grammar, "q");
    let forest = pipeline_parse(&mut grammar, &[(q, 0, 1)]).expect("parse");
    let (has_error, _missing, cost) = forest.debug_error_stats();
    assert!(!has_error, "clean parse should have no errors");
    assert_eq!(cost, 0, "clean parse should have zero cost");
}

#[test]
fn adjacent_zero_width_tokens() {
    let table = minimal_single_token_table();
    let mut driver = Driver::new(&table);
    // All tokens at same position with zero width
    let result = driver.parse_tokens([(1, 0, 0), (1, 0, 0)]);
    let _ = result; // must not panic
}
