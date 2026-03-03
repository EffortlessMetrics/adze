//! Integration tests for the runtime parse pipeline.
//!
//! Exercises: parser creation → parse → extract, error recovery,
//! visitor traversal, and (when `serialization` is enabled) roundtrip
//! serialization.

#![cfg(test)]
#![allow(unused_imports, dead_code)]

use adze::pure_parser::{
    ExternalScanner, ParseResult, ParsedNode, Parser, Point, TSLanguage, TSLexState, TSParseAction,
    TSRule,
};
use std::ptr;

// ---------------------------------------------------------------------------
// Shared test language: a tiny grammar with 7 symbols.
//
// Symbols: EOF(0), digit(1), plus(2), multiply(3), number(4), addition(5),
//          multiplication(6)
//
// The parse table is intentionally zeroed – the pure-rust parser falls back
// to error-recovery/iteration-limit paths, which is exactly what we want
// for testing the pipeline *around* the parser without requiring a fully
// wired grammar (those are covered by golden tests).
// ---------------------------------------------------------------------------

fn test_language() -> &'static TSLanguage {
    static PARSE_ACTIONS: [TSParseAction; 10] = [
        TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 1,
        },
        TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 2,
        },
        TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 3,
        },
        TSParseAction {
            action_type: 1,
            extra: 0,
            child_count: 1,
            dynamic_precedence: 0,
            symbol: 4,
        },
        TSParseAction {
            action_type: 1,
            extra: 0,
            child_count: 3,
            dynamic_precedence: 0,
            symbol: 5,
        },
        TSParseAction {
            action_type: 1,
            extra: 0,
            child_count: 3,
            dynamic_precedence: 0,
            symbol: 6,
        },
        TSParseAction {
            action_type: 2,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        },
        TSParseAction {
            action_type: 3,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        },
        TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        },
        TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        },
    ];

    static PARSE_TABLE: [u16; 100] = [0; 100];
    static SMALL_PARSE_TABLE: [u16; 100] = [0; 100];
    static SMALL_PARSE_TABLE_MAP: [u32; 10] = [0; 10];
    static LEX_MODES: [u32; 10] = [0; 10];
    static PRODUCTION_ID_MAP: [u16; 10] = [0; 10];

    static NAME_EOF: &[u8] = b"end\0";
    static NAME_DIGIT: &[u8] = b"digit\0";
    static NAME_PLUS: &[u8] = b"+\0";
    static NAME_MUL: &[u8] = b"*\0";
    static NAME_NUMBER: &[u8] = b"number\0";
    static NAME_ADD: &[u8] = b"addition\0";
    static NAME_MULT: &[u8] = b"multiplication\0";

    #[repr(transparent)]
    struct Names([*const u8; 7]);
    unsafe impl Sync for Names {}

    static SYMBOL_NAMES: Names = Names([
        NAME_EOF.as_ptr(),
        NAME_DIGIT.as_ptr(),
        NAME_PLUS.as_ptr(),
        NAME_MUL.as_ptr(),
        NAME_NUMBER.as_ptr(),
        NAME_ADD.as_ptr(),
        NAME_MULT.as_ptr(),
    ]);

    static SYMBOL_METADATA: [u8; 7] = [
        0x01, // EOF: visible
        0x01, // digit: visible terminal
        0x01, // plus: visible terminal
        0x01, // multiply: visible terminal
        0x03, // number: visible + named
        0x03, // addition: visible + named
        0x03, // multiplication: visible + named
    ];

    static LANGUAGE: TSLanguage = TSLanguage {
        version: 15,
        symbol_count: 7,
        alias_count: 0,
        token_count: 4,
        external_token_count: 0,
        state_count: 10,
        large_state_count: 5,
        production_id_count: 3,
        field_count: 0,
        max_alias_sequence_length: 0,
        eof_symbol: 0,
        rules: ptr::null(),
        rule_count: 0,
        production_count: 3,
        production_lhs_index: ptr::null(),
        production_id_map: PRODUCTION_ID_MAP.as_ptr(),
        parse_table: PARSE_TABLE.as_ptr(),
        small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
        small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
        parse_actions: PARSE_ACTIONS.as_ptr(),
        symbol_names: SYMBOL_NAMES.0.as_ptr(),
        field_names: ptr::null(),
        field_map_slices: ptr::null(),
        field_map_entries: ptr::null(),
        symbol_metadata: SYMBOL_METADATA.as_ptr(),
        public_symbol_map: ptr::null(),
        alias_map: ptr::null(),
        alias_sequences: ptr::null(),
        lex_modes: LEX_MODES.as_ptr() as *const _,
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: ExternalScanner::default(),
        primary_state_ids: ptr::null(),
    };

    &LANGUAGE
}

// ===== 1. Parse → extract pipeline =====

#[test]
fn parse_pipeline_produces_root_or_errors() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let result = parser.parse_string("1 + 2");

    // A well-formed pipeline always yields a root OR non-empty errors.
    assert!(
        result.root.is_some() || !result.errors.is_empty(),
        "parse pipeline must produce a root node or report errors"
    );
}

#[test]
fn parse_pipeline_empty_input() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let result = parser.parse_string("");
    assert!(
        result.root.is_some() || !result.errors.is_empty(),
        "empty input must not silently disappear"
    );
}

#[test]
fn parse_pipeline_root_spans_input() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let source = "42";
    let result = parser.parse_string(source);
    if let Some(ref root) = result.root {
        // Root should start at byte 0.
        assert_eq!(root.start_byte(), 0, "root must start at byte 0");
    }
}

// ===== 2. Error recovery: malformed input → errors, not panics =====

#[test]
fn error_recovery_does_not_panic_on_garbage() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let result = parser.parse_string("@#$%^&!");
    // We only care that this doesn't panic.
    assert!(
        result.root.is_some() || !result.errors.is_empty(),
        "garbage input must not panic"
    );
}

#[test]
fn error_recovery_does_not_panic_on_deeply_nested() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let deep = "(".repeat(200);
    let result = parser.parse_string(&deep);
    assert!(
        result.root.is_some() || !result.errors.is_empty(),
        "deeply nested input must not panic"
    );
}

#[test]
fn error_recovery_does_not_panic_on_long_input() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let long_input = "1 + ".repeat(500);
    let result = parser.parse_string(&long_input);
    assert!(
        result.root.is_some() || !result.errors.is_empty(),
        "long input must not panic"
    );
}

#[test]
fn parse_without_language_returns_error() {
    let mut parser = Parser::new();
    // Deliberately skip set_language
    let result = parser.parse_string("1 + 2");
    assert!(
        result.root.is_none(),
        "parsing without a language must not produce a root"
    );
    assert!(
        !result.errors.is_empty(),
        "parsing without a language must report an error"
    );
}

// ===== 3. Visitor API: traverse a parsed tree and collect node kinds =====

#[cfg(feature = "pure-rust")]
mod visitor_tests {
    use super::*;
    use adze::visitor::{TreeWalker, Visitor, VisitorAction};

    struct KindCollector {
        kinds: Vec<String>,
    }

    impl Visitor for KindCollector {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.kinds.push(node.kind().to_string());
            VisitorAction::Continue
        }
    }

    #[test]
    fn visitor_collects_node_kinds() {
        let mut parser = Parser::new();
        parser.set_language(test_language()).unwrap();

        let result = parser.parse_string("1 + 2");
        if let Some(ref root) = result.root {
            let source = b"1 + 2";
            let walker = TreeWalker::new(source);
            let mut collector = KindCollector { kinds: Vec::new() };
            walker.walk(root, &mut collector);

            // The visitor should have been called at least once (for the root).
            assert!(
                !collector.kinds.is_empty(),
                "visitor must visit at least the root node"
            );
        }
    }

    struct StopAfterFirst {
        count: usize,
    }

    impl Visitor for StopAfterFirst {
        fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
            self.count += 1;
            VisitorAction::Stop
        }
    }

    #[test]
    fn visitor_stop_action_halts_traversal() {
        let mut parser = Parser::new();
        parser.set_language(test_language()).unwrap();

        let result = parser.parse_string("1 + 2");
        if let Some(ref root) = result.root {
            let walker = TreeWalker::new(b"1 + 2");
            let mut stopper = StopAfterFirst { count: 0 };
            walker.walk(root, &mut stopper);

            assert_eq!(stopper.count, 1, "Stop should halt after the first node");
        }
    }

    struct SkipChildrenCollector {
        kinds: Vec<String>,
    }

    impl Visitor for SkipChildrenCollector {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.kinds.push(node.kind().to_string());
            VisitorAction::SkipChildren
        }
    }

    #[test]
    fn visitor_skip_children_action() {
        let mut parser = Parser::new();
        parser.set_language(test_language()).unwrap();

        let result = parser.parse_string("1 + 2");
        if let Some(ref root) = result.root {
            let walker = TreeWalker::new(b"1 + 2");
            let mut skipper = SkipChildrenCollector { kinds: Vec::new() };
            walker.walk(root, &mut skipper);

            // Only root should appear because children are skipped.
            assert_eq!(
                skipper.kinds.len(),
                1,
                "SkipChildren should only visit root"
            );
        }
    }
}

// ===== 4. Pure parser creation and basic parsing =====

#[test]
fn pure_parser_new_has_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn pure_parser_set_language_succeeds() {
    let mut parser = Parser::new();
    assert!(parser.set_language(test_language()).is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn pure_parser_rejects_invalid_version() {
    let mut parser = Parser::new();

    static BAD_LANG: TSLanguage = TSLanguage {
        version: 999,
        symbol_count: 0,
        alias_count: 0,
        token_count: 0,
        external_token_count: 0,
        state_count: 0,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        eof_symbol: 0,
        rules: ptr::null(),
        rule_count: 0,
        production_count: 0,
        production_lhs_index: ptr::null(),
        production_id_map: ptr::null(),
        parse_table: ptr::null(),
        small_parse_table: ptr::null(),
        small_parse_table_map: ptr::null(),
        parse_actions: ptr::null(),
        symbol_names: ptr::null(),
        field_names: ptr::null(),
        field_map_slices: ptr::null(),
        field_map_entries: ptr::null(),
        symbol_metadata: ptr::null(),
        public_symbol_map: ptr::null(),
        alias_map: ptr::null(),
        alias_sequences: ptr::null(),
        lex_modes: ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: ExternalScanner::default(),
        primary_state_ids: ptr::null(),
    };

    assert!(parser.set_language(&BAD_LANG).is_err());
}

#[test]
fn pure_parser_parse_bytes_matches_parse_string() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let source = "1 + 2";
    let from_string = parser.parse_string(source);
    let from_bytes = parser.parse_bytes(source.as_bytes());

    // Both paths should agree on whether a root was produced.
    assert_eq!(from_string.root.is_some(), from_bytes.root.is_some());
}

#[test]
fn pure_parser_timeout_terminates() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();
    parser.set_timeout_micros(1); // extremely short

    let result = parser.parse_string("1 + 2 * 3 + 4 * 5");
    // Must complete (possibly with errors) rather than hang.
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn pure_parser_cancellation_works() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let cancel = AtomicBool::new(true);
    parser.set_cancellation_flag(Some(&cancel as *const AtomicBool));

    let result = parser.parse_string("1 + 2");
    assert!(
        result.root.is_some() || !result.errors.is_empty(),
        "cancelled parse must not hang"
    );
}

// ===== 5. Tree serialization roundtrip (serialization feature) =====

#[cfg(feature = "serialization")]
mod serialization_tests {
    use super::*;
    use adze::pure_incremental::Tree;
    use adze::serialization::{SerializedNode, TreeSerializer};

    #[test]
    fn serialize_tree_to_json_roundtrip() {
        let mut parser = Parser::new();
        let lang = test_language();
        parser.set_language(lang).unwrap();

        let source = "42";
        let result = parser.parse_string(source);

        if let Some(root) = result.root {
            let tree = Tree::new(root, lang, source.as_bytes());
            let serializer = TreeSerializer::new(source.as_bytes());

            let json = serializer
                .serialize_tree(&tree)
                .expect("serialize must succeed");
            assert!(!json.is_empty(), "JSON output must not be empty");

            // Deserialize back and verify structure is preserved.
            let deserialized: SerializedNode =
                serde_json::from_str(&json).expect("JSON must be valid");
            assert_eq!(deserialized.start_byte, 0, "root start_byte must be 0");
        }
    }

    #[test]
    fn serialize_includes_unnamed_nodes() {
        let mut parser = Parser::new();
        let lang = test_language();
        parser.set_language(lang).unwrap();

        let source = "1";
        let result = parser.parse_string(source);

        if let Some(root) = result.root {
            let tree = Tree::new(root, lang, source.as_bytes());
            let serializer = TreeSerializer::new(source.as_bytes()).with_unnamed_nodes();

            let json = serializer
                .serialize_tree(&tree)
                .expect("serialize must succeed");
            assert!(!json.is_empty());
        }
    }
}
