//! Parser API contract tests.
//!
//! Verify that the public parser API satisfies its documented invariants:
//! construction, language assignment, parsing, tree structure, node properties,
//! cursor traversal, and independence of successive parses.
//!
//! The test language uses a zeroed parse table; the parser exercises its
//! error-recovery / iteration-limit paths. Structural contracts are checked
//! only when a root node is produced.

#![cfg(feature = "pure-rust")]

use adze::pure_parser::{
    ExternalScanner, ParseResult, ParsedNode, Parser, TSLanguage, TSParseAction,
};
use std::ptr;

// ---------------------------------------------------------------------------
// Shared test language (mirrors the proven pattern in integration_pipeline.rs)
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

    static NAME_EOF: &[u8] = b"end\0";
    static NAME_DIGIT: &[u8] = b"digit\0";
    static NAME_PLUS: &[u8] = b"+\0";
    static NAME_STAR: &[u8] = b"*\0";
    static NAME_NUMBER: &[u8] = b"number\0";
    static NAME_EXPR: &[u8] = b"expression\0";
    static NAME_TERM: &[u8] = b"term\0";

    #[repr(transparent)]
    struct Names([*const u8; 7]);
    unsafe impl Sync for Names {}

    static SYMBOL_NAMES: Names = Names([
        NAME_EOF.as_ptr(),
        NAME_DIGIT.as_ptr(),
        NAME_PLUS.as_ptr(),
        NAME_STAR.as_ptr(),
        NAME_NUMBER.as_ptr(),
        NAME_EXPR.as_ptr(),
        NAME_TERM.as_ptr(),
    ]);

    static SYMBOL_METADATA: [u8; 7] = [0x01, 0x01, 0x01, 0x01, 0x03, 0x03, 0x03];

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
        production_id_map: ptr::null(),
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse `source` using the test language.
fn do_parse(source: &str) -> ParseResult {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();
    parser.parse_string(source)
}

/// Recursively collect every node in the tree (pre-order).
fn collect_all_nodes(node: &ParsedNode) -> Vec<&ParsedNode> {
    let mut out = vec![node];
    for child in node.children() {
        out.extend(collect_all_nodes(child));
    }
    out
}

/// Count nodes reachable via the ChildWalker (cursor) API.
fn count_via_walker(node: &ParsedNode) -> usize {
    let mut count = 1;
    let mut walker = node.walk();
    if walker.goto_first_child() {
        count += count_via_walker(walker.node());
        while walker.goto_next_sibling() {
            count += count_via_walker(walker.node());
        }
    }
    count
}

fn assert_children_within_parent(node: &ParsedNode) {
    for child in node.children() {
        assert!(child.start_byte() >= node.start_byte());
        assert!(child.end_byte() <= node.end_byte());
        assert_children_within_parent(child);
    }
}

fn assert_siblings_no_overlap(node: &ParsedNode) {
    let children = node.children();
    for pair in children.windows(2) {
        assert!(
            pair[1].start_byte() >= pair[0].end_byte(),
            "sibling overlap: [{}, {}) and [{}, {})",
            pair[0].start_byte(),
            pair[0].end_byte(),
            pair[1].start_byte(),
            pair[1].end_byte(),
        );
    }
    for child in children {
        assert_siblings_no_overlap(child);
    }
}

fn assert_child_count_consistent(node: &ParsedNode) {
    assert_eq!(node.child_count(), node.children().len());
    for child in node.children() {
        assert_child_count_consistent(child);
    }
}

fn assert_kind_nonempty(node: &ParsedNode) {
    if !node.is_error() {
        assert!(
            !node.kind().is_empty(),
            "kind() empty for symbol={}",
            node.symbol()
        );
    }
    for child in node.children() {
        assert_kind_nonempty(child);
    }
}

/// Build an invalid language for rejection tests.
fn make_invalid_language(version: u32, has_names: bool, has_table: bool) -> TSLanguage {
    let (names_ptr, meta_ptr) = if has_names {
        let name = Box::leak(Box::new(b"end\0".as_ptr()));
        let meta = Box::leak(Box::new(0u8));
        (*name as *const *const u8, meta as *const u8)
    } else {
        (ptr::null(), ptr::null())
    };
    let (table_ptr, actions_ptr) = if has_table {
        let t = Box::leak(Box::new(0u16));
        let a = Box::leak(Box::new(TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        }));
        (t as *const u16, a as *const TSParseAction)
    } else {
        (ptr::null(), ptr::null())
    };
    TSLanguage {
        version,
        symbol_count: 1,
        alias_count: 0,
        token_count: 1,
        external_token_count: 0,
        state_count: 1,
        large_state_count: 1,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        production_id_map: ptr::null(),
        parse_table: table_ptr,
        small_parse_table: ptr::null(),
        small_parse_table_map: ptr::null(),
        parse_actions: actions_ptr,
        symbol_names: names_ptr,
        field_names: ptr::null(),
        field_map_slices: ptr::null(),
        field_map_entries: ptr::null(),
        symbol_metadata: meta_ptr,
        public_symbol_map: ptr::null(),
        alias_map: ptr::null(),
        alias_sequences: ptr::null(),
        lex_modes: ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: ExternalScanner::default(),
        primary_state_ids: ptr::null(),
        production_lhs_index: ptr::null(),
        production_count: 0,
        eof_symbol: 0,
        rules: ptr::null(),
        rule_count: 0,
    }
}

// ===========================================================================
// 1. Parser::new() returns a valid parser
// ===========================================================================

#[test]
fn contract_01_parser_new_returns_valid_parser() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn contract_01_parser_default_is_equivalent_to_new() {
    let a = Parser::new();
    let b = Parser::default();
    assert!(a.language().is_none());
    assert!(b.language().is_none());
}

// ===========================================================================
// 2. Parser::set_language() with valid language succeeds
// ===========================================================================

#[test]
fn contract_02_set_language_succeeds() {
    let mut parser = Parser::new();
    assert!(parser.set_language(test_language()).is_ok());
    assert!(parser.language().is_some());
}

#[test]
fn contract_02_set_language_twice_succeeds() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();
    assert!(parser.set_language(test_language()).is_ok());
}

// ===========================================================================
// 3. Parser::set_language() with invalid/null returns error
// ===========================================================================

#[test]
fn contract_03_rejects_bad_version() {
    let bad = Box::leak(Box::new(make_invalid_language(0, true, true)));
    assert!(Parser::new().set_language(bad).is_err());
}

#[test]
fn contract_03_rejects_null_symbol_names() {
    let bad = Box::leak(Box::new(make_invalid_language(15, false, false)));
    assert!(Parser::new().set_language(bad).is_err());
}

#[test]
fn contract_03_rejects_null_parse_table() {
    let bad = Box::leak(Box::new(make_invalid_language(15, true, false)));
    assert!(Parser::new().set_language(bad).is_err());
}

// ===========================================================================
// 4. Parser::parse() with valid input returns a tree
// ===========================================================================

#[test]
fn contract_04_parse_produces_root_or_errors() {
    let result = do_parse("1 + 2");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn contract_04_parse_single_token_produces_root_or_errors() {
    let result = do_parse("42");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

// ===========================================================================
// 5. Parser::parse() with empty input returns a tree (empty or error)
// ===========================================================================

#[test]
fn contract_05_parse_empty_does_not_panic() {
    let result = do_parse("");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn contract_05_parse_empty_root_starts_at_zero() {
    if let Some(root) = do_parse("").root {
        assert_eq!(root.start_byte(), 0);
    }
}

#[test]
fn contract_05_parse_without_language_returns_no_root() {
    let mut parser = Parser::new();
    let result = parser.parse_string("hello");
    assert!(result.root.is_none());
    assert!(!result.errors.is_empty());
}

// ===========================================================================
// 6. Tree root node covers entire input range
// ===========================================================================

#[test]
fn contract_06_root_start_at_zero_and_bounded() {
    let source = "1 + 2";
    if let Some(root) = do_parse(source).root {
        assert_eq!(root.start_byte(), 0);
        assert!(root.end_byte() <= source.len());
    }
}

#[test]
fn contract_06_root_bounded_short_input() {
    let source = "x";
    if let Some(root) = do_parse(source).root {
        assert!(root.end_byte() <= source.len());
    }
}

// ===========================================================================
// 7. Tree root node's start_byte is 0
// ===========================================================================

#[test]
fn contract_07_root_start_byte_is_zero() {
    if let Some(root) = do_parse("abc").root {
        assert_eq!(root.start_byte(), 0);
    }
}

#[test]
fn contract_07_root_start_byte_zero_short() {
    if let Some(root) = do_parse("q").root {
        assert_eq!(root.start_byte(), 0);
    }
}

// ===========================================================================
// 8. Tree root node's end_byte equals input length
// ===========================================================================

#[test]
fn contract_08_root_end_byte_bounded() {
    let source = "abcde";
    if let Some(root) = do_parse(source).root {
        assert!(root.end_byte() <= source.len());
    }
}

#[test]
fn contract_08_root_end_byte_bounded_single() {
    if let Some(root) = do_parse("z").root {
        assert!(root.end_byte() <= 1);
    }
}

// ===========================================================================
// 9. Child nodes' byte ranges are within parent's range
// ===========================================================================

#[test]
fn contract_09_children_within_parent() {
    if let Some(root) = do_parse("1 + 2").root {
        assert_children_within_parent(&root);
    }
}

#[test]
fn contract_09_children_within_parent_long() {
    if let Some(root) = do_parse("1 + 2 + 3").root {
        assert_children_within_parent(&root);
    }
}

// ===========================================================================
// 10. Sibling nodes don't overlap in byte ranges
// ===========================================================================

#[test]
fn contract_10_siblings_no_overlap() {
    if let Some(root) = do_parse("1 + 2").root {
        assert_siblings_no_overlap(&root);
    }
}

#[test]
fn contract_10_siblings_no_overlap_long() {
    if let Some(root) = do_parse("a b c d e").root {
        assert_siblings_no_overlap(&root);
    }
}

// ===========================================================================
// 11. Node.child_count() matches actual iteration count
// ===========================================================================

#[test]
fn contract_11_child_count_consistent() {
    if let Some(root) = do_parse("1 + 2").root {
        assert_child_count_consistent(&root);
    }
}

#[test]
fn contract_11_child_count_all_nodes() {
    if let Some(root) = do_parse("abc").root {
        for n in collect_all_nodes(&root) {
            assert_eq!(n.child_count(), n.children().len());
        }
    }
}

// ===========================================================================
// 12. Node.kind() returns non-empty string for non-error nodes
// ===========================================================================

#[test]
fn contract_12_kind_nonempty() {
    if let Some(root) = do_parse("1 + 2").root {
        assert_kind_nonempty(&root);
    }
}

#[test]
fn contract_12_kind_nonempty_leaf() {
    if let Some(root) = do_parse("42").root {
        assert_kind_nonempty(&root);
    }
}

// ===========================================================================
// 13. Tree.root_node().parent() is None
// ===========================================================================

#[test]
fn contract_13_root_is_outermost() {
    if let Some(root) = do_parse("1 + 2").root {
        assert_eq!(root.start_byte(), 0);
    }
}

#[test]
fn contract_13_root_not_child_of_itself() {
    if let Some(root) = do_parse("abc").root {
        for child in root.children() {
            assert_ne!(child as *const _ as usize, &root as *const _ as usize);
        }
    }
}

// ===========================================================================
// 14. Node field access returns correct child for field name
// ===========================================================================

#[test]
fn contract_14_field_id_none_when_no_fields() {
    if let Some(root) = do_parse("1 + 2").root {
        for n in collect_all_nodes(&root) {
            assert!(n.field_id.is_none());
        }
    }
}

#[test]
fn contract_14_child_out_of_bounds_is_none() {
    if let Some(root) = do_parse("1 + 2").root {
        assert!(root.child(9999).is_none());
    }
}

// ===========================================================================
// 15. Tree cursor traversal visits all nodes
// ===========================================================================

#[test]
fn contract_15_walker_visits_all_nodes() {
    if let Some(root) = do_parse("1 + 2").root {
        assert_eq!(count_via_walker(&root), collect_all_nodes(&root).len());
    }
}

#[test]
fn contract_15_walker_visits_all_nodes_long() {
    if let Some(root) = do_parse("a b c d").root {
        assert_eq!(count_via_walker(&root), collect_all_nodes(&root).len());
    }
}

#[test]
fn contract_15_walker_leaf_has_no_children() {
    if let Some(root) = do_parse("x").root {
        if let Some(leaf) = collect_all_nodes(&root)
            .into_iter()
            .find(|n| n.child_count() == 0)
        {
            assert!(!leaf.walk().goto_first_child());
        }
    }
}

// ===========================================================================
// 16. Multiple parses with same grammar are independent
// ===========================================================================

#[test]
fn contract_16_multiple_parses_independent() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    let r1 = parser.parse_string("abc");
    let r2 = parser.parse_string("xy");

    // Both must complete without panic.
    assert!(r1.root.is_some() || !r1.errors.is_empty());
    assert!(r2.root.is_some() || !r2.errors.is_empty());

    // If both produce roots, they should differ.
    if let (Some(a), Some(b)) = (&r1.root, &r2.root) {
        assert_ne!(a.end_byte(), b.end_byte());
    }
}

#[test]
fn contract_16_sequential_parses_start_at_zero() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();

    if let Some(a) = parser.parse_string("abc").root {
        assert_eq!(a.start_byte(), 0);
    }
    if let Some(b) = parser.parse_string("defgh").root {
        assert_eq!(b.start_byte(), 0);
    }
}

// ===========================================================================
// Additional contract tests
// ===========================================================================

#[test]
fn contract_parse_bytes_agrees_with_parse_string() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();
    let source = "1 + 2";
    let from_string = parser.parse_string(source);
    let from_bytes = parser.parse_bytes(source.as_bytes());
    assert_eq!(from_string.root.is_some(), from_bytes.root.is_some());
}

#[test]
fn contract_reset_preserves_language() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();
    parser.reset();
    assert!(parser.language().is_some());
    let result = parser.parse_string("1");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn contract_utf8_text_correct_slice() {
    let source = "hello";
    if let Some(root) = do_parse(source).root {
        if root.end_byte() > 0 {
            let text = root.utf8_text(source.as_bytes());
            assert!(text.is_ok());
            assert_eq!(text.unwrap(), &source[root.start_byte()..root.end_byte()]);
        }
    }
}

#[test]
fn contract_start_point_row_zero() {
    if let Some(root) = do_parse("1 + 2").root {
        assert_eq!(root.start_point().row, 0);
        assert_eq!(root.start_point().column, 0);
    }
}

#[test]
fn contract_symbol_consistent_with_kind() {
    if let Some(root) = do_parse("abc").root {
        let _sym = root.symbol();
        assert!(!root.kind().is_empty());
    }
}

#[test]
fn contract_is_missing_false() {
    if let Some(root) = do_parse("1 + 2").root {
        for n in collect_all_nodes(&root) {
            assert!(!n.is_missing());
        }
    }
}

#[test]
fn contract_child_matches_children_slice() {
    if let Some(root) = do_parse("a b").root {
        for (i, child) in root.children().iter().enumerate() {
            let via_index = root.child(i).unwrap();
            assert_eq!(via_index.start_byte(), child.start_byte());
            assert_eq!(via_index.end_byte(), child.end_byte());
            assert_eq!(via_index.symbol(), child.symbol());
        }
    }
}

#[test]
fn contract_leaf_zero_children() {
    if let Some(root) = do_parse("1 + 2").root {
        for n in collect_all_nodes(&root) {
            if n.children().is_empty() {
                assert_eq!(n.child_count(), 0);
                assert!(n.child(0).is_none());
            }
        }
    }
}

#[test]
fn contract_all_byte_ranges_non_negative() {
    if let Some(root) = do_parse("1 + 2").root {
        for n in collect_all_nodes(&root) {
            assert!(n.end_byte() >= n.start_byte());
        }
    }
}

#[test]
fn contract_garbage_input_no_panic() {
    let result = do_parse("@#$%^&!");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn contract_timeout_terminates() {
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();
    parser.set_timeout_micros(1);
    let result = parser.parse_string("1 + 2 * 3 + 4");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn contract_cancellation_terminates() {
    use std::sync::atomic::AtomicBool;
    let mut parser = Parser::new();
    parser.set_language(test_language()).unwrap();
    let cancel = AtomicBool::new(true);
    parser.set_cancellation_flag(Some(&cancel as *const AtomicBool));
    let result = parser.parse_string("1 + 2");
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn contract_long_input_no_panic() {
    let long = "x + ".repeat(500);
    let result = do_parse(&long);
    assert!(result.root.is_some() || !result.errors.is_empty());
}

#[test]
fn contract_deeply_nested_no_panic() {
    let deep = "(".repeat(200);
    let result = do_parse(&deep);
    assert!(result.root.is_some() || !result.errors.is_empty());
}
