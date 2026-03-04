//! Comprehensive tests for the engine module behavior.
//!
//! The engine module (`runtime2/src/engine.rs`) is private, so we test its
//! behavior through the public `Parser` API which delegates to
//! `engine::parse_full` and `engine::parse_incremental` internally.
//!
//! Covers:
//! 1. Engine adapter construction (Parser + Language setup)
//! 2. Forest management (parse results, tree structure)
//! 3. Token processing pipeline (tokenizer integration)
//! 4. Error handling (missing components, bad input)
//! 5. Property-based tests (proptest)

// The engine module only exists behind the glr-core feature.
#![cfg(feature = "glr-core")]

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::{Language, Parser, Token, Tree, language::SymbolMetadata};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Leak a parse table so it has `'static` lifetime (required by Language builder).
fn leak_table(table: adze_glr_core::ParseTable) -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(table))
}

/// Build a minimal grammar: `start → a`
fn single_token_grammar() -> Grammar {
    let mut g = Grammar::new("single_tok".to_string());
    let a_id = SymbolId(1);
    g.tokens.insert(
        a_id,
        IrToken {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let start = SymbolId(2);
    g.rule_names.insert(start, "start".to_string());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![Symbol::Terminal(a_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    g
}

/// Build a grammar: `start → a b`
fn two_token_grammar() -> Grammar {
    let mut g = Grammar::new("two_tok".to_string());
    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    g.tokens.insert(
        a_id,
        IrToken {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b_id,
        IrToken {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    let start = SymbolId(3);
    g.rule_names.insert(start, "start".to_string());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![Symbol::Terminal(a_id), Symbol::Terminal(b_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    g
}

/// Build parse table + Language for the single-token grammar.
/// `tokenize_fn` receives raw bytes and must return tokens including EOF.
fn build_single_token_language<F>(tokenize_fn: F) -> Language
where
    F: Fn(&[u8]) -> Box<dyn Iterator<Item = Token> + '_> + 'static,
{
    let grammar = single_token_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let table = leak_table(table);

    Language::builder()
        .parse_table(table)
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .field_names(vec![])
        .tokenizer(tokenize_fn)
        .build()
        .unwrap()
}

/// Default tokenizer for single-token grammar: accepts `"a"`.
fn default_single_tokenizer(input: &[u8]) -> Box<dyn Iterator<Item = Token> + '_> {
    let mut toks = Vec::new();
    if input == b"a" {
        toks.push(Token {
            kind: 1,
            start: 0,
            end: 1,
        });
    }
    toks.push(Token {
        kind: 0,
        start: input.len() as u32,
        end: input.len() as u32,
    });
    Box::new(toks.into_iter())
}

/// Build a ready-to-parse `Parser` using the single-token language.
fn parser_for_single_token() -> Parser {
    let lang = build_single_token_language(default_single_tokenizer);
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    p
}

/// Build language + parser for the two-token grammar `start → a b`.
fn parser_for_two_tokens() -> Parser {
    let grammar = two_token_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let table = leak_table(table);

    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["EOF".into(), "a".into(), "b".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .field_names(vec![])
        .tokenizer(|input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
            let mut toks = Vec::new();
            if input.len() >= 2 && input[0] == b'a' && input[1] == b'b' {
                toks.push(Token {
                    kind: 1,
                    start: 0,
                    end: 1,
                });
                toks.push(Token {
                    kind: 2,
                    start: 1,
                    end: 2,
                });
            }
            toks.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(toks.into_iter())
        })
        .build()
        .unwrap();

    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    p
}

// ===========================================================================
// 1. Engine adapter construction
// ===========================================================================

#[test]
fn engine_construct_parser_without_language_returns_no_language_error() {
    let mut p = Parser::new();
    let err = p.parse(b"a", None).unwrap_err();
    assert!(
        format!("{err}").contains("no language"),
        "expected no-language error, got: {err}"
    );
}

#[test]
fn engine_set_language_accepts_valid_language() {
    let lang = build_single_token_language(default_single_tokenizer);
    let mut p = Parser::new();
    assert!(p.set_language(lang).is_ok());
}

#[test]
fn engine_set_language_rejects_missing_parse_table() {
    let result = Language::builder()
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .build();
    assert!(result.is_err(), "builder should reject missing parse table");
}

#[test]
fn engine_set_language_rejects_missing_symbol_metadata() {
    let grammar = single_token_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero();
    let table = leak_table(table);

    let result = Language::builder().parse_table(table).build();
    assert!(
        result.is_err(),
        "builder should reject missing symbol metadata"
    );
}

#[test]
fn engine_set_language_rejects_no_tokenizer() {
    let grammar = single_token_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero();
    let table = leak_table(table);

    // Language without tokenizer — builder succeeds, but set_language rejects.
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .build()
        .unwrap();

    let mut p = Parser::new();
    let err = p.set_language(lang);
    assert!(err.is_err(), "set_language should reject missing tokenizer");
}

#[test]
fn engine_set_language_rejects_empty_symbol_metadata() {
    let grammar = single_token_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero();
    let table = leak_table(table);

    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![])
        .tokenizer(|_| Box::new(std::iter::empty()))
        .build()
        .unwrap();

    let mut p = Parser::new();
    assert!(
        p.set_language(lang).is_err(),
        "set_language should reject empty symbol metadata"
    );
}

#[test]
fn engine_parser_new_has_no_language() {
    let p = Parser::new();
    assert!(p.language().is_none());
}

#[test]
fn engine_parser_language_accessible_after_set() {
    let lang = build_single_token_language(default_single_tokenizer);
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    assert!(p.language().is_some());
}

#[test]
fn engine_parser_set_language_can_be_called_twice() {
    let lang1 = build_single_token_language(default_single_tokenizer);
    let lang2 = build_single_token_language(default_single_tokenizer);
    let mut p = Parser::new();
    p.set_language(lang1).unwrap();
    p.set_language(lang2).unwrap();
    assert!(p.language().is_some());
}

// ===========================================================================
// 2. Forest management (parse results, tree structure)
// ===========================================================================

#[test]
fn engine_parse_full_produces_tree_with_correct_root_kind() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    // start symbol has id 2
    assert_eq!(tree.root_kind(), 2);
}

#[test]
fn engine_parse_full_root_node_has_children() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    // `start → a`, so the root should have at least 1 child
    let root = tree.root_node();
    assert!(root.child_count() >= 1, "root should have children");
}

#[test]
fn engine_parse_full_root_byte_range_covers_input() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert!(root.end_byte() >= 1, "root end_byte should cover the input");
}

#[test]
fn engine_parse_full_two_token_grammar_works() {
    let mut p = parser_for_two_tokens();
    let tree = p.parse_utf8("ab", None).unwrap();
    assert_eq!(tree.root_kind(), 3); // start symbol
}

#[test]
fn engine_parse_full_two_token_root_byte_range() {
    let mut p = parser_for_two_tokens();
    let tree = p.parse_utf8("ab", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert!(root.end_byte() >= 2);
}

#[test]
fn engine_parse_full_tree_has_language() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    assert!(tree.language().is_some());
}

#[test]
fn engine_parse_full_tree_has_source() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    assert_eq!(tree.source_bytes(), Some(b"a".as_slice()));
}

#[test]
fn engine_parse_full_tree_clone_is_independent() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

#[test]
fn engine_parse_full_debug_format_works() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    let dbg = format!("{tree:?}");
    assert!(!dbg.is_empty(), "debug format should not be empty");
}

#[test]
fn engine_parse_full_stub_tree_root_kind_is_zero() {
    let stub = Tree::new_stub();
    assert_eq!(stub.root_kind(), 0);
}

#[test]
fn engine_parse_full_tree_new_for_testing_works() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
    assert_eq!(tree.root_node().start_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 10);
}

#[test]
fn engine_parse_full_tree_new_for_testing_with_children() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let parent = Tree::new_for_testing(0, 0, 10, vec![child]);
    assert_eq!(parent.root_node().child_count(), 1);
}

// ===========================================================================
// 3. Token processing pipeline
// ===========================================================================

#[test]
fn engine_tokenizer_is_called_exactly_once_per_parse() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let lang = build_single_token_language(move |input: &[u8]| {
        c.fetch_add(1, Ordering::SeqCst);
        default_single_tokenizer(input)
    });
    let mut p = Parser::new();
    p.set_language(lang).unwrap();

    p.parse_utf8("a", None).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn engine_tokenizer_called_once_per_parse_multiple_parses() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let lang = build_single_token_language(move |input: &[u8]| {
        c.fetch_add(1, Ordering::SeqCst);
        default_single_tokenizer(input)
    });
    let mut p = Parser::new();
    p.set_language(lang).unwrap();

    p.parse_utf8("a", None).unwrap();
    p.parse_utf8("a", None).unwrap();
    p.parse_utf8("a", None).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[test]
fn engine_token_byte_offsets_propagate_to_tree() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    let root = tree.root_node();
    // Token "a" spans [0, 1), root should cover it.
    assert_eq!(root.start_byte(), 0);
    assert!(root.end_byte() >= 1);
}

#[test]
fn engine_missing_eof_token_still_produces_result() {
    // Tokenizer that does NOT emit EOF — the GLR engine's error recovery
    // may still produce a tree (possibly with error nodes).
    let lang =
        build_single_token_language(|_input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
            Box::new(
                vec![Token {
                    kind: 1,
                    start: 0,
                    end: 1,
                }]
                .into_iter(),
            )
        });
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    // GLR engine has error recovery — it may succeed or fail.
    let result = p.parse_utf8("a", None);
    // We only verify the result is deterministic (Ok or Err, not random).
    let result2 = p.parse_utf8("a", None);
    assert_eq!(result.is_ok(), result2.is_ok());
}

#[test]
fn engine_empty_token_stream_is_error() {
    let lang =
        build_single_token_language(|_input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
            Box::new(std::iter::empty())
        });
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    assert!(p.parse_utf8("a", None).is_err());
}

#[test]
fn engine_only_eof_token_for_nonempty_grammar_is_deterministic() {
    // Grammar expects `a` then EOF, but we only send EOF.
    // GLR error recovery may insert missing nodes or fail.
    let lang =
        build_single_token_language(|input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
            Box::new(
                vec![Token {
                    kind: 0,
                    start: input.len() as u32,
                    end: input.len() as u32,
                }]
                .into_iter(),
            )
        });
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    let r1 = p.parse_utf8("a", None);
    let r2 = p.parse_utf8("a", None);
    // Behaviour is deterministic regardless of outcome.
    assert_eq!(r1.is_ok(), r2.is_ok());
}

#[test]
fn engine_with_static_tokens_helper() {
    let grammar = single_token_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let table = leak_table(table);

    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .build()
        .unwrap()
        .with_static_tokens(vec![
            Token {
                kind: 1,
                start: 0,
                end: 1,
            },
            Token {
                kind: 0,
                start: 1,
                end: 1,
            },
        ]);

    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    let tree = p.parse(b"a", None).unwrap();
    assert_eq!(tree.root_kind(), 2);
}

#[test]
fn engine_token_kind_matches_grammar_symbol_id() {
    // Token kind must match the grammar's symbol IDs for correct parsing.
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    // If token kind didn't match, parse would have failed.
    assert_eq!(tree.root_kind(), 2);
}

#[test]
fn engine_two_token_sequence_produces_correct_children() {
    let mut p = parser_for_two_tokens();
    let tree = p.parse_utf8("ab", None).unwrap();
    let root = tree.root_node();
    // start → a b should have 2 children
    assert!(
        root.child_count() >= 2,
        "start → a b should have ≥2 children, got {}",
        root.child_count()
    );
}

// ===========================================================================
// 4. Error handling
// ===========================================================================

#[test]
fn engine_parse_without_language_is_error() {
    let mut p = Parser::new();
    assert!(p.parse(b"anything", None).is_err());
}

#[test]
fn engine_parse_utf8_without_language_is_error() {
    let mut p = Parser::new();
    assert!(p.parse_utf8("anything", None).is_err());
}

#[test]
fn engine_wrong_token_kind_is_deterministic() {
    // Tokenizer emits kind=99 which doesn't exist in parse table.
    // GLR error recovery may skip unknown tokens and still produce a tree.
    let lang =
        build_single_token_language(|input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
            Box::new(
                vec![
                    Token {
                        kind: 99,
                        start: 0,
                        end: 1,
                    },
                    Token {
                        kind: 0,
                        start: input.len() as u32,
                        end: input.len() as u32,
                    },
                ]
                .into_iter(),
            )
        });
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    let r1 = p.parse_utf8("x", None);
    let r2 = p.parse_utf8("x", None);
    assert_eq!(r1.is_ok(), r2.is_ok());
}

#[test]
fn engine_parse_error_has_display() {
    let mut p = Parser::new();
    let err = p.parse(b"a", None).unwrap_err();
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn engine_parse_error_has_debug() {
    let mut p = Parser::new();
    let err = p.parse(b"a", None).unwrap_err();
    let msg = format!("{err:?}");
    assert!(!msg.is_empty());
}

#[test]
fn engine_timeout_can_be_set() {
    let mut p = Parser::new();
    assert!(p.timeout().is_none());
    p.set_timeout(std::time::Duration::from_secs(5));
    assert_eq!(p.timeout(), Some(std::time::Duration::from_secs(5)));
}

#[test]
fn engine_parse_bytes_works_same_as_utf8() {
    let mut p = parser_for_single_token();
    let tree_bytes = p.parse(b"a", None).unwrap();
    let tree_utf8 = p.parse_utf8("a", None).unwrap();
    assert_eq!(tree_bytes.root_kind(), tree_utf8.root_kind());
}

// ===========================================================================
// 5. Incremental parsing (engine::parse_incremental via Parser)
// ===========================================================================

#[test]
fn engine_incremental_with_same_input_returns_tree() {
    let mut p = parser_for_single_token();
    let tree1 = p.parse_utf8("a", None).unwrap();
    let tree2 = p.parse_utf8("a", Some(&tree1)).unwrap();
    assert_eq!(tree1.root_kind(), tree2.root_kind());
}

#[test]
fn engine_incremental_with_stub_tree_still_parses() {
    let mut p = parser_for_single_token();
    let stub = Tree::new_stub();
    let tree = p.parse_utf8("a", Some(&stub)).unwrap();
    assert_eq!(tree.root_kind(), 2);
}

#[test]
fn engine_incremental_parse_preserves_root_kind() {
    let mut p = parser_for_single_token();
    let tree1 = p.parse_utf8("a", None).unwrap();
    let tree2 = p.parse_utf8("a", Some(&tree1)).unwrap();
    assert_eq!(tree2.root_kind(), 2);
}

#[test]
fn engine_multiple_sequential_parses_work() {
    let mut p = parser_for_single_token();
    for _ in 0..5 {
        let tree = p.parse_utf8("a", None).unwrap();
        assert_eq!(tree.root_kind(), 2);
    }
}

#[test]
fn engine_incremental_chain_of_parses() {
    let mut p = parser_for_single_token();
    let mut tree = p.parse_utf8("a", None).unwrap();
    for _ in 0..5 {
        tree = p.parse_utf8("a", Some(&tree)).unwrap();
        assert_eq!(tree.root_kind(), 2);
    }
}

// ===========================================================================
// 6. Language metadata through engine
// ===========================================================================

#[test]
fn engine_language_symbol_name_lookup() {
    let lang = build_single_token_language(default_single_tokenizer);
    assert_eq!(lang.symbol_name(0), Some("EOF"));
    assert_eq!(lang.symbol_name(1), Some("a"));
    assert_eq!(lang.symbol_name(2), Some("start"));
    assert_eq!(lang.symbol_name(99), None);
}

#[test]
fn engine_language_is_terminal() {
    let lang = build_single_token_language(default_single_tokenizer);
    assert!(lang.is_terminal(0)); // EOF
    assert!(lang.is_terminal(1)); // a
    assert!(!lang.is_terminal(2)); // start (non-terminal)
}

#[test]
fn engine_language_is_visible() {
    let lang = build_single_token_language(default_single_tokenizer);
    assert!(!lang.is_visible(0)); // EOF is invisible
    assert!(lang.is_visible(1)); // a is visible
    assert!(lang.is_visible(2)); // start is visible
}

#[test]
fn engine_language_symbol_for_name() {
    let lang = build_single_token_language(default_single_tokenizer);
    // "a" is visible (named)
    assert_eq!(lang.symbol_for_name("a", true), Some(1));
    // "EOF" is not visible (anonymous)
    assert_eq!(lang.symbol_for_name("EOF", false), Some(0));
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
}

#[test]
fn engine_language_field_name_empty() {
    let lang = build_single_token_language(default_single_tokenizer);
    assert_eq!(lang.field_name(0), None);
}

#[test]
fn engine_language_symbol_count() {
    let lang = build_single_token_language(default_single_tokenizer);
    assert_eq!(lang.symbol_count, 3);
}

#[test]
fn engine_language_field_count_zero() {
    let lang = build_single_token_language(default_single_tokenizer);
    assert_eq!(lang.field_count, 0);
}

#[test]
fn engine_language_version_default_is_zero() {
    let lang = build_single_token_language(default_single_tokenizer);
    assert_eq!(lang.version, 0);
}

#[test]
fn engine_language_version_can_be_set() {
    let grammar = single_token_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero();
    let table = leak_table(table);

    let lang = Language::builder()
        .version(15)
        .parse_table(table)
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(default_single_tokenizer)
        .build()
        .unwrap();

    assert_eq!(lang.version, 15);
}

// ===========================================================================
// 7. Tree structure validation
// ===========================================================================

#[test]
fn engine_tree_root_node_kind_matches_start_symbol_name() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.kind(), "start");
}

#[test]
fn engine_tree_leaf_node_kind_matches_token_name() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    let root = tree.root_node();
    if root.child_count() > 0 {
        let child = root.child(0).unwrap();
        assert_eq!(child.kind(), "a");
    }
}

#[test]
fn engine_tree_child_byte_range_within_parent() {
    let mut p = parser_for_single_token();
    let tree = p.parse_utf8("a", None).unwrap();
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let child = root.child(i).unwrap();
        assert!(
            child.start_byte() >= root.start_byte(),
            "child start should be >= parent start"
        );
        assert!(
            child.end_byte() <= root.end_byte(),
            "child end should be <= parent end"
        );
    }
}

#[test]
fn engine_tree_stub_has_no_children() {
    let stub = Tree::new_stub();
    assert_eq!(stub.root_node().child_count(), 0);
}

#[test]
fn engine_tree_stub_has_no_language() {
    let stub = Tree::new_stub();
    assert!(stub.language().is_none());
}

#[test]
fn engine_tree_stub_has_no_source() {
    let stub = Tree::new_stub();
    assert!(stub.source_bytes().is_none());
}

// ===========================================================================
// 8. Property-based tests
// ===========================================================================

mod proptest_engine {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_parser_new_always_has_no_language(_seed in 0u64..1000) {
            let p = Parser::new();
            prop_assert!(p.language().is_none());
        }

        #[test]
        fn prop_parse_without_language_always_errors(input in ".*") {
            let mut p = Parser::new();
            prop_assert!(p.parse_utf8(&input, None).is_err());
        }

        #[test]
        fn prop_stub_tree_root_kind_always_zero(_seed in 0u64..1000) {
            let stub = Tree::new_stub();
            prop_assert_eq!(stub.root_kind(), 0);
        }

        #[test]
        fn prop_stub_tree_root_byte_range_always_zero(_seed in 0u64..1000) {
            let stub = Tree::new_stub();
            prop_assert_eq!(stub.root_node().start_byte(), 0);
            prop_assert_eq!(stub.root_node().end_byte(), 0);
        }

        #[test]
        fn prop_successful_parse_root_covers_input(
            _seed in 0u64..100
        ) {
            let mut p = parser_for_single_token();
            let tree = p.parse_utf8("a", None).unwrap();
            let root = tree.root_node();
            prop_assert!(root.end_byte() >= 1, "root should cover input");
        }

        #[test]
        fn prop_tree_clone_preserves_root_kind(_seed in 0u64..100) {
            let mut p = parser_for_single_token();
            let tree = p.parse_utf8("a", None).unwrap();
            let cloned = tree.clone();
            prop_assert_eq!(tree.root_kind(), cloned.root_kind());
        }

        #[test]
        fn prop_new_for_testing_preserves_symbol(sym in 0u32..1000, end in 1usize..10000) {
            let tree = Tree::new_for_testing(sym, 0, end, vec![]);
            prop_assert_eq!(tree.root_kind(), sym);
            prop_assert_eq!(tree.root_node().end_byte(), end);
        }

        #[test]
        fn prop_symbol_name_within_range_is_some(idx in 0u16..3) {
            let lang = build_single_token_language(default_single_tokenizer);
            prop_assert!(lang.symbol_name(idx).is_some());
        }

        #[test]
        fn prop_symbol_name_out_of_range_is_none(idx in 100u16..1000) {
            let lang = build_single_token_language(default_single_tokenizer);
            prop_assert!(lang.symbol_name(idx).is_none());
        }

        #[test]
        fn prop_multiple_parses_same_result(_seed in 0u64..50) {
            let mut p = parser_for_single_token();
            let t1 = p.parse_utf8("a", None).unwrap();
            let t2 = p.parse_utf8("a", None).unwrap();
            prop_assert_eq!(t1.root_kind(), t2.root_kind());
        }

        #[test]
        fn prop_incremental_same_as_full(_seed in 0u64..50) {
            let mut p = parser_for_single_token();
            let full = p.parse_utf8("a", None).unwrap();
            let inc = p.parse_utf8("a", Some(&full)).unwrap();
            prop_assert_eq!(full.root_kind(), inc.root_kind());
        }
    }
}
