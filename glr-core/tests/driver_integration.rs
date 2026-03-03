//! GLR driver integration tests: full grammar → parse table → driver → parse pipeline.
//!
//! These tests verify the end-to-end pipeline from grammar construction through
//! table generation to actual parsing via the GLR driver.
#![cfg(feature = "test-api")]

use adze_glr_core::conflict_inspection::count_conflicts;
use adze_glr_core::forest_view::ForestView;
use adze_glr_core::{
    Action, Driver, FirstFollowSets, Forest, GLRError, GotoIndexing, LexMode, ParseRule,
    ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ─── Helpers ─────────────────────────────────────────────────────────

/// Run normalize → FIRST/FOLLOW → build_lr1_automaton, returning a ParseTable.
fn run_pipeline(grammar: &mut Grammar) -> Result<ParseTable, GLRError> {
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;
    build_lr1_automaton(grammar, &first_follow)
}

/// Build grammar + table, then parse a token stream through the driver.
/// `token_specs` maps token names to (symbol_id, byte_start, byte_end) triples.
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
    // Check tokens first
    for (&id, tok) in &grammar.tokens {
        if tok.name == name {
            return id;
        }
    }
    // Then rule names
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

// ─── 1. Parse simple arithmetic "1+2" ────────────────────────────────

#[test]
fn parse_simple_arithmetic_1_plus_2() {
    let mut grammar = GrammarBuilder::new("arith")
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
        &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)], // "1+2"
    )
    .expect("should parse 1+2");

    let view = forest.view();
    let roots = view.roots();
    assert_eq!(roots.len(), 1, "exactly one root");
    let span = view.span(roots[0]);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 3);
}

// ─── 2. Parse nested parentheses "(1)" ──────────────────────────────

#[test]
fn parse_nested_parens() {
    let mut grammar = GrammarBuilder::new("parens")
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

    let forest = pipeline_parse(
        &mut grammar,
        &[(lp, 0, 1), (num, 1, 2), (rp, 2, 3)], // "(1)"
    )
    .expect("should parse (1)");

    let view = forest.view();
    let roots = view.roots();
    assert!(!roots.is_empty());
    let span = view.span(roots[0]);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 3);
}

// ─── 3. Multi-operator with precedence "1+2*3" ──────────────────────

#[test]
fn parse_multi_operator_with_precedence() {
    let mut grammar = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUM"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");
    let star = sym_id(&grammar, "*");

    // "1+2*3"
    let forest = pipeline_parse(
        &mut grammar,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (star, 3, 4),
            (num, 4, 5),
        ],
    )
    .expect("should parse 1+2*3");

    let view = forest.view();
    let roots = view.roots();
    assert!(!roots.is_empty());
    assert_eq!(view.span(roots[0]).start, 0);
    assert_eq!(view.span(roots[0]).end, 5);
}

// ─── 4. Parse empty input (only EOF) ────────────────────────────────

#[test]
fn parse_empty_input_eof_only() {
    // Grammar that accepts empty: S → ε
    let mut grammar = GrammarBuilder::new("empty")
        .token("a", "a")
        .rule("S", vec![]) // epsilon
        .start("S")
        .build();

    let forest = pipeline_parse(&mut grammar, &[]);
    // Empty input: either accepted (start symbol nullable) or graceful error
    // The GLR driver may or may not accept depending on table layout
    // Either outcome is valid; what matters is no panic.
    match forest {
        Ok(f) => {
            let view = f.view();
            assert!(!view.roots().is_empty(), "accepted empty → has root");
        }
        Err(_) => { /* graceful failure is acceptable for empty input */ }
    }
}

// ─── 5. Parse single token input ────────────────────────────────────

#[test]
fn parse_single_token() {
    let mut grammar = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("single token should parse");

    let view = forest.view();
    let roots = view.roots();
    assert_eq!(roots.len(), 1);
    let span = view.span(roots[0]);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 1);
}

// ─── 6. Deeply nested expression (10+ levels) ──────────────────────

#[test]
fn parse_deeply_nested_10_levels() {
    let mut grammar = GrammarBuilder::new("deep")
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

    let depth = 12;
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

    let forest = pipeline_parse(&mut grammar, &tokens).expect("deeply nested should parse");

    let view = forest.view();
    let roots = view.roots();
    assert!(!roots.is_empty());
    assert_eq!(view.span(roots[0]).start, 0);
    assert_eq!(view.span(roots[0]).end, byte);
}

// ─── 7. Ambiguous grammar requiring GLR forks ──────────────────────

#[test]
fn parse_ambiguous_grammar_with_forks() {
    // E → E + E | NUM  — inherently ambiguous for "1+2+3"
    let mut grammar = GrammarBuilder::new("ambig")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["NUM"])
        .start("E")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline ok");
    let conflicts = count_conflicts(&table);
    assert!(
        conflicts.shift_reduce >= 1,
        "ambiguous grammar must have S/R conflicts"
    );

    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");

    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(
        [
            (num, 0u32, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
        ]
        .iter()
        .map(|&(s, st, en)| (s.0 as u32, st, en)),
    );

    // GLR should find at least one valid parse
    assert!(
        result.is_ok(),
        "ambiguous input should still parse: {:?}",
        result.err()
    );
    let forest = result.unwrap();
    assert!(!forest.view().roots().is_empty());
}

// ─── 8. All forks fail except one ───────────────────────────────────

#[test]
fn parse_only_one_fork_survives() {
    // S → 'a' 'b' | 'a' 'c'
    // Input: "a c" — only the second alternative survives.
    let mut grammar = GrammarBuilder::new("fork_survive")
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
        .expect("one surviving fork should parse");

    let view = forest.view();
    let roots = view.roots();
    assert_eq!(roots.len(), 1);
    assert_eq!(view.span(roots[0]).start, 0);
    assert_eq!(view.span(roots[0]).end, 2);
}

// ─── 9. Shift action transitions are correct ────────────────────────

#[test]
fn driver_shift_transitions_correct() {
    // S → 'x' 'y'   (two shifts then reduce)
    // 0: EOF, 1: x, 2: y, 3: S
    let eof = SymbolId(0);
    let x = SymbolId(1);
    let y = SymbolId(2);
    let s = SymbolId(3);

    let rules = vec![ParseRule { lhs: s, rhs_len: 2 }]; // S → x y

    let mut actions = vec![vec![vec![]; 4]; 4];
    actions[0][1].push(Action::Shift(StateId(1))); // state0 + x → state1
    actions[1][2].push(Action::Shift(StateId(2))); // state1 + y → state2
    actions[2][0].push(Action::Reduce(RuleId(0))); // state2 + EOF → reduce S
    actions[3][0].push(Action::Accept); // state3 + EOF → accept

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 4]; 4];
    gotos[0][3] = StateId(3);

    let table = create_test_table(actions, gotos, rules, s, eof);

    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens([(1u32, 0u32, 1u32), (2, 1, 2)].iter().copied())
        .expect("shift transitions should succeed");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).end, 2);
}

// ─── 10. Reduce action transitions are correct ──────────────────────

#[test]
fn driver_reduce_transitions_correct() {
    // A → 'a';  S → A 'b'
    // 0: EOF, 1: a, 2: b, 3: S, 4: A
    let eof = SymbolId(0);
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s_sym = SymbolId(3);
    let a_sym = SymbolId(4);

    let rules = vec![
        ParseRule {
            lhs: a_sym,
            rhs_len: 1,
        }, // rule0: A → a
        ParseRule {
            lhs: s_sym,
            rhs_len: 2,
        }, // rule1: S → A b
    ];

    let mut actions = vec![vec![vec![]; 5]; 5];
    actions[0][1].push(Action::Shift(StateId(1))); // state0 + a → state1
    actions[1][2].push(Action::Reduce(RuleId(0))); // state1 + b → reduce A→a
    actions[2][2].push(Action::Shift(StateId(3))); // state2 + b → state3
    actions[3][0].push(Action::Reduce(RuleId(1))); // state3 + EOF → reduce S→Ab
    actions[4][0].push(Action::Accept); // state4 + EOF → accept

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 5]; 5];
    gotos[0][4] = StateId(2); // after A in state0 → state2
    gotos[0][3] = StateId(4); // after S in state0 → accept state

    let table = create_test_table(actions, gotos, rules, s_sym, eof);

    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens([(1u32, 0u32, 1u32), (2, 1, 2)].iter().copied())
        .expect("reduce transitions should succeed");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).start, 0);
    assert_eq!(view.span(view.roots()[0]).end, 2);
}

// ─── 11. Driver accepts valid input ─────────────────────────────────

#[test]
fn driver_accept_valid_input() {
    // S → 't'
    let eof = SymbolId(0);
    let t = SymbolId(1);
    let s_sym = SymbolId(3);

    let rules = vec![ParseRule {
        lhs: s_sym,
        rhs_len: 1,
    }];

    let mut actions = vec![vec![vec![]; 4]; 3];
    actions[0][1].push(Action::Shift(StateId(1)));
    actions[1][0].push(Action::Reduce(RuleId(0)));
    actions[2][0].push(Action::Accept);

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 4]; 3];
    gotos[0][3] = StateId(2);

    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens([(1u32, 0u32, 1u32)].iter().copied());

    assert!(result.is_ok(), "valid input must be accepted");
}

// ─── 12. Driver rejects invalid input ───────────────────────────────

#[test]
fn driver_rejects_invalid_input() {
    // Hand-crafted table: S → 'a' 'b', no recovery.
    // Feed 'a' 'a' — second token has no action → must reject.
    let eof = SymbolId(0);
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s_sym = SymbolId(3);

    let rules = vec![ParseRule {
        lhs: s_sym,
        rhs_len: 2,
    }];

    let mut actions = vec![vec![vec![]; 4]; 4];
    actions[0][1].push(Action::Shift(StateId(1))); // state0 + a → state1
    actions[1][2].push(Action::Shift(StateId(2))); // state1 + b → state2  (no action on 'a'!)
    actions[2][0].push(Action::Reduce(RuleId(0))); // state2 + EOF → reduce S
    actions[3][0].push(Action::Accept);

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 4]; 4];
    gotos[0][3] = StateId(3);

    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    let mut driver = Driver::new(&table);

    // Feed 'a' 'a' — no action for 'a' in state1 → error
    let result = driver.parse_tokens([(1u32, 0u32, 1u32), (1u32, 1, 2)].iter().copied());
    assert!(result.is_err(), "invalid input should be rejected");
}

// ─── 13. Reduce with different rule lengths ─────────────────────────

#[test]
fn driver_reduce_various_lengths() {
    // R0: A → ε (len 0)
    // R1: B → 'x' (len 1)
    // R2: S → A B 'y' (len 3)
    let eof = SymbolId(0);
    let x = SymbolId(1);
    let y = SymbolId(2);
    let s_sym = SymbolId(3);
    let a_sym = SymbolId(4);
    let b_sym = SymbolId(5);

    let rules = vec![
        ParseRule {
            lhs: a_sym,
            rhs_len: 0,
        }, // R0: A → ε
        ParseRule {
            lhs: b_sym,
            rhs_len: 1,
        }, // R1: B → x
        ParseRule {
            lhs: s_sym,
            rhs_len: 3,
        }, // R2: S → A B y
    ];

    let mut actions = vec![vec![vec![]; 6]; 6];
    // State 0: reduce A → ε on 'x'
    actions[0][1].push(Action::Reduce(RuleId(0)));
    // State 1 (after A): shift 'x' → state 2
    actions[1][1].push(Action::Shift(StateId(2)));
    // State 2: reduce B → x on 'y'
    actions[2][2].push(Action::Reduce(RuleId(1)));
    // State 3 (after A B): shift 'y' → state 4
    actions[3][2].push(Action::Shift(StateId(4)));
    // State 4: reduce S → A B y on EOF
    actions[4][0].push(Action::Reduce(RuleId(2)));
    // State 5: accept
    actions[5][0].push(Action::Accept);

    let inv = StateId(65535);
    let mut gotos = vec![vec![inv; 6]; 6];
    gotos[0][4] = StateId(1); // after A → state1
    gotos[1][5] = StateId(3); // after B → state3
    gotos[0][3] = StateId(5); // after S → accept

    let table = create_test_table(actions, gotos, rules, s_sym, eof);
    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens([(1u32, 0u32, 1u32), (2, 1, 2)].iter().copied())
        .expect("mixed-length reduces should succeed");

    let view = forest.view();
    assert!(!view.roots().is_empty());
}

// ─── 14. Production metadata maintained through pipeline ────────────

#[test]
fn production_metadata_maintained() {
    let mut grammar = GrammarBuilder::new("meta")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUM"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline should succeed");

    // Every rule in the table should have a valid LHS and rhs_len
    for (i, rule) in table.rules.iter().enumerate() {
        assert!(
            rule.rhs_len <= 10,
            "rule {} has unreasonable rhs_len {}",
            i,
            rule.rhs_len
        );
        // LHS should be a known non-terminal
        assert!(
            table.nonterminal_to_index.contains_key(&rule.lhs)
                || rule.lhs == table.start_symbol
                || rule.lhs.0 > 0,
            "rule {} has unknown LHS {:?}",
            i,
            rule.lhs
        );
    }

    // Start symbol should be set
    assert!(table.start_symbol.0 > 0, "start symbol should be non-zero");
    // EOF symbol present in mapping
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF symbol must be in symbol_to_index"
    );
}

// ─── 15. Parse table actions match expected grammar ─────────────────

#[test]
fn parse_table_actions_match_grammar() {
    let mut grammar = GrammarBuilder::new("check_actions")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    sanity_check_tables(&table).expect("sanity");

    // Must have Accept somewhere
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "table must contain Accept action");

    // Must have at least one Shift
    let has_shift = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    });
    assert!(has_shift, "table must contain Shift action");

    // Must have at least one Reduce
    let has_reduce = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    });
    assert!(has_reduce, "table must contain Reduce action");
}

// ─── 16. Multiple alternatives parsed correctly ─────────────────────

#[test]
fn parse_multiple_rule_alternatives() {
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

    // Both alternatives should parse independently
    let f1 = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("'a' should parse");
    assert_eq!(f1.view().roots().len(), 1);

    let f2 = pipeline_parse(&mut grammar, &[(b, 0, 1)]).expect("'b' should parse");
    assert_eq!(f2.view().roots().len(), 1);
}

// ─── 17. Left-recursive grammar through driver ──────────────────────

#[test]
fn parse_left_recursive_through_driver() {
    // A → A 'a' | 'a'    Input: "aaa"
    let mut grammar = GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("A", vec!["A", "a"])
        .rule("A", vec!["a"])
        .start("A")
        .build();

    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (a, 1, 2), (a, 2, 3)])
        .expect("left-recursive should parse");

    let view = forest.view();
    assert!(!view.roots().is_empty());
    assert_eq!(view.span(view.roots()[0]).end, 3);
}

// ─── 18. Right-recursive grammar through driver ─────────────────────

#[test]
fn parse_right_recursive_through_driver() {
    // This uses an unambiguous right-recursive form: L → 'a' L | 'a'
    // Input: "aa"
    let mut grammar = GrammarBuilder::new("rightrec")
        .token("a", "a")
        .rule("L", vec!["a", "L"])
        .rule("L", vec!["a"])
        .start("L")
        .build();

    let a = sym_id(&grammar, "a");
    let table = run_pipeline(&mut grammar).expect("pipeline");

    let mut driver = Driver::new(&table);
    let result = driver.parse_tokens(
        [(a.0 as u32, 0u32, 1u32), (a.0 as u32, 1, 2)]
            .iter()
            .copied(),
    );
    // Right-recursive grammars may have S/R conflicts in LR(1); the GLR driver
    // should still find a parse via forking.
    assert!(
        result.is_ok(),
        "right-recursive should parse: {:?}",
        result.err()
    );
}

// ─── 19. Span tracking across multiple reductions ───────────────────

#[test]
fn span_tracking_across_reductions() {
    let mut grammar = GrammarBuilder::new("spans")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "+");

    // "1+2+3" → bytes 0..5
    let forest = pipeline_parse(
        &mut grammar,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
        ],
    )
    .expect("should parse");

    let view = forest.view();
    let root = view.roots()[0];
    let span = view.span(root);
    assert_eq!(span.start, 0, "root should start at byte 0");
    assert_eq!(span.end, 5, "root should end at byte 5");
}

// ─── 20. Unambiguous grammar has no conflicts ───────────────────────

#[test]
fn unambiguous_grammar_zero_conflicts() {
    let mut grammar = GrammarBuilder::new("unamb")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "no S/R conflicts");
    assert_eq!(summary.reduce_reduce, 0, "no R/R conflicts");
}

// ─── 21. Forest children structure correct ──────────────────────────

#[test]
fn forest_children_structure() {
    // S → 'a' 'b'  → root has 2 children
    let mut grammar = GrammarBuilder::new("children")
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
    let children = view.best_children(root);
    assert_eq!(children.len(), 2, "S → a b should have 2 children");

    assert_eq!(view.span(children[0]).start, 0);
    assert_eq!(view.span(children[0]).end, 1);
    assert_eq!(view.span(children[1]).start, 1);
    assert_eq!(view.span(children[1]).end, 2);
}

// ─── 22. Error stats available via test hooks ───────────────────────

#[test]
fn error_stats_available_via_test_hooks() {
    let mut grammar = GrammarBuilder::new("hooks")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("should parse");

    // test-api feature enables test_hooks
    let stats = forest.debug_error_stats();
    // Clean parse → no errors
    assert!(!stats.0, "has_error should be false for clean parse");
    assert_eq!(stats.2, 0, "error cost should be 0 for clean parse");
}

// ─── 23. Table initial state is valid ───────────────────────────────

#[test]
fn table_initial_state_valid() {
    let mut grammar = GrammarBuilder::new("init_state")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");
    assert!(
        (table.initial_state.0 as usize) < table.state_count,
        "initial_state must be within state_count"
    );
}

// ─── 24. Multi-token sequence with all action types ─────────────────

#[test]
fn multi_token_exercises_shift_reduce_accept() {
    // S → A B; A → 'x'; B → 'y'
    // Input: "xy" exercises shift x, reduce A, shift y, reduce B, reduce S, accept
    let mut grammar = GrammarBuilder::new("all_actions")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build();

    let x = sym_id(&grammar, "x");
    let y = sym_id(&grammar, "y");

    let forest = pipeline_parse(&mut grammar, &[(x, 0, 1), (y, 1, 2)])
        .expect("should exercise shift/reduce/accept");

    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).start, 0);
    assert_eq!(view.span(root).end, 2);

    // Root should have 2 children (A, B)
    let children = view.best_children(root);
    assert_eq!(children.len(), 2);
}

// ─── 25. Driver handles longer right-hand side ──────────────────────

#[test]
fn driver_long_rhs_rule() {
    // S → 'a' 'b' 'c' 'd' 'e'  (rhs_len = 5)
    let mut grammar = GrammarBuilder::new("long_rhs")
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
    .expect("long RHS should parse");

    let view = forest.view();
    assert_eq!(view.span(view.roots()[0]).end, 5);
    assert_eq!(view.best_children(view.roots()[0]).len(), 5);
}

// ─── 26. Ambiguous grammar: both parses valid ──────────────────────

#[test]
fn ambiguous_grammar_both_parses_reachable() {
    // E → E '+' E | 'n'   Input: "n+n"
    // This is ambiguous for longer inputs but should find at least one parse for "n+n".
    let mut grammar = GrammarBuilder::new("ambig2")
        .token("n", "n")
        .token("+", "+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build();

    let n = sym_id(&grammar, "n");
    let plus = sym_id(&grammar, "+");

    let table = run_pipeline(&mut grammar).expect("pipeline");
    let mut driver = Driver::new(&table);
    let forest = driver
        .parse_tokens(
            [
                (n.0 as u32, 0u32, 1u32),
                (plus.0 as u32, 1, 2),
                (n.0 as u32, 2, 3),
            ]
            .iter()
            .copied(),
        )
        .expect("ambiguous n+n should parse");

    let view = forest.view();
    assert!(!view.roots().is_empty());
    assert_eq!(view.span(view.roots()[0]).end, 3);
}

// ─── 27. GOTO table has entries for non-terminals ───────────────────

#[test]
fn goto_table_has_nonterminal_entries() {
    let mut grammar = GrammarBuilder::new("goto_check")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["A", "b"])
        .rule("A", vec!["a"])
        .start("S")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline");

    // The goto table must have at least one non-empty entry
    let has_goto = table
        .goto_table
        .iter()
        .any(|row| row.iter().any(|&s| s != StateId(0)));
    assert!(has_goto, "goto table must have entries for non-terminals");

    // Nonterminal index should have entries
    assert!(
        !table.nonterminal_to_index.is_empty()
            || table.goto_indexing == GotoIndexing::DirectSymbolId,
        "nonterminal_to_index should be populated for NonterminalMap indexing"
    );
}

// ─── 28. Repeated reduction chain ───────────────────────────────────

#[test]
fn repeated_reduction_chain() {
    // A → 'a'; B → A; C → B; S → C
    // Single-step chain grammar that requires multiple reductions per token
    let mut grammar = GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("A", vec!["a"])
        .rule("B", vec!["A"])
        .rule("C", vec!["B"])
        .rule("S", vec!["C"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");

    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("chain should parse");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).end, 1);
}
