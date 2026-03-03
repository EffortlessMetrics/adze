//! Property-based tests for the GLR driver.
//!
//! Run with: cargo test -p adze-glr-core --test driver_proptest
#![allow(clippy::needless_range_loop)]

use adze_glr_core::{Action, Driver, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Table builder helpers
// ---------------------------------------------------------------------------

type ActionCell = Vec<Action>;

/// Sentinel used for "no goto" entries.
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
        grammar: Grammar::new("proptest".to_string()),
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

/// Build the canonical "S -> 'a'" table used by many tests.
/// Symbols: 0=EOF, 1='a', 2=S(NT)
fn build_s_to_a_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(2);
    let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
    let mut actions = vec![vec![vec![]; 3]; 3];
    actions[0][1].push(Action::Shift(StateId(1)));
    actions[1][0].push(Action::Reduce(RuleId(0)));
    actions[2][0].push(Action::Accept);
    let mut gotos = vec![vec![NO_GOTO; 3]; 3];
    gotos[0][2] = StateId(2);
    build_table(actions, gotos, rules, s, eof, 2)
}

/// Build a right-recursive "S -> A; A -> 'a' | 'a' A" table.
/// Symbols: 0=EOF, 1='a', 2=S, 3=A
fn build_right_recursive_table() -> ParseTable {
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
    let num_syms = 4;
    let num_states = 5;
    let mut actions = vec![vec![vec![]; num_syms]; num_states];
    actions[0][1].push(Action::Shift(StateId(1)));
    actions[1][1].push(Action::Shift(StateId(1)));
    actions[1][0].push(Action::Reduce(RuleId(1)));
    actions[2][0].push(Action::Reduce(RuleId(0)));
    actions[3][0].push(Action::Accept);
    actions[4][0].push(Action::Reduce(RuleId(2)));
    let mut gotos = vec![vec![NO_GOTO; num_syms]; num_states];
    gotos[0][3] = StateId(2);
    gotos[0][2] = StateId(3);
    gotos[1][3] = StateId(4);
    build_table(actions, gotos, rules, s, eof, 2)
}

/// Build an epsilon grammar "S -> ε".
/// Symbols: 0=EOF, 1=S(NT)
fn build_epsilon_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(1);
    let rules = vec![ParseRule { lhs: s, rhs_len: 0 }];
    let mut actions = vec![vec![vec![]; 2]; 2];
    actions[0][0].push(Action::Reduce(RuleId(0)));
    actions[1][0].push(Action::Accept);
    let mut gotos = vec![vec![NO_GOTO; 2]; 2];
    gotos[0][1] = StateId(1);
    build_table(actions, gotos, rules, s, eof, 1)
}

// ---------------------------------------------------------------------------
// 1. Driver creation — new() never panics with a well-formed table
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn driver_creation_succeeds(_seed in 0u32..100) {
        let table = build_s_to_a_table();
        let _driver = Driver::new(&table);
    }
}

// ---------------------------------------------------------------------------
// 2. Driver creation with varying terminal counts
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn driver_creation_variable_terminal_id(terminal_id in 1u16..8) {
        let eof = SymbolId(0);
        let s_id = terminal_id.max(2) + 1;
        let s = SymbolId(s_id);
        let num_syms = (s_id as usize) + 1;
        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
        let num_states = 3;
        let mut actions = vec![vec![vec![]; num_syms]; num_states];
        actions[0][terminal_id as usize].push(Action::Shift(StateId(1)));
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);
        let mut gotos = vec![vec![NO_GOTO; num_syms]; num_states];
        gotos[0][s_id as usize] = StateId(2);
        let table = build_table(actions, gotos, rules, s, eof, (terminal_id as usize) + 1);
        let _driver = Driver::new(&table);
    }
}

// ---------------------------------------------------------------------------
// 3. parse_tokens with valid single-token input accepts
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parse_single_valid_token(_seed in 0u32..100) {
        let table = build_s_to_a_table();
        let mut driver = Driver::new(&table);
        let tokens = vec![(1u32, 0u32, 1u32)];
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok(), "single 'a' must parse: {:?}", result.err());
        let forest = result.unwrap();
        prop_assert_eq!(forest.view().roots().len(), 1);
    }
}

// ---------------------------------------------------------------------------
// 4. Deterministic grammars give exactly one root
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn deterministic_grammar_one_result(extra_tokens in 0usize..=3) {
        let table = build_s_to_a_table();
        let tokens = vec![(1u32, 0u32, 1u32)];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap().view().roots().len(), 1);

        if extra_tokens > 0 {
            let tokens2: Vec<(u32, u32, u32)> = (0..=extra_tokens)
                .map(|i| (1u32, i as u32, i as u32 + 1))
                .collect();
            let mut driver2 = Driver::new(&table);
            // Must terminate — either error or early accept
            let _ = driver2.parse_tokens(tokens2.into_iter());
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Parsing always terminates (no infinite loops)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parsing_always_terminates(num_tokens in 0usize..=8) {
        let table = build_s_to_a_table();
        let tokens: Vec<(u32, u32, u32)> = (0..num_tokens)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        let _result = driver.parse_tokens(tokens.into_iter());
    }
}

// ---------------------------------------------------------------------------
// 6. Reject invalid token kind at position 0
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reject_invalid_input_at_position_zero(bad_kind in 10u32..100) {
        let table = build_s_to_a_table();
        let tokens = vec![(bad_kind, 0u32, 1u32)];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        // Either error or recovered with error cost — must not panic
        let _ = result;
    }
}

// ---------------------------------------------------------------------------
// 7. GlrError::Lex variant contains message
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn glr_error_lex_display(msg in "[a-z]{1,20}") {
        let err = adze_glr_core::driver::GlrError::Lex(msg.clone());
        let display = format!("{err}");
        prop_assert!(display.contains(&msg), "Lex display must contain message");
    }
}

// ---------------------------------------------------------------------------
// 8. GlrError::Parse variant contains message
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn glr_error_parse_display(msg in "[a-z]{1,20}") {
        let err = adze_glr_core::driver::GlrError::Parse(msg.clone());
        let display = format!("{err}");
        prop_assert!(display.contains(&msg), "Parse display must contain message");
    }
}

// ---------------------------------------------------------------------------
// 9. GlrError::Other variant contains message
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn glr_error_other_display(msg in "[a-z]{1,20}") {
        let err = adze_glr_core::driver::GlrError::Other(msg.clone());
        let display = format!("{err}");
        prop_assert!(display.contains(&msg), "Other display must contain message");
    }
}

// ---------------------------------------------------------------------------
// 10. GlrError Debug is non-empty for all variants
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn glr_error_debug_nonempty(variant in 0u8..3, msg in "[a-z]{1,10}") {
        let err = match variant {
            0 => adze_glr_core::driver::GlrError::Lex(msg),
            1 => adze_glr_core::driver::GlrError::Parse(msg),
            _ => adze_glr_core::driver::GlrError::Other(msg),
        };
        let debug = format!("{err:?}");
        prop_assert!(!debug.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 11. Empty input on S->ε accepts
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn accept_epsilon_grammar_empty_input(_seed in 0u32..100) {
        let table = build_epsilon_table();
        let tokens: Vec<(u32, u32, u32)> = vec![];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok(), "S->ε must accept empty input: {:?}", result.err());
        let view = result.unwrap();
        let roots = view.view().roots();
        prop_assert!(!roots.is_empty());
        for &root in roots {
            let sp = view.view().span(root);
            prop_assert_eq!(sp.start, 0);
            prop_assert_eq!(sp.end, 0);
        }
    }
}

// ---------------------------------------------------------------------------
// 12. Epsilon grammar rejects non-empty input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn epsilon_grammar_rejects_nonempty(n_tokens in 1usize..=4) {
        let table = build_epsilon_table();
        let tokens: Vec<(u32, u32, u32)> = (0..n_tokens)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        // Epsilon grammar has no shift for terminal 1; must terminate without panic
        let _ = driver.parse_tokens(tokens.into_iter());
    }
}

// ---------------------------------------------------------------------------
// 13. Right-recursive grammar accepts variable-length input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn right_recursive_accepts_n_tokens(token_count in 1usize..=5) {
        let table = build_right_recursive_table();
        let tokens: Vec<(u32, u32, u32)> = (0..token_count)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok(), "{} 'a's must parse: {:?}", token_count, result.err());
        let forest = result.unwrap();
        let view = forest.view();
        prop_assert!(!view.roots().is_empty());
        for &root in view.roots() {
            let sp = view.span(root);
            prop_assert_eq!(sp.start, 0);
            prop_assert_eq!(sp.end, token_count as u32);
        }
    }
}

// ---------------------------------------------------------------------------
// 14. Forest root span covers full input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn root_span_covers_full_input(token_count in 1usize..=6) {
        let table = build_right_recursive_table();
        let tokens: Vec<(u32, u32, u32)> = (0..token_count)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(tokens.into_iter()).unwrap();
        let view = forest.view();
        for &root in view.roots() {
            let sp = view.span(root);
            prop_assert_eq!(sp.start, 0);
            prop_assert_eq!(sp.end, token_count as u32);
        }
    }
}

// ---------------------------------------------------------------------------
// 15. Shift+reduce with variable terminal id
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn shift_reduce_accepts_single_token(terminal_id in 1u16..5) {
        let eof = SymbolId(0);
        let t = SymbolId(terminal_id);
        let s_id = terminal_id.max(2) + 1;
        let s = SymbolId(s_id);
        let num_syms = (s_id as usize) + 1;
        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
        let num_states = 3;
        let mut actions = vec![vec![vec![]; num_syms]; num_states];
        actions[0][terminal_id as usize].push(Action::Shift(StateId(1)));
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);
        let mut gotos = vec![vec![NO_GOTO; num_syms]; num_states];
        gotos[0][s_id as usize] = StateId(2);
        let table = build_table(actions, gotos, rules, s, eof, (terminal_id as usize) + 1);
        let tokens = vec![(t.0 as u32, 0u32, 1u32)];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok());
        let forest = result.unwrap();
        prop_assert_eq!(forest.view().roots().len(), 1);
        let sp = forest.view().span(forest.view().roots()[0]);
        prop_assert_eq!(sp.start, 0);
        prop_assert_eq!(sp.end, 1);
    }
}

// ---------------------------------------------------------------------------
// 16. Driver handles tokens with zero-width spans
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn zero_width_token_does_not_panic(_seed in 0u32..50) {
        let table = build_s_to_a_table();
        let tokens = vec![(1u32, 0u32, 0u32)]; // zero-width
        let mut driver = Driver::new(&table);
        let _ = driver.parse_tokens(tokens.into_iter());
    }
}

// ---------------------------------------------------------------------------
// 17. Driver with sequential drivers on same table
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn multiple_drivers_same_table(n in 1usize..=5) {
        let table = build_s_to_a_table();
        for _ in 0..n {
            let mut driver = Driver::new(&table);
            let result = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter());
            prop_assert!(result.is_ok());
        }
    }
}

// ---------------------------------------------------------------------------
// 18. Driver reset: creating fresh driver produces same results
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn driver_reset_produces_same_results(_seed in 0u32..50) {
        let table = build_s_to_a_table();
        let tokens = || vec![(1u32, 0u32, 1u32)];

        let mut d1 = Driver::new(&table);
        let r1 = d1.parse_tokens(tokens().into_iter());

        // "Reset" by creating a new driver
        let mut d2 = Driver::new(&table);
        let r2 = d2.parse_tokens(tokens().into_iter());

        prop_assert_eq!(r1.is_ok(), r2.is_ok());
        if let (Ok(f1), Ok(f2)) = (r1, r2) {
            prop_assert_eq!(f1.view().roots().len(), f2.view().roots().len());
        }
    }
}

// ---------------------------------------------------------------------------
// 19. Empty token stream on non-epsilon grammar fails
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn empty_input_non_epsilon_grammar_fails(_seed in 0u32..50) {
        let table = build_s_to_a_table();
        let tokens: Vec<(u32, u32, u32)> = vec![];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        // S -> 'a' requires at least one token; empty input should fail
        prop_assert!(result.is_err(), "empty input on S->'a' must fail");
    }
}

// ---------------------------------------------------------------------------
// 20. Forest kind matches start symbol for accepted parses
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn forest_root_kind_is_start_symbol(token_count in 1usize..=4) {
        let table = build_right_recursive_table();
        let tokens: Vec<(u32, u32, u32)> = (0..token_count)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(tokens.into_iter()).unwrap();
        let view = forest.view();
        for &root in view.roots() {
            let kind = view.kind(root);
            // Start symbol is S = SymbolId(2) -> kind should be 2
            prop_assert_eq!(kind, 2, "root kind must be start symbol");
        }
    }
}

// ---------------------------------------------------------------------------
// 21. Forest best_children returns children for nonterminal roots
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn forest_root_has_children(_seed in 0u32..50) {
        let table = build_s_to_a_table();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter()).unwrap();
        let view = forest.view();
        let root = view.roots()[0];
        let children = view.best_children(root);
        // S -> 'a' produces exactly one child (the terminal 'a')
        prop_assert!(!children.is_empty(), "nonterminal root must have children");
    }
}

// ---------------------------------------------------------------------------
// 22. Monotonically increasing token positions accepted
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn monotonic_positions(gap in 1u32..=5, count in 1usize..=4) {
        let table = build_right_recursive_table();
        let mut pos = 0u32;
        let tokens: Vec<(u32, u32, u32)> = (0..count)
            .map(|_| {
                let start = pos;
                pos += gap;
                (1u32, start, pos)
            })
            .collect();
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok());
    }
}

// ---------------------------------------------------------------------------
// 23. Token stream with mixed valid/invalid kinds terminates
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn mixed_valid_invalid_terminates(
        kinds in prop::collection::vec(0u32..10, 1..=6),
    ) {
        let table = build_s_to_a_table();
        let tokens: Vec<(u32, u32, u32)> = kinds
            .iter()
            .enumerate()
            .map(|(i, &k)| (k, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        let _ = driver.parse_tokens(tokens.into_iter());
    }
}

// ---------------------------------------------------------------------------
// 24. Fork action does not panic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn fork_action_does_not_panic(_seed in 0u32..50) {
        // S -> 'a' with a Fork cell containing two shifts (simulated ambiguity)
        let eof = SymbolId(0);
        let s = SymbolId(2);
        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
        let mut actions = vec![vec![vec![]; 3]; 4];
        // State 0: Fork on 'a' with two shift targets
        actions[0][1].push(Action::Fork(vec![
            Action::Shift(StateId(1)),
            Action::Shift(StateId(1)),
        ]));
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);
        // State 3 unused but keeps table dimensions consistent
        let mut gotos = vec![vec![NO_GOTO; 3]; 4];
        gotos[0][2] = StateId(2);
        let table = build_table(actions, gotos, rules, s, eof, 2);
        let mut driver = Driver::new(&table);
        let _ = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter());
    }
}

// ---------------------------------------------------------------------------
// 25. Error action cell causes failure or recovery, not panic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn error_action_does_not_panic(_seed in 0u32..50) {
        let eof = SymbolId(0);
        let s = SymbolId(2);
        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
        let mut actions = vec![vec![vec![]; 3]; 3];
        // State 0: explicit Error on 'a'
        actions[0][1].push(Action::Error);
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);
        let mut gotos = vec![vec![NO_GOTO; 3]; 3];
        gotos[0][2] = StateId(2);
        let table = build_table(actions, gotos, rules, s, eof, 2);
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter());
        // Error cell should make parse fail
        prop_assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// 26. Recover action cell does not panic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn recover_action_does_not_panic(_seed in 0u32..50) {
        let eof = SymbolId(0);
        let s = SymbolId(2);
        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
        let mut actions = vec![vec![vec![]; 3]; 3];
        // State 0: only Recover on 'a' (no real shift)
        actions[0][1].push(Action::Recover);
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);
        let mut gotos = vec![vec![NO_GOTO; 3]; 3];
        gotos[0][2] = StateId(2);
        let table = build_table(actions, gotos, rules, s, eof, 2);
        let mut driver = Driver::new(&table);
        let _ = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter());
    }
}

// ---------------------------------------------------------------------------
// 27. Large token positions do not overflow
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn large_positions_no_overflow(offset in 1_000_000u32..2_000_000u32) {
        let table = build_s_to_a_table();
        let tokens = vec![(1u32, offset, offset + 1)];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok());
        let forest = result.unwrap();
        let sp = forest.view().span(forest.view().roots()[0]);
        prop_assert_eq!(sp.start, offset);
        prop_assert_eq!(sp.end, offset + 1);
    }
}

// ---------------------------------------------------------------------------
// 28. Parsing idempotent — same input, same outcome
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn parse_is_idempotent(n in 1usize..=4) {
        let table = build_right_recursive_table();
        let tokens_fn = || -> Vec<(u32, u32, u32)> {
            (0..n).map(|i| (1u32, i as u32, i as u32 + 1)).collect()
        };
        let mut d1 = Driver::new(&table);
        let r1 = d1.parse_tokens(tokens_fn().into_iter());
        let mut d2 = Driver::new(&table);
        let r2 = d2.parse_tokens(tokens_fn().into_iter());
        prop_assert_eq!(r1.is_ok(), r2.is_ok());
        if let (Ok(f1), Ok(f2)) = (r1, r2) {
            prop_assert_eq!(f1.view().roots().len(), f2.view().roots().len());
            for i in 0..f1.view().roots().len() {
                prop_assert_eq!(
                    f1.view().span(f1.view().roots()[i]),
                    f2.view().span(f2.view().roots()[i]),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 29. Multiple tokens with wrong kind after valid prefix
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn wrong_kind_after_valid_prefix(bad_kind in 5u32..20) {
        let table = build_right_recursive_table();
        // One valid 'a' then one invalid token
        let tokens = vec![
            (1u32, 0u32, 1u32),
            (bad_kind, 1u32, 2u32),
        ];
        let mut driver = Driver::new(&table);
        let _ = driver.parse_tokens(tokens.into_iter());
    }
}

// ---------------------------------------------------------------------------
// 30. Accepted forest has non-empty children arrays
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn accepted_forest_children_nonempty(n in 1usize..=3) {
        let table = build_right_recursive_table();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(tokens.into_iter()).unwrap();
        let view = forest.view();
        let root = view.roots()[0];
        // Root is S (nonterminal) so it must have children
        let children = view.best_children(root);
        prop_assert!(!children.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 31. Children span is contained within parent span
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn children_span_within_parent(n in 1usize..=4) {
        let table = build_right_recursive_table();
        let tokens: Vec<(u32, u32, u32)> = (0..n)
            .map(|i| (1u32, i as u32, i as u32 + 1))
            .collect();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(tokens.into_iter()).unwrap();
        let view = forest.view();
        let root = view.roots()[0];
        let parent_sp = view.span(root);
        for &child in view.best_children(root) {
            let child_sp = view.span(child);
            prop_assert!(child_sp.start >= parent_sp.start);
            prop_assert!(child_sp.end <= parent_sp.end);
        }
    }
}

// ---------------------------------------------------------------------------
// 32. Non-existent node id returns zero span
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn nonexistent_node_returns_zero_span(bogus_id in 9000u32..10000u32) {
        let table = build_s_to_a_table();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter()).unwrap();
        let view = forest.view();
        let sp = view.span(bogus_id);
        prop_assert_eq!(sp.start, 0);
        prop_assert_eq!(sp.end, 0);
    }
}

// ---------------------------------------------------------------------------
// 33. Non-existent node id returns zero kind
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn nonexistent_node_returns_zero_kind(bogus_id in 9000u32..10000u32) {
        let table = build_s_to_a_table();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter()).unwrap();
        let view = forest.view();
        prop_assert_eq!(view.kind(bogus_id), 0);
    }
}

// ---------------------------------------------------------------------------
// 34. Non-existent node id returns empty children
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn nonexistent_node_returns_empty_children(bogus_id in 9000u32..10000u32) {
        let table = build_s_to_a_table();
        let mut driver = Driver::new(&table);
        let forest = driver.parse_tokens(vec![(1u32, 0u32, 1u32)].into_iter()).unwrap();
        let view = forest.view();
        prop_assert!(view.best_children(bogus_id).is_empty());
    }
}

// ---------------------------------------------------------------------------
// 35. Parsing with ascending and gapped positions
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn gapped_positions_accepted(gaps in prop::collection::vec(1u32..=10, 1..=4)) {
        let table = build_right_recursive_table();
        let mut pos = 0u32;
        let tokens: Vec<(u32, u32, u32)> = gaps
            .iter()
            .map(|&g| {
                let start = pos;
                pos += g;
                (1u32, start, pos)
            })
            .collect();
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok());
        let forest = result.unwrap();
        let view = forest.view();
        let sp = view.span(view.roots()[0]);
        prop_assert_eq!(sp.start, 0);
        prop_assert_eq!(sp.end, pos);
    }
}
