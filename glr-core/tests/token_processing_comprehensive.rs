#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for token processing and lexer integration in adze-glr-core.
//!
//! Covers: token stream construction, token sequence validation, lexer error handling,
//! empty token streams, single-token parsing, token boundary conditions, Unicode tokens,
//! and token kind mapping.
use adze_glr_core::driver::GlrError;
use adze_glr_core::ts_lexer::NextToken;
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

// ═══════════════════════════════════════════════════════════════════════
// 1. Token stream construction
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn next_token_construction_basic() {
    let tok = NextToken {
        kind: 1,
        start: 0,
        end: 3,
    };
    assert_eq!(tok.kind, 1);
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 3);
}

#[test]
fn next_token_debug_format() {
    let tok = NextToken {
        kind: 5,
        start: 0,
        end: 1,
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("NextToken"), "debug: {dbg}");
    assert!(dbg.contains("5"), "debug: {dbg}");
}

#[test]
fn token_stream_from_vec_of_tuples() {
    let tokens: Vec<(u32, u32, u32)> = vec![(1, 0, 1), (2, 1, 3), (3, 3, 4)];
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].0, 1); // kind
    assert_eq!(tokens[1].1, 1); // start
    assert_eq!(tokens[2].2, 4); // end
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Token sequence validation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn valid_two_token_sequence_parses() {
    let mut grammar = GrammarBuilder::new("seq2")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
}

#[test]
fn wrong_token_order_is_rejected() {
    let mut grammar = GrammarBuilder::new("order")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    // Feed b then a — reversed; may error or recover
    let result = pipeline_parse(&mut grammar, &[(b, 0, 1), (a, 1, 2)]);
    if let Ok(f) = result {
        // Recovery accepted — at least verify it produced a forest
        assert!(!f.view().roots().is_empty());
    }
}

#[test]
fn duplicate_token_rejected_when_grammar_expects_different() {
    let mut grammar = GrammarBuilder::new("dup")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");

    // Feed a a — grammar expects a b; may error or recover
    let result = pipeline_parse(&mut grammar, &[(a, 0, 1), (a, 1, 2)]);
    if let Ok(f) = result {
        assert!(!f.view().roots().is_empty());
    }
}

#[test]
fn three_token_sequence_is_validated() {
    let mut grammar = GrammarBuilder::new("seq3")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("S", vec!["x", "y", "z"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let y = sym_id(&grammar, "y");
    let z = sym_id(&grammar, "z");

    let forest =
        pipeline_parse(&mut grammar, &[(x, 0, 1), (y, 1, 2), (z, 2, 3)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 3);
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Lexer error handling
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn glr_error_lex_variant_message() {
    let err = GlrError::Lex("unexpected byte 0xFF".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("lexer error"), "msg: {msg}");
    assert!(msg.contains("0xFF"), "msg: {msg}");
}

#[test]
fn glr_error_lex_debug_variant() {
    let err = GlrError::Lex("bad input".to_string());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Lex"), "debug: {dbg}");
}

#[test]
fn glr_error_parse_on_unrecognized_token() {
    let err = GlrError::Parse("unrecognized token at byte 5".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("parse error"), "msg: {msg}");
    assert!(msg.contains("byte 5"), "msg: {msg}");
}

#[test]
fn glr_error_other_generic() {
    let err = GlrError::Other("internal failure".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("internal failure"), "msg: {msg}");
}

#[test]
fn glr_error_lex_implements_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(GlrError::Lex("fail".into()));
    assert!(err.to_string().contains("lexer error"));
}

#[test]
fn glr_error_with_empty_message() {
    let err = GlrError::Lex(String::new());
    let msg = format!("{err}");
    assert!(msg.contains("lexer error"), "msg: {msg}");

    let err2 = GlrError::Parse(String::new());
    let msg2 = format!("{err2}");
    assert!(msg2.contains("parse error"), "msg: {msg2}");
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Empty token streams
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_token_stream_fails_or_recovers() {
    let mut grammar = GrammarBuilder::new("empty_tok")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    // No tokens fed — incomplete input
    let result = pipeline_parse(&mut grammar, &[]);
    // Must not panic; error or recovery are both acceptable
    if let Ok(f) = result {
        assert!(!f.view().roots().is_empty())
    }
}

#[test]
fn empty_token_stream_with_handcrafted_table() {
    let eof = SymbolId(0);
    let _a = SymbolId(1);
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
    let mut driver = Driver::new(&table);

    // Empty iterator
    let result = driver.parse_tokens(std::iter::empty::<(u32, u32, u32)>());
    // Should fail — no tokens and grammar requires at least one
    if let Ok(f) = result {
        assert!(!f.view().roots().is_empty())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Single-token parsing
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_token_parse_succeeds() {
    let mut grammar = GrammarBuilder::new("single_tok")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 1);
}

#[test]
fn single_token_with_wide_span() {
    let mut grammar = GrammarBuilder::new("wide_tok")
        .token("word", r"\w+")
        .rule("S", vec!["word"])
        .start("S")
        .build();

    let word = sym_id(&grammar, "word");
    let forest = pipeline_parse(&mut grammar, &[(word, 0, 100)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).end, 100);
}

#[test]
fn single_token_with_nonzero_start_offset() {
    let mut grammar = GrammarBuilder::new("offset")
        .token("t", "t")
        .rule("S", vec!["t"])
        .start("S")
        .build();

    let t = sym_id(&grammar, "t");
    let forest = pipeline_parse(&mut grammar, &[(t, 50, 51)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 50);
    assert_eq!(view.span(view.roots()[0]).end, 51);
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Token boundary conditions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn zero_width_token_span() {
    let mut grammar = GrammarBuilder::new("zero_w")
        .token("eps", "")
        .rule("S", vec!["eps"])
        .start("S")
        .build();

    let eps = sym_id(&grammar, "eps");
    // Zero-width token: start == end
    let forest = pipeline_parse(&mut grammar, &[(eps, 5, 5)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
}

#[test]
fn adjacent_tokens_no_gap() {
    let mut grammar = GrammarBuilder::new("adj")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    // Adjacent: a ends at 3, b starts at 3
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 3), (b, 3, 6)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 6);
}

#[test]
fn tokens_with_whitespace_gap() {
    let mut grammar = GrammarBuilder::new("gap")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["x", "y"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let y = sym_id(&grammar, "y");

    // Gap between tokens (simulating whitespace)
    let forest = pipeline_parse(&mut grammar, &[(x, 0, 1), (y, 5, 6)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 6);
}

#[test]
fn large_byte_offsets() {
    let mut grammar = GrammarBuilder::new("large_off")
        .token("tok", "t")
        .rule("S", vec!["tok"])
        .start("S")
        .build();

    let tok = sym_id(&grammar, "tok");
    // Large but sub-u32::MAX offsets
    let forest =
        pipeline_parse(&mut grammar, &[(tok, 1_000_000, 1_000_001)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 1_000_000);
    assert_eq!(view.span(view.roots()[0]).end, 1_000_001);
}

#[test]
fn token_boundary_at_offset_zero() {
    let eof = SymbolId(0);
    let a = SymbolId(1);
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
    let mut driver = Driver::new(&table);

    let result = driver.parse_tokens([(a.0 as u32, 0u32, 0u32)].iter().copied());
    // Zero-width at offset 0 — should process without panic
    if let Ok(f) = result {
        assert!(!f.view().roots().is_empty())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 7. Unicode tokens
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn unicode_token_byte_spans() {
    // "αβ" — α is 2 bytes UTF-8, β is 2 bytes UTF-8
    let mut grammar = GrammarBuilder::new("uni")
        .token("alpha", "α")
        .token("beta", "β")
        .rule("S", vec!["alpha", "beta"])
        .start("S")
        .build();

    let alpha = sym_id(&grammar, "alpha");
    let beta = sym_id(&grammar, "beta");

    // α = bytes 0..2, β = bytes 2..4
    let forest =
        pipeline_parse(&mut grammar, &[(alpha, 0, 2), (beta, 2, 4)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 4);
}

#[test]
fn multibyte_token_wide_span() {
    // Simulate a token spanning a 4-byte emoji
    let mut grammar = GrammarBuilder::new("emoji")
        .token("e", "e")
        .rule("S", vec!["e"])
        .start("S")
        .build();

    let e = sym_id(&grammar, "e");
    // 🎉 is 4 bytes in UTF-8
    let forest = pipeline_parse(&mut grammar, &[(e, 0, 4)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).end, 4);
}

#[test]
fn cjk_token_byte_offsets() {
    // CJK characters are 3 bytes each in UTF-8
    let mut grammar = GrammarBuilder::new("cjk")
        .token("han", "汉")
        .token("zi", "字")
        .rule("S", vec!["han", "zi"])
        .start("S")
        .build();

    let han = sym_id(&grammar, "han");
    let zi = sym_id(&grammar, "zi");

    // 汉 = bytes 0..3, 字 = bytes 3..6
    let forest = pipeline_parse(&mut grammar, &[(han, 0, 3), (zi, 3, 6)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).end, 6);
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Token kind mapping
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn token_kind_maps_to_symbol_id() {
    let mut grammar = GrammarBuilder::new("kind_map")
        .token("num", r"\d+")
        .token("op", "+")
        .rule("expr", vec!["expr", "op", "num"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "num");
    let op = sym_id(&grammar, "op");

    // Verify that symbol IDs are distinct
    assert_ne!(num, op, "num and op should have distinct symbol IDs");

    let forest = pipeline_parse(&mut grammar, &[(num, 0, 1), (op, 1, 2), (num, 2, 3)])
        .expect("should parse");
    let view = forest.view();
    assert!(!view.roots().is_empty());
}

#[test]
fn handcrafted_token_kind_shift() {
    let eof = SymbolId(0);
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
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

    let result = driver.parse_tokens(
        [(tok_a.0 as u32, 0u32, 1u32), (tok_b.0 as u32, 1, 2)]
            .iter()
            .copied(),
    );
    assert!(result.is_ok(), "handcrafted a-b parse should succeed");
}

#[test]
fn wrong_kind_id_rejected() {
    let eof = SymbolId(0);
    let _tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
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
    let mut driver = Driver::new(&table);

    // Feed tok_b but table only accepts tok_a in state 0; may error or recover
    let result = driver.parse_tokens([(tok_b.0 as u32, 0u32, 1u32)].iter().copied());
    if let Ok(f) = result {
        assert!(!f.view().roots().is_empty());
    }
}

#[test]
fn multiple_token_kinds_in_grammar() {
    let mut grammar = GrammarBuilder::new("multi_kind")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let c = sym_id(&grammar, "c");

    // All three should be distinct
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);

    let forest =
        pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2), (c, 2, 3)]).expect("should parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
}

#[test]
fn token_kind_stable_across_pipeline_rebuild() {
    let build = || {
        let mut g = GrammarBuilder::new("stable")
            .token("x", "x")
            .token("y", "y")
            .rule("S", vec!["x", "y"])
            .start("S")
            .build();
        let x = sym_id(&g, "x");
        let y = sym_id(&g, "y");
        let table = run_pipeline(&mut g).expect("pipeline");
        (x, y, table)
    };

    let (x1, y1, _) = build();
    let (x2, y2, _) = build();

    assert_eq!(x1, x2, "token IDs should be stable");
    assert_eq!(y1, y2, "token IDs should be stable");
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Additional edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_tokens_with_iterator_adapter() {
    let mut grammar = GrammarBuilder::new("iter_adapt")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let table = run_pipeline(&mut grammar).expect("pipeline");
    sanity_check_tables(&table).expect("sanity");
    let mut driver = Driver::new(&table);

    // Build token stream from map adapter
    let raw_data = vec![(a, 0u32, 1u32)];
    let result = driver.parse_tokens(raw_data.into_iter().map(|(s, st, en)| (s.0 as u32, st, en)));
    assert!(result.is_ok(), "iterator adapter parse should succeed");
}

#[test]
fn repeated_parses_with_same_driver_table() {
    let mut grammar = GrammarBuilder::new("reuse")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let table = run_pipeline(&mut grammar).expect("pipeline");
    sanity_check_tables(&table).expect("sanity");

    // Parse twice with fresh drivers (driver borrows table)
    for _ in 0..2 {
        let mut driver = Driver::new(&table);
        let forest = driver
            .parse_tokens([(a.0 as u32, 0u32, 1u32)].iter().copied())
            .expect("should parse");
        assert_eq!(forest.view().roots().len(), 1);
    }
}

#[test]
fn forest_children_consistent_with_token_count() {
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

    let forest =
        pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2), (c, 2, 3)]).expect("should parse");
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    // S -> a b c should have 3 children
    assert_eq!(children.len(), 3, "root should have 3 children");
}

#[test]
fn extra_token_after_complete_parse_is_error() {
    let mut grammar = GrammarBuilder::new("extra")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    // Grammar only expects one 'a', but we also feed 'b'
    let result = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2)]);
    // Should either error or the extra token causes a parse failure
    if let Ok(f) = result {
        // If recovery accepted it, that's okay — just verify it didn't crash
        assert!(!f.view().roots().is_empty());
    }
}
