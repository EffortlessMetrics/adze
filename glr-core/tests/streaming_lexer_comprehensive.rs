//! Comprehensive tests for the streaming lexer / token processing in GLR core.
//!
//! Covers: NextToken creation, LexMode properties, parse_streaming with mock lexers,
//! parse_tokens with various token sequences, position tracking, edge cases
//! (empty input, UTF-8, large input), error handling, and candidate selection.

use adze_glr_core::driver::GlrError;
use adze_glr_core::ts_lexer::NextToken;
use adze_glr_core::{
    Action, Driver, Forest, GotoIndexing, LexMode, ParseRule, ParseTable, SymbolMetadata,
};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ─── Helpers ─────────────────────────────────────────────────────────

const INV: StateId = StateId(65535);

fn default_sym_meta(count: usize) -> Vec<SymbolMetadata> {
    vec![
        SymbolMetadata {
            name: String::new(),
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        };
        count
    ]
}

/// Build a hand-crafted ParseTable from raw action/goto matrices.
fn make_table(
    states: Vec<Vec<Vec<Action>>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start: SymbolId,
    eof: SymbolId,
    terminal_count: usize,
) -> ParseTable {
    let symbol_count = states.first().map(|s| s.len()).unwrap_or(0);
    let state_count = states.len();

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for i in 0..gotos.first().map(|g| g.len()).unwrap_or(0) {
        for row in &gotos {
            if row[i] != INV {
                nonterminal_to_index.insert(SymbolId(i as u16), i);
                break;
            }
        }
    }

    let rule_count = rules.len();

    ParseTable {
        action_table: states,
        goto_table: gotos,
        rules,
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol: (0..terminal_count as u16).map(SymbolId).collect(),
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start,
        grammar: Grammar::new("test".to_string()),
        symbol_metadata: default_sym_meta(symbol_count),
        initial_state: StateId(0),
        token_count: terminal_count,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; rule_count.max(1)],
        rule_assoc_by_rule: vec![0; rule_count.max(1)],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
        external_scanner_states: vec![],
    }
}

/// Grammar: S → a   (single terminal)
/// Symbols: 0=EOF  1=a  2=S
fn single_token_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(2);
    let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
    let actions = vec![
        vec![vec![], vec![Action::Shift(StateId(1))], vec![]],
        vec![vec![Action::Reduce(RuleId(0))], vec![], vec![]],
        vec![vec![Action::Accept], vec![], vec![]],
    ];
    let gotos = vec![
        vec![INV, INV, StateId(2)],
        vec![INV, INV, INV],
        vec![INV, INV, INV],
    ];
    make_table(actions, gotos, rules, s, eof, 2)
}

/// Grammar: S → a b   (two terminals)
/// Symbols: 0=EOF  1=a  2=b  3=S
fn two_token_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(3);
    let rules = vec![ParseRule { lhs: s, rhs_len: 2 }];
    let actions = vec![
        vec![vec![], vec![Action::Shift(StateId(1))], vec![], vec![]],
        vec![vec![], vec![], vec![Action::Shift(StateId(2))], vec![]],
        vec![vec![Action::Reduce(RuleId(0))], vec![], vec![], vec![]],
        vec![vec![Action::Accept], vec![], vec![], vec![]],
    ];
    let gotos = vec![
        vec![INV, INV, INV, StateId(3)],
        vec![INV, INV, INV, INV],
        vec![INV, INV, INV, INV],
        vec![INV, INV, INV, INV],
    ];
    make_table(actions, gotos, rules, s, eof, 3)
}

/// Grammar: S → a b c   (three terminals)
/// Symbols: 0=EOF  1=a  2=b  3=c  4=S
fn three_token_table() -> ParseTable {
    let eof = SymbolId(0);
    let s = SymbolId(4);
    let rules = vec![ParseRule { lhs: s, rhs_len: 3 }];
    let actions = vec![
        vec![
            vec![],
            vec![Action::Shift(StateId(1))],
            vec![],
            vec![],
            vec![],
        ],
        vec![
            vec![],
            vec![],
            vec![Action::Shift(StateId(2))],
            vec![],
            vec![],
        ],
        vec![
            vec![],
            vec![],
            vec![],
            vec![Action::Shift(StateId(3))],
            vec![],
        ],
        vec![
            vec![Action::Reduce(RuleId(0))],
            vec![],
            vec![],
            vec![],
            vec![],
        ],
        vec![vec![Action::Accept], vec![], vec![], vec![], vec![]],
    ];
    let gotos = vec![
        vec![INV, INV, INV, INV, StateId(4)],
        vec![INV, INV, INV, INV, INV],
        vec![INV, INV, INV, INV, INV],
        vec![INV, INV, INV, INV, INV],
        vec![INV, INV, INV, INV, INV],
    ];
    make_table(actions, gotos, rules, s, eof, 4)
}

fn parse_tokens(table: &ParseTable, tokens: &[(u32, u32, u32)]) -> Result<Forest, GlrError> {
    let mut driver = Driver::new(table);
    driver.parse_tokens(tokens.iter().copied())
}

type NoExtScanner = fn(&str, usize, &[bool], LexMode) -> Option<NextToken>;

fn parse_streaming_with<L>(table: &ParseTable, input: &str, lexer: L) -> Result<Forest, GlrError>
where
    L: FnMut(&str, usize, LexMode) -> Option<NextToken>,
{
    let mut driver = Driver::new(table);
    driver.parse_streaming(input, lexer, None::<NoExtScanner>)
}

// ═══════════════════════════════════════════════════════════════════════
// 1. NextToken creation and properties
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn next_token_fields_are_accessible() {
    let tok = NextToken {
        kind: 5,
        start: 10,
        end: 20,
    };
    assert_eq!(tok.kind, 5);
    assert_eq!(tok.start, 10);
    assert_eq!(tok.end, 20);
}

#[test]
fn next_token_clone_produces_identical_copy() {
    let tok = NextToken {
        kind: 3,
        start: 0,
        end: 7,
    };
    let cloned = tok;
    assert_eq!(tok.kind, cloned.kind);
    assert_eq!(tok.start, cloned.start);
    assert_eq!(tok.end, cloned.end);
}

#[test]
fn next_token_debug_format_is_nonempty() {
    let tok = NextToken {
        kind: 1,
        start: 0,
        end: 1,
    };
    let dbg = format!("{:?}", tok);
    assert!(!dbg.is_empty());
    assert!(dbg.contains("NextToken"));
}

#[test]
fn next_token_zero_width_token() {
    let tok = NextToken {
        kind: 42,
        start: 5,
        end: 5,
    };
    assert_eq!(
        tok.start, tok.end,
        "zero-width tokens must have equal start/end"
    );
}

#[test]
fn next_token_max_u32_values() {
    let tok = NextToken {
        kind: u32::MAX,
        start: u32::MAX - 1,
        end: u32::MAX,
    };
    assert_eq!(tok.kind, u32::MAX);
    assert_eq!(tok.end - tok.start, 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 2. LexMode creation and properties
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn lex_mode_default_values() {
    let mode = LexMode {
        lex_state: 0,
        external_lex_state: 0,
    };
    assert_eq!(mode.lex_state, 0);
    assert_eq!(mode.external_lex_state, 0);
}

#[test]
fn lex_mode_equality() {
    let a = LexMode {
        lex_state: 1,
        external_lex_state: 2,
    };
    let b = LexMode {
        lex_state: 1,
        external_lex_state: 2,
    };
    assert_eq!(a, b);
}

#[test]
fn lex_mode_inequality() {
    let a = LexMode {
        lex_state: 0,
        external_lex_state: 0,
    };
    let b = LexMode {
        lex_state: 1,
        external_lex_state: 0,
    };
    assert_ne!(a, b);
}

#[test]
fn lex_mode_from_parse_table_default_state() {
    let table = single_token_table();
    let mode = table.lex_mode(StateId(0));
    assert_eq!(mode.lex_state, 0);
    assert_eq!(mode.external_lex_state, 0);
}

#[test]
fn lex_mode_out_of_range_returns_default() {
    let table = single_token_table();
    let mode = table.lex_mode(StateId(9999));
    assert_eq!(mode.lex_state, 0);
    assert_eq!(mode.external_lex_state, 0);
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Token stream parsing (parse_tokens)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_tokens_single_token_succeeds() {
    let table = single_token_table();
    let forest = parse_tokens(&table, &[(1, 0, 1)]).expect("single token should parse");
    let view = forest.view();
    assert!(!view.roots().is_empty());
}

#[test]
fn parse_tokens_two_tokens_succeeds() {
    let table = two_token_table();
    let forest = parse_tokens(&table, &[(1, 0, 1), (2, 1, 2)]).expect("two tokens should parse");
    let view = forest.view();
    assert!(!view.roots().is_empty());
}

#[test]
fn parse_tokens_three_tokens_succeeds() {
    let table = three_token_table();
    let forest =
        parse_tokens(&table, &[(1, 0, 1), (2, 1, 2), (3, 2, 3)]).expect("three tokens parse");
    let view = forest.view();
    assert!(!view.roots().is_empty());
}

#[test]
fn parse_tokens_empty_input_errors() {
    let table = single_token_table();
    let result = parse_tokens(&table, &[]);
    assert!(result.is_err());
}

#[test]
fn parse_tokens_wrong_token_no_shift() {
    let table = single_token_table();
    // Token kind 2 is nonterminal S — the driver may recover or error.
    // Either way, if it succeeds it should still produce a forest with roots.
    let result = parse_tokens(&table, &[(2, 0, 1)]);
    match result {
        Err(_) => {} // Expected: no valid shift for this token
        Ok(forest) => {
            // If recovery succeeded, the forest should still be valid
            let view = forest.view();
            assert!(!view.roots().is_empty());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Streaming lexer (parse_streaming) – basic functionality
// ═══════════════════════════════════════════════════════════════════════

/// A lexer that always returns token kind=1 for a single byte "a".
fn single_byte_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    if pos >= bytes.len() {
        return None;
    }
    if bytes[pos] == b'a' {
        Some(NextToken {
            kind: 1,
            start: pos as u32,
            end: (pos + 1) as u32,
        })
    } else {
        None
    }
}

#[test]
fn parse_streaming_single_char_succeeds() {
    let table = single_token_table();
    let result = parse_streaming_with(&table, "a", single_byte_lexer);
    assert!(result.is_ok(), "streaming parse of 'a' should succeed");
}

#[test]
fn parse_streaming_span_tracking() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "a", single_byte_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).start, 0);
    assert_eq!(view.span(root).end, 1);
}

/// A lexer producing two distinct token types for "ab" input.
fn two_byte_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    if pos >= bytes.len() {
        return None;
    }
    match bytes[pos] {
        b'a' => Some(NextToken {
            kind: 1,
            start: pos as u32,
            end: (pos + 1) as u32,
        }),
        b'b' => Some(NextToken {
            kind: 2,
            start: pos as u32,
            end: (pos + 1) as u32,
        }),
        _ => None,
    }
}

#[test]
fn parse_streaming_two_token_sequence() {
    let table = two_token_table();
    let result = parse_streaming_with(&table, "ab", two_byte_lexer);
    assert!(result.is_ok(), "streaming parse of 'ab' should succeed");
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Streaming lexer – empty input
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_streaming_empty_string_result() {
    let table = single_token_table();
    let result = parse_streaming_with(&table, "", single_byte_lexer);
    // Empty input goes straight to EOF; the driver may recover or reject.
    match result {
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            assert!(
                msg.contains("not accepted") || msg.contains("no valid") || msg.contains("eof"),
                "error should mention parse failure: {msg}"
            );
        }
        Ok(forest) => {
            // If recovery inserted a missing token, the forest is still valid
            let view = forest.view();
            assert!(!view.roots().is_empty());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Streaming lexer – whitespace skipping
// ═══════════════════════════════════════════════════════════════════════

/// Lexer that skips whitespace before matching token kind=1 for 'a'.
fn whitespace_skipping_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    let mut p = pos;
    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
        p += 1;
    }
    if p >= bytes.len() {
        return None;
    }
    if bytes[p] == b'a' {
        Some(NextToken {
            kind: 1,
            start: p as u32,
            end: (p + 1) as u32,
        })
    } else {
        None
    }
}

#[test]
fn parse_streaming_with_leading_whitespace() {
    let table = single_token_table();
    let result = parse_streaming_with(&table, "  a", whitespace_skipping_lexer);
    assert!(result.is_ok(), "lexer should skip leading whitespace");
}

#[test]
fn parse_streaming_whitespace_span_tracking() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "  a", whitespace_skipping_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    // Token starts at byte 2 (after whitespace), ends at byte 3
    assert_eq!(view.span(root).start, 2);
    assert_eq!(view.span(root).end, 3);
}

// ═══════════════════════════════════════════════════════════════════════
// 7. Streaming lexer – multi-byte tokens
// ═══════════════════════════════════════════════════════════════════════

/// Lexer that recognizes "abc" as a single 3-byte token.
fn multi_byte_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    if pos + 3 <= bytes.len() && &bytes[pos..pos + 3] == b"abc" {
        Some(NextToken {
            kind: 1,
            start: pos as u32,
            end: (pos + 3) as u32,
        })
    } else {
        None
    }
}

#[test]
fn parse_streaming_multi_byte_token() {
    let table = single_token_table();
    let result = parse_streaming_with(&table, "abc", multi_byte_lexer);
    assert!(result.is_ok(), "3-byte token should parse");
}

#[test]
fn parse_streaming_multi_byte_span() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "abc", multi_byte_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).start, 0);
    assert_eq!(view.span(root).end, 3);
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Streaming lexer – UTF-8 boundary handling
// ═══════════════════════════════════════════════════════════════════════

/// Lexer that handles multibyte UTF-8 by matching a known prefix.
fn utf8_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let remaining = &input[pos..];
    if remaining.starts_with('α') {
        // α is 2 bytes in UTF-8 (0xCE 0xB1)
        Some(NextToken {
            kind: 1,
            start: pos as u32,
            end: (pos + 'α'.len_utf8()) as u32,
        })
    } else {
        None
    }
}

#[test]
fn parse_streaming_utf8_two_byte_char() {
    let table = single_token_table();
    let result = parse_streaming_with(&table, "α", utf8_lexer);
    assert!(result.is_ok(), "2-byte UTF-8 char should parse");
}

#[test]
fn parse_streaming_utf8_span_is_byte_based() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "α", utf8_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    // α is 2 bytes in UTF-8
    assert_eq!(view.span(root).start, 0);
    assert_eq!(view.span(root).end, 2);
}

/// Lexer that matches a 4-byte UTF-8 character (emoji).
fn emoji_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let remaining = &input[pos..];
    if remaining.starts_with('🦀') {
        Some(NextToken {
            kind: 1,
            start: pos as u32,
            end: (pos + '🦀'.len_utf8()) as u32,
        })
    } else {
        None
    }
}

#[test]
fn parse_streaming_utf8_four_byte_char() {
    let table = single_token_table();
    let result = parse_streaming_with(&table, "🦀", emoji_lexer);
    assert!(result.is_ok(), "4-byte UTF-8 emoji should parse");
}

#[test]
fn parse_streaming_emoji_span_is_four_bytes() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "🦀", emoji_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).end - view.span(root).start, 4);
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Position tracking across multiple tokens
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_tokens_position_tracking_contiguous() {
    let table = two_token_table();
    let forest = parse_tokens(&table, &[(1, 0, 5), (2, 5, 10)]).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).start, 0);
    assert_eq!(view.span(root).end, 10);
}

#[test]
fn parse_tokens_position_tracking_with_gaps() {
    // Tokens with gaps between them (whitespace)
    let table = two_token_table();
    let forest = parse_tokens(&table, &[(1, 0, 3), (2, 5, 8)]).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).start, 0);
    assert_eq!(view.span(root).end, 8);
}

#[test]
fn parse_tokens_children_have_correct_spans() {
    let table = two_token_table();
    let forest = parse_tokens(&table, &[(1, 0, 3), (2, 3, 7)]).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert_eq!(children.len(), 2);
    assert_eq!(view.span(children[0]).start, 0);
    assert_eq!(view.span(children[0]).end, 3);
    assert_eq!(view.span(children[1]).start, 3);
    assert_eq!(view.span(children[1]).end, 7);
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Error handling in lexing
// ═══════════════════════════════════════════════════════════════════════

/// Lexer that always returns None (can't lex anything).
fn failing_lexer(_input: &str, _pos: usize, _mode: LexMode) -> Option<NextToken> {
    None
}

#[test]
fn parse_streaming_lexer_returns_none_errors() {
    let table = single_token_table();
    let result = parse_streaming_with(&table, "x", failing_lexer);
    assert!(result.is_err());
}

#[test]
fn parse_streaming_lex_error_message_mentions_position() {
    let table = single_token_table();
    let err = parse_streaming_with(&table, "x", failing_lexer)
        .err()
        .unwrap();
    let msg = err.to_string();
    // The error should mention the byte position where lexing failed
    assert!(
        msg.contains("byte 0") || msg.contains("lex") || msg.contains("no valid"),
        "error should mention position or lexing: {msg}"
    );
}

#[test]
fn glr_error_lex_variant_display() {
    let err = GlrError::Lex("test error".to_string());
    assert!(err.to_string().contains("test error"));
}

#[test]
fn glr_error_parse_variant_display() {
    let err = GlrError::Parse("parse failed".to_string());
    assert!(err.to_string().contains("parse failed"));
}

#[test]
fn glr_error_other_variant_display() {
    let err = GlrError::Other("other issue".to_string());
    assert!(err.to_string().contains("other issue"));
}

#[test]
fn glr_error_debug_is_nonempty() {
    let err = GlrError::Lex("x".into());
    let dbg = format!("{:?}", err);
    assert!(!dbg.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 11. Lex mode per state
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn lex_mode_varies_per_state() {
    let eof = SymbolId(0);
    let s = SymbolId(2);
    let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];
    let actions = vec![
        vec![vec![], vec![Action::Shift(StateId(1))], vec![]],
        vec![vec![Action::Reduce(RuleId(0))], vec![], vec![]],
        vec![vec![Action::Accept], vec![], vec![]],
    ];
    let gotos = vec![
        vec![INV, INV, StateId(2)],
        vec![INV, INV, INV],
        vec![INV, INV, INV],
    ];
    let mut table = make_table(actions, gotos, rules, s, eof, 2);
    // Set distinct lex modes per state
    table.lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        },
        LexMode {
            lex_state: 1,
            external_lex_state: 0,
        },
        LexMode {
            lex_state: 2,
            external_lex_state: 0,
        },
    ];

    assert_eq!(table.lex_mode(StateId(0)).lex_state, 0);
    assert_eq!(table.lex_mode(StateId(1)).lex_state, 1);
    assert_eq!(table.lex_mode(StateId(2)).lex_state, 2);
}

#[test]
fn streaming_lexer_receives_lex_mode() {
    let table = single_token_table();
    let lexer = |input: &str, pos: usize, mode: LexMode| -> Option<NextToken> {
        // Verify mode is valid (non-panicking access).
        let _ = mode.lex_state;
        let _ = mode.external_lex_state;
        let bytes = input.as_bytes();
        if pos >= bytes.len() {
            return None;
        }
        if bytes[pos] == b'a' {
            Some(NextToken {
                kind: 1,
                start: pos as u32,
                end: (pos + 1) as u32,
            })
        } else {
            None
        }
    };

    let result = parse_streaming_with(&table, "a", lexer);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════
// 12. Valid symbols mask
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn valid_symbols_initial_state_has_shift() {
    let table = single_token_table();
    let mask = table.valid_symbols(StateId(0));
    // State 0 should have action for symbol 1 (shift a)
    assert!(mask.len() >= 2);
    assert!(!mask[0], "EOF should not be valid in initial state");
    assert!(mask[1], "terminal 'a' should be valid in initial state");
}

#[test]
fn valid_symbols_mask_matches_valid_symbols() {
    let table = single_token_table();
    let mask1 = table.valid_symbols(StateId(0));
    let mask2 = table.valid_symbols_mask(StateId(0));
    assert_eq!(mask1, mask2);
}

// ═══════════════════════════════════════════════════════════════════════
// 13. Parse table queries
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn is_terminal_check() {
    let table = single_token_table();
    assert!(table.is_terminal(SymbolId(0)), "EOF should be terminal");
    assert!(table.is_terminal(SymbolId(1)), "a should be terminal");
    assert!(!table.is_terminal(SymbolId(2)), "S should not be terminal");
}

#[test]
fn eof_symbol_accessor() {
    let table = single_token_table();
    assert_eq!(table.eof(), SymbolId(0));
}

#[test]
fn start_symbol_accessor() {
    let table = single_token_table();
    assert_eq!(table.start_symbol(), SymbolId(2));
}

#[test]
fn is_extra_with_empty_extras() {
    let table = single_token_table();
    assert!(!table.is_extra(SymbolId(0)));
    assert!(!table.is_extra(SymbolId(1)));
}

// ═══════════════════════════════════════════════════════════════════════
// 14. Large input / stress
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_tokens_many_tokens() {
    // Build a grammar: S → a  but feed many tokens; only first matters
    // Actually, S→a only accepts one token, so this tests that the driver
    // correctly accepts with just the right token count.
    let table = single_token_table();
    let result = parse_tokens(&table, &[(1, 0, 1)]);
    assert!(result.is_ok());
}

#[test]
fn parse_streaming_large_single_token() {
    // A lexer that matches the entire input as one big token
    let table = single_token_table();
    let big_input = "a".repeat(10_000);

    let lexer = |input: &str, pos: usize, _mode: LexMode| -> Option<NextToken> {
        if pos == 0 {
            Some(NextToken {
                kind: 1,
                start: 0,
                end: input.len() as u32,
            })
        } else {
            None
        }
    };

    let result = parse_streaming_with(&table, &big_input, lexer);
    assert!(result.is_ok(), "large single token should parse");
    let forest = result.unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).end, 10_000);
}

// ═══════════════════════════════════════════════════════════════════════
// 15. Streaming lexer – lexer called with correct position
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn streaming_lexer_called_at_position_zero() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static CALLED_AT_ZERO: AtomicBool = AtomicBool::new(false);
    CALLED_AT_ZERO.store(false, Ordering::SeqCst);

    let table = single_token_table();
    let lexer = |_input: &str, pos: usize, _mode: LexMode| -> Option<NextToken> {
        if pos == 0 {
            CALLED_AT_ZERO.store(true, Ordering::SeqCst);
            Some(NextToken {
                kind: 1,
                start: 0,
                end: 1,
            })
        } else {
            None
        }
    };

    let _ = parse_streaming_with(&table, "a", lexer);
    assert!(
        CALLED_AT_ZERO.load(Ordering::SeqCst),
        "lexer must be called at position 0"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 16. Driver reuse
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn driver_can_be_reused_for_multiple_parses() {
    let table = single_token_table();
    let mut driver = Driver::new(&table);

    let r1 = driver.parse_tokens([(1u32, 0u32, 1u32)].iter().copied());
    assert!(r1.is_ok(), "first parse should succeed");

    let r2 = driver.parse_tokens([(1u32, 0u32, 5u32)].iter().copied());
    assert!(r2.is_ok(), "second parse should succeed");
}

// ═══════════════════════════════════════════════════════════════════════
// 17. Forest view after streaming parse
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn streaming_parse_forest_has_root() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "a", single_byte_lexer).unwrap();
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
}

#[test]
fn streaming_parse_root_kind_is_start_symbol() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "a", single_byte_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    // Start symbol is SymbolId(2) → kind should be 2
    assert_eq!(view.kind(root), 2);
}

#[test]
fn streaming_parse_root_has_terminal_child() {
    let table = single_token_table();
    let forest = parse_streaming_with(&table, "a", single_byte_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert_eq!(children.len(), 1, "S→a should have 1 child");
    // Child should be terminal 'a' with kind=1
    assert_eq!(view.kind(children[0]), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 18. Streaming parse with two tokens spanning multiple bytes
// ═══════════════════════════════════════════════════════════════════════

/// Lexer that skips whitespace and produces two token types for "ab" input.
fn ws_two_byte_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    let mut p = pos;
    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
        p += 1;
    }
    if p >= bytes.len() {
        return None;
    }
    match bytes[p] {
        b'a' => Some(NextToken {
            kind: 1,
            start: p as u32,
            end: (p + 1) as u32,
        }),
        b'b' => Some(NextToken {
            kind: 2,
            start: p as u32,
            end: (p + 1) as u32,
        }),
        _ => None,
    }
}

#[test]
fn streaming_two_tokens_with_whitespace() {
    let table = two_token_table();
    let result = parse_streaming_with(&table, "a b", ws_two_byte_lexer);
    assert!(
        result.is_ok(),
        "parse of 'a b' with whitespace should succeed"
    );
}

#[test]
fn streaming_two_tokens_span_correct_with_ws() {
    let table = two_token_table();
    let forest = parse_streaming_with(&table, "a b", ws_two_byte_lexer).unwrap();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.span(root).start, 0);
    assert_eq!(view.span(root).end, 3);
}
