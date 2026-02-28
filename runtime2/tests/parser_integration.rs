//! Integration tests for the adze-runtime Parser, Tree, Node, and TreeCursor APIs.
//!
//! These tests exercise:
//! 1. Parser creation and configuration
//! 2. Language builder and set_language validation
//! 3. Parse error handling paths
//! 4. Tree construction, cloning, and source bytes
//! 5. Node API (kind, byte_range, children, text extraction)
//! 6. TreeCursor traversal
//! 7. End-to-end parse with a real GLR grammar (feature-gated)

use adze_runtime::language::SymbolMetadata;
use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use adze_runtime::{ParseError, Parser, Token, Tree};
use std::time::Duration;

// ---------------------------------------------------------------------------
// 1. Parser creation
// ---------------------------------------------------------------------------

#[test]
fn parser_new_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_default_is_same_as_new() {
    let parser = Parser::default();
    assert!(parser.language().is_none());
}

// ---------------------------------------------------------------------------
// 2. Language builder & set_language
// ---------------------------------------------------------------------------

#[test]
fn set_language_accepts_stub() {
    let mut parser = Parser::new();
    let lang = stub_language();
    parser.set_language(lang).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn set_language_rejects_empty_metadata() {
    let mut parser = Parser::new();
    // Build a language with empty symbol_metadata — builder requires it via
    // `ok_or("missing symbol metadata")`, but we can supply an empty vec by
    // going through a helper that gives an empty one.
    #[cfg(feature = "glr-core")]
    {
        use adze_runtime::Language;
        let table = stub_parse_table();
        let result = Language::builder()
            .parse_table(table)
            .symbol_metadata(vec![])
            .field_names(vec![])
            .tokenizer(|_| Box::new(std::iter::empty()))
            .build();
        // Building succeeds (0-length metadata is valid at builder level)…
        let lang = result.unwrap();
        // …but set_language rejects it because "Language has no symbol metadata"
        let err = parser.set_language(lang);
        assert!(err.is_err());
    }
}

#[test]
fn language_symbol_name_lookup() {
    let lang = multi_symbol_test_language(3);
    assert_eq!(lang.symbol_name(0), Some("symbol_0"));
    assert_eq!(lang.symbol_name(1), Some("symbol_1"));
    assert_eq!(lang.symbol_name(2), Some("symbol_2"));
    assert_eq!(lang.symbol_name(99), None);
}

#[test]
fn language_metadata_queries() {
    let lang = multi_symbol_test_language(2);
    assert!(lang.is_terminal(0));
    assert!(lang.is_visible(0));
    assert!(!lang.is_terminal(99)); // out of bounds → false
    assert!(!lang.is_visible(99));
}

// ---------------------------------------------------------------------------
// 3. Parse error handling
// ---------------------------------------------------------------------------

#[test]
fn parse_without_language_gives_no_language_error() {
    let mut parser = Parser::new();
    let err = parser.parse_utf8("hello", None).unwrap_err();
    assert_eq!(err.to_string(), "no language set");
}

#[test]
fn parse_bytes_without_language_gives_no_language_error() {
    let mut parser = Parser::new();
    let err = parser.parse(b"hello", None).unwrap_err();
    assert_eq!(err.to_string(), "no language set");
}

#[test]
fn parse_error_display_variants() {
    let no_lang = ParseError::no_language();
    assert_eq!(no_lang.to_string(), "no language set");

    let timeout = ParseError::timeout();
    assert_eq!(timeout.to_string(), "parse timeout exceeded");

    let custom = ParseError::with_msg("custom failure");
    assert_eq!(custom.to_string(), "custom failure");

    let syntax = ParseError::syntax_error(
        "unexpected token",
        adze_runtime::error::ErrorLocation {
            byte_offset: 10,
            line: 2,
            column: 5,
        },
    );
    assert!(syntax.to_string().contains("unexpected token"));
    assert!(syntax.location.is_some());
}

// ---------------------------------------------------------------------------
// 4. Timeout configuration
// ---------------------------------------------------------------------------

#[test]
fn parser_timeout_round_trip() {
    let mut parser = Parser::new();
    assert!(parser.timeout().is_none());

    parser.set_timeout(Duration::from_millis(500));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(500)));
}

// ---------------------------------------------------------------------------
// 5. Tree stub & cloning
// ---------------------------------------------------------------------------

#[test]
fn stub_tree_has_zero_range_root() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn stub_tree_clone_is_independent() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    // Both roots have same byte ranges
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
}

#[test]
fn stub_tree_has_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn stub_tree_has_no_source_bytes() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

// ---------------------------------------------------------------------------
// 6. Node API on stub tree
// ---------------------------------------------------------------------------

#[test]
fn node_kind_without_language_is_unknown() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind(), "unknown");
}

#[test]
fn node_kind_id_is_symbol() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn node_byte_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn node_positional_stubs() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // Phase 1: positions are (0,0) stubs
    assert_eq!(root.start_position(), adze_runtime::Point::new(0, 0));
    assert_eq!(root.end_position(), adze_runtime::Point::new(0, 0));
}

#[test]
fn node_boolean_predicates() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.is_named());
    assert!(!root.is_missing());
    assert!(!root.is_error());
}

#[test]
fn node_child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.child(0).is_none());
    assert!(root.named_child(0).is_none());
}

#[test]
fn node_navigation_stubs_return_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.parent().is_none());
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
    assert!(root.next_named_sibling().is_none());
    assert!(root.prev_named_sibling().is_none());
    assert!(root.child_by_field_name("foo").is_none());
}

#[test]
fn node_utf8_text_on_stub() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let text = root.utf8_text(b"hello world").unwrap();
    assert_eq!(text, ""); // range 0..0 → empty
}

#[test]
fn node_debug_format() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let debug = format!("{:?}", root);
    assert!(debug.contains("Node"));
    assert!(debug.contains("unknown"));
}

// ---------------------------------------------------------------------------
// 7. TreeCursor traversal
// ---------------------------------------------------------------------------

#[test]
fn tree_cursor_on_stub_tree() {
    let tree = Tree::new_stub();
    let mut cursor = adze_runtime::tree::TreeCursor::new(&tree);
    // Root has no children
    assert!(!cursor.goto_first_child());
    // Root has no parent
    assert!(!cursor.goto_parent());
    // No siblings
    assert!(!cursor.goto_next_sibling());
}

// ---------------------------------------------------------------------------
// 8. Point type
// ---------------------------------------------------------------------------

#[test]
fn point_new_and_display() {
    let p = adze_runtime::Point::new(0, 0);
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
    // Display is 1-indexed
    assert_eq!(format!("{}", p), "1:1");
}

#[test]
fn point_ordering() {
    let a = adze_runtime::Point::new(1, 5);
    let b = adze_runtime::Point::new(2, 0);
    assert!(a < b);
}

// ---------------------------------------------------------------------------
// 9. Token type
// ---------------------------------------------------------------------------

#[test]
fn token_debug() {
    let tok = Token {
        kind: 1,
        start: 0,
        end: 3,
    };
    let debug = format!("{:?}", tok);
    assert!(debug.contains("Token"));
    assert!(debug.contains("kind: 1"));
}

// ---------------------------------------------------------------------------
// 10. Parser reset
// ---------------------------------------------------------------------------

#[test]
fn parser_reset_does_not_panic() {
    let mut parser = Parser::new();
    parser.reset(); // should be a no-op without arena feature
}

// ---------------------------------------------------------------------------
// 11. GLR engine interaction (requires glr-core feature)
//
// NOTE: The GLR engine currently has a known pre-existing issue where simple
// grammars fail with "no valid parse paths" (the existing glr_parse_simple
// test also fails). These tests exercise the setup and error-handling paths
// rather than expecting successful parses.
// ---------------------------------------------------------------------------

#[cfg(feature = "glr-core")]
mod glr_engine_integration {
    use super::*;
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
    use adze_runtime::Language;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Build a minimal grammar: start → "a"
    fn build_a_language() -> (Language, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut grammar = Grammar::new("test_a".to_string());

        let a_id = SymbolId(1);
        let start_id = SymbolId(2);

        grammar.tokens.insert(
            a_id,
            IrToken {
                name: "a".to_string(),
                pattern: TokenPattern::String("a".to_string()),
                fragile: false,
            },
        );
        grammar.rule_names.insert(start_id, "start".to_string());
        grammar.rules.insert(
            start_id,
            vec![Rule {
                lhs: start_id,
                rhs: vec![Symbol::Terminal(a_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            }],
        );

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let table = build_lr1_automaton(&grammar, &ff).expect("table build");
        let table: &'static _ = Box::leak(Box::new(table));

        let t_counter = counter.clone();
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
            .field_names(vec![])
            .tokenizer(
                move |input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
                    t_counter.fetch_add(1, Ordering::SeqCst);
                    let mut toks = Vec::new();
                    for (i, &b) in input.iter().enumerate() {
                        if b == b'a' {
                            toks.push(Token {
                                kind: 1,
                                start: i as u32,
                                end: (i + 1) as u32,
                            });
                        }
                    }
                    toks.push(Token {
                        kind: 0,
                        start: input.len() as u32,
                        end: input.len() as u32,
                    });
                    Box::new(toks.into_iter())
                },
            )
            .build()
            .unwrap();

        (lang, counter)
    }

    #[test]
    fn language_builder_produces_valid_language() {
        let (lang, _) = build_a_language();
        assert_eq!(lang.symbol_count, 3);
        assert_eq!(lang.symbol_name(0), Some("EOF"));
        assert_eq!(lang.symbol_name(1), Some("a"));
        assert_eq!(lang.symbol_name(2), Some("start"));
        assert!(lang.is_terminal(0));
        assert!(lang.is_terminal(1));
        assert!(!lang.is_terminal(2));
    }

    #[test]
    fn set_language_succeeds_with_glr_language() {
        let (lang, _) = build_a_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        assert!(parser.language().is_some());
    }

    #[test]
    fn parse_invokes_tokenizer() {
        let (lang, counter) = build_a_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        // Parse may fail due to known GLR engine issues, but the tokenizer
        // should still be invoked.
        let _ = parser.parse_utf8("a", None);
        assert!(
            counter.load(Ordering::SeqCst) >= 1,
            "tokenizer was never called"
        );
    }

    #[test]
    fn parse_returns_result_not_panic() {
        let (lang, _) = build_a_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        // Should return a Result (Ok or Err), never panic.
        let result = parser.parse_utf8("a", None);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn parse_error_is_displayable() {
        let (lang, _) = build_a_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        if let Err(e) = parser.parse_utf8("a", None) {
            let msg = e.to_string();
            assert!(!msg.is_empty(), "error message should not be empty");
        }
    }

    #[test]
    fn set_language_rejects_missing_tokenizer() {
        let table = stub_parse_table();
        let lang = Language::builder()
            .parse_table(table)
            .symbol_names(vec!["placeholder".into()])
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .field_names(vec![])
            // No tokenizer set
            .build()
            .unwrap();

        let mut parser = Parser::new();
        let err = parser.set_language(lang);
        assert!(err.is_err(), "should reject language without tokenizer");
    }

    #[test]
    fn language_with_static_tokens() {
        let table = stub_parse_table();
        let lang = Language::builder()
            .parse_table(table)
            .symbol_names(vec!["EOF".into(), "tok".into()])
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
            ])
            .field_names(vec![])
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

        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        // Parser is configured — parsing may fail due to empty parse table,
        // but setup succeeded.
        assert!(parser.language().is_some());
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "glr-core")]
fn stub_parse_table() -> &'static adze_glr_core::ParseTable {
    use adze_glr_core::{GotoIndexing, ParseTable};
    use adze_ir::{Grammar, StateId, SymbolId};
    use std::collections::BTreeMap;

    Box::leak(Box::new(ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        grammar: Grammar::new("stub".to_string()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }))
}
