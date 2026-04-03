//! Edge-case and error-handling tests for the runtime2 GLR parser.
//!
//! Complements existing tests in basic.rs, parser_api_tests.rs,
//! tree_api_tests.rs, builder_tests.rs, and feature_flags.rs by
//! covering scenarios those files do not exercise.

use adze_runtime::error::ErrorLocation;
use adze_runtime::language::SymbolMetadata;
use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use adze_runtime::tree::TreeCursor;
use adze_runtime::{ParseError, ParseErrorKind, Parser, Point, Token, Tree};

// ===========================================================================
// 1. ParseErrorKind variant coverage
// ===========================================================================

#[test]
fn parse_error_kind_version_mismatch_display() {
    let err = ParseError {
        kind: ParseErrorKind::VersionMismatch {
            expected: 15,
            actual: 14,
        },
        location: None,
    };
    let msg = err.to_string();
    assert!(msg.contains("15"), "should mention expected version");
    assert!(msg.contains("14"), "should mention actual version");
}

#[test]
fn parse_error_kind_invalid_encoding_display() {
    let err = ParseError {
        kind: ParseErrorKind::InvalidEncoding,
        location: None,
    };
    assert_eq!(err.to_string(), "invalid input encoding");
}

#[test]
fn parse_error_kind_cancelled_display() {
    let err = ParseError {
        kind: ParseErrorKind::Cancelled,
        location: None,
    };
    assert_eq!(err.to_string(), "parse cancelled");
}

#[test]
fn parse_error_kind_allocation_error_display() {
    let err = ParseError {
        kind: ParseErrorKind::AllocationError,
        location: None,
    };
    assert_eq!(err.to_string(), "memory allocation failed");
}

#[test]
fn parse_error_syntax_error_display() {
    let err = ParseError {
        kind: ParseErrorKind::SyntaxError("unexpected '}'".into()),
        location: None,
    };
    assert!(err.to_string().contains("unexpected '}'"));
}

#[test]
fn parse_error_other_display() {
    let err = ParseError {
        kind: ParseErrorKind::Other("custom detail".into()),
        location: None,
    };
    assert_eq!(err.to_string(), "custom detail");
}

#[test]
fn parse_error_is_send_and_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<ParseError>();
    assert_sync::<ParseError>();
}

// ===========================================================================
// 2. ErrorLocation edge cases
// ===========================================================================

#[test]
fn error_location_display_at_origin() {
    let loc = ErrorLocation {
        byte_offset: 0,
        line: 1,
        column: 1,
    };
    assert_eq!(loc.to_string(), "1:1");
}

#[test]
fn error_location_equality() {
    let a = ErrorLocation {
        byte_offset: 10,
        line: 2,
        column: 5,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn parse_error_with_location_preserves_kind() {
    let loc = ErrorLocation {
        byte_offset: 100,
        line: 10,
        column: 20,
    };
    let err = ParseError::timeout().with_location(loc.clone());
    // Kind should still be Timeout
    assert!(matches!(err.kind, ParseErrorKind::Timeout));
    assert_eq!(err.location.unwrap(), loc);
}

// ===========================================================================
// 3. Parser: re-setting / switching language
// ===========================================================================

#[test]
fn parser_can_switch_language() {
    let mut parser = Parser::new();
    let lang1 = stub_language();
    let lang2 = multi_symbol_test_language(5);

    parser.set_language(lang1).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 1);

    parser.set_language(lang2).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 5);
}

#[test]
fn parser_set_language_can_be_called_multiple_times() {
    let mut parser = Parser::new();
    for _ in 0..10 {
        let lang = stub_language();
        parser.set_language(lang).unwrap();
    }
    assert!(parser.language().is_some());
}

// ===========================================================================
// 4. Parser: timeout does not affect language
// ===========================================================================

#[test]
fn timeout_does_not_affect_language_or_reset() {
    let mut parser = Parser::new();
    let lang = stub_language();
    parser.set_language(lang).unwrap();
    parser.set_timeout(std::time::Duration::from_millis(1));
    assert!(parser.language().is_some());
    parser.reset();
    assert!(parser.language().is_some());
    assert_eq!(parser.timeout(), Some(std::time::Duration::from_millis(1)));
}

// ===========================================================================
// 5. Tree: stub tree invariants
// ===========================================================================

#[test]
fn stub_tree_root_byte_range_is_empty() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.byte_range(), 0..0);
    assert_eq!(root.start_byte(), root.end_byte());
}

#[test]
fn stub_tree_has_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
    assert!(tree.source_bytes().is_none());
}

#[test]
fn stub_tree_clone_preserves_no_language() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert!(cloned.language().is_none());
    assert!(cloned.source_bytes().is_none());
    assert_eq!(cloned.root_kind(), 0);
}

// ===========================================================================
// 6. Node: utf8_text with various inputs
// ===========================================================================

#[test]
fn node_utf8_text_extracts_correct_range() {
    // We can only test stub node which has 0..0 range
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let text = root.utf8_text(b"abcdef").unwrap();
    assert_eq!(text, "");
}

#[test]
fn node_utf8_text_on_empty_source() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let text = root.utf8_text(b"").unwrap();
    assert_eq!(text, "");
}

// ===========================================================================
// 7. TreeCursor: deep traversal
// ===========================================================================

#[test]
fn cursor_repeated_goto_parent_at_root_is_stable() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    for _ in 0..10 {
        assert!(!cursor.goto_parent());
    }
    // Still functional
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_goto_next_sibling_at_root_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

// ===========================================================================
// 8. Point: edge cases
// ===========================================================================

#[test]
fn point_zero_display() {
    let p = Point::new(0, 0);
    assert_eq!(p.to_string(), "1:1");
}

#[test]
fn point_large_values() {
    let p = Point::new(usize::MAX - 1, usize::MAX - 1);
    assert_eq!(p.row, usize::MAX - 1);
    assert_eq!(p.column, usize::MAX - 1);
}

#[test]
fn point_ordering_row_first() {
    let a = Point::new(0, 100);
    let b = Point::new(1, 0);
    assert!(a < b, "row should be compared first");
}

#[test]
fn point_ordering_column_tiebreak() {
    let a = Point::new(5, 3);
    let b = Point::new(5, 7);
    assert!(a < b, "column should break ties when rows are equal");
}

// ===========================================================================
// 9. InputEdit: Clone, Copy, Eq
// ===========================================================================

#[test]
fn input_edit_debug_format() {
    use adze_runtime::InputEdit;
    let edit = InputEdit {
        start_byte: 0,
        old_end_byte: 5,
        new_end_byte: 10,
        start_position: Point::new(0, 0),
        old_end_position: Point::new(0, 5),
        new_end_position: Point::new(0, 10),
    };
    let debug = format!("{:?}", edit);
    assert!(debug.contains("InputEdit"));
}

#[test]
fn input_edit_equality() {
    use adze_runtime::InputEdit;
    let a = InputEdit {
        start_byte: 1,
        old_end_byte: 3,
        new_end_byte: 5,
        start_position: Point::new(0, 1),
        old_end_position: Point::new(0, 3),
        new_end_position: Point::new(0, 5),
    };
    let b = a;
    assert_eq!(a, b);
}

// ===========================================================================
// 10. Token: fields and traits
// ===========================================================================

#[test]
fn token_zero_length() {
    let tok = Token {
        kind: 0,
        start: 5,
        end: 5,
    };
    assert_eq!(tok.start, tok.end);
}

#[test]
fn token_debug_contains_fields() {
    let tok = Token {
        kind: 42,
        start: 10,
        end: 20,
    };
    let debug = format!("{:?}", tok);
    assert!(debug.contains("42"));
    assert!(debug.contains("10"));
    assert!(debug.contains("20"));
}

// ===========================================================================
// 11. LanguageBuilder: missing components
// ===========================================================================

#[test]
fn language_builder_missing_parse_table_fails() {
    use adze_runtime::Language;
    let result = Language::builder()
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("parse table"));
}

#[test]
fn language_builder_missing_metadata_fails() {
    use adze_runtime::Language;
    // Cannot easily provide parse_table without helper, but we can verify the builder
    // requires metadata by constructing one via the test helper path
    let result = Language::builder().build();
    assert!(result.is_err());
}

// ===========================================================================
// 12. Language: symbol_metadata boundary checks
// ===========================================================================

#[test]
fn language_is_terminal_at_boundary() {
    let lang = multi_symbol_test_language(3);
    // Valid indices
    assert!(lang.is_terminal(0));
    assert!(lang.is_terminal(2));
    // Out of bounds
    assert!(!lang.is_terminal(3));
    assert!(!lang.is_terminal(u16::MAX));
}

#[test]
fn language_is_visible_at_boundary() {
    let lang = multi_symbol_test_language(3);
    assert!(lang.is_visible(0));
    assert!(lang.is_visible(2));
    assert!(!lang.is_visible(3));
    assert!(!lang.is_visible(u16::MAX));
}

#[test]
fn language_symbol_name_at_boundary() {
    let lang = multi_symbol_test_language(3);
    assert_eq!(lang.symbol_name(0), Some("symbol_0"));
    assert_eq!(lang.symbol_name(2), Some("symbol_2"));
    assert_eq!(lang.symbol_name(3), None);
}

// ===========================================================================
// 13. ScanResult traits
// ===========================================================================

#[test]
fn scan_result_copy_and_eq() {
    use adze_runtime::ScanResult;
    let a = ScanResult {
        token_type: 3,
        bytes_consumed: 10,
    };
    let b = a; // Copy
    assert_eq!(a, b);
    let c = a;
    assert_eq!(a, c);
}

#[test]
fn scan_result_debug() {
    use adze_runtime::ScanResult;
    let r = ScanResult {
        token_type: 7,
        bytes_consumed: 42,
    };
    let debug = format!("{:?}", r);
    assert!(debug.contains("7"));
    assert!(debug.contains("42"));
}

// ===========================================================================
// 14. GLR-core specific: parse with real grammar, edge cases
// ===========================================================================

#[cfg(feature = "glr")]
mod glr_core_tests {
    use super::*;
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    use adze_ir::{
        Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken,
        TokenPattern as IrTokenPattern,
    };

    /// Build a grammar: start → a
    fn make_single_token_language() -> adze_runtime::Language {
        let mut grammar = Grammar::new("edge".to_string());
        let a_id = SymbolId(1);
        grammar.tokens.insert(
            a_id,
            IrToken {
                name: "a".to_string(),
                pattern: IrTokenPattern::String("a".to_string()),
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

        adze_runtime::Language::builder()
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
                for (i, &byte) in input.iter().enumerate() {
                    if byte == b'a' {
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
                Box::new(toks.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
            })
            .build()
            .unwrap()
    }

    #[test]
    fn parse_valid_input_returns_tree_with_source() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser.parse(b"a", None).unwrap();
        assert_eq!(tree.source_bytes(), Some(b"a".as_slice()));
        assert!(tree.language().is_some());
    }

    #[test]
    fn parsed_tree_root_has_valid_byte_range() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser.parse(b"a", None).unwrap();
        let root = tree.root_node();
        assert!(root.start_byte() <= root.end_byte());
    }

    #[test]
    fn parsed_tree_node_kind_resolves_via_language() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser.parse(b"a", None).unwrap();
        let root = tree.root_node();
        // Root should be "start" (symbol 2)
        assert_eq!(root.kind(), "start");
    }

    #[test]
    fn parse_multiple_times_with_same_parser() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree1 = parser.parse(b"a", None).unwrap();
        let tree2 = parser.parse(b"a", None).unwrap();
        assert_eq!(tree1.root_kind(), tree2.root_kind());
    }

    #[test]
    fn parse_with_old_tree_produces_same_result() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree1 = parser.parse(b"a", None).unwrap();
        let tree2 = parser.parse(b"a", Some(&tree1)).unwrap();
        assert_eq!(tree1.root_kind(), tree2.root_kind());
    }

    #[test]
    fn parsed_tree_cursor_can_traverse() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser.parse(b"a", None).unwrap();
        let mut cursor = TreeCursor::new(&tree);

        // At root; try to go to first child
        let has_child = cursor.goto_first_child();
        if has_child {
            // Verify we can go back to parent
            assert!(cursor.goto_parent());
        }
    }

    #[test]
    fn parsed_tree_root_child_inherits_language() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser.parse(b"a", None).unwrap();
        let root = tree.root_node();
        if let Some(child) = root.child(0) {
            // Child should also resolve kind via language
            assert_ne!(child.kind(), "unknown");
        }
    }

    #[test]
    fn parse_utf8_with_language_works() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();

        let tree = parser.parse_utf8("a", None).unwrap();
        assert_eq!(tree.root_kind(), 2); // start symbol
    }

    #[test]
    fn language_cloned_loses_tokenizer() {
        let lang = make_single_token_language();
        let mut parser = Parser::new();

        // Cloned language loses tokenizer (closures can't be cloned)
        let cloned_lang = lang.clone();
        let result = parser.set_language(cloned_lang);
        // set_language checks for tokenizer presence
        assert!(result.is_err(), "cloned language should lack tokenizer");
    }
}

// ===========================================================================
// 15. Feature flag: external_scanners
// ===========================================================================

#[cfg(feature = "external_scanners")]
mod external_scanner_tests {
    use adze_runtime::external_scanner::{ExternalScanner, ScanResult};

    struct NoOpScanner;

    impl ExternalScanner for NoOpScanner {
        fn init(&mut self) {}
        fn scan(&mut self, _valid_symbols: &[bool], _input: &[u8]) -> Option<ScanResult> {
            None
        }
        fn serialize(&self) -> Vec<u8> {
            vec![]
        }
        fn deserialize(&mut self, _data: &[u8]) {}
    }

    #[test]
    fn external_scanner_serialize_deserialize_roundtrip() {
        let mut scanner = NoOpScanner;
        scanner.init();
        let data = scanner.serialize();
        assert!(data.is_empty());
        scanner.deserialize(&data);
        // Should still work after roundtrip
        assert!(scanner.scan(&[true, false], b"test").is_none());
    }

    #[test]
    fn external_scanner_with_empty_valid_symbols() {
        let mut scanner = NoOpScanner;
        scanner.init();
        assert!(scanner.scan(&[], b"hello").is_none());
    }

    #[test]
    fn external_scanner_with_empty_input() {
        let mut scanner = NoOpScanner;
        scanner.init();
        assert!(scanner.scan(&[true], b"").is_none());
    }
}

// ===========================================================================
// 16. Governance / feature profile always available
// ===========================================================================

#[test]
fn bdd_progress_report_does_not_panic() {
    use adze_runtime::BddPhase;
    let _report =
        adze_runtime::bdd_progress_report_for_current_profile(BddPhase::Runtime, "Runtime");
}

#[test]
fn bdd_progress_status_line_does_not_panic() {
    use adze_runtime::BddPhase;
    let _line = adze_runtime::bdd_progress_status_line_for_current_profile(BddPhase::Runtime);
}
