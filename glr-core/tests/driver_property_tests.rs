//! Property tests for the GLR driver.
//!
//! Covers: Driver initialization, step-by-step parsing, fork/merge behavior,
//! accept/reject results, valid-token-sequence properties, and edge cases
//! (empty input, single token, repeated tokens).
//!
//! Run with: cargo test -p adze-glr-core --test driver_property_tests
#![allow(clippy::needless_range_loop)]
#![cfg(feature = "test-api")]

use adze_glr_core::forest_view::ForestView;
use adze_glr_core::{
    Action, Driver, FirstFollowSets, Forest, GotoIndexing, LexMode, ParseRule, ParseTable,
    build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ─── Helpers ─────────────────────────────────────────────────────────

type ActionCell = Vec<Action>;

const NO_GOTO: StateId = StateId(65535);

/// Build a minimal `ParseTable` from raw action/goto matrices.
fn build_table(
    actions: Vec<Vec<ActionCell>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start: SymbolId,
    eof: SymbolId,
    num_terminals: usize,
) -> ParseTable {
    let symbol_count = actions.first().map(|r| r.len()).unwrap_or(0);
    let state_count = actions.len();
    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }
    let mut nonterminal_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        for row in &gotos {
            if i < row.len() && row[i] != NO_GOTO {
                nonterminal_to_index.insert(SymbolId(i as u16), i);
                break;
            }
        }
    }
    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: rules.clone(),
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start,
        grammar: Grammar::new("prop".to_string()),
        symbol_metadata: vec![],
        initial_state: StateId(0),
        token_count: num_terminals,
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

/// Run normalize → FIRST/FOLLOW → build_lr1_automaton.
fn run_pipeline(grammar: &mut Grammar) -> Result<ParseTable, adze_glr_core::GLRError> {
    let ff = FirstFollowSets::compute_normalized(grammar)?;
    build_lr1_automaton(grammar, &ff)
}

/// Build grammar + table, then parse a token stream.
fn pipeline_parse(
    grammar: &mut Grammar,
    tokens: &[(SymbolId, u32, u32)],
) -> Result<Forest, adze_glr_core::driver::GlrError> {
    let table = run_pipeline(grammar).expect("pipeline should produce a table");
    sanity_check_tables(&table).expect("table sanity check");
    let mut driver = Driver::new(&table);
    driver.parse_tokens(tokens.iter().map(|&(s, a, b)| (s.0 as u32, a, b)))
}

/// Resolve a symbol name to its SymbolId.
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
    panic!("symbol '{name}' not found");
}

// ─── Canonical hand-crafted tables ──────────────────────────────────

/// S -> 'a'  (symbols: 0=EOF, 1='a', 2=S)
fn table_s_to_a() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(2);
    let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
    let mut a = vec![vec![vec![]; 3]; 3];
    a[0][1].push(Action::Shift(StateId(1)));
    a[1][0].push(Action::Reduce(RuleId(0)));
    a[2][0].push(Action::Accept);
    let mut g = vec![vec![NO_GOTO; 3]; 3];
    g[0][2] = StateId(2);
    build_table(a, g, rules, s, eof, 2)
}

/// S -> ε  (symbols: 0=EOF, 1=S)
fn table_epsilon() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(1);
    let rules = vec![ParseRule { lhs: s, rhs_len: 0 }];
    let mut a = vec![vec![vec![]; 2]; 2];
    a[0][0].push(Action::Reduce(RuleId(0)));
    a[1][0].push(Action::Accept);
    let mut g = vec![vec![NO_GOTO; 2]; 2];
    g[0][1] = StateId(1);
    build_table(a, g, rules, s, eof, 1)
}

/// Right-recursive:  S -> A;  A -> 'a' | 'a' A
/// Symbols: 0=EOF, 1='a', 2=S, 3=A
fn table_right_rec() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(2);
    let a_nt = SymbolId(3);
    let rules = vec![
        ParseRule { lhs: s, rhs_len: 1 },
        ParseRule {
            lhs: a_nt,
            rhs_len: 1,
        },
        ParseRule {
            lhs: a_nt,
            rhs_len: 2,
        },
    ];
    let ns = 4;
    let nst = 5;
    let mut act = vec![vec![vec![]; ns]; nst];
    act[0][1].push(Action::Shift(StateId(1)));
    act[1][1].push(Action::Shift(StateId(1)));
    act[1][0].push(Action::Reduce(RuleId(1)));
    act[2][0].push(Action::Reduce(RuleId(0)));
    act[3][0].push(Action::Accept);
    act[4][0].push(Action::Reduce(RuleId(2)));
    let mut go = vec![vec![NO_GOTO; ns]; nst];
    go[0][3] = StateId(2);
    go[0][2] = StateId(3);
    go[1][3] = StateId(4);
    build_table(act, go, rules, s, eof, 2)
}

/// S -> 'a' 'b'  (symbols: 0=EOF, 1='a', 2='b', 3=S)
fn table_a_b() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(3);
    let rules = vec![ParseRule { lhs: s, rhs_len: 2 }];
    let mut a = vec![vec![vec![]; 4]; 4];
    a[0][1].push(Action::Shift(StateId(1)));
    a[1][2].push(Action::Shift(StateId(2)));
    a[2][0].push(Action::Reduce(RuleId(0)));
    a[3][0].push(Action::Accept);
    let mut g = vec![vec![NO_GOTO; 4]; 4];
    g[0][3] = StateId(3);
    build_table(a, g, rules, s, eof, 3)
}

// ═══════════════════════════════════════════════════════════════════════
// Section 1 — Driver initialization
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn init_minimal_handcrafted_table() {
    let table = table_s_to_a();
    let _d = Driver::new(&table);
}

#[test]
fn init_epsilon_table() {
    let table = table_epsilon();
    let _d = Driver::new(&table);
}

#[test]
fn init_right_recursive_table() {
    let table = table_right_rec();
    let _d = Driver::new(&table);
}

#[test]
fn init_two_terminal_table() {
    let table = table_a_b();
    let _d = Driver::new(&table);
}

#[test]
fn init_pipeline_single_token_grammar() {
    let mut g = GrammarBuilder::new("s1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let t = run_pipeline(&mut g).unwrap();
    let _d = Driver::new(&t);
}

#[test]
fn init_pipeline_multi_rule_grammar() {
    let mut g = GrammarBuilder::new("m1")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["p", "q"])
        .rule("p", vec!["a"])
        .rule("q", vec!["b"])
        .start("s")
        .build();
    let t = run_pipeline(&mut g).unwrap();
    let _d = Driver::new(&t);
}

#[test]
fn init_pipeline_left_recursive_grammar() {
    let mut g = GrammarBuilder::new("lr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t = run_pipeline(&mut g).unwrap();
    let _d = Driver::new(&t);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn init_with_varying_terminal_id(tid in 1u16..8) {
        let eof = SymbolId(0);
        let s_id = tid.max(2) + 1;
        let s = SymbolId(s_id);
        let ns = (s_id as usize) + 1;
        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
        let mut act = vec![vec![vec![]; ns]; 3];
        act[0][tid as usize].push(Action::Shift(StateId(1)));
        act[1][0].push(Action::Reduce(RuleId(0)));
        act[2][0].push(Action::Accept);
        let mut go = vec![vec![NO_GOTO; ns]; 3];
        go[0][s_id as usize] = StateId(2);
        let tbl = build_table(act, go, rules, s, eof, (tid as usize) + 1);
        let _d = Driver::new(&tbl);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Section 2 — Step-by-step parsing through tokens
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_single_token_handcrafted() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let r = d.parse_tokens(vec![(1u32, 0, 1)].into_iter());
    assert!(r.is_ok());
    let f = r.unwrap();
    assert_eq!(f.view().roots().len(), 1);
}

#[test]
fn parse_two_tokens_handcrafted() {
    let table = table_a_b();
    let mut d = Driver::new(&table);
    let r = d.parse_tokens(vec![(1, 0, 1), (2, 1, 2)].into_iter());
    assert!(r.is_ok());
    let f = r.unwrap();
    let v = f.view();
    assert_eq!(v.roots().len(), 1);
    assert_eq!(v.span(v.roots()[0]).start, 0);
    assert_eq!(v.span(v.roots()[0]).end, 2);
}

#[test]
fn parse_single_token_pipeline() {
    let mut g = GrammarBuilder::new("t1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let a = sym_id(&g, "a");
    let f = pipeline_parse(&mut g, &[(a, 0, 1)]).unwrap();
    assert_eq!(f.view().roots().len(), 1);
}

#[test]
fn parse_three_token_sequence_pipeline() {
    let mut g = GrammarBuilder::new("t3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let a = sym_id(&g, "a");
    let b = sym_id(&g, "b");
    let c = sym_id(&g, "c");
    let f = pipeline_parse(&mut g, &[(a, 0, 1), (b, 1, 2), (c, 2, 3)]).unwrap();
    let v = f.view();
    assert_eq!(v.span(v.roots()[0]).start, 0);
    assert_eq!(v.span(v.roots()[0]).end, 3);
}

#[test]
fn parse_arithmetic_chain() {
    let mut g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&g, "NUM");
    let plus = sym_id(&g, "+");
    // 1+2+3
    let f = pipeline_parse(
        &mut g,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
        ],
    )
    .unwrap();
    let v = f.view();
    assert_eq!(v.span(v.roots()[0]).start, 0);
    assert_eq!(v.span(v.roots()[0]).end, 5);
}

#[test]
fn parse_wide_byte_spans() {
    let mut g = GrammarBuilder::new("wide")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&g, "NUM");
    let plus = sym_id(&g, "+");
    // "123+45" → NUM(0,3) +(3,4) NUM(4,6)
    let f = pipeline_parse(&mut g, &[(num, 0, 3), (plus, 3, 4), (num, 4, 6)]).unwrap();
    let v = f.view();
    assert_eq!(v.span(v.roots()[0]).end, 6);
}

#[test]
fn parse_tokens_with_gaps() {
    let mut g = GrammarBuilder::new("gap")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let a = sym_id(&g, "a");
    let b = sym_id(&g, "b");
    // whitespace gap: "a  b"
    let f = pipeline_parse(&mut g, &[(a, 0, 1), (b, 3, 4)]).unwrap();
    let v = f.view();
    assert_eq!(v.span(v.roots()[0]).start, 0);
    assert_eq!(v.span(v.roots()[0]).end, 4);
}

// ═══════════════════════════════════════════════════════════════════════
// Section 3 — Fork / merge behavior
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn fork_action_does_not_panic() {
    // Hand-craft a table with an explicit Fork action
    let eof = SymbolId(0);
    let s = SymbolId(2);
    let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
    let mut act = vec![vec![vec![]; 3]; 3];
    // Fork: two shifts to different states, but both lead to reduce→accept path
    act[0][1].push(Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(1)),
    ]));
    act[1][0].push(Action::Reduce(RuleId(0)));
    act[2][0].push(Action::Accept);
    let mut go = vec![vec![NO_GOTO; 3]; 3];
    go[0][2] = StateId(2);
    let tbl = build_table(act, go, rules, s, eof, 2);
    let mut d = Driver::new(&tbl);
    let r = d.parse_tokens(vec![(1, 0, 1)].into_iter());
    assert!(r.is_ok(), "Fork action should not cause panic");
}

#[test]
fn ambiguous_grammar_produces_result() {
    // E -> E + E | NUM  (inherently ambiguous)
    let mut g = GrammarBuilder::new("amb")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&g, "NUM");
    let plus = sym_id(&g, "+");
    let f = pipeline_parse(
        &mut g,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
        ],
    )
    .unwrap();
    let v = f.view();
    assert!(!v.roots().is_empty());
}

#[test]
fn shift_reduce_conflict_grammar_parses() {
    // Classic dangling-else style: inherent shift/reduce conflict
    let mut g = GrammarBuilder::new("sr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&g, "NUM");
    let plus = sym_id(&g, "+");
    let star = sym_id(&g, "*");
    // 1 + 2 * 3
    let r = pipeline_parse(
        &mut g,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (star, 3, 4),
            (num, 4, 5),
        ],
    );
    assert!(
        r.is_ok(),
        "shift/reduce conflict grammar should still parse"
    );
}

#[test]
fn fork_with_reduce_action() {
    // Fork that contains a Reduce action — driver must not panic.
    // The table may or may not accept depending on how Fork(Reduce) interacts with EOF.
    let eof = SymbolId(0);
    let s = SymbolId(2);
    let rules = vec![
        ParseRule { lhs: s, rhs_len: 1 }, // rule 0: S -> 'a'
    ];
    let mut act = vec![vec![vec![]; 3]; 3];
    act[0][1].push(Action::Shift(StateId(1)));
    act[1][0].push(Action::Fork(vec![Action::Reduce(RuleId(0))]));
    act[2][0].push(Action::Accept);
    let mut go = vec![vec![NO_GOTO; 3]; 3];
    go[0][2] = StateId(2);
    let tbl = build_table(act, go, rules, s, eof, 2);
    let mut d = Driver::new(&tbl);
    // Must not panic — result may be Ok or Err depending on driver internals
    let _ = d.parse_tokens(vec![(1, 0, 1)].into_iter());
}

// ═══════════════════════════════════════════════════════════════════════
// Section 4 — Accept / reject results
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn accept_valid_input() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    assert!(d.parse_tokens(vec![(1, 0, 1)].into_iter()).is_ok());
}

#[test]
fn reject_completely_wrong_token() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    // symbol 99 has no entry
    let r = d.parse_tokens(vec![(99, 0, 1)].into_iter());
    // Must not panic; should be Err or recovered
    let _ = r;
}

#[test]
fn reject_extra_token_after_complete_parse() {
    let mut g = GrammarBuilder::new("ex")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let a = sym_id(&g, "a");
    // Feed two 'a' tokens to a grammar that expects exactly one
    let r = pipeline_parse(&mut g, &[(a, 0, 1), (a, 1, 2)]);
    // Either error or early accept — must not panic
    let _ = r;
}

#[test]
fn reject_wrong_sequence() {
    let table = table_a_b();
    let mut d = Driver::new(&table);
    // Expects 'a' 'b' but gets 'b' 'a'
    let r = d.parse_tokens(vec![(2, 0, 1), (1, 1, 2)].into_iter());
    // Should not panic
    let _ = r;
}

#[test]
fn accept_empty_on_epsilon() {
    let table = table_epsilon();
    let mut d = Driver::new(&table);
    let r = d.parse_tokens(std::iter::empty());
    assert!(r.is_ok());
    let v = r.unwrap();
    assert!(!v.view().roots().is_empty());
}

#[test]
fn reject_nonempty_on_epsilon() {
    let table = table_epsilon();
    let mut d = Driver::new(&table);
    let r = d.parse_tokens(vec![(1, 0, 1)].into_iter());
    // S->ε has no shift for terminal 1; must terminate
    let _ = r;
}

#[test]
fn accept_right_recursive_1_token() {
    let table = table_right_rec();
    let mut d = Driver::new(&table);
    let r = d.parse_tokens(vec![(1, 0, 1)].into_iter());
    assert!(r.is_ok());
}

#[test]
fn accept_right_recursive_3_tokens() {
    let table = table_right_rec();
    let mut d = Driver::new(&table);
    let r = d.parse_tokens(vec![(1, 0, 1), (1, 1, 2), (1, 2, 3)].into_iter());
    assert!(r.is_ok());
    let v = r.unwrap();
    assert_eq!(v.view().span(v.view().roots()[0]).end, 3);
}

#[test]
fn forest_root_kind_is_start_symbol() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let f = d.parse_tokens(vec![(1, 0, 1)].into_iter()).unwrap();
    let v = f.view();
    // root kind should be S (SymbolId 2)
    assert_eq!(v.kind(v.roots()[0]), 2);
}

#[test]
fn forest_root_has_children() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let f = d.parse_tokens(vec![(1, 0, 1)].into_iter()).unwrap();
    let v = f.view();
    let kids = v.best_children(v.roots()[0]);
    // S -> 'a' has exactly 1 child
    assert_eq!(kids.len(), 1);
}

#[test]
fn forest_child_is_terminal() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let f = d.parse_tokens(vec![(1, 0, 1)].into_iter()).unwrap();
    let v = f.view();
    let kids = v.best_children(v.roots()[0]);
    // Child should be 'a' (SymbolId 1)
    assert_eq!(v.kind(kids[0]), 1);
    assert_eq!(v.span(kids[0]).start, 0);
    assert_eq!(v.span(kids[0]).end, 1);
}

#[test]
fn two_terminal_rule_has_two_children() {
    let table = table_a_b();
    let mut d = Driver::new(&table);
    let f = d
        .parse_tokens(vec![(1, 0, 1), (2, 1, 2)].into_iter())
        .unwrap();
    let v = f.view();
    let kids = v.best_children(v.roots()[0]);
    assert_eq!(kids.len(), 2);
    assert_eq!(v.kind(kids[0]), 1);
    assert_eq!(v.kind(kids[1]), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// Section 5 — Property tests: valid token sequences always succeed
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Right-recursive grammar accepts any positive number of 'a' tokens.
    #[test]
    fn right_rec_accepts_n_tokens(n in 1usize..=8) {
        let table = table_right_rec();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut d = Driver::new(&table);
        let r = d.parse_tokens(tokens.into_iter());
        prop_assert!(r.is_ok(), "right-rec should accept {} 'a's: {:?}", n, r.err());
        let f = r.unwrap();
        let v = f.view();
        prop_assert!(!v.roots().is_empty());
        prop_assert_eq!(v.span(v.roots()[0]).start, 0);
        prop_assert_eq!(v.span(v.roots()[0]).end, n as u32);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Parsing always terminates regardless of input length.
    #[test]
    fn parsing_always_terminates(n in 0usize..=10) {
        let table = table_s_to_a();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut d = Driver::new(&table);
        let _ = d.parse_tokens(tokens.into_iter());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Root span always covers all input (start=0, end=input_len) on success.
    #[test]
    fn root_span_covers_full_input(n in 1usize..=6) {
        let table = table_right_rec();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut d = Driver::new(&table);
        let f = d.parse_tokens(tokens.into_iter()).unwrap();
        let v = f.view();
        for &root in v.roots() {
            let sp = v.span(root);
            prop_assert_eq!(sp.start, 0);
            prop_assert_eq!(sp.end, n as u32);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Deterministic grammar always yields exactly one root.
    #[test]
    fn deterministic_one_root(_seed in 0u32..100) {
        let table = table_s_to_a();
        let mut d = Driver::new(&table);
        let f = d.parse_tokens(vec![(1, 0, 1)].into_iter()).unwrap();
        prop_assert_eq!(f.view().roots().len(), 1);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Pipeline-based left-recursive expr grammar accepts chains of NUM (+NUM)*.
    #[test]
    fn left_rec_expr_accepts_chain(adds in 0usize..=4) {
        let mut g = GrammarBuilder::new("lr_prop")
            .token("NUM", r"\d+")
            .token("+", "+")
            .rule("expr", vec!["expr", "+", "NUM"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let num = sym_id(&g, "NUM");
        let plus = sym_id(&g, "+");
        // Build token stream: NUM (+ NUM)*
        let mut toks = vec![(num, 0u32, 1u32)];
        let mut pos = 1u32;
        for _ in 0..adds {
            toks.push((plus, pos, pos + 1));
            pos += 1;
            toks.push((num, pos, pos + 1));
            pos += 1;
        }
        let f = pipeline_parse(&mut g, &toks);
        prop_assert!(f.is_ok(), "expr chain with {} additions failed: {:?}", adds, f.err());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Shift+reduce with varying terminal IDs always accepts one token.
    #[test]
    fn shift_reduce_variable_tid(tid in 1u16..6) {
        let eof = SymbolId(0);
        let s_id = tid.max(2) + 1;
        let s = SymbolId(s_id);
        let ns = (s_id as usize) + 1;
        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
        let mut act = vec![vec![vec![]; ns]; 3];
        act[0][tid as usize].push(Action::Shift(StateId(1)));
        act[1][0].push(Action::Reduce(RuleId(0)));
        act[2][0].push(Action::Accept);
        let mut go = vec![vec![NO_GOTO; ns]; 3];
        go[0][s_id as usize] = StateId(2);
        let tbl = build_table(act, go, rules, s, eof, (tid as usize) + 1);
        let mut d = Driver::new(&tbl);
        let r = d.parse_tokens(vec![(tid as u32, 0, 1)].into_iter());
        prop_assert!(r.is_ok());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Multiple sequential parses on different drivers sharing the same table give identical results.
    #[test]
    fn multiple_drivers_same_table(n in 1usize..=5) {
        let table = table_s_to_a();
        for _ in 0..n {
            let mut d = Driver::new(&table);
            let r = d.parse_tokens(vec![(1, 0, 1)].into_iter());
            prop_assert!(r.is_ok());
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Pipeline two-nonterminal grammar: S -> A B, A -> 'x', B -> 'y'.
    #[test]
    fn two_nonterminal_pipeline_accepts(_seed in 0u32..50) {
        let mut g = GrammarBuilder::new("2nt")
            .token("x", "x")
            .token("y", "y")
            .rule("s", vec!["p", "q"])
            .rule("p", vec!["x"])
            .rule("q", vec!["y"])
            .start("s")
            .build();
        let x = sym_id(&g, "x");
        let y = sym_id(&g, "y");
        let f = pipeline_parse(&mut g, &[(x, 0, 1), (y, 1, 2)]);
        prop_assert!(f.is_ok(), "two-NT grammar failed: {:?}", f.err());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Section 6 — Edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_input_on_non_epsilon_grammar_terminates() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let _ = d.parse_tokens(std::iter::empty());
}

#[test]
fn single_token_on_two_token_grammar_terminates() {
    let table = table_a_b();
    let mut d = Driver::new(&table);
    let _ = d.parse_tokens(vec![(1, 0, 1)].into_iter());
}

#[test]
fn zero_width_token_does_not_panic() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let _ = d.parse_tokens(vec![(1, 0, 0)].into_iter());
}

#[test]
fn very_large_byte_offset_does_not_panic() {
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let _ = d.parse_tokens(vec![(1, u32::MAX - 1, u32::MAX)].into_iter());
}

#[test]
fn token_kind_zero_does_not_panic() {
    // Kind 0 is typically EOF — driver must not crash
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let _ = d.parse_tokens(vec![(0, 0, 1)].into_iter());
}

#[test]
fn repeated_single_valid_token_terminates() {
    // Feed 20 identical tokens to a grammar expecting just 1
    let table = table_s_to_a();
    let mut d = Driver::new(&table);
    let tokens: Vec<(u32, u32, u32)> = (0..20).map(|i| (1, i, i + 1)).collect();
    let _ = d.parse_tokens(tokens.into_iter());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Repeated 'a' on right-recursive grammar: spans are contiguous.
    #[test]
    fn repeated_tokens_contiguous_spans(n in 1usize..=8) {
        let table = table_right_rec();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut d = Driver::new(&table);
        let f = d.parse_tokens(tokens.into_iter()).unwrap();
        let v = f.view();
        // Root span is exactly [0, n)
        let sp = v.span(v.roots()[0]);
        prop_assert_eq!(sp.start, 0);
        prop_assert_eq!(sp.end, n as u32);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Random invalid symbol IDs never cause panics.
    #[test]
    fn random_invalid_symbol_no_panic(kind in 10u32..200) {
        let table = table_s_to_a();
        let mut d = Driver::new(&table);
        let _ = d.parse_tokens(vec![(kind, 0, 1)].into_iter());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Random sequence of valid and invalid tokens never panics.
    #[test]
    fn random_mixed_tokens_no_panic(len in 0usize..=6) {
        let table = table_s_to_a();
        let mut d = Driver::new(&table);
        let tokens: Vec<(u32, u32, u32)> = (0..len)
            .map(|i| {
                let kind = if i % 2 == 0 { 1 } else { 50 };
                (kind, i as u32, i as u32 + 1)
            })
            .collect();
        let _ = d.parse_tokens(tokens.into_iter());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Section 7 — GlrError variant properties
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn error_lex_display_contains_msg(msg in "[a-z]{1,15}") {
        let e = adze_glr_core::driver::GlrError::Lex(msg.clone());
        let display = format!("{}", e);
        prop_assert!(display.contains(&msg));
    }

    #[test]
    fn error_parse_display_contains_msg(msg in "[a-z]{1,15}") {
        let e = adze_glr_core::driver::GlrError::Parse(msg.clone());
        let display = format!("{}", e);
        prop_assert!(display.contains(&msg));
    }

    #[test]
    fn error_other_display_contains_msg(msg in "[a-z]{1,15}") {
        let e = adze_glr_core::driver::GlrError::Other(msg.clone());
        let display = format!("{}", e);
        prop_assert!(display.contains(&msg));
    }

    #[test]
    fn error_debug_is_nonempty(variant in 0u8..3, msg in "[a-z]{1,10}") {
        let e = match variant {
            0 => adze_glr_core::driver::GlrError::Lex(msg),
            1 => adze_glr_core::driver::GlrError::Parse(msg),
            _ => adze_glr_core::driver::GlrError::Other(msg),
        };
        let debug = format!("{:?}", e);
        prop_assert!(!debug.is_empty());
    }
}

#[test]
fn glr_error_is_std_error() {
    let e: Box<dyn std::error::Error> = Box::new(adze_glr_core::driver::GlrError::Lex("x".into()));
    assert!(!e.to_string().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Section 8 — Forest structure properties
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn epsilon_root_span_is_zero() {
    let table = table_epsilon();
    let mut d = Driver::new(&table);
    let f = d.parse_tokens(std::iter::empty()).unwrap();
    let v = f.view();
    for &r in v.roots() {
        let sp = v.span(r);
        assert_eq!(sp.start, 0);
        assert_eq!(sp.end, 0);
    }
}

#[test]
fn epsilon_root_has_no_children() {
    let table = table_epsilon();
    let mut d = Driver::new(&table);
    let f = d.parse_tokens(std::iter::empty()).unwrap();
    let v = f.view();
    for &r in v.roots() {
        assert!(v.best_children(r).is_empty());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Forest child spans are within the root span.
    #[test]
    fn child_spans_within_root(n in 1usize..=5) {
        let table = table_right_rec();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut d = Driver::new(&table);
        let f = d.parse_tokens(tokens.into_iter()).unwrap();
        let v = f.view();
        let root_sp = v.span(v.roots()[0]);
        fn check_children(v: &dyn ForestView, id: u32, root_start: u32, root_end: u32) {
            for &child in v.best_children(id) {
                let sp = v.span(child);
                assert!(sp.start >= root_start, "child start {} < root start {}", sp.start, root_start);
                assert!(sp.end <= root_end, "child end {} > root end {}", sp.end, root_end);
                check_children(v, child, root_start, root_end);
            }
        }
        check_children(v, v.roots()[0], root_sp.start, root_sp.end);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Section 9 — Pipeline grammar variety
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nested_nonterminals_pipeline() {
    // S -> A; A -> B; B -> 'x'
    let mut g = GrammarBuilder::new("nest")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["x"])
        .start("s")
        .build();
    let x = sym_id(&g, "x");
    let f = pipeline_parse(&mut g, &[(x, 0, 1)]).unwrap();
    assert_eq!(f.view().roots().len(), 1);
}

#[test]
fn optional_via_epsilon_alternative() {
    // S -> A 'x'; A -> 'y' | ε
    let mut g = GrammarBuilder::new("opt")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["a", "x"])
        .rule("a", vec!["y"])
        .rule("a", vec![])
        .start("s")
        .build();
    let x = sym_id(&g, "x");
    let y = sym_id(&g, "y");
    // With 'y':
    let f1 = pipeline_parse(&mut g, &[(y, 0, 1), (x, 1, 2)]).unwrap();
    assert_eq!(f1.view().span(f1.view().roots()[0]).end, 2);
    // Without 'y' (epsilon path): driver may or may not handle this depending on
    // how epsilon productions interact with the parse table — must not panic.
    let _ = pipeline_parse(&mut g, &[(x, 0, 1)]);
}

#[test]
fn pipeline_with_precedence() {
    let mut g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&g, "NUM");
    let plus = sym_id(&g, "+");
    let star = sym_id(&g, "*");
    let f = pipeline_parse(
        &mut g,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (star, 3, 4),
            (num, 4, 5),
        ],
    )
    .unwrap();
    assert!(!f.view().roots().is_empty());
}

#[test]
fn longer_right_hand_side() {
    // S -> 'a' 'b' 'c' 'd'
    let mut g = GrammarBuilder::new("long_rhs")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d"])
        .start("s")
        .build();
    let a = sym_id(&g, "a");
    let b = sym_id(&g, "b");
    let c = sym_id(&g, "c");
    let d = sym_id(&g, "d");
    let f = pipeline_parse(&mut g, &[(a, 0, 1), (b, 1, 2), (c, 2, 3), (d, 3, 4)]).unwrap();
    let v = f.view();
    assert_eq!(v.span(v.roots()[0]).end, 4);
    assert_eq!(v.best_children(v.roots()[0]).len(), 4);
}

#[test]
fn multiple_alternatives_same_nonterminal() {
    // S -> 'a' | 'b' | 'c'
    let mut g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let a = sym_id(&g, "a");
    let b = sym_id(&g, "b");
    let c = sym_id(&g, "c");
    // Each alternative should accept its token
    assert!(pipeline_parse(&mut g, &[(a, 0, 1)]).is_ok());
    assert!(pipeline_parse(&mut g, &[(b, 0, 1)]).is_ok());
    assert!(pipeline_parse(&mut g, &[(c, 0, 1)]).is_ok());
}

// ═══════════════════════════════════════════════════════════════════════
// Section 10 — Stress / boundary
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Moderately long right-recursive input succeeds.
    #[test]
    fn right_rec_moderate_length(n in 1usize..=20) {
        let table = table_right_rec();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut d = Driver::new(&table);
        let r = d.parse_tokens(tokens.into_iter());
        prop_assert!(r.is_ok());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Fresh driver on same table gives identical root count.
    #[test]
    fn driver_determinism(n in 1usize..=5) {
        let table = table_right_rec();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut d1 = Driver::new(&table);
        let f1 = d1.parse_tokens(tokens.clone().into_iter()).unwrap();
        let mut d2 = Driver::new(&table);
        let f2 = d2.parse_tokens(tokens.into_iter()).unwrap();
        prop_assert_eq!(f1.view().roots().len(), f2.view().roots().len());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Pipeline-based left-recursive expr with varying span widths.
    #[test]
    fn varying_span_widths(width in 1u32..=5) {
        let mut g = GrammarBuilder::new("vw")
            .token("NUM", r"\d+")
            .token("+", "+")
            .rule("expr", vec!["expr", "+", "NUM"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let num = sym_id(&g, "NUM");
        let plus = sym_id(&g, "+");
        // NUM(0,w) + NUM(w+1, 2w+1)
        let toks = vec![
            (num, 0, width),
            (plus, width, width + 1),
            (num, width + 1, 2 * width + 1),
        ];
        let f = pipeline_parse(&mut g, &toks);
        prop_assert!(f.is_ok());
        let forest = f.unwrap();
        prop_assert_eq!(forest.view().span(forest.view().roots()[0]).end, 2 * width + 1);
    }
}
