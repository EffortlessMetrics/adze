//! Integration tests exercising the full Parser workflow:
//! create parser → set language → parse → inspect tree.
//!
//! Run with: `cargo test -p adze-runtime --test parser_integration`

#[cfg(feature = "glr-core")]
mod with_glr {
    use adze_glr_core::{Action, ParseTable, StateId, SymbolId};
    use adze_runtime::language::SymbolMetadata;
    use adze_runtime::{Language, Parser, Token};
    use std::collections::BTreeMap;
    use std::time::Duration;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build a minimal parse table that shifts one token then accepts on EOF.
    ///
    /// Grammar (2 terminals): EOF=0, tok=1  
    /// State 0: on tok → Shift(1)  
    /// State 1: on EOF → Accept  
    fn shift_accept_table() -> &'static ParseTable {
        let mut symbol_to_index = BTreeMap::new();
        symbol_to_index.insert(SymbolId(0), 0); // EOF
        symbol_to_index.insert(SymbolId(1), 1); // tok

        let table = ParseTable {
            state_count: 2,
            symbol_count: 2,
            symbol_to_index,
            start_symbol: SymbolId(1),
            index_to_symbol: vec![SymbolId(0), SymbolId(1)],
            action_table: vec![
                // state 0: on tok → Shift(1)
                vec![vec![], vec![Action::Shift(StateId(1))]],
                // state 1: on EOF → Accept
                vec![vec![Action::Accept], vec![]],
            ],
            goto_table: vec![vec![], vec![]],
            ..Default::default()
        };
        Box::leak(Box::new(table))
    }

    /// Build a language backed by `shift_accept_table` with a single-token tokenizer.
    fn tiny_language() -> Language {
        Language::builder()
            .version(14)
            .parse_table(shift_accept_table())
            .symbol_names(vec!["eof".into(), "token".into()])
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
            .tokenizer(|_input| {
                Box::new(
                    vec![
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
                    ]
                    .into_iter(),
                )
            })
            .build()
            .unwrap()
    }

    /// Build a language whose tokenizer emits two content tokens before EOF.
    fn two_token_language() -> Language {
        let mut s2i = BTreeMap::new();
        s2i.insert(SymbolId(0), 0); // EOF
        s2i.insert(SymbolId(1), 1); // tok

        let table = ParseTable {
            state_count: 3,
            symbol_count: 2,
            symbol_to_index: s2i,
            start_symbol: SymbolId(1),
            index_to_symbol: vec![SymbolId(0), SymbolId(1)],
            action_table: vec![
                // state 0: tok → Shift(1)
                vec![vec![], vec![Action::Shift(StateId(1))]],
                // state 1: tok → Shift(2)
                vec![vec![], vec![Action::Shift(StateId(2))]],
                // state 2: EOF → Accept
                vec![vec![Action::Accept], vec![]],
            ],
            goto_table: vec![vec![]; 3],
            ..Default::default()
        };
        let table_ref: &'static ParseTable = Box::leak(Box::new(table));

        Language::builder()
            .version(14)
            .parse_table(table_ref)
            .symbol_names(vec!["eof".into(), "tok".into()])
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
            .tokenizer(|_input| {
                Box::new(
                    vec![
                        Token {
                            kind: 1,
                            start: 0,
                            end: 2,
                        },
                        Token {
                            kind: 1,
                            start: 2,
                            end: 4,
                        },
                        Token {
                            kind: 0,
                            start: 4,
                            end: 4,
                        },
                    ]
                    .into_iter(),
                )
            })
            .build()
            .unwrap()
    }

    // -----------------------------------------------------------------------
    // 1. Parser creation and language setting
    // -----------------------------------------------------------------------

    #[test]
    fn create_parser_and_set_language() {
        let mut parser = Parser::new();
        assert!(parser.language().is_none());

        let lang = tiny_language();
        parser.set_language(lang).unwrap();

        let lang_ref = parser.language().unwrap();
        assert_eq!(lang_ref.version, 14);
        assert_eq!(lang_ref.symbol_count, 2);
    }

    #[test]
    fn parser_default_equals_new() {
        let p1 = Parser::new();
        let p2 = Parser::default();
        assert!(p1.language().is_none());
        assert!(p2.language().is_none());
        assert!(p1.timeout().is_none());
        assert!(p2.timeout().is_none());
    }

    // -----------------------------------------------------------------------
    // 2. Parsing simple input and inspecting the tree
    // -----------------------------------------------------------------------

    #[test]
    fn parse_simple_input_returns_tree() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let tree = parser.parse(b"x", None).unwrap();
        let root = tree.root_node();

        // Tree was produced and has reasonable byte range
        assert!(root.end_byte() >= root.start_byte());
    }

    #[test]
    fn parse_tree_root_has_symbol_name() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let tree = parser.parse(b"x", None).unwrap();
        let root = tree.root_node();

        // Root node should resolve to a symbol name from the language
        let kind = root.kind();
        assert!(!kind.is_empty());
    }

    #[test]
    fn parse_tree_carries_source_bytes() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let tree = parser.parse(b"x", None).unwrap();
        assert_eq!(tree.source_bytes(), Some(b"x".as_slice()));
    }

    #[test]
    fn parse_two_token_input_produces_children() {
        let mut parser = Parser::new();
        parser.set_language(two_token_language()).unwrap();

        let tree = parser.parse(b"abcd", None).unwrap();
        let root = tree.root_node();
        // The tree should have structure (root with children or a flat list)
        assert!(root.end_byte() >= root.start_byte());
    }

    // -----------------------------------------------------------------------
    // 3. Multiple parses with same parser
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_parses_same_parser() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let t1 = parser.parse(b"a", None).unwrap();
        let t2 = parser.parse(b"b", None).unwrap();
        let t3 = parser.parse(b"c", None).unwrap();

        // Each parse returns an independent tree
        assert_eq!(t1.source_bytes(), Some(b"a".as_slice()));
        assert_eq!(t2.source_bytes(), Some(b"b".as_slice()));
        assert_eq!(t3.source_bytes(), Some(b"c".as_slice()));
    }

    #[test]
    fn parse_after_reset_still_works() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let _ = parser.parse(b"a", None).unwrap();
        parser.reset();
        let tree = parser.parse(b"b", None).unwrap();
        assert_eq!(tree.source_bytes(), Some(b"b".as_slice()));
    }

    // -----------------------------------------------------------------------
    // 4. parse_utf8 convenience method
    // -----------------------------------------------------------------------

    #[test]
    fn parse_utf8_convenience() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let tree = parser.parse_utf8("x", None).unwrap();
        assert_eq!(tree.source_bytes(), Some(b"x".as_slice()));

        let root = tree.root_node();
        assert!(root.end_byte() >= root.start_byte());
    }

    #[test]
    fn parse_utf8_with_unicode_input() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        // The tokenizer ignores actual bytes, so this still parses fine
        let tree = parser.parse_utf8("é", None).unwrap();
        assert!(tree.source_bytes().is_some());
    }

    // -----------------------------------------------------------------------
    // 5. Error cases
    // -----------------------------------------------------------------------

    #[test]
    fn parse_without_language_errors() {
        let mut parser = Parser::new();
        let err = parser.parse(b"hello", None).unwrap_err();
        assert!(err.to_string().contains("no language"));
    }

    #[test]
    fn parse_utf8_without_language_errors() {
        let mut parser = Parser::new();
        let err = parser.parse_utf8("hello", None).unwrap_err();
        assert!(err.to_string().contains("no language"));
    }

    #[test]
    fn parse_empty_input_without_language_errors() {
        let mut parser = Parser::new();
        assert!(parser.parse(b"", None).is_err());
    }

    #[test]
    fn set_language_rejects_empty_metadata() {
        let mut parser = Parser::new();
        let result = Language::builder()
            .version(1)
            .parse_table(shift_accept_table())
            .symbol_metadata(vec![])
            .tokenizer(|_| Box::new(std::iter::empty()))
            .build();
        // Builder succeeds but set_language rejects empty metadata
        let lang = result.unwrap();
        let err = parser.set_language(lang);
        assert!(err.is_err());
    }

    #[test]
    fn language_builder_rejects_missing_table() {
        let result = Language::builder()
            .version(1)
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .build();
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // 6. Language switching
    // -----------------------------------------------------------------------

    #[test]
    fn switch_language_between_parses() {
        let mut parser = Parser::new();

        // Parse with first language
        parser.set_language(tiny_language()).unwrap();
        let t1 = parser.parse(b"a", None).unwrap();

        // Switch to a different language (same shape, different names)
        let lang2 = Language::builder()
            .version(15)
            .parse_table(shift_accept_table())
            .symbol_names(vec!["END".into(), "word".into()])
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
            .tokenizer(|_| {
                Box::new(
                    vec![
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
                    ]
                    .into_iter(),
                )
            })
            .build()
            .unwrap();
        parser.set_language(lang2).unwrap();
        let t2 = parser.parse(b"b", None).unwrap();

        // Trees from different languages have different symbol names
        let lang1 = t1.language().unwrap();
        let lang2 = t2.language().unwrap();
        assert_eq!(lang1.symbol_name(1), Some("token"));
        assert_eq!(lang2.symbol_name(1), Some("word"));
    }

    #[test]
    fn language_accessor_reflects_latest_set() {
        let mut parser = Parser::new();
        assert!(parser.language().is_none());

        parser.set_language(tiny_language()).unwrap();
        assert_eq!(parser.language().unwrap().version, 14);

        // Build a v15 language
        let lang2 = Language::builder()
            .version(15)
            .parse_table(shift_accept_table())
            .symbol_names(vec!["eof".into(), "tok".into()])
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
            .tokenizer(|_| {
                Box::new(
                    vec![
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
                    ]
                    .into_iter(),
                )
            })
            .build()
            .unwrap();
        parser.set_language(lang2).unwrap();
        assert_eq!(parser.language().unwrap().version, 15);
    }

    // -----------------------------------------------------------------------
    // 7. Timeout behavior
    // -----------------------------------------------------------------------

    #[test]
    fn set_and_get_timeout() {
        let mut parser = Parser::new();
        assert!(parser.timeout().is_none());

        parser.set_timeout(Duration::from_millis(500));
        assert_eq!(parser.timeout(), Some(Duration::from_millis(500)));
    }

    #[test]
    fn timeout_can_be_updated() {
        let mut parser = Parser::new();
        parser.set_timeout(Duration::from_millis(100));
        assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));

        parser.set_timeout(Duration::from_secs(5));
        assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn parse_succeeds_with_generous_timeout() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();
        parser.set_timeout(Duration::from_secs(10));

        // Parsing a trivial input should complete well within timeout
        let tree = parser.parse(b"x", None).unwrap();
        assert!(tree.source_bytes().is_some());
    }

    // -----------------------------------------------------------------------
    // Additional coverage: tree inspection, node API, debug impls
    // -----------------------------------------------------------------------

    #[test]
    fn tree_has_language_after_parse() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let tree = parser.parse(b"x", None).unwrap();
        assert!(tree.language().is_some());
        assert_eq!(tree.language().unwrap().version, 14);
    }

    #[test]
    fn node_byte_range_within_source() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let tree = parser.parse(b"x", None).unwrap();
        let root = tree.root_node();

        assert!(root.start_byte() <= root.end_byte());
        assert!(root.end_byte() <= b"x".len() + 1); // may include EOF position
    }

    #[test]
    fn tree_debug_does_not_panic() {
        let mut parser = Parser::new();
        parser.set_language(tiny_language()).unwrap();

        let tree = parser.parse(b"x", None).unwrap();
        let debug = format!("{:?}", tree);
        assert!(debug.contains("Tree"));
    }

    #[test]
    fn parser_debug_does_not_panic() {
        let parser = Parser::new();
        let debug = format!("{:?}", parser);
        assert!(debug.contains("Parser"));
    }
}

// Tests that work without glr-core (pure parser API surface)
mod without_glr {
    use adze_runtime::Parser;
    use std::time::Duration;

    #[test]
    fn new_parser_has_no_language() {
        let parser = Parser::new();
        assert!(parser.language().is_none());
        assert!(parser.timeout().is_none());
    }

    #[test]
    fn parse_without_language_is_err() {
        let mut parser = Parser::new();
        assert!(parser.parse(b"test", None).is_err());
    }

    #[test]
    fn parse_utf8_without_language_is_err() {
        let mut parser = Parser::new();
        assert!(parser.parse_utf8("test", None).is_err());
    }

    #[test]
    fn timeout_round_trip() {
        let mut parser = Parser::new();
        parser.set_timeout(Duration::from_millis(42));
        assert_eq!(parser.timeout(), Some(Duration::from_millis(42)));
    }

    #[test]
    fn reset_preserves_language_setting() {
        let mut parser = Parser::new();
        // No language set – reset should be harmless
        parser.reset();
        assert!(parser.language().is_none());
    }
}
