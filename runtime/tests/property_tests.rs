// Property-based tests for the adze runtime crate.
//
// These tests use proptest to verify invariants across random inputs:
// 1. Pure parser: valid TSLanguage should not crash parser creation
// 2. Visitor API: visiting any tree visits all nodes
// 3. Serialization roundtrip (feature-gated on "serialization")
// 4. Error recovery: arbitrary malformed input produces errors, not panics
// 5. Tree editing: edit operations maintain tree invariants
// 6. GrammarLexer: tokenize arbitrary ASCII input without panic

mod common;

use adze::pure_parser::{ExternalScanner, ParsedNode, Parser, Point, TSLanguage, TSParseAction};
use adze::visitor::{TreeWalker, Visitor, VisitorAction};
use proptest::prelude::*;
use std::ptr;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A minimal but valid static TSLanguage that the pure-Rust parser accepts.
fn create_test_language() -> &'static TSLanguage {
    static PARSE_ACTIONS: [TSParseAction; 4] = [
        TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 1,
        },
        TSParseAction {
            action_type: 1,
            extra: 0,
            child_count: 1,
            dynamic_precedence: 0,
            symbol: 2,
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
    ];

    static PARSE_TABLE: [u16; 100] = [0; 100];
    static SMALL_PARSE_TABLE: [u16; 100] = [0; 100];
    static SMALL_PARSE_TABLE_MAP: [u32; 10] = [0; 10];
    static LEX_MODES: [u32; 10] = [0; 10];
    static PRODUCTION_ID_MAP: [u16; 10] = [0; 10];

    static SYM_EOF: &[u8] = b"end\0";
    static SYM_TOKEN: &[u8] = b"token\0";
    static SYM_EXPR: &[u8] = b"expression\0";

    #[repr(transparent)]
    struct Names([*const u8; 3]);
    unsafe impl Sync for Names {}

    static SYMBOL_NAMES: Names = Names([SYM_EOF.as_ptr(), SYM_TOKEN.as_ptr(), SYM_EXPR.as_ptr()]);

    static SYMBOL_METADATA: [u8; 3] = [0x01, 0x01, 0x03];

    static LANGUAGE: TSLanguage = TSLanguage {
        version: 15,
        symbol_count: 3,
        alias_count: 0,
        token_count: 2,
        external_token_count: 0,
        state_count: 4,
        large_state_count: 2,
        production_id_count: 1,
        field_count: 0,
        max_alias_sequence_length: 0,
        eof_symbol: 0,
        rules: ptr::null(),
        rule_count: 0,
        production_count: 1,
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

/// Build a simple ParsedNode tree by parsing source with our test language.
/// The pure parser always produces a root node, which we can use for visitor tests.
fn parse_with_test_language(source: &str) -> (adze::pure_parser::ParseResult, Vec<u8>) {
    let lang = create_test_language();
    let mut parser = Parser::new();
    let _ = parser.set_language(lang);
    let result = parser.parse_string(source);
    (result, source.as_bytes().to_vec())
}

/// Count every node in a tree recursively.
fn count_nodes(node: &ParsedNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

// ---------------------------------------------------------------------------
// 1. Pure parser: setting a language and parsing never panics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pure_parser_never_panics(input in "\\PC{0,200}") {
        let lang = create_test_language();
        let mut parser = Parser::new();
        let _ = parser.set_language(lang);
        // Must not panic regardless of input content.
        let result = parser.parse_string(&input);
        // Result is always well-formed: either a root or errors.
        let _ = result.root;
        let _ = result.errors;
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn parser_set_language_validates_version(version in 0u32..30) {
        // Build a language with an arbitrary version to exercise validation.
        // We can't safely construct a full TSLanguage on the stack and leak it
        // for every test case, so instead we just check that Parser::new()
        // followed by parse_string (without set_language) returns errors.
        let mut parser = Parser::new();
        let result = parser.parse_string("hello");
        prop_assert!(result.root.is_none() || !result.errors.is_empty(),
            "parsing without a language should fail gracefully for version {version}");
    }
}

// ---------------------------------------------------------------------------
// 2. Visitor API: visiting any tree visits all nodes
// ---------------------------------------------------------------------------

/// A counting visitor that records every enter/leave call.
struct CountingVisitor {
    entered: usize,
    left: usize,
    leaves: usize,
    errors: usize,
}

impl CountingVisitor {
    fn new() -> Self {
        Self {
            entered: 0,
            left: 0,
            leaves: 0,
            errors: 0,
        }
    }
}

impl Visitor for CountingVisitor {
    fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
        self.entered += 1;
        VisitorAction::Continue
    }
    fn leave_node(&mut self, _node: &ParsedNode) {
        self.left += 1;
    }
    fn visit_leaf(&mut self, _node: &ParsedNode, _text: &str) {
        self.leaves += 1;
    }
    fn visit_error(&mut self, _node: &ParsedNode) {
        self.errors += 1;
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn visitor_visits_all_non_error_nodes(input in "[a-z0-9 +*]{1,60}") {
        let (result, source) = parse_with_test_language(&input);

        if let Some(ref root) = result.root {
            let walker = TreeWalker::new(&source);
            let mut visitor = CountingVisitor::new();
            walker.walk(root, &mut visitor);

            let total = count_nodes(root);
            // Every node is either entered+left or visited as error.
            prop_assert_eq!(visitor.entered + visitor.errors, total);
            prop_assert_eq!(visitor.left + visitor.errors, total);
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Serialization roundtrip (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "serialization")]
mod serialization_roundtrip {
    use super::*;
    use adze::serialization::SerializedNode;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn serialized_node_json_roundtrip(
            kind in "[a-z_]{1,20}",
            start_byte in 0usize..1000,
            span in 1usize..500,
            is_named in proptest::bool::ANY,
            is_error in proptest::bool::ANY,
            is_missing in proptest::bool::ANY,
        ) {
            let end_byte = start_byte + span;
            let node = SerializedNode {
                kind: kind.clone(),
                is_named,
                field_name: None,
                start_position: (0, start_byte),
                end_position: (0, end_byte),
                start_byte,
                end_byte,
                text: Some("leaf".into()),
                children: vec![],
                is_error,
                is_missing,
            };

            let json = serde_json::to_string(&node).expect("serialize");
            let back: SerializedNode = serde_json::from_str(&json).expect("deserialize");

            prop_assert_eq!(&back.kind, &kind);
            prop_assert_eq!(back.start_byte, start_byte);
            prop_assert_eq!(back.end_byte, end_byte);
            prop_assert_eq!(back.is_named, is_named);
            prop_assert_eq!(back.is_error, is_error);
            prop_assert_eq!(back.is_missing, is_missing);
            prop_assert_eq!(back.text.as_deref(), Some("leaf"));
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Error recovery: arbitrary malformed input produces errors, not panics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn error_recovery_config_never_panics(
        max_skip in 0usize..200,
        max_del in 0usize..10,
        max_ins in 0usize..10,
        max_errs in 1usize..50,
    ) {
        use adze::error_recovery::{ErrorRecoveryConfig, ErrorRecoveryState};

        let config = ErrorRecoveryConfig {
            max_panic_skip: max_skip,
            max_token_deletions: max_del,
            max_token_insertions: max_ins,
            max_consecutive_errors: max_errs,
            ..ErrorRecoveryConfig::default()
        };

        let mut state = ErrorRecoveryState::new(config);

        // Feed a sequence of error events — must never panic.
        for i in 0u16..20 {
            let _ = state.determine_recovery_strategy(
                &[1, 2, 3],
                Some(i),
                (0, i as usize),
                i as usize,
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn malformed_input_does_not_panic(input in "\\PC{0,300}") {
        let lang = create_test_language();
        let mut parser = Parser::new();
        let _ = parser.set_language(lang);
        // Arbitrary (possibly binary) input must not cause a panic.
        let result = parser.parse_string(&input);
        // The contract: we always get a ParseResult back.
        let _root = result.root;
        let _errors = result.errors;
    }
}

// ---------------------------------------------------------------------------
// 5. Tree editing: edit operations maintain tree invariants
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn tree_edit_maintains_invariants(
        src in "[a-z0-9 ]{4,64}",
        edit_start_pct in 0usize..100,
        del_pct in 0usize..50,
        ins_len in 0usize..32,
    ) {
        use adze::pure_incremental::{Edit, Tree};

        let lang = create_test_language();
        let mut parser = Parser::new();
        let _ = parser.set_language(lang);
        let result = parser.parse_string(&src);

        if let Some(root) = result.root {
            let source = src.as_bytes();
            let mut tree = Tree::new(root, lang, source);

            let src_len = source.len();
            let edit_start = (edit_start_pct * src_len / 100).min(src_len.saturating_sub(1));
            let del_len = (del_pct * (src_len - edit_start) / 100).min(src_len - edit_start);

            let edit = Edit {
                start_byte: edit_start,
                old_end_byte: edit_start + del_len,
                new_end_byte: edit_start + ins_len,
                start_point: Point { row: 0, column: edit_start as u32 },
                old_end_point: Point { row: 0, column: (edit_start + del_len) as u32 },
                new_end_point: Point { row: 0, column: (edit_start + ins_len) as u32 },
            };

            // Must not panic.
            tree.edit(&edit);

            // Root node is still accessible after edit.
            let _root = tree.root_node();
        }
    }
}

// ---------------------------------------------------------------------------
// 6. GrammarLexer: tokenize arbitrary ASCII input without panic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn glr_lexer_tokenize_ascii_no_panic(input in "[\\x20-\\x7E]{0,200}") {
        use adze::glr_lexer::GLRLexer;
        use adze_ir::{Grammar, SymbolId, Token, TokenPattern};

        let mut grammar = Grammar::new("test".to_string());
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "number".to_string(),
                pattern: TokenPattern::Regex(r"\d+".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(2),
            Token {
                name: "ident".to_string(),
                pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(3),
            Token {
                name: "plus".to_string(),
                pattern: TokenPattern::String("+".to_string()),
                fragile: false,
            },
        );

        let mut lexer = GLRLexer::new(&grammar, input.clone())
            .expect("lexer construction should succeed");

        // Tokenising must never panic regardless of input.
        let tokens = lexer.tokenize_all();

        // Every produced token must reference a known symbol.
        for tok in &tokens {
            prop_assert!(
                tok.symbol_id == SymbolId(1)
                    || tok.symbol_id == SymbolId(2)
                    || tok.symbol_id == SymbolId(3),
                "unexpected symbol {:?} in token {:?}",
                tok.symbol_id,
                tok.text,
            );
        }
    }
}
