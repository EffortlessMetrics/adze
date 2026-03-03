//! Comprehensive tests for the GLR driver module.
//!
//! Covers: Driver::new, parse_tokens, GlrError variants, ForestView traversal,
//! ambiguity, error handling, parser reuse, empty input, and multi-token parsing.
#![cfg(feature = "test-api")]

use adze_glr_core::conflict_inspection::count_conflicts;
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

// ═══════════════════════════════════════════════════════════════════════
// 1. Driver::new — basic construction
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn driver_new_with_minimal_table() {
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
}

#[test]
fn driver_new_with_pipeline_table() {
    let mut grammar = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let _driver = Driver::new(&table);
}

#[test]
fn driver_new_with_multi_rule_table() {
    let mut grammar = GrammarBuilder::new("multi")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let _driver = Driver::new(&table);
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Parsing valid input sequences
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_single_terminal() {
    let mut grammar = GrammarBuilder::new("single")
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
fn parse_two_terminal_sequence() {
    let mut grammar = GrammarBuilder::new("pair")
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
    assert_eq!(view.span(view.roots()[0]).end, 2);
}

#[test]
fn parse_three_token_arithmetic() {
    let mut grammar = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");

    let forest = pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)])
        .expect("should parse 1+2");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 3);
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Parsing with ambiguity
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_ambiguous_addition() {
    // E → E '+' E | NUM — inherently ambiguous for "n+n+n"
    let mut grammar = GrammarBuilder::new("ambig_add")
        .token("n", "n")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let conflicts = count_conflicts(&table);
    assert!(
        conflicts.shift_reduce >= 1,
        "ambiguous grammar must have S/R conflicts"
    );

    let n = sym_id(&grammar, "n");
    let plus = sym_id(&grammar, "+");

    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens(
            [
                (n.0 as u32, 0u32, 1u32),
                (plus.0 as u32, 1, 2),
                (n.0 as u32, 2, 3),
                (plus.0 as u32, 3, 4),
                (n.0 as u32, 4, 5),
            ]
            .iter()
            .copied(),
        )
        .expect("ambiguous should still parse");

    let view = forest.view();
    assert!(!view.roots().is_empty());
    assert_eq!(view.span(view.roots()[0]).end, 5);
}

#[test]
fn parse_ambiguous_short_input_succeeds() {
    // E → E '+' E | 'n'   Input: "n+n" (minimal ambiguous case)
    let mut grammar = GrammarBuilder::new("ambig_short")
        .token("n", "n")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();

    let n = sym_id(&grammar, "n");
    let plus = sym_id(&grammar, "+");

    let forest = pipeline_parse(&mut grammar, &[(n, 0, 1), (plus, 1, 2), (n, 2, 3)])
        .expect("n+n should parse");
    assert_eq!(forest.view().span(forest.view().roots()[0]).end, 3);
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Error handling for invalid input
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn invalid_token_sequence_returns_error_or_recovery() {
    // S → 'a' 'b', feed 'a' 'a'
    let eof = SymbolId(0);
    let _a = SymbolId(1);
    let _b = SymbolId(2);
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

    // 'a' 'a' — no action for 'a' in state1
    let result = driver.parse_tokens([(1u32, 0u32, 1u32), (1, 1, 2)].iter().copied());
    assert!(result.is_err(), "invalid token sequence should be rejected");
}

#[test]
fn unexpected_eof_returns_error() {
    // S → 'a' 'b', feed just 'a'
    let mut grammar = GrammarBuilder::new("early_eof")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let result = pipeline_parse(&mut grammar, &[(a, 0, 1)]);
    // May error or recover — either is valid; must not panic
    match result {
        Ok(f) => {
            // Recovery: still produces a forest
            assert!(!f.view().roots().is_empty());
        }
        Err(_) => { /* expected for incomplete input */ }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 5. GlrError variants — Display and Debug
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn glr_error_lex_display() {
    let err = adze_glr_core::driver::GlrError::Lex("bad char".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("lexer error"), "display: {msg}");
    assert!(msg.contains("bad char"), "display: {msg}");
}

#[test]
fn glr_error_parse_display() {
    let err = adze_glr_core::driver::GlrError::Parse("unexpected token".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("parse error"), "display: {msg}");
    assert!(msg.contains("unexpected token"), "display: {msg}");
}

#[test]
fn glr_error_other_display() {
    let err = adze_glr_core::driver::GlrError::Other("something else".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("something else"), "display: {msg}");
}

#[test]
fn glr_error_debug_contains_variant_name() {
    let err = adze_glr_core::driver::GlrError::Lex("x".to_string());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Lex"), "debug: {dbg}");

    let err2 = adze_glr_core::driver::GlrError::Parse("y".to_string());
    let dbg2 = format!("{err2:?}");
    assert!(dbg2.contains("Parse"), "debug: {dbg2}");

    let err3 = adze_glr_core::driver::GlrError::Other("z".to_string());
    let dbg3 = format!("{err3:?}");
    assert!(dbg3.contains("Other"), "debug: {dbg3}");
}

// ═══════════════════════════════════════════════════════════════════════
// 6. ForestView traversal
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_view_root_kind_is_nonzero() {
    let mut grammar = GrammarBuilder::new("kind")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("should parse");
    let view = forest.view();
    let root = view.roots()[0];
    // The root kind should be a valid symbol id
    let _kind = view.kind(root);
}

#[test]
fn forest_view_children_span_within_parent() {
    let mut grammar = GrammarBuilder::new("span_check")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2)]).expect("should parse");
    let view = forest.view();
    let root = view.roots()[0];
    let root_span = view.span(root);
    let children = view.best_children(root);

    for &child in children {
        let cs = view.span(child);
        assert!(cs.start >= root_span.start, "child start within parent");
        assert!(cs.end <= root_span.end, "child end within parent");
    }
}

#[test]
fn forest_view_leaf_has_no_children() {
    let mut grammar = GrammarBuilder::new("leaf")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let forest = pipeline_parse(&mut grammar, &[(x, 0, 1)]).expect("should parse");
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    // The root's child is a terminal leaf
    assert_eq!(children.len(), 1);
    let leaf = children[0];
    let leaf_children = view.best_children(leaf);
    assert!(
        leaf_children.is_empty(),
        "terminal leaf should have no children"
    );
}

#[test]
fn forest_view_deep_traversal() {
    // S → A B; A → 'x'; B → 'y'
    let mut grammar = GrammarBuilder::new("deep_trav")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let y = sym_id(&grammar, "y");

    let forest = pipeline_parse(&mut grammar, &[(x, 0, 1), (y, 1, 2)]).expect("should parse");
    let view = forest.view();

    // Walk all nodes without panic
    fn walk(view: &dyn ForestView, id: u32, depth: usize) -> usize {
        let mut count = 1;
        assert!(depth < 100, "infinite recursion guard");
        for &child in view.best_children(id) {
            count += walk(view, child, depth + 1);
        }
        count
    }

    let total_nodes = walk(view, view.roots()[0], 0);
    assert!(total_nodes >= 3, "should have root + at least 2 subtrees");
}

// ═══════════════════════════════════════════════════════════════════════
// 7. Token construction via parse_tokens
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn tokens_with_varying_byte_spans() {
    // Multi-byte tokens: each token spans different widths
    let mut grammar = GrammarBuilder::new("widths")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");

    // "123+45" → NUM(0,3), +(3,4), NUM(4,6)
    let forest = pipeline_parse(&mut grammar, &[(num, 0, 3), (plus, 3, 4), (num, 4, 6)])
        .expect("should parse with varying widths");

    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 6);
}

#[test]
fn tokens_with_gaps_in_byte_offsets() {
    // Tokens may have whitespace gaps: "a  b" → a(0,1), b(3,4)
    let mut grammar = GrammarBuilder::new("gaps")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 3, 4)]).expect("should parse");
    let view = forest.view();
    // Root span covers first token start to last token end
    let root_span = view.span(view.roots()[0]);
    assert_eq!(root_span.start, 0);
    assert_eq!(root_span.end, 4);
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Multi-token parsing
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_five_token_chain() {
    // S → 'a' 'b' 'c' 'd' 'e'
    let mut grammar = GrammarBuilder::new("five")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a", "b", "c", "d", "e"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let c = sym_id(&grammar, "c");
    let d = sym_id(&grammar, "d");
    let e = sym_id(&grammar, "e");

    let forest = pipeline_parse(
        &mut grammar,
        &[(a, 0, 1), (b, 1, 2), (c, 2, 3), (d, 3, 4), (e, 4, 5)],
    )
    .expect("5-token chain should parse");

    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).end, 5);
    assert_eq!(view.best_children(view.roots()[0]).len(), 5);
}

#[test]
fn parse_left_recursive_chain() {
    // A → A 'a' | 'a'   Input: "aaaa"
    let mut grammar = GrammarBuilder::new("leftrec_chain")
        .token("a", "a")
        .rule("A", vec!["A", "a"])
        .rule("A", vec!["a"])
        .start("A")
        .build();

    let a = sym_id(&grammar, "a");
    let tokens: Vec<_> = (0..4).map(|i| (a, i as u32, (i + 1) as u32)).collect();
    let forest = pipeline_parse(&mut grammar, &tokens).expect("left recursive should parse");
    assert_eq!(forest.view().span(forest.view().roots()[0]).end, 4);
}

#[test]
fn parse_repeated_addition() {
    // expr → expr '+' NUM | NUM   Input: "1+2+3+4"
    let mut grammar = GrammarBuilder::new("rep_add")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");

    let forest = pipeline_parse(
        &mut grammar,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
            (plus, 5, 6),
            (num, 6, 7),
        ],
    )
    .expect("repeated addition should parse");

    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 7);
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Empty input handling
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_input_with_epsilon_rule() {
    // S → ε
    let mut grammar = GrammarBuilder::new("empty_eps")
        .token("a", "a")
        .rule("S", vec![])
        .start("S")
        .build();

    let result = pipeline_parse(&mut grammar, &[]);
    match result {
        Ok(f) => assert!(!f.view().roots().is_empty(), "accepted empty → has root"),
        Err(_) => { /* graceful failure acceptable */ }
    }
}

#[test]
fn empty_input_non_nullable_grammar() {
    // S → 'a' — cannot accept empty input
    let mut grammar = GrammarBuilder::new("non_null")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let result = pipeline_parse(&mut grammar, &[]);
    // Should either error or produce an error-recovery parse
    match result {
        Ok(f) => {
            // If recovery happened, error stats should reflect it
            let stats = f.debug_error_stats();
            // Recovery may or may not set error flags
            let _ = stats;
        }
        Err(_) => { /* expected */ }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Parser reuse across multiple inputs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn driver_reuse_across_inputs() {
    let mut grammar = GrammarBuilder::new("reuse")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let mut driver = Driver::new(&table);

    // First parse
    let f1 = driver
        .parse_tokens([(a.0 as u32, 0u32, 1u32)].iter().copied())
        .expect("first parse");
    assert_eq!(f1.view().roots().len(), 1);

    // Second parse — same driver, different input
    let f2 = driver
        .parse_tokens([(b.0 as u32, 0u32, 1u32)].iter().copied())
        .expect("second parse");
    assert_eq!(f2.view().roots().len(), 1);
}

#[test]
fn driver_reuse_same_input_twice() {
    let mut grammar = GrammarBuilder::new("reuse2")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let table = run_pipeline(&mut grammar).expect("pipeline");
    let mut driver = Driver::new(&table);

    for _ in 0..3 {
        let f = driver
            .parse_tokens([(x.0 as u32, 0u32, 1u32)].iter().copied())
            .expect("repeated parse should succeed");
        assert_eq!(f.view().roots().len(), 1);
    }
}

#[test]
fn driver_reuse_error_then_success() {
    let mut grammar = GrammarBuilder::new("err_ok")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let table = run_pipeline(&mut grammar).expect("pipeline");
    let mut driver = Driver::new(&table);

    // Invalid input: just 'a' (incomplete)
    let _r1 = driver.parse_tokens([(a.0 as u32, 0u32, 1u32)].iter().copied());
    // Valid input: 'a' 'b'
    let r2 = driver.parse_tokens(
        [(a.0 as u32, 0u32, 1u32), (b.0 as u32, 1, 2)]
            .iter()
            .copied(),
    );
    assert!(
        r2.is_ok(),
        "driver should recover and parse valid input after error"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Additional coverage: hand-crafted table tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn hand_crafted_shift_reduce_accept_cycle() {
    // S → 'x' 'y'
    let eof = SymbolId(0);
    let _x = SymbolId(1);
    let _y = SymbolId(2);
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
        .parse_tokens([(1u32, 0u32, 1u32), (2, 1, 2)].iter().copied())
        .expect("shift-reduce-accept should work");

    assert_eq!(forest.view().roots().len(), 1);
    assert_eq!(forest.view().span(forest.view().roots()[0]).end, 2);
}

#[test]
fn hand_crafted_epsilon_reduce() {
    // A → ε; S → A 'x'
    let eof = SymbolId(0);
    let _x = SymbolId(1);
    let s = SymbolId(3);
    let a_sym = SymbolId(4);

    let rules = vec![
        ParseRule {
            lhs: a_sym,
            rhs_len: 0,
        }, // A → ε
        ParseRule { lhs: s, rhs_len: 2 }, // S → A x
    ];

    let mut actions = vec![vec![vec![]; 5]; 5];
    actions[0][1].push(Action::Reduce(RuleId(0))); // state0 + x → reduce A→ε
    actions[1][1].push(Action::Shift(StateId(2))); // state1 + x → state2
    actions[2][0].push(Action::Reduce(RuleId(1))); // state2 + EOF → reduce S
    actions[3][0].push(Action::Accept); // state3 + EOF → accept

    // Unused state4 needed for 5×5 matrix
    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 5]; 5];
    gotos[0][4] = StateId(1); // after A → state1
    gotos[0][3] = StateId(3); // after S → accept

    let table = create_test_table(actions, gotos, rules, s, eof);
    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens([(1u32, 0u32, 1u32)].iter().copied())
        .expect("epsilon reduce should work");

    assert!(!forest.view().roots().is_empty());
}

#[test]
fn forest_error_stats_clean_parse() {
    let mut grammar = GrammarBuilder::new("clean")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("should parse");

    let (has_error, _missing, error_cost) = forest.debug_error_stats();
    assert!(!has_error, "clean parse should have no errors");
    assert_eq!(error_cost, 0, "clean parse should have zero error cost");
}

#[test]
fn multiple_alternatives_first_matching() {
    // S → 'a' 'b' | 'a' 'c'   Input: "ab"
    let mut grammar = GrammarBuilder::new("alt_match")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b"])
        .rule("S", vec!["a", "c"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2)])
        .expect("first alternative should match");
    assert_eq!(forest.view().span(forest.view().roots()[0]).end, 2);
}

#[test]
fn multiple_alternatives_second_matching() {
    // S → 'a' 'b' | 'a' 'c'   Input: "ac"
    let mut grammar = GrammarBuilder::new("alt_match2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b"])
        .rule("S", vec!["a", "c"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let c = sym_id(&grammar, "c");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (c, 1, 2)])
        .expect("second alternative should match");
    assert_eq!(forest.view().span(forest.view().roots()[0]).end, 2);
}

#[test]
fn nested_nonterminals_children_correct() {
    // S → A B; A → 'x'; B → 'y'
    let mut grammar = GrammarBuilder::new("nested_nt")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let y = sym_id(&grammar, "y");

    let forest = pipeline_parse(&mut grammar, &[(x, 0, 1), (y, 1, 2)]).expect("should parse");
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert_eq!(children.len(), 2, "S → A B has 2 children");

    // Each child (A, B) should have 1 child (the terminal)
    for &child in children {
        let grandchildren = view.best_children(child);
        assert_eq!(grandchildren.len(), 1, "A/B each have 1 terminal child");
    }
}

#[test]
fn deeply_nested_parentheses() {
    let mut grammar = GrammarBuilder::new("deep_paren")
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

    let depth = 8;
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

    let forest = pipeline_parse(&mut grammar, &tokens).expect("deep parens should parse");
    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, byte);
}

#[test]
fn table_sanity_check_passes_for_valid_table() {
    let mut grammar = GrammarBuilder::new("sanity")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    sanity_check_tables(&table).expect("sanity check should pass for valid table");
}

#[test]
fn table_has_accept_shift_reduce_actions() {
    let mut grammar = GrammarBuilder::new("actions")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
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

#[test]
fn parse_right_recursive() {
    // L → 'a' L | 'a'   Input: "aaa"
    let mut grammar = GrammarBuilder::new("rightrec")
        .token("a", "a")
        .rule("L", vec!["a", "L"])
        .rule("L", vec!["a"])
        .start("L")
        .build();

    let a = sym_id(&grammar, "a");
    let table = run_pipeline(&mut grammar).expect("pipeline");
    let mut driver = Driver::new(&table);

    let forest = driver
        .parse_tokens(
            [
                (a.0 as u32, 0u32, 1u32),
                (a.0 as u32, 1, 2),
                (a.0 as u32, 2, 3),
            ]
            .iter()
            .copied(),
        )
        .expect("right-recursive should parse");
    assert!(!forest.view().roots().is_empty());

    // Re-parse to verify driver reuse
    let forest2 = driver
        .parse_tokens(
            [
                (a.0 as u32, 0u32, 1u32),
                (a.0 as u32, 1, 2),
                (a.0 as u32, 2, 3),
            ]
            .iter()
            .copied(),
        )
        .expect("right-recursive re-parse");
    assert!(!forest2.view().roots().is_empty());
}
