//! Comprehensive integration tests for the GLR Driver in adze-glr-core.
//!
//! Covers: construction, token parsing, error handling, multi-path (ambiguity),
//! forest output structure, large input handling, and empty input parsing.
#![cfg(feature = "test-api")]
#![allow(clippy::needless_range_loop)]

use adze_glr_core::driver::GlrError;
use adze_glr_core::forest_view::ForestView;
use adze_glr_core::{
    Action, Driver, FirstFollowSets, Forest, GotoIndexing, LexMode, ParseRule, ParseTable,
    build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ─── Helpers ─────────────────────────────────────────────────────────

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

/// Run normalize → FIRST/FOLLOW → build_lr1_automaton, returning a ParseTable.
fn run_pipeline(grammar: &mut Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("LR1 automaton")
}

/// Build grammar + table, then parse a token stream through the driver.
fn pipeline_parse(
    grammar: &mut Grammar,
    tokens: &[(SymbolId, u32, u32)],
) -> Result<Forest, GlrError> {
    let table = run_pipeline(grammar);
    sanity_check_tables(&table).expect("sanity");
    let mut driver = Driver::new(&table);
    driver.parse_tokens(
        tokens
            .iter()
            .map(|&(sym, start, end)| (sym.0 as u32, start, end)),
    )
}

type ActionCell = Vec<Action>;

/// Hand-craft a minimal ParseTable for low-level driver tests.
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

// ═══════════════════════════════════════════════════════════════════════
// 1. Driver construction from parse table
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn construct_driver_from_pipeline_table() {
    let mut grammar = GrammarBuilder::new("ctr")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = run_pipeline(&mut grammar);
    // Driver::new should not panic
    let _driver = Driver::new(&table);
}

#[test]
fn construct_driver_from_hand_crafted_table() {
    let eof = SymbolId(0);
    let t = SymbolId(1);
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
}

#[test]
fn driver_reuses_table_initial_state() {
    let mut grammar = GrammarBuilder::new("init")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let table = run_pipeline(&mut grammar);
    assert!(
        (table.initial_state.0 as usize) < table.state_count,
        "initial_state must be valid"
    );
    let _driver = Driver::new(&table);
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Parsing simple token sequences
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_single_terminal() {
    let mut grammar = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("single token");
    assert_eq!(f.view().roots().len(), 1);
}

#[test]
fn parse_two_token_sequence() {
    let mut grammar = GrammarBuilder::new("two")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["x", "y"])
        .start("S")
        .build();
    let x = sym_id(&grammar, "x");
    let y = sym_id(&grammar, "y");
    let f = pipeline_parse(&mut grammar, &[(x, 0, 1), (y, 1, 2)]).expect("two tokens");
    let v = f.view();
    assert_eq!(v.roots().len(), 1);
    assert_eq!(v.span(v.roots()[0]).end, 2);
}

#[test]
fn parse_three_token_arithmetic() {
    let mut grammar = GrammarBuilder::new("arith3")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");
    let f = pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("1+2");
    let v = f.view();
    assert_eq!(v.span(v.roots()[0]).start, 0);
    assert_eq!(v.span(v.roots()[0]).end, 3);
}

#[test]
fn parse_five_token_chain() {
    let mut grammar = GrammarBuilder::new("chain5")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");
    let f = pipeline_parse(
        &mut grammar,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
        ],
    )
    .expect("1+2+3");
    assert_eq!(f.view().span(f.view().roots()[0]).end, 5);
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Error handling for invalid input
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn error_on_unexpected_token() {
    // S → 'a' 'b', feed 'a' 'a'
    let eof = SymbolId(0);
    let a = SymbolId(1);
    let b = SymbolId(2);
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
    let result = driver.parse_tokens([(1u32, 0, 1), (1u32, 1, 2)].iter().copied());
    assert!(result.is_err(), "unexpected token must error");
}

#[test]
fn error_message_contains_byte_offset() {
    let eof = SymbolId(0);
    let a = SymbolId(1);
    let b = SymbolId(2);
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
    let err = match driver.parse_tokens([(1u32, 0, 1), (1u32, 1, 2)].iter().copied()) {
        Err(e) => e,
        Ok(_) => panic!("expected error for invalid input"),
    };
    let msg = err.to_string();
    // Error should mention byte position or state info
    assert!(
        msg.contains("byte") || msg.contains("state") || msg.contains("parse"),
        "error message should be informative: {msg}"
    );
}

#[test]
fn glr_error_lex_variant() {
    let e = GlrError::Lex("bad token".into());
    assert!(e.to_string().contains("bad token"));
}

#[test]
fn glr_error_parse_variant() {
    let e = GlrError::Parse("unexpected".into());
    assert!(e.to_string().contains("unexpected"));
}

#[test]
fn glr_error_other_variant() {
    let e = GlrError::Other("misc".into());
    assert!(e.to_string().contains("misc"));
}

#[test]
fn glr_error_is_std_error() {
    let e: Box<dyn std::error::Error> = Box::new(GlrError::Other("x".into()));
    assert!(!e.to_string().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Multi-path parsing (ambiguity)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn ambiguous_expr_finds_parse() {
    // E → E '+' E | 'n'
    let mut grammar = GrammarBuilder::new("ambig_e")
        .token("n", "n")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let n = sym_id(&grammar, "n");
    let plus = sym_id(&grammar, "+");
    let table = run_pipeline(&mut grammar);
    let mut driver = Driver::new(&table);
    let f = driver
        .parse_tokens(
            [
                (n.0 as u32, 0u32, 1),
                (plus.0 as u32, 1, 2),
                (n.0 as u32, 2, 3),
                (plus.0 as u32, 3, 4),
                (n.0 as u32, 4, 5),
            ]
            .iter()
            .copied(),
        )
        .expect("ambiguous n+n+n");
    assert!(!f.view().roots().is_empty());
}

#[test]
fn ambiguous_grammar_has_conflicts() {
    let mut grammar = GrammarBuilder::new("ambig_c")
        .token("n", "n")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let table = run_pipeline(&mut grammar);
    let conflicts = adze_glr_core::conflict_inspection::count_conflicts(&table);
    assert!(
        conflicts.shift_reduce >= 1,
        "ambiguous grammar should have S/R conflicts"
    );
}

#[test]
fn fork_one_branch_survives() {
    // S → 'a' 'b' | 'a' 'c'   Input: "ac"
    let mut grammar = GrammarBuilder::new("fork_s")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b"])
        .rule("S", vec!["a", "c"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let c = sym_id(&grammar, "c");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1), (c, 1, 2)]).expect("ac");
    assert_eq!(f.view().roots().len(), 1);
    assert_eq!(f.view().span(f.view().roots()[0]).end, 2);
}

#[test]
fn both_alternatives_parseable_independently() {
    let mut grammar = GrammarBuilder::new("alts2")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let f1 = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("a");
    let f2 = pipeline_parse(&mut grammar, &[(b, 0, 1)]).expect("b");
    assert_eq!(f1.view().roots().len(), 1);
    assert_eq!(f2.view().roots().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Forest output structure
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_root_span_covers_full_input() {
    let mut grammar = GrammarBuilder::new("span_full")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let c = sym_id(&grammar, "c");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2), (c, 2, 3)]).expect("abc");
    let v = f.view();
    let root = v.roots()[0];
    assert_eq!(v.span(root).start, 0);
    assert_eq!(v.span(root).end, 3);
}

#[test]
fn forest_children_count_matches_rhs() {
    // S → 'a' 'b' 'c' → 3 children
    let mut grammar = GrammarBuilder::new("child_cnt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let c = sym_id(&grammar, "c");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2), (c, 2, 3)]).expect("abc");
    let v = f.view();
    let children = v.best_children(v.roots()[0]);
    assert_eq!(children.len(), 3);
}

#[test]
fn forest_child_spans_are_contiguous() {
    let mut grammar = GrammarBuilder::new("contig")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["x", "y"])
        .start("S")
        .build();
    let x = sym_id(&grammar, "x");
    let y = sym_id(&grammar, "y");
    let f = pipeline_parse(&mut grammar, &[(x, 0, 3), (y, 3, 7)]).expect("xy");
    let v = f.view();
    let ch = v.best_children(v.roots()[0]);
    assert_eq!(v.span(ch[0]).start, 0);
    assert_eq!(v.span(ch[0]).end, 3);
    assert_eq!(v.span(ch[1]).start, 3);
    assert_eq!(v.span(ch[1]).end, 7);
}

#[test]
fn forest_nested_rule_has_intermediate_node() {
    // S → A 'b'; A → 'a'   → root has children [A_node, b_leaf]
    let mut grammar = GrammarBuilder::new("nested")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["A", "b"])
        .rule("A", vec!["a"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2)]).expect("ab");
    let v = f.view();
    let root_children = v.best_children(v.roots()[0]);
    assert_eq!(root_children.len(), 2, "S → A b has 2 children");
    // First child (A) should itself have children
    let a_children = v.best_children(root_children[0]);
    assert_eq!(a_children.len(), 1, "A → a has 1 child");
}

#[test]
fn forest_error_stats_clean_parse() {
    let mut grammar = GrammarBuilder::new("clean")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("clean");
    let (has_err, _missing, cost) = f.debug_error_stats();
    assert!(!has_err, "clean parse has no errors");
    assert_eq!(cost, 0, "clean parse has zero cost");
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Large input handling
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_100_token_chain() {
    let mut grammar = GrammarBuilder::new("big")
        .token("a", "a")
        .rule("L", vec!["L", "a"])
        .rule("L", vec!["a"])
        .start("L")
        .build();
    let a = sym_id(&grammar, "a");
    let tokens: Vec<_> = (0..100u32).map(|i| (a, i, i + 1)).collect();
    let f = pipeline_parse(&mut grammar, &tokens).expect("100 tokens");
    assert_eq!(f.view().span(f.view().roots()[0]).end, 100);
}

#[test]
fn parse_500_token_chain() {
    let mut grammar = GrammarBuilder::new("big500")
        .token("a", "a")
        .rule("L", vec!["L", "a"])
        .rule("L", vec!["a"])
        .start("L")
        .build();
    let a = sym_id(&grammar, "a");
    let tokens: Vec<_> = (0..500u32).map(|i| (a, i, i + 1)).collect();
    let f = pipeline_parse(&mut grammar, &tokens).expect("500 tokens");
    let v = f.view();
    assert!(!v.roots().is_empty());
    assert_eq!(v.span(v.roots()[0]).end, 500);
}

#[test]
fn parse_deep_nesting_20_levels() {
    let mut grammar = GrammarBuilder::new("deep20")
        .token("NUM", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let lp = sym_id(&grammar, "(");
    let rp = sym_id(&grammar, ")");
    let depth = 20u32;
    let mut tokens = Vec::new();
    let mut byte = 0u32;
    for _ in 0..depth {
        tokens.push((lp, byte, byte + 1));
        byte += 1;
    }
    tokens.push((num, byte, byte + 1));
    byte += 1;
    for _ in 0..depth {
        tokens.push((rp, byte, byte + 1));
        byte += 1;
    }
    let f = pipeline_parse(&mut grammar, &tokens).expect("20-deep nesting");
    assert_eq!(f.view().span(f.view().roots()[0]).end, byte);
}

// ═══════════════════════════════════════════════════════════════════════
// 7. Empty input parsing
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_token_stream_no_panic() {
    let mut grammar = GrammarBuilder::new("emp")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    // Empty stream for a non-nullable grammar → should not panic
    let result = pipeline_parse(&mut grammar, &[]);
    // Either error or (unlikely) recovery; no panic is the key assertion.
    let _ = result;
}

#[test]
fn epsilon_grammar_empty_input() {
    let mut grammar = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("S", vec![])
        .start("S")
        .build();
    let result = pipeline_parse(&mut grammar, &[]);
    // Epsilon grammar may or may not accept; no panic.
    match result {
        Ok(f) => assert!(!f.view().roots().is_empty()),
        Err(_) => { /* graceful error is acceptable */ }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Additional coverage
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn reduction_chain_multiple_levels() {
    // A → 'a'; B → A; C → B; S → C
    let mut grammar = GrammarBuilder::new("rchain")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("B", vec!["A"])
        .rule("C", vec!["B"])
        .rule("S", vec!["C"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("chain");
    assert_eq!(f.view().roots().len(), 1);
}

#[test]
fn left_recursive_grammar_parses() {
    let mut grammar = GrammarBuilder::new("lrec")
        .token("a", "a")
        .rule("A", vec!["A", "a"])
        .rule("A", vec!["a"])
        .start("A")
        .build();
    let a = sym_id(&grammar, "a");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 1), (a, 1, 2), (a, 2, 3)]).expect("aaa");
    assert_eq!(f.view().span(f.view().roots()[0]).end, 3);
}

#[test]
fn right_recursive_grammar_parses() {
    let mut grammar = GrammarBuilder::new("rrec")
        .token("a", "a")
        .rule("L", vec!["a", "L"])
        .rule("L", vec!["a"])
        .start("L")
        .build();
    let a = sym_id(&grammar, "a");
    let table = run_pipeline(&mut grammar);
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(a.0 as u32, 0u32, 1), (a.0 as u32, 1, 2)].iter().copied());
    assert!(result.is_ok(), "right-recursive: {:?}", result.err());
}

#[test]
fn span_tracking_non_unit_byte_widths() {
    // Tokens with multi-byte spans (e.g., 3-byte tokens)
    let mut grammar = GrammarBuilder::new("wide")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let f = pipeline_parse(&mut grammar, &[(a, 0, 10), (b, 10, 25)]).expect("wide spans");
    let v = f.view();
    assert_eq!(v.span(v.roots()[0]).start, 0);
    assert_eq!(v.span(v.roots()[0]).end, 25);
}

#[test]
fn table_contains_accept_shift_reduce() {
    let mut grammar = GrammarBuilder::new("acts")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = run_pipeline(&mut grammar);
    let has_accept = table
        .action_table
        .iter()
        .flat_map(|r| r.iter())
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)));
    let has_shift = table
        .action_table
        .iter()
        .flat_map(|r| r.iter())
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))));
    let has_reduce = table
        .action_table
        .iter()
        .flat_map(|r| r.iter())
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))));
    assert!(has_accept, "table needs Accept");
    assert!(has_shift, "table needs Shift");
    assert!(has_reduce, "table needs Reduce");
}

#[test]
fn eof_symbol_in_symbol_to_index() {
    let mut grammar = GrammarBuilder::new("eof_map")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = run_pipeline(&mut grammar);
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF must be in symbol_to_index"
    );
}

#[test]
fn hand_crafted_shift_reduce_accept_cycle() {
    // S → 'a' 'b'
    let eof = SymbolId(0);
    let a = SymbolId(1);
    let b = SymbolId(2);
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
    let f = driver
        .parse_tokens([(1u32, 0, 1), (2u32, 1, 2)].iter().copied())
        .expect("shift-reduce-accept");
    assert_eq!(f.view().roots().len(), 1);
    assert_eq!(f.view().span(f.view().roots()[0]).end, 2);
}
