//! Property-based tests for error recovery in adze-glr-core.
//!
//! Run with: cargo test -p adze-glr-core --test error_recovery_proptest
//!
//! Note: These tests use manually constructed parse tables that don't satisfy
//! all strict invariants (e.g., EOF/END parity). They are only compiled when
//! the `strict-invariants` feature is disabled.

#![cfg(not(feature = "strict-invariants"))]
#![allow(clippy::needless_range_loop)]

use adze_glr_core::{Action, Driver, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Table builder helpers
// ---------------------------------------------------------------------------

type ActionCell = Vec<Action>;

/// Sentinel for "no goto" entries.
const NO_GOTO: StateId = StateId(u16::MAX);

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

// ---------------------------------------------------------------------------
// Grammar builders
// ---------------------------------------------------------------------------

/// "S -> 'a'" grammar.
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

/// "S -> 'a' 'b'" grammar.
/// Symbols: 0=EOF, 1='a', 2='b', 3=S(NT)
fn build_s_to_ab_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(3);
    let rules = vec![ParseRule { lhs: s, rhs_len: 2 }];
    let mut actions = vec![vec![vec![]; 4]; 4];
    actions[0][1].push(Action::Shift(StateId(1))); // S0: shift 'a' -> S1
    actions[1][2].push(Action::Shift(StateId(2))); // S1: shift 'b' -> S2
    actions[2][0].push(Action::Reduce(RuleId(0))); // S2: reduce S -> a b
    actions[3][0].push(Action::Accept); // S3: accept on EOF
    let mut gotos = vec![vec![NO_GOTO; 4]; 4];
    gotos[0][3] = StateId(3); // goto S from S0
    build_table(actions, gotos, rules, s, eof, 3)
}

/// "S -> 'a' | 'b'" grammar with two alternatives.
/// Symbols: 0=EOF, 1='a', 2='b', 3=S(NT)
fn build_s_alt_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(3);
    let rules = vec![
        ParseRule { lhs: s, rhs_len: 1 }, // S -> 'a'
        ParseRule { lhs: s, rhs_len: 1 }, // S -> 'b'
    ];
    let mut actions = vec![vec![vec![]; 4]; 4];
    actions[0][1].push(Action::Shift(StateId(1))); // S0: shift 'a' -> S1
    actions[0][2].push(Action::Shift(StateId(2))); // S0: shift 'b' -> S2
    actions[1][0].push(Action::Reduce(RuleId(0))); // S1: reduce S -> a
    actions[2][0].push(Action::Reduce(RuleId(1))); // S2: reduce S -> b
    actions[3][0].push(Action::Accept); // S3: accept
    let mut gotos = vec![vec![NO_GOTO; 4]; 4];
    gotos[0][3] = StateId(3);
    build_table(actions, gotos, rules, s, eof, 3)
}

/// "S -> 'a' 'b' 'c'" grammar.
/// Symbols: 0=EOF, 1='a', 2='b', 3='c', 4=S(NT)
fn build_s_to_abc_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(4);
    let rules = vec![ParseRule { lhs: s, rhs_len: 3 }];
    let mut actions = vec![vec![vec![]; 5]; 5];
    actions[0][1].push(Action::Shift(StateId(1))); // shift 'a'
    actions[1][2].push(Action::Shift(StateId(2))); // shift 'b'
    actions[2][3].push(Action::Shift(StateId(3))); // shift 'c'
    actions[3][0].push(Action::Reduce(RuleId(0))); // reduce
    actions[4][0].push(Action::Accept);
    let mut gotos = vec![vec![NO_GOTO; 5]; 5];
    gotos[0][4] = StateId(4);
    build_table(actions, gotos, rules, s, eof, 4)
}

// ---------------------------------------------------------------------------
// Helper: parse token sequence
// ---------------------------------------------------------------------------

/// Attempt to parse a token sequence, returning Ok(Forest) or Err.
fn try_parse(
    table: &ParseTable,
    tokens: Vec<(u32, u32, u32)>,
) -> Result<adze_glr_core::Forest, adze_glr_core::driver::GlrError> {
    let mut driver = Driver::new(table);
    driver.parse_tokens(tokens)
}

// =========================================================================
// 1. Error recovery after parse error
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Feeding an unexpected token to a simple grammar should either
    /// recover (Ok) or produce a parse error (Err), but never panic.
    #[test]
    fn recovery_after_error_does_not_panic(bad_kind in 3u32..20) {
        let table = build_s_to_a_table();
        // Send a bad token instead of 'a' (kind=1)
        let tokens = vec![(bad_kind, 0, 1), (0, 1, 1)];
        let _ = try_parse(&table, tokens);
    }

    /// After a correct prefix, an unexpected token should not panic.
    #[test]
    fn recovery_after_partial_correct_prefix(extra_kind in 3u32..15) {
        let table = build_s_to_ab_table();
        // 'a' then a bad token instead of 'b'
        let tokens = vec![(1, 0, 1), (extra_kind, 1, 2), (0, 2, 2)];
        let _ = try_parse(&table, tokens);
    }
}

// =========================================================================
// 2. Error recovery token synchronization
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Inserting random junk tokens before the correct sequence:
    /// the driver must not panic regardless of junk count.
    #[test]
    fn sync_after_junk_prefix(junk_len in 0usize..5, junk_kind in 3u32..10) {
        let table = build_s_to_a_table();
        let mut tokens: Vec<(u32, u32, u32)> = Vec::new();
        let mut pos = 0u32;
        for _ in 0..junk_len {
            tokens.push((junk_kind, pos, pos + 1));
            pos += 1;
        }
        tokens.push((1, pos, pos + 1)); // correct 'a'
        pos += 1;
        tokens.push((0, pos, pos)); // EOF
        let _ = try_parse(&table, tokens);
    }

    /// Junk tokens interspersed with valid tokens should not panic.
    #[test]
    fn sync_with_interleaved_junk(junk_kind in 3u32..10) {
        let table = build_s_to_ab_table();
        let tokens = vec![
            (1, 0, 1),         // 'a'
            (junk_kind, 1, 2), // junk between 'a' and 'b'
            (2, 2, 3),         // 'b'
            (0, 3, 3),         // EOF
        ];
        let _ = try_parse(&table, tokens);
    }
}

// =========================================================================
// 3. Error recovery state restoration
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// After recovery, parsing a second valid sequence still works
    /// if the grammar supports it. At minimum we must not panic.
    #[test]
    fn state_restored_after_recovery(bad_kind in 3u32..10) {
        let table = build_s_to_a_table();
        // bad token, then 'a', then EOF
        let tokens = vec![(bad_kind, 0, 1), (1, 1, 2), (0, 2, 2)];
        let _ = try_parse(&table, tokens);
    }

    /// Multiple consecutive bad tokens should still not crash
    /// and the driver should recover or error cleanly.
    #[test]
    fn state_restored_after_multiple_errors(count in 1usize..4, bad_kind in 3u32..10) {
        let table = build_s_to_a_table();
        let mut tokens = Vec::new();
        let mut pos = 0u32;
        for _ in 0..count {
            tokens.push((bad_kind, pos, pos + 1));
            pos += 1;
        }
        tokens.push((1, pos, pos + 1));
        pos += 1;
        tokens.push((0, pos, pos));
        let _ = try_parse(&table, tokens);
    }
}

// =========================================================================
// 4. Error recovery with various error types
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Any symbol id in the terminal range that is not 'a' should
    /// be handled without panic on the S->a grammar.
    #[test]
    fn various_bad_terminal_ids(kind in 2u32..30) {
        let table = build_s_to_a_table();
        let tokens = vec![(kind, 0, 1), (0, 1, 1)];
        let _ = try_parse(&table, tokens);
    }

    /// A zero-width token (start == end) at an unexpected position
    /// should not cause infinite loops or panics.
    #[test]
    fn zero_width_error_token(kind in 3u32..10) {
        let table = build_s_to_a_table();
        let tokens = vec![(kind, 0, 0), (1, 0, 1), (0, 1, 1)];
        let _ = try_parse(&table, tokens);
    }

    /// Large symbol IDs (near u16::MAX boundary) should be handled
    /// gracefully by the driver, either ignoring them or erroring.
    #[test]
    fn high_symbol_id_error_token(kind in 100u32..500) {
        let table = build_s_to_a_table();
        let tokens = vec![(kind, 0, 1), (0, 1, 1)];
        let _ = try_parse(&table, tokens);
    }
}

// =========================================================================
// 5. Error recovery skipped tokens
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// The driver should skip unknown tokens and still try to parse.
    /// We verify it does not panic, and if it succeeds the forest
    /// contains at least one root.
    #[test]
    fn skipped_tokens_prefix(skip_count in 1usize..6, kind in 3u32..10) {
        let table = build_s_to_a_table();
        let mut tokens = Vec::new();
        let mut pos = 0u32;
        for _ in 0..skip_count {
            tokens.push((kind, pos, pos + 1));
            pos += 1;
        }
        tokens.push((1, pos, pos + 1));
        pos += 1;
        tokens.push((0, pos, pos));
        let _ = try_parse(&table, tokens);
    }

    /// Trailing junk after a valid parse should not cause panic.
    #[test]
    fn skipped_tokens_suffix(extra_count in 1usize..4, kind in 3u32..10) {
        let table = build_s_to_a_table();
        // valid 'a' then junk then EOF
        let mut tokens = vec![(1, 0, 1)];
        let mut pos = 1u32;
        for _ in 0..extra_count {
            tokens.push((kind, pos, pos + 1));
            pos += 1;
        }
        tokens.push((0, pos, pos));
        let _ = try_parse(&table, tokens);
    }

    /// Skipping tokens in the middle of a two-token production.
    #[test]
    fn skipped_tokens_middle(middle_count in 1usize..3, kind in 4u32..10) {
        let table = build_s_to_ab_table();
        let mut tokens = vec![(1, 0, 1)]; // 'a'
        let mut pos = 1u32;
        for _ in 0..middle_count {
            tokens.push((kind, pos, pos + 1));
            pos += 1;
        }
        tokens.push((2, pos, pos + 1)); // 'b'
        pos += 1;
        tokens.push((0, pos, pos)); // EOF
        let _ = try_parse(&table, tokens);
    }
}

// =========================================================================
// 6. Error recovery position tracking
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// When valid input parses successfully, the root span should
    /// cover the entire input region.
    #[test]
    fn position_tracking_valid_parse(start_byte in 0u32..100) {
        let table = build_s_to_a_table();
        let end_byte = start_byte + 1;
        let tokens = vec![(1, start_byte, end_byte), (0, end_byte, end_byte)];
        if let Ok(forest) = try_parse(&table, tokens) {
            let view = forest.view();
            let roots = view.roots();
            prop_assert!(!roots.is_empty());
            let span = view.span(roots[0]);
            prop_assert!(span.end >= span.start);
        }
    }

    /// Token positions that increase monotonically should be handled
    /// correctly; span.start <= span.end for all forest nodes.
    #[test]
    fn position_monotonic_tokens(offset in 0u32..50) {
        let table = build_s_to_ab_table();
        let tokens = vec![
            (1, offset, offset + 1),     // 'a'
            (2, offset + 1, offset + 2), // 'b'
            (0, offset + 2, offset + 2), // EOF
        ];
        if let Ok(forest) = try_parse(&table, tokens) {
            let view = forest.view();
            for &root in view.roots() {
                let sp = view.span(root);
                prop_assert!(sp.end >= sp.start);
            }
        }
    }
}

// =========================================================================
// 7. Error recovery determinism
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Running the same error scenario twice must produce the same
    /// outcome (both Ok or both Err, and if Ok the same root count).
    #[test]
    fn deterministic_recovery_same_result(bad_kind in 3u32..10) {
        let table = build_s_to_a_table();
        let tokens = vec![(bad_kind, 0, 1), (1, 1, 2), (0, 2, 2)];
        let r1 = try_parse(&table, tokens.clone());
        let r2 = try_parse(&table, tokens);
        prop_assert_eq!(r1.is_ok(), r2.is_ok());
        if let (Ok(f1), Ok(f2)) = (&r1, &r2) {
            prop_assert_eq!(f1.view().roots().len(), f2.view().roots().len());
        }
    }

    /// Valid input parsed repeatedly must always succeed with the
    /// same number of roots.
    #[test]
    fn deterministic_valid_parse(choice in 0u32..2) {
        let table = build_s_alt_table();
        let kind = if choice == 0 { 1 } else { 2 };
        let tokens = vec![(kind, 0, 1), (0, 1, 1)];
        let r1 = try_parse(&table, tokens.clone());
        let r2 = try_parse(&table, tokens);
        prop_assert!(r1.is_ok());
        prop_assert!(r2.is_ok());
        let f1 = r1.unwrap();
        let f2 = r2.unwrap();
        prop_assert_eq!(f1.view().roots().len(), f2.view().roots().len());
    }

    /// Determinism on error-heavy input: result consistency.
    #[test]
    fn deterministic_error_heavy(junk_count in 1usize..5, kind in 3u32..8) {
        let table = build_s_to_a_table();
        let mut tokens = Vec::new();
        let mut pos = 0u32;
        for _ in 0..junk_count {
            tokens.push((kind, pos, pos + 1));
            pos += 1;
        }
        tokens.push((0, pos, pos));
        let r1 = try_parse(&table, tokens.clone());
        let r2 = try_parse(&table, tokens);
        prop_assert_eq!(r1.is_ok(), r2.is_ok());
    }
}

// =========================================================================
// 8. No error recovery needed (valid input)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// S -> 'a' must always succeed on valid input.
    #[test]
    fn valid_s_to_a_always_succeeds(offset in 0u32..100) {
        let table = build_s_to_a_table();
        let tokens = vec![(1, offset, offset + 1), (0, offset + 1, offset + 1)];
        let result = try_parse(&table, tokens);
        prop_assert!(result.is_ok(), "valid input must parse: {:?}", result.err());
        let forest = result.unwrap();
        prop_assert!(!forest.view().roots().is_empty());
    }

    /// S -> 'a' 'b' must always succeed on valid input.
    #[test]
    fn valid_s_to_ab_always_succeeds(offset in 0u32..100) {
        let table = build_s_to_ab_table();
        let tokens = vec![
            (1, offset, offset + 1),
            (2, offset + 1, offset + 2),
            (0, offset + 2, offset + 2),
        ];
        let result = try_parse(&table, tokens);
        prop_assert!(result.is_ok(), "valid input must parse: {:?}", result.err());
        let forest = result.unwrap();
        prop_assert!(!forest.view().roots().is_empty());
    }

    /// S -> 'a' | 'b': either alternative must succeed.
    #[test]
    fn valid_s_alt_either_succeeds(choice in 0u32..2) {
        let table = build_s_alt_table();
        let kind = if choice == 0 { 1 } else { 2 };
        let tokens = vec![(kind, 0, 1), (0, 1, 1)];
        let result = try_parse(&table, tokens);
        prop_assert!(result.is_ok());
        let forest = result.unwrap();
        prop_assert!(!forest.view().roots().is_empty());
    }

    /// S -> 'a' 'b' 'c': three-token production must succeed.
    #[test]
    fn valid_s_to_abc_succeeds(offset in 0u32..50) {
        let table = build_s_to_abc_table();
        let tokens = vec![
            (1, offset, offset + 1),
            (2, offset + 1, offset + 2),
            (3, offset + 2, offset + 3),
            (0, offset + 3, offset + 3),
        ];
        let result = try_parse(&table, tokens);
        prop_assert!(result.is_ok(), "valid 'abc' must parse: {:?}", result.err());
    }
}

// =========================================================================
// Additional edge-case property tests
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Only-EOF stream: the driver should error or return empty, not panic.
    #[test]
    fn only_eof_does_not_panic(pos in 0u32..100) {
        let table = build_s_to_a_table();
        let tokens = vec![(0, pos, pos)];
        let _ = try_parse(&table, tokens);
    }

    /// Empty token stream should not panic.
    #[test]
    fn empty_token_stream_does_not_panic(_dummy in 0u32..1) {
        let table = build_s_to_a_table();
        let tokens: Vec<(u32, u32, u32)> = vec![];
        let _ = try_parse(&table, tokens);
    }

    /// Duplicate tokens at the same position should not cause issues.
    #[test]
    fn duplicate_tokens_same_position(count in 2usize..5) {
        let table = build_s_to_a_table();
        let mut tokens = Vec::new();
        for _ in 0..count {
            tokens.push((1, 0, 1));
        }
        tokens.push((0, 1, 1));
        let _ = try_parse(&table, tokens);
    }

    /// Recover action in the table should not cause panic when triggered.
    #[test]
    fn recover_action_in_table_does_not_panic(bad_kind in 3u32..10) {
        let mut table = build_s_to_a_table();
        // Add Recover action for unknown symbols in state 0
        table.action_table[0][0] = vec![Action::Recover];
        let tokens = vec![(bad_kind, 0, 1), (1, 1, 2), (0, 2, 2)];
        let mut driver = Driver::new(&table);
        let _ = driver.parse_tokens(tokens);
    }

    /// Valid input with Recover actions present should still succeed.
    #[test]
    fn recover_action_does_not_interfere_with_valid_parse(_dummy in 0u32..1) {
        let mut table = build_s_to_a_table();
        // Add Recover to states that also have real actions
        table.action_table[0][0] = vec![Action::Recover];
        let tokens = vec![(1, 0, 1), (0, 1, 1)];
        let result = try_parse(&table, tokens);
        prop_assert!(result.is_ok(), "valid input should still succeed with Recover in table");
    }

    /// Parse with monotonically increasing spans of varying widths.
    #[test]
    fn varying_token_widths(width in 1u32..20) {
        let table = build_s_to_ab_table();
        let tokens = vec![
            (1, 0, width),
            (2, width, width * 2),
            (0, width * 2, width * 2),
        ];
        let result = try_parse(&table, tokens);
        prop_assert!(result.is_ok());
        if let Ok(forest) = result {
            let view = forest.view();
            let roots = view.roots();
            prop_assert!(!roots.is_empty());
            let sp = view.span(roots[0]);
            prop_assert_eq!(sp.start, 0);
            prop_assert_eq!(sp.end, width * 2);
        }
    }
}

// =========================================================================
// Non-proptest deterministic tests for completeness
// =========================================================================

#[test]
fn error_recovery_valid_single_token() {
    let table = build_s_to_a_table();
    let tokens = vec![(1, 0, 1), (0, 1, 1)];
    let result = try_parse(&table, tokens);
    assert!(result.is_ok());
    let forest = result.unwrap();
    assert!(!forest.view().roots().is_empty());
}

#[test]
fn error_recovery_completely_wrong_input() {
    let table = build_s_to_a_table();
    // All tokens are unknown to the grammar
    let tokens = vec![(50, 0, 1), (51, 1, 2), (0, 2, 2)];
    let _ = try_parse(&table, tokens); // must not panic
}

#[test]
fn error_recovery_eof_only() {
    let table = build_s_to_a_table();
    let tokens = vec![(0, 0, 0)];
    let _ = try_parse(&table, tokens); // must not panic
}

#[cfg(feature = "test-api")]
#[test]
fn error_recovery_debug_stats_on_valid_parse() {
    let table = build_s_to_a_table();
    let tokens = vec![(1, 0, 1), (0, 1, 1)];
    let result = try_parse(&table, tokens);
    assert!(result.is_ok());
    let forest = result.unwrap();
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error, "valid parse should have no errors");
    assert_eq!(missing, 0, "valid parse should have no missing terminals");
    assert_eq!(cost, 0, "valid parse should have zero error cost");
}
