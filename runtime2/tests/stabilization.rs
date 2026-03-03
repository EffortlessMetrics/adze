//! Stabilization tests for the runtime2 GLR runtime.
//!
//! These tests exercise the core public API surface to prevent regressions:
//! - Parser API: new(), set_language(), parse(), reparse via old_tree
//! - Tree API: root_node(), kind(), children, traversal
//! - Tree edit: edit() with valid/invalid ranges (incremental_glr feature)
//! - Error handling: parse without language, empty metadata rejection
//! - Forest-to-tree conversion correctness (glr-core feature)
//! - Performance monitoring via ADZE_LOG_PERFORMANCE env var

// ============================================================================
// 1. Parser API
// ============================================================================

mod parser_api {
    use adze_runtime::{Parser, test_helpers::stub_language};
    use std::time::Duration;

    #[test]
    fn new_returns_parser_with_no_language() {
        let p = Parser::new();
        assert!(p.language().is_none());
        assert!(p.timeout().is_none());
    }

    #[test]
    fn default_is_equivalent_to_new() {
        let p = Parser::default();
        assert!(p.language().is_none());
    }

    #[test]
    fn set_language_accepts_valid_language() {
        let mut p = Parser::new();
        let lang = stub_language();
        p.set_language(lang)
            .expect("valid language should be accepted");
        assert!(p.language().is_some());
    }

    #[test]
    fn set_language_can_be_called_twice() {
        let mut p = Parser::new();
        p.set_language(stub_language()).unwrap();
        p.set_language(stub_language()).unwrap();
        assert!(p.language().is_some());
    }

    #[test]
    fn set_timeout_and_get_timeout() {
        let mut p = Parser::new();
        p.set_timeout(Duration::from_millis(250));
        assert_eq!(p.timeout(), Some(Duration::from_millis(250)));
    }

    #[test]
    fn parse_without_language_returns_no_language_error() {
        let mut p = Parser::new();
        let err = p.parse(b"hello", None).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("no language"));
    }

    #[test]
    fn parse_utf8_without_language_returns_error() {
        let mut p = Parser::new();
        let err = p.parse_utf8("hello", None).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("no language"));
    }

    #[test]
    fn reset_preserves_language() {
        let mut p = Parser::new();
        p.set_language(stub_language()).unwrap();
        p.reset();
        assert!(p.language().is_some(), "reset should not clear language");
    }
}

// ============================================================================
// 2. Tree API – stub trees (no glr-core required)
// ============================================================================

mod tree_api {
    use adze_runtime::{Tree, tree::TreeCursor};

    #[test]
    fn stub_tree_root_node_kind_is_unknown() {
        let tree = Tree::new_stub();
        assert_eq!(tree.root_node().kind(), "unknown");
    }

    #[test]
    fn stub_tree_root_kind_returns_zero() {
        let tree = Tree::new_stub();
        assert_eq!(tree.root_kind(), 0);
    }

    #[test]
    fn stub_tree_root_node_byte_range() {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        assert_eq!(root.start_byte(), 0);
        assert_eq!(root.end_byte(), 0);
        assert_eq!(root.byte_range(), 0..0);
    }

    #[test]
    fn stub_tree_root_has_no_children() {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        assert_eq!(root.child_count(), 0);
        assert!(root.child(0).is_none());
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

    #[test]
    fn stub_tree_clone_is_independent() {
        let tree = Tree::new_stub();
        let cloned = tree.clone();
        assert_eq!(tree.root_kind(), cloned.root_kind());
        assert_eq!(tree.source_bytes(), cloned.source_bytes());
    }

    #[test]
    fn stub_tree_utf8_text_on_zero_range() {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        let text = root.utf8_text(b"hello").unwrap();
        assert_eq!(text, "");
    }

    // -- TreeCursor on stub tree --

    #[test]
    fn cursor_on_stub_tree_cannot_descend() {
        let tree = Tree::new_stub();
        let mut cursor = TreeCursor::new(&tree);
        assert!(!cursor.goto_first_child());
        assert!(!cursor.goto_next_sibling());
        assert!(!cursor.goto_parent());
    }
}

// ============================================================================
// 3. Tree edit – requires incremental_glr feature
// ============================================================================

#[cfg(feature = "incremental_glr")]
mod tree_edit {
    use adze_runtime::{InputEdit, Point, Tree, tree::EditError};

    fn sample_tree() -> Tree {
        // Build a tree with known byte ranges via internal TreeNode.
        // We can't call Tree::new (pub(crate)), so use new_stub and edit it.
        // Instead, rely on clone of a stub — edits on zero-range are still valid.
        Tree::new_stub()
    }

    #[test]
    fn edit_with_valid_range_succeeds() {
        let mut tree = sample_tree();
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte: 0,
            new_end_byte: 5,
            start_position: Point::new(0, 0),
            old_end_position: Point::new(0, 0),
            new_end_position: Point::new(0, 5),
        };
        tree.edit(&edit).expect("insertion at start should succeed");
    }

    #[test]
    fn edit_rejects_old_end_before_start() {
        let mut tree = sample_tree();
        let edit = InputEdit {
            start_byte: 10,
            old_end_byte: 5,
            new_end_byte: 15,
            start_position: Point::new(0, 10),
            old_end_position: Point::new(0, 5),
            new_end_position: Point::new(0, 15),
        };
        assert!(matches!(
            tree.edit(&edit),
            Err(EditError::InvalidRange { .. })
        ));
    }

    #[test]
    fn edit_rejects_new_end_before_start() {
        let mut tree = sample_tree();
        let edit = InputEdit {
            start_byte: 10,
            old_end_byte: 15,
            new_end_byte: 5,
            start_position: Point::new(0, 10),
            old_end_position: Point::new(0, 15),
            new_end_position: Point::new(0, 5),
        };
        assert!(matches!(
            tree.edit(&edit),
            Err(EditError::InvalidRange { .. })
        ));
    }
}

// ============================================================================
// 4. Error handling
// ============================================================================

mod error_handling {
    use adze_runtime::{ParseError, ParseErrorKind, error::ErrorLocation};

    #[test]
    fn no_language_error_kind() {
        let err = ParseError::no_language();
        assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
        assert!(err.location.is_none());
    }

    #[test]
    fn timeout_error_kind() {
        let err = ParseError::timeout();
        assert!(matches!(err.kind, ParseErrorKind::Timeout));
    }

    #[test]
    fn syntax_error_preserves_location() {
        let loc = ErrorLocation {
            byte_offset: 10,
            line: 2,
            column: 5,
        };
        let err = ParseError::syntax_error("unexpected }", loc);
        assert!(err.to_string().contains("unexpected }"));
        assert_eq!(err.location.unwrap().byte_offset, 10);
    }

    #[test]
    fn with_msg_creates_other_variant() {
        let err = ParseError::with_msg("test message");
        assert!(matches!(err.kind, ParseErrorKind::Other(_)));
        assert_eq!(err.to_string(), "test message");
    }

    #[test]
    fn with_location_chaining() {
        let err = ParseError::no_language().with_location(ErrorLocation {
            byte_offset: 0,
            line: 1,
            column: 1,
        });
        assert!(err.location.is_some());
    }

    #[test]
    fn error_location_display() {
        let loc = ErrorLocation {
            byte_offset: 0,
            line: 3,
            column: 7,
        };
        assert_eq!(loc.to_string(), "3:7");
    }
}

// ============================================================================
// 5. Forest-to-tree conversion (requires glr-core)
// ============================================================================

#[cfg(feature = "glr-core")]
mod forest_to_tree {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
    use adze_runtime::{Language, Parser, Token, language::SymbolMetadata};

    /// Build a minimal grammar: start → a
    fn make_language() -> Language {
        let mut grammar = Grammar::new("stab_test".to_string());
        let a_id = SymbolId(1);
        grammar.tokens.insert(
            a_id,
            IrToken {
                name: "a".to_string(),
                pattern: TokenPattern::String("a".to_string()),
                fragile: false,
            },
        );
        let start_id = SymbolId(2);
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
        let table = build_lr1_automaton(&grammar, &ff)
            .expect("table")
            .normalize_eof_to_zero()
            .with_detected_goto_indexing();
        let table: &'static _ = Box::leak(Box::new(table));

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
            .tokenizer(|input: &[u8]| {
                let mut toks = Vec::new();
                if !input.is_empty() {
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
                Box::new(toks.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
            })
            .build()
            .unwrap()
    }

    #[test]
    fn parse_produces_tree_with_valid_root() {
        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree = parser.parse(b"a", None).unwrap();
        let root = tree.root_node();
        assert!(root.start_byte() <= root.end_byte());
    }

    #[test]
    fn parsed_tree_has_language_set() {
        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree = parser.parse(b"a", None).unwrap();
        assert!(tree.language().is_some());
    }

    #[test]
    fn parsed_tree_stores_source_bytes() {
        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree = parser.parse(b"a", None).unwrap();
        assert_eq!(tree.source_bytes(), Some(b"a".as_slice()));
    }

    #[test]
    fn root_node_kind_resolves_via_language() {
        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree = parser.parse(b"a", None).unwrap();
        let root = tree.root_node();
        // Root should be "start" (the non-terminal) or one of the known symbols
        let kind = root.kind();
        assert_ne!(kind, "unknown", "kind should resolve via language");
    }

    #[test]
    fn reparse_with_old_tree_succeeds() {
        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree1 = parser.parse(b"a", None).unwrap();
        // Passing old tree triggers reparse path (falls back to full parse if
        // incremental is disabled)
        let tree2 = parser.parse(b"a", Some(&tree1)).unwrap();
        assert_eq!(tree1.root_kind(), tree2.root_kind());
    }

    #[test]
    fn parsed_tree_clone_preserves_structure() {
        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree = parser.parse(b"a", None).unwrap();
        let cloned = tree.clone();
        assert_eq!(tree.root_kind(), cloned.root_kind());
        assert_eq!(
            tree.root_node().start_byte(),
            cloned.root_node().start_byte()
        );
        assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
    }

    #[test]
    fn child_nodes_inherit_language() {
        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree = parser.parse(b"a", None).unwrap();
        let root = tree.root_node();
        if root.child_count() > 0 {
            let child = root.child(0).unwrap();
            // Child should also resolve kind via language
            assert_ne!(child.kind(), "unknown");
        }
    }
}

// ============================================================================
// 6. Performance monitoring (ADZE_LOG_PERFORMANCE)
// ============================================================================

#[cfg(feature = "glr-core")]
mod performance_monitoring {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
    use adze_runtime::{Language, Parser, Token, language::SymbolMetadata};

    fn make_language() -> Language {
        let mut grammar = Grammar::new("perf_test".to_string());
        let a_id = SymbolId(1);
        grammar.tokens.insert(
            a_id,
            IrToken {
                name: "a".to_string(),
                pattern: TokenPattern::String("a".to_string()),
                fragile: false,
            },
        );
        let start_id = SymbolId(2);
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
        let table = build_lr1_automaton(&grammar, &ff)
            .expect("table")
            .normalize_eof_to_zero()
            .with_detected_goto_indexing();
        let table: &'static _ = Box::leak(Box::new(table));

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
            .tokenizer(|input: &[u8]| {
                let mut toks = Vec::new();
                if !input.is_empty() {
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
                Box::new(toks.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
            })
            .build()
            .unwrap()
    }

    /// Verify that parsing succeeds with ADZE_LOG_PERFORMANCE set.
    /// The env var triggers eprintln! in builder.rs; we just confirm no panic.
    #[test]
    fn parse_with_performance_logging_does_not_panic() {
        // Set the env var for this test only (safe: tests run single-threaded
        // per test binary unless overridden, and we restore immediately).
        // SAFETY: This test runs in isolation; no other threads depend on this var.
        unsafe { std::env::set_var("ADZE_LOG_PERFORMANCE", "1") };

        let mut parser = Parser::new();
        parser.set_language(make_language()).unwrap();
        let tree = parser.parse(b"a", None).unwrap();
        assert!(tree.root_node().start_byte() <= tree.root_node().end_byte());

        // SAFETY: Restoring env state after test.
        unsafe { std::env::remove_var("ADZE_LOG_PERFORMANCE") };
    }
}

// ============================================================================
// 7. Language builder validation
// ============================================================================

mod language_builder {
    use adze_runtime::Language;

    #[test]
    fn builder_without_parse_table_fails() {
        use adze_runtime::language::SymbolMetadata;
        let result = Language::builder()
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_without_symbol_metadata_fails() {
        // Cannot call .build() without metadata — test that it errors
        let result = Language::builder().build();
        assert!(result.is_err());
    }
}
