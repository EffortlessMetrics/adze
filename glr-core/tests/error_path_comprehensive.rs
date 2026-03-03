#![allow(clippy::needless_range_loop)]
//! Comprehensive error-path and edge-case tests for adze-glr-core.
//!
//! Covers: empty tables, invalid indices, missing symbols, GlrError formatting,
//! parse failures, empty grammars, and FIRST/FOLLOW edge cases.

use adze_glr_core::driver::{Driver, GlrError};
use adze_glr_core::{
    Action, FirstFollowSets, GLRError, GotoIndexing, LexMode, ParseRule, ParseTable, SymbolId,
    build_lr1_automaton,
};
use adze_ir::*;
use std::collections::BTreeMap;

// ─── helpers ────────────────────────────────────────────────────────

/// Build a ParseTable with the given dimensions but no meaningful content.
fn empty_parse_table() -> ParseTable {
    ParseTable::default()
}

/// Build a minimal ParseTable with explicit action/goto tables.
fn make_table(
    actions: Vec<Vec<Vec<Action>>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start: SymbolId,
    eof: SymbolId,
) -> ParseTable {
    let state_count = actions.len();
    let col_count = actions.first().map(|r| r.len()).unwrap_or(0);

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..col_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for i in 0..col_count {
        for row in &gotos {
            if let Some(&s) = row.get(i)
                && s.0 != u16::MAX
            {
                nonterminal_to_index.insert(SymbolId(i as u16), i);
                break;
            }
        }
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules,
        state_count,
        symbol_count: col_count,
        symbol_to_index,
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start,
        grammar: Grammar::new("test".into()),
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
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
    }
}

/// Simple grammar: S → a
fn simple_grammar() -> Grammar {
    let mut g = Grammar::new("simple".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

// ════════════════════════════════════════════════════════════════════
// 1. Empty parse tables
// ════════════════════════════════════════════════════════════════════

#[test]
fn empty_table_actions_returns_empty_slice() {
    let table = empty_parse_table();
    let actions = table.actions(StateId(0), SymbolId(0));
    assert!(actions.is_empty());
}

#[test]
fn empty_table_goto_returns_none() {
    let table = empty_parse_table();
    assert!(table.goto(StateId(0), SymbolId(0)).is_none());
}

#[test]
fn empty_table_eof_is_default() {
    let table = empty_parse_table();
    assert_eq!(table.eof(), SymbolId(0));
}

#[test]
fn empty_table_valid_symbols_mask_empty() {
    let table = empty_parse_table();
    let mask = table.valid_symbols_mask(StateId(0));
    assert!(mask.is_empty() || mask.iter().all(|&v| !v));
}

// ════════════════════════════════════════════════════════════════════
// 2. Invalid state indices
// ════════════════════════════════════════════════════════════════════

#[test]
fn actions_with_out_of_bounds_state_returns_empty() {
    let table = make_table(
        vec![vec![vec![Action::Shift(StateId(1))], vec![]]],
        vec![vec![StateId(u16::MAX)]],
        vec![],
        SymbolId(0),
        SymbolId(0),
    );
    // State 999 does not exist
    let actions = table.actions(StateId(999), SymbolId(0));
    assert!(actions.is_empty());
}

#[test]
fn goto_with_out_of_bounds_state_returns_none() {
    let table = make_table(
        vec![vec![vec![], vec![]]],
        vec![vec![StateId(1)]],
        vec![],
        SymbolId(0),
        SymbolId(0),
    );
    assert!(table.goto(StateId(999), SymbolId(0)).is_none());
}

#[test]
fn lex_mode_out_of_bounds_returns_default() {
    let table = empty_parse_table();
    let mode = table.lex_mode(StateId(42));
    assert_eq!(mode.lex_state, 0);
    assert_eq!(mode.external_lex_state, 0);
}

// ════════════════════════════════════════════════════════════════════
// 3. Missing symbols in table
// ════════════════════════════════════════════════════════════════════

#[test]
fn actions_with_unmapped_symbol_returns_empty() {
    let table = make_table(
        vec![vec![vec![Action::Accept], vec![]]],
        vec![vec![StateId(u16::MAX)]],
        vec![],
        SymbolId(0),
        SymbolId(0),
    );
    // SymbolId(99) is not in symbol_to_index
    let actions = table.actions(StateId(0), SymbolId(99));
    assert!(actions.is_empty());
}

#[test]
fn goto_with_unmapped_nonterminal_returns_none() {
    let table = make_table(
        vec![vec![vec![], vec![]]],
        vec![vec![StateId(1), StateId(u16::MAX)]],
        vec![],
        SymbolId(0),
        SymbolId(0),
    );
    // SymbolId(50) is not in nonterminal_to_index
    assert!(table.goto(StateId(0), SymbolId(50)).is_none());
}

#[test]
fn is_extra_returns_false_for_unknown_symbol() {
    let table = empty_parse_table();
    assert!(!table.is_extra(SymbolId(42)));
}

// ════════════════════════════════════════════════════════════════════
// 4. GlrError (driver) formatting and matching
// ════════════════════════════════════════════════════════════════════

#[test]
fn glr_error_lex_display() {
    let err = GlrError::Lex("unexpected byte 0xFF".into());
    let msg = format!("{err}");
    assert!(msg.contains("lexer error"), "got: {msg}");
    assert!(msg.contains("0xFF"), "got: {msg}");
}

#[test]
fn glr_error_parse_display() {
    let err = GlrError::Parse("no valid parse paths".into());
    let msg = format!("{err}");
    assert!(msg.contains("parse error"), "got: {msg}");
}

#[test]
fn glr_error_other_display() {
    let err = GlrError::Other("something went wrong".into());
    let msg = format!("{err}");
    assert!(msg.contains("something went wrong"), "got: {msg}");
}

#[test]
fn glr_error_debug_includes_variant_name() {
    let err = GlrError::Lex("bad".into());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Lex"), "got: {dbg}");
}

#[test]
fn glr_error_variants_are_distinguishable() {
    let lex = GlrError::Lex("x".into());
    let parse = GlrError::Parse("x".into());
    let other = GlrError::Other("x".into());

    // Each variant produces a different Display message
    let msgs: Vec<String> = [lex, parse, other].iter().map(|e| format!("{e}")).collect();
    assert_ne!(msgs[0], msgs[1]);
    assert_ne!(msgs[1], msgs[2]);
    assert_ne!(msgs[0], msgs[2]);
}

// ════════════════════════════════════════════════════════════════════
// 5. GLRError (lib-level) formatting and matching
// ════════════════════════════════════════════════════════════════════

#[test]
fn glr_lib_error_conflict_resolution_display() {
    let err = GLRError::ConflictResolution("ambiguous at state 5".into());
    let msg = format!("{err}");
    assert!(msg.contains("Conflict resolution failed"), "got: {msg}");
}

#[test]
fn glr_lib_error_state_machine_display() {
    let err = GLRError::StateMachine("overflow".into());
    let msg = format!("{err}");
    assert!(msg.contains("State machine"), "got: {msg}");
}

#[test]
fn glr_lib_error_complex_symbols_display() {
    let err = GLRError::ComplexSymbolsNotNormalized {
        operation: "FIRST set".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("normalized"), "got: {msg}");
    assert!(msg.contains("FIRST set"), "got: {msg}");
}

#[test]
fn glr_lib_error_expected_simple_symbol_display() {
    let err = GLRError::ExpectedSimpleSymbol {
        expected: "Terminal".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("Terminal"), "got: {msg}");
}

// ════════════════════════════════════════════════════════════════════
// 6. Parse failures with various invalid inputs
// ════════════════════════════════════════════════════════════════════

#[test]
fn driver_parse_tokens_empty_input_fails() {
    // A table that only has Shift(1) on SymbolId(0) in state 0 — no Accept anywhere
    let table = make_table(
        vec![
            vec![vec![Action::Shift(StateId(1))], vec![]],
            vec![vec![], vec![]],
        ],
        vec![vec![StateId(u16::MAX); 2]; 2],
        vec![],
        SymbolId(1),
        SymbolId(0),
    );
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(std::iter::empty());
    assert!(result.is_err(), "empty token stream should fail");
}

#[test]
fn driver_parse_tokens_unknown_symbol_fails() {
    let table = make_table(
        vec![
            vec![vec![Action::Shift(StateId(1))], vec![]],
            vec![vec![], vec![]],
        ],
        vec![vec![StateId(u16::MAX); 2]; 2],
        vec![],
        SymbolId(1),
        SymbolId(0),
    );
    let mut driver = Driver::new(&table);
    // Feed a symbol (99) that doesn't exist in the table
    let result = driver.parse_tokens(vec![(99, 0, 1)]);
    assert!(result.is_err(), "unknown symbol should cause parse error");
}

#[test]
fn driver_parse_tokens_error_action_fails() {
    // State 0 has Error on SymbolId(0)
    let table = make_table(
        vec![vec![vec![Action::Error], vec![]]],
        vec![vec![StateId(u16::MAX); 2]],
        vec![],
        SymbolId(1),
        SymbolId(0),
    );
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(vec![(0, 0, 1)]);
    assert!(result.is_err());
}

// ════════════════════════════════════════════════════════════════════
// 7. Empty grammars
// ════════════════════════════════════════════════════════════════════

#[test]
fn empty_grammar_first_follow_succeeds_with_empty_sets() {
    let grammar = Grammar::new("empty".into());
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    assert!(ff.first(SymbolId(0)).is_none());
    assert!(ff.follow(SymbolId(0)).is_none());
}

#[test]
fn empty_grammar_build_automaton_fails() {
    let grammar = Grammar::new("empty".into());
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_err(), "empty grammar must fail automaton build");
}

#[test]
fn empty_grammar_is_nullable_returns_false() {
    let grammar = Grammar::new("empty".into());
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    assert!(!ff.is_nullable(SymbolId(0)));
    assert!(!ff.is_nullable(SymbolId(99)));
}

// ════════════════════════════════════════════════════════════════════
// 8. FIRST/FOLLOW edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn first_follow_single_terminal_rule() {
    let grammar = simple_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let s = SymbolId(10);
    let a = SymbolId(1);
    let first_s = ff.first(s).expect("FIRST(S) should exist");
    assert!(
        first_s.contains(a.0 as usize),
        "FIRST(S) should contain 'a'"
    );
    assert!(!ff.is_nullable(s), "S should not be nullable");
}

#[test]
fn first_follow_epsilon_rule_is_nullable() {
    let mut g = Grammar::new("eps".into());
    let s = SymbolId(10);
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(s), "S → ε should be nullable");
}

#[test]
fn first_follow_undefined_nonterminal_no_crash() {
    // S → B, but B has no rules
    let mut g = Grammar::new("undef".into());
    let s = SymbolId(10);
    let b = SymbolId(20);
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok(), "should not crash on undefined NT");
}

#[test]
fn first_follow_left_recursive_grammar() {
    // E → E '+' a | a
    let mut g = Grammar::new("leftrec".into());
    let a = SymbolId(1);
    let plus = SymbolId(2);
    let e = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(plus),
                    Symbol::Terminal(a),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_e = ff.first(e).expect("FIRST(E) should exist");
    assert!(
        first_e.contains(a.0 as usize),
        "FIRST(E) should contain 'a'"
    );
    assert!(!ff.is_nullable(e));
}

#[test]
fn first_of_sequence_with_complex_symbol_errors() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let complex = vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))];
    let result = ff.first_of_sequence(&complex);
    assert!(result.is_err(), "complex symbols should be rejected");
    if let Err(GLRError::ComplexSymbolsNotNormalized { .. }) = result {
        // expected
    } else {
        panic!("expected ComplexSymbolsNotNormalized error");
    }
}

// ════════════════════════════════════════════════════════════════════
// 9. ParseTable edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn actions_returns_correct_cell_content() {
    let table = make_table(
        vec![vec![
            vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
            vec![Action::Accept],
        ]],
        vec![vec![StateId(u16::MAX); 2]],
        vec![],
        SymbolId(1),
        SymbolId(0),
    );
    let cell = table.actions(StateId(0), SymbolId(0));
    assert_eq!(cell.len(), 2);
    assert!(matches!(cell[0], Action::Shift(StateId(1))));
    assert!(matches!(cell[1], Action::Reduce(RuleId(0))));
}

#[test]
fn goto_returns_none_for_sentinel_value() {
    // goto table has u16::MAX meaning "no transition"
    let mut table = empty_parse_table();
    table.goto_table = vec![vec![StateId(u16::MAX)]];
    table.nonterminal_to_index.insert(SymbolId(5), 0);
    assert!(table.goto(StateId(0), SymbolId(5)).is_none());
}

#[test]
fn is_terminal_boundary_check() {
    let mut table = empty_parse_table();
    table.token_count = 3;
    table.external_token_count = 2;
    assert!(table.is_terminal(SymbolId(0)));
    assert!(table.is_terminal(SymbolId(4)));
    assert!(!table.is_terminal(SymbolId(5)));
    assert!(!table.is_terminal(SymbolId(100)));
}

#[test]
fn valid_symbols_with_valid_state() {
    let table = make_table(
        vec![vec![
            vec![Action::Shift(StateId(1))],
            vec![],
            vec![Action::Accept],
        ]],
        vec![vec![StateId(u16::MAX); 3]],
        vec![],
        SymbolId(2),
        SymbolId(0),
    );
    let mask = table.valid_symbols(StateId(0));
    // token_count=2, external_token_count=0, so boundary is 2
    assert_eq!(mask.len(), 2);
    assert!(mask[0]); // column 0 has Shift
    assert!(!mask[1]); // column 1 is empty
}

// ════════════════════════════════════════════════════════════════════
// 10. Action variant coverage
// ════════════════════════════════════════════════════════════════════

#[test]
fn action_error_variant_debug() {
    let a = Action::Error;
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Error"));
}

#[test]
fn action_recover_variant_debug() {
    let a = Action::Recover;
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Recover"));
}

#[test]
fn action_fork_variant_holds_nested_actions() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))]);
    if let Action::Fork(ref inner) = fork {
        assert_eq!(inner.len(), 2);
    } else {
        panic!("expected Fork variant");
    }
}

#[test]
fn action_equality() {
    assert_eq!(Action::Accept, Action::Accept);
    assert_eq!(Action::Error, Action::Error);
    assert_eq!(Action::Shift(StateId(1)), Action::Shift(StateId(1)));
    assert_ne!(Action::Shift(StateId(1)), Action::Shift(StateId(2)));
    assert_ne!(Action::Shift(StateId(1)), Action::Reduce(RuleId(1)));
}

// ════════════════════════════════════════════════════════════════════
// 11. build_lr1_automaton error paths
// ════════════════════════════════════════════════════════════════════

#[test]
fn build_automaton_no_start_symbol_errors() {
    // Grammar has rules but no start symbol (tokens without rule names → no start)
    let mut g = Grammar::new("nostart".into());
    let a = SymbolId(1);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    // No rules → no start symbol
    let ff = FirstFollowSets::compute(&g).unwrap();
    let result = build_lr1_automaton(&g, &ff);
    assert!(result.is_err(), "grammar without start symbol should fail");
}

#[test]
fn build_automaton_simple_grammar_succeeds() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let result = build_lr1_automaton(&g, &ff);
    assert!(result.is_ok(), "simple S→a grammar should succeed");
    let table = result.unwrap();
    assert!(table.state_count > 0);
}
